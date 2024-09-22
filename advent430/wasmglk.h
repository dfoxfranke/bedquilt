#ifndef WASMGLK_H
#define WASMGLK_H

/*
    Original copyright 1998-2017 by Andrew Plotkin.
    This version for wasm2glulx copyright 2024 Daniel Fox Franke.
    SPDX-License-Identifier: BSD-2-Clause
*/

typedef unsigned int glui32;
typedef int glsi32;

typedef struct glk_window_struct *winid_t;
typedef struct glk_stream_struct *strid_t;
typedef struct glk_fileref_struct *frefid_t;
typedef struct glk_schannel_struct *schanid_t;
typedef unsigned int glkarea_t;

#define gestalt_Version (0)
#define gestalt_CharInput (1)
#define gestalt_LineInput (2)
#define gestalt_CharOutput (3)
#define gestalt_CharOutput_CannotPrint (0)
#define gestalt_CharOutput_ApproxPrint (1)
#define gestalt_CharOutput_ExactPrint (2)
#define gestalt_MouseInput (4)
#define gestalt_Timer (5)
#define gestalt_Graphics (6)
#define gestalt_DrawImage (7)
#define gestalt_Sound (8)
#define gestalt_SoundVolume (9)
#define gestalt_SoundNotify (10)
#define gestalt_Hyperlinks (11)
#define gestalt_HyperlinkInput (12)
#define gestalt_SoundMusic (13)
#define gestalt_GraphicsTransparency (14)
#define gestalt_Unicode (15)
#define gestalt_UnicodeNorm (16)
#define gestalt_LineInputEcho (17)
#define gestalt_LineTerminators (18)
#define gestalt_LineTerminatorKey (19)
#define gestalt_DateTime (20)
#define gestalt_Sound2 (21)
#define gestalt_ResourceStream (22)
#define gestalt_GraphicsCharInput (23)

#define evtype_None (0)
#define evtype_Timer (1)
#define evtype_CharInput (2)
#define evtype_LineInput (3)
#define evtype_MouseInput (4)
#define evtype_Arrange (5)
#define evtype_Redraw (6)
#define evtype_SoundNotify (7)
#define evtype_Hyperlink (8)
#define evtype_VolumeNotify (9)

typedef struct event_struct {
  glui32 type;
  winid_t win;
  glui32 val1, val2;
} event_t;

#define keycode_Unknown (0xffffffff)
#define keycode_Left (0xfffffffe)
#define keycode_Right (0xfffffffd)
#define keycode_Up (0xfffffffc)
#define keycode_Down (0xfffffffb)
#define keycode_Return (0xfffffffa)
#define keycode_Delete (0xfffffff9)
#define keycode_Escape (0xfffffff8)
#define keycode_Tab (0xfffffff7)
#define keycode_PageUp (0xfffffff6)
#define keycode_PageDown (0xfffffff5)
#define keycode_Home (0xfffffff4)
#define keycode_End (0xfffffff3)
#define keycode_Func1 (0xffffffef)
#define keycode_Func2 (0xffffffee)
#define keycode_Func3 (0xffffffed)
#define keycode_Func4 (0xffffffec)
#define keycode_Func5 (0xffffffeb)
#define keycode_Func6 (0xffffffea)
#define keycode_Func7 (0xffffffe9)
#define keycode_Func8 (0xffffffe8)
#define keycode_Func9 (0xffffffe7)
#define keycode_Func10 (0xffffffe6)
#define keycode_Func11 (0xffffffe5)
#define keycode_Func12 (0xffffffe4)
/* The last keycode is always (0x100000000 - keycode_MAXVAL) */
#define keycode_MAXVAL (28)

#define style_Normal (0)
#define style_Emphasized (1)
#define style_Preformatted (2)
#define style_Header (3)
#define style_Subheader (4)
#define style_Alert (5)
#define style_Note (6)
#define style_BlockQuote (7)
#define style_Input (8)
#define style_User1 (9)
#define style_User2 (10)
#define style_NUMSTYLES (11)

typedef struct stream_result_struct {
  glui32 readcount;
  glui32 writecount;
} stream_result_t;

#define wintype_AllTypes (0)
#define wintype_Pair (1)
#define wintype_Blank (2)
#define wintype_TextBuffer (3)
#define wintype_TextGrid (4)
#define wintype_Graphics (5)

