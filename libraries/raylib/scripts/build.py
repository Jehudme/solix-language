#!/usr/bin/env python3
"""Build script for Solix Raylib package.

Automates the build workflow:
1. Validates build profile (debug, release, test)
2. Runs solix_toolbox.auto_build
"""

import sys
from pathlib import Path

scripts_dir = Path(__file__).resolve().parent
sys.path.insert(0, str(scripts_dir))

import solix_toolbox

def main():
    profile = "debug"
    for argument in sys.argv[1:]:
        if argument in ("debug", "release", "test"):
            profile = argument
        elif argument.startswith("--profile="):
            profile = argument.split("=", 1)[1]
        elif argument == "--release":
            profile = "release"
        elif argument == "--test":
            profile = "test"
        elif argument == "--debug":
            profile = "debug"

    project_root = scripts_dir.parent
    solix_toolbox.auto_build(profile=profile, project_root=project_root)

if __name__ == "__main__":
    main()
