use crate::error::Result;
use crate::sync::{MappedMutexGuard, Mutex, MutexGuard};
use crate::sys::glk::{
    cancel_char, cancel_hyperlink, cancel_line, cancel_mouse, request_char, request_hyperlink, request_line, request_mouse, write_str, WinId
};

use alloc::borrow::ToOwned;
use alloc::{
    boxed::Box,
    collections::{BinaryHeap, VecDeque},
    string::String,
    sync::Arc,
    task::Wake,
    vec::Vec,
};
use core::{
    any::Any,
    cmp::Reverse,
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use hashbrown::{HashMap, HashSet};
use wasm2glulx_ffi::glk::Keycode;

type TaskId = u64;
type TimerId = u64;
type Tick = u64;
type SoundNotifyId = u32;

type TaskResult = Box<dyn Any + Send>;
type Task = Pin<Box<dyn Future<Output = TaskResult> + Send>>;

/// The result of a line input event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LineEvent {
    /// The line of input entered by the user.
    pub input: String,
    /// The way in which the line input was completed.
    pub termination: LineTermination,
}

/// The reason for completion of a line input event.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LineTermination {
    /// Line input was ended by the user pressing enter.
    Normal,
    /// Line input was ended by the user pressing the given key.
    Terminator(Keycode),
    /// Line input was ended because the program cancelled the request.
    Cancelled,
}

/// The result of a character input event.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CharEvent {
    /// The given character was entered.
    Normal(char),
    /// The given key was pressed.
    Terminator(Keycode),
    /// The character request was cancelled.
    Cancelled,
}

/// The result of a mouse input event.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseEvent {
    /// The user clicked at the given coordinates.
    Click {
        /// Pixels or characters right from the left edge of the window.
        x: u32,
        /// Pixels or characters down from the top edge of the window.
        y: u32,
    },
    /// The mouse input request was cancelled.
    Cancelled,
}

/// The result of a hyperlink input event.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum HyperlinkEvent {
    /// The hyperlink with the given number was clicked.
    Click(u32),
    /// The request was cancelled.
    Cancelled,
}

#[derive(Debug, Clone, Default)]
struct PendingWindowInput {
    char_pending: bool,
    line_pending: bool,
    mouse_pending: bool,
    hyperlink_pending: bool,
    char_waiters: usize,
    line_waiters: usize,
    mouse_waiters: usize,
    hyperlink_waiters: usize,
    line_initial: VecDeque<String>,
}

pub struct Reactor(Mutex<Option<ReactorState>>);

#[derive(Default)]
struct ReactorState {
    next_task_id: TaskId,
    next_timer_id: TimerId,
    next_sound_notify_id: SoundNotifyId,

    redraw_count: u64,
    tick_count: Tick,
    pending_input: HashMap<WinId, PendingWindowInput>,
    line_input: HashMap<WinId, VecDeque<LineEvent>>,
    char_input: HashMap<WinId, VecDeque<CharEvent>>,
    mouse_input: HashMap<WinId, VecDeque<MouseEvent>>,
    hyperlink_input: HashMap<WinId, VecDeque<HyperlinkEvent>>,
    sound_notifications: HashSet<SoundNotifyId>,
    dropped_sound_futures: HashSet<SoundNotifyId>,

    timers: BinaryHeap<(Reverse<Tick>, TimerId)>,

    window_interest: HashMap<WinId, Vec<Waker>>,
    redraw_interest: Vec<Waker>,
    timer_interest: HashMap<TimerId, Vec<Waker>>,
    join_interest: HashMap<TaskId, Vec<Waker>>,
    sound_interest: HashMap<SoundNotifyId, Vec<Waker>>,

    ready_tasks: VecDeque<(TaskId, Task)>,
    unready_tasks: HashMap<TaskId, Task>,
    done_tasks: HashMap<TaskId, TaskResult>,
    dropped_handles: HashSet<TaskId>,
}

struct TaskWaker(TaskId);

/// Handle to poll for task completion and collect its result.
///
/// Join handles can safely be dropped at any time. This will neither leak
/// resources nor cancel the task.
#[derive(Debug)]
pub struct JoinHandle<T> {
    task: TaskId,
    result: PhantomData<T>,
    cancelled: bool,
}

