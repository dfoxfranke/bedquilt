//! Window management and user interaction.
//! 
//! Bedquilt deals in four types of windows:
//! 
//! 1. **Text buffer** windows display running text, possibly with support for
//!    styling, hyperlinks, and images. The usual "main window" of a text adventure
//!    is a text buffer window.
//! 
//! 2. **Text grid** windows display monospaced text and support repositioning
//!    the cursor. These are typically used for displaying status bars.
//! 
//! 3. **Graphics** windows provide a canvas for displaying images and drawing
//!    rectangles.
//! 
//! 4. **Blank** windows are… blank.
//! 
//! In Glk, there is a fifth window type, a **pair** window. A pair window
//! divides up the screen space allocated to it to display exactly two child
//! windows, which might themselves be pair windows, by splitting the space
//! horizontally or vertically. The set of all windows thereby forms a binary
//! tree, with all interior nodes being pair windows and all leaf nodes being
//! one of the four type above.
//! 
//! When you close one of a pair window's two children, this automatically
//! destroys the pair window as well, with the surviving child's grandparent
//! becoming its parent. This behavior doesn't play well with Rust's borrow
//! rules. Bedquilt deals with this by never directly providing a handle to a
//! pair window. In Bedquilt, when you want to operate on a pair window, you
//! invoke a method on one of the pair window's descendants and supply a
//! non-zero `nth_ancestor` argument to indicate which of the leaf window's
//! ancestors you want to operate on.
//! 
//! To get started with this module, use the [`Window::create_as_root`] method
//! to create your first window, and then use that window's `split_`* methods to
//! create additional ones. You can print text to a [`TextBufferWindow`] or
//! [`TextGridWindow`] by using its [`core::fmt::Write`] implementation. `Write`
//! is implemented not only on the window structures but also on shared
//! references to them, so you can write to them given only a shared reference
//! when `Write`'s methods would otherwise require a `&mut Self`. 
//! 
//! Once have a window, you can use the [`CharInput`], [`LineInput`],
//! [`MouseInput`], and [`HyperlinkInput`] traits to request user input from
//! them. All of these traits return futures which become ready when the input
//! is available. Bedquilt *does* allow multiple futures of the same type to be
//! pending at the same time for the same window, in which case they will become
//! ready one at a time, each with a different instance of user input. However,
//! it is unspecified what order they will become ready in; the first one
//! created is not necessarily the first to become ready. In some future
//! release, this specification may be refined to provide more useful
//! guarantees.
//! 
//! Currently, only one window at a time is allowed to have line input pending;
//! further requests will return
//! [`GlkAreaAllocError`](crate::error::Error::GlkAreaAllocError). This limit
//! will increase in a future release.

use self::futures::{
    ArcCharFuture, ArcHyperlinkFuture, ArcLineFuture, ArcMouseFuture, CharFuture, HyperlinkFuture,
    LineFuture, MouseFuture, RedrawFuture,
};
use crate::fs::FileRef;
pub use crate::reactor::{CharEvent, HyperlinkEvent, LineEvent, LineTermination, MouseEvent};
/// Returns the dimensions of an image resource.
pub use crate::sys::glk::image_size;
use crate::{
    error::Result,
    reactor::GLOBAL_REACTOR,
    sys::glk::{set_style_hint, WinId, WindowImpl},
};
use alloc::{string::String, sync::Arc};
use core::marker::PhantomData;
/// How to align an image printed to a text buffer window.
pub use wasm2glulx_ffi::glk::ImageAlign;
use wasm2glulx_ffi::glk::{Keycode, Style as GlkStyle, WinMethod, WinType};

/// Common capabilities shared by all windows.
pub trait Window: Sized {
    /// Creates a new window as the game's root window.
    /// 
    /// This call will fail if any windows already exist.
    fn create_as_root() -> Result<Self>;

