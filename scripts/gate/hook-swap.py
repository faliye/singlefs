#!/usr/bin/env python3
# admission: always 每一次换钩子都要现判这一刻的规格、草稿与主工作区那几份，上一次换过不替这一次作保
# run-condition: command bash python3
"""先验后换：.claude/hooks/ 下的钩子与它读的共用模块、登记表，先在草稿里套上定点替换验过，再在主工作区定点换上；
换上之后再验一遍，不对就按反向定点替换回滚（只把这一次换上的几处改回去，不整份覆盖，别的会话这段时间里改的照留）。

用法：
  hook-swap.py <规格.json> --hook <钩子> --scratch <草稿目录> [--feeds <喂法.json>] [--root <仓根>] [--dry-run]
  hook-swap.py --selftest        # HOOK_SWAP_BREAK=<项> 时必须判红

<规格.json> 同 scripts/kb/replace-batch.py：JSON 数组，每项 {"file": 路径, "old": 旧串, "new": 新串, "count": 可省，默认 1}，
  同一个文件的多处按数组次序依次施加；file 写相对仓根的路径或绝对路径，只收 <仓根>/.claude/hooks/ 这一层里已有的普通文件；旧串、新串都不许空。
<钩子> 是要喂 JSON 的那一份（.claude/hooks/ 下的 .sh），不在规格里也照样做语法检查与喂法。
<喂法.json>：JSON 数组，每项 {"label": 说明, "command": Bash 命令, "exit": 0 或 2, "agent_type": 可省（省了是主 agent）, "cwd": 可省（默认 {root}），
  "neutral": 可省（true 是新旧两版判得一样的，回滚之后拿它们再核一遍）}；也可以写 "input": 一整份钩子 JSON 代替 command、cwd 与 agent_type。
  command、cwd 与 input 里的 {root} 换成这一次判的仓根（草稿里是草稿树，换上之后是主工作区）。
  不给 --feeds 时用内置的那一份（BUILTIN_FEEDS，按钩子相对仓根的路径取；没有内置的就要给）。
撤回一次换上：把规格倒过来（old 与 new 对调、次序倒排；每次换上都写一份在 <草稿目录>/hook-swap/reverse-<时刻>.json）再跑一遍，
  --feeds 给一份不含这一次新判法的喂法。

做法：
  1. 读规格里每份文件此刻的内容、记下字节；在内存里依次施加，每处要恰好命中 count 次；再从改后的文本倒着施加（新旧对调、次序倒排），
     要回得到原文——回不到（新串在改后的文件里不止 count 处）就停：反向定点替换回滚不了的规格不收。
  2. 草稿树 <草稿目录>/hook-swap/tree：.claude/hooks/ 整份拷一份（不带 __pycache__），套上改后的文本；仓根与 .claude/ 下别的项一律符号链接到主工作区
     （钩子按自己所在的位置找仓根、登记表与 preflight，链接过去判的就是主工作区的那几份）。
  3. 草稿里验：规格里的文件与 <钩子> 是 .sh 的跑 `bash -n`，再把里面喂给 python 的 heredoc（`python3 … <<'PY'`）逐段编译；.py 整份编译；
     逐条喂 <喂法>（CLAUDE_PROJECT_DIR 设成草稿树、检出记录写草稿目录、不带 SINGLEFS_HEAVY_TESTS），退出码要等于写的那一个，
     stderr 里不许有「没判成」「读不到共用模块」（钩子坏了放行时说的话）。有一条不对就停，主工作区一个字节不写。--dry-run 停在这里。
  4. 再读主工作区那几份，要与第 1 步记的逐字节相同（这段时间里别的会话改了就停，一个字节不写）；整批先预演一遍（同目录临时文件写完、fsync，
     改名之前停下），全过了逐份改名换上（scripts/kb/lib_atomic_replace.py），回读。
  5. 主工作区里照第 3 步再验一遍（CLAUDE_PROJECT_DIR 设成仓根）。不对就回滚：逐份在此刻的内容上做反向定点替换（每处要恰好命中 count 次），
     改名换上；再对回滚后的文件做语法检查、喂标了 neutral 的几条。
  草稿树跑完就删；喂法的输出与检出记录留在 <草稿目录>/hook-swap/ 下。

钩子在 bash 这一层坏了时，每一条 Bash 调用都会被它拒（退 2），连改回去的那一条也起不来：这条脚本在同一个进程里回滚，不靠下一条 Bash；
退出码 6 时只剩 Edit / Write 能改，提示行逐处写出要改回的地方，反向规格在草稿目录里。

退出码：0 换上了、换上之后验过（--dry-run：草稿里验过）；1 草稿里验不过（一个字节没写）；2 参数、路径或规格不对（一个字节没写）；
3 草稿验过之后主工作区那几份被改了（一个字节没写）；4 换上时失败（已换上的几份回滚了）；
5 换上之后验不过、已按反向定点替换回滚、回滚后验过；6 回滚不成或回滚后还验不过（看提示行）。

弄坏开关（只给自证判红用，生效时往 stderr 打一行）：HOOK_SWAP_BREAK=
  swap-on-draft-red      草稿里验不过也换上：「草稿红不换」三格红
  skip-embedded-python   不编译 .sh 里喂给 python 的 heredoc：「内嵌 python 编译不过」那一格红（喂法也红，但说明里点不出是哪一段）
  skip-base-check        换上之前不核主工作区那几份在验的时候改没改：「验的时候主工作区那份被改了」那一格红
  skip-post-swap-check   换上之后不再验：「换上之后红回滚」那一格红
  rollback-whole-file    回滚时整份写回换上之前的内容（盖掉别的会话这段时间里的改动）：「换上之后红回滚」那一格红
不认识的值直接拒绝，免得拼错的开关被当成「没开」。
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from typing import NamedTuple

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(SCRIPT_DIRECTORY)
sys.path[:0] = [os.path.join(REPOSITORY_ROOT, ".claude", "scripts"), SCRIPT_DIRECTORY]
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "kb"))
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402

BREAK_VARIABLE = "HOOK_SWAP_BREAK"
KNOWN_BREAK_MODES = ("swap-on-draft-red", "skip-embedded-python", "skip-base-check", "skip-post-swap-check", "rollback-whole-file")
HOOKS_DIRECTORY = os.path.join(".claude", "hooks")
WORK_DIRECTORY_NAME = "hook-swap"
FEED_TIMEOUT_SECONDS = 120
FEED_EXIT_CODES = (0, 2)
# 钩子坏了、照常放行时 stderr 里说的话（heavy-test-guard.sh 的「没判成」与「读不到共用模块」）
BROKEN_HOOK_MARKERS = ("没判成", "读不到共用模块")
# .sh 里喂给 python 的 heredoc 起头那一行：python / python3 …… <<'PY'（<<- 去掉行首制表符）
EMBEDDED_PYTHON_START = re.compile(r"\bpython3?\b[^\n]*?<<(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
EXIT_SWAPPED, EXIT_DRAFT_RED, EXIT_BAD_INPUT, EXIT_CHANGED, EXIT_SWAP_FAILED, EXIT_ROLLED_BACK, EXIT_ROLLBACK_FAILED = 0, 1, 2, 3, 4, 5, 6
WRITER = "implementation-writer"
# 内置喂法（钩子相对仓根的路径 → 喂法）。neutral 的几条新旧两版判得一样（回滚之后拿它们核钩子没坏）；
# 别的几条钉的是这一版的判法（命令词带通配的照展开判，test-script-align-r2 判决 U1-1），旧版喂了会红
BUILTIN_FEEDS = {
    ".claude/hooks/heavy-test-guard.sh": [
        {"label": "主 agent 跑 ls -la 放行", "command": "ls -la", "exit": 0, "neutral": True},
        {"label": "实现员跑 git status 放行", "agent_type": WRITER, "command": "git status --short", "exit": 0, "neutral": True},
        {"label": "实现员跑 cargo test --all 照拒", "agent_type": WRITER, "command": "cargo test --all", "exit": 2, "neutral": True},
        {"label": "实现员跑 checker 档包照拒", "agent_type": WRITER, "command": "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-checker-tier",
         "exit": 2, "neutral": True},
        {"label": "实现员单独跑登记成轻的自证放行", "agent_type": WRITER, "command": "python3 scripts/gate/admission.py --selftest", "exit": 0, "neutral": True},
        {"label": "实现员 ls 合法的通配放行", "agent_type": WRITER, "command": "ls scripts/*.sh", "exit": 0},
        {"label": "实现员通配起 checker-tier-layer0-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/checker-tier-layer0-replay.s? {root}", "exit": 2},
        {"label": "实现员通配起 QEMU 那一道照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/checker-tier-qemu-device-streams.s? {root}", "exit": 2},
        {"label": "实现员通配起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutate.s? crates/mutations.tsv", "exit": 2},
        {"label": "实现员通配起 vm-bench.sh --selftest 照拒", "agent_type": WRITER, "command": "bash scripts/vm-bench*.sh --selftest", "exit": 2},
        {"label": "实现员通配起登记成重的自证照拒", "agent_type": WRITER, "command": "bash scripts/vm-bench.s? --selftest", "exit": 2},
        # compile-then-swap.py 与双机驱动的自证换成假驱动、登记成 light 之后，通配起它们照展开判、放行（旧版登记成 heavy，喂了会红）
        {"label": "实现员通配起登记成轻的 compile-then-swap 自证放行", "agent_type": WRITER, "command": "python3 scripts/compile-then-swap.p? --selftest", "exit": 0},
        {"label": "实现员通配起登记成轻的双机驱动自证放行", "agent_type": WRITER, "command": "bash scripts/crash-amplification-two-host?.sh --selftest", "exit": 0},
        # 带 POSIX 字符类、等价类、整理符号的交给 bash 展开；崩溃放量三份入口按名字算重型（test-script-align-r3 攻方 A1、A2）
        {"label": "实现员 POSIX 字符类起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutat[[:alpha:]].sh crates/mutations.tsv", "exit": 2},
        {"label": "实现员等价类起 checker-tier-layer0-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/checker-tier-layer0-replay.s[[=h=]] {root}", "exit": 2},
        {"label": "实现员整理符号起 QEMU 那一道照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/checker-tier-qemu-device-streams.s[[.h.]] {root}", "exit": 2},
        {"label": "实现员合法的 POSIX 字符类 ls 放行", "agent_type": WRITER, "command": "ls scripts/[[:alpha:]]*.sh", "exit": 0, "neutral": True},
        {"label": "实现员起崩溃放量全量照拒", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh", "exit": 2},
        {"label": "实现员起崩溃放量分项驱动照拒", "agent_type": WRITER,
         "command": "bash scripts/crash/crash-amplification-item.sh crash_amplification_fixed_script_through_rollback", "exit": 2},
        {"label": "实现员起崩溃放量双机驱动照拒", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification-two-hosts.sh", "exit": 2},
        {"label": "实现员单独跑 crash-amplification.sh --selftest 放行", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh --selftest",
         "exit": 0, "neutral": True},
        {"label": "实现员列崩溃放量的分项放行", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh --list-items", "exit": 0, "neutral": True},
    ],
}


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


def resolve_edits(spec, root):
    """规格 → [Edit]；路径不在 .claude/hooks/ 这一层、不是已有的普通文件、旧串或新串为空、count 不是正整数，抛 BadInput。"""
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
        if os.path.dirname(path) != hooks or not os.path.isfile(path):
            raise BadInput(f"规格第 {number} 处的文件 {item['file']} 不是 {hooks} 这一层里已有的普通文件\n"
                           "→ 怎么办：这条脚本只换 .claude/hooks/ 下的钩子、共用模块与登记表；别的文件照原来的办法改，新文件先排他建好再换")
        edits.append(Edit(number, os.path.relpath(path, root), path, item["old"], item["new"], count))
    return edits


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


def embedded_python_segments(text):
    """.sh 里喂给 python 的 heredoc：[(起头那一行的行号, 正文, 有没有收尾行)]。注释行里的不算。"""
    lines, segments, index = text.split("\n"), [], 0
    while index < len(lines):
        match = EMBEDDED_PYTHON_START.search(lines[index])
        if match is None or lines[index].lstrip().startswith("#"):
            index += 1
            continue
        strip_tabs, delimiter = match.group(1) == "-", match.group(3)
        body, cursor = [], index + 1
        while cursor < len(lines) and (lines[cursor].lstrip("\t") if strip_tabs else lines[cursor]) != delimiter:
            body.append(lines[cursor])
            cursor += 1
        segments.append((index + 1, "\n".join(body) + "\n", cursor < len(lines)))
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
            for number, (start, body, closed) in enumerate(embedded_python_segments(text), 1):
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


def feed_problems(hook_path, root, feeds, log_path, detections_path, only_neutral=False):
    """逐条喂 hook_path；交 (问题列表, 喂了几条)。"""
    environment = {key: value for key, value in os.environ.items() if key != "SINGLEFS_HEAVY_TESTS"}
    environment.update(CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections_path, PYTHONDONTWRITEBYTECODE="1")
    problems, fed = [], 0
    with open(log_path, "a", encoding="utf-8") as log:
        for feed in feeds:
            if only_neutral and not feed.get("neutral"):
                continue
            fed += 1
            hook_input = hook_input_of(feed, root)
            try:
                done = subprocess.run(["bash", hook_path], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
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


def check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode, only_neutral=False):
    """root 下这几份文件的语法检查与喂法；交 (问题列表, 一句做了什么)。"""
    problems, segments = [], 0
    for relative in sorted(set(relatives) | {hook_relative}):
        found, compiled = static_problems(os.path.join(root, relative), relative, mode)
        problems += found
        segments += compiled
    fed_problems, fed = feed_problems(os.path.join(root, hook_relative), root, feeds, log_path, detections_path, only_neutral)
    problems += fed_problems
    return problems, f"{len(set(relatives) | {hook_relative})} 份文件语法检查、内嵌 python {segments} 段编译、喂了 {fed} 条"


def build_draft_tree(root, tree, planned):
    """草稿树：.claude/hooks/ 整份拷一份并套上改后的文本，仓根与 .claude/ 下别的项符号链接到主工作区。"""
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    os.makedirs(os.path.join(tree, ".claude"))
    for entry in sorted(os.listdir(root)):
        if entry not in (".claude", ".git"):
            os.symlink(os.path.join(root, entry), os.path.join(tree, entry))
    for entry in sorted(os.listdir(os.path.join(root, ".claude"))):
        if entry != "hooks":
            os.symlink(os.path.join(root, ".claude", entry), os.path.join(tree, ".claude", entry))
    shutil.copytree(os.path.join(root, HOOKS_DIRECTORY), os.path.join(tree, HOOKS_DIRECTORY), symlinks=True,
                    ignore=shutil.ignore_patterns("__pycache__"))
    for relative, text in planned.items():
        with open(os.path.join(tree, relative), "w", encoding="utf-8") as handle:
            handle.write(text)


def rehearse(path, text):
    """走一遍整条换法、停在改名之前：会被拒的在这里抛 ReplaceRefused，原文件一个字节没动。"""
    def stop_before_rename():
        raise RehearsalStoppedBeforeRename()
    try:
        replace_file_contents_by_rename(path, text, check_before_rename=stop_before_rename)
    except RehearsalStoppedBeforeRename:
        return


def manual_rollback_lines(relative, edits):
    """回滚不成时逐处写出要改回的地方（Bash 被坏钩子拒时只剩 Edit / Write）。"""
    lines = [f"    {relative} 要改回的几处（Edit：旧串写下面的「现在」，新串写「改回」）："]
    for edit in reversed([edit for edit in edits if edit.relative == relative]):
        lines.append(f"      第 {edit.number} 处 现在 {edit.new[:120]!r} → 改回 {edit.old[:120]!r}（命中 {edit.count} 次）")
    return lines


def rollback(swapped, edits, base_texts, mode):
    """逐份在此刻的内容上做反向定点替换、改名换上、回读；交 (问题行列表, 回滚了的相对路径)。"""
    problem_lines, rolled = [], []
    for relative, path in reversed(swapped):
        try:
            current = read_text(path)
        except (OSError, UnicodeDecodeError) as error:
            problem_lines += [f"  - {relative} 读不了（{error}），没回滚", *manual_rollback_lines(relative, edits)]
            continue
        if mode == "rollback-whole-file":
            restored = base_texts[relative]
        else:
            restored_texts, problem = apply_edits({relative: current}, [edit for edit in edits if edit.relative == relative], reverse=True)
            if problem:
                problem_lines += [f"  - {relative} 反向定点替换对不上（{problem}）：换上之后有人改了这几处，没回滚",
                                  *manual_rollback_lines(relative, edits)]
                continue
            restored = restored_texts[relative]
        try:
            replace_file_contents_by_rename(path, restored)
        except ReplaceRefused as error:
            problem_lines += [f"  - {relative} 写回时失败（{str(error).splitlines()[0]}），没回滚", *manual_rollback_lines(relative, edits)]
            continue
        if read_text(path) != restored:
            problem_lines += [f"  - {relative} 回滚后回读对不上：有别的进程同时在写", *manual_rollback_lines(relative, edits)]
            continue
        rolled.append(relative)
    return problem_lines, rolled


def swap(spec, hook, scratch, root, feeds=None, dry_run=False, after_draft=None, after_swap=None):
    """交回 (退出码, 提示行)。after_draft、after_swap 只给自证用：草稿验完之后、换上之后各调一次（模拟别的会话在这段时间里改文件）。"""
    mode = break_mode()
    root = os.path.realpath(root)
    scratch = os.path.abspath(scratch)
    if os.path.realpath(scratch) == root or is_inside(scratch, root):
        raise BadInput(f"草稿目录 {scratch} 在主工作区里\n→ 怎么办：放进派发提示给的草稿目录（工作根 ~/.local/share/singlefs/work/<轮>/ 这类仓外位置）")
    edits = resolve_edits(spec, root)
    hook_path = os.path.realpath(hook if os.path.isabs(hook) else os.path.join(root, hook))
    if os.path.dirname(hook_path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY)) or not hook_path.endswith(".sh") or not os.path.isfile(hook_path):
        raise BadInput(f"--hook {hook} 不是 {os.path.join(root, HOOKS_DIRECTORY)} 这一层里已有的 .sh\n→ 怎么办：--hook 写要喂 JSON 的那一份钩子（例如 .claude/hooks/heavy-test-guard.sh）")
    hook_relative = os.path.relpath(hook_path, root)
    if feeds is None:
        feeds = BUILTIN_FEEDS.get(hook_relative)
        if feeds is None:
            raise BadInput(f"{hook_relative} 没有内置喂法\n→ 怎么办：--feeds 给一份喂法（放行几条、该拒几条，写法见文件头）")
    validate_feeds(feeds)
    relatives = sorted({edit.relative for edit in edits})
    base_bytes = {relative: read_bytes(os.path.join(root, relative)) for relative in relatives}
    try:
        base_texts = {relative: data.decode("utf-8") for relative, data in base_bytes.items()}
    except UnicodeDecodeError as error:
        raise BadInput(f"规格里有文件不是 UTF-8 文本（{error}）\n→ 怎么办：这条脚本只换文本文件") from error
    planned, problem = apply_edits(base_texts, edits)
    if problem:
        raise BadInput(f"{problem}，一个字节没写\n→ 怎么办：对一下这一处的原文（空白、全半角、同义字），改规格再跑")
    restored, problem = apply_edits(planned, edits, reverse=True)
    if problem or restored != base_texts:
        raise BadInput(f"倒着施加回不到原文（{problem or '倒回来的文本与原文不同'}），这份规格换上之后没法定点回滚，一个字节没写\n"
                       "→ 怎么办：给这几处的新串多带一点前后文，让它在改后的文件里只出现 count 次")
    work = os.path.join(scratch, WORK_DIRECTORY_NAME)
    os.makedirs(work, exist_ok=True)
    stamp = f"{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{os.getpid()}"
    log_path, detections_path = os.path.join(work, f"feeds-{stamp}.log"), os.path.join(work, f"detections-{stamp}.jsonl")
    reverse_spec_path = os.path.join(work, f"reverse-{stamp}.json")
    with open(reverse_spec_path, "w", encoding="utf-8") as handle:
        json.dump([{"file": edit.relative, "old": edit.new, "new": edit.old, "count": edit.count} for edit in reversed(edits)], handle, ensure_ascii=False, indent=1)
    lines = [f"规格 {len(edits)} 处、{len(relatives)} 份文件（{'、'.join(relatives)}）；喂的钩子 {hook_relative}；喂法 {len(feeds)} 条",
             *(f"  {relative} 换上之前 sha256 {sha256_of(base_bytes[relative])}，换上之后应当是 {sha256_of(planned[relative])}" for relative in relatives),
             f"喂法输出 {log_path}；反向规格 {reverse_spec_path}"]
    tree = os.path.join(work, "tree")
    try:
        build_draft_tree(root, tree, planned)
        draft_problems, draft_summary = check_tree(tree, relatives, hook_relative, feeds, log_path, detections_path, mode)
    finally:
        shutil.rmtree(tree, ignore_errors=True)
    if draft_problems:
        lines += [f"✗ 草稿里验不过（{draft_summary}），主工作区一个字节没写：", *(f"  - {problem}" for problem in draft_problems),  # gate-lint:summary
                  f"→ 怎么办：照上面几条改规格再跑；喂法的整段 stderr 在 {log_path}"]
        if mode != "swap-on-draft-red":
            return EXIT_DRAFT_RED, lines
    else:
        lines.append(f"✓ 草稿里验过：{draft_summary}")
    if dry_run:
        lines.append("--dry-run：停在草稿，主工作区一个字节没写")
        return (EXIT_DRAFT_RED if draft_problems else EXIT_SWAPPED), lines
    if after_draft is not None:
        after_draft()
    changed = [relative for relative in relatives if read_bytes(os.path.join(root, relative)) != base_bytes[relative]]
    if changed and mode != "skip-base-check":
        lines += [f"✗ 草稿验过之后主工作区这几份被改了：{'、'.join(changed)}，一个字节没写",
                  "→ 怎么办：先看是谁改的（git diff 那几份、问主 agent）；照此刻的原文重写规格再跑，不盖掉别人的改动"]
        return EXIT_CHANGED, lines
    for relative in relatives:
        try:
            rehearse(os.path.join(root, relative), planned[relative])
        except ReplaceRefused as error:
            lines += [f"✗ 预演换上 {relative} 时被拒（{str(error).splitlines()[0]}），一个字节没写",
                      "→ 怎么办：照被拒的原因处理那一份（硬链接、只读、目录权限、空间），再跑"]
            return EXIT_SWAP_FAILED, lines
    swapped = []
    for relative in relatives:
        path = os.path.join(root, relative)

        def unchanged_since_draft(path=path, relative=relative):
            if mode != "skip-base-check" and read_bytes(path) != base_bytes[relative]:
                raise ReplaceRefused(f"{relative} 在换上前一刻被改了\n→ 怎么办：照此刻的原文重写规格再跑")
        try:
            replace_file_contents_by_rename(path, planned[relative], check_before_rename=unchanged_since_draft)
        except ReplaceRefused as error:
            problem_lines, rolled = rollback(swapped, edits, base_texts, mode)
            lines += [f"✗ 换上 {relative} 时失败（{str(error).splitlines()[0]}）；已经换上的 {len(swapped)} 份回滚了 {len(rolled)} 份", *problem_lines,
                      "→ 怎么办：照上面的原因处理那一份再跑；有回滚不成的，照写出的几处用 Edit 改回"]
            return (EXIT_ROLLBACK_FAILED if problem_lines else EXIT_SWAP_FAILED), lines
        swapped.append((relative, path))
    readback = [relative for relative in relatives if read_text(os.path.join(root, relative)) != planned[relative]]
    if after_swap is not None:
        after_swap()
    if mode == "skip-post-swap-check":
        lines.append(f"✓ 换上了 {len(swapped)} 份（弄坏开关：换上之后没再验）")
        return EXIT_SWAPPED, lines
    live_problems, live_summary = check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode)
    live_problems = [f"{relative} 换上之后回读与草稿里验过的不同" for relative in readback] + live_problems
    if not live_problems:
        lines.append(f"✓ 换上了 {len(swapped)} 份（改名换上新 inode、回读一致），主工作区里再验过：{live_summary}")
        return EXIT_SWAPPED, lines
    lines += [f"✗ 换上之后在主工作区验不过（{live_summary}）：", *(f"  - {problem}" for problem in live_problems)]  # gate-lint:summary
    problem_lines, rolled = rollback(swapped, edits, base_texts, mode)
    if problem_lines:
        lines += [f"✗ 回滚只成了 {len(rolled)} 份：", *problem_lines,
                  f"→ 怎么办：照上面写出的几处用 Edit 改回（钩子在 bash 这一层坏了时 Bash 调用会被拒，Edit / Write 不经它）；反向规格 {reverse_spec_path}"]
        return EXIT_ROLLBACK_FAILED, lines
    after_problems, after_summary = check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode, only_neutral=True)
    if after_problems:
        lines += [f"✗ 已按反向定点替换回滚 {len(rolled)} 份，回滚之后还验不过（{after_summary}）：", *(f"  - {problem}" for problem in after_problems),  # gate-lint:summary
                  f"→ 怎么办：钩子现在可能是坏的；看 {log_path} 的 stderr，用 Edit 修，交主 agent"]
        return EXIT_ROLLBACK_FAILED, lines
    lines.append(f"→ 已按反向定点替换回滚 {len(rolled)} 份（{'、'.join(rolled)}），回滚之后验过（{after_summary}）；"
                 f"照上面几条改规格，必要时 --dry-run 先看草稿，再跑；喂法的整段 stderr 在 {log_path}")
    return EXIT_ROLLED_BACK, lines


SELFTEST_GUARD = """#!/usr/bin/env bash
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 /dev/fd/3 "$HOOK_DIR" 3<<'PY'
import importlib.util, json, os, sys
spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(sys.argv[1], "fake_lib.py"))
lib = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lib)
command = json.load(sys.stdin).get("tool_input", {}).get("command", "")
if lib.refused(command):
    print("✗ 被拒：" + command, file=sys.stderr)
    sys.exit(2)
