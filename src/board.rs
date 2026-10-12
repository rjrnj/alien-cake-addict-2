use bevy::prelude::*;
use rand::{Rng, RngExt};

use crate::assets::Scenes;
use crate::game::{GameState, Random, StartRound, reset_game};

pub const BOARD_SIZE_I: usize = 14;
pub const BOARD_SIZE_J: usize = 21;

const TILE_HEIGHT_VARIATION: f32 = 0.1;
const MAIN_LIGHT_INTENSITY: f32 = 2_000_000.0;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            // the board is drawn from the round's freshly seeded RNG
            spawn_board.in_set(StartRound).after(reset_game),
        );
    }
}

struct Cell {
    height: f32,
}

/// The current round's tiles, indexed by `i` along x and `j` along z.
#[derive(Resource)]
pub struct Board(Vec<Vec<Cell>>);

impl Board {
    fn generate(rng: &mut impl Rng) -> Self {
        Self(
            (0..BOARD_SIZE_J)
                .map(|_| {
                    (0..BOARD_SIZE_I)
                        .map(|_| Cell {
                            height: rng.random_range(-TILE_HEIGHT_VARIATION..TILE_HEIGHT_VARIATION),
                        })
                        .collect()
                })
                .collect(),
        )
    }

    /// Height of the surface of tile (`i`, `j`).
    pub fn height(&self, i: usize, j: usize) -> f32 {
        self.0[j][i].height
    }
}

/// Whether (`i`, `j`) is a cell on the board.
pub fn on_board(i: i32, j: i32) -> bool {
    (0..BOARD_SIZE_I as i32).contains(&i) && (0..BOARD_SIZE_J as i32).contains(&j)
}

/// A uniformly random cell other than `avoid`, chosen with a single draw.
pub fn random_cell_except(rng: &mut impl Rng, avoid: (usize, usize)) -> (usize, usize) {
    debug_assert!(avoid.0 < BOARD_SIZE_I && avoid.1 < BOARD_SIZE_J);
    let avoid_index = avoid.1 * BOARD_SIZE_I + avoid.0;
    // draw from all cells but one, then shift draws at or past the avoided cell up by one
    let mut index = rng.random_range(0..BOARD_SIZE_I * BOARD_SIZE_J - 1);
    if index >= avoid_index {
        index += 1;
    }
    (index % BOARD_SIZE_I, index / BOARD_SIZE_I)
}

// lay out a new board of tiles at random heights, lit by the main light
fn spawn_board(mut commands: Commands, mut rng: ResMut<Random>, scenes: Res<Scenes>) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        PointLight {
            intensity: MAIN_LIGHT_INTENSITY,
            shadow_maps_enabled: true,
            range: 30.0,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0),
    ));

    let board = Board::generate(&mut **rng);
    for j in 0..BOARD_SIZE_J {
        for i in 0..BOARD_SIZE_I {
            commands.spawn((
                DespawnOnExit(GameState::Playing),
                Transform::from_xyz(i as f32, board.height(i, j) - 0.2, j as f32),
                WorldAssetRoot(scenes.tile.clone()),
            ));
        }
    }
    commands.insert_resource(board);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;

    #[test]
    fn generated_board_has_every_cell_within_the_height_variation() {
        let board = Board::generate(&mut ChaCha8Rng::seed_from_u64(1));
        assert_eq!(board.0.len(), BOARD_SIZE_J);
        for (j, row) in board.0.iter().enumerate() {
            assert_eq!(row.len(), BOARD_SIZE_I);
            for i in 0..BOARD_SIZE_I {
                let height = board.height(i, j);
                assert!((-TILE_HEIGHT_VARIATION..TILE_HEIGHT_VARIATION).contains(&height));
            }
        }
    }

    #[test]
    fn on_board_accepts_cells_inside_and_rejects_cells_outside() {
        let (last_i, last_j) = (BOARD_SIZE_I as i32 - 1, BOARD_SIZE_J as i32 - 1);
        assert!(on_board(0, 0));
        assert!(on_board(last_i, last_j));
        assert!(!on_board(-1, 0));
        assert!(!on_board(0, -1));
        assert!(!on_board(last_i + 1, 0));
        assert!(!on_board(0, last_j + 1));
    }

    #[test]
    fn random_cell_except_never_returns_the_avoided_cell() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        for avoid in [(0, 0), (BOARD_SIZE_I - 1, BOARD_SIZE_J - 1), (7, 10)] {
            for _ in 0..2_000 {
                let (i, j) = random_cell_except(&mut rng, avoid);
                assert!(i < BOARD_SIZE_I && j < BOARD_SIZE_J);
                assert_ne!((i, j), avoid);
            }
        }
    }

    #[test]
    fn random_cell_except_can_return_every_other_cell() {
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let seen: HashSet<_> = (0..20_000)
            .map(|_| random_cell_except(&mut rng, (7, 10)))
            .collect();
        assert_eq!(seen.len(), BOARD_SIZE_I * BOARD_SIZE_J - 1);
    }
}
