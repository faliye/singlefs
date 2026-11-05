#!/usr/bin/env python3
# admission: always 判的是此刻环境里的 TASK_SERVER_SOURCE_REPOSITORY 与此刻盘上的会话记录目录，上一次的结论不替这一次作保
# run-condition: none 被 import 的函数库（只用标准库，import 本身不读文件、不起子进程）；直接执行只打一行目录或跑自证，除了 python3 没有环境要求
"""读会话记录的脚本取默认会话目录的唯一取法：被 import 的函数库，也可直接执行打一行目录（watch.sh 经它取，bash 里不另拼）。

    sys.path.insert(0, <仓根>/scripts/agents); import lib_session_directory as session_directories

为什么：任务两台都可能派到，脚本不得绑定一台机器。会话记录目录两台挂在同一绝对路径上，但第二台的家目录是另一个、脚本所在的仓路径也可能不同
（拷树那时 agent-cost.py 在第二台拼出拷贝的路径、退 2），所以不经 $HOME 与脚本自己的路径拼。

取法（Claude Code 按项目的绝对路径把 / 换成 - 起会话目录名：<家目录>/.claude/projects/<仓根路径把 / 换成 ->）：
  环境变量 TASK_SERVER_SOURCE_REPOSITORY 设了（统一任务服务给每次尝试带的、提交方那一份仓的绝对路径，本机与第二台都给同一个值）：
    仓路径取它；家目录从这个路径取——它的各层上级里最近的一层，其 .claude/projects/<这个仓路径把 / 换成 -> 在盘上（会话记录就在那里）；
    没有这样的，取最近的一层带 .claude/projects 的（会话目录还没建过）；还没有，路径是 /home/<用户>/…（或 /Users/<用户>/…）的取 /home/<用户>，
    不是这种形状的报错（SessionDirectoryError，说明里写出路）。不是绝对路径的报错。不用 $HOME。
  没设（服务外手跑）：仓路径取调用方给的、脚本自己所在的仓；家目录取本机的（环境里的 HOME，没有取 expanduser）。
  调用方另给了仓（例 move-directory-in-use-check.py 的 --repository）：会话目录名按那个仓拼，家目录照上面取。
  - session_directory(own_repository, repository=None, environment=None)   <家目录>/.claude/projects/<仓路径改写>
  - projects_root(environment=None)                                          <家目录>/.claude/projects（各项目的会话目录都在它底下）
  - home_directory(environment=None)、source_repository(environment=None)、mangle(path)
  environment 不给取 os.environ（自证传造出来的字典）。

命令行：
  python3 scripts/agents/lib_session_directory.py session-directory [--own-repository <脚本所在的仓>] [--repository <另指的仓>]   打一行会话目录
  python3 scripts/agents/lib_session_directory.py projects-root                                                              打一行 <家目录>/.claude/projects
  python3 scripts/agents/lib_session_directory.py --selftest
  取不到（变量不是绝对路径、找不到家目录）退 2，标准错误写出路，标准输出空。

直接执行而不是 --selftest 时设了 SESSION_DIRECTORY_BREAK：参数解析之前就拒、退 2，点名它只在自证里认（refuse_break_outside_selftest()）；
被 import 时照认（五份调用方的自证要用它，它们各自的入口守卫也拒平常跑时设着它）。

弄坏开关（只给自证证红用；被 import 时五份调用方的自证同样红）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（失败行「✗ lib_session_directory 自证：<格名>：…」）：
  SESSION_DIRECTORY_BREAK=home-from-environment  home_directory() 设了 TASK_SERVER_SOURCE_REPOSITORY 也照改前取环境里的 HOME（改前：~/.claude/projects 接脚本自己所在的仓路径）
      「✗ lib_session_directory 自证：设了来源仓根时家目录从它的路径取、不用 HOME」
      「✗ lib_session_directory 自证：会话目录还没建过时取最近一层带 .claude/projects 的上级」
      「✗ lib_session_directory 自证：来源仓根在 /home/<用户> 底下而哪层都没有 .claude/projects 时按 /home/<用户> 取」
      「✗ lib_session_directory 自证：另指了仓时会话目录名按那个仓拼、家目录仍从来源仓根取」
  SESSION_DIRECTORY_BREAK=honor-break-outside-selftest  refuse_break_outside_selftest() 不拒：直接执行而不是 --selftest 时设了开关也照常往下走（改前的样子）
      「✗ lib_session_directory 自证：平常跑设开关要被拒」
"""
import os
import shutil
import subprocess
import sys
import tempfile

