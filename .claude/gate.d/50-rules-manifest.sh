#!/usr/bin/env bash
# gate-stage: 项目规则清单一致
#
# 还 checks-owed.md C4（项目规则清单不一致）。
# 判据：`.claude/rules/` 下的文件集合，必须与 CLAUDE.md 里 `@.claude/rules/` 的引用集合逐项相等。
# 任一侧多出一项即判红——多出的那一份规则**不会被读进上下文**，等于没写。
#
# ⚠️ 上游 manifest.sh 只覆盖 SOP 包自己的 CLAUDE.md + rules/，
# 项目本地那一份此前没有任何检查在看（C4 的原文）。
# 双向比对用共用库 lib-manifest.py，与 62、63、98 号同一份代码。没有 .claude/rules 或没有 CLAUDE.md ⇒ 无对象可判，退 77（不记通过）。
#
#   bash .claude/gate.d/50-rules-manifest.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - "$(cd "$(dirname "$0")" && pwd)/lib-manifest.py" <<'PY'
import fnmatch, importlib.util, os, re, sys

try:
    spec = importlib.util.spec_from_file_location("lib_manifest", sys.argv[1])
    manifest = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(manifest)
except OSError as error:
    print(f"  ✗ 读不到共用库 {sys.argv[1]}：{error}")
    print("     → 怎么办：目录与清单的双向比对只有那一份，恢复它，别在阶段里再抄一份。")
    sys.exit(1)

RULES = ".claude/rules"
MD = "CLAUDE.md"
if not os.path.isdir(RULES):
    print(f"  ! 没有 {RULES}，本阶段无对象可判")
    sys.exit(77)
if not os.path.isfile(MD):
    print(f"  ! 找不到 {MD}，本阶段跳过")
    sys.exit(77)

on_disk = sorted(name for name in os.listdir(RULES) if fnmatch.fnmatchcase(name, "*.md"))
with open(MD, encoding="utf-8", errors="replace") as handle:
    referenced = sorted({os.path.basename(match) for match in re.findall(r"@\.claude/rules/[A-Za-z0-9._-]+\.md", handle.read())})
missing, extra = manifest.two_way(on_disk, referenced)

if missing:
    print(f"  ✗ 这些规则文件存在，但 {MD} 没有 @ 引用它们——不会被读进上下文，等于没写：")   # gate-lint:detail
    for name in missing:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if extra:
    print(f"  ✗ {MD} 引用了这些规则，但文件不存在——引用悬空：")
    for name in extra:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if missing or extra:
    print("     → 怎么办：补上缺的 @ 引用，或删掉悬空引用；两侧必须逐项相等。")
    sys.exit(1)
print(f"  ✓ 项目规则清单一致（{len(on_disk)} 条，与 {MD} 的引用逐项相符）")
PY
