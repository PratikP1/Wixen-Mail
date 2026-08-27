//! Loading the handler library and asking it to register itself.
//!
//! The setup tool is a separate program from the library the indexer loads, and
//! that separation has to be respected here. The library writes its own path
//! into the registry by asking Windows which file the running code came from, so
//! a tool that called `DllRegisterServer` through the linked library would ask
//! about *itself* and register the setup tool as the protocol handler. The
//! indexer would then load a program that is not a COM server and fail on every
//! item, with nothing anywhere to say why.
//!
//! So the tool does what `regsvr32` does: loads the library by name and calls
//! the function it exports. That also proves something worth proving. If the
//! library is missing a dependency or was built for another processor, this is
//! where it shows, with a message, rather than on a machine where the indexer
//! silently never loads it.

use std::path::{Path, PathBuf};
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows_core::{HRESULT, HSTRING, PCSTR};

/// The name of the library an install puts beside the tool.
pub const LIBRARY_FILE: &str = "wixen_mail_search.dll";

/// The two functions every in-process COM server exports.
///
/// Written as bytes with their own terminator because `GetProcAddress` takes a
/// narrow string, and a name without a terminator would have Windows read past
/// the end of the literal looking for one.
const REGISTER: &[u8] = b"DllRegisterServer\0";
const UNREGISTER: &[u8] = b"DllUnregisterServer\0";

/// Why the library could not be asked to change the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    /// The file is not where the tool was told to look.
    NotThere(PathBuf),
    /// Windows would not load it.
    ///
    /// Almost always a missing dependency or the wrong processor architecture,
    /// and the code says which.
    WillNotLoad(PathBuf, HRESULT),
    /// It loaded and has no such function in it, so it is not this handler.
    NotAHandler(&'static str),
    /// The function ran and reported a failure.
    Refused(HRESULT),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotThere(path) => write!(f, "there is no library at {}", path.display()),
            Self::WillNotLoad(path, code) => write!(
                f,
                "Windows would not load {} (code {:#010X}). A missing dependency or a \
                 library built for another processor would both look like this.",
                path.display(),
                code.0
            ),
            Self::NotAHandler(name) => write!(
                f,
                "that library has no {name} in it, so it is not the Wixen Mail search handler"
            ),
            Self::Refused(code) => write!(
                f,
                "the library could not write its registry entries (code {:#010X}). It \
                 reports every failure the same way, and the usual one is not having \
                 administrator rights, so try an administrator prompt first.",
                code.0
            ),
        }
    }
}

/// Ask the library to write its registry entries.
pub fn register(path: &Path) -> Result<(), LibraryError> {
    call(path, REGISTER, "DllRegisterServer")
}

/// Ask the library to take them out again.
pub fn unregister(path: &Path) -> Result<(), LibraryError> {
    call(path, UNREGISTER, "DllUnregisterServer")
}

/// Load the library, call one of its two registration functions, let it go.
fn call(path: &Path, export: &[u8], named: &'static str) -> Result<(), LibraryError> {
    if !path.is_file() {
        return Err(LibraryError::NotThere(path.to_path_buf()));
    }

    let full = HSTRING::from(path.as_os_str());
    let library = unsafe { LoadLibraryW(&full) }
        .map_err(|e| LibraryError::WillNotLoad(path.to_path_buf(), e.code()))?;
    let found = unsafe { GetProcAddress(library, PCSTR(export.as_ptr())) };

    let outcome = match found {
        None => Err(LibraryError::NotAHandler(named)),
        Some(address) => {
            // Every in-process COM server exports these two with exactly this
            // shape. A library exporting the name with a different shape is not
            // a COM server, and nothing in a DLL would let this check that,
            // here or in regsvr32.
            let run: extern "system" fn() -> HRESULT = unsafe { std::mem::transmute(address) };
            // Called once and the answer kept. Calling again to look at the
            // failure would write every registry entry a second time.
            let said = run();
            match said.is_ok() {
                true => Ok(()),
                false => Err(LibraryError::Refused(said)),
            }
        }
    };

    // Safe to let go: registration hands out no objects, so nothing is left
    // pointing into this library once the call has returned.
    unsafe {
        let _ = FreeLibrary(library);
    };

    outcome
}

/// Where the library is when nobody said.
///
/// Beside the tool, which is where an install puts both of them. Worked out from
/// the running program rather than from the current directory, so running the
/// tool from somewhere else still finds its own library.
pub fn beside_this_program() -> Option<PathBuf> {
    let program = std::env::current_exe().ok()?;

    Some(program.parent()?.join(LIBRARY_FILE))
}
