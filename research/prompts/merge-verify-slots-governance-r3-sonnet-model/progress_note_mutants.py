#!/usr/bin/env python3
# admission: always 模型只读被判文件、每次现取 progress_note 的源码改坏再判，上一次的读数不替这一次作保
# run-condition: command python3
"""正推腿的小模型：从被判文件 scripts/agents/agent_watch_alerts.py 里按语法树取出 progress_note 的源码，逐个改坏，
对两组断言判红绿，回答「新加的那一格分不分得出取头与取尾、别的改坏有没有照样绿」。
  真自证那一组（real）：照抄真自证里碰 progress_note 的四处——selftest_scratch_mentions 新加的那一格（多于 PROGRESS_TAIL_LINES 行）、
    selftest_session_watch_in 的整点询问附末几行、selftest_scratch_mentions 里没有「草稿目录：」一行取不到、tests/test_agents.py 的读之前文件已消失；
  提议那一组（proposed）：真自证没有的几格（空行与首尾空白、超长行、次序、只有一行、空文件、改于几秒前、读文件抛 OSError）。
替身只有四个：progress_file_of 取 transcript 上的路径、own_scratch_directories 取 transcript 上的目录、readable_mtime 取 os.stat、format_duration 取四舍五入的秒数；
其余原样用真源码，所以这里的红绿是模型替身上的读数，不是真自证的结局（真自证要主 agent 在副本里经 task-server/task submit 跑 agent-watch.py --selftest）。
用法：python3 progress_note_mutants.py --repo <仓根> --scratch <新的空目录>
输出行：MODEL_BASELINE real=<绿/红> proposed=<绿/红>；MODEL_MUTANT name=<名> real_red=<红的格，逗号隔开，无则 none> proposed_red=<同> verdict=<killed-by-real|only-proposed-kills|survived-everything>；
  MODEL_SUMMARY 各类个数；改坏逐个只许命中一处（命中数不是 1 的退 1）。
退出码：0 基线两组全绿、每个改坏命中恰好一处；1 基线红或锚点没命中；2 用法错。
"""
import argparse
import ast
import datetime
import os
import sys
import types

REAL_CELLS = ("tail-cell-more-lines-than-limit", "inquiry-attaches-last-line", "no-scratch-line-says-unreachable", "file-vanished-before-read")
PROPOSED_CELLS = ("blank-lines-whitespace-order-and-truncation", "single-line-file", "empty-file", "directory-without-file", "age-in-seconds",
                  "invalid-utf8-bytes", "open-raises-os-error")
VANISHED = "；progress.md 在检查期间已消失或不可读"


def extract(repository):
    path = os.path.join(repository, "scripts", "agents", "agent_watch_alerts.py")
    text = open(path, encoding="utf-8").read()
    source, tail_lines = None, None
    for node in ast.parse(text).body:
        if isinstance(node, ast.FunctionDef) and node.name == "progress_note":
            source = ast.get_source_segment(text, node)
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "PROGRESS_TAIL_LINES" for target in node.targets):
            tail_lines = ast.literal_eval(node.value)
    if source is None or not isinstance(tail_lines, int):
        raise SystemExit(f"  ✗ {path} 里取不到 progress_note 或 PROGRESS_TAIL_LINES")
    return source, tail_lines


def readable_mtime(path):
    try:
        return os.stat(path).st_mtime
    except OSError:
        return None


def build(source, tail_lines, broken):
    namespace = {
        "BROKEN_DETECTION": broken, "PROGRESS_TAIL_LINES": tail_lines,
        "datetime": datetime.datetime, "timezone": datetime.timezone,
        "progress_file_of": lambda transcript: transcript.progress_path,
        "own_scratch_directories": lambda transcript: transcript.scratch_directories,
        "readable_mtime": readable_mtime,
        "format_duration": lambda seconds: f"{round(seconds)}秒",
    }
    exec(compile(source, "progress_note", "exec"), namespace)
    return namespace["progress_note"]


def transcript(path, directories=("x",)):
    return types.SimpleNamespace(progress_path=path, scratch_directories=list(directories))


def write(path, content, binary=False):
    with open(path, "wb" if binary else "w", **({} if binary else {"encoding": "utf-8"})) as handle:
        handle.write(content)
    return path


