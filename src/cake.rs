use bevy::prelude::*;

use crate::assets::{Fonts, Scenes};
use crate::board::{Board, random_cell_except};
use crate::game::{Game, GameState, Random};
use crate::player::Player;

const BONUS_SPAWN_SECONDS: f32 = 5.0;
const BONUS_HOVER_HEIGHT: f32 = 0.2;
const BONUS_LIGHT_INTENSITY: f32 = 500_000.0;
const CAKE_POINTS: i32 = 2;
const MISSED_CAKE_PENALTY: i32 = 3;
const GAME_OVER_SCORE: i32 = -5;
// the cake pulses more the further the score is from zero, up to doubling in size
const BONUS_PULSE_PER_POINT: f32 = 0.1;
const BONUS_MAX_PULSE: f32 = 1.0;

// floating "+N" text shown when a cake is eaten
const BONUS_VFX_SECONDS: f32 = 3.0;
const BONUS_VFX_RISE_SPEED: f32 = 75.0;
const BONUS_VFX_SWAY: f32 = 35.0;

pub struct CakePlugin;

impl Plugin for CakePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BonusSpawnTimer(Timer::from_seconds(
            BONUS_SPAWN_SECONDS,
            TimerMode::Repeating,
        )))
        .add_systems(OnEnter(GameState::Playing), reset_bonus_timer)
        .add_systems(
            Update,
            (eat_cake, rotate_bonus, spawn_bonus, play_bonus_vfx)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Bonus;

#[derive(Component)]
struct BonusVfxMarker {
    timer: Timer,
    x: f32,
    y: f32,
}

#[derive(Resource, Deref, DerefMut)]
struct BonusSpawnTimer(Timer);

// start every round with a full spawn interval, not what was left from the last round
fn reset_bonus_timer(mut timer: ResMut<BonusSpawnTimer>) {
    timer.reset();
}

fn eat_cake(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut game: ResMut<Game>,
    fonts: Res<Fonts>,
    player_query: Query<&Transform, With<Player>>,
    bonus_query: Query<(Entity, &Transform), With<Bonus>>,
) {
    // eat the cake!
    if let Ok((bonus_entity, bonus_transform)) = bonus_query.single()
        && let Ok(player_transform) = player_query.single()
    {
        let player_pos = Vec2::new(
            player_transform.translation.x,
            player_transform.translation.z,
        );
        let bonus_pos = Vec2::new(bonus_transform.translation.x, bonus_transform.translation.z);
        if player_pos != bonus_pos {
            return;
        }
        game.score += CAKE_POINTS;
        game.cake_eaten += 1;

        // Convert the cake's world position to screen space for positioning the "+N" text.
        if let Ok(bonus_position) = camera.0.world_to_viewport(
            camera.1,
            Vec3::new(
                bonus_transform.translation.x,
                bonus_transform.translation.y + 0.2,
                bonus_transform.translation.z,
            ),
        ) {
            commands.spawn((
                DespawnOnExit(GameState::Playing),
                Text::new(format!("+{CAKE_POINTS}")),
                TextFont::from_font_size(25.0).with_font(fonts.score.clone()),
                BonusVfxMarker {
                    timer: Timer::from_seconds(BONUS_VFX_SECONDS, TimerMode::Once),
                    x: bonus_position.x,
                    y: bonus_position.y,
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: px(bonus_position.y),
                    left: px(bonus_position.x),
                    ..default()
                },
            ));
        }

        commands.entity(bonus_entity).despawn();
    }
}

// despawn the bonus if there is one, then spawn a new one at a random location
fn spawn_bonus(
    mut commands: Commands,
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut timer: ResMut<BonusSpawnTimer>,
    mut rng: ResMut<Random>,
    scenes: Res<Scenes>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<(Entity, &Bonus)>,
    player_transform: Single<&Transform, With<Player>>,
    board: Res<Board>,
) {
    // make sure we wait enough time before spawning the next cake
    if !timer.tick(time.delta()).is_finished() {
        return;
    }

    if let Ok((entity, _)) = query.single() {
        game.score -= MISSED_CAKE_PENALTY;
        commands.entity(entity).despawn();
        if game.score <= GAME_OVER_SCORE {
            next_state.set(GameState::GameOver);
            return;
        }
    }

    // ensure bonus doesn't spawn on the player
    let (bonus_i, bonus_j) = random_cell_except(
        &mut **rng,
        (
            player_transform.translation.x as usize,
            player_transform.translation.z as usize,
        ),
    );

    commands.spawn((
        Bonus,
        DespawnOnExit(GameState::Playing),
        Transform::from_xyz(
            bonus_i as f32,
            board.height(bonus_i, bonus_j) + BONUS_HOVER_HEIGHT,
            bonus_j as f32,
        ),
        WorldAssetRoot(scenes.bonus.clone()),
        children![(
            PointLight {
                color: Color::srgb(1.0, 1.0, 0.0),
                intensity: BONUS_LIGHT_INTENSITY,
                range: 10.0,
                ..default()
            },
            Transform::from_xyz(0.0, 2.0, 0.0),
        )],
    ));
}

// let the cake turn on itself
fn rotate_bonus(
    game: Res<Game>,
    time: Res<Time>,
    mut transforms: Query<&mut Transform, With<Bonus>>,
) {
    if let Ok(mut cake_transform) = transforms.single_mut() {
        cake_transform.rotate_y(time.delta_secs());
        cake_transform.scale =
            Vec3::splat(1.0 + pulse_amplitude(game.score) * ops::sin(time.elapsed_secs()).abs());
    }
}

/// How much the cake grows at the peak of its pulse, as a fraction of its size.
fn pulse_amplitude(score: i32) -> f32 {
    (score.unsigned_abs() as f32 * BONUS_PULSE_PER_POINT).min(BONUS_MAX_PULSE)
}

// Give the player a visual indication that they actually picked up the cake
// node.top = F(x) => time.delta() * C is a linear function that moves the text up at a consistent speed.
// node.left = G(x) => marker.x + sin(marker.timer.elapsed_secs()) * C is a sine function that gives the text a left-to-right motion.
// All movement happens in Bevy's UI coordinate space, using the Node component's top and left properties.
fn play_bonus_vfx(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut Node, &mut BonusVfxMarker)>,
    window: Single<&Window>,
) {
    for (entity, mut node, mut marker) in query {
        if marker.timer.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        } else {
            // Y movement is driven consistently upwards by delta time
            let y_offset = time.delta_secs() * BONUS_VFX_RISE_SPEED;
            marker.y -= y_offset;
            node.top = px(marker.y);

            // X movement is a sine function of the timer, which gives it a more dynamic, floating look.
            let x_offset = ops::sin(marker.timer.elapsed_secs()) * BONUS_VFX_SWAY;
            let x_pos = marker.x + x_offset;
            if (x_pos > 0.0) && (x_pos < window.width()) {
                node.left = px(x_pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_scales_with_score_and_is_symmetric_around_zero() {
        assert_eq!(pulse_amplitude(0), 0.0);
        assert!((pulse_amplitude(3) - 0.3).abs() < 1e-6);
        assert!((pulse_amplitude(-3) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn pulse_is_capped() {
        assert_eq!(pulse_amplitude(1_000), BONUS_MAX_PULSE);
        assert_eq!(pulse_amplitude(i32::MIN), BONUS_MAX_PULSE);
    }
}
