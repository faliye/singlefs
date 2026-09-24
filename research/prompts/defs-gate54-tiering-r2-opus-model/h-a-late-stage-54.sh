#!/usr/bin/env bash
# H-A（V1）：X 建完 HEAD + 暂存区的 worktree 之后，Y 往暂存区放了一个只改 54 号的改动（54 号不在输入哈希里）。
# 前缀固定：HEAD 的 54 号是 old（没有「多核只起 1 个工作线程判红」），X 这一批把 crash.rs 改成 SINGLE_THREADED（或对照：多线程）。
# 放开扫的用户动作：Y 暂存的时刻 T0（X 建 worktree 之前）/ T1（X 的 --full 跑的途中）/ T2（X 的 --full 跑完、X 起门禁之前）/ T3（X 的门禁之后）；
# Y 放进来的是 54 号（new）还是一处 crates 改动（对照：登记的输入）。
source "$(dirname "$0")/lib.sh"
outpath_file "$MODEL/inputs/54-post.sh" "$RUNS/outpath.sh"

y_stages() { # <仓> <arm> <what>
  if [[ "$3" == 54 ]]; then set_54 "$1" "$2" new; git -C "$1" add "$STAGE"
  else printf 'pub mod crash;\n// Y 的改动\n' > "$1/crates/singlefs-harness/src/lib.rs"; git -C "$1" add crates/singlefs-harness/src/lib.rs; fi
}

run_case() { # <arm> <timing> <X 的记号> <Y 放什么>
  local arm="$1" timing="$2" token="$3" what="$4" r hold pid
  want "$arm-$timing-${token:-multi}-$what" || return 0
  r="$RUNS/h-a-$arm-$timing-${token:-multi}-$what"
  echo "════ H-A [$arm] Y 在 $timing 暂存 ${what}；X 的 crash.rs 记号=${token:-多线程}"
  make_repo "$r" "$arm" old ""
  set_crash "$r" x "$token"; git -C "$r" add crates/singlefs-harness/src/crash.rs
  [[ "$timing" == T0 ]] && y_stages "$r" "$arm" "$what"
  if [[ "$arm" == post ]]; then
    hold="$RUNS/tmp/hold-$$-$RANDOM"
    ( cd "$r" && FAKE_CARGO_HOLD="$hold" bash "$RUNS/outpath.sh" > "$r.x-full.log" 2>&1 ) &
    pid=$!
    for _ in $(seq 1 600); do [[ -e "$hold.started" ]] && break; sleep 0.1; done
    [[ "$timing" == T1 ]] && y_stages "$r" "$arm" "$what"
    : > "$hold.release"; wait "$pid"
    echo "  ── X 的 --full（出路句三行原样，worktree 建于 Y 暂存$([[ $timing == T0 ]] && echo 之后 || echo 之前)）："
    grep -E '^  (✓ 全绿|✗)' "$r.x-full.log" | trim 150
    cells "$r"
  fi
  # 分档之前没有 X 的那一趟 --full：T1、T2 都落在 X 起门禁之前
  if [[ "$timing" == T2 || ( "$arm" == pre && "$timing" == T1 ) ]]; then y_stages "$r" "$arm" "$what"; fi
  echo "  ── X 起门禁（gate.sh --staged 的建法，只跑 54 号）；暂存区：$(git -C "$r" diff --cached --name-only | tr '\n' ' ')"
  gate_staged_54 "$r"; echo "  ── 真值（X 门禁那一刻的 HEAD + 暂存区）"; truth_full "$r"
  if [[ "$timing" == T3 ]]; then
    y_stages "$r" "$arm" "$what"
    echo "  ── Y 起门禁；暂存区：$(git -C "$r" diff --cached --name-only | tr '\n' ' ')"
    gate_staged_54 "$r"; echo "  ── 真值（Y 门禁那一刻）"; truth_full "$r"
  fi
  echo
}

for arm in post pre; do
  for timing in T0 T1 T2 T3; do
    run_case "$arm" "$timing" SINGLE_THREADED 54
  done
done
echo "════════ 对照：X 的内容本来就是多线程（new 54 --full 也该绿）"
for timing in T1 T3; do run_case post "$timing" "" 54; done
echo "════════ 对照：Y 放进来的是登记的输入（crates/）"
for timing in T1 T2; do run_case post "$timing" SINGLE_THREADED crates; done
