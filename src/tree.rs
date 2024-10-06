use std::f32::consts::PI;

use crate::{
    game::get_initial_camera_transform,
    mesh_loader::{self, load_level, MeshLoader},
};
use bevy::prelude::*;

pub struct TreePlugin;

const TREE_LIMIT: f32 = 0.75;
const MAP_LIMIT: Vec3 = Vec3::new(1.8, 1.0, 0.0);
const ADD_DEBUG_PLANE: bool = false;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tree.after(mesh_loader::setup));
        app.add_systems(Update, move_to_tree);
    }
}

fn spawn_tree(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mesh_loader: ResMut<MeshLoader>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    load_level(String::from("tree.glb"), asset_server, mesh_loader);

    if ADD_DEBUG_PLANE {
        let debug_plane = meshes.add(Plane3d {
            normal: Dir3::new(Vec3::Z).unwrap(),
            half_size: Vec2::splat(1.0),
        });

        let debug_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.8, 0.5),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: debug_plane.clone(),
            material: debug_material.clone(),
            transform: Transform::from_translation(Vec3::new(-2.0, 0.5, 0.0)),
            ..default()
        });
    }
}

fn move_to_tree(
    mut camera_transform_query: Query<(&mut Transform, &Camera)>,
    time: Res<Time>,
    mut swap_camera_angle: Local<bool>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let s = (time.elapsed_seconds() / 2.0).min(1.0);
    let t = 3.0 * s * s - 2.0 * s * s * s;

    if keys.just_released(KeyCode::KeyQ) {
        *swap_camera_angle = !*swap_camera_angle;
    }

    let target_camera_pos = if *swap_camera_angle {
        get_target_camera_position_2()
    } else {
        get_target_camera_position()
    };

    let target_camera_rot = if *swap_camera_angle {
        get_target_camera_direction_2()
    } else {
        get_target_camera_direction()
    };

    if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
        camera_transform.translation =
            ((1.0 - t) * get_initial_camera_transform().translation) + t * target_camera_pos;
        camera_transform.rotation = get_initial_camera_transform()
            .rotation
            .lerp(target_camera_rot, t)
    }
}

pub fn 树里有点吗(点: Vec3) -> bool {
    if !照相机里有点吗(点) {
        return false;
    }

    let 差 = get_arena_center() - 点;

    差.y > TREE_LIMIT || 差.y < -TREE_LIMIT
}

pub fn 树里有小路吗(开始: Vec3, 结尾: Vec3) -> bool {
    if !树里有点吗(开始) || !树里有点吗(结尾) {
        return false;
    }

    // 如果不好提高这号码
    for i in 1..10 {
        let t = i as f32 / 10.0;
        if !树里有点吗(开始 * t + 结尾 * (1.0 - t)) {
            return false;
        }
    }

    true
}

pub fn 照相机里有点吗(点: Vec3) -> bool {
    let 差 = get_arena_center() - 点;

    !(差.y > MAP_LIMIT.y || 差.y < -MAP_LIMIT.y || 差.x > MAP_LIMIT.x || 差.x < -MAP_LIMIT.x)
}

pub fn get_target_camera_position() -> Vec3 {
    Vec3::new(-2.0, 0.5, 1.75)
}

pub fn get_target_camera_position_2() -> Vec3 {
    Vec3::new(-3.0, 0.5, 1.75)
}

pub fn get_target_camera_direction() -> Quat {
    Quat::from_axis_angle(Vec3::Y, 0.0)
}

pub fn get_target_camera_direction_2() -> Quat {
    Quat::from_axis_angle(Vec3::Y, -PI / 6.0)
}

pub fn get_arena_center() -> Vec3 {
    Vec3::new(
        get_target_camera_position().x,
        get_target_camera_position().y,
        0.0,
    )
}
