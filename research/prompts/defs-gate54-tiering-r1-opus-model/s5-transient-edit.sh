#!/usr/bin/env bash
# S5　--full 只在开跑与跑完各算一次输入哈希（54 号第 246 行与第 310 行）：跑的中途别的会话改了又改回（A→B→A），两端相等，标记照写。
# 假 cargo 的钩子在第二条流「编译」前改 crash.rs、跑完后改回；FAKE_CARGO_COMPILE_LOG 记每条流编译时 crates/ 的内容指纹。
# S5a：A 这一批本身坏（STREAM2_BUG，只有第二条流全量抓得到），B 在那一刻临时删掉了这一行 ⇒ 看该红的红不红。
# S5b：A 这一批是好的，B 在那一刻临时加了 STREAM2_BUG ⇒ 看不该红的红不红。
# 臂：post（主工作区跑 --full，main-agent.md 第 43 行）｜pre（分档前：门禁在临时 worktree 里自己跑全量）｜
#     f5（我提的：收尾 --full 一律在「HEAD + 暂存区」的 worktree 里跑，即把出路句当成默认路；只在这个模型上量过、被攻过零轮）。
source "$(dirname "$0")/lib.sh"
base=/tmp/claude-1000/defs54-attack/s5
for variant in a b; do
  for arm in post pre f5; do
    repo="$base/repo-$variant-$arm"; build_repo "$repo" "$( [[ $arm == pre ]] && echo pre || echo post )"; cd "$repo" || exit 2
    target="$repo/crates/singlefs-harness/src/crash.rs"
    if [[ "$variant" == a ]]; then printf 'pub fn enumerate() {}\n// STREAM2_BUG\n' > "$target"; else printf 'pub fn enumerate() {}\n// good\n' > "$target"; fi
    gitq add crates; cp "$target" "$base/saved-$variant-$arm"
    if [[ "$variant" == a ]]; then hook_before="sed -i '/STREAM2_BUG/d' '$target'"; else hook_before="echo '// STREAM2_BUG' >> '$target'"; fi
    hook_after="cp '$base/saved-$variant-$arm' '$target'"
    compile_log="$base/compile-$variant-$arm.log"; rm -f "$compile_log"
    echo "════ S5$variant [$arm]"
    if [[ "$arm" == pre ]]; then
      echo "  ── gate.sh --staged（门禁自己跑全量；B 在同一时刻改主工作区）"
      gate_staged_54 "$repo" FAKE_CARGO_COMPILE_LOG="$compile_log" FAKE_CARGO_HOOK_BEFORE_SECOND="$hook_before" FAKE_CARGO_HOOK_AFTER_SECOND="$hook_after" 2>&1 | grep -E '^  [✗✓]|退出码' | sed 's/^/    /'
    else
      if [[ "$arm" == f5 ]]; then root="$base/wt-$variant"; build_batch_worktree "$repo" "$root"; else root="$repo"; fi
      echo "  ── 收尾 --full（根：$root；B 在第二条流编译前改主工作区的 crash.rs、跑完改回）"
      FAKE_CARGO_COMPILE_LOG="$compile_log" FAKE_CARGO_HOOK_BEFORE_SECOND="$hook_before" FAKE_CARGO_HOOK_AFTER_SECOND="$hook_after" \
        bash "$root/$STAGE" --full "$root" 2>&1 | grep -E '^  [✗✓·]' | sed 's/^/    /'
      echo "    $(show_marker "$repo")"
      echo "  ── gate.sh --staged"; gate_staged_54 "$repo" 2>&1 | grep -E '^  [✗✓]|退出码' | sed 's/^/    /'
      [[ "$arm" == f5 ]] && gitq -C "$repo" worktree remove --force "$root"
    fi
    echo "  ── 两条流编译时 crates/ 的内容指纹（假 cargo 记的）与主工作区此刻的："; sed 's/^/    /' "$compile_log"
    echo "    主工作区此刻 crash.rs 与开跑前逐字相同：$(cmp -s "$target" "$base/saved-$variant-$arm" && echo 是 || echo 否)"
  done
done
