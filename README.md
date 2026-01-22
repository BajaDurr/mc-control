# mc-control

A lightweight Rust-based controller for running and managing a self-hosted Minecraft server.

This tool acts as a **wrapper/launcher** around `server.jar` and provides:

- ✅ Automatic crash restart
- 🔁 Scheduled automatic restarts (every 2 hours)
- ⏱️ In-game restart countdown warnings (5 min, 1 min, 30 sec, 5..4..3..2..1)
- 🧼 Clean shutdown using `stop` (prevents world corruption)
- 🖥️ Console passthrough (see full Minecraft output)
- 🔐 Designed to run headless over SSH on Linux / Raspberry Pi

---

## 📁 Folder Layout

Your project directory should look like this:

```text
mc_control/
├── Cargo.toml
├── src/
│ └── main.rs
├── server.jar # Minecraft server jar (vanilla or Fabric)
├── world/ # Minecraft world data (auto-generated)
├── versions/
│ └── 1.21.11/
│ └──── server-1.21.11.jar
├── eula.txt
├── server.properties
└── logs/
```


> ⚠️ The `server.jar` file **must** be in the same directory where you run `cargo run`.

---

## 🛠️ Requirements

- Linux (tested on Raspberry Pi / Ubuntu)
- Rust (stable)
- Java 21+ (or whatever your Minecraft version requires)

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
rustup default stable
```
Install Java:

```bash
sudo apt update
sudo apt install openjdk-21-jre -y
```

### Check:
```bash
rustc --version
cargo --version
java -version
```

## 📥 Setup
1. Go download the minecraft server jar file at [Minecraft: Java Edition Server](https://www.minecraft.net/en-us/download/server)

2. Put your Minecraft server jar in the project folder:
   ```bash
   mv minecraft_server.1.21.1.jar server.jar
   ```
3. Run it once manually to generate files:
   ```bash
   java -Xmx1024M -jar server.jar nogui
   ```
4. Accept the EULA
   ```bash
   nano eula.txt
   ```
Set:
```ini
eula=true
```
5. Stop the server.

## ▶️ Running the Controller
From the project folder:
```bash
cargo run
```

The program will:
- Start the Minecraft server
- Monitor it
- Restart it if it crashes
- Restart it automatically every 2 hours with in-game warnings

## ⏱️ Restart Schedule
By default, the server restarts every **2 hours.**

Players will see chat messages:

- 5 minutes before restart
- 1 minute before restart
- 30 seconds before restart
- Final countdown: 5,4,3,2,1
  Then the server will:
  - Send ```stop```
  - Wait for clean shutdown
  - Restart automatically

## ⚙️ Configuration
### Change RAM
In ```main.rs```:
```rust
"-Xmx1024M"
```
Change to:
```rust
"-Xmx4G"
```
or whatever your Pi/PC can handle.

### Change Restart Interval
Find:
```rust
let restart_interval = Duration::from_secs(2 * 60 * 60);
```
Examples:
```rust
// 1 hour
Duration::from_secs(60 * 60);

// 6 hours
Duration::from_secs(6 * 60 * 60);
```

## 🌍 Networking
 - Default port: 25565
 - Make sure your firewall allows it:

```bash
sudo ufw allow 25565/tcp
sudo ufw allow 25565/udp
```
- For external access, set up **port forwarding** in your router to your server's IP.

## 💾 Backups
Your world is stored in:

```text
world/
```
Back it up with:
```bash
tar -czf world_backup_$(date +%Y-%m-%d).tar.gz world
```
>❗ Always stop the server before backing up.

## 🛑 Stopping the Server
Press:
```text
Ctrl + C
```
The controller will:
- Send ```stop``` to Minecraft
- Wait
- Kill the process if needed
- Exit cleanly

## 🧠 Why This Exists
This project was built to:
- Run a reliable Minecraft server on a Raspberry Pi
- Avoid world corruption
- Automate restarts
- Provide basic "production-style" server management in a simple Rust binary

## 📜 License

Personal project. Use and modify freely.
