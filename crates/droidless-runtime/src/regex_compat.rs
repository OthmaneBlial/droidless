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
