//! The application core: `AppState`, the message/command alphabet, and the pure
//! `update` function (The Elm Architecture, `docs/design/tui-client.md`).
//!
//! `update(&mut AppState, Msg) -> Vec<Cmd>` is pure — it mutates state and returns
//! side-effect requests, but performs no I/O. The runtime executes the `Cmd`s and
//! feeds results back as `Msg`s. This is what makes the whole client unit-testable
//! without a terminal.

pub mod cmd;
pub mod msg;
pub mod route;
pub mod update;

pub use cmd::Cmd;
pub use msg::{Loaded, Msg};
pub use route::Route;
pub use update::update;

use owt_client::config::EffectiveConfig;
use owt_client::reconnect::ConnState;

use crate::input::Keymap;
use crate::palette::PaletteState;
use crate::screens::ScreenStates;
use crate::theme::Theme;

/// Terminal-size responsiveness breakpoints (`docs/design/tui-client.md` § Resize).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    /// ≥ 120 cols: full multi-column layout.
    Wide,
    /// 96–119 cols: some columns collapse to tabs.
    Medium,
    /// 80–95 cols: single-column, columns become tabs.
    Narrow,
    /// < 80 cols: below the supported minimum; a hint is shown.
    TooSmall,
}

impl Breakpoint {
    /// Classify a terminal width.
    pub fn for_width(cols: u16) -> Self {
        match cols {
            0..=79 => Breakpoint::TooSmall,
            80..=95 => Breakpoint::Narrow,
            96..=119 => Breakpoint::Medium,
            _ => Breakpoint::Wide,
        }
    }
}

/// A transient on-screen notice.
#[derive(Debug, Clone)]
pub struct Toast {
    /// The message text.
    pub text: String,
    /// Severity, driving color.
    pub kind: ToastKind,
}

/// Toast severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// Informational.
    Info,
    /// An error / failure.
    Error,
}

/// The whole application state.
#[derive(Debug)]
pub struct AppState {
    /// Navigation history; the last entry is the current screen. Never empty.
    pub route_stack: Vec<Route>,
    /// Per-screen state.
    pub screens: ScreenStates,
    /// Connection health (status bar).
    pub conn: ConnState,
    /// Command palette.
    pub palette: PaletteState,
    /// Active toasts, oldest first.
    pub toasts: Vec<Toast>,
    /// Effective configuration.
    pub cfg: EffectiveConfig,
    /// Resolved color theme.
    pub theme: Theme,
    /// Resolved keymap.
    pub keymap: Keymap,
    /// Current terminal size `(cols, rows)`.
    pub size: (u16, u16),
    /// Whether the help overlay is showing.
    pub help_open: bool,
    /// Set by a `Quit` intent; the runtime exits when true.
    pub should_quit: bool,
    /// Whether the next frame needs a redraw.
    pub dirty: bool,
    /// `--debug` overlay (FPS / frame time) enabled.
    pub debug: bool,
    /// Whether the last key was a lone `g` (for the `gg` chord).
    pub pending_g: bool,
}

impl AppState {
    /// Build the initial state from effective config.
    pub fn new(cfg: EffectiveConfig) -> Self {
        let theme = Theme::from_choice(cfg.ui.theme);
        let keymap = Keymap::new(&cfg.keys);
        Self {
            route_stack: vec![Route::Search],
            screens: ScreenStates::default(),
            conn: ConnState::default(),
            palette: PaletteState::default(),
            toasts: Vec::new(),
            cfg,
            theme,
            keymap,
            size: (80, 24),
            help_open: false,
            should_quit: false,
            dirty: true,
            debug: false,
            pending_g: false,
        }
    }

    /// The current (top-of-stack) route.
    pub fn route(&self) -> &Route {
        self.route_stack.last().unwrap_or(&Route::Search)
    }

    /// The active breakpoint for the current width.
    pub fn breakpoint(&self) -> Breakpoint {
        Breakpoint::for_width(self.size.0)
    }

    /// Push a route and mark dirty.
    pub fn push_route(&mut self, route: Route) {
        self.route_stack.push(route);
        self.dirty = true;
    }

    /// Pop the current route if there's something to go back to. Returns whether a
    /// pop happened (false when already at the root).
    pub fn pop_route(&mut self) -> bool {
        if self.route_stack.len() > 1 {
            self.route_stack.pop();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Push an informational toast.
    pub fn info(&mut self, text: impl Into<String>) {
        self.push_toast(text.into(), ToastKind::Info);
    }

    /// Push an error toast.
    pub fn error(&mut self, text: impl Into<String>) {
        self.push_toast(text.into(), ToastKind::Error);
    }

    fn push_toast(&mut self, text: String, kind: ToastKind) {
        self.toasts.push(Toast { text, kind });
        // Bound the backlog; only the newest few are ever shown.
        const MAX_TOASTS: usize = 20;
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
        self.dirty = true;
    }
}
