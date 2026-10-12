use std::f32::consts::PI;

use bevy::prelude::*;

use crate::assets::Scenes;
use crate::board::{BOARD_SIZE_I, BOARD_SIZE_J, Board, on_board};
use crate::game::{GameState, StartRound};

const PLAYER_START_I: usize = BOARD_SIZE_I / 2;
const PLAYER_START_J: usize = BOARD_SIZE_J / 2;
const PLAYER_MOVE_COOLDOWN_SECONDS: f32 = 0.3;

// (key, step along x, step along z, facing), in priority order
const DIRECTIONS: [(KeyCode, i32, i32, f32); 4] = [
    (KeyCode::ArrowUp, 1, 0, -PI / 2.),
    (KeyCode::ArrowDown, -1, 0, PI / 2.),
    (KeyCode::ArrowRight, 0, 1, PI),
    (KeyCode::ArrowLeft, 0, -1, 0.0),
];

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player.after(StartRound))
            .add_systems(Update, move_player.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Default, Component)]
pub struct Player;

#[derive(Component, Deref, DerefMut)]
struct PlayerMoveCooldown(Timer);

fn spawn_player(mut commands: Commands, scenes: Res<Scenes>, board: Res<Board>) {
    commands.spawn((
        Player,
        DespawnOnExit(GameState::Playing),
        Transform {
            translation: Vec3::new(
                PLAYER_START_I as f32,
                board.height(PLAYER_START_I, PLAYER_START_J),
                PLAYER_START_J as f32,
            ),
            rotation: Quat::from_rotation_y(-PI / 2.),
            ..default()
        },
        WorldAssetRoot(scenes.player.clone()),
        PlayerMoveCooldown(Timer::from_seconds(
            PLAYER_MOVE_COOLDOWN_SECONDS,
            TimerMode::Once,
        )),
    ));
}

fn move_player(
    mut player_transform: Single<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut move_cooldown: Single<&mut PlayerMoveCooldown>,
    board: Res<Board>,
) {
    if !move_cooldown.tick(time.delta()).is_finished() {
        return;
    }

    let pressed = DIRECTIONS
        .iter()
        .filter(|(key, ..)| keyboard_input.pressed(*key))
        .map(|&(_, dx, dz, facing)| (dx, dz, facing));
    let Some((new_x, new_z, facing)) = choose_move(
        player_transform.translation.x as i32,
        player_transform.translation.z as i32,
        pressed,
    ) else {
        return;
    };

    move_cooldown.reset();
    **player_transform = Transform {
        // stand on the surface of the destination tile
        translation: Vec3::new(
            new_x as f32,
            board.height(new_x as usize, new_z as usize),
            new_z as f32,
        ),
        rotation: Quat::from_rotation_y(facing),
        ..default()
    };
}

/// The cell and facing the player moves to from (`x`, `z`), given the pressed arrows as
/// (step along x, step along z, facing) in priority order, or `None` if nothing is pressed.
///
/// One direction per move, so held arrows can't combine into a diagonal step. The first pressed
/// arrow that isn't blocked by the board edge wins, so you can slide along walls. If every pressed
/// arrow is blocked, the player stays put but turns to face the first one.
fn choose_move(
    x: i32,
    z: i32,
    mut pressed: impl Iterator<Item = (i32, i32, f32)> + Clone,
) -> Option<(i32, i32, f32)> {
    let (_, _, first_facing) = pressed.clone().next()?;
    let step = pressed.find_map(|(dx, dz, facing)| {
        let (new_x, new_z) = (x + dx, z + dz);
        on_board(new_x, new_z).then_some((new_x, new_z, facing))
    });
    Some(step.unwrap_or((x, z, first_facing)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAST_I: i32 = BOARD_SIZE_I as i32 - 1;
    const LAST_J: i32 = BOARD_SIZE_J as i32 - 1;

    // the move `move_player` makes with exactly `keys` held
    fn move_with(x: i32, z: i32, keys: &[KeyCode]) -> Option<(i32, i32, f32)> {
        let pressed = DIRECTIONS
            .iter()
            .filter(|(key, ..)| keys.contains(key))
            .map(|&(_, dx, dz, facing)| (dx, dz, facing));
        choose_move(x, z, pressed)
    }

    #[test]
    fn no_key_means_no_move() {
        assert_eq!(move_with(5, 5, &[]), None);
    }

    #[test]
    fn each_arrow_moves_one_cell() {
        assert_eq!(move_with(5, 5, &[KeyCode::ArrowUp]), Some((6, 5, -PI / 2.)));
        assert_eq!(
            move_with(5, 5, &[KeyCode::ArrowDown]),
            Some((4, 5, PI / 2.))
        );
        assert_eq!(move_with(5, 5, &[KeyCode::ArrowRight]), Some((5, 6, PI)));
        assert_eq!(move_with(5, 5, &[KeyCode::ArrowLeft]), Some((5, 4, 0.0)));
    }

    #[test]
    fn two_arrows_never_move_diagonally() {
        assert_eq!(
            move_with(5, 5, &[KeyCode::ArrowUp, KeyCode::ArrowRight]),
            Some((6, 5, -PI / 2.))
        );
    }

    #[test]
    fn opposite_arrows_still_move() {
        assert_eq!(
            move_with(5, 5, &[KeyCode::ArrowUp, KeyCode::ArrowDown]),
            Some((6, 5, -PI / 2.))
        );
    }

    #[test]
    fn a_blocked_arrow_lets_the_next_one_move() {
        assert_eq!(
            move_with(LAST_I, 5, &[KeyCode::ArrowUp, KeyCode::ArrowRight]),
            Some((LAST_I, 6, PI))
        );
    }

    #[test]
    fn when_every_arrow_is_blocked_the_player_turns_but_stays() {
        assert_eq!(
            move_with(LAST_I, LAST_J, &[KeyCode::ArrowUp, KeyCode::ArrowRight]),
            Some((LAST_I, LAST_J, -PI / 2.))
        );
        assert_eq!(move_with(0, 0, &[KeyCode::ArrowLeft]), Some((0, 0, 0.0)));
    }
}
