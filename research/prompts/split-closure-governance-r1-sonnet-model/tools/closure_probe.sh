#!/usr/bin/env bash
# 算键库的探针：samples/ 下每个样本根各起一次 scripts/gate/lib_function_units.py 的 closure（p6 起 key），退出码与输出落进 <输出目录>，
# 每个样本打一行「sample=<名> mode=<closure|key> exit=<码> expect=<读码推出的> refusals=<拒算行数> ...」，推断写在每行 expect= 里，不是判定。
# 用法：closure_probe.sh <仓根> <样本目录> <输出目录>（输出目录要不存在或为空）；退出码恒 0（样本的退出码是数据），2 用法错。
set -u
if (($# != 3)); then
  echo "用法：closure_probe.sh <仓根> <样本目录> <输出目录>" >&2
  exit 2
fi
repo="$1"
samples="$2"
out="$3"
mkdir -p "$out"
library="$repo/scripts/gate/lib_function_units.py"
[[ -f "$library" ]] || { echo "  ✗ 找不到 $library" >&2; exit 2; }

probe() {   # probe <样本名> <模式> <入口> <推断>
  local name="$1" mode="$2" entry="$3" expect="$4" code refusals
  python3 "$library" "$mode" "$samples/$name" "$entry" > "$out/$name.$mode.txt" 2>&1
  code=$?
  refusals=$(grep -c 'compiled-part-exec\|inline-script-not-literal\|subprocess-command-unfolded' "$out/$name.$mode.txt")
  echo "sample=$name mode=$mode exit=$code refusal-lines=$refusals expect=$expect"
}

probe p1-two-printf-statements closure s/entry.sh "exit=4 拒算（函数体两条 printf，shell_index.py:810 与 :1393 都不认）"
probe p1c-one-printf-two-variables closure s/entry.sh "exit=0 且输出里有 helper-used.sh（一条 printf 打两个变量，折得出）"
probe p2-read-from-file closure s/entry.sh "exit=4 拒算 compiled-part-exec（read 从 \"\$1\" 读，shell_index.py:506 不记赋值）"
probe p3-read-from-file-after-literal closure s/entry.sh "exit=0 无拒算（推的：先赋过字面空串的变量被 read 改写，assignment_sites 只看到空串；若是 exit=4 就是读错了）"
probe p4-module-object-handed-out closure s/entry.sh "exit=0 且输出里有 helper-unused.sh（模块对象交出去，按整个闭包见过的属性名连）"
probe p4c-module-object-contained closure s/entry.sh "exit=0 且输出里有 helper-used.sh、没有 helper-unused.sh（模块对象出不去，只连调到的属性）"
probe p5-name-collision-registered closure .claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh "exit=0 无拒算（推的：登记行按（文件、函数短名 run、词 command）认，不管是哪个类的 run）"
probe p5c-name-collision-unregistered closure s/entry.sh "exit=4 拒算（同一段正文放在没登记的路径上，对照）"
probe p6-service-entry-a key task-server/probe_entry.py "键 K1"
probe p6-service-entry-b key task-server/probe_entry.py "键与 K1 不同（推的：入口单元的指纹经 entry 行进键，closure_keys.py:366）"
probe p6-service-entry-helper-changed key task-server/probe_entry.py "键与 K1 相同（测试服务代码里非入口的单元不进键）"
echo "name=closure_probe samples=11 out=$out"
exit 0
