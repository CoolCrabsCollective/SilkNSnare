use bevy::{log, prelude::*};
use bevy_rapier3d::prelude::{Collider, CollisionEvent, ContactForceEvent};

use crate::{config::冰淇淋, flying_insect::flying_insect::FlyingInsect};

use super::{render::WebSegmentCollision, spring::Spring, Web};

pub const ENSNARE_MY_BALLS: bool = false;

#[derive(Component)]
pub struct Ensnared;

#[derive(Debug, Clone)]
pub struct EnsnaredEntity {
    /// the entity that is snared in the web
    pub entity: Entity,
    /// the position along the spring at which it's ensnared.
    ///  ranges from 0 (first particle) -> 1 (second particle)
    pub snare_position: f32,
    pub mass: f32,
}

impl EnsnaredEntity {
    pub fn snare_position_from_world_space(
        snare_position_world_space: Vec3,
        first_particle_position: Vec3,
        second_particle_position: Vec3,
    ) -> f32 {
        (snare_position_world_space - first_particle_position).length()
            / (second_particle_position - first_particle_position).length()
    }

    pub fn from_snare_position_world_space(
        entity: Entity,
        mass: f32,
        snare_position_world_space: Vec3,
        first_particle_position: Vec3,
        second_particle_position: Vec3,
    ) -> Self {
        let snare_position = Self::snare_position_from_world_space(
            snare_position_world_space,
            first_particle_position,
            second_particle_position,
        )
        .clamp(0.0, 1.0);

        error!("不冰淇淋");
        //assert!(snare_position >= 0.0 && snare_position <= 1.0);

        EnsnaredEntity {
            entity,
            snare_position,
            mass,
        }
    }
    pub fn snare_position_world_space(
        &self,
        first_particle_position: Vec3,
        second_particle_position: Vec3,
    ) -> Vec3 {
        ((1.0 - self.snare_position) * first_particle_position)
            + (self.snare_position * second_particle_position)
    }
}

pub fn ensnare_enemies(
    mut commands: Commands,
    enemies_query: Query<(&FlyingInsect, &Transform), Without<Ensnared>>,
    web_segment_collisions_query: Query<&WebSegmentCollision>,
    mut web_query: Query<&mut Web>,
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    let Ok(mut web) = web_query.get_single_mut() else {
        error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    let mut handle_ensnare =
        |enemy_entity: Entity,
         (enemy, enemy_transform): (&FlyingInsect, &Transform),
         web_segment_collision: &WebSegmentCollision| {
            // warn!("Handling ensnare");
            let i1 = web.springs[web_segment_collision.spring_index].first_index;
            let i2 = web.springs[web_segment_collision.spring_index].second_index;
            let first_particle_position = web.particles[i1].position;
            let second_particle_position = web.particles[i2].position;
            let spring = &mut web.springs[web_segment_collision.spring_index];
            let enemy_position = enemy_transform.translation;

            let snare_position = (enemy_position - first_particle_position)
                .dot(second_particle_position - first_particle_position)
                / (second_particle_position - first_particle_position)
                    .dot(second_particle_position - first_particle_position);

            if snare_position < -0.1 || snare_position > 1.1 {
                error!(
                    "不冰淇淋, \
            first_particle_position={first_particle_position}, \
            second_particle_position={second_particle_position}, \
            enemy_position={enemy_position}, \
            snare_position={snare_position}"
                );
            }

            let t = snare_position.clamp(0.0, 1.0);

            let ensnared_entity = EnsnaredEntity {
                entity: enemy_entity,
                snare_position: t,
                mass: enemy.weight,
            };

            commands.entity(enemy_entity).insert(Ensnared);

            spring.ensnared_entities.push(ensnared_entity);
            web.particles[i1].impulse = Vec3::new(0.0, 0.0, 1.0) * 10000.0 * (1.0 - t);
            web.particles[i1].impulse_duration = 0.1;
            web.particles[i2].impulse = Vec3::new(0.0, 0.0, 1.0) * 10000.0 * t;
            web.particles[i2].impulse_duration = 0.1;
        };

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity_a, entity_b, _) = collision_event {
            match (
                enemies_query.get(*entity_a),
                enemies_query.get(*entity_b),
                web_segment_collisions_query.get(*entity_a),
                web_segment_collisions_query.get(*entity_b),
            ) {
                (Ok(enemy), Err(_), Ok(web_segment_collision), Err(_)) => {
                    handle_ensnare(*entity_a, enemy, web_segment_collision);
                }
                (Ok(enemy), Err(_), Err(_), Ok(web_segment_collision)) => {
                    handle_ensnare(*entity_a, enemy, web_segment_collision);
                }
                (Err(_), Ok(enemy), Ok(web_segment_collision), Err(_)) => {
                    handle_ensnare(*entity_b, enemy, web_segment_collision);
                }
                (Err(_), Ok(enemy), Err(_), Ok(web_segment_collision)) => {
                    handle_ensnare(*entity_b, enemy, web_segment_collision);
                }
                _ => {}
            }
        }
    }
}

