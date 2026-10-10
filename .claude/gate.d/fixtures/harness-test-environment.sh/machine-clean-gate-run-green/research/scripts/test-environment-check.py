#!/usr/bin/env python3
# 门禁 harness-test-environment 的样本 machine-clean-gate-run-green 里的替身，不是真脚本的拷贝：只核这一道交给它的参数。
# 在 gate.sh 里（GATE_NOT_RUN_FILE 所在的目录正是 TMPDIR）该带 --gate-run-temporary-directory <TMPDIR>，单跑时不带；
# 对上了照真脚本的汇总行格式报判红 0 类、退 0，对不上报判红 1 类、退 1。
import os
import sys

arguments = sys.argv[1:]
not_run_file = os.environ.get('GATE_NOT_RUN_FILE', '')
temporary = os.environ.get('TMPDIR', '')
in_gate = bool(not_run_file and temporary) and os.path.realpath(os.path.dirname(not_run_file)) == os.path.realpath(temporary)
passed = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == '--gate-run-temporary-directory']
wanted = [temporary] if in_gate else []
if arguments[:1] != ['check'] or passed != wanted:
    print(f'  ✗ 临时目录残留：替身收到的参数是 {arguments}，--gate-run-temporary-directory 该是 {wanted}，收到 {passed}')
    print('     → 怎么办：看门禁 harness-test-environment 里 gate_run_arguments 那一段：在 gate.sh 里交这一轮的 TMPDIR，单跑时不交')
    print('  汇总：查了 1 类；✓ 0 类，判红 1 类（临时目录残留），没查成 0 类，没跑 0 类')
    sys.exit(1)
print(f'  ✓ 临时目录残留：替身收到的 --gate-run-temporary-directory 对得上（{"在 gate.sh 里" if in_gate else "单跑"}）')
print('  汇总：查了 1 类；✓ 1 类，判红 0 类，没查成 0 类，没跑 0 类')
