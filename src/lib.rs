mod arena;
mod bullet;
mod bumpin;
mod camera;
mod crosshair;
mod level_reloading;
mod menu;
mod opponent;
mod player;
mod rifle;
mod utils;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::RapierConfiguration;

use self::arena::ArenaPlugin;
use self::bullet::BulletPlugin;
use self::bumpin::BumpinPlugin;
use self::camera::GameCameraPlugin;
use self::crosshair::CrosshairPlugin;
use self::level_reloading::LevelReloadingPlugin;
use self::menu::{AppState, MenuPlugin};
use self::opponent::OpponentPlugin;
use self::player::PlayerPlugin;

pub struct GamePlugin;
pub use self::menu::MenuActionForKbgp;
use self::rifle::RiflePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>();
        app.add_plugin(MenuPlugin);
        app.add_plugin(GameCameraPlugin);
        app.add_plugin(ArenaPlugin);
        app.add_plugin(PlayerPlugin);
        app.add_plugin(OpponentPlugin);
        app.add_plugin(RiflePlugin);
        app.add_plugin(LevelReloadingPlugin);
        app.add_plugin(BumpinPlugin);
        app.add_plugin(CrosshairPlugin);
        app.add_plugin(BulletPlugin);

        app.add_system(enable_disable_when_in_game_or_not);

        app.configure_sets(
            (
                ShootingSequenceSet::ShootInitiator,
                ShootingSequenceSet::GenerateBullet,
                ShootingSequenceSet::RifleRecoil,
            )
                .chain()
                .in_set(OnUpdate(AppState::Game)),
        );
    }
}

fn enable_disable_when_in_game_or_not(
    mut already_in_game: Local<Option<bool>>,
    state: Res<State<AppState>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    mut windows_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let in_game = state.0 == AppState::Game;
    if *already_in_game == Some(in_game) {
        return;
    }
    rapier_configuration.physics_pipeline_active = in_game;
    if let Ok(mut window) = windows_query.get_single_mut() {
        window.cursor.grab_mode = if in_game {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        window.cursor.visible = !in_game;
    }
    *already_in_game = Some(in_game);
}

mod collision_groups {
    use bevy_rapier3d::prelude::Group;

    pub const GENERAL: Group = Group::GROUP_1;
    pub const PARTICIPANT: Group = Group::GROUP_2;
    pub const WEAPON: Group = Group::GROUP_3;
}

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ShootingSequenceSet {
    ShootInitiator,
    GenerateBullet,
    RifleRecoil,
}
