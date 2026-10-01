use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;
use regex::RegexBuilder;
use std::sync::Arc;

const MAX_PATTERN_BYTES: usize = 4096;
const MAX_INPUT_BYTES: usize = 1_048_576;
const MAX_REGEX_SIZE: usize = 1_048_576;

impl Runtime {
    pub(crate) fn regex_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let argument = |index: usize| {
            args.get(index)
                .copied()
                .with_context(|| format!("{} argument missing", method.key()))
        };
        let signature = method.signature();
        match (method.class.as_str(), signature.as_str()) {
            (
                "Ljava/lang/String;",
                "replaceAll(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            ) => {
                let source = self.heap.text(argument(0)?)?.to_owned();
                let pattern = self.heap.text(argument(1)?)?;
                let replacement = self.heap.text(argument(2)?)?;
                ensure!(
                    source.len() <= MAX_INPUT_BYTES && pattern.len() <= MAX_PATTERN_BYTES,
                    "String.replaceAll input exceeds runtime limit"
                );
                let replaced = replace_all(pattern, &source, replacement)?;
                Ok(Some(vec![self.heap.string(replaced)?]))
            }
            (
                "Ljava/util/regex/Pattern;",
                "compile(Ljava/lang/String;)Ljava/util/regex/Pattern;",
            ) => {
                let source = self.heap.text(argument(0)?)?.to_owned();
                ensure!(
                    source.len() <= MAX_PATTERN_BYTES,
                    fault(
                        "Ljava/util/regex/PatternSyntaxException;",
                        "pattern exceeds 4096 UTF-8 bytes",
                    )
                );
                let regex = RegexBuilder::new(&source)
                    .size_limit(MAX_REGEX_SIZE)
                    .dfa_size_limit(MAX_REGEX_SIZE)
                    .build()
                    .map_err(|error| {
                        fault(
                            "Ljava/util/regex/PatternSyntaxException;",
                            error.to_string(),
                        )
                    })?;
                let pattern = self.heap.instance("Ljava/util/regex/Pattern;")?;
                self.heap.get_mut(pattern)?.data = Data::Pattern(Arc::new(regex));
                Ok(Some(vec![pattern]))
            }
            (
                "Ljava/util/regex/Pattern;",
                "matcher(Ljava/lang/CharSequence;)Ljava/util/regex/Matcher;",
            ) => {
                let pattern = argument(0)?;
                ensure!(
                    matches!(self.heap.get(pattern)?.data, Data::Pattern(_)),
                    "uninitialized Pattern"
                );
                let input = argument(1)?;
                ensure!(
                    self.heap.text(input)?.len() <= MAX_INPUT_BYTES,
                    "regex input exceeds 1 MiB"
                );
                let matcher = self.heap.instance("Ljava/util/regex/Matcher;")?;
                let object = self.heap.get_mut(matcher)?;
                object.fields.insert("pattern".into(), vec![pattern]);
                object.fields.insert("input".into(), vec![input]);
                object.data = Data::Matcher {
                    groups: None,
                    next_search: 0,
                };
                Ok(Some(vec![matcher]))
            }
            (
                "Ljava/util/regex/Matcher;",
                "find()Z"
                | "matches()Z"
                | "group()Ljava/lang/String;"
                | "group(I)Ljava/lang/String;",
            ) => {
                let matcher = argument(0)?;
                let object = self.heap.get(matcher)?;
                let pattern = object
                    .fields
                    .get("pattern")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized Matcher pattern")?;
                let input = object
                    .fields
                    .get("input")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized Matcher input")?;
                let regex = match &self.heap.get(pattern)?.data {
                    Data::Pattern(regex) => regex.clone(),
                    _ => anyhow::bail!("uninitialized Pattern"),
                };
                let source = self.heap.text(input)?.to_owned();

                match signature.as_str() {
                    "find()Z" | "matches()Z" => {
                        let old_offset = match &self.heap.get(matcher)?.data {
                            Data::Matcher { next_search, .. } => *next_search,
                            _ => anyhow::bail!("uninitialized Matcher"),
                        };
                        let captures = if signature == "find()Z" {
                            (old_offset <= source.len())
                                .then(|| regex.captures_at(&source, old_offset))
                                .flatten()
                        } else {
                            regex.captures(&source).filter(|captures| {
                                captures.get(0).is_some_and(|whole| {
                                    whole.start() == 0 && whole.end() == source.len()
                                })
                            })
                        };
                        let groups = captures.map(|captures| {
                            (0..captures.len())
                                .map(|index| captures.get(index).map(|m| (m.start(), m.end())))
                                .collect::<Vec<_>>()
                        });
                        let next_search = if signature == "find()Z" {
                            groups
                                .as_ref()
                                .and_then(|groups| groups.first().copied().flatten())
                                .map_or(source.len().saturating_add(1), |(start, end)| {
                                    if start != end {
                                        end
                                    } else {
                                        end + source[end..].chars().next().map_or(1, char::len_utf8)
                                    }
                                })
                        } else {
                            old_offset
                        };
                        let found = groups.is_some();
                        let Data::Matcher {
                            groups: current,
                            next_search: next,
                        } = &mut self.heap.get_mut(matcher)?.data
                        else {
                            anyhow::bail!("uninitialized Matcher")
                        };
                        *current = groups;
                        *next = next_search;
                        Ok(Some(vec![Word::from(i32::from(found))]))
                    }
                    "group()Ljava/lang/String;" | "group(I)Ljava/lang/String;" => {
                        let index = if signature == "group()Ljava/lang/String;" {
                            0
                        } else {
                            usize::try_from(argument(1)?.int()?).map_err(|_| {
                                fault(
                                    "Ljava/lang/IndexOutOfBoundsException;",
                                    "negative group index",
                                )
                            })?
                        };
                        let groups = match &self.heap.get(matcher)?.data {
                            Data::Matcher {
                                groups: Some(groups),
                                ..
                            } => groups.clone(),
                            Data::Matcher { groups: None, .. } => {
                                return Err(fault(
                                    "Ljava/lang/IllegalStateException;",
                                    "no successful match",
                                ));
                            }
                            _ => anyhow::bail!("uninitialized Matcher"),
                        };
                        let Some(range) = groups.get(index) else {
                            return Err(fault(
                                "Ljava/lang/IndexOutOfBoundsException;",
                                format!("no capture group {index}"),
                            ));
                        };
                        let Some((start, end)) = range else {
                            return Ok(Some(vec![Word::ZERO]));
                        };
                        let text = self.heap.string(source[*start..*end].to_owned())?;
                        Ok(Some(vec![text]))
                    }
                    _ => unreachable!(),
                }
            }
            _ => Ok(None),
        }
    }
}

