use crate::{
    binary::{Bytes, string_at, string_pool},
    xml::Value,
};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Resource {
    pub id: u32,
    pub name: String,
    pub value: Option<Value>,
    pub bag: BTreeMap<u32, Value>,
}
#[derive(Default, Debug, Serialize)]
pub struct Resources {
    pub entries: BTreeMap<u32, Resource>,
}
impl Resources {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let root = Bytes(data).chunk(0)?;
        ensure!(
            root.kind == 2 && root.header >= 12 && root.bytes.0.len() == data.len(),
            "invalid resource table"
        );
        let children = root.children()?;
        let pool = children
            .iter()
            .find(|c| c.kind == 1)
            .ok_or_else(|| anyhow::anyhow!("missing resource string pool"))?;
        let strings = string_pool(pool)?;
        let mut result = Self::default();
        for package in children.iter().filter(|c| c.kind == 0x200) {
            ensure!(package.header >= 284, "invalid resource package header");
            let p = package.bytes;
            let package_id = p.u32(8)?;
            ensure!(package_id <= 255, "invalid resource package ID");
            let types = string_pool(&p.chunk(p.u32(268)? as usize)?)?;
            let keys = string_pool(&p.chunk(p.u32(276)? as usize)?)?;
            let type_offset = if package.header >= 288 {
                p.u32(284)?
            } else {
                0
            };
            for chunk in package.children()?.iter().filter(|c| c.kind == 0x201) {
                let b = chunk.bytes;
                ensure!(chunk.header >= 24, "invalid resource type header");
                let tid = u32::from(b.u8(8)?);
                ensure!(
                    tid > 0 && tid + type_offset <= 255,
                    "invalid resource type ID"
                );
                let type_name = string_at(&types, tid - 1)?;
                let flags = b.u8(9)?;
                ensure!(
                    flags & !3 == 0 && flags != 3,
                    "unsupported resource type flags"
                );
                let count = b.u32(12)? as usize;
                let start = b.u32(16)? as usize;
                ensure!(
                    count <= 65536 && start >= chunk.header,
                    "invalid resource entries"
                );
                let config_size = b.u32(20)? as usize;
                ensure!(
                    config_size >= 4 && 20 + config_size <= chunk.header,
                    "invalid resource configuration"
                );
                let default = b.slice(24, config_size - 4)?.iter().all(|v| *v == 0);
                let stride = if flags == 2 { 2 } else { 4 };
                b.table(chunk.header, count, stride)?;
                ensure!(
                    start >= chunk.header + count * stride,
                    "resource data overlaps offsets"
                );
                for i in 0..count {
                    let (entry_idx, offset) = if flags == 1 {
                        (
                            u32::from(b.u16(chunk.header + i * 4)?),
                            u32::from(b.u16(chunk.header + i * 4 + 2)?) * 4,
                        )
                    } else if flags == 2 {
                        (
                            i as u32,
                            match b.u16(chunk.header + i * 2)? {
                                0xffff => u32::MAX,
                                v => u32::from(v) * 4,
                            },
                        )
                    } else {
                        (i as u32, b.u32(chunk.header + i * 4)?)
                    };
                    if offset == u32::MAX {
                        continue;
                    }
                    let at = start
                        .checked_add(offset as usize)
                        .ok_or_else(|| anyhow::anyhow!("resource offset overflow"))?;
                    let size = usize::from(b.u16(at)?);
                    let entry_flags = b.u16(at + 2)?;
                    ensure!(
                        size >= 8 && entry_flags & 8 == 0,
                        "compact resource entries unsupported"
                    );
                    b.slice(at, size)?;
                    let name = format!("{type_name}/{}", string_at(&keys, b.u32(at + 4)?)?);
                    let id = (package_id << 24) | ((tid + type_offset) << 16) | entry_idx;
                    let mut resource = Resource {
                        id,
                        name,
                        value: None,
                        bag: BTreeMap::new(),
                    };
                    if entry_flags & 1 == 0 {
                        resource.value = Some(value(b, at + size, &strings)?);
                    } else {
                        ensure!(size >= 16, "invalid resource map entry");
                        let count = b.u32(at + 12)? as usize;
                        b.table(at + size, count, 12)?;
                        for j in 0..count {
                            let v = at + size + j * 12;
                            resource.bag.insert(b.u32(v)?, value(b, v + 4, &strings)?);
                        }
                    }
                    // ponytail: prefer default config, otherwise first variant; add qualifier matching when a tested APK needs it.
                    if default || !result.entries.contains_key(&id) {
                        result.entries.insert(id, resource);
                    }
                }
            }
        }
        Ok(result)
    }
    pub fn resolve(&self, id: u32) -> Result<&Value> {
        let mut current = id;
        for _ in 0..32 {
            let v = self
                .entries
                .get(&current)
                .and_then(|e| e.value.as_ref())
                .ok_or_else(|| anyhow::anyhow!("resource @0x{current:08x} missing or complex"))?;
            if v.kind != 1 {
                return Ok(v);
            }
            current = v.data;
        }
        anyhow::bail!("resource reference cycle at @0x{id:08x}")
    }
    pub fn text(&self, id: u32) -> Result<String> {
        Ok(self.resolve(id)?.display())
    }
}
fn value(b: Bytes<'_>, at: usize, strings: &[String]) -> Result<Value> {
    ensure!(b.u16(at)? == 8, "invalid resource value size");
    let kind = b.u8(at + 3)?;
    let data = b.u32(at + 4)?;
    let text = if kind == 3 {
        Some(string_at(strings, data)?.to_owned())
    } else {
        None
    };
    Ok(Value { kind, data, text })
}
