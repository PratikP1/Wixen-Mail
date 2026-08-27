//! The object that answers questions about one URL.

use super::filter::MailFilter;
use super::{Alive, open_store_for};
use crate::chunks::{self, Child, Chunk};
use crate::record::Message;
use crate::registration::FILTER_CLSID_VALUE;
use crate::store::{Store, StoreError};
use crate::url::{ItemUrl, Place};
use std::sync::OnceLock;
use windows::Win32::Foundation::{E_NOTIMPL, FILETIME, S_FALSE};
use windows::Win32::Storage::IndexServer::IFilter;
use windows::Win32::System::Com::IStream;
use windows::Win32::System::Com::StructuredStorage::{PROPSPEC, PROPVARIANT};
use windows::Win32::System::Search::{IUrlAccessor, IUrlAccessor_Impl};
use windows_core::{GUID, PWSTR, implement};

/// What this handler calls the shape of its items.
///
/// The indexer uses a document format to find a filter for an item when it has
/// not been given one directly. This handler always gives one, through
/// [`IUrlAccessor::BindToFilter`], so this string is only ever a label.
const DOCUMENT_FORMAT: &str = "WixenMail.Message";

/// The host every one of our URLs names.
const HOST: &str = "localhost";

/// The answer to a question this handler has no answer for.
///
/// Returned rather than a made up value in several places below. Each one is
/// commented with what it would take to answer properly, because an
/// unimplemented method that nobody wrote down is indistinguishable from one
/// that was forgotten.
fn no_answer() -> windows_core::Error {
    windows_core::Error::from_hresult(E_NOTIMPL)
}

/// A successful no.
///
/// Windows says "not a directory" with `S_FALSE`, which is a success code.
/// Rust's `Result` has no room for a success that is also a negative, so it
/// travels as an error carrying a success code, which is what the conversion
/// back to an `HRESULT` puts on the wire.
fn successful_no() -> windows_core::Error {
    windows_core::Error::from_hresult(S_FALSE)
}

/// One URL, and everything the indexer may ask about it.
#[implement(IUrlAccessor)]
pub struct UrlAccessor {
    url: ItemUrl,
    /// Opened once when the accessor is made. A failure is kept rather than
    /// retried, because every method would otherwise try again and a missing
    /// database would be opened once per question instead of once per item.
    store: Result<Store, StoreError>,
    /// The message this URL names, read at most once however many questions
    /// are asked about it.
    message: OnceLock<Option<Message>>,
    _alive: Alive,
}

impl UrlAccessor {
    pub fn new(url: ItemUrl) -> Self {
        let store = open_store_for(url.user.as_deref());
        Self {
            url,
            store,
            message: OnceLock::new(),
            _alive: Alive::new(),
        }
    }

    /// The message this URL names, if it names one and it is still there.
    fn message(&self) -> Option<&Message> {
        self.message
            .get_or_init(|| match (&self.store, &self.url.place) {
                (
                    Ok(store),
                    Place::Message {
                        account,
                        folder,
                        uid,
                    },
                ) => store.message(account, folder, *uid).ok().flatten(),
                _ => None,
            })
            .as_ref()
    }

    /// Everything the filter for this URL will hand over.
    ///
    /// A container enumerates what is inside it, every time and whether or not
    /// anything changed. Microsoft's page is explicit that this is how the
    /// indexer notices deletions: an item that stops being enumerated is an
    /// item it removes.
    fn chunks(&self) -> Option<Vec<Chunk>> {
        let store = self.store.as_ref().ok()?;

        match &self.url.place {
            Place::Message { .. } => self.message().map(chunks::for_message),
            Place::Root => {
                let children = store
                    .accounts()
                    .ok()?
                    .into_iter()
                    .map(|account| self.child_at(Place::Account { account }, None))
                    .collect::<Vec<_>>();
                Some(chunks::for_children(&children))
            }
            Place::Account { account } => {
                let children = store
                    .folders(account)
                    .ok()?
                    .into_iter()
                    .map(|folder| {
                        self.child_at(
                            Place::Folder {
                                account: account.clone(),
                                folder,
                            },
                            None,
                        )
                    })
                    .collect::<Vec<_>>();
                Some(chunks::for_children(&children))
            }
            Place::Folder { account, folder } => {
                let children = store
                    .message_stubs(account, folder)
                    .ok()?
                    .into_iter()
                    .map(|stub| {
                        self.child_at(
                            Place::Message {
                                account: account.clone(),
                                folder: folder.clone(),
                                uid: stub.uid,
                            },
                            stub.modified,
                        )
                    })
                    .collect::<Vec<_>>();
                Some(chunks::for_children(&children))
            }
        }
    }

    /// A child URL below this one, keeping whichever user this URL named.
    fn child_at(&self, place: Place, modified: Option<i64>) -> Child {
        Child {
            url: ItemUrl {
                user: self.url.user.clone(),
                place,
            }
            .to_string(),
            modified,
        }
    }

    /// What this item is called, for whatever Windows shows it as.
    fn name(&self) -> String {
        match &self.url.place {
            Place::Root => "Wixen Mail".to_string(),
            Place::Account { account } => account.clone(),
            Place::Folder { folder, .. } => folder.clone(),
            Place::Message { uid, .. } => match self.message() {
                Some(message) => message.display_name().to_string(),
                None => uid.to_string(),
            },
        }
    }
}