#[derive(Debug)]
pub struct CharFuture(WinId);

#[derive(Debug)]
pub struct LineFuture(WinId);

#[derive(Debug)]
pub struct MouseFuture(WinId);

#[derive(Debug)]
pub struct HyperlinkFuture(WinId);

/// Future which becomes ready when a timer elapses.
#[derive(Debug)]
pub struct TimerFuture(TimerId, Tick);

/// Future which becomes ready on the next redraw or rearrange event.
#[derive(Debug)]
pub struct RedrawFuture(u64);

#[derive(Debug)]
pub struct SoundFuture(pub(crate) SoundNotifyId);

impl<T> Unpin for JoinHandle<T> {}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let mut state = GLOBAL_REACTOR.state();
        if let Some(task) = state.unready_tasks.remove(&self.0) {
            state.ready_tasks.push_back((self.0, task));
        }
    }
}

impl Reactor {
    fn state(&self) -> MappedMutexGuard<'_, ReactorState> {
        MutexGuard::map(self.0.lock(), |opt| {
            opt.get_or_insert(ReactorState::default())
        })
    }

    pub fn spawn<F>(&self, fut: F) -> JoinHandle<F::Output>
    where
        F: IntoFuture,
        F::IntoFuture: Send + 'static,
        F::Output: Send + 'static,
    {
        let fut = fut.into_future();
        let task: Task = Box::pin(async {
            let result = fut.await;
            let boxed: TaskResult = Box::new(result);
            boxed
        });

        let mut state = self.state();
        let task_id = state.next_task_id;
        state.next_task_id += 1;
        state.ready_tasks.push_back((task_id, task));
        JoinHandle {
            task: task_id,
            result: PhantomData,
            cancelled: false,
        }
    }

    fn next_task(&self) -> Option<(TaskId, Task)> {
        let mut state = self.state();
        state.ready_tasks.pop_front()
    }

    pub fn run(&self) {
        loop {
            while let Some((task_id, mut task)) = self.next_task() {
                let waker = Arc::new(TaskWaker(task_id)).into();
                let mut ctx = Context::from_waker(&waker);

                match task.as_mut().poll(&mut ctx) {
                    Poll::Ready(result) => {
                        let mut state = self.state();
                        if !state.dropped_handles.remove(&task_id) {
                            state.done_tasks.insert(task_id, result);
                        }
                        let wakers = state.join_interest.remove(&task_id).unwrap_or_default();
                        core::mem::drop(state);
                        for waker in wakers {
                            waker.wake();
                        }
                    }
                    Poll::Pending => {
                        let mut state = self.state();
                        state.unready_tasks.insert(task_id, task);
                    }
                }
            }

            let state = self.state();
            if state.unready_tasks.is_empty() {
                return;
            }
            if state.window_interest.is_empty()
                && state.timer_interest.is_empty()
                && state.join_interest.is_empty()
                && state.redraw_interest.is_empty()
            {
                panic!("Executor deadlocked")
            }
            core::mem::drop(state);

            match crate::sys::glk::event_wait() {
                crate::win::Event::Timer => {
                    let mut state_guard = self.state();
                    let state = &mut *state_guard;
                    state.tick_count += 1;
                    let mut wakers = Vec::new();
                    while let Some((Reverse(tick_ref), timer_id_ref)) = state.timers.peek() {
                        if *tick_ref <= state.tick_count {
                            let timer_id = *timer_id_ref;
                            if let Some(mut timer_wakers) = state.timer_interest.remove(&timer_id) {
                                wakers.append(&mut timer_wakers);
                            }
                            state.timers.pop();
                        } else {
                            break;
                        }
                    }
                    core::mem::drop(state_guard);
                    for waker in wakers {
                        waker.wake();
                    }
                }
                crate::win::Event::CharInput { win, input } => {
                    let mut state = self.state();
                    if let Some(pending) = state.pending_input.get_mut(&win) {
                        pending.char_waiters = pending.char_waiters.saturating_sub(1);
                        pending.char_pending = false;
                        requeue_win_events(&mut state, win);
                    }
                    let mut input_events = state.char_input.remove(&win).unwrap_or_default();
                    input_events.push_back(CharEvent::Normal(input));
                    state.char_input.insert(win, input_events);
                    let wakers = state.window_interest.remove(&win).unwrap_or_default();
                    core::mem::drop(state);
                    for waker in wakers {
                        waker.wake();
                    }
                }
                crate::win::Event::CharInputSpecial { win, input } => {
                    let mut state = self.state();
                    if let Some(pending) = state.pending_input.get_mut(&win) {
                        pending.char_waiters = pending.char_waiters.saturating_sub(1);
                        pending.char_pending = false;
                        requeue_win_events(&mut state, win);
                    }
                    let mut input_events = state.char_input.remove(&win).unwrap_or_default();
                    input_events.push_back(CharEvent::Terminator(input));
                    state.char_input.insert(win, input_events);
                    let wakers = state.window_interest.remove(&win).unwrap_or_default();
                    core::mem::drop(state);
                    for waker in wakers {
                        waker.wake();
                    }
                }
                crate::win::Event::LineInput {
                    win,
                    input,
                    terminator,
                } => {
                    let mut state = self.state();
                    if let Some(pending) = state.pending_input.get_mut(&win) {
                        pending.line_waiters = pending.line_waiters.saturating_sub(1);
                        pending.line_pending = false;
                        requeue_win_events(&mut state, win);
                    }
                    let mut input_events = state.line_input.remove(&win).unwrap_or_default();
                    input_events.push_back(LineEvent {
                        input,
                        termination: terminator
                            .map(LineTermination::Terminator)
                            .unwrap_or(LineTermination::Normal),
                    });
                    state.line_input.insert(win, input_events);
                    let wakers = state.window_interest.remove(&win).unwrap_or_default();
                    core::mem::drop(state);
                    for waker in wakers {
                        waker.wake();
                    }
                }
                crate::win::Event::MouseInput { win, x, y } => {
                    let mut state = self.state();
                    if let Some(counts) = state.pending_input.get_mut(&win) {
                        counts.mouse_pending = false;
                    }
                    let mut input_events = state.mouse_input.remove(&win).unwrap_or_default();
                    input_events.push_back(MouseEvent::Click { x, y });
                    state.mouse_input.insert(win, input_events);
                    let wakers = state.window_interest.remove(&win).unwrap_or_default();
                    core::mem::drop(state);
                    for waker in wakers {
                        waker.wake();
                    }
                }
                crate::win::Event::Hyperlink { win, linkid } => {
                    let mut state = self.state();
                    if let Some(counts) = state.pending_input.get_mut(&win) {
                        counts.hyperlink_pending = false;
                    }
                    let mut input_events = state.hyperlink_input.remove(&win).unwrap_or_default();
                    input_events.push_back(HyperlinkEvent::Click(linkid));
                    state.hyperlink_input.insert(win, input_events);
                    let wakers = state.window_interest.remove(&win).unwrap_or_default();
                    core::mem::drop(state);
                    for waker in wakers {
                        waker.wake();
                    }
                }

                crate::win::Event::Arrange { win: _ } | crate::win::Event::Redraw { win: _ } => {
                    self.redraw();
                }
                crate::win::Event::SoundNotify {
                    resource: _,
                    notify,
                }
                | crate::win::Event::VolumeNotify { notify } => {
                    let mut state = self.state();
                    if !state.dropped_sound_futures.remove(&notify) {
                        state.sound_notifications.insert(notify);
                    }
                    let wakers = state.sound_interest.remove(&notify).unwrap_or_default();
                    for waker in wakers {
                        waker.wake();
                    }
                }
            }
        }
    }

    pub fn redraw(&self) {
        let mut state = self.state();
        state.redraw_count += 1;
        let wakers = core::mem::take(&mut state.redraw_interest);
        core::mem::drop(state);
        for waker in wakers {
            waker.wake();
        }
    }

    fn poll_task(&self, cx: &mut Context<'_>, task_id: TaskId) -> Poll<TaskResult> {
        let mut state = self.state();
        if let Some(result) = state.done_tasks.remove(&task_id) {
            Poll::Ready(result)
        } else {
            let mut wakers = state.join_interest.remove(&task_id).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.join_interest.insert(task_id, wakers);
            Poll::Pending
        }
    }

    pub fn request_char(&self, win: WinId) -> Result<CharFuture> {
        let mut state = self.state();
        let mut pending = state.pending_input.remove(&win).unwrap_or_default();
        if !pending.char_pending && !pending.line_pending {
            if let Err(e) = request_char(win) {
                state.pending_input.insert(win, pending);
                return Err(e);
            }
            pending.char_pending = true;
        }
        pending.char_waiters += 1;
        state.pending_input.insert(win, pending);
        Ok(CharFuture(win))
    }

    pub fn request_line(&self, win: WinId, initial: &str) -> Result<LineFuture> {
        let mut state = self.state();
        let mut pending = state.pending_input.remove(&win).unwrap_or_default();
        if !pending.char_pending && !pending.line_pending {
            if let Err(e) = request_line(win, initial) {
                state.pending_input.insert(win, pending);
                return Err(e);
            }
            pending.line_pending = true;
        } else {
            pending.line_initial.push_back(initial.to_owned());
        }
        pending.line_waiters += 1;
        state.pending_input.insert(win, pending);
        Ok(LineFuture(win))
    }

    fn poll_char(&self, cx: &mut Context<'_>, win: WinId) -> Poll<CharEvent> {
        let mut state = self.state();
        let mut char_input = state.char_input.remove(&win).unwrap_or_default();
        let result = char_input.pop_front();
        state.char_input.insert(win, char_input);
        if let Some(event) = result {
            Poll::Ready(event)
        } else {
            let mut wakers = state.window_interest.remove(&win).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.window_interest.insert(win, wakers);
            Poll::Pending
        }
    }

    fn poll_line(&self, cx: &mut Context<'_>, win: WinId) -> Poll<LineEvent> {
        let mut state = self.state();
        let mut line_input = state.line_input.remove(&win).unwrap_or_default();
        let result = line_input.pop_front();
        state.line_input.insert(win, line_input);
        if let Some(event) = result {
            Poll::Ready(event)
        } else {
            let mut wakers = state.window_interest.remove(&win).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.window_interest.insert(win, wakers);
            Poll::Pending
        }
    }

    pub fn cancel_keyboard_request(&self, win: WinId) {
        let mut state_guard = self.state();
        let state = &mut *state_guard;
        if let Some(pending) = state.pending_input.get_mut(&win) {
            if pending.char_pending {
                pending.char_waiters = pending.char_waiters.saturating_sub(1);
                let mut char_input = state.char_input.remove(&win).unwrap_or_default();
                char_input.push_back(CharEvent::Cancelled);
                state.char_input.insert(win, char_input);
                requeue_win_events(state, win);
            } else if pending.line_pending {
                pending.line_waiters = pending.line_waiters.saturating_sub(1);
                pending.line_pending = false;
                let cancelled = cancel_line(win);
                let mut line_input = state.line_input.remove(&win).unwrap_or_default();
                line_input.push_back(LineEvent {
                    input: cancelled,
                    termination: LineTermination::Cancelled,
                });
                state.line_input.insert(win, line_input);
                requeue_win_events(state, win);
            }
        }
    }

    pub fn request_mouse(&self, win: WinId) -> Result<MouseFuture> {
        let mut state = self.state();
        let mut pending = state.pending_input.remove(&win).unwrap_or_default();
        if !pending.mouse_pending {
            if let Err(e) = request_mouse(win) {
                state.pending_input.insert(win, pending);
                return Err(e);
            }
            pending.mouse_pending = true;
        }
        pending.mouse_waiters += 1;
        state.pending_input.insert(win, pending);
        Ok(MouseFuture(win))
    }

    pub fn poll_mouse(&self, cx: &mut Context<'_>, win: WinId) -> Poll<MouseEvent> {
        let mut state = self.state();
        let mut mouse_input = state.mouse_input.remove(&win).unwrap_or_default();
        let result = mouse_input.pop_front();
        state.mouse_input.insert(win, mouse_input);
        if let Some(event) = result {
            Poll::Ready(event)
        } else {
            let mut wakers = state.window_interest.remove(&win).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.window_interest.insert(win, wakers);
            Poll::Pending
        }
    }

    pub fn cancel_mouse_request(&self, win: WinId) {
        let mut state_guard = self.state();
        let state = &mut *state_guard;
        if let Some(pending) = state.pending_input.get_mut(&win) {
            if pending.mouse_pending {
                pending.mouse_waiters = pending.mouse_waiters.saturating_sub(1);
                let mut mouse_input = state.mouse_input.remove(&win).unwrap_or_default();
                mouse_input.push_back(MouseEvent::Cancelled);
                state.mouse_input.insert(win, mouse_input);
                requeue_win_events(state, win);
            }
        }
    }

    pub fn request_hyperlink(&self, win: WinId) -> Result<HyperlinkFuture> {
        let mut state = self.state();
        let mut pending = state.pending_input.remove(&win).unwrap_or_default();
        if !pending.hyperlink_pending {
            if let Err(e) = request_hyperlink(win) {
                state.pending_input.insert(win, pending);
                return Err(e);
            }
            pending.hyperlink_pending = true;
        }
        pending.hyperlink_waiters += 1;
        state.pending_input.insert(win, pending);
        Ok(HyperlinkFuture(win))
    }

    pub fn poll_hyperlink(&self, cx: &mut Context<'_>, win: WinId) -> Poll<HyperlinkEvent> {
        let mut state = self.state();
        let mut hyperlink_input = state.hyperlink_input.remove(&win).unwrap_or_default();
        let result = hyperlink_input.pop_front();
        state.hyperlink_input.insert(win, hyperlink_input);
        if let Some(event) = result {
            Poll::Ready(event)
        } else {
            let mut wakers = state.window_interest.remove(&win).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.window_interest.insert(win, wakers);
            Poll::Pending
        }
    }

    pub fn cancel_hyperlink_request(&self, win: WinId) {
        let mut state_guard = self.state();
        let state = &mut *state_guard;
        if let Some(pending) = state.pending_input.get_mut(&win) {
            if pending.hyperlink_pending {
                pending.hyperlink_waiters = pending.hyperlink_waiters.saturating_sub(1);
                let mut hyperlink_input = state.hyperlink_input.remove(&win).unwrap_or_default();
                hyperlink_input.push_back(HyperlinkEvent::Cancelled);
                state.hyperlink_input.insert(win, hyperlink_input);
                requeue_win_events(state, win);
            }
        }
    }

    pub fn poll_redraw(&self, cx: &mut Context<'_>, count: u64) -> Poll<()> {
        let mut state = self.state();
        if count > state.redraw_count {
            Poll::Ready(())
        } else {
            add_waker(&mut state.redraw_interest, cx);
            Poll::Pending
        }
    }

    pub fn on_redraw(&self) -> RedrawFuture {
        RedrawFuture(self.state().redraw_count)
    }

    pub fn current_tick(&self) -> Tick {
        self.state().tick_count
    }

    pub fn set_timer(&self, tick: Tick) -> TimerFuture {
        let mut state = self.state();
        let timer_id = state.next_timer_id;
        state.next_timer_id += 1;
        state.timers.push((Reverse(tick), timer_id));
        TimerFuture(timer_id, tick)
    }

    pub fn poll_timer(&self, cx: &mut Context<'_>, timer_id: TimerId, tick: Tick) -> Poll<()> {
        let mut state = self.state();
        if state.tick_count < tick {
            let mut wakers = state.timer_interest.remove(&timer_id).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.timer_interest.insert(timer_id, wakers);
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }

    pub fn close_window(&self, win: WinId) {
        let mut state = self.state();
        if let Some(pending) = state.pending_input.remove(&win) {
            if pending.char_pending {
                cancel_char(win);
            }
            if pending.line_pending {
                cancel_line(win);
            }
            if pending.mouse_pending {
                let _ = cancel_mouse(win);
            }
            if pending.hyperlink_pending {
                let _ = cancel_hyperlink(win);
            }
        }

        state.char_input.remove(&win);
        state.line_input.remove(&win);
        state.mouse_input.remove(&win);
        state.hyperlink_input.remove(&win);
        let wakers = state.window_interest.remove(&win).unwrap_or_default();
        core::mem::drop(state);
        for waker in wakers {
            waker.wake();
        }
    }

    pub fn sound_notify(&self) -> SoundFuture {
        let mut state = self.state();
        if state.next_sound_notify_id == 0 {
            state.next_sound_notify_id = 1;
        }

        let fut = SoundFuture(state.next_sound_notify_id);
        state.next_sound_notify_id += 1;
        fut
    }

    pub fn poll_sound(&self, cx: &mut Context<'_>, sound_id: SoundNotifyId) -> Poll<()> {
        let mut state = self.state();
        if state.sound_notifications.remove(&sound_id) {
            Poll::Ready(())
        } else {
            let mut wakers = state.sound_interest.remove(&sound_id).unwrap_or_default();
            add_waker(&mut wakers, cx);
            state.sound_interest.insert(sound_id, wakers);
            Poll::Pending
        }
    }

    pub fn drop_sound(&self, sound_id: SoundNotifyId) {
        let mut state = self.state();
        state.dropped_sound_futures.insert(sound_id);
    }

    pub fn cancel_task(&self, task_id: TaskId) {
        let mut _task = None;
        let mut state = self.state();
        _task = state.unready_tasks.remove(&task_id);
        if let Some(ready_index) =
            state
                .ready_tasks
                .iter()
                .enumerate()
                .find_map(|(i, (task_id_ref, _))| {
                    if *task_id_ref == task_id {
                        Some(i)
                    } else {
                        None
                    }
                })
        {
            _task = state.ready_tasks.remove(ready_index).map(|(_, task)| task);
        }
        let _done = state.done_tasks.remove(&task_id);
        state.dropped_handles.remove(&task_id);
        core::mem::drop(state);
    }
    
    pub fn write_str(&self, win: WinId, s: &str) -> core::fmt::Result {
        let state = self.state();
        if let Some(pending) = state.pending_input.get(&win) {
            if pending.line_pending {
                Err(core::fmt::Error)
            } else {
                write_str(win, s)
            }
        } else {
            write_str(win, s)
        }
    }
}

