use bevy::prelude::*;

use crate::rifle::AimElevation;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera);
        app.add_system(update_camera);
    }
}

#[derive(Component)]
pub struct CameraFollow {
    pub direction: Vec3,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn update_camera(
    mut cameras_query: Query<&mut Transform, With<Camera3d>>,
    camera_follow_query: Query<(&CameraFollow, &GlobalTransform, &AimElevation)>,
) {
    let Ok((camera_follow, camera_follow_transform, AimElevation(aim_elevation))) = camera_follow_query.get_single() else { return };
    let sideways = camera_follow.direction.cross(Vec3::Y).normalize_or_zero();
    let object_at = camera_follow_transform.translation();
    let camera_at = object_at - 10.0 * camera_follow.direction + 1.0 * Vec3::Y;
    let mut target_transform =
        Transform::from_translation(camera_at).looking_at(object_at, Vec3::Y);
    target_transform.translation += 1.2 * sideways;
    target_transform.rotate_around(
        0.5 * (object_at + camera_at),
        Quat::from_axis_angle(sideways, *aim_elevation),
    );

    for mut camera_transform in cameras_query.iter_mut() {
        *camera_transform = target_transform;
    }
}
