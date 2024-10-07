use super::fruit_fly::DAVID_DEBUG;
use crate::flying_insect::fruit_fly::{fly_hentai_anime_setup, spawn_fruit_fly, Animation};
use crate::game::GameState;
use crate::mesh_loader::{self, load_level, MeshLoader};
use crate::spider::Spider;
use crate::web::ensnare::{free_enemy_from_web, Ensnared};
use crate::web::Web;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::color::Color;
use bevy::math::{Mat3, Vec3};
use bevy::pbr::StandardMaterial;
use bevy::{log, prelude::*};
use bevy_health_bar3d::prelude::Percentage;
use rand::Rng;
use std::f32::consts::PI;
use std::time::Duration;

pub struct FlyingInsectPlugin;

#[derive(Resource)]
pub struct FruitFlySpawnTimer {
    pub timer: Timer,
}

#[derive(Resource)]
pub struct EnsnareRollModel {
    pub mesh: Handle<Mesh>,
    pub material: StandardMaterial,
    pub transform: Transform,
}

impl Plugin for FlyingInsectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_ensnare_roll_model.after(mesh_loader::setup));
        app.add_systems(Update, move_flying_insect.run_if(in_state(GameState::Game)));
        app.add_systems(Update, spawn_fruit_fly.run_if(in_state(GameState::Game)));
        app.add_systems(
            Update,
            insect_ensnared_tick_cooking_and_free.run_if(in_state(GameState::Game)),
        );
        app.add_systems(
            Update,
            update_ensnare_roll_model.run_if(in_state(GameState::Game)),
        );
        app.add_systems(
            Update,
            fly_hentai_anime_setup.run_if(in_state(GameState::Game)),
        );
        app.insert_resource(FruitFlySpawnTimer {
            timer: Timer::new(
                Duration::from_millis(if DAVID_DEBUG { 3000 } else { 500 }),
                TimerMode::Repeating,
            ),
        });

        app.insert_resource(EnsnareRollModel {
            mesh: Default::default(),
            material: Default::default(),
            transform: Default::default(),
        });
        app.insert_resource(Animation {
            animation_list: vec![],
            graph: Default::default(),
        });
    }
}

#[derive(Reflect)]
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

#[derive(Component, Reflect)]
pub struct FlyingInsect {
    pub speed: f32,
    pub progress: f32,
    pub weight: f32,
    pub offset: f32,
    pub path: BezierCurve,
    pub break_free_position: Vec3,
    pub snare_roll_progress: f32,
    pub snare_timer: Timer,
    pub cooking_progress: f32,
    pub cooking_timer: Timer,
    pub freed_timer: Timer,
    pub rolled_ensnare_entity: Option<Entity>,
}

impl FlyingInsect {
    pub fn new(speed: f32, weight: f32, bezier: BezierCurve) -> Self {
        let mut rng = rand::thread_rng();
        let mut new_flying = FlyingInsect {
            speed,
            progress: 0.0,
            weight,
            offset: rng.gen_range(0.0..2.0 * PI),
            path: bezier,
            break_free_position: Vec3::new(0.0, 0.0, 0.0),
            snare_roll_progress: 0.0,
            cooking_progress: 0.0,
            snare_timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
            cooking_timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
            freed_timer: Timer::new(Duration::from_secs(15), TimerMode::Repeating),
            rolled_ensnare_entity: None,
        };

        new_flying.freed_timer.pause();
        new_flying.cooking_timer.pause();

        new_flying
    }
}

fn move_flying_insect(
    mut fly_query: Query<(&mut FlyingInsect, &mut Transform, Entity), Without<Ensnared>>,
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
                    if DAVID_DEBUG {
                        0.0
                    } else {
                        (2.0 * PI * time.elapsed_seconds() * 0.65 + fly.offset).sin() * 0.05
                    },
                    0.0,
                )
                + fly.break_free_position;

            let tangent = fly.path.tangent_at(fly.progress).normalize();
            let up = Vec3::new(0.0, 1.0, 0.0);
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

