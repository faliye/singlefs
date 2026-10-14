#!/usr/bin/env python3
# admission: always 直接执行只打此刻的登记值或跑自证，判的是此刻的登记文件与环境，上一次的结论不替这一次作保
# run-condition: none 被 import 的函数库；直接执行只读登记文件、在临时目录里造样本，除了 python3 与 bash 没有环境要求
"""每台默认重活线程数的 python 取法：被 import 的函数库（只用标准库，import 本身不读文件、不起子进程）。

登记位是仓根的 .claude/heavy-thread-default（写法在它的文件头）；shell 那一份是 scripts/hosts/heavy-threads.sh 的 heavy_thread_default，
两份同一个判法，两边的自证拿同几份登记互相对拍。

    sys.path.insert(0, <仓根>/scripts/hosts); import heavy_threads
    threads = heavy_threads.default_heavy_threads()          # 正整数；拒的时候抛 HeavyThreadDefaultError（str 是原因，.howto 是出路）
    python3 scripts/hosts/heavy_threads.py --print            打此刻的值，拒了退 2
    python3 scripts/hosts/heavy_threads.py --selftest         自证

default_heavy_threads(environment=os.environ, *, registration_file=None) -> int：
  environment 里 SINGLEFS_THREAD_CAP 设了（非空）：是正整数就交它（经 bash scripts/hosts/capped.sh <N> 起的命令里它就是 N），不是就拒；
  没设：读 registration_file（不给是这份文件往上两级那棵树的 .claude/heavy-thread-default，REGISTRATION_FILE），第一条非注释行
    （空行与 # 开头的行不算）要是「<正整数><空格或制表符>#<理由>」（理由不空），交那个数；文件不在、读不了、没有非注释行、
    那一行不是这个形状，都拒。拒了不退回按核数算。

弄坏开关 HEAVY_THREADS_BREAK=<项>（逗号分隔，只给自证证红；与 heavy-threads.sh 认同样的名字），打开后自证哪几格红
（失败行「✗ heavy_threads.py 自证：<格名>：…」）：
  cap-unchecked                         SINGLEFS_THREAD_CAP 原样交出、不判正整数：「SINGLEFS_THREAD_CAP=0 拒」「SINGLEFS_THREAD_CAP=abc 拒」
  missing-registration-uses-core-count  登记文件不在时照改前取本机核数：「登记文件不在拒」
"""
import os
import re
import sys

REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REGISTRATION_FILE = os.path.join(REPOSITORY, ".claude", "heavy-thread-default")
THREAD_CAP_VARIABLE = "SINGLEFS_THREAD_CAP"
BREAK_VARIABLE = "HEAVY_THREADS_BREAK"
POSITIVE_INTEGER = re.compile(r"[1-9][0-9]*")
COMMENT_OR_BLANK = re.compile(r"[ \t]*(#|$)")
REGISTRATION_LINE = re.compile(r"[ \t]*([1-9][0-9]*)[ \t]+#[ \t]*[^ \t]")
REGISTRATION_HOWTO = "第一条非注释行写成「31  # <理由>」：数是正整数，数与 # 之间至少一个空格，# 后面写为什么是这个数"


class HeavyThreadDefaultError(ValueError):
    """取不了每台默认重活线程数：str 是原因，howto 是下一步怎么办。"""

    def __init__(self, reason, howto):
        super().__init__(reason)
        self.howto = howto


def break_is_set(name, environment=None):
    return name in (environment if environment is not None else os.environ).get(BREAK_VARIABLE, "").split(",")


