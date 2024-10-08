pub mod ensnare;
mod render;
pub mod spring;

use crate::flying_insect::flying_insect::FlyingInsect;
use crate::flying_obstacle::flying_obstacle::FlyingObstacle;
use crate::tree::{get_arena_center, 照相机里有点吗};
use crate::web::ensnare::{free_enemy_from_web, split_ensnared_entities_for_spring_split};
use crate::web::render::WebSegmentCollision;
use crate::web::spring::Spring;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;
use bevy_rapier3d::prelude::Collider;
use ensnare::{debug_ensnare_entities, ensnare_enemies, update_ensnared_entities};
use render::{clear_web, render_web};
use std::f32::consts::PI;

pub const START_WITH_A_WEB: bool = false; // FOR NOOBS
pub static mut splitting_spring: i32 = 0;
pub static mut destroy_call: i32 = 0;

pub struct WebSimulationPlugin;

pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub force: Vec3,
    pub impulse: Vec3,
    pub impulse_duration: f32,
    pub mass: f32,
    pub pinned: bool,
}

#[derive(Component)]
pub struct Breaker;

#[derive(Component)]
pub struct Web {
    pub particles: Vec<Particle>,
    pub springs: Vec<Spring>,
    pub mass_per_unit_length: f32,
}

impl Web {
    pub fn 破壊する(
        &mut self,
        ポイント: Vec3,
        insect_query: &Query<&FlyingInsect>,
        commands: &mut Commands,
    ) {
        if !照相机里有点吗(ポイント) {
            return;
        }

        unsafe {
            println!("Calling destroy: {}", destroy_call);
            destroy_call += 1;
        }

        let カウント = self.springs.len();
        for インデックス in 0..カウント {
            let 粒子1 = self.particles[self.springs[インデックス].first_index].position;
            let 粒子2 = self.particles[self.springs[インデックス].second_index].position;

            let 差1 = (ポイント - 粒子1).normalize();
            let 差2 = (ポイント - 粒子2).normalize();

            if 差1.dot(差2) > -0.98 || 差1.is_nan() || 差2.is_nan() {
                continue;
            }

            unsafe {
                println!(
                    "Splitting spring: {}, 差1: {}, 差2: {}",
                    splitting_spring, 差1, 差2
                );
                splitting_spring += 1;
            }
            let あるバネのパラメーター = (ポイント - 粒子1).length() / (粒子2 - 粒子1).length();
            while self.springs[インデックス].ensnared_entities.len() > 0 {
                let 罠にかかった = &self.springs[インデックス].ensnared_entities[0];
                free_enemy_from_web(
                    commands,
                    罠にかかった.entity,
                    insect_query.get(罠にかかった.entity).ok(),
                    self,
                );
            }

            let あるバネのポイント =
                粒子2 * あるバネのパラメーター + 粒子1 * (1.0 - あるバネのパラメーター);

            self.particles.push(Particle {
                position: あるバネのポイント,
                velocity: Default::default(),
                force: Default::default(),
                impulse: Vec3::new(0.0, 0.0, 1.0) * 10000.0,
                impulse_duration: 0.1,
                mass: 0.0,
                pinned: false,
            });

            self.particles.push(Particle {
                position: あるバネのポイント,
                velocity: Default::default(),
                force: Default::default(),
                impulse: Vec3::new(0.0, 0.0, 1.0) * 10000.0,
                impulse_duration: 0.1,
                mass: 0.0,
                pinned: false,
            });

            let 新粒子1 = self.particles.len() - 2;
            let 新粒子2 = self.particles.len() - 1;

            self.springs.push(Spring {
                first_index: self.springs[インデックス].first_index,
                second_index: 新粒子1,
                stiffness: self.springs[インデックス].stiffness,
                damping: self.springs[インデックス].damping,
                rest_length: self.springs[インデックス].rest_length
                    * あるバネのパラメーター,
                ensnared_entities: vec![],
            });

            self.springs.push(Spring {
                first_index: 新粒子2,
                second_index: self.springs[インデックス].second_index,
                stiffness: self.springs[インデックス].stiffness,
                damping: self.springs[インデックス].damping,
                rest_length: self.springs[インデックス].rest_length
                    * (1.0 - あるバネのパラメーター),
                ensnared_entities: vec![],
            });

            self.springs.swap_remove(インデックス);
        }
    }
}

impl Web {
    pub fn get_spring(&self, p1: usize, p2: usize) -> Option<usize> {
        for i in 0..self.springs.len() {
            let spring = &self.springs[i];
            if spring.first_index == p1 && spring.second_index == p2
                || spring.first_index == p2 && spring.second_index == p1
            {
                return Some(i);
            }
        }
        None
    }
}

impl Default for Web {
    fn default() -> Self {
        Web {
            particles: vec![],
            springs: vec![],
            mass_per_unit_length: 0.1,
        }
    }
}

