# Credits

None of this is legally required. Every source below is CC0, which needs no
attribution at all. It is written down anyway, both as a courtesy to the
people who made these sounds and free to use, and as a worked example of
what "safe to bundle" looks like for the next scheme built here.

## Soft Chimes

16 sounds taken from [UI SFX](https://uisfx.com/)
([source](https://github.com/romainsimon/uisfx)), the "soft" pack, one of
its 12 sound packs across 78 semantic cues. Dedicated to the public domain
under CC0 1.0 Universal
([full license text](https://github.com/romainsimon/uisfx/blob/main/packages/uisfx/LICENSE-AUDIO)).

Downloaded as MP3, converted to mono 44.1kHz WAV with `ffmpeg`, no other
editing. Each of Wixen Mail's 16 feedback events was matched to the closest
fitting cue name from UI SFX's own taxonomy:

| Wixen Mail event | UI SFX cue |
|---|---|
| `thread_landed` | `select` |
| `edge_of_list` | `blocked` |
| `new_mail` | `receive` |
| `message_sent` | `send` |
| `send_failed` | `error` |
| `connection_lost` | `disconnect` |
| `connection_restored` | `connect` |
| `sync_complete` | `complete` |
| `action_refused` | `cancel` |
| `unsafe_message` | `warning` |
| `misspelled_word` | `typing` |
| `reminder` | `notification` |
| `has_attachment` | `info` |
| `account_needs_attention` | `unlock` |
| `confirmed` | `check` |
| `nothing_found` | `retry` |

Not verified by ear. Every clip is real, decodable audio between 0.1 and 1
second long, checked against the actual downloaded files, not assumed; this
is a real gap, not a false claim, and belongs to a real listening pass
before this scheme is called finished, matching the same honesty this
project already applies to a proposed tone's hertz and length.
