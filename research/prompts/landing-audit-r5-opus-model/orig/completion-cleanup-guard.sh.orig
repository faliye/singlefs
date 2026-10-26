#!/usr/bin/env bash
# 完工清理闸（Claude Code 钩子）：主 agent 结束一轮、子 agent 交回与收工时，自己认领、自己建的仓副本、编译目录与运行目录里的库拷贝还在，就拦下，
# 列出路径与 du 的大小，要它删完再收工（用完就删：.claude/main-agent.md「一轮怎么开、怎么收」第 11 条，.claude/agent-common.md「写」一节）。
# hook-events: Stop SubagentStop PreToolUse:SubagentHandback
# gate-similar: handback-scratch-check.sh 上游的交回尾门禁，同挂 PreToolUse:SubagentHandback 与 SubagentStop、同样判自己建的编译目录与仓副本，但只看临时目录（/tmp、$TMPDIR）、按调用时间窗认归属、交回报告写一行「没删」就放行，也不管主 agent；这里看工作根里本会话认领的目录与崩溃放量的运行目录、按归属文件认、只认交付物声明、不收「没删」说明、还挂主 agent 的 Stop；它在上游副本里，不许就地改，并不进去
# gate-similar: handback-guard.sh 同挂 PreToolUse:SubagentHandback，判交回正文、报告与自己起的后台任务还在跑；这里判磁盘上留下的副本。后台任务在跑的认法两边都从 scripts/agents/agent-watch.py 导入 own_background_tasks_still_running，不另写一份；判的对象与出路不同，并进去要让交回正文闸兼管归属文件与运行目录
# gate-similar: remind-rearm-stop-guard.sh 同挂 Stop，判循环提醒的登记有没有进程在等、没在等就交出重起命令；这里判磁盘上的副本，读的输入（提醒登记目录对归属文件、会话记录与运行目录）与放行条件都不同
# gate-similar: gate-reuse-check.sh 上游的 Stop / SubagentStop 收工自检，判这一轮新写的门禁与钩子写没写 gate-similar，本仓没注册；对象不同
#
# 什么时候判：
#   Stop（主 agent）：手里没有在跑的后台任务与子 agent 时才判；有就放行、什么都不打。
#     在跑的后台任务：会话记录里起过的后台任务此刻还有进程开着它的输出文件（从 scripts/agents/agent-watch.py 导入 own_background_tasks_still_running，与交回闸 ④ 同一份认法）；
#       开着输出文件的最上层进程，命令（去掉 Bash 工具的外包装之后）跑的全是循环提醒 remind.sh 或看门狗 watch.sh、agent-watch.py 的，不算在跑的活。
#     在跑的子 agent：主会话目录 subagents/ 下的会话记录，最后修改在 SUBAGENT_IDLE_SECONDS（30 分钟）以内，而主会话记录里没有它最后一次写之后（放 NOTIFICATION_SLACK_SECONDS）
#       的任务通知或 TaskStop（从 agent-watch.py 导入 session_task_events）。30 分钟没写过的不算在跑。
#   SubagentStop 与交回工具 SubagentHandback 的 PreToolUse（子 agent）：它自己起的后台任务还有进程开着输出文件时放行（那一条归交回闸 ④），别的时候都判。
#
# 判哪几处：
#   主 agent：工作根（.claude/hooks/lib_work_root.py）下一层、归属文件 .owner 的 session 是本会话的每个目录（scripts/hosts/work-directory-ownership.sh 写的那份；
#     子 agent 认领的会话号与主 agent 相同，也归本会话），加本会话的临时草稿目录 <旧根>/<项目目录名>/<会话号>/（下一层的 tasks/ 不看）。
#   子 agent：它自己的 Bash 调用里 work-directory-ownership.sh claim 认领的目录（归属文件要是本会话的），与派发提示里「草稿目录：」点名的那一个
#     （在工作根下；它所在的那个工作根下一层目录有归属文件的，要是本会话的）。主 agent 的别的目录、兄弟子 agent 的目录不判。
#   两者都另判崩溃放量的运行目录：$SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/ 下一层（没设时 ~/.local/share/singlefs/crash-amplification*/runs/ 下一层），
#     只判认得出确凿归属的；创建时间与 Bash 调用的时间窗重叠一条不算。确凿归属这样认：
#     起它的进程：运行目录名里的「<时刻>-<进程号>」（入口脚本拿自己的 $$ 起名，建目录的是它或它的子进程）那个进程此刻还在、不是僵尸、
#       开跑时刻（/proc/<pid>/stat 的 starttime）不晚于目录的创建时间（statx 的 btime）加 CREATOR_START_SLACK_SECONDS（晚于的是进程号被重用了）。
#       它还在就是那一趟还在跑：整个运行目录不列。它环境里的会话号（依次取 SESSION_VARIABLES，同 scripts/hosts/work-directory-ownership.sh）是归属；
#       顺着父进程往上找父进程是 Claude 进程（它环境里的 CLAUDE_PID）的那一层，就是起它的那次 Bash 调用的 shell，取它 -c 里 eval 的那条命令，记下 sha256。
#       看到一次就排他记账：<工作根>/completion-cleanup-guard/run-owners/<运行目录名>-<路径摘要>.json（每个会话的这道闸都记看到的全部运行目录，先看到的记，记了不改；
#       运行目录不在了的那几份顺手删）。
#     那个进程不在了：照记账认；运行目录里有归属记录 .owner（三行，写法同 work-directory-ownership.sh 的归属文件，入口脚本建运行目录时写）的，会话号以它为准。
#     归属是本会话的：主 agent 算自己的；子 agent 另要记账里那条命令的 sha256 等于它自己会话记录里某条 Bash 命令的（只有归属记录、没有记账的归主 agent）。
#     认不出的（进程不在、没记账、没归属记录）不列；其中创建时间落在它自己某次 Bash 调用的时间窗里（发起到结果，后台跑的到完成通知，没有的到现在，
#       两头各放 WINDOW_SLACK_SECONDS；那条命令点了 RUN_COMMAND_KEYWORDS 之一：崩溃放量、崩溃点重放、广谱变异、门禁、detached-run.sh）、里面又留了副本的，
#       拦下时在单子后面另起一节「认不出是谁的」列出来，只报不拦；不拦的时候不打。
# 判什么（逐个目录往下找，认出一样就不再往里找；认领目录与草稿目录最深 SCAN_MAX_DEPTH 层，运行目录 RUN_SCAN_MAX_DEPTH 层；不跟符号链接；仓根本身与它的上层不判）：
#   编译目录：带签名对得上的 CACHEDIR.TAG，或带 .rustc_info.json（cargo 的 target 与 CARGO_TARGET_DIR 指的目录都带）；
#   仓副本：有 .git（目录；.git 是文件的报成工作树），或同时有 Cargo.toml 与 crates（这两样的认法从 agent-watch.py 导入 scratch_hog_kind）；
#   库拷贝：同时有 CURRENT 与 MANIFEST-*（判定库的整份拷贝：送第二台的库快照、取回的库这类）。
#   在用的不列，判法调 scripts/hosts/run_build_directory.py 的 open_holders（与它的 holders 同一份：本进程以外的进程的工作目录、根目录、可执行文件、
#     打开的文件描述符或内存映射落在里面；只看得到本用户的）。认领目录与草稿目录看列出的那一处（含它底下各层）；运行目录看整个运行目录：
#     那一趟的日志或别的子目录有进程开着，就是那一趟还在跑，里面的库拷贝与编译目录都不列。
# 要留的交付物：报告、补丁与打补丁前的备份是普通文件与目录的，本来就不在单上。要留的副本写进它所在那个认领目录的 .deliverables
#   （与归属文件同一层；子 agent 的草稿目录本身也认），一行一条：
#     <相对那个目录的路径>  <种类>  <为什么留>      种类四选一：报告、补丁、备份、交接；为什么留至少 4 个字；# 起头的行是注释
#   交接：子 agent 交给主 agent 接着用的副本（主 agent 照它的报告在里面跑命令）。子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删。
#   声明了的那一棵里仓副本与库拷贝不列，编译目录照列（编译目录不能声明成交付物）；路径落到那个目录外、种类不认得、理由太短的行不算，列进拦下的说明。
# 拦下：Stop 与 SubagentStop 打 {"decision": "block", "reason": …}；PreToolUse 退 2、说明写 stderr。说明逐个列种类、路径与 du -sh 的大小
#   （du 合起来最多 DU_TIMEOUT_SECONDS 秒，量不出写「?」）和删法。
# 拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户定（免得卡死）；单子变了照拦。PreToolUse 每次都拦。
#   拦下的单记在 <工作根>/completion-cleanup-guard/<会话号>-<main 或子 agent 号>.json。
# 判不了（输入不是 JSON 对象、认不出会话号、找不到会话记录、导入不了共用模块、自己出错）：放行，stderr 写一句「没判」。
# 认不出的（照样要删）：没认领、也不是派发提示点名的草稿目录里建的；起它的进程这道闸一次都没看到还在跑、入口脚本又没写归属记录的运行目录
#   （从开跑到跑完之间没有哪个会话收过工、交过回；只在「认不出是谁的」里报）；第二台上的副本。
# 退役判据：工作根与崩溃放量的运行目录不再放副本与库拷贝（驱动与脚本都改成每趟自己的目录、跑完自己删），连续四周这道闸一次都没拦。
#
#   completion-cleanup-guard.sh              从 stdin 读钩子的 JSON
#   completion-cleanup-guard.sh --selftest   临时目录里造工作根、运行目录、会话记录与占着文件的假进程，逐格核；没设弄坏开关时末尾逐个打开每个开关（并行），核它红在点名的格
# 弄坏开关（只在自证里认，平常跑设了就退 2；几个用逗号连）。逐条写开关名与值、弄坏哪一处，下一行是打开后红的那一格（「」里是失败行的开头）：
#   COMPLETION_CLEANUP_BREAK=build-directory-unrecognized  不认编译目录（CACHEDIR.TAG 与 .rustc_info.json）
#       「✗ 自证 main-build-directory-left：」
#   COMPLETION_CLEANUP_BREAK=repository-copy-unrecognized  不认仓副本（.git、Cargo.toml 加 crates）
#       「✗ 自证 main-repository-copy-left：」
#   COMPLETION_CLEANUP_BREAK=library-copy-unrecognized  不认库拷贝（CURRENT 加 MANIFEST-*）
#       「✗ 自证 main-run-library-copy-left：」
#   COMPLETION_CLEANUP_BREAK=all-sessions  归属是别的会话的目录也当本会话的判
#       「✗ 自证 other-session-directory-ignored：」
#   COMPLETION_CLEANUP_BREAK=deliverables-ignored  不读 .deliverables
#       「✗ 自证 deliverables-only-passes：」
#   COMPLETION_CLEANUP_BREAK=build-directory-declarable  声明成交付物的那一棵里编译目录也不列
#       「✗ 自证 build-directory-never-deliverable：」
#   COMPLETION_CLEANUP_BREAK=handover-exempt-for-main  主 agent 也不列声明成交接的副本
#       「✗ 自证 main-handover-copy-left：」
#   COMPLETION_CLEANUP_BREAK=background-ignored  不看后台任务在不在跑
#       「✗ 自证 background-running-passes：」
#   COMPLETION_CLEANUP_BREAK=reminder-counts-as-running  循环提醒的后台任务也算在跑的活
#       「✗ 自证 reminder-only-still-judged：」
#   COMPLETION_CLEANUP_BREAK=subagents-ignored  不看子 agent 在不在跑
#       「✗ 自证 subagent-running-passes：」
#   COMPLETION_CLEANUP_BREAK=notifications-ignored  不看任务通知，写过不到 30 分钟的子 agent 都当在跑
#       「✗ 自证 subagent-finished-still-judged：」
#   COMPLETION_CLEANUP_BREAK=open-files-ignored  不看 /proc，有进程在用的也列
#       「✗ 自证 held-copy-not-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-sees-whole-session  子 agent 也判本会话认领的全部目录
#       「✗ 自证 subagent-ignores-siblings：」
#   COMPLETION_CLEANUP_BREAK=prompt-scratch-ignored  不认派发提示里「草稿目录：」点名的那一个
#       「✗ 自证 subagent-assigned-directory-left：」
#   COMPLETION_CLEANUP_BREAK=run-windows-ignored  运行目录不认归属，全算自己的
#       「✗ 自证 foreign-run-directory-ignored：」
#   COMPLETION_CLEANUP_BREAK=attribution-by-window  运行目录退回按时间窗认归属（创建时间落在自己 Bash 调用的时间窗里就算自己的）
#       「✗ 自证 foreign-session-run-in-window-not-listed：」
#   COMPLETION_CLEANUP_BREAK=ledger-ignored  不记、不读起它的进程的记账
#       「✗ 自证 own-run-blocked-after-creator-exits：」
#   COMPLETION_CLEANUP_BREAK=creator-alive-ignored  起它的进程还在也照判
#       「✗ 自证 run-in-progress-not-listed：」
#   COMPLETION_CLEANUP_BREAK=subagent-run-session-level  子 agent 把本会话别的 Bash 调用起的运行目录也算自己的
#       「✗ 自证 subagent-ignores-sibling-runs：」
#   COMPLETION_CLEANUP_BREAK=run-holders-leaf-only  运行目录里只看列出的那一处有没有进程开着，不看整个运行目录
#       「✗ 自证 run-held-deep-not-listed：」
#   COMPLETION_CLEANUP_BREAK=release-never  续跑回来还是同一张单也照拦
#       「✗ 自证 unchanged-after-block-releases：」
#   COMPLETION_CLEANUP_BREAK=release-on-any-continuation  续跑回来单子变了也放行
#       「✗ 自证 changed-after-block-blocks-again：」
#   COMPLETION_CLEANUP_BREAK=size-omitted  单上不写 du 的大小
#       「✗ 自证 block-lists-size：」
#   COMPLETION_CLEANUP_BREAK=scratchpad-ignored  主 agent 不判本会话的临时草稿目录
#       「✗ 自证 main-scratchpad-copy-left：」
#   COMPLETION_CLEANUP_BREAK=guard-off  平常跑设了弄坏开关也不拒
#       「✗ 自证 break-switch-refused-outside-selftest：」
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给钩子的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import concurrent.futures, glob, hashlib, importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile, time, traceback
from datetime import datetime, timezone

