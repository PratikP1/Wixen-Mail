//! Decoding a picture somebody was sent, into pixels a window can draw.
//!
//! RED half. `read` answers one outcome for everything so the tests below fail
//! on the answer rather than on a build that never happened. The real decoding,
//! and the bounds that make it safe, arrive with the green commit.

use crate::common::Result;

/// The widest and the tallest a picture may say it is.
///
/// Read out of the header before anything is decoded, so a file claiming to be
/// a hundred thousand pixels across costs nothing to refuse. Ten thousand is
/// larger than any camera, any scan and any screenshot somebody attaches to
/// mail, and a file past it is either a mistake or a bomb.
///
/// A loose bound on purpose. It is not what keeps the memory in hand; that is
/// [`MOST_A_PICTURE_MAY_COST`], which is tighter over most of the range this
/// admits. This one exists to refuse the obvious case for nothing.
pub const WIDEST: u32 = 10_000;

/// The same, for the other axis.
pub const TALLEST: u32 = 10_000;

/// The most memory one picture may cost.
///
/// Forty-eight megabytes is a twelve megapixel photograph in four bytes a
/// pixel, which is what a phone sends, and it is the point past which a preview
/// stops being worth what it costs the window.
///
/// Not the same question as
/// [`crate::application::pictures::MOST_ONE_PICTURE_MAY_BE`], which is two
/// megabytes, and the two do not agree because they are not about the same
/// thing. That one bounds the compressed bytes of an inline picture kept in the
/// message cache, and is about disk. This one bounds what a decoder allocates
/// and what crosses to the window, and is about memory and about a file that is
/// small until it is decoded. A two megabyte JPEG routinely decodes to fifty.
pub const MOST_A_PICTURE_MAY_COST: u64 = 48 * 1024 * 1024;

/// A picture, decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureReading {
    /// RGBA, four bytes a pixel, `width * height * 4` bytes long.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Decode a picture into pixels.
pub fn read(bytes: &[u8]) -> Result<PictureReading> {
    let _ = bytes;
    Ok(PictureReading {
        pixels: vec![0; 4],
        width: 1,
        height: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG of `width` by `height`, made here rather than kept as a file.
    ///
    /// Four bytes a pixel, because what the decoder allocates is what the
    /// allocation bound is about and a grayscale picture of the same size costs
    /// a quarter as much to decode.
    fn a_png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(width, height))
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("a PNG this test made");
        bytes
    }

    /// A grayscale PNG, which costs a quarter of [`a_png`] to decode and the
    /// same to hand on as RGBA.
    fn a_grey_png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        image::DynamicImage::ImageLuma8(image::GrayImage::new(width, height))
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("a PNG this test made");
        bytes
    }

    #[test]
    fn test_a_real_picture_comes_back_with_its_own_size() {
        let read = read(&a_png(40, 25)).expect("a picture this test made");

        assert_eq!((read.width, read.height), (40, 25));
    }

    #[test]
    fn test_a_jpeg_is_read_here_now_that_this_build_decodes_one() {
        // The feature this task added to `image`, exercised. Without it a JPEG
        // comes back as an unsupported format, and every photograph anybody was
        // ever sent would say it could not be read, which is a sentence about a
        // fault for what is really this build own limit. A JPEG rather than
        // another PNG on purpose: the PNG path was already here.
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::new(40, 25))
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Jpeg,
            )
            .expect("a JPEG this test made");

        let read = read(&bytes).expect("a JPEG");

        assert_eq!((read.width, read.height), (40, 25));
    }

    #[test]
    fn test_a_picture_has_four_bytes_for_every_pixel() {
        // wxWidgets reads the buffer as RGBA and is told only the width and the
        // height. A buffer that does not match those is read past its end or
        // drawn as noise, and neither says what went wrong. `art.rs` carries
        // the same assertion about the icon this program ships, for the same
        // reason.
        let read = read(&a_png(40, 25)).expect("a picture this test made");

        assert_eq!(read.pixels.len() as u32, read.width * read.height * 4);
    }

    #[test]
    fn test_a_picture_declaring_more_than_the_bound_is_refused_on_its_declaration() {
        // The header's own claim, refused before a pixel is decoded. What says
        // the refusal came from there rather than from the decoder is the
        // sentence: this one names the size the file declared, and the
        // decoder's does not.
        let refused = read(&a_png(WIDEST + 1, 1));

        let why = refused
            .expect_err("a picture wider than the bound")
            .to_string();
        assert!(why.contains(&(WIDEST + 1).to_string()), "{why}");
    }

    #[test]
    fn test_a_picture_that_declares_less_than_it_costs_is_refused_while_decoding() {
        // Both axes are well under the declared-size gate, so that gate passes
        // it. What refuses it is what decoding really allocates. A declared size
        // is metadata the sender controls, which is the same reason
        // `sound_scheme_import` refuses on the declared size of a zip entry and
        // again on what decompression really produces.
        let across = 3600;
        assert!(across < WIDEST);
        assert!(u64::from(across) * u64::from(across) * 4 > MOST_A_PICTURE_MAY_COST);

        assert!(read(&a_png(across, across)).is_err());
    }

    #[test]
    fn test_a_picture_that_grows_on_the_way_to_the_window_is_refused_too() {
        // A grey picture costs a quarter as much to decode as it costs to hand
        // on, because what crosses to the window is always four bytes a pixel.
        // So the decoder's own bound can be satisfied by a picture this must
        // still refuse, and the bound is asked again of what is really handed
        // over.
        let across = 4000;
        assert!(u64::from(across) * u64::from(across) <= MOST_A_PICTURE_MAY_COST);
        assert!(u64::from(across) * u64::from(across) * 4 > MOST_A_PICTURE_MAY_COST);

        assert!(read(&a_grey_png(across, across)).is_err());
    }

    #[test]
    fn test_a_truncated_picture_is_refused_rather_than_drawn() {
        // Half a file is what arrives when a fetch is cut off, and it must come
        // back as a refusal somebody hears rather than as a panic or a blank.
        let whole = a_png(40, 25);
        let half = &whole[..whole.len() / 2];

        assert!(read(half).is_err());
    }

    #[test]
    fn test_bytes_that_are_not_a_picture_at_all_are_refused() {
        assert!(read(b"this is not a picture, whatever it was called").is_err());
    }

    #[test]
    fn test_a_file_that_is_not_the_kind_its_name_claims_is_refused() {
        // A sender can name anything `.png`. Nothing here checks the bytes
        // against the label beyond what the decoder itself rejects, and this is
        // that rejection reaching the reader as words.
        let mut pretending = b"\x89PNG\r\n\x1a\n".to_vec();
        pretending.extend_from_slice(b"and then nothing a decoder can use");

        assert!(read(&pretending).is_err());
    }

    #[test]
    fn test_nothing_at_all_is_refused() {
        assert!(read(b"").is_err());
    }
}
