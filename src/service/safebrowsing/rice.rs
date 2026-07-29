//! Decoding the compressed form Safe Browsing sends its lists in.
//!
//! A threat list update is hundreds of thousands of 32-bit prefixes. Sent as
//! raw bytes that is megabytes on every refresh, so Google sorts them, takes
//! the difference between neighbours, and Rice-Golomb encodes the differences.
//! Sorted values have small differences, small differences have short codes,
//! and the whole list arrives in a fraction of the space.
//!
//! # The part that goes wrong
//!
//! The bit order. The encoded data is a byte string, and the bits are read from
//! the *least* significant bit of each byte upwards, which is the opposite of
//! how bits are usually drawn on a page. Reading them the other way produces a
//! decode that succeeds, returns the right number of entries, and is entirely
//! wrong.
//!
//! That failure is silent and it is dangerous in a specific way: a corrupted
//! prefix list does not report a phishing site as safe, it reports random sites
//! as suspect and never matches the real ones. The list checksum in
//! [`super::database`] is what catches it, and these tests are what stop it
//! happening in the first place.

use crate::common::{Error, Result};

/// The most entries one update may claim to hold.
///
/// Google's own lists run to a few hundred thousand. This is a bound on what a
/// malformed or hostile response can make this allocate, not a limit anybody
/// legitimate will reach.
const MAX_ENTRIES: u32 = 2_000_000;

/// The Rice parameter Google uses, bounded to what it can mean.
///
/// A parameter over 32 would shift a `u32` off its own end. Google sends 2 to
/// 28 in practice, and anything outside 0 to 31 is a response to refuse rather
/// than to interpret.
const MAX_PARAMETER: u32 = 31;

/// One Rice-Golomb encoded run of values, as the API sends it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiceDelta {
    /// The first value, sent in the clear because there is no previous one to
    /// take a difference from.
    pub first_value: u32,
    /// How many bits of each difference are sent literally.
    pub parameter: u32,
    /// How many differences follow the first value.
    pub entries: u32,
    /// The bit stream, least significant bit of each byte first.
    pub data: Vec<u8>,
}

/// Decode one run into the values it stands for.
///
/// The result is `entries + 1` values, ascending, because the first value is
/// sent separately and every later one is the previous plus a difference.
pub fn decode(encoded: &RiceDelta) -> Result<Vec<u32>> {
    if encoded.entries == 0 {
        return Ok(vec![encoded.first_value]);
    }
    if encoded.entries > MAX_ENTRIES {
        return Err(Error::Protocol(format!(
            "The threat list update claims {} entries, which is more than any \
             real list holds",
            encoded.entries
        )));
    }
    if encoded.parameter > MAX_PARAMETER {
        return Err(Error::Protocol(format!(
            "The threat list update uses Rice parameter {}, which cannot be \
             read",
            encoded.parameter
        )));
    }

    let mut reader = BitReader::new(&encoded.data);
    let mut values = Vec::with_capacity(encoded.entries as usize + 1);
    let mut current = encoded.first_value;
    values.push(current);

    for entry in 0..encoded.entries {
        // The quotient is unary: ones until a zero.
        //
        // The bound is the one the arithmetic implies rather than a number
        // chosen to feel safe. A quotient above this shifts off the top of a
        // u32, which makes the response malformed rather than making the
        // value something to wrap around and carry on from.
        //
        // Termination does not rest on the bound. The reader's position runs
        // across the whole stream rather than restarting per entry, so a
        // response that is all ones reads to the end once and then reports a
        // truncated entry. It cannot rescan the buffer two million times.
        let ceiling = u32::MAX >> encoded.parameter;
        let mut quotient = 0u32;
        loop {
            match reader.bit() {
                Some(1) => {
                    quotient += 1;
                    if quotient > ceiling {
                        return Err(Error::Protocol(
                            "The threat list update is malformed: a value ran \
                             past what a prefix can hold"
                                .into(),
                        ));
                    }
                }
                Some(_) => break,
                None => return Err(truncated(entry)),
            }
        }
        let remainder = reader
            .bits(encoded.parameter)
            .ok_or_else(|| truncated(entry))?;

        // Safe because of the ceiling: the shift cannot lose a bit, and the
        // remainder is smaller than the low bits the shift left free.
        let difference = (quotient << encoded.parameter) | remainder;
        // Wrapping is what Google's own decoder does here, and the values are
        // hash prefixes rather than quantities, so the arithmetic is modular by
        // nature. Overflowing is not a corruption to reject.
        current = current.wrapping_add(difference);
        values.push(current);
    }

    Ok(values)
}