impl<T> JoinHandle<T>
where
    T: 'static,
{
    /// This convenience method allows polling a join handle from a synchronous
    /// context using a no-op waker, without having to pin it or construct a
    /// context. It is useful for collecting a task's exit status after `run`
    /// has returned, at which point it is guaranteed to be ready.
    pub fn poll_nowake(&mut self) -> Poll<T> {
        let waker = futures_task::noop_waker_ref();
        let mut cx = Context::from_waker(waker);
        let pinned = Pin::new(self);
        pinned.poll(&mut cx)
    }

    /// Cancels the task along with any pending futures that it owns.
    /// 
    /// This method must be used with care. Although all futures produced by
    /// `bedquilt-io` are cancel-safe, those produced by third-party crates may
    /// not be.
    pub fn cancel(mut self) {
        GLOBAL_REACTOR.cancel_task(self.task);
        self.cancelled = true;
    }
}

impl<T> Future for JoinHandle<T>
where
    T: 'static,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match GLOBAL_REACTOR.poll_task(cx, self.task) {
            Poll::Ready(result) => Poll::Ready(*(result.downcast::<T>().unwrap())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        if !self.cancelled {
            let mut state = GLOBAL_REACTOR.state();
            if state.done_tasks.remove(&self.task).is_none() {
                state.dropped_handles.insert(self.task);
            }
        }
    }
}

