use bevy::prelude::*;

use crate::board::{BOARD_SIZE_I, BOARD_SIZE_J};
use crate::cake::Bonus;
use crate::game::GameState;
use crate::player::Player;

const RESET_FOCUS: Vec3 = Vec3::new(
    BOARD_SIZE_I as f32 / 2.0,
    0.0,
    BOARD_SIZE_J as f32 / 2.0 - 0.5,
);
const CAMERA_SPEED: f32 = 2.0;
// the camera stops turning once it is this close to its target
const CAMERA_DEADZONE: f32 = 0.2;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFocus>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Playing), reset_camera_focus)
            .add_systems(Update, focus_camera.run_if(in_state(GameState::Playing)));
    }
}

/// Where the camera should be looking, and where it is looking now as it turns towards that.
#[derive(Resource)]
struct CameraFocus {
    should_focus: Vec3,
    is_focus: Vec3,
}

impl Default for CameraFocus {
    fn default() -> Self {
        Self {
            should_focus: RESET_FOCUS,
            is_focus: RESET_FOCUS,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            -(BOARD_SIZE_I as f32 / 2.0),
            2.0 * BOARD_SIZE_J as f32 / 3.0,
            BOARD_SIZE_J as f32 / 2.0 - 0.5,
        )
        .looking_at(RESET_FOCUS, Vec3::Y),
    ));
}

fn reset_camera_focus(mut focus: ResMut<CameraFocus>) {
    *focus = CameraFocus::default();
}

// change the focus of the camera
fn focus_camera(
    time: Res<Time>,
    mut focus: ResMut<CameraFocus>,
    mut transforms: ParamSet<(Query<&mut Transform, With<Camera3d>>, Query<&Transform>)>,
    player_query: Query<Entity, With<Player>>,
    bonus_query: Query<Entity, With<Bonus>>,
) {
    // if there is both a player and a bonus, target the mid-point of them
    if let Ok(player_entity) = player_query.single()
        && let Ok(bonus_entity) = bonus_query.single()
    {
        let transform_query = transforms.p1();
        if let (Ok(player_transform), Ok(bonus_transform)) = (
            transform_query.get(player_entity),
            transform_query.get(bonus_entity),
        ) {
            focus.should_focus = player_transform
                .translation
                .lerp(bonus_transform.translation, 0.5);
        }
        // otherwise, if there is only a player, target the player
    } else if let Ok(player_entity) = player_query.single() {
        if let Ok(player_transform) = transforms.p1().get(player_entity) {
            focus.should_focus = player_transform.translation;
        }
        // otherwise, target the middle
    } else {
        focus.should_focus = RESET_FOCUS;
    }
    // calculate the camera motion based on the difference between where the camera is looking
    // and where it should be looking; the greater the distance, the faster the motion;
    // smooth out the camera movement using the frame time
    let mut camera_motion = focus.should_focus - focus.is_focus;
    if camera_motion.length() > CAMERA_DEADZONE {
        camera_motion *= CAMERA_SPEED * time.delta_secs();
        // set the new camera's actual focus
        focus.is_focus += camera_motion;
    }
    // look at that new camera's actual focus
    for mut transform in transforms.p0().iter_mut() {
        *transform = transform.looking_at(focus.is_focus, Vec3::Y);
    }
}
