use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_turborand::prelude::*;

use crate::arena::Ground;
use crate::collision_groups;
use crate::level_reloading::{CleanOnLevelReload, LevelPopulationSet};
use crate::menu::AppState;
use crate::player::PlayerControlsSet;
use crate::utils::entities_ordered_by_type;

pub struct RiflePlugin;

impl Plugin for RiflePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShootCommand>();
        app.add_system({
            setup_rifle
                .in_schedule(OnEnter(AppState::LoadLevel))
                .in_set(LevelPopulationSet)
        });

        app.add_systems(
            (handle_rifle_collisions, pose_rifle, update_rifle_elevation)
                .in_set(OnUpdate(AppState::Game)),
        );
        app.add_system(
            handle_shooting
                .after(PlayerControlsSet)
                .in_set(OnUpdate(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct AimElevation(pub f32);

#[derive(Component)]
pub enum RifleHolder {
    NoRifle,
    HasRifle(Entity),
}

#[derive(Component)]
pub enum RifleStatus {
    Ragdoll,
    Floating,
    Equiped(Entity),
    Cooldown(Timer),
}

pub struct ShootCommand {
    pub rifle: Entity,
}

fn setup_rifle(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(CleanOnLevelReload);
    cmd.insert(SceneBundle {
        scene: asset_server.load("rifle.glb#Scene0"),
        transform: Transform::from_xyz(-2.0, 1.0, 5.0),
        ..Default::default()
    });

    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Velocity::default());
    cmd.insert(Collider::cuboid(0.1, 0.1, 1.5));
    cmd.insert(ActiveEvents::COLLISION_EVENTS);
    cmd.insert(SolverGroups {
        memberships: collision_groups::WEAPON,
        filters: collision_groups::GENERAL,
    });

    cmd.insert(RifleStatus::Ragdoll);
}

fn handle_rifle_collisions(
    mut reader: EventReader<CollisionEvent>,
    mut rifles_query: Query<&mut RifleStatus>,
    ground_query: Query<&Ground>,
    mut rifle_holder_query: Query<&mut RifleHolder>,
    mut commands: Commands,
) {
    for event in reader.iter() {
        let CollisionEvent::Started(e1, e2, _) = event else { continue };
        let Some([rifle, other]) = entities_ordered_by_type!([*e1, *e2], rifles_query) else { continue };

        let mut rifle_status = rifles_query.get_mut(rifle).unwrap();
        if matches!(*rifle_status, RifleStatus::Equiped(_)) {
            continue;
        }
        if ground_query.contains(other) {
            *rifle_status = RifleStatus::Floating;
        } else if let Ok(mut rifle_holder) = rifle_holder_query.get_mut(other) {
            if !matches!(*rifle_status, RifleStatus::Cooldown(_))
                && matches!(*rifle_holder, RifleHolder::NoRifle)
            {
                *rifle_status = RifleStatus::Equiped(other);
                *rifle_holder = RifleHolder::HasRifle(rifle);
                let joint = FixedJointBuilder::new().local_anchor1(Vec3::new(1.2, 0.0, 0.0));
                commands
                    .entity(rifle)
                    .insert(ImpulseJoint::new(other, joint));
            }
        }
    }
}

fn pose_rifle(
    time: Res<Time>,
    mut rifles_query: Query<(&mut RifleStatus, &GlobalTransform, &mut Velocity)>,
) {
    if time.delta_seconds() == 0.0 {
        return;
    }
    for (mut rifle_status, transform, mut velocity) in rifles_query.iter_mut() {
        match rifle_status.as_mut() {
            RifleStatus::Ragdoll => {
                continue;
            }
            RifleStatus::Floating => {
                let (_, _, translation) = transform.to_scale_rotation_translation();
                let desired_height = 2.0;
                let one_frame_velocity =
                    Vec3::Y * (desired_height - translation.y) / time.delta_seconds();
                let desired_velocitry = one_frame_velocity.clamp_length_max(5.0);
                velocity.linvel = desired_velocitry;

                // TODO: make it spin?
            }
            RifleStatus::Equiped(_holder) => {
                // TODO: move the rifle up or down according to aim
            }
            RifleStatus::Cooldown(timer) => {
                if timer.tick(time.delta()).finished() {
                    *rifle_status = RifleStatus::Ragdoll;
                }
            }
        }
    }
}

fn update_rifle_elevation(
    holders_query: Query<(&RifleHolder, &AimElevation)>,
    mut rifles_query: Query<&mut ImpulseJoint>,
) {
    for (rifle_holder, AimElevation(aim_elevation)) in holders_query.iter() {
        let RifleHolder::HasRifle(rifle) = rifle_holder else { continue };
        let Ok(mut joint) = rifles_query.get_mut(*rifle) else { continue };
        joint
            .data
            .set_local_basis1(Quat::from_rotation_x(*aim_elevation));
    }
}

fn handle_shooting(
    mut reader: EventReader<ShootCommand>,
    mut rifles_query: Query<(&mut RifleStatus, &GlobalTransform, &mut Velocity)>,
    mut holders_query: Query<&mut RifleHolder>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    for ShootCommand { rifle } in reader.iter() {
        let Ok((mut rifle_status, transform, mut velocity)) = rifles_query.get_mut(*rifle) else { continue };

        commands.entity(*rifle).remove::<ImpulseJoint>();

        let RifleStatus::Equiped(holder_entity) = *rifle_status else { continue };

        *rifle_status = RifleStatus::Cooldown(Timer::from_seconds(1.0, TimerMode::Once));
        if let Ok(mut rifle_holder) = holders_query.get_mut(holder_entity) {
            *rifle_holder = RifleHolder::NoRifle;
        } else {
            warn!("No rifle holder for {:?}", rifle);
        }

        let (_, rotation, _) = transform.to_scale_rotation_translation();
        velocity.linvel = rotation.mul_vec3(Vec3::new(10.0 * rng.f32_normalized(), 10.0, 20.0));
        velocity.angvel = Quat::from_axis_angle(rotation.mul_vec3(Vec3::X), -PI).xyz() * 5.0;
    }
}