fn insect_ensnared_tick_cooking_and_free(
    mut commands: Commands,
    mut web_query: Query<&mut Web>,
    mut insect_query: Query<(&mut FlyingInsect, Entity), With<Ensnared>>,
    time: Res<Time>,
) {
    for (mut insect, insect_entity) in insect_query.iter_mut() {
        if insect.freed_timer.paused() {
            insect.freed_timer.unpause();
        }

        insect.freed_timer.tick(time.delta());
        if insect.freed_timer.just_finished() && insect.snare_roll_progress < 1.0 {
            free_enemy_from_web(&mut commands, insect_entity, &mut *web_query.single_mut());
            if insect.rolled_ensnare_entity != None {
                commands
                    .entity(insect.rolled_ensnare_entity.unwrap())
                    .despawn();
            }

            insect.cooking_timer.reset();
            insect.cooking_timer.pause();

            insect.freed_timer.pause();
            insect.freed_timer.reset();

            insect.cooking_progress = 0.0;
        }

        if insect.snare_roll_progress >= 1.0 {
            if insect.cooking_timer.paused() {
                insect.cooking_timer.unpause();
                insect.freed_timer.reset();
            }

            insect.cooking_timer.tick(time.delta());

            insect.cooking_progress +=
                time.delta_seconds() / insect.cooking_timer.duration().as_secs_f32();

            if insect.cooking_timer.just_finished() {
                insect.cooking_timer.reset();
                insect.cooking_timer.pause();

                insect.freed_timer.reset();
            }
        }
    }
}

fn load_ensnare_roll_model(
    mut asset_server: ResMut<AssetServer>,
    mut mesh_loader: ResMut<MeshLoader>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ensnare_roll_model: ResMut<EnsnareRollModel>,
) {
    load_level("food.glb".into(), &mut asset_server, &mut mesh_loader);
}

fn update_ensnare_roll_model(
    mut commands: Commands,
    mut ensnare_roll_model: ResMut<EnsnareRollModel>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut insects_query: Query<(&mut FlyingInsect, &Transform, Entity), With<Ensnared>>,
    mut transform_query: Query<&mut Transform, Without<Ensnared>>,
) {
    for (mut insect, insect_trans, insect_entity) in insects_query.iter_mut() {
        if insect.snare_roll_progress > 0.0 {
            if insect.rolled_ensnare_entity == None {
                // log::warn!(
                //     "adding cocoon mesh: {:?}, {:?}",
                //     ensnare_roll_model.mesh.clone(),
                //     insect_trans.scale
                // );
                materials.add(ensnare_roll_model.material.clone());
                let entity = commands.spawn(PbrBundle {
                    mesh: ensnare_roll_model.mesh.clone(),
                    material: materials.add(ensnare_roll_model.material.clone()),
                    transform: insect_trans.with_scale(
                        1.5 * insect_trans.scale.x * ensnare_roll_model.transform.scale,
                    ),
                    ..default()
                });

                insect.rolled_ensnare_entity = Some(entity.id());
                return;
            }

            if let Ok(material_handle) = material_query.get(insect.rolled_ensnare_entity.unwrap()) {
                if let Some(material) = materials.get_mut(material_handle) {
                    // log::warn!(
                    //     "{:?}: snare_roll_progress={:?}, cooking_progress={:?}",
                    //     insect_entity,
                    //     insect.snare_roll_progress,
                    //     insect.cooking_progress
                    // );
                    let cook_t = (1.0 - insect.cooking_progress).clamp(0.0, 1.0);
                    material.base_color =
                        Color::srgba(1.0, cook_t, cook_t, insect.snare_roll_progress);
                } else {
                    log::error!("no mat 2");
                }
            } else {
                log::error!("no mat");
            };

            // Move ensnared roll model with insect
            let Ok(mut ensnared_trans) =
                transform_query.get_mut(insect.rolled_ensnare_entity.unwrap())
            else {
                panic!("NO TRANSFORM FOUND FOR ROLLED ENSNARE MODEL FUCKING SHIT");
            };

            ensnared_trans.translation = insect_trans.translation;
            ensnared_trans.rotation =
                insect_trans.rotation * Quat::from_axis_angle(Vec3::X, PI / 2.0);
        } else if insect.rolled_ensnare_entity != None {
            commands
                .entity(insect.rolled_ensnare_entity.unwrap())
                .despawn();

            insect.rolled_ensnare_entity = None;
        }
    }
}
