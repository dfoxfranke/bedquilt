use crate::error::*;
use crate::fs::*;
use crate::win::*;
use alloc::string::String;
use core::{ffi::CStr, fmt::Write};
pub use wasm2glulx_ffi::glk::WinId;
use wasm2glulx_ffi::{
    glk::{
        cancel_char_event, cancel_hyperlink_event, cancel_line_event, cancel_mouse_event,
        fileref_create_by_name, fileref_create_by_prompt, fileref_create_from_fileref,
        fileref_create_temp, fileref_delete_file, fileref_destroy, fileref_does_file_exist,
        fileref_get_rock, gestalt, get_buffer_stream, put_buffer_stream, put_char_stream,
        put_char_stream_uni, request_char_event, request_char_event_uni, request_hyperlink_event,
        request_line_event, request_line_event_uni, request_mouse_event, select,
        set_hyperlink_stream, stream_close, stream_get_position, stream_open_file,
        stream_open_file_uni, stream_open_resource, stream_set_position, style_distinguish,
        style_measure, stylehint_clear, stylehint_set, window_close, window_get_arrangement,
        window_get_echo_stream, window_get_parent, window_get_stream, window_get_type, window_open,
        window_set_arrangement, window_set_echo_stream, EvType, Event as GlkEvent,
        FileUsage as GlkFileUsage, FrefId, Gestalt, Keycode, StrId, StyleHint as GlkStyleHint,
        StyleHintJustification, WinMethod, WinType,
    },
    glulx::{glkarea_get_byte, glkarea_get_word, glkarea_put_byte, glkarea_put_word, glkarea_size},
};

use std::char;

static mut GLKAREA_IN_USE: bool = false;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BlankWindowImpl(WinId);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextBufferWindowImpl(WinId);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextGridWindowImpl(WinId);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GraphicsWindowImpl(WinId);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileRefImpl(FrefId);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RoFileImpl(StrId);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RwFileImpl(StrId);

impl BlankWindowImpl {
    #[allow(dead_code)]
    pub fn id(&self) -> WinId {
        self.0
    }
}

impl Window for BlankWindowImpl {
    fn create_as_root() -> Result<Self> {
        unsafe {
            let win = window_open(WinId::null(), WinMethod::empty(), 0, WinType::Blank, 0);
            if win.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(win))
            }
        }
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(BlankWindow(BlankWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Blank,
            nth_parent,
        )?)))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextBufferWindow(TextBufferWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextBuffer,
            nth_parent,
        )?)))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextGridWindow(TextGridWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextGrid,
            nth_parent,
        )?)))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(GraphicsWindow(GraphicsWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Graphics,
            nth_parent,
        )?)))
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        do_resize(self.0, percent, false, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        do_measure(self.0, style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        do_distinguish(self.0, style1, style2)
    }
}

impl Drop for BlankWindowImpl {
    fn drop(&mut self) {
        unsafe {
            window_close(self.0, core::ptr::null_mut());
        }
    }
}

impl TextBufferWindowImpl {
    pub fn id(&self) -> WinId {
        self.0
    }

    pub fn resize_chars(&self, chars: u32, nth_parent: u32) {
        do_resize(self.0, chars, true, nth_parent);
    }

    pub fn set_hyperlink(&self, linkval: u32) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::Hyperlinks, 0) != 0
                && gestalt(Gestalt::HyperlinkInput, WinType::TextBuffer.into()) != 0
            {
                set_hyperlink_stream(window_get_stream(self.0), linkval);
                Ok(())
            } else {
                Err(GlkError)
            }
        }
    }

    pub fn transcript(&self, fref: &FileRefImpl, append: bool) -> Result<()> {
        unsafe {
            let textfile = fileref_create_from_fileref(
                GlkFileUsage::TEXT_MODE | GlkFileUsage::TRANSCRIPT,
                fref.0,
                0,
            );
            if textfile.is_null() {
                return Err(GlkError);
            }
            let strid = if gestalt(Gestalt::Unicode, 0) != 0 {
                stream_open_file_uni(
                    textfile,
                    if append {
                        FileMode::WriteAppend
                    } else {
                        FileMode::Write
                    },
                    0,
                )
            } else {
                stream_open_file(
                    textfile,
                    if append {
                        FileMode::WriteAppend
                    } else {
                        FileMode::Write
                    },
                    0,
                )
            };

            fileref_destroy(textfile);

            if strid.is_null() {
                return Err(GlkError);
            }

            let oldstrid = window_get_echo_stream(self.0);
            if !oldstrid.is_null() {
                stream_close(oldstrid, core::ptr::null_mut());
            }
            window_set_echo_stream(self.0, strid);

            Ok(())
        }
    }

    pub fn transcript_off(&self) {
        unsafe {
            let oldstrid = window_get_echo_stream(self.0);
            if !oldstrid.is_null() {
                window_set_echo_stream(self.0, StrId::null());
                stream_close(oldstrid, core::ptr::null_mut());
            }
        }
    }
}

