#!/usr/bin/env python3
# admission: always 判的是此刻仓里 core.hooksPath 的值与钩子文件的状态，上一次的结论不替这一次作保
# run-condition: command git python3
# gate-similar: .claude/hooks/lib_git_hooks.py 装没装上的判法只在那一份（检出钩子 ㉒ 与这里共用），这里是装（写 core.hooksPath）、核进仓的文件模式与第二台的核对和自证；门禁 code-tooling 的 git-hooks-installed 格调这里的 check
"""装进仓的 git 钩子（scripts/gate/git-hooks/ 下 .claude/hooks/lib_git_hooks.py 的 HOOK_NAMES 列的五份），核它装对了没有。

用法：
    git-hooks-install.py install [<仓根>]
        `git config core.hooksPath <主工作树里的 scripts/gate/git-hooks 的绝对路径>`（写进共用的 .git/config，本仓所有链接工作树读到同一个值），钩子文件没有执行位的补上，再核一遍；
        仓根不给就是这份脚本所在的仓；从链接工作树里起也写主工作树的那个目录。
    git-hooks-install.py check [<仓根>] [--both-machines]
        只读核：core.hooksPath 解开指到主工作树的钩子目录而且是绝对路径、HOOK_NAMES 里每一份在而且有执行位、进仓的文件模式是 100755（新克隆出来的仓靠它），
        这台的 git 认 `rev-parse --path-format=absolute`（钩子里取私有闸用）；有问题时另写一行 ⊘ 说检出钩子 ㉒ 拦不拦这棵树里的 git commit / git push。
        --both-machines 另经 scripts/hosts/peer-host-lib.sh 在第二台上跑同一条 check（仓、.git/config 与钩子在共享根下，两台读到同一个绝对路径）；
        多机配置里双机没开就只判本机并写一行 ⊘。
    git-hooks-install.py --selftest [<格名>…]
        临时目录里造装没装、装错、写成相对路径、钩子缺、没有执行位、模式不对、链接工作树检出老提交、主工作树检出老提交、钩子只在暂存区时 reset --hard、第二台判红判绿几种局面；
        每一格自己建临时目录与仓，格与格互不依赖，按每台默认重活线程数并行；不点名格就全跑，再把文件头每条弄坏开关逐个打开、各格并进同一个线程池核红在点名的格；
        点名了格的只跑那几格（单跑与整份跑判定相同，不跑开关轮）；设了 GIT_HOOKS_INSTALL_BREAK=<项> 时只跑这一条开关下的格。

core.hooksPath 装成绝对路径：相对路径从「钩子运行的目录」起算（非裸仓里是工作树的顶层目录），--git-dir、--work-tree、GIT_DIR 起的 git、检出了不带钩子目录的老提交的链接工作树、稀疏检出的树里
找不到钩子目录，git 不报错、什么都不判；绝对路径指向主工作树里的钩子目录，不论 git 在哪棵树里起都跑同一份钩子。主工作树自己检出了不带钩子目录的老提交、或钩子只在暂存区里时 git reset --hard，
钩子文件就不在了，core.hooksPath 还指着那里，check 报「钩子 … 不在」，检出钩子 ㉒ 拦这棵树里的 git commit / git push（lib_git_hooks.py 的 project_checkout 认得出这样的树）。
.git/hooks 里的钩子在设了 core.hooksPath 之后 git 不再读：用户私有的 pre-commit（提交窗口闸）由进仓的 pre-commit 与 pre-merge-commit 先调（large-file-git-hook.py）。

退出码：0 装对了；1 有问题（逐条列出）；2 用法错；78 准入没满足。

弄坏开关 GIT_HOOKS_INSTALL_BREAK=<项>（逗号隔开可给几项；只给 --selftest 与门禁 code-tooling 的 git-hooks-installed 格的样本根证红用，平常设了退 2、点名只在自证标记在的时候与这两处认；
红的格逐条以「✗ 自证 <格名>：」起头；单跑一条开关：GIT_HOOKS_INSTALL_BREAK=<项> python3 scripts/gate/git-hooks-install.py --selftest）：
  GIT_HOOKS_INSTALL_BREAK=install-writes-nothing  install 不写 core.hooksPath
      「✗ 自证 install-then-check-green：」「✗ 自证 install-from-linked-worktree-writes-main-path：」「✗ 自证 old-commit-linked-worktree-hooks-run：」
  GIT_HOOKS_INSTALL_BREAK=install-writes-relative-path  install 写相对路径 scripts/gate/git-hooks
      「✗ 自证 install-then-check-green：」「✗ 自证 install-from-linked-worktree-writes-main-path：」「✗ 自证 old-commit-linked-worktree-hooks-run：」
  GIT_HOOKS_INSTALL_BREAK=install-writes-own-tree-path  install 写这棵树自己的钩子目录（从链接工作树里起时不是主工作树的）
      「✗ 自证 install-from-linked-worktree-writes-main-path：」
  GIT_HOOKS_INSTALL_BREAK=check-accepts-relative-path  check 不报相对路径的 core.hooksPath
      「✗ 自证 relative-hooks-path-red：」
  GIT_HOOKS_INSTALL_BREAK=check-ignores-path  check 只看 core.hooksPath 设没设、不看它指到哪
      「✗ 自证 wrong-hooks-path-red：」「✗ 自证 linked-worktree-own-path-red：」
  GIT_HOOKS_INSTALL_BREAK=check-ignores-exec-bit  check 不看钩子文件有没有执行位
      「✗ 自证 no-exec-bit-pre-commit-red：」「✗ 自证 no-exec-bit-pre-merge-commit-red：」「✗ 自证 no-exec-bit-prepare-commit-msg-red：」「✗ 自证 no-exec-bit-pre-applypatch-red：」「✗ 自证 no-exec-bit-pre-push-red：」
  GIT_HOOKS_INSTALL_BREAK=check-ignores-tracked-mode  check 不看进仓的文件模式
      「✗ 自证 tracked-mode-pre-commit-red：」「✗ 自证 tracked-mode-pre-merge-commit-red：」「✗ 自证 tracked-mode-prepare-commit-msg-red：」「✗ 自证 tracked-mode-pre-applypatch-red：」「✗ 自证 tracked-mode-pre-push-red：」
  GIT_HOOKS_INSTALL_BREAK=check-skips-added-hooks  check 只核 pre-commit 与 pre-push，不核 pre-merge-commit、prepare-commit-msg、pre-applypatch（不看在不在、执行位、进仓的模式）
      「✗ 自证 missing-pre-merge-commit-red：」「✗ 自证 missing-prepare-commit-msg-red：」「✗ 自证 missing-pre-applypatch-red：」「✗ 自证 no-exec-bit-pre-merge-commit-red：」「✗ 自证 no-exec-bit-prepare-commit-msg-red：」「✗ 自证 no-exec-bit-pre-applypatch-red：」「✗ 自证 tracked-mode-pre-merge-commit-red：」「✗ 自证 tracked-mode-prepare-commit-msg-red：」「✗ 自证 tracked-mode-pre-applypatch-red：」
  GIT_HOOKS_INSTALL_BREAK=detector-scope-tree-only  check 写的「检出钩子 ㉒ 拦不拦」只认这棵树自带钩子目录的（不认主工作树自带、core.hooksPath 已指到钩子目录的）
      「✗ 自证 main-tree-old-commit-red：」「✗ 自证 hooks-staged-then-reset-hard-red：」
  GIT_HOOKS_INSTALL_BREAK=peer-skipped  --both-machines 不去第二台
      「✗ 自证 peer-green：」「✗ 自证 peer-failure-red：」「✗ 自证 peer-switch-off-warned：」
"""
import os
import shlex
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "hooks"))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
from preflight import preflight  # noqa: E402
import heavy_threads  # noqa: E402
import lib_git_hooks as hooks  # noqa: E402

