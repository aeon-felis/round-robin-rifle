use bevy::prelude::*;
use bevy_tnua::TnuaPlatformerControls;
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

#[derive(Component, Default)]
pub enum OpponentBehavior {
    #[default]
    GetRifle,
    FindTarget,
    WaitToShoot(Timer),
    Shoot {
        rifle: Entity,
    },
    Panic {
        run_from: Vec3,
    },
}

fn decide_what_to_do(
    time: Res<Time>,
    rifles_query: Query<(Entity, &RifleStatus, &GlobalTransform)>,
    aimmedatables_query: Query<&Aimedatable>,
    mut opponents_query: Query<(Entity, &mut OpponentBehavior)>,
) {
    let Ok((rifle, rifle_status, rifle_transform)) = rifles_query.get_single() else { return };
    let rifle_position = rifle_transform.translation();
    for (entity, mut behavior) in opponents_query.iter_mut() {
        if let RifleStatus::Equiped(holder) = rifle_status {
            if *holder == entity {
                if let OpponentBehavior::WaitToShoot(timer) = behavior.as_mut() {
                    if timer.tick(time.delta()).finished() {
                        *behavior = OpponentBehavior::Shoot { rifle };
                    }
                } else if aimmedatables_query
                    .iter()
                    .any(|aimedatable| aimedatable.aimed_at_by == Some(*holder))
                {
                    *behavior =
                        OpponentBehavior::WaitToShoot(Timer::from_seconds(1.0, TimerMode::Once));
                } else {
                    *behavior = OpponentBehavior::FindTarget;
                }
            } else {
                *behavior = OpponentBehavior::Panic {
                    run_from: rifle_position,
                };
            }
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
            OpponentBehavior::WaitToShoot(_) => {}
            OpponentBehavior::Shoot { rifle } => {
                shoot_commands_writer.send(ShootCommand {
                    shooter: entity,
                    rifle: *rifle,
                });
            }
            OpponentBehavior::Panic { run_from } => {
                let run_direction = project_by_normal(transform.translation() - *run_from, Vec3::Y)
                    .normalize_or_zero();
                controls.desired_velocity = run_direction;
                controls.desired_forward = run_direction;
            }
        }
    }
}
