#![doc = include_str!("../Readme.md")]

// pub mod helpers;
mod navigation_menu;
pub mod style;
mod systems;
mod types;
mod widgets;

use bevy::prelude::*;
use style::Stylesheet;
use types::{CleanUpUI, MenuAssets};

use std::fmt::Debug;
use std::hash::Hash;

pub use navigation_menu::NavigationMenu;
pub use types::{
    ButtonComponent, Menu, MenuIcon, MenuItem, MenuOptions, MenuSelection, NavigationEvent,
    PrimaryMenu, RedrawEvent, RichTextEntry, Selections, VerticalMenuComponent,
};

/// The quickmenu plugin.
/// It requires multiple generic parameters in order to setup. A minimal example.
/// For a full explanation refer to the examples or the README.
/// ```
/// #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// enum Actions {
///     SoundOn,
///     SoundOff,
/// }
///
/// #[derive(Debug)]
/// enum MyEvent { SoundChanged }
///
/// impl ActionTrait for Actions {
///    type State = CustomState;
///    type Event = MyEvent;
///    fn handle(&self, state: &mut CustomState, event_writer: &mut EventWriter<MyEvent>) {
///         // handle action
///    }
/// }
///
/// #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// enum Screens {
///     Root,
///     Sound,
/// }
///
/// impl ScreenTrait for Screens {
///     fn resolve(&self, state: &CustomState) -> Menu<Actions, Screens, CustomState> {
///         root_menu(state)
///     }
/// }
///
/// fn root_menu(_state: &CustomState) -> Menu<Actions, Screens, CustomState> {
///     Menu {
///         id: "root",
///         entries: vec![
///             MenuItem::headline("Sound Control"),
///             MenuItem::action("Sound On", Actions::SoundOn),
///             MenuItem::screen("Sound Off", Actions::SoundOff),
///         ]
///     }
/// }
///
/// #[derive(Debug, Clone)]
/// struct CustomState { sound_on: bool }
///
/// impl Plugin for MyApp {
///   fn build(&self, app: &mut App) {
///     app
///         .add_event::<MyEvent>()
///         .add_plugin(QuickMenuPlugin::<CustomState, Actions, Screens>::default())
///   }
/// }
/// ```
pub struct QuickMenuPlugin<S>
where
    S: ScreenTrait + 'static,
{
    s: std::marker::PhantomData<S>,
    options: Option<MenuOptions>,
}

impl<S> QuickMenuPlugin<S>
where
    S: ScreenTrait + 'static,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            s: Default::default(),
            options: None,
        }
    }

    pub fn with_options(options: MenuOptions) -> Self {
        Self {
            s: Default::default(),
            options: Some(options),
        }
    }
}

impl<State, A, S> Plugin for QuickMenuPlugin<S>
where
    State: 'static + Send + Sync,
    A: ActionTrait<State = State> + 'static,
    S: ScreenTrait<Action = A, State = State> + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.options.unwrap_or_default())
            .init_resource::<MenuAssets>()
            .insert_resource(Selections::default())
            .add_event::<NavigationEvent>()
            .add_event::<RedrawEvent>()
            .add_system(systems::cleanup_system::<S>.run_if(resource_exists::<CleanUpUI>()))
            .add_systems((
                systems::mouse_system::<S>.run_if(resource_exists::<MenuState<S>>()),
                systems::input_system::<S>.run_if(resource_exists::<MenuState<S>>()),
                systems::redraw_system::<S>.run_if(resource_exists::<MenuState<S>>()),
                systems::keyboard_input_system.run_if(resource_exists::<MenuState<S>>()),
            ));
    }
}

/// Remove the menu
pub fn cleanup(commands: &mut Commands) {
    commands.init_resource::<CleanUpUI>();
}

/// A type conforming to this trait is used to handle the events that
/// are generated as the user interacts with the menu
pub trait ActionTrait: Debug + PartialEq + Eq + Clone + Copy + Hash + Send + Sync {
    type State;
    type Event: Send + Sync + 'static;
    fn handle(&self, state: &mut Self::State, event_writer: &mut EventWriter<Self::Event>);
}

/// Each Menu / Screen uses this trait to define which menu items lead
/// to which other screens
pub trait ScreenTrait: Debug + PartialEq + Eq + Clone + Copy + Hash + Send + Sync {
    type Action: ActionTrait<State = Self::State>;
    type State: Send + Sync + 'static;
    fn resolve(&self, state: &<<Self as ScreenTrait>::Action as ActionTrait>::State) -> Menu<Self>;
}

/// The primary state resource of the menu
#[derive(Resource)]
pub struct MenuState<S>
where
    S: ScreenTrait + 'static,
{
    menu: NavigationMenu<S>,
    pub initial_render_done: bool,
}

impl<S> MenuState<S>
where
    S: ScreenTrait + 'static,
{
    pub fn new(state: S::State, screen: S, sheet: Option<Stylesheet>) -> Self {
        Self {
            menu: NavigationMenu::new(state, screen, sheet),
            initial_render_done: false,
        }
    }

    /// Get a mutable reference to the state in order to change it.
    /// Changing something here will cause a re-render in the next frame.
    /// Due to the way bevy works, just getting this reference, without actually performing
    /// a change is enough to cause a re-render.
    pub fn state_mut(&mut self) -> &mut S::State {
        &mut self.menu.state
    }

    /// Can a immutable reference to the state.
    pub fn state(&self) -> &S::State {
        &self.menu.state
    }
}
