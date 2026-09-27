"""项目自己的 python 脚本从这一份取 preflight，不直接 import 规范副本里的 preflight.py（理由与同目录的 preflight.sh 相同）。

规范副本 .claude/singlefs-ai-sop/ 被 .gitignore 挡着，git worktree 里没有它：提交时在「HEAD + 暂存区」临时树里跑 54 号 --full，
它调的 research/scripts/admission.py 就在那种树里跑。直接 import 副本会当场 ImportError。
这一份随仓走，每棵树里都在：先用这棵树里的副本，没有就用主仓（git 的公共目录所在的那一份仓）里的，
两处都没有就判红退 1。只在调 preflight 时才去找，import 这一份本身不起子进程、不开文件。
"""
import os
import subprocess
import sys


def shared_preflight_directory():
    """规范副本 scripts/ 目录的绝对路径：本树的优先，其次主仓的；都没有回 None。"""
    tree_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
    candidates = [os.path.join(tree_root, '.claude', 'singlefs-ai-sop', 'scripts')]
    try:
        common_directory = subprocess.run(['git', '-C', tree_root, 'rev-parse', '--path-format=absolute', '--git-common-dir'],
                                          capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.strip()
    except OSError:
        common_directory = ''
    if common_directory:
        candidates.append(os.path.join(os.path.dirname(common_directory), '.claude', 'singlefs-ai-sop', 'scripts'))
    return next((candidate for candidate in candidates if os.path.isfile(os.path.join(candidate, 'preflight.py'))), None)


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
