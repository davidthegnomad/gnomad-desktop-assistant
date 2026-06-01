#!/usr/bin/env node
/**
 * Export all project Markdown docs to polished .txt and .html siblings.
 * Usage: node scripts/export-docs.mjs
 */
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname, basename, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { marked } from "marked";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");

const CSS = readFileSync(join(ROOT, "docs", "_assets", "doc-theme.css"), "utf8");

/** @param {string} dir */
function collectMarkdownFiles(dir, acc = []) {
  for (const name of readdirSync(dir)) {
    if (name.startsWith("_")) continue;
    const full = join(dir, name);
    const st = statSync(full);
    if (st.isDirectory()) {
      if (name === "node_modules" || name === "dist" || name === "target") continue;
      collectMarkdownFiles(full, acc);
    } else if (name.endsWith(".md")) {
      acc.push(full);
    }
  }
  return acc;
}

/** @param {string} text */
function slugify(text) {
  return text
    .toLowerCase()
    .replace(/<[^>]+>/g, "")
    .replace(/[^\w\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-")
    .slice(0, 64);
}

/** @param {string} md */
function markdownToPlainText(md) {
  const lines = md.replace(/\r\n/g, "\n").split("\n");
  const out = [];
  let inCode = false;
  let codeFence = "";
  let tableRows = [];

  const flushTable = () => {
    if (tableRows.length === 0) return;
    const parsed = tableRows
      .filter((row) => !/^\|[\s\-:|]+\|$/.test(row.trim()))
      .map((row) =>
        row
          .split("|")
          .slice(1, -1)
          .map((c) => stripInline(c.trim()))
      );
    if (parsed.length === 0) {
      tableRows = [];
      return;
    }
    const widths = parsed[0].map((_, i) =>
      Math.max(...parsed.map((r) => (r[i] || "").length), 3)
    );
    for (const row of parsed) {
      const cells = row.map((c, i) => (c || "").padEnd(widths[i]));
      out.push("  " + cells.join("  |  "));
    }
    out.push("");
    tableRows = [];
  };

  /** @param {string} s */
  function stripInline(s) {
    return s
      .replace(/!\[([^\]]*)\]\([^)]+\)/g, "$1")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .replace(/\*\*([^*]+)\*\*/g, "$1")
      .replace(/\*([^*]+)\*/g, "$1")
      .replace(/`([^`]+)`/g, "$1")
      .replace(/<[^>]+>/g, "");
  }

  /** @param {string} title */
  function formatTitle(title, level) {
    const t = stripInline(title).trim();
    if (!t) return;
    const bar = level === 1 ? "=" : level === 2 ? "-" : "·";
    if (level <= 2) {
      out.push("");
      out.push(t.toUpperCase());
      out.push(bar.repeat(Math.max(t.length, bar.length)));
      out.push("");
    } else {
      out.push("");
      out.push(`${"  ".repeat(Math.max(0, level - 3))}${t}`);
      out.push("");
    }
  }

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    if (trimmed.startsWith("```")) {
      if (!inCode) {
        flushTable();
        inCode = true;
        codeFence = trimmed.slice(3).trim();
        out.push(`  [${codeFence || "code"}]`);
      } else {
        inCode = false;
        codeFence = "";
        out.push("");
      }
      continue;
    }

    if (inCode) {
      out.push("    " + line);
      continue;
    }

    if (trimmed.startsWith("|")) {
      tableRows.push(trimmed);
      continue;
    } else {
      flushTable();
    }

    if (/^---+$/.test(trimmed) || /^\*\*\*+$/.test(trimmed)) {
      out.push("────────────────────────────────────────");
      out.push("");
      continue;
    }

    const h = /^(#{1,6})\s+(.+)$/.exec(line);
    if (h) {
      formatTitle(h[2], h[1].length);
      continue;
    }

    if (/^[-*+]\s+/.test(trimmed)) {
      out.push("  • " + stripInline(trimmed.replace(/^[-*+]\s+/, "")));
      continue;
    }

    if (/^\d+\.\s+/.test(trimmed)) {
      out.push("  " + stripInline(trimmed));
      continue;
    }

    if (trimmed === "") {
      if (out.length && out[out.length - 1] !== "") out.push("");
      continue;
    }

    out.push(stripInline(line));
  }

  flushTable();

  let text = out.join("\n");
  text = text.replace(/\n{3,}/g, "\n\n").trim();
  return text;
}

