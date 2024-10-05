use crate::web::spring::Spring;
use crate::web::{Particle, Web};
use bevy::math::NormedVectorSpace;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::na::ComplexField;
use std::f32::consts::PI;

pub const NNN: bool = false; // currently october, set this to true in november

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
    target_position: Vec3,
    current_rotation: f32,
}

impl Spider {
    pub fn new(max_food: f64, target_position: Vec3) -> Self {
        Spider {
            food: max_food,
            max_food,
            target_position,
            current_rotation: 0.0,
        }
    }
}

impl Plugin for SpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spider);
        app.add_systems(Update, move_spider);
        app.insert_resource(WebPlane {
            plane: Vec4::new(0.0, 0.0, -1.0, 0.25),
            left: Vec3::new(0.0, 1.0, 0.0),
        });
    }
}
fn move_spider(
    mut spider_query: Query<(&mut Spider, &mut Transform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut web_query: Query<&mut Web>,
    spider_plane: Res<WebPlane>,
) {
    if let Ok((mut spider, mut spider_transform)) = spider_query.get_single_mut() {
        if buttons.just_pressed(MouseButton::Left) {
            if let Some(position) = q_windows.single().cursor_position() {
                let (camera, camera_global_transform) = camera_query.single();

                if let Some(ray) = camera.viewport_to_world(&camera_global_transform, position) {
                    let n = spider_plane.plane.xyz();
                    let d = spider_plane.plane.w;
                    let λ = -(n.dot(ray.origin) + d) / (n.dot(*ray.direction));
                    let p = ray.origin + ray.direction * λ;

                    spider.target_position = p;
                    set_new_target(
                        p,
                        &mut *spider,
                        &mut *spider_transform,
                        &mut *web_query.single_mut(),
                    );
                }
            } else {
                println!("Cursor is not in the game window.");
            }
        }

        let web = web_query.single();

        for spring in &web.springs {
            let result = spring.intersects(
                web,
                Vec3::new(0.0, 0.0, -1.0),
                spider_transform.translation,
                spider.target_position,
            );
            if result.is_none() {
                continue;
            }

            let new_pos = result.unwrap();
            if new_pos.distance_squared(spider_transform.translation) < 0.1 * 0.1 {
                continue;
            }

            spider.target_position = new_pos;
        }

        if (spider_transform.translation - spider.target_position).norm() < 1e-2 {
            spider_transform.translation = spider.target_position;
        } else {
            let move_dir = (spider.target_position - spider_transform.translation).normalize();
            let raw_angle = move_dir.y.atan2(move_dir.x);
            let angle = if raw_angle < 0.0 {
                raw_angle + 2.0 * PI
            } else {
                raw_angle
            };

            let current_angle = if spider.current_rotation + PI / 2.0 > 2.0 * PI {
                spider.current_rotation + PI / 2.0 - 2.0 * PI
            } else {
                spider.current_rotation + PI / 2.0
            };

            let spider_plane_up = spider_plane.plane.xyz().cross(spider_plane.left);
            let base_transform_mat = Mat3::from_cols(
                spider_plane.left,
                -spider_plane.plane.xyz(),
                spider_plane_up,
            );

            if (current_angle - angle).abs() < 0.05
                || (current_angle - angle - 2.0 * PI).abs() < 0.05
            {
                // move
                spider_transform.translation =
                    spider_transform.translation + move_dir * time.delta_seconds() * 0.8;
            } else {
                // rotate
                let angular_velocity = 2.8 * PI * time.delta_seconds();
                let new_angle = if (current_angle - angle).abs()
                    < ((current_angle - angle).abs() - 2.0 * PI).abs()
                {
                    let diff_sign = (current_angle - angle).signum();
                    let updated_angle =
                        current_angle + angular_velocity * (angle - current_angle).signum();
                    let new_diff_sign = (updated_angle - angle).signum();

                    if diff_sign != new_diff_sign {
                        angle
                    } else {
                        updated_angle
                    }
                } else {
                    let diff_sign = (current_angle - angle).signum();
                    let updated_angle =
                        current_angle + angular_velocity * -(angle - current_angle).signum();
                    let new_diff_sign = (updated_angle - angle).signum();

                    if diff_sign != new_diff_sign {
                        angle
                    } else {
                        updated_angle
                    }
                };
                spider.current_rotation = if new_angle - PI / 2.0 < 0.0 {
                    new_angle - PI / 2.0 + 2.0 * PI
                } else {
                    new_angle - PI / 2.0
                };

                spider_transform.rotation =
                    Quat::from_axis_angle(-spider_plane.plane.xyz(), spider.current_rotation)
                        * Quat::from_mat3(&base_transform_mat);
            }
        }
    }
}

fn set_new_target(p: Vec3, spider: &mut Spider, spider_transform: &Transform, web: &mut Web) {
    spider.target_position = p;

    if NNN {
        return;
    }

    let mut spring_idx: Option<usize> = None;

    for i in 0..web.springs.len() {
        let spring = &web.springs[i];
        let result = spring.intersects(
            web,
            Vec3::new(0.0, 0.0, -1.0),
            spider_transform.translation,
            spider.target_position,
        );
        if result.is_none() {
            continue;
        }

        let new_pos = result.unwrap();
        if new_pos.distance_squared(spider_transform.translation) < 0.1 * 0.1 {
            continue;
        }

        spider.target_position = new_pos;
        spring_idx = Some(i);
    }

    if spring_idx.is_some() {
        let existing_p1 = web.get_particle_index(spider_transform.translation, 0.2);
        let existing_p2 = web.get_particle_index(spider.target_position, 0.2);

        let p1 = if existing_p1.is_none() {
            web.particles.push(Particle {
                position: spider_transform.translation,
                velocity: Default::default(),
                force: Default::default(),
                mass: 0.0,
                pinned: true,
            });
            web.particles.len() - 1
        } else {
            existing_p1.unwrap()
        };

        let p2 = if existing_p2.is_none() {
            web.particles.push(Particle {
                position: spider.target_position,
                velocity: Default::default(),
                force: Default::default(),
                mass: 0.0,
                pinned: false,
            });
            let spring: Spring = web.springs.swap_remove(spring_idx.unwrap());
            web.springs.push(Spring::new(
                web,
                web.particles.len() - 1,
                spring.first_index,
                20.0,
                0.5,
            ));
            web.springs.push(Spring::new(
                web,
                web.particles.len() - 1,
                spring.second_index,
                20.0,
                0.5,
            ));

            web.particles.len() - 1
        } else {
            existing_p2.unwrap()
        };

        web.springs.push(Spring::new(web, p1, p2, 20.0, 0.5));
    }
}

fn spawn_spider(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut _camera_transform_query: Query<(&mut Transform, &Camera)>,
    spider_plane: Res<WebPlane>,
) {
    let start_pos = Vec3::new(-2.0, 0.0, 0.0);
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