sys.dont_write_bytecode = True
HOOK_DIR = sys.argv[1]
ARGUMENTS = sys.argv[2:]
HOOK_PATH = os.path.join(HOOK_DIR, "completion-cleanup-guard.sh")
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(HOOK_DIR))
BREAK_VARIABLE = "COMPLETION_CLEANUP_BREAK"
BREAK_TABLE = [
    ("build-directory-unrecognized", "main-build-directory-left"),
    ("repository-copy-unrecognized", "main-repository-copy-left"),
    ("library-copy-unrecognized", "main-run-library-copy-left"),
    ("all-sessions", "other-session-directory-ignored"),
    ("deliverables-ignored", "deliverables-only-passes"),
    ("build-directory-declarable", "build-directory-never-deliverable"),
    ("handover-exempt-for-main", "main-handover-copy-left"),
    ("background-ignored", "background-running-passes"),
    ("reminder-counts-as-running", "reminder-only-still-judged"),
    ("subagents-ignored", "subagent-running-passes"),
    ("notifications-ignored", "subagent-finished-still-judged"),
    ("open-files-ignored", "held-copy-not-listed"),
    ("subagent-sees-whole-session", "subagent-ignores-siblings"),
    ("prompt-scratch-ignored", "subagent-assigned-directory-left"),
    ("run-windows-ignored", "foreign-run-directory-ignored"),
    ("attribution-by-window", "foreign-session-run-in-window-not-listed"),
    ("ledger-ignored", "own-run-blocked-after-creator-exits"),
    ("creator-alive-ignored", "run-in-progress-not-listed"),
    ("subagent-run-session-level", "subagent-ignores-sibling-runs"),
    ("run-holders-leaf-only", "run-held-deep-not-listed"),
    ("release-never", "unchanged-after-block-releases"),
    ("release-on-any-continuation", "changed-after-block-blocks-again"),
    ("size-omitted", "block-lists-size"),
    ("scratchpad-ignored", "main-scratchpad-copy-left"),
    ("guard-off", "break-switch-refused-outside-selftest"),
]
BREAK_ITEMS = tuple(item for item, _ in BREAK_TABLE)
SELFTEST_ENTRIES = (["--selftest"], ["--selftest-cells"])
NAME_PATTERN = re.compile(r"^[A-Za-z0-9_-]+$")
HANDBACK_TOOL = "SubagentHandback"
SUBAGENT_IDLE_SECONDS = 1800
NOTIFICATION_SLACK_SECONDS = 30
WINDOW_SLACK_SECONDS = 1.0
SCAN_MAX_DEPTH = 6
RUN_SCAN_MAX_DEPTH = 3
SCAN_MAX_DIRECTORIES = 50000
DU_TIMEOUT_SECONDS = 30
DECLARATION_FILE = ".deliverables"
DELIVERABLE_KINDS = ("报告", "补丁", "备份", "交接")
HANDOVER_KIND = "交接"
REASON_MINIMUM_CHARACTERS = 4
RUN_COMMAND_KEYWORDS = re.compile(r"crash-amplification|crash-point-replay|broad-mutation|mutation-shard-run|checker-tier-|gate-staged|gate\.sh|detached-run\.sh")
TIMER_AND_WATCHDOG_SCRIPTS = ("remind.sh", "watch.sh", "agent-watch.py")
LAUNCHER_WORDS = ("bash", "sh", "python3", "python", "nice", "env", "exec")
CLAIM_PATTERN = re.compile(r"work-directory-ownership\.sh['\"]?\s+claim\s+(['\"]?)([^\s'\"]+)\1")
SCRATCH_LINE_PATTERN = re.compile(r"^[ \t]*草稿目录[：:][ \t]*`?([^\s`]+)`?", re.M)
STATE_DIRECTORY_NAME = "completion-cleanup-guard"
SCRATCHPAD_ROOT_VARIABLE = "COMPLETION_CLEANUP_SCRATCHPAD_ROOT"
KIND_BUILD, KIND_WORKTREE, KIND_REPOSITORY, KIND_LIBRARY = "编译目录", "工作树", "仓副本", "库拷贝"
RUN_NAME_PATTERN = re.compile(r"(?:^|-)([0-9]{8}T[0-9]{6}Z)-([0-9]+)(?=-|$)")
SESSION_VARIABLES = ("WORK_DIRECTORY_OWNER_SESSION", "CLAUDE_CODE_SESSION_ID", "CLAUDE_SESSION_ID")
CLAUDE_PID_VARIABLE = "CLAUDE_PID"
CREATOR_START_SLACK_SECONDS = 1.0
ANCESTRY_MAX_DEPTH = 64
RUN_LEDGER_NAME = "run-owners"
RUN_OWNER_FILE = ".owner"
UNKNOWN_SECTION_TITLE = "认不出是谁的"


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def refuse_break_outside_selftest():
    """弄坏开关只在自证里认：平常跑设了就退 2，点名变量；不认识的项名一样拒。"""
    value = os.environ.get(BREAK_VARIABLE, "")
    if not value:
        return
    if ARGUMENTS not in SELFTEST_ENTRIES and not broken("guard-off"):
        print(f"✗ 完工清理闸：{BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
        print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 bash {HOOK_PATH} --selftest", file=sys.stderr)
        sys.exit(2)
    unknown = [item for item in value.split(",") if item and item not in BREAK_ITEMS]
    if unknown:
        print(f"✗ 完工清理闸：{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识（只认 {'、'.join(BREAK_ITEMS)}）", file=sys.stderr)
        print(f"→ 怎么办：项名照 {HOOK_PATH} 文件头「弄坏开关」一段写", file=sys.stderr)
        sys.exit(2)


MODULE_PATHS = {
    "lib_work_root": os.path.join(HOOK_DIR, "lib_work_root.py"),
    "agent_watch": os.path.join(REPOSITORY_ROOT, "scripts", "agents", "agent-watch.py"),
    "session_transcript": os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", "session-transcript.py"),
    "run_build_directory": os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "run_build_directory.py"),
}
LOADED_MODULES = {}


