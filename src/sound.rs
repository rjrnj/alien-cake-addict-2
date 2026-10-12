use bevy::prelude::*;

use crate::assets::Music;
use crate::game::GameState;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), play_music)
            .add_systems(OnEnter(GameState::GameOver), play_game_over_sound);
    }
}

// loop the song for as long as the round lasts
fn play_music(mut commands: Commands, music: Res<Music>) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        AudioPlayer(music.game_playing.clone()),
        PlaybackSettings::LOOP,
    ));
}

fn play_game_over_sound(mut commands: Commands, music: Res<Music>) {
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        AudioPlayer(music.game_over.clone()),
    ));
}