def default_heavy_threads(environment=os.environ, *, registration_file=None):
    """每台默认重活线程数（判法在文件头）。拒的时候抛 HeavyThreadDefaultError。"""
    cap = environment.get(THREAD_CAP_VARIABLE, "")
    if cap:
        if POSITIVE_INTEGER.fullmatch(cap) or break_is_set("cap-unchecked", environment):
            return int(cap) if cap.isdigit() else cap
        raise HeavyThreadDefaultError(f"{THREAD_CAP_VARIABLE}={cap} 不是正整数，取不了每台默认重活线程数",
                                      f"设成正整数（经 bash scripts/hosts/capped.sh <N> 起就是 N），或 unset 它取 .claude/heavy-thread-default 的登记值")
    path = registration_file or REGISTRATION_FILE
    if not os.path.isfile(path):
        if break_is_set("missing-registration-uses-core-count", environment):
            return os.cpu_count() or 1
        raise HeavyThreadDefaultError(f"每台默认重活线程数的登记文件 {path} 不在",
                                      "在仓根建 .claude/heavy-thread-default，第一条非注释行写「31  # <理由>」（写法在仓里那一份的文件头）；"
                                      "临时要别的数设 SINGLEFS_THREAD_CAP 或经 capped.sh <N> 起，不退回按核数算")
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
    except OSError as error:
        raise HeavyThreadDefaultError(f"每台默认重活线程数的登记文件 {path} 读不了：{error}", "看它的权限，改成本用户可读再跑；不退回按核数算") from error
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    for number, line in enumerate(lines, 1):
        if COMMENT_OR_BLANK.match(line):
            continue
        found = REGISTRATION_LINE.match(line)
        if found:
            return int(found.group(1))
        raise HeavyThreadDefaultError(f"{path} 第 {number} 行「{line}」不是「<正整数>  # <理由>」", REGISTRATION_HOWTO)
    raise HeavyThreadDefaultError(f"{path} 里没有非注释行，读不出每台默认重活线程数", REGISTRATION_HOWTO)


# ── 直接执行：--print 与 --selftest ─────────────────────────────────────────────

# 每格：格名、登记文件的内容（None 是不建这份文件）、SINGLEFS_THREAD_CAP（None 是不设）、期望（数，或 "refused"）。与 heavy-threads.sh 的自证同一张
SELFTEST_CELLS = [
    ("登记 7、带理由", "7  # 样本理由\n", None, 7),
    ("登记前有注释行与空行、只认第一条非注释行", "# 注释\n\n7  # 样本理由\n8  # 第二条不认\n", None, 7),
    ("登记文件不在拒", None, None, "refused"),
    ("写坏：不是数拒", "abc  # 样本理由\n", None, "refused"),
    ("写坏：0 拒", "0  # 样本理由\n", None, "refused"),
    ("写坏：没写理由拒", "7\n", None, "refused"),
    ("写坏：数与 # 之间没有空白拒", "7# 样本理由\n", None, "refused"),
    ("写坏：只有注释拒", "# 只有注释\n", None, "refused"),
    ("SINGLEFS_THREAD_CAP=5 盖过登记的 7", "7  # 样本理由\n", "5", 5),
    ("SINGLEFS_THREAD_CAP=5、登记文件不在也照它", None, "5", 5),
    ("SINGLEFS_THREAD_CAP=0 拒", "7  # 样本理由\n", "0", "refused"),
    ("SINGLEFS_THREAD_CAP=abc 拒", "7  # 样本理由\n", "abc", "refused"),
    ("SINGLEFS_THREAD_CAP 设成空串当没设", "7  # 样本理由\n", "", 7),
]


def outcome_of(call):
    try:
        return call()
    except HeavyThreadDefaultError:
        return "refused"


