use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{dex::Method, xml::PullEvent};

const RESOURCES: &str = "Landroid/content/res/Resources;";
const XML_PARSER: &str = "Landroid/content/res/XmlResourceParser;";
const XML_PULL_PARSER: &str = "Lorg/xmlpull/v1/XmlPullParser;";

impl Runtime {
    pub(crate) fn xml_resource_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        if method.class == "Landroid/util/Xml;"
            && signature
                == "asAttributeSet(Lorg/xmlpull/v1/XmlPullParser;)Landroid/util/AttributeSet;"
        {
            let parser = args
                .first()
                .copied()
                .context("Xml.asAttributeSet parser missing")?;
            ensure!(
                matches!(&self.heap.get(parser)?.data, Data::XmlPull { .. }),
                "Xml.asAttributeSet requires an XmlPullParser"
            );
            return Ok(Some(vec![parser]));
        }
        if method.class == RESOURCES
            && signature == "getXml(I)Landroid/content/res/XmlResourceParser;"
        {
            let id = args
                .get(1)
                .copied()
                .context("Resources.getXml resource ID missing")?
                .int()? as u32;
            let value = self.apk.resources.resolve(id)?.clone();
            let path = value
                .text
                .as_deref()
                .context("XML resource does not name an APK file")?;
            ensure!(
                value.kind == 3
                    && path.starts_with("res/")
                    && path.ends_with(".xml")
                    && path
                        .split('/')
                        .all(|part| !part.is_empty() && part != "." && part != "..")
                    && !path.contains(['\\', '\0']),
                "invalid XML resource path"
            );
            let bytes = self
                .apk
                .files
                .get(path)
                .context("XML resource file missing")?;
            let events = droidless_formats::xml::parse_events(bytes)?;
            let parser = self.heap.instance(XML_PARSER)?;
            self.heap.get_mut(parser)?.data = Data::XmlPull {
                events,
                position: 0,
                closed: false,
            };
            return Ok(Some(vec![parser]));
        }
        if ![XML_PARSER, XML_PULL_PARSER].contains(&method.class.as_str()) {
            return Ok(None);
        }

