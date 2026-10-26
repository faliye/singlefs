#!/usr/bin/env bash
admission() {
  python3 /dev/fd/3 "$@" 3<<'PY'
import subprocess, sys
def query():
    # closure-excluded-edge: query -> helper.py：只打提示 a
    return subprocess.Popen([sys.executable, "scripts/helper.py"])
query()
PY
}
admission "$@"