fn replace_all(pattern: &str, source: &str, replacement: &str) -> Result<String> {
    let Some((open, close)) = terminal_positive_lookahead(pattern)? else {
        let regex = RegexBuilder::new(pattern)
            .size_limit(MAX_REGEX_SIZE)
            .dfa_size_limit(MAX_REGEX_SIZE)
            .build()
            .map_err(|error| {
                fault(
                    "Ljava/util/regex/PatternSyntaxException;",
                    error.to_string(),
                )
            })?;
        return Ok(regex.replace_all(source, replacement).into_owned());
    };

    if let Some((group_open, group_close, branches)) = lookahead_alternation(pattern, open, close) {
        if replacement.is_empty() {
            return replace_all_empty_lookahead_alternation(
                pattern,
                source,
                open,
                close,
                group_open,
                group_close,
                &branches,
            );
        }
        return Err(fault(
            "Ljava/util/regex/PatternSyntaxException;",
            "String.replaceAll look-ahead alternations require an empty replacement",
        ));
    }

    let consumed = format!("{}{}", &pattern[..open], &pattern[close + 1..]);
    let flags_len = leading_flag_bytes(&consumed);
    let assertion = &pattern[open + 3..close];
    let actual_name = unique_group_name(pattern, "actual");
    let assertion_name = unique_group_name(pattern, "assertion");
    let combined = format!(
        "{}(?P<{actual_name}>(?:{}))(?P<{assertion_name}>(?:{assertion}))",
        &consumed[..flags_len],
        &consumed[flags_len..]
    );
    let regex = RegexBuilder::new(&combined)
        .size_limit(MAX_REGEX_SIZE)
        .dfa_size_limit(MAX_REGEX_SIZE)
        .build()
        .map_err(|error| {
            fault(
                "Ljava/util/regex/PatternSyntaxException;",
                error.to_string(),
            )
        })?;
    let original_groups = regex.captures_len().saturating_sub(3);
    let mut output = String::with_capacity(source.len());
    let (mut search, mut copied) = (0, 0);
    while search <= source.len() {
        let Some(captures) = regex.captures_at(source, search) else {
            break;
        };
        let actual = captures
            .name(&actual_name)
            .context("look-ahead match lost its consumed range")?;
        ensure!(actual.start() >= copied, "look-ahead regex moved backwards");
        append_limited(&mut output, &source[copied..actual.start()])?;
        expand_replacement(
            &mut output,
            replacement,
            source,
            &captures,
            &actual_name,
            &assertion_name,
            original_groups,
        )?;
        copied = actual.end();
        if actual.end() > actual.start() {
            search = actual.end();
        } else if let Some(character) = source[actual.end()..].chars().next() {
            search = actual.end() + character.len_utf8();
        } else {
            search = source.len() + 1;
        }
    }
    append_limited(&mut output, &source[copied..])?;
    Ok(output)
}

