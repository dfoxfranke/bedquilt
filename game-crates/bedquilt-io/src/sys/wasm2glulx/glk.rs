use crate::error::{Error, Result};
use crate::fs::{FileMode, FileUsage, ReadFile, SeekFile, SeekMode, WriteFile};
use crate::win::{
    Color, Event, GraphicsWindowSize, Justification, Style, StyleClass, StyleHint,
    StyleMeasurement, TextWindowSize, Weight, WindowPosition,
};
use alloc::string::String;
use core::time::Duration;
use core::{ffi::CStr, fmt::Write};
pub use wasm2glulx_ffi::glk::WinId;
use wasm2glulx_ffi::{
    glk::{
        cancel_char_event, cancel_hyperlink_event, cancel_line_event, cancel_mouse_event,
        current_time, date_to_time_utc, exit, fileref_create_by_name, fileref_create_by_prompt,
        fileref_create_from_fileref, fileref_create_temp, fileref_delete_file, fileref_destroy,
        fileref_does_file_exist, fileref_get_rock, gestalt, get_buffer_stream, image_draw,
        image_draw_scaled, image_get_info, put_buffer_stream, put_char_stream, put_char_stream_uni,
        request_char_event, request_char_event_uni, request_hyperlink_event, request_line_event,
        request_line_event_uni, request_mouse_event, request_timer_events, schannel_create_ext,
        schannel_destroy, schannel_pause, schannel_play_ext, schannel_set_volume_ext,
        schannel_stop, schannel_unpause, select, set_echo_line_event, set_hyperlink_stream,
        set_style_stream, set_terminators_line_event, stream_close, stream_get_position,
        stream_open_file, stream_open_file_uni, stream_open_resource, stream_set_current,
        stream_set_position, style_distinguish, style_measure, stylehint_clear, stylehint_set,
        time_to_date_local, window_clear, window_close, window_erase_rect, window_fill_rect,
        window_flow_break, window_get_arrangement, window_get_echo_stream, window_get_parent,
        window_get_root, window_get_size, window_get_stream, window_get_type, window_iterate,
        window_move_cursor, window_open, window_set_arrangement, window_set_background_color,
        window_set_echo_stream, Date, EvType, Event as GlkEvent, FileUsage as GlkFileUsage, FrefId,
        Gestalt, Keycode, SchanId, StrId, StyleHint as GlkStyleHint, StyleHintJustification,
        Timeval, WinMethod, WinType,
    },
    glulx::{glkarea_get_byte, glkarea_get_word, glkarea_put_byte, glkarea_put_word, glkarea_size},
};

static mut GLKAREA_IN_USE: bool = false;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct WindowImpl(WinId);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileRefImpl(FrefId);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RoFileImpl(StrId);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RwFileImpl(StrId);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SchanImpl(SchanId);

impl WindowImpl {
    pub fn id(&self) -> WinId {
        self.0
    }

    pub fn resplit_proportional(&self, percent: u32, nth_parent: u32) {
        do_resplit(self.0, percent, false, nth_parent);
    }

    pub fn resplit_text(&self, size: TextWindowSize, nth_parent: u32) {
        match size {
            TextWindowSize::Proportional { percent } => {
                do_resplit(self.0, percent, false, nth_parent)
            }
            TextWindowSize::Fixed { chars } => do_resplit(self.0, chars, true, nth_parent),
        }
    }

    pub fn resplit_graphics(&self, size: GraphicsWindowSize, nth_parent: u32) {
        match size {
            GraphicsWindowSize::Proportional { percent } => {
                do_resplit(self.0, percent, false, nth_parent)
            }
            GraphicsWindowSize::Fixed { pixels } => do_resplit(self.0, pixels, true, nth_parent),
        }
    }

