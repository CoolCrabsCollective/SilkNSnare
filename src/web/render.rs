use bevy::{
    log,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

use super::Web;

pub const WEB_SILK_THICKNESS: f32 = 0.025;

#[derive(Component)]
pub struct WebRenderSegment(Handle<Mesh>);

pub fn clear_web(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    web_render_segments_query: Query<(Entity, &WebRenderSegment)>,
) {
    // TODO: lets validate that we only have one web entity per frame..
    for (web_render_segment_entity, WebRenderSegment(web_segment_mesh_handle)) in
        web_render_segments_query.iter()
    {
        meshes.remove(web_segment_mesh_handle);
        commands.entity(web_render_segment_entity).despawn();
    }
}

pub fn render_web(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    web_query: Query<&Web>,
    camera_query: Query<(&Transform, &Camera)>,
    time: Res<Time>,
) {
    let Ok(web_data) = web_query.get_single() else {
        log::error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    let Ok((camera_transform, _)) = camera_query.get_single() else {
        log::error!("ERROR NO CAMERA OR MORE THAN ONE CAMERA");
        return;
    };

    let mesh_handle: Handle<Mesh> = meshes.add(create_web_mesh(&web_data, camera_transform));

    let t = (time.elapsed_seconds() / 4.0).min(1.0);

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(t, 0.0, 0.0),
                ..default()
            }),
            ..default()
        },
        WebRenderSegment(mesh_handle.clone()),
    ));
}

fn create_web_mesh(web_data: &Web, camera_transform: &Transform) -> Mesh {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<Vec2> = Vec::new();

    let mut indices: Vec<u32> = Vec::new();

    for spring in &web_data.springs {
        let first_index = spring.first_index;
        let second_index = spring.second_index;
        let first_position = web_data.particles[first_index].position;
        let second_position = web_data.particles[second_index].position;
        let center_position = (first_position + second_position) / 2.0;

        let to_camera = (camera_transform.translation - center_position).normalize();
        let segment_direction = second_position - first_position;
        let perp = segment_direction.cross(to_camera).normalize();
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
    }

    Mesh::new(
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
    .with_inserted_indices(Indices::U32(indices))
}
