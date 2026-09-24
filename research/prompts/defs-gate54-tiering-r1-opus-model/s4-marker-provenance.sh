#!/usr/bin/env bash
# S4　快档只核标记里的输入哈希与「恰好三行计数」（54 号第 213–234 行），不核这三行说了什么、也不核标记是哪一版 54 号写的；
# 输入哈希里没有 54 号脚本自己（stage-inputs.tsv 第 11 行只登记 crates/ Cargo.toml Cargo.lock）。
# 历史：A 这一批让第一条流的枚举退化成非全量（假 cargo 见到 NONEXHAUSTIVE 就打 exhaustive=false，用例本身不红）；
# 主工作区里 54 号脚本有 B 会话没暂存的半成品（把第一条流的 exhaustive 判定关了）；A 收尾照 main-agent.md 第 43 行在主工作区
# 跑 `bash .claude/gate.d/54-layer0-replay.sh --full`——跑的是工作区那一份脚本。然后 A 只暂存自己的 crates/ 改动，起 gate.sh --staged。
# 臂：post（今天的 54 号）｜pre（分档前，门禁自己跑全量）｜f4a（我提的：快档另核标记里两行都有 exhaustive=true）｜
#     f4b（我提的：标记记写它的 54 号脚本的 sha256，快档与自己比）。f4a / f4b 只在这个模型上量过、被攻过零轮。
source "$(dirname "$0")/lib.sh"
base=/tmp/claude-1000/defs54-attack/s4
for arm in post pre f4a f4b; do
  repo="$base/repo-$arm"; build_repo "$repo" "$( [[ $arm == pre ]] && echo pre || echo post )"; cd "$repo" || exit 2
  if [[ "$arm" == f4a ]]; then
    sed -i '/^  marker_finished_utc="/i\  if [[ "$(grep -c "exhaustive=true" <<< "$marker_count_lines")" != 2 ]]; then echo "  ✗ （f4a）标记里的 LAYER0 / LAYER0B 两行不都是 exhaustive=true：$(grep -o "^LAYER0B\\? .*exhaustive=[a-z]*" <<< "$marker_count_lines" | grep -o "^LAYER0B\\?\\|exhaustive=[a-z]*" | paste -sd" ")"; exit 1; fi' "$STAGE"
    gitq commit -qam f4a
  elif [[ "$arm" == f4b ]]; then
    sed -i '/^  echo "judged_root=\$ROOT"$/a\  echo "stage_script_sha256=$(sha256sum < "$0" | cut -c1-64)"' "$STAGE"
    sed -i '/^  marker_finished_utc="/i\  if [[ "$(sed -n "s/^stage_script_sha256=//p" "$full_green_marker_path")" != "$(sha256sum < "$0" | cut -c1-64)" ]]; then echo "  ✗ （f4b）标记不是这一版 54 号写的"; exit 1; fi' "$STAGE"
    gitq commit -qam f4b
  fi
  printf 'pub fn enumerate() { /* NONEXHAUSTIVE：某一段没展开子集 */ }\n' > crates/singlefs-harness/src/crash.rs
  gitq add crates
  # B 会话没暂存的 54 号半成品：第一条流的 exhaustive 判定被关掉
  sed -i 's/^if \[\[ "\$line" != \*"exhaustive=true"\* \]\]; then$/if false; then  # B 会话改到一半/' "$STAGE"
  echo "════ S4 [$arm] 工作区 vs 暂存区：$(git status --porcelain | tr '\n' ' ')；B 的改动命中 $(grep -c 'B 会话改到一半' "$STAGE") 行"
  if [[ "$arm" != pre ]]; then
    echo "  ── 收尾：主工作区 bash $STAGE --full"; bash "$STAGE" --full 2>&1 | grep -E '^  [✗✓]' | sed 's/^/    /'; echo "    $(show_marker "$repo")"
  fi
  echo "  ── gate.sh --staged"; gate_staged_54 "$repo" 2>&1 | grep -E '^  [✗✓]|^      LAYER0 |退出码' | sed 's/^/    /'
done