fn lookahead_alternation(
    pattern: &str,
    look_open: usize,
    look_close: usize,
) -> Option<(usize, usize, Vec<usize>)> {
    let bytes = pattern.as_bytes();
    let (mut index, mut escaped, mut in_class) = (0, false, false);
    let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut result = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if in_class {
            if byte == b']' {
                in_class = false;
            }
        } else if byte == b'[' {
            in_class = true;
        } else if bytes[index..].starts_with(b"(?=") {
            index = matching_paren(bytes, index).ok()?;
        } else if byte == b'(' {
            groups.push((index, Vec::new()));
        } else if byte == b'|' {
            groups.last_mut()?.1.push(index);
        } else if byte == b')' {
            let (open, pipes) = groups.pop()?;
            if result.is_none() && open < look_open && look_close < index && !pipes.is_empty() {
                result = Some((open, index, pipes));
            }
        }
        index += 1;
    }
    result
}

fn replace_all_empty_lookahead_alternation(
    pattern: &str,
    source: &str,
    look_open: usize,
    look_close: usize,
    group_open: usize,
    group_close: usize,
    pipes: &[usize],
) -> Result<String> {
    let starts = std::iter::once(group_open + 1)
        .chain(pipes.iter().map(|pipe| pipe + 1))
        .collect::<Vec<_>>();
    let ends = pipes
        .iter()
        .copied()
        .chain(std::iter::once(group_close))
        .collect::<Vec<_>>();
    ensure!(
        starts.len() == ends.len(),
        "invalid regular-expression alternation"
    );

    let assertion = &pattern[look_open + 3..look_close];
    ensure!(
        !has_capture_group(assertion) && !pattern.contains("\\1") && !pattern.contains("\\k<"),
        fault(
            "Ljava/util/regex/PatternSyntaxException;",
            "capturing look-ahead or backreferences are unsupported in alternations",
        )
    );
    let target = starts
        .iter()
        .zip(&ends)
        .position(|(start, end)| *start <= look_open && look_close < *end)
        .context("look-ahead is outside its alternation")?;
    ensure!(
        ends[target] == look_close + 1,
        fault(
            "Ljava/util/regex/PatternSyntaxException;",
            "String.replaceAll supports look-ahead at the end of an alternation branch only",
        )
    );

    let mut names = Vec::with_capacity(starts.len());
    let mut alternatives = String::new();
    for branch in 0..starts.len() {
        if branch != 0 {
            alternatives.push('|');
        }
        let name = unique_group_name(pattern, &format!("branch_{branch}"));
        let branch_end = if branch == target {
            look_open
        } else {
            ends[branch]
        };
        names.push(name.clone());
        alternatives.push_str(&format!(
            "(?P<{name}>(?:{}))",
            &pattern[starts[branch]..branch_end]
        ));
        if branch == target {
            alternatives.push_str("(?:");
            alternatives.push_str(assertion);
            alternatives.push(')');
        }
    }
    let combined = format!(
        "{}(?:{}){}",
        &pattern[..group_open],
        alternatives,
        &pattern[group_close + 1..]
    );
    let regex = RegexBuilder::new(&combined)
        .size_limit(MAX_REGEX_SIZE)
        .dfa_size_limit(MAX_REGEX_SIZE)
        .build()
        .map_err(|error| {
            fault(
                "Ljava/util/regex/PatternSyntaxException;",
                error.to_string(),
            )
        })?;
    let mut output = String::with_capacity(source.len());
    let (mut search, mut copied) = (0, 0);
    while search <= source.len() {
        let Some(captures) = regex.captures_at(source, search) else {
            break;
        };
        let actual = names
            .iter()
            .find_map(|name| captures.name(name))
            .context("look-ahead alternation lost its consumed range")?;
        append_limited(&mut output, &source[copied..actual.start()])?;
        copied = actual.end();
        if actual.end() > actual.start() {
            search = actual.end();
        } else if let Some(character) = source[actual.end()..].chars().next() {
            search = actual.end() + character.len_utf8();
        } else {
            search = source.len() + 1;
        }
    }
    append_limited(&mut output, &source[copied..])?;
    Ok(output)
}

