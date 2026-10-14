#!/usr/bin/env python3
# admission: always 判的是此刻谁拿着主工作区合入锁，上一次的结论不替这一次作保
# run-condition: command git
"""主工作区合入锁：往主工作区打补丁、在主工作区合入后验证时持这一把锁，别的合入排队等它放。

为什么：两条线往同一个主工作区合入、在上面跑重活，互相撞，编译错、两趟都白跑
（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 20 行；
.claude/singlefs-ai-sop/rules/session-wrapup.md「4. 同一个仓里有没有别的会话在飞？」「这棵工作树上只有我在编」那一条）。

调它的：scripts/gate/apply-writer-patch.py（打进主工作区时，按路径载入本库调 acquire / release）、
scripts/gate/merge-and-verify.sh（不带 --copy 时，命令行 acquire / release，持有者是它自己的进程号）。

锁文件：SINGLEFS_WORKSPACE_LOCK_FILE 设了用它（自证用、或要另指一处时），没设是 <树的 git common-dir>/singlefs-main-workspace.lock。
  文件不删，里面记持有者一行一项：pid=、start=（/proc/<pid>/stat 第 22 段，防进程号被复用）、session=（CLAUDE_CODE_SESSION_ID）、
  command=、since=（拿到时的 epoch 秒）；空着就是没人拿。读写记录都在这份文件的 flock 里做，flock 只拿一瞬、不跨命令持有。
判法：
  ① 记录空着：写上自己，拿到。
  ② 记录里的持有者还活着（进程在、start 对得上）：排队。每过 SINGLEFS_WORKSPACE_LOCK_POLL_SECONDS（默认 2）秒再看一次，
     头一次与之后每过 SINGLEFS_WORKSPACE_LOCK_REPORT_SECONDS（默认 60）秒打一行「在排队：被会话 …、进程 …（命令 …）占着……」。
     give_up_seconds（命令行 --give-up-after；没给时取 SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS，都没有就一直等）给了而等到点还被占着：不拿，退 3。
  ③ 记录里的持有者已死（进程不在、或进程号被别的进程复用）：打一行「上一个持有者 … 已死，锁判失效，接管」，写上自己，拿到。
  ④ 重入：SINGLEFS_WORKSPACE_LOCK_HOLDER=<进程号> 设着、记录里的持有者正是那个进程而且活着、它是这个进程自己或祖先：
     已经在持有者之下（merge-and-verify.sh 持锁时起的 apply-writer-patch.py），不另拿，放锁时也不动记录。
     这个变量会漏给持有者起的所有子进程，靠「是祖先」这一条判，漏到别处的不算数。
放锁：记录里还是自己（pid 与 start 都对得上）才清空；不是自己（被人判死接管了）不动，交回那一句。

命令行：
  lib_workspace_lock.py acquire --root <树> --holder-pid <进程号> --command <文字> [--give-up-after <秒>]
      退 0 拿到（或重入）；1 建不了锁文件、树不在 git 仓里、持有者进程号不对；2 用法错；3 等到点还被占着
  lib_workspace_lock.py release --root <树> --holder-pid <进程号>      退 0 放了（或本来就不是自己的、写明）；1 读写不了
  lib_workspace_lock.py status --root <树>                             打出谁拿着；退 0 没人拿、3 有人拿着
  lib_workspace_lock.py --selftest

弄坏开关 WORKSPACE_LOCK_BREAK=<项>（只给证红用）。项名、弄坏哪一处、打开后自证红在哪几格
（自证里每一格各用一份新锁文件、每一次等锁都有放弃时限（SELFTEST_GIVE_UP_SECONDS），哪个开关打开都在 60 秒内退 1、不挂住）：
  trust-dead-holder  不看持有者死没死，记录里有人就排队
      本库「持有者已死的锁文件：报出来、接管」（等到 give-up 点退 3）与「放锁之后记录清空」（死的那份记录留着）；
      连带 merge-and-verify.sh「锁文件里的持有者已死……」与 apply-writer-patch.py「主工作区合入锁的持有者已死……」那一格
  no-reentry         不认 SINGLEFS_WORKSPACE_LOCK_HOLDER，持有者之下的子进程也排队
      本库「持有者之下的子进程：重入、不另拿」；连带 apply-writer-patch.py「merge-and-verify 持锁时起的 apply 应当不排队……」那一格
  no-wait            记录里的持有者活着也照拿（不排队）
      本库「两个进程同时要锁：后到的排队、报出前者、前者放了才拿到」「HOLDER 变量指着的持有者不是祖先：照排队……」
      「持有者已死的锁文件：报出来、接管」（不看死活就不报接管）；
      连带 merge-and-verify.sh「两个合入同时起……」「锁文件里的持有者已死……」与 apply-writer-patch.py「主工作区合入锁被别的合入占着时……」「主工作区合入锁的持有者已死时……」
"""
import fcntl
import os
import subprocess
import sys
import tempfile
import time
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "WORKSPACE_LOCK_BREAK"
LOCK_NAME = "singlefs-main-workspace.lock"
LOCK_FILE_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_FILE"
HOLDER_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_HOLDER"
REPORT_SECONDS_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_REPORT_SECONDS"
POLL_SECONDS_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_POLL_SECONDS"
GIVE_UP_SECONDS_VARIABLE = "SINGLEFS_WORKSPACE_LOCK_GIVE_UP_SECONDS"
EXIT_FAILED = 1
EXIT_USAGE = 2
EXIT_GAVE_UP = 3


