#!/usr/bin/env bash
# C439 第二轮辩方腿 K1：外壳 git write-tree 算出的树（tree28）与里层建出的临时树是不是同一棵。
# 两种落法各建一次，逐案对 tree28 比，并比两种落法在磁盘上的文件内容：
#   today  ：冻结副本 .claude/singlefs-ai-sop/scripts/gate.sh 第 56、59、87 行的做法——diff --cached --binary、worktree add HEAD、apply --index
#   geng2  ：庚二的落法——worktree add HEAD 之后 read-tree --reset -u <tree28>（被判的树就是外壳 write-tree 的那棵）
# 用法：bash k1-tree-identity.sh <工作目录（新建）>
# 每案一行：name=k1 case=<案> tree28=<..> today_tree=<..> today_tree_eq=yes|no|fail(..) geng2_tree=<..> geng2_tree_eq=yes|no|fail(..) files_eq=yes|no|na
#          today_judged_lacks=<被判的树比 tree28 少的路径（树对树的差）> today_shell_advances=yes|no（门禁收尾时外壳 gate-staged.sh 第 39 行比暂存树：此刻的 write-tree 等于开跑时那棵就前移）
#          today_leak=yes：被判的树缺了 tree28 里有的东西、外壳却照前移（staged-green 挂上没判过的树）；庚二的落法同样给 geng2_judged_lacks / geng2_shell_advances / geng2_leak
# 钩子的位置：m1 在外壳 write-tree 之后、里层 diff --cached 之前；m2 在 diff 之后、worktree add 之前；m3 在整趟判完之后、外壳收尾比树之前。
set -uo pipefail
WORK="$1"; mkdir -p "$WORK" || exit 2
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=m GIT_AUTHOR_EMAIL=m@example.invalid GIT_COMMITTER_NAME=m GIT_COMMITTER_EMAIL=m@example.invalid
export GIT_AUTHOR_DATE="2026-10-14T00:00:00+0900" GIT_COMMITTER_DATE="2026-10-14T00:00:00+0900"  # clock-times:allow 造历史用的固定提交时刻，是被测输入
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE

