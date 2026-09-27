#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 每个实验二进制都要有钉绝对值的断言（research/e7-index-bench/src/bin 下全部，别处 src/bin 下以 e<数字>_ 开头的；其余成功行逐个列名）
#
# 还 test-discipline.md 的这一条：
# 「只让多条臂互相比，测不出『所有臂一起错』……每一条互比断言旁边，
#   必须有一条把绝对值钉死的断言。」
#
# **判据只做机器判得了的那一半**：这个实验有没有**任何一条**把某个量与数字字面量比死的断言。
# 判不了的那一半（钉的是不是对的那个量、绝对值是不是独立算出来的）留给人。
#
# ⚠️ **零绝对值断言是一个可判定的、且实测出过问题的形态**（2026-08-29 对抗验证）：
# `e9_keylayout` 与 `e21_cpu` 当时各是 0 条，七个 / 两个单测钉的全是结构性质。
# e9 支撑 D8（核心索引结构） 的 key 布局（1.46×），e21 支撑 D24（后台重活能不能卸给 GPU）
# 的跨语言比值——两个都是承重结论，而它们量出来的那个数没有任何东西钉。
#
# 判别力已证（2026-08-29）：把 e9 的四条绝对值断言注释掉 ⇒ 本阶段判红。
#
# 射程：research/e7-index-bench/src/bin 下的每一份，加上别处 `crates/*/src/bin`、`research/*/src/bin` 下文件名以 `e<数字>_` 开头的
# （实验编号的写法，`.claude/abbreviations` 登记的 `e<数字>`）——按「是不是实验」认，不按住在哪个目录认：
# 住在 crates/ 下的实验与 research/ 下的同规矩。别处不以 `e<数字>_` 开头的是装置工具（例：拿设备日志逐项比 ground truth 的），
# 不判，成功行逐个列名，清单现算；research/prompts/ 下腿的模型是冻结证据，不算实验二进制，不列。
# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T4。
#
# ⚠️ **断言要按语句认，不按行认**（2026-09-25 修）：rustfmt 会把长断言拆成多行——
# `assert!(` / `assert_eq!(` 单独一行起，中间的比较式、消息各占一行，`);` 单独一行收尾。
# 逐行 grep 只在断言仍是单行时管用，拆成多行之后同一条断言在任何一行上都凑不齐
# 「宏名 + 数字字面量 + 收尾括号」，会被误判成零绝对值断言（实测：
# `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` 被 rustfmt 拆行后，
# 本阶段判它一条都没有；那份文件里的三条 `assert!(X == 字面量, "…")` 其实都在）。
# 判据本身不改，只改「怎么认出一条断言」：从 `assert(_eq)?!(` 起用圆括号配平找到语句收尾的那个 `)`，
# 拼成一条逻辑行再套判据；拼之前先挖掉字符串字面量的内容与行注释，
# 免得消息文本里的括号、或分号后的注释，把配平或判据算乱。
# 样本：fixtures/80-absolute-assertions.sh/red 放一份只比相对值的（e7 目录，单行）、一份住在 crates/ 下、
# 以 e<数字>_ 开头、只比相对值的（单行），再加一份 e7 目录下拆成多行、但比的是字符串的（证明多行不会被误判成钉了绝对值）；
# 三份都要点名判红。green 除了两份单行钉了绝对值的（e7 目录一份、crates/ 一份），
# 再各加一份拆成多行、真钉了数字的（e7 目录用 `assert_eq!`、crates/ 用 `assert!` 带消息，照 E142 那三条的样子）、判绿；
# 另放一份不以 e<数字>_ 开头的 crates/demo/src/bin/probe.rs，成功行要把它列成没判的。
#
#   bash .claude/gate.d/80-absolute-assertions.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
BINS=research/e7-index-bench/src/bin
# $BINS 不在不等于无对象：住在 crates/ 下、以 e<数字>_ 开头的实验照判；两处都一份没有，才在末尾退 77

