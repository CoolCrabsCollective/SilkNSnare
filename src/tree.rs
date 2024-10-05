use bevy::prelude::*;

use crate::game::get_initial_camera_transform;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tree);
        app.add_systems(Update, move_to_tree);
    }
}

fn spawn_tree(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("tree.glb#Scene0"),
        ..default()
    });
}




fn move_to_tree(mut camera_transform_query: Query<(&mut Transform, &Camera)>, time: Res<Time>) {
    let t = (time.elapsed_seconds() / 2.0).min(1.0);
    if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
        camera_transform.translation = ((1.0 - t) * get_initial_camera_transform().translation)
            + t * get_target_camera_position();
    }
}

pub fn get_target_camera_position() -> Vec3 {
    Vec3::new(-1.75, 0.5, 2.0)
}

pub fn get_arena_center() -> Vec3 {
    Vec3::new(
        get_target_camera_position().x,
        get_target_camera_position().y,
        0.0,
    )
}
