use std::process::{Command, Child};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

fn start_server() -> Child {
    println!("Starting Minecraft server...");

    Command::new("java")
        .args([
            "-Xmx1024M",
            "-jar",
            "server.jar",
            "nogui",
        ])
        .spawn()
        .expect("Failed to start Java server")
}

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nShutdown signal received.");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut server = start_server();

    while running.load(Ordering::SeqCst) {
        match server.try_wait() {
            Ok(Some(status)) => {
                println!("Server exited with {} — restarting in 5s...", status);
                thread::sleep(Duration::from_secs(5));
                server = start_server();
            }
            Ok(None) => {
                thread::sleep(Duration::from_secs(2));
            }
            Err(e) => {
                eprintln!("Error checking server status: {}", e);
                break;
            }
        }
    }

    println!("Stopping server...");
    let _ = server.kill();
}