#define winmethod_Left (0x00)
#define winmethod_Right (0x01)
#define winmethod_Above (0x02)
#define winmethod_Below (0x03)
#define winmethod_DirMask (0x0f)

#define winmethod_Fixed (0x10)
#define winmethod_Proportional (0x20)
#define winmethod_DivisionMask (0xf0)

#define winmethod_Border (0x000)
#define winmethod_NoBorder (0x100)
#define winmethod_BorderMask (0x100)

#define fileusage_Data (0x00)
#define fileusage_SavedGame (0x01)
#define fileusage_Transcript (0x02)
#define fileusage_InputRecord (0x03)
#define fileusage_TypeMask (0x0f)

#define fileusage_TextMode (0x100)
#define fileusage_BinaryMode (0x000)

#define filemode_Write (0x01)
#define filemode_Read (0x02)
#define filemode_ReadWrite (0x03)
#define filemode_WriteAppend (0x05)

#define seekmode_Start (0)
#define seekmode_Current (1)
#define seekmode_End (2)

#define stylehint_Indentation (0)
#define stylehint_ParaIndentation (1)
#define stylehint_Justification (2)
#define stylehint_Size (3)
#define stylehint_Weight (4)
#define stylehint_Oblique (5)
#define stylehint_Proportional (6)
#define stylehint_TextColor (7)
#define stylehint_BackColor (8)
#define stylehint_ReverseColor (9)
#define stylehint_NUMHINTS (10)

#define stylehint_just_LeftFlush (0)
#define stylehint_just_LeftRight (1)
#define stylehint_just_Centered (2)
#define stylehint_just_RightFlush (3)

extern void glk_exit(void)
    __attribute__((noreturn, import_module("glk"), import_name("exit")));
extern void glk_tick(void)
    __attribute__((import_module("glk"), import_name("tick")));

extern glui32 glk_gestalt(glui32 sel, glui32 val)
    __attribute__((import_module("glk"), import_name("gestalt")));
extern glui32 glk_gestalt_ext(glui32 sel, glui32 val, glui32 *arr,
                              glui32 arrlen)
    __attribute__((import_module("glk"), import_name("gestalt_ext")));

extern glui32 glk_char_to_lower(glui32 ch);
extern glui32 glk_char_to_upper(glui32 ch);

extern winid_t glk_window_get_root(void)
    __attribute__((import_module("glk"), import_name("window_get_root")));
extern winid_t glk_window_open(winid_t split, glui32 method, glui32 size,
                               glui32 wintype, glui32 rock)
    __attribute__((import_module("glk"), import_name("window_open")));
extern void glk_window_close(winid_t win, stream_result_t *result)
    __attribute__((import_module("glk"), import_name("window_close")));
extern void glk_window_get_size(winid_t win, glui32 *widthptr,
                                glui32 *heightptr)
    __attribute__((import_module("glk"), import_name("window_get_size")));
extern void glk_window_set_arrangement(winid_t win, glui32 method, glui32 size,
                                       winid_t keywin)
    __attribute__((import_module("glk"),
                   import_name("window_set_arrangement")));
extern void glk_window_get_arrangement(winid_t win, glui32 *methodptr,
                                       glui32 *sizeptr, winid_t *keywinptr)
    __attribute__((import_module("glk"),
                   import_name("window_get_arrangement")));
extern winid_t glk_window_iterate(winid_t win, glui32 *rockptr)
    __attribute__((import_module("glk"), import_name("window_iterate")));
extern glui32 glk_window_get_rock(winid_t win)
    __attribute__((import_module("glk"), import_name("window_get_rock")));
extern glui32 glk_window_get_type(winid_t win)
    __attribute__((import_module("glk"), import_name("window_get_type")));
extern winid_t glk_window_get_parent(winid_t win)
    __attribute__((import_module("glk"), import_name("window_get_parent")));
extern winid_t glk_window_get_sibling(winid_t win)
    __attribute__((import_module("glk"), import_name("window_get_sibling")));
extern void glk_window_clear(winid_t win)
    __attribute__((import_module("glk"), import_name("window_clear")));
extern void glk_window_move_cursor(winid_t win, glui32 xpos, glui32 ypos)
    __attribute__((import_module("glk"), import_name("window_move_cursor")));

