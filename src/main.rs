use std::ffi::OsString;
use std::fs::File;
use std::io::{Write, Read};
use std::os::unix::net::UnixStream;
use serde::Serialize;

fn flatten_subtable(table: &toml::Table, res: &mut Vec<String>, current_path: &str) {
    for (key, value) in table.iter() {
        match value {
            toml::Value::String(_) |
            toml::Value::Integer(_) |
            toml::Value::Float(_) | 
            toml::Value::Boolean(_) |
            toml::Value::Datetime(_) => {
                if current_path.is_empty() {
                    res.push(format!("{key} = {value}"));
                } else {
                    res.push(format!("{current_path}.{key} = {value}"));
                }
            },
            toml::Value::Table(tbl) => {
                if current_path.is_empty() {
                    flatten_subtable(tbl, res, key);
                } else {
                    flatten_subtable(tbl, res, &format!("{current_path}.{key}"));
                }
            }
            toml::Value::Array(_) => {
                // NB(directxman12): right now this is serialized as yaml, but really it should be
                // toml in the future.  For basic values the syntax is close enough to be fine

                // TODO(directxman12): special handling for this?
                if current_path.is_empty() {
                    res.push(format!("{key} = {value}"));
                } else {
                    res.push(format!("{current_path}.{key} = {value}"));
                }
            }
        }
    }
}

fn flatten_settings(raw: toml::Table) -> Vec<String> {
    let mut res = vec![];
    flatten_subtable(&raw, &mut res, "");
    res
}

#[derive(Debug)]
struct Args {
    window_id: Option<i128>,
    input: Input,
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
impl FromIterator<OsString> for Args {
    fn from_iter<T: IntoIterator<Item = OsString>>(raw: T) -> Self {
        let mut args = Args::default();
        let mut raw = raw.into_iter();
        raw.next().expect("should have program name at least");
        let mut seen_filename = false;
        while let Some(arg) = raw.next() {
            match arg.to_str() {
                Some("--window" | "-w") => {
                    let raw_id = raw.next().expect("must pass an id to -w|--window");
                    args.window_id = Some(match raw_id {
                        _ if raw_id == "all" => -1,
                        _ => raw_id.to_str().expect("invalid window id").parse().expect("invalid window id"),
                    });
                },
                Some(arg) if arg.starts_with("--window=") || arg.starts_with("-w=") => {
                    let (_, raw_id) = arg.split_once("=").expect("by the above match");
                    args.window_id = Some(match raw_id {
                        "all" => -1,
                        _ => raw_id.parse().expect("invalid window id"),
                    });
                },
                Some(_) | None if !seen_filename => {
                    seen_filename = true;
                    args.input = Input::File(File::open(arg).expect("unable to open toml file"));
                },
                _ => {
                    panic!("unexpected/invalid argument {arg:?}");
                },
            }
        }
        args
    }
}
#[derive(Debug)]
enum Input {
    StdIn,
    File(File),
}
impl Input {
    fn read_to_string(self) -> std::io::Result<String> {
        let mut res = String::new();
        match self {
            Input::StdIn => std::io::stdin().lock().read_to_string(&mut res)?,
            Input::File(mut f) => f.read_to_string(&mut res)?,
        };
        Ok(res)
    }
}

fn main() {
    let args: Args = std::env::args_os().collect();

    let options = {
        let raw = args.input.read_to_string().expect("couldn't read that file");
        let deserialized = toml::from_str(&raw).expect("that file wasn't a valid toml table");

        flatten_settings(deserialized)
    };


    let msg = SocketMessage::Config(IpcConfig {
        options,
        window_id: args.window_id,
        reset: false,
    });

    let mut socket = {
        let path = std::env::var_os("ALACRITTY_SOCKET").expect("no ALACRITTY_SOCKET found");
        UnixStream::connect(path).expect("could not connect to alacritty socket {path:?}")
    };

    socket.write_all(serde_json::to_string(&msg).expect("couldn't serialize message").as_bytes()).expect("couldn't send message");
    socket.flush().expect("couldn't flush socket");
}

#[derive(Serialize, Debug)]
enum SocketMessage {
    Config(IpcConfig),
}
#[derive(Serialize, Debug)]
struct IpcConfig {
    options: Vec<String>,
    window_id: Option<i128>,
    reset: bool,
}
