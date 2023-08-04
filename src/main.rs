use std::ffi::OsString;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::process::ExitCode;
use serde::Serialize;

mod flatten;
mod args;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("unable to read input -- {0}")]
    InputReading(std::io::Error),
    #[error("unable to parse input as toml table -- {0}")]
    InputParsing(toml::de::Error),
    #[error("no ALACRITTY_SOCKET found")]
    NoSocket,
    #[error("unable to connect to alacritty socket {socket:?} -- {error}")]
    UnableToConnect { socket: OsString, error: std::io::Error },
    #[error("unable to serialize message for alacritty -- {0}")]
    UnableToSerialize(serde_json::Error),
    #[error("unable to send message to alacritty -- {0}")]
    UnableToSend(std::io::Error),
}

fn main() -> Result<ExitCode, Error> {
    let args: args::Args = match lexopt::Parser::from_env().try_into() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            eprintln!("");
            eprintln!("usage: alap [--window (self|all|ID)] [FILE|-]");
            eprintln!("\t--window (self|all|ID): window to apply to, defaults to 'all'");
            eprintln!("\t[FILE|-]: toml file to read from, default to '-' (stdin)");

            return Ok(ExitCode::FAILURE);
        },
    };

    let options = {
        let raw = args.input.read_to_string().map_err(Error::InputReading)?;
        let deserialized = toml::from_str(&raw).map_err(Error::InputParsing)?;

        flatten::settings(deserialized)
    };


    let msg = SocketMessage::Config(IpcConfig {
        options,
        window_id: args.window_id,
        reset: false,
    });

    let mut socket = {
        let path = std::env::var_os("ALACRITTY_SOCKET").ok_or(Error::NoSocket)?;
        UnixStream::connect(&path).map_err(|error| Error::UnableToConnect { socket: path, error })?
    };

    socket.write_all(serde_json::to_string(&msg).map_err(Error::UnableToSerialize)?.as_bytes()).map_err(Error::UnableToSend)?;
    socket.flush().map_err(Error::UnableToSend)?;

    Ok(ExitCode::SUCCESS)
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
