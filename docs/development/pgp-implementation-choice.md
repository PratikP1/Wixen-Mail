# Which OpenPGP implementation this project uses

Written 2026-09-06, for phase 4 plan 09. This is the record of a choice, so it
names the candidate that was refused and why, not only the one that was kept.

**Nothing is in `Cargo.toml` yet.** This project requires a person to look at a
package before it is added, and that check is answered before the dependency
exists rather than after it has been building for a week. This document is what
that person is given.

## What the choice is for

Reading a PGP-encrypted message that somebody on this computer holds the private
key for. That is criterion 5 of phase 4 and the first `[D]` line of READ-02.

## The criteria, written before any candidate was looked at

Written first on purpose. Criteria shaped by whatever the first candidate
happens to do are not criteria, they are a description.

**What is needed.** Read an ASCII-armoured message. Import an armoured private
key and hold it. Decrypt a message encrypted to that key. Tell apart three
outcomes: there is no key here, the key here does not open this message, and the
message is damaged.

**What is not needed.** Signing, encrypting, key generation, key servers, web of
trust, revocation checking.

**What each candidate is judged on.** Seven things. The first is not in the plan
that asked for this document and it is the one that decides the answer:

1. **Licence, against this project's own.** Wixen Mail is MIT and ships a
   statically linked Windows installer, so it distributes a combined binary. A
   dependency whose licence puts obligations on that binary is a decision about
   how the whole product may be distributed, not a decision about a library.
2. Transitive cost, measured against `Cargo.lock` rather than counted in the
   abstract, because a crate this tree already builds costs nothing to add.
3. Whether a C library or a build toolchain beyond `cargo` is needed. This
   project ships a Windows installer built on the MSVC toolchain and `ring` is
   currently the only thing in the tree near this territory.
4. Whether it builds on Windows without extra setup, on the toolchain this
   project actually uses.
5. Release history. `Cargo.toml`'s comment over `ring` refuses the `cms` crate
   for having only ever published pre-release versions, and says why: a
   pre-release is not something to put under a security decision people read as
   an answer. The same reasoning applies here.
6. How much of the needed surface it covers, against how much of its own surface
   comes along unused.
7. What is known about its security, including what it says about itself.

The licence criterion was added after the plan was written. Without it the
comparison reaches the same answer for incomplete reasons, which is worse than
reaching the wrong one, because nobody can tell later which reasons were load
bearing.

## The two candidates

### `pgp` 0.20.0, the rPGP project. Chosen.

| | |
|---|---|
| Licence | MIT OR Apache-2.0 |
| Repository | https://github.com/rpgp/rpgp |
| Downloads | 6.0M total, 1.59M recent |
| Latest stable | 0.20.0, 2026-06-23 |
| First published | 2017-09-16 |
| Minimum Rust | 1.88 |
| C library needed | none |
| New crates in this tree | 78 |

**Licence.** MIT OR Apache-2.0, which is this project's own licence and adds no
obligation to the shipped binary.

**Transitive cost.** 143 crates resolve in total and 78 of them are new to this
tree, measured against `Cargo.lock` on 2026-09-06. 45 of the 78 belong to the
RustCrypto organisation and 4 to dalek-cryptography. Every one of the 78 is
permissively licensed: 73 under some combination of MIT and Apache-2.0, 4 under
BSD-3-Clause, and `libbz2-rs-sys` under the bzip2 licence, which is a BSD-style
permissive licence. None is copyleft. The full list, with versions, licences,
download counts and repositories, is in the audit table below.

Turning off the one default feature that can be turned off, `bzip2`, saves two
crates and takes the new count from 78 to 76. That is not worth doing. A message
compressed with bzip2 is legal OpenPGP, and without the feature it would be
reported as damaged, which is the reader being told something false rather than
something incomplete.

**Build requirements.** Pure Rust. No C library, no `pkg-config`, no toolchain
beyond `cargo`. This matters more than it usually would because the Windows
installer is built by `scripts/build-installer.sh` and a C dependency is a
second build system in that path.

