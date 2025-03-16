//! Parser for GStreamer logs
//! 
//! This module is adapted from the gst-log-parser crate by Guillaume Desmottes
//! Original repository: https://github.com/gdesmott/gst-log-parser/
// "browser": {
    //   "command": "/Users/rafaelcaricio/.asdf/shims/npx",
    //   "args": ["-y", "@modelcontextprotocol/server-puppeteer"]
//  "fetch": {
//       "command": "/Users/rafaelcaricio/.local/bin/mcp-server-fetch"
//     },   
use itertools::join;
use std::fmt;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::io::Read;
use std::str;
use std::str::FromStr;

use anyhow::Result;
use gstreamer::{ClockTime, DebugLevel, Structure};
use lazy_static::lazy_static;
use regex::Regex;
#[derive(Debug, PartialEq)]
pub enum TimestampField {
    Hour,
    Minute,
    Second,
    SubSecond,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Timestamp { field: Option<TimestampField> },
    PID,
    Thread,
    Level,
    Category,
    File,
    LineNumber,
    Function,
    Message,
    Object,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParsingError {
    #[error("invalid debug level: {name}")]
    InvalidDebugLevel { name: String },
    #[error("invalid timestamp: {ts} : {field:?}")]
    InvalidTimestamp { ts: String, field: TimestampField },
    #[error("missing token: {t:?}")]
    MissingToken { t: Token },
    #[error("invalid PID: {pid}")]
    InvalidPID { pid: String },
    #[error("missing location")]
    MissingLocation,
    #[error("invalid line number: {line}")]
    InvalidLineNumber { line: String },
}

#[derive(Debug)]
pub struct Entry {
    pub ts: ClockTime,
    pub pid: u32,
    pub thread: String,
    pub level: DebugLevel,
    pub category: String,
    pub file: String,
    pub line: u32,
    pub function: String,
    pub message: String,
    pub object: Option<String>,
}

fn parse_debug_level(s: &str) -> Result<DebugLevel, ParsingError> {
    match s {
        "ERROR" => Ok(DebugLevel::Error),
        "WARN" => Ok(DebugLevel::Warning),
        "FIXME" => Ok(DebugLevel::Fixme),
        "INFO" => Ok(DebugLevel::Info),
        "DEBUG" => Ok(DebugLevel::Debug),
        "LOG" => Ok(DebugLevel::Log),
        "TRACE" => Ok(DebugLevel::Trace),
        "MEMDUMP" => Ok(DebugLevel::Memdump),
        _ => Err(ParsingError::InvalidDebugLevel {
            name: s.to_string(),
        }),
    }
}

fn parse_time(ts: &str) -> Result<ClockTime, ParsingError> {
    let mut split = ts.splitn(3, ':');
    let h: u64 = split
        .next()
        .ok_or(ParsingError::MissingToken {
            t: Token::Timestamp {
                field: Some(TimestampField::Hour),
            },
        })?
        .parse()
        .map_err(|_e| ParsingError::InvalidTimestamp {
            ts: ts.to_string(),
            field: TimestampField::Hour,
        })?;

    let m: u64 = split
        .next()
        .ok_or(ParsingError::MissingToken {
            t: Token::Timestamp {
                field: Some(TimestampField::Minute),
            },
        })?
        .parse()
        .map_err(|_e| ParsingError::InvalidTimestamp {
            ts: ts.to_string(),
            field: TimestampField::Minute,
        })?;

    split = split
        .next()
        .ok_or(ParsingError::MissingToken {
            t: Token::Timestamp {
                field: Some(TimestampField::Second),
            },
        })?
        .splitn(2, '.');
    let secs: u64 = split
        .next()
        .ok_or(ParsingError::MissingToken {
            t: Token::Timestamp {
                field: Some(TimestampField::Second),
            },
        })?
        .parse()
        .map_err(|_e| ParsingError::InvalidTimestamp {
            ts: ts.to_string(),
            field: TimestampField::Second,
        })?;

    let subsecs: u64 = split
        .next()
        .ok_or(ParsingError::MissingToken {
            t: Token::Timestamp {
                field: Some(TimestampField::SubSecond),
            },
        })?
        .parse()
        .map_err(|_e| ParsingError::InvalidTimestamp {
            ts: ts.to_string(),
            field: TimestampField::SubSecond,
        })?;

    Ok(ClockTime::from_seconds(h * 60 * 60 + m * 60 + secs) + ClockTime::from_nseconds(subsecs))
}

fn split_location(location: &str) -> Result<(String, u32, String, Option<String>), ParsingError> {
    let mut split = location.splitn(4, ':');
    let file = split
        .next()
        .ok_or(ParsingError::MissingToken { t: Token::File })?;
    let line_str = split.next().ok_or(ParsingError::MissingToken {
        t: Token::LineNumber,
    })?;
    let line = line_str
        .parse()
        .map_err(|_e| ParsingError::InvalidLineNumber {
            line: line_str.to_string(),
        })?;

    let function = split
        .next()
        .ok_or(ParsingError::MissingToken { t: Token::Function })?;

    let object = split
        .next()
        .ok_or(ParsingError::MissingToken { t: Token::Object })?;

    let object_name = {
        if !object.is_empty() {
            let object = object
                .to_string()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string();

            Some(object)
        } else {
            None
        }
    };

    Ok((file.to_string(), line, function.to_string(), object_name))
}

impl Entry {
    fn new(line: &str) -> Result<Entry, ParsingError> {
        // Strip color codes
        lazy_static! {
            static ref RE: Regex = Regex::new("\x1b\\[[0-9;]*m").unwrap();
        }
        let line = RE.replace_all(line, "");

        let mut it = line.split(' ');
        let ts_str = it.next().ok_or(ParsingError::MissingToken {
            t: Token::Timestamp { field: None },
        })?;
        let ts = parse_time(ts_str)?;

        let mut it = it.skip_while(|x| x.is_empty());
        let pid_str = it
            .next()
            .ok_or(ParsingError::MissingToken { t: Token::PID })?;
        let pid = pid_str.parse().map_err(|_e| ParsingError::InvalidPID {
            pid: pid_str.to_string(),
        })?;

        let mut it = it.skip_while(|x| x.is_empty());
        let thread = it
            .next()
            .ok_or(ParsingError::MissingToken { t: Token::Thread })?
            .to_string();

        let mut it = it.skip_while(|x| x.is_empty());
        let level_str = it
            .next()
            .ok_or(ParsingError::MissingToken { t: Token::Level })?;
        let level = parse_debug_level(level_str)?;

        let mut it = it.skip_while(|x| x.is_empty());
        let category = it
            .next()
            .ok_or(ParsingError::MissingToken { t: Token::Category })?
            .to_string();

        let mut it = it.skip_while(|x| x.is_empty());
        let location_str = it.next().ok_or(ParsingError::MissingLocation)?;
        let (file, line, function, object) = split_location(location_str)?;
        let message: String = join(it, " ");

        Ok(Entry {
            ts,
            pid,
            thread,
            level,
            category,
            file,
            line,
            function,
            object,
            message,
        })
    }

    pub fn message_to_struct(&self) -> Option<Structure> {
        Structure::from_str(&self.message).ok()
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}  {} {} {:?} {} {}:{}:{}:<{}> {}",
            self.ts,
            self.pid,
            self.thread,
            self.level,
            self.category,
            self.file,
            self.line,
            self.function,
            self.object.clone().unwrap_or_else(|| "".to_string()),
            self.message
        )
    }
}

pub struct ParserIterator<R: Read> {
    lines: Lines<BufReader<R>>,
}

impl<R: Read> ParserIterator<R> {
    fn new(lines: Lines<BufReader<R>>) -> Self {
        Self { lines }
    }
}

impl<R: Read> Iterator for ParserIterator<R> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        match self.lines.next() {
            None => None,
            Some(line) => match Entry::new(&line.unwrap()) {
                Ok(entry) => Some(entry),
                Err(_err) => self.next(),
            },
        }
    }
}

/// Parse GStreamer log entries from a reader
pub fn parse<R: Read>(r: R) -> ParserIterator<R> {
    // We don't initialize gstreamer here as it's done in main.rs
    // and we don't want to initialize it multiple times

    let file = BufReader::new(r);

    ParserIterator::new(file.lines())
}
