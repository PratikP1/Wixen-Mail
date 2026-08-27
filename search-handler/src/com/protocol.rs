//! The object that turns a URL into something the indexer can ask about.

use super::accessor::UrlAccessor;
use super::{Alive, LONGEST_URL, read_wide};
use crate::url::ItemUrl;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::System::Search::{
    AUTHENTICATION_INFO, INCREMENTAL_ACCESS_INFO, IProtocolHandlerSite, ISearchProtocol,
    ISearchProtocol_Impl, ITEM_INFO, IUrlAccessor, PROXY_INFO, TIMEOUT_INFO,
};
use windows_core::{PCWSTR, Ref, implement};

/// The protocol handler itself.
///
/// It holds no state. Everything about one item belongs to the accessor made
/// for it, which is what lets the indexer work on several at once without this
/// object needing to know.
#[implement(ISearchProtocol)]
pub struct SearchProtocol {
    _alive: Alive,
}

impl SearchProtocol {
    pub fn new() -> Self {
        Self {
            _alive: Alive::new(),
        }
    }
}

impl Default for SearchProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ISearchProtocol_Impl for SearchProtocol_Impl {
    /// Nothing to set up.
    ///
    /// The timeouts, the site and the proxy settings are all for a handler
    /// that reaches out over a network. This one reads a file on the same
    /// machine, so none of them apply and none is stored.
    fn Init(
        &self,
        _ptimeoutinfo: *const TIMEOUT_INFO,
        _pprotocolhandlersite: Ref<IProtocolHandlerSite>,
        _pproxyinfo: *const PROXY_INFO,
    ) -> windows_core::Result<()> {
        Ok(())
    }

    /// Make something that can answer questions about one URL.
    ///
    /// A URL that is not ours is refused here rather than further in. The
    /// indexer should only ever send our own scheme, and this is the boundary
    /// of a DLL running inside somebody else's process, so it checks anyway.
    fn CreateAccessor(
        &self,
        pcwszurl: &PCWSTR,
        _pauthenticationinfo: *const AUTHENTICATION_INFO,
        _pincrementalaccessinfo: *const INCREMENTAL_ACCESS_INFO,
        _piteminfo: *const ITEM_INFO,
    ) -> windows_core::Result<IUrlAccessor> {
        let text = unsafe { read_wide(*pcwszurl, LONGEST_URL) }
            .ok_or_else(|| windows_core::Error::from_hresult(E_INVALIDARG))?;

        let url =
            ItemUrl::parse(&text).map_err(|_| windows_core::Error::from_hresult(E_INVALIDARG))?;

        Ok(UrlAccessor::new(url).into())
    }

    /// Nothing to close.
    ///
    /// The accessor owns its own database connection and closes it when the
    /// indexer lets go of it. Keeping a list here so it could be closed early
    /// would mean this object holding references the indexer thinks it has
    /// released.
    fn CloseAccessor(&self, _paccessor: Ref<IUrlAccessor>) -> windows_core::Result<()> {
        Ok(())
    }

    /// Nothing to shut down.
    fn ShutDown(&self) -> windows_core::Result<()> {
        Ok(())
    }
}
