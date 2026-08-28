  // ---- heatmap: tooltip ----

  function createTooltip() {
    const element = document.createElement("div");
    element.className = "tooltip";
    element.setAttribute("role", "tooltip");
    element.hidden = true;

    const path = document.createElement("p");
    path.className = "tooltip__path";
    const meta = document.createElement("p");
    meta.className = "tooltip__meta";
    const rows = document.createElement("div");
    rows.className = "tooltip__rows";
    const hint = document.createElement("p");
    hint.className = "tooltip__hint";
    element.append(path, meta, rows, hint);

    return {
      element: element,
      show: function (node, anchor) {
        path.textContent = node.path;
        meta.textContent = tooltipMeta(node);
        rows.replaceChildren(...violationRows(node));
        hint.textContent = node.kind === "file" ? "click for detail" : isDrillable(node) ? "click to open" : "";
        element.hidden = false;
        place(anchor);
      },
      move: function (anchor) {
        if (!element.hidden) {
          place(anchor);
        }
      },
      hide: function () {
        element.hidden = true;
      },
    };

    function place(anchor) {
      const width = element.offsetWidth;
      const height = element.offsetHeight;
      let left = anchor.x + 14;
      let top = anchor.y + 16;
      if (left + width > window.innerWidth) {
        left = anchor.x - width - 14;
      }
      if (top + height > window.innerHeight) {
        top = anchor.y - height - 16;
      }
      element.style.left = Math.max(left, 0) + "px";
      element.style.top = Math.max(top, 0) + "px";
    }
  }

  function tooltipMeta(node) {
    const size = sizeOf(node);
    if (node.kind === "file") {
      return node.lines === null ? "file · not analyzed" : "file · " + size.lines + " lines";
    }
    return "directory · " + size.files + " files · " + size.lines + " lines";
  }

  function violationRows(node) {
    const rows = [detailRow("Total", node.violations.total)];
    for (const measure of doc.measures) {
      rows.push(detailRow(label(measure), node.violations[measure]));
    }
    return rows;
  }
})();
