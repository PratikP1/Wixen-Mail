//! The object that hands the indexer one item's text and properties.
//!
//! The lint below asks that a public function dereferencing a raw pointer be
//! marked unsafe. These are not functions this crate declared: they are the
//! methods of a COM interface, and their signatures come from the Windows
//! bindings, raw pointers and all. There is no way to mark them unsafe and
//! still satisfy the trait, so the allow says what would otherwise be a
//! silence. Every one of those pointers is checked for null before it is used.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use super::{Alive, values};
use crate::chunks::{Chunk, Taken, Walk};
use crate::registration::FILTER_CLSID_VALUE;
use std::sync::Mutex;
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, E_NOTIMPL, E_OUTOFMEMORY, S_FALSE, S_OK};
use windows::Win32::Storage::IndexServer::{
    CHUNK_EOS, CHUNK_TEXT, CHUNK_VALUE, FILTER_E_END_OF_CHUNKS, FILTER_E_NO_MORE_TEXT,
    FILTER_E_NO_MORE_VALUES, FILTER_E_NO_TEXT, FILTER_E_NO_VALUES, FILTERREGION, FULLPROPSPEC,
    IFilter, IFilter_Impl, STAT_CHUNK,
};
use windows::Win32::System::Com::StructuredStorage::{PROPSPEC, PROPSPEC_0, PROPVARIANT};
use windows::Win32::System::Com::{IPersist_Impl, IPersistStream, IPersistStream_Impl, IStream};
use windows::core::{BOOL, GUID, HRESULT, PWSTR, Ref, implement};

/// A property named by its number rather than by a string.
///
/// Every property this handler uses has a number, so the other kind never
/// comes up and this is the only value ever written.
const BY_NUMBER: windows::Win32::System::Com::StructuredStorage::PROPSPEC_KIND =
    windows::Win32::System::Com::StructuredStorage::PRSPEC_PROPID;

/// The filter Windows Search reads one item through.
///
/// It is handed everything it will ever say when it is made, by
/// [`super::accessor`], so it never touches the database itself. That keeps
/// the database open for the shortest time and keeps this object, which the
/// indexer may hold for a while, from holding a connection with it.
#[implement(IFilter, IPersistStream)]
pub struct MailFilter {
    /// What this item has to say, kept so [`IFilter::Init`] can start again.
    chunks: Vec<Chunk>,
    /// How far through it the indexer has got.
    position: Mutex<Walk>,
    _alive: Alive,
}

impl MailFilter {
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Self {
            position: Mutex::new(Walk::new(chunks.clone())),
            chunks,
            _alive: Alive::new(),
        }
    }

    /// Do one piece of work with the position held.
    ///
    /// A poisoned lock means a thread panicked while holding it. That should
    /// be impossible here, and if it ever happens the honest answer is a
    /// failure rather than carrying on with a position nobody can trust.
    fn with_position<T>(&self, work: impl FnOnce(&mut Walk) -> T, on_failure: T) -> T {
        match self.position.lock() {
            Ok(mut position) => work(&mut position),
            Err(_) => on_failure,
        }
    }
}

impl IFilter_Impl for MailFilter_Impl {
    /// Start, or start again.
    ///
    /// The flags asking for particular attributes are not honoured. This
    /// filter has a short fixed list of properties and hands over all of them;
    /// filtering that list to the indexer's request would save nothing worth
    /// the code. `pflags` is set to zero, which says there are no separate
    /// document properties to fetch.
    fn Init(
        &self,
        _grfflags: u32,
        _cattributes: u32,
        _aattributes: *const FULLPROPSPEC,
        pflags: *mut u32,
    ) -> i32 {
        if !pflags.is_null() {
            unsafe { *pflags = 0 };
        }

        let restarted = Walk::new(self.chunks.clone());
        match self.position.lock() {
            Ok(mut position) => {
                *position = restarted;
                S_OK.0
            }
            Err(_) => E_FAIL.0,
        }
    }

    fn GetChunk(&self, pstat: *mut STAT_CHUNK) -> i32 {
        if pstat.is_null() {
            return E_INVALIDARG.0;
        }

        self.with_position(
            |position| match position.advance() {
                None => FILTER_E_END_OF_CHUNKS.0,
                Some(marker) => {
                    let described = STAT_CHUNK {
                        idChunk: marker.id,
                        // Every chunk here is a complete thought: a whole
                        // property or the whole of an item's text. Saying so
                        // stops the indexer running the last word of one chunk
                        // into the first word of the next.
                        breakType: CHUNK_EOS,
                        flags: match marker.is_text {
                            true => CHUNK_TEXT,
                            false => CHUNK_VALUE,
                        },
                        // Neutral. The cache does not record what language a
                        // message is in, and claiming one would make the
                        // indexer break words by the wrong rules. Zero leaves
                        // that to Windows.
                        locale: 0,
                        attribute: FULLPROPSPEC {
                            guidPropSet: marker.attribute.fmtid,
                            psProperty: PROPSPEC {
                                ulKind: BY_NUMBER,
                                Anonymous: PROPSPEC_0 {
                                    propid: marker.attribute.pid,
                                },
                            },
                        },
                        idChunkSource: marker.id,
                        cwcStartSource: 0,
                        cwcLenSource: 0,
                    };
                    unsafe { pstat.write(described) };
                    S_OK.0
                }
            },
            E_FAIL.0,
        )
    }

