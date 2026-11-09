//! Coordinates explicit Google Drive actions with the browser and editor.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::{api, auth, picker, session};
use crate::{Config, DriveServices, FileKind};

/// What the app gives the Drive code: how to show a message and redraw.
pub struct Host {
    /// Show one of [`crate::MESSAGES`], in English: the app translates it.
    pub notify: Box<dyn Fn(&'static str)>,
    pub repaint: Box<dyn Fn()>,
    /// Set while an upload runs, so the page can warn before it is closed.
    pub busy: Rc<Cell<bool>>,
}

/// Something to do once the sign-in is restored.
type Action = Box<dyn FnOnce(&Drive)>;

/// A document (id, revision) opened from Drive with this file name and contents.
type Opened = (u64, u64, String, Vec<u8>);

/// A Drive file the user picked, with the content version it had when it was opened or last saved.
struct DriveFile {
    id: String,
    name: String,
    version: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Disconnected,
    Loading,
    Ready,
    SigningIn,
    Connected,
    Working,
}

struct State {
    phase: Phase,
    token: Option<auth::AccessToken>,
    opened: Option<Opened>,
    /// The file behind `opened`, waiting for the editor to open it as a document.
    fetched: Option<DriveFile>,
    /// The document (by id) that was opened from a Drive file and can be saved back to it.
    linked: Option<(u64, DriveFile)>,
    /// A document (id, revision) whose contents are now saved in its Drive file.
    saved: Option<(u64, u64)>,
    /// A save back to Drive waiting for sign-in: the document (id, revision) and its file bytes.
    pending_save: Option<(u64, u64, Vec<u8>)>,
    pending_action: Option<Action>,
    connection: Option<String>,
    epoch: u64,
    force_refresh: bool,
    files: Option<Vec<(String, String)>>,
}

#[derive(Clone)]
pub struct Drive {
    state: Rc<RefCell<State>>,
    host: Rc<Host>,
    app: &'static str,
    kind: FileKind,
    picker_key: Option<&'static str>,
    client_id: &'static str,
    project_number: &'static str,
    backend: Option<&'static str>,
}

impl Drive {
    /// Production credentials must be supplied explicitly; localhost can use the test client.
    /// `None` hides Drive (a public site built without its settings).
    pub fn configured(config: Config, host: Host) -> Option<Self> {
        let key = config.picker_key.filter(|key| !key.is_empty());
        let site = web_sys::window()?.location().hostname().ok()?;
        let (client_id, project_number) =
            crate::oauth_configuration(&site, config.client_id, config.project_number, crate::LOCAL_CLIENT_ID, crate::LOCAL_PROJECT_NUMBER)?;
        let this = Self {
            state: Rc::new(RefCell::new(State {
                phase: Phase::Disconnected,
                token: None,
                opened: None,
                fetched: None,
                linked: None,
                saved: None,
                pending_save: None,
                pending_action: None,
                connection: None,
                epoch: 0,
                force_refresh: false,
                files: None,
            })),
            host: Rc::new(host),
            app: config.app,
            kind: config.kind,
            picker_key: key,
            client_id,
            project_number,
            backend: crate::backend(config.backend_url, config.backend),
        };
        if this.backend.is_some() {
            this.restore_session();
        }
        Some(this)
    }

    pub fn services(&self) -> DriveServices {
        let status = self.clone();
        let connect = self.clone();
        let disconnect = self.clone();
        let open = self.clone();
        let save = self.clone();
        let files = self.clone();
        let select = self.clone();
        let linked = self.clone();
        let save_over = self.clone();
        DriveServices {
            status: Box::new(move || status.status()),
            connect: Box::new(move || connect.connect()),
            disconnect: self.backend.map(|_| Box::new(move || disconnect.disconnect()) as Box<dyn Fn()>),
            open: Box::new(move |document, revision| open.open(document, revision)),
            files: Box::new(move || files.state.borrow().files.clone()),
            select_file: Box::new(move |id, document, revision| select.select_file(id, document, revision)),
            save_copy: Box::new(move |name, bytes| save.save_copy(name, bytes)),
            linked_name: Box::new(move |document| {
                let state = linked.state.borrow();
                state.linked.as_ref().filter(|(linked, _)| *linked == document).map(|(_, file)| file.name.clone())
            }),
            save_over: Box::new(move |document, revision, bytes| save_over.save_over(document, revision, bytes)),
        }
    }

