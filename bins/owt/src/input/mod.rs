//! Keymap: crossterm key events → semantic [`Action`]s.
//!
//! A default map (`docs/design/tui-client.md` § Keybindings) is overlaid with the
//! user's `[keys]` config table. Each binding is keyed by a normalized token string
//! (e.g. `ctrl-c`, `tab`, `j`, `:`), so remapping is a table edit and the resolver is
//! pure and unit-testable.

pub mod action;

use std::collections::HashMap;

pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use owt_client::config::KeysConfig;

/// A key→action table, built from the defaults plus config overrides.
#[derive(Debug, Clone)]
pub struct Keymap {
    bindings: HashMap<String, Action>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new(&KeysConfig::default())
    }
}

impl Keymap {
    /// Build a keymap: defaults first, then apply `[keys]` overrides (action → token).
    pub fn new(overrides: &KeysConfig) -> Self {
        let mut bindings: HashMap<String, Action> = HashMap::new();
        for (token, action) in DEFAULTS {
            bindings.insert((*token).to_owned(), *action);
        }
        for (name, spec) in overrides.0.iter() {
            if let Some(action) = action_by_name(name) {
                bindings.insert(spec.clone(), action);
            }
        }
        Self { bindings }
    }

    /// Resolve a key event in the navigation context.
    pub fn resolve(&self, key: &KeyEvent) -> Option<Action> {
        self.bindings.get(&token(key)).copied()
    }
}

/// Normalize a key event into a lookup token, e.g. `ctrl-c`, `tab`, `:`, `j`, `G`.
///
/// Character case is preserved (so `G` = shift-g is distinct from `g`); `ctrl-`
/// combinations are lowercased for a stable spelling.
pub fn token(key: &KeyEvent) -> String {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                format!("ctrl-{}", c.to_ascii_lowercase())
            } else {
                c.to_string()
            }
        }
        KeyCode::Tab => "tab".to_owned(),
        KeyCode::BackTab => "shift-tab".to_owned(),
        KeyCode::Enter => "enter".to_owned(),
        KeyCode::Esc => "esc".to_owned(),
        KeyCode::Up => "up".to_owned(),
        KeyCode::Down => "down".to_owned(),
        KeyCode::Left => "left".to_owned(),
        KeyCode::Right => "right".to_owned(),
        _ => String::new(),
    }
}

/// The default key→action bindings.
const DEFAULTS: &[(&str, Action)] = &[
    (":", Action::OpenPalette(':')),
    ("/", Action::OpenPalette('/')),
    ("?", Action::Help),
    ("q", Action::Quit),
    ("ctrl-c", Action::Quit),
    ("esc", Action::Back),
    ("tab", Action::FocusNext),
    ("shift-tab", Action::FocusPrev),
    ("enter", Action::Open),
    ("w", Action::ToggleWatch),
    ("g", Action::GPrefix),
    ("G", Action::Bottom),
    ("k", Action::Up),
    ("up", Action::Up),
    ("j", Action::Down),
    ("down", Action::Down),
    ("h", Action::Left),
    ("left", Action::Left),
    ("l", Action::Right),
    ("right", Action::Right),
    ("1", Action::Jump(1)),
    ("2", Action::Jump(2)),
    ("3", Action::Jump(3)),
    ("4", Action::Jump(4)),
    ("5", Action::Jump(5)),
    ("6", Action::Jump(6)),
    ("7", Action::Jump(7)),
    ("8", Action::Jump(8)),
    ("9", Action::Jump(9)),
];

/// Map a config action name (the `[keys]` key) to an [`Action`]. `Jump`/`GPrefix`
/// and the palette prefixes are not remappable in the skeleton.
fn action_by_name(name: &str) -> Option<Action> {
    Some(match name {
        "help" => Action::Help,
        "quit" => Action::Quit,
        "back" => Action::Back,
        "focus_next" => Action::FocusNext,
        "focus_prev" => Action::FocusPrev,
        "open" => Action::Open,
        "watch" => Action::ToggleWatch,
        "up" => Action::Up,
        "down" => Action::Down,
        "left" => Action::Left,
        "right" => Action::Right,
        "top" => Action::Top,
        "bottom" => Action::Bottom,
        "palette" => Action::OpenPalette(':'),
        _ => return None,
    })
}
