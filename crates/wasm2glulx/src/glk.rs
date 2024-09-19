use std::collections::HashMap;
use std::sync::OnceLock;

use walrus::{ImportedFunction, ValType};

use crate::common::*;

#[derive(Debug, Copy, Clone)]
enum GlkParam {
    /// Parameter is a scalar, not a pointer
    Scalar,
    /// Parameter is a pointer to a scalar value this many words long
    ScalarPtr(u32),
    /// Parameter a pointer to an array of bytes, with length given by the
    /// indicated argument
    ByteArrayPtr(u32),
    /// Parameter a pointer to an array of words, with length given by the
    /// indicated argument
    WordArrayPtr(u32),
    /// Parameter is a pointer to a string terminated by a null byte
    Lat1Ptr,
    /// Parameter is a pointer to a string terminated by a null word
    UnicodePtr,
    /// Parameter is a pointer to an byte array in Glk-owned memory, with
    /// length given by the indicated argument.
    OwnedByteArrayPtr(u32),
    /// Parameter is a pointer to a word array in Glk-owned memory, with
    /// length given by the indicated argument.
    OwnedWordArrayPtr(u32),
}

#[derive(Debug, Copy, Clone)]
struct GlkFunction {
    name: &'static str,
    selector: u16,
    params: &'static [GlkParam],
    has_return: bool,
}

