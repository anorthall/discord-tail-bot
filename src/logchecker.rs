use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::{fs::File, io};

use regex::Regex;

use serenity::model::id::ChannelId;
use serenity::prelude::*;

use crate::configuration::Settings;

pub async fn do_log_check(
    settings: Arc<Settings>,
    ctx: Arc<Context>,
    last_position: Arc<Mutex<u64>>,
) {
    log::info!("Checking log file '{}'.", settings.log_file);
    let buffer = match seek_to_position(&settings.log_file, last_position).await {
        Ok(buffer) => buffer,
        Err(e) => {
            log::error!(
                "Unable to seek to position in file '{}': {}",
                settings.log_file,
                e
            );
            return;
        }
    };

    if buffer.trim().is_empty() {
        log::info!("Log file checked - no new messages to send.");
        return;
    } else {
        log::info!("Log file checked - sending messages to channel.");
        log::info!("Buffer: {}", buffer);
    }

    notify_channel(ctx, settings, buffer).await;
}

async fn seek_to_position(
    file_path: &str,
    last_position: Arc<Mutex<u64>>,
) -> Result<String, io::Error> {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    // Lock the seek position from the Mutex
    let mut seek = match last_position.lock() {
        Ok(seek) => seek,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to lock the seek position.",
            ));
        }
    };

    log::info!("Seek position: {}.", seek);

    // If no seek position is set, set it to the end of the file and return
    // a blank string to avoid sending the whole file on startup
    if *seek == 0 {
        *seek = file.seek(SeekFrom::End(0))?;
        return Ok(String::new());
    } else {
        file.seek(SeekFrom::Start(*seek - 1))?;
    }

    // Read the file contents
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    // Set the saved seek position to the current position within the file
    *seek = file.stream_position()?;

    Ok(buffer)
}

async fn notify_channel(ctx: Arc<Context>, settings: Arc<Settings>, buffer: String) {
    log::info!(
        "Sending messages to channel with ID {}.",
        settings.channel_id
    );
    let channel = ChannelId(settings.channel_id);
    let mut successful_messages = 0;
    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }

        match format_log_line(line, &settings.pattern) {
            Some(line) => {
                match channel.send_message(&ctx.http, |m| m.content(line)).await {
                    Ok(_) => successful_messages += 1,
                    Err(e) => {
                        log::error!("Unable to send message to channel: {}", e);
                        return;
                    }
                };
            }
            None => continue,
        };
    }

    log::info!(
        "Sent {} messages to channel from the log file.",
        successful_messages
    );
}

fn format_log_line(line: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).unwrap();
    let caps = match re.captures(line) {
        Some(caps) => caps,
        None => return None,
    };

    Some(String::from(caps.get(1).unwrap().as_str()))
}

mod tests {
    #[test]
    fn test_format_log_line_with_valid_input() {
        let line = "[INFO] [2020-04-19 20:00:00,000] Line content.";
        let pattern = r"^\[INFO] \[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3}] (.*)$";
        let expected = String::from("Line content.");
        let actual = super::format_log_line(line, pattern).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_format_log_line_with_invalid_input() {
        let line = "Invalid content.";
        let pattern = r"^\[INFO] \[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3}] (.*)$";
        let actual = super::format_log_line(line, pattern);
        assert!(actual.is_none());
    }
}
