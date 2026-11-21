#!/usr/bin/env python3
"""refresh_probe.sh 交给 trim_slot_target 的 RUN_BUILD_DIRECTORY：不删任何东西，只打一行它被叫去删哪一份，退 0。"""
import sys

print("REMOVE_STUB " + " ".join(sys.argv[1:]))
