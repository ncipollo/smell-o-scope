  // ---- header / errors ----

  function renderHeader(document_) {
    const header = document.createElement("div");
    header.className = "header";

    const title = document.createElement("h1");
    title.className = "header__title";
    title.textContent = document_.tool.name + " " + document_.tool.version;
    header.appendChild(title);

    const meta = document.createElement("p");
    meta.className = "header__meta";
    meta.textContent =
      document_.aggregation +
      " aggregation · measures: " +
      (document_.measures.length > 0 ? document_.measures.map(label).join(", ") : "none configured");
    header.appendChild(meta);

    const totals = document.createElement("div");
    totals.className = "header__totals";
    totals.appendChild(renderBadges(document_.totals, document_.measures));
    header.appendChild(totals);

    return header;
  }

  function renderErrors(errors) {
    const box = document.createElement("div");
    box.className = "errors";

    const title = document.createElement("p");
    title.className = "errors__title";
    title.textContent = errors.length + " path error(s)";
    box.appendChild(title);

    for (const error of errors) {
      const line = document.createElement("p");
      line.textContent = error.path + ": " + error.message;
      box.appendChild(line);
    }
    return box;
  }

  // ---- mode toggle ----

  function renderModes() {
    const wrap = document.createElement("div");
    wrap.className = "modes";
    wrap.setAttribute("role", "group");
    wrap.setAttribute("aria-label", "display mode");
    modeButtons = [modeButton("directory", "Directory"), modeButton("heatmap", "Heat map")];
    wrap.append(modeButtons[0], modeButtons[1]);
    return wrap;
  }

  function modeButton(mode, text) {
    const button = document.createElement("button");
    button.className = "modes__button";
    button.dataset.mode = mode;
    button.textContent = text;
    button.addEventListener("click", () => showMode(mode));
    return button;
  }

  function showMode(mode) {
    state.mode = mode;
    tooltip.hide();
    if (!views[mode]) {
      views[mode] = mode === "directory" ? renderTree(doc) : renderHeatmap();
    }
    viewHost.replaceChildren(views[mode]);
    updateModeButtons();
    if (mode === "heatmap") {
      layoutHeatmap();
    }
  }

  function updateModeButtons() {
    for (const button of modeButtons) {
      button.setAttribute("aria-pressed", String(button.dataset.mode === state.mode));
    }
  }

  function onKeydown(event) {
    if (event.key === "Escape") {
      tooltip.hide();
    }
  }

  function scheduleLayout() {
    if (state.mode !== "heatmap" || resizePending) {
      return;
    }
    resizePending = true;
    window.requestAnimationFrame(function () {
      resizePending = false;
      layoutHeatmap();
    });
  }

  // ---- directory view ----

  function renderTree(document_) {
    const list = document.createElement("ul");
    list.className = "tree";
    for (const root of document_.roots) {
      list.appendChild(renderNode(root, document_.measures, true));
    }
    return list;
  }

  function renderNode(node, measures, expanded) {
    return node.kind === "directory"
      ? renderDirectory(node, measures, expanded)
      : renderFile(node, measures, expanded);
  }

  function renderDirectory(node, measures, expanded) {
    const item = nodeItem(node, measures, expanded, node.children.length > 0);

    const children = document.createElement("ul");
    children.className = "node__children";
    for (const child of node.children) {
      children.appendChild(renderNode(child, measures, false));
    }
    item.appendChild(children);
    return item;
  }

  function renderFile(node, measures, expanded) {
    const item = nodeItem(node, measures, expanded, true);
    item.appendChild(renderFileDetail(node, measures));
    return item;
  }

  function nodeItem(node, measures, expanded, expandable) {
    const item = document.createElement("li");
    item.className = "node node--" + node.kind + " " + (expanded ? "node--expanded" : "node--collapsed");

    const row = document.createElement("div");
    row.className = "node__row";
    row.tabIndex = 0;
    row.setAttribute("role", "button");
    row.setAttribute("aria-expanded", String(expanded));
    row.title = node.path;

    const toggle = document.createElement("button");
    toggle.className = "node__toggle" + (expandable ? "" : " node__toggle--leaf");
    toggle.setAttribute("aria-hidden", "true");
    toggle.tabIndex = -1;
    toggle.textContent = "▶";
    row.appendChild(toggle);

    const icon = document.createElement("span");
    icon.className = "node__icon";
    icon.setAttribute("aria-hidden", "true");
    icon.textContent = node.kind === "directory" ? "📁" : "📄";
    row.appendChild(icon);

    const name = document.createElement("span");
    name.className = "node__name";
    name.textContent = node.name;
    row.appendChild(name);

    row.appendChild(renderBadges(node.violations, measures));

    if (expandable) {
      row.addEventListener("click", () => toggleNode(item, row));
      row.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          toggleNode(item, row);
        }
      });
    }

    item.appendChild(row);
    treeItems.set(node, { item: item, row: row });
    return item;
  }

  function setExpanded(item, row, expanded) {
    item.classList.toggle("node--expanded", expanded);
    item.classList.toggle("node--collapsed", !expanded);
    row.setAttribute("aria-expanded", String(expanded));
  }

  function toggleNode(item, row) {
    setExpanded(item, row, !item.classList.contains("node--expanded"));
  }