class LockError(Exception):
    """拿不成或放不成：(卡在哪, 下一步)。"""

    def __init__(self, problem, howto, code=EXIT_FAILED):
        super().__init__(problem)
        self.problem, self.howto, self.code = problem, howto, code


def broken(item):
    return os.environ.get(BREAK_VARIABLE, "") == item


def lock_path(root):
    override = os.environ.get(LOCK_FILE_VARIABLE, "")
    if override:
        return override
    completed = subprocess.run(["git", "-C", root, "rev-parse", "--git-common-dir"], capture_output=True, text=True)
    if completed.returncode != 0 or not completed.stdout.strip():
        raise LockError(f"{root} 不在 git 仓里，找不到放主工作区合入锁的 git common-dir（{completed.stderr.strip()[:200]}）",
                        f"在仓里跑；要另指一处锁文件设 {LOCK_FILE_VARIABLE}")
    common = completed.stdout.strip()
    return os.path.join(common if os.path.isabs(common) else os.path.join(root, common), LOCK_NAME)


def process_start(pid):
    """/proc/<pid>/stat 第 22 段（开机以来的时钟滴答）；进程不在交 None。"""
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    return text.rsplit(")", 1)[1].split()[19]


def parent_of(pid):
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace") as handle:
            return int(handle.read().rsplit(")", 1)[1].split()[1])
    except (OSError, ValueError, IndexError):
        return None


def is_self_or_ancestor(pid):
    current = os.getpid()
    while current and current > 1:
        if current == pid:
            return True
        current = parent_of(current)
    return pid == current


def parse_record(text):
    record = {}
    for line in text.splitlines():
        key, separator, value = line.partition("=")
        if separator:
            record[key.strip()] = value.strip()
    return record if record.get("pid") else None


def holder_alive(record):
    try:
        pid = int(record["pid"])
    except (KeyError, ValueError):
        return False
    return process_start(pid) is not None and process_start(pid) == record.get("start")


def describe(record, now=None):
    now = time.time() if now is None else now
    try:
        since = float(record.get("since", ""))
        since_text = f"{time.strftime('%Y-%m-%d', time.localtime(since))} 起、已拿了 {int(now - since)} 秒"
    except ValueError:
        since_text = "起于不明"
    return (f"会话 {record.get('session') or '不明'}、进程 {record.get('pid')}（{'活着' if holder_alive(record) else '已死'}），"
            f"命令「{record.get('command') or '不明'}」，{since_text}")


class Held:
    def __init__(self, path, pid, start, reentrant):
        self.path, self.pid, self.start, self.reentrant = path, pid, start, reentrant


