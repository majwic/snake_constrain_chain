use bevy::prelude::*;

#[derive(Component)]

pub struct ChainNode {
    pub next: Option<Entity>,
    pub distance: f32,
    pub angle_threshold: f32,
} 

#[derive(Component)]
pub struct ChainHead {
    pub target: Vec2,
    pub direction: Vec2,    
}

pub struct ChainPlugin;

fn constrain_distance(point: Vec2, anchor: Vec2, distance: f32) -> Vec2 {
    ((point - anchor).normalize_or_zero() * distance) + anchor
}

pub fn rotate(direction: Vec2, angle: f32) -> Vec2 {
    let rotated_x: f32 = direction.x * angle.cos() - direction.y * angle.sin();
    let rotated_y: f32 = direction.x * angle.sin() + direction.y * angle.cos();
    Vec2::new(rotated_x, rotated_y)
}

fn clamp_angle(
    current_direction: Vec2,
    prev_direction: Vec2,
    prev_position: Vec2,
    prev_size: f32,
    min_angle: f32,
) -> Option<Vec2> {
    let angle: f32 = prev_direction.angle_to(current_direction);

    if angle < -min_angle || angle > min_angle {
        let clamped_angle = angle.clamp(-min_angle, min_angle);
        let rotated_direction = -rotate(prev_direction.normalize(), clamped_angle);

        Some(prev_position + rotated_direction.normalize() * prev_size)
    } else {
        None
    }
}

pub fn move_chain(
    mut q_heads: Query<(&ChainNode, Option<&mut Transform>, &mut ChainHead)>,
    mut q_nodes: Query<(&ChainNode, Option<&mut Transform>), Without<ChainHead>>
) {
    for (head_node, opt_transform, head) in &mut q_heads {
        let Some(head_transform) = opt_transform else {
            continue;
        };
        
        let mut anchor_distance = head_node.distance;
        let mut anchor_direction = head.direction;
        let mut anchor = head_transform.translation.truncate();
        let mut next_entity = head_node.next;

        while let Some(entity) = next_entity {
            if let Ok((node, opt_transform)) = q_nodes.get_mut(entity) {
                let Some(mut node_transform) = opt_transform else {
                    break;
                };

                let mut node_position = constrain_distance(
                    node_transform.translation.truncate(), 
                    anchor, 
                    node.distance,
                );

                if let Some(clamped_position) = clamp_angle(
                    anchor - node_transform.translation.truncate(),
                    anchor_direction,
                    anchor,
                    anchor_distance,
                    node.angle_threshold,
                ) {
                    node_position = clamped_position;
                };

                let axis_angle = (anchor - node_transform.translation.truncate()).to_angle();
                node_transform.translation = node_position.extend(node_transform.translation.z);
                node_transform.rotation = Quat::from_axis_angle(Vec3::Z, axis_angle);
                
                anchor_distance = node.distance;
                anchor_direction = anchor - node_position;
                anchor = node_position;
                next_entity = node.next;
            } else {
                break;
            }
        }
    }
}

impl Plugin for ChainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_chain);
    }
}