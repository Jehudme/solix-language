#!/usr/bin/env python3
"""Build entrypoint for Solix Raylib library.

Delegates directly to scripts/build.py run_pipeline().
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

# Add scripts directory to module search path
SCRIPTS_DIR = Path(__file__).resolve().parent / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import build as raylib_build


def main() -> None:
    selected_profile = os.environ.get("SOLIX_PROFILE", "").strip() or "release"
    arguments = sys.argv[1:]
    index = 0
    while index < len(arguments):
        argument_str = arguments[index]
        if argument_str in ("debug", "release", "test"):
            selected_profile = argument_str
        elif argument_str.startswith("--profile="):
            selected_profile = argument_str.split("=", 1)[1]
        elif argument_str == "--profile" and index + 1 < len(arguments):
            selected_profile = arguments[index + 1]
            index += 1
        elif argument_str.startswith("--target="):
            os.environ["SOLIX_TARGET_TRIPLE"] = argument_str.split("=", 1)[1]
        elif argument_str == "--target" and index + 1 < len(arguments):
            os.environ["SOLIX_TARGET_TRIPLE"] = arguments[index + 1]
            index += 1
        elif argument_str == "--strict-cross":
            os.environ["SOLIX_STRICT_CROSS"] = "1"
        elif argument_str == "--release":
            selected_profile = "release"
        elif argument_str == "--test":
            selected_profile = "test"
        elif argument_str == "--debug":
            selected_profile = "debug"
        index += 1

    project_root = Path(__file__).resolve().parent
    raylib_build.run_pipeline(build_profile=selected_profile, root_path=project_root)


if __name__ == "__main__":
    main()
