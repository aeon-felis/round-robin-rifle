use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::rifle::ShootCommand;
use crate::ShootingSequenceSet;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(generate_bullet.in_set(ShootingSequenceSet::GenerateBullet));
    }
}

fn generate_bullet(
    mut reader: EventReader<ShootCommand>,
    rifles_query: Query<&GlobalTransform>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for ShootCommand { rifle } in reader.iter() {
        let Ok(rifle_transform) = rifles_query.get(*rifle) else { continue };
        let mut cmd = commands.spawn_empty();
        cmd.insert(SceneBundle {
            scene: asset_server.load("bullet.glb#Scene0"),
            transform: rifle_transform.mul_transform(Transform::from_xyz(0.0, 0.0, -2.0)).into(),
            ..Default::default()
        });

        cmd.insert(RigidBody::KinematicVelocityBased);
        cmd.insert(Collider::capsule_z(0.25, 0.25));
        cmd.insert(Velocity {
            linvel: 100.0 * rifle_transform.forward(),
            angvel: Vec3::ZERO,
        });
    }
}
