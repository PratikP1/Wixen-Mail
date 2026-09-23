//! What gets baked into the executable before the code is compiled: the
//! manifest, the icon, and the name Windows shows people.

/// What the application is called, everywhere a person can see it.
///
/// Not `wixen-mail`. That is the crate, the executable, the data folder and
/// the log prefix, and it is right in all of those, but it is a machine name
/// and it was leaking into places people read: Task Manager's list of running
/// programs, the elevation prompt, and the Details tab of the file's
/// properties. A screen reader reads out what is there, so it was announcing
/// a hyphenated file name to the people this application exists for.
#[cfg(target_os = "windows")]
const PRODUCT: &str = "Wixen Mail";

fn main() {
    // Which commit this build came from, when it was built by
    // scripts/build-installer.sh. Empty for an ordinary `cargo build`, so a
    // development build does not relink every time the working tree changes.
    // See src/common/version.rs for why it is worth carrying at all.
    let build = std::env::var("WIXEN_BUILD").unwrap_or_default();
    println!("cargo:rustc-env=WIXEN_BUILD={build}");
    println!("cargo:rerun-if-env-changed=WIXEN_BUILD");

    // Embed the Windows application manifest that declares Common Controls v6.
    // This silences the wxWidgets manifest warning and enables modern UI controls.
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file("wixen-mail.exe.manifest");
        // The executable had no icon at all, so Windows drew the generic one
        // everywhere: the taskbar, Alt+Tab, the shortcut, and Apps and
        // Features, where the installer points at this file for its own.
        // Generated from assets/icon.svg by scripts/make-icon.py.
        res.set_icon("assets/icon.ico");
        println!("cargo:rerun-if-changed=assets/icon.ico");

        // Left to itself, winresource fills these in from the crate name, so
        // both of the fields Windows shows in a list said "wixen-mail".
        //
        // FileDescription is the one that matters most: it is the name in Task
        // Manager and the "Program name" on the elevation prompt. The two that
        // keep the machine name are the ones meant to hold it, and neither is
        // shown as a label.
        res.set("ProductName", PRODUCT);
        res.set("FileDescription", PRODUCT);
        res.set("CompanyName", "Pratik Patel");
        // In step with LICENSE and the About dialog, which read it from
        // `application::about::COPYRIGHT`; a build script cannot reach that
        // constant, so `tests/the_about_dialog_names_its_owners_and_its_links.rs`
        // holds this line to it instead.
        res.set(
            "LegalCopyright",
            "Copyright (c) 2024-2026 Pratik Patel and the Wixen Project, with other contributors. MIT licensed.",
        );
        res.set("OriginalFilename", "wixen-mail.exe");
        res.set("InternalName", "wixen-mail");

        if let Err(e) = res.compile() {
            // Non-fatal: the application still runs, just with a deprecation warning.
            eprintln!("cargo:warning=Failed to embed manifest: {}", e);
        }
    }
}