impl Window for TextBufferWindowImpl {
    fn create_as_root() -> Result<Self> {
        unsafe {
            let win = window_open(WinId::null(), WinMethod::empty(), 0, WinType::TextBuffer, 0);
            if win.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(win))
            }
        }
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(BlankWindow(BlankWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Blank,
            nth_parent,
        )?)))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextBufferWindow(TextBufferWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextBuffer,
            nth_parent,
        )?)))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextGridWindow(TextGridWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextGrid,
            nth_parent,
        )?)))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(GraphicsWindow(GraphicsWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Graphics,
            nth_parent,
        )?)))
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        do_resize(self.0, percent, false, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        do_measure(self.0, style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        do_distinguish(self.0, style1, style2)
    }
}

impl Write for TextBufferWindowImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            let stream = window_get_stream(self.0);
            if stream.is_null() {
                return Err(core::fmt::Error);
            }
            if gestalt(Gestalt::Unicode, 0) != 0 {
                for ch in s.chars() {
                    put_char_stream_uni(stream, ch.into());
                }
                Ok(())
            } else {
                for ch in s.chars() {
                    let code: u32 = ch.into();
                    if code < 256 {
                        put_char_stream(stream, code);
                    } else {
                        put_char_stream(stream, '?'.into());
                    }
                }
                Ok(())
            }
        }
    }
}

impl Drop for TextBufferWindowImpl {
    fn drop(&mut self) {
        self.transcript_off();
        unsafe {
            window_close(self.0, core::ptr::null_mut());
        }
    }
}

impl TextGridWindowImpl {
    pub fn id(&self) -> WinId {
        self.0
    }

    pub fn resize_chars(&self, chars: u32, nth_parent: u32) {
        do_resize(self.0, chars, true, nth_parent);
    }
}

impl Window for TextGridWindowImpl {
    fn create_as_root() -> Result<Self> {
        unsafe {
            let win = window_open(WinId::null(), WinMethod::empty(), 0, WinType::TextGrid, 0);
            if win.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(win))
            }
        }
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(BlankWindow(BlankWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Blank,
            nth_parent,
        )?)))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextBufferWindow(TextBufferWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextBuffer,
            nth_parent,
        )?)))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextGridWindow(TextGridWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextGrid,
            nth_parent,
        )?)))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(GraphicsWindow(GraphicsWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Graphics,
            nth_parent,
        )?)))
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        do_resize(self.0, percent, false, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        do_measure(self.0, style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        do_distinguish(self.0, style1, style2)
    }
}

impl Drop for TextGridWindowImpl {
    fn drop(&mut self) {
        unsafe {
            window_close(self.0, core::ptr::null_mut());
        }
    }
}

impl GraphicsWindowImpl {
    #[allow(dead_code)]
    pub fn id(&self) -> WinId {
        self.0
    }

    pub fn resize_pixels(&self, pixels: u32, nth_parent: u32) {
        do_resize(self.0, pixels, true, nth_parent);
    }
}

impl Window for GraphicsWindowImpl {
    fn create_as_root() -> Result<Self> {
        unsafe {
            let win = window_open(WinId::null(), WinMethod::empty(), 0, WinType::Graphics, 0);
            if win.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(win))
            }
        }
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(BlankWindow(BlankWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Blank,
            nth_parent,
        )?)))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextBufferWindow(TextBufferWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextBuffer,
            nth_parent,
        )?)))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(TextGridWindow(TextGridWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextGrid,
            nth_parent,
        )?)))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(GraphicsWindow(GraphicsWindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Graphics,
            nth_parent,
        )?)))
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        do_resize(self.0, percent, false, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        do_measure(self.0, style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        do_distinguish(self.0, style1, style2)
    }
}

