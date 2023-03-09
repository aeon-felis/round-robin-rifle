use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::rifle::RifleStatus;

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(create_crossair);
        app.add_system(update_crosshairs);
    }
}

#[derive(Component)]
struct Crosshair {
    owner: Entity,
}

fn create_crossair(
    rifles_query: Query<Entity, Added<RifleStatus>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for rifle in rifles_query.iter() {
        let mut cmd = commands.spawn_empty();
        cmd.insert(Crosshair { owner: rifle });
        cmd.insert(SceneBundle {
            scene: asset_server.load("crosshair.glb#Scene0"),
            visibility: Visibility::Hidden,
            ..Default::default()
        });
    }
}

fn update_crosshairs(
    mut crossairs_query: Query<(Entity, &Crosshair, &mut Visibility, &mut Transform)>,
    rifles_query: Query<(&RifleStatus, &GlobalTransform)>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
) {
    for (crosshair_entity, crosshair, mut visibility, mut transform) in crossairs_query.iter_mut() {
        transform.translation = Vec3::new(0.0, 2.0, 0.0);
        if let Ok((rifle_status, rifle_transform)) = rifles_query.get(crosshair.owner) {
            if let RifleStatus::Equiped(_entity) = rifle_status {
                let (_, rifle_rotation, rifle_translation) =
                    rifle_transform.to_scale_rotation_translation();
                if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(
                    rifle_translation,
                    rifle_rotation.mul_vec3(-Vec3::Z),
                    f32::INFINITY,
                    false,
                    QueryFilter::default().exclude_collider(crosshair.owner),
                ) {
                    *visibility = Visibility::Inherited;
                    transform.translation = intersection.point;
                    transform.look_at(rifle_translation, Vec3::Y);
                } else {
                    *visibility = Visibility::Hidden;
                }
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            commands.entity(crosshair_entity).despawn_recursive();
        }
    }
}
