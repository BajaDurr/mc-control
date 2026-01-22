use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

fn start_server() -> Child {
    println!("Starting Minecraft server...");

    Command::new("java")
        .args(["-Xmx1024M", "-jar", "server.jar", "nogui"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Minecraft server")
}

fn send_cmd(server: &mut Child, cmd: &str) {
    if let Some(stdin) = server.stdin.as_mut() {
        let _ = stdin.write_all(cmd.as_bytes());
        let _ = stdin.write_all(b"\n");
        let _ = stdin.flush();
    }
}

fn say(server: &mut Child, msg: &str) {
    // Broadcast message to all players
    // You can add formatting codes here if you want (e.g., §c for red)
    send_cmd(server, &format!("say {}", msg));
}

fn restart_countdown_messages() -> Vec<(Duration, &'static str)> {
    // These are offsets BEFORE restart time.
    // We'll send a message when remaining time <= offset.
    vec![
        (Duration::from_secs(5 * 60), "Server restarting in 5 minutes!"),
        (Duration::from_secs(60), "Server restarting in 1 minute!"),
        (Duration::from_secs(30), "Server restarting in 30 seconds!"),
        (Duration::from_secs(5), "Restarting in 5..."),
        (Duration::from_secs(4), "4..."),
        (Duration::from_secs(3), "3..."),
        (Duration::from_secs(2), "2..."),
        (Duration::from_secs(1), "1..."),
        (Duration::from_secs(0), "Rebooting!"),
    ]
}

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nShutdown signal received.");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let restart_interval = Duration::from_secs(2 * 60 * 60); // 2 hours
    let mut server = start_server();

    // When the next scheduled restart should happen
    let mut next_restart = Instant::now() + restart_interval;

    // Warning schedule state
    let warnings = restart_countdown_messages();
    let mut next_warning_index: usize = 0;

    // Main loop tick
    let tick = Duration::from_secs(1);

    while running.load(Ordering::SeqCst) {
        // Crash detection: if server exits unexpectedly, restart it.
        match server.try_wait() {
            Ok(Some(status)) => {
                println!("Server exited with {} — restarting in 5s...", status);
                thread::sleep(Duration::from_secs(5));
                server = start_server();
                next_restart = Instant::now() + restart_interval;
                next_warning_index = 0;
                continue;
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error checking server status: {}", e);
                break;
            }
        }

        let now = Instant::now();

        // Send countdown warnings as we approach the scheduled restart.
        if now < next_restart {
            let remaining = next_restart - now;

            // Walk forward through warnings in order once their threshold is reached.
            while next_warning_index < warnings.len() {
                let (threshold, msg) = warnings[next_warning_index];

                // If remaining time is less than or equal to threshold, send it.
                if remaining <= threshold {
                    say(&mut server, msg);
                    next_warning_index += 1;
                } else {
                    break;
                }
            }
        }

        // Time to restart
        if now >= next_restart {
            println!("Scheduled restart time reached — stopping server...");

            // Ask server to stop cleanly
            send_cmd(&mut server, "stop");

            // Give it a moment to save & exit cleanly
            let mut waited = 0u64;
            let graceful_wait = 20u64;
            while waited < graceful_wait {
                match server.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        thread::sleep(Duration::from_secs(1));
                        waited += 1;
                    }
                    Err(_) => break,
                }
            }

            // If still alive, force it down
            let _ = server.kill();
            let _ = server.wait();

            // Start back up
            server = start_server();
            next_restart = Instant::now() + restart_interval;
            next_warning_index = 0;
        }

        thread::sleep(tick);
    }

    // Graceful shutdown on Ctrl+C
    println!("Stopping server...");
    send_cmd(&mut server, "stop");
    thread::sleep(Duration::from_secs(10));
    let _ = server.kill();
}
