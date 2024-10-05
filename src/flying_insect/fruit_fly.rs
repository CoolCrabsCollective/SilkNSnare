use crate::flying_insect::flying_insect::{BezierCurve, FlyingInsect, FruitFlySpawnTimer};
use bevy::prelude::*;
use std::f32::consts::FRAC_1_SQRT_2;

#[derive(Component)]
struct FruitFly;

pub fn spawn_fruit_fly(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    time: Res<Time>,
    mut ff_spawn_timer: ResMut<FruitFlySpawnTimer>,
) {
    //ff_spawn_timer.timer.tick(time.delta());
    let start_pos = Vec3::new(-2.5, 0.5, 1.0);
    let end_pos = Vec3::new(-2.0, 0.6, -2.0);
    commands.spawn((
        FlyingInsect::new(
            0.1,
            5.0,
            BezierCurve::random_from_endpoints(start_pos, end_pos),
        ),
        FruitFly,
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