static GLK_FUNCTIONS: &[GlkFunction] = [
    GlkFunction {
        name: "exit",
        selector: 0x0001,
        params: &[],
        has_return: false,
    },
    GlkFunction {
        name: "tick",
        selector: 0x0003,
        params: &[],
        has_return: false,
    },
    GlkFunction {
        name: "gestalt",
        selector: 0x0004,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "gestalt_ext",
        selector: 0x0005,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(3),
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "window_iterate",
        selector: 0x0020,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(1)],
        has_return: true,
    },
    GlkFunction {
        name: "window_get_rock",
        selector: 0x0021,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "window_get_root",
        selector: 0x0022,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "window_open",
        selector: 0x0023,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "window_close",
        selector: 0x0024,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(2)],
        has_return: false,
    },
    GlkFunction {
        name: "window_get_size",
        selector: 0x0025,
        params: &[
            GlkParam::Scalar,
            GlkParam::ScalarPtr(1),
            GlkParam::ScalarPtr(1),
        ],
        has_return: false,
    },
    GlkFunction {
        name: "window_set_arrangement",
        selector: 0x0026,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "window_get_arrangement",
        selector: 0x0027,
        params: &[
            GlkParam::Scalar,
            GlkParam::ScalarPtr(1),
            GlkParam::ScalarPtr(1),
            GlkParam::ScalarPtr(1),
        ],
        has_return: false,
    },
    GlkFunction {
        name: "window_get_type",
        selector: 0x0028,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "window_get_parent",
        selector: 0x0029,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "window_clear",
        selector: 0x002a,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "window_move_cursor",
        selector: 0x002b,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "window_get_stream",
        selector: 0x002c,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "window_set_echo_stream",
        selector: 0x002d,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "window_get_echo_stream",
        selector: 0x002e,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "set_window",
        selector: 0x002f,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "window_get_sibling",
        selector: 0x0030,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "stream_iterate",
        selector: 0x0040,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(1)],
        has_return: true,
    },
    GlkFunction {
        name: "stream_get_rock",
        selector: 0x0041,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "stream_open_file",
        selector: 0x0042,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "stream_open_memory",
        selector: 0x0043,
        params: &[
            GlkParam::OwnedByteArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "stream_close",
        selector: 0x0044,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(2)],
        has_return: false,
    },
    GlkFunction {
        name: "stream_set_position",
        selector: 0x0045,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "stream_get_position",
        selector: 0x0046,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "stream_set_current",
        selector: 0x0047,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "stream_get_current",
        selector: 0x0048,
        params: &[],
        has_return: true,
    },
    GlkFunction {
        name: "stream_open_resource",
        selector: 0x0049,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_create_temp",
        selector: 0x0060,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_create_by_name",
        selector: 0x0061,
        params: &[GlkParam::Scalar, GlkParam::Lat1Ptr, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_create_by_prompt",
        selector: 0x0062,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_destroy",
        selector: 0x0063,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "fileref_iterate",
        selector: 0x0064,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(1)],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_get_rock",
        selector: 0x0065,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_delete_file",
        selector: 0x0066,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "fileref_does_file_exist",
        selector: 0x0067,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "fileref_create_from_fileref",
        selector: 0x0068,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "put_char",
        selector: 0x0080,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_char_stream",
        selector: 0x0081,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_string",
        selector: 0x0082,
        params: &[GlkParam::Lat1Ptr],
        has_return: false,
    },
    GlkFunction {
        name: "put_string_stream",
        selector: 0x0083,
        params: &[GlkParam::Scalar, GlkParam::Lat1Ptr],
        has_return: false,
    },
    GlkFunction {
        name: "put_buffer",
        selector: 0x0084,
        params: &[GlkParam::ByteArrayPtr(1), GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_buffer_stream",
        selector: 0x0085,
        params: &[
            GlkParam::Scalar,
            GlkParam::ByteArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "set_style",
        selector: 0x0086,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "set_style_stream",
        selector: 0x0087,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "get_char_stream",
        selector: 0x0090,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "get_line_stream",
        selector: 0x0091,
        params: &[
            GlkParam::Scalar,
            GlkParam::ByteArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "get_buffer_stream",
        selector: 0x0092,
        params: &[
            GlkParam::Scalar,
            GlkParam::ByteArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "char_to_lower",
        selector: 0x00a0,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "char_to_upper",
        selector: 0x00a1,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "stylehint_set",
        selector: 0x00b0,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "stylehint_clear",
        selector: 0x00b1,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "style_distinguish",
        selector: 0x00b2,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "style_measure",
        selector: 0x00b3,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::ScalarPtr(1),
        ],
        has_return: true,
    },
    GlkFunction {
        name: "select",
        selector: 0x00c0,
        params: &[GlkParam::ScalarPtr(4)],
        has_return: false,
    },
    GlkFunction {
        name: "select_poll",
        selector: 0x00c1,
        params: &[GlkParam::ScalarPtr(4)],
        has_return: false,
    },
    GlkFunction {
        name: "request_line_event",
        selector: 0x00d0,
        params: &[
            GlkParam::Scalar,
            GlkParam::OwnedByteArrayPtr(2),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "cancel_line_event",
        selector: 0x00d1,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(4)],
        has_return: false,
    },
    GlkFunction {
        name: "request_char_event",
        selector: 0x00d2,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "cancel_char_event",
        selector: 0x00d3,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "request_mouse_event",
        selector: 0x00d4,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "cancel_mouse_event",
        selector: 0x00d5,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "request_timer_events",
        selector: 0x00d6,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "image_get_info",
        selector: 0x00e0,
        params: &[
            GlkParam::Scalar,
            GlkParam::ScalarPtr(1),
            GlkParam::ScalarPtr(1),
        ],
        has_return: true,
    },
    GlkFunction {
        name: "image_draw",
        selector: 0x00e1,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "image_draw_scaled",
        selector: 0x00e2,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "window_flow_break",
        selector: 0x00e8,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "window_erase_rect",
        selector: 0x00e9,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "window_fill_rect",
        selector: 0x00ea,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "window_set_background_color",
        selector: 0x00eb,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "schannel_iterate",
        selector: 0x00f0,
        params: &[GlkParam::Scalar, GlkParam::ScalarPtr(1)],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_get_rock",
        selector: 0x00f1,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_create",
        selector: 0x00f2,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_destroy",
        selector: 0x00f3,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "schannel_create_ext",
        selector: 0x00f4,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_play_multi",
        selector: 0x00f7,
        params: &[
            GlkParam::WordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(3),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_play",
        selector: 0x00f8,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_play_ext",
        selector: 0x00f9,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "schannel_stop",
        selector: 0x00fa,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "schannel_set_volume",
        selector: 0x00fb,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "sound_load_hint",
        selector: 0x00fc,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "schannel_set_volume_ext",
        selector: 0x00fb,
        params: &[
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "schannel_pause",
        selector: 0x00fe,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "schannel_unpause",
        selector: 0x00ff,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "set_hyperlink",
        selector: 0x0100,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "set_hyperlink_stream",
        selector: 0x0101,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "request_hyperlink_event",
        selector: 0x0102,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "cancel_hyperlink_event",
        selector: 0x0103,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "buffer_to_lower_case_uni",
        selector: 0x0120,
        params: &[
            GlkParam::WordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "buffer_to_upper_case_uni",
        selector: 0x0121,
        params: &[
            GlkParam::WordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "buffer_to_title_case_uni",
        selector: 0x0122,
        params: &[
            GlkParam::WordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "buffer_canon_decompose_uni",
        selector: 0x0123,
        params: &[
            GlkParam::WordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "buffer_canon_normalize_uni",
        selector: 0x0124,
        params: &[
            GlkParam::WordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "put_char_uni",
        selector: 0x0128,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_string_uni",
        selector: 0x0129,
        params: &[GlkParam::UnicodePtr],
        has_return: false,
    },
    GlkFunction {
        name: "put_buffer_uni",
        selector: 0x012a,
        params: &[GlkParam::WordArrayPtr(1), GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_char_stream_uni",
        selector: 0x012b,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "put_string_stream_uni",
        selector: 0x012c,
        params: &[GlkParam::Scalar, GlkParam::UnicodePtr],
        has_return: false,
    },
    GlkFunction {
        name: "put_buffer_stream_uni",
        selector: 0x012d,
        params: &[
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "get_char_stream_uni",
        selector: 0x0130,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "get_buffer_stream_uni",
        selector: 0x0131,
        params: &[
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "get_line_stream_uni",
        selector: 0x0132,
        params: &[
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "stream_open_file_uni",
        selector: 0x0138,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "stream_open_memory_uni",
        selector: 0x0139,
        params: &[
            GlkParam::OwnedWordArrayPtr(1),
            GlkParam::Scalar,
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: true,
    },
    GlkFunction {
        name: "stream_open_resource_uni",
        selector: 0x013a,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "request_char_event_uni",
        selector: 0x0140,
        params: &[GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "request_line_event_uni",
        selector: 0x0141,
        params: &[
            GlkParam::Scalar,
            GlkParam::OwnedWordArrayPtr(2),
            GlkParam::Scalar,
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "set_echo_line_event",
        selector: 0x0150,
        params: &[GlkParam::Scalar, GlkParam::Scalar],
        has_return: false,
    },
    GlkFunction {
        name: "set_terminators_line_event",
        selector: 0x0151,
        params: &[
            GlkParam::Scalar,
            GlkParam::WordArrayPtr(2),
            GlkParam::Scalar,
        ],
        has_return: false,
    },
    GlkFunction {
        name: "current_time",
        selector: 0x0160,
        params: &[GlkParam::ScalarPtr(3)],
        has_return: false,
    },
    GlkFunction {
        name: "current_simple_time",
        selector: 0x0161,
        params: &[GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "time_to_date_utc",
        selector: 0x0168,
        params: &[GlkParam::ScalarPtr(3), GlkParam::ScalarPtr(8)],
        has_return: false,
    },
    GlkFunction {
        name: "time_to_date_local",
        selector: 0x0169,
        params: &[GlkParam::ScalarPtr(3), GlkParam::ScalarPtr(8)],
        has_return: false,
    },
    GlkFunction {
        name: "simple_time_to_date_utc",
        selector: 0x016a,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::ScalarPtr(8)],
        has_return: false,
    },
    GlkFunction {
        name: "simple_time_to_date_local",
        selector: 0x016b,
        params: &[GlkParam::Scalar, GlkParam::Scalar, GlkParam::ScalarPtr(8)],
        has_return: false,
    },
    GlkFunction {
        name: "date_to_time_utc",
        selector: 0x016c,
        params: &[GlkParam::ScalarPtr(8), GlkParam::ScalarPtr(3)],
        has_return: false,
    },
    GlkFunction {
        name: "date_to_time_local",
        selector: 0x016d,
        params: &[GlkParam::ScalarPtr(8), GlkParam::ScalarPtr(3)],
        has_return: false,
    },
    GlkFunction {
        name: "date_to_simple_time_utc",
        selector: 0x016e,
        params: &[GlkParam::ScalarPtr(8), GlkParam::Scalar],
        has_return: true,
    },
    GlkFunction {
        name: "date_to_simple_time_local",
        selector: 0x016f,
        params: &[GlkParam::ScalarPtr(8), GlkParam::Scalar],
        has_return: true,
    },
]
.as_slice();

fn get_glk_function(name: &str) -> Option<GlkFunction> {
    static GLK_FUNCTION_MAP: OnceLock<HashMap<&'static str, GlkFunction>> = OnceLock::new();

    let map = GLK_FUNCTION_MAP.get_or_init(|| GLK_FUNCTIONS.iter().map(|v| (v.name, *v)).collect());

    map.get(name).copied()
}

impl GlkFunction {
    fn codegen(&self, ctx: &mut Context, my_label: Label) {
        use glulx_asm::concise::*;
        let nargs: u32 = self.params.len().try_into().unwrap();
        let mem = ctx.layout.memory();
        let glk_area = ctx.layout.glk_area();

        ctx.rom_items.push(label(my_label));
        ctx.rom_items.push(fnhead_local(nargs));
        for (num, param) in self.params.iter().copied().rev().enumerate() {
            let argnum: u32 = num.try_into().unwrap();
            match param {
                GlkParam::Scalar => {
                    ctx.rom_items.push(copy(lloc(argnum), push()));
                }
                GlkParam::ByteArrayPtr(sizearg) => {
                    ctx.rom_items.push(callfiii(
                        imml(ctx.rt.checkaddr),
                        lloc(argnum),
                        imm(0),
                        lloc(sizearg),
                        discard(),
                    ));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(mem.addr), push()));
                }
                GlkParam::Lat1Ptr => {
                    ctx.rom_items
                        .push(callfi(imml(ctx.rt.checkstr), lloc(argnum), discard()));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(mem.addr), push()));
                }
                GlkParam::ScalarPtr(n) => {
                    ctx.rom_items.push(callfiii(
                        imml(ctx.rt.checkaddr),
                        lloc(argnum),
                        imm(0),
                        uimm(4 * n),
                        discard(),
                    ));
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(argnum),
                        uimm(n),
                        discard(),
                    ));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(mem.addr), push()));
                }
                GlkParam::WordArrayPtr(sizearg) => {
                    ctx.rom_items.push(jgt(
                        lloc(sizearg),
                        uimm(0x3fffffff),
                        ctx.rt.trap_out_of_bounds_memory_access,
                    ));
                    ctx.rom_items.push(shiftl(lloc(sizearg), imm(2), push()));
                    ctx.rom_items.push(callfiii(
                        imml(ctx.rt.checkaddr),
                        lloc(argnum),
                        imm(0),
                        pop(),
                        discard(),
                    ));
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(argnum),
                        lloc(sizearg),
                        discard(),
                    ));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(mem.addr), push()));
                }
                GlkParam::UnicodePtr => {
                    ctx.rom_items
                        .push(callfi(imml(ctx.rt.checkunistr), lloc(argnum), discard()));
                    ctx.rom_items
                        .push(callfi(imml(ctx.rt.swapunistr), lloc(argnum), discard()));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(mem.addr), push()));
                }
                GlkParam::OwnedByteArrayPtr(sizearg) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.checkglkaddr),
                        lloc(argnum),
                        lloc(sizearg),
                        discard(),
                    ));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(glk_area.addr), push()));
                }
                GlkParam::OwnedWordArrayPtr(sizearg) => {
                    ctx.rom_items.push(jgt(
                        lloc(sizearg),
                        uimm(0x3fffffff),
                        ctx.rt.trap_out_of_bounds_memory_access,
                    ));
                    ctx.rom_items.push(shiftl(lloc(sizearg), imm(2), push()));
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.checkglkaddr),
                        lloc(argnum),
                        pop(),
                        discard(),
                    ));
                    ctx.rom_items
                        .push(add(lloc(argnum), imml(glk_area.addr), push()));
                }
            }
        }
        ctx.rom_items.push(glk(
            uimm(self.selector.into()),
            uimm(nargs),
            if self.has_return { push() } else { discard() },
        ));
        for (num, param) in self.params.iter().copied().rev().enumerate() {
            let num: u32 = num.try_into().unwrap();
            match param {
                GlkParam::ScalarPtr(n) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(num),
                        uimm(n),
                        discard(),
                    ));
                }
                GlkParam::WordArrayPtr(sizearg) => {
                    ctx.rom_items.push(callfii(
                        imml(ctx.rt.swaparray),
                        lloc(num),
                        lloc(sizearg),
                        discard(),
                    ));
                }
                GlkParam::UnicodePtr => {
                    ctx.rom_items
                        .push(callfi(imml(ctx.rt.swapunistr), lloc(num), discard()));
                }
                _ => {}
            }
        }
        if self.has_return {
            ctx.rom_items.push(ret(pop()));
        } else {
            ctx.rom_items.push(ret(imm(0)));
        }
    }
}

pub fn gen_glk(ctx: &mut Context, imported_func: &ImportedFunction, label: Label) {
    let import = ctx.module.imports.get(imported_func.import);
    let name = &import.name;

    if let Some(function) = get_glk_function(name.as_str()) {
        let expected_params = vec![ValType::I32; function.params.len()];
        let expected_results = if function.has_return {
            vec![ValType::I32]
        } else {
            vec![]
        };
        let ty = ctx.module.types.get(imported_func.ty);
        if expected_params == ty.params() && expected_results == ty.results() {
            function.codegen(ctx, label);
        } else {
            ctx.errors
                .push(crate::CompilationError::IncorrectlyTypedImport {
                    import: ctx.module.imports.get(imported_func.import).clone(),
                    expected: (expected_params, expected_results),
                    actual: (ty.params().to_owned(), ty.results().to_owned()),
                });
        }
    } else {
        ctx.errors
            .push(crate::CompilationError::UnrecognizedImport(import.clone()))
    }
}
