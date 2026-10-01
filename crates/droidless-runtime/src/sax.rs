use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{dex::Method, xml::Value};
use quick_xml::{Reader, XmlVersion, escape::resolve_predefined_entity, events::Event};
use std::collections::BTreeMap;

const MAX_XML_BYTES: usize = 1_048_576;
const MAX_XML_EVENTS: usize = 100_000;
const MAX_XML_DEPTH: usize = 256;
const MAX_XML_ATTRIBUTES: usize = 4096;
const INPUT_READER: &str = "droidless:sax:reader";

enum SaxEvent {
    Start(String, BTreeMap<String, String>),
    End(String),
    Text(String),
}

fn parse_events(source: &str) -> Result<Vec<SaxEvent>> {
    ensure!(source.len() <= MAX_XML_BYTES, "XML input exceeds 1 MiB");
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    reader.config_mut().check_end_names = true;
    let mut stack = Vec::new();
    let mut root_seen = false;
    let mut events = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(tag) => {
                let (name, attributes) = tag_data(&tag)?;
                if stack.is_empty() {
                    ensure!(!root_seen, "XML has multiple roots");
                    root_seen = true;
                }
                ensure!(stack.len() < MAX_XML_DEPTH, "XML nesting limit reached");
                push_event(&mut events, SaxEvent::Start(name.clone(), attributes))?;
                stack.push(name);
            }
            Event::Empty(tag) => {
                let (name, attributes) = tag_data(&tag)?;
                if stack.is_empty() {
                    ensure!(!root_seen, "XML has multiple roots");
                    root_seen = true;
                }
                push_event(&mut events, SaxEvent::Start(name.clone(), attributes))?;
                push_event(&mut events, SaxEvent::End(name))?;
            }
            Event::End(tag) => {
                let name = tag.name().as_ref().to_owned();
                ensure!(
                    stack.pop().as_deref() == Some(name.as_str()),
                    "mismatched XML end tag"
                );
                push_event(&mut events, SaxEvent::End(name))?;
            }
            Event::Text(text) => {
                let text = text.xml10_content().into_owned();
                if stack.is_empty() {
                    ensure!(text.trim().is_empty(), "text outside XML root");
                } else if !text.is_empty() {
                    push_event(&mut events, SaxEvent::Text(text))?;
                }
            }
            Event::CData(text) => {
                let text = text.as_ref().to_owned();
                ensure!(!stack.is_empty(), "CDATA outside XML root");
                if !text.is_empty() {
                    push_event(&mut events, SaxEvent::Text(text))?;
                }
            }
            Event::GeneralRef(reference) => {
                let text = if let Some(character) = reference.resolve_char_ref()? {
                    character.to_string()
                } else {
                    resolve_predefined_entity(reference.as_ref())
                        .context("unsupported XML entity")?
                        .to_owned()
                };
                ensure!(!stack.is_empty(), "entity outside XML root");
                push_event(&mut events, SaxEvent::Text(text))?;
            }
            Event::DocType(_) => bail!("XML document types are unsupported"),
            Event::Decl(_) | Event::PI(_) | Event::Comment(_) => {}
            Event::Eof => break,
        }
    }
    ensure!(root_seen && stack.is_empty(), "incomplete XML document");
    Ok(events)
}

fn tag_data(tag: &quick_xml::events::BytesStart<'_>) -> Result<(String, BTreeMap<String, String>)> {
    let name = tag.name().as_ref().to_owned();
    let mut attributes = BTreeMap::new();
    for attribute in tag.attributes().with_checks(true) {
        let attribute = attribute?;
        ensure!(
            attributes.len() < MAX_XML_ATTRIBUTES,
            "XML attribute limit reached"
        );
        let name = attribute.key.as_ref().to_owned();
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)?
            .into_owned();
        ensure!(
            attributes.insert(name, value).is_none(),
            "duplicate XML attribute"
        );
    }
    Ok((name, attributes))
}

fn push_event(events: &mut Vec<SaxEvent>, event: SaxEvent) -> Result<()> {
    ensure!(events.len() < MAX_XML_EVENTS, "XML event limit reached");
    events.push(event);
    Ok(())
}

