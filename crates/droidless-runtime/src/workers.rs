//! Bounded guest workers share one heap and resume managed DEX continuations.
use crate::{
    heap::{Word, fault},
    vm::{Frame, Runtime},
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;
use std::collections::{BTreeMap, VecDeque};

const THREAD_LOCAL: &str = "Ljava/lang/ThreadLocal;";

#[derive(Clone, Copy, Debug)]
pub(crate) enum Waiting {
    Take(Word),
    Put(Word),
    Monitor(Word),
    ObjectWait { object: Word, deadline: Option<u64> },
    Completion { owner: Word, deadline: Option<u64> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::apk::Apk;
    #[test]
    fn monitor_ownership_roots_and_worker_capacity_preserve_state() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap(),
        )
        .unwrap();
        vm.launch().unwrap();
        let lock = vm.heap.instance("Ljava/lang/Object;").unwrap();
        vm.enter_monitor(lock).unwrap();
        vm.enter_monitor(lock).unwrap();
        vm.collect();
        vm.heap.get(lock).unwrap();
        let other = vm.heap.instance("Ljava/lang/Thread;").unwrap();
        vm.workers.current = Some(other);
        let error = vm.exit_monitor(lock).unwrap_err();
        assert!(format!("{error:#}").contains("IllegalMonitorStateException"));
        assert!(
            vm.enter_monitor(lock)
                .unwrap_err()
                .downcast_ref::<Waiting>()
                .is_some()
        );
        vm.workers.current = None;
        vm.exit_monitor(lock).unwrap();
        assert_eq!(vm.workers.monitors[&lock.reference().unwrap()].1, 1);
        vm.exit_monitor(lock).unwrap();
        vm.collect();
        assert!(vm.heap.get(lock).is_err());
        let prepare = Method {
            class: "Lorg/droidless/scheduling/WorkerContract;".into(),
            name: "prepareCapacity".into(),
            parameters: vec![],
            returns: "V".into(),
        };
        let error = vm.invoke(prepare, vec![], false).unwrap_err();
        assert!(format!("{error:#}").contains("guest worker limit"));
        assert_eq!(vm.workers.pending.len(), 64);
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
        vm.poll_messages().unwrap();
        assert!(vm.workers.pending.is_empty());
        assert!(vm.workers.current.is_none());
        vm.close().unwrap();
    }
}
impl std::fmt::Display for Waiting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "guest worker waiting: {self:?}")
    }
}
impl std::error::Error for Waiting {}

