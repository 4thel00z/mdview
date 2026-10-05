use muda::accelerator::Accelerator;
use muda::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Open,
    Reveal,
    Print,
    Find,
    FindNext,
    FindPrevious,
    Reload,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Back,
    Forward,
}

const ITEMS: [(Action, &str, &str, &str); 12] = [
    (Action::Open, "open", "Open…", "CmdOrCtrl+O"),
    (Action::Reveal, "reveal", "Reveal in Finder", "CmdOrCtrl+Shift+R"),
    (Action::Print, "print", "Print…", "CmdOrCtrl+P"),
    (Action::Find, "find", "Find…", "CmdOrCtrl+F"),
    (Action::FindNext, "find-next", "Find Next", "CmdOrCtrl+G"),
    (Action::FindPrevious, "find-previous", "Find Previous", "CmdOrCtrl+Shift+G"),
    (Action::Reload, "reload", "Reload", "CmdOrCtrl+R"),
    (Action::ZoomIn, "zoom-in", "Zoom In", "CmdOrCtrl+Equal"),
    (Action::ZoomOut, "zoom-out", "Zoom Out", "CmdOrCtrl+Minus"),
    (Action::ZoomReset, "zoom-reset", "Actual Size", "CmdOrCtrl+Digit0"),
    (Action::Back, "back", "Back", "CmdOrCtrl+BracketLeft"),
    (Action::Forward, "forward", "Forward", "CmdOrCtrl+BracketRight"),
];

pub fn action_for(id: &str) -> Option<Action> {
    ITEMS.iter().find(|(_, item_id, _, _)| *item_id == id).map(|(action, ..)| *action)
}

fn item(action: Action) -> MenuItem {
    let (_, id, label, shortcut) = ITEMS
        .iter()
        .find(|(candidate, ..)| *candidate == action)
        .copied()
        .unwrap_or((action, "", "", ""));
    MenuItem::with_id(id, label, true, shortcut.parse::<Accelerator>().ok())
}

pub fn build() -> muda::Result<Menu> {
    let about = AboutMetadata {
        name: Some("mdview".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        comments: Some(env!("CARGO_PKG_DESCRIPTION").into()),
        ..Default::default()
    };
    let app = Submenu::with_items(
        "mdview",
        true,
        &[
            &PredefinedMenuItem::about(None, Some(about)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ],
    )?;
    let file = Submenu::with_items(
        "File",
        true,
        &[
            &item(Action::Open),
            &item(Action::Reveal),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::close_window(None),
            &PredefinedMenuItem::separator(),
            &item(Action::Print),
        ],
    )?;
    let edit = Submenu::with_items(
        "Edit",
        true,
        &[
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::select_all(None),
            &PredefinedMenuItem::separator(),
            &item(Action::Find),
            &item(Action::FindNext),
            &item(Action::FindPrevious),
        ],
    )?;
    let view = Submenu::with_items(
        "View",
        true,
        &[
            &item(Action::Reload),
            &PredefinedMenuItem::separator(),
            &item(Action::ZoomReset),
            &item(Action::ZoomIn),
            &item(Action::ZoomOut),
            &PredefinedMenuItem::separator(),
            &item(Action::Back),
            &item(Action::Forward),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::fullscreen(None),
        ],
    )?;
    let window = Submenu::with_items(
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::bring_all_to_front(None),
        ],
    )?;
    let menu = Menu::with_items(&[&app, &file, &edit, &view, &window])?;
    menu.init_for_nsapp();
    window.set_as_windows_menu_for_nsapp();
    Ok(menu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_a_parseable_shortcut_and_round_trips() {
        for (action, id, _, shortcut) in ITEMS {
            assert_eq!(action_for(id), Some(action));
            assert!(shortcut.parse::<Accelerator>().is_ok(), "{shortcut}");
        }
    }
}
