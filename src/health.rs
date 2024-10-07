use crate::spider::{Spider, SpiderFeastEvent};
use crate::tree::{
    get_death_target_position, get_death_target_rotation, get_target_camera_direction,
    get_target_camera_position,
};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{
    Camera, Component, EventReader, Query, Res, ResMut, Resource, Style, Time, Without,
};
use bevy::ui::Val;

pub struct HealthPlugin;
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_health);
        app.add_systems(Update, update_death_screen);
        app.add_systems(Update, feast_on_bugs);
        app.add_event::<SpiderFeastEvent>();
        app.insert_resource(IsDead {
            is_dead: false,
            death_camera_progress: 0.0,
        });
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Resource)]
pub struct IsDead {
    pub is_dead: bool,
    pub death_camera_progress: f32,
}

fn update_health(
    mut spider_query: Query<(&mut Spider), Without<HealthBar>>,
    mut health_query: Query<(&HealthBar, &mut Style)>,
) {
    let spider = spider_query.single_mut();
    let (_, mut style) = health_query.single_mut();

    style.width = Val::Percent((100.0 * (spider.food / spider.max_food)) as f32);
}

fn update_death_screen(
    mut is_dead: ResMut<IsDead>,
    mut camera_transform_query: Query<(&mut bevy::prelude::Transform, &Camera)>,
    time: Res<Time>,
) {
    if is_dead.is_dead {
        println!("{:?}: ", is_dead.death_camera_progress);
        if is_dead.death_camera_progress < 1.0 {
            let s = is_dead.death_camera_progress;
            let t = 3.0 * s * s - 2.0 * s * s * s;
            if let Ok((mut camera_transform, _)) = camera_transform_query.get_single_mut() {
                camera_transform.translation =
                    ((1.0 - t) * get_target_camera_position()) + t * get_death_target_position();
                camera_transform.rotation =
                    get_target_camera_direction().lerp(get_death_target_rotation(), t);

                is_dead.death_camera_progress += 0.5 * time.delta_seconds();
            }
        }
    }
}

fn feast_on_bugs(
    mut ev_feast: EventReader<SpiderFeastEvent>,
    mut spider_query: Query<(&mut Spider)>,
) {
    let mut spider = spider_query.single_mut();
    for ev in ev_feast.read() {
        let food = ev.0;
        println!("Feast!!");
        spider.food = spider.max_food.min(spider.food + food);
    }
}
