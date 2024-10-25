//! Sound.
//!
//! This module provides access to audio-playback features on Glk
//! implementations that support it. Any sounds that you wish to play must be
//! include as `Snd` resources in your game's Blorb file. You can open a sound
//! channel using [`SoundChannel::new`] and then use its `play` methods to play
//! sounds identified by their resource index, optionally obtaining a future
//! which will become ready when playback finishes.

use crate::{
    error::Result,
    reactor::{add_waker, GLOBAL_REACTOR},
    sync::Mutex,
    sys::glk::SchanImpl,
};
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
    task::{Context, Poll, Waker},
};

/// A channel through which sounds can be played.
/// 
/// Each sound channel that you construct is capable of playing one sound
/// resource at a time. However, the name is a slight misnomer because each
/// resource may have multiple audio channels encoded within it. That is, to
/// play an .ogg file encoded in stereo, you only need one sound channel, not
/// two. But playing two .ogg files in parallel requires two channels.
#[derive(Debug)]
pub struct SoundChannel {
    inner: SchanImpl,
    stop_count: AtomicU32,
    wakers: Mutex<Vec<Waker>>,
}

impl SoundChannel {
    /// Constant representing full sound volume.
    /// 
    /// See [`set_volume`](Self::set_volume) for detail about the interpretation of this value.
    pub const FULL_VOLUME: u32 = 0x10000;

    fn stop_and_wake(&self) {
        self.stop_count.fetch_add(1, Ordering::Release);
        let wakers = core::mem::take(&mut *self.wakers.lock());
        for waker in wakers {
            waker.wake();
        }
    }

    /// Opens a new sound channel.
    pub fn new() -> Result<Self> {
        Ok(SoundChannel {
            inner: SchanImpl::new(Self::FULL_VOLUME)?,
            stop_count: AtomicU32::new(0),
            wakers: Mutex::new(Vec::new()),
        })
    }

    /// Plays a sound.
    pub fn play(&self, resource_index: u32, repeats: u32) -> Result<()> {
        self.inner.play(resource_index, repeats, 0)?;
        self.stop_and_wake();
        Ok(())
    }

