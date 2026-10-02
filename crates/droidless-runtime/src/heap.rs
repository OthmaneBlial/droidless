use crate::ui::View;
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// A Java fault raised by the host bridge. The interpreter materializes it as a
/// guest Throwable at the faulting instruction so normal DEX catches can run.
#[derive(Debug)]
pub(crate) struct GuestFault(pub &'static str, pub String);
impl std::fmt::Display for GuestFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}
impl std::error::Error for GuestFault {}
pub(crate) fn fault(class: &'static str, message: impl Into<String>) -> anyhow::Error {
    GuestFault(class, message.into()).into()
}
pub(crate) fn exception_parent(class: &str) -> Option<&'static str> {
    Some(match class {
        "Ljava/lang/ArithmeticException;"
        | "Ljava/lang/NullPointerException;"
        | "Ljava/lang/ClassCastException;"
        | "Ljava/lang/NegativeArraySizeException;"
        | "Ljava/lang/ArrayStoreException;"
        | "Ljava/lang/IllegalArgumentException;"
        | "Ljava/lang/IllegalStateException;"
        | "Ljava/lang/IllegalMonitorStateException;"
        | "Ljava/lang/UnsupportedOperationException;"
        | "Ljava/util/concurrent/RejectedExecutionException;"
        | "Ljava/util/NoSuchElementException;"
        | "Ljava/util/ConcurrentModificationException;"
        | "Ljava/lang/IndexOutOfBoundsException;" => "Ljava/lang/RuntimeException;",
        "Ljava/lang/NumberFormatException;" | "Ljava/util/regex/PatternSyntaxException;" => {
            "Ljava/lang/IllegalArgumentException;"
        }
        "Ljava/lang/IllegalThreadStateException;" => "Ljava/lang/IllegalArgumentException;",
        "Ljava/util/MissingResourceException;"
        | "Landroid/content/res/Resources$NotFoundException;" => "Ljava/lang/RuntimeException;",
        "Ljava/util/concurrent/CancellationException;" => "Ljava/lang/IllegalStateException;",
        "Ljava/util/concurrent/ExecutionException;" | "Ljava/util/concurrent/TimeoutException;" => {
            "Ljava/lang/Exception;"
        }
        "Ljava/lang/InterruptedException;" | "Lorg/xml/sax/SAXException;" => {
            "Ljava/lang/Exception;"
        }
        "Ljava/lang/ArrayIndexOutOfBoundsException;"
        | "Ljava/lang/StringIndexOutOfBoundsException;" => "Ljava/lang/IndexOutOfBoundsException;",
        "Ljava/lang/RuntimeException;" => "Ljava/lang/Exception;",
        "Landroid/database/sqlite/SQLiteDoneException;" => {
            "Landroid/database/sqlite/SQLiteException;"
        }
        "Landroid/database/sqlite/SQLiteException;" => "Ljava/lang/RuntimeException;",
        "Ljava/lang/ClassNotFoundException;"
        | "Ljava/lang/NoSuchFieldException;"
        | "Ljava/lang/NoSuchMethodException;"
        | "Ljava/lang/reflect/InvocationTargetException;"
        | "Ljava/lang/InstantiationException;"
        | "Ljava/lang/IllegalAccessException;" => "Ljava/lang/ReflectiveOperationException;",
        "Ljava/lang/ReflectiveOperationException;" => "Ljava/lang/Exception;",
        "Ljava/lang/ExceptionInInitializerError;" | "Ljava/lang/NoClassDefFoundError;" => {
            "Ljava/lang/LinkageError;"
        }
        "Ljava/lang/LinkageError;" => "Ljava/lang/Error;",
        "Ljava/lang/NoSuchFieldError;" | "Ljava/lang/IllegalAccessError;" => {
            "Ljava/lang/IncompatibleClassChangeError;"
        }
        "Ljava/lang/IncompatibleClassChangeError;" => "Ljava/lang/LinkageError;",
        "Ljava/lang/AssertionError;" => "Ljava/lang/Error;",
        "Ljava/lang/Error;" => "Ljava/lang/Throwable;",
        "Ljava/lang/Exception;" => "Ljava/lang/Throwable;",
        "Ljava/lang/Throwable;" => "Ljava/lang/Object;",
        "Ljava/io/FileNotFoundException;" => "Ljava/io/IOException;",
        "Ljava/io/UnsupportedEncodingException;" => "Ljava/io/IOException;",
        "Ljava/nio/channels/ClosedChannelException;" => "Ljava/io/IOException;",
        "Ljava/nio/channels/NonReadableChannelException;"
        | "Ljava/nio/channels/NonWritableChannelException;" => "Ljava/lang/IllegalStateException;",
        "Ljava/io/IOException;" => "Ljava/lang/Exception;",
        _ => return None,
    })
}