pub fn debug_ensnare_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut web_query: Query<&mut Web>,
) {
    if !ENSNARE_MY_BALLS {
        return;
    }

    let Ok(mut web_data) = web_query.get_single_mut() else {
        error!("ERROR NO WEB OR MORE THAN ONE WEB");
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

            let entity = commands.spawn((
                PbrBundle {
                    mesh: debug_mesh.clone(),
                    material: debug_material.clone(),
                    ..default()
                },
                Ensnared,
            ));

            spring.ensnared_entities.push(EnsnaredEntity {
                entity: entity.id(),
                snare_position: random_position,
                mass: 0.0,
            });
        }
    }
}

pub fn update_ensnared_entities(
    web_query: Query<&Web>,
    mut transform_query: Query<&mut Transform>,
) {
    let Ok(web) = web_query.get_single() else {
        log::error!("ERROR NO WEB OR MORE THAN ONE WEB");
        return;
    };

    for spring in web.springs.iter() {
        for ensnared_entity in spring.ensnared_entities.iter() {
            let Ok(mut ensnared_entity_transform) = transform_query.get_mut(ensnared_entity.entity)
            else {
                continue;
            };

            ensnared_entity_transform.translation = ensnared_entity.snare_position_world_space(
                web.particles[spring.first_index].position,
                web.particles[spring.second_index].position,
            );
        }
    }
}

pub fn split_ensnared_entities_for_spring_split(
    web: &Web,
    old_spring: &Spring,
    split_position: Vec3,
) -> (Vec<EnsnaredEntity>, Vec<EnsnaredEntity>) {
    let new_particle_t = EnsnaredEntity::snare_position_from_world_space(
        split_position,
        web.particles[old_spring.first_index].position,
        web.particles[old_spring.second_index].position,
    );

    assert!(new_particle_t >= 0.0 && new_particle_t <= 1.0);

    let new_spring_1_ensnared_entities = old_spring
        .ensnared_entities
        .iter()
        .filter(|ensnared| ensnared.snare_position <= new_particle_t)
        .map(|ensnared| {
            let snare_position_world_space = ensnared.snare_position_world_space(
                web.particles[old_spring.first_index].position,
                web.particles[old_spring.second_index].position,
            );

            EnsnaredEntity::from_snare_position_world_space(
                ensnared.entity.clone(),
                ensnared.mass,
                snare_position_world_space,
                web.particles[old_spring.first_index].position,
                split_position,
            )
        })
        .collect();
    let new_spring_2_ensnared_entities = old_spring
        .ensnared_entities
        .iter()
        .filter(|ensnared| ensnared.snare_position > new_particle_t)
        .map(|ensnared| {
            let snare_position_world_space = ensnared.snare_position_world_space(
                web.particles[old_spring.first_index].position,
                web.particles[old_spring.second_index].position,
            );

            EnsnaredEntity::from_snare_position_world_space(
                ensnared.entity.clone(),
                ensnared.mass,
                snare_position_world_space,
                split_position,
                web.particles[old_spring.second_index].position,
            )
        })
        .collect();

    (
        new_spring_1_ensnared_entities,
        new_spring_2_ensnared_entities,
    )
}
