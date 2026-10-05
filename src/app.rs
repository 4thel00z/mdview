use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use muda::MenuEvent;
use notify::RecommendedWatcher;
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use tao::dpi::LogicalSize;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Theme, Window, WindowBuilder, WindowId};
use wry::{NewWindowResponse, PageLoadEvent, WebView, WebViewBuilder};

use crate::appearance::Appearance;
use crate::macos;
use crate::menu::{self, Action, AppMenu};
use crate::protocol::{self, SCHEME};
use crate::render::renderer;

const SCRIPT: &str = include_str!("assets/app.js");
const ZOOM_LEVELS: [f64; 11] = [0.5, 0.67, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5];
const DEFAULT_ZOOM: usize = 4;
const LAUNCH_GRACE: Duration = Duration::from_millis(500);
const WATCH_DEBOUNCE: Duration = Duration::from_millis(80);

#[derive(Debug)]
pub enum UserEvent {
    Menu(Action),
    PageLoaded(WindowId, String),
    FileChanged(WindowId),
}

struct Document {
    window: Window,
    webview: WebView,
    path: PathBuf,
    zoom: usize,
    watcher: Option<Debouncer<RecommendedWatcher>>,
}

struct App {
    documents: HashMap<WindowId, Document>,
    focused: Option<WindowId>,
    proxy: EventLoopProxy<UserEvent>,
    prompted: bool,
    pending: Vec<PathBuf>,
    appearance: Appearance,
    menu: AppMenu,
}

pub fn run(paths: Vec<PathBuf>) -> Result<()> {
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let menu_proxy = proxy.clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let Some(action) = menu::action_for(event.id().as_ref()) else {
            return;
        };
        menu_proxy.send_event(UserEvent::Menu(action)).ok();
    }));
    let appearance = Appearance::load();
    let mut app = App {
        documents: HashMap::new(),
        focused: None,
        proxy,
        prompted: false,
        pending: paths,
        appearance,
        menu: menu::build(appearance)?,
    };
    event_loop.run(move |event, target, control_flow| app.handle(event, target, control_flow))
}

