#!/usr/bin/env python3
"""Project build script invoked by `solix build`."""

import os
import sys
import solix_build as tb


def verify(ctx: tb.BuildContext) -> None:
    """Pre-flight verification hook run before native or Solix compilation."""
    _ = ctx


def main() -> None:
    profile = os.environ.get("SOLIX_PROFILE", "").strip() or "release"
    verify_only = os.environ.get("SOLIX_VERIFY_ONLY", "").strip().lower() in ("1", "true", "yes")
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
            raw_target = argument_item.split("=", 1)[1]
            os.environ["SOLIX_TARGET_TRIPLE"] = (
                tb.get_host_target_triple() if raw_target.lower() == "host" else raw_target
            )
        elif argument_item == "--target" and index + 1 < len(arguments):
            raw_target = arguments[index + 1]
            os.environ["SOLIX_TARGET_TRIPLE"] = (
                tb.get_host_target_triple() if raw_target.lower() == "host" else raw_target
            )
            index += 1
        elif argument_item in ("--check-env", "--verify-only"):
            verify_only = True
        elif argument_item == "--release":
            profile = "release"
        elif argument_item == "--test":
            profile = "test"
        elif argument_item == "--debug":
            profile = "debug"
        index += 1
    tb.auto_build(profile, verify_fn=verify, verify_only=verify_only)


if __name__ == "__main__":
    tb.run_cli(main)
