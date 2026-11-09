//! Google Picker uses explicit file selection with the narrow `drive.file` scope.

use std::cell::RefCell;
use wasm_bindgen::{JsCast as _, JsValue, closure::Closure};

thread_local! {
    // Keep the callback alive while Picker is open. It is replaced on the next opening.
    static PICKER_CALLBACK: RefCell<Option<Callback>> = RefCell::new(None);
}

type Callback = Closure<dyn FnMut(JsValue)>;

const PICKER_LOADER: &str = "https://apis.google.com/js/api.js";
const FOLDER_MIME: &str = "application/vnd.google-apps.folder";

pub(crate) struct PickedFile {
    pub id: String,
    pub name: String,
}

pub(crate) fn ready() -> bool {
    picker().is_some()
}

pub(crate) async fn load_library() -> Result<(), String> {
    if picker().is_some() {
        return Ok(());
    }
    super::auth::load_script(PICKER_LOADER).await?;
    let window = web_sys::window().ok_or("Browser window unavailable")?;
    let gapi = js_sys::Reflect::get(&window, &JsValue::from_str("gapi")).map_err(|_| "Google Picker is unavailable")?;
    let load = method(&gapi, "load")?;
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        let failed = reject.clone();
        let onerror = Closure::once_into_js(move || {
            let _ = failed.call0(&JsValue::NULL);
        });
        let timeout_reject = reject.clone();
        let ontimeout = Closure::once_into_js(move || {
            let _ = timeout_reject.call0(&JsValue::NULL);
        });
        let options = js_sys::Object::new();
        for (key, value) in [("callback", callback), ("onerror", onerror), ("ontimeout", ontimeout), ("timeout", JsValue::from_f64(15000.0))] {
            if js_sys::Reflect::set(&options, &JsValue::from_str(key), &value).is_err() {
                let _ = reject.call0(&JsValue::NULL);
                return;
            }
        }
        if load.call2(&gapi, &JsValue::from_str("picker"), &options).is_err() {
            let _ = reject.call0(&JsValue::NULL);
        }
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|_| "Google Picker is unavailable")?;
    picker().ok_or("Google Picker is unavailable")?;
    Ok(())
}

/// Open the Picker after the user has signed in. The key identifies the project, not a user.
pub(crate) fn open(
    token: &str,
    api_key: &str,
    project_number: &str,
    kind: crate::FileKind,
    done: impl FnOnce(Result<Option<PickedFile>, String>) + 'static,
) -> Result<(), String> {
    if token.is_empty() || api_key.is_empty() || project_number.is_empty() || !project_number.bytes().all(|c| c.is_ascii_digit()) {
        return Err("Google Drive is not configured".into());
    }
    let picker = picker().ok_or("Google Picker is not ready")?;
    let view_constructor = method(&picker, "DocsView")?;
    let view = js_sys::Reflect::construct(&view_constructor, &js_sys::Array::new()).map_err(|_| "Could not prepare Google Picker")?;
    // Browse My Drive by folder; folders can be opened but only the app's own kind of file picked.
    // The folder MIME type must be listed, or the filter hides folders too.
    call(&view, "setMimeTypes", &JsValue::from_str(&format!("{},{FOLDER_MIME}", kind.mime)))?;
    call(&view, "setIncludeFolders", &JsValue::TRUE)?;
    call(&view, "setSelectFolderEnabled", &JsValue::FALSE)?;
    call(&view, "setParent", &JsValue::from_str("root"))?;
    if let Some(list) = js_sys::Reflect::get(&picker, &JsValue::from_str("DocsViewMode"))
        .ok()
        .and_then(|modes| js_sys::Reflect::get(&modes, &JsValue::from_str("LIST")).ok())
        .filter(|mode| !mode.is_undefined())
    {
        call(&view, "setMode", &list)?;
    }
    let builder_constructor = method(&picker, "PickerBuilder")?;
    let builder = js_sys::Reflect::construct(&builder_constructor, &js_sys::Array::new()).map_err(|_| "Could not prepare Google Picker")?;
    call(&builder, "addView", &view)?;
    call(&builder, "setOAuthToken", &JsValue::from_str(token))?;
    call(&builder, "setDeveloperKey", &JsValue::from_str(api_key))?;
    call(&builder, "setAppId", &JsValue::from_str(project_number))?;
    let mut done = Some(done);
    let callback = Closure::wrap(Box::new(move |data: JsValue| {
        let field = |object: &JsValue, key: &str| js_sys::Reflect::get(object, &JsValue::from_str(key)).ok();
        let action = field(&data, "action").and_then(|v| v.as_string()).unwrap_or_default();
        if action != "cancel" && action != "picked" && action != "error" {
            return;
        }
        let result = if action == "cancel" {
            Ok(None)
        } else if action == "picked" {
            let docs = field(&data, "docs").map(|v| js_sys::Array::from(&v));
            let file = docs.filter(|a| a.length() == 1).map(|a| a.get(0));
            file.and_then(|f| {
                let id = field(&f, "id")?.as_string()?;
                let name = field(&f, "name")?.as_string()?;
                (crate::valid_file_id(&id) && crate::valid_file_name(kind, &name)).then_some(Some(PickedFile { id, name }))
            })
            .ok_or_else(|| "Google Picker did not return a supported file".to_string())
        } else {
            Err("Google Picker did not complete".to_string())
        };
        if let Some(done) = done.take() {
            done(result);
        }
    }) as Box<dyn FnMut(JsValue)>);
    call(&builder, "setCallback", callback.as_ref())?;
    PICKER_CALLBACK.with(|slot| *slot.borrow_mut() = Some(callback));
    let built = method(&builder, "build")?.call0(&builder).map_err(|_| "Could not open Google Picker")?;
    call(&built, "setVisible", &JsValue::TRUE)?;
    Ok(())
}

fn picker() -> Option<JsValue> {
    let window = web_sys::window()?;
    let google = js_sys::Reflect::get(&window, &JsValue::from_str("google")).ok()?;
    let value = js_sys::Reflect::get(&google, &JsValue::from_str("picker")).ok()?;
    (!value.is_undefined() && !value.is_null()).then_some(value)
}

fn method(object: &JsValue, name: &str) -> Result<js_sys::Function, String> {
    js_sys::Reflect::get(object, &JsValue::from_str(name))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
        .ok_or_else(|| "Google Picker is unavailable".to_string())
}

fn call(object: &JsValue, name: &str, value: &JsValue) -> Result<(), String> {
    method(object, name)?.call1(object, value).map_err(|_| "Could not prepare Google Picker")?;
    Ok(())
}
