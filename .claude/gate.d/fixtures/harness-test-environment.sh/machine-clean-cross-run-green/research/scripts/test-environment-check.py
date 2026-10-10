#!/usr/bin/env python3
# 门禁 harness-test-environment 的样本 machine-clean-cross-run-green 里的替身，不是真脚本的拷贝：只核这一道交给它的参数。
# GATE_CROSS_RUN_TMPDIR 设了（gate.sh 设它：开跑前的那个 TMPDIR）该带 --cross-run-temporary-directory <它>，没设不带；
# 对上了照真脚本的汇总行格式报判红 0 类、退 0，对不上报判红 1 类、退 1。
import os
import sys

arguments = sys.argv[1:]
cross_run = os.environ.get('GATE_CROSS_RUN_TMPDIR', '')
passed = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == '--cross-run-temporary-directory']
wanted = [cross_run] if cross_run else []
if arguments[:1] != ['check'] or passed != wanted:
    print(f'  ✗ 临时目录残留：替身收到的参数是 {arguments}，--cross-run-temporary-directory 该是 {wanted}，收到 {passed}')
    print('     → 怎么办：看门禁 harness-test-environment 里 GATE_CROSS_RUN_TMPDIR 那一段：设了就交给 --cross-run-temporary-directory，没设不交')
    print('  汇总：查了 1 类；✓ 0 类，判红 1 类（临时目录残留），没查成 0 类，没跑 0 类')
    sys.exit(1)
print(f'  ✓ 临时目录残留：替身收到的 --cross-run-temporary-directory 对得上（{"GATE_CROSS_RUN_TMPDIR 设了" if cross_run else "GATE_CROSS_RUN_TMPDIR 没设"}）')
print('  汇总：查了 1 类；✓ 1 类，判红 0 类，没查成 0 类，没跑 0 类')