sys.exit(0)
PY
"""
SELFTEST_LIBRARY = ('MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n\n\n'
                    'def refused_v2(command):\n    return MARKER in command or "*" in command\n')
SELFTEST_LIBRARY_WITHOUT_V2 = 'MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n'
SELFTEST_FEEDS = [{"label": "轻的放行", "command": "ls -la", "exit": 0, "neutral": True},
                  {"label": "重的照拒", "command": "run HEAVY", "exit": 2, "neutral": True},
                  {"label": "新判法：通配照拒", "command": "run GLOB*", "exit": 2}]
SELFTEST_GOOD_EDIT = {"file": ".claude/hooks/fake_lib.py", "old": "    return MARKER in command\n",
                      "new": "    return MARKER in command or \"*\" in command  # 新判法\n"}
SELFTEST_OTHER_SESSION_LINE = "# 别的会话这段时间里加的一行\n"


def selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="hook-swap-selftest-")

    def fresh_repository(name):
        root = os.path.join(work, name, "repo")
        os.makedirs(os.path.join(root, HOOKS_DIRECTORY))
        os.makedirs(os.path.join(root, "research"))
        for file_name, text, file_mode in (("fake-guard.sh", SELFTEST_GUARD, 0o755), ("fake_lib.py", SELFTEST_LIBRARY, 0o644)):
            path = os.path.join(root, HOOKS_DIRECTORY, file_name)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            os.chmod(path, file_mode)
        return root

    def hooks_file(root, file_name):
        return os.path.join(root, HOOKS_DIRECTORY, file_name)

    def run(label, spec, want_code, want_files, want_text=None, after_draft=None, after_swap=None, want_new_inode=None):
        nonlocal checked
        checked += 1
        root = fresh_repository(f"case-{checked}")
        scratch = os.path.join(work, f"case-{checked}", "scratch")
        inode_before = os.stat(hooks_file(root, want_new_inode)).st_ino if want_new_inode else None
        try:
            code, lines = swap(spec, ".claude/hooks/fake-guard.sh", scratch, root, feeds=SELFTEST_FEEDS,
                               after_draft=(lambda: after_draft(root)) if after_draft else None,
                               after_swap=(lambda: after_swap(root)) if after_swap else None)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        for file_name, want in want_files.items():
            got = read_text(hooks_file(root, file_name))
            if got != want:
                problems.append(f"{file_name} 的内容不对（sha256 {sha256_of(got)[:12]}，应当 {sha256_of(want)[:12]}）")
        if want_text is not None and not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        if code not in (EXIT_SWAPPED, EXIT_BAD_INPUT) and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        leftovers = [name for name in os.listdir(os.path.join(root, HOOKS_DIRECTORY)) if name.startswith(".")]
        if leftovers:
            problems.append(f"钩子目录里留下临时文件 {leftovers}")
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if want_new_inode and os.stat(hooks_file(root, want_new_inode)).st_ino == inode_before:
            problems.append(f"{want_new_inode} 换上之后 inode 没变：在原 inode 上写的")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    good_library = SELFTEST_LIBRARY.replace(SELFTEST_GOOD_EDIT["old"], SELFTEST_GOOD_EDIT["new"], 1)
    untouched = {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": SELFTEST_LIBRARY}
    # 过：草稿与换上之后都验过，定点换上（新 inode）
    run("过：草稿里验过、换上、换上之后再验过", [SELFTEST_GOOD_EDIT], EXIT_SWAPPED, {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": good_library},
        want_text="✓ 换上了 1 份", want_new_inode="fake_lib.py")
    # 草稿红不换（开关 swap-on-draft-red 下这三格红）
    run("草稿红不换：钩子 bash 这一层的语法错（尾部多一行 if true; then）",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "\nPY\n", "new": "\nPY\nif true; then\n"}], EXIT_DRAFT_RED, untouched, want_text="bash -n 不过")
    run("草稿红不换：钩子内嵌 python 编译不过，静态检查点名那一段（开关 skip-embedded-python 下红）",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "sys.exit(0)\n", "new": "sys.exit(0\n"}], EXIT_DRAFT_RED, untouched,
        want_text="内嵌 python 第 1 段（第 4 行起）编译不过")
    run("草稿红不换：判法改错，该拒的放行了", [{"file": ".claude/hooks/fake_lib.py", "old": 'MARKER = "HEAVY"', "new": 'MARKER = "HEAVYX"'}],
        EXIT_DRAFT_RED, untouched, want_text="喂「重的照拒」：退 0（应当 2）")

    # 换上之后红回滚：验完草稿之后别的会话删了被调的函数；换上之后它又在同一份钩子里加了一行，回滚只改回这一次的那一处，那一行留着
    # （开关 skip-post-swap-check、rollback-whole-file 下红）
    def other_session_drops_v2(root):
        with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_LIBRARY_WITHOUT_V2)

    def other_session_appends(root):
        with open(hooks_file(root, "fake-guard.sh"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
    run("换上之后红回滚：反向定点替换只改回这一次的那一处，别的会话加的一行留着",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}], EXIT_ROLLED_BACK,
        {"fake-guard.sh": SELFTEST_GUARD + SELFTEST_OTHER_SESSION_LINE, "fake_lib.py": SELFTEST_LIBRARY_WITHOUT_V2},
        want_text="已按反向定点替换回滚 1 份", after_draft=other_session_drops_v2, after_swap=other_session_appends)

    # 验的时候主工作区那份被改了：不换（开关 skip-base-check 下红）
    def other_session_edits_library(root):
        with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE + SELFTEST_LIBRARY)
    run("验的时候主工作区那份被改了：一个字节不写", [SELFTEST_GOOD_EDIT], EXIT_CHANGED,
        {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": SELFTEST_OTHER_SESSION_LINE + SELFTEST_LIBRARY}, after_draft=other_session_edits_library)
    # 规格不收的两种
    run("规格里的文件不在 .claude/hooks/ 这一层：不收", [{"file": "research", "old": "a", "new": "b"}], EXIT_BAD_INPUT, untouched)
    run("新串在改后的文件里不止一处（回滚找不回原处）：不收", [{"file": ".claude/hooks/fake_lib.py", "old": '"HEAVY"', "new": "command"}],
        EXIT_BAD_INPUT, untouched)
    # 内嵌 python 的切段：注释行里的不算、<<- 去掉行首制表符、没收尾的报出来
    checked += 1
    segments = embedded_python_segments("# python3 - <<'X'\npython3 - <<-'EOF'\nprint(1)\n\tEOF\npython3 /dev/fd/3 3<<PY\nx = (\n")
    if [(start, closed) for start, _, closed in segments] != [(2, True), (5, False)]:
        failures.append(f"内嵌 python 切段不对：{[(start, closed) for start, _, closed in segments]}（应当 [(2, True), (5, False)]）")
    shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ hook-swap 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 swap() / check_tree() / rollback() / embedded_python_segments() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ hook-swap 自检通过（查了 {checked} 种）：草稿与换上之后都验过的换上新 inode；钩子 bash 层语法错、内嵌 python 编译不过、判法改错的草稿一个字节不写；"
          "换上之后验不过的按反向定点替换回滚、别的会话这段时间里加的一行留着；验的时候主工作区被改了不换；"
          "不在 .claude/hooks/ 的文件、回滚找不回原处的规格不收；内嵌 python 切段认 <<- 与没收尾的")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    usage = "hook-swap.py <规格.json> --hook <钩子> --scratch <草稿目录> [--feeds <喂法.json>] [--root <仓根>] [--dry-run]，或 hook-swap.py --selftest"
    options = {"--hook": None, "--scratch": None, "--feeds": None, "--root": REPOSITORY_ROOT}
    dry_run, positional, position = False, [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in options and position + 1 < len(arguments):
            options[argument] = arguments[position + 1]
            position += 2
            continue
        if argument == "--dry-run":
            dry_run = True
        else:
            positional.append(argument)
        position += 1
    if len(positional) != 1 or options["--hook"] is None or options["--scratch"] is None:
        print("✗ 参数不对：要一份规格、--hook 与 --scratch")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    try:
        with open(positional[0], encoding="utf-8") as handle:
            spec = json.load(handle)
        feeds = None
        if options["--feeds"] is not None:
            with open(options["--feeds"], encoding="utf-8") as handle:
                feeds = json.load(handle)
    except (OSError, ValueError) as error:
        print(f"✗ 读不了规格或喂法（{error}）")
        print("→ 怎么办：两份都是 UTF-8 的 JSON 数组，写法见文件头")
        return EXIT_BAD_INPUT
    try:
        code, lines = swap(spec, options["--hook"], options["--scratch"], options["--root"], feeds=feeds, dry_run=dry_run)
    except BadInput as error:
        problem, _, next_step = str(error).partition("\n")
        print(f"✗ {problem}")
        print(f"→ {next_step.lstrip('→ ') or usage}")
        return EXIT_BAD_INPUT
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
