use std::f32::consts::PI;

use bevy::prelude::*;
use rand::RngExt;

use crate::assets::Scenes;
use crate::board::{BOARD_SIZE_I, BOARD_SIZE_J};
use crate::game::{GameState, Random};
use crate::player::Player;

const MISSILE_SPAWN_SECONDS: f32 = 3.0;
const MISSILE_SPEED: f32 = 10.0;
const MISSILE_HEIGHT: f32 = 1.0;
const MISSILE_SCALE: f32 = 0.25;
const GAME_OVER_DISTANCE: f32 = 1.0;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MissileSpawnTimer(Timer::from_seconds(
            MISSILE_SPAWN_SECONDS,
            TimerMode::Repeating,
        )))
        .add_systems(OnEnter(GameState::Playing), reset_missile_timer)
        .add_systems(Update, spawn_missile.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct Missile;

#[derive(Resource, Deref, DerefMut)]
struct MissileSpawnTimer(Timer);

// start every round with a full spawn interval, not what was left from the last round
fn reset_missile_timer(mut timer: ResMut<MissileSpawnTimer>) {
    timer.reset();
}

// Spawn a missile that flies across the board and despawns once it has left the board. If the missile hits the player, it's game over.
fn spawn_missile(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<Random>,
    mut missile_timer: ResMut<MissileSpawnTimer>,
    query: Query<(Entity, &mut Transform), (With<Missile>, Without<Player>)>,
    player_transform: Single<&Transform, With<Player>>,
    scenes: Res<Scenes>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if missile_timer.tick(time.delta()).is_finished() {
        // spawn a new one at a random position along the board
        let missile_z = rng.random_range(0.0..BOARD_SIZE_J as f32);
        commands.spawn((
            DespawnOnExit(GameState::Playing),
            SceneRoot(scenes.missile.clone()),
            Transform {
                translation: Vec3::new(BOARD_SIZE_I as f32, MISSILE_HEIGHT, missile_z),
                rotation: Quat::from_rotation_x(PI / 2.) * Quat::from_rotation_z(PI / 2.),
                scale: Vec3::splat(MISSILE_SCALE),
            },
            Missile,
        ));
    }
    // move every missile along -x and despawn the ones that have left the board
    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.z,
    );
    for (entity, mut transform) in query {
        let start_x = transform.translation.x;
        let end_x = start_x - time.delta_secs() * MISSILE_SPEED;
        transform.translation.x = end_x;
        if hits_player(player_pos, start_x, end_x, transform.translation.z) {
            next_state.set(GameState::GameOver);
            break;
        }
        // past the x = 0 edge by more than the hit distance, it can no longer reach the player
        if end_x < -GAME_OVER_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}

/// Whether a missile flying along -x at `z`, from `start_x` to `end_x` this frame, came within
/// [`GAME_OVER_DISTANCE`] of the player. The whole path is tested, so a long frame can't carry a
/// missile through the player.
fn hits_player(player_pos: Vec2, start_x: f32, end_x: f32, z: f32) -> bool {
    let closest = Vec2::new(player_pos.x.clamp(end_x, start_x), z);
    player_pos.distance(closest) <= GAME_OVER_DISTANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: Vec2 = Vec2::new(5.0, 5.0);

    #[test]
    fn a_missile_ending_the_frame_next_to_the_player_hits() {
        assert!(hits_player(PLAYER, 6.5, 5.5, 5.0));
    }

    #[test]
    fn a_long_frame_cannot_carry_a_missile_through_the_player() {
        assert!(hits_player(PLAYER, 10.0, 0.0, 5.0));
    }

    #[test]
    fn a_missile_in_another_lane_misses() {
        assert!(!hits_player(
            PLAYER,
            10.0,
            0.0,
            5.0 + GAME_OVER_DISTANCE + 0.1
        ));
    }

    #[test]
    fn a_missile_before_or_past_the_player_misses() {
        assert!(!hits_player(PLAYER, 10.0, 9.0, 5.0));
        assert!(!hits_player(PLAYER, 3.0, 2.0, 5.0));
    }

    #[test]
    fn a_zero_length_frame_still_tests_the_missile_position() {
        assert!(hits_player(PLAYER, 5.5, 5.5, 5.0));
        assert!(!hits_player(PLAYER, 9.0, 9.0, 5.0));
    }
}
