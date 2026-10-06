"""Read a proof back from a Wixen Mail profile without printing anybody's mail.

Phase 14's sittings are read before and after from the tester's own profile
(answer 9 of 2026-10-05): read-only, counts and masked lines, never a body,
and no subject unless it begins "Wixen proof". This script is that rule
written once, and tests/the_proof_reader_prints_no_private_mail.rs holds it
to it over a fixture profile.

    python scripts/read-a-proof.py --since 2026-10-05T23:50Z
    python scripts/read-a-proof.py --since 2026-10-05T23:50Z --until 2026-10-06T00:10Z
        --folder "Wixen proof" --message 1234

What it reads, and how:

- cache/message_cache.db through sqlite3 with mode=ro, which reads the write
  log too, and never through the program's own store, which writes when it
  opens. Each table's columns are read before anything is selected from it,
  and the account's stored secret, its address and every message's text are
  never selected. A subject is selected only where it begins "Wixen proof".
- config/app_config.json, for the log level and the allowed changes.
- every daily log, wixen-mail.YYYY-MM-DD.log, that the UTC window touches,
  since the file rolls at UTC midnight and an evening sitting spans two.
- logs/crash.log, whose stamps are local time.

What it prints from the logs: the lines it has a template for, masked. A
"Speaking:" line is printed only through a template that knows where its
subject sits, with the subject withheld unless it begins "Wixen proof", and
otherwise as its length. Other warnings and errors are tallied by shape.
Every other line is counted and not printed.

Masking mirrors the feedback report's (src/application/feedback_report.rs,
redact): everything after a subject marker goes, and every local@host keeps
two characters of its local part and the domain. Quoted text, in double
quotes or backticks, is replaced by its length, since a panic's message can
quote the string it failed on.

Python standard library only.
"""

import argparse
import datetime
import json
import os
import re
import sqlite3
import sys
from pathlib import Path

PROOF = "Wixen proof"

# A number this program files a message under is counted down from
# 4294967295 (src/data/message_cache/messages.rs, FIRST_RESERVED_UID). A
# server counts up from 1, so a number in the top half is one reserved here.
RESERVED_FROM = 2**31

STARTED = "Starting Wixen Mail v"
LEVEL = "Logging initialized at level: "
SPEAKING = "Speaking: "
SPOKEN_UNWRITTEN = " characters that are not written down here"
PANIC = "PANIC at "
NOT_A_PAGE = ", which is not a page. Nothing was opened."

# The three addresses the page window's test refused into the tester's own
# crash file on every library run until 14-06 (ledger 828).
THE_TESTS_REFUSALS = tuple(
    '--show-page was given "{}"{}'.format(address, NOT_A_PAGE)
    for address in ("mailto:somebody@example.com", "not-a-page", "")
)

# The lines printed by template: the heading, a fixed text the program still
# writes (the target holds each to src), and the pattern a message matches
# from its start.
LOG_TEMPLATES = (
    ("replay", "A waiting ", r"A waiting "),
    ("crossing", "A crossing of message ", r"A crossing of message "),
    ("crossing", "A move of message ", r"A move of message "),
    ("refusal", "A refused delete could not be put back here", r"A refused delete "),
    ("refusal", "The server would not mark message ", r"The server would not mark message "),
    ("sign-in", "Signed in to ", r"Signed in to "),
    ("sign-in", "Email sent", r"Email sent"),
    ("sync", "Asked ", r"Asked (?:Google|Google Tasks|Microsoft To Do): "),
    ("sync", " sync finished, ", r"(?:calendar|contacts|tasks) sync finished, "),
    ("sync", "Google was not asked for ", r"Google was not asked for "),
    ("sync", " sync error: ", r"(?:Contacts|Calendar) sync error: "),
    ("sync", "Task sync: ", r"Task sync: "),
    ("sync", "Notes sync: ", r"Notes sync: "),
)
HEADINGS = ("replay", "crossing", "refusal", "sign-in", "sync")

