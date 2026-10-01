use crate::{
    heap::{Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

pub(crate) const MERGER: &str = "Ldroidless/runtime/InflaterFactoryMerger;";
const INFLATER: &str = "Landroid/view/LayoutInflater;";
const FACTORY: &str = "Landroid/view/LayoutInflater$Factory;";
const FACTORY2: &str = "Landroid/view/LayoutInflater$Factory2;";
const CONTEXT: &str = "droidless:layout-inflater:context";
const FIRST: &str = "droidless:layout-inflater:factory";
const SECOND: &str = "droidless:layout-inflater:factory2";
const SET: &str = "droidless:layout-inflater:factory-set";

impl Runtime {
    fn inflater_field(&self, object: Word, key: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(object)?
            .fields
            .get(key)
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    pub(crate) fn cached_layout_inflater(&mut self, context: Word) -> Result<Word> {
        let cached = self.inflater_field(context, "droidless:layout-inflater")?;
        if cached != Word::ZERO {
            return Ok(cached);
        }
        let inflater = self.heap.instance(INFLATER)?;
        self.heap
            .get_mut(inflater)?
            .fields
            .insert(CONTEXT.into(), vec![context]);
        self.heap
            .get_mut(context)?
            .fields
            .insert("droidless:layout-inflater".into(), vec![inflater]);
        Ok(inflater)
    }
    pub(crate) fn clone_layout_inflater(&mut self, inflater: Word, context: Word) -> Result<Word> {
        self.heap.get(context)?;
        let clone = self.heap.instance(INFLATER)?;
        let mut fields = self.heap.get(inflater)?.fields.clone();
        fields.remove(SET);
        fields.insert(CONTEXT.into(), vec![context]);
        self.heap.get_mut(clone)?.fields = fields;
        Ok(clone)
    }
    pub(crate) fn layout_inflater_from(&mut self, context: Word) -> Result<Word> {
        let name = self.heap.string("layout_inflater".into())?;
        let words = self.invoke(
            Method {
                class: "Landroid/content/Context;".into(),
                name: "getSystemService".into(),
                parameters: vec!["Ljava/lang/String;".into()],
                returns: "Ljava/lang/Object;".into(),
            },
            vec![context, name],
            true,
        )?;
        let inflater = *words
            .first()
            .context("getSystemService returned no value")?;
        ensure!(
            inflater != Word::ZERO,
            fault("Ljava/lang/AssertionError;", "LayoutInflater not found")
        );
        ensure!(
            self.is_a(&self.heap.get(inflater)?.class, INFLATER),
            "layout_inflater service is not a LayoutInflater"
        );
        Ok(inflater)
    }
    fn call_inflater_factory(
        &mut self,
        factory: Word,
        factory2: Word,
        parent: Word,
        name: Word,
        context: Word,
        attrs: Word,
    ) -> Result<Word> {
        let (class, parameters, args) = if factory2 != Word::ZERO {
            (
                FACTORY2,
                vec![
                    "Landroid/view/View;",
                    "Ljava/lang/String;",
                    "Landroid/content/Context;",
                    "Landroid/util/AttributeSet;",
                ],
                vec![factory2, parent, name, context, attrs],
            )
        } else {
            (
                FACTORY,
                vec![
                    "Ljava/lang/String;",
                    "Landroid/content/Context;",
                    "Landroid/util/AttributeSet;",
                ],
                vec![factory, name, context, attrs],
            )
        };
        let result = self.invoke(
            Method {
                class: class.into(),
                name: "onCreateView".into(),
                parameters: parameters.into_iter().map(String::from).collect(),
                returns: "Landroid/view/View;".into(),
            },
            args,
            true,
        )?;
        let view = *result
            .first()
            .context("inflater factory returned no value")?;
        ensure!(
            view == Word::ZERO || self.heap.get(view)?.view.is_some(),
            "inflater factory returned a non-View"
        );
        Ok(view)
    }
    pub(crate) fn create_inflater_view(
        &mut self,
        inflater: Word,
        parent: Word,
        name: &str,
        context: Word,
        attrs: Word,
    ) -> Result<Word> {
        let factory = self.inflater_field(inflater, FIRST)?;
        let factory2 = self.inflater_field(inflater, SECOND)?;
        if factory == Word::ZERO && factory2 == Word::ZERO {
            return Ok(Word::ZERO);
        }
        let roots = self.native_roots.len();
        let name = self.heap.string(name.into())?;
        self.native_roots
            .extend([inflater, parent, name, context, attrs]);
        let result = self.call_inflater_factory(factory, factory2, parent, name, context, attrs);
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn inflater_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let recognized = if method.class == INFLATER {
            matches!(
                signature.as_str(),
                "from(Landroid/content/Context;)Landroid/view/LayoutInflater;"
                    | "cloneInContext(Landroid/content/Context;)Landroid/view/LayoutInflater;"
                    | "getContext()Landroid/content/Context;"
                    | "getFactory()Landroid/view/LayoutInflater$Factory;"
                    | "getFactory2()Landroid/view/LayoutInflater$Factory2;"
                    | "setFactory(Landroid/view/LayoutInflater$Factory;)V"
                    | "setFactory2(Landroid/view/LayoutInflater$Factory2;)V"
                    | "inflate(ILandroid/view/ViewGroup;)Landroid/view/View;"
                    | "inflate(ILandroid/view/ViewGroup;Z)Landroid/view/View;"
            )
        } else {
            method.class == MERGER
                && matches!(
                    signature.as_str(),
                    "onCreateView(Ljava/lang/String;Landroid/content/Context;Landroid/util/AttributeSet;)Landroid/view/View;"
                        | "onCreateView(Landroid/view/View;Ljava/lang/String;Landroid/content/Context;Landroid/util/AttributeSet;)Landroid/view/View;"
                )
        };
        if !recognized {
            return Ok(None);
        }
        self.require_main_thread()?;
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let argument = |index| {
            args.get(index)
                .copied()
                .context("LayoutInflater argument missing")
        };
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let result = (|| -> Result<Vec<Word>> {
            let receiver = argument(0)?;
            if method.class == MERGER {
                let parent_aware = method.parameters.len() == 4;
                let (parent, name, context, attrs) = if parent_aware {
                    (argument(1)?, argument(2)?, argument(3)?, argument(4)?)
                } else {
                    (Word::ZERO, argument(1)?, argument(2)?, argument(3)?)
                };
                for key in ["first", "second"] {
                    let pair = self
                        .heap
                        .get(receiver)?
                        .fields
                        .get(key)
                        .context("invalid inflater factory merger")?
                        .clone();
                    ensure!(pair.len() == 2, "invalid inflater factory merger pair");
                    let view = self.call_inflater_factory(
                        pair[0],
                        if parent_aware { pair[1] } else { Word::ZERO },
                        parent,
                        name,
                        context,
                        attrs,
                    )?;
                    if view != Word::ZERO {
                        return Ok(vec![view]);
                    }
                }
                return Ok(vec![Word::ZERO]);
            }
            match method.name.as_str() {
                "from" => Ok(vec![self.layout_inflater_from(receiver)?]),
                "getContext" => Ok(vec![self.inflater_field(receiver, CONTEXT)?]),
                "getFactory" => Ok(vec![self.inflater_field(receiver, FIRST)?]),
                "getFactory2" => Ok(vec![self.inflater_field(receiver, SECOND)?]),
                "cloneInContext" => Ok(vec![self.clone_layout_inflater(receiver, argument(1)?)?]),
                "setFactory" | "setFactory2" => {
                    ensure!(
                        !self.inflater_field(receiver, SET)?.truth(),
                        fault(
                            "Ljava/lang/IllegalStateException;",
                            "A factory has already been set on this LayoutInflater"
                        )
                    );
                    let factory = argument(1)?;
                    ensure!(
                        factory != Word::ZERO,
                        fault(
                            "Ljava/lang/NullPointerException;",
                            "Given factory can not be null"
                        )
                    );
                    let is_second = method.name == "setFactory2";
                    ensure!(
                        self.is_a(
                            &self.heap.get(factory)?.class,
                            if is_second { FACTORY2 } else { FACTORY }
                        ),
                        "invalid LayoutInflater factory type"
                    );
                    let old = self.inflater_field(receiver, FIRST)?;
                    let combined = if old == Word::ZERO {
                        factory
                    } else {
                        let old2 = self.inflater_field(receiver, SECOND)?;
                        let depth = if self.heap.get(old)?.class == MERGER {
                            self.inflater_field(old, "depth")?
                                .int()?
                                .checked_add(1)
                                .context("invalid inflater factory merge depth")?
                        } else {
                            1
                        };
                        // ponytail: 64 merged factory layers; use iterative traversal if deeper chains are needed.
                        ensure!(
                            depth <= 64,
                            "LayoutInflater factory merge limit reached (64)"
                        );
                        let merger = self.heap.instance(MERGER)?;
                        let fields = &mut self.heap.get_mut(merger)?.fields;
                        fields.insert(
                            "first".into(),
                            vec![factory, if is_second { factory } else { Word::ZERO }],
                        );
                        fields.insert("second".into(), vec![old, old2]);
                        fields.insert("depth".into(), vec![Word::from(depth)]);
                        merger
                    };
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.insert(SET.into(), vec![Word::from(1)]);
                    fields.insert(FIRST.into(), vec![combined]);
                    if is_second {
                        fields.insert(SECOND.into(), vec![combined]);
                    }
                    Ok(vec![])
                }
                "inflate" => {
                    let id = argument(1)?.int()? as u32;
                    let parent = argument(2)?;
                    let attach = if method.parameters.len() == 2 {
                        parent != Word::ZERO
                    } else {
                        argument(3)?.truth()
                    };
                    let context = self.inflater_field(receiver, CONTEXT)?;
                    let view = self.inflate_id(id, 0, context, receiver, parent, attach)?;
                    self.native_roots.push(view);
                    if parent != Word::ZERO && parent != view {
                        self.inflated_layout_params(parent, view)?;
                    }
                    if attach && parent != Word::ZERO && parent != view {
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
                        Ok(vec![parent])
                    } else {
                        Ok(vec![view])
                    }
                }
                _ => unreachable!(),
            }
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }
}
