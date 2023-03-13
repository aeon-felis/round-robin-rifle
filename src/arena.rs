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
    const HALF_SIDE: f32 = 64.0;

    let mesh = Mesh::from(shape::Box {
        min_x: -HALF_SIDE,
        max_x: HALF_SIDE,
        min_y: -HALF_SIDE,
        max_y: HALF_SIDE,
        min_z: 0.0,
        max_z: 1.0,
    });
    let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
    let mesh = mesh_assets.add(mesh);

    // Ground
    let mut cmd = commands.spawn_empty();
    cmd.insert(PbrBundle {
        mesh: mesh.clone(),
        material: material_assets.add(Color::WHITE.into()),
        transform: Transform::default().looking_at(Vec3::Y, Vec3::Z),
        ..Default::default()
    });
    cmd.insert(collider.clone());
    cmd.insert(Ground);

    // Walls
    for direction in [-Vec3::Z, Vec3::X, Vec3::Z, -Vec3::X] {
        let mut cmd = commands.spawn_empty();
        cmd.insert(PbrBundle {
            mesh: mesh.clone(),
            material: material_assets.add(Color::WHITE.into()),
            transform: Transform::from_translation(HALF_SIDE * direction + HALF_SIDE * Vec3::Y)
                .looking_to(direction, Vec3::Y),
            ..Default::default()
        });
        cmd.insert(collider.clone());
    }

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
