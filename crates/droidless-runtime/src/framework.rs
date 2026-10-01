use crate::{
    heap::{Data, TimeUnit, Word, bits64, exception_parent, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{
    dex::{Field, Method},
    xml::{Element, Value},
};

const OBSERVERS: &str = "Landroid/database/Observable;->mObservers:Ljava/util/ArrayList;";

pub(crate) fn porter_duff_mode_ordinal(name: &str) -> Option<i32> {
    [
        "CLEAR", "SRC", "DST", "SRC_OVER", "DST_OVER", "SRC_IN", "DST_IN", "SRC_OUT", "DST_OUT",
        "SRC_ATOP", "DST_ATOP", "XOR", "DARKEN", "LIGHTEN", "MULTIPLY", "SCREEN", "ADD", "OVERLAY",
    ]
    .iter()
    .position(|mode| *mode == name)
    .map(|ordinal| ordinal as i32)
}

// Fixed virtual API branch profile, independent of the APK and host OS.
pub(crate) const SDK_INT: i32 = 21;

pub(crate) fn known_class(class: &str) -> bool {
    crate::ui::View::for_class(class).is_some()
        || exception_parent(class).is_some()
        || [
            "Ljava/lang/Object;",
            "Ljava/lang/Enum;",
            "Ljava/lang/StringBuilder;",
            "Ljava/lang/String;",
            "Ljava/lang/Class;",
            "Ljava/lang/ref/Reference;",
            "Ljava/lang/ref/WeakReference;",
            "Ljava/lang/reflect/Method;",
            "Ljava/lang/reflect/AccessibleObject;",
            "Ljava/lang/Boolean;",
            "Ljava/lang/Double;",
            "Ljava/lang/Integer;",
            "Ljava/lang/Long;",
            "Ljava/lang/Number;",
            "Ljava/lang/Math;",
            "Ljava/lang/Thread;",
            "Ljava/lang/ThreadLocal;",
            "Ljava/util/Date;",
            "Ljava/util/Locale;",
            "Ljava/util/ResourceBundle;",
            "Ljava/util/ListResourceBundle;",
            "Ljava/util/BitSet;",
            "Ljava/util/concurrent/Executors;",
            "Ljava/util/concurrent/Executor;",
            "Ljava/util/concurrent/ExecutorService;",
            "Ljava/util/concurrent/ThreadPoolExecutor;",
            "Ljava/util/concurrent/ConcurrentHashMap;",
            "Landroid/animation/Animator;",
            "Landroid/animation/ObjectAnimator;",
            "Landroid/animation/StateListAnimator;",
            "Ljava/lang/ThreadGroup;",
            "Landroid/view/animation/Animation;",
            "Landroid/view/animation/AnimationUtils;",
            "Landroid/view/animation/AccelerateDecelerateInterpolator;",
            "Landroid/view/animation/AccelerateInterpolator;",
            "Landroid/view/animation/DecelerateInterpolator;",
            "Landroid/view/animation/LinearInterpolator;",
            "Ljava/lang/System;",
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            "Ljava/util/concurrent/atomic/AtomicLong;",
            "Ljava/util/concurrent/atomic/AtomicBoolean;",
            "Ljava/util/regex/Pattern;",
            "Ljava/util/regex/Matcher;",
            "Ljava/util/concurrent/TimeUnit;",
            "Landroid/net/ConnectivityManager;",
            "Landroid/net/ConnectivityManager$NetworkCallback;",
            "Landroid/net/NetworkRequest;",
            "Landroid/net/NetworkRequest$Builder;",
            "Landroid/view/accessibility/AccessibilityManager;",
            "Ljava/io/InputStream;",
            "Ljava/io/Reader;",
            "Ljava/io/StringReader;",
            "Ljavax/xml/parsers/SAXParserFactory;",
            "Ljavax/xml/parsers/SAXParser;",
            "Ljavax/xml/parsers/ParserConfigurationException;",
            "Lorg/xml/sax/InputSource;",
            "Lorg/xml/sax/Attributes;",
            "Lorg/xml/sax/SAXException;",
            "Lorg/xml/sax/helpers/DefaultHandler;",
            "Ljava/io/File;",
            "Ljava/io/FileInputStream;",
            "Landroid/database/sqlite/SQLiteOpenHelper;",
            "Landroid/database/sqlite/SQLiteDatabase;",
            "Landroid/database/sqlite/SQLiteStatement;",
            "Landroid/database/Cursor;",
            "Landroid/database/Observable;",
            "Landroid/content/ContentValues;",
            "Landroid/os/Handler;",
            "Landroid/os/Message;",
            "Landroid/os/Looper;",
            "Landroid/os/SystemClock;",
            "Landroid/os/Process;",
            "Landroid/os/Trace;",
            "Landroid/view/View$MeasureSpec;",
            "Landroid/os/Build$VERSION;",
            "Landroid/net/LocalServerSocket;",
            "Landroid/net/LocalSocket;",
            "Ljava/util/HashSet;",
            "Ljava/util/Vector;",
            "Ljava/util/Stack;",
            "Ljava/util/TreeSet;",
            "Ljava/util/HashMap;",
            "Ljava/util/ArrayList;",
            "Ljava/util/LinkedHashMap;",
            "Ljava/util/WeakHashMap;",
            "Ljava/util/concurrent/LinkedBlockingQueue;",
            "Ljava/util/concurrent/CopyOnWriteArrayList;",
            "Ljava/lang/Throwable;",
            "Ljava/lang/Exception;",
            "Ljava/lang/RuntimeException;",
            "Landroid/app/Activity;",
            "Landroid/app/Application;",
            "Landroid/app/Application$ActivityLifecycleCallbacks;",
            "Landroid/content/res/Resources;",
            "Landroid/content/res/XmlResourceParser;",
            "Landroid/content/res/AssetManager;",
            "Landroid/content/res/Resources$Theme;",
            "Landroid/content/res/Configuration;",
            "Landroid/content/ContextWrapper;",
            "Landroid/content/res/TypedArray;",
            "Landroid/util/TypedValue;",
            "Landroid/util/Log;",
            "Landroid/content/res/ColorStateList;",
            "Landroid/graphics/drawable/Drawable;",
            "Landroid/graphics/drawable/Drawable$ConstantState;",
            "Landroid/graphics/drawable/Drawable$Callback;",
            "Landroid/graphics/drawable/ColorDrawable;",
            "Landroid/graphics/drawable/GradientDrawable;",
            "Landroid/graphics/drawable/LayerDrawable;",
            "Landroid/graphics/drawable/RippleDrawable;",
            "Landroid/graphics/drawable/InsetDrawable;",
            "Landroid/graphics/drawable/BitmapDrawable;",
            "Landroid/graphics/Bitmap;",
            "Landroid/graphics/BitmapFactory;",
            "Landroid/graphics/BitmapFactory$Options;",
            "Landroid/util/DisplayMetrics;",
            "Landroid/util/SparseArray;",
            "Landroid/util/SparseIntArray;",
            "Landroid/util/AttributeSet;",
            "Lorg/xmlpull/v1/XmlPullParser;",
            "Landroid/graphics/Rect;",
            "Landroid/graphics/RectF;",
            "Landroid/graphics/Paint;",
            "Landroid/graphics/PorterDuff$Mode;",
            "Landroid/text/TextUtils;",
            "Ljava/lang/CharSequence;",
            "Landroid/text/Spanned;",
            "Landroid/text/Spannable;",
            "Landroid/text/Editable;",
            "Landroid/text/SpannableStringBuilder;",
            "Landroid/text/SpannableString;",
            "Landroid/text/SpannedString;",
            "Landroid/view/WindowManager;",
            "Landroid/view/Display;",
            "Landroid/view/ViewConfiguration;",
            "Landroid/widget/OverScroller;",
            "Landroid/view/Window;",
            "Landroid/view/ViewTreeObserver;",
            "Landroid/view/ViewOutlineProvider;",
            "Landroid/text/TextUtils$TruncateAt;",
            "Landroid/view/WindowManager$LayoutParams;",
            "Landroid/view/ViewGroup$LayoutParams;",
            "Landroid/view/ViewGroup$MarginLayoutParams;",
            "Landroid/widget/LinearLayout$LayoutParams;",
            "Landroid/widget/FrameLayout$LayoutParams;",
            "Landroid/view/View$AccessibilityDelegate;",
            "Landroid/view/LayoutInflater;",
            "Landroid/view/LayoutInflater$Factory;",
            "Landroid/view/LayoutInflater$Factory2;",
            "Landroid/os/Bundle;",
            "Landroid/content/Intent;",
            "Landroid/content/ComponentName;",
            "Landroid/content/pm/PackageManager;",
            "Landroid/content/pm/ActivityInfo;",
            "Landroid/content/pm/ApplicationInfo;",
            "Landroid/content/pm/PackageInfo;",
            "Landroid/content/pm/ResolveInfo;",
            "Landroid/view/KeyEvent;",
        ]
        .contains(&class)
}
impl Runtime {
    fn manifest_theme_style(&self, class: &str) -> Option<u32> {
        let application = self
            .apk
            .manifest
            .document
            .children
            .iter()
            .find(|node| node.name == "application")?;
        let app_theme = application
            .attr("theme")
            .filter(|value| value.kind == 1)
            .map(|value| value.data);
        let package = &self.apk.manifest.package;
        application
            .children
            .iter()
            .filter(|node| node.name == "activity" || node.name == "activity-alias")
            .find(|node| {
                node.text("name").is_some_and(|name| {
                    let full_name = if name.starts_with('.') {
                        format!("{package}{name}")
                    } else if !name.contains('.') {
                        format!("{package}.{name}")
                    } else {
                        name
                    };
                    crate::vm::descriptor(&full_name) == class
                })
            })
            .and_then(|node| node.attr("theme"))
            .filter(|value| value.kind == 1)
            .map(|value| value.data)
            .or(app_theme)
    }

    fn theme_styles(&self, context: Word) -> Result<Vec<u32>> {
        if let Some(styles) = self.heap.get(context)?.fields.get("droidless:theme:styles") {
            return Ok(styles
                .iter()
                .filter_map(|style| style.int().ok())
                .map(|style| style as u32)
                .collect());
        }
        if let Some(theme) = self
            .heap
            .get(context)?
            .fields
            .get("droidless:theme")
            .and_then(|values| values.first())
            .copied()
            && let Some(styles) = self.heap.get(theme)?.fields.get("droidless:theme:styles")
        {
            return Ok(styles
                .iter()
                .filter_map(|style| style.int().ok())
                .map(|style| style as u32)
                .collect());
        }
        Ok(self
            .manifest_theme_style(&self.heap.get(context)?.class)
            .into_iter()
            .collect())
    }

    fn styled_attributes(&self, styles: &[u32]) -> Result<std::collections::BTreeMap<u32, Value>> {
        let mut attributes = std::collections::BTreeMap::new();
        for style in styles.iter().copied().filter(|style| *style != 0) {
            if style >> 24 == 1 && !self.apk.resources.entries.contains_key(&style) {
                continue;
            }
            attributes.extend(self.apk.resources.style(style)?);
        }
        Ok(attributes)
    }

    fn typed_array(
        &mut self,
        attrs: Word,
        attributes: &std::collections::BTreeMap<u32, Value>,
    ) -> Result<Word> {
        let values = match attrs {
            Word::Bits(0) => vec![],
            attrs => {
                let Data::Array { values, .. } = &self.heap.get(attrs)?.data else {
                    bail!("TypedArray attributes must be an int array");
                };
                values
                    .iter()
                    .map(|value| {
                        let id = value
                            .first()
                            .copied()
                            .context("empty attribute id")?
                            .int()? as u32;
                        Ok(attributes.get(&id).cloned())
                    })
                    .collect::<Result<Vec<_>>>()?
            }
        };
        let array = self.heap.instance("Landroid/content/res/TypedArray;")?;
        self.heap.get_mut(array)?.data = Data::TypedArray(values);
        Ok(array)
    }

    pub(crate) fn sdk_field(&self, field: &Field) -> bool {
        field.class == "Landroid/os/Build$VERSION;"
            && field.name == "SDK_INT"
            && self.class_location(&field.class).is_none()
    }
    pub(crate) fn collections_empty_list_field(&self, field: &Field) -> bool {
        field.class == "Ljava/util/Collections;"
            && field.name == "EMPTY_LIST"
            && field.ty == "Ljava/util/List;"
            && self.class_location(&field.class).is_none()
    }
    pub(crate) fn view_outline_provider_field(&self, field: &Field) -> bool {
        field.class == "Landroid/view/ViewOutlineProvider;"
            && field.name == "BOUNDS"
            && field.ty == "Landroid/view/ViewOutlineProvider;"
            && self.class_location(&field.class).is_none()
    }
    pub(crate) fn text_truncate_at_field(&self, field: &Field) -> bool {
        field.class == "Landroid/text/TextUtils$TruncateAt;"
            && field.ty == field.class
            && ["START", "MIDDLE", "END", "MARQUEE", "END_SMALL"].contains(&field.name.as_str())
            && self.class_location(&field.class).is_none()
    }
    pub(crate) fn text_truncate_at_object(&mut self, field: &Field) -> Result<Word> {
        let key = field.key();
        if let Some(value) = self.statics.get(&key).and_then(|values| values.first()) {
            return Ok(*value);
        }
        let value = self.heap.instance(&field.class)?;
        let name = self.intern(field.name.clone())?;
        let ordinal = ["START", "MIDDLE", "END", "MARQUEE", "END_SMALL"]
            .iter()
            .position(|name| *name == field.name)
            .context("unknown TextUtils.TruncateAt value")?;
        let fields = &mut self.heap.get_mut(value)?.fields;
        fields.insert("droidless:enum:name".into(), vec![name]);
        fields.insert(
            "droidless:enum:ordinal".into(),
            vec![Word::from(ordinal as i32)],
        );
        self.statics.insert(key, vec![value]);
        Ok(value)
    }
    pub(crate) fn view_outline_provider_object(&mut self, field: &Field) -> Result<Word> {
        let key = field.key();
        if let Some(value) = self.statics.get(&key).and_then(|values| values.first()) {
            return Ok(*value);
        }
        let value = self.heap.instance(&field.class)?;
        self.statics.insert(key, vec![value]);
        Ok(value)
    }
    pub(crate) fn porter_duff_mode(&mut self, name: &str) -> Result<Word> {
        let ordinal = porter_duff_mode_ordinal(name)
            .with_context(|| format!("unknown PorterDuff mode {name}"))?;
        let field = Field {
            class: "Landroid/graphics/PorterDuff$Mode;".into(),
            name: name.into(),
            ty: "Landroid/graphics/PorterDuff$Mode;".into(),
        };
        if let Some(value) = self
            .statics
            .get(&field.key())
            .and_then(|values| values.first())
        {
            return Ok(*value);
        }
        let mode = self.heap.instance(&field.class)?;
        let name = self.intern(name.into())?;
        let fields = &mut self.heap.get_mut(mode)?.fields;
        fields.insert("droidless:enum:name".into(), vec![name]);
        fields.insert("droidless:enum:ordinal".into(), vec![Word::from(ordinal)]);
        self.statics.insert(field.key(), vec![mode]);
        Ok(mode)
    }
    pub(crate) fn time_unit_field(&self, field: &Field) -> Option<TimeUnit> {
        (field.class == "Ljava/util/concurrent/TimeUnit;"
            && self.class_location(&field.class).is_none())
        .then(|| TimeUnit::named(&field.name))
        .flatten()
    }
    pub(crate) fn time_unit_object(&mut self, unit: TimeUnit) -> Result<Word> {
        let key = format!("droidless:time-unit:{}", unit.name());
        if let Some(value) = self.statics.get(&key).and_then(|value| value.first()) {
            return Ok(*value);
        }
        let object = self.heap.instance("Ljava/util/concurrent/TimeUnit;")?;
        self.heap.get_mut(object)?.data = Data::TimeUnit(unit);
        self.statics.insert(key, vec![object]);
        Ok(object)
    }
    pub(crate) fn native(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        if method.class.starts_with("Landroid/view/")
            || method.class.starts_with("Landroid/widget/")
            || method.class == "Landroid/app/Activity;"
        {
            self.require_main_thread()?;
        }
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        if let Some(result) = self.xml_resource_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.text_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.component_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.animation_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.enum_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.preference_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.file_io_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.sqlite_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.sax_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.string_format_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.atomic_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.regex_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.time_unit_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.thread_local_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.system_services_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.system_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.collection_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.queue_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.reflection_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.scheduling_native(method, args)? {
            return Ok(Some(result));
        }
        let signature = method.signature();
        let arg =
            |n| -> Result<Word> { args.get(n).copied().context("framework argument missing") };
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let mut result = vec![];
        if method.class == "Landroid/graphics/BitmapFactory;" {
            let options = match signature.as_str() {
                "decodeResource(Landroid/content/res/Resources;I)Landroid/graphics/Bitmap;" => None,
                "decodeResource(Landroid/content/res/Resources;ILandroid/graphics/BitmapFactory$Options;)Landroid/graphics/Bitmap;" => {
                    Some(arg(2)?)
                }
                "decodeStream(Ljava/io/InputStream;)Landroid/graphics/Bitmap;" => None,
                "decodeStream(Ljava/io/InputStream;Landroid/graphics/Rect;Landroid/graphics/BitmapFactory$Options;)Landroid/graphics/Bitmap;" => {
                    Some(arg(2)?)
                }
                "decodeByteArray([BII)Landroid/graphics/Bitmap;" => None,
                "decodeByteArray([BIILandroid/graphics/BitmapFactory$Options;)Landroid/graphics/Bitmap;" => {
                    Some(arg(3)?)
                }
                _ => bail!("unsupported BitmapFactory method {}", method.key()),
            };
            let decoded = match signature.as_str() {
                s if s.starts_with("decodeResource(") => {
                    let resources = arg(0)?;
                    ensure!(
                        resources != Word::ZERO,
                        fault("Ljava/lang/NullPointerException;", "resources is null")
                    );
                    ensure!(
                        self.is_a(
                            &self.heap.get(resources)?.class,
                            "Landroid/content/res/Resources;"
                        ),
                        "BitmapFactory requires Resources"
                    );
                    self.image_resource(arg(1)?.int()? as u32)?
                }
                s if s.starts_with("decodeStream(") => {
                    let stream = arg(0)?;
                    ensure!(
                        stream != Word::ZERO,
                        fault("Ljava/lang/NullPointerException;", "stream is null")
                    );
                    ensure!(
                        self.is_a(&self.heap.get(stream)?.class, "Ljava/io/InputStream;"),
                        "BitmapFactory requires an InputStream"
                    );
                    let data = &mut self.heap.get_mut(stream)?.data;
                    let Data::ByteStream {
                        bytes,
                        position,
                        closed,
                    } = data
                    else {
                        bail!("BitmapFactory requires a DROIDLESS input stream");
                    };
                    ensure!(!*closed, fault("Ljava/io/IOException;", "Stream closed"));
                    let remaining = bytes[*position..].to_vec();
                    *position = bytes.len();
                    Self::encoded_image(remaining)
                }
                _ => {
                    let array = arg(0)?;
                    ensure!(
                        array != Word::ZERO,
                        fault(
                            "Ljava/lang/NullPointerException;",
                            "image byte array is null"
                        )
                    );
                    let offset = arg(1)?.int()?;
                    let length = arg(2)?.int()?;
                    ensure!(
                        offset >= 0 && length >= 0,
                        fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "negative byte array offset or length"
                        )
                    );
                    let (offset, length, values) = match &self.heap.get(array)?.data {
                        Data::Array { element, values } if element == "B" => {
                            (offset as usize, length as usize, values.clone())
                        }
                        _ => bail!("BitmapFactory requires a byte array"),
                    };
                    ensure!(
                        offset <= values.len() && length <= values.len() - offset,
                        fault(
                            "Ljava/lang/IndexOutOfBoundsException;",
                            "byte array range is out of bounds"
                        )
                    );
                    let bytes = values[offset..offset + length]
                        .iter()
                        .map(|value| value[0].int().map(|byte| byte as i8 as u8))
                        .collect::<Result<Vec<_>>>()?;
                    Self::encoded_image(bytes)
                }
            };
            if let Some((info, bytes)) = decoded {
                result.push(self.bitmap(info, bytes, options)?);
            } else {
                if let Some(options) = options.filter(|value| *value != Word::ZERO) {
                    let fields = &mut self.heap.get_mut(options)?.fields;
                    fields.insert(
                        "Landroid/graphics/BitmapFactory$Options;->outWidth:I".into(),
                        vec![Word::from(-1)],
                    );
                    fields.insert(
                        "Landroid/graphics/BitmapFactory$Options;->outHeight:I".into(),
                        vec![Word::from(-1)],
                    );
                    fields.insert(
                        "Landroid/graphics/BitmapFactory$Options;->outMimeType:Ljava/lang/String;"
                            .into(),
                        vec![Word::ZERO],
                    );
                }
                result.push(Word::ZERO);
            }
            return Ok(Some(result));
        }
        if method.class == "Landroid/graphics/Bitmap;" {
            match signature.as_str() {
                "getWidth()I" | "getHeight()I" => {
                    let Data::Bitmap { width, height, .. } = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized Bitmap");
                    };
                    result.push(Word::from(if method.name == "getWidth" {
                        *width as i32
                    } else {
                        *height as i32
                    }));
                }
                "isRecycled()Z" => {
                    let Data::Bitmap { recycled, .. } = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized Bitmap");
                    };
                    result.push(Word::from(i32::from(*recycled)));
                }
                "recycle()V" => {
                    let Data::Bitmap {
                        bytes, recycled, ..
                    } = &mut self.heap.get_mut(receiver)?.data
                    else {
                        bail!("uninitialized Bitmap");
                    };
                    bytes.clear();
                    *recycled = true;
                }
                _ => bail!("unsupported Bitmap method {}", method.key()),
            }
            return Ok(Some(result));
        }
        if method.class == "Landroid/graphics/BitmapFactory$Options;" && signature == "<init>()V" {
            self.heap.get(receiver)?;
            self.heap.get_mut(receiver)?.fields.insert(
                "Landroid/graphics/BitmapFactory$Options;->inSampleSize:I".into(),
                vec![Word::from(1)],
            );
            return Ok(Some(result));
        }
        if receiver != Word::ZERO
            && matches!(receiver, Word::Ref(_))
            && self.is_a(&self.heap.get(receiver)?.class, "Landroid/view/View;")
            && signature == "onScrollChanged(IIII)V"
        {
            self.require_main_thread()?;
            self.view_mut(receiver)?;
            return Ok(Some(result));
        }
        if receiver != Word::ZERO
            && matches!(receiver, Word::Ref(_))
            && self.is_a(&self.heap.get(receiver)?.class, "Landroid/view/ViewGroup;")
        {
            match signature.as_str() {
                "detachViewFromParent(I)V" => {
                    self.require_main_thread()?;
                    let index = arg(1)?.int()?;
                    let children = &mut self.view_mut(receiver)?.children;
                    ensure!(
                        index >= 0 && (index as usize) < children.len(),
                        "View child index is out of bounds"
                    );
                    children.remove(index as usize);
                    return Ok(Some(result));
                }
                "attachViewToParent(Landroid/view/View;ILandroid/view/ViewGroup$LayoutParams;)V" => {
                    self.require_main_thread()?;
                    let child = arg(1)?;
                    let index = arg(2)?.int()?;
                    let params = arg(3)?;
                    self.view_mut(child)?;
                    if params != Word::ZERO {
                        self.heap.get(params)?;
                    }
                    let children = &mut self.view_mut(receiver)?.children;
                    ensure!(
                        index >= -1 && index <= children.len() as i32,
                        "View child index is out of bounds"
                    );
                    children.insert(
                        if index == -1 {
                            children.len()
                        } else {
                            index as usize
                        },
                        child,
                    );
                    let fields = &mut self.heap.get_mut(child)?.fields;
                    fields.insert("droidless:view:parent".into(), vec![receiver]);
                    fields.insert("droidless:view:layout-params".into(), vec![params]);
                    return Ok(Some(result));
                }
                _ => {}
            }
        }
        if receiver != Word::ZERO
            && matches!(receiver, Word::Ref(_))
            && self.is_a(
                &self.heap.get(receiver)?.class,
                "Landroid/database/Observable;",
            )
        {
            match signature.as_str() {
                "registerObserver(Ljava/lang/Object;)V" => {
                    let observer = arg(1)?;
                    if observer == Word::ZERO {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "observer must not be null",
                        ));
                    }
                    let observers = *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(OBSERVERS)
                        .and_then(|values| values.first())
                        .context("Observable has no observer list")?;
                    let contains = self.invoke(
                        Method {
                            class: "Ljava/util/ArrayList;".into(),
                            name: "contains".into(),
                            parameters: vec!["Ljava/lang/Object;".into()],
                            returns: "Z".into(),
                        },
                        vec![observers, observer],
                        true,
                    )?;
                    if contains
                        .first()
                        .context("contains returned no result")?
                        .truth()
                    {
                        return Err(fault(
                            "Ljava/lang/IllegalStateException;",
                            "observer is already registered",
                        ));
                    }
                    self.invoke(
                        Method {
                            class: "Ljava/util/ArrayList;".into(),
                            name: "add".into(),
                            parameters: vec!["Ljava/lang/Object;".into()],
                            returns: "Z".into(),
                        },
                        vec![observers, observer],
                        true,
                    )?;
                    return Ok(Some(result));
                }
                "unregisterObserver(Ljava/lang/Object;)V" => {
                    let observer = arg(1)?;
                    if observer == Word::ZERO {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "observer must not be null",
                        ));
                    }
                    let observers = *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(OBSERVERS)
                        .and_then(|values| values.first())
                        .context("Observable has no observer list")?;
                    let removed = self.invoke(
                        Method {
                            class: "Ljava/util/ArrayList;".into(),
                            name: "remove".into(),
                            parameters: vec!["Ljava/lang/Object;".into()],
                            returns: "Z".into(),
                        },
                        vec![observers, observer],
                        true,
                    )?;
                    if !removed
                        .first()
                        .context("remove returned no result")?
                        .truth()
                    {
                        return Err(fault(
                            "Ljava/lang/IllegalStateException;",
                            "observer was not registered",
                        ));
                    }
                    return Ok(Some(result));
                }
                "unregisterAll()V" => {
                    let observers = *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(OBSERVERS)
                        .and_then(|values| values.first())
                        .context("Observable has no observer list")?;
                    self.invoke(
                        Method {
                            class: "Ljava/util/ArrayList;".into(),
                            name: "clear".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![observers],
                        true,
                    )?;
                    return Ok(Some(result));
                }
                _ => {}
            }
        }
        match (method.class.as_str(), signature.as_str()) {
            ("Landroid/view/KeyEvent;", "getAction()I") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("action")
                    .context("uninitialized KeyEvent")?
                    .clone();
            }
            ("Landroid/view/KeyEvent;", "getKeyCode()I") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("keycode")
                    .context("uninitialized KeyEvent")?
                    .clone();
            }
            ("Landroid/view/accessibility/AccessibilityManager;", "isEnabled()Z") => {
                self.heap.get(receiver)?;
                result.push(Word::ZERO);
            }
            (class, "<init>()V") if exception_parent(class).is_some() => {
                self.heap.get(receiver)?;
            }
            ("Landroid/net/ConnectivityManager$NetworkCallback;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            (class, "<init>(Ljava/lang/String;)V") if exception_parent(class).is_some() => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("message".into(), vec![arg(1)?]);
            }
            ("Ljava/lang/Throwable;", "getMessage()Ljava/lang/String;") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("message")
                    .cloned()
                    .unwrap_or_else(|| vec![Word::ZERO]);
            }
            ("Ljava/lang/Throwable;", "getCause()Ljava/lang/Throwable;")
            | ("Ljava/lang/ExceptionInInitializerError;", "getException()Ljava/lang/Throwable;") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("cause")
                    .cloned()
                    .unwrap_or_else(|| vec![Word::ZERO]);
            }
            ("Ljava/lang/Throwable;", "toString()Ljava/lang/String;") => {
                let object = self.heap.get(receiver)?;
                let class = object
                    .class
                    .trim_start_matches('L')
                    .trim_end_matches(';')
                    .replace('/', ".");
                let message = object
                    .fields
                    .get("message")
                    .and_then(|v| v.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                let text = if message == Word::ZERO {
                    class
                } else {
                    format!("{class}: {}", self.heap.text(message)?)
                };
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;", "<init>(Ljava/lang/Object;)V")
            | ("Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;", "<init>(Ljava/lang/Object;Ljava/lang/ref/ReferenceQueue;)V") => {
                // ponytail: references stay strong until the guest heap models garbage collection.
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:reference:referent".into(),
                    vec![arg(1)?],
                );
            }
            ("Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;", "get()Ljava/lang/Object;") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:reference:referent")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;", "clear()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:reference:referent".into(), vec![Word::ZERO]);
            }
            ("Ljava/lang/String;", "valueOf(Ljava/lang/Object;)Ljava/lang/String;") => {
                if arg(0)? == Word::ZERO {
                    result.push(self.intern("null".into())?);
                } else {
                    result = self.invoke(
                        Method {
                            class: "Ljava/lang/Object;".into(),
                            name: "toString".into(),
                            parameters: vec![],
                            returns: "Ljava/lang/String;".into(),
                        },
                        vec![arg(0)?],
                        true,
                    )?;
                }
            }
            ("Ljava/lang/Object;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/database/Observable;", "<init>()V") => {
                self.heap.get(receiver)?;
                let observers = self.heap.instance("Ljava/util/ArrayList;")?;
                self.heap.get_mut(observers)?.data = Data::Collection {
                    values: vec![],
                    version: 0,
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(OBSERVERS.into(), vec![observers]);
            }
            ("Ljava/lang/Object;", "toString()Ljava/lang/String;") => {
                let o = self.heap.get(receiver)?;
                let s = format!("{}@{:x}", o.class, receiver.reference()?);
                result.push(self.heap.string(s)?);
            }
            ("Ljava/lang/Object;", "equals(Ljava/lang/Object;)Z") => {
                result.push(Word::from(i32::from(receiver == arg(1)?)))
            }
            ("Ljava/lang/Object;", "hashCode()I") => {
                result.push(Word::from(receiver.reference()? as i32));
            }
            ("Landroid/graphics/drawable/ColorDrawable;", "<init>()V") => {
                self.heap.get_mut(receiver)?.fields.insert(
                    "color".into(),
                    vec![Word::from(0x00000000_i32)],
                );
            }
            ("Landroid/graphics/drawable/ColorDrawable;", "<init>(I)V")
            | ("Landroid/graphics/drawable/ColorDrawable;", "setColor(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("color".into(), vec![arg(1)?]);
            }
            ("Landroid/graphics/drawable/ColorDrawable;", "getColor()I") => {
                result.push(
                    *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("color")
                        .and_then(|values| values.first())
                        .context("uninitialized ColorDrawable")?,
                );
            }
            ("Landroid/graphics/drawable/GradientDrawable;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/drawable/LayerDrawable;", "<init>([Landroid/graphics/drawable/Drawable;)V") => {
                let layers = arg(1)?;
                ensure!(matches!(self.heap.get(layers)?.data, Data::Array { .. }), "LayerDrawable layers must be an array");
                self.heap.get_mut(receiver)?.fields.insert("droidless:drawable:layers".into(), vec![layers]);
            }
            ("Landroid/graphics/drawable/RippleDrawable;", "<init>(Landroid/content/res/ColorStateList;Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;)V") => {
                for (field, value) in [
                    ("droidless:drawable:ripple-color", arg(1)?),
                    ("droidless:drawable:ripple-content", arg(2)?),
                    ("droidless:drawable:ripple-mask", arg(3)?),
                ] {
                    if value != Word::ZERO { self.heap.get(value)?; }
                    self.heap.get_mut(receiver)?.fields.insert(field.into(), vec![value]);
                }
            }
            ("Landroid/graphics/drawable/InsetDrawable;", "<init>(Landroid/graphics/drawable/Drawable;IIII)V") => {
                let drawable = arg(1)?;
                self.heap.get(drawable)?;
                self.heap.get_mut(receiver)?.fields.insert("droidless:drawable:inset-content".into(), vec![drawable]);
            }
            ("Landroid/animation/StateListAnimator;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Ljava/util/BitSet;", "<init>(I)V") => {
                let size = arg(1)?.int()?;
                if size < 0 { return Err(fault("Ljava/lang/NegativeArraySizeException;", size.to_string())); }
                ensure!(size <= 1_000_000, "BitSet size limit reached");
                self.heap.get_mut(receiver)?.fields.insert("droidless:bitset:values".into(), vec![Word::ZERO; size as usize]);
            }
            ("Ljava/util/BitSet;", "set(IZ)V") => {
                let index = arg(1)?.int()?;
                if index < 0 { return Err(fault("Ljava/lang/IndexOutOfBoundsException;", index.to_string())); }
                ensure!(index < 1_000_000, "BitSet size limit reached");
                let value = Word::from(i32::from(arg(2)?.truth()));
                let values = self.heap.get_mut(receiver)?.fields.entry("droidless:bitset:values".into()).or_default();
                values.resize(values.len().max(index as usize + 1), Word::ZERO);
                values[index as usize] = value;
            }
            ("Ljava/util/BitSet;", "set(IIZ)V") => {
                let from = arg(1)?.int()?;
                let to = arg(2)?.int()?;
                if from < 0 || to < from { return Err(fault("Ljava/lang/IndexOutOfBoundsException;", format!("{from}..{to}"))); }
                ensure!(to <= 1_000_000, "BitSet size limit reached");
                let value = Word::from(i32::from(arg(3)?.truth()));
                let values = self.heap.get_mut(receiver)?.fields.entry("droidless:bitset:values".into()).or_default();
                values.resize(values.len().max(to as usize), Word::ZERO);
                values[from as usize..to as usize].fill(value);
            }
            ("Ljava/util/BitSet;", "clear()V") => {
                if let Some(values) = self.heap.get_mut(receiver)?.fields.get_mut("droidless:bitset:values") { values.fill(Word::ZERO); }
            }
            ("Ljava/util/BitSet;", "clear(I)V") => {
                let index = arg(1)?.int()?;
                if index < 0 { return Err(fault("Ljava/lang/IndexOutOfBoundsException;", index.to_string())); }
                if let Some(value) = self.heap.get_mut(receiver)?.fields.get_mut("droidless:bitset:values").and_then(|values| values.get_mut(index as usize)) { *value = Word::ZERO; }
            }
            ("Ljava/util/BitSet;", "get(I)Z") => {
                let index = arg(1)?.int()?;
                if index < 0 { return Err(fault("Ljava/lang/IndexOutOfBoundsException;", index.to_string())); }
                let set = self.heap.get(receiver)?.fields.get("droidless:bitset:values").and_then(|values| values.get(index as usize)).is_some_and(|value| value.truth());
                result.push(Word::from(i32::from(set)));
            }
            ("Ljava/util/BitSet;", "isEmpty()Z") => {
                let empty = self.heap.get(receiver)?.fields.get("droidless:bitset:values").is_none_or(|values| values.iter().all(|value| !value.truth()));
                result.push(Word::from(i32::from(empty)));
            }
            ("Ljava/util/Date;", "<init>()V") => {
                let now = self.invoke(Method { class: "Ljava/lang/System;".into(), name: "currentTimeMillis".into(), parameters: vec![], returns: "J".into() }, vec![], false)?;
                self.heap.get_mut(receiver)?.fields.insert("droidless:date:time".into(), now);
            }
            ("Ljava/util/Date;", "<init>(J)V" | "setTime(J)V") => {
                let millis = args.get(1..3).context("Date timestamp missing")?.to_vec();
                self.heap.get_mut(receiver)?.fields.insert("droidless:date:time".into(), millis);
            }
            ("Ljava/util/Date;", "getTime()J") => {
                let time = self.heap.get(receiver)?.fields.get("droidless:date:time").cloned().unwrap_or_else(|| wide(0));
                result.extend(time);
            }
            ("Ljava/util/Date;", "before(Ljava/util/Date;)Z")
            | ("Ljava/util/Date;", "after(Ljava/util/Date;)Z")
            | ("Ljava/util/Date;", "compareTo(Ljava/util/Date;)I") => {
                let left = self.heap.get(receiver)?.fields.get("droidless:date:time").cloned().unwrap_or_else(|| wide(0));
                let right = self.heap.get(arg(1)?)?.fields.get("droidless:date:time").cloned().unwrap_or_else(|| wide(0));
                let comparison = crate::heap::bits64(&left)?.cmp(&crate::heap::bits64(&right)?);
                result.push(Word::from(if method.name == "before" { i32::from(comparison.is_lt()) } else if method.name == "after" { i32::from(comparison.is_gt()) } else { comparison as i32 }));
            }
            ("Ljava/util/concurrent/Executors;", "newCachedThreadPool()Ljava/util/concurrent/ExecutorService;") => {
                result.push(self.heap.instance("Ljava/util/concurrent/ThreadPoolExecutor;")?);
            }
            ("Ljava/util/concurrent/Executor;", "execute(Ljava/lang/Runnable;)V") => {
                // ponytail: cached-pool tasks run in caller order; add worker scheduling when an APK depends on concurrency.
                self.invoke(
                    Method { class: "Ljava/lang/Runnable;".into(), name: "run".into(), parameters: vec![], returns: "V".into() },
                    vec![arg(1)?],
                    true,
                )?;
            }
            ("Ljava/util/concurrent/ExecutorService;", "shutdown()V") => { self.heap.get(receiver)?; }
            ("Ljava/util/concurrent/ExecutorService;", "shutdownNow()Ljava/util/List;") => {
                result.push(self.heap.instance("Ljava/util/ArrayList;")?);
            }
            ("Ljava/util/concurrent/ExecutorService;", "isShutdown()Z" | "isTerminated()Z") => { result.push(Word::ZERO); }
            ("Ljava/util/concurrent/ExecutorService;", "awaitTermination(JLjava/util/concurrent/TimeUnit;)Z") => { result.push(Word::from(1)); }
            ("Landroid/animation/StateListAnimator;", "addState([ILandroid/animation/Animator;)V") => {
                let state = arg(1)?;
                let animator = arg(2)?;
                self.heap.get(state)?;
                self.heap.get(animator)?;
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.entry("droidless:animation:states".into()).or_default().extend([state, animator]);
            }
            ("Landroid/animation/ObjectAnimator;", "ofFloat(Ljava/lang/Object;Ljava/lang/String;[F)Landroid/animation/ObjectAnimator;") => {
                let target = arg(0)?;
                let property = arg(1)?;
                let values = arg(2)?;
                self.heap.get(target)?;
                self.heap.text(property)?;
                self.heap.get(values)?;
                let animator = self.heap.instance("Landroid/animation/ObjectAnimator;")?;
                let fields = &mut self.heap.get_mut(animator)?.fields;
                fields.insert("droidless:animation:target".into(), vec![target]);
                fields.insert("droidless:animation:property".into(), vec![property]);
                fields.insert("droidless:animation:values".into(), vec![values]);
                result.push(animator);
            }
            ("Landroid/animation/Animator;", "setInterpolator(Landroid/animation/TimeInterpolator;)V") => {
                let interpolator = arg(1)?;
                self.heap.get(interpolator)?;
                self.heap.get_mut(receiver)?.fields.insert("droidless:animation:interpolator".into(), vec![interpolator]);
            }
            ("Landroid/graphics/drawable/GradientDrawable;", "setShape(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:shape".into(), vec![arg(1)?]);
            }
            ("Landroid/graphics/drawable/GradientDrawable;", "setColor(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("color".into(), vec![arg(1)?]);
            }
            ("Landroid/graphics/drawable/GradientDrawable;", "setCornerRadius(F)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:corner-radius".into(), vec![arg(1)?]);
            }
            ("Landroid/graphics/drawable/Drawable;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/drawable/Drawable$ConstantState;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/drawable/Drawable;", "mutate()Landroid/graphics/drawable/Drawable;") => {
                self.heap.get(receiver)?;
                result.push(receiver);
            }
            ("Landroid/graphics/drawable/Drawable;", "setCallback(Landroid/graphics/drawable/Drawable$Callback;)V") => {
                let callback = arg(1)?;
                if callback != Word::ZERO {
                    self.heap.get(callback)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:callback".into(), vec![callback]);
            }
            ("Landroid/graphics/drawable/Drawable;", "isVisible()Z") => {
                let visible = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:visible")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::from(1));
                result.push(visible);
            }
            ("Landroid/graphics/drawable/Drawable;", "setVisible(ZZ)Z") => {
                let visible = arg(1)?;
                let previous = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:visible")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::from(1));
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:visible".into(), vec![visible]);
                result.push(Word::from(i32::from(previous != visible)));
            }
            ("Landroid/graphics/drawable/Drawable;", "getState()[I") => {
                let state = if let Some(state) = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:state")
                    .and_then(|values| values.first())
                    .copied()
                {
                    state
                } else {
                    let state = self.array("I".into(), 0)?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:drawable:state".into(), vec![state]);
                    state
                };
                result.push(state);
            }
            ("Landroid/graphics/drawable/Drawable;", "setState([I)Z") => {
                let state = arg(1)?;
                let next = match &self.heap.get(state)?.data {
                    Data::Array { element, values } if element == "I" => values,
                    _ => bail!("Drawable state must be an int array"),
                };
                let previous = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:state")
                    .and_then(|values| values.first())
                    .copied();
                let changed = if let Some(previous) = previous {
                    match &self.heap.get(previous)?.data {
                        Data::Array { element, values } if element == "I" => values != next,
                        _ => bail!("stored Drawable state is not an int array"),
                    }
                } else {
                    !next.is_empty()
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:state".into(), vec![state]);
                result.push(Word::from(i32::from(changed)));
            }
            ("Landroid/graphics/drawable/Drawable;", "getLevel()I") => {
                let level = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:level")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(level);
            }
            ("Landroid/graphics/drawable/Drawable;", "setLevel(I)Z") => {
                let level = arg(1)?;
                let previous = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:level")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:level".into(), vec![level]);
                result.push(Word::from(i32::from(previous != level)));
            }
            ("Landroid/graphics/drawable/Drawable;", "getBounds()Landroid/graphics/Rect;") => {
                let bounds = if let Some(bounds) = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:bounds")
                    .and_then(|values| values.first())
                    .copied()
                {
                    bounds
                } else {
                    let bounds = self.heap.instance("Landroid/graphics/Rect;")?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:drawable:bounds".into(), vec![bounds]);
                    bounds
                };
                result.push(bounds);
            }
            ("Landroid/graphics/drawable/Drawable;", "setBounds(Landroid/graphics/Rect;)V") => {
                let bounds = arg(1)?;
                self.heap.get(bounds)?;
                let copy = self.heap.get(bounds)?.fields.clone();
                let stored = if let Some(stored) = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:drawable:bounds")
                    .and_then(|values| values.first())
                    .copied()
                {
                    stored
                } else {
                    self.heap.instance("Landroid/graphics/Rect;")?
                };
                self.heap.get_mut(stored)?.fields = copy;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:bounds".into(), vec![stored]);
            }
            ("Landroid/graphics/drawable/Drawable;", "setBounds(IIII)V") => {
                let bounds = self.heap.instance("Landroid/graphics/Rect;")?;
                let fields = &mut self.heap.get_mut(bounds)?.fields;
                for (name, value) in ["left", "top", "right", "bottom"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(name.into(), vec![value]);
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:bounds".into(), vec![bounds]);
            }
            ("Landroid/graphics/drawable/Drawable;", "getConstantState()Landroid/graphics/drawable/Drawable$ConstantState;") => {
                result.push(Word::ZERO);
            }
            ("Landroid/graphics/drawable/Drawable;", "invalidateSelf()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/drawable/Drawable;", "clearColorFilter()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .remove("droidless:drawable:color-filter");
            }
            ("Landroid/graphics/drawable/Drawable;", "setColorFilter(ILandroid/graphics/PorterDuff$Mode;)V") => {
                self.heap.get(receiver)?;
                self.heap.get(arg(2)?)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:color-filter".into(), vec![arg(1)?, arg(2)?]);
            }
            ("Landroid/graphics/drawable/Drawable;", "setColorFilter(Landroid/graphics/ColorFilter;)V") => {
                let filter = arg(1)?;
                if filter != Word::ZERO {
                    self.heap.get(filter)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:color-filter".into(), vec![filter]);
            }
            ("Landroid/content/res/ColorStateList;", "getDefaultColor()I") => {
                let fields = &self.heap.get(receiver)?.fields;
                if let Some(color) = fields.get("droidless:color-state-list:default").and_then(|values| values.first()) {
                    result.push(*color);
                } else {
                    let resource = fields
                    .get("resourceId")
                    .and_then(|values| values.first())
                    .context("ColorStateList has no resource ID")?
                    .int()? as u32;
                    result.push(Word::from(self.apk.resources.resolve(resource)?.data as i32));
                }
            }
            ("Landroid/content/res/ColorStateList;", "getColorForState([II)I") => {
                let state = arg(1)?;
                self.heap.get(state)?;
                let fields = &self.heap.get(receiver)?.fields;
                let color = if let Some(color) = fields.get("droidless:color-state-list:default").and_then(|values| values.first()) {
                    *color
                } else {
                let resource = fields
                    .get("resourceId")
                    .and_then(|values| values.first())
                    .map(|value| value.int())
                    .transpose()?;
                if let Some(resource) = resource {
                    Word::from(self.apk.resources.resolve(resource as u32).map_or(arg(2)?.int()?, |value| value.data as i32))
                } else {
                    arg(2)?
                }
                };
                result.push(color);
            }
            ("Landroid/content/res/ColorStateList;", "isStateful()Z") => {
                result.push(Word::ZERO);
            }
            ("Landroid/graphics/Color;", "colorToHSV(I[F)V") => {
                let hsv = rgb_to_hsv(arg(0)?.int()? as u32);
                let array = arg(1)?;
                let Data::Array { values, element } = &mut self.heap.get_mut(array)?.data else {
                    bail!("Color.colorToHSV requires a float array");
                };
                ensure!(element == "F" && values.len() >= 3, "Color.colorToHSV requires a float array of length 3");
                for (value, component) in values.iter_mut().zip(hsv) {
                    *value = vec![Word::Bits(component.to_bits())];
                }
            }
            ("Landroid/content/res/ColorStateList;", "valueOf(I)Landroid/content/res/ColorStateList;") => {
                let color_list = self.heap.instance("Landroid/content/res/ColorStateList;")?;
                self.heap.get_mut(color_list)?.fields.insert("droidless:color-state-list:default".into(), vec![arg(0)?]);
                result.push(color_list);
            }
            ("Landroid/util/Log;", "isLoggable(Ljava/lang/String;I)Z") => {
                self.heap.text(arg(0)?)?;
                arg(1)?.int()?;
                result.push(Word::ZERO);
            }
            ("Landroid/os/Trace;", "beginSection(Ljava/lang/String;)V") => {
                self.heap.text(arg(0)?)?;
            }
            ("Landroid/os/Trace;", "endSection()V") => {}
            ("Landroid/os/Trace;", "isEnabled()Z") => result.push(Word::ZERO),
            ("Landroid/view/View$MeasureSpec;", "makeMeasureSpec(II)I") => {
                let size = arg(0)?.int()? as u32;
                let mode = arg(1)?.int()? as u32;
                result.push(Word::from(((size & 0x3fff_ffff) | (mode & 0xc000_0000)) as i32));
            }
            ("Landroid/view/View$MeasureSpec;", "getMode(I)I") => {
                result.push(Word::from(((arg(0)?.int()? as u32) & 0xc000_0000) as i32));
            }
            ("Landroid/view/View$MeasureSpec;", "getSize(I)I") => {
                result.push(Word::from(arg(0)?.int()? & 0x3fff_ffff));
            }
            ("Ljava/util/ResourceBundle;", "getBundle(Ljava/lang/String;Ljava/util/Locale;)Ljava/util/ResourceBundle;") => {
                let base = self.heap.text(arg(0)?)?.replace('.', "/");
                let locale = self.heap.get(arg(1)?)?;
                let language = locale
                    .fields
                    .get("droidless:locale:language")
                    .and_then(|values| values.first())
                    .copied()
                    .map(|value| self.heap.text(value))
                    .transpose()?
                    .unwrap_or("");
                let country = locale
                    .fields
                    .get("droidless:locale:country")
                    .and_then(|values| values.first())
                    .copied()
                    .map(|value| self.heap.text(value))
                    .transpose()?
                    .unwrap_or("");
                let candidates = [
                    (!language.is_empty() && !country.is_empty())
                        .then(|| format!("L{base}_{language}_{country};")),
                    (!language.is_empty()).then(|| format!("L{base}_{language};")),
                    Some(format!("L{base};")),
                ];
                let class = candidates
                    .into_iter()
                    .flatten()
                    .find(|candidate| self.class_location(candidate).is_some())
                    .context("ResourceBundle class not found")?;
                let bundle = self.new_instance(&class)?;
                self.invoke(
                    Method {
                        class,
                        name: "<init>".into(),
                        parameters: vec![],
                        returns: "V".into(),
                    },
                    vec![bundle],
                    false,
                )?;
                result.push(bundle);
            }
            ("Ljava/util/ResourceBundle;", "getString(Ljava/lang/String;)Ljava/lang/String;") => {
                let key = arg(1)?;
                let Some(value) = resource_bundle_object(self, receiver, key)? else {
                    return Err(fault(
                        "Ljava/util/MissingResourceException;",
                        format!("resource bundle key {:?} was not found", self.heap.text(key)?),
                    ));
                };
                self.heap.text(value).map_err(|_| {
                    fault(
                        "Ljava/lang/ClassCastException;",
                        "resource bundle value is not a String",
                    )
                })?;
                result.push(value);
            }
            ("Ljava/util/ResourceBundle;", "containsKey(Ljava/lang/String;)Z") => {
                result.push(Word::from(i32::from(
                    resource_bundle_object(self, receiver, arg(1)?)?.is_some(),
                )));
            }
            ("Ljava/util/ListResourceBundle;", "<init>()V")
            | ("Ljava/util/ResourceBundle;", "<init>()V") => {}
            ("Ljava/util/Locale;", "getDefault()Ljava/util/Locale;") => {
                let key = "droidless:locale:default";
                let locale = if let Some(locale) = self.statics.get(key).and_then(|v| v.first()) {
                    *locale
                } else {
                    let locale = self.heap.instance("Ljava/util/Locale;")?;
                    for (field, value) in [("language", "en"), ("country", "US"), ("variant", "")] {
                        let value = self.heap.string(value.into())?;
                        self.heap.get_mut(locale)?.fields.insert(
                            format!("droidless:locale:{field}"),
                            vec![value],
                        );
                    }
                    self.statics.insert(key.into(), vec![locale]);
                    locale
                };
                result.push(locale);
            }
            ("Ljava/util/Locale;", "getLanguage()Ljava/lang/String;")
            | ("Ljava/util/Locale;", "getCountry()Ljava/lang/String;")
            | ("Ljava/util/Locale;", "getVariant()Ljava/lang/String;") => {
                let field = match method.name.as_str() {
                    "getLanguage" => "language",
                    "getCountry" => "country",
                    _ => "variant",
                };
                result.push(
                    *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(&format!("droidless:locale:{field}"))
                        .and_then(|v| v.first())
                        .context("uninitialized Locale")?,
                );
            }
            ("Ljava/util/Locale;", "toString()Ljava/lang/String;") => {
                let language = self.heap.get(receiver)?.fields.get("droidless:locale:language").and_then(|v| v.first()).copied().context("uninitialized Locale")?;
                let country = self.heap.get(receiver)?.fields.get("droidless:locale:country").and_then(|v| v.first()).copied().context("uninitialized Locale")?;
                let language = self.heap.text(language)?;
                let country = self.heap.text(country)?;
                result.push(self.heap.string(if country.is_empty() { language.into() } else { format!("{language}_{country}") })?);
            }
            ("Ljava/lang/String;", "toString()Ljava/lang/String;") => {
                self.heap.text(receiver)?;
                result.push(receiver);
            }
            ("Ljava/lang/String;", "length()I") => result.push(Word::from(
                self.heap.text(receiver)?.encode_utf16().count() as i32,
            )),
            ("Ljava/lang/String;", "isEmpty()Z") => {
                result.push(Word::from(i32::from(self.heap.text(receiver)?.is_empty())))
            }
            ("Ljava/lang/String;", "trim()Ljava/lang/String;") => {
                let value = self.heap.text(receiver)?.trim_matches(|character| character <= '\u{20}').to_owned();
                result.push(self.heap.string(value)?);
            }
            ("Ljava/lang/String;", "equals(Ljava/lang/Object;)Z") => {
                let rhs = arg(1)?;
                result.push(Word::from(i32::from(
                    rhs != Word::ZERO
                        && self
                            .heap
                            .text(rhs)
                            .is_ok_and(|s| s == self.heap.text(receiver).unwrap_or("")),
                )));
            }
            ("Ljava/lang/String;", "compareTo(Ljava/lang/Object;)I")
            | ("Ljava/lang/String;", "compareTo(Ljava/lang/String;)I") => {
                let left = self.heap.text(receiver)?.encode_utf16().collect::<Vec<_>>();
                let right = self.heap.text(arg(1)?)?.encode_utf16().collect::<Vec<_>>();
                let comparison = left
                    .iter()
                    .zip(&right)
                    .find_map(|(left, right)| {
                        (left != right).then_some(i32::from(*left) - i32::from(*right))
                    })
                    .unwrap_or_else(|| left.len() as i32 - right.len() as i32);
                result.push(Word::from(comparison));
            }
            ("Ljava/lang/String;", "startsWith(Ljava/lang/String;)Z") => {
                result.push(Word::from(i32::from(
                    self.heap
                        .text(receiver)?
                        .starts_with(self.heap.text(arg(1)?)?),
                )))
            }
            ("Ljava/lang/String;", "endsWith(Ljava/lang/String;)Z") => {
                result.push(Word::from(i32::from(
                    self.heap.text(receiver)?.ends_with(self.heap.text(arg(1)?)?),
                )))
            }
            ("Ljava/lang/String;", "contains(Ljava/lang/CharSequence;)Z") => {
                result.push(Word::from(i32::from(
                    self.heap.text(receiver)?.contains(self.heap.text(arg(1)?)?),
                )))
            }
            ("Ljava/lang/String;", "replace(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;") => {
                let source = self.heap.text(receiver)?.to_owned();
                let target = self.heap.text(arg(1)?)?.to_owned();
                let replacement = self.heap.text(arg(2)?)?.to_owned();
                result.push(self.heap.string(source.replace(&target, &replacement))?);
            }
            ("Ljava/lang/String;", "substring(I)Ljava/lang/String;")
            | ("Ljava/lang/String;", "substring(II)Ljava/lang/String;") => {
                let units = self.heap.text(receiver)?.encode_utf16().collect::<Vec<_>>();
                let start = usize::try_from(arg(1)?.int()?).map_err(|_| {
                    fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "negative substring start",
                    )
                })?;
                let end = if args.len() == 3 {
                    usize::try_from(arg(2)?.int()?).map_err(|_| {
                        fault(
                            "Ljava/lang/StringIndexOutOfBoundsException;",
                            "negative substring end",
                        )
                    })?
                } else {
                    units.len()
                };
                let text = String::from_utf16(units.get(start..end).ok_or_else(|| {
                    fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "substring indexes out of bounds",
                    )
                })?)?;
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/String;", "concat(Ljava/lang/String;)Ljava/lang/String;") => {
                let text = format!("{}{}", self.heap.text(receiver)?, self.heap.text(arg(1)?)?);
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/String;", "charAt(I)C") => {
                let index = usize::try_from(arg(1)?.int()?).map_err(|_| {
                    fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "negative string index",
                    )
                })?;
                let c = self
                    .heap
                    .text(receiver)?
                    .encode_utf16()
                    .nth(index)
                    .ok_or_else(|| {
                        fault(
                            "Ljava/lang/StringIndexOutOfBoundsException;",
                            "charAt index out of bounds",
                        )
                    })?;
                result.push(Word::Bits(u32::from(c)));
            }
            ("Ljava/lang/String;", "toCharArray()[C") => {
                let values = self.heap.text(receiver)?.encode_utf16().collect::<Vec<_>>();
                let array = self.array("C".into(), values.len())?;
                if let Data::Array { values: target, .. } = &mut self.heap.get_mut(array)?.data {
                    for (slot, value) in target.iter_mut().zip(values) {
                        slot[0] = Word::Bits(u32::from(value));
                    }
                }
                result.push(array);
            }
            ("Ljava/lang/String;", "valueOf(I)Ljava/lang/String;")
            | ("Ljava/lang/Integer;", "toString(I)Ljava/lang/String;") => {
                let s = arg(0)?.int()?.to_string();
                result.push(self.heap.string(s)?);
            }
            ("Ljava/lang/String;", "valueOf(J)Ljava/lang/String;")
            | ("Ljava/lang/Long;", "toString(J)Ljava/lang/String;") => {
                result.push(self.heap.string((bits64(args)? as i64).to_string())?);
            }
            ("Ljava/lang/Integer;", "valueOf(I)Ljava/lang/Integer;") => {
                let value = arg(0)?;
                let object = self.heap.instance("Ljava/lang/Integer;")?;
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("value".into(), vec![value]);
                result.push(object);
            }
            ("Ljava/lang/Boolean;", "valueOf(Z)Ljava/lang/Boolean;") => {
                let object = self.heap.instance("Ljava/lang/Boolean;")?;
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("value".into(), vec![arg(0)?]);
                result.push(object);
            }
            ("Ljava/lang/Boolean;", "<init>(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("value".into(), vec![arg(1)?]);
            }
            ("Ljava/lang/Boolean;", "booleanValue()Z") => result.push(
                *self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .context("uninitialized Boolean")?,
            ),
            ("Ljava/lang/Boolean;", "equals(Ljava/lang/Object;)Z") => {
                let other = arg(1)?;
                let equal = other != Word::ZERO
                    && self.heap.get(other)?.class == "Ljava/lang/Boolean;"
                    && self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("value")
                        .and_then(|values| values.first())
                        == self
                            .heap
                            .get(other)?
                            .fields
                            .get("value")
                            .and_then(|values| values.first());
                result.push(Word::from(i32::from(equal)));
            }
            ("Ljava/lang/Boolean;", "hashCode()I") => {
                let value = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .context("uninitialized Boolean")?
                    .truth();
                result.push(Word::from(if value { 1231 } else { 1237 }));
            }
            ("Ljava/lang/Boolean;", "toString()Ljava/lang/String;") => {
                let value = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .context("uninitialized Boolean")?
                    .truth();
                result.push(self.heap.string(value.to_string())?);
            }
            ("Ljava/lang/Integer;", "intValue()I") => {
                result.push(
                    *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("value")
                        .and_then(|values| values.first())
                        .context("uninitialized Integer")?,
                );
            }
            ("Ljava/lang/Number;", "intValue()I") => {
                let object = self.heap.get(receiver)?;
                let value = object
                    .fields
                    .get("value")
                    .context("uninitialized Number")?;
                let value = match object.class.as_str() {
                    "Ljava/lang/Byte;"
                    | "Ljava/lang/Character;"
                    | "Ljava/lang/Short;"
                    | "Ljava/lang/Integer;" => value[0].int()?,
                    "Ljava/lang/Long;" => bits64(value)? as i64 as i32,
                    "Ljava/lang/Float;" => {
                        f32::from_bits(value[0].int()? as u32) as i32
                    }
                    "Ljava/lang/Double;" => f64::from_bits(bits64(value)?) as i32,
                    class => bail!("unsupported Number subclass {class}"),
                };
                result.push(Word::from(value));
            }
            ("Ljava/lang/Integer;", "equals(Ljava/lang/Object;)Z") => {
                let other = arg(1)?;
                let equal = other != Word::ZERO
                    && self.heap.get(other)?.class == "Ljava/lang/Integer;"
                    && self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("value")
                        .and_then(|values| values.first())
                        == self
                            .heap
                            .get(other)?
                            .fields
                            .get("value")
                            .and_then(|values| values.first());
                result.push(Word::from(i32::from(equal)));
            }
            ("Ljava/lang/Integer;", "hashCode()I") => result.push(
                *self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .context("uninitialized Integer")?,
            ),
            ("Ljava/lang/Integer;", "toString()Ljava/lang/String;") => {
                let value = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .and_then(|values| values.first())
                    .context("uninitialized Integer")?
                    .int()?;
                result.push(self.heap.string(value.to_string())?);
            }
            ("Ljava/lang/Long;", "valueOf(J)Ljava/lang/Long;") => {
                let value = wide(bits64(args)?);
                let object = self.heap.instance("Ljava/lang/Long;")?;
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("value".into(), value);
                result.push(object);
            }
            ("Ljava/lang/Long;", "<init>(J)V") => {
                let value = wide(bits64(&args[1..])?);
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("value".into(), value);
            }
            ("Ljava/lang/Long;", "longValue()J") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .context("uninitialized Long")?
                    .clone();
            }
            ("Ljava/lang/Long;", "equals(Ljava/lang/Object;)Z") => {
                let other = arg(1)?;
                let equal = other != Word::ZERO
                    && self.heap.get(other)?.class == "Ljava/lang/Long;"
                    && self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("value")
                        .context("uninitialized Long")?
                        == self
                            .heap
                            .get(other)?
                            .fields
                            .get("value")
                            .context("uninitialized Long")?;
                result.push(Word::from(i32::from(equal)));
            }
            ("Ljava/lang/Long;", "hashCode()I") => {
                let value = bits64(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("value")
                        .context("uninitialized Long")?,
                )?;
                result.push(Word::from((value ^ (value >> 32)) as i32));
            }
            ("Ljava/lang/Long;", "toString()Ljava/lang/String;") => {
                let value = bits64(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("value")
                        .context("uninitialized Long")?,
                )? as i64;
                result.push(self.heap.string(value.to_string())?);
            }
            ("Ljava/lang/Double;", "toString(D)Ljava/lang/String;")
            | ("Ljava/lang/String;", "valueOf(D)Ljava/lang/String;") => result.push(
                self.heap
                    .string(java_double(f64::from_bits(bits64(args)?)))?,
            ),
            ("Ljava/lang/Double;", "valueOf(D)Ljava/lang/Double;") => {
                let value = wide(bits64(args)?);
                let object = self.heap.instance("Ljava/lang/Double;")?;
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("value".into(), value);
                result.push(object);
            }
            ("Ljava/lang/Double;", "doubleValue()D") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .context("uninitialized Double")?
                    .clone();
            }
            ("Ljava/lang/Double;", "toString()Ljava/lang/String;") => {
                let words = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .context("uninitialized Double")?;
                result.push(
                    self.heap
                        .string(java_double(f64::from_bits(bits64(words)?)))?,
                );
            }
            ("Ljava/lang/Double;", "isNaN(D)Z") => {
                result.push(Word::from(i32::from(
                    f64::from_bits(bits64(args)?).is_nan(),
                )));
            }
            ("Ljava/lang/Double;", "equals(Ljava/lang/Object;)Z")
            | ("Ljava/lang/Double;", "hashCode()I") => {
                bail!("unsupported Double method {}", method.key());
            }
            ("Ljava/lang/Double;", "parseDouble(Ljava/lang/String;)D") => {
                let value = self
                    .heap
                    .text(arg(0)?)?
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| {
                        fault(
                            "Ljava/lang/NumberFormatException;",
                            "invalid floating-point string",
                        )
                    })?;
                result = wide(value.to_bits());
            }
            ("Ljava/lang/Integer;", "parseInt(Ljava/lang/String;)I") => {
                let input = arg(0)?;
                if input == Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/NumberFormatException;",
                        "null integer string",
                    ));
                }
                let value = self.heap.text(input)?.parse::<i32>().map_err(|_| {
                    fault(
                        "Ljava/lang/NumberFormatException;",
                        "invalid integer string",
                    )
                })?;
                result.push(Word::from(value));
            }
            ("Ljava/lang/StringBuilder;", "<init>()V") => {
                self.heap.get_mut(receiver)?.data = Data::Builder(String::new())
            }
            ("Ljava/lang/StringBuilder;", "<init>(I)V") => {
                let capacity = arg(1)?.int()?;
                if capacity < 0 { return Err(fault("Ljava/lang/NegativeArraySizeException;", capacity.to_string())); }
                ensure!(capacity <= 1_048_576, "StringBuilder capacity limit reached");
                self.heap.get_mut(receiver)?.data = Data::Builder(String::new())
            }
            ("Ljava/lang/StringBuilder;", "<init>(Ljava/lang/String;)V") => {
                let s = self.heap.text(arg(1)?)?.to_owned();
                self.heap.get_mut(receiver)?.data = Data::Builder(s);
            }
            (
                "Ljava/lang/StringBuilder;",
                "append(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            )
            | (
                "Ljava/lang/StringBuilder;",
                "append(Ljava/lang/CharSequence;)Ljava/lang/StringBuilder;",
            )
            | ("Ljava/lang/StringBuilder;", "append(I)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(C)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(J)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(D)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(Ljava/lang/Object;)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(Z)Ljava/lang/StringBuilder;") => {
                let text = match method.parameters[0].as_str() {
                    "I" => arg(1)?.int()?.to_string(),
                    "C" => String::from_utf16_lossy(&[arg(1)?.int()? as u16]),
                    "J" => (bits64(&args[1..])? as i64).to_string(),
                    "Z" => (arg(1)?.int()? != 0).to_string(),
                    "D" => java_double(f64::from_bits(bits64(&args[1..])?)),
                    "Ljava/lang/Object;" => {
                        let string = self.invoke(
                            Method {
                                class: "Ljava/lang/String;".into(),
                                name: "valueOf".into(),
                                parameters: vec!["Ljava/lang/Object;".into()],
                                returns: "Ljava/lang/String;".into(),
                            },
                            vec![arg(1)?],
                            false,
                        )?;
                        self.heap.text(*string.first().context("String.valueOf returned no result")?)?.to_owned()
                    }
                    _ => {
                        if arg(1)? == Word::ZERO {
                            "null".into()
                        } else {
                            self.heap.text(arg(1)?)?.to_owned()
                        }
                    }
                };
                let Data::Builder(s) = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("StringBuilder not initialized");
                };
                ensure!(
                    s.len() + text.len() <= 1_048_576,
                    "StringBuilder limit reached"
                );
                s.push_str(&text);
                result.push(receiver);
            }
            ("Ljava/lang/StringBuilder;", "toString()Ljava/lang/String;") => {
                let text = self.heap.text(receiver)?.to_owned();
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/StringBuilder;", "setLength(I)V") => {
                let length = arg(1)?.int()?;
                if length < 0 {
                    return Err(fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "negative StringBuilder length",
                    ));
                }
                ensure!(length <= 1_048_576, "StringBuilder limit reached");
                let mut units = self.heap.text(receiver)?.encode_utf16().collect::<Vec<_>>();
                units.resize(length as usize, 0);
                let Data::Builder(text) = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("StringBuilder not initialized");
                };
                *text = String::from_utf16_lossy(&units);
            }
            ("Ljava/lang/Math;", sig)
                if [
                    "sqrt(D)D", "cbrt(D)D", "sin(D)D", "cos(D)D", "tan(D)D", "log(D)D", "exp(D)D",
                    "abs(D)D", "pow(DD)D",
                ]
                .contains(&sig) =>
            {
                let x = f64::from_bits(bits64(args)?);
                let value = match method.name.as_str() {
                    "sqrt" => x.sqrt(),
                    "cbrt" => x.cbrt(),
                    "sin" => x.sin(),
                    "cos" => x.cos(),
                    "tan" => x.tan(),
                    "log" => x.ln(),
                    "exp" => x.exp(),
                    "abs" => x.abs(),
                    _ => x.powf(f64::from_bits(bits64(&args[2..])?)),
                };
                result = wide(value.to_bits());
            }
            ("Ljava/lang/Math;", "abs(J)J") => {
                result = wide((bits64(args)? as i64).wrapping_abs() as u64);
            }
            ("Ljava/lang/Math;", "abs(I)I") => {
                result.push(Word::from(arg(0)?.int()?.wrapping_abs()));
            }
            ("Ljava/lang/Math;", "max(II)I") => {
                result.push(Word::from(arg(0)?.int()?.max(arg(1)?.int()?)));
            }
            ("Ljava/lang/Math;", "min(II)I") => {
                result.push(Word::from(arg(0)?.int()?.min(arg(1)?.int()?)));
            }
            ("Ljava/lang/Long;", "bitCount(J)I") => {
                result.push(Word::from(bits64(args)?.count_ones() as i32));
            }
            ("Ljava/lang/Long;", "rotateRight(JI)J") => {
                result = wide(bits64(args)?.rotate_right(arg(2)?.int()? as u32));
            }
            ("Ljava/lang/Integer;", "bitCount(I)I") => {
                result.push(Word::from(arg(0)?.int()?.count_ones() as i32));
            }
            ("Landroid/app/Activity;", "<init>()V")
            | ("Landroid/app/Application;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/app/Activity;", "onCreate(Landroid/os/Bundle;)V")
            | ("Landroid/app/Activity;", "onStart()V")
            | ("Landroid/app/Activity;", "onResume()V")
            | ("Landroid/app/Activity;", "onPause()V")
            | ("Landroid/app/Activity;", "onStop()V")
            | ("Landroid/app/Activity;", "onDestroy()V") => {
                self.heap.get(receiver)?;
                let callback = match method.name.as_str() {
                    "onCreate" => "onActivityCreated",
                    "onStart" => "onActivityStarted",
                    "onResume" => "onActivityResumed",
                    "onPause" => "onActivityPaused",
                    "onStop" => "onActivityStopped",
                    _ => "onActivityDestroyed",
                };
                self.dispatch_activity_callback(callback, args)?;
            }
            ("Landroid/app/Activity;", "onRestart()V")
            | ("Landroid/app/Application;", "onCreate()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/app/Activity;", "getLastNonConfigurationInstance()Ljava/lang/Object;") => {
                result.push(Word::ZERO);
            }
            ("Landroid/app/Activity;", "setContentView(Landroid/view/View;)V") => {
                let view = arg(1)?;
                ensure!(
                    self.heap.get(view)?.view.is_some(),
                    "setContentView expects View"
                );
                self.set_content(receiver, view)?;
            }
            ("Landroid/app/Activity;", "setContentView(I)V") => {
                let root = self.inflate_id(arg(1)?.int()? as u32, 0, receiver)?;
                self.set_content(receiver, root)?;
            }
            ("Landroid/app/Activity;", "findViewById(I)Landroid/view/View;")
            | ("Landroid/view/View;", "findViewById(I)Landroid/view/View;") => {
                let root = if method.class == "Landroid/app/Activity;" {
                    self.screen(receiver)?
                        .root
                        .context("findViewById before setContentView")?
                } else {
                    receiver
                };
                result.push(
                    self.find_view(root, arg(1)?.int()? as u32, 0)?
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/app/Activity;", "setTitle(Ljava/lang/CharSequence;)V") => {
                self.set_title(receiver, self.heap.text(arg(1)?)?.to_owned())?;
            }
            ("Landroid/app/Activity;", "getTitle()Ljava/lang/CharSequence;") => {
                result.push(self.heap.string(self.screen(receiver)?.title.clone())?);
            }
            ("Landroid/util/AttributeSet;", "getAttributeCount()I") => {
                let count = match &self.heap.get(receiver)?.data {
                    Data::Attributes(attributes) => attributes.len() as i32,
                    Data::XmlPull { events, position, .. } => {
                        let event = events.get(*position).context("invalid XML parser position")?;
                        if event.kind == 2 {
                            event.attributes.len() as i32
                        } else {
                            -1
                        }
                    }
                    _ => bail!("uninitialized AttributeSet"),
                };
                result.push(Word::from(count));
            }
            ("Landroid/util/AttributeSet;", "getAttributeValue(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;") => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                result.push(if let Some(value) = value {
                    self.heap.string(value.display())?
                } else {
                    Word::ZERO
                });
            }
            ("Landroid/util/AttributeSet;", "getAttributeResourceValue(Ljava/lang/String;Ljava/lang/String;I)I") => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                result.push(Word::from(value.filter(|value| value.kind == 1).map_or(arg(3)?.int()?, |value| value.data as i32)));
            }
            ("Landroid/util/AttributeSet;", "getAttributeIntValue(Ljava/lang/String;Ljava/lang/String;I)I") => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                result.push(Word::from(value.and_then(|value| {
                    (value.kind == 0x10 || value.kind == 0x11).then_some(value.data as i32)
                        .or_else(|| value.text.and_then(|text| text.parse().ok()))
                }).unwrap_or(arg(3)?.int()?)));
            }
            ("Landroid/util/AttributeSet;", "getAttributeBooleanValue(Ljava/lang/String;Ljava/lang/String;Z)Z") => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                let enabled = value.and_then(|value| {
                    (value.kind == 0x12).then_some(value.data != 0)
                        .or_else(|| value.text.and_then(|text| text.parse().ok()))
                }).map_or(arg(3)?.int()? != 0, |value| value);
                result.push(Word::from(i32::from(enabled)));
            }
            ("Landroid/content/Context;", "getString(I)Ljava/lang/String;")
            | ("Landroid/content/res/Resources;", "getString(I)Ljava/lang/String;") => {
                let text = self.resource_text(arg(1)?.int()? as u32)?;
                result.push(self.heap.string(text)?);
            }
            ("Landroid/content/Context;", "getResources()Landroid/content/res/Resources;") => {
                let resources = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:resources")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(resources) = resources {
                    resources
                } else {
                    let resources = self.heap.instance("Landroid/content/res/Resources;")?;
                    self.heap
                        .get_mut(resources)?
                        .fields
                        .insert("droidless:resources:context".into(), vec![receiver]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:resources".into(), vec![resources]);
                    resources
                });
            }
            ("Landroid/content/Context;", "getAssets()Landroid/content/res/AssetManager;")
            | ("Landroid/content/res/Resources;", "getAssets()Landroid/content/res/AssetManager;") => {
                let manager = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:asset-manager")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(manager) = manager {
                    manager
                } else {
                    let manager = self.heap.instance("Landroid/content/res/AssetManager;")?;
                    self.heap.get_mut(receiver)?.fields.insert(
                        "droidless:asset-manager".into(),
                        vec![manager],
                    );
                    manager
                });
            }
            ("Landroid/content/res/AssetManager;", "open(Ljava/lang/String;)Ljava/io/InputStream;") => {
                let name = self.heap.text(arg(1)?)?;
                ensure!(
                    !name.is_empty()
                        && name.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
                        && !name.contains(['\\', '\0']),
                    fault("Ljava/io/FileNotFoundException;", "invalid asset path")
                );
                let path = format!("assets/{name}");
                let bytes = self.apk.files.get(&path).cloned().ok_or_else(|| {
                    fault("Ljava/io/FileNotFoundException;", format!("asset not found: {name}"))
                })?;
                let stream = self.heap.instance("Ljava/io/FileInputStream;")?;
                self.heap.get_mut(stream)?.data = Data::ByteStream {
                    bytes,
                    position: 0,
                    closed: false,
                };
                result.push(stream);
            }
            ("Landroid/content/res/AssetManager;", "list(Ljava/lang/String;)[Ljava/lang/String;") => {
                let directory = self.heap.text(arg(1)?)?;
                ensure!(
                    directory.split('/').all(|part| part != ".." && part != ".")
                        && !directory.contains(['\\', '\0']),
                    fault("Ljava/lang/IllegalArgumentException;", "invalid asset path")
                );
                let prefix = if directory.is_empty() {
                    "assets/".to_owned()
                } else {
                    format!("assets/{}/", directory.trim_matches('/'))
                };
                let names = self
                    .apk
                    .files
                    .keys()
                    .filter_map(|path| path.strip_prefix(&prefix))
                    .filter_map(|path| path.split('/').next())
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned)
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .map(|name| self.intern(name))
                    .collect::<Result<Vec<_>>>()?;
                let array = self.array("Ljava/lang/String;".into(), names.len())?;
                let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
                    unreachable!();
                };
                for (slot, name) in values.iter_mut().zip(names) {
                    *slot = vec![name];
                }
                result.push(array);
            }
            ("Landroid/view/View;", "getContext()Landroid/content/Context;") => {
                result.push(
                    *self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:context")
                        .and_then(|values| values.first())
                        .context("View has no Context")?,
                );
            }
            ("Landroid/view/View;", "getViewTreeObserver()Landroid/view/ViewTreeObserver;") => {
                self.view_mut(receiver)?;
                let observer = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:tree-observer")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(observer) = observer {
                    observer
                } else {
                    let observer = self.heap.instance("Landroid/view/ViewTreeObserver;")?;
                    self.heap.get_mut(receiver)?.fields.insert(
                        "droidless:view:tree-observer".into(),
                        vec![observer],
                    );
                    observer
                });
            }
            ("Landroid/view/View;", "getResources()Landroid/content/res/Resources;") => {
                let context = *self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:context")
                    .and_then(|values| values.first())
                    .context("View has no Context")?;
                result = self.invoke(
                    Method {
                        class: "Landroid/content/Context;".into(),
                        name: "getResources".into(),
                        parameters: vec![],
                        returns: "Landroid/content/res/Resources;".into(),
                    },
                    vec![context],
                    false,
                )?;
            }
            ("Landroid/content/Context;", "getTheme()Landroid/content/res/Resources$Theme;") => {
                let theme = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:theme")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(theme) = theme {
                    theme
                } else {
                    let theme = self.heap.instance("Landroid/content/res/Resources$Theme;")?;
                    let styles = self
                        .manifest_theme_style(&self.heap.get(receiver)?.class)
                        .map(|style| vec![Word::from(style as i32)])
                        .unwrap_or_default();
                    self.heap
                        .get_mut(theme)?
                        .fields
                        .insert("droidless:theme:styles".into(), styles);
                    self.heap
                        .get_mut(theme)?
                        .fields
                        .insert("droidless:theme:context".into(), vec![receiver]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:theme".into(), vec![theme]);
                    theme
                });
            }
            ("Landroid/content/Context;", sig)
                if [
                    "obtainStyledAttributes([I)Landroid/content/res/TypedArray;",
                    "obtainStyledAttributes(I[I)Landroid/content/res/TypedArray;",
                    "obtainStyledAttributes(Landroid/util/AttributeSet;[I)Landroid/content/res/TypedArray;",
                    "obtainStyledAttributes(Landroid/util/AttributeSet;[III)Landroid/content/res/TypedArray;",
                ]
                .contains(&sig) =>
            {
                let attrs = match method.parameters.as_slice() {
                    [only] if only == "[I" => arg(1)?,
                    [style, _] if style == "I" => arg(2)?,
                    _ => arg(2)?,
                };
                let mut styles = self.theme_styles(receiver)?;
                if method.parameters.first().is_some_and(|parameter| parameter == "I") {
                    styles.push(arg(1)?.int()? as u32);
                }
                if method.parameters.len() == 4 {
                    styles.push(arg(4)?.int()? as u32);
                }
                let attributes = self.styled_attributes(&styles)?;
                result.push(self.typed_array(attrs, &attributes)?);
            }
            ("Landroid/content/res/Resources;", "newTheme()Landroid/content/res/Resources$Theme;") => {
                let theme = self.heap.instance("Landroid/content/res/Resources$Theme;")?;
                if let Some(context) = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:resources:context")
                    .and_then(|values| values.first())
                    .copied()
                {
                    self.heap
                        .get_mut(theme)?
                        .fields
                        .insert("droidless:theme:context".into(), vec![context]);
                    let styles = self.theme_styles(context)?;
                    self.heap
                        .get_mut(theme)?
                        .fields
                        .insert(
                            "droidless:theme:styles".into(),
                            styles.into_iter().map(|style| Word::from(style as i32)).collect(),
                        );
                }
                result.push(theme);
            }
            ("Landroid/content/res/Resources;", "getDisplayMetrics()Landroid/util/DisplayMetrics;") => {
                let metrics = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:display-metrics")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(metrics) = metrics {
                    metrics
                } else {
                    let metrics = self.heap.instance("Landroid/util/DisplayMetrics;")?;
                    let fields = &mut self.heap.get_mut(metrics)?.fields;
                    fields.insert("widthPixels".into(), vec![Word::from(self.width as i32)]);
                    fields.insert("heightPixels".into(), vec![Word::from(self.height as i32)]);
                    for name in ["density", "scaledDensity", "xdpi", "ydpi"] {
                        fields.insert(name.into(), vec![Word::Bits(1.0f32.to_bits())]);
                    }
                    fields.insert("densityDpi".into(), vec![Word::from(160)]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:display-metrics".into(), vec![metrics]);
                    metrics
                });
            }
            ("Landroid/content/res/Resources;", "getConfiguration()Landroid/content/res/Configuration;") => {
                let configuration = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:configuration")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(configuration) = configuration {
                    configuration
                } else {
                    let configuration = self.heap.instance("Landroid/content/res/Configuration;")?;
                    let fields = &mut self.heap.get_mut(configuration)?.fields;
                    fields.insert("orientation".into(), vec![Word::from(1)]);
                    fields.insert("keyboard".into(), vec![Word::from(2)]);
                    fields.insert("screenWidthDp".into(), vec![Word::from(self.width as i32)]);
                    fields.insert("screenHeightDp".into(), vec![Word::from(self.height as i32)]);
                    fields.insert("smallestScreenWidthDp".into(), vec![Word::from(self.width as i32)]);
                    fields.insert("densityDpi".into(), vec![Word::from(160)]);
                    fields.insert("fontScale".into(), vec![Word::Bits(1.0f32.to_bits())]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:configuration".into(), vec![configuration]);
                    configuration
                });
            }
            ("Landroid/content/res/Resources;", "getBoolean(I)Z") => {
                let value = self.apk.resources.resolve(arg(1)?.int()? as u32)?;
                result.push(Word::from(i32::from(value.data != 0)));
            }
            ("Landroid/content/res/Resources;", "getColor(I)I")
            | ("Landroid/content/res/Resources;", "getColor(ILandroid/content/res/Resources$Theme;)I")
            | ("Landroid/content/Context;", "getColor(I)I") => {
                let value = self.apk.resources.resolve(arg(1)?.int()? as u32)?;
                ensure!((0x1c..=0x1f).contains(&value.kind), "resource is not a color");
                result.push(Word::from(value.data as i32));
            }
            ("Landroid/content/res/Resources;", "getInteger(I)I") => {
                let id = arg(1)?.int()? as u32;
                let value = match id {
                    0x010e0000 => 200,
                    0x010e0001 => 400,
                    0x010e0002 => 500,
                    _ => {
                        let value = self.apk.resources.resolve(id)?;
                        ensure!(
                            [0x10, 0x11].contains(&value.kind),
                            "resource is not an integer"
                        );
                        value.data as i32
                    }
                };
                result.push(Word::from(value));
            }
            ("Landroid/content/res/Resources;", "getDimensionPixelOffset(I)I") => {
                let value = self.apk.resources.resolve(arg(1)?.int()? as u32)?;
                result.push(Word::from(dimension(value)?.trunc() as i32));
            }
            ("Landroid/content/res/Resources;", "getDimension(I)F") => {
                let value = self.apk.resources.resolve(arg(1)?.int()? as u32)?;
                result.push(Word::Bits(dimension(value)?.to_bits()));
            }
            ("Landroid/content/res/Resources;", "getDimensionPixelSize(I)I") => {
                let value = dimension(
                    self.apk.resources.resolve(arg(1)?.int()? as u32)?,
                )?;
                let rounded = value.round() as i32;
                result.push(Word::from(if rounded == 0 && value != 0.0 {
                    if value.is_sign_positive() { 1 } else { -1 }
                } else {
                    rounded
                }));
            }
            ("Landroid/content/res/Resources;", "getValue(ILandroid/util/TypedValue;Z)V") => {
                let id = arg(1)?.int()? as u32;
                let value = if arg(3)?.truth() {
                    match self.apk.resources.resolve(id) {
                        Ok(value) => value.clone(),
                        Err(_) if id >> 24 == 1 => Value {
                            kind: 1,
                            data: id,
                            text: None,
                        },
                        Err(error) => return Err(error),
                    }
                } else {
                    self.apk
                        .resources
                        .entries
                        .get(&id)
                        .and_then(|resource| resource.value.clone())
                        .or_else(|| {
                            (id >> 24 == 1).then_some(Value {
                                kind: 1,
                                data: id,
                                text: None,
                            })
                        })
                        .context("resource has no simple value")?
                };
                let typed_value = arg(2)?;
                let class = "Landroid/util/TypedValue;";
                let fields = &mut self.heap.get_mut(typed_value)?.fields;
                fields.insert(format!("{class}->type:I"), vec![Word::from(i32::from(value.kind))]);
                fields.insert(format!("{class}->data:I"), vec![Word::from(value.data as i32)]);
                let asset_cookie = if value.kind == 1 && value.data == id && id >> 24 == 1 {
                    0
                } else {
                    1
                };
                fields.insert(
                    format!("{class}->assetCookie:I"),
                    vec![Word::from(asset_cookie)],
                );
                fields.insert(format!("{class}->resourceId:I"), vec![Word::from(id as i32)]);
                fields.insert(format!("{class}->changingConfigurations:I"), vec![Word::ZERO]);
                let string = if value.kind == 3 {
                    self.heap.string(value.display())?
                } else {
                    Word::ZERO
                };
                self.heap.get_mut(typed_value)?.fields.insert(
                    format!("{class}->string:Ljava/lang/CharSequence;"),
                    vec![string],
                );
            }
            ("Landroid/content/Context;", "getDrawable(I)Landroid/graphics/drawable/Drawable;")
            | ("Landroid/content/res/Resources;", "getDrawable(I)Landroid/graphics/drawable/Drawable;")
            | ("Landroid/content/res/Resources;", "getDrawable(ILandroid/content/res/Resources$Theme;)Landroid/graphics/drawable/Drawable;") => {
                let id = arg(1)?;
                let drawable = self.heap.instance("Landroid/graphics/drawable/Drawable;")?;
                self.heap.get_mut(drawable)?.fields.insert("resourceId".into(), vec![id]);
                if let Some((info, bytes)) = self.image_resource(id.int()? as u32)? {
                    let bitmap = self.bitmap(info, bytes, None)?;
                    self.heap.get_mut(drawable)?.fields.insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                }
                result.push(drawable);
            }
            ("Landroid/content/res/Resources;", "obtainAttributes(Landroid/util/AttributeSet;[I)Landroid/content/res/TypedArray;")
            | ("Landroid/content/res/Resources$Theme;", "obtainStyledAttributes([I)Landroid/content/res/TypedArray;")
            | ("Landroid/content/res/Resources$Theme;", "obtainStyledAttributes(Landroid/util/AttributeSet;[III)Landroid/content/res/TypedArray;") => {
                let attrs = arg(if method.parameters.first().is_some_and(|ty| ty == "[I") { 1 } else { 2 })?;
                let mut styles = if method.class == "Landroid/content/res/Resources$Theme;" {
                    self.theme_styles(receiver)?
                } else {
                    vec![]
                };
                if method.parameters.len() == 4 {
                    styles.push(arg(4)?.int()? as u32);
                }
                let attributes = self.styled_attributes(&styles)?;
                result.push(self.typed_array(attrs, &attributes)?);
            }
            ("Landroid/content/res/Resources$Theme;", "applyStyle(IZ)V") => {
                let style = arg(1)?;
                let force = arg(2)?.truth();
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                let styles = fields
                    .entry("droidless:theme:styles".into())
                    .or_default();
                if force {
                    styles.push(style);
                } else {
                    styles.insert(0, style);
                }
            }
            ("Landroid/content/res/Resources$Theme;", "setTo(Landroid/content/res/Resources$Theme;)V") => {
                self.heap.get_mut(receiver)?.fields = self.heap.get(arg(1)?)?.fields.clone();
            }
            ("Landroid/content/res/Resources$Theme;", "resolveAttribute(ILandroid/util/TypedValue;Z)Z") => {
                result.push(Word::ZERO);
            }
            ("Landroid/content/res/TypedArray;", sig)
                if [
                    "getBoolean(IZ)Z",
                    "getColor(II)I",
                    "getDimension(IF)F",
                    "getDimensionPixelOffset(II)I",
                    "getDimensionPixelSize(II)I",
                    "getFloat(IF)F",
                    "getInt(II)I",
                    "getInteger(II)I",
                    "getLayoutDimension(II)I",
                    "getResourceId(II)I",
                    "getString(I)Ljava/lang/String;",
                    "getText(I)Ljava/lang/CharSequence;",
                    "getDrawable(I)Landroid/graphics/drawable/Drawable;",
                    "getColorStateList(I)Landroid/content/res/ColorStateList;",
                    "getTextArray(I)[Ljava/lang/CharSequence;",
                    "getValue(ILandroid/util/TypedValue;)Z",
                    "hasValue(I)Z",
                    "length()I",
                    "getIndexCount()I",
                    "getPositionDescription()Ljava/lang/String;",
                    "recycle()V",
                ]
                .contains(&sig) =>
            {
                let index = if method.parameters.is_empty() {
                    None
                } else {
                    args.get(1).copied().map(Word::int).transpose()?.and_then(|index| usize::try_from(index).ok())
                };
                let (length, value) = match &self.heap.get(receiver)?.data {
                    Data::TypedArray(values) => (
                        values.len(),
                        index.and_then(|index| values.get(index).cloned().flatten()),
                    ),
                    _ => bail!("uninitialized TypedArray"),
                };
                match sig {
                    "getBoolean(IZ)Z" => result.push(value.as_ref().map_or(arg(2)?.int()?, |value| i32::from(value.data != 0)).into()),
                    "getColor(II)I" | "getInt(II)I" | "getInteger(II)I" | "getLayoutDimension(II)I" => {
                        result.push(Word::from(value.map_or(arg(2)?.int()?, |value| value.data as i32)));
                    }
                    "getResourceId(II)I" => {
                        result.push(Word::from(value.filter(|value| value.kind == 1).map_or(arg(2)?.int()?, |value| value.data as i32)));
                    }
                    "getDimension(IF)F" | "getFloat(IF)F" => {
                        let fallback = arg(2)?.int()? as u32;
                        let bits = value.map_or(fallback, |value| value.data);
                        result.push(Word::Bits(bits));
                    }
                    "getDimensionPixelOffset(II)I" | "getDimensionPixelSize(II)I" => result.push(arg(2)?),
                    "getString(I)Ljava/lang/String;" | "getText(I)Ljava/lang/CharSequence;" => {
                        if let Some(value) = value {
                            if let Some(text) = value.text {
                                result.push(self.heap.string(text)?);
                            } else if value.kind == 1 {
                                result.push(self.heap.string(self.resource_text(value.data)? )?);
                            } else {
                                result.push(Word::ZERO);
                            }
                        } else {
                            result.push(Word::ZERO);
                        }
                    }
                    "getDrawable(I)Landroid/graphics/drawable/Drawable;" | "getColorStateList(I)Landroid/content/res/ColorStateList;" => {
                        result.push(if let Some(value) = value.filter(|value| value.kind == 1) {
                            let class = if sig.starts_with("getDrawable") { "Landroid/graphics/drawable/Drawable;" } else { "Landroid/content/res/ColorStateList;" };
                            let object = self.heap.instance(class)?;
                            self.heap.get_mut(object)?.fields.insert("resourceId".into(), vec![Word::from(value.data as i32)]);
                            if class == "Landroid/graphics/drawable/Drawable;"
                                && let Some((info, bytes)) = self.image_resource(value.data)?
                            {
                                let bitmap = self.bitmap(info, bytes, None)?;
                                self.heap.get_mut(object)?.fields.insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                            }
                            object
                        } else { Word::ZERO });
                    }
                    "getTextArray(I)[Ljava/lang/CharSequence;" => result.push(Word::ZERO),
                    "getValue(ILandroid/util/TypedValue;)Z" => result.push(Word::from(i32::from(value.is_some()))),
                    "hasValue(I)Z" => result.push(Word::from(i32::from(value.is_some()))),
                    "length()I" | "getIndexCount()I" => result.push(Word::from(length as i32)),
                    "getPositionDescription()Ljava/lang/String;" => result.push(self.heap.string("TypedArray".into())?),
                    "recycle()V" => {}
                    _ => unreachable!(),
                }
            }
            ("Landroid/app/Activity;", "getWindowManager()Landroid/view/WindowManager;") => {
                result.push(self.heap.instance("Landroid/view/WindowManager;")?)
            }
            ("Landroid/app/Activity;", "getLayoutInflater()Landroid/view/LayoutInflater;") => {
                result = self.invoke(
                    Method {
                        class: "Landroid/view/LayoutInflater;".into(),
                        name: "from".into(),
                        parameters: vec!["Landroid/content/Context;".into()],
                        returns: "Landroid/view/LayoutInflater;".into(),
                    },
                    vec![receiver],
                    false,
                )?;
            }
            ("Landroid/app/Activity;", "getWindow()Landroid/view/Window;") => {
                self.screen(receiver)?;
                let window = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:window")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(window) = window {
                    window
                } else {
                    let window = self.heap.instance("Landroid/view/Window;")?;
                    self.heap
                        .get_mut(window)?
                        .fields
                        .insert("droidless:window:owner".into(), vec![receiver]);
                    self.heap
                        .get_mut(window)?
                        .fields
                        .insert("droidless:window:callback".into(), vec![receiver]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:window".into(), vec![window]);
                    window
                });
            }
            (
                "Landroid/view/Window;",
                "getDecorView()Landroid/view/View;" | "peekDecorView()Landroid/view/View;",
            ) => {
                let owner = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:window:owner")
                    .and_then(|values| values.first())
                    .copied()
                    .context("Window has no Activity")?;
                let root = if let Some(root) = self.screen(owner)?.root {
                    root
                } else {
                    let root = self.heap.instance("Landroid/widget/FrameLayout;")?;
                    self.set_content(owner, root)?;
                    root
                };
                result.push(root);
            }
            ("Landroid/view/Window;", "setContentView(Landroid/view/View;)V") => {
                let owner = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:window:owner")
                    .and_then(|values| values.first())
                    .copied()
                    .context("Window has no Activity")?;
                let view = arg(1)?;
                ensure!(
                    self.heap.get(view)?.view.is_some(),
                    "Window content must be a View"
                );
                self.install_android_content_id(view, 0)?;
                self.set_content(owner, view)?;
            }
            ("Landroid/view/Window;", "findViewById(I)Landroid/view/View;") => {
                let owner = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:window:owner")
                    .and_then(|values| values.first())
                    .copied()
                    .context("Window has no Activity")?;
                result.push(if let Some(root) = self.screen(owner)?.root {
                    self.find_view(root, arg(1)?.int()? as u32, 0)?
                        .unwrap_or(Word::ZERO)
                } else {
                    Word::ZERO
                });
            }
            ("Landroid/view/Window;", "getCallback()Landroid/view/Window$Callback;") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:window:callback")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/Window;", "setCallback(Landroid/view/Window$Callback;)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:window:callback".into(), vec![arg(1)?]);
            }
            ("Landroid/view/Window;", "requestFeature(I)Z") => {
                self.heap.get_mut(receiver)?.fields.insert(
                    format!("droidless:window:feature:{}", arg(1)?.int()?),
                    vec![Word::from(1)],
                );
                result.push(Word::from(1));
            }
            ("Landroid/view/Window;", "setFlags(II)V") => {
                let object = self.heap.get_mut(receiver)?;
                let old = object
                    .fields
                    .get("droidless:window:flags")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                let flags = arg(1)?.int()?;
                let mask = arg(2)?.int()?;
                object.fields.insert(
                    "droidless:window:flags".into(),
                    vec![Word::from((old & !mask) | (flags & mask))],
                );
            }
            (
                "Landroid/view/Window;",
                "getAttributes()Landroid/view/WindowManager$LayoutParams;",
            ) => {
                result.push(
                    self.heap
                        .instance("Landroid/view/WindowManager$LayoutParams;")?,
                );
            }
            ("Landroid/view/WindowManager;", "getDefaultDisplay()Landroid/view/Display;") => {
                result.push(self.heap.instance("Landroid/view/Display;")?)
            }
            (
                "Landroid/view/LayoutInflater;",
                "from(Landroid/content/Context;)Landroid/view/LayoutInflater;",
            ) => {
                let context = arg(0)?;
                let inflater = self
                    .heap
                    .get(context)?
                    .fields
                    .get("droidless:layout-inflater")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(inflater) = inflater {
                    inflater
                } else {
                    let inflater = self.heap.instance("Landroid/view/LayoutInflater;")?;
                    self.heap
                        .get_mut(inflater)?
                        .fields
                        .insert("droidless:layout-inflater:context".into(), vec![context]);
                    self.heap
                        .get_mut(context)?
                        .fields
                        .insert("droidless:layout-inflater".into(), vec![inflater]);
                    inflater
                });
            }
            (
                "Landroid/view/LayoutInflater;",
                "cloneInContext(Landroid/content/Context;)Landroid/view/LayoutInflater;",
            ) => {
                let clone = self.heap.instance("Landroid/view/LayoutInflater;")?;
                self.heap.get_mut(clone)?.fields = self.heap.get(receiver)?.fields.clone();
                self.heap
                    .get_mut(clone)?
                    .fields
                    .insert("droidless:layout-inflater:context".into(), vec![arg(1)?]);
                result.push(clone);
            }
            (
                "Landroid/view/LayoutInflater;",
                "getFactory()Landroid/view/LayoutInflater$Factory;",
            ) => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:layout-inflater:factory")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            (
                "Landroid/view/LayoutInflater;",
                "setFactory(Landroid/view/LayoutInflater$Factory;)V",
            )
            | (
                "Landroid/view/LayoutInflater;",
                "setFactory2(Landroid/view/LayoutInflater$Factory2;)V",
            ) => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:layout-inflater:factory".into(), vec![arg(1)?]);
            }
            (
                "Landroid/view/LayoutInflater;",
                "inflate(ILandroid/view/ViewGroup;)Landroid/view/View;",
            )
            | (
                "Landroid/view/LayoutInflater;",
                "inflate(ILandroid/view/ViewGroup;Z)Landroid/view/View;",
            ) => {
                let resource = arg(1)?.int()? as u32;
                let parent = arg(2)?;
                let attach = if method.parameters.len() == 2 {
                    parent != Word::ZERO
                } else {
                    arg(3)?.int()? != 0
                };
                let context = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:layout-inflater:context")
                    .and_then(|values| values.first())
                    .copied()
                    .context("LayoutInflater has no Context")?;
                let view = self.inflate_id(resource, 0, context)?;
                if attach && parent != Word::ZERO {
                    self.invoke(
                        Method {
                            class: "Landroid/view/ViewGroup;".into(),
                            name: "addView".into(),
                            parameters: vec!["Landroid/view/View;".into()],
                            returns: "V".into(),
                        },
                        vec![parent, view],
                        true,
                    )?;
                    result.push(parent);
                } else {
                    result.push(view);
                }
            }
            ("Landroid/util/DisplayMetrics;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/Rect;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/RectF;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/content/res/Resources;", "<init>(Landroid/content/res/AssetManager;Landroid/util/DisplayMetrics;Landroid/content/res/Configuration;)V") => {
                for (name, value) in [
                    ("droidless:resources:assets", arg(1)?),
                    ("droidless:resources:display-metrics", arg(2)?),
                    ("droidless:resources:configuration", arg(3)?),
                ] {
                    self.heap.get(value)?;
                    self.heap.get_mut(receiver)?.fields.insert(name.into(), vec![value]);
                }
            }
            ("Landroid/graphics/Paint;", "<init>()V") => {
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:paint:color".into(),
                    vec![Word::from(0xff00_0000u32 as i32)],
                );
            }
            ("Landroid/graphics/Paint;", "<init>(I)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("droidless:paint:flags".into(), vec![arg(1)?]);
                fields.insert(
                    "droidless:paint:color".into(),
                    vec![Word::from(0xff00_0000u32 as i32)],
                );
            }
            ("Landroid/graphics/Paint;", "setColor(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:paint:color".into(), vec![arg(1)?]);
            }
            ("Landroid/graphics/Paint;", "getColor()I") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:paint:color")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::from(0xff00_0000u32 as i32)),
                );
            }
            ("Landroid/graphics/Rect;", "<init>(IIII)V")
            | ("Landroid/graphics/Rect;", "set(IIII)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for (name, value) in ["left", "top", "right", "bottom"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(name.into(), vec![value]);
                }
            }
            ("Landroid/graphics/RectF;", "<init>(FFFF)V")
            | ("Landroid/graphics/RectF;", "set(FFFF)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for (name, value) in ["left", "top", "right", "bottom"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(name.into(), vec![value]);
                }
            }
            ("Landroid/graphics/Rect;", "set(Landroid/graphics/Rect;)V") => {
                self.heap.get_mut(receiver)?.fields = self.heap.get(arg(1)?)?.fields.clone();
            }
            ("Landroid/graphics/RectF;", "set(Landroid/graphics/RectF;)V") => {
                self.heap.get_mut(receiver)?.fields = self.heap.get(arg(1)?)?.fields.clone();
            }
            ("Landroid/content/res/Resources;", "getSystem()Landroid/content/res/Resources;") => {
                let key = "droidless:system-resources";
                let resources = if let Some(resources) = self
                    .statics
                    .get(key)
                    .and_then(|values| values.first())
                    .copied()
                {
                    resources
                } else {
                    let resources = self.heap.instance("Landroid/content/res/Resources;")?;
                    self.statics.insert(key.into(), vec![resources]);
                    resources
                };
                result.push(resources);
            }
            ("Landroid/util/TypedValue;", "applyDimension(IFLandroid/util/DisplayMetrics;)F") => {
                let unit = arg(0)?.int()?;
                let value = f32::from_bits(arg(1)?.int()? as u32);
                let metrics = self.heap.get(arg(2)?)?;
                let metric = |name: &str, fallback: f32| -> Result<f32> {
                    metrics
                        .fields
                        .get(&format!("Landroid/util/DisplayMetrics;->{name}:F"))
                        .and_then(|values| values.first())
                        .copied()
                        .map(|value| Ok(f32::from_bits(value.int()? as u32)))
                        .unwrap_or(Ok(fallback))
                };
                let density = metric("density", 1.0)?;
                let xdpi = metric("xdpi", 160.0)?;
                let scaled_density = metric("scaledDensity", density)?;
                let result_value = match unit {
                    0 => value,
                    1 => value * density,
                    2 => value * scaled_density,
                    3 => value * xdpi / 72.0,
                    4 => value * xdpi,
                    5 => value * xdpi / 25.4,
                    _ => 0.0,
                };
                result.push(Word::Bits(result_value.to_bits()));
            }
            ("Landroid/content/ContextWrapper;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/content/ContextWrapper;", "attachBaseContext(Landroid/content/Context;)V") => {
                let context = arg(1)?;
                self.heap.get(context)?;
                self.heap.get_mut(receiver)?.fields.insert("droidless:context:base".into(), vec![context]);
            }
            ("Landroid/content/ContextWrapper;", "getBaseContext()Landroid/content/Context;") => {
                let context = self.heap.get(receiver)?.fields.get("droidless:context:base").and_then(|values| values.first()).copied().unwrap_or(Word::ZERO);
                result.push(context);
            }
            ("Landroid/view/ViewGroup$LayoutParams;", "<init>(II)V")
            | ("Landroid/view/ViewGroup$MarginLayoutParams;", "<init>(II)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->width:I".into(), vec![arg(1)?]);
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->height:I".into(), vec![arg(2)?]);
            }
            ("Landroid/view/ViewGroup$MarginLayoutParams;", "<init>(Landroid/view/ViewGroup$MarginLayoutParams;)V") => {
                let source = self.heap.get(arg(1)?)?.fields.clone();
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for key in [
                    "Landroid/view/ViewGroup$LayoutParams;->width:I",
                    "Landroid/view/ViewGroup$LayoutParams;->height:I",
                    "Landroid/view/ViewGroup$MarginLayoutParams;->leftMargin:I",
                    "Landroid/view/ViewGroup$MarginLayoutParams;->topMargin:I",
                    "Landroid/view/ViewGroup$MarginLayoutParams;->rightMargin:I",
                    "Landroid/view/ViewGroup$MarginLayoutParams;->bottomMargin:I",
                ] {
                    if let Some(value) = source.get(key) {
                        fields.insert(key.into(), value.clone());
                    }
                }
            }
            ("Landroid/widget/LinearLayout$LayoutParams;", "<init>(II)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->width:I".into(), vec![arg(1)?]);
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->height:I".into(), vec![arg(2)?]);
            }
            ("Landroid/widget/LinearLayout$LayoutParams;", "<init>(IIF)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->width:I".into(), vec![arg(1)?]);
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->height:I".into(), vec![arg(2)?]);
                fields.insert("Landroid/widget/LinearLayout$LayoutParams;->weight:F".into(), vec![arg(3)?]);
            }
            ("Landroid/widget/FrameLayout$LayoutParams;", "<init>(II)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->width:I".into(), vec![arg(1)?]);
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->height:I".into(), vec![arg(2)?]);
            }
            ("Landroid/widget/FrameLayout$LayoutParams;", "<init>(III)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->width:I".into(), vec![arg(1)?]);
                fields.insert("Landroid/view/ViewGroup$LayoutParams;->height:I".into(), vec![arg(2)?]);
                fields.insert("Landroid/widget/FrameLayout$LayoutParams;->gravity:I".into(), vec![arg(3)?]);
            }
            ("Landroid/view/ViewGroup$MarginLayoutParams;", "setMargins(IIII)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for (name, value) in ["leftMargin", "topMargin", "rightMargin", "bottomMargin"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(format!("Landroid/view/ViewGroup$MarginLayoutParams;->{name}:I"), vec![value]);
                }
            }
            (
                "Landroid/view/View;"
                | "Landroid/view/ViewGroup;",
                "<init>(Landroid/content/Context;)V",
            ) => {
                self.view_mut(receiver)?;
                let context = arg(1)?;
                self.heap.get(context)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:context".into(), vec![context]);
            }
            ("Landroid/util/TypedValue;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/view/View$AccessibilityDelegate;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            (
                "Landroid/view/View;"
                | "Landroid/view/ViewGroup;"
                | "Landroid/widget/LinearLayout;"
                | "Landroid/widget/FrameLayout;"
                | "Landroid/widget/TextView;"
                | "Landroid/widget/Button;"
                | "Landroid/widget/EditText;"
                | "Landroid/widget/ImageView;"
                | "Landroid/widget/ImageButton;"
                | "Landroid/widget/RelativeLayout;"
                | "Landroid/widget/ScrollView;"
                | "Landroid/widget/HorizontalScrollView;"
                | "Landroid/widget/Space;",
                "<init>(Landroid/content/Context;Landroid/util/AttributeSet;)V",
            )
            | (
                "Landroid/view/View;"
                | "Landroid/view/ViewGroup;"
                | "Landroid/widget/LinearLayout;"
                | "Landroid/widget/FrameLayout;"
                | "Landroid/widget/TextView;"
                | "Landroid/widget/Button;"
                | "Landroid/widget/EditText;"
                | "Landroid/widget/ImageView;"
                | "Landroid/widget/ImageButton;"
                | "Landroid/widget/RelativeLayout;"
                | "Landroid/widget/ScrollView;"
                | "Landroid/widget/HorizontalScrollView;"
                | "Landroid/widget/Space;",
                "<init>(Landroid/content/Context;Landroid/util/AttributeSet;I)V",
            ) => {
                self.view_mut(receiver)?;
                self.heap.get(arg(1)?)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:context".into(), vec![arg(1)?]);
                if arg(2)? != Word::ZERO {
                    self.heap.get(arg(2)?)?;
                }
            }
            ("Landroid/view/Display;", "getMetrics(Landroid/util/DisplayMetrics;)V") => {
                let (width, height) = (self.width as i32, self.height as i32);
                let fields = &mut self.heap.get_mut(arg(1)?)?.fields;
                fields.insert(
                    "Landroid/util/DisplayMetrics;->widthPixels:I".into(),
                    vec![Word::from(width)],
                );
                fields.insert(
                    "Landroid/util/DisplayMetrics;->heightPixels:I".into(),
                    vec![Word::from(height)],
                );
                fields.insert(
                    "Landroid/util/DisplayMetrics;->density:F".into(),
                    vec![Word::Bits(1.0f32.to_bits())],
                );
            }
            ("Landroid/view/ViewConfiguration;", "get(Landroid/content/Context;)Landroid/view/ViewConfiguration;") => {
                self.heap.get(arg(0)?)?;
                let configuration = self
                    .statics
                    .get("droidless:view-configuration")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(configuration) = configuration {
                    configuration
                } else {
                    let configuration = self.heap.instance("Landroid/view/ViewConfiguration;")?;
                    self.statics.insert(
                        "droidless:view-configuration".into(),
                        vec![configuration],
                    );
                    configuration
                });
            }
            ("Landroid/view/ViewConfiguration;", "getScaledTouchSlop()I") => {
                self.heap.get(receiver)?;
                result.push(Word::from(8));
            }
            ("Landroid/view/ViewConfiguration;", "getScaledMaximumFlingVelocity()I") => {
                self.heap.get(receiver)?;
                result.push(Word::from(8000));
            }
            ("Landroid/view/ViewConfiguration;", "getScaledMinimumFlingVelocity()I") => {
                self.heap.get(receiver)?;
                result.push(Word::from(50));
            }
            ("Landroid/widget/OverScroller;", "<init>(Landroid/content/Context;Landroid/view/animation/Interpolator;)V") => {
                self.heap.get(receiver)?;
                self.heap.get(arg(1)?)?;
                if arg(2)? != Word::ZERO {
                    self.heap.get(arg(2)?)?;
                }
            }
            ("Landroid/widget/OverScroller;", "abortAnimation()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/widget/LinearLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/FrameLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/TextView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/Button;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/ImageView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/ImageButton;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/RelativeLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/ScrollView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/HorizontalScrollView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/Space;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/EditText;", "<init>(Landroid/content/Context;)V") => {
                self.view_mut(receiver)?;
                self.heap.get(arg(1)?)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:context".into(), vec![arg(1)?]);
            }
            ("Landroid/widget/LinearLayout;", "setOrientation(I)V") => {
                self.view_mut(receiver)?.orientation = arg(1)?.int()?
            }
            ("Landroid/widget/LinearLayout;", "setGravity(I)V")
            | ("Landroid/widget/TextView;", "setGravity(I)V") => {
                self.view_mut(receiver)?.gravity = arg(1)?.int()? as u32;
            }
            ("Landroid/widget/LinearLayout;", "getGravity()I")
            | ("Landroid/widget/TextView;", "getGravity()I") => {
                result.push(Word::from(self.view_mut(receiver)?.gravity as i32));
            }
            (
                "Landroid/view/ViewGroup;",
                "setOnHierarchyChangeListener(Landroid/view/ViewGroup$OnHierarchyChangeListener;)V",
            ) => {
                let listener = arg(1)?;
                if listener != Word::ZERO {
                    self.heap.get(listener)?;
                }
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:hierarchy-listener".into(),
                    vec![listener],
                );
            }
            ("Landroid/view/ViewGroup;", "checkLayoutParams(Landroid/view/ViewGroup$LayoutParams;)Z") => {
                let params = arg(1)?;
                result.push(Word::from(i32::from(
                    params != Word::ZERO
                        && self.is_a(&self.heap.get(params)?.class, "Landroid/view/ViewGroup$LayoutParams;"),
                )));
            }
            ("Landroid/view/ViewGroup;", "addView(Landroid/view/View;)V") => {
                let child = arg(1)?;
                self.view_mut(child)?;
                self.view_mut(receiver)?.children.push(child);
                self.heap.get_mut(child)?.fields.insert("droidless:view:parent".into(), vec![receiver]);
                self.hierarchy_change(receiver, child, true)?;
            }
            ("Landroid/view/ViewGroup;", "addView(Landroid/view/View;I)V") => {
                let child = arg(1)?;
                let index = arg(2)?.int()?;
                self.view_mut(child)?;
                let children = &mut self.view_mut(receiver)?.children;
                ensure!(
                    index >= -1 && index <= children.len() as i32,
                    "View child index is out of bounds"
                );
                children.insert(if index == -1 { children.len() } else { index as usize }, child);
                self.heap.get_mut(child)?.fields.insert("droidless:view:parent".into(), vec![receiver]);
                self.hierarchy_change(receiver, child, true)?;
            }
            ("Landroid/view/ViewGroup;", "addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V") => {
                let child = arg(1)?;
                let params = arg(2)?;
                self.view_mut(child)?;
                if params != Word::ZERO { self.heap.get(params)?; }
                self.heap.get_mut(child)?.fields.insert("droidless:view:parent".into(), vec![receiver]);
                self.heap.get_mut(child)?.fields.insert("droidless:view:layout-params".into(), vec![params]);
                self.view_mut(receiver)?.children.push(child);
                self.hierarchy_change(receiver, child, true)?;
            }
            ("Landroid/view/ViewGroup;", "getChildCount()I") => {
                result.push(Word::from(self.view_mut(receiver)?.children.len() as i32));
            }
            ("Landroid/view/ViewGroup;", "getChildAt(I)Landroid/view/View;") => {
                let index = arg(1)?.int()?;
                result.push(if index >= 0 {
                    self.view_mut(receiver)?
                        .children
                        .get(index as usize)
                        .copied()
                        .unwrap_or(Word::ZERO)
                } else {
                    Word::ZERO
                });
            }
            ("Landroid/view/ViewGroup;", "removeAllViews()V") => {
                let children = std::mem::take(&mut self.view_mut(receiver)?.children);
                for child in children {
                    self.heap.get_mut(child)?.fields.remove("droidless:view:parent");
                    self.hierarchy_change(receiver, child, false)?;
                }
            }
            ("Landroid/view/ViewGroup;", "removeView(Landroid/view/View;)V") => {
                let child = arg(1)?;
                let children = &mut self.view_mut(receiver)?.children;
                if let Some(index) = children.iter().position(|view| *view == child) {
                    children.remove(index);
                    self.heap.get_mut(child)?.fields.remove("droidless:view:parent");
                    self.hierarchy_change(receiver, child, false)?;
                }
            }
            ("Landroid/view/ViewGroup;", "setMotionEventSplittingEnabled(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:motion-event-splitting".into(), vec![arg(1)?]);
            }
            ("Landroid/widget/TextView;", "setText(Ljava/lang/CharSequence;)V") => {
                self.set_text_view(receiver, arg(1)?)?;
            }
            ("Landroid/widget/TextView;", "setText(I)V") => {
                let text = self.resource_text(arg(1)?.int()? as u32)?;
                self.set_view_text(receiver, text, vec![])?;
            }
            ("Landroid/widget/TextView;", "append(Ljava/lang/CharSequence;)V") => {
                self.append_text_view(receiver, arg(1)?)?;
            }
            ("Landroid/widget/TextView;", "getText()Ljava/lang/CharSequence;")
            | ("Landroid/widget/EditText;", "getText()Landroid/text/Editable;") => {
                if method.class == "Landroid/widget/EditText;" {
                    result.push(self.editable_text(receiver)?);
                } else {
                    let text = self.view_mut(receiver)?.text.clone();
                    result.push(self.heap.string(text)?);
                }
            }
            ("Landroid/widget/TextView;", "setEllipsize(Landroid/text/TextUtils$TruncateAt;)V") => {
                let value = arg(1)?;
                if value != Word::ZERO { self.heap.get(value)?; }
                self.heap.get_mut(receiver)?.fields.insert("droidless:text:ellipsize".into(), vec![value]);
            }
            ("Landroid/widget/TextView;", "setSingleLine()V") => {
                self.heap.get_mut(receiver)?.fields.insert("droidless:text:single-line".into(), vec![Word::from(1)]);
            }
            ("Landroid/widget/TextView;", "setSingleLine(Z)V") => {
                self.heap.get_mut(receiver)?.fields.insert("droidless:text:single-line".into(), vec![arg(1)?]);
            }
            ("Landroid/widget/TextView;", "setMaxLines(I)V")
            | ("Landroid/widget/TextView;", "setMinLines(I)V") => {
                self.heap.get_mut(receiver)?.fields.insert(format!("droidless:text:{}", method.name), vec![arg(1)?]);
            }
            ("Landroid/widget/ImageView;", "getDrawable()Landroid/graphics/drawable/Drawable;") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:image:drawable")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/widget/ImageView;", "setImageDrawable(Landroid/graphics/drawable/Drawable;)V") => {
                let drawable = arg(1)?;
                if drawable != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(drawable)?.class, "Landroid/graphics/drawable/Drawable;"),
                        "ImageView requires a Drawable"
                    );
                }
                let image = if drawable == Word::ZERO {
                    None
                } else if let Some(bitmap) = self.heap.get(drawable)?.fields
                    .get("droidless:drawable:bitmap").and_then(|values| values.first()).copied()
                {
                    match &self.heap.get(bitmap)?.data {
                        Data::Bitmap { bytes, recycled: false, .. } => Some(bytes.clone()),
                        Data::Bitmap { .. } => bail!("cannot display a recycled Bitmap"),
                        _ => bail!("Drawable image is not initialized"),
                    }
                } else {
                    self.heap.get(drawable)?.fields.get("resourceId")
                        .and_then(|values| values.first())
                        .map(|value| value.int().map(|id| id as u32))
                        .transpose()?
                        .map(|id| self.image_resource(id))
                        .transpose()?
                        .flatten()
                        .map(|(_, bytes)| bytes)
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:image:drawable".into(), vec![drawable]);
                self.view_mut(receiver)?.image = image;
            }
            ("Landroid/widget/ImageView;", "setImageBitmap(Landroid/graphics/Bitmap;)V") => {
                let bitmap = arg(1)?;
                let drawable = if bitmap == Word::ZERO {
                    Word::ZERO
                } else {
                    let Data::Bitmap { bytes, recycled, .. } = &self.heap.get(bitmap)?.data else {
                        bail!("ImageView requires a Bitmap");
                    };
                    ensure!(!recycled, "cannot display a recycled Bitmap");
                    let image = bytes.clone();
                    let drawable = self.heap.instance("Landroid/graphics/drawable/BitmapDrawable;")?;
                    self.heap.get_mut(drawable)?.fields.insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                    self.view_mut(receiver)?.image = Some(image);
                    drawable
                };
                self.heap.get_mut(receiver)?.fields.insert("droidless:image:drawable".into(), vec![drawable]);
                if bitmap == Word::ZERO { self.view_mut(receiver)?.image = None; }
            }
            ("Landroid/widget/ImageView;", "setImageResource(I)V") => {
                let id = arg(1)?.int()?;
                let image = if id == 0 {
                    None
                } else {
                    self.image_resource(id as u32)?.map(|(_, bytes)| bytes)
                };
                let drawable = if id == 0 {
                    Word::ZERO
                } else {
                    let drawable = self.heap.instance("Landroid/graphics/drawable/Drawable;")?;
                    self.heap
                        .get_mut(drawable)?
                        .fields
                        .insert("resourceId".into(), vec![Word::from(id)]);
                    drawable
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:image:drawable".into(), vec![drawable]);
                self.view_mut(receiver)?.image = image;
            }
            ("Landroid/graphics/drawable/BitmapDrawable;", "<init>(Landroid/content/res/Resources;Landroid/graphics/Bitmap;)V") => {
                let bitmap = arg(2)?;
                self.heap.get(arg(1)?)?;
                ensure!(matches!(&self.heap.get(bitmap)?.data, Data::Bitmap { recycled: false, .. }), "BitmapDrawable requires a live Bitmap");
                self.heap.get_mut(receiver)?.fields.insert("droidless:drawable:bitmap".into(), vec![bitmap]);
            }
            ("Landroid/graphics/drawable/BitmapDrawable;", "<init>(Landroid/graphics/Bitmap;)V") => {
                let bitmap = arg(1)?;
                ensure!(matches!(&self.heap.get(bitmap)?.data, Data::Bitmap { recycled: false, .. }), "BitmapDrawable requires a live Bitmap");
                self.heap.get_mut(receiver)?.fields.insert("droidless:drawable:bitmap".into(), vec![bitmap]);
            }
            ("Landroid/graphics/drawable/BitmapDrawable;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/widget/TextView;", "setTextSize(F)V") => {
                let size = f32::from_bits(arg(1)?.int()? as u32);
                ensure!(size.is_finite() && size >= 0.0, "invalid text size");
                self.view_mut(receiver)?.text_size = size;
            }
            ("Landroid/widget/TextView;", "setTextColor(I)V") => {
                self.view_mut(receiver)?.text_color = arg(1)?.int()? as u32
            }
            ("Landroid/widget/TextView;", "setKeyListener(Landroid/text/method/KeyListener;)V") => {
                ensure!(arg(1)? == Word::ZERO, "non-null KeyListener unsupported");
                self.view_mut(receiver)?.editable = false;
            }
            ("Landroid/widget/EditText;", "setSelection(I)V") => {
                let n = arg(1)?.int()?;
                ensure!(
                    n >= 0 && n as usize <= self.view_mut(receiver)?.text.encode_utf16().count(),
                    "selection index out of range"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("selection".into(), vec![Word::from(n)]);
            }
            ("Landroid/view/View;", "setOnClickListener(Landroid/view/View$OnClickListener;)V") => {
                let listener = arg(1)?;
                self.view_mut(receiver)?.listener = if listener == Word::ZERO {
                    None
                } else {
                    self.heap.get(listener)?;
                    Some(listener)
                };
            }
            ("Landroid/view/View;", "setOnFocusChangeListener(Landroid/view/View$OnFocusChangeListener;)V") => {
                let listener = arg(1)?;
                if listener != Word::ZERO {
                    self.heap.get(listener)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:focus-change-listener".into(), vec![listener]);
            }
            ("Landroid/view/View;", "setOnTouchListener(Landroid/view/View$OnTouchListener;)V") => {
                let listener = arg(1)?;
                if listener != Word::ZERO {
                    self.heap.get(listener)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:touch-listener".into(), vec![listener]);
            }
            ("Landroid/view/View;", "setOnApplyWindowInsetsListener(Landroid/view/View$OnApplyWindowInsetsListener;)V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/view/View;", "setFitsSystemWindows(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:fits-system-windows".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "getFitsSystemWindows()Z") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:fits-system-windows")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "postOnAnimation(Ljava/lang/Runnable;)V") => {
                self.view_mut(receiver)?;
                self.heap.get(arg(1)?)?;
            }
            ("Landroid/view/ViewTreeObserver;", "addOnPreDrawListener(Landroid/view/ViewTreeObserver$OnPreDrawListener;)V")
            | ("Landroid/view/ViewTreeObserver;", "removeOnPreDrawListener(Landroid/view/ViewTreeObserver$OnPreDrawListener;)V") => {
                self.heap.get(receiver)?;
                self.heap.get(arg(1)?)?;
            }
            ("Landroid/view/ViewTreeObserver;", "addOnGlobalLayoutListener(Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;)V") => {
                let listener = arg(1)?;
                self.heap.get(listener)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .entry("droidless:global-layout-listeners".into())
                    .or_default()
                    .push(listener);
            }
            ("Landroid/view/ViewTreeObserver;", "removeOnGlobalLayoutListener(Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;)V")
            | ("Landroid/view/ViewTreeObserver;", "removeGlobalOnLayoutListener(Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;)V") => {
                let listener = arg(1)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .entry("droidless:global-layout-listeners".into())
                    .or_default()
                    .retain(|registered| *registered != listener);
            }
            ("Landroid/view/ViewTreeObserver;", "isAlive()Z") => {
                self.heap.get(receiver)?;
                result.push(Word::from(1));
            }
            ("Landroid/view/ViewTreeObserver;", "dispatchOnPreDraw()Z") => {
                self.heap.get(receiver)?;
                result.push(Word::from(1));
            }
            ("Landroid/view/View;", "computeFitSystemWindows(Landroid/graphics/Rect;Landroid/graphics/Rect;)V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/view/View;", "makeOptionalFitsSystemWindows()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/view/View;", "setOnKeyListener(Landroid/view/View$OnKeyListener;)V") => {
                let listener = arg(1)?;
                self.view_mut(receiver)?.key_listener = if listener == Word::ZERO {
                    None
                } else {
                    self.heap.get(listener)?;
                    Some(listener)
                };
            }
            ("Landroid/view/View;", "setId(I)V") => {
                self.view_mut(receiver)?.id = arg(1)?.int()? as u32
            }
            ("Landroid/view/View;", "getId()I") => {
                result.push(Word::Bits(self.view_mut(receiver)?.id))
            }
            ("Landroid/view/View;", "getVisibility()I") => {
                result.push(Word::from(self.view_mut(receiver)?.visible));
            }
            ("Landroid/view/View;", "isInEditMode()Z") => {
                self.view_mut(receiver)?;
                result.push(Word::ZERO);
            }
            ("Landroid/view/View;", "getOverScrollMode()I") => {
                let mode = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:over-scroll-mode")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::from(1));
                result.push(mode);
            }
            ("Landroid/view/View;", "getLayoutDirection()I") => {
                self.view_mut(receiver)?;
                result.push(Word::ZERO);
            }
            ("Landroid/view/View;", "getScrollX()I")
            | ("Landroid/view/View;", "getScrollY()I") => {
                self.view_mut(receiver)?;
                let axis = if method.name == "getScrollX" { "x" } else { "y" };
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(&format!("droidless:view:scroll-{axis}"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "getWidth()I")
            | ("Landroid/view/View;", "getHeight()I") => {
                self.view_mut(receiver)?;
                let field = if method.name == "getWidth" {
                    "droidless:view:right"
                } else {
                    "droidless:view:bottom"
                };
                let start = if method.name == "getWidth" {
                    "droidless:view:left"
                } else {
                    "droidless:view:top"
                };
                let end = self.heap.get(receiver)?.fields.get(field)
                    .and_then(|values| values.first()).copied().unwrap_or(Word::ZERO).int()?;
                let start = self.heap.get(receiver)?.fields.get(start)
                    .and_then(|values| values.first()).copied().unwrap_or(Word::ZERO).int()?;
                result.push(Word::from((end - start).max(0)));
            }
            ("Landroid/view/View;", "getMeasuredWidth()I")
            | ("Landroid/view/View;", "getMeasuredHeight()I") => {
                self.view_mut(receiver)?;
                let edge = if method.name == "getMeasuredWidth" {
                    "width"
                } else {
                    "height"
                };
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(&format!("droidless:view:measured-{edge}"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "getMinimumWidth()I")
            | ("Landroid/view/View;", "getMinimumHeight()I") => {
                self.view_mut(receiver)?;
                let edge = if method.name == "getMinimumWidth" {
                    "width"
                } else {
                    "height"
                };
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(&format!("droidless:view:minimum-{edge}"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "setMinimumWidth(I)V")
            | ("Landroid/view/View;", "setMinimumHeight(I)V") => {
                self.view_mut(receiver)?;
                let edge = if method.name == "setMinimumWidth" {
                    "width"
                } else {
                    "height"
                };
                self.heap.get_mut(receiver)?.fields.insert(
                    format!("droidless:view:minimum-{edge}"),
                    vec![Word::from(arg(1)?.int()?.max(0))],
                );
            }
            ("Landroid/view/View;", "getLeft()I")
            | ("Landroid/view/View;", "getTop()I")
            | ("Landroid/view/View;", "getRight()I")
            | ("Landroid/view/View;", "getBottom()I") => {
                self.view_mut(receiver)?;
                let edge = method.name.strip_prefix("get").context("invalid View edge getter")?.to_ascii_lowercase();
                let value = self.heap.get(receiver)?.fields.get(&format!("droidless:view:{edge}"))
                    .and_then(|values| values.first()).copied().unwrap_or(Word::ZERO);
                result.push(value);
            }
            ("Landroid/view/View;", "isLaidOut()Z") => {
                self.view_mut(receiver)?;
                result.push(Word::from(i32::from(self.heap.get(receiver)?.fields
                    .get("droidless:view:laid-out").and_then(|values| values.first())
                    .is_some_and(|value| value.truth()))));
            }
            ("Landroid/view/View;", "isLayoutRequested()Z") => {
                self.view_mut(receiver)?;
                let fields = &self.heap.get(receiver)?.fields;
                result.push(Word::from(i32::from(
                    fields
                        .get("droidless:view:layout-requested")
                        .and_then(|values| values.first())
                        .copied()
                        .map(Word::truth)
                        .unwrap_or_else(|| {
                            !fields
                                .get("droidless:view:laid-out")
                                .and_then(|values| values.first())
                                .is_some_and(|value| value.truth())
                        }),
                )));
            }
            ("Landroid/view/View;", "measure(II)V") => {
                let specs = [arg(1)?, arg(2)?];
                self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "onMeasure".into(),
                        parameters: vec!["I".into(), "I".into()],
                        returns: "V".into(),
                    },
                    std::iter::once(receiver).chain(specs).collect(),
                    true,
                )?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:layout-requested".into(), vec![Word::ZERO]);
            }
            ("Landroid/view/View;", "onMeasure(II)V") => {
                self.view_mut(receiver)?;
                for (spec, horizontal, edge) in [(arg(1)?, true, "width"), (arg(2)?, false, "height")] {
                    let spec = spec.int()? as u32;
                    let mode = spec & 0xc000_0000;
                    let size = (spec & 0x3fff_ffff) as i32;
                    let minimum = self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(&format!("droidless:view:minimum-{edge}"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()?;
                    let desired = (crate::ui::dimension(
                        &self.heap,
                        receiver,
                        horizontal,
                        size as f32,
                    )? as i32)
                        .max(minimum);
                    let measured = match mode {
                        0x4000_0000 => size,
                        0x8000_0000 => desired.min(size),
                        _ => desired,
                    }.max(0);
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("droidless:view:measured-{edge}"),
                        vec![Word::from(measured)],
                    );
                }
            }
            ("Landroid/view/View;", "setMeasuredDimension(II)V") => {
                self.view_mut(receiver)?;
                for (edge, value) in [("width", arg(1)?), ("height", arg(2)?)] {
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("droidless:view:measured-{edge}"),
                        vec![Word::from(value.int()?.max(0))],
                    );
                }
            }
            ("Landroid/view/View;", "layout(IIII)V") => {
                let bounds = [arg(1)?, arg(2)?, arg(3)?, arg(4)?];
                let names = ["left", "top", "right", "bottom"];
                let changed = {
                    let object = self.heap.get_mut(receiver)?;
                    let changed = names.iter().zip(bounds).any(|(name, value)| {
                        object.fields.get(&format!("droidless:view:{name}"))
                            .and_then(|values| values.first()).copied().unwrap_or(Word::ZERO) != value
                    });
                    for (name, value) in names.into_iter().zip(bounds) {
                        object.fields.insert(format!("droidless:view:{name}"), vec![value]);
                    }
                    object.fields.insert("droidless:view:laid-out".into(), vec![Word::from(1)]);
                    object.fields.insert("droidless:view:layout-requested".into(), vec![Word::ZERO]);
                    changed
                };
                self.invoke(Method {
                    class: "Landroid/view/View;".into(),
                    name: "onLayout".into(),
                    parameters: vec!["Z".into(), "I".into(), "I".into(), "I".into(), "I".into()],
                    returns: "V".into(),
                }, std::iter::once(receiver)
                    .chain([Word::from(i32::from(changed))])
                    .chain(bounds)
                    .collect(), true)?;
            }
            ("Landroid/view/View;", "offsetLeftAndRight(I)V")
            | ("Landroid/view/View;", "offsetTopAndBottom(I)V") => {
                let (start, end) = if method.name == "offsetLeftAndRight" {
                    ("left", "right")
                } else {
                    ("top", "bottom")
                };
                let offset = arg(1)?.int()?;
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for edge in [start, end] {
                    let key = format!("droidless:view:{edge}");
                    let current = fields
                        .get(&key)
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()?;
                    fields.insert(key, vec![Word::from(current.saturating_add(offset))]);
                }
            }
            ("Landroid/view/View;", "onLayout(ZIIII)V")
            | ("Landroid/view/ViewGroup;", "onLayout(ZIIII)V") => {
                self.view_mut(receiver)?;
            }
            ("Landroid/view/View;", "getParent()Landroid/view/ViewParent;") => {
                let parent = self.heap.get(receiver)?.fields.get("droidless:view:parent").and_then(|values| values.first()).copied().unwrap_or(Word::ZERO);
                result.push(parent);
            }
            ("Landroid/view/View;", "setTag(Ljava/lang/Object;)V") => {
                let tag = arg(1)?;
                if tag != Word::ZERO { self.heap.get(tag)?; }
                self.heap.get_mut(receiver)?.fields.insert("droidless:view:tag".into(), vec![tag]);
            }
            ("Landroid/view/View;", "getTag()Ljava/lang/Object;") => {
                let tag = self.heap.get(receiver)?.fields.get("droidless:view:tag").and_then(|values| values.first()).copied().unwrap_or(Word::ZERO);
                result.push(tag);
            }
            ("Landroid/view/View;", "setTag(ILjava/lang/Object;)V") => {
                let tag = arg(2)?;
                if tag != Word::ZERO { self.heap.get(tag)?; }
                self.heap.get_mut(receiver)?.fields.insert(format!("droidless:view:tag:{}", arg(1)?.int()?), vec![tag]);
            }
            ("Landroid/view/View;", "getTag(I)Ljava/lang/Object;") => {
                let tag = self.heap.get(receiver)?.fields.get(&format!("droidless:view:tag:{}", arg(1)?.int()?)).and_then(|values| values.first()).copied().unwrap_or(Word::ZERO);
                result.push(tag);
            }
            ("Landroid/view/View;", "post(Ljava/lang/Runnable;)Z")
            | ("Landroid/view/View;", "postDelayed(Ljava/lang/Runnable;J)Z") => {
                let handler = if let Some(handler) = self.heap.get(receiver)?.fields.get("droidless:view:handler").and_then(|values| values.first()).copied() {
                    handler
                } else {
                    let handler = self.heap.instance("Landroid/os/Handler;")?;
                    self.invoke(Method { class: "Landroid/os/Handler;".into(), name: "<init>".into(), parameters: vec![], returns: "V".into() }, vec![handler], false)?;
                    self.heap.get_mut(receiver)?.fields.insert("droidless:view:handler".into(), vec![handler]);
                    handler
                };
                let mut forwarded = vec![handler, arg(1)?];
                if method.name == "postDelayed" { forwarded.extend_from_slice(args.get(2..4).context("missing postDelayed interval")?); }
                let posted = self.invoke(
                    Method { class: "Landroid/os/Handler;".into(), name: method.name.clone(), parameters: method.parameters.clone(), returns: "Z".into() },
                    forwarded,
                    false,
                )?;
                result.extend(posted);
            }
            ("Landroid/view/View;", "removeCallbacks(Ljava/lang/Runnable;)Z") => {
                let handler = self.heap.get(receiver)?.fields.get("droidless:view:handler").and_then(|values| values.first()).copied();
                let mut removed = false;
                if let Some(handler) = handler {
                    let found = self.invoke(Method { class: "Landroid/os/Handler;".into(), name: "hasCallbacks".into(), parameters: vec!["Ljava/lang/Runnable;".into()], returns: "Z".into() }, vec![handler, arg(1)?], false)?;
                    removed = found.first().is_some_and(|value| value.truth());
                    self.invoke(Method { class: "Landroid/os/Handler;".into(), name: "removeCallbacks".into(), parameters: vec!["Ljava/lang/Runnable;".into()], returns: "V".into() }, vec![handler, arg(1)?], false)?;
                }
                result.push(Word::from(i32::from(removed)));
            }
            ("Landroid/view/View;", "setOverScrollMode(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:over-scroll-mode".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "requestLayout()V")
            | ("Landroid/support/v7/widget/ContentFrameLayout;", "requestLayout()V") => {
                self.view_mut(receiver)?;
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:layout-requested".into(),
                    vec![Word::from(1)],
                );
            }
            ("Landroid/view/View;", "getPaddingLeft()I")
            | ("Landroid/view/View;", "getPaddingTop()I")
            | ("Landroid/view/View;", "getPaddingRight()I")
            | ("Landroid/view/View;", "getPaddingBottom()I") => {
                result.push(Word::from(self.view_mut(receiver)?.padding as i32));
            }
            ("Landroid/view/View;", "setVisibility(I)V") => {
                let v = arg(1)?.int()?;
                ensure!([0, 4, 8].contains(&v), "invalid View visibility");
                self.view_mut(receiver)?.visible = v;
            }
            ("Landroid/view/View;", "setEnabled(Z)V") => {
                self.view_mut(receiver)?.enabled = arg(1)?.int()? != 0
            }
            ("Landroid/view/View;", "setSaveFromParentEnabled(Z)V") => {
                self.heap.get_mut(receiver)?.fields.insert("droidless:view:save-from-parent".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setWillNotDraw(Z)V")
            | ("Landroid/support/v7/widget/ViewStubCompat;", "setWillNotDraw(Z)V") => {
                self.view_mut(receiver)?;
            }
            ("Landroid/view/View;", "setDescendantFocusability(I)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setDescendantFocusability(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:descendant-focusability".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setFocusableInTouchMode(Z)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setFocusableInTouchMode(Z)V")
            | ("Landroid/view/View;", "setFocusable(Z)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setFocusable(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("droidless:{}", method.name), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setImportantForAccessibility(I)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setImportantForAccessibility(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:important-for-accessibility".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "getImportantForAccessibility()I") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:important-for-accessibility")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "setAccessibilityDelegate(Landroid/view/View$AccessibilityDelegate;)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setAccessibilityDelegate(Landroid/view/View$AccessibilityDelegate;)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:accessibility-delegate".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setBackgroundColor(I)V") => {
                self.view_mut(receiver)?.background = Some(arg(1)?.int()? as u32)
            }
            ("Landroid/view/View;", "getBackground()Landroid/graphics/drawable/Drawable;")
            | ("Landroid/support/v7/widget/bs;", "getBackground()Landroid/graphics/drawable/Drawable;") => {
                let drawable = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:background-drawable")
                    .and_then(|values| values.first())
                    .copied();
                let drawable = if let Some(drawable) = drawable {
                    drawable
                } else if let Some(color) = self.view_mut(receiver)?.background {
                    let drawable = self.heap.instance("Landroid/graphics/drawable/ColorDrawable;")?;
                    self.heap.get_mut(drawable)?.fields.insert("color".into(), vec![Word::from(color as i32)]);
                    self.heap.get_mut(receiver)?.fields.insert(
                        "droidless:view:background-drawable".into(),
                        vec![drawable],
                    );
                    drawable
                } else {
                    Word::ZERO
                };
                result.push(drawable);
            }
            ("Landroid/view/View;", "setBackgroundDrawable(Landroid/graphics/drawable/Drawable;)V")
            | ("Landroid/support/v7/widget/bs;", "setBackgroundDrawable(Landroid/graphics/drawable/Drawable;)V") => {
                let drawable = arg(1)?;
                let color = if drawable == Word::ZERO {
                    None
                } else if let Some(id) = self
                    .heap
                    .get(drawable)?
                    .fields
                    .get("resourceId")
                    .and_then(|values| values.first())
                {
                    self.apk
                        .resources
                        .resolve(id.int()? as u32)
                        .ok()
                        .and_then(|value| self.drawable_color(value, 0).ok().flatten())
                } else if self.heap.get(drawable)?.class == "Landroid/graphics/drawable/ColorDrawable;" {
                    self.heap
                        .get(drawable)?
                        .fields
                        .get("color")
                        .and_then(|values| values.first())
                        .map(|value| value.int().map(|color| color as u32))
                        .transpose()?
                } else {
                    None
                };
                self.view_mut(receiver)?.background = color;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:background-drawable".into(), vec![drawable]);
            }
            ("Landroid/view/View;", "setElevation(F)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:elevation".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setStateListAnimator(Landroid/animation/StateListAnimator;)V") => {
                let animator = arg(1)?;
                if animator != Word::ZERO { self.heap.get(animator)?; }
                self.heap.get_mut(receiver)?.fields.insert("droidless:view:state-list-animator".into(), vec![animator]);
            }
            ("Landroid/view/View;", "setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V") => {
                let params = arg(1)?;
                if params != Word::ZERO { self.heap.get(params)?; }
                self.heap.get_mut(receiver)?.fields.insert("droidless:view:layout-params".into(), vec![params]);
            }
            ("Landroid/view/View;", "getLayoutParams()Landroid/view/ViewGroup$LayoutParams;") => {
                let params = self.heap.get(receiver)?.fields.get("droidless:view:layout-params").and_then(|values| values.first()).copied().unwrap_or(Word::ZERO);
                result.push(params);
            }
            ("Landroid/view/View;", "setOutlineProvider(Landroid/view/ViewOutlineProvider;)V") => {
                let provider = arg(1)?;
                if provider != Word::ZERO {
                    self.heap.get(provider)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:outline-provider".into(), vec![provider]);
            }
            ("Landroid/view/View;", "setClipToOutline(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:clip-to-outline".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "startAnimation(Landroid/view/animation/Animation;)V") => {
                let animation = arg(1)?;
                if animation != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(animation)?.class, "Landroid/view/animation/Animation;"),
                        "View animation has the wrong type"
                    );
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:animation".into(), vec![animation]);
            }
            ("Landroid/view/View;", "getAnimation()Landroid/view/animation/Animation;") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:animation")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "clearAnimation()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .remove("droidless:view:animation");
            }
            ("Landroid/view/View;", "setScrollContainer(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:scroll-container".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setPadding(IIII)V") => {
                let values = args[1..]
                    .iter()
                    .map(|w| w.int())
                    .collect::<Result<Vec<_>>>()?;
                ensure!(
                    values.len() == 4
                        && values.iter().all(|n| *n >= 0)
                        && values.iter().all(|n| *n == values[0]),
                    "only uniform non-negative padding supported"
                );
                self.view_mut(receiver)?.padding = values[0] as f32;
            }
            ("Landroid/util/Log;", sig)
                if [
                    "d(Ljava/lang/String;Ljava/lang/String;)I",
                    "i(Ljava/lang/String;Ljava/lang/String;)I",
                    "w(Ljava/lang/String;Ljava/lang/String;)I",
                    "e(Ljava/lang/String;Ljava/lang/String;)I",
                ]
                .contains(&sig) =>
            {
                eprintln!(
                    "{}/{}: {}",
                    method.name.to_uppercase(),
                    self.heap.text(arg(0)?)?,
                    self.heap.text(arg(1)?)?
                );
                result.push(Word::ZERO);
            }
            ("Landroid/text/TextUtils;", "isEmpty(Ljava/lang/CharSequence;)Z") => {
                result.push(Word::from(i32::from(
                    arg(0)? == Word::ZERO || self.heap.text(arg(0)?)?.is_empty(),
                )));
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
    fn view_mut(&mut self, word: Word) -> Result<&mut crate::ui::View> {
        self.heap
            .get_mut(word)?
            .view
            .as_mut()
            .context("framework method expects a View")
    }
    fn hierarchy_change(&mut self, parent: Word, child: Word, added: bool) -> Result<()> {
        let listener = self
            .heap
            .get(parent)?
            .fields
            .get("droidless:view:hierarchy-listener")
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(Word::ZERO);
        if listener != Word::ZERO {
            self.invoke(
                Method {
                    class: "Landroid/view/ViewGroup$OnHierarchyChangeListener;".into(),
                    name: if added {
                        "onChildViewAdded"
                    } else {
                        "onChildViewRemoved"
                    }
                    .into(),
                    parameters: vec!["Landroid/view/View;".into(); 2],
                    returns: "V".into(),
                },
                vec![listener, parent, child],
                true,
            )?;
        }
        Ok(())
    }
    pub(crate) fn resource_text(&self, id: u32) -> Result<String> {
        // Android's stable public framework string IDs, not application-specific output.
        match id {
            0x0104000a => Ok("OK".into()),
            0x01040000 => Ok("Cancel".into()),
            _ => self.apk.resources.text(id),
        }
    }
    fn encoded_image(bytes: Vec<u8>) -> Option<(crate::graphics::ImageInfo, Vec<u8>)> {
        crate::graphics::inspect(&bytes)
            .ok()
            .flatten()
            .map(|info| (info, bytes))
    }
    fn image_resource(&self, id: u32) -> Result<Option<(crate::graphics::ImageInfo, Vec<u8>)>> {
        let value = match self.apk.resources.resolve(id) {
            Ok(value) => value,
            Err(_) if id >> 24 == 1 => return Ok(None),
            Err(error) => return Err(error),
        };
        if value.kind != 3 {
            return Ok(None);
        }
        let name = value.display();
        let bytes = self
            .apk
            .files
            .get(&name)
            .with_context(|| format!("image resource {name} missing"))?;
        Ok(Self::encoded_image(bytes.clone()))
    }
    fn bitmap(
        &mut self,
        info: crate::graphics::ImageInfo,
        bytes: Vec<u8>,
        options: Option<Word>,
    ) -> Result<Word> {
        let mut sample = 1;
        if let Some(options) = options.filter(|value| *value != Word::ZERO) {
            ensure!(
                self.is_a(
                    &self.heap.get(options)?.class,
                    "Landroid/graphics/BitmapFactory$Options;"
                ),
                "BitmapFactory options type mismatch"
            );
            let key = "Landroid/graphics/BitmapFactory$Options;->inJustDecodeBounds:Z";
            let just_bounds = self
                .heap
                .get(options)?
                .fields
                .get(key)
                .and_then(|values| values.first())
                .is_some_and(|value| value.truth());
            let sample_key = "Landroid/graphics/BitmapFactory$Options;->inSampleSize:I";
            sample = self
                .heap
                .get(options)?
                .fields
                .get(sample_key)
                .and_then(|values| values.first())
                .map_or(1, |value| value.int().unwrap_or(1))
                .max(1) as u32;
            let mime = self.heap.string(info.mime.into())?;
            let fields = &mut self.heap.get_mut(options)?.fields;
            fields.insert(
                "Landroid/graphics/BitmapFactory$Options;->outWidth:I".into(),
                vec![Word::from(info.width as i32)],
            );
            fields.insert(
                "Landroid/graphics/BitmapFactory$Options;->outHeight:I".into(),
                vec![Word::from(info.height as i32)],
            );
            fields.insert(
                "Landroid/graphics/BitmapFactory$Options;->outMimeType:Ljava/lang/String;".into(),
                vec![mime],
            );
            if just_bounds {
                return Ok(Word::ZERO);
            }
        }
        let bitmap = self.heap.instance("Landroid/graphics/Bitmap;")?;
        self.heap.get_mut(bitmap)?.data = Data::Bitmap {
            bytes,
            width: (info.width / sample).max(1),
            height: (info.height / sample).max(1),
            mime: info.mime,
            recycled: false,
        };
        Ok(bitmap)
    }
    fn attribute_set_value(&self, attrs: Word, name: &str) -> Result<Option<Value>> {
        match &self.heap.get(attrs)?.data {
            Data::Attributes(attributes) => Ok(attributes.get(name).cloned()),
            Data::XmlPull {
                events, position, ..
            } => {
                let event = events
                    .get(*position)
                    .context("invalid XML parser position")?;
                Ok(event
                    .attributes
                    .iter()
                    .find(|attribute| attribute.name == name)
                    .map(|attribute| attribute.value.clone()))
            }
            _ => bail!("uninitialized AttributeSet"),
        }
    }
    fn find_view(&self, word: Word, id: u32, depth: usize) -> Result<Option<Word>> {
        ensure!(depth < 128, "View search nesting limit");
        let view = self
            .heap
            .get(word)?
            .view
            .as_ref()
            .context("expected View")?;
        if view.id == id {
            return Ok(Some(word));
        }
        for child in &view.children {
            if let Some(v) = self.find_view(*child, id, depth + 1)? {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }
    fn install_android_content_id(&mut self, word: Word, depth: usize) -> Result<bool> {
        ensure!(depth < 128, "content view nesting limit");
        if self.heap.get(word)?.class == "Landroid/support/v7/widget/ContentFrameLayout;" {
            self.view_mut(word)?.id = 0x0102_0002;
            return Ok(true);
        }
        let children = self
            .heap
            .get(word)?
            .view
            .as_ref()
            .context("expected a View")?
            .children
            .clone();
        for child in children {
            if self.install_android_content_id(child, depth + 1)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn inflate_id(&mut self, id: u32, depth: usize, context: Word) -> Result<Word> {
        ensure!(depth < 64, "layout inflation nesting limit");
        let name = self.apk.resources.text(id)?;
        let data = self
            .apk
            .files
            .get(&name)
            .with_context(|| format!("layout file {name} missing"))?;
        let element = droidless_formats::xml::parse(data)?;
        self.inflate(&element, depth + 1, context)
    }
    fn construct_inflated_view(
        &mut self,
        view: Word,
        class: &str,
        context: Word,
        element: &Element,
    ) -> Result<()> {
        let Some((dex, index)) = self.class_location(class) else {
            return Ok(());
        };
        let definition = &self.apk.dex[dex].classes[index];
        let context_type = "Landroid/content/Context;";
        let attributes_type = "Landroid/util/AttributeSet;";
        let constructor = [
            vec![context_type, attributes_type],
            vec![context_type, attributes_type, "I"],
            vec![context_type],
        ]
        .into_iter()
        .find_map(|parameters| {
            definition
                .methods
                .iter()
                .map(|encoded| &self.apk.dex[dex].methods[encoded.index])
                .find(|method| method.name == "<init>" && method.parameters == parameters)
                .cloned()
        });
        let Some(constructor) = constructor else {
            return Ok(());
        };
        let mut args = vec![view, context];
        if constructor
            .parameters
            .iter()
            .any(|ty| ty == attributes_type)
        {
            let attrs = self.heap.instance(attributes_type)?;
            self.heap.get_mut(attrs)?.data = Data::Attributes(element.attributes.clone());
            args.push(attrs);
        }
        if constructor.parameters.last().is_some_and(|ty| ty == "I") {
            args.push(Word::ZERO);
        }
        self.invoke(constructor, args, false)?;
        Ok(())
    }
    fn attribute(&self, value: &Value) -> Result<Value> {
        if value.kind == 1 {
            if value.data == 0 {
                Ok(Value {
                    kind: 0x1f,
                    data: 0,
                    text: None,
                })
            } else {
                Ok(self.apk.resources.resolve(value.data)?.clone())
            }
        } else {
            Ok(value.clone())
        }
    }
    fn inflate(&mut self, element: &Element, depth: usize, context: Word) -> Result<Word> {
        ensure!(depth < 64, "layout XML nesting limit");
        if element.name == "merge" {
            let container = self.new_instance("Landroid/widget/FrameLayout;")?;
            let mut view = self
                .heap
                .get(container)?
                .view
                .clone()
                .context("FrameLayout is not a ViewGroup")?;
            for child in &element.children {
                let child = self.inflate(child, depth + 1, context)?;
                self.heap
                    .get_mut(child)?
                    .fields
                    .insert("droidless:view:parent".into(), vec![container]);
                view.children.push(child);
            }
            self.heap.get_mut(container)?.view = Some(view);
            return Ok(container);
        }
        if element.name == "include" {
            return self.inflate_id(
                element.number("layout").context("include missing layout")?,
                depth + 1,
                context,
            );
        }
        let class = if element.name.contains('.') {
            crate::vm::descriptor(&element.name)
        } else {
            format!("Landroid/widget/{};", element.name)
        };
        let word = self.new_instance(&class)?;
        self.construct_inflated_view(word, &class, context, element)?;
        let mut view = self
            .heap
            .get(word)?
            .view
            .clone()
            .with_context(|| format!("unsupported layout View {}", element.name))?;
        view.id = element.number("id").unwrap_or(0);
        for (name, raw) in &element.attributes {
            match name.as_str() {
                "text" => {
                    view.text = if raw.kind == 1 {
                        self.resource_text(raw.data)?
                    } else {
                        raw.display()
                    }
                }
                "title" if class == "Landroid/support/v7/widget/Toolbar;" => {
                    view.text = if raw.kind == 1 {
                        self.resource_text(raw.data)?
                    } else {
                        raw.display()
                    }
                }
                "layout_width" => view.width = dimension(&self.attribute(raw)?)?,
                "layout_height" => view.height = dimension(&self.attribute(raw)?)?,
                "layout_weight" => {
                    view.weight = if raw.kind == 4 {
                        f32::from_bits(raw.data)
                    } else {
                        raw.data as f32
                    }
                }
                "orientation" => view.orientation = raw.data as i32,
                "textSize" => view.text_size = dimension(&self.attribute(raw)?)?,
                "textColor" => {
                    let v = self.attribute(raw)?;
                    if (0x1c..=0x1f).contains(&v.kind) {
                        view.text_color = v.data;
                    } else {
                        bail!("unsupported text color state list");
                    }
                }
                "src" | "srcCompat" if raw.kind == 1 => {
                    let id = raw.data;
                    let image = self.image_resource(id)?;
                    let drawable = self.heap.instance("Landroid/graphics/drawable/Drawable;")?;
                    self.heap
                        .get_mut(drawable)?
                        .fields
                        .insert("resourceId".into(), vec![Word::from(id as i32)]);
                    if let Some((info, bytes)) = image {
                        let bitmap = self.bitmap(info, bytes.clone(), None)?;
                        self.heap
                            .get_mut(drawable)?
                            .fields
                            .insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                        view.image = Some(bytes);
                    }
                    self.heap
                        .get_mut(word)?
                        .fields
                        .insert("droidless:image:drawable".into(), vec![drawable]);
                }
                "background" => view.background = self.drawable_color(raw, 0)?,
                "padding" => view.padding = dimension(&self.attribute(raw)?)?,
                "layout_margin" => view.margins = [dimension(&self.attribute(raw)?)?; 4],
                "layout_marginLeft"
                | "layout_marginTop"
                | "layout_marginRight"
                | "layout_marginBottom" => {
                    let i = match name.as_str() {
                        "layout_marginLeft" => 0,
                        "layout_marginTop" => 1,
                        "layout_marginRight" => 2,
                        _ => 3,
                    };
                    view.margins[i] = dimension(&self.attribute(raw)?)?;
                }
                "gravity" => view.gravity = raw.data,
                "onClick" => view.xml_click = Some(raw.display()),
                "visibility" => view.visible = raw.data as i32,
                "enabled" => view.enabled = raw.data != 0,
                "inputType" if raw.data == 0 => {
                    view.editable = false;
                }
                _ => {} // Styling outside this subset is documented; execution APIs still fail explicitly.
            }
        }
        ensure!(
            view.weight.is_finite()
                && view.weight >= 0.0
                && view.text_size.is_finite()
                && view.text_size >= 0.0,
            "invalid layout numeric attribute"
        );
        for child in &element.children {
            let child = self.inflate(child, depth + 1, context)?;
            self.heap
                .get_mut(child)?
                .fields
                .insert("droidless:view:parent".into(), vec![word]);
            view.children.push(child);
        }
        let children = view.children.clone();
        self.heap.get_mut(word)?.view = Some(view);
        for child in children {
            self.hierarchy_change(word, child, true)?;
        }
        Ok(word)
    }
    fn drawable_color(&self, raw: &Value, depth: usize) -> Result<Option<u32>> {
        ensure!(depth < 32, "drawable reference nesting limit");
        let value = self.attribute(raw)?;
        if (0x1c..=0x1f).contains(&value.kind) {
            return Ok(Some(value.data));
        }
        if value.kind == 3 {
            let name = value.display();
            if name.ends_with(".xml") {
                let data = self.apk.files.get(&name).context("drawable XML missing")?;
                let element = droidless_formats::xml::parse(data)?;
                fn color(node: &Element) -> Option<&Value> {
                    if node.name == "solid" {
                        node.attr("color")
                    } else if node.name == "item"
                        && !node.attributes.contains_key("state_pressed")
                        && !node.attributes.contains_key("state_focused")
                    {
                        node.attr("drawable")
                            .or_else(|| node.children.iter().find_map(color))
                    } else {
                        node.children.iter().find_map(color)
                    }
                }
                if let Some(c) = color(&element) {
                    return self.drawable_color(c, depth + 1);
                }
            }
        }
        Ok(None)
    }
}
fn resource_bundle_object(runtime: &mut Runtime, bundle: Word, key: Word) -> Result<Option<Word>> {
    let contents = runtime
        .invoke(
            Method {
                class: "Ljava/util/ListResourceBundle;".into(),
                name: "getContents".into(),
                parameters: vec![],
                returns: "[[Ljava/lang/Object;".into(),
            },
            vec![bundle],
            true,
        )?
        .into_iter()
        .next()
        .context("ResourceBundle contents missing")?;
    let Data::Array { values: rows, .. } = &runtime.heap.get(contents)?.data else {
        bail!("ResourceBundle contents are not an array");
    };
    for row in rows {
        let row = row.first().copied().context("empty ResourceBundle entry")?;
        let Data::Array { values: pair, .. } = &runtime.heap.get(row)?.data else {
            bail!("ResourceBundle entry is not an array");
        };
        let entry_key = pair
            .first()
            .and_then(|words| words.first())
            .copied()
            .context("ResourceBundle entry key missing")?;
        if runtime.heap.text(entry_key)? == runtime.heap.text(key)? {
            return Ok(Some(
                pair.get(1)
                    .and_then(|words| words.first())
                    .copied()
                    .context("ResourceBundle entry value missing")?,
            ));
        }
    }
    Ok(None)
}
fn dimension(v: &Value) -> Result<f32> {
    let n = match v.kind {
        2 => -2.0, // ponytail: unresolved theme dimensions use wrap-content until theme attribute resolution exists.
        0x10 | 0x11 => v.data as i32 as f32,
        4 => f32::from_bits(v.data),
        5 => {
            let radix = (v.data >> 4) & 3;
            let unit = v.data & 15;
            ensure!(unit <= 2, "only px/dp/sp dimensions supported");
            (v.data & 0xffffff00) as i32 as f32
                * [
                    1.0 / 256.0,
                    1.0 / 32768.0,
                    1.0 / 8388608.0,
                    1.0 / 2147483648.0,
                ][radix as usize]
        }
        _ => bail!("unsupported layout dimension type {}", v.kind),
    };
    ensure!(n.is_finite() && n >= -2.0, "invalid layout dimension");
    Ok(n)
}
fn rgb_to_hsv(color: u32) -> [f32; 3] {
    let [r, g, b] = [
        ((color >> 16) & 0xff) as f32 / 255.0,
        ((color >> 8) & 0xff) as f32 / 255.0,
        (color & 0xff) as f32 / 255.0,
    ];
    let maximum = r.max(g).max(b);
    let minimum = r.min(g).min(b);
    let delta = maximum - minimum;
    let hue = if delta == 0.0 {
        0.0
    } else if maximum == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if maximum == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    [
        hue,
        if maximum == 0.0 { 0.0 } else { delta / maximum },
        maximum,
    ]
}
fn java_double(v: f64) -> String {
    if v.is_nan() {
        "NaN".into()
    } else if v == f64::INFINITY {
        "Infinity".into()
    } else if v == f64::NEG_INFINITY {
        "-Infinity".into()
    } else {
        let s = v.to_string();
        if s.contains('.') || s.contains('e') {
            s
        } else {
            format!("{s}.0")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rgb_to_hsv;

    #[test]
    fn rgb_to_hsv_handles_primary_and_gray_colors() {
        assert_eq!(rgb_to_hsv(0xffff_0000), [0.0, 1.0, 1.0]);
        assert_eq!(rgb_to_hsv(0xff00_ff00), [120.0, 1.0, 1.0]);
        assert_eq!(rgb_to_hsv(0xff00_00ff), [240.0, 1.0, 1.0]);
        assert_eq!(rgb_to_hsv(0xff80_8080), [0.0, 0.0, 128.0 / 255.0]);
    }
}
