# 完工清理闸 completion-cleanup-guard.sh 的 python 部分之一：弄坏开关表、认领与运行目录与仓副本与编译目录与库拷贝的扫法、任务现场的读法、声明（.deliverables）、后台任务与子 agent 的认法、decide 与 hook_main。
# 入口 completion-cleanup-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 COMPLETION_CLEANUP_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
from __main__ import UNJUDGED_SKIP_PREFIX

import concurrent.futures, fnmatch, glob, hashlib, importlib.machinery, importlib.util, io, json, os, re, shlex, shutil, sqlite3, subprocess, sys, tempfile, time, traceback, urllib.parse
from datetime import datetime, timezone

sys.dont_write_bytecode = True
HOOK_DIR = os.path.dirname(os.path.abspath(__file__))   # 部分文件与入口同目录（引导把 argv[1] 放在 sys.path 最前再 import 它们）；算键库认得 __file__、不认部分文件里的 sys.argv
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
    ("prompt-scratch-relative-by-cwd", "subagent-assigned-relative-by-repository-root"),
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
    ("unjudged-pass-silent", "truncated-scan-pass-reported"),
    ("unjudged-skip-silent", "unjudged-skip-recorded"),
    ("unreadable-directory-unnoted", "unreadable-directory-pass-reported"),
    ("work-root-unlisted-unnoted", "work-root-unlisted-pass-reported"),
    ("transcript-unreadable-unnoted", "transcript-unreadable-skip-reported"),
    ("guard-off", "break-switch-refused-outside-selftest"),
    ("guard-off-with-others", "guard-off-combined-refused-in-process"),
    ("run-root-default-unseen", "run-root-default-under-work-root"),
    ("data-root-leftovers-unseen", "data-root-family-library-blocked"),
    ("prune-state-directory-unseen", "prune-state-library-copy-blocked"),
    ("service-library-listed", "service-library-not-listed"),
    ("unit-file-ignored", "unit-file-state-directory-read"),
    ("old-library-unrecognized", "data-root-family-library-blocked"),
    ("leftover-unjudged-unnoted", "species-unreadable-pass-reported"),
    ("no-owner-directories-skipped", "no-owner-mentioned-copy-blocked"),
    ("owned-directories-leftover-scanned", "other-session-leftover-ignored"),
    ("registered-entries-listed", "registered-entry-not-listed"),
    ("mention-ancestors-ignored", "data-root-family-library-blocked"),
    ("cd-ignored", "relative-after-cd-attributed"),
    ("anchor-mentions-counted", "anchor-mentions-not-attributed"),
    ("unmentioned-counted-own", "no-owner-unmentioned-reported-not-blocked"),
    ("read-only-mentions-counted", "read-only-mentions-not-attributed"),
    ("leftover-mentions-session-level", "subagent-leftover-by-own-commands"),
    ("archives-unrecognized", "library-archive-blocked"),
    ("archive-any-name", "library-archive-blocked"),
    ("declaration-outside-accepted", "data-root-leftover-not-declarable"),
    ("hand-declaration-ignored", "subagent-hand-to-main-passes"),
    ("hand-reason-unchecked", "subagent-hand-short-reason-blocks"),
    ("hand-ancestor-accepted", "subagent-hand-ancestor-blocks"),
    ("hand-all-kinds", "subagent-hand-library-copy-blocks"),
    ("hand-exempt-for-main", "main-hand-declaration-ignored"),
    ("hand-way-unlisted", "subagent-block-names-hand-way"),
    ("hand-way-example-oversized", "subagent-hand-way-passes-handback-gates"),
    ("handback-declaration-unchecked", "handback-bad-declaration-blocked"),
    ("handback-parent-declaration-judged", "handback-parent-declaration-passes"),
    ("declaration-quotes-unsplit", "quoted-declaration-with-space-accepted"),
    ("declaration-symlink-skipped", "handback-symlinked-declaration-blocked"),
    ("kept-scenes-ignored", "main-kept-scene-left"),
    ("kept-scene-removal-by-rm", "main-kept-scene-left"),
    ("kept-scenes-keep-unchecked", "unkept-scene-not-listed"),
    ("kept-scenes-session-unchecked", "other-session-kept-scene-ignored"),
    ("kept-scenes-unfinished-listed", "running-kept-scene-not-listed"),
    ("subagent-kept-scenes-judged", "subagent-kept-scenes-not-listed"),
    ("kept-scenes-uncollected-unchecked", "uncollected-scene-left"),
    ("uncollected-scene-removal-by-clean", "uncollected-scene-left"),
    ("kept-scene-removal-uncollected-first", "main-kept-scene-left"),
    ("kept-scene-removal-wait-skipped", "main-kept-scene-left"),
    ("kept-scenes-keep-variable-unchecked", "kept-collected-scene-left"),
    ("kept-scene-attempts-by-session-column", "main-kept-scene-left"),
    ("merge-snapshots-unseen", "main-merge-snapshot-left"),
    ("snapshot-session-unchecked", "other-session-merge-snapshot-ignored"),
    ("snapshot-owner-unchecked", "merge-snapshot-without-owner-ignored"),
    ("snapshot-removed-listed", "merge-snapshot-removed-not-listed"),
    ("baseline-declaration-ignored", "merge-snapshot-declared-baseline-passes"),
    ("baseline-exempt-for-subagent", "subagent-baseline-declaration-listed"),
    ("subagent-snapshots-judged", "subagent-merge-snapshots-not-listed"),
    ("merge-snapshot-root-unlisted-unnoted", "merge-snapshot-root-unlisted-pass-reported"),
    ("swap-tree-always-listed", "swap-tree-in-use-not-listed"),
    ("nested-declaration-ignored", "nested-lower-handover-passes"),
    ("nested-declaration-overtrusted", "nested-undeclared-copy-blocks"),
    ("nested-declaration-problems-unnamed", "nested-lower-broken-line-named"),
    ("part-unreadable-passes", "bootstrap-part-unreadable-refused"),
    ("part-unreadable-always-refuses", "bootstrap-part-unreadable-stop-hook-active-released"),
]
BREAK_ITEMS = tuple(item for item, _ in BREAK_TABLE)
SELFTEST_ENTRIES = (["--selftest"], ["--selftest-cells"])
NOT_RUN = []   # 自证里别的闸不在、没喂成的格：成功句里报「本次未跑」
NAME_PATTERN = re.compile(r"^[A-Za-z0-9_-]+$")
HANDBACK_TOOL = "SubagentHandback"
SUBAGENT_IDLE_SECONDS = 1800
NOTIFICATION_SLACK_SECONDS = 30
WINDOW_SLACK_SECONDS = 1.0
SCAN_MAX_DEPTH = 6
RUN_SCAN_MAX_DEPTH = 3
SCAN_MAX_DIRECTORIES = 50000
SCAN_LIMIT_VARIABLE = "COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES"   # 只给自证造截断用：设成正整数时上限照它
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"
UNJUDGED_FINDING_PREFIX = "完工清理闸没判全、照常放行"
INPUT_PREFIX_BYTES = 200
DU_TIMEOUT_SECONDS = 30
DECLARATION_FILE = ".deliverables"
DELIVERABLE_KINDS = ("报告", "补丁", "备份", "交接", "基线")
HANDOVER_KIND = "交接"
BASELINE_KIND = "基线"   # 主 agent 自己留着、后面要在它上面跑命令或以它为基线建副本的树：只认主 agent 的声明（文件头「要留的交付物」）
MERGE_SNAPSHOT_DIRECTORY_NAME = "merge-and-verify"   # 合入后验证的日志目录根：<工作根>/merge-and-verify/<一层>/（scripts/gate/merge-and-verify.sh）
MERGE_SNAPSHOT_NAME = "repo"   # 留下的快照在每一层里的名字
SWAP_TREE_PARTS = ("hook-swap", "tree")   # scripts/gate/hook-swap.py 在 --scratch 点的草稿目录下建的草稿树：<草稿目录>/hook-swap/tree，那一趟自己跑完或出错就删
REASON_MINIMUM_CHARACTERS = 4
# 子 agent 删不掉的副本交给主 agent 删：交回正文里的一行「交主 agent 删 <全路径>：<原因>」（文件头「删不掉的」）
HAND_TO_MAIN_PHRASE = "交主 agent 删"
HAND_TO_MAIN_REASON_MINIMUM = 8
HAND_TO_MAIN_LINE = re.compile(r"^[ \t>*•-]*" + HAND_TO_MAIN_PHRASE + r"[ \t]*[`'\"]?(/[^\s`'\"：:]+)[`'\"]?[ \t]*[：:][ \t]*(.+?)[ \t]*$", re.MULTILINE)
HAND_TO_MAIN_EXAMPLE_REASON = "rm 被权限拒，删不掉"
HAND_TO_MAIN_OVERSIZED_PADDING = 4200   # 弄坏开关 hand-way-example-oversized：例子行的原因后面接这么多个字，越过交回正文闸的字数上限
RUN_COMMAND_KEYWORDS = re.compile(r"crash-amplification|crash-point-replay|broad-mutation|mutation-shard-run|qemu-device-streams|lkmm\.sh|crates-mutation-replay"
                                  r"|formal-verification|research-build-and-replay|model-differential-and-scenarios|gate-staged|gate\.sh")
