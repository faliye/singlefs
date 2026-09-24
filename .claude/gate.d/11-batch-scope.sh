#!/usr/bin/env bash
# gate-stage: 这一批要碰的触发文件有没有登记进范围
#
# 为什么：门禁 68 号的阶段同步按**触发文件**算范围——每多碰一个触发文件，事实表就多一行、候选表多几十到
# 几百行、逐行判与主 agent 的逐行全看跟着涨。而「顺手把踩过的那个脚本修一下」的 diff 只有几行，这个放大器
# 在判「它挡不挡着本轮的课题」的时候看不见。2026-09-22 实测：C468（判错的原始证据不落盘） 本身只要 3 行候选，
# 同一批里顺手修的两个脚本贡献了 249 行候选，逐行判下来 249 行全是「不相干」，多花的挂钟以小时计。
# 写一句「先判阻塞」拦不住手，拦得住的是暂存之后当场红的一道闸（C472（暂存区里多出来的触发文件没人拦））。
#
# 改动范围：**还没进 HEAD 的**——工作区、暂存区与未跟踪文件，基准是 HEAD，不是 GATE_BASE
# （取法用共用库 research/scripts/changed-paths.sh 的 head 取法，与 56、68、69、97 号同一份代码）。
# 与 68 号的基准不同是有意的，两者回答的不是同一个问题：
#   68 号问「这一轮要回扫哪些」，一轮可以跨几个提交，范围取 GATE_BASE；
#   这一道问「这一次提交要带哪些」，那就只能是还没进 HEAD 的那几个。
# 取 GATE_BASE 会把**上一个提交**的触发文件也算进来（gate-ok 不存在时基准退回 HEAD~1），
# 而那些属于上一批、不在这一批的登记里，每次提交之后必然误红——与判据 ⑤ 躲开的是同一格（C450）。
#
# 判据：
#   ① 触发文件：改动范围里命中 .claude/gate.d/knowledge-sync-triggers.tsv 里任一条正则的路径。
#      那份表是触发文件的唯一登记位，68 号读同一份；表读不到或一条正则都没有 ⇒ 红。
#   ② 每个触发文件都要在 .claude/batch-scope 里有一行；少一个 ⇒ 红，出路给两条。
#   ③ .claude/batch-scope 里的每个路径都要是这一批的触发文件；多出来的 ⇒ 红
#      （上一批留下的登记会让下一批白放行，判据同 .claude/naming-lint-exclude 的「排除只缩不涨」）。
#   ④ 登记行要写理由：路径之后 # 起，# 后面非空；空理由 ⇒ 红。
#   ⑤ 登记的路径要从仓库根起写（不以 / ./ ../ 开头、不含 /../）；写歪了 ⇒ 红。
#   ⑥ 68 号的正文里要还有这份清单的文件名——它与这一道算的是同一个量，清单只许有一份；找不到 ⇒ 红。
#   改动范围一个文件都没有 ⇒ 无对象可判，退 77（不记通过）。
#
# .claude/batch-scope 的格式：一行一条 <从仓库根起的路径><制表符或空格>#<为什么这一批要碰它>；
# 空行与整行以 # 开头的行是注释。一批提交完把它清空（只留注释）。
#
# 管不到的：理由写得对不对、这一批该不该碰这个文件、登记是不是开工时写的（也可能是判红之后补的）。
# 这几样是语义判断，靠人与 review。
# ⚠️ 几个会话共写一个仓时，不带 --staged 跑它会把**别的会话未提交的改动**也算进这一批，报出一堆不是你的触发文件。
# 判这一批的范围要跑 `bash .claude/scripts/gate.sh --staged`：它在临时 worktree 上只拿 HEAD + 暂存区，
# 那时报出来的就是这一次提交真要带的那几个（`session-wrapup.md` 第 4 条）。
# 判别力：fixtures/11-batch-scope.sh/red 放一个没登记的触发文件、一个登记了却不在这一批里的路径、
# 一行没写理由的登记、一个不是从仓库根起的路径、一份不再读触发文件清单的 68 号，五种必须同时判红；
# green 放三个触发文件（工作区改、暂存新增、未跟踪）都登记了、理由都写了、另有两个不是触发文件的路径不用登记，
# 必须判绿并报对数。
#
#   bash .claude/gate.d/11-batch-scope.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; exit 1; }
changed="$(gate_changed_paths "$(gate_diff_base head)" untracked)" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 HEAD）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
  exit 1
}
# 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
python3 - <(printf '%s\n' "$changed") <<'PY'
import os, re, sys

with open(sys.argv[1], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})

TRIGGER_TABLE = ".claude/gate.d/knowledge-sync-triggers.tsv"
SCOPE_FILE = ".claude/batch-scope"