# The spoken sentences that carry a subject, as src/application/server_delete.rs
# and src/service/protocols/imap.rs word them: the fixed text, and a pattern
# naming where the subject sits. Where a sentence could be cut two ways, the
# pattern withholds the larger part.
SPOKEN_TEMPLATES = (
    ("Moved to ", r"(?P<said>Moved to [^:]*): (?P<subject>.*)(?P<after>)"),
    ("Copied to ", r"(?P<said>Copied to [^:]*): (?P<subject>.*)(?P<after>)"),
    ("Deleted", r"(?P<said>Deleted): (?P<subject>.*)(?P<after>)"),
    ("Marked for removal", r"(?P<said>Marked for removal[^:]*): (?P<subject>.*)(?P<after>)"),
    (
        "Nothing was copied.",
        r"(?P<said>Nothing was copied\.(?: [^.]* refused it\.)?) (?P<subject>.*)"
        r"(?P<after> is still in .*)",
    ),
    (
        "Nothing was moved.",
        r"(?P<said>Nothing was moved\.(?: [^.]* refused it\.)?) (?P<subject>.*)"
        r"(?P<after> is still in .*)",
    ),
)

SUBJECT_REDACTED = "[subject redacted]"
SUBJECT_WITHHELD = "[subject withheld]"

LOG_LINE = re.compile(
    r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)(?:\.\d+)?Z\s+(TRACE|DEBUG|INFO|WARN|ERROR)\s+(\S+?): (.*)$"
)
CRASH_STAMP = re.compile(r"^\[(\d{4}-\d\d-\d\d \d\d:\d\d:\d\d)\] (.*)$")
QUOTED = re.compile(r'"[^"]*"|`[^`]*`')
DIGITS = re.compile(r"\d+")


# -- Masking, one rule in two languages ------------------------------------


def mask_email(email):
    """The log's own rule, src/common/logging.rs mask_email.

    >>> mask_email("user@example.com")
    'us***@example.com'
    >>> mask_email("a@example.com")
    '***@example.com'
    >>> mask_email("test")
    '***@***'
    """
    at = email.find("@")
    if at < 0:
        return "***@***"
    local, domain = email[:at], email[at:]
    return local[:2] + "***" + domain if len(local) > 2 else "***" + domain


def is_local_part(c):
    return c.isalnum() or c in "._%+-'"


def is_host_part(c):
    return c.isalnum() or c in ".-"


def without_subject(line):
    """A line cut after the first subject marker, in any case.

    >>> without_subject("Subject: Lunch")
    'Subject: [subject redacted]'
    >>> without_subject("x subject=Lunch")
    'x subject=[subject redacted]'
    """
    lowered = line.lower()
    cuts = []
    for marker, gap in (("subject:", " "), ("subject=", "")):
        at = lowered.find(marker)
        if at >= 0:
            cuts.append((at + len(marker), gap))
    if not cuts:
        return line
    end, gap = min(cuts, key=lambda cut: cut[0])
    return line[:end] + gap + SUBJECT_REDACTED


def without_addresses(line):
    """A line with every local@host masked.

    >>> without_addresses("from ada@example.com to [b@x.org], not @here")
    'from ad***@example.com to [***@x.org], not @here'
    """
    masked = []
    copied_to = 0
    for at, c in enumerate(line):
        if c != "@" or at < copied_to:
            continue
        start = at
        while start > copied_to and is_local_part(line[start - 1]):
            start -= 1
        end = at + 1
        while end < len(line) and is_host_part(line[end]):
            end += 1
        if start == at or end == at + 1:
            continue
        masked.append(line[copied_to:start])
        masked.append(mask_email(line[start:end]))
        copied_to = end
    masked.append(line[copied_to:])
    return "".join(masked)


def redact(line):
    """The feedback report's redact, for one line."""
    return without_addresses(without_subject(line))


def without_quoted_text(text):
    """Quoted text replaced by how long it was.

    >>> without_quoted_text('of `Lunch` and "x"')
    'of `[5 characters]` and "[1 characters]"'
    """
    return QUOTED.sub(
        lambda found: "{0}[{1} characters]{0}".format(found.group(0)[0], len(found.group(0)) - 2),
        text,
    )


