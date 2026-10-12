mod assets;
mod board;
mod cake;
mod camera;
mod game;
mod missile;
mod player;
mod sound;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // after DefaultPlugins, which provides the AssetServer the handles are loaded from
            assets::AssetsPlugin,
            game::GamePlugin,
            board::BoardPlugin,
            player::PlayerPlugin,
            cake::CakePlugin,
            missile::MissilePlugin,
            camera::CameraPlugin,
            ui::UiPlugin,
            sound::SoundPlugin,
        ))
        .run();
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use crate::assets::{Fonts, Scenes};
    use crate::board::Board;
    use crate::game::{Game, GameState};
    use crate::player::Player;

    // the round-setup plugins, headless, with placeholder asset handles; the first update enters `Playing`
    fn round_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Scenes {
                tile: default(),
                player: default(),
                bonus: default(),
                missile: default(),
            })
            .insert_resource(Fonts {
                score: default(),
                game_over: default(),
            })
            .add_plugins((
                crate::game::GamePlugin,
                crate::board::BoardPlugin,
                crate::player::PlayerPlugin,
                crate::ui::UiPlugin,
            ));
        app.update();
        app
    }

    fn players(app: &mut App) -> Vec<Transform> {
        app.world_mut()
            .query_filtered::<&Transform, With<Player>>()
            .iter(app.world())
            .copied()
            .collect()
    }

    fn texts(app: &mut App) -> Vec<String> {
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|text| text.0.clone())
            .collect()
    }

    fn change_state(app: &mut App, state: GameState) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        app.update();
    }

    #[test]
    fn a_round_starts_with_one_player_standing_on_the_board() {
        let mut app = round_app();

        let players = players(&mut app);
        assert_eq!(players.len(), 1);
        let position = players[0].translation;
        let board = app.world().resource::<Board>();
        assert_eq!(
            position.y,
            board.height(position.x as usize, position.z as usize)
        );
        assert_eq!(texts(&mut app), vec!["Sugar Rush: 0".to_string()]);
    }

    #[test]
    fn restarting_clears_the_last_round() {
        let mut app = round_app();
        app.world_mut().resource_mut::<Game>().score = 5;

        change_state(&mut app, GameState::GameOver);
        assert!(players(&mut app).is_empty());

        change_state(&mut app, GameState::Playing);
        assert_eq!(players(&mut app).len(), 1);
        assert_eq!(app.world().resource::<Game>().score, 0);
        assert_eq!(texts(&mut app), vec!["Sugar Rush: 0".to_string()]);
    }
}
