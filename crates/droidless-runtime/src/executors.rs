//! Queued executor tasks and cancellable results share the managed worker engine.
use crate::{
    heap::{Data, TimeUnit, Word, bits64, fault},
    interpreter::Thrown,
    vm::Runtime,
    workers::Waiting,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;
use std::collections::{BTreeMap, VecDeque};

const EXECUTORS: &str = "Ljava/util/concurrent/Executors;";
const POOL: &str = "Ljava/util/concurrent/ThreadPoolExecutor;";
const SERVICE: &str = "Ljava/util/concurrent/ExecutorService;";
const EXECUTOR: &str = "Ljava/util/concurrent/Executor;";
const FUTURE: &str = "Ljava/util/concurrent/Future;";
const TASK: &str = "Ljava/util/concurrent/FutureTask;";
const RUNNABLE_FUTURE: &str = "Ljava/util/concurrent/RunnableFuture;";
const RUNNABLE: &str = "Ljava/lang/Runnable;";
const CALLABLE: &str = "Ljava/util/concurrent/Callable;";
const THREAD: &str = "Ljava/lang/Thread;";
const REJECTED: &str = "Ljava/util/concurrent/RejectedExecutionException;";
const LIMIT: usize = 16_384;
const WAIT_OWNER: &str = "droidless:completion:owner";
const WAIT_DEADLINE: &str = "droidless:completion:deadline";

#[derive(Clone, Debug)]
pub struct Executor {
    tasks: VecDeque<Word>,
    workers: BTreeMap<usize, Option<Word>>,
    maximum: usize,
    cached: bool,
    shutdown: bool,
    stopping: bool,
    id: u64,
    sequence: u64,
}
impl Executor {
    pub(crate) fn roots(&self) -> impl Iterator<Item = Word> + '_ {
        self.tasks
            .iter()
            .copied()
            .chain(self.workers.keys().copied().map(Word::Ref))
            .chain(self.workers.values().filter_map(|task| *task))
    }
}
#[derive(Clone, Debug)]
enum Outcome {
    New,
    Running,
    Value(Word),
    Failure(Word),
    Cancelled,
    Aborted(String),
}
#[derive(Clone, Debug)]
pub struct Future {
    body: Word,
    callable: bool,
    preset: Word,
    runner: Option<Word>,
    outcome: Outcome,
}
impl Future {
    pub(crate) fn roots(&self) -> impl Iterator<Item = Word> + '_ {
        [self.body, self.preset]
            .into_iter()
            .chain(self.runner)
            .chain(match self.outcome {
                Outcome::Value(word) | Outcome::Failure(word) => Some(word),
                _ => None,
            })
    }
    fn done(&self) -> bool {
        !matches!(self.outcome, Outcome::New | Outcome::Running)
    }
}
fn method(class: &str, name: &str, returns: &str) -> Method {
    Method {
        class: class.into(),
        name: name.into(),
        parameters: vec![],
        returns: returns.into(),
    }
}
impl Runtime {
    fn executor(&self, owner: Word) -> Result<&Executor> {
        match &self.heap.get(owner)?.data {
            Data::Executor(pool) => Ok(pool),
            _ => bail!("uninitialized ExecutorService"),
        }
    }
    fn executor_mut(&mut self, owner: Word) -> Result<&mut Executor> {
        match &mut self.heap.get_mut(owner)?.data {
            Data::Executor(pool) => Ok(pool),
            _ => bail!("uninitialized ExecutorService"),
        }
    }
    fn future(&self, owner: Word) -> Result<&Future> {
        match &self.heap.get(owner)?.data {
            Data::Future(future) => Ok(future),
            _ => bail!("uninitialized FutureTask"),
        }
    }
    fn future_mut(&mut self, owner: Word) -> Result<&mut Future> {
        match &mut self.heap.get_mut(owner)?.data {
            Data::Future(future) => Ok(future),
            _ => bail!("uninitialized FutureTask"),
        }
    }
    /// Preserve APK run() overrides; only unwrap the native FutureTask continuation.
    pub(crate) fn native_future(&self, task: Word) -> Result<bool> {
        let mut class = self.heap.get(task)?.class.clone();
        if !self.is_a(&class, TASK) {
            return Ok(false);
        }
        for _ in 0..128 {
            if class == TASK {
                self.future(task)?;
                return Ok(true);
            }
            if let Some((d, c)) = self.class_location(&class)
                && self.apk.dex[d].classes[c]
                    .methods
                    .iter()
                    .any(|m| self.apk.dex[d].methods[m.index].signature() == "run()V")
            {
                return Ok(false);
            }
            class = self
                .parent(&class)
                .context("FutureTask has no run hierarchy")?;
        }
        bail!("FutureTask hierarchy limit")
    }
    fn init_future(&mut self, owner: Word, body: Word, callable: bool, preset: Word) -> Result<()> {
        if body == Word::ZERO {
            return Err(fault(
                "Ljava/lang/NullPointerException;",
                "null FutureTask body",
            ));
        }
        let interface = if callable { CALLABLE } else { RUNNABLE };
        ensure!(
            self.is_a(&self.heap.get(body)?.class, interface),
            "FutureTask requires {interface}"
        );
        preset.reference()?;
        self.heap.get_mut(owner)?.data = Data::Future(Future {
            body,
            callable,
            preset,
            runner: None,
            outcome: Outcome::New,
        });
        Ok(())
    }
    pub(crate) fn begin_future_task(
        &mut self,
        owner: Word,
        thread: Word,
    ) -> Result<Option<(Method, Word)>> {
        let future = self.future_mut(owner)?;
        if !matches!(future.outcome, Outcome::New) {
            return Ok(None);
        }
        future.outcome = Outcome::Running;
        future.runner = Some(thread);
        Ok(Some((
            if future.callable {
                method(CALLABLE, "call", "Ljava/lang/Object;")
            } else {
                method(RUNNABLE, "run", "V")
            },
            future.body,
        )))
    }
    fn future_done_callback(&mut self, owner: Word) -> Result<()> {
        self.invoke(method(TASK, "done", "V"), vec![owner], true)?;
        Ok(())
    }
    pub(crate) fn finish_future_task(
        &mut self,
        owner: Word,
        result: Result<Vec<Word>>,
    ) -> Result<()> {
        let result = result.and_then(|words| {
            if self.future(owner)?.callable && !self.future(owner)?.done() {
                ensure!(words.len() == 1, "Callable returned invalid result width");
                words[0].reference()?;
            }
            Ok(words)
        });
        let outcome = match result {
            Ok(words) => {
                let future = self.future(owner)?;
                if future.done() {
                    self.future_mut(owner)?.runner = None;
                    return Ok(());
                }
                let value = if future.callable {
                    words[0]
                } else {
                    future.preset
                };
                Outcome::Value(value)
            }
            Err(error) => {
                if let Some(thrown) = error.downcast_ref::<Thrown>() {
                    if self.future(owner)?.done() {
                        self.future_mut(owner)?.runner = None;
                        return Ok(());
                    }
                    Outcome::Failure(thrown.0)
                } else {
                    let future = self.future_mut(owner)?;
                    if !future.done() {
                        future.outcome = Outcome::Aborted(format!("{error:#}"));
                    }
                    future.body = Word::ZERO;
                    future.preset = Word::ZERO;
                    future.runner = None;
                    return Err(error);
                }
            }
        };
        let future = self.future_mut(owner)?;
        future.outcome = outcome;
        future.body = Word::ZERO;
        future.preset = Word::ZERO;
        future.runner = None;
        self.future_done_callback(owner)
    }
    fn cancel_future(&mut self, owner: Word, interrupt: bool, callback: bool) -> Result<bool> {
        let future = self.future_mut(owner)?;
        if future.done() {
            return Ok(false);
        }
        let runner = future.runner;
        future.outcome = Outcome::Cancelled;
        future.body = Word::ZERO;
        future.preset = Word::ZERO;
        if interrupt && let Some(thread) = runner {
            self.heap
                .get_mut(thread)?
                .fields
                .insert("interrupted".into(), vec![Word::from(1)]);
        }
        if callback {
            self.future_done_callback(owner)?;
        }
        Ok(true)
    }
    fn new_executor(&mut self, maximum: usize, cached: bool) -> Result<Word> {
        ensure!(maximum <= 64, "executor worker limit reached (64)");
        let id = self
            .queue
            .executor_id
            .checked_add(1)
            .context("executor name sequence exhausted")?;
        let owner = self.heap.instance(POOL)?;
        self.heap.get_mut(owner)?.data = Data::Executor(Executor {
            tasks: VecDeque::new(),
            workers: BTreeMap::new(),
            maximum,
            cached,
            shutdown: false,
            stopping: false,
            id,
            sequence: 0,
        });
        self.queue.executor_id = id;
        Ok(owner)
    }
    fn spawn_executor_worker(&mut self, owner: Word) -> Result<()> {
        let pool = self.executor(owner)?;
        let sequence = pool
            .sequence
            .checked_add(1)
            .context("executor thread sequence exhausted")?;
        let name = self
            .heap
            .string(format!("pool-{}-thread-{sequence}", pool.id))?;
        let thread = self.heap.instance(THREAD)?;
        let roots = self.native_roots.len();
        self.native_roots.extend([owner, thread, name]);
        let result = (|| -> Result<()> {
            self.invoke(
                Method {
                    class: THREAD.into(),
                    name: "<init>".into(),
                    parameters: vec!["Ljava/lang/String;".into()],
                    returns: "V".into(),
                },
                vec![thread, name],
                false,
            )?;
            self.heap
                .get_mut(thread)?
                .fields
                .insert("droidless:executor:owner".into(), vec![owner]);
            self.heap
                .get_mut(thread)?
                .fields
                .insert("daemon".into(), vec![Word::ZERO]);
            self.start_executor_worker(thread, owner)?;
            let pool = self.executor_mut(owner)?;
            pool.sequence = sequence;
            pool.workers.insert(thread.reference()?, None);
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    fn enqueue_executor(&mut self, owner: Word, task: Word) -> Result<()> {
        if task == Word::ZERO {
            return Err(fault(
                "Ljava/lang/NullPointerException;",
                "null executor task",
            ));
        }
        ensure!(
            self.is_a(&self.heap.get(task)?.class, RUNNABLE),
            "Executor requires Runnable"
        );
        let pool = self.executor(owner)?;
        if pool.shutdown || self.queue.closed {
            return Err(fault(REJECTED, "executor is shut down"));
        }
        if pool.tasks.len() >= LIMIT {
            return Err(fault(REJECTED, "executor queue limit reached (16384)"));
        }
        let idle = pool.workers.values().filter(|task| task.is_none()).count();
        let spawn = pool.tasks.len() >= idle && pool.workers.len() < pool.maximum;
        if pool.cached && pool.tasks.len() >= idle && !spawn {
            return Err(fault(REJECTED, "cached executor worker limit reached (64)"));
        }
        if spawn {
            if self.workers.pending.len() + usize::from(self.workers.current.is_some()) >= 64 {
                return Err(fault(REJECTED, "guest worker limit reached (64)"));
            }
            self.spawn_executor_worker(owner)?;
        }
        self.executor_mut(owner)?.tasks.push_back(task);
        Ok(())
    }
    pub(crate) fn executor_ready(&self, owner: Word, idle_since: u64) -> Result<bool> {
        let pool = self.executor(owner)?;
        Ok(!pool.tasks.is_empty()
            || pool.shutdown
            || (pool.cached && self.uptime_ms().saturating_sub(idle_since) >= 60_000))
    }
    pub(crate) fn take_executor_task(&mut self, owner: Word, thread: Word) -> Result<Option<Word>> {
        let task = self.executor_mut(owner)?.tasks.pop_front();
        self.executor_mut(owner)?
            .workers
            .insert(thread.reference()?, task);
        if !self.executor(owner)?.stopping {
            self.heap
                .get_mut(thread)?
                .fields
                .insert("interrupted".into(), vec![Word::ZERO]);
        }
        Ok(task)
    }
    pub(crate) fn finish_executor_task(
        &mut self,
        owner: Word,
        thread: Word,
        idle_since: u64,
    ) -> Result<bool> {
        self.executor_mut(owner)?
            .workers
            .insert(thread.reference()?, None);
        let pool = self.executor(owner)?;
        Ok(!pool.tasks.is_empty()
            || (!pool.shutdown
                && (!pool.cached || self.uptime_ms().saturating_sub(idle_since) < 60_000)))
    }
    pub(crate) fn retire_executor_worker(&mut self, owner: Word, thread: Word) -> Result<()> {
        self.executor_mut(owner)?
            .workers
            .remove(&thread.reference()?);
        let pool = self.executor(owner)?;
        if !pool.stopping && !pool.tasks.is_empty() && pool.workers.is_empty() {
            self.spawn_executor_worker(owner)?;
        }
        Ok(())
    }
    fn shutdown_executor(&mut self, owner: Word, immediate: bool) -> Result<Vec<Word>> {
        let pool = self.executor_mut(owner)?;
        pool.shutdown = true;
        if !immediate {
            return Ok(vec![]);
        }
        pool.stopping = true;
        let pending = pool.tasks.drain(..).collect();
        let threads: Vec<_> = pool.workers.keys().copied().map(Word::Ref).collect();
        for thread in threads {
            self.heap
                .get_mut(thread)?
                .fields
                .insert("interrupted".into(), vec![Word::from(1)]);
        }
        Ok(pending)
    }
    pub(crate) fn close_executor(&mut self, owner: Word) -> Result<()> {
        let pending = self.shutdown_executor(owner, true)?;
        for task in pending {
            if matches!(self.heap.get(task)?.data, Data::Future(_)) {
                self.cancel_future(task, false, false)?;
            }
        }
        Ok(())
    }
    pub(crate) fn close_future(&mut self, owner: Word) -> Result<()> {
        self.cancel_future(owner, false, false)?;
        self.future_mut(owner)?.runner = None;
        Ok(())
    }
    pub(crate) fn completion_done(&self, owner: Word) -> Result<bool> {
        match &self.heap.get(owner)?.data {
            Data::Future(future) => Ok(future.done()),
            Data::Executor(pool) => Ok((pool.shutdown || self.queue.closed)
                && pool.workers.is_empty()
                && pool.tasks.is_empty()),
            _ => bail!("invalid completion wait owner"),
        }
    }
    fn timeout_ms(&self, words: &[Word], unit: Word) -> Result<u64> {
        let Data::TimeUnit(unit) = self.heap.get(unit)?.data else {
            bail!("timeout requires TimeUnit");
        };
        let nanos = unit
            .convert(TimeUnit::Nanoseconds, bits64(words)? as i64)
            .max(0) as u64;
        // ponytail: waits use the runtime's millisecond clock; sub-ms waits round up.
        Ok(nanos.div_ceil(1_000_000))
    }
    fn clear_completion_wait(&mut self, owner: Word) -> Result<()> {
        let thread = self.current_thread()?;
        if self.thread_word(thread, WAIT_OWNER)? == owner {
            let fields = &mut self.heap.get_mut(thread)?.fields;
            fields.remove(WAIT_OWNER);
            fields.remove(WAIT_DEADLINE);
        }
        Ok(())
    }
    fn wait_completion(&mut self, owner: Word, duration: Option<u64>) -> Result<bool> {
        if let Err(error) = self.check_interrupt() {
            self.clear_completion_wait(owner)?;
            return Err(error);
        }
        let thread = self.current_thread()?;
        let deadline = if self.thread_word(thread, WAIT_OWNER)? == owner {
            self.heap
                .get(thread)?
                .fields
                .get(WAIT_DEADLINE)
                .map(|words| bits64(words))
                .transpose()?
        } else {
            duration
                .map(|duration| {
                    self.uptime_ms()
                        .checked_add(duration)
                        .filter(|time| *time <= i64::MAX as u64)
                        .context("completion deadline limit reached")
                })
                .transpose()?
        };
        if deadline.is_some_and(|time| self.uptime_ms() >= time) {
            self.clear_completion_wait(owner)?;
            return Ok(false);
        }
        let fields = &mut self.heap.get_mut(thread)?.fields;
        fields.insert(WAIT_OWNER.into(), vec![owner]);
        if let Some(deadline) = deadline {
            fields.insert(WAIT_DEADLINE.into(), crate::heap::wide(deadline));
        }
        let result = self.wait_worker(Waiting::Completion { owner, deadline });
        if result
            .as_ref()
            .is_err_and(|e| e.downcast_ref::<Waiting>().is_none())
        {
            self.clear_completion_wait(owner)?;
        }
        result?;
        unreachable!("completion wait must suspend")
    }
    fn get_future(&mut self, owner: Word, duration: Option<u64>) -> Result<Vec<Word>> {
        let outcome = self.future(owner)?.outcome.clone();
        match outcome {
            Outcome::Value(value) => {
                self.clear_completion_wait(owner)?;
                Ok(vec![value])
            }
            Outcome::Failure(cause) => {
                self.clear_completion_wait(owner)?;
                let text = self.invoke(
                    method("Ljava/lang/Throwable;", "toString", "Ljava/lang/String;"),
                    vec![cause],
                    true,
                )?;
                let message = self
                    .heap
                    .text(
                        *text
                            .first()
                            .context("Throwable.toString returned no value")?,
                    )?
                    .to_owned();
                Err(self.guest_exception(
                    "Ljava/util/concurrent/ExecutionException;",
                    message,
                    Some(cause),
                )?)
            }
            Outcome::Cancelled => {
                self.clear_completion_wait(owner)?;
                Err(fault(
                    "Ljava/util/concurrent/CancellationException;",
                    "Future cancelled",
                ))
            }
            Outcome::Aborted(diagnostic) => {
                self.clear_completion_wait(owner)?;
                bail!("Future task aborted: {diagnostic}")
            }
            Outcome::New | Outcome::Running => {
                self.wait_completion(owner, duration)?;
                Err(fault(
                    "Ljava/util/concurrent/TimeoutException;",
                    "Future wait timed out",
                ))
            }
        }
    }
    pub(crate) fn executor_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if ![
            EXECUTORS,
            POOL,
            SERVICE,
            EXECUTOR,
            FUTURE,
            TASK,
            RUNNABLE_FUTURE,
        ]
        .contains(&method.class.as_str())
        {
            return Ok(None);
        }
        let roots = self.native_roots.len();
        self.native_roots.extend_from_slice(args);
        let result = self.executor_call(method, args);
        self.native_roots.truncate(roots);
        result
    }
    fn executor_call(&mut self, call: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        let signature = call.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let arg = |n| args.get(n).copied().context("executor argument missing");
        if call.class == EXECUTORS {
            let (maximum, cached) = match signature.as_str() {
                "newCachedThreadPool()Ljava/util/concurrent/ExecutorService;" => (64, true),
                "newSingleThreadExecutor()Ljava/util/concurrent/ExecutorService;" => (1, false),
                "newFixedThreadPool(I)Ljava/util/concurrent/ExecutorService;" => {
                    let count = arg(0)?.int()?;
                    if count <= 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "nonpositive executor size",
                        ));
                    }
                    (count as usize, false)
                }
                _ => return Ok(None),
            };
            return Ok(Some(vec![self.new_executor(maximum, cached)?]));
        }
        if [FUTURE, TASK, RUNNABLE_FUTURE].contains(&call.class.as_str()) {
            return Ok(Some(match signature.as_str() {
                "<init>(Ljava/util/concurrent/Callable;)V" => {
                    self.init_future(receiver, arg(1)?, true, Word::ZERO)?;
                    vec![]
                }
                "<init>(Ljava/lang/Runnable;Ljava/lang/Object;)V" => {
                    self.init_future(receiver, arg(1)?, false, arg(2)?)?;
                    vec![]
                }
                "isDone()Z" => vec![Word::from(i32::from(self.future(receiver)?.done()))],
                "isCancelled()Z" => vec![Word::from(i32::from(matches!(
                    self.future(receiver)?.outcome,
                    Outcome::Cancelled
                )))],
                "cancel(Z)Z" => vec![Word::from(i32::from(self.cancel_future(
                    receiver,
                    arg(1)?.truth(),
                    true,
                )?))],
                "get()Ljava/lang/Object;" => self.get_future(receiver, None)?,
                "get(JLjava/util/concurrent/TimeUnit;)Ljava/lang/Object;" => {
                    let duration = self.timeout_ms(&args[1..], arg(3)?)?;
                    self.get_future(receiver, Some(duration))?
                }
                "run()V" => {
                    let thread = self.current_thread()?;
                    if let Some((entry, body)) = self.begin_future_task(receiver, thread)? {
                        let result = self.invoke(entry, vec![body], true);
                        self.finish_future_task(receiver, result)?;
                    }
                    vec![]
                }
                "done()V" => {
                    self.future(receiver)?;
                    vec![]
                }
                "set(Ljava/lang/Object;)V" | "setException(Ljava/lang/Throwable;)V" => {
                    let value = arg(1)?;
                    value.reference()?;
                    let outcome = if call.name == "setException" {
                        if value == Word::ZERO {
                            return Err(fault(
                                "Ljava/lang/NullPointerException;",
                                "null FutureTask exception",
                            ));
                        }
                        ensure!(
                            self.is_a(&self.heap.get(value)?.class, "Ljava/lang/Throwable;"),
                            "expected Throwable"
                        );
                        Outcome::Failure(value)
                    } else {
                        Outcome::Value(value)
                    };
                    if !self.future(receiver)?.done() {
                        let future = self.future_mut(receiver)?;
                        future.outcome = outcome;
                        future.body = Word::ZERO;
                        future.preset = Word::ZERO;
                        self.future_done_callback(receiver)?;
                    }
                    vec![]
                }
                _ => return Ok(None),
            }));
        }
        Ok(Some(match signature.as_str() {
            "execute(Ljava/lang/Runnable;)V" => {
                self.enqueue_executor(receiver, arg(1)?)?;
                vec![]
            }
            "submit(Ljava/util/concurrent/Callable;)Ljava/util/concurrent/Future;"
            | "submit(Ljava/lang/Runnable;)Ljava/util/concurrent/Future;"
            | "submit(Ljava/lang/Runnable;Ljava/lang/Object;)Ljava/util/concurrent/Future;" => {
                let task = self.heap.instance(TASK)?;
                self.native_roots.push(task);
                self.init_future(
                    task,
                    arg(1)?,
                    call.parameters[0] == CALLABLE,
                    if call.parameters.len() == 2 {
                        arg(2)?
                    } else {
                        Word::ZERO
                    },
                )?;
                self.enqueue_executor(receiver, task)?;
                vec![task]
            }
            "shutdown()V" => {
                self.shutdown_executor(receiver, false)?;
                vec![]
            }
            "shutdownNow()Ljava/util/List;" => {
                let pending = self.shutdown_executor(receiver, true)?;
                let list = self.heap.instance("Ljava/util/ArrayList;")?;
                self.heap.get_mut(list)?.data = Data::Collection {
                    values: pending,
                    version: 0,
                };
                vec![list]
            }
            "isShutdown()Z" => vec![Word::from(i32::from(
                self.queue.closed || self.executor(receiver)?.shutdown,
            ))],
            "isTerminated()Z" => vec![Word::from(i32::from(self.completion_done(receiver)?))],
            "awaitTermination(JLjava/util/concurrent/TimeUnit;)Z" => {
                let duration = self.timeout_ms(&args[1..], arg(3)?)?;
                if let Err(error) = self.check_interrupt() {
                    self.clear_completion_wait(receiver)?;
                    return Err(error);
                }
                if self.completion_done(receiver)? {
                    self.clear_completion_wait(receiver)?;
                    vec![Word::from(1)]
                } else {
                    self.wait_completion(receiver, Some(duration))?;
                    vec![Word::ZERO]
                }
            }
            _ => return Ok(None),
        }))
    }
}