def run_selftest():
    import shutil
    import subprocess
    import tempfile

    here = os.path.dirname(os.path.abspath(__file__))
    failures, checked = [], 0

    def expect(label, holds, detail):
        if not holds:
            failures.append(f"{label}：{detail}")

    scratch = tempfile.mkdtemp(prefix="heavy-threads-selftest-")
    try:
        base_environment = {name: value for name, value in os.environ.items() if name != THREAD_CAP_VARIABLE}
        for index, (label, registration, cap, want) in enumerate(SELFTEST_CELLS, 1):
            tree = os.path.join(scratch, f"tree-{index}")
            os.makedirs(os.path.join(tree, "scripts", "hosts"))
            os.makedirs(os.path.join(tree, ".claude"))
            for name in ("heavy_threads.py", "heavy-threads.sh"):
                shutil.copy(os.path.join(here, name), os.path.join(tree, "scripts", "hosts", name))
            registration_file = os.path.join(tree, ".claude", "heavy-thread-default")
            if registration is not None:
                with open(registration_file, "w", encoding="utf-8") as handle:
                    handle.write(registration)
            environment = dict(base_environment)
            if cap is not None:
                environment[THREAD_CAP_VARIABLE] = cap
            # 直接给 registration_file 调
            checked += 1
            got = outcome_of(lambda: default_heavy_threads(environment, registration_file=registration_file))
            expect(label, got == want, f"应当{'拒' if want == 'refused' else f'交 {want}'}，实际交「{got}」")
            # 样本树里那一份按自己的位置找登记（不给 registration_file）
            checked += 1
            completed = subprocess.run([sys.executable, "-B", "-c", "import sys; sys.path.insert(0, sys.argv[1]); import heavy_threads\n"
                                        "try:\n    print(heavy_threads.default_heavy_threads())\n"
                                        "except heavy_threads.HeavyThreadDefaultError as error:\n    print('refused'); print(error, file=sys.stderr)",
                                        os.path.join(tree, "scripts", "hosts")], env=environment, capture_output=True, text=True, check=False)
            located = completed.stdout.strip()
            expect(f"{label}（样本树里那一份按自己的位置找登记）", located == str(want), f"应当交「{want}」，实际「{located}」，stderr「{completed.stderr.strip()[:200]}」")
            # 与 heavy-threads.sh 对拍
            checked += 1
            completed = subprocess.run(["bash", "-c", 'source "$1"; heavy_thread_default', "heavy-threads-case",
                                        os.path.join(tree, "scripts", "hosts", "heavy-threads.sh")], env=environment, capture_output=True, text=True, check=False)
            shell_got = completed.stdout.strip() if completed.returncode == 0 else ("refused" if completed.returncode == 2 and not completed.stdout else
                                                                                   f"退 {completed.returncode}、stdout「{completed.stdout.strip()}」")
            expect(f"{label}（与 heavy-threads.sh 对拍）", shell_got == str(got), f"python 交「{got}」，shell 交「{shell_got}」")
        # 拒的时候带出路
        checked += 1
        try:
            default_heavy_threads(dict(base_environment, **{THREAD_CAP_VARIABLE: "0"}))
            expect("拒的时候带出路", False, "SINGLEFS_THREAD_CAP=0 没拒")
        except HeavyThreadDefaultError as error:
            expect("拒的时候带出路", bool(str(error)) and bool(error.howto), f"原因「{error}」，出路「{error.howto}」")
        # 本仓那一份：登记读得出，--print 同值
        checked += 1
        real = outcome_of(lambda: default_heavy_threads(base_environment))
        expect("本仓的登记读得出", isinstance(real, int) and real >= 1, f"交「{real}」（{REGISTRATION_FILE}）")
        checked += 1
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "--print"], env=base_environment, capture_output=True, text=True, check=False)
        expect("本仓的登记：--print 与函数同值", completed.returncode == 0 and completed.stdout.strip() == str(real),
               f"退 {completed.returncode}、打「{completed.stdout.strip()}」，函数交「{real}」")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    if failures:
        for failure in failures:
            print(f"  ✗ heavy_threads.py 自证：{failure}")  # gate-lint:detail
        print(f"  ✗ heavy_threads.py 自证 {len(failures)} 处不对（查了 {checked} 项）")
        print("     → 怎么办：照文件头的判法改 default_heavy_threads（与 scripts/hosts/heavy-threads.sh 的 heavy_thread_default 同一个判法）；HEAVY_THREADS_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ heavy_threads.py 自证通过（查了 {checked} 项：{len(SELFTEST_CELLS)} 份样本登记各直接调、按样本树的位置找、与 heavy-threads.sh 对拍各一次，"
          "拒的时候带出路，本仓那一份读得出、--print 同值）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return run_selftest()
    if arguments == ["--print"]:
        try:
            print(default_heavy_threads())
        except HeavyThreadDefaultError as error:
            print(f"  ✗ {error}", file=sys.stderr)
            print(f"     → 怎么办：{error.howto}", file=sys.stderr)
            return 2
        return 0
    print(f"  ✗ 认不出的用法：{' '.join(arguments)}", file=sys.stderr)
    print("     → 怎么办：import 它再调 default_heavy_threads()；直接执行只认 --print（打此刻的值）与 --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
