use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaFreeFallBehavior, TnuaPlatformerAnimatingOutput, TnuaPlatformerBundle,
    TnuaPlatformerConfig,
};

use crate::animation::{GltfSceneHandler, HumanAnimationState};
use crate::bumpin::BumpStatus;
use crate::collision_groups;
use crate::crosshair::{Aimedatable, Intimidatable};
use crate::killing::Killable;
use crate::level_reloading::{CleanOnLevelReload, LevelPopulationSet};
use crate::menu::AppState;
use crate::opponent_behavior::OpponentBehavior;
use crate::rifle::{AimElevation, RifleHolder};

pub struct OpponentPlugin;

impl Plugin for OpponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system({
            setup_opponents
                .in_schedule(OnEnter(AppState::LoadLevel))
                .in_set(LevelPopulationSet)
        });
    }
}

fn setup_opponents(mut commands: Commands, asset_server: Res<AssetServer>) {
    // const NUM_OPPONENTS: usize = 8;
    const NUM_OPPONENTS: usize = 1;
    for idx in 0..NUM_OPPONENTS {
        let angle = idx as f32 * 2.0 * PI / NUM_OPPONENTS as f32;
        let position = Quat::from_rotation_y(angle).mul_vec3(Vec3::X * 20.0);
        let mut cmd = commands.spawn_empty();
        cmd.insert(CleanOnLevelReload);
        cmd.insert(SceneBundle {
            scene: asset_server.load("human.glb#Scene0"),
            transform: Transform::from_xyz(position.x, 2.0, position.y),
            ..Default::default()
        });
        cmd.insert(GltfSceneHandler {
            names_from: asset_server.load("human.glb"),
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
                full_speed: 20.0,
                full_jump_height: 4.0,
                up: Vec3::Y,
                forward: -Vec3::Z,
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
                turning_angvel: 5.0,
            },
        ));
        cmd.insert(TnuaPlatformerAnimatingOutput::default());
        cmd.insert(TnuaAnimatingState::<HumanAnimationState>::default());

        cmd.insert(BumpStatus::default());
        cmd.insert(RifleHolder::NoRifle);
        cmd.insert(AimElevation(0.0));
        cmd.insert(Killable { killed: false });
        cmd.insert(Intimidatable);
        cmd.insert(Aimedatable::default());
        cmd.insert(OpponentBehavior::default());
    }
}
