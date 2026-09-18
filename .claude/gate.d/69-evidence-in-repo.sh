#!/usr/bin/env bash
# gate-stage: 跑出来的产物与引来的依据都要落在仓里，不许只留在 /tmp
#
# 实测（2026-09-18 核出）：E142（第一个事务的干跑） 第十一次跑改了装置与变异表、跑了一个半小时，
# 产物只在 /tmp/claude-1000/e142-r11-runner/e142-r11-draft.out，research/results/ 里最新的还是 09-17 那份，
# research/scripts/replay.sh 的登记行也还指着它；另一处是一份还清依据写成 /tmp/claude-1000/<轮>/report.md。
# /tmp 下的草稿目录会话一重启就没了，那一轮的活等于白干，而此前没有任何东西报警。用户 2026-09-18 定：这类事拿门禁约束。
#
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；
# 工作区、暂存区与未跟踪文件都算（取法与 56-crates-adversarial-review.sh、68-knowledge-sync.sh 相同，
# 多一个 core.quotepath=false：不加的话中文路径被 git 转成带引号的八进制，对不上）。
#
# 判据一「实验跑了没留存」：改动范围里每个还在盘上的
#   research/e7-index-bench/src/bin/e<号>_*.rs 或 research/mutations/e<号>_*.tsv，
#   research/results/ 下（含子目录）要有一份这个实验号的产物——文件名形如 e<号> 后面跟非数字或到头（e142-…、e142_…、e12.1.out）——
#   而且它的 mtime 不早于那份源码。一个这样的产物都没有、或最新的那份比源码旧 ⇒ 红。
#   只改了 // 注释的装置源码（与基准比，增删的每一行都是 // 注释或空行）不判、在成功行里逐个列名：
#   path-moves.md 要求全仓改路径，注释里的路径改了碰不到产物（2026-09-18 实测六份装置因此误判）。
# 判据二「证据指向仓外」：下面这些文件里，把 /tmp 路径当依据引用 ⇒ 红。认的形态是
#   「依据 / 证据 / 出处 / 产物 / 报告 / 原件 / 结论 / 存档 / 留档 / 留存 / 记录 / 落点 / 见 / 详见 / 参见」
#   之后隔至多两个空白、至多一个连接记号（是 / 在 / 为 / ： / : / = / →）、至多两个空白，再（可带一个 ` 或 「）接上 /tmp/ 路径。
#   判的文件是：.claude/kb/ 下全部 .md；research/prompts/ 下这一轮**新写**的 .md（新增或未跟踪）。
#
# 为什么两半的射程不一样：kb 正文只写现状（kb-discipline.md 第 8 条），旧文件照样该改；
# 而 research/prompts/ 登记在 .claude/doc-lint-exclude 里，是「原样保存的证据不许事后改」
# （evidence-discipline.md），一道要求回去改旧提示的检查只会逼出证据链断掉或整个被绕过（sop-first.md）。
# 新写的那一批还没冻，此刻正是能改的时候，所以只判它们。
#
# 不判的，成功行现算着列：这次改动没碰实验装置与变异表时判据一没有对象；
# research/prompts/ 下这一轮没新写的 .md（数目现算）；判据二只扫 .md，模型目录里的 .py 不扫。
# 在 --staged 的临时 worktree 里整道退 77 不记通过：那里全部文件是同一刻检出的，mtime 分不出装置与产物谁新。
#
# ⚠️ 管不到的：产物对不对得上源码（那是 87-replay.sh 的逐字节复跑，本阶段只看有没有、新不新，
# 而变异表的重跑日志 87 号一行都不看）；replay.sh 的登记行指没指到最新那份（只写在出路里，不判）；
# 句子是说做法还是引依据（本阶段按上面那个形状认，说「草稿放 /tmp/…」不算引依据；反过来，
# 一句「报告原件在 /tmp/…，这里已原样转存」照样判红，改法是把 /tmp 那一截删掉）；
# 已经提交的 prompts 文件后来加进去的 /tmp 依据（只判整份新写的文件，不判改动的行）；
# git checkout / stash pop 会把源码的 mtime 刷新，那时判据一会要求重跑一次。
# 判别力：fixtures/69-evidence-in-repo.sh/red 放一个产物比源码旧的装置、一个一份产物都没有的变异表、
# 一份 kb 里写「依据：/tmp/…」、一份这一轮新写的 prompts 里写「报告 `/tmp/…`」，必须判红；
# green 放一个产物比源码新的装置、一个不在改动范围里的旧装置、kb 里三种说做法的 /tmp 写法、
# 一份改过但不是新写的 prompts 里的「依据：/tmp/…」，必须判绿并报对数。
#
#   bash .claude/gate.d/69-evidence-in-repo.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
if [[ "$(git rev-parse --git-dir 2>/dev/null)" == */worktrees/* ]]; then
  echo "  ! $ROOT 是 --staged 的临时 worktree，文件的时间戳全是这一刻检出的，分不出装置与产物谁新，本阶段未跑（不记通过）"
  echo "    不带 --staged 在工作区里跑一遍 bash .claude/scripts/gate.sh 才判得了这一道。"
  exit 77
fi
base="HEAD"
if [[ -n "${GATE_BASE:-}" ]] && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
  base="$GATE_BASE"
elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1; then
  base="$(git merge-base HEAD '@{upstream}')"
fi
changed="$( { git -c core.quotepath=false diff --name-only "$base" -- ; git -c core.quotepath=false diff --name-only --cached -- ; git -c core.quotepath=false ls-files --others --exclude-standard -- ; } | sort -u )"
added="$( { git -c core.quotepath=false diff --name-only --diff-filter=A "$base" -- ; git -c core.quotepath=false diff --name-only --diff-filter=A --cached -- ; git -c core.quotepath=false ls-files --others --exclude-standard -- ; } | sort -u )"
# 两份清单经进程替换当文件传：当成命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
python3 - "$base" <(printf '%s\n' "$changed") <(printf '%s\n' "$added") <<'PY'
import os, re, subprocess, sys
from datetime import datetime, timezone

base = sys.argv[1]

def read_list(path):
    with open(path, encoding="utf-8", errors="replace") as handle:
        return sorted({line for line in handle.read().split("\n") if line})

changed_files = read_list(sys.argv[2])
added_files = set(read_list(sys.argv[3]))

results_dir = "research/results"
kb_dir = ".claude/kb"
prompts_dir = "research/prompts"

source_forms = (
    (re.compile(r"^research/e7-index-bench/src/bin/e([0-9]+)_[^/]*\.rs$"), "装置源码"),
    (re.compile(r"^research/mutations/e([0-9]+)_[^/]*\.tsv$"), "变异表"),
)
product_form = re.compile(r"^e([0-9]+)(?![0-9])")

# ── 判据一：改动范围里的实验装置与变异表，要有一份不比它旧的产物 ───────────
changed_sources = []
for path in changed_files:
    for form, kind in source_forms:
        match = form.match(path)
        if match and os.path.isfile(path):
            changed_sources.append((path, match.group(1), kind))
            break

newest_product_by_number = {}
if os.path.isdir(results_dir):
    for directory, _subdirectories, file_names in os.walk(results_dir):
        for file_name in file_names:
            match = product_form.match(file_name)
            if not match:
                continue
            product_path = os.path.join(directory, file_name)
            try:
                modified_at = os.path.getmtime(product_path)
            except OSError:
                continue
            number = match.group(1)
            previous = newest_product_by_number.get(number)
            if previous is None or modified_at > previous[1]:
                newest_product_by_number[number] = (product_path, modified_at)

def moment(seconds):
    return datetime.fromtimestamp(seconds, timezone.utc).strftime("%Y-%m-%d %H:%M")

def changes_only_comments(path):
    """装置源码与基准比，增删的每一行都是 // 注释或空行：这种改动（多半是按 path-moves.md 改注释里的路径）碰不到产物。"""
    if path in added_files:
        return False
    diff = subprocess.run(["git", "-c", "core.quotepath=false", "diff", "-U0", base, "--", path],
                          capture_output=True, text=True, errors="replace")
    if diff.returncode != 0:
        return False
    changed_lines = [line[1:] for line in diff.stdout.split("\n")
                     if line[:1] in "+-" and not line.startswith(("+++", "---"))]
    return bool(changed_lines) and all(line.strip() == "" or line.lstrip().startswith("//") for line in changed_lines)

unstored_runs = []
comment_only_sources = []
for path, number, kind in changed_sources:
    if kind == "装置源码" and changes_only_comments(path):
        comment_only_sources.append(path)
        continue
    source_modified_at = os.path.getmtime(path)
    newest = newest_product_by_number.get(number)
    if newest is None:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：{results_dir} 下一份 E{number} 的产物都没有")
    elif newest[1] < source_modified_at:
        unstored_runs.append(f"{path}（{kind}，改于 {moment(source_modified_at)}）：最新的 E{number} 产物 {newest[0]} 还是 {moment(newest[1])} 那份")

# ── 判据二：kb 与这一轮新写的提示里，不许把 /tmp 路径当依据引用 ────────────
citation_form = re.compile(
    r"(依据|证据|出处|产物|报告|原件|结论|存档|留档|留存|记录|落点|详见|参见|见)"
    r"[ \t]{0,2}(?:是|在|为|：|:|=|→)?[ \t]{0,2}[`「]?"
    r"(/tmp/[^\s`\"'()（）「」，。；：、|]*)"
)
tmp_form = re.compile(r"/tmp/[^\s`\"'()（）「」，。；：、|]*")

scanned_files = []
skipped_prompt_files = 0
if os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if file_name.endswith(".md"):
                scanned_files.append(os.path.join(directory, file_name))
if os.path.isdir(prompts_dir):
    for directory, _subdirectories, file_names in os.walk(prompts_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            if path in added_files:
                scanned_files.append(path)
            else:
                skipped_prompt_files += 1

outside_citations = []
tmp_mentions = 0
for path in sorted(set(scanned_files)):
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        continue
    for line_number, line in enumerate(lines, 1):
        tmp_mentions += len(tmp_form.findall(line))
        for match in citation_form.finditer(line):
            excerpt = line[max(0, match.start() - 24):match.end()].strip()
            outside_citations.append(f"{path}:{line_number}：…{excerpt}")

if not changed_sources and not scanned_files:
    print(f"  ! 这次改动没碰实验装置与变异表（基准 {base}），{kb_dir} 与 {prompts_dir} 下也没有要扫的 .md，本阶段无对象可判")
    sys.exit(77)

failed = False
if unstored_runs:
    failed = True
    print("  ✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物——这一跑多半只留在 /tmp 的草稿里：")  # gate-lint:summary
    for entry in unstored_runs:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：跑一次把产物存进 {results_dir}/，并在 research/scripts/replay.sh 把这个实验的登记行指到它。")
    print("               这一跑已经跑过、产物还在 /tmp 的草稿目录里，就现在拷进来（会话一重启那个目录就没了）；")
    print("               只改了 // 注释的装置本阶段不判；改了字符串里的路径，照 .claude/rules/path-moves.md 第 4 条连同留存产物一起改，复跑仍逐字相同。")
if outside_citations:
    failed = True
    print("  ✗ 这些地方把 /tmp 下的东西当依据引用了——那是会话自己的草稿目录，一重启就没了，三个月后没人核得动：")  # gate-lint:summary
    for entry in outside_citations:
        print(f"     {entry}")  # gate-lint:detail
    print(f"     → 怎么办：把 /tmp 下那份东西拷进仓里该在的位置（报告、提示与判决进 {prompts_dir}/，跑出来的数进 {results_dir}/），")
    print("               引用改成指那一份：在引用它的文件里把 /tmp 路径逐处换成仓内路径（path-moves.tsv 只收从仓库根写的路径，/tmp 登记不进去）。")
    print("               原件已经没了，就写明它没了、把还核得动的那部分落进仓里；只是在说做法（草稿放在哪），把句子里的依据词去掉。")
if failed:
    sys.exit(1)

print(f"  ✓ 产物与依据都落在仓里（判了 {len(changed_sources)} 份这一轮改过的实验装置与变异表、{len(set(scanned_files))} 份 .md，扫到 {tmp_mentions} 处 /tmp 路径、没有一处是引依据；基准 {base}）")
if not changed_sources:
    print("    没判：这次改动没碰 research/e7-index-bench/src/bin/ 与 research/mutations/，判据一这一半没有对象")
print(f"    没判 {skipped_prompt_files} 份 {prompts_dir} 下这一轮没新写的 .md（登记在 .claude/doc-lint-exclude 的原样保存证据，改不得）")
if comment_only_sources:
    print(f"    没判 {len(comment_only_sources)} 份只改了 // 注释的装置源码（碰不到产物，不要求重跑）：{'、'.join(comment_only_sources)}")
PY
