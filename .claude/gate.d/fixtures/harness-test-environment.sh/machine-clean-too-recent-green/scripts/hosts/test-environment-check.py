#!/usr/bin/env python3
# 门禁 harness-test-environment 的样本 machine-clean-too-recent-green 里的替身，不是真脚本的拷贝：只核这一格交给它的参数。
# 开跑时刻文件 = ${GATE_CROSS_RUN_TMPDIR:-<数据根>/cross-run-cache}/gate-run-times；没设 GATE_CROSS_RUN_TMPDIR 时与阶段同一个取法：
#   起这份替身的阶段（祖先进程里命令行带 harness-test-environment.sh 的那一个）旁边的 ../hooks/lib_work_root.py shared cross-run-cache，
#   不经 ~ 与 $HOME（第二台的家目录是另一个，统一任务服务带的 SINGLEFS_DATA_ROOT 才是两台同一份）。
# 在 gate.sh 里（祖先进程里有命令行带 gate.sh 的）：先调 record-run-start --run-times-file <文件> --run-started <大于 0>，
#   再调 clean --refuse-too-recent --run-times-file <文件> --produced-after <大于 0> --temporary-directory <…>；
# 单跑（祖先里没有 gate.sh）：不调 record-run-start，clean 不带 --produced-after。
# 对上了：clean 退 0，check 报判红 0 类；对不上：报错退 1。
import os
import subprocess
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


STAGE_NAME = 'harness-test-environment.sh'


def ancestor_stage_file():
    """祖先进程里最近的那一个阶段 bash 的阶段文件绝对路径：命令行里以 /harness-test-environment.sh 结尾的绝对路径（阶段的 $0），
    写的是相对路径时取 bash 开着脚本的那个描述符（/proc/<进程号>/fd/255）；认不出交 None。"""
    process_identifier = os.getppid()
    while process_identifier > 1:
        try:
            with open(f'/proc/{process_identifier}/cmdline', 'rb') as handle:
                words = [word.decode('utf-8', 'replace') for word in handle.read().split(b'\0') if word]
            named = [word for word in words if os.path.basename(word) == STAGE_NAME]
            if named:
                if os.path.isabs(named[0]):
                    return named[0]
                opened = os.readlink(f'/proc/{process_identifier}/fd/255')
                return opened if os.path.basename(opened) == STAGE_NAME else None
            with open(f'/proc/{process_identifier}/stat') as handle:
                process_identifier = int(handle.read().rsplit(')', 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return None
    return None


def cross_run_cache():
    """GATE_CROSS_RUN_TMPDIR 设了照它；没设照阶段的取法问阶段旁边的 lib_work_root.py（数据根下共享的 cross-run-cache）。"""
    configured = os.environ.get('GATE_CROSS_RUN_TMPDIR')
    if configured:
        return configured
    stage_file = ancestor_stage_file()
    if stage_file is None:
        fail(f'祖先进程里找不到起这份替身的阶段 {STAGE_NAME}，算不出阶段该交的 cross-run-cache')
    library = os.path.join(os.path.dirname(stage_file), '..', 'hooks', 'lib_work_root.py')
    asked = subprocess.run([sys.executable, library, 'shared', 'cross-run-cache'], capture_output=True, text=True)
    if asked.returncode != 0 or not asked.stdout.strip():
        fail(f'{library} shared cross-run-cache 退 {asked.returncode}、打的是「{asked.stdout.strip()}」：{asked.stderr.strip()[-300:]}')
    return asked.stdout.strip()


expected_file = os.path.join(cross_run_cache(), 'gate-run-times')
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
