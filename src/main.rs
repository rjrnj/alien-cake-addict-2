use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::audio::{AudioPlayer, PlaybackMode, Volume};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

// Missile by Jarlan Perez [CC-BY] (https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/1Xid2Qhqn2s)

// Music produced by me, feel free to rip it

const SEED: u64 = 19878367467713;

const BOARD_SIZE_I: usize = 14;
const BOARD_SIZE_J: usize = 21;

const PLAYER_START_I: usize = BOARD_SIZE_I / 2;
const PLAYER_START_J: usize = BOARD_SIZE_J / 2;

const RESET_FOCUS: [f32; 3] = [
    BOARD_SIZE_I as f32 / 2.0,
    0.0,
    BOARD_SIZE_J as f32 / 2.0 - 0.5,
];

const GAME_OVER_DISTANCE: f32 = 1.0;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

struct Cell {
    height: f32,
}

#[derive(Component, Deref, DerefMut)]
struct Board(Vec<Vec<Cell>>);

#[derive(Resource, Deref, DerefMut)]
struct Random(ChaCha8Rng);

#[derive(Component)]
struct Bonus;

#[derive(Component)]
struct BonusVfxMarker {
    timer: Timer,
    x: f32,
    y: f32,
}

#[derive(Resource, Deref, DerefMut)]
struct BonusSpawnTimer(Timer);

#[derive(Deref, DerefMut, Component)]
struct Missile (Timer);

#[derive(Resource, Deref, DerefMut)]
struct MissileSpawnTimer(Timer);

#[derive(Default, Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct PlayerMoveCooldown(Timer);

#[derive(Resource, Default)]
struct Scenes {
    tile: Handle<Scene>,
    player: Handle<Scene>,
    bonus: Handle<Scene>,
    missile: Handle<Scene>,
}

#[derive(Resource, Default)]
struct Music {
    game_playing: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
}

#[derive(Resource, Default)]
struct Fonts {
    score: Handle<Font>,
    game_over: Handle<Font>,
}

#[derive(Resource, Default)]
struct Game {
    score: i32,
    cake_eaten: u32,
    camera_should_focus: Vec3,
    camera_is_focus: Vec3,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Game>()
        .init_resource::<Fonts>()
        .init_resource::<Scenes>()
        .init_resource::<Music>()
        .insert_resource(MissileSpawnTimer(Timer::from_seconds(
            3.0,
            TimerMode::Repeating,
        )))
        .insert_resource(BonusSpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(Update, (
            move_player,
            eat_cake,
            focus_camera,
            rotate_bonus,
            scoreboard_system,
            spawn_bonus,
            play_bonus_vfx,
            spawn_missile,
        )
            .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), display_game_over)
        .add_systems(
            Update, 
            game_over_keyboard.run_if(in_state(GameState::GameOver)),
        )
        .run();
}

impl Game {
    fn reset(&mut self) {
        self.reset_camera_focus();
        self.cake_eaten = 0;
        self.score = 0;
    }

    fn reset_camera_focus(&mut self) {
        self.camera_should_focus = Vec3::from(RESET_FOCUS);
        self.camera_is_focus = self.camera_should_focus;
    }
}

fn setup_camera(mut commands: Commands, mut game: ResMut<Game>) {
    game.reset_camera_focus();
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            -(BOARD_SIZE_I as f32 / 2.0),
            2.0 * BOARD_SIZE_J as f32 / 3.0,
            BOARD_SIZE_J as f32 / 2.0 - 0.5,
        )
        .looking_at(game.camera_is_focus, Vec3::Y),
    ));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    mut scenes: ResMut<Scenes>,
    mut fonts: ResMut<Fonts>,
    mut music: ResMut<Music>,
) {
    let mut rng = if std::env::var("GITHUB_ACTIONS") == Ok("true".to_string()) {
        // We're seeding the PRNG here to make this example deterministic for testing purposes.
        // This isn't strictly required in practical use unless you need your app to be deterministic.
        ChaCha8Rng::seed_from_u64(SEED)
    } else {
        ChaCha8Rng::from_rng(&mut rand::rng())
    };

    game.reset();

    scenes.tile = asset_server.load("models/AlienCake/tile.glb#Scene0");
    scenes.player = asset_server.load("models/AlienCake/alien.glb#Scene0");
    scenes.bonus = asset_server.load("models/AlienCake/cakeBirthday.glb#Scene0");
    scenes.missile = asset_server.load("models/AlienCake/missile.glb#Scene0");

    fonts.score = asset_server.load("fonts/score_font.otf");
    fonts.game_over = asset_server.load("fonts/game_over_font.otf");

    music.game_playing = asset_server.load("audio/song.ogg");
    music.game_over = asset_server.load("audio/game_over.ogg");

    // spawn the main light
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 4.0),
    ));

    // spawn the game board
    let board: Vec<Vec<Cell>> = (0..BOARD_SIZE_J)
        .map(|j| {
            (0..BOARD_SIZE_I)
                .map(|i| {
                    let height = rng.random_range(-0.1..0.1);
                    commands.spawn((
                        DespawnOnExit(GameState::Playing),
                        Transform::from_xyz(i as f32, height - 0.2, j as f32),
                        SceneRoot(scenes.tile.clone()),
                    ));
                    Cell { height }
                })
                .collect()
        })
        .collect();

    let player_start_height = board[PLAYER_START_J][PLAYER_START_I].height;

    commands.spawn((
        Board(board),
        DespawnOnExit(GameState::Playing),
    ));

    // spawn the game character
    commands
        .spawn((
            Player,
            DespawnOnExit(GameState::Playing),
            Transform {
                translation: Vec3::new(
                    PLAYER_START_I as f32,
                    player_start_height,
                    PLAYER_START_J as f32,
                ),
                rotation: Quat::from_rotation_y(-PI / 2.),
                ..default()
            },
            SceneRoot(scenes.player.clone()),
            PlayerMoveCooldown(Timer::from_seconds(0.3, TimerMode::Once)),
        ));

    // scoreboard
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
        BorderColor {
            top: Color::WHITE,
            right: Color::WHITE,
            bottom: Color::WHITE,
            left: Color::WHITE,
        },
        children![(
            Text::new("Score:"),
            TextFont {
                font_size: 33.0,
                font: fonts.score.clone(),
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 1.0)),
        )],
    ));

    // Music loop
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        AudioPlayer(music.game_playing.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(1.0),
            ..default()
        },
    ));

    commands.insert_resource(Random(rng));
}