    pub fn take_opened(&self) -> Option<Opened> {
        self.state.borrow_mut().opened.take()
    }

    /// Called each frame: when the document changed to `document`, named `name`, it is the file
    /// from `take_opened` if the names match, and can be saved back to it. `None` drops that file.
    pub fn adopt(&self, document: Option<u64>, name: Option<&str>) {
        let mut state = self.state.borrow_mut();
        let Some(file) = state.fetched.take() else { return };
        if let Some(document) = document
            && name == Some(file.name.as_str())
        {
            state.linked = Some((document, file));
        }
    }

    pub fn take_saved(&self) -> Option<(u64, u64)> {
        self.state.borrow_mut().saved.take()
    }

    fn status(&self) -> &'static str {
        let state = self.state.borrow();
        match state.phase {
            Phase::Disconnected => "Not connected",
            Phase::Loading => "Loading Google Drive…",
            Phase::Ready => "Click Connect to sign in",
            Phase::SigningIn => "Signing in…",
            Phase::Connected if self.backend.is_some() || state.token.as_ref().and_then(auth::AccessToken::get).is_some() => "Connected",
            Phase::Connected => "Click Connect to sign in",
            Phase::Working => "Working with Google Drive…",
        }
    }

    fn notify(&self, text: &'static str) {
        (self.host.notify)(text);
        self.repaint();
    }

    fn repaint(&self) {
        (self.host.repaint)();
    }