def masked(text):
    """What any printed text goes through."""
    return redact(without_quoted_text(text))


# -- The window ------------------------------------------------------------


def a_moment(written):
    """A UTC time from 2026-10-05T23:50Z, 2026-10-05T23:50:00Z or 2026-10-05."""
    trimmed = written.strip().rstrip("Zz")
    for shape in ("%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M", "%Y-%m-%d"):
        try:
            moment = datetime.datetime.strptime(trimmed, shape)
        except ValueError:
            continue
        return moment.replace(tzinfo=datetime.timezone.utc)
    raise argparse.ArgumentTypeError(
        "{!r} is not a UTC time such as 2026-10-05T23:50Z".format(written)
    )


def stamped(moment):
    return moment.strftime("%Y-%m-%dT%H:%M:%SZ")


def the_days(since, until):
    day = since.date()
    while day <= until.date():
        yield day
        day += datetime.timedelta(days=1)


# -- The log ---------------------------------------------------------------


def log_lines(logs, since, until, counted):
    """Each stamped line of every daily file the window touches, in order."""
    for day in the_days(since, until):
        path = logs / "wixen-mail.{}.log".format(day.isoformat())
        if not path.is_file():
            counted["missing files"].append(path.name)
            continue
        with open(path, encoding="utf-8", errors="replace") as text:
            for line in text:
                found = LOG_LINE.match(line.rstrip("\r\n"))
                if not found:
                    counted["without a time stamp"] += 1
                    continue
                stamp, level, module, message = found.groups()
                moment = datetime.datetime.strptime(stamp, "%Y-%m-%dT%H:%M:%S").replace(
                    tzinfo=datetime.timezone.utc
                )
                if since <= moment <= until:
                    counted["read"] += 1
                    yield moment, level, module, message


def the_template_for(message):
    for heading, _, pattern in LOG_TEMPLATES:
        if re.match(pattern, message):
            return heading
    return None


def without_the_fields(message):
    """A spoken message without the topic field the log writes after it."""
    at = message.rfind(" topic=")
    return message[:at] if at >= 0 else message


def spoken(message):
    """A Speaking line as it may be printed."""
    if not message.startswith(SPEAKING):
        # "Speaking N characters that are not written down here": a count.
        return without_the_fields(message)
    said = without_the_fields(message)[len(SPEAKING) :]
    for _, pattern in SPOKEN_TEMPLATES:
        found = re.fullmatch(pattern, said)
        if found:
            subject = found.group("subject")
            shown = subject if subject.startswith(PROOF) else SUBJECT_WITHHELD
            if found.group("said").startswith("Nothing"):
                rendered = found.group("said") + " " + shown + found.group("after")
            else:
                rendered = found.group("said") + ": " + shown + found.group("after")
            return SPEAKING + masked(rendered)
    return SPEAKING + "[{} characters in a shape this reader has no template for]".format(
        len(said)
    )


def a_shape(module, message):
    """What a warning is tallied by: its module and its words before the first colon."""
    before = message.split(":", 1)[0]
    if any(c in before for c in "\"`'@"):
        before = "[a line of {} words]".format(len(before.split()))
    return "{}: {}".format(module.replace("wixen_mail::", "", 1), DIGITS.sub("N", before)[:100])


