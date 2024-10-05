use bevy::{log, prelude::*};

use super::{spring::Spring, Web};

#[derive(Debug, Clone)]
pub struct EnsnaredEntity {
    /// the entity that is snared in the web
    pub entity: Entity,
    /// the position along the spring at which it's ensnared.
    ///  ranges from 0 (first particle) -> 1 (second particle)
    pub snare_position: f32,
}

impl EnsnaredEntity {
    pub fn get_position_world_space(&self, web: &Web, spring: &Spring) -> Vec3 {
        let first_particle_position = web.particles[spring.first_index].position;
        let second_particle_position = web.particles[spring.second_index].position;

        ((1.0 - self.snare_position) * first_particle_position)
            + (self.snare_position * second_particle_position)
    }
}

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

            ensnared_entity_transform.translation =
                ensnared_entity.get_position_world_space(web_data, spring);
        }
    }
}

pub fn split_ensnared_entities_for_spring_split(
    web: &Web,
    old_spring: &Spring,
    split_position: Vec3,
) -> (Vec<EnsnaredEntity>, Vec<EnsnaredEntity>) {
    let new_particle_t = (split_position - web.particles[old_spring.first_index].position).length()
        / (web.particles[old_spring.second_index].position
            - web.particles[old_spring.first_index].position)
            .length();

    let new_spring_1_ensnared_entities = old_spring
        .ensnared_entities
        .iter()
        .filter(|ensnared| ensnared.snare_position <= new_particle_t)
        .map(|ensnared| {
            let ensnared_position_world_space = ensnared.get_position_world_space(web, old_spring);

            let snare_position = (ensnared_position_world_space
                - web.particles[old_spring.first_index].position)
                .length()
                / (split_position - web.particles[old_spring.first_index].position).length();

            dbg!(ensnared.snare_position, snare_position);

            EnsnaredEntity {
                entity: ensnared.entity.clone(),
                snare_position,
            }
        })
        .collect();
    let new_spring_2_ensnared_entities = old_spring
        .ensnared_entities
        .iter()
        .filter(|ensnared| ensnared.snare_position > new_particle_t)
        .map(|ensnared| {
            let ensnared_position_world_space = ensnared.get_position_world_space(web, old_spring);

            let snare_position = (ensnared_position_world_space - split_position).length()
                / (web.particles[old_spring.second_index].position - split_position).length();

            dbg!(
                ensnared_position_world_space,
                split_position,
                web.particles[old_spring.first_index].position,
                web.particles[old_spring.second_index].position,
                ensnared_position_world_space - split_position,
                web.particles[old_spring.second_index].position - split_position,
                ensnared.snare_position,
                snare_position
            );

            EnsnaredEntity {
                entity: ensnared.entity.clone(),
                snare_position,
            }
        })
        .collect();

    (
        new_spring_1_ensnared_entities,
        new_spring_2_ensnared_entities,
    )
}
