//! This application's own icon, as pixels wxWidgets will take.
//!
//! The icon lives in `assets/icon.ico` and is built into the executable as a
//! Windows resource by `build.rs`, which is what Explorer and the taskbar show.
//! The notification area is asked for something else: wxWidgets wants a bitmap,
//! which is raw pixels, and an `.ico` is a container holding one or more PNG or
//! BMP images. So it has to be decoded.
//!
//! # Why it is built in rather than read from disk
//!
//! Reading it from beside the executable would mean an icon that goes missing
//! when somebody moves the program, and a path to work out at run time on a
//! machine where the working directory is anybody's guess. `include_bytes!`
//! settles it at compile time: the pixels are in the binary or the build fails,
//! and there is no third answer where the program runs without its own face.
//!
//! # Which size
//!
//! An `.ico` holds several. The largest is decoded and Windows shrinks it to
//! the 16 by 16 the notification area draws at, which is the right direction to
//! be wrong in: shrinking a drawing keeps its edges where enlarging one softens
//! them.

use crate::common::{Error, Result};
use wxdragon::bitmap::Bitmap;

/// The icon, exactly as it ships.
const ICON: &[u8] = include_bytes!("../../assets/icon.ico");

/// This application's icon as a bitmap, ready for the notification area.
pub fn tray_picture() -> Result<Bitmap> {
    let (pixels, width, height) = icon_pixels(ICON)?;
    Bitmap::from_rgba(&pixels, width, height).ok_or_else(|| {
        Error::Other("The application icon could not be turned into a bitmap".to_string())
    })
}

/// Decode an `.ico` to RGBA pixels.
///
/// Kept apart from the wxWidgets call so the decoding can be tested. Building a
/// bitmap needs a running application and this does not.
///
/// The decoder picks the largest image in the file, which is not the size the
/// notification area draws at. That is the right direction to be wrong in:
/// Windows shrinks the picture to fit, and shrinking a drawing keeps its edges
/// where enlarging one would soften them. Choosing a particular entry would
/// mean reading the directory by hand, which is more code than the difference
/// is worth.
fn icon_pixels(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Ico)
        .map_err(|why| Error::Other(format!("The application icon could not be read: {why}")))?;
    let rgba = decoded.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    Ok((rgba.into_raw(), width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_icon_this_application_ships_can_be_read() {
        // The file is built into the binary, so this cannot fail for a missing
        // file. What it can catch is an icon replaced with something the
        // decoder will not take, which would otherwise show up as a program
        // that runs with no picture in the notification area and no reason
        // given.
        let read = icon_pixels(ICON);

        assert!(read.is_ok(), "{read:?}");
    }

    #[test]
    fn test_the_icon_has_four_bytes_for_every_pixel() {
        // wxWidgets reads the buffer as RGBA and is told only the width and
        // height. A buffer that does not match those is read past its end or
        // drawn as noise, and neither says what went wrong.
        let (pixels, width, height) = icon_pixels(ICON).expect("the shipped icon");

        assert_eq!(
            pixels.len() as u32,
            width * height * 4,
            "{width} by {height} does not describe {} bytes",
            pixels.len()
        );
    }

    #[test]
    fn test_something_that_is_not_an_icon_is_refused_rather_than_drawn() {
        // The decoder is reachable only from the file this project ships, so
        // this is not a security boundary. It is a check that a failure comes
        // back as an error somebody can read rather than as a panic or a
        // picture of nothing.
        let refused = icon_pixels(b"this is not an icon at all");

        assert!(refused.is_err());
    }
}
