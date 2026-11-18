#!/usr/bin/env python3
"""Static Offline Documentation Portal Builder for `solix wiki`.

Converts user-facing Markdown files under `docs/` (excluding `docs/internals/`)
into a self-contained static HTML documentation site staged into
`crates/solix-toolchain/assets/docs/` with relative navigation, syntax
styling, and an offline `search_index.js` script compatible with `file://`.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import html
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tomllib
from urllib.parse import quote


USER_DOC_DIRS = ("guide", "wiki", "features", "spec", "architecture", "overview")
SECTION_TITLES = {
    "home": "Overview",
    "guide": "User Guides",
    "features": "Features",
    "wiki": "Reference Wiki",
    "spec": "Language & CLI Specs",
    "architecture": "Architecture",
    "overview": "System Overview",
    "api": "Library API Docs",
}
SECTION_ORDER = ["home", "guide", "wiki", "features", "spec", "architecture", "overview", "api"]
ALERT_LABELS = {
    "NOTE": ("note", "Note"),
    "TIP": ("tip", "Tip"),
    "IMPORTANT": ("important", "Important"),
    "WARNING": ("warning", "Warning"),
    "CAUTION": ("caution", "Caution"),
}

HEADING_RE = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")
FENCE_RE = re.compile(r"^\s*(```|~~~)\s*([A-Za-z0-9_+-]*)\s*$")
ORDERED_ITEM_RE = re.compile(r"^\s*\d+\.\s+(.*)$")
UNORDERED_ITEM_RE = re.compile(r"^\s*[-*+]\s+(.*)$")
HR_RE = re.compile(r"^\s*(?:-{3,}|\*{3,}|_{3,})\s*$")
ALERT_HEADER_RE = re.compile(r"^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*(.*)$")


@dataclass(frozen=True)
class DocPage:
    """Represents a source Markdown page and its rendered target location."""

    source_path: Path
    repo_rel_path: str
    out_rel_path: str
    section: str
    title: str
    headings: list[str]
    plain_excerpt: str
    markdown_text: str


def find_repo_root(start: Path) -> Path:
    """Locates the Solix repository root containing Cargo.toml and docs/."""
    candidate = start.resolve()
    for parent in (candidate, *candidate.parents):
        if (parent / "Cargo.toml").is_file() and (parent / "docs").is_dir():
            return parent
    raise FileNotFoundError("Could not locate Solix repository root containing Cargo.toml and docs/")


def read_workspace_version(repo_root: Path) -> str:
    """Reads `[workspace.package].version` from root Cargo.toml."""
    cargo_toml_path = repo_root / "Cargo.toml"
    data = tomllib.loads(cargo_toml_path.read_text(encoding="utf-8"))
    version = data.get("workspace", {}).get("package", {}).get("version")
    if isinstance(version, str) and version.strip():
        return version.strip()
    return "0.0.0"


def slugify_heading(text: str) -> str:
    """Creates GitHub-compatible anchor slugs for Markdown headings."""
    cleaned = re.sub(r"`([^`]*)`", r"\1", text).strip().lower()
    cleaned = re.sub(r"[^\w\s-]", "", cleaned)
    cleaned = re.sub(r"\s+", "-", cleaned)
    cleaned = re.sub(r"-{2,}", "-", cleaned)
    return cleaned.strip("-") or "section"


def markdown_rel_to_html_rel(docs_rel_path: Path) -> str:
    """Maps a path relative to `docs/` to its static HTML path (`README.md` -> `index.html`)."""
    posix_parts = list(docs_rel_path.parts)
    if not posix_parts:
        return "index.html"
    filename = posix_parts[-1]
    if filename.lower() == "readme.md":
        posix_parts[-1] = "index.html"
    elif filename.lower().endswith(".md"):
        posix_parts[-1] = filename[:-3] + ".html"
    return "/".join(posix_parts)


def collect_user_doc_sources(docs_root: Path) -> list[tuple[Path, str]]:
    """Collects user-facing Markdown files from `docs/`, excluding `docs/internals/`."""
    discovered: list[tuple[Path, str]] = []
    root_readme = docs_root / "README.md"
    if root_readme.is_file():
        discovered.append((root_readme, "home"))

    for dirname in USER_DOC_DIRS:
        subdir = docs_root / dirname
        if not subdir.is_dir():
            continue
        for md_file in sorted(subdir.rglob("*.md")):
            if "internals" in md_file.relative_to(docs_root).parts:
                continue
            discovered.append((md_file, dirname))

    return discovered


def extract_title_and_headings(markdown_text: str, fallback_name: str) -> tuple[str, list[str]]:
    """Extracts the primary `#` title and all heading titles outside code fences."""
    title: str | None = None
    headings: list[str] = []
    in_fence = False
    for line in markdown_text.splitlines():
        if FENCE_RE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        match = HEADING_RE.match(line)
        if match:
            heading_text = re.sub(r"`([^`]*)`", r"\1", match.group(2)).strip()
            heading_text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", heading_text)
            if len(match.group(1)) == 1 and title is None:
                title = heading_text
            headings.append(heading_text)
    if not title:
        title = fallback_name.replace("_", " ").replace("-", " ").title()
    return title, headings


def extract_plain_excerpt(markdown_text: str) -> str:
    """Produces clean searchable text for client-side offline search."""
    without_fences = re.sub(r"```[\s\S]*?```", " ", markdown_text)
    without_links = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", without_fences)
    without_markup = re.sub(r"[#>*`|_~]", " ", without_links)
    collapsed = re.sub(r"\s+", " ", without_markup).strip()
    return collapsed[:1200]


def rewrite_link_target(
    raw_target: str,
    source_file: Path,
    docs_root: Path,
    repo_root: Path,
    user_page_map: dict[Path, str],
    current_out_rel: str,
    version: str,
) -> str:
    """Rewrites relative `.md` links to static `.html` paths or repository fallback URLs."""
    target = raw_target.strip()
    if not target or target.startswith(("#", "http://", "https://", "mailto:", "data:")):
        return target

    base_part, hash_sep, fragment = target.partition("#")
    anchor = f"#{fragment}" if hash_sep else ""
    if not base_part:
        return anchor

    resolved = (source_file.parent / base_part).resolve()
    if resolved in user_page_map:
        dest_rel = user_page_map[resolved]
        current_dir = os.path.dirname(current_out_rel)
        rel_href = os.path.relpath(dest_rel, start=current_dir or ".")
        return rel_href.replace(os.sep, "/") + anchor

    try:
        repo_rel = resolved.relative_to(repo_root).as_posix()
        return f"https://github.com/Jehudme/solix-language/blob/v{version}/{quote(repo_rel, safe='/')}{anchor}"
    except ValueError:
        try:
            docs_rel = resolved.relative_to(docs_root).as_posix()
            return f"https://github.com/Jehudme/solix-language/blob/v{version}/docs/{quote(docs_rel, safe='/')}{anchor}"
        except ValueError:
            return target


def render_inline_markdown(
    text: str,
    *,
    link_rewriter: callable[[str], str],
) -> str:
    """Renders inline Markdown formatting (`code`, links, bold, italic, strikethrough)."""
    code_tokens: list[str] = []

    def stash_code(match: re.Match[str]) -> str:
        index = len(code_tokens)
        code_tokens.append(f"<code>{html.escape(match.group(1))}</code>")
        return f"\x00CODE{index}\x00"

    masked = re.sub(r"`([^`\n]+)`", stash_code, text)

    link_tokens: list[str] = []

    def stash_image(match: re.Match[str]) -> str:
        index = len(link_tokens)
        alt_text = html.escape(match.group(1))
        src_url = html.escape(link_rewriter(match.group(2)), quote=True)
        link_tokens.append(f'<img src="{src_url}" alt="{alt_text}" loading="lazy">')
        return f"\x00LINK{index}\x00"

    def stash_link(match: re.Match[str]) -> str:
        index = len(link_tokens)
        label_html = render_inline_markdown(match.group(1), link_rewriter=link_rewriter)
        raw_href = link_rewriter(match.group(2))
        escaped_href = html.escape(raw_href, quote=True)
        external_attr = (
            ' target="_blank" rel="noopener noreferrer"'
            if raw_href.startswith(("http://", "https://"))
            else ""
        )
        link_tokens.append(f'<a href="{escaped_href}"{external_attr}>{label_html}</a>')
        return f"\x00LINK{index}\x00"

    masked = re.sub(r"!\[([^\]]*)\]\(([^)\s]+)\)", stash_image, masked)
    masked = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", stash_link, masked)

    escaped = html.escape(masked)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    escaped = re.sub(r"__([^_]+)__", r"<strong>\1</strong>", escaped)
    escaped = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<em>\1</em>", escaped)
    escaped = re.sub(r"~~([^~]+)~~", r"<del>\1</del>", escaped)

    for index, token_html in enumerate(link_tokens):
        escaped = escaped.replace(f"\x00LINK{index}\x00", token_html)
    for index, token_html in enumerate(code_tokens):
        escaped = escaped.replace(f"\x00CODE{index}\x00", token_html)

    return escaped


def split_table_row(row_line: str) -> list[str]:
    """Splits a Markdown pipe table row into trimmed cells."""
    stripped = row_line.strip()
    if stripped.startswith("|"):
        stripped = stripped[1:]
    if stripped.endswith("|"):
        stripped = stripped[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", stripped)]


def is_table_separator(row_line: str) -> bool:
    """Checks whether a line is a Markdown table alignment separator row."""
    if "|" not in row_line:
        return False
    cells = split_table_row(row_line)
    if not cells:
        return False
    return all(bool(re.match(r"^:?-{2,}:?$", cell.strip())) for cell in cells if cell.strip() != "")


def parse_table_alignments(sep_line: str) -> list[str]:
    """Extracts left/center/right CSS text-align values from a table separator row."""
    alignments: list[str] = []
    for cell in split_table_row(sep_line):
        trimmed = cell.strip()
        if trimmed.startswith(":") and trimmed.endswith(":"):
            alignments.append("center")
        elif trimmed.endswith(":"):
            alignments.append("right")
        else:
            alignments.append("left")
    return alignments


def render_markdown_to_html(
    markdown_text: str,
    *,
    link_rewriter: callable[[str], str],
) -> tuple[str, list[tuple[int, str, str]]]:
    """Converts Markdown to HTML and collects `(level, slug, text)` TOC entries."""
    lines = markdown_text.splitlines()
    blocks: list[str] = []
    toc_entries: list[tuple[int, str, str]] = []
    slug_counts: dict[str, int] = {}
    line_idx = 0
    total_lines = len(lines)

    def unique_slug(raw_heading: str) -> str:
        base_slug = slugify_heading(raw_heading)
        count = slug_counts.get(base_slug, 0)
        slug_counts[base_slug] = count + 1
        return base_slug if count == 0 else f"{base_slug}-{count}"

    while line_idx < total_lines:
        line = lines[line_idx]
        stripped = line.strip()

        if not stripped:
            line_idx += 1
            continue

        # HTML comment blocks
        if stripped.startswith("<!--"):
            while line_idx < total_lines and "-->" not in lines[line_idx]:
                line_idx += 1
            line_idx += 1
            continue

        # Fenced code blocks
        fence_match = FENCE_RE.match(line)
        if fence_match:
            fence_marker = fence_match.group(1)
            lang = fence_match.group(2).lower()
            code_lines: list[str] = []
            line_idx += 1
            while line_idx < total_lines and not lines[line_idx].strip().startswith(fence_marker):
                code_lines.append(lines[line_idx])
                line_idx += 1
            line_idx += 1
            escaped_code = html.escape("\n".join(code_lines))
            lang_label = html.escape(lang or "text")
            if lang == "mermaid":
                blocks.append(
                    f'<div class="code-card"><div class="code-header"><span>mermaid diagram</span></div>'
                    f'<pre class="mermaid"><code>{escaped_code}</code></pre></div>'
                )
            else:
                blocks.append(
                    f'<div class="code-card"><div class="code-header"><span>{lang_label}</span>'
                    f'<button type="button" class="copy-btn" onclick="copyCodeBlock(this)">Copy</button></div>'
                    f'<pre><code class="language-{lang_label}">{escaped_code}</code></pre></div>'
                )
            continue

        # Headings
        heading_match = HEADING_RE.match(line)
        if heading_match:
            level = len(heading_match.group(1))
            raw_heading = heading_match.group(2).strip()
            heading_slug = unique_slug(raw_heading)
            plain_label = re.sub(r"`([^`]*)`", r"\1", raw_heading)
            plain_label = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", plain_label)
            if 2 <= level <= 3:
                toc_entries.append((level, heading_slug, plain_label))
            rendered_heading = render_inline_markdown(raw_heading, link_rewriter=link_rewriter)
            blocks.append(
                f'<h{level} id="{html.escape(heading_slug)}">'
                f'<a class="anchor-link" href="#{html.escape(heading_slug)}">#</a>{rendered_heading}'
                f"</h{level}>"
            )
            line_idx += 1
            continue

        # Horizontal rule
        if HR_RE.match(line):
            blocks.append("<hr>")
            line_idx += 1
            continue

        # Tables
        if (
            "|" in line
            and line_idx + 1 < total_lines
            and is_table_separator(lines[line_idx + 1])
        ):
            header_cells = split_table_row(line)
            alignments = parse_table_alignments(lines[line_idx + 1])
            line_idx += 2
            body_rows: list[list[str]] = []
            while line_idx < total_lines and "|" in lines[line_idx] and lines[line_idx].strip():
                body_rows.append(split_table_row(lines[line_idx]))
                line_idx += 1

            thead_html = "".join(
                f'<th style="text-align:{alignments[idx] if idx < len(alignments) else "left"}">'
                f"{render_inline_markdown(cell, link_rewriter=link_rewriter)}</th>"
                for idx, cell in enumerate(header_cells)
            )
            tbody_rows_html: list[str] = []
            for row in body_rows:
                row_html = "".join(
                    f'<td style="text-align:{alignments[idx] if idx < len(alignments) else "left"}">'
                    f"{render_inline_markdown(cell, link_rewriter=link_rewriter)}</td>"
                    for idx, cell in enumerate(row)
                )
                tbody_rows_html.append(f"<tr>{row_html}</tr>")
            blocks.append(
                f'<div class="table-wrap"><table><thead><tr>{thead_html}</tr></thead>'
                f'<tbody>{"".join(tbody_rows_html)}</tbody></table></div>'
            )
            continue

        # Blockquotes and GitHub Alerts
        if stripped.startswith(">"):
            quote_lines: list[str] = []
            while line_idx < total_lines and lines[line_idx].strip().startswith(">"):
                quote_content = re.sub(r"^\s*>\s?", "", lines[line_idx])
                quote_lines.append(quote_content)
                line_idx += 1

            first_quote_line = quote_lines[0].strip() if quote_lines else ""
            alert_match = ALERT_HEADER_RE.match(first_quote_line)
            if alert_match:
                alert_kind = alert_match.group(1)
                css_class, label = ALERT_LABELS[alert_kind]
                remainder = [alert_match.group(2), *quote_lines[1:]]
                inner_html, _ = render_markdown_to_html(
                    "\n".join(remainder).strip(),
                    link_rewriter=link_rewriter,
                )
                blocks.append(
                    f'<div class="callout callout-{css_class}">'
                    f'<div class="callout-title">{label}</div>{inner_html}</div>'
                )
            else:
                inner_html, _ = render_markdown_to_html(
                    "\n".join(quote_lines),
                    link_rewriter=link_rewriter,
                )
                blocks.append(f"<blockquote>{inner_html}</blockquote>")
            continue

        # Lists (unordered and ordered)
        if UNORDERED_ITEM_RE.match(line) or ORDERED_ITEM_RE.match(line):
            is_ordered = bool(ORDERED_ITEM_RE.match(line))
            list_tag = "ol" if is_ordered else "ul"
            item_pattern = ORDERED_ITEM_RE if is_ordered else UNORDERED_ITEM_RE
            items_html: list[str] = []
            while line_idx < total_lines:
                current_line = lines[line_idx]
                item_match = item_pattern.match(current_line)
                if not item_match:
                    break
                item_lines = [item_match.group(1)]
                line_idx += 1
                while line_idx < total_lines:
                    continuation = lines[line_idx]
                    if (
                        not continuation.strip()
                        or item_pattern.match(continuation)
                        or HEADING_RE.match(continuation)
                        or FENCE_RE.match(continuation)
                        or HR_RE.match(continuation)
                    ):
                        break
                    if continuation.startswith(("  ", "\t")):
                        item_lines.append(continuation.strip())
                        line_idx += 1
                    else:
                        break
                joined_item = " ".join(item_lines).strip()
                checkbox_prefix = ""
                if joined_item.startswith("[ ] "):
                    checkbox_prefix = '<span class="task-box" aria-hidden="true">&#9744;</span> '
                    joined_item = joined_item[4:]
                elif joined_item.startswith(("[x] ", "[X] ")):
                    checkbox_prefix = '<span class="task-box checked" aria-hidden="true">&#9745;</span> '
                    joined_item = joined_item[4:]
                rendered_item = render_inline_markdown(joined_item, link_rewriter=link_rewriter)
                items_html.append(f"<li>{checkbox_prefix}{rendered_item}</li>")
            blocks.append(f"<{list_tag}>{''.join(items_html)}</{list_tag}>")
            continue

        # Paragraph
        para_lines = [stripped]
        line_idx += 1
        while line_idx < total_lines:
            next_line = lines[line_idx]
            next_stripped = next_line.strip()
            if (
                not next_stripped
                or HEADING_RE.match(next_line)
                or FENCE_RE.match(next_line)
                or HR_RE.match(next_line)
                or next_stripped.startswith((">", "<!--"))
                or UNORDERED_ITEM_RE.match(next_line)
                or ORDERED_ITEM_RE.match(next_line)
                or ("|" in next_line and line_idx + 1 < total_lines and is_table_separator(lines[line_idx + 1]))
            ):
                break
            para_lines.append(next_stripped)
            line_idx += 1
        para_html = render_inline_markdown(" ".join(para_lines), link_rewriter=link_rewriter)
        blocks.append(f"<p>{para_html}</p>")

    return "\n".join(blocks), toc_entries


def asset_prefix_for(out_rel_path: str) -> str:
    """Computes relative prefix (e.g. `../../`) back to site root."""
    depth = out_rel_path.count("/")
    return "../" * depth


def build_page_html(
    page: DocPage,
    version: str,
    user_page_map: dict[Path, str],
    docs_root: Path,
    repo_root: Path,
) -> str:
    """Renders a complete standalone HTML page for `page`."""
    prefix = asset_prefix_for(page.out_rel_path)

    def rewrite(target: str) -> str:
        return rewrite_link_target(
            target,
            page.source_path,
            docs_root,
            repo_root,
            user_page_map,
            page.out_rel_path,
            version,
        )

    article_html, toc_entries = render_markdown_to_html(page.markdown_text, link_rewriter=rewrite)

    toc_html = ""
    if toc_entries:
        toc_items = "".join(
            f'<li class="toc-lvl-{level}"><a href="#{html.escape(slug, quote=True)}">{html.escape(label)}</a></li>'
            for level, slug, label in toc_entries
        )
        toc_html = (
            '<aside class="toc-panel">'
            '<div class="toc-title">On This Page</div>'
            f"<ul>{toc_items}</ul></aside>"
        )

    section_name = SECTION_TITLES.get(page.section, page.section.title())
    root_href = f"{prefix}index.html"
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{html.escape(page.title)} — Solix v{html.escape(version)} Docs</title>
  <link rel="stylesheet" href="{prefix}assets/style.css">
</head>
<body data-root-prefix="{html.escape(prefix, quote=True)}" data-current-path="{html.escape(page.out_rel_path, quote=True)}" data-current-section="{html.escape(page.section, quote=True)}">
  <header class="topbar">
    <div class="topbar-left">
      <button type="button" class="menu-toggle" onclick="toggleSidebar()" aria-label="Toggle navigation">&#9776;</button>
      <a class="brand" href="{html.escape(root_href, quote=True)}">
        <span class="brand-badge">Solix</span>
        <span class="brand-title">Documentation</span>
      </a>
      <span class="version-pill">v{html.escape(version)}</span>
    </div>
    <div class="search-wrap">
      <input id="wiki-search-input" type="search" placeholder="Search docs, keywords, errors (press /)..." autocomplete="off" aria-label="Search Solix documentation">
      <div id="wiki-search-results" class="search-results" hidden></div>
    </div>
    <div class="topbar-actions">
      <button type="button" id="theme-toggle" class="theme-btn" onclick="toggleTheme()" title="Toggle theme">Theme</button>
    </div>
  </header>

  <div class="layout">
    <nav id="sidebar" class="sidebar" aria-label="Documentation Navigation">
      <noscript><a class="nav-link" href="{html.escape(root_href, quote=True)}">Documentation Home</a></noscript>
    </nav>
    <main class="main-content">
      <div class="breadcrumb">
        <a href="{html.escape(root_href, quote=True)}">Solix v{html.escape(version)}</a>
        <span>/</span>
        <span>{html.escape(section_name)}</span>
        <span>/</span>
        <span>{html.escape(page.title)}</span>
      </div>
      <article class="doc-article">
        {article_html}
      </article>
      <footer class="doc-footer">
        <span>Offline documentation bundled for Solix toolchain <strong>v{html.escape(version)}</strong></span>
        <span>Source: <code>{html.escape(page.repo_rel_path)}</code></span>
      </footer>
    </main>
    {toc_html}
  </div>

  <script src="{prefix}assets/search_index.js"></script>
  <script src="{prefix}assets/app.js"></script>
</body>
</html>
"""


