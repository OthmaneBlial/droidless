use crate::{
    heap::{Data, Word, bits64, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

impl Runtime {
    pub(crate) fn string_format_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class == "Landroid/webkit/MimeTypeMap;"
            && method.signature() == "getFileExtensionFromUrl(Ljava/lang/String;)Ljava/lang/String;"
        {
            let url = self
                .heap
                .text(*args.first().context("MimeTypeMap URL missing")?)?;
            let path = url.split(['?', '#']).next().unwrap_or("");
            let name = path.rsplit('/').next().unwrap_or("");
            let extension = name
                .rfind('.')
                .filter(|index| *index + 1 < name.len())
                .map_or("", |index| &name[index + 1..]);
            return Ok(Some(vec![self.heap.string(extension.to_owned())?]));
        }
        if method.class == "Ljava/net/URLEncoder;"
            && method.signature()
                == "encode(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"
        {
            let value = self
                .heap
                .text(*args.first().context("URLEncoder input missing")?)?;
            let charset = self
                .heap
                .text(*args.get(1).context("URLEncoder charset missing")?)?;
            ensure!(
                matches!(
                    charset.to_ascii_lowercase().replace('_', "-").as_str(),
                    "utf-8" | "utf8"
                ),
                fault(
                    "Ljava/io/UnsupportedEncodingException;",
                    format!("unsupported charset: {charset}"),
                )
            );
            let mut encoded = String::with_capacity(value.len());
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            for byte in value.bytes() {
                if byte.is_ascii_alphanumeric() || b"-_. *".contains(&byte) {
                    encoded.push(if byte == b' ' { '+' } else { byte as char });
                } else {
                    encoded.push('%');
                    encoded.push(HEX[usize::from(byte >> 4)] as char);
                    encoded.push(HEX[usize::from(byte & 15)] as char);
                }
            }
            ensure!(encoded.len() <= 1_048_576, "encoded string exceeds 1 MiB");
            return Ok(Some(vec![self.heap.string(encoded)?]));
        }
        if method.class != "Ljava/lang/String;"
            || method.signature()
                != "format(Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/String;"
        {
            return Ok(None);
        }
        let format = self
            .heap
            .text(*args.first().context("String.format format missing")?)?
            .to_owned();
        let array = self
            .heap
            .get(*args.get(1).context("String.format arguments missing")?)?;
        let Data::Array { values, .. } = &array.data else {
            return Err(fault(
                "Ljava/lang/IllegalArgumentException;",
                "String.format arguments must be an object array",
            ));
        };
        let values = values
            .iter()
            .map(|value| value.first().copied().unwrap_or(Word::ZERO))
            .collect::<Vec<_>>();
        let roots = self.native_roots.len();
        self.native_roots.extend(values.iter().copied());
        let result = self.format_java(&format, &values);
        self.native_roots.truncate(roots);
        Ok(Some(vec![self.heap.string(result?)?]))
    }

    fn format_java(&mut self, format: &str, args: &[Word]) -> Result<String> {
        let chars = format.chars().collect::<Vec<_>>();
        let mut output = String::new();
        let mut cursor = 0;
        let mut next_argument = 0;
        let mut last_argument = None;
        while cursor < chars.len() {
            if chars[cursor] != '%' {
                output.push(chars[cursor]);
                cursor += 1;
                continue;
            }
            cursor += 1;
            if cursor == chars.len() {
                return Err(format_error("trailing '%' in format string"));
            }
            if chars[cursor] == '%' {
                output.push('%');
                cursor += 1;
                continue;
            }
            if chars[cursor] == 'n' {
                output.push('\n');
                cursor += 1;
                continue;
            }
            let start = cursor;
            let digits = read_digits(&chars, &mut cursor);
            let explicit = if cursor < chars.len() && chars[cursor] == '$' {
                cursor += 1;
                Some(
                    digits
                        .context("missing format argument index")?
                        .saturating_sub(1),
                )
            } else {
                cursor = start;
                None
            };
            let mut flags = String::new();
            while cursor < chars.len() && "-+ 0,(#<".contains(chars[cursor]) {
                flags.push(chars[cursor]);
                cursor += 1;
            }
            let width = read_digits(&chars, &mut cursor);
            let precision = if cursor < chars.len() && chars[cursor] == '.' {
                cursor += 1;
                Some(read_digits(&chars, &mut cursor).context("missing format precision")?)
            } else {
                None
            };
            let conversion = *chars.get(cursor).context("missing format conversion")?;
            cursor += 1;
            let argument = if let Some(index) = explicit {
                index
            } else if flags.contains('<') {
                last_argument.context("relative format argument has no previous value")?
            } else {
                let index = next_argument;
                next_argument += 1;
                index
            };
            last_argument = Some(argument);
            let value = *args
                .get(argument)
                .ok_or_else(|| format_error(format!("missing format argument {}", argument + 1)))?;
            let formatted = self.format_conversion(value, conversion, &flags, precision)?;
            output.push_str(&pad(
                formatted,
                width,
                flags.contains('-'),
                flags.contains('0'),
            ));
            ensure_format_size(output.len())?;
        }
        Ok(output)
    }

    fn format_conversion(
        &mut self,
        value: Word,
        conversion: char,
        flags: &str,
        precision: Option<usize>,
    ) -> Result<String> {
        let upper = conversion.is_ascii_uppercase();
        let conversion = conversion.to_ascii_lowercase();
        let mut result = match conversion {
            's' => {
                let text = self.format_string_value(value)?;
                precision.map_or(text.clone(), |count| text.chars().take(count).collect())
            }
            'b' => {
                let truth = if value == Word::ZERO {
                    false
                } else {
                    let object = self.heap.get(value)?;
                    if object.class == "Ljava/lang/Boolean;" {
                        object
                            .fields
                            .get("value")
                            .and_then(|values| values.first())
                            .copied()
                            .context("uninitialized Boolean")?
                            .int()?
                            != 0
                    } else {
                        true
                    }
                };
                truth.to_string()
            }
            'd' | 'x' | 'o' => {
                let integer = self.format_integer(value)?;
                format_integer(integer, conversion, flags, precision)
            }
            'f' | 'e' => {
                let number = self.format_float(value)?;
                let precision = precision.unwrap_or(6);
                if precision > 1000 {
                    return Err(format_error("format precision exceeds 1000"));
                }
                if conversion == 'f' {
                    format!("{number:.precision$}")
                } else {
                    format!("{number:.precision$e}")
                }
            }
            'c' => {
                let codepoint = u32::try_from(self.format_integer(value)?)
                    .map_err(|_| format_error("invalid character argument"))?;
                char::from_u32(codepoint)
                    .context("invalid character argument")?
                    .to_string()
            }
            _ => {
                return Err(format_error(format!(
                    "unsupported format conversion '{conversion}'"
                )));
            }
        };
        if upper {
            result = result.to_uppercase();
        }
        Ok(result)
    }

    fn format_string_value(&mut self, value: Word) -> Result<String> {
        if value == Word::ZERO {
            return Ok("null".into());
        }
        let object = self.heap.get(value)?;
        if let Data::String(text) = &object.data {
            return Ok(text.clone());
        }
        if object.class == "Ljava/lang/Boolean;" {
            let value = object
                .fields
                .get("value")
                .and_then(|values| values.first())
                .copied()
                .context("uninitialized Boolean")?
                .int()?;
            return Ok((value != 0).to_string());
        }
        if matches!(
            object.class.as_str(),
            "Ljava/lang/Byte;"
                | "Ljava/lang/Character;"
                | "Ljava/lang/Short;"
                | "Ljava/lang/Integer;"
        ) {
            return Ok(self.format_integer(value)?.to_string());
        }
        if object.class == "Ljava/lang/Long;" {
            return Ok(self.format_integer(value)?.to_string());
        }
        if object.class == "Ljava/lang/Float;" || object.class == "Ljava/lang/Double;" {
            return Ok(self.format_float(value)?.to_string());
        }
        let text = self.invoke(
            Method {
                class: "Ljava/lang/Object;".into(),
                name: "toString".into(),
                parameters: vec![],
                returns: "Ljava/lang/String;".into(),
            },
            vec![value],
            true,
        )?;
        Ok(self
            .heap
            .text(*text.first().context("toString returned no value")?)?
            .to_owned())
    }

    fn format_integer(&self, value: Word) -> Result<i64> {
        let object = self.heap.get(value)?;
        if object.class == "Ljava/lang/Long;" {
            return Ok(bits64(object.fields.get("value").context("uninitialized Long")?)? as i64);
        }
        if matches!(
            object.class.as_str(),
            "Ljava/lang/Byte;"
                | "Ljava/lang/Character;"
                | "Ljava/lang/Short;"
                | "Ljava/lang/Integer;"
                | "Ljava/lang/Boolean;"
        ) {
            return Ok(i64::from(
                object
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .copied()
                    .context("uninitialized integer wrapper")?
                    .int()?,
            ));
        }
        Err(format_error(format!(
            "integer format requires an integral value, got {}",
            object.class
        )))
    }

    fn format_float(&self, value: Word) -> Result<f64> {
        let object = self.heap.get(value)?;
        match object.class.as_str() {
            "Ljava/lang/Double;" => Ok(f64::from_bits(bits64(
                object.fields.get("value").context("uninitialized Double")?,
            )?)),
            "Ljava/lang/Float;" => {
                let bits = object
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .copied()
                    .context("uninitialized Float")?
                    .int()? as u32;
                Ok(f64::from(f32::from_bits(bits)))
            }
            _ => Err(format_error(format!(
                "floating format requires a floating value, got {}",
                object.class
            ))),
        }
    }
}

fn read_digits(chars: &[char], cursor: &mut usize) -> Option<usize> {
    let start = *cursor;
    let mut value = 0usize;
    while let Some(char) = chars.get(*cursor).filter(|char| char.is_ascii_digit()) {
        value = value
            .saturating_mul(10)
            .saturating_add(char.to_digit(10)? as usize);
        *cursor += 1;
    }
    (*cursor > start).then_some(value)
}

fn format_integer(value: i64, conversion: char, flags: &str, precision: Option<usize>) -> String {
    let negative = value < 0 && conversion == 'd';
    let magnitude = value.unsigned_abs();
    let mut digits = match conversion {
        'x' => format!("{magnitude:x}"),
        'o' => format!("{magnitude:o}"),
        _ => magnitude.to_string(),
    };
    if let Some(minimum) = precision {
        digits = format!("{digits:0>minimum$}");
    }
    if flags.contains(',') && conversion == 'd' {
        let mut grouped = String::new();
        for (index, digit) in digits.chars().rev().enumerate() {
            if index != 0 && index % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(digit);
        }
        digits = grouped.chars().rev().collect();
    }
    let sign = if negative {
        "-"
    } else if flags.contains('+') {
        "+"
    } else if flags.contains(' ') {
        " "
    } else {
        ""
    };
    if negative && flags.contains('(') {
        format!("({digits})")
    } else {
        format!("{sign}{digits}")
    }
}

fn pad(value: String, width: Option<usize>, left: bool, zero: bool) -> String {
    let Some(width) = width else {
        return value;
    };
    let length = value.chars().count();
    if width <= length {
        return value;
    }
    let pad_char = if zero && !left { '0' } else { ' ' };
    let padding = pad_char.to_string().repeat(width - length);
    if left {
        format!("{value}{padding}")
    } else if pad_char == '0' && value.starts_with(['-', '+']) {
        format!("{}{}{}", &value[..1], padding, &value[1..])
    } else {
        format!("{padding}{value}")
    }
}

fn ensure_format_size(size: usize) -> Result<()> {
    if size > 1_048_576 {
        Err(format_error("formatted string exceeds 1 MiB"))
    } else {
        Ok(())
    }
}

fn format_error(message: impl Into<String>) -> anyhow::Error {
    fault("Ljava/lang/IllegalArgumentException;", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use droidless_formats::apk::Apk;
    use std::collections::BTreeMap;

    #[test]
    fn string_format_handles_positions_padding_percent_and_guest_values() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let format = vm.heap.string("%1$s %2$04d %%".into()).unwrap();
        let label = vm.heap.string("row".into()).unwrap();
        let number = vm.heap.instance("Ljava/lang/Integer;").unwrap();
        vm.heap
            .get_mut(number)
            .unwrap()
            .fields
            .insert("value".into(), vec![Word::from(12)]);
        let arguments = vm
            .heap
            .alloc(crate::heap::Object {
                class: "[Ljava/lang/Object;".into(),
                fields: BTreeMap::new(),
                data: Data::Array {
                    element: "Ljava/lang/Object;".into(),
                    values: vec![vec![label], vec![number]],
                },
                view: None,
            })
            .unwrap();
        let result = vm
            .invoke(
                Method {
                    class: "Ljava/lang/String;".into(),
                    name: "format".into(),
                    parameters: vec!["Ljava/lang/String;".into(), "[Ljava/lang/Object;".into()],
                    returns: "Ljava/lang/String;".into(),
                },
                vec![format, arguments],
                false,
            )
            .unwrap()[0];
        assert_eq!(vm.heap.text(result).unwrap(), "row 0012 %");
    }

    #[test]
    fn url_encoder_uses_utf8_form_encoding() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let value = vm.heap.string("notes café/backup 2.nbu".into()).unwrap();
        let charset = vm.heap.string("UTF-8".into()).unwrap();
        let result = vm
            .invoke(
                Method {
                    class: "Ljava/net/URLEncoder;".into(),
                    name: "encode".into(),
                    parameters: vec!["Ljava/lang/String;".into(), "Ljava/lang/String;".into()],
                    returns: "Ljava/lang/String;".into(),
                },
                vec![value, charset],
                false,
            )
            .unwrap()[0];
        assert_eq!(
            vm.heap.text(result).unwrap(),
            "notes+caf%C3%A9%2Fbackup+2.nbu"
        );
    }
}