/// One Dalvik register word. References are handles, never host addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Word {
    Bits(u32),
    Ref(usize),
}
pub(crate) const WEAK_REFERENT: &str = "droidless:reference:weak-referent";
impl Word {
    pub const ZERO: Self = Self::Bits(0);
    pub fn int(self) -> Result<i32> {
        match self {
            Self::Bits(v) => Ok(v as i32),
            _ => anyhow::bail!("reference used as primitive"),
        }
    }
    pub fn reference(self) -> Result<usize> {
        match self {
            Self::Ref(h) => Ok(h),
            Self::Bits(0) => Ok(0),
            _ => anyhow::bail!("primitive used as reference"),
        }
    }
    pub fn truth(self) -> bool {
        self != Self::ZERO
    }
}
impl From<i32> for Word {
    fn from(v: i32) -> Self {
        Self::Bits(v as u32)
    }
}
pub fn wide(v: u64) -> Vec<Word> {
    vec![Word::Bits(v as u32), Word::Bits((v >> 32) as u32)]
}
pub fn bits64(words: &[Word]) -> Result<u64> {
    ensure!(words.len() >= 2, "wide value needs two registers");
    Ok(u64::from(words[0].int()? as u32) | (u64::from(words[1].int()? as u32) << 32))
}
pub fn default_value(ty: &str) -> Vec<Word> {
    if ty == "J" || ty == "D" {
        vec![Word::ZERO; 2]
    } else {
        vec![Word::ZERO]
    }
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub object: Word,
    pub start: usize,
    pub end: usize,
    pub flags: i32,
}

