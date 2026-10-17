//! `--capture`: render offscreen at an exact size and scale, save a screenshot, exit.
//!
//! Capture mode has no window: the camera renders into an image, so the result does not depend on
//! the display, the window manager, or whether the screen is locked — locally or in CI.

use crate::cli::Args;
use crate::viewport::Viewport;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::path::Path;

/// The offscreen image the camera renders into in capture mode.
#[derive(Resource, Debug, Clone)]
pub struct CaptureTarget(pub Handle<Image>);

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, create_target)
            .add_systems(Last, request_capture);
    }
}

fn create_target(mut commands: Commands, args: Res<Args>, mut images: ResMut<Assets<Image>>) {
    if args.capture.is_none() {
        return;
    }
    let mut image = Image::new_target_texture(
        args.window.x,
        args.window.y,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    commands.insert_resource(CaptureTarget(images.add(image)));
    commands.insert_resource(Viewport {
        size: args.window,
        scale: args.scale.unwrap_or(1.0),
    });
}

fn request_capture(
    mut commands: Commands,
    args: Res<Args>,
    target: Option<Res<CaptureTarget>>,
    mut frames: Local<u32>,
    mut requested: Local<bool>,
) {
    let (Some(path), Some(target)) = (args.capture.clone(), target) else {
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
    commands.spawn(Screenshot::image(target.0.clone())).observe(
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

fn save_png(image: &Image, path: &Path) -> Result<(), String> {
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

/// Capture mode never creates a window.
pub fn is_headless(args: &Args) -> bool {
    args.capture.is_some()
}
