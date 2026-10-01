use crate::{
    heap::{Data, Word},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;
use std::collections::{HashSet, VecDeque};

pub(crate) struct ActivityResult {
    pub request: i32,
    pub code: i32,
    pub data: Word,
}

pub(crate) struct Screen {
    pub root: Option<Word>,
    pub title: String,
    pub intent: Word,
    pub finishing: bool,
    pub resumed: bool,
    pub caller: Option<(Word, i32)>,
    pub result_code: i32,
    pub result_data: Word,
    pub results: VecDeque<ActivityResult>,
    pub window_token: Word,
    pub menu: Option<Word>,
    pub menu_preparing: bool,
    pub menu_ready: bool,
}
pub(crate) enum Navigation {
    Start {
        intent: Word,
        caller: Option<(Word, i32)>,
    },
    Finish {
        activity: Word,
        code: i32,
        data: Word,
    },
    PickDirectory {
        caller: Word,
        request: i32,
    },
}
impl Screen {
    pub fn roots(&self) -> impl Iterator<Item = Word> + '_ {
        [self.intent, self.result_data, self.window_token]
            .into_iter()
            .chain(self.root)
            .chain(self.menu)
            .chain(self.caller.map(|(caller, _)| caller))
            .chain(self.results.iter().map(|result| result.data))
    }
}
impl Navigation {
    pub fn roots(&self) -> [Word; 3] {
        match self {
            Self::Start { intent, caller } => [
                *intent,
                caller.map_or(Word::ZERO, |(caller, _)| caller),
                Word::ZERO,
            ],
            Self::Finish { activity, data, .. } => [*activity, *data, Word::ZERO],
            Self::PickDirectory { caller, .. } => [*caller, Word::ZERO, Word::ZERO],
        }
    }
}

