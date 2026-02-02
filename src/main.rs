use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    process::Command,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

#[derive(Debug, Deserialize)]
struct Config {
    watchers: Watchers,
}

#[derive(Debug, Deserialize)]
struct Watchers {
    item: Vec<WatcherItem>,
}

#[derive(Debug, Deserialize)]
struct WatcherItem {
    paths: Vec<String>,
    services: Vec<String>,
}

fn restart_service(service: &str) {
    let status = Command::new("systemctl")
        .arg("restart")
        .arg(service)
        .status();

    match status {
        Ok(s) => {
            if !s.success() {
                eprintln!("failed to restart {}: exit code {:?}", service, s.code());
            }
        }
        Err(e) => eprintln!("failed to execute systemctl for {}: {}", service, e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration
    let config_path = "/etc/sentinel.toml";
    let config_content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
    let config: Config = toml::from_str(&config_content)?;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;

    // Register watches
    for item in &config.watchers.item {
        for path in &item.paths {
            watcher.watch(Path::new(path), RecursiveMode::Recursive)?;
        }
    }

    // Rate limiter: Service Name -> Last Restart Time
    let mut last_restart: HashMap<String, Instant> = HashMap::new();

    println!("Sentinel started. Watching configured paths...");

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                let changed_paths = event.paths;
                for item in &config.watchers.item {
                    // Check if any changed path pertains to this item
                    let is_match = changed_paths.iter().any(|changed_path| {
                        item.paths
                            .iter()
                            .any(|watched_path| changed_path.starts_with(watched_path))
                    });

                    if is_match {
                        for service in &item.services {
                            let now = Instant::now();
                            let should_restart = match last_restart.get(service) {
                                Some(last) => now.duration_since(*last) > Duration::from_secs(2),
                                None => true,
                            };

                            if should_restart {
                                println!("Restarting service: {}", service);
                                restart_service(service);
                                last_restart.insert(service.clone(), now);
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("watch error: {:?}", e),
            Err(e) => return Err(e.into()),
        }
    }
}
