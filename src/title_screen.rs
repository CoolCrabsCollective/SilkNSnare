use crate::game::GameState;
use crate::tree::GameStart;
use bevy::prelude::*;

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            start_game_click_handler.run_if(in_state(GameState::TitleScreen)),
        );
    }
}

fn start_game_click_handler(
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Left) || touches.any_just_pressed() {
        next_state.set(GameState::Game);
        commands.spawn(GameStart {
            game_start: time.elapsed_seconds(),
        });
    }
}
