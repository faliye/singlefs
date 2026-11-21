#!/usr/bin/env bash
HANDED_LIBRARY="$(cat <<'PY_HANDED'
import subprocess


def used():
    return subprocess.run(["bash", "s/helper-used.sh"], check=False)


def unused():
    return subprocess.run(["bash", "s/helper-unused.sh"], check=False)
PY_HANDED
)"
python3 - "$HANDED_LIBRARY" <<'PY'
import sys, types
handed = types.ModuleType("lib_handed_out")
exec(compile(sys.argv[1], "handed-out", "exec"), handed.__dict__)
sys.modules["lib_handed_out"] = handed
handed.used()
PY