impl Drop for GraphicsWindowImpl {
    fn drop(&mut self) {
        unsafe {
            window_close(self.0, core::ptr::null_mut());
        }
    }
}

impl FileRefImpl {
    pub fn create_temp(usage: FileUsage) -> Result<Self> {
        unsafe {
            let glk_usage = usage.to_glk(false);
            let fref = fileref_create_temp(glk_usage, glk_usage.bits());
            if fref.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(fref))
            }
        }
    }

    pub fn create_by_prompt(usage: FileUsage, mode: FileMode) -> Result<Self> {
        unsafe {
            let glk_usage = usage.to_glk(false);
            let fref = fileref_create_by_prompt(glk_usage, mode, glk_usage.bits());
            if fref.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(fref))
            }
        }
    }

    pub fn create_by_name(usage: FileUsage, name: &CStr) -> Result<Self> {
        unsafe {
            let glk_usage = usage.to_glk(false);
            let fref = fileref_create_by_name(glk_usage, name.as_ptr(), glk_usage.bits());
            if fref.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(fref))
            }
        }
    }

    pub fn delete(&self) {
        unsafe {
            fileref_delete_file(self.0);
        }
    }

    pub fn exists(&self) -> bool {
        unsafe { fileref_does_file_exist(self.0) != 0 }
    }
}

impl Clone for FileRefImpl {
    fn clone(&self) -> Self {
        unsafe {
            let usage = fileref_get_rock(self.0);
            let fref =
                fileref_create_from_fileref(GlkFileUsage::from_bits_retain(usage), self.0, usage);
            FileRefImpl(fref)
        }
    }
}

impl Drop for FileRefImpl {
    fn drop(&mut self) {
        unsafe {
            fileref_destroy(self.0);
        }
    }
}

impl RoFileImpl {
    pub fn open(fref: &FileRefImpl) -> Result<Self> {
        unsafe {
            let strid = stream_open_file(fref.0, FileMode::Read, 0);
            if strid.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(strid))
            }
        }
    }

    pub fn open_resource(num: u32) -> Result<Self> {
        unsafe {
            let strid = stream_open_resource(num, 0);
            if strid.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(strid))
            }
        }
    }
}

impl ReadFile for RoFileImpl {
    fn read(&self, buf: &mut [u8]) -> u32 {
        unsafe { get_buffer_stream(self.0, buf.as_mut_ptr().cast(), buf.len() as u32) }
    }
}

impl SeekFile for RoFileImpl {
    fn seek(&self, pos: i32, mode: SeekMode) {
        unsafe { stream_set_position(self.0, pos, mode) }
    }

    fn pos(&self) -> u32 {
        unsafe { stream_get_position(self.0) }
    }
}

impl Drop for RoFileImpl {
    fn drop(&mut self) {
        unsafe {
            stream_close(self.0, core::ptr::null_mut());
        }
    }
}

impl RwFileImpl {
    pub fn open(fref: &FileRefImpl) -> Result<Self> {
        unsafe {
            let strid = stream_open_file(fref.0, FileMode::ReadWrite, 0);
            if strid.is_null() {
                Err(GlkError)
            } else {
                Ok(Self(strid))
            }
        }
    }
}

impl ReadFile for RwFileImpl {
    fn read(&self, buf: &mut [u8]) -> u32 {
        unsafe { get_buffer_stream(self.0, buf.as_mut_ptr().cast(), buf.len() as u32) }
    }
}

impl SeekFile for RwFileImpl {
    fn seek(&self, pos: i32, mode: SeekMode) {
        unsafe { stream_set_position(self.0, pos, mode) }
    }

    fn pos(&self) -> u32 {
        unsafe { stream_get_position(self.0) }
    }
}

impl WriteFile for RwFileImpl {
    fn write(&self, buf: &[u8]) {
        unsafe {
            put_buffer_stream(self.0, buf.as_ptr().cast(), buf.len() as u32);
        }
    }
}

