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
one_statement_code() { printf '%s\n' "$IMPORTS_LIBRARY" "$FUNCTIONS_LIBRARY"; }
python3 - "$(one_statement_code)" <<'PY'
import sys, types
one_statement = types.ModuleType("lib_one_statement")
exec(compile(sys.argv[1], "one-statement", "exec"), one_statement.__dict__)
one_statement.used()
PY
