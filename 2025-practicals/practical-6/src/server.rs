//! A Secure, multi-threaded HTTP/1.1 server implementation using TLS for secure communication.
//! This server demonstrates concurrency management, secure password storage and itergration with
//! external services like PostgreSQL and Redis.
use colored::Colorize;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

pub struct State {}

/// Prints server information on startup.
///
/// # Arguments
/// - `port`: The port the server is currently running on.
fn print_server_info(port: u16) {
    println!("{}", "Server started:".cyan());

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "address: ".cyan(),
        "127.0.0.1".red().bold()
    );

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "port: ".cyan(),
        port.to_string().red().bold()
    );

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "HTTP/1.1: ".cyan(),
        "true".red().bold()
    );

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "shutdown: ".cyan(),
        "ctrl C".red().bold()
    );

    println!(
        "{}{}\n",
        "Server has launched from http://127.0.0.1:".red().bold(),
        port.to_string().red().bold()
    );
}
