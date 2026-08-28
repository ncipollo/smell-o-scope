(function () {
  "use strict";

  const MIN_WEIGHT = 1;

  const doc = JSON.parse(document.getElementById("smell-data").textContent);
  const maxTotal = computeMaxTotal(doc);
  const sizes = computeSizes(doc);
  const app = document.getElementById("app");
  const viewHost = document.createElement("div");
  viewHost.className = "view";
  const views = { directory: null, heatmap: null };
  const state = { mode: "directory" };
  const heat = { path: [], selected: null, canvas: null, crumbs: null, legend: null, detail: null };
  const tooltip = createTooltip();
  let modeButtons = [];
  let resizePending = false;

  main();

  function main() {
    app.appendChild(renderHeader(doc));
    if (doc.errors.length > 0) {
      app.appendChild(renderErrors(doc.errors));
    }
    if (doc.roots.length === 0) {
      app.appendChild(emptyState("no files analyzed"));
      return;
    }
    app.appendChild(renderModes());
    app.appendChild(viewHost);
    document.body.appendChild(tooltip.element);
    window.addEventListener("resize", scheduleLayout);
    document.addEventListener("keydown", onKeydown);
    showMode("directory");
  }

  // ---- shared: measures, badges, heat scale, file detail ----

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

  function heatLevelFor(count, max) {
    if (count <= 0) {
      return 0;
    }
    if (max <= 0) {
      return 1;
    }
    const ratio = count / max;
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

  function heatLevel(count) {
    return heatLevelFor(count, maxTotal);
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

  function emptyState(text) {
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.textContent = text;
    return empty;
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