SOURCE_REPOSITORY_VARIABLE = "TASK_SERVER_SOURCE_REPOSITORY"   # 统一任务服务每次尝试交的提交方那一份仓的绝对路径（task-server/task_server_lib.py 的 SOURCE_REPOSITORY_VARIABLE）
BREAK_VARIABLE = "SESSION_DIRECTORY_BREAK"
KNOWN_BREAK_MODES = ("home-from-environment", "honor-break-outside-selftest")
HOME_PARENT_NAMES = ("home", "Users")   # 家目录在 /<这两个名字之一>/<用户> 的形状时，哪层都没有 .claude/projects 也按它取


class SessionDirectoryError(ValueError):
    """取不到会话目录：message 写什么不对，howto 写出路。"""

    def __init__(self, message, howto):
        super().__init__(message)
        self.howto = howto


def break_mode():
    """弄坏开关的值；认不出的值抛 ValueError（拼错的开关不能被当成「没开」）。"""
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise ValueError(f"{BREAK_VARIABLE}={mode} 认不出（只认 {'、'.join(KNOWN_BREAK_MODES)}）")
    return mode


def mangle(path):
    """Claude Code 起会话目录名：仓的绝对路径把 / 换成 -。"""
    return path.replace("/", "-")


def source_repository(environment=None):
    """TASK_SERVER_SOURCE_REPOSITORY 的值（规范成不带结尾斜杠的绝对路径）；没设或空是 None；设了而不是绝对路径抛 SessionDirectoryError。"""
    environment = os.environ if environment is None else environment
    value = environment.get(SOURCE_REPOSITORY_VARIABLE, "")
    if not value:
        return None
    if not os.path.isabs(value):
        raise SessionDirectoryError(f"{SOURCE_REPOSITORY_VARIABLE}={value} 不是绝对路径",
                                    f"统一任务服务给的是提交方那份仓的绝对路径；手跑时去掉 {SOURCE_REPOSITORY_VARIABLE}，或写成绝对路径")
    return os.path.normpath(value)


def home_of_repository(repository):
    """会话记录所在的家目录，从来源仓根的路径取（文件头「取法」）。"""
    ancestors = []
    current = os.path.dirname(repository)
    while current != "/":
        ancestors.append(current)
        current = os.path.dirname(current)
    mangled = mangle(repository)
    for candidate in ancestors:
        if os.path.isdir(os.path.join(candidate, ".claude", "projects", mangled)):
            return candidate
    for candidate in ancestors:
        if os.path.isdir(os.path.join(candidate, ".claude", "projects")):
            return candidate
    parts = repository.split("/")
    if len(parts) >= 4 and parts[1] in HOME_PARENT_NAMES:
        return "/".join(parts[:3])
    raise SessionDirectoryError(f"从 {repository} 的上级目录里找不到会话记录（没有哪一层带 .claude/projects/{mangled}，路径也不是 /home/<用户>/… 的形状）",
                                "手给会话目录（agent-cost.py 的 --root、move-directory-in-use-check.py 的 --projects-directory 这类选项），或核对 "
                                f"{SOURCE_REPOSITORY_VARIABLE} 指的是不是跑 Claude Code 的那份仓")


def home_directory(environment=None):
    """设了 TASK_SERVER_SOURCE_REPOSITORY 从它的路径取家目录；没设取环境里的 HOME（没有取 expanduser）。弄坏开关 home-from-environment 下设了也取 HOME。"""
    environment = os.environ if environment is None else environment
    source = source_repository(environment)
    if source is None or break_mode() == "home-from-environment":
        return environment.get("HOME") or os.path.expanduser("~")
    return home_of_repository(source)


