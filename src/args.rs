use std::env::VarError;
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;

use lexopt::{Arg, ValueExt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] lexopt::Error),
    #[error("unexpected flag {0}")]
    UnexpectedFlag(String),
    #[error("cannot pass {0} multiple times")]
    Duplicate(String),
    #[error(transparent)]
    BadInput(#[from] std::io::Error),
    #[error("invalid window id -- {0}")]
    InvalidWindowId(Box<dyn std::error::Error>),
    #[error("missing window id -- not specificed/invalid, no ALACRITTY_WINDOW_ID set ({0})")]
    MissingWindowId(#[from] VarError),
    #[error("Help:")]
    Help,
}

#[derive(Debug)]
pub struct Args {
    /// the window to target
    ///
    /// Per alacritty, `Some(-1)` means all
    pub window_id: Option<i128>,

    /// the source of the config (stdin or a file)
    pub input: Input,
}
impl Default for Args {
    fn default() -> Self {
        let window_id = std::env::var("ALACRITTY_WINDOW_ID").ok().and_then(|id| if id.is_empty() {
            None
        } else {
            Some(id)
        }).map(|id| id.parse().expect("couldn't parse window id from ALACRITTY_WINDOW_ID"));
        Self { window_id, input: Input::StdIn }
    }
}
impl TryFrom<lexopt::Parser> for Args {
    type Error = Error;

    fn try_from(mut raw: lexopt::Parser) -> Result<Self, Self::Error> {
        let mut saw_input = false;
        let mut res = Args {
            window_id: None,
            input: Input::StdIn,
        };
        while let Some(arg) = raw.next()? {
            match arg {
                Arg::Short('h') | Arg::Long("help") => {
                    return Err(Error::Help)
                },
                Arg::Short('w') | Arg::Long("window") => {
                    if res.window_id.is_some() {
                        return Err(Error::Duplicate("window".to_string()));
                    }
                    let val = raw.value()?;
                    res.window_id = Some(match val {
                        _ if val == "all" => -1,
                        _ if val == "self" => std::env::var("ALACRITTY_WINDOW_ID")?.parse().map_err(|e: ParseIntError| Error::InvalidWindowId(e.into()))?,
                        _ => val.parse().map_err(|e| Error::InvalidWindowId(e.into()))?,
                    });
                },
                Arg::Value(v) if v == "-" => {
                    if saw_input {
                        return Err(Error::Duplicate("FILE (stdin)".to_string()));
                    }
                    saw_input = true;
                    // already stdin
                },
                Arg::Value(v) => {
                    if saw_input {
                        return Err(Error::Duplicate("FILE".to_string()));
                    }
                    saw_input = true;
                    res.input = Input::File(File::open(v)?);
                },
                Arg::Short(s) => return Err(Error::UnexpectedFlag(format!("-{s}"))),
                Arg::Long(l) => return Err(Error::UnexpectedFlag(format!("--{l}"))),
            }
        };

        if res.window_id.is_none() {
            res.window_id = Some(-1);
        }

        Ok(res)
    }
}
#[derive(Debug)]
pub enum Input {
    StdIn,
    File(File),
}
impl Input {
    pub fn read_to_string(self) -> std::io::Result<String> {
        let mut res = String::new();
        match self {
            Input::StdIn => std::io::stdin().lock().read_to_string(&mut res)?,
            Input::File(mut f) => f.read_to_string(&mut res)?,
        };
        Ok(res)
    }
}
