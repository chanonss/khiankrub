//! Google Identity Services, loaded only after the user explicitly asks to connect.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{JsCast as _, JsValue, closure::Closure};

const GIS_URL: &str = "https://accounts.google.com/gsi/client";
const DRIVE_FILE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";

/// The access token is never persisted or written to a log.
pub(crate) struct AccessToken {
    value: String,
    expires_at_ms: f64,
}

impl AccessToken {
    pub(super) fn new(value: String, seconds: f64) -> Option<Self> {
        (!value.is_empty() && value.len() <= 16384 && seconds.is_finite() && seconds > 30.0)
            .then(|| Self { value, expires_at_ms: js_sys::Date::now() + seconds.min(86_400.0) * 1000.0 })
    }

    /// Return a token only while it has at least 30 seconds of validity left.
    pub(super) fn get(&self) -> Option<&str> {
        (js_sys::Date::now() + 30_000.0 < self.expires_at_ms).then_some(self.value.as_str())
    }
}

/// Whether the browser still treats the page as just clicked, so it allows a sign-in window.
/// False where the browser cannot say (`navigator.userActivation` is missing).
pub(crate) fn click_still_active() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w.navigator(), &JsValue::from_str("userActivation")).ok())
        .filter(|activation| activation.is_object())
        .and_then(|activation| js_sys::Reflect::get(&activation, &JsValue::from_str("isActive")).ok())
        .and_then(|active| active.as_bool())
        .unwrap_or(false)
}

/// Load Google's sign-in library following a user-initiated action.
/// The token is requested at once if that click is still active, otherwise after a second click.
pub(crate) async fn load_library() -> Result<(), String> {
    if oauth().is_some() {
        return Ok(());
    }
    load_script(GIS_URL).await?;
    oauth().ok_or_else(|| "Google sign-in is unavailable".to_string())?;
    Ok(())
}

pub(crate) async fn load_script(url: &str) -> Result<(), String> {
    let document = web_sys::window().and_then(|w| w.document()).ok_or("Browser document unavailable")?;
    let script = document.create_element("script").map_err(|_| "Could not load Google sign-in")?;
    script.set_attribute("src", url).map_err(|_| "Could not load Google sign-in")?;
    script.set_attribute("async", "").map_err(|_| "Could not load Google sign-in")?;
    let element = script.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let success = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
        let failure = Closure::once_into_js(move || {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("Could not load Google sign-in"));
        });
        let _ = js_sys::Reflect::set(&element, &JsValue::from_str("onload"), &success);
        let _ = js_sys::Reflect::set(&element, &JsValue::from_str("onerror"), &failure);
    });
    document.body().ok_or("Browser document unavailable")?.append_child(&script).map_err(|_| "Could not load Google sign-in")?;
    wasm_bindgen_futures::JsFuture::from(promise).await.map_err(|_| "Could not load Google sign-in")?;
    Ok(())
}

type Completion = Rc<RefCell<Option<Box<dyn FnOnce(Result<AccessToken, String>)>>>>;

/// Request narrowly scoped Drive access. Must be called directly from a user action.
pub(crate) fn request_token(client_id: &str, done: impl FnOnce(Result<AccessToken, String>) + 'static) -> Result<(), String> {
    if client_id.is_empty() || client_id.len() > 512 {
        return Err("Google Drive is not configured".into());
    }
    let oauth = oauth().ok_or("Google sign-in is not ready")?;
    let init = js_sys::Reflect::get(&oauth, &JsValue::from_str("initTokenClient"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
        .ok_or("Google sign-in is not ready")?;
    let done: Completion = Rc::new(RefCell::new(Some(Box::new(done))));
    let finish = done.clone();
    let callback = Closure::once_into_js(move |response: JsValue| {
        let get = |name| js_sys::Reflect::get(&response, &JsValue::from_str(name)).ok();
        let result = match (get("access_token").and_then(|v| v.as_string()), get("expires_in").and_then(|v| v.as_f64())) {
            (Some(value), Some(seconds)) if !value.is_empty() && seconds.is_finite() && seconds > 0.0 => {
                Ok(AccessToken { value, expires_at_ms: js_sys::Date::now() + seconds.min(86_400.0) * 1000.0 })
            }
            _ => Err("Google Drive sign-in was not completed".to_string()),
        };
        if let Some(done) = finish.borrow_mut().take() {
            done(result);
        }
    });
    let failure = Closure::once_into_js(move |_error: JsValue| {
        if let Some(done) = done.borrow_mut().take() {
            done(Err("Google Drive sign-in was not completed".into()));
        }
    });
    let options = js_sys::Object::new();
    for (name, value) in [
        ("client_id", JsValue::from_str(client_id)),
        ("scope", JsValue::from_str(DRIVE_FILE_SCOPE)),
        ("callback", callback),
        ("error_callback", failure),
    ] {
        js_sys::Reflect::set(&options, &JsValue::from_str(name), &value).map_err(|_| "Could not prepare Google sign-in")?;
    }
    let client = init.call1(&oauth, &options).map_err(|_| "Could not prepare Google sign-in")?;
    let request = js_sys::Reflect::get(&client, &JsValue::from_str("requestAccessToken"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
        .ok_or("Google sign-in is not ready")?;
    request.call0(&client).map_err(|_| "Could not start Google sign-in")?;
    Ok(())
}

fn oauth() -> Option<JsValue> {
    let mut value: JsValue = web_sys::window()?.into();
    for part in ["google", "accounts", "oauth2"] {
        value = js_sys::Reflect::get(&value, &JsValue::from_str(part)).ok()?;
        if value.is_null() || value.is_undefined() {
            return None;
        }
    }
    Some(value)
}
