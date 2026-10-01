use crate::binary::{Bytes, string_at};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Field {
    pub class: String,
    pub name: String,
    pub ty: String,
}
impl Field {
    pub fn key(&self) -> String {
        format!("{}->{}:{}", self.class, self.name, self.ty)
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Method {
    pub class: String,
    pub name: String,
    pub parameters: Vec<String>,
    pub returns: String,
}
impl Method {
    pub fn signature(&self) -> String {
        format!(
            "{}({}){}",
            self.name,
            self.parameters.concat(),
            self.returns
        )
    }
    pub fn key(&self) -> String {
        format!("{}->{}", self.class, self.signature())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Handler {
    pub start: u32,
    pub end: u32,
    pub catches: Vec<(Option<String>, u32)>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Code {
    pub registers: u16,
    pub ins: u16,
    pub outs: u16,
    pub instructions: Vec<u16>,
    pub handlers: Vec<Handler>,
    pub debug_offset: u32,
}
#[derive(Clone, Debug, Serialize)]
pub struct EncodedMethod {
    pub index: usize,
    pub access: u32,
    pub code: Option<Code>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<Annotation>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Annotation {
    pub visibility: u8,
    pub class: String,
    pub values: Vec<(String, EncodedValue)>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Class {
    pub name: String,
    pub access: u32,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<EncodedMethod>,
    pub static_fields: Vec<usize>,
    pub instance_fields: Vec<usize>,
    pub static_values: Vec<EncodedValue>,
}
#[derive(Clone, Debug, Serialize)]
pub enum EncodedValue {
    Bits(u64),
    String(String),
    Null,
    Array(Vec<EncodedValue>),
    Type(String),
    Field(Field),
    Method(Method),
    Enum { class: String, name: String },
    Annotation(Box<Annotation>),
}
#[derive(Debug, Serialize)]
pub struct Dex {
    pub version: String,
    pub strings: Vec<String>,
    pub types: Vec<String>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub classes: Vec<Class>,
}

impl Dex {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let b = Bytes(data);
        ensure!(
            b.slice(0, 4)? == b"dex\n" && b.u8(7)? == 0,
            "invalid DEX magic"
        );
        let version = std::str::from_utf8(b.slice(4, 3)?)?.to_owned();
        ensure!(
            ["035", "037", "038", "039", "040"].contains(&version.as_str()),
            "unsupported DEX version {version} (container DEX 041 is not supported)"
        );
        ensure!(
            b.u32(32)? as usize == data.len() && b.u32(36)? == 112 && b.u32(40)? == 0x12345678,
            "invalid DEX size/header/endianness"
        );
        ensure!(
            Sha1::digest(b.slice(32, data.len() - 32)?).as_slice() == b.slice(12, 20)?,
            "DEX SHA-1 mismatch"
        );
        let (mut a, mut z) = (1u32, 0u32);
        for byte in b.slice(12, data.len() - 12)? {
            a = (a + u32::from(*byte)) % 65521;
            z = (z + a) % 65521;
        }
        ensure!(b.u32(8)? == (z << 16) | a, "DEX Adler-32 mismatch");
        let table = |header, stride| -> Result<(usize, usize)> {
            let count = b.u32(header)? as usize;
            let at = b.u32(header + 4)? as usize;
            ensure!(count == 0 || at >= 112, "DEX table points into header");
            b.table(at, count, stride)?;
            Ok((count, at))
        };
        let map = b.u32(52)? as usize;
        ensure!(map >= 112, "missing DEX map");
        b.table(map + 4, b.u32(map)? as usize, 12)?;
        let data_off = b.u32(108)? as usize;
        b.slice(data_off, b.u32(104)? as usize)?;
        let (count, at) = table(56, 4)?;
        let strings = (0..count)
            .map(|i| mutf8(b, b.u32(at + i * 4)? as usize))
            .collect::<Result<Vec<_>>>()?;
        let (count, at) = table(64, 4)?;
        let types = (0..count)
            .map(|i| Ok(string_at(&strings, b.u32(at + i * 4)?)?.to_owned()))
            .collect::<Result<Vec<_>>>()?;
        let ty = |idx: u32| -> Result<String> {
            types
                .get(idx as usize)
                .cloned()
                .context("invalid DEX type index")
        };
        let type_list = |at: usize| -> Result<Vec<String>> {
            if at == 0 {
                return Ok(vec![]);
            }
            let n = b.u32(at)? as usize;
            b.table(at + 4, n, 2)?;
            (0..n)
                .map(|i| ty(u32::from(b.u16(at + 4 + i * 2)?)))
                .collect()
        };
        let (count, at) = table(72, 12)?;
        let protos = (0..count)
            .map(|i| {
                let p = at + i * 12;
                string_at(&strings, b.u32(p)?)?;
                Ok((ty(b.u32(p + 4)?)?, type_list(b.u32(p + 8)? as usize)?))
            })
            .collect::<Result<Vec<_>>>()?;
        let (count, at) = table(80, 8)?;
        let fields = (0..count)
            .map(|i| {
                let p = at + i * 8;
                Ok(Field {
                    class: ty(u32::from(b.u16(p)?))?,
                    ty: ty(u32::from(b.u16(p + 2)?))?,
                    name: string_at(&strings, b.u32(p + 4)?)?.to_owned(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let (count, at) = table(88, 8)?;
        let methods = (0..count)
            .map(|i| {
                let p = at + i * 8;
                let (returns, parameters) = protos
                    .get(usize::from(b.u16(p + 2)?))
                    .context("invalid DEX proto index")?;
                Ok(Method {
                    class: ty(u32::from(b.u16(p)?))?,
                    name: string_at(&strings, b.u32(p + 4)?)?.to_owned(),
                    parameters: parameters.clone(),
                    returns: returns.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let (count, at) = table(96, 32)?;
        let mut classes = vec![];
        for i in 0..count {
            let p = at + i * 32;
            let name = ty(b.u32(p)?)?;
            ensure!(
                !classes.iter().any(|c: &Class| c.name == name),
                "duplicate DEX class"
            );
            let super_idx = b.u32(p + 8)?;
            let super_class = if super_idx == u32::MAX {
                None
            } else {
                Some(ty(super_idx)?)
            };
            let source_idx = b.u32(p + 16)?;
            if source_idx != u32::MAX {
                string_at(&strings, source_idx)?;
            }
            let annotations = b.u32(p + 20)? as usize;
            let mut method_annotations = if annotations == 0 {
                BTreeMap::new()
            } else {
                parse_method_annotations(
                    b,
                    annotations,
                    &strings,
                    &types,
                    &fields,
                    &methods,
                    &name,
                )?
            };
            let mut class = Class {
                name,
                access: b.u32(p + 4)?,
                super_class,
                interfaces: type_list(b.u32(p + 12)? as usize)?,
                methods: vec![],
                static_fields: vec![],
                instance_fields: vec![],
                static_values: vec![],
            };
            let mut pos = b.u32(p + 24)? as usize;
            if pos != 0 {
                let statics = b.uleb(&mut pos)? as usize;
                let instances = b.uleb(&mut pos)? as usize;
                let direct = b.uleb(&mut pos)? as usize;
                let virtuals = b.uleb(&mut pos)? as usize;
                ensure!(
                    statics + instances <= fields.len() && direct + virtuals <= methods.len(),
                    "invalid class member counts"
                );
                for (n, list) in [
                    (statics, &mut class.static_fields),
                    (instances, &mut class.instance_fields),
                ] {
                    let mut idx = 0usize;
                    for j in 0..n {
                        let diff = b.uleb(&mut pos)? as usize;
                        ensure!(j == 0 || diff != 0, "duplicate encoded field");
                        idx = idx.checked_add(diff).context("field index overflow")?;
                        ensure!(
                            fields.get(idx).is_some_and(|f| f.class == class.name),
                            "invalid encoded field reference"
                        );
                        b.uleb(&mut pos)?;
                        list.push(idx);
                    }
                }
                for n in [direct, virtuals] {
                    let mut idx = 0usize;
                    for j in 0..n {
                        let diff = b.uleb(&mut pos)? as usize;
                        ensure!(j == 0 || diff != 0, "duplicate encoded method");
                        idx = idx.checked_add(diff).context("method index overflow")?;
                        ensure!(
                            methods.get(idx).is_some_and(|m| m.class == class.name),
                            "invalid encoded method reference"
                        );
                        let access = b.uleb(&mut pos)?;
                        let code_at = b.uleb(&mut pos)? as usize;
                        let code = if code_at == 0 {
                            None
                        } else {
                            Some(
                                parse_code(b, code_at, &types)
                                    .with_context(|| methods[idx].key())?,
                            )
                        };
                        class.methods.push(EncodedMethod {
                            index: idx,
                            access,
                            code,
                            annotations: method_annotations.remove(&idx).unwrap_or_default(),
                        });
                    }
                }
            }
            let mut pos = b.u32(p + 28)? as usize;
            if pos != 0 {
                let count = b.uleb(&mut pos)? as usize;
                ensure!(count <= class.static_fields.len(), "too many static values");
                for _ in 0..count {
                    class.static_values.push(encoded_value(
                        b, &mut pos, &strings, &types, &fields, &methods, 0,
                    )?);
                }
            }
            classes.push(class);
        }
        Ok(Self {
            version,
            strings,
            types,
            fields,
            methods,
            classes,
        })
    }
}

fn parse_method_annotations(
    b: Bytes<'_>,
    at: usize,
    strings: &[String],
    types: &[String],
    fields: &[Field],
    methods: &[Method],
    owner: &str,
) -> Result<BTreeMap<usize, Vec<Annotation>>> {
    b.slice(at, 16)?;
    let class_annotations = b.u32(at)? as usize;
    let field_count = b.u32(at + 4)? as usize;
    let method_count = b.u32(at + 8)? as usize;
    let parameter_count = b.u32(at + 12)? as usize;
    let entries = field_count
        .checked_add(method_count)
        .and_then(|count| count.checked_add(parameter_count))
        .context("annotation directory count overflow")?;
    ensure!(entries <= 1_000_000, "annotation directory limit reached");
    b.table(at + 16, entries, 8)?;
    if class_annotations != 0 {
        let _ = annotation_set(b, class_annotations, strings, types, fields, methods)?;
    }
    let mut pos = at + 16 + field_count * 8;
    let mut annotated = BTreeMap::new();
    for _ in 0..method_count {
        let index = b.u32(pos)? as usize;
        let set = b.u32(pos + 4)? as usize;
        pos += 8;
        ensure!(
            methods
                .get(index)
                .is_some_and(|method| method.class == owner),
            "invalid annotated method index"
        );
        ensure!(
            annotated
                .insert(
                    index,
                    annotation_set(b, set, strings, types, fields, methods)?
                )
                .is_none(),
            "duplicate method annotation entry"
        );
    }
    for _ in 0..parameter_count {
        let index = b.u32(pos)? as usize;
        let references = b.u32(pos + 4)? as usize;
        pos += 8;
        ensure!(
            methods
                .get(index)
                .is_some_and(|method| method.class == owner),
            "invalid parameter annotation method index"
        );
        let count = b.u32(references)? as usize;
        ensure!(count <= 1_000_000, "parameter annotation limit reached");
        b.table(references + 4, count, 4)?;
    }
    Ok(annotated)
}

fn annotation_set(
    b: Bytes<'_>,
    at: usize,
    strings: &[String],
    types: &[String],
    fields: &[Field],
    methods: &[Method],
) -> Result<Vec<Annotation>> {
    let count = b.u32(at)? as usize;
    ensure!(count <= 1_000_000, "annotation set limit reached");
    b.table(at + 4, count, 4)?;
    let mut annotations = Vec::with_capacity(count);
    for index in 0..count {
        let mut pos = b.u32(at + 4 + index * 4)? as usize;
        let visibility = b.u8(pos)?;
        pos += 1;
        ensure!(visibility <= 2, "invalid annotation visibility");
        let mut annotation = encoded_annotation(b, &mut pos, strings, types, fields, methods, 0)?;
        annotation.visibility = visibility;
        annotations.push(annotation);
    }
    Ok(annotations)
}

fn encoded_annotation(
    b: Bytes<'_>,
    pos: &mut usize,
    strings: &[String],
    types: &[String],
    fields: &[Field],
    methods: &[Method],
    depth: usize,
) -> Result<Annotation> {
    ensure!(depth < 64, "annotation nesting limit");
    let class = types
        .get(b.uleb(pos)? as usize)
        .context("invalid annotation type")?
        .clone();
    let elements = b.uleb(pos)? as usize;
    ensure!(elements <= 65_536, "annotation element limit reached");
    let mut values = Vec::with_capacity(elements);
    for _ in 0..elements {
        let name = string_at(strings, b.uleb(pos)?)?.to_owned();
        let value = encoded_value(b, pos, strings, types, fields, methods, depth + 1)?;
        values.push((name, value));
    }
    Ok(Annotation {
        visibility: 1,
        class,
        values,
    })
}

fn parse_code(b: Bytes<'_>, at: usize, types: &[String]) -> Result<Code> {
    ensure!(at.is_multiple_of(4), "unaligned code item");
    let registers = b.u16(at)?;
    let ins = b.u16(at + 2)?;
    let outs = b.u16(at + 4)?;
    let tries = usize::from(b.u16(at + 6)?);
    let debug_offset = b.u32(at + 8)?;
    if debug_offset != 0 {
        b.slice(debug_offset as usize, 1)?;
    }
    ensure!(registers >= ins, "code has fewer registers than arguments");
    let n = b.u32(at + 12)? as usize;
    b.table(at + 16, n, 2)?;
    let instructions = (0..n)
        .map(|i| b.u16(at + 16 + i * 2))
        .collect::<Result<Vec<_>>>()?;
    let mut handlers = vec![];
    if tries != 0 {
        let start = at + 16 + n * 2 + (n % 2) * 2;
        b.table(start, tries, 8)?;
        let handler_start = start + tries * 8;
        let mut pos = handler_start;
        let count = b.uleb(&mut pos)? as usize;
        ensure!(count <= b.0.len(), "invalid catch count");
        let mut offsets = BTreeMap::new();
        for _ in 0..count {
            let offset = pos - handler_start;
            let size = b.sleb(&mut pos)?;
            let mut catches = vec![];
            ensure!(
                size.unsigned_abs() as usize <= types.len(),
                "invalid handler size"
            );
            for _ in 0..size.unsigned_abs() {
                let ty = types
                    .get(b.uleb(&mut pos)? as usize)
                    .context("invalid catch type")?
                    .clone();
                let pc = b.uleb(&mut pos)?;
                ensure!((pc as usize) < n, "catch PC out of range");
                catches.push((Some(ty), pc));
            }
            if size <= 0 {
                let pc = b.uleb(&mut pos)?;
                ensure!((pc as usize) < n, "catch-all PC out of range");
                catches.push((None, pc));
            }
            offsets.insert(offset, catches);
        }
        for i in 0..tries {
            let p = start + i * 8;
            let start = b.u32(p)?;
            let end = start
                .checked_add(u32::from(b.u16(p + 4)?))
                .context("try range overflow")?;
            ensure!(end as usize <= n, "try range outside code");
            let catches = offsets
                .get(&usize::from(b.u16(p + 6)?))
                .context("invalid handler offset")?
                .clone();
            handlers.push(Handler {
                start,
                end,
                catches,
            });
        }
    }
    Ok(Code {
        registers,
        ins,
        outs,
        instructions,
        handlers,
        debug_offset,
    })
}

fn mutf8(b: Bytes<'_>, mut at: usize) -> Result<String> {
    let length = b.uleb(&mut at)? as usize;
    ensure!(length <= b.0.len(), "invalid DEX string length");
    let mut units = Vec::with_capacity(length);
    loop {
        let c = b.u8(at)?;
        at += 1;
        if c == 0 {
            break;
        }
        let unit = if c < 0x80 {
            u16::from(c)
        } else if c & 0xe0 == 0xc0 {
            let d = b.u8(at)?;
            at += 1;
            ensure!(d & 0xc0 == 0x80, "invalid MUTF-8 continuation");
            let u = (u16::from(c & 31) << 6) | u16::from(d & 63);
            ensure!(u == 0 || u >= 0x80, "overlong MUTF-8");
            u
        } else {
            ensure!(c & 0xf0 == 0xe0, "invalid MUTF-8 lead byte");
            let d = b.u8(at)?;
            let e = b.u8(at + 1)?;
            at += 2;
            ensure!(d & 0xc0 == 0x80 && e & 0xc0 == 0x80, "invalid MUTF-8");
            let u = (u16::from(c & 15) << 12) | (u16::from(d & 63) << 6) | u16::from(e & 63);
            ensure!(u >= 0x800, "overlong MUTF-8");
            u
        };
        units.push(unit);
        ensure!(units.len() <= length, "DEX string exceeds declared length");
    }
    ensure!(units.len() == length, "DEX string length mismatch");
    // Rust cannot represent isolated UTF-16 surrogates; reject instead of changing guest data.
    Ok(String::from_utf16(&units)?)
}

fn encoded_value(
    b: Bytes<'_>,
    pos: &mut usize,
    strings: &[String],
    types: &[String],
    fields: &[Field],
    methods: &[Method],
    depth: usize,
) -> Result<EncodedValue> {
    ensure!(depth < 64, "encoded value nesting limit");
    let tag = b.u8(*pos)?;
    *pos += 1;
    let kind = tag & 31;
    let arg = tag >> 5;
    if kind == 0x1e {
        ensure!(arg == 0, "invalid null value");
        return Ok(EncodedValue::Null);
    }
    if kind == 0x1f {
        ensure!(arg <= 1, "invalid boolean value");
        return Ok(EncodedValue::Bits(u64::from(arg)));
    }
    if kind == 0x1c {
        ensure!(arg == 0, "invalid encoded array");
        let n = b.uleb(pos)? as usize;
        ensure!(
            n <= b.0.len().saturating_sub(*pos),
            "invalid encoded array length"
        );
        return Ok(EncodedValue::Array(
            (0..n)
                .map(|_| encoded_value(b, pos, strings, types, fields, methods, depth + 1))
                .collect::<Result<_>>()?,
        ));
    }
    if kind == 0x1d {
        ensure!(arg == 0, "invalid nested annotation");
        return Ok(EncodedValue::Annotation(Box::new(encoded_annotation(
            b,
            pos,
            strings,
            types,
            fields,
            methods,
            depth + 1,
        )?)));
    }
    let width = usize::from(arg) + 1;
    ensure!(
        match kind {
            0 => width == 1,
            2 | 3 => width <= 2,
            4 | 0x10 | 0x17 | 0x18 => width <= 4,
            6 | 0x11 => width <= 8,
            0x19..=0x1b => width <= 4,
            _ => false,
        },
        "unsupported/invalid encoded value 0x{tag:x}"
    );
    let mut value = 0u64;
    for (i, byte) in b.slice(*pos, width)?.iter().enumerate() {
        value |= u64::from(*byte) << (8 * i);
    }
    *pos += width;
    if kind == 0x17 {
        return Ok(EncodedValue::String(
            string_at(strings, value as u32)?.to_owned(),
        ));
    }
    if kind == 0x18 {
        return Ok(EncodedValue::Type(
            types
                .get(value as usize)
                .context("invalid encoded type")?
                .clone(),
        ));
    }
    if kind == 0x19 {
        return Ok(EncodedValue::Field(
            fields
                .get(value as usize)
                .context("invalid encoded field reference")?
                .clone(),
        ));
    }
    if kind == 0x1a {
        return Ok(EncodedValue::Method(
            methods
                .get(value as usize)
                .context("invalid encoded method reference")?
                .clone(),
        ));
    }
    if kind == 0x1b {
        let field = fields
            .get(value as usize)
            .context("invalid encoded enum field")?;
        return Ok(EncodedValue::Enum {
            class: field.class.clone(),
            name: field.name.clone(),
        });
    }
    if matches!(kind, 0 | 2 | 4 | 6) && width < 8 && value & (1 << (width * 8 - 1)) != 0 {
        value |= u64::MAX << (width * 8);
    }
    if kind == 0x10 {
        value <<= (4 - width) * 8;
    } else if kind == 0x11 {
        value <<= (8 - width) * 8;
    }
    Ok(EncodedValue::Bits(value))
}
