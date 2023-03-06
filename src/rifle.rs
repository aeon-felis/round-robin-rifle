use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::arena::Ground;
use crate::collision_groups;
use crate::level_reloading::{CleanOnLevelReload, LevelPopulationLabel};
use crate::menu::AppState;
use crate::utils::entities_ordered_by_type;

pub struct RiflePlugin;

impl Plugin for RiflePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set({
            SystemSet::on_enter(AppState::LoadLevel)
                .label(LevelPopulationLabel)
                .with_system(setup_rifle)
        });

        app.add_system_set({
            SystemSet::on_update(AppState::Game)
                .with_system(handle_rifle_collisions)
                .with_system(pose_rifle)
        });
    }
}

#[derive(Component)]
pub enum RifleHolder {
    NoRifle,
    HasRifle(Entity),
}

#[derive(Component)]
enum RifleStatus {
    Ragdoll,
    Floating,
    Equiped(Entity),
}

fn setup_rifle(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(CleanOnLevelReload);
    cmd.insert(PbrBundle {
        mesh: mesh_assets.add(Mesh::from(shape::Box {
            min_x: -0.1,
            max_x: 0.1,
            min_y: -0.1,
            max_y: 0.1,
            min_z: -0.5,
            max_z: 0.5,
        })),
        material: material_assets.add(Color::BEIGE.into()),
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
            if matches!(*rifle_holder, RifleHolder::NoRifle) {
                *rifle_status = RifleStatus::Equiped(other);
                *rifle_holder = RifleHolder::HasRifle(rifle);
                let joint = FixedJointBuilder::new().local_anchor1(Vec3::new(-1.2, 0.0, 0.0));
                commands
                    .entity(rifle)
                    .insert(ImpulseJoint::new(other, joint));
            }
        }
    }
}

fn pose_rifle(
    time: Res<Time>,
    mut rifles_query: Query<(&RifleStatus, &GlobalTransform, &mut Velocity)>,
) {
    if time.delta_seconds() == 0.0 {
        return;
    }
    for (rifle_status, transform, mut velocity) in rifles_query.iter_mut() {
        match rifle_status {
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
        }
    }
}
