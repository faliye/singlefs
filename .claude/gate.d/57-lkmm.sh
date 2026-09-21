#!/usr/bin/env bash
# gate-stage: 内存序（herd7 + litmus/，本工程自己的阶段）
#
# 上游 singlefs-ai-sop 2026-09-16 起不再跑 LKMM（移交那次的记录与删前原样在提交 fbae43e 里，`git show fbae43e:.claude/handover/qemu-herd7/README.md`），
# 从此 litmus/ 下每条 Never 有没有对照组、绑没绑到代码、herd7 判定与声明符不符，只有这一道阶段在判。
# 逻辑全在 .claude/scripts/lkmm.sh（本工程接管的那份），这里只负责把它接进门禁并给出路。
#
# 判别力样本（89 号）拿 --static-only 喂：样本目录里放一个 .lkmm-static-only 标记文件，本阶段就只跑不需要 herd7 的那几层
# （fs-design.md 五条硬要求第 2 条：只供测试的开关；静态检查全过退 3，绿样本的 expect 写 exit=3）。
#
#   bash .claude/gate.d/57-lkmm.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LKMM="$(cd "$(dirname "$0")/../scripts" && pwd)/lkmm.sh"
[[ -f "$LKMM" ]] || { echo "  ✗ 找不到 $LKMM"; echo "     → 怎么办：它随仓走（.claude/scripts/lkmm.sh），被删了就从 git 找回来。"; exit 1; }
[[ -d "$ROOT/litmus" ]] || { echo "  ! $ROOT 下没有 litmus/，本阶段跳过"; exit 77; }
static_only=()
[[ -f "$ROOT/.lkmm-static-only" ]] && static_only=(--static-only)
bash "$LKMM" "$ROOT" "${static_only[@]}"
rc=$?
if [[ ${#static_only[@]} -gt 0 ]]; then
  exit "$rc"
fi
if [[ "$rc" -ne 0 ]]; then
  echo "  ✗ 内存序判红（细节在上面 lkmm.sh 的输出里）"
  echo "     → 怎么办：改 litmus 还是改代码先想清楚：判定与声明不符时先核 singlefs-models 指的那段代码今天发的次序，"
  echo "               再决定是 litmus 没跟上代码、还是代码把屏障漏了；缺 herd7 就跑 bash .claude/scripts/fetch-deps.sh --check。"
  exit 1
fi
echo "  ✓ 内存序：litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符"
