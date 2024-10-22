use crate::config::{COLLISION_GROUP_ALL, COLLISION_GROUP_PLAYER, COLLISION_GROUP_TERRAIN};
use crate::flying_insect::flying_insect::{BezierCurve, FlyingInsect};
use crate::game::GameState;
use crate::health::IsDead;
use crate::tree::{树里有小路吗, 树里有点吗, 照相机里有点吗};
use crate::ui::progress_bar::CookingInsect;
use crate::web::ensnare::{free_enemy_from_web, Ensnared};
use crate::web::spring::Spring;
use crate::web::{Particle, Web};
use bevy::ecs::observer::TriggerTargets;
use bevy::ecs::query::QueryEntityError;
use bevy::input::touch::TouchPhase;
use bevy::log;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_health_bar3d::configuration::BarSettings;
use bevy_health_bar3d::prelude::BarHeight;
use bevy_rapier3d::na::ComplexField;
use bevy_rapier3d::pipeline::CollisionEvent;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, Group, QueryFilter,
};
use std::f32::consts::PI;
use std::time::Duration;

pub const NNN: bool = false; // currently october, set this to true in november
pub const SPIDER_ROTATE_SPEED: f32 = 5.6;

pub struct SpiderPlugin;

#[derive(Resource)]
struct WebPlane {
    plane: Vec4, // ax + by + cz + d = 0
    left: Vec3,
}

#[derive(Event)]
pub struct SpiderFeastEvent(pub f32);

#[derive(Component)]
pub struct Spider {
    pub food: f32,
    pub max_food: f32,
    pub current_position: SpiderPosition,
    pub current_rotation: f32,
    pub target_position: SpiderPosition,
    pub touching_insects: Vec<Entity>,

    pub current_roll: f32,
    pub lerp_roll: f32,
}

#[derive(Copy, Clone)]
enum SpiderPosition {
    WEB(usize, f32),
    TREE(Vec3),
}

impl SpiderPosition {
    pub(crate) fn is_tree(&self) -> bool {
        match self {
            SpiderPosition::WEB(_, _) => false,
            SpiderPosition::TREE(_) => true,
        }
    }
}

impl SpiderPosition {
    pub fn to_vec3(&self, 网: &Web) -> Vec3 {
        match self {
            SpiderPosition::WEB(第, t) => {
                let spring = &网.springs[*第];
                let p1 = 网.particles[spring.first_index].position;
                let p2 = 网.particles[spring.second_index].position;

                p1 + (p2 - p1) * *t
            }
            SpiderPosition::TREE(p) => *p,
        }
    }

    pub fn 加(&self, 网: &Web, δ: Vec3) -> SpiderPosition {
        match self {
            SpiderPosition::WEB(第, t) => {
                let spring = &网.springs[*第];
                let p1 = 网.particles[spring.first_index].position;
                let p2 = 网.particles[spring.second_index].position;

                let p = p1 + (p2 - p1) * *t + δ;
                let 新t = ((p - p1).length() / (p2 - p1).length()).clamp(0.0, 1.0);
                SpiderPosition::WEB(*第, 新t)
            }
            SpiderPosition::TREE(p) => SpiderPosition::TREE(*p + δ),
        }
    }

    pub fn 同(&self, 其他: &SpiderPosition) -> bool {
        match (self, 其他) {
            (SpiderPosition::WEB(第1, t1), SpiderPosition::WEB(第2, t2)) => {
                *第1 == *第2 && *t1 == *t2
            }
            (SpiderPosition::TREE(p1), SpiderPosition::TREE(p2)) => {
                (*p1 - *p2).length_squared() < 0.01 * 0.01
            }
            _ => false,
        }
    }
}

impl Spider {
    pub fn new(max_food: f32, pos: Vec3) -> Self {
        Spider {
            food: max_food,
            max_food,
            target_position: SpiderPosition::TREE(pos),
            current_position: SpiderPosition::TREE(pos),
            current_rotation: 0.0,
            touching_insects: vec![],
            current_roll: 0.0,
            lerp_roll: 0.0,
        }
    }
}

