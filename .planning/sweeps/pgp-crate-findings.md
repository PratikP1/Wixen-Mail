# OpenPGP crate: what the web says, gathered 2026-09-05

Gathered while 04-05 was executing, so 04-09's blocking checkpoint does not
start from nothing. This is not the audit table. Task 2 still writes its
criteria first, checks transitive cost against `Cargo.lock`, and produces the
table. This is corroboration to check that table against.

## The criterion the plan left out

`04-09-PLAN.md` lists four criteria for the comparison: transitive cost against
`Cargo.lock`, whether a C library or toolchain beyond cargo is needed, whether
it builds on Windows without setup, and release history. It never mentions the
licence. The licence is what decides this one, so the criteria want widening
before task 2 evaluates anything, or the comparison will reach its answer for
the wrong reasons and the document will not say why.

Wixen Mail is `license = "MIT"` in `Cargo.toml`, and it ships a Windows
installer, so it distributes a statically linked binary.

## The two candidates

### sequoia-openpgp 2.4.1

- **Licence LGPL-2.0-or-later.** Against an MIT project shipping a statically
  linked installer, that carries a relinking obligation MIT does not. This is a
  distribution constraint rather than a code one and it is not a call to make
  quietly.
- Default crypto backend is Nettle, a C library. It has other backends,
  including a pure Rust one and a Windows CNG one, so the C dependency is
  avoidable, but the default is not what this project wants.
- 1.80M total downloads, 343k recent. 60 published versions. Repository on
  GitLab at `sequoia-pgp/sequoia`.
- Releases: 2.1.0 Nov 2025, 2.2.0 Feb 2026, 2.3.0 May 2026, 2.4.0 Jul 2026,
  2.4.1 Jul 2026.

### pgp 0.20.0, the rPGP project

- **Licence MIT OR Apache-2.0.** Matches this project.
- Pure Rust. No C library, no toolchain beyond cargo. That matters here because
  `ring` is currently the only thing in this tree near cryptography, and the
  installer build should not gain a C dependency.
- 6.00M total downloads, 1.59M recent, so roughly four times sequoia's use.
- Repository `github.com/rpgp/rpgp`.
- Releases, stable and regular: 0.13.2 Aug 2024, 0.14.0 Sep 2024, 0.14.1 and
  0.14.2 Dec 2024, 0.15.0 Jan 2025, 0.16.0 May 2025, 0.17.0 Sep 2025, 0.18.0
  Nov 2025, 0.19.0 Feb 2026, 0.20.0 Jun 2026. Roughly quarterly.
- **It passes the `x509-parser` test written in this project's own
  `Cargo.toml`.** That comment refuses a crate for having only ever published
  pre-releases. 0.16.0 had four alphas and then a stable release, and every
  other version listed is stable, so the objection does not apply.
- Two independent security audits: Include Security in 2019, funded by OTF,
  report published at `delta.chat/assets/1907-otf-deltachat-rpgp-rustrsa-gb-reportv1.pdf`;
  and Radically Open Security in December 2024, commissioned by NLnet.
- In production in Delta Chat on Windows, Linux, macOS, Android and iOS, 32 and
  64 bit. Windows is the platform this project cares about and it is covered.
- Implements RFC 9580, v4 and v6 key formats, AEAD.
- Its own README says the API documentation is thin. That is a cost to the
  executor's time rather than a reason to refuse it, but it should be in the
  comparison honestly.

## What still has to be measured rather than read

- What `pgp 0.20` pulls in transitively and how much of that is already in
  `Cargo.lock`. Not checked here. It is pure Rust, so it will bring RustCrypto
  crates, and some of those may already be present through `ring` or
  `x509-parser`. The audit table needs the real diff.
- Whether the needed surface, reading an armoured message, importing an
  armoured private key, decrypting to it, and telling no key from wrong key
  from damaged, is actually covered. The last of those is the one to check
  rather than assume, because it is the difference between a useful error and
  one error string.

## Sources

- https://crates.io/crates/pgp
- https://crates.io/crates/sequoia-openpgp
- https://github.com/rpgp/rpgp
- https://delta.chat/en/2025-08-04-encryption-v2
- https://delta.chat/assets/1907-otf-deltachat-rpgp-rustrsa-gb-reportv1.pdf
