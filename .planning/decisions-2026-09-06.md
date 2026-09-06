# Decisions taken 2026-09-06

Eight answers from Pratik, covering phases 5 and 7. Recorded here first because
an executor holds the checkout; these move into `.planning/` and into the plans
themselves when it frees up.

## Phase 5

**1. Moving a task that lives on a provider: build it, with a local cache as the
safeguard.** Not the refusal. The move is a delete on one list and a create on
the other, and the failure this is about happens between the two, so the local
cache holds the task's full content across the gap and the recovery reads from
it. That makes a crash or a closed program recoverable rather than a loss. Own
plan, because it needs stored in-progress state and a resume path.

**2. Notes syncing: all three, not one.** CalDAV VJOURNAL, a second
implementation to prove the seam, and OneNote. Pratik's words: "We need full
support depending on the service people use." Add a phase if necessary. This
overturns the single-backend assumption every phase-5 plan was written against.

**3. Move and copy: all five modules,** events, tasks, notes, contacts and
reminders.

**4. A reminder moves between accounts.** That is the container reminders
already have, so there is no new table and no invented concept. Moving a
reminder means moving it to a different account, the way mail already moves
between accounts. This answers the gap under decision 3: "move a reminder" had
no meaning before this.

**5. Allow Changes stays as it is, and the move says so sooner.** Nobody is
refused at move time and nobody is left uninformed, they find out one sync
later. The plan already makes the move itself say what has and has not happened
at the moment it happens. The written requirement is reworded to match, rather
than the behaviour changed, because every other edit in the program behaves this
way and making task moves the exception is worse for somebody moving by keyboard.

## Phase 7

**6. Azure Artifact Signing.** About $10 a month, Pratik's own name as the
publisher, no hardware. **The United States or Canada residence requirement is
met**, asked and confirmed 2026-09-06, so the cheapest option stands and
criterion 1 can close once the account exists.

EV was ruled out on the facts before this was asked: Microsoft's page of
2026-08-17 says EV certificates no longer bypass SmartScreen. The roadmap's
criterion 1 still says the opposite and wants correcting.

**7. Updates: download directly.** A real self-updater, not the browser link the
plan recommended. Three parts to it:

- fetch and run the installer, which means this program downloads an executable
  and runs it, the largest single increase in attack surface in the milestone.
  It is worth much more with decision 6 in place than without it, and the
  ordering should reflect that: sign first, then update.
- **rewrite `docs/privacy.md` to say what GitHub does with the request.** The
  page currently promises "no update check that says who you are", and GitHub
  associates an unauthenticated request with the originating IP address, with a
  limit of 60 an hour per address. The promise as written stops being true, so
  the page says plainly what is sent and what GitHub keeps.
- **offer a choice of public releases or development releases.** New, not in any
  plan or requirement. It is a setting, so the reachability rule binds the plan
  that introduces it: it belongs in a section somebody would look in, and a
  top-level field fails on arrival until a screen offers it.

## What these answers cost

Three of them overturn assumptions the nine phase-5 plans were written against:
single notes backend, three modules for move and copy, and the task-move
refusal standing. Those plans need re-splitting before they are moved into the
repository. The research under them and the premise corrections they made are
still good; the split and the wave order are not.

Phase 7 grows by a self-updater and a release channel setting, which `07-05` was
explicitly written not to build.
