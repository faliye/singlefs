#!/usr/bin/env bash
FILE_LIBRARY=""
IFS= read -r -d '' FILE_LIBRARY < "$1" || true
FILE_LIBRARY="$FILE_LIBRARY" python3 - <<'PY'
import os, types
file_library = types.ModuleType("lib_read_file_library")
exec(compile(os.environ["FILE_LIBRARY"], "read-file-library", "exec"), file_library.__dict__)
file_library.used()
PY
