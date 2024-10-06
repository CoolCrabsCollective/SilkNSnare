use bevy::{
    log,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use bevy_rapier3d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};
use std::f32::consts::PI;

use super::Web;

pub const WEB_SILK_THICKNESS: f32 = 0.03;
pub const WEB_SILK_PRISM_BASE: i32 = 4;

#[derive(Component)]
pub struct WebRenderMesh {
    mesh_handle: Handle<Mesh>,
    material_handle: Handle<StandardMaterial>,
}

/// used only for collision
#[derive(Component)]
pub struct WebSegmentCollision {
    pub spring_index: usize,
}

pub fn clear_web(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    web_render_segments_query: Query<(Entity, &WebRenderMesh)>,
    web_segment_collisions_query: Query<(Entity, &WebSegmentCollision)>,
) {
    for (
        web_render_segment_entity,
        WebRenderMesh {
            mesh_handle: web_segment_mesh_handle,
            material_handle: web_segment_material_handle,
        },
    ) in web_render_segments_query.iter()
    {
        meshes.remove(web_segment_mesh_handle);
        materials.remove(web_segment_material_handle);
        commands.entity(web_render_segment_entity).despawn();
    }

    for (web_segment_collision_entity, _) in web_segment_collisions_query.iter() {
        commands.entity(web_segment_collision_entity).despawn();
    }
}

pub fn render_web(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    web_query: Query<&Web>,
    camera_query: Query<(&Transform, &Camera)>,
    time: Res<Time>,
) {
    let Ok(web_data) = web_query.get_single() else {
        error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    let Ok((camera_transform, _)) = camera_query.get_single() else {
        error!("ERROR NO CAMERA OR MORE THAN ONE CAMERA");
        return;
    };

    let (mesh, segment_colliders) = create_web_mesh(&web_data, camera_transform);
    let mesh_handle: Handle<Mesh> = meshes.add(mesh);

    let material_handle: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..default()
    });

    for (segment_collider, spring_index) in segment_colliders {
        commands
            .spawn((segment_collider, WebSegmentCollision { spring_index }))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC);
    }

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            ..default()
        },
        WebRenderMesh {
            mesh_handle: mesh_handle.clone(),
            material_handle: material_handle.clone(),
        },
    ));
}

fn create_web_mesh(web_data: &Web, camera_transform: &Transform) -> (Mesh, Vec<(Collider, usize)>) {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut indices: Vec<u32> = Vec::new();

    let mut segment_colliders: Vec<(Collider, usize)> = Vec::new();

    for (spring_index, spring) in web_data.springs.iter().enumerate() {
        let first_index = spring.first_index;
        let second_index = spring.second_index;
        let first_position = web_data.particles[first_index].position;
        let second_position = web_data.particles[second_index].position;
        let center_position = (first_position + second_position) / 2.0;
        let segment_as_vec = second_position - first_position;

        segment_colliders.push((
            Collider::capsule(first_position, second_position, WEB_SILK_THICKNESS / 2.0),
            spring_index,
        ));

        if WEB_SILK_PRISM_BASE < 3 {
            let to_camera = (camera_transform.translation - center_position).normalize();
            let perp = segment_as_vec.cross(to_camera).normalize();
            let top_left = first_position + perp * WEB_SILK_THICKNESS / 2.0;
            let top_right = first_position - perp * WEB_SILK_THICKNESS / 2.0;

            let bottom_left = second_position + perp * WEB_SILK_THICKNESS / 2.0;
            let bottom_right = second_position - perp * WEB_SILK_THICKNESS / 2.0;

            let top_left_index = positions.len();
            let top_right_index = top_left_index + 1;
            let bottom_left_index = top_left_index + 2;
            let bottom_right_index = top_left_index + 3;

            positions.push(top_left);
            positions.push(top_right);
            positions.push(bottom_left);
            positions.push(bottom_right);

            normals.push(to_camera);
            normals.push(to_camera);
            normals.push(to_camera);
            normals.push(to_camera);

            uvs.push(Vec2::new(0.0, 0.0));
            uvs.push(Vec2::new(1.0, 0.0));
            uvs.push(Vec2::new(0.0, 1.0));
            uvs.push(Vec2::new(1.0, 1.0));

            // triangle 1
            indices.push(bottom_left_index.try_into().unwrap());
            indices.push(top_right_index.try_into().unwrap());
            indices.push(top_left_index.try_into().unwrap());

            // triangle 2
            indices.push(bottom_left_index.try_into().unwrap());
            indices.push(bottom_right_index.try_into().unwrap());
            indices.push(top_right_index.try_into().unwrap());
            continue;
        }

        for i in 0..WEB_SILK_PRISM_BASE {
            let quat = Quat::from_axis_angle(
                segment_as_vec.normalize(),
                i as f32 / WEB_SILK_PRISM_BASE as f32 * 2.0 * PI,
            );
            let normal = quat.mul_vec3(Vec3::new(0.0, 0.0, 1.0));

            let top_left_index = positions.len();
            let top_right_index = top_left_index + 1;
            let bottom_left_index = top_left_index + 2;
            let bottom_right_index = top_left_index + 3;

            let perp = segment_as_vec.cross(normal).normalize();
            let top_left = first_position
                + perp * WEB_SILK_THICKNESS / 2.0
                + normal * WEB_SILK_THICKNESS / 2.0;
            let top_right = first_position - perp * WEB_SILK_THICKNESS / 2.0
                + normal * WEB_SILK_THICKNESS / 2.0;

            let bottom_left = second_position
                + perp * WEB_SILK_THICKNESS / 2.0
                + normal * WEB_SILK_THICKNESS / 2.0;
            let bottom_right = second_position - perp * WEB_SILK_THICKNESS / 2.0
                + normal * WEB_SILK_THICKNESS / 2.0;

            positions.push(top_left);
            positions.push(top_right);
            positions.push(bottom_left);
            positions.push(bottom_right);

            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
            normals.push(normal);

            uvs.push(Vec2::new(0.0, 0.0));
            uvs.push(Vec2::new(1.0, 0.0));
            uvs.push(Vec2::new(0.0, 1.0));
            uvs.push(Vec2::new(1.0, 1.0));

            // triangle 1
            indices.push(bottom_left_index.try_into().unwrap());
            indices.push(top_right_index.try_into().unwrap());
            indices.push(top_left_index.try_into().unwrap());

            // triangle 2
            indices.push(bottom_left_index.try_into().unwrap());
            indices.push(bottom_right_index.try_into().unwrap());
            indices.push(top_right_index.try_into().unwrap());
        }
    }

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions
            .iter()
            .map(|position| position.to_array())
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals
            .iter()
            .map(|normal| normal.to_array())
            .collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        uvs.iter().map(|uv| uv.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U32(indices));

    (mesh, segment_colliders)
}
