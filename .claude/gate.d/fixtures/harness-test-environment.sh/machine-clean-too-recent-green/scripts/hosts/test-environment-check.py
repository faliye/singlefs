#!/usr/bin/env python3
# 门禁 harness-test-environment 的样本 machine-clean-too-recent-green 里的替身，不是真脚本的拷贝：只核这一格交给它的参数。
# 开跑时刻文件 = ${GATE_CROSS_RUN_TMPDIR:-$HOME/.local/share/singlefs/cross-run-cache}/gate-run-times。
# 在 gate.sh 里（祖先进程里有命令行带 gate.sh 的）：先调 record-run-start --run-times-file <文件> --run-started <大于 0>，
#   再调 clean --refuse-too-recent --run-times-file <文件> --produced-after <大于 0> --temporary-directory <…>；
# 单跑（祖先里没有 gate.sh）：不调 record-run-start，clean 不带 --produced-after。
# 对上了：clean 退 0，check 报判红 0 类；对不上：报错退 1。
import os
import sys

arguments = sys.argv[1:]


def in_gate():
    process_identifier = os.getppid()
    while process_identifier > 1:
        try:
            with open(f'/proc/{process_identifier}/cmdline', 'rb') as handle:
                if b'gate.sh' in handle.read():
                    return True
            with open(f'/proc/{process_identifier}/stat') as handle:
                process_identifier = int(handle.read().rsplit(')', 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def value_of(option):
    values = [arguments[index + 1] for index, word in enumerate(arguments[:-1]) if word == option]
    return values


def fail(message):
    print(f'  ✗ 预演或开跑时刻的参数不对：{message}（收到 {arguments}）')
    print('     → 怎么办：看门禁 harness-test-environment 里 machine-clean 格的 record-run-start 与 clean 预演那一段')
    sys.exit(1)


expected_file = os.path.join(os.environ.get('GATE_CROSS_RUN_TMPDIR') or os.path.expanduser('~/.local/share/singlefs/cross-run-cache'),
                             'gate-run-times')
marker = '.run-start-recorded'
gate = in_gate()
if arguments[:1] == ['record-run-start']:
    started = value_of('--run-started')
    if not gate:
        fail('单跑（祖先里没有 gate.sh）不该记开跑时刻')
    if value_of('--run-times-file') != [expected_file]:
        fail(f'--run-times-file 该是 {expected_file}')
    if len(started) != 1 or float(started[0]) <= 0:
        fail('--run-started 该是大于 0 的 epoch 秒')
    open(marker, 'w').close()
    print('  ✓ 替身：开跑时刻已记')
    sys.exit(0)
if arguments[:1] == ['clean']:
    if '--refuse-too-recent' not in arguments or '--yes' in arguments:
        fail('预演要带 --refuse-too-recent、不带 --yes')
    if value_of('--run-times-file') != [expected_file]:
        fail(f'--run-times-file 该是 {expected_file}')
    if not value_of('--temporary-directory'):
        fail('该用 --temporary-directory 点名要扫的目录')
    produced_after = value_of('--produced-after')
    if gate and (len(produced_after) != 1 or float(produced_after[0]) <= 0):
        fail('在 gate.sh 里该带 --produced-after <本趟开跑时刻>')
    if not gate and produced_after:
        fail('单跑时不带 --produced-after')
    if gate != os.path.exists(marker):
        fail('在 gate.sh 里该先记开跑时刻再预演；单跑时不记')
    print('test-environment-check：clean（替身），没有太新的项')
    sys.exit(0)
if arguments[:1] != ['check']:
    fail('只认 record-run-start / clean / check')
print('  ✓ 临时目录残留：替身收到的预演与开跑时刻参数对得上（' + ('在 gate.sh 里' if gate else '单跑') + '）')
print('  汇总：查了 1 类；✓ 1 类，判红 0 类，没查成 0 类，没跑 0 类')
