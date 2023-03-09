use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::TnuaPlatformerConfig;

use crate::menu::AppState;

pub struct BumpinPlugin;

impl Plugin for BumpinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (detect_bumpin, apply_bumpin)
                .chain()
                .in_set(OnUpdate(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct BumpInitiator;

#[derive(Component, Default, Debug)]
pub enum BumpStatus {
    #[default]
    NoBump,
    TriggerBump {
        impulse: Vec3,
    },
    LostFooting {
        timer: Timer,
        acceleration_restoration: AccelerationRestoration,
        air_acceleration_restoration: AccelerationRestoration,
        // orig_accelerations: (f32, f32),
        // acceleration_drops
    },
}

#[derive(Debug)]
pub struct AccelerationRestoration {
    original: f32,
    lowered: f32,
}

impl AccelerationRestoration {
    fn get_and_update(target: &mut f32, lowered: f32) -> Self {
        let result = Self {
            original: *target,
            lowered,
        };
        *target = result.calc(0.0);
        result
    }

    fn calc(&self, percent: f32) -> f32 {
        let delta = self.original - self.lowered;
        (self.lowered + percent * delta).max(0.0)
    }
}

impl BumpStatus {
    fn trigger_bump_if_empty(&mut self, impulse: Vec3) {
        if matches!(self, Self::NoBump) {
            *self = Self::TriggerBump { impulse };
        }
    }
}

fn detect_bumpin(
    mut reader: EventReader<CollisionEvent>,
    initiator_query: Query<&BumpInitiator>,
    mut status_query: Query<&mut BumpStatus>,
    rapier: Res<RapierContext>,
) {
    for event in reader.iter() {
        let CollisionEvent::Started(e1, e2, _flags) = event else { continue };
        // ie stands for "initiator entity", and oe for "other entity". These prefixes will also be
        // used for the various queried components in this system.
        let (ie, oe) = if initiator_query.contains(*e1) {
            (*e1, *e2)
        } else if initiator_query.contains(*e2) {
            (*e2, *e1)
        } else {
            continue;
        };
        let Ok([mut istatus, mut ostatus]) = status_query.get_many_mut([ie, oe]) else { continue };

        let Some(contact_pair) = rapier.contact_pair(ie, oe) else { continue };
        let normal = contact_pair
            .manifolds()
            .map(|manifold| manifold.normal())
            .sum::<Vec3>()
            .normalize_or_zero();
        let normal = if ie == contact_pair.collider1() {
            -normal
        } else {
            normal
        };

        let normal = normal * 20.0;
        istatus.trigger_bump_if_empty(normal);
        ostatus.trigger_bump_if_empty(-normal);
    }
}

fn apply_bumpin(
    mut query: Query<(&mut BumpStatus, &mut Velocity, &mut TnuaPlatformerConfig)>,
    time: Res<Time>,
) {
    for (mut status, mut velocity, mut tnua_config) in query.iter_mut() {
        match status.as_mut() {
            BumpStatus::NoBump => {}
            BumpStatus::TriggerBump { impulse } => {
                velocity.linvel += *impulse;
                *status = BumpStatus::LostFooting {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                    acceleration_restoration: AccelerationRestoration::get_and_update(
                        &mut tnua_config.acceleration,
                        -60.0,
                    ),
                    air_acceleration_restoration: AccelerationRestoration::get_and_update(
                        &mut tnua_config.air_acceleration,
                        -60.0,
                    ),
                }
            }
            BumpStatus::LostFooting {
                timer,
                acceleration_restoration,
                air_acceleration_restoration,
            } => {
                timer.tick(time.delta());
                let percent = timer.percent();
                if timer.finished() {
                    tnua_config.acceleration = acceleration_restoration.original;
                    tnua_config.air_acceleration = air_acceleration_restoration.original;
                    *status = BumpStatus::NoBump;
                } else {
                    tnua_config.acceleration = acceleration_restoration.calc(percent);
                    tnua_config.air_acceleration = air_acceleration_restoration.calc(percent);
                }
            }
        }
    }
}
