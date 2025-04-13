use chrono::Local;
use clap::Parser;
use colored::Colorize;
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use practical_6::mail;
use std::process;

/// CLI definition
#[derive(Parser, Debug)]
#[command(name = "Alarm System")]
#[command(about = "Simulates an alarm system that sends emails on keypress", long_about = None)]
struct Args {
    /// Email address to send alerts to
    #[arg(short, long)]
    target_email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    log::info!(target: "request_logger","CLI Application Started");

    let args = Args::parse();

    println!(
        "{}. {}:\n{} {}\n{} {}\n{} {}\n{} {}\n{} {}\n{} {}",
        "🚨 Alarm system running".red(),
        "Press keys to simulate sensors",
        ">> ".red().bold(),
        "M: Motion ".cyan(),
        ">> ".red().bold(),
        "D: Door ".cyan(),
        ">> ".red().bold(),
        "W: Window ".cyan(),
        ">> ".red().bold(),
        "F: Fire ".cyan(),
        ">> ".red().bold(),
        "G: Garage ".cyan(),
        ">> ".red().bold(),
        "Q: Quit ".cyan(),
    );

    println!("🚨 Alarm system running. Press keys to simulate sensors:");
    println!("  M = Motion");
    println!("  D = Door");
    println!("  W = Window");
    println!("  F = Fire");
    println!("  G = Garage");
    println!("  Q = Quit");

    loop {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        match read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            })) => match c.to_ascii_uppercase() {
                'M' => {
                    let body: String = format!("Motion detected at {timestamp}.");
                    println!(
                        "{}{} {}",
                        ">> ".red().bold(),
                        "Alert: ".cyan(),
                        "Motion Detected".red().bold()
                    );

                    match mail::send_mail(
                        "Motion Detected".to_string(),
                        body,
                        args.target_email.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error Occured when attempting to send email: {e}");
                        }
                    }
                }
                'D' => {
                    let body: String = format!("The front door was opened at {timestamp}.");
                    println!(
                        "{}{} {}",
                        ">> ".red().bold(),
                        "Alert: ".cyan(),
                        "Door Access Detected".red().bold()
                    );

                    match mail::send_mail(
                        "Door Opened".to_string(),
                        body,
                        args.target_email.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error Occured when attempting to send email: {e}");
                        }
                    }
                }
                'W' => {
                    let body: String = format!("The window was opened at {timestamp}.");
                    println!(
                        "{}{} {}",
                        ">> ".red().bold(),
                        "Alert: ".cyan(),
                        "Window Opened".red().bold()
                    );

                    match mail::send_mail(
                        "Window Opened".to_string(),
                        body,
                        args.target_email.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error Occured when attempting to send email: {e}");
                        }
                    }
                }
                'F' => {
                    let body: String = format!("Fire alarm triggered at {timestamp}.");
                    println!(
                        "{}{} {}",
                        ">> ".red().bold(),
                        "Alert: ".cyan(),
                        "Fire Alarm Triggered".red().bold()
                    );

                    match mail::send_mail(
                        "Fire Alarm Triggered".to_string(),
                        body,
                        args.target_email.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error Occured when attempting to send email: {e}");
                        }
                    }
                }
                'G' => {
                    let body: String = format!("Garage accessed at {timestamp}.");
                    println!(
                        "{}{} {}",
                        ">> ".red().bold(),
                        "Alert: ".cyan(),
                        "Garage Access Detected".red().bold()
                    );

                    match mail::send_mail(
                        "Garage Access".to_string(),
                        body,
                        args.target_email.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error Occured when attempting to send email: {e}");
                        }
                    }
                }
                'Q' => {
                    println!(
                        "{} {}",
                        ">> ".red().bold(),
                        "Exiting Alarm System".red().bold()
                    );
                    process::exit(0);
                }
                _ => {
                    let body: String = format!("⚠️  Unknown sensor triggerd at {timestamp}.");
                    println!(
                        "{}{} {}",
                        ">> ".red().bold(),
                        "Alert: ".cyan(),
                        "Unknown Sensor Triggered".red().bold()
                    );

                    match mail::send_mail(
                        "Alrm System Event".to_string(),
                        body,
                        args.target_email.clone(),
                    )
                    .await
                    {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error Occured when attempting to send email: {e}");
                        }
                    }
                }
            },
            _ => {}
        }
        println!(
            "{}{} {}",
            ">> ".red().bold(),
            "Email Sent: ".cyan(),
            "Success".red().bold()
        );
    }
}