    fn connect(&self) {
        if let Some(base) = self.backend {
            self.connect_persistent(base);
            return;
        }
        let phase = self.state.borrow().phase;
        match phase {
            Phase::Disconnected => {
                self.state.borrow_mut().phase = Phase::Loading;
                let this = self.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = async {
                        auth::load_library().await?;
                        if this.picker_key.is_some() {
                            picker::load_library().await?;
                        }
                        Ok::<(), String>(())
                    }
                    .await;
                    this.state.borrow_mut().phase = if result.is_ok() { Phase::Ready } else { Phase::Disconnected };
                    if result.is_err() {
                        this.notify("Could not load Google Drive. Check your connection and try again.");
                    } else if auth::click_still_active() {
                        // The Connect click still allows a sign-in window, so one click is enough.
                        // Otherwise the page shows "Click Connect to sign in" for a second click.
                        this.sign_in();
                    } else if this.state.borrow().pending_save.is_some() {
                        this.notify("Press Save again to sign in to Google Drive and save.");
                    }
                    this.repaint();
                });
            }
            // A sign-in whose window never appeared or was closed without an answer can be
            // started again: the stale request's answer, if it ever comes, is still accepted.
            Phase::Ready | Phase::Connected | Phase::SigningIn => self.sign_in(),
            Phase::Loading | Phase::Working => {}
        }
        self.repaint();
    }

    fn sign_in(&self) {
        self.state.borrow_mut().phase = Phase::SigningIn;
        let this = self.clone();
        if auth::request_token(self.client_id, move |result| {
            let mut state = this.state.borrow_mut();
            match result {
                Ok(token) => {
                    state.token = Some(token);
                    state.phase = Phase::Connected;
                }
                Err(_) => {
                    state.token = None;
                    state.phase = Phase::Ready;
                }
            }
            // A Save that was waiting for sign-in goes ahead once signed in, and is dropped if not.
            let waiting = state.pending_save.take().filter(|_| state.phase == Phase::Connected);
            drop(state);
            if this.state.borrow().phase == Phase::Ready {
                this.notify("Google Drive sign-in was not completed.");
            }
            if let Some((document, revision, bytes)) = waiting {
                this.save_over(document, revision, bytes);
            }
            this.repaint();
        })
        .is_err()
        {
            let mut state = self.state.borrow_mut();
            state.phase = Phase::Ready;
            state.pending_save = None;
            drop(state);
            self.notify("Could not start Google sign-in.");
        }
        self.repaint();
    }

    /// Refresh an existing server session; authorization is never opened automatically.
    fn queue_refresh(&self, action: impl FnOnce(&Drive) + 'static) -> bool {
        if self.backend.is_none() {
            return false;
        }
        if matches!(self.state.borrow().phase, Phase::Loading | Phase::Working | Phase::SigningIn) {
            self.notify("Google Drive is busy. Try saving again in a moment.");
            return true;
        }
        self.state.borrow_mut().pending_action = Some(Box::new(action));
        self.restore_session();
        true
    }

    fn restore_session(&self) {
        let Some(base) = self.backend else { return };
        self.state.borrow_mut().phase = Phase::Loading;
        let this = self.clone();
        let epoch = this.state.borrow().epoch;
        wasm_bindgen_futures::spawn_local(async move {
            let force = this.state.borrow().force_refresh;
            let result = session::token(base, force).await;
            if this.state.borrow().epoch != epoch {
                return;
            }
            if result.is_ok() && this.picker_key.is_some() && picker::load_library().await.is_err() {
                this.notify("Could not load Google Drive. Check your connection and try again.");
            }
            if this.state.borrow().epoch != epoch {
                return;
            }
            let mut state = this.state.borrow_mut();
            let had_pending = state.pending_action.is_some();
            let unavailable = matches!(result, Err(session::Error::Unavailable));
            let mut changed = false;
            match result {
                Ok(session) => {
                    changed = state.connection.as_ref().is_some_and(|old| *old != session.connection);
                    if changed {
                        state.linked = None;
                        state.fetched = None;
                        state.files = None;
                    }
                    state.connection = Some(session.connection);
                    state.force_refresh = false;
                    state.token = Some(session.token);
                    state.phase = Phase::Connected;
                }
                Err(_) => {
                    state.token = None;
                    state.phase = Phase::Disconnected;
                }
            }
            let ready = state.phase == Phase::Connected;
            let pending = state.pending_action.take().filter(|_| ready && !changed);
            drop(state);
            if changed {
                this.notify("Google Drive connection changed. Reopen the Drive file before saving back to it.");
            } else if had_pending && !ready {
                this.notify(if unavailable {
                    "Could not restore Google Drive. Check your connection and try again."
                } else {
                    "Please connect Google Drive first."
                });
            }
            if let Some(action) = pending {
                action(&this);
            }
            this.repaint();
        });
        self.repaint();
    }

    fn connect_persistent(&self, base: &'static str) {
        if matches!(self.state.borrow().phase, Phase::Loading | Phase::Working) {
            return;
        }
        let popup = web_sys::window().and_then(|window| {
            window
                .open_with_url_and_target_and_features(
                    &format!("{base}/oauth/start"),
                    &format!("{}-drive-connect", self.app),
                    "popup,width=560,height=720",
                )
                .ok()
                .flatten()
        });
        let Some(popup) = popup else {
            self.notify("Could not start Google sign-in.");
            return;
        };
        let mut state = self.state.borrow_mut();
        state.epoch = state.epoch.wrapping_add(1);
        let epoch = state.epoch;
        state.phase = Phase::SigningIn;
        drop(state);
        let this = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            for _ in 0..180 {
                session::pause(1000).await;
                if this.state.borrow().epoch != epoch {
                    return;
                }
                match session::token(base, false).await {
                    Ok(_) => {
                        let _ = popup.close();
                        this.restore_session();
                        return;
                    }
                    Err(_) if popup.closed().unwrap_or(false) => break,
                    Err(_) => {}
                }
            }
            if this.state.borrow().epoch == epoch {
                this.state.borrow_mut().phase = Phase::Disconnected;
                this.notify("Google Drive sign-in was not completed.");
            }
        });
        self.repaint();
    }

    fn disconnect(&self) {
        let Some(base) = self.backend else { return };
        if self.state.borrow().phase != Phase::Connected {
            return;
        }
        self.state.borrow_mut().phase = Phase::Working;
        let this = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if session::disconnect(base).await.is_ok() {
                let mut state = this.state.borrow_mut();
                state.epoch = state.epoch.wrapping_add(1);
                state.phase = Phase::Disconnected;
                state.token = None;
                state.connection = None;
                state.linked = None;
                state.fetched = None;
                state.files = None;
                state.pending_save = None;
                state.pending_action = None;
                drop(state);
                this.notify("Google Drive disconnected.");
            } else {
                this.state.borrow_mut().phase = Phase::Connected;
                this.notify("Could not disconnect Google Drive. Check your connection and try again.");
            }
            this.repaint();
        });
        self.repaint();
    }

    fn expired<T>(&self, result: &Result<T, String>) {
        if result.as_ref().is_err_and(|error| error == "Google Drive request failed (HTTP 401)") {
            let mut state = self.state.borrow_mut();
            state.token = None;
            state.force_refresh = true;
        }
    }

    fn token(&self) -> Option<String> {
        self.state.borrow().token.as_ref().and_then(auth::AccessToken::get).map(str::to_owned)
    }

    fn open(&self, document: u64, revision: u64) {
        let Some(token) = self.token() else {
            if !self.queue_refresh(move |drive| drive.open(document, revision)) {
                self.notify("Please connect Google Drive first.");
            }
            return;
        };
        if self.state.borrow().phase != Phase::Connected {
            return;
        }
        if self.backend.is_some() && self.picker_key.is_some() && !picker::ready() {
            self.state.borrow_mut().phase = Phase::Working;
            let this = self.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let ready = picker::load_library().await.is_ok();
                this.state.borrow_mut().phase = Phase::Connected;
                if ready {
                    this.open(document, revision);
                } else {
                    this.notify("Could not load Google Drive. Check your connection and try again.");
                }
                this.repaint();
            });
            return;
        }
        let Some(key) = self.picker_key else {
            self.state.borrow_mut().phase = Phase::Working;
            let this = self.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = api::list(&token, this.kind).await;
                this.expired(&result);
                this.state.borrow_mut().phase = Phase::Connected;
                match result {
                    Ok(files) => this.state.borrow_mut().files = Some(files),
                    Err(_) => this.notify("Could not list Google Drive files."),
                }
                this.repaint();
            });
            return;
        };
        self.state.borrow_mut().phase = Phase::Working;
        let this = self.clone();
        let picked_token = token.clone();
        if picker::open(&token, key, self.project_number, self.kind, move |result| match result {
            Ok(Some(file)) => {
                let this = this.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = fetch(&picked_token, &file.id, &file.name).await;
                    this.expired(&result);
                    match result {
                        Ok((bytes, fetched)) => {
                            let mut state = this.state.borrow_mut();
                            state.opened = Some((document, revision, file.name, bytes));
                            state.fetched = fetched;
                        }
                        Err(_) => this.notify("Could not open the Google Drive file."),
                    }
                    this.state.borrow_mut().phase = Phase::Connected;
                    this.repaint();
                });
            }
            Ok(None) => {
                this.state.borrow_mut().phase = Phase::Connected;
                this.repaint();
            }
            Err(_) => {
                this.state.borrow_mut().phase = Phase::Connected;
                this.notify("Could not open Google Picker.");
            }
        })
        .is_err()
        {
            self.state.borrow_mut().phase = Phase::Connected;
            self.notify("Could not open Google Picker.");
        }
    }

    fn select_file(&self, id: String, document: u64, revision: u64) {
        let Some(token) = self.token() else {
            if !self.queue_refresh(move |drive| drive.select_file(id, document, revision)) {
                self.notify("Please connect Google Drive first.");
            }
            return;
        };
        let name = {
            let state = self.state.borrow();
            if state.phase != Phase::Connected {
                return;
            }
            state.files.as_ref().and_then(|files| files.iter().find(|(file_id, _)| file_id == &id)).map(|(_, name)| name.clone())
        };
        let Some(name) = name else {
            return;
        };
        self.state.borrow_mut().phase = Phase::Working;
        let this = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = fetch(&token, &id, &name).await;
            this.expired(&result);
            match result {
                Ok((bytes, fetched)) => {
                    let mut state = this.state.borrow_mut();
                    state.opened = Some((document, revision, name, bytes));
                    state.fetched = fetched;
                }
                Err(_) => this.notify("Could not open the Google Drive file."),
            }
            let mut state = this.state.borrow_mut();
            state.phase = Phase::Connected;
            state.files = None;
            drop(state);
            this.repaint();
        });
    }

    fn save_copy(&self, name: String, bytes: Vec<u8>) {
        let Some(token) = self.token() else {
            if !self.queue_refresh(move |drive| drive.save_copy(name, bytes)) {
                self.notify("Please connect Google Drive first.");
            }
            return;
        };
        if self.state.borrow().phase != Phase::Connected {
            return;
        }
        self.state.borrow_mut().phase = Phase::Working;
        self.host.busy.set(true);
        let this = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api::upload_copy(&token, this.kind, &name, &bytes).await;
            this.expired(&result);
            this.host.busy.set(false);
            this.state.borrow_mut().phase = Phase::Connected;
            this.notify(match result {
                Ok(_) => "A new copy was saved to Google Drive.",
                Err(error) if error == "The document is too large for this upload" => "Google Drive copies are limited to 5 MiB.",
                Err(_) => "Could not save the copy to Google Drive.",
            });
        });
    }

    /// Replace the Drive file `document` was opened from, unless someone changed it there since.
    fn save_over(&self, document: u64, revision: u64, bytes: Vec<u8>) {
        if matches!(self.state.borrow().phase, Phase::Loading | Phase::Working) {
            self.notify("Google Drive is busy. Try saving again in a moment.");
            return;
        }
        // Signed out, or the sign-in expired (it lasts about an hour): sign in, then save.
        let Some(token) = self.token() else {
            if self.backend.is_some() {
                self.queue_refresh(move |drive| drive.save_over(document, revision, bytes));
                return;
            }
            self.state.borrow_mut().pending_save = Some((document, revision, bytes));
            self.notify("Signing in to Google Drive to save…");
            self.connect();
            return;
        };
        let target = {
            let state = self.state.borrow();
            if state.phase != Phase::Connected {
                return;
            }
            state.linked.as_ref().filter(|(linked, _)| *linked == document).map(|(_, file)| (file.id.clone(), file.version.clone()))
        };
        let Some((id, known)) = target else {
            return;
        };
        self.state.borrow_mut().phase = Phase::Working;
        self.host.busy.set(true);
        let this = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = match api::version(&token, &id).await {
                Ok(current) if current != known => Err("changed".to_string()),
                Ok(_) => api::update(&token, this.kind, &id, &bytes).await,
                Err(error) => Err(error),
            };
            this.expired(&result);
            this.host.busy.set(false);
            let mut state = this.state.borrow_mut();
            state.phase = Phase::Connected;
            if let Ok(version) = &result {
                if let Some((_, file)) = state.linked.as_mut().filter(|(linked, file)| *linked == document && file.id == id) {
                    file.version = version.clone();
                }
                state.saved = Some((document, revision));
            }
            drop(state);
            this.notify(match result {
                Ok(_) => "Saved to Google Drive.",
                Err(error) if error == "changed" => "This file changed on Google Drive after you opened it. Save a new copy instead.",
                Err(error) if error == "The document is too large for this upload" => "Google Drive saves are limited to 5 MiB.",
                Err(_) => "Could not save to Google Drive.",
            });
        });
    }
}

/// Download a picked file with the version it has now. Without a version it still opens,
/// but can't be saved back over safely, so it isn't linked.
async fn fetch(token: &str, id: &str, name: &str) -> Result<(Vec<u8>, Option<DriveFile>), String> {
    let version = api::version(token, id).await.ok();
    let bytes = api::download(token, id).await?;
    Ok((bytes, version.map(|version| DriveFile { id: id.to_string(), name: name.to_string(), version })))
}
