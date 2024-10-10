// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
// Copyright 2024 Daniel Fox Franke.
use bitflags::bitflags;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use core::ffi::c_char;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum Gestalt {
    Version = 0,
    CharInput = 1,
    LineInput = 2,
    CharOutput = 3,
    MouseInput = 4,
    Timer = 5,
    Graphics = 6,
    DrawImage = 7,
    Sound = 8,
    SoundVolume = 9,
    SoundNotify = 10,
    Hyperlinks = 11,
    HyperlinkInput = 12,
    SoundMusic = 13,
    GraphicsTransparency = 14,
    Unicode = 15,
    UnicodeNorm = 16,
    LineInputEcho = 17,
    LineTerminators = 18,
    LineTerminatorKey = 19,
    DateTime = 20,
    Sound2 = 21,
    ResourceStream = 22,
    GraphicsCharInput = 23,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum GestaltCharOutput {
    CannotPrint = 0,
    ApproxPrint = 1,
    ExactPrint = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, IntoPrimitive, TryFromPrimitive)]
pub enum EvType {
    #[default]
    None = 0,
    Timer = 1,
    CharInput = 2,
    LineInput = 3,
    MouseInput = 4,
    Arrange = 5,
    Redraw = 6,
    SoundNotify = 7,
    Hyperlink = 8,
    VolumeNotify = 9,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Event {
    /// You can call [`EvType`]`::try_from` on this field, but it cannot *be* an
    /// `EvType` because future Glk versions or extensions may add new event
    /// types, and if those are not accounted for in the enumeration then UB
    /// could result.
    pub evtype: u32,
    pub win: WinId,
    pub val1: u32,
    pub val2: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, IntoPrimitive, TryFromPrimitive)]
pub enum Keycode {
    #[default]
    Unknown = 0xffffffff,
    Left = 0xfffffffe,
    Right = 0xfffffffd,
    Up = 0xfffffffc,
    Down = 0xfffffffb,
    Return = 0xfffffffa,
    Delete = 0xfffffff9,
    Escape = 0xfffffff8,
    Tab = 0xfffffff7,
    PageUp = 0xfffffff6,
    PageDown = 0xfffffff5,
    Home = 0xfffffff4,
    End = 0xfffffff3,
    Func1 = 0xffffffef,
    Func2 = 0xffffffee,
    Func3 = 0xffffffed,
    Func4 = 0xffffffec,
    Func5 = 0xffffffeb,
    Func6 = 0xffffffea,
    Func7 = 0xffffffe9,
    Func8 = 0xffffffe8,
    Func9 = 0xffffffe7,
    Func10 = 0xffffffe6,
    Func11 = 0xffffffe5,
    Func12 = 0xffffffe4,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, IntoPrimitive, TryFromPrimitive)]
pub enum Style {
    #[default]
    Normal = 0,
    Emphasized = 1,
    Preformatted = 2,
    Header = 3,
    Subheader = 4,
    Alert = 5,
    Note = 6,
    BlockQuote = 7,
    Input = 8,
    User1 = 9,
    User2 = 10,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct StreamResult {
    pub readcount: u32,
    pub writecount: u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, IntoPrimitive, TryFromPrimitive)]
