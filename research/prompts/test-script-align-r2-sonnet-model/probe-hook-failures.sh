#!/usr/bin/env bash
# 辩方 U4 探针：改钩子那条流程里「一次坏的换上」在 heavy-test-guard.sh 上各是什么结局。
# 只读主工作区：把四份钩子文件拷到草稿目录的变异根里，在拷贝上造四种坏法，只把 hook 的 JSON 从标准输入喂进去、看退出码与 stderr，
# 被判的命令一条都不起。不碰主工作区。
# 用法：bash probe-hook-failures.sh <草稿目录>   （目录已存在拒绝，换名字）
set -uo pipefail
scratch="${1:?用法：probe-hook-failures.sh <草稿目录>}"
repository="${SINGLEFS_REPOSITORY:-/home/fy5090/code/singlefs}"
[[ ! -e "$scratch" ]] || { echo "草稿目录已存在：$scratch"; exit 2; }
mkdir -p "$scratch"

make_variant() {   # make_variant <名> <坏法>
  python3 - "$repository" "$scratch" "$1" "$2" <<'PY'
import os, sys
repository, scratch, name, kind = sys.argv[1:5]
source = os.path.join(repository, ".claude", "hooks")
destination = os.path.join(scratch, name, ".claude", "hooks")
os.makedirs(destination)
for file_name in ("lib_heavy_tests.py", "lib_shell_words.py", "selftest-registry.tsv", "heavy-test-guard.sh"):
    data = open(os.path.join(source, file_name), encoding="utf-8").read()
    if kind == "lib-syntax" and file_name == "lib_heavy_tests.py":
        data += "\ndef broken(:\n"
    if kind == "callee-missing" and file_name == "lib_heavy_tests.py":
        marker = "def stage_samples_row(stage):"
        index = data.index(marker)
        data = data[:index] + data[index:].replace(marker, "def stage_samples_row_renamed(stage):", 1)
    if kind == "embedded-python-syntax" and file_name == "heavy-test-guard.sh":
        data = data.replace("def heavy_tests_library_path(hook_dir):", "def heavy_tests_library_path(:", 1)
    if kind == "bash-tail-syntax" and file_name == "heavy-test-guard.sh":
        data += "\nif true; then\n"
    with open(os.path.join(destination, file_name), "w", encoding="utf-8") as handle:
        handle.write(data)
os.symlink(os.path.join(repository, ".claude", "scripts"), os.path.join(scratch, name, ".claude", "scripts"))
os.symlink(os.path.join(repository, ".claude", "singlefs-ai-sop"), os.path.join(scratch, name, ".claude", "singlefs-ai-sop"))
PY
}

feed() {   # feed <变异根> <标签> <cwd> <agent|MAIN> <命令>
  local root="$1" label="$2" directory="$3" agent="$4" command="$5" json code
  json="$(python3 -c 'import json,sys
payload={"tool_name":"Bash","cwd":sys.argv[1],"tool_input":{"command":sys.argv[3]}}
if sys.argv[2]!="MAIN": payload["agent_type"]=sys.argv[2]
print(json.dumps(payload))' "$directory" "$agent" "$command")"
  printf '%s' "$json" | AGENT_HOOK_DETECTIONS="$scratch/detections.jsonl" CLAUDE_PROJECT_DIR="$root" nice -n 19 bash "$root/.claude/hooks/heavy-test-guard.sh" >"$scratch/out.txt" 2>"$scratch/err.txt"
  code=$?
  printf '%s\texit=%s\t%s\t%s\n' "$label" "$code" "$(printf '%s' "$command" | cut -c1-60)" "$(head -1 "$scratch/err.txt" | cut -c1-110)"
}

static_checks() {  # static_checks <变异根>：交回前「语法检查」能不能看出来
  local root="$1" shell_ok python_lib_ok embedded_ok
  bash -n "$root/.claude/hooks/heavy-test-guard.sh" >/dev/null 2>&1 && shell_ok=pass || shell_ok=FAIL
  python3 - "$root/.claude/hooks/lib_heavy_tests.py" >/dev/null 2>&1 <<'PY' && python_lib_ok=pass || python_lib_ok=FAIL
import sys
compile(open(sys.argv[1], encoding="utf-8").read(), sys.argv[1], "exec")
PY
  python3 - "$root/.claude/hooks/heavy-test-guard.sh" >/dev/null 2>&1 <<'PY' && embedded_ok=pass || embedded_ok=FAIL
import sys
text = open(sys.argv[1], encoding="utf-8").read()
start = text.index("3<<'PY'\n") + len("3<<'PY'\n")
end = text.index("\nPY\n", start)
compile(text[start:end], "embedded", "exec")
PY
  printf '  语法检查：bash -n 钩子=%s；编译 lib=%s；抽出钩子里内嵌的 python 再编=%s\n' "$shell_ok" "$python_lib_ok" "$embedded_ok"
}

for spec in intact lib-syntax callee-missing embedded-python-syntax bash-tail-syntax; do make_variant "v-$spec" "$spec"; done

echo "# 标签<TAB>退出码<TAB>命令<TAB>stderr 首行；cargo test --all 由 implementation-writer 喂（该拒，退 2），ls -la 谁喂都该放（退 0）"
for spec in intact lib-syntax callee-missing embedded-python-syntax bash-tail-syntax; do
  root="$scratch/v-$spec"
  echo "== $spec"
  static_checks "$root"
  feed "$root" "$spec/heavy" /tmp implementation-writer "cargo test --all"
  feed "$root" "$spec/benign" /tmp implementation-writer "ls -la"
  feed "$root" "$spec/benign-main" /tmp MAIN "ls -la"
  feed "$root" "$spec/54-stage" "$repository" tooling-writer "bash .claude/gate.d/checker-tier-layer0-replay.sh $repository"
done
echo "== 检出记录（只有走到 record_detection 的才写）"
python3 - "$scratch/detections.jsonl" <<'PY'
import json, sys
for number, line in enumerate(open(sys.argv[1], encoding="utf-8"), 1):
    record = json.loads(line)
    print(f"  {number}\t{record.get('agent_type')}\t{record.get('command')}\t{str(record.get('findings'))[:90]}")
PY
