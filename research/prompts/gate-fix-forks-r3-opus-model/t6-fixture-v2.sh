#!/usr/bin/env bash
# T6 改法（本腿提的，只在这份模型上量过、被攻过零轮）：52 号红样本从五行扩成十行加两句提示，每条比对分支各有一处会红，
# expect 逐条点名；然后拿它重跑 t6-mutants.sh，看第二轮的 M1 与这一轮的 N1–N10 各被抓到没有。
# N10（声明的路径集合剩不下恰好一条）走的是「结构性错误、退 2」那一支，与逐行比对的错放不进同一份红样本——这一格不修，写明。
# 用法：bash t6-fixture-v2.sh <快照树> <真仓> <草稿目录>
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; D="$3"; HERE="$(cd "$(dirname "$0")" && pwd)"
R="$D/red-v2"; rm -rf "$D"; mkdir -p "$R/.claude/kb/layout" "$R/research/results" "$R/research/scripts" "$R/crates/demo/tests"
cp "$SNAP/.claude/gate.d/fixtures/52-segment-registry.sh/red/research/scripts/replay.sh" "$R/research/scripts/"
cat > "$R/.claude/kb/layout/01-first-txn.md" <<'MD'
## 八、合成登记表

十行加整条流（`path=a / b / c / f / g / d / e / m`）。每行埋一种错：段序列、种类、操作数、状态数、产物里没有这条 path、
没写段序列、产物这一行没有 kinds、钉住的用例文件不在、用例里没有那个函数、出处既无 path 也无用例；整条流的状态数写错；
第二条流的写数、数组、用例里的状态数各错一处。脚本每条比对分支都有一处会红。

| 路径 | 段序列 | 出处 | 层 0 |
|---|---|---|---|
| 甲路径 | `3+1`，3 次操作、5 个崩溃状态，种类 `[x×2]\|[y]` | 产物 `name=segments path=a` | 无 |
| 乙路径 | `1+1`，2 次操作、3 个崩溃状态，种类 `[x]\|[z]` | 产物 `name=segments path=b` | 无 |
| 丙路径 | `1+1`，3 次操作、3 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=c` | 无 |
| 丁路径 | `1+1`，2 次操作、4 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=d` | 无 |
| 戊路径 | `1+1`，2 次操作、3 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=e` | 无 |
| 己路径 | 两段各一次，2 次操作、3 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=f` | 无 |
| 庚路径 | `1+1`，2 次操作、3 个崩溃状态，种类 `[x]\|[y]` | 产物 `name=segments path=g` | 无 |
| 辛路径 | `1+1`，种类 `[x]\|[y]` | 活代码（`crates/demo/tests/gone.rs` 的 `pinned_sequence` 钉住） | 无 |
| 壬路径 | `1+1`，种类 `[x]\|[y]` | 活代码（`crates/demo/tests/pin.rs` 的 `renamed_sequence` 钉住） | 无 |
| 癸路径 | `1+1`，种类 `[x]\|[y]` | 一（出处没写产物也没写用例） | 无 |

⚠️ 整条流 `1+1`、4 个状态（种类 `[x]|[y]`）。

⚠️ 第二条流的段序列 `2+1`、4 次写、5 个状态，装置钉住：`crates/demo/tests/pin.rs`。

## 九、下一节
MD
{ printf 'E7RESULT name=segments path=a operations=3 segments=2+1 closed_form=5 kinds=[x×2]|[y]\n'
  for p in b c d f m; do printf 'E7RESULT name=segments path=%s operations=2 segments=1+1 closed_form=3 kinds=[x]|[y]\n' "$p"; done
  printf 'E7RESULT name=segments path=g operations=2 segments=1+1 closed_form=3\n'; } > "$R/research/results/syn.out"
printf 'fn other_sequence() {\n    let segments = [1, 2];\n    assert_eq!(segments.len(), 2);\n}\n' > "$R/crates/demo/tests/pin.rs"
# expect：先让快照树里的原脚本对这份样本跑一遍，确认退 1，再把每一处不符的原话逐条写成 want
out="$(python3 "$SNAP/research/scripts/check-segment-registry.py" --root "$R" 2>&1)"; rc=$?
echo "原脚本对改过的红样本：退 $rc"; sed 's/^/  /' <<<"$out"
{ echo 'exit=1'; grep -m1 -oE '对不上，共 [0-9]+ 处' <<<"$out" | sed 's/^/want=/'
  grep -E '^     [^→ ]' <<<"$out" | grep -v '产物是过期了\|再让 research' | sed -E 's/^ +//; s/^/want=/'; } > "$R/expect"
echo "expect $(grep -c '^want=' "$R/expect") 条 want"
echo
nice -n 19 bash "$HERE/t6-mutants.sh" "$SNAP" "$REPO" "$D/run" "$R"
