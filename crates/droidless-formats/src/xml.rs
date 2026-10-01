use crate::binary::{Bytes, string_at, string_pool};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Value {
    pub kind: u8,
    pub data: u32,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PullAttribute {
    pub namespace: Option<String>,
    pub prefix: Option<String>,
    pub name: String,
    pub name_resource: u32,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PullEvent {
    pub kind: u8,
    pub depth: usize,
    pub line: i32,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub prefix: Option<String>,
    pub text: Option<String>,
    pub attributes: Vec<PullAttribute>,
    pub namespaces: Vec<(usize, Option<String>, String)>,
}
impl Value {
    pub fn display(&self) -> String {
        self.text.clone().unwrap_or_else(|| match self.kind {
            1 => format!("@0x{:08x}", self.data),
            0x12 => (self.data != 0).to_string(),
            _ => self.data.to_string(),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Element {
    pub name: String,
    pub attributes: BTreeMap<String, Value>,
    pub resource_attributes: BTreeMap<u32, Value>,
    pub children: Vec<Element>,
}
impl Element {
    pub fn attr(&self, name: &str) -> Option<&Value> {
        self.attributes.get(name)
    }
    pub fn text(&self, name: &str) -> Option<String> {
        self.attr(name).map(Value::display)
    }
    pub fn number(&self, name: &str) -> Option<u32> {
        self.attr(name).map(|v| v.data)
    }
}

pub fn parse(data: &[u8]) -> Result<Element> {
    parse_document(data).map(|(document, _)| document)
}

pub fn parse_events(data: &[u8]) -> Result<Vec<PullEvent>> {
    parse_document(data).map(|(_, events)| events)
}

fn parse_document(data: &[u8]) -> Result<(Element, Vec<PullEvent>)> {
    let root = Bytes(data).chunk(0)?;
    ensure!(
        root.kind == 3 && root.bytes.0.len() == data.len(),
        "expected binary Android XML"
    );
    let mut strings = vec![];
    let mut resource_map = None;
    let mut stack: Vec<Element> = vec![];
    let mut document = None;
    let mut namespaces: Vec<(usize, Option<String>, String)> = vec![];
    let mut events = vec![PullEvent {
        kind: 0,
        depth: 0,
        line: -1,
        name: None,
        namespace: None,
        prefix: None,
        text: None,
        attributes: vec![],
        namespaces: vec![],
    }];
    for chunk in root.children()? {
        let b = chunk.bytes;
        match chunk.kind {
            1 => {
                ensure!(strings.is_empty(), "duplicate XML string pool");
                strings = string_pool(&chunk)?;
            }
            0x100 => {
                ensure!(namespaces.len() < 64, "XML namespace limit reached (64)");
                let prefix = b.u32(16)?;
                let uri = string_at(&strings, b.u32(20)?)?.to_owned();
                namespaces.push((
                    stack.len() + 1,
                    (prefix != u32::MAX)
                        .then(|| string_at(&strings, prefix).map(str::to_owned))
                        .transpose()?,
                    uri,
                ));
            }
            0x101 => {
                let prefix = b.u32(16)?;
                let uri = string_at(&strings, b.u32(20)?)?;
                let expected = (
                    (prefix != u32::MAX)
                        .then(|| string_at(&strings, prefix).map(str::to_owned))
                        .transpose()?,
                    uri.to_owned(),
                );
                let active = namespaces.pop();
                ensure!(
                    active
                        .as_ref()
                        .is_some_and(|(_, prefix, uri)| (prefix, uri) == (&expected.0, &expected.1)),
                    "unbalanced XML namespace"
                );
            }
            0x180 => {
                let payload = b.0.len() - chunk.header;
                ensure!(
                    chunk.header == 8 && payload % 4 == 0 && payload / 4 <= strings.len(),
                    "invalid XML resource map"
                );
                ensure!(resource_map.is_none(), "duplicate XML resource map");
                resource_map = Some(
                    (0..payload / 4)
                        .map(|i| b.u32(chunk.header + i * 4))
                        .collect::<Result<Vec<_>>>()?,
                );
            }
            0x102 => {
                ensure!(chunk.header == 16, "invalid start element header");
                ensure!(events.len() < 100_000, "XML event limit reached (100000)");
                let namespace = b.u32(16)?;
                let namespace = (namespace != u32::MAX)
                    .then(|| string_at(&strings, namespace).map(str::to_owned))
                    .transpose()?;
                let name = string_at(&strings, b.u32(20)?)?.to_owned();
                let prefix = namespace.as_ref().and_then(|uri| {
                    namespaces
                        .iter()
                        .rev()
                        .find(|(_, _, active_uri)| active_uri == uri)
                        .and_then(|(_, prefix, _)| prefix.clone())
                });
                let start = 16 + usize::from(b.u16(24)?);
                let stride = usize::from(b.u16(26)?);
                let count = usize::from(b.u16(28)?);
                ensure!(
                    start >= 36 && stride >= 20 && stack.len() < 256 && count <= 1024,
                    "invalid attributes or XML nesting too deep"
                );
                b.table(start, count, stride)?;
                let mut attributes = BTreeMap::new();
                let mut resource_attributes = BTreeMap::new();
                let mut pull_attributes = Vec::with_capacity(count);
                for i in 0..count {
                    let at = start + i * stride;
                    let attribute_namespace = b.u32(at)?;
                    let attribute_namespace = (attribute_namespace != u32::MAX)
                        .then(|| string_at(&strings, attribute_namespace).map(str::to_owned))
                        .transpose()?;
                    let name_index = b.u32(at + 4)?;
                    let key = string_at(&strings, name_index)?.to_owned();
                    let name_resource = resource_map
                        .as_ref()
                        .and_then(|map| map.get(name_index as usize))
                        .copied()
                        .unwrap_or(0);
                    let raw = b.u32(at + 8)?;
                    ensure!(b.u16(at + 12)? == 8, "invalid XML typed value");
                    let kind = b.u8(at + 15)?;
                    let value = b.u32(at + 16)?;
                    let text = if raw != u32::MAX {
                        Some(string_at(&strings, raw)?.to_owned())
                    } else if kind == 3 {
                        Some(string_at(&strings, value)?.to_owned())
                    } else {
                        None
                    };
                    let prefix = attribute_namespace.as_ref().and_then(|uri| {
                        namespaces
                            .iter()
                            .rev()
                            .find(|(_, _, active_uri)| active_uri == uri)
                            .and_then(|(_, prefix, _)| prefix.clone())
                    });
                    pull_attributes.push(PullAttribute {
                        namespace: attribute_namespace,
                        prefix,
                        name: key.clone(),
                        name_resource,
                        value: Value {
                            kind,
                            data: value,
                            text: text.clone(),
                        },
                    });
                    if name_resource != 0 {
                        resource_attributes.insert(
                            name_resource,
                            Value {
                                kind,
                                data: value,
                                text: text.clone(),
                            },
                        );
                    }
                    ensure!(
                        attributes
                            .insert(
                                key,
                                Value {
                                    kind,
                                    data: value,
                                    text
                                }
                            )
                            .is_none(),
                        "duplicate XML attribute"
                    );
                }
                events.push(PullEvent {
                    kind: 2,
                    depth: stack.len() + 1,
                    line: b.u32(8)? as i32,
                    name: Some(name.clone()),
                    namespace,
                    prefix,
                    text: None,
                    attributes: pull_attributes,
                    namespaces: namespaces.clone(),
                });
                stack.push(Element {
                    name,
                    attributes,
                    resource_attributes,
                    children: vec![],
                });
            }
            0x103 => {
                ensure!(events.len() < 100_000, "XML event limit reached (100000)");
                let element = stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("unbalanced XML end tag"))?;
                ensure!(
                    string_at(&strings, b.u32(20)?)? == element.name,
                    "mismatched XML end tag"
                );
                let namespace = b.u32(16)?;
                let namespace = (namespace != u32::MAX)
                    .then(|| string_at(&strings, namespace).map(str::to_owned))
                    .transpose()?;
                let prefix = namespace.as_ref().and_then(|uri| {
                    namespaces
                        .iter()
                        .rev()
                        .find(|(_, _, active_uri)| active_uri == uri)
                        .and_then(|(_, prefix, _)| prefix.clone())
                });
                events.push(PullEvent {
                    kind: 3,
                    depth: stack.len() + 1,
                    line: b.u32(8)? as i32,
                    name: Some(element.name.clone()),
                    namespace,
                    prefix,
                    text: None,
                    attributes: vec![],
                    namespaces: namespaces.clone(),
                });
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    ensure!(document.is_none(), "multiple XML roots");
                    document = Some(element);
                }
            }
            0x104 => {
                ensure!(events.len() < 100_000, "XML event limit reached (100000)");
                b.slice(16, 12)?;
                events.push(PullEvent {
                    kind: 4,
                    depth: stack.len(),
                    line: b.u32(8)? as i32,
                    name: None,
                    namespace: None,
                    prefix: None,
                    text: Some(string_at(&strings, b.u32(16)?)?.to_owned()),
                    attributes: vec![],
                    namespaces: namespaces.clone(),
                });
            }
            _ => anyhow::bail!("unsupported binary XML chunk 0x{:04x}", chunk.kind),
        }
    }
    ensure!(stack.is_empty(), "unclosed XML tags");
    ensure!(namespaces.is_empty(), "unclosed XML namespace");
    let document = document.ok_or_else(|| anyhow::anyhow!("XML has no root"))?;
    events.push(PullEvent {
        kind: 1,
        depth: 0,
        line: -1,
        name: None,
        namespace: None,
        prefix: None,
        text: None,
        attributes: vec![],
        namespaces: vec![],
    });
    Ok((document, events))
}

#[derive(Debug, Serialize)]
pub struct Manifest {
    pub package: String,
    pub version_name: Option<String>,
    pub version_code: Option<u32>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub application: Option<String>,
    pub activities: Vec<String>,
    pub main_activity: Option<String>,
    pub permissions: Vec<String>,
    pub services: Vec<String>,
    pub receivers: Vec<String>,
    pub providers: Vec<String>,
    pub label: Option<Value>,
    pub document: Element,
}
impl Manifest {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let document = parse(data)?;
        ensure!(document.name == "manifest", "manifest XML root missing");
        let package = document
            .text("package")
            .ok_or_else(|| anyhow::anyhow!("manifest package missing"))?;
        ensure!(!package.is_empty(), "empty package name");
        let qualify = |s: String| {
            if s.starts_with('.') {
                format!("{package}{s}")
            } else if !s.contains('.') {
                format!("{package}.{s}")
            } else {
                s
            }
        };
        let mut result = Self {
            version_name: document.text("versionName"),
            version_code: document.number("versionCode"),
            min_sdk: None,
            target_sdk: None,
            application: None,
            activities: vec![],
            main_activity: None,
            permissions: vec![],
            services: vec![],
            receivers: vec![],
            providers: vec![],
            label: None,
            package: package.clone(),
            document: document.clone(),
        };
        for node in &document.children {
            match node.name.as_str() {
                "uses-sdk" => {
                    result.min_sdk = node.number("minSdkVersion");
                    result.target_sdk = node.number("targetSdkVersion");
                }
                "uses-permission" | "uses-permission-sdk-23" => {
                    if let Some(p) = node.text("name") {
                        result.permissions.push(p);
                    }
                }
                "application" => {
                    result.application = node.text("name").map(qualify);
                    result.label = node.attr("label").cloned();
                    for child in &node.children {
                        let Some(name) = child.text("name").map(qualify) else {
                            continue;
                        };
                        match child.name.as_str() {
                            "activity" | "activity-alias" => {
                                if child.name == "activity" {
                                    result.activities.push(name.clone());
                                }
                                for filter in
                                    child.children.iter().filter(|c| c.name == "intent-filter")
                                {
                                    let main = filter.children.iter().any(|c| {
                                        c.name == "action"
                                            && c.text("name").as_deref()
                                                == Some("android.intent.action.MAIN")
                                    });
                                    let launcher = filter.children.iter().any(|c| {
                                        c.name == "category"
                                            && c.text("name").as_deref()
                                                == Some("android.intent.category.LAUNCHER")
                                    });
                                    if main
                                        && launcher
                                        && child.text("enabled").as_deref() != Some("false")
                                    {
                                        result.main_activity = Some(
                                            child
                                                .text("targetActivity")
                                                .map(qualify)
                                                .unwrap_or(name.clone()),
                                        );
                                    }
                                }
                            }
                            "service" => result.services.push(name),
                            "receiver" => result.receivers.push(name),
                            "provider" => result.providers.push(name),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }
}