impl Web {
    pub fn get_particle_index(&self, pos: Vec3, ε: f32) -> Option<usize> {
        for i in 0..self.particles.len() {
            if self.particles[i].position.distance_squared(pos) < ε * ε {
                return Some(i);
            }
        }
        None
    }

    pub fn split_spring(&mut self, spring_index: usize, position: Vec3) {
        self.particles.push(Particle {
            position: position,
            velocity: Default::default(),
            force: Default::default(),
            impulse: Default::default(),
            impulse_duration: 0.0,
            mass: 0.0,
            pinned: false,
        });

        let old_spring: Spring = self.springs.swap_remove(spring_index);
        let t = (position - self.particles[old_spring.first_index].position).length()
            / (self.particles[old_spring.second_index].position
                - self.particles[old_spring.first_index].position)
                .length();

        let (new_spring_1_ensnared_entities, new_spring_2_ensnared_entities) =
            split_ensnared_entities_for_spring_split(&self, &old_spring, position);

        self.springs.push(Spring::new_with_length(
            self,
            old_spring.first_index,
            self.particles.len() - 1,
            20.0,
            0.5,
            old_spring.rest_length * t,
            new_spring_1_ensnared_entities,
        ));
        self.springs.push(Spring::new_with_length(
            self,
            self.particles.len() - 1,
            old_spring.second_index,
            20.0,
            0.5,
            old_spring.rest_length * (1.0 - t),
            new_spring_2_ensnared_entities,
        ));
    }
}

impl Plugin for WebSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_simulation);
        app.add_systems(Update, update_simulation);

        app.add_systems(Update, clear_web);
        app.add_systems(Update, render_web.after(clear_web));

        app.add_systems(Startup, debug_ensnare_entities.after(spawn_simulation));
        app.add_systems(Update, ensnare_enemies);
        app.add_systems(Update, update_ensnared_entities);

        app.add_systems(Update, handle_obstacles_destroy_web);
    }
}

fn spawn_simulation(mut commands: Commands) {
    println!("WebSimulationPlugin init");
    let web = if START_WITH_A_WEB {
        generate_web(4, 8, 1.0, 0.1, 30.0, 0.5)
    } else {
        Default::default()
    };
    commands.spawn(web);
}

fn generate_2_particle_example() -> Web {
    let arena_center = get_arena_center();
    let mut web: Web = Default::default();
    web.particles.push(Particle {
        position: arena_center + Vec3::new(0.0, 0.0, 0.0),
        velocity: Default::default(),
        force: Default::default(),
        impulse: Default::default(),
        impulse_duration: 0.0,
        mass: 0.0,
        pinned: false,
    });
    web.particles.push(Particle {
        position: arena_center + Vec3::new(0.0, 1.0, 0.0),
        velocity: Default::default(),
        force: Default::default(),
        impulse: Default::default(),
        impulse_duration: 0.0,
        mass: 0.0,
        pinned: true,
    });
    web.springs.push(Spring {
        first_index: 0,
        second_index: 1,
        stiffness: 100.0,
        damping: 1.0,
        rest_length: 1.0,
        ensnared_entities: vec![],
    });
    web
}

fn generate_web(
    row_count: usize,
    col_count: usize,
    size: f32,
    mass_density: f32,
    stiffness: f32,
    damping: f32,
) -> Web {
    let arena_center = get_arena_center();
    let mut web: Web = Default::default();
    web.mass_per_unit_length = mass_density;
    web.particles.push(Particle {
        position: arena_center,
        velocity: Default::default(),
        force: Default::default(),
        impulse: Default::default(),
        impulse_duration: 0.0,
        mass: 0.0,
        pinned: false,
    });
    for i in 0..row_count {
        for j in 0..col_count {
            let left = if i == 0 {
                0
            } else {
                web.particles.len() - col_count
            };
            let prev = web.particles.len() - 1;

            let r = (i as f32 + 1.0) / row_count as f32 * size;
            let θ = j as f32 / col_count as f32 * 2.0 * PI;

            let dir = Vec3::new(θ.cos(), θ.sin(), 0.0);

            let pos = arena_center + dir * r;

            web.particles.push(Particle {
                position: pos,
                velocity: Default::default(),
                force: Default::default(),
                impulse: Default::default(),
                impulse_duration: 0.0,
                mass: 0.0,
                pinned: i == row_count - 1,
            });

            let new = web.particles.len() - 1;

            web.springs
                .push(Spring::new(&web, new, left, stiffness, damping, vec![]));

            if i != row_count - 1 && j != 0 {
                web.springs
                    .push(Spring::new(&web, new, prev, stiffness, damping, vec![]));

                if j == col_count - 1 {
                    web.springs.push(Spring::new(
                        &web,
                        new,
                        web.particles.len() - col_count,
                        stiffness,
                        damping,
                        vec![],
                    ));
                }
            }
        }
    }
    web
}

