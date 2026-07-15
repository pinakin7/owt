//! The semantic actions keys resolve to, decoupled from physical keys so remapping
//! (`[keys]` config) is just a different key→action table.

/// A resolved user intent in the navigation (non-palette) context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Open the command palette with the given prefix (`:` or `/`).
    OpenPalette(char),
    /// Toggle the help overlay.
    Help,
    /// Quit the application.
    Quit,
    /// Close overlay / pop the route stack.
    Back,
    /// Cycle pane focus forward.
    FocusNext,
    /// Cycle pane focus backward.
    FocusPrev,
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move focus/selection left.
    Left,
    /// Move focus/selection right.
    Right,
    /// Jump to the top of the current list (`gg`).
    Top,
    /// Jump to the bottom of the current list (`G`).
    Bottom,
    /// Open the current selection.
    Open,
    /// Toggle watch on the current selection.
    ToggleWatch,
    /// Jump to watchlist `n` (1–9).
    Jump(u8),
    /// The first `g` of a potential `gg`; resolved statefully by `update`.
    GPrefix,
}
