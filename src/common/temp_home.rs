//! A value and the temporary folder it lives in, removed together.
//!
//! Tests here open a `MessageCache`, and a `MessageCache` holds an open SQLite
//! connection to a file inside a folder. Windows will not unlink a file that is
//! open, so the folder can only be removed after the cache is dropped. Get that
//! order wrong and `TempDir`'s own cleanup fails, and it fails silently:
//! `TempDir::drop` throws the error away. A tree that leaks and a tree that
//! does not look identical from the test output.
//!
//! That is what happened. Every run of the suite left 404 folders in the
//! system temporary directory, the commit hook runs the suite on every commit,
//! and `scripts/mutants.sh` runs it once per mutant for two days. They were
//! counted in the millions before anybody looked.
//!
//! The order lives here, once, in the order the fields are declared, rather
//! than at every place that unpacks a pair. A struct drops its fields in
//! declaration order, so `thing` goes first and `folder` follows it.

use std::ops::{Deref, DerefMut};
use std::path::Path;
use tempfile::TempDir;

/// Something built inside a temporary folder that outlives neither.
///
/// Use it through `Deref`, so a `TempHome<MessageCache>` is called exactly the
/// way a `MessageCache` is. Hold it in one binding and let it fall out of scope
/// at the end of the test.
pub struct TempHome<T> {
    /// Declared first so it is dropped first. Anything holding a file open
    /// inside `folder` lives here, and has to let go before the folder can be
    /// removed. Swapping these two lines is the whole bug, and
    /// `test_the_test_cache_takes_its_folder_with_it` is what catches it.
    thing: T,
    /// Declared last so it is dropped last, once nothing is reading from it.
    folder: TempDir,
}

impl<T> TempHome<T> {
    /// Builds a value inside a fresh temporary folder.
    ///
    /// The folder gets a random name, so two tests running side by side never
    /// share one. That matters beyond tidiness: tests sharing a database made
    /// an earlier mutation run report every mutant as caught.
    pub fn new(build: impl FnOnce(&Path) -> T) -> Self {
        Self::holding(tempfile::tempdir().expect("a temporary folder"), build)
    }

    /// The same, with a readable prefix on the folder name.
    ///
    /// For helpers that already take a label to say which test a folder belongs
    /// to. The random part is still there, so the name is a debugging aid
    /// rather than the thing keeping two tests apart.
    pub fn named(prefix: &str, build: impl FnOnce(&Path) -> T) -> Self {
        let folder = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .expect("a temporary folder");
        Self::holding(folder, build)
    }

    fn holding(folder: TempDir, build: impl FnOnce(&Path) -> T) -> Self {
        let thing = build(folder.path());
        Self { thing, folder }
    }

    /// Where the folder is, for a test that needs to reopen it or look inside.
    pub fn path(&self) -> &Path {
        self.folder.path()
    }
}

impl<T> Deref for TempHome<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.thing
    }
}

impl<T> DerefMut for TempHome<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.thing
    }
}
