#!/usr/bin/env bash
# 辩方 U3 探针：对本仓现在这一份 heavy-test-guard.sh 喂几条命令的 JSON（只喂 JSON、看退出码，被判的命令一条都不起），外加 54 号样本的 git 模式。
# 用法：bash probe-guard-spot-checks.sh <草稿目录>（已存在拒绝）
set -uo pipefail
scratch="${1:?用法：probe-guard-spot-checks.sh <草稿目录>}"
repository="${SINGLEFS_REPOSITORY:-/home/fy5090/code/singlefs}"
[[ ! -e "$scratch" ]] || { echo "草稿目录已存在：$scratch"; exit 2; }
mkdir -p "$scratch/flat" "$scratch/mirrored/.claude/hooks" "$scratch/gate-copy"
cp "$repository/.claude/hooks/lib_heavy_tests.py" "$scratch/flat/lib_heavy_tests.py"
cp "$repository/.claude/hooks/lib_heavy_tests.py" "$scratch/mirrored/.claude/hooks/lib_heavy_tests.py"

feed() {   # feed <标签> <cwd> <agent|MAIN> <命令>
  local label="$1" directory="$2" agent="$3" command="$4" json code
  json="$(python3 -c 'import json,sys
payload={"tool_name":"Bash","cwd":sys.argv[1],"tool_input":{"command":sys.argv[3]}}
if sys.argv[2]!="MAIN": payload["agent_type"]=sys.argv[2]
print(json.dumps(payload))' "$directory" "$agent" "$command")"
  printf '%s' "$json" | AGENT_HOOK_DETECTIONS="$scratch/detections.jsonl" CLAUDE_PROJECT_DIR="$repository" nice -n 19 bash "$repository/.claude/hooks/heavy-test-guard.sh" >"$scratch/out.txt" 2>"$scratch/err.txt"
  code=$?
  printf '%s\texit=%s\t%s\n' "$label" "$code" "$(head -1 "$scratch/err.txt" | cut -c1-90)"
}

echo "## A 草稿目录里的 lib_heavy_tests.py --selftest（T2-3）：路径尾部是不是 .claude/hooks/ 决定认不认它是自证"
feed "A1 tooling-writer，平的草稿目录 <草稿>/flat/" "$repository" tooling-writer "python3 $scratch/flat/lib_heavy_tests.py --selftest"
feed "A2 tooling-writer，仿 .claude/hooks 的草稿根 <草稿>/mirrored/.claude/hooks/" "$repository" tooling-writer "python3 $scratch/mirrored/.claude/hooks/lib_heavy_tests.py --selftest"
feed "A3 主 agent 不带前缀，仿 .claude/hooks 的草稿根" "$repository" MAIN "python3 $scratch/mirrored/.claude/hooks/lib_heavy_tests.py --selftest"
feed "A4 主 agent 带 SINGLEFS_HEAVY_TESTS=commit，仿 .claude/hooks 的草稿根" "$repository" MAIN "SINGLEFS_HEAVY_TESTS=commit python3 $scratch/mirrored/.claude/hooks/lib_heavy_tests.py --selftest"
feed "A5 tooling-writer，本仓那一份" "$repository" tooling-writer "python3 $repository/.claude/hooks/lib_heavy_tests.py --selftest"

echo "## B 经 stage-selftest.sh 喂样本（T2-4）：喂的是阶段目录的拷贝，样本目录改与不改"
cp -r "$repository/.claude/gate.d/." "$scratch/gate-copy/"
feed "B1 tooling-writer，拷贝原样，--item doc-text.sh" "$repository" tooling-writer "bash $repository/.claude/singlefs-ai-sop/scripts/stage-selftest.sh $scratch/gate-copy --item doc-text.sh"
printf '#!/usr/bin/env bash\ncargo test --release -p singlefs-checker-tier --lib --tests\n' > "$scratch/gate-copy/fixtures/doc-text.sh/prime-marks-red/setup.sh"
feed "B2 tooling-writer，prime-marks-red 样本多了一份 setup.sh（里面写 cargo test），--item doc-text.sh" "$repository" tooling-writer "bash $repository/.claude/singlefs-ai-sop/scripts/stage-selftest.sh $scratch/gate-copy --item doc-text.sh"
feed "B3 tooling-writer，本仓阶段目录、不带 --item（含 samples-heavy 的几道）" "$repository" tooling-writer "bash $repository/.claude/singlefs-ai-sop/scripts/stage-selftest.sh $repository/.claude/gate.d"

echo "## C 54 号样本的 git 模式（T2-2）"
git -C "$repository" ls-files -s -- '.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/*/.layer0-sample-tools/cargo' '.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/*/.layer0-sample-tools/rustc' '.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/*/.layer0-sample-tools/quick-tier' | awk '{print $1, $4}' | sed 's#.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/##'