    /// Plays a sound, returning a future that becomes ready when it finishes.
    /// 
    /// The future will also become ready if playback is ended prematurely as a
    /// result of calling `stop` or playing a different sound.
    pub fn play_notify(&self, resource_index: u32, repeats: u32) -> Result<SoundFuture<'_>> {
        let mut fut = GLOBAL_REACTOR.sound_notify();
        match self.inner.play(resource_index, repeats, fut.0) {
            Ok(()) => {
                self.stop_and_wake();
                Ok(SoundFuture {
                    inner: fut,
                    channel: self,
                    stop_count: self.stop_count.load(Ordering::Acquire),
                })
            }
            Err(e) => {
                fut.cancel();
                Err(e)
            }
        }
    }

    /// Plays a sound on a reference-counted channel, returning a future that
    /// becomes ready when it finishes.
    ///
    /// The returned future holds a *weak* reference to the sound channel, so
    /// holding the future will not prevent the channel from closing if all
    /// strong references to it are dropped. If this occurs, the future will
    /// immediately become ready. The future will also become ready if playback
    /// is ended prematurely as a result of calling `stop` or playing a
    /// different sound.
    pub fn arc_play_notify(
        self: &Arc<Self>,
        resource_index: u32,
        repeats: u32,
    ) -> Result<ArcSoundFuture> {
        let mut fut = GLOBAL_REACTOR.sound_notify();
        match self.inner.play(resource_index, repeats, fut.0) {
            Ok(()) => {
                self.stop_and_wake();
                Ok(ArcSoundFuture {
                    inner: fut,
                    channel: Arc::downgrade(self),
                    stop_count: self.stop_count.load(Ordering::Acquire),
                })
            }
            Err(e) => {
                fut.cancel();
                Err(e)
            }
        }
    }

    /// Pauses the sound channel.
    pub fn pause(&self) {
        self.inner.pause();
    }

    /// Unpauses the sound channel, resuming whatever sound was playing when it
    /// was paused.
    pub fn unpause(&self) {
        self.inner.unpause();
    }

    /// Stops whatever sound is currently playing on the channel.
    pub fn stop(&self) {
        self.inner.stop();
        self.stop_and_wake();
    }

    /// Sets the sound volume.
    ///
    /// A volume argument of [`Self::FULL_VOLUME`] will pass the sound directly
    /// to the OS sound driver without adjustment. Values in excess of
    /// `FULL_VOLUME` are allowed, but may result in clipping. Sound pressure is
    /// linear in this argument, so increasing or decreasing it by a factor of
    /// two will alter the volume by ±6 dB SPL.
    ///
    /// If `duration` is non-zero, the volume will be adjusted gradually over
    /// that many milliseconds.
    pub fn set_volume(&self, volume: u32, duration: u32) {
        self.inner.set_volume(volume, duration, 0);
    }

    /// Sets the sound volume, returning a future which becomes ready when it
    /// has fully adjusted.
    pub fn set_volume_notify(&self, volume: u32, duration: u32) -> SoundFuture<'_> {
        let fut = GLOBAL_REACTOR.sound_notify();
        self.inner.set_volume(volume, duration, fut.0);
        SoundFuture {
            inner: fut,
            channel: self,
            stop_count: self.stop_count.load(Ordering::Acquire),
        }
    }

    /// Sets the sound volume on a reference-counted channel, returning a future
    /// which becomes ready when it has fully adjusted.
    /// 
    /// The future holds a *weak* reference to the underlying channel and will
    /// become ready immediately if the channel is dropped.
    pub fn arc_set_volume_notify(self: &Arc<Self>, volume: u32, duration: u32) -> ArcSoundFuture {
        let fut = GLOBAL_REACTOR.sound_notify();
        self.inner.set_volume(volume, duration, fut.0);
        ArcSoundFuture {
            inner: fut,
            channel: Arc::downgrade(self),
            stop_count: self.stop_count.load(Ordering::Acquire),
        }
    }
}

impl Drop for SoundChannel {
    fn drop(&mut self) {
        self.stop_and_wake();
    }
}

use self::futures::*;

/// Named futures returned from sound channels.
/// 
/// Since you rarely should need to refer to any of these types by name, they're
/// shuffled into this separate module in order to declutter things.
pub mod futures {
    use super::*;

    /// Future which becomes ready when a sound finishes playing.
    #[derive(Debug)]
    pub struct SoundFuture<'a> {
        pub(super) inner: crate::reactor::SoundFuture,
        pub(super) channel: &'a SoundChannel,
        pub(super) stop_count: u32,
    }

    /// Future which becomes ready when a sound finishes playing (reference-counting
    /// version).
    #[derive(Debug)]
    pub struct ArcSoundFuture {
        pub(super) inner: crate::reactor::SoundFuture,
        pub(super) channel: Weak<SoundChannel>,
        pub(super) stop_count: u32,
    }

    impl Future for SoundFuture<'_> {
        type Output = ();
    
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let pinned = Pin::new(&mut this.inner);
            if pinned.poll(cx).is_ready() {
                Poll::Ready(())
            } else if this.channel.stop_count.load(Ordering::Acquire) != this.stop_count {
                this.inner.cancel();
                Poll::Ready(())
            } else {
                let mut wakers = this.channel.wakers.lock();
                add_waker(&mut wakers, cx);
                Poll::Pending
            }
        }
    }
    
    impl Future for ArcSoundFuture {
        type Output = ();
    
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if let Some(strong) = this.channel.upgrade() {
                let pinned = Pin::new(&mut this.inner);
                if pinned.poll(cx).is_ready() {
                    Poll::Ready(())
                } else if strong.stop_count.load(Ordering::Acquire) != this.stop_count {
                    this.inner.cancel();
                    Poll::Ready(())
                } else {
                    let mut wakers = strong.wakers.lock();
                    add_waker(&mut wakers, cx);
                    Poll::Pending
                }
            } else {
                this.inner.cancel();
                Poll::Ready(())
            }
        }
    }
    
}

