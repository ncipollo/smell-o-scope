  // ---- search: index ----

  const SEARCH_LIMIT = 50;

  function buildPathIndex(document_) {
    const entries = [];
    const parentOf = new WeakMap();
    walk(document_.roots, null);
    return { entries: entries, parentOf: parentOf };

    function walk(nodes, parent) {
      for (const node of nodes) {
        if (parent !== null) {
          parentOf.set(node, parent);
        }
        entries.push({ node: node, haystack: normalizePath(node.path) });
        if (node.kind === "directory") {
          walk(node.children, node);
        }
      }
    }
  }

  function normalizePath(path) {
    return path.toLowerCase().replace(/\\/g, "/");
  }

  function ancestorsOf(node) {
    const chain = [];
    let parent = pathIndex.parentOf.get(node);
    while (parent) {
      chain.unshift(parent);
      parent = pathIndex.parentOf.get(parent);
    }
    return chain;
  }

  // ---- search: matching ----

  function matchEntries(query) {
    const needle = normalizePath(query);
    const matches = [];
    for (const entry of pathIndex.entries) {
      if (entry.haystack.includes(needle)) {
        matches.push({ node: entry.node, nameMatch: normalizePath(entry.node.name).includes(needle) });
      }
    }
    matches.sort(compareMatches);
    return matches;
  }

  function compareMatches(a, b) {
    if (a.nameMatch !== b.nameMatch) {
      return a.nameMatch ? -1 : 1;
    }
    return b.node.violations.total - a.node.violations.total || (a.node.path < b.node.path ? -1 : 1);
  }

  // ---- search: UI ----

  function renderSearch() {
    const wrap = document.createElement("div");
    wrap.className = "search";
    wrap.setAttribute("role", "search");
    wrap.appendChild(searchField());

    const status = document.createElement("p");
    status.className = "search__status";
    status.setAttribute("role", "status");
    search.status = status;
    wrap.appendChild(status);

    const results = document.createElement("ul");
    results.className = "search__results";
    results.id = "smell-search-results";
    results.setAttribute("role", "listbox");
    results.setAttribute("aria-label", "search results");
    results.hidden = true;
    search.results = results;
    wrap.appendChild(results);

    return wrap;
  }

  function searchField() {
    const field = document.createElement("div");
    field.className = "search__field";

    const label = document.createElement("label");
    label.className = "search__label";
    label.htmlFor = "smell-search";
    label.textContent = "Search files and folders";
    field.appendChild(label);

    const input = document.createElement("input");
    input.className = "search__input";
    input.id = "smell-search";
    input.type = "search";
    input.placeholder = "Search files and folders";
    input.autocomplete = "off";
    input.spellcheck = false;
    input.setAttribute("role", "combobox");
    input.setAttribute("aria-expanded", "false");
    input.setAttribute("aria-autocomplete", "list");
    input.setAttribute("aria-controls", "smell-search-results");
    input.addEventListener("input", onSearchInput);
    input.addEventListener("keydown", onSearchKeydown);
    search.input = input;
    field.appendChild(input);

    const clear = document.createElement("button");
    clear.type = "button";
    clear.className = "search__clear";
    clear.setAttribute("aria-label", "clear search");
    clear.textContent = "✕";
    clear.hidden = true;
    clear.addEventListener("click", clearSearch);
    search.clear = clear;
    field.appendChild(clear);

    return field;
  }

  function onSearchInput() {
    runSearch(search.input.value);
  }

  function runSearch(query) {
    const trimmed = query.trim();
    search.clear.hidden = trimmed.length === 0;
    if (trimmed.length === 0) {
      search.matches = [];
      renderMatches(search.matches);
      showResults(false);
      setStatus(0, 0);
      clearTarget();
      return;
    }
    const matches = matchEntries(trimmed);
    search.matches = matches.slice(0, SEARCH_LIMIT);
    renderMatches(search.matches);
    showResults(true);
    setStatus(search.matches.length, matches.length);
    setActive(search.matches.length > 0 ? 0 : -1);
  }

  function showResults(visible) {
    search.results.hidden = !visible;
    search.input.setAttribute("aria-expanded", String(visible));
  }

  function setStatus(shown, total) {
    if (total === 0) {
      search.status.textContent = "no matches";
    } else if (shown < total) {
      search.status.textContent = "showing " + shown + " of " + total + " matches — refine your search";
    } else {
      search.status.textContent = shown + (shown === 1 ? " match" : " matches");
    }
  }

  function renderMatches(matches) {
    const rows = [];
    for (let i = 0; i < matches.length; i++) {
      rows.push(renderMatch(matches[i].node, i));
    }
    search.results.replaceChildren(...rows);
  }

  function renderMatch(node, index) {
    const item = document.createElement("li");
    item.className = "search__result";
    item.id = "smell-search-result-" + index;
    item.setAttribute("role", "option");
    item.setAttribute("aria-selected", "false");
    item.title = node.path;
    item.addEventListener("click", () => jumpTo(node));

    const icon = document.createElement("span");
    icon.className = "search__result-icon";
    icon.setAttribute("aria-hidden", "true");
    icon.textContent = node.kind === "directory" ? "📁" : "📄";
    item.appendChild(icon);

    const path = document.createElement("span");
    path.className = "search__result-path";
    const name = document.createElement("span");
    name.className = "search__result-name";
    name.textContent = node.name;
    path.appendChild(name);
    const dirLength = node.path.length - node.name.length;
    if (dirLength > 0) {
      const dir = document.createElement("span");
      dir.className = "search__result-dir";
      dir.textContent = node.path.slice(0, dirLength - 1);
      path.appendChild(dir);
    }
    item.appendChild(path);

    item.appendChild(renderBadges(node.violations, doc.measures));
    return item;
  }

  // ---- search: keyboard nav ----

  function onSearchKeydown(event) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveActive(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      moveActive(-1);
    } else if (event.key === "Home" && search.matches.length > 0) {
      event.preventDefault();
      setActive(0);
    } else if (event.key === "End" && search.matches.length > 0) {
      event.preventDefault();
      setActive(search.matches.length - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      commitActive();
    } else if (event.key === "Escape") {
      clearSearch();
    }
  }

  function moveActive(delta) {
    if (search.matches.length > 0) {
      setActive(search.active + delta);
    }
  }

  function setActive(next) {
    const options = search.results.children;
    if (options.length === 0) {
      search.active = -1;
      search.input.removeAttribute("aria-activedescendant");
      return;
    }
    const index = ((next % options.length) + options.length) % options.length;
    for (let i = 0; i < options.length; i++) {
      const on = i === index;
      options[i].classList.toggle("search__result--active", on);
      options[i].setAttribute("aria-selected", String(on));
    }
    search.active = index;
    search.input.setAttribute("aria-activedescendant", options[index].id);
    options[index].scrollIntoView({ block: "nearest" });
  }

  function commitActive() {
    if (search.active >= 0 && search.active < search.matches.length) {
      jumpTo(search.matches[search.active].node);
    }
  }

  function clearSearch() {
    search.input.value = "";
    runSearch("");
    search.input.focus();
  }

  // ---- search: jump-to ----

  function jumpTo(node) {
    markTarget(node);
    if (state.mode === "heatmap") {
      revealInHeatmap(node);
    } else {
      revealInTree(node);
    }
  }

  function markTarget(node) {
    if (search.target !== null) {
      const previous = treeItems.get(search.target);
      if (previous) {
        previous.item.classList.remove("node--found");
      }
    }
    search.target = node;
  }

  function clearTarget() {
    if (search.target === null) {
      return;
    }
    const entry = treeItems.get(search.target);
    if (entry) {
      entry.item.classList.remove("node--found");
    }
    search.target = null;
    if (state.mode === "heatmap" && heat.canvas !== null) {
      layoutHeatmap();
    }
  }

  function revealInTree(node) {
    for (const ancestor of ancestorsOf(node)) {
      const entry = treeItems.get(ancestor);
      if (entry) {
        setExpanded(entry.item, entry.row, true);
      }
    }
    const target = treeItems.get(node);
    if (!target) {
      return;
    }
    setExpanded(target.item, target.row, true);
    target.item.classList.add("node--found");
    target.row.scrollIntoView({ block: "center" });
    target.row.focus({ preventScroll: true });
  }

  function revealInHeatmap(node) {
    heat.path = ancestorsOf(node);
    heat.selected = null;
    layoutHeatmap();
    const el = cellFor(node);
    if (!el) {
      return;
    }
    if (node.kind === "file") {
      selectFile(node, el);
    }
    heat.canvas.scrollIntoView({ block: "center" });
    el.focus({ preventScroll: true });
  }

  function cellFor(node) {
    for (const el of heat.canvas.children) {
      if (el.dataset.path === node.path) {
        return el;
      }
    }
    return null;
  }
