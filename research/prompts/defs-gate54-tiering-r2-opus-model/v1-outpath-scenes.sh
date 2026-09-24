#!/usr/bin/env bash
# V1：出路句那三行命令在「暂存区为空」「暂存区里只有别的会话的东西」「HEAD 变了」三种现场下各给出什么；$TMPDIR 留下的目录会不会被下一次读进来。
source "$(dirname "$0")/lib.sh"
outpath_file "$MODEL/inputs/54-post.sh" "$RUNS/outpath.sh"
printed_outpath() { # <54 号的输出文件> <写到哪>：从 54 号真打出来的红句里取出那三行
  grep -A3 '在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 --full' "$1" | tail -3 | sed -e 's/^ *在项目根、暂存之后：//' -e 's/^ *//' > "$2"
}

echo "════ V1-a 暂存区为空：代码轮刚改完 crates/、还没暂存，崩溃验证员照定义第 2 步在主工作区跑 54 号（默认快档）"
r="$RUNS/v1-a"; make_repo "$r" post new ""
set_crash "$r" c1 ""
echo "  ── 暂存区：[$(git -C "$r" diff --cached --name-only | tr '\n' ' ')] 工作区相对 HEAD：$(git -C "$r" diff --name-only | tr '\n' ' ')"
(cd "$r" && bash "$STAGE" > "$r.q1.out" 2>&1); echo "  ── 崩溃验证员的快档，退出码 $?"; grep -E '^  (✓|✗)' "$r.q1.out" | trim 150
printed_outpath "$r.q1.out" "$r.printed.sh"
if cmp -s "$r.printed.sh" "$RUNS/outpath.sh"; then echo "  ── 红句里打出的三行与 54 号出路函数的三行逐字相同（cmp 0）"; else echo "  ── 红句里打出的三行与出路函数不同："; diff "$r.printed.sh" "$RUNS/outpath.sh"; fi
(cd "$r" && bash "$r.printed.sh" > "$r.full.out" 2>&1); echo "  ── 照红句三行原样跑完"; grep -E '^  (· --full|✓ 全绿|✗)' "$r.full.out" | trim 150
cells "$r"
(cd "$r" && bash "$STAGE" > "$r.q2.out" 2>&1); echo "  ── 崩溃验证员再跑快档，退出码 $?"; grep -E '^  (✓|✗)' "$r.q2.out" | trim 150
echo "  ── gate --staged（暂存区为空）"; gate_staged_54 "$r"
echo

echo "════ V1-b 暂存区里只有别的会话的东西：X 这一批只改文档；Y 暂存了 crates/ 改动、还没跑它自己的 --full"
r="$RUNS/v1-b"; make_repo "$r" post new ""
printf 'x doc\n' >> "$r/docs/note.md"; git -C "$r" add docs/note.md
set_crash "$r" y1 ""; git -C "$r" add crates/singlefs-harness/src/crash.rs
echo "  ── 暂存区：$(git -C "$r" diff --cached --name-only | tr '\n' ' ')"
echo "  ── X 起门禁"; gate_staged_54 "$r"
(cd "$r" && bash "$RUNS/outpath.sh" > "$r.full.out" 2>&1); echo "  ── X 照出路三行跑"; grep -E '^  (✓ 全绿|✗)' "$r.full.out" | trim 150
echo "  ── X 再起门禁"; gate_staged_54 "$r"; echo "  ── 真值"; truth_full "$r"
echo

for style in whole-index own-paths; do
  echo "════ V1-c HEAD 变了：出路第 1 行跑完、第 2 行之前，别的会话提交（$style）"
  r="$RUNS/v1-c-$style"; make_repo "$r" post new ""
  set_crash "$r" mine ""; git -C "$r" add crates/singlefs-harness/src/crash.rs
  printf 'pub mod crash;\n// 别的会话的改动\n' > "$r/crates/singlefs-harness/src/lib.rs"; git -C "$r" add crates/singlefs-harness/src/lib.rs
  echo "  ── 暂存区：$(git -C "$r" diff --cached --name-only | tr '\n' ' ')"
  sed -n '1p' "$RUNS/outpath.sh" > "$r.line1"; sed -n '2,3p' "$RUNS/outpath.sh" > "$r.line23"
  # 三行放在同一个 shell 里跑（变量要跨行），第 1 行之后插进别的会话的提交
  if [[ "$style" == whole-index ]]; then other_commit="git commit -q -m other"; else other_commit="git commit -q -m other -- crates/singlefs-harness/src/lib.rs"; fi
  (cd "$r" && bash -c "$(cat "$r.line1"); $other_commit; echo \"  ── 插进来的提交之后 HEAD=\$(git rev-parse --short HEAD)，暂存区：\$(git diff --cached --name-only | tr '\n' ' ')\"; $(cat "$r.line23")" > "$r.full.out" 2>&1)
  grep -E '^  ── 插进来|error|^  (✓ 全绿|✗)|does not apply|already exists' "$r.full.out" | trim 150
  cells "$r"
  echo "  ── 起门禁"; gate_staged_54 "$r"; echo "  ── 真值"; truth_full "$r"
  echo
done

echo "════ V1-d \$TMPDIR 里留下的目录：同一条出路连跑两次，看第二次读没读第一次留下的东西"
r="$RUNS/v1-d"; make_repo "$r" post new ""
set_crash "$r" d1 ""; git -C "$r" add -A
before="$(ls -d "$TMPDIR"/tmp.* 2>/dev/null | sort)"
(cd "$r" && bash "$RUNS/outpath.sh" > "$r.full1.out" 2>&1)
mid="$(ls -d "$TMPDIR"/tmp.* 2>/dev/null | sort)"
set_crash "$r" d2 ""; git -C "$r" add -A
(cd "$r" && bash "$RUNS/outpath.sh" > "$r.full2.out" 2>&1)
after="$(ls -d "$TMPDIR"/tmp.* 2>/dev/null | sort)"
first_dir="$(comm -13 <(echo "$before") <(echo "$mid"))"; second_dir="$(comm -13 <(echo "$mid") <(echo "$after"))"
echo "  ── 第一次留下：$(basename "$first_dir")：$(ls -A "$first_dir" | tr '\n' ' ')；第二次留下：$(basename "$second_dir")：$(ls -A "$second_dir" | tr '\n' ' ')"
echo "  ── 两次的 staged.patch 相同吗：$(cmp -s "$first_dir/staged.patch" "$second_dir/staged.patch" && echo 相同 || echo 不同)；第二次那一趟的 judged_root：$(grep -o 'judged_root=.*' "$(common_dir "$r")"/singlefs-layer0-full-green.* 2>/dev/null | sed 's#.*/\(tmp\.[^/]*\)/tree#\1#' | tr '\n' ' ')"
echo "  ── worktree 登记：$(git -C "$r" worktree list | wc -l) 条（只剩主工作树）"
cells "$r"