def now():
    return datetime.datetime.now(datetime.timezone.utc)


def run_cells(note_of, tail_lines, scratch):
    """→ (真自证那一组红的格, 提议那一组红的格)。格里抛异常算红。"""
    def tail_cell():
        total = tail_lines + 2
        path = write(os.path.join(scratch, "tail.md"), "".join(f"进度行{index:02d}\n" for index in range(1, total + 1)))
        note = note_of(transcript(path), now())
        kept = [f"进度行{index:02d}" for index in range(total - tail_lines + 1, total + 1)]
        dropped = [f"进度行{index:02d}" for index in range(1, total - tail_lines + 1)]
        return f"末 {tail_lines} 行" in note and all(line in note for line in kept) and not any(line in note for line in dropped)

    def inquiry_cell():
        path = write(os.path.join(scratch, "inquiry.md"), "第一格跑完\n第二格跑完\n第三格在跑，预计 40 分钟\n")
        return "第三格在跑，预计 40 分钟" in note_of(transcript(path), now())

    def no_line_cell():
        return "没有行首「草稿目录：」一行，取不到 progress.md" in note_of(transcript(None, ()), now())

    def vanished_cell():
        return "已消失或不可读" in note_of(transcript(os.path.join(scratch, "missing.md")), now())

    def formatting_cell():
        long_line = "乙" * 130
        path = write(os.path.join(scratch, "format.md"), f"甲行\n\n{long_line}\n丙行  \n\n丁行\n\n\n")
        wanted = f"末 3 行：{'乙' * 120} ｜ 丙行 ｜ 丁行"
        return wanted in note_of(transcript(path), now())

    def single_line_cell():
        return "末 1 行：唯一一行" in note_of(transcript(write(os.path.join(scratch, "one.md"), "唯一一行\n")), now())

    def empty_cell():
        return "末 0 行：（空）" in note_of(transcript(write(os.path.join(scratch, "empty.md"), "")), now())

    def directory_cell():
        return "；派发提示点名的草稿目录下没有 progress.md" in note_of(transcript(None, ("x",)), now())

    def age_cell():
        path = write(os.path.join(scratch, "age.md"), "一行\n")
        moment = now().timestamp() - 600
        os.utime(path, (moment, moment))
        fixed_now = datetime.datetime.fromtimestamp(moment + 600, datetime.timezone.utc)
        return "改于 600秒 前" in note_of(transcript(path), fixed_now)

    def invalid_bytes_cell():
        path = write(os.path.join(scratch, "bytes.md"), b"\xff\xfe\xe5\x89\x8d\n", binary=True)
        return "末 1 行" in note_of(transcript(path), now())

    def os_error_cell():
        directory = os.path.join(scratch, "a-directory")
        os.makedirs(directory, exist_ok=True)
        return VANISHED in note_of(transcript(directory), now())

    def judge(cells):
        red = []
        for name, cell in cells:
            try:
                green = cell()
            except Exception:  # noqa: BLE001 格里抛什么都算红
                green = False
            if not green:
                red.append(name)
        return red

    return (judge(zip(REAL_CELLS, (tail_cell, inquiry_cell, no_line_cell, vanished_cell))),
            judge(zip(PROPOSED_CELLS, (formatting_cell, single_line_cell, empty_cell, directory_cell, age_cell, invalid_bytes_cell, os_error_cell))))


