use bevy::prelude::*;

pub struct SpiderPlugin;


impl Plugin for SpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spider);
        app.add_systems(Update, move_spider);
    }
}
fn move_spider(mut camera_transform_query: Query<(&mut Transform, &Camera)>, time: Res<Time>) {
    // let t = (time.elapsed_seconds() / 2.0).min(1.0);
    // if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
    //     camera_transform.translation = ((1.0 - t) * get_initial_camera_transform().translation)
    //         + t * crate::tree::get_target_camera_position();
    // }
}

fn spawn_spider(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut camera_transform_query: Query<(&mut Transform, &Camera)>,
)
{
    commands.spawn(SceneBundle {
        scene: asset_server.load("spider.glb#Scene0"),
        transform: Transform{
            translation: Vec3::new(0.0, 0.0, 2.0),
            rotation: Quat::default(),
            scale: Vec3::new(0.25, 0.25, 0.25),
        },
        ..default()
    });
}