impl Plugin for SpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spider);
        app.add_systems(Update, update_spider.run_if(in_state(GameState::Game)));
        app.add_systems(
            Update,
            handle_ensnared_insect_collision.run_if(in_state(GameState::Game)),
        );
        app.insert_resource(WebPlane {
            plane: Vec4::new(0.0, 0.0, -1.0, 0.0),
            left: Vec3::new(0.0, 1.0, 0.0),
        });
    }
}
fn update_spider(
    mut commands: Commands,
    mut spider_query: Query<(&mut Spider, &mut Transform), Without<Ensnared>>,
    insect_query: Query<&FlyingInsect>,
    mut flies: Query<(&FlyingInsect, &Transform), With<Ensnared>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    time: Res<Time>,
    mut web_query: Query<&mut Web>,
    mut is_dead: ResMut<IsDead>,
    spider_plane: Res<WebPlane>,
    rapier_context: Res<RapierContext>,
) {
    let result = spider_query.get_single_mut();

    if !result.is_ok() {
        println!("F U C K");
        return;
    }
    let (mut spider, mut spider_transform) = result.unwrap();

    spider.food -= 0.25 * time.delta_seconds();
    if spider.food <= 0.0 {
        is_dead.is_dead = true;
    }
    let web = &mut *web_query.single_mut();
    /*// tree position debug code
    if let Some(position) = q_windows.single().cursor_position() {
        let (camera, camera_global_transform) = camera_query.single();

        if let Some(ray) = camera.viewport_to_world(&camera_global_transform, position) {
            let n = spider_plane.plane.xyz();
            let d = spider_plane.plane.w;
            let λ = -(n.dot(ray.origin) + d) / (n.dot(*ray.direction));
            let p = ray.origin + ray.direction * λ;
            if 树里有点吗(p, &rapier_context, camera, camera_global_transform) {
                println!("树");
            } else {
                println!("不树");
            }
        }
    }*/
    if buttons.just_pressed(MouseButton::Left) || touches.any_just_pressed() {
        let touch = touches.iter_just_pressed().next();
        let mut touch_pos = None;
        if touch.is_some() {
            touch_pos = Some(touch.unwrap().position())
        }

        if let Some(position) = touch_pos.or(q_windows.single().cursor_position()) {
            let (camera, camera_global_transform) = camera_query.single();

            if let Some(ray) = camera.viewport_to_world(&camera_global_transform, position) {
                let n = spider_plane.plane.xyz();
                let d = spider_plane.plane.w;
                let λ = -(n.dot(ray.origin) + d) / (n.dot(*ray.direction));
                let p = ray.origin + ray.direction * λ;

                set_new_target(
                    p - spider.current_position.to_vec3(web),
                    &mut *spider,
                    web,
                    &rapier_context,
                    camera,
                    camera_global_transform,
                );
            }
        }
    } else if buttons.just_pressed(MouseButton::Right) {
        if let Some(position) = q_windows.single().cursor_position() {
            let (camera, camera_global_transform) = camera_query.single();

            if let Some(ray) = camera.viewport_to_world(&camera_global_transform, position) {
                let n = spider_plane.plane.xyz();
                let d = spider_plane.plane.w;
                let λ = -(n.dot(ray.origin) + d) / (n.dot(*ray.direction));
                let p = ray.origin + ray.direction * λ;

                web.破壊する(p, &insect_query, &mut commands);
            }
        }
    }

    if !照相机里有点吗(spider_transform.translation) {
        spider.current_position = SpiderPosition::TREE(Vec3::new(-2.0, -0.3, 0.0));
        spider.target_position = SpiderPosition::TREE(Vec3::new(-2.0, -0.3, 0.0));
    }

    move_spider(web, &mut *spider, &time);
    rotate_spider(web, &mut *spider, &time);

    if spider.current_position.is_tree() {
        spider.current_roll = 0.0;
    } else {
        let mut has_insect_close = false;
        if let SpiderPosition::WEB(idx, t) = spider.current_position {
            let spring = &web.springs[idx];
            for (fly, trans) in flies.iter() {
                if (trans.translation - spider.current_position.to_vec3(web)).length_squared()
                    < 0.1 * 0.1
                {
                    has_insect_close = true;
                }
            }
        }

        let rem = spider.current_roll % (2.0 * PI);
        if (has_insect_close && (rem.abs() < 0.1 || rem > 2.0 * PI - 0.1))
            || (!has_insect_close && (rem - PI).abs() < 0.1)
        {
            spider.current_roll += PI;
        }
    }

    spider_transform.translation = spider.current_position.to_vec3(web);
    spider_transform.translation.z += 0.05 * spider.lerp_roll.cos();
    spider.lerp_roll = spider.lerp_roll * 0.5 + spider.current_roll * 0.5;

    let spider_plane_up = spider_plane.plane.xyz().cross(spider_plane.left);
    let base_transform_mat = Mat3::from_cols(
        spider_plane.left,
        -spider_plane.plane.xyz(),
        spider_plane_up,
    );
    spider_transform.rotation =
        Quat::from_axis_angle(-spider_plane.plane.xyz(), spider.current_rotation)
            * Quat::from_mat3(&base_transform_mat)
            * Quat::from_axis_angle(Vec3::new(1f32, 0f32, 0f32), spider.lerp_roll);
}