def module(name):
    if name not in LOADED_MODULES:
        specification = importlib.util.spec_from_file_location(name, MODULE_PATHS[name])
        loaded = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(loaded)
        LOADED_MODULES[name] = loaded
    return LOADED_MODULES[name]


class CannotJudge(Exception):
    pass


def is_strictly_under(path, directory):
    return path.startswith(directory.rstrip("/") + "/")


def is_project_root_or_above(path):
    real = os.path.realpath(path)
    project = os.path.realpath(REPOSITORY_ROOT)
    return real == project or is_strictly_under(project, real)


# ── 归属与判哪几处
def owner_session(directory):
    """<目录>/.owner 三行（session、created、purpose）都在才认，交 session 的值；没有或写得不对交 None。"""
    path = os.path.join(directory, ".owner")
    if os.path.islink(path) or not os.path.isfile(path):
        return None
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError:
        return None
    values = dict(line.split(": ", 1) for line in lines if ": " in line)
    if len(lines) != 3 or not all(values.get(key) for key in ("session", "created", "purpose")):
        return None
    return values["session"]


def work_root():
    return module("lib_work_root").work_root()


def scratchpad_root():
    return os.environ.get(SCRATCHPAD_ROOT_VARIABLE) or module("lib_work_root").legacy_root()


def main_roots(session, main_transcript):
    """主 agent 判的目录：[(目录, 它的认领目录或 None, 下一层不看的名字)]。"""
    roots = []
    base = work_root()
    try:
        names = sorted(os.listdir(base))
    except OSError:
        names = []
    for name in names:
        path = os.path.join(base, name)
        if os.path.islink(path) or not os.path.isdir(path):
            continue
        owner = owner_session(path)
        if owner == session or (owner is not None and broken("all-sessions")):
            roots.append((path, path, set()))
    if not broken("scratchpad-ignored"):
        scratchpad = os.path.join(scratchpad_root(), os.path.basename(os.path.dirname(main_transcript)), session)
        if os.path.isdir(scratchpad) and not os.path.islink(scratchpad):
            roots.append((scratchpad, None, {"tasks"}))
    return roots


