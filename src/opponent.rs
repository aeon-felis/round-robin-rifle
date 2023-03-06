use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig};

use crate::bumpin::BumpStatus;
use crate::collision_groups;
use crate::level_reloading::{CleanOnLevelReload, LevelPopulationLabel};
use crate::menu::AppState;
use crate::rifle::RifleHolder;

pub struct OpponentPlugin;

impl Plugin for OpponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set({
            SystemSet::on_enter(AppState::LoadLevel)
                .label(LevelPopulationLabel)
                .with_system(setup_opponents)
        });
    }
}

fn setup_opponents(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(CleanOnLevelReload);
    cmd.insert(PbrBundle {
        mesh: mesh_assets.add(Mesh::from(shape::Capsule {
            radius: 1.0,
            rings: 10,
            depth: 1.0,
            latitudes: 10,
            longitudes: 10,
            uv_profile: shape::CapsuleUvProfile::Fixed,
        })),
        material: material_assets.add(Color::YELLOW.into()),
        transform: Transform::from_xyz(-5.0, 2.0, 0.0),
        ..Default::default()
    });

    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Velocity::default());
    cmd.insert(Collider::capsule_y(0.5, 1.0));
    cmd.insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z);
    cmd.insert(SolverGroups {
        memberships: collision_groups::PARTICIPANT,
        filters: collision_groups::GENERAL | collision_groups::PARTICIPANT,
    });

    cmd.insert(TnuaPlatformerBundle::new_with_config(
        TnuaPlatformerConfig {
            full_speed: 40.0,
            full_jump_height: 4.0,
            up: Vec3::Y,
            forward: Vec3::X,
            float_height: 2.0,
            cling_distance: 1.0,
            spring_strengh: 40.0,
            spring_dampening: 10.0,
            acceleration: 60.0,
            air_acceleration: 20.0,
            coyote_time: 0.15,
            jump_start_extra_gravity: 30.0,
            jump_fall_extra_gravity: 20.0,
            jump_shorten_extra_gravity: 40.0,
            free_fall_behavior: TnuaFreeFallBehavior::LikeJumpShorten,
            tilt_offset_angvel: 0.0,
            tilt_offset_angacl: 0.0,
            turning_angvel: 10.0,
        },
    ));

    cmd.insert(BumpStatus::default());
    cmd.insert(RifleHolder::NoRifle);
}
