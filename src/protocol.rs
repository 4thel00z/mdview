use std::borrow::Cow;
use std::path::{Path, PathBuf};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use wry::http::{Request, Response, StatusCode, header::CONTENT_TYPE};

use crate::render::renderer;

pub const SCHEME: &str = "mdview";
const ASSET_PREFIX: &str = "/.mdview/";
const ASSETS: [(&str, &[u8]); 3] = [
    ("fonts/space-grotesk.woff2", include_bytes!("assets/fonts/space-grotesk.woff2")),
    ("fonts/space-mono-400.woff2", include_bytes!("assets/fonts/space-mono-400.woff2")),
    ("fonts/space-mono-700.woff2", include_bytes!("assets/fonts/space-mono-700.woff2")),
];
const ORIGIN: &str = "mdview://localhost";
const MARKDOWN_EXTENSIONS: [&str; 9] =
    ["md", "markdown", "mdown", "mkd", "mkdn", "mdwn", "mdtxt", "mdtext", "rmd"];
const PATH_ESCAPES: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

pub fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| MARKDOWN_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

pub fn url_for(path: &Path) -> String {
    format!("{ORIGIN}{}", utf8_percent_encode(&path.to_string_lossy(), PATH_ESCAPES))
}

pub fn path_from_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix(ORIGIN)?;
    let path = rest.split(['?', '#']).next().unwrap_or_default();
    decode_path(path)
}

fn decode_path(path: &str) -> Option<PathBuf> {
    if path.is_empty() {
        return None;
    }
    let decoded = percent_decode_str(path).decode_utf8().ok()?;
    Some(PathBuf::from(decoded.into_owned()))
}

pub fn respond(request: &Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    if let Some(name) = request.uri().path().strip_prefix(ASSET_PREFIX) {
        return asset(name);
    }
    let Some(path) = decode_path(request.uri().path()) else {
        return status(StatusCode::BAD_REQUEST);
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return status(StatusCode::NOT_FOUND);
    };
    if !is_markdown(&path) {
        return content(mime_for(&bytes), bytes);
    }
    let title = path.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
    let page = renderer().page(&String::from_utf8_lossy(&bytes), &title);
    content("text/html; charset=utf-8", page.into_bytes())
}

pub fn mime_for(bytes: &[u8]) -> &'static str {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_ascii_lowercase();
    if head.contains("<svg") {
        return "image/svg+xml";
    }
    infer::get(bytes).map_or("application/octet-stream", |kind| kind.mime_type())
}

fn asset(name: &str) -> Response<Cow<'static, [u8]>> {
    let Some((_, bytes)) = ASSETS.iter().find(|(asset_name, _)| *asset_name == name) else {
        return status(StatusCode::NOT_FOUND);
    };
    Response::builder()
        .header(CONTENT_TYPE, "font/woff2")
        .header("Cache-Control", "max-age=31536000, immutable")
        .body(Cow::Borrowed(*bytes))
        .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR))
}

fn content(mime: &str, body: Vec<u8>) -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .header(CONTENT_TYPE, mime)
        .body(Cow::Owned(body))
        .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR))
}

fn status(code: StatusCode) -> Response<Cow<'static, [u8]>> {
    let mut response = Response::new(Cow::Borrowed(&[][..]));
    *response.status_mut() = code;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_round_trips_paths_with_spaces_and_unicode() {
        let path = Path::new("/Users/me/My Notes/naïve #1.md");
        let url = url_for(path);
        assert!(url.starts_with("mdview://localhost/Users/me/My%20Notes/"), "{url}");
        assert_eq!(path_from_url(&url).as_deref(), Some(path));
    }

    #[test]
    fn path_from_url_drops_fragment_and_query() {
        let path = path_from_url("mdview://localhost/tmp/a.md#section?x=1");
        assert_eq!(path.as_deref(), Some(Path::new("/tmp/a.md")));
    }

    #[test]
    fn path_from_url_rejects_foreign_urls() {
        assert_eq!(path_from_url("https://example.com/a.md"), None);
    }

    #[test]
    fn markdown_detection_is_case_insensitive() {
        assert!(is_markdown(Path::new("README.MD")));
        assert!(is_markdown(Path::new("notes.markdown")));
        assert!(!is_markdown(Path::new("image.png")));
    }

    #[test]
    fn mime_comes_from_bytes() {
        assert_eq!(mime_for(b"\x89PNG\r\n\x1a\n0000"), "image/png");
        assert_eq!(mime_for(b"<?xml version=\"1.0\"?><svg xmlns=\"\"></svg>"), "image/svg+xml");
        assert_eq!(mime_for(b""), "application/octet-stream");
    }

    #[test]
    fn serves_rendered_markdown_and_raw_assets() {
        let dir = std::env::temp_dir().join(format!("mdview-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let doc = dir.join("doc.md");
        std::fs::write(&doc, "# Title\n").ok();
        let request = |path: &Path| {
            Request::builder().uri(url_for(path)).body(Vec::new()).unwrap_or_default()
        };
        let page = respond(&request(&doc));
        assert_eq!(page.status(), StatusCode::OK, "{}", url_for(&doc));
        assert!(String::from_utf8_lossy(page.body()).contains(">Title<"));
        let font = respond(&request(Path::new("/.mdview/fonts/space-mono-400.woff2")));
        assert_eq!(font.status(), StatusCode::OK);
        assert_eq!(font.headers()[CONTENT_TYPE], "font/woff2");
        let missing = respond(&request(&dir.join("missing.png")));
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
        std::fs::remove_dir_all(&dir).ok();
    }
}