/// Copy text into a buffer Windows supplied, with a terminator.
///
/// The length written back never counts the terminator. A buffer too small is
/// a failure rather than a truncation, because a name cut in half is a name
/// that matches nothing and says nothing about why.
fn write_into(text: &str, buffer: PWSTR, room: u32, written: *mut u32) -> windows_core::Result<()> {
    if buffer.is_null() {
        return Err(windows_core::Error::from_hresult(
            windows::Win32::Foundation::E_POINTER,
        ));
    }

    let units: Vec<u16> = text.encode_utf16().collect();
    if units.len() + 1 > room as usize {
        return Err(windows_core::Error::from_hresult(
            windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER.to_hresult(),
        ));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(units.as_ptr(), buffer.0, units.len());
        *buffer.0.add(units.len()) = 0;
        if !written.is_null() {
            *written = units.len() as u32;
        }
    }
    Ok(())
}

impl IUrlAccessor_Impl for UrlAccessor_Impl {
    /// Not implemented.
    ///
    /// It exists so the indexer can pass a store extra parameters, such as a
    /// mail profile name. There is nothing this store needs beyond the URL.
    fn AddRequestParameter(
        &self,
        _pspec: *const PROPSPEC,
        _pvar: *const PROPVARIANT,
    ) -> windows_core::Result<()> {
        Err(no_answer())
    }

    fn GetDocFormat(
        &self,
        wszdocformat: PWSTR,
        dwsize: u32,
        pdwlength: *mut u32,
    ) -> windows_core::Result<()> {
        write_into(DOCUMENT_FORMAT, wszdocformat, dwsize, pdwlength)
    }

    /// Which class can read one of these items.
    ///
    /// The filter in this same DLL. Answering here as well as through
    /// `BindToFilter` covers both of the ways the indexer may go looking for
    /// one.
    fn GetCLSID(&self) -> windows_core::Result<GUID> {
        Ok(GUID::from_u128(FILTER_CLSID_VALUE))
    }

    fn GetHost(
        &self,
        wszhost: PWSTR,
        dwsize: u32,
        pdwlength: *mut u32,
    ) -> windows_core::Result<()> {
        write_into(HOST, wszhost, dwsize, pdwlength)
    }

    /// Whether this URL holds other things.
    ///
    /// Success means yes and `S_FALSE` means no, which is why the negative
    /// travels as an error carrying a success code.
    fn IsDirectory(&self) -> windows_core::Result<()> {
        match self.url.place {
            Place::Message { .. } => Err(successful_no()),
            _ => Ok(()),
        }
    }

    /// How much text this item has.
    ///
    /// Zero for a container, which holds no text of its own.
    fn GetSize(&self) -> windows_core::Result<u64> {
        Ok(self
            .message()
            .map(|message| message.searchable_text().len() as u64)
            .unwrap_or(0))
    }

    /// When this item last changed.
    ///
    /// A message never changes after it arrives, so the moment it was sent is
    /// the answer. A container has none: Microsoft's page says the indexer
    /// ignores the time on a directory and re-enumerates it regardless, which
    /// is also how it notices that something has been deleted.
    fn GetLastModified(&self) -> windows_core::Result<FILETIME> {
        let ticks = self
            .message()
            .and_then(|message| message.sent)
            .and_then(crate::record::windows_ticks)
            .unwrap_or(0);

        Ok(FILETIME {
            dwLowDateTime: (ticks & 0xFFFF_FFFF) as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        })
    }

    fn GetFileName(
        &self,
        wszfilename: PWSTR,
        dwsize: u32,
        pdwlength: *mut u32,
    ) -> windows_core::Result<()> {
        write_into(&self.name(), wszfilename, dwsize, pdwlength)
    }

    /// Not implemented, and this is worth knowing about.
    ///
    /// A store can describe who may see an item, and the indexer uses that to
    /// keep one person's results out of another's. This store has nothing to
    /// describe it with: the cache records no permissions. The protection that
    /// remains is the user identifier in the URL, which is what tells the
    /// indexer whose data an item is. Answering here would mean building a
    /// Windows security descriptor for the profile the mail belongs to, which
    /// is real work and needs a real indexer run to check.
    fn GetSecurityDescriptor(
        &self,
        _psd: *mut u8,
        _dwsize: u32,
        _pdwlength: *mut u32,
    ) -> windows_core::Result<()> {
        Err(no_answer())
    }

    /// Not implemented. Nothing in this store moves to another address.
    fn GetRedirectedURL(
        &self,
        _wszredirectedurl: PWSTR,
        _dwsize: u32,
        _pdwlength: *mut u32,
    ) -> windows_core::Result<()> {
        Err(no_answer())
    }

    /// Not implemented, for the same reason as the security descriptor above.
    fn GetSecurityProvider(&self) -> windows_core::Result<GUID> {
        Err(no_answer())
    }

    /// Not implemented, on purpose.
    ///
    /// This is the other way an item can be handed over: as a stream of bytes
    /// for a filter to parse. Nothing here is a stream of bytes, it is a row
    /// in a table, so the filter is handed the item directly instead through
    /// `BindToFilter`.
    fn BindToStream(&self) -> windows_core::Result<IStream> {
        Err(no_answer())
    }

    /// Hand back a filter already holding everything this item has to say.
    fn BindToFilter(&self) -> windows_core::Result<IFilter> {
        let chunks = self
            .chunks()
            .ok_or_else(|| windows_core::Error::from_hresult(E_NOTIMPL))?;

        Ok(MailFilter::new(chunks).into())
    }
}
