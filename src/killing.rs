use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_tnua::{TnuaMotor, TnuaPlatformerControls};

use crate::bullet::Bullet;
use crate::bumpin::BumpStatus;
use crate::collision_groups;
use crate::menu::AppState;
use crate::score::ScoreHaver;
use crate::utils::entities_ordered_by_type;

pub struct KillingPlugin;

impl Plugin for KillingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_bullet_hits.in_set(OnUpdate(AppState::Game)));
    }
}

#[derive(Component)]
pub struct Killable {
    pub killed: bool,
}

#[allow(clippy::type_complexity)]
fn handle_bullet_hits(
    mut reader: EventReader<CollisionEvent>,
    bullets_query: Query<&Bullet>,
    mut victims_query: Query<(
        &mut Killable,
        &mut LockedAxes,
        &mut SolverGroups,
        &GlobalTransform,
        &mut Velocity,
    )>,
    mut commands: Commands,
    mut score_havers_query: Query<&mut ScoreHaver>,
    mut state: ResMut<NextState<AppState>>,
) {
    for event in reader.iter() {
        let CollisionEvent::Started(e1, e2, _) = event else { continue };
        let Some([bullet, victim]) = entities_ordered_by_type!([*e1, *e2], bullets_query, victims_query) else { continue };
        let Bullet { shooter } = bullets_query.get(bullet).unwrap();
        if *shooter == victim {
            continue;
        }

        let (mut killable, mut locked_axes, mut solver_groups, transform, mut velocity) =
            victims_query.get_mut(victim).unwrap();
        if killable.killed {
            continue;
        }
        killable.killed = true;
        commands
            .entity(victim)
            .remove::<(TnuaPlatformerControls, TnuaMotor, BumpStatus)>();
        *locked_axes = Default::default();
        solver_groups.filters = collision_groups::GENERAL;
        velocity.linvel = Vec3::Y * 3.0;
        velocity.angvel = Quat::from_axis_angle(transform.right(), 1.0).xyz();

        if let Ok(mut score_haver) = score_havers_query.get_mut(*shooter) {
            score_haver.score += 1;
        }

        let remaining_alive = victims_query
            .iter()
            .filter(|(killable, _, _, _, _)| !killable.killed)
            .count();

        if remaining_alive <= 1 {
            state.set(AppState::GameOver);
        }
    }
}
