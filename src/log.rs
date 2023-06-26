use crate::errores::NodoBitcoinError;
use chrono::DateTime;
use chrono::Local;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub enum LogMessages {
    Info(String),
    Error(String),
}

struct LoggerActor {
    log_file: Option<File>,
}

impl LoggerActor {
    fn handle_message(&mut self, message: LogMessages) {
        let current_time: DateTime<Local> = Local::now();
        let formatted_time = current_time.format("%Y-%m-%d %H:%M:%S");
        match message {
            LogMessages::Info(comment) => {
                if let Some(file) = &mut self.log_file {
                    if let Err(err) = writeln!(file, "{} - Info: {}", formatted_time, comment) {
                        eprintln!("Error writing to log file: {}", err);
                    }
                } else {
                    println!("{} - Info: {}", formatted_time, comment);
                }
            }
            LogMessages::Error(error_msg) => {
                if let Some(file) = &mut self.log_file {
                    if let Err(err) = writeln!(file, "{} - Error: {}", formatted_time, error_msg) {
                        eprintln!("Error writing to log file: {}", err);
                    }
                } else {
                    println!("{} - Error: {}", formatted_time, error_msg);
                }
            }
        }
    }
}

pub fn create_logger_actor(log_file_path: Result<String, NodoBitcoinError>) -> Sender<LogMessages> {
    let (sender, receiver) = channel();

    let log_file = match log_file_path {
        Ok(path) => match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => Some(file),
            Err(err) => {
                eprintln!("Error opening log file: {}", err);
                None
            }
        },
        _ => None,
    };

    let actor = Arc::new(Mutex::new(LoggerActor { log_file }));

    thread::spawn(move || {
        let actor = actor;
        while let Ok(message) = receiver.recv() {
            match actor.lock(){
                Ok(mut log_actor) => log_actor.handle_message(message),
                Err(_) => continue,
            };
        }
    });

    sender
}

pub fn log_info_message(logger: Sender<LogMessages>, log_msg: String) -> bool {
    logger.send(LogMessages::Info(log_msg)).is_ok()
}

pub fn log_error_message(logger: Sender<LogMessages>, log_msg: String) -> bool {
    logger.send(LogMessages::Error(log_msg)).is_ok()
}
