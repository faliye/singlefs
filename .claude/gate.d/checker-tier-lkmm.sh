#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 research/scripts/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）
# gate-stage: 内存序（herd7 + litmus/，本工程自己的阶段）
# gate-category: checker-tier 类
# gate-similar: checker-tier-qemu-device-streams.sh 同是 checker-tier 类里要外部工具的那几道之一，但它起 QEMU 判真设备上收到的写；这里起 herd7 判 litmus 的内存序声明，工具、登记的输入与失败方式都不同
# gate-similar: code-source-discipline.sh 它静态判源码的写法与登记（名字、格式常量、实验源码纪律）；这里的 litmus 也要绑到 crates 的代码锚点，但判的是 herd7 对每条 Never / Sometimes 的判定与声明符不符
#
# 一格（--list 打格名表，--check litmus-verdicts 只跑它，格名写错退 2）：
# gate-cell: litmus-verdicts litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh（写法与判据在它的文件头）；整段判法照改用共用库之前那一版，只包进格函数。
#
# 上游 singlefs-ai-sop 2026-09-16 起不再跑 LKMM（移交那次的记录与删前原样在提交 fbae43e 里，`git show fbae43e:.claude/handover/qemu-herd7/README.md`），
# 从此 litmus/ 下每条 Never 有没有对照组、绑没绑到代码、herd7 判定与声明符不符，只有这一道阶段在判。
# 逻辑全在 .claude/scripts/lkmm.sh（本工程接管的那份），这里只负责把它接进门禁并给出路。
#
# 判别力样本（共享门禁的 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 跑）拿 --static-only 喂：样本目录里放一个 .lkmm-static-only 标记文件，本阶段就只跑不需要 herd7 的那几层
# （fs-design.md 五条硬要求第 2 条：只供测试的开关；静态检查全过 lkmm.sh 退 3，这一格把它换成 77（本次未跑：herd7 那一层没判），
# 整道跟着退 77，绿样本的 expect 写 exit=77；改用共用库之前原样退 3，改用共用库之后格退 0、77 之外的码一律算红）。
# 样本 fixtures/checker-tier-lkmm.sh/{red,green} 各放 .gate-cells 点名 litmus-verdicts。
# 样本那一档不问复用、不判前提、不记环境：下面三件都只在不带 --static-only 的那一档做。
#
# 准入（门禁与实验共用 research/scripts/admission.py，登记表 .claude/gate.d/stage-inputs.tsv 里本阶段那一行）：
#   ① 先问能不能复用上一次整轮全绿的判定（research/scripts/stage-must-run.sh，与 checker-tier-crates-mutation-replay.sh 同一种接法）：登记的路径在
#      refs/sop/staged-green 那棵树与这一次的暂存树之间没变，且 herd7 的版本（登记表第三列 environment=，
#      .claude/scripts/lkmm.sh --herd7-version 打的那一行）与本阶段上一次判绿时记下的相同，才退 77。
#      版本不在 git 树里，所以判绿之后要调 admission.py gate-record-environment 把它记进 git common-dir。
#   ② 前提（第三列 probe=：找得到 herd7）没齐判红，不退 77（admission.py gate-preconditions）。
#   内核树（SINGLEFS_KERNEL_TREE 或自动找到的那棵）里的 tools/memory-model 不进复用判定：它也不在 git 树里，找它可能要联网取，
#   只靠 stage-must-run.sh 的 24 小时复用上限兜。
#
#   bash .claude/gate.d/checker-tier-lkmm.sh [项目根]
#   bash .claude/gate.d/checker-tier-lkmm.sh --list      逐行打格名与判什么，不起 herd7
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
ADMISSION_MODULE="$STAGE_REPOSITORY/research/scripts/admission.py"
LKMM="$(cd "$(dirname "$0")/../scripts" && pwd)/lkmm.sh"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell litmus-verdicts cell_litmus_verdicts "litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符" \
  "改 litmus 还是改代码先想清楚（判定与声明不符时先核 singlefs-models 指的那段代码今天发的次序）；缺 herd7 跑 bash .claude/scripts/fetch-deps.sh --check"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check litmus-verdicts 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || exit 2

# ── litmus-verdicts：整段判法照改用共用库之前那一版 ──
cell_litmus_verdicts() {
[[ -f "$LKMM" ]] || { echo "  ✗ 找不到 $LKMM"; echo "     → 怎么办：它随仓走（.claude/scripts/lkmm.sh），被删了就从 git 找回来。"; exit 1; }
[[ -d "$ROOT/litmus" ]] || { echo "  ! $ROOT 下没有 litmus/，本阶段跳过"; exit 77; }
static_only=()
[[ -f "$ROOT/.lkmm-static-only" ]] && static_only=(--static-only)
if [[ ${#static_only[@]} -eq 0 ]]; then
  source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
    "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径、herd7 的版本怎么进判定见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
  python3 "$ADMISSION_MODULE" gate-preconditions "$STAGE_REPOSITORY" "$(basename "$0")" || exit 1
fi
bash "$LKMM" "$ROOT" "${static_only[@]}"
rc=$?
if [[ ${#static_only[@]} -gt 0 ]]; then
  # 静态档全过 lkmm.sh 退 3：herd7 那一层没判，这一格记本次未跑（77），不记通过；别的码原样交出（1 是红）
  if [[ "$rc" == 3 ]]; then
    echo "  ! 静态档（.lkmm-static-only）：herd7 那一层没判，这一格本次未跑"
    exit 77
  fi
  exit "$rc"
fi
if [[ "$rc" -ne 0 ]]; then
  echo "  ✗ 内存序判红（细节在上面 lkmm.sh 的输出里）"
  echo "     → 怎么办：改 litmus 还是改代码先想清楚：判定与声明不符时先核 singlefs-models 指的那段代码今天发的次序，"
  echo "               再决定是 litmus 没跟上代码、还是代码把屏障漏了；缺 herd7 就跑 bash .claude/scripts/fetch-deps.sh --check。"
  exit 1
fi
# 判绿之后记下 herd7 的版本（只在 gate-staged.sh 起的那一趟记）：下一趟整轮输入与版本都没变，① 才判得出可跳过
# 记不下只报不红：缺这份记录时下一趟判「要跑」，方向是多跑，不会让哪一道被跳过
if ! python3 "$ADMISSION_MODULE" gate-record-environment "$ROOT" "$(basename "$0")"; then
  echo "  ! 没记下这一次的 herd7 版本（admission.py gate-record-environment 没成）：下一趟整轮这一道照跑，不会复用"
fi
litmus_file_count="$(find "$ROOT/litmus" -maxdepth 1 -name '*.litmus' -type f | wc -l)"
echo "  ✓ 内存序：litmus/ 下 $litmus_file_count 份 litmus 里每条 Never 有对照组、绑到代码，herd7 判定与声明相符（逐条的判定在上面 lkmm.sh 的输出里）"
# 判绿之后写这一道这批输入的全绿标记（git common-dir）：整轮门禁与下一趟按它复用（用户 2026-09-27 定：照 54 号写标记、只跑变了的）；写不成只报不红。
if ! python3 "$ADMISSION_MODULE" stage-marker-write "$ROOT" "$(basename "$0")" >/dev/null; then
  echo "  ! 没写成这一道的全绿标记（admission.py stage-marker-write 没成）：下一趟这一道照跑，不会复用"
fi
}

stage_cells_run "$ROOT"
