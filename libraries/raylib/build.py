#!/usr/bin/env python3
"""Build entrypoint for Solix Raylib library.

Delegates directly to scripts/build.py run_pipeline().
"""

from __future__ import annotations

import sys
from pathlib import Path

# Add scripts directory to module search path
SCRIPTS_DIR = Path(__file__).resolve().parent / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import build as raylib_build


def main() -> None:
    selected_profile = "debug"
    for argument_str in sys.argv[1:]:
        if argument_str in ("debug", "release", "test"):
            selected_profile = argument_str
        elif argument_str.startswith("--profile="):
            selected_profile = argument_str.split("=", 1)[1]
        elif argument_str == "--release":
            selected_profile = "release"
        elif argument_str == "--test":
            selected_profile = "test"
        elif argument_str == "--debug":
            selected_profile = "debug"

    project_root = Path(__file__).resolve().parent
    raylib_build.run_pipeline(build_profile=selected_profile, root_path=project_root)


if __name__ == "__main__":
    main()
