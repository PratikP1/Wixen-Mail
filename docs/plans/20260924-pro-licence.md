# A pro licence: what it gates, how a key is checked, and what is Pratik's to decide

This is a design and a list of decisions. Nothing in the program is gated, keyed, checked or
worded by it. The code comes in a phase of its own, planned from Pratik's answers to the table
in section 9, and not before.

Written 2026-09-24 against `main` at `015035f7` for plan 12-11 (ALPHA-03, #65). Every claim
about the tree below carries the command it was read with. Those sentences are measurements of
that commit and go stale as the tree moves. Prices and merchant fees are Pratik's, quoted from
his comments on #65 of 2026-09-16, and carry that date.

## 1. What this is for, and what it is not

The tester asked on 2026-09-16, in #65:

> plan a pro license with gated features under pro. Multiple account support. Future RSS
> reader. PGP support. Allow working with multiple calendars. Future sound packs for event
> announcements. Suggest other potential pro features.

So this document answers four questions: which features sit behind a licence, how the program
checks a key without asking a server, what happens when a licence lapses, and what the alpha
carries in the meantime. It also lists, in one table, every choice that is Pratik's.

What it is not:

- It is not code. No field, setting, seam, sentence or test is added by the plan that wrote it.
- It is not a gate. Nothing in the alpha changes because this document exists.
- It is not a decision. Where it proposes something, the proposal is marked as one, and the
  answer column of the table in section 9 is empty.

## 2. What exists today

Read on 2026-09-24 at `015035f7`.

**Nothing in the program knows about a licence.**

```
$ grep -rniE 'licen[cs]e key|entitlement' src --include=*.rs; echo "exit $?"
exit 1
```

The issue's wider search also matched `entitle`, which answers 23 lines in 10 files
(`grep -rniE 'entitle' src | wc -l` and `grep -rliE 'entitle' src | wc -l`). None is about a
licence. One of them is a type one letter away from the name the issue proposed for the seam,
and section 5 deals with it:

```
$ grep -rn 'type Entitles' src
src/presentation/wx_app.rs:13150:type Entitles = Rc<dyn Fn(Option<&str>)>;
```

That alias and its field `entitle` carry the page window's title from the browser control to
the window. The plan for this document placed them at `wx_app.rs:12920-12971` on 2026-09-20;
they have moved about 230 lines since.

**Several mail accounts work today and cost nothing.** `application::accounts` holds any number
of them, and nothing counts them against a limit:

```
$ grep -rliE 'max_accounts|account_limit' src; echo "exit $?"
exit 1
```

So gating a second account takes something back from anybody who already has one.

**PGP reads with one key.** `src/service/pgp/mod.rs` says of itself: "One key, imported; one
message, opened; four ways of failing, each said in its own words. Several keys, choosing
between them, public keys, key servers, revocation, and anything outgoing are all outside it."
The rest is #49 (a key manager) and #52 (PGP/MIME, signing, encrypting), both open.

**Several calendars per account sync and show; free/busy asks one place per account.**
`application::asking_when_free::where_to_ask` (`grep -rn 'pub fn where_to_ask' src`, line 172)
asks the first calendar server the account can sign in to, else Microsoft Graph, else nobody.
Every source an account has is #57, open.

**Sound schemes exist, with an import.** The generated tones, the bundled schemes and
`import_zip` are built:

```
$ grep -rn 'pub fn import_zip' src
src/presentation/accessibility/sound_scheme_import.rs:62:pub fn import_zip(zip_path: &Path, id: &str, schemes_dir: &Path) -> Result<SoundScheme> {
```

What is missing is somewhere to download packs from. The
[earcon design](20260823-earcon-sound-schemes.md) keeps that as its phase 5, blocked on
`wixen.app`. So "sound packs" is a question of distribution, not of mechanism.

**There is no RSS or Atom reader.**

```
$ grep -rniE '\brss\b|atom feed' src --include=*.rs; echo "exit $?"
exit 1
```

A wider search for `feed reader` finds one comment in `src/application/invitations.rs`, about
the reader a subscribed calendar feed goes through. That is iCalendar, not news.

**The crates an offline check and a stored key need are already here.**

```
$ grep -nE '^(ring|keyring|pgp) = ' Cargo.toml
115:keyring = "4"
284:ring = "0.17"
318:pgp = "0.20"
$ grep -nE '^name = "(ed25519-dalek|ring|keyring)"' Cargo.lock
1705:name = "ed25519-dalek"
3081:name = "keyring"
4869:name = "ring"
```

`ring` and `keyring` are direct dependencies. `ring` already verifies signatures for signed
mail (`grep -rn 'ring::signature' src --include=*.rs` answers 5 lines, all in
`src/service/signed_mail.rs`), and it verifies Ed25519. `ed25519-dalek` is in the lock file only
because `pgp` depends on it; using it directly would add a line to `Cargo.toml`. So the check
proposed below uses `ring`, and no crate is added.

**One correction to the issue.** #65 said `ring` provides Ed25519 "through the update
verifier's dependencies" and named `update_download.rs:858` as the pattern. The line is right
(`grep -rn 'pub fn verify' src/service/update_download.rs` answers `:858`), and the function is
the right shape: a thing that arrives from outside, checked here, and refused unless it is ours.
The mechanism is different. That function checks the installer's Authenticode signature
through Windows (`WinVerifyTrust`), not a signature over a string with a key held in the
binary. The licence check copies its shape and not its code.

**The one-seam pattern.** `application::allowed` answers whether a change may go out. Every
client asks `allowed_for` (line 912) before it is built, the answer is the narrowest of the
command line, the application's setting and the account's own, and the setting has a section
of Settings named by one constant, `SETTINGS_SECTION` (line 186), so the sentence that sends
somebody there and the heading they find are the same words. The seam in section 5 copies
that.

**Secrets have one way into the credential store, and one way out.** `service::secret_store`
opens every entry. `application::forget`, the uninstaller's erase, lists the entries of every
owner, and its imports name the six:

```
$ grep -n 'use crate::service::' src/application/forget.rs
13:use crate::service::{caldav, carddav, credentials, oauth, pgp, security};
```

A licence would be a seventh owner. Its comment on the address books says why it joins that list
in the commit that first names its service: "An owner added here later than the code that
writes it is a password left on the machine after this said everything was erased."

## 3. The proposed line between free and pro

The product's own line comes first, and it is not a proposal: **nothing a blind person needs
to work a mailbox is gated, and no accessibility feature ever is.** Wixen Mail exists so that
somebody who cannot see the screen can read, answer and file their mail. A licence that stood
between them and that would defeat the product.

Under that line, #65's body proposed: one account of each kind, every reading feature, every
accessibility feature and every safety feature are free; breadth, integration and convenience
are pro. The table applies that proposal to each feature the issue names and each it added.
Where this document departs from the issue's list, the row says so. The whole line is
decision 1 in section 9.

| Feature | In the tree on 2026-09-24 | Proposed side | What gating it takes back |
|---|---|---|---|
| Reading, answering and filing mail; everything the screen reader is told; keyboard access | Built | Free, always | Not gated |
| Earcons: generated tones, bundled schemes, importing any scheme zip | Built | Free, always | Not gated: sound is how several events are told apart without sight |
| Safety: S/MIME signature checks, phishing warnings, Safe Browsing, reading PGP with one key | Built | Free, always | Not gated |
| Export: File, Export Mailbox for mail, Export vCard for contacts | Built | Free, always, and so is any export added later | Not gated: a lapse must never lock anybody's data in |
| One mail account of each kind | Built | Free | Nothing |
| More than one mail account | Built, free, no limit | Pro (the issue's proposal) | Every second account anybody has today, which is why section 6 grandfathers them and decision 2 asks whether to gate at all |
| Several calendars per account, synced and shown | Built, free | Free | Would take back what people have, so this document proposes leaving it free even though the tester named "multiple calendars" as pro |
| Free/busy from every source an account has (#57) | Not built | Pro | Nothing |
| PGP beyond one key: key manager, several keys, signing, encrypting, PGP/MIME (#49, #52) | Not built | Pro | Nothing |
| Shared mailboxes and delegation (#59) | Not built | Pro | Nothing |
| Directory lookup: an LDAP directory named on an account | Built, free | Free | Would take back what people have |
| Directory lookup: Graph people search, LDAP with a sign-in (#55) | Not built | Pro | Nothing |
| Downloadable sound packs from the site | Not built; waits on `wixen.app` | Pro, proposed | Nothing, but see the note below the table |
| RSS and Atom reader | Not built | Pro | Nothing |
| Quick Steps (#60) | Not built | Pro | Nothing |
| Running a rule over a folder on demand (#61) | Not built; rules run on arrival today | Pro | Nothing |
| A rule that says a phrase first when a row is read (#62) | Built, free (commit `39d53503`) | Free: a change from the issue's list | It is how a row is spoken, so the product's own line keeps it free |
| Importing Outlook data files and folders of saved messages (#53) | Built and reached from File (commit `8eba6a38`) | Free: a change from the issue's list | Import is how somebody leaves Outlook, which is what this program replaces |
| Priority support through Send Feedback (#64) | Send Feedback built and free; no priority | Pro, the ordering only | Nothing: every report is still answered |
| Early access to alpha builds | Builds go to testers now | Pro, the issue's proposal | Nothing today |

Two rows need a sentence each.

**Sound packs sit close to the line.** Earcons are an accessibility feature. The tester named
sound packs as pro, and the proposal honours that only for curated packs downloaded from the
site. The mechanism, the built-in schemes and importing a zip from anywhere stay free, so a
blind person never needs a licence to hear their mail. If Pratik reads packs as accessibility,
the row moves to free.

**The table is not every feature.** It lists what #65 named and what its body added. Anything
not in it is free until a later plan puts it in this table and Pratik agrees.

## 4. The licence

**A short signed string.** A licence is a payload and an Ed25519 signature over it, written in
a form that survives email and pasting: letters and digits in groups, case and spaces ignored.
The payload holds:

- a format number, so a later format can be told apart;
- the plan: supporter, pro yearly, or pro perpetual;
- an identifier chosen at random when the key is minted, and not the buyer's email address;
- the date it was issued;
- the date it expires, or for a perpetual licence the date its updates end, if decision 5 gives
  it one.

The project's private key signs licences and never enters the tree. Its public half is compiled
into the program. The program checks a licence with `ring` and nothing else, offline, every time
it starts.

**Entered on Settings, in a section of its own.** One field for the key, one sentence saying what
it unlocks and when it ends, and nothing else. The project's rule is that every setting is
reachable from Settings. The key is meant to be pasted from the email that delivers it. The field
must accept a paste and must never ask anybody to retype or transcribe it, which is WCAG 2.2's
3.3.8, accessible authentication.

**Kept in the credential store.** Like every other secret here, through `service::secret_store`,
under a service name only one module owns, and named in `application::forget` so the
uninstaller's erase removes it. Never in `message_cache.db`, never in the settings file, never
logged. A log line may say which plan is active and when it ends, never the key.

**Checked offline.** Nothing is asked of any server to run the program. An online check for
revocation happens only if Pratik decides it should (decision 4). If it does, the privacy page
gains a row in its table of who the program talks to, and its sentence "There is no server
belonging to this project" (`docs/privacy.md`, line 190) stops being true and has to be
rewritten in the same commit.

**Where keys come from.** Somebody has to sign each key when it is bought. Paddle and Stripe
Managed Payments mint no keys; each sends a signed webhook after a sale, and the seller mints
the key. That means a small service holding the private key and answering the webhook, which
this project does not have. #64 considered a service beside `wixen.app` for form posts, and the
same one could do this. Lemon Squeezy and Gumroad mint keys of their own, checked online through
their APIs. Those are not the offline Ed25519 strings described here, so either the program asks
their servers, or the project mints its own keys from their webhooks anyway. The alternative
while sales are few is Pratik minting each key by hand from a small tool; it costs his time and a
delay for the buyer. This is a cost of decision 3, not a separate decision.

**The trial.** A 60-day trial of pro is decided. The proposal is that it starts in the program
at first run, with no card and no account, which is decision 6. It is a dated marker in the same
credential store: the day the trial began, and the latest day the program has seen. Two things
it cannot prevent, said plainly:

- Setting the machine's clock back does not extend it, because the program counts from the
  latest day it has seen. Setting the clock forward ends it early, and setting the clock right
  again does not bring it back.
- The uninstaller's erase clears the credential store, so an erase and a reinstall start a new
  trial. Keeping the marker out of the erase's reach would break the rule that uninstalling
  clears the secrets by clearing one place. The proposal accepts the reset: a 60-day trial is not
  worth hiding data from somebody who asked for it to be removed.

## 5. The seam in the code

**One question, asked in one place.** On the pattern of `application::allowed`, one module
reads the licence once at start and answers one question where a pro feature starts: is this
feature unlocked? Nothing else in the program reads the licence, parses it or knows its plans.

**Its name.** The issue proposed `Entitlement`. This document proposes a module
`application::licence`, with `Licence` for a checked key and `Unlocked` for the answer, and not
`Entitlement`, for two reasons:

- `type Entitles` and its field `entitle` in `src/presentation/wx_app.rs` (line 13150 on
  2026-09-24) carry the page window's title and have nothing to do with a licence. The compiler
  would accept both names, because the alias is private to that file. People would not: every
  search for one would find the other.
- `grep -rniE 'licen[cs]e key|entitlement' src` is how ALPHA-03 and this plan established that
  nothing knows a licence. Naming the seam `Licence` keeps that search meaning what it has meant.
  After the seam lands, the same search is how somebody finds it.

**A gated command stays where it is.** A control that disappears is one a screen reader user
cannot find out about. So a gated command stays visible, and its label says it is a pro feature
and how to get one. There are two cases, and they need different answers:

- In a menu, the item is greyed. Windows lets the arrow keys land on a greyed item, and a screen
  reader says it is unavailable, so the label is heard.
- In a dialog, a greyed button is not enough. Windows passes over a disabled control when Tab
  moves focus, so somebody moving by keyboard never reaches its label. There the button stays
  enabled and answers with the sentence saying it is a pro feature and how to get it.

Both are the platform's documented behaviour. Neither has been heard with a screen reader for
this purpose, and the phase that builds the seam does that.

**A test holds the line.** The features a licence can unlock are one closed list, an enum, and
the seam answers only about members of it. A test in the tree reads every place that asks the
seam and refuses any inside the modules that carry accessibility: the feedback and earcon code,
the screen reader's announcements, the naming of controls. A second test holds that every
gated place asks through the seam and nowhere else.

**The alpha carries it unlocked.** Once built, the seam can ship before it gates anything, with
every feature unlocked, so the entry, the storage and the check are tried by testers before
anybody pays. When gating starts is decision 9.

## 6. When a licence lapses, or was never bought

Nothing is deleted and nothing becomes unreadable. What stops is what the licence paid for, and
the program says so in a sentence rather than failing without explanation.

- **Extra mail accounts** keep receiving mail, and everything in them can be read, searched and
  exported. After a grace period they stop sending and stop sending changes to the server. The
  account's line in the folder tree and a sentence in the status bar say why and what to do.
  The length of the grace period is decision 10.
- **Imported PGP keys** keep opening mail. What stops is anything beyond reading with one key.
- **Downloaded sound packs** keep playing. What stops is downloading more.
- **Everything free stays free**, including export, so nobody is held by their own data.

**Accounts set up before licences arrive are kept.** A dated rule, written into the code by the
release that introduces licences: every account that existed before that release stays fully
working with no licence. The date follows from decision 9.

## 7. Prices, as decided

Pratik decided these on 2026-09-16, in his comment on #65:

> Pricing decided by Pratik on 2026-09-16: a $10 a year supporter licence; a $19 pro licence
> (yearly); a $99 perpetual pro licence; a 60-day trial of pro.

They are carried here as decided. His same comment left two questions open, and they are
decisions 5 and 6: "whether the perpetual licence carries updates for ever or for a stated
period, which is the usual shape of a perpetual licence for software that keeps shipping; and
whether the trial runs in the program from first run with no card, which needs no merchant
support and is the accessible shape."

## 8. The merchants

Pratik's table, from his comment on #65, carried whole. It was read from each merchant's own
pricing and documentation pages on **2026-09-16**, each claim checked by a second reader against
the page it came from. **The fee pages carry no dates, so read them again before signing
anything.** Fees are for a card sale by a US-based seller; each merchant's international and
PayPal surcharges are added where they apply.

| Merchant | Merchant of record | Fee per sale | Licence keys | What approval needs | Notes |
|---|---|---|---|---|---|
| Paddle (Billing) | Yes | 5% + 50 cents, subscriptions and one-off alike; products under $10 are "contact us" | None built in (deprecated in Billing); signed `transaction.completed` webhook with `custom_data`, so the seller mints his own | A live HTTPS site showing the product, pricing, features, and Terms, Refund Policy and Privacy Policy naming the legal name; ID via Sumsub; individuals skip business verification; manual review 5 to 7 business days | Acceptable-use policy forbids sponsorship or donation-shaped products, so the $10 supporter tier must deliver a real licence. FTC settlement of June 2025 ($5m) over payment processing for deceptive tech-support sellers; no change of ownership. Payouts free domestic, $15 international wire, up to 1.5% conversion. |
| Lemon Squeezy | Yes | 5% + 50 cents, plus 0.5% on subscription payments, plus 1.5% international, plus 1.5% PayPal; payouts 1% outside the US | Built in, with an online validation and activation API; also signed `order_created` and `subscription_payment_success` webhooks with `custom_data` | ID check and a W-9 or W-8, 2 to 3 business days; individuals fine | Owned by Stripe since July 2024. Stripe's own merchant-of-record product, Managed Payments, launched 2025 with "no changes needed" for Lemon Squeezy users and a promised self-serve move; the blog has one 2026 post. A working product in its parent's shadow. |
| Stripe Managed Payments | Yes | 2.9% + 30 cents plus 3.5% for Managed Payments, plus 1.5% international; card-free trials and one-off payments supported since late 2025 | None; `checkout.session.completed` webhook, the seller mints his own | An eligibility review "based on business type and geography"; otherwise Stripe's ordinary onboarding | The technically cleanest, and the one Lemon Squeezy is being folded into. Plain Stripe without Managed Payments is not a merchant of record: the seller registers for and files sales tax and VAT himself (Stripe Tax at 0.5% per transaction helps with the calculation only). |
| Gumroad | Yes, since 2025-01-01 | 10% + 50 cents plus card processing 2.9% + 30 cents (about 13% + 80 cents); 30% on sales through its Discover marketplace | Built in, online verification; webhooks | None: no website, no company | Too expensive at $10 and $19. Open-sourced 2025; new CEO February 2026. |
| FastSpring | Yes | Not published; a revenue share agreed with sales; a vendor risk fee for sellers under $5,000 a year | Generated by hosted scripts, not validated at runtime; HMAC webhooks | Sales conversation | Built for larger sellers; the fee is the unknown. |

His worked cost of one $10 sale, as he wrote it: "Paddle $1.00, Lemon Squeezy $1.05, Stripe
Managed Payments about $1.01, Gumroad about $1.79."

### What one sale costs, worked from the table's fee lines

The figures below are derived here, on 2026-09-24, from the fee column above and nothing else:
a card sale by a US seller, no international or PayPal surcharge, no marketplace sale. The $10
supporter and $19 pro licences are yearly, so Lemon Squeezy's 0.5% on subscription payments is
counted.

| Merchant | $10 sale | $19 sale | Arithmetic |
|---|---|---|---|
| Paddle | $1.00 | $1.45 | 5% + $0.50 |
| Lemon Squeezy | $1.05 | $1.545, about $1.55 | 5% + $0.50 + 0.5% |
| Stripe Managed Payments | $0.94 | $1.516, about $1.52 | 2.9% + $0.30 + 3.5% |
| Gumroad | $2.09 | $3.251, about $3.25 | 10% + $0.50 + 2.9% + $0.30 |
| FastSpring | Unknown | Unknown | Not published |

Two of these disagree with his $10 figures, and the difference is reported rather than chosen
between:

- **Stripe Managed Payments:** the fee line gives $0.94; his comment gives about $1.01. The
  table does not say where the other seven cents come from. Reading the page again settles it.
- **Gumroad:** the fee line gives $2.09; his comment gives about $1.79. The difference is
  exactly the 30 cents of card processing's fixed part, which the $1.79 appears to leave out.

Neither changes the order: Paddle, Lemon Squeezy and Stripe Managed Payments are within a few
cents of each other at both prices, and Gumroad costs about twice as much.

### The recommendation, as it was made

On 2026-09-16 Pratik's comment recommended **Paddle**, undecided. It is carried unchanged, with
the reasons against each merchant as his table gives them:

- **Paddle:** mints no keys, so it needs the webhook service in section 4; its acceptable-use
  policy means the supporter tier must deliver a real licence (decision 7); the FTC settlement
  of June 2025; products under $10 are "contact us", and the supporter licence sits exactly at
  $10.
- **Lemon Squeezy:** owned by Stripe and being folded into Managed Payments; the most
  surcharges; its built-in keys are checked online.
- **Stripe Managed Payments:** mints no keys; approval is an eligibility review whose terms are
  "based on business type and geography".
- **Gumroad:** "Too expensive at $10 and $19"; its keys are checked online.
- **FastSpring:** the fee is not published; built for larger sellers.

## 9. Decisions for Pratik

Every row is Pratik's. The document proposes where the issue or this design proposed something,
and none is answered here. Decisions 1 to 8 come from #65 and the plan for this document; 9 and
10 are raised by this design.

Already settled on #65 on 2026-09-16, and not asked again: the four prices and the 60-day
trial, in section 7.

| # | Decision | Choices | What each costs | Proposed | Pratik's answer |
|---|---|---|---|---|---|
| 1 | The line between free and pro | The table in section 3 as it stands; with changes | Every pro row is a feature somebody without a licence cannot use; every free row is revenue not asked for | Section 3's table, including its two departures from the issue: #62's spoken phrase and #53's import stay free | |
| 2 | Whether several mail accounts are gated at all | Gate a second account, with the grandfather rule; leave several accounts free | Gating takes back what everybody has today and needs section 6's dated rule; leaving it free removes the issue's first and most tangible pro feature | Gate, with the grandfather rule (the issue's proposal) | |
| 3 | The merchant | Paddle; Lemon Squeezy; Stripe Managed Payments; Gumroad; FastSpring | Section 8's tables. Paddle and Stripe need a service to mint keys; Lemon Squeezy and Gumroad check their own keys online | Paddle, as recommended on 2026-09-16 | |
| 4 | Whether revocation is checked online | Never; an occasional check when the program is online | Never: a refunded or leaked key keeps working. A check: a server of the project's own, a row on the privacy page, and the sentence "There is no server belonging to this project" rewritten | Never, at first; revisit if keys leak | |
| 5 | How long a perpetual licence carries updates | For ever; for a stated period, then the version it has keeps working | For ever: $99 once buys every later version. A period: a date in the key, and a sentence when updates stop | Not proposed | |
| 6 | Whether the trial runs from first run with no card | In the program from first run, no card; through the merchant | In the program: an erase and a reinstall restart it (section 4). Through the merchant: a checkout before the first use, and of the five the table records trials only for Stripe Managed Payments | In the program, no card, as his comment calls it the accessible shape | |
| 7 | Whether the $10 supporter tier delivers a real licence | A licence that unlocks something; a licence that unlocks nothing | Paddle's acceptable-use policy forbids donation-shaped products, so under Paddle it must deliver something; the issue proposed it unlock nothing | Not proposed; depends on decision 3 | |
| 8 | How priority support is carried in a feedback report | The report carries the licence's plan and identifier; the report carries nothing and the buyer's address is matched by hand | Carrying it: a line on the privacy page's support row and in the Send Feedback window's list of what goes. Matching by hand: nothing sent, and the ordering is slower and can miss | The report carries the plan and the identifier, shown in the window before Send like every other fact (#64) | |
| 9 | When gating starts | At 1.0.0; at a later release; during the alpha | Earlier: testers lose coverage of what is gated. Later: longer with no revenue. The grandfather date in section 6 follows from this | At 1.0.0, the issue's proposal; the alpha carries the seam unlocked | |
| 10 | How long the grace period after a lapse is | None; a number of days | None: an account stops sending the day a renewal fails. Longer: more time using pro unpaid | 30 days | |

## 10. What happens after the decisions

A phase of its own, planned from the answers above. It touches:

- **Settings:** a Licence section with the key field and its sentence. The key lives in the
  credential store, not in the settings file, so the check that every setting is offered by a
  screen, `test_every_setting_somebody_can_change_is_offered_by_a_screen` in
  `src/data/config.rs`, will not see it; the phase adds a companion naming the key by hand.
- **The privacy page:** what the key holds, that it stays on this computer, and a row for any
  online check decision 4 allows.
- **The first-run screen, `--help` and the user guide:** what is free, what is pro, the trial,
  and what a lapse does, in the same words.
- **`application::forget`:** the licence's service name, so the erase removes it.
- **Every gated feature's own plan:** each asks the seam where it starts, and each is listed in
  the enum section 5 describes.
- **#64's priority support:** the report carries what decision 8 says, and the receiving end
  orders by it. Every report is still answered.

The version at which gating starts is decision 9. Until then, nothing in the program knows
about a licence, and this document is where the work waits.