def first_user_text(transcript_path):
    """会话记录里第一条 user 记录的文字（子 agent 的派发提示）。"""
    try:
        with open(transcript_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if '"user"' not in line:
                    continue
                try:
                    record = json.loads(line)
                except ValueError:
                    continue
                message = record.get("message") if isinstance(record, dict) else None
                if not isinstance(message, dict) or message.get("role") != "user":
                    continue
                content = message.get("content")
                if isinstance(content, str):
                    return content
                if isinstance(content, list):
                    return "\n".join(block.get("text", "") for block in content if isinstance(block, dict) and block.get("type") == "text")
    except OSError:
        return ""
    return ""


def subagent_roots(session, agent_transcript, calls):
    """子 agent 判的目录：它自己 claim 的工作根下一层目录，与派发提示「草稿目录：」点名的那一个。"""
    base = work_root()
    candidates = []
    for call in calls:
        if call["name"] != "Bash":
            continue
        for match in CLAIM_PATTERN.finditer(str(call["input"].get("command") or "")):
            text = os.path.expanduser(match.group(2)).rstrip("/")
            candidates.append((os.path.normpath(text if "/" in text else os.path.join(base, text)), "claim"))
    if not broken("prompt-scratch-ignored"):
        for match in SCRATCH_LINE_PATTERN.finditer(first_user_text(agent_transcript)):
            written = module("lib_work_root").expand_written_path(match.group(1).rstrip("/"))
            candidates.append((os.path.normpath(os.path.abspath(written)), "assigned"))
    roots, seen = [], set()
    for path, origin in candidates:
        if path in seen or not is_strictly_under(path, base) or os.path.islink(path) or not os.path.isdir(path):
            continue
        top = os.path.join(base, os.path.relpath(path, base).split(os.sep)[0])
        owner = owner_session(top)
        if origin == "claim" and (path != top or owner != session):
            continue
        if owner is not None and owner != session:
            continue
        seen.add(path)
        roots.append((path, top, set()))
    return [root for root in roots if not any(is_strictly_under(root[0], other[0]) for other in roots)]


# ── 交付物声明
def declarations(directories, role):
    """读这几个目录里的 .deliverables：交 ({声明了的绝对路径: 种类}, [写坏的行])；只收这个角色免列的种类。"""
    exempt_kinds = set() if broken("deliverables-ignored") else {"报告", "补丁", "备份"}
    if exempt_kinds and (role == "subagent" or broken("handover-exempt-for-main")):
        exempt_kinds.add(HANDOVER_KIND)
    exempt, problems = {}, []
    for directory in dict.fromkeys(directory for directory in directories if directory):
        path = os.path.join(directory, DECLARATION_FILE)
        if os.path.islink(path) or not os.path.isfile(path):
            continue
        try:
            lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
        except OSError:
            problems.append(f"{path} 读不了，里面的声明不算")
            continue
        for number, line in enumerate(lines, 1):
            text = line.strip()
            if not text or text.startswith("#"):
                continue
            parts = text.split(None, 2)
            if len(parts) < 3 or parts[1] not in DELIVERABLE_KINDS or len(parts[2].strip()) < REASON_MINIMUM_CHARACTERS:
                problems.append(f"{path}:{number} 不是「<相对路径> <种类> <为什么留>」（种类：{'、'.join(DELIVERABLE_KINDS)}；理由至少 {REASON_MINIMUM_CHARACTERS} 个字），这一行不算")
                continue
            relative = parts[0].strip("`'\"").rstrip("/")
            target = os.path.normpath(os.path.join(directory, relative))
            if os.path.isabs(relative) or not is_strictly_under(target, directory):
                problems.append(f"{path}:{number} 的路径 {relative} 不在 {directory} 里面，这一行不算")
                continue
            if parts[1] in exempt_kinds:
                exempt[target] = parts[1]
    return exempt, problems


# ── 认副本
def kind_of(path, names):
    kind = module("agent_watch").scratch_hog_kind(path, names)
    if kind is None and ".rustc_info.json" in names:
        kind = KIND_BUILD
    if kind == KIND_BUILD and broken("build-directory-unrecognized"):
        kind = None
    if kind == KIND_REPOSITORY:
        if broken("repository-copy-unrecognized"):
            kind = None
        elif os.path.isfile(os.path.join(path, ".git")):
            kind = KIND_WORKTREE
    if (kind is None and "CURRENT" in names and any(name.startswith("MANIFEST-") for name in names)
            and not broken("library-copy-unrecognized")):
        kind = KIND_LIBRARY
    return kind


def scan(root, max_depth, exempt, skipped_names, budget):
    """从 root 往下找副本：交 [(路径, 种类)]；认出一样就不再往里找，声明了的那一棵只找编译目录。"""
    found = []
    stack = [(root, 0, False)]
    while stack:
        path, depth, inside_exempt = stack.pop()
        if budget["visited"] >= SCAN_MAX_DIRECTORIES:
            budget["truncated"].add(root)
            break
        budget["visited"] += 1
        if is_project_root_or_above(path):
            continue
        try:
            entries = list(os.scandir(path))
        except OSError:
            continue
        here_exempt = inside_exempt or path in exempt
        kind = kind_of(path, {entry.name for entry in entries})
        if kind is not None:
            if kind == KIND_BUILD:
                if not (here_exempt and broken("build-directory-declarable")):
                    found.append((path, kind))
                continue
            if not here_exempt:
                found.append((path, kind))
                continue
        if depth >= max_depth:
            continue
        for entry in sorted(entries, key=lambda item: item.name, reverse=True):
            if depth == 0 and entry.name in skipped_names:
                continue
            try:
                is_directory = entry.is_dir(follow_symlinks=False)
            except OSError:
                continue
            if is_directory:
                stack.append((entry.path, depth + 1, here_exempt))
    return found


# ── 运行目录
def run_roots():
    home = os.environ.get("SINGLEFS_CRASH_AMPLIFICATION_HOME")
    if home:
        return [os.path.join(home, "runs")]
    return sorted(glob.glob(os.path.expanduser("~/.local/share/singlefs/crash-amplification*/runs")))


def birth_times(paths):
    ordered = sorted(paths)
    if not ordered:
        return {}
    completed = subprocess.run(["stat", "--printf=%.9W\\t%n\\0", "--", *ordered], capture_output=True, text=True, check=False)
    reported = {}
    for record in completed.stdout.split("\0"):
        seconds_text, separator, path = record.partition("\t")
        if separator:
            try:
                reported[path] = float(seconds_text)
            except ValueError:
                pass
    return {path: (reported[path] if reported.get(path, 0.0) > 0 else None) for path in ordered}


BOOT_SECONDS = []


def boot_seconds():
    if not BOOT_SECONDS:
        value = None
        try:
            for line in open("/proc/stat", encoding="ascii", errors="replace"):
                if line.startswith("btime "):
                    value = float(line.split()[1])
        except (OSError, ValueError):
            value = None
        BOOT_SECONDS.append(value)
    return BOOT_SECONDS[0]


def process_status(pid):
    """(父进程号, 开跑时刻)：进程不在、是僵尸或读不了交 None。"""
    try:
        text = open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    fields = text.rpartition(")")[2].split()
    boot = boot_seconds()
    if len(fields) < 20 or fields[0] in ("Z", "X") or boot is None:
        return None
    try:
        return int(fields[1]), boot + int(fields[19]) / os.sysconf("SC_CLK_TCK")
    except ValueError:
        return None


def process_environment(pid):
    try:
        data = open(f"/proc/{pid}/environ", "rb").read()
    except OSError:
        return {}
    wanted = {name.encode() for name in (*SESSION_VARIABLES, CLAUDE_PID_VARIABLE)}
    environment = {}
    for item in data.split(b"\0"):
        key, separator, value = item.partition(b"=")
        if separator and key in wanted:
            environment[key.decode()] = value.decode(errors="replace")
    return environment


def session_of(environment):
    for name in SESSION_VARIABLES:
        value = environment.get(name, "")
        if value and NAME_PATTERN.match(value):
            return value
    return None


def evaluated_command(pid):
    """Bash 工具起的那层 shell（bash -c '… && eval <命令> && …'）里 eval 的那条命令；认不出交 None。"""
    try:
        words = [word.decode(errors="replace") for word in open(f"/proc/{pid}/cmdline", "rb").read().split(b"\0")]
    except OSError:
        return None
    if len(words) < 3 or words[1] != "-c":
        return None
    try:
        tokens = shlex.split(words[2])
    except ValueError:
        return None
    for index, token in enumerate(tokens[:-1]):
        if token == "eval":
            return tokens[index + 1]
    return None


def command_digest(command):
    return hashlib.sha256(command.strip().encode()).hexdigest()


def launcher_digest(pid, claude_pid):
    """从 pid 顺着父进程往上，找父进程是 Claude 进程的那一层，交它 eval 的命令的 sha256；找不到交 None。"""
    current = pid
    for _ in range(ANCESTRY_MAX_DEPTH):
        status = process_status(current)
        if status is None:
            return None
        if status[0] == claude_pid:
            command = evaluated_command(current)
            return command_digest(command) if command is not None else None
        if status[0] <= 1:
            return None
        current = status[0]
    return None


def live_creator(path, born):
    """运行目录名里的进程号此刻还在、开跑不晚于目录的创建时间：交 {"pid", "session", "launcher_sha256"}，否则 None。"""
    match = RUN_NAME_PATTERN.search(os.path.basename(path))
    if match is None or born is None:
        return None
    pid = int(match.group(2))
    status = process_status(pid)
    if status is None or status[1] > born + CREATOR_START_SLACK_SECONDS:
        return None
    environment = process_environment(pid)
    claude = environment.get(CLAUDE_PID_VARIABLE, "")
    return {"pid": pid, "session": session_of(environment),
            "launcher_sha256": launcher_digest(pid, int(claude)) if claude.isdigit() else None}


def ledger_directory():
    return os.path.join(module("lib_work_root").state_directory(STATE_DIRECTORY_NAME), RUN_LEDGER_NAME)


def ledger_path(path):
    return os.path.join(ledger_directory(), f"{os.path.basename(path)}-{hashlib.sha256(path.encode()).hexdigest()[:12]}.json")


def record_ledger(path, born, creator):
    """排他记账：先写临时文件，再 link 到记账名（已有就不动）。"""
    if broken("ledger-ignored") or not creator.get("session"):
        return
    target = ledger_path(path)
    temporary = f"{target}.{os.getpid()}.tmp"
    try:
        os.makedirs(os.path.dirname(target), exist_ok=True)
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump({"path": path, "born": born, **creator}, handle, ensure_ascii=False)
        os.link(temporary, target)
    except OSError:
        pass
    finally:
        try:
            os.remove(temporary)
        except OSError:
            pass


def ledger_entry(path, born):
    if broken("ledger-ignored") or born is None:
        return None
    try:
        entry = json.load(open(ledger_path(path), encoding="utf-8"))
    except (OSError, ValueError):
        return None
    if not isinstance(entry, dict) or entry.get("path") != path or not isinstance(entry.get("born"), (int, float)) or abs(entry["born"] - born) > 0.001:
        return None
    return entry


def prune_ledger():
    """运行目录不在了的记账删掉。"""
    directory = ledger_directory()
    try:
        names = os.listdir(directory)
    except OSError:
        return
    for name in names:
        if not name.endswith(".json"):
            continue
        try:
            recorded = json.load(open(os.path.join(directory, name), encoding="utf-8")).get("path")
        except (OSError, ValueError, AttributeError):
            continue
        if isinstance(recorded, str) and recorded and not os.path.lexists(recorded):
            try:
                os.remove(os.path.join(directory, name))
            except OSError:
                pass


def run_owner(path, born):
    """交 (状态, 归属)：状态是 running（起它的进程还在）、known 或 unknown；归属 {"session", "launcher_sha256"}。"""
    creator = live_creator(path, born)
    if creator is not None:
        record_ledger(path, born, creator)
        if not broken("creator-alive-ignored"):
            return "running", creator
    entry = ledger_entry(path, born) or (creator if creator and creator.get("session") else None)
    recorded = owner_session(path)
    if recorded:
        same = entry is not None and entry.get("session") == recorded
        return "known", {"session": recorded, "launcher_sha256": entry.get("launcher_sha256") if same else None}
    if entry is not None:
        return "known", {"session": entry.get("session"), "launcher_sha256": entry.get("launcher_sha256")}
    return "unknown", None


def call_windows(calls, now):
    return [(call["started_at"] - WINDOW_SLACK_SECONDS, (call["ended_at"] if call["ended_at"] is not None else now) + WINDOW_SLACK_SECONDS)
            for call in calls if call["name"] == "Bash" and RUN_COMMAND_KEYWORDS.search(str(call["input"].get("command") or ""))]


def classify_runs(role, session, own_transcript, calls, now, notes):
    """交 (自己的运行目录, 认不出是谁、创建时间落在自己 Bash 调用时间窗里的运行目录)。起它的进程还在的两边都不放。"""
    directories = []
    for root in run_roots():
        try:
            names = sorted(os.listdir(root))
        except OSError:
            continue
        directories += [os.path.join(root, name) for name in names
                        if os.path.isdir(os.path.join(root, name)) and not os.path.islink(os.path.join(root, name))]
    if not directories:
        return [], []
    prune_ledger()
    if broken("run-windows-ignored"):
        return directories, []
    transcripts = module("session_transcript")
    started = transcripts.epoch_seconds(transcripts.first_timestamp(own_transcript))
    windows = call_windows(calls, now)
    own_digests = {command_digest(str(call["input"].get("command") or "")) for call in calls if call["name"] == "Bash"}

    def recent(path):
        try:
            return started is None or os.lstat(path).st_mtime >= started - WINDOW_SLACK_SECONDS
        except OSError:
            return False

    def creator_alive(path):
        match = RUN_NAME_PATTERN.search(os.path.basename(path))
        return match is not None and os.path.exists(f"/proc/{match.group(2)}")

    births = birth_times([path for path in directories if recent(path) or creator_alive(path)])
    missing = sorted(path for path, born in births.items() if born is None)
    if missing:
        notes.append(f"取不到创建时间的运行目录 {len(missing)} 个没判（认不出起它的进程）：{' '.join(missing[:5])}{' …' if len(missing) > 5 else ''}")
    own, unknown = [], []
    for path, born in sorted(births.items()):
        if born is None:
            continue
        state, owner = run_owner(path, born)
        if state == "running":
            continue
        in_window = (started is None or born >= started - WINDOW_SLACK_SECONDS) and any(start <= born <= end for start, end in windows)
        if broken("attribution-by-window"):
            if in_window:
                own.append(path)
            continue
        if owner is not None and owner.get("session") == session:
            if role == "main" or broken("subagent-run-session-level") or owner.get("launcher_sha256") in own_digests:
                own.append(path)
            continue
        if owner is None and in_window:
            unknown.append(path)
    return own, unknown


# ── 在跑的活
def launched_script(command):
    """Bash 工具外包装去掉之后，这条命令跑的是哪个脚本（basename）；是复合命令或认不出交 None。"""
    shown = module("agent_watch").command_for_display(command)
    if re.search(r"&&|\|\||[;|&]", shown):
        return None
    try:
        words = shlex.split(shown)
    except ValueError:
        words = shown.split()
    while words and (re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", words[0]) or os.path.basename(words[0]) in LAUNCHER_WORDS or words[0].startswith("-")):
        words = words[1:]
    return os.path.basename(words[0]) if words else None


def background_work(transcript):
    """自己起的、还有进程开着输出文件的后台任务（循环提醒与看门狗不算）：[(任务号, 命令)]。"""
    if broken("background-ignored"):
        return []
    work = []
    for task_id, _, tops in module("agent_watch").own_background_tasks_still_running(transcript):
        commands = [command for _, _, command, _ in tops]
        if (commands and not broken("reminder-counts-as-running")
                and all(launched_script(command) in TIMER_AND_WATCHDOG_SCRIPTS for command in commands)):
            continue
        work.append((task_id, commands[0] if commands else ""))
    return work


def running_subagents(main_transcript, now):
    if broken("subagents-ignored"):
        return []
    paths = glob.glob(os.path.join(main_transcript[:-len(".jsonl")], "subagents", "agent-*.jsonl"))
    recent = []
    for path in paths:
        try:
            modified = os.stat(path).st_mtime
        except OSError:
            continue
        if now - modified <= SUBAGENT_IDLE_SECONDS:
            recent.append((path, modified))
    if not recent:
        return []
    stops, notifications, _ = module("agent_watch").session_task_events(main_transcript)
    running = []
    for path, modified in recent:
        agent_id = os.path.basename(path)[len("agent-"):-len(".jsonl")]
        _, notified_at = notifications.get(agent_id, (None, None))
        stopped_at = stops.get(agent_id)
        ended = any(moment is not None and modified <= moment.timestamp() + NOTIFICATION_SLACK_SECONDS for moment in (notified_at, stopped_at))
        if ended and not broken("notifications-ignored"):
            continue
        running.append(agent_id)
    return running


# ── 在用
def in_use(path):
    """有进程开着 path 里的文件（scripts/hosts/run_build_directory.py 的 open_holders，与它的 holders 同一份判法）。"""
    return bool(module("run_build_directory").open_holders(os.path.abspath(path)))


def not_in_use(found_in_roots, found_in_runs):
    """认领目录与草稿目录里看列出的那一处；运行目录看整个运行目录（弄坏开关 run-holders-leaf-only 时只看列出的那一处）。"""
    if broken("open-files-ignored"):
        return found_in_roots + [item for items in found_in_runs.values() for item in items]
    kept = [(path, kind) for path, kind in found_in_roots if not in_use(path)]
    for run_directory, items in found_in_runs.items():
        if broken("run-holders-leaf-only"):
            kept += [(path, kind) for path, kind in items if not in_use(path)]
        elif items and not in_use(run_directory):
            kept += items
    return kept


# ── 单子
def sizes(paths):
    if not paths:
        return {}
    try:
        completed = subprocess.run(["timeout", str(DU_TIMEOUT_SECONDS), "du", "-sh", "--", *paths], capture_output=True, text=True, check=False)
    except OSError:
        return {}
    measured = {}
    for line in completed.stdout.splitlines():
        size, separator, path = line.partition("\t")
        if separator:
            measured[path] = size
    return measured


def removal_command(path, kind):
    if kind == KIND_WORKTREE:
        try:
            pointer = open(os.path.join(path, ".git"), encoding="utf-8").read().strip()
        except OSError:
            pointer = ""
        source = pointer[len("gitdir: "):].split("/.git/worktrees/")[0] if pointer.startswith("gitdir: ") and "/.git/worktrees/" in pointer else ""
        if source:
            return f"git -C {shlex.quote(source)} worktree remove --force {shlex.quote(path)}"
    return f"rm -rf -- {shlex.quote(path)}"


def leftover_lines(leftovers):
    measured = {} if broken("size-omitted") else sizes([path for path, _ in leftovers])
    lines = []
    for path, kind in leftovers:
        lines.append(f"  {kind}  {path}" if broken("size-omitted") else f"  {kind}  {path}  {measured.get(path, '?')}")
    return lines


def unknown_lines(unknown):
    if not unknown:
        return []
    return [f"「{UNKNOWN_SECTION_TITLE}」（只报不拦：运行目录建在你某次 Bash 调用的时间窗里，但起它的进程已经不在、没记账、也没有归属记录，认不出是不是你起的；"
            f"删之前先核是不是你那一趟的，不是的别动）：", *leftover_lines(unknown)]


def block_text(role, leftovers, problems, notes, unknown=()):
    if role == "main":
        head = "✗ 完工清理：这一轮做完了（没有在跑的后台任务与子 agent），本会话认领的目录与它起的运行目录里还剩下面这些仓副本、编译目录与库拷贝；用完就删，删完再收工。"
        rule = ".claude/main-agent.md「一轮怎么开、怎么收」第 11 条"
        kinds = "报告、补丁、备份"
    else:
        head = "✗ 完工清理：交回之前先删掉你自己认领、自己建的仓副本、编译目录与库拷贝（下面这些还在）；用完就删，删完再交回。"
        rule = ".claude/agent-common.md「写」一节"
        kinds = "报告、补丁、备份、交接（交给主 agent 接着用的副本，报告里写明主 agent 在里面跑什么）"
    lines = [head, *leftover_lines(leftovers),
             "→ 怎么办：逐个删（删之前 du -sh 记下的大小写进报告），删法：",
             *[f"    {removal_command(path, kind)}" for path, kind in leftovers],
             f"   不留给以后的趟复用，也不等别的活来用（{rule}）。",
             f"   报告、补丁与打补丁前的备份是普通文件的不在单上；要留的副本写进它所在认领目录的 {DECLARATION_FILE}：一行「<相对路径> <种类> <为什么留>」，"
             f"种类是 {kinds}；编译目录声明了也照列。写法在 {HOOK_PATH} 文件头「要留的交付物」。"]
    lines += [f"   ! {problem}" for problem in problems + notes]
    lines += unknown_lines(list(unknown))
    return "\n".join(lines)


def release_text(leftovers, unknown=()):
    return "\n".join(["完工清理闸：拦下之后这几项一个没删，这一次放行收工，交用户定：", *leftover_lines(leftovers), *unknown_lines(list(unknown))])


def state_path(session, actor):
    return os.path.join(module("lib_work_root").state_directory(STATE_DIRECTORY_NAME), f"{session}-{actor}.json")


def load_state(path):
    try:
        return json.load(open(path, encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def save_state(path, paths):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump({"paths": paths}, handle, ensure_ascii=False)
    os.replace(temporary, path)


def clear_state(path):
    try:
        os.remove(path)
    except OSError:
        pass


# ── 判
def decide(hook_input):
    """钩子输入 → {"action": pass / block / release / skip, "text": …, "paths": […]}。"""
    if not isinstance(hook_input, dict):
        return {"action": "skip", "text": "钩子输入不是 JSON 对象"}
    event = hook_input.get("hook_event_name")
    session = str(hook_input.get("session_id") or "")
    main_transcript = str(hook_input.get("transcript_path") or "")
    if event == "Stop":
        role, actor, own_transcript = "main", "main", main_transcript
    elif event == "SubagentStop":
        role, actor, own_transcript = "subagent", str(hook_input.get("agent_id") or ""), str(hook_input.get("agent_transcript_path") or "")
        if not actor and own_transcript:
            actor = os.path.basename(own_transcript)[len("agent-"):-len(".jsonl")]
    elif event == "PreToolUse":
        actor = str(hook_input.get("agent_id") or "")
        if hook_input.get("tool_name") != HANDBACK_TOOL or not actor or actor == "main":
            return {"action": "pass"}
        role = "subagent"
        own_transcript = f"{main_transcript[:-len('.jsonl')]}/subagents/agent-{actor}.jsonl" if main_transcript.endswith(".jsonl") else ""
    else:
        return {"action": "pass"}
    if not NAME_PATTERN.match(session) or not NAME_PATTERN.match(actor):
        return {"action": "skip", "text": "认不出会话号或子 agent 号"}
    if not own_transcript or not os.path.isfile(own_transcript) or (role == "main" and not main_transcript.endswith(".jsonl")):
        return {"action": "skip", "text": f"找不到会话记录（{own_transcript or '输入里没有'}）"}
    now = time.time()
    calls, _ = module("session_transcript").tool_calls(own_transcript)
    roots = main_roots(session, main_transcript) if role == "main" or broken("subagent-sees-whole-session") else subagent_roots(session, own_transcript, calls)
    exempt, problems = declarations([directory for root in roots for directory in (root[0], root[1])], role)
    budget = {"visited": 0, "truncated": set()}
    notes = []
    found_in_roots = []
    for path, _, skipped in roots:
        found_in_roots += scan(path, SCAN_MAX_DEPTH, exempt, skipped, budget)
    own_runs, unknown_runs = classify_runs(role, session, own_transcript, calls, now, notes)
    found_in_runs = {path: scan(path, RUN_SCAN_MAX_DEPTH, {}, set(), budget) for path in own_runs}
    found_unknown = {path: scan(path, RUN_SCAN_MAX_DEPTH, {}, set(), budget) for path in unknown_runs}
    if budget["truncated"]:
        notes.append(f"看了 {SCAN_MAX_DIRECTORIES} 个目录到顶，这几处没看完：{' '.join(sorted(budget['truncated']))}")
    leftovers = sorted(dict(not_in_use(found_in_roots, found_in_runs)).items())
    state_file = state_path(session, actor)
    if not leftovers:
        clear_state(state_file)
        return {"action": "pass", "notes": notes, "unknown": sorted(dict(not_in_use([], found_unknown)).items())}
    if background_work(own_transcript) or (role == "main" and running_subagents(main_transcript, now)):
        return {"action": "pass", "running": True}
    unknown = sorted(dict(not_in_use([], found_unknown)).items())
    paths = [path for path, _ in leftovers]
    if event != "PreToolUse" and hook_input.get("stop_hook_active") is True and not broken("release-never"):
        if load_state(state_file).get("paths") == paths or broken("release-on-any-continuation"):
            return {"action": "release", "text": release_text(leftovers, unknown), "paths": paths, "unknown": unknown}
    save_state(state_file, paths)
    return {"action": "block", "text": block_text(role, leftovers, problems, notes, unknown), "paths": paths, "unknown": unknown}


def hook_main():
    try:
        hook_input = json.loads(sys.stdin.read())
    except ValueError:
        return 0
    try:
        outcome = decide(hook_input)
    except Exception:  # noqa: BLE001 —— 自己出错不拦，免得一个缺陷挡住所有收工
        print("! 完工清理闸自己出错了，这一次没判（放行）：", file=sys.stderr)
        traceback.print_exc(file=sys.stderr)
        return 0
    action = outcome["action"]
    if action == "skip":
        print(f"! 完工清理闸：{outcome['text']}，这一次没判（放行）", file=sys.stderr)
        return 0
    if action == "release":
        print(json.dumps({"systemMessage": outcome["text"]}, ensure_ascii=False))
        return 0
    if action == "block":
        if hook_input.get("hook_event_name") == "PreToolUse":
            print(outcome["text"], file=sys.stderr)
            return 2
        print(json.dumps({"decision": "block", "reason": outcome["text"]}, ensure_ascii=False))
    return 0


# ── 自证：每一格造一个答案已知的局面，在本进程里调 decide（包装那几格起钩子本身）
def iso(seconds):
    return datetime.fromtimestamp(seconds, timezone.utc).isoformat().replace("+00:00", "Z")


class Scenario:
    """一格一个：临时目录里的工作根、崩溃放量的家目录、旧根、项目目录与主会话记录。"""

    def __init__(self, base, name):
        self.root = os.path.join(base, name)
        self.work = os.path.join(self.root, "work")
        self.home = os.path.join(self.root, "crash-home")
        self.runs = os.path.join(self.home, "runs")
        self.scratch = os.path.join(self.root, "legacy-root")
        self.project = os.path.join(self.root, "projects", "project-directory")
        self.session = "s-" + re.sub(r"[^A-Za-z0-9_-]", "-", name)
        self.main_transcript = os.path.join(self.project, self.session + ".jsonl")
        self.processes = []
        for directory in (self.work, self.runs, self.scratch, self.project):
            os.makedirs(directory, exist_ok=True)
        self.now = time.time()
        self.write(self.main_transcript, [self.user_text(self.now - 100, "开工")])

    def environment(self):
        return {"SINGLEFS_WORK_ROOT": self.work, "SINGLEFS_CRASH_AMPLIFICATION_HOME": self.home, SCRATCHPAD_ROOT_VARIABLE: self.scratch}

    @staticmethod
    def write(path, records, append=False):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a" if append else "w", encoding="utf-8") as handle:
            for record in records:
                handle.write(json.dumps(record, ensure_ascii=False) + "\n")

    @staticmethod
    def user_text(seconds, text):
        return {"type": "user", "timestamp": iso(seconds), "message": {"role": "user", "content": text}}

    @staticmethod
    def bash_call(call_id, started, ended, command, result="完成", background=False):
        tool_input = {"command": command}
        if background:
            tool_input["run_in_background"] = True
        records = [{"type": "assistant", "timestamp": iso(started),
                    "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call_id, "name": "Bash", "input": tool_input}]}}]
        if ended is not None:
            records.append({"type": "user", "timestamp": iso(ended),
                            "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": call_id, "content": result}]}})
        return records

    @staticmethod
    def notification(seconds, task_id, status="completed"):
        return {"type": "user", "timestamp": iso(seconds), "origin": {"kind": "task-notification"},
                "message": {"role": "user", "content": f"<task-notification>\n<task-id>{task_id}</task-id>\n<status>{status}</status>\n</task-notification>"}}

    def owned(self, name, session=None):
        path = os.path.join(self.work, name)
        os.makedirs(path, exist_ok=True)
        with open(os.path.join(path, ".owner"), "w", encoding="utf-8") as handle:
            handle.write(f"session: {session or self.session}\ncreated: 2000-01-01\npurpose: 自证\n")
        return path

    @staticmethod
    def build_directory(path):
        os.makedirs(os.path.join(path, "release", "deps"), exist_ok=True)
        with open(os.path.join(path, "CACHEDIR.TAG"), "w") as handle:
            handle.write("Signature: 8a477f597d28d172789f06886806bc55\n")
        with open(os.path.join(path, "release", "deps", "libx.rlib"), "wb") as handle:
            handle.write(b"x" * 4096)
        return path

    @staticmethod
    def repository_copy(path):
        os.makedirs(os.path.join(path, ".git"), exist_ok=True)
        os.makedirs(os.path.join(path, "crates", "core"), exist_ok=True)
        open(os.path.join(path, "Cargo.toml"), "w").close()
        return path

    @staticmethod
    def library_copy(path):
        os.makedirs(path, exist_ok=True)
        for name in ("CURRENT", "MANIFEST-000004", "000005.sst"):
            open(os.path.join(path, name), "w").close()
        return path

    def run_directory(self, pid, suffix="", owner_session=None):
        """运行目录根下一层：名字照入口脚本的「<前缀>-<时刻>-<进程号>[-<后缀>]」；给了 owner_session 的写归属记录。"""
        path = os.path.join(self.runs, f"crash-amplification-{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{pid}{suffix}")
        os.makedirs(path)
        if owner_session:
            with open(os.path.join(path, RUN_OWNER_FILE), "w", encoding="utf-8") as handle:
                handle.write(f"session: {owner_session}\ncreated: 2000-01-01\npurpose: 自证的运行目录\n")
        return path

    @staticmethod
    def dead_pid():
        """比 pid_max 大的进程号：一定不在。"""
        try:
            return int(open("/proc/sys/kernel/pid_max").read()) + 1
        except (OSError, ValueError):
            return 99999999

    def creator(self, session, launcher_command=None):
        """起运行目录的假入口：环境里带会话号与 CLAUDE_PID（本进程当 Claude 进程）。给了 launcher_command 的照 Bash 工具那样起
        bash -c 'true && eval <命令> && true'，命令用内建的 read 等在管道上（不起子进程，停它只停这一个进程号）。"""
        environment = {key: value for key, value in os.environ.items() if key not in SESSION_VARIABLES}
        environment.update({"CLAUDE_CODE_SESSION_ID": session, CLAUDE_PID_VARIABLE: str(os.getpid())})
        arguments = ["sleep", "120"] if launcher_command is None else ["bash", "-c", f"true && eval {shlex.quote(launcher_command)} && true"]
        process = subprocess.Popen(arguments, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=environment)
        self.processes.append(process)
        return process

    def end(self, process):
        process.kill()
        process.wait()
        self.processes.remove(process)

    def subagent(self, agent_id, prompt, records=()):
        path = os.path.join(self.project, self.session, "subagents", f"agent-{agent_id}.jsonl")
        self.write(path, [self.user_text(self.now - 90, prompt), *records])
        return path

    def hold(self, arguments, **options):
        process = subprocess.Popen(arguments, stdin=subprocess.DEVNULL, stderr=subprocess.DEVNULL, **options)
        self.processes.append(process)
        return process

    def background_task(self, task_id, command_arguments):
        """主会话记录里一条还没完成通知的后台任务，它的输出文件由一个活着的进程开着。"""
        output = os.path.join(self.root, "tasks", f"{task_id}.output")
        os.makedirs(os.path.dirname(output), exist_ok=True)
        handle = open(output, "w")
        process = self.hold(command_arguments, stdout=handle)
        handle.close()
        # 等它开着输出文件、exec 换好进程名（bash -c 'exec -a …' 那种，换好之前命令行还是 bash -c …）
        for _ in range(250):
            try:
                holding = os.path.realpath(os.readlink(f"/proc/{process.pid}/fd/1")) == os.path.realpath(output)
                renamed = command_arguments[0] != "bash" or not open(f"/proc/{process.pid}/cmdline", "rb").read().startswith(b"bash")
                if holding and renamed:
                    break
            except OSError:
                pass
            time.sleep(0.02)
        self.write(self.main_transcript, self.bash_call(f"toolu_{task_id}", self.now - 50, self.now - 49, "bash x.sh",
                                                        f"Command running in background with ID: {task_id}. Output is being written to: {output}",
                                                        background=True), append=True)

    def stop(self):
        for process in self.processes:
            process.kill()
            process.wait()
        self.processes = []

    def decide(self, hook_input):
        saved = {key: os.environ.get(key) for key in self.environment()}
        os.environ.update(self.environment())
        try:
            return decide(hook_input)
        finally:
            for key, value in saved.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value

    def stop_input(self, active=False):
        return {"hook_event_name": "Stop", "session_id": self.session, "transcript_path": self.main_transcript, "stop_hook_active": active}

    def subagent_stop_input(self, agent_id, path, active=False):
        return {"hook_event_name": "SubagentStop", "session_id": self.session, "transcript_path": self.main_transcript,
                "agent_id": agent_id, "agent_transcript_path": path, "stop_hook_active": active}

    def handback_input(self, agent_id):
        return {"hook_event_name": "PreToolUse", "tool_name": HANDBACK_TOOL, "session_id": self.session,
                "transcript_path": self.main_transcript, "agent_id": agent_id, "tool_input": {"message": "交回"}}


def selftest_cells():
    base = tempfile.mkdtemp(prefix="completion-cleanup-selftest-")
    failures, checked = [], [0]

    def check(cell, ok, detail):
        checked[0] += 1
        if not ok:
            failures.append(cell)
            print(f"✗ 自证 {cell}：{detail}")
            print(f"→ 怎么办：看 completion-cleanup-guard.sh 里 {cell} 对应的判定；{BREAK_VARIABLE} 设着的话这里本来就该红")

    def blocked_with(outcome, *paths):
        return outcome.get("action") == "block" and all(path in outcome.get("text", "") for path in paths)

    scenarios = []

    def scenario(name):
        made = Scenario(base, name)
        scenarios.append(made)
        return made

    try:
        # 主 agent：认领目录里留了编译目录
        s = scenario("main-build")
        target = s.build_directory(os.path.join(s.owned("w-build"), "copy", "target"))
        outcome = s.decide(s.stop_input())
        check("main-build-directory-left", blocked_with(outcome, target), f"留了编译目录 {target} 应当拦下并列出，实际 {outcome}")
        # 主 agent：认领目录里留了仓副本
        s = scenario("main-repository")
        copy = s.repository_copy(os.path.join(s.owned("w-copy"), "repository"))
        outcome = s.decide(s.stop_input())
        check("main-repository-copy-left", blocked_with(outcome, copy, f"rm -rf -- {copy}"), f"留了仓副本 {copy} 应当拦下、列出删法，实际 {outcome}")
        # 主 agent：归属记录是本会话的崩溃放量运行目录里留了库拷贝（起它的进程已经不在）
        s = scenario("main-run")
        library = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), "-two-hosts", owner_session=s.session), "library-for-peer"))
        outcome = s.decide(s.stop_input())
        check("main-run-library-copy-left", blocked_with(outcome, library), f"运行目录里留了库拷贝 {library} 应当拦下，实际 {outcome}")
        # 本会话的进程起的运行目录：起它的进程还在时整个不列（那一趟还在跑），看到一次就记账；进程退了照记账拦
        s = scenario("own-run")
        process = s.creator(s.session)
        run = s.run_directory(process.pid)
        library = s.library_copy(os.path.join(run, "library-for-peer"))
        target = s.build_directory(os.path.join(run, "target"))
        running = s.decide(s.stop_input())
        check("run-in-progress-not-listed", running.get("action") == "pass" and library not in running.get("text", ""),
              f"起运行目录的进程 {process.pid} 还在（那一趟还在跑），里面的 {library} 不该列，实际 {running}")
        s.end(process)
        outcome = s.decide(s.stop_input())
        check("own-run-blocked-after-creator-exits", blocked_with(outcome, library, target),
              f"本会话的进程起的运行目录（看到它还在跑时记过账），进程退了之后留下的 {library}、{target} 应当拦下，实际 {outcome}")
        # 别的会话的进程起的运行目录，创建时间落在本会话的 Bash 调用时间窗里：不拦（记账认的、归属记录认的各一份）
        s = scenario("foreign-session-run")
        s.write(s.main_transcript, s.bash_call("toolu_run", s.now - 30, s.now + 600, "bash scripts/crash/crash-amplification.sh --item x"), append=True)
        process = s.creator("s-somebody-else")
        seen = s.library_copy(os.path.join(s.run_directory(process.pid), "library-for-peer"))
        s.decide(s.stop_input())
        s.end(process)
        recorded = s.library_copy(os.path.join(s.run_directory(s.dead_pid(), "-two-hosts", owner_session="s-somebody-else"), "library-for-peer"))
        outcome = s.decide(s.stop_input())
        check("foreign-session-run-in-window-not-listed",
              outcome.get("action") == "pass" and not any(path in str(outcome) for path in (seen, recorded)),
              f"别的会话的进程起的运行目录（{seen}、{recorded}）时间窗与本会话重叠也不该拦、不该报，实际 {outcome}")
        # 认不出是谁的（进程不在、没记账、没归属记录）、建在本会话 Bash 调用的时间窗里：只在拦下时另起一节报，不进单子
        s = scenario("unknown-run")
        s.write(s.main_transcript, s.bash_call("toolu_run", s.now - 30, s.now + 600, "bash scripts/crash/crash-amplification.sh --item x"), append=True)
        unknown = s.library_copy(os.path.join(s.run_directory(s.dead_pid()), "library-for-peer"))
        alone = s.decide(s.stop_input())
        target = s.build_directory(os.path.join(s.owned("w-unknown"), "target"))
        outcome = s.decide(s.stop_input())
        text = outcome.get("text", "")
        check("unknown-run-reported-not-blocked",
              alone.get("action") == "pass" and blocked_with(outcome, target) and unknown not in outcome.get("paths", [])
              and UNKNOWN_SECTION_TITLE in text and text.index(UNKNOWN_SECTION_TITLE) < text.index(unknown),
              f"认不出是谁的 {unknown} 不该拦，只在拦下时列进「{UNKNOWN_SECTION_TITLE}」一节，实际单独时 {alone}，有别的要拦时 {outcome}")
        # 运行目录里深处有文件被开着（那一趟还在写日志），库拷贝本身没人开着：整个运行目录不列
        s = scenario("run-held-deep")
        run = s.run_directory(s.dead_pid(), owner_session=s.session)
        library = s.library_copy(os.path.join(run, "library-for-peer"))
        os.makedirs(os.path.join(run, "logs", "deeper"))
        with open(os.path.join(run, "logs", "deeper", "share.log"), "w") as handle:
            s.hold(["sleep", "120"], stdout=handle)
        outcome = s.decide(s.stop_input())
        check("run-held-deep-not-listed", outcome.get("action") == "pass" and library not in outcome.get("text", ""),
              f"运行目录 {run} 深处的日志有进程开着，里面的 {library} 不该列，实际 {outcome}")
        s.stop()
        # 只经内存映射开着（文件描述符已关）的编译目录不列（run_build_directory.py 的判法看 /proc/<pid>/maps）
        s = scenario("mapped")
        mapped = s.build_directory(os.path.join(s.owned("w-mapped"), "target"))
        mapped_file = os.path.join(mapped, "release", "deps", "libx.rlib")
        ready = os.path.join(s.root, "mapped.ready")
        s.hold([sys.executable, "-c", "import mmap, sys, time\nhandle = open(sys.argv[1], 'rb')\nmapping = mmap.mmap(handle.fileno(), 0, access=mmap.ACCESS_READ)\n"
                "handle.close()\nopen(sys.argv[2], 'w').close()\ntime.sleep(120)\n", mapped_file, ready], stdout=subprocess.DEVNULL)
        for _ in range(250):
            if os.path.exists(ready):
                break
            time.sleep(0.02)
        outcome = s.decide(s.stop_input())
        check("mapped-copy-not-listed", os.path.exists(ready) and outcome.get("action") == "pass",
              f"编译目录 {mapped} 里的文件只经内存映射开着也算在用，不该列，实际 {outcome}")
        s.stop()
        # 子 agent：只认它自己的 Bash 调用起的运行目录；同一会话别的调用（主 agent、兄弟子 agent）起的不算它的
        s = scenario("subagent-run")
        mine_command, sibling_command = ": crash-amplification mine; read -r _", ": crash-amplification sibling; read -r _"
        agent = s.subagent("arun", "派发", s.bash_call("toolu_mine", s.now - 20, s.now + 600, mine_command))
        mine_process, sibling_process = s.creator(s.session, mine_command), s.creator(s.session, sibling_command)
        mine = s.library_copy(os.path.join(s.run_directory(mine_process.pid), "library-for-peer"))
        sibling = s.library_copy(os.path.join(s.run_directory(sibling_process.pid), "library-for-peer"))
        s.decide(s.subagent_stop_input("arun", agent))
        s.end(mine_process)
        s.end(sibling_process)
        outcome = s.decide(s.subagent_stop_input("arun", agent))
        check("subagent-own-run-left", blocked_with(outcome, mine), f"子 agent 自己的 Bash 调用起的运行目录里的 {mine} 应当拦下，实际 {outcome}")
        check("subagent-ignores-sibling-runs", sibling not in outcome.get("text", ""), f"同一会话别的调用起的 {sibling} 不该算到这个子 agent 头上，实际 {outcome}")
        # 别的会话认领的目录不判
        s = scenario("other-session")
        s.build_directory(os.path.join(s.owned("w-theirs", session="s-somebody-else"), "target"))
        outcome = s.decide(s.stop_input())
        check("other-session-directory-ignored", outcome.get("action") == "pass", f"别的会话认领的目录不该判，实际 {outcome}")
        # 只剩交付物：报告、补丁、声明成备份的副本
        s = scenario("deliverables")
        directory = s.owned("w-deliver")
        open(os.path.join(directory, "result.md"), "w").close()
        os.makedirs(os.path.join(directory, "patch"))
        open(os.path.join(directory, "patch", "crates.patch"), "w").close()
        s.repository_copy(os.path.join(directory, "before-patch"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("# 交付物\nbefore-patch 备份 打补丁之前的原样\nbroken-line 留着\n")
        outcome = s.decide(s.stop_input())
        check("deliverables-only-passes", outcome.get("action") == "pass", f"只剩报告、补丁与声明了的备份应当放行，实际 {outcome}")
        # 声明成交付物的编译目录照列
        s = scenario("declared-build")
        directory = s.owned("w-declared")
        kept = s.build_directory(os.path.join(directory, "kept-target"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("kept-target 备份 想留着下一趟接着编\n")
        outcome = s.decide(s.stop_input())
        check("build-directory-never-deliverable", blocked_with(outcome, kept), f"声明成备份的编译目录 {kept} 照样要列，实际 {outcome}")
        # 交接副本：主 agent 照列，子 agent 交回时不列
        s = scenario("handover")
        directory = s.owned("w-handover")
        handed = s.repository_copy(os.path.join(directory, "copy"))
        with open(os.path.join(directory, DECLARATION_FILE), "w", encoding="utf-8") as handle:
            handle.write("copy 交接 主 agent 在里面跑轻型崩溃与变异\n")
        outcome = s.decide(s.stop_input())
        check("main-handover-copy-left", blocked_with(outcome, handed), f"主 agent 结束一轮时交接副本 {handed} 照列，实际 {outcome}")
        agent = s.subagent("ahandover", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                              "bash scripts/hosts/work-directory-ownership.sh claim w-handover --purpose 自证"))
        outcome = s.decide(s.subagent_stop_input("ahandover", agent))
        check("subagent-handover-passes", outcome.get("action") == "pass", f"子 agent 交回时交接副本不列，实际 {outcome}")
        # 有在跑的后台任务：放行
        s = scenario("background")
        s.build_directory(os.path.join(s.owned("w-busy"), "target"))
        s.background_task("bwork", ["sleep", "120"])
        outcome = s.decide(s.stop_input())
        check("background-running-passes", outcome.get("action") == "pass", f"手里有在跑的后台任务应当放行，实际 {outcome}")
        s.stop()
        # 后台只有循环提醒在等：照判
        s = scenario("reminder")
        target = s.build_directory(os.path.join(s.owned("w-remind"), "target"))
        s.background_task("bremind", ["bash", "-c", "exec -a remind.sh sleep 120"])
        outcome = s.decide(s.stop_input())
        check("reminder-only-still-judged", blocked_with(outcome, target), f"后台只有循环提醒在等不算在跑的活，应当照拦，实际 {outcome}")
        s.stop()
        # 有在跑的子 agent：放行；子 agent 收到完成通知之后：照判
        s = scenario("subagent-running")
        s.build_directory(os.path.join(s.owned("w-sub"), "target"))
        s.subagent("arunning", "派发", s.bash_call("toolu_pending", s.now - 5, None, "sleep 5"))
        outcome = s.decide(s.stop_input())
        check("subagent-running-passes", outcome.get("action") == "pass", f"手里有在跑的子 agent 应当放行，实际 {outcome}")
        s = scenario("subagent-finished")
        target = s.build_directory(os.path.join(s.owned("w-sub"), "target"))
        s.subagent("afinished", "派发")
        s.write(s.main_transcript, [s.notification(time.time() + 5, "afinished")], append=True)
        outcome = s.decide(s.stop_input())
        check("subagent-finished-still-judged", blocked_with(outcome, target), f"子 agent 已收到完成通知、不算在跑，应当照拦，实际 {outcome}")
        # 有进程在用的不列
        s = scenario("held")
        directory = s.owned("w-held")
        held = s.build_directory(os.path.join(directory, "held-target"))
        idle = s.build_directory(os.path.join(directory, "idle-target"))
        s.hold(["sleep", "120"], cwd=held)
        outcome = s.decide(s.stop_input())
        check("held-copy-not-listed", blocked_with(outcome, idle) and held not in outcome.get("text", ""),
              f"有进程在用的 {held} 不该列、没人用的 {idle} 照列，实际 {outcome}")
        s.stop()
        # 子 agent：只判自己认领的，主 agent 的别的目录不判
        s = scenario("subagent-scope")
        mine = s.build_directory(os.path.join(s.owned("sub-claimed"), "target"))
        sibling = s.build_directory(os.path.join(s.owned("main-own"), "target"))
        agent = s.subagent("ascope", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79,
                                                           "bash scripts/hosts/work-directory-ownership.sh claim sub-claimed --purpose '自证'"))
        outcome = s.decide(s.subagent_stop_input("ascope", agent))
        check("subagent-build-directory-left", blocked_with(outcome, mine), f"子 agent 自己认领的目录里的编译目录 {mine} 应当拦下，实际 {outcome}")
        check("subagent-ignores-siblings", sibling not in outcome.get("text", ""), f"主 agent 的别的目录 {sibling} 不该算到子 agent 头上，实际 {outcome}")
        # 子 agent：派发提示点名的草稿目录
        s = scenario("subagent-assigned")
        directory = s.owned("assigned")
        assigned_copy = s.repository_copy(os.path.join(directory, "copy"))
        agent = s.subagent("aassigned", f"# 派发\n草稿目录：{directory}\n报告：{directory}/result.md\n")
        outcome = s.decide(s.subagent_stop_input("aassigned", agent))
        check("subagent-assigned-directory-left", blocked_with(outcome, assigned_copy), f"派发提示点名的草稿目录里的仓副本 {assigned_copy} 应当拦下，实际 {outcome}")
        # 别的时间窗里建的运行目录不算这里的
        s = scenario("foreign-run")
        s.library_copy(os.path.join(s.runs, "crash-amplification-foreign", "library-for-peer"))
        s.write(s.main_transcript, [s.user_text(s.now - 7300, "早先")] + s.bash_call("toolu_old", s.now - 7200, s.now - 7100, "bash scripts/crash/crash-amplification.sh"), append=False)
        outcome = s.decide(s.stop_input())
        check("foreign-run-directory-ignored", outcome.get("action") == "pass", f"不在自己 Bash 调用时间窗里建的运行目录不该判，实际 {outcome}")
        # 续跑回来同一张单：放行交用户；单子变了：照拦
        s = scenario("release")
        directory = s.owned("w-release")
        target = s.build_directory(os.path.join(directory, "target"))
        first = s.decide(s.stop_input())
        second = s.decide(s.stop_input(active=True))
        check("unchanged-after-block-releases", first.get("action") == "block" and second.get("action") == "release" and target in second.get("text", ""),
              f"拦过一次、续跑回来一个没删应当放行并把单子交用户，实际 {first.get('action')} 之后 {second}")
        s.build_directory(os.path.join(directory, "another-target"))
        third = s.decide(s.stop_input(active=True))
        check("changed-after-block-blocks-again", third.get("action") == "block", f"续跑回来单子变了应当照拦，实际 {third}")
        # 单上写 du 的大小
        s = scenario("size")
        target = s.build_directory(os.path.join(s.owned("w-size"), "target"))
        outcome = s.decide(s.stop_input())
        check("block-lists-size", re.search(rf"  {KIND_BUILD}  {re.escape(target)}  [0-9.,]+[KMGTP]?\n", outcome.get("text", "")) is not None,
              f"单上每一行要写 du -sh 的大小，实际 {outcome}")
        # 主 agent：本会话的临时草稿目录（tasks/ 不看）
        s = scenario("scratchpad")
        pad = os.path.join(s.scratch, "project-directory", s.session)
        pad_copy = s.repository_copy(os.path.join(pad, "copy"))
        task_copy = s.build_directory(os.path.join(pad, "tasks", "target"))
        outcome = s.decide(s.stop_input())
        check("main-scratchpad-copy-left", blocked_with(outcome, pad_copy) and task_copy not in outcome.get("text", ""),
              f"本会话临时草稿目录里的仓副本 {pad_copy} 应当拦下、tasks/ 下的不看，实际 {outcome}")
        # 什么都没留：放行
        s = scenario("clean")
        open(os.path.join(s.owned("w-clean"), "result.md"), "w").close()
        outcome = s.decide(s.stop_input())
        check("nothing-left-passes", outcome.get("action") == "pass", f"只剩报告应当放行，实际 {outcome}")
        # 包装：起钩子本身（不带弄坏开关）
        clean_environment = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
        s = scenario("wrapper")
        target = s.build_directory(os.path.join(s.owned("w-wrap"), "target"))
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.stop_input()), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment()))
        try:
            printed = json.loads(completed.stdout)
        except ValueError:
            printed = {}
        check("wrapper-stop-json", completed.returncode == 0 and printed.get("decision") == "block" and target in printed.get("reason", ""),
              f"Stop 上拦下应当退 0、打 decision=block 的 JSON，实际退 {completed.returncode}，输出 {completed.stdout!r}{completed.stderr!r}")
        mine = s.build_directory(os.path.join(s.owned("w-wrap-sub"), "target"))
        s.subagent("awrap", "派发", s.bash_call("toolu_claim", s.now - 80, s.now - 79, "bash scripts/hosts/work-directory-ownership.sh claim w-wrap-sub --purpose x"))
        completed = subprocess.run(["bash", HOOK_PATH], input=json.dumps(s.handback_input("awrap")), capture_output=True, text=True,
                                   env=dict(clean_environment, **s.environment()))
        check("wrapper-handback-exit-2", completed.returncode == 2 and mine in completed.stderr and completed.stdout == "",
              f"交回之前拦下应当退 2、清单写 stderr，实际退 {completed.returncode}，stderr {completed.stderr!r}")
        state = True
        for raw in ("", "不是 JSON", "[]", json.dumps({}), json.dumps({"hook_event_name": "Stop", "session_id": "../x", "transcript_path": "/nowhere.jsonl"})):
            completed = subprocess.run(["bash", HOOK_PATH], input=raw, capture_output=True, text=True, env=dict(clean_environment, **s.environment()))
            state = state and completed.returncode == 0 and completed.stdout == ""
        check("bad-input-passes", state, "读不出的输入、没有会话号、会话号带路径符号都应当放行（退 0、标准输出为空）")
        # 平常跑设了弄坏开关：退 2、点名变量只在自证里认
        probe = "build-directory-unrecognized" + (",guard-off" if broken("guard-off") else "")
        completed = subprocess.run(["bash", HOOK_PATH, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                   env=dict(clean_environment, **{BREAK_VARIABLE: probe}))
        check("break-switch-refused-outside-selftest", completed.returncode == 2 and BREAK_VARIABLE in completed.stderr and "自证" in completed.stderr,
              f"平常跑设了 {BREAK_VARIABLE} 应当退 2 并点名它只在自证里认，实际退 {completed.returncode}，stderr {completed.stderr!r}")
    finally:
        for made in scenarios:
            made.stop()
        shutil.rmtree(base, ignore_errors=True)
    return failures, checked[0]


def selftest_cells_main():
    failures, checked = selftest_cells()
    if failures:
        print(f"✗ 自证：{len(failures)} 格红（共判了 {checked} 格）")
        print("→ 怎么办：照上面每一格的下一步改，再单独跑 completion-cleanup-guard.sh --selftest")
        return 1
    print(f"✓ 各格都对（判了 {checked} 格）")
    return 0


def run_switch(switch):
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", HOOK_PATH, "--selftest-cells"], capture_output=True, text=True, env=dict(clean, **{BREAK_VARIABLE: switch}), timeout=300)
    return completed.returncode, completed.stdout + completed.stderr


def selftest():
    if os.environ.get(BREAK_VARIABLE):
        failures, checked = selftest_cells()
        if not failures:
            print(f"✗ 自证：打开弄坏开关 {os.environ[BREAK_VARIABLE]} 之后 {checked} 格仍全绿")
            print("→ 怎么办：那个开关弄坏的那一处要真影响它点名的格；两边对不上就改开关或改格")
        return 1
    clean = {key: value for key, value in os.environ.items() if key != BREAK_VARIABLE}
    completed = subprocess.run(["bash", HOOK_PATH, "--selftest-cells"], capture_output=True, text=True, env=clean, timeout=300)
    if completed.returncode != 0:
        print("✗ 自证：不带弄坏开关时各格应当全绿，实际：")
        print(completed.stdout + completed.stderr)
        print("→ 怎么办：先让 --selftest-cells 不带弄坏开关跑绿，再回来跑 --selftest")
        return 1
    workers = max(1, min(8, os.cpu_count() or 1))
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        results = list(pool.map(lambda pair: (pair, run_switch(pair[0])), BREAK_TABLE))
    failed = 0
    if len(results) != len(BREAK_TABLE):
        print(f"✗ 自证：派出去 {len(BREAK_TABLE)} 个开关、收回 {len(results)} 个")
        print("→ 怎么办：整轮作废，重跑 --selftest")
        return 1
    for (switch, cell), (code, output) in results:
        if code == 0 or f"✗ 自证 {cell}：" not in output:
            failed += 1
            print(f"✗ 自证：打开弄坏开关 {switch} 之后应当红在「{cell}」格，实际退 {code}，输出：{output[-1500:]!r}")
            print(f"→ 怎么办：{switch} 弄坏的那一处要真影响「{cell}」格的判定；两边对不上就改开关或改格")
    if failed:
        print(f"✗ 自证：{len(BREAK_TABLE)} 个弄坏开关里 {failed} 个没红在点名的格")
        print("→ 怎么办：照上面逐个改，再跑 --selftest")
        return 1
    summary = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    print(f"✓ completion-cleanup-guard.sh 自证：{summary.lstrip('✓ ')}；{len(BREAK_TABLE)} 个弄坏开关各红在点名的格")
    return 0


def main():
    refuse_break_outside_selftest()
    if ARGUMENTS == ["--selftest"]:
        return selftest()
    if ARGUMENTS == ["--selftest-cells"]:
        return selftest_cells_main()
    return hook_main()


sys.exit(main())
PY
