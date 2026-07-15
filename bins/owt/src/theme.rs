//! Color theme (`docs/design/tui-client.md` § Terminal hygiene).
//!
//! Skeleton pass uses named ANSI colors so the palette degrades cleanly to a
//! 16-color monochrome-safe terminal; truecolor/256-color refinement is a follow-up.
//! Every color the UI draws goes through a [`Theme`] so a light/dark swap is one
//! value change.

use owt_client::config::Theme as ThemeChoice;
use ratatui::style::Color;

/// The resolved palette the widgets draw with.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Primary foreground.
    pub fg: Color,
    /// Dimmed / secondary text.
    pub dim: Color,
    /// Accent (focus, selection, titles).
    pub accent: Color,
    /// Pane borders.
    pub border: Color,
    /// Focused pane border.
    pub border_focus: Color,
    /// "Good" / connected / buy.
    pub good: Color,
    /// "Warn" / degraded / stale.
    pub warn: Color,
    /// "Bad" / offline / sell.
    pub bad: Color,
}

impl Theme {
    /// Resolve a theme from the configured choice.
    pub fn from_choice(choice: ThemeChoice) -> Self {
        match choice {
            ThemeChoice::Dark => Self::dark(),
            ThemeChoice::Light => Self::light(),
        }
    }

    /// The default dark theme.
    pub fn dark() -> Self {
        Self {
            fg: Color::Gray,
            dim: Color::DarkGray,
            accent: Color::Cyan,
            border: Color::DarkGray,
            border_focus: Color::Cyan,
            good: Color::Green,
            warn: Color::Yellow,
            bad: Color::Red,
        }
    }

    /// The light theme.
    pub fn light() -> Self {
        Self {
            fg: Color::Black,
            dim: Color::DarkGray,
            accent: Color::Blue,
            border: Color::Gray,
            border_focus: Color::Blue,
            good: Color::Green,
            warn: Color::Yellow,
            bad: Color::Red,
        }
    }

    /// The color for a connection state's status dot.
    pub fn conn_color(&self, state: owt_client::reconnect::ConnState) -> Color {
        use owt_client::reconnect::ConnState::*;
        match state {
            Connected => self.good,
            Degraded | Reconnecting => self.warn,
            Offline => self.bad,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
