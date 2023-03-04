mod arena;
mod camera;
mod level_reloading;
mod menu;
mod opponent;
mod player;

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::RapierConfiguration;

use self::arena::ArenaPlugin;
use self::camera::GameCameraPlugin;
use self::level_reloading::LevelReloadingPlugin;
use self::menu::{AppState, MenuPlugin, MenuState};
use self::opponent::OpponentPlugin;
use self::player::PlayerPlugin;

pub struct GamePlugin;
pub use self::menu::MenuActionForKbgp;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(AppState::Menu(MenuState::Main));
        app.add_plugin(MenuPlugin);
        app.add_plugin(GameCameraPlugin);
        app.add_plugin(ArenaPlugin);
        app.add_plugin(PlayerPlugin);
        app.add_plugin(OpponentPlugin);
        app.add_plugin(LevelReloadingPlugin);

        app.add_system(enable_disable_when_in_game_or_not);
    }
}

fn enable_disable_when_in_game_or_not(
    mut already_in_game: Local<Option<bool>>,
    state: Res<State<AppState>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    mut windows: ResMut<Windows>,
) {
    let in_game = *state.current() == AppState::Game;
    if *already_in_game == Some(in_game) {
        return;
    }
    rapier_configuration.physics_pipeline_active = in_game;
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_grab_mode(if in_game {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        });
        window.set_cursor_visibility(!in_game);
    }
    *already_in_game = Some(in_game);
}