# 改坏表：(名, 旧串, 新串)；旧串在 progress_note 源码里要恰好出现一次。
MUTANTS = (
    ("tail-takes-one-more-line", "filled_lines[-PROGRESS_TAIL_LINES:]", "filled_lines[-PROGRESS_TAIL_LINES - 1:]"),
    ("tail-drops-the-last-line", "filled_lines[-PROGRESS_TAIL_LINES:]", "filled_lines[-PROGRESS_TAIL_LINES:-1]"),
    ("first-line-plus-last-two", "filled_lines[-PROGRESS_TAIL_LINES:]", "filled_lines[:1] + filled_lines[-PROGRESS_TAIL_LINES + 1:]"),
    ("tail-in-reverse-order", "filled_lines[-PROGRESS_TAIL_LINES:]", "filled_lines[-PROGRESS_TAIL_LINES:][::-1]"),
    ("blank-lines-kept", "if line.strip()]", "if True]"),
    ("lines-not-stripped", "[line.strip() for line in handle.read().splitlines()", "[line for line in handle.read().splitlines()"),
    ("no-truncation", "line[:120]", "line"),
    ("truncation-at-twenty", "line[:120]", "line[:20]"),
    ("truncation-at-three", "line[:120]", "line[:3]"),
    ("empty-text-blank", "'（空）'", "''"),
    ("count-is-the-constant", "末 {len(lines)} 行", "末 {PROGRESS_TAIL_LINES} 行"),
    ("directory-without-file-text-dropped", "\"；派发提示点名的草稿目录下没有 progress.md\"", "\"；\""),
    ("age-always-zero", "format_duration((now - modified).total_seconds())", "format_duration(0)"),
    ("age-sign-flipped", "(now - modified)", "(modified - now)"),
    ("invalid-utf8-raises", "errors=\"replace\"", "errors=\"strict\""),
    ("join-without-separator", "' ｜ '.join", "''.join"),
    ("unreadable-mtime-text-dropped", "if moment is None:\n            return \"" + VANISHED + "\"", "if moment is None:\n            return \"\""),
    ("os-error-branch-text-dropped", "except OSError:\n        return \"" + VANISHED + "\"", "except OSError:\n        return \"\""),
)
SWITCHES = (("switch-progresstailhead", "progresstailhead"), ("switch-progressnote", "progressnote"))


def main(arguments):
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--repo")
    parser.add_argument("--scratch")
    try:
        parsed = parser.parse_args(arguments)
    except SystemExit:
        parsed = None
    if parsed is None or not parsed.repo or not parsed.scratch:
        print("用法：progress_note_mutants.py --repo <仓根> --scratch <新的空目录>", file=sys.stderr)
        return 2
    if os.path.exists(parsed.scratch) and os.listdir(parsed.scratch):
        print(f"  ✗ --scratch {parsed.scratch} 已存在且非空：给一个新的目录", file=sys.stderr)
        return 2
    source, tail_lines = extract(parsed.repo)
    exit_code = 0
    counter = 0

    def fresh_scratch():
        nonlocal counter
        counter += 1
        directory = os.path.join(parsed.scratch, f"run{counter:02d}")
        os.makedirs(directory)
        return directory

    real_red, proposed_red = run_cells(build(source, tail_lines, ""), tail_lines, fresh_scratch())
    print(f"MODEL_BASELINE tail_lines={tail_lines} real_red={','.join(real_red) or 'none'} proposed_red={','.join(proposed_red) or 'none'}")
    if real_red or proposed_red:
        print("  ✗ 基线就红：替身与真函数对不上，下面的读数不作数")
        exit_code = 1
    verdicts = {}
    variants = []
    for name, old, new in MUTANTS:
        hits = source.count(old)
        if hits != 1:
            print(f"  ✗ 改坏 {name} 的旧串在 progress_note 源码里命中 {hits} 次（要恰好 1 次）")
            exit_code = 1
            continue
        variants.append((name, source.replace(old, new), ""))
    variants += [(name, source, broken) for name, broken in SWITCHES]
    for name, mutated, broken in variants:
        real_red, proposed_red = run_cells(build(mutated, tail_lines, broken), tail_lines, fresh_scratch())
        verdict = "killed-by-real" if real_red else ("only-proposed-kills" if proposed_red else "survived-everything")
        verdicts[name] = verdict
        print(f"MODEL_MUTANT name={name} real_red={','.join(real_red) or 'none'} proposed_red={','.join(proposed_red) or 'none'} verdict={verdict}")
    summary = {kind: sum(1 for value in verdicts.values() if value == kind) for kind in ("killed-by-real", "only-proposed-kills", "survived-everything")}
    print(f"MODEL_SUMMARY mutants={len(verdicts)} killed-by-real={summary['killed-by-real']} only-proposed-kills={summary['only-proposed-kills']} "
          f"survived-everything={summary['survived-everything']}")
    return exit_code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