def open_locked(path):
    """打开锁文件、拿 flock（只拿一瞬，读写记录用）。"""
    try:
        descriptor = os.open(path, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC, 0o644)
    except OSError as error:
        raise LockError(f"建不了主工作区合入锁文件 {path}：{error}", "看 git common-dir 在不在、可不可写；锁拿不到就别往主工作区合入")
    fcntl.flock(descriptor, fcntl.LOCK_EX)
    return descriptor


def read_record(descriptor):
    os.lseek(descriptor, 0, os.SEEK_SET)
    chunks = []
    while True:
        chunk = os.read(descriptor, 65536)
        if not chunk:
            break
        chunks.append(chunk)
    return parse_record(b"".join(chunks).decode("utf-8", errors="replace"))


def write_record(descriptor, text):
    os.ftruncate(descriptor, 0)
    os.lseek(descriptor, 0, os.SEEK_SET)
    if text:
        os.write(descriptor, text.encode("utf-8"))
    os.fsync(descriptor)


def acquire(root, holder_pid, command, say=print, give_up_seconds=None, clock=time.monotonic, sleep=time.sleep):
    """拿主工作区合入锁（文件头「判法」①–④）：→ Held；拿不成抛 LockError。"""
    path = lock_path(root)
    start = process_start(holder_pid)
    if start is None:
        raise LockError(f"持有者进程 {holder_pid} 不在", "--holder-pid 给持锁那一方自己的进程号（merge-and-verify.sh 给 $$）")
    if give_up_seconds is None and os.environ.get(GIVE_UP_SECONDS_VARIABLE, ""):
        give_up_seconds = float(os.environ[GIVE_UP_SECONDS_VARIABLE])
    report_seconds = float(os.environ.get(REPORT_SECONDS_VARIABLE, "") or 60)
    poll_seconds = float(os.environ.get(POLL_SECONDS_VARIABLE, "") or 2)
    began = clock()
    next_report = began
    while True:
        descriptor = open_locked(path)
        try:
            record = read_record(descriptor)
            if record and not broken("no-reentry") and os.environ.get(HOLDER_VARIABLE, "") == record.get("pid") \
                    and holder_alive(record) and is_self_or_ancestor(int(record["pid"])):
                say(f"  主工作区合入锁：已在持有者 {record['pid']} 之下（{HOLDER_VARIABLE}），不另拿")
                return Held(path, int(record["pid"]), record.get("start"), True)
            if record and record.get("pid") == str(holder_pid) and record.get("start") == start:
                return Held(path, holder_pid, start, True)
            free = record is None or broken("no-wait")
            if record is not None and not free and not holder_alive(record) and not broken("trust-dead-holder"):
                say(f"  ! 主工作区合入锁 {path}：上一个持有者已死（{describe(record)}），锁判失效，接管")
                free = True
            if free:
                write_record(descriptor, f"pid={holder_pid}\nstart={start}\nsession={os.environ.get('CLAUDE_CODE_SESSION_ID', '')}\n"
                                         f"command={' '.join(command.split())}\nsince={time.time()}\n")
                waited = clock() - began
                say(f"  拿到主工作区合入锁 {path}（持有者进程 {holder_pid}{f'，排队 {int(waited)} 秒' if waited >= 1 else ''}）")
                return Held(path, holder_pid, start, False)
        finally:
            os.close(descriptor)
        now = clock()
        if give_up_seconds is not None and now - began >= give_up_seconds:
            raise LockError(f"等了 {int(now - began)} 秒，主工作区合入锁还被占着：{describe(record)}",
                            "等它合完再起；它该停就按进程号停它（停了之后锁自动判失效），别删锁文件", EXIT_GAVE_UP)
        if now >= next_report:
            say(f"  … 在排队：主工作区合入锁 {path} 被{describe(record)}占着，本次已等 {int(now - began)} 秒；每 {report_seconds:g} 秒报一次")
            next_report = now + report_seconds
        sleep(poll_seconds)


def release(held):
    """放锁：记录还是自己才清空。→ None，或一句没放成的原因（不抛）。"""
    if held is None or held.reentrant:
        return None
    descriptor = open_locked(held.path)
    try:
        record = read_record(descriptor)
        if record and record.get("pid") == str(held.pid) and record.get("start") == held.start:
            write_record(descriptor, "")
            return None
        return f"锁文件 {held.path} 里的持有者已不是进程 {held.pid}（{describe(record) if record else '空着'}），没动它"
    finally:
        os.close(descriptor)


