// Simple validation tests
use std::process::Command;

fn main() {
    println!("Running validation checks...");
    
    // Check if cargo check passes
    let output = Command::new("cargo")
        .args(&["check"])
        .output()
        .expect("Failed to execute cargo check");
        
    if output.status.success() {
        println!("✅ Cargo check passed");
    } else {
        println!("❌ Cargo check failed");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Check if cargo build passes
    let output = Command::new("cargo")
        .args(&["build"])
        .output()
        .expect("Failed to execute cargo build");
        
    if output.status.success() {
        println!("✅ Cargo build passed");
    } else {
        println!("❌ Cargo build failed");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
