use std::time::Duration;

use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_tnua::{TnuaAnimatingState, TnuaPlatformerAnimatingOutput, TnuaSystemSet};

use crate::crosshair::{Aimedatable, Intimidatable};
use crate::killing::Killable;
use crate::menu::AppState;
use crate::opponent_behavior::OpponentBehavior;

pub struct GameAnimationPlugin;

impl Plugin for GameAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_patcher_system);
        app.add_system(
            animate
                .in_set(OnUpdate(AppState::Game))
                .after(TnuaSystemSet::Logic),
        );
    }
}

#[derive(Component)]
pub struct GltfSceneHandler {
    pub names_from: Handle<Gltf>,
}

#[derive(Component)]
pub struct AnimationsHandler {
    pub animation_player_entity: Entity,
    pub animations: HashMap<String, Handle<AnimationClip>>,
}

#[derive(Component)]
struct AnimationPlayerAlreadyRegistered;

fn animation_patcher_system(
    animation_players_query: Query<
        Entity,
        (
            With<AnimationPlayer>,
            Without<AnimationPlayerAlreadyRegistered>,
        ),
    >,
    parents_query: Query<&Parent>,
    scene_handlers_query: Query<&GltfSceneHandler>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for animation_player_entity in animation_players_query.iter() {
        let mut entity = animation_player_entity;
        loop {
            if let Ok(GltfSceneHandler { names_from }) = scene_handlers_query.get(entity) {
                let Some(gltf) = gltf_assets.get(names_from) else {
                    warn!("AnimationPlayer was created but the GLTF is not fully loaded yet");
                    break;
                };
                let mut cmd = commands.entity(entity);
                cmd.remove::<GltfSceneHandler>();
                cmd.insert(AnimationsHandler {
                    animation_player_entity,
                    animations: gltf.named_animations.clone(),
                });
                commands
                    .entity(animation_player_entity)
                    .insert(AnimationPlayerAlreadyRegistered);
                break;
            }
            entity = if let Ok(parent) = parents_query.get(entity) {
                **parent
            } else {
                break;
            };
        }
    }
}

pub enum HumanAnimationState {
    Standing,
    Running(f32),
    Jumping,
    Dead,
    HandsUp,
    Panic,
}

#[allow(clippy::type_complexity)]
fn animate(
    mut humans_query: Query<(
        &mut TnuaAnimatingState<HumanAnimationState>,
        &TnuaPlatformerAnimatingOutput,
        &AnimationsHandler,
        &Killable,
        &Aimedatable,
        Option<&Intimidatable>,
        Option<&OpponentBehavior>,
    )>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
) {
    for (
        mut animating_state,
        animation_output,
        handler,
        killable,
        aimedatable,
        intimidatable,
        behavior,
    ) in humans_query.iter_mut()
    {
        let Ok(mut player) = animation_players_query.get_mut(handler.animation_player_entity) else { continue} ;
        match animating_state.update_by_discriminant('state: {
            if killable.killed {
                break 'state HumanAnimationState::Dead;
            }
            if intimidatable.is_some() && aimedatable.aimed_at_by.is_some() {
                break 'state HumanAnimationState::HandsUp;
            }
            if matches!(behavior, Some(OpponentBehavior::Panic { .. })) {
                break 'state HumanAnimationState::Panic;
            }
            if animation_output.jumping_velocity.is_some() {
                break 'state HumanAnimationState::Jumping;
            }
            let speed = animation_output.running_velocity.length();
            if 0.01 < speed {
                break 'state HumanAnimationState::Running(0.1 * speed);
            } else {
                break 'state HumanAnimationState::Standing;
            }
        }) {
            bevy_tnua::TnuaAnimatingStateDirective::Maintain { state } => {
                if let HumanAnimationState::Running(speed) = state {
                    player.set_speed(*speed);
                }
            }
            bevy_tnua::TnuaAnimatingStateDirective::Alter {
                old_state: _,
                state,
            } => match state {
                HumanAnimationState::Standing => {
                    player
                        .play_with_transition(
                            handler.animations["Stand"].clone(),
                            Duration::from_secs_f32(0.1),
                        )
                        .set_speed(1.0)
                        .repeat();
                }
                HumanAnimationState::Running(speed) => {
                    player
                        .play_with_transition(
                            handler.animations["Run"].clone(),
                            Duration::from_secs_f32(0.1),
                        )
                        .set_speed(*speed)
                        .repeat();
                }
                HumanAnimationState::Jumping => {
                    player
                        .play_with_transition(
                            handler.animations["Jump"].clone(),
                            Duration::from_secs_f32(0.1),
                        )
                        .set_speed(1.0);
                }
                HumanAnimationState::Dead => {
                    player
                        .play_with_transition(
                            handler.animations["Dead"].clone(),
                            Duration::from_secs_f32(0.1),
                        )
                        .set_speed(1.0);
                }
                HumanAnimationState::HandsUp => {
                    player
                        .play_with_transition(
                            handler.animations["HandsUp"].clone(),
                            Duration::from_secs_f32(0.1),
                        )
                        .set_speed(1.5);
                }
                HumanAnimationState::Panic => {
                    player
                        .play_with_transition(
                            handler.animations["Panic"].clone(),
                            Duration::from_secs_f32(0.1),
                        )
                        .set_speed(2.0)
                        .repeat();
                }
            },
        }
    }
}
