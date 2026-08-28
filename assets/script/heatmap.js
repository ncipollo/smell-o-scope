  // ---- heatmap: size index ----

  function computeSizes(document_) {
    const map = new WeakMap();
    walk(document_.roots);
    return map;

    function walk(nodes) {
      let weight = 0;
      let lines = 0;
      let files = 0;
      for (const node of nodes) {
        const size = node.kind === "directory" ? walk(node.children) : fileSize(node);
        map.set(node, size);
        weight += size.weight;
        lines += size.lines;
        files += size.files;
      }
      return { weight: Math.max(weight, MIN_WEIGHT), lines: lines, files: files };
    }
  }

  function fileSize(node) {
    const lines = node.lines || 0;
    return { weight: Math.max(lines, MIN_WEIGHT), lines: lines, files: 1 };
  }

  function sizeOf(node) {
    return sizes.get(node);
  }

  // ---- heatmap: squarified treemap layout (pure, no DOM) ----

  function heatmapItems(nodes) {
    const items = [];
    for (const node of nodes) {
      items.push({ node: node, weight: sizeOf(node).weight });
    }
    items.sort((a, b) => b.weight - a.weight || (a.node.path < b.node.path ? -1 : 1));
    return items;
  }

  function weightSum(items) {
    let sum = 0;
    for (const item of items) {
      sum += item.weight;
    }
    return sum || 1;
  }

  // Bruls/Huizing/van Wijk squarified treemap. Lays out one level at a time
  // (cells are the immediate children of the current zoom node) rather than
  // recursing into a nested layout — keeps hit-testing and tooltips
  // unambiguous, and keeps this a pure function of one rectangle.
  function squarify(items, rect) {
    const cells = [];
    let box = { x: rect.x, y: rect.y, w: rect.w, h: rect.h };
    let rest = items;
    while (rest.length > 0 && box.w > 0 && box.h > 0) {
      const scale = (box.w * box.h) / weightSum(rest);
      const row = nextRow(rest, box, scale);
      const laid = layoutRow(row, box, scale);
      for (const cell of laid.cells) {
        cells.push(cell);
      }
      box = shrinkBox(box, laid);
      rest = rest.slice(row.length);
    }
    return cells;
  }

  function nextRow(items, box, scale) {
    const side = Math.min(box.w, box.h);
    let row = items.slice(0, 1);
    let best = worstRatio(row, side, scale);
    for (let i = 1; i < items.length; i++) {
      const candidate = items.slice(0, i + 1);
      const ratio = worstRatio(candidate, side, scale);
      if (ratio > best) {
        break;
      }
      row = candidate;
      best = ratio;
    }
    return row;
  }

  function worstRatio(row, side, scale) {
    const area = weightSum(row) * scale;
    const min = row[row.length - 1].weight * scale;
    const max = row[0].weight * scale;
    const span = side * side;
    return Math.max((span * max) / (area * area), (area * area) / (span * min));
  }

  function layoutRow(row, box, scale) {
    const area = weightSum(row) * scale;
    const horizontal = box.w <= box.h;
    const thickness = horizontal ? area / box.w : area / box.h;
    const cells = [];
    let offset = 0;
    for (const item of row) {
      const length = (item.weight * scale) / thickness;
      cells.push(
        horizontal
          ? { node: item.node, x: box.x + offset, y: box.y, w: length, h: thickness }
          : { node: item.node, x: box.x, y: box.y + offset, w: thickness, h: length },
      );
      offset += length;
    }
    return { cells: cells, thickness: thickness, horizontal: horizontal };
  }

  function shrinkBox(box, laid) {
    return laid.horizontal
      ? { x: box.x, y: box.y + laid.thickness, w: box.w, h: Math.max(box.h - laid.thickness, 0) }
      : { x: box.x + laid.thickness, y: box.y, w: Math.max(box.w - laid.thickness, 0), h: box.h };
  }

  // ---- heatmap: drill-down state ----

  function currentNodes() {
    return heat.path.length === 0 ? doc.roots : heat.path[heat.path.length - 1].children;
  }

  function initialPath(document_) {
    const path = [];
    let nodes = document_.roots;
    while (nodes.length === 1 && nodes[0].kind === "directory" && nodes[0].children.length > 0) {
      path.push(nodes[0]);
      nodes = nodes[0].children;
    }
    return path;
  }

  function isDrillable(node) {
    return node.kind === "directory" && node.children.length > 0;
  }

  function isUnanalyzed(node) {
    return node.kind === "file" && node.lines === null;
  }

  function drillInto(node) {
    heat.path.push(node);
    heat.selected = null;
    layoutHeatmap();
  }

  function navigateTo(depth) {
    heat.path = heat.path.slice(0, depth);
    heat.selected = null;
    layoutHeatmap();
  }

  function selectFile(node, el) {
    heat.selected = node;
    for (const cell of heat.canvas.children) {
      cell.classList.remove("cell--selected");
    }
    el.classList.add("cell--selected");
    const heading = document.createElement("p");
    heading.className = "heatmap__detail-heading";
    heading.textContent = node.path;
    heat.detail.replaceChildren(heading, renderFileDetail(node, doc.measures));
  }

  function activateCell(node, el) {
    if (node.kind === "directory") {
      if (isDrillable(node)) {
        drillInto(node);
      }
    } else {
      selectFile(node, el);
    }
  }

  // ---- heatmap: shell + layout orchestration ----

  function renderHeatmap() {
    heat.path = initialPath(doc);
    heat.selected = null;

    const wrap = document.createElement("div");
    wrap.className = "heatmap";

    const bar = document.createElement("div");
    bar.className = "heatmap__bar";
    heat.crumbs = document.createElement("div");
    heat.crumbs.className = "crumbs";
    heat.legend = document.createElement("div");
    heat.legend.className = "legend";
    heat.legend.setAttribute("role", "img");
    heat.legend.setAttribute("aria-label", "violation color scale");
    bar.append(heat.crumbs, heat.legend);

    heat.canvas = document.createElement("div");
    heat.canvas.className = "heatmap__canvas";

    heat.detail = document.createElement("div");
    heat.detail.className = "heatmap__detail";

    wrap.append(bar, heat.canvas, heat.detail);
    return wrap;
  }

  function layoutHeatmap() {
    tooltip.hide();
    renderCrumbs();
    const nodes = currentNodes();
    const max = levelMax(nodes);
    renderLegend(nodes, max);
    heat.detail.replaceChildren();

    if (nodes.length === 0) {
      heat.canvas.replaceChildren(emptyState("nothing to show"));
      return;
    }
    const rect = heat.canvas.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) {
      return;
    }
    const items = heatmapItems(nodes);
    const cells = squarify(items, { x: 0, y: 0, w: rect.width, h: rect.height });
    const elements = [];
    for (const cell of cells) {
      elements.push(renderCell(cell, max));
    }
    heat.canvas.replaceChildren(...elements);
  }

  function levelMax(nodes) {
    let max = 0;
    for (const node of nodes) {
      max = Math.max(max, node.violations.total);
    }
    return max;
  }

  // ---- heatmap: breadcrumb ----

  function renderCrumbs() {
    const items = [crumb("all", 0, heat.path.length === 0)];
    for (let i = 0; i < heat.path.length; i++) {
      items.push(crumbSeparator());
      items.push(crumb(heat.path[i].name, i + 1, i === heat.path.length - 1));
    }
    heat.crumbs.replaceChildren(...items);
  }

  function crumb(text, depth, current) {
    if (current) {
      const span = document.createElement("span");
      span.className = "crumbs__item crumbs__item--current";
      span.setAttribute("aria-current", "location");
      span.textContent = text;
      return span;
    }
    const button = document.createElement("button");
    button.className = "crumbs__item";
    button.textContent = text;
    button.addEventListener("click", () => navigateTo(depth));
    return button;
  }

  function crumbSeparator() {
    const sep = document.createElement("span");
    sep.className = "crumbs__sep";
    sep.setAttribute("aria-hidden", "true");
    sep.textContent = "/";
    return sep;
  }

  // ---- heatmap: legend ----

  function renderLegend(nodes, max) {
    const parts = [];
    if (max <= 0) {
      parts.push(legendSwatch(0, null));
      const note = document.createElement("span");
      note.className = "legend__note";
      note.textContent = "no violations at this level";
      parts.push(note);
    } else {
      const caption = document.createElement("span");
      caption.className = "legend__caption";
      caption.textContent = "violations";
      parts.push(caption);

      const low = document.createElement("span");
      low.className = "legend__end";
      low.textContent = "0";
      parts.push(low);

      const scale = document.createElement("span");
      scale.className = "legend__scale";
      for (let level = 0; level <= 4; level++) {
        scale.appendChild(legendSwatch(level, heatRange(level, max)));
      }
      parts.push(scale);

      const high = document.createElement("span");
      high.className = "legend__end";
      high.textContent = String(max);
      parts.push(high);
    }

    if (hasUnanalyzed(nodes)) {
      parts.push(unanalyzedNote());
    }

    heat.legend.replaceChildren(...parts);
  }

  function legendSwatch(level, range) {
    const swatch = document.createElement("span");
    swatch.className = "legend__swatch";
    swatch.dataset.level = String(level);
    swatch.setAttribute("aria-hidden", "true");
    if (range !== null) {
      swatch.title = range;
    }
    return swatch;
  }

  function heatRange(level, max) {
    if (level === 0) {
      return "0";
    }
    if (max <= 4) {
      return null;
    }
    const lo = level === 1 ? 1 : Math.floor(max * (level - 1) * 0.25) + 1;
    const hi = level === 4 ? max : Math.floor(max * level * 0.25);
    return lo >= hi ? String(hi) : lo + "–" + hi;
  }

  function hasUnanalyzed(nodes) {
    for (const node of nodes) {
      if (isUnanalyzed(node)) {
        return true;
      }
    }
    return false;
  }

  function unanalyzedNote() {
    const note = document.createElement("span");
    note.className = "legend__note";
    const swatch = document.createElement("span");
    swatch.className = "legend__swatch";
    swatch.dataset.state = "unanalyzed";
    note.append(swatch, document.createTextNode("not analyzed"));
    return note;
  }

  // ---- heatmap: cells ----

  function renderCell(cell, max) {
    const node = cell.node;
    const el = document.createElement("div");
    el.className = "cell cell--" + node.kind;
    el.dataset.path = node.path;
    el.dataset.level = String(heatLevelFor(node.violations.total, max));
    if (isUnanalyzed(node)) {
      el.dataset.state = "unanalyzed";
    }
    if (heat.selected === node) {
      el.classList.add("cell--selected");
    }
    positionCell(el, cell);
    el.appendChild(cellLabel(node, cell));

    const interactive = node.kind === "file" || isDrillable(node);
    if (interactive) {
      el.tabIndex = 0;
      el.setAttribute("role", "button");
      el.addEventListener("click", () => activateCell(node, el));
      el.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          activateCell(node, el);
        }
      });
      el.addEventListener("focus", () => tooltip.show(node, cellAnchor(el)));
      el.addEventListener("blur", () => tooltip.hide());
    }
    el.addEventListener("pointerenter", (event) => tooltip.show(node, { x: event.clientX, y: event.clientY }));
    el.addEventListener("pointermove", (event) => tooltip.move({ x: event.clientX, y: event.clientY }));
    el.addEventListener("pointerleave", () => tooltip.hide());

    return el;
  }

  function positionCell(el, cell) {
    const left = Math.round(cell.x);
    const top = Math.round(cell.y);
    const width = Math.round(cell.x + cell.w) - left;
    const height = Math.round(cell.y + cell.h) - top;
    el.style.left = left + "px";
    el.style.top = top + "px";
    el.style.width = width + "px";
    el.style.height = height + "px";
  }

  function cellLabel(node, cell) {
    const wrap = document.createElement("div");
    wrap.className = "cell__label";
    if (cell.w < 32 || cell.h < 18) {
      return wrap;
    }
    const name = document.createElement("span");
    name.className = "cell__name";
    name.textContent = node.name;
    wrap.appendChild(name);
    if (node.violations.total > 0) {
      const count = document.createElement("span");
      count.className = "cell__count";
      count.textContent = String(node.violations.total);
      wrap.appendChild(count);
    }
    return wrap;
  }

  function cellAnchor(el) {
    const rect = el.getBoundingClientRect();
    return { x: rect.left, y: rect.bottom };
  }

