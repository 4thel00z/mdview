# Field notes

A quick tour of what **mdview** renders. Edit this file while it is open and the window updates on save.

## Highlighted code

```rust
fn main() {
    let html = renderer().page(&markdown, "notes.md");
    println!("{} bytes", html.len());
}
```

## Tables and tasks

| Format | Extensions | Default |
|---|---|---|
| Markdown | `.md`, `.markdown` | yes |
| Variants | `.mdown`, `.mkd` | Open With |

- [x] GitHub Flavored Markdown
- [x] Light and dark themes
- [ ] Math blocks

> [!TIP]
> Press <kbd>⌘</kbd> <kbd>F</kbd> to search the page, and <kbd>⌘</kbd> <kbd>=</kbd> to zoom.

Footnotes render at the bottom.[^1]

[^1]: Like this one.
