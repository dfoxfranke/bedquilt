//! Filesystem and resource access.
//! 
//! This module handles reading and writing to files on disk, as well reading
//! from Blorb data resources. To open a file, first obtain a [`FileRef`] using
//! one of its three constructors, and then pass that to a constructor of
//! [`RoFile`] or [`RwFile`]. To open a data resource, use
//! [`RoFile::open_resource`].
//! 
//! Files are always opened in Glk's "binary" mode, meaning that it stays out of
//! the way with respect to character encoding and line endings. Reading and
//! writing UTF-8 always works regardless of whether the Glk implementation
//! supports Unicode. `RwFile` implements [`core::fmt::Write`] in addition to
//! the three traits defined in this module.
//! 
//! Files are automatically closed when dropped.

use crate::{
    error::Result,
    sys::glk::{FileRefImpl, RoFileImpl, RwFileImpl},
};

use core::ffi::CStr;
use wasm2glulx_ffi::glk::FileUsage as GlkFileUsage;


/// Read/write/append bits.
pub use wasm2glulx_ffi::glk::FileMode;

/// Seek relative to the beginning, end, or current position in a file.
pub use wasm2glulx_ffi::glk::SeekMode;

/// Read bytes from a file or resource.
pub trait ReadFile {
    /// Read bytes into `buf`, returning the number of bytes read.
    fn read(&self, buf: &mut [u8]) -> u32;
}

/// Write bytes to a file.
pub trait WriteFile {
    /// Write out the contents of `buf`.
    fn write(&self, buf: &[u8]);
}

/// Seek around in a file or resource.
pub trait SeekFile {
    /// Seeks to the position given by `pos`, relative to the position
    /// determined by `mode`.
    fn seek(&self, pos: i32, mode: SeekMode);
    /// Returns the current position, given in bytes from the beginning of the
    /// file.
    fn pos(&self) -> u32;
}

/// A reference to a file on disk.
/// 
/// A `FileRef` just holds a path; the referenced file may or may not exist, and
/// is not open until you obtain a [`RoFile`] or [`RwFile`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileRef(pub(crate) FileRefImpl);

/// A file or resource opened for read-only access.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RoFile(pub(crate) RoFileImpl);

/// A file opened for read/write access.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RwFile(pub(crate) RwFileImpl);

/// The purpose for which a file will be used.
/// 
/// This is passed as an argument to [`FileRef`]'s constructors. What is done
/// with it is dependent on the Glk implementation. It may determine what
/// extension gets added to the filename, and it may affect what prompt is
/// presented to the user when calling [`FileRef::create_by_prompt`].
/// 
/// Unlike in Glk, this enum does not specify text mode versus binary mode.
/// [`RoFile`]s and [`RwFile`]s are always opened in binary mode. Transcript
/// files created by
/// [`TextBufferWindow::transcript`](crate::win::TextBufferWindow::transcript)
/// are always opened in text mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FileUsage {
    /// Saved game file.
    SavedGame,
    /// Transcript file.
    Transcript,
    /// Files which stores a record of user input.
    InputRecord,
    /// Generic data file.
    Data,
}

impl RoFile {
    /// Opens a file for read-only access.
    pub fn open(fref: &FileRef) -> Result<Self> {
        Ok(RoFile(RoFileImpl::open(&fref.0)?))
    }

    /// Opens a data resource.
    pub fn open_resource(resource_index: u32) -> Result<Self> {
        Ok(RoFile(RoFileImpl::open_resource(resource_index)?))
    }
}

impl ReadFile for RoFile {
    fn read(&self, buf: &mut [u8]) -> u32 {
        self.0.read(buf)
    }
}

impl SeekFile for RoFile {
    fn pos(&self) -> u32 {
        self.0.pos()
    }

    fn seek(&self, pos: i32, mode: SeekMode) {
        self.0.seek(pos, mode)
    }
}

impl RwFile {
    /// Opens a file for read/write access.
    pub fn open(fref: &FileRef) -> Result<Self> {
        Ok(RwFile(RwFileImpl::open(&fref.0)?))
    }
}

impl ReadFile for RwFile {
    fn read(&self, buf: &mut [u8]) -> u32 {
        self.0.read(buf)
    }
}

impl SeekFile for RwFile {
    fn pos(&self) -> u32 {
        self.0.pos()
    }

    fn seek(&self, pos: i32, mode: SeekMode) {
        self.0.seek(pos, mode)
    }
}

impl WriteFile for RwFile {
    fn write(&self, buf: &[u8]) {
        self.0.write(buf);
    }
}

impl core::fmt::Write for RwFile {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(s)
    }
}

impl core::fmt::Write for &RwFile {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        (&self.0).write_str(s)
    }
}

impl FileRef {
    /// Creates a reference to a temporary file.
    pub fn create_temp(usage: FileUsage) -> Result<Self> {
        Ok(Self(FileRefImpl::create_temp(usage)?))
    }

    /// Prompts the user for a file name and returns a reference to it.
    pub fn create_by_prompt(usage: FileUsage, mode: FileMode) -> Result<Self> {
        Ok(Self(FileRefImpl::create_by_prompt(usage, mode)?))
    }

    /// Creates a reference to a file with the given name.
    pub fn create_by_name(usage: FileUsage, name: &CStr) -> Result<Self> {
        Ok(Self(FileRefImpl::create_by_name(usage, name)?))
    }

    /// Deletes the file.
    pub fn delete(&self) {
        self.0.delete();
    }

    /// Returns true if the file exists.
    pub fn exists(&self) -> bool {
        self.0.exists()
    }
}

impl FileUsage {
    pub(crate) fn to_glk(self, text: bool) -> GlkFileUsage {
        match self {
            FileUsage::SavedGame => GlkFileUsage::SAVED_GAME,
            FileUsage::Transcript => GlkFileUsage::TRANSCRIPT,
            FileUsage::InputRecord => GlkFileUsage::INPUT_RECORD,
            FileUsage::Data => GlkFileUsage::DATA,
        }
        .union(if text {
            GlkFileUsage::TEXT_MODE
        } else {
            GlkFileUsage::BINARY_MODE
        })
    }
}
