#!/usr/bin/env bash
# hook-events: SessionStart
# gate-similar: 无 查过 .claude/settings.json 里 SessionStart 上的注册（只有本文件：上下文压缩之后的对齐提示原是单独一份 after-compact.sh，挂同一个事件、读同一份 hook 输入、写同一个标准输出，已并进来）与 gate-overlap.py --list 列出的另外 9 个钩子（挂 PreToolUse、PostToolUse、Stop、SubagentStop，判的都是一条工具调用或收工）
# SessionStart hook：会话开始时往上下文里补两样东西，按 hook 输入的 source 分，只报、不改任何东西。
#   compact           上下文压缩之后，接续工作要先对齐的几件：还有没有子 agent 在跑、工作区里哪些没提交、这件活的状态记在哪份记录里。
#   startup / resume  近 SESSION_START_OOM_HOURS 小时（默认 12）内核因为内存不够杀过的进程：journalctl -k 里的 Killed process 行，
#                     每条写时间（UTC 与 JST）、进程号、进程名、anon-rss，整机 OOM 与内存上限（cgroup）里的分开列，外加一行「怎么办」。
#                     读不到内核日志（没有 journalctl、权限不够看不到内核日志、journalctl 出错或超时）报「没查成」，不当成没有 OOM。
#
# 为什么：2026-09-25 两次整机 OOM 把 Claude Code 会话拖垮，第一次崩了之后主 agent 没查 `journalctl -k` 就照原样重派同一步，
# 第二次 OOM 就是这样来的（records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。会话崩了再起，是 startup 或 resume。
# 压缩那一半：长会话里任务转向时要尽快压缩上下文（用户 2026-09-17），压缩摘要会丢细节，接续最容易错的就是上面那三件。
# SessionStart 的标准输出会进新上下文，所以在这里补。matcher 写 startup|resume|compact，三种注册在同一条上。
#
#   session-start.sh             # 从 stdin 读 hook 的 JSON，往标准输出写提示，退出码恒 0
#   session-start.sh --selftest  # 喂假 JSON 与假 journalctl 走一遍；弄坏开关见 selftest() 末尾的出路，设着时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone
from zoneinfo import ZoneInfo

BROKEN_JUDGEMENT = os.environ.get("SESSION_START_BREAK", "")
DEFAULT_OOM_HOURS = 12        # 半天：会话夜里崩、第二天早上再开也罩得住，又不至于把前一天早已查清的 OOM 每次都翻出来
JOURNAL_TIMEOUT_SECONDS = 20  # journalctl 读大日志可能慢；超时就报没查成，不拖住会话启动
KILLS_LISTED_ONE_BY_ONE = 12  # 杀得多时逐条列 anon-rss 最大的这么多条，其余按进程名合计
KILLED_PROCESS_LINE = re.compile(
    r"^(?P<timestamp>\S+) \S+ kernel: (?P<kind>Memory cgroup out of memory|Out of memory)(?: \([^)]*\))?: "
    r"Killed process (?P<pid>\d+) \((?P<name>.*)\) total-vm:\d+kB, anon-rss:(?P<anon_rss_kibibytes>\d+)kB")
# journalctl 看不到别人的日志时退 0、只在 stderr 给这句提示：不认它，读不到就会被当成「没有 OOM」
JOURNAL_PERMISSION_HINT = "not seeing messages from other users and the system"
TOKYO = ZoneInfo("Asia/Tokyo")