dump() {   # dump <目录>：文件相对路径、模式、内容摘要（符号链接写目标），不含 .git
  (cd "$1" && find . -path ./.git -prune -o \( -type f -o -type l \) -print | LC_ALL=C sort | while read -r f; do
    if [[ -L "$f" ]]; then printf 'L %s -> %s\n' "$f" "$(readlink "$f")"; else printf 'F %s %s %s\n' "$f" "$([[ -x "$f" ]] && echo x || echo -)" "$(sha256sum < "$f" | cut -c1-16)"; fi
  done)
}
fresh() {   # fresh <目录> <建仓函数>：两个提交（root、c1）之后跑建仓函数
  local d="$1" setup="$2"
  rm -rf "$d"; mkdir -p "$d"; git init -q -b master --template= "$d"
  ( cd "$d" && printf 'base\n' > base.txt && printf 'two\n' > two.txt && git add -A && git commit -q -m root \
    && printf 'c1\n' >> base.txt && git add -A && git commit -q -m c1 && "$setup" "$d" )
}
land_today() {   # land_today <仓> <目标目录> <m1> <m2>
  local R="$1" wt="$2" m1="$3" m2="$4"
  [[ -n "$m1" ]] && "$m1" "$R"
  git -C "$R" diff --cached --binary > "$R/../p.patch" || return 3
  [[ -n "$m2" ]] && "$m2" "$R"
  git -C "$R" worktree add -q --detach "$wt" HEAD >/dev/null 2>&1 || return 3
  if [[ -s "$R/../p.patch" ]]; then git -C "$wt" apply --index "$R/../p.patch" 2>"$R/../apply.err" || return 4; fi
  git -C "$wt" write-tree
}
land_geng2() {   # land_geng2 <仓> <目标目录> <tree28> <m1> <m2>
  local R="$1" wt="$2" t28="$3" m1="$4" m2="$5"
  [[ -n "$m1" ]] && "$m1" "$R"
  [[ -n "$m2" ]] && "$m2" "$R"
  git -C "$R" worktree add -q --detach "$wt" HEAD >/dev/null 2>&1 || return 3
  git -C "$wt" read-tree --reset -u "$t28" || return 4
  git -C "$wt" write-tree
}
run_case() {   # run_case <案名> <建仓函数> <m1> <m2> <m3>
  local name="$1" setup="$2" m1="${3:-}" m2="${4:-}" m3="${5:-}" lacks1 lacks2 adv1 adv2 leak1 leak2 now1 now2 A="$WORK/$1/a/repo" B="$WORK/$1/b/repo" tA tB tt tg rt rg eq1 eq2 feq
  fresh "$A" "$setup"; fresh "$B" "$setup"
  tA="$(git -C "$A" write-tree 2>/dev/null)" || { echo "name=k1 case=$name write-tree=fail(外壳第 28 行就退 2，不进里层)"; return; }
  tB="$(git -C "$B" write-tree)"
  [[ "$tA" == "$tB" ]] || echo "name=k1 case=$name 警告=两份仓的 tree28 不同"
  tt="$(land_today "$A" "$WORK/$name/a/wt" "$m1" "$m2")"; rt=$?
  if [[ $rt -eq 0 ]]; then eq1=$([[ "$tt" == "$tA" ]] && echo yes || echo no); else eq1="fail(rc=$rt;$(head -c 70 "$WORK/$name/a/apply.err" 2>/dev/null | tr '\n' ' '))"; fi
  tg="$(land_geng2 "$B" "$WORK/$name/b/wt" "$tB" "$m1" "$m2")"; rg=$?
  if [[ $rg -eq 0 ]]; then eq2=$([[ "$tg" == "$tB" ]] && echo yes || echo no); else eq2="fail(rc=$rg)"; fi
  if [[ $rt -eq 0 && $rg -eq 0 ]]; then
    [[ "$(dump "$WORK/$name/a/wt")" == "$(dump "$WORK/$name/b/wt")" ]] && feq=yes || feq=no
  else feq=na; fi
  [[ -n "$m3" ]] && { "$m3" "$A"; "$m3" "$B"; }
  now1="$(git -C "$A" write-tree)"; now2="$(git -C "$B" write-tree)"
  adv1=$([[ "$now1" == "$tA" ]] && echo yes || echo no); adv2=$([[ "$now2" == "$tB" ]] && echo yes || echo no)
  lacks1=none; lacks2=none; leak1=no; leak2=no
  if [[ $rt -eq 0 ]]; then
    lacks1="$(git -C "$A" diff-tree -r --name-only --diff-filter=AMT "$tt" "$tA" | tr '\n' ',' | sed 's/,$//')"; lacks1="${lacks1:-none}"   # tree28 里有、而被判的树里没有或内容不同的路径（被判的树多出来的路径不算：多判不算漏）
    [[ "$lacks1" != none && "$adv1" == yes ]] && leak1=yes
  fi
  if [[ $rg -eq 0 ]]; then
    lacks2="$(git -C "$B" diff-tree -r --name-only --diff-filter=AMT "$tg" "$tB" | tr '\n' ',' | sed 's/,$//')"; lacks2="${lacks2:-none}"
    [[ "$lacks2" != none && "$adv2" == yes ]] && leak2=yes
  fi
  echo "name=k1 case=$name tree28=${tA:0:12} today_tree=${tt:0:12} today_tree_eq=$eq1 geng2_tree=${tg:0:12} geng2_tree_eq=$eq2 files_eq=$feq today_judged_lacks=$lacks1 today_shell_advances=$adv1 today_leak=$leak1 geng2_judged_lacks=$lacks2 geng2_shell_advances=$adv2 geng2_leak=$leak2"
}

