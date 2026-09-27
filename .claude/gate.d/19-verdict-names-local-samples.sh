#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树）里这一次新写的判决，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）
#
# 规则在 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「本地腿交回的每一份样本都读，判决里逐份按路径列出」那一条：
# 干净样本、通读判带损坏的样本、损坏闸留下的作废副本（`-output-void<n>.md`）都要在判决里按路径列出，不许只写「作废」「不稳定不采」。
# 判据（形式）：
#   ① 对象是这一次改动新写的判决 `research/prompts/<轮>-main-verification.md`（只认 research/prompts/ 顶层）。新写的只认三种：
#      相对基准新增（`git diff --diff-filter=A`）、暂存区新增、未跟踪；被改过的旧判决不算、不追溯。改动范围取法用共用库
#      research/scripts/changed-paths.sh 的 gate 取法（与 56、68、69、72、97 号同一份代码）。
#   ② 轮名取判决文件名去掉 `-main-verification.md`。那一轮本地腿的样本：research/prompts/ 顶层里文件名以 `<轮>-local-` 起头、
#      含 `-output-s` 或 `-output-void`、以 `.md` 结尾的普通文件（含 `<轮>-local-attack-part1-output-s1.md` 这类拆题的）。
#   ③ 每一份样本的文件名都要在那份判决里逐字出现（按文件名字面找，带不带目录都算）；漏的逐个列出，判红。
#   0 字节的样本（ask-local.sh 退 5、退 6 时重定向建出的空 s<n>）不判，成功句逐个报跳过的；那一轮没有本地腿样本的判决不判，照样计数、逐个报出。
#   没有新写的判决 ⇒ 无对象可判，退 77（本次未跑，不记通过）。
#
# ⚠️ 管不到的：点名了读没读、标得对不对（干净还是参考）、「不稳定」格里的线索有没有逐条写去向——靠人看；
# 文件名出现在「没读」那一句里也算点名（与 56、72 号同一个盲区）。轮名互为前缀的（`x-r1` 与 `x-r1-local-y-r1`）会把后者的样本算给前者。
#
# 判别力：fixtures/19-verdict-names-local-samples.sh/red（两份新写的判决各漏一份作废副本：一份未跟踪、一份暂存新增）与
# missing-clean-sample（漏一份干净样本）必须判红并逐个列出漏的；green（点名齐、0 字节 s2 没点名、别一轮与子目录里的样本、
# 没有本地腿样本的判决、只补了一句的旧判决）必须判绿并报对数。
# 弄坏开关 VERDICT_LOCAL_SAMPLES_BREAK：=skip-void 不认作废副本（red 转绿），=require-empty 连 0 字节的也要点名（green 转红）；
# 带着任一项跑 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d`，那一例判错、自证判红。
#
# gate-similar: 72-agent-def-adversarial-review.sh 它判改过的定义有没有被新写的判决点名，对象是定义、没改定义就退 77；这里的对象是新写的判决本身、判的是判决点没点名那一轮的样本，并进去会让它在只写了判决时也得跑、77 的条件与成功句都要拆成两套
# gate-similar: 56-crates-adversarial-review.sh 同 72 号，对象是改过的 crates 源文件；新判决的取法两边已经共用 changed-paths.sh，剩下的只有读判决全文那一句
# gate-similar: 66-abandoned-rounds.sh 它判发过腿的轮有没有判决（全树、含归档），不看判决的内容；这里只看这一次新写的判决、判它点名了哪些样本，射程与对象都不同
# gate-similar: 58-implementation-premise.sh 它判三方正文与材料的形式（crates/ 前提、本地腿派哪一侧、开工快照），不读判决与样本
# gate-overlap:copy-kept 72-agent-def-adversarial-review.sh 开头取改动范围的几行是每个用改动范围的阶段都照写的固定写法：preflight 那两行规范要求逐字写在脚本里，取法已经抽成 research/scripts/changed-paths.sh，剩下的只是 cd 进仓、判是不是 git 仓与 source 它
#
#   bash .claude/gate.d/19-verdict-names-local-samples.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段无对象可判"; exit 77; }
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里另抄一份。"; exit 1; }
base="$(gate_diff_base gate)"
added="$(gate_changed_paths "$base" untracked A)" || {
  echo "  ✗ 取不到这次改动新增了哪些路径（基准 $base）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一阶段什么都没比，不是通过。"
  exit 1
}
mapfile -t new_verdicts < <(grep -E '^research/prompts/[^/]+-main-verification\.md$' <<<"$added" || true)
if ((${#new_verdicts[@]} == 0)); then
  echo "  ! 这次改动没有新写的判决 research/prompts/<轮>-main-verification.md，本阶段无对象可判（基准 $base）"
  exit 77
fi
python3 - "$base" "${new_verdicts[@]}" <<'PY'
import os
import sys

base = sys.argv[1]
verdict_paths = [path for path in sys.argv[2:] if path]
prompts_directory = "research/prompts"
suffix = "-main-verification.md"
breakage = os.environ.get("VERDICT_LOCAL_SAMPLES_BREAK", "")

top_level_names = sorted(os.listdir(prompts_directory)) if os.path.isdir(prompts_directory) else []


def samples_of(round_name):
    """那一轮本地腿的样本文件名（research/prompts/ 顶层的普通文件）。"""
    prefix = round_name + "-local-"
    found = []
    for name in top_level_names:
        if not (name.startswith(prefix) and name.endswith(".md")):
            continue
        is_void = "-output-void" in name
        if not ("-output-s" in name or is_void):
            continue
        if is_void and breakage == "skip-void":
            continue
        if not os.path.isfile(os.path.join(prompts_directory, name)):
            continue
        found.append(name)
    return found


named_total = 0
with_samples = 0
without_samples = []
skipped_empty = []
missing_by_verdict = []
for verdict_path in verdict_paths:
    if not os.path.isfile(verdict_path):
        continue
    round_name = os.path.basename(verdict_path)[: -len(suffix)]
    samples = samples_of(round_name)
    if not samples:
        without_samples.append(verdict_path)
        continue
    with open(verdict_path, encoding="utf-8", errors="replace") as handle:
        verdict_text = handle.read()
    judged = 0
    missing = []
    for name in samples:
        sample_path = os.path.join(prompts_directory, name)
        if os.path.getsize(sample_path) == 0 and breakage != "require-empty":
            skipped_empty.append(sample_path)
            continue
        judged += 1
        if name in verdict_text:
            named_total += 1
        else:
            missing.append(name)
    if judged:
        with_samples += 1
    else:
        without_samples.append(verdict_path)
    if missing:
        missing_by_verdict.append((verdict_path, missing))

verdict_count = len(verdict_paths)
if missing_by_verdict:
    missing_count = sum(len(missing) for _path, missing in missing_by_verdict)
    print(f"  ✗ 新写的判决里 {len(missing_by_verdict)} 份没按路径点名那一轮本地腿的全部样本（共漏 {missing_count} 份）：")  # gate-lint:summary
    for verdict_path, missing in missing_by_verdict:
        print(f"      {verdict_path} 漏 {len(missing)} 份：")  # gate-lint:detail
        for name in missing:
            print(f"        {name}")  # gate-lint:detail
    print("     → 怎么办：逐份读这一轮本地腿交回的样本（干净的、通读判带损坏的、损坏闸留下的 -output-void<n>.md），在这份判决里按路径列出、")
    print("               各标干净或参考，写它说了什么、去向如何（.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」一节")
    print("               「本地腿交回的每一份样本都读，判决里逐份按路径列出」那一条）；不改样本、不删样本。")
    sys.exit(1)

print(f"  ✓ 新写的判决 {verdict_count} 份：{with_samples} 份逐份点名了那一轮本地腿的 {named_total} 份样本，"
      f"{len(without_samples)} 份那一轮没有本地腿样本；0 字节样本跳过 {len(skipped_empty)} 份（基准 {base}）")
if without_samples:
    print("    没有本地腿样本的：" + "、".join(without_samples))
if skipped_empty:
    print(f"    0 字节样本跳过 {len(skipped_empty)} 份：" + "、".join(skipped_empty))
PY