TIMER_AND_WATCHDOG_SCRIPTS = ("remind.sh", "watch.sh", "agent-watch.py")
LAUNCHER_WORDS = ("bash", "sh", "python3", "python", "nice", "env", "exec")
CLAIM_PATTERN = re.compile(r"work-directory-ownership\.sh['\"]?\s+claim\s+(['\"]?)([^\s'\"]+)\1")
STATE_DIRECTORY_NAME = "completion-cleanup-guard"
SCRATCHPAD_ROOT_VARIABLE = "COMPLETION_CLEANUP_SCRATCHPAD_ROOT"
KIND_BUILD, KIND_WORKTREE, KIND_REPOSITORY, KIND_LIBRARY = "编译目录", "工作树", "仓副本", "库拷贝"
KIND_MERGE_SNAPSHOT = "合入后验证快照"   # 合入后验证带 MERGE_AND_VERIFY_KEEP_COPY=1 留下的 <一层>/repo
KIND_KEPT_SCENE = "任务现场"   # 已有结局、结局没取走或提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的任务留下的现场目录：结局没取走的经 task-server/task wait <号> 取走，设了留现场的经 task clean <号> 删
KEEP_SCENE_VARIABLE = "SINGLEFS_KEEP_TEST_TEMPORARY_FILES"   # 提交时设成 KEEP_SCENE_VALUE 的任务，现场只由 task clean 删（task-server/task_server_lib.py 文件头「现场」）
KEEP_SCENE_VALUE = "1"
TASK_SERVER_DATABASE_VARIABLE = "COMPLETION_CLEANUP_TASK_SERVER_DATABASE"   # 只给自证造样本库用：设了就读它，平常不设
TASK_SERVER_CLIENT = "task-server/task"
TASK_SERVER_FINISHED_STATE = "finished"
KEPT_SCENE_TASKS = {}   # 这一次判出的任务现场 → (删它用的任务号（分片的子任务记父任务号）, 设没设留现场, 结局没取走)
RUN_NAME_PATTERN = re.compile(r"(?:^|-)([0-9]{8}T[0-9]{6}Z)-([0-9]+)(?=-|$)")
SESSION_VARIABLES = ("WORK_DIRECTORY_OWNER_SESSION", "CLAUDE_CODE_SESSION_ID", "CLAUDE_SESSION_ID")
CLAUDE_PID_VARIABLE = "CLAUDE_PID"
CREATOR_START_SLACK_SECONDS = 1.0
ANCESTRY_MAX_DEPTH = 64
RUN_LEDGER_NAME = "run-owners"
RUN_OWNER_FILE = ".owner"
CRASH_AMPLIFICATION_HOME_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_HOME"
CRASH_AMPLIFICATION_DIRECTORY_NAME = "crash-amplification"   # 没设上面那个变量时，工作根下的这一层是崩溃放量的家目录
UNKNOWN_SECTION_TITLE = "认不出是谁的"
UNKNOWN_SHOWN_MAX = 30   # 「认不出是谁的」一节最多列几处（每处要核一遍在不在用），其余只报个数
KIND_OLD_LIBRARY, KIND_LIBRARY_ARCHIVE = "旧文件库", "库备份"
HAND_TO_MAIN_KINDS = (KIND_BUILD, KIND_WORKTREE, KIND_REPOSITORY)   # 「交主 agent 删」只认这三种
LEFTOVER_KINDS = frozenset((KIND_LIBRARY, KIND_OLD_LIBRARY, KIND_LIBRARY_ARCHIVE))   # 迁库遗留那几处只认这三样
LEFTOVER_SCAN_MAX_DEPTH = 4
PRUNE_STATE_SCAN_MAX_DEPTH = 2
ARCHIVE_SUFFIXES = (".tar", ".tar.gz", ".tgz", ".tar.zst")
ARCHIVE_WORDS = ("verdict", "library")
SPECIES_TOKEN = r"[a-z][a-z0-9]*(?:-[a-z0-9]+)+"   # 种类名的样子：小写、带连字符的目录名
# 种类名现读的两处（文件头「判什么」旧文件库那一段）；自证换成样本文件
SPECIES_SOURCES = {
    "client": os.path.join(REPOSITORY_ROOT, "scripts", "prune-server", "prune_server_client.py"),
    "registry": os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "verdict-library-scripts.tsv"),
}
CLIENT_KEY_SPACE_HEADING = "键空间"
GLOB_CHARACTERS = re.compile(r"[*?\[]")
VARIABLE_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$")
VARIABLE_REFERENCE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
WORK_ROOT_SUBSTITUTION = re.compile(r"\$\(\s*python3\s+[^\s()]*lib_work_root\.py(?:\s+(data-root|shared\s+([A-Za-z0-9._-]+)))?\s*\)")
SHELL_PUNCTUATION = "();<>|&\n"   # 换行也切出来：命令按段判只读，换行隔开的是两段
LOOSE_WORDS = re.compile(r"([;&|()\n]+)|[ \t\r\f\v<>`'\"]+")   # 拆不开时的切法：段的分隔留作一个词，别的符号丢掉
SEGMENT_SEPARATOR = re.compile(r"[;&|()\n]+")
DIRECTORY_CHANGERS = ("cd", "pushd")
# 命令词是这几样的那一段里的路径不算点到（看一眼的不算本会话的）
READ_ONLY_COMMANDS = frozenset(("ls", "du", "stat", "find", "cat", "head", "tail", "less", "grep", "rg", "wc", "sha256sum", "md5sum",
                                "diff", "cmp", "file", "readlink", "realpath", "tree", "df"))
FIND_MODIFYING_PREDICATES = frozenset(("-delete", "-exec", "-execdir"))   # find 带这几样会删、会起别的命令，不算只读


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


GUARD_OFF_ITEM = "guard-off"                         # 关守卫项：只在它是唯一一项时认
GUARD_OFF_LENIENT_ITEM = "guard-off-with-others"     # 弄坏开关：自证进程本身里调守卫时，关守卫项与别的项同写也照认