BREAK_VARIABLE = "GIT_HOOKS_INSTALL_BREAK"
PEER_LIBRARY_VARIABLE = "GIT_HOOKS_INSTALL_PEER_LIBRARY"   # 自证里换成假的第二台库
SELFTEST_VARIABLE = "GIT_HOOKS_INSTALL_SELFTEST_DIRECTORY"
SELFTEST_MARKER_NAME = ".git-hooks-install-selftest-marker"
BREAKS = ("install-writes-nothing", "install-writes-relative-path", "install-writes-own-tree-path", "check-accepts-relative-path", "check-ignores-path", "check-ignores-exec-bit",
          "check-ignores-tracked-mode", "check-skips-added-hooks", "detector-scope-tree-only", "peer-skipped")
CORE_HOOK_NAMES = ("pre-commit", "pre-push")
ADDED_HOOK_NAMES = tuple(name for name in hooks.HOOK_NAMES if name not in CORE_HOOK_NAMES)
PEER_RUNNER = 'source "$1" || exit 70; peer_host_load_configuration "$2" "$3"; status=$?; if ((status != 0)); then exit "$status"; fi; peer_host_run "$3" "$4"'
PEER_SECONDS = 120
broken = set()


def inside_selftest():
    """自证标记在：SELFTEST_VARIABLE 点名的目录里有 SELFTEST_MARKER_NAME 那个文件（自证建临时目录时建、删临时目录时一起没）。祖先进程的命令行不算。"""
    directory = os.environ.get(SELFTEST_VARIABLE, "")
    return bool(directory) and os.path.isfile(os.path.join(directory, SELFTEST_MARKER_NAME))