extern strid_t glk_window_get_stream(winid_t win)
    __attribute__((import_module("glk"), import_name("window_get_stream")));
extern void glk_window_set_echo_stream(winid_t win, strid_t str)
    __attribute__((import_module("glk"),
                   import_name("window_set_echo_stream")));
extern strid_t glk_window_get_echo_stream(winid_t win)
    __attribute__((import_module("glk"),
                   import_name("window_get_echo_stream")));
extern void glk_set_window(winid_t win)
    __attribute__((import_module("glk"), import_name("set_window")));

extern strid_t glk_stream_open_file(frefid_t fileref, glui32 fmode, glui32 rock)
    __attribute__((import_module("glk"), import_name("stream_open_file")));
extern strid_t glk_stream_open_memory(glkarea_t buf, glui32 buflen,
                                      glui32 fmode, glui32 rock)
    __attribute__((import_module("glk"), import_name("stream_open_memory")));
extern void glk_stream_close(strid_t str, stream_result_t *result)
    __attribute__((import_module("glk"), import_name("stream_close")));
extern strid_t glk_stream_iterate(strid_t str, glui32 *rockptr)
    __attribute__((import_module("glk"), import_name("stream_iterate")));
extern glui32 glk_stream_get_rock(strid_t str)
    __attribute__((import_module("glk"), import_name("stream_get_rock")));
extern void glk_stream_set_position(strid_t str, glsi32 pos, glui32 seekmode)
    __attribute__((import_module("glk"), import_name("stream_set_position")));
extern glui32 glk_stream_get_position(strid_t str)
    __attribute__((import_module("glk"), import_name("stream_get_position")));
extern void glk_stream_set_current(strid_t str)
    __attribute__((import_module("glk"), import_name("stream_set_current")));
extern strid_t glk_stream_get_current(void)
    __attribute__((import_module("glk"), import_name("stream_get_current")));

extern void glk_put_char(glui32 ch)
    __attribute__((import_module("glk"), import_name("put_char")));
extern void glk_put_char_stream(strid_t str, unsigned char ch)
    __attribute__((import_module("glk"), import_name("put_char_stream")));
extern void glk_put_string(const char *s)
    __attribute__((import_module("glk"), import_name("put_string")));
extern void glk_put_string_stream(strid_t str, const char *s)
    __attribute__((import_module("glk"), import_name("put_string_stream")));
extern void glk_put_buffer(const char *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("put_buffer")));
extern void glk_put_buffer_stream(strid_t str, const char *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("put_buffer_stream")));
extern void glk_set_style(glui32 styl)
    __attribute__((import_module("glk"), import_name("set_style")));
extern void glk_set_style_stream(strid_t str, glui32 styl)
    __attribute__((import_module("glk"), import_name("set_style_stream")));

extern glsi32 glk_get_char_stream(strid_t str)
    __attribute__((import_module("glk"), import_name("get_char_stream")));
extern glui32 glk_get_line_stream(strid_t str, char *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("get_line_stream")));
extern glui32 glk_get_buffer_stream(strid_t str, char *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("get_buffer_stream")));

extern void glk_stylehint_set(glui32 wintype, glui32 styl, glui32 hint,
                              glsi32 val)
    __attribute__((import_module("glk"), import_name("stylehint_set")));
extern void glk_stylehint_clear(glui32 wintype, glui32 styl, glui32 hint)
    __attribute__((import_module("glk"), import_name("stylehint_clear")));
extern glui32 glk_style_distinguish(winid_t win, glui32 styl1, glui32 styl2)
    __attribute__((import_module("glk"), import_name("style_distinguish")));
extern glui32 glk_style_measure(winid_t win, glui32 styl, glui32 hint,
                                glui32 *result)
    __attribute__((import_module("glk"), import_name("style_measure")));

extern frefid_t glk_fileref_create_temp(glui32 usage, glui32 rock)
    __attribute__((import_module("glk"), import_name("fileref_create_temp")));
extern frefid_t glk_fileref_create_by_name(glui32 usage, const char *name,
                                           glui32 rock)
    __attribute__((import_module("glk"),
                   import_name("fileref_create_by_name")));
extern frefid_t glk_fileref_create_by_prompt(glui32 usage, glui32 fmode,
                                             glui32 rock)
    __attribute__((import_module("glk"),
                   import_name("fileref_create_by_prompt")));