/** @param {string} md @param {string} title @param {string} relPath */
function markdownToHtml(md, title, relPath) {
  const headings = [];
  const renderer = new marked.Renderer();
  renderer.heading = function ({ tokens, depth }) {
    const text = marked.Parser.parseInline(tokens);
    const id = slugify(text);
    if (depth >= 2 && depth <= 3) headings.push({ id, text, depth });
    return `<h${depth} id="${id}">${text}</h${depth}>`;
  };

  const body = marked.parse(md, { renderer, gfm: true, breaks: false });

  const toc =
    headings.length > 0
      ? `<nav class="sidebar" aria-label="Table of contents">
          <div class="brand"><span class="brand-icon">🍄</span><h1>Gnomad</h1></div>
          <p class="doc-path">${relPath}</p>
          ${headings
            .map(
              (h) =>
                `<a href="#${h.id}" class="toc-h${h.depth}">${h.text.replace(/<[^>]+>/g, "")}</a>`
            )
            .join("\n")}
        </nav>`
      : `<nav class="sidebar" aria-label="Document">
          <div class="brand"><span class="brand-icon">🍄</span><h1>Gnomad</h1></div>
          <p class="doc-path">${relPath}</p>
        </nav>`;

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>${escapeHtml(title)} — Gnomad Desktop Assistant</title>
  <meta name="description" content="${escapeHtml(title)} documentation for Gnomad Desktop Assistant." />
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Google+Sans+Flex:opsz,wght@8..144,400;8..144,500;8..144,600;8..144,700&family=Roboto:wght@400;500;700&display=swap" rel="stylesheet" />
  <style>${CSS}</style>
</head>
<body>
  <div class="mesh-bg" aria-hidden="true"></div>
  <div class="layout">
    ${toc}
    <main class="content doc-body">
      <header class="hero">
        <h2>${escapeHtml(title)}</h2>
        <p class="formats">Also available: <a href="${basename(relPath, ".md")}.md">Markdown</a> · <a href="${basename(relPath, ".md")}.txt">Plain text</a></p>
      </header>
      ${body}
      <footer class="doc-footer">
        <p>Built with <span aria-label="love">❤️</span> by <a href="https://gnomadstudio.org">Gnomad Studio</a> 🦙</p>
      </footer>
    </main>
  </div>
  <script>
    const links = document.querySelectorAll(".sidebar a[href^='#']");
    const sections = [...links].map((a) => document.querySelector(a.getAttribute("href")));
    const onScroll = () => {
      let cur = 0;
      sections.forEach((s, i) => { if (s && s.getBoundingClientRect().top <= 120) cur = i; });
      links.forEach((l, i) => l.classList.toggle("active", i === cur));
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
  </script>
</body>
</html>`;
}

/** @param {string} s */
function escapeHtml(s) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** @param {string} md */
function extractTitle(md, fallback) {
  const m = /^#\s+(.+)$/m.exec(md);
  if (m) return stripForTitle(m[1]);
  return fallback;
}

/** @param {string} s */
function stripForTitle(s) {
  return s.replace(/\*\*/g, "").replace(/—.*/, "").trim();
}

/** @param {string} mdPath */
function wrapPlainText(mdPath, title, body) {
  const rel = relative(ROOT, mdPath);
  const line = "═".repeat(72);
  return `${line}
  ${title.toUpperCase()}
  Gnomad Desktop Assistant · ${rel}
${line}

${body}

${line}
Built with ❤️ by Gnomad Studio 🦙
https://gnomadstudio.org
${line}
`;
}

function main() {
  const files = [
    join(ROOT, "README.md"),
    join(ROOT, "CHANGELOG.md"),
    join(ROOT, "CONTRIBUTING.md"),
    ...collectMarkdownFiles(join(ROOT, "docs")),
  ];

  /** Paths that must not be overwritten (e.g. GitHub Pages landing). */
  const SKIP_HTML = new Set([
    join(ROOT, "docs", "index.html").toLowerCase(),
  ]);

  let docCount = 0;
  let fileCount = 0;
  for (const mdPath of files) {
    const md = readFileSync(mdPath, "utf8");
    const base = mdPath.slice(0, -3);
    const rel = relative(ROOT, mdPath);
    const title = extractTitle(md, basename(mdPath, ".md"));
    const htmlPath = `${base}.html`;

    writeFileSync(`${base}.txt`, wrapPlainText(mdPath, title, markdownToPlainText(md)), "utf8");
    fileCount++;

    if (SKIP_HTML.has(htmlPath.toLowerCase())) {
      console.log(`  ⊘ ${rel} → .txt only (reserved ${basename(htmlPath)})`);
      docCount++;
      continue;
    }

    writeFileSync(htmlPath, markdownToHtml(md, title, rel), "utf8");
    fileCount++;
    docCount++;
    console.log(`  ✓ ${rel} → .txt, .html`);
  }

  console.log(`\nExported ${docCount} documents (${fileCount} files).`);
}

main();
