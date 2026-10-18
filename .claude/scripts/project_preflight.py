"""项目自己的 python 脚本从这一份取 preflight，不直接 import 规范副本里的 preflight.py（理由与同目录的 preflight.sh 相同）。

规范副本 .claude/singlefs-ai-sop/ 被 .gitignore 挡着，git worktree 里没有它：提交时在「HEAD + 暂存区」临时树里跑 checker-tier-crash-point-replay --full，
它调的 scripts/gate/admission.py 就在那种树里跑。直接 import 副本会当场 ImportError。
这一份随仓走，每棵树里都在：先用这棵树里的副本（相对这一份的字面路径 .claude/scripts/../singlefs-ai-sop/scripts/，
gate.sh --staged 的临时树与只拷了 .claude/ 的小树里副本跟着拷），没有就用主仓（git 的公共目录所在的那一份仓）里的，
两处都没有就判红退 1。只在调 preflight 时才去找，import 这一份本身不起子进程、不开文件。
两处路径都写成函数单元库（scripts/gate/lib_function_units.py）折得出的样子：字面路径拼 __file__ 的目录，主仓那一处拼 git rev-parse --git-common-dir。
"""
import os
import subprocess
import sys

LOADER_DIRECTORY = os.path.dirname(os.path.realpath(__file__))


def shared_preflight_directory():
    """规范副本 scripts/ 目录的绝对路径：本树的优先，其次主仓的；都没有回 None。只找，不改 sys.path。"""
    directory = os.path.join(LOADER_DIRECTORY, '..', 'singlefs-ai-sop', 'scripts')
    if os.path.isfile(os.path.join(directory, 'preflight.py')):
        return os.path.normpath(directory)
    try:
        common_directory = subprocess.run(['git', '-C', LOADER_DIRECTORY, 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                                          capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.strip()
    except OSError:
        return None
    if not common_directory:
        return None
    directory = os.path.join(os.path.dirname(common_directory), '.claude', 'singlefs-ai-sop', 'scripts')
    if os.path.isfile(os.path.join(directory, 'preflight.py')):
        return os.path.normpath(directory)
    return None


def shared_preflight_module(script_file):
    directory = shared_preflight_directory()
    if directory is None:
        print(f'  ✗ {script_file}：找不到规范副本里的 preflight.py（这棵树与主仓里都没有），准入与运行条件一条都判不了，拒绝往下跑', file=sys.stderr)
        print('     → 怎么办：规范副本 .claude/singlefs-ai-sop/ 不随 git 走；在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh），'
              '临时树从主仓的副本里找得到。', file=sys.stderr)
        sys.exit(1)
    if directory not in sys.path:
        sys.path.insert(0, directory)
    import preflight as shared_preflight
    return shared_preflight


def preflight(script_file, arguments=None):
    """同规范副本 preflight.py 的 preflight：摘掉 --force、现判条件，不满足退 78。"""
    return shared_preflight_module(script_file).preflight(script_file, arguments)


def preflight_record_success():
    """同规范副本 preflight.py 的 preflight_record_success。"""
    return shared_preflight_module('preflight_record_success').preflight_record_success()
