#!/usr/bin/env bash
# needed_files_strace.sh <树> <输出目录>：在树里照 HostLookup 的调法跑一次 host-resources.py --json -- true（多机开关关着，只查这一台），
# 用 strace -f 记下所有子进程打开的树内文件（含 bash source 进来的），按规范路径去重，打 路径、字节数，末尾总数与 base64 长度。
set -u
tree="$(cd "$1" && pwd)"; out="$2"; mkdir -p "$out"
cat > "$out/multi-host-off.env" <<'ENV'
PEER_SSH_HOST=selftest-peer-host-name
PEER_REPOSITORY_DIRECTORY=/selftest/peer/home
PEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin
PEER_MEMORY_CAP=40G
QUIESCE_STOP_COMMAND=
QUIESCE_STOPPED_CHECK_COMMAND=
QUIESCE_START_COMMAND=
QUIESCE_STARTED_CHECK_COMMAND=
ENABLE_ACROSS_MACHINES=0
ENV
command -v strace >/dev/null || { echo "name=needed_files_strace status=skipped reason=没有 strace"; exit 0; }
(cd "$tree" && SINGLEFS_MULTI_HOST_CONFIG="$out/multi-host-off.env" HOST_RESOURCES_SAMPLE_SECONDS=0.2 RUN_WITH_MEMORY_CAP_PEAKS="$out/peaks.tsv" RUN_WITH_MEMORY_CAP_STATE_DIR="$out/state" \
  timeout 90 nice -n 19 strace -f -qq -e trace=openat,execve -o "$out/strace.txt" python3 -B scripts/hosts/host-resources.py --json -- true > "$out/stdout.txt" 2> "$out/stderr.txt")
python3 - "$tree" "$out/strace.txt" <<'PY'
import os, re, sys
tree, log = os.path.realpath(sys.argv[1]), sys.argv[2]
files = set()
for line in open(log, encoding="utf-8", errors="replace"):
    for match in re.finditer(r'"([^"]+)"', line):
        path = match.group(1)
        if path.startswith("/"):
            real = os.path.realpath(path)
            if real.startswith(tree + os.sep) and os.path.isfile(real) and "/__pycache__/" not in real and "/.git/" not in real:
                files.add(real)
total = 0
for real in sorted(files):
    size = os.path.getsize(real); total += size
    print(f"{size:8d} {os.path.relpath(real, tree)}")
outside = [f for f in files if not os.path.relpath(f, tree).startswith("scripts/hosts/")]
print(f"name=needed_files_strace files={len(files)} outside_scripts_hosts={len(outside)} bytes={total} base64_bytes={(total + 2) // 3 * 4} single_argument_limit=131071")
PY