impl Write for RwFileImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            put_buffer_stream(self.0, s.as_ptr().cast(), s.len() as u32);
        }
        Ok(())
    }
}

impl Drop for RwFileImpl {
    fn drop(&mut self) {
        unsafe {
            stream_close(self.0, core::ptr::null_mut());
        }
    }
}

pub fn event_wait() -> Event {
    let mut ev = GlkEvent::default();

    unsafe {
        loop {
            select(&mut ev);

            return match EvType::try_from(ev.evtype).unwrap_or_default() {
                EvType::None => continue,
                EvType::Timer => Event::Timer,
                EvType::CharInput => {
                    if let Some(ch) = char::from_u32(ev.val1) {
                        Event::CharInput {
                            win: ev.win,
                            input: ch,
                        }
                    } else if let Ok(code) = Keycode::try_from(ev.val1) {
                        Event::CharInputSpecial {
                            win: ev.win,
                            input: code,
                        }
                    } else {
                        unreachable!(
                            "character input should be valid unicode or a valid special keycode"
                        )
                    }
                }
                EvType::LineInput => {
                    let mut s = String::new();

                    if gestalt(Gestalt::Unicode, 0) != 0 {
                        for i in 0..ev.val1 {
                            s.push(char::from_u32_unchecked(glkarea_get_word(4 * i)));
                        }
                    } else {
                        for i in 0..ev.val1 {
                            s.push(char::from_u32_unchecked(glkarea_get_byte(i)));
                        }
                    }
                    GLKAREA_IN_USE = false;

                    Event::LineInput {
                        win: ev.win,
                        input: s,
                        terminator: Keycode::try_from(ev.val2).ok(),
                    }
                }
                EvType::MouseInput => Event::MouseInput {
                    win: ev.win,
                    x: ev.val1,
                    y: ev.val2,
                },
                EvType::Arrange => Event::Arrange { win: ev.win },
                EvType::Redraw => Event::Redraw { win: ev.win },
                EvType::SoundNotify => Event::SoundNotify {
                    resource: ev.val1,
                    notify: ev.val2,
                },
                EvType::Hyperlink => Event::Hyerlink {
                    win: ev.win,
                    linkid: ev.val1,
                },
                EvType::VolumeNotify => Event::VolumeNotify { notify: ev.val1 },
            };
        }
    }
}

pub fn request_char(win: WinId) -> Result<()> {
    unsafe {
        if gestalt(Gestalt::Unicode, 0) != 0 {
            request_char_event_uni(win);
            Ok(())
        } else {
            request_char_event(win);
            Ok(())
        }
    }
}

pub fn cancel_char(win: WinId) {
    unsafe {
        cancel_char_event(win);
    }
}

pub fn request_line(win: WinId, initial: &str) -> Result<()> {
    unsafe {
        if GLKAREA_IN_USE {
            return Err(GlkError);
        }
        GLKAREA_IN_USE = true;

        let glksize = glkarea_size();
        let mut chars = 0;
        if gestalt(Gestalt::Unicode, 0) != 0 {
            for (i, ch) in initial.chars().enumerate() {
                if (4 * i + 3) as u32 >= glksize {
                    break;
                }
                glkarea_put_word((4 * i) as u32, ch.into());
                chars += 1;
            }
            request_line_event_uni(win, 0, glksize / 4, chars);
        } else {
            for (i, ch) in initial.chars().enumerate() {
                if i as u32 >= glksize {
                    break;
                }
                if ch as u32 > 255 {
                    glkarea_put_byte(i as u32, b'?' as u32);
                } else {
                    glkarea_put_byte(i as u32, ch as u32);
                }
                chars += 1;
            }
            request_line_event(win, 0, glksize, chars);
        }
        Ok(())
    }
}

pub fn cancel_line(win: WinId) -> String {
    unsafe {
        let mut out = String::new();
        let mut ev = GlkEvent::default();

        cancel_line_event(win, &mut ev);
        if gestalt(Gestalt::Unicode, 0) != 0 {
            for i in 0..ev.val1 {
                out.push(char::from_u32_unchecked(glkarea_get_word(4 * i)));
            }
        } else {
            for i in 0..ev.val1 {
                out.push(char::from_u32_unchecked(glkarea_get_byte(i)));
            }
        }
        GLKAREA_IN_USE = false;
        out
    }
}

