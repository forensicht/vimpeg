use std::path::Path;

use super::{decoder, search};

pub async fn search_videos<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<search::Video>> {
    search::search_videos(path).await
}

pub async fn get_video_thumbnail<P: AsRef<Path>>(
    video_path: P,
) -> anyhow::Result<decoder::VideoThumb> {
    let video_path = video_path.as_ref().to_owned();

    tokio::task::spawn_blocking(move || decoder::get_thumbnail(video_path)).await?
}

pub async fn dump_video_frames_into_image<P: AsRef<Path>>(
    video_path: P,
    image_path: P,
    cols: usize,
    rows: usize,
) -> anyhow::Result<()> {
    let video_path = video_path.as_ref().to_owned();
    let image_path = image_path.as_ref().to_owned();

    tokio::task::spawn_blocking(move || {
        decoder::dump_video_frames_into_image(video_path, image_path, cols, rows)
    })
    .await?
}