def read_the_log(logs, since, until):
    counted = {"read": 0, "without a time stamp": 0, "missing files": []}
    builds, levels, spoken_lines, shapes = [], [], [], {}
    by_heading = {heading: [] for heading in HEADINGS}
    for moment, level, module, message in log_lines(logs, since, until, counted):
        if message.startswith(STARTED):
            builds.append((moment, message[len(STARTED) :]))
        elif message.startswith(LEVEL):
            levels.append((moment, message[len(LEVEL) :]))
        elif message.startswith(SPEAKING) or SPOKEN_UNWRITTEN in message:
            spoken_lines.append((moment, spoken(message)))
        elif the_template_for(message):
            by_heading[the_template_for(message)].append((moment, masked(message)))
        elif level in ("WARN", "ERROR"):
            shape = a_shape(module, message)
            shapes[shape] = shapes.get(shape, 0) + 1

    print("The build and the log level")
    for moment, build in builds:
        print("  {}  build {}".format(stamped(moment), masked(build)))
    for moment, level in levels:
        print("  {}  level {}".format(stamped(moment), masked(level)))
    if not builds:
        print("  no start in the window")

    print()
    print("Log lines by template")
    print(
        "  {} lines read, {} without a time stamp".format(
            counted["read"], counted["without a time stamp"]
        )
    )
    for name in counted["missing files"]:
        print("  no {}".format(name))
    for heading in HEADINGS:
        lines = by_heading[heading]
        print("  {}, {} lines".format(heading, len(lines)))
        for moment, line in lines:
            print("    {}  {}".format(stamped(moment), line))

    print()
    print("Spoken")
    print("  A Speaking: line is what the program handed the screen reader, not what NVDA said.")
    for moment, line in spoken_lines:
        print("    {}  {}".format(stamped(moment), line))

    print()
    print("Other warnings and errors, by shape")
    for shape, count in sorted(shapes.items()):
        print("  {}  {}".format(count, shape))


# -- The settings ----------------------------------------------------------


def read_the_settings(config):
    print("Settings")
    path = config / "app_config.json"
    try:
        with open(path, encoding="utf-8") as text:
            settings = json.load(text)
    except (OSError, ValueError) as why:
        print("  could not be read: {}".format(type(why).__name__))
        return
    print("  log_level {}".format(masked(str(settings.get("log_level")))))
    allowed = settings.get("allowed_changes")
    if isinstance(allowed, dict):
        for name, value in sorted(allowed.items()):
            if isinstance(value, bool):
                print("  allowed_changes {} {}".format(masked(name), "on" if value else "off"))


# -- The database ----------------------------------------------------------


def columns_of(con, table):
    return {row[1] for row in con.execute("PRAGMA table_info({})".format(table))}


def count_of(con, table):
    if not columns_of(con, table):
        return "not in this database"
    return con.execute("SELECT COUNT(*) FROM {}".format(table)).fetchone()[0]


def reserved(uid):
    return "reserved here" if uid is not None and uid >= RESERVED_FROM else "not reserved"


def a_row(path, uid, filed, deleted):
    return "in {}, number {}, {}, filed here {}, deleted {}".format(
        masked(str(path)), uid, reserved(uid), filed, deleted
    )


def the_queue(con):
    print("The waiting queue")
    have = columns_of(con, "moves_waiting")
    wanted = [
        "message_row_id",
        "kind",
        "from_folder_path",
        "uid",
        "into_folder_path",
        "asked_at",
        "to_account_name",
    ]
    present = [column for column in wanted if column in have]
    rows = (
        con.execute("SELECT {} FROM moves_waiting".format(", ".join(present))).fetchall()
        if present
        else []
    )
    print("  moves waiting: {}".format(len(rows)))
    for values in rows:
        row = dict(zip(present, values))
        into = row.get("into_folder_path")
        said = "    row {}: {}, from {} number {}{}, asked {}".format(
            row.get("message_row_id"),
            row.get("kind"),
            row.get("from_folder_path"),
            row.get("uid"),
            " into {}".format(into) if into else "",
            row.get("asked_at"),
        )
        if row.get("to_account_name"):
            said += ", to the account {}".format(row["to_account_name"])
        print(masked(said))
    print("  moves in flight: {}".format(count_of(con, "move_in_flight")))
    print("  messages waiting to send: {}".format(count_of(con, "outbox_queue")))


def filed_column(con, of="m."):
    """The filed_here column, or NULL in a database from before it existed."""
    return of + "filed_here" if "filed_here" in columns_of(con, "messages") else "NULL"


def the_proof_messages(con):
    print("Proof messages, whose subjects begin {!r}".format(PROOF))
    rows = con.execute(
        "SELECT m.id, f.path, m.uid, {}, m.deleted, m.subject FROM messages m "
        "LEFT JOIN folders f ON f.id = m.folder_id "
        "WHERE substr(m.subject, 1, {}) = ? ORDER BY m.id".format(filed_column(con), len(PROOF)),
        (PROOF,),
    ).fetchall()
    for row, path, uid, filed, deleted, subject in rows:
        print("  row {} {}: {}".format(row, a_row(path, uid, filed, deleted), masked(subject)))
    if not rows:
        print("  none")


