use crate::spider::Spider;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Component, Query, Style, Without};
use bevy::ui::Val;

pub struct HealthPlugin;
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_health);
    }
}

#[derive(Component)]
pub struct HealthBar;

fn update_health(
    mut spider_query: Query<(&mut Spider), Without<HealthBar>>,
    mut health_query: Query<(&HealthBar, &mut Style)>,
) {
    let spider = spider_query.single_mut();
    let (_, mut style) = health_query.single_mut();

    style.width = Val::Percent((100.0 * (spider.food / spider.max_food)) as f32);
}
