#!/usr/bin/env python3
"""Project build script invoked by `solix build`."""

import sys
from pathlib import Path
import solix_build as tb


def main() -> None:
    profile = "debug"
    for argument_item in sys.argv[1:]:
        if argument_item in ("debug", "release", "test"):
            profile = argument_item
        elif argument_item.startswith("--profile="):
            profile = argument_item.split("=", 1)[1]
        elif argument_item == "--profile" and len(sys.argv) > sys.argv.index(argument_item) + 1:
            profile = sys.argv[sys.argv.index(argument_item) + 1]
        elif argument_item == "--release":
            profile = "release"
        elif argument_item == "--test":
            profile = "test"
        elif argument_item == "--debug":
            profile = "debug"
    tb.auto_build(profile)


if __name__ == "__main__":
    tb.run_cli(main)
