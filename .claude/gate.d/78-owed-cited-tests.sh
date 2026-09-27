#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 已还清的欠账行里点名的测试名，仓里还在不在
#
# 判据：`.claude/kb/checks-owed.md` 的「### 已还清」那张表里，反引号括起来的 snake_case 标识符
# （形如 `a_b_c`，至少三段）必须在仓里的 `.rs` / `.py` / `.sh` 的代码里按整词找得到；找不到就红。
# 按整词（`a_b_c_v2` 不算 `a_b_c` 还在）、只认注释之外（`.rs` 的 `//` 之后、`.py` / `.sh` 的 `#` 之后不算：
# 旧名只剩在一句注释里，那个测试其实已经没了）。已还清表一行都没有、或行里一个这样的标识符都没有，本次无对象可判，退 77。
# 一条已还清的行说「某某单测钉着这件事」，那个单测正是这条定案**唯一的记录位**——它被改名或删掉时，
# 今天没有任何东西会说话。
#
# 射程只到「已还清」那张表。**欠着的那张表不查**：那张表写的正是「知道要拦什么但还拦不了」，
# 里面点名一个还不存在的测试名是它应有的样子，拿这条判据去扫它一定是错的。
#
# 为什么只认「至少三段」的 snake_case：2026-09-20 在真语料上量过假阳性——已还清表 44 行、
# 命中 6 个标识符、仓里找不到的 1 个，而那 1 个正是要拦的那种（C313 行引的那条单测名，
# E142 第六次跑之后状态数变了、单测跟着改了名，这条已还清的行没跟上）。
# 放宽到两段会把 `log_append`、`write_cache` 这类词组也扫进来。
#
# ⚠️ 注释里不写被拦的那个标识符的整串字面量，搜索也排掉 `.claude/gate.d/`：第一版在注释里写了，
# 于是 grep 扫到这个阶段自己、把该红的那一条判成「还在」（与 `pgrep -f` 匹配到自己的命令行同形）。
# 门禁脚本里出现一个测试名，本来也不构成「这个测试存在」的证据——测试住在 crates/ 与 research/。
#
# 不扫 `research/prompts/`：当时原样发给模型的材料是冻结证据，里面引的旧名字本来就不该跟着改
# （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」）。
#
#   bash .claude/gate.d/78-owed-cited-tests.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }

OWED=".claude/kb/checks-owed.md"
if [[ ! -f "$OWED" ]]; then
  echo "  ! 没有 $OWED，本阶段跳过"
  exit 77
fi

python3 - "$OWED" <<'PY'
import pathlib, re, subprocess, sys

owed = pathlib.Path(sys.argv[1])
lines = owed.read_text(encoding="utf-8").splitlines()

settled_heading = None
for index, line in enumerate(lines):
    if line.strip().startswith("### 已还清"):
        settled_heading = index
        break
if settled_heading is None:
    print("  ✗ 找不到「### 已还清」那一节，判不了已还清的行引的测试名还在不在")
    print("     → 怎么办：确认 .claude/kb/checks-owed.md 里那一节的标题没被改写；改了标题就把这一阶段的取法一起改。")
    raise SystemExit(1)

identifier = re.compile(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+){2,})`")
row_number = re.compile(r"^\|\s*(C\d+)\s*\|")

rows = []
for line in lines[settled_heading + 1:]:
    stripped = line.strip()
    if stripped.startswith("## ") or (stripped.startswith("### ") and not stripped.startswith("### 已还清")):
        break
    # 只认首格是 `C<数字>` 的登记行。表头（真表写 `| # | 简称 | …`）与分隔行不是登记行：
    # 数进去会让成功句里报的行数比真实行数多，而那个数没有任何东西钉着
    # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「成功那句里报的数也要有东西钉住」）。
    # 一行掉了编号算表的形状坏了，那是门禁 43 号判的事，这一阶段不重复判。
    if stripped.startswith("|") and stripped.count("|") >= 4 and row_number.match(stripped):
        rows.append(stripped)

cited = []
skipped = []
for row in rows:
    number = row_number.match(row).group(1)
    names = list(dict.fromkeys(identifier.findall(row)))  # 同一行里引两次算一个
    if names:
        for name in names:
            cited.append((number, name))
    else:
        skipped.append(number)

if not rows:
    print("  ⊘ 本次无对象可判：「### 已还清」那张表里一行首格是 C<编号> 的登记行都没有")
    raise SystemExit(77)
if not cited:
    print(f"  ⊘ 本次无对象可判：已还清的 {len(rows)} 行里没有一个至少三段的 snake_case 标识符："
          + "、".join(skipped))
    raise SystemExit(77)

def code_part(line, suffix):
    """去掉注释之后的那一截：.rs 砍掉 `//` 之后，.py / .sh 砍掉行首或空白之后的 `#` 之后。"""
    if suffix == ".rs":
        return line.split("//", 1)[0]
    return re.split(r"(?:^|\s)#", line, maxsplit=1)[0]

def is_in_the_repository(name):
    # grep 只用来缩小候选（按字面、按整词）；注释里的命中在下面逐行剔掉
    found = subprocess.run(
        ["grep", "-rnwF", "--include=*.rs", "--include=*.py", "--include=*.sh",
         "--exclude-dir=prompts", "--exclude-dir=target", "--exclude-dir=.git",
         "--exclude-dir=gate.d", "--", name, "."],
        capture_output=True, text=True)
    if found.returncode not in (0, 1):
        print(f"  ✗ grep 找 `{name}` 时出错（退 {found.returncode}）：{found.stderr.strip()[:120]}")
        print("     → 怎么办：按 grep 的报错修好（读不了的目录、权限），再跑；找的时候出错不等于没找到，也不等于找到了。")
        raise SystemExit(1)
    word = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])")
    for hit in found.stdout.splitlines():
        path, _, rest = hit.partition(":")
        _, _, text = rest.partition(":")
        if word.search(code_part(text, pathlib.Path(path).suffix)):
            return True
    return False

missing = [(number, name) for number, name in cited if not is_in_the_repository(name)]

if missing:
    for number, name in missing:
        print(f"  ✗ {number} 引的 `{name}` 在仓里的 .rs / .py / .sh 里一处都找不到")  # gate-lint:detail
    print(f"  ✗ 已还清的行里有 {len(missing)} 个点名的标识符已经不在仓里了")  # gate-lint:summary
    print("     → 怎么办：那条欠账是靠这个测试还清的。先 grep 它今天叫什么（多半是改了名），")
    print("               用 research/scripts/replace-once.py 把行里的旧名字定点换成现在的名字；")
    print("               它要是被删掉了，那这条欠账并没有还清，把整行挪回欠着的那张表。")
    raise SystemExit(1)

print(f"  ✓ 已还清的行引的测试名都还在（{len(rows)} 行，查了 {len(cited)} 个标识符："
      + "、".join(f"{number} {name}" for number, name in cited) + "）")
print(f"     没查的 {len(skipped)} 行（行里没有至少三段的 snake_case 标识符）："
      + "、".join(skipped))
PY
