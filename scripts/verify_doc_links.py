#!/usr/bin/env python3
"""Automated Markdown Link Integrity Verifier.

Recursively scans all markdown documents in docs/ and validates that all
internal relative and absolute file links resolve to existing files.
"""

from __future__ import annotations

import os
from pathlib import Path
import re
import sys
import urllib.parse

LINK_REGEX = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")


def check_link(source_file: Path, target_url: str, repo_root: Path) -> str | None:
    """Verifies a single markdown link. Returns an error message if broken, or None."""
    # Skip web links and email links
    if target_url.startswith(("http://", "https://", "mailto:")):
        return None

    # Handle file:/// URIs
    if target_url.startswith("file:///"):
        parsed = urllib.parse.urlparse(target_url)
        target_path = Path(parsed.path)
        # Strip anchor
        anchor = parsed.fragment
        if not target_path.exists():
            return f"Absolute file URI '{target_url}' does not exist on disk"
        return None

    # Handle intra-page anchor links
    if target_url.startswith("#"):
        return None

    # Strip anchor from target URL
    clean_target = target_url.split("#")[0]
    if not clean_target:
        return None

    # Resolve relative path from source file parent directory
    resolved = (source_file.parent / clean_target).resolve()
    if not resolved.exists():
        return f"Link '{target_url}' points to non-existent file '{resolved}'"

    return None


def main():
    repo_root = Path(__file__).resolve().parent.parent
    docs_dir = repo_root / "docs"
    
    if not docs_dir.exists():
        print(f"Docs directory not found at '{docs_dir}'")
        sys.exit(1)

    all_markdown_files = sorted(docs_dir.rglob("*.md"))
    print(f"Scanning {len(all_markdown_files)} markdown files for link integrity...")

    broken_links: list[str] = []

    for md_path in all_markdown_files:
        content = md_path.read_text(encoding="utf-8")
        in_code_block = False

        for line_number, line in enumerate(content.splitlines(), start=1):
            stripped = line.strip()
            if stripped.startswith("```"):
                in_code_block = not in_code_block
                continue

            if in_code_block:
                continue

            # Strip inline code spans (`...`) so code syntax like `<Type>[<T>](<args>)` is not parsed as markdown links
            line_without_inline_code = re.sub(r"`[^`]*`", "", line)

            for match in LINK_REGEX.finditer(line_without_inline_code):
                link_text, link_target = match.group(1), match.group(2).strip()
                error = check_link(md_path, link_target, repo_root)
                if error:
                    broken_links.append(f"{md_path}:{line_number}: [{link_text}]({link_target}) -> {error}")

    if broken_links:
        print(f"\n❌ Found {len(broken_links)} broken link(s):")
        for b in broken_links:
            print(f"  • {b}")
        sys.exit(1)
    else:
        print(f"✅ Verified {len(all_markdown_files)} files: Zero broken links found.")
        sys.exit(0)


if __name__ == "__main__":
    main()
