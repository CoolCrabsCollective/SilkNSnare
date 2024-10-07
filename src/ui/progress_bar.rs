use crate::flying_insect::flying_insect::FlyingInsect;
use bevy::app::{App, Plugin};
use bevy::prelude::{Color, Component, Reflect};
use bevy_health_bar3d::configuration::Percentage;
use bevy_health_bar3d::prelude::{ColorScheme, ForegroundColor, HealthBarPlugin};

pub struct ProgressBarPlugin;

#[derive(Component, Reflect)]
pub struct CookingInsect {
    progress: f32,
}

impl Percentage for CookingInsect {
    fn value(&self) -> f32 {
        self.progress
    }
}

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CookingInsect>()
            .add_plugins(HealthBarPlugin::<CookingInsect>::default())
            .insert_resource(ColorScheme::<CookingInsect>::new().foreground_color(
                ForegroundColor::Static(Color::srgba(
                    245.0 / 255.0,
                    144.0 / 255.0,
                    66.0 / 255.0,
                    1.0,
                )),
            ));
    }
}