    /// Creates a new blank window by splitting `self` or one of its ancestors.
    /// 
    /// An `nth_ancestor` argument of 0 will split the window itself. 1 will
    /// split the window's parent, 2 will split its grandparent, and so forth.
    /// An out-of-range `nth_ancestor` argument will cause an error to be
    /// returned.
    fn split_blank(
        &self,
        position: WindowPosition,
        size_pct: u32,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<BlankWindow>;

    /// Creates a new text buffer window by splitting `self` or one of its
    /// ancestors.
    ///
    /// An `nth_ancestor` argument of 0 will split the window itself. 1 will
    /// split the window's parent, 2 will split its grandparent, and so forth.
    /// An out-of-range `nth_ancestor` argument will cause an error to be
    /// returned.
    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextBufferWindow>;

    /// Creates a new text grid window by splitting `self` or one of its
    /// ancestors.
    ///
    /// An `nth_ancestor` argument of 0 will split the window itself. 1 will
    /// split the window's parent, 2 will split its grandparent, and so forth.
    /// An out-of-range `nth_ancestor` argument will cause an error to be
    /// returned.
    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextGridWindow>;

    /// Creates a new graphics window by splitting `self` or one of its
    /// ancestors.
    ///
    /// An `nth_ancestor` argument of 0 will split the window itself. 1 will
    /// split the window's parent, 2 will split its grandparent, and so forth.
    /// An out-of-range `nth_ancestor` argument will cause an error to be
    /// returned.
    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<GraphicsWindow>;

    /// Repositions the split of one of the window's ancestors.
    /// 
    /// An `nth_ancestor` argument of 1 will operate on the widow's parent, 2
    /// will operate on its grandparent, and so-forth. If the `nth_ancestor`
    /// argument is zero or out of range, the call will have no effect.
    fn resplit_proportional(&self, percent: u32, nth_ancestor: u32);

    /// Sets the styling of text printed to this window.
    /// 
    /// Although this method is supported for all window types, it is meaningful
    /// only for text windows.
    fn set_style(&self, style: Style);

    /// Returns information describing the visual apperance of the given style.
    /// 
    /// Not all Glk implementations support this; those which do not will return
    /// something whose fields are all `None`.
    fn measure_style(&self, style: Style) -> StyleMeasurement;

    /// Interrogates whether the two provided styles are visually
    /// distinguishable from each other.
    /// 
    /// Not all Glk implementations have access to this information, and may
    /// return false negatives or just return `false` unconditionally. However,
    /// a true return value is reliably an assurance that the styles are in fact
    /// distinguishable.
    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool;

    /// Sets this window's output stream as current.
    ///
    /// Nothing in the Bedquilt API ever cares what stream is current; you
    /// always have to be explicit about what you're writing to. However, some
    /// Glk internals might care. For example, the current stream might be used
    /// for error messages or file prompts.
    ///
    /// Note that creating a window via `create_as_root` also automatically sets
    /// it as current.
    fn set_as_current(&self);

    /// Returns the current size of the window.
    ///
    /// The units are dependent on the type of window. For text windows, the
    /// return value is in characters; for graphics windows, it is in pixels.
    /// Blank windows always return (0,0).
    fn get_size(&self) -> (u32, u32);

    /// Erases the contents of the window.
    fn clear(&self);
}

/// Windows which support character input.
pub trait CharInput: Window {
    /// Requests character input from the user.
    /// 
    /// Returns a futures which will output a [`CharEvent`].
    fn request_char(&self) -> Result<CharFuture<'_>>;

    /// Requests character input from the user, in a reference-counted window.
    /// 
    /// The returned future holds a *weak* reference to the window. If the
    /// window is closed while the future is still pending, it will become ready
    /// and output [`CharEvent::Cancelled`].
    fn arc_request_char(self: &Arc<Self>) -> Result<ArcCharFuture<Self>>;

    /// Cancels a request for character or line input.
    fn cancel_keyboard_request(&self);
}

/// Windows which support line input.
pub trait LineInput: CharInput {
    /// Requests line input from the user.
    /// 
    /// The input line will be pre-populated with the contents of `initial`. 
    /// 
    /// Returns a future which will output a [`LineEvent`].
    fn request_line<'a>(&'a self, initial: &str) -> Result<LineFuture<'a>>;

    /// Requests line input from the user, in a reference-counted window.
    /// 
    /// The returned future holds a *weak* reference to the window. If the
    /// window is closed while the future is still pending, it will become ready
    /// and output a [`LineEvent`] whose `termination` field is
    /// [`LineTermination::Cancelled`].
    fn arc_request_line(self: &Arc<Self>, initial: &str) -> Result<ArcLineFuture<Self>>;

    /// Sets whether completed line input will be echoed to the window.
    /// 
    /// The default is `true`. If `echo` is false, line input will appear in the
    /// window only while the user is entering it, and will be erased after it
    /// is completed.
    fn set_echo(&self, echo: bool) -> Result<()>;

    /// Sets a list of keys which, when pressed by the user, will complete a
    /// line input event.
    /// 
    /// "Enter" will always complete line input no matter what is set here.
    fn set_terminators(&self, terminators: &[Keycode]) -> Result<()>;
}

/// Windows which support mouse input.
pub trait MouseInput: Window {
    /// Requests mouse input.
    /// 
    /// Returns a future which will output a [`MouseEvent`].
    fn request_mouse(&self) -> Result<MouseFuture<'_>>;
    /// Requests mouse input, in a reference-counted window.
    /// 
    /// The returned future holds a *weak* reference to the window. If the
    /// window is closed while the future is still pending, it will become ready
    /// and output [`MouseEvent::Cancelled`].
    fn arc_request_mouse(self: &Arc<Self>) -> Result<ArcMouseFuture<Self>>;

    /// Cancels a request for mouse input.
    fn cancel_mouse_request(&self);
}

/// Windows which support hyperlinks.
pub trait HyperlinkInput: Window {
    /// Sets the window's hyperlink value.
    /// 
    /// Setting a non-zero `linkval` will cause further content written to the
    /// window to be associated with a hyperlink; a zero `linkval` ends the
    /// hyperlink. Whatever `linkval` you supply will be returned as a
    /// [`HyperlinkEvent`] when the user selects the link.
    fn set_hyperlink(&self, linkval: u32) -> Result<()>;

    /// Requests hyperlink input.
    /// 
    /// Returns a future which will output a [`HyperlinkEvent`].
    fn request_hyperlink(&self) -> Result<HyperlinkFuture<'_>>;

    /// Requests hyperlink input, in a reference-counted window.
    /// 
    /// The returned future holds a *weak* reference to the window. If the
    /// window is closed while the future is still pending, it will become ready
    /// and output [`HyperlinkEvent::Cancelled`].
    fn arc_request_hyperlink(self: &Arc<Self>) -> Result<ArcHyperlinkFuture<Self>>;

    /// Cancels a request for hyperlink input.
    fn cancel_hyperlink_request(&self);
}

