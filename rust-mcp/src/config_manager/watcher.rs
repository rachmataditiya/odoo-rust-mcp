use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{error, info};

pub type ConfigChangeEvent = String; // filename that changed

#[derive(Clone)]
pub struct ConfigWatcher {
    tx: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigWatcher {
    pub fn new(config_dir: PathBuf) -> anyhow::Result<Self> {
        let (tx, _) = broadcast::channel(100);
        let tx_clone = tx.clone();

        // Spawn file watcher in background
        std::thread::spawn(move || {
            if let Err(e) = Self::start_watching(config_dir, tx_clone) {
                error!("Config watcher error: {}", e);
            }
        });

        Ok(Self { tx })
    }

    fn start_watching(
        config_dir: PathBuf,
        tx: broadcast::Sender<ConfigChangeEvent>,
    ) -> NotifyResult<()> {
        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: NotifyResult<Event>| match res {
                Ok(event) => {
                    for path in event.paths {
                        if let Some(filename) = path.file_name()
                            && let Some(name) = filename.to_str()
                            && name.ends_with(".json")
                        {
                            let _ = tx.send(name.to_string());
                            info!("Config file changed: {}", name);
                        }
                    }
                }
                Err(e) => error!("Watcher error: {}", e),
            })?;

        watcher.watch(&config_dir, RecursiveMode::NonRecursive)?;

        info!("Config watcher started for {:?}", config_dir);

        // Keep watcher alive
        std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.tx.subscribe()
    }

    pub fn notify(&self, filename: &str) {
        let _ = self.tx.send(filename.to_string());
    }
}
