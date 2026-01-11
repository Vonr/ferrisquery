use std::path::{Path, PathBuf};
use std::time::SystemTime;

use poise::serenity_prelude::CreateAttachment;
use poise::CreateReply;
use tokio::fs;

use crate::{Context, Error};

/// Upload the latest crash report.
#[poise::command(slash_command, global_cooldown = 30)]
pub async fn crash(ctx: Context<'_>) -> Result<(), Error> {
    let mut path = PathBuf::from(&*ctx.data().server_directory);
    path.push("crash-reports");

    let (file, created) = get_latest_file(path).await?;
    let created = created
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    ctx.send(
        CreateReply::default()
            .attachment(CreateAttachment::path(&file).await?)
            .content(format!("This file was created <t:{created}:R>.",)),
    )
    .await?;

    Ok(())
}

async fn get_latest_file(directory: impl AsRef<Path>) -> Result<(PathBuf, SystemTime), Error> {
    let mut read_dir = fs::read_dir(directory).await?;
    let mut latest: Option<(PathBuf, SystemTime)> = None;

    while let Some(dir_entry) = read_dir.next_entry().await? {
        let Ok(metadata) = dir_entry.metadata().await else {
            eprintln!("Could not get metadata of {dir_entry:?}");
            continue;
        };

        if !metadata.is_file() {
            continue;
        }

        let Ok(created) = metadata.created() else {
            eprintln!("Could not get creation date of {dir_entry:?}");
            continue;
        };

        if latest.as_ref().is_none_or(|(_, max)| *max < created) {
            latest = Some((dir_entry.path(), created));
        }
    }

    latest.ok_or_else(|| "No files available".into())
}
