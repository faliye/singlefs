#!/usr/bin/env bash
# gate-stage: 阶段头部声称的判别力样本必须真的存在
#
# 判据：三条，任一条不成立判红。
#   ① 一个阶段的头部注释里写了 `fixtures/<它自己的文件名>`，那个样本目录就必须存在；
#   ② `.claude/gate.d/fixtures/` 下的每个目录都要对应一个存在的阶段文件——孤儿样本目录没有任何东西会跑它，
#      而目录摆在那里看着就像验过了；
#   ③ 样本目录里至少要有 `red` 或 `green` 之一，每个子目录都要有 `expect`：
#      空目录既骗过第 ① 条，又会被判别力自检整个跳过。
#
# 为什么：上游的判别力自检（`.claude/singlefs-ai-sop/scripts/stage-selftest.sh`）把没有样本的阶段
# 列成「未自检」，这很好——但它只看目录在不在，不看**阶段自己怎么说**。一个阶段的头部逐字写着
# 「判别力：fixtures/<自己>/red 是一个……必须判红；green ……必须判绿」，而那个目录压根不存在时，
# 读脚本的人以为验过了，自检的名单里那一行又只是一句轻描淡写的「未自检」，两边对不上没有人会发现。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：`91-archive-past-rounds.sh` 的头部就是这么写的，
# 而 `.claude/gate.d/fixtures/91-archive-past-rounds.sh/` 不存在——它立项那天起就没验过判别力。
#
# ⚠️ 射程：判的是「声称与目录对不对得上」，判不了样本本身有没有判别力——那一层归
# `stage-selftest.sh`（红样本真判红、绿样本真判绿）。**没有声称、也没有样本**的阶段这一道不判，
# 它们由自检的「未自检」名单管着。
#
# 判别力：fixtures/95-fixture-claims.sh/red 摆一个假的 gate.d，三条各犯一次，必须判红；green 三条都干净。
#
#   bash .claude/gate.d/95-fixture-claims.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - <<'PY'
import glob, os, re, sys

STAGE_DIR = ".claude/gate.d"
FIXTURE_DIR = os.path.join(STAGE_DIR, "fixtures")
stages = sorted(glob.glob(os.path.join(STAGE_DIR, "[0-9][0-9]-*.sh")))
if not stages:
    print(f"  ✗ {STAGE_DIR} 下一个 NN-*.sh 都没有")
    print("     → 怎么办：门禁目录搬了位置就同步改这个阶段里的路径；扫到 0 个阶段而报绿，与判过了一模一样。")
    sys.exit(1)

claimed_without_directory, hollow = [], []
claiming = 0
for stage in stages:
    name = os.path.basename(stage)
    head = open(stage, encoding="utf-8").read()
    claims = re.search(rf"fixtures/{re.escape(name)}", head) is not None
    directory = os.path.join(FIXTURE_DIR, name)
    if claims:
        claiming += 1
        if not os.path.isdir(directory):
            claimed_without_directory.append(f"{name}：头部写着 fixtures/{name}/…，而 {directory} 不存在")
            continue
    if not os.path.isdir(directory):
        continue
    kinds = [kind for kind in ("red", "green") if os.path.isdir(os.path.join(directory, kind))]
    if not kinds:
        hollow.append(f"{name}：{directory} 在，里面一个 red / green 都没有")
        continue
    for kind in kinds:
        if not os.path.isfile(os.path.join(directory, kind, "expect")):
            hollow.append(f"{name}：{directory}/{kind}/ 里没有 expect，判别力自检会整个跳过它")

orphans = []
if os.path.isdir(FIXTURE_DIR):
    for entry in sorted(os.listdir(FIXTURE_DIR)):
        if os.path.isdir(os.path.join(FIXTURE_DIR, entry)) and not os.path.isfile(os.path.join(STAGE_DIR, entry)):
            orphans.append(f"{entry}：样本目录在，而 {STAGE_DIR}/{entry} 不存在")

failed = False
if claimed_without_directory:
    failed = True
    print(f"  ✗ {len(claimed_without_directory)} 个阶段的头部声称有判别力样本，而目录不存在：")  # gate-lint:summary
    for entry in claimed_without_directory:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：两条出路，别两边都不做。① 把样本建起来（fixtures/<阶段名>/{red,green}/ 各一份加 expect），")
    print("               让那句声称变成真的；② 这个阶段确实装不进沙箱（要真起虚机、要 cargo 真跑），就把头部那句改成")
    print("               「本阶段没有 fixtures 样本：<为什么装不进>，判别力靠 <别的什么> 」——说清楚，别留一句对不上的声称。")
if orphans:
    failed = True
    print(f"  ✗ {len(orphans)} 个孤儿样本目录，没有对应的阶段：")  # gate-lint:summary
    for entry in orphans:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：阶段退役时把它的样本目录一起删掉；阶段改名了就把目录改成新名字——")
    print("               留着的目录没有任何东西会跑它，而它摆在那里看着就像那个阶段验过了。")
if hollow:
    failed = True
    print(f"  ✗ {len(hollow)} 个样本目录是空壳：")  # gate-lint:summary
    for entry in hollow:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：red 至少要有一条 want=（失败信息里必须出现的片段），green 写 exit=0；")
    print("               少了 expect，判别力自检会跳过那一份，而目录在、看着像验过了。")
if failed:
    sys.exit(1)

print(f"  ✓ 阶段头部声称的判别力样本都在（{len(stages)} 个阶段里 {claiming} 个声称有样本），"
      f"{FIXTURE_DIR} 下没有孤儿目录，也没有缺 expect 的空壳")
PY
