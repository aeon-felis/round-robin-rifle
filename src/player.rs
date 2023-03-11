use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaFreeFallBehavior, TnuaPlatformerAnimatingOutput, TnuaPlatformerBundle,
    TnuaPlatformerConfig, TnuaPlatformerControls,
};
use leafwing_input_manager::prelude::*;

use crate::animation::{GltfSceneHandler, HumanAnimationState};
use crate::bumpin::{BumpInitiator, BumpStatus};
use crate::camera::CameraFollow;
use crate::killing::Killable;
use crate::level_reloading::{CleanOnLevelReload, LevelPopulationSet};
use crate::menu::AppState;
use crate::rifle::{AimElevation, RifleHolder, ShootCommand};
use crate::{collision_groups, ShootingSequenceSet};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.add_system({
            setup_player
                .in_schedule(OnEnter(AppState::LoadLevel))
                .in_set(LevelPopulationSet)
        });
        app.add_system(player_controls.in_set(ShootingSequenceSet::ShootInitiator));
    }
}

#[derive(Actionlike, Clone, Debug)]
enum PlayerAction {
    Run,
    Jump,
    TurnWithMouse,
    TurnWithGamepad,
    Shoot,
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(CleanOnLevelReload);
    cmd.insert(SceneBundle {
        scene: asset_server.load("human.glb#Scene0"),
        transform: Transform::from_xyz(0.0, 2.0, 10.0),
        ..Default::default()
    });
    cmd.insert(GltfSceneHandler {
        names_from: asset_server.load("human.glb"),
    });

    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Velocity::default());
    cmd.insert(Collider::capsule_y(0.5, 1.0));
    cmd.insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z);
    cmd.insert(ActiveEvents::COLLISION_EVENTS);
    cmd.insert(SolverGroups {
        memberships: collision_groups::PARTICIPANT,
        filters: collision_groups::GENERAL | collision_groups::PARTICIPANT,
    });

    cmd.insert(TnuaPlatformerBundle::new_with_config(
        TnuaPlatformerConfig {
            full_speed: 40.0,
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
            turning_angvel: 10.0,
        },
    ));
    cmd.insert(TnuaPlatformerAnimatingOutput::default());
    cmd.insert(TnuaAnimatingState::<HumanAnimationState>::default());

    cmd.insert(BumpInitiator);
    cmd.insert(BumpStatus::default());
    cmd.insert(RifleHolder::NoRifle);
    cmd.insert(AimElevation(0.0));
    cmd.insert(Killable { killed: false });

    cmd.insert(InputManagerBundle::<PlayerAction> {
        action_state: ActionState::default(),
        input_map: {
            let mut input_map = InputMap::default();
            input_map.insert(VirtualDPad::wasd(), PlayerAction::Run);
            input_map.insert(KeyCode::Space, PlayerAction::Jump);
            input_map.insert(DualAxis::mouse_motion(), PlayerAction::TurnWithMouse);
            input_map.insert(MouseButton::Left, PlayerAction::Shoot);
            #[cfg(not(target_arch = "wasm32"))]
            {
                input_map.insert(VirtualDPad::dpad(), PlayerAction::Run);
                input_map.insert(DualAxis::left_stick(), PlayerAction::Run);
                input_map.insert(GamepadButtonType::South, PlayerAction::Jump);
                input_map.insert(GamepadButtonType::LeftTrigger, PlayerAction::Jump);
                input_map.insert(GamepadButtonType::LeftTrigger2, PlayerAction::Jump);
                input_map.insert(DualAxis::right_stick(), PlayerAction::TurnWithGamepad);
                input_map.insert(GamepadButtonType::RightTrigger, PlayerAction::Shoot);
                input_map.insert(GamepadButtonType::RightTrigger2, PlayerAction::Shoot);
            }

            // TODO: remove these before submitting the game:
            input_map.insert(
                VirtualDPad {
                    up: KeyCode::I.into(),
                    down: KeyCode::K.into(),
                    left: KeyCode::J.into(),
                    right: KeyCode::L.into(),
                },
                PlayerAction::TurnWithGamepad,
            );
            input_map.insert(KeyCode::Semicolon, PlayerAction::Shoot);

            input_map
        },
    });

    cmd.insert(CameraFollow {
        direction: -Vec3::Z,
    });
}

#[allow(clippy::type_complexity)]
fn player_controls(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &ActionState<PlayerAction>,
        &mut TnuaPlatformerControls,
        &mut CameraFollow,
        &mut AimElevation,
        &RifleHolder,
    )>,
    mut shoot_commands_writer: EventWriter<ShootCommand>,
) {
    for (entity, action_state, mut controls, mut camera_follow, mut aim_elevation, rifle_holder) in
        query.iter_mut()
    {
        let turn: Vec2 = [
            (Vec2::new(0.1, -0.05), PlayerAction::TurnWithMouse),
            (Vec2::new(2.0, 2.0), PlayerAction::TurnWithGamepad),
        ]
        .into_iter()
        .filter_map(|(factor, action)| {
            let turn = action_state.axis_pair(action)?;
            Some(Vec2::new(factor.x * turn.x(), factor.y * turn.y()))
        })
        .sum();
        let turn_to_direction =
            Quat::from_rotation_y(time.delta_seconds() * -turn.x).mul_vec3(camera_follow.direction);
        camera_follow.direction = turn_to_direction;
        controls.desired_forward = turn_to_direction;

        aim_elevation.0 += time.delta_seconds() * turn.y;
        aim_elevation.0 = aim_elevation.0.clamp(-0.5, 0.5);

        let sideway = camera_follow.direction.cross(Vec3::Y);

        let direction = if let Some(axis_pair) = action_state.clamped_axis_pair(PlayerAction::Run) {
            (axis_pair.x() * sideway + camera_follow.direction * axis_pair.y())
                .clamp_length_max(1.0)
        } else {
            Vec3::ZERO
        };
        controls.desired_velocity = direction;
        controls.jump = {
            let action_data = action_state.action_data(PlayerAction::Jump);
            if action_data.state.pressed() {
                Some(action_data.value)
            } else {
                None
            }
        };

        if action_state.just_pressed(PlayerAction::Shoot) {
            if let RifleHolder::HasRifle(rifle) = rifle_holder {
                shoot_commands_writer.send(ShootCommand {
                    rifle: *rifle,
                    shooter: entity,
                })
            }
        }
    }
}
