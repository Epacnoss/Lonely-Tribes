use super::welcome_state::StartGameState;
use amethyst::{
    core::ecs::{Builder, World, WorldExt},
    input::{InputEvent, VirtualKeyCode},
    ui::{Anchor, LineMode, UiText, UiTransform},
    GameData, SimpleState, SimpleTrans, StateData, StateEvent,
};
use lonely_tribes_lib::states_util::{get_scaling_factor, load_font};
use amethyst::winit::{WindowEvent, Event};
use lonely_tribes_lib::config::change_screen_res;

///Text displayed in HelpState
pub const HELP_TXT: &str = include_str!("help_text.txt");

///State to show Help
#[derive(Default)]
pub struct HelpState;

impl SimpleState for HelpState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.delete_all();

        get_help_txt(world);
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let mut t = SimpleTrans::None;
        match event {
            StateEvent::Input(InputEvent::KeyPressed {key_code, ..}) => {
                if key_code == VirtualKeyCode::Space || key_code == VirtualKeyCode::Return {
                    t = SimpleTrans::Switch(Box::new(StartGameState::default()));
                }
            },
            StateEvent::Window(Event::WindowEvent {window_id: _, event}) => {
                if let WindowEvent::Resized(size) = event {
                    change_screen_res(size.width as u32, size.height as u32);
                }
            }
            _ => {}
        }
        
        t
    }
}

///Function to insert text onto the Help screen
///The text is **not** interactable.
///
///By default, it uses Atkinson Hyperlegible
fn get_help_txt(world: &mut World) {
    let (sf_x, sf_y) = get_scaling_factor();
    let trans = UiTransform::new(
        "help_txt".to_string(),
        Anchor::Middle,
        Anchor::Middle,
        0.0,
        0.0,
        0.5,
        sf_x * 1500.0,
        sf_y * 800.0,
    );
    let txt = UiText::new(
        load_font(world, "Hyperlegible"),
        HELP_TXT.to_string(),
        [1.0; 4],
        sf_y * 42.5,
        LineMode::Wrap,
        Anchor::MiddleLeft,
    );
    world.create_entity().with(trans).with(txt).build();
}
