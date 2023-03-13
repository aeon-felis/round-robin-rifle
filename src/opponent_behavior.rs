use bevy::prelude::*;
use bevy_tnua::TnuaPlatformerControls;
use bevy_turborand::{DelegatedRng, GlobalRng};
use float_ord::FloatOrd;

use crate::crosshair::Aimedatable;
use crate::killing::Killable;
use crate::menu::AppState;
use crate::rifle::{RifleStatus, ShootCommand};
use crate::utils::project_by_normal;

pub struct OpponentBehaviorPlugin;

impl Plugin for OpponentBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (decide_what_to_do, process_behavior)
                .chain()
                .in_set(OnUpdate(AppState::Game)),
        );
    }
}

#[derive(Component, Default, Debug)]
pub enum OpponentBehavior {
    #[default]
    GetRifle,
    FindTarget,
    Shoot {
        rifle: Entity,
    },
    Panic {
        run_from: Vec3,
        run_direction_in_shooter_coord: Vec3,
    },
    HandsUp {
        aimed_at_by: Entity,
    },
    WaitBefore {
        timer: Timer,
        followup: Option<Box<OpponentBehavior>>,
    },
}

impl OpponentBehavior {
    fn wait_before(seconds: f32, followup: OpponentBehavior) -> Self {
        Self::WaitBefore {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            followup: Some(Box::new(followup)),
        }
    }

    fn wait(seconds: f32) -> Self {
        Self::WaitBefore {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            followup: None,
        }
    }
}

fn decide_what_to_do(
    time: Res<Time>,
    rifles_query: Query<(Entity, &RifleStatus, &GlobalTransform)>,
    aimmedatables_query: Query<&Aimedatable>,
    mut opponents_query: Query<(Entity, &mut OpponentBehavior)>,
    mut rng: ResMut<GlobalRng>,
) {
    let Ok((rifle, rifle_status, rifle_transform)) = rifles_query.get_single() else { return };
    let rifle_position = rifle_transform.translation();
    for (entity, mut behavior) in opponents_query.iter_mut() {
        if let OpponentBehavior::WaitBefore { timer, followup } = behavior.as_mut() {
            if timer.tick(time.delta()).finished() {
                if let Some(followup) = followup.take() {
                    *behavior = *followup;
                    continue;
                } // else branch follows through to select a behavior
            } else {
                continue;
            }
        }
        if let RifleStatus::Equiped(holder) = rifle_status {
            #[allow(clippy::collapsible_else_if)]
            if *holder == entity {
                if aimmedatables_query
                    .iter()
                    .any(|aimedatable| aimedatable.aimed_at_by == Some(*holder))
                {
                    *behavior =
                        OpponentBehavior::wait_before(1.0, OpponentBehavior::Shoot { rifle });
                } else if !matches!(*behavior, OpponentBehavior::FindTarget) {
                    *behavior = OpponentBehavior::wait_before(1.0, OpponentBehavior::FindTarget);
                }
            } else {
                if matches!(*behavior, OpponentBehavior::Shoot { .. }) {
                    *behavior = OpponentBehavior::wait_before(1.0, OpponentBehavior::GetRifle);
                } else if let Some(aimed_at_by) = aimmedatables_query
                    .get(entity)
                    .ok()
                    .and_then(|aimedatable| aimedatable.aimed_at_by)
                {
                    *behavior = OpponentBehavior::HandsUp { aimed_at_by };
                } else if let OpponentBehavior::Panic {
                    run_from,
                    run_direction_in_shooter_coord: _,
                } = behavior.as_mut()
                {
                    *run_from = rifle_position;
                } else {
                    if matches!(*behavior, OpponentBehavior::HandsUp { .. }) {
                        *behavior = OpponentBehavior::wait(1.0);
                    } else {
                        *behavior = OpponentBehavior::Panic {
                            run_from: rifle_position,
                            run_direction_in_shooter_coord: {
                                let mut direction =
                                    Quat::from_rotation_y(0.5 * rng.f32_normalized())
                                        .mul_vec3(Vec3::X);
                                if rng.bool() {
                                    direction *= -1.0;
                                }
                                direction
                            },
                        };
                    }
                }
            }
        } else if matches!(*behavior, OpponentBehavior::Shoot { .. }) {
            *behavior = OpponentBehavior::wait_before(1.0, OpponentBehavior::GetRifle);
        } else {
            *behavior = OpponentBehavior::GetRifle;
        }
    }
}