fn handle_ensnared_insect_collision(
    mut commands: Commands,
    mut web_query: Query<&mut Web>,
    mut spider_query: Query<(&mut Spider, Entity)>,
    mut insects_query: Query<&mut FlyingInsect>,
    mut collision_events: EventReader<CollisionEvent>,
    mut ev_feast: EventWriter<SpiderFeastEvent>,
    time: Res<Time>,
) {
    let result = spider_query.get_single_mut();

    if !result.is_ok() {
        println!("F U C K!!! NO FUCKING SPIDERRRRR");
        return;
    }

    let (mut spider, s_entity) = result.unwrap();

    let mut on_touch_insect = |mut commands: &mut Commands,
                               mut insects_query: &mut Query<&mut FlyingInsect>,
                               insect_entity: Entity,
                               mut web_query: &mut Query<&mut Web>,
                               mut s: &mut Spider| {
        let Ok(mut insect) = insects_query.get_mut(insect_entity) else {
            error!("구르기 시작하거나 먹는 곤충이 발견되지 않음");
            return;
        };

        if !s.touching_insects.contains(&insect_entity) {
            log::warn!("start snaring fly {:?}", insect_entity);
            s.touching_insects.push(insect_entity);
            insect.freed_timer.pause();
        }
    };

    let mut on_leave_insect = |mut commands: &mut Commands,
                               mut insects_query: &mut Query<&mut FlyingInsect>,
                               insect_entity: Entity,
                               mut web_query: &mut Query<&mut Web>,
                               mut s: &mut Spider| {
        let Ok(mut insect) = insects_query.get_mut(insect_entity) else {
            error!("구르기 시작하거나 먹는 곤충이 발견되지 않음");
            return;
        };

        if let Some(index) = s
            .touching_insects
            .iter()
            .copied()
            .position(|item| item == insect_entity)
        {
            s.touching_insects.swap_remove(index);

            log::warn!("stop snaring fly {:?}", insect_entity);

            insect.snare_timer.pause();
            if insect.snare_roll_progress < 0.99 {
                insect.snare_roll_progress = 0.0;
                insect.snare_timer.reset();
            }

            insect.freed_timer.unpause();
        };
    };

    let collision_events: Vec<CollisionEvent> = collision_events.read().cloned().collect();

    for collision_event in &collision_events {
        if let CollisionEvent::Started(entity_a, entity_b, _) = collision_event {
            match (
                s_entity == *entity_a,
                s_entity == *entity_b,
                insects_query.get(*entity_a),
                insects_query.get(*entity_b),
            ) {
                (true, false, Err(_), Ok(_)) => {
                    on_touch_insect(
                        &mut commands,
                        &mut insects_query,
                        *entity_b,
                        &mut web_query,
                        spider.as_mut(),
                    );
                }
                (false, true, Ok(_), Err(_)) => {
                    on_touch_insect(
                        &mut commands,
                        &mut insects_query,
                        *entity_a,
                        &mut web_query,
                        spider.as_mut(),
                    );
                }
                _ => {
                    // the collision involved other entity types
                }
            }
        }
        if let CollisionEvent::Stopped(entity_a, entity_b, _) = collision_event {
            match (
                s_entity == *entity_a,
                s_entity == *entity_b,
                insects_query.get(*entity_a),
                insects_query.get(*entity_b),
            ) {
                (true, false, Err(_), Ok(_)) => {
                    on_leave_insect(
                        &mut commands,
                        &mut insects_query,
                        *entity_b,
                        &mut web_query,
                        spider.as_mut(),
                    );
                }
                (false, true, Ok(_), Err(_)) => {
                    on_leave_insect(
                        &mut commands,
                        &mut insects_query,
                        *entity_a,
                        &mut web_query,
                        spider.as_mut(),
                    );
                }
                _ => {
                    // the collision involved other entity types
                }
            }
        }
    }

    let mut to_remove = vec![];

    for touching_insect_entity in &spider.touching_insects {
        if let Err(_) = insects_query.get_mut(*touching_insect_entity) {
            to_remove.push(*touching_insect_entity);
        }
    }

    for entity in to_remove {
        let index_of = spider
            .touching_insects
            .iter()
            .copied()
            .position(|item| item == entity);
        if let Some(index) = index_of {
            spider.touching_insects.swap_remove(index);
        }
    }

    for snaring_insect_entity in spider.touching_insects.iter() {
        let Ok(mut insect) = insects_query.get_mut(*snaring_insect_entity) else {
            error!("구르기 시작하거나 먹는 곤충이 발견되지 않음");
            continue;
        };

        if insect.cooking_progress >= 1.0 {
            // TIME TO EAT!!!!!!
            insect.snare_roll_progress = 0.0; // TODO: why do we need this?
            ev_feast.send(SpiderFeastEvent(1.75));
            commands
                .entity(insect.rolled_ensnare_entity.unwrap())
                .despawn();

            free_enemy_from_web(
                &mut commands,
                *snaring_insect_entity,
                Some(&insect),
                &mut *web_query.single_mut(),
            );
            commands
                .entity(*snaring_insect_entity)
                .remove::<BarSettings<CookingInsect>>();
            commands
                .entity(*snaring_insect_entity)
                .remove::<CookingInsect>();
            commands.entity(*snaring_insect_entity).despawn_recursive();
            println!("DESPAWN!!!!!!");
            continue;
        }

        if insect.snare_roll_progress >= 1.0 {
            continue;
        }

        if insect.snare_timer.paused() {
            insect.snare_timer.unpause();
        }
        insect.snare_timer.tick(time.delta());

        insect.snare_roll_progress +=
            time.delta_seconds() / insect.snare_timer.duration().as_secs_f32();

        if insect.snare_timer.just_finished() {
            insect.snare_timer.reset();
            insect.snare_timer.pause();
            commands
                .entity(*snaring_insect_entity)
                .insert(CookingInsect { progress: 0.0 });

            let mut web = web_query.single_mut();
            for spring in &mut web.springs {
                for ensnared in &mut spring.ensnared_entities {
                    if ensnared.entity == *snaring_insect_entity {
                        ensnared.done_ensnaring = true;
                        break;
                    }
                }
            }
        }
    }
}

