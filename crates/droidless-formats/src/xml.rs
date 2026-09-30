use crate::binary::{Bytes, string_at, string_pool};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Value {
    pub kind: u8,
    pub data: u32,
    pub text: Option<String>,
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
    let root = Bytes(data).chunk(0)?;
    ensure!(
        root.kind == 3 && root.bytes.0.len() == data.len(),
        "expected binary Android XML"
    );
    let mut strings = vec![];
    let mut stack: Vec<Element> = vec![];
    let mut document = None;
    for chunk in root.children()? {
        let b = chunk.bytes;
        match chunk.kind {
            1 => {
                ensure!(strings.is_empty(), "duplicate XML string pool");
                strings = string_pool(&chunk)?;
            }
            0x100 | 0x101 | 0x180 => {}
            0x102 => {
                ensure!(chunk.header == 16, "invalid start element header");
                let name = string_at(&strings, b.u32(20)?)?.to_owned();
                let start = 16 + usize::from(b.u16(24)?);
                let stride = usize::from(b.u16(26)?);
                let count = usize::from(b.u16(28)?);
                ensure!(
                    start >= 36 && stride >= 20 && stack.len() < 256,
                    "invalid attributes or XML nesting too deep"
                );
                b.table(start, count, stride)?;
                let mut attributes = BTreeMap::new();
                for i in 0..count {
                    let at = start + i * stride;
                    let key = string_at(&strings, b.u32(at + 4)?)?.to_owned();
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
                stack.push(Element {
                    name,
                    attributes,
                    children: vec![],
                });
            }
            0x103 => {
                let element = stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("unbalanced XML end tag"))?;
                ensure!(
                    string_at(&strings, b.u32(20)?)? == element.name,
                    "mismatched XML end tag"
                );
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    ensure!(document.is_none(), "multiple XML roots");
                    document = Some(element);
                }
            }
            0x104 => {
                b.slice(16, 12)?;
            }
            _ => anyhow::bail!("unsupported binary XML chunk 0x{:04x}", chunk.kind),
        }
    }
    ensure!(stack.is_empty(), "unclosed XML tags");
    document.ok_or_else(|| anyhow::anyhow!("XML has no root"))
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
