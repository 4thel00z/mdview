(() => {
  let bar = null;
  let input = null;
  let status = null;

  const replace = (html) => {
    const main = document.getElementById("mdview-content");
    if (!main) return;
    const y = window.scrollY;
    main.innerHTML = html;
    window.scrollTo(0, y);
  };

  const search = (backwards) => {
    if (!input || !input.value) return;
    const found = window.find(input.value, false, backwards, true, false, false, false);
    status.textContent = found ? "" : "no match";
  };

  const close = () => {
    if (!bar) return;
    bar.classList.remove("open");
    window.getSelection()?.removeAllRanges();
  };

  const ensure = () => {
    if (bar) return;
    bar = document.createElement("div");
    bar.id = "mdview-find";
    input = document.createElement("input");
    input.type = "search";
    input.placeholder = "Find";
    input.spellcheck = false;
    status = document.createElement("span");
    bar.append(input, status);
    document.body.append(bar);
    input.addEventListener("keydown", (event) => {
      if (event.key === "Escape") return close();
      if (event.key !== "Enter") return;
      event.preventDefault();
      search(event.shiftKey);
    });
    input.addEventListener("input", () => { status.textContent = ""; });
  };

  const openFind = () => {
    ensure();
    bar.classList.add("open");
    input.focus();
    input.select();
  };


  const palette = { root: null, input: null, list: null, items: [], selected: 0, files: [] };
  const BLOCKS = "#mdview-content p, #mdview-content li, #mdview-content td, #mdview-content th, #mdview-content pre, #mdview-content dd";
  const HEADINGS = "#mdview-content h1, #mdview-content h2, #mdview-content h3, #mdview-content h4, #mdview-content h5, #mdview-content h6";

  const fuzzy = (query, text) => {
    const haystack = text.toLowerCase();
    const index = haystack.indexOf(query);
    if (index >= 0) return 1000 - index - text.length / 100;
    let position = 0;
    let gaps = 0;
    for (const char of query) {
      const found = haystack.indexOf(char, position);
      if (found < 0) return null;
      gaps += found - position;
      position = found + 1;
    }
    return 500 - gaps - text.length / 100;
  };

  const ranked = (entries, query, limit) => {
    if (!query) return entries.slice(0, limit);
    return entries
      .map((entry) => ({ entry, score: fuzzy(query, entry.label + " " + (entry.detail || "")) }))
      .filter((match) => match.score !== null)
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
      .map((match) => match.entry);
  };

  const flash = (element) => {
    element.scrollIntoView({ block: "center" });
    element.classList.remove("mdview-flash");
    void element.offsetWidth;
    element.classList.add("mdview-flash");
  };

  const headingEntries = () =>
    [...document.querySelectorAll(HEADINGS)].map((element) => ({
      group: "Headings",
      label: element.textContent.replace("¶", "").trim(),
      detail: element.tagName.toLowerCase(),
      run: () => flash(element),
    }));

  const textEntries = (query) => {
    if (query.length < 2) return [];
    return [...document.querySelectorAll(BLOCKS)]
      .filter((element) => !element.querySelector("p, li, pre"))
      .map((element) => ({ element, text: element.textContent.replace(/↩/g, "").replace(/\s+/g, " ").trim() }))
      .filter(({ text }) => text.toLowerCase().includes(query))
      .slice(0, 8)
      .map(({ element, text }) => {
        const at = text.toLowerCase().indexOf(query);
        const start = Math.max(0, at - 32);
        const snippet = (start > 0 ? "…" : "") + text.slice(start, at + query.length + 64);
        return { group: "In this document", label: snippet, detail: element.tagName.toLowerCase(), highlight: query, run: () => flash(element) };
      });
  };

  const fileEntries = () =>
    palette.files.map((file) => ({
      group: "Files",
      label: file.name,
      detail: [file.dir, file.current ? "open" : ""].filter(Boolean).join(" · "),
      run: () => { if (!file.current) window.location.href = file.url; },
    }));

  const labelled = (label, highlight) => {
    const span = document.createElement("span");
    span.className = "mdview-palette-label";
    if (!highlight) {
      span.textContent = label;
      return span;
    }
    const at = label.toLowerCase().indexOf(highlight);
    if (at < 0) {
      span.textContent = label;
      return span;
    }
    const mark = document.createElement("mark");
    mark.textContent = label.slice(at, at + highlight.length);
    span.append(label.slice(0, at), mark, label.slice(at + highlight.length));
    return span;
  };

  const render = () => {
    const query = palette.input.value.trim().toLowerCase();
    palette.items = [
      ...ranked(headingEntries(), query, 8),
      ...textEntries(query),
      ...ranked(fileEntries(), query, 10),
    ];
    palette.selected = Math.min(palette.selected, Math.max(0, palette.items.length - 1));
    palette.list.replaceChildren();
    if (!palette.items.length) {
      const empty = document.createElement("div");
      empty.className = "mdview-palette-empty";
      empty.textContent = "Nothing matches.";
      palette.list.append(empty);
      return;
    }
    let group = null;
    palette.items.forEach((item, index) => {
      if (item.group !== group) {
        group = item.group;
        const heading = document.createElement("div");
        heading.className = "mdview-palette-group";
        heading.textContent = group;
        palette.list.append(heading);
      }
      const row = document.createElement("div");
      row.className = "mdview-palette-row" + (index === palette.selected ? " selected" : "");
      const detail = document.createElement("span");
      detail.className = "mdview-palette-detail";
      detail.textContent = item.detail || "";
      row.append(labelled(item.label, item.highlight), detail);
      row.addEventListener("mousemove", () => select(index));
      row.addEventListener("mousedown", (event) => { event.preventDefault(); choose(index); });
      palette.list.append(row);
    });
  };

  const select = (index) => {
    if (index === palette.selected) return;
    palette.selected = index;
    palette.list.querySelectorAll(".mdview-palette-row").forEach((row, position) => {
      row.classList.toggle("selected", position === index);
    });
    palette.list.querySelector(".mdview-palette-row.selected")?.scrollIntoView({ block: "nearest" });
  };

  const closePalette = () => {
    if (!palette.root) return;
    palette.root.classList.remove("open");
  };

  const choose = (index) => {
    const item = palette.items[index];
    closePalette();
    if (item) item.run();
  };

  const buildPalette = () => {
    if (palette.root) return;
    palette.root = document.createElement("div");
    palette.root.id = "mdview-palette";
    const panel = document.createElement("div");
    panel.className = "mdview-palette-panel";
    palette.input = document.createElement("input");
    palette.input.type = "text";
    palette.input.placeholder = "Search headings, text and files";
    palette.input.spellcheck = false;
    palette.list = document.createElement("div");
    palette.list.className = "mdview-palette-list";
    const hint = document.createElement("div");
    hint.className = "mdview-palette-hint";
    hint.textContent = "↑↓ move   ↵ open   esc close";
    panel.append(palette.input, palette.list, hint);
    palette.root.append(panel);
    document.body.append(palette.root);
    palette.root.addEventListener("mousedown", (event) => {
      if (event.target === palette.root) closePalette();
    });
    palette.input.addEventListener("input", () => { palette.selected = 0; render(); });
    palette.input.addEventListener("keydown", (event) => {
      const down = event.key === "ArrowDown" || (event.ctrlKey && event.key === "n");
      const up = event.key === "ArrowUp" || (event.ctrlKey && event.key === "p");
      if (event.key === "Escape") return closePalette();
      if (event.key === "Enter") { event.preventDefault(); return choose(palette.selected); }
      if (!down && !up) return;
      event.preventDefault();
      const count = palette.items.length;
      if (!count) return;
      select((palette.selected + (down ? 1 : count - 1)) % count);
    });
  };

  const togglePalette = (files) => {
    buildPalette();
    if (palette.root.classList.contains("open")) return closePalette();
    palette.files = files || [];
    palette.selected = 0;
    palette.input.value = "";
    palette.root.classList.add("open");
    render();
    palette.input.focus();
  };

  document.addEventListener("keydown", (event) => {
    if (!event.metaKey || event.altKey || event.ctrlKey || event.shiftKey) return;
    if (event.target instanceof HTMLInputElement) return;
    if (event.key === "ArrowLeft") { event.preventDefault(); history.back(); }
    if (event.key === "ArrowRight") { event.preventDefault(); history.forward(); }
  });

  window.mdview = {
    togglePalette,
    replace,
    openFind,
    findNext: () => { ensure(); search(false); },
    findPrevious: () => { ensure(); search(true); },
  };
})();
