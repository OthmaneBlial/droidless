use crate::{
    heap::{Data, GuestFault, Word, bits64, default_value, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

#[derive(Debug)]
pub(crate) struct Thrown(pub Word, pub String);
impl std::fmt::Display for Thrown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uncaught guest exception {}", self.1)
    }
}
impl std::error::Error for Thrown {}
enum Flow {
    Next(usize),
    Jump(usize),
    Return(Vec<Word>),
    Call,
}

impl Runtime {
    pub(crate) fn throw_reference(&self, object: Word) -> Result<anyhow::Error> {
        let value = self.heap.get(object)?;
        ensure!(
            self.is_a(&value.class, "Ljava/lang/Throwable;"),
            "throw requires a Throwable"
        );
        let mut description = value.class.clone();
        if let Some(message) = value.fields.get("message").and_then(|v| v.first())
            && *message != Word::ZERO
        {
            description.push_str(": ");
            description.push_str(self.heap.text(*message)?);
        }
        Ok(Thrown(object, description).into())
    }
    fn unit(&self, f: usize, offset: usize) -> Result<u16> {
        self.frames[f]
            .code
            .instructions
            .get(self.frames[f].pc + offset)
            .copied()
            .context("truncated DEX instruction")
    }
    fn reg(&self, f: usize, n: usize) -> Result<Word> {
        self.frames[f]
            .registers
            .get(n)
            .copied()
            .context("DEX register index out of range")
    }
    fn put(&mut self, f: usize, n: usize, words: &[Word]) -> Result<()> {
        let end = n
            .checked_add(words.len())
            .context("register range overflow")?;
        self.frames[f]
            .registers
            .get_mut(n..end)
            .context("DEX register range out of bounds")?
            .copy_from_slice(words);
        Ok(())
    }
    fn pair(&self, f: usize, n: usize) -> Result<u64> {
        bits64(&[self.reg(f, n)?, self.reg(f, n + 1)?])
    }
    fn array_value(&self, element: &str, words: &[Word]) -> Result<()> {
        ensure!(
            words.len() == if matches!(element, "J" | "D") { 2 } else { 1 },
            "array value width mismatch"
        );
        if element.starts_with(['L', '[']) {
            if words[0] != Word::ZERO && !self.is_a(&self.heap.get(words[0])?.class, element) {
                return Err(fault(
                    "Ljava/lang/ArrayStoreException;",
                    format!("incompatible element for [{element}"),
                ));
            }
        } else {
            for word in words {
                word.int()?;
            }
        }
        Ok(())
    }
    fn jump(&self, f: usize, offset: i32) -> Result<Flow> {
        let pc = self.frames[f].pc as i64 + i64::from(offset);
        ensure!(
            pc >= 0 && pc < self.frames[f].code.instructions.len() as i64,
            "DEX branch target out of range"
        );
        Ok(Flow::Jump(pc as usize))
    }
    pub(crate) fn execute(&mut self, base: usize) -> Result<Vec<Word>> {
        loop {
            if let Some(words) = self.execute_slice(base, 1024)? {
                return Ok(words);
            }
        }
    }
    /// Run a bounded slice, retaining every DEX caller/callee frame on pause.
    /// Native bridges and class initialization remain synchronous within a step.
    pub(crate) fn execute_slice(
        &mut self,
        base: usize,
        quantum: usize,
    ) -> Result<Option<Vec<Word>>> {
        ensure!(base < self.frames.len(), "missing execution continuation");
        for _ in 0..quantum {
            let f = self.frames.len() - 1;
            let flow = (|| {
                self.tick()?;
                if self.trace.bytecode {
                    eprintln!(
                        "{} {:04x}: {:04x}",
                        self.frames[f].method.key(),
                        self.frames[f].pc,
                        self.unit(f, 0)?
                    );
                }
                self.step(f)
            })();
            let flow = match flow {
                Ok(flow) => flow,
                Err(error) => {
                    if let Some(waiting) = error.downcast_ref::<crate::workers::Waiting>() {
                        if base == 0 && self.sync_depth == 0 && self.workers.current.is_some() {
                            self.workers.waiting = Some(*waiting);
                            return Ok(None);
                        }
                        self.unwind_frames(base, anyhow::anyhow!("unsupported worker suspension across a synchronous native bridge or class initializer: {waiting}"))?;
                        continue;
                    }
                    self.unwind_frames(base, error)?;
                    continue;
                }
            };
            match flow {
                Flow::Next(n) => self.frames[f].pc += n,
                Flow::Jump(pc) => self.frames[f].pc = pc,
                Flow::Call => {
                    debug_assert_eq!(self.frames.len(), f + 2);
                }
                Flow::Return(words) => {
                    if !self.frames[f].monitors.is_empty() {
                        self.unwind_frames(
                            base,
                            anyhow::anyhow!("unbalanced DEX monitors at method return"),
                        )?;
                        continue;
                    }
                    let finished = self.frames.pop().context("frame stack underflow")?;
                    if f == base {
                        return Ok(Some(words));
                    }
                    let caller = &mut self.frames[f - 1];
                    caller.result = words;
                    caller.pc = finished
                        .return_pc
                        .context("missing DEX return continuation")?;
                }
            }
        }
        Ok(None)
    }
    fn unwind_frames(&mut self, base: usize, mut error: anyhow::Error) -> Result<()> {
        if let Some(fault) = error.downcast_ref::<GuestFault>() {
            error = self
                .guest_exception(fault.0, fault.1.clone(), None)
                .unwrap_or_else(|e| e);
        }
        loop {
            let f = self.frames.len() - 1;
            if let Some(thrown) = error.downcast_ref::<Thrown>() {
                let exception = thrown.0;
                match self.heap.get(exception) {
                    Ok(object) => {
                        let class = &object.class;
                        let frame = &self.frames[f];
                        let pc = frame.pc as u32;
                        let target = frame
                            .code
                            .handlers
                            .iter()
                            .filter(|h| pc >= h.start && pc < h.end)
                            .flat_map(|h| &h.catches)
                            .find(|(ty, _)| ty.as_ref().is_none_or(|ty| self.is_a(class, ty)))
                            .map(|(_, pc)| *pc);
                        if let Some(pc) = target {
                            self.frames[f].exception = Some(exception);
                            self.frames[f].pc = pc as usize;
                            return Ok(());
                        }
                    }
                    Err(invalid) => error = invalid,
                }
            }
            let finished = self.frames.pop().context("frame stack underflow")?;
            self.release_frame_monitors(&finished)?;
            if let Some(looper) = finished.looper_return {
                self.finish_looper_message(looper)?;
                self.clear_looper_frame(looper)?;
            }
            error = error.context(finished.location());
            if f == base {
                return Err(error);
            }
        }
    }
    fn step(&mut self, f: usize) -> Result<Flow> {
        let word = self.unit(f, 0)?;
        let op = word as u8;
        let a = usize::from(word >> 8);
        let lo = a & 15;
        let hi = a >> 4;
        let d = self.frames[f].dex;
        macro_rules! u {
            ($n:expr) => {
                self.unit(f, $n)?
            };
        }
        macro_rules! r {
            ($n:expr) => {
                self.reg(f, $n)?
            };
        }
        macro_rules! int {
            ($n:expr) => {
                r!($n).int()?
            };
        }
        macro_rules! put {
            ($n:expr,$v:expr) => {
                self.put(f, $n, &[$v])?
            };
        }
        let mut next = 1;
        match op {
            0x00 => {
                ensure!(word == 0, "executed DEX payload as instruction");
            }
            0x01..=0x09 => {
                let (dest, src, width) = match op {
                    1 | 4 | 7 => (lo, hi, 1),
                    2 | 5 | 8 => (a, usize::from(u!(1)), 2),
                    _ => (usize::from(u!(1)), usize::from(u!(2)), 3),
                };
                let count = if (4..=6).contains(&op) { 2 } else { 1 };
                let words = (0..count)
                    .map(|n| self.reg(f, src + n))
                    .collect::<Result<Vec<_>>>()?;
                self.put(f, dest, &words)?;
                next = width;
            }
            0x0a..=0x0c => {
                let result = self.frames[f].result.clone();
                ensure!(
                    result.len() == if op == 0x0b { 2 } else { 1 },
                    "move-result has no matching invoke result"
                );
                self.put(f, a, &result)?;
                self.frames[f].result.clear();
            }
            0x0d => {
                let exception = self.frames[f]
                    .exception
                    .take()
                    .context("move-exception outside catch handler")?;
                put!(a, exception);
            }
            0x0e => return Ok(Flow::Return(vec![])),
            0x0f | 0x11 => return Ok(Flow::Return(vec![r!(a)])),
            0x10 => return Ok(Flow::Return(vec![r!(a), r!(a + 1)])),
            0x12 => {
                put!(lo, Word::from(((word as i16) >> 12) as i32));
            }
            0x13 => {
                put!(a, Word::from(u!(1) as i16 as i32));
                next = 2;
            }
            0x14 => {
                put!(a, Word::Bits(u32::from(u!(1)) | (u32::from(u!(2)) << 16)));
                next = 3;
            }
            0x15 => {
                put!(a, Word::Bits(u32::from(u!(1)) << 16));
                next = 2;
            }
            0x16..=0x19 => {
                let value = match op {
                    0x16 => u!(1) as i16 as i64 as u64,
                    0x17 => (u32::from(u!(1)) | (u32::from(u!(2)) << 16)) as i32 as i64 as u64,
                    0x18 => {
                        u64::from(u!(1))
                            | (u64::from(u!(2)) << 16)
                            | (u64::from(u!(3)) << 32)
                            | (u64::from(u!(4)) << 48)
                    }
                    _ => u64::from(u!(1)) << 48,
                };
                self.put(f, a, &wide(value))?;
                next = match op {
                    0x16 | 0x19 => 2,
                    0x17 => 3,
                    _ => 5,
                };
            }
            0x1a | 0x1b => {
                let idx = if op == 0x1a {
                    u32::from(u!(1))
                } else {
                    u32::from(u!(1)) | (u32::from(u!(2)) << 16)
                };
                let s = self.apk.dex[d]
                    .strings
                    .get(idx as usize)
                    .context("invalid const-string index")?
                    .clone();
                let v = self.intern(s)?;
                put!(a, v);
                next = if op == 0x1a { 2 } else { 3 };
            }
            0x1c => {
                let class = self.apk.dex[d]
                    .types
                    .get(usize::from(u!(1)))
                    .context("invalid const-class type")?
                    .clone();
                let v = self.class_object(&class)?;
                put!(a, v);
                next = 2;
            }
            0x1d | 0x1e => {
                let object = r!(a);
                if op == 0x1d {
                    self.enter_monitor(object)?;
                    self.frames[f].monitors.push(object);
                } else {
                    self.heap.get(object)?;
                    let index = self.frames[f]
                        .monitors
                        .iter()
                        .rposition(|w| *w == object)
                        .ok_or_else(|| {
                            fault(
                                "Ljava/lang/IllegalMonitorStateException;",
                                "DEX monitor not acquired by this frame",
                            )
                        })?;
                    self.exit_monitor(object)?;
                    self.frames[f].monitors.remove(index);
                }
            }
            0x1f | 0x20 => {
                let ty = self.apk.dex[d]
                    .types
                    .get(usize::from(u!(1)))
                    .context("invalid cast type")?
                    .clone();
                let object = r!(if op == 0x20 { hi } else { a });
                let matches = object == Word::ZERO || self.is_a(&self.heap.get(object)?.class, &ty);
                if op == 0x1f {
                    if !matches {
                        let actual = if object == Word::ZERO {
                            "null".into()
                        } else {
                            self.heap.get(object)?.class.clone()
                        };
                        return Err(fault(
                            "Ljava/lang/ClassCastException;",
                            format!("{actual} cannot be cast to {ty}"),
                        ));
                    }
                } else {
                    put!(lo, Word::from(i32::from(object != Word::ZERO && matches)));
                }
                next = 2;
            }
            0x21 => {
                let Data::Array { values, .. } = &self.heap.get(r!(hi))?.data else {
                    bail!("array-length on non-array");
                };
                let n = values.len() as i32;
                put!(lo, Word::from(n));
            }
            0x22 => {
                let class = self.apk.dex[d]
                    .types
                    .get(usize::from(u!(1)))
                    .context("invalid new-instance type")?
                    .clone();
                let v = self.new_instance(&class)?;
                put!(a, v);
                next = 2;
            }
            0x23 => {
                let class = self.apk.dex[d]
                    .types
                    .get(usize::from(u!(1)))
                    .context("invalid new-array type")?
                    .clone();
                let element = class
                    .strip_prefix('[')
                    .context("new-array expects array type")?
                    .to_owned();
                let length = usize::try_from(int!(hi)).map_err(|_| {
                    fault(
                        "Ljava/lang/NegativeArraySizeException;",
                        "negative array length",
                    )
                })?;
                let v = self.array(element, length)?;
                put!(lo, v);
                next = 2;
            }
            0x24 | 0x25 => {
                let regs = self.invoke_registers(f, op == 0x25)?;
                let class = self.apk.dex[d]
                    .types
                    .get(usize::from(u!(1)))
                    .context("invalid filled-array type")?
                    .clone();
                let element = class
                    .strip_prefix('[')
                    .context("filled-new-array expects array type")?
                    .to_owned();
                ensure!(
                    element != "J" && element != "D",
                    "filled-new-array cannot contain wide values"
                );
                let values = regs
                    .iter()
                    .map(|n| Ok(vec![self.reg(f, *n)?]))
                    .collect::<Result<Vec<_>>>()?;
                for value in &values {
                    self.array_value(&element, value)?;
                }
                let array = self.array(element, values.len())?;
                if let Data::Array { values: target, .. } = &mut self.heap.get_mut(array)?.data {
                    *target = values;
                }
                self.frames[f].result = vec![array];
                next = 3;
            }
            0x26 => {
                let offset = (u32::from(u!(1)) | (u32::from(u!(2)) << 16)) as i32;
                let Flow::Jump(pc) = self.jump(f, offset)? else {
                    unreachable!()
                };
                let code = &self.frames[f].code.instructions;
                let unit = |n| -> Result<u16> {
                    code.get(pc + n).copied().context("truncated array payload")
                };
                ensure!(unit(0)? == 0x300, "invalid fill-array-data payload");
                let width = usize::from(unit(1)?);
                ensure!([1, 2, 4, 8].contains(&width), "invalid array payload width");
                let count = (u32::from(unit(2)?) | (u32::from(unit(3)?) << 16)) as usize;
                ensure!(count <= 1_000_000, "array payload too large");
                let mut values = vec![];
                for i in 0..count {
                    let mut v = 0u64;
                    for j in 0..width {
                        let off = i * width + j;
                        let bits = unit(4 + off / 2)?;
                        v |= u64::from((bits >> (8 * (off % 2))) as u8) << (8 * j);
                    }
                    values.push(if width == 8 {
                        wide(v)
                    } else {
                        vec![Word::Bits(v as u32)]
                    });
                }
                let Data::Array {
                    element,
                    values: target,
                } = &mut self.heap.get_mut(r!(a))?.data
                else {
                    bail!("fill-array-data on non-array");
                };
                let expected_width = match element.as_str() {
                    "Z" | "B" => 1,
                    "C" | "S" => 2,
                    "I" | "F" => 4,
                    "J" | "D" => 8,
                    _ => bail!("fill-array-data requires primitive array"),
                };
                ensure!(
                    width == expected_width,
                    "array payload element width mismatch"
                );
                if target.len() < count {
                    return Err(fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "fill-array-data exceeds array length",
                    ));
                }
                target[..count].clone_from_slice(&values);
                next = 3;
            }
            0x27 => {
                let object = r!(a);
                return Err(self.throw_reference(object)?);
            }
            0x28 => return self.jump(f, (word >> 8) as u8 as i8 as i32),
            0x29 => return self.jump(f, u!(1) as i16 as i32),
            0x2a => return self.jump(f, (u32::from(u!(1)) | (u32::from(u!(2)) << 16)) as i32),
            0x2b | 0x2c => {
                let key = int!(a);
                let offset = (u32::from(u!(1)) | (u32::from(u!(2)) << 16)) as i32;
                let Flow::Jump(pc) = self.jump(f, offset)? else {
                    unreachable!()
                };
                let code = &self.frames[f].code.instructions;
                let unit = |n| -> Result<u16> {
                    code.get(pc + n)
                        .copied()
                        .context("truncated switch payload")
                };
                ensure!(
                    unit(0)? == if op == 0x2b { 0x100 } else { 0x200 },
                    "invalid switch payload kind"
                );
                let n = usize::from(unit(1)?);
                let i32at = |at| -> Result<i32> {
                    Ok((u32::from(unit(at)?) | (u32::from(unit(at + 1)?) << 16)) as i32)
                };
                let index = if op == 0x2b {
                    let first = i32at(2)?;
                    let idx = i64::from(key) - i64::from(first);
                    if idx >= 0 && idx < n as i64 {
                        Some(idx as usize)
                    } else {
                        None
                    }
                } else {
                    let mut found = None;
                    for i in 0..n {
                        if i32at(2 + i * 2)? == key {
                            found = Some(i);
                            break;
                        }
                    }
                    found
                };
                if let Some(i) = index {
                    return self.jump(
                        f,
                        i32at(if op == 0x2b {
                            4 + i * 2
                        } else {
                            2 + n * 2 + i * 2
                        })?,
                    );
                }
                next = 3;
            }
            0x2d..=0x31 => {
                let p = u!(1);
                let b = usize::from(p as u8);
                let c = usize::from(p >> 8);
                let result = if op == 0x31 {
                    (self.pair(f, b)? as i64).cmp(&(self.pair(f, c)? as i64)) as i32
                } else {
                    let (x, y) = if op <= 0x2e {
                        (
                            f64::from(f32::from_bits(int!(b) as u32)),
                            f64::from(f32::from_bits(int!(c) as u32)),
                        )
                    } else {
                        (
                            f64::from_bits(self.pair(f, b)?),
                            f64::from_bits(self.pair(f, c)?),
                        )
                    };
                    if x.is_nan() || y.is_nan() {
                        if op == 0x2e || op == 0x30 { 1 } else { -1 }
                    } else if x < y {
                        -1
                    } else if x > y {
                        1
                    } else {
                        0
                    }
                };
                put!(a, Word::from(result));
                next = 2;
            }
            0x32..=0x3d => {
                let (x, y) = if op < 0x38 {
                    (r!(lo), r!(hi))
                } else {
                    (r!(a), Word::ZERO)
                };
                let cond = match (op - 0x32) % 6 {
                    0 => x == y,
                    1 => x != y,
                    2 => x.int()? < y.int()?,
                    3 => x.int()? >= y.int()?,
                    4 => x.int()? > y.int()?,
                    _ => x.int()? <= y.int()?,
                };
                if cond {
                    return self.jump(f, u!(1) as i16 as i32);
                }
                next = 2;
            }
            0x44..=0x51 => {
                let p = u!(1);
                let array = r!(usize::from(p as u8));
                let Data::Array { element, values } = &self.heap.get(array)?.data else {
                    bail!("array access on non-array");
                };
                let element = element.clone();
                let length = values.len();
                let index = usize::try_from(int!(usize::from(p >> 8))).map_err(|_| {
                    fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "negative array index",
                    )
                })?;
                if index >= length {
                    return Err(fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "array index out of bounds",
                    ));
                }
                let get = op <= 0x4a;
                let sub = if get { op - 0x44 } else { op - 0x4b };
                let compatible = match sub {
                    0 => matches!(element.as_str(), "I" | "F"),
                    1 => matches!(element.as_str(), "J" | "D"),
                    2 => element.starts_with(['L', '[']),
                    3 => element == "Z",
                    4 => element == "B",
                    5 => element == "C",
                    6 => element == "S",
                    _ => false,
                };
                ensure!(
                    compatible,
                    "array opcode does not match element type {element}"
                );
                if get {
                    let Data::Array { values, .. } = &self.heap.get(array)?.data else {
                        bail!("aget on non-array");
                    };
                    let mut value = values
                        .get(index)
                        .ok_or_else(|| {
                            fault(
                                "Ljava/lang/ArrayIndexOutOfBoundsException;",
                                "array index out of bounds",
                            )
                        })?
                        .clone();
                    if sub >= 3 {
                        let n = value[0].int()?;
                        value[0] = Word::from(match sub {
                            3 => i32::from(n != 0),
                            4 => n as i8 as i32,
                            5 => n as u16 as i32,
                            _ => n as i16 as i32,
                        });
                    }
                    self.put(f, a, &value)?;
                } else {
                    let mut value = if sub == 1 {
                        vec![r!(a), r!(a + 1)]
                    } else {
                        vec![r!(a)]
                    };
                    if sub >= 3 {
                        let n = value[0].int()?;
                        value[0] = Word::from(match sub {
                            3 => i32::from(n != 0),
                            4 => n as i8 as i32,
                            5 => n as u16 as i32,
                            _ => n as i16 as i32,
                        });
                    }
                    self.array_value(&element, &value)?;
                    let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
                        bail!("aput on non-array");
                    };
                    *values.get_mut(index).ok_or_else(|| {
                        fault(
                            "Ljava/lang/ArrayIndexOutOfBoundsException;",
                            "array index out of bounds",
                        )
                    })? = value;
                }
                next = 2;
            }
            0x52..=0x6d => {
                let field = self.apk.dex[d]
                    .fields
                    .get(usize::from(u!(1)))
                    .context("invalid field index")?
                    .clone();
                let static_field = op >= 0x60;
                let field = self.resolve_field(&field, static_field)?;
                let key = field.key();
                let get = if static_field { op <= 0x66 } else { op <= 0x58 };
                let dest = if static_field { a } else { lo };
                if static_field {
                    self.initialize(&field.class)?;
                } else {
                    ensure!(
                        self.is_a(&self.heap.get(r!(hi))?.class, &field.class),
                        "field receiver is not an instance of {}",
                        field.class
                    );
                }
                if get {
                    let value = if let Some(primitive) = self.primitive_field(&field) {
                        vec![self.class_object(primitive)?]
                    } else if self.sdk_field(&field) {
                        vec![Word::from(crate::framework::SDK_INT)]
                    } else if self.collections_empty_list_field(&field) {
                        let value = if let Some(value) = self
                            .statics
                            .get(&key)
                            .and_then(|values| values.first())
                            .copied()
                        {
                            value
                        } else {
                            let list = self.heap.instance("Ljava/util/ArrayList;")?;
                            self.invoke(
                                Method {
                                    class: "Ljava/util/ArrayList;".into(),
                                    name: "<init>".into(),
                                    parameters: vec![],
                                    returns: "V".into(),
                                },
                                vec![list],
                                true,
                            )?;
                            let wrapped = self.invoke(
                                Method {
                                    class: "Ljava/util/Collections;".into(),
                                    name: "unmodifiableList".into(),
                                    parameters: vec!["Ljava/util/List;".into()],
                                    returns: "Ljava/util/List;".into(),
                                },
                                vec![list],
                                false,
                            )?;
                            let value = wrapped
                                .first()
                                .copied()
                                .context("Collections.unmodifiableList returned no value")?;
                            self.statics.insert(key.clone(), vec![value]);
                            value
                        };
                        vec![value]
                    } else if self.view_outline_provider_field(&field) {
                        vec![self.view_outline_provider_object(&field)?]
                    } else if self.text_truncate_at_field(&field) {
                        vec![self.text_truncate_at_object(&field)?]
                    } else if let Some(unit) = self.time_unit_field(&field) {
                        vec![self.time_unit_object(unit)?]
                    } else if crate::framework::graphics_enum_names(&field.class).is_some() {
                        vec![self.graphics_enum_object(&field)?]
                    } else if static_field {
                        self.statics
                            .get(&key)
                            .cloned()
                            .unwrap_or_else(|| default_value(&field.ty))
                    } else {
                        self.heap
                            .get(r!(hi))?
                            .fields
                            .get(&key)
                            .cloned()
                            .unwrap_or_else(|| default_value(&field.ty))
                    };
                    self.put(f, dest, &value)?;
                } else {
                    if self.primitive_field(&field).is_some()
                        || self.sdk_field(&field)
                        || self.collections_empty_list_field(&field)
                        || self.view_outline_provider_field(&field)
                        || self.time_unit_field(&field).is_some()
                        || crate::framework::graphics_enum_names(&field.class).is_some()
                    {
                        return Err(fault(
                            "Ljava/lang/IllegalAccessError;",
                            format!("cannot write final field {key}"),
                        ));
                    }
                    let value = if field.ty == "J" || field.ty == "D" {
                        vec![r!(dest), r!(dest + 1)]
                    } else {
                        vec![r!(dest)]
                    };
                    if static_field {
                        self.statics.insert(key, value);
                    } else {
                        self.heap.get_mut(r!(hi))?.fields.insert(key, value);
                    }
                }
                next = 2;
            }
            0x6e..=0x72 | 0x74..=0x78 => {
                let method = self.apk.dex[d]
                    .methods
                    .get(usize::from(u!(1)))
                    .context("invalid invoke method index")?
                    .clone();
                let regs = self.invoke_registers(f, op >= 0x74)?;
                let args = regs
                    .iter()
                    .map(|n| self.reg(f, *n))
                    .collect::<Result<Vec<_>>>()?;
                match self.begin_invoke(method, args, matches!(op, 0x6e | 0x72 | 0x74 | 0x78))? {
                    Some(words) => self.frames[f].result = words,
                    None => {
                        let return_pc = self.frames[f].pc + 3;
                        self.frames
                            .last_mut()
                            .context("missing invoked frame")?
                            .return_pc
                            .get_or_insert(return_pc);
                        return Ok(Flow::Call);
                    }
                }
                next = 3;
            }
            0x7b..=0x8f => {
                let b = hi;
                let dest = lo;
                let value = match op {
                    0x7b => vec![Word::from(int!(b).wrapping_neg())],
                    0x7c => vec![Word::from(!int!(b))],
                    0x7d => wide((self.pair(f, b)? as i64).wrapping_neg() as u64),
                    0x7e => wide(!self.pair(f, b)?),
                    0x7f => vec![Word::Bits((-f32::from_bits(int!(b) as u32)).to_bits())],
                    0x80 => wide((-f64::from_bits(self.pair(f, b)?)).to_bits()),
                    0x81 => wide(int!(b) as i64 as u64),
                    0x82 => vec![Word::Bits((int!(b) as f32).to_bits())],
                    0x83 => wide((int!(b) as f64).to_bits()),
                    0x84 => vec![Word::Bits(self.pair(f, b)? as u32)],
                    0x85 => vec![Word::Bits((self.pair(f, b)? as i64 as f32).to_bits())],
                    0x86 => wide((self.pair(f, b)? as i64 as f64).to_bits()),
                    0x87 => vec![Word::from(f32::from_bits(int!(b) as u32) as i32)],
                    0x88 => wide(f32::from_bits(int!(b) as u32) as i64 as u64),
                    0x89 => wide(f64::from(f32::from_bits(int!(b) as u32)).to_bits()),
                    0x8a => vec![Word::from(f64::from_bits(self.pair(f, b)?) as i32)],
                    0x8b => wide(f64::from_bits(self.pair(f, b)?) as i64 as u64),
                    0x8c => vec![Word::Bits(
                        (f64::from_bits(self.pair(f, b)?) as f32).to_bits(),
                    )],
                    0x8d => vec![Word::from(int!(b) as i8 as i32)],
                    0x8e => vec![Word::from(int!(b) as u16 as i32)],
                    _ => vec![Word::from(int!(b) as i16 as i32)],
                };
                self.put(f, dest, &value)?;
            }
            0x90..=0xcf => {
                let (operation, dest, b, c) = if op >= 0xb0 {
                    (op - 0x20, lo, lo, hi)
                } else {
                    let p = u!(1);
                    next = 2;
                    (op, a, usize::from(p as u8), usize::from(p >> 8))
                };
                let value = if operation <= 0x9a {
                    vec![Word::from(int_op(operation - 0x90, int!(b), int!(c))?)]
                } else if operation <= 0xa5 {
                    let x = self.pair(f, b)?;
                    let y = if operation >= 0xa3 {
                        int!(c) as u64
                    } else {
                        self.pair(f, c)?
                    };
                    wide(long_op(operation - 0x9b, x, y)?)
                } else if operation <= 0xaa {
                    vec![Word::Bits(
                        float_op(
                            operation - 0xa6,
                            f32::from_bits(int!(b) as u32),
                            f32::from_bits(int!(c) as u32),
                        )
                        .to_bits(),
                    )]
                } else {
                    wide(
                        double_op(
                            operation - 0xab,
                            f64::from_bits(self.pair(f, b)?),
                            f64::from_bits(self.pair(f, c)?),
                        )
                        .to_bits(),
                    )
                };
                self.put(f, dest, &value)?;
            }
            0xd0..=0xe2 => {
                let (dest, b, literal, operation) = if op <= 0xd7 {
                    (lo, hi, u!(1) as i16 as i32, op - 0xd0)
                } else {
                    let p = u!(1);
                    (
                        a,
                        usize::from(p as u8),
                        (p >> 8) as u8 as i8 as i32,
                        op - 0xd8,
                    )
                };
                let x = int!(b);
                let value = if operation == 1 {
                    literal.wrapping_sub(x)
                } else {
                    int_op(operation, x, literal)?
                };
                put!(dest, Word::from(value));
                next = 2;
            }
            _ => bail!("unsupported DEX opcode 0x{op:02x}"),
        }
        Ok(Flow::Next(next))
    }
    fn invoke_registers(&self, f: usize, range: bool) -> Result<Vec<usize>> {
        let word = self.unit(f, 0)?;
        let count = usize::from(if range { word >> 8 } else { word >> 12 });
        let p = self.unit(f, 2)?;
        if range {
            let start = usize::from(p);
            Ok((start..start + count).collect())
        } else {
            ensure!(count <= 5, "invoke register count exceeds five");
            let all = [
                usize::from(p & 15),
                usize::from((p >> 4) & 15),
                usize::from((p >> 8) & 15),
                usize::from(p >> 12),
                usize::from((word >> 8) & 15),
            ];
            Ok(all[..count].to_vec())
        }
    }
}