def flat(text, limit=200):
    """子进程的输出折成一行再引：换行留着会让引文里以 ✗ 起头的行变成自证的失败行。"""
    return " ".join(str(text).split())[:limit]


def sample_root():
    """门禁样本根：当前目录里有 .gate-cells 或 .gate-break-sample、也没设 GATE_STAGED_FROM（门禁 code-tooling 的 git-hooks-installed 格在样本根里把 CODE_TOOLING_BREAK 折成这里的开关转来）。"""
    return (os.path.exists(".gate-cells") or os.path.exists(".gate-break-sample")) and not os.environ.get("GATE_STAGED_FROM")


def toplevel(directory):
    code, top = hooks.git_output(directory, "rev-parse", "--show-toplevel")
    return os.path.realpath(top) if code == 0 and top else None


def tracked_mode_problems(top):
    """进仓的钩子文件模式不是 100755 的（git 已经跟踪了它）：新克隆出来的仓里它没有执行位。"""
    if "check-ignores-tracked-mode" in broken:
        return []
    problems = []
    for name in hooks.HOOK_NAMES:
        code, listing = hooks.git_output(top, "ls-files", "-s", "--", os.path.join(hooks.HOOKS_DIRECTORY, name))
        mode = listing.split()[0] if code == 0 and listing else ""
        if mode and mode != "100755":
            problems.append(f"{os.path.join(hooks.HOOKS_DIRECTORY, name)} 进仓的模式是 {mode}，不是 100755（克隆出来的仓里没有执行位）")
    return problems


def tree_problems(top):
    problems = hooks.hooks_path_problems(top)
    if "check-ignores-path" in broken:
        problems = [problem for problem in problems if "不是进仓的钩子目录" not in problem]
    if "check-accepts-relative-path" in broken:
        problems = [problem for problem in problems if "是相对路径" not in problem]
    if "check-ignores-exec-bit" in broken:
        problems = [problem for problem in problems if "没有执行位" not in problem]
    code, _ = hooks.git_output(top, "rev-parse", "--path-format=absolute", "--git-common-dir")
    if code != 0:
        problems.append("这台的 git 不认 rev-parse --path-format=absolute（git 2.31 起才有），钩子取不到私有的提交窗口闸")
    problems += tracked_mode_problems(top)
    if "check-skips-added-hooks" in broken:
        problems = [problem for problem in problems if not any(name in problem for name in ADDED_HOOK_NAMES)]
    return problems


def detector_note(top):
    """检出钩子 ㉒ 对这棵树的态度：认得出是进仓钩子目录的树（lib_git_hooks.py 的 project_checkout）它拦 git commit / git push，别的树当别的仓放行。"""
    judged = (os.path.isfile(os.path.join(top, hooks.HOOKS_DIRECTORY, hooks.SENTINEL_HOOK_NAME)) if "detector-scope-tree-only" in broken else hooks.project_checkout(top))
    if judged:
        return "检出钩子 ㉒ 会拦这棵树里的 git commit / git push，直到上面的问题修好"
    return "检出钩子 ㉒ 当这棵树是别的仓、不拦它里面的 git commit / git push（这棵树不带进仓的钩子目录，core.hooksPath 也没指到它）"


