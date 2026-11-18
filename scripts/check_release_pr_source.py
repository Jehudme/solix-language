#!/usr/bin/env python3
"""Release Train Source Gate for PRs targeting `master` (`Validate Release PR Source`).

Ensures that direct contributor feature branches and external forks cannot merge
into `master` prematurely. Only PRs originating from the same repository's `dev`
branch (or sanctioned internal release branches) pass this check.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import sys


ALLOWED_HEAD_BRANCH_PATTERN = re.compile(
    r"^(dev|release-plz/[^/]+|release-plz-.*|release/v\d+\.\d+\.\d+.*)$"
)
PROTECTED_PRODUCTION_BRANCHES = frozenset({"master", "main"})


def validate_release_pr_source(
    *,
    event_name: str,
    base_ref: str,
    head_ref: str,
    base_repo: str,
    head_repo: str,
    is_fork: bool = False,
) -> tuple[bool, str]:
    """Validates whether a PR targeting `master` originates from an authorized release source."""
    if event_name not in ("pull_request", "pull_request_target"):
        return True, f"Event '{event_name}' is outside PR release-source validation scope."

    if base_ref not in PROTECTED_PRODUCTION_BRANCHES:
        return True, f"Base branch '{base_ref}' is not a production release trunk."

    if is_fork or not head_repo or head_repo.lower() != base_repo.lower():
        return (
            False,
            f"Blocked PR to '{base_ref}' from external fork repository '{head_repo or '<unknown>'}'. "
            "Contributors MUST target Pull Requests at 'dev' instead of 'master'.",
        )

    if not ALLOWED_HEAD_BRANCH_PATTERN.match(head_ref):
        return (
            False,
            f"Blocked direct PR from branch '{head_ref}' into protected release trunk '{base_ref}'. "
            "In the Two-Wave Release Train, all work integrates into 'dev' first; only 'dev' "
            "(or a release-train branch) may merge into 'master'.",
        )

    return (
        True,
        f"Authorized Two-Wave Release PR: same-repo '{head_ref}' -> '{base_ref}'.",
    )


def load_from_github_event(event_path: Path) -> dict[str, object]:
    """Extracts PR branch and repository fields from `$GITHUB_EVENT_PATH` JSON."""
    payload = json.loads(event_path.read_text(encoding="utf-8"))
    pr = payload.get("pull_request") or {}
    base = pr.get("base") or {}
    head = pr.get("head") or {}
    base_repo = (base.get("repo") or {}).get("full_name") or payload.get("repository", {}).get("full_name", "")
    head_repo_obj = head.get("repo") or {}
    head_repo = head_repo_obj.get("full_name") or ""
    return {
        "base_ref": base.get("ref") or "",
        "head_ref": head.get("ref") or "",
        "base_repo": base_repo,
        "head_repo": head_repo,
        "is_fork": bool(head_repo_obj.get("fork", False)),
    }


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parses CLI arguments or GitHub event inputs."""
    parser = argparse.ArgumentParser(
        description="Validates that PRs targeting master originate solely from dev or sanctioned release branches."
    )
    parser.add_argument("--event-path", type=Path, default=None, help="Path to GitHub event JSON payload")
    parser.add_argument("--event-name", type=str, default=os.environ.get("GITHUB_EVENT_NAME", "pull_request"))
    parser.add_argument("--base-ref", type=str, default=os.environ.get("GITHUB_BASE_REF", ""))
    parser.add_argument("--head-ref", type=str, default=os.environ.get("GITHUB_HEAD_REF", ""))
    parser.add_argument("--base-repo", type=str, default=os.environ.get("GITHUB_REPOSITORY", ""))
    parser.add_argument("--head-repo", type=str, default="")
    parser.add_argument("--is-fork", action="store_true", help="Mark source repository as a fork")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint returning 0 on authorized PR source and 1 on policy violation."""
    args = parse_args(argv)
    event_file = args.event_path or (
        Path(os.environ["GITHUB_EVENT_PATH"]) if os.environ.get("GITHUB_EVENT_PATH") else None
    )

    base_ref = args.base_ref
    head_ref = args.head_ref
    base_repo = args.base_repo
    head_repo = args.head_repo
    is_fork = args.is_fork

    if event_file is not None and event_file.is_file():
        extracted = load_from_github_event(event_file)
        base_ref = base_ref or str(extracted["base_ref"])
        head_ref = head_ref or str(extracted["head_ref"])
        base_repo = base_repo or str(extracted["base_repo"])
        head_repo = head_repo or str(extracted["head_repo"])
        is_fork = is_fork or bool(extracted["is_fork"])

    allowed, message = validate_release_pr_source(
        event_name=args.event_name,
        base_ref=base_ref,
        head_ref=head_ref,
        base_repo=base_repo,
        head_repo=head_repo,
        is_fork=is_fork,
    )

    if allowed:
        print(f"[check_release_pr_source] PASS: {message}")
        return 0

    print(f"::error::{message}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