/// A blank window.
/// 
/// Blank windows do nothing but fill out screen space. They cannot be written
/// to and do not support any input events.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BlankWindow(pub(crate) WindowImpl);
/// A text buffer window.
/// 
/// The "main" window of a text adventure is typically a text buffer window. All
/// Glk implementations support them, and support at least character and line
/// input. Depending on the Glk implementation, text buffer windows may also be
/// capable of hyperlink input, rich text styling, and/or image rendering.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextBufferWindow(pub(crate) WindowImpl);

/// A text grid window.
/// 
/// Text grid windows always use a fixed-width font, and support repositioning
/// the cursor. Glk implementations which support text grid windows at all will
/// always support at least character and line input events, and may support
/// mouse and hyperlink input as well. Text grid windows are most often used for
/// rendering status bars.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextGridWindow(pub(crate) WindowImpl);

/// A graphics window.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GraphicsWindow(pub(crate) WindowImpl);

/// How to position a new window.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WindowPosition {
    /// Place the new window above the one being split.
    Above,
    /// Place the new window below the one being split.
    Below,
    /// Place the new window to the left of the one being split.
    Left,
    /// Place the new window to the right of the one being split.
    Right,
}

/// Window split keyed on a text buffer or text grid window.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextWindowSize {
    /// Splits a pair window proportionally, ignoring the key window.
    Proportional { 
        /// Split percentage.
        percent: u32
    },
    /// Splits a pair window such that one part is sized to fit the given number
    /// of characters in the key window's font size.
    Fixed { 
        /// Size in characters.
        chars: u32
    },
}

/// Window split keyed on a graphics window.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GraphicsWindowSize {
    /// Splits a pair window proportionally, ignoring the key window.
    Proportional { 
        /// Split percentage.
        percent: u32
    },
    /// Splits a pair window such that one part is sized to fit the given number
    /// of pixels.
    Fixed { 
        /// Size in pixels.
        pixels: u32
    },
}

/// Text styling.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Style {
    /// The style of normal or body text.
    Normal,
    /// Text which is emphasized.
    Emphasized,
    /// Text which has a particular arrangement of characters.
    Preformatted,
    /// Text which introduces a large section. This is suitable for the title
    /// of an entire game, or a major division such as a chapter.
    Header,
    ///  Text which introduces a smaller section within a large section.
    Subheader,
    /// Text which warns of a dangerous condition, or one which the player
    /// should pay attention to.
    Alert,
    /// Text which notifies of an interesting condition.
    Note,
    /// Text which forms a quotation or otherwise abstracted text.
    BlockQuote,
    /// Text which the player has entered.
    Input,
    /// User-defined style with no prior meaning.
    User1,
    /// User-defined style with no prior meaning.
    User2,
}

/// The requested appearance of a text style.
/// 
/// This enum can be passed as an argument to [`Style::set_hint`]. Setting the
/// argument of any enum variant to `None` will request unsetting of that hint.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StyleHint {
    /// How much to indent lines of text.
    /// 
    /// May be a negative number, to indent outward rather than inward. The
    /// units are implementation-defined but can be assumed to be characters if
    /// on a terminal or ems if in a browser.
    Indentation(Option<i32>),
    /// How much to indent the first line of a paragraph.
    /// 
    /// This is cummulative with `Indentation`.
    ParaIndentation(Option<i32>),
    /// Controls how text should be justified.
    Justification(Option<Justification>),
    /// Controls font size.
    /// 
    /// The argument is in given in points, relative to the default size. For
    /// example, if the system default is 12pt, an argument of -2 will result in
    /// a 10pt font.
    Size(Option<i32>),
    /// Controls text weight.
    Weight(Option<Weight>),
    /// Set to true for an oblique font, or false for an upright one.
    Oblique(Option<bool>),
    /// Set to true for a proportional font, or false for a fixed-width one.
    Proportional(Option<bool>),
    /// Sets the text color.
    TextColor(Option<Color>),
    /// Sets the background color.
    BackColor(Option<Color>),
    /// Set to true to swap text and background colors.
    ReverseColor(Option<bool>),
}

/// The class of windows to which a style hint should be applied.
/// 
/// This enum is passed as an argument to [`Style::set_hint`].
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StyleClass {
    /// Apply to all type of windows.
    All,
    /// Apply only to text buffer windows.
    TextBuffer,
    /// Apply only to text grid windows.
    TextGrid,
    /// Apply only to graphics windows.
    /// 
    /// Since styles only affect text and text cannot be printed to graphics
    /// windows, this variant is rather meaningless, but could potentially
    /// become meaningful with future extenions to Glk.
    Graphics,
}

/// Characterizes how a style appears in a particular window.
/// 
/// Not all Glk implementations are capable of providing this information, so
/// some or all fields may be `None`, indicating "unknown".
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct StyleMeasurement {
    /// How much each line of text is indented.
    /// 
    /// A negative number indicates outward indentation. The units are
    /// implementation-defined but can be assumed to be characters if on a
    /// terminal or ems if in a browser.
    pub indentation: Option<i32>,
    /// How much the first line of a paragraph is indented.
    /// 
    /// This is cummulative with `indentation`.
    pub para_indentation: Option<i32>,
    /// The way in which text is justified.
    pub justification: Option<Justification>,
    /// The font size, in points, relative to the default size. For example, if
    /// the system default is 12pt, then -2 means a 10pt font.
    pub size: Option<i32>,
    /// The text weight.
    pub weight: Option<Weight>,
    /// True for an oblique font, or false for an upright one.
    pub oblique: Option<bool>,
    /// True for a proportional font, or false for a fixed-width one.
    pub proportional: Option<bool>,
    /// The text color.
    pub text_color: Option<Color>,
    /// The background color.
    pub back_color: Option<Color>,
    /// Whether text and background color are reversed.
    pub reverse_color: Option<bool>,
}