def projects_root(environment=None):
    """<家目录>/.claude/projects：各项目的会话目录都在它底下。"""
    return os.path.join(home_directory(environment), ".claude", "projects")


def session_directory(own_repository, repository=None, environment=None):
    """仓的会话目录 <家目录>/.claude/projects/<仓路径改写>。仓路径：调用方另给的 repository，其次 TASK_SERVER_SOURCE_REPOSITORY，其次 own_repository（脚本自己所在的仓）。"""
    environment = os.environ if environment is None else environment
    return os.path.join(projects_root(environment), mangle(repository or source_repository(environment) or own_repository))


def refuse_break_outside_selftest(arguments):
    """直接执行时、参数解析之前调：不是 --selftest 而设了弄坏开关就打拒绝、交 2；没设或是 --selftest 交 None。"""
    mode = os.environ.get(BREAK_VARIABLE, "")
    if not mode or arguments == ["--selftest"] or mode == "honor-break-outside-selftest":
        return None
    print(f"  ✗ {BREAK_VARIABLE}={mode} 只在自证里认（只给 --selftest 证红用），这一趟不是 --selftest，不往下走", file=sys.stderr)
    print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/agents/lib_session_directory.py --selftest", file=sys.stderr)
    return 2


def build_selftest_tree(work):
    """在 work 里造一棵自证用的树，五份调用方的自证共用：来源仓（家目录 home 底下）、它的会话目录（里面有会话 selftest-session）、脚本自己所在的另一份仓 own_repository、
    一个别处的家目录 other_home（里面有脚本自己那份仓的会话目录，找错了会找到一个在但不对的目录），和 HOME 指到别处、来源仓根指到来源仓的环境字典。
    交回 {名字: 路径或字典}。来源仓的上级里另放一层带 .claude/projects 却没有这个仓会话目录的 code/，找家目录取最近一层带这个仓会话目录的，不取它。"""
    home = os.path.join(work, "home")
    source = os.path.join(home, "code", "source-repository")
    own_repository = os.path.join(work, "script-tree", "repository")
    other_home = os.path.join(work, "other-home")
    session = os.path.join(home, ".claude", "projects", mangle(source))
    for directory in (source, own_repository, os.path.join(session, "selftest-session", "subagents"), os.path.join(home, "code", ".claude", "projects"),
                      os.path.join(other_home, ".claude", "projects", mangle(own_repository), "selftest-session", "subagents")):
        os.makedirs(directory)
    return {"home": home, "other_home": other_home, "source_repository": source, "own_repository": own_repository,
            "session_directory": session, "projects_root": os.path.join(home, ".claude", "projects"),
            "environment": {"HOME": other_home, SOURCE_REPOSITORY_VARIABLE: source}}


