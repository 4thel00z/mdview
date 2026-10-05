use std::path::Path;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSDocumentController, NSWindow};
use objc2_foundation::{NSString, NSURL};
use tao::platform::macos::WindowExtMacOS;
use tao::window::Window;

pub fn set_represented_file(window: &Window, path: &Path) {
    let pointer = window.ns_window() as *const NSWindow;
    let Some(ns_window) = (unsafe { pointer.as_ref() }) else {
        return;
    };
    ns_window.setRepresentedFilename(&NSString::from_str(&path.to_string_lossy()));
}

pub fn note_recent(path: &Path) {
    let Some(marker) = MainThreadMarker::new() else {
        return;
    };
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    NSDocumentController::sharedDocumentController(marker).noteNewRecentDocumentURL(&url);
}
