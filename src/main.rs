// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_tnua::{TnuaPlatformerPlugin, TnuaRapier3dPlugin};
use round_robin_rifle::GamePlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugin(TnuaRapier3dPlugin);
    app.add_plugin(TnuaPlatformerPlugin);

    app.add_plugin(GamePlugin);

    app.run();
}
