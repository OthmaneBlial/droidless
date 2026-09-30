use crate::{heap::Word, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

pub(crate) struct Screen {
    pub root: Option<Word>,
    pub title: String,
    pub intent: Word,
    pub finishing: bool,
}
pub(crate) enum Navigation {
    Start(Word),
    Finish(Word),
}

impl Runtime {
    pub fn activity_depth(&self) -> usize {
        self.back_stack.len()
    }
    pub(crate) fn screen(&self, activity: Word) -> Result<&Screen> {
        self.heap.get(activity)?;
        self.screens
            .get(&activity.reference()?)
            .context("unregistered Activity")
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
    pub(crate) fn create_screen(&mut self, class: &str, intent: Word) -> Result<()> {
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
            },
        );
        self.back_stack.push(activity);
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
        Ok(())
    }
    pub(crate) fn queue_navigation(&mut self, navigation: Navigation) -> Result<()> {
        self.require_main_thread()?;
        ensure!(
            self.navigation.len() < 128,
            "pending navigation limit reached (128)"
        );
        if let Navigation::Finish(activity) = &navigation {
            self.screens
                .get_mut(&activity.reference()?)
                .context("unregistered Activity")?
                .finishing = true;
        }
        self.navigation.push_back(navigation);
        Ok(())
    }
    pub(crate) fn drain_navigation(&mut self) -> Result<()> {
        while let Some(action) = self.navigation.pop_front() {
            match action {
                Navigation::Start(intent) => {
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
                    self.create_screen(&target, intent)?;
                    if let Some(activity) = previous {
                        let class = self.heap.get(activity)?.class.clone();
                        self.lifecycle_call(activity, &class, "onStop", vec![])?;
                    }
                }
                Navigation::Finish(activity) => {
                    let Some(position) = self.back_stack.iter().position(|a| *a == activity) else {
                        continue;
                    };
                    let active = self.activity == Some(activity);
                    let class = self.heap.get(activity)?.class.clone();
                    if active {
                        self.lifecycle_call(activity, &class, "onPause", vec![])?;
                    }
                    self.back_stack.remove(position);
                    if active {
                        if let Some(previous) = self.back_stack.last().copied() {
                            self.activate(previous)?;
                            let parent_class = self.heap.get(previous)?.class.clone();
                            for name in ["onRestart", "onStart", "onResume"] {
                                self.lifecycle_call(previous, &parent_class, name, vec![])?;
                            }
                        } else {
                            self.activity = None;
                            self.root = None;
                            self.stop_messages()?;
                        }
                        self.lifecycle_call(activity, &class, "onStop", vec![])?;
                    }
                    self.lifecycle_call(activity, &class, "onDestroy", vec![])?;
                    self.screens.remove(&activity.reference()?);
                }
            }
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
