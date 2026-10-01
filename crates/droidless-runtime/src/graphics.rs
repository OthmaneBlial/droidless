use anyhow::{Result, ensure};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub mime: &'static str,
}

pub fn inspect(bytes: &[u8]) -> Result<Option<ImageInfo>> {
    let info = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        ensure!(
            bytes.len() >= 24 && &bytes[12..16] == b"IHDR",
            "invalid PNG header"
        );
        Some(ImageInfo {
            width: u32::from_be_bytes(bytes[16..20].try_into()?),
            height: u32::from_be_bytes(bytes[20..24].try_into()?),
            mime: "image/png",
        })
    } else if bytes.starts_with(b"\xff\xd8") {
        jpeg_info(bytes)?
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        webp_info(bytes)?
    } else {
        None
    };
    if let Some(info) = info {
        ensure!(
            info.width > 0
                && info.height > 0
                && info.width <= 32_768
                && info.height <= 32_768
                && u64::from(info.width) * u64::from(info.height) <= 32_000_000,
            "image dimensions exceed DROIDLESS limits"
        );
    }
    Ok(info)
}

fn jpeg_info(bytes: &[u8]) -> Result<Option<ImageInfo>> {
    let mut at = 2;
    while at < bytes.len() {
        ensure!(bytes[at] == 0xff, "invalid JPEG marker");
        while bytes.get(at) == Some(&0xff) {
            at += 1;
        }
        let marker = *bytes
            .get(at)
            .ok_or_else(|| anyhow::anyhow!("truncated JPEG marker"))?;
        at += 1;
        if marker == 0xd9 || marker == 0xda {
            break;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let length = usize::from(u16::from_be_bytes(
            bytes
                .get(at..at + 2)
                .ok_or_else(|| anyhow::anyhow!("truncated JPEG segment"))?
                .try_into()?,
        ));
        ensure!(
            length >= 2 && at + length <= bytes.len(),
            "invalid JPEG segment length"
        );
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            ensure!(length >= 8, "invalid JPEG frame header");
            let components = usize::from(bytes[at + 7]);
            ensure!(length >= 8 + 3 * components, "truncated JPEG frame header");
            return Ok(Some(ImageInfo {
                height: u32::from(u16::from_be_bytes([bytes[at + 3], bytes[at + 4]])),
                width: u32::from(u16::from_be_bytes([bytes[at + 5], bytes[at + 6]])),
                mime: "image/jpeg",
            }));
        }
        at += length;
    }
    Ok(None)
}

fn webp_info(bytes: &[u8]) -> Result<Option<ImageInfo>> {
    let riff_length = u32::from_le_bytes(bytes[4..8].try_into()?) as usize;
    ensure!(
        riff_length >= 12 && riff_length <= bytes.len().saturating_sub(8),
        "invalid WebP length"
    );
    ensure!(bytes.len() >= 25, "truncated WebP header");
    let chunk_length = u32::from_le_bytes(bytes[16..20].try_into()?) as usize;
    ensure!(
        chunk_length <= riff_length - 12,
        "invalid WebP chunk length"
    );
    let (width, height) = match &bytes[12..16] {
        b"VP8X" => {
            ensure!(
                chunk_length >= 10 && bytes.len() >= 30,
                "invalid WebP VP8X header"
            );
            (
                (u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]) & 0x00ff_ffff) + 1,
                (u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]) & 0x00ff_ffff) + 1,
            )
        }
        b"VP8 " => {
            ensure!(
                chunk_length >= 10 && bytes.len() >= 30,
                "invalid WebP VP8 header"
            );
            ensure!(
                bytes[23..26] == [0x9d, 0x01, 0x2a],
                "invalid WebP VP8 frame"
            );
            (
                u32::from(u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3fff),
                u32::from(u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3fff),
            )
        }
        b"VP8L" => {
            ensure!(
                chunk_length >= 5 && bytes[20] == 0x2f,
                "invalid WebP VP8L header"
            );
            let (b0, b1, b2, b3) = (bytes[21], bytes[22], bytes[23], bytes[24]);
            (
                1 + u32::from(b0) + (u32::from(b1 & 0x3f) << 8),
                1 + u32::from((b1 >> 6) & 0x03)
                    + (u32::from(b2) << 2)
                    + (u32::from(b3 & 0x0f) << 10),
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(ImageInfo {
        width,
        height,
        mime: "image/webp",
    }))
}

#[cfg(test)]
mod tests {
    use super::{ImageInfo, inspect};

    #[test]
    fn reads_png_jpeg_and_webp_dimensions() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&2u32.to_be_bytes());
        png.extend_from_slice(&3u32.to_be_bytes());
        assert_eq!(
            inspect(&png).unwrap(),
            Some(ImageInfo {
                width: 2,
                height: 3,
                mime: "image/png"
            })
        );

        let jpeg = [0xff, 0xd8, 0xff, 0xc0, 0, 11, 8, 0, 3, 0, 2, 1, 1, 0x11, 0];
        assert_eq!(
            inspect(&jpeg).unwrap(),
            Some(ImageInfo {
                width: 2,
                height: 3,
                mime: "image/jpeg"
            })
        );

        let mut webp = [0; 30];
        webp[..4].copy_from_slice(b"RIFF");
        webp[4..8].copy_from_slice(&22u32.to_le_bytes());
        webp[8..12].copy_from_slice(b"WEBP");
        webp[12..16].copy_from_slice(b"VP8X");
        webp[16..20].copy_from_slice(&10u32.to_le_bytes());
        webp[24] = 1;
        webp[27] = 2;
        assert_eq!(
            inspect(&webp).unwrap(),
            Some(ImageInfo {
                width: 2,
                height: 3,
                mime: "image/webp"
            })
        );
        let mut lossless = [0; 25];
        lossless[..4].copy_from_slice(b"RIFF");
        lossless[4..8].copy_from_slice(&17u32.to_le_bytes());
        lossless[8..12].copy_from_slice(b"WEBP");
        lossless[12..16].copy_from_slice(b"VP8L");
        lossless[16..20].copy_from_slice(&5u32.to_le_bytes());
        lossless[20] = 0x2f;
        lossless[21] = 1;
        lossless[22] = 0x80;
        assert_eq!(
            inspect(&lossless).unwrap(),
            Some(ImageInfo {
                width: 2,
                height: 3,
                mime: "image/webp"
            })
        );
        let mut invalid = webp;
        invalid[16..20].copy_from_slice(&11u32.to_le_bytes());
        assert!(inspect(&invalid).is_err());
    }

    #[test]
    fn rejects_oversized_or_truncated_images() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&8192u32.to_be_bytes());
        png.extend_from_slice(&8192u32.to_be_bytes());
        assert!(inspect(&png).is_err());
        assert!(inspect(b"\x89PNG\r\n\x1a\n").is_err());
        assert_eq!(inspect(b"not an image").unwrap(), None);
    }
}
