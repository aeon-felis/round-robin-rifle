use bevy::prelude::*;

use crate::menu::AppState;

pub struct LevelReloadingPlugin;

#[derive(Component)]
pub struct CleanOnLevelReload;

#[derive(SystemSet, Clone, PartialEq, Eq, Debug, Hash)]
pub struct LevelPopulationSet;

impl Plugin for LevelReloadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system({
            clean_entities
                .in_schedule(OnEnter(AppState::LoadLevel))
                .before(LevelPopulationSet)
        });
        app.add_system({
            move_to_game_state
                .in_schedule(OnEnter(AppState::LoadLevel))
                .after(LevelPopulationSet)
        });
    }
}

fn clean_entities(query: Query<Entity, With<CleanOnLevelReload>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn move_to_game_state(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::Game);
}