fn truncated(entry: u32) -> Error {
    Error::Protocol(format!(
        "The threat list update stopped part way through entry {}",
        entry + 1
    ))
}

/// Bits out of a byte string, least significant bit of each byte first.
///
/// The order is the whole point. Reading from the most significant bit instead
/// decodes without complaint and gets every value wrong.
struct BitReader<'a> {
    data: &'a [u8],
    /// How many bits have been consumed, across the whole string.
    position: usize,
}

impl<'a> BitReader<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// The next bit, or `None` when the string has run out.
    fn bit(&mut self) -> Option<u32> {
        let byte = self.data.get(self.position / 8)?;
        let shift = self.position % 8;
        self.position += 1;
        Some(u32::from((byte >> shift) & 1))
    }

    /// The next `count` bits as a number, first bit read being the lowest.
    fn bits(&mut self, count: u32) -> Option<u32> {
        let mut value = 0u32;
        for index in 0..count {
            value |= self.bit()? << index;
        }
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode values the way the API does, so a round trip can be tested.
    ///
    /// Only used by the tests. Writing an encoder to test a decoder is
    /// circular if the encoder is derived from the decoder, so this is written
    /// from the specification rather than from the code above, and the fixed
    /// vectors below check them both against numbers worked out by hand.
    fn encode(values: &[u32], parameter: u32) -> RiceDelta {
        let mut bits: Vec<u8> = Vec::new();
        for pair in values.windows(2) {
            let difference = pair[1].wrapping_sub(pair[0]);
            let quotient = difference >> parameter;
            let remainder = difference & ((1u32 << parameter) - 1);
            bits.extend(std::iter::repeat_n(1u8, quotient as usize));
            bits.push(0);
            for index in 0..parameter {
                bits.push(((remainder >> index) & 1) as u8);
            }
        }
        let mut data = vec![0u8; bits.len().div_ceil(8)];
        for (index, bit) in bits.iter().enumerate() {
            data[index / 8] |= bit << (index % 8);
        }
        RiceDelta {
            first_value: values.first().copied().unwrap_or(0),
            parameter,
            entries: values.len().saturating_sub(1) as u32,
            data,
        }
    }

    #[test]
    fn test_the_bits_are_read_from_the_bottom_of_each_byte() {
        // The one that matters. Worked out by hand rather than round-tripped,
        // because a decoder tested only against its own encoder agrees with
        // itself about the wrong bit order.
        //
        // Parameter 2, one entry. The byte 0b0000_1101 read from the bottom
        // gives bits 1, 0, 1, 1: quotient 1 from the leading one, then the
        // terminating zero, then remainder bits 1 and 1 which is 3 read
        // lowest-first. So the difference is (1 << 2) + 3, which is 7.
        let encoded = RiceDelta {
            first_value: 100,
            parameter: 2,
            entries: 1,
            data: vec![0b0000_1101],
        };

        let values = decode(&encoded).expect("one entry");

        assert_eq!(values, vec![100, 107]);
    }

    #[test]
    fn test_a_run_with_no_entries_is_just_its_first_value() {
        let encoded = RiceDelta {
            first_value: 42,
            parameter: 4,
            entries: 0,
            data: Vec::new(),
        };

        assert_eq!(decode(&encoded).expect("no entries"), vec![42]);
    }

    #[test]
    fn test_a_list_survives_the_round_trip() {
        // Sorted ascending, which is what the protocol sends and what the
        // difference encoding assumes.
        //
        // The spacing follows the parameter, because that is what an encoder
        // does: the parameter is chosen from the data so that most quotients
        // come out at nought or one. Values spaced far wider than the
        // parameter suits are not a case Google sends, they are a case where
        // the unary run alone would be hundreds of millions of bits.
        //
        // The jitter has to stay smaller than the step or the values stop
        // ascending, and a descending pair encodes as a difference of nearly
        // four billion. The smallest step here is four, so the jitter is two.
        for parameter in [2u32, 5, 12, 20, 28] {
            let step = 1u32 << parameter;
            let values: Vec<u32> = (0..200u32)
                .map(|n| n.wrapping_mul(step).wrapping_add(n % 3))
                .collect();

            let decoded = decode(&encode(&values, parameter)).expect("round trip");

            assert_eq!(decoded, values, "at parameter {parameter}");
        }
    }

    #[test]
    fn test_a_difference_too_large_for_its_parameter_is_refused() {
        // A quotient that would shift off the top of a u32. Real data never
        // gets here, because the parameter is chosen from the data; a response
        // that does is malformed, and reading it as a wrapped value would put
        // a wrong prefix into the list quietly.
        let encoded = RiceDelta {
            first_value: 0,
            parameter: 28,
            entries: 1,
            // Sixteen bytes of ones: 128 unary bits, and at parameter 28 the
            // ceiling is 15.
            data: vec![0xFF; 16],
        };

        let error = decode(&encoded).expect_err("too large");

        assert!(error.to_string().contains("ran past"), "{error}");
    }

    #[test]
    fn test_the_values_come_back_ascending() {
        // The database binary-searches them, so an out of order list is a list
        // that reports matches at random.
        let values: Vec<u32> = (0..500).map(|n| n * 7919).collect();

        let decoded = decode(&encode(&values, 8)).expect("round trip");

        assert!(decoded.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn test_a_truncated_update_is_refused_rather_than_half_read() {
        // Half a list looks exactly like a whole list to everything
        // downstream, and a prefix set missing its second half reports the
        // sites in it as unlisted.
        let mut encoded = encode(&[1, 500, 1_000, 20_000], 5);
        encoded.data.truncate(1);

        let error = decode(&encoded).expect_err("truncated");

        assert!(
            error.to_string().contains("stopped part way"),
            "unhelpful message: {error}"
        );
    }

    #[test]
    fn test_an_update_claiming_more_entries_than_exist_is_refused() {
        // A response is a stranger's bytes like any other. Believing this
        // number would allocate several gigabytes on the strength of it.
        let encoded = RiceDelta {
            first_value: 0,
            parameter: 4,
            entries: u32::MAX,
            data: vec![0xFF; 16],
        };

        assert!(decode(&encoded).is_err());
    }

    #[test]
    fn test_an_impossible_rice_parameter_is_refused_rather_than_shifted() {
        // A parameter of 32 or more shifts a u32 off its own end, which is a
        // panic in debug and nonsense in release.
        for parameter in [32, 64, u32::MAX] {
            let encoded = RiceDelta {
                first_value: 0,
                parameter,
                entries: 1,
                data: vec![0x00; 8],
            };

            assert!(decode(&encoded).is_err(), "parameter {parameter}");
        }
    }

    #[test]
    fn test_a_stream_of_ones_does_not_spin() {
        // A quotient is unary, so a response that is all ones asks the decoder
        // to count forever. It has to stop by itself rather than by running
        // out of bytes two million times over.
        let encoded = RiceDelta {
            first_value: 0,
            parameter: 2,
            entries: 1_000,
            data: vec![0xFF; 4096],
        };

        assert!(decode(&encoded).is_err());
    }

    #[test]
    fn test_an_empty_stream_with_entries_promised_is_refused() {
        let encoded = RiceDelta {
            first_value: 0,
            parameter: 4,
            entries: 10,
            data: Vec::new(),
        };

        assert!(decode(&encoded).is_err());
    }
}