SITE_CSS = """\
:root {
  --bg: #0d1117;
  --surface: #161b22;
  --surface-alt: #1f242c;
  --border: #30363d;
  --text: #e6edf3;
  --muted: #8b949e;
  --accent: #58a6ff;
  --accent-soft: rgba(88, 166, 255, 0.14);
  --code-bg: #12171e;
  --note: #388bfd;
  --tip: #3fb950;
  --important: #a371f7;
  --warning: #d29922;
  --caution: #f85149;
}

body.light-theme {
  --bg: #ffffff;
  --surface: #f6f8fa;
  --surface-alt: #eef2f6;
  --border: #d0d7de;
  --text: #1f2328;
  --muted: #656d76;
  --accent: #0969da;
  --accent-soft: rgba(9, 105, 218, 0.12);
  --code-bg: #f3f5f7;
}

* { box-sizing: border-box; }

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.65;
}

a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }

.topbar {
  position: sticky;
  top: 0;
  z-index: 40;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 20px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
}

.topbar-left { display: flex; align-items: center; gap: 10px; }
.brand { display: flex; align-items: center; gap: 8px; color: var(--text); font-weight: 700; }
.brand:hover { text-decoration: none; }
.brand-badge {
  background: var(--accent);
  color: #ffffff;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 0.85rem;
  letter-spacing: 0.02em;
}
.version-pill {
  font-size: 0.78rem;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface-alt);
  color: var(--muted);
}

.search-wrap { position: relative; flex: 1; max-width: 460px; }
#wiki-search-input {
  width: 100%;
  padding: 7px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
  font-size: 0.92rem;
}
.search-results {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  max-height: 420px;
  overflow-y: auto;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
  z-index: 60;
}
.search-result-item {
  display: block;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  color: var(--text);
}
.search-result-item:hover { background: var(--accent-soft); text-decoration: none; }
.search-result-title { font-weight: 600; color: var(--accent); }
.search-result-meta { font-size: 0.78rem; color: var(--muted); }
.search-result-snippet { font-size: 0.84rem; color: var(--muted); margin-top: 2px; }

.menu-toggle {
  display: none;
  background: var(--surface-alt);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 4px 10px;
  cursor: pointer;
}

.theme-btn {
  background: var(--surface-alt);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 5px 12px;
  cursor: pointer;
  font-size: 0.85rem;
}

.layout {
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr) 230px;
  min-height: calc(100vh - 53px);
}

.sidebar {
  background: var(--surface);
  border-right: 1px solid var(--border);
  padding: 16px 14px;
  overflow-y: auto;
  max-height: calc(100vh - 53px);
  position: sticky;
  top: 53px;
}

.nav-group { margin-bottom: 10px; }
.nav-group summary {
  cursor: pointer;
  font-weight: 600;
  font-size: 0.86rem;
  color: var(--text);
  padding: 4px 6px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.nav-group .count {
  font-size: 0.75rem;
  color: var(--muted);
  background: var(--surface-alt);
  padding: 0 6px;
  border-radius: 999px;
}
.nav-group ul { list-style: none; margin: 4px 0 8px 8px; padding: 0; }
.nav-link {
  display: block;
  padding: 4px 8px;
  border-radius: 5px;
  font-size: 0.86rem;
  color: var(--muted);
}
.nav-link:hover { background: var(--surface-alt); color: var(--text); text-decoration: none; }
.nav-link.active { background: var(--accent-soft); color: var(--accent); font-weight: 600; }

.main-content {
  padding: 28px 42px;
  max-width: 920px;
  width: 100%;
  margin: 0 auto;
}

.breadcrumb {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 0.84rem;
  color: var(--muted);
  margin-bottom: 18px;
}

.doc-article h1, .doc-article h2, .doc-article h3, .doc-article h4 {
  margin-top: 1.5em;
  margin-bottom: 0.55em;
  line-height: 1.3;
}
.doc-article h1 { margin-top: 0; font-size: 2rem; border-bottom: 1px solid var(--border); padding-bottom: 0.35em; }
.doc-article h2 { font-size: 1.45rem; border-bottom: 1px solid var(--border); padding-bottom: 0.25em; }
.anchor-link { color: var(--muted); opacity: 0; margin-right: 6px; font-weight: 400; }
h1:hover .anchor-link, h2:hover .anchor-link, h3:hover .anchor-link { opacity: 1; }

code {
  background: var(--surface-alt);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 0.88em;
}

.code-card {
  margin: 14px 0;
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--code-bg);
}
.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 5px 12px;
  background: var(--surface-alt);
  border-bottom: 1px solid var(--border);
  font-size: 0.76rem;
  color: var(--muted);
  text-transform: uppercase;
}
.copy-btn {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 4px;
  padding: 1px 8px;
  cursor: pointer;
  font-size: 0.75rem;
}
.code-card pre {
  margin: 0;
  padding: 14px;
  overflow-x: auto;
}
.code-card pre code { background: transparent; padding: 0; font-size: 0.88rem; }

.table-wrap { overflow-x: auto; margin: 14px 0; }
table { width: 100%; border-collapse: collapse; }
th, td { border: 1px solid var(--border); padding: 8px 12px; font-size: 0.92rem; }
th { background: var(--surface-alt); font-weight: 600; }

blockquote {
  margin: 14px 0;
  padding: 4px 16px;
  border-left: 3px solid var(--border);
  color: var(--muted);
}

.callout {
  margin: 14px 0;
  padding: 12px 16px;
  border-radius: 6px;
  border-left: 4px solid var(--note);
  background: var(--surface);
}
.callout-title { font-weight: 700; margin-bottom: 4px; font-size: 0.9rem; }
.callout-tip { border-left-color: var(--tip); }
.callout-important { border-left-color: var(--important); }
.callout-warning { border-left-color: var(--warning); }
.callout-caution { border-left-color: var(--caution); }

.toc-panel {
  padding: 24px 16px;
  border-left: 1px solid var(--border);
  position: sticky;
  top: 53px;
  max-height: calc(100vh - 53px);
  overflow-y: auto;
  font-size: 0.84rem;
}
.toc-title { font-weight: 700; margin-bottom: 8px; color: var(--text); }
.toc-panel ul { list-style: none; padding: 0; margin: 0; }
.toc-panel li { margin: 4px 0; }
.toc-lvl-3 { padding-left: 12px; }

.doc-footer {
  margin-top: 42px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 0.82rem;
  color: var(--muted);
}

@media (max-width: 1120px) {
  .layout { grid-template-columns: 260px minmax(0, 1fr); }
  .toc-panel { display: none; }
}

@media (max-width: 820px) {
  .menu-toggle { display: inline-block; }
  .layout { grid-template-columns: minmax(0, 1fr); }
  .sidebar { display: none; position: fixed; left: 0; width: 280px; z-index: 50; }
  .sidebar.open { display: block; }
  .main-content { padding: 20px; }
}
"""

