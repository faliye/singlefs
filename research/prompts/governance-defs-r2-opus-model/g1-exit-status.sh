#!/usr/bin/env bash
# 用法：bash g1-exit-status.sh <仓根> <草稿目录>
# 把 54 号 print_staged_worktree_full_commands 今天打印的三行原样取出，只把两个名字换成桩
# （54 号脚本 → stub-full.sh，内存包装 → stub-memory-cap.sh；换名是为了不让这份模型碰到层 0 与内存包装本身），
# 在一个临时仓里按 main-agent.md 的写法放进同一次 bash 调用，看三种结局下这次调用的退出码。
set -uo pipefail
root="$1"; draft="$2"
printed="$(bash -c "$(sed -n '/^print_staged_worktree_full_commands() {/,/^}/p' "$root/.claude/gate.d/54-layer0-replay.sh"); print_staged_worktree_full_commands")"
three_lines="$(printf '%s\n' "$printed" | sed -n '2,4p' | sed 's/^ *//' \
  | sed 's#\.claude/gate\.d/54-layer0-replay\.sh#.claude/gate.d/stub-full.sh#; s#research/scripts/run-with-memory-cap\.sh#research/scripts/stub-memory-cap.sh#')"
for outcome in green red memory-cap-250 scope-unavailable-251; do
  repo="$(mktemp -d "$draft/g1-repo-XXXXXX")"
  git -C "$repo" init -q; git -C "$repo" config user.email m@x; git -C "$repo" config user.name m
  mkdir -p "$repo/.claude/gate.d" "$repo/research/scripts" "$repo/crates"
  echo 'fn a() {}' > "$repo/crates/lib.rs"
  case "$outcome" in
    green) full_exit=0; cap_exit="" ;;
    red) full_exit=1; cap_exit="" ;;
    memory-cap-250) full_exit=0; cap_exit=250 ;;
    scope-unavailable-251) full_exit=0; cap_exit=251 ;;
  esac
  printf '#!/usr/bin/env bash\necho "stub-full 在 $2 上跑，退 %s"\nexit %s\n' "$full_exit" "$full_exit" > "$repo/.claude/gate.d/stub-full.sh"
  if [[ -n "$cap_exit" ]]; then
    printf '#!/usr/bin/env bash\necho "stub-memory-cap：退 %s，命令一行都没跑或被杀"\nexit %s\n' "$cap_exit" "$cap_exit" > "$repo/research/scripts/stub-memory-cap.sh"
  else
    printf '#!/usr/bin/env bash\nshift\nexec "$@"\n' > "$repo/research/scripts/stub-memory-cap.sh"
  fi
  git -C "$repo" add -A; git -C "$repo" commit -qm base
  echo 'fn b() {}' >> "$repo/crates/lib.rs"; git -C "$repo" add crates/lib.rs
  ( cd "$repo" && TMPDIR="$draft" bash -c "$three_lines" ) > "$draft/g1-$outcome.log" 2>&1
  call_exit=$?
  worktrees_left="$(git -C "$repo" worktree list | wc -l)"
  printf '%s\t这一次 Bash 调用的退出码：%s\t输出：%s\t剩下的 worktree 数（含主树）：%s\n' "$outcome" "$call_exit" "$(grep -v '^Preparing\|^HEAD is now' "$draft/g1-$outcome.log" | tr '\n' ' ')" "$worktrees_left"
done
