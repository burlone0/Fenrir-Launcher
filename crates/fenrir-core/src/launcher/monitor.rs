use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Child;
use std::time::Instant;
use tracing::{debug, error, info};

pub struct LaunchResult {
    pub exit_code: Option<i32>,
    pub play_time_secs: u64,
}

/// Monitor a game process: log output, track playtime.
pub fn monitor_process(mut child: Child, log_path: &Path) -> LaunchResult {
    let start = Instant::now();

    // Log stderr in a background thread
    if let Some(stderr) = child.stderr.take() {
        let log_clone = log_path.to_path_buf();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&log_clone)
                .ok();

            for line in reader.lines().map_while(Result::ok) {
                debug!("game stderr: {}", line);
                if let Some(ref mut f) = file {
                    let _ = writeln!(f, "[stderr] {}", line);
                }
            }
        });
    }

    // Log stdout on current thread (blocks until process closes stdout)
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_path)
            .ok();

        for line in reader.lines().map_while(Result::ok) {
            debug!("game stdout: {}", line);
            if let Some(ref mut f) = file {
                let _ = writeln!(f, "[stdout] {}", line);
            }
        }
    }

    // Wait for process exit
    let exit_code = match child.wait() {
        Ok(status) => {
            info!("game exited with status: {}", status);
            status.code()
        }
        Err(e) => {
            error!("failed to wait for game process: {}", e);
            None
        }
    };

    let play_time = start.elapsed().as_secs();

    LaunchResult {
        exit_code,
        play_time_secs: play_time,
    }
}
