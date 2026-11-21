"""先验后换（入口 hook-swap.py）的一块：草稿树的搭建与演练、回滚、结局文件、草稿目录与仓的占用记录、换上（swap）。
入口 hook-swap.py 把这一块的名字再导出。
"""

import json
import os
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import threading
import time
from typing import NamedTuple
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402

from hook_swap_base import (
     REPOSITORY_LOCK_MODES, HOOKS_DIRECTORY, VERIFY_TIMEOUT_SECONDS, WORK_DIRECTORY_NAME, EXIT_BAD_INPUT, EXIT_CHANGED, EXIT_DRAFT_RED,
     EXIT_PENDING_OUTCOME, EXIT_ROLLBACK_FAILED, EXIT_ROLLED_BACK, EXIT_SWAPPED, EXIT_SWAP_FAILED, EXIT_NEUTRAL_FEED_STALE,
     EXIT_PREVIOUS_SWAP_RUNNING, EXIT_SCRATCH_BUSY, OUTCOME_NAME, START_ROW_COLUMNS, OUTCOME_COMMENTS, builtin_feeds_for, BadInput,
     RehearsalStoppedBeforeRename, break_mode, is_inside, read_bytes, read_text, sha256_of, resolve_edits, is_script_edit, apply_edits,
     validate_feeds, neutral_feeds_failing_on_base, neutral_feed_stale_lines, check_tree,
)
from hook_swap_callers import caller_names_check, unjudged_names_lines

def build_scripts_draft_tree(tree, planned):
    """脚本规格的草稿树：只放改后的那几份（按它们在仓里的相对路径），不链接仓里别的东西，写进去的路径不会穿过链接落到主工作区。"""
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    for relative, text in planned.items():
        destination = os.path.join(tree, relative)
        os.makedirs(os.path.dirname(destination), exist_ok=True)
        with open(destination, "w", encoding="utf-8") as handle:
            handle.write(text)


