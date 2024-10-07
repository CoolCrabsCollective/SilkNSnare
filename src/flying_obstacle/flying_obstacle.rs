use std::time::Duration;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::math::{EulerRot, Mat4};
use bevy::prelude::{Commands, Component, Entity, EventReader, Quat, Query, Res, Resource, Time, Timer, TimerMode, Transform, Vec3, Without};
use bevy_rapier3d::pipeline::CollisionEvent;
use crate::flying_obstacle::rock::spawn_rock;
use crate::web::Web;

pub struct FlyingObstaclePlugin;

#[derive(Resource)]
pub struct RockSpawnTimer {
    pub timer: Timer,
}

pub struct ParabolicMotion {
    pub start_pos: Vec3,
    pub velocity: Vec3,
    pub gravity: Vec3,
    pub time: f32
}

#[derive(Component)]
pub struct FlyingObstacle {
    pub motion: ParabolicMotion,
    pub despawn_duration: Duration,
    pub spin: Vec3
}

impl FlyingObstacle {
    pub fn new(start_pos: Vec3, velocity: Vec3, gravity: Vec3, spin: Vec3, lifespan: Duration) -> Self {
        FlyingObstacle {
            motion: ParabolicMotion {
                start_pos,
                velocity,
                gravity,
                time: 0.0
            },
            despawn_duration: lifespan,
            spin
        }
    }
}

impl Plugin for FlyingObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_obstacle);
        app.add_systems(Update, spawn_rock);
        app.insert_resource(RockSpawnTimer {
            timer: Timer::new(
                Duration::from_millis(1000 ),
                TimerMode::Repeating,
            ),
        });
    }
}

fn move_obstacle(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut FlyingObstacle, &mut Transform, Entity)>,
) {
    for (mut obstacle, mut transform, entity) in query.iter_mut() {
        obstacle.motion.time += time.delta_seconds();

        // parabolic motion
        transform.translation.x = obstacle.motion.start_pos.x + obstacle.motion.velocity.x * obstacle.motion.time;
        transform.translation.z = obstacle.motion.start_pos.z + obstacle.motion.velocity.z * obstacle.motion.time;

        transform.translation.y = obstacle.motion.start_pos.y + obstacle.motion.velocity.y * obstacle.motion.time
            + 0.5 * obstacle.motion.gravity.y * obstacle.motion.time * obstacle.motion.time;

        // rotation tumble
        transform.rotation = Quat::from_euler(EulerRot::XYZ,
                                              obstacle.spin.x * obstacle.motion.time,
                                              obstacle.spin.y * obstacle.motion.time,
                                              obstacle.spin.z * obstacle.motion.time);

        if obstacle.motion.time > obstacle.despawn_duration.as_secs_f32() {
            commands.entity(entity).despawn();
        }
    }
}
