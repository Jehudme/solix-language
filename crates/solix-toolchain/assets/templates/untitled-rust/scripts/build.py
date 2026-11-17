#!/usr/bin/env python3
"""Project build script invoked by `solix build`."""

import os
import sys
import solix_build as tb


def main() -> None:
    profile = os.environ.get("SOLIX_PROFILE", "").strip() or "release"
    arguments = sys.argv[1:]
    index = 0
    while index < len(arguments):
        argument_item = arguments[index]
        if argument_item in ("debug", "release", "test"):
            profile = argument_item
        elif argument_item.startswith("--profile="):
            profile = argument_item.split("=", 1)[1]
        elif argument_item == "--profile" and index + 1 < len(arguments):
            profile = arguments[index + 1]
            index += 1
        elif argument_item.startswith("--target="):
            os.environ["SOLIX_TARGET_TRIPLE"] = argument_item.split("=", 1)[1]
        elif argument_item == "--target" and index + 1 < len(arguments):
            os.environ["SOLIX_TARGET_TRIPLE"] = arguments[index + 1]
            index += 1
        elif argument_item == "--strict-cross":
            os.environ["SOLIX_STRICT_CROSS"] = "1"
        elif argument_item == "--release":
            profile = "release"
        elif argument_item == "--test":
            profile = "test"
        elif argument_item == "--debug":
            profile = "debug"
        index += 1
    tb.auto_build(profile)


if __name__ == "__main__":
    tb.run_cli(main)
