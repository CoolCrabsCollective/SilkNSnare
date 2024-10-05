use crate::game::get_initial_camera_transform;
use bevy::prelude::*;

pub struct TreePlugin;

#[derive(Component)]
struct Tree;

#[derive(Resource)]
struct TreeScene(Handle<Gltf>);

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tree);
        app.add_systems(Update, create_tree_collider);
        app.add_systems(Update, move_to_tree);
    }
}

fn spawn_tree(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let gltf = asset_server.load("tree.glb#Scene0");
    commands.insert_resource(TreeScene(gltf));
}

fn move_to_tree(mut camera_transform_query: Query<(&mut Transform, &Camera)>, time: Res<Time>) {
    let s = (time.elapsed_seconds() / 2.0).min(1.0);
    let t = 3.0 * s * s - 2.0 * s * s * s;
    if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
        camera_transform.translation = ((1.0 - t) * get_initial_camera_transform().translation)
            + t * get_target_camera_position();
    }
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

fn create_tree_collider(
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    tree_scene: Res<TreeScene>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }

    let Some(gltf) = gltf_assets.get(&tree_scene.0) else {
        println!("Waiting on loaded tree");
        return;
    };
    *loaded = true;

    //let tree_mesh: Handle<GltfMesh> = gltf.meshes[0];
    commands.spawn(
        (SceneBundle {
            scene: gltf.scenes[0].clone(),
            ..default()
        }),
    );
}
