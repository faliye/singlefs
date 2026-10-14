#!/usr/bin/env bash
# R5 探针：重型测试闸（冻结副本拷贝里的 heavy-test-guard.sh 入口，喂 JSON）对「工作根底下仓副本里的自证」与「-p 点名当前目录的探针包」几种写法判什么。
# 用法：bash r5-heavy-guard.sh <冻结副本> <新的工作目录>；每格一行 name=r5 case=<格> rc=<退出码> checker_msg=<stderr 里提没提 checker>
# 不起 cargo、不跑任何自证：只把命令行喂给钩子判。工作根用 SINGLEFS_WORK_ROOT 指到 <工作目录>/r5-work-root（钩子进程的环境）。
set -euo pipefail
FROZEN="$1"; WORK="$2"
WROOT="$WORK/r5-work-root"
HOOK="$WROOT/hook-root"                     # 钩子与「主仓」登记表所在的那一份
mkdir -p "$HOOK"
rsync -a --exclude __pycache__ "$FROZEN/.claude" "$FROZEN/scripts" "$HOOK/"
# 仓副本：登记表与主仓逐字相同，脚本正文换成起 checker 档测试（钩子不读正文）
make_copy() {   # make_copy <目录>
  mkdir -p "$1/.claude/hooks" "$1/scripts/agents"
  cp "$HOOK/.claude/hooks/selftest-registry.tsv" "$1/.claude/hooks/selftest-registry.tsv"
  printf '#!/usr/bin/env python3\nimport subprocess\nsubprocess.run(["cargo", "test", "-p", "singlefs-checker-tier", "--", "--include-ignored"])\n' > "$1/scripts/agents/agent-cost.py"
}
make_copy "$WROOT/copy-a"
make_copy "$WORK/r5-outside-copy"
# 探针包：链接 checker 档包（假的同名包，只有 Cargo.toml）
mkdir -p "$HOOK/probes/fake-tier/src" "$HOOK/probes/linked/src"
printf '[package]\nname = "singlefs-checker-tier"\nversion = "0.1.0"\nedition = "2021"\n' > "$HOOK/probes/fake-tier/Cargo.toml"
printf '[package]\nname = "attack-probe"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nsinglefs-checker-tier = { path = "../fake-tier" }\n' > "$HOOK/probes/linked/Cargo.toml"
: > "$HOOK/probes/linked/src/main.rs"; : > "$HOOK/probes/fake-tier/src/lib.rs"
feed() {   # feed <格名> <agent_type，空串是主 agent> <cwd> <命令> [<SINGLEFS_WORK_ROOT>]
  local label="$1" agent="$2" cwd="$3" command="$4" work_root="${5:-$WROOT}" rc
  python3 - "$agent" "$cwd" "$command" > "$WORK/r5-$label.json" <<'PY'
import json, sys
agent, cwd, command = sys.argv[1:4]
data = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": command}}
if agent:
    data["agent_type"] = agent
print(json.dumps(data, ensure_ascii=False))
PY
  if env SINGLEFS_WORK_ROOT="$work_root" CLAUDE_PROJECT_DIR="$HOOK" bash "$HOOK/.claude/hooks/heavy-test-guard.sh" < "$WORK/r5-$label.json" > "$WORK/r5-$label.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=r5 case=$label rc=$rc checker_msg=$(grep -c 'checker' "$WORK/r5-$label.out" || true)"
}
# 仓副本里的自证：正文已改成起 checker 档测试
feed copy-light-selftest-body-heavy three-way-attack "$WROOT/copy-a" "python3 scripts/agents/agent-cost.py --selftest"
feed copy-light-selftest-body-heavy-abs implementation-writer "$WORK" "python3 $WROOT/copy-a/scripts/agents/agent-cost.py --selftest"
feed copy-outside-work-root three-way-attack "$WORK/r5-outside-copy" "python3 scripts/agents/agent-cost.py --selftest"
# 主仓那一份（对照：登记成 light，放行）
feed main-light-selftest three-way-attack "$HOOK" "python3 scripts/agents/agent-cost.py --selftest"
# -p 点名当前目录的探针（几种起法）
wrap="bash $HOOK/scripts/hosts/run-with-memory-cap.sh 8G"
feed p-cwd-probe implementation-writer "$HOOK/probes/linked" "$wrap cargo test -p attack-probe"
feed p-cwd-probe-src-subdir implementation-writer "$HOOK/probes/linked/src" "$wrap cargo test -p attack-probe"
feed p-glob implementation-writer "$HOOK/probes/linked" "$wrap cargo test -p 'attack*'"
feed p-version implementation-writer "$HOOK/probes/linked" "$wrap cargo test --package attack-probe@0.1.0"
feed p-env-C implementation-writer "$HOOK" "$wrap env -C probes/linked cargo test -p attack-probe"
feed p-pushd implementation-writer "$HOOK" "$wrap bash -c 'pushd probes/linked >/dev/null && cargo test -p attack-probe'"
feed p-cd-semicolon implementation-writer "$HOOK" "cd probes/linked; $wrap cargo test -p attack-probe"
feed p-manifest-eq implementation-writer "$HOOK" "$wrap cargo test --manifest-path=probes/linked/Cargo.toml -p attack-probe"
feed bench-cwd-probe implementation-writer "$HOOK/probes/linked" "$wrap cargo bench"
feed run-cwd-probe-bin implementation-writer "$HOOK/probes/linked" "$wrap cargo run --bin attack-probe"
# env -C（GNU env 换目录再起）：钩子按 JSON 里的 cwd 判，不跟 env 换的目录。仓副本只拷各 Cargo.toml（钩子只读清单，不编）
COPY="$WROOT/repo-copy"
mkdir -p "$COPY" "$WORK/r5-scratch"
cp "$FROZEN/Cargo.toml" "$COPY/"
(cd "$FROZEN" && find crates -name Cargo.toml -not -path '*/target/*') | while read -r manifest; do
  mkdir -p "$COPY/$(dirname "$manifest")"; cp "$FROZEN/$manifest" "$COPY/$manifest"
done
feed envC-from-scratch-into-probe implementation-writer "$WORK/r5-scratch" "$wrap env -C $HOOK/probes/linked cargo test"
feed copy-tier-cd-bare implementation-writer "$COPY/crates/singlefs-checker-tier" "$wrap cargo test"
feed copy-tier-manifest-path implementation-writer "$WORK/r5-scratch" "$wrap cargo test --manifest-path $COPY/crates/singlefs-checker-tier/Cargo.toml"
feed copy-tier-envC implementation-writer "$WORK/r5-scratch" "$wrap env -C $COPY/crates/singlefs-checker-tier cargo test"
feed copy-tier-envC-long implementation-writer "$WORK/r5-scratch" "$wrap env --chdir=$COPY/crates/singlefs-checker-tier cargo test --release"
