//! Google Drive transfers. Callers obtain a short-lived OAuth token after user consent.
//! Nothing is sent until an explicit action calls a transfer. Tokens stay in memory.

use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::FileKind;

const MAX_DOWNLOAD_BYTES: u64 = 200 * 1024 * 1024;

/// List files of `kind` that the drive.file grant makes available to this application.
pub(crate) async fn list(token: &str, kind: FileKind) -> Result<Vec<(String, String)>, String> {
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    let headers = web_sys::Headers::new().map_err(|_| "Could not prepare Drive request")?;
    headers.set("Authorization", &format!("Bearer {token}")).map_err(|_| "Could not prepare Drive authorization")?;
    init.set_headers(&headers.into());
    let query = String::from(js_sys::encode_uri_component(&format!("trashed=false and mimeType='{}'", kind.mime)));
    let url = format!("https://www.googleapis.com/drive/v3/files?pageSize=100&fields=files(id%2Cname)&q={query}");
    let response = fetch(&url, &init).await?;
    let json = JsFuture::from(response.json().map_err(|_| "Invalid Drive response")?).await.map_err(|_| "Invalid Drive response")?;
    let files = js_sys::Reflect::get(&json, &JsValue::from_str("files")).map_err(|_| "Invalid Drive response")?;
    if !js_sys::Array::is_array(&files) {
        return Err("Invalid Drive response".into());
    }
    let mut result = Vec::new();
    for entry in js_sys::Array::from(&files).iter() {
        let id = js_sys::Reflect::get(&entry, &JsValue::from_str("id")).ok().and_then(|v| v.as_string());
        let name = js_sys::Reflect::get(&entry, &JsValue::from_str("name")).ok().and_then(|v| v.as_string());
        if let (Some(id), Some(name)) = (id, name)
            && crate::valid_file_id(&id)
            && crate::valid_file_name(kind, &name)
        {
            result.push((id, name));
        }
    }
    Ok(result)
}

/// Upload a new file; an existing Drive document is never overwritten by this operation.
pub(crate) async fn upload_copy(token: &str, kind: FileKind, name: &str, bytes: &[u8]) -> Result<String, String> {
    let (boundary, body) = crate::multipart_upload(kind, name, bytes)?;
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    let array = js_sys::Uint8Array::from(body.as_slice());
    init.set_body(&array.into());
    let headers = web_sys::Headers::new().map_err(|_| "Could not prepare Drive request")?;
    headers.set("Authorization", &format!("Bearer {token}")).map_err(|_| "Could not prepare Drive authorization")?;
    headers.set("Content-Type", &format!("multipart/related; boundary={boundary}")).map_err(|_| "Could not prepare Drive upload")?;
    init.set_headers(&headers.into());
    let response = fetch("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id", &init).await?;
    let json = JsFuture::from(response.json().map_err(|_| "Invalid Drive response")?).await.map_err(|_| "Invalid Drive response")?;
    let id = js_sys::Reflect::get(&json, &JsValue::from_str("id"))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|id| crate::valid_file_id(id))
        .ok_or("Drive did not return a valid file ID")?;
    Ok(id)
}

/// The file's current content version, which changes whenever anyone saves it.
pub(crate) async fn version(token: &str, id: &str) -> Result<String, String> {
    if !crate::valid_file_id(id) {
        return Err("Invalid Drive file ID".into());
    }
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    let headers = web_sys::Headers::new().map_err(|_| "Could not prepare Drive request")?;
    headers.set("Authorization", &format!("Bearer {token}")).map_err(|_| "Could not prepare Drive authorization")?;
    init.set_headers(&headers.into());
    let response = fetch(&format!("https://www.googleapis.com/drive/v3/files/{id}?fields=version"), &init).await?;
    version_of(response).await
}

/// Replace the contents of a file the user picked; Drive keeps the earlier contents as a revision.
pub(crate) async fn update(token: &str, kind: FileKind, id: &str, bytes: &[u8]) -> Result<String, String> {
    if !crate::valid_file_id(id) {
        return Err("Invalid Drive file ID".into());
    }
    crate::check_upload_size(bytes)?;
    let init = web_sys::RequestInit::new();
    init.set_method("PATCH");
    let array = js_sys::Uint8Array::from(bytes);
    init.set_body(&array.into());
    let headers = web_sys::Headers::new().map_err(|_| "Could not prepare Drive request")?;
    headers.set("Authorization", &format!("Bearer {token}")).map_err(|_| "Could not prepare Drive authorization")?;
    headers.set("Content-Type", kind.mime).map_err(|_| "Could not prepare Drive upload")?;
    init.set_headers(&headers.into());
    let url = format!("https://www.googleapis.com/upload/drive/v3/files/{id}?uploadType=media&fields=version");
    let response = fetch(&url, &init).await?;
    version_of(response).await
}

async fn version_of(response: web_sys::Response) -> Result<String, String> {
    let json = JsFuture::from(response.json().map_err(|_| "Invalid Drive response")?).await.map_err(|_| "Invalid Drive response")?;
    js_sys::Reflect::get(&json, &JsValue::from_str("version"))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|version| !version.is_empty() && version.len() <= 32 && version.bytes().all(|b| b.is_ascii_digit()))
        .ok_or_else(|| "Drive did not return a file version".to_string())
}

/// Download a selected binary file. A malformed ID can never change the request's destination.
pub(crate) async fn download(token: &str, id: &str) -> Result<Vec<u8>, String> {
    if !crate::valid_file_id(id) {
        return Err("Invalid Drive file ID".into());
    }
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    let headers = web_sys::Headers::new().map_err(|_| "Could not prepare Drive request")?;
    headers.set("Authorization", &format!("Bearer {token}")).map_err(|_| "Could not prepare Drive authorization")?;
    init.set_headers(&headers.into());
    let response = fetch(&format!("https://www.googleapis.com/drive/v3/files/{id}?alt=media"), &init).await?;
    let blob = JsFuture::from(response.blob().map_err(|_| "Could not read Drive file")?)
        .await
        .map_err(|_| "Could not read Drive file")?
        .dyn_into::<web_sys::Blob>()
        .map_err(|_| "Invalid Drive file response")?;
    if blob.size() > MAX_DOWNLOAD_BYTES as f64 {
        return Err("Drive file is too large to open".into());
    }
    let buffer = JsFuture::from(blob.array_buffer()).await.map_err(|_| "Could not read Drive file")?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

async fn fetch(url: &str, init: &web_sys::RequestInit) -> Result<web_sys::Response, String> {
    let window = web_sys::window().ok_or("Browser window unavailable")?;
    let request = web_sys::Request::new_with_str_and_init(url, init).map_err(|_| "Could not prepare Drive request")?;
    let result = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|_| "Could not connect to Google Drive")?;
    let response = result.dyn_into::<web_sys::Response>().map_err(|_| "Invalid Drive response")?;
    if !response.ok() {
        return Err(format!("Google Drive request failed (HTTP {})", response.status()));
    }
    Ok(response)
}
