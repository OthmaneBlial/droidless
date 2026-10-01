//! OverScroller's timed scroll mode, driven by the shared monotonic runtime clock.
use crate::{
    heap::{Word, bits64, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const SCROLLER: &str = "Landroid/widget/OverScroller;";
const INTERPOLATOR: &str = "Landroid/view/animation/Interpolator;";

fn viscous(input: f32) -> f32 {
    let curve = |input: f32| {
        let scaled = input * 8.0;
        if scaled < 1.0 {
            scaled - (1.0 - f64::from(-scaled).exp() as f32)
        } else {
            let start = (-1.0f64).exp() as f32;
            start + (1.0 - f64::from(1.0 - scaled).exp() as f32) * (1.0 - start)
        }
    };
    let normalize = 1.0 / curve(1.0);
    let value = normalize * curve(input);
    if value > 0.0 {
        value + (1.0 - normalize * curve(1.0))
    } else {
        value
    }
}

impl Runtime {
    fn scroller_int(&self, receiver: Word, name: &str) -> Result<i32> {
        self.heap
            .get(receiver)?
            .fields
            .get(name)
            .and_then(|v| v.first())
            .context("uninitialized OverScroller")?
            .int()
    }
    fn scroller_set(&mut self, receiver: Word, name: &str, value: i32) -> Result<()> {
        self.heap
            .get_mut(receiver)?
            .fields
            .insert(name.into(), vec![Word::from(value)]);
        Ok(())
    }
    pub(crate) fn scrolling_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != SCROLLER {
            return Ok(None);
        }
        let arg = |i| {
            args.get(i)
                .copied()
                .context("missing OverScroller argument")
        };
        let receiver = arg(0)?;
        let signature = method.signature();
        let result = match signature.as_str() {
            "<init>(Landroid/content/Context;)V"
            | "<init>(Landroid/content/Context;Landroid/view/animation/Interpolator;)V" => {
                ensure!(
                    self.is_a(&self.heap.get(arg(1)?)?.class, "Landroid/content/Context;"),
                    "OverScroller requires Context"
                );
                let interpolator = if args.len() > 2 { arg(2)? } else { Word::ZERO };
                if interpolator != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(interpolator)?.class, INTERPOLATOR),
                        "invalid scroll interpolator"
                    );
                }
                for name in [
                    "startX", "startY", "currX", "currY", "finalX", "finalY", "duration",
                ] {
                    self.scroller_set(receiver, name, 0)?;
                }
                self.scroller_set(receiver, "finished", 1)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("interpolator".into(), vec![interpolator]);
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("startTime".into(), wide(0));
                vec![]
            }
            "isFinished()Z" => vec![Word::from(self.scroller_int(receiver, "finished")?)],
            "getStartX()I" | "getStartY()I" | "getCurrX()I" | "getCurrY()I" | "getFinalX()I"
            | "getFinalY()I" | "getDuration()I" => {
                let name = method
                    .name
                    .strip_prefix("get")
                    .context("invalid scroll getter")?;
                let name = name[..1].to_ascii_lowercase() + &name[1..];
                vec![Word::from(self.scroller_int(receiver, &name)?)]
            }
            "forceFinished(Z)V" => {
                self.scroller_set(receiver, "finished", i32::from(arg(1)?.truth()))?;
                vec![]
            }
            "startScroll(IIII)V" | "startScroll(IIIII)V" => {
                for (axis, start, delta) in [
                    ("X", arg(1)?.int()?, arg(3)?.int()?),
                    ("Y", arg(2)?.int()?, arg(4)?.int()?),
                ] {
                    self.scroller_set(receiver, &format!("start{axis}"), start)?;
                    self.scroller_set(
                        receiver,
                        &format!("final{axis}"),
                        start.wrapping_add(delta),
                    )?;
                }
                let duration = if args.len() > 5 { arg(5)?.int()? } else { 250 };
                self.scroller_set(receiver, "duration", duration)?;
                self.scroller_set(receiver, "finished", 0)?;
                let start_time = self.uptime_ms();
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("startTime".into(), wide(start_time));
                vec![]
            }
            "abortAnimation()V" => {
                self.finish_scroll(receiver)?;
                vec![]
            }
            "computeScrollOffset()Z" => {
                if self.scroller_int(receiver, "finished")? != 0 {
                    return Ok(Some(vec![Word::ZERO]));
                }
                let start = bits64(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("startTime")
                        .context("uninitialized OverScroller time")?,
                )?;
                let elapsed = self.uptime_ms().saturating_sub(start);
                let duration = self.scroller_int(receiver, "duration")?;
                if duration <= 0 || elapsed >= duration as u64 {
                    self.finish_scroll(receiver)?;
                } else {
                    let input = elapsed as f32 / duration as f32;
                    let interpolator = *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("interpolator")
                        .and_then(|v| v.first())
                        .context("uninitialized OverScroller interpolator")?;
                    let progress = if interpolator == Word::ZERO {
                        viscous(input)
                    } else {
                        let roots = self.native_roots.len();
                        self.native_roots.extend([receiver, interpolator]);
                        let result = self.invoke(
                            Method {
                                class: INTERPOLATOR.into(),
                                name: "getInterpolation".into(),
                                parameters: vec!["F".into()],
                                returns: "F".into(),
                            },
                            vec![interpolator, Word::Bits(input.to_bits())],
                            true,
                        );
                        self.native_roots.truncate(roots);
                        f32::from_bits(
                            result?
                                .first()
                                .context("interpolator result missing")?
                                .int()? as u32,
                        )
                    };
                    for axis in ["X", "Y"] {
                        let start = self.scroller_int(receiver, &format!("start{axis}"))?;
                        let end = self.scroller_int(receiver, &format!("final{axis}"))?;
                        // Java Math.round(float) rounds negative halves toward positive infinity.
                        let distance = (f64::from(progress * end.wrapping_sub(start) as f32) + 0.5)
                            .floor() as i32;
                        self.scroller_set(
                            receiver,
                            &format!("curr{axis}"),
                            start.wrapping_add(distance),
                        )?;
                    }
                }
                vec![Word::from(1)]
            }
            // ponytail: real timed scrolls only; fling/springback physics and View gesture delivery remain explicit unsupported APIs.
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
    fn finish_scroll(&mut self, receiver: Word) -> Result<()> {
        for axis in ["X", "Y"] {
            let end = self.scroller_int(receiver, &format!("final{axis}"))?;
            self.scroller_set(receiver, &format!("curr{axis}"), end)?;
        }
        self.scroller_set(receiver, "finished", 1)
    }
}
