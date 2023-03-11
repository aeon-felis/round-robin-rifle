use std::time::Duration;

use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_tnua::{TnuaAnimatingState, TnuaPlatformerAnimatingOutput, TnuaSystemSet};

use crate::menu::AppState;

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
    pub owner_entity: Entity,
    pub animations: HashMap<String, Handle<AnimationClip>>,
}

fn animation_patcher_system(
    animation_players_query: Query<Entity, Added<AnimationPlayer>>,
    parents_query: Query<&Parent>,
    scene_handlers_query: Query<&GltfSceneHandler>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for owner_entity in animation_players_query.iter() {
        let mut entity = owner_entity;
        loop {
            if let Ok(GltfSceneHandler { names_from }) = scene_handlers_query.get(entity) {
                let gltf = gltf_assets.get(names_from).unwrap();
                let mut cmd = commands.entity(entity);
                cmd.remove::<GltfSceneHandler>();
                cmd.insert(AnimationsHandler {
                    owner_entity,
                    animations: gltf.named_animations.clone(),
                });
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
}

fn animate(
    mut humans_query: Query<(
        &mut TnuaAnimatingState<HumanAnimationState>,
        &TnuaPlatformerAnimatingOutput,
        &AnimationsHandler,
    )>,
    mut animation_players_query: Query<&mut AnimationPlayer>,
) {
    for (mut animating_state, animation_output, handler) in humans_query.iter_mut() {
        let Ok(mut player) = animation_players_query.get_mut(handler.owner_entity) else { continue} ;
        match animating_state.update_by_discriminant({
            if animation_output.jumping_velocity.is_some() {
                HumanAnimationState::Jumping
            } else {
                let speed = animation_output.running_velocity.length();
                if 0.01 < speed {
                    HumanAnimationState::Running(0.1 * speed)
                } else {
                    HumanAnimationState::Standing
                }
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
            },
        }
    }
}