def check_peer(top):
    """在第二台上跑同一条 check：交 (退出码, 输出)；退 3 是多机配置里双机没开。"""
    library = (os.environ.get(PEER_LIBRARY_VARIABLE) if inside_selftest() else "") or os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "peer-host-lib.sh")
    configuration_check = os.path.join(REPOSITORY_ROOT, "scripts", "crash", "crash-point-replay-shard-configuration-check.sh")
    command = f"python3 scripts/gate/git-hooks-install.py check {shlex.quote(top)} --this-machine-only"
    try:
        completed = subprocess.run(["bash", "-c", PEER_RUNNER, "peer", library, configuration_check, top, command], capture_output=True, text=True, timeout=PEER_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return 1, f"第二台没连上或超过 {PEER_SECONDS} 秒没答（{type(error).__name__}）"
    return completed.returncode, (completed.stdout + completed.stderr).strip()


def run_check(top, both_machines):
    problems = tree_problems(top)
    if both_machines and "peer-skipped" not in broken:
        code, output = check_peer(top)
        if code == 3:
            print("  ⊘ 多机配置里双机没开（ENABLE_ACROSS_MACHINES），只判了本机；仓、.git/config 与钩子在共享根下，第二台读到同一份")
        elif code != 0:
            problems.append(f"第二台上的 check 退 {code}：{flat(output, 300)}")
    if problems:
        for problem in problems:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ⊘ {detector_note(top)}")
        print(f"  ✗ {top} 里进仓的 git 钩子没装好（{len(problems)} 处，逐条列在上面）：提交与推送前的大文件判法就不起作用")
        print("     → 怎么办：python3 scripts/gate/git-hooks-install.py install（写 core.hooksPath、再核一遍）；钩子文件没有执行位的 chmod +x，"
              f"进仓的模式不对的 git add --chmod=+x -- {' '.join(os.path.join(hooks.HOOKS_DIRECTORY, name) for name in hooks.HOOK_NAMES)}；"
              "主工作树检出了不带钩子目录的老提交的，先 git switch 回带它的分支；第二台的问题在第二台上同样跑 check 看")
        return 1
    print(f"  ✓ {top} 里进仓的 git 钩子装好了：core.hooksPath 是主工作树钩子目录的绝对路径 {hooks.installed_hooks_path(top)}，{'、'.join(hooks.HOOK_NAMES)} 在、有执行位、进仓的模式是 100755"
          + ("，第二台同样" if both_machines and "peer-skipped" not in broken else ""))
    return 0


def run_install(top):
    value = hooks.HOOKS_DIRECTORY if "install-writes-relative-path" in broken else os.path.join(top, hooks.HOOKS_DIRECTORY) if "install-writes-own-tree-path" in broken else hooks.installed_hooks_path(top)
    if "install-writes-nothing" not in broken:
        completed = subprocess.run(["git", "-C", top, "config", "core.hooksPath", value], capture_output=True, text=True)
        if completed.returncode != 0:
            print(f"  ✗ git config core.hooksPath 退 {completed.returncode}：{flat(completed.stderr)}")
            print("     → 怎么办：按 git 的报错修好 .git/config 的权限，再 python3 scripts/gate/git-hooks-install.py install")
            return 1
    for name in hooks.HOOK_NAMES:
        path = os.path.join(hooks.installed_hooks_path(top), name)
        if os.path.isfile(path) and not os.access(path, os.X_OK):
            os.chmod(path, os.stat(path).st_mode | 0o111)
    return run_check(top, False)


# ── 自证 ──
HOOK_STUB = "#!/usr/bin/env bash\nexit 0\n"
FAKE_PEER_LIBRARY = """peer_host_load_configuration() { return "${FAKE_PEER_CONFIGURATION_EXIT:-0}"; }
peer_host_run() { printf 'fake peer ran in %s: %s\\n' "$1" "$2"; return "${FAKE_PEER_RUN_EXIT:-0}"; }
"""
CELLS = {}


def selftest_cell(function):
    """把函数登记成一格自证：格名是函数名里的下划线换成连字符；函数收一个 Sandbox，交 (对不对, 不对时的说明)。"""
    CELLS[function.__name__.replace("_", "-")] = function
    return function


class Sandbox:
    """一格自证的临时目录：仓与假的第二台库都建在里面；git 一律读不到用户与系统的配置；switch 是这一格设的弄坏开关（逗号隔开），传给自证起的 check 与 install。"""

    def __init__(self, directory, switch=""):
        self.directory = directory
        with open(os.path.join(directory, SELFTEST_MARKER_NAME), "w", encoding="utf-8") as handle:
            handle.write("自证的标记：这个目录在，脚本才认弄坏开关与假的第二台库\n")
        self.environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
        self.environment.update(GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1", GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@t", GIT_COMMITTER_NAME="t",
                                GIT_COMMITTER_EMAIL="t@t", GIT_CEILING_DIRECTORIES=directory, **{SELFTEST_VARIABLE: directory})
        if switch:
            self.environment[BREAK_VARIABLE] = switch

    def git(self, repository, *arguments):
        completed = subprocess.run(["git", "-C", repository, *arguments], capture_output=True, text=True, env=self.environment)
        if completed.returncode != 0:
            raise RuntimeError(f"装置里的 git {' '.join(arguments)} 退 {completed.returncode}：{flat(completed.stderr)}")
        return completed.stdout

    def write_stubs(self, repository, stub=HOOK_STUB):
        for hook in hooks.HOOK_NAMES:
            path = os.path.join(repository, hooks.HOOKS_DIRECTORY, hook)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(stub)
            os.chmod(path, 0o755)

    def repository(self, name, track_hooks=True):
        """一个小仓：第一笔提交没有钩子目录；第二笔把 HOOK_NAMES 的桩钩子（有执行位）提交进去（track_hooks 假时不建第二笔，桩钩子也不写）。"""
        path = os.path.join(self.directory, name)
        subprocess.run(["git", "init", "-q", path], check=True, env=self.environment, capture_output=True)
        self.git(path, "symbolic-ref", "HEAD", "refs/heads/main")
        with open(os.path.join(path, "base.txt"), "w", encoding="utf-8") as handle:
            handle.write("base\n")
        self.git(path, "add", "base.txt")
        self.git(path, "commit", "-q", "-m", "base")
        if track_hooks:
            self.write_stubs(path)
            self.git(path, "add", hooks.HOOKS_DIRECTORY)
            self.git(path, "commit", "-q", "-m", "hooks")
        return path

    def installed(self, name):
        """core.hooksPath 直接写成主工作树钩子目录的绝对路径（不经 install）的仓。"""
        path = self.repository(name)
        self.git(path, "config", "core.hooksPath", os.path.join(path, hooks.HOOKS_DIRECTORY))
        return path

    def linked_worktree(self, repository, name, *revision):
        """repository 的一棵链接工作树，revision 给了就检出那个提交（游离），没给就新建一个分支。"""
        linked = os.path.join(self.directory, name)
        if revision:
            self.git(repository, "worktree", "add", "-q", "--detach", linked, *revision)
        else:
            self.git(repository, "worktree", "add", "-q", "-b", name, linked)
        return linked

    def hooks_directory_of(self, repository):
        return os.path.join(repository, hooks.HOOKS_DIRECTORY)

    def run(self, *arguments, environment=None):
        return subprocess.run([sys.executable, "-B", os.path.realpath(__file__), *arguments], capture_output=True, text=True,
                              env=dict(self.environment, **(environment or {})))


def stdout_report(done):
    text = flat(done.stdout, 300)

    return f"退 {done.returncode}：{text}"


@selftest_cell
def not_installed_red(box):
    repository = box.repository("repo")
    done = box.run("check", repository)
    return done.returncode == 1 and "core.hooksPath 没设" in done.stdout, f"没装的仓 check 应当红、点名 core.hooksPath 没设；{stdout_report(done)}"


@selftest_cell
def install_then_check_green(box):
    repository = box.repository("repo")
    done = box.run("install", repository)
    configured = hooks.git_output(repository, "config", "--get", "core.hooksPath")[1]
    expected = box.hooks_directory_of(repository)
    return (done.returncode == 0 and os.path.isabs(configured) and os.path.realpath(configured) == os.path.realpath(expected),
            f"install 应当把 core.hooksPath 写成主工作树钩子目录的绝对路径 {expected} 并核绿；值 {configured!r}，{stdout_report(done)}")


@selftest_cell
def install_from_linked_worktree_writes_main_path(box):
    repository = box.repository("repo")
    linked = box.linked_worktree(repository, "linked")
    done = box.run("install", linked)
    configured = hooks.git_output(repository, "config", "--get", "core.hooksPath")[1]
    expected = box.hooks_directory_of(repository)
    return (done.returncode == 0 and os.path.isabs(configured) and os.path.realpath(configured) == os.path.realpath(expected),
            f"在链接工作树里 install 也应当写主工作树的钩子目录 {expected}（共用的 .git/config 只有一份）；值 {configured!r}，{stdout_report(done)}")


@selftest_cell
def relative_hooks_path_red(box):
    repository = box.repository("repo")
    box.git(repository, "config", "core.hooksPath", hooks.HOOKS_DIRECTORY)
    done = box.run("check", repository)
    return done.returncode == 1 and "是相对路径" in done.stdout, f"core.hooksPath 写成相对路径应当红、点名相对路径；{stdout_report(done)}"


@selftest_cell
def wrong_hooks_path_red(box):
    repository = box.repository("repo")
    box.git(repository, "config", "core.hooksPath", ".git/hooks")
    done = box.run("check", repository)
    return done.returncode == 1 and "不是进仓的钩子目录" in done.stdout, f"core.hooksPath 指到别处应当红；{stdout_report(done)}"


@selftest_cell
def linked_worktree_own_path_red(box):
    repository = box.repository("repo")
    linked = box.linked_worktree(repository, "linked")
    box.git(repository, "config", "core.hooksPath", box.hooks_directory_of(linked))
    done = box.run("check", linked)
    return done.returncode == 1 and "不是进仓的钩子目录" in done.stdout, f"core.hooksPath 指到链接工作树自己的钩子目录（链接工作树会被删）应当红，要的是主工作树的；{stdout_report(done)}"


def hook_problem_cell(kind, hook, break_it, wanted):
    """一份钩子一种问题一格：repository 里装好之后只把 hook 弄坏（break_it(box, 仓)），check 应当红、恰好 1 处、点名 wanted 那句。"""
    def cell(box):
        repository = box.installed("repo")
        break_it(box, repository, hook)
        done = box.run("check", repository)
        return (done.returncode == 1 and wanted(hook) in done.stdout and "（1 处，逐条列在上面）" in done.stdout,
                f"{hook} {kind}：check 应当红、恰好 1 处、点名「{wanted(hook)}」；{stdout_report(done)}")
    cell.__name__ = f"{kind}_{hook}_red".replace("-", "_")
    return cell


def remove_hook(box, repository, hook):
    os.remove(os.path.join(box.hooks_directory_of(repository), hook))


def drop_exec_bit(box, repository, hook):
    os.chmod(os.path.join(box.hooks_directory_of(repository), hook), 0o644)


def untrack_exec_mode(box, repository, hook):
    box.git(repository, "update-index", "--chmod=-x", "--", os.path.join(hooks.HOOKS_DIRECTORY, hook))   # 进仓的模式变 100644，工作区里的执行位还在


for hook_name in hooks.HOOK_NAMES:
    selftest_cell(hook_problem_cell("missing", hook_name, remove_hook, lambda name: f"{name} 不在"))
    selftest_cell(hook_problem_cell("no-exec-bit", hook_name, drop_exec_bit, lambda name: f"{name} 没有执行位"))
    selftest_cell(hook_problem_cell("tracked-mode", hook_name, untrack_exec_mode, lambda name: f"{os.path.join(hooks.HOOKS_DIRECTORY, name)} 进仓的模式是 100644"))


@selftest_cell
def old_commit_linked_worktree_hooks_run(box):
    repository = box.repository("repo")
    log = os.path.join(box.directory, "pre-commit-ran.log")
    with open(os.path.join(box.hooks_directory_of(repository), "pre-commit"), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\necho ran >> {shlex.quote(log)}\n")
    box.run("install", repository)
    linked = box.linked_worktree(repository, "linked", "HEAD~1")   # 检出的是不带钩子目录的老提交
    checked = box.run("check", linked)
    box.git(linked, "commit", "-q", "--allow-empty", "-m", "x")
    return (checked.returncode == 0 and os.path.exists(log),
            f"检出老提交的链接工作树里 check 应当绿、git commit 应当起主工作树的 pre-commit；check {stdout_report(checked)}，钩子{'起了' if os.path.exists(log) else '没起'}")


@selftest_cell
def main_tree_old_commit_red(box):
    repository = box.installed("repo")
    box.git(repository, "checkout", "-q", "--detach", "HEAD~1")   # 主工作树自己检出了不带钩子目录的老提交
    done = box.run("check", repository)
    return (done.returncode == 1 and "不在" in done.stdout and "检出钩子 ㉒ 会拦" in done.stdout and hooks.project_checkout(repository),
            f"主工作树检出老提交、钩子目录不在：check 应当红、写检出钩子 ㉒ 会拦这棵树，project_checkout 应当认得出它；{stdout_report(done)}")


@selftest_cell
def hooks_staged_then_reset_hard_red(box):
    repository = box.repository("repo", track_hooks=False)
    box.write_stubs(repository)
    box.git(repository, "add", hooks.HOOKS_DIRECTORY)   # 钩子只在暂存区里
    box.git(repository, "config", "core.hooksPath", box.hooks_directory_of(repository))
    box.git(repository, "reset", "-q", "--hard")        # 暂存区里不在 HEAD 的文件连工作区里的一起删
    done = box.run("check", repository)
    return (done.returncode == 1 and "不在" in done.stdout and "检出钩子 ㉒ 会拦" in done.stdout and hooks.project_checkout(repository),
            f"钩子只在暂存区时 git reset --hard 把它们删了：check 应当红、写检出钩子 ㉒ 会拦这棵树，project_checkout 应当认得出它；{stdout_report(done)}")


@selftest_cell
def linked_worktree_green(box):
    repository = box.installed("repo")
    linked = box.linked_worktree(repository, "linked")
    done = box.run("check", linked)
    return done.returncode == 0, f"链接工作树读共用的 core.hooksPath、指到主工作树的钩子目录，应当绿；{stdout_report(done)}"


# 第二台：假的 peer-host-lib.sh 里两个函数的退出码由环境变量定
def fake_peer_library(box):
    library = os.path.join(box.directory, "fake-peer-lib.sh")
    with open(library, "w", encoding="utf-8") as handle:
        handle.write(FAKE_PEER_LIBRARY)
    return {PEER_LIBRARY_VARIABLE: library}


@selftest_cell
def peer_green(box):
    repository = box.installed("repo")
    done = box.run("check", repository, "--both-machines", environment=fake_peer_library(box))
    return done.returncode == 0 and "第二台同样" in done.stdout, f"第二台上的 check 退 0 应当绿；{stdout_report(done)}"


@selftest_cell
def peer_failure_red(box):
    repository = box.installed("repo")
    done = box.run("check", repository, "--both-machines", environment=dict(fake_peer_library(box), FAKE_PEER_RUN_EXIT="1"))
    return done.returncode == 1 and "第二台上的 check 退 1" in done.stdout, f"第二台上的 check 退 1 应当红；{stdout_report(done)}"


@selftest_cell
def peer_switch_off_warned(box):
    repository = box.installed("repo")
    done = box.run("check", repository, "--both-machines", environment=dict(fake_peer_library(box), FAKE_PEER_CONFIGURATION_EXIT="3"))
    return done.returncode == 0 and "双机没开" in done.stdout, f"多机配置里双机没开：只判本机、写一行 ⊘、不红；{stdout_report(done)}"


def named(*hook_names, kind):
    return {f"{kind}-{name}-red" for name in hook_names}


EXPECTED_RED = {
    "install-writes-nothing": {"install-then-check-green", "install-from-linked-worktree-writes-main-path", "old-commit-linked-worktree-hooks-run"},
    "install-writes-relative-path": {"install-then-check-green", "install-from-linked-worktree-writes-main-path", "old-commit-linked-worktree-hooks-run"},
    "install-writes-own-tree-path": {"install-from-linked-worktree-writes-main-path"},
    "check-accepts-relative-path": {"relative-hooks-path-red"},
    "check-ignores-path": {"wrong-hooks-path-red", "linked-worktree-own-path-red"},
    "check-ignores-exec-bit": named(*hooks.HOOK_NAMES, kind="no-exec-bit"),
    "check-ignores-tracked-mode": named(*hooks.HOOK_NAMES, kind="tracked-mode"),
    "check-skips-added-hooks": named(*ADDED_HOOK_NAMES, kind="missing") | named(*ADDED_HOOK_NAMES, kind="no-exec-bit") | named(*ADDED_HOOK_NAMES, kind="tracked-mode"),
    "detector-scope-tree-only": {"main-tree-old-commit-red", "hooks-staged-then-reset-hard-red"},
    "peer-skipped": {"peer-green", "peer-failure-red", "peer-switch-off-warned"},
}


def run_cell(task):
    """一格自证：自己建临时目录与仓、自己删；装置出错（git 起不来、文件写不进）也是这一格红，不让整份自证崩。"""
    name, switch = task
    with tempfile.TemporaryDirectory(prefix="git-hooks-install-selftest-") as directory:
        try:
            ok, message = CELLS[name](Sandbox(directory, switch))
        except (RuntimeError, OSError, IndexError, ValueError, subprocess.SubprocessError) as error:
            ok, message = False, f"装置出错（{type(error).__name__}）：{flat(error)}"
    return name, ok, message


def run_cells(tasks, jobs):
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        return list(pool.map(run_cell, tasks))


def switch_wheel(jobs):
    """逐个打开文件头登记的弄坏开关，各自要红在点名的格（含点名的，且只红点名的）；开关与格拼成一个任务表、并进同一个线程池。"""
    tasks = [(name, switch) for switch in EXPECTED_RED for name in CELLS]
    red_of = {switch: set() for switch in EXPECTED_RED}
    for (name, switch), (_name, ok, _message) in zip(tasks, run_cells(tasks, jobs)):
        if not ok:
            red_of[switch].add(name)
    failures = 0
    for switch, cells in EXPECTED_RED.items():
        if red_of[switch] == cells:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(sorted(cells))}」，与点名的相同")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后红在「{'、'.join(sorted(red_of[switch])) or '（没有）'}」，点名的是「{'、'.join(sorted(cells))}」")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(EXPECTED_RED)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(EXPECTED_RED)} 个开关逐个打开（{len(tasks)} 格次，并行 {jobs} 路），各红在点名的格")
    return 0