impl Runtime {
    pub(crate) fn application_context(&mut self, receiver: Word) -> Result<Word> {
        let object = if let Some(object) = self
            .statics
            .get("droidless:application")
            .and_then(|v| v.first())
            .copied()
        {
            object
        } else if self.is_a(&self.heap.get(receiver)?.class, "Landroid/app/Application;") {
            receiver
        } else {
            self.new_instance("Landroid/app/Application;")?
        };
        self.statics
            .insert("droidless:application".into(), vec![object]);
        Ok(object)
    }
    pub(crate) fn lifecycle_callbacks(&mut self, application: Word) -> Result<Word> {
        let object = self.heap.get(application)?;
        ensure!(
            self.is_a(&object.class, "Landroid/app/Application;"),
            "callback registry requires Application"
        );
        if let Some(list) = object
            .fields
            .get("droidless:lifecycle")
            .and_then(|v| v.first())
            .copied()
        {
            return Ok(list);
        }
        let list = self.heap.instance("Ljava/util/ArrayList;")?;
        self.heap.get_mut(list)?.data = Data::Collection {
            values: vec![],
            version: 0,
        };
        self.heap
            .get_mut(application)?
            .fields
            .insert("droidless:lifecycle".into(), vec![list]);
        Ok(list)
    }
    pub(crate) fn dispatch_activity_callback(&mut self, name: &str, args: &[Word]) -> Result<()> {
        let Some(application) = self
            .statics
            .get("droidless:application")
            .and_then(|v| v.first())
            .copied()
        else {
            return Ok(());
        };
        let Some(list) = self
            .heap
            .get(application)?
            .fields
            .get("droidless:lifecycle")
            .and_then(|v| v.first())
            .copied()
        else {
            return Ok(());
        };
        let callbacks = self.collection(list)?.0.to_vec();
        let roots = self.native_roots.len();
        self.native_roots.extend([application, list]);
        self.native_roots.extend(args.iter().copied());
        self.native_roots.extend(callbacks.iter().copied());
        let result = (|| -> Result<()> {
            let mut parameters = vec!["Landroid/app/Activity;".into()];
            if args.len() == 2 {
                parameters.push("Landroid/os/Bundle;".into());
            }
            for callback in callbacks {
                self.invoke(
                    Method {
                        class: "Landroid/app/Application$ActivityLifecycleCallbacks;".into(),
                        name: name.into(),
                        parameters: parameters.clone(),
                        returns: "V".into(),
                    },
                    std::iter::once(callback)
                        .chain(args.iter().copied())
                        .collect(),
                    true,
                )?;
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    pub fn activity_depth(&self) -> usize {
        self.back_stack.len()
    }
    pub(crate) fn screen(&self, activity: Word) -> Result<&Screen> {
        self.heap.get(activity)?;
        self.screens
            .get(&activity.reference()?)
            .context("unregistered Activity")
    }
    /// Embedding hosts report focus before dispatching guest callbacks. Headless has no focused host window.
    pub fn set_host_window_focus(&mut self, focused: bool) {
        self.host_window_focused = focused;
    }
    pub(crate) fn view_has_window_focus(&self, view: Word) -> Result<bool> {
        let token = self.view_window_token(view)?;
        Ok(self.host_window_focused
            && token != Word::ZERO
            && self
                .activity
                .and_then(|activity| self.screens.get(&activity.reference().ok()?))
                .is_some_and(|screen| screen.window_token == token))
    }
    pub(crate) fn view_window_token(&self, mut view: Word) -> Result<Word> {
        for _ in 0..128 {
            let object = self.heap.get(view)?;
            ensure!(object.view.is_some(), "window token requires a View");
            if let Some(screen) = self
                .screens
                .values()
                .find(|screen| screen.root == Some(view))
            {
                return Ok(screen.window_token);
            }
            view = object
                .fields
                .get("droidless:view:parent")
                .and_then(|v| v.first())
                .copied()
                .unwrap_or(Word::ZERO);
            if view == Word::ZERO {
                return Ok(Word::ZERO);
            }
        }
        anyhow::bail!("cyclic or too deep View attachment hierarchy")
    }
    pub(crate) fn set_content(&mut self, activity: Word, root: Word) -> Result<()> {
        self.screens
            .get_mut(&activity.reference()?)
            .context("unregistered Activity")?
            .root = Some(root);
        if self.activity == Some(activity) {
            self.root = Some(root);
        }
        Ok(())
    }
    pub(crate) fn set_title(&mut self, activity: Word, title: String) -> Result<()> {
        self.screens
            .get_mut(&activity.reference()?)
            .context("unregistered Activity")?
            .title = title.clone();
        if self.activity == Some(activity) {
            self.title = title;
        }
        Ok(())
    }
    fn activate(&mut self, activity: Word) -> Result<()> {
        let screen = self.screen(activity)?;
        let (root, title) = (screen.root, screen.title.clone());
        self.activity = Some(activity);
        self.root = root;
        self.title = title;
        Ok(())
    }
    pub(crate) fn create_screen(
        &mut self,
        class: &str,
        intent: Word,
        caller: Option<(Word, i32)>,
    ) -> Result<()> {
        let roots = self.native_roots.len();
        self.native_roots.push(intent);
        self.native_roots.extend(caller.map(|(caller, _)| caller));
        let result = self.create_screen_inner(class, intent, caller);
        self.native_roots.truncate(roots);
        result
    }
    fn create_screen_inner(
        &mut self,
        class: &str,
        intent: Word,
        caller: Option<(Word, i32)>,
    ) -> Result<()> {
        ensure!(
            self.back_stack.len() < 64,
            "Activity back stack limit reached (64)"
        );
        ensure!(
            self.is_a(class, "Landroid/app/Activity;"),
            "launcher is not an Activity: {class}"
        );
        let activity = self.new_instance(class)?;
        self.screens.insert(
            activity.reference()?,
            Screen {
                root: None,
                title: self.default_title.clone(),
                intent,
                finishing: false,
                resumed: false,
                caller,
                result_code: 0,
                result_data: Word::ZERO,
                results: VecDeque::new(),
                window_token: Word::ZERO,
                menu: None,
                menu_preparing: false,
                menu_ready: false,
            },
        );
        self.back_stack.push(activity);
        self.heap
            .get_mut(activity)?
            .fields
            .insert("droidless:fragment:state".into(), vec![Word::from(1)]);
        self.activate(activity)?;
        self.invoke(
            Method {
                class: class.into(),
                name: "<init>".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![activity],
            false,
        )?;
        for name in ["onCreate", "onStart", "onResume"] {
            self.lifecycle_call(
                activity,
                class,
                name,
                if name == "onCreate" {
                    vec![Word::ZERO]
                } else {
                    vec![]
                },
            )?;
        }
        let token = self.heap.instance("Landroid/os/Binder;")?;
        self.screens
            .get_mut(&activity.reference()?)
            .context("Activity missing")?
            .window_token = token;
        self.dispatch_global_layout(activity)?;
        Ok(())
    }
    fn dispatch_global_layout(&mut self, activity: Word) -> Result<()> {
        let Some(root) = self.screen(activity)?.root else {
            return Ok(());
        };
        let mut pending = vec![root];
        let mut seen = HashSet::new();
        let roots = self.native_roots.len();
        self.native_roots.extend([activity, root]);
        let result = (|| -> Result<()> {
            while let Some(view) = pending.pop() {
                if !seen.insert(view.reference()?) {
                    continue;
                }
                let object = self.heap.get(view)?;
                let observer = object
                    .fields
                    .get("droidless:view:tree-observer")
                    .and_then(|values| values.first())
                    .copied();
                let children = object
                    .view
                    .as_ref()
                    .context("expected a View")?
                    .children
                    .clone();
                pending.extend(children);
                if let Some(observer) = observer {
                    let listeners = self
                        .heap
                        .get(observer)?
                        .fields
                        .get("droidless:global-layout-listeners")
                        .cloned()
                        .unwrap_or_default();
                    self.native_roots.extend(listeners.iter().copied());
                    for listener in listeners {
                        self.invoke(
                            Method {
                                class: "Landroid/view/ViewTreeObserver$OnGlobalLayoutListener;"
                                    .into(),
                                name: "onGlobalLayout".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![listener],
                            true,
                        )?;
                    }
                }
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn queue_navigation(&mut self, navigation: Navigation) -> Result<()> {
        self.require_main_thread()?;
        ensure!(
            self.navigation.len() < 128,
            "pending navigation limit reached (128)"
        );
        if let Navigation::Finish { activity, .. } = &navigation {
            self.screens
                .get_mut(&activity.reference()?)
                .context("unregistered Activity")?
                .finishing = true;
        }
        self.navigation.push_back(navigation);
        Ok(())
    }
    pub(crate) fn finish_activity(&mut self, activity: Word) -> Result<()> {
        let screen = self.screen(activity)?;
        if screen.finishing {
            return Ok(());
        }
        let code = screen.result_code;
        let source = screen.result_data;
        let data = if source == Word::ZERO {
            source
        } else {
            self.snapshot_intent(source)?
        };
        self.queue_navigation(Navigation::Finish {
            activity,
            code,
            data,
        })
    }
    fn dispatch_results(&mut self, activity: Word) -> Result<()> {
        while let Some(result) = self
            .screens
            .get_mut(&activity.reference()?)
            .context("result Activity missing")?
            .results
            .pop_front()
        {
            let roots = self.native_roots.len();
            self.native_roots.extend([activity, result.data]);
            let outcome = self.invoke(
                Method {
                    class: "Landroid/app/Activity;".into(),
                    name: "onActivityResult".into(),
                    parameters: vec!["I".into(), "I".into(), "Landroid/content/Intent;".into()],
                    returns: "V".into(),
                },
                vec![
                    activity,
                    Word::from(result.request),
                    Word::from(result.code),
                    result.data,
                ],
                true,
            );
            self.native_roots.truncate(roots);
            outcome?;
        }
        Ok(())
    }
    pub(crate) fn enqueue_result(&mut self, activity: Word, result: ActivityResult) -> Result<()> {
        let Some(screen) = self.screens.get_mut(&activity.reference()?) else {
            return Ok(());
        };
        if !screen.finishing {
            ensure!(
                screen.results.len() < 128,
                "pending Activity result limit reached (128)"
            );
            screen.results.push_back(result);
        }
        Ok(())
    }
    pub(crate) fn flush_active_results(&mut self) -> Result<()> {
        let Some(activity) = self.activity else {
            return Ok(());
        };
        let screen = self.screen(activity)?;
        if screen.results.is_empty() || screen.finishing {
            return Ok(());
        }
        if screen.resumed {
            self.lifecycle_call(activity, "Landroid/app/Activity;", "onPause", vec![])?;
        }
        self.dispatch_results(activity)?;
        self.lifecycle_call(activity, "Landroid/app/Activity;", "onResume", vec![])?;
        Ok(())
    }
    pub(crate) fn drain_navigation(&mut self) -> Result<()> {
        self.drain_fragment_transactions()?;
        while let Some(action) = self.navigation.pop_front() {
            // Guest lifecycle callbacks may collect after an action leaves the queue.
            let roots = self.native_roots.len();
            self.native_roots.extend(action.roots());
            let result = (|| -> Result<()> {
                match action {
                    Navigation::Start { intent, caller } => {
                        let target = *self
                            .heap
                            .get(intent)?
                            .fields
                            .get("component")
                            .and_then(|v| v.first())
                            .context("implicit/external Intent unsupported")?;
                        let target = self.heap.text(target)?.to_owned();
                        let previous = self.activity;
                        if let Some(activity) = previous {
                            let class = self.heap.get(activity)?.class.clone();
                            self.lifecycle_call(activity, &class, "onPause", vec![])?;
                        }
                        self.create_screen(&target, intent, caller)?;
                        if let Some(activity) = previous {
                            let class = self.heap.get(activity)?.class.clone();
                            self.lifecycle_call(activity, &class, "onStop", vec![])?;
                        }
                    }
                    Navigation::Finish {
                        activity,
                        code,
                        data,
                    } => {
                        let Some(position) = self.back_stack.iter().position(|a| *a == activity)
                        else {
                            return Ok(());
                        };
                        let active = self.activity == Some(activity);
                        let class = self.heap.get(activity)?.class.clone();
                        if active && self.screen(activity)?.resumed {
                            self.lifecycle_call(activity, &class, "onPause", vec![])?;
                        }
                        if let Some((caller, request)) = self.screen(activity)?.caller {
                            self.enqueue_result(
                                caller,
                                ActivityResult {
                                    request,
                                    code,
                                    data,
                                },
                            )?;
                        }
                        if self
                            .directory_request
                            .is_some_and(|(caller, _)| caller == activity)
                        {
                            self.directory_request = None;
                        }
                        self.back_stack.remove(position);
                        let restore = (|| -> Result<()> {
                            if !active {
                                return Ok(());
                            }
                            if let Some(previous) = self.back_stack.last().copied() {
                                self.activate(previous)?;
                                let parent_class = self.heap.get(previous)?.class.clone();
                                for name in ["onRestart", "onStart"] {
                                    self.lifecycle_call(previous, &parent_class, name, vec![])?;
                                }
                                self.dispatch_results(previous)?;
                                self.lifecycle_call(previous, &parent_class, "onResume", vec![])?;
                            } else {
                                self.activity = None;
                                self.root = None;
                                self.stop_messages()?;
                            }
                            Ok(())
                        })();
                        let teardown = (|| -> Result<()> {
                            if active {
                                self.lifecycle_call(activity, &class, "onStop", vec![])?;
                            }
                            self.lifecycle_call(activity, &class, "onDestroy", vec![])
                        })();
                        self.screens.remove(&activity.reference()?);
                        if let (Err(primary), Err(secondary)) = (&restore, &teardown) {
                            anyhow::bail!(
                                "Activity result/restore failed: {primary:#}; finished Activity teardown also failed: {secondary:#}"
                            );
                        }
                        restore?;
                        teardown?;
                    }
                    Navigation::PickDirectory { caller, request } => {
                        if self.screen(caller)?.finishing {
                            return Ok(());
                        }
                        ensure!(
                            self.activity == Some(caller),
                            "directory picker from a stopped Activity is unsupported"
                        );
                        ensure!(
                            self.directory_request.is_none(),
                            "directory picker is already pending"
                        );
                        if self.screen(caller)?.resumed {
                            self.lifecycle_call(
                                caller,
                                "Landroid/app/Activity;",
                                "onPause",
                                vec![],
                            )?;
                        }
                        self.directory_request = Some((caller, request));
                    }
                };
                self.flush_active_results()?;
                Ok(())
            })();
            self.native_roots.truncate(roots);
            result?;
        }
        if self.activity.is_some() {
            ensure!(
                self.root.is_some(),
                "foreground Activity has no content View"
            );
        }
        Ok(())
    }
    pub fn back(&mut self) -> Result<()> {
        self.reset_budget();
        if let Some(activity) = self.activity {
            let class = self.heap.get(activity)?.class.clone();
            self.invoke(
                Method {
                    class,
                    name: "onBackPressed".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                vec![activity],
                true,
            )?;
            self.drain_navigation()?;
            self.collect();
        }
        Ok(())
    }
}
