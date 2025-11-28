use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flate2::write::GzEncoder;
use flate2::Compression;
use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;

const FILES_TO_BACKUP: &[&str] = &[
    "/root/settings.yaml",
    "/root/client_secrets.json",
    "/root/auto-tune-presets",
    "/root/.api-report.json",
    "/root/.ssh",
    "/root/.bashrc",
    "/root/.profile",
    "/home/pi/handshakes",
    "/root/peers",
    "/etc/pwnagotchi/",
    "/usr/local/share/pwnagotchi/custom-plugins",
    "/etc/ssh/",
    "/home/pi/.bashrc",
    "/home/pi/.profile",
    "/home/pi/.wpa_sec_uploads",
];

#[derive(Parser)]
#[command(name = "pwn-backup")]
#[command(version = "2.8.9")]
#[command(about = "Backup and restore tool for Pwnagotchi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Backup files from remote device
    Backup {
        /// Hostname or IP of remote device
        #[arg(short = 'n', long, default_value = "10.0.0.2")]
        host: String,

        /// Username for SSH
        #[arg(short = 'u', long, default_value = "pi")]
        user: String,

        /// Output archive path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Restore files to remote device
    Restore {
        /// Hostname or IP of remote device
        #[arg(short = 'n', long, default_value = "10.0.0.2")]
        host: String,

        /// Username for SSH
        #[arg(short = 'u', long, default_value = "pi")]
        user: String,

        /// Backup archive to restore
        #[arg(short = 'b', long)]
        backup: Option<PathBuf>,
    },
}

fn check_connectivity(host: &str) -> Result<()> {
    println!("@ Checking connectivity to {} ...", host);

    let output = Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg(host)
        .output()
        .context("Failed to execute ping")?;

    if !output.status.success() {
        anyhow::bail!(
            "@ unit {} can't be reached, check network or USB interface IP.",
            host
        );
    }

    Ok(())
}

fn connect_ssh(host: &str, user: &str) -> Result<Session> {
    let tcp =
        TcpStream::connect(format!("{}:22", host)).context("Failed to connect to SSH server")?;

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Try to authenticate with SSH agent first
    sess.userauth_agent(user)
        .context("SSH authentication failed. Make sure your SSH key is added.")?;

    if !sess.authenticated() {
        anyhow::bail!("SSH authentication failed");
    }

    Ok(sess)
}

fn do_backup(host: &str, user: &str, output: Option<PathBuf>) -> Result<()> {
    check_connectivity(host)?;

    let output_path = output.unwrap_or_else(|| {
        let timestamp = chrono::Utc::now().timestamp();
        PathBuf::from(format!("{}-backup-{}.tgz", host, timestamp))
    });

    println!("@ Backing up {} to {:?} ...", host, output_path);

    let sess = connect_ssh(host, user)?;

    // Build tar command
    let files_str = FILES_TO_BACKUP.join(" ");
    let tar_cmd = format!("sudo tar -cf - {}", files_str);

    let mut channel = sess.channel_session()?;
    channel.exec(&tar_cmd)?;

    // Read tar output and compress
    let file = File::create(&output_path).context("Failed to create output file")?;
    let mut encoder = GzEncoder::new(file, Compression::best());

    let mut buffer = [0u8; 8192];
    loop {
        let n = channel.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        encoder.write_all(&buffer[..n])?;
    }

    encoder.finish()?;
    channel.wait_close()?;

    println!("@ Backup finished. Archive: {:?}", output_path);
    Ok(())
}

fn find_latest_backup(host: &str) -> Option<PathBuf> {
    let pattern = format!("{}-backup-*.tgz", host);

    // Find all matching files
    let mut backups: Vec<PathBuf> = std::fs::read_dir(".")
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.to_str()
                .map(|s| s.contains(&format!("{}-backup-", host)) && s.ends_with(".tgz"))
                .unwrap_or(false)
        })
        .collect();

    backups.sort();
    backups.pop()
}

fn do_restore(host: &str, user: &str, backup: Option<PathBuf>) -> Result<()> {
    let backup_file = if let Some(path) = backup {
        path
    } else {
        let found = find_latest_backup(host)
            .context("Can't find backup file. Please specify one with '-b'.")?;

        println!("@ Found backup file: {:?}", found);
        print!("@ Continue restoring this file? (y/n) ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            anyhow::bail!("Restore cancelled by user");
        }

        found
    };

    check_connectivity(host)?;

    println!("@ Restoring {:?} to {} ...", backup_file, host);

    let sess = connect_ssh(host, user)?;

    // Read backup file
    let mut backup_data = Vec::new();
    File::open(&backup_file)
        .context("Failed to open backup file")?
        .read_to_end(&mut backup_data)?;

    // Extract on remote
    let mut channel = sess.channel_session()?;
    channel.exec("sudo tar xzv -C /")?;
    channel.write_all(&backup_data)?;
    channel.send_eof()?;

    // Wait for completion
    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    println!("{}", output);

    channel.wait_close()?;

    println!("@ Restore finished.");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup { host, user, output } => {
            do_backup(&host, &user, output)?;
        }
        Commands::Restore { host, user, backup } => {
            do_restore(&host, &user, backup)?;
        }
    }

    Ok(())
}