fn terminal_positive_lookahead(pattern: &str) -> Result<Option<(usize, usize)>> {
    let bytes = pattern.as_bytes();
    let (mut index, mut depth, mut escaped, mut in_class) = (0, 0isize, false, false);
    let mut found = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if in_class {
            if byte == b']' {
                in_class = false;
            }
        } else if byte == b'[' {
            in_class = true;
        } else if bytes[index..].starts_with(b"(?=") {
            ensure!(
                found.is_none(),
                fault(
                    "Ljava/util/regex/PatternSyntaxException;",
                    "multiple positive look-aheads are unsupported by String.replaceAll"
                )
            );
            let close = matching_paren(bytes, index)?;
            ensure!(
                bytes[close + 1..].iter().all(|byte| *byte == b')'),
                fault(
                    "Ljava/util/regex/PatternSyntaxException;",
                    "String.replaceAll supports a terminal positive look-ahead only"
                )
            );
            ensure!(
                !has_capture_group(&pattern[index + 3..close]),
                fault(
                    "Ljava/util/regex/PatternSyntaxException;",
                    "capturing groups inside look-ahead are unsupported"
                )
            );
            found = Some((index, close));
            index = close;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth -= 1;
            ensure!(
                depth >= 0,
                fault(
                    "Ljava/util/regex/PatternSyntaxException;",
                    "unmatched closing parenthesis"
                )
            );
        }
        index += 1;
    }
    ensure!(
        depth == 0,
        fault(
            "Ljava/util/regex/PatternSyntaxException;",
            "unclosed regular-expression group"
        )
    );
    Ok(found)
}

