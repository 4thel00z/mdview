use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

use ammonia::Builder;
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html::write_opening_tag;
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Options, markdown_to_html_with_plugins};
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, css_for_theme_with_class_style};

const CLASS_PREFIX: &str = "hl-";
const LIGHT_THEME: &str = "InspiredGitHub";
const DARK_THEME: &str = "base16-eighties.dark";
const STYLE: &str = include_str!("assets/style.css");

pub struct Renderer {
    highlighter: Highlighter,
    sanitizer: Builder<'static>,
    syntax_css: String,
}

pub fn renderer() -> &'static Renderer {
    static RENDERER: OnceLock<Renderer> = OnceLock::new();
    RENDERER.get_or_init(Renderer::new)
}

impl Renderer {
    fn new() -> Self {
        let themes = ThemeSet::load_defaults();
        let syntax_css = syntax_css(&themes);
        let highlighter = Highlighter(
            SyntectAdapterBuilder::new()
                .css_with_class_prefix(CLASS_PREFIX)
                .theme_set(themes)
                .build(),
        );
        Self { highlighter, sanitizer: sanitizer(), syntax_css }
    }

    pub fn body(&self, markdown: &str) -> String {
        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&self.highlighter);
        let html = markdown_to_html_with_plugins(markdown, &options(), &plugins);
        self.sanitizer.clean(&html).to_string()
    }

    pub fn page(&self, markdown: &str, title: &str) -> String {
        format!(
            concat!(
                "<!doctype html><html><head><meta charset=\"utf-8\">",
                "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">",
                "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; ",
                "img-src 'self' mdview: https: http: data:; media-src 'self' mdview: https: http:; ",
                "style-src 'unsafe-inline'; font-src mdview:; script-src 'none'\">",
                "<title>{title}</title><style>{style}\n{syntax}</style></head>",
                "<body><main id=\"mdview-content\" class=\"markdown\">{body}</main></body></html>"
            ),
            title = escape_text(title),
            style = STYLE,
            syntax = self.syntax_css,
            body = self.body(markdown),
        )
    }
}

struct Highlighter(SyntectAdapter);

impl SyntaxHighlighterAdapter for Highlighter {
    fn write_highlighted(&self, output: &mut dyn fmt::Write, lang: Option<&str>, code: &str) -> fmt::Result {
        self.0.write_highlighted(output, lang, code)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        mut attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        attributes.insert("class", Cow::Borrowed("syntax-highlighting"));
        let mut sorted: Vec<_> = attributes.into_iter().collect();
        sorted.sort_by_key(|(name, _)| *name);
        write_opening_tag(output, "pre", sorted)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        let mut sorted: Vec<_> = attributes.into_iter().collect();
        sorted.sort_by_key(|(name, _)| *name);
        write_opening_tag(output, "code", sorted)
    }
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;
    options.extension.description_lists = true;
    options.extension.header_id_prefix = Some(String::new());
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.parse.smart = false;
    options.render.r#unsafe = true;
    options.render.github_pre_lang = true;
    options.render.tasklist_classes = true;
    options
}

fn sanitizer() -> Builder<'static> {
    let mut builder = Builder::default();
    builder
        .add_tags(["input", "picture", "source", "video", "audio", "kbd", "mark", "figure", "figcaption", "section"])
        .add_generic_attributes(["class", "id", "align", "title", "dir", "lang"])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .add_tag_attributes("img", ["src", "alt", "width", "height", "loading"])
        .add_tag_attributes("source", ["src", "srcset", "media", "type"])
        .add_tag_attributes("video", ["src", "controls", "width", "height", "poster", "muted", "loop"])
        .add_tag_attributes("audio", ["src", "controls"])
        .add_tag_attributes("a", ["href", "name", "aria-hidden", "data-footnote-ref", "data-footnote-backref"])
        .add_tag_attributes("td", ["colspan", "rowspan"])
        .add_tag_attributes("th", ["colspan", "rowspan"])
        .add_tag_attributes("ol", ["start"])
        .add_tag_attributes("section", ["data-footnotes"])
        .link_rel(Some("noopener noreferrer"))
        .id_prefix(None::<&str>)
        .clean_content_tags(HashSet::from(["script", "style"]));
    builder
}

fn syntax_css(themes: &ThemeSet) -> String {
    let style = ClassStyle::SpacedPrefixed { prefix: CLASS_PREFIX };
    let css = |name: &str| {
        themes
            .themes
            .get(name)
            .and_then(|theme| css_for_theme_with_class_style(theme, style).ok())
            .unwrap_or_default()
    };
    format!(
        "{light}\n@media (prefers-color-scheme: dark) {{\n{dark}\n}}",
        light = css(LIGHT_THEME),
        dark = css(DARK_THEME)
    )
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(markdown: &str) -> String {
        renderer().body(markdown)
    }

    #[test]
    fn renders_gfm_tables() {
        let html = body("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<table>"), "{html}");
        assert!(html.contains("<td>1</td>"), "{html}");
    }

    #[test]
    fn renders_task_lists_as_disabled_checkboxes() {
        let html = body("- [x] done\n- [ ] todo\n");
        assert!(html.contains("type=\"checkbox\""), "{html}");
        assert!(html.contains("checked"), "{html}");
        assert!(html.contains("disabled"), "{html}");
    }

    #[test]
    fn highlights_fenced_code_with_prefixed_classes() {
        let html = body("```rust\nfn main() { let x = 1; }\n```\n");
        assert!(html.contains("class=\"hl-"), "{html}");
        assert!(html.contains("lang=\"rust\""), "{html}");
    }

    #[test]
    fn strips_scripts_and_event_handlers() {
        let html = body("<script>alert(1)</script>\n\n<img src=\"a.png\" onerror=\"alert(2)\">\n");
        assert!(!html.contains("script"), "{html}");
        assert!(!html.contains("onerror"), "{html}");
        assert!(html.contains("src=\"a.png\""), "{html}");
    }

    #[test]
    fn strips_javascript_urls() {
        let html = body("[x](javascript:alert(1))\n");
        assert!(!html.contains("javascript:"), "{html}");
    }

    #[test]
    fn keeps_readme_style_raw_html() {
        let html = body("<p align=\"center\"><img src=\"logo.png\" width=\"120\"></p>\n\n<details><summary>More</summary>hidden</details>\n");
        assert!(html.contains("align=\"center\""), "{html}");
        assert!(html.contains("width=\"120\""), "{html}");
        assert!(html.contains("<details>"), "{html}");
    }

    #[test]
    fn keeps_heading_anchor_ids() {
        let html = body("# Hello World\n");
        assert!(html.contains("id=\"hello-world\""), "{html}");
    }

    #[test]
    fn renders_github_alerts() {
        let html = body("> [!NOTE]\n> Useful.\n");
        assert!(html.contains("markdown-alert"), "{html}");
    }

    #[test]
    fn hides_front_matter() {
        let html = body("---\ntitle: x\n---\n# Body\n");
        assert!(!html.contains("title: x"), "{html}");
        assert!(html.contains("Body"), "{html}");
    }

    #[test]
    fn renders_footnotes() {
        let html = body("Text[^1].\n\n[^1]: Note.\n");
        assert!(html.contains("data-footnotes"), "{html}");
    }

    #[test]
    fn page_includes_both_syntax_themes_and_escapes_title() {
        let html = renderer().page("# x", "<a>.md");
        assert!(html.contains("prefers-color-scheme: dark"));
        assert!(html.contains(".hl-"));
        assert!(html.contains("<title>&lt;a&gt;.md</title>"));
    }
}
