use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_egui_kbgp::prelude::*;

use crate::killing::Killable;
use crate::player::IsPlayer;

#[derive(Clone, PartialEq, Eq)]
pub struct MenuActionForKbgp;

#[derive(States, Default, Clone, Hash, Debug, PartialEq, Eq)]
pub enum AppState {
    #[default]
    MainMenu,
    PauseMenu,
    LoadLevel,
    Game,
    GameOver,
    //LevelCompleted,
    //Editor,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(pause_unpause_game);
        app.add_system(main_menu.in_set(OnUpdate(AppState::MainMenu)));
        app.add_system(pause_menu.in_set(OnUpdate(AppState::PauseMenu)));
        app.add_system(game_over_menu.in_set(OnUpdate(AppState::GameOver)));
    }
}

fn pause_unpause_game(
    mut egui_context: EguiContexts,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match state.0 {
        AppState::MainMenu => {}
        AppState::PauseMenu => {}
        AppState::LoadLevel => {}
        AppState::Game => {
            let egui_context = egui_context.ctx_mut();
            if egui_context.kbgp_user_action() == Some(MenuActionForKbgp) {
                next_state.set(AppState::PauseMenu);
                egui_context.kbgp_clear_input();
            }
        }
        AppState::GameOver => {}
    }
}

fn menu_layout(egui_context: &egui::Context, dlg: impl FnOnce(&mut egui::Ui)) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center);
            ui.with_layout(layout, |ui| {
                ui.add_space(50.0);
                dlg(ui);
            });
        });
}

#[derive(PartialEq)]
enum FocusLabel {
    Start,
    BackToMainMenu,
    Exit,
}

fn main_menu(
    mut egui_context: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: EventWriter<bevy::app::AppExit>,
) {
    menu_layout(egui_context.ctx_mut(), |ui| {
        if ui.kbgp_user_action() == Some(MenuActionForKbgp) {
            ui.kbgp_set_focus_label(FocusLabel::Exit);
        }
        if ui
            .button("Start")
            .kbgp_navigation()
            .kbgp_initial_focus()
            .kbgp_focus_label(FocusLabel::Start)
            .clicked()
        {
            state.set(AppState::LoadLevel);
            ui.kbgp_clear_input();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui
            .button("Exit")
            .kbgp_navigation()
            .kbgp_focus_label(FocusLabel::Exit)
            .clicked()
        {
            exit.send(bevy::app::AppExit);
        }
    });
}

fn pause_menu(
    mut egui_context: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: EventWriter<bevy::app::AppExit>,
) {
    menu_layout(egui_context.ctx_mut(), |ui| {
        if ui
            .button("Resume")
            .kbgp_navigation()
            .kbgp_initial_focus()
            .clicked()
            || ui.kbgp_user_action() == Some(MenuActionForKbgp)
        {
            state.set(AppState::Game);
        }
        if ui.button("Retry").kbgp_navigation().clicked() {
            state.set(AppState::LoadLevel);
        }
        if ui.button("Main Menu").kbgp_navigation().clicked() {
            state.set(AppState::MainMenu);
            ui.kbgp_clear_input();
            ui.kbgp_set_focus_label(FocusLabel::Start);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui
            .button("Exit")
            .kbgp_navigation()
            .kbgp_focus_label(FocusLabel::BackToMainMenu)
            .clicked()
        {
            exit.send(bevy::app::AppExit);
        }
    });
}

fn game_over_menu(
    mut egui_context: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    player_query: Query<&Killable, With<IsPlayer>>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: EventWriter<bevy::app::AppExit>,
) {
    menu_layout(egui_context.ctx_mut(), |ui| {
        ui.label(egui::RichText::new("Game Over!").size(24.0).strong());
        if player_query.iter().any(|killable| killable.killed) {
            ui.label(
                egui::RichText::new("You Died...")
                    .size(24.0)
                    .strong()
                    .color(egui::Color32::RED),
            );
        } else {
            ui.label(
                egui::RichText::new("You Survived!!!")
                    .size(24.0)
                    .strong()
                    .color(egui::Color32::GREEN),
            );
        }
        if ui.button("Retry").kbgp_navigation().clicked() {
            state.set(AppState::LoadLevel);
        }
        if ui.button("Main Menu").kbgp_navigation().clicked() {
            state.set(AppState::MainMenu);
            ui.kbgp_clear_input();
            ui.kbgp_set_focus_label(FocusLabel::Start);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if ui
            .button("Exit")
            .kbgp_navigation()
            .kbgp_focus_label(FocusLabel::BackToMainMenu)
            .clicked()
        {
            exit.send(bevy::app::AppExit);
        }
    });
}