impl Future for CharFuture {
    type Output = CharEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_null() {
            panic!("Poll of completed CharFuture");
        }
        let result = GLOBAL_REACTOR.poll_char(cx, self.0);
        if result.is_ready() {
            self.get_mut().0 = WinId::null();
        }
        result
    }
}

impl Drop for CharFuture {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let mut state = GLOBAL_REACTOR.state();
            if let Some(pending) = state.pending_input.get_mut(&self.0) {
                pending.char_waiters = pending.char_waiters.saturating_sub(1);
                requeue_win_events(&mut state, self.0);
            }
        }
    }
}

impl Future for LineFuture {
    type Output = LineEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_null() {
            panic!("Poll of completed CharFuture");
        }
        let result = GLOBAL_REACTOR.poll_line(cx, self.0);
        if result.is_ready() {
            self.get_mut().0 = WinId::null();
        }
        result
    }
}

impl Drop for LineFuture {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let mut state = GLOBAL_REACTOR.state();
            if let Some(pending) = state.pending_input.get_mut(&self.0) {
                pending.line_waiters = pending.line_waiters.saturating_sub(1);
                requeue_win_events(&mut state, self.0);
            }
        }
    }
}

impl Future for MouseFuture {
    type Output = MouseEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_null() {
            panic!("Poll of completed MouseFuture");
        }
        let result = GLOBAL_REACTOR.poll_mouse(cx, self.0);
        if result.is_ready() {
            self.get_mut().0 = WinId::null();
        }
        result
    }
}