extern frefid_t glk_fileref_create_from_fileref(glui32 usage, frefid_t fref,
                                                glui32 rock)
    __attribute__((import_module("glk"),
                   import_name("fileref_create_from_fileref")));
extern void glk_fileref_destroy(frefid_t fref)
    __attribute__((import_module("glk"), import_name("fileref_destroy")));
extern frefid_t glk_fileref_iterate(frefid_t fref, glui32 *rockptr)
    __attribute__((import_module("glk"), import_name("fileref_iterate")));
extern glui32 glk_fileref_get_rock(frefid_t fref)
    __attribute__((import_module("glk"), import_name("fileref_get_rock")));
extern void glk_fileref_delete_file(frefid_t fref)
    __attribute__((import_module("glk"), import_name("fileref_delete_file")));
extern glui32 glk_fileref_does_file_exist(frefid_t fref)
    __attribute__((import_module("glk"),
                   import_name("fileref_does_file_exist")));

extern void glk_select(event_t *event)
    __attribute__((import_module("glk"), import_name("select")));
extern void glk_select_poll(event_t *event)
    __attribute__((import_module("glk"), import_name("select_poll")));

extern void glk_request_timer_events(glui32 millisecs)
    __attribute__((import_module("glk"), import_name("request_timer_events")));

extern void glk_request_line_event(winid_t win, glkarea_t buf, glui32 maxlen,
                                   glui32 initlen)
    __attribute__((import_module("glk"), import_name("request_line_event")));
extern void glk_request_char_event(winid_t win)
    __attribute__((import_module("glk"), import_name("request_char_event")));
extern void glk_request_mouse_event(winid_t win)
    __attribute__((import_module("glk"), import_name("request_mouse_event")));

extern void glk_cancel_line_event(winid_t win, event_t *event)
    __attribute__((import_module("glk"), import_name("cancel_line_event")));
extern void glk_cancel_char_event(winid_t win)
    __attribute__((import_module("glk"), import_name("cancel_char_event")));
extern void glk_cancel_mouse_event(winid_t win)
    __attribute__((import_module("glk"), import_name("cancel_mouse_event")));

extern void glk_set_echo_line_event(winid_t win, glui32 val)
    __attribute__((import_module("glk"), import_name("set_echo_line_event")));

extern void glk_set_terminators_line_event(winid_t win, glui32 *keycodes,
                                           glui32 count)
    __attribute__((import_module("glk"),
                   import_name("set_terminators_line_event")));

extern glui32 glk_buffer_to_lower_case_uni(glui32 *buf, glui32 len,
                                           glui32 numchars)
    __attribute__((import_module("glk"),
                   import_name("buffer_to_lower_case_uni")));
extern glui32 glk_buffer_to_upper_case_uni(glui32 *buf, glui32 len,
                                           glui32 numchars)
    __attribute__((import_module("glk"),
                   import_name("buffer_to_upper_case_uni")));
extern glui32 glk_buffer_to_title_case_uni(glui32 *buf, glui32 len,
                                           glui32 numchars, glui32 lowerrest)
    __attribute__((import_module("glk"),
                   import_name("buffer_to_title_case_uni")));

extern void glk_put_char_uni(glui32 ch)
    __attribute__((import_module("glk"), import_name("put_char_uni")));
extern void glk_put_string_uni(glui32 *s)
    __attribute__((import_module("glk"), import_name("put_string_uni")));
extern void glk_put_buffer_uni(glui32 *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("put_buffer_uni")));
extern void glk_put_char_stream_uni(strid_t str, glui32 ch)
    __attribute__((import_module("glk"), import_name("put_char_stream_uni")));
extern void glk_put_string_stream_uni(strid_t str, glui32 *s)
    __attribute__((import_module("glk"), import_name("put_string_stream_uni")));
extern void glk_put_buffer_stream_uni(strid_t str, glui32 *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("put_buffer_stream_uni")));

extern glsi32 glk_get_char_stream_uni(strid_t str)
    __attribute__((import_module("glk"), import_name("get_char_stream_uni")));
extern glui32 glk_get_buffer_stream_uni(strid_t str, glui32 *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("get_buffer_stream_uni")));
extern glui32 glk_get_line_stream_uni(strid_t str, glui32 *buf, glui32 len)
    __attribute__((import_module("glk"), import_name("get_line_stream_uni")));

