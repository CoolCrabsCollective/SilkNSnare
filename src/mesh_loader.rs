use bevy::asset::UntypedAssetId;
use bevy::{
    asset::LoadState,
    gltf::{Gltf, GltfMesh, GltfNode},
    log,
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group};
use std::any::Any;

use crate::config::{
    COLLISION_GROUP_ENEMIES, COLLISION_GROUP_PLAYER, COLLISION_GROUP_TERRAIN, COLLISION_GROUP_WALLS,
};
use crate::game::ORANGE_LIGHT_COLOR;
use crate::pumpkin::Pumpkin;

pub struct MeshLoaderPlugin;

pub struct LoadedGltf {
    pub gltf_handle: Handle<Gltf>,
    pub processed: bool,
}

#[derive(Resource)]
pub struct MeshLoader(Vec<LoadedGltf>);

impl Plugin for MeshLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, process_loaded_gltfs.after(setup));
    }
}

pub fn setup(mut commands: Commands) {
    commands.insert_resource(MeshLoader(vec![]));
}

pub fn load_level(
    asset_path: String,
    asset_server: Res<AssetServer>,
    mut mesh_loader: ResMut<MeshLoader>,
) {
    mesh_loader.0.push(LoadedGltf {
        gltf_handle: asset_server.load(asset_path),
        processed: false,
    });
}

#[allow(clippy::too_many_arguments)]
fn process_loaded_gltfs(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    nodes: Res<Assets<GltfNode>>,
    mut mesh_loader: ResMut<MeshLoader>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    for loaded_gltf in mesh_loader.0.iter_mut() {
        if loaded_gltf.processed {
            continue;
        }

        let Some(gltf) = gltf_assets.get(&loaded_gltf.gltf_handle) else {
            continue;
        };

        let first_scene_handle = gltf.scenes[0].clone();

        for (name, node_handle) in &gltf.named_nodes {
            println!("{}", name);
            if name.to_lowercase().contains("terrain") || name.to_lowercase().contains("wall") {
                info!("Generating collider from level object: {name:?}");
                if let (Some(mesh), Some(material_handle), Some(transform)) = (
                    get_mesh_from_gltf_node(node_handle, &meshes, &gltf_meshes, &nodes),
                    get_material_from_gltf_node(node_handle, &gltf_meshes, &nodes),
                    nodes.get(node_handle).map(|node| node.transform),
                ) {
                    if name.to_lowercase().contains("wall") {
                        materials.get_mut(&material_handle).unwrap().base_color =
                            Color::srgb(0.0, 0.0, 0.0);
                        materials.get_mut(&material_handle).unwrap().alpha_mode = AlphaMode::Blend;
                    }
                    match get_collider_from_mesh(mesh, &transform) {
                        Ok(collider) => {
                            commands.spawn(collider).insert(
                                if name.to_lowercase().contains("wall") {
                                    CollisionGroups {
                                        memberships: COLLISION_GROUP_WALLS,
                                        filters: COLLISION_GROUP_PLAYER,
                                    }
                                } else if name.to_lowercase().contains("terrain") {
                                    println!("{}", name.to_lowercase());
                                    CollisionGroups {
                                        memberships: COLLISION_GROUP_TERRAIN,
                                        filters: COLLISION_GROUP_PLAYER | COLLISION_GROUP_ENEMIES,
                                    }
                                } else {
                                    CollisionGroups {
                                        memberships: Group::NONE,
                                        filters: Group::NONE,
                                    }
                                },
                            );
                        }
                        Err(err) => {
                            error!("{err:?}");
                        }
                    }
                } else {
                    error!("Node {name:?} was missing either a mesh or a transform");
                }
            }

            if name.to_lowercase().contains("pumpkin") {
                if let Some(transform) = nodes.get(node_handle).map(|node| node.transform) {
                    log::warn!(
                        "Spawning point light at {:?}",
                        transform.translation + Vec3::new(0.0, 5.0, 0.0)
                    );
                    commands.spawn((
                        PointLightBundle {
                            transform: transform
                                .with_translation(transform.translation + Vec3::new(0.0, 5.0, 0.0)),
                            point_light: PointLight {
                                intensity: 500_000.0,
                                color: ORANGE_LIGHT_COLOR,
                                shadows_enabled: true,
                                radius: 1.0,
                                ..default()
                            },
                            ..default()
                        },
                        Pumpkin,
                    ));
                }
            }
        }

        commands.spawn(SceneBundle {
            scene: first_scene_handle,
            ..default()
        });

        loaded_gltf.processed = true;
    }
}

fn get_mesh_from_gltf_node<'a>(
    node_handle: &Handle<GltfNode>,
    meshes: &'a Res<Assets<Mesh>>,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    nodes: &Res<Assets<GltfNode>>,
) -> Option<&'a Mesh> {
    nodes
        .get(node_handle)
        .and_then(|node| node.mesh.as_ref())
        .and_then(|mesh_handle| gltf_meshes.get(mesh_handle))
        .and_then(|gltf_mesh| gltf_mesh.primitives.get(0))
        .and_then(|first_primitive| meshes.get(&first_primitive.mesh))
}

fn get_material_from_gltf_node<'a>(
    node_handle: &Handle<GltfNode>,
    gltf_meshes: &Res<Assets<GltfMesh>>,
    nodes: &Res<Assets<GltfNode>>,
) -> Option<Handle<StandardMaterial>> {
    nodes
        .get(node_handle)
        .and_then(|node| node.mesh.as_ref())
        .and_then(|mesh_handle| gltf_meshes.get(mesh_handle))
        .and_then(|gltf_mesh| gltf_mesh.primitives.get(0))
        .and_then(|first_primitive| first_primitive.material.clone())
}

// taken from https://github.com/Defernus/bevy_gltf_collider/blob/9f27253e6d2e645c3570bebead34a493e4da1deb/src/mesh_collider.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColliderFromMeshError {
    MissingPositions,
    MissingIndices,
    InvalidIndicesCount(usize),
    InvalidPositionsType(&'static str),
}

fn get_collider_from_mesh(
    mesh: &Mesh,
    transform: &Transform,
) -> Result<Collider, ColliderFromMeshError> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .map_or(Err(ColliderFromMeshError::MissingPositions), Ok)?;

    let indices = mesh
        .indices()
        .map_or(Err(ColliderFromMeshError::MissingIndices), Ok)?;

    let positions = match positions {
        VertexAttributeValues::Float32x3(positions) => positions,
        v => {
            return Err(ColliderFromMeshError::InvalidPositionsType(
                v.enum_variant_name(),
            ));
        }
    };

    let indices: Vec<u32> = match indices {
        Indices::U32(indices) => indices.clone(),
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
    };

    if indices.len() % 3 != 0 {
        return Err(ColliderFromMeshError::InvalidIndicesCount(indices.len()));
    }

    let triple_indices = indices.chunks(3).map(|v| [v[0], v[1], v[2]]).collect();
    let vertices = positions
        .iter()
        .map(|v| {
            let p = Vec4::new(v[0], v[1], v[2], 1.0);
            let p_transformed = transform.compute_matrix() * p;
            Vec3::new(
                p_transformed.x / p_transformed.w,
                p_transformed.y / p_transformed.w,
                p_transformed.z / p_transformed.w,
            )
        })
        .collect();

    let collider = Collider::trimesh(vertices, triple_indices);

    Ok(collider)
}