fn move_player(    
    mut transforms: Query<&mut Transform>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Single<Entity, With<Player>>,
    mut move_cooldown: Single<&mut PlayerMoveCooldown>,
) {
    if move_cooldown.tick(time.delta()).is_finished() {
        let mut moved = false;
        let mut rotation = 0.0;

        if let Ok(player_transform) = transforms.get(player.entity()) {
            let mut player_translation = Vec3::from(player_transform.translation);

            if keyboard_input.pressed(KeyCode::ArrowUp) {
                if player_translation.x < BOARD_SIZE_I as f32 - 1. {
                    player_translation.x += 1.;
                }
                rotation = -PI / 2.;
                moved = true;
            }

            if keyboard_input.pressed(KeyCode::ArrowDown) {
                if player_translation.x > 0. {
                    player_translation.x -= 1.;
                }
                rotation = PI / 2.;
                moved = true;
            }

            if keyboard_input.pressed(KeyCode::ArrowRight) {
                if player_translation.z < BOARD_SIZE_J as f32 - 1. {
                    player_translation.z += 1.;
                }
                rotation = PI;
                moved = true;
            }

            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                if player_translation.z > 0. {
                    player_translation.z -= 1.;
                }
                rotation = 0.0;
                moved = true;
            }
            
            // move on the board
            if moved {
                move_cooldown.reset();
                *transforms.get_mut(player.entity()).unwrap() = Transform {
                    translation: player_translation,
                    rotation: Quat::from_rotation_y(rotation),
                    ..default()
                };
            }
        }
    }
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
        let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);
        let bonus_pos = Vec2::new(bonus_transform.translation.x, bonus_transform.translation.z);
        if player_pos != bonus_pos { return; }
        game.score += 2;
        game.cake_eaten += 1;

        // Convert the cake's world position to screen space for positioning the "+2" text.
        if let Ok(bonus_position) = camera.0.world_to_viewport(
            &camera.1,
            Vec3::new(
                bonus_transform.translation.x,
                bonus_transform.translation.y + 0.2,
                bonus_transform.translation.z,
            )
        ) {
            commands.spawn((
                DespawnOnExit(GameState::Playing),
                Text::new("+2"),
                TextFont {
                    font_size: 25.0,
                    font: fonts.score.clone(),
                    ..default()
                },
                BonusVfxMarker {
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                    x: bonus_position.x,
                    y: bonus_position.y,
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: px(bonus_position.y),
                    left: px(bonus_position.x),
                    ..default()
                }
            ));
        }

        commands.entity(bonus_entity).despawn();
    }
}

