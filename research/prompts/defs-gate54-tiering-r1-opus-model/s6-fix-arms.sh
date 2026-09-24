#!/usr/bin/env bash
# S6　两个改法在打中的格上重跑（都是我提的，只在这个模型上量过、被攻过零轮）：
#   f3：标记按输入哈希分格（<common-dir>/singlefs-layer0-full-green.<输入哈希>），--full 开跑不删任何一格——重跑 S3a、S3b 的全部格。
#       （「--full 判红时删掉同一哈希那一格」没实现，是推的。）
#   f5：收尾 --full 在「HEAD + 暂存区」的 worktree 里、用那棵树里的 54 号跑（出路句当默认路）——重跑 S4。
source "$(dirname "$0")/lib.sh"
base=/tmp/claude-1000/defs54-attack/s6
apply_f3() { # 在合成仓的 54 号上打 f3，打完提交；三处替换各核命中 1 次
  local s="$1/$STAGE"
  sed -i '/^  write_layer0_input_manifest "\$quick_manifest" || fail_without_input_manifest$/a\  full_green_marker_path="$full_green_marker_path.$layer0_input_hash"  # f3' "$s"
  sed -i 's/^rm -f -- "\${full_green_marker_path:?}"$/: # f3：开跑不删/' "$s"
  sed -i '/^write_layer0_input_manifest "\$manifest_at_start" || fail_without_input_manifest$/a\full_green_marker_path="$full_green_marker_path.$layer0_input_hash"  # f3' "$s"
  echo "  (f3 打上：$(grep -c '# f3' "$s") 处)"
  gitq -C "$1" commit -qam f3
}
markers() { ls "$(git -C "$1" rev-parse --path-format=absolute --git-common-dir)" | grep -c '^singlefs-layer0-full-green' ; }
gate_line() { gate_staged_54 "$1" 2>&1 | grep -E '^  [✗✓!]|退出码' | sed 's/^/    /'; }
echo "════ S3a × f3"
for verifier in yes no; do
  for gate_when in before during after; do
    repo="$base/a-$verifier-$gate_when"; build_repo "$repo" post; apply_f3 "$repo"; cd "$repo" || exit 2
    printf 'pub mod crash;\npub fn batch_a() {}\n' > crates/singlefs-harness/src/lib.rs; gitq add -A crates
    [[ "$verifier" == yes ]] && bash "$STAGE" --full >/dev/null
    echo "── [f3｜验证员先跑过：$verifier｜门禁起在收尾 --full 的：$gate_when] 标记格数 $(markers "$repo")"
    if [[ "$gate_when" == before ]]; then gate_line "$repo"; bash "$STAGE" --full >/dev/null
    elif [[ "$gate_when" == during ]]; then
      FAKE_CARGO_SLEEP_FULL=3 bash "$STAGE" --full > "$base/a-closeout.log" 2>&1 &
      sleep 1; gate_line "$repo"; wait
    else bash "$STAGE" --full >/dev/null; gate_line "$repo"; fi
  done
done
echo "════ S3b × f3"
for b_result in green red; do
  for order in A-then-B B-then-A B-inside-A A-inside-B; do
    repo="$base/b-$b_result-$order"; build_repo "$repo" post; apply_f3 "$repo"; cd "$repo" || exit 2
    printf 'pub mod crash;\npub fn batch_a() {}\n' > crates/singlefs-harness/src/lib.rs; gitq add -A crates
    [[ "$b_result" == red ]] && printf 'pub fn wip() { /* STREAM1_BUG */ }\n' > crates/singlefs-core/src/b_wip.rs
    [[ "$b_result" == green ]] && printf 'pub fn wip() {}\n' > crates/singlefs-core/src/b_wip.rs
    wt="$base/b-$b_result-$order-outlet"; build_batch_worktree "$repo" "$wt"
    run_a() { FAKE_CARGO_SLEEP_FULL="$1" bash "$wt/$STAGE" --full "$wt" > /dev/null 2>&1; echo "    A 的 --full 退出码 $?"; }
    run_b() { FAKE_CARGO_SLEEP_FULL="$1" bash "$repo/$STAGE" --full "$repo" > /dev/null 2>&1; echo "    B 的 --full 退出码 $?"; }
    case "$order" in
      A-then-B) run_a 0; run_b 0 ;;
      B-then-A) run_b 0; run_a 0 ;;
      B-inside-A) run_a 3 & { sleep 1; run_b 0; }; wait ;;
      A-inside-B) run_b 3 & { sleep 1; run_a 0; }; wait ;;
    esac
    echo "── [f3｜B 的结果：$b_result｜先后：$order] 标记格数 $(markers "$repo")"; gate_line "$repo"
    gitq -C "$repo" worktree remove --force "$wt"
  done
done
echo "════ S4 × f5"
repo="$base/s4-f5"; build_repo "$repo" post; cd "$repo" || exit 2
printf 'pub fn enumerate() { /* NONEXHAUSTIVE：某一段没展开子集 */ }\n' > crates/singlefs-harness/src/crash.rs; gitq add crates
sed -i 's/^if \[\[ "\$line" != \*"exhaustive=true"\* \]\]; then$/if false; then  # B 会话改到一半/' "$STAGE"
echo "  工作区 vs 暂存区：$(git status --porcelain | tr '\n' ' ')；B 的改动命中 $(grep -c 'B 会话改到一半' "$STAGE") 行"
wt="$base/s4-f5-wt"; build_batch_worktree "$repo" "$wt"
echo "  ── 收尾（f5）：bash <worktree>/$STAGE --full <worktree>"; bash "$wt/$STAGE" --full "$wt" 2>&1 | grep -E '^  [✗✓]' | sed 's/^/    /'; echo "    $(show_marker "$repo")"
echo "  ── gate.sh --staged"; gate_line "$repo"
gitq -C "$repo" worktree remove --force "$wt"
