mod arena;
mod camera;
mod opponent;
mod player;

use bevy::prelude::*;

use self::arena::ArenaPlugin;
use self::camera::GameCameraPlugin;
use self::opponent::OpponentPlugin;
use self::player::PlayerPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(GameCameraPlugin);
        app.add_plugin(ArenaPlugin);
        app.add_plugin(PlayerPlugin);
        app.add_plugin(OpponentPlugin);
    }
}
