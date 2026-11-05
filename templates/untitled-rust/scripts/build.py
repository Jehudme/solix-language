#!/usr/bin/env python3
"""Project build script invoked by `solix build`."""

from __future__ import annotations

import argparse
from pathlib import Path
import solix_toolbox as tb


def main() -> int:
    parser = argparse.ArgumentParser(description="Build Solix project")
    parser.add_argument(
        "--profile",
        choices=["debug", "release", "test"],
        default="debug",
        help="Build profile mode (debug, release, or test)",
    )
    args = parser.parse_args()

    project_root = Path(__file__).resolve().parent.parent
    tb.auto_build(profile=args.profile, project_root=project_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
