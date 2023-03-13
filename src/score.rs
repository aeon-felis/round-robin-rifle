use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::killing::Killable;

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

fn show_score(
    mut egui_context: EguiContexts,
    score_havers: Query<&ScoreHaver>,
    opponents_query: Query<&Killable, Without<ScoreHaver>>,
) {
    let mut num_opponents = 0;
    let mut opponents_alive = 0;
    for killable in opponents_query.iter() {
        num_opponents += 1;
        if !killable.killed {
            opponents_alive += 1;
        }
    }
    let panel = egui::Area::new("score-area").fixed_pos([0.0, 0.0]);
    panel.show(egui_context.ctx_mut(), |ui| {
        ui.label(
            egui::RichText::new(format!(
                "Remaining: {} / {}",
                opponents_alive, num_opponents
            ))
            .strong(),
        );
        for score_haver in score_havers.iter() {
            ui.label(
                egui::RichText::new(format!(
                    "Killed by {}: {}",
                    score_haver.name, score_haver.score
                ))
                .strong(),
            );
        }
    });
}