**Windows.** rPGP's own `docs/PLATFORMS.md` marks `x86_64-pc-windows-gnu` as
checked in CI and `x86_64-pc-windows-msvc` as "should work, but not in CI". This
project builds `x86_64-pc-windows-msvc`. So the toolchain this project ships on
is not one rPGP tests, which is a real gap and is stated here rather than
smoothed over. What stands against it is that Delta Chat ships rPGP on Windows
to a large number of people, and that pure Rust code has far less to go wrong
between the two Windows toolchains than a C library would.

**Release history.** Stable releases roughly quarterly: 0.13.2 in August 2024,
then 0.14.0, 0.14.1, 0.14.2, 0.15.0, 0.16.0, 0.17.0, 0.18.0, 0.19.0 and 0.20.0
in June 2026. 0.16.0 had four alpha releases before its stable one. Every other
version listed is stable, so the objection `Cargo.toml` raises against `cms`,
that a crate has only ever published pre-releases, does not apply.

**Surface.** Larger than what is needed. rPGP implements RFC 9580 in full,
including signing, key generation and v6 keys, none of which this project asks
for. That is the ordinary cost of using a complete implementation rather than
writing a partial one, and it is the trade `Cargo.toml`'s comment over `ring`
already made for signature checking.

**Security.** Three independent reviews, listed in rPGP's own
`docs/SECURITY_STATUS.md`:

- December 2024, Radically Open Security, commissioned by NLnet. Two advisories
  came out of it: CVE-2024-53856, panics on malformed untrusted input, and
  CVE-2024-53857, resource exhaustion on untrusted messages. Both are fixed.
  Both are exactly the failure mode a mail client cares about, since every
  message it opens is untrusted input.
- March 2024, a security analysis by ETH researchers, through Delta Chat.
- 2019, Include Security, funded by the Open Technology Fund. No critical flaws.

**What rPGP says is wrong with it, said here because it is a cost and not a
footnote.** Its `SECURITY_STATUS.md` records that the `rsa` crate it depends on
is vulnerable to the Marvin attack, a timing side channel, and that this is
being tracked upstream at RustCrypto/RSA issue 19. `rsa` is one of the 78 crates
below. For this project's use, reading a message on a machine the reader
controls, a timing attack needs an attacker who can measure decryption timings
on that machine, which is a much weaker position than the network attacker the
attack is usually described against. It is still a known unfixed weakness in a
cryptographic dependency and it belongs in the record.

It also uses `sha1-checked` rather than plain `sha1` for fingerprints and
signature hashes, which mitigates the known practical SHA-1 collision attacks.

### `sequoia-openpgp` 2.4.1. Refused.

| | |
|---|---|
| Licence | LGPL-2.0-or-later |
| Repository | https://gitlab.com/sequoia-pgp/sequoia |
| Downloads | 1.80M total, 345k recent |
| Latest stable | 2.4.1, 2026-07-09 |
| Default crypto backend | Nettle, a C library |
| New crates in this tree | 16 with Nettle, 70 with the pure Rust backend |

**Refused on the licence.** LGPL-2.0-or-later against an MIT project that ships
a statically linked binary. Static linking against an LGPL library carries an
obligation to let the recipient relink the program against a modified version of
that library, which usually means shipping object files or an equivalent. MIT
carries no such obligation, so taking this on would change how Wixen Mail may be
distributed. That is a decision about the product, and it is not one to make
quietly in a dependency line.

**It is a good implementation and it loses on fit, not on quality.** Sequoia is
mature, well regarded, and its transitive cost in its default configuration is
the lowest of anything measured here: 70 crates in total, of which only 16 are
new to this tree, against rPGP's 78.

**That advantage disappears in the configuration this project could actually
use.** The 16 include `nettle` and `nettle-sys`, which need the Nettle C
library, GMP, and `pkg-config` present at build time. That is the second build
system this project's installer path must not gain. Sequoia does offer a pure
Rust backend, and switching to it was measured: the tree grows to 155 crates of
which **70 are new**, which is within a few crates of rPGP's 78. So the cheaper
number belongs to the configuration with a C library in it, and the C-free
configuration costs the same as the alternative.

