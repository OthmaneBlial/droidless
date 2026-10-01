use crate::{
    heap::{Data, PathCommand, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const IDENTITY: [f32; 9] = [1., 0., 0., 0., 1., 0., 0., 0., 1.];

fn map_point(m: [f32; 9], x: f32, y: f32) -> [f32; 2] {
    let denominator = m[6] * x + m[7] * y + m[8];
    [
        (m[0] * x + m[1] * y + m[2]) / denominator,
        (m[3] * x + m[4] * y + m[5]) / denominator,
    ]
}

fn multiply(left: [f32; 9], right: [f32; 9]) -> [f32; 9] {
    std::array::from_fn(|i| {
        (0..3)
            .map(|k| left[i / 3 * 3 + k] * right[k * 3 + i % 3])
            .sum()
    })
}

impl Runtime {
    pub(crate) fn path_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != "Landroid/graphics/Path;" {
            return Ok(None);
        }
        let arg = |index| args.get(index).copied().context("Path argument missing");
        let receiver = arg(0)?;
        match method.signature().as_str() {
            "<init>()V" | "reset()V" | "rewind()V" => {
                self.heap.get_mut(receiver)?.data = Data::Path(vec![])
            }
            "<init>(Landroid/graphics/Path;)V" | "set(Landroid/graphics/Path;)V" => {
                let Data::Path(commands) = &self.heap.get(arg(1)?)?.data else {
                    bail!("uninitialized Path");
                };
                self.heap.get_mut(receiver)?.data = Data::Path(commands.clone());
            }
            "moveTo(FF)V" | "lineTo(FF)V" | "quadTo(FFFF)V" | "cubicTo(FFFFFF)V" | "close()V" => {
                let float =
                    |index| -> Result<f32> { Ok(f32::from_bits(arg(index)?.int()? as u32)) };
                let command = match method.name.as_str() {
                    "moveTo" => PathCommand::Move([float(1)?, float(2)?]),
                    "lineTo" => PathCommand::Line([float(1)?, float(2)?]),
                    "quadTo" => PathCommand::Quad([float(1)?, float(2)?, float(3)?, float(4)?]),
                    "cubicTo" => PathCommand::Cubic([
                        float(1)?,
                        float(2)?,
                        float(3)?,
                        float(4)?,
                        float(5)?,
                        float(6)?,
                    ]),
                    _ => PathCommand::Close,
                };
                let Data::Path(commands) = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("uninitialized Path");
                };
                ensure!(
                    commands.len() < 100_000,
                    "Path command limit reached (100000)"
                );
                if commands.is_empty()
                    && !matches!(command, PathCommand::Move(_) | PathCommand::Close)
                {
                    commands.push(PathCommand::Move([0., 0.]));
                }
                if !commands.is_empty() || !matches!(command, PathCommand::Close) {
                    commands.push(command);
                }
            }
            _ => bail!("unsupported Path method {}", method.key()),
        }
        Ok(Some(vec![]))
    }

    fn matrix_values(&self, object: Word) -> Result<[f32; 9]> {
        let Data::Matrix(values) = self.heap.get(object)?.data else {
            bail!("uninitialized Matrix");
        };
        Ok(values)
    }

    pub(crate) fn matrix_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != "Landroid/graphics/Matrix;" {
            return Ok(None);
        }
        let arg = |index| args.get(index).copied().context("Matrix argument missing");
        let receiver = arg(0)?;
        let float = |index| -> Result<f32> { Ok(f32::from_bits(arg(index)?.int()? as u32)) };
        let signature = method.signature();
        let mut result = vec![];
        let matrix = match signature.as_str() {
            "<init>()V" | "reset()V" => IDENTITY,
            "<init>(Landroid/graphics/Matrix;)V" | "set(Landroid/graphics/Matrix;)V" => {
                if arg(1)? == Word::ZERO {
                    IDENTITY
                } else {
                    self.matrix_values(arg(1)?)?
                }
            }
            "isIdentity()Z" => {
                return Ok(Some(vec![Word::from(i32::from(
                    self.matrix_values(receiver)? == IDENTITY,
                ))]));
            }
            "equals(Ljava/lang/Object;)Z" => {
                let equal = matches!(self.heap.get(arg(1)?).map(|o| &o.data), Ok(Data::Matrix(other)) if *other == self.matrix_values(receiver)?);
                return Ok(Some(vec![Word::from(i32::from(equal))]));
            }
            "getValues([F)V" | "setValues([F)V" => {
                let matrix = self.matrix_values(receiver)?;
                let Data::Array { element, values } = &mut self.heap.get_mut(arg(1)?)?.data else {
                    bail!("Matrix values require float[]");
                };
                ensure!(element == "F", "Matrix values require float[]");
                ensure!(
                    values.len() >= 9,
                    fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "Matrix needs nine values"
                    )
                );
                if method.name == "getValues" {
                    for (slot, value) in values.iter_mut().zip(matrix) {
                        *slot = vec![Word::Bits(value.to_bits())];
                    }
                    return Ok(Some(vec![]));
                }
                let mut matrix = [0.; 9];
                for (slot, value) in matrix.iter_mut().zip(values) {
                    *slot =
                        f32::from_bits(value.first().copied().unwrap_or(Word::ZERO).int()? as u32);
                }
                matrix
            }
            "mapPoints([F)V" | "mapVectors([F)V" => {
                let m = self.matrix_values(receiver)?;
                let vectors = method.name == "mapVectors";
                let Data::Array { element, values } = &mut self.heap.get_mut(arg(1)?)?.data else {
                    bail!("Matrix coordinates require float[]");
                };
                ensure!(element == "F", "Matrix coordinates require float[]");
                for pair in values.chunks_exact_mut(2) {
                    let x = f32::from_bits(
                        pair[0].first().copied().unwrap_or(Word::ZERO).int()? as u32
                    );
                    let y = f32::from_bits(
                        pair[1].first().copied().unwrap_or(Word::ZERO).int()? as u32
                    );
                    let denominator = m[6] * x + m[7] * y + m[8];
                    if vectors && m[6] == 0. && m[7] == 0. {
                        pair[0] = vec![Word::Bits(((m[0] * x + m[1] * y) / denominator).to_bits())];
                        pair[1] = vec![Word::Bits(((m[3] * x + m[4] * y) / denominator).to_bits())];
                        continue;
                    }
                    let origin = if vectors {
                        [m[2] / m[8], m[5] / m[8]]
                    } else {
                        [0., 0.]
                    };
                    let point = map_point(m, x, y);
                    pair[0] = vec![Word::Bits((point[0] - origin[0]).to_bits())];
                    pair[1] = vec![Word::Bits((point[1] - origin[1]).to_bits())];
                }
                return Ok(Some(vec![]));
            }
            "mapRect(Landroid/graphics/RectF;)Z" => {
                let rect = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(rect)?.class, "Landroid/graphics/RectF;"),
                    "Matrix bounds require RectF"
                );
                let m = self.matrix_values(receiver)?;
                let fields = &self.heap.get(rect)?.fields;
                let edge = |name: &str| -> Result<f32> {
                    Ok(f32::from_bits(
                        fields
                            .get(&format!("Landroid/graphics/RectF;->{name}:F"))
                            .and_then(|values| values.first())
                            .copied()
                            .unwrap_or(Word::ZERO)
                            .int()? as u32,
                    ))
                };
                let (left, top, right, bottom) =
                    (edge("left")?, edge("top")?, edge("right")?, edge("bottom")?);
                let points = [(left, top), (right, top), (right, bottom), (left, bottom)]
                    .map(|(x, y)| map_point(m, x, y));
                ensure!(
                    points.iter().flatten().all(|v| v.is_finite()),
                    "non-finite mapped rectangle unsupported"
                );
                let values = [
                    points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min),
                    points.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min),
                    points
                        .iter()
                        .map(|p| p[0])
                        .fold(f32::NEG_INFINITY, f32::max),
                    points
                        .iter()
                        .map(|p| p[1])
                        .fold(f32::NEG_INFINITY, f32::max),
                ];
                for (edge, value) in ["left", "top", "right", "bottom"].into_iter().zip(values) {
                    self.heap.get_mut(rect)?.fields.insert(
                        format!("Landroid/graphics/RectF;->{edge}:F"),
                        vec![Word::Bits(value.to_bits())],
                    );
                }
                let stays_rect = m[6] == 0.
                    && m[7] == 0.
                    && m[8] == 1.
                    && ((m[1] == 0. && m[3] == 0.) || (m[0] == 0. && m[4] == 0.));
                return Ok(Some(vec![Word::from(i32::from(stays_rect))]));
            }
            "preConcat(Landroid/graphics/Matrix;)Z"
            | "postConcat(Landroid/graphics/Matrix;)Z"
            | "postTranslate(FF)Z"
            | "preTranslate(FF)Z"
            | "setTranslate(FF)V"
            | "postScale(FF)Z"
            | "preScale(FF)Z"
            | "setScale(FF)V"
            | "postRotate(F)Z"
            | "postRotate(FFF)Z" => {
                let transform = if method.name.ends_with("Concat") {
                    self.matrix_values(arg(1)?)?
                } else if method.name.ends_with("Translate") {
                    [1., 0., float(1)?, 0., 1., float(2)?, 0., 0., 1.]
                } else if method.name.ends_with("Scale") {
                    [float(1)?, 0., 0., 0., float(2)?, 0., 0., 0., 1.]
                } else {
                    let (sin, cos) = float(1)?.to_radians().sin_cos();
                    let (px, py) = if args.len() == 4 {
                        (float(2)?, float(3)?)
                    } else {
                        (0., 0.)
                    };
                    [
                        cos,
                        -sin,
                        px - cos * px + sin * py,
                        sin,
                        cos,
                        py - sin * px - cos * py,
                        0.,
                        0.,
                        1.,
                    ]
                };
                if method.name.starts_with("set") {
                    transform
                } else {
                    result.push(Word::from(1));
                    let current = self.matrix_values(receiver)?;
                    if method.name.starts_with("pre") {
                        multiply(current, transform)
                    } else {
                        multiply(transform, current)
                    }
                }
            }
            _ => bail!("unsupported Matrix method {}", method.key()),
        };
        self.heap.get_mut(receiver)?.data = Data::Matrix(matrix);
        Ok(Some(result))
    }
}

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
    fn path_commands_copy_without_aliasing_and_reject_missing_coordinates() {
        use super::*;
        use droidless_formats::apk::Apk;
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
        )
        .unwrap();
        let path = vm.heap.instance("Landroid/graphics/Path;").unwrap();
        let call = |vm: &mut Runtime, name: &str, parameters: &[&str], args: Vec<Word>| {
            vm.invoke(
                Method {
                    class: "Landroid/graphics/Path;".into(),
                    name: name.into(),
                    parameters: parameters.iter().map(|s| (*s).into()).collect(),
                    returns: "V".into(),
                },
                args,
                true,
            )
        };
        call(&mut vm, "<init>", &[], vec![path]).unwrap();
        call(
            &mut vm,
            "lineTo",
            &["F", "F"],
            vec![path, Word::Bits(2f32.to_bits()), Word::Bits(3f32.to_bits())],
        )
        .unwrap();
        call(&mut vm, "close", &[], vec![path]).unwrap();
        let copy = vm.heap.instance("Landroid/graphics/Path;").unwrap();
        call(
            &mut vm,
            "<init>",
            &["Landroid/graphics/Path;"],
            vec![copy, path],
        )
        .unwrap();
        call(&mut vm, "reset", &[], vec![path]).unwrap();
        let Data::Path(commands) = &vm.heap.get(copy).unwrap().data else {
            unreachable!()
        };
        assert_eq!(
            commands,
            &[
                PathCommand::Move([0., 0.]),
                PathCommand::Line([2., 3.]),
                PathCommand::Close
            ]
        );
        let error = call(
            &mut vm,
            "quadTo",
            &["F", "F", "F", "F"],
            vec![copy, Word::ZERO],
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("Path argument missing"));
    }

    #[test]
    fn matrix_composition_preserves_android_order_pivots_and_array_bounds() {
        use super::*;
        use droidless_formats::apk::Apk;
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
        )
        .unwrap();
        let matrix = vm.heap.instance("Landroid/graphics/Matrix;").unwrap();
        let call =
            |vm: &mut Runtime, name: &str, parameters: &[&str], returns: &str, args: Vec<Word>| {
                vm.invoke(
                    Method {
                        class: "Landroid/graphics/Matrix;".into(),
                        name: name.into(),
                        parameters: parameters.iter().map(|s| (*s).into()).collect(),
                        returns: returns.into(),
                    },
                    args,
                    true,
                )
            };
        let f = |value: f32| Word::Bits(value.to_bits());
        call(&mut vm, "<init>", &[], "V", vec![matrix]).unwrap();
        call(
            &mut vm,
            "postTranslate",
            &["F", "F"],
            "Z",
            vec![matrix, f(10.), f(20.)],
        )
        .unwrap();
        call(
            &mut vm,
            "postScale",
            &["F", "F"],
            "Z",
            vec![matrix, f(2.), f(3.)],
        )
        .unwrap();
        assert_eq!(
            vm.matrix_values(matrix).unwrap(),
            [2., 0., 20., 0., 3., 60., 0., 0., 1.]
        );
        call(
            &mut vm,
            "preTranslate",
            &["F", "F"],
            "Z",
            vec![matrix, f(1.), f(2.)],
        )
        .unwrap();
        assert_eq!(vm.matrix_values(matrix).unwrap()[2], 22.);
        assert_eq!(vm.matrix_values(matrix).unwrap()[5], 66.);
        let copy = vm.heap.instance("Landroid/graphics/Matrix;").unwrap();
        call(
            &mut vm,
            "<init>",
            &["Landroid/graphics/Matrix;"],
            "V",
            vec![copy, matrix],
        )
        .unwrap();
        call(&mut vm, "reset", &[], "V", vec![matrix]).unwrap();
        assert_eq!(vm.matrix_values(copy).unwrap()[2], 22.);
        call(
            &mut vm,
            "postRotate",
            &["F", "F", "F"],
            "Z",
            vec![matrix, f(90.), f(2.), f(3.)],
        )
        .unwrap();
        let points = vm.array("F".into(), 4).unwrap();
        if let Data::Array { values, .. } = &mut vm.heap.get_mut(points).unwrap().data {
            *values = [2., 3., 3., 3.].map(|n| vec![f(n)]).into();
        }
        call(&mut vm, "mapPoints", &["[F"], "V", vec![matrix, points]).unwrap();
        let Data::Array { values, .. } = &vm.heap.get(points).unwrap().data else {
            unreachable!()
        };
        for (actual, expected) in values.iter().zip([2., 3., 2., 4.]) {
            assert!((f32::from_bits(actual[0].int().unwrap() as u32) - expected).abs() < 0.00001);
        }
        let short = vm.array("F".into(), 8).unwrap();
        let error = call(&mut vm, "getValues", &["[F"], "V", vec![matrix, short]).unwrap_err();
        assert!(format!("{error:#}").contains("ArrayIndexOutOfBoundsException"));
        call(
            &mut vm,
            "set",
            &["Landroid/graphics/Matrix;"],
            "V",
            vec![matrix, Word::ZERO],
        )
        .unwrap();
        assert_eq!(vm.matrix_values(matrix).unwrap(), IDENTITY);
        call(
            &mut vm,
            "setTranslate",
            &["F", "F"],
            "V",
            vec![matrix, f(1e30), f(1e30)],
        )
        .unwrap();
        if let Data::Array { values, .. } = &mut vm.heap.get_mut(points).unwrap().data {
            *values = [1., 2., 3., 4.].map(|n| vec![f(n)]).into();
        }
        call(&mut vm, "mapVectors", &["[F"], "V", vec![matrix, points]).unwrap();
        let Data::Array { values, .. } = &vm.heap.get(points).unwrap().data else {
            unreachable!()
        };
        assert_eq!(*values, [1., 2., 3., 4.].map(|n| vec![f(n)]));
        let perspective = vm.array("F".into(), 9).unwrap();
        if let Data::Array { values, .. } = &mut vm.heap.get_mut(perspective).unwrap().data {
            *values = [1., 0., 10., 0., 1., 20., 0.5, 0., 1.]
                .map(|n| vec![f(n)])
                .into();
        }
        call(
            &mut vm,
            "setValues",
            &["[F"],
            "V",
            vec![matrix, perspective],
        )
        .unwrap();
        if let Data::Array { values, .. } = &mut vm.heap.get_mut(points).unwrap().data {
            *values = [2., 4.].map(|n| vec![f(n)]).into();
        }
        call(&mut vm, "mapVectors", &["[F"], "V", vec![matrix, points]).unwrap();
        let Data::Array { values, .. } = &vm.heap.get(points).unwrap().data else {
            unreachable!()
        };
        assert_eq!(*values, [-4., -8.].map(|n| vec![f(n)]));
    }

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