#[derive(Clone, Debug)]
pub enum Data {
    Instance,
    String(String),
    Builder(String),
    Spanned {
        text: String,
        spans: Vec<TextSpan>,
    },
    File(String),
    Array {
        element: String,
        values: Vec<Vec<Word>>,
    },
    Bundle(BTreeMap<String, (String, Vec<Word>)>),
    Motion(crate::touch::Motion),
    Velocity(crate::touch::Velocity),
    Gesture(crate::touch::Gesture),
    Timer(crate::timers::Timer),
    Executor(crate::executors::Executor),
    Future(crate::executors::Future),
    Parcel {
        bytes: Vec<u8>,
        position: usize,
        depth: usize,
        read_limit: Option<usize>,
        recycled: bool,
    },
    TypedArray(Vec<Option<droidless_formats::xml::Value>>),
    Attributes {
        named: BTreeMap<String, droidless_formats::xml::Value>,
        resources: BTreeMap<u32, droidless_formats::xml::Value>,
    },
    XmlPull {
        events: Vec<droidless_formats::xml::PullEvent>,
        position: usize,
        closed: bool,
    },
    ReflectedMethod(droidless_formats::dex::Method),
    ReflectedField(droidless_formats::dex::Field),
    Collection {
        values: Vec<Word>,
        version: u32,
    },
    Menu(Vec<Word>),
    Map {
        entries: Vec<(Word, Word)>,
        version: u32,
        access_order: bool,
    },
    SparseArray(std::collections::BTreeMap<i32, Word>),
    ByteStream {
        bytes: Vec<u8>,
        position: usize,
        closed: bool,
    },
    ByteOutput {
        path: Vec<String>,
        bytes: Vec<u8>,
        position: usize,
        closed: bool,
    },
    Bitmap {
        bytes: Vec<u8>,
        width: u32,
        height: u32,
        mime: &'static str,
        recycled: bool,
    },
    Matrix([f32; 9]),
    Path(Vec<PathCommand>),
    AtomicInteger(std::sync::Arc<std::sync::atomic::AtomicI32>),
    AtomicLong(std::sync::Arc<std::sync::atomic::AtomicI64>),
    AtomicBoolean(std::sync::Arc<std::sync::atomic::AtomicBool>),
    Pattern(std::sync::Arc<regex::Regex>),
    Matcher {
        groups: Option<Vec<Option<(usize, usize)>>>,
        next_search: usize,
    },
    TimeUnit(TimeUnit),
    SqlDatabase(String),
    SqlStatement {
        database: String,
        sql: String,
        bindings: Vec<SqlValue>,
    },
    Cursor {
        columns: Vec<String>,
        rows: Vec<Vec<SqlValue>>,
        position: i32,
        closed: bool,
    },
    ContentValues(BTreeMap<String, SqlValue>),
    NetworkRequestBuilder(Vec<i32>),
    NetworkRequest(Vec<i32>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathCommand {
    Move([f32; 2]),
    Line([f32; 2]),
    Quad([f32; 4]),
    Cubic([f32; 6]),
    Close,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}
impl TimeUnit {
    pub(crate) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "NANOSECONDS" => Self::Nanoseconds,
            "MICROSECONDS" => Self::Microseconds,
            "MILLISECONDS" => Self::Milliseconds,
            "SECONDS" => Self::Seconds,
            "MINUTES" => Self::Minutes,
            "HOURS" => Self::Hours,
            "DAYS" => Self::Days,
            _ => return None,
        })
    }
    fn nanos(self) -> u64 {
        match self {
            Self::Nanoseconds => 1,
            Self::Microseconds => 1_000,
            Self::Milliseconds => 1_000_000,
            Self::Seconds => 1_000_000_000,
            Self::Minutes => 60_000_000_000,
            Self::Hours => 3_600_000_000_000,
            Self::Days => 86_400_000_000_000,
        }
    }
    pub(crate) fn convert(self, target: Self, value: i64) -> i64 {
        let (source, destination) = (self.nanos(), target.nanos());
        if source == destination {
            value
        } else if source > destination {
            value.saturating_mul((source / destination) as i64)
        } else {
            value / (destination / source) as i64
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Nanoseconds => "NANOSECONDS",
            Self::Microseconds => "MICROSECONDS",
            Self::Milliseconds => "MILLISECONDS",
            Self::Seconds => "SECONDS",
            Self::Minutes => "MINUTES",
            Self::Hours => "HOURS",
            Self::Days => "DAYS",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Object {
    pub class: String,
    pub fields: BTreeMap<String, Vec<Word>>,
    pub data: Data,
    pub view: Option<View>,
}
#[derive(Default)]
pub struct Heap {
    objects: Vec<Option<Object>>,
    pub allocations: u64,
    pub collections: u64,
    pub reclaimed: u64,
}
impl Heap {
    pub fn alloc(&mut self, object: Object) -> Result<Word> {
        ensure!(self.objects.len() < 1_000_000, "heap handle limit reached");
        self.objects.push(Some(object));
        self.allocations += 1;
        Ok(Word::Ref(self.objects.len()))
    }
    pub fn instance(&mut self, class: &str) -> Result<Word> {
        self.alloc(Object {
            class: class.to_owned(),
            fields: BTreeMap::new(),
            data: Data::Instance,
            view: View::for_class(class),
        })
    }
    pub fn string(&mut self, text: String) -> Result<Word> {
        ensure!(text.len() <= 1_048_576, "guest string exceeds 1 MiB");
        self.alloc(Object {
            class: "Ljava/lang/String;".into(),
            fields: BTreeMap::new(),
            data: Data::String(text),
            view: None,
        })
    }
    pub fn get(&self, word: Word) -> Result<&Object> {
        let h = word.reference()?;
        if h == 0 {
            return Err(fault("Ljava/lang/NullPointerException;", "null reference"));
        }
        self.objects
            .get(h.wrapping_sub(1))
            .and_then(Option::as_ref)
            .context("invalid guest object reference")
    }
    pub fn get_mut(&mut self, word: Word) -> Result<&mut Object> {
        let h = word.reference()?;
        if h == 0 {
            return Err(fault("Ljava/lang/NullPointerException;", "null reference"));
        }
        self.objects
            .get_mut(h.wrapping_sub(1))
            .and_then(Option::as_mut)
            .context("invalid guest object reference")
    }
    pub fn text(&self, word: Word) -> Result<&str> {
        match &self.get(word)?.data {
            Data::String(s) | Data::Builder(s) => Ok(s),
            Data::Spanned { text, .. } => Ok(text),
            _ => anyhow::bail!("expected string, got {}", self.get(word)?.class),
        }
    }
    pub fn live(&self) -> usize {
        self.objects.iter().filter(|o| o.is_some()).count()
    }
    pub fn collect(&mut self, roots: impl IntoIterator<Item = Word>) -> usize {
        let mut marked = BTreeSet::new();
        let mut work: Vec<Word> = roots.into_iter().collect();
        while let Some(word) = work.pop() {
            let Word::Ref(h) = word else {
                continue;
            };
            let Some(object) = h
                .checked_sub(1)
                .and_then(|i| self.objects.get(i))
                .and_then(Option::as_ref)
            else {
                continue;
            };
            if !marked.insert(h) {
                continue;
            }
            work.extend(
                object
                    .fields
                    .iter()
                    .filter(|(key, _)| key.as_str() != WEAK_REFERENT)
                    .flat_map(|(_, values)| values)
                    .copied(),
            );
            if let Data::Array { values, .. } = &object.data {
                work.extend(values.iter().flatten().copied());
            }
            if let Data::Bundle(values) = &object.data {
                work.extend(values.values().flat_map(|(_, words)| words).copied());
            }
            if let Data::Gesture(gesture) = &object.data {
                work.extend(gesture.roots());
            }
            if let Data::Timer(timer) = &object.data {
                work.extend(timer.tasks.values().copied());
                work.extend(timer.active);
            }
            if let Data::Executor(executor) = &object.data {
                work.extend(executor.roots());
            }
            if let Data::Future(future) = &object.data {
                work.extend(future.roots());
            }
            if let Data::Spanned { spans, .. } = &object.data {
                work.extend(spans.iter().map(|span| span.object));
            }
            if let Data::Collection { values, .. } = &object.data {
                work.extend(values.iter().copied());
            }
            if let Data::Menu(items) = &object.data {
                work.extend(items.iter().copied());
            }
            if let Data::Map { entries, .. } = &object.data {
                work.extend(entries.iter().flat_map(|(key, value)| [*key, *value]));
            }
            if let Data::SparseArray(values) = &object.data {
                work.extend(values.values().copied());
            }
            if let Some(view) = &object.view {
                work.extend(view.children.iter().copied());
                if let Some(listener) = view.listener {
                    work.push(listener);
                }
                if let Some(listener) = view.key_listener {
                    work.push(listener);
                }
            }
        }
        let mut reclaimed = 0;
        for (i, object) in self.objects.iter_mut().enumerate() {
            if let Some(object) = object
                && let Some(values) = object.fields.get_mut(WEAK_REFERENT)
            {
                for value in values {
                    if matches!(value, Word::Ref(handle) if !marked.contains(handle)) {
                        *value = Word::ZERO;
                    }
                }
            }
            if object.is_some() && !marked.contains(&(i + 1)) {
                *object = None;
                reclaimed += 1;
            }
        }
        // Handles are never reused: stale guest references cannot alias a newer object.
        self.collections += 1;
        self.reclaimed += reclaimed as u64;
        reclaimed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wide_and_gc() {
        assert_eq!(
            bits64(&wide(0xfedcba9876543210)).unwrap(),
            0xfedcba9876543210
        );
        let mut heap = Heap::default();
        let root = heap.instance("LRoot;").unwrap();
        let child = heap.string("kept".into()).unwrap();
        let dead = heap.string("discarded".into()).unwrap();
        heap.get_mut(root)
            .unwrap()
            .fields
            .insert("child".into(), vec![child]);
        assert_eq!(heap.collect([root]), 1);
        assert_eq!(heap.text(child).unwrap(), "kept");
        assert!(heap.get(dead).is_err());
    }
}
