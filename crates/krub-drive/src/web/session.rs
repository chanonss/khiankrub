//! A same-site backend remembers Drive consent; only short-lived access tokens reach the page.
use super::auth::AccessToken;
use wasm_bindgen::{JsCast as _, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

pub(crate) struct SessionToken {
    pub token: AccessToken,
    pub connection: String,
}
#[derive(Clone, Copy)]
pub(crate) enum Error {
    SignedOut,
    Unavailable,
}

async fn request(base: &str, path: &str) -> Result<web_sys::Response, Error> {
    let window = web_sys::window().ok_or(Error::Unavailable)?;
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_credentials(web_sys::RequestCredentials::Include);
    let headers = web_sys::Headers::new().map_err(|_| Error::Unavailable)?;
    headers.set("X-Khiankrub-Request", "1").map_err(|_| Error::Unavailable)?;
    init.set_headers(&headers.into());
    let mut attempt = 0;
    let response = loop {
        let value = JsFuture::from(window.fetch_with_str_and_init(&format!("{base}{path}"), &init)).await.map_err(|_| Error::Unavailable)?;
        let response = value.dyn_into::<web_sys::Response>().map_err(|_| Error::Unavailable)?;
        // Another tab may own the short renewal lease. Give it time to finish before retrying.
        if response.status() != 503 || !path.starts_with("/token") || attempt >= 8 {
            break response;
        }
        attempt += 1;
        pause(1500).await;
    };
    if response.status() == 401 {
        return Err(Error::SignedOut);
    }
    if !response.ok() {
        return Err(Error::Unavailable);
    }
    Ok(response)
}
pub(crate) async fn token(base: &str, refresh: bool) -> Result<SessionToken, Error> {
    let response = request(base, if refresh { "/token?refresh=true" } else { "/token" }).await?;
    let json = JsFuture::from(response.json().map_err(|_| Error::Unavailable)?).await.map_err(|_| Error::Unavailable)?;
    let field = |name| js_sys::Reflect::get(&json, &JsValue::from_str(name)).map_err(|_| Error::Unavailable);
    let value = field("access_token")?.as_string().ok_or(Error::Unavailable)?;
    let seconds = field("expires_in")?.as_f64().ok_or(Error::Unavailable)?;
    let connection = field("connection")?.as_string().filter(|s| s.len() == 43).ok_or(Error::Unavailable)?;
    let token = AccessToken::new(value, seconds).ok_or(Error::Unavailable)?;
    Ok(SessionToken { token, connection })
}
pub(crate) async fn disconnect(base: &str) -> Result<(), Error> {
    request(base, "/disconnect").await.map(|_| ())
}

pub(crate) async fn pause(milliseconds: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let callback = Closure::once_into_js(move || {
                let _ = resolve.call0(&JsValue::NULL);
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), milliseconds);
        }
    });
    let _ = JsFuture::from(promise).await;
}