SITE_JS = """\
(function () {
  var rootPrefix = (document.body && document.body.getAttribute("data-root-prefix")) || "";
  var searchInput = document.getElementById("wiki-search-input");
  var resultsBox = document.getElementById("wiki-search-results");

  try {
    if (window.localStorage && window.localStorage.getItem("solix_docs_theme") === "light") {
      document.body.classList.add("light-theme");
    }
  } catch (storageError) {
    /* file:// storage may be restricted in hardened browser profiles */
  }

  window.toggleTheme = function () {
    document.body.classList.toggle("light-theme");
    try {
      if (window.localStorage) {
        var mode = document.body.classList.contains("light-theme") ? "light" : "dark";
        window.localStorage.setItem("solix_docs_theme", mode);
      }
    } catch (storageError) {
      /* ignore */
    }
  };

  window.toggleSidebar = function () {
    var sidebar = document.getElementById("sidebar");
    if (sidebar) {
      sidebar.classList.toggle("open");
    }
  };

  window.copyCodeBlock = function (button) {
    var card = button.closest(".code-card");
    var codeElement = card ? card.querySelector("pre code") : null;
    if (!codeElement || !navigator.clipboard) {
      return;
    }
    navigator.clipboard.writeText(codeElement.textContent || "").then(function () {
      button.textContent = "Copied";
      setTimeout(function () {
        button.textContent = "Copy";
      }, 1400);
    });
  };

  function escapeHtml(rawValue) {
    return String(rawValue)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function runOfflineSearch(rawQuery) {
    if (!resultsBox) {
      return;
    }
    var trimmed = (rawQuery || "").trim().toLowerCase();
    if (!trimmed) {
      resultsBox.hidden = true;
      resultsBox.innerHTML = "";
      return;
    }

    var terms = trimmed.split(/\\s+/).filter(Boolean);
    var entries = Array.isArray(window.SOLIX_DOCS_INDEX) ? window.SOLIX_DOCS_INDEX : [];
    var scored = [];

    for (var entryIdx = 0; entryIdx < entries.length; entryIdx += 1) {
      var entry = entries[entryIdx];
      var titleLower = (entry.title || "").toLowerCase();
      var headingsLower = (entry.headings || []).join(" ").toLowerCase();
      var excerptLower = (entry.excerpt || "").toLowerCase();
      var pathLower = (entry.path || "").toLowerCase();
      var score = 0;
      var matchedAll = true;

      for (var termIdx = 0; termIdx < terms.length; termIdx += 1) {
        var term = terms[termIdx];
        var termScore = 0;
        if (titleLower.indexOf(term) !== -1) {
          termScore += 12;
        }
        if (pathLower.indexOf(term) !== -1) {
          termScore += 6;
        }
        if (headingsLower.indexOf(term) !== -1) {
          termScore += 5;
        }
        if (excerptLower.indexOf(term) !== -1) {
          termScore += 2;
        }
        if (termScore === 0) {
          matchedAll = false;
          break;
        }
        score += termScore;
      }

      if (matchedAll && score > 0) {
        scored.push({ score: score, entry: entry });
      }
    }

    scored.sort(function (left, right) {
      return right.score - left.score;
    });

    if (scored.length === 0) {
      resultsBox.innerHTML = '<div class="search-result-item"><div class="search-result-meta">No matching documentation pages found.</div></div>';
      resultsBox.hidden = false;
      return;
    }

    var topMatches = scored.slice(0, 14);
    var htmlParts = [];
    for (var itemIdx = 0; itemIdx < topMatches.length; itemIdx += 1) {
      var item = topMatches[itemIdx].entry;
      var href = rootPrefix + item.path;
      htmlParts.push(
        '<a class="search-result-item" href="' + escapeHtml(href) + '">' +
        '<div class="search-result-title">' + escapeHtml(item.title) + '</div>' +
        '<div class="search-result-meta">' + escapeHtml(item.sectionLabel) + ' &bull; ' + escapeHtml(item.path) + '</div>' +
        '<div class="search-result-snippet">' + escapeHtml((item.excerpt || "").slice(0, 140)) + '...</div>' +
        '</a>'
      );
    }
    resultsBox.innerHTML = htmlParts.join("");
    resultsBox.hidden = false;
  }

  function renderSidebar() {
    var sidebar = document.getElementById("sidebar");
    if (!sidebar) {
      return;
    }
    var currentPath = (document.body && document.body.getAttribute("data-current-path")) || "";
    var entries = Array.isArray(window.SOLIX_DOCS_INDEX) ? window.SOLIX_DOCS_INDEX : [];
    var sectionOrder = ["home", "guide", "wiki", "features", "spec", "architecture", "overview", "api"];
    var grouped = {};
    for (var idx = 0; idx < entries.length; idx += 1) {
      var entry = entries[idx];
      var secKey = entry.section || "other";
      if (!grouped[secKey]) {
        grouped[secKey] = { label: entry.sectionLabel || secKey, items: [] };
      }
      grouped[secKey].items.push(entry);
    }

    var navHtml = [];
    for (var sIdx = 0; sIdx < sectionOrder.length; sIdx += 1) {
      var sec = sectionOrder[sIdx];
      var bucket = grouped[sec];
      if (!bucket || bucket.items.length === 0) {
        continue;
      }
      bucket.items.sort(function (left, right) {
        var leftIdx = left.path.slice(-10) === "index.html" ? 0 : 1;
        var rightIdx = right.path.slice(-10) === "index.html" ? 0 : 1;
        if (leftIdx !== rightIdx) {
          return leftIdx - rightIdx;
        }
        return left.path < right.path ? -1 : left.path > right.path ? 1 : 0;
      });
      var hasActive = false;
      var liParts = [];
      for (var itemIdx = 0; itemIdx < bucket.items.length; itemIdx += 1) {
        var item = bucket.items[itemIdx];
        var active = item.path === currentPath;
        if (active) {
          hasActive = true;
        }
        var cls = active ? "nav-link active" : "nav-link";
        liParts.push(
          '<li><a class="' + cls + '" href="' + escapeHtml(rootPrefix + item.path) + '">' +
          escapeHtml(item.title) + '</a></li>'
        );
      }
      var openAttr = (hasActive || sec === "home" || sec === "guide" || sec === "wiki") ? " open" : "";
      navHtml.push(
        '<details class="nav-group"' + openAttr + '>' +
        '<summary>' + escapeHtml(bucket.label) + ' <span class="count">' + bucket.items.length + '</span></summary>' +
        '<ul>' + liParts.join("") + '</ul></details>'
      );
    }
    sidebar.innerHTML = navHtml.join("");
  }

  renderSidebar();

  if (searchInput) {
    searchInput.addEventListener("input", function () {
      runOfflineSearch(searchInput.value);
    });

    document.addEventListener("keydown", function (event) {
      if (event.key === "/" && document.activeElement !== searchInput) {
        event.preventDefault();
        searchInput.focus();
      } else if (event.key === "Escape" && resultsBox) {
        resultsBox.hidden = true;
      }
    });

    document.addEventListener("click", function (event) {
      if (resultsBox && !resultsBox.contains(event.target) && event.target !== searchInput) {
        resultsBox.hidden = true;
      }
    });

    try {
      var params = new URLSearchParams(window.location.search || "");
      var initialQuery = params.get("q") || params.get("topic") || "";
      if (initialQuery) {
        searchInput.value = initialQuery;
        runOfflineSearch(initialQuery);
      }
    } catch (urlError) {
      /* ignore */
    }
  }
})();
"""


