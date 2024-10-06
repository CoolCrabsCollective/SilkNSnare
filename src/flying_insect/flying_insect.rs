use crate::flying_insect::fruit_fly::spawn_fruit_fly;
use bevy::app::{App, Plugin, Update};
use bevy::math::{Mat3, Vec3};
use bevy::prelude::{
    Commands, Component, Entity, Quat, Query, Res, Resource, Time, Timer, TimerMode, Transform,
};
use rand::Rng;
use std::f32::consts::PI;
use std::time::Duration;
use bevy::log::error;
use crate::web::ensnare::EnsnaredEntity;
use crate::web::Web;

pub struct FlyingInsectPlugin;

#[derive(Resource)]
pub struct FruitFlySpawnTimer {
    pub timer: Timer,
}

impl Plugin for FlyingInsectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_flying_insect);
        app.add_systems(Update, spawn_fruit_fly);
        app.insert_resource(FruitFlySpawnTimer {
            timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        });
    }
}

fn debug_enstare_bug(
    mut fly_query: Query<(&FlyingInsect, &Transform, Entity)>,
    mut web_query: Query<&mut Web>
) {
    let Ok(mut web_data) = web_query.get_single_mut() else {
        error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    let Ok((insect, _, fly_entity)) = fly_query.get_single_mut() else {
        error!("ERROR NO fly");
        return;
    };

    let spring = web_data.springs.get_mut(0).unwrap();
    let ensnared = EnsnaredEntity{
        entity: fly_entity.clone(),
        snare_position: 0.5,
        mass: insect.weight,
    };
    spring.ensnared_entities.push(ensnared);
}

pub struct BezierCurve {
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
}

impl BezierCurve {
    pub fn new(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        BezierCurve { p0, p1, p2, p3 }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        if t < 0.0 || t > 1.0 {
            panic!("CRINGE INVALID t USED FOR FLY BEZIER CURVE");
        }

        f32::powf(1.0 - t, 3.0) * self.p0
            + 3.0 * (1.0 - t) * (1.0 - t) * t * self.p1
            + 3.0 * (1.0 - t) * t * t * self.p2
            + t * t * t * self.p3
    }

    pub fn random_from_endpoints(p0: Vec3, p3: Vec3) -> Self {
        let (p1, p2) = generate_bezier_handles(p0, p3);
        BezierCurve::new(p0, p1, p2, p3)
    }

    pub fn tangent_at(&self, t: f32) -> Vec3 {
        3.0 * f32::powf(1.0 - t, 2.0) * (self.p1 - self.p0)
            + 6.0 * (1.0 - t) * t * (self.p2 - self.p1)
            + 3.0 * t * t * (self.p3 - self.p2)
    }
}

fn generate_bezier_handles(p0: Vec3, p3: Vec3) -> (Vec3, Vec3) {
    let mut rng = rand::thread_rng();

    let x_min = p0.x.min(p3.x);
    let y_min = p0.y.min(p3.y);
    let z_min = p0.z.min(p3.z);
    let x_max = p0.x.max(p3.x);
    let y_max = p0.y.max(p3.y);
    let z_max = p0.z.max(p3.z);

    let p1 = Vec3::new(
        rng.gen_range(x_min..x_max),
        rng.gen_range(y_min..y_max),
        rng.gen_range(z_min..z_max),
    );

    let p2 = Vec3::new(
        rng.gen_range(x_min..x_max),
        rng.gen_range(y_min..y_max),
        rng.gen_range(z_min..z_max),
    );

    if p2.z < p1.z {
        (p2, p1)
    } else {
        (p1, p2)
    }
}

#[derive(Component)]
pub struct FlyingInsect {
    speed: f32,
    progress: f32,
    weight: f32,
    offset: f32,
    path: BezierCurve,
    break_free_position: Vec3,
}

impl FlyingInsect {
    pub fn new(speed: f32, weight: f32, bezier: BezierCurve) -> Self {
        let mut rng = rand::thread_rng();
        FlyingInsect {
            speed,
            progress: 0.0,
            weight,
            offset: rng.gen_range(0.0..2.0 * PI),
            path: bezier,
            break_free_position: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

fn move_flying_insect(
    mut fly_query: Query<(&mut FlyingInsect, &mut Transform, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut fly, mut transform, entity) in &mut fly_query {
        fly.progress += time.delta_seconds() * fly.speed;

        if fly.progress > 1.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation = fly.path.at(fly.progress)
                + Vec3::new(
                    0.0,
                    (2.0 * PI * time.elapsed_seconds() * 0.65 + fly.offset).sin() * 0.05,
                    0.0,
                )
                + fly.break_free_position;

            let tangent = fly.path.tangent_at(fly.progress).normalize();
            let up = Vec3::new(0.0, 1.0, 0.0);
            let left = up.cross(tangent);
            let base_transform_mat = Mat3::from_cols(
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
            );

            transform.rotation = Quat::from_axis_angle(
                Vec3::new(0.0, 0.0, 1.0),
                ((PI / 2.0) * (2.0 * PI * time.elapsed_seconds() * 0.25).sin() - PI / 4.0) * 0.3,
            ) * Quat::from_mat3(&base_transform_mat);
        }
    }
}
