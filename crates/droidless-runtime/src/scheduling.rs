//! Main-thread messages and monotonic time. Guest callbacks always execute in our DEX VM.
use crate::{
    heap::{Data, Word, bits64, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;
use std::{collections::BTreeMap, time::Instant};

const HANDLER: &str = "Landroid/os/Handler;";
const MESSAGE: &str = "Landroid/os/Message;";
const LOOPER: &str = "Landroid/os/Looper;";
const THREAD: &str = "Ljava/lang/Thread;";
const THREAD_GROUP: &str = "Ljava/lang/ThreadGroup;";
const LOCAL_SERVER_SOCKET: &str = "Landroid/net/LocalServerSocket;";
const RUNNABLE: &str = "Ljava/lang/Runnable;";
const LIMIT: usize = 16_384;

#[derive(Default)]
pub(crate) struct MainQueue {
    pub pending: BTreeMap<(u64, u64), Word>,
    pub active: Option<Word>,
    time: u64,
    epoch: Option<Instant>,
    sequence: u64,
    pub closed: bool,
    thread_id: u64,
}
impl Runtime {
    /// Process-relative monotonic milliseconds, deterministic until a native host enables its clock.
    pub fn uptime_ms(&self) -> u64 {
        let elapsed = self
            .queue
            .epoch
            .map_or(0, |epoch| epoch.elapsed().as_millis());
        (u128::from(self.queue.time) + elapsed).min(i64::MAX as u128) as u64
    }
    pub fn use_realtime_clock(&mut self) -> Result<()> {
        ensure!(
            self.frames.is_empty() && self.queue.active.is_none(),
            "clock change during guest execution"
        );
        if self.queue.epoch.is_none() {
            self.queue.epoch = Some(Instant::now());
        }
        Ok(())
    }
    pub fn advance_time(&mut self, milliseconds: u64) -> Result<usize> {
        ensure!(
            self.queue.epoch.is_none(),
            "manual time advance requires the deterministic clock"
        );
        ensure!(
            self.frames.is_empty() && self.queue.active.is_none(),
            "time advance during guest execution"
        );
        let time = self
            .queue
            .time
            .checked_add(milliseconds)
            .filter(|t| *t <= i64::MAX as u64)
            .context("monotonic clock limit reached")?;
        self.queue.time = time;
        self.poll_messages()
    }
    /// Drain due messages in deadline/FIFO order at a host event-loop boundary.
    pub fn poll_messages(&mut self) -> Result<usize> {
        ensure!(
            self.frames.is_empty() && self.queue.active.is_none(),
            "reentrant message dispatch"
        );
        self.reset_budget();
        self.drain_navigation()?;
        let mut worker_slices = 64;
        self.poll_workers(&mut worker_slices)?;
        let mut count = 0;
        while let Some((&key, &message)) = self.queue.pending.first_key_value() {
            if key.0 > self.uptime_ms() || self.queue.closed {
                break;
            }
            ensure!(
                count < 1024,
                "message dispatch limit reached (1024 per poll)"
            );
            let handler = self.message_word(message, "target")?;
            self.queue.pending.remove(&key);
            self.queue.active = Some(message);
            let result = self.invoke(
                Method {
                    class: HANDLER.into(),
                    name: "dispatchMessage".into(),
                    parameters: vec![MESSAGE.into()],
                    returns: "V".into(),
                },
                vec![handler, message],
                true,
            );
            self.queue.active = None;
            self.retire_message(message)?;
            result.with_context(|| format!("dispatching main-thread message at {} ms", key.0))?;
            self.drain_navigation()?;
            count += 1;
            self.poll_workers(&mut worker_slices)?;
        }
        if count > 0 {
            self.collect();
        }
        Ok(count)
    }
    pub(crate) fn stop_messages(&mut self) -> Result<()> {
        self.queue.closed = true;
        self.stop_workers()?;
        if let Some(thread) = self
            .statics
            .get("droidless:mainThread")
            .and_then(|v| v.first())
            .copied()
        {
            self.heap
                .get_mut(thread)?
                .fields
                .insert("alive".into(), vec![Word::ZERO]);
        }
        let messages = std::mem::take(&mut self.queue.pending);
        for message in messages.into_values() {
            self.retire_message(message)?;
        }
        Ok(())
    }
    fn message_word(&self, message: Word, name: &str) -> Result<Word> {
        let key = if name == "obj" {
            format!("{MESSAGE}->obj:Ljava/lang/Object;")
        } else {
            name.into()
        };
        Ok(self
            .heap
            .get(message)?
            .fields
            .get(&key)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    fn retire_message(&mut self, message: Word) -> Result<()> {
        let fields = &mut self.heap.get_mut(message)?.fields;
        fields.clear();
        fields.insert("used".into(), vec![Word::from(1)]);
        Ok(())
    }
    fn new_message(&mut self, handler: Word, callback: Word, token: Word) -> Result<Word> {
        for word in [handler, callback, token] {
            word.reference()?;
        }
        let message = self.heap.instance(MESSAGE)?;
        let fields = &mut self.heap.get_mut(message)?.fields;
        fields.insert("target".into(), vec![handler]);
        fields.insert("callback".into(), vec![callback]);
        fields.insert(format!("{MESSAGE}->obj:Ljava/lang/Object;"), vec![token]);
        Ok(message)
    }
    fn message_time(&self, words: &[Word], delayed: bool) -> Result<u64> {
        let time = (bits64(words)? as i64).max(0) as u64;
        if delayed {
            self.uptime_ms()
                .checked_add(time)
                .filter(|t| *t <= i64::MAX as u64)
                .context("message deadline limit reached")
        } else {
            Ok(time)
        }
    }
    fn enqueue_message(&mut self, handler: Word, message: Word, when: u64) -> Result<bool> {
        ensure!(
            self.is_a(&self.heap.get(message)?.class, MESSAGE),
            "expected Message"
        );
        if self.message_word(message, "used")?.truth() {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "Message already queued or consumed",
            ));
        }
        if self.queue.closed {
            return Ok(false);
        }
        ensure!(
            self.message_word(handler, "looper")? == self.main_looper()?,
            "Handler has no supported Looper"
        );
        ensure!(
            self.queue.pending.len() < LIMIT,
            "message queue limit reached ({LIMIT})"
        );
        let sequence = self
            .queue
            .sequence
            .checked_add(1)
            .context("message sequence exhausted")?;
        let fields = &mut self.heap.get_mut(message)?.fields;
        fields.insert("target".into(), vec![handler]);
        fields.insert("when".into(), wide(when));
        fields.insert("used".into(), vec![Word::from(1)]);
        self.queue.sequence = sequence;
        self.queue.pending.insert((when, sequence), message);
        Ok(true)
    }
    fn main_looper(&mut self) -> Result<Word> {
        if let Some(word) = self
            .statics
            .get("droidless:mainLooper")
            .and_then(|v| v.first())
        {
            return Ok(*word);
        }
        let thread = self.main_thread()?;
        let looper = self.heap.instance(LOOPER)?;
        self.heap
            .get_mut(looper)?
            .fields
            .insert("thread".into(), vec![thread]);
        self.statics
            .insert("droidless:mainLooper".into(), vec![looper]);
        Ok(looper)
    }
    pub(crate) fn main_thread(&mut self) -> Result<Word> {
        if let Some(word) = self
            .statics
            .get("droidless:mainThread")
            .and_then(|v| v.first())
        {
            return Ok(*word);
        }
        let thread = self.heap.instance(THREAD)?;
        let name = self.heap.string("main".into())?;
        let fields = &mut self.heap.get_mut(thread)?.fields;
        fields.insert("name".into(), vec![name]);
        fields.insert("id".into(), wide(1));
        fields.insert("started".into(), vec![Word::from(1)]);
        fields.insert(
            "alive".into(),
            vec![Word::from(i32::from(!self.queue.closed))],
        );
        self.statics
            .insert("droidless:mainThread".into(), vec![thread]);
        self.queue.thread_id = 1;
        Ok(thread)
    }
    pub(crate) fn scheduling_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |n| args.get(n).copied().context("scheduling argument missing");
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let sig = method.signature();
        let mut result = vec![];
        match (method.class.as_str(), sig.as_str()) {
            (LOCAL_SERVER_SOCKET, "<init>(Ljava/lang/String;)V") => {
                let name = arg(1)?;
                self.heap.text(name)?;
                let object = self.heap.get_mut(receiver)?;
                object.data = Data::Collection {
                    values: vec![],
                    version: 0,
                };
                object.fields.insert("name".into(), vec![name]);
                object.fields.insert("closed".into(), vec![Word::ZERO]);
            }
            (LOCAL_SERVER_SOCKET, "accept()Landroid/net/LocalSocket;") => {
                if self.collection(receiver)?.0.is_empty() {
                    if self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("closed")
                        .and_then(|values| values.first())
                        .is_some_and(|value| value.truth())
                    {
                        return Err(fault(
                            "Ljava/net/SocketException;",
                            "LocalServerSocket is closed",
                        ));
                    }
                    self.wait_worker(crate::workers::Waiting::Take(receiver))?;
                }
                let mut pending = self.collection(receiver)?.0.to_vec();
                result.push(pending.remove(0));
                self.change_collection(receiver, pending)?;
            }
            (LOCAL_SERVER_SOCKET, "close()V") | ("Landroid/net/LocalSocket;", "close()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("closed".into(), vec![Word::from(1)]);
            }
            ("Ljava/lang/Object;", "wait()V" | "wait(J)V") => {
                let timeout = if sig == "wait()V" {
                    None
                } else {
                    let millis = bits64(&args[1..])? as i64;
                    if millis < 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "negative wait timeout",
                        ));
                    }
                    (millis > 0).then(|| self.uptime_ms().saturating_add(millis as u64))
                };
                let thread = self.current_thread()?;
                let pending = self.thread_word(thread, "droidless:wait:object")? == receiver;
                if pending {
                    let deadline = self
                        .heap
                        .get(thread)?
                        .fields
                        .get("droidless:wait:deadline")
                        .context("missing Object.wait deadline")?;
                    let deadline = bits64(deadline)?;
                    let timed = self
                        .thread_word(thread, "droidless:wait:hasDeadline")?
                        .truth()
                        && self.uptime_ms() >= deadline;
                    let interrupted = self.thread_word(thread, "interrupted")?.truth();
                    if !self.thread_word(thread, "droidless:wait:notified")?.truth()
                        && !timed
                        && !interrupted
                    {
                        self.wait_worker(crate::workers::Waiting::ObjectWait {
                            object: receiver,
                            deadline: self
                                .thread_word(thread, "droidless:wait:hasDeadline")?
                                .truth()
                                .then_some(deadline),
                        })?;
                        return Ok(Some(vec![]));
                    }
                    self.enter_monitor(receiver)?;
                    let depth = self.thread_word(thread, "droidless:wait:depth")?.int()? as usize;
                    if let Some((owner, monitor_depth)) =
                        self.workers.monitors.get_mut(&receiver.reference()?)
                        && *owner == thread
                    {
                        *monitor_depth = depth.max(1);
                    }
                    let fields = &mut self.heap.get_mut(thread)?.fields;
                    for key in [
                        "droidless:wait:object",
                        "droidless:wait:deadline",
                        "droidless:wait:hasDeadline",
                        "droidless:wait:notified",
                        "droidless:wait:depth",
                    ] {
                        fields.remove(key);
                    }
                    if self.take_interrupt()? {
                        return Err(fault(
                            "Ljava/lang/InterruptedException;",
                            "interrupted while waiting",
                        ));
                    }
                } else {
                    if self.take_interrupt()? {
                        return Err(fault(
                            "Ljava/lang/InterruptedException;",
                            "interrupted while waiting",
                        ));
                    }
                    let handle = receiver.reference()?;
                    let depth = self
                        .workers
                        .monitors
                        .get(&handle)
                        .filter(|(owner, _)| *owner == thread)
                        .map(|(_, depth)| *depth)
                        .ok_or_else(|| {
                            fault(
                                "Ljava/lang/IllegalMonitorStateException;",
                                "current thread does not own this monitor",
                            )
                        })?;
                    self.workers.monitors.remove(&handle);
                    let fields = &mut self.heap.get_mut(thread)?.fields;
                    fields.insert("droidless:wait:object".into(), vec![receiver]);
                    fields.insert("droidless:wait:deadline".into(), wide(timeout.unwrap_or(0)));
                    fields.insert(
                        "droidless:wait:hasDeadline".into(),
                        vec![Word::from(i32::from(timeout.is_some()))],
                    );
                    fields.insert("droidless:wait:notified".into(), vec![Word::ZERO]);
                    fields.insert(
                        "droidless:wait:depth".into(),
                        vec![Word::from(depth.min(i32::MAX as usize) as i32)],
                    );
                    self.wait_worker(crate::workers::Waiting::ObjectWait {
                        object: receiver,
                        deadline: timeout,
                    })?;
                    return Ok(Some(vec![]));
                }
            }
            ("Ljava/lang/Object;", "notify()V" | "notifyAll()V") => {
                self.notify_waiters(receiver, sig == "notifyAll()V")?;
            }
            ("Ljava/lang/System;", "gc()V") => {
                self.collect();
            }
            ("Landroid/os/SystemClock;", "uptimeMillis()J" | "elapsedRealtime()J") => {
                result = wide(self.uptime_ms())
            }
            (LOOPER, "getMainLooper()Landroid/os/Looper;") => result.push(self.main_looper()?),
            (LOOPER, "myLooper()Landroid/os/Looper;") => {
                result.push(match self.workers.current {
                    Some(thread) => self.thread_word(thread, "looper")?,
                    None => self.main_looper()?,
                });
            }
            (LOOPER, "prepare()V") => {
                let thread = self.current_thread()?;
                if self.workers.current.is_none()
                    || self.thread_word(thread, "looper")? != Word::ZERO
                {
                    return Err(fault(
                        "Ljava/lang/RuntimeException;",
                        "Only one Looper may be created per thread",
                    ));
                }
                let looper = self.heap.instance(LOOPER)?;
                self.heap
                    .get_mut(looper)?
                    .fields
                    .insert("thread".into(), vec![thread]);
                self.heap
                    .get_mut(thread)?
                    .fields
                    .insert("looper".into(), vec![looper]);
            }
            (LOOPER, "getThread()Ljava/lang/Thread;") => {
                result.push(self.message_word(receiver, "thread")?)
            }
            (LOOPER, "quit()V" | "quitSafely()V") => {
                ensure!(
                    receiver == self.main_looper()?,
                    "unsupported worker Looper quit/delivery"
                );
                return Err(fault(
                    "Ljava/lang/IllegalStateException;",
                    "the main Looper cannot quit",
                ));
            }
            (THREAD, "currentThread()Ljava/lang/Thread;") => result.push(self.current_thread()?),
            (THREAD_GROUP, "<init>(Ljava/lang/String;)V") => {
                let name = arg(1)?;
                self.heap.text(name)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("name".into(), vec![name]);
            }
            (
                THREAD,
                "<init>()V"
                | "<init>(Ljava/lang/Runnable;)V"
                | "<init>(Ljava/lang/String;)V"
                | "<init>(Ljava/lang/Runnable;Ljava/lang/String;)V"
                | "<init>(Ljava/lang/ThreadGroup;Ljava/lang/Runnable;Ljava/lang/String;)V",
            ) => {
                self.main_thread()?;
                let target = match method.parameters.as_slice() {
                    [parameter] if parameter == "Ljava/lang/Runnable;" => arg(1)?,
                    [runnable, _] if runnable == "Ljava/lang/Runnable;" => arg(1)?,
                    [group, runnable, _]
                        if group == THREAD_GROUP && runnable == "Ljava/lang/Runnable;" =>
                    {
                        let group = arg(1)?;
                        ensure!(
                            self.is_a(&self.heap.get(group)?.class, THREAD_GROUP),
                            "expected ThreadGroup"
                        );
                        arg(2)?
                    }
                    _ => Word::ZERO,
                };
                if target != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(target)?.class, RUNNABLE),
                        "expected Runnable"
                    );
                }
                let id = self
                    .queue
                    .thread_id
                    .checked_add(1)
                    .filter(|id| *id <= i64::MAX as u64)
                    .context("thread ID limit reached")?;
                let name = match method.parameters.as_slice() {
                    [parameter] if parameter == "Ljava/lang/String;" => arg(1)?,
                    [runnable, _] if runnable == "Ljava/lang/Runnable;" => arg(2)?,
                    [group, runnable, _]
                        if group == THREAD_GROUP && runnable == "Ljava/lang/Runnable;" =>
                    {
                        arg(3)?
                    }
                    _ => self.heap.string(format!("Thread-{}", id - 1))?,
                };
                self.heap.text(name)?;
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("name".into(), vec![name]);
                fields.insert("target".into(), vec![target]);
                fields.insert("id".into(), wide(id));
                fields.insert("alive".into(), vec![Word::ZERO]);
                fields.insert("started".into(), vec![Word::ZERO]);
                self.queue.thread_id = id;
            }
            (THREAD, "getName()Ljava/lang/String;" | "getId()J" | "isAlive()Z") => {
                let key = match method.name.as_str() {
                    "getName" => "name",
                    "getId" => "id",
                    _ => "alive",
                };
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(key)
                    .context("uninitialized Thread")?
                    .clone();
            }
            (THREAD, "setName(Ljava/lang/String;)V") => {
                let name = arg(1)?;
                self.heap.text(name)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("name".into(), vec![name]);
            }
            (THREAD, "run()V") => {
                let target = self.message_word(receiver, "target")?;
                if target != Word::ZERO {
                    self.invoke(
                        Method {
                            class: RUNNABLE.into(),
                            name: "run".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![target],
                        true,
                    )?;
                }
            }
            (THREAD, "start()V") => {
                self.start_worker(receiver)?;
            }
            (THREAD, "interrupt()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("interrupted".into(), vec![Word::from(1)]);
            }
            (THREAD, "isInterrupted()Z") => result.push(self.thread_word(receiver, "interrupted")?),
            (THREAD, "interrupted()Z") => {
                result.push(Word::from(i32::from(self.take_interrupt()?)))
            }
            (THREAD, "holdsLock(Ljava/lang/Object;)Z") => {
                let object = arg(0)?;
                self.heap.get(object)?;
                let thread = self.current_thread()?;
                result.push(Word::from(i32::from(
                    self.workers
                        .monitors
                        .get(&object.reference()?)
                        .is_some_and(|(owner, _)| *owner == thread),
                )));
            }
            (MESSAGE, "<init>()V") => {
                self.heap.get(receiver)?;
            }
            (MESSAGE, "obtain()Landroid/os/Message;") => {
                result.push(self.new_message(Word::ZERO, Word::ZERO, Word::ZERO)?)
            }
            (MESSAGE, "getTarget()Landroid/os/Handler;") => {
                result.push(self.message_word(receiver, "target")?)
            }
            (MESSAGE, "getCallback()Ljava/lang/Runnable;") => {
                result.push(self.message_word(receiver, "callback")?)
            }
            (MESSAGE, "getWhen()J") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("when")
                    .cloned()
                    .unwrap_or_else(|| wide(0))
            }
            (MESSAGE, "setTarget(Landroid/os/Handler;)V") => {
                arg(1)?.reference()?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("target".into(), vec![arg(1)?]);
            }
            (
                HANDLER,
                "<init>()V"
                | "<init>(Landroid/os/Looper;)V"
                | "<init>(Landroid/os/Handler$Callback;)V"
                | "<init>(Landroid/os/Looper;Landroid/os/Handler$Callback;)V",
            ) => {
                let looper = self.main_looper()?;
                if method.parameters.first().is_some_and(|p| p == LOOPER) {
                    self.heap.get(arg(1)?)?;
                    ensure!(arg(1)? == looper, "only the main Looper is supported");
                } else {
                    ensure!(
                        self.workers.current.is_none(),
                        "unsupported implicit worker Handler; use the main Looper explicitly"
                    );
                }
                let callback = if method
                    .parameters
                    .last()
                    .is_some_and(|p| p == "Landroid/os/Handler$Callback;")
                {
                    *args.last().context("missing Handler callback")?
                } else {
                    Word::ZERO
                };
                if callback != Word::ZERO {
                    ensure!(
                        self.is_a(
                            &self.heap.get(callback)?.class,
                            "Landroid/os/Handler$Callback;"
                        ),
                        "expected Handler.Callback"
                    );
                }
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("looper".into(), vec![looper]);
                fields.insert("handlerCallback".into(), vec![callback]);
            }
            (HANDLER, "getLooper()Landroid/os/Looper;") => {
                result.push(self.message_word(receiver, "looper")?)
            }
            (
                HANDLER,
                "post(Ljava/lang/Runnable;)Z"
                | "postDelayed(Ljava/lang/Runnable;J)Z"
                | "postAtTime(Ljava/lang/Runnable;J)Z"
                | "postAtTime(Ljava/lang/Runnable;Ljava/lang/Object;J)Z",
            ) => {
                let callback = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(callback)?.class, RUNNABLE),
                    "expected Runnable"
                );
                let token = if method.parameters.len() == 3 {
                    arg(2)?
                } else {
                    Word::ZERO
                };
                let when = if method.parameters.len() > 1 {
                    let index = if method.parameters.len() == 3 { 3 } else { 2 };
                    self.message_time(
                        args.get(index..index + 2).context("missing time words")?,
                        method.name == "postDelayed",
                    )?
                } else {
                    self.uptime_ms()
                };
                let message = self.new_message(receiver, callback, token)?;
                result.push(Word::from(i32::from(
                    self.enqueue_message(receiver, message, when)?,
                )));
            }
            (
                HANDLER,
                "obtainMessage()Landroid/os/Message;" | "obtainMessage(I)Landroid/os/Message;",
            ) => {
                let message = self.new_message(receiver, Word::ZERO, Word::ZERO)?;
                if !method.parameters.is_empty() {
                    arg(1)?.int()?;
                    self.heap
                        .get_mut(message)?
                        .fields
                        .insert(format!("{MESSAGE}->what:I"), vec![arg(1)?]);
                }
                result.push(message);
            }
            (
                HANDLER,
                "sendMessage(Landroid/os/Message;)Z"
                | "sendMessageDelayed(Landroid/os/Message;J)Z"
                | "sendMessageAtTime(Landroid/os/Message;J)Z"
                | "sendEmptyMessage(I)Z"
                | "sendEmptyMessageDelayed(IJ)Z"
                | "sendEmptyMessageAtTime(IJ)Z",
            ) => {
                let when = if method.parameters.len() == 1 {
                    self.uptime_ms()
                } else {
                    self.message_time(
                        args.get(2..4).context("missing time words")?,
                        method.name.ends_with("Delayed"),
                    )?
                };
                let message = if method.name.starts_with("sendEmptyMessage") {
                    let message = self.new_message(receiver, Word::ZERO, Word::ZERO)?;
                    self.heap
                        .get_mut(message)?
                        .fields
                        .insert(format!("{MESSAGE}->what:I"), vec![arg(1)?]);
                    message
                } else {
                    arg(1)?
                };
                result.push(Word::from(i32::from(
                    self.enqueue_message(receiver, message, when)?,
                )));
            }
            (
                HANDLER,
                "hasCallbacks(Ljava/lang/Runnable;)Z"
                | "removeCallbacks(Ljava/lang/Runnable;)V"
                | "removeCallbacks(Ljava/lang/Runnable;Ljava/lang/Object;)V"
                | "removeCallbacksAndMessages(Ljava/lang/Object;)V",
            ) => {
                let callback = if method.name == "removeCallbacksAndMessages" {
                    None
                } else {
                    Some(arg(1)?)
                };
                let token = if method.name == "removeCallbacksAndMessages" {
                    arg(1)?
                } else if method.parameters.len() == 2 {
                    arg(2)?
                } else {
                    Word::ZERO
                };
                if let Some(word) = callback {
                    word.reference()?;
                }
                token.reference()?;
                let mut found = vec![];
                for (key, message) in &self.queue.pending {
                    let matches_callback = match callback {
                        Some(callback) => self.message_word(*message, "callback")? == callback,
                        None => true,
                    };
                    if self.message_word(*message, "target")? == receiver
                        && matches_callback
                        && (token == Word::ZERO || self.message_word(*message, "obj")? == token)
                    {
                        found.push(*key);
                    }
                }
                if method.name == "hasCallbacks" {
                    result.push(Word::from(i32::from(!found.is_empty())));
                } else {
                    for key in found {
                        let message = self
                            .queue
                            .pending
                            .remove(&key)
                            .context("message disappeared")?;
                        self.retire_message(message)?;
                    }
                }
            }
            (
                HANDLER,
                "removeMessages(I)V"
                | "removeMessages(ILjava/lang/Object;)V"
                | "hasMessages(I)Z"
                | "hasMessages(ILjava/lang/Object;)Z",
            ) => {
                let what = arg(1)?;
                let token = if method.parameters.len() == 2 {
                    arg(2)?
                } else {
                    Word::ZERO
                };
                let mut found = vec![];
                for (key, message) in &self.queue.pending {
                    if self.message_word(*message, "target")? == receiver
                        && self.message_word(*message, "Landroid/os/Message;->what:I")? == what
                        && (token == Word::ZERO || self.message_word(*message, "obj")? == token)
                    {
                        found.push(*key);
                    }
                }
                if method.name == "hasMessages" {
                    result.push(Word::from(i32::from(!found.is_empty())));
                    return Ok(Some(result));
                }
                for key in found {
                    let message = self
                        .queue
                        .pending
                        .remove(&key)
                        .context("message disappeared")?;
                    self.retire_message(message)?;
                }
            }
            (HANDLER, "dispatchMessage(Landroid/os/Message;)V") => {
                let message = arg(1)?;
                let callback = self.message_word(message, "callback")?;
                if callback != Word::ZERO {
                    self.invoke(
                        Method {
                            class: RUNNABLE.into(),
                            name: "run".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![callback],
                        true,
                    )?;
                } else {
                    let callback = self.message_word(receiver, "handlerCallback")?;
                    let handled = if callback == Word::ZERO {
                        false
                    } else {
                        let result = self.invoke(
                            Method {
                                class: "Landroid/os/Handler$Callback;".into(),
                                name: "handleMessage".into(),
                                parameters: vec![MESSAGE.into()],
                                returns: "Z".into(),
                            },
                            vec![callback, message],
                            true,
                        )?;
                        result
                            .first()
                            .context("Handler.Callback returned no value")?
                            .int()?
                            != 0
                    };
                    if !handled {
                        self.invoke(
                            Method {
                                class: HANDLER.into(),
                                name: "handleMessage".into(),
                                parameters: vec![MESSAGE.into()],
                                returns: "V".into(),
                            },
                            vec![receiver, message],
                            true,
                        )?;
                    }
                }
            }
            (HANDLER, "handleMessage(Landroid/os/Message;)V") => {
                self.heap.get(arg(1)?)?;
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::apk::Apk;

    #[test]
    fn queue_capacity_and_deadline_limits_preserve_existing_messages() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap(),
        )
        .unwrap();
        vm.launch().unwrap();
        let prepare = Method {
            class: "Lorg/droidless/scheduling/MainActivity;".into(),
            name: "prepareCapacity".into(),
            parameters: vec![],
            returns: "V".into(),
        };
        assert!(
            format!("{:#}", vm.invoke(prepare, vec![], false).unwrap_err())
                .contains("message queue limit")
        );
        assert_eq!(vm.queue.pending.len(), 16_384);
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
        let before = vm.queue.pending.clone();
        assert_eq!(vm.advance_time(1).unwrap(), 0);
        let (&key, &message) = before.first_key_value().unwrap();
        assert_eq!(key.0, 100_000);
        let handler = vm.message_word(message, "target").unwrap();
        let callback = vm.message_word(message, "callback").unwrap();
        let post = Method {
            class: HANDLER.into(),
            name: "postDelayed".into(),
            parameters: vec![RUNNABLE.into(), "J".into()],
            returns: "Z".into(),
        };
        let mut args = vec![handler, callback];
        args.extend(wide(i64::MAX as u64));
        assert!(
            format!("{:#}", vm.invoke(post, args, true).unwrap_err()).contains("deadline limit")
        );
        assert_eq!(vm.queue.pending, before);
        assert_eq!(vm.queue.sequence, 16_384);
        vm.close().unwrap();
        assert!(vm.queue.pending.is_empty());
    }
}
