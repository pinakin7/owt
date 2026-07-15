//! Command palette state and its grammar parser.
//!
//! Activated with `:` or `/` (both, per the LLD, for muscle memory). Holds the input
//! buffer, the last inline parse error, and history. Completion is server-driven in
//! the full design (debounced `/v1/search`); the skeleton leaves completion as a
//! follow-up and focuses on the input + parse + dispatch path.

pub mod parse;

pub use parse::{Arg, Command, Op, ParseError, Verb, parse};

/// The palette's live state.
#[derive(Debug, Clone, Default)]
pub struct PaletteState {
    /// Whether the palette overlay is showing.
    pub open: bool,
    /// The prefix the user opened with (`:` or `/`), shown in the prompt.
    pub prefix: char,
    /// Current input buffer (without the prefix).
    pub buffer: String,
    /// The last parse error, rendered inline with its caret.
    pub error: Option<ParseError>,
    /// Recently submitted commands, newest last.
    pub history: Vec<String>,
}

impl PaletteState {
    /// Open the palette with the given prefix, clearing prior input.
    pub fn open(&mut self, prefix: char) {
        self.open = true;
        self.prefix = prefix;
        self.buffer.clear();
        self.error = None;
    }

    /// Close the palette and clear transient state.
    pub fn close(&mut self) {
        self.open = false;
        self.buffer.clear();
        self.error = None;
    }

    /// Append a typed character.
    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
        self.error = None;
    }

    /// Delete the last character; returns `true` if the buffer is now empty.
    pub fn backspace(&mut self) -> bool {
        self.buffer.pop();
        self.error = None;
        self.buffer.is_empty()
    }

    /// Parse the current buffer, recording history and any inline error.
    pub fn submit(&mut self) -> Result<Command, ParseError> {
        let result = parse(&self.buffer);
        match &result {
            Ok(_) => self.history.push(self.buffer.clone()),
            Err(e) => self.error = Some(e.clone()),
        }
        result
    }
}
