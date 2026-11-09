//! Google Drive for the KrubOffice web apps (KhianKrub, PhutKrub, KhitKrub): connect with one
//! click, open a file the user picks, save back over it, save new copies, and stay signed in
//! through the Drive connection service (`apps/khiankrub-drive-worker` in KhianKrub).
//!
//! Shared source: keep this crate identical in all three KrubOffice repositories. Each app gives
//! it a [`Config`] (its file type and build settings) and shows [`DriveServices`] in its File
//! pages. Only the `drive.file` scope is used: the app sees files the user picked or it created.
//! Nothing is sent until the user clicks; access tokens stay in memory.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod protocol;
#[cfg(target_arch = "wasm32")]
mod web;

pub use protocol::{backend, check_upload_size, multipart_upload, oauth_configuration, valid_file_id, valid_file_name};
#[cfg(target_arch = "wasm32")]
pub use web::{Drive, Host};

/// The kind of file an app keeps on Drive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileKind {
    pub mime: &'static str,
    /// Lower case, without the dot.
    pub extension: &'static str,
}

pub const DOCX: FileKind = FileKind { mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document", extension: "docx" };
pub const PPTX: FileKind = FileKind { mime: "application/vnd.openxmlformats-officedocument.presentationml.presentation", extension: "pptx" };
pub const XLSX: FileKind = FileKind { mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", extension: "xlsx" };

/// Google's test web client (project `khiankrub-512900`), allowed only on localhost. Its ID is
/// public; there is no client secret in any app.
pub const LOCAL_CLIENT_ID: &str = "111030744924-0do3c52ks064c0dmpf9ind9g60eh99ok.apps.googleusercontent.com";
pub const LOCAL_PROJECT_NUMBER: &str = "111030744924";

/// One app's Drive settings. The `Option`s come from `option_env!` in the app's web crate.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Short lower-case app name (`khiankrub`): names the sign-in window.
    pub app: &'static str,
    pub kind: FileKind,
    pub client_id: Option<&'static str>,
    pub project_number: Option<&'static str>,
    /// Google Picker key; without it Open lists the app's own files instead.
    pub picker_key: Option<&'static str>,
    /// The Drive connection service the build asks for, used only if it is `backend`.
    pub backend_url: Option<&'static str>,
    /// The one connection service this app may use (`https://drive.khiankrub.app`).
    pub backend: &'static str,
}

/// A file in the list Open shows without a Picker: (Drive id, name).
pub type ListedFile = (String, String);

/// Google Drive actions the File pages offer. Saving is always an explicit click: a new copy,
/// or replacing the Drive file the document was opened from.
pub struct DriveServices {
    /// One of [`STATUS`].
    pub status: Box<dyn Fn() -> &'static str>,
    pub connect: Box<dyn Fn()>,
    /// Present when the sign-in is kept by the connection service, which can forget it.
    pub disconnect: Option<Box<dyn Fn()>>,
    /// Pick a file to open into the document (id, revision) on screen now.
    pub open: Box<dyn Fn(u64, u64)>,
    /// The app's own files, listed when there is no Picker.
    pub files: Box<dyn Fn() -> Option<Vec<ListedFile>>>,
    pub select_file: Box<dyn Fn(String, u64, u64)>,
    /// Upload a new file with this name and contents.
    pub save_copy: Box<dyn Fn(String, Vec<u8>)>,
    /// The name of the Drive file a document (by id) was opened from, if it can be saved back.
    pub linked_name: Box<dyn Fn(u64) -> Option<String>>,
    /// Replace that Drive file with the document (id, revision) as these bytes.
    pub save_over: Box<dyn Fn(u64, u64, Vec<u8>)>,
}

/// What [`DriveServices::status`] says, in English (each app translates them).
pub const STATUS: &[&str] =
    &["Not connected", "Loading Google Drive…", "Click Connect to sign in", "Signing in…", "Connected", "Working with Google Drive…"];

/// Every message the Drive code shows in the status bar, in English, for the apps' translations.
pub const MESSAGES: &[&str] = &[
    "Could not load Google Drive. Check your connection and try again.",
    "Press Save again to sign in to Google Drive and save.",
    "Google Drive sign-in was not completed.",
    "Could not start Google sign-in.",
    "Google Drive is busy. Try saving again in a moment.",
    "Google Drive connection changed. Reopen the Drive file before saving back to it.",
    "Could not restore Google Drive. Check your connection and try again.",
    "Please connect Google Drive first.",
    "Google Drive disconnected.",
    "Could not disconnect Google Drive. Check your connection and try again.",
    "Could not list Google Drive files.",
    "Could not open the Google Drive file.",
    "Could not open Google Picker.",
    "A new copy was saved to Google Drive.",
    "Google Drive copies are limited to 5 MiB.",
    "Could not save the copy to Google Drive.",
    "Signing in to Google Drive to save…",
    "Saved to Google Drive.",
    "This file changed on Google Drive after you opened it. Save a new copy instead.",
    "Google Drive saves are limited to 5 MiB.",
    "Could not save to Google Drive.",
];

#[cfg(test)]
mod tests {
    /// Each app translates [`STATUS`] and [`MESSAGES`]: every sentence the browser code can show
    /// must be listed there, or it would appear in English in a Thai interface.
    #[test]
    fn every_sentence_the_browser_code_shows_is_listed() {
        let source = include_str!("web/controller.rs");
        // Errors the code only compares against; they are never shown.
        let internal = ["Google Drive request failed (HTTP 401)", "The document is too large for this upload"];
        let mut found = 0;
        for literal in source.split('"').skip(1).step_by(2) {
            let sentence = literal.chars().next().is_some_and(char::is_uppercase) && literal.contains(' ');
            if sentence && !literal.contains('{') && !internal.contains(&literal) {
                found += 1;
                assert!(super::STATUS.contains(&literal) || super::MESSAGES.contains(&literal), "not listed: {literal}");
            }
        }
        assert!(found > 20);
    }
}
