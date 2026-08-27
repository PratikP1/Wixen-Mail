//! The four functions Windows looks for by name in an in-process COM server.
//!
//! Registration writes only what [`crate::registration`] lists, so what goes
//! into somebody's registry can be read and reviewed without reading this file.
//! Everything goes under `HKEY_LOCAL_MACHINE`, which means `regsvr32` has to be
//! run from an elevated prompt.
//!
//! Registration stops at the first entry it cannot write and reports a failure.
//! It does not undo what it wrote before that, so a run without administrator
//! rights leaves nothing behind only because the very first entry fails. A run
//! that fails part way through, which is what a permissions problem partway
//! down would look like, leaves some keys written. Running
//! `regsvr32 /u` afterwards clears them, and it is written to keep going past a
//! key that was never there so that it can.

use super::factory::{Class, ClassFactory};
use super::{is_success, nothing_is_in_use};
use crate::registration::{
    Entry, FILTER_CLSID_VALUE, Hive, PROTOCOL_CLSID_VALUE, entries, keys_to_remove, value_to_remove,
};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, E_FAIL, E_POINTER, HMODULE, MAX_PATH, S_FALSE, S_OK,
};
use windows::Win32::System::Com::IClassFactory;
use windows::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteTreeW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows_core::{GUID, HRESULT, HSTRING, Interface, PCWSTR};

/// Hand COM the object that makes one of our classes.
///
/// # Safety
///
/// Called by COM with pointers it owns. Both are checked before use.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() || rclsid.is_null() || riid.is_null() {
        return E_POINTER;
    }
    unsafe { *ppv = std::ptr::null_mut() };

    let asked_for = unsafe { *rclsid };
    let class = if asked_for == GUID::from_u128(PROTOCOL_CLSID_VALUE) {
        Class::Protocol
    } else if asked_for == GUID::from_u128(FILTER_CLSID_VALUE) {
        Class::Filter
    } else {
        return CLASS_E_CLASSNOTAVAILABLE;
    };

    let factory: IClassFactory = ClassFactory::new(class).into();
    unsafe { factory.query(riid, ppv) }
}

/// Whether COM may unload this DLL.
///
/// `S_OK` means yes. Answering yes while an object is still alive unloads the
/// code out from under a live pointer in the indexer's process, so the count
/// has to be right in both directions.
#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    match nothing_is_in_use() {
        true => S_OK,
        false => S_FALSE,
    }
}

/// Write everything that makes Windows Search aware of this handler.
#[unsafe(no_mangle)]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    let Some(path) = own_path() else {
        return E_FAIL;
    };

    for entry in entries(&path) {
        if !write_entry(&entry) {
            return E_FAIL;
        }
    }

    S_OK
}

/// Take all of it out again.
///
/// Every key is attempted even after one fails, because stopping half way
/// leaves a registration that is neither on nor off. The first failure is what
/// is reported.
#[unsafe(no_mangle)]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    let mut outcome = S_OK;

    for key in keys_to_remove() {
        let name = HSTRING::from(key);
        let removed = unsafe { RegDeleteTreeW(HKEY_LOCAL_MACHINE, &name) };
        // A key that was never there is not a failure. Taking out a handler
        // that was only partly registered has to be able to finish.
        if !is_success(removed) && removed != windows::Win32::Foundation::ERROR_FILE_NOT_FOUND {
            outcome = E_FAIL;
        }
    }

    let (key, value) = value_to_remove();
    if !remove_value(key, value) {
        outcome = E_FAIL;
    }

    outcome
}

/// Put one value into the registry, making the key if it is not there.
fn write_entry(entry: &Entry) -> bool {
    let Hive::LocalMachine = entry.hive;
    let key_name = HSTRING::from(entry.key.as_str());
    let mut key = HKEY::default();

    let opened = unsafe {
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            &key_name,
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
    };
    if !is_success(opened) {
        return false;
    }

    let value_name = entry.name.as_ref().map(|name| HSTRING::from(name.as_str()));
    let value: Vec<u16> = entry
        .value
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let bytes = unsafe {
        std::slice::from_raw_parts(value.as_ptr().cast::<u8>(), value.len() * size_of::<u16>())
    };

    let written = unsafe {
        RegSetValueExW(
            key,
            match &value_name {
                Some(name) => PCWSTR(name.as_ptr()),
                // No name means the key's own unnamed value, which is what
                // most of a COM registration is written into.
                None => PCWSTR::null(),
            },
            None,
            REG_SZ,
            Some(bytes),
        )
    };
    let _ = unsafe { RegCloseKey(key) };

    is_success(written)
}

/// Take one named value out of a key that belongs to somebody else.
fn remove_value(key: &str, value: &str) -> bool {
    let key_name = HSTRING::from(key);
    let mut opened = HKEY::default();

    let result =
        unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, &key_name, None, KEY_WRITE, &mut opened) };
    if !is_success(result) {
        // The key belongs to Windows Search. If it is not there, neither is
        // our value, and there is nothing to undo.
        return true;
    }

    let value_name = HSTRING::from(value);
    let removed = unsafe { RegDeleteValueW(opened, &value_name) };
    let _ = unsafe { RegCloseKey(opened) };

    is_success(removed) || removed == windows::Win32::Foundation::ERROR_FILE_NOT_FOUND
}

/// Where this DLL is on disk.
///
/// Asked of Windows using an address inside this file rather than worked out
/// from anything the caller supplied, so the path written into the registry is
/// always the file that is actually running. A path built any other way is how
/// a handler ends up registered against a copy that has been moved or deleted.
fn own_path() -> Option<String> {
    let mut module = HMODULE::default();
    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(own_path as *const () as *const u16),
            &mut module,
        )
        .ok()?;
    }

    // Room for the long form as well as the old limit, because Program Files
    // under a redirected drive can exceed the old one.
    let mut buffer = [0u16; MAX_PATH as usize * 4];
    let used = unsafe { GetModuleFileNameW(Some(module), &mut buffer) } as usize;
    if used == 0 || used >= buffer.len() {
        return None;
    }

    String::from_utf16(&buffer[..used]).ok()
}