fn process_behavior(
    rifles_query: Query<&GlobalTransform, With<RifleStatus>>,
    mut opponents_query: Query<(
        Entity,
        &OpponentBehavior,
        &GlobalTransform,
        &mut TnuaPlatformerControls,
    )>,
    killables_query: Query<(Entity, &Killable, &GlobalTransform)>,
    transform_query: Query<&GlobalTransform>,
    mut shoot_commands_writer: EventWriter<ShootCommand>,
) {
    let Ok(rifle_transform) = rifles_query.get_single() else { return };
    let rifle_position = rifle_transform.translation();
    for (entity, behavior, transform, mut controls) in opponents_query.iter_mut() {
        match behavior {
            OpponentBehavior::GetRifle => {
                let direction_to_rifle =
                    project_by_normal(rifle_position - transform.translation(), Vec3::Y)
                        .normalize_or_zero();
                controls.desired_forward = direction_to_rifle;
                controls.desired_velocity = direction_to_rifle;
            }
            OpponentBehavior::FindTarget => {
                // Yes, hard-coding the rifle position is bad. It's a game jam, I don't have time
                // to do it properly.
                let rifle_position = transform.transform_point(Vec3::new(0.65, 0.0, 0.0));
                let current_direction = transform.forward();
                controls.desired_velocity = Vec3::ZERO;
                let chosen_killable_direction = {
                    killables_query
                        .iter()
                        .filter_map(|(killables_entity, killable, killable_transform)| {
                            if killables_entity == entity || killable.killed {
                                None
                            } else {
                                let direction_to_killable = project_by_normal(
                                    killable_transform.translation() - rifle_position,
                                    Vec3::Y,
                                )
                                .normalize_or_zero();
                                Some(direction_to_killable)
                            }
                        })
                        .min_by_key(|direction_to_killable| {
                            FloatOrd(current_direction.angle_between(*direction_to_killable))
                        })
                };
                if let Some(direction_to_killable) = chosen_killable_direction {
                    controls.desired_forward = direction_to_killable;
                }
            }
            OpponentBehavior::Shoot { rifle } => {
                shoot_commands_writer.send(ShootCommand {
                    shooter: entity,
                    rifle: *rifle,
                });
            }
            OpponentBehavior::HandsUp { aimed_at_by } => {
                controls.desired_velocity = Vec3::ZERO;
                controls.desired_forward = match transform_query.get(*aimed_at_by) {
                    Ok(aimer_transform) => project_by_normal(
                        aimer_transform.translation() - transform.translation(),
                        Vec3::Y,
                    )
                    .normalize_or_zero(),
                    Err(_) => Vec3::ZERO,
                };
            }
            OpponentBehavior::Panic {
                run_from,
                run_direction_in_shooter_coord,
            } => {
                let direction_from_danger =
                    project_by_normal(transform.translation() - *run_from, Vec3::Y)
                        .normalize_or_zero();
                let transform_from_danger =
                    Transform::default().looking_to(direction_from_danger, Vec3::Y);
                let panic_direction =
                    transform_from_danger.transform_point(*run_direction_in_shooter_coord);
                controls.desired_velocity = panic_direction;
                controls.desired_forward = panic_direction;
            }
            OpponentBehavior::WaitBefore { .. } => {
                controls.desired_velocity = Vec3::ZERO;
                controls.desired_forward = Vec3::ZERO;
            }
        }
    }
}
