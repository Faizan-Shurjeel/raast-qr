use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "raast-qr")]
#[command(about = "CLI tool to inspect, generate, and verify Raast & EMVCo QR payloads", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify an EMVCo / Raast QR string
    Verify { payload: String },
}

fn main() {
    println!("raast-qr engine initialized.");
}