fn matching_paren(bytes: &[u8], open: usize) -> Result<usize> {
    let (mut index, mut depth, mut escaped, mut in_class) = (open, 0isize, false, false);
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if in_class {
            if byte == b']' {
                in_class = false;
            }
        } else if byte == b'[' {
            in_class = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth -= 1;
            if depth == 0 {
                return Ok(index);
            }
        }
        index += 1;
    }
    Err(fault(
        "Ljava/util/regex/PatternSyntaxException;",
        "unclosed positive look-ahead",
    ))
}

fn has_capture_group(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    let (mut index, mut escaped, mut in_class) = (0, false, false);
    while index < bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if in_class {
            if byte == b']' {
                in_class = false;
            }
        } else if byte == b'[' {
            in_class = true;
        } else if byte == b'(' && !bytes[index..].starts_with(b"(?:") {
            return true;
        }
        index += 1;
    }
    false
}

fn leading_flag_bytes(pattern: &str) -> usize {
    let bytes = pattern.as_bytes();
    let mut end = 0;
    while bytes[end..].starts_with(b"(?") {
        let Some(close) = bytes[end + 2..].iter().position(|byte| *byte == b')') else {
            break;
        };
        let close = end + 2 + close;
        let flags = &bytes[end + 2..close];
        if flags.is_empty()
            || !flags
                .iter()
                .all(|byte| byte.is_ascii_alphabetic() || *byte == b'-')
        {
            break;
        }
        end = close + 1;
    }
    end
}