**The pure Rust backend also has to be asked for by name in a way worth
reading.** It is enabled with `crypto-rust` alongside `allow-experimental-crypto`
and `allow-variable-time-crypto`. Those two feature names are the maintainers
saying what they think of it. A mail client is not the place to turn on
variable-time cryptography over a stranger's bytes.

**Release history.** Stable and regular: 1.21.2 in July 2024, 1.22.0 in December
2024, 2.0.0 in March 2025, then 2.1.0, 2.2.0, 2.3.0, 2.4.0 and 2.4.1 in July
2026. No objection here.

## The decision

`pgp` 0.20, the rPGP project.

The licence decides it. Everything else is consistent with that answer rather
than the reason for it: pure Rust suits an installer that should not gain a C
toolchain, the transitive cost is within a few crates of the alternative once
the alternative is put in a configuration this project could ship, and the
security review history is the stronger of the two.

If the legitimacy check refuses a crate that rPGP brings, the next candidate is
`sequoia-openpgp` with the Nettle backend, and taking it means answering the
licence question first, deliberately, as a decision about how Wixen Mail is
distributed.

## The credential store name a private key lives under

`wixen-mail-pgp`, with the account name `private-key`. It is defined once in
`src/service/pgp/mod.rs` as `KEYRING_SERVICE` and `KEYRING_PRIVATE_KEY`.

**The name is permanent from the commit that wrote it.** Changing it orphans a
private key on every machine that has one: the code that erases secrets names
its entries by this string, so a renamed service leaves the old entry behind,
unreadable, belonging to nothing, and invisible to the uninstaller.

It is registered in `application::forget::entries_for`, which is the whole list
of credential store entries an uninstall erases. That function's own comment
records what happens when two lists of the same entries are kept apart: a
removed account left its refresh token on the machine and the sweep never saw it
again. `application::forget` now also carries a guard that fails when a module
under `src/service/` owns credential store entries and is not named in that
list, which was not true before.

## Package legitimacy audit

Every crate `pgp` 0.20 brings that is not already in this project's
`Cargo.lock`, measured on 2026-09-06 with default features and normal
dependency edges only. 78 crates. Download counts and repositories are from
`crates.io`; licences and versions are from the resolved package manifests.

Every one of these is `[ASSUMED]` under this project's rules, because none of
them is in this tree today. None is `[SLOP]`: every crate resolves to a real
repository, every one has a download history consistent with its age, and no
name is a near-miss for a more popular crate.

**The four worth looking at first**, because they are the only ones under ten
million downloads or outside a well-known organisation:

| Crate | Downloads | Why it stands out |
|---|---|---|
| `cx448` | 1.2M | X448 curve support, published by `dignifiedquire`, who is the rPGP maintainer. A first-party crate of the author of the crate being audited, so it inherits rPGP's trust rather than adding independent evidence. |
| `bitfields` | 2.3M | Bit-field derive macro by `gregorygaines`. Not a cryptographic crate and not part of a known organisation. |
| `bitfields-impl` | 2.3M | The proc-macro half of the above, same repository. |
| `ocb3` | 4.5M | RustCrypto AEAD implementation. Low downloads for RustCrypto, which reflects OCB3 being rare rather than the crate being obscure. |

**`rsa` is on the list and carries a known unfixed weakness**, described above:
the Marvin timing attack, acknowledged by both RustCrypto and rPGP and tracked
upstream. Its download count is 214M and its provenance is not in question. It
is named here so the weakness is a decision rather than a surprise.

### The full list