def verify_problem(command, root, log_path):
    """换上之后在仓根跑 --verify 的命令：退 0 返回 None，否则返回一句问题；输出写 log_path。"""
    environment = dict(os.environ)
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    try:
        done = subprocess.run(["bash", "-c", command], cwd=root, stdin=subprocess.DEVNULL, capture_output=True, text=True, env=environment,
                              timeout=VERIFY_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        with open(log_path, "a", encoding="utf-8") as log:
            log.write(f"{command}\n超时 {VERIFY_TIMEOUT_SECONDS} 秒\n")
        return f"验证命令 {VERIFY_TIMEOUT_SECONDS} 秒没跑完：{command[:160]}"
    with open(log_path, "a", encoding="utf-8") as log:
        log.write(f"{command}\n退 {done.returncode}\n{done.stdout}\n{done.stderr}\n")
    if done.returncode == 0:
        return None
    tail = [line for line in (done.stdout + done.stderr).splitlines() if line.strip()][-3:]
    return f"验证命令退 {done.returncode}：{command[:160]}；输出末几行 {tail}"


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
    write_planned_texts(tree, planned)


def write_planned_texts(tree, planned):
    """改后的文本写进草稿树里同一相对路径的那几份（planned 为空什么都不写，树里是换上之前那一版）。"""
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


class OutcomeUnreadable(Exception):
    """结局文件读不了或不是结局文件。"""


class Outcome(NamedTuple):
    path: str
    root: str
    process: str
    reverse_spec: str
    files: list      # [(相对仓根的路径, 换前 sha256, 换后 sha256)]
    readback: dict   # 相对仓根的路径 → 回读算出的 sha256
    ending: str      # "完成" 或 "退回"；都没有是 ""
    started_seconds: int   # 开始行的开始秒数
    machine: str     # 起这一趟的那一台：/etc/machine-id 的 sha256 前 16 位，读不到是 UNKNOWN_IDENTITY
    boot: str        # 那一台那一次开机的 boot_id
    start_ticks: str   # 起这一趟的进程的启动刻（/proc/<进程号>/stat 第 22 列）


# 认出起上一趟的进程此刻在不在（文件头「结局文件」）
UNKNOWN_IDENTITY = "-"
MACHINE_ID_PATH = "/etc/machine-id"
BOOT_ID_PATH = "/proc/sys/kernel/random/boot_id"
PROC_STAT_STATE_FIELD_AFTER_COMM, PROC_STAT_START_FIELD_AFTER_COMM = 0, 19   # /proc/<pid>/stat 右括号之后的字段从第 3 列起数：状态是第 3 列，启动刻是第 22 列
EXITED_PROCESS_STATES = ("Z", "X")   # 僵尸与已死：退出了，不算还在
SWAP_PROCESS_RUNNING, SWAP_PROCESS_GONE, SWAP_PROCESS_ELSEWHERE = "还在", "不在", "在另一台"


def identity_file_value(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read().strip() or UNKNOWN_IDENTITY
    except (OSError, UnicodeError):
        return UNKNOWN_IDENTITY


def this_machine():
    """这一台的机器标识：/etc/machine-id 的 sha256 前 16 位（结局文件不留原值）；读不到交 UNKNOWN_IDENTITY。"""
    value = identity_file_value(MACHINE_ID_PATH)
    return UNKNOWN_IDENTITY if value == UNKNOWN_IDENTITY else sha256_of(value)[:16]


def process_state_and_start(pid):
    """/proc/<pid>/stat 的状态与启动刻 → (状态, 启动刻)；进程不在或读不出交 None。命令名里可以有空格与括号，从最后一个右括号往后切。
    scripts/agents/lib_session_registry.py 的 process_start_ticks 只交启动刻，这里还要状态认僵尸，所以另读一次。"""
    try:
        with open(f"/proc/{int(pid)}/stat", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except (OSError, ValueError):
        return None
    fields = text[text.rfind(")") + 2:].split()
    if len(fields) <= PROC_STAT_START_FIELD_AFTER_COMM:
        return None
    return fields[PROC_STAT_STATE_FIELD_AFTER_COMM], fields[PROC_STAT_START_FIELD_AFTER_COMM]


def swap_process_state(outcome, mode):
    """结局文件开始行记的那个进程此刻：SWAP_PROCESS_RUNNING（机器与开机都是这一台此刻的、/proc/<进程号> 在、不是僵尸、启动刻对得上）、
    SWAP_PROCESS_ELSEWHERE（机器对不上或有一边读不到：这一台看不到它）、SWAP_PROCESS_GONE（别的情形）。弄坏开关 swap-process-pid-only 下只看 /proc/<进程号> 在不在。"""
    found = process_state_and_start(outcome.process)
    if mode == "swap-process-pid-only":
        return SWAP_PROCESS_RUNNING if found is not None else SWAP_PROCESS_GONE
    here = this_machine()
    if UNKNOWN_IDENTITY in (here, outcome.machine) or outcome.machine != here:
        return SWAP_PROCESS_ELSEWHERE
    if outcome.boot != identity_file_value(BOOT_ID_PATH) or found is None:   # 这一台开过机了，或进程号上没有进程
        return SWAP_PROCESS_GONE
    state, ticks = found
    return SWAP_PROCESS_RUNNING if state not in EXITED_PROCESS_STATES and ticks == outcome.start_ticks else SWAP_PROCESS_GONE


def git_work_tree_directory(root):
    """root 自己就是 git 工作树的根时交它的 git-dir 绝对路径；不是（含在别的仓的子目录里、没有 git）交 None。"""
    try:
        top = subprocess.run(["git", "-C", root, "rev-parse", "--show-toplevel"], capture_output=True, text=True)
        done = subprocess.run(["git", "-C", root, "rev-parse", "--absolute-git-dir"], capture_output=True, text=True)
    except OSError:
        return None
    if top.returncode != 0 or done.returncode != 0 or os.path.realpath(top.stdout.strip()) != os.path.realpath(root):
        return None
    return done.stdout.strip() or None


def outcome_path(root, scratch):
    """结局文件的位置：root 是 git 工作树的根就放它的 git-dir 下（下一趟不管用哪个草稿目录都看得见），否则放 <草稿目录>/hook-swap/outcome.tsv。"""
    directory = git_work_tree_directory(root)
    if directory is not None:
        return os.path.join(directory, OUTCOME_NAME)
    if scratch is None:
        raise BadInput(f"{root} 不是 git 工作树的根，结局文件在草稿目录里，要 --scratch 点名\n→ 怎么办：加 --scratch <上一趟换用的草稿目录>")
    return os.path.join(os.path.abspath(scratch), WORK_DIRECTORY_NAME, "outcome.tsv")


def write_outcome_atomically(path, text):
    """同目录临时文件写完、刷盘，再改名换上，再刷目录：读的人看到旧的全文或新的全文，没有半截。"""
    directory = os.path.dirname(path)
    os.makedirs(directory, exist_ok=True)
    temporary = f"{path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write(text)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def outcome_text(rows):
    return "".join("\t".join(str(field) for field in row) + "\n" for row in rows)


def append_outcome(path, rows):
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(outcome_text(rows))
        handle.flush()
        os.fsync(handle.fileno())


def write_outcome_start(path, root, reverse_spec_path, relatives, base_bytes, planned, mode):
    """结局文件整份重写：开始行、说明、每份要换的文件一行（换前与换后的 sha256）。弄坏开关 outcome-complete-first 下紧接着写上完成行。"""
    own = process_state_and_start(os.getpid())
    start = ("开始", root, os.getpid(), int(time.time()), reverse_spec_path, sha256_of(read_bytes(reverse_spec_path)),
             this_machine(), identity_file_value(BOOT_ID_PATH), own[1] if own is not None else UNKNOWN_IDENTITY)
    files = [("文件", relative, sha256_of(base_bytes[relative]), sha256_of(planned[relative])) for relative in relatives]
    if mode == "outcome-complete-first":
        files.append(("完成", len(relatives), int(time.time())))
    write_outcome_atomically(path, outcome_text([start]) + "".join(f"{line}\n" for line in OUTCOME_COMMENTS) + outcome_text(files))


def parse_outcome(path):
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except (OSError, UnicodeError) as error:
        raise OutcomeUnreadable(f"读不了结局文件 {path}（{error}）") from error
    rows = [line.split("\t") for line in text.splitlines() if line.strip() and not line.startswith("#")]
    starts = [row for row in rows if row[0] == "开始"]
    if not starts:
        raise OutcomeUnreadable(f"{path} 不是结局文件：没有「开始」行")
    start = dict(zip(START_ROW_COLUMNS, starts[-1]))
    if len(starts[-1]) != len(START_ROW_COLUMNS) or not start["开始秒数"].isdigit():
        raise OutcomeUnreadable(f"{path} 的开始行有 {len(starts[-1])} 段（这一版写 {len(START_ROW_COLUMNS)} 段：{' '.join(START_ROW_COLUMNS)}，开始秒数是整数），"
                                "不是这一版写的结局文件")
    files = [(row[1], row[2], row[3]) for row in rows if row[0] == "文件" and len(row) >= 4]
    readback = {row[1]: row[2] for row in rows if row[0] == "回读" and len(row) >= 3}
    ending = next((row[0] for row in reversed(rows) if row[0] in ("完成", "退回")), "")
    return Outcome(path, start["仓根"], start["进程号"], start["反向规格路径"], files, readback, ending,
                   int(start["开始秒数"]), start["机器"], start["开机"], start["进程启动刻"])


def current_digest(root, relative):
    try:
        return sha256_of(read_bytes(os.path.join(root, relative)))
    except OSError:
        return "-"


STATE_LANDED, STATE_NOT_LANDED, STATE_TOUCHED = "已换上", "没换上", "被别的改动动过"


def file_states(outcome, root, mode):
    """每份文件现在的状态，按内容判 → [(相对路径, 换前, 换后, 现在, 状态)]。弄坏开关 outcome-reader-trusts-file 下信文件里写的换后，不现算。"""
    rows = []
    for relative, before, after in outcome.files:
        now = after if mode == "outcome-reader-trusts-file" else current_digest(root, relative)
        rows.append((relative, before, after, now, STATE_LANDED if now == after else STATE_NOT_LANDED if now == before else STATE_TOUCHED))
    return rows


def state_lines(states):
    return [f"    {state}：{relative}（换前 {before[:12]}，换后 {after[:12]}，现在 {now[:12]}）" for relative, before, after, now, state in states]


def recovery_lines(outcome, states):
    """没换完时的出路：已换上的几份按反向规格退回，或没换上的几份按规格补完（逐处列出 Edit 的旧串与新串）。"""
    try:
        with open(outcome.reverse_spec, encoding="utf-8") as handle:
            items = [item for item in json.load(handle) if isinstance(item, dict)]
        problem = None
    except (OSError, ValueError) as error:
        items, problem = [], error
    lines = [f"→ 怎么办：二选一，改完按内容判，几份文件全回到换前、或全到换后并 --seal-outcome 补写完成之后就不再拦；反向规格 {outcome.reverse_spec}"]
    if problem is not None:
        lines.append(f"    反向规格读不了（{problem}）：对着 git diff 手工改")
    landed = [state[0] for state in states if state[4] == STATE_LANDED]
    not_landed = [state[0] for state in states if state[4] == STATE_NOT_LANDED]
    if landed:
        lines.append(f"  - 退回（已换上的 {len(landed)} 份，按反向规格里属于它的几处用 Edit 改回，旧串写「现在」，新串写「改回」；钩子坏了时 Bash 会被拒，Edit 不经它）：")
        for relative in landed:
            lines += [f"      {relative} 第 {number} 处 现在 {item['old'][:120]!r} → 改回 {item['new'][:120]!r}（命中 {item.get('count', 1)} 次）"
                      for number, item in enumerate((item for item in items if item.get("file") == relative), 1)]
    if not_landed:
        lines.append(f"  - 补完（没换上的 {len(not_landed)} 份，按规格用 Edit 换上，旧串写「现在」，新串写「换后」；换完先 hook-swap.py --feed-only --hook <钩子> 与 --check-callers 验过，再 hook-swap.py --seal-outcome）：")
        for relative in not_landed:
            lines += [f"      {relative} 第 {number} 处 现在 {item['new'][:120]!r} → 换后 {item['old'][:120]!r}（命中 {item.get('count', 1)} 次）"
                      for number, item in enumerate(reversed([item for item in items if item.get("file") == relative]), 1)]
    if any(state[4] == STATE_TOUCHED for state in states):
        lines.append("  - 被别的改动动过的几份：先看 git diff 是谁改的，别人的改动不盖；问主 agent")
    return lines


def pending_refusal(root, outcome_file, mode):
    """起手先读上一趟的结局文件：没有完成也没有退回、几份文件又不是都在换前的，交 (退出码, 拒绝说明的几行)；别的交 None。
    起那一趟的进程还在这一台上跑（swap_process_state）的退 9、只叫等它；不在的退 7、给退回与补完；在另一台起的退 7、先叫看那一趟还在不在。
    弄坏开关 pending-not-checked 下不读；running-swap-not-checked 下那个进程还在也按不在办。"""
    if mode == "pending-not-checked" or not os.path.exists(outcome_file):
        return None
    try:
        outcome = parse_outcome(outcome_file)
    except OutcomeUnreadable as error:
        return EXIT_PENDING_OUTCOME, [f"✗ 上一趟的结局文件读不了：{error}，不知道几份文件停在哪，拒绝再换，一个字节没写",
                                      "→ 怎么办：看一眼这份文件和 git diff；确认几份钩子与库都是该有的样子之后，删掉这份结局文件再跑"]
    if outcome.ending:
        return None
    states = file_states(outcome, root, mode)
    if all(state[4] == STATE_NOT_LANDED for state in states):
        return None   # 被杀在第一份换上之前（或已手工退回）：一份都没换，没有半截
    process = swap_process_state(outcome, mode)
    if process == SWAP_PROCESS_RUNNING and mode != "running-swap-not-checked":
        elapsed = max(0, int(time.time()) - outcome.started_seconds)
        return EXIT_PREVIOUS_SWAP_RUNNING, [
            f"✗ 上一趟换上还没完（进程 {outcome.process}，起了 {elapsed} 秒），等它写完成再来：结局文件 {outcome_file}，几份文件现在是这样（对着主工作区现算 sha256）：",
            *state_lines(states),
            "→ 怎么办：等那一趟换完、验完（它自己会写完成或退回），再跑这一条", "一个字节没写"]
    if process == SWAP_PROCESS_ELSEWHERE:
        return EXIT_PENDING_OUTCOME, [
            f"✗ 上一趟换上没有写完成（进程 {outcome.process} 不是在这一台上起的，这一台看不到它还在不在）：结局文件 {outcome_file}，几份文件现在是这样（对着主工作区现算 sha256）：",
            *state_lines(states),
            "→ 先看起它的那一趟还在不在跑（task-server/task status 列本会话的任务；别的会话起的问主 agent）：还在跑就等它写完成再来，不在了再照下面二选一",
            *recovery_lines(outcome, states), "一个字节没写"]
    return EXIT_PENDING_OUTCOME, [f"✗ 上一趟换上没有写完成（进程 {outcome.process} 起的，被杀或回滚不成）：结局文件 {outcome_file}，几份文件现在是这样（对着主工作区现算 sha256）：",
                                  *state_lines(states), *recovery_lines(outcome, states), "一个字节没写"]


def read_outcome(path, root, mode):
    """--read-outcome：只读结局文件、对着主工作区现算每份文件的 sha256，判上一趟换没换完 → (退出码, 行)。"""
    try:
        outcome = parse_outcome(path)
    except OutcomeUnreadable as error:
        return 2, [f"✗ {error}", "→ 怎么办：核对路径与 --root / --scratch；没有这份文件说明还没有真换过，格式见 hook-swap.py 文件头「结局文件」"]
    states = file_states(outcome, root, mode)
    if outcome.ending == "完成":
        changed = [state for state in states if state[3] != outcome.readback.get(state[0], state[2])]
        if not changed:
            return 0, [f"✓ 换完了：{path}（进程 {outcome.process}）有完成行，{len(states)} 份文件现在的 sha256 都等于回读"]
        return 4, [f"✗ 完成过，可是这几份现在的 sha256 不等于回读的换后（之后被改了，或被退回了）：{path}",
                   *(f"    {state[0]}（回读 {outcome.readback.get(state[0], state[2])[:12]}，现在 {state[3][:12]}）" for state in changed)]
    if outcome.ending == "退回":
        changed = [state for state in states if state[3] != state[1]]
        if not changed:
            return 1, [f"已退回：{path}（进程 {outcome.process}）有退回行，{len(states)} 份文件现在的 sha256 都等于换前，这一趟一处没留"]
        return 4, [f"✗ 退回过，可是这几份现在的 sha256 不等于换前（之后被改了，或又换上了）：{path}",
                   *(f"    {state[0]}（换前 {state[1][:12]}，现在 {state[3][:12]}）" for state in changed)]
    return 3, [f"✗ 没换完：{path}（进程 {outcome.process}）有开始行，没有完成也没有退回（被杀或回滚不成）；对着主工作区现算 sha256：",
               *state_lines(states), *recovery_lines(outcome, states)]


def seal_outcome(path, root, mode):
    """--seal-outcome：上一趟没写完成，而几份文件现在都等于换后（被杀在回读或换上之后再验之间）：回读一遍，补写回读与完成。→ (退出码, 行)。
    先占按仓的占用记录（文件头「按仓的占用」；占不到退 EXIT_SCRATCH_BUSY），占到之后 seal_holding_repository 再看开始行那一趟还在不在。弄坏开关 seal-outcome-lock-skipped 下不占。"""
    try:
        parse_outcome(path)
    except OutcomeUnreadable as error:
        return EXIT_BAD_INPUT, [f"✗ {error}", "→ 怎么办：核对路径与 --root / --scratch"]
    held = None
    if mode != "seal-outcome-lock-skipped":
        held, busy_lines = hold_repository(path, None, mode)
        if held is None:
            return EXIT_SCRATCH_BUSY, busy_lines
    try:
        return seal_holding_repository(path, root, mode)
    finally:
        if held is not None:
            release_scratch_lock(held)


def seal_holding_repository(path, root, mode):
    """seal_outcome 占到按仓的占用记录之后的整段：读结局文件、开始行那一趟还在就退 EXIT_PREVIOUS_SWAP_RUNNING，几份文件都是换后的才补写。
    弄坏开关 seal-outcome-running-unchecked 下不看开始行那一趟。"""
    try:
        outcome = parse_outcome(path)
    except OutcomeUnreadable as error:
        return EXIT_BAD_INPUT, [f"✗ {error}", "→ 怎么办：核对路径与 --root / --scratch"]
    if outcome.ending:
        return EXIT_BAD_INPUT, [f"✗ 结局文件 {path} 已经有{outcome.ending}行，没有要补的", "→ 怎么办：--read-outcome 看现在的状态"]
    if mode != "seal-outcome-running-unchecked" and swap_process_state(outcome, mode) == SWAP_PROCESS_RUNNING:
        elapsed = max(0, int(time.time()) - outcome.started_seconds)
        return EXIT_PREVIOUS_SWAP_RUNNING, [f"✗ 开始行记的那一趟还在（进程 {outcome.process}，起了 {elapsed} 秒），不补写完成：结局文件 {path}",
                                            "→ 怎么办：等那一趟换完、验完（它自己会写完成或退回）；补写只给被杀、回滚不成的那一趟", "一个字节没写"]
    states = file_states(outcome, root, mode)
    if any(state[4] != STATE_LANDED for state in states):
        return EXIT_PENDING_OUTCOME, ["✗ 还有几份不是换后的内容，不补写完成：", *state_lines(states), *recovery_lines(outcome, states)]
    append_outcome(path, [("回读", state[0], state[3]) for state in states] + [("完成", len(states), int(time.time()), "补记")])
    return EXIT_SWAPPED, [f"✓ 补写了完成行：{path}（{len(states)} 份文件现在都等于换后，回读已记）"]


# 草稿目录的占用（文件头「草稿目录的占用」）：同一个 --scratch 同一时刻只许一趟在跑
SCRATCH_LOCK_NAME = "swap.lock"
SCRATCH_LOCK_HEARTBEAT_SECONDS = 15
SCRATCH_LOCK_STALE_SECONDS = 300   # 另一台起的持有者，记录这么久没刷新就当它被杀了；心跳间隔的 20 倍，容得下共享盘属性缓存与两台时钟的差
SCRATCH_LOCK_ATTEMPTS = 3   # 摘旧记录、放自己的记录时被别的趟抢先，重判几次
SCRATCH_LOCK_COLUMNS = ("机器", "开机", "进程号", "进程启动刻", "开始秒数", "随机串", "主机名")
# 按仓的占用（文件头「按仓的占用」）：放在结局文件同一个目录，记录比草稿目录那一份末尾多一段这一趟用的草稿目录
REPOSITORY_LOCK_NAME = "singlefs-hook-swap.lock"
REPOSITORY_LOCK_COLUMNS = SCRATCH_LOCK_COLUMNS + ("草稿目录",)
REMOVAL_SUFFIX = ".taking"   # 摘旧记录时先排他放一份这一趟自己的身份的文件，名字是记录路径加这个后缀
REAP_INFIX = ".reap-"   # 摘已死的 .taking（或已死的摘取记录）时排他放的摘取记录：<被摘的文件><这一段><读到的内容的 sha256 前 16 位>
RECORD_REMOVED, RECORD_CHANGED, RECORD_BEING_REMOVED, RECORD_TAKEN_OVER = "摘掉了", "记录变了", "别的趟正在摘", "被接手了"
REMOVAL_TAKEN_OVER = "被接手"   # occupying_holder 交的第三段：这一趟摘的当中 .taking 被别的趟当成已死摘掉、接手了（别的取值是正在摘的那一趟放的文件的路径）
HANDLED_SIGNALS = (signal.SIGTERM, signal.SIGHUP)   # 默认处理会让进程直接死、跑不到 finally 的信号
HELD_SCRATCH_LOCKS = []   # 这个进程此刻占着的记录（草稿目录的与按仓的）；信号处理按它放锁
HELD_REMOVAL_CLAIMS = []   # 这个进程此刻放着的摘旧记录的占用（REMOVAL_SUFFIX 那个文件或摘取记录的路径, 内容）；信号处理按它放掉
REMOVAL_CLAIM_HEARTBEATS = {}   # 放着的 REMOVAL_SUFFIX 那个文件 (路径, 内容) → 停它心跳的信号；放掉时停


class ScratchHolder(NamedTuple):
    """占用记录写的那一趟。机器、开机、进程号、启动刻四段的名字与 Outcome 的同名字段一致，swap_process_state 两边通用。directory 只有按仓的记录有（这一趟用的草稿目录）。"""
    machine: str
    boot: str
    process: str
    start_ticks: str
    started_seconds: int
    token: str
    host: str
    directory: str = ""


class ScratchLock:
    """这一趟占着的一份记录（草稿目录的或按仓的）：占用记录的路径与内容、心跳停止信号、装上的信号处理（装之前的处理，放锁时还原）。"""

    def __init__(self, path, content, mode):
        self.path = path
        self.content = content
        self.mode = mode
        self.stop_heartbeat = threading.Event()
        self.previous_handlers = {}


def scratch_lock_path(scratch, mode):
    """占用记录的位置：<草稿目录>/hook-swap/swap.lock。弄坏开关 scratch-lock-keyed-by-parent 下放在草稿目录的上一层（同一层下的几个草稿目录互相挡）。"""
    directory = os.path.dirname(scratch) if mode == "scratch-lock-keyed-by-parent" else os.path.join(scratch, WORK_DIRECTORY_NAME)
    return os.path.join(directory, SCRATCH_LOCK_NAME)


def scratch_lock_content(directory=None):
    """这一趟的占用记录：一行，段名取 SCRATCH_LOCK_COLUMNS；directory 给了（按仓的记录）末尾加一段草稿目录，段名取 REPOSITORY_LOCK_COLUMNS。"""
    own = process_state_and_start(os.getpid())
    fields = (this_machine(), identity_file_value(BOOT_ID_PATH), os.getpid(), own[1] if own is not None else UNKNOWN_IDENTITY,
              int(time.time()), secrets.token_hex(8), socket.gethostname() or UNKNOWN_IDENTITY)
    if directory is not None:
        fields += (directory,)
    return "\t".join(str(field) for field in fields) + "\n"


def parse_scratch_holder(text, columns=SCRATCH_LOCK_COLUMNS):
    """占用记录的文本 → ScratchHolder；段数不等于 columns 的、进程号或开始秒数不是整数的交 None（读不全，按另一台起的办）。"""
    row = dict(zip(columns, text.rstrip("\n").split("\t")))
    if len(row) != len(columns) or text.count("\t") != len(columns) - 1 or not row["进程号"].isdigit() or not row["开始秒数"].isdigit():
        return None
    return ScratchHolder(row["机器"], row["开机"], row["进程号"], row["进程启动刻"], int(row["开始秒数"]), row["随机串"], row["主机名"], row.get("草稿目录", ""))


def scratch_holder_occupies(holder, age_seconds, mode):
    """写占用记录的那一趟此刻还占着草稿目录吗。这一台起的看进程（swap_process_state，传空开关：那里的弄坏开关不波及这一处）：进程在算占着，
    没了算放了，不管记录多新；另一台起的、记录读不全的，这一台看不到进程，看记录多少秒前刷新过（持有期间有心跳），SCRATCH_LOCK_STALE_SECONDS 内的算占着。
    弄坏开关 dead-holder-blocks 下这一台起的、进程已经没了的也算占着；elsewhere-holder-ignored 下另一台起的一律算放了；elsewhere-holder-never-expires 下另一台起的一律算占着。"""
    state = swap_process_state(holder, "") if holder is not None else SWAP_PROCESS_ELSEWHERE
    if state == SWAP_PROCESS_RUNNING:
        return True
    if state == SWAP_PROCESS_GONE:
        return mode == "dead-holder-blocks"
    if mode == "elsewhere-holder-ignored":
        return False
    if mode == "elsewhere-holder-never-expires":
        return True
    return age_seconds < SCRATCH_LOCK_STALE_SECONDS


def scratch_busy_lines(scratch, path, holder, age_seconds, removing=""):
    """拒绝说明：哪个草稿目录被哪一趟占着（读得到记录就写进程号、主机名、这一台还是另一台、开跑时刻与已跑多少秒），出路。holder 是 None 的是记录读不全或连判几次它都在变；
    removing 给了路径的，holder 是正在摘 path 上旧记录的那一趟（写那个文件的：REMOVAL_SUFFIX 那一个或摘取记录）；是 REMOVAL_TAKEN_OVER 的，这一趟摘的当中被接手了（holder_description）。"""
    who, on_this_machine = holder_description(path, holder, age_seconds, removing)
    lines = [f"✗ 草稿目录 {scratch} 正被另一趟 hook-swap.py 占着（{who}）：草稿树 {os.path.join(scratch, WORK_DIRECTORY_NAME, 'tree')} 只有一棵，"
             "并行起会互相删掉、盖掉；这一趟没建树、没写文件",
             "→ 怎么办：等那一趟结束再交，或换一个 --scratch（--dry-run 要并行的几条各给一个草稿目录；真换在同一个仓里一次只许一趟，另一趟真换在跑时换草稿目录也会被拒）"]
    if not on_this_machine:
        lines.append(stale_holder_hint(path, removing))
    return lines


def stale_holder_hint(path, removing):
    """另一台起的（或读不全的）持有者可能已被杀：过了期就不再挡，确认它死了就删它放的那个文件（removing 给了的是正在摘的那一趟放的那个文件，否则是占用记录）。"""
    if removing:
        return (f"→ 摘的那一趟若已被杀：{removing} {SCRATCH_LOCK_STALE_SECONDS} 秒没刷新就不再挡；"
                f"确认它死了也可以直接删 {removing} 再来")
    return f"→ 那一趟若已被杀：记录 {SCRATCH_LOCK_STALE_SECONDS} 秒没刷新就不再挡；确认它死了也可以直接删 {path} 再来"


def holder_description(path, holder, age_seconds, removing=""):
    """拒绝说明里写占着的那一趟 → (一句话, 是不是这一台起的)。读得到记录就写进程号、这一台还是另一台、主机、按仓的记录再写用的草稿目录、开跑时刻与已跑多少秒；
    holder 是 None 的是记录读不全或连判几次它都在变。removing 给了路径的，写的是正在摘 path 上旧记录的那一趟：holder 来自它放的那个文件
    （REMOVAL_SUFFIX 那一个或摘取记录，就是 removing；None 是它读不全），age_seconds 是它多久前刷新过。
    removing 是 REMOVAL_TAKEN_OVER 的，写这一趟自己摘的当中被别的趟接手了（holder 不用），交「是这一台」（不加另一台过期的出路）。"""
    if removing == REMOVAL_TAKEN_OVER:
        return (f"这一趟摘占用记录 {path} 的旧记录时停住了，放在 {path}{REMOVAL_SUFFIX} 的身份被别的趟当成已死摘掉、接手了摘旧记录；"
                "这一趟恢复后删掉的可能是接手的那一趟刚放的记录，这一趟不占"), True
    here = this_machine()
    on_this_machine = holder is not None and holder.machine == here and here != UNKNOWN_IDENTITY
    if holder is None:
        if removing:
            return f"别的趟正在摘占用记录 {path} 的旧记录，它放在 {removing} 的身份读不全（{int(age_seconds)} 秒前刷新过）", on_this_machine
        return f"占用记录 {path} 读不全，或连判几次它都在变", on_this_machine
    started = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(holder.started_seconds))
    elapsed = max(0, int(time.time()) - holder.started_seconds)
    place = "这一台" if on_this_machine else "另一台，这一台看不到那个进程，靠记录的刷新时间判"
    directory = ""
    if holder.directory:
        directory = "，--seal-outcome 补写完成，没有草稿目录" if holder.directory == UNKNOWN_IDENTITY else f"，用的草稿目录 {holder.directory}"
    detail = f"进程 {holder.process}，{place}，主机 {holder.host}{directory}，开跑于 {started}（已 {elapsed} 秒）"
    if removing:
        return f"别的趟正在摘占用记录 {path} 的旧记录，摘的那一趟：{detail}，它放的 {removing} {int(age_seconds)} 秒前刷新过", on_this_machine
    return f"{detail}，占用记录 {int(age_seconds)} 秒前刷新过", on_this_machine


def repository_busy_lines(outcome_file, path, holder, age_seconds, removing=""):
    """拒绝说明：这个仓的真换（或 --seal-outcome）正被哪一趟占着、出路；removing 的含义同 scratch_busy_lines。"""
    who, on_this_machine = holder_description(path, holder, age_seconds, removing)
    lines = [f"✗ 这个仓的换上正被另一趟 hook-swap.py 占着（{who}）：结局文件 {outcome_file} 按仓只有一份，真换与 --seal-outcome 一次只许一趟；这一趟没读结局文件、没建树、没写文件",
             "→ 怎么办：等那一趟结束再交（换草稿目录没有用；--dry-run 不占这一道，各给一个 --scratch 可以并行）"]
    if not on_this_machine:
        lines.append(stale_holder_hint(path, removing))
    return lines


def read_scratch_record(path):
    return read_bytes(path).decode("utf-8", errors="replace")


def occupying_holder(path, mode, columns=SCRATCH_LOCK_COLUMNS, after_judged=None, after_taken=None, heartbeat_seconds=SCRATCH_LOCK_HEARTBEAT_SECONDS):
    """占用记录 path 此刻还挡着吗 → (写记录的那一趟 ScratchHolder，读不全的是 None；记录多少秒前刷新过)；
    没有记录、或写它的那一趟已经放了（旧记录已摘掉）→ None；连摘几次记录都变了 → (None, 0.0)，当作占着。
    别的趟正在摘这份旧记录、它放的文件（REMOVAL_SUFFIX 那一个或摘取记录）还占着 → (写那个文件的那一趟 ScratchHolder 或读不全的 None, 它多久前刷新过, 那个文件的路径)，当作占着。
    这一趟摘的当中被别的趟接手了 → (None, 0.0, REMOVAL_TAKEN_OVER)，不占。摘旧记录见 remove_stale_record，heartbeat_seconds 是摘的期间刷新 REMOVAL_SUFFIX 那个文件的间隔。
    after_judged（判过不挡、动手摘之前）与 after_taken（摘的当中）只给自证插别的趟用，平常是 None。弄坏开关 taking-holder-unnamed 下别的趟正在摘时不交摘的那一趟，交 (None, 0.0)。"""
    for _attempt in range(SCRATCH_LOCK_ATTEMPTS):
        try:
            text = read_scratch_record(path)
            age_seconds = max(0.0, time.time() - os.stat(path).st_mtime)
        except FileNotFoundError:
            return None
        holder = parse_scratch_holder(text, columns)
        if scratch_holder_occupies(holder, age_seconds, mode):
            return holder, age_seconds
        if after_judged is not None:
            after_judged()
        removal, remover = remove_stale_record(path, text, mode, after_taken, heartbeat_seconds)
        if removal == RECORD_REMOVED:
            return None
        if removal == RECORD_TAKEN_OVER:
            return None, 0.0, REMOVAL_TAKEN_OVER
        if removal == RECORD_BEING_REMOVED:
            return (None, 0.0) if mode == "taking-holder-unnamed" else remover
    return None, 0.0


def remove_stale_record(path, text, mode, after_taken=None, heartbeat_seconds=SCRATCH_LOCK_HEARTBEAT_SECONDS):
    """path 上读到的 text 已判成不再挡：只在 path 上此刻还是这一份时删掉它 → (状态, 正在摘的那一趟)。状态：RECORD_REMOVED、RECORD_CHANGED（path 上没有记录了，或是别的趟刚放的新记录，没动它，重判）、
    RECORD_BEING_REMOVED（别的趟正在摘，第二项是 (写它放的那个文件的那一趟 ScratchHolder 或读不全的 None, 它多久前刷新过, 那个文件的路径)）、
    RECORD_TAKEN_OVER（这一趟摘的当中 REMOVAL_SUFFIX 那个文件被别的趟当成已死摘掉、接手了：这一趟不占）；别的状态第二项是 None。
    先在 path + REMOVAL_SUFFIX 上排他放这一趟自己的身份（claim_record_removal，放着的期间每 heartbeat_seconds 秒刷新），放成了再读 path：不是 text（没了，或别的趟刚放了新的）不动它，
    是才删；删之前与之后各核一次 REMOVAL_SUFFIX 那个文件还是这一趟放的（removal_claim_lost），不是了交 RECORD_TAKEN_OVER；最后放掉 REMOVAL_SUFFIX 那个文件。
    path 上的记录从头到尾不挪开，别的趟不会在中间看到空位放进去；摘的当中别的趟要摘同一份会撞上 REMOVAL_SUFFIX 那个文件、当占着。
    摘的这一段里 TERM 与 HUP 由 release_then_die 处理，放掉 REMOVAL_SUFFIX 那个文件（已占着记录的一趟，处理早就装上了）。
    弄坏开关 stale-record-renamed-away 下改名挪开、核内容，不是 text 就放回去，放不回去（别的趟已放了新的）就删掉摘到的那一份。"""
    if mode == "stale-record-renamed-away":
        removed = f"{path}.removed-{secrets.token_hex(4)}"
        try:
            os.rename(path, removed)
        except FileNotFoundError:
            return RECORD_CHANGED, None
        if after_taken is not None:
            after_taken()
        taken = read_scratch_record(removed)
        if taken == text:
            os.unlink(removed)
            return RECORD_REMOVED, None
        try:
            os.link(removed, path)
        except FileExistsError:
            pass
        os.unlink(removed)
        return RECORD_CHANGED, None
    taking = path + REMOVAL_SUFFIX
    previous_handlers = {}
    install_release_handlers(previous_handlers, mode)
    try:
        claim, remover = claim_record_removal(taking, mode, heartbeat_seconds)
        if claim is None:
            return RECORD_BEING_REMOVED, remover
        try:
            if after_taken is not None:
                after_taken()
            try:
                current = read_scratch_record(path)
            except FileNotFoundError:
                return RECORD_CHANGED, None
            if current != text:
                return RECORD_CHANGED, None
            if removal_claim_lost(taking, claim, mode):
                return RECORD_TAKEN_OVER, None
            try:
                os.unlink(path)
            except FileNotFoundError:
                pass
            if removal_claim_lost(taking, claim, mode):
                return RECORD_TAKEN_OVER, None
            return RECORD_REMOVED, None
        finally:
            release_removal_claim(taking, claim)
    finally:
        restore_signal_handlers(previous_handlers)


def removal_claim_lost(taking, claim, mode):
    """摘的当中 taking 上已不是这一趟放的 claim 了吗（没了，或上面已是别的趟放的身份）：是就是这一趟停住时被当成已死、别的趟摘掉 taking 接手了。
    弄坏开关 taken-over-unchecked 下一概当还是自己的。"""
    if mode == "taken-over-unchecked":
        return False
    try:
        return read_scratch_record(taking) != claim
    except FileNotFoundError:
        return True


def claim_record_removal(taking, mode, heartbeat_seconds=SCRATCH_LOCK_HEARTBEAT_SECONDS):
    """摘旧记录要先占住 taking（记录路径加 REMOVAL_SUFFIX）：把这一趟自己的身份（scratch_lock_content，写同目录临时文件再 os.link）排他放在 taking 上 →
    (放成的内容, None)；别的趟还占着 → (None, (写它放的那个文件的那一趟 ScratchHolder 或读不全的 None, 它多久前刷新过, 那个文件的路径))，
    那个文件是 taking，或别的趟正在摘已死的 taking 时放的摘取记录；连判几次都被别的趟抢先 → (None, (None, 0.0, taking))。
    taking 在而写它的那一趟不再占着（被 KILL、被 TERM 来不及放、进程没了，另一台起的过了期；判法同占用记录，scratch_holder_occupies）的经 reap_dead_file 摘掉（只许一趟动手）、重来；
    link 报已有而读回来是自己写的（共享盘上 link 在服务端做成、回复丢了）算放成。放之前先登记到 HELD_REMOVAL_CLAIMS，被 TERM 或 HUP 杀掉时由 release_then_die 放掉。
    放成之后起一条心跳线程，每 heartbeat_seconds 秒刷新 taking 的修改时间（另一台上看它的靠这个），release_removal_claim 时停。
    弄坏开关 taking-claim-unjudged 下 taking 在就一概当别的趟还占着；taking-reply-lost-refused 下 link 报已有一概当别的趟占着；taking-claim-kept-on-term 下不登记；
    taking-no-heartbeat 下不起心跳。"""
    content = scratch_lock_content()
    entry = (taking, content)
    if mode != "taking-claim-kept-on-term":
        HELD_REMOVAL_CLAIMS.append(entry)
    claimed = False
    try:
        for _attempt in range(SCRATCH_LOCK_ATTEMPTS):
            if link_scratch_record(taking, content, "link-reply-lost-refused" if mode == "taking-reply-lost-refused" else ""):
                claimed = True
                if mode != "taking-no-heartbeat":
                    stop = threading.Event()
                    REMOVAL_CLAIM_HEARTBEATS[entry] = stop
                    threading.Thread(target=keep_file_fresh, args=(taking, content, stop, heartbeat_seconds), daemon=True).start()
                return content, None
            try:
                text = read_scratch_record(taking)
                age_seconds = max(0.0, time.time() - os.stat(taking).st_mtime)
            except FileNotFoundError:
                continue
            holder = parse_scratch_holder(text)
            if mode == "taking-claim-unjudged" or scratch_holder_occupies(holder, age_seconds, mode):
                return None, (holder, age_seconds, taking)
            reaper = reap_dead_file(taking, text, mode)
            if reaper is not None:
                return None, reaper
        return None, (None, 0.0, taking)
    finally:
        if not claimed:
            forget_removal_claim(entry)


def reap_dead_file(path, text, mode, depth=0):
    """path 上读到的 text 已判成写它的那一趟不再占着：只许一趟删它 → None（删掉了，或 path 上已不是这一份：调用方重判）；
    别的趟正在删它 → (写摘取记录的那一趟 ScratchHolder 或读不全的 None, 摘取记录多久前放的, 摘取记录的路径)，当作占着。
    摘取记录是 path + REAP_INFIX + text 的 sha256 前 16 位：名字跟着这一份已死的内容走，排他放这一趟自己的身份（link_scratch_record），同一份只有一趟放得成；
    放成的那一趟再读 path，还是 text 才删，再放掉摘取记录。放不成的判写摘取记录的那一趟（scratch_holder_occupies）：还占着就交它；
    不再占着（放成之后、删之前被 KILL）的照同一个办法摘掉那份摘取记录再重来，往下最多 SCRATCH_LOCK_ATTEMPTS 层，再深交 (None, 0.0, 摘取记录的路径)。
    摘取记录放之前就登记到 HELD_REMOVAL_CLAIMS（被 TERM 或 HUP 杀掉时 release_then_die 放掉）；它只占几次系统调用，没有心跳。
    弄坏开关 taking-reap-unclaimed 下不放摘取记录：读到还是 text 就删（几趟同时摘都删得下手，后删的删掉先删那一趟刚放的）。"""
    if mode == "taking-reap-unclaimed":
        try:
            if read_scratch_record(path) == text:
                os.unlink(path)
        except FileNotFoundError:
            pass
        return None
    claim_path = f"{path}{REAP_INFIX}{sha256_of(text)[:16]}"
    for _attempt in range(SCRATCH_LOCK_ATTEMPTS):
        content = scratch_lock_content()
        HELD_REMOVAL_CLAIMS.append((claim_path, content))
        try:
            if link_scratch_record(claim_path, content):
                try:
                    if read_scratch_record(path) == text:
                        os.unlink(path)
                except FileNotFoundError:
                    pass
                return None
        finally:
            release_removal_claim(claim_path, content)
        try:
            claim_text = read_scratch_record(claim_path)
            age_seconds = max(0.0, time.time() - os.stat(claim_path).st_mtime)
        except FileNotFoundError:
            continue
        holder = parse_scratch_holder(claim_text)
        if scratch_holder_occupies(holder, age_seconds, mode):
            return holder, age_seconds, claim_path
        if depth + 1 >= SCRATCH_LOCK_ATTEMPTS:
            return None, 0.0, claim_path
        reaper = reap_dead_file(claim_path, claim_text, mode, depth + 1)
        if reaper is not None:
            return reaper
    return None, 0.0, claim_path


def forget_removal_claim(entry):
    """entry 从 HELD_REMOVAL_CLAIMS 摘掉（不在就算了：信号处理可能已经摘过）。"""
    try:
        HELD_REMOVAL_CLAIMS.remove(entry)
    except ValueError:
        pass


def release_removal_claim(taking, content):
    """放掉 claim_record_removal 放成的 taking（或 reap_dead_file 放的摘取记录）：停它的心跳，taking 上还是自己写的内容才删（已被别人换成别的内容的不删）；可重复调。"""
    forget_removal_claim((taking, content))
    stop = REMOVAL_CLAIM_HEARTBEATS.pop((taking, content), None)
    if stop is not None:
        stop.set()
    try:
        if read_scratch_record(taking) == content:
            os.unlink(taking)
    except FileNotFoundError:
        pass
    except OSError as error:
        print(f"  ! 放不掉摘旧记录的占用 {taking}（{error}）：别的趟会照「进程没了就不挡」让路，确认没有在跑的 hook-swap.py 之后可以手工删", file=sys.stderr)


def link_scratch_record(path, content, mode="", link=None):
    """content 写进同目录的临时文件（刷盘），再 os.link 成 path：成了交 True，path 已有（别的趟刚放成）交 False；临时文件无论如何都删。
    link 报已有而 path 上读回来的就是 content（共享盘上 link 在服务端做成、回复丢了、重发报已有）也交 True；link 给了就用它代替 os.link（只给自证造这种回复用）。
    弄坏开关 scratch-record-overwrites 下改名盖上去（后放的盖掉先放的，都交 True）；link-reply-lost-refused 下报已有一概交 False。"""
    temporary = f"{path}.{os.getpid()}.{secrets.token_hex(4)}.tmp"
    try:
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write(content)
            handle.flush()
            os.fsync(handle.fileno())
        if mode == "scratch-record-overwrites":
            os.replace(temporary, path)
        else:
            (link or os.link)(temporary, path)
        return True
    except FileExistsError:
        if mode == "link-reply-lost-refused":
            return False
        try:
            return read_scratch_record(path) == content
        except OSError:
            return False
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def keep_scratch_lock_fresh(held, interval_seconds):
    """持有期间每 interval_seconds 秒刷新一次占用记录的修改时间（另一台靠它看出持有者还在）；记录已被换成别人的就不刷。"""
    keep_file_fresh(held.path, held.content, held.stop_heartbeat, interval_seconds)


def keep_file_fresh(path, content, stop, interval_seconds):
    """stop 设上之前每 interval_seconds 秒刷新一次 path 的修改时间；path 上已不是 content（被别人换了、没了）就不刷。占用记录与 REMOVAL_SUFFIX 那个文件的心跳都走这一份。"""
    while not stop.wait(interval_seconds):
        try:
            if read_scratch_record(path) == content:
                os.utime(path)
        except OSError:
            continue


def release_scratch_lock(held):
    """放锁，可重复调：停心跳、还原装上的信号处理、记录还是自己写的就删掉（已被别人换成别的内容的不删）。弄坏开关 scratch-lock-kept-on-exit 下记录不删。"""
    held.stop_heartbeat.set()
    if held in HELD_SCRATCH_LOCKS:
        HELD_SCRATCH_LOCKS.remove(held)
    restore_signal_handlers(held.previous_handlers)
    if held.mode == "scratch-lock-kept-on-exit":
        return
    try:
        if read_scratch_record(held.path) == held.content:
            os.unlink(held.path)
    except FileNotFoundError:
        pass
    except OSError as error:
        print(f"  ! 放不掉草稿目录的占用记录 {held.path}（{error}）：别的趟会照「进程没了就不挡」与心跳过期让路，确认没有在跑的 hook-swap.py 之后可以手工删", file=sys.stderr)


def release_then_die(signal_number, _frame):
    """被 TERM 或 HUP：先放掉这个进程放着的摘旧记录的占用（HELD_REMOVAL_CLAIMS）与占着的草稿目录和仓，再还原默认处理、重发同一个信号，退出状态仍是被这个信号杀的。"""
    for taking, content in list(HELD_REMOVAL_CLAIMS):
        release_removal_claim(taking, content)
    for held in list(HELD_SCRATCH_LOCKS):
        release_scratch_lock(held)
    signal.signal(signal_number, signal.SIG_DFL)
    signal.raise_signal(signal_number)


def install_release_handlers(previous_handlers, mode):
    """TERM 与 HUP 的处理装成 release_then_die，装之前的处理记进 previous_handlers（restore_signal_handlers 还原）。只在主线程里装；本来就忽略的信号、
    已经是 release_then_die 的（同一个进程已占着一份记录或已在摘旧记录，处理按 HELD_SCRATCH_LOCKS 与 HELD_REMOVAL_CLAIMS 放全部）不碰。弄坏开关 scratch-lock-term-unhandled 下什么都不装。"""
    if mode == "scratch-lock-term-unhandled" or threading.current_thread() is not threading.main_thread():
        return
    for signal_number in HANDLED_SIGNALS:
        previous = signal.getsignal(signal_number)
        if previous != signal.SIG_IGN and previous is not release_then_die:
            previous_handlers[signal_number] = previous if previous is not None else signal.SIG_DFL
            signal.signal(signal_number, release_then_die)


def restore_signal_handlers(previous_handlers):
    """install_release_handlers 装之前的处理还原回去，previous_handlers 清空；可重复调。"""
    for signal_number, previous in previous_handlers.items():
        signal.signal(signal_number, previous)
    previous_handlers.clear()


def start_holding(held, heartbeat_seconds):
    """记录放成之后：登记到 HELD_SCRATCH_LOCKS、起心跳线程、装 TERM 与 HUP 的处理（install_release_handlers）。
    弄坏开关 scratch-lock-no-heartbeat 下不起心跳；scratch-lock-term-unhandled 下不装信号处理。"""
    HELD_SCRATCH_LOCKS.append(held)
    if held.mode != "scratch-lock-no-heartbeat":
        threading.Thread(target=keep_scratch_lock_fresh, args=(held, heartbeat_seconds), daemon=True).start()
    install_release_handlers(held.previous_handlers, held.mode)


def hold_scratch(scratch, mode, heartbeat_seconds=SCRATCH_LOCK_HEARTBEAT_SECONDS):
    """一趟开跑就占住草稿目录 scratch → (ScratchLock, None)；被另一趟占着 → (None, 拒绝说明的几行)，不建树、不写文件。放记录建不成（权限、空间、不能建硬链接）抛 BadInput。
    弄坏开关 scratch-lock-skipped 下不放记录，交一份没有记录的 ScratchLock。"""
    path = scratch_lock_path(scratch, mode)
    held = ScratchLock(path, scratch_lock_content(), mode)
    if mode == "scratch-lock-skipped":
        return held, None
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        busy = acquire_record(held, SCRATCH_LOCK_COLUMNS, heartbeat_seconds)
    except OSError as error:
        raise BadInput(f"放不了草稿目录的占用记录 {path}（{error}）\n→ 怎么办：核对草稿目录的写权限、剩余空间；占用记录靠硬链接放，共享根要能建硬链接") from error
    return (held, None) if busy is None else (None, scratch_busy_lines(scratch, path, *busy))


def acquire_record(held, columns, heartbeat_seconds, after_judged=None, after_taken=None):
    """在 held.path 上放 held.content：放成了（已 start_holding）交 None；被占着交 occupying_holder 交的那一组（写记录的那一趟 ScratchHolder 或 None，记录多少秒前刷新过，
    别的趟正在摘或这一趟被接手时多一段）。OSError 往上抛。columns 是这份记录的段名；heartbeat_seconds 是占着记录与摘旧记录时放着 .taking 的心跳间隔；
    after_judged、after_taken 交给 occupying_holder（只给自证）。"""
    for _attempt in range(SCRATCH_LOCK_ATTEMPTS):
        occupied = occupying_holder(held.path, held.mode, columns, after_judged, after_taken, heartbeat_seconds)
        if occupied is not None:
            return occupied
        if link_scratch_record(held.path, held.content, held.mode):
            start_holding(held, heartbeat_seconds)
            return None
    return None, 0.0


def repository_lock_path(outcome_file, scratch, mode):
    """按仓的占用记录的位置：结局文件所在的目录下 REPOSITORY_LOCK_NAME。弄坏开关 repository-lock-keyed-by-scratch 下（scratch 给了的）放在这一趟草稿目录的 hook-swap/ 里。"""
    if mode == "repository-lock-keyed-by-scratch" and scratch is not None:
        return os.path.join(scratch, WORK_DIRECTORY_NAME, REPOSITORY_LOCK_NAME)
    return os.path.join(os.path.dirname(outcome_file), REPOSITORY_LOCK_NAME)


def hold_repository(outcome_file, scratch, mode, heartbeat_seconds=SCRATCH_LOCK_HEARTBEAT_SECONDS):
    """真换（与 --seal-outcome）占住这个仓 → (ScratchLock, None)；被另一趟占着 → (None, 拒绝说明的几行)，没读结局文件、没写文件。放记录建不成抛 BadInput。
    scratch 是这一趟用的草稿目录，--seal-outcome 没有，给 None（记录里写 UNKNOWN_IDENTITY）。弄坏开关里只有 REPOSITORY_LOCK_MODES 的波及这一份。"""
    path = repository_lock_path(outcome_file, scratch, mode)
    held = ScratchLock(path, scratch_lock_content(scratch if scratch is not None else UNKNOWN_IDENTITY), mode if mode in REPOSITORY_LOCK_MODES else "")
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        busy = acquire_record(held, REPOSITORY_LOCK_COLUMNS, heartbeat_seconds)
    except OSError as error:
        raise BadInput(f"放不了这个仓的换上占用记录 {path}（{error}）\n→ 怎么办：核对 {os.path.dirname(path)} 的写权限、剩余空间；占用记录靠硬链接放，要能建硬链接") from error
    return (held, None) if busy is None else (None, repository_busy_lines(outcome_file, path, *busy))


def swap(spec, hook, scratch, root, feeds=None, dry_run=False, after_draft=None, after_swap=None, verify=None, after_file_swapped=None):
    """交回 (退出码, 提示行)。after_draft、after_swap、after_file_swapped 只给自证用：草稿验完之后、全部换上之后各调一次，每换上一份调一次（模拟别的会话在这段时间里改文件、或被杀在两份之间）。
    脚本规格（文件都在脚本规格目录下）：hook 与 feeds 给 None，verify 是换上之后跑的命令（可省）；钩子规格：hook 必给，verify 给 None。"""
    mode = break_mode()
    root = os.path.realpath(root)
    scratch = os.path.abspath(scratch)
    if os.path.realpath(scratch) == root or is_inside(scratch, root):
        raise BadInput(f"草稿目录 {scratch} 在主工作区里\n→ 怎么办：放进派发提示给的草稿目录（工作根 ~/.local/share/singlefs/work/<轮>/ 这类仓外位置）")
    held, busy_lines = hold_scratch(scratch, mode)   # 先占草稿目录：被占着就在建树、写任何文件之前拒绝（文件头「草稿目录的占用」）
    if held is None:
        return EXIT_SCRATCH_BUSY, busy_lines
    try:
        return swap_holding_scratch(spec, hook, scratch, root, mode, feeds, dry_run, after_draft, after_swap, verify, after_file_swapped)
    finally:
        release_scratch_lock(held)


def swap_holding_scratch(spec, hook, scratch, root, mode, feeds, dry_run, after_draft, after_swap, verify, after_file_swapped):
    """swap 占住草稿目录之后的整段：真换先占这个仓（文件头「按仓的占用」，占不到在读结局文件之前拒），到写完成或退回、或出错才放；--dry-run 不占。
    参数的意思同 swap，root 已是真路径、scratch 已是绝对路径。"""
    outcome_file = outcome_path(root, scratch)
    if dry_run:
        return swap_holding_repository(spec, hook, scratch, root, mode, feeds, dry_run, after_draft, after_swap, verify, after_file_swapped, outcome_file)
    held, busy_lines = hold_repository(outcome_file, scratch, mode)
    if held is None:
        return EXIT_SCRATCH_BUSY, busy_lines
    try:
        return swap_holding_repository(spec, hook, scratch, root, mode, feeds, dry_run, after_draft, after_swap, verify, after_file_swapped, outcome_file)
    finally:
        release_scratch_lock(held)


def swap_holding_repository(spec, hook, scratch, root, mode, feeds, dry_run, after_draft, after_swap, verify, after_file_swapped, outcome_file):
    """swap_holding_scratch 占好之后（--dry-run 不占）的整段：先读结局文件 outcome_file，再验、换、再验。"""
    unfinished = pending_refusal(root, outcome_file, mode)   # 上一趟还在换、还在验，或换到一半被杀：几份文件停在半截，不在它上面再换
    if unfinished is not None:
        return unfinished
    edits = resolve_edits(spec, root, mode)
    script_spec = is_script_edit(edits[0], root)
    if script_spec:
        if hook is not None or feeds is not None:
            raise BadInput("规格里的文件都在脚本规格目录下，不带 --hook 与 --feeds：脚本没有钩子要喂\n"
                           "→ 怎么办：去掉 --hook / --feeds；换上之后要跑的检查（例：被换脚本的 --selftest）写成 --verify '<命令>'")
        hook_relative = None
    else:
        if verify is not None:
            raise BadInput("--verify 只给 scripts/ 下的脚本规格；钩子规格换上之后拿 JSON 喂钩子验\n→ 怎么办：去掉 --verify，要喂的补进 --feeds")
        if hook is None:
            raise BadInput("规格里是 .claude/hooks/ 下的文件，要 --hook 点名拿哪一份钩子喂\n→ 怎么办：加 --hook <钩子>（例如 .claude/hooks/project-code-guard.sh）")
        hook_path = os.path.realpath(hook if os.path.isabs(hook) else os.path.join(root, hook))
        if os.path.dirname(hook_path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY)) or not hook_path.endswith(".sh") or not os.path.isfile(hook_path):
            raise BadInput(f"--hook {hook} 不是 {os.path.join(root, HOOKS_DIRECTORY)} 这一层里已有的 .sh\n→ 怎么办：--hook 写要喂 JSON 的那一份钩子（例如 .claude/hooks/project-code-guard.sh）")
        hook_relative = os.path.relpath(hook_path, root)
        if feeds is None:
            feeds = builtin_feeds_for(hook_relative, mode)
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
    verify_log_path = os.path.join(work, f"verify-{stamp}.log")
    with open(reverse_spec_path, "w", encoding="utf-8") as handle:
        json.dump([{"file": edit.relative, "old": edit.new, "new": edit.old, "count": edit.count} for edit in reversed(edits)], handle, ensure_ascii=False, indent=1)
    if script_spec:
        head = (f"规格 {len(edits)} 处、{len(relatives)} 份文件（{'、'.join(relatives)}）；脚本规格，不喂钩子；"
                f"换上之后的验证命令 {verify if verify else '无（只做语法检查与回读）'}")
        tail = f"验证输出 {verify_log_path}；反向规格 {reverse_spec_path}；结局文件 {outcome_file}"
    else:
        head = f"规格 {len(edits)} 处、{len(relatives)} 份文件（{'、'.join(relatives)}）；喂的钩子 {hook_relative}；喂法 {len(feeds)} 条"
        tail = f"喂法输出 {log_path}；反向规格 {reverse_spec_path}；结局文件 {outcome_file}"
    lines = [head,
             *(f"  {relative} 换上之前 sha256 {sha256_of(base_bytes[relative])}，换上之后应当是 {sha256_of(planned[relative])}" for relative in relatives),
             tail]
    tree = os.path.join(work, "tree")
    failing_neutral_feeds = []
    try:
        if script_spec:
            build_scripts_draft_tree(tree, planned)
        else:
            build_draft_tree(root, tree, {})
            failing_neutral_feeds = neutral_feeds_failing_on_base(tree, hook_relative, feeds, log_path, detections_path, mode)
            if not failing_neutral_feeds:
                write_planned_texts(tree, planned)
        if not failing_neutral_feeds:
            draft_problems, draft_summary = check_tree(tree, relatives, hook_relative, feeds, log_path, detections_path, mode)
    finally:
        shutil.rmtree(tree, ignore_errors=True)
    if failing_neutral_feeds:
        return EXIT_NEUTRAL_FEED_STALE, [*lines, *neutral_feed_stale_lines(failing_neutral_feeds, log_path)]
    names_problems, names_summary, names_unjudged = caller_names_check(root, relatives, planned, mode)
    draft_problems += names_problems
    draft_summary += f"、{names_summary}" if names_summary else ""
    lines += unjudged_names_lines(names_unjudged)
    if draft_problems:
        lines += [f"✗ 草稿里验不过（{draft_summary}），主工作区一个字节没写：", *(f"  - {problem}" for problem in draft_problems),  # gate-lint:summary
                  f"→ 怎么办：照上面几条改规格再跑" + ("" if script_spec else f"；喂法的整段 stderr 在 {log_path}")
                  + ("；缺的名字补回库里，或把引它的调用方连同这份库写进同一份规格（调用方按规格换上之后的文本核）" if names_problems else "")]
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
    # 结局文件：动手换之前先写开始与每份的换前、换后 sha256，全部换完、回读、再验过才写完成；换到一半被杀，下一趟读它就知道停在哪（文件头「结局文件」）
    outcome_started = False

    def record(rows):
        """结局文件追加几行；还没写开始行的先写（弄坏开关 outcome-start-omitted 下开始行拖到第一次追加的时候）。"""
        nonlocal outcome_started
        if not outcome_started:
            outcome_started = True
            write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)
        append_outcome(outcome_file, rows)
    if mode != "outcome-start-omitted":
        outcome_started = True
        write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)
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
            if not problem_lines:
                record([("退回", len(swapped), int(time.time()))])
            lines += [f"✗ 换上 {relative} 时失败（{str(error).splitlines()[0]}）；已经换上的 {len(swapped)} 份回滚了 {len(rolled)} 份", *problem_lines,
                      "→ 怎么办：照上面的原因处理那一份再跑；有回滚不成的，照写出的几处用 Edit 改回"]
            return (EXIT_ROLLBACK_FAILED if problem_lines else EXIT_SWAP_FAILED), lines
        swapped.append((relative, path))
        if after_file_swapped is not None:
            after_file_swapped(relative)
    readback = [relative for relative in relatives if read_text(os.path.join(root, relative)) != planned[relative]]
    record([("回读", relative, current_digest(root, relative)) for relative in relatives])
    if after_swap is not None:
        after_swap()
    if mode == "skip-post-swap-check":
        record([("完成", len(swapped), int(time.time()))])
        lines.append(f"✓ 换上了 {len(swapped)} 份（弄坏开关：换上之后没再验）")
        return EXIT_SWAPPED, lines
    live_problems, live_summary = check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode)
    live_problems = [f"{relative} 换上之后回读与草稿里验过的不同" for relative in readback] + live_problems
    live_names_problems, live_names_summary, live_names_unjudged = caller_names_check(root, relatives, {}, mode)
    live_problems += live_names_problems
    live_summary += f"、{live_names_summary}" if live_names_summary else ""
    lines += unjudged_names_lines([item for item in live_names_unjudged if item not in names_unjudged])
    if script_spec and verify and mode != "script-verify-skipped":
        failed = verify_problem(verify, root, verify_log_path)
        if failed:
            live_problems.append(failed)
        else:
            live_summary += f"、验证命令退 0（{verify[:80]}）"
    if not live_problems:
        record([("完成", len(swapped), int(time.time()))])
        lines.append(f"✓ 换上了 {len(swapped)} 份（改名换上新 inode、回读一致），主工作区里再验过：{live_summary}")
        return EXIT_SWAPPED, lines
    lines += [f"✗ 换上之后在主工作区验不过（{live_summary}）：", *(f"  - {problem}" for problem in live_problems)]  # gate-lint:summary
    problem_lines, rolled = rollback(swapped, edits, base_texts, mode)
    if not problem_lines:
        record([("退回", len(swapped), int(time.time()))])
    if problem_lines:
        lines += [f"✗ 回滚只成了 {len(rolled)} 份：", *problem_lines,
                  f"→ 怎么办：照上面写出的几处用 Edit 改回（钩子在 bash 这一层坏了时 Bash 调用会被拒，Edit / Write 不经它）；反向规格 {reverse_spec_path}"]
        return EXIT_ROLLBACK_FAILED, lines
    after_problems, after_summary = check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode, only_neutral=True)
    if after_problems:
        lines += [f"✗ 已按反向定点替换回滚 {len(rolled)} 份，回滚之后还验不过（{after_summary}）：", *(f"  - {problem}" for problem in after_problems),  # gate-lint:summary
                  f"→ 怎么办：{'这几份脚本' if script_spec else '钩子'}现在可能是坏的；看 {verify_log_path if script_spec else log_path} 里的输出，用 Edit 修，交主 agent"]
        return EXIT_ROLLBACK_FAILED, lines
    lines.append(f"→ 已按反向定点替换回滚 {len(rolled)} 份（{'、'.join(rolled)}），回滚之后验过（{after_summary}）；"
                 f"照上面几条改规格，必要时 --dry-run 先看草稿，再跑" + ("" if script_spec else f"；喂法的整段 stderr 在 {log_path}"))
    return EXIT_ROLLED_BACK, lines
