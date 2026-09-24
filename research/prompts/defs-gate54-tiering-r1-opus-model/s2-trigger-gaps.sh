#!/usr/bin/env bash
# S2　触发条件漏：这一批只改 Cargo.lock / 只改 Cargo.toml / 只改测试文件 / 只追加 crates/mutations.tsv / 这一批没碰而别的会话改了 crates/。
# 每一格量三样：K3 那一句按字面触不触发（这一批暂存的路径里有没有 crates/ 开头的）；gate.sh --staged 下 54 号（post＝分档后、pre＝分档前）
# 的退出码；另加一臂 fix＝把 54 号「二问」的前缀从 crates/ 换成它自己登记的三条（我提的改法，只在这个模型上量过、被攻过零轮）。
# 前提：base 上已经跑过一次 --full（标记与 base 的输入相等），这一批之后没人再跑 --full（K3 没触发）。
# 两种起门禁的法：gate-triage 定义第 25 行的 `gate.sh --staged`（没有 SINGLEFS_STAGED_TREE）与 gate-staged.sh（有，staged-green 指 base 那棵树）。
source "$(dirname "$0")/lib.sh"
base=/tmp/claude-1000/defs54-attack/s2
apply_batch() { # <格>
  case "$1" in
    lock-only)  printf '# lock v1\n[[package]]\nname = "dep"\nversion = "1.0.1"\n' > Cargo.lock; gitq add Cargo.lock ;;
    toml-only)  printf '[workspace]\nmembers = ["crates/*"]\n[profile.release]\ndebug-assertions = true\n' > Cargo.toml; gitq add Cargo.toml ;;
    test-only)  printf '#[test] fn quick() {}\n#[test] fn quick_new() {}\n' > crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs; gitq add crates/singlefs-harness/tests ;;
    mutations-only) printf '# 变异表\nE999\tcrates/singlefs-harness/src/crash.rs\t...\n' > crates/mutations.tsv; gitq add crates/mutations.tsv ;;
    other-session-crates) printf 'docs\n' > NOTES.md; gitq add NOTES.md; printf 'pub fn core() { /* 别的会话 */ }\n' > crates/singlefs-core/src/lib.rs ;;
  esac
}
for cell in lock-only toml-only test-only mutations-only other-session-crates; do
  for arm in post pre fix; do
    repo="$base/repo-$cell-$arm"; build_repo "$repo" "$( [[ $arm == pre ]] && echo pre || echo post )"; cd "$repo" || exit 2
    if [[ "$arm" == fix ]]; then
      sed -i 's#change-touches-crates.sh" "$ROOT" crates/)#change-touches-crates.sh" "$ROOT" crates/ Cargo.toml Cargo.lock)#' "$STAGE"
      gitq commit -qam 'fix arm'
    fi
    [[ "$arm" != pre ]] && bash "$STAGE" --full >/dev/null
    base_tree="$(git write-tree)"; green="$(gitq commit-tree "$base_tree" -m green)"; git update-ref refs/sop/staged-green "$green"
    apply_batch "$cell"
    staged_paths="$(git diff --cached --name-only | tr '\n' ' ')"
    trigger=no; git diff --cached --name-only | grep -q '^crates/' && trigger=yes
    echo "════ S2 [$cell｜$arm] 暂存的路径：$staged_paths｜K3 字面触发 --full：$trigger｜$(show_marker "$repo")"
    for how in gate-triage gate-staged; do
      if [[ "$how" == gate-staged ]]; then extra=(SINGLEFS_STAGED_TREE="$(git write-tree)"); else extra=(); fi
      echo "  ── 起门禁：$how"
      gate_staged_54 "$repo" "${extra[@]}" 2>&1 | grep -E '^  [✗✓!]|只在|内容不同|退出码' | sed 's/^/    /'
    done
  done
done