pub(crate) struct Worker {
    thread: Word,
    timer: Option<Word>,
    executor: Option<Word>,
    future: Option<Word>,
    task: Option<Word>,
    idle_since: u64,
    entry: Option<(Method, Word)>,
    frames: Vec<Frame>,
    waiting: Option<Waiting>,
}
#[derive(Default)]
pub(crate) struct Workers {
    pub current: Option<Word>,
    pub pending: VecDeque<Worker>,
    pub waiting: Option<Waiting>,
    pub monitors: BTreeMap<usize, (Word, usize)>,
}
impl Workers {
    pub fn roots(&self) -> impl Iterator<Item = Word> + '_ {
        self.current
            .into_iter()
            .chain(
                self.monitors
                    .iter()
                    .flat_map(|(h, (owner, _))| [Word::Ref(*h), *owner]),
            )
            .chain(self.pending.iter().flat_map(|w| {
                [w.thread]
                    .into_iter()
                    .chain(w.timer)
                    .chain(w.executor)
                    .chain(w.future)
                    .chain(w.task)
                    .chain(w.entry.as_ref().map(|(_, receiver)| *receiver))
                    .chain(w.frames.iter().flat_map(Frame::roots))
            }))
    }
}
impl Runtime {
    pub(crate) fn thread_word(&self, thread: Word, name: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(thread)?
            .fields
            .get(name)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    pub(crate) fn current_thread(&mut self) -> Result<Word> {
        match self.workers.current {
            Some(thread) => Ok(thread),
            None => self.main_thread(),
        }
    }
    pub(crate) fn thread_local_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != THREAD_LOCAL {
            return Ok(None);
        }
        let receiver = *args.first().context("ThreadLocal receiver missing")?;
        match method.signature().as_str() {
            "<init>()V" => {
                self.heap.get(receiver)?;
                Ok(Some(vec![]))
            }
            "initialValue()Ljava/lang/Object;" => {
                self.heap.get(receiver)?;
                Ok(Some(vec![Word::ZERO]))
            }
            "get()Ljava/lang/Object;" => {
                let thread = self.current_thread()?;
                let key = format!("droidless:thread-local:{}", receiver.reference()?);
                if let Some(value) = self
                    .heap
                    .get(thread)?
                    .fields
                    .get(&key)
                    .and_then(|values| values.first())
                    .copied()
                {
                    return Ok(Some(vec![value]));
                }
                let value = self
                    .invoke(
                        Method {
                            class: THREAD_LOCAL.into(),
                            name: "initialValue".into(),
                            parameters: vec![],
                            returns: "Ljava/lang/Object;".into(),
                        },
                        vec![receiver],
                        true,
                    )?
                    .first()
                    .copied()
                    .context("ThreadLocal.initialValue returned no value")?;
                self.heap.get_mut(thread)?.fields.insert(key, vec![value]);
                Ok(Some(vec![value]))
            }
            "set(Ljava/lang/Object;)V" => {
                let value = *args.get(1).context("ThreadLocal value missing")?;
                value.reference()?;
                let thread = self.current_thread()?;
                let key = format!("droidless:thread-local:{}", receiver.reference()?);
                self.heap.get_mut(thread)?.fields.insert(key, vec![value]);
                Ok(Some(vec![]))
            }
            "remove()V" => {
                let thread = self.current_thread()?;
                let key = format!("droidless:thread-local:{}", receiver.reference()?);
                self.heap.get_mut(thread)?.fields.remove(&key);
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }
    pub(crate) fn require_main_thread(&self) -> Result<()> {
        ensure!(
            self.workers.current.is_none(),
            "unsupported UI access from a guest worker; post to the main Handler"
        );
        Ok(())
    }
    pub(crate) fn take_interrupt(&mut self) -> Result<bool> {
        let thread = self.current_thread()?;
        let interrupted = self.thread_word(thread, "interrupted")?.truth();
        if interrupted {
            self.heap
                .get_mut(thread)?
                .fields
                .insert("interrupted".into(), vec![Word::ZERO]);
        }
        Ok(interrupted)
    }
    pub(crate) fn check_interrupt(&mut self) -> Result<()> {
        if self.take_interrupt()? {
            return Err(fault(
                "Ljava/lang/InterruptedException;",
                "worker interrupted",
            ));
        }
        Ok(())
    }
    pub(crate) fn wait_worker(&self, waiting: Waiting) -> Result<()> {
        ensure!(
            self.workers.current.is_some(),
            "unsupported blocking wait on the main thread"
        );
        Err(waiting.into())
    }
    pub(crate) fn notify_waiters(&mut self, object: Word, all: bool) -> Result<()> {
        let thread = self.current_thread()?;
        if self
            .workers
            .monitors
            .get(&object.reference()?)
            .is_none_or(|(owner, _)| *owner != thread)
        {
            return Err(fault(
                "Ljava/lang/IllegalMonitorStateException;",
                "current thread does not own this monitor",
            ));
        }
        for worker in &self.workers.pending {
            if matches!(
                worker.waiting,
                Some(Waiting::ObjectWait { object: waiting, .. }) if waiting == object
            ) {
                self.heap
                    .get_mut(worker.thread)?
                    .fields
                    .insert("droidless:wait:notified".into(), vec![Word::from(1)]);
                if !all {
                    break;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn enter_monitor(&mut self, object: Word) -> Result<()> {
        self.heap.get(object)?;
        let thread = self.current_thread()?;
        let handle = object.reference()?;
        match self.workers.monitors.get_mut(&handle) {
            Some((owner, depth)) if *owner == thread => {
                *depth = depth.checked_add(1).context("monitor recursion limit")?;
            }
            Some(_) => return self.wait_worker(Waiting::Monitor(object)),
            None => {
                self.workers.monitors.insert(handle, (thread, 1));
            }
        }
        Ok(())
    }
    pub(crate) fn exit_monitor(&mut self, object: Word) -> Result<()> {
        self.heap.get(object)?;
        let thread = self.current_thread()?;
        let handle = object.reference()?;
        match self.workers.monitors.get_mut(&handle) {
            Some((owner, depth)) if *owner == thread => {
                *depth -= 1;
                if *depth == 0 {
                    self.workers.monitors.remove(&handle);
                }
            }
            _ => {
                return Err(fault(
                    "Ljava/lang/IllegalMonitorStateException;",
                    "monitor is not owned by this thread",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn release_frame_monitors(&mut self, frame: &Frame) -> Result<()> {
        for object in frame.monitors.iter().rev() {
            self.exit_monitor(*object)?;
        }
        Ok(())
    }
    pub(crate) fn start_worker(&mut self, thread: Word) -> Result<()> {
        ensure!(!self.queue.closed, "guest runtime is closed");
        ensure!(
            self.heap.get(thread)?.fields.contains_key("id"),
            "uninitialized Thread"
        );
        if self.thread_word(thread, "started")?.truth() {
            return Err(fault(
                "Ljava/lang/IllegalThreadStateException;",
                "Thread already started",
            ));
        }
        ensure!(
            self.workers.pending.len() + usize::from(self.workers.current.is_some()) < 64,
            "guest worker limit reached (64)"
        );
        let mut class = self.heap.get(thread)?.class.clone();
        let mut receiver = thread;
        // Resolve the APK's Thread.run override; otherwise execute its stored Runnable directly.
        // Avoid keeping a synchronous native Thread.run bridge across a blocking guest call.
        for _ in 0..128 {
            if class == "Ljava/lang/Thread;" {
                receiver = self.thread_word(thread, "target")?;
                class = "Ljava/lang/Runnable;".into();
                break;
            }
            if let Some((d, c)) = self.class_location(&class)
                && self.apk.dex[d].classes[c]
                    .methods
                    .iter()
                    .any(|m| self.apk.dex[d].methods[m.index].signature() == "run()V")
            {
                break;
            }
            class = self.parent(&class).context("Thread has no run hierarchy")?;
        }
        let entry = (receiver != Word::ZERO).then(|| {
            (
                Method {
                    class,
                    name: "run".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                receiver,
            )
        });
        let fields = &mut self.heap.get_mut(thread)?.fields;
        fields.insert("started".into(), vec![Word::from(1)]);
        fields.insert("alive".into(), vec![Word::from(1)]);
        self.workers.pending.push_back(Worker {
            thread,
            timer: None,
            executor: None,
            future: None,
            task: None,
            idle_since: self.uptime_ms(),
            entry,
            frames: vec![],
            waiting: None,
        });
        Ok(())
    }
    pub(crate) fn start_timer_worker(&mut self, thread: Word, timer: Word) -> Result<()> {
        self.start_worker(thread)?;
        self.workers
            .pending
            .back_mut()
            .context("Timer worker missing")?
            .timer = Some(timer);
        Ok(())
    }
    pub(crate) fn start_executor_worker(&mut self, thread: Word, executor: Word) -> Result<()> {
        self.start_worker(thread)?;
        self.workers
            .pending
            .back_mut()
            .context("executor worker missing")?
            .executor = Some(executor);
        Ok(())
    }
    fn worker_ready(&self, worker: &Worker) -> Result<bool> {
        if let Some(timer) = worker.timer
            && worker.frames.is_empty()
            && worker.entry.is_none()
            && worker.waiting.is_none()
        {
            return self.timer_ready(timer);
        }
        if let Some(executor) = worker.executor
            && worker.task.is_none()
            && worker.frames.is_empty()
            && worker.entry.is_none()
            && worker.waiting.is_none()
        {
            return self.executor_ready(executor, worker.idle_since);
        }
        if !matches!(worker.waiting, Some(Waiting::Monitor(_)))
            && self.thread_word(worker.thread, "interrupted")?.truth()
        {
            return Ok(true);
        }
        Ok(match worker.waiting {
            None => true,
            Some(Waiting::Take(queue)) => !self.collection(queue)?.0.is_empty(),
            Some(Waiting::Put(queue)) => {
                self.collection(queue)?.0.len() < self.queue_capacity(queue)? as usize
            }
            Some(Waiting::Monitor(object)) => self
                .workers
                .monitors
                .get(&object.reference()?)
                .is_none_or(|(owner, _)| *owner == worker.thread),
            Some(Waiting::ObjectWait { object, deadline }) => {
                self.thread_word(worker.thread, "droidless:wait:notified")?
                    .truth()
                    || deadline.is_some_and(|deadline| self.uptime_ms() >= deadline)
                    || self.thread_word(worker.thread, "droidless:wait:object")? != object
            }
            Some(Waiting::Completion { owner, deadline }) => {
                self.completion_done(owner)?
                    || deadline.is_some_and(|deadline| self.uptime_ms() >= deadline)
            }
        })
    }
    /// Move the VM's exclusive borrow onto a host worker executor for this poll.
    /// No guest heap copies and no host UI calls occur on that executor.
    pub(crate) fn poll_workers(&mut self, slices: &mut usize) -> Result<usize> {
        ensure!(
            self.frames.is_empty() && self.workers.current.is_none(),
            "reentrant worker dispatch"
        );
        if self.queue.closed || self.workers.pending.is_empty() || *slices == 0 {
            return Ok(0);
        }
        // ponytail: one serial host executor per poll; use a persistent executor if spawn overhead is measured.
        let count = std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("droidless-workers".into())
                .spawn_scoped(scope, || self.drive_workers(*slices))?
                .join()
                .map_err(|_| anyhow::anyhow!("host worker executor panicked"))?
        })?;
        *slices -= count;
        Ok(count)
    }
    fn drive_workers(&mut self, slices: usize) -> Result<usize> {
        let mut count = 0;
        let mut idle = 0;
        // Bounded slices keep a spinning worker from starving native event processing.
        while count < slices && !self.workers.pending.is_empty() {
            let mut worker = self
                .workers
                .pending
                .pop_front()
                .context("worker queue underflow")?;
            let ready = match self.worker_ready(&worker) {
                Ok(ready) => ready,
                Err(error) => {
                    self.workers.pending.push_front(worker);
                    return Err(error);
                }
            };
            if !ready {
                self.workers.pending.push_back(worker);
                idle += 1;
                if idle >= self.workers.pending.len() {
                    break;
                }
                continue;
            }
            idle = 0;
            self.workers.current = Some(worker.thread);
            self.frames = std::mem::take(&mut worker.frames);
            self.workers.waiting = None;
            let mut result = (|| -> Result<Option<Vec<Word>>> {
                if let Some(timer) = worker.timer
                    && self.frames.is_empty()
                    && worker.entry.is_none()
                    && let Some(task) = self.take_timer_task(timer)?
                {
                    worker.entry = Some((
                        Method {
                            class: "Ljava/util/TimerTask;".into(),
                            name: "run".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        task,
                    ));
                }
                if let Some(executor) = worker.executor
                    && worker.task.is_none()
                    && self.frames.is_empty()
                    && worker.entry.is_none()
                    && let Some(task) = self.take_executor_task(executor, worker.thread)?
                {
                    worker.task = Some(task);
                    worker.entry = Some((
                        Method {
                            class: "Ljava/lang/Runnable;".into(),
                            name: "run".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        task,
                    ));
                }
                if let Some((method, receiver)) = worker.entry.take() {
                    let entry = if self.native_future(receiver)? {
                        let entry = self.begin_future_task(receiver, worker.thread)?;
                        if entry.is_some() {
                            worker.future = Some(receiver);
                            self.heap
                                .get_mut(worker.thread)?
                                .fields
                                .insert("droidless:future:owner".into(), vec![receiver]);
                        }
                        entry
                    } else {
                        Some((method, receiver))
                    };
                    if let Some((method, receiver)) = entry
                        && let Some(words) = self.begin_invoke(method, vec![receiver], true)?
                    {
                        return Ok(Some(words));
                    }
                }
                if self.frames.is_empty() {
                    return Ok(Some(vec![]));
                }
                self.execute_slice(0, 1024)
            })();
            count += 1;
            worker.frames = std::mem::take(&mut self.frames);
            worker.waiting = self.workers.waiting.take();
            if !matches!(result, Ok(None))
                && let Some(future) = worker.future.take()
            {
                // done() overrides run on the completing guest worker, before restoring main identity.
                result = self
                    .finish_future_task(future, result.map(|words| words.unwrap_or_default()))
                    .map(|()| Some(vec![]));
                self.heap
                    .get_mut(worker.thread)?
                    .fields
                    .remove("droidless:future:owner");
            }
            self.workers.current = None;
            match result {
                Ok(None) => self.workers.pending.push_back(worker),
                done => {
                    if let Some(timer) = worker.timer
                        && self.finish_timer_task(timer, done.is_err())?
                    {
                        worker.waiting = None;
                        self.workers.pending.push_back(worker);
                        continue;
                    }
                    if let Some(executor) = worker.executor {
                        if worker.task.take().is_some() {
                            worker.idle_since = self.uptime_ms();
                        }
                        if done.is_ok()
                            && self.finish_executor_task(
                                executor,
                                worker.thread,
                                worker.idle_since,
                            )?
                        {
                            worker.waiting = None;
                            self.workers.pending.push_back(worker);
                            continue;
                        }
                    }
                    self.heap
                        .get_mut(worker.thread)?
                        .fields
                        .insert("alive".into(), vec![Word::ZERO]);
                    if let Some(executor) = worker.executor {
                        self.retire_executor_worker(executor, worker.thread)?;
                    }
                    for field in [
                        "droidless:timer:owner",
                        "droidless:executor:owner",
                        "droidless:future:owner",
                    ] {
                        self.heap.get_mut(worker.thread)?.fields.remove(field);
                    }
                    // Terminal host errors bypass guest finally; release all locks of the dead worker.
                    self.workers
                        .monitors
                        .retain(|_, (owner, _)| *owner != worker.thread);
                    done.with_context(|| {
                        format!(
                            "executing guest worker {}",
                            worker.thread.reference().unwrap_or(0)
                        )
                    })?;
                }
            }
        }
        Ok(count)
    }
    pub(crate) fn stop_workers(&mut self) -> Result<()> {
        ensure!(
            self.workers.current.is_none(),
            "cannot stop workers from a worker"
        );
        let workers: Vec<_> = self.workers.pending.drain(..).collect();
        for worker in workers {
            if let Some(timer) = worker.timer {
                self.finish_timer_task(timer, true)?;
            }
            if let Some(executor) = worker.executor {
                self.close_executor(executor)?;
                self.retire_executor_worker(executor, worker.thread)?;
            }
            if let Some(future) = worker
                .future
                .or_else(|| worker.entry.as_ref().map(|(_, receiver)| *receiver))
                && matches!(self.heap.get(future)?.data, crate::heap::Data::Future(_))
            {
                self.close_future(future)?;
            }
            self.heap
                .get_mut(worker.thread)?
                .fields
                .insert("alive".into(), vec![Word::ZERO]);
            for field in [
                "droidless:timer:owner",
                "droidless:executor:owner",
                "droidless:future:owner",
            ] {
                self.heap.get_mut(worker.thread)?.fields.remove(field);
            }
            self.workers
                .monitors
                .retain(|_, (owner, _)| *owner != worker.thread);
        }
        Ok(())
    }
}
