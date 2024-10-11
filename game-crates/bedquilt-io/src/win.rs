use crate::{error::*, reactor::GLOBAL_REACTOR, sys::io::*};
use alloc::sync::Arc;
use core::marker::PhantomData;
use wasm2glulx_ffi::glk::{Keycode, Style as GlkStyle, WinMethod, WinType};

pub use crate::reactor::{CharEvent, HyperlinkEvent, LineEvent, LineTermination, MouseEvent};

pub mod futures {
    use alloc::sync::Weak;
    use core::{
        future::Future,
        marker::PhantomData,
        pin::Pin,
        task::{Context, Poll},
    };

    use crate::reactor::{CharEvent, HyperlinkEvent, LineEvent, LineTermination, MouseEvent};
    pub use crate::reactor::RedrawFuture;

    #[derive(Debug)]
    pub struct CharFuture<'a> {
        pub(super) inner: crate::reactor::CharFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    #[derive(Debug)]
    pub struct ArcCharFuture<T> {
        pub(super) inner: crate::reactor::CharFuture,
        pub(super) weak: Weak<T>,
    }

    #[derive(Debug)]
    pub struct LineFuture<'a> {
        pub(super) inner: crate::reactor::LineFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    #[derive(Debug)]
    pub struct ArcLineFuture<T> {
        pub(super) inner: crate::reactor::LineFuture,
        pub(super) weak: Weak<T>,
    }

    #[derive(Debug)]
    pub struct MouseFuture<'a> {
        pub(super) inner: crate::reactor::MouseFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

    #[derive(Debug)]
    pub struct ArcMouseFuture<T> {
        pub(super) inner: crate::reactor::MouseFuture,
        pub(super) weak: Weak<T>,
    }

    #[derive(Debug)]
    pub struct HyperlinkFuture<'a> {
        pub(super) inner: crate::reactor::HyperlinkFuture,
        pub(super) phantom: PhantomData<&'a ()>,
    }

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

use futures::*;

pub trait Window: Sized {
    fn create_as_root() -> Result<Self>;
    fn split_blank(
        &self,
        position: WindowPosition,
        size_pct: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow>;
    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow>;
    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow>;
    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow>;
    fn resize_proportional(&self, percent: u32, nth_parent: u32);
    fn measure_style(&self, style: Style) -> StyleMeasurement;
    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool;
}

pub trait KeyboardInput: Window {
    fn request_char(&self) -> Result<CharFuture<'_>>;
    fn arc_request_char(self: &Arc<Self>) -> Result<ArcCharFuture<Self>>;
    fn request_line<'a>(&'a self, initial: &str) -> Result<LineFuture<'a>>;
    fn arc_request_line(self: &Arc<Self>, initial: &str) -> Result<ArcLineFuture<Self>>;
    fn cancel_keyboard_request(&self);
}

