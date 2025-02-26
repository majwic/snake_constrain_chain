use bevy::{prelude::*, window::PrimaryWindow};

pub struct CameraPlugin {
    pub speed: f32,
}

#[derive(Resource)]
struct CameraSpeed(f32);

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn move_camera(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    speed: Res<CameraSpeed>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero() * speed.0 * time.delta_secs();
    camera.translation += move_delta.extend(0.);
}

pub fn get_cursor_world_position(
    q_windows: &Query<&Window, With<PrimaryWindow>>,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = q_windows.single();
    let (camera, cam_transform) = q_camera.single();

    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(cam_transform, cursor).ok())
        .map(|wp| wp.origin.truncate())
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSpeed(self.speed))
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, move_camera);
    }
}