extern strid_t glk_stream_open_file_uni(frefid_t fileref, glui32 fmode,
                                        glui32 rock)
    __attribute__((import_module("glk"), import_name("stream_open_file_uni")));
extern strid_t glk_stream_open_memory_uni(glkarea_t buf, glui32 buflen,
                                          glui32 fmode, glui32 rock)
    __attribute__((import_module("glk"),
                   import_name("stream_open_memory_uni")));

extern void glk_request_char_event_uni(winid_t win)
    __attribute__((import_module("glk"),
                   import_name("request_char_event_uni")));
extern void glk_request_line_event_uni(winid_t win, glkarea_t buf,
                                       glui32 maxlen, glui32 initlen)
    __attribute__((import_module("glk"),
                   import_name("request_line_event_uni")));

extern glui32 glk_buffer_canon_decompose_uni(glui32 *buf, glui32 len,
                                             glui32 numchars)
    __attribute__((import_module("glk"),
                   import_name("buffer_canon_decompose_uni")));
extern glui32 glk_buffer_canon_normalize_uni(glui32 *buf, glui32 len,
                                             glui32 numchars)
    __attribute__((import_module("glk"),
                   import_name("buffer_canon_normalize_uni")));

#define imagealign_InlineUp (0x01)
#define imagealign_InlineDown (0x02)
#define imagealign_InlineCenter (0x03)
#define imagealign_MarginLeft (0x04)
#define imagealign_MarginRight (0x05)

extern glui32 glk_image_draw(winid_t win, glui32 image, glsi32 val1,
                             glsi32 val2)
    __attribute__((import_module("glk"), import_name("image_draw")));
extern glui32 glk_image_draw_scaled(winid_t win, glui32 image, glsi32 val1,
                                    glsi32 val2, glui32 width, glui32 height)
    __attribute__((import_module("glk"), import_name("image_draw_scaled")));
extern glui32 glk_image_get_info(glui32 image, glui32 *width, glui32 *height)
    __attribute__((import_module("glk"), import_name("image_get_info")));

extern void glk_window_flow_break(winid_t win)
    __attribute__((import_module("glk"), import_name("window_flow_break")));

extern void glk_window_erase_rect(winid_t win, glsi32 left, glsi32 top,
                                  glui32 width, glui32 height)
    __attribute__((import_module("glk"), import_name("window_erase_rect")));
extern void glk_window_fill_rect(winid_t win, glui32 color, glsi32 left,
                                 glsi32 top, glui32 width, glui32 height)
    __attribute__((import_module("glk"), import_name("window_fill_rect")));
extern void glk_window_set_background_color(winid_t win, glui32 color)
    __attribute__((import_module("glk"),
                   import_name("window_set_background_color")));

extern schanid_t glk_schannel_create(glui32 rock)
    __attribute__((import_module("glk"), import_name("schannel_create")));
extern void glk_schannel_destroy(schanid_t chan)
    __attribute__((import_module("glk"), import_name("schannel_destroy")));
extern schanid_t glk_schannel_iterate(schanid_t chan, glui32 *rockptr)
    __attribute__((import_module("glk"), import_name("schannel_iterate")));
extern glui32 glk_schannel_get_rock(schanid_t chan)
    __attribute__((import_module("glk"), import_name("schannel_get_rock")));

extern glui32 glk_schannel_play(schanid_t chan, glui32 snd)
    __attribute__((import_module("glk"), import_name("schannel_play")));
extern glui32 glk_schannel_play_ext(schanid_t chan, glui32 snd, glui32 repeats,
                                    glui32 notify)
    __attribute__((import_module("glk"), import_name("schannel_play_ext")));
extern void glk_schannel_stop(schanid_t chan)
    __attribute__((import_module("glk"), import_name("schannel_stop")));
extern void glk_schannel_set_volume(schanid_t chan, glui32 vol)
    __attribute__((import_module("glk"), import_name("schannel_set_volume")));

extern void glk_sound_load_hint(glui32 snd, glui32 flag)
    __attribute__((import_module("glk"), import_name("sound_load_hint")));

extern schanid_t glk_schannel_create_ext(glui32 rock, glui32 volume)
    __attribute__((import_module("glk"), import_name("schannel_create_ext")));
