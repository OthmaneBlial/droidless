use crate::{
    heap::{Data, TimeUnit, Word, bits64, exception_parent, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{
    dex::{Field, Method},
    xml::{Element, Value},
};

pub(crate) const OBSERVERS: &str =
    "Landroid/database/Observable;->mObservers:Ljava/util/ArrayList;";

pub(crate) fn graphics_enum_names(class: &str) -> Option<&'static [&'static str]> {
    Some(match class {
        "Landroid/widget/ImageView$ScaleType;" => &[
            "MATRIX",
            "FIT_XY",
            "FIT_START",
            "FIT_CENTER",
            "FIT_END",
            "CENTER",
            "CENTER_CROP",
            "CENTER_INSIDE",
        ],
        "Landroid/graphics/PorterDuff$Mode;" => &[
            "CLEAR", "SRC", "DST", "SRC_OVER", "DST_OVER", "SRC_IN", "DST_IN", "SRC_OUT",
            "DST_OUT", "SRC_ATOP", "DST_ATOP", "XOR", "DARKEN", "LIGHTEN", "MULTIPLY", "SCREEN",
            "ADD", "OVERLAY",
        ],
        "Landroid/graphics/Paint$Cap;" => &["BUTT", "ROUND", "SQUARE"],
        "Landroid/graphics/Paint$Join;" => &["MITER", "ROUND", "BEVEL"],
        "Landroid/graphics/Paint$Style;" => &["FILL", "STROKE", "FILL_AND_STROKE"],
        "Landroid/graphics/Path$FillType;" => {
            &["WINDING", "EVEN_ODD", "INVERSE_WINDING", "INVERSE_EVEN_ODD"]
        }
        _ => return None,
    })
}

// Fixed virtual API branch profile, independent of the APK and host OS.
pub(crate) const SDK_INT: i32 = 21;

pub(crate) fn known_class(class: &str) -> bool {
    crate::ui::View::for_class(class).is_some()
        || exception_parent(class).is_some()
        || graphics_enum_names(class).is_some()
        || [
            "Ljava/lang/Object;",
            "Ljava/lang/Enum;",
            "Ljava/lang/StringBuilder;",
            "Ljava/lang/String;",
            "Ljava/lang/Class;",
            "Ljava/lang/ref/Reference;",
            "Ljava/lang/ref/WeakReference;",
            "Ljava/lang/reflect/Method;",
            "Ljava/lang/reflect/Constructor;",
            "Ljava/lang/reflect/AccessibleObject;",
            "Ljava/lang/Boolean;",
            "Ljava/lang/Double;",
            "Ljava/lang/Integer;",
            "Ljava/lang/Long;",
            "Ljava/lang/Number;",
            "Ljava/lang/Math;",
            "Landroid/text/Layout;",
            "Landroid/text/TextWatcher;",
            "Landroid/graphics/Typeface;",
            "Ljava/lang/Thread;",
            "Ljava/lang/ThreadLocal;",
            "Ljava/util/Date;",
            "Ljava/util/Timer;",
            "Ljava/util/TimerTask;",
            "Ljava/util/Locale;",
            "Ljava/util/ResourceBundle;",
            "Ljava/util/ListResourceBundle;",
            "Ljava/util/BitSet;",
            "Ljava/util/concurrent/Executors;",
            "Ljava/util/concurrent/Executor;",
            "Ljava/util/concurrent/ExecutorService;",
            "Ljava/util/concurrent/ThreadPoolExecutor;",
            "Ljava/util/concurrent/Callable;",
            "Ljava/util/concurrent/Future;",
            "Ljava/util/concurrent/RunnableFuture;",
            "Ljava/util/concurrent/FutureTask;",
            "Ljava/util/concurrent/ConcurrentHashMap;",
            "Landroid/animation/Animator;",
            "Landroid/animation/AnimatorListenerAdapter;",
            "Landroid/animation/Animator$AnimatorListener;",
            "Landroid/animation/Animator$AnimatorPauseListener;",
            "Landroid/animation/ObjectAnimator;",
            "Landroid/animation/ValueAnimator;",
            "Landroid/animation/ValueAnimator$AnimatorUpdateListener;",
            "Landroid/animation/TimeInterpolator;",
            "Landroid/view/ViewPropertyAnimator;",
            "Landroid/animation/StateListAnimator;",
            "Landroid/animation/LayoutTransition;",
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
            "Landroid/database/DataSetObservable;",
            "Landroid/database/DataSetObserver;",
            "Landroid/widget/BaseAdapter;",
            "Landroid/widget/Adapter;",
            "Landroid/widget/ListAdapter;",
            "Landroid/widget/SpinnerAdapter;",
            "Landroid/widget/AdapterView;",
            "Landroid/widget/AbsListView;",
            "Landroid/widget/AbsListView$LayoutParams;",
            "Landroid/widget/AdapterView$OnItemClickListener;",
            "Ldroidless/runtime/GridObserver;",
            "Ldroidless/runtime/GridClick;",
            "Landroid/content/ContentValues;",
            "Landroid/os/Handler;",
            "Landroid/os/Message;",
            "Landroid/os/Looper;",
            "Landroid/os/SystemClock;",
            "Landroid/os/Process;",
            "Landroid/os/Trace;",
            "Landroid/view/View$MeasureSpec;",
            "Landroid/view/Menu;",
            "Landroid/view/MenuItem;",
            "Landroid/view/MenuInflater;",
            "Landroid/view/MenuItem$OnMenuItemClickListener;",
            "Landroid/os/Build$VERSION;",
            "Landroid/net/LocalServerSocket;",
            "Landroid/net/LocalSocket;",
            "Ljava/util/HashSet;",
            "Ljava/util/Vector;",
            "Ljava/util/Stack;",
            "Ljava/util/TreeSet;",
            "Ljava/util/HashMap;",
            "Ljava/util/Hashtable;",
            "Ljava/util/Dictionary;",
            "Ljava/util/ArrayList;",
            "Ljava/util/LinkedHashMap;",
            "Ljava/util/WeakHashMap;",
            "Ljava/util/concurrent/LinkedBlockingQueue;",
            "Ljava/util/concurrent/CopyOnWriteArrayList;",
            "Ljava/lang/Throwable;",
            "Ljava/lang/Exception;",
            "Ljava/lang/RuntimeException;",
            "Landroid/app/Activity;",
            "Landroid/app/Fragment;",
            "Landroid/app/FragmentManager;",
            "Landroid/app/FragmentTransaction;",
            "Landroid/app/Application;",
            "Landroid/app/Application$ActivityLifecycleCallbacks;",
            "Landroid/content/res/Resources;",
            "Landroid/content/res/XmlResourceParser;",
            "Landroid/content/res/AssetManager;",
            "Landroid/content/res/Resources$Theme;",
            "Landroid/content/res/Configuration;",
            "Landroid/content/ContextWrapper;",
            "Landroid/view/ContextThemeWrapper;",
            "Landroid/app/Dialog;",
            "Landroid/app/Dialog$ListenersHandler;",
            "Landroid/content/DialogInterface;",
            "Landroid/content/DialogInterface$OnCancelListener;",
            "Landroid/content/DialogInterface$OnDismissListener;",
            "Landroid/content/DialogInterface$OnShowListener;",
            "Landroid/content/DialogInterface$OnKeyListener;",
            "Landroid/content/res/TypedArray;",
            "Landroid/util/TypedValue;",
            "Landroid/util/StateSet;",
            "Landroid/util/Log;",
            "Landroid/content/res/ColorStateList;",
            "Landroid/graphics/drawable/Drawable;",
            "Landroid/graphics/drawable/Drawable$ConstantState;",
            "Landroid/graphics/drawable/Drawable$Callback;",
            "Landroid/graphics/drawable/ColorDrawable;",
            "Landroid/graphics/drawable/DrawableContainer;",
            "Landroid/graphics/drawable/StateListDrawable;",
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
            "Landroid/graphics/Matrix;",
            "Landroid/graphics/Path;",
            "Landroid/graphics/Paint;",
            "Landroid/text/TextPaint;",
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
            "Landroid/widget/TableLayout$LayoutParams;",
            "Landroid/widget/TableRow$LayoutParams;",
            "Landroid/widget/FrameLayout$LayoutParams;",
            "Landroid/view/View$AccessibilityDelegate;",
            "Landroid/view/LayoutInflater;",
            "Landroid/view/LayoutInflater$Factory;",
            "Landroid/view/LayoutInflater$Factory2;",
            crate::inflater::MERGER,
            "Landroid/os/Bundle;",
            "Landroid/os/Parcel;",
            "Landroid/os/Parcelable;",
            "Landroid/os/Parcelable$Creator;",
            "Landroid/os/Parcelable$ClassLoaderCreator;",
            "Ljava/lang/ClassLoader;",
            "Landroid/view/GestureDetector;",
            "Landroid/view/GestureDetector$OnGestureListener;",
            "Landroid/view/GestureDetector$OnDoubleTapListener;",
            "Landroid/view/GestureDetector$SimpleOnGestureListener;",
            "Landroid/view/MotionEvent;",
            "Landroid/view/VelocityTracker;",
            "Landroid/view/ViewParent;",
            "Landroid/view/Gravity;",
            "Landroid/view/InputEvent;",
            "Landroid/view/inputmethod/InputMethodManager;",
            "Ldroidless/runtime/GestureTimer;",
            "Landroid/content/Intent;",
            "Landroid/net/Uri;",
            "Landroid/content/ContentResolver;",
            "Landroid/provider/DocumentsContract;",
            "Landroid/os/Binder;",
            "Landroid/os/IBinder;",
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
    fn group_drawable_states(&mut self, receiver: Word, extra: Word) -> Result<Word> {
        // ponytail: bounded synchronous child snapshots; use managed continuations
        // if deeper state trees or mutation during these callbacks becomes required.
        ensure!(
            self.drawable_state_path.len() < 16 && !self.drawable_state_path.contains(&receiver),
            "drawable state nesting limit or cycle"
        );
        ensure!(
            (0..=100_000).contains(&extra.int()?),
            "drawable state capacity limit"
        );
        let children = self.view_mut(receiver)?.children.clone();
        let depth = self.drawable_state_path.len();
        self.drawable_state_path.push(receiver);
        let roots = self.native_roots.len();
        self.native_roots.push(receiver);
        self.native_roots.extend(children.iter().copied());
        let state = (|| -> Result<Word> {
            let aggregate = self
                .focus_field(receiver, "droidless:view:add-child-states")?
                .truth();
            let mut capacity = extra.int()?;
            if aggregate {
                for child in &children {
                    let state = self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "getDrawableState".into(),
                            parameters: vec![],
                            returns: "[I".into(),
                        },
                        vec![*child],
                        true,
                    )?[0];
                    if state != Word::ZERO {
                        let Data::Array { element, values } = &self.heap.get(state)?.data else {
                            bail!("drawable states require int arrays");
                        };
                        ensure!(element == "I", "drawable states require int arrays");
                        capacity = capacity
                            .checked_add(i32::try_from(values.len())?)
                            .context("drawable state capacity overflow")?;
                        ensure!(capacity <= 100_000, "drawable state capacity limit");
                    }
                }
            }
            let state = self.invoke(
                Method {
                    class: "Landroid/view/View;".into(),
                    name: "onCreateDrawableState".into(),
                    parameters: vec!["I".into()],
                    returns: "[I".into(),
                },
                vec![receiver, Word::from(capacity)],
                false,
            )?[0];
            self.native_roots.push(state);
            if aggregate {
                for child in children {
                    let additional = self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "getDrawableState".into(),
                            parameters: vec![],
                            returns: "[I".into(),
                        },
                        vec![child],
                        true,
                    )?[0];
                    if additional != Word::ZERO {
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "mergeDrawableStates".into(),
                                parameters: vec!["[I".into(), "[I".into()],
                                returns: "[I".into(),
                            },
                            vec![state, additional],
                            false,
                        )?;
                    }
                }
            }
            Ok(state)
        })();
        self.native_roots.truncate(roots);
        self.drawable_state_path.truncate(depth);
        state
    }

    fn drawable_state_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if ![
            "Landroid/view/View;",
            "Landroid/view/ViewGroup;",
            "Landroid/widget/CheckedTextView;",
        ]
        .contains(&method.class.as_str())
            || !matches!(
                method.name.as_str(),
                "onCreateDrawableState"
                    | "getDrawableState"
                    | "mergeDrawableStates"
                    | "refreshDrawableState"
                    | "setAddStatesFromChildren"
                    | "addStatesFromChildren"
                    | "childDrawableStateChanged"
                    | "drawableStateChanged"
                    | "setEnabled"
                    | "setDuplicateParentStateEnabled"
            )
        {
            return Ok(None);
        }
        self.require_main_thread()?;
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let arg = |n| -> Result<Word> {
            args.get(n)
                .copied()
                .context("drawable state argument missing")
        };
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let mut result = vec![];
        match (method.class.as_str(), method.signature().as_str()) {
            ("Landroid/view/View;", "onCreateDrawableState(I)[I") => {
                ensure!(self.sync_depth < 32, "drawable state nesting limit");
                let extra =
                    usize::try_from(arg(1)?.int()?).context("negative drawable state capacity")?;
                ensure!(extra <= 100_000, "drawable state capacity limit");
                let fields = &self.heap.get(receiver)?.fields;
                if fields
                    .get("droidless:view:duplicate-parent-state")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth())
                    && let Some(parent) = fields
                        .get("droidless:view:parent")
                        .and_then(|values| values.first())
                        .copied()
                {
                    ensure!(
                        self.drawable_state_path.len() < 16
                            && parent != receiver
                            && !self.drawable_state_path.contains(&parent),
                        "drawable state nesting limit or cycle"
                    );
                    let depth = self.drawable_state_path.len();
                    self.drawable_state_path.push(receiver);
                    let roots = self.native_roots.len();
                    self.native_roots.extend([receiver, parent]);
                    let state = self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "onCreateDrawableState".into(),
                            parameters: vec!["I".into()],
                            returns: "[I".into(),
                        },
                        vec![parent, arg(1)?],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    self.drawable_state_path.truncate(depth);
                    result.extend(state?);
                    return Ok(Some(result));
                }
                let pressed = fields
                    .get("droidless:touch:pressed")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth());
                let mut states = vec![];
                if self.view_mut(receiver)?.enabled {
                    states.push(Word::from(16842910));
                }
                if pressed {
                    states.push(Word::from(16842919));
                }
                if self
                    .focus_field(receiver, "droidless:view:focused")?
                    .truth()
                {
                    states.push(Word::from(16842908));
                }
                // ponytail: enabled/pressed/focused states; add selection/window states with their event lifecycle.
                let array = self.array("I".into(), states.len() + extra)?;
                let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
                    bail!("drawable state is not array");
                };
                for (slot, value) in values.iter_mut().zip(states) {
                    *slot = vec![value];
                }
                result.push(array);
            }
            ("Landroid/view/ViewGroup;", "onCreateDrawableState(I)[I") => {
                result.push(self.group_drawable_states(receiver, arg(1)?)?);
            }
            ("Landroid/widget/CheckedTextView;", "onCreateDrawableState(I)[I") => {
                let extra = arg(1)?
                    .int()?
                    .checked_add(1)
                    .context("drawable state capacity overflow")?;
                let state = self.invoke(
                    Method {
                        class: "Landroid/widget/TextView;".into(),
                        name: "onCreateDrawableState".into(),
                        parameters: vec!["I".into()],
                        returns: "[I".into(),
                    },
                    vec![receiver, Word::from(extra)],
                    false,
                )?[0];
                if self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:checked")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth())
                {
                    let Data::Array { values, .. } = &mut self.heap.get_mut(state)?.data else {
                        bail!("checked drawable state is not array");
                    };
                    let end = values
                        .iter()
                        .rposition(|words| words.first().is_some_and(|word| word.truth()))
                        .map_or(0, |i| i + 1);
                    values[end] = vec![Word::from(16842912)];
                }
                result.push(state);
            }
            ("Landroid/view/View;", "getDrawableState()[I") => {
                result.extend(self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "onCreateDrawableState".into(),
                        parameters: vec!["I".into()],
                        returns: "[I".into(),
                    },
                    vec![receiver, Word::ZERO],
                    true,
                )?);
            }
            ("Landroid/view/View;", "mergeDrawableStates([I[I)[I") => {
                let (base, additional) = (arg(0)?, arg(1)?);
                let Data::Array { element, values } = &self.heap.get(base)?.data else {
                    bail!("drawable states require int arrays");
                };
                ensure!(element == "I", "drawable states require int arrays");
                let end = values
                    .iter()
                    .rposition(|words| words.first().is_some_and(|word| word.truth()))
                    .map_or(0, |i| i + 1);
                let Data::Array { element, values } = &self.heap.get(additional)?.data else {
                    bail!("drawable states require int arrays");
                };
                ensure!(element == "I", "drawable states require int arrays");
                self.invoke(
                    Method {
                        class: "Ljava/lang/System;".into(),
                        name: "arraycopy".into(),
                        parameters: vec![
                            "Ljava/lang/Object;".into(),
                            "I".into(),
                            "Ljava/lang/Object;".into(),
                            "I".into(),
                            "I".into(),
                        ],
                        returns: "V".into(),
                    },
                    vec![
                        additional,
                        Word::ZERO,
                        base,
                        Word::from(end as i32),
                        Word::from(values.len() as i32),
                    ],
                    false,
                )?;
                result.push(base);
            }
            ("Landroid/view/View;", "refreshDrawableState()V") => {
                ensure!(self.sync_depth < 32, "drawable state nesting limit");
                self.focus_root(receiver)
                    .context("drawable state nesting limit or cycle")?;
                let roots = self.native_roots.len();
                self.native_roots.push(receiver);
                let refreshed = (|| -> Result<()> {
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "drawableStateChanged".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    )?;
                    if let Some(parent) = self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:parent")
                        .and_then(|values| values.first())
                        .copied()
                    {
                        self.invoke(
                            Method {
                                class: "Landroid/view/ViewGroup;".into(),
                                name: "childDrawableStateChanged".into(),
                                parameters: vec!["Landroid/view/View;".into()],
                                returns: "V".into(),
                            },
                            vec![parent, receiver],
                            true,
                        )?;
                    }
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                refreshed?;
            }
            (
                "Landroid/view/ViewGroup;",
                "setAddStatesFromChildren(Z)V" | "addStatesFromChildren()Z",
            ) => {
                self.view_mut(receiver)?;
                if method.name == "setAddStatesFromChildren" {
                    let value = Word::from(i32::from(arg(1)?.int()? != 0));
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:add-child-states".into(), vec![value]);
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "refreshDrawableState".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    )?;
                } else {
                    result.push(self.focus_field(receiver, "droidless:view:add-child-states")?);
                }
            }
            ("Landroid/view/ViewGroup;", "childDrawableStateChanged(Landroid/view/View;)V") => {
                self.view_mut(receiver)?;
                self.view_mut(arg(1)?)?;
                if self
                    .focus_field(receiver, "droidless:view:add-child-states")?
                    .truth()
                {
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "refreshDrawableState".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    )?;
                }
            }
            ("Landroid/view/ViewGroup;", "drawableStateChanged()V") => {
                ensure!(self.sync_depth < 32, "drawable state nesting limit");
                let children = self.view_mut(receiver)?.children.clone();
                let duplicate = children
                    .into_iter()
                    .filter_map(|child| {
                        match self.focus_field(child, "droidless:view:duplicate-parent-state") {
                            Ok(flag) if flag.truth() => Some(Ok(child)),
                            Ok(_) => None,
                            Err(error) => Some(Err(error)),
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                if !duplicate.is_empty()
                    && self
                        .focus_field(receiver, "droidless:view:add-child-states")?
                        .truth()
                {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "child duplicateParentState conflicts with addStatesFromChildren",
                    ));
                }
                let roots = self.native_roots.len();
                self.native_roots.push(receiver);
                self.native_roots.extend(duplicate.iter().copied());
                let changed = (|| -> Result<()> {
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "drawableStateChanged".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        false,
                    )?;
                    for child in duplicate {
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "refreshDrawableState".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![child],
                            true,
                        )?;
                    }
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                changed?;
            }
            ("Landroid/view/View;", "setEnabled(Z)V") => {
                let enabled = arg(1)?.int()? != 0;
                if self.view_mut(receiver)?.enabled != enabled {
                    self.view_mut(receiver)?.enabled = enabled;
                    let roots = self.native_roots.len();
                    self.native_roots.push(receiver);
                    let refreshed = self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "refreshDrawableState".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    refreshed?;
                }
            }
            ("Landroid/view/View;", "drawableStateChanged()V") => {
                self.view_mut(receiver)?;
                let fields = &self.heap.get(receiver)?.fields;
                ensure!(
                    !fields
                        .get("droidless:view:state-list-animator")
                        .and_then(|values| values.first())
                        .is_some_and(|word| word.truth()),
                    "StateListAnimator state changes unsupported"
                );
                let background = fields
                    .get("droidless:view:background-drawable")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                if background != Word::ZERO {
                    let stateful = self.invoke(
                        Method {
                            class: "Landroid/graphics/drawable/Drawable;".into(),
                            name: "isStateful".into(),
                            parameters: vec![],
                            returns: "Z".into(),
                        },
                        vec![background],
                        true,
                    )?;
                    if stateful[0].truth() {
                        let roots = self.native_roots.len();
                        self.native_roots.extend([receiver, background]);
                        let changed = (|| -> Result<()> {
                            let state = self.invoke(
                                Method {
                                    class: "Landroid/view/View;".into(),
                                    name: "getDrawableState".into(),
                                    parameters: vec![],
                                    returns: "[I".into(),
                                },
                                vec![receiver],
                                true,
                            )?[0];
                            self.invoke(
                                Method {
                                    class: "Landroid/graphics/drawable/Drawable;".into(),
                                    name: "setState".into(),
                                    parameters: vec!["[I".into()],
                                    returns: "Z".into(),
                                },
                                vec![background, state],
                                true,
                            )?;
                            Ok(())
                        })();
                        self.native_roots.truncate(roots);
                        changed?;
                    }
                    self.view_mut(receiver)?.background =
                        self.background_drawable_color(background, 0)?;
                }
                self.refresh_foreground(receiver, true)?;
            }
            ("Landroid/view/View;", "setDuplicateParentStateEnabled(Z)V") => {
                self.view_mut(receiver)?;
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:duplicate-parent-state".into(),
                    vec![arg(1)?],
                );
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }

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
            let mut ancestor = style;
            for _ in 0..32 {
                let light = match ancestor {
                    0x0103_0237 | 0x0103_0241 => Some(true), // Material.Light[/NoActionBar], API 21.
                    0x0103_0224 | 0x0103_022e => Some(false), // Material[/NoActionBar], API 21.
                    _ => None,
                };
                if let Some(light) = light {
                    // ponytail: default text colors are flat; full framework selector/theme resources remain ahead.
                    for (id, data) in [
                        (0x0101_0036, if light { 0xde00_0000 } else { 0xffff_ffff }),
                        (0x0101_0038, if light { 0x8a00_0000 } else { 0xb3ff_ffff }),
                        (0x0101_009a, if light { 0x8000_0000 } else { 0x80ff_ffff }),
                    ] {
                        attributes.insert(
                            id,
                            Value {
                                kind: 0x1c,
                                data,
                                text: None,
                            },
                        );
                    }
                    attributes.insert(
                        0x0101_0033,
                        Value {
                            kind: 4,
                            data: if light { 0.26f32 } else { 0.30f32 }.to_bits(),
                            text: None,
                        },
                    );
                    attributes.insert(
                        0x0101_02eb,
                        Value {
                            kind: 5,
                            data: (56 << 8) | 1,
                            text: None,
                        },
                    );
                    break;
                }
                let Some(parent) = self.apk.resources.style_parent(ancestor) else {
                    break;
                };
                ancestor = parent;
            }
            if style >> 24 == 1 && !self.apk.resources.entries.contains_key(&style) {
                continue;
            }
            attributes.extend(self.apk.resources.style(style)?);
        }
        Ok(attributes)
    }

    fn theme_style_reference(
        &self,
        theme: &std::collections::BTreeMap<u32, Value>,
        mut value: Option<Value>,
    ) -> Result<Option<u32>> {
        let mut seen = std::collections::BTreeSet::new();
        while let Some(current) = value {
            if current.kind != 2 {
                return Ok((current.kind == 1).then_some(current.data));
            }
            ensure!(
                seen.len() < 32 && seen.insert(current.data),
                "theme style reference cycle or depth limit"
            );
            value = theme.get(&current.data).cloned();
        }
        Ok(None)
    }

    fn styled_set_attributes(
        &self,
        snapshot: &[u32],
        set: Word,
        default_attribute: u32,
        default_resource: u32,
    ) -> Result<std::collections::BTreeMap<u32, Value>> {
        let theme = self.styled_attributes(snapshot)?;
        let default = if default_attribute == 0 {
            default_resource
        } else {
            self.theme_style_reference(&theme, theme.get(&default_attribute).cloned())?
                .unwrap_or(default_resource)
        };
        if self.trace.framework {
            eprintln!(
                "styled default: theme={snapshot:x?}, attr=?0x{default_attribute:08x}, style=@0x{default:08x}"
            );
        }
        let mut attributes = theme.clone();
        if default != 0 {
            attributes.extend(self.styled_attributes(&[default])?);
        }
        if set != Word::ZERO {
            let style =
                self.theme_style_reference(&theme, self.attribute_set_value(set, "style")?)?;
            if let Some(style) = style.filter(|style| *style != 0) {
                attributes.extend(self.styled_attributes(&[style])?);
            }
            self.overlay_attributes(set, &mut attributes)?;
        }
        Ok(attributes)
    }

    fn child_attachment_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let attachment = method.class == "Landroid/view/ViewGroup;"
            && matches!(
                signature.as_str(),
                "addView(Landroid/view/View;)V"
                    | "addView(Landroid/view/View;I)V"
                    | "addView(Landroid/view/View;II)V"
                    | "addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"
            );
        let sized = method.class == "Landroid/view/ViewGroup;"
            && signature == "addView(Landroid/view/View;II)V";
        let factory = method.name == "generateDefaultLayoutParams"
            && method.parameters.is_empty()
            && matches!(
                method.class.as_str(),
                "Landroid/view/ViewGroup;"
                    | "Landroid/widget/LinearLayout;"
                    | "Landroid/widget/FrameLayout;"
                    | "Landroid/widget/TableLayout;"
                    | "Landroid/widget/TableRow;"
            )
            && (method.returns == "Landroid/view/ViewGroup$LayoutParams;"
                || (method.returns == "Landroid/widget/LinearLayout$LayoutParams;"
                    && matches!(
                        method.class.as_str(),
                        "Landroid/widget/LinearLayout;"
                            | "Landroid/widget/TableLayout;"
                            | "Landroid/widget/TableRow;"
                    ))
                || (method.returns == "Landroid/widget/FrameLayout$LayoutParams;"
                    && method.class == "Landroid/widget/FrameLayout;"));
        if !attachment && !factory {
            return Ok(None);
        }
        self.require_main_thread()?;
        ensure!(self.sync_depth < 32, "sized child callback nesting limit");
        ensure!(
            args.len() == method.parameters.len() + 1,
            "invalid child attachment argument count"
        );
        let receiver = args[0];
        self.view_mut(receiver)?;
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let result = (|| -> Result<Vec<Word>> {
            if signature == "addView(Landroid/view/View;)V" {
                self.invoke(
                    Method {
                        class: "Landroid/view/ViewGroup;".into(),
                        name: "addView".into(),
                        parameters: vec!["Landroid/view/View;".into(), "I".into()],
                        returns: "V".into(),
                    },
                    vec![receiver, args[1], Word::from(-1)],
                    true,
                )?;
                return Ok(vec![]);
            }
            if attachment && !sized {
                let (index, mut params) = if signature == "addView(Landroid/view/View;I)V" {
                    (
                        args[2].int()?,
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "getLayoutParams".into(),
                                parameters: vec![],
                                returns: "Landroid/view/ViewGroup$LayoutParams;".into(),
                            },
                            vec![args[1]],
                            true,
                        )?[0],
                    )
                } else {
                    (-1, args[2])
                };
                self.native_roots.push(params);
                if params == Word::ZERO && signature == "addView(Landroid/view/View;I)V" {
                    params = self.invoke(
                        Method {
                            class: "Landroid/view/ViewGroup;".into(),
                            name: "generateDefaultLayoutParams".into(),
                            parameters: vec![],
                            returns: "Landroid/view/ViewGroup$LayoutParams;".into(),
                        },
                        vec![receiver],
                        true,
                    )?[0];
                    self.native_roots.push(params);
                    if params == Word::ZERO {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "generateDefaultLayoutParams() cannot return null",
                        ));
                    }
                }
                self.invoke(
                    Method {
                        class: "Landroid/view/ViewGroup;".into(),
                        name: "addView".into(),
                        parameters: vec![
                            "Landroid/view/View;".into(),
                            "I".into(),
                            "Landroid/view/ViewGroup$LayoutParams;".into(),
                        ],
                        returns: "V".into(),
                    },
                    vec![receiver, args[1], Word::from(index), params],
                    true,
                )?;
                return Ok(vec![]);
            }
            if sized {
                let width = args[2].int()?;
                let height = args[3].int()?;
                let params = self.invoke(
                    Method {
                        class: "Landroid/view/ViewGroup;".into(),
                        name: "generateDefaultLayoutParams".into(),
                        parameters: vec![],
                        returns: "Landroid/view/ViewGroup$LayoutParams;".into(),
                    },
                    vec![receiver],
                    true,
                )?[0];
                if params == Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/NullPointerException;",
                        "null default layout parameters",
                    ));
                }
                self.native_roots.push(params);
                ensure!(
                    self.is_a(
                        &self.heap.get(params)?.class,
                        "Landroid/view/ViewGroup$LayoutParams;"
                    ),
                    "default factory returned invalid layout parameters"
                );
                let fields = &mut self.heap.get_mut(params)?.fields;
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->width:I".into(),
                    vec![Word::from(width)],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->height:I".into(),
                    vec![Word::from(height)],
                );
                self.invoke(
                    Method {
                        class: "Landroid/view/ViewGroup;".into(),
                        name: "addView".into(),
                        parameters: vec![
                            "Landroid/view/View;".into(),
                            "I".into(),
                            "Landroid/view/ViewGroup$LayoutParams;".into(),
                        ],
                        returns: "V".into(),
                    },
                    vec![receiver, args[1], Word::from(-1), params],
                    true,
                )?;
                return Ok(vec![]);
            }
            let (class, constructor, width, height) = match method.class.as_str() {
                "Landroid/widget/LinearLayout;" => {
                    let width = match self.view_mut(receiver)?.orientation {
                        0 => -2,
                        1 => -1,
                        _ => return Ok(vec![Word::ZERO]),
                    };
                    (
                        "Landroid/widget/LinearLayout$LayoutParams;",
                        "Landroid/widget/LinearLayout$LayoutParams;",
                        width,
                        -2,
                    )
                }
                "Landroid/widget/FrameLayout;" => (
                    "Landroid/widget/FrameLayout$LayoutParams;",
                    "Landroid/widget/FrameLayout$LayoutParams;",
                    -1,
                    -1,
                ),
                "Landroid/widget/TableLayout;" => (
                    "Landroid/widget/TableLayout$LayoutParams;",
                    "Landroid/widget/LinearLayout$LayoutParams;",
                    -1,
                    -2,
                ),
                "Landroid/widget/TableRow;" => (
                    "Landroid/widget/TableRow$LayoutParams;",
                    "Landroid/widget/LinearLayout$LayoutParams;",
                    -1,
                    -2,
                ),
                _ => (
                    "Landroid/view/ViewGroup$LayoutParams;",
                    "Landroid/view/ViewGroup$LayoutParams;",
                    -2,
                    -2,
                ),
            };
            let params = self.new_instance(class)?;
            self.native_roots.push(params);
            self.invoke(
                Method {
                    class: constructor.into(),
                    name: "<init>".into(),
                    parameters: vec!["I".into(), "I".into()],
                    returns: "V".into(),
                },
                vec![params, Word::from(width), Word::from(height)],
                false,
            )?;
            if class == "Landroid/widget/TableRow$LayoutParams;" {
                let fields = &mut self.heap.get_mut(params)?.fields;
                fields.insert(format!("{class}->column:I"), vec![Word::from(-1)]);
                fields.insert(format!("{class}->span:I"), vec![Word::from(1)]);
            }
            Ok(vec![params])
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }

    fn text_appearance_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != "Landroid/widget/TextView;"
            || !matches!(
                method.signature().as_str(),
                "setTextAppearance(Landroid/content/Context;I)V" | "setTextAppearance(I)V"
            )
        {
            return Ok(None);
        }
        self.require_main_thread()?;
        ensure!(self.sync_depth < 32, "text appearance nesting limit");
        let arg = |n| -> Result<Word> {
            args.get(n)
                .copied()
                .context("text appearance argument missing")
        };
        let receiver = arg(0)?;
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let applied = (|| -> Result<()> {
            let context = if method.parameters.len() == 2 {
                arg(1)?
            } else {
                self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "getContext".into(),
                        parameters: vec![],
                        returns: "Landroid/content/Context;".into(),
                    },
                    vec![receiver],
                    true,
                )?[0]
            };
            self.native_roots.push(context);
            let style = Word::from(arg(method.parameters.len())?.int()?);
            let attributes = self.array("I".into(), 2)?;
            self.native_roots.push(attributes);
            let Data::Array { values, .. } = &mut self.heap.get_mut(attributes)?.data else {
                bail!("text appearance attributes are not an array");
            };
            values[0] = vec![Word::from(0x0101_0098)];
            values[1] = vec![Word::from(0x0101_0095)];
            let appearance = self.invoke(
                Method {
                    class: "Landroid/content/Context;".into(),
                    name: "obtainStyledAttributes".into(),
                    parameters: vec!["I".into(), "[I".into()],
                    returns: "Landroid/content/res/TypedArray;".into(),
                },
                vec![context, style, attributes],
                true,
            )?[0];
            self.native_roots.push(appearance);
            let colors = self.invoke(
                Method {
                    class: "Landroid/content/res/TypedArray;".into(),
                    name: "getColorStateList".into(),
                    parameters: vec!["I".into()],
                    returns: "Landroid/content/res/ColorStateList;".into(),
                },
                vec![appearance, Word::ZERO],
                true,
            )?[0];
            if colors != Word::ZERO {
                self.native_roots.push(colors);
                self.invoke(
                    Method {
                        class: "Landroid/widget/TextView;".into(),
                        name: "setTextColor".into(),
                        parameters: vec!["Landroid/content/res/ColorStateList;".into()],
                        returns: "V".into(),
                    },
                    vec![receiver, colors],
                    true,
                )?;
            }
            let pixels = self.invoke(
                Method {
                    class: "Landroid/content/res/TypedArray;".into(),
                    name: "getDimensionPixelSize".into(),
                    parameters: vec!["I".into(), "I".into()],
                    returns: "I".into(),
                },
                vec![appearance, Word::from(1), Word::ZERO],
                true,
            )?[0]
                .int()?;
            ensure!(pixels >= 0, "invalid text appearance size");
            if pixels != 0 && self.view_mut(receiver)?.text_size != pixels as f32 {
                self.view_mut(receiver)?.text_size = pixels as f32;
                self.invalidate_text_layout(receiver)?;
            }
            // ponytail: color and pixel size profile; add typeface, hint/link/shadow
            // appearance attributes with their native text rendering contracts.
            self.invoke(
                Method {
                    class: "Landroid/content/res/TypedArray;".into(),
                    name: "recycle".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                vec![appearance],
                true,
            )?;
            Ok(())
        })();
        self.native_roots.truncate(roots);
        applied?;
        Ok(Some(vec![]))
    }

    fn styled_array_native(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        if ![
            "Landroid/content/Context;",
            "Landroid/content/res/Resources;",
            "Landroid/content/res/Resources$Theme;",
            "Landroid/content/res/TypedArray;",
        ]
        .contains(&method.class.as_str())
            || !matches!(
                method.name.as_str(),
                "obtainStyledAttributes"
                    | "obtainAttributes"
                    | "getColor"
                    | "getColorStateList"
                    | "resolveAttribute"
            )
        {
            return Ok(None);
        }
        ensure!(self.sync_depth < 32, "styled attributes nesting limit");
        let arg = |n| -> Result<Word> {
            args.get(n)
                .copied()
                .context("styled attribute argument missing")
        };
        let receiver = arg(0)?;
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, &method.class),
            "invalid styled attribute receiver"
        );
        let signature = method.signature();
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let result = (|| -> Result<Option<Vec<Word>>> {
            match (method.class.as_str(), signature.as_str()) {
                (
                    "Landroid/content/res/Resources$Theme;",
                    "resolveAttribute(ILandroid/util/TypedValue;Z)Z",
                ) => {
                    ensure!(
                        self.is_a(
                            &self.heap.get(receiver)?.class,
                            "Landroid/content/res/Resources$Theme;"
                        ),
                        "resolveAttribute expects Theme"
                    );
                    let output = arg(2)?;
                    let class = "Landroid/util/TypedValue;";
                    ensure!(
                        self.is_a(&self.heap.get(output)?.class, class),
                        "resolveAttribute expects TypedValue"
                    );
                    let resolve = arg(3)?.int()? != 0;
                    let attributes = self.styled_attributes(&self.theme_styles(receiver)?)?;
                    let Some(mut value) = attributes.get(&(arg(1)?.int()? as u32)).cloned() else {
                        return Ok(Some(vec![Word::ZERO]));
                    };
                    let mut seen = std::collections::BTreeSet::new();
                    while value.kind == 2 {
                        if seen.len() == 20 || !seen.insert(value.data) {
                            return Ok(Some(vec![Word::ZERO]));
                        }
                        let Some(next) = attributes.get(&value.data) else {
                            return Ok(Some(vec![Word::ZERO]));
                        };
                        value = next.clone();
                    }
                    if value.kind == 0 {
                        return Ok(Some(vec![Word::ZERO]));
                    }
                    let mut resource_id = 0;
                    let mut asset_cookie = 1;
                    if resolve {
                        for _ in 0..20 {
                            if value.kind != 1 || value.data == 0 {
                                break;
                            }
                            resource_id = value.data;
                            if let Some(entry) = self.apk.resources.entries.get(&value.data) {
                                let Some(next) = &entry.value else {
                                    break;
                                };
                                value = next.clone();
                            } else if matches!(value.data, 0x0106_000b..=0x0106_000d) {
                                value = self.attribute(&value)?;
                                asset_cookie = 0;
                            } else {
                                return Ok(Some(vec![Word::ZERO]));
                            }
                        }
                    }
                    let string = if value.kind == 3 {
                        self.heap.string(value.display())?
                    } else {
                        Word::ZERO
                    };
                    let fields = &mut self.heap.get_mut(output)?.fields;
                    for (name, data) in [
                        ("type", i32::from(value.kind)),
                        ("data", value.data as i32),
                        ("resourceId", resource_id as i32),
                        ("assetCookie", asset_cookie),
                        ("changingConfigurations", 0),
                    ] {
                        fields.insert(format!("{class}->{name}:I"), vec![Word::from(data)]);
                    }
                    fields.insert(
                        format!("{class}->string:Ljava/lang/CharSequence;"),
                        vec![string],
                    );
                    Ok(Some(vec![Word::from(1)]))
                }
                (
                    "Landroid/content/Context;",
                    "obtainStyledAttributes([I)Landroid/content/res/TypedArray;"
                    | "obtainStyledAttributes(I[I)Landroid/content/res/TypedArray;"
                    | "obtainStyledAttributes(Landroid/util/AttributeSet;[I)Landroid/content/res/TypedArray;"
                    | "obtainStyledAttributes(Landroid/util/AttributeSet;[III)Landroid/content/res/TypedArray;",
                ) => {
                    let attrs = if method.parameters.len() == 1 {
                        arg(1)?
                    } else {
                        arg(2)?
                    };
                    let theme = self.invoke(
                        Method {
                            class: "Landroid/content/Context;".into(),
                            name: "getTheme".into(),
                            parameters: vec![],
                            returns: "Landroid/content/res/Resources$Theme;".into(),
                        },
                        vec![receiver],
                        true,
                    )?[0];
                    self.native_roots.push(theme);
                    ensure!(
                        self.is_a(
                            &self.heap.get(theme)?.class,
                            "Landroid/content/res/Resources$Theme;"
                        ),
                        "Context.getTheme returned invalid Theme"
                    );
                    let snapshot = self.theme_styles(theme)?;
                    let mut styles = snapshot.clone();
                    if method
                        .parameters
                        .first()
                        .is_some_and(|parameter| parameter == "I")
                    {
                        styles.push(arg(1)?.int()? as u32);
                    }
                    let attributes = if method
                        .parameters
                        .first()
                        .is_some_and(|parameter| parameter == "Landroid/util/AttributeSet;")
                    {
                        self.styled_set_attributes(
                            &snapshot,
                            arg(1)?,
                            if method.parameters.len() == 4 {
                                arg(3)?.int()? as u32
                            } else {
                                0
                            },
                            if method.parameters.len() == 4 {
                                arg(4)?.int()? as u32
                            } else {
                                0
                            },
                        )?
                    } else {
                        self.styled_attributes(&styles)?
                    };
                    Ok(Some(vec![self.typed_array(
                        attrs,
                        &attributes,
                        &snapshot,
                    )?]))
                }
                (
                    "Landroid/content/res/Resources;",
                    "obtainAttributes(Landroid/util/AttributeSet;[I)Landroid/content/res/TypedArray;",
                )
                | (
                    "Landroid/content/res/Resources$Theme;",
                    "obtainStyledAttributes([I)Landroid/content/res/TypedArray;"
                    | "obtainStyledAttributes(I[I)Landroid/content/res/TypedArray;"
                    | "obtainStyledAttributes(Landroid/util/AttributeSet;[III)Landroid/content/res/TypedArray;",
                ) => {
                    let attrs = arg(if method.parameters.len() == 1 { 1 } else { 2 })?;
                    let snapshot = if method.class == "Landroid/content/res/Resources$Theme;" {
                        self.theme_styles(receiver)?
                    } else {
                        vec![]
                    };
                    let attributes = if method.parameters.first().is_some_and(|p| p == "I") {
                        let mut styles = snapshot.clone();
                        styles.push(arg(1)?.int()? as u32);
                        self.styled_attributes(&styles)?
                    } else if method.class == "Landroid/content/res/Resources$Theme;" {
                        self.styled_set_attributes(
                            &snapshot,
                            if method.parameters.len() == 4 {
                                arg(1)?
                            } else {
                                Word::ZERO
                            },
                            if method.parameters.len() == 4 {
                                arg(3)?.int()? as u32
                            } else {
                                0
                            },
                            if method.parameters.len() == 4 {
                                arg(4)?.int()? as u32
                            } else {
                                0
                            },
                        )?
                    } else {
                        let mut attributes = std::collections::BTreeMap::new();
                        self.overlay_attributes(arg(1)?, &mut attributes)?;
                        attributes
                    };
                    Ok(Some(vec![self.typed_array(
                        attrs,
                        &attributes,
                        &snapshot,
                    )?]))
                }
                (
                    "Landroid/content/res/TypedArray;",
                    "getColor(II)I" | "getColorStateList(I)Landroid/content/res/ColorStateList;",
                ) => {
                    let index = usize::try_from(arg(1)?.int()?).map_err(|_| {
                        fault(
                            "Ljava/lang/ArrayIndexOutOfBoundsException;",
                            "typed color index",
                        )
                    })?;
                    let fallback = if signature == "getColor(II)I" {
                        Word::from(arg(2)?.int()?)
                    } else {
                        Word::ZERO
                    };
                    let Data::TypedArray(values) = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized TypedArray");
                    };
                    let value = values.get(index).cloned().ok_or_else(|| {
                        fault(
                            "Ljava/lang/ArrayIndexOutOfBoundsException;",
                            "typed color index",
                        )
                    })?;
                    let value = value
                        .map(|value| {
                            self.themed_attribute(receiver, &value).map_err(|error| {
                                fault(
                                    "Ljava/lang/RuntimeException;",
                                    format!(
                                        "Failed to resolve attribute at index {index}: {error:#}"
                                    ),
                                )
                            })
                        })
                        .transpose()?;
                    let colors = if let Some(value) = value
                        .filter(|value| value.kind != 0 && !(value.kind == 1 && value.data == 0))
                    {
                        self.color_state_list(&value)?
                    } else {
                        Word::ZERO
                    };
                    let result = if signature == "getColor(II)I" {
                        if colors == Word::ZERO {
                            fallback
                        } else {
                            self.native_roots.push(colors);
                            self.invoke(
                                Method {
                                    class: "Landroid/content/res/ColorStateList;".into(),
                                    name: "getDefaultColor".into(),
                                    parameters: vec![],
                                    returns: "I".into(),
                                },
                                vec![colors],
                                true,
                            )?[0]
                        }
                    } else {
                        colors
                    };
                    Ok(Some(vec![result]))
                }
                _ => Ok(None),
            }
        })();
        self.native_roots.truncate(roots);
        result
    }

    fn typed_array(
        &mut self,
        attrs: Word,
        attributes: &std::collections::BTreeMap<u32, Value>,
        theme_styles: &[u32],
    ) -> Result<Word> {
        let values = match attrs {
            Word::Bits(0) => vec![],
            attrs => {
                let Data::Array { element, values } = &self.heap.get(attrs)?.data else {
                    bail!("TypedArray attributes must be an int array");
                };
                ensure!(
                    element == "I" && values.len() <= 4096,
                    "invalid or oversized TypedArray attribute array"
                );
                values
                    .iter()
                    .map(|value| {
                        let id = value
                            .first()
                            .copied()
                            .context("empty attribute id")?
                            .int()? as u32;
                        let value = attributes.get(&id).cloned();
                        if self.trace.framework {
                            eprintln!(
                                "styled attribute ?0x{id:08x}: {:?}",
                                value.as_ref().map(|v| (v.kind, v.data))
                            );
                        }
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>>>()?
            }
        };
        let array = self.heap.instance("Landroid/content/res/TypedArray;")?;
        self.heap.get_mut(array)?.data = Data::TypedArray(values);
        self.heap.get_mut(array)?.fields.insert(
            "droidless:theme:styles".into(),
            theme_styles
                .iter()
                .map(|style| Word::from(*style as i32))
                .collect(),
        );
        Ok(array)
    }

    fn overlay_attributes(
        &self,
        set: Word,
        attributes: &mut std::collections::BTreeMap<u32, Value>,
    ) -> Result<()> {
        if set == Word::ZERO {
            return Ok(());
        }
        match &self.heap.get(set)?.data {
            Data::Attributes { resources, .. } => attributes.extend(resources.clone()),
            Data::XmlPull {
                events,
                position,
                closed,
            } => {
                ensure!(!closed, "closed XmlResourceParser");
                let event = events
                    .get(*position)
                    .context("invalid XML parser position")?;
                attributes.extend(
                    event
                        .attributes
                        .iter()
                        .filter(|a| a.name_resource != 0)
                        .map(|a| (a.name_resource, a.value.clone())),
                );
            }
            _ => bail!("uninitialized AttributeSet"),
        }
        Ok(())
    }

    pub(crate) fn sdk_field(&self, field: &Field) -> bool {
        field.class == "Landroid/os/Build$VERSION;"
            && field.name == "SDK_INT"
            && self.class_location(&field.class).is_none()
    }
    fn reference_key(&self, receiver: Word) -> Result<&'static str> {
        let class = &self.heap.get(receiver)?.class;
        ensure!(
            self.is_a(class, "Ljava/lang/ref/Reference;"),
            "Reference receiver required"
        );
        Ok(if self.is_a(class, "Ljava/lang/ref/WeakReference;") {
            crate::heap::WEAK_REFERENT
        } else {
            "droidless:reference:referent"
        })
    }
    pub(crate) fn view_empty_state_field(&self, field: &Field) -> bool {
        field.class == "Landroid/view/View;"
            && field.name == "EMPTY_STATE_SET"
            && self.class_location(&field.class).is_none()
    }
    pub(crate) fn view_empty_state_object(&mut self, field: &Field) -> Result<Word> {
        let key = field.key();
        if let Some(value) = self.statics.get(&key).and_then(|words| words.first()) {
            return Ok(*value);
        }
        let value = self.array("I".into(), 0)?;
        self.statics.insert(key, vec![value]);
        Ok(value)
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
    pub(crate) fn graphics_enum_object(&mut self, field: &Field) -> Result<Word> {
        let ordinal = graphics_enum_names(&field.class)
            .and_then(|names| names.iter().position(|name| *name == field.name))
            .context("unknown graphics enum value")?;
        if let Some(value) = self
            .statics
            .get(&field.key())
            .and_then(|values| values.first())
        {
            return Ok(*value);
        }
        let mode = self.heap.instance(&field.class)?;
        let name = self.intern(field.name.clone())?;
        let fields = &mut self.heap.get_mut(mode)?.fields;
        fields.insert("droidless:enum:name".into(), vec![name]);
        fields.insert(
            "droidless:enum:ordinal".into(),
            vec![Word::from(ordinal as i32)],
        );
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
        if let Some(result) = self.foreground_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.selector_drawable_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.dialog_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.window_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.context_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.typography_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.text_native(method, args)? {
            return Ok(Some(result));
        }
        // Recursive UI callbacks avoid the large fallback dispatcher's debug stack frame.
        // Each small dispatcher retains the shared UI-thread guard.
        if let Some(result) = self.drawable_state_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.styled_array_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.text_appearance_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.child_attachment_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.focus_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.inflater_native(method, args)? {
            return Ok(Some(result));
        }
        self.native_framework(method, args)
    }
    fn native_framework(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        if let Some(result) = self.throwable_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.document_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.fragment_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.scrolling_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.grid_native(method, args)? {
            return Ok(Some(result));
        }
        if method.class.starts_with("Landroid/view/")
            || method.class.starts_with("Landroid/widget/")
            || method.class == "Landroid/app/Activity;"
        {
            self.require_main_thread()?;
        }
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        if let Some(result) = self.menu_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.xml_resource_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.matrix_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.path_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.text_layout_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.component_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.parcel_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.touch_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.animation_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.property_animation_native(method, args)? {
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
        if let Some(result) = self.timer_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.executor_native(method, args)? {
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
                    let child = children.remove(index as usize);
                    self.heap
                        .get_mut(child)?
                        .fields
                        .remove("droidless:view:parent");
                    return Ok(Some(result));
                }
                "removeDetachedView(Landroid/view/View;Z)V" => {
                    self.require_main_thread()?;
                    let child = arg(1)?;
                    self.view_mut(child)?;
                    arg(2)?.int()?;
                    let fields = &self.heap.get(receiver)?.fields;
                    ensure!(
                        fields
                            .get("droidless:touch:child")
                            .and_then(|v| v.first())
                            .copied()
                            .unwrap_or(Word::ZERO)
                            != child,
                        "removing a detached active touch target is unsupported"
                    );
                    ensure!(
                        fields
                            .get("droidless:view:layout-transition")
                            .and_then(|v| v.first())
                            .copied()
                            .unwrap_or(Word::ZERO)
                            == Word::ZERO,
                        "detached removal with LayoutTransition is unsupported"
                    );
                    // ponytail: unanimated removal; window attachment and disappearing-view animations need their own lifecycle.
                    self.invoke(
                        Method {
                            class: "Landroid/view/ViewGroup;".into(),
                            name: "onViewRemoved".into(),
                            parameters: vec!["Landroid/view/View;".into()],
                            returns: "V".into(),
                        },
                        vec![receiver, child],
                        true,
                    )?;
                    return Ok(Some(result));
                }
                "onViewRemoved(Landroid/view/View;)V" => {
                    self.require_main_thread()?;
                    self.view_mut(arg(1)?)?;
                    self.hierarchy_change(receiver, arg(1)?, false)?;
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
                    self.focus_hierarchy_change(receiver, child, true)?;
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
            ("Landroid/net/ConnectivityManager$NetworkCallback;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            (
                "Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;",
                "<init>(Ljava/lang/Object;)V",
            )
            | (
                "Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;",
                "<init>(Ljava/lang/Object;Ljava/lang/ref/ReferenceQueue;)V",
            ) => {
                let key = self.reference_key(receiver)?;
                let referent = arg(1)?;
                if referent != Word::ZERO {
                    self.heap.get(referent)?;
                }
                ensure!(
                    method.parameters.len() == 1 || arg(2)? == Word::ZERO,
                    "ReferenceQueue registration unsupported"
                );

                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(key.into(), vec![referent]);
            }
            (
                "Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;",
                "get()Ljava/lang/Object;",
            ) => {
                let key = self.reference_key(receiver)?;

                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(key)
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Ljava/lang/ref/Reference;" | "Ljava/lang/ref/WeakReference;", "clear()V") => {
                let key = self.reference_key(receiver)?;

                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(key.into(), vec![Word::ZERO]);
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
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("color".into(), vec![Word::from(0x00000000_i32)]);
            }
            ("Landroid/graphics/drawable/ColorDrawable;", "<init>(I)V")
            | ("Landroid/graphics/drawable/ColorDrawable;", "setColor(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("color".into(), vec![arg(1)?]);
                if method.name == "setColor" {
                    let roots = self.native_roots.len();
                    self.native_roots.push(receiver);
                    let changed = self.invoke(
                        Method {
                            class: "Landroid/graphics/drawable/Drawable;".into(),
                            name: "invalidateSelf".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    changed?;
                }
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
            (
                "Landroid/graphics/drawable/LayerDrawable;",
                "<init>([Landroid/graphics/drawable/Drawable;)V",
            ) => {
                let layers = arg(1)?;
                ensure!(
                    matches!(self.heap.get(layers)?.data, Data::Array { .. }),
                    "LayerDrawable layers must be an array"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:layers".into(), vec![layers]);
            }
            (
                "Landroid/graphics/drawable/RippleDrawable;",
                "<init>(Landroid/content/res/ColorStateList;Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;)V",
            ) => {
                for (field, value) in [
                    ("droidless:drawable:ripple-color", arg(1)?),
                    ("droidless:drawable:ripple-content", arg(2)?),
                    ("droidless:drawable:ripple-mask", arg(3)?),
                ] {
                    if value != Word::ZERO {
                        self.heap.get(value)?;
                    }
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(field.into(), vec![value]);
                }
            }
            (
                "Landroid/graphics/drawable/InsetDrawable;",
                "<init>(Landroid/graphics/drawable/Drawable;IIII)V",
            ) => {
                let drawable = arg(1)?;
                self.heap.get(drawable)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:inset-content".into(), vec![drawable]);
            }
            ("Landroid/animation/StateListAnimator;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Ljava/util/BitSet;", "<init>(I)V") => {
                let size = arg(1)?.int()?;
                if size < 0 {
                    return Err(fault(
                        "Ljava/lang/NegativeArraySizeException;",
                        size.to_string(),
                    ));
                }
                ensure!(size <= 1_000_000, "BitSet size limit reached");
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:bitset:values".into(),
                    vec![Word::ZERO; size as usize],
                );
            }
            ("Ljava/util/BitSet;", "set(IZ)V") => {
                let index = arg(1)?.int()?;
                if index < 0 {
                    return Err(fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        index.to_string(),
                    ));
                }
                ensure!(index < 1_000_000, "BitSet size limit reached");
                let value = Word::from(i32::from(arg(2)?.truth()));
                let values = self
                    .heap
                    .get_mut(receiver)?
                    .fields
                    .entry("droidless:bitset:values".into())
                    .or_default();
                values.resize(values.len().max(index as usize + 1), Word::ZERO);
                values[index as usize] = value;
            }
            ("Ljava/util/BitSet;", "set(IIZ)V") => {
                let from = arg(1)?.int()?;
                let to = arg(2)?.int()?;
                if from < 0 || to < from {
                    return Err(fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        format!("{from}..{to}"),
                    ));
                }
                ensure!(to <= 1_000_000, "BitSet size limit reached");
                let value = Word::from(i32::from(arg(3)?.truth()));
                let values = self
                    .heap
                    .get_mut(receiver)?
                    .fields
                    .entry("droidless:bitset:values".into())
                    .or_default();
                values.resize(values.len().max(to as usize), Word::ZERO);
                values[from as usize..to as usize].fill(value);
            }
            ("Ljava/util/BitSet;", "clear()V") => {
                if let Some(values) = self
                    .heap
                    .get_mut(receiver)?
                    .fields
                    .get_mut("droidless:bitset:values")
                {
                    values.fill(Word::ZERO);
                }
            }
            ("Ljava/util/BitSet;", "clear(I)V") => {
                let index = arg(1)?.int()?;
                if index < 0 {
                    return Err(fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        index.to_string(),
                    ));
                }
                if let Some(value) = self
                    .heap
                    .get_mut(receiver)?
                    .fields
                    .get_mut("droidless:bitset:values")
                    .and_then(|values| values.get_mut(index as usize))
                {
                    *value = Word::ZERO;
                }
            }
            ("Ljava/util/BitSet;", "get(I)Z") => {
                let index = arg(1)?.int()?;
                if index < 0 {
                    return Err(fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        index.to_string(),
                    ));
                }
                let set = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:bitset:values")
                    .and_then(|values| values.get(index as usize))
                    .is_some_and(|value| value.truth());
                result.push(Word::from(i32::from(set)));
            }
            ("Ljava/util/BitSet;", "isEmpty()Z") => {
                let empty = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:bitset:values")
                    .is_none_or(|values| values.iter().all(|value| !value.truth()));
                result.push(Word::from(i32::from(empty)));
            }
            ("Ljava/util/Date;", "<init>()V") => {
                let now = self.invoke(
                    Method {
                        class: "Ljava/lang/System;".into(),
                        name: "currentTimeMillis".into(),
                        parameters: vec![],
                        returns: "J".into(),
                    },
                    vec![],
                    false,
                )?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:date:time".into(), now);
            }
            ("Ljava/util/Date;", "<init>(J)V" | "setTime(J)V") => {
                let millis = args.get(1..3).context("Date timestamp missing")?.to_vec();
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:date:time".into(), millis);
            }
            ("Ljava/util/Date;", "getTime()J") => {
                let time = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:date:time")
                    .cloned()
                    .unwrap_or_else(|| wide(0));
                result.extend(time);
            }
            ("Ljava/util/Date;", "before(Ljava/util/Date;)Z")
            | ("Ljava/util/Date;", "after(Ljava/util/Date;)Z")
            | ("Ljava/util/Date;", "compareTo(Ljava/util/Date;)I") => {
                let left = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:date:time")
                    .cloned()
                    .unwrap_or_else(|| wide(0));
                let right = self
                    .heap
                    .get(arg(1)?)?
                    .fields
                    .get("droidless:date:time")
                    .cloned()
                    .unwrap_or_else(|| wide(0));
                let comparison = crate::heap::bits64(&left)?.cmp(&crate::heap::bits64(&right)?);
                result.push(Word::from(if method.name == "before" {
                    i32::from(comparison.is_lt())
                } else if method.name == "after" {
                    i32::from(comparison.is_gt())
                } else {
                    comparison as i32
                }));
            }
            (
                "Landroid/animation/StateListAnimator;",
                "addState([ILandroid/animation/Animator;)V",
            ) => {
                let state = arg(1)?;
                let animator = arg(2)?;
                self.heap.get(state)?;
                self.heap.get(animator)?;
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields
                    .entry("droidless:animation:states".into())
                    .or_default()
                    .extend([state, animator]);
            }
            (
                "Landroid/animation/ObjectAnimator;",
                "ofFloat(Ljava/lang/Object;Ljava/lang/String;[F)Landroid/animation/ObjectAnimator;",
            ) => {
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
            (
                "Landroid/animation/Animator;",
                "setInterpolator(Landroid/animation/TimeInterpolator;)V",
            ) => {
                let interpolator = arg(1)?;
                self.heap.get(interpolator)?;
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:animation:interpolator".into(),
                    vec![interpolator],
                );
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "getMinimumWidth()I" | "getMinimumHeight()I",
            ) => {
                let intrinsic = self.invoke(
                    Method {
                        class: "Landroid/graphics/drawable/Drawable;".into(),
                        name: if method.name.ends_with("Width") {
                            "getIntrinsicWidth"
                        } else {
                            "getIntrinsicHeight"
                        }
                        .into(),
                        parameters: vec![],
                        returns: "I".into(),
                    },
                    vec![receiver],
                    true,
                )?;
                result.push(Word::from(
                    intrinsic
                        .first()
                        .context("drawable intrinsic size missing")?
                        .int()?
                        .max(0),
                ));
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "getIntrinsicWidth()I" | "getIntrinsicHeight()I",
            ) => {
                let object = self.heap.get(receiver)?;
                let bitmap = object
                    .fields
                    .get("droidless:drawable:bitmap")
                    .and_then(|values| values.first())
                    .copied();
                let size = if let Some(bitmap) = bitmap {
                    let Data::Bitmap {
                        width,
                        height,
                        recycled: false,
                        ..
                    } = &self.heap.get(bitmap)?.data
                    else {
                        bail!("drawable requires a live Bitmap");
                    };
                    (if method.name.ends_with("Width") {
                        *width
                    } else {
                        *height
                    }) as i32
                } else {
                    ensure!(
                        object.class == "Landroid/graphics/drawable/Drawable;"
                            || [
                                "Landroid/graphics/drawable/ColorDrawable;",
                                "Landroid/graphics/drawable/GradientDrawable;"
                            ]
                            .iter()
                            .any(|class| self.is_a(&object.class, class)),
                        "composite drawable intrinsic size unsupported"
                    );
                    -1
                };
                result.push(Word::from(size));
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
            (
                "Landroid/graphics/drawable/Drawable;",
                "mutate()Landroid/graphics/drawable/Drawable;",
            ) => {
                self.heap.get(receiver)?;
                result.push(receiver);
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "setCallback(Landroid/graphics/drawable/Drawable$Callback;)V",
            ) => {
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
                let visible = Word::from(i32::from(arg(1)?.int()? != 0));
                arg(2)?.int()?;
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
                if previous.truth() != visible.truth() {
                    let roots = self.native_roots.len();
                    self.native_roots.push(receiver);
                    let changed = self.invoke(
                        Method {
                            class: "Landroid/graphics/drawable/Drawable;".into(),
                            name: "invalidateSelf".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    changed?;
                }
                result.push(Word::from(i32::from(previous.truth() != visible.truth())));
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
                    fields.insert(format!("Landroid/graphics/Rect;->{name}:I"), vec![value]);
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:bounds".into(), vec![bounds]);
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "getConstantState()Landroid/graphics/drawable/Drawable$ConstantState;",
            ) => {
                result.push(Word::ZERO);
            }
            ("Landroid/graphics/drawable/Drawable;", "getChangingConfigurations()I") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:drawable:configurations")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/graphics/drawable/Drawable;", "setChangingConfigurations(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:configurations".into(), vec![arg(1)?]);
            }
            ("Landroid/graphics/drawable/Drawable;", "setTint(I)V") => {
                let colors = self
                    .invoke(
                        Method {
                            class: "Landroid/content/res/ColorStateList;".into(),
                            name: "valueOf".into(),
                            parameters: vec!["I".into()],
                            returns: "Landroid/content/res/ColorStateList;".into(),
                        },
                        vec![arg(1)?],
                        false,
                    )?
                    .first()
                    .copied()
                    .context("missing tint color list")?;
                let roots = self.native_roots.len();
                self.native_roots.extend([receiver, colors]);
                let tinted = self.invoke(
                    Method {
                        class: "Landroid/graphics/drawable/Drawable;".into(),
                        name: "setTintList".into(),
                        parameters: vec!["Landroid/content/res/ColorStateList;".into()],
                        returns: "V".into(),
                    },
                    vec![receiver, colors],
                    true,
                );
                self.native_roots.truncate(roots);
                tinted?;
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "setTintList(Landroid/content/res/ColorStateList;)V"
                | "setTintMode(Landroid/graphics/PorterDuff$Mode;)V",
            ) => {
                let value = arg(1)?;
                let (key, ty) = if method.name == "setTintList" {
                    (
                        "droidless:drawable:tint-list",
                        "Landroid/content/res/ColorStateList;",
                    )
                } else {
                    (
                        "droidless:drawable:tint-mode",
                        "Landroid/graphics/PorterDuff$Mode;",
                    )
                };
                ensure!(
                    value == Word::ZERO || self.is_a(&self.heap.get(value)?.class, ty),
                    "invalid drawable tint value"
                );
                // Tint metadata is retained; native compound-icon painting remains outside this profile.
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(key.into(), vec![value]);
            }
            ("Landroid/graphics/drawable/Drawable;", "clearColorFilter()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .remove("droidless:drawable:color-filter");
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "setColorFilter(ILandroid/graphics/PorterDuff$Mode;)V",
            ) => {
                self.heap.get(receiver)?;
                self.heap.get(arg(2)?)?;
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:drawable:color-filter".into(),
                    vec![arg(1)?, arg(2)?],
                );
            }
            (
                "Landroid/graphics/drawable/Drawable;",
                "setColorFilter(Landroid/graphics/ColorFilter;)V",
            ) => {
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
                let cached = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:color-state-list:default")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(color) = cached {
                    color
                } else {
                    let colors = self.color_states(receiver)?;
                    let default = colors
                        .iter()
                        .rev()
                        .find(|(states, _)| states.is_empty())
                        .or_else(|| colors.first())
                        .context("empty color selector")?;
                    Word::from(default.1)
                });
            }
            ("Landroid/content/res/ColorStateList;", "<init>([[I[I)V") => {
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:color-state-list:arrays".into(),
                    vec![arg(1)?, arg(2)?],
                );
                let colors = self.color_states(receiver)?;
                let default = colors
                    .iter()
                    .rev()
                    .find(|(states, _)| states.is_empty())
                    .or_else(|| colors.first())
                    .map_or(0xffff0000u32 as i32, |(_, color)| *color);
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:color-state-list:default".into(),
                    vec![Word::from(default)],
                );
            }
            ("Landroid/content/res/ColorStateList;", "getColorForState([II)I") => {
                let states = if arg(1)? == Word::ZERO {
                    None
                } else {
                    Some(self.color_int_array(arg(1)?)?)
                };
                let colors = self.color_states(receiver)?;
                let matched = colors.iter().find(|(required, _)| {
                    crate::drawables::state_matches(required, states.as_deref())
                });
                result.push(Word::from(
                    matched.map_or(arg(2)?.int()?, |(_, color)| *color),
                ));
            }
            ("Landroid/content/res/ColorStateList;", "isStateful()Z") => {
                result.push(Word::from(i32::from(
                    self.color_states(receiver)?.len() > 1,
                )));
            }
            (
                "Landroid/content/res/Resources;",
                "getColorStateList(I)Landroid/content/res/ColorStateList;",
            ) => {
                result.push(self.color_state_list(&Value {
                    kind: 1,
                    data: arg(1)?.int()? as u32,
                    text: None,
                })?);
            }
            ("Landroid/graphics/Color;", "colorToHSV(I[F)V") => {
                let hsv = rgb_to_hsv(arg(0)?.int()? as u32);
                let array = arg(1)?;
                let Data::Array { values, element } = &mut self.heap.get_mut(array)?.data else {
                    bail!("Color.colorToHSV requires a float array");
                };
                ensure!(
                    element == "F" && values.len() >= 3,
                    "Color.colorToHSV requires a float array of length 3"
                );
                for (value, component) in values.iter_mut().zip(hsv) {
                    *value = vec![Word::Bits(component.to_bits())];
                }
            }
            ("Landroid/graphics/Color;", "alpha(I)I" | "red(I)I" | "green(I)I" | "blue(I)I") => {
                let shift = match method.name.as_str() {
                    "alpha" => 24,
                    "red" => 16,
                    "green" => 8,
                    _ => 0,
                };
                result.push(Word::from(
                    (((arg(0)?.int()? as u32) >> shift) & 255) as i32,
                ));
            }
            ("Landroid/graphics/Color;", "rgb(III)I" | "argb(IIII)I") => {
                let (alpha, offset) = if method.name == "argb" {
                    (arg(0)?.int()? as u32, 1)
                } else {
                    (255, 0)
                };
                // API 21 shifts raw Java ints; it does not clamp or mask input components.
                let color = (alpha << 24)
                    | ((arg(offset)?.int()? as u32) << 16)
                    | ((arg(offset + 1)?.int()? as u32) << 8)
                    | (arg(offset + 2)?.int()? as u32);
                result.push(Word::Bits(color));
            }
            (
                "Landroid/content/res/ColorStateList;",
                "valueOf(I)Landroid/content/res/ColorStateList;",
            ) => {
                let color_list = self.heap.instance("Landroid/content/res/ColorStateList;")?;
                self.heap
                    .get_mut(color_list)?
                    .fields
                    .insert("droidless:color-state-list:default".into(), vec![arg(0)?]);
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
                result.push(Word::from(
                    ((size & 0x3fff_ffff) | (mode & 0xc000_0000)) as i32,
                ));
            }
            ("Landroid/view/View$MeasureSpec;", "getMode(I)I") => {
                result.push(Word::from(((arg(0)?.int()? as u32) & 0xc000_0000) as i32));
            }
            ("Landroid/view/View$MeasureSpec;", "getSize(I)I") => {
                result.push(Word::from(arg(0)?.int()? & 0x3fff_ffff));
            }
            (
                "Ljava/util/ResourceBundle;",
                "getBundle(Ljava/lang/String;Ljava/util/Locale;)Ljava/util/ResourceBundle;",
            ) => {
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
                        format!(
                            "resource bundle key {:?} was not found",
                            self.heap.text(key)?
                        ),
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
                        self.heap
                            .get_mut(locale)?
                            .fields
                            .insert(format!("droidless:locale:{field}"), vec![value]);
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
                let language = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:locale:language")
                    .and_then(|v| v.first())
                    .copied()
                    .context("uninitialized Locale")?;
                let country = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:locale:country")
                    .and_then(|v| v.first())
                    .copied()
                    .context("uninitialized Locale")?;
                let language = self.heap.text(language)?;
                let country = self.heap.text(country)?;
                result.push(self.heap.string(if country.is_empty() {
                    language.into()
                } else {
                    format!("{language}_{country}")
                })?);
            }
            ("Ljava/lang/String;", "toString()Ljava/lang/String;") => {
                self.heap.text(receiver)?;
                result.push(receiver);
            }
            ("Ljava/lang/String;", "length()I") => result.push(Word::from(
                self.heap.text(receiver)?.encode_utf16().count() as i32,
            )),
            ("Ljava/lang/String;", "hashCode()I") => result.push(Word::from(
                self.heap
                    .text(receiver)?
                    .encode_utf16()
                    .fold(0i32, |hash, unit| {
                        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
                    }),
            )),
            ("Ljava/lang/String;", "isEmpty()Z") => {
                result.push(Word::from(i32::from(self.heap.text(receiver)?.is_empty())))
            }
            ("Ljava/lang/String;", "trim()Ljava/lang/String;") => {
                let value = self
                    .heap
                    .text(receiver)?
                    .trim_matches(|character| character <= '\u{20}')
                    .to_owned();
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
                    self.heap
                        .text(receiver)?
                        .ends_with(self.heap.text(arg(1)?)?),
                )))
            }
            (
                "Ljava/lang/String;",
                "lastIndexOf(I)I"
                | "lastIndexOf(II)I"
                | "indexOf(I)I"
                | "indexOf(II)I"
                | "lastIndexOf(Ljava/lang/String;)I"
                | "lastIndexOf(Ljava/lang/String;I)I"
                | "indexOf(Ljava/lang/String;)I"
                | "indexOf(Ljava/lang/String;I)I",
            ) => {
                let source = self.heap.text(receiver)?.encode_utf16().collect::<Vec<_>>();
                let needle = if method.parameters[0] == "I" {
                    let code = arg(1)?.int()?;
                    if (0..=0xffff).contains(&code) {
                        vec![code as u16]
                    } else {
                        let Some(character) =
                            u32::try_from(arg(1)?.int()?).ok().and_then(char::from_u32)
                        else {
                            return Ok(Some(vec![Word::from(-1)]));
                        };
                        let mut buffer = [0; 2];
                        character.encode_utf16(&mut buffer).to_vec()
                    }
                } else {
                    self.heap.text(arg(1)?)?.encode_utf16().collect::<Vec<_>>()
                };
                let forward = method.name == "indexOf";
                let from = if method.parameters.len() == 2 {
                    arg(2)?.int()?
                } else if forward {
                    0
                } else {
                    i32::MAX
                };
                let matches = |index: &usize| {
                    (if forward {
                        *index >= (from.max(0) as usize).min(source.len())
                    } else {
                        *index as i64 <= i64::from(from)
                    }) && source.get(*index..index.saturating_add(needle.len()))
                        == Some(needle.as_slice())
                };
                let found = if forward {
                    (0..=source.len()).find(matches)
                } else {
                    (0..=source.len()).rev().find(matches)
                };
                result.push(Word::from(found.map_or(-1, |index| index as i32)));
            }
            ("Ljava/lang/String;", "contains(Ljava/lang/CharSequence;)Z") => {
                result.push(Word::from(i32::from(
                    self.heap.text(receiver)?.contains(self.heap.text(arg(1)?)?),
                )))
            }
            (
                "Ljava/lang/String;",
                "replace(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;",
            ) => {
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
                let value = object.fields.get("value").context("uninitialized Number")?;
                let value = match object.class.as_str() {
                    "Ljava/lang/Byte;"
                    | "Ljava/lang/Character;"
                    | "Ljava/lang/Short;"
                    | "Ljava/lang/Integer;" => value[0].int()?,
                    "Ljava/lang/Long;" => bits64(value)? as i64 as i32,
                    "Ljava/lang/Float;" => f32::from_bits(value[0].int()? as u32) as i32,
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
            ("Ljava/lang/Double;", "doubleValue()D") | ("Ljava/lang/Float;", "floatValue()F") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .context("uninitialized floating-point wrapper")?
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
            ("Ljava/lang/Float;", "parseFloat(Ljava/lang/String;)F") => {
                let value = java_decimal_literal(self.heap.text(arg(0)?)?)?
                    .parse::<f32>()
                    .map_err(|_| {
                        fault(
                            "Ljava/lang/NumberFormatException;",
                            "invalid floating-point string",
                        )
                    })?;
                result.push(Word::Bits(value.to_bits()));
            }
            ("Ljava/lang/Double;", "parseDouble(Ljava/lang/String;)D") => {
                let value = java_decimal_literal(self.heap.text(arg(0)?)?)?
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
                if capacity < 0 {
                    return Err(fault(
                        "Ljava/lang/NegativeArraySizeException;",
                        capacity.to_string(),
                    ));
                }
                ensure!(
                    capacity <= 1_048_576,
                    "StringBuilder capacity limit reached"
                );
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
            | (
                "Ljava/lang/StringBuilder;",
                "append(Ljava/lang/Object;)Ljava/lang/StringBuilder;",
            )
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
                        self.heap
                            .text(
                                *string
                                    .first()
                                    .context("String.valueOf returned no result")?,
                            )?
                            .to_owned()
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
            ("Ljava/lang/Math;", "abs(F)F") => {
                result.push(Word::Bits(
                    f32::from_bits(arg(0)?.int()? as u32).abs().to_bits(),
                ));
            }
            ("Ljava/lang/Math;", "round(F)I") => {
                // Promote before adding: f32 addition misrounds the value just below 0.5.
                let value = f64::from(f32::from_bits(arg(0)?.int()? as u32));
                result.push(Word::from((value + 0.5).floor() as i32));
            }
            ("Ljava/lang/Math;", "max(II)I") => {
                result.push(Word::from(arg(0)?.int()?.max(arg(1)?.int()?)));
            }
            ("Ljava/lang/Math;", "max(FF)F") => {
                let left = f32::from_bits(arg(0)?.int()? as u32);
                let right = f32::from_bits(arg(1)?.int()? as u32);
                let value = if left.is_nan() {
                    left
                } else if left == 0.0 && right == 0.0 && right.is_sign_positive() {
                    right
                } else if left >= right {
                    left
                } else {
                    right
                };
                result.push(Word::Bits(value.to_bits()));
            }
            ("Ljava/lang/Math;", "min(FF)F") => {
                let left = f32::from_bits(arg(0)?.int()? as u32);
                let right = f32::from_bits(arg(1)?.int()? as u32);
                let value = if left.is_nan() {
                    left
                } else if left == 0.0 && right == 0.0 && right.is_sign_negative() {
                    right
                } else if left <= right {
                    left
                } else {
                    right
                };
                result.push(Word::Bits(value.to_bits()));
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
            | ("Landroid/app/Activity;", "onActivityResult(IILandroid/content/Intent;)V")
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
                let inflater = self.layout_inflater_from(receiver)?;
                let root = *self
                    .invoke(
                        Method {
                            class: "Landroid/view/LayoutInflater;".into(),
                            name: "inflate".into(),
                            parameters: vec![
                                "I".into(),
                                "Landroid/view/ViewGroup;".into(),
                                "Z".into(),
                            ],
                            returns: "Landroid/view/View;".into(),
                        },
                        vec![inflater, arg(1)?, Word::ZERO, Word::ZERO],
                        true,
                    )?
                    .first()
                    .context("Activity inflater returned no View")?;
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
                    Data::Attributes { named, .. } => named.len() as i32,
                    Data::XmlPull {
                        events, position, ..
                    } => {
                        let event = events
                            .get(*position)
                            .context("invalid XML parser position")?;
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
            (
                "Landroid/util/AttributeSet;",
                "getAttributeValue(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            ) => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                result.push(if let Some(value) = value {
                    self.heap.string(value.display())?
                } else {
                    Word::ZERO
                });
            }
            (
                "Landroid/util/AttributeSet;",
                "getAttributeResourceValue(Ljava/lang/String;Ljava/lang/String;I)I",
            ) => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                result.push(Word::from(
                    value
                        .filter(|value| value.kind == 1)
                        .map_or(arg(3)?.int()?, |value| value.data as i32),
                ));
            }
            (
                "Landroid/util/AttributeSet;",
                "getAttributeIntValue(Ljava/lang/String;Ljava/lang/String;I)I",
            ) => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                result.push(Word::from(
                    value
                        .and_then(|value| {
                            (value.kind == 0x10 || value.kind == 0x11)
                                .then_some(value.data as i32)
                                .or_else(|| value.text.and_then(|text| text.parse().ok()))
                        })
                        .unwrap_or(arg(3)?.int()?),
                ));
            }
            (
                "Landroid/util/AttributeSet;",
                "getAttributeBooleanValue(Ljava/lang/String;Ljava/lang/String;Z)Z",
            ) => {
                let value = self.attribute_set_value(receiver, self.heap.text(arg(2)?)?)?;
                let enabled = value
                    .and_then(|value| {
                        (value.kind == 0x12)
                            .then_some(value.data != 0)
                            .or_else(|| value.text.and_then(|text| text.parse().ok()))
                    })
                    .map_or(arg(3)?.int()? != 0, |value| value);
                result.push(Word::from(i32::from(enabled)));
            }
            ("Landroid/content/Context;", "getString(I)Ljava/lang/String;")
            | ("Landroid/content/res/Resources;", "getString(I)Ljava/lang/String;") => {
                let text = self.resource_text(arg(1)?.int()? as u32)?;
                result.push(self.heap.string(text)?);
            }
            ("Landroid/content/res/Resources;", "getResourceEntryName(I)Ljava/lang/String;") => {
                let id = arg(1)?.int()? as u32;
                // Names belong to the requested entry, including aliases and bags;
                // resolving its value would return the wrong alias name.
                let name = self
                    .apk
                    .resources
                    .entries
                    .get(&id)
                    .and_then(|entry| entry.name.split_once('/'))
                    .map(|(_, name)| name.to_owned())
                    .ok_or_else(|| {
                        fault(
                            "Landroid/content/res/Resources$NotFoundException;",
                            format!("Resource ID #0x{id:08x}"),
                        )
                    })?;
                result.push(self.heap.string(name)?);
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
            | (
                "Landroid/content/res/Resources;",
                "getAssets()Landroid/content/res/AssetManager;",
            ) => {
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
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:asset-manager".into(), vec![manager]);
                    manager
                });
            }
            (
                "Landroid/content/res/AssetManager;",
                "open(Ljava/lang/String;)Ljava/io/InputStream;",
            ) => {
                let name = self.heap.text(arg(1)?)?;
                ensure!(
                    !name.is_empty()
                        && name
                            .split('/')
                            .all(|part| !part.is_empty() && part != "." && part != "..")
                        && !name.contains(['\\', '\0']),
                    fault("Ljava/io/FileNotFoundException;", "invalid asset path")
                );
                let path = format!("assets/{name}");
                let bytes = self.apk.files.get(&path).cloned().ok_or_else(|| {
                    fault(
                        "Ljava/io/FileNotFoundException;",
                        format!("asset not found: {name}"),
                    )
                })?;
                let stream = self.heap.instance("Ljava/io/FileInputStream;")?;
                self.heap.get_mut(stream)?.data = Data::ByteStream {
                    bytes,
                    position: 0,
                    closed: false,
                };
                result.push(stream);
            }
            (
                "Landroid/content/res/AssetManager;",
                "list(Ljava/lang/String;)[Ljava/lang/String;",
            ) => {
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
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:tree-observer".into(), vec![observer]);
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
                    let theme = self
                        .heap
                        .instance("Landroid/content/res/Resources$Theme;")?;
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
            (
                "Landroid/content/res/Resources;",
                "newTheme()Landroid/content/res/Resources$Theme;",
            ) => {
                let theme = self
                    .heap
                    .instance("Landroid/content/res/Resources$Theme;")?;
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
                }
                self.heap
                    .get_mut(theme)?
                    .fields
                    .insert("droidless:theme:styles".into(), vec![]);
                result.push(theme);
            }
            (
                "Landroid/content/res/Resources;",
                "getDisplayMetrics()Landroid/util/DisplayMetrics;",
            ) => {
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
            (
                "Landroid/content/res/Resources;",
                "getConfiguration()Landroid/content/res/Configuration;",
            ) => {
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
                    let configuration =
                        self.heap.instance("Landroid/content/res/Configuration;")?;
                    let fields = &mut self.heap.get_mut(configuration)?.fields;
                    fields.insert(
                        "Landroid/content/res/Configuration;->orientation:I".into(),
                        vec![Word::from(1)],
                    );
                    fields.insert(
                        "Landroid/content/res/Configuration;->keyboard:I".into(),
                        vec![Word::from(2)],
                    );
                    fields.insert(
                        "Landroid/content/res/Configuration;->screenWidthDp:I".into(),
                        vec![Word::from(self.width as i32)],
                    );
                    fields.insert(
                        "Landroid/content/res/Configuration;->screenHeightDp:I".into(),
                        vec![Word::from(self.height as i32)],
                    );
                    fields.insert(
                        "Landroid/content/res/Configuration;->smallestScreenWidthDp:I".into(),
                        vec![Word::from(self.width as i32)],
                    );
                    fields.insert(
                        "Landroid/content/res/Configuration;->densityDpi:I".into(),
                        vec![Word::from(160)],
                    );
                    fields.insert(
                        "Landroid/content/res/Configuration;->fontScale:F".into(),
                        vec![Word::Bits(1.0f32.to_bits())],
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:configuration".into(), vec![configuration]);
                    configuration
                });
            }
            ("Landroid/content/res/Configuration;", "<init>()V") => {
                self.heap.get_mut(receiver)?.fields.insert(
                    "Landroid/content/res/Configuration;->fontScale:F".into(),
                    vec![Word::Bits(1.0f32.to_bits())],
                );
            }
            (
                "Landroid/content/res/Configuration;",
                "<init>(Landroid/content/res/Configuration;)V",
            ) => {
                ensure!(
                    self.is_a(
                        &self.heap.get(arg(1)?)?.class,
                        "Landroid/content/res/Configuration;"
                    ),
                    "invalid Configuration source"
                );
                self.heap.get_mut(receiver)?.fields = self.heap.get(arg(1)?)?.fields.clone();
            }
            (
                "Landroid/content/res/Configuration;",
                "equals(Landroid/content/res/Configuration;)Z" | "equals(Ljava/lang/Object;)Z",
            ) => {
                let other = arg(1)?;
                let equal = if other == Word::ZERO
                    || !self.is_a(
                        &self.heap.get(other)?.class,
                        "Landroid/content/res/Configuration;",
                    ) {
                    false
                } else {
                    let values = |object: Word| -> Result<Vec<i32>> {
                        let fields = &self.heap.get(object)?.fields;
                        [
                            ("orientation", "I"),
                            ("keyboard", "I"),
                            ("screenWidthDp", "I"),
                            ("screenHeightDp", "I"),
                            ("smallestScreenWidthDp", "I"),
                            ("densityDpi", "I"),
                            ("fontScale", "F"),
                        ]
                        .iter()
                        .map(|(name, ty)| {
                            let key = format!("Landroid/content/res/Configuration;->{name}:{ty}");
                            fields
                                .get(&key)
                                .and_then(|values| values.first())
                                .copied()
                                .unwrap_or(Word::ZERO)
                                .int()
                        })
                        .collect()
                    };
                    let a = values(receiver)?;
                    let b = values(other)?;
                    let fa = f32::from_bits(a[6] as u32);
                    let fb = f32::from_bits(b[6] as u32);
                    a[..6] == b[..6]
                        && matches!(fa.partial_cmp(&fb), None | Some(std::cmp::Ordering::Equal))
                };
                // ponytail: compare the seven profile fields; locale and full qualifier configuration remain unsupported.
                result.push(Word::from(i32::from(equal)));
            }
            ("Landroid/content/res/Resources;", "getBoolean(I)Z") => {
                let value = self.apk.resources.resolve(arg(1)?.int()? as u32)?;
                result.push(Word::from(i32::from(value.data != 0)));
            }
            ("Landroid/content/res/Resources;", "getColor(I)I")
            | (
                "Landroid/content/res/Resources;",
                "getColor(ILandroid/content/res/Resources$Theme;)I",
            )
            | ("Landroid/content/Context;", "getColor(I)I") => {
                let id = arg(1)?.int()? as u32;
                ensure!(id != 0, "resource @0x00000000 missing or complex");
                let value = self.attribute(&Value {
                    kind: 1,
                    data: id,
                    text: None,
                })?;
                ensure!(
                    (0x1c..=0x1f).contains(&value.kind),
                    "resource is not a color"
                );
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
                result.push(Word::from(dimension_pixel_size(
                    self.apk.resources.resolve(arg(1)?.int()? as u32)?,
                )?));
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
                fields.insert(
                    format!("{class}->type:I"),
                    vec![Word::from(i32::from(value.kind))],
                );
                fields.insert(
                    format!("{class}->data:I"),
                    vec![Word::from(value.data as i32)],
                );
                let asset_cookie = if value.kind == 1 && value.data == id && id >> 24 == 1 {
                    0
                } else {
                    1
                };
                fields.insert(
                    format!("{class}->assetCookie:I"),
                    vec![Word::from(asset_cookie)],
                );
                fields.insert(
                    format!("{class}->resourceId:I"),
                    vec![Word::from(id as i32)],
                );
                fields.insert(
                    format!("{class}->changingConfigurations:I"),
                    vec![Word::ZERO],
                );
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
            ("Landroid/content/Context;", "getDrawable(I)Landroid/graphics/drawable/Drawable;") => {
                let id = arg(1)?;
                let roots = self.native_roots.len();
                self.native_roots.push(receiver);
                let loaded = (|| -> Result<Vec<Word>> {
                    let resources = *self
                        .invoke(
                            Method {
                                class: "Landroid/content/Context;".into(),
                                name: "getResources".into(),
                                parameters: vec![],
                                returns: "Landroid/content/res/Resources;".into(),
                            },
                            vec![receiver],
                            true,
                        )?
                        .first()
                        .context("Context resources missing")?;
                    self.native_roots.push(resources);
                    let theme = *self
                        .invoke(
                            Method {
                                class: "Landroid/content/Context;".into(),
                                name: "getTheme".into(),
                                parameters: vec![],
                                returns: "Landroid/content/res/Resources$Theme;".into(),
                            },
                            vec![receiver],
                            true,
                        )?
                        .first()
                        .context("Context theme missing")?;
                    self.native_roots.push(theme);
                    self.invoke(
                        Method {
                            class: "Landroid/content/res/Resources;".into(),
                            name: "getDrawable".into(),
                            parameters: vec![
                                "I".into(),
                                "Landroid/content/res/Resources$Theme;".into(),
                            ],
                            returns: "Landroid/graphics/drawable/Drawable;".into(),
                        },
                        vec![resources, id, theme],
                        true,
                    )
                })();
                self.native_roots.truncate(roots);
                return Ok(Some(loaded?));
            }
            (
                "Landroid/content/res/Resources;",
                "getDrawable(I)Landroid/graphics/drawable/Drawable;",
            )
            | (
                "Landroid/content/res/Resources;",
                "getDrawable(ILandroid/content/res/Resources$Theme;)Landroid/graphics/drawable/Drawable;",
            ) => {
                let id = arg(1)?;
                let drawable = self.heap.instance("Landroid/graphics/drawable/Drawable;")?;
                self.heap
                    .get_mut(drawable)?
                    .fields
                    .insert("resourceId".into(), vec![id]);
                if let Some((info, bytes)) = self.image_resource(id.int()? as u32)? {
                    let bitmap = self.bitmap(info, bytes, None)?;
                    self.heap
                        .get_mut(drawable)?
                        .fields
                        .insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                }
                result.push(drawable);
            }
            ("Landroid/content/res/Resources$Theme;", "applyStyle(IZ)V") => {
                let style = arg(1)?;
                let force = arg(2)?.truth();
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                let styles = fields.entry("droidless:theme:styles".into()).or_default();
                if force {
                    styles.push(style);
                } else {
                    styles.insert(0, style);
                }
            }
            (
                "Landroid/content/res/Resources$Theme;",
                "setTo(Landroid/content/res/Resources$Theme;)V",
            ) => {
                let source = arg(1)?;
                ensure!(
                    self.is_a(
                        &self.heap.get(receiver)?.class,
                        "Landroid/content/res/Resources$Theme;"
                    ),
                    "Theme.setTo expects Theme receiver"
                );
                ensure!(
                    self.is_a(
                        &self.heap.get(source)?.class,
                        "Landroid/content/res/Resources$Theme;"
                    ),
                    "Theme.setTo expects Theme"
                );
                let styles = self
                    .heap
                    .get(source)?
                    .fields
                    .get("droidless:theme:styles")
                    .cloned()
                    .unwrap_or_default();
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:theme:styles".into(), styles);
            }
            ("Landroid/content/res/TypedArray;", sig)
                if [
                    "getBoolean(IZ)Z",
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
                    args.get(1)
                        .copied()
                        .map(Word::int)
                        .transpose()?
                        .and_then(|index| usize::try_from(index).ok())
                };
                let (length, value) = match &self.heap.get(receiver)?.data {
                    Data::TypedArray(values) => (
                        values.len(),
                        index.and_then(|index| values.get(index).cloned().flatten()),
                    ),
                    _ => bail!("uninitialized TypedArray"),
                };
                match sig {
                    "getBoolean(IZ)Z" => result.push(
                        value
                            .as_ref()
                            .map_or(arg(2)?.int()?, |value| i32::from(value.data != 0))
                            .into(),
                    ),
                    "getInt(II)I" | "getInteger(II)I" => {
                        result.push(Word::from(
                            value.map_or(arg(2)?.int()?, |value| value.data as i32),
                        ));
                    }
                    "getLayoutDimension(II)I" => {
                        let fallback = arg(2)?.int()?;
                        let pixels = if let Some(value) = value {
                            let value =
                                self.attribute(&self.themed_attribute(receiver, &value)?)?;
                            match value.kind {
                                0x10..=0x1f => value.data as i32,
                                5 => dimension_pixel_size(&value)?,
                                _ => fallback,
                            }
                        } else {
                            fallback
                        };
                        result.push(Word::from(pixels));
                    }
                    "getResourceId(II)I" => {
                        result.push(Word::from(
                            value
                                .filter(|value| value.kind == 1)
                                .map_or(arg(2)?.int()?, |value| value.data as i32),
                        ));
                    }
                    "getDimension(IF)F" | "getFloat(IF)F" => {
                        let number = if let Some(value) = value {
                            let value =
                                self.attribute(&self.themed_attribute(receiver, &value)?)?;
                            if sig.starts_with("getDimension") {
                                dimension(&value)?
                            } else {
                                match value.kind {
                                    4 => f32::from_bits(value.data),
                                    0x10..=0x1f => value.data as i32 as f32,
                                    3 => value
                                        .text
                                        .as_deref()
                                        .context("missing float text")?
                                        .parse()?,
                                    _ => bail!("TypedArray value is not a float"),
                                }
                            }
                        } else {
                            f32::from_bits(arg(2)?.int()? as u32)
                        };
                        result.push(Word::Bits(number.to_bits()));
                    }
                    "getDimensionPixelOffset(II)I" | "getDimensionPixelSize(II)I" => {
                        let pixels = if let Some(value) = value {
                            let value =
                                self.attribute(&self.themed_attribute(receiver, &value)?)?;
                            if sig.starts_with("getDimensionPixelSize") {
                                dimension_pixel_size(&value)?
                            } else {
                                dimension(&value)?.trunc() as i32
                            }
                        } else {
                            arg(2)?.int()?
                        };
                        result.push(Word::from(pixels));
                    }
                    "getString(I)Ljava/lang/String;" | "getText(I)Ljava/lang/CharSequence;" => {
                        if let Some(value) = value {
                            if let Some(text) = value.text {
                                result.push(self.heap.string(text)?);
                            } else if value.kind == 1 {
                                result.push(self.heap.string(self.resource_text(value.data)?)?);
                            } else {
                                result.push(Word::ZERO);
                            }
                        } else {
                            result.push(Word::ZERO);
                        }
                    }
                    "getDrawable(I)Landroid/graphics/drawable/Drawable;" => {
                        result.push(if let Some(value) = value.filter(|value| value.kind == 1) {
                            let class = "Landroid/graphics/drawable/Drawable;";
                            let object = self.heap.instance(class)?;
                            self.heap
                                .get_mut(object)?
                                .fields
                                .insert("resourceId".into(), vec![Word::from(value.data as i32)]);
                            if class == "Landroid/graphics/drawable/Drawable;"
                                && let Some((info, bytes)) = self.image_resource(value.data)?
                            {
                                let bitmap = self.bitmap(info, bytes, None)?;
                                self.heap
                                    .get_mut(object)?
                                    .fields
                                    .insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                            }
                            object
                        } else {
                            Word::ZERO
                        });
                    }
                    "getTextArray(I)[Ljava/lang/CharSequence;" => result.push(Word::ZERO),
                    "getValue(ILandroid/util/TypedValue;)Z" => {
                        let output = arg(2)?;
                        ensure!(
                            index.is_some_and(|index| index < length),
                            fault(
                                "Ljava/lang/ArrayIndexOutOfBoundsException;",
                                "typed value index"
                            )
                        );
                        let mut value = value
                            .map(|value| self.themed_attribute(receiver, &value))
                            .transpose()?;
                        let mut resource_id = 0;
                        let mut seen = std::collections::BTreeSet::new();
                        while let Some(current) =
                            value.as_ref().filter(|v| v.kind == 1 && v.data != 0)
                        {
                            ensure!(
                                seen.len() < 32 && seen.insert(current.data),
                                "typed value resource reference cycle or depth limit"
                            );
                            resource_id = current.data;
                            value = Some(
                                if let Some(entry) = self.apk.resources.entries.get(&resource_id) {
                                    if let Some(next) = &entry.value {
                                        self.themed_attribute(receiver, next)?
                                    } else {
                                        break; // Complex resources remain references, as in the API-21 resource model.
                                    }
                                } else {
                                    self.attribute(current)?
                                },
                            );
                        }
                        if let Some(value) =
                            value.filter(|v| v.kind != 0 && !(v.kind == 1 && v.data == 0))
                        {
                            let class = "Landroid/util/TypedValue;";
                            ensure!(
                                self.is_a(&self.heap.get(output)?.class, class),
                                "getValue expects TypedValue output"
                            );
                            let string = if value.kind == 3 {
                                self.heap.string(value.display())?
                            } else {
                                Word::ZERO
                            };
                            let fields = &mut self.heap.get_mut(output)?.fields;
                            // ponytail: one virtual resource pool/default configuration; qualifier provenance requires parser metadata.
                            for (name, data) in [
                                ("type", i32::from(value.kind)),
                                ("data", value.data as i32),
                                ("resourceId", resource_id as i32),
                                ("assetCookie", 1),
                                ("changingConfigurations", 0),
                                ("density", 0),
                            ] {
                                fields.insert(format!("{class}->{name}:I"), vec![Word::from(data)]);
                            }
                            fields.insert(
                                format!("{class}->string:Ljava/lang/CharSequence;"),
                                vec![string],
                            );
                            result.push(Word::from(1));
                        } else {
                            result.push(Word::ZERO);
                        }
                    }
                    "hasValue(I)Z" => result.push(Word::from(i32::from(value.is_some()))),
                    "length()I" | "getIndexCount()I" => result.push(Word::from(length as i32)),
                    "getPositionDescription()Ljava/lang/String;" => {
                        result.push(self.heap.string("TypedArray".into())?)
                    }
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
            ("Landroid/view/WindowManager;", "getDefaultDisplay()Landroid/view/Display;") => {
                result.push(self.heap.instance("Landroid/view/Display;")?)
            }
            ("Landroid/util/DisplayMetrics;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/graphics/Rect;", "<init>()V" | "setEmpty()V") => {
                for edge in ["left", "top", "right", "bottom"] {
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("Landroid/graphics/Rect;->{edge}:I"),
                        vec![Word::ZERO],
                    );
                }
            }
            ("Landroid/util/StateSet;", "trimStateSet([II)[I") => {
                let states = arg(0)?;
                let Data::Array { element, values } = &self.heap.get(states)?.data else {
                    bail!("StateSet requires an int array")
                };
                ensure!(element == "I", "StateSet requires an int array");
                let size = arg(1)?.int()?;
                ensure!(
                    size >= 0,
                    fault(
                        "Ljava/lang/NegativeArraySizeException;",
                        "negative StateSet size"
                    )
                );
                let size = size as usize;
                ensure!(
                    size <= values.len(),
                    fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "StateSet size exceeds array length"
                    )
                );
                if size == values.len() {
                    result.push(states);
                } else {
                    let values = values[..size].to_vec();
                    let trimmed = self.array("I".into(), size)?;
                    self.heap.get_mut(trimmed)?.data = Data::Array {
                        element: "I".into(),
                        values,
                    };
                    result.push(trimmed);
                }
            }
            ("Landroid/graphics/RectF;", "<init>()V") => {
                for edge in ["left", "top", "right", "bottom"] {
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("Landroid/graphics/RectF;->{edge}:F"),
                        vec![Word::ZERO],
                    );
                }
            }
            (
                "Landroid/content/res/Resources;",
                "<init>(Landroid/content/res/AssetManager;Landroid/util/DisplayMetrics;Landroid/content/res/Configuration;)V",
            ) => {
                for (name, value) in [
                    ("droidless:resources:assets", arg(1)?),
                    ("droidless:resources:display-metrics", arg(2)?),
                    ("droidless:resources:configuration", arg(3)?),
                ] {
                    self.heap.get(value)?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(name.into(), vec![value]);
                }
            }
            (
                "Landroid/graphics/Paint;" | "Landroid/text/TextPaint;",
                "<init>()V" | "<init>(I)V",
            ) => {
                let flags = if method.parameters.is_empty() {
                    0
                } else {
                    arg(1)?.int()?
                };
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert(
                    "droidless:paint:flags".into(),
                    vec![Word::from(flags | 0x500)],
                );
                fields.insert(
                    "droidless:paint:color".into(),
                    vec![Word::from(0xff00_0000u32 as i32)],
                );
                fields.insert(
                    "droidless:paint:text-size".into(),
                    vec![Word::Bits(12.0f32.to_bits())],
                );
                if method.class == "Landroid/text/TextPaint;" {
                    fields.insert(
                        "Landroid/text/TextPaint;->density:F".into(),
                        vec![Word::Bits(1.0f32.to_bits())],
                    );
                }
            }
            ("Landroid/graphics/Paint;", "getFlags()I" | "setFlags(I)V") => {
                if method.name == "setFlags" {
                    let flags = Word::from(arg(1)?.int()?);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:paint:flags".into(), vec![flags]);
                } else {
                    result.push(
                        self.heap
                            .get(receiver)?
                            .fields
                            .get("droidless:paint:flags")
                            .and_then(|values| values.first())
                            .copied()
                            .unwrap_or(Word::from(0x500)),
                    );
                }
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
            ("Landroid/graphics/Paint;", "setShadowLayer(FFFI)V") => {
                ensure!(args.len() == 5, "invalid Paint shadow arguments");
                for value in &args[1..4] {
                    ensure!(
                        f32::from_bits(value.int()? as u32).is_finite(),
                        "non-finite Paint shadow"
                    );
                }
                // ponytail: retain Paint shadow state; Canvas shadow rasterization is still unsupported.
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:paint:shadow".into(), args[1..].to_vec());
            }
            ("Landroid/graphics/Paint;", "clearShadowLayer()V") => {
                ensure!(self.sync_depth < 32, "Paint callback nesting limit");
                self.invoke(
                    Method {
                        class: "Landroid/graphics/Paint;".into(),
                        name: "setShadowLayer".into(),
                        parameters: vec!["F".into(), "F".into(), "F".into(), "I".into()],
                        returns: "V".into(),
                    },
                    vec![receiver, Word::ZERO, Word::ZERO, Word::ZERO, Word::ZERO],
                    true,
                )?;
            }
            ("Landroid/graphics/Paint;", "hasShadowLayer()Z") => {
                let radius = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:paint:shadow")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(Word::from(i32::from(
                    f32::from_bits(radius.int()? as u32) > 0.0,
                )));
            }
            ("Landroid/graphics/Paint;", "setStyle(Landroid/graphics/Paint$Style;)V")
            | ("Landroid/graphics/Paint;", "setStrokeCap(Landroid/graphics/Paint$Cap;)V")
            | ("Landroid/graphics/Paint;", "setStrokeJoin(Landroid/graphics/Paint$Join;)V") => {
                let value = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(value)?.class, &method.parameters[0]),
                    "invalid Paint enum value"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("droidless:paint:{}", method.name), vec![value]);
            }
            ("Landroid/graphics/Paint;", "setStrokeWidth(F)V")
            | ("Landroid/graphics/Paint;", "setStrokeMiter(F)V") => {
                let value = arg(1)?;
                let number = f32::from_bits(value.int()? as u32);
                ensure!(
                    number.is_finite() && number >= 0.,
                    "invalid Paint stroke dimension"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("droidless:paint:{}", method.name), vec![value]);
            }
            ("Landroid/graphics/Rect;", "<init>(IIII)V")
            | ("Landroid/graphics/Rect;", "set(IIII)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for (name, value) in ["left", "top", "right", "bottom"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(format!("Landroid/graphics/Rect;->{name}:I"), vec![value]);
                }
            }
            ("Landroid/graphics/RectF;", "<init>(FFFF)V")
            | ("Landroid/graphics/RectF;", "set(FFFF)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for (name, value) in ["left", "top", "right", "bottom"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(format!("Landroid/graphics/RectF;->{name}:F"), vec![value]);
                }
            }
            ("Landroid/graphics/RectF;", "set(Landroid/graphics/Rect;)V") => {
                let original = self.heap.get(arg(1)?)?.fields.clone();
                for edge in ["left", "top", "right", "bottom"] {
                    let value = original
                        .get(&format!("Landroid/graphics/Rect;->{edge}:I"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()? as f32;
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("Landroid/graphics/RectF;->{edge}:F"),
                        vec![Word::Bits(value.to_bits())],
                    );
                }
            }
            ("Landroid/graphics/Rect;", "centerX()I" | "centerY()I" | "width()I" | "height()I") => {
                let (first, last) = if matches!(method.name.as_str(), "centerX" | "width") {
                    ("left", "right")
                } else {
                    ("top", "bottom")
                };
                let fields = &self.heap.get(receiver)?.fields;
                let edge = |name: &str| -> Result<i32> {
                    fields
                        .get(&format!("Landroid/graphics/Rect;->{name}:I"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()
                };
                result.push(Word::from(if method.name.starts_with("center") {
                    edge(first)?.wrapping_add(edge(last)?) >> 1
                } else {
                    edge(last)?.wrapping_sub(edge(first)?)
                }));
            }
            (
                "Landroid/graphics/Rect;",
                "contains(II)Z" | "intersects(IIII)Z" | "intersect(IIII)Z",
            ) => {
                let fields = &self.heap.get(receiver)?.fields;
                let edge = |name: &str| -> Result<i32> {
                    fields
                        .get(&format!("Landroid/graphics/Rect;->{name}:I"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()
                };
                let original = [edge("left")?, edge("top")?, edge("right")?, edge("bottom")?];
                if method.name == "contains" {
                    let (x, y) = (arg(1)?.int()?, arg(2)?.int()?);
                    result.push(Word::from(i32::from(
                        original[0] < original[2]
                            && original[1] < original[3]
                            && x >= original[0]
                            && x < original[2]
                            && y >= original[1]
                            && y < original[3],
                    )));
                    return Ok(Some(result));
                }
                let other = [
                    arg(1)?.int()?,
                    arg(2)?.int()?,
                    arg(3)?.int()?,
                    arg(4)?.int()?,
                ];
                let intersects = original[0] < other[2]
                    && other[0] < original[2]
                    && original[1] < other[3]
                    && other[1] < original[3];
                if intersects && method.name == "intersect" {
                    let intersection = [
                        original[0].max(other[0]),
                        original[1].max(other[1]),
                        original[2].min(other[2]),
                        original[3].min(other[3]),
                    ];
                    for (edge, value) in ["left", "top", "right", "bottom"]
                        .into_iter()
                        .zip(intersection)
                    {
                        self.heap.get_mut(receiver)?.fields.insert(
                            format!("Landroid/graphics/Rect;->{edge}:I"),
                            vec![Word::from(value)],
                        );
                    }
                }
                result.push(Word::from(i32::from(intersects)));
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
            | (
                "Landroid/content/ContextWrapper;",
                "attachBaseContext(Landroid/content/Context;)V",
            ) => {
                let context = arg(1)?;
                ensure!(
                    self.is_a(
                        &self.heap.get(receiver)?.class,
                        "Landroid/content/ContextWrapper;"
                    ),
                    "ContextWrapper expects wrapper receiver"
                );
                ensure!(
                    context == Word::ZERO
                        || self.is_a(&self.heap.get(context)?.class, "Landroid/content/Context;"),
                    "ContextWrapper expects Context"
                );
                if method.name == "attachBaseContext" {
                    ensure!(
                        self.heap
                            .get(receiver)?
                            .fields
                            .get("droidless:context:base")
                            .and_then(|words| words.first())
                            .is_none_or(|base| *base == Word::ZERO),
                        fault(
                            "Ljava/lang/IllegalStateException;",
                            "Base context already set"
                        )
                    );
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:context:base".into(), vec![context]);
            }
            ("Landroid/view/Gravity;", "getAbsoluteGravity(II)I") => {
                let mut gravity = arg(0)?.int()?;
                let rtl = arg(1)?.int()? == 1;
                if gravity & 0x0080_0000 != 0 {
                    if gravity & 0x0080_0003 == 0x0080_0003 {
                        gravity = (gravity & !0x0080_0003) | if rtl { 5 } else { 3 };
                    } else if gravity & 0x0080_0005 == 0x0080_0005 {
                        gravity = (gravity & !0x0080_0005) | if rtl { 3 } else { 5 };
                    }
                    gravity &= !0x0080_0000;
                }
                result.push(Word::from(gravity));
            }
            (
                "Landroid/view/Gravity;",
                "apply(IIILandroid/graphics/Rect;Landroid/graphics/Rect;)V"
                | "apply(IIILandroid/graphics/Rect;Landroid/graphics/Rect;I)V",
            ) => {
                for rect in [arg(3)?, arg(4)?] {
                    ensure!(
                        self.is_a(&self.heap.get(rect)?.class, "Landroid/graphics/Rect;"),
                        "Gravity requires Rect bounds"
                    );
                }
                let gravity = if args.len() == 6 {
                    self.invoke(
                        Method {
                            class: "Landroid/view/Gravity;".into(),
                            name: "getAbsoluteGravity".into(),
                            parameters: vec!["I".into(); 2],
                            returns: "I".into(),
                        },
                        vec![arg(0)?, arg(5)?],
                        false,
                    )?[0]
                        .int()?
                } else {
                    arg(0)?.int()?
                };
                let container = self.heap.get(arg(3)?)?.fields.clone();
                let edge = |name: &str| -> Result<i32> {
                    container
                        .get(&format!("Landroid/graphics/Rect;->{name}:I"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()
                };
                for (shift, start, end, size) in [
                    (0, "left", "right", arg(1)?.int()?),
                    (4, "top", "bottom", arg(2)?.int()?),
                ] {
                    let (low, high) = (edge(start)?, edge(end)?);
                    let pull = (gravity >> shift) & 6;
                    let (mut first, mut last) = match pull {
                        2 => (low, low.wrapping_add(size)),
                        4 => (high.wrapping_sub(size), high),
                        6 => (low, high),
                        _ => {
                            let first =
                                low.wrapping_add(high.wrapping_sub(low).wrapping_sub(size) / 2);
                            (first, first.wrapping_add(size))
                        }
                    };
                    if (gravity >> shift) & 8 != 0 {
                        if pull != 2 && pull != 6 {
                            first = first.max(low);
                        }
                        if pull != 4 && pull != 6 {
                            last = last.min(high);
                        }
                    }
                    let fields = &mut self.heap.get_mut(arg(4)?)?.fields;
                    fields.insert(
                        format!("Landroid/graphics/Rect;->{start}:I"),
                        vec![Word::from(first)],
                    );
                    fields.insert(
                        format!("Landroid/graphics/Rect;->{end}:I"),
                        vec![Word::from(last)],
                    );
                }
            }
            ("Landroid/content/Context;", "getClassLoader()Ljava/lang/ClassLoader;") => {
                self.heap.get(receiver)?;
                result.push(self.apk_class_loader()?);
            }
            ("Landroid/content/ContextWrapper;", "getBaseContext()Landroid/content/Context;") => {
                let context = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:context:base")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(context);
            }
            (
                "Landroid/view/ViewGroup$LayoutParams;"
                | "Landroid/view/ViewGroup$MarginLayoutParams;"
                | "Landroid/widget/FrameLayout$LayoutParams;"
                | "Landroid/widget/LinearLayout$LayoutParams;"
                | "Landroid/widget/TableLayout$LayoutParams;"
                | "Landroid/widget/TableRow$LayoutParams;",
                "<init>(Landroid/content/Context;Landroid/util/AttributeSet;)V",
            ) => {
                self.heap.get(arg(1)?)?;
                let attrs = arg(2)?;
                for edge in ["width", "height"] {
                    let table = matches!(
                        method.class.as_str(),
                        "Landroid/widget/TableLayout$LayoutParams;"
                            | "Landroid/widget/TableRow$LayoutParams;"
                    );
                    let raw = self.attribute_set_value(attrs, &format!("layout_{edge}"))?;
                    let value = if edge == "width"
                        && (method.class == "Landroid/widget/TableLayout$LayoutParams;"
                            || (table && raw.is_none()))
                    {
                        -1
                    } else if let Some(value) = raw {
                        dimension(&self.attribute(&value)?)? as i32
                    } else if table {
                        -2
                    } else {
                        bail!("layout_{edge} is required");
                    };
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("Landroid/view/ViewGroup$LayoutParams;->{edge}:I"),
                        vec![Word::from(value)],
                    );
                }
                if method.class != "Landroid/view/ViewGroup$LayoutParams;" {
                    let all = self.attribute_set_value(attrs, "layout_margin")?;
                    for edge in ["Left", "Top", "Right", "Bottom"] {
                        let value = if let Some(value) = &all {
                            Some(value.clone())
                        } else {
                            self.attribute_set_value(attrs, &format!("layout_margin{edge}"))?
                        };
                        let value = if let Some(value) = value {
                            dimension(&self.attribute(&value)?)? as i32
                        } else {
                            0
                        };
                        let name = format!("{}Margin", edge.to_ascii_lowercase());
                        self.heap.get_mut(receiver)?.fields.insert(
                            format!("Landroid/view/ViewGroup$MarginLayoutParams;->{name}:I"),
                            vec![Word::from(value)],
                        );
                    }
                }
                if matches!(
                    method.class.as_str(),
                    "Landroid/widget/FrameLayout$LayoutParams;"
                        | "Landroid/widget/LinearLayout$LayoutParams;"
                ) {
                    let gravity = self
                        .attribute_set_value(attrs, "layout_gravity")?
                        .map(|value| self.attribute(&value).map(|value| value.data as i32))
                        .transpose()?
                        .unwrap_or(-1);
                    self.heap.get_mut(receiver)?.fields.insert(
                        format!("{}->gravity:I", method.class),
                        vec![Word::from(gravity)],
                    );
                }
                if method.class == "Landroid/widget/LinearLayout$LayoutParams;" {
                    let weight = self
                        .attribute_set_value(attrs, "layout_weight")?
                        .map(|value| {
                            if value.kind == 4 {
                                f32::from_bits(value.data)
                            } else {
                                value.data as f32
                            }
                        })
                        .unwrap_or(0.0);
                    ensure!(weight.is_finite() && weight >= 0.0, "invalid layout weight");
                    self.heap.get_mut(receiver)?.fields.insert(
                        "Landroid/widget/LinearLayout$LayoutParams;->weight:F".into(),
                        vec![Word::Bits(weight.to_bits())],
                    );
                }
            }
            ("Landroid/view/ViewGroup$LayoutParams;", "<init>(II)V")
            | ("Landroid/widget/AbsListView$LayoutParams;", "<init>(II)V")
            | ("Landroid/view/ViewGroup$MarginLayoutParams;", "<init>(II)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->width:I".into(),
                    vec![arg(1)?],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->height:I".into(),
                    vec![arg(2)?],
                );
            }
            (
                "Landroid/view/ViewGroup$MarginLayoutParams;",
                "<init>(Landroid/view/ViewGroup$MarginLayoutParams;)V",
            ) => {
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
                fields.insert(
                    "Landroid/widget/LinearLayout$LayoutParams;->gravity:I".into(),
                    vec![Word::from(-1)],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->width:I".into(),
                    vec![arg(1)?],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->height:I".into(),
                    vec![arg(2)?],
                );
            }
            ("Landroid/widget/LinearLayout$LayoutParams;", "<init>(IIF)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->width:I".into(),
                    vec![arg(1)?],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->height:I".into(),
                    vec![arg(2)?],
                );
                fields.insert(
                    "Landroid/widget/LinearLayout$LayoutParams;->weight:F".into(),
                    vec![arg(3)?],
                );
            }
            ("Landroid/widget/FrameLayout$LayoutParams;", "<init>(II)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert(
                    "Landroid/widget/FrameLayout$LayoutParams;->gravity:I".into(),
                    vec![Word::from(-1)],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->width:I".into(),
                    vec![arg(1)?],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->height:I".into(),
                    vec![arg(2)?],
                );
            }
            ("Landroid/widget/FrameLayout$LayoutParams;", "<init>(III)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->width:I".into(),
                    vec![arg(1)?],
                );
                fields.insert(
                    "Landroid/view/ViewGroup$LayoutParams;->height:I".into(),
                    vec![arg(2)?],
                );
                fields.insert(
                    "Landroid/widget/FrameLayout$LayoutParams;->gravity:I".into(),
                    vec![arg(3)?],
                );
            }
            ("Landroid/view/ViewGroup$MarginLayoutParams;", "setMargins(IIII)V") => {
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                for (name, value) in ["leftMargin", "topMargin", "rightMargin", "bottomMargin"]
                    .into_iter()
                    .zip(args.iter().skip(1).copied())
                {
                    fields.insert(
                        format!("Landroid/view/ViewGroup$MarginLayoutParams;->{name}:I"),
                        vec![value],
                    );
                }
            }
            (
                "Landroid/view/View;" | "Landroid/view/ViewGroup;",
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
            ("Landroid/util/TypedValue;", "getFloat()F") => {
                let value = self.heap.get(receiver)?;
                ensure!(
                    self.is_a(&value.class, "Landroid/util/TypedValue;"),
                    "getFloat expects TypedValue"
                );
                let data = value
                    .fields
                    .get("Landroid/util/TypedValue;->data:I")
                    .and_then(|words| words.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(Word::Bits(data.int()? as u32));
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
                | "Landroid/widget/CheckedTextView;"
                | "Landroid/widget/Button;"
                | "Landroid/widget/EditText;"
                | "Landroid/widget/ImageView;"
                | "Landroid/widget/ImageButton;"
                | "Landroid/widget/RelativeLayout;"
                | "Landroid/widget/ScrollView;"
                | "Landroid/widget/HorizontalScrollView;"
                | "Landroid/widget/TableLayout;"
                | "Landroid/widget/TableRow;"
                | "Landroid/widget/Space;",
                "<init>(Landroid/content/Context;Landroid/util/AttributeSet;)V",
            )
            | (
                "Landroid/view/View;"
                | "Landroid/view/ViewGroup;"
                | "Landroid/widget/LinearLayout;"
                | "Landroid/widget/FrameLayout;"
                | "Landroid/widget/TextView;"
                | "Landroid/widget/CheckedTextView;"
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
            (
                "Landroid/view/ViewStub;",
                "<init>(Landroid/content/Context;)V"
                | "<init>(Landroid/content/Context;Landroid/util/AttributeSet;)V"
                | "<init>(Landroid/content/Context;Landroid/util/AttributeSet;I)V",
            ) => {
                self.heap.get(arg(1)?)?;
                self.view_mut(receiver)?.visible = 8;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:context".into(), vec![arg(1)?]);
            }
            ("Landroid/view/ViewStub;", "inflate()Landroid/view/View;") => {
                result.push(self.inflate_view_stub(receiver)?);
            }
            ("Landroid/view/ViewStub;", "getLayoutInflater()Landroid/view/LayoutInflater;") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:stub:inflater")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/ViewStub;", "setLayoutInflater(Landroid/view/LayoutInflater;)V") => {
                let inflater = arg(1)?;
                ensure!(
                    inflater == Word::ZERO
                        || self.is_a(
                            &self.heap.get(inflater)?.class,
                            "Landroid/view/LayoutInflater;"
                        ),
                    "invalid ViewStub inflater"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:stub:inflater".into(), vec![inflater]);
            }
            ("Landroid/view/ViewStub;", "setVisibility(I)V") => {
                let visibility = arg(1)?.int()?;
                ensure!([0, 4, 8].contains(&visibility), "invalid View visibility");
                let inflated = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:stub:inflated")
                    .and_then(|values| values.first())
                    .copied();
                if let Some(view) = inflated {
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "setVisibility".into(),
                            parameters: vec!["I".into()],
                            returns: "V".into(),
                        },
                        vec![view, arg(1)?],
                        true,
                    )?;
                } else {
                    self.view_mut(receiver)?.visible = visibility;
                    if visibility != 8 {
                        self.inflate_view_stub(receiver)?;
                    }
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
            (
                "Landroid/view/ViewConfiguration;",
                "get(Landroid/content/Context;)Landroid/view/ViewConfiguration;",
            ) => {
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
                    self.statics
                        .insert("droidless:view-configuration".into(), vec![configuration]);
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
            (
                "Landroid/widget/OverScroller;",
                "<init>(Landroid/content/Context;Landroid/view/animation/Interpolator;)V",
            ) => {
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
            | ("Landroid/widget/CheckedTextView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/Button;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/ImageView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/ImageButton;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/RelativeLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/ScrollView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/HorizontalScrollView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/TableLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/TableRow;", "<init>(Landroid/content/Context;)V")
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
                self.view_mut(receiver)?.orientation = arg(1)?.int()?;
                self.request_view_layout(receiver)?;
            }
            ("Landroid/widget/LinearLayout;", "getOrientation()I") => {
                result.push(Word::from(self.view_mut(receiver)?.orientation));
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
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:hierarchy-listener".into(), vec![listener]);
            }
            (
                "Landroid/view/ViewGroup$MarginLayoutParams;",
                "getMarginStart()I" | "getMarginEnd()I",
            ) => {
                let edge = if method.name == "getMarginStart" {
                    "leftMargin"
                } else {
                    "rightMargin"
                };
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(&format!(
                            "Landroid/view/ViewGroup$MarginLayoutParams;->{edge}:I"
                        ))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/ViewGroup;", "getChildMeasureSpec(III)I") => {
                let spec = arg(0)?.int()? as u32;
                let mode = spec & 0xc000_0000;
                let available = ((spec & 0x3fff_ffff) as i32)
                    .saturating_sub(arg(1)?.int()?)
                    .max(0);
                let dimension = arg(2)?.int()?;
                let (size, mode) = if dimension >= 0 && mode != 0xc000_0000 {
                    (dimension as u32, 0x4000_0000)
                } else {
                    match (mode, dimension) {
                        (0x4000_0000, -1) => (available as u32, mode),
                        (0x4000_0000 | 0x8000_0000, -2 | -1) => (available as u32, 0x8000_0000),
                        _ => (0, 0),
                    }
                };
                result.push(Word::from(((size & 0x3fff_ffff) | mode) as i32));
            }
            (
                "Landroid/view/ViewGroup;",
                "generateLayoutParams(Landroid/util/AttributeSet;)Landroid/view/ViewGroup$LayoutParams;",
            ) => {
                self.view_mut(receiver)?;
                let class = self.heap.get(receiver)?.class.clone();
                let params_class = if self.is_a(&class, "Landroid/widget/TableLayout;") {
                    "Landroid/widget/TableLayout$LayoutParams;"
                } else if self.is_a(&class, "Landroid/widget/TableRow;") {
                    "Landroid/widget/TableRow$LayoutParams;"
                } else if self.is_a(&class, "Landroid/widget/FrameLayout;") {
                    "Landroid/widget/FrameLayout$LayoutParams;"
                } else if self.is_a(&class, "Landroid/widget/LinearLayout;") {
                    "Landroid/widget/LinearLayout$LayoutParams;"
                } else {
                    "Landroid/view/ViewGroup$LayoutParams;"
                };
                let context = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:context")
                    .and_then(|values| values.first())
                    .copied()
                    .context("ViewGroup has no Context")?;
                let params = self.new_instance(params_class)?;
                self.invoke(
                    Method {
                        class: params_class.into(),
                        name: "<init>".into(),
                        parameters: vec![
                            "Landroid/content/Context;".into(),
                            "Landroid/util/AttributeSet;".into(),
                        ],
                        returns: "V".into(),
                    },
                    vec![params, context, arg(1)?],
                    false,
                )?;
                result.push(params);
            }
            ("Landroid/view/ViewGroup;", "measureChildWithMargins(Landroid/view/View;IIII)V") => {
                let child = arg(1)?;
                let params = self
                    .heap
                    .get(child)?
                    .fields
                    .get("droidless:view:layout-params")
                    .and_then(|values| values.first())
                    .copied()
                    .context("child has no layout parameters")?;
                ensure!(
                    self.is_a(
                        &self.heap.get(params)?.class,
                        "Landroid/view/ViewGroup$MarginLayoutParams;"
                    ),
                    "child requires margin layout parameters"
                );
                self.measure_child(
                    receiver,
                    child,
                    [arg(2)?, arg(4)?],
                    [arg(3)?.int()?, arg(5)?.int()?],
                    [None; 2],
                )?;
            }
            (
                "Landroid/view/ViewGroup;",
                "checkLayoutParams(Landroid/view/ViewGroup$LayoutParams;)Z",
            ) => {
                let params = arg(1)?;
                result.push(Word::from(i32::from(
                    params != Word::ZERO
                        && self.is_a(
                            &self.heap.get(params)?.class,
                            "Landroid/view/ViewGroup$LayoutParams;",
                        ),
                )));
            }
            (
                "Landroid/view/ViewGroup;",
                "setLayoutTransition(Landroid/animation/LayoutTransition;)V",
            ) => {
                self.view_mut(receiver)?;
                let transition = arg(1)?;
                if transition != Word::ZERO {
                    ensure!(
                        self.is_a(
                            &self.heap.get(transition)?.class,
                            "Landroid/animation/LayoutTransition;"
                        ),
                        "invalid layout transition"
                    );
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:layout-transition".into(), vec![transition]);
            }
            (
                "Landroid/view/ViewGroup;",
                "addView(Landroid/view/View;ILandroid/view/ViewGroup$LayoutParams;)V",
            ) => {
                let child = arg(1)?;
                let index = arg(2)?.int()?;
                self.view_mut(child)?;
                ensure!(args.len() == 4, "invalid indexed child argument count");
                {
                    let params = arg(3)?;
                    ensure!(
                        params != Word::ZERO
                            && self.is_a(
                                &self.heap.get(params)?.class,
                                "Landroid/view/ViewGroup$LayoutParams;"
                            ),
                        "indexed child requires layout parameters"
                    );
                    self.heap
                        .get_mut(child)?
                        .fields
                        .insert("droidless:view:layout-params".into(), vec![params]);
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
                self.heap
                    .get_mut(child)?
                    .fields
                    .insert("droidless:view:parent".into(), vec![receiver]);
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
                let roots = self.native_roots.len();
                self.native_roots.push(receiver);
                self.native_roots.extend(children.iter().copied());
                let removed = (|| -> Result<()> {
                    for child in children {
                        self.focus_before_remove(receiver, child)?;
                        self.heap
                            .get_mut(child)?
                            .fields
                            .remove("droidless:view:parent");
                        self.hierarchy_change(receiver, child, false)?;
                    }
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                removed?;
            }
            (
                "Landroid/view/ViewGroup;",
                "removeView(Landroid/view/View;)V"
                | "removeViewInLayout(Landroid/view/View;)V"
                | "removeViewAt(I)V",
            ) => {
                let child = if method.name == "removeViewAt" {
                    let index = arg(1)?.int()?;
                    self.view_mut(receiver)?
                        .children
                        .get(usize::try_from(index).map_err(|_| {
                            fault("Ljava/lang/IndexOutOfBoundsException;", "child index")
                        })?)
                        .copied()
                        .ok_or_else(|| {
                            fault("Ljava/lang/IndexOutOfBoundsException;", "child index")
                        })?
                } else {
                    arg(1)?
                };
                if self.view_mut(receiver)?.children.contains(&child) {
                    self.focus_before_remove(receiver, child)?;
                    // A focus-loss callback may already have removed this child.
                    let children = &mut self.view_mut(receiver)?.children;
                    if let Some(index) = children.iter().position(|view| *view == child) {
                        children.remove(index);
                        self.heap
                            .get_mut(child)?
                            .fields
                            .remove("droidless:view:parent");
                        self.hierarchy_change(receiver, child, false)?;
                    }
                }
            }
            ("Landroid/view/ViewGroup;", "setMotionEventSplittingEnabled(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:motion-event-splitting".into(), vec![arg(1)?]);
            }
            ("Landroid/widget/TextView;", "setEllipsize(Landroid/text/TextUtils$TruncateAt;)V") => {
                let value = arg(1)?;
                if value != Word::ZERO {
                    self.heap.get(value)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:text:ellipsize".into(), vec![value]);
            }
            (
                "Landroid/widget/TextView;",
                "getTransformationMethod()Landroid/text/method/TransformationMethod;",
            ) => {
                self.view_mut(receiver)?;
                // ponytail: the text profile is untransformed; implement guest transformations when a setter is required.
                result.push(Word::ZERO);
            }
            ("Landroid/widget/TextView;", "setSingleLine()V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:text:single-line".into(), vec![Word::from(1)]);
                self.invalidate_text_layout(receiver)?;
            }
            ("Landroid/widget/TextView;", "setSingleLine(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:text:single-line".into(), vec![arg(1)?]);
                self.invalidate_text_layout(receiver)?;
            }
            ("Landroid/widget/TextView;", "setMaxLines(I)V")
            | ("Landroid/widget/TextView;", "setMinLines(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("droidless:text:{}", method.name), vec![arg(1)?]);
                self.invalidate_text_layout(receiver)?;
            }
            (
                "Landroid/widget/ImageView;",
                "getScaleType()Landroid/widget/ImageView$ScaleType;",
            ) => {
                let ordinal = self.view_mut(receiver)?.image_scale;
                let class = "Landroid/widget/ImageView$ScaleType;";
                let name = graphics_enum_names(class).unwrap()[ordinal as usize];
                result.push(self.graphics_enum_object(&Field {
                    class: class.into(),
                    name: name.into(),
                    ty: class.into(),
                })?);
            }
            (
                "Landroid/widget/ImageView;",
                "setScaleType(Landroid/widget/ImageView$ScaleType;)V",
            ) => {
                let value = arg(1)?;
                ensure!(
                    value != Word::ZERO,
                    fault("Ljava/lang/NullPointerException;", "null scale type")
                );
                let object = self.heap.get(value)?;
                ensure!(
                    object.class == "Landroid/widget/ImageView$ScaleType;",
                    "expected ImageView.ScaleType"
                );
                let ordinal = object
                    .fields
                    .get("droidless:enum:ordinal")
                    .and_then(|v| v.first())
                    .context("uninitialized scale type")?
                    .int()?;
                ensure!(
                    (1..=7).contains(&ordinal),
                    "ImageView matrix scaling is unsupported"
                );
                self.view_mut(receiver)?.image_scale = ordinal;
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
            (
                "Landroid/widget/ImageView;",
                "setImageDrawable(Landroid/graphics/drawable/Drawable;)V",
            ) => {
                let drawable = arg(1)?;
                if drawable != Word::ZERO {
                    ensure!(
                        self.is_a(
                            &self.heap.get(drawable)?.class,
                            "Landroid/graphics/drawable/Drawable;"
                        ),
                        "ImageView requires a Drawable"
                    );
                }
                let image = if drawable == Word::ZERO {
                    None
                } else if let Some(bitmap) = self
                    .heap
                    .get(drawable)?
                    .fields
                    .get("droidless:drawable:bitmap")
                    .and_then(|values| values.first())
                    .copied()
                {
                    match &self.heap.get(bitmap)?.data {
                        Data::Bitmap {
                            bytes,
                            recycled: false,
                            ..
                        } => Some(bytes.clone()),
                        Data::Bitmap { .. } => bail!("cannot display a recycled Bitmap"),
                        _ => bail!("Drawable image is not initialized"),
                    }
                } else {
                    self.heap
                        .get(drawable)?
                        .fields
                        .get("resourceId")
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
            ("Landroid/widget/ImageView;", "setImageURI(Landroid/net/Uri;)V") => {
                self.view_mut(receiver)?;
                let context = *self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:context")
                    .and_then(|v| v.first())
                    .context("ImageView has no Context")?;
                let roots = self.native_roots.len();
                self.native_roots.extend_from_slice(args);
                let loaded = (|| -> Result<()> {
                    let bitmap = if arg(1)? == Word::ZERO {
                        Word::ZERO
                    } else {
                        self.uri_bitmap(context, arg(1)?)?
                    };
                    self.native_roots.push(bitmap);
                    self.invoke(
                        Method {
                            class: "Landroid/widget/ImageView;".into(),
                            name: "setImageBitmap".into(),
                            parameters: vec!["Landroid/graphics/Bitmap;".into()],
                            returns: "V".into(),
                        },
                        vec![receiver, bitmap],
                        true,
                    )?;
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                loaded?;
            }
            ("Landroid/widget/ImageView;", "setImageBitmap(Landroid/graphics/Bitmap;)V") => {
                let bitmap = arg(1)?;
                let drawable = if bitmap == Word::ZERO {
                    Word::ZERO
                } else {
                    let Data::Bitmap {
                        bytes, recycled, ..
                    } = &self.heap.get(bitmap)?.data
                    else {
                        bail!("ImageView requires a Bitmap");
                    };
                    ensure!(!recycled, "cannot display a recycled Bitmap");
                    let image = bytes.clone();
                    let drawable = self
                        .heap
                        .instance("Landroid/graphics/drawable/BitmapDrawable;")?;
                    self.heap
                        .get_mut(drawable)?
                        .fields
                        .insert("droidless:drawable:bitmap".into(), vec![bitmap]);
                    self.view_mut(receiver)?.image = Some(image);
                    drawable
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:image:drawable".into(), vec![drawable]);
                if bitmap == Word::ZERO {
                    self.view_mut(receiver)?.image = None;
                }
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
            (
                "Landroid/graphics/drawable/BitmapDrawable;",
                "<init>(Landroid/content/res/Resources;Landroid/graphics/Bitmap;)V",
            ) => {
                let bitmap = arg(2)?;
                self.heap.get(arg(1)?)?;
                ensure!(
                    matches!(
                        &self.heap.get(bitmap)?.data,
                        Data::Bitmap {
                            recycled: false,
                            ..
                        }
                    ),
                    "BitmapDrawable requires a live Bitmap"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:bitmap".into(), vec![bitmap]);
            }
            (
                "Landroid/graphics/drawable/BitmapDrawable;",
                "<init>(Landroid/graphics/Bitmap;)V",
            ) => {
                let bitmap = arg(1)?;
                ensure!(
                    matches!(
                        &self.heap.get(bitmap)?.data,
                        Data::Bitmap {
                            recycled: false,
                            ..
                        }
                    ),
                    "BitmapDrawable requires a live Bitmap"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:drawable:bitmap".into(), vec![bitmap]);
            }
            ("Landroid/graphics/drawable/BitmapDrawable;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            (
                "Landroid/widget/TextView;",
                "setCompoundDrawablesRelative(Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;)V"
                | "setCompoundDrawables(Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;Landroid/graphics/drawable/Drawable;)V",
            ) => {
                self.view_mut(receiver)?;
                let drawables = args[1..].to_vec();
                ensure!(
                    drawables.len() == 4,
                    "compound drawables require four slots"
                );
                for drawable in &drawables {
                    ensure!(
                        *drawable == Word::ZERO
                            || self.is_a(
                                &self.heap.get(*drawable)?.class,
                                "Landroid/graphics/drawable/Drawable;"
                            ),
                        "compound slot requires Drawable"
                    );
                }
                // Managed identity/bounds are retained; compound-icon painting remains outside the native text profile.
                let relative = if method.name == "setCompoundDrawablesRelative" {
                    drawables.clone()
                } else {
                    vec![Word::ZERO, drawables[1], Word::ZERO, drawables[3]]
                };
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("droidless:view:compound-relative".into(), relative);
                // ponytail: current View layout direction is LTR; map start/end after RTL layout is implemented.
                fields.insert("droidless:view:compound-absolute".into(), drawables);
            }
            (
                "Landroid/widget/TextView;",
                "getCompoundDrawablesRelative()[Landroid/graphics/drawable/Drawable;"
                | "getCompoundDrawables()[Landroid/graphics/drawable/Drawable;",
            ) => {
                self.view_mut(receiver)?;
                let key = if method.name == "getCompoundDrawablesRelative" {
                    "droidless:view:compound-relative"
                } else {
                    "droidless:view:compound-absolute"
                };
                let drawables = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| vec![Word::ZERO; 4]);
                let array = self.array("Landroid/graphics/drawable/Drawable;".into(), 4)?;
                let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
                    bail!("compound drawables are not array");
                };
                for (slot, value) in values.iter_mut().zip(drawables) {
                    *slot = vec![value];
                }
                result.push(array);
            }
            ("Landroid/widget/TextView;", "setTextSize(F)V") => {
                let size = f32::from_bits(arg(1)?.int()? as u32);
                ensure!(size.is_finite() && size >= 0.0, "invalid text size");
                self.view_mut(receiver)?.text_size = size;
                self.invalidate_text_layout(receiver)?;
            }
            ("Landroid/widget/TextView;", "getTextSize()F") => {
                result.push(Word::Bits(self.view_mut(receiver)?.text_size.to_bits()));
            }
            ("Landroid/widget/TextView;", "setTextColor(I)V") => {
                self.view_mut(receiver)?.text_color = arg(1)?.int()? as u32
            }
            (
                "Landroid/widget/TextView;",
                "setTextColor(Landroid/content/res/ColorStateList;)V",
            ) => {
                let colors = arg(1)?;
                let color = self.invoke(
                    Method {
                        class: "Landroid/content/res/ColorStateList;".into(),
                        name: "getDefaultColor".into(),
                        parameters: vec![],
                        returns: "I".into(),
                    },
                    vec![colors],
                    true,
                )?;
                self.view_mut(receiver)?.text_color = color[0].int()? as u32;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:text-colors".into(), vec![colors]);
            }
            (
                "Landroid/widget/TextView;",
                "setHintTextColor(Landroid/content/res/ColorStateList;)V"
                | "setLinkTextColor(Landroid/content/res/ColorStateList;)V",
            ) => {
                let value = arg(1)?;
                ensure!(
                    value == Word::ZERO
                        || self.is_a(
                            &self.heap.get(value)?.class,
                            "Landroid/content/res/ColorStateList;"
                        ),
                    "invalid text ColorStateList"
                );
                let key = if method.name == "setHintTextColor" {
                    "droidless:text:hint-colors"
                } else {
                    "droidless:text:link-colors"
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(key.into(), vec![value]);
            }
            (
                "Landroid/widget/TextView;",
                "getHintTextColors()Landroid/content/res/ColorStateList;"
                | "getLinkTextColors()Landroid/content/res/ColorStateList;",
            ) => {
                let key = if method.name == "getHintTextColors" {
                    "droidless:text:hint-colors"
                } else {
                    "droidless:text:link-colors"
                };
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(key)
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "getBaseline()I") => {
                self.view_mut(receiver)?;
                result.push(Word::from(-1));
            }
            ("Landroid/widget/TextView;", "getBaseline()I") => {
                let measured = self
                    .heap
                    .get(receiver)?
                    .fields
                    .contains_key("droidless:view:measured-height");
                let view = self.view_mut(receiver)?;
                // ponytail: approximate ascent, like current intrinsic text sizing;
                // use shared native font metrics when the text layout profile gains font parity.
                result.push(Word::from(if measured {
                    (view.padding[1] + view.text_size * 0.8).round() as i32
                } else {
                    -1
                }));
            }
            ("Landroid/widget/TextView;", "getCurrentTextColor()I") => {
                result.push(Word::from(self.view_mut(receiver)?.text_color as i32));
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
                self.view_mut(receiver)?.xml_click = None;
                if listener != Word::ZERO {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:touch:clickable".into(), vec![Word::from(1)]);
                }
            }
            ("Landroid/view/View;", "hasOnClickListeners()Z") => {
                let view = self.view_mut(receiver)?;
                result.push(Word::from(
                    (view.listener.is_some() || view.xml_click.is_some()) as i32,
                ));
            }
            (
                "Landroid/view/View;",
                "setOnFocusChangeListener(Landroid/view/View$OnFocusChangeListener;)V",
            ) => {
                let listener = arg(1)?;
                if listener != Word::ZERO {
                    self.heap.get(listener)?;
                }
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:focus-change-listener".into(),
                    vec![listener],
                );
            }
            (
                "Landroid/widget/TextView;",
                "setOnEditorActionListener(Landroid/widget/TextView$OnEditorActionListener;)V",
            ) => {
                let listener = arg(1)?;
                ensure!(
                    listener == Word::ZERO
                        || self.is_a(
                            &self.heap.get(listener)?.class,
                            "Landroid/widget/TextView$OnEditorActionListener;"
                        ),
                    "invalid editor action listener"
                );
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:text:editor-action-listener".into(),
                    vec![listener],
                );
            }
            ("Landroid/widget/TextView;", "onEditorAction(I)V") => {
                let action = arg(1)?.int()?;
                let listener = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:text:editor-action-listener")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                if listener != Word::ZERO {
                    let roots = self.native_roots.len();
                    self.native_roots.extend([receiver, listener]);
                    let callback = self.invoke(
                        Method {
                            class: "Landroid/widget/TextView$OnEditorActionListener;".into(),
                            name: "onEditorAction".into(),
                            parameters: vec![
                                "Landroid/widget/TextView;".into(),
                                "I".into(),
                                "Landroid/view/KeyEvent;".into(),
                            ],
                            returns: "Z".into(),
                        },
                        vec![listener, receiver, Word::from(action), Word::ZERO],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    callback?;
                }
                // ponytail: guest action callbacks only; IME focus navigation and host Return delivery remain ahead.
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
            (
                "Landroid/view/View;",
                "setOnApplyWindowInsetsListener(Landroid/view/View$OnApplyWindowInsetsListener;)V",
            ) => {
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
            ("Landroid/view/View;", "fitSystemWindows(Landroid/graphics/Rect;)Z") => {
                let insets = arg(1)?;
                let fits = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:fits-system-windows")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth());
                let mut consumed = false;
                if insets != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(insets)?.class, "Landroid/graphics/Rect;"),
                        "insets require Rect"
                    );
                    if fits {
                        // ponytail: legacy padding fallback only; add WindowInsets/listener dispatch with a compiled contract.
                        let fields = &self.heap.get(insets)?.fields;
                        let edge = |name: &str| -> Result<f32> {
                            Ok(fields
                                .get(&format!("Landroid/graphics/Rect;->{name}:I"))
                                .and_then(|values| values.first())
                                .copied()
                                .unwrap_or(Word::ZERO)
                                .int()?
                                .max(0) as f32)
                        };
                        let padding =
                            [edge("left")?, edge("top")?, edge("right")?, edge("bottom")?];
                        self.view_mut(receiver)?.padding = padding;
                        self.invoke(
                            Method {
                                class: "Landroid/graphics/Rect;".into(),
                                name: "setEmpty".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![insets],
                            true,
                        )?;
                        consumed = true;
                    }
                }
                result.push(Word::from(i32::from(consumed)));
            }
            ("Landroid/view/View;", "postOnAnimation(Ljava/lang/Runnable;)V") => {
                self.view_mut(receiver)?;
                self.heap.get(arg(1)?)?;
            }
            (
                "Landroid/view/ViewTreeObserver;",
                "addOnPreDrawListener(Landroid/view/ViewTreeObserver$OnPreDrawListener;)V",
            )
            | (
                "Landroid/view/ViewTreeObserver;",
                "removeOnPreDrawListener(Landroid/view/ViewTreeObserver$OnPreDrawListener;)V",
            ) => {
                self.heap.get(receiver)?;
                self.heap.get(arg(1)?)?;
            }
            (
                "Landroid/view/ViewTreeObserver;",
                "addOnGlobalLayoutListener(Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;)V",
            ) => {
                let listener = arg(1)?;
                self.heap.get(listener)?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .entry("droidless:global-layout-listeners".into())
                    .or_default()
                    .push(listener);
            }
            (
                "Landroid/view/ViewTreeObserver;",
                "removeOnGlobalLayoutListener(Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;)V",
            )
            | (
                "Landroid/view/ViewTreeObserver;",
                "removeGlobalOnLayoutListener(Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;)V",
            ) => {
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
            (
                "Landroid/view/View;",
                "computeFitSystemWindows(Landroid/graphics/Rect;Landroid/graphics/Rect;)V",
            ) => {
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
            ("Landroid/view/View;", "getWindowSystemUiVisibility()I") => {
                self.view_mut(receiver)?;
                // ponytail: no Android system-bar configuration in this desktop profile;
                // aggregate requested flags when system-UI setters are implemented.
                result.push(Word::ZERO);
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
            ("Landroid/view/View;", "getMatrix()Landroid/graphics/Matrix;") => {
                self.view_mut(receiver)?;
                let fields = &self.heap.get(receiver)?.fields;
                let translation = |axis| -> Result<f32> {
                    let value = fields
                        .get(&format!("droidless:view:translation-{axis}"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()?;
                    let value = f32::from_bits(value as u32);
                    ensure!(value.is_finite(), "invalid View translation");
                    Ok(value)
                };
                let (x, y) = (translation("x")?, translation("y")?);
                let matrix = if let Some(matrix) = fields
                    .get("droidless:view:matrix")
                    .and_then(|values| values.first())
                    .copied()
                {
                    matrix
                } else {
                    let matrix = self.heap.instance("Landroid/graphics/Matrix;")?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:matrix".into(), vec![matrix]);
                    matrix
                };
                // The current View profile exposes translation; other transform setters remain unsupported.
                self.heap.get_mut(matrix)?.data = Data::Matrix([1., 0., x, 0., 1., y, 0., 0., 1.]);
                result.push(matrix);
            }
            (
                "Landroid/view/ViewGroup;",
                "offsetDescendantRectToMyCoords(Landroid/view/View;Landroid/graphics/Rect;)V"
                | "offsetRectIntoDescendantCoords(Landroid/view/View;Landroid/graphics/Rect;)V",
            ) => {
                self.offset_descendant_rect(
                    receiver,
                    arg(1)?,
                    arg(2)?,
                    method.name == "offsetRectIntoDescendantCoords",
                )?;
            }
            (
                "Landroid/view/View;",
                "getScrollX()I"
                | "getScrollY()I"
                | "computeHorizontalScrollOffset()I"
                | "computeVerticalScrollOffset()I",
            ) => {
                self.view_mut(receiver)?;
                let axis = if method.name == "getScrollX"
                    || method.name == "computeHorizontalScrollOffset"
                {
                    "x"
                } else {
                    "y"
                };
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
            (
                "Landroid/view/View;",
                "getWidth()I"
                | "getHeight()I"
                | "computeHorizontalScrollRange()I"
                | "computeHorizontalScrollExtent()I"
                | "computeVerticalScrollRange()I"
                | "computeVerticalScrollExtent()I",
            ) => {
                self.view_mut(receiver)?;
                let horizontal =
                    method.name == "getWidth" || method.name.starts_with("computeHorizontal");
                let field = if horizontal {
                    "droidless:view:right"
                } else {
                    "droidless:view:bottom"
                };
                let start = if horizontal {
                    "droidless:view:left"
                } else {
                    "droidless:view:top"
                };
                let end = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(field)
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                let start = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(start)
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                result.push(Word::from((end - start).max(0)));
            }
            ("Landroid/view/View;", "canScrollHorizontally(I)Z" | "canScrollVertically(I)Z") => {
                self.view_mut(receiver)?;
                let direction = arg(1)?.int()?;
                let axis = if method.name == "canScrollHorizontally" {
                    "Horizontal"
                } else {
                    "Vertical"
                };
                let roots = self.native_roots.len();
                self.native_roots.push(receiver);
                let answer = (|| -> Result<bool> {
                    let mut metrics = [0; 3];
                    for (slot, suffix) in metrics.iter_mut().zip(["Offset", "Range", "Extent"]) {
                        *slot = self
                            .invoke(
                                Method {
                                    class: "Landroid/view/View;".into(),
                                    name: format!("compute{axis}Scroll{suffix}"),
                                    parameters: vec![],
                                    returns: "I".into(),
                                },
                                vec![receiver],
                                true,
                            )?
                            .first()
                            .context("View scroll metric missing")?
                            .int()?;
                    }
                    let range = metrics[1].wrapping_sub(metrics[2]);
                    Ok(range != 0
                        && if direction < 0 {
                            metrics[0] > 0
                        } else {
                            metrics[0] < range.wrapping_sub(1)
                        })
                })();
                self.native_roots.truncate(roots);
                result.push(Word::from(i32::from(answer?)));
            }
            ("Landroid/view/View;", "getMeasuredWidth()I")
            | ("Landroid/view/View;", "getMeasuredHeight()I")
            | ("Landroid/view/View;", "getMeasuredWidthAndState()I")
            | ("Landroid/view/View;", "getMeasuredHeightAndState()I") => {
                self.view_mut(receiver)?;
                let edge = if method.name.starts_with("getMeasuredWidth") {
                    "width"
                } else {
                    "height"
                };
                let value = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(&format!("droidless:view:measured-{edge}"))
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                result.push(Word::from(if method.name.ends_with("AndState") {
                    value
                } else {
                    value & 0x00ff_ffff
                }));
            }
            ("Landroid/view/View;", "getMeasuredState()I") => {
                self.view_mut(receiver)?;
                let fields = &self.heap.get(receiver)?.fields;
                let value = |axis| -> Result<i32> {
                    fields
                        .get(&format!("droidless:view:measured-{axis}"))
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()
                };
                result.push(Word::from(
                    (value("width")? & 0xff00_0000u32 as i32)
                        | ((value("height")? >> 16) & 0x0000_ff00),
                ));
            }
            ("Landroid/view/View;", "combineMeasuredStates(II)I") => {
                result.push(Word::from(arg(0)?.int()? | arg(1)?.int()?));
            }
            ("Landroid/view/View;", "resolveSizeAndState(III)I") => {
                let desired = arg(0)?.int()?;
                let spec = arg(1)?.int()? as u32;
                let size = (spec & 0x3fff_ffff) as i32;
                let measured = match spec & 0xc000_0000 {
                    0x4000_0000 => size,
                    0x8000_0000 if size < desired => size | 0x0100_0000,
                    _ => desired,
                };
                result.push(Word::from(
                    measured | (arg(2)?.int()? & 0xff00_0000u32 as i32),
                ));
            }
            ("Landroid/view/View;", "getMinimumWidth()I")
            | ("Landroid/view/View;", "getMinimumHeight()I")
            | ("Landroid/view/View;", "getSuggestedMinimumWidth()I")
            | ("Landroid/view/View;", "getSuggestedMinimumHeight()I") => {
                self.view_mut(receiver)?;
                let edge = if method.name.ends_with("Width") {
                    "width"
                } else {
                    "height"
                };
                let mut minimum = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(&format!("droidless:view:minimum-{edge}"))
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                if method.name.starts_with("getSuggested") {
                    let background = self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:background-drawable")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO);
                    if background != Word::ZERO {
                        let roots = self.native_roots.len();
                        self.native_roots.extend([receiver, background]);
                        let background_minimum = self.invoke(
                            Method {
                                class: "Landroid/graphics/drawable/Drawable;".into(),
                                name: if edge == "width" {
                                    "getMinimumWidth"
                                } else {
                                    "getMinimumHeight"
                                }
                                .into(),
                                parameters: vec![],
                                returns: "I".into(),
                            },
                            vec![background],
                            true,
                        );
                        self.native_roots.truncate(roots);
                        minimum = minimum.max(
                            background_minimum?
                                .first()
                                .context("background minimum missing")?
                                .int()?,
                        );
                    }
                }
                result.push(Word::from(minimum));
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
                let edge = method
                    .name
                    .strip_prefix("get")
                    .context("invalid View edge getter")?
                    .to_ascii_lowercase();
                let value = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(&format!("droidless:view:{edge}"))
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(value);
            }
            ("Landroid/view/View;", "isLaidOut()Z") => {
                self.view_mut(receiver)?;
                result.push(Word::from(i32::from(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:laid-out")
                        .and_then(|values| values.first())
                        .is_some_and(|value| value.truth()),
                )));
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
                let recycler = "Landroid/support/v7/widget/RecyclerView;";
                if self.is_a(&self.heap.get(receiver)?.class, recycler)
                    && let Some((dex, class)) = self.class_location(recycler)
                {
                    // ponytail: this profile has no item-animation clock. Apply the existing
                    // unanimated policy before ancestor callbacks can lay out the list.
                    let mut setters = self.apk.dex[dex].classes[class]
                        .methods
                        .iter()
                        .filter(|encoded| encoded.access & 8 == 0)
                        .map(|encoded| &self.apk.dex[dex].methods[encoded.index])
                        .filter(|method| {
                            method.name == "setItemAnimator"
                                && method.returns == "V"
                                && method.parameters.len() == 1
                                && method.parameters[0].starts_with('L')
                        });
                    let setter = setters
                        .next()
                        .cloned()
                        .context("RecyclerView item animator setter missing")?;
                    ensure!(
                        setters.next().is_none(),
                        "ambiguous RecyclerView item animator setter"
                    );
                    self.invoke(setter, vec![receiver, Word::ZERO], true)?;
                }

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
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:layout-required".into(), vec![Word::from(1)]);
            }
            ("Landroid/view/View;", "onMeasure(II)V") => {
                self.measure_view(receiver, [arg(1)?, arg(2)?], false)?;
            }
            ("Landroid/view/View;", "setMeasuredDimension(II)V") => {
                self.view_mut(receiver)?;
                for (edge, value) in [("width", arg(1)?), ("height", arg(2)?)] {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(format!("droidless:view:measured-{edge}"), vec![value]);
                }
            }
            ("Landroid/view/View;", "layout(IIII)V") => {
                let bounds = [arg(1)?, arg(2)?, arg(3)?, arg(4)?];
                let names = ["left", "top", "right", "bottom"];
                let changed = {
                    let object = self.heap.get_mut(receiver)?;
                    let changed = names.iter().zip(bounds).any(|(name, value)| {
                        object
                            .fields
                            .get(&format!("droidless:view:{name}"))
                            .and_then(|values| values.first())
                            .copied()
                            .unwrap_or(Word::ZERO)
                            != value
                    });
                    for (name, value) in names.into_iter().zip(bounds) {
                        object
                            .fields
                            .insert(format!("droidless:view:{name}"), vec![value]);
                    }
                    object
                        .fields
                        .insert("droidless:view:laid-out".into(), vec![Word::from(1)]);
                    object
                        .fields
                        .insert("droidless:view:layout-requested".into(), vec![Word::ZERO]);
                    changed
                };
                self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "onLayout".into(),
                        parameters: vec![
                            "Z".into(),
                            "I".into(),
                            "I".into(),
                            "I".into(),
                            "I".into(),
                        ],
                        returns: "V".into(),
                    },
                    std::iter::once(receiver)
                        .chain([Word::from(i32::from(changed))])
                        .chain(bounds)
                        .collect(),
                    true,
                )?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:layout-required".into(), vec![Word::ZERO]);
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
            (
                "Landroid/view/ViewParent;" | "Landroid/view/ViewGroup;",
                "onStartNestedScroll(Landroid/view/View;Landroid/view/View;I)Z",
            ) => {
                ensure!(
                    self.is_a(&self.heap.get(receiver)?.class, "Landroid/view/ViewGroup;"),
                    "nested-scroll parent requires ViewGroup"
                );
                self.view_mut(arg(1)?)?;
                self.view_mut(arg(2)?)?;
                arg(3)?.int()?;
                result.push(Word::ZERO);
            }
            ("Landroid/view/ViewGroup;", "isChildrenDrawingOrderEnabled()Z") => {
                self.view_mut(receiver)?;
                // Android's base group uses insertion order until explicitly configured.
                // Custom drawing-order configuration remains unsupported.
                result.push(Word::ZERO);
            }
            (
                "Landroid/view/ViewGroup;",
                "setClipChildren(Z)V"
                | "setClipToPadding(Z)V"
                | "getClipChildren()Z"
                | "getClipToPadding()Z",
            ) => {
                ensure!(
                    self.is_a(&self.heap.get(receiver)?.class, "Landroid/view/ViewGroup;"),
                    "clipping requires ViewGroup"
                );
                let value = if method.name.starts_with("set") {
                    Some(arg(1)?.int()? != 0)
                } else {
                    None
                };
                let view = self.view_mut(receiver)?;
                let flag = if method.name.ends_with("Children") {
                    &mut view.clip_children
                } else {
                    &mut view.clip_to_padding
                };
                if let Some(value) = value {
                    *flag = value;
                } else {
                    result.push(Word::from(i32::from(*flag)));
                }
            }
            ("Landroid/view/View;", "getParent()Landroid/view/ViewParent;") => {
                let parent = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:parent")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(parent);
            }
            (
                "Landroid/view/View;",
                "getWindowToken()Landroid/os/IBinder;" | "isAttachedToWindow()Z",
            ) => {
                let token = self.view_window_token(receiver)?;
                result.push(if method.name == "isAttachedToWindow" {
                    Word::from(i32::from(token != Word::ZERO))
                } else {
                    token
                });
            }
            ("Landroid/view/View;", "hasWindowFocus()Z") => {
                result.push(Word::from(i32::from(self.view_has_window_focus(receiver)?)));
            }
            ("Landroid/view/View;", "setTag(Ljava/lang/Object;)V") => {
                let tag = arg(1)?;
                if tag != Word::ZERO {
                    self.heap.get(tag)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:tag".into(), vec![tag]);
            }
            ("Landroid/view/View;", "getTag()Ljava/lang/Object;") => {
                let tag = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:tag")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(tag);
            }
            ("Landroid/view/View;", "setTag(ILjava/lang/Object;)V") => {
                let tag = arg(2)?;
                if tag != Word::ZERO {
                    self.heap.get(tag)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("droidless:view:tag:{}", arg(1)?.int()?), vec![tag]);
            }
            ("Landroid/view/View;", "getTag(I)Ljava/lang/Object;") => {
                let tag = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(&format!("droidless:view:tag:{}", arg(1)?.int()?))
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(tag);
            }
            ("Landroid/view/View;", "post(Ljava/lang/Runnable;)Z")
            | ("Landroid/view/View;", "postDelayed(Ljava/lang/Runnable;J)Z") => {
                let handler = if let Some(handler) = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:handler")
                    .and_then(|values| values.first())
                    .copied()
                {
                    handler
                } else {
                    let handler = self.heap.instance("Landroid/os/Handler;")?;
                    self.invoke(
                        Method {
                            class: "Landroid/os/Handler;".into(),
                            name: "<init>".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![handler],
                        false,
                    )?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:handler".into(), vec![handler]);
                    handler
                };
                let mut forwarded = vec![handler, arg(1)?];
                if method.name == "postDelayed" {
                    forwarded
                        .extend_from_slice(args.get(2..4).context("missing postDelayed interval")?);
                }
                let posted = self.invoke(
                    Method {
                        class: "Landroid/os/Handler;".into(),
                        name: method.name.clone(),
                        parameters: method.parameters.clone(),
                        returns: "Z".into(),
                    },
                    forwarded,
                    false,
                )?;
                result.extend(posted);
            }
            ("Landroid/view/View;", "removeCallbacks(Ljava/lang/Runnable;)Z") => {
                let handler = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:handler")
                    .and_then(|values| values.first())
                    .copied();
                let mut removed = false;
                if let Some(handler) = handler {
                    let found = self.invoke(
                        Method {
                            class: "Landroid/os/Handler;".into(),
                            name: "hasCallbacks".into(),
                            parameters: vec!["Ljava/lang/Runnable;".into()],
                            returns: "Z".into(),
                        },
                        vec![handler, arg(1)?],
                        false,
                    )?;
                    removed = found.first().is_some_and(|value| value.truth());
                    self.invoke(
                        Method {
                            class: "Landroid/os/Handler;".into(),
                            name: "removeCallbacks".into(),
                            parameters: vec!["Ljava/lang/Runnable;".into()],
                            returns: "V".into(),
                        },
                        vec![handler, arg(1)?],
                        false,
                    )?;
                }
                result.push(Word::from(i32::from(removed)));
            }
            ("Landroid/view/View;", "setOverScrollMode(I)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:over-scroll-mode".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "computeScroll()V") => {
                self.view_mut(receiver)?;
            }
            ("Landroid/view/View;", "postInvalidateOnAnimation()V") => {
                self.view_mut(receiver)?;
                // ponytail: coalesce full-frame redraws at the next host poll; no dirty rectangles or vsync emulation.
                if !self.queue.closed && self.view_window_token(receiver)? != Word::ZERO {
                    self.queue.redraw = true;
                }
            }
            (
                "Landroid/view/View;",
                "requestLayout()V"
                | "requestApplyInsets()V"
                | "invalidate()V"
                | "invalidate(Landroid/graphics/Rect;)V",
            )
            | ("Landroid/support/v7/widget/ContentFrameLayout;", "requestLayout()V") => {
                // ponytail: desktop content has zero Android system-bar insets; request a layout
                // pass. Full WindowInsets/listener dispatch belongs with system-bar emulation.
                self.view_mut(receiver)?;
                // ponytail: redraw the full snapshot rather than tracking dirty rectangles.
                if args.len() == 2 {
                    self.heap.get(arg(1)?)?;
                }
                self.request_view_layout(receiver)?;
            }
            ("Landroid/view/View;", "getPaddingLeft()I")
            | ("Landroid/view/View;", "getPaddingStart()I")
            | ("Landroid/view/View;", "getPaddingTop()I")
            | ("Landroid/view/View;", "getPaddingRight()I")
            | ("Landroid/view/View;", "getPaddingEnd()I")
            | ("Landroid/view/View;", "getPaddingBottom()I") => {
                // ponytail: the current layout profile is LTR; resolve start/end again when RTL is supported.
                let edge = match method.name.as_str() {
                    "getPaddingLeft" | "getPaddingStart" => 0,
                    "getPaddingTop" => 1,
                    "getPaddingRight" | "getPaddingEnd" => 2,
                    _ => 3,
                };
                result.push(Word::from(self.view_mut(receiver)?.padding[edge] as i32));
            }
            ("Landroid/view/View;", "isPaddingRelative()Z") => {
                self.view_mut(receiver)?;
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:padding-relative")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "setVisibility(I)V") => {
                let v = arg(1)?.int()?;
                ensure!([0, 4, 8].contains(&v), "invalid View visibility");
                self.view_mut(receiver)?.visible = v;
                let foreground = self.window_word(receiver, "droidless:view:foreground")?;
                if foreground != Word::ZERO {
                    let roots = self.native_roots.len();
                    self.native_roots.extend([receiver, foreground]);
                    let changed = self.invoke(
                        Method {
                            class: "Landroid/graphics/drawable/Drawable;".into(),
                            name: "setVisible".into(),
                            parameters: vec!["Z".into(), "Z".into()],
                            returns: "Z".into(),
                        },
                        vec![foreground, Word::from(i32::from(v == 0)), Word::ZERO],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    changed?;
                }

                if self.find_focus(receiver)? != Word::ZERO
                    && (v == 8 || (v == 4 && self.focus_root(receiver)? != receiver))
                {
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "clearFocus".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    )?;
                }
            }
            ("Landroid/view/View;", "hasTransientState()Z") => {
                let mut pending = vec![receiver];
                let mut seen = std::collections::BTreeSet::new();
                let mut transient = false;
                while let Some(view) = pending.pop() {
                    if !seen.insert(view.reference()?) {
                        continue;
                    }
                    ensure!(seen.len() <= 65_536, "transient-state hierarchy limit");
                    let object = self.heap.get(view)?;
                    if object
                        .fields
                        .get("droidless:view:transient-count")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        .int()?
                        > 0
                    {
                        transient = true;
                        break;
                    }
                    pending.extend(
                        object
                            .view
                            .as_ref()
                            .context("transient state requires View")?
                            .children
                            .iter()
                            .copied(),
                    );
                }
                result.push(Word::from(i32::from(transient)));
            }
            ("Landroid/view/View;", "setHasTransientState(Z)V") => {
                self.view_mut(receiver)?;
                let count = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:transient-count")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                let count = if arg(1)?.truth() {
                    count
                        .checked_add(1)
                        .context("transient-state count limit")?
                } else {
                    (count - 1).max(0)
                };
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:transient-count".into(),
                    vec![Word::from(count)],
                );
                // ponytail: counted state and subtree queries; add ViewParent change callbacks
                // with a compiled parent-notification contract when that lifecycle is needed.
            }
            ("Landroid/graphics/drawable/Drawable;", "isStateful()Z") => {
                let class = &self.heap.get(receiver)?.class;
                ensure!(
                    class == "Landroid/graphics/drawable/Drawable;"
                        || [
                            "Landroid/graphics/drawable/ColorDrawable;",
                            "Landroid/graphics/drawable/GradientDrawable;",
                            "Landroid/graphics/drawable/BitmapDrawable;"
                        ]
                        .iter()
                        .any(|parent| self.is_a(class, parent)),
                    "composite drawable state support incomplete"
                );
                result.push(Word::ZERO);
            }
            ("Landroid/widget/CheckedTextView;", "setChecked(Z)V" | "isChecked()Z") => {
                self.view_mut(receiver)?;
                if method.name == "setChecked" {
                    let value = Word::from(i32::from(arg(1)?.truth()));
                    let old = self
                        .heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:checked".into(), vec![value]);
                    if old
                        .as_ref()
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        != value
                    {
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "refreshDrawableState".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![receiver],
                            true,
                        )?;
                    }
                } else {
                    result.push(
                        self.heap
                            .get(receiver)?
                            .fields
                            .get("droidless:view:checked")
                            .and_then(|values| values.first())
                            .copied()
                            .unwrap_or(Word::ZERO),
                    );
                }
            }
            (
                "Landroid/widget/CheckedTextView;",
                "setCheckMarkDrawable(Landroid/graphics/drawable/Drawable;)V",
            ) => {
                let value = arg(1)?;
                ensure!(
                    value == Word::ZERO
                        || self.is_a(
                            &self.heap.get(value)?.class,
                            "Landroid/graphics/drawable/Drawable;"
                        ),
                    "invalid checkmark Drawable"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:checkmark:drawable".into(), vec![value]);
                self.request_view_layout(receiver)?;
            }
            (
                "Landroid/widget/CheckedTextView;",
                "getCheckMarkDrawable()Landroid/graphics/drawable/Drawable;",
            ) => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:checkmark:drawable")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "setSaveFromParentEnabled(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:save-from-parent".into(), vec![arg(1)?]);
            }
            ("Landroid/view/View;", "setWillNotDraw(Z)V")
            | ("Landroid/support/v7/widget/ViewStubCompat;", "setWillNotDraw(Z)V") => {
                self.view_mut(receiver)?;
            }
            ("Landroid/view/View;", "setFocusableInTouchMode(Z)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setFocusableInTouchMode(Z)V")
            | ("Landroid/view/View;", "setFocusable(Z)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setFocusable(Z)V") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("droidless:{}", method.name), vec![arg(1)?]);
                if method.name == "setFocusableInTouchMode" && arg(1)?.truth() {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:setFocusable".into(), vec![Word::from(1)]);
                } else if method.name == "setFocusable" && !arg(1)?.truth() {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:setFocusableInTouchMode".into(), vec![Word::ZERO]);
                    if self
                        .focus_field(receiver, "droidless:view:focused")?
                        .truth()
                    {
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "clearFocus".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![receiver],
                            true,
                        )?;
                    }
                }
            }
            ("Landroid/view/View;", "setImportantForAccessibility(I)V")
            | ("Landroid/support/v4/widget/DrawerLayout;", "setImportantForAccessibility(I)V") => {
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:important-for-accessibility".into(),
                    vec![arg(1)?],
                );
            }
            ("Landroid/view/View;", "onFinishInflate()V") => {
                self.view_mut(receiver)?;
            }
            ("Landroid/view/View;", "setAccessibilityLiveRegion(I)V") => {
                self.view_mut(receiver)?;
                // API-21 stores two mode bits. Accessibility-service event delivery is outside this profile.
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:accessibility-live-region".into(),
                    vec![Word::from(arg(1)?.int()? & 3)],
                );
            }
            ("Landroid/view/View;", "getAccessibilityLiveRegion()I") => {
                self.view_mut(receiver)?;
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:accessibility-live-region")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
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
            (
                "Landroid/view/View;",
                "setAccessibilityDelegate(Landroid/view/View$AccessibilityDelegate;)V",
            )
            | (
                "Landroid/support/v4/widget/DrawerLayout;",
                "setAccessibilityDelegate(Landroid/view/View$AccessibilityDelegate;)V",
            ) => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:accessibility-delegate".into(), vec![arg(1)?]);
            }
            (
                "Landroid/view/View;",
                "getBackgroundTintList()Landroid/content/res/ColorStateList;"
                | "getBackgroundTintMode()Landroid/graphics/PorterDuff$Mode;",
            ) => {
                self.view_mut(receiver)?;
                let key = if method.name == "getBackgroundTintList" {
                    "droidless:view:background-tint-list"
                } else {
                    "droidless:view:background-tint-mode"
                };
                // Queries expose only retained state. XML/setter tint application is not implemented.
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(key)
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "setBackgroundResource(I)V") => {
                self.view_mut(receiver)?;
                let id = arg(1)?.int()?;
                let key = "droidless:view:background-resource";
                if id != 0
                    && self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(key)
                        .and_then(|values| values.first())
                        .copied()
                        == Some(Word::from(id))
                {
                    return Ok(Some(vec![]));
                }
                let roots = self.native_roots.len();
                self.native_roots.push(receiver);
                let changed = (|| -> Result<()> {
                    let drawable = if id == 0 {
                        Word::ZERO
                    } else {
                        let context = self
                            .heap
                            .get(receiver)?
                            .fields
                            .get("droidless:view:context")
                            .and_then(|values| values.first())
                            .copied()
                            .context("View has no Context")?;
                        self.native_roots.push(context);
                        *self
                            .invoke(
                                Method {
                                    class: "Landroid/content/Context;".into(),
                                    name: "getDrawable".into(),
                                    parameters: vec!["I".into()],
                                    returns: "Landroid/graphics/drawable/Drawable;".into(),
                                },
                                vec![context, Word::from(id)],
                                true,
                            )?
                            .first()
                            .context("Context drawable missing")?
                    };
                    self.native_roots.push(drawable);
                    self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "setBackground".into(),
                            parameters: vec!["Landroid/graphics/drawable/Drawable;".into()],
                            returns: "V".into(),
                        },
                        vec![receiver, drawable],
                        true,
                    )?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(key.into(), vec![Word::from(id)]);
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                changed?;
            }
            ("Landroid/view/View;", "setBackgroundColor(I)V") => {
                self.view_mut(receiver)?.background = Some(arg(1)?.int()? as u32);
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .remove("droidless:view:background-resource");
            }
            ("Landroid/view/View;", "setContentDescription(Ljava/lang/CharSequence;)V") => {
                let description = arg(1)?;
                self.view_mut(receiver)?;
                let previous = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:content-description")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                if previous == description {
                    return Ok(Some(vec![]));
                }
                if previous != Word::ZERO {
                    let roots = self.native_roots.len();
                    self.native_roots.extend([receiver, previous, description]);
                    let equal = self.invoke(
                        Method {
                            class: "Ljava/lang/Object;".into(),
                            name: "equals".into(),
                            parameters: vec!["Ljava/lang/Object;".into()],
                            returns: "Z".into(),
                        },
                        vec![previous, description],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    if equal?
                        .first()
                        .context("description equality result missing")?
                        .truth()
                    {
                        return Ok(Some(vec![]));
                    }
                }
                let text = if description == Word::ZERO {
                    None
                } else {
                    Some(self.heap.text(description)?.to_owned())
                };
                if text.as_ref().is_some_and(|text| !text.is_empty())
                    && self
                        .heap
                        .get(receiver)?
                        .fields
                        .get("droidless:important-for-accessibility")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO)
                        == Word::ZERO
                {
                    self.heap.get_mut(receiver)?.fields.insert(
                        "droidless:important-for-accessibility".into(),
                        vec![Word::from(1)],
                    );
                }
                self.view_mut(receiver)?.content_description = text;
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:content-description".into(),
                    vec![description],
                );
            }
            ("Landroid/view/View;", "getContentDescription()Ljava/lang/CharSequence;") => {
                self.view_mut(receiver)?;
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("droidless:view:content-description")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/view/View;", "getBackground()Landroid/graphics/drawable/Drawable;")
            | (
                "Landroid/support/v7/widget/bs;",
                "getBackground()Landroid/graphics/drawable/Drawable;",
            ) => {
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
                    let drawable = self
                        .heap
                        .instance("Landroid/graphics/drawable/ColorDrawable;")?;
                    self.heap
                        .get_mut(drawable)?
                        .fields
                        .insert("color".into(), vec![Word::from(color as i32)]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:background-drawable".into(), vec![drawable]);
                    drawable
                } else {
                    Word::ZERO
                };
                result.push(drawable);
            }
            (
                "Landroid/view/View;",
                "setBackgroundDrawable(Landroid/graphics/drawable/Drawable;)V",
            )
            | (
                "Landroid/support/v7/widget/bs;",
                "setBackgroundDrawable(Landroid/graphics/drawable/Drawable;)V",
            ) => {
                let drawable = arg(1)?;
                if drawable != Word::ZERO {
                    ensure!(
                        self.is_a(
                            &self.heap.get(drawable)?.class,
                            "Landroid/graphics/drawable/Drawable;"
                        ),
                        "background requires Drawable"
                    );
                }
                if self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:background-drawable")
                    .and_then(|values| values.first())
                    .copied()
                    == Some(drawable)
                {
                    return Ok(Some(vec![]));
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .remove("droidless:view:background-resource");
                let color = self.background_drawable_color(drawable, 0)?;
                self.view_mut(receiver)?.background = color;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:background-drawable".into(), vec![drawable]);
                if drawable != Word::ZERO
                    && self.is_a(
                        &self.heap.get(drawable)?.class,
                        "Landroid/graphics/drawable/StateListDrawable;",
                    )
                {
                    let roots = self.native_roots.len();
                    self.native_roots.extend([receiver, drawable]);
                    let changed = self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "drawableStateChanged".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    );
                    self.native_roots.truncate(roots);
                    changed?;
                }
            }
            ("Landroid/view/View;", "setBackground(Landroid/graphics/drawable/Drawable;)V") => {
                let drawable = arg(1)?;
                let roots = self.native_roots.len();
                self.native_roots.extend([receiver, drawable]);
                let changed = self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "setBackgroundDrawable".into(),
                        parameters: vec!["Landroid/graphics/drawable/Drawable;".into()],
                        returns: "V".into(),
                    },
                    vec![receiver, drawable],
                    true,
                );
                self.native_roots.truncate(roots);
                changed?;
            }
            ("Landroid/view/View;", "setElevation(F)V" | "getElevation()F" | "getZ()F") => {
                self.view_mut(receiver)?;
                if method.name == "setElevation" {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:view:elevation".into(), vec![arg(1)?]);
                } else {
                    result.push(
                        self.heap
                            .get(receiver)?
                            .fields
                            .get("droidless:view:elevation")
                            .and_then(|values| values.first())
                            .copied()
                            .unwrap_or(Word::ZERO),
                    );
                }
            }
            (
                "Landroid/view/View;",
                "setStateListAnimator(Landroid/animation/StateListAnimator;)V",
            ) => {
                let animator = arg(1)?;
                if animator != Word::ZERO {
                    self.heap.get(animator)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:state-list-animator".into(), vec![animator]);
            }
            ("Landroid/view/View;", "setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V") => {
                let params = arg(1)?;
                if params != Word::ZERO {
                    self.heap.get(params)?;
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:view:layout-params".into(), vec![params]);
                self.request_view_layout(receiver)?;
            }
            ("Landroid/view/View;", "getLayoutParams()Landroid/view/ViewGroup$LayoutParams;") => {
                let params = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:view:layout-params")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
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
                        self.is_a(
                            &self.heap.get(animation)?.class,
                            "Landroid/view/animation/Animation;"
                        ),
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
            ("Landroid/view/View;", "setAlpha(F)V" | "getAlpha()F") => {
                if method.name == "setAlpha" {
                    let value = f32::from_bits(arg(1)?.int()? as u32);
                    ensure!(value.is_finite(), "invalid View alpha");
                    self.view_mut(receiver)?.alpha = value;
                } else {
                    result.push(Word::Bits(self.view_mut(receiver)?.alpha.to_bits()));
                }
            }
            (
                "Landroid/view/View;",
                "setTranslationX(F)V"
                | "setTranslationY(F)V"
                | "getTranslationX()F"
                | "getTranslationY()F",
            ) => {
                let key = if method.name.ends_with('X') {
                    "droidless:view:translation-x"
                } else {
                    "droidless:view:translation-y"
                };
                if method.name.starts_with("set") {
                    let value = f32::from_bits(arg(1)?.int()? as u32);
                    ensure!(
                        value.is_finite() && value.abs() <= 1_000_000.0,
                        "invalid View translation"
                    );
                    self.view_mut(receiver)?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(key.into(), vec![arg(1)?]);
                } else {
                    result.push(
                        self.heap
                            .get(receiver)?
                            .fields
                            .get(key)
                            .and_then(|v| v.first())
                            .copied()
                            .unwrap_or(Word::ZERO),
                    );
                }
            }
            ("Landroid/view/View;", "setPadding(IIII)V" | "setPaddingRelative(IIII)V") => {
                let values = args[1..]
                    .iter()
                    .map(|w| w.int())
                    .collect::<Result<Vec<_>>>()?;
                ensure!(
                    values.len() == 4 && values.iter().all(|n| *n >= 0),
                    "padding must have four non-negative values"
                );
                self.view_mut(receiver)?.padding = [
                    values[0] as f32,
                    values[1] as f32,
                    values[2] as f32,
                    values[3] as f32,
                ];
                self.heap.get_mut(receiver)?.fields.insert(
                    "droidless:view:padding-relative".into(),
                    vec![Word::from(i32::from(method.name == "setPaddingRelative"))],
                );
                self.invalidate_text_layout(receiver)?;
            }
            (
                "Landroid/util/Log;",
                "d(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I"
                | "i(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I"
                | "w(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I"
                | "e(Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)I",
            ) => {
                let tag = if arg(0)? == Word::ZERO {
                    String::new()
                } else {
                    self.heap.text(arg(0)?)?.to_owned()
                };
                let message = if arg(1)? == Word::ZERO {
                    "null".to_owned()
                } else {
                    self.heap.text(arg(1)?)?.to_owned()
                };
                let trace = self.throwable_trace(arg(2)?)?;
                eprintln!("{}/{tag}: {message}\n{trace}", method.name.to_uppercase());
                // Host stderr sink; Android log-buffer byte counts are outside this profile.
                result.push(Word::ZERO);
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
    fn offset_descendant_rect(
        &mut self,
        parent: Word,
        mut child: Word,
        rect: Word,
        into_child: bool,
    ) -> Result<()> {
        self.require_main_thread()?;
        ensure!(
            self.is_a(&self.heap.get(parent)?.class, "Landroid/view/ViewGroup;"),
            "coordinate conversion requires ViewGroup"
        );
        if parent == child {
            return Ok(());
        }
        // ponytail: 128 parent steps; reject cycles/deeper hierarchies instead of looping on hostile guest state.
        for _ in 0..128 {
            let ancestor = self.focus_field(child, "droidless:view:parent")?;
            if ancestor == Word::ZERO || self.heap.get(ancestor)?.view.is_none() {
                return Err(fault(
                    "Ljava/lang/IllegalArgumentException;",
                    "parameter must be a descendant of this view",
                ));
            }
            let delta = |edge, axis| -> Result<i32> {
                let position = self.focus_field(child, edge)?.int()?;
                let scroll = self.focus_field(child, axis)?.int()?;
                Ok(if into_child {
                    scroll.wrapping_sub(position)
                } else {
                    position.wrapping_sub(scroll)
                })
            };
            let x = delta("droidless:view:left", "droidless:view:scroll-x")?;
            let y = delta("droidless:view:top", "droidless:view:scroll-y")?;
            ensure!(
                self.is_a(&self.heap.get(rect)?.class, "Landroid/graphics/Rect;"),
                "coordinate conversion requires Rect"
            );
            let fields = &mut self.heap.get_mut(rect)?.fields;
            for (edge, offset) in [("left", x), ("top", y), ("right", x), ("bottom", y)] {
                let key = format!("Landroid/graphics/Rect;->{edge}:I");
                let value = fields
                    .get(&key)
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                fields.insert(key, vec![Word::from(value.wrapping_add(offset))]);
            }
            if ancestor == parent {
                return Ok(());
            }
            child = ancestor;
        }
        bail!("cyclic or too deep View coordinate hierarchy")
    }
    pub(crate) fn view_mut(&mut self, word: Word) -> Result<&mut crate::ui::View> {
        self.heap
            .get_mut(word)?
            .view
            .as_mut()
            .context("framework method expects a View")
    }
    fn hierarchy_change(&mut self, parent: Word, child: Word, added: bool) -> Result<()> {
        let roots = self.native_roots.len();
        self.native_roots.extend([parent, child]);
        let result = (|| -> Result<()> {
            self.focus_hierarchy_change(parent, child, added)?;
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
        })();
        self.native_roots.truncate(roots);
        result
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
            Data::Attributes { named, .. } => Ok(named.get(name).cloned()),
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
    pub(crate) fn find_view(&self, word: Word, id: u32, depth: usize) -> Result<Option<Word>> {
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
    pub(crate) fn install_android_content_id(&mut self, word: Word, depth: usize) -> Result<bool> {
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
    pub(crate) fn inflate_id(
        &mut self,
        id: u32,
        depth: usize,
        context: Word,
        inflater: Word,
        parent: Word,
        attach: bool,
    ) -> Result<Word> {
        ensure!(depth < 64, "layout inflation nesting limit");
        let name = self.apk.resources.text(id)?;
        let data = self
            .apk
            .files
            .get(&name)
            .with_context(|| format!("layout file {name} missing"))?;
        let element = droidless_formats::xml::parse(data)?;
        if element.name == "merge" {
            ensure!(
                attach && parent != Word::ZERO,
                "merge requires an attached parent"
            );
            let roots = self.native_roots.len();
            self.native_roots.extend([parent, context, inflater]);
            let attached = (|| -> Result<()> {
                for child in &element.children {
                    self.inflate_child(parent, child, depth + 1, context, inflater)?;
                }
                Ok(())
            })();
            self.native_roots.truncate(roots);
            attached?;
            return Ok(parent);
        }
        self.inflate(&element, depth + 1, context, inflater, parent)
    }
    fn construct_inflated_view(
        &mut self,
        view: Word,
        class: &str,
        context: Word,
        attrs: Word,
    ) -> Result<()> {
        self.heap
            .get_mut(view)?
            .fields
            .insert("droidless:view:xml-attributes".into(), vec![attrs]);
        let context_type = "Landroid/content/Context;";
        let attributes_type = "Landroid/util/AttributeSet;";
        let constructor = if let Some((dex, index)) = self.class_location(class) {
            let definition = &self.apk.dex[dex].classes[index];
            [
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
            })
        } else if class.starts_with("Landroid/widget/")
            || [
                "Landroid/view/View;",
                "Landroid/view/ViewGroup;",
                "Landroid/view/ViewStub;",
            ]
            .contains(&class)
        {
            Some(Method {
                class: class.into(),
                name: "<init>".into(),
                parameters: vec![context_type.into(), attributes_type.into()],
                returns: "V".into(),
            })
        } else {
            None
        };
        let Some(constructor) = constructor else {
            return Ok(());
        };
        let mut args = vec![view, context];
        if constructor
            .parameters
            .iter()
            .any(|ty| ty == attributes_type)
        {
            args.push(attrs);
        }
        if constructor.parameters.last().is_some_and(|ty| ty == "I") {
            args.push(Word::ZERO);
        }
        self.invoke(constructor, args, false)?;
        Ok(())
    }
    fn inflate_view_stub(&mut self, stub: Word) -> Result<Word> {
        let fields = &self.heap.get(stub)?.fields;
        let parent = fields
            .get("droidless:view:parent")
            .and_then(|values| values.first())
            .copied()
            .ok_or_else(|| {
                fault(
                    "Ljava/lang/IllegalStateException;",
                    "ViewStub has no ViewGroup parent",
                )
            })?;
        ensure!(
            self.is_a(&self.heap.get(parent)?.class, "Landroid/view/ViewGroup;"),
            "ViewStub parent must be ViewGroup"
        );
        let context = fields
            .get("droidless:view:context")
            .and_then(|values| values.first())
            .copied()
            .context("ViewStub has no Context")?;
        let attrs = fields
            .get("droidless:view:xml-attributes")
            .and_then(|values| values.first())
            .copied()
            .context("ViewStub has no XML attributes")?;
        let params = fields
            .get("droidless:view:layout-params")
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(Word::ZERO);
        let inflater = fields
            .get("droidless:stub:inflater")
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(Word::ZERO);
        let layout = self
            .attribute_set_value(attrs, "layout")?
            .filter(|value| value.kind == 1 && value.data != 0)
            .ok_or_else(|| {
                fault(
                    "Ljava/lang/IllegalArgumentException;",
                    "ViewStub requires a layout resource",
                )
            })?
            .data;
        let inflated_id = self
            .attribute_set_value(attrs, "inflatedId")?
            .map(|value| value.data);
        let index = self
            .view_mut(parent)?
            .children
            .iter()
            .position(|child| *child == stub)
            .context("ViewStub missing from parent")?;
        let roots = self.native_roots.len();
        self.native_roots
            .extend([stub, parent, context, attrs, params]);
        let inflated = (|| -> Result<Word> {
            let inflater = if inflater == Word::ZERO {
                self.layout_inflater_from(context)?
            } else {
                inflater
            };
            self.native_roots.push(inflater);
            let view = *self
                .invoke(
                    Method {
                        class: "Landroid/view/LayoutInflater;".into(),
                        name: "inflate".into(),
                        parameters: vec!["I".into(), "Landroid/view/ViewGroup;".into(), "Z".into()],
                        returns: "Landroid/view/View;".into(),
                    },
                    vec![inflater, Word::from(layout as i32), parent, Word::ZERO],
                    true,
                )?
                .first()
                .context("ViewStub inflater returned no View")?;
            self.native_roots.push(view);
            let params = if params == Word::ZERO {
                self.heap
                    .get(view)?
                    .fields
                    .get("droidless:view:layout-params")
                    .and_then(|values| values.first())
                    .copied()
                    .context("inflated View has no layout parameters")?
            } else {
                params
            };
            self.native_roots.push(params);
            if let Some(id) = inflated_id {
                self.view_mut(view)?.id = id;
            }
            self.invoke(
                Method {
                    class: "Landroid/view/ViewGroup;".into(),
                    name: "removeViewInLayout".into(),
                    parameters: vec!["Landroid/view/View;".into()],
                    returns: "V".into(),
                },
                vec![parent, stub],
                true,
            )?;
            self.invoke(
                Method {
                    class: "Landroid/view/ViewGroup;".into(),
                    name: "addView".into(),
                    parameters: vec![
                        "Landroid/view/View;".into(),
                        "I".into(),
                        "Landroid/view/ViewGroup$LayoutParams;".into(),
                    ],
                    returns: "V".into(),
                },
                vec![parent, view, Word::from(index as i32), params],
                true,
            )?;
            // ponytail: retain the inflated view strongly, like the current Reference profile;
            // use weak heap edges when Reference GC semantics are implemented.
            self.heap
                .get_mut(stub)?
                .fields
                .insert("droidless:stub:inflated".into(), vec![view]);
            Ok(view)
        })();
        self.native_roots.truncate(roots);
        inflated
    }
    fn themed_attribute(&self, context: Word, raw: &Value) -> Result<Value> {
        let attributes = self.styled_attributes(&self.theme_styles(context)?)?;
        let mut value = raw.clone();
        for _ in 0..32 {
            if value.kind != 2 {
                return Ok(value);
            }
            value = attributes
                .get(&value.data)
                .cloned()
                .with_context(|| format!("theme attribute ?0x{:08x} missing", value.data))?;
        }
        bail!("theme attribute reference cycle")
    }
    fn color_state_list(&mut self, raw: &Value) -> Result<Word> {
        self.color_entries(raw)?;
        let list = self.heap.instance("Landroid/content/res/ColorStateList;")?;
        let value = self.attribute(raw)?;
        let (key, word) = if raw.kind == 1 {
            ("resourceId", Word::from(raw.data as i32))
        } else if (0x1c..=0x1f).contains(&value.kind) {
            (
                "droidless:color-state-list:default",
                Word::from(value.data as i32),
            )
        } else {
            bail!("color selector requires resource reference");
        };
        self.heap
            .get_mut(list)?
            .fields
            .insert(key.into(), vec![word]);
        Ok(list)
    }
    fn color_states(&self, list: Word) -> Result<Vec<(Vec<i32>, i32)>> {
        let fields = &self.heap.get(list)?.fields;
        if let Some(arrays) = fields.get("droidless:color-state-list:arrays") {
            ensure!(arrays.len() == 2, "invalid ColorStateList arrays");
            let colors = self.color_int_array(arrays[1])?;
            let Data::Array { element, values } = &self.heap.get(arrays[0])?.data else {
                bail!("color specs require int[][]")
            };
            ensure!(
                element == "[I" && values.len() == colors.len() && values.len() <= 1024,
                "invalid or oversized ColorStateList arrays"
            );
            return values
                .iter()
                .zip(colors)
                .map(|(words, color)| {
                    let states =
                        self.color_int_array(*words.first().context("missing color spec")?)?;
                    ensure!(states.len() <= 128, "color state spec limit reached (128)");
                    Ok((states, color))
                })
                .collect();
        }
        if let Some(color) = fields
            .get("droidless:color-state-list:default")
            .and_then(|values| values.first())
        {
            return Ok(vec![(vec![], color.int()?)]);
        }
        let resource = fields
            .get("resourceId")
            .and_then(|values| values.first())
            .context("ColorStateList has no color or resource")?
            .int()? as u32;
        self.color_entries(&Value {
            kind: 1,
            data: resource,
            text: None,
        })
    }
    pub(crate) fn color_int_array(&self, array: Word) -> Result<Vec<i32>> {
        let Data::Array { element, values } = &self.heap.get(array)?.data else {
            bail!("color states require int[]")
        };
        ensure!(
            element == "I" && values.len() <= 4096,
            "invalid or oversized color state array"
        );
        values
            .iter()
            .map(|words| words.first().context("missing color state")?.int())
            .collect()
    }
    fn color_entries(&self, raw: &Value) -> Result<Vec<(Vec<i32>, i32)>> {
        let value = self.attribute(raw)?;
        if (0x1c..=0x1f).contains(&value.kind) {
            return Ok(vec![(vec![], value.data as i32)]);
        }
        ensure!(value.kind == 3, "unsupported color resource type");
        let name = value
            .text
            .as_deref()
            .context("color selector has no filename")?;
        let data = self
            .apk
            .files
            .get(name)
            .context("color selector XML missing")?;
        let selector = droidless_formats::xml::parse(data)?;
        ensure!(
            selector.name == "selector"
                && !selector.children.is_empty()
                && selector.children.len() <= 1024,
            "invalid color selector"
        );
        selector
            .children
            .iter()
            .map(|item| {
                ensure!(
                    item.name == "item" && item.children.is_empty(),
                    "invalid color selector item"
                );
                let color = self.attribute(
                    item.attr("color")
                        .context("color selector item missing color")?,
                )?;
                ensure!(
                    (0x1c..=0x1f).contains(&color.kind),
                    "color selector item requires flat color"
                );
                ensure!(
                    item.attributes
                        .keys()
                        .all(|name| name == "color" || name.starts_with("state_")),
                    "unsupported color selector item attribute"
                );
                let mut states = vec![];
                for (id, value) in &item.resource_attributes {
                    if *id == 0x0101_01a5 {
                        continue;
                    }
                    ensure!(
                        *id <= i32::MAX as u32 && value.kind == 0x12 && states.len() < 64,
                        "invalid color selector state"
                    );
                    states.push(if value.data != 0 {
                        *id as i32
                    } else {
                        -(*id as i32)
                    });
                }
                Ok((states, color.data as i32))
            })
            .collect()
    }
    fn attribute(&self, value: &Value) -> Result<Value> {
        let mut current = value.clone();
        for _ in 0..32 {
            if current.kind != 1 {
                return Ok(current);
            }
            // Fixed API-21 flat framework colors. Other framework resources stay unsupported.
            let color = match current.data {
                0 | 0x0106_000d => Some(0),
                0x0106_000b => Some(0xffff_ffff),
                0x0106_000c => Some(0xff00_0000),
                _ => None,
            };
            if let Some(color) = color {
                return Ok(Value {
                    kind: 0x1f,
                    data: color,
                    text: None,
                });
            }
            current = self
                .apk
                .resources
                .entries
                .get(&current.data)
                .and_then(|entry| entry.value.as_ref())
                .with_context(|| format!("resource @0x{:08x} missing or complex", current.data))?
                .clone();
        }
        bail!("resource reference cycle at @0x{:08x}", value.data)
    }
    fn inflate(
        &mut self,
        element: &Element,
        depth: usize,
        context: Word,
        inflater: Word,
        parent: Word,
    ) -> Result<Word> {
        let roots = self.native_roots.len();
        self.native_roots.extend([context, inflater, parent]);
        let result = self.inflate_inner(element, depth, context, inflater, parent);
        self.native_roots.truncate(roots);
        result
    }
    fn inflate_child(
        &mut self,
        parent: Word,
        element: &Element,
        depth: usize,
        context: Word,
        inflater: Word,
    ) -> Result<()> {
        let child = if element.name == "include" {
            self.inflate_id(
                element.number("layout").context("include missing layout")?,
                depth,
                context,
                inflater,
                parent,
                true,
            )?
        } else {
            self.inflate(element, depth, context, inflater, parent)?
        };
        if child == parent {
            return Ok(());
        }
        self.native_roots.push(child);
        self.inflated_layout_params(parent, child)?;
        self.invoke(
            Method {
                class: "Landroid/view/ViewGroup;".into(),
                name: "addView".into(),
                parameters: vec!["Landroid/view/View;".into()],
                returns: "V".into(),
            },
            vec![parent, child],
            true,
        )?;
        Ok(())
    }
    pub(crate) fn inflated_layout_params(&mut self, parent: Word, child: Word) -> Result<()> {
        let attrs = self
            .heap
            .get(child)?
            .fields
            .get("droidless:view:xml-attributes")
            .and_then(|values| values.first())
            .copied()
            .context("inflated View has no XML attributes")?;
        let params = self.invoke(
            Method {
                class: "Landroid/view/ViewGroup;".into(),
                name: "generateLayoutParams".into(),
                parameters: vec!["Landroid/util/AttributeSet;".into()],
                returns: "Landroid/view/ViewGroup$LayoutParams;".into(),
            },
            vec![parent, attrs],
            true,
        )?;
        ensure!(
            params.len() == 1
                && params[0] != Word::ZERO
                && self.is_a(
                    &self.heap.get(params[0])?.class,
                    "Landroid/view/ViewGroup$LayoutParams;"
                ),
            "invalid generated LayoutParams"
        );
        self.heap
            .get_mut(child)?
            .fields
            .insert("droidless:view:layout-params".into(), params);
        Ok(())
    }
    fn inflate_inner(
        &mut self,
        element: &Element,
        depth: usize,
        context: Word,
        inflater: Word,
        parent: Word,
    ) -> Result<Word> {
        ensure!(depth < 64, "layout XML nesting limit");
        ensure!(element.name != "merge", "merge requires an attached parent");
        if element.name == "include" {
            return self.inflate_id(
                element.number("layout").context("include missing layout")?,
                depth + 1,
                context,
                inflater,
                parent,
                false,
            );
        }
        let name = if element.name == "view" {
            element
                .text("class")
                .filter(|name| !name.is_empty())
                .context("view tag missing class")?
        } else {
            element.name.clone()
        };
        let class = if name.contains('.') {
            crate::vm::descriptor(&name)
        } else if ["View", "ViewGroup", "ViewStub"].contains(&name.as_str()) {
            format!("Landroid/view/{name};")
        } else {
            format!("Landroid/widget/{name};")
        };
        let attrs = self.heap.instance("Landroid/util/AttributeSet;")?;
        self.heap.get_mut(attrs)?.data = Data::Attributes {
            named: element.attributes.clone(),
            resources: element.resource_attributes.clone(),
        };
        self.native_roots.push(attrs);
        let supplied = self.create_inflater_view(inflater, parent, &name, context, attrs)?;
        let word = if supplied == Word::ZERO {
            self.new_instance(&class)?
        } else {
            supplied
        };
        self.native_roots.push(word);
        if supplied == Word::ZERO {
            self.construct_inflated_view(word, &class, context, attrs)?;
        }
        self.heap
            .get_mut(word)?
            .fields
            .insert("droidless:view:xml-attributes".into(), vec![attrs]);
        if self.is_a(&self.heap.get(word)?.class, "Landroid/view/ViewStub;") {
            let view_context = self
                .heap
                .get(word)?
                .fields
                .get("droidless:view:context")
                .and_then(|values| values.first())
                .copied()
                .unwrap_or(context);
            let clone = self.clone_layout_inflater(inflater, view_context)?;
            self.heap
                .get_mut(word)?
                .fields
                .insert("droidless:stub:inflater".into(), vec![clone]);
        }
        let mut view = self
            .heap
            .get(word)?
            .view
            .clone()
            .with_context(|| format!("unsupported layout View {}", element.name))?;
        view.id = element.number("id").unwrap_or(0);
        if let Some(raw) = element.attributes.get("padding") {
            view.padding = [dimension(&self.attribute(raw)?)?; 4];
        }
        for (name, raw) in &element.attributes {
            match name.as_str() {
                "text" => {
                    view.text = if raw.kind == 1 {
                        self.resource_text(raw.data)?
                    } else {
                        raw.display()
                    }
                }
                "contentDescription" => {
                    let text = if raw.kind == 0 {
                        None
                    } else if raw.kind == 1 {
                        Some(self.resource_text(raw.data)?)
                    } else {
                        Some(raw.display())
                    };
                    let description = if let Some(text) = &text {
                        self.heap.string(text.clone())?
                    } else {
                        Word::ZERO
                    };
                    if text.as_ref().is_some_and(|text| !text.is_empty()) {
                        self.heap.get_mut(word)?.fields.insert(
                            "droidless:important-for-accessibility".into(),
                            vec![Word::from(1)],
                        );
                    }
                    view.content_description = text;
                    self.heap.get_mut(word)?.fields.insert(
                        "droidless:view:content-description".into(),
                        vec![description],
                    );
                }
                "layout_width" => view.width = dimension(&self.attribute(raw)?)?,
                "layout_height" => view.height = dimension(&self.attribute(raw)?)?,
                "layout_gravity" => {
                    let gravity = self.attribute(raw)?.data as i32;
                    self.heap.get_mut(word)?.fields.insert(
                        "droidless:view:layout-gravity".into(),
                        vec![Word::from(gravity)],
                    );
                }
                "layout_weight" => {
                    view.weight = if raw.kind == 4 {
                        f32::from_bits(raw.data)
                    } else {
                        raw.data as f32
                    }
                }
                "numColumns" if view.grid.is_some() => {
                    view.grid.as_mut().unwrap().columns = raw.data as i32;
                }
                "stretchMode" if view.grid.is_some() => {
                    view.grid.as_mut().unwrap().stretch = raw.data as i32;
                }
                "columnWidth" | "horizontalSpacing" | "verticalSpacing" if view.grid.is_some() => {
                    let value = dimension(&self.attribute(raw)?)? as i32;
                    let grid = view.grid.as_mut().unwrap();
                    match name.as_str() {
                        "columnWidth" => grid.column_width = value,
                        "horizontalSpacing" => grid.horizontal_spacing = value,
                        _ => grid.vertical_spacing = value,
                    }
                }
                "orientation" => view.orientation = raw.data as i32,
                "clipChildren" => view.clip_children = self.attribute(raw)?.data != 0,
                "clipToPadding" => view.clip_to_padding = self.attribute(raw)?.data != 0,
                "scaleType" if view.kind == "ImageView" => {
                    ensure!(
                        (1..=7).contains(&raw.data),
                        "ImageView matrix scaling is unsupported"
                    );
                    view.image_scale = raw.data as i32;
                }
                "textSize" => view.text_size = dimension(&self.attribute(raw)?)?,
                "textColor" => {
                    let value = self.themed_attribute(context, raw)?;
                    let colors = self.color_state_list(&value)?;
                    view.text_color = self.invoke(
                        Method {
                            class: "Landroid/content/res/ColorStateList;".into(),
                            name: "getDefaultColor".into(),
                            parameters: vec![],
                            returns: "I".into(),
                        },
                        vec![colors],
                        true,
                    )?[0]
                        .int()? as u32;
                    self.heap
                        .get_mut(word)?
                        .fields
                        .insert("droidless:view:text-colors".into(), vec![colors]);
                    // ponytail: native text paints the default palette entry; update it from drawable states when stateful typography is implemented.
                }
                "src" | "srcCompat" if raw.kind == 1 => {
                    let id = raw.data;
                    if id == 0 {
                        view.image = None;
                        self.heap
                            .get_mut(word)?
                            .fields
                            .insert("droidless:image:drawable".into(), vec![Word::ZERO]);
                        continue;
                    }
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
                "paddingLeft" | "paddingTop" | "paddingRight" | "paddingBottom" => {
                    let edge = match name.as_str() {
                        "paddingLeft" => 0,
                        "paddingTop" => 1,
                        "paddingRight" => 2,
                        _ => 3,
                    };
                    view.padding[edge] = dimension(&self.attribute(raw)?)?;
                }
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
                "focusable" => {
                    self.heap.get_mut(word)?.fields.insert(
                        "droidless:setFocusable".into(),
                        vec![Word::from(i32::from(raw.data != 0))],
                    );
                }
                "focusableInTouchMode" => {
                    self.heap.get_mut(word)?.fields.insert(
                        "droidless:setFocusableInTouchMode".into(),
                        vec![Word::from(i32::from(raw.data != 0))],
                    );
                    if raw.data != 0 {
                        self.heap
                            .get_mut(word)?
                            .fields
                            .insert("droidless:setFocusable".into(), vec![Word::from(1)]);
                    }
                }
                "descendantFocusability" => {
                    let mode = [0x20000, 0x40000, 0x60000]
                        .get(raw.data as usize)
                        .copied()
                        .context("invalid XML descendant focusability")?;
                    self.heap.get_mut(word)?.fields.insert(
                        "droidless:descendant-focusability".into(),
                        vec![Word::from(mode)],
                    );
                }
                "inputType" if raw.data == 0 => {
                    view.editable = false;
                }
                _ => {} // Styling outside this subset is documented; execution APIs still fail explicitly.
            }
        }
        // Resolve relative XML edges after physical edges, independent of attribute ordering.
        for (name, edge) in [("paddingStart", 0), ("paddingEnd", 2)] {
            if let Some(raw) = element.attributes.get(name) {
                let padding = dimension(&self.attribute(raw)?)?;
                ensure!(padding >= 0.0, "relative padding must be non-negative");
                view.padding[edge] = padding;
                self.heap.get_mut(word)?.fields.insert(
                    "droidless:view:padding-relative".into(),
                    vec![Word::from(1)],
                );
            }
        }
        ensure!(
            view.weight.is_finite()
                && view.weight >= 0.0
                && view.text_size.is_finite()
                && view.text_size >= 0.0,
            "invalid layout numeric attribute"
        );
        self.heap.get_mut(word)?.view = Some(view);
        for child in &element.children {
            self.inflate_child(word, child, depth + 1, context, inflater)?;
        }
        self.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "onFinishInflate".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![word],
            true,
        )?;
        Ok(word)
    }
    pub(crate) fn drawable_color(&self, raw: &Value, depth: usize) -> Result<Option<u32>> {
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
fn dimension_pixel_size(value: &Value) -> Result<i32> {
    let pixels = dimension(value)?;
    let rounded = (pixels + 0.5).trunc() as i32;
    Ok(if rounded == 0 && pixels != 0.0 {
        if pixels.is_sign_positive() { 1 } else { -1 }
    } else {
        rounded
    })
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
fn java_decimal_literal(text: &str) -> Result<&str> {
    static DECIMAL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\A[+-]?(?:(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?[fFdD]?|NaN|Infinity)\z").unwrap()
    });
    let text = text.trim_matches(|c| c <= '\u{20}');
    let unsigned = text.trim_start_matches(['+', '-']);
    ensure!(
        !unsigned.starts_with("0x") && !unsigned.starts_with("0X"),
        "hexadecimal floating-point literals are unsupported"
    );
    ensure!(
        DECIMAL.is_match(text),
        fault(
            "Ljava/lang/NumberFormatException;",
            "invalid floating-point string"
        )
    );
    Ok(text.trim_end_matches(['f', 'F', 'd', 'D']))
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
    fn paint_shadow_state_retains_parameters_and_clears() {
        use crate::{Runtime, heap::Word};
        use droidless_formats::{apk::Apk, dex::Method};
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap(),
        )
        .unwrap();
        let paint = vm.heap.instance("Landroid/text/TextPaint;").unwrap();
        let call = |vm: &mut Runtime,
                    name: &str,
                    parameters: Vec<String>,
                    returns: &str,
                    values: Vec<Word>| {
            vm.invoke(
                Method {
                    class: "Landroid/graphics/Paint;".into(),
                    name: name.into(),
                    parameters,
                    returns: returns.into(),
                },
                std::iter::once(paint).chain(values).collect(),
                true,
            )
            .unwrap()
        };
        assert_eq!(
            call(&mut vm, "hasShadowLayer", vec![], "Z", vec![]),
            [Word::ZERO]
        );
        let values = vec![
            Word::Bits(2.0f32.to_bits()),
            Word::Bits(1.0f32.to_bits()),
            Word::Bits((-1.0f32).to_bits()),
            Word::Bits(0xff123456),
        ];
        call(
            &mut vm,
            "setShadowLayer",
            vec!["F".into(), "F".into(), "F".into(), "I".into()],
            "V",
            values.clone(),
        );
        assert_eq!(
            call(&mut vm, "hasShadowLayer", vec![], "Z", vec![]),
            [Word::from(1)]
        );
        assert_eq!(
            vm.heap.get(paint).unwrap().fields["droidless:paint:shadow"],
            values
        );
        call(&mut vm, "clearShadowLayer", vec![], "V", vec![]);
        assert_eq!(
            call(&mut vm, "hasShadowLayer", vec![], "Z", vec![]),
            [Word::ZERO]
        );
        assert_eq!(vm.stack_depth(), 0);
    }

    #[test]
    fn color_channels_and_packing_follow_java_argb_bits() {
        use crate::{Runtime, heap::Word};
        use droidless_formats::{apk::Apk, dex::Method};
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap(),
        )
        .unwrap();
        let mut call = |name: &str, args: Vec<Word>| {
            vm.invoke(
                Method {
                    class: "Landroid/graphics/Color;".into(),
                    name: name.into(),
                    parameters: vec!["I".into(); args.len()],
                    returns: "I".into(),
                },
                args,
                false,
            )
            .unwrap()[0]
        };
        for (name, expected) in [("alpha", 128), ("red", 18), ("green", 52), ("blue", 86)] {
            assert_eq!(
                call(name, vec![Word::Bits(0x80123456)]),
                Word::from(expected)
            );
            assert_eq!(call(name, vec![Word::from(-1)]), Word::from(255));
            assert_eq!(call(name, vec![Word::ZERO]), Word::ZERO);
        }
        assert_eq!(
            call(
                "argb",
                vec![128, 18, 52, 86].into_iter().map(Word::from).collect()
            ),
            Word::Bits(0x80123456)
        );
        assert_eq!(
            call(
                "rgb",
                vec![18, 52, 86].into_iter().map(Word::from).collect()
            ),
            Word::Bits(0xff123456)
        );
        assert_eq!(
            call("rgb", vec![0, 0, 256].into_iter().map(Word::from).collect()),
            Word::Bits(0xff000100)
        );
        assert_eq!(
            call(
                "argb",
                vec![0, 0, 0, -1].into_iter().map(Word::from).collect()
            ),
            Word::from(-1)
        );
    }

    #[test]
    fn rgb_to_hsv_handles_primary_and_gray_colors() {
        assert_eq!(rgb_to_hsv(0xffff_0000), [0.0, 1.0, 1.0]);
        assert_eq!(rgb_to_hsv(0xff00_ff00), [120.0, 1.0, 1.0]);
        assert_eq!(rgb_to_hsv(0xff00_00ff), [240.0, 1.0, 1.0]);
        assert_eq!(rgb_to_hsv(0xff80_8080), [0.0, 0.0, 128.0 / 255.0]);
    }
}
