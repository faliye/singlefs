#!/usr/bin/env bash
# agent-defs-r2 云端攻方腿模型二：kb-scribe 第 2 步「49 号 --write 跑前跑后各比一次，改到这一轮之外的条目就停」
# ① 只写这一轮自己的一条（带快查）：--write 之后 decisions-history.md 里被挤掉的旧行属于这一轮之外的条目；
# ② 再放一条别的会话刚写、还没写快查的条目：--write 先把「（待补）」写进那条，比对只能在写完之后看见。
# 只在临时目录里拷 .claude/kb 与 49 号脚本；不碰仓里任何文件，不做 git 操作（用 diff 比跑前跑后两份快照）。
#   TMPDIR=<草稿目录> bash history-write-diff.sh
set -uo pipefail
REPO="${REPO:-/home/fy5090/code/singlefs}"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK:?}"' EXIT

prepare() {  # $1 目录
  mkdir -p "$1/.claude/gate.d"
  cp -a "$REPO/.claude/kb" "$1/.claude/kb"
  cp "$REPO/.claude/gate.d/49-history-brief.sh" "$REPO/.claude/gate.d/lib-history-brief.py" "$1/.claude/gate.d/"
}
insert_after_anchor() {  # $1 月文件  $2 要插的文本文件
  python3 - "$1" "$2" <<'PY'
import sys
path, extra = sys.argv[1], open(sys.argv[2], encoding="utf-8").read()
text = open(path, encoding="utf-8").read()
anchor = "\n## 历史版本\n\n"
at = text.index(anchor) + len(anchor)
open(path, "w", encoding="utf-8").write(text[:at] + extra + text[at:])
PY
}
cat > "$WORK/mine.txt" <<'T'
### 2026-09-17（其九十）：D22（单元原子性怎么合成） 攻方模型的虚构条目（这一轮自己的）

> 快查·改前：模型里的改前。
>
> 快查·改后：模型里的改后。

- **改前**：模型里的改前。
- **改后**：模型里的改后。
- **依据**：模型。

T
cat > "$WORK/foreign.txt" <<'T'
### 2026-09-17（其九十一）：D23（journal 的角色与格式） 别的会话刚写下、快查还没补的虚构条目

- **改前**：甲。
- **改后**：乙。
- **依据**：丙。

T

run_case() {  # $1 名字  $2 放不放别的会话的条目 yes|no
  local name="$1" foreign="$2" dir month rc
  dir="$WORK/$name"; prepare "$dir"; month="$dir/.claude/kb/decisions-history/2026-09.md"
  [[ "$foreign" == yes ]] && insert_after_anchor "$month" "$WORK/foreign.txt"
  insert_after_anchor "$month" "$WORK/mine.txt"
  cp "$month" "$dir/month.before"; cp "$dir/.claude/kb/decisions-history.md" "$dir/main.before"
  ( cd "$dir" && bash .claude/gate.d/49-history-brief.sh --write >"$dir/write.log" 2>&1 ); rc=$?
  local placeholders_in_foreign removed_rows removed_rows_not_mine added_rows_foreign
  placeholders_in_foreign="$(diff "$dir/month.before" "$month" | grep -c '^> > 快查·.*（待补）')"
  removed_rows="$(diff "$dir/main.before" "$dir/.claude/kb/decisions-history.md" | grep -c '^< | ')"
  removed_rows_not_mine="$(diff "$dir/main.before" "$dir/.claude/kb/decisions-history.md" | grep '^< | ' | grep -vc '攻方模型的虚构条目')"
  added_rows_foreign="$(diff "$dir/main.before" "$dir/.claude/kb/decisions-history.md" | grep '^> | ' | grep -c '别的会话刚写下')"
  echo "name=case case=$name foreign_entry=$foreign write_rc=$rc placeholders_written_into_foreign_entry=$placeholders_in_foreign card_rows_removed=$removed_rows card_rows_removed_not_this_round=$removed_rows_not_mine card_rows_added_for_foreign_entry=$added_rows_foreign write_log=$(tail -1 "$dir/write.log" | tr -s ' ' | tr ' ' '_')"
}
run_case W1_only_this_round_entry no
run_case W2_plus_foreign_entry_without_quick yes
echo "name=done cases=2"