def run_selftest():
    work = tempfile.mkdtemp(prefix="session-directory-selftest-")
    failures, checked = [], 0
    try:
        tree = build_selftest_tree(work)
        environment, own = tree["environment"], tree["own_repository"]
        # 设了来源仓根：HOME 指到别处、脚本自己所在的仓也在别处，仍取到来源仓的会话目录（来源仓上级里近的一层 code/ 带 .claude/projects 而没有这个仓的会话目录，不取它）
        checked += 1
        got = (session_directory(own, environment=environment), projects_root(environment), home_directory(environment))
        want = (tree["session_directory"], tree["projects_root"], tree["home"])
        if got != want:
            failures.append(f"设了来源仓根时家目录从它的路径取、不用 HOME：（会话目录, projects, 家目录）应当 {want}，实际 {got}")
        # 没设：脚本自己所在的仓、环境里的 HOME
        checked += 1
        unset = {"HOME": tree["home"]}
        got, want = session_directory(own, environment=unset), os.path.join(tree["home"], ".claude", "projects", mangle(own))
        if got != want:
            failures.append(f"没设来源仓根时取脚本自己所在的仓与本机家目录：应当 {want}，实际 {got}")
        # 设了而是空串，同没设
        checked += 1
        got = session_directory(own, environment={"HOME": tree["home"], SOURCE_REPOSITORY_VARIABLE: ""})
        if got != want:
            failures.append(f"来源仓根是空串时同没设：应当 {want}，实际 {got}")
        # 会话目录还没建过：取最近一层带 .claude/projects 的上级
        checked += 1
        lonely_home = os.path.join(work, "lonely-home")
        lonely_source = os.path.join(lonely_home, "code", "fresh-repository")
        os.makedirs(os.path.join(lonely_home, ".claude", "projects"))
        os.makedirs(lonely_source)
        got = session_directory(own, environment={"HOME": tree["other_home"], SOURCE_REPOSITORY_VARIABLE: lonely_source})
        want = os.path.join(lonely_home, ".claude", "projects", mangle(lonely_source))
        if got != want:
            failures.append(f"会话目录还没建过时取最近一层带 .claude/projects 的上级：应当 {want}，实际 {got}")
        # 哪层都没有 .claude/projects：路径是 /home/<用户>/… 的形状就按 /home/<用户> 取，别的形状报错
        checked += 1
        guessed_source = "/home/session-directory-selftest-user/workspace/repository"
        got = session_directory(own, environment={"HOME": tree["other_home"], SOURCE_REPOSITORY_VARIABLE: guessed_source})
        want = os.path.join("/home/session-directory-selftest-user", ".claude", "projects", mangle(guessed_source))
        problem = None
        try:
            session_directory(own, environment={"HOME": tree["other_home"], SOURCE_REPOSITORY_VARIABLE: "/session-directory-selftest-root/repository"})
        except SessionDirectoryError as error:
            problem = error
        if got != want or problem is None or not problem.howto:
            failures.append(f"来源仓根在 /home/<用户> 底下而哪层都没有 .claude/projects 时按 /home/<用户> 取：应当 {want}，实际 {got}；"
                            f"不是这种形状的应当报 SessionDirectoryError 并写出路，实际 {problem!r}")
        # 来源仓根不是绝对路径
        checked += 1
        problem = None
        try:
            session_directory(own, environment={"HOME": tree["home"], SOURCE_REPOSITORY_VARIABLE: "relative/repository"})
        except SessionDirectoryError as error:
            problem = error
        if problem is None or not problem.howto:
            failures.append(f"来源仓根不是绝对路径应当报 SessionDirectoryError 并写出路，实际 {problem!r}")
        # 另指了仓：会话目录名按它拼，家目录仍从来源仓根取
        checked += 1
        got = session_directory(own, repository="/session-directory-selftest/other/repository", environment=environment)
        want = os.path.join(tree["projects_root"], mangle("/session-directory-selftest/other/repository"))
        if got != want:
            failures.append(f"另指了仓时会话目录名按那个仓拼、家目录仍从来源仓根取：应当 {want}，实际 {got}")
        # 命令行入口打出同一个目录（子进程的环境去掉弄坏开关：入口守卫平常跑就拒它）
        checked += 1
        child_environment = {name: text for name, text in os.environ.items() if name != BREAK_VARIABLE}
        child_environment.update(environment)
        results = {}
        for label, command in (("session-directory", ["session-directory", "--own-repository", own]), ("projects-root", ["projects-root"])):
            done = subprocess.run([sys.executable, "-B", os.path.realpath(__file__)] + command, env=child_environment, stdin=subprocess.DEVNULL,
                                  capture_output=True, text=True, errors="replace", timeout=60)
            results[label] = (done.returncode, done.stdout.strip())
        bad_source = dict(child_environment, **{SOURCE_REPOSITORY_VARIABLE: "relative/repository"})
        done = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "projects-root"], env=bad_source, stdin=subprocess.DEVNULL,
                              capture_output=True, text=True, errors="replace", timeout=60)
        expected = {"session-directory": (0, tree["session_directory"]), "projects-root": (0, tree["projects_root"])}
        if results != expected or done.returncode != 2 or done.stdout.strip() or "怎么办" not in done.stderr:
            failures.append(f"命令行入口打出同一个目录、取不到时退 2 并写出路：应当 {expected}，实际 {results}；来源仓根不是绝对路径应当退 2、标准输出空、标准错误有出路，"
                            f"实际退 {done.returncode}、标准输出 {done.stdout.strip()!r}、标准错误 {done.stderr.strip()[-200:]!r}")
        # 平常跑设开关要被拒：起自己一次（不是 --selftest），设上开关要退 2、点名它只在自证里认；不设就不拒（照常报认不出的用法）
        checked += 1
        own_mode = os.environ.get(BREAK_VARIABLE, "")
        probe_mode = own_mode if own_mode == "honor-break-outside-selftest" else "home-from-environment"
        probes = {}
        for label, value in (("设上", probe_mode), ("不设", None)):
            probe_environment = {name: text for name, text in os.environ.items() if name != BREAK_VARIABLE}
            if value is not None:
                probe_environment[BREAK_VARIABLE] = value
            done = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--break-guard-probe-unknown-option"], env=probe_environment,
                                  stdin=subprocess.DEVNULL, capture_output=True, text=True, errors="replace", timeout=60)
            named = [row for row in (done.stdout + done.stderr).splitlines() if BREAK_VARIABLE in row and "自证" in row]
            probes[label] = (done.returncode, named)
        if probes["设上"][0] != 2 or not probes["设上"][1] or probes["不设"][1]:
            failures.append(f"平常跑设开关要被拒：设上 {BREAK_VARIABLE}={probe_mode} 应当退 2 并点名它只在自证里认、不设不点名，"
                            f"实际设上退 {probes['设上'][0]}、点名 {len(probes['设上'][1])} 行，不设点名 {len(probes['不设'][1])} 行")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ lib_session_directory 自证：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 home_directory()、home_of_repository()、session_directory() 与 refuse_break_outside_selftest()；SESSION_DIRECTORY_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_session_directory 自证通过（查了 {checked} 项）：设了来源仓根时家目录从它的路径取、不用 HOME（HOME 指到别处、脚本自己所在的仓在别处也取到来源仓的会话目录），"
          "没设时取脚本自己所在的仓与本机家目录，会话目录还没建过的与 /home/<用户> 底下的取得到、不是绝对路径的报错，命令行入口打出同一个目录；"
          "直接执行不是 --selftest 而设了开关的被拒、退 2")
    return 0


