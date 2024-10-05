use bevy::prelude::*;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tree);
        app.add_systems(Update, move_to_tree);
    }
}

fn spawn_tree(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut camera_transform_query: Query<(&mut Transform, &Camera)>,
) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("tree.glb#Scene0"),
        ..default()
    });
}

fn move_to_tree(mut camera_transform_query: Query<(&mut Transform, &Camera)>, time: Res<Time>) {
    if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
        camera_transform.translation = get_target_camera_position();
    }
}

pub fn get_target_camera_position() -> Vec3 {
    Vec3::new(2.0, 0.5, 1.75)
}
