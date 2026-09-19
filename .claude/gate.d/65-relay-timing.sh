#!/usr/bin/env bash
# gate-stage: 读子进程输出的循环不许一边给行打时间戳一边转打
#
# 判据与够不着的写法写在 research/scripts/relay-timing-lint.py 的文件头：扫 research/ 与 crates/ 下的 .rs 与 .py
# （target/ 与冻结证据 research/prompts/、research/results/ 不扫），读子进程输出的循环里同时取时间与输出就判红；
# 循环头那一行或上一行写 relay-timing-lint:allow 加理由可以豁免，理由为空也判红。
# 实测（2026-09-17）：E152（按里程碑对比六家文件系统的文件性能） 的来宾程序 run_singlefs 在 `for line in BufReader::new(child_output).lines()`
# 里先 `started.elapsed()` 打时间戳、再 `emitter.emit(...)` 转打。虚机里标准输出是串口，转打阻塞几毫秒而子进程写管道不被挡，
# 产物里连着打出、中间只有几微秒计算的几行时间戳跨了 21.7–25.8 ms，两次跑 10 轮里 9 轮外层算出的第二个事务挂钟
# 小于子进程自己计的时间——物理上不可能，而当时没有任何检查报出来。
# 判别力：fixtures/65-relay-timing.sh/red 放一个照那个旧写法的 .rs，必须判红；green 放辅助函数先读到 EOF、调用方再转打的写法
# 与一个不读子进程输出的 lines() 循环，必须判绿。脚本自己的 --selftest（含弱判据下必须判红那一条）由 47 号复跑。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
SCRIPT="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/relay-timing-lint.py"
if python3 "$SCRIPT" --check "$ROOT"; then
  exit 0
else
  status=$?
fi
[[ $status -eq 77 ]] && exit 77
echo "  ✗ 读子进程输出的循环里有一边打时间戳一边转打的写法，或者豁免没写理由（上面逐处列出；退出码 $status）"
echo "     → 怎么办：按上面那条的出路改装置——在被测进程里自己计时并在结果行里报；或者先把子进程输出读到 EOF（读的循环里只记时间与行）再转打；时间确实不进计时结论的，在循环头那一行或上一行写 // relay-timing-lint:allow <理由>。改完单跑 bash .claude/gate.d/65-relay-timing.sh 看它转绿。"
exit 1