# 绝对值断言：与数字字面量比死。两种形态都认，判据不变（见文件头的多行修法）。
# assert! 那一支只认 ==、<=、>=、<、>，而且运算符前一个字不能是 !、=、<、>：`!= 0` 是结构性质（非空、非零），不是把量钉死在一个数上。
ABSOLUTE_ASSERTION_PATTERN='assert_eq!\([^;]*, *-?[0-9][0-9_]*(\.[0-9]+)?\)|assert!\([^;]*[^!<>=](==|<=|>=|<|>) *-?[0-9][0-9_]*(\.[0-9]+)?[,)]'

# 把一个文件里被拆成多行的 assert!()/assert_eq!() 拼回一条条完整语句，逐条打到 stdout。
# 用圆括号配平判定语句收尾：从匹配到的 `assert(_eq)?!(` 起累计括号深度，深度回到 0 就是这一条的收尾。
# 拼之前用 strip() 把字符串字面量整体换成 `""`、行注释砍掉，配平只数真代码里的括号。
# 每行先掐掉两端空白再拼接、不额外插分隔符：判据要求「数字紧跟收尾括号」，
# rustfmt 常把收尾的 `);` 单独放一行、前面缩进一截——按原样拼会在数字和 `)` 之间垫出空白，
# 反而把这一条断言拼成判据认不出的样子；逐行掐两端空白再首尾相接，才是单行写法本来的样子。
join_assertion_statements() {
  awk '
    function trim(s) {
      gsub(/^[ \t]+|[ \t]+$/, "", s)
      return s
    }
    function strip(line,    s) {
      s = line
      gsub(/"([^"\\]|\\.)*"/, "\"\"", s)
      sub(/\/\/.*/, "", s)
      return s
    }
    function paren_delta(s,    i, c, d) {
      d = 0
      for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "(") d++
        else if (c == ")") d--
      }
      return d
    }
    BEGIN { in_statement = 0; depth = 0; buffer = "" }
    {
      raw_line = $0
      if (!in_statement) {
        # 起点在挖掉字符串与行注释之后的那一行上找：注释掉的 `// assert_eq!(x, 5);`、字符串里的「assert!(」都不是一条断言
        code_line = strip(raw_line)
        if (!match(code_line, /assert(_eq)?!\(/)) next
        piece = trim(substr(code_line, RSTART))
        buffer = piece
        depth = paren_delta(piece)
        in_statement = 1
      } else {
        # 判据套在挖掉字符串与行注释之后的语句上：消息文本里的「, 5)」不算把量钉死
        piece = trim(strip(raw_line))
        buffer = buffer piece
        depth += paren_delta(piece)
      }
      if (depth <= 0) {
        print buffer
        in_statement = 0
        buffer = ""
        depth = 0
      }
    }
  ' "$1"
}

judged=() uncovered=()
for f in "$BINS"/*.rs; do
  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
done
in_bins=${#judged[@]}
for directory in crates/*/src/bin research/*/src/bin; do
  [[ -d "$directory" && "$directory" != "$BINS" ]] || continue
  for f in "$directory"/*.rs; do
    [[ -f "$f" ]] || continue
    if [[ "$(basename "$f")" =~ ^e[0-9]+_ ]]; then judged+=("$f"); else uncovered+=("$f"); fi
  done
done
bad=0; n=0
for f in "${judged[@]}"; do
  n=$((n+1))
  c=$(join_assertion_statements "$f" | grep -cE "$ABSOLUTE_ASSERTION_PATTERN")
  if (( c == 0 )); then
    echo "  ✗ $f 一条绝对值断言都没有"
    bad=$((bad+1))
  fi
done

if ((bad)); then
  echo "     → 怎么办：给它加一条把**被量的那个数**钉死的断言，绝对值要由**独立算术**给出，"
  echo "               不许从代码里读回来。加完用 research/scripts/mutate.sh 证明它会红——"
  echo "               实测教训：先加的断言可能一条变异都拦不住（E9 踩过），只有变异测试分得开。"
  exit 1
fi
((n)) || { echo "  ! $BINS 下一个 .rs 都没有、别处也没有 e<数字>_ 开头的，本阶段无对象可判"; exit 77; }
echo "  ✓ $n 个实验二进制各自至少有一条绝对值断言（$BINS 下 $in_bins 份，别处 src/bin 下以 e<数字>_ 开头的 $(( n - in_bins )) 份）"
echo "    没判的 ${#uncovered[@]} 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）："
for f in "${uncovered[@]}"; do echo "      $f"; done
