#!/usr/bin/env python3
import sys
from pathlib import Path
import solix_toolbox as tb

def main():
    profile = "debug"
    for arg in sys.argv[1:]:
        if arg in ("debug", "release", "test"):
            profile = arg
        elif arg.startswith("--profile="):
            profile = arg.split("=", 1)[1]
        elif arg == "--release":
            profile = "release"
        elif arg == "--test":
            profile = "test"
        elif arg == "--debug":
            profile = "debug"
    tb.auto_build(profile)

if __name__ == "__main__":
    main()