| Crate | Version | Licence | Downloads | Repository |
|---|---|---|---|---|
| `aes-kw` | 0.2.1 | MIT OR Apache-2.0 | 7.9M | https://github.com/RustCrypto/key-wraps/tree/aes-kw |
| `argon2` | 0.5.3 | MIT OR Apache-2.0 | 50.5M | https://github.com/RustCrypto/password-hashes/tree/master/argon2 |
| `base16ct` | 0.2.0 | Apache-2.0 OR MIT | 226.7M | https://github.com/RustCrypto/formats/tree/master/base16ct |
| `base64ct` | 1.8.3 | Apache-2.0 OR MIT | 386.8M | https://github.com/RustCrypto/formats |
| `bitfields` | 1.0.3 | MIT | 2.3M | https://github.com/gregorygaines/bitfields-rs |
| `bitfields-impl` | 1.0.3 | MIT | 2.3M | https://github.com/gregorygaines/bitfields-rs |
| `bitvec` | 1.1.1 | MIT | 272.1M | https://github.com/bitvecto-rs/bitvec |
| `blake2` | 0.10.6 | MIT OR Apache-2.0 | 159.2M | https://github.com/RustCrypto/hashes |
| `blowfish` | 0.9.1 | MIT OR Apache-2.0 | 43.9M | https://github.com/RustCrypto/block-ciphers |
| `buffer-redux` | 1.1.0 | MIT OR Apache-2.0 | 6.4M | https://github.com/dignifiedquire/buffer-redux |
| `bzip2` | 0.6.1 | MIT OR Apache-2.0 | 159.3M | https://github.com/trifectatechfoundation/bzip2-rs |
| `camellia` | 0.1.0 | MIT OR Apache-2.0 | 7.9M | https://github.com/RustCrypto/block-ciphers |
| `cast5` | 0.11.1 | MIT OR Apache-2.0 | 7.0M | https://github.com/RustCrypto/block-ciphers |
| `cfb-mode` | 0.8.2 | MIT OR Apache-2.0 | 16.2M | https://github.com/RustCrypto/block-modes |
| `cmac` | 0.7.2 | MIT OR Apache-2.0 | 20.9M | https://github.com/RustCrypto/MACs |
| `convert_case` | 0.10.0 | MIT | 509.2M | https://github.com/rutrum/convert-case |
| `crc24` | 0.1.6 | MIT/Apache-2.0 | 6.9M | https://github.com/sellibitze/crc24-rs.git |
| `crypto-bigint` | 0.5.5 | Apache-2.0 OR MIT | 266.7M | https://github.com/RustCrypto/crypto-bigint |
| `curve25519-dalek` | 4.1.3 | BSD-3-Clause | 247.9M | https://github.com/dalek-cryptography/curve25519-dalek/tree/main/curve25519-dalek |
| `curve25519-dalek-derive` | 0.1.1 | MIT/Apache-2.0 | 175.3M | https://github.com/dalek-cryptography/curve25519-dalek |
| `cx448` | 0.1.1 | BSD-3-Clause | 1.2M | https://github.com/dignifiedquire/cx448 |
| `darling` | 0.20.11 | MIT | 700.4M | https://github.com/TedDriggs/darling |
| `darling_core` | 0.20.11 | MIT | 700.5M | https://github.com/TedDriggs/darling |
| `darling_macro` | 0.20.11 | MIT | 700.6M | https://github.com/TedDriggs/darling |
| `dbl` | 0.3.2 | MIT OR Apache-2.0 | 21.1M | https://github.com/RustCrypto/utils |
| `der` | 0.7.10 | Apache-2.0 OR MIT | 443.4M | https://github.com/RustCrypto/formats/tree/master/der |
| `derive_builder` | 0.20.2 | MIT OR Apache-2.0 | 191.6M | https://github.com/colin-kiegel/rust-derive-builder |
| `derive_builder_core` | 0.20.2 | MIT OR Apache-2.0 | 191.6M | https://github.com/colin-kiegel/rust-derive-builder |
| `derive_builder_macro` | 0.20.2 | MIT OR Apache-2.0 | 182.6M | https://github.com/colin-kiegel/rust-derive-builder |
| `des` | 0.8.1 | MIT OR Apache-2.0 | 33.5M | https://github.com/RustCrypto/block-ciphers |
| `dsa` | 0.6.3 | Apache-2.0 OR MIT | 7.3M | https://github.com/RustCrypto/signatures/tree/master/dsa |
| `eax` | 0.5.0 | Apache-2.0 OR MIT | 5.2M | https://github.com/RustCrypto/AEADs |
| `ecdsa` | 0.16.9 | Apache-2.0 OR MIT | 217.3M | https://github.com/RustCrypto/signatures/tree/master/ecdsa |
| `ed25519` | 2.2.3 | Apache-2.0 OR MIT | 215.1M | https://github.com/RustCrypto/signatures/tree/master/ed25519 |
| `ed25519-dalek` | 2.2.0 | BSD-3-Clause | 204.0M | https://github.com/dalek-cryptography/curve25519-dalek/tree/main/ed25519-dalek |
| `elliptic-curve` | 0.13.8 | Apache-2.0 OR MIT | 221.0M | https://github.com/RustCrypto/traits/tree/master/elliptic-curve |
| `ff` | 0.13.1 | MIT/Apache-2.0 | 227.0M | https://github.com/zkcrypto/ff |
| `funty` | 2.0.0 | MIT | 261.6M | https://github.com/myrrlyn/funty |
| `group` | 0.13.0 | MIT/Apache-2.0 | 223.4M | https://github.com/zkcrypto/group |
| `idea` | 0.5.1 | MIT OR Apache-2.0 | 12.3M | https://github.com/RustCrypto/block-ciphers |
| `ident_case` | 1.0.1 | MIT/Apache-2.0 | 449.5M | https://github.com/TedDriggs/ident_case |
| `k256` | 0.13.4 | Apache-2.0 OR MIT | 76.9M | https://github.com/RustCrypto/elliptic-curves/tree/master/k256 |
| `keccak` | 0.1.6 | Apache-2.0 OR MIT | 149.6M | https://github.com/RustCrypto/sponges/tree/master/keccak |
| `libbz2-rs-sys` | 0.2.5 | bzip2-1.0.6 | 35.0M | https://github.com/trifectatechfoundation/libbzip2-rs |
| `md-5` | 0.10.6 | MIT OR Apache-2.0 | 341.2M | https://github.com/RustCrypto/hashes |
| `num-bigint-dig` | 0.8.6 | MIT/Apache-2.0 | 193.4M | https://github.com/dignifiedquire/num-bigint |
| `ocb3` | 0.1.0 | Apache-2.0 OR MIT | 4.5M | https://github.com/RustCrypto/AEADs |
| `opaque-debug` | 0.3.1 | MIT OR Apache-2.0 | 321.9M | https://github.com/RustCrypto/utils |
| `p256` | 0.13.2 | Apache-2.0 OR MIT | 175.5M | https://github.com/RustCrypto/elliptic-curves/tree/master/p256 |
| `p384` | 0.13.1 | Apache-2.0 OR MIT | 84.1M | https://github.com/RustCrypto/elliptic-curves/tree/master/p384 |
| `p521` | 0.13.3 | Apache-2.0 OR MIT | 29.3M | https://github.com/RustCrypto/elliptic-curves/tree/master/p521 |
| `password-hash` | 0.5.0 | MIT OR Apache-2.0 | 116.5M | https://github.com/RustCrypto/traits/tree/master/password-hash |
| `pem-rfc7468` | 0.7.0 | Apache-2.0 OR MIT | 309.4M | https://github.com/RustCrypto/formats/tree/master/pem-rfc7468 |
| `pgp` | 0.20.0 | MIT OR Apache-2.0 | 6.0M | https://github.com/rpgp/rpgp |
| `pkcs1` | 0.7.5 | Apache-2.0 OR MIT | 205.8M | https://github.com/RustCrypto/formats/tree/master/pkcs1 |
| `pkcs8` | 0.10.2 | Apache-2.0 OR MIT | 393.1M | https://github.com/RustCrypto/formats/tree/master/pkcs8 |
| `primeorder` | 0.13.6 | Apache-2.0 OR MIT | 122.9M | https://github.com/RustCrypto/elliptic-curves/tree/master/primeorder |
| `radium` | 0.7.0 | MIT | 267.4M | https://github.com/bitvecto-rs/radium |
| `replace_with` | 0.1.8 | MIT OR Apache-2.0 | 21.9M | https://github.com/alecmocatta/replace_with |
| `rfc6979` | 0.4.0 | Apache-2.0 OR MIT | 210.2M | https://github.com/RustCrypto/signatures/tree/master/rfc6979 |
| `ripemd` | 0.1.3 | MIT OR Apache-2.0 | 47.2M | https://github.com/RustCrypto/hashes |
| `rsa` | 0.9.10 | MIT OR Apache-2.0 | 214.3M | https://github.com/RustCrypto/RSA |
| `sec1` | 0.7.3 | Apache-2.0 OR MIT | 216.5M | https://github.com/RustCrypto/formats/tree/master/sec1 |
| `serdect` | 0.3.0 | Apache-2.0 OR MIT | 44.9M | https://github.com/RustCrypto/formats |
| `sha1` | 0.10.7 | MIT OR Apache-2.0 | 497.3M | https://github.com/RustCrypto/hashes |
| `sha1-checked` | 0.10.0 | MIT OR Apache-2.0 | 30.2M | https://github.com/RustCrypto/hashes |
| `sha3` | 0.10.9 | MIT OR Apache-2.0 | 161.6M | https://github.com/RustCrypto/hashes |
| `signature` | 2.2.0 | Apache-2.0 OR MIT | 421.2M | https://github.com/RustCrypto/traits/tree/master/signature |
| `snafu` | 0.9.2 | MIT OR Apache-2.0 | 107.1M | https://github.com/shepmaster/snafu |
| `snafu-derive` | 0.9.2 | MIT OR Apache-2.0 | 107.2M | https://github.com/shepmaster/snafu |
| `spin` | 0.9.9 | MIT | 641.6M | https://github.com/mvdnes/spin-rs.git |
| `spki` | 0.7.3 | Apache-2.0 OR MIT | 408.0M | https://github.com/RustCrypto/formats/tree/master/spki |
| `strsim` | 0.11.1 | MIT | 1047.6M | https://github.com/rapidfuzz/strsim-rs |
| `twofish` | 0.7.1 | MIT OR Apache-2.0 | 7.8M | https://github.com/RustCrypto/block-ciphers |
| `unicode-xid` | 0.2.6 | MIT OR Apache-2.0 | 594.9M | https://github.com/unicode-rs/unicode-xid |
| `wyz` | 0.5.1 | MIT | 261.4M | https://github.com/myrrlyn/wyz |
| `x25519-dalek` | 2.0.1 | BSD-3-Clause | 70.9M | https://github.com/dalek-cryptography/curve25519-dalek/tree/main/x25519-dalek |
| `zeroize_derive` | 1.5.0 | Apache-2.0 OR MIT | 248.0M | https://github.com/RustCrypto/utils |

## One thing that has to change in `Cargo.toml` besides the dependency line

This project declares `rust-version = "1.87"`. rPGP 0.20 declares 1.88, so
adding it raises this project's minimum Rust version to 1.88. The toolchain in
use is 1.97.1, so nothing fails to build, but the declared floor would be wrong
and has to move in the same commit.

## How these numbers were measured

The counts and licences come from resolving `pgp = "0.20"` and
`sequoia-openpgp = "2.4"` in throwaway crates outside this repository, with
`cargo tree --edges normal` and `cargo metadata`. Nothing was compiled and no
build script ran. The "new to this tree" figure is the resolved set minus every
crate name in this project's `Cargo.lock`. Download counts come from the
`crates.io` API on 2026-09-06.

Re-measure rather than quoting these if the question comes up again after any
version moves. This project already has ten plans that were misled by quoting a
number somebody else measured.