def refuse_break_outside_selftest(value=None, arguments=None):
    """弄坏开关只在自证里认：平常跑设了就退 2，点名变量；关守卫项 guard-off 只在它是唯一一项时认，与别的项同写照拒并点名它（records/2026-10-23-subagent拆分提案.md
    第一百二十二节 ②）；不认识的项名一样拒。value、arguments 不给时取环境里的开关与这一次的参数；自证在进程里给值探同写那一格，
    guard-off-with-others 只在自证进程本身（这一次的参数是自证入口）里认，平常跑同写着它照拒。"""
    value = os.environ.get(BREAK_VARIABLE, "") if value is None else value
    arguments = ARGUMENTS if arguments is None else arguments
    if not value:
        return
    items = [item for item in value.split(",") if item]
    if arguments not in SELFTEST_ENTRIES:
        others = [item for item in items if item != GUARD_OFF_ITEM]
        if GUARD_OFF_ITEM in items and others and not (ARGUMENTS in SELFTEST_ENTRIES and broken(GUARD_OFF_LENIENT_ITEM)):
            print(f"✗ 完工清理闸：{BREAK_VARIABLE}={value}：关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；"
                  "弄坏开关只在自证（--selftest）里认，这一次没判", file=sys.stderr)
            print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看某一项红在哪一格，跑 {BREAK_VARIABLE}=<项> bash {HOOK_PATH} --selftest，一次只设那一项", file=sys.stderr)
            sys.exit(2)
        if GUARD_OFF_ITEM not in items:
            print(f"✗ 完工清理闸：{BREAK_VARIABLE}={value} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
            print(f"→ 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 bash {HOOK_PATH} --selftest", file=sys.stderr)
            sys.exit(2)
    unknown = [item for item in items if item not in BREAK_ITEMS]
    if unknown:
        print(f"✗ 完工清理闸：{BREAK_VARIABLE} 里的 {'、'.join(unknown)} 不认识（只认 {'、'.join(BREAK_ITEMS)}）", file=sys.stderr)
        print(f"→ 怎么办：项名照 {HOOK_PATH} 文件头「弄坏开关」一段写", file=sys.stderr)
        sys.exit(2)


MODULE_PATHS = {
    "lib_work_root": os.path.join(HOOK_DIR, "lib_work_root.py"),
    # 「草稿目录：」一行怎么认（只认行首那一行，与派发闸 ⑯ ⑰、交回闸 ⑥ 共用一份；判决 research/prompts/landing-audit-r4-main-verification.md K2-4）
    "lib_prompt_scratch_line": os.path.join(HOOK_DIR, "lib_prompt_scratch_line.py"),
    "agent_watch": os.path.join(REPOSITORY_ROOT, "scripts", "agents", "agent-watch.py"),
    "session_transcript": os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", "session-transcript.py"),
    "run_build_directory": os.path.join(REPOSITORY_ROOT, "scripts", "hosts", "run_build_directory.py"),
    # 判定库服务的状态目录与服务自己那份库在哪：只用它的 Paths、unit_library 与 LIBRARY_DIRECTORY_NAME，不起服务（文件没有 .py 后缀，照源码载入）
    "prune_server_control": os.path.join(REPOSITORY_ROOT, "prune-server", "control"),
    # 服务库在哪：只用它的 read_configuration、default_values 与 DATABASE_NAME，库照 sqlite 只读模式自己打开（不经它的 Store，不写）
    "task_server_lib": os.path.join(REPOSITORY_ROOT, "task-server", "task_server_lib.py"),
}
LOADED_MODULES = {}


def module(name):
    if name not in LOADED_MODULES:
        path = MODULE_PATHS[name]
        loader = None if path.endswith(".py") else importlib.machinery.SourceFileLoader(name, path)
        specification = importlib.util.spec_from_file_location(name, path, loader=loader)
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
    return os.environ.get(SCRATCHPAD_ROOT_VARIABLE) or module("lib_work_root").claude_session_temporary_root()


def main_roots(session, main_transcript, notes=None):
    """主 agent 判的目录：[(目录, 它的认领目录或 None, 下一层不看的名字)]。工作根在而列不出（读不了）时本会话认领的目录一个都没判，
    给了 notes 的往里记一句（照「放行但必记检出」；弄坏开关 work-root-unlisted-unnoted 下不记）；工作根不在的照旧什么都不记。"""
    roots = []
    base = work_root()
    try:
        names = sorted(os.listdir(base))
    except FileNotFoundError:
        names = []
    except OSError as error:
        names = []
        if notes is not None and not broken("work-root-unlisted-unnoted"):
            notes.append(f"工作根 {base} 列不出（{type(error).__name__}: {error}），本会话认领的目录一个都没判；出路：ls -ld {base} 看权限，修好再收工")
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
        reading = module("lib_prompt_scratch_line")
        for value in reading.scratch_line_values(first_user_text(agent_transcript)):
            written = module("lib_work_root").expand_written_path(value.rstrip("/"))
            if not os.path.isabs(written) and not broken("prompt-scratch-relative-by-cwd"):   # 写相对路径的按仓根解（与那份库的 scratch_directories 同一种解法），不按钩子此刻的当前目录
                written = os.path.join(reading.repository_root(), written)
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
def declaration_parts(text):
    """一行声明 → [路径, 种类, 理由]（不足三段时有几段给几段）。路径用反引号、单引号或双引号包着的，先按引号切出路径（里面可以有空格），
    引号后面要紧跟空白或到行尾，再按空白切种类与理由；没包着的整行按空白切三段。弄坏开关 declaration-quotes-unsplit 下不认引号。"""
    if text[:1] in ("`", "'", '"') and not broken("declaration-quotes-unsplit"):
        close = text.find(text[0], 1)
        if close > 1 and (close + 1 == len(text) or text[close + 1].isspace()):
            return [text[1:close], *text[close + 1:].split(None, 1)]
    return text.split(None, 2)


def declarations(directories, role):
    """读这几个目录里的 .deliverables：交 ({声明了的绝对路径: 种类}, [写坏的行])；只收这个角色免列的种类
    （主 agent 免列报告、补丁、备份、基线，子 agent 免列报告、补丁、备份、交接；弄坏开关 baseline-declaration-ignored 下主 agent 不认基线，
    baseline-exempt-for-subagent 下子 agent 也认基线）。
    路径落到那个目录外的行不算（数据根层的旧库靠声明放不过）；弄坏开关 declaration-outside-accepted 下照收。
    路径带引号的先按引号切出路径再切种类与理由（弄坏开关 declaration-quotes-unsplit 下整行按空白切三段）；
    .deliverables 是符号链接的判写法不合、点名它（弄坏开关 declaration-symlink-skipped 下整份跳过）。"""
    exempt_kinds = set() if broken("deliverables-ignored") else {"报告", "补丁", "备份"}
    if exempt_kinds and (role == "subagent" or broken("handover-exempt-for-main")):
        exempt_kinds.add(HANDOVER_KIND)
    if exempt_kinds and (broken("baseline-exempt-for-subagent") or (role == "main" and not broken("baseline-declaration-ignored"))):
        exempt_kinds.add(BASELINE_KIND)
    exempt, problems = {}, []
    for directory in dict.fromkeys(directory for directory in directories if directory):
        path = os.path.join(directory, DECLARATION_FILE)
        if os.path.islink(path) and not broken("declaration-symlink-skipped"):
            problems.append(f"{path} 是符号链接（指向 {os.readlink(path)}），不读，里面的声明不算（要写成普通文件）")
            continue
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
            parts = declaration_parts(text)
            if len(parts) < 3 or parts[1] not in DELIVERABLE_KINDS or len(parts[2].strip()) < REASON_MINIMUM_CHARACTERS:
                problems.append(f"{path}:{number} 不是「<相对路径> <种类> <为什么留>」（种类：{'、'.join(DELIVERABLE_KINDS)}；理由至少 {REASON_MINIMUM_CHARACTERS} 个字），这一行不算")
                continue
            relative = parts[0].strip("`'\"").rstrip("/")
            target = os.path.normpath(os.path.join(directory, relative))
            if (os.path.isabs(relative) or not is_strictly_under(target, directory)) and not broken("declaration-outside-accepted"):
                problems.append(f"{path}:{number} 的路径 {relative} 不在 {directory} 里面，这一行不算")
                continue
            if parts[1] in exempt_kinds:
                exempt[target] = parts[1]
    return exempt, problems


# ── 认副本
def species_in_name(name, species):
    """名字里带的那个种类名（整个名字，或前后是 . - _ 与名字两头）；没有交 None。"""
    for candidate in species:
        if re.search(rf"(?:^|[._-]){re.escape(candidate)}(?:$|[._-])", name):
            return candidate
    return None


def is_library_archive(name, species):
    """库备份压缩包（文件头「判什么」库备份那一段）。弄坏开关 archives-unrecognized 下一个都不认，archive-any-name 下压缩包都认。"""
    if broken("archives-unrecognized"):
        return False
    lowered = name.lower()
    if not lowered.endswith(ARCHIVE_SUFFIXES):
        return False
    if broken("archive-any-name"):
        return True
    return any(word in lowered for word in ARCHIVE_WORDS) or species_in_name(lowered, species) is not None


def kind_of(path, names, species=(), by_name=False):
    """这个目录是什么（文件头「判什么」）；by_name 为真时（不是起头那一处）另按种类名认旧文件库。"""
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
    if (kind is None and by_name and names and not broken("old-library-unrecognized")
            and species_in_name(os.path.basename(path), species) is not None):
        kind = KIND_OLD_LIBRARY
    return kind


def scan_limit():
    """扫多少个目录到顶：COMPLETION_CLEANUP_SCAN_MAX_DIRECTORIES 设成正整数的照它（只给自证造截断用），别的取 SCAN_MAX_DIRECTORIES。"""
    text = os.environ.get(SCAN_LIMIT_VARIABLE, "")
    return int(text) if text.isdigit() and int(text) > 0 else SCAN_MAX_DIRECTORIES


def scan(root, max_depth, exempt, skipped_names, budget, wanted=None, skip=None, nested_role=None):
    """从 root 往下找副本：交 [(路径, 种类)]；认出一样就不再往里找，声明了的那一棵只找编译目录。
    wanted 给了只交这几种（认出别的种类照样不再往里找）；skip(路径, 层数) 为真的下层目录不进。种类名取 budget["species"]，
    budget["not_by_name"] 里的目录不按名字认旧文件库（数据根表里登记的在用项）。
    nested_role 给了（子 agent 判自己的认领目录与草稿目录时给它的角色）：root 底下不是副本的下层目录里的 DECLARATION_FILE 也读，
    只取这个角色免列的声明（嵌套派发里下层子 agent 在它那一层声明了的副本）；写坏的行不判。不改调用方的 exempt。"""
    if nested_role is not None:
        exempt = dict(exempt)
    found = []
    species = budget.get("species", ())
    stack = [(root, 0, False)]
    while stack:
        path, depth, inside_exempt = stack.pop()
        if budget["visited"] >= budget["limit"]:
            budget["truncated"].add(root)
            break
        budget["visited"] += 1
        if is_project_root_or_above(path):
            continue
        try:
            entries = list(os.scandir(path))
        except OSError as error:
            if not broken("unreadable-directory-unnoted"):   # 读不了的目录里有没有副本没判：记下来，放行时照「放行但必记检出」报（静默跳过）
                budget.setdefault("unreadable", []).append(f"{path}（{type(error).__name__}: {error.strerror or error}）")
            continue
        here_exempt = inside_exempt or path in exempt
        names = {entry.name for entry in entries}
        kind = kind_of(path, names, species, by_name=depth > 0 and path not in budget.get("not_by_name", ()))
        if nested_role is not None and depth > 0 and kind is None and DECLARATION_FILE in names:
            exempt.update(declarations([path], nested_role)[0])
            if broken("nested-declaration-overtrusted"):   # 下层那一份在就把这一层整棵算已声明，不看它声明了哪几行
                here_exempt = True
        if kind is not None:
            if wanted is not None and kind not in wanted:
                continue
            if kind == KIND_BUILD:
                if not (here_exempt and broken("build-directory-declarable")):
                    found.append((path, kind))
                continue
            if not here_exempt:
                found.append((path, kind))
                continue
        for entry in entries:   # 库备份是文件：这一层的文件逐个认，声明了的不列
            if wanted is not None and KIND_LIBRARY_ARCHIVE not in wanted:
                break
            try:
                is_file = entry.is_file(follow_symlinks=False)
            except OSError:
                continue
            if is_file and is_library_archive(entry.name, species) and not (here_exempt or entry.path in exempt):
                found.append((entry.path, KIND_LIBRARY_ARCHIVE))
        if depth >= max_depth:
            continue
        for entry in sorted(entries, key=lambda item: item.name, reverse=True):
            if depth == 0 and entry.name in skipped_names:
                continue
            try:
                is_directory = entry.is_dir(follow_symlinks=False)
            except OSError:
                continue
            if is_directory and not (skip is not None and skip(entry.path, depth + 1)):
                stack.append((entry.path, depth + 1, here_exempt))
    return found


# ── 运行目录
def run_roots():
    """崩溃放量的运行目录根：SINGLEFS_CRASH_AMPLIFICATION_HOME 设了是它下面的 runs，没设是工作根下 crash-amplification/runs
    （同 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_home_directory）。弄坏开关 run-root-default-unseen 下没设时交空表。"""
    home = os.environ.get(CRASH_AMPLIFICATION_HOME_VARIABLE)
    if home:
        return [os.path.join(home, "runs")]
    if broken("run-root-default-unseen"):
        return []
    return [os.path.join(work_root(), CRASH_AMPLIFICATION_DIRECTORY_NAME, "runs")]


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
        notes.append(f"取不到创建时间的运行目录 {len(missing)} 个没判（认不出起它的进程）：{' '.join(missing[:5])}{' …' if len(missing) > 5 else ''}；"
                     "出路：逐个看是不是你起的那一趟，是的用完就删（rm -rf -- <路径>），不是的别动")
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


# ── 迁库遗留（文件头「判哪几处」迁库遗留那一段）
def client_species(path):
    """prune_server_client.py 文件头「键空间」一节里「例 / 种类固定 / 种类是」后面的名字（一节从行首是「键空间」的那一行起，到下一行顶格的为止）。"""
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().splitlines()
    section, inside = [], False
    for line in lines:
        if line.startswith(CLIENT_KEY_SPACE_HEADING):
            inside = True
        elif inside and line and not line[0].isspace():
            break
        if inside:
            section.append(line)
    names = []
    for match in re.finditer(rf"(?:例|种类固定|种类是)\s*({SPECIES_TOKEN}(?:/[^\s、，）]*)?(?:\s*、\s*{SPECIES_TOKEN})*)", "\n".join(section)):
        names += [part.split("/")[0] for part in re.split(r"\s*、\s*", match.group(1))]
    return names


def registry_species(path):
    """verdict-library-scripts.tsv 第三列里「键空间 <族>/<种类>/」与「<族> 族 <种类>/」的种类。"""
    names = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if line.startswith("#") or len(columns) < 3:
                continue
            for match in re.finditer(rf"键空间 {SPECIES_TOKEN}/({SPECIES_TOKEN})/|{SPECIES_TOKEN} 族 ({SPECIES_TOKEN})/", columns[2]):
                names.append(match.group(1) or match.group(2))
    return names


def library_species(notes):
    """旧文件库与库备份认的种类名：SPECIES_SOURCES 两处现读、合起来去重。哪一处读不了或一个都读不出，往 notes 记一处没判全
    （弄坏开关 leftover-unjudged-unnoted 下不记）。"""
    readers = (("client", client_species, f"文件头「{CLIENT_KEY_SPACE_HEADING}」一节"), ("registry", registry_species, "第三列的键空间"))
    names = []
    for key, reader, where in readers:
        path = SPECIES_SOURCES[key]
        try:
            found, reason = reader(path), "一个种类名都没读出"
        except (OSError, UnicodeDecodeError) as error:
            found, reason = [], f"读不了（{type(error).__name__}: {error}）"
        if not found and not broken("leftover-unjudged-unnoted"):
            notes.append(f"种类名读不出：{path} {where}{reason}，旧文件库与带种类名的库备份认不全；出路：把那份文件放回仓里、照它原来的写法写，再收工")
        names += found
    return tuple(dict.fromkeys(names))


def prune_service_directories(notes):
    """判定库服务的状态目录与服务自己那份库：交 ([状态目录…], 服务那份库或 None)。取法照 prune-server/control：单元文件的 --state-directory
    （unit_library），读不到单元文件时取它的缺省状态目录；两处不同的都交。control 载不进、单元文件在而读不了往 notes 记一处没判全；
    弄坏开关 unit-file-ignored 下不读单元文件。"""
    noting = not broken("leftover-unjudged-unnoted")
    try:
        control = module("prune_server_control")
        paths = control.Paths()
    except Exception as error:  # noqa: BLE001 —— 载不进就记一处没判全，别的照判
        if noting:
            notes.append(f"读不到 {MODULE_PATHS['prune_server_control']}（{error!r}），判定库服务状态目录里的迁库遗留没判；出路：把 prune-server/ 放回仓里再收工")
        return [], None
    library = None
    if not broken("unit-file-ignored"):
        library, origin = control.unit_library(paths)
        if library is None and os.path.lexists(paths.unit_file) and noting:
            notes.append(f"判定库服务的单元文件读不了（{origin}），只看了缺省的状态目录 {paths.state_directory}；出路：ls -l {paths.unit_file} 看权限，修好再收工")
    if library is None:
        library = os.path.join(paths.state_directory, control.LIBRARY_DIRECTORY_NAME)
    library = os.path.normpath(library)
    return list(dict.fromkeys([os.path.dirname(library), os.path.normpath(paths.state_directory)])), library


def with_ancestors(paths):
    every = set()
    for path in paths:
        current = os.path.normpath(path)
        while True:
            every.add(current)
            parent = os.path.dirname(current)
            if parent == current:
                break
            current = parent
    return every


def leftover_places(judged_roots, notes):
    """迁库遗留扫哪几处：交 ([(根, 最深几层, 是不是工作根)], 不进的目录, 服务那份库或 None, 锚点与它们的上层, 数据根或 None)。
    弄坏开关 data-root-leftovers-unseen 下不看数据根与崩溃放量的家目录，prune-state-directory-unseen 下不看判定库服务的状态目录，
    no-owner-directories-skipped 下不看工作根下没写归属文件的目录，service-library-listed 下服务那份库照进。"""
    library = module("lib_work_root")
    base = os.path.normpath(work_root())
    runs = [os.path.normpath(path) for path in run_roots()]
    home = os.path.normpath(os.environ.get(CRASH_AMPLIFICATION_HOME_VARIABLE) or os.path.join(base, CRASH_AMPLIFICATION_DIRECTORY_NAME))
    places, anchors, data = [], [library.home_directory(), base, home, *runs], None
    if not broken("data-root-leftovers-unseen"):
        try:
            data = os.path.normpath(library.shared_data_root())
        except ValueError as error:
            if not broken("leftover-unjudged-unnoted"):
                notes.append(f"数据根取不到（{error}），数据根下的迁库遗留没判；出路：{library.DATA_ROOT_HOWTO}")
        if data is not None:
            places.append((data, LEFTOVER_SCAN_MAX_DEPTH, False))
            anchors.append(data)
        if not any(home == root or is_strictly_under(home, root) for root in (data, base) if root):
            places.append((home, LEFTOVER_SCAN_MAX_DEPTH, False))
    state_directories, service = prune_service_directories(notes)
    anchors += state_directories
    if not broken("prune-state-directory-unseen"):
        places += [(directory, PRUNE_STATE_SCAN_MAX_DEPTH, False) for directory in state_directories]
    if not broken("no-owner-directories-skipped"):
        places.append((base, LEFTOVER_SCAN_MAX_DEPTH, True))
    skipped = {base, *runs, *(root for root, _, _ in places), *(os.path.normpath(root[0]) for root in judged_roots)}
    if service is not None and not broken("service-library-listed"):
        skipped.add(service)
    return places, skipped, service, with_ancestors(anchors), data


def substituted_work_roots(command):
    """$(python3 …/lib_work_root.py [data-root | shared <名>]) 换成它打的那一处；取不到的原样留着（后面认不出、不算点到）。"""
    library = module("lib_work_root")

    def replace(match):
        try:
            if match.group(1) is None:
                return library.work_root()
            if match.group(2):
                return library.shared_data_path(match.group(2))
            return library.shared_data_root()
        except ValueError:
            return match.group(0)
    return WORK_ROOT_SUBSTITUTION.sub(replace, command)


def command_words(command):
    """一条 Bash 命令拆成词：shlex（带 shell 符号）拆得开照它，拆不开（引号不成对、here-doc 里的散字）照空白与 shell 符号切。
    段的分隔（SEGMENT_SEPARATOR：| || && ; & 括号与换行）两种拆法都留作单独的词。"""
    try:
        lexer = shlex.shlex(command, posix=True, punctuation_chars=SHELL_PUNCTUATION)
        lexer.whitespace = " \t\r\f\v"
        lexer.whitespace_split = True
        return list(lexer)
    except ValueError:
        return [word for word in LOOSE_WORDS.split(command) if word]


def read_only_flags(words):
    """每个词在不在只读的那一段里：段照 SEGMENT_SEPARATOR 切，命令词是起头的 NAME=值 之后那个词的最后一节文件名，
    在 READ_ONLY_COMMANDS 里（find 另要不带 FIND_MODIFYING_PREDICATES）就是只读。弄坏开关 read-only-mentions-counted 下一段都不算只读。"""
    flags, start = [False] * len(words), 0
    for end in [*(index for index, word in enumerate(words) if SEGMENT_SEPARATOR.fullmatch(word)), len(words)]:
        segment = words[start:end]
        command = next((word for word in segment if not VARIABLE_ASSIGNMENT.match(word)), None)
        name = os.path.basename(command) if command is not None else None
        if (name in READ_ONLY_COMMANDS and not (name == "find" and FIND_MODIFYING_PREDICATES.intersection(segment))
                and not broken("read-only-mentions-counted")):
            flags[start:end] = [True] * len(segment)
        start = end + 1
    return flags


def resolved_word(word, variables, directory):
    """一个词 → 它点的绝对路径（normpath）；认不出交 None。相对路径照 directory（同一条命令里前面 cd 到的）解，弄坏开关 cd-ignored 下不解。"""
    text = VARIABLE_REFERENCE.sub(lambda match: variables.get(match.group(1) or match.group(2), match.group(0)), word)
    text = module("lib_work_root").expand_written_path(text)
    if not text or "$" in text:
        return None
    if not os.path.isabs(text):
        if directory is None or broken("cd-ignored"):
            return None
        text = os.path.join(directory, text)
    return os.path.normpath(text)


def mentioned_paths(calls, names=None):
    """自己的 Bash 命令点到的路径：(点到的绝对路径集合, 带通配的列表)。names 给了只拆文字里带其中一个名字的命令（别的命令点不到要找的那几处）。
    只读的那一段（read_only_flags）里的路径不收，赋值与 cd 照记（后面的段要用）。"""
    exact, patterns = set(), []
    for call in calls:
        if call["name"] != "Bash":
            continue
        command = str(call["input"].get("command") or "")
        if names is not None and not any(name in command for name in names):
            continue
        words = command_words(substituted_work_roots(command))
        read_only = read_only_flags(words)
        variables, directory = {}, None
        for index, word in enumerate(words):
            assignment = VARIABLE_ASSIGNMENT.match(word)
            values = [assignment.group(2)] if assignment else ([] if word.startswith("-") or SEGMENT_SEPARATOR.fullmatch(word) else [word])
            if not assignment and "=" in word:
                values.append(word.split("=", 1)[1])
            if read_only[index]:
                values = []
            for value in values:
                path = resolved_word(value, variables, directory)
                if path is None:
                    continue
                if GLOB_CHARACTERS.search(path):
                    patterns.append(path)
                else:
                    exact.add(path)
            if assignment:
                value = resolved_word(assignment.group(2), variables, directory)
                if value is not None:
                    variables[assignment.group(1)] = value
            if index > 0 and words[index - 1] in DIRECTORY_CHANGERS:
                value = resolved_word(word, variables, directory)
                if value is not None and not GLOB_CHARACTERS.search(value):
                    directory = value
    return exact, patterns


def glob_matches(path, pattern):
    path_parts, pattern_parts = path.split("/"), pattern.split("/")
    return len(path_parts) == len(pattern_parts) and all(fnmatch.fnmatchcase(part, wanted) for part, wanted in zip(path_parts, pattern_parts))


def literal_prefix(pattern):
    """通配里头一个带 * ? [ 的那一层之前的那一截。"""
    parts = pattern.split("/")
    for index, part in enumerate(parts):
        if GLOB_CHARACTERS.search(part):
            return "/".join(parts[:index]) or "/"
    return pattern


def point_attributed(path, exact, patterns, anchors):
    """path 本身或它的上层（到锚点为止，锚点本身不算）被自己的 Bash 命令点到过。弄坏开关 unmentioned-counted-own 下都算，
    anchor-mentions-counted 下锚点与字面前缀是锚点的通配也算，mention-ancestors-ignored 下只认 path 本身。"""
    if broken("unmentioned-counted-own"):
        return True
    anchors_counted = broken("anchor-mentions-counted")
    usable = [pattern for pattern in patterns if anchors_counted or literal_prefix(pattern) not in anchors]
    current = path
    while True:
        if current in anchors and not anchors_counted:
            return False
        if current in exact or any(glob_matches(current, pattern) for pattern in usable):
            return True
        parent = os.path.dirname(current)
        if broken("mention-ancestors-ignored") or parent == current:
            return False
        current = parent


def migration_leftovers(role, main_transcript, calls, judged_roots, exempt, budget, notes):
    """迁库遗留：交 (自己的 [(路径, 种类)], 认不出是谁的 [(路径, 种类)])，在不在用留给调用方判。归属只看自己的会话记录
    （弄坏开关 leftover-mentions-session-level 下子 agent 另算主会话记录里的命令）。"""
    places, skipped, _service, anchors, data = leftover_places(judged_roots, notes)
    if data is not None and not broken("registered-entries-listed"):
        library = module("lib_work_root")
        budget["not_by_name"] = {os.path.join(data, name) for name in (*library.SHARED_DATA_ENTRIES, *library.PER_HOST_DATA_ENTRIES)}
    found = {}
    for root, depth, is_work_root in places:
        if os.path.islink(root) or not os.path.isdir(root):
            continue

        def skip(path, level, is_work_root=is_work_root):
            if path in skipped:
                return True
            return (is_work_root and level == 1 and owner_session(path) is not None
                    and not broken("owned-directories-leftover-scanned"))
        for path, kind in scan(root, depth, exempt, set(), budget, LEFTOVER_KINDS, skip):
            found.setdefault(path, kind)
    if not found:
        return [], []
    names = None
    if not broken("anchor-mentions-counted"):
        names = set()
        for path in found:
            current = path
            while current not in anchors and os.path.dirname(current) != current:
                names.add(os.path.basename(current))
                current = os.path.dirname(current)
    own_calls = list(calls)
    if role == "subagent" and broken("leftover-mentions-session-level"):
        own_calls += module("session_transcript").tool_calls(main_transcript)[0]
    exact, patterns = mentioned_paths(own_calls, names)
    own, unknown = [], []
    for path, kind in sorted(found.items()):
        (own if point_attributed(path, exact, patterns, anchors) else unknown).append((path, kind))
    return own, unknown


def shown_unknown(items):
    """「认不出是谁的」里列哪几处迁库遗留：在用的去掉，最多 UNKNOWN_SHOWN_MAX 处；交 (列的, 没核、没列的个数)。"""
    kept = []
    for index, item in enumerate(items):
        if len(kept) >= UNKNOWN_SHOWN_MAX:
            return kept, len(items) - index
        if broken("open-files-ignored") or not in_use(item[0]):
            kept.append(item)
    return kept, 0


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
def file_held(path):
    """有本进程以外的进程开着这一份文件：/proc/<pid>/fd 或 maps 指着它（库备份是文件，open_holders 只认目录；只看得到本用户的）。"""
    real = os.path.realpath(path)
    own = os.getpid()
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == own:
            continue
        base = os.path.join("/proc", entry)
        try:
            descriptors = os.listdir(os.path.join(base, "fd"))
        except OSError:
            descriptors = []
        for descriptor in descriptors:
            try:
                if os.readlink(os.path.join(base, "fd", descriptor)) == real:
                    return True
            except OSError:
                pass
        try:
            with open(os.path.join(base, "maps"), encoding="utf-8", errors="replace") as handle:
                if any(len(line.split(None, 5)) == 6 and line.split(None, 5)[5].strip() == real for line in handle):
                    return True
        except OSError:
            pass
    return False


def in_use(path):
    """有进程开着 path 里的文件（目录照 scripts/hosts/run_build_directory.py 的 open_holders，与它的 holders 同一份判法；文件照 file_held）。"""
    if not os.path.isdir(path) and os.path.lexists(path):
        return file_held(path)
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


# ── 合入后验证留下的快照（文件头「判哪几处」合入后验证快照那一段）
def merge_snapshot_layers(session, notes):
    """<工作根>/merge-and-verify/ 下归属是本会话的每一层：交 [一层的路径]。.owner 要三行写全（owner_session），没有或写得不对的不认；
    <工作根>/merge-and-verify/ 在而列不出时往 notes 记一句、交空表（照「放行但必记检出」）。
    弄坏开关 snapshot-owner-unchecked 下没有归属文件的一层也认，snapshot-session-unchecked 下别的会话的也认，merge-snapshot-root-unlisted-unnoted 下列不出不记。"""
    base = os.path.join(os.path.normpath(work_root()), MERGE_SNAPSHOT_DIRECTORY_NAME)
    if os.path.islink(base) or not os.path.isdir(base):
        return []
    try:
        names = sorted(os.listdir(base))
    except OSError as error:
        if not broken("merge-snapshot-root-unlisted-unnoted"):
            notes.append(f"合入后验证的日志目录根 {base} 列不出（{type(error).__name__}: {error}），本会话留下的快照没判；出路：ls -ld {base} 看权限，修好再收工")
        return []
    layers = []
    for name in names:
        path = os.path.join(base, name)
        if os.path.islink(path) or not os.path.isdir(path):
            continue
        owner = owner_session(path)
        if owner is None and not broken("snapshot-owner-unchecked"):
            continue
        if owner is not None and owner != session and not broken("snapshot-session-unchecked"):
            continue
        layers.append(path)
    return layers


def merge_snapshots(layers, exempt):
    """每一层里还在的快照：交 {一层: [(快照路径, 种类)]}。快照是同层的 repo/（目录、不是符号链接）；一层的 .deliverables 声明成交付物的（exempt）不列。
    弄坏开关 snapshot-removed-listed 下 repo/ 已删的一层也列（列那一层的目录），merge-snapshots-unseen 下不看快照、交空表。"""
    found = {}
    if broken("merge-snapshots-unseen"):
        return found
    for layer in layers:
        snapshot = os.path.join(layer, MERGE_SNAPSHOT_NAME)
        present = os.path.isdir(snapshot) and not os.path.islink(snapshot)
        target = snapshot if present else (layer if broken("snapshot-removed-listed") else None)
        if target is not None and snapshot not in exempt:
            found[layer] = [(target, KIND_MERGE_SNAPSHOT)]
    return found


def snapshots_not_in_use(found):
    """一层里有进程开着（那一趟还在跑，或有进程在快照里）的整层不列，判法同运行目录（看整层，不只看列出的那一处）；弄坏开关 open-files-ignored 下都列。"""
    if broken("open-files-ignored"):
        return [item for items in found.values() for item in items]
    return [item for layer, items in found.items() if not in_use(layer) for item in items]


# ── 任务现场（文件头「判哪几处」任务现场那一段）
def task_server_database(notes):
    """服务库的路径：COMPLETION_CLEANUP_TASK_SERVER_DATABASE 设了照它（自证用）；别的取 task_server_lib.py 的 read_configuration，读不了取 default_values。
    共用模块载不进、算不出库的位置记一处没判全、交 None。"""
    override = os.environ.get(TASK_SERVER_DATABASE_VARIABLE)
    if override:
        return override
    try:
        library = module("task_server_lib")
    except Exception as error:
        notes.append(f"读不到 {MODULE_PATHS['task_server_lib']}（{error!r}），任务现场（结局没取走、或设了 {KEEP_SCENE_VARIABLE}={KEEP_SCENE_VALUE}）没判；出路：把 task-server/ 放回仓里，"
                     f"本会话交的任务逐个 {TASK_SERVER_CLIENT} status <号> 看现场还在不在，结局没取走的 {TASK_SERVER_CLIENT} wait <号>，设了它的读完再 {TASK_SERVER_CLIENT} clean <号>")
        return None
    try:
        return library.read_configuration().database
    except Exception:
        try:
            defaults = library.default_values(dict(os.environ))
            return os.path.join(defaults["TASK_SERVER_STATE_DIRECTORY"], library.DATABASE_NAME)
        except Exception as error:
            notes.append(f"算不出服务库的位置（{error!r}），任务现场（结局没取走、或设了 {KEEP_SCENE_VARIABLE}={KEEP_SCENE_VALUE}）没判；"
                         f"出路：{TASK_SERVER_CLIENT} status 看本会话的任务，结局没取走的 {TASK_SERVER_CLIENT} wait <号>，设了它交的逐个读完现场再 {TASK_SERVER_CLIENT} clean <号>")
            return None


def kept_scenes(role, session, notes):
    """本会话已有结局、尝试记的现场目录还在盘上，又占着服务的两个不到期条件之一（结局没取走；提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1）的任务：
    [(现场目录, (删它用的任务号, 设没设留现场, 结局没取走))]（读法在文件头「判哪几处」任务现场那一段）。
    只判主 agent；子 agent 不交任务、不起 task clean，交一张空表。服务库只读打开（sqlite 只读模式）：任务按会话列取，尝试按任务号接任务表、用任务表的会话判
    （服务写尝试行时会话列是空串）。
    弄坏开关 kept-scenes-ignored 下一个都不认，kept-scenes-keep-unchecked 下两个条件都不看（结局已取走、没设留现场的也算），kept-scenes-uncollected-unchecked 下不看结局取没取走，
    kept-scenes-keep-variable-unchecked 下列不列只看结局取没取走（设了留现场、结局已取走的不列），kept-scenes-session-unchecked 下别的会话的也算，kept-scenes-unfinished-listed 下在排、在跑的也算，subagent-kept-scenes-judged 下子 agent 也算本会话的全部，
    kept-scene-attempts-by-session-column 下尝试按尝试表自己的会话列取。"""
    if broken("kept-scenes-ignored"):
        return []
    if role != "main" and not broken("subagent-kept-scenes-judged"):
        return []
    database = task_server_database(notes)
    if database is None or not os.path.exists(database):
        return []
    any_session = broken("kept-scenes-session-unchecked")
    where, parameters = ("1", ()) if any_session else ("session = ?", (session,))
    if broken("kept-scene-attempts-by-session-column"):
        attempt_query = f"SELECT task_id, record FROM attempts WHERE {where}"
    else:
        attempt_query = ("SELECT attempts.task_id, attempts.record FROM attempts JOIN tasks ON attempts.task_id = tasks.id WHERE "
                         + ("1" if any_session else "tasks.session = ?"))
    try:
        connection = sqlite3.connect(f"file:{urllib.parse.quote(os.path.abspath(database))}?mode=ro", uri=True, timeout=5)
        try:
            tasks = {row[0]: (row[1], row[2]) for row in connection.execute(f"SELECT id, state, record FROM tasks WHERE {where}", parameters)}
            attempts = connection.execute(attempt_query, parameters).fetchall()
        finally:
            connection.close()
    except sqlite3.Error as error:
        notes.append(f"服务库 {database} 读不了（{error}），任务现场（结局没取走、或设了 {KEEP_SCENE_VARIABLE}={KEEP_SCENE_VALUE}）没判；出路：{TASK_SERVER_CLIENT} status 看本会话的任务，"
                     f"结局没取走的 {TASK_SERVER_CLIENT} wait <号>，设了它交的逐个读完现场再 {TASK_SERVER_CLIENT} clean <号>；库读不了是服务那一侧的事，交主 agent 看 python3 task-server/control check")
        return []

    def record_of(text):
        try:
            decoded = json.loads(text)
        except (TypeError, ValueError):
            return {}
        return decoded if isinstance(decoded, dict) else {}

    owner = {}   # 任务号 → 删它用的任务号（分片的子任务交父任务号：task clean <父任务号> 连子任务的现场一起删，留不留现场、取没取走也照父任务）
    for task_id, (_state, text) in tasks.items():
        parent = record_of(text).get("parent_id")
        owner[task_id] = parent if isinstance(parent, int) and parent in tasks else task_id
    found = {}
    for task_id, text in attempts:
        top = owner.get(task_id)
        if top is None:
            continue
        state, top_text = tasks[top]
        if state != TASK_SERVER_FINISHED_STATE and not broken("kept-scenes-unfinished-listed"):
            continue
        top_record = record_of(top_text)
        environment = top_record.get("environment")
        kept = isinstance(environment, dict) and environment.get(KEEP_SCENE_VARIABLE) == KEEP_SCENE_VALUE
        uncollected = not top_record.get("collected") and not broken("kept-scenes-uncollected-unchecked")
        listed = (kept and not broken("kept-scenes-keep-variable-unchecked")) or uncollected
        if not listed and not broken("kept-scenes-keep-unchecked"):
            continue
        directory = record_of(text).get("scene_directory")
        if isinstance(directory, str) and directory and os.path.isdir(directory):
            found.setdefault(os.path.abspath(directory), (top, kept, uncollected))
    return sorted(found.items())


# ── 换上流程的草稿树（文件头「判什么」换上流程的草稿树那一段）
def swap_tree_scratch(path):
    """path 是 <草稿目录>/hook-swap/tree 这一形就交草稿目录，不是交 None。"""
    parts = os.path.normpath(path).split(os.sep)
    if len(parts) > len(SWAP_TREE_PARTS) + 1 and tuple(parts[-len(SWAP_TREE_PARTS):]) == SWAP_TREE_PARTS:
        return os.sep.join(parts[:-len(SWAP_TREE_PARTS)])
    return None


def named_on_command_lines(tree, scratch):
    """本进程以外有进程的命令行点到这棵树（树本身或它底下的路径），或点到它的草稿目录本身（词里 --x=<路径> 取等号后面也算；草稿目录底下别的路径不算）。"""
    own = os.getpid()
    pattern = re.compile(r"(?<![\w./-])(?:" + re.escape(tree) + r"(?=$|[/\s'\"=:,;)])|" + re.escape(scratch) + r"(?=$|[\s'\"=:,;)]))")
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == own:
            continue
        try:
            with open(os.path.join("/proc", entry, "cmdline"), "rb") as handle:
                command = handle.read().replace(b"\0", b" ").decode(errors="replace")
        except OSError:
            continue
        if pattern.search(command):
            return True
    return False


def swap_trees_still_left(leftovers):
    """决定拦之前最后再看一眼：<草稿目录>/hook-swap/tree 这一形的行，有进程在用（in_use 的判法，或命令行点到树与草稿目录：那一趟换上还在跑）、
    或这一刻已经没了（那一趟刚删掉）的不列；没有进程在用、还在的照列，别的行原样留下。弄坏开关 swap-tree-always-listed 下不看，都列。"""
    if broken("swap-tree-always-listed"):
        return leftovers
    kept = []
    for path, kind in leftovers:
        scratch = swap_tree_scratch(path)
        if scratch is not None and (not os.path.lexists(path) or in_use(path) or named_on_command_lines(path, scratch)):
            continue
        kept.append((path, kind))
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
    if kind == KIND_KEPT_SCENE and not broken("kept-scene-removal-by-rm"):
        # 只是结局没取走的取走结局（服务过一小时自己删）；设了留现场、结局已取走的由 task clean 删；两样都占的先取走结局、再 task clean
        # （task clean 只删现场、不算取走结局，结局照旧算没人取，服务之后每次收这个会话的任务都打严厉警告：task-server/task_server.py 文件头「提交时警告」②）。
        # 弄坏开关 kept-scene-removal-uncollected-first 下两样都占的只写 wait，kept-scene-removal-wait-skipped 下两样都占的只写 clean，
        # uncollected-scene-removal-by-clean 下只是结局没取走的也写 clean
        task_id, kept, uncollected = KEPT_SCENE_TASKS.get(path, ("<号>", True, False))
        wait, clean = f"{TASK_SERVER_CLIENT} wait {task_id}", f"{TASK_SERVER_CLIENT} clean {task_id}"
        if kept and uncollected:
            if broken("kept-scene-removal-uncollected-first"):
                return wait
            if broken("kept-scene-removal-wait-skipped"):
                return clean
            return f"先 {wait}，再 {clean}"
        return wait if uncollected and not broken("uncollected-scene-removal-by-clean") else clean
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


def unknown_lines(unknown, omitted=0):
    if not unknown and not omitted:
        return []
    lines = [f"「{UNKNOWN_SECTION_TITLE}」（只报不拦：认不出是不是你建的。运行目录：建在你某次 Bash 调用的时间窗里，但起它的进程已经不在、没记账、也没有归属记录；"
             "迁库遗留：在数据根、判定库服务的状态目录或没有归属文件的工作根目录里，你的 Bash 命令没点到它和它的上层目录；"
             "删之前先核是谁的，不是你的别动）：", *leftover_lines(unknown)]
    if omitted:
        lines.append(f"  另有 {omitted} 处迁库遗留没列（这一节最多列 {UNKNOWN_SHOWN_MAX} 处，没核在不在用）")
    return lines


def declaration_problems_text(problems):
    """交回那一次（PreToolUse）拒的说明：DECLARATION_FILE 里写法不合的行不算声明，点名它们、写出种类与例子。"""
    return "\n".join([
        f"✗ 交回前先改对 {DECLARATION_FILE}：下面 {len(problems)} 处写法不合，那几行不算声明（它们要留的副本会在每次收工时照列）：",
        *[f"   ! {problem}" for problem in problems],
        f"→ 怎么办：每行写成「<相对那个认领目录的路径> <种类> <为什么留>」，种类五选一：{'、'.join(DELIVERABLE_KINDS)}，"
        f"理由至少 {REASON_MINIMUM_CHARACTERS} 个字，路径落在那个认领目录里面（带空格的路径整个用引号包着）；# 起头的行是注释；{DECLARATION_FILE} 要是普通文件、不是符号链接。",
        "   例：result.md  报告  这一轮的报告，主 agent 按路径存档",
        f"   子 agent 交回时免列的是{'、'.join(kind for kind in DELIVERABLE_KINDS if kind != BASELINE_KIND)}；{BASELINE_KIND}只认主 agent 的声明。"
        f"写法在 {HOOK_PATH} 文件头「要留的交付物」。"])


def nested_declaration_notes(leftovers, roots, role):
    """上层子 agent 被拦时：单子上落在下层目录里的副本（不是认领目录或草稿目录的直接下一层），逐个点名下层的声明为什么没认上。
    看从那个目录往下到这一份的那几层：.deliverables 是符号链接（写出指向哪）、不是普通文件、读不了、写坏的行（照 declarations 的说法，带文件:行号），
    或这几层都不在（没写，或写了又被删了）；都好好的就点名它们里没有声明这一份的行。不看弄坏开关 nested-declaration-ignored 与 declaration-symlink-skipped：
    这里只说下层那几份此刻的样子。主 agent 不读下层的声明，一句都不写；弄坏开关 nested-declaration-problems-unnamed 下也一句都不写。"""
    if role != "subagent" or broken("nested-declaration-problems-unnamed"):
        return []
    exempt_kinds = "、".join(kind for kind in DELIVERABLE_KINDS if kind != BASELINE_KIND)
    notes = []
    for path, _ in leftovers:
        top = next((root for root, _, _ in roots if is_strictly_under(path, root)), None)
        if top is None:
            continue
        layers, directory = [], os.path.dirname(path)
        while is_strictly_under(directory, top):
            layers.append(directory)
            directory = os.path.dirname(directory)
        if not layers:
            continue
        states, present = [], []
        for layer in layers:
            declaration = os.path.join(layer, DECLARATION_FILE)
            if os.path.islink(declaration):
                try:
                    pointed = os.readlink(declaration)
                except OSError as error:
                    pointed = f"读不出指向：{error.strerror or error}"
                states.append(f"{declaration} 是符号链接（指向 {pointed}），不读，里面的声明不算")
            elif not os.path.lexists(declaration):
                continue
            elif not os.path.isfile(declaration):
                states.append(f"{declaration} 不是普通文件，不读")
            else:
                present.append(declaration)
                states += declarations([layer], role)[1]
        if states:
            detail = "；".join(states)
        elif present:
            detail = f"{'、'.join(present)} 里没有一行把它声明成{exempt_kinds}"
        else:
            detail = "、".join(f"{os.path.join(layer, DECLARATION_FILE)} 不在" for layer in layers) + "（下层没写，或写了又被删了）"
        notes.append(f"{path} 在下层目录 {layers[0]} 底下，下层的声明没认上它：{detail}")
    return notes


def block_text(role, leftovers, problems, notes, unknown=(), omitted=0, place_paths=(), event=""):
    if role == "main":
        head = ("✗ 完工清理：这一轮做完了（没有在跑的后台任务与子 agent），本会话认领的目录与它起的运行目录里还剩下面这些仓副本、编译目录与库拷贝，"
                "或本会话合入后验证留下的快照、点到过的迁库遗留、结局没取走或设了留现场的任务现场还在；用完就删，删完再收工。")
        rule = ".claude/main-agent.md「一轮怎么开、怎么收」第 11 条"
        kinds = f"报告、补丁、备份、{BASELINE_KIND}（主 agent 自己留着、后面要在它上面跑命令或以它为基线建副本的树）"
    else:
        head = "✗ 完工清理：交回之前先删掉你自己认领、自己建的仓副本、编译目录与库拷贝，与你的命令点到过的迁库遗留（下面这些还在）；用完就删，删完再交回。"
        rule = ".claude/agent-common.md「写」一节"
        kinds = "报告、补丁、备份、交接（交给主 agent 接着用的副本，报告里写明主 agent 在里面跑什么）"
    lines = [head, *leftover_lines(leftovers),
             "→ 怎么办：逐个删（删之前 du -sh 记下的大小写进报告），删法：",
             *[f"    {command}" for command in dict.fromkeys(removal_command(path, kind) for path, kind in leftovers)],
             f"   不留给以后的趟复用，也不等别的活来用（{rule}）。",
             f"   报告、补丁与打补丁前的备份是普通文件的不在单上；要留的副本写进它所在认领目录的 {DECLARATION_FILE}：一行「<相对路径> <种类> <为什么留>」，"
             f"种类是 {kinds}；编译目录声明了也照列。写法在 {HOOK_PATH} 文件头「要留的交付物」。"]
    handable = next((path for path, kind in leftovers if kind in HAND_TO_MAIN_KINDS), None)
    if role != "main" and handable is not None and not broken("hand-way-unlisted"):
        where = "回复里" if event == "SubagentStop" else "这一次交回的正文里"
        example_reason = HAND_TO_MAIN_EXAMPLE_REASON + ("字" * HAND_TO_MAIN_OVERSIZED_PADDING if broken("hand-way-example-oversized") else "")
        lines += [f"   删不掉的{'、'.join(HAND_TO_MAIN_KINDS)}（rm 被权限拒、被别的闸挡）：{where}逐个写一行「{HAND_TO_MAIN_PHRASE} <全路径>：<删不掉的原因，至少 {HAND_TO_MAIN_REASON_MINIMUM} 个字>」"
                  f"（全路径照抄上面的清单，上层目录不替下层），写了再{'收工' if event == 'SubagentStop' else '交回'}就放行；主 agent 结束一轮时照旧列它们、由它删。"
                  f"{KIND_LIBRARY}、{KIND_OLD_LIBRARY}、{KIND_LIBRARY_ARCHIVE}不认这一种写法。",
                  f"   例：{HAND_TO_MAIN_PHRASE} {handable}：{example_reason}"]
    if any(kind == KIND_KEPT_SCENE for _, kind in leftovers):
        lines.append(f"   {KIND_KEPT_SCENE}是 task-server 给每一次尝试的现场目录（尝试记录的 scene_directory），本会话的任务已有结局、服务还不到时删"
                     f"（task-server/task_server_scheduler_building.py 的 scene_due）：只是结局没取走的，经 `{TASK_SERVER_CLIENT} wait <号>` 取走结局，服务过一小时自己删；"
                     f"提交时设了 {KEEP_SCENE_VARIABLE}={KEEP_SCENE_VALUE}、结局已取走的只由 task clean 删，还要读的读完再经 `{TASK_SERVER_CLIENT} clean <号>` 删；"
                     f"两样都占的先 `{TASK_SERVER_CLIENT} wait <号>`、再 `{TASK_SERVER_CLIENT} clean <号>`（clean 只删现场、不算取走结局，结局没取走的服务之后每次收这个会话的任务都打严厉警告）。"
                     f"分片的写父任务号，不直接 rm（task-server/task_server_lib.py 文件头「现场」）。")
    if any(kind == KIND_MERGE_SNAPSHOT for _, kind in leftovers):
        lines.append(f"   {KIND_MERGE_SNAPSHOT}是合入后验证带 MERGE_AND_VERIFY_KEEP_COPY=1 留下的 <日志目录>/repo，那一层的 .owner 记着是本会话的：只删 repo、日志留着；"
                     f"验证段后面还要在它上面验、或修补要以它为基线建副本的，在它所在日志目录的 {DECLARATION_FILE} 写一行「{MERGE_SNAPSHOT_NAME} {BASELINE_KIND} <为什么留>」，用完就删。")
    if any(path in place_paths for path, _ in leftovers):
        lines.append(f"   数据根、判定库服务的状态目录与没有归属文件的工作根目录里的迁库遗留（{KIND_LIBRARY}、{KIND_OLD_LIBRARY}、{KIND_LIBRARY_ARCHIVE}）"
                     f"是你的命令点到过的：判定已在 prune-server 那一份里，这些不靠声明留；真要留，先问用户。")
    lines += [f"   ! {problem}" for problem in problems + notes]
    lines += unknown_lines(list(unknown), omitted)
    return "\n".join(lines)


def release_text(leftovers, unknown=(), omitted=0):
    return "\n".join(["完工清理闸：拦下之后这几项一个没删，这一次放行收工，交用户定：", *leftover_lines(leftovers), *unknown_lines(list(unknown), omitted)])


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


# ── 交主 agent 删
def hand_to_main_texts(role, event, hook_input, calls, own_transcript):
    """可能写着「交主 agent 删」行的文字：交回工具那一次是这一次交回的 message；子 agent 收工是已送达（结果不是错误）的交回正文加收工前的回复；
    主 agent 只在弄坏开关 hand-exempt-for-main 下取自己会话里的回复。"""
    if role == "subagent" and event == "PreToolUse":
        tool_input = hook_input.get("tool_input")
        message = tool_input.get("message") if isinstance(tool_input, dict) else None
        return [message] if isinstance(message, str) else []
    texts = []
    if role == "subagent":
        texts += [str(call["input"].get("message") or "") for call in calls if call["name"] == HANDBACK_TOOL and not call["result_is_error"]]
    texts += [text for _, text in module("session_transcript").assistant_texts(own_transcript)]
    return texts


def handed_to_main(role, event, hook_input, calls, own_transcript):
    """{路径: 原因}：交回正文（或收工前的回复）里写了「交主 agent 删 <全路径>：<原因>」、原因够长的路径。主 agent 不认（弄坏开关 hand-exempt-for-main 下认），
    弄坏开关 hand-declaration-ignored 下谁都不认，hand-reason-unchecked 下原因多短都认。"""
    if broken("hand-declaration-ignored") or (role == "main" and not broken("hand-exempt-for-main")):
        return {}
    minimum = 0 if broken("hand-reason-unchecked") else HAND_TO_MAIN_REASON_MINIMUM
    handed = {}
    for text in hand_to_main_texts(role, event, hook_input, calls, own_transcript):
        for match in HAND_TO_MAIN_LINE.finditer(text):
            reason = match.group(2).strip()
            if len(reason) >= minimum:
                handed[os.path.normpath(match.group(1))] = reason
    return handed


def handed_leftovers(found_in_roots, handed):
    """认领目录与草稿目录里单子上的那几处里，已经写了「交主 agent 删」的路径：只认 HAND_TO_MAIN_KINDS 三种，路径要与单子上的一字不差
    （弄坏开关 hand-all-kinds 下什么种类都认，hand-ancestor-accepted 下上层目录写了也算）。"""
    declared = set()
    for path, kind in found_in_roots:
        if kind not in HAND_TO_MAIN_KINDS and not broken("hand-all-kinds"):
            continue
        if path in handed or (broken("hand-ancestor-accepted") and any(is_strictly_under(path, above) for above in handed)):
            declared.add(path)
    return declared


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
    if not os.access(own_transcript, os.R_OK) and not broken("transcript-unreadable-unnoted"):   # 在而读不了：读它的共用模块静默交空表，认领与草稿目录都认不出（当没有、照常放行）
        return {"action": "skip", "text": f"会话记录 {own_transcript} 在而读不了，认领与派发提示点名的草稿目录都认不出"}
    now = time.time()
    calls, _ = module("session_transcript").tool_calls(own_transcript)
    notes = []
    roots = main_roots(session, main_transcript, notes) if role == "main" or broken("subagent-sees-whole-session") else subagent_roots(session, own_transcript, calls)
    layers = merge_snapshot_layers(session, notes) if role == "main" or broken("subagent-snapshots-judged") else []
    exempt, problems = declarations([*(directory for root in roots for directory in (root[0], root[1])), *layers], role)
    # 交回时只拦这个 agent 自己写的那几层：认领的目录、点名的草稿目录与（主 agent 的）合入后验证那一层；它们所在的工作根下一层是别人写的，读它只为取免列声明
    # （弄坏开关 handback-parent-declaration-judged 下那一层里的坏行也拦）
    own_problems = problems if broken("handback-parent-declaration-judged") else declarations([*(root[0] for root in roots), *layers], role)[1]
    if own_problems and event == "PreToolUse" and not broken("handback-declaration-unchecked"):
        return {"action": "block", "text": declaration_problems_text(own_problems), "paths": [], "unknown": []}
    species = library_species(notes)
    budget = {"visited": 0, "truncated": set(), "limit": scan_limit(), "unreadable": [], "species": species}
    found_in_roots = []
    # 上层子 agent 交回时，下层子 agent 在它那一层声明了的副本算已声明（弄坏开关 nested-declaration-overtrusted 下主 agent 也读）
    nested_role = None if broken("nested-declaration-ignored") or (role != "subagent" and not broken("nested-declaration-overtrusted")) else role
    for path, _, skipped in roots:
        found_in_roots += scan(path, SCAN_MAX_DEPTH, exempt, skipped, budget, nested_role=nested_role)
    handed = handed_to_main(role, event, hook_input, calls, own_transcript)
    if handed:
        declared_away = handed_leftovers(found_in_roots, handed)
        found_in_roots = [(path, kind) for path, kind in found_in_roots if path not in declared_away]
    own_runs, unknown_runs = classify_runs(role, session, own_transcript, calls, now, notes)
    found_in_runs = {path: scan(path, RUN_SCAN_MAX_DEPTH, {}, set(), budget) for path in own_runs}
    found_unknown = {path: scan(path, RUN_SCAN_MAX_DEPTH, {}, set(), budget) for path in unknown_runs}
    if budget["truncated"]:
        notes.append(f"看了 {budget['limit']} 个目录到顶，这几处没看完：{' '.join(sorted(budget['truncated']))}；"
                     f"出路：这几处里不用的删掉（目录少了下一次才看得完），要留的在它所在认领目录的 {DECLARATION_FILE} 里登记（写法在 {HOOK_PATH} 文件头「要留的交付物」）")
    if budget["unreadable"]:
        shown = budget["unreadable"]
        notes.append(f"读不了的目录 {len(shown)} 个没看（里面有没有副本、编译目录与库拷贝没判）：{' '.join(shown[:5])}{' …' if len(shown) > 5 else ''}；"
                     "出路：ls -ld 看权限，不用的删掉，要留的在它所在认领目录的 .deliverables 里登记")
    # 迁库遗留另用一份扫描上限：上面那几处的没判全照旧单独一句，不与这一类混在一起
    leftover_budget = {"visited": 0, "truncated": set(), "limit": scan_limit(), "unreadable": [], "species": species}
    own_leftovers, unknown_leftovers = migration_leftovers(role, main_transcript, calls, roots, exempt, leftover_budget, notes)
    if leftover_budget["truncated"]:
        notes.append(f"迁库遗留看了 {leftover_budget['limit']} 个目录到顶，这几处没看完：{' '.join(sorted(leftover_budget['truncated']))}；"
                     "出路：这几处里不用的旧库与备份删掉（目录少了下一次才看得完）")
    if leftover_budget["unreadable"]:
        shown = leftover_budget["unreadable"]
        notes.append(f"迁库遗留那几处有 {len(shown)} 个目录读不了、没看（里面有没有旧库与备份没判）：{' '.join(shown[:5])}{' …' if len(shown) > 5 else ''}；"
                     "出路：ls -ld 看权限，不用的删掉")
    scenes = kept_scenes(role, session, notes)
    KEPT_SCENE_TASKS.update(scenes)
    snapshots = snapshots_not_in_use(merge_snapshots(layers, exempt))
    leftovers = sorted(dict(not_in_use(found_in_roots + own_leftovers, found_in_runs) + [(path, KIND_KEPT_SCENE) for path, _ in scenes] + snapshots).items())
    listed = {path for path, _ in leftovers}
    leftover_unknown, omitted = shown_unknown([item for item in unknown_leftovers if item[0] not in listed])
    unknown = sorted(dict(not_in_use([], found_unknown) + leftover_unknown).items())
    state_file = state_path(session, actor)
    if leftovers:
        if background_work(own_transcript) or (role == "main" and running_subagents(main_transcript, now)):
            return {"action": "pass", "running": True}
        leftovers = swap_trees_still_left(leftovers)
    if not leftovers:
        clear_state(state_file)
        return {"action": "pass", "notes": notes, "unknown": unknown, "unknown_omitted": omitted}
    paths = [path for path, _ in leftovers]
    place_paths = {path for path, _ in own_leftovers}
    if event != "PreToolUse" and hook_input.get("stop_hook_active") is True and not broken("release-never"):
        if load_state(state_file).get("paths") == paths or broken("release-on-any-continuation"):
            return {"action": "release", "text": release_text(leftovers, unknown, omitted), "paths": paths, "unknown": unknown}
    save_state(state_file, paths)
    nested_notes = nested_declaration_notes(leftovers, roots, role)
    return {"action": "block", "text": block_text(role, leftovers, problems, notes + nested_notes, unknown, omitted, place_paths, event), "paths": paths, "unknown": unknown}


def report_unjudged_pass(hook_input, notes, stream, detections_path=None):
    """放行而没判全（扫到上限没看完、运行目录取不到创建时间）：照「放行但必记检出」往 stream 逐条打出、检出记录追加一行（结局「只记」），看门狗据此报给主 agent。
    detections_path 不给取 AGENT_HOOK_DETECTIONS，没设取 lib_work_root.py 的 default_detections_path。交打出的行数；弄坏开关 unjudged-pass-silent 下不打、不记。"""
    if not notes or broken("unjudged-pass-silent"):
        return 0
    recorded = record_detection(hook_input, [f"{UNJUDGED_FINDING_PREFIX}：{note}" for note in notes], detections_path)
    lines = ["! 完工清理闸：这一次没判全，照「放行但必记检出」照常放行：", *[f"   ! {note}" for note in notes], f"   {recorded}"]
    print("\n".join(lines), file=stream)
    return len(lines)


def record_detection(hook_input, findings, detections_path=None):
    """检出记录追加一行（结局「只记」）：report_unjudged_pass 与 report_unjudged 共用。detections_path 不给取 AGENT_HOOK_DETECTIONS，
    没设取 lib_work_root.py 的 default_detections_path。交一句给人看的说明：记进了哪里，或为什么没记成。"""
    hook_input = hook_input if isinstance(hook_input, dict) else {}
    event = str(hook_input.get("hook_event_name") or "")
    path = detections_path or os.environ.get(DETECTIONS_VARIABLE)
    if not path:
        try:
            path = module("lib_work_root").default_detections_path()
        except Exception:  # noqa: BLE001 —— 读不到工作根的定义：不知道往哪写，下面照打一句
            path = None
    entry = {"time": datetime.now(timezone.utc).isoformat(), "session_id": hook_input.get("session_id") or "读不到", "agent_id": hook_input.get("agent_id"),
             "agent_type": hook_input.get("agent_type") or ("主 agent" if event == "Stop" else "子 agent"),
             "transcript_path": hook_input.get("transcript_path"), "command": f"完工清理闸（{event or '事件读不到'}）",
             "findings": list(findings), "outcome": "只记"}
    if not path:
        return f"检出没记成：取不到检出记录的位置（{DETECTIONS_VARIABLE} 没设，lib_work_root.py 也读不到）"
    try:
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        return f"检出没记成：写不了 {path}（{error}）"
    return f"检出记进 {path}"


def report_unjudged(hook_input, text, stream, detections_path=None):
    """判不了（输入不是 JSON 对象、认不出会话号、找不到会话记录、导入不了共用模块、自己出错）：照「放行但必记检出」往 stream 打一句「没判」、
    检出记录追加一行（结局「只记」，findings 以 UNJUDGED_SKIP_PREFIX 起头），看门狗据此报给主 agent。弄坏开关 unjudged-skip-silent 下只打那一句、不记。"""
    if broken("unjudged-skip-silent"):
        print(f"! 完工清理闸：{text}，这一次没判（放行）", file=stream)
        return
    recorded = record_detection(hook_input, [f"{UNJUDGED_SKIP_PREFIX}：{text}"], detections_path)
    print(f"! 完工清理闸：{text}，这一次没判（放行），照「放行但必记检出」{recorded}", file=stream)


def hook_main():
    raw = sys.stdin.buffer.read().decode("utf-8", "replace")
    try:
        hook_input = json.loads(raw)
    except ValueError as error:
        report_unjudged({}, f"钩子输入不是合法的 JSON（{type(error).__name__}: {error}；读到 {len(raw)} 个字符，前 {INPUT_PREFIX_BYTES} 个：{raw[:INPUT_PREFIX_BYTES]!r}）",
                        sys.stderr)
        return 0
    try:
        outcome = decide(hook_input)
    except Exception as error:  # noqa: BLE001 —— 自己出错不拦，免得一个缺陷挡住所有收工
        print("! 完工清理闸自己出错了，这一次没判（放行）：", file=sys.stderr)
        traceback.print_exc(file=sys.stderr)
        report_unjudged(hook_input, f"自己出错（{type(error).__name__}: {error}）", sys.stderr)
        return 0
    action = outcome["action"]
    if action == "pass" and outcome.get("notes"):
        report_unjudged_pass(hook_input, outcome["notes"], sys.stderr)
        return 0
    if action == "skip":
        report_unjudged(hook_input, outcome["text"], sys.stderr)
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