def the_folders_asked_for(con, names):
    if not names:
        return
    print()
    print("Folders asked for")
    for name in names:
        folders = con.execute(
            "SELECT id, account_id, path FROM folders WHERE name = ? OR path = ?", (name, name)
        ).fetchall()
        if not folders:
            print("  {}: no folder of that name".format(masked(name)))
        filed = filed_column(con, of="")
        for folder_id, account, path in folders:
            total, filed_here, holding = con.execute(
                "SELECT COUNT(*), TOTAL({}), TOTAL(uid >= ?) FROM messages "
                "WHERE folder_id = ?".format(filed),
                (RESERVED_FROM, folder_id),
            ).fetchone()
            print(
                masked(
                    "  {}: {} messages, {} filed here, {} holding a reserved number, "
                    "in account {}".format(path, total, int(filed_here), int(holding), account)
                )
            )


def the_messages_asked_for(con, rows):
    if not rows:
        return
    print()
    print("Messages asked for")
    for asked in rows:
        found = con.execute(
            "SELECT f.path, m.uid, {}, m.deleted, "
            "CASE WHEN substr(m.subject, 1, {}) = ? THEN m.subject END "
            "FROM messages m LEFT JOIN folders f ON f.id = m.folder_id "
            "WHERE m.id = ?".format(filed_column(con), len(PROOF)),
            (PROOF, asked),
        ).fetchone()
        if found is None:
            print("  row {}: no such message".format(asked))
            continue
        path, uid, filed, deleted, subject = found
        print(
            "  row {}: {}, {}".format(
                asked,
                a_row(path, uid, filed, deleted),
                "subject {}".format(masked(subject)) if subject else "subject withheld",
            )
        )


def the_accounts(con):
    print("Accounts")
    have = columns_of(con, "accounts")
    present = [column for column in ("id", "provider", "protocol", "use_oauth") if column in have]
    if not present:
        print("  not in this database")
        return
    for values in con.execute("SELECT {} FROM accounts".format(", ".join(present))):
        row = dict(zip(present, values))
        print(
            masked(
                "  account {}: provider {}, protocol {}, use_oauth {}".format(
                    row.get("id"), row.get("provider"), row.get("protocol"), row.get("use_oauth")
                )
            )
        )


def the_personal_information(con):
    print("Calendars, contacts, tasks and sync state")
    if "source_provider" in columns_of(con, "calendars"):
        for provider, count in con.execute(
            "SELECT COALESCE(source_provider, 'none'), COUNT(*) FROM calendars "
            "GROUP BY 1 ORDER BY 1"
        ):
            print("  calendars from {}: {}".format(masked(provider), count))
    print("  events: {}".format(count_of(con, "calendar_events")))
    print("  contacts: {}".format(count_of(con, "contacts")))
    print("  tasks: {}".format(count_of(con, "tasks")))
    print("  sync state rows: {}".format(count_of(con, "sync_state")))
    have = columns_of(con, "sync_state")
    wanted = ["account_id", "sync_type", "provider", "last_full_sync", "last_incremental_sync"]
    present = [column for column in wanted if column in have]
    if present:
        for values in con.execute("SELECT {} FROM sync_state".format(", ".join(present))):
            print(masked("    " + ", ".join("{} {}".format(*pair) for pair in zip(present, values))))


def read_the_database(database, folders, messages):
    if not database.is_file():
        print("No database at {}".format(database))
        return
    con = sqlite3.connect(database.resolve().as_uri() + "?mode=ro", uri=True)
    try:
        the_accounts(con)
        print()
        the_queue(con)
        print()
        the_proof_messages(con)
        the_folders_asked_for(con, folders)
        the_messages_asked_for(con, messages)
        print()
        the_personal_information(con)
    finally:
        con.close()


