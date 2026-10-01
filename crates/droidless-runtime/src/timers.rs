//! Timers own a bounded queue and one resumable guest worker, never a main-thread callback.
use crate::{
    heap::{Data, Word, bits64, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;
use std::collections::BTreeMap;

const TIMER: &str = "Ljava/util/Timer;";
const TASK: &str = "Ljava/util/TimerTask;";
const THREAD: &str = "Ljava/lang/Thread;";
const LIMIT: usize = 16_384;
const USED: &str = "droidless:timer-task:used";
const CANCELLED: &str = "droidless:timer-task:cancelled";
const WHEN: &str = "droidless:timer-task:when";
const PERIOD: &str = "droidless:timer-task:period";
const FIXED_RATE: &str = "droidless:timer-task:fixed-rate";
const SCHEDULED: &str = "droidless:timer-task:scheduled";

#[derive(Clone, Debug, Default)]
pub struct Timer {
    pub(crate) tasks: BTreeMap<(i64, u64), Word>,
    pub(crate) active: Option<Word>,
    cancelled: bool,
    sequence: u64,
}
impl Runtime {
    fn timer(&self, owner: Word) -> Result<&Timer> {
        let Data::Timer(timer) = &self.heap.get(owner)?.data else {
            bail!("uninitialized Timer");
        };
        Ok(timer)
    }
    fn timer_mut(&mut self, owner: Word) -> Result<&mut Timer> {
        let Data::Timer(timer) = &mut self.heap.get_mut(owner)?.data else {
            bail!("uninitialized Timer");
        };
        Ok(timer)
    }
    fn task_long(&self, task: Word, key: &str) -> Result<i64> {
        Ok(self
            .heap
            .get(task)?
            .fields
            .get(key)
            .map(|value| bits64(value))
            .transpose()?
            .unwrap_or(0) as i64)
    }
    pub(crate) fn timer_ready(&self, owner: Word) -> Result<bool> {
        let timer = self.timer(owner)?;
        Ok(timer.cancelled
            || match timer.tasks.first_key_value() {
                Some(((due, _), task)) => {
                    *due <= self.uptime_ms() as i64 || self.thread_word(*task, CANCELLED)?.truth()
                }
                None => false,
            })
    }
    pub(crate) fn take_timer_task(&mut self, owner: Word) -> Result<Option<Word>> {
        loop {
            let timer = self.timer(owner)?;
            if timer.cancelled {
                return Ok(None);
            }
            let Some((&key, &task)) = timer.tasks.first_key_value() else {
                return Ok(None);
            };
            if self.thread_word(task, CANCELLED)?.truth() {
                self.timer_mut(owner)?.tasks.remove(&key);
                continue;
            }
            if key.0 > self.uptime_ms() as i64 {
                return Ok(None);
            }
            let when = self.task_long(task, WHEN)?;
            let period = self.task_long(task, PERIOD)?;
            // Like API 21 Timer, fixed-delay rescheduling uses dispatch time, before run().
            let next = if period > 0 {
                let fixed = self.thread_word(task, FIXED_RATE)?.truth();
                let wall = if fixed { when } else { self.wall_time_ms() };
                let due = if fixed {
                    key.0
                } else {
                    self.uptime_ms() as i64
                };
                Some((
                    wall.checked_add(period)
                        .context("Timer wall deadline overflow")?,
                    due.checked_add(period).context("Timer deadline overflow")?,
                ))
            } else {
                None
            };
            let fields = &mut self.heap.get_mut(task)?.fields;
            fields.insert(SCHEDULED.into(), wide(when as u64));
            fields.insert(WHEN.into(), wide(next.map_or(0, |(wall, _)| wall as u64)));
            let timer = self.timer_mut(owner)?;
            timer.tasks.remove(&key);
            timer.active = Some(task);
            if let Some((_, due)) = next {
                timer.tasks.insert((due, key.1), task);
            }
            return Ok(Some(task));
        }
    }
    pub(crate) fn finish_timer_task(&mut self, owner: Word, failed: bool) -> Result<bool> {
        let timer = self.timer_mut(owner)?;
        timer.active = None;
        if failed {
            timer.cancelled = true;
            timer.tasks.clear();
        }
        Ok(!timer.cancelled)
    }
    fn schedule_timer(
        &mut self,
        owner: Word,
        task: Word,
        when: i64,
        period: i64,
        fixed: bool,
    ) -> Result<()> {
        if task == Word::ZERO {
            return Err(fault("Ljava/lang/NullPointerException;", "null TimerTask"));
        }
        ensure!(
            self.is_a(&self.heap.get(task)?.class, TASK),
            "expected TimerTask"
        );
        if self.timer(owner)?.cancelled {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "Timer cancelled or terminated",
            ));
        }
        if self.thread_word(task, USED)?.truth() || self.thread_word(task, CANCELLED)?.truth() {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "TimerTask already scheduled or cancelled",
            ));
        }
        ensure!(
            self.timer(owner)?.tasks.len() < LIMIT,
            "Timer task limit reached ({LIMIT})"
        );
        // ponytail: anchor Date deadlines to monotonic uptime; wall-clock jumps need a rebase policy.
        let due = i64::try_from(
            i128::from(self.uptime_ms()) + i128::from(when) - i128::from(self.wall_time_ms()),
        )
        .map_err(|_| {
            fault(
                "Ljava/lang/IllegalArgumentException;",
                "Timer deadline overflow",
            )
        })?;
        let sequence = self
            .timer(owner)?
            .sequence
            .checked_add(1)
            .context("Timer sequence exhausted")?;
        let fields = &mut self.heap.get_mut(task)?.fields;
        fields.insert(USED.into(), vec![Word::from(1)]);
        fields.insert(WHEN.into(), wide(when as u64));
        fields.insert(PERIOD.into(), wide(period as u64));
        fields.insert(FIXED_RATE.into(), vec![Word::from(i32::from(fixed))]);
        let timer = self.timer_mut(owner)?;
        timer.sequence = sequence;
        timer.tasks.insert((due, sequence), task);
        Ok(())
    }
    pub(crate) fn timer_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != TIMER && method.class != TASK {
            return Ok(None);
        }
        let roots = self.native_roots.len();
        self.native_roots.extend_from_slice(args);
        let result = self.timer_call(method, args);
        self.native_roots.truncate(roots);
        result
    }
    fn timer_call(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        let receiver = *args.first().context("Timer receiver missing")?;
        let sig = method.signature();
        if method.class == TASK {
            return Ok(match sig.as_str() {
                "<init>()V" => {
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.insert(USED.into(), vec![Word::ZERO]);
                    fields.insert(CANCELLED.into(), vec![Word::ZERO]);
                    fields.insert(WHEN.into(), wide(0));
                    fields.insert(SCHEDULED.into(), wide(0));
                    Some(vec![])
                }
                "cancel()Z" => {
                    let pending = !self.thread_word(receiver, CANCELLED)?.truth()
                        && self.task_long(receiver, WHEN)? > 0;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(CANCELLED.into(), vec![Word::from(1)]);
                    Some(vec![Word::from(i32::from(pending))])
                }
                "scheduledExecutionTime()J" => {
                    Some(wide(self.task_long(receiver, SCHEDULED)? as u64))
                }
                _ => None,
            });
        }
        match sig.as_str() {
            "<init>()V"
            | "<init>(Z)V"
            | "<init>(Ljava/lang/String;)V"
            | "<init>(Ljava/lang/String;Z)V" => {
                let named = method
                    .parameters
                    .first()
                    .is_some_and(|ty| ty == "Ljava/lang/String;");
                let name = if named {
                    let name = *args.get(1).context("Timer name missing")?;
                    if name == Word::ZERO {
                        return Err(fault("Ljava/lang/NullPointerException;", "null Timer name"));
                    }
                    self.heap.text(name)?;
                    name
                } else {
                    let id = self.queue.timer_id;
                    self.queue.timer_id =
                        id.checked_add(1).context("Timer name sequence exhausted")?;
                    self.heap.string(format!("Timer-{id}"))?
                };
                let daemon = if method.parameters.last().is_some_and(|ty| ty == "Z") {
                    args.last().copied().context("Timer daemon missing")?
                } else {
                    Word::ZERO
                };
                let thread = self.heap.instance(THREAD)?;
                self.native_roots.extend([thread, name]);
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
                    .insert("daemon".into(), vec![Word::from(i32::from(daemon.truth()))]);
                self.heap
                    .get_mut(thread)?
                    .fields
                    .insert("droidless:timer:owner".into(), vec![receiver]);
                self.heap.get_mut(receiver)?.data = Data::Timer(Timer::default());
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:timer:thread".into(), vec![thread]);
                self.start_timer_worker(thread, receiver)?;
            }
            "cancel()V" => {
                let timer = self.timer_mut(receiver)?;
                timer.cancelled = true;
                timer.tasks.clear();
            }
            "purge()I" => {
                let cancelled: Vec<_> = self
                    .timer(receiver)?
                    .tasks
                    .iter()
                    .filter_map(|(key, task)| match self.thread_word(*task, CANCELLED) {
                        Ok(word) if word.truth() => Some(Ok(*key)),
                        Ok(_) => None,
                        Err(error) => Some(Err(error)),
                    })
                    .collect::<Result<_>>()?;
                for key in &cancelled {
                    self.timer_mut(receiver)?.tasks.remove(key);
                }
                return Ok(Some(vec![Word::from(cancelled.len() as i32)]));
            }
            "schedule(Ljava/util/TimerTask;J)V"
            | "schedule(Ljava/util/TimerTask;Ljava/util/Date;)V"
            | "schedule(Ljava/util/TimerTask;JJ)V"
            | "schedule(Ljava/util/TimerTask;Ljava/util/Date;J)V"
            | "scheduleAtFixedRate(Ljava/util/TimerTask;JJ)V"
            | "scheduleAtFixedRate(Ljava/util/TimerTask;Ljava/util/Date;J)V" => {
                let task = *args.get(1).context("TimerTask missing")?;
                let date = method.parameters[1] == "Ljava/util/Date;";
                let fixed = method.name == "scheduleAtFixedRate";
                let now = self.wall_time_ms();
                let when = if date {
                    let date = *args.get(2).context("Timer Date missing")?;
                    if date == Word::ZERO {
                        return Err(fault("Ljava/lang/NullPointerException;", "null Timer Date"));
                    }
                    ensure!(
                        self.is_a(&self.heap.get(date)?.class, "Ljava/util/Date;"),
                        "expected Date"
                    );
                    let value = bits64(&self.invoke(
                        Method {
                            class: "Ljava/util/Date;".into(),
                            name: "getTime".into(),
                            parameters: vec![],
                            returns: "J".into(),
                        },
                        vec![date],
                        true,
                    )?)? as i64;
                    if value < 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "negative Timer Date",
                        ));
                    }
                    if fixed {
                        value
                    } else {
                        value.max(self.wall_time_ms())
                    }
                } else {
                    let delay = bits64(&args[2..])? as i64;
                    if delay < 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "negative Timer delay",
                        ));
                    }
                    now.checked_add(delay).ok_or_else(|| {
                        fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "Timer delay overflow",
                        )
                    })?
                };
                let period = if method.parameters.len() == 3 {
                    let period = bits64(&args[if date { 3 } else { 4 }..])? as i64;
                    if period <= 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "nonpositive Timer period",
                        ));
                    }
                    period
                } else {
                    -1
                };
                self.schedule_timer(receiver, task, when, period, fixed)?;
            }
            _ => return Ok(None),
        }
        Ok(Some(vec![]))
    }
}
