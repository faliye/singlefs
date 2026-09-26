#!/usr/bin/env bash
# 用法：bash g1-exit-status-fixed.sh <仓根> <草稿目录>
# 与 g1-exit-status.sh 同一套桩与历史，只把第三行换成攻方自己提的写法（被攻过零轮）：
# 记下 --full 那一条的退出码，删 worktree、删临时目录之后用它退出。
set -uo pipefail
root="$1"; draft="$2"
printed="$(bash -c "$(sed -n '/^print_staged_worktree_full_commands() {/,/^}/p' "$root/.claude/gate.d/54-layer0-replay.sh"); print_staged_worktree_full_commands")"
first_two="$(printf '%s\n' "$printed" | sed -n '2,3p' | sed 's/^ *//')"
third='SINGLEFS_HEAVY_TESTS=commit bash research/scripts/stub-memory-cap.sh 16G bash "$layer0_full_base/tree/.claude/gate.d/stub-full.sh" --full "$layer0_full_base/tree"; layer0_full_rc=$?; git worktree remove --force "$layer0_full_base/tree"; rm -rf "$layer0_full_base"; exit "$layer0_full_rc"'
for outcome in green red memory-cap-250 scope-unavailable-251; do
  repo="$(mktemp -d "$draft/g1f-repo-XXXXXX")"
  git -C "$repo" init -q; git -C "$repo" config user.email m@x; git -C "$repo" config user.name m
  mkdir -p "$repo/.claude/gate.d" "$repo/research/scripts" "$repo/crates"; echo 'fn a() {}' > "$repo/crates/lib.rs"
  case "$outcome" in green) f=0; c="" ;; red) f=1; c="" ;; memory-cap-250) f=0; c=250 ;; scope-unavailable-251) f=0; c=251 ;; esac
  printf '#!/usr/bin/env bash\nexit %s\n' "$f" > "$repo/.claude/gate.d/stub-full.sh"
  if [[ -n "$c" ]]; then printf '#!/usr/bin/env bash\nexit %s\n' "$c" > "$repo/research/scripts/stub-memory-cap.sh"; else printf '#!/usr/bin/env bash\nshift\nexec "$@"\n' > "$repo/research/scripts/stub-memory-cap.sh"; fi
  git -C "$repo" add -A; git -C "$repo" commit -qm base; echo 'fn b() {}' >> "$repo/crates/lib.rs"; git -C "$repo" add crates/lib.rs
  tmp_before="$(find "$draft/g1f-tmp" -maxdepth 1 -mindepth 1 2>/dev/null | wc -l)"; mkdir -p "$draft/g1f-tmp"
  ( cd "$repo" && TMPDIR="$draft/g1f-tmp" bash -c "$first_two"$'\n'"$third" ) > /dev/null 2>&1; rc=$?
  printf '%s\t改后那一次调用的退出码：%s\t剩下的临时目录：%s\t剩下的 worktree 数（含主树）：%s\n' "$outcome" "$rc" "$(find "$draft/g1f-tmp" -maxdepth 1 -mindepth 1 | wc -l)" "$(git -C "$repo" worktree list | wc -l)"
done