impl Drop for MouseFuture {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let mut state = GLOBAL_REACTOR.state();
            if let Some(pending) = state.pending_input.get_mut(&self.0) {
                pending.mouse_waiters = pending.mouse_waiters.saturating_sub(1);
                requeue_win_events(&mut state, self.0);
            }
        }
    }
}

impl Future for HyperlinkFuture {
    type Output = HyperlinkEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.is_null() {
            panic!("Poll of completed HyperlinkFuture");
        }
        let result = GLOBAL_REACTOR.poll_hyperlink(cx, self.0);
        if result.is_ready() {
            self.get_mut().0 = WinId::null();
        }
        result
    }
}

impl Drop for HyperlinkFuture {
    fn drop(&mut self) {
        if !self.0.is_null() {
            let mut state = GLOBAL_REACTOR.state();
            if let Some(pending) = state.pending_input.get_mut(&self.0) {
                pending.hyperlink_waiters = pending.hyperlink_waiters.saturating_sub(1);
                requeue_win_events(&mut state, self.0);
            }
        }
    }
}

impl Future for RedrawFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        GLOBAL_REACTOR.poll_redraw(cx, self.0)
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        GLOBAL_REACTOR.poll_timer(cx, self.0, self.1)
    }
}

impl Future for SoundFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == 0 {
            panic!("Poll of completed SoundFuture");
        }
        let result = GLOBAL_REACTOR.poll_sound(cx, self.0);
        if result.is_ready() {
            self.0 = 0;
        }
        result
    }
}

