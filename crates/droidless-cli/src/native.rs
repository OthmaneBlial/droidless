use anyhow::{Context, Result};
use droidless_runtime::{Runtime, ui::Node};
use std::ffi::{CStr, CString, c_char, c_void};
use std::os::unix::ffi::OsStringExt;

#[repr(C)]
struct NativeView {
    handle: usize,
    kind: u32,
    enabled: u32,
    editable: u32,
    visible: u32,
    foreground: u32,
    background: u32,
    has_background: u32,
    gravity: u32,
    key_listener: u32,
    image_scale: i32,
    font_family: u32,
    font_style: u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text_size: f32,
    alpha: f32,
    padding: [f32; 4],
    text: *const c_char,
    description: *const c_char,
    click_target: usize,
    image: *const u8,
    image_len: usize,
}
#[repr(C)]
struct NativeMotion {
    time: u64,
    action: i32,
    x: f32,
    y: f32,
}
type Callback = extern "C" fn(*mut c_void, u32, usize, *const c_char, *const NativeMotion) -> i32;
unsafe extern "C" {
    fn dl_open(
        title: *const c_char,
        width: f32,
        height: f32,
        context: *mut c_void,
        callback: Callback,
        uptime: u64,
    ) -> *mut c_void;
    fn dl_begin(host: *mut c_void, title: *const c_char, touch_enabled: u32, touch_active: u32);
    fn dl_view(host: *mut c_void, node: *const NativeView);
    fn dl_end(host: *mut c_void);
    fn dl_menu_clear(host: *mut c_void);
    fn dl_menu_item(
        host: *mut c_void,
        handle: usize,
        title: *const c_char,
        enabled: u32,
        checked: u32,
    );
    fn dl_run(host: *mut c_void);
    fn dl_has_window_focus(host: *mut c_void) -> i32;
    fn dl_destroy(host: *mut c_void);
    fn dl_choose_directory(host: *mut c_void, path: *mut *mut c_char) -> i32;
    fn dl_free_path(path: *mut c_char);
}
struct ContextData<'a> {
    runtime: &'a mut Runtime,
    host: *mut c_void,
    error: Option<anyhow::Error>,
}
extern "C" fn event(
    context: *mut c_void,
    kind: u32,
    handle: usize,
    text: *const c_char,
    motion: *const NativeMotion,
) -> i32 {
    // SAFETY: dl_run calls synchronously on the main thread while ContextData is alive.
    let context = unsafe { &mut *context.cast::<ContextData<'_>>() };
    let mut consumed = false;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
        // SAFETY: the main-thread event loop owns this live host for the callback's duration.
        context
            .runtime
            .set_host_window_focus(unsafe { dl_has_window_focus(context.host) != 0 });
        let dispatched = if kind == 6 {
            context.runtime.poll_messages()?
        } else {
            0
        };
        if context.runtime.activity.is_none() {
            return Ok(());
        }
        match kind {
            1 => {
                context.runtime.click(handle)?;
            }
            2 => {
                anyhow::ensure!(!text.is_null(), "native text callback missing text");
                // SAFETY: AppKit's UTF-8 string stays alive for this synchronous callback.
                let text = unsafe { CStr::from_ptr(text) }.to_str()?;
                context.runtime.edit(handle, text)?;
            }
            3 | 4 => {
                anyhow::ensure!(!text.is_null(), "native key event missing text");
                // SAFETY: AppKit's event string stays alive during this synchronous callback.
                let text = unsafe { CStr::from_ptr(text) }.to_str()?;
                consumed = context
                    .runtime
                    .key_text(handle, if kind == 3 { 0 } else { 1 }, text)?;
            }
            5 => {
                context.runtime.back()?;
                consumed = true;
            }
            6 => {}
            7 => {
                anyhow::ensure!(!motion.is_null(), "native touch event missing data");
                // SAFETY: AppKit keeps this fixed-size event alive throughout the synchronous call.
                let motion = unsafe { &*motion };
                context
                    .runtime
                    .touch_at(motion.action, motion.x, motion.y, motion.time)?;
                consumed = true;
            }
            8 => {
                let entries = context.runtime.options_menu()?;
                let titles = entries
                    .iter()
                    .map(|entry| CString::new(entry.title.as_str()))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .context("NUL in menu title is unsupported by AppKit bridge")?;
                // SAFETY: the live main-thread host copies each title before this call returns.
                unsafe {
                    dl_menu_clear(context.host);
                    for (entry, title) in entries.iter().zip(&titles) {
                        dl_menu_item(
                            context.host,
                            entry.handle,
                            title.as_ptr(),
                            u32::from(entry.enabled),
                            u32::from(entry.checked),
                        );
                    }
                    if entries.is_empty() {
                        dl_menu_item(context.host, 0, c"No options".as_ptr(), 0, 0);
                    }
                }
            }
            9 => {
                // Revalidate the guest handle against the foreground Activity's prepared menu.
                context.runtime.select_menu_item(handle)?;
            }
            _ => anyhow::bail!("unknown native event {kind}"),
        }
        if context.runtime.activity.is_some() && (kind != 6 || dispatched > 0) {
            draw(context)?;
        }
        if context.runtime.directory_picker_pending() {
            let mut path = std::ptr::null_mut();
            // SAFETY: host is live on the main thread; C returns an owned string or null on cancel.
            let selected = unsafe { dl_choose_directory(context.host, &mut path) };
            anyhow::ensure!(selected >= 0, "native directory picker failed");
            let path = if selected == 0 {
                None
            } else {
                anyhow::ensure!(!path.is_null(), "native directory picker returned no path");
                // SAFETY: selected paths are NUL-terminated and allocated by dl_choose_directory.
                let bytes = unsafe { CStr::from_ptr(path) }.to_bytes().to_vec();
                // SAFETY: release this C allocation exactly once, after copying its bytes.
                unsafe {
                    dl_free_path(path);
                }
                Some(std::path::PathBuf::from(std::ffi::OsString::from_vec(
                    bytes,
                )))
            };
            context.runtime.complete_directory_picker(path.as_deref())?;
            if context.runtime.activity.is_some() {
                draw(context)?;
            }
        }
        Ok(())
    }))
    .unwrap_or_else(|_| Err(anyhow::anyhow!("panic in native event callback")));
    if let Err(error) = result {
        eprintln!("DROIDLESS: {error:#}");
        context.error = Some(error);
        return 0;
    }
    if context.runtime.activity.is_none() {
        return 0;
    }
    if consumed { 2 } else { 1 }
}
fn draw(context: &mut ContextData<'_>) -> Result<()> {
    // SAFETY: drawing runs on the owning thread with the live host returned by dl_open.
    context
        .runtime
        .set_host_window_focus(unsafe { dl_has_window_focus(context.host) != 0 });
    fn node(host: *mut c_void, n: &Node, ancestor_click: usize, ancestor_alpha: f32) -> Result<()> {
        if n.view.visible != 0 {
            return Ok(());
        }
        let alpha = ancestor_alpha * n.view.alpha.clamp(0.0, 1.0);
        let click_target = if n.view.listener.is_some() || n.view.xml_click.is_some() {
            n.handle
        } else {
            ancestor_click
        };
        let text = CString::new(n.view.text.as_str())
            .context("NUL in UI text is unsupported by AppKit bridge")?;
        let description = n
            .view
            .content_description
            .as_deref()
            .map(CString::new)
            .transpose()
            .context("NUL in content description is unsupported by AppKit bridge")?;
        let image = n.view.image.as_deref().unwrap_or_default();
        let view = NativeView {
            handle: n.handle,
            kind: match n.view.kind.as_str() {
                "Button" => 1,
                "TextView" => 2,
                "EditText" => 3,
                "ImageView" => 4,
                _ => 0,
            },
            enabled: u32::from(n.view.enabled),
            editable: u32::from(n.view.editable),
            visible: n.view.visible as u32,
            foreground: n.view.text_color,
            background: n.view.background.unwrap_or(0),
            has_background: u32::from(n.view.background.is_some()),
            gravity: n.view.gravity,
            key_listener: u32::from(n.view.key_listener.is_some()),
            image_scale: n.view.image_scale,
            font_family: n.view.font_family,
            font_style: n.view.font_style,
            x: n.rect.x,
            y: n.rect.y,
            width: n.rect.width,
            height: n.rect.height,
            text_size: n.view.text_size,
            alpha,
            padding: n.view.padding,
            text: text.as_ptr(),
            description: description
                .as_ref()
                .map_or(std::ptr::null(), |label| label.as_ptr()),
            click_target,
            image: image.as_ptr(),
            image_len: image.len(),
        };
        // SAFETY: C copies the string and struct fields before returning. Host is retained by dl_open.
        unsafe {
            dl_view(host, &view);
        }
        for child in &n.children {
            node(host, child, click_target, alpha)?;
        }
        Ok(())
    }
    let tree = context.runtime.layout_snapshot()?;
    let title = CString::new(format!("{} — DROIDLESS", context.runtime.title.trim()))?;
    // SAFETY: the live host pointer comes only from dl_open and is used on the same thread.
    unsafe {
        dl_begin(
            context.host,
            title.as_ptr(),
            u32::from(context.runtime.touch_input_enabled()),
            u32::from(context.runtime.touch_active()),
        );
    }
    node(context.host, &tree, 0, 1.0)?;
    // SAFETY: same live host as above.
    unsafe {
        dl_end(context.host);
    }
    Ok(())
}
pub fn run(runtime: &mut Runtime) -> Result<()> {
    if runtime.activity.is_none() {
        return Ok(());
    }
    runtime.use_realtime_clock()?;
    let title = CString::new(format!("{} — DROIDLESS", runtime.title.trim()))?;
    let mut context = ContextData {
        runtime,
        host: std::ptr::null_mut(),
        error: None,
    };
    // SAFETY: called on the CLI main thread; context is pinned by its stack lifetime until dl_run returns.
    context.host = unsafe {
        dl_open(
            title.as_ptr(),
            context.runtime.width,
            context.runtime.height,
            (&mut context as *mut ContextData<'_>).cast(),
            event,
            context.runtime.uptime_ms(),
        )
    };
    anyhow::ensure!(!context.host.is_null(), "native window creation failed");
    let result = draw(&mut context);
    if result.is_ok() {
        // SAFETY: host and callback context both remain alive for this blocking event loop.
        unsafe {
            dl_run(context.host);
        }
    }
    // SAFETY: release the retained host exactly once, after its event loop has ended.
    unsafe {
        dl_destroy(context.host);
    }
    context.runtime.set_host_window_focus(false);
    result?;
    if let Some(error) = context.error {
        return Err(error);
    }
    Ok(())
}
