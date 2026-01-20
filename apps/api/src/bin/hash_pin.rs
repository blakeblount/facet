//! Utility to generate argon2 hashes for PINs.
//!
//! Usage: cargo run --bin hash_pin <pin>

use api::auth::hash_pin;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <pin>", args[0]);
        std::process::exit(1);
    }

    let pin = &args[1];
    match hash_pin(pin) {
        Ok(hash) => println!("{}", hash),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
