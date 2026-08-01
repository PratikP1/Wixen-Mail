//! Turning a list of UIDs into the sequence sets an IMAP command takes.
//!
//! A mailbox here may hold hundreds of thousands of messages, and asking for
//! them one UID at a time is one round trip each. IMAP takes a set instead:
//! `1:500,732,900:1200`. Two things have to be right about building one.
//!
//! It has to be compact, because a server may refuse an over-long command line
//! and RFC 7162 suggests 8192 octets as a floor rather than a promise. And it
//! has to be exact: a UID silently dropped while splitting a large set is a
//! message missing from the list with nothing to show that it is missing.

/// The most characters of sequence set we put in one command.
///
/// Well under the 8192-octet line length servers are asked to accept, leaving
/// room for the command word and the item list that follow it. Being generous
/// here buys nothing: the round trips saved past a few hundred UIDs per command
/// are already down in the noise.
pub const MAX_SET_LENGTH: usize = 1024;

/// Split the UIDs across as many sequence sets as the length limit needs.
///
/// Every UID appears in exactly one of the returned sets. A set is never
/// returned empty, and the list is empty only when there was nothing to ask
/// for.
pub fn chunks(uids: &[u32], max_len: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();

    for (first, last) in runs(uids) {
        let piece = format_run(first, last);
        if !current.is_empty() && current.len() + 1 + piece.len() > max_len {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(',');
        }
        // A run that alone exceeds the limit still goes out whole. It takes a
        // limit under 21 characters to reach that, and an over-long command
        // fails where somebody can see it, where a dropped UID is a message
        // missing from the list with nothing to say it is missing.
        current.push_str(&piece);
    }

    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// The consecutive runs in the UIDs, sorted, deduplicated, zeros removed.
fn runs(uids: &[u32]) -> Vec<(u32, u32)> {
    let mut sorted: Vec<u32> = uids.iter().copied().filter(|uid| *uid > 0).collect();
    sorted.sort_unstable();
    sorted.dedup();

    let mut out: Vec<(u32, u32)> = Vec::new();
    for uid in sorted {
        match out.last_mut() {
            Some((_, last)) if *last + 1 == uid => *last = uid,
            _ => out.push((uid, uid)),
        }
    }
    out
}

/// One run as IMAP writes it.
fn format_run(first: u32, last: u32) -> String {
    if first == last {
        first.to_string()
    } else {
        format!("{first}:{last}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Read a sequence set back into the UIDs it names.
    ///
    /// Written out here rather than reused from the module under test, so the
    /// property tests below are not the code checking itself.
    fn expand(set: &str) -> Vec<u32> {
        let mut out = Vec::new();
        for part in set.split(',').filter(|p| !p.is_empty()) {
            match part.split_once(':') {
                Some((first, last)) => {
                    let first: u32 = first.parse().expect("range start should be a number");
                    let last: u32 = last.parse().expect("range end should be a number");
                    assert!(first <= last, "range runs backwards: {part}");
                    out.extend(first..=last);
                }
                None => out.push(part.parse().expect("uid should be a number")),
            }
        }
        out
    }

    /// The one set a small list of UIDs produces.
    fn one_set(uids: &[u32]) -> String {
        chunks(uids, MAX_SET_LENGTH)
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    #[test]
    fn test_nothing_to_ask_for_produces_nothing() {
        assert!(chunks(&[], MAX_SET_LENGTH).is_empty());
    }

    #[test]
    fn test_a_single_uid_is_written_alone() {
        assert_eq!(one_set(&[42]), "42");
    }

    #[test]
    fn test_consecutive_uids_become_a_range() {
        assert_eq!(one_set(&[1, 2, 3, 4, 5]), "1:5");
        assert_eq!(one_set(&[7, 8]), "7:8");
    }

    #[test]
    fn test_gaps_split_the_set_into_parts() {
        assert_eq!(one_set(&[1, 2, 3, 7, 10, 11, 12]), "1:3,7,10:12");
    }

    #[test]
    fn test_unsorted_input_still_produces_a_correct_set() {
        // A caller reading UIDs out of a hash set has no order to offer, and a
        // set built in arrival order would be wrong rather than merely untidy.
        assert_eq!(one_set(&[12, 3, 1, 11, 2, 10, 7]), "1:3,7,10:12");
    }

    #[test]
    fn test_duplicates_are_asked_for_once() {
        assert_eq!(one_set(&[5, 5, 5, 6, 6]), "5:6");
    }

    #[test]
    fn test_uid_zero_is_dropped_rather_than_sent() {
        // IMAP numbers from 1. A zero makes the server reject the command, and
        // with it every other message in the batch.
        assert_eq!(one_set(&[0, 1, 2]), "1:2");
        assert_eq!(one_set(&[0]), "");
    }

    #[test]
    fn test_a_chunk_never_exceeds_the_limit() {
        let uids: Vec<u32> = (1..=5000).map(|n| n * 2).collect(); // no runs at all
        for chunk in chunks(&uids, 100) {
            assert!(chunk.len() <= 100, "chunk too long: {} chars", chunk.len());
            assert!(!chunk.is_empty(), "empty chunk");
        }
    }

    #[test]
    fn test_chunking_asks_for_every_uid_exactly_once() {
        // The failure this guards against is a message missing from the list
        // with nothing on screen to say it is missing.
        let uids: Vec<u32> = (1..=2000)
            .map(|n: u32| n.wrapping_mul(7919) % 100_000 + 1)
            .collect();
        let mut expected: Vec<u32> = uids.clone();
        expected.sort_unstable();
        expected.dedup();

        let mut got: Vec<u32> = chunks(&uids, 64).iter().flat_map(|c| expand(c)).collect();
        got.sort_unstable();

        assert_eq!(got, expected);
    }

    #[test]
    fn test_a_contiguous_run_costs_one_command_however_long_it_is() {
        // The whole reason for ranges. A hundred thousand consecutive messages
        // are eight characters, not eight hundred thousand.
        let uids: Vec<u32> = (1..=100_000).collect();
        assert_eq!(one_set(&uids), "1:100000");
        assert_eq!(chunks(&uids, MAX_SET_LENGTH).len(), 1);
    }

    #[test]
    fn test_a_run_wider_than_the_limit_is_still_asked_for() {
        // Refusing to ask hides messages. An over-long command fails where
        // somebody can see it.
        let produced = chunks(&[4_294_967_295], 3);
        assert_eq!(produced, vec!["4294967295".to_string()]);
        assert_eq!(chunks(&[1, 2, 3], 2), vec!["1:3".to_string()]);
    }

    #[test]
    fn test_the_default_limit_holds_a_useful_number_of_uids() {
        // Worst case: no two UIDs adjacent, so every one is written in full.
        let uids: Vec<u32> = (1..=10_000).map(|n| n * 2).collect();
        let produced = chunks(&uids, MAX_SET_LENGTH);
        let per_command = uids.len() / produced.len();
        assert!(
            per_command >= 100,
            "only {per_command} uids per command; too many round trips"
        );
    }

    /// A set of `count` UIDs with no two adjacent, every one four digits wide,
    /// so the length it writes to is arithmetic rather than guesswork:
    /// four characters each and one comma between them.
    fn four_digit_uids(count: u32) -> Vec<u32> {
        (0..count).map(|n| 1000 + n * 2).collect()
    }

    fn written_length(count: u32) -> usize {
        (count as usize) * 5 - 1
    }

    #[test]
    fn test_a_set_that_exactly_fills_the_limit_is_not_split() {
        // The comparison deciding this is the difference between `>` and `>=`,
        // and nothing was watching it. Splitting one character early is not
        // wrong, it is a second round trip on every batch that lands on the
        // boundary, which on a large mailbox is a folder that takes twice as
        // long to open for no reason anybody could see.
        let uids = four_digit_uids(5);
        let exactly = written_length(5);

        let produced = chunks(&uids, exactly);

        assert_eq!(
            produced,
            vec!["1000,1002,1004,1006,1008".to_string()],
            "a set that fits exactly was split"
        );
        assert_eq!(produced[0].len(), exactly);
    }

    #[test]
    fn test_one_character_past_the_limit_starts_a_new_command() {
        // The other side of the same comparison. Over the limit has to split,
        // or the server rejects a command that is one character too long and
        // the folder does not load at all.
        let uids = four_digit_uids(5);
        let one_short = written_length(5) - 1;

        assert_eq!(
            chunks(&uids, one_short),
            vec!["1000,1002,1004,1006".to_string(), "1008".to_string()],
            "a set one character over the limit was sent whole"
        );
    }

    #[test]
    fn test_everything_that_fits_goes_in_one_command() {
        let uids = [1, 2, 3, 9, 40, 41];
        assert_eq!(
            chunks(&uids, MAX_SET_LENGTH),
            vec!["1:3,9,40:41".to_string()]
        );
    }
}
