mod render;
mod spring;

use std::f32::consts::PI;
use crate::{tree::get_arena_center, web::spring::Spring};
use bevy::prelude::*;
use render::{clear_web, render_web};

pub struct WebSimulationPlugin;

pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub force: Vec3,
    pub mass: f32,
    pub pinned: bool,
}

#[derive(Component)]
pub struct Web {
    pub particles: Vec<Particle>,
    pub springs: Vec<Spring>,
}

impl Default for Web {
    fn default() -> Self {
        Web {
            particles: vec![],
            springs: vec![],
        }
    }
}

impl Plugin for WebSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_simulation);
        app.add_systems(Update, clear_web);
        app.add_systems(Update, render_web.after(clear_web));
        app.add_systems(Startup, spawn_simulation);
    }
}

fn spawn_simulation(mut commands: Commands) {
    println!("WebSimulationPlugin init");
    let web = generate_web(5, 7, 1.0);
    commands.spawn(web);
}

fn generate_2_particle_example() -> Web {
    let arena_center = get_arena_center();
    let mut web: Web = Default::default();
    web.particles.push(Particle {
        position: arena_center + Vec3::new(0.0, 0.0, 0.0),
        velocity: Default::default(),
        force: Default::default(),
        mass: 1.0,
        pinned: false,
    });
    web.particles.push(Particle {
        position: arena_center + Vec3::new(0.0, 1.0, 0.0),
        velocity: Default::default(),
        force: Default::default(),
        mass: 1.0,
        pinned: true,
    });
    web.springs.push(Spring {
        first_index: 0,
        second_index: 1,
        stiffness: 100.0,
        damping: 1.0,
        rest_length: 1.0,
    });
    web
}

fn generate_web(row_count: usize, col_count: usize, size: f32) -> Web {
    let arena_center = get_arena_center();
    let mut web: Web = Default::default();
    let mass = 0.3;
    web.particles.push(Particle {
        position: arena_center,
        velocity: Default::default(),
        force: Default::default(),
        mass,
        pinned: false,
    });
    for i in 0..row_count {
        for j in 0..col_count {
            let left = if i == 0 { 0 } else { web.particles.len() - col_count };
            let prev = web.particles.len() - 1;

            let r = (i as f32 + 1.0) / row_count as f32 * size;
            let θ = j as f32 / col_count as f32 * 2.0 * PI;

            let dir = Vec3::new(θ.cos(), θ.sin(), 0.0);

            let pos = arena_center + dir * r;

            web.particles.push(Particle {
                position: pos,
                velocity: Default::default(),
                force: Default::default(),
                mass,
                pinned: i == row_count - 1,
            });

            let new = web.particles.len() - 1;

            web.springs.push(Spring::new(&web, new, left, 400.0, 1.0));

            if i != row_count - 1 && j != 0 {
                web.springs.push(Spring::new(&web, new, prev, 400.0, 1.0));
            }

            if j == col_count - 1 {
                web.springs.push(Spring::new(&web, new, web.particles.len() - col_count, 400.0, 1.0));
            }
        }
    }
    web
}

fn update_simulation(mut query: Query<(&mut Web)>, time: Res<Time>) {
    let h = time.delta_seconds();
    for mut web in &mut query {
        for i in 0..web.particles.len() {
            if web.particles[i].pinned {
                continue;
            }

            web.particles[i].force = Vec3::new(0.0, -9.81 * web.particles[i].mass, 0.0);
        }

        for j in 0..web.springs.len() {
            let force = web.springs[j].get_force_p1(&web);
            let p1 = web.springs[j].first_index;
            let p2 = web.springs[j].second_index;
            if !web.particles[p1].pinned {
                web.particles[p1].force += force;
            }
            if !web.particles[p2].pinned {
                web.particles[p2].force -= force;
            }
        }

        for particle in &mut web.particles {
            if particle.pinned {
                continue;
            }

            particle.velocity += particle.force / particle.mass * h;
            particle.position += particle.velocity * h;
        }
    }
}
