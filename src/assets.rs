use bevy::prelude::*;

// asset credits and licenses are in CREDITS.md

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scenes>()
            .init_resource::<Music>()
            .init_resource::<Fonts>();
    }
}

// Asset handles are loaded in `FromWorld` so they exist before the first `OnEnter(Playing)`,
// which Bevy runs before `Startup`; every round reuses them.
#[derive(Resource)]
pub struct Scenes {
    pub tile: Handle<WorldAsset>,
    pub player: Handle<WorldAsset>,
    pub bonus: Handle<WorldAsset>,
    pub missile: Handle<WorldAsset>,
}

#[derive(Resource)]
pub struct Music {
    pub game_playing: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct Fonts {
    pub score: Handle<Font>,
    pub game_over: Handle<Font>,
}

impl FromWorld for Scenes {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            tile: asset_server.load("models/AlienCake/tile.glb#Scene0"),
            player: asset_server.load("models/AlienCake/alien.glb#Scene0"),
            bonus: asset_server.load("models/AlienCake/cakeBirthday.glb#Scene0"),
            missile: asset_server.load("models/AlienCake/missile.glb#Scene0"),
        }
    }
}

impl FromWorld for Music {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            game_playing: asset_server.load("audio/song.ogg"),
            game_over: asset_server.load("audio/game_over.ogg"),
        }
    }
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            score: asset_server.load("fonts/score_font.otf"),
            game_over: asset_server.load("fonts/game_over_font.otf"),
        }
    }
}