fn int_op(op: u8, x: i32, y: i32) -> Result<i32> {
    Ok(match op {
        0 => x.wrapping_add(y),
        1 => x.wrapping_sub(y),
        2 => x.wrapping_mul(y),
        3 => {
            if y == 0 {
                return Err(fault("Ljava/lang/ArithmeticException;", "divide by zero"));
            }
            x.wrapping_div(y)
        }
        4 => {
            if y == 0 {
                return Err(fault(
                    "Ljava/lang/ArithmeticException;",
                    "remainder by zero",
                ));
            }
            x.wrapping_rem(y)
        }
        5 => x & y,
        6 => x | y,
        7 => x ^ y,
        8 => x.wrapping_shl(y as u32 & 31),
        9 => x.wrapping_shr(y as u32 & 31),
        10 => ((x as u32).wrapping_shr(y as u32 & 31)) as i32,
        _ => bail!("invalid integer operation"),
    })
}
fn long_op(op: u8, x: u64, y: u64) -> Result<u64> {
    Ok(match op {
        0 => x.wrapping_add(y),
        1 => x.wrapping_sub(y),
        2 => x.wrapping_mul(y),
        3 => {
            if y == 0 {
                return Err(fault("Ljava/lang/ArithmeticException;", "divide by zero"));
            }
            (x as i64).wrapping_div(y as i64) as u64
        }
        4 => {
            if y == 0 {
                return Err(fault(
                    "Ljava/lang/ArithmeticException;",
                    "remainder by zero",
                ));
            }
            (x as i64).wrapping_rem(y as i64) as u64
        }
        5 => x & y,
        6 => x | y,
        7 => x ^ y,
        8 => x.wrapping_shl(y as u32 & 63),
        9 => (x as i64).wrapping_shr(y as u32 & 63) as u64,
        10 => x.wrapping_shr(y as u32 & 63),
        _ => bail!("invalid long operation"),
    })
}
fn float_op(op: u8, x: f32, y: f32) -> f32 {
    match op {
        0 => x + y,
        1 => x - y,
        2 => x * y,
        3 => x / y,
        _ => x % y,
    }
}
fn double_op(op: u8, x: f64, y: f64) -> f64 {
    match op {
        0 => x + y,
        1 => x - y,
        2 => x * y,
        3 => x / y,
        _ => x % y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::{apk::Apk, dex::Method};

    #[test]
    fn sliced_calls_returns_exceptions_gc_and_terminal_unwinding() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
        )
        .unwrap();
        let method = |name: &str, parameters: &[&str], returns: &str| Method {
            class: "Lorg/droidless/counter/FrameContract;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        };
        fn sliced(vm: &mut Runtime, method: Method, args: Vec<Word>) -> Result<Vec<Word>> {
            assert!(vm.begin_invoke(method, args, false)?.is_none());
            let before = vm.instructions;
            assert!(vm.execute_slice(0, 0)?.is_none());
            assert_eq!(vm.instructions, before);
            let mut nested = false;
            for _ in 0..10_000 {
                if let Some(words) = vm.execute_slice(0, 1)? {
                    assert!(nested, "compiled method never entered its callee");
                    assert_eq!(vm.stack_depth(), 0);
                    return Ok(words);
                }
                nested |= vm.stack_depth() > 1;
                vm.collect();
            }
            panic!("authored frame contract did not terminate");
        }
        let value = vm.heap.string("slice".into()).unwrap();
        let words = sliced(
            &mut vm,
            method("nested", &["I", "Ljava/lang/String;"], "Ljava/lang/String;"),
            vec![Word::from(3), value],
        )
        .unwrap();
        assert_eq!(vm.heap.text(words[0]).unwrap(), "slice/leaf!!!");
        let mut args = vec![Word::from(4)];
        args.extend(wide(100));
        assert_eq!(
            bits64(&sliced(&mut vm, method("wide", &["I", "J"], "J"), args).unwrap()).unwrap(),
            114
        );
        let words = sliced(&mut vm, method("caught", &[], "Ljava/lang/String;"), vec![]).unwrap();
        assert_eq!(vm.heap.text(words[0]).unwrap(), "stack failure");
        let error = sliced(
            &mut vm,
            method("throwNested", &["I"], "V"),
            vec![Word::from(4)],
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("stack failure"));
        assert_eq!(vm.stack_depth(), 0);
        let error = sliced(
            &mut vm,
            method("unsupported", &["I"], "V"),
            vec![Word::from(4)],
        )
        .unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("System;->exit"));
        assert_eq!(
            message
                .matches("FrameContract;->unsupported(I)V [classes.dex, PC")
                .count(),
            5
        );
        assert_eq!(vm.stack_depth(), 0);
        let error = sliced(
            &mut vm,
            method("wide", &["I", "J"], "J"),
            vec![Word::from(200), Word::ZERO, Word::ZERO],
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("guest call stack limit"));
        assert_eq!(vm.stack_depth(), 0);
        let words = vm
            .invoke(method("contract", &[], "I"), vec![], false)
            .unwrap();
        assert_eq!(words, vec![Word::from(1)]);
    }
    #[test]
    fn dalvik_arithmetic_edges() {
        assert_eq!(int_op(0, i32::MAX, 1).unwrap(), i32::MIN);
        assert_eq!(int_op(3, i32::MIN, -1).unwrap(), i32::MIN);
        assert_eq!(int_op(10, -1, 1).unwrap(), i32::MAX);
        assert_eq!(int_op(8, 1, 33).unwrap(), 2);
        assert!(int_op(3, 1, 0).is_err());
        assert_eq!(long_op(9, i64::MIN as u64, 63).unwrap(), u64::MAX);
        assert!(double_op(3, 0.0, 0.0).is_nan());
    }
}
