use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_arena);
    }
}

#[derive(Component)]
pub struct Ground;

fn setup_arena(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let mut cmd = commands.spawn_empty();
    cmd.insert(PbrBundle {
        mesh: mesh_assets.add(Mesh::from(shape::Plane {
            size: 128.0,
            subdivisions: 0,
        })),
        material: material_assets.add(Color::WHITE.into()),
        ..Default::default()
    });
    cmd.insert(Collider::halfspace(Vec3::Y).unwrap());
    cmd.insert(Ground);

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });
}