def status(root):
    path = lock_path(root)
    descriptor = open_locked(path)
    try:
        return path, read_record(descriptor)
    finally:
        os.close(descriptor)


# ── 自证 ──
CHILD_HOLDER = r'''
import importlib.util, os, sys, time
specification = importlib.util.spec_from_file_location("lib_workspace_lock_child", sys.argv[1])
lock = importlib.util.module_from_spec(specification); specification.loader.exec_module(lock)
tag, timeline, hold_seconds, give_up_seconds = sys.argv[2], sys.argv[3], float(sys.argv[4]), float(sys.argv[6])
def say(line):
    print(line, flush=True)
held = lock.acquire(sys.argv[5], os.getpid(), "假合入 " + tag, say=say, give_up_seconds=give_up_seconds)
with open(timeline, "a") as handle:
    handle.write(f"in {tag} {time.time_ns()}\n")
time.sleep(hold_seconds)
with open(timeline, "a") as handle:
    handle.write(f"out {tag} {time.time_ns()}\n")
lock.release(held)
'''
# 自证里每一次等锁的放弃时限（秒）：锁永远拿不到（弄坏开关、或哪一格没放锁）时到点退 3、那一格判红，整趟不挂住。
# 两个子进程那一格：后到的最多等 SELFTEST_GIVE_UP_SECONDS，自证等每个子进程退出最多 SELFTEST_CHILD_WAIT_SECONDS，到点停掉自己起的那个进程、判红。
# 各格加起来不过 60 秒：两个子进程那一格最多 10 + 2 × 15 秒；其余几格各用一份新锁文件，等锁的放弃时限是 2、1、0.5 秒，
# 自己先拿锁那两处（重入、持有者进程号不在）锁文件是空的、不等。
SELFTEST_GIVE_UP_SECONDS = 8
SELFTEST_CHILD_WAIT_SECONDS = 15


def selftest_wait(process, seconds=SELFTEST_CHILD_WAIT_SECONDS):
    """等自证自己起的子进程退出：→ 退出码；到点还没退就停掉它（只停这一个进程号），交 "超时"。"""
    try:
        return process.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()
        return "超时"


def selftest_acquire(root, holder_pid, command, said, give_up_seconds):
    """自证里要锁：→ (Held 或 None, 没拿成的原因或 "")。拿不成（到点放弃、别的 LockError）不抛，交给那一格判红。"""
    try:
        return acquire(root, holder_pid, command, say=said.append, give_up_seconds=give_up_seconds), ""
    except LockError as error:
        return None, f"LockError 退 {error.code}：{error.problem}"


def selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="lib-workspace-lock-selftest-")
    saved = {name: os.environ.pop(name, None) for name in (LOCK_FILE_VARIABLE, HOLDER_VARIABLE, REPORT_SECONDS_VARIABLE, POLL_SECONDS_VARIABLE, GIVE_UP_SECONDS_VARIABLE)}
    try:
        os.environ[REPORT_SECONDS_VARIABLE] = "0.3"
        os.environ[POLL_SECONDS_VARIABLE] = "0.05"

        def fresh_lock(name):
            """每一格各用一份新的锁文件：上一格留下的记录（弄坏开关下没接管、没放的）不带进下一格，红只落在点名的那几格。"""
            os.environ[LOCK_FILE_VARIABLE] = os.path.join(work, f"{name}.lock")
            return os.environ[LOCK_FILE_VARIABLE]

        def record_in(path):
            return parse_record(open(path).read()) if os.path.exists(path) else None

        def cell(name, condition, detail):
            nonlocal checked
            checked += 1
            if not condition:
                failures.append(f"{name}：{detail}")

        # 两个假的合入同时起：A 先拿着 1.5 秒，B 后到；B 要排队、报出 A 的进程号、A 放了才进
        fresh_lock("two-processes")
        timeline = os.path.join(work, "timeline.txt")
        child = os.path.join(work, "child.py")
        with open(child, "w") as handle:
            handle.write(CHILD_HOLDER)
        first_output, second_output = os.path.join(work, "a.txt"), os.path.join(work, "b.txt")
        with open(first_output, "w") as first_log:
            first = subprocess.Popen([sys.executable, child, os.path.realpath(__file__), "A", timeline, "1.5", work, str(SELFTEST_GIVE_UP_SECONDS)],
                                     stdout=first_log, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and first.poll() is None and not (os.path.exists(timeline) and "in A" in open(timeline).read()):
            time.sleep(0.05)
        with open(second_output, "w") as second_log:
            second = subprocess.Popen([sys.executable, child, os.path.realpath(__file__), "B", timeline, "0", work, str(SELFTEST_GIVE_UP_SECONDS)],
                                      stdout=second_log, stderr=subprocess.STDOUT)
        first_code, second_code = selftest_wait(first), selftest_wait(second)
        events = {(word, tag): int(stamp) for word, tag, stamp in (line.split() for line in open(timeline))} if os.path.exists(timeline) else {}
        second_text = open(second_output).read()
        cell("两个进程同时要锁：后到的排队、报出前者、前者放了才拿到",
             first_code == 0 and second_code == 0 and ("out", "A") in events and ("in", "B") in events and events[("in", "B")] > events[("out", "A")]
             and f"进程 {first.pid}" in second_text and "在排队" in second_text,
             f"A 退 {first_code}、B 退 {second_code}，时间线 {sorted(events.items(), key=lambda item: item[1])}，B 的输出 {second_text.strip()[-300:]!r}")
        # 持有者已死的锁文件：记一个已经退出的进程，再要锁要报「已死」并接管；放锁之后记录清空（这两格用同一份锁文件）
        dead_lock = fresh_lock("dead-holder")
        dead = subprocess.Popen(["true"])
        dead.wait()
        with open(dead_lock, "w") as handle:
            handle.write(f"pid={dead.pid}\nstart=1\nsession=死掉的会话\ncommand=假合入 dead\nsince={time.time() - 30}\n")
        said = []
        held, problem = selftest_acquire(work, os.getpid(), "接管者", said, 2)
        taken = held is not None and not held.reentrant and (record_in(dead_lock) or {}).get("pid") == str(os.getpid())
        if held is not None:
            release(held)
        cell("持有者已死的锁文件：报出来、接管", taken and any("已死" in line and "接管" in line and "死掉的会话" in line for line in said), f"{problem or '拿到了'}：{said}")
        cell("放锁之后记录清空", record_in(dead_lock) is None, open(dead_lock).read())
        # 重入：自己拿着锁、设了 HOLDER 变量，子进程再要锁不排队
        fresh_lock("reentry")
        said = []
        held, problem = selftest_acquire(work, os.getpid(), "持有者", said, SELFTEST_GIVE_UP_SECONDS)
        if held is None:
            nested_code, nested_text = "没跑", f"自己先要锁就没拿成（{problem}）"
        else:
            try:
                # 子进程以自己的进程号要锁（bash 的 $$ 经 exec 就是 python 的进程号）
                nested = subprocess.run(["bash", "-c", 'exec "$0" "$1" acquire --root "$2" --holder-pid $$ --command 子进程 --give-up-after 1',
                                         sys.executable, os.path.realpath(__file__), work], capture_output=True, text=True, timeout=SELFTEST_CHILD_WAIT_SECONDS,
                                        env=dict(os.environ, **{HOLDER_VARIABLE: str(os.getpid())}))
                nested_code, nested_text = nested.returncode, nested.stdout + nested.stderr
            except subprocess.TimeoutExpired as error:
                nested_code, nested_text = "超时", f"{error.stdout or ''}{error.stderr or ''}"
            finally:
                release(held)
        cell("持有者之下的子进程：重入、不另拿", nested_code == 0 and "已在持有者" in nested_text, f"退 {nested_code}：{nested_text.strip()!r}")
        # 变量漏到不相干的进程：记录里的持有者活着、HOLDER 变量也指着它，但它不是要锁那一方的祖先（自证起的一个睡着的子进程）：照排队、到点退 3
        foreign_lock = fresh_lock("foreign-holder")
        sleeper = subprocess.Popen(["sleep", "30"])
        try:
            with open(foreign_lock, "w") as handle:
                handle.write(f"pid={sleeper.pid}\nstart={process_start(sleeper.pid)}\nsession=别的会话\ncommand=假合入 sleeper\nsince={time.time()}\n")
            os.environ[HOLDER_VARIABLE] = str(sleeper.pid)
            said = []
            try:
                held, problem = selftest_acquire(work, os.getpid(), "不相干", said, 0.5)
                outcome = "拿到了" if held is not None else problem
            finally:
                os.environ.pop(HOLDER_VARIABLE, None)
        finally:
            sleeper.kill()
            sleeper.wait()
        cell("HOLDER 变量指着的持有者不是祖先：照排队、到点退 3、报出持有者",
             outcome.startswith(f"LockError 退 {EXIT_GAVE_UP}") and any(f"进程 {sleeper.pid}" in line and "在排队" in line for line in said), f"{outcome}：{said}")
        missing_lock = fresh_lock("missing-holder")
        try:
            missing_holder = subprocess.run([sys.executable, os.path.realpath(__file__), "acquire", "--root", work, "--holder-pid", str(dead.pid), "--command", "x",
                                             "--give-up-after", str(SELFTEST_GIVE_UP_SECONDS)], capture_output=True, text=True, timeout=SELFTEST_CHILD_WAIT_SECONDS)
            missing_code, missing_text = missing_holder.returncode, missing_holder.stdout
        except subprocess.TimeoutExpired as error:
            missing_code, missing_text = "超时", f"{error.stdout or ''}"
        cell("持有者进程号不在：退 1、不拿", missing_code == EXIT_FAILED and "不在" in missing_text and record_in(missing_lock) is None,
             f"退 {missing_code}：{missing_text.strip()!r}")
    finally:
        for name, value in saved.items():
            os.environ.pop(name, None)
            if value is not None:
                os.environ[name] = value
        import shutil
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 acquire / release 的判法（文件头「判法」①–④）；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return EXIT_FAILED
    print(f"  ✓ lib_workspace_lock 自检通过（查了 {checked} 格）：两个假的合入同时起后到的排队、报出前者、前者放了才进，持有者已死的锁文件报出来并接管，"
          f"放锁后记录清空，持有者之下的子进程重入不另拿，HOLDER 变量指着的不是祖先时照排队到点退 3，持有者进程号不在退 1")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    usage = ("✗ 用法不对\n→ 怎么办：lib_workspace_lock.py acquire --root <树> --holder-pid <进程号> --command <文字> [--give-up-after <秒>]；"
             "release --root <树> --holder-pid <进程号>；status --root <树>")
    if not arguments or arguments[0] not in ("acquire", "release", "status"):
        print(usage)
        return EXIT_USAGE
    options, rest = {}, arguments[1:]
    while rest:
        if len(rest) < 2 or rest[0] not in ("--root", "--holder-pid", "--command", "--give-up-after"):
            print(usage)
            return EXIT_USAGE
        options[rest[0]], rest = rest[1], rest[2:]
    required = {"acquire": ("--root", "--holder-pid", "--command"), "release": ("--root", "--holder-pid"), "status": ("--root",)}[arguments[0]]
    if any(name not in options for name in required):
        print(usage)
        return EXIT_USAGE
    try:
        if arguments[0] == "status":
            path, record = status(options["--root"])
            print(f"主工作区合入锁 {path}：{describe(record) if record else '没人拿'}")
            return EXIT_GAVE_UP if record else 0
        holder = int(options["--holder-pid"])
        if arguments[0] == "acquire":
            give_up = float(options["--give-up-after"]) if "--give-up-after" in options else None
            acquire(options["--root"], holder, options["--command"], say=lambda line: print(line, flush=True), give_up_seconds=give_up)
            return 0
        path = lock_path(options["--root"])
        problem = release(Held(path, holder, process_start(holder), False))
        if problem:
            print(f"  ! {problem}")
        return 0
    except ValueError:
        print(usage)
        return EXIT_USAGE
    except LockError as error:
        print(f"  ✗ {error.problem}\n  → 怎么办：{error.howto}", flush=True)
        return error.code


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