# -- The crash file --------------------------------------------------------


def crash_entries(path):
    """Each entry of the crash file: its local stamp and its lines."""
    entry = None
    with open(path, encoding="utf-8", errors="replace") as text:
        for line in text:
            line = line.rstrip("\r\n")
            found = CRASH_STAMP.match(line)
            if found:
                if entry:
                    yield entry
                entry = (found.group(1), [found.group(2)])
            elif entry:
                entry[1].append(line)
    if entry:
        yield entry


def read_the_crash_file(logs, since, until):
    print("The crash file, stamped in local time")
    path = logs / "crash.log"
    if not path.is_file():
        print("  no crash file")
        return
    dropped = 0
    for stamp, lines in crash_entries(path):
        moment = datetime.datetime.strptime(stamp, "%Y-%m-%d %H:%M:%S").astimezone(
            datetime.timezone.utc
        )
        if not since <= moment <= until:
            continue
        if len(lines) == 1 and lines[0] in THE_TESTS_REFUSALS:
            dropped += 1
            continue
        print("  {} local: {}".format(stamp, masked(lines[0])))
        for line in lines[1:]:
            print("    {}".format(masked(line.strip())))
    print(
        "  dropped {} lines the library's tests wrote (the page window's refusals, "
        "ledger 828)".format(dropped)
    )


# -- The command -----------------------------------------------------------


def the_default_profile():
    named = os.environ.get("WIXEN_MAIL_DATA")
    if named:
        return Path(named)
    return Path(os.environ.get("LOCALAPPDATA", ".")) / "wixen-mail"


def the_arguments(argv):
    parser = argparse.ArgumentParser(
        description="Read a proof back from a Wixen Mail profile, read-only, printing no mail."
    )
    parser.add_argument(
        "--profile",
        type=Path,
        default=the_default_profile(),
        help="the profile folder; WIXEN_MAIL_DATA, else %%LOCALAPPDATA%%\\wixen-mail",
    )
    parser.add_argument("--since", type=a_moment, help="the window's start, UTC, such as 2026-10-05T23:50Z")
    parser.add_argument("--until", type=a_moment, help="the window's end, UTC; now when not given")
    parser.add_argument(
        "--folder", action="append", default=[], help="a folder to count, by name or path; may repeat"
    )
    parser.add_argument(
        "--message", action="append", type=int, default=[], help="a message row to read; may repeat"
    )
    parser.add_argument(
        "--mask-stdin",
        action="store_true",
        help="print the lines on standard input masked by the feedback report's rule, and stop",
    )
    parser.add_argument(
        "--openings",
        action="store_true",
        help="print every fixed text this reader matches, one per line, and stop",
    )
    arguments = parser.parse_args(argv)
    if not (arguments.mask_stdin or arguments.openings or arguments.since):
        parser.error("--since is needed to read a profile")
    return arguments


def the_openings():
    fixed = [STARTED, LEVEL, SPEAKING, SPOKEN_UNWRITTEN, PANIC, NOT_A_PAGE]
    fixed += [text for _, text, _ in LOG_TEMPLATES]
    fixed += [text for text, _ in SPOKEN_TEMPLATES]
    return fixed


def main(argv):
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    arguments = the_arguments(argv)
    if arguments.openings:
        print("\n".join(the_openings()))
        return 0
    if arguments.mask_stdin:
        for line in sys.stdin.read().splitlines():
            print(redact(line))
        return 0

    since = arguments.since
    until = arguments.until or datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0)
    if until < since:
        print("The window ends before it starts: {} to {}".format(stamped(since), stamped(until)))
        return 2
    profile = arguments.profile
    print("Wixen Mail proof reader, read-only")
    print("Profile: {}".format(profile))
    print("Window: {} to {} (UTC)".format(stamped(since), stamped(until)))
    print()
    read_the_log(profile / "logs", since, until)
    print()
    read_the_settings(profile / "config")
    print()
    read_the_database(profile / "cache" / "message_cache.db", arguments.folder, arguments.message)
    print()
    read_the_crash_file(profile / "logs", since, until)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
