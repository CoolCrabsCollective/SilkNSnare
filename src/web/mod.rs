mod spring;

use bevy::prelude::*;
use crate::web::spring::Spring;

pub struct WebSimulationPlugin;

pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub force: Vec3,
    pub mass: f32,
    pub pinned: bool
}

#[derive(Component)]
pub struct Web {
    pub particles: Vec<Particle>,
    pub springs: Vec<Spring>
}

impl Default for Web {
    fn default() -> Self {
        Web {
            particles: vec![], springs: vec![]
        }
    }
}

impl Plugin for WebSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_simulation);
        app.add_systems(Startup, spawn_simulation);
    }
}

fn spawn_simulation(mut commands: Commands) {
    println!("WebSimulationPlugin init");
    let mut web: Web = Default::default();
    web.particles.push(Particle {
        position: Vec3::new(0.0, 0.0, 0.0),
        velocity: Default::default(),
        force: Default::default(),
        mass: 1.0,
        pinned: false,
    });
    web.particles.push(Particle {
        position: Vec3::new(0.0, 1.0, 0.0),
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
    commands.spawn(web);
}

fn update_simulation(mut query: Query<(&mut Web)>,
                     time: Res<Time>) {
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
            println!("Particle position: {}", particle.position)
        }
    }
}