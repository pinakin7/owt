//! The command-palette grammar parser (`docs/design/tui-client.md` § Command palette
//! & grammar).
//!
//! Implements the LLD EBNF:
//!
//! ```ebnf
//! command = "/" verb [ ws args ] ;
//! args    = { arg ws } arg ;
//! arg     = ident | quoted | filter ;
//! filter  = key ":" [ op ] value ;   (* tag:politics  liquidity:>10000 *)
//! op      = ">" | "<" | ">=" | "<=" ;
//! ```
//!
//! A leading `/` is optional here: the palette is opened with `:` or `/`, so the
//! buffer may or may not carry the slash. The parser is pure and standalone — the LLD
//! wants it promoted into `owt-domain` once search filters and alert predicates share
//! it; for the skeleton it lives with its only consumer.

/// A parsed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The verb.
    pub verb: Verb,
    /// Positional / filter arguments.
    pub args: Vec<Arg>,
}

/// The recognized verbs (`docs/product/prd-mvp.md` § Commands).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    /// Open a market or event detail screen.
    Open,
    /// Open a topic page.
    Topic,
    /// Add to a watchlist.
    Watch,
    /// Remove from a watchlist.
    Unwatch,
    /// Pin a pane.
    Pin,
    /// Compare two markets side by side.
    Compare,
    /// Scoped news search.
    News,
    /// Export the current pane.
    Export,
    /// Saved views.
    View,
    /// Alerts (parses in MVP; responds "alerts arrive in v1").
    Alert,
    /// Help overlay.
    Help,
    /// Quit.
    Quit,
}

impl Verb {
    fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "open" => Verb::Open,
            "topic" => Verb::Topic,
            "watch" => Verb::Watch,
            "unwatch" => Verb::Unwatch,
            "pin" => Verb::Pin,
            "compare" => Verb::Compare,
            "news" => Verb::News,
            "export" => Verb::Export,
            "view" => Verb::View,
            "alert" => Verb::Alert,
            "help" => Verb::Help,
            "quit" | "q" => Verb::Quit,
            _ => return None,
        })
    }
}

/// A single argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    /// A bare identifier / slug.
    Ident(String),
    /// A quoted string (quotes stripped).
    Quoted(String),
    /// A `key:[op]value` filter.
    Filter {
        /// The filter key (e.g. `tag`, `liquidity`).
        key: String,
        /// The comparison operator.
        op: Op,
        /// The filter value.
        value: String,
    },
}

/// A filter comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Equality (no operator prefix).
    Eq,
    /// Greater than.
    Gt,
    /// Less than.
    Lt,
    /// Greater than or equal.
    Ge,
    /// Less than or equal.
    Le,
}

/// A parse failure, carrying the caret column the palette highlights inline
/// (`docs/design/tui-client.md`: "Errors render inline with the caret position").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable message.
    pub message: String,
    /// Zero-based caret column into the input where the error is.
    pub caret: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Parse a palette buffer into a [`Command`].
pub fn parse(input: &str) -> Result<Command, ParseError> {
    let trimmed = input.trim_start_matches('/');
    let lead = input.len() - trimmed.len();

    let tokens = tokenize(trimmed, lead)?;
    let mut iter = tokens.into_iter();

    let (verb_word, verb_at) = match iter.next() {
        Some(t) => t,
        None => {
            return Err(ParseError {
                message: "type a command".to_owned(),
                caret: lead,
            });
        }
    };

    let verb = Verb::parse(&verb_word).ok_or_else(|| ParseError {
        message: format!("unknown command `{verb_word}`"),
        caret: verb_at,
    })?;

    let mut args = Vec::new();
    for (word, at) in iter {
        args.push(classify(&word, at)?);
    }

    Ok(Command { verb, args })
}