def compact_reminder(hook_input, project_root):
    if os.environ.get("AFTER_COMPACT_DISABLE") == "1":
        return ""
    def git(*arguments):
        completed = subprocess.run(["git", "-C", project_root, *arguments], capture_output=True, text=True)
        return completed.stdout.strip() if completed.returncode == 0 else ""
    branch = git("branch", "--show-current") or "（读不出）"
    status_lines = [line for line in git("status", "--porcelain").split("\n") if line.strip()]
    records_dir = os.path.join(project_root, "records")
    records = []
    if os.path.isdir(records_dir):
        records = sorted((name for name in os.listdir(records_dir) if name.endswith(".md")),
                         key=lambda name: os.path.getmtime(os.path.join(records_dir, name)), reverse=True)[:3]
    transcript = hook_input.get("transcript_path") or ""
    session_id = hook_input.get("session_id") or ""
    session_dir = os.path.join(os.path.dirname(transcript), session_id) if transcript and session_id else "<会话目录>"
    lines = [
        "【上下文刚压缩过，接着干活之前先对齐这几件】",
        f"- 分支 {branch}；工作区未提交 {len(status_lines)} 个路径（`git status --short` 看清哪些是这一轮的，别的会话的不碰）。",
        "- 最近动过的记录：" + ("、".join(f"records/{name}" for name in records) if records else "（没有）") + "；接续的状态以记录里的状态表与岔路单为准，不凭压缩摘要里的印象。",
        f"- 派出过子 agent 的，先看还有没有在跑：`python3 research/scripts/agent-watch.py report --session-dir {session_dir}`；在跑的照 CLAUDE.md「派出去之后要盯住」重起看门狗。",
    ]
    return "\n".join(lines)


def oom_hours():
    try:
        hours = float(os.environ.get("SESSION_START_OOM_HOURS", DEFAULT_OOM_HOURS))
    except ValueError:
        hours = DEFAULT_OOM_HOURS
    return hours if hours > 0 else DEFAULT_OOM_HOURS


