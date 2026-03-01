fn main() {
    // Embed the Windows application manifest that declares Common Controls v6.
    // This silences the wxWidgets manifest warning and enables modern UI controls.
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file("wixen-mail.exe.manifest");
        if let Err(e) = res.compile() {
            // Non-fatal: the application still runs, just with a deprecation warning.
            eprintln!("cargo:warning=Failed to embed manifest: {}", e);
        }
    }
}