impl App {
    fn handle(
        &mut self,
        event: Event<'_, UserEvent>,
        target: &EventLoopWindowTarget<UserEvent>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + LAUNCH_GRACE);
                let pending = std::mem::take(&mut self.pending);
                self.open_paths(target, &pending);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                *control_flow = ControlFlow::Wait;
                if !self.documents.is_empty() || self.prompted {
                    return;
                }
                self.prompted = true;
                self.prompt(target);
            }
            Event::Opened { urls } => {
                self.prompted = true;
                urls.iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .for_each(|path| self.open(target, &path));
            }
            Event::Reopen { has_visible_windows: false, .. } => self.prompt(target),
            Event::WindowEvent { window_id, event, .. } => self.window_event(window_id, event),
            Event::UserEvent(event) => self.user_event(target, event),
            _ => {}
        }
    }

    fn window_event(&mut self, id: WindowId, event: WindowEvent<'_>) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                self.documents.remove(&id);
            }
            WindowEvent::Focused(true) => self.focused = Some(id),
            _ => {}
        }
    }

    fn user_event(&mut self, target: &EventLoopWindowTarget<UserEvent>, event: UserEvent) {
        match event {
            UserEvent::Menu(action) => self.perform(target, action),
            UserEvent::PageLoaded(id, url) => self.loaded(id, &url),
            UserEvent::FileChanged(id) => self.refresh(id),
        }
    }

    fn open_paths(&mut self, target: &EventLoopWindowTarget<UserEvent>, paths: &[PathBuf]) {
        paths.iter().for_each(|path| self.open(target, path));
    }

    fn prompt(&mut self, target: &EventLoopWindowTarget<UserEvent>) {
        let picked = rfd::FileDialog::new()
            .set_title("Open Markdown")
            .add_filter("Markdown", &["md", "markdown", "mdown", "mkd", "mkdn", "mdwn", "mdtxt", "mdtext", "rmd"])
            .add_filter("All Files", &["*"])
            .pick_files()
            .unwrap_or_default();
        self.open_paths(target, &picked);
    }

    fn open(&mut self, target: &EventLoopWindowTarget<UserEvent>, path: &Path) {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if let Some(existing) = self.documents.values().find(|doc| doc.path == path) {
            existing.window.set_visible(true);
            existing.window.set_focus();
            return;
        }
        match self.create(target, &path) {
            Ok(document) => {
                macos::note_recent(&path);
                self.focused = Some(document.window.id());
                self.documents.insert(document.window.id(), document);
            }
            Err(error) => eprintln!("mdview: cannot open {}: {error:#}", path.display()),
        }
    }

    fn create(&self, target: &EventLoopWindowTarget<UserEvent>, path: &Path) -> Result<Document> {
        let window = WindowBuilder::new()
            .with_title(title_for(path))
            .with_inner_size(LogicalSize::new(880.0, 1000.0))
            .with_min_inner_size(LogicalSize::new(360.0, 240.0))
            .with_visible(false)
            .with_theme(self.appearance.theme())
            .build(target)?;
        let id = window.id();
        let load_proxy = self.proxy.clone();
        let webview = WebViewBuilder::new()
            .with_custom_protocol(SCHEME.to_owned(), |_, request| protocol::respond(&request))
            .with_initialization_script(SCRIPT)
            .with_navigation_handler(navigate)
            .with_new_window_req_handler(|url, _| {
                navigate(url);
                NewWindowResponse::Deny
            })
            .with_on_page_load_handler(move |event, url| {
                if !matches!(event, PageLoadEvent::Finished) {
                    return;
                }
                load_proxy.send_event(UserEvent::PageLoaded(id, url)).ok();
            })
            .with_url(protocol::url_for(path))
            .build(&window)?;
        macos::set_represented_file(&window, path);
        Ok(Document {
            watcher: self.watch(id, path),
            window,
            webview,
            path: path.to_path_buf(),
            zoom: DEFAULT_ZOOM,
        })
    }

    fn watch(&self, id: WindowId, path: &Path) -> Option<Debouncer<RecommendedWatcher>> {
        let directory = path.parent()?.to_path_buf();
        let name = path.file_name()?.to_os_string();
        let proxy = self.proxy.clone();
        let mut debouncer = new_debouncer(WATCH_DEBOUNCE, move |result: DebounceEventResult| {
            let Ok(events) = result else {
                return;
            };
            if !events.iter().any(|event| event.path.file_name() == Some(name.as_os_str())) {
                return;
            }
            proxy.send_event(UserEvent::FileChanged(id)).ok();
        })
        .ok()?;
        debouncer.watcher().watch(&directory, notify::RecursiveMode::NonRecursive).ok()?;
        Some(debouncer)
    }

    fn loaded(&mut self, id: WindowId, url: &str) {
        let Some(path) = protocol::path_from_url(url) else {
            return;
        };
        let changed = self.documents.get(&id).is_some_and(|doc| doc.path != path);
        let watcher = if changed { self.watch(id, &path) } else { None };
        let Some(document) = self.documents.get_mut(&id) else {
            return;
        };
        document.window.set_visible(true);
        if !changed {
            return;
        }
        document.window.set_title(&title_for(&path));
        macos::set_represented_file(&document.window, &path);
        document.watcher = watcher;
        document.path = path;
    }

    fn refresh(&self, id: WindowId) {
        let Some(document) = self.documents.get(&id) else {
            return;
        };
        let Ok(markdown) = std::fs::read_to_string(&document.path) else {
            return;
        };
        let body = renderer().body(&markdown);
        let Ok(literal) = serde_json::to_string(&body) else {
            return;
        };
        document.webview.evaluate_script(&format!("window.mdview.replace({literal})")).ok();
    }

    fn set_appearance(&mut self, appearance: Appearance) {
        self.appearance = appearance;
        appearance.save();
        self.menu.show_appearance(appearance);
        self.documents.values().for_each(|document| document.window.set_theme(appearance.theme()));
    }

    fn current_theme(&self) -> Theme {
        self.focused
            .and_then(|id| self.documents.get(&id))
            .or_else(|| self.documents.values().next())
            .map_or(Theme::Light, |document| document.window.theme())
    }

    fn perform(&mut self, target: &EventLoopWindowTarget<UserEvent>, action: Action) {
        match action {
            Action::Open => return self.prompt(target),
            Action::SetAppearance(appearance) => return self.set_appearance(appearance),
            Action::ToggleAppearance => return self.set_appearance(Appearance::opposite_of(self.current_theme())),
            _ => {}
        }
        let Some(document) = self.focused.and_then(|id| self.documents.get_mut(&id)) else {
            return;
        };
        let script = match action {
            Action::Find => "window.mdview.openFind()",
            Action::FindNext => "window.mdview.findNext()",
            Action::FindPrevious => "window.mdview.findPrevious()",
            Action::Back => "history.back()",
            Action::Forward => "history.forward()",
            _ => "",
        };
        if !script.is_empty() {
            document.webview.evaluate_script(script).ok();
            return;
        }
        match action {
            Action::Reveal => reveal(&document.path),
            Action::Print => {
                document.webview.print().ok();
            }
            Action::Reload => {
                document.webview.reload().ok();
            }
            Action::ZoomIn => document.set_zoom(document.zoom.saturating_add(1)),
            Action::ZoomOut => document.set_zoom(document.zoom.saturating_sub(1)),
            Action::ZoomReset => document.set_zoom(DEFAULT_ZOOM),
            _ => {}
        }
    }
}

impl Document {
    fn set_zoom(&mut self, level: usize) {
        self.zoom = level.min(ZOOM_LEVELS.len() - 1);
        self.webview.zoom(ZOOM_LEVELS[self.zoom]).ok();
    }
}

fn navigate(url: String) -> bool {
    let Some(path) = protocol::path_from_url(&url) else {
        if url.starts_with("about:") {
            return true;
        }
        open::that_detached(&url).ok();
        return false;
    };
    if protocol::is_markdown(&path) {
        return true;
    }
    open::that_detached(&path).ok();
    false
}

fn reveal(path: &Path) {
    std::process::Command::new("open").arg("-R").arg(path).spawn().ok();
}

fn title_for(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "mdview".to_owned())
}
