//! Platform fragments without a View: queued additions and guest lifecycle callbacks.
use crate::{
    heap::{Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const FRAGMENT: &str = "Landroid/app/Fragment;";
const MANAGER: &str = "Landroid/app/FragmentManager;";
const TRANSACTION: &str = "Landroid/app/FragmentTransaction;";

impl Runtime {
    fn fragment_word(&self, object: Word, name: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(object)?
            .fields
            .get(name)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    fn fragment_list(&self, object: Word, name: &str) -> Result<Vec<Word>> {
        Ok(self
            .heap
            .get(object)?
            .fields
            .get(name)
            .cloned()
            .unwrap_or_default())
    }
    fn fragment_set(&mut self, object: Word, name: &str, value: Word) -> Result<()> {
        self.heap
            .get_mut(object)?
            .fields
            .insert(name.into(), vec![value]);
        Ok(())
    }
    fn fragment_callback(
        &mut self,
        fragment: Word,
        name: &str,
        parameters: &[&str],
        args: Vec<Word>,
        returns: &str,
    ) -> Result<Vec<Word>> {
        if self.trace.lifecycle {
            eprintln!(
                "fragment lifecycle: {}->{name}",
                self.heap.get(fragment)?.class
            );
        }
        self.invoke(
            Method {
                class: FRAGMENT.into(),
                name: name.into(),
                parameters: parameters.iter().map(|p| (*p).into()).collect(),
                returns: returns.into(),
            },
            std::iter::once(fragment).chain(args).collect(),
            true,
        )
    }
    fn move_fragment(&mut self, fragment: Word, target: i32) -> Result<()> {
        let mut state = self
            .fragment_word(fragment, "droidless:fragment:state")?
            .int()?;
        ensure!(
            (0..=4).contains(&state) && (0..=4).contains(&target),
            "invalid Fragment lifecycle state"
        );
        while state != target {
            self.fragment_set(
                fragment,
                "droidless:fragment:state",
                Word::from(if state < target { state + 1 } else { state - 1 }),
            )?;
            if state < target {
                match state {
                    0 => {
                        let manager = self.fragment_word(fragment, "droidless:fragment:manager")?;
                        let activity = self.fragment_word(manager, "activity")?;
                        self.fragment_set(fragment, "droidless:fragment:activity", activity)?;
                        self.fragment_callback(
                            fragment,
                            "onAttach",
                            &["Landroid/app/Activity;"],
                            vec![activity],
                            "V",
                        )?;
                        self.invoke(
                            Method {
                                class: "Landroid/app/Activity;".into(),
                                name: "onAttachFragment".into(),
                                parameters: vec![FRAGMENT.into()],
                                returns: "V".into(),
                            },
                            vec![activity, fragment],
                            true,
                        )?;
                        self.fragment_callback(
                            fragment,
                            "onCreate",
                            &["Landroid/os/Bundle;"],
                            vec![Word::ZERO],
                            "V",
                        )?;
                    }
                    1 => {
                        let activity =
                            self.fragment_word(fragment, "droidless:fragment:activity")?;
                        let inflater = self
                            .invoke(
                                Method {
                                    class: "Landroid/app/Activity;".into(),
                                    name: "getLayoutInflater".into(),
                                    parameters: vec![],
                                    returns: "Landroid/view/LayoutInflater;".into(),
                                },
                                vec![activity],
                                true,
                            )?
                            .first()
                            .copied()
                            .context("LayoutInflater result missing")?;
                        let view = self.fragment_callback(
                            fragment,
                            "onCreateView",
                            &[
                                "Landroid/view/LayoutInflater;",
                                "Landroid/view/ViewGroup;",
                                "Landroid/os/Bundle;",
                            ],
                            vec![inflater, Word::ZERO, Word::ZERO],
                            "Landroid/view/View;",
                        )?;
                        // ponytail: only headless fragments; mount returned Views when a real APK requires them.
                        ensure!(
                            view == [Word::ZERO],
                            "Fragment View mounting is unsupported"
                        );
                        self.fragment_callback(
                            fragment,
                            "onActivityCreated",
                            &["Landroid/os/Bundle;"],
                            vec![Word::ZERO],
                            "V",
                        )?;
                    }
                    2 => {
                        self.fragment_callback(fragment, "onStart", &[], vec![], "V")?;
                    }
                    3 => {
                        self.fragment_callback(fragment, "onResume", &[], vec![], "V")?;
                    }
                    _ => unreachable!(),
                }
                state += 1;
            } else {
                match state {
                    4 => {
                        self.fragment_callback(fragment, "onPause", &[], vec![], "V")?;
                    }
                    3 => {
                        self.fragment_callback(fragment, "onStop", &[], vec![], "V")?;
                    }
                    2 => {
                        self.fragment_callback(fragment, "onDestroyView", &[], vec![], "V")?;
                    }
                    1 => {
                        self.fragment_callback(fragment, "onDestroy", &[], vec![], "V")?;
                        self.fragment_callback(fragment, "onDetach", &[], vec![], "V")?;
                        for key in [
                            "droidless:fragment:activity",
                            "droidless:fragment:manager",
                            "droidless:fragment:added",
                        ] {
                            self.fragment_set(fragment, key, Word::ZERO)?;
                        }
                    }
                    _ => unreachable!(),
                }
                state -= 1;
            }
        }
        Ok(())
    }
    fn execute_fragment_transactions(&mut self, manager: Word) -> Result<bool> {
        self.require_main_thread()?;
        if self.fragment_word(manager, "executing")? != Word::ZERO {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "recursive fragment transaction execution",
            ));
        }
        self.fragment_set(manager, "executing", Word::from(1))?;
        let roots = self.native_roots.len();
        self.native_roots.push(manager);
        let result = (|| {
            let mut executed = false;
            for _ in 0..1024 {
                let pending = self.fragment_list(manager, "pending")?;
                if pending.is_empty() {
                    return Ok(executed);
                }
                self.native_roots.extend(pending.iter().copied());
                self.heap.get_mut(manager)?.fields.remove("pending");
                for transaction in pending {
                    for fragment in self.fragment_list(transaction, "adds")? {
                        if self.fragment_word(fragment, "droidless:fragment:added")? != Word::ZERO {
                            return Err(fault(
                                "Ljava/lang/IllegalStateException;",
                                "Fragment already added",
                            ));
                        }
                        let fragments = self
                            .heap
                            .get_mut(manager)?
                            .fields
                            .entry("fragments".into())
                            .or_default();
                        ensure!(fragments.len() < 64, "fragment limit reached (64)");
                        fragments.push(fragment);
                        self.fragment_set(fragment, "droidless:fragment:added", Word::from(1))?;
                        let activity = self.fragment_word(manager, "activity")?;
                        let state = self
                            .fragment_word(activity, "droidless:fragment:state")?
                            .int()?;
                        self.move_fragment(fragment, if state == 0 { 1 } else { state })?;
                    }
                }
                executed = true;
            }
            anyhow::bail!("fragment transaction dispatch limit reached (1024)")
        })();
        self.native_roots.truncate(roots);
        self.fragment_set(manager, "executing", Word::ZERO)?;
        result
    }
    pub(crate) fn drain_fragment_transactions(&mut self) -> Result<()> {
        for activity in self.back_stack.clone() {
            let manager = self.fragment_word(activity, "droidless:fragment:manager")?;
            if manager != Word::ZERO {
                self.execute_fragment_transactions(manager)?;
            }
        }
        Ok(())
    }
    pub(crate) fn fragments_before_activity(&mut self, activity: Word, name: &str) -> Result<()> {
        let manager = self.fragment_word(activity, "droidless:fragment:manager")?;
        if manager != Word::ZERO && ["onStart", "onResume", "onDestroy"].contains(&name) {
            self.execute_fragment_transactions(manager)?;
        }
        match name {
            "onPause" => self.move_activity_fragments(activity, 3),
            "onStop" => self.move_activity_fragments(activity, 2),
            "onDestroy" => {
                if manager != Word::ZERO {
                    self.fragment_set(manager, "destroyed", Word::from(1))?;
                }
                self.move_activity_fragments(activity, 0)?;
                if manager != Word::ZERO {
                    self.heap.get_mut(manager)?.fields.remove("fragments");
                    self.fragment_set(manager, "activity", Word::ZERO)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub(crate) fn fragments_after_activity(&mut self, activity: Word, name: &str) -> Result<()> {
        match name {
            "onCreate" => self.move_activity_fragments(activity, 2)?,
            "onStart" => self.move_activity_fragments(activity, 3)?,
            "onResume" => {
                self.move_activity_fragments(activity, 4)?;
                let manager = self.fragment_word(activity, "droidless:fragment:manager")?;
                if manager != Word::ZERO {
                    self.execute_fragment_transactions(manager)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn move_activity_fragments(&mut self, activity: Word, state: i32) -> Result<()> {
        self.fragment_set(activity, "droidless:fragment:state", Word::from(state))?;
        let manager = self.fragment_word(activity, "droidless:fragment:manager")?;
        if manager == Word::ZERO {
            return Ok(());
        }
        let fragments = self.fragment_list(manager, "fragments")?;
        let roots = self.native_roots.len();
        self.native_roots.extend([activity, manager]);
        self.native_roots.extend(fragments.iter().copied());
        let result = fragments
            .into_iter()
            .try_for_each(|f| self.move_fragment(f, state));
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn fragment_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let class = method.class.as_str();
        if ![FRAGMENT, MANAGER, TRANSACTION, "Landroid/app/Activity;"].contains(&class) {
            return Ok(None);
        }
        let sig = method.signature();
        let arg = |i| args.get(i).copied().context("missing fragment argument");
        let receiver = arg(0)?;
        let result = match (class, sig.as_str()) {
            ("Landroid/app/Activity;", "getFragmentManager()Landroid/app/FragmentManager;") => {
                self.screen(receiver)?;
                let mut manager = self.fragment_word(receiver, "droidless:fragment:manager")?;
                if manager == Word::ZERO {
                    manager = self.heap.instance(MANAGER)?;
                    if self.fragment_word(receiver, "droidless:fragment:state")? == Word::ZERO {
                        self.fragment_set(manager, "destroyed", Word::from(1))?;
                    } else {
                        self.fragment_set(manager, "activity", receiver)?;
                    }
                    self.fragment_set(receiver, "droidless:fragment:manager", manager)?;
                }
                vec![manager]
            }
            ("Landroid/app/Activity;", "onAttachFragment(Landroid/app/Fragment;)V") => vec![],
            (MANAGER, "beginTransaction()Landroid/app/FragmentTransaction;") => {
                let transaction = self.heap.instance(TRANSACTION)?;
                self.fragment_set(transaction, "manager", receiver)?;
                vec![transaction]
            }
            (MANAGER, "findFragmentByTag(Ljava/lang/String;)Landroid/app/Fragment;") => {
                let tag = arg(1)?;
                let mut found = Word::ZERO;
                if tag != Word::ZERO {
                    let text = self.heap.text(tag)?;
                    for fragment in self.fragment_list(receiver, "fragments")?.into_iter().rev() {
                        let other = self.fragment_word(fragment, "droidless:fragment:tag")?;
                        if other != Word::ZERO && self.heap.text(other)? == text {
                            found = fragment;
                            break;
                        }
                    }
                }
                vec![found]
            }
            (MANAGER, "executePendingTransactions()Z") => vec![Word::from(i32::from(
                self.execute_fragment_transactions(receiver)?,
            ))],
            (MANAGER, "isDestroyed()Z") => vec![self.fragment_word(receiver, "destroyed")?],
            (
                TRANSACTION,
                "add(Landroid/app/Fragment;Ljava/lang/String;)Landroid/app/FragmentTransaction;",
            ) => {
                self.require_main_thread()?;
                let fragment = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(fragment)?.class, FRAGMENT),
                    "transaction requires Fragment"
                );
                let tag = arg(2)?;
                if tag != Word::ZERO {
                    let old = self.fragment_word(fragment, "droidless:fragment:tag")?;
                    if old != Word::ZERO && self.heap.text(old)? != self.heap.text(tag)? {
                        return Err(fault(
                            "Ljava/lang/IllegalStateException;",
                            "cannot change Fragment tag",
                        ));
                    }
                    self.heap.text(tag)?;
                }
                let manager = self.fragment_word(receiver, "manager")?;
                let old = self.fragment_word(fragment, "droidless:fragment:manager")?;
                ensure!(
                    old == Word::ZERO || old == manager,
                    "Fragment belongs to a different manager"
                );
                let adds = self
                    .heap
                    .get_mut(receiver)?
                    .fields
                    .entry("adds".into())
                    .or_default();
                ensure!(adds.len() < 64, "fragment transaction limit reached (64)");
                adds.push(fragment);
                self.fragment_set(fragment, "droidless:fragment:manager", manager)?;
                if tag != Word::ZERO {
                    self.fragment_set(fragment, "droidless:fragment:tag", tag)?;
                }
                vec![receiver]
            }
            (TRANSACTION, "commit()I") | (TRANSACTION, "commitAllowingStateLoss()I") => {
                self.require_main_thread()?;
                let manager = self.fragment_word(receiver, "manager")?;
                if self.fragment_word(receiver, "committed")? != Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "commit already called",
                    ));
                }
                if self.fragment_word(manager, "destroyed")? != Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "Activity has been destroyed",
                    ));
                }
                let pending = self
                    .heap
                    .get_mut(manager)?
                    .fields
                    .entry("pending".into())
                    .or_default();
                ensure!(
                    pending.len() < 128,
                    "pending fragment transaction limit reached (128)"
                );
                pending.push(receiver);
                self.fragment_set(receiver, "committed", Word::from(1))?;
                vec![Word::from(-1)]
            }
            (FRAGMENT, "getActivity()Landroid/app/Activity;")
            | (FRAGMENT, "getContext()Landroid/content/Context;") => {
                vec![self.fragment_word(receiver, "droidless:fragment:activity")?]
            }
            (FRAGMENT, "getFragmentManager()Landroid/app/FragmentManager;") => {
                vec![self.fragment_word(receiver, "droidless:fragment:manager")?]
            }
            (FRAGMENT, "getTag()Ljava/lang/String;") => {
                vec![self.fragment_word(receiver, "droidless:fragment:tag")?]
            }
            (FRAGMENT, "getArguments()Landroid/os/Bundle;") => {
                vec![self.fragment_word(receiver, "arguments")?]
            }
            (FRAGMENT, "setArguments(Landroid/os/Bundle;)V") => {
                if self.fragment_word(receiver, "droidless:fragment:added")? != Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "Fragment already active",
                    ));
                }
                let bundle = arg(1)?;
                if bundle != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(bundle)?.class, "Landroid/os/Bundle;"),
                        "arguments require Bundle"
                    );
                }
                self.fragment_set(receiver, "arguments", bundle)?;
                vec![]
            }
            (FRAGMENT, "isAdded()Z") => {
                vec![self.fragment_word(receiver, "droidless:fragment:added")?]
            }
            (FRAGMENT, "isResumed()Z") => vec![Word::from(i32::from(
                self.fragment_word(receiver, "droidless:fragment:state")?
                    .int()?
                    == 4,
            ))],
            (FRAGMENT, "getView()Landroid/view/View;")
            | (
                FRAGMENT,
                "onCreateView(Landroid/view/LayoutInflater;Landroid/view/ViewGroup;Landroid/os/Bundle;)Landroid/view/View;",
            ) => vec![Word::ZERO],
            (FRAGMENT, "<init>()V")
            | (FRAGMENT, "onAttach(Landroid/app/Activity;)V")
            | (FRAGMENT, "onCreate(Landroid/os/Bundle;)V")
            | (FRAGMENT, "onActivityCreated(Landroid/os/Bundle;)V")
            | (FRAGMENT, "onStart()V")
            | (FRAGMENT, "onResume()V")
            | (FRAGMENT, "onPause()V")
            | (FRAGMENT, "onStop()V")
            | (FRAGMENT, "onDestroyView()V")
            | (FRAGMENT, "onDestroy()V")
            | (FRAGMENT, "onDetach()V") => {
                self.heap.get(receiver)?;
                vec![]
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
}
