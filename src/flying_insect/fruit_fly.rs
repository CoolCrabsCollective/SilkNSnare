use crate::flying_insect::flying_insect::{BezierCurve, FlyingInsect, FruitFlySpawnTimer};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};
use rand::Rng;

pub const DAVID_DEBUG: bool = false;

#[derive(Component)]
struct FruitFly;

pub fn spawn_fruit_fly(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    time: Res<Time>,
    mut ff_spawn_timer: ResMut<FruitFlySpawnTimer>,
) {
    ff_spawn_timer.timer.tick(time.delta());
    if ff_spawn_timer.timer.just_finished() {
        let mut rng = rand::thread_rng();
        let x_begin = rng.gen_range(-4.0..0.0);
        let x_end = rng.gen_range(-3.0..-1.0);
        let y_begin = rng.gen_range(0.0..1.0);
        let y_end = rng.gen_range(0.0..1.0);

        let start_pos = Vec3::new(x_begin, y_begin, -2.0);
        let end_pos = Vec3::new(x_end, y_end, 3.5);

        let david_debug_pos = Vec2::new(-2.0, 0.1);

        commands
            .spawn((
                FlyingInsect::new(
                    0.1,
                    0.01,
                    if DAVID_DEBUG {
                        BezierCurve::new(
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, -1.0),
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, -1.0),
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, 3.0),
                            Vec3::new(david_debug_pos.x, david_debug_pos.y, 3.0),
                        )
                    } else {
                        BezierCurve::random_from_endpoints(start_pos, end_pos)
                    },
                ),
                FruitFly,
                SceneBundle {
                    scene: asset_server.load("fruit_fly.glb#Scene0"),
                    transform: Transform {
                        translation: start_pos,
                        rotation: Quat::default(),
                        scale: Vec3::new(0.02, 0.02, 0.02),
                    },
                    global_transform: Default::default(),
                    visibility: Default::default(),
                    inherited_visibility: Default::default(),
                    view_visibility: Default::default(),
                },
                Collider::capsule_y(1.0, 1.0),
            ))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC);
    }
}