fn unique_group_name(pattern: &str, name: &str) -> String {
    let mut index = 0;
    loop {
        let candidate = format!("droidless_{name}_{index}");
        if !pattern.contains(&candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn append_limited(output: &mut String, text: &str) -> Result<()> {
    ensure!(
        output.len() + text.len() <= MAX_INPUT_BYTES,
        fault(
            "Ljava/lang/IllegalArgumentException;",
            "regex replacement exceeds 1 MiB"
        )
    );
    output.push_str(text);
    Ok(())
}

fn expand_replacement(
    output: &mut String,
    replacement: &str,
    source: &str,
    captures: &regex::Captures<'_>,
    actual_name: &str,
    assertion_name: &str,
    original_groups: usize,
) -> Result<()> {
    let mut index = 0;
    while index < replacement.len() {
        let character = replacement[index..].chars().next().unwrap();
        if character != '$' {
            append_limited(output, &character.to_string())?;
            index += character.len_utf8();
            continue;
        }
        let after_dollar = index + 1;
        if replacement[after_dollar..].starts_with('$') {
            append_limited(output, "$")?;
            index = after_dollar + 1;
            continue;
        }
        if replacement[after_dollar..].starts_with('{') {
            let name_start = after_dollar + 1;
            if let Some(relative_end) = replacement[name_start..].find('}') {
                let name = &replacement[name_start..name_start + relative_end];
                let number = name.parse::<usize>().ok();
                let value = if name == "0" {
                    captures.name(actual_name)
                } else if name == assertion_name {
                    None
                } else {
                    captures.name(name)
                };
                if name == assertion_name {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "replacement refers to an internal regex capture",
                    ));
                }
                if !name.is_empty() && value.is_none() && name != "0" && number.is_none() {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        format!("unknown regex capture group {name}"),
                    ));
                }
                if name != "0" {
                    if let Some(number) = number {
                        if number > original_groups {
                            return Err(fault(
                                "Ljava/lang/IndexOutOfBoundsException;",
                                format!("no regex capture group {number}"),
                            ));
                        }
                        let value = captures.get(number + 1);
                        if let Some(value) = value {
                            append_limited(output, value.as_str())?;
                        }
                    } else if let Some(value) = value {
                        append_limited(output, value.as_str())?;
                    }
                } else if let Some(value) = value {
                    append_limited(output, &source[value.start()..value.end()])?;
                }
                index = name_start + relative_end + 1;
                continue;
            }
        } else if replacement[after_dollar..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
        {
            let digits_end = replacement[after_dollar..]
                .char_indices()
                .take_while(|(_, character)| character.is_ascii_digit())
                .map(|(offset, character)| after_dollar + offset + character.len_utf8())
                .last()
                .unwrap();
            let digits = &replacement[after_dollar..digits_end];
            let selected = (1..=digits.len()).rev().find_map(|length| {
                digits[..length]
                    .parse::<usize>()
                    .ok()
                    .filter(|number| *number <= original_groups)
                    .map(|number| (length, number))
            });
            if let Some((length, number)) = selected {
                if let Some(value) = captures.get(number + 1) {
                    append_limited(output, value.as_str())?;
                }
                index = after_dollar + length;
                continue;
            }
        }
        append_limited(output, "$")?;
        index = after_dollar;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use droidless_formats::{apk::Apk, dex::Method};

    fn runtime() -> Runtime {
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
            .unwrap()
    }
    fn call(
        vm: &mut Runtime,
        class: &str,
        name: &str,
        parameters: &[&str],
        returns: &str,
        args: &[Word],
    ) -> Vec<Word> {
        vm.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters: parameters.iter().map(|p| (*p).into()).collect(),
                returns: returns.into(),
            },
            args.to_vec(),
            false,
        )
        .unwrap()
    }
    fn compile(vm: &mut Runtime, source: &str) -> Word {
        let source = vm.heap.string(source.into()).unwrap();
        call(
            vm,
            "Ljava/util/regex/Pattern;",
            "compile",
            &["Ljava/lang/String;"],
            "Ljava/util/regex/Pattern;",
            &[source],
        )[0]
    }
    fn make_matcher(vm: &mut Runtime, pattern: Word, input: &str) -> Word {
        let input = vm.heap.string(input.into()).unwrap();
        call(
            vm,
            "Ljava/util/regex/Pattern;",
            "matcher",
            &["Ljava/lang/CharSequence;"],
            "Ljava/util/regex/Matcher;",
            &[pattern, input],
        )[0]
    }
    #[test]
    fn string_replace_all_expands_capture_groups() {
        let mut vm = runtime();
        let source = vm.heap.string("note 12".into()).unwrap();
        let pattern = vm.heap.string("([0-9]+)".into()).unwrap();
        let replacement = vm.heap.string("#$1".into()).unwrap();
        let output = call(
            &mut vm,
            "Ljava/lang/String;",
            "replaceAll",
            &["Ljava/lang/String;", "Ljava/lang/String;"],
            "Ljava/lang/String;",
            &[source, pattern, replacement],
        )[0];
        assert_eq!(vm.heap.text(output).unwrap(), "note #12");
    }

    #[test]
    fn replace_all_supports_terminal_lookahead_and_preserves_its_text() {
        let mut vm = runtime();
        let source = vm.heap.string("a   b".into()).unwrap();
        let pattern = vm.heap.string(" +(?= |$)".into()).unwrap();
        let replacement = vm.heap.string("_".into()).unwrap();
        let output = call(
            &mut vm,
            "Ljava/lang/String;",
            "replaceAll",
            &["Ljava/lang/String;", "Ljava/lang/String;"],
            "Ljava/lang/String;",
            &[source, pattern, replacement],
        )[0];
        assert_eq!(vm.heap.text(output).unwrap(), "a_ b");

        let source = vm.heap.string("abc! x".into()).unwrap();
        let pattern = vm.heap.string("([a-z]+)(?=!)".into()).unwrap();
        let replacement = vm.heap.string("<$0:$1>".into()).unwrap();
        let output = call(
            &mut vm,
            "Ljava/lang/String;",
            "replaceAll",
            &["Ljava/lang/String;", "Ljava/lang/String;"],
            "Ljava/lang/String;",
            &[source, pattern, replacement],
        )[0];
        assert_eq!(vm.heap.text(output).unwrap(), "<abc:abc>! x");
    }

    #[test]
    fn replace_all_handles_notepad_rich_text_whitespace_pattern() {
        let mut vm = runtime();
        let source = vm.heap.string("  Notes survive restart   ".into()).unwrap();
        let pattern = vm.heap.string("(?m)(^ *| +(?= |$))".into()).unwrap();
        let replacement = vm.heap.string(String::new()).unwrap();
        let output = call(
            &mut vm,
            "Ljava/lang/String;",
            "replaceAll",
            &["Ljava/lang/String;", "Ljava/lang/String;"],
            "Ljava/lang/String;",
            &[source, pattern, replacement],
        )[0];
        assert_eq!(vm.heap.text(output).unwrap(), "Notes survive restart");
    }
    fn string(vm: &mut Runtime, class: &str, matcher: Word, group: Option<i32>) -> Option<String> {
        let (name, parameters, args) = match group {
            Some(index) => ("group", vec!["I"], vec![matcher, Word::from(index)]),
            None => ("group", vec![], vec![matcher]),
        };
        let value = call(vm, class, name, &parameters, "Ljava/lang/String;", &args)[0];
        if value == Word::ZERO {
            None
        } else {
            Some(vm.heap.text(value).unwrap().to_owned())
        }
    }

    #[test]
    fn regex_find_matches_groups_gc_and_syntax_faults() {
        let mut vm = runtime();
        let pattern = compile(&mut vm, "([a-z]+)-([0-9]+)");
        let matcher = make_matcher(&mut vm, pattern, "id-12, ref-34");
        vm.heap.collect([matcher]);
        assert_eq!(
            call(
                &mut vm,
                "Ljava/util/regex/Matcher;",
                "find",
                &[],
                "Z",
                &[matcher],
            )[0]
            .int()
            .unwrap(),
            1
        );
        assert_eq!(
            string(&mut vm, "Ljava/util/regex/Matcher;", matcher, Some(0)).as_deref(),
            Some("id-12")
        );
        assert_eq!(
            string(&mut vm, "Ljava/util/regex/Matcher;", matcher, Some(1)).as_deref(),
            Some("id")
        );
        assert_eq!(
            string(&mut vm, "Ljava/util/regex/Matcher;", matcher, Some(2)).as_deref(),
            Some("12")
        );
        assert_eq!(
            call(
                &mut vm,
                "Ljava/util/regex/Matcher;",
                "find",
                &[],
                "Z",
                &[matcher],
            )[0]
            .int()
            .unwrap(),
            1
        );
        assert_eq!(
            string(&mut vm, "Ljava/util/regex/Matcher;", matcher, Some(2)).as_deref(),
            Some("34")
        );
        assert_eq!(
            call(
                &mut vm,
                "Ljava/util/regex/Matcher;",
                "find",
                &[],
                "Z",
                &[matcher],
            )[0]
            .int()
            .unwrap(),
            0
        );
        let exact = make_matcher(&mut vm, pattern, "ref-34");
        assert_eq!(
            call(
                &mut vm,
                "Ljava/util/regex/Matcher;",
                "matches",
                &[],
                "Z",
                &[exact],
            )[0]
            .int()
            .unwrap(),
            1
        );
        assert_eq!(
            string(&mut vm, "Ljava/util/regex/Matcher;", exact, None).as_deref(),
            Some("ref-34")
        );
        let invalid = vm.heap.string("[".into()).unwrap();
        assert!(
            vm.invoke(
                Method {
                    class: "Ljava/util/regex/Pattern;".into(),
                    name: "compile".into(),
                    parameters: vec!["Ljava/lang/String;".into()],
                    returns: "Ljava/util/regex/Pattern;".into(),
                },
                vec![invalid],
                false,
            )
            .unwrap_err()
            .downcast_ref::<crate::heap::GuestFault>()
            .is_some_and(|error| error.0 == "Ljava/util/regex/PatternSyntaxException;")
        );
    }
}
