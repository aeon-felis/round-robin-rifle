use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{
    TnuaFreeFallBehavior, TnuaPlatformerBundle, TnuaPlatformerConfig, TnuaPlatformerControls,
};
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
        app.add_startup_system(setup_player);
        app.add_system(player_controls);
    }
}

#[derive(Actionlike, Clone, Debug)]
enum PlayerAction {
    Run,
    Jump,
}

fn setup_player(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(PbrBundle {
        mesh: mesh_assets.add(Mesh::from(shape::Capsule {
            radius: 1.0,
            rings: 10,
            depth: 1.0,
            latitudes: 10,
            longitudes: 10,
            uv_profile: shape::CapsuleUvProfile::Fixed,
        })),
        material: material_assets.add(Color::ORANGE.into()),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..Default::default()
    });

    cmd.insert(RigidBody::Dynamic);
    cmd.insert(Velocity::default());
    cmd.insert(Collider::capsule_y(0.5, 1.0));
    cmd.insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z);

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

    cmd.insert(InputManagerBundle::<PlayerAction> {
        action_state: ActionState::default(),
        input_map: {
            let mut input_map = InputMap::default();
            input_map.insert(VirtualDPad::wasd(), PlayerAction::Run);
            input_map.insert(VirtualDPad::dpad(), PlayerAction::Run);
            input_map.insert(DualAxis::left_stick(), PlayerAction::Run);
            input_map.insert(KeyCode::Space, PlayerAction::Jump);
            input_map.insert(GamepadButtonType::South, PlayerAction::Jump);
            input_map
        },
    });
}

fn player_controls(mut query: Query<(&ActionState<PlayerAction>, &mut TnuaPlatformerControls)>) {
    for (action_state, mut controls) in query.iter_mut() {
        let direction = if let Some(axis_pair) = action_state.clamped_axis_pair(PlayerAction::Run) {
            Vec3::new(axis_pair.x(), 0.0, -axis_pair.y()).clamp_length_max(1.0)
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
    }
}
