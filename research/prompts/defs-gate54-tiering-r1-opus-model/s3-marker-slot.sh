#!/usr/bin/env bash
# S3　标记只有一格、--full 开跑先删（54 号第 243 行）：K2（崩溃验证员 --full）与 K3（收尾 --full）撞在一起，或两个会话各跑一次。
# S3a 同一会话：验证员 --full 绿 → 收尾照 main-agent.md 第 43 行无条件再后台跑 --full → 下一行 gate-triage 在它跑的途中起门禁。
#     放开扫：门禁起在收尾 --full（开跑前 / 途中 / 跑完后）× 验证员先前（跑过 / 没跑）；外加 pre 臂（分档前，门禁自己跑全量）。
# S3b 两个会话：A 在出路 worktree 里 --full（标记 H_A），B 在主工作区 --full（B 的半成品在里面）；扫 B 的结果（绿 / 红）× 四种先后。
source "$(dirname "$0")/lib.sh"
base=/tmp/claude-1000/defs54-attack/s3
gate_line() { gate_staged_54 "$1" 2>&1 | grep -E '^  [✗✓!]|退出码' | sed 's/^/    /'; }
echo "════ S3a"
for verifier in yes no; do
  for gate_when in before during after; do
    repo="$base/a-$verifier-$gate_when"; build_repo "$repo" post; cd "$repo" || exit 2
    printf 'pub mod crash;\npub fn batch_a() {}\n' > crates/singlefs-harness/src/lib.rs; gitq add -A crates
    [[ "$verifier" == yes ]] && bash "$STAGE" --full >/dev/null
    echo "── [验证员先跑过 --full：$verifier｜门禁起在收尾 --full 的：$gate_when] 起门禁前 $(show_marker "$repo")"
    if [[ "$gate_when" == before ]]; then gate_line "$repo"; bash "$STAGE" --full >/dev/null
    elif [[ "$gate_when" == during ]]; then
      FAKE_CARGO_SLEEP_FULL=3 bash "$STAGE" --full > "$base/a-closeout.log" 2>&1 &
      sleep 1; echo "    (收尾 --full 正在跑：$(show_marker "$repo"))"; gate_line "$repo"; wait
    else bash "$STAGE" --full >/dev/null; gate_line "$repo"; fi
  done
done
repo="$base/a-pre"; build_repo "$repo" pre; cd "$repo" || exit 2
printf 'pub mod crash;\npub fn batch_a() {}\n' > crates/singlefs-harness/src/lib.rs; gitq add -A crates
echo "── [pre 臂：分档前的 54 号，门禁自己在临时 worktree 里跑全量]"; gate_line "$repo"
echo "════ S3b"
# 时序用假 cargo 第一条流的 sleep 排开：长的那一次 3 秒，短的 1 秒，后起的晚 1 秒。
for b_result in green red; do
  for order in A-then-B B-then-A B-inside-A A-inside-B; do
    repo="$base/b-$b_result-$order"; build_repo "$repo" post; cd "$repo" || exit 2
    printf 'pub mod crash;\npub fn batch_a() {}\n' > crates/singlefs-harness/src/lib.rs; gitq add -A crates
    [[ "$b_result" == red ]] && printf 'pub fn wip() { /* STREAM1_BUG */ }\n' > crates/singlefs-core/src/b_wip.rs
    [[ "$b_result" == green ]] && printf 'pub fn wip() {}\n' > crates/singlefs-core/src/b_wip.rs
    wt="$base/b-$b_result-$order-outlet"; build_batch_worktree "$repo" "$wt"
    run_a() { FAKE_CARGO_SLEEP_FULL="$1" bash "$wt/$STAGE" --full "$wt" > "$base/b-A.log" 2>&1; echo "    A 的 --full 退出码 $?（出路 worktree）"; }
    run_b() { FAKE_CARGO_SLEEP_FULL="$1" bash "$repo/$STAGE" --full "$repo" > "$base/b-B.log" 2>&1; echo "    B 的 --full 退出码 $?（主工作区）"; }
    case "$order" in
      A-then-B) run_a 0; run_b 0 ;;
      B-then-A) run_b 0; run_a 0 ;;
      B-inside-A) run_a 3 & { sleep 1; run_b 0; }; wait ;;
      A-inside-B) run_b 3 & { sleep 1; run_a 0; }; wait ;;
    esac
    echo "── [B 的结果：$b_result｜先后：$order] A 起门禁前 $(show_marker "$repo")"; gate_line "$repo"
    gitq -C "$repo" worktree remove --force "$wt"
  done
done
repo="$base/b-pre"; build_repo "$repo" pre; cd "$repo" || exit 2
printf 'pub mod crash;\npub fn batch_a() {}\n' > crates/singlefs-harness/src/lib.rs; gitq add -A crates
printf 'pub fn wip() { /* STREAM1_BUG */ }\n' > crates/singlefs-core/src/b_wip.rs
echo "── [pre 臂：B 的半成品红在主工作区，A 的门禁在临时 worktree 里自己跑全量]"; gate_line "$repo"
echo "════ S3c 两份定义里「别的 gate.sh 在跑」那道等待（crash-verifier.md 第 23 行、gate-triage.md 第 24 行）看得见收尾那一趟吗"
repo="$base/c"; build_repo "$repo" post; cd "$repo" || exit 2
FAKE_CARGO_SLEEP_FULL=2 bash "$STAGE" --full > /dev/null 2>&1 &
full_pid=$!
sleep 1
echo "    收尾 --full 的进程（pid $full_pid）的命令行：$(ps -o args= -p "$full_pid")"
echo "    这条命令行里含 gate.sh 的次数：$(ps -o args= -p "$full_pid" | grep -c 'gate\.sh')"
wait
