use bevy::prelude::*;

use crate::assets::Fonts;
use crate::game::{Game, GameState, StartRound};

const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            // after the score has been reset, so the label starts at 0
            spawn_scoreboard.after(StartRound),
        )
        .add_systems(
            Update,
            scoreboard_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), display_game_over)
        .add_systems(
            Update,
            game_over_keyboard.run_if(in_state(GameState::GameOver)),
        );
    }
}

#[derive(Component)]
struct ScoreText;

fn score_label(score: i32) -> String {
    format!("Sugar Rush: {score}")
}

fn spawn_scoreboard(mut commands: Commands, game: Res<Game>, fonts: Res<Fonts>) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            border_radius: BorderRadius::all(px(5)),
            border: UiRect::all(px(2)),
            padding: UiRect::all(px(5)),
            top: px(5),
            left: px(5),
            ..default()
        },
        BorderColor::all(Color::WHITE),
        children![(
            ScoreText,
            Text::new(score_label(game.score)),
            TextFont::from_font_size(33.0).with_font(fonts.score.clone()),
            TextColor(TEXT_COLOR),
        )],
    ));
}

// update the score displayed during the game
fn scoreboard_system(game: Res<Game>, mut display: Single<&mut Text, With<ScoreText>>) {
    display.0 = score_label(game.score);
}

// restart the game when pressing spacebar
fn game_over_keyboard(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

// display the number of cake eaten before losing and instructions to restart the game
fn display_game_over(mut commands: Commands, game: Res<Game>, fonts: Res<Fonts>) {
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Percent(20.),
            ..default()
        },
        children![
            (
                Text::new(format!("Cake eaten: {}", game.cake_eaten)),
                TextFont::from_font_size(67.0).with_font(fonts.game_over.clone()),
                TextColor(TEXT_COLOR),
            ),
            (
                Text::new("Press SPACE to restart"),
                TextFont::from_font_size(23.0).with_font(fonts.game_over.clone()),
                TextColor(TEXT_COLOR),
            )
        ],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    // other `Text` entities, like the floating "+N", must not stop the scoreboard updating
    #[test]
    fn scoreboard_updates_while_other_text_is_on_screen() {
        let mut app = App::new();
        app.insert_resource(Game {
            score: 7,
            cake_eaten: 0,
        })
        .add_systems(Update, scoreboard_system);
        let scoreboard = app
            .world_mut()
            .spawn((ScoreText, Text::new(score_label(0))))
            .id();
        app.world_mut().spawn(Text::new("+2"));

        app.update();

        assert_eq!(
            app.world().get::<Text>(scoreboard).unwrap().0,
            "Sugar Rush: 7"
        );
    }
}
