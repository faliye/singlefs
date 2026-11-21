"""先验后换（入口 hook-swap.py）的一块：常量与弄坏开关集合、内置喂法、定点替换的解析与套用、钩子与脚本的静态检查、喂法的验证与只喂不换、草稿树的检查。
入口 hook-swap.py 把这一块的名字再导出。
"""

import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from typing import NamedTuple
import lib_repository_copy as repository_copy  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))


BREAK_VARIABLE = "HOOK_SWAP_BREAK"
KNOWN_BREAK_MODES = ("swap-on-draft-red", "skip-embedded-python", "skip-base-check", "skip-post-swap-check", "rollback-whole-file", "stale-builtin-feed",
                     "brace-feeds-dropped", "script-verify-skipped", "script-paths-unrestricted", "skip-caller-names",
                     "outcome-start-omitted", "outcome-complete-first", "pending-not-checked", "outcome-reader-trusts-file",
                     "feed-target-unchecked", "script-gate-fixtures-accepted", "script-server-target-accepted", "script-server-src-accepted",
                     "neutral-base-skipped", "running-swap-not-checked", "swap-process-pid-only", "subscript-key-ignored", "registry-read-unseen", "from-import-only-skipped",
                     "library-by-stem-only", "registry-get-ignored", "registry-alias-ignored", "heredoc-continuation-ignored", "imported-sys-path-ignored",
                     "environment-default-unfolded", "imported-constants-unfolded", "unrecognized-loads-unlisted", "module-attribute-key-unfolded",
                     "function-local-names-ignored", "several-choices-unfolded", "known-tail-unfolded", "vague-directories-ignored",
                     "scratch-lock-skipped", "scratch-lock-keyed-by-parent", "dead-holder-blocks", "scratch-lock-kept-on-exit", "scratch-lock-term-unhandled",
                     "scratch-lock-no-heartbeat", "elsewhere-holder-ignored", "elsewhere-holder-never-expires", "scratch-record-overwrites",
                     "repository-lock-keyed-by-scratch", "seal-outcome-lock-skipped", "seal-outcome-running-unchecked", "stale-record-renamed-away", "link-reply-lost-refused",
                     "taking-claim-unjudged", "taking-claim-kept-on-term", "taking-reply-lost-refused", "taking-holder-unnamed",
                     "taking-reap-unclaimed", "taking-no-heartbeat", "taken-over-unchecked")
REPOSITORY_LOCK_MODES = ("repository-lock-keyed-by-scratch", "taking-claim-unjudged", "taking-claim-kept-on-term", "taking-reply-lost-refused",
                         "taking-holder-unnamed", "taking-reap-unclaimed", "taking-no-heartbeat",
                         "taken-over-unchecked")   # 弄坏开关里波及按仓的占用记录的（摘旧记录的几个两份记录都走同一段）；别的占用类开关只波及草稿目录那一份