        let receiver = *args.first().context("XmlResourceParser receiver missing")?;
        let event = self.xml_event(receiver)?;
        let arg = |index| -> Result<Word> {
            args.get(index)
                .copied()
                .context("XML parser argument missing")
        };
        let result = match signature.as_str() {
            "close()V" => {
                let Data::XmlPull { closed, .. } = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("uninitialized XmlResourceParser");
                };
                *closed = true;
                vec![]
            }
            "getEventType()I" => vec![Word::from(event.kind as i32)],
            "next()I" | "nextToken()I" => {
                vec![Word::from(self.next_xml_event(receiver)?.kind as i32)]
            }
            "nextTag()I" => {
                let mut next = self.next_xml_event(receiver)?;
                while next.kind == 4
                    && next
                        .text
                        .as_deref()
                        .is_some_and(|text| text.trim().is_empty())
                {
                    next = self.next_xml_event(receiver)?;
                }
                ensure!(
                    next.kind == 2 || next.kind == 3,
                    fault(
                        "Lorg/xmlpull/v1/XmlPullParserException;",
                        "expected start or end tag"
                    )
                );
                vec![Word::from(next.kind as i32)]
            }
            "nextText()Ljava/lang/String;" => {
                ensure!(
                    event.kind == 2,
                    fault(
                        "Ljava/lang/IllegalStateException;",
                        "nextText requires a start tag"
                    )
                );
                let text_event = self.next_xml_event(receiver)?;
                let text = if text_event.kind == 4 {
                    let value = text_event.text.unwrap_or_default();
                    let end = self.next_xml_event(receiver)?;
                    ensure!(
                        end.kind == 3,
                        fault(
                            "Lorg/xmlpull/v1/XmlPullParserException;",
                            "text is not followed by an end tag"
                        )
                    );
                    value
                } else if text_event.kind == 3 {
                    String::new()
                } else {
                    bail!(fault(
                        "Lorg/xmlpull/v1/XmlPullParserException;",
                        "expected text or end tag"
                    ))
                };
                vec![self.heap.string(text)?]
            }
            "getName()Ljava/lang/String;" => vec![self.optional_string(event.name)?],
            "getNamespace()Ljava/lang/String;" => vec![self.optional_string(event.namespace)?],
            "getPrefix()Ljava/lang/String;" => vec![self.optional_string(event.prefix)?],
            "getText()Ljava/lang/String;" => vec![self.optional_string(event.text)?],
            "getDepth()I" => vec![Word::from(event.depth as i32)],
            "getLineNumber()I" => vec![Word::from(event.line)],
            "getPositionDescription()Ljava/lang/String;" => vec![
                self.heap
                    .string(format!("Binary XML line {}", event.line))?,
            ],
            "isWhitespace()Z" => vec![Word::from(i32::from(
                event.kind == 4
                    && event
                        .text
                        .as_deref()
                        .is_some_and(|text| text.trim().is_empty()),
            ))],
            "isEmptyElementTag()Z" => {
                let next = self.xml_event_at(receiver, 1)?;
                vec![Word::from(i32::from(
                    event.kind == 2 && next.kind == 3 && next.depth == event.depth,
                ))]
            }
            "getInputEncoding()Ljava/lang/String;" => vec![Word::ZERO],
            "getAttributeCount()I" => vec![Word::from(if event.kind == 2 {
                event.attributes.len() as i32
            } else {
                -1
            })],
            "getAttributeName(I)Ljava/lang/String;"
            | "getAttributeNamespace(I)Ljava/lang/String;"
            | "getAttributePrefix(I)Ljava/lang/String;"
            | "getAttributeValue(I)Ljava/lang/String;"
            | "getAttributeType(I)Ljava/lang/String;"
            | "getAttributeNameResource(I)I"
            | "getAttributeResourceValue(II)I"
            | "getAttributeIntValue(II)I"
            | "getAttributeUnsignedIntValue(II)I"
            | "getAttributeBooleanValue(IZ)Z"
            | "getAttributeFloatValue(IF)F"
            | "isAttributeDefault(I)Z" => {
                let index =
                    usize::try_from(arg(1)?.int()?).context("negative XML attribute index")?;
                let attribute = event.attributes.get(index).ok_or_else(|| {
                    fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "XML attribute index out of bounds",
                    )
                })?;
                match signature.as_str() {
                    "getAttributeName(I)Ljava/lang/String;" => {
                        vec![self.heap.string(attribute.name.clone())?]
                    }
                    "getAttributeNamespace(I)Ljava/lang/String;" => {
                        vec![self.optional_string(attribute.namespace.clone())?]
                    }
                    "getAttributePrefix(I)Ljava/lang/String;" => {
                        vec![self.optional_string(attribute.prefix.clone())?]
                    }
                    "getAttributeValue(I)Ljava/lang/String;" => {
                        vec![self.heap.string(attribute.value.display())?]
                    }
                    "getAttributeType(I)Ljava/lang/String;" => {
                        vec![self.heap.string("CDATA".into())?]
                    }
                    "getAttributeNameResource(I)I" => vec![Word::ZERO],
                    "getAttributeResourceValue(II)I" => {
                        vec![Word::from(if attribute.value.kind == 1 {
                            attribute.value.data as i32
                        } else {
                            arg(2)?.int()?
                        })]
                    }
                    "getAttributeIntValue(II)I" | "getAttributeUnsignedIntValue(II)I" => {
                        let value = &attribute.value;
                        let parsed = (value.kind == 0x10 || value.kind == 0x11)
                            .then_some(value.data as i32)
                            .or_else(|| value.text.as_deref().and_then(|text| text.parse().ok()));
                        vec![Word::from(parsed.unwrap_or(arg(2)?.int()?))]
                    }
                    "getAttributeBooleanValue(IZ)Z" => {
                        let value = &attribute.value;
                        let parsed = (value.kind == 0x12)
                            .then_some(value.data != 0)
                            .or_else(|| value.text.as_deref().and_then(|text| text.parse().ok()));
                        vec![Word::from(i32::from(parsed.unwrap_or(arg(2)?.int()? != 0)))]
                    }
                    "getAttributeFloatValue(IF)F" => {
                        let fallback = f32::from_bits(arg(2)?.int()? as u32);
                        let value = &attribute.value;
                        let parsed = (value.kind == 0x04)
                            .then(|| f32::from_bits(value.data))
                            .or_else(|| value.text.as_deref().and_then(|text| text.parse().ok()));
                        vec![Word::Bits(parsed.unwrap_or(fallback).to_bits())]
                    }
                    "isAttributeDefault(I)Z" => vec![Word::ZERO],
                    _ => unreachable!(),
                }
            }
            "getAttributeValue(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;" => {
                let namespace = if arg(1)? == Word::ZERO {
                    None
                } else {
                    Some(self.heap.text(arg(1)?)?)
                };
                let name = self.heap.text(arg(2)?)?;
                let value = event.attributes.iter().find(|attribute| {
                    attribute.name == name
                        && attribute.namespace.as_deref().unwrap_or("") == namespace.unwrap_or("")
                });
                vec![match value {
                    Some(attribute) => self.heap.string(attribute.value.display())?,
                    None => Word::ZERO,
                }]
            }
            "getNamespaceCount(I)I" => {
                let depth = usize::try_from(arg(1)?.int()?).context("negative XML depth")?;
                vec![Word::from(
                    event
                        .namespaces
                        .iter()
                        .filter(|(declared_at, _, _)| *declared_at <= depth)
                        .count() as i32,
                )]
            }
            "getNamespacePrefix(I)Ljava/lang/String;" | "getNamespaceUri(I)Ljava/lang/String;" => {
                let index =
                    usize::try_from(arg(1)?.int()?).context("negative XML namespace index")?;
                let namespaces = event
                    .namespaces
                    .iter()
                    .filter(|(declared_at, _, _)| *declared_at <= event.depth)
                    .collect::<Vec<_>>();
                let (_, prefix, uri) = namespaces.get(index).copied().ok_or_else(|| {
                    fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "XML namespace index out of bounds",
                    )
                })?;
                if signature.starts_with("getNamespacePrefix") {
                    vec![self.optional_string(prefix.clone())?]
                } else {
                    vec![self.heap.string(uri.clone())?]
                }
            }
            "getFeature(Ljava/lang/String;)Z" => vec![Word::ZERO],
            "require(ILjava/lang/String;Ljava/lang/String;)V" => {
                ensure!(
                    event.kind == arg(1)?.int()? as u8,
                    fault(
                        "Lorg/xmlpull/v1/XmlPullParserException;",
                        "unexpected XML event"
                    )
                );
                if arg(2)? != Word::ZERO {
                    ensure!(
                        event.namespace.as_deref() == Some(self.heap.text(arg(2)?)?),
                        fault(
                            "Lorg/xmlpull/v1/XmlPullParserException;",
                            "unexpected XML namespace"
                        )
                    );
                }
                if arg(3)? != Word::ZERO {
                    ensure!(
                        event.name.as_deref() == Some(self.heap.text(arg(3)?)?),
                        fault(
                            "Lorg/xmlpull/v1/XmlPullParserException;",
                            "unexpected XML tag"
                        )
                    );
                }
                vec![]
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn xml_event(&self, parser: Word) -> Result<PullEvent> {
        self.xml_event_at(parser, 0)
    }

    fn xml_event_at(&self, parser: Word, offset: usize) -> Result<PullEvent> {
        let Data::XmlPull {
            events,
            position,
            closed,
        } = &self.heap.get(parser)?.data
        else {
            bail!("uninitialized XmlResourceParser");
        };
        ensure!(
            !closed,
            fault("Ljava/lang/IllegalStateException;", "XML parser is closed")
        );
        events
            .get(position.saturating_add(offset))
            .cloned()
            .context("invalid XML parser position")
    }

    fn next_xml_event(&mut self, parser: Word) -> Result<PullEvent> {
        let Data::XmlPull {
            events,
            position,
            closed,
        } = &mut self.heap.get_mut(parser)?.data
        else {
            bail!("uninitialized XmlResourceParser");
        };
        ensure!(
            !*closed,
            fault("Ljava/lang/IllegalStateException;", "XML parser is closed")
        );
        if *position + 1 < events.len() {
            *position += 1;
        }
        events
            .get(*position)
            .cloned()
            .context("invalid XML parser position")
    }

    fn optional_string(&mut self, value: Option<String>) -> Result<Word> {
        value
            .map(|value| self.heap.string(value))
            .transpose()
            .map(|value| value.unwrap_or(Word::ZERO))
    }
}
