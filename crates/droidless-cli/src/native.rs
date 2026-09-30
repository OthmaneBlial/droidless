use anyhow::{Context, Result};
use droidless_runtime::{Runtime, ui::Node};
use std::ffi::{CStr, CString, c_char, c_void};

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
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text_size: f32,
    text: *const c_char,
}
type Callback = extern "C" fn(*mut c_void, u32, usize, *const c_char) -> i32;
unsafe extern "C" {
    fn dl_open(
        title: *const c_char,
        width: f32,
        height: f32,
        context: *mut c_void,
        callback: Callback,
    ) -> *mut c_void;
    fn dl_begin(host: *mut c_void, title: *const c_char);
    fn dl_view(host: *mut c_void, node: *const NativeView);
    fn dl_end(host: *mut c_void);
    fn dl_run(host: *mut c_void);
    fn dl_destroy(host: *mut c_void);
}
struct ContextData<'a> {
    runtime: &'a mut Runtime,
    host: *mut c_void,
    error: Option<anyhow::Error>,
}
extern "C" fn event(context: *mut c_void, kind: u32, handle: usize, text: *const c_char) -> i32 {
    // SAFETY: dl_run calls synchronously on the main thread while ContextData is alive.
    let context = unsafe { &mut *context.cast::<ContextData<'_>>() };
    let mut consumed = false;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
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
            _ => anyhow::bail!("unknown native event {kind}"),
        }
        if context.runtime.activity.is_some() && (kind != 6 || dispatched > 0) {
            draw(context)?;
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
    fn node(host: *mut c_void, n: &Node) -> Result<()> {
        let text = CString::new(n.view.text.as_str())
            .context("NUL in UI text is unsupported by AppKit bridge")?;
        let view = NativeView {
            handle: n.handle,
            kind: match n.view.kind.as_str() {
                "Button" => 1,
                "TextView" => 2,
                "EditText" => 3,
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
            x: n.rect.x,
            y: n.rect.y,
            width: n.rect.width,
            height: n.rect.height,
            text_size: n.view.text_size,
            text: text.as_ptr(),
        };
        // SAFETY: C copies the string and struct fields before returning. Host is retained by dl_open.
        unsafe {
            dl_view(host, &view);
        }
        for child in &n.children {
            node(host, child)?;
        }
        Ok(())
    }
    let tree = context.runtime.snapshot()?;
    let title = CString::new(format!("{} — DROIDLESS", context.runtime.title.trim()))?;
    // SAFETY: the live host pointer comes only from dl_open and is used on the same thread.
    unsafe {
        dl_begin(context.host, title.as_ptr());
    }
    node(context.host, &tree)?;
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
    result?;
    if let Some(error) = context.error {
        return Err(error);
    }
    Ok(())
}