pub enum WinType {
    #[default]
    AllTypes = 0,
    Pair = 1,
    Blank = 2,
    TextBuffer = 3,
    TextGrid = 4,
    Graphics = 5,
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct WinMethod : u32 {
        const LEFT = 0x00;
        const RIGHT = 0x01;
        const ABOVE = 0x02;
        const BELOW = 0x03;
        const DIR_MASK = 0xf;

        const FIXED = 0x10;
        const PROPORTIONAL = 0x20;
        const DIVISION_MASK = 0xf0;

        const BORDER = 0x000;
        const NO_BORDER = 0x100;
        const BORDER_MASK = 0x100;
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct FileUsage : u32 {
        const DATA = 0x00;
        const SAVED_GAME = 0x01;
        const TRANSCRIPT = 0x02;
        const INPUT_RECORD = 0x03;
        const TYPE_MASK = 0x0f;

        const TEXT_MODE = 0x100;
        const BINARY_MODE = 0x000;
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum FileMode {
    Write = 0x01,
    Read = 0x02,
    ReadWrite = 0x03,
    WriteAppend = 0x05,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum SeekMode {
    Start = 0,
    Current = 1,
    End = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum StyleHint {
    Indentation = 0,
    ParaIndentation = 1,
    Justification = 2,
    Size = 3,
    Weight = 4,
    Oblique = 5,
    Proportional = 6,
    TextColor = 7,
    BackColor = 8,
    ReverseColor = 9,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum StyleHintJustification {
    LeftFlush = 0,
    LeftRight = 1,
    Centered = 2,
    RightFlush = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
pub enum ImageAlign {
    InlineUp = 0x01,
    InlineDown = 0x02,
    InlineCenter = 0x03,
    MarginLeft = 0x04,
    MarginRight = 0x05,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timeval {
    pub high_sec: i32,
    pub low_sec: u32,
    pub microsec: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Date {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub weekday: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub microsec: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct WinId(u32);

impl WinId {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct StrId(u32);

impl StrId {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct FrefId(u32);

impl FrefId {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct SchanId(u32);

impl SchanId {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[link(wasm_import_module = "glk")]
extern "C" {
    pub fn exit() -> !;
    pub fn tick();

    pub fn gestalt(sel: Gestalt, val: u32) -> u32;
    pub fn gestalt_ext(sel: Gestalt, val: u32, arr: *mut u32, arrlen: u32) -> u32;

    pub fn char_to_upper(ch: u32) -> u32;
    pub fn char_to_lower(ch: u32) -> u32;

    pub fn window_get_root() -> WinId;
    pub fn window_open(
        split: WinId,
        method: WinMethod,
        size: u32,
        wintype: WinType,
        rock: u32,
    ) -> WinId;
    pub fn window_close(win: WinId, result: *mut StreamResult);
    pub fn window_get_size(win: WinId, widthptr: *mut u32, heightputr: *mut u32);
    pub fn window_set_arrangement(win: WinId, method: WinMethod, size: u32, keywin: WinId);
    pub fn window_get_arrangement(
        win: WinId,
        methodptr: *mut WinMethod,
        sizeptr: *mut u32,
        keywinptr: *mut WinId,
    );
    pub fn window_iterate(win: WinId, rockptr: *mut u32) -> WinId;
    pub fn window_get_rock(win: WinId) -> u32;
    pub fn window_get_type(win: WinId) -> WinType;
    pub fn window_get_parent(win: WinId) -> WinId;
    pub fn window_get_sibling(win: WinId) -> WinId;
    pub fn window_clear(win: WinId);
    pub fn window_move_cursor(win: WinId, xpos: u32, ypos: u32);
    pub fn window_get_stream(win: WinId) -> StrId;
    pub fn window_set_echo_stream(win: WinId, str: StrId);
    pub fn window_get_echo_stream(win: WinId) -> StrId;
    pub fn set_window(win: WinId);

    pub fn stream_open_file(fileref: FrefId, mode: FileMode, rock: u32) -> StrId;
    pub fn strem_open_memory(glkaddr: u32, buflen: u32, mode: FileMode, rock: u32) -> StrId;
    pub fn stream_close(str: StrId, result: *mut StreamResult);
    pub fn stream_iterate(str: StrId, rockptr: *mut u32) -> StrId;
    pub fn stream_get_rock(str: StrId) -> u32;
    pub fn stream_set_position(str: StrId, pos: i32, seekmode: SeekMode);
    pub fn stream_get_position(str: StrId) -> u32;
    pub fn stream_set_current(str: StrId);
    pub fn stream_get_current() -> StrId;

    pub fn put_char(ch: u32);
    pub fn put_char_stream(str: StrId, ch: u32);
    pub fn put_string(s: *const c_char);
    pub fn put_string_stream(str: StrId, s: *const c_char);
    pub fn put_buffer(buf: *const c_char, len: u32);
    pub fn put_buffer_stream(str: StrId, buf: *const c_char, len: u32);
    pub fn set_style(styl: Style);
    pub fn set_style_stream(str: StrId, styl: Style);

    pub fn get_char_stream(str: StrId) -> i32;
    pub fn get_line_stream(str: StrId, buf: *mut c_char, len: u32) -> u32;
    pub fn get_buffer_stream(str: StrId, buf: *mut c_char, len: u32) -> u32;

    pub fn stylehint_set(wintype: WinType, styl: Style, hint: StyleHint, val: i32);
    pub fn stylehint_clear(wintype: WinType, styl: Style, hint: StyleHint);
    pub fn style_distinguish(win: WinId, styl1: Style, styl2: Style) -> u32;
    pub fn style_measure(win: WinId, styl: Style, hint: StyleHint, result: *mut u32) -> u32;

    pub fn fileref_create_temp(usage: FileUsage, rock: u32) -> FrefId;
    pub fn fileref_create_by_name(usage: FileUsage, name: *const c_char, rock: u32) -> FrefId;
    pub fn fileref_create_by_prompt(usage: FileUsage, fmode: FileMode, rock: u32) -> FrefId;
    pub fn fileref_create_from_fileref(usage: FileUsage, fref: FrefId, rock: u32) -> FrefId;
    pub fn fileref_destroy(fref: FrefId);
    pub fn fileref_iterate(fref: FrefId, rockptr: *mut u32) -> FrefId;
    pub fn fileref_get_rock(fref: FrefId) -> u32;
    pub fn fileref_delete_file(fref: FrefId);
    pub fn fileref_does_file_exist(fref: FrefId) -> u32;

    pub fn select(event: *mut Event);
    pub fn select_poll(event: *mut Event);

    pub fn request_timer_events(millisecs: u32);

    pub fn request_line_event(win: WinId, glkaddr: u32, maxlen: u32, initlen: u32);
    pub fn request_char_event(win: WinId);
    pub fn request_mouse_event(win: WinId);

    pub fn cancel_line_event(win: WinId, event: *mut Event);
    pub fn cancel_char_event(win: WinId);
    pub fn cancel_mouse_event(win: WinId);

    pub fn set_echo_line_event(win: WinId, val: u32);
    pub fn set_terminators_line_event(win: WinId, keycodes: *const Keycode, count: u32);

    pub fn buffer_to_lower_case_uni(buf: *mut u32, len: u32, numchars: u32) -> u32;
    pub fn buffer_to_upper_case_uni(buf: *mut u32, len: u32, numchars: u32) -> u32;
    pub fn buffer_to_title_case_uni(buf: *mut u32, len: u32, numchars: u32, lowerrest: u32) -> u32;

    pub fn put_char_uni(ch: u32);
    pub fn put_string_uni(s: *const u32);
    pub fn put_buffer_uni(buf: *const u32, len: u32);
    pub fn put_char_stream_uni(str: StrId, ch: u32);
    pub fn put_string_stream_uni(str: StrId, s: *const u32);
    pub fn put_buffer_stream_uni(str: StrId, buf: *const u32, len: u32);

    pub fn get_char_stream_uni(str: StrId) -> i32;
    pub fn get_buffer_stream_uni(str: StrId, buf: *mut u32, len: u32) -> u32;
    pub fn get_line_stream_uni(str: StrId, buf: *mut u32, len: u32) -> u32;

    pub fn stream_open_file_uni(fileref: FrefId, mode: FileMode, rock: u32) -> StrId;
    pub fn stream_open_memory_uni(glkaddr: u32, buflen: u32, mode: FileMode, rock: u32) -> StrId;

    pub fn request_char_event_uni(win: WinId);
    pub fn request_line_event_uni(win: WinId, glkaddr: u32, maxlen: u32, initlen: u32);

    pub fn buffer_canon_decompose_uni(buf: *mut u32, len: u32, numchars: u32) -> u32;
    pub fn buffer_canon_normalize_uni(buf: *mut u32, len: u32, numchars: u32) -> u32;

    pub fn image_draw(win: WinId, image: u32, val1: i32, val2: i32) -> u32;
    pub fn image_draw_scaled(
        win: WinId,
        image: u32,
        val1: i32,
        val2: i32,
        width: u32,
        height: u32,
    ) -> u32;
    pub fn image_get_info(image: u32, width: *mut u32, height: *mut u32) -> u32;

    pub fn window_flow_break(win: WinId);

    pub fn window_erase_rect(win: WinId, left: i32, top: i32, width: u32, height: u32);
    pub fn window_fill_rect(win: WinId, color: u32, left: i32, top: i32, width: u32, height: u32);
    pub fn window_set_background_color(win: WinId, color: u32);

    pub fn schannel_create(rock: u32) -> SchanId;
    pub fn schannel_destroy(chan: SchanId);
    pub fn schannel_iterate(chan: SchanId, rockptr: *mut u32) -> SchanId;
    pub fn schannel_get_rock(chan: SchanId) -> u32;

    pub fn schannel_play(chan: SchanId, snd: u32) -> u32;
    pub fn schannel_play_ext(chan: SchanId, snd: u32, repeats: u32, notify: u32) -> u32;
    pub fn schannel_stop(chan: SchanId);
    pub fn schannel_set_volume(chan: SchanId, vol: u32);

    pub fn sound_load_hint(snd: u32, flag: u32);

    pub fn schannel_create_ext(rock: u32, volume: u32) -> SchanId;
    pub fn schannel_pause(chan: SchanId);
    pub fn schannel_unpause(chan: SchanId);
    pub fn schannel_set_volume_ext(chan: SchanId, vol: u32, duration: u32, notify: u32);

    pub fn set_hyperlink(linkval: u32);
    pub fn set_hyperlink_stream(str: StrId, linkval: u32);
    pub fn request_hyperlink_event(win: WinId);
    pub fn cancel_hyperlink_event(win: WinId);

    pub fn current_time(time: *mut Timeval);
    pub fn current_simple_time(factor: u32) -> i32;
    pub fn time_to_date_utc(time: *const Timeval, date: *mut Date);
    pub fn time_to_date_local(time: *const Timeval, date: *mut Date);
    pub fn simple_time_to_date_utc(time: i32, factor: u32, date: *mut Date);
    pub fn simple_time_to_date_local(time: i32, factor: u32, date: *mut Date);
    pub fn date_to_simple_time_utc(date: *const Date, factor: u32) -> i32;
    pub fn date_to_simple_time_local(date: *const Date, factor: u32) -> i32;

    pub fn stream_open_resource(filenum: u32, rock: u32) -> StrId;
    pub fn stream_open_resource_uni(flilnum: u32, rock: u32) -> StrId;
}