    pub fn set_hyperlink(&self, linkval: u32) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::Hyperlinks, 0) != 0
                && gestalt(Gestalt::HyperlinkInput, WinType::TextBuffer.into()) != 0
            {
                set_hyperlink_stream(window_get_stream(self.0), linkval);
                Ok(())
            } else {
                Err(Error::Unsupported)
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
                return Err(Error::GlkError);
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
                return Err(Error::GlkError);
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

    pub fn create_as_root(wintype: WinType) -> Result<Self> {
        unsafe {
            let win = window_open(WinId::null(), WinMethod::empty(), 0, wintype, 0);
            if win.is_null() {
                Err(Error::GlkError)
            } else {
                let this = Self(win);
                this.set_as_current();
                Ok(this)
            }
        }
    }

    pub fn split_blank(
        &self,
        position: WindowPosition,
        mut size_pct: u32,
        border: bool,
        nth_parent: u32,
    ) -> Result<WindowImpl> {
        if size_pct > 100 {
            size_pct = 100;
        }
        Ok(WindowImpl(do_split(
            self.0,
            position.to_method(false, border),
            size_pct,
            WinType::Blank,
            nth_parent,
        )?))
    }

    pub fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<WindowImpl> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(WindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextBuffer,
            nth_parent,
        )?))
    }

    pub fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<WindowImpl> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(WindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::TextGrid,
            nth_parent,
        )?))
    }

    pub fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<WindowImpl> {
        let (method, size) = size.to_method_and_size(position, border);
        Ok(WindowImpl(do_split(
            self.0,
            method,
            size,
            WinType::Graphics,
            nth_parent,
        )?))
    }

    pub fn set_style(&self, style: Style) {
        unsafe {
            set_style_stream(window_get_stream(self.0), style.to_glk());
        }
    }

    pub fn measure_style(&self, style: Style) -> StyleMeasurement {
        do_measure(self.0, style)
    }

    pub fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        do_distinguish(self.0, style1, style2)
    }

    pub fn move_cursor(&self, x: u32, y: u32) {
        unsafe {
            window_move_cursor(self.0, x, y);
        }
    }

    pub fn set_as_current(&self) {
        unsafe {
            stream_set_current(window_get_stream(self.0));
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        unsafe {
            let mut x = 0;
            let mut y = 0;

            window_get_size(self.0, &mut x, &mut y);
            (x, y)
        }
    }

    pub fn clear(&self) {
        unsafe {
            window_clear(self.0);
        }
    }

    pub fn draw_image(&self, image: u32, val1: i32, val2: i32) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::DrawImage, window_get_type(self.0).into()) == 0 {
                Err(Error::Unsupported)
            } else if image_draw(self.0, image, val1, val2) != 0 {
                Ok(())
            } else {
                Err(Error::GlkError)
            }
        }
    }

    pub fn draw_image_scaled(
        &self,
        image: u32,
        val1: i32,
        val2: i32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::DrawImage, window_get_type(self.0).into()) == 0 {
                Err(Error::Unsupported)
            } else if image_draw_scaled(self.0, image, val1, val2, width, height) != 0 {
                Ok(())
            } else {
                Err(Error::GlkError)
            }
        }
    }

    pub fn set_background_color(&self, color: Color) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::Graphics, 0) == 0 {
                Err(Error::Unsupported)
            } else {
                window_set_background_color(self.0, color.into());
                Ok(())
            }
        }
    }

    pub fn fill_rect(
        &self,
        color: Color,
        left: i32,
        top: i32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::Graphics, 0) == 0 {
                Err(Error::Unsupported)
            } else {
                window_fill_rect(self.0, color.into(), left, top, width, height);
                Ok(())
            }
        }
    }

    pub fn erase_rect(&self, left: i32, top: i32, width: u32, height: u32) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::Graphics, 0) == 0 {
                Err(Error::Unsupported)
            } else {
                window_erase_rect(self.0, left, top, width, height);
                Ok(())
            }
        }
    }

    pub fn flow_break(&self) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::Graphics, 0) == 0 {
                Err(Error::Unsupported)
            } else {
                window_flow_break(self.0);
                Ok(())
            }
        }
    }

    pub fn set_echo(&self, echo: bool) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::LineInputEcho, 0) == 0 {
                Err(Error::Unsupported)
            } else {
                set_echo_line_event(self.0, echo.into());
                Ok(())
            }
        }
    }

    pub fn set_terminators(&self, terminators: &[Keycode]) -> Result<()> {
        unsafe {
            if gestalt(Gestalt::LineTerminators, 0) == 0 {
                Err(Error::Unsupported)
            } else {
                for terminator in terminators {
                    if gestalt(Gestalt::LineTerminatorKey, (*terminator).into()) == 0 {
                        return Err(Error::Unsupported);
                    }
                }
                set_terminators_line_event(self.0, terminators.as_ptr(), terminators.len() as u32);
                Ok(())
            }
        }
    }
}

