use bevy::{log, prelude::*, sprite::Mesh2dHandle};

use super::{spring::EnsnaredEntity, Web};

pub fn debug_ensnare_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut web_query: Query<&mut Web>,
) {
    let Ok(mut web_data) = web_query.get_single_mut() else {
        log::error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    let debug_mesh = meshes.add(Sphere { radius: 0.01 }.mesh().ico(5).unwrap());

    let debug_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 0.8),
        ..default()
    });

    for spring in web_data.springs.iter_mut() {
        for _ in 0..2 {
            let random_position: f32 = rand::random();

            let entity = commands.spawn(PbrBundle {
                mesh: debug_mesh.clone(),
                material: debug_material.clone(),
                ..default()
            });

            spring.ensnared_entities.push(EnsnaredEntity {
                entity: entity.id(),
                snare_position: random_position,
            });
        }
    }
}

pub fn update_ensnared_entities(
    web_query: Query<&Web>,
    mut transform_query: Query<&mut Transform>,
) {
    let Ok(web_data) = web_query.get_single() else {
        log::error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    for spring in web_data.springs.iter() {
        for ensnared_entity in spring.ensnared_entities.iter() {
            let Ok(mut ensnared_entity_transform) = transform_query.get_mut(ensnared_entity.entity)
            else {
                continue;
            };

            let snare_position = ensnared_entity.snare_position;
            let first_particle_position = web_data.particles[spring.first_index].position;
            let second_particle_position = web_data.particles[spring.second_index].position;

            ensnared_entity_transform.translation = ((1.0 - snare_position)
                * first_particle_position)
                + (snare_position * second_particle_position)
        }
    }
}