def run_selftest(names):
    unknown = [name for name in names if name not in CELLS]
    if unknown:
        print(f"  ✗ 没有这几格自证：{'、'.join(unknown)}", file=sys.stderr)
        print(f"     → 怎么办：点名的格名从下面选：{'、'.join(CELLS)}", file=sys.stderr)
        return 2
    try:
        jobs = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return 2
    switch = ",".join(item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item in BREAKS)
    results = run_cells([(name, switch) for name in names or CELLS], jobs)
    for name, passed, message in results:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    failed = [name for name, passed, _ in results if not passed]
    if failed:
        print(f"  ✗ --selftest：{len(results)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(results)} 格都对（并行 {jobs} 路；没装、装上、从链接工作树里装、写成相对路径、指错、指到链接工作树自己的钩子目录、"
          "五份钩子各自缺、没有执行位、进仓模式不对、检出老提交的链接工作树里钩子照跑、主工作树检出老提交与钩子只在暂存区时 reset --hard 之后红且检出钩子 ㉒ 会拦、"
          "链接工作树绿、第二台判红判绿与双机没开）")
    return 0 if names or switch else switch_wheel(jobs)


def settle_break_switches(arguments):
    value = os.environ.get(BREAK_VARIABLE)
    if not value or arguments[:1] == ["--selftest"] or inside_selftest() or sample_root():
        return None
    print(f"  ✗ {BREAK_VARIABLE} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）与门禁样本根里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 python3 scripts/gate/git-hooks-install.py --selftest（末尾逐个打开）", file=sys.stderr)
    return 2


def usage_error(text):
    print(f"  ✗ {text}", file=sys.stderr)
    print("     → 怎么办：用法 git-hooks-install.py install [<仓根>] | check [<仓根>] [--both-machines] | --selftest [<格名>…]", file=sys.stderr)
    return 2


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest(arguments[1:])
    broken.update(item for item in (os.environ.get(BREAK_VARIABLE, "") if inside_selftest() or sample_root() else "").split(",") if item in BREAKS)
    if not arguments or arguments[0] not in ("install", "check"):
        return usage_error(f"认不得的参数：{' '.join(arguments)}")
    command, rest = arguments[0], arguments[1:]
    both_machines = "--both-machines" in rest
    rest = [word for word in rest if word not in ("--both-machines", "--this-machine-only")]
    if len(rest) > 1 or any(word.startswith("-") for word in rest) or (command == "install" and both_machines):
        return usage_error(f"认不得的参数：{' '.join(rest)}")
    root = rest[0] if rest else REPOSITORY_ROOT
    top = toplevel(root) if os.path.isdir(root) else None
    if top is None:
        return usage_error(f"{root} 不是 git 仓里的目录")
    return run_install(top) if command == "install" else run_check(top, both_machines)


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
