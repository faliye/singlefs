#!/usr/bin/env bash
# H-C（V2）：同一批输入（输入哈希相同）上另一趟 --full 判红，EXIT trap 删掉这批输入那一格——而那一格是 X 在
# HEAD + 暂存区的 worktree 里用入库的 54 号跑绿写下的。另一趟（Z）在主工作区里跑：
#   wip    Z 是正在改 54 号的会话，主工作区里的 54 号是它没暂存的半成品（判得比入库的严），它拿半成品在主工作区试跑 --full
#   edit   Z 用的是入库的 54 号（例：崩溃验证员被点名跑全量，定义第 2 步在主工作区起 54 号），跑的途中主工作区里的 crates/ 被人改了
# 前缀固定：HEAD 的 54 号是 new；X 这一批改 crash.rs（多线程，入库 54 号的 --full 该绿）并暂存；主工作区的 crates/ 与暂存区相同。
# 放开扫的用户动作：Z 这一趟相对 X 那一趟的先后——Zfirst（Z 跑完才起 X）/ Zlast（X 跑完才起 Z）/
# overlapZlast（两趟同时在跑，X 先跑完）/ overlapXlast（两趟同时在跑，Z 先跑完）。
source "$(dirname "$0")/lib.sh"
outpath_file "$MODEL/inputs/54-post.sh" "$RUNS/outpath.sh"

start_x() { ( cd "$1" && FAKE_CARGO_HOLD="$2" bash "$RUNS/outpath.sh" > "$1.x-full.log" 2>&1 ) & x_pid=$!; wait_started "$2"; }
start_z() { # 分档之前的 54 号不认 --full，全量是它的默认
  local tier_argument=(--full); grep -qF -- '--full) layer0_tier="full" ;;' "$1/$STAGE" || tier_argument=()
  ( cd "$1" && FAKE_CARGO_HOLD="$2" bash "$STAGE" "${tier_argument[@]}" > "$1.z-full.log" 2>&1 ) & z_pid=$!; wait_started "$2"; }
wait_started() { for _ in $(seq 1 600); do [[ -e "$1.started" ]] && break; sleep 0.1; done; }
finish_x() { : > "$1.release"; wait "$x_pid"; }
finish_z() { # <仓> <hold> <kind>
  [[ "$3" == edit ]] && printf 'pub mod crash;\n// 主工作区里接着改的下一处（没暂存）\n' > "$1/crates/singlefs-harness/src/lib.rs"
  : > "$2.release"; wait "$z_pid"
}

run_case() { # <arm> <kind> <order>
  local arm="$1" kind="$2" order="$3" r hx hz
  want "$arm-$kind-$order" || return 0
  r="$RUNS/h-c-$arm-$kind-$order"
  echo "════ H-C [$arm] Z=$kind 先后=$order"
  make_repo "$r" "$arm" new ""
  set_crash "$r" c1 ""; git -C "$r" add crates/singlefs-harness/src/crash.rs
  [[ "$kind" == wip ]] && set_54 "$r" "$arm" wip-strict
  echo "  ── 主工作区相对暂存区（Z 开跑前）：$(git -C "$r" diff --name-only | tr '\n' ' ')"
  hx="$RUNS/tmp/hx-$$-$RANDOM"; hz="$RUNS/tmp/hz-$$-$RANDOM"
  if [[ "$arm" == post ]]; then
    case "$order" in
      Zfirst)       start_z "$r" "$hz"; finish_z "$r" "$hz" "$kind"; start_x "$r" "$hx"; finish_x "$hx" ;;
      Zlast)        start_x "$r" "$hx"; finish_x "$hx"; start_z "$r" "$hz"; finish_z "$r" "$hz" "$kind" ;;
      overlapZlast) start_x "$r" "$hx"; start_z "$r" "$hz"; finish_x "$hx"; finish_z "$r" "$hz" "$kind" ;;
      overlapXlast) start_x "$r" "$hx"; start_z "$r" "$hz"; finish_z "$r" "$hz" "$kind"; finish_x "$hx" ;;
    esac
    echo "  ── X 那一趟（worktree，入库的 54 号）："; grep -E '^  (✓ 全绿|✗)' "$r.x-full.log" | trim 150
  else
    start_z "$r" "$hz"; finish_z "$r" "$hz" "$kind"
  fi
  echo "  ── Z 那一趟（主工作区）："; grep -E '^  (✓ 全绿|✗)' "$r.z-full.log" | trim 150
  cells "$r"
  echo "  ── X 起门禁"; gate_staged_54 "$r"
  echo "  ── 真值"; truth_full "$r"
  echo
}

for kind in wip edit; do
  for order in Zfirst Zlast overlapZlast overlapXlast; do run_case post "$kind" "$order"; done
  run_case pre "$kind" Zlast
done
