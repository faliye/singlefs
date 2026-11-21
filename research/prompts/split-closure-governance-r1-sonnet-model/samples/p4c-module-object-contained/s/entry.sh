#!/usr/bin/env bash
CONTAINED_LIBRARY="$(cat <<'PY_CONTAINED'
import subprocess


def used():
    return subprocess.run(["bash", "s/helper-used.sh"], check=False)


def unused():
    return subprocess.run(["bash", "s/helper-unused.sh"], check=False)
PY_CONTAINED
)"
python3 - "$CONTAINED_LIBRARY" <<'PY'
import sys, types
contained = types.ModuleType("lib_contained")
exec(compile(sys.argv[1], "contained", "exec"), contained.__dict__)
contained.used()
PY
