use crate::config::COLLISION_GROUP_ENEMIES;
use crate::flying_obstacle::flying_obstacle::{rock_timer_value, FlyingObstacle, RockSpawnTimer};
use crate::tree::GameStart;
use bevy::asset::AssetServer;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    Commands, Component, Query, Res, ResMut, SceneBundle, Time, Timer, TimerMode, Transform,
};
use bevy_rapier3d::geometry::{ActiveEvents, Group};
use bevy_rapier3d::prelude::{ActiveCollisionTypes, Collider, CollisionGroups};
use rand::Rng;
use std::time::Duration;

#[derive(Component)]
struct Rock;

pub fn spawn_rock(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    time: Res<Time>,
    mut r_spawn_timer: ResMut<RockSpawnTimer>,
    start_query: Query<&GameStart>,
) {
    if let Ok(game_start) = start_query.get_single() {
        r_spawn_timer.timer.tick(time.delta());
        if r_spawn_timer.timer.just_finished() {
            let next_rock_time = Duration::from_secs_f32(rock_timer_value(
                time.elapsed_seconds() - game_start.game_start,
            ));
            println!(
                "Throwing a rock, resetting timer to {:?}",
                next_rock_time.as_secs_f32()
            );
            r_spawn_timer.timer = Timer::new(next_rock_time, TimerMode::Repeating);
            let mut rng = rand::thread_rng();
            let x_begin = rng.gen_range(-3.0..0.0);
            let y_begin = 1.0;
            let start_pos = Vec3::new(x_begin, y_begin, -2.0);

            let y_begin_vel = rng.gen_range(0.75..3.0);
            let z_begin_vel = 4.0;
            let vel = Vec3::new(0.0, y_begin_vel, z_begin_vel);

            commands
                .spawn((
                    FlyingObstacle::new(
                        start_pos,
                        vel,
                        Vec3::new(0.0, -9.81, 0.0),
                        Vec3::new(0.0, 5.0, 5.0),
                        Duration::from_secs(20),
                    ),
                    Rock,
                    SceneBundle {
                        scene: asset_server.load("stone.glb#Scene0"),
                        transform: Transform {
                            translation: start_pos,
                            rotation: Quat::default(),
                            scale: Vec3::new(0.07, 0.07, 0.07) * 1.5,
                        },
                        global_transform: Default::default(),
                        visibility: Default::default(),
                        inherited_visibility: Default::default(),
                        view_visibility: Default::default(),
                    },
                    // Collider::capsule_y(1.0, 1.0),
                    Collider::ball(0.75),
                ))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
                .insert(CollisionGroups {
                    memberships: COLLISION_GROUP_ENEMIES,
                    filters: Group::ALL,
                });
        }
    }
}
