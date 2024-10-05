mod spring;

use bevy::prelude::*;
use crate::web::spring::Spring;

pub struct WebSimulationPlugin;


pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub pinned: bool
}

#[derive(Component)]
pub struct Web {
    pub particles: Vec<Particle>,
    pub segments: Vec<Spring>
}

impl Default for Web {
    fn default() -> Self {
        let web = Web {
            particles: vec![], segments: vec![]
        };

        web
    }
}

impl Plugin for WebSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_simulation);
        println!("WebSimulationPlugin init");
        let mut web: Web = Default::default();
        web.particles.push(Particle {
            position: Vec3::new(0.0, 0.0, 0.0),
            velocity: Default::default(),
            mass: 1.0,
            pinned: false,
        });
        web.particles.push(Particle {
            position: Vec3::new(0.0, 1.0, 0.0),
            velocity: Default::default(),
            mass: 1.0,
            pinned: true,
        });
        web.segments.push(Spring {
            first_index: 0,
            second_index: 1,
            stiffness: 1.0,
            damping: 1.0,
            rest_length: 1.0,
        });
    }
}

fn update_simulation(mut query: Query<(&mut Web)>,) {
    for mut web in &mut query {
        for particle in &mut web.particles {
            let mut force: f32 = 0.0;

        }
    }
}