fn move_spider(web: &Web, spider: &mut Spider, time: &Res<Time>) {
    if spider.current_position.同(&spider.target_position) {
        return; // spider not moving
    }

    let position = spider.current_position.to_vec3(web);
    let destination = spider.target_position.to_vec3(web);

    let move_dir = (destination - position).normalize();
    spider.current_position = spider
        .current_position
        .加(web, move_dir * time.delta_seconds() * 0.8);

    if (position - destination).length_squared() < 0.01 * 0.01 {
        spider.current_position = spider.target_position;
    }
}

fn rotate_spider(web: &Web, spider: &mut Spider, time: &Res<Time>) {
    let position = spider.current_position.to_vec3(web);
    let destination = spider.target_position.to_vec3(web);
    if (position - destination).length_squared() < 0.01 * 0.01 {
        return;
    }

    let move_dir = (destination - position).normalize();
    let 肉θ = move_dir.y.atan2(move_dir.x);
    let θ = if 肉θ < 0.0 { 肉θ + 2.0 * PI } else { 肉θ };

    let current_angle = if spider.current_rotation + PI / 2.0 > 2.0 * PI {
        spider.current_rotation + PI / 2.0 - 2.0 * PI
    } else {
        spider.current_rotation + PI / 2.0
    };

    if (current_angle - θ).abs() < 0.05 || (current_angle - θ - 2.0 * PI).abs() < 0.05 {
        // move
    } else {
        // rotate
        let angular_velocity = SPIDER_ROTATE_SPEED * PI * time.delta_seconds();
        let 新θ = if (current_angle - θ).abs() < ((current_angle - θ).abs() - 2.0 * PI).abs() {
            let diff_sign = (current_angle - θ).signum();
            let updated_angle = current_angle + angular_velocity * (θ - current_angle).signum();
            let new_diff_sign = (updated_angle - θ).signum();

            if diff_sign != new_diff_sign {
                θ
            } else {
                updated_angle
            }
        } else {
            let diff_sign = (current_angle - θ).signum();
            let updated_angle = current_angle + angular_velocity * -(θ - current_angle).signum();
            let new_diff_sign = (updated_angle - θ).signum();

            if diff_sign != new_diff_sign {
                θ
            } else {
                updated_angle
            }
        };
        spider.current_rotation = if 新θ - PI / 2.0 < 0.0 {
            新θ - PI / 2.0 + 2.0 * PI
        } else {
            新θ - PI / 2.0
        };
    }
}

