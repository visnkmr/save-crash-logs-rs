//! This module encapsulates the report of a failure event.
//!
//! A `Report` contains the metadata collected about the event
//! to construct a helpful error message.

use backtrace::Backtrace;
use serde_derive::Serialize;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::fs::create_dir_all;
use std::mem;
use std::{env, fs::File, io::Write, path::Path, path::PathBuf};
use uuid::Uuid;

/// Method of failure.
#[derive(Debug, Serialize, Clone, Copy)]
pub enum Method {
    /// Failure caused by a panic.
    Panic,
}

/// Contains metadata about the crash like the backtrace and
/// information about the crate and operating system. Can
/// be used to be serialized and persisted or printed as
/// information to the user.
#[derive(Debug, Serialize)]
pub struct Report {
    name: String,
    operating_system: String,
    crate_version: String,
    explanation: String,
    cause: String,
    method: Method,
    backtrace: String,
    path: PathBuf,
}

impl Report {
    /// Create a new instance.
    pub fn new(
        name: &str,
        version: &str,
        method: Method,
        explanation: String,
        cause: String,
        path_to_save: &PathBuf,
    ) -> Self {
        let operating_system = os_info::get().to_string();

        //We skip 3 frames from backtrace library
        //Then we skip 3 frames for our own library
        //(including closure that we set as hook)
        //Then we skip 2 functions from Rust's runtime
        //that calls panic hook
        const SKIP_FRAMES_NUM: usize = 8;
        //We take padding for address and extra two letters
        //to padd after index.
        const HEX_WIDTH: usize = mem::size_of::<usize>() + 2;
        //Padding for next lines after frame's address
        const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;

        let mut backtrace = String::new();

        //Here we iterate over backtrace frames
        //(each corresponds to function's stack)
        //We need to print its address
        //and symbol(e.g. function name),
        //if it is available
        for (idx, frame) in Backtrace::new()
            .frames()
            .iter()
            .skip(SKIP_FRAMES_NUM)
            .enumerate()
        {
            let ip = frame.ip();
            let _ = write!(backtrace, "\n{idx:4}: {ip:HEX_WIDTH$?}");

            let symbols = frame.symbols();
            if symbols.is_empty() {
                let _ = write!(backtrace, " - <unresolved>");
                continue;
            }

            for (idx, symbol) in symbols.iter().enumerate() {
                //Print symbols from this address,
                //if there are several addresses
                //we need to put it on next line
                if idx != 0 {
                    let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
                }

                if let Some(name) = symbol.name() {
                    let _ = write!(backtrace, " - {name}");
                } else {
                    let _ = write!(backtrace, " - <unknown>");
                }

                //See if there is debug information with file name and line
                if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                    let _ = write!(
                        backtrace,
                        "\n{:3$}at {}:{}",
                        "",
                        file.display(),
                        line,
                        NEXT_SYMBOL_PADDING
                    );
                }
            }
        }

        Self {
            crate_version: version.into(),
            name: name.into(),
            operating_system,
            method,
            explanation,
            cause,
            backtrace,
            path:path_to_save.to_owned()
        }
    }

    /// Serialize the `Report` to a TOML string.
    pub fn serialize(&self) -> Option<String> {
        toml::to_string_pretty(&self).ok()
    }

    /// Write a file to disk.
    pub fn persist(&self) -> Result<PathBuf, Box<dyn Error + 'static>> {
        let uuid = Uuid::new_v4().hyphenated().to_string();
        let binding = env::temp_dir();
        let tmp_dir = match(create_dir_all(&self.path)){
            Ok(_) => &self.path,
            Err(_) => &binding,
        };
        let date = chrono::Local::now();
        let current_date = date.format("%Y-%m-%d %H:%M").to_string();
        let file_name = format!("report-{}-{}.toml",current_date, &uuid);
        let file_path = Path::new(&tmp_dir).join(file_name);
        let mut file = File::create(&file_path)?;
        let toml = self.serialize().unwrap();
        file.write_all(toml.as_bytes())?;
        Ok(file_path)
    }
}