    fn GetText(&self, pcwcbuffer: *mut u32, awcbuffer: PWSTR) -> i32 {
        if pcwcbuffer.is_null() || awcbuffer.is_null() {
            return E_INVALIDARG.0;
        }

        let room = unsafe { *pcwcbuffer } as usize;
        self.with_position(
            |position| match position.take_text(room) {
                Taken::WrongKind => FILTER_E_NO_TEXT.0,
                Taken::AlreadyGiven => FILTER_E_NO_MORE_TEXT.0,
                Taken::Some(units) => {
                    unsafe {
                        std::ptr::copy_nonoverlapping(units.as_ptr(), awcbuffer.0, units.len());
                        // A terminator when there is room for one. The count
                        // returned never includes it, so a caller that ignores
                        // terminators is unaffected and one that expects them
                        // does not read past the end.
                        if units.len() < room {
                            *awcbuffer.0.add(units.len()) = 0;
                        }
                        *pcwcbuffer = units.len() as u32;
                    }
                    S_OK.0
                }
            },
            E_FAIL.0,
        )
    }

    fn GetValue(&self, pppropvalue: *mut *mut PROPVARIANT) -> i32 {
        if pppropvalue.is_null() {
            return E_INVALIDARG.0;
        }
        unsafe { *pppropvalue = std::ptr::null_mut() };

        self.with_position(
            |position| match position.take_value() {
                Taken::WrongKind => FILTER_E_NO_VALUES.0,
                Taken::AlreadyGiven => FILTER_E_NO_MORE_VALUES.0,
                Taken::Some(value) => match unsafe { values::allocate(&value) } {
                    Some(built) => {
                        unsafe { *pppropvalue = built };
                        S_OK.0
                    }
                    None => E_OUTOFMEMORY.0,
                },
            },
            E_FAIL.0,
        )
    }

    /// Not implemented, and Microsoft documents it as reserved.
    ///
    /// It exists so a caller can ask which part of an original document a
    /// chunk came from. There is no original document here: the text is
    /// assembled from database columns, so there is no region to point at.
    fn BindRegion(
        &self,
        _origpos: &FILTERREGION,
        _riid: *const GUID,
        _ppunk: *mut *mut core::ffi::c_void,
    ) -> i32 {
        E_NOTIMPL.0
    }
}

impl IPersist_Impl for MailFilter_Impl {
    fn GetClassID(&self) -> windows::core::Result<GUID> {
        Ok(GUID::from_u128(FILTER_CLSID_VALUE))
    }
}

impl IPersistStream_Impl for MailFilter_Impl {
    /// Never. This object only reads.
    fn IsDirty(&self) -> HRESULT {
        S_FALSE
    }

    /// Deliberately not implemented, and this is a real limit worth knowing.
    ///
    /// A filter can be reached two ways. The way this handler uses is
    /// `IUrlAccessor::BindToFilter`, where the accessor makes the filter and
    /// hands it everything it needs. The other way is for the indexer to make
    /// one from its class identifier and pour a stream into it through here,
    /// which is how filters for file types work. This filter has no way to
    /// rebuild an item from a stream of bytes, because what it hands over is
    /// several database columns and not the contents of a file.
    ///
    /// So this returns a failure rather than succeeding and producing nothing.
    /// If a real indexer run turns out to take this path, that will show up as
    /// a clear error rather than as a mailbox that indexes to silence, and the
    /// answer would be to give the accessor's `BindToStream` something to hand
    /// over and teach this to read it back.
    fn Load(&self, _pstm: Ref<IStream>) -> windows::core::Result<()> {
        Err(windows::core::Error::from_hresult(E_NOTIMPL))
    }

    /// Never. Nothing in this DLL writes.
    fn Save(&self, _pstm: Ref<IStream>, _fcleardirty: BOOL) -> windows::core::Result<()> {
        Err(windows::core::Error::from_hresult(E_NOTIMPL))
    }

    fn GetSizeMax(&self) -> windows::core::Result<u64> {
        Err(windows::core::Error::from_hresult(E_NOTIMPL))
    }
}
