//! Making one of the two classes this DLL offers.
//!
//! Same reason for the allow as in [`super::filter`]: these are COM interface
//! methods whose signatures come from the Windows bindings and cannot be
//! marked unsafe without failing to implement the trait. Both pointers are
//! checked for null first.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use super::filter::MailFilter;
use super::protocol::SearchProtocol;
use super::{Alive, hold_server};
use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_POINTER};
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};
use windows_core::{BOOL, GUID, IUnknown, Interface, Ref, implement};

/// Which of this DLL's classes a factory makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Teaches the indexer the URL scheme and finds items.
    Protocol,
    /// Reads one item's text and properties.
    Filter,
}

/// The object COM asks for one of ours.
#[implement(IClassFactory)]
pub struct ClassFactory {
    class: Class,
    /// A factory counts too. COM hands these out and holds on to them, and a
    /// server that answered "nothing is in use" while one was still alive
    /// would be unloaded with a live pointer to it left in the indexer.
    _alive: Alive,
}

impl ClassFactory {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            _alive: Alive::new(),
        }
    }
}

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()> {
        if ppvobject.is_null() || riid.is_null() {
            return Err(windows_core::Error::from_hresult(E_POINTER));
        }
        unsafe { *ppvobject = std::ptr::null_mut() };

        // Aggregation lets one object be built into another and share its
        // identity. Neither of these classes is written for it, and saying so
        // is what the rules require: pretending otherwise gives the caller an
        // object whose reference counting is wrong.
        if punkouter.is_some() {
            return Err(windows_core::Error::from_hresult(CLASS_E_NOAGGREGATION));
        }

        let made: IUnknown = match self.class {
            Class::Protocol => SearchProtocol::new().into(),
            // A filter made this way has nothing to read. The working path is
            // the accessor building one with an item already in it; this exists
            // so the registered class can be created at all, and it honestly
            // reports having no chunks rather than inventing any.
            Class::Filter => MailFilter::new(Vec::new()).into(),
        };

        unsafe { made.query(riid, ppvobject).ok() }
    }

    /// Keep this DLL loaded, or stop keeping it loaded.
    fn LockServer(&self, flock: BOOL) -> windows_core::Result<()> {
        hold_server(flock.as_bool());
        Ok(())
    }
}