fn set_new_target(
    target_δ: Vec3,
    spider: &mut Spider,
    web: &mut Web,
    rapier_context: &Res<RapierContext>,
    cam: &Camera,
    cam_transform: &GlobalTransform,
) {
    let position = spider.current_position.to_vec3(web);

    if target_δ.length_squared() < 0.01 {
        return;
    }

    if NNN {
        spider.current_position = SpiderPosition::TREE(position);
        spider.target_position = SpiderPosition::TREE(position + target_δ);
        return;
    }

    let target_dir = target_δ.normalize();
    let mut target_pos = position + target_dir * 10.0;

    let mut dest_spring_idx: Option<usize> = None;
    let mut from_spring: Option<(usize, f32)> = match spider.current_position {
        SpiderPosition::WEB(idx, t) => Some((idx, t)),
        SpiderPosition::TREE(_) => None,
    };

    let mut from_particle_idx: Option<usize> = None;

    if let Some((spring_index, _)) = from_spring {
        let from_spring: &Spring = &web.springs[spring_index];

        let t = match spider.current_position {
            SpiderPosition::WEB(_, t) => t,
            _ => -1.0,
        };

        if t == 0.0
            || web.particles[from_spring.first_index]
                .position
                .distance_squared(spider.current_position.to_vec3(web))
                < 0.03 * 0.03
        {
            from_particle_idx = Some(from_spring.first_index);
        } else if t == 1.0
            || web.particles[from_spring.second_index]
                .position
                .distance_squared(spider.current_position.to_vec3(web))
                < 0.03 * 0.03
        {
            from_particle_idx = Some(from_spring.second_index);
        }
    }

    if from_particle_idx.is_some() {
        for i in 0..web.springs.len() {
            let spring: &Spring = &web.springs[i];

            if spring.first_index == from_particle_idx.unwrap()
                || spring.second_index == from_particle_idx.unwrap()
            {
                let mut dir = web.particles[spring.second_index].position
                    - web.particles[spring.first_index].position;
                let dir_len = dir.length();
                let mut t = 0.0;

                if spring.second_index == from_particle_idx.unwrap() {
                    dir *= -1.0;
                    t = 1.0;
                }

                dir = dir.normalize();
                if dir.dot(target_dir) > 0.98 {
                    let mut dest_t = (target_δ.dot(dir).abs() / dir_len).clamp(0.0, 1.0);
                    if spring.second_index == from_particle_idx.unwrap() {
                        dest_t = 1.0 - dest_t;
                    }

                    spider.current_position = SpiderPosition::WEB(i, t);
                    spider.target_position = SpiderPosition::WEB(i, dest_t);

                    println!("Moving along spring from particle location");
                    return;
                }
            }
        }
    } else if let Some((spring_index, current_t)) = from_spring {
        let spring: &Spring = &web.springs[spring_index];
        let mut dir = web.particles[spring.second_index].position
            - web.particles[spring.first_index].position;
        let dir_len = dir.length();

        dir = dir.normalize();
        if dir.dot(target_dir) > 0.98 {
            let delta_t = (target_δ.dot(dir).abs() / dir_len);
            spider.target_position =
                SpiderPosition::WEB(spring_index, (current_t + delta_t).clamp(0.0, 1.0));
            println!("Moving along spring from middle of spring");
            return;
        }

        if dir.dot(target_dir) < -0.98 {
            let delta_t = (target_δ.dot(dir).abs() / dir_len);
            spider.target_position =
                SpiderPosition::WEB(spring_index, (current_t - delta_t).clamp(0.0, 1.0));
            println!("Moving along spring from middle of spring");
            return;
        }
    }

    for i in 0..web.springs.len() {
        if from_spring.is_some() && from_spring.unwrap().0 == i {
            continue;
        }

        let spring = &web.springs[i];
        let result = spring.intersects(
            web,
            Vec3::new(0.0, 0.0, -1.0),
            position - target_dir,
            position + target_dir * 10.0,
        );

        if result.is_none() {
            continue;
        }

        let new_pos = result.unwrap();

        if target_dir.dot(new_pos) - target_dir.dot(position) <= 0.05 {
            continue;
        }

        if target_dir.dot(new_pos) >= target_dir.dot(target_pos) {
            continue;
        }

        target_pos = new_pos;
        dest_spring_idx = Some(i);
    }

    let existing_p1 = web.get_particle_index(position, 0.03);
    let existing_p2 = web.get_particle_index(target_pos, 0.03);

    if existing_p1 == existing_p2 && existing_p1.is_some() {
        println!("Not initiating to move far enough to initiate movement");

        if dest_spring_idx.is_some() {
            let spring = &web.springs[dest_spring_idx.unwrap()];
            spider.target_position = SpiderPosition::WEB(
                dest_spring_idx.unwrap(),
                if spring.first_index == existing_p2.unwrap() {
                    0.0
                } else {
                    1.0
                },
            );
        }
        return; // not initiating to move far enough to initiate movement
    }

    // no destination found, set target_pos from nearest tree point
    if dest_spring_idx.is_none() {
        target_pos = position + target_δ;

        let mut i = 0;
        while !树里有点吗(target_pos, rapier_context, cam, cam_transform) && i < 10 {
            target_pos += target_dir * 0.1;
            i += 1;
        }

        if !树里有点吗(target_pos, rapier_context, cam, cam_transform) {
            // 这个向没有树
            println!("Clicked in direction with nothing in front, doing nothing");
            return;
        }

        if 树里有小路吗(position, target_pos, rapier_context, cam, cam_transform) {
            println!("Tree to Tree movement no silk");
            spider.current_position = SpiderPosition::TREE(position);
            spider.target_position = SpiderPosition::TREE(position + target_δ);
            return;
        }
    } else if 树里有小路吗(position, target_pos, rapier_context, cam, cam_transform) {
        println!("Tree to Tree movement no silk");
        spider.current_position = SpiderPosition::TREE(position);
        spider.target_position = SpiderPosition::TREE(position + target_δ);
        return;
    }

    if existing_p1.is_some() && existing_p2.is_some() {
        let spring_idx = web.get_spring(existing_p1.unwrap(), existing_p2.unwrap());

        if spring_idx.is_some() {
            let spring = &web.springs[spring_idx.unwrap()];
            let target = position + target_δ;

            let spring_p1 = web.particles[spring.first_index].position;
            let spring_p2 = web.particles[spring.second_index].position;

            // find t from target
            let mut t =
                (target.length() - spring_p1.length()) / (spring_p2.length() - spring_p1.length());

            if spring.first_index == existing_p1.unwrap() {
                t = 1.0 - t;
            }

            let t_start = if spring.first_index == existing_p1.unwrap() {
                0.0
            } else {
                1.0
            };

            spider.current_position = SpiderPosition::WEB(spring_idx.unwrap(), t_start);
            spider.target_position = SpiderPosition::WEB(spring_idx.unwrap(), t);
            println!("Path is along existing spring");
            return;
        }
    }

    let hack_spring_count = web.springs.len();
    let mut hack_swap_removed_a_spring = false;

    let p1 = if existing_p1.is_none() {
        if let Some((from_spring_index, _)) = from_spring {
            web.split_spring(from_spring_index, position);
            hack_swap_removed_a_spring = true;
        } else {
            let in_tree = 树里有点吗(position, rapier_context, cam, cam_transform);
            //if !in_tree {
            //    println!("[FUCK] Trying to create new spring start point but NOT IN TREE");
            //    return;
            //}

            web.particles.push(Particle {
                position: position,
                velocity: Default::default(),
                force: Default::default(),
                impulse: Default::default(),
                impulse_duration: 0.0,
                mass: 0.0,
                pinned: in_tree,
            });
        }
        web.particles.len() - 1
    } else {
        existing_p1.unwrap()
    };

    let p2 = if existing_p2.is_none() {
        if dest_spring_idx.is_none() {
            let in_tree = 树里有点吗(target_pos, rapier_context, cam, cam_transform);
            if !in_tree {
                println!("[FUCK] Trying to create new spring end point but target NOT IN TREE");
                return;
            }

            web.particles.push(Particle {
                position: target_pos,
                velocity: Default::default(),
                force: Default::default(),
                impulse: Default::default(),
                impulse_duration: 0.0,
                mass: 0.0,
                pinned: true,
            });
        } else {
            // 可怕的黑客
            // 基本上这意味着 dest_spring_idx 在上述对 split_spring 的调用中由于其内部发生的 swap_remove
            // 调用而失效，因此我们检测到此副作用并更正索引
            if hack_swap_removed_a_spring && dest_spring_idx.unwrap() == hack_spring_count - 1 {
                dest_spring_idx = Some(from_spring.unwrap().0);
            }

            web.split_spring(dest_spring_idx.unwrap(), target_pos);
        }
        web.particles.len() - 1
    } else {
        existing_p2.unwrap()
    };

    web.springs.push(Spring::new_with_length(
        web,
        p1,
        p2,
        20.0,
        0.5,
        (web.particles[p1].position - web.particles[p2].position).length() * 0.75,
        vec![],
    ));

    spider.current_position = SpiderPosition::WEB(web.springs.len() - 1, 0.0);
    spider.target_position = SpiderPosition::WEB(web.springs.len() - 1, 1.0);
    println!("New path created");
}

fn spawn_spider(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut _camera_transform_query: Query<(&mut Transform, &Camera)>,
    spider_plane: Res<WebPlane>,
) {
    let start_pos = Vec3::new(-2.0, -0.3, 0.0);
    let spider_plane_up = spider_plane.plane.xyz().cross(spider_plane.left);
    let base_transform_mat = bevy::math::mat3(
        spider_plane.left,
        -spider_plane.plane.xyz(),
        spider_plane_up,
    );
    commands
        .spawn((
            Spider::new(10.0, start_pos),
            SceneBundle {
                scene: asset_server.load("spider.glb#Scene0"),
                transform: Transform {
                    translation: start_pos,
                    rotation: Quat::from_mat3(&base_transform_mat),
                    scale: Vec3::new(0.1, 0.1, 0.1),
                },
                global_transform: Default::default(),
                visibility: Default::default(),
                inherited_visibility: Default::default(),
                view_visibility: Default::default(),
            },
            Collider::capsule_y(1.0, 1.0),
        ))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
        .insert(CollisionGroups {
            memberships: COLLISION_GROUP_PLAYER,
            filters: Group::ALL,
        });
}
