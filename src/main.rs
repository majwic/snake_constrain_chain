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

const CAMERA_SPEED: f32 = 500.;
const SNAKE_SPEED: f32 = 500.;

#[derive(Component, Debug)]
struct SnakeNode {
    next: Option<Entity>,
}

#[derive(Component)]
struct SnakeHead;

fn constrain_distance(point: Vec2, anchor: Vec2, distance: f32) -> Option<Vec2> {
    let direction = point - anchor;
    if direction.length_squared() <= 1. {
        return None;
    }
    Some(direction.normalize() * distance + anchor)
}

fn rotate(direction: Vec2, angle: f32) -> Vec2 {
    let angle_rad: f32 = angle.to_radians();
    let rotated_x: f32 = direction.x * angle_rad.cos() - direction.y * angle_rad.sin();
    let rotated_y: f32 = direction.x * angle_rad.sin() + direction.y * angle_rad.cos();
    return Vec2::new(rotated_x, rotated_y);
}

fn clamp_angle(
    current_direction: Vec2,
    prev_direction: Vec2,
    prev_position: Vec2,
    prev_size: f32,
    min_angle: f32,
) -> Option<Vec2> {
    let angle: f32 = prev_direction.angle_to(current_direction).to_degrees();

    if angle < -min_angle || angle > min_angle {
        let angle: f32 = angle.clamp(-min_angle, min_angle);
        let rotated_direction: Vec2 = -rotate(prev_direction, angle);

        return Some(prev_position + rotated_direction.normalize() * prev_size);
    }

    return None;
}

fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let snake_size = 20.;
    let length = 40;
    
    let mut prev_entity = None;

    for i in (0..length).rev() {
        let mut entity = commands.spawn((
            SnakeNode { next: prev_entity },
            Mesh2d(meshes.add(Circle::new(snake_size))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::from_xyz(0., -i as f32 * snake_size, 0. - i as f32),
        ));

        if i == 0 {
            entity.insert(SnakeHead);
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

fn move_snake(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_heads: Query<(&SnakeNode, &mut Transform), With<SnakeHead>>,
    mut q_nodes: Query<(&SnakeNode, &mut Transform), Without<SnakeHead>>,
    time: Res<Time>,
) {
    let Some(target_position) = get_cursor_world_position(&q_windows, &q_camera) else { return };
    
    for (snake_node, mut transform) in &mut q_heads {
        let mut direction = target_position - transform.translation.truncate();
        if direction.length_squared() <= 100. {
            return;
        }

        let mut next_entity = snake_node.next;

        if let Some(next) = next_entity {
            if let Ok((_, next_transform)) = q_nodes.get(next) {
                let predicted_direction = (transform.translation - next_transform.translation).truncate();
                let angle = predicted_direction.angle_to(direction).to_degrees().clamp(-20., 20.);

                direction = rotate(predicted_direction, angle);
            }
        }

        transform.translation += (direction.normalize() * SNAKE_SPEED * time.delta_secs()).extend(0.);

        let mut anchor = transform.translation.truncate();
        let mut prev_direction = direction;

        while let Some(entity) = next_entity {
            if let Ok((node, mut transform)) = q_nodes.get_mut(entity) {
                let Some(mut new_position) = constrain_distance(
                    transform.translation.truncate(), 
                    anchor, 
                    20.
                ) else { break };

                let current_direction = anchor - transform.translation.truncate();

                if let Some(clamped_position) = clamp_angle(
                    current_direction,
                    prev_direction,
                    anchor,
                    20.,
                    20.,
                ) {
                    new_position = clamped_position;
                };

                prev_direction = anchor - new_position;
                transform.translation = new_position.extend(transform.translation.z);
                anchor = transform.translation.truncate();
                next_entity = node.next;
            } else {
                break;
            }
        }
    }
}

fn main() {
    App::new()
        .add_systems(Startup, spawn_snake)
        .add_systems(Update, move_snake)
        .add_plugins((DefaultPlugins, CameraPlugin { speed: CAMERA_SPEED }))
        .run();
}

// vertice 0 [0,   0,  z] Top-Left
// vertice 1 [1,   0,  z] Top-Right
// vertice 2 [0,   -1, z] Next-Left
// vertice 3 [1,   -1, z] Next-Right
// vertice 4 [0,   -2, z] Next-Left
// vertice 5 [1,   -2, z] Next-Right
// vertice 6 [0,   -3, z] Next-Left
// vertice 7 [1,   -3, z] Next-Right
// vertice 8 [0.5, -4, z] Bottom-Middle
//
// indices: [
// 0, 1, 2 Triangle 1
// 1, 2, 3 Triangle 2
// 2, 3, 4 Triangle 3
// 3, 4, 5 Triangle 4
// 4, 5, 6 Triangle 5
// 5, 6, 7 Triangle 6
// 6, 7, 8 Triangle 7
// ]