pub trait MouseInput: Window {
    fn request_mouse(&self) -> Result<MouseFuture<'_>>;
    fn arc_request_mouse(self: &Arc<Self>) -> Result<ArcMouseFuture<Self>>;
    fn cancel_mouse_request(&self);
    fn request_hyperlink(&self) -> Result<HyperlinkFuture<'_>>;
    fn arc_request_hyperlink(self: &Arc<Self>) -> Result<ArcHyperlinkFuture<Self>>;
    fn cancel_hyperlink_request(&self);
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BlankWindow(pub(crate) BlankWindowImpl);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextBufferWindow(pub(crate) TextBufferWindowImpl);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TextGridWindow(pub(crate) TextGridWindowImpl);
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GraphicsWindow(pub(crate) GraphicsWindowImpl);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WindowPosition {
    Above,
    Below,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlankWindowSize {
    Proportional { percent: u32 },
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextWindowSize {
    Proportional { percent: u32 },
    Fixed { chars: u32 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GraphicsWindowSize {
    Proportional { percent: u32 },
    Fixed { pixels: u32 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Style {
    Normal,
    Emphasized,
    Preformatted,
    Header,
    Subheader,
    Alert,
    Note,
    BlockQuote,
    Input,
    User1,
    User2,
}

impl Style {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StyleHint {
    Indentation(Option<i32>),
    ParaIndentation(Option<i32>),
    Justification(Option<Justification>),
    Size(Option<i32>),
    Weight(Option<Weight>),
    Oblique(Option<bool>),
    Proportional(Option<bool>),
    TextColor(Option<Color>),
    BackColor(Option<Color>),
    ReverseColor(Option<bool>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StyleClass {
    All,
    TextBuffer,
    TextGrid,
    Graphics,
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

pub struct StyleMeasurement {
    pub indentation: Option<i32>,
    pub para_indentation: Option<i32>,
    pub justification: Option<Justification>,
    pub size: Option<i32>,
    pub weight: Option<Weight>,
    pub oblique: Option<bool>,
    pub proportional: Option<bool>,
    pub text_color: Option<Color>,
    pub back_color: Option<Color>,
    pub reverse_color: Option<bool>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Justification {
    LeftFlush,
    LeftRight,
    Centered,
    RightFlush,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Weight {
    Light,
    Normal,
    Bold,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
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
    Hyerlink {
        win: WinId,
        linkid: u32,
    },
    VolumeNotify {
        notify: u32,
    },
}

impl BlankWindowSize {
    pub(crate) fn to_method_and_size(
        self,
        position: WindowPosition,
        border: bool,
    ) -> (WinMethod, u32) {
        let Self::Proportional { mut percent } = self;
        if percent > 100 {
            percent = 100;
        }

        (position.to_method(true, border), percent)
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

impl Window for BlankWindow {
    fn create_as_root() -> Result<Self> {
        Ok(Self(BlankWindowImpl::create_as_root()?))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        self.0.split_blank(position, size, border, nth_parent)
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        self.0.split_text_buffer(position, size, border, nth_parent)
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        self.0.split_text_grid(position, size, border, nth_parent)
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        self.0.split_graphics(position, size, border, nth_parent)
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        self.0.resize_proportional(percent, nth_parent);
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }
}

impl Window for TextBufferWindow {
    fn create_as_root() -> Result<Self> {
        Ok(Self(TextBufferWindowImpl::create_as_root()?))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        self.0.split_blank(position, size, border, nth_parent)
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        self.0.split_text_buffer(position, size, border, nth_parent)
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        self.0.split_text_grid(position, size, border, nth_parent)
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        self.0.split_graphics(position, size, border, nth_parent)
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        self.0.resize_proportional(percent, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }
}

impl KeyboardInput for TextBufferWindow {
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

    fn cancel_keyboard_request(&self) {
        GLOBAL_REACTOR.cancel_keyboard_request(self.0.id());
    }
}

impl MouseInput for TextBufferWindow {
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

impl TextBufferWindow {
    pub fn resize_chars(&self, chars: u32, nth_parent: u32) {
        self.0.resize_chars(chars, nth_parent);
    }

    pub fn transcript(&self, fref: &crate::fs::FileRef, append: bool) -> Result<()> {
        self.0.transcript(&fref.0, append)
    }

    pub fn transcript_off(&self) {
        self.0.transcript_off();
    }

    pub fn set_hyperlink(&self, linkval: u32) -> Result<()> {
        self.0.set_hyperlink(linkval)
    }
}

impl Window for TextGridWindow {
    fn create_as_root() -> Result<Self> {
        Ok(Self(TextGridWindowImpl::create_as_root()?))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        self.0.split_blank(position, size, border, nth_parent)
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        self.0.split_text_buffer(position, size, border, nth_parent)
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        self.0.split_text_grid(position, size, border, nth_parent)
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        self.0.split_graphics(position, size, border, nth_parent)
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        self.0.resize_proportional(percent, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
    }
}

impl KeyboardInput for TextGridWindow {
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

    fn cancel_keyboard_request(&self) {
        GLOBAL_REACTOR.cancel_keyboard_request(self.0.id());
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

impl TextGridWindow {
    pub fn resize_chars(&self, chars: u32, nth_parent: u32) {
        self.0.resize_chars(chars, nth_parent);
    }
}

impl Window for GraphicsWindow {
    fn create_as_root() -> Result<Self> {
        Ok(Self(GraphicsWindowImpl::create_as_root()?))
    }

    fn split_blank(
        &self,
        position: WindowPosition,
        size: BlankWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<BlankWindow> {
        self.0.split_blank(position, size, border, nth_parent)
    }

    fn split_text_buffer(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextBufferWindow> {
        self.0.split_text_buffer(position, size, border, nth_parent)
    }

    fn split_text_grid(
        &self,
        position: WindowPosition,
        size: TextWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<TextGridWindow> {
        self.0.split_text_grid(position, size, border, nth_parent)
    }

    fn split_graphics(
        &self,
        position: WindowPosition,
        size: GraphicsWindowSize,
        border: bool,
        nth_parent: u32,
    ) -> Result<GraphicsWindow> {
        self.0.split_graphics(position, size, border, nth_parent)
    }

    fn resize_proportional(&self, percent: u32, nth_parent: u32) {
        self.0.resize_proportional(percent, nth_parent);
    }

    fn measure_style(&self, style: Style) -> StyleMeasurement {
        self.0.measure_style(style)
    }

    fn distinguish_styles(&self, style1: Style, style2: Style) -> bool {
        self.0.distinguish_styles(style1, style2)
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

impl GraphicsWindow {
    pub fn resize_pixels(&self, pixels: u32, nth_parent: u32) {
        self.0.resize_pixels(pixels, nth_parent);
    }
}

/// Returns a future which becomes ready the next time that windows are
/// rearranged or need to be redrawn.
pub fn on_redraw() -> RedrawFuture {
    GLOBAL_REACTOR.on_redraw()
}