fn update_simulation(mut query: Query<&mut Web>, time: Res<Time>) {
    let h = time.delta_seconds();
    let desired_h = 0.001;
    let count: i32 = (h / desired_h).ceil() as i32;
    let air_damping = 0.5;

    for i in 0..count {
        for mut web in &mut query {
            step(
                &mut *web,
                air_damping,
                if i == count - 1 {
                    h - (count - 1) as f32 * desired_h
                } else {
                    desired_h
                },
            );
        }
    }
}

pub fn step(web: &mut Web, air_damping: f32, h: f32) {
    for i in 0..web.particles.len() {
        if web.particles[i].pinned {
            continue;
        }
        web.particles[i].mass = 0.0;
        web.particles[i].force = Vec3::new(0.0, 0.0, 0.0);
    }

    for j in 0..web.springs.len() {
        let force = web.springs[j].get_force_p1(&web);
        let p1 = web.springs[j].first_index;
        let p2 = web.springs[j].second_index;

        // calculate mass of ensnared_entities
        for ensnared in web.springs[j].ensnared_entities.clone() {
            if !web.particles[p1].pinned {
                web.particles[p1].mass += ensnared.mass * (1.0 - ensnared.snare_position);
            }

            if !web.particles[p2].pinned {
                web.particles[p2].mass += ensnared.mass * (ensnared.snare_position);
            }
        }

        if !web.particles[p1].pinned {
            web.particles[p1].force += force;
            web.particles[p1].mass += web.mass_per_unit_length * web.springs[j].rest_length / 2.0;
        }
        if !web.particles[p2].pinned {
            web.particles[p2].force -= force;
            web.particles[p2].mass += web.mass_per_unit_length * web.springs[j].rest_length / 2.0;
        }
    }

    for particle in &mut web.particles {
        if particle.pinned {
            continue;
        }

        particle.force.y -= 9.81 * particle.mass;
        particle.force += particle.velocity * -air_damping;

        if particle.impulse_duration > 0.0 {
            particle.force += particle.impulse * h;
            particle.impulse_duration -= h;
            if particle.impulse_duration <= 0.0 {
                particle.impulse = Vec3::ZERO;
                particle.impulse_duration = 0.0;
            }
        }

        particle.velocity += particle.force / particle.mass * h;
        particle.position += particle.velocity * h;
    }
}

fn handle_obstacles_destroy_web(
    mut commands: Commands,
    mut web_query: Query<&mut Web>,
    insect_query: Query<&FlyingInsect>,
    mut collision_events: EventReader<CollisionEvent>,
    web_segment_collisions_query: Query<&WebSegmentCollision>,
    mut obstacle_query: Query<(&mut FlyingObstacle, &mut Transform), Without<Breaker>>,
) {
    let Ok(mut web) = web_query.get_single_mut() else {
        panic!("FUCK NO WEB");
    };

    let mut call_counter = 0;

    let mut handle_spring_break =
        |web: &mut Web, web_segment: &WebSegmentCollision, obstacle_trans: Vec3, entity: Entity| {
            let spring = web.springs.get(web_segment.spring_index).unwrap();
            let first_particle_position = web.particles.get(spring.first_index).unwrap().position;
            let second_particle_position = web.particles.get(spring.second_index).unwrap().position;

            let obstacle_position_t = (obstacle_trans - first_particle_position)
                .dot(second_particle_position - first_particle_position)
                / (second_particle_position - first_particle_position)
                    .dot(second_particle_position - first_particle_position);

            if obstacle_position_t < -0.1 || obstacle_position_t > 1.1 {
                error!(
                    "不冰淇淋, \
            first_particle_position={first_particle_position}, \
            second_particle_position={second_particle_position}, \
            obstacle_trans={obstacle_trans}, \
            obstacle_position_t={obstacle_position_t}"
                );
            }

            call_counter += 1;
            assert!(call_counter < 10000);

            let t = obstacle_position_t.clamp(0.0, 1.0);

            let obstacle_position =
                ((1.0 - t) * first_particle_position) + (t * second_particle_position);
            web.破壊する(obstacle_position, &insect_query, &mut commands);
            commands.entity(entity).insert(Breaker);
            commands.entity(entity).remove::<Collider>();
        };

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity_a, entity_b, _) = collision_event {
            match (
                obstacle_query.get(*entity_a),
                obstacle_query.get(*entity_b),
                web_segment_collisions_query.get(*entity_a),
                web_segment_collisions_query.get(*entity_b),
            ) {
                (Ok((_, trans)), Err(_), Ok(web_segment), Err(_)) => {
                    handle_spring_break(&mut web, web_segment, trans.translation, *entity_a);
                }
                (Ok((_, trans)), Err(_), Err(_), Ok(web_segment)) => {
                    handle_spring_break(&mut web, web_segment, trans.translation, *entity_a);
                }
                (Err(_), Ok((_, trans)), Ok(web_segment), Err(_)) => {
                    handle_spring_break(&mut web, web_segment, trans.translation, *entity_b);
                }
                (Err(_), Ok((_, trans)), Err(_), Ok(web_segment)) => {
                    handle_spring_break(&mut web, web_segment, trans.translation, *entity_b);
                }
                _ => {}
            }
        }
    }
}
