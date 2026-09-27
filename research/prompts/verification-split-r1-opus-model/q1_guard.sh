#!/usr/bin/env bash
# Q1：快照树的 heavy-test-guard.sh 对几条漏判命令的真判定（hook 的 JSON 从 stdin 喂）。用法：bash q1_guard.sh [快照目录]
set -uo pipefail
S="${1:-/tmp/claude-1000/three-way-attack/verification-split-r1/snap}"
while IFS='|' read -r command directory agent; do
  python3 -c 'import json,sys; d={"tool_name":"Bash","tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}
if sys.argv[3]: d["agent_type"]=sys.argv[3]
print(json.dumps(d))' "$command" "$directory" "$agent" > /tmp/claude-1000/three-way-attack/verification-split-r1/guard-input.json
  bash "$S/.claude/hooks/heavy-test-guard.sh" < /tmp/claude-1000/three-way-attack/verification-split-r1/guard-input.json > /tmp/claude-1000/three-way-attack/verification-split-r1/guard-output.txt 2>&1
  echo "exit=$? agent=${agent:-main} cwd=${directory#$S} [$command] $(head -1 /tmp/claude-1000/three-way-attack/verification-split-r1/guard-output.txt | cut -c1-120)"
done <<ROWS
bash research/scripts/run-with-memory-cap.sh 8G cargo test -p 'singlefs-check*'|$S|three-way-attack
bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-checker@0.1.0|$S|implementation-writer
cargo test --release -p singlefs-checker@0.1.0|$S|
cargo test --release -p 'singlefs-check*'|$S|
CARGO_ALIAS_XT='test --release -p singlefs-checker' cargo xt|$S/crates/singlefs-harness|
cargo test --release -p singlefs-checker|$S|
ROWS