/// Text justification.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Justification {
    /// Flush with the left margin, ragged on the right.
    LeftFlush,
    /// Full justification, *i.e.*, flush with both margins. 
    LeftRight,
    /// Centered; flush with neither margin.
    Centered,
    /// Flush with the right margin, ragged on the left.
    RightFlush,
}

/// Text weight.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Weight {
    /// Light weight.
    Light,
    /// Normal weight.
    Normal,
    /// Bold weight.
    Bold,
}

/// An sRGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red
    pub r: u8,
    /// Green
    pub g: u8,
    /// Blue
    pub b: u8,
}

pub(crate) enum Event {
    Timer,
    CharInput {
        win: WinId,
        input: char,
    },
    CharInputSpecial {
        win: WinId,
        input: Keycode,
    },
    LineInput {
        win: WinId,
        input: String,
        terminator: Option<Keycode>,
    },
    MouseInput {
        win: WinId,
        x: u32,
        y: u32,
    },
    Arrange {
        #[allow(dead_code)]
        win: WinId,
    },
    Redraw {
        #[allow(dead_code)]
        win: WinId,
    },
    SoundNotify {
        #[allow(dead_code)]
        resource: u32,
        notify: u32,
    },
    Hyperlink {
        win: WinId,
        linkid: u32,
    },
    VolumeNotify {
        notify: u32,
    },
}