def read_kernel_journal(hours):
    """(文本, None) 或 (None, 没查成的原因)。先确认看得到内核日志（最近一条内核消息读得出来），再读近 hours 小时的。"""
    if shutil.which("journalctl") is None:
        return None, "这台机器上没有 journalctl"
    try:
        probe = subprocess.run(["journalctl", "-k", "-n", "1", "--no-pager", "-q"], capture_output=True, text=True, timeout=JOURNAL_TIMEOUT_SECONDS)
        recent = subprocess.run(["journalctl", "-k", "--since", f"-{hours:g}h", "--utc", "-o", "short-iso-precise", "--no-pager", "-q"],
                                capture_output=True, text=True, timeout=JOURNAL_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return None, f"journalctl 过了 {JOURNAL_TIMEOUT_SECONDS} 秒没返回"
    except OSError as error:
        return None, f"journalctl 起不来（{error}）"
    for completed in (probe, recent):
        if completed.returncode != 0:
            return None, f"journalctl 退出码 {completed.returncode}：{completed.stderr.strip()[:200]}"
        if JOURNAL_PERMISSION_HINT in completed.stderr and BROKEN_JUDGEMENT != "unreadable-as-clean":
            return None, "权限不够，journalctl 看不到内核日志（要在 adm 或 systemd-journal 组里）"
    if not probe.stdout.strip() and BROKEN_JUDGEMENT != "unreadable-as-clean":
        return None, "journalctl -k 连最近一条内核消息都读不出来，看不到内核日志"
    return recent.stdout, None


def killed_processes(journal_text):
    kills = []
    for line in journal_text.splitlines():
        matched = KILLED_PROCESS_LINE.match(line)
        if not matched:
            continue
        try:
            moment = datetime.fromisoformat(matched.group("timestamp")).astimezone(timezone.utc)
        except ValueError:
            continue
        kills.append({"moment": moment, "pid": matched.group("pid"), "name": matched.group("name"),
                      "anon_rss_kibibytes": int(matched.group("anon_rss_kibibytes")),
                      "whole_machine": matched.group("kind") == "Out of memory"})
    return kills


def format_size(kibibytes):
    if kibibytes >= 1024 * 1024:
        return f"{kibibytes / (1024 * 1024):.1f} GiB"
    return f"{kibibytes / 1024:.0f} MiB"


def kill_lines(kills):
    """逐条列 anon-rss 最大的 KILLS_LISTED_ONE_BY_ONE 条（按时间排），其余按进程名合计成一行。"""
    listed_indexes = set(sorted(range(len(kills)), key=lambda index: -kills[index]["anon_rss_kibibytes"])[:KILLS_LISTED_ONE_BY_ONE])
    lines = [f"  - {kill['moment'].strftime('%Y-%m-%d %H:%M:%S')} UTC / {kill['moment'].astimezone(TOKYO).strftime('%m-%d %H:%M:%S')} JST  "
             f"进程 {kill['pid']}（{kill['name']}）anon-rss {format_size(kill['anon_rss_kibibytes'])}"
             for kill in sorted((kills[index] for index in listed_indexes), key=lambda kill: kill["moment"])]
    rest = [kill for index, kill in enumerate(kills) if index not in listed_indexes]
    if rest:
        counts = {}
        for kill in rest:
            counts[kill["name"]] = counts.get(kill["name"], 0) + 1
        largest = max(kill["anon_rss_kibibytes"] for kill in rest)
        lines.append(f"  - 另有 {len(rest)} 次没逐条列（anon-rss 都不超过 {format_size(largest)}）："
                     + "、".join(f"{name} ×{count}" for name, count in sorted(counts.items(), key=lambda item: -item[1])))
    return lines


def oom_report(hours):
    if BROKEN_JUDGEMENT == "skip-oom":
        return ""
    journal_text, problem = read_kernel_journal(hours)
    if problem is not None:
        return "\n".join([
            f"【没查成：近 {hours:g} 小时内核有没有因为内存不够杀进程】{problem}。",
            f"- 怎么办：手动跑 `journalctl -k --since '-{hours:g}h' | grep 'Killed process'` 看一眼；查不了就当不知道，不当成没有 OOM——"
            "会话要是突然断掉才重开的，先别照原样重派断掉之前在跑的那件活。"])
    kills = killed_processes(journal_text)
    if not kills:
        return f"近 {hours:g} 小时内核没有因为内存不够杀过进程（journalctl -k 查过，看得到内核日志）。"
    whole_machine = [kill for kill in kills if kill["whole_machine"]]
    capped = [kill for kill in kills if not kill["whole_machine"]]
    lines = [f"【近 {hours:g} 小时内核因为内存不够杀过 {len(kills)} 个进程（journalctl -k 里的 Killed process 行）】"]
    if whole_machine:
        lines.append(f"- 整机 OOM（Out of memory）杀的 {len(whole_machine)} 个：整机内存耗尽，本地模型服务与 Claude Code 会话可能被连带杀掉")
        lines += kill_lines(whole_machine)
    if capped:
        lines.append(f"- 内存上限（cgroup）里杀的 {len(capped)} 个：撞的是那一个 cgroup 的上限（例 run-with-memory-cap.sh 给变异定的），整机没耗尽")
        lines += kill_lines(capped)
    lines.append("- 怎么办：先查这几个进程是谁起的、为什么涨（进程名对上是哪个实验二进制或测试，去派它的子 agent 的会话记录与变异表里找那一步），"
                 "查清之前别照原样重派同一件活；重派时让那一步经 research/scripts/run-with-memory-cap.sh 带上限跑。")
    return "\n".join(lines)


def reminder(hook_input, project_root):
    source = hook_input.get("source") or ""
    if source == "compact":
        return compact_reminder(hook_input, project_root)
    if source in ("startup", "resume"):
        return oom_report(oom_hours())
    return ""


FAKE_KILL_LINES = (
    "2026-09-25T07:58:02.631401+00:00 host kernel: Out of memory: Killed process 2405 (ray::IDLE) total-vm:22011912kB, anon-rss:33792kB, file-rss:3272kB, shmem-rss:0kB, UID:1000 pgtables:648kB oom_score_adj:1000\n"
    "2026-09-25T07:58:02.681372+00:00 host kernel: Out of memory: Killed process 1691212 (e142_region_dif) total-vm:67315408kB, anon-rss:56156416kB, file-rss:1460kB, shmem-rss:0kB, UID:1000 pgtables:120792kB oom_score_adj:0\n"
    "2026-09-25T08:20:52.953154+00:00 host kernel: Memory cgroup out of memory: Killed process 2295476 (python3) total-vm:222788kB, anon-rss:65280kB, file-rss:6820kB, shmem-rss:0kB, UID:1000 pgtables:196kB oom_score_adj:0\n"
    "2026-09-25T08:20:52.953000+00:00 host kernel: oom-kill:constraint=CONSTRAINT_MEMCG,task=python3,pid=2295476,uid=1000\n")


def write_fake_journalctl(directory, recent_output, probe_output, stderr_text="", exit_code=0):
    """假 journalctl：-n 1（探一下看不看得到内核日志）打 probe_output，其余打 recent_output；stderr 与退出码按给的来。"""
    for name, content in (("recent.txt", recent_output), ("probe.txt", probe_output), ("stderr.txt", stderr_text)):
        with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
            handle.write(content)
    script = os.path.join(directory, "journalctl")
    with open(script, "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\n"
                     f"here={directory!r}\n"
                     'case " $* " in *" -n 1 "*) cat "$here/probe.txt" ;; *) cat "$here/recent.txt" ;; esac\n'
                     'cat "$here/stderr.txt" >&2\n'
                     f"exit {exit_code}\n")
    os.chmod(script, 0o755)


def selftest(hook_dir):
    script = os.path.join(hook_dir, "session-start.sh")
    project_root = os.path.dirname(os.path.dirname(hook_dir))
    work = tempfile.mkdtemp(prefix="session-start-selftest-")
    failures, checked = [], 0

    def run_hook(source, journal_directory=None, path_without_journalctl=False):
        hook_input = {"hook_event_name": "SessionStart", "source": source, "session_id": "abc",
                      "transcript_path": "/home/x/.claude/projects/p/abc.jsonl"}
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=project_root)
        if journal_directory is not None:
            environment["PATH"] = journal_directory + os.pathsep + environment["PATH"]
        if path_without_journalctl:
            bare = os.path.join(work, "bare-bin")
            os.makedirs(bare, exist_ok=True)
            for tool in ("bash", "python3", "git", "dirname"):   # 入口要的几样，唯独没有 journalctl
                located = shutil.which(tool)
                if located and not os.path.exists(os.path.join(bare, tool)):
                    os.symlink(located, os.path.join(bare, tool))
            environment["PATH"] = bare
        return subprocess.run(["bash", script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=60)

    def expect(label, completed, wanted, unwanted=()):
        nonlocal checked
        checked += 1
        missing = [piece for piece in wanted if piece not in completed.stdout]
        present = [piece for piece in unwanted if piece in completed.stdout]
        if completed.returncode != 0 or missing or present:
            failures.append(f"{label}：退出码 {completed.returncode}（应当是 0），缺 {missing}，不该有却有 {present}；输出：{completed.stdout[-600:]}{completed.stderr[-300:]}")

    expect("压缩之后", run_hook("compact"),
           ["上下文刚压缩过", "分支 ", "工作区未提交", "最近动过的记录", "agent-watch.py report --session-dir /home/x/.claude/projects/p/abc"],
           ["Killed process", "内存不够"])
    journal = os.path.join(work, "with-kills")
    os.makedirs(journal)
    write_fake_journalctl(journal, FAKE_KILL_LINES, "2026-09-25T08:30:00+00:00 host kernel: something\n")
    for source in ("startup", "resume"):
        expect(f"{source} 且近几小时有 OOM", run_hook(source, journal),
               ["内存不够杀过 3 个进程", "整机 OOM（Out of memory）杀的 2 个",
                "2026-09-25 07:58:02 UTC / 09-25 16:58:02 JST  进程 1691212（e142_region_dif）anon-rss 53.6 GiB",
                "进程 2405（ray::IDLE）anon-rss 33 MiB", "内存上限（cgroup）里杀的 1 个", "进程 2295476（python3）anon-rss 64 MiB",
                "- 怎么办：先查这几个进程是谁起的、为什么涨", "查清之前别照原样重派同一件活"],
               ["上下文刚压缩过"])
    many = os.path.join(work, "many-kills")
    os.makedirs(many)
    many_lines = "".join(f"2026-09-25T07:58:{second:02d}.000000+00:00 host kernel: Out of memory: Killed process {3000 + second} (ray::IDLE) "
                         f"total-vm:1kB, anon-rss:{30000 + second}kB, file-rss:1kB\n" for second in range(20))
    write_fake_journalctl(many, many_lines + FAKE_KILL_LINES, "x\n")
    expect("杀得多时逐条列最大的、其余合计", run_hook("startup", many),
           ["进程 1691212（e142_region_dif）anon-rss 53.6 GiB", f"另有 {20 + 2 - KILLS_LISTED_ONE_BY_ONE} 次没逐条列", "ray::IDLE ×"])
    clean = os.path.join(work, "clean")
    os.makedirs(clean)
    write_fake_journalctl(clean, "2026-09-25T08:30:00+00:00 host kernel: usb 1-1: new device\n", "2026-09-25T08:30:00+00:00 host kernel: usb 1-1: new device\n")
    expect("看得到内核日志、没有 OOM", run_hook("startup", clean), ["内核没有因为内存不够杀过进程（journalctl -k 查过，看得到内核日志）"], ["没查成"])
    no_permission = os.path.join(work, "no-permission")
    os.makedirs(no_permission)
    write_fake_journalctl(no_permission, "", "", "Hint: You are currently not seeing messages from other users and the system.\n")
    expect("权限不够看不到内核日志", run_hook("startup", no_permission), ["没查成", "权限不够", "不当成没有 OOM"], ["内核没有因为内存不够杀过进程"])
    silent = os.path.join(work, "silent")
    os.makedirs(silent)
    write_fake_journalctl(silent, "", "")
    expect("连一条内核消息都读不出", run_hook("resume", silent), ["没查成", "连最近一条内核消息都读不出来"], ["内核没有因为内存不够杀过进程"])
    failing = os.path.join(work, "failing")
    os.makedirs(failing)
    write_fake_journalctl(failing, "", "", "Failed to open journal\n", exit_code=1)
    expect("journalctl 出错", run_hook("startup", failing), ["没查成", "journalctl 退出码 1"], ["内核没有因为内存不够杀过进程"])
    expect("没有 journalctl", run_hook("startup", path_without_journalctl=True), ["没查成", "没有 journalctl"], ["内核没有因为内存不够杀过进程"])
    expect("clear 不报", run_hook("clear", journal), [], ["Killed process", "内存不够", "上下文刚压缩过"])
    shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("    → 看 reminder() / oom_report() / read_kernel_journal() / killed_processes() 与入口；"
              "AFTER_COMPACT_DISABLE=1 或 SESSION_START_BREAK（skip-oom、unreadable-as-clean）设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 种）：压缩之后补分支、未提交数、最近的记录与看门狗命令；startup 与 resume 列近几小时内核杀的进程"
          f"（时间写 UTC 与 JST、进程号、进程名、anon-rss，整机 OOM 与 cgroup 上限分开，杀得多时逐条列最大的 {KILLS_LISTED_ONE_BY_ONE} 个、其余按进程名合计，"
          f"外加「怎么办」），看得到内核日志而没有 OOM 时说查过；权限不够、一条内核消息都读不出、journalctl 出错、没有 journalctl 都报「没查成」；clear 不报；退出码恒 0")
    return 0


hook_dir = sys.argv[1]
if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
    sys.exit(selftest(hook_dir))
try:
    hook_input = json.load(sys.stdin)
except ValueError:
    hook_input = {}
project_root = os.environ.get("CLAUDE_PROJECT_DIR") or hook_input.get("cwd") or os.getcwd()
text = reminder(hook_input, project_root)
if text:
    print(text)
sys.exit(0)
PY
