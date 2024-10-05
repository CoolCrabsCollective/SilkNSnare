use bevy::{prelude::*};
use std::f32::consts::FRAC_1_SQRT_2;

pub struct FruitFlyPlugin;

#[derive(Component)]
struct FruitFly {

}

impl FruitFly {
    pub fn new() -> Self {
        FruitFly {

        }
    }
}

impl Plugin for FruitFlyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_fruit_fly);
        // app.add_systems(Update, move_fruit_fly);
    }
}

fn spawn_fruit_fly(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>
) {
    let start_pos = Vec3::new(-2.5, 0.0, 0.0);

    commands.spawn((
        FruitFly::new(),
        SceneBundle {
            scene: asset_server.load("fruit_fly.glb#Scene0"),
            transform: Transform {
                translation: start_pos,
                rotation: Quat::from_xyzw(0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2),
                scale: Vec3::new(0.02, 0.02, 0.02),
            },
            global_transform: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
        },
    ));
}