pub fn request_mouse(win: WinId) -> Result<()> {
    unsafe {
        let wintype = window_get_type(win);
        if gestalt(Gestalt::MouseInput, wintype.into()) == 0 {
            return Err(GlkError);
        }
        request_mouse_event(win);
        Ok(())
    }
}

pub fn cancel_mouse(win: WinId) -> Result<()> {
    unsafe {
        let wintype = window_get_type(win);
        if gestalt(Gestalt::MouseInput, wintype.into()) == 0 {
            return Err(GlkError);
        }
        cancel_mouse_event(win);
        Ok(())
    }
}

pub fn request_hyperlink(win: WinId) -> Result<()> {
    unsafe {
        let wintype = window_get_type(win);
        if gestalt(Gestalt::Hyperlinks, 0) != 0
            && gestalt(Gestalt::HyperlinkInput, wintype.into()) != 0
        {
            request_hyperlink_event(win);
            Ok(())
        } else {
            Err(GlkError)
        }
    }
}

pub fn cancel_hyperlink(win: WinId) -> Result<()> {
    unsafe {
        let wintype = window_get_type(win);
        if gestalt(Gestalt::Hyperlinks, 0) != 0
            && gestalt(Gestalt::HyperlinkInput, wintype.into()) != 0
        {
            cancel_hyperlink_event(win);
            Ok(())
        } else {
            Err(GlkError)
        }
    }
}

pub fn set_style_hint(style: Style, class: StyleClass, hint: StyleHint) {
    let glk_style = style.to_glk();
    let wintype = class.to_glk();

    unsafe {
        match hint {
            StyleHint::Indentation(Some(i)) => {
                stylehint_set(wintype, glk_style, GlkStyleHint::Indentation, i);
            }
            StyleHint::Indentation(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::Indentation);
            }
            StyleHint::ParaIndentation(Some(i)) => {
                stylehint_set(wintype, glk_style, GlkStyleHint::ParaIndentation, i);
            }
            StyleHint::ParaIndentation(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::ParaIndentation);
            }
            StyleHint::Justification(Some(justification)) => stylehint_set(
                wintype,
                glk_style,
                GlkStyleHint::Justification,
                match justification {
                    Justification::LeftFlush => 0,
                    Justification::LeftRight => 1,
                    Justification::Centered => 2,
                    Justification::RightFlush => 3,
                },
            ),
            StyleHint::Justification(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::Justification);
            }
            StyleHint::Size(Some(size)) => {
                stylehint_set(wintype, glk_style, GlkStyleHint::Size, size);
            }
            StyleHint::Size(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::Size);
            }
            StyleHint::Weight(Some(weight)) => stylehint_set(
                wintype,
                glk_style,
                GlkStyleHint::Weight,
                match weight {
                    Weight::Light => -1,
                    Weight::Normal => 0,
                    Weight::Bold => 1,
                },
            ),
            StyleHint::Weight(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::Weight);
            }
            StyleHint::Oblique(Some(oblique)) => {
                stylehint_set(wintype, glk_style, GlkStyleHint::Oblique, oblique.into());
            }
            StyleHint::Oblique(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::Oblique);
            }
            StyleHint::Proportional(Some(proportional)) => {
                stylehint_set(
                    wintype,
                    glk_style,
                    GlkStyleHint::Proportional,
                    proportional.into(),
                );
            }
            StyleHint::Proportional(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::Proportional);
            }
            StyleHint::TextColor(Some(Color { r, g, b })) => {
                stylehint_set(
                    wintype,
                    glk_style,
                    GlkStyleHint::TextColor,
                    ((r as i32) << 16) | ((g as i32) << 8) | (b as i32),
                );
            }
            StyleHint::TextColor(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::TextColor);
            }
            StyleHint::BackColor(Some(Color { r, g, b })) => {
                stylehint_set(
                    wintype,
                    glk_style,
                    GlkStyleHint::BackColor,
                    ((r as i32) << 16) | ((g as i32) << 8) | (b as i32),
                );
            }
            StyleHint::BackColor(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::BackColor);
            }
            StyleHint::ReverseColor(Some(reverse)) => {
                stylehint_set(
                    wintype,
                    glk_style,
                    GlkStyleHint::ReverseColor,
                    reverse.into(),
                );
            }
            StyleHint::ReverseColor(None) => {
                stylehint_clear(wintype, glk_style, GlkStyleHint::ReverseColor);
            }
        }
    }
}

