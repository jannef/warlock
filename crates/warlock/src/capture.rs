//! `--capture`: render a few frames, save a screenshot, exit.

use crate::cli::Args;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::path::PathBuf;

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, request_capture);
    }
}

fn request_capture(
    mut commands: Commands,
    args: Res<Args>,
    mut frames: Local<u32>,
    mut requested: Local<bool>,
) {
    let Some(path) = args.capture.clone() else {
        return;
    };
    if *requested {
        return;
    }
    *frames += 1;
    if *frames < args.capture_after_frames {
        return;
    }
    *requested = true;
    info!("capturing screenshot to {}", path.display());
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| match save_png(
            &captured.image,
            &path,
        ) {
            Ok(()) => {
                info!("saved {}", path.display());
                exit.write(AppExit::Success);
            }
            Err(err) => {
                error!("failed to save screenshot: {err}");
                exit.write(AppExit::error());
            }
        },
    );
}

fn save_png(image: &Image, path: &PathBuf) -> Result<(), String> {
    let dynamic = image
        .clone()
        .try_into_dynamic()
        .map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    dynamic
        .to_rgb8()
        .save_with_format(path, image::ImageFormat::Png)
        .map_err(|e| e.to_string())
}
