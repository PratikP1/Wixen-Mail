# How the mail protocols were connected to the interface

_Rewritten 2026-08-29. The previous version was a three-phase build plan from
early in the project. All three phases finished, but the document went on
describing Phase 1 as in progress, counted 64 tests against the 5,430 that run
today, and ticked a 95% coverage target that was never met. It is kept as a
record of how the layers were joined, not as a plan._

This is not a guide to integrating anything with Wixen Mail. Wixen Mail is a
desktop application, not a library, and nothing here is a public interface. If
you arrived looking for how to build against it, there is nothing to build
against.

## What connects to what

Four layers, joined in one direction. A layer never reaches upward.

| Layer | Holds | Example |
|-------|-------|---------|
| `presentation` | Windows, controls, announcements | `wx_app.rs` |
| `application` | What a command means, and the rules it follows | `mail_controller.rs` |
| `service` | Protocols and providers | `protocols/imap.rs` |
| `data` | The SQLite cache and account records | `message_cache/` |

`application::mail_controller` is the join between the interface and the
protocols. It owns the async runtime, so the window never blocks on a socket:
work goes to a Tokio task, and the answer comes back as a `UIUpdate` the message
pump applies on the interface thread.

For what each layer is responsible for, read
[the architecture](architecture.md). For which features work,
read [what does and does not work](IMPLEMENTATION_STATUS.md).

## What the joining cost

Three things were harder than the original plan allowed for, and each is worth
knowing before touching that seam again.

**A window cannot wait.** Every protocol call is async and every widget call
must happen on the interface thread. The rule that came out of it: a worker
never touches a widget, and a `UIUpdate` never carries a live handle.

**Announcing a change is not the same as making it.** Reads, flags and moves are
applied to the row on screen first and announced at once, because a keystroke
that waits on a round trip feels broken. That means an announcement can turn out
to be wrong, so a refused change is put back and the reason said. `wx_app.rs`
does this in one place for flags, and `spawn_server_change` does it for the
server.

**None of it has run against a real account.** The protocol code is tested
against parsing and against loopback servers, which is why coverage in
`service/protocols` and the provider clients is low. That is tracked as work,
not as a gap in the tests.

## Where the old plan's phases ended up

- Connecting IMAP and SMTP to the interface: done, through `mail_controller`.
- Persistent caching and HTML rendering: done. The cache is
  `data/message_cache/`, with message bodies split into `bodies.rs` and kept
  under a size budget. HTML is sanitized with `ammonia` before it reaches the
  preview.
- Advanced features and polish: overtaken. What shipped is recorded in
  [the changelog](changelog.md) rather than against these phase numbers.
