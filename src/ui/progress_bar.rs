use crate::flying_insect::flying_insect::FlyingInsect;
use crate::game::GameState;
use bevy::app::{App, Plugin};
use bevy::prelude::{in_state, Color, Component, IntoSystemConfigs, Query, Reflect, Update};
use bevy_health_bar3d::configuration::Percentage;
use bevy_health_bar3d::prelude::{ColorScheme, ForegroundColor, HealthBarPlugin};

pub struct ProgressBarPlugin;

#[derive(Component, Reflect)]
pub struct CookingInsect {
    pub(crate) progress: f32,
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
                ForegroundColor::TriSpectrum {
                    high: Color::srgba(81.0 / 255.0, 245.0 / 255.0, 66.0 / 255.0, 1.0),
                    moderate: Color::srgba(245.0 / 255.0, 144.0 / 255.0, 66.0 / 255.0, 1.0),
                    low: Color::srgba(245.0 / 255.0, 144.0 / 255.0, 66.0 / 255.0, 1.0),
                },
            ));
        app.add_systems(
            Update,
            update_cooking_insects.run_if(in_state(GameState::Game)),
        );
    }
}

pub fn update_cooking_insects(mut cooking: Query<(&mut CookingInsect, &FlyingInsect)>) {
    for (mut cook, fly) in cooking.iter_mut() {
        cook.progress = fly.cooking_progress;
    }
}