impl Window for BlankWindow {
    fn create_as_root() -> Result<Self> {
        let result = WindowImpl::create_as_root(WinType::Blank)?;
        GLOBAL_REACTOR.redraw();
        Ok(Self(result))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size_pct: u32,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<BlankWindow> {
        let result = self.0.split_blank(position, size_pct, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(BlankWindow(result))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextBufferWindow> {
        let result = self
            .0
            .split_text_buffer(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextBufferWindow(result))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextGridWindow> {
        let result = self.0.split_text_grid(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextGridWindow(result))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<GraphicsWindow> {
        let result = self.0.split_graphics(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(GraphicsWindow(result))
    }

    fn set_style(&self, style: Style) {
        self.0.set_style(style);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn resplit_proportional(&self, percent: u32, nth_ancestor: u32) {
        self.0.resplit_proportional(percent, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }

    fn set_as_current(&self) {
        self.0.set_as_current();
    }

    fn get_size(&self) -> (u32, u32) {
        self.0.get_size()
    }

    fn clear(&self) {
        self.0.clear();
    }
}

impl Drop for BlankWindow {
    fn drop(&mut self) {
        GLOBAL_REACTOR.close_window(self.0.id());
    }
}

impl Window for TextBufferWindow {
    fn create_as_root() -> Result<Self> {
        let result = WindowImpl::create_as_root(WinType::TextBuffer)?;
        GLOBAL_REACTOR.redraw();
        Ok(Self(result))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size_pct: u32,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<BlankWindow> {
        let result = self.0.split_blank(position, size_pct, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(BlankWindow(result))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextBufferWindow> {
        let result = self
            .0
            .split_text_buffer(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextBufferWindow(result))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextGridWindow> {
        let result = self.0.split_text_grid(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextGridWindow(result))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<GraphicsWindow> {
        let result = self.0.split_graphics(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(GraphicsWindow(result))
    }

    fn set_style(&self, style: Style) {
        self.0.set_style(style);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn resplit_proportional(&self, percent: u32, nth_ancestor: u32) {
        self.0.resplit_proportional(percent, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }

    fn set_as_current(&self) {
        self.0.set_as_current();
    }

    fn get_size(&self) -> (u32, u32) {
        self.0.get_size()
    }

    fn clear(&self) {
        self.0.clear();
    }
}

impl CharInput for TextBufferWindow {
    fn request_char(&self) -> Result<CharFuture<'_>> {
        Ok(CharFuture {
            inner: GLOBAL_REACTOR.request_char(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_char(self: &Arc<Self>) -> Result<ArcCharFuture<Self>> {
        Ok(ArcCharFuture {
            inner: GLOBAL_REACTOR.request_char(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_keyboard_request(&self) {
        GLOBAL_REACTOR.cancel_keyboard_request(self.0.id());
    }
}

impl LineInput for TextBufferWindow {
    fn request_line<'a>(&'a self, initial: &str) -> Result<LineFuture<'a>> {
        Ok(LineFuture {
            inner: GLOBAL_REACTOR.request_line(self.0.id(), initial)?,
            phantom: PhantomData,
        })
    }

    fn arc_request_line(self: &Arc<Self>, initial: &str) -> Result<ArcLineFuture<Self>> {
        Ok(ArcLineFuture {
            inner: GLOBAL_REACTOR.request_line(self.0.id(), initial)?,
            weak: Arc::downgrade(self),
        })
    }

    fn set_echo(&self, echo: bool) -> Result<()> {
        self.0.set_echo(echo)
    }

    fn set_terminators(&self, terminators: &[Keycode]) -> Result<()> {
        self.0.set_terminators(terminators)
    }
}

impl HyperlinkInput for TextBufferWindow {
    fn set_hyperlink(&self, linkval: u32) -> Result<()> {
        self.0.set_hyperlink(linkval)
    }

    fn request_hyperlink(&self) -> Result<HyperlinkFuture<'_>> {
        Ok(HyperlinkFuture {
            inner: GLOBAL_REACTOR.request_hyperlink(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_hyperlink(self: &Arc<Self>) -> Result<ArcHyperlinkFuture<Self>> {
        Ok(ArcHyperlinkFuture {
            inner: GLOBAL_REACTOR.request_hyperlink(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_hyperlink_request(&self) {
        GLOBAL_REACTOR.cancel_hyperlink_request(self.0.id());
    }
}

impl Drop for TextBufferWindow {
    fn drop(&mut self) {
        GLOBAL_REACTOR.close_window(self.0.id());
    }
}

impl core::fmt::Write for TextBufferWindow {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        GLOBAL_REACTOR.write_str(self.0.id(), s)
    }
}

impl core::fmt::Write for &TextBufferWindow {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        GLOBAL_REACTOR.write_str(self.0.id(), s)
    }
}

impl TextBufferWindow {
    /// Repositions the split of one of the window's ancestors, using `self` as a
    /// key window.
    /// 
    /// An `nth_ancestor` argument of 1 will operate on the widow's parent, 2 will
    /// operate on its grandparent, and so-forth. If the `nth_ancestor` argument
    /// is zero or out of range, the call will have no effect.
    /// 
    /// Fixed-size splits will be computed according to `self`'s font size.
    pub fn resplit(&self, size: TextWindowSize, nth_ancestor: u32) {
        self.0.resplit_text(size, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    /// Begins recording window activity to a transcript file.
    /// 
    /// The file path is given by `fref`. If `append` is true, activity will be
    /// appended to the end of the file's current contents. If false, the file
    /// will be overwritten. Only window activity which occurs after this method
    /// is called will be reflected in the transcript; past activity will not be
    /// recorded.
    /// 
    /// Only one transcsript file per window can be active at a time. Calling
    /// this method multiple times will cancel all but the last invocation. The
    /// result of attempting to record to the same transcript file from multiple
    /// windows is Glk implementation-dependent, but will be nothing good. If
    /// any of the restrictions described in this paragraph are inconvenient for
    /// you, you are best off not using this method, and instead implementing
    /// your own wrapper object around a window and an
    /// [`RwFile`](crate::fs::RwFile).
    pub fn transcript(&self, fref: &FileRef, append: bool) -> Result<()> {
        self.0.transcript(&fref.0, append)
    }

    /// Stops recording window activity.
    /// 
    /// This cancels any previous call to `transcript`.
    pub fn transcript_off(&self) {
        self.0.transcript_off();
    }

    /// Draws an image into the window.
    /// 
    /// `resource_index` is the Blorb chunk index of a `Pict` resource. The
    /// image will be aligned in the window according to `alignment`. The image
    /// will be drawn at its natural scale.
    pub fn draw_image(&self, resource_index: u32, alignment: ImageAlign) -> Result<()> {
        self.0.draw_image(resource_index, alignment.into(), 0)
    }

    /// Draws a scaled image into the window.
    /// 
    /// `resource_index` is the Blorb chunk index of a `Pict` resource. The
    /// image will be aligned in the window according to `alignment` and scaled
    /// to `width` and `height`.
    pub fn draw_image_scaled(
        &self,
        resource_index: u32,
        alignment: ImageAlign,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.0
            .draw_image_scaled(resource_index, alignment.into(), 0, width, height)
    }

    /// Advances output past the end of the last margin-align image.
    /// 
    /// When an image is written to the window using `draw_image` or
    /// `draw_image_scaled`, and `alignment` is `MarginLeft` or `MarginRight`,
    /// text printed to the window thereafter will be wrapped around the image.
    /// Calling this method will advance to a new paragraph which begins below
    /// the bottom edge of the image.
    pub fn flow_break(&self) -> Result<()> {
        self.0.flow_break()
    }
}

impl Window for TextGridWindow {
    fn create_as_root() -> Result<Self> {
        let result = WindowImpl::create_as_root(WinType::TextGrid)?;
        GLOBAL_REACTOR.redraw();
        Ok(Self(result))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size_pct: u32,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<BlankWindow> {
        let result = self.0.split_blank(position, size_pct, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(BlankWindow(result))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextBufferWindow> {
        let result = self
            .0
            .split_text_buffer(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextBufferWindow(result))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextGridWindow> {
        let result = self.0.split_text_grid(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextGridWindow(result))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<GraphicsWindow> {
        let result = self.0.split_graphics(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(GraphicsWindow(result))
    }

    fn set_style(&self, style: Style) {
        self.0.set_style(style);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn resplit_proportional(&self, percent: u32, nth_ancestor: u32) {
        self.0.resplit_proportional(percent, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }

    fn set_as_current(&self) {
        self.0.set_as_current();
    }

    fn get_size(&self) -> (u32, u32) {
        self.0.get_size()
    }

    fn clear(&self) {
        self.0.clear();
    }
}

impl CharInput for TextGridWindow {
    fn request_char(&self) -> Result<CharFuture<'_>> {
        Ok(CharFuture {
            inner: GLOBAL_REACTOR.request_char(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_char(self: &Arc<Self>) -> Result<ArcCharFuture<Self>> {
        Ok(ArcCharFuture {
            inner: GLOBAL_REACTOR.request_char(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_keyboard_request(&self) {
        GLOBAL_REACTOR.cancel_keyboard_request(self.0.id());
    }
}

impl LineInput for TextGridWindow {
    fn request_line<'a>(&'a self, initial: &str) -> Result<LineFuture<'a>> {
        Ok(LineFuture {
            inner: GLOBAL_REACTOR.request_line(self.0.id(), initial)?,
            phantom: PhantomData,
        })
    }

    fn arc_request_line(self: &Arc<Self>, initial: &str) -> Result<ArcLineFuture<Self>> {
        Ok(ArcLineFuture {
            inner: GLOBAL_REACTOR.request_line(self.0.id(), initial)?,
            weak: Arc::downgrade(self),
        })
    }

    fn set_echo(&self, echo: bool) -> Result<()> {
        self.0.set_echo(echo)
    }

    fn set_terminators(&self, terminators: &[Keycode]) -> Result<()> {
        self.0.set_terminators(terminators)
    }
}

impl MouseInput for TextGridWindow {
    fn request_mouse(&self) -> Result<MouseFuture<'_>> {
        Ok(MouseFuture {
            inner: GLOBAL_REACTOR.request_mouse(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_mouse(self: &Arc<Self>) -> Result<ArcMouseFuture<Self>> {
        Ok(ArcMouseFuture {
            inner: GLOBAL_REACTOR.request_mouse(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_mouse_request(&self) {
        GLOBAL_REACTOR.cancel_mouse_request(self.0.id());
    }
}

impl HyperlinkInput for TextGridWindow {
    fn set_hyperlink(&self, linkval: u32) -> Result<()> {
        self.0.set_hyperlink(linkval)
    }

    fn request_hyperlink(&self) -> Result<HyperlinkFuture<'_>> {
        Ok(HyperlinkFuture {
            inner: GLOBAL_REACTOR.request_hyperlink(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_hyperlink(self: &Arc<Self>) -> Result<ArcHyperlinkFuture<Self>> {
        Ok(ArcHyperlinkFuture {
            inner: GLOBAL_REACTOR.request_hyperlink(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_hyperlink_request(&self) {
        GLOBAL_REACTOR.cancel_hyperlink_request(self.0.id());
    }
}

impl Drop for TextGridWindow {
    fn drop(&mut self) {
        GLOBAL_REACTOR.close_window(self.0.id());
    }
}

impl core::fmt::Write for TextGridWindow {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        GLOBAL_REACTOR.write_str(self.0.id(), s)
    }
}

impl core::fmt::Write for &TextGridWindow {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        GLOBAL_REACTOR.write_str(self.0.id(), s)
    }
}

impl TextGridWindow {
    /// Repositions the split of one of the window's ancestors, using `self` as a
    /// key window.
    /// 
    /// An `nth_ancestor` argument of 1 will operate on the widow's parent, 2 will
    /// operate on its grandparent, and so-forth. If the `nth_ancestor` argument
    /// is zero or out of range, the call will have no effect.
    /// 
    /// Fixed-size splits will be computed according to `self`'s font size.
    pub fn resplit(&self, size: TextWindowSize, nth_ancestor: u32) {
        self.0.resplit_text(size, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    /// Sets the cursor position at which the text next printed to the window
    /// will appear.
    pub fn move_cursor(&self, x: u32, y: u32) {
        self.0.move_cursor(x, y);
    }
}

impl Window for GraphicsWindow {
    fn create_as_root() -> Result<Self> {
        let result = WindowImpl::create_as_root(WinType::Graphics)?;
        GLOBAL_REACTOR.redraw();
        Ok(Self(result))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size_pct: u32,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<BlankWindow> {
        let result = self.0.split_blank(position, size_pct, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(BlankWindow(result))
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextBufferWindow> {
        let result = self
            .0
            .split_text_buffer(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextBufferWindow(result))
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<TextGridWindow> {
        let result = self.0.split_text_grid(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(TextGridWindow(result))
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_ancestor: u32,
    ) -> Result<GraphicsWindow> {
        let result = self.0.split_graphics(position, size, border, nth_ancestor)?;
        GLOBAL_REACTOR.redraw();
        Ok(GraphicsWindow(result))
    }

    fn set_style(&self, style: Style) {
        self.0.set_style(style);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn resplit_proportional(&self, percent: u32, nth_ancestor: u32) {
        self.0.resplit_proportional(percent, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }

    fn set_as_current(&self) {
        self.0.set_as_current();
    }

    fn get_size(&self) -> (u32, u32) {
        self.0.get_size()
    }

    fn clear(&self) {
        self.0.clear();
    }
}

impl CharInput for GraphicsWindow {
    fn request_char(&self) -> Result<CharFuture<'_>> {
        Ok(CharFuture {
            inner: GLOBAL_REACTOR.request_char(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_char(self: &Arc<Self>) -> Result<ArcCharFuture<Self>> {
        Ok(ArcCharFuture {
            inner: GLOBAL_REACTOR.request_char(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_keyboard_request(&self) {
        GLOBAL_REACTOR.cancel_keyboard_request(self.0.id());
    }
}

impl MouseInput for GraphicsWindow {
    fn request_mouse(&self) -> Result<MouseFuture<'_>> {
        Ok(MouseFuture {
            inner: GLOBAL_REACTOR.request_mouse(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_mouse(self: &Arc<Self>) -> Result<ArcMouseFuture<Self>> {
        Ok(ArcMouseFuture {
            inner: GLOBAL_REACTOR.request_mouse(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_mouse_request(&self) {
        GLOBAL_REACTOR.cancel_mouse_request(self.0.id());
    }
}

impl HyperlinkInput for GraphicsWindow {
    fn set_hyperlink(&self, linkval: u32) -> Result<()> {
        self.0.set_hyperlink(linkval)
    }

    fn request_hyperlink(&self) -> Result<HyperlinkFuture<'_>> {
        Ok(HyperlinkFuture {
            inner: GLOBAL_REACTOR.request_hyperlink(self.0.id())?,
            phantom: PhantomData,
        })
    }

    fn arc_request_hyperlink(self: &Arc<Self>) -> Result<ArcHyperlinkFuture<Self>> {
        Ok(ArcHyperlinkFuture {
            inner: GLOBAL_REACTOR.request_hyperlink(self.0.id())?,
            weak: Arc::downgrade(self),
        })
    }

    fn cancel_hyperlink_request(&self) {
        GLOBAL_REACTOR.cancel_hyperlink_request(self.0.id());
    }
}

impl Drop for GraphicsWindow {
    fn drop(&mut self) {
        GLOBAL_REACTOR.close_window(self.0.id());
    }
}

impl GraphicsWindow {
    /// Repositions the split of one of the window's ancestors, using `self` as a
    /// key window.
    /// 
    /// An `nth_ancestor` argument of 1 will operate on the widow's parent, 2 will
    /// operate on its grandparent, and so-forth. If the `nth_ancestor` argument
    /// is zero or out of range, the call will have no effect.
    pub fn resplit(&self, size: GraphicsWindowSize, nth_ancestor: u32) {
        self.0.resplit_graphics(size, nth_ancestor);
        GLOBAL_REACTOR.redraw();
    }

    /// Draws an image.
    /// 
    /// `resource_index` is the Blorb chunk index of a `Pict` resource. `x` and
    /// `y` are the coordinates in the graphics window where the image's
    /// upper-left corner should be placed. The image will be drawn at its
    /// natural scale.
    pub fn draw_image(&self, resource_index: u32, x: i32, y: i32) -> Result<()> {
        self.0.draw_image(resource_index, x, y)
    }

    /// Draws a scaled image.
    /// 
    /// `resource_index` is the Blorb chunk index of a `Pict` resource. `x` and
    /// `y` are the coordinates in the graphics window where the image's
    /// upper-left corner should be placed. `width` and `height` give the
    /// dimensions to which the image will be scaled.
    pub fn draw_image_scaled(
        &self,
        resource_index: u32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.0.draw_image_scaled(resource_index, x, y, width, height)
    }

    /// Sets the window's background color.
    pub fn set_background_color(&self, color: Color) -> Result<()> {
        self.0.set_background_color(color)
    }

    /// Draws a solid-colored rectangle.
    pub fn fill_rect(&self, color: Color, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
        self.0.fill_rect(color, x, y, width, height)
    }

    /// Erases a rectangular area, restoring it to the background color.
    pub fn erase_rect(&self, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
        self.0.erase_rect(x, y, width, height)
    }
}

impl Style {
    /// Sets or clears a style hint.
    /// 
    /// This will request that when `self` is the current active style for any
    /// window matching `class`, it should appear as specified by `hint`.
    /// 
    /// This call always succeeds, but is not guaranteed to have any visible
    /// effect. You can use [`Window::measure_style`] or
    /// [`Window::distinguish_styles`] to try to check whether anything really
    /// happened, but not all Glk implementations return any useful from these
    /// methods either.
    pub fn set_hint(self, class: StyleClass, hint: StyleHint) {
        set_style_hint(self, class, hint)
    }

    pub(crate) fn to_glk(self) -> GlkStyle {
        match self {
            Style::Normal => GlkStyle::Normal,
            Style::Emphasized => GlkStyle::Emphasized,
            Style::Preformatted => GlkStyle::Preformatted,
            Style::Header => GlkStyle::Header,
            Style::Subheader => GlkStyle::Subheader,
            Style::Alert => GlkStyle::Alert,
            Style::Note => GlkStyle::Note,
            Style::BlockQuote => GlkStyle::BlockQuote,
            Style::Input => GlkStyle::Input,
            Style::User1 => GlkStyle::User1,
            Style::User2 => GlkStyle::User2,
        }
    }
}

impl StyleClass {
    pub(crate) fn to_glk(self) -> WinType {
        match self {
            StyleClass::All => WinType::AllTypes,
            StyleClass::TextBuffer => WinType::TextBuffer,
            StyleClass::TextGrid => WinType::TextGrid,
            StyleClass::Graphics => WinType::Graphics,
        }
    }
}

impl TextWindowSize {
    pub(crate) fn to_method_and_size(
        self,
        position: WindowPosition,
        border: bool,
    ) -> (WinMethod, u32) {
        match self {
            TextWindowSize::Proportional { mut percent } => {
                if percent > 100 {
                    percent = 100;
                }

                (position.to_method(true, border), percent)
            }
            TextWindowSize::Fixed { chars } => (position.to_method(false, border), chars),
        }
    }
}

impl GraphicsWindowSize {
    pub(crate) fn to_method_and_size(
        self,
        position: WindowPosition,
        border: bool,
    ) -> (WinMethod, u32) {
        match self {
            GraphicsWindowSize::Proportional { mut percent } => {
                if percent > 100 {
                    percent = 100;
                }

                (position.to_method(true, border), percent)
            }
            GraphicsWindowSize::Fixed { pixels } => (position.to_method(false, border), pixels),
        }
    }
}

impl WindowPosition {
    pub(crate) fn to_method(self, fixed: bool, border: bool) -> WinMethod {
        match self {
            WindowPosition::Above => WinMethod::ABOVE,
            WindowPosition::Below => WinMethod::BELOW,
            WindowPosition::Left => WinMethod::LEFT,
            WindowPosition::Right => WinMethod::RIGHT,
        }
        .union(if fixed {
            WinMethod::FIXED
        } else {
            WinMethod::PROPORTIONAL
        })
        .union(if border {
            WinMethod::BORDER
        } else {
            WinMethod::NO_BORDER
        })
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        (value.r as u32) << 16 | (value.g as u32) << 8 | value.b as u32
    }
}

/// Returns a future which becomes ready the next time that windows are
/// rearranged or need to be redrawn.
///
/// This will trigger on program-initiated window rerrangements as well as
/// user-initiated ones.
pub fn on_redraw() -> RedrawFuture {
    GLOBAL_REACTOR.on_redraw()
}

/// Named futures returned by methods which request user input.
///
/// Since you rarely should need to refer to any of these types by name, they're
/// shuffled into this separate module in order to declutter things.
pub mod futures {
    use alloc::{string::String, sync::Weak};
    use core::{
        future::Future,
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    pub use crate::reactor::RedrawFuture;
    use crate::reactor::{CharEvent, HyperlinkEvent, LineEvent, LineTermination, MouseEvent};

    /// Future representing the result of a character input request.
    #[derive(Debug)]
    pub struct CharFuture<'a> {
        pub(super) inner: crate::reactor::CharFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    /// Future representing the result of a character input request
    /// (reference-counting version).
    #[derive(Debug)]
    pub struct ArcCharFuture<T> {
        pub(super) inner: crate::reactor::CharFuture,
        pub(super) weak: Weak<T>,
    }

    /// Future representing the result of a line input request.
    #[derive(Debug)]
    pub struct LineFuture<'a> {
        pub(super) inner: crate::reactor::LineFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    /// Future representing the result of a line input request
    /// (reference-counting version).
    #[derive(Debug)]
    pub struct ArcLineFuture<T> {
        pub(super) inner: crate::reactor::LineFuture,
        pub(super) weak: Weak<T>,
    }

    /// Future representing the result of a mouse input request.
    #[derive(Debug)]
    pub struct MouseFuture<'a> {
        pub(super) inner: crate::reactor::MouseFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    /// Future representing the result of a mouse input request
    /// (reference-counting version).
    #[derive(Debug)]
    pub struct ArcMouseFuture<T> {
        pub(super) inner: crate::reactor::MouseFuture,
        pub(super) weak: Weak<T>,
    }

    /// Future representing the result of a hyperlink input request.
    #[derive(Debug)]
    pub struct HyperlinkFuture<'a> {
        pub(super) inner: crate::reactor::HyperlinkFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    /// Future representing the result of a hyperlink input request
    /// (reference-counting version).
    #[derive(Debug)]
    pub struct ArcHyperlinkFuture<T> {
        pub(super) inner: crate::reactor::HyperlinkFuture,
        pub(super) weak: Weak<T>,
    }

    impl Future for CharFuture<'_> {
        type Output = CharEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let pinned = Pin::new(&mut this.inner);
            pinned.poll(cx)
        }
    }

    impl<T> Future for ArcCharFuture<T> {
        type Output = CharEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(_strong) = this.weak.upgrade() {
                let pinned = Pin::new(&mut this.inner);
                pinned.poll(cx)
            } else {
                Poll::Ready(CharEvent::Cancelled)
            }
        }
    }

    impl Future for LineFuture<'_> {
        type Output = LineEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let pinned = Pin::new(&mut this.inner);
            pinned.poll(cx)
        }
    }

    impl<T> Future for ArcLineFuture<T> {
        type Output = LineEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(_strong) = this.weak.upgrade() {
                let pinned = Pin::new(&mut this.inner);
                pinned.poll(cx)
            } else {
                Poll::Ready(LineEvent {
                    input: String::new(),
                    termination: LineTermination::Cancelled,
                })
            }
        }
    }

    impl Future for MouseFuture<'_> {
        type Output = MouseEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let pinned = Pin::new(&mut this.inner);
            pinned.poll(cx)
        }
    }

    impl<T> Future for ArcMouseFuture<T> {
        type Output = MouseEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(_strong) = this.weak.upgrade() {
                let pinned = Pin::new(&mut this.inner);
                pinned.poll(cx)
            } else {
                Poll::Ready(MouseEvent::Cancelled)
            }
        }
    }

    impl Future for HyperlinkFuture<'_> {
        type Output = HyperlinkEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let pinned = Pin::new(&mut this.inner);
            pinned.poll(cx)
        }
    }

    impl<T> Future for ArcHyperlinkFuture<T> {
        type Output = HyperlinkEvent;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(_strong) = this.weak.upgrade() {
                let pinned = Pin::new(&mut this.inner);
                pinned.poll(cx)
            } else {
                Poll::Ready(HyperlinkEvent::Cancelled)
            }
        }
    }
}
