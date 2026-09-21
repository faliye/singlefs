#!/usr/bin/env bash
# gate-stage: kb 目录里的每一份都要在 CLAUDE.md 的「项目本地事实」表里有一行
#
# 判据：三条，任一条不成立判红。
#   ① `.claude/kb/` 根目录下每一份 `.md`，都要在 `CLAUDE.md` 的「## 项目本地事实」表里被按路径点名；
#   ② `.claude/kb/` 下每一个子目录也要被点名（形态是带斜杠的路径，例如 `.claude/kb/decisions/`）；
#   ③ 反过来，那张表里点到的每一个 `.claude/kb/…` 路径都要真的存在。
#
# 为什么：那张表是 kb 各文件职责的唯一登记位——一份文件是什么、归谁读、与别处什么关系，只写在那里一行。
# 新建一份 kb 文件而不登记，它对别的会话与派出去的 agent 就是个来历不明的东西；删掉一份而不撤行，
# 表就指向空处。门禁 62 号早就在盯 `stage-owners.tsv` 与 `.claude/gate.d/` 逐项一致，kb 这一侧一直没有对应的闸。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：建 `.claude/kb/feature-bits.md` 时漏了登记，一查那张表还同时漏着
# `INDEX.md`、`term-renames.md`、`tooling.md` 三份——四份文件没有任何东西说得出它们是什么。
#
# ⚠️ 射程：判的是「登记了没有」，判不了那一行写得对不对——后者要人看。
# `decisions-history/` 这类由别的阶段管形状的子目录同样要有一行，表里写它与谁同进退。
#
# 判别力：fixtures/98-kb-registry.sh/red 放一份没登记的 kb 文件与一行指向空处的登记，必须判红；green 两边对齐。
#
#   bash .claude/gate.d/98-kb-registry.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - <<'PY'
import glob, os, re, sys

KB = ".claude/kb"
MANIFEST = "CLAUDE.md"
SECTION = "## 项目本地事实"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

if not os.path.isdir(KB):
    print(f"  ! 没有 {KB}，本阶段无对象可判")
    sys.exit(77)
if not os.path.isfile(MANIFEST):
    fail(f"找不到 {MANIFEST}", ["怎么办：项目说明改了名就同步改这个阶段里的路径。"])

text = open(MANIFEST, encoding="utf-8").read()
start = text.find(SECTION)
if start < 0:
    fail(f"{MANIFEST} 里没有「{SECTION}」这一节", [
        f"怎么办：这一节是 kb 各文件职责的登记位。改了标题就同步改这个阶段里的 SECTION，别让这道闸扫空。",
    ])
end = text.find("\n## ", start + len(SECTION))
section = text[start:end if end > 0 else len(text)]
registered = set(re.findall(r"`(\.claude/kb/[^`]*)`", section))
if not registered:
    fail(f"「{SECTION}」那一节里一个 `.claude/kb/…` 路径都没点到", [
        "怎么办：扫到 0 项而报绿，与判过了一模一样。那一节的表每行第一格写路径，用反引号包起来。",
    ])

on_disk = set()
for path in sorted(glob.glob(f"{KB}/*.md")):
    on_disk.add(path)
for entry in sorted(os.listdir(KB)):
    full = os.path.join(KB, entry)
    if os.path.isdir(full):
        on_disk.add(full + "/")

def is_registered(path):
    if path in registered:
        return True
    # 表里写成带月份占位的形态（`.claude/kb/decisions-history/<年-月>.md`）时，按目录前缀认
    return any(entry.startswith(path) or path.startswith(entry.rstrip("/") + "/") for entry in registered if entry != path)

missing = sorted(path for path in on_disk if not is_registered(path))
dangling = sorted(entry for entry in registered
                  if not os.path.exists(entry) and not os.path.exists(entry.rstrip("/"))
                  and "<" not in entry)

failed = False
if missing:
    failed = True
    print(f"  ✗ kb 里 {len(missing)} 份没有在 {MANIFEST} 的「{SECTION}」表里登记：")  # gate-lint:summary
    for path in missing:
        print(f"      {path}")  # gate-lint:detail
    print(f"     → 怎么办：往那张表加一行，第一格写路径（反引号包起来）、第二格写它是什么、归谁读、与别处什么关系。")
    print("               写不出那一行，多半说明这份文件该并进已有的某一份，或者压根不该单开。")
if dangling:
    failed = True
    print(f"  ✗ 那张表里 {len(dangling)} 个路径指向不存在的东西：")  # gate-lint:summary
    for entry in dangling:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：文件搬家或改名了就同步改那一行（.claude/rules/path-moves.md）；删掉了就把那一行撤掉——")
    print("               指向空处的登记比没有登记更糟，它让人以为那一格有人管着。")
if failed:
    sys.exit(1)

print(f"  ✓ kb 里 {len(on_disk)} 项（{len([p for p in on_disk if p.endswith('.md')])} 份 .md、"
      f"{len([p for p in on_disk if p.endswith('/')])} 个子目录）都在 {MANIFEST} 的「{SECTION}」表里登记着，"
      f"表里 {len(registered)} 个路径也都指得到东西")
PY