/// Split into whitespace-separated tokens, honoring `"double quotes"`. Each token
/// keeps the absolute caret column (offset by `lead`) for error reporting.
fn tokenize(s: &str, lead: usize) -> Result<Vec<(String, usize)>, ParseError> {
    let mut out = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(start, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '"' {
            chars.next(); // opening quote
            let mut buf = String::from('"');
            let mut closed = false;
            for (_, ch) in chars.by_ref() {
                buf.push(ch);
                if ch == '"' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err(ParseError {
                    message: "unterminated quoted string".to_owned(),
                    caret: lead + start,
                });
            }
            out.push((buf, lead + start));
        } else {
            let mut buf = String::new();
            while let Some(&(_, ch)) = chars.peek() {
                if ch.is_whitespace() {
                    break;
                }
                buf.push(ch);
                chars.next();
            }
            out.push((buf, lead + start));
        }
    }
    Ok(out)
}

/// Classify a single non-verb token into an [`Arg`].
fn classify(word: &str, at: usize) -> Result<Arg, ParseError> {
    if word.starts_with('"') {
        // tokenizer guarantees a closing quote
        let inner = word.trim_matches('"').to_owned();
        return Ok(Arg::Quoted(inner));
    }
    if let Some((key, rest)) = word.split_once(':') {
        if key.is_empty() {
            return Err(ParseError {
                message: "filter is missing a key".to_owned(),
                caret: at,
            });
        }
        let (op, value) = split_op(rest);
        if value.is_empty() {
            return Err(ParseError {
                message: format!("filter `{key}` is missing a value"),
                caret: at,
            });
        }
        return Ok(Arg::Filter {
            key: key.to_owned(),
            op,
            value: value.to_owned(),
        });
    }
    Ok(Arg::Ident(word.to_owned()))
}

/// Peel a leading comparison operator off a filter value.
fn split_op(rest: &str) -> (Op, &str) {
    if let Some(v) = rest.strip_prefix(">=") {
        (Op::Ge, v)
    } else if let Some(v) = rest.strip_prefix("<=") {
        (Op::Le, v)
    } else if let Some(v) = rest.strip_prefix('>') {
        (Op::Gt, v)
    } else if let Some(v) = rest.strip_prefix('<') {
        (Op::Lt, v)
    } else {
        (Op::Eq, rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verb_and_slug() {
        let c = parse("/open will-fed-cut").unwrap();
        assert_eq!(c.verb, Verb::Open);
        assert_eq!(c.args, vec![Arg::Ident("will-fed-cut".to_owned())]);
    }

    #[test]
    fn leading_slash_is_optional() {
        assert_eq!(parse("open x").unwrap().verb, Verb::Open);
        assert_eq!(parse("/open x").unwrap().verb, Verb::Open);
    }

    #[test]
    fn quoted_argument() {
        let c = parse(r#"/topic "jerome powell""#).unwrap();
        assert_eq!(c.args, vec![Arg::Quoted("jerome powell".to_owned())]);
    }

    #[test]
    fn filters_with_and_without_operators() {
        let c = parse("/open tag:politics liquidity:>10000").unwrap();
        assert_eq!(
            c.args,
            vec![
                Arg::Filter {
                    key: "tag".to_owned(),
                    op: Op::Eq,
                    value: "politics".to_owned(),
                },
                Arg::Filter {
                    key: "liquidity".to_owned(),
                    op: Op::Gt,
                    value: "10000".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn all_operators() {
        let c = parse("/news a:>=1 b:<=2 c:<3").unwrap();
        let ops: Vec<Op> = c
            .args
            .iter()
            .filter_map(|a| match a {
                Arg::Filter { op, .. } => Some(*op),
                _ => None,
            })
            .collect();
        assert_eq!(ops, vec![Op::Ge, Op::Le, Op::Lt]);
    }

    #[test]
    fn unknown_verb_reports_caret() {
        let err = parse("/frobnicate x").unwrap_err();
        assert_eq!(err.caret, 1, "caret points at the verb after the slash");
        assert!(err.message.contains("frobnicate"));
    }

    #[test]
    fn empty_input_is_an_error() {
        assert!(parse("").is_err());
        assert!(parse("/").is_err());
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        assert!(parse(r#"/topic "unclosed"#).is_err());
    }

    #[test]
    fn missing_filter_value_is_an_error() {
        assert!(parse("/open tag:").is_err());
    }

    #[test]
    fn quit_alias() {
        assert_eq!(parse("/q").unwrap().verb, Verb::Quit);
        assert_eq!(parse("/quit").unwrap().verb, Verb::Quit);
    }
}
