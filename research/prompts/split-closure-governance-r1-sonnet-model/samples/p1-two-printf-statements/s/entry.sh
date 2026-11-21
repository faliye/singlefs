#!/usr/bin/env bash
IMPORTS_LIBRARY="$(cat <<'PY_IMPORTS'
import subprocess
PY_IMPORTS
)"
FUNCTIONS_LIBRARY="$(cat <<'PY_FUNCTIONS'


def used():
    return subprocess.run(["bash", "s/helper-used.sh"], check=False)
PY_FUNCTIONS
)"
two_statement_code() {
  printf '%s\n' "$IMPORTS_LIBRARY"
  printf '%s\n' "$FUNCTIONS_LIBRARY"
}
python3 - "$(two_statement_code)" <<'PY'
import sys, types
two_statements = types.ModuleType("lib_two_statements")
exec(compile(sys.argv[1], "two-statements", "exec"), two_statements.__dict__)
two_statements.used()
PY
