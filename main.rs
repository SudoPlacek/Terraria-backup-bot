use std::{fs, env, time::Duration};
use std::path::{Path, PathBuf};
use tokio::time;
use serenity::model::id::ChannelId;
use serenity::builder::{CreateAttachment, CreateMessage};

const TERRARIA_PATH: &str = "/home/placek/.local/share/Terraria/Worlds/"; //path to world
const BACKUP_PATH: &str = "./backups/";

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set"); //token discord bot
    let http = serenity::http::Http::new(&token);

    let channel_id_val: u64 = env::var("DISCORD_CHANNEL_ID")
        .expect("DISCORD_CHANNEL_ID not set")
        .parse()
        .expect("DISCORD_CHANNEL_ID must be a number");

    let http_ref = &http;

    fs::create_dir_all(BACKUP_PATH).unwrap();

    


    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        backup_worlds();
        if let Some(file_path) = last_backup() {
            let channel_id = ChannelId::new(channel_id_val);
            match tokio::fs::read(&file_path).await {
                Ok(bytes) => {
                    let filename = file_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("backup.sav")
                        .to_string();

                    let attachment = CreateAttachment::bytes(bytes, filename.clone());

                    let message = CreateMessage::default()
                        .content(format!("new backup Terraria: {}", filename))
                        .add_file(attachment);

                    if let Err(e) = channel_id.send_message(http_ref, message).await {
                        eprintln!("Failed to send backup to Discord: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to read backup file {}: {}", file_path.display(), e),
            }
        }
    }
}

fn backup_worlds() {
    let entries = fs::read_dir(TERRARIA_PATH).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap();
            let dest = Path::new(BACKUP_PATH)
                .join(format!("{}_backup", file_name.to_string_lossy()));
            fs::copy(&path, &dest).unwrap_or_else(|e| {
                eprintln!("Failed to backup {}: {}", file_name.to_string_lossy(), e);
                0
            });
        }
    }

    println!("Backup complete!");
}


fn last_backup() -> Option<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(BACKUP_PATH)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();

    entries.sort();
    entries.pop()
}
