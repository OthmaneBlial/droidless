//! Worker message delivery resumes real DEX callbacks through the managed stack.
use crate::{
    heap::{Word, fault},
    vm::Runtime,
    workers::Waiting,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const LOOPER: &str = "Landroid/os/Looper;";
const HANDLER: &str = "Landroid/os/Handler;";
const MESSAGE: &str = "Landroid/os/Message;";
const ACTIVE: &str = "droidless:looper:active";
const PHASE: &str = "droidless:looper:phase";
const DEPTH: &str = "droidless:looper:frame-depth";

impl Runtime {
    pub(crate) fn validate_looper(&mut self, looper: Word) -> Result<()> {
        ensure!(
            self.is_a(&self.heap.get(looper)?.class, LOOPER),
            "Handler requires Looper"
        );
        let thread = self.message_word(looper, "thread")?;
        self.heap.get(thread)?;
        ensure!(
            looper == self.main_looper()? || self.thread_word(thread, "looper")? == looper,
            "uninitialized Looper"
        );
        Ok(())
    }

    pub(crate) fn next_looper_message(&self, looper: Word) -> Result<Option<((u64, u64), Word)>> {
        if !self.queue.closed {
            // ponytail: one bounded ordered queue; split per Looper if scans become a measured cost.
            for (&key, &message) in &self.queue.pending {
                if key.0 > self.uptime_ms() {
                    break;
                }
                if self.message_word(message, "looper")? == looper {
                    return Ok(Some((key, message)));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn looper_ready(&self, looper: Word) -> Result<bool> {
        Ok(self.message_word(looper, "quitting")?.truth()
            || self.next_looper_message(looper)?.is_some())
    }

    pub(crate) fn quit_worker_looper(&mut self, looper: Word, safely: bool) -> Result<()> {
        self.validate_looper(looper)?;
        if looper == self.main_looper()? {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "the main Looper cannot quit",
            ));
        }
        if self.message_word(looper, "quitting")?.truth() {
            return Ok(());
        }
        self.heap
            .get_mut(looper)?
            .fields
            .insert("quitting".into(), vec![Word::from(1)]);
        let now = self.uptime_ms();
        let mut removed = vec![];
        for (&key, &message) in &self.queue.pending {
            if self.message_word(message, "looper")? == looper && (!safely || key.0 > now) {
                removed.push(key);
            }
        }
        for key in removed {
            let message = self
                .queue
                .pending
                .remove(&key)
                .context("Looper message disappeared")?;
            self.retire_message(message)?;
        }
        Ok(())
    }

    pub(crate) fn finish_looper_message(&mut self, looper: Word) -> Result<()> {
        let thread = self.message_word(looper, "thread")?;
        let message = self.thread_word(thread, ACTIVE)?;
        if message != Word::ZERO {
            self.retire_message(message)?;
        }
        let fields = &mut self.heap.get_mut(thread)?.fields;
        fields.remove(ACTIVE);
        fields.remove(PHASE);
        Ok(())
    }

    pub(crate) fn clear_looper_frame(&mut self, looper: Word) -> Result<()> {
        let thread = self.message_word(looper, "thread")?;
        self.heap.get_mut(thread)?.fields.remove(DEPTH);
        Ok(())
    }

    fn apk_dispatch_message(&self, handler: Word) -> Result<bool> {
        let mut class = self.heap.get(handler)?.class.clone();
        for _ in 0..128 {
            if class == HANDLER {
                return Ok(false);
            }
            if let Some((dex, index)) = self.class_location(&class)
                && self.apk.dex[dex].classes[index]
                    .methods
                    .iter()
                    .any(|method| {
                        self.apk.dex[dex].methods[method.index].signature()
                            == "dispatchMessage(Landroid/os/Message;)V"
                    })
            {
                return Ok(true);
            }
            class = self
                .parent(&class)
                .context("Handler dispatch hierarchy missing")?;
        }
        anyhow::bail!("Handler dispatch hierarchy limit reached")
    }

    pub(crate) fn begin_worker_looper(&mut self) -> Result<Option<Vec<Word>>> {
        let thread = self.current_thread()?;
        let looper = if self.workers.current.is_some() {
            self.thread_word(thread, "looper")?
        } else {
            self.main_looper()?
        };
        if looper == Word::ZERO {
            return Err(fault(
                "Ljava/lang/RuntimeException;",
                "No Looper; call Looper.prepare first",
            ));
        }
        ensure!(
            self.workers.current.is_some() && self.sync_depth == 0,
            "unsupported Looper.loop on main or across a synchronous native bridge"
        );
        let depth = self.frames.len();
        ensure!(depth > 0, "Looper.loop requires a managed caller");
        let recorded = self.thread_word(thread, DEPTH)?.int()? as usize;
        // ponytail: one message pump per worker; use stacked pump state if an APK needs nested loop().
        ensure!(
            recorded == 0 || recorded == depth,
            "nested worker Looper.loop unsupported"
        );
        self.heap
            .get_mut(thread)?
            .fields
            .insert(DEPTH.into(), vec![Word::from(depth as i32)]);
        let return_pc = self.frames.last().context("Looper caller missing")?.pc;
        loop {
            let mut message = self.thread_word(thread, ACTIVE)?;
            let mut phase = self.thread_word(thread, PHASE)?.int()?;
            if message != Word::ZERO {
                let handled = phase != 2
                    || self
                        .frames
                        .last()
                        .context("Looper caller missing")?
                        .result
                        .first()
                        .context("Handler.Callback returned no value")?
                        .int()?
                        != 0;
                if handled {
                    self.finish_looper_message(looper)?;
                    message = Word::ZERO;
                } else {
                    phase = 3;
                }
            }
            if message == Word::ZERO {
                let Some((key, next)) = self.next_looper_message(looper)? else {
                    if self.message_word(looper, "quitting")?.truth() {
                        self.clear_looper_frame(looper)?;
                        return Ok(Some(vec![]));
                    }
                    self.wait_worker(Waiting::Looper(looper))?;
                    unreachable!();
                };
                self.queue.pending.remove(&key);
                message = next;
                phase = 0;
                self.heap
                    .get_mut(thread)?
                    .fields
                    .insert(ACTIVE.into(), vec![message]);
            }
            let handler = self.message_word(message, "target")?;
            let callback = self.message_word(message, "callback")?;
            let handler_callback = self.message_word(handler, "handlerCallback")?;
            let (class, name, parameters, returns, args, next_phase) =
                if phase == 0 && self.apk_dispatch_message(handler)? {
                    (
                        HANDLER,
                        "dispatchMessage",
                        vec![MESSAGE.into()],
                        "V",
                        vec![handler, message],
                        1,
                    )
                } else if phase == 0 && callback != Word::ZERO {
                    (
                        "Ljava/lang/Runnable;",
                        "run",
                        vec![],
                        "V",
                        vec![callback],
                        1,
                    )
                } else if phase == 0 && handler_callback != Word::ZERO {
                    (
                        "Landroid/os/Handler$Callback;",
                        "handleMessage",
                        vec![MESSAGE.into()],
                        "Z",
                        vec![handler_callback, message],
                        2,
                    )
                } else {
                    (
                        HANDLER,
                        "handleMessage",
                        vec![MESSAGE.into()],
                        "V",
                        vec![handler, message],
                        1,
                    )
                };
            self.heap
                .get_mut(thread)?
                .fields
                .insert(PHASE.into(), vec![Word::from(next_phase)]);
            let delivery = self.begin_invoke(
                Method {
                    class: class.into(),
                    name: name.into(),
                    parameters,
                    returns: returns.into(),
                },
                args,
                true,
            );
            let delivery = match delivery {
                Ok(delivery) => delivery,
                Err(error) => {
                    self.finish_looper_message(looper)?;
                    self.clear_looper_frame(looper)?;
                    return Err(error);
                }
            };
            match delivery {
                None => {
                    let frame = self
                        .frames
                        .last_mut()
                        .context("Looper callback frame missing")?;
                    frame.return_pc = Some(return_pc);
                    frame.looper_return = Some(looper);
                    return Ok(None);
                }
                Some(words) => {
                    self.frames
                        .last_mut()
                        .context("Looper caller missing")?
                        .result = words;
                    if next_phase == 1 {
                        self.finish_looper_message(looper)?;
                        // Native no-op callbacks yield too, sharing the worker slice ceiling.
                        self.wait_worker(Waiting::Looper(looper))?;
                        unreachable!();
                    }
                }
            }
        }
    }
}
