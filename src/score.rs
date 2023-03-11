use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(show_score);
    }
}

#[derive(Component)]
pub struct ScoreHaver {
    name: String,
    pub score: usize,
}

impl ScoreHaver {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            score: 0,
        }
    }
}

fn show_score(mut egui_context: EguiContexts, score_havers: Query<&ScoreHaver>) {
    let panel = egui::Area::new("score-area").fixed_pos([0.0, 0.0]);
    panel.show(egui_context.ctx_mut(), |ui| {
        for score_haver in score_havers.iter() {
            ui.label(
                egui::RichText::new(format!("{}: {}", score_haver.name, score_haver.score))
                    .strong(),
            );
        }
    });
}