// change the focus of the camera
fn focus_camera(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut transforms: ParamSet<(Query<&mut Transform, With<Camera3d>>, Query<&Transform>)>,
    player_query: Query<Entity, With<Player>>,
    bonus_query: Query<Entity, With<Bonus>>,
) {
    const SPEED: f32 = 2.0;
    // if there is both a player and a bonus, target the mid-point of them
    if let Ok(player_entity) = player_query.single() && let Ok(bonus_entity) = bonus_query.single(){
        let transform_query = transforms.p1();
        if let (Ok(player_transform), Ok(bonus_transform)) = (
            transform_query.get(player_entity),
            transform_query.get(bonus_entity),
        ) {
            game.camera_should_focus = player_transform
                .translation
                .lerp(bonus_transform.translation, 0.5);
        }
        // otherwise, if there is only a player, target the player
    } else if let Ok(player_entity) = player_query.single() {
        if let Ok(player_transform) = transforms.p1().get(player_entity) {
            game.camera_should_focus = player_transform.translation;
        }
        // otherwise, target the middle
    } else {
        game.camera_should_focus = Vec3::from(RESET_FOCUS);
    }
    // calculate the camera motion based on the difference between where the camera is looking
    // and where it should be looking; the greater the distance, the faster the motion;
    // smooth out the camera movement using the frame time
    let mut camera_motion = game.camera_should_focus - game.camera_is_focus;
    if camera_motion.length() > 0.2 {
        camera_motion *= SPEED * time.delta_secs();
        // set the new camera's actual focus
        game.camera_is_focus += camera_motion;
    }
    // look at that new camera's actual focus
    for mut transform in transforms.p0().iter_mut() {
        *transform = transform.looking_at(game.camera_is_focus, Vec3::Y);
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
    board: Single<&Board>,
) {
    // make sure we wait enough time before spawning the next cake
    if !timer.tick(time.delta()).is_finished() {
        return;
    }

    if let Ok((entity, _)) = query.single() {
        game.score -= 3;
        commands.entity(entity).despawn();
        if game.score <= -5 {
            next_state.set(GameState::GameOver);
            return;
        }
    }

    // ensure bonus doesn't spawn on the player
    // q: is this good or is there a better way?
    let mut bonus_i;
    let mut bonus_j;
    loop {
        bonus_i = rng.random_range(0..BOARD_SIZE_I);
        bonus_j = rng.random_range(0..BOARD_SIZE_J);
        if bonus_i != player_transform.translation.x as usize || bonus_j != player_transform.translation.z as usize {
            break;
        }
    }
    
    commands.spawn((
        Bonus,
        DespawnOnExit(GameState::Playing),
        Transform::from_xyz(
            bonus_i as f32,
            board[bonus_j][bonus_i].height + 0.2,
            bonus_j as f32,
        ),
        SceneRoot(scenes.bonus.clone()),
        children![(
            PointLight {
                color: Color::srgb(1.0, 1.0, 0.0),
                intensity: 500_000.0,
                range: 10.0,
                ..default()
            },
            Transform::from_xyz(0.0, 2.0, 0.0),
        )],
    ));
}

// let the cake turn on itself
fn rotate_bonus(game: Res<Game>, time: Res<Time>, mut transforms: Query<&mut Transform, With<Bonus>>,) {
    if let Ok(mut cake_transform) = transforms.single_mut(){
        cake_transform.rotate_y(time.delta_secs());
        cake_transform.scale =
            Vec3::splat(1.0 + (game.score as f32 / 10.0 * ops::sin(time.elapsed_secs())).abs());
    }
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
            let y_offset = time.delta_secs() * 75.0;
            marker.y -= y_offset;
            node.top = px(marker.y);

            // X movement is a sine function of the timer, which gives it a more dynamic, floating look.
            let x_offset = ops::sin(marker.timer.elapsed_secs()) * 35.0;
            let x_pos = marker.x + x_offset;
            if (x_pos > 0.0) && (x_pos < window.width()) {
                node.left = px(x_pos);
            }
        }
    }
}

// Spawn a missile that flies across the board and despawns after a certain time. If the missile hits the player, it's game over.
fn spawn_missile(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<Random>,
    mut missile_timer: ResMut<MissileSpawnTimer>,
    query: Query<(Entity, &mut Transform, &mut Missile), Without<Player>>,
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
                translation: Vec3::new(BOARD_SIZE_I as f32, 1.0, missile_z),
                rotation: Quat::from_rotation_x(PI / 2.) * Quat::from_rotation_z(PI / 2.),
                scale: Vec3::splat(0.25),
            },
            Missile(Timer::from_seconds(5.0, TimerMode::Once)),
        ));
    }
    // tick the timer of all missiles, and despawn any missile whose timer has finished, and move all other missiles downwards    
    let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.z);
    for (entity, mut transform, mut missile) in query {
        if missile.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation.x -= time.delta_secs() * 10.0;
            // if the missile is close enough, it's game over
            let missile_pos = Vec2::new(transform.translation.x, transform.translation.z);
            if player_pos.distance(missile_pos) <= GAME_OVER_DISTANCE {
                next_state.set(GameState::GameOver);
                break;
            }
        }        
    }
}

// update the score displayed during the game
fn scoreboard_system(game: Res<Game>, mut display: Single<&mut Text>) {
    display.0 = format!("Sugar Rush: {}", game.score);
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
fn display_game_over(mut commands: Commands, game: Res<Game>, fonts: Res<Fonts>, music: Res<Music>) {
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
        children![(
            Text::new(format!("Cake eaten: {}", game.cake_eaten)),
            TextFont {
                font_size: 67.0,
                font: fonts.game_over.clone(),
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 1.0)),
        ),
        (
            Text::new("Press SPACE to restart"),
            TextFont {
                font_size: 23.0,
                font: fonts.game_over.clone(),
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 1.0)),
        )],
    ));

    // Play the game over sound effect
    commands.spawn((
        DespawnOnExit(GameState::GameOver),
        AudioPlayer(music.game_over.clone())
    ));
}