def run_command(arguments):
    commands = ("projects-root", "session-directory")
    if not arguments or arguments[0] not in commands:
        print(f"  ✗ 认不出的用法：{' '.join(arguments)}", file=sys.stderr)
        print("     → 怎么办：写 session-directory [--own-repository 仓] [--repository 仓]、projects-root 或 --selftest；被 import 时调 session_directory()、projects_root()", file=sys.stderr)
        return 2
    command, own_repository, repository, rest = arguments[0], os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")), None, arguments[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--own-repository" and rest:
            own_repository = rest.pop(0)
        elif flag == "--repository" and rest and command == "session-directory":
            repository = rest.pop(0)
        else:
            print(f"  ✗ 认不出参数 {flag}", file=sys.stderr)
            print("     → 怎么办：session-directory 认 --own-repository 与 --repository，projects-root 什么参数都不带", file=sys.stderr)
            return 2
    try:
        text = projects_root() if command == "projects-root" else session_directory(own_repository, repository)
    except SessionDirectoryError as error:
        print(f"  ✗ 取不到会话目录：{error}", file=sys.stderr)
        print(f"     → 怎么办：{error.howto}", file=sys.stderr)
        return 2
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"     → 怎么办：弄坏开关只给 --selftest 用，去掉 {BREAK_VARIABLE}", file=sys.stderr)
        return 2
    print(text)
    return 0


def main(arguments):
    refused = refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    if arguments == ["--selftest"]:
        return run_selftest()
    return run_command(arguments)


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