if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，门禁 68 号与这一道都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if body and not body.startswith("#"):
        patterns.append(re.compile(body))
if not patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)

if not changed_files:
    print("  ! 相对 HEAD 一个改动都没有，本阶段无对象可判")
    sys.exit(77)

triggers = [path for path in changed_files if any(pattern.search(path) for pattern in patterns)]

registered = {}
no_reason, bad_path = [], []
if os.path.isfile(SCOPE_FILE):
    for number, raw in enumerate(open(SCOPE_FILE, encoding="utf-8").read().split("\n"), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        path, _, reason = line.partition("#")
        path = path.strip()
        if not path:
            continue
        registered[path] = number
        if not reason.strip():
            no_reason.append(f"{SCOPE_FILE}:{number}：{path} 后面没写为什么这一批要碰它")
        if path.startswith(("/", "./", "../")) or "/../" in path:
            bad_path.append(f"{SCOPE_FILE}:{number}：{path} 不是从仓库根起写的")

missing = [path for path in triggers if path not in registered]
stale = [path for path in registered if path not in triggers]

failed = False
if missing:
    failed = True
    print(f"  ✗ {len(missing)} 个触发文件没登记进这一批的范围（{SCOPE_FILE} 现在登记了 {len(registered)} 个）：")
    for path in sorted(missing):
        print(f"      {path}")                                  # gate-lint:detail
    print(f"     → 怎么办：每多碰一个触发文件就多一轮阶段同步（门禁 68 号按触发文件算范围），所以先问一句"
          "「它挡不挡着这一批的课题」——")
    print(f"               不挡：从暂存区撤出去（git restore --staged <路径>，工作区的改动留着），"
          f"把这件事记进 .claude/kb/checks-owed.md 往后延；")
    print(f"               挡着：在 {SCOPE_FILE} 里补一行「<路径><制表符>#<为什么>」，并认下它带来的那一轮阶段同步。")
if stale:
    failed = True
    print(f"  ✗ {len(stale)} 个登记的路径不是这一批的触发文件（登记项不起作用，会让下一批白放行）：")
    for path in sorted(stale):
        print(f"      {SCOPE_FILE}:{registered[path]}：{path}")  # gate-lint:detail
    print(f"     → 怎么办：上一批提交完没清表就会这样，把这几行删掉；"
          f"路径写错了就照 git status 的写法改（从仓库根起，中文路径不要带引号）。")
if no_reason:
    failed = True
    print(f"  ✗ {len(no_reason)} 行登记没写理由：")
    for entry in no_reason:
        print(f"      {entry}")                                  # gate-lint:detail
    print(f"     → 怎么办：路径后面写 #，# 后面写为什么这一批非碰它不可，理由不许省"
          f"（规矩同 .claude/abbreviations 与 .claude/naming-lint-exclude）。")
if bad_path:
    failed = True
    print(f"  ✗ {len(bad_path)} 行登记的路径不是从仓库根起写的：")
    for entry in bad_path:
        print(f"      {entry}")                                  # gate-lint:detail
    print("     → 怎么办：照 git status 的写法写（例 .claude/gate.d/68-knowledge-sync.sh），别写 ./ 开头或相对别处的路径。")
# ⑥ 68 号必须还在读这份表。它与这一道算的是同一个量（什么算触发文件），
# 而「两道各自存一份清单」这件事一旦发生，两边都不会红——一道说要回扫、另一道说不用登记，
# 谁都说不清该信哪个（`show-me-test.md`「两套装置算同一个量，就要有一条检查逼它们落到同一个数」）。
# 拦法是查 68 号的正文里还有没有这份表的路径：把清单抄回脚本里的人，多半同时把读表那几行删掉。
SYNC_STAGE = ".claude/gate.d/68-knowledge-sync.sh"
if os.path.isfile(SYNC_STAGE) and os.path.basename(TRIGGER_TABLE) not in open(SYNC_STAGE, encoding="utf-8").read():
    print(f"  ✗ {SYNC_STAGE} 里找不到 {os.path.basename(TRIGGER_TABLE)}：它不再读这份清单了")
    print(f"     → 怎么办：两道门禁算的是同一个量（什么算触发文件），清单只许有一份。"
          f"把 68 号改回从 {TRIGGER_TABLE} 读，别在脚本里另存一份。")
    failed = True
if failed:
    sys.exit(1)

print(f"  ✓ 这一批的 {len(triggers)} 个触发文件都登记进了 {SCOPE_FILE}，登记的 {len(registered)} 个都在这一批里"
      f"（相对 HEAD 共 {len(changed_files)} 个改动路径，另外 {len(changed_files) - len(triggers)} 个不是触发文件、不用登记；"
      f"触发文件的判据是 {TRIGGER_TABLE} 的 {len(patterns)} 条正则）")
PY