def build_site(
    repo_root: Path,
    output_dir: Path,
    *,
    docs_dir: Path | None = None,
    version_override: str | None = None,
    extra_api_dirs: list[tuple[str, Path]] | None = None,
) -> dict[str, int | str]:
    """Builds the offline static HTML documentation portal into `output_dir`."""
    docs_root = (docs_dir or (repo_root / "docs")).resolve()
    version = version_override or read_workspace_version(repo_root)

    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    assets_out = output_dir / "assets"
    assets_out.mkdir(parents=True, exist_ok=True)

    sources = collect_user_doc_sources(docs_root)
    pages: list[DocPage] = []
    user_page_map: dict[Path, str] = {}

    for source_path, section in sources:
        docs_rel = source_path.relative_to(docs_root)
        out_rel = markdown_rel_to_html_rel(docs_rel)
        markdown_text = source_path.read_text(encoding="utf-8")
        title, headings = extract_title_and_headings(markdown_text, source_path.stem)
        excerpt = extract_plain_excerpt(markdown_text)
        try:
            repo_rel = source_path.relative_to(repo_root).as_posix()
        except ValueError:
            repo_rel = f"docs/{docs_rel.as_posix()}"

        page = DocPage(
            source_path=source_path.resolve(),
            repo_rel_path=repo_rel,
            out_rel_path=out_rel,
            section=section,
            title=title,
            headings=headings,
            plain_excerpt=excerpt,
            markdown_text=markdown_text,
        )
        pages.append(page)
        user_page_map[source_path.resolve()] = out_rel

    # Copy any non-markdown images/assets inside user docs dirs
    copied_static = 0
    for asset_file in sorted(docs_root.rglob("*")):
        if not asset_file.is_file() or asset_file.suffix.lower() == ".md":
            continue
        rel = asset_file.relative_to(docs_root)
        if "internals" in rel.parts:
            continue
        if rel.parts and rel.parts[0] in USER_DOC_DIRS:
            dest_file = output_dir / rel
            dest_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(asset_file, dest_file)
            copied_static += 1

    # Optionally integrate pre-built API doc directories (e.g., solixlib / raylib HTML from `solix doc`)
    api_files_copied = 0
    for api_name, api_src_dir in extra_api_dirs or []:
        if not api_src_dir.is_dir():
            continue
        dest_api_dir = output_dir / "api" / api_name
        shutil.copytree(api_src_dir, dest_api_dir, dirs_exist_ok=True)
        index_html = dest_api_dir / "index.html"
        if index_html.is_file():
            pages.append(
                DocPage(
                    source_path=index_html,
                    repo_rel_path=f"api/{api_name}/index.html",
                    out_rel_path=f"api/{api_name}/index.html",
                    section="api",
                    title=f"{api_name} API Reference",
                    headings=[f"{api_name} API Reference"],
                    plain_excerpt=f"Generated Solix API documentation for package {api_name}.",
                    markdown_text="",
                )
            )
        api_files_copied += sum(1 for item in dest_api_dir.rglob("*") if item.is_file())

    # Write shared CSS & JS assets
    (assets_out / "style.css").write_text(SITE_CSS, encoding="utf-8")
    (assets_out / "app.js").write_text(SITE_JS, encoding="utf-8")

    # Write offline search index (`window.SOLIX_DOCS_INDEX`) for CORS-free `file://` search
    index_payload = [
        {
            "title": page.title,
            "path": page.out_rel_path,
            "section": page.section,
            "sectionLabel": SECTION_TITLES.get(page.section, page.section.title()),
            "headings": page.headings,
            "excerpt": page.plain_excerpt,
        }
        for page in sorted(pages, key=lambda item: item.out_rel_path)
    ]
    search_js = "window.SOLIX_DOCS_INDEX = " + json.dumps(index_payload, ensure_ascii=False) + ";\n"
    (assets_out / "search_index.js").write_text(search_js, encoding="utf-8")

    # Render Markdown pages to HTML (preserve prebuilt `api/` HTML pages untouched)
    rendered_count = 0
    for page in pages:
        if page.section == "api":
            continue
        out_path = output_dir / page.out_rel_path
        out_path.parent.mkdir(parents=True, exist_ok=True)
        rendered = build_page_html(page, version, user_page_map, docs_root, repo_root)
        out_path.write_text(rendered, encoding="utf-8")
        rendered_count += 1

    # Write build manifest metadata
    meta_payload = {
        "version": version,
        "page_count": rendered_count,
        "copied_static": copied_static,
        "api_files_copied": api_files_copied,
    }
    (output_dir / "manifest.json").write_text(
        json.dumps(meta_payload, indent=2) + "\n",
        encoding="utf-8",
    )

    return meta_payload


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parses CLI arguments for the static documentation portal generator."""
    parser = argparse.ArgumentParser(
        description="Build offline static HTML documentation portal for `solix wiki`."
    )
    parser.add_argument(
        "--docs-dir",
        type=Path,
        default=None,
        help="Path to docs directory [default: <repo>/docs]",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output static site directory [default: <repo>/crates/solix-toolchain/assets/docs]",
    )
    parser.add_argument(
        "--version",
        type=str,
        default=None,
        help="Override Solix version string baked into HTML header",
    )
    parser.add_argument(
        "--api-dir",
        action="append",
        default=[],
        help="Optional prebuilt API doc directory in `name=path` format (repeatable)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """Entry point for CLI execution."""
    args = parse_args(argv)
    repo_root = find_repo_root(Path(__file__).resolve())
    output_dir = (
        args.output
        if args.output is not None
        else repo_root / "crates" / "solix-toolchain" / "assets" / "docs"
    )
    extra_api_dirs: list[tuple[str, Path]] = []
    for raw_spec in args.api_dir:
        if "=" not in raw_spec:
            print(f"[build_docs_site] Ignoring invalid --api-dir '{raw_spec}' (expected name=path)", file=sys.stderr)
            return 2
        api_name, api_path_str = raw_spec.split("=", 1)
        extra_api_dirs.append((api_name.strip(), Path(api_path_str.strip())))

    summary = build_site(
        repo_root,
        output_dir,
        docs_dir=args.docs_dir,
        version_override=args.version,
        extra_api_dirs=extra_api_dirs,
    )
    print(
        f"[build_docs_site] Built {summary['page_count']} HTML page(s) for Solix v{summary['version']} "
        f"into '{output_dir}'."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
