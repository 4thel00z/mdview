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

  window.mdview = {
    replace,
    openFind,
    findNext: () => { ensure(); search(false); },
    findPrevious: () => { ensure(); search(true); },
  };
})();
