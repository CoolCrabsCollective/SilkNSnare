use crate::web::Web;
use bevy::{prelude::*, window::PrimaryWindow};

pub struct SpiderPlugin;

#[derive(Resource)]
struct WebPlane {
    plane: Vec4, // ax + by + cz + d = 0
}

#[derive(Component)]
struct Spider {
    food: f64,
    max_food: f64,
    target_position: Vec3,
}

impl Spider {
    pub fn new(max_food: f64, target_position: Vec3) -> Self {
        Spider {
            food: max_food,
            max_food,
            target_position,
        }
    }
}

impl Plugin for SpiderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spider);
        app.add_systems(Update, move_spider);
        app.insert_resource(WebPlane { plane: Vec4::new(0.0, 0.0, -1.0, 0.25) });
    }
}
fn move_spider(
    mut spider_query: Query<(&mut Spider, &mut Transform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    _time: Res<Time>,
    web_query: Query<&mut Web>,
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

                    let new_direction = spider.target_position - spider_transform.translation;
                    // assumes 0,0,-1 plane
                    let angle = new_direction.y.atan2(new_direction.x);


                 }
            } else {
                println!("Cursor is not in the game window.");
            }
        }
        //spider_transform.rotation = Quat::from_axis_angle(spider_plane.plane.xyz(), 90.0);


        let web = web_query.single();

        for spring in &web.springs {
            if spring.intersects(spider_transform.translation, spider.target_position) {

            }
        }

        spider_transform.translation = spider.target_position;
    }
}

fn spawn_spider(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut _camera_transform_query: Query<(&mut Transform, &Camera)>,
) {
    let start_pos = Vec3::new(-2.0, 0.0, 0.0);
    commands.spawn((Spider::new(10.0, start_pos), SceneBundle {
        scene: asset_server.load("spider.glb#Scene0"),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
            scale: Vec3::new(0.25, 0.25, 0.25),
        },
        global_transform: Default::default(),
        visibility: Default::default(),
        inherited_visibility: Default::default(),
        view_visibility: Default::default(),
    }));
}
