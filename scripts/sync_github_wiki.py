#!/usr/bin/env python3
"""One-Way GitHub Wiki Staging Generator from Solix `docs/` Source of Truth (MS063-T10).

Reads user documentation (`docs/README.md`, `docs/guide/`, `docs/wiki/`,
`docs/spec/`, and `docs/architecture/`) directly inside the repository without
moving or deleting source files, flattens filenames cleanly for GitHub Wiki,
rewrites internal relative links, builds `Home.md` and `_Sidebar.md`, and injects
a release-version banner guiding older-version users to their Git tag's `/docs`
or offline `solix wiki` portal.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import shutil
import tomllib


MIRROR_DIRS = ("guide", "wiki", "spec", "architecture")
LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")


def read_workspace_version(repo_root: Path) -> str:
    """Reads `[workspace.package].version` from root `Cargo.toml`."""
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.is_file():
        return "0.0.0"
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    version = data.get("workspace", {}).get("package", {}).get("version")
    return str(version).strip() if version else "0.0.0"


def docs_rel_to_wiki_page_name(rel_path: Path) -> str:
    """Converts a path relative to `docs/` into a deterministic GitHub Wiki page stem."""
    posix = rel_path.as_posix()
    if posix == "README.md":
        return "Home"
    stem = posix[:-3] if posix.endswith(".md") else posix
    return stem.replace("/", "--")


def build_version_banner(version: str, source_rel_path: str) -> str:
    """Constructs the multi-version banner pointing older versions to tagged docs or `solix wiki`."""
    tag = f"v{version}"
    return (
        f"> **Documentation Snapshot:** Published for **Solix {tag}** "
        f"([repository source](https://github.com/Jehudme/solix-language/blob/{tag}/{source_rel_path})). "
        f"Pinned to an older version? Browse your exact release docs on GitHub at "
        f"[`tree/<tag>/docs`](https://github.com/Jehudme/solix-language/tags) or run **`solix wiki`** "
        f"locally to open the offline documentation bundled inside your installed toolchain.\n\n"
    )


def rewrite_markdown_for_wiki(
    markdown_text: str,
    *,
    source_file: Path,
    docs_root: Path,
    repo_root: Path,
    wiki_map: dict[Path, str],
    version: str,
) -> str:
    """Rewrites Markdown relative links inside non-fenced lines to Wiki page names or GitHub URLs."""
    output_lines: list[str] = []
    in_fence = False
    tag = f"v{version}"

    def replace_link(match: re.Match[str]) -> str:
        label = match.group(1)
        raw_target = match.group(2).strip()
        if not raw_target or raw_target.startswith(("#", "http://", "https://", "mailto:")):
            return match.group(0)
        base_target, hash_sep, fragment = raw_target.partition("#")
        anchor = f"#{fragment}" if hash_sep else ""
        if not base_target:
            return match.group(0)

        resolved = (source_file.parent / base_target).resolve()
        if resolved in wiki_map:
            return f"[{label}]({wiki_map[resolved]}{anchor})"

        try:
            repo_rel = resolved.relative_to(repo_root).as_posix()
            return f"[{label}](https://github.com/Jehudme/solix-language/blob/{tag}/{repo_rel}{anchor})"
        except ValueError:
            try:
                docs_rel = resolved.relative_to(docs_root).as_posix()
                return f"[{label}](https://github.com/Jehudme/solix-language/blob/{tag}/docs/{docs_rel}{anchor})"
            except ValueError:
                return match.group(0)

    for line in markdown_text.splitlines():
        if line.strip().startswith("```"):
            in_fence = not in_fence
            output_lines.append(line)
            continue
        if in_fence:
            output_lines.append(line)
            continue
        output_lines.append(LINK_RE.sub(replace_link, line))

    return "\n".join(output_lines) + "\n"


def stage_github_wiki(
    repo_root: Path,
    output_dir: Path,
    *,
    version_override: str | None = None,
) -> dict[str, int | str]:
    """Generates clean flattened GitHub Wiki Markdown files into `output_dir`."""
    docs_root = (repo_root / "docs").resolve()
    version = version_override or read_workspace_version(repo_root)

    if output_dir.exists():
        for existing in output_dir.iterdir():
            if existing.name == ".git":
                continue
            if existing.is_dir():
                shutil.rmtree(existing)
            else:
                existing.unlink()
    output_dir.mkdir(parents=True, exist_ok=True)

    source_files: list[Path] = []
    if (docs_root / "README.md").is_file():
        source_files.append((docs_root / "README.md").resolve())
    for subdir in MIRROR_DIRS:
        candidate_dir = docs_root / subdir
        if candidate_dir.is_dir():
            for md_file in sorted(candidate_dir.rglob("*.md")):
                source_files.append(md_file.resolve())

    wiki_map: dict[Path, str] = {}
    for md_file in source_files:
        wiki_map[md_file] = docs_rel_to_wiki_page_name(md_file.relative_to(docs_root))

    sidebar_entries: list[str] = [
        f"## Solix v{version}",
        "",
        "* [Documentation Home](Home)",
        "* **Use `solix wiki` offline**",
        "",
        "### Sections",
    ]
    written_count = 0

    for md_file in source_files:
        page_name = wiki_map[md_file]
        repo_rel = md_file.relative_to(repo_root).as_posix()
        banner = build_version_banner(version, repo_rel)
        body = rewrite_markdown_for_wiki(
            md_file.read_text(encoding="utf-8"),
            source_file=md_file,
            docs_root=docs_root,
            repo_root=repo_root,
            wiki_map=wiki_map,
            version=version,
        )
        (output_dir / f"{page_name}.md").write_text(banner + body, encoding="utf-8")
        written_count += 1
        if page_name != "Home":
            parts = page_name.split("--")
            indent = "  " * max(0, len(parts) - 2)
            label = parts[-1].replace("_", " ").replace("-", " ")
            sidebar_entries.append(f"{indent}* [{label}]({page_name})")

    (output_dir / "_Sidebar.md").write_text("\n".join(sidebar_entries) + "\n", encoding="utf-8")
    return {"version": version, "pages": written_count}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parses CLI flags for GitHub Wiki generation."""
    parser = argparse.ArgumentParser(description="Stage one-way GitHub Wiki files from docs/.")
    parser.add_argument("--output", type=Path, required=True, help="Target wiki clone/output folder")
    parser.add_argument("--version", type=str, default=None, help="Override Solix release version banner")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint."""
    args = parse_args(argv)
    repo_root = Path(__file__).resolve().parent.parent
    result = stage_github_wiki(repo_root, args.output, version_override=args.version)
    print(f"[sync_github_wiki] Staged {result['pages']} wiki page(s) for v{result['version']} into '{args.output}'.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
