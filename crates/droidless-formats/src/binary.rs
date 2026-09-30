use anyhow::{Result, bail, ensure};

#[derive(Clone, Copy)]
pub struct Bytes<'a>(pub &'a [u8]);

impl<'a> Bytes<'a> {
    pub fn slice(self, at: usize, len: usize) -> Result<&'a [u8]> {
        let end = at
            .checked_add(len)
            .ok_or_else(|| anyhow::anyhow!("offset overflow"))?;
        self.0
            .get(at..end)
            .ok_or_else(|| anyhow::anyhow!("truncated binary at 0x{at:x}, need {len} bytes"))
    }
    pub fn u8(self, at: usize) -> Result<u8> {
        Ok(self.slice(at, 1)?[0])
    }
    pub fn u16(self, at: usize) -> Result<u16> {
        Ok(u16::from_le_bytes(self.slice(at, 2)?.try_into()?))
    }
    pub fn u32(self, at: usize) -> Result<u32> {
        Ok(u32::from_le_bytes(self.slice(at, 4)?.try_into()?))
    }
    pub fn uleb(self, at: &mut usize) -> Result<u32> {
        let mut value = 0;
        for shift in (0..35).step_by(7) {
            let byte = self.u8(*at)?;
            *at += 1;
            ensure!(shift != 28 || byte & 0xf0 == 0, "ULEB128 overflow");
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        bail!("unterminated ULEB128")
    }
    pub fn sleb(self, at: &mut usize) -> Result<i32> {
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = self.u8(*at)?;
            *at += 1;
            if shift == 28 {
                ensure!(byte & 0xf0 == 0 || byte & 0xf0 == 0x70, "SLEB128 overflow");
            }
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                if shift < 28 && byte & 0x40 != 0 {
                    value |= u32::MAX << (shift + 7);
                }
                return Ok(value as i32);
            }
        }
        bail!("unterminated SLEB128")
    }
    pub fn table(self, at: usize, count: usize, stride: usize) -> Result<()> {
        self.slice(
            at,
            count
                .checked_mul(stride)
                .ok_or_else(|| anyhow::anyhow!("table overflow"))?,
        )?;
        Ok(())
    }
    pub fn chunk(self, at: usize) -> Result<Chunk<'a>> {
        let kind = self.u16(at)?;
        let header = usize::from(self.u16(at + 2)?);
        let size = self.u32(at + 4)? as usize;
        ensure!(
            header >= 8 && size >= header,
            "invalid chunk header at 0x{at:x}"
        );
        Ok(Chunk {
            kind,
            header,
            bytes: Bytes(self.slice(at, size)?),
        })
    }
}

pub struct Chunk<'a> {
    pub kind: u16,
    pub header: usize,
    pub bytes: Bytes<'a>,
}

impl Chunk<'_> {
    pub fn children(&self) -> Result<Vec<Chunk<'_>>> {
        let mut children = vec![];
        let mut at = self.header;
        while at < self.bytes.0.len() {
            let child = self.bytes.chunk(at)?;
            at += child.bytes.0.len();
            children.push(child);
        }
        Ok(children)
    }
}

pub fn string_pool(chunk: &Chunk<'_>) -> Result<Vec<String>> {
    ensure!(chunk.kind == 1 && chunk.header >= 28, "invalid string pool");
    let b = chunk.bytes;
    let count = b.u32(8)? as usize;
    let flags = b.u32(16)?;
    let start = b.u32(20)? as usize;
    b.table(chunk.header, count, 4)?;
    ensure!(
        start >= chunk.header + count * 4 && start <= b.0.len(),
        "invalid string data start"
    );
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let mut at = start
            .checked_add(b.u32(chunk.header + i * 4)? as usize)
            .ok_or_else(|| anyhow::anyhow!("string offset overflow"))?;
        if flags & 0x100 != 0 {
            let units = length8(b, &mut at)?;
            let len = length8(b, &mut at)?;
            let s = std::str::from_utf8(b.slice(at, len)?)?;
            ensure!(
                b.u8(at + len)? == 0 && s.encode_utf16().count() == units,
                "invalid UTF-8 string pool length"
            );
            result.push(s.to_owned());
        } else {
            let first = b.u16(at)?;
            at += 2;
            let len = if first & 0x8000 != 0 {
                let lo = b.u16(at)?;
                at += 2;
                (usize::from(first & 0x7fff) << 16) | usize::from(lo)
            } else {
                usize::from(first)
            };
            b.table(at, len, 2)?;
            let units = (0..len)
                .map(|n| b.u16(at + n * 2))
                .collect::<Result<Vec<_>>>()?;
            ensure!(b.u16(at + len * 2)? == 0, "unterminated UTF-16 string pool");
            result.push(String::from_utf16(&units)?);
        }
    }
    Ok(result)
}

fn length8(b: Bytes<'_>, at: &mut usize) -> Result<usize> {
    let first = b.u8(*at)?;
    *at += 1;
    if first & 0x80 == 0 {
        return Ok(usize::from(first));
    }
    let next = b.u8(*at)?;
    *at += 1;
    Ok((usize::from(first & 0x7f) << 8) | usize::from(next))
}

pub fn string_at(strings: &[String], idx: u32) -> Result<&str> {
    strings
        .get(idx as usize)
        .map(String::as_str)
        .ok_or_else(|| anyhow::anyhow!("invalid string index {idx}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_reads() {
        assert!(Bytes(&[0; 8]).slice(usize::MAX, 2).is_err());
        assert!(Bytes(&[0x80; 8]).uleb(&mut 0).is_err());
        assert_eq!(Bytes(&[0x7f]).sleb(&mut 0).unwrap(), -1);
        assert_eq!(Bytes(&[0x80, 0x7f]).sleb(&mut 0).unwrap(), -128);
    }
}
