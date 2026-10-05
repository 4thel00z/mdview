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
> Press <kbd>⌘</kbd> <kbd>⇧</kbd> <kbd>L</kbd> to switch between light and dark.

> Simplicity is prerequisite for reliability.

Footnotes render at the bottom.[^1]

[^1]: Like this one.
