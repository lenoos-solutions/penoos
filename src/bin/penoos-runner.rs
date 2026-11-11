/*!
 * Penoos Rust Runner CLI
 * Copyright (c) 2024 Penoos Rust Port
 * MIT Licensed
 */

use penoos::PenoosRunner;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut runner = PenoosRunner::new();

    match runner.start(args).await {
        Ok(true) => {
            println!("🚀 Penoos broker started successfully");

            // Wait for shutdown signal
            if let Err(e) = runner.wait_for_shutdown().await {
                eprintln!("❌ Error during shutdown: {}", e);
                process::exit(1);
            }
        }
        Ok(false) => {
            // Help or version was shown, exit normally
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Failed to start Penoos runner: {}", e);
            process::exit(1);
        }
    }
}
