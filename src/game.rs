use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

const SEED: u64 = 19878367467713;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<Game>()
            .init_resource::<Random>()
            .add_systems(OnEnter(GameState::Playing), reset_game.in_set(StartRound));
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

/// The `OnEnter(GameState::Playing)` systems that set up a new round's shared state (score, RNG, board).
/// Systems that spawn the round's entities run after this set.
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StartRound;

#[derive(Resource, Default)]
pub struct Game {
    pub score: i32,
    pub cake_eaten: u32,
}

#[derive(Resource, Deref, DerefMut)]
pub struct Random(ChaCha8Rng);

impl Default for Random {
    fn default() -> Self {
        Self(round_rng())
    }
}

fn round_rng() -> ChaCha8Rng {
    if std::env::var("GITHUB_ACTIONS") == Ok("true".to_string()) {
        // We're seeding the PRNG here to make this example deterministic for testing purposes.
        // This isn't strictly required in practical use unless you need your app to be deterministic.
        ChaCha8Rng::seed_from_u64(SEED)
    } else {
        ChaCha8Rng::from_rng(&mut rand::rng())
    }
}

// clear the score and reseed the RNG for a new round
pub fn reset_game(mut game: ResMut<Game>, mut rng: ResMut<Random>) {
    *game = Game::default();
    rng.0 = round_rng();
}