HOOKS_DIRECTORY = os.path.join(".claude", "hooks")
# 脚本规格收的目录（相对仓根，任意层）与其中不收的子目录 → 收它的弄坏开关（文件头 <规格.json>）。
# .claude/hooks/write-guard.sh 第十一道的 SWAP_ONLY_DIRECTORIES 是同一组目录；这里不收的 target/ 在那边是 SWAP_ONLY_EXCLUDED，src/ 在那边是 SWAP_ONLY_SOURCE_DIRECTORIES
# （就地改照拒、出路是仓副本与补丁）。两边各写一份、改的时候一起改
SCRIPT_SPEC_DIRECTORIES = ("scripts", os.path.join(".claude", "gate.d"), os.path.join(".claude", "scripts"), "task-server", "prune-server", "gpu-verification-server")
SCRIPT_SPEC_EXCLUDED = {
    os.path.join(".claude", "gate.d", "fixtures"): "script-gate-fixtures-accepted",
    os.path.join("prune-server", "target"): "script-server-target-accepted",
    os.path.join("gpu-verification-server", "target"): "script-server-target-accepted",
    os.path.join("prune-server", "src"): "script-server-src-accepted",
    os.path.join("gpu-verification-server", "src"): "script-server-src-accepted",
}
VERIFY_TIMEOUT_SECONDS = 900
WORK_DIRECTORY_NAME = "hook-swap"
FEED_TIMEOUT_SECONDS = 120
FEED_EXIT_CODES = (0, 2)
# 钩子坏了、照常放行时 stderr 里说的话（project-code-guard.sh 的「没判成」与「读不到共用模块」）
BROKEN_HOOK_MARKERS = ("没判成", "读不到共用模块")
# .sh 里喂给 python 的 heredoc 起头那一行：python / python3 …… <<'PY'（<<- 去掉行首制表符）
EMBEDDED_PYTHON_START = re.compile(r"\bpython3?\b[^\n]*?<<(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
EXIT_SWAPPED, EXIT_DRAFT_RED, EXIT_BAD_INPUT, EXIT_CHANGED, EXIT_SWAP_FAILED, EXIT_ROLLED_BACK, EXIT_ROLLBACK_FAILED, EXIT_PENDING_OUTCOME = 0, 1, 2, 3, 4, 5, 6, 7
EXIT_NEUTRAL_FEED_STALE = 8
EXIT_PREVIOUS_SWAP_RUNNING = 9
EXIT_SCRATCH_BUSY = 10
OUTCOME_NAME = "singlefs-hook-swap-outcome.tsv"   # <git-dir> 下的结局文件（文件头「结局文件」）；仓根不是 git 工作树的根时放 <草稿目录>/hook-swap/outcome.tsv
# 开始行的各段（文件头「结局文件」）：parse_outcome 按段名取下标，段数不等的不是这一版写的结局文件
START_ROW_COLUMNS = ("开始", "仓根", "进程号", "开始秒数", "反向规格路径", "反向规格sha256", "机器", "开机", "进程启动刻")
OUTCOME_COMMENTS = (
    "# 上面是这份结局文件的开始行；格式见 scripts/gate/hook-swap.py 文件头「结局文件」，制表符分隔，每行第一段是行种类：",
    "#   " + " ".join(START_ROW_COLUMNS) + " | 文件 路径 换前sha256 换后sha256 | 回读 路径 回读算出的sha256 | 完成 份数 秒数 | 退回 份数 秒数",
    "# 没有完成也没有退回就是没换完：对着主工作区现算每份文件的 sha256，等于换后的已换上，等于换前的没换上（hook-swap.py --read-outcome）",
)
WRITER = "implementation-writer"
# 花括号与参数里的通配照 bash 展开再判（钩子自己的自证表 project-code-guard.sh「花括号与参数里的通配照 bash 展开再判」那几格，C644）：
# 换上的钩子在这六条上退回放行，换之前的先验照样过，所以内置喂法里要有。路径写现位置 scripts/mutation/；正常写法用仓里真有的路径（stale_feed_paths 要展开得到）
BRACE_AND_PARAMETER_GLOB_FEEDS = [
    {"label": "实现员花括号起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutate.{sh,x} crates/mutations.tsv", "exit": 2},
    {"label": "实现员花括号起 crash-point-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/crash-point-replay.{sh,} {root}", "exit": 2},
    {"label": "实现员参数里的通配点到 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutate.sh crates/mutation?.tsv", "exit": 2},
    {"label": "实现员花括号写在包装上（{bash,-x}）照拒", "agent_type": WRITER, "command": "{bash,-x} scripts/mutation/mutate.sh crates/mutations.tsv", "exit": 2},
    {"label": "实现员合法的花括号 ls 放行", "agent_type": WRITER, "command": "ls scripts/{kb,gate}/*.py", "exit": 0, "neutral": True},
    {"label": "实现员合法的参数通配 cat 放行", "agent_type": WRITER, "command": "cat crates/*.toml", "exit": 0, "neutral": True},
]
# 内置喂法（钩子相对仓根的路径 → 喂法）。neutral 的几条新旧两版判得一样（回滚之后拿它们核钩子没坏）；
# 别的几条钉的是这一版的判法（命令词带通配的照展开判，test-script-align-r2 判决 U1-1），旧版喂了会红
BUILTIN_FEEDS = {
    ".claude/hooks/project-code-guard.sh": [
        {"label": "主 agent 跑 ls -la 放行", "command": "ls -la", "exit": 0, "neutral": True},
        {"label": "实现员跑 git status 放行", "agent_type": WRITER, "command": "git status --short", "exit": 0, "neutral": True},
        {"label": "实现员跑 cargo test --all 照拒", "agent_type": WRITER, "command": "cargo test --all", "exit": 2, "neutral": True},
        {"label": "实现员跑 验证包照拒", "agent_type": WRITER, "command": "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification",
         "exit": 2, "neutral": True},
        {"label": "实现员单独跑自证照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "python3 scripts/gate/admission.py --selftest", "exit": 2, "neutral": True},
        {"label": "实现员 ls 合法的通配放行", "agent_type": WRITER, "command": "ls scripts/*/*.sh", "exit": 0},
        {"label": "实现员通配起 crash-point-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/crash-point-replay.s? {root}", "exit": 2},
        {"label": "实现员通配起 QEMU 那一道照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/qemu-device-streams.s? {root}", "exit": 2},
        {"label": "实现员通配起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutate.s? crates/mutations.tsv", "exit": 2},
        {"label": "实现员通配起 vm-bench.sh --selftest 照拒", "agent_type": WRITER, "command": "bash scripts/devices/vm-bench*.sh --selftest", "exit": 2},
        {"label": "实现员通配起登记成重的自证照拒", "agent_type": WRITER, "command": "bash scripts/devices/vm-bench.s? --selftest", "exit": 2},
        # 本仓的脚本写成通配起，展开之后照样是直接跑项目代码，一律拒
        {"label": "实现员通配起 compile-then-swap 自证照拒", "agent_type": WRITER, "command": "python3 scripts/hosts/compile-then-swap.p? --selftest", "exit": 2},
        {"label": "实现员通配起双机驱动自证照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification-two-host?.sh --selftest", "exit": 2},
        # 带 POSIX 字符类、等价类、整理符号的交给 bash 展开；崩溃放量三份入口是本仓的脚本（test-script-align-r3 攻方 A1、A2）
        {"label": "实现员 POSIX 字符类起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutat[[:alpha:]].sh crates/mutations.tsv", "exit": 2},
        {"label": "实现员等价类起 crash-point-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/crash-point-replay.s[[=h=]] {root}", "exit": 2},
        {"label": "实现员整理符号起 QEMU 那一道照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/qemu-device-streams.s[[.h.]] {root}", "exit": 2},
        {"label": "实现员合法的 POSIX 字符类 ls 放行", "agent_type": WRITER, "command": "ls scripts/*/[[:alpha:]]*.sh", "exit": 0, "neutral": True},
        {"label": "实现员起崩溃放量全量照拒", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh", "exit": 2},
        {"label": "实现员起崩溃放量分项驱动照拒", "agent_type": WRITER,
         "command": "bash scripts/crash/crash-amplification-item.sh crash_amplification_fixed_script_through_rollback", "exit": 2},
        {"label": "实现员起崩溃放量双机驱动照拒", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification-two-hosts.sh", "exit": 2},
        {"label": "实现员单独跑 crash-amplification.sh --selftest 照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh --selftest",
         "exit": 2, "neutral": True},
        {"label": "实现员列崩溃放量的分项照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh --list-items", "exit": 2, "neutral": True},
        # 本仓之外的外部程序要经 task-server 起（lib_project_code_runs.py 的 EXTERNAL_PROGRAM_NAMES），只问版本的放行
        {"label": "实现员直接起 herd7 照拒", "agent_type": WRITER, "command": "herd7 x.litmus", "exit": 2},
        {"label": "实现员直接起 qemu-system-x86_64 照拒", "agent_type": WRITER, "command": "qemu-system-x86_64 -nographic", "exit": 2},
        {"label": "主 agent 直接起 herd7 照拒", "command": "herd7 x.litmus", "exit": 2},
        {"label": "实现员只问 herd7 的版本放行", "agent_type": WRITER, "command": "herd7 -version", "exit": 0},
        # 管道里读 JSON、往标准输出打字的 python3 -c，代码里的 'task' 字样不算写了后面执行的 task-server/task；真往它写了再执行的照拒
        {"label": "主 agent 管道读 JSON 的 python3 -c 提到 'task'，后面再起 task-server/task：放行",
         "command": "task-server/task wait 7191 >/dev/null 2>&1; task-server/task status 7191 --json | python3 -c \"import json,sys; d=json.load(sys.stdin); print(d['task'].get('compiles'))\"; "
                    "task-server/task log 7191 --all 2>&1 | head -6", "exit": 0},
        {"label": "主 agent 循环里管道读 JSON 的 python3 -c 提到 'task'，后面再起 task-server/task：放行",
         "command": "for n in 6778 6780; do task-server/task status $n --json | python3 -c \"import sys,json; d=json.load(sys.stdin); print(d['task'].get('compiles'))\"; done; "
                    "task-server/task status 6778 --json | python3 -c \"import sys,json; print(sorted(json.load(sys.stdin)['attempts'][0].keys()))\"", "exit": 0},
        {"label": "主 agent python3 -c 往 task-server/task 写了再执行它：照拒", "command": "python3 -c \"open('task-server/task','w').write('true')\"; task-server/task status 7191",
         "exit": 2, "neutral": True},
        *BRACE_AND_PARAMETER_GLOB_FEEDS,
    ],
}


def builtin_feeds_for(hook_relative, mode):
    """钩子相对仓根的路径 → 内置喂法，没有交 None。brace-feeds-dropped 下去掉花括号与参数通配那六条（只给自证判红用）。"""
    feeds = BUILTIN_FEEDS.get(hook_relative)
    if feeds is not None and mode == "brace-feeds-dropped":
        feeds = [feed for feed in feeds if feed not in BRACE_AND_PARAMETER_GLOB_FEEDS]
    return feeds


def stale_feed_paths(feeds, root):
    """喂法命令里 scripts/、.claude/ 开头的词（带通配的交 bash 展开）在 root 下一个都对不上的：[(label, 词)]。
    脚本搬进分类子目录之后旧路径只会让「该拒」的喂法退 0、「该放」的喂法空放，不报错。"""
    stale = []
    for feed in feeds:
        for word in str(feed.get("command", "")).split():
            if not word.startswith(("scripts/", ".claude/")):
                continue
            # 花括号要先于变量展开才展得开，所以这里 eval；词只收路径里会出现的字符，别的字符（$、;、引号、反引号）不 eval
            if not re.fullmatch(r"[A-Za-z0-9_./*?\[\]{},:=+@-]+", word):
                stale.append((feed.get("label", "?"), word))
                continue
            found = subprocess.run(["bash", "-c", 'shopt -s nullglob; eval "found=($1)"; [ -e "${found[0]:-}" ]', "bash", word], cwd=root, stdin=subprocess.DEVNULL,
                                   capture_output=True)
            if found.returncode != 0:
                stale.append((feed.get("label", "?"), word))
    return stale


class BadInput(Exception):
    """参数、路径或规格不对：str 是两行，第二行以「→」开头写下一步。"""


class Edit(NamedTuple):
    number: int     # 规格里第几处
    relative: str   # 相对仓根的路径
    path: str       # 主工作区里的绝对路径（解过符号链接）
    old: str
    new: str
    count: int


class RehearsalStoppedBeforeRename(Exception):
    """预演：临时文件写完、fsync 完，在改名换上之前停下。"""


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise BadInput(f"{BREAK_VARIABLE}={mode} 不认识，一个字节没写\n"
                       f"→ 怎么办：弄坏开关只认 {'、'.join(KNOWN_BREAK_MODES)}；平时不设这个变量")
    if mode:
        print(f"  ! {BREAK_VARIABLE}={mode} 生效：故意走坏的写法，只给自证判红用", file=sys.stderr)
    return mode


def is_inside(path, directory):
    return os.path.realpath(path).startswith(os.path.realpath(directory) + os.sep)


def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


def read_text(path):
    return read_bytes(path).decode("utf-8")


def sha256_of(data):
    return hashlib.sha256(data if isinstance(data, bytes) else data.encode("utf-8")).hexdigest()


def script_spec_problem(path, root, mode):
    """脚本规格收不收 path（解过符号链接的绝对路径）：收交 None，不收交一句为什么。
    弄坏开关（只给自证判红用）：script-paths-unrestricted 下仓根之下都收；SCRIPT_SPEC_EXCLUDED 里每一处在它自己那条开关下也收。"""
    real_root = os.path.realpath(root)
    if mode == "script-paths-unrestricted":
        return None if is_inside(path, real_root) else "不在仓根下"
    for excluded, switch in SCRIPT_SPEC_EXCLUDED.items():
        if mode != switch and is_inside(path, os.path.join(real_root, excluded)):
            return f"在 {excluded}/ 下，脚本规格不收这一处"
    if any(is_inside(path, os.path.join(real_root, directory)) for directory in SCRIPT_SPEC_DIRECTORIES):
        return None
    return "不在脚本规格目录下"


def resolve_edits(spec, root, mode=""):
    """规格 → [Edit]；路径不在 .claude/hooks/ 这一层、也不在脚本规格目录下（script_spec_problem）、不是已有的普通文件、钩子与脚本混在一份里、旧串或新串为空、
    count 不是正整数，抛 BadInput。mode 是弄坏开关（script_spec_problem 认的那几条，只给自证判红用）。"""
    if not isinstance(spec, list) or not spec:
        raise BadInput("规格不是非空的 JSON 数组\n→ 怎么办：写成 [{\"file\": …, \"old\": …, \"new\": …}, …]（同 replace-batch.py）")
    hooks = os.path.realpath(os.path.join(root, HOOKS_DIRECTORY))
    edits = []
    for number, item in enumerate(spec, 1):
        if not isinstance(item, dict) or not all(isinstance(item.get(key), str) for key in ("file", "old", "new")):
            raise BadInput(f"规格第 {number} 处缺 file、old、new 之一，或不是字符串\n→ 怎么办：每处写 {{\"file\", \"old\", \"new\", \"count\"（可省）}}")
        count = item.get("count", 1)
        if not isinstance(count, int) or isinstance(count, bool) or count < 1:
            raise BadInput(f"规格第 {number} 处的 count={count!r} 不是正整数\n→ 怎么办：省掉 count（默认 1），或写这处旧串该命中的次数")
        if not item["old"] or not item["new"]:
            raise BadInput(f"规格第 {number} 处的旧串或新串是空的：空串回滚时找不回原处\n→ 怎么办：两边都带上前后文（删一段就把它前后的一行一起写进旧串与新串）")
        written = item["file"] if os.path.isabs(item["file"]) else os.path.join(root, item["file"])
        path = os.path.realpath(written)
        problem = None if os.path.dirname(path) == hooks else script_spec_problem(path, root, mode)
        if problem or not os.path.isfile(path):
            raise BadInput(f"规格第 {number} 处的文件 {item['file']} 不是 {hooks} 这一层、也不是脚本规格目录下已有的普通文件（{problem or '不是已有的普通文件'}）\n"
                           f"→ 怎么办：这条脚本只换 .claude/hooks/ 下的钩子、共用模块与登记表，和 {'、'.join(directory + '/' for directory in SCRIPT_SPEC_DIRECTORIES)} 下的脚本"
                           f"（不含 {'、'.join(excluded + '/' for excluded in SCRIPT_SPEC_EXCLUDED)}）；门禁样本、编译产物与 Rust 源码照各自的办法改，新文件先排他建好再换")
        edits.append(Edit(number, os.path.relpath(path, root), path, item["old"], item["new"], count))
    kinds = {is_script_edit(edit, root) for edit in edits}
    if len(kinds) > 1:
        raise BadInput("规格里既有 .claude/hooks/ 下的钩子文件、又有脚本规格目录下的脚本：两种验法不同（钩子拿 JSON 喂，脚本换上之后跑 --verify 的命令），混在一份里换不了\n"
                       "→ 怎么办：拆成两份规格各跑一次；脚本规格先换（它不影响钩子），钩子规格后换")
    return edits


def is_script_edit(edit, root):
    """这一处的文件不在 .claude/hooks/ 这一层：按脚本规格的验法办。"""
    return os.path.dirname(edit.path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY))


def apply_edits(texts, edits, reverse=False):
    """依次施加（reverse 时新旧对调、次序倒排），每处要恰好命中 count 次；交 (新文本表, None) 或 (None, 问题一句)。"""
    texts = dict(texts)
    for edit in (reversed(edits) if reverse else edits):
        if edit.relative not in texts:
            continue
        old, new = (edit.new, edit.old) if reverse else (edit.old, edit.new)
        hits = texts[edit.relative].count(old)
        if hits != edit.count:
            return None, (f"第 {edit.number} 处 {edit.relative}：{'倒着施加时' if reverse else ''}期望命中 {edit.count} 次，实际 {hits}："
                          f"{old[:80]!r}")
        texts[edit.relative] = texts[edit.relative].replace(old, new)
    return texts, None


def continues_on_next_line(line):
    """shell 的一行以奇数个反斜杠收尾，就和下一行接成一行。"""
    return (len(line) - len(line.rstrip("\\"))) % 2 == 1


def embedded_python_segments(text, mode=""):
    """.sh 里喂给 python 的 heredoc：[(正文前一行的行号, 正文, 有没有收尾行)]。注释行里的不算。
    起头那一行以 \\ 收尾的，heredoc 正文从接上的最后一行的下一行起（那一行是正文前一行）；没有续行的就是起头那一行。
    弄坏开关 heredoc-continuation-ignored：续行也当正文（续行里的 || fail … 被当成 python）。"""
    lines, segments, index = text.split("\n"), [], 0
    while index < len(lines):
        match = EMBEDDED_PYTHON_START.search(lines[index])
        if match is None or lines[index].lstrip().startswith("#"):
            index += 1
            continue
        strip_tabs, delimiter = match.group(1) == "-", match.group(3)
        header_end = index
        while mode != "heredoc-continuation-ignored" and header_end + 1 < len(lines) and continues_on_next_line(lines[header_end]):
            header_end += 1
        body, cursor = [], header_end + 1
        while cursor < len(lines) and (lines[cursor].lstrip("\t") if strip_tabs else lines[cursor]) != delimiter:
            body.append(lines[cursor])
            cursor += 1
        segments.append((header_end + 1, "\n".join(body) + "\n", cursor < len(lines)))
        index = cursor + 1
    return segments


def static_problems(path, label, mode):
    """.sh：bash -n 与内嵌 python 逐段编译；.py：整份编译。交 (问题列表, 编了几段内嵌 python)。"""
    problems, segments_compiled = [], 0
    try:
        text = read_text(path)
    except (OSError, UnicodeDecodeError) as error:
        return [f"{label}：读不了（{error}）"], 0
    if path.endswith(".sh"):
        checked = subprocess.run(["bash", "-n", path], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if checked.returncode != 0:
            problems.append(f"{label}：bash -n 不过（{(checked.stderr.strip().splitlines() or [''])[0][:200]}）")
        if mode != "skip-embedded-python":
            for number, (start, body, closed) in enumerate(embedded_python_segments(text, mode), 1):
                segments_compiled += 1
                if not closed:
                    problems.append(f"{label}：内嵌 python 第 {number} 段（第 {start} 行起）没有收尾行")
                    continue
                try:
                    compile("\n" * start + body, f"{label}（内嵌 python 第 {number} 段）", "exec")
                except (SyntaxError, ValueError) as error:
                    problems.append(f"{label}：内嵌 python 第 {number} 段（第 {start} 行起）编译不过：{getattr(error, 'msg', error)}"
                                    f"（第 {getattr(error, 'lineno', '?')} 行）")
    elif path.endswith(".py"):
        try:
            compile(text, label, "exec")
        except (SyntaxError, ValueError) as error:
            problems.append(f"{label}：编译不过：{getattr(error, 'msg', error)}（第 {getattr(error, 'lineno', '?')} 行）")
    return problems, segments_compiled


def substitute(value, root):
    if isinstance(value, str):
        return value.replace("{root}", root)
    if isinstance(value, list):
        return [substitute(item, root) for item in value]
    if isinstance(value, dict):
        return {key: substitute(item, root) for key, item in value.items()}
    return value


def hook_input_of(feed, root):
    if "input" in feed:
        return substitute(feed["input"], root)
    hook_input = {"tool_name": feed.get("tool_name", "Bash"), "cwd": substitute(feed.get("cwd", "{root}"), root),
                  "tool_input": {"command": substitute(feed["command"], root)}, "session_id": "hook-swap"}
    if feed.get("agent_type"):
        hook_input["agent_type"] = feed["agent_type"]
    return hook_input


def validate_feeds(feeds):
    if not isinstance(feeds, list) or not feeds:
        raise BadInput("喂法不是非空的 JSON 数组\n→ 怎么办：写成 [{\"label\": …, \"command\": …, \"exit\": 0 或 2}, …]")
    for number, feed in enumerate(feeds, 1):
        if (not isinstance(feed, dict) or not isinstance(feed.get("label"), str) or feed.get("exit") not in FEED_EXIT_CODES
                or not (isinstance(feed.get("command"), str) or isinstance(feed.get("input"), dict))):
            raise BadInput(f"喂法第 {number} 条缺 label、exit（0 或 2）、command 或 input\n"
                           "→ 怎么办：每条写 {\"label\": 说明, \"command\": 命令, \"exit\": 0 或 2}，可加 agent_type、cwd、neutral")


def feed_target_problem(feed, hook_input, root, mode):
    """喂法的 tool_input.file_path 解开符号链接之后不在 root 里：交一句点名这一条与出路的话；在里面、没写 file_path 交 None。
    草稿树里只有 .claude/hooks/ 是实拷的，仓根与 .claude/ 下别的项是指回主工作区的符号链接；按落点判的钩子（write-guard.sh）把树外的目标一律当仓外放行，
    落在链接上的喂法就是空过。弄坏开关 feed-target-unchecked 下不核。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) else None
    file_path = tool_input.get("file_path") if isinstance(tool_input, dict) else None
    if mode == "feed-target-unchecked" or not isinstance(file_path, str) or not file_path:
        return None
    tree = os.path.realpath(root)
    landing = os.path.realpath(os.path.normpath(file_path if os.path.isabs(file_path) else os.path.join(root, file_path)))
    if landing == tree or landing.startswith(tree + os.sep):
        return None
    return (f"喂「{feed['label']}」：file_path {file_path} 解开符号链接之后落在 {landing}，不在 {tree} 里；按落点判的钩子把树外的目标当仓外放行，这一条喂法等于没判，没喂；"
            f"出路：file_path 写成 {{root}}/.claude/hooks/<名字>（草稿树里只有 .claude/hooks/ 是实拷的，别的目录是指回主工作区的符号链接）")


def feed_environment(root, detections_path):
    """喂钩子的环境：草稿树（符号链接拼的、不是 git 工作树）里起钩子，多机配置照 lib_repository_copy.py 的 environment_for_commands_in_the_copy
    显式交给 SINGLEFS_MULTI_HOST_CONFIG（取这份脚本所在的主工作区那一份），不靠钩子照树的位置找。"""
    environment = repository_copy.environment_for_commands_in_the_copy(REPOSITORY_ROOT)
    environment.update(CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections_path, PYTHONDONTWRITEBYTECODE="1")
    return environment


def feed_only(hook, root, feeds=None):
    """只喂不换：拿 root 下此刻的钩子逐条喂，一条一行 ✓/✗ 加实际退出码；交 (退出码, 提示行)。检出记录写临时目录、用完就删，仓里一个字节不写。"""
    mode = break_mode()
    root = os.path.realpath(root)
    hook_path = os.path.realpath(hook if os.path.isabs(hook) else os.path.join(root, hook))
    if os.path.dirname(hook_path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY)) or not hook_path.endswith(".sh") or not os.path.isfile(hook_path):
        raise BadInput(f"--hook {hook} 不是 {os.path.join(root, HOOKS_DIRECTORY)} 这一层里已有的 .sh\n→ 怎么办：--hook 写要喂 JSON 的那一份钩子（例如 .claude/hooks/project-code-guard.sh）")
    hook_relative = os.path.relpath(hook_path, root)
    if feeds is None:
        feeds = builtin_feeds_for(hook_relative, mode)
        if feeds is None:
            raise BadInput(f"{hook_relative} 没有内置喂法\n→ 怎么办：--feeds 给一份喂法（放行几条、该拒几条，写法见文件头）")
    validate_feeds(feeds)
    # 起的是 <root>/.claude/hooks/ 这一层里点名的那一份（上面核过）：路径照「钩子目录 + 文件名」拼，函数单元库把这一层的钩子都算进闭包
    hook_file = os.path.join(root, HOOKS_DIRECTORY, os.path.basename(hook_path))
    lines, wrong = [], 0
    with tempfile.TemporaryDirectory(prefix="hook-swap-feed-only-") as scratch:
        environment = feed_environment(root, os.path.join(scratch, "detections.jsonl"))
        for feed in feeds:
            hook_input = hook_input_of(feed, root)
            target_problem = feed_target_problem(feed, hook_input, root, mode)
            if target_problem:
                wrong += 1
                lines.append(f"  ✗ {target_problem}")  # gate-lint:detail
                continue
            command = hook_input.get("tool_input", {}).get("command", "") if isinstance(hook_input, dict) else ""
            try:
                done = subprocess.run(["bash", hook_file], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
                                      env=environment, cwd=root, timeout=FEED_TIMEOUT_SECONDS)
                code, stderr = done.returncode, done.stderr
            except subprocess.TimeoutExpired:
                code, stderr = "超时", ""
            marker = next((marker for marker in BROKEN_HOOK_MARKERS if marker in stderr), None)
            good = code == feed["exit"] and marker is None
            wrong += 0 if good else 1
            lines.append(f"  {'✓' if good else '✗'} 退 {code}（应当 {feed['exit']}）{'：stderr 里有「' + marker + '」' if marker else ''} {feed['label']} ← {command}")
    lines.append(f"{'✗' if wrong else '✓'} 只喂不换：{hook_relative} 喂了 {len(feeds)} 条，{wrong} 条不对")  # gate-lint:summary
    if wrong:
        lines.append("→ 怎么办：不对的几条是钩子此刻的判法与喂法对不上；先看是钩子退回了、还是喂法写错了，再决定改哪边（这个模式不改任何文件）")
    return (EXIT_DRAFT_RED if wrong else EXIT_SWAPPED), lines


def feed_problems(hook_path, root, feeds, log_path, detections_path, only_neutral=False, mode=""):
    """逐条喂 hook_path（<root>/.claude/hooks/ 这一层里的一份）；交 (问题列表, 喂了几条)。目标落在 root 之外的喂法记一条问题、不喂（feed_target_problem）。"""
    environment = feed_environment(root, detections_path)
    hook_file = os.path.join(root, HOOKS_DIRECTORY, os.path.basename(hook_path))  # 照「钩子目录 + 文件名」拼，函数单元库把这一层的钩子都算进闭包
    problems, fed = [], 0
    with open(log_path, "a", encoding="utf-8") as log:
        for feed in feeds:
            if only_neutral and not feed.get("neutral"):
                continue
            hook_input = hook_input_of(feed, root)
            target_problem = feed_target_problem(feed, hook_input, root, mode)
            if target_problem:
                problems.append(target_problem)
                log.write(f"{hook_path}\t{feed['label']}\t目标在草稿树外，没喂\n")
                continue
            fed += 1
            try:
                done = subprocess.run(["bash", hook_file], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
                                      env=environment, cwd=root, timeout=FEED_TIMEOUT_SECONDS)
            except subprocess.TimeoutExpired:
                problems.append(f"喂「{feed['label']}」：{FEED_TIMEOUT_SECONDS} 秒没判完")
                log.write(f"{hook_path}\t{feed['label']}\t超时\n")
                continue
            log.write(f"{hook_path}\t{feed['label']}\t退 {done.returncode}（应当 {feed['exit']}）\t{json.dumps(hook_input, ensure_ascii=False)}\n"
                      f"{done.stderr}\n")
            wrong = []
            if done.returncode != feed["exit"]:
                wrong.append(f"退 {done.returncode}（应当 {feed['exit']}）")
            marker = next((marker for marker in BROKEN_HOOK_MARKERS if marker in done.stderr), None)
            if marker:
                wrong.append(f"stderr 里有「{marker}」")
            if wrong:
                first = (done.stderr.strip().splitlines() or [""])[0]
                problems.append(f"喂「{feed['label']}」：{'；'.join(wrong)}；stderr 头一行 {first[:160]!r}")
    return problems, fed


def neutral_feeds_failing_on_base(root, hook_relative, feeds, log_path, detections_path, mode):
    """root 是拷好、还没套规格的草稿树：把标了 neutral 的喂法逐条喂给换上之前那一版；交 [(喂法, 实际退出码或「超时」, stderr 里的坏钩子标记或 None)]，对得上的不列。
    目标落在 root 之外的喂法不在这里喂（套规格之后 feed_problems 照旧点名）；没有 neutral 喂法时什么都不做。弄坏开关 neutral-base-skipped 下整步跳过。"""
    neutral = [feed for feed in feeds if feed.get("neutral")]
    if mode == "neutral-base-skipped" or not neutral:
        return []
    environment = feed_environment(root, detections_path)
    hook_file = os.path.join(root, HOOKS_DIRECTORY, os.path.basename(hook_relative))  # 照「钩子目录 + 文件名」拼，同 feed_problems
    failing = []
    with open(log_path, "a", encoding="utf-8") as log:
        for feed in neutral:
            hook_input = hook_input_of(feed, root)
            if feed_target_problem(feed, hook_input, root, mode):
                continue
            try:
                done = subprocess.run(["bash", hook_file], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
                                      env=environment, cwd=root, timeout=FEED_TIMEOUT_SECONDS)
            except subprocess.TimeoutExpired:
                failing.append((feed, "超时", None))
                log.write(f"{hook_file}\t{feed['label']}\t换上之前那一版：超时\n")
                continue
            log.write(f"{hook_file}\t{feed['label']}\t换上之前那一版：退 {done.returncode}（应当 {feed['exit']}）\t{json.dumps(hook_input, ensure_ascii=False)}\n{done.stderr}\n")
            marker = next((marker for marker in BROKEN_HOOK_MARKERS if marker in done.stderr), None)
            if done.returncode != feed["exit"] or marker:
                failing.append((feed, done.returncode, marker))
    return failing


def neutral_feed_stale_lines(failing, log_path):
    """neutral_feeds_failing_on_base 的结果 → 拒绝说明：逐条点名喂法与退出码，说是喂法失效、不是规格的毛病，带出路。"""
    lines = [f"✗ 标了 neutral 的喂法有 {len(failing)} 条对换上之前那一版在草稿树里就不成立，规格没套，主工作区一个字节没写：",  # gate-lint:summary
             *(f"  - 喂「{feed['label']}」：这条喂法对换上之前那一版在草稿树里就不成立（退 {got}，应当 {feed['exit']}）"
               f"{'，stderr 里有「' + marker + '」' if marker else ''}，是喂法失效，不是规格的毛病" for feed, got, marker in failing)]
    lines.append("→ 怎么办：把这几条喂法的 neutral 去掉、改它们的目标，或删掉这几条，再跑；常见的因：目标落在草稿树里而判法按落点分仓内仓外、工作根内外；"
                 f"拿主工作区此刻那一版核一条喂法用 hook-swap.py --feed-only --hook <钩子> --feeds <喂法.json>；喂法的整段 stderr 在 {log_path}")
    return lines


def check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode, only_neutral=False):
    """root 下这几份文件的语法检查与喂法；交 (问题列表, 一句做了什么)。hook_relative 是 None（脚本规格）时只做语法检查，不喂。"""
    problems, segments = [], 0
    checked = set(relatives) | ({hook_relative} if hook_relative else set())
    for relative in sorted(checked):
        found, compiled = static_problems(os.path.join(root, relative), relative, mode)
        problems += found
        segments += compiled
    if hook_relative is None:
        return problems, f"{len(checked)} 份文件语法检查、内嵌 python {segments} 段编译"
    fed_problems, fed = feed_problems(os.path.join(root, hook_relative), root, feeds, log_path, detections_path, only_neutral, mode)
    problems += fed_problems
    return problems, f"{len(checked)} 份文件语法检查、内嵌 python {segments} 段编译、喂了 {fed} 条"


# ---- 调用方引到的名字：换上的库里要有（载入它的变量上 <变量>.<名字> 用到的每个名字） ----
# 调用方 = 本仓里载入这些库的钩子、共用库、task-server/ 与 scripts/ 下的文件（.py 整份，.sh 只读喂给 python 的 heredoc）。
# 认「载入了哪个库、绑在哪个变量上」的写法（别的写法逐处报「本次未判」，不当成过）：
#   载入：importlib.util.spec_from_file_location("<库名>", …<库文件名>…) 再 module_from_spec；名字里带 load / import 的调用（load_module、import_module、__import__、
#         自己包的 load_sibling_module 这类），和本文件里函数体内起了这几样的函数（module("lib_x")），参数里有库名（lib_x）或库文件名（lib_x.py，含 os.path.join 里
#         单拎出来的那一段、同一层函数里变量的赋值）；import lib_x [as y]；from lib_x import …
#   绑定：赋给变量或属性（self.library）、元组两边逐项对上（a, b = module, module.shell_words）、global 声明过的名字、函数返回值、
#         append 进一个列表后按下标取出、作参数交给本文件里定义的函数（形参接着当库）、作参数交给另一份目标库里的函数（lib_x.f(…, module)，那份库里的形参接着当库）、
#         库自己导出的另一份目标库（module.shell_words 载入的是目标库 lib_shell_words）。
#   按键存取的容器（<容器>[<键>] = module 再 <容器>[<键>] 取出，sys.modules 这类）按键分开认（container_entry）：键「折得出」是写死的字符串、同一层函数里只赋过
#         一个写死字符串的变量，或 spec_from_file_location 的结果的 .name（那一处载入时给的名字）。取的键折得出：存进去时键相同的那份库算数；
#         没对上的，再看本文件里 spec_from_file_location 载入时给的名字，是这一次跟着走的库就算它，是别的库（不跟着走的）就不是被核的库、不核也不报。
#         取的键折不出、折出的名字哪份库都对不上、对上不止一份库，或容器里有键折不出的存法而取的键没对上存法的：逐处列成「本次未判」。
#         键是数（列表按下标）的照 append 那一路：容器装过哪份库，取出的就是哪份。
#         本文件没按键存过的 sys.modules（MODULE_REGISTRIES）按键取（registry_entry）：键折得出而对得上某份目标库的名字（库名、库文件名，或本文件 spec_from_file_location
#         载入时给的名字），取出的就是那份库，名字照核；折得出而哪份库都对不上的不是被核的库；折不出的（__name__ 除外）逐处列成「本次未判」。
#         取法两种都认：<容器>[<键>] 与 <容器>.get(<键>[, 缺省])；容器的名字照 import 的别名认（import sys as s 之后的 s.modules、from sys import modules 之后的 modules）。
#         键是 X.__module__、X 是 from <目标库> import X 引进来的名字：折成那份目标库的库名（fold_key），取出的就是那份库，名字照核。
#   from <目标库> import 名字：文件里没有别的绑定、也没有载入调用时照样核（names_library_without_binding），引到的名字逐个收；from <目标库> import * 列成「本次未判」。
#   载入与绑定都没认出、文本里却点名了目标库的写法，逐处列成「本次未判」（CallerFlow.unrecognized_loads）：不带 as 的带点 import（import a.b.库 绑的是 a）、
#         SourceFileLoader 这类名字以 Loader 收尾的调用、runpy 的 run_path 与 run_module，参数里有库名或库文件名；载入的库后面怎么用没跟。
#   判不到的写法（文本里没有库名或路径折不出，不读、不列）：库名拼出来的（importlib.import_module("fake_" + "target")，整份文本里没有库名）；往上两层以上的相对导入
#         （from .. import 库）按调用方自己所在的目录找，同目录里另有一份同名文件时解到那一份，不核也不列；载入的路径折不出的别的写法。
#   同名的库：库按文件名（库名）认，引到的名字照调用方运行时真解得到的那一份核（SearchPaths）。仓里（CALLER_SCAN_DIRECTORIES 下与这一次要核的库）叫这个名字的 .py 只有一份就是它；
#         不止一份时按解析次序取：调用方按路径载入时写死的那一份（spec_from_file_location 这类名字带 load / import 的调用，路径折得出）；往 sys.path 最前放的目录
#         （sys.path.insert(0, …)、sys.path[:0] = […]）；调用方自己所在的目录（.py 文件才有，.sh 里的 heredoc 没有）；往 sys.path 后面放的目录（append、extend、+=）。
#         往 sys.path 放的目录不只看调用方自己写的：它在模块级 import 进来的仓内模块（import X [as Y]、from X import …、from . import X，包括 if / try 里的；
#         X 在调用方自己的目录、往 sys.path 放的目录里找得到）在载入时往 sys.path 放的目录同样生效，那几份模块 import 进来的也算（SearchPaths.imported_reach）；
#         放在最前的排在调用方自己放在最前的之后，放在后面的排在调用方自己放在后面的之后，调用方自己所在的目录夹在两段之间。
#         路径折得出 = 写死的字符串、__file__、os.path.dirname / join / realpath / abspath / normpath、pathlib 的 Path、/、.parent、.parents[n]、.resolve()、.joinpath(…)、
#         名字在它所在的函数里（那一层没绑它就往外一层、最后是模块层）只赋过一次且没有别的绑定（形参、循环变量、import、解包）、import 进来的仓内模块的模块级常量
#         （X.NAME，按那份模块里 NAME 唯一一次的赋值折，那份模块在调用方自己的目录里找）、
#         os.environ.get(…) / os.getenv(…) / os.environ[…] 顶替的那一边照没设时的缺省折（<环境变量读> or <缺省>，缺省只有一个）；
#         折不出的（argv、形参、相对路径）认不出。
#         往 sys.path 放的目录折不出一处时，再试两种半折（SearchPaths.directory_choices），折成「几种可能」或「只知道末尾几段」记进 vague，不当成折不出：
#           几种可能 = 名字在一个函数里赋了几处，与本文件里模块层 def 的不带参数的函数有几处返回（return None 与不带值的不算）；原样交回的路径函数（normpath 这类）照里面那个看；
#           只知道末尾几段 = os.path.join / Path / / 拼的路径里，从后往前连着折得出的几个操作数拼成末尾几段（去掉开头的 ..），前面的操作数折不出（形参、argv、命令输出）。
#         半折的目录只在仓里叫这个名字的某份文件所在的目录落在其中一种里（几种可能里有这个目录，或这个目录以末尾几段收尾）时才认不出，列成「本次未判」；
#         一份都不落在里面的不挡后面的认法（同目录那一条照常取）。
#         解到的那一份不是被核的库就不核（同目录里自己那一份同名文件挡在前面的就是这样）；
#         认不出是哪一份的（路径折不出、往 sys.path 放的目录有折不出的——含 import 进来的模块放的、半折的目录里有这个名字的文件、哪个目录里都没有）列成「本次未判」，点名叫这个名字的几份文件，不当成查过。
CALLER_SCAN_DIRECTORIES = (os.path.join(".claude", "hooks"), "task-server", "scripts", os.path.join(".claude", "scripts"), os.path.join(".claude", "gate.d"))
CALLER_SKIPPED_DIRECTORIES = frozenset({"__pycache__", "target", "node_modules", ".git", "fixtures"})
LOADER_CALLEE = re.compile(r"load|import|spec_from")
HARMLESS_CALLEES = frozenset({"hasattr", "getattr", "isinstance", "id", "repr", "str", "print", "type", "callable", "bool", "len", "dir", "exec_module"})
LOADER_BODY_CALLEES = frozenset({"spec_from_file_location", "module_from_spec", "load_module", "import_module", "__import__"})
CONTAINER_METHODS = frozenset({"append", "add", "insert"})
MODULE_REGISTRIES = frozenset({"sys.modules"})   # 本文件没存过也值得按键取出来看的容器：全进程的库都登记在里面
DYNAMIC_NAME_CALLEES = frozenset({"globals", "vars", "setattr", "exec"})
FLOW_ROUNDS = 30
CROSS_FILE_ROUNDS = 5
PATH_FOLD_DEPTH = 12       # SearchPaths.fold 沿名字与调用往里追的层数上限
# SearchPaths.fold 认的路径函数（点路径照 import 别名还原之后）：原样交回唯一的参数 / 拼接全部参数 / 不带参数的方法（Path(…).resolve()）
PATH_PASS_THROUGH_CALLEES = frozenset({"os.path.realpath", "os.path.abspath", "os.path.normpath", "os.fspath", "str"})
PATH_JOIN_CALLEES = frozenset({"os.path.join", "pathlib.Path", "pathlib.PurePath", "pathlib.PosixPath"})
PATH_IDENTITY_METHODS = frozenset({"resolve", "absolute"})
ENVIRONMENT_READ_CALLEES = frozenset({"os.environ.get", "os.getenv"})   # <读环境变量> or <缺省>：SearchPaths.fold 按缺省折
UNRECOGNIZED_LOADER_CALLEES = frozenset({"run_path", "run_module"})     # 载入库而去向不跟的调用（名字以 Loader 收尾的另算）


ENTRY_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "hook-swap.py")   # 入口脚本：块文件里的 __file__ 是块自己的文件，要起入口的地方用这个
