(function () {
  "use strict";

  const doc = JSON.parse(document.getElementById("smell-data").textContent);
  const maxTotal = computeMaxTotal(doc);
  const app = document.getElementById("app");

  app.appendChild(renderHeader(doc));
  if (doc.errors.length > 0) {
    app.appendChild(renderErrors(doc.errors));
  }
  app.appendChild(renderTree(doc));

  function computeMaxTotal(document_) {
    let max = 0;
    walk(document_.roots);
    return max;

    function walk(nodes) {
      for (const node of nodes) {
        max = Math.max(max, node.violations.total);
        if (node.kind === "directory") {
          walk(node.children);
        }
      }
    }
  }

  function heatLevel(count) {
    if (count <= 0) {
      return 0;
    }
    if (maxTotal <= 0) {
      return 1;
    }
    const ratio = count / maxTotal;
    if (ratio > 0.75) {
      return 4;
    }
    if (ratio > 0.5) {
      return 3;
    }
    if (ratio > 0.25) {
      return 2;
    }
    return 1;
  }

  function label(measure) {
    return measure.charAt(0).toUpperCase() + measure.slice(1);
  }

  function renderBadges(breakdown, measures) {
    const wrap = document.createElement("span");
    wrap.className = "node__badges";
    wrap.appendChild(badge("total", breakdown.total));
    for (const measure of measures) {
      wrap.appendChild(badge(measure, breakdown[measure]));
    }
    return wrap;
  }

  function badge(measure, count) {
    const el = document.createElement("span");
    el.className = "badge";
    el.dataset.level = String(heatLevel(count));
    const value = document.createElement("span");
    value.textContent = String(count);
    const name = document.createElement("span");
    name.className = "badge__label";
    name.textContent = label(measure);
    el.append(value, name);
    return el;
  }

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

  function renderTree(document_) {
    if (document_.roots.length === 0) {
      const empty = document.createElement("p");
      empty.className = "empty";
      empty.textContent = "no files analyzed";
      return empty;
    }
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
    return item;
  }

  function toggleNode(item, row) {
    const nowExpanded = item.classList.toggle("node--expanded");
    item.classList.toggle("node--collapsed", !nowExpanded);
    row.setAttribute("aria-expanded", String(nowExpanded));
  }

  function renderFileDetail(node, measures) {
    const detail = document.createElement("div");
    detail.className = "file__detail";

    if (node.lines === null) {
      detail.appendChild(detailNote("not analyzed"));
      return detail;
    }

    let any = false;
    for (const measure of measures) {
      const value = node.detail[measure];
      if (Array.isArray(value)) {
        for (const offender of value) {
          any = true;
          detail.appendChild(detailRow(label(measure) + " · " + offender.name, offender.value));
        }
      } else if (value !== null) {
        any = true;
        detail.appendChild(detailRow(label(measure), value));
      }
    }
    if (!any) {
      detail.appendChild(detailNote("no violations"));
    }
    return detail;
  }

  function detailRow(name, value) {
    const row = document.createElement("div");
    row.className = "file__detail-row";
    const nameEl = document.createElement("span");
    nameEl.className = "file__detail-name";
    nameEl.textContent = name;
    const valueEl = document.createElement("span");
    valueEl.textContent = String(value);
    row.append(nameEl, valueEl);
    return row;
  }

  function detailNote(text) {
    const note = document.createElement("div");
    note.className = "file__detail-row file__detail-empty";
    note.textContent = text;
    return note;
  }
})();
