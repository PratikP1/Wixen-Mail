"""Whether Windows says this session is locked: one line, for the commit gate.

Prints `locked`, `unlocked`, or `not said: ` and why, and exits 0 whenever it
printed a line. `scripts/check.sh` asks it when a run reaches the target whose
tests need the clipboard, which Windows refuses to every program while the
session is locked (ledger 716), and leaves those tests out on `locked` alone.

It asks the question `tests/a_passphrase_box_takes_a_real_paste.rs` asks:
`WTSQuerySessionInformationW` for this session, `WTSSessionInfoEx`. It never
reads the process list, because `LockApp.exe` stays resident after an unlock,
and never tries the clipboard, which would put the gate itself on it.

Added 2026-10-23 by 13.3-24.8, on Pratik's "yes" of that day. Importing it does
nothing, so `python -m doctest` can load it and run the worked examples below.
"""

import sys

WTS_CURRENT_SERVER_HANDLE = None
WTS_CURRENT_SESSION = 0xFFFFFFFF
WTS_SESSION_INFO_EX = 25
WTS_SESSIONSTATE_LOCK = 0
WTS_SESSIONSTATE_UNLOCK = 1


def the_lock_in(answer: bytes) -> str:
    """The lock state in a `WTSINFOEXW` answer: its level in the first four
    bytes and the session's flags at byte 16, both little-endian.

    Level 1 with flags 0 is a locked session:

    >>> the_lock_in(bytes([1, 0, 0, 0]) + bytes(12) + bytes([0, 0, 0, 0]))
    'locked'

    Level 1 with flags 1 an unlocked one:

    >>> the_lock_in(bytes([1, 0, 0, 0]) + bytes(12) + bytes([1, 0, 0, 0]))
    'unlocked'

    Any other flags say nothing, and nor does an answer too short to hold
    them or one of another level:

    >>> the_lock_in(bytes([1, 0, 0, 0]) + bytes(12) + bytes([0xFF, 0xFF, 0xFF, 0xFF]))
    'not said: 20 bytes, level 1, session flags 4294967295'
    >>> the_lock_in(bytes([1, 0, 0, 0]) + bytes(12))
    'not said: 16 bytes, level 1, session flags none'
    >>> the_lock_in(bytes([2, 0, 0, 0]) + bytes(12) + bytes([0, 0, 0, 0]))
    'not said: 20 bytes, level 2, session flags 0'
    >>> the_lock_in(b"")
    'not said: 0 bytes, level none, session flags none'
    """

    def word(at: int):
        four = answer[at : at + 4]
        return int.from_bytes(four, "little") if len(four) == 4 else None

    def shown(value) -> str:
        return "none" if value is None else str(value)

    level, flags = word(0), word(16)
    if level == 1 and flags == WTS_SESSIONSTATE_LOCK:
        return "locked"
    if level == 1 and flags == WTS_SESSIONSTATE_UNLOCK:
        return "unlocked"
    return f"not said: {len(answer)} bytes, level {shown(level)}, session flags {shown(flags)}"


def this_sessions_lock() -> str:
    """Ask Windows, through `ctypes`, and read its answer with `the_lock_in`."""
    if sys.platform != "win32":
        return f"not said: this is {sys.platform}, and only Windows is asked"

    import ctypes
    from ctypes import wintypes

    wtsapi32 = ctypes.WinDLL("wtsapi32", use_last_error=True)
    query = wtsapi32.WTSQuerySessionInformationW
    query.argtypes = [
        wintypes.HANDLE,
        wintypes.DWORD,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_void_p),
        ctypes.POINTER(wintypes.DWORD),
    ]
    query.restype = wintypes.BOOL
    free = wtsapi32.WTSFreeMemory
    free.argtypes = [ctypes.c_void_p]
    free.restype = None

    answer = ctypes.c_void_p()
    length = wintypes.DWORD()
    asked = query(
        WTS_CURRENT_SERVER_HANDLE,
        WTS_CURRENT_SESSION,
        WTS_SESSION_INFO_EX,
        ctypes.byref(answer),
        ctypes.byref(length),
    )
    if not asked or not answer.value:
        return f"not said: WTSQuerySessionInformationW failed with error {ctypes.get_last_error()}"
    try:
        copied = ctypes.string_at(answer.value, length.value)
    finally:
        free(answer)
    return the_lock_in(copied)


if __name__ == "__main__":
    # Through the buffer, so Windows' text mode adds no carriage return for
    # the gate to compare against.
    sys.stdout.buffer.write((this_sessions_lock() + "\n").encode("utf-8"))
