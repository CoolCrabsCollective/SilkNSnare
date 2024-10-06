use crate::tree::{树里有点吗, 树里的开始, 树里的结尾};
use crate::web::spring::Spring;
use crate::web::{Particle, Web};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::na::ComplexField;
use std::f32::consts::PI;

pub const NNN: bool = false; // currently october, set this to true in november
pub const SPIDER_ROTATE_SPEED: f32 = 5.6;

pub struct SpiderPlugin;

#[derive(Resource)]
struct WebPlane {
    plane: Vec4, // ax + by + cz + d = 0
    left: Vec3,
}

#[derive(Component)]
struct Spider {
    food: f64,
    max_food: f64,
    current_position: SpiderPosition,
    current_rotation: f32,
    target_position: SpiderPosition,
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
        }
    }
}

impl Plugin for SpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spider);
        app.add_systems(Update, update_spider);
        app.insert_resource(WebPlane {
            plane: Vec4::new(0.0, 0.0, -1.0, 0.0),
            left: Vec3::new(0.0, 1.0, 0.0),
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
) {
    let result = spider_query.get_single_mut();

    if !result.is_ok() {
        println!("F U C K");
        return;
    }

    let (mut spider, mut spider_transform) = result.unwrap();
    let web = &mut *web_query.single_mut();

    if buttons.just_pressed(MouseButton::Left)
        && spider.current_position.同(&spider.target_position)
    {
        if let Some(position) = q_windows.single().cursor_position() {
            let (camera, camera_global_transform) = camera_query.single();

            if let Some(ray) = camera.viewport_to_world(&camera_global_transform, position) {
                let n = spider_plane.plane.xyz();
                let d = spider_plane.plane.w;
                let λ = -(n.dot(ray.origin) + d) / (n.dot(*ray.direction));
                let p = ray.origin + ray.direction * λ;

                set_new_target(p - spider.current_position.to_vec3(web), &mut *spider, web);
            }
        } else {
            println!("Cursor is not in the game window.");
        }
    }

    move_spider(web, &mut *spider, &time);
    rotate_spider(web, &mut *spider, &time);
    spider_transform.translation = spider.current_position.to_vec3(web);

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

fn set_new_target(target_δ: Vec3, spider: &mut Spider, web: &mut Web) {
    let position = spider.current_position.to_vec3(web);

    if target_δ.length_squared() < 0.01 {
        return;
    }

    if NNN {
        spider.current_position = SpiderPosition::TREE(position);
        spider.target_position = SpiderPosition::TREE(position + target_δ);
        return;
    }

    let mut dest_spring_idx: Option<usize> = None;
    let mut from_spring_idx: Option<usize> = match spider.current_position {
        SpiderPosition::WEB(idx, _) => Some(idx),
        SpiderPosition::TREE(_) => None,
    };

    let target_dir = target_δ.normalize();
    let mut target_pos = position + target_dir * 10.0;

    for i in 0..web.springs.len() {
        if from_spring_idx.is_some() && from_spring_idx.unwrap() == i {
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
        let result = 树里的开始(position - target_dir * 0.1, target_dir);

        if result.is_none() {
            println!("Tree to Tree movement no silk");
            spider.current_position = SpiderPosition::TREE(position);
            spider.target_position = SpiderPosition::TREE(position + target_δ);
            return;
        }

        target_pos = result.unwrap() + target_dir * 0.1;

        if 树里有点吗(position) || 树里有点吗(position - target_dir * 0.1) {
            let 结尾 = 树里的结尾(position - target_dir * 0.1, target_dir);

            if 结尾.is_none() {
                println!("Tree to Tree movement no silk");
                spider.current_position = SpiderPosition::TREE(position);
                spider.target_position = SpiderPosition::TREE(target_pos);
                return;
            }
        }
    }

    if from_spring_idx.is_some() {
        let from_spring: &Spring = &web.springs[from_spring_idx.unwrap()];

        if existing_p2.is_some()
            && (from_spring.first_index == existing_p2.unwrap()
                || from_spring.first_index == existing_p1.unwrap())
        {
            // move within the existing spring
            // find t from target
        }
    }

    if existing_p1.is_some() && existing_p2.is_some() {
        let spring_idx = web.get_spring(existing_p1.unwrap(), existing_p2.unwrap());

        if spring_idx.is_some() {
            let spring = &web.springs[spring_idx.unwrap()];
            let target = position + target_δ;

            // find t from target

            let t = if spring.first_index == existing_p1.unwrap() {
                1.0
            } else {
                0.0
            };
            spider.current_position = SpiderPosition::WEB(spring_idx.unwrap(), 1.0 - t);
            spider.target_position = SpiderPosition::WEB(spring_idx.unwrap(), t);
            println!("Path is along existing spring");
            return;
        }
    }

    let p1 = if existing_p1.is_none() {
        if from_spring_idx.is_none() {
            web.particles.push(Particle {
                position: position,
                velocity: Default::default(),
                force: Default::default(),
                mass: 0.0,
                pinned: true,
            });
        } else {
            web.split_spring(from_spring_idx.unwrap(), position);
        }
        web.particles.len() - 1
    } else {
        existing_p1.unwrap()
    };

    let p2 = if existing_p2.is_none() {
        if dest_spring_idx.is_none() {
            web.particles.push(Particle {
                position: target_pos,
                velocity: Default::default(),
                force: Default::default(),
                mass: 0.0,
                pinned: true,
            });
        } else {
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
    commands.spawn((
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
    ));
}
