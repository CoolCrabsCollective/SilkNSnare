use crate::config::{COLLISION_GROUP_ALL, COLLISION_GROUP_TERRAIN};
use crate::flying_insect::flying_insect::FlyingInsect;
use crate::tree::{树里有小路吗, 树里有点吗};
use crate::web::ensnare::{free_enemy_from_web, Ensnared};
use crate::web::spring::Spring;
use crate::web::{Particle, Web};
use bevy::ecs::observer::TriggerTargets;
use bevy::ecs::query::QueryEntityError;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::na::ComplexField;
use bevy_rapier3d::pipeline::CollisionEvent;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, QueryFilter,
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

#[derive(Resource)]
pub struct SnareTimer {
    pub timer: Timer,
}

#[derive(Component)]
struct Spider {
    food: f64,
    max_food: f64,
    current_position: SpiderPosition,
    current_rotation: f32,
    target_position: SpiderPosition,
    snaring_insect: Option<Entity>,
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
    pub fn new(max_food: f64, pos: Vec3) -> Self {
        Spider {
            food: max_food,
            max_food,
            target_position: SpiderPosition::TREE(pos),
            current_position: SpiderPosition::TREE(pos),
            current_rotation: 0.0,
            snaring_insect: None,
        }
    }
}

impl Plugin for SpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spider);
        app.add_systems(Update, update_spider);
        app.add_systems(Update, handle_ensnared_insect_collision);
        app.insert_resource(WebPlane {
            plane: Vec4::new(0.0, 0.0, -1.0, 0.0),
            left: Vec3::new(0.0, 1.0, 0.0),
        });
        app.insert_resource(SnareTimer {
            timer: Timer::new(Duration::from_millis(2000), TimerMode::Repeating),
        });
    }
}
fn update_spider(
    mut spider_query: Query<(&mut Spider, &mut Transform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut web_query: Query<&mut Web>,
    spider_plane: Res<WebPlane>,
    rapier_context: Res<RapierContext>,
) {
    let result = spider_query.get_single_mut();

    if !result.is_ok() {
        println!("F U C K");
        return;
    }

    let (mut spider, mut spider_transform) = result.unwrap();
    let web = &mut *web_query.single_mut();

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = q_windows.single().cursor_position() {
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
    }

    move_spider(web, &mut *spider, &time);
    rotate_spider(web, &mut *spider, &time);
    spider_transform.translation = spider.current_position.to_vec3(web);
    spider_transform.translation.z += 0.05;

    let spider_plane_up = spider_plane.plane.xyz().cross(spider_plane.left);
    let base_transform_mat = Mat3::from_cols(
        spider_plane.left,
        -spider_plane.plane.xyz(),
        spider_plane_up,
    );
    spider_transform.rotation =
        Quat::from_axis_angle(-spider_plane.plane.xyz(), spider.current_rotation)
            * Quat::from_mat3(&base_transform_mat);
}

fn handle_ensnared_insect_collision(
    mut commands: Commands,
    mut web_query: Query<&mut Web>,
    mut spider_query: Query<(&mut Spider, Entity)>,
    mut insects_query: Query<(&mut FlyingInsect), With<Ensnared>>,
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,
    mut ss_snare_timer: ResMut<SnareTimer>,
) {
    let result = spider_query.get_single_mut();

    if !result.is_ok() {
        println!("F U C K!!! NO FUCKING SPIDERRRRR");
        return;
    }

    let (mut spider, s_entity) = result.unwrap();

    let mut roll_or_eat_insect =
        |mut commands: &mut Commands,
         mut insects_query: &mut Query<(&mut FlyingInsect), With<Ensnared>>,
         entity: Entity,
         mut web_query: &mut Query<&mut Web>,
         mut s: &mut Spider| {
            let Ok(mut insect) = insects_query.get_mut(entity) else {
                error!("구르기 시작하거나 먹는 곤충이 발견되지 않음");
                return;
            };
            if insect.ensnared_and_rolled & insect.cooked {
                // TIME TO EAT!!!!!!
                insect.ensnared_and_rolled = false;
                commands
                    .entity(insect.rolled_ensnare_entity.unwrap())
                    .despawn();
                free_enemy_from_web(commands, entity, web_query);
                commands.entity(entity).despawn();
            } else if !insect.ensnared_and_rolled {
                s.snaring_insect = Some(entity); // only start rolling
                insect.freed_timer.pause();
            }
        };

    if spider.snaring_insect == None {
        for collision_event in collision_events.read() {
            if let CollisionEvent::Started(entity_a, entity_b, _) = collision_event {
                match (
                    s_entity == *entity_a,
                    s_entity == *entity_b,
                    insects_query.get(*entity_a),
                    insects_query.get(*entity_b),
                ) {
                    (true, false, Ok(mut insect), Err(_)) => {
                        roll_or_eat_insect(
                            &mut commands,
                            &mut insects_query,
                            *entity_a,
                            &mut web_query,
                            spider.as_mut(),
                        );
                    }
                    (true, false, Err(_), Ok(insect)) => {
                        roll_or_eat_insect(
                            &mut commands,
                            &mut insects_query,
                            *entity_b,
                            &mut web_query,
                            spider.as_mut(),
                        );
                    }
                    (false, true, Ok(insect), Err(_)) => {
                        roll_or_eat_insect(
                            &mut commands,
                            &mut insects_query,
                            *entity_a,
                            &mut web_query,
                            spider.as_mut(),
                        );
                    }
                    (false, true, Err(_), Ok(insect)) => {
                        roll_or_eat_insect(
                            &mut commands,
                            &mut insects_query,
                            *entity_b,
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
    } else {
        let mut still_snaring = true;
        for collision_event in collision_events.read() {
            if let CollisionEvent::Stopped(entity_a, entity_b, _) = collision_event {
                match (
                    s_entity == *entity_a,
                    s_entity == *entity_b,
                    spider.snaring_insect.unwrap() == *entity_a,
                    spider.snaring_insect.unwrap() == *entity_b,
                ) {
                    (true, false, true, false) => {
                        still_snaring = false;
                    }
                    (true, false, false, true) => {
                        still_snaring = false;
                    }
                    (false, true, true, false) => {
                        still_snaring = false;
                    }
                    (false, true, false, true) => {
                        still_snaring = false;
                    }
                    _ => {
                        // the collision involved other entity types
                    }
                }

                if !still_snaring {
                    break;
                }
            }
        }

        if !still_snaring {
            let Ok(mut insect) = insects_query.get_mut(spider.snaring_insect.unwrap()) else {
                error!("구르기 시작하거나 먹는 곤충이 발견되지 않음");
                return;
            };
            insect.freed_timer.unpause();
            spider.snaring_insect = None;
            ss_snare_timer.timer.reset();
            ss_snare_timer.timer.pause();
            return;
        }

        if ss_snare_timer.timer.paused() {
            ss_snare_timer.timer.unpause()
        }
        ss_snare_timer.timer.tick(time.delta());

        if ss_snare_timer.timer.just_finished() {
            ss_snare_timer.timer.reset();
            ss_snare_timer.timer.pause();

            // Mark insect as rolled, wait on timeout before allowing to eat
            if let Ok(mut insect) = insects_query.get_mut(spider.snaring_insect.unwrap()) {
                insect.ensnared_and_rolled = true;
                spider.snaring_insect = None;
            };
        }
        if spider.snaring_insect != None {
            match insects_query.get_mut(spider.snaring_insect.unwrap()) {
                Ok(_) => {}
                Err(_) => spider.snaring_insect = None,
            };
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

        if t == 0.0 {
            from_particle_idx = Some(from_spring.first_index);
        } else if t == 1.0 {
            from_particle_idx = Some(from_spring.second_index);
        }
    }

    if let Some((spring_index, current_t)) = from_spring {
        let spring: &Spring = &web.springs[spring_index];
        let mut dir = web.particles[spring.second_index].position
            - web.particles[spring.first_index].position;
        let dir_len = dir.length();

        dir = dir.normalize();
        if dir.dot(target_dir) > 0.98 {
            let delta_t = (target_δ.dot(dir).abs() / dir_len);
            spider.target_position =
                SpiderPosition::WEB(spring_index, (current_t + delta_t).clamp(0.0, 1.0));
            println!("Moving along spring from particle location");
            return;
        }

        if dir.dot(target_dir) < -0.98 {
            let delta_t = (target_δ.dot(dir).abs() / dir_len);
            spider.target_position =
                SpiderPosition::WEB(spring_index, (current_t - delta_t).clamp(0.0, 1.0));
            println!("Moving along spring from particle location");
            return;
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
                    let delta_t = (target_δ.dot(dir).abs() / dir_len).clamp(0.0, 1.0);
                    spider.current_position = SpiderPosition::WEB(i, t);
                    spider.target_position = SpiderPosition::WEB(i, 1.0 - delta_t);

                    println!("Moving along spring from particle location");
                    return;
                }
            }
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

    let existing_p1 = web.get_particle_index(position, 0.1);
    let existing_p2 = web.get_particle_index(target_pos, 0.1);

    if existing_p1 == existing_p2 && existing_p1.is_some() {
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

            spider.current_position = SpiderPosition::WEB(spring_idx.unwrap(), 1.0 - t);
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
            if !in_tree {
                println!("[FUCK] Trying to create new spring start point but NOT IN TREE");
                return;
            }

            web.particles.push(Particle {
                position: position,
                velocity: Default::default(),
                force: Default::default(),
                impulse: Default::default(),
                impulse_duration: 0.0,
                mass: 0.0,
                pinned: true,
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
            // HORRIBLE HACK
            // basically this means that dest_spring_idx got invalidated in the
            // above call to split_spring due to a swap_remove call that happens inside it
            // so we detect this side effect and correct the index
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
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC);
}