fn do_split(
    mut win: WinId,
    method: WinMethod,
    size: u32,
    wintype: WinType,
    mut nth_parent: u32,
) -> Result<WinId> {
    unsafe {
        while nth_parent > 0 {
            win = window_get_parent(win);
            if win.is_null() {
                return Err(GlkError);
            }
            nth_parent -= 1;
        }

        let result = window_open(win, method, size, wintype, 0);
        if result.is_null() {
            Err(GlkError)
        } else {
            Ok(result)
        }
    }
}

fn do_resize(win: WinId, size: u32, fixed: bool, mut nth_parent: u32) {
    let size = if !fixed && size > 100 { 100 } else { size };
    if nth_parent == 0 {
        return;
    }
    unsafe {
        let mut parent = window_get_parent(win);
        nth_parent -= 1;
        if parent.is_null() {
            return;
        }

        while nth_parent > 0 {
            parent = window_get_parent(parent);
            nth_parent -= 1;
            if parent.is_null() {
                return;
            }
        }

        let mut method = WinMethod::empty();

        window_get_arrangement(
            parent,
            &mut method,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        method &= WinMethod::DIVISION_MASK.complement();
        method |= if fixed {
            WinMethod::FIXED
        } else {
            WinMethod::PROPORTIONAL
        };
        window_set_arrangement(parent, method, size, win);
    }
}

fn do_measure(win: WinId, style: Style) -> StyleMeasurement {
    let glk_style = style.to_glk();

    unsafe {
        StyleMeasurement {
            indentation: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::Indentation, &mut result);
                if ok != 0 {
                    Some(result as i32)
                } else {
                    None
                }
            },
            para_indentation: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::ParaIndentation, &mut result);
                if ok != 0 {
                    Some(result as i32)
                } else {
                    None
                }
            },
            justification: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::Justification, &mut result);
                if ok != 0 {
                    Some(
                        match StyleHintJustification::try_from(result)
                            .expect("Justification style hint should be a valid enum member")
                        {
                            StyleHintJustification::LeftFlush => Justification::LeftFlush,
                            StyleHintJustification::LeftRight => Justification::LeftRight,
                            StyleHintJustification::Centered => Justification::Centered,
                            StyleHintJustification::RightFlush => Justification::RightFlush,
                        },
                    )
                } else {
                    None
                }
            },
            size: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::Size, &mut result);
                if ok != 0 {
                    Some(result as i32)
                } else {
                    None
                }
            },
            weight: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::Weight, &mut result);
                if ok != 0 {
                    Some(if (result as i32) < 0 {
                        Weight::Light
                    } else if result == 0 {
                        Weight::Normal
                    } else {
                        Weight::Bold
                    })
                } else {
                    None
                }
            },
            oblique: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::Oblique, &mut result);
                if ok != 0 {
                    Some(result != 0)
                } else {
                    None
                }
            },
            proportional: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::Proportional, &mut result);
                if ok != 0 {
                    Some(result != 0)
                } else {
                    None
                }
            },
            text_color: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::TextColor, &mut result);
                if ok != 0 {
                    Some(Color {
                        r: (result >> 16) as u8,
                        g: (result >> 8) as u8,
                        b: result as u8,
                    })
                } else {
                    None
                }
            },
            back_color: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::BackColor, &mut result);
                if ok != 0 {
                    Some(Color {
                        r: (result >> 16) as u8,
                        g: (result >> 8) as u8,
                        b: result as u8,
                    })
                } else {
                    None
                }
            },
            reverse_color: {
                let mut result = 0;
                let ok = style_measure(win, glk_style, GlkStyleHint::ReverseColor, &mut result);
                if ok != 0 {
                    Some(result != 0)
                } else {
                    None
                }
            },
        }
    }
}

fn do_distinguish(win: WinId, style1: Style, style2: Style) -> bool {
    unsafe { style_distinguish(win, style1.to_glk(), style2.to_glk()) != 0 }
}
