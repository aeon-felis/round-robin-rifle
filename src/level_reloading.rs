use bevy::prelude::*;

use crate::menu::AppState;

pub struct LevelReloadingPlugin;

#[derive(Component)]
pub struct CleanOnLevelReload;

#[derive(SystemLabel)]
pub struct LevelPopulationLabel;

impl Plugin for LevelReloadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set({
            SystemSet::on_enter(AppState::LoadLevel)
                .before(LevelPopulationLabel)
                .with_system(clean_entities)
        });
        app.add_system_set({
            SystemSet::on_enter(AppState::LoadLevel)
                .after(LevelPopulationLabel)
                .with_system(move_to_game_state)
        });
    }
}

fn clean_entities(query: Query<Entity, With<CleanOnLevelReload>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn move_to_game_state(mut state: ResMut<State<AppState>>) {
    state.overwrite_set(AppState::Game).unwrap();
}
