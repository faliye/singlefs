#!/usr/bin/env bash
# 全部钩子判定：合成 PreToolUse[Bash] JSON 喂 .claude/hooks/*.sh（probe.sh），检出记录写进 $SCRATCH/detections.jsonl，不进默认检出文件。
# 用法：SCRATCH=<草稿目录> bash probes.sh
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; R=/home/user/singlefs
SCRATCH=${SCRATCH:?草稿目录}; export SCRATCH; C="$SCRATCH/copy"
p() { LINES_SHOWN=${LINES_SHOWN:-2} bash "$HERE/probe.sh" "$@"; }
echo "## G1 主 agent 的层 0 全量命令（main-agent.md:59）"
p heavy-test-guard.sh - $R 'SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash /tmp/tmp.X/tree/.claude/gate.d/54-layer0-replay.sh --full /tmp/tmp.X/tree' true
p heavy-test-guard.sh - $R 'bash research/scripts/run-with-memory-cap.sh 16G bash /tmp/tmp.X/tree/.claude/gate.d/54-layer0-replay.sh --full /tmp/tmp.X/tree' true
p heavy-test-guard.sh crash-verifier $R 'SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh'
p heavy-test-guard.sh gate-triage $R 'SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged'
echo "## G2 子 agent 经内存包装（agent-common.md:46）"
p heavy-test-guard.sh implementation-writer $C 'bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract'
p heavy-test-guard.sh three-way-attack $C 'bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract'
p heavy-test-guard.sh three-way-attack "$C/research" 'bash scripts/run-with-memory-cap.sh 4G cargo run --release --bin e7-foo'
p heavy-test-guard.sh implementation-writer $C 'cargo test -p singlefs-core --test core_contract'
p heavy-test-guard.sh implementation-writer $R 'nice -n 19 bash .claude/gate.d/74-model-differential.sh'
p heavy-test-guard.sh implementation-writer $R 'cargo test --release -p singlefs-harness --test model_differential -- --nocapture'
echo "## G2 实验执行员第 5 步（experiment-runner.md:32）"
LINES_SHOWN=4 p heavy-test-guard.sh experiment-runner $R 'bash research/scripts/replay.sh E158'
p heavy-test-guard.sh experiment-runner $R 'bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E158'
echo "## G3 本地腿第 3、4 步（three-way-local-attack.md:27-28），run_in_background"
P=research/prompts/rX-local-attack
p bash-command-detector.sh three-way-local-attack $R "bash research/scripts/ask-local.sh $P.md > $P-output-s1.md" true
p bash-command-detector.sh three-way-local-attack $R "set -o noclobber; bash research/scripts/ask-local.sh $P.md > $P-output-s1.md" true
p bash-command-detector.sh three-way-local-attack $R "bash research/scripts/ask-local.sh $P.md >| $P-output-s1.md" true
echo "## G6 & 与 wait（agent-common.md:55）"
p bash-command-detector.sh implementation-writer $R 'sleep 1 & first=$!; sleep 2 & second=$!; wait "$first"; a=$?; wait "$second"; b=$?; echo $a $b' true
p bash-command-detector.sh implementation-writer $R 'sleep 1 & sleep 2 & wait' true
p bash-command-detector.sh implementation-writer $R 'sleep 1 &' true
