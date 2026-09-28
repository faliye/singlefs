#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 判不出来它自己按「要跑」回 0，拒绝反而会让门禁阶段按红记
# 这一道阶段能不能复用上一次的判定：**要跑退 0，可跳过退 1，判不出来退 0**（与 change-touches-crates.sh 同极性）。
#
#   stage-must-run.sh <项目根> <阶段文件名>    判一次，把判据与依据打到 stdout
#   stage-must-run.sh --selftest               自证（格数由它自己的成功行现算）
#
# 名字说的是哪一边为真：**要不要跑**。退 0 = 要跑（输入变了、判不出来、或不许复用），退 1 = 可跳过。
# 与 change-touches-crates.sh 同极性（那边 0 = 碰了 = 要跑），接法也一样，阶段里照抄那四行就行。
#
# 判法住在门禁与实验共用的准入模块里：research/scripts/admission.py 的 gate-reuse（登记表同一份，
# .claude/gate.d/stage-inputs.tsv）。这一份只做两件事：把模块的「可跳过」（退 10）翻成这里的 1，
# 以及**模块自己出错一律按要跑处理**——python 起不来、抛异常退 1，都不许被读成「可跳过」，那等于把这道门禁关掉。
#
# 判据只有一句：**这一道读的那几条路径，在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间，git 说变没变。**
# 在这之前先看一眼全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿时按登记输入在被判那棵树上的指纹写 git common-dir 里的
# `singlefs-stage-green.<阶段文件名>.<指纹>`（admission.py stage-marker-write），在且没过 SINGLEFS_REUSE_HOURS 就可跳过——
# 崩溃验证员在工作区跑过的那一趟，整轮门禁不用再跑第二遍（用户 2026-09-27 定：照 54 号写标记、只跑变了的）。
# 路径清单在 `.claude/gate.d/stage-inputs.tsv`，那是唯一登记位；清单自己也进比对（清单变了必定重跑）。
#
# 为什么用树不用提交、为什么只一条 ref：
#   `refs/sop/gate-ok` 断言的是「**不带 --staged 的整轮**在哪个**提交**上绿过」，用来算 diff 窗口；
#   带 `--staged` 跑绿从不让它前移（窗口是人为收窄的，C449（两套基准算法让带不带 --staged 判出不同的改动范围）
#   与 C454（--staged 跑绿不让基准前移，而且不说）记的都是这一格）。
#   `refs/sop/staged-green` 断言的只有一句「**带 --staged 的整轮**在这棵**树**上绿过」，而 `--staged` 恰好完整验了这一句。
#   整轮全绿意味着每一道都在那棵树上绿过 ⇒ **一条 ref 就够**，不必按阶段各存一条；ref 是覆盖语义，存不下第二条。
#
# 为什么只认暂存树，不看工作区：
#   几个会话共写一个仓时，工作区随时被别人改，它不代表任何人打算提交的东西
#   （共享 gate.sh 的「跑的过程中工作区变没变」那道指纹检查立的就是这个理由）。
#   所以这一道只在 `SINGLEFS_STAGED_TREE` 给了树对象时才许复用——那个值由 research/scripts/gate-staged.sh 算一次、
#   传给整轮、跑绿之后前移同一棵。没有这个变量（有人直接跑 gate.sh）一律当成要跑。
#
# 管不到的：
#   **环境变了而内容没变**——rustc 升级、系统库换了，树一模一样而结论已经不作数。
#   把版本串纳入比对要么落盘、要么塞进 ref 名，两样都跟「只一条 ref、不落盘」冲突；
#   这里用**复用上限**替代：绿判定超过 SINGLEFS_REUSE_HOURS 小时（默认 24）就强制跑一趟并重新前移。
#   `Cargo.lock` 在各阶段的清单里，依赖变化看得见；工具链升级看不见，靠上限兜。
#   `SINGLEFS_GATE_FULL=1` 强制当成要跑。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${1:-}" == "--selftest" ]]; then
  exec bash "$HERE/stage-must-run-selftest.sh"
fi

if reuse_reason="$(python3 "$HERE/admission.py" gate-reuse "$@")"; then reuse_rc=0; else reuse_rc=$?; fi
case "$reuse_rc" in
  0)  printf '%s\n' "$reuse_reason"; exit 0 ;;   # 要跑
  10) printf '%s\n' "$reuse_reason"; exit 1 ;;   # 可跳过
  *)  printf '%s\n' "判不出来：准入模块 research/scripts/admission.py 退 ${reuse_rc}（python 起不来或出了异常，stderr 里有原因），按要跑处理"
      exit 0 ;;
esac