# ── 静态案：没有并发，只看两种落法对各种文件形态是不是同一棵 ──
s_plain()    { printf 'x\n' >> two.txt; printf 'new\n' > new.txt; git rm -q base.txt; git add -A; }
s_mode_link(){ printf '#!/bin/sh\n' > run.sh; git add run.sh; git commit -q -m r; chmod +x run.sh; ln -s two.txt link; git add -A; rm two.txt; ln -s base.txt two.txt; git add -A; }
s_binary()   { python3 -c 'import sys; sys.stdout.buffer.write(bytes(range(256)) * 16)' > bin.dat; git add bin.dat; git commit -q -m b
               python3 -c 'import sys; sys.stdout.buffer.write(bytes(reversed(range(256))) * 16)' > bin.dat; python3 -c 'import sys; sys.stdout.buffer.write(bytes(range(0, 256, 3)) * 7)' > bin2.dat; git add -A; }
s_gitlink()  { git update-index --add --cacheinfo "160000,$(git rev-parse HEAD),sub"; }
s_filter()   { git config filter.up.clean 'tr a-z A-Z'; git config filter.up.smudge 'tr A-Z a-z'; printf '*.flt filter=up\n' > .gitattributes; git add .gitattributes; git commit -q -m attr; printf 'hello\n' > a.flt; git add a.flt; }
s_eol()      { git config core.autocrlf false; printf '*.crl text eol=crlf\n' > .gitattributes; git add .gitattributes; git commit -q -m attr; printf 'line1\r\nline2\r\n' > a.crl; git add a.crl; }
s_ita()      { printf 'ita\n' > ita.txt; git add -N ita.txt; }
s_typechange(){ git rm -q two.txt; mkdir two.txt; printf 'in\n' > two.txt/f; git add -A; }
s_rename()   { git mv two.txt moved.txt; printf 'edited\n' >> moved.txt; git add -A; }
s_empty()    { : > empty.txt; git add empty.txt; }
s_space()    { mkdir -p "目录 一"; printf 'z\n' > "目录 一/文件 二.md"; git add -A; }
s_conflict() { git checkout -q -b side; printf 's\n' > two.txt; git commit -qam s; git checkout -q master; printf 'm\n' > two.txt; git commit -qam m; git merge side >/dev/null 2>&1 || true; }
# ── 并发案：设置 + 两个钩子 ──
s_stage_two(){ printf 'A\n' >> two.txt; git add two.txt; }
m_extra_stage(){ ( cd "$1" && printf 'b\n' > extra.txt && git add extra.txt ); }
m_unstage_two(){ git -C "$1" restore -q --staged two.txt; }
m_unstage_readd(){ git -C "$1" restore -q --staged two.txt; git -C "$1" add two.txt; }
m_commit_all(){ git -C "$1" commit -q -m other; }
m_commit_unrelated(){ ( cd "$1" && printf 'u\n' > extra.txt && git add extra.txt && git commit -q -m unrelated -- extra.txt ); }
m_soft_reset(){ git -C "$1" reset -q --soft HEAD~1; }
m_readd_two(){ git -C "$1" add two.txt; }

echo "== 静态案"
for c in plain mode_link binary gitlink filter eol ita typechange rename empty space conflict; do run_case "static-$c" "s_$c"; done
echo "== 并发案（钩子位置见文件头）"
run_case race-extra-stage-before-diff       s_stage_two m_extra_stage ""
run_case race-unstage-before-diff           s_stage_two m_unstage_two ""
run_case race-unstage-readd-before-diff     s_stage_two m_unstage_readd ""
run_case race-commit-all-before-diff        s_stage_two m_commit_all ""
run_case race-commit-all-before-worktree    s_stage_two "" m_commit_all
run_case race-commit-unrelated-before-worktree s_stage_two "" m_commit_unrelated
run_case race-soft-reset-before-worktree    s_stage_two "" m_soft_reset
run_case race-extra-stage-before-worktree   s_stage_two "" m_extra_stage
run_case race-unstage-then-readd-after-judge s_stage_two m_unstage_two "" m_readd_two