extern glui32 glk_schannel_play_multi(schanid_t *chanarray, glui32 chancount,
                                      glui32 *sndarray, glui32 soundcount,
                                      glui32 notify)
    __attribute__((import_module("glk"), import_name("schannel_play_multi")));
extern void glk_schannel_pause(schanid_t chan)
    __attribute__((import_module("glk"), import_name("schannel_pause")));
extern void glk_schannel_unpause(schanid_t chan)
    __attribute__((import_module("glk"), import_name("schannel_unpause")));
extern void glk_schannel_set_volume_ext(schanid_t chan, glui32 vol,
                                        glui32 duration, glui32 notify)
    __attribute__((import_module("glk"),
                   import_name("schannel_set_volume_ext")));

extern void glk_set_hyperlink(glui32 linkval)
    __attribute__((import_module("glk"), import_name("set_hyperlink")));
extern void glk_set_hyperlink_stream(strid_t str, glui32 linkval)
    __attribute__((import_module("glk"), import_name("set_hyperlink_stream")));
extern void glk_request_hyperlink_event(winid_t win)
    __attribute__((import_module("glk"),
                   import_name("request_hyperlink_event")));
extern void glk_cancel_hyperlink_event(winid_t win)
    __attribute__((import_module("glk"),
                   import_name("cancel_hyperlink_event")));

typedef struct glktimeval_struct {
  glsi32 high_sec;
  glui32 low_sec;
  glsi32 microsec;
} glktimeval_t;

typedef struct glkdate_struct {
  glsi32 year;     /* full (four-digit) year */
  glsi32 month;    /* 1-12, 1 is January */
  glsi32 day;      /* 1-31 */
  glsi32 weekday;  /* 0-6, 0 is Sunday */
  glsi32 hour;     /* 0-23 */
  glsi32 minute;   /* 0-59 */
  glsi32 second;   /* 0-59, maybe 60 during a leap second */
  glsi32 microsec; /* 0-999999 */
} glkdate_t;

extern void glk_current_time(glktimeval_t *time)
    __attribute__((import_module("glk"), import_name("current_time")));
extern glsi32 glk_current_simple_time(glui32 factor)
    __attribute__((import_module("glk"), import_name("current_simple_time")));
extern void glk_time_to_date_utc(glktimeval_t *time, glkdate_t *date)
    __attribute__((import_module("glk"), import_name("time_to_date_utc")));
extern void glk_time_to_date_local(glktimeval_t *time, glkdate_t *date)
    __attribute__((import_module("glk"), import_name("time_to_date_local")));
extern void glk_simple_time_to_date_utc(glsi32 time, glui32 factor,
                                        glkdate_t *date)
    __attribute__((import_module("glk"),
                   import_name("simple_time_to_date_utc")));
extern void glk_simple_time_to_date_local(glsi32 time, glui32 factor,
                                          glkdate_t *date)
    __attribute__((import_module("glk"),
                   import_name("simple_time_to_date_local")));
extern void glk_date_to_time_utc(glkdate_t *date, glktimeval_t *time)
    __attribute__((import_module("glk"), import_name("date_to_time_utc")));
extern void glk_date_to_time_local(glkdate_t *date, glktimeval_t *time)
    __attribute__((import_module("glk"), import_name("date_to_time_local")));
extern glsi32 glk_date_to_simple_time_utc(glkdate_t *date, glui32 factor)
    __attribute__((import_module("glk"),
                   import_name("date_to_simple_time_utc")));
extern glsi32 glk_date_to_simple_time_local(glkdate_t *date, glui32 factor)
    __attribute__((import_module("glk"),
                   import_name("date_to_simple_time_local")));

extern strid_t glk_stream_open_resource(glui32 filenum, glui32 rock)
    __attribute__((import_module("glk"), import_name("stream_open_resource")));
extern strid_t glk_stream_open_resource_uni(glui32 filenum, glui32 rock)
    __attribute__((import_module("glk"),
                   import_name("stream_open_resource_uni")));

extern void glulx_glkarea_get_bytes(void *, glkarea_t, size_t)
    __attribute__((import_module("glulx"), import_name("glkarea_get_bytes")));

extern glsi32 glulx_random(glsi32)
    __attribute__((import_module("glulx"), import_name("random")));
extern void glulx_setrandom(glsi32)
    __attribute__((import_module("glulx"), import_name("setrandom")));

#endif /* WASMGLK_H */
