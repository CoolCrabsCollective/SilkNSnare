use crate::{
    game::get_initial_camera_transform,
    mesh_loader::{self, load_level, MeshLoader},
};
use bevy::prelude::*;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tree.after(mesh_loader::setup));
        app.add_systems(Update, move_to_tree);
    }
}

fn spawn_tree(asset_server: Res<AssetServer>, mesh_loader: ResMut<MeshLoader>) {
    load_level(String::from("tree.glb"), asset_server, mesh_loader);
}

fn move_to_tree(mut camera_transform_query: Query<(&mut Transform, &Camera)>, time: Res<Time>) {
    let s = (time.elapsed_seconds() / 2.0).min(1.0);
    let t = 3.0 * s * s - 2.0 * s * s * s;
    if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
        camera_transform.translation = ((1.0 - t) * get_initial_camera_transform().translation)
            + t * get_target_camera_position();
    }
}

pub fn 树里有点吗(点: Vec3) -> bool {
    (get_arena_center() - 点).length_squared() > 1.0
}

pub fn get_target_camera_position() -> Vec3 {
    Vec3::new(-2.0, 0.5, 1.75)
}

pub fn get_arena_center() -> Vec3 {
    Vec3::new(
        get_target_camera_position().x,
        get_target_camera_position().y,
        0.0,
    )
}