impl Runtime {
    pub(crate) fn sax_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |index| args.get(index).copied().context("SAX argument missing");
        match (method.class.as_str(), method.signature().as_str()) {
            (
                "Ljavax/xml/parsers/SAXParserFactory;",
                "newInstance()Ljavax/xml/parsers/SAXParserFactory;",
            ) => Ok(Some(vec![
                self.heap.instance("Ljavax/xml/parsers/SAXParserFactory;")?,
            ])),
            (
                "Ljavax/xml/parsers/SAXParserFactory;",
                "newSAXParser()Ljavax/xml/parsers/SAXParser;",
            ) => Ok(Some(vec![
                self.heap.instance("Ljavax/xml/parsers/SAXParser;")?,
            ])),
            ("Ljava/io/StringReader;", "<init>(Ljava/lang/String;)V") => {
                let contents = self.heap.text(arg(1)?)?.to_owned();
                self.heap.get_mut(arg(0)?)?.data = Data::String(contents);
                Ok(Some(vec![]))
            }
            ("Lorg/xml/sax/InputSource;", "<init>(Ljava/io/Reader;)V") => {
                self.heap
                    .get_mut(arg(0)?)?
                    .fields
                    .insert(INPUT_READER.into(), vec![arg(1)?]);
                Ok(Some(vec![]))
            }
            ("Lorg/xml/sax/helpers/DefaultHandler;", "<init>()V") => Ok(Some(vec![])),
            ("Lorg/xml/sax/Attributes;", "getValue(Ljava/lang/String;)Ljava/lang/String;") => {
                let name = self.heap.text(arg(1)?)?;
                let value = match &self.heap.get(arg(0)?)?.data {
                    Data::Attributes { named, .. } => {
                        named.get(name).and_then(|value| value.text.clone())
                    }
                    _ => bail!("uninitialized SAX Attributes"),
                };
                Ok(Some(vec![match value {
                    Some(value) => self.heap.string(value)?,
                    None => Word::ZERO,
                }]))
            }
            (
                "Ljavax/xml/parsers/SAXParser;",
                "parse(Lorg/xml/sax/InputSource;Lorg/xml/sax/helpers/DefaultHandler;)V",
            ) => {
                let input = self.heap.get(arg(1)?)?;
                let reader = input
                    .fields
                    .get(INPUT_READER)
                    .and_then(|values| values.first())
                    .copied()
                    .context("uninitialized SAX InputSource")?;
                let source = self.heap.text(reader)?.to_owned();
                let events = parse_events(&source)
                    .map_err(|error| fault("Lorg/xml/sax/SAXException;", error.to_string()))?;
                let handler = arg(2)?;
                for event in events {
                    match event {
                        SaxEvent::Start(name, attributes) => {
                            let attributes = attributes
                                .into_iter()
                                .map(|(name, value)| {
                                    (
                                        name,
                                        Value {
                                            kind: 3,
                                            data: 0,
                                            text: Some(value),
                                        },
                                    )
                                })
                                .collect();
                            let attributes_object =
                                self.heap.instance("Lorg/xml/sax/Attributes;")?;
                            self.heap.get_mut(attributes_object)?.data = Data::Attributes {
                                named: attributes,
                                resources: Default::default(),
                            };
                            let uri = self.heap.string(String::new())?;
                            let local_name = self.heap.string(String::new())?;
                            let qualified_name = self.heap.string(name)?;
                            self.handler_call(
                                handler,
                                "startElement",
                                vec![
                                    "Ljava/lang/String;".into(),
                                    "Ljava/lang/String;".into(),
                                    "Ljava/lang/String;".into(),
                                    "Lorg/xml/sax/Attributes;".into(),
                                ],
                                vec![uri, local_name, qualified_name, attributes_object],
                            )?;
                        }
                        SaxEvent::End(name) => {
                            let name = self.heap.string(name)?;
                            let uri = self.heap.string(String::new())?;
                            let local_name = self.heap.string(String::new())?;
                            self.handler_call(
                                handler,
                                "endElement",
                                vec![
                                    "Ljava/lang/String;".into(),
                                    "Ljava/lang/String;".into(),
                                    "Ljava/lang/String;".into(),
                                ],
                                vec![uri, local_name, name],
                            )?;
                        }
                        SaxEvent::Text(text) => {
                            let units = text.encode_utf16().map(i32::from).collect::<Vec<_>>();
                            let length = units.len() as i32;
                            let characters = self.array("C".into(), units.len())?;
                            if let Data::Array { values, .. } =
                                &mut self.heap.get_mut(characters)?.data
                            {
                                for (value, unit) in values.iter_mut().zip(units) {
                                    *value = vec![Word::from(unit)];
                                }
                            }
                            self.handler_call(
                                handler,
                                "characters",
                                vec!["[C".into(), "I".into(), "I".into()],
                                vec![characters, Word::ZERO, Word::from(length)],
                            )?;
                        }
                    }
                }
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }

    fn handler_call(
        &mut self,
        handler: Word,
        name: &str,
        parameters: Vec<String>,
        arguments: Vec<Word>,
    ) -> Result<()> {
        self.invoke(
            Method {
                class: "Lorg/xml/sax/helpers/DefaultHandler;".into(),
                name: name.into(),
                parameters,
                returns: "V".into(),
            },
            std::iter::once(handler).chain(arguments).collect(),
            true,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sax_events_decode_xml_text_and_attributes_and_reject_dtds() {
        let events =
            parse_events("<root style=\"bold &amp; clear\"><b>one &lt; two</b><br/></root>")
                .unwrap();
        assert!(matches!(
            &events[0],
            SaxEvent::Start(name, attributes)
                if name == "root" && attributes.get("style").is_some_and(|value| value == "bold & clear")
        ));
        let text = events
            .iter()
            .filter_map(|event| match event {
                SaxEvent::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>();
        assert_eq!(text, "one < two");
        assert!(parse_events("<!DOCTYPE root><root/>").is_err());
        let mut vm = Runtime::new(
            droidless_formats::apk::Apk::parse(include_bytes!(
                "../../../fixtures/generated/counter.apk"
            ))
            .unwrap(),
        )
        .unwrap();
        let exception = vm.heap.instance("Lorg/xml/sax/SAXException;").unwrap();
        assert!(vm.is_a("Lorg/xml/sax/SAXException;", "Ljava/lang/Exception;"));
        assert!(
            vm.throw_reference(exception)
                .unwrap()
                .to_string()
                .contains("uncaught guest exception")
        );
    }
}