impl Drop for WindowImpl {
    fn drop(&mut self) {
        self.transcript_off();
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
                Err(Error::GlkError)
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
                Err(Error::GlkError)
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
                Err(Error::GlkError)
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
                Err(Error::GlkError)
            } else {
                Ok(Self(strid))
            }
        }
    }

    pub fn open_resource(num: u32) -> Result<Self> {
        unsafe {
            let strid = stream_open_resource(num, 0);
            if strid.is_null() {
                Err(Error::GlkError)
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
                Err(Error::GlkError)
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

impl Write for &RwFileImpl {
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

impl SchanImpl {
    pub fn new(volume: u32) -> Result<Self> {
        unsafe {
            if gestalt(Gestalt::Sound2, 0) != 0 {
                Ok(Self(schannel_create_ext(0, volume)))
            } else {
                Err(Error::Unsupported)
            }
        }
    }

    pub fn play(&self, snd: u32, repeats: u32, notify: u32) -> Result<()> {
        unsafe {
            if schannel_play_ext(self.0, snd, repeats, notify) != 0 {
                Ok(())
            } else {
                Err(Error::GlkError)
            }
        }
    }

    pub fn pause(&self) {
        unsafe {
            schannel_pause(self.0);
        }
    }

    pub fn unpause(&self) {
        unsafe {
            schannel_unpause(self.0);
        }
    }

    pub fn stop(&self) {
        unsafe {
            schannel_stop(self.0);
        }
    }

    pub fn set_volume(&self, volume: u32, duration: u32, notify: u32) {
        unsafe {
            schannel_set_volume_ext(self.0, volume, duration, notify);
        }
    }
}

impl Drop for SchanImpl {
    fn drop(&mut self) {
        unsafe {
            schannel_destroy(self.0);
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
                    } else {
                        let code = Keycode::try_from(ev.val1).unwrap_or_default();
                        Event::CharInputSpecial {
                            win: ev.win,
                            input: code,
                        }
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
                        terminator: if ev.val2 == 0 {
                            None
                        } else {
                            Some(Keycode::try_from(ev.val2).unwrap_or_default())
                        },
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
                EvType::Hyperlink => Event::Hyperlink {
                    win: ev.win,
                    linkid: ev.val1,
                },
                EvType::VolumeNotify => Event::VolumeNotify { notify: ev.val1 },
            };
        }
    }
}

pub fn write_str(win: WinId, s: &str) -> core::fmt::Result {
    unsafe {
        let stream = window_get_stream(win);
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

pub fn request_char(win: WinId) -> Result<()> {
    unsafe {
        if window_get_type(win) == WinType::Graphics && gestalt(Gestalt::GraphicsCharInput, 0) == 0
        {
            Err(Error::Unsupported)
        } else if gestalt(Gestalt::Unicode, 0) != 0 {
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
            return Err(Error::GlkAreaAllocError);
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
            return Err(Error::Unsupported);
        }
        request_mouse_event(win);
        Ok(())
    }
}

pub fn cancel_mouse(win: WinId) -> Result<()> {
    unsafe {
        let wintype = window_get_type(win);
        if gestalt(Gestalt::MouseInput, wintype.into()) == 0 {
            return Err(Error::Unsupported);
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
            Err(Error::Unsupported)
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
            Err(Error::Unsupported)
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

pub fn set_tick_interval(millisecs: u32) -> Result<()> {
    unsafe {
        if gestalt(Gestalt::Timer, 0) == 0 {
            Err(Error::Unsupported)
        } else {
            request_timer_events(millisecs);
            Ok(())
        }
    }
}

pub fn get_time_and_offset() -> Result<(Duration, i32)> {
    unsafe {
        if gestalt(Gestalt::DateTime, 0) == 0 {
            Err(Error::Unsupported)
        } else {
            let mut utc_time = Timeval::default();
            let mut local_time = Timeval::default();
            let mut local_date = Date::default();

            current_time(&mut utc_time);

            if utc_time.high_sec < 0 {
                return Err(Error::GlkError);
            }

            time_to_date_local(&utc_time, &mut local_date);
            date_to_time_utc(&local_date, &mut local_time);

            let signed_utc = ((utc_time.high_sec as i64) << 32) | utc_time.low_sec as i64;
            let signed_local = ((local_time.high_sec as i64) << 32) | local_time.low_sec as i64;

            let offset = i32::try_from(
                signed_local
                    .checked_sub(signed_utc)
                    .ok_or(Error::GlkError)?,
            )
            .map_err(|_| Error::GlkError)?;

            let sec = ((utc_time.high_sec as u64) << 32) | utc_time.low_sec as u64;
            let duration =
                Duration::from_secs(sec) + Duration::from_micros(utc_time.microsec as u64);
            Ok((duration, offset))
        }
    }
}

pub fn image_size(image: u32) -> Result<(u32, u32)> {
    unsafe {
        if gestalt(Gestalt::Graphics, 0) == 0 {
            Err(Error::Unsupported)
        } else {
            let mut x = 0;
            let mut y = 0;
            if image_get_info(image, &mut x, &mut y) != 0 {
                Ok((x, y))
            } else {
                Err(Error::GlkError)
            }
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(test), panic_handler)]
fn panic_handler(info: &core::panic::PanicInfo<'_>) -> ! {
    struct RawWrite(WinId);

    impl core::fmt::Write for RawWrite {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            write_str(self.0, s)
        }
    }

    unsafe {
        let root = window_get_root();
        let winid = if root.is_null() {
            let winid = window_open(WinId::null(), WinMethod::empty(), 0, WinType::TextBuffer, 0);
            if winid.is_null() {
                exit()
            } else {
                winid
            }
        } else {
            let mut winid = window_iterate(WinId::null(), core::ptr::null_mut());
            while !winid.is_null() {
                if window_get_type(winid) == WinType::TextBuffer {
                    break;
                }
                winid = window_iterate(WinId::null(), core::ptr::null_mut());
            }
            if winid.is_null() {
                winid = window_open(
                    root,
                    WinMethod::BELOW | WinMethod::FIXED,
                    3,
                    WinType::TextBuffer,
                    0,
                );
            }
            if winid.is_null() {
                exit();
            }
            winid
        };

        cancel_line_event(winid, core::ptr::null_mut());
        let _ = writeln!(RawWrite(winid), "{}", info);
        exit();
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
                return Err(Error::GlkError);
            }
            nth_parent -= 1;
        }

        let result = window_open(win, method, size, wintype, 0);
        if result.is_null() {
            Err(Error::GlkError)
        } else {
            Ok(result)
        }
    }
}

fn do_resplit(win: WinId, size: u32, fixed: bool, mut nth_parent: u32) {
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
