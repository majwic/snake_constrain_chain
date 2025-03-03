use bevy::{
    color::palettes::basic::PURPLE, 
    prelude::*, 
    window::PrimaryWindow
};

mod camera;
use camera::{
    CameraPlugin,
    get_cursor_world_position,
};

mod chain;
use chain::{
    ChainPlugin, 
    ChainNode, 
    ChainHead, 
    rotate,
    move_chain
};

const CAMERA_SPEED: f32 = 500.;
const SNAKE_SPEED: f32 = 500.;

fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let snake_size = 20.;
    let length = 60;
    
    let mut prev_entity = None;

    for i in (0..length).rev() {
        let mut entity = commands.spawn((
            ChainNode { next: prev_entity, distance: 20.0, angle_threshold: (20.0 as f32).to_radians() },
            Mesh2d(meshes.add(Circle::new(snake_size))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::from_xyz(0., -i as f32 * snake_size, 0.0 - i as f32),
        ));

        if i == 0 {
            entity.insert(ChainHead { target: Vec2::new(0.0, 20.0), direction: Vec2::new(0.0, 1.0) });
        }

        entity.with_children(|parent| {
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(snake_size + 4.))),
                MeshMaterial2d(materials.add(Color::BLACK)),
                Transform::from_xyz(0., 0., -0.1),
            ));
        });

        prev_entity = Some(entity.id());
    }
}

fn move_snakes(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_heads: Query<(&mut Transform, &ChainNode, &mut ChainHead)>,
    q_nodes: Query<(&Transform, &ChainNode), Without<ChainHead>>,
    time: Res<Time>
) {
    let Some(target_position) = get_cursor_world_position(&q_windows, &q_camera) else { return };

    for (mut transform, node, mut head) in &mut q_heads {
        let mut direction = target_position - transform.translation.truncate();
        if direction.length_squared() <= 100.0 {
            return;
        }

        let next_entity = node.next;
        if let Some(next) = next_entity {
            if let Ok((next_transform, _)) = q_nodes.get(next) {
                let predicted_direction = (transform.translation - next_transform.translation).truncate();
                let angle = predicted_direction.angle_to(direction).to_degrees().clamp(-20., 20.);
                
                direction = rotate(predicted_direction, angle.to_radians());
            }
        }

        head.target = target_position;
        head.direction = direction;
        transform.translation += (direction.normalize_or_zero() * SNAKE_SPEED * time.delta_secs()).extend(0.0);
        transform.rotation = Quat::from_axis_angle(Vec3::Z, direction.to_angle());
    }
}

fn main() {
    App::new()
        .add_systems(Startup, spawn_snake)
        .add_systems(Update, move_snakes.before(move_chain))
        .add_plugins((DefaultPlugins, CameraPlugin { speed: CAMERA_SPEED }, ChainPlugin))
        .run();
}