impl Drop for SoundFuture {
    fn drop(&mut self) {
        if self.0 != 0 {
            GLOBAL_REACTOR.drop_sound(self.0);
        }
    }
}

impl SoundFuture {
    pub fn cancel(&mut self) {
        self.0 = 0;
    }
}

pub static GLOBAL_REACTOR: Reactor = Reactor(Mutex::new(None));

pub fn add_waker(wakers: &mut Vec<Waker>, cx: &mut Context<'_>) {
    let new_waker = cx.waker();
    for waker in wakers.iter() {
        if waker.will_wake(new_waker) {
            return;
        }
    }
    wakers.push(new_waker.clone());
}

fn requeue_win_events(state: &mut ReactorState, win: WinId) {
    if let Some(pending) = state.pending_input.get_mut(&win) {
        if pending.char_pending && pending.char_waiters == 0 {
            cancel_char(win);
            pending.char_pending = false;
        }

        if pending.line_pending && pending.line_waiters == 0 {
            cancel_line(win);
            pending.line_pending = false;
        }

        if pending.mouse_pending && pending.mouse_waiters == 0 {
            if cancel_mouse(win).is_err() {
                panic!("Error cancelling mouse request");
            }
            pending.mouse_pending = false;
        }

        if pending.hyperlink_pending && pending.hyperlink_waiters == 0 {
            if cancel_hyperlink(win).is_err() {
                panic!("Error cancelling hyperlink request")
            }
            pending.hyperlink_pending = false;
        }

        if !pending.char_pending && !pending.line_pending {
            if pending.char_waiters != 0 {
                if request_char(win).is_err() {
                    panic!("Error enqueueing new char request");
                }
            } else if pending.line_waiters != 0 {
                let initial = pending.line_initial.pop_front().unwrap_or_default();
                if request_line(win, initial.as_str()).is_err() {
                    panic!("Error enqueueing new line request");
                }
            }
        }

        if !pending.mouse_pending && pending.mouse_waiters != 0 && request_mouse(win).is_err() {
            panic!("Error enqueueing new mouse request");
        }

        if !pending.hyperlink_pending
            && pending.hyperlink_waiters != 0
            && request_hyperlink(win).is_err()
        {
            panic!("Error enqueueing new hyperlink request");
        }
    }
}
