//! Decoding a picture somebody was sent, into pixels a window can draw.
//!
//! This is the sharpest untrusted boundary in the reading window. An image
//! decoder over a stranger's file is a classic target, and until now the only
//! decode in this program read one file the project ships itself. So the rule
//! here is that it fails closed: every way of not producing a picture comes back
//! as an error the reader is told about in words, and none of them is a blank
//! space, a panic, or a window that stops answering while something enormous is
//! allocated. `unwrap` and `expect` appear nowhere outside the tests.
//!
//! # Why there are three bounds and not one
//!
//! A picture can be small on disk and enormous in memory, which is the whole of
//! the decompression bomb. Three questions are asked, in this order, and each
//! catches something the others do not.
//!
//! What the header says, before any pixel is decoded. That is the cheapest
//! refusal there is and it is the one that stops a file claiming to be a hundred
//! thousand pixels across. It is not enough on its own: a declared size is
//! metadata the sender controls and can misstate, which is the same reason
//! `sound_scheme_import` refuses a zip entry on its declared size and then again
//! on what decompression really produces.
//!
//! What decoding really allocates, through `image`'s own limits. That catches
//! the file whose header understates it. It is not enough on its own either, and
//! `image`'s documentation says why in as many words: `max_alloc` "is non-strict
//! by default and some decoders may ignore it", while `max_image_width` and
//! `max_image_height` are strict. So the header gate is not a cheap shortcut for
//! the allocation cap, and the allocation cap is not a replacement for the
//! header gate.
//!
//! What is really handed to the window, after decoding and after conversion.
//! That third one is not in the plan this was built from and it is here because
//! the first two do not cover it: a decoder that produces a grey picture
//! allocates one byte a pixel and this hands on four, so a picture can satisfy
//! the decoder's own bound and still be four times the size by the time it
//! crosses to the window, where it is cloned into the record of every open tab.
//!
//! # What it will not do
//!
//! It does not scale, rotate or colour-manage anything, and it does not check
//! that a file's bytes are the kind its name claims beyond what the decoder
//! itself rejects. It opens no window: bytes in, pixels out, so it can be tested
//! against real files without one, the same as `pdf.rs` and `plain_text.rs`.

use crate::common::{Error, Result};
use image::ImageReader;
use std::io::Cursor;

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

/// Decode a picture into pixels a window can draw.
///
/// Fails whenever a picture does not come out, and every failure carries a
/// sentence rather than a code, because the reader is told what it says.
pub fn read(bytes: &[u8]) -> Result<PictureReading> {
    refuse_what_the_header_declares(bytes)?;

    let mut decoding = reader_over(bytes)?;
    let mut bounds = image::Limits::default();
    // Strict, both of them, and asked again here rather than left to the header
    // gate: this is what the decoder itself enforces while it works, on the size
    // it really finds rather than the size the file claimed.
    bounds.max_image_width = Some(WIDEST);
    bounds.max_image_height = Some(TALLEST);
    bounds.max_alloc = Some(MOST_A_PICTURE_MAY_COST);
    decoding.limits(bounds);

    let decoded = decoding
        .decode()
        .map_err(|why| Error::Other(format!("the picture could not be decoded: {why}")))?;

    let (width, height) = (decoded.width(), decoded.height());
    if what_it_costs_to_hand_on(width, height) > MOST_A_PICTURE_MAY_COST {
        return Err(Error::Other(format!(
            "the picture is {width} by {height}, which is more than a preview may hold"
        )));
    }

    let rgba = decoded.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    Ok(PictureReading {
        pixels: rgba.into_raw(),
        width,
        height,
    })
}

/// Refuse a picture on the size its own header declares, before decoding.
///
/// The error names the declared size, which is what tells a reader of the tests
/// that the refusal came from here rather than from the decoder further down.
fn refuse_what_the_header_declares(bytes: &[u8]) -> Result<()> {
    let (width, height) = reader_over(bytes)?
        .into_dimensions()
        .map_err(|why| Error::Other(format!("the picture has no readable size: {why}")))?;

    if width > WIDEST || height > TALLEST {
        return Err(Error::Other(format!(
            "the picture says it is {width} by {height}, which is past the {WIDEST} by \
             {TALLEST} a preview will read"
        )));
    }
    // Deliberately not asking here what the picture would cost, although the
    // numbers to ask it are in hand. Asking it here would refuse everything the
    // allocation cap below would have refused, which sounds like defence in
    // depth and is the opposite: the cap would then never fire, and a cap that
    // never fires is a cap nothing can show still works. This gate answers the
    // cheap question, whether the file is a plausible picture at all, and the
    // cap answers the expensive one on the size decoding really finds.
    Ok(())
}

/// A reader over these bytes, with the format worked out from them.
///
/// From the bytes rather than from the name or the type the sender gave, both of
/// which are their claims about a file rather than facts about it.
fn reader_over(bytes: &[u8]) -> Result<ImageReader<Cursor<&[u8]>>> {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|why| Error::Other(format!("the picture could not be read at all: {why}")))
}

/// What a picture of this size costs to hand to the window.
///
/// Always four bytes a pixel, whatever the decoder produced, because that is
/// what `Bitmap::from_rgba` reads. In `u64` so the multiplication of two `u32`
/// cannot wrap: a picture is refused for being large, never for arithmetic that
/// quietly came back small.
fn what_it_costs_to_hand_on(width: u32, height: u32) -> u64 {
    u64::from(width) * u64::from(height) * 4
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
        // Sixteen bits a channel, which is where the decoder's own bound is the
        // only thing that can refuse a picture. Both axes are well under the
        // declared-size gate, so that gate passes it. Decoding it allocates
        // eight bytes a pixel, over the bound; what would be handed to the
        // window afterwards is four bytes a pixel, under it. So neither the gate
        // before nor the check after can see this one, and taking the
        // allocation cap away is what makes this test go red. Measured by hand
        // rather than assumed.
        //
        // A declared size is metadata the sender controls and can misstate,
        // which is why there is a second bound at all. `sound_scheme_import`
        // refuses a zip entry on its declared size and again on what
        // decompression really produces, for the same reason.
        let across = 2600;
        assert!(across < WIDEST);
        assert!(u64::from(across) * u64::from(across) * 8 > MOST_A_PICTURE_MAY_COST);
        assert!(u64::from(across) * u64::from(across) * 4 < MOST_A_PICTURE_MAY_COST);

        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba16(image::ImageBuffer::new(across, across))
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("a sixteen bit PNG this test made");

        assert!(read(&bytes).is_err());
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
