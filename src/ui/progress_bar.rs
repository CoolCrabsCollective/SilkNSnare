use crate::flying_insect::flying_insect::FlyingInsect;
use bevy::app::{App, Plugin};
use bevy_health_bar3d::configuration::Percentage;
use bevy_health_bar3d::prelude::HealthBarPlugin;

pub struct ProgressBarPlugin;

impl Percentage for FlyingInsect {
    fn value(&self) -> f32 {
        self.cooking_timer.elapsed_secs() / self.cooking_timer.duration().as_secs() as f32
    }
}

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FlyingInsect>()
            .add_plugins(HealthBarPlugin::<FlyingInsect>::default())
            .run();
    }
}
