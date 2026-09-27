"""自证的临时目录用完即删：几份脚本的自证共用的建法与三格探查。

门禁每轮给每个阶段一个私有 TMPDIR，跑完里面还剩东西就判红（.claude/singlefs-ai-sop/rules/command-safety.md「测试镜像一律放临时目录」）。
用这份库的自证把主体交给 run_with_scratch(主体, 前缀, keep, 角色变量, 拉起自证的命令)，再把它回的两样交给 merged_exit_code：
  通过那条路   主体跑的时候 TMPDIR（环境变量与 tempfile.tempdir）指进一个探查目录，主体回来之后那里必须是空的；不另跑一遍主体。
  判红、抛异常 拉起被测脚本自己的子进程，环境里设角色变量（red / crash），TMPDIR 各指进一个探查目录；子进程在临时目录建好之后
               往里写一个文件，red 打一行判红、退 1，crash 抛异常；子进程退出之后探查目录必须是空的。
弄坏开关：被测脚本在自己的弄坏开关设着时把 keep=True 传进来，临时目录走回 mkdtemp 不删，三格都必须判红。
用它的：research/scripts/quote-kb.py、research/scripts/agent-watch.py、.claude/hooks/bash-command-detector.sh、.claude/hooks/session-start.sh。
"""
import contextlib
import os
import shutil
import subprocess
import tempfile

ROLE_RED = "red"
ROLE_CRASH = "crash"
RED_LABEL = "子进程按自证角色 red 判红"
CRASH_MARKER = "自证角色 crash 故意抛的异常"
CELLS = 3   # 通过、判红、抛异常三条路各一格
CHILD_TIMEOUT_SECONDS = 120   # 判红与抛异常的子进程在临时目录建好之后当场就退，不跑主体
PASS_PROBE_PREFIX = "selftest-scratch-pass-probe-"
CHILD_PROBE_PREFIX = "selftest-scratch-child-probe-"


@contextlib.contextmanager
def scratch_directory(prefix, keep):
    """自证的临时目录：keep 为假时通过、判红、抛异常都删（TemporaryDirectory）；keep 为真是弄坏开关，mkdtemp 建了不删。"""
    if keep:
        yield tempfile.mkdtemp(prefix=prefix)
        return
    with tempfile.TemporaryDirectory(prefix=prefix) as directory:
        yield directory


class PassPathProbe:
    """通过那条路的探查目录：directory 是它的路径，left_behind 是主体回来之后里面还剩的名字。"""

    def __init__(self):
        self.directory = None
        self.left_behind = []


@contextlib.contextmanager
def pass_path_probe(enabled):
    """enabled 时建一个探查目录，把 TMPDIR 与 tempfile.tempdir 指进去；退出时换回原样，记下里面还剩什么，再整个删掉。"""
    probe = PassPathProbe()
    if not enabled:
        yield probe
        return
    probe.directory = tempfile.mkdtemp(prefix=PASS_PROBE_PREFIX)
    saved_environment_value = os.environ.get("TMPDIR")
    saved_tempdir = tempfile.tempdir
    os.environ["TMPDIR"] = probe.directory
    tempfile.tempdir = probe.directory
    try:
        yield probe
    finally:
        if saved_environment_value is None:
            os.environ.pop("TMPDIR", None)
        else:
            os.environ["TMPDIR"] = saved_environment_value
        tempfile.tempdir = saved_tempdir
        probe.left_behind = sorted(os.listdir(probe.directory))
        shutil.rmtree(probe.directory, ignore_errors=True)


def act_as_role(role, scratch):
    """子进程里在临时目录建好之后调：red 与 crash 先往临时目录写一个文件；red 打一行判红、回 1，crash 抛异常；没有角色回 None，照常跑主体。"""
    if role not in (ROLE_RED, ROLE_CRASH):
        return None
    with open(os.path.join(scratch, "left-by-" + role), "w", encoding="utf-8") as handle:
        handle.write(role + "\n")
    if role == ROLE_CRASH:
        raise RuntimeError(CRASH_MARKER)
    print(f"  ✗ 自检：{RED_LABEL}")
    print("    → 这是自证拉起的子进程故意走的判红那条路，看的是它退出之后临时目录删没删，不是被测脚本坏了")
    return 1


def child_path_problems(command, role_variable):
    """判红、抛异常两条路：拉起子进程走，TMPDIR 各指进一个探查目录；回问题列表（空 = 两格都过）。"""
    problems = []
    with tempfile.TemporaryDirectory(prefix=CHILD_PROBE_PREFIX) as probe:
        for role, expected_text in ((ROLE_RED, RED_LABEL), (ROLE_CRASH, CRASH_MARKER)):
            child_temporary_directory = os.path.join(probe, role)
            os.mkdir(child_temporary_directory)
            environment = dict(os.environ, TMPDIR=child_temporary_directory)
            environment[role_variable] = role
            try:
                child = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=CHILD_TIMEOUT_SECONDS)
            except subprocess.TimeoutExpired:
                problems.append(f"{role} 那条路的子进程 {CHILD_TIMEOUT_SECONDS} 秒没退出：{' '.join(command)}")
                continue
            child_output = child.stdout + child.stderr
            left_behind = sorted(os.listdir(child_temporary_directory))
            if child.returncode == 0 or expected_text not in child_output:
                problems.append(f"{role} 那条路没走到（子进程退出码 {child.returncode}，输出里没有「{expected_text}」）：{child_output[-400:]}")
            elif left_behind:
                problems.append(f"{role} 那条路上临时目录没删：TMPDIR 里留下 {'、'.join(left_behind)}")
    return problems


def run_with_scratch(body, prefix, keep, role_variable, command):
    """跑自证的主体 body(临时目录)，回 (主体的退出码, 临时目录三格的问题列表)。
    环境里有角色变量时是被拉起的子进程：只走那一条路，问题列表回 None（这一层不判那三格），不再往下拉子进程。"""
    role = os.environ.get(role_variable, "")
    with pass_path_probe(enabled=not role) as probe:
        with scratch_directory(prefix, keep) as scratch:
            exit_code = act_as_role(role, scratch)
            if exit_code is None:
                exit_code = body(scratch)
    if role:
        return exit_code, None
    problems = []
    if probe.left_behind:
        problems.append(f"通过那条路上临时目录没删：主体跑的时候 TMPDIR 指着的探查目录里留下 {'、'.join(probe.left_behind)}")
    return exit_code, problems + child_path_problems(command, role_variable)


def merged_exit_code(exit_code, problems, where_to_look, break_switch):
    """把 run_with_scratch 回的两样并成自证的退出码：三格有问题就逐条报、给出路、回 1；都过就报一行，回主体的退出码。
    problems 是 None（被拉起的子进程）时不判、不报，原样回主体的退出码。break_switch 写被测脚本自己那个让临时目录不删的弄坏开关。"""
    if problems is None:
        return exit_code
    for problem in problems:
        print(f"  ✗ 自检：{problem}")  # gate-lint:detail
    if problems:
        print(f"    → 看 {where_to_look}：临时目录要用 run_with_scratch 建（通过、判红、抛异常都删），主体里别再自己 mkdtemp；"
              f"{break_switch} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检：临时目录在通过、判红、抛异常三条路上都删了（{CELLS} 格：通过那条路 TMPDIR 指进探查目录，另两条拉起子进程走）")
    return exit_code
