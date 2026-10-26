#!/usr/bin/env python3
# 门禁 harness-test-environment 的样本 machine-clean-gate-run-simulated-green 里的替身，不是真脚本的拷贝：只核在 gate.sh 里那一支交给它的参数。
# 样本根放了 .gate-run-simulated，这一道模拟在 gate.sh 里（GATE_NOT_RUN_FILE 落在 TMPDIR 里）：check 该带 --gate-run-temporary-directory <TMPDIR>。
# 认不出在 gate.sh 里（模拟没起来）也算对不上：这份样本就是要证在 gate.sh 里那一支。对上了照真脚本的汇总行格式报判红 0 类、退 0，对不上报判红 1 类、退 1。
import os
import sys

arguments = sys.argv[1:]
if arguments[:1] in (['clean'], ['record-run-start']):
    sys.exit(0)   # 开跑时刻与预演的参数由 machine-clean-too-recent-green 核
not_run_file = os.environ.get('GATE_NOT_RUN_FILE', '')
temporary = os.environ.get('TMPDIR', '')
in_gate = bool(not_run_file and temporary) and os.path.realpath(os.path.dirname(not_run_file)) == os.path.realpath(temporary)
passed = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == '--gate-run-temporary-directory']
if arguments[:1] != ['check'] or not in_gate or passed != [temporary]:
    print(f'  ✗ 临时目录残留：替身收到的参数是 {arguments}（认出在 gate.sh 里：{in_gate}），--gate-run-temporary-directory 该是 [{temporary!r}]，收到 {passed}')
    print('     → 怎么办：看门禁 harness-test-environment 里 .gate-run-simulated 与 gate_run_arguments 那两段：模拟在 gate.sh 里时交这一轮的 TMPDIR')
    print('  汇总：查了 1 类；✓ 0 类，判红 1 类（临时目录残留），没查成 0 类，没跑 0 类')
    sys.exit(1)
print('  ✓ 临时目录残留：替身收到的 --gate-run-temporary-directory 对得上（在 gate.sh 里）')
print('  汇总：查了 1 类；✓ 1 类，判红 0 类，没查成 0 类，没跑 0 类')
