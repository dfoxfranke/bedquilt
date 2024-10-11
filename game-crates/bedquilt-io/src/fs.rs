pub use wasm2glulx_ffi::glk::{FileMode, SeekMode, FileUsage as GlkFileUsage};
use core::ffi::CStr;
use crate::sys::io::*;
use crate::error::*;

pub trait ReadFile {
    fn read(&self, buf: &mut [u8]) -> u32;
}

pub trait WriteFile {
    fn write(&self, buf: &[u8]);
}

pub trait SeekFile {
    fn seek(&self, pos: i32, mode: SeekMode);
    fn pos(&self) -> u32;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileRef(pub(crate) FileRefImpl);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RoFile(pub(crate) RoFileImpl);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RwFile(pub(crate) RwFileImpl);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FileUsage {
    SavedGame,
    Transcript,
    InputRecord,
    Data,
}

impl RoFile {
    pub fn open(fref: &FileRef) -> Result<Self> {
        Ok(RoFile(RoFileImpl::open(&fref.0)?))
    }

    pub fn open_resource(num: u32) -> Result<Self> {
        Ok(RoFile(RoFileImpl::open_resource(num)?))
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
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
    }
}

impl FileRef {
    pub fn create_temp(usage: FileUsage) -> Result<Self> {
        Ok(Self(FileRefImpl::create_temp(usage)?))
    }

    pub fn create_by_prompt(usage: FileUsage, mode: FileMode) -> Result<Self> {
        Ok(Self(FileRefImpl::create_by_prompt(usage, mode)?))
    }

    pub fn create_by_name(usage: FileUsage, name: &CStr) -> Result<Self> {
        Ok(Self(FileRefImpl::create_by_name(usage, name)?))
    }

    pub fn delete(&self) {
        self.0.delete();
    }

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
