"""进仓的 git 钩子目录在哪、某棵树里装没装上（git 钩子安装脚本 scripts/gate/git-hooks-install.py 与检出钩子 bash-command-detector.sh 的 ㉒ 共用这一份）。

进仓的钩子目录是 HOOKS_DIRECTORY（scripts/gate/git-hooks/），里面是 HOOK_NAMES 列的五份：pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push；
经 `git config core.hooksPath` 生效。core.hooksPath 装成主工作树里那个钩子目录的绝对路径：相对路径在 `--git-dir`、`--work-tree`、`GIT_DIR` 起的 git、
链接工作树检出了不带钩子目录的老提交、稀疏检出的树里找不到钩子，git 不报错、一个都不跑；绝对路径指向的是同一份钩子，不论 git 在哪棵树里起。
主工作树的钩子目录自己也会不在：主工作树检出了不带它的老提交、钩子只在暂存区里时 git reset --hard，那时 core.hooksPath 还指着它、钩子文件不在，hooks_path_problems 照报。
core.hooksPath 写在共用的 .git/config 里，所有链接工作树读到同一个值；仓、.git/config 与钩子在共享根下，两台机器读到同一个绝对路径。

用法：python 里把这份所在的目录放进 sys.path（或按文件路径导入）再用；不单独起。
  main_worktree(top)              这棵树所属仓的主工作树（共用 git 目录叫 .git 时取它的上一级；别的形状取 top 自己）
  installed_hooks_path(top)       安装脚本该写、检出钩子该认的 core.hooksPath：主工作树里 HOOKS_DIRECTORY 的绝对路径
  accepted_directories(top)       core.hooksPath 解开之后算装对的目录（集合）：只有 installed_hooks_path 那一个（与它解开符号链接之后的写法）；链接工作树自己的钩子目录不算，链接工作树会被删
  project_checkout(top)           这棵树要求装钩子：它或主工作树自带钩子目录（pre-commit 在），或 core.hooksPath 已经指到名为 HOOKS_DIRECTORY 的目录（装过了而钩子文件后来不在）；别的仓（样本仓、临时仓）不判
  hooks_path_problems(top)        交问题列表（字符串），空表是装好了：core.hooksPath 没设、指错或写成相对路径、钩子文件缺、没有执行位
  configured_hooks_path(top)      git 读到的 core.hooksPath（~ 已展开）；没设交空串
"""
import os
import subprocess

HOOKS_DIRECTORY = os.path.join("scripts", "gate", "git-hooks")
HOOK_NAMES = ("pre-commit", "pre-merge-commit", "prepare-commit-msg", "pre-applypatch", "pre-push")
SENTINEL_HOOK_NAME = "pre-commit"


def git_output(top, *arguments):
    """在 top 里跑 git，交 (退出码, 标准输出去掉末尾换行)；起不来（git 不在）交 (127, "")。"""
    try:
        completed = subprocess.run(["git", "-C", top, *arguments], capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return 127, ""
    return completed.returncode, completed.stdout.rstrip("\n")


def main_worktree(top):
    code, common = git_output(top, "rev-parse", "--path-format=absolute", "--git-common-dir")
    if code == 0 and os.path.basename(common) == ".git":
        return os.path.dirname(common)
    return top


def installed_hooks_path(top):
    return os.path.join(main_worktree(top), HOOKS_DIRECTORY)


def accepted_directories(top):
    """core.hooksPath 解开之后算装对的目录：只有主工作树里的钩子目录（installed_hooks_path，与它解开符号链接之后的写法）；链接工作树自己的钩子目录不算，链接工作树会被删。"""
    installed = installed_hooks_path(top)
    return {installed, os.path.realpath(installed)}


def configured_hooks_path(top):
    code, value = git_output(top, "config", "--type=path", "--get", "core.hooksPath")
    return value if code == 0 else ""


def project_checkout(top):
    if os.path.isfile(os.path.join(top, HOOKS_DIRECTORY, SENTINEL_HOOK_NAME)) or os.path.isfile(os.path.join(installed_hooks_path(top), SENTINEL_HOOK_NAME)):
        return True
    value = configured_hooks_path(top)
    return bool(value) and os.path.normpath(value).endswith(os.sep + HOOKS_DIRECTORY)


def hooks_path_problems(top):
    """top（工作树的顶层目录）里装钩子的问题，逐条一句；空表是装好了。"""
    value = configured_hooks_path(top)
    if not value:
        return [f"core.hooksPath 没设（git 只读 .git/hooks，进仓的钩子 {HOOKS_DIRECTORY}/ 不起作用）"]
    expected = installed_hooks_path(top)
    resolved = os.path.realpath(value if os.path.isabs(value) else os.path.join(top, value))
    if resolved not in accepted_directories(top):
        return [f"core.hooksPath 是 {value}，解开是 {resolved}，不是进仓的钩子目录（主工作树的 {expected}）"]
    if not os.path.isabs(value):
        return [f"core.hooksPath 是相对路径 {value}：--git-dir、--work-tree、GIT_DIR 起的 git、检出了老提交的链接工作树、稀疏检出的树里找不到它，钩子一个都不跑；要写成绝对路径 {expected}"]
    problems = []
    for name in HOOK_NAMES:
        path = os.path.join(resolved, name)
        if not os.path.isfile(path):
            problems.append(f"钩子 {path} 不在")
        elif not os.access(path, os.X_OK):
            problems.append(f"钩子 {path} 没有执行位（git 不执行没有执行位的钩子，提交、合并、am 与推送不再被判）")
    return problems
