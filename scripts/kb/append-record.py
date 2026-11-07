#!/usr/bin/env python3
# admission: always 每一次追加都是对此刻那份记录做一次写入，上一次判过的文本不替这一次作保
# run-condition: none 只读写仓里 records/ 下点名的那一份记录与判据库，除了 python3 之外没有环境要求
"""往 records/ 下一份已有的 .md 末尾追加一段文本：写之前照门禁 doc-text 的 clock-times、session-names 与批次代号同一份判据判这段文本，
命中就拒、点名第几行；读时记 sha256、换上之前复核、同目录临时文件加改名换上。
Bash 里往 records/ 下的 .md 追加（>>、&>>、tee -a、python 的 open(…, 'a')）由 .claude/hooks/bash-command-detector.sh 的 ⑲ 在执行前拒，出路指到这里。

用法：
  append-record.py <记录> <文本文件> [--dry-run] [--root 仓根]
  append-record.py <记录> - [--dry-run] [--root 仓根]     # 文本从标准输入读
  append-record.py --selftest

  <记录>      records/ 下一份已有的 .md（后缀不分大小写，子目录里的也收；相对路径从当前目录起算，解开符号链接之后要落在 <仓根>/records/ 下）。
              新记录用 Write 新建（写闸 write-guard.sh 照判钟点、会话号与批次代号），之后再经这里追加。
  <文本文件>  要追加的文本，UTF-8。记录不以换行收尾的先补一个换行，文本不以换行收尾的补一个；别的不加（要空一行就写进文本）。
              文本里不许有一、二级标题（去掉 0–3 个前导空格之后以 # 或 ## 加空白或行尾起头的行；四个空格以上缩进的是代码，不算；
              围栏代码块里的不算，围栏按 scripts/kb/lib_markdown.py 认：0–3 个空格缩进的 ``` 或 ~~~ 起、同字符且不短于开头的那一行收，没收的护到文末；
              三级以下照写）：新开一节用 Edit 接在文件末尾（写闸照判同几样），
              拆分提案现行账本加节经 scripts/kb/ledger-append-section.py（它现取节号、索引行与节一次写入）。
  --root      仓根，默认是这份脚本所在的仓；判据库取 <仓根>/.claude/gate.d/lib-forbidden-notations.py。
  判文本：钟点、时区词、时间戳（scripts/kb/lib_notation_check.py 的 clock_refusal）与会话号、批次代号（同一份的 session_name_refusal），
  与 insert-row.py、ledger-append-section.py、写闸 write-guard.sh 的第五道与第七道、门禁 doc-text 的 clock-times 与 session-names 格读同一份判据库，这里不另抄。
  拒绝说明里的「第 N 行」是追加之后那一行在记录里的行号。

写之前复核：读记录时记下 sha256，临时文件写完、改名换上之前再算一次，变了就拒（别的会话这段时间里写过，重跑即可）；写回走 lib_atomic_replace.py
（同目录排他新建临时文件、fsync、照原权限位、改名换上），正在读这份记录的进程读的仍是旧 inode 的旧内容。复核到改名之间还有一小段空档，与 insert-row.py 同。

退出码：0 追加了（或干跑）；1 用法不对、文本文件读不到或不是 UTF-8、自检没过；2 记录不在 <仓根>/records/ 下、不是 .md、不在或读不到；
4 读到写之间记录被改过（没写，重跑）；5 没换上（硬链接、只读、建不了临时文件、写或改名失败）；6 文本带钟点、时区词或时间戳；
7 文本带会话号或批次代号；8 文本或记录不合（文本是空的、有一二级标题、带 CR；记录带 CR 或不是 UTF-8）。2–8 都是记录没动、没留临时文件。
9 已经追加了（改名换上做完），之后被 Ctrl-C 打断（lib_atomic_replace.py 的 ReplacedThenInterrupted / ReplacedThenFailed）：别重跑（会追加两遍），先看记录末尾。

弄坏开关（只给 --selftest 证红用，平常跑设了退 2、点名只在自证里认；不认识的值同样退 2）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（括号里是那条「✗ 自检失败：」的开头）：
  APPEND_RECORD_BREAK=no-recheck-before-write  换上之前不复核 sha256，读到写之间别人写进去的那一行被盖掉
      读到写之间被改过（「✗ 自检失败：读到写之间被改过没拒」）
  APPEND_RECORD_BREAK=no-notation-guard  不判文本里的钟点、时区词、时间戳、会话号与批次代号
      写之前判文本（「✗ 自检失败：写之前判文本没判对」）
  APPEND_RECORD_BREAK=headings-allowed  不拒文本里的一、二级标题
      拒绝的几种输入（「✗ 自检失败：拒绝的几种输入没判对」）
  APPEND_RECORD_BREAK=records-only-unchecked  不核记录落不落在 <仓根>/records/ 下、是不是 .md
      只认 records/ 下的 .md（「✗ 自检失败：只认 records/ 下已有的 .md 没判对」）
  APPEND_RECORD_BREAK=fence-toggle  围栏按布尔翻转认：去掉任意多前导空格之后以 ``` 或 ~~~ 起头的行都翻一次，不记字符与长度
      正常追加（「✗ 自检失败：正常追加没成」：四个反引号的围栏里那一行 ``` 被当成收尾，后面围栏里的二级标题被拒）
  APPEND_RECORD_BREAK=heading-indent-unstripped  一、二级标题按没去前导空格的行认
      拒绝的几种输入（「✗ 自检失败：拒绝的几种输入没判对」：两个空格缩进的二级标题没拒）
  ATOMIC_REPLACE_BREAK=rawafterrename  共用库 lib_atomic_replace.py 的弄坏开关（写法在它文件头）：改名之后到的 Ctrl-C 原样抛，自证同样走它
      写回的方式（「✗ 自检失败：写回的方式不对」，下面几行以「[改名之后被打断]」起头）
"""
import argparse
import contextlib
import hashlib
import io
import os
import re
import subprocess
import sys
import tempfile

from lib_atomic_replace import ReplaceRefused, ReplacedThenFailed, ReplacedThenInterrupted, inside_selftest, leftover_temporary_files, \
    problems_with_replacement_by_rename, replace_file_contents_by_rename
from lib_markdown import fenced_flags
from lib_notation_check import clock_refusal, copy_library_into, repository_root, session_name_refusal
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "APPEND_RECORD_BREAK"
BROKEN = os.environ.get(BREAK_VARIABLE, "")
KNOWN_BREAKS = ("no-recheck-before-write", "no-notation-guard", "headings-allowed", "records-only-unchecked", "fence-toggle", "heading-indent-unstripped")
RECORDS_DIRECTORY = "records"
# 一、二级标题：0–3 个前导空格之后 # 或 ##，再接空白或行尾（四个空格以上缩进的是代码；三级以下不算）；与 ledger-append-section.py 的 TOP_HEADING 同一种认法
TOP_HEADING = re.compile(r" {0,3}#{1,2}(?:[ \t]|$)")
TOP_HEADING_WITHOUT_INDENT = re.compile(r"#{1,2}(?:[ \t]|$)")   # 弄坏开关 heading-indent-unstripped 用的认法
FENCE_OPENERS = ("```", "~~~")   # 弄坏开关 fence-toggle 用的认法
LEDGER_TOOL = "scripts/kb/ledger-append-section.py"
SELF_TOOL = "scripts/kb/append-record.py"


class Refused(Exception):
    """没写：code 是退出码，lines 是说明（第一行摘要，之后的行带缩进与「→」）。"""

    def __init__(self, code, *lines):
        super().__init__(lines[0])
        self.code = code
        self.lines = lines


class ChangedSinceRead(Exception):
    """写前复核：读到写之间记录的 sha256 变了。"""

    def __init__(self, digest_now):
        super().__init__(digest_now)
        self.digest_now = digest_now


def record_target(record, root):
    """<记录> → (解开符号链接之后的绝对路径, 相对仓根的路径)。不在 <root>/records/ 下、不是 .md、不是已有的普通文件都抛 Refused(2)。
    <root>/records 本身是链接的，比的是它指到的目录（与 bash-command-detector.sh ⑲ 的 record_append_landing 同一种比法）。"""
    real = os.path.realpath(record)
    records = os.path.realpath(os.path.join(root, RECORDS_DIRECTORY))
    relative = os.path.join(RECORDS_DIRECTORY, os.path.relpath(real, records)) if real.startswith(records + os.sep) else real
    if BROKEN == "records-only-unchecked":   # 弄坏开关：不核落点
        return real, relative
    if not real.startswith(records + os.sep) or not real.lower().endswith(".md"):
        raise Refused(2, f"{record}（解开符号链接之后是 {real}）不是 {records}/ 下的 .md，一个字节没写",
                      "     → 怎么办：这个工具只往 records/ 下的记录追加；别的文件用 Edit 改，或用 scripts/kb/insert-row.py、replace-once.py 定点改")
    if not os.path.isfile(real):
        raise Refused(2, f"{relative} 不在（或不是普通文件），一个字节没写",
                      "     → 怎么办：新记录用 Write 新建（写闸照判钟点、会话号与批次代号），之后再经这个工具追加；路径写错了的改对再跑")
    return real, relative


def text_problem(text):
    """要追加的文本不合的一处 → 说明；合格交 None。"""
    if not text.strip():
        return "要追加的文本是空的"
    if "\r" in text:
        return "要追加的文本带 CR（CRLF 换行），这个工具只认 LF"
    if BROKEN == "headings-allowed":   # 弄坏开关：不拒一、二级标题
        return None
    heading = TOP_HEADING_WITHOUT_INDENT if BROKEN == "heading-indent-unstripped" else TOP_HEADING   # 弄坏开关：标题不去前导空格
    lines = text.split("\n")
    for number, (line, fenced) in enumerate(zip(lines, fenced_line_flags(lines)), 1):
        if not fenced and heading.match(line):
            return f"文本第 {number} 行是一、二级标题「{line[:40]}」"
    return None


def fenced_line_flags(lines):
    """每一行在不在代码围栏里（开合两行也算在里面）：lib_markdown.py 的 fenced_flags（记字符与长度、只认 0–3 个空格缩进、没收的护到文末）。
    弄坏开关 fence-toggle 换成布尔翻转的认法：去掉任意多前导空格之后以 ``` 或 ~~~ 起头的行都翻一次。"""
    if BROKEN != "fence-toggle":
        return fenced_flags(lines)
    flags, inside = [], False
    for line in lines:
        if line.lstrip(" ").startswith(FENCE_OPENERS):
            inside = not inside
            flags.append(True)
        else:
            flags.append(inside)
    return flags


def plan(original, text):
    """记录此刻的文本 + 要追加的文本 → (新文本, 补过换行的追加文本, 它在新文件里的首行行号, 末行行号)。不合抛 Refused(8)。"""
    if "\r" in original:
        raise Refused(8, "记录带 CR（CRLF 换行），这个工具只认 LF，一个字节没写",
                      "     → 怎么办：记录一直是 LF；先查是谁把它改成了 CRLF，改回 LF 再追加")
    problem = text_problem(text)
    if problem:
        raise Refused(8, f"{problem}，一个字节没写",
                      "     → 怎么办：文本非空、只用 LF、不带一二级标题（三级以下可以）；新开一节用 Edit 接在记录末尾（写闸照判钟点、会话号与批次代号），"
                      f"拆分提案现行账本加节经 {LEDGER_TOOL}，改完重跑")
    head = original if original == "" or original.endswith("\n") else original + "\n"
    body = text if text.endswith("\n") else text + "\n"
    first_line = head.count("\n") + 1
    return head + body, body, first_line, first_line + body.count("\n") - 1


def notation_refusal(path, body, first_line, notation_root):
    """钟点与会话号、批次代号：与 insert-row.py、ledger-append-section.py 同一份判据（lib_notation_check.py）；放行交 None，拒绝交 Refused。"""
    if BROKEN == "no-notation-guard":   # 弄坏开关：不判
        return None
    refusal = clock_refusal(path, body, first_line, notation_root)
    if refusal:
        return Refused(6, "\n".join(refusal), "  → 怎么办：文本里的钟点、时区词与时间戳删掉，只写日期，再追加")
    refusal = session_name_refusal(path, body, first_line, notation_root)
    if refusal:
        return Refused(7, "\n".join(refusal), "  → 怎么办：会话号换成「别的会话」这类不带号的说法，批次代号换成那一批做了什么，事情与日期照留，再追加")
    return None


def append_text(path, text, dry_run=False, notation_root=None, interleave=None):
    """记录 path 末尾追加 text → 退出码，说明打在标准输出。落点在不在 records/ 下由 record_target 核，这里不核。
    interleave 只给自证用：读完、判完之后、换上之前调一次（自证在这里往记录里另写一行，造「读到写之间被改过」）。"""
    try:
        with open(path, "rb") as handle:
            original = handle.read()
    except OSError as error:
        print(f"  ✗ 读不到记录 {path}（{error}），一个字节没写")
        print("     → 怎么办：检查路径与权限；记录是不是被改名或删了")
        return 2
    before = hashlib.sha256(original).hexdigest()
    try:
        try:
            decoded = original.decode("utf-8")
        except UnicodeDecodeError as error:
            raise Refused(8, f"记录 {path} 不是 UTF-8（{error}），一个字节没写", "     → 怎么办：先查记录是被谁写坏的，改回 UTF-8 再追加") from error
        new_text, body, first_line, last_line = plan(decoded, text)
        refusal = notation_refusal(path, body, first_line, notation_root)
        if refusal:
            raise refusal
    except Refused as refused:
        print("  ✗ " + "\n".join(refused.lines))  # Refused 的说明第二行起是带「→」的出路
        return refused.code
    if dry_run:
        print(f"  （干跑）会往 {path} 追加 {last_line - first_line + 1} 行（第 {first_line}–{last_line} 行），文本判过了，没写")
        return 0

    def verify_unchanged_since_read():
        if BROKEN == "no-recheck-before-write":   # 弄坏开关：不复核
            return
        with open(path, "rb") as handle:
            digest_now = hashlib.sha256(handle.read()).hexdigest()   # 每次现算；排在临时文件写完之后、改名换上之前
        if digest_now != before:
            raise ChangedSinceRead(digest_now)

    try:
        if interleave is not None:
            interleave()
        replaced = replace_file_contents_by_rename(path, new_text, check_before_rename=verify_unchanged_since_read)
    except ChangedSinceRead as changed:
        print(f"  ✗ 读到写之间 {path} 被改过（sha256 {before[:12]} → {changed.digest_now[:12]}），没写")
        print("     → 怎么办：别的会话刚往这份记录写过，重跑本命令（文本接在它写的后面）")
        return 4
    except ReplaceRefused as refused:
        print("  ✗ " + str(refused))  # str(ReplaceRefused) 第二行是「→」起头的出路（lib_atomic_replace.py 里每个 ReplaceRefused 都带着）
        return 5
    except (ReplacedThenInterrupted, ReplacedThenFailed) as replaced_then_failed:
        print(f"  ✗ {path} 已经追加了第 {first_line}–{last_line} 行（改名换上做完），之后被打断"
              f"（{replaced_then_failed.__class__.__name__}，原因：{replaced_then_failed.__cause__!r}）")
        print("     → 怎么办：别重跑本命令（会追加两遍）；先看记录末尾，追加的文本在就算写完了")
        return 9
    print(f"  ✓ 追加了 {last_line - first_line + 1} 行到 {path}（第 {first_line}–{last_line} 行）；写前复核 sha256 {before[:12]} 未变，改名换上新 inode")
    for note in replaced:
        print("  ! " + note)
    return 0


def append_record(record, text, root, dry_run=False):
    """入口的那一套：核落点、再追加 → 退出码。"""
    try:
        path, _ = record_target(record, root)
    except Refused as refused:
        print("  ✗ " + "\n".join(refused.lines))  # Refused 的说明第二行起是带「→」的出路
        return refused.code
    return append_text(path, text, dry_run, notation_root=root)


# ───────────────────────────── 自证 ─────────────────────────────

SAMPLE_RECORD = "# 样本调度记录\n\n## 进度\n\n- 甲做完\n"   # 五行


def make_root(work, name):
    """假仓：records/、scripts/ 与一份真判据库的拷贝；records/run.md 是样本记录。"""
    root = os.path.join(work, name)
    for directory in (RECORDS_DIRECTORY, "scripts"):
        os.makedirs(os.path.join(root, directory))
    copy_library_into(root)
    record = os.path.join(root, RECORDS_DIRECTORY, "run.md")
    write(record, SAMPLE_RECORD)
    return root, record


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def read(path):
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def quietly(call):
    """跑 call()，交回 (退出码, 打到标准输出的说明)。"""
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = call()
    return code, captured.getvalue()


def problems_with_plain_append(work):
    """正常追加：接在末尾、补换行、三级标题与围栏里的二级标题照写、行号说对、干跑不写、不留临时文件。"""
    problems = []
    root, record = make_root(work, "plain")
    text = "- 乙做完\n### 三级标题照写\n```\n## 围栏里的不算\n```"
    code, said = quietly(lambda: append_record(record, text, root))
    if code != 0 or read(record) != SAMPLE_RECORD + text + "\n" or "第 6–10 行" not in said or leftover_temporary_files(os.path.dirname(record)) != []:
        problems.append(f"五行的记录追加五行（不以换行收尾）：本该退 0、接在末尾补一个换行、说第 6–10 行、不留临时文件，实际退 {code}：{said!r}；文件尾部 {read(record)[-80:]!r}")
    write(record, SAMPLE_RECORD.rstrip("\n"))
    code, said = quietly(lambda: append_record(record, "- 乙做完\n", root))
    if code != 0 or read(record) != SAMPLE_RECORD + "- 乙做完\n":
        problems.append(f"记录不以换行收尾：本该先补一个换行再接，实际退 {code}：{said!r}；文件尾部 {read(record)[-40:]!r}")
    before = read(record)
    code, said = quietly(lambda: append_record(record, "- 丙做完\n", root, dry_run=True))
    if code != 0 or "（干跑）" not in said or read(record) != before:
        problems.append(f"干跑：本该退 0、说「（干跑）」、记录不动，实际退 {code}：{said!r}")
    # 围栏按字符与长度收：四个反引号开的围栏里，三个反引号那一行不收尾，后面的「## 」仍在代码里
    before = read(record)
    nested = "````\n```\n## 仍在围栏里\n````\n"
    code, said = quietly(lambda: append_record(record, nested, root))
    if code != 0 or read(record) != before + nested:
        problems.append(f"四个反引号的围栏里有一行三个反引号、再一行「## 」：本该照写（三个反引号收不了四个反引号开的围栏），实际退 {code}：{said!r}"
                        "（APPEND_RECORD_BREAK=fence-toggle 设着的话这一条本来就该红）")
    return problems


def problems_with_target_checks(work):
    """只认 <仓根>/records/ 下已有的 .md：别处的、别的后缀、不在的、经 .. 或符号链接爬出去的拒（退 2、文件不动）；子目录、大写后缀、指进 records 的链接放行。"""
    problems = []
    root, record = make_root(work, "target")
    outside = os.path.join(work, "outside.md")
    for path in (os.path.join(root, RECORDS_DIRECTORY, "table.tsv"), os.path.join(root, "scripts", "notes.md"), outside,
                 os.path.join(root, "records-old", "run.md"), os.path.join(root, RECORDS_DIRECTORY, "sub", "notes.md"),
                 os.path.join(root, RECORDS_DIRECTORY, "UPPER.MD")):
        write(path, SAMPLE_RECORD)
    os.symlink(outside, os.path.join(root, RECORDS_DIRECTORY, "link-out.md"))
    os.symlink(os.path.join("..", RECORDS_DIRECTORY, "run.md"), os.path.join(root, "scripts", "link-in.md"))
    for label, target, refused, landing in (
            ("records 下的 .tsv", os.path.join(root, RECORDS_DIRECTORY, "table.tsv"), True, None),
            ("records 之外的 .md", os.path.join(root, "scripts", "notes.md"), True, None),
            ("仓外的 .md", outside, True, None),
            ("records 同名前缀的兄弟目录", os.path.join(root, "records-old", "run.md"), True, None),
            ("records 下不在的 .md", os.path.join(root, RECORDS_DIRECTORY, "missing.md"), True, None),
            ("经 .. 爬出 records", os.path.join(root, RECORDS_DIRECTORY, "..", "scripts", "notes.md"), True, None),
            ("records 里指到仓外的符号链接", os.path.join(root, RECORDS_DIRECTORY, "link-out.md"), True, outside),
            ("records 子目录里的 .md", os.path.join(root, RECORDS_DIRECTORY, "sub", "notes.md"), False, None),
            ("大写后缀 .MD", os.path.join(root, RECORDS_DIRECTORY, "UPPER.MD"), False, None),
            ("records 外指进 records 的符号链接", os.path.join(root, "scripts", "link-in.md"), False, record)):
        landing = landing or target
        before = read(landing) if os.path.exists(landing) else None
        code, said = quietly(lambda: append_record(target, "- 乙做完\n", root))
        after = read(landing) if os.path.exists(landing) else None
        if refused and (code != 2 or after != before or "→ 怎么办" not in said):
            problems.append(f"{label}：本该退 2、说明带出路、文件一个字节不动，实际退 {code}：{said!r}")
        if not refused and (code != 0 or after != before + "- 乙做完\n"):
            problems.append(f"{label}：本该放行（退 0）并接在 {os.path.relpath(landing, root)} 末尾，实际退 {code}：{said!r}")
    if not os.path.islink(os.path.join(root, "scripts", "link-in.md")):
        problems.append("经符号链接追加之后链接本身被换成了普通文件（该换的是它指向的那一份）")
    return problems


def problems_with_refused_inputs(work):
    """文本或记录不合（退 8）：空文本、一二级标题（账本节标题另点名 ledger-append-section.py）、CR、记录 CRLF 或不是 UTF-8；记录一个字节不动。"""
    problems = []
    root, record = make_root(work, "refused")
    for label, text, content, wanted in (
            ("文本是空的", "\n\n", SAMPLE_RECORD, ""),
            ("文本有二级标题", "前\n## 新一节\n后\n", SAMPLE_RECORD, "一、二级标题"),
            ("文本有一级标题", "# 新文件头\n", SAMPLE_RECORD, "一、二级标题"),
            ("两个空格缩进的二级标题", "前\n  ## 缩进的新一节\n", SAMPLE_RECORD, "一、二级标题"),
            ("四个空格缩进的三个反引号（是代码、不是围栏）之后的二级标题", "    ```\n## 真标题\n", SAMPLE_RECORD, "一、二级标题"),
            ("文本有账本节标题", "## 一百九十、新节\n\n正文\n", SAMPLE_RECORD, LEDGER_TOOL),
            ("文本带 CR", "甲\r\n", SAMPLE_RECORD, "CR"),
            ("记录是 CRLF", "乙\n", SAMPLE_RECORD.replace("\n", "\r\n"), "CR"),
            ("记录不是 UTF-8", "乙\n", None, "UTF-8")):
        if content is None:
            with open(record, "wb") as handle:
                handle.write(b"\xff\xfe\n")
        else:
            write(record, content)
        with open(record, "rb") as handle:
            before = handle.read()
        code, said = quietly(lambda: append_record(record, text, root))
        with open(record, "rb") as handle:
            after = handle.read()
        if code != 8 or after != before or wanted not in said or "→ 怎么办" not in said or leftover_temporary_files(os.path.dirname(record)) != []:
            problems.append(f"{label}：本该退 8、说明里有「{wanted}」与出路、记录一个字节不动，实际退 {code}：{said!r}")
    return problems


def problems_with_notation_guard(work):
    """文本带时间戳拒（6），带会话号或批次代号拒（7），干跑也拒，点名命中的那一行；只写日期放行。
    样本里的会话名与时间戳拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）。"""
    problems = []
    glue = "".join
    root, record = make_root(work, "notation")
    stamp = glue(("20261014", "T153000Z"))
    session = glue(("singlefs-", "e152"))
    for label, text, wanted_code, wanted_texts in (
            ("第二行带紧凑时间戳", glue(("- 乙做完\n运行目录 ", stamp, "\n")), 6, (stamp, "第 7 行")),
            ("带会话号", glue(("已发消息，", session, " 回话\n")), 7, (session, "第 6 行")),
            ("带批次代号", "- 24c-1 交回\n", 7, ("写了批次代号", "第 6 行"))):
        for dry_run in (False, True):
            code, said = quietly(lambda: append_record(record, text, root, dry_run))
            if code != wanted_code or any(item not in said for item in wanted_texts) or "→ 怎么办" not in said or read(record) != SAMPLE_RECORD:
                problems.append(f"{label}{'（干跑）' if dry_run else ''}：本该退 {wanted_code}、点名{'、'.join(wanted_texts)}与出路、记录不动，实际退 {code}：{said!r}")
    code, said = quietly(lambda: append_record(record, "- 只写日期 2026-11-03 的一行\n", root))
    if code != 0:
        problems.append(f"只写日期的文本本该放行，实际退 {code}：{said!r}")
    return problems


def problems_with_interleaved_write(work):
    """读完、判完之后别的会话往记录里写了一行：换上之前复核 sha256 发现变了，拒（退 4），别人那一行留着、这一次的不写、不留临时文件。"""
    root, record = make_root(work, "interleaved")
    other = "- 别的会话写的一行\n"

    def interleave():
        with open(record, "a", encoding="utf-8") as handle:
            handle.write(other)

    code, said = quietly(lambda: append_text(record, "- 乙做完\n", notation_root=root, interleave=interleave))
    if code != 4 or read(record) != SAMPLE_RECORD + other or "读到写之间" not in said or leftover_temporary_files(os.path.dirname(record)) != []:
        return [f"本该退 4、说「读到写之间」、记录是原文加别人写的那一行、不留临时文件，实际退 {code}：{said!r}；记录尾部 {read(record)[-60:]!r}"]
    return []


def problems_with_write_back():
    return problems_with_replacement_by_rename(lambda target: append_text(target, "一句\n"), SAMPLE_RECORD, SAMPLE_RECORD + "一句\n",
                                               exit_code_when_replaced_then_interrupted=9)


def problems_with_entry(work):
    """从入口起这份脚本：相对路径从当前目录算、文本从文件读与从标准输入读都追加；带会话号退 7、打 ✗ 起头的说明；缺参数退 1。"""
    problems = []
    root, record = make_root(work, "entry")
    text_file = os.path.join(work, "entry-text.txt")
    write(text_file, "- 乙做完\n")
    session_file = os.path.join(work, "entry-session.txt")
    write(session_file, "".join(("会话 ", "singlefs-", "e152", " 回话\n")))
    script = os.path.realpath(__file__)
    for label, arguments, standard_input, wanted_code, wanted_record, wanted_start in (
            ("相对路径加文本文件", ["records/run.md", text_file], None, 0, SAMPLE_RECORD + "- 乙做完\n", "  ✓"),
            ("文本从标准输入读", ["records/run.md", "-"], "- 丙做完\n", 0, SAMPLE_RECORD + "- 乙做完\n- 丙做完\n", "  ✓"),
            ("文本带会话号", ["records/run.md", session_file], None, 7, SAMPLE_RECORD + "- 乙做完\n- 丙做完\n", "  ✗ 新串写了会话号"),
            ("缺文本参数", ["records/run.md"], None, 1, SAMPLE_RECORD + "- 乙做完\n- 丙做完\n", "  ✗ 用法")):
        finished = subprocess.run([sys.executable, script, "--root", root, *arguments], cwd=root, input=standard_input or "",
                                  capture_output=True, text=True, timeout=60)
        if finished.returncode != wanted_code or read(record) != wanted_record or not finished.stdout.startswith(wanted_start):
            problems.append(f"{label}：本该退 {wanted_code}、标准输出以「{wanted_start}」起头，实际退 {finished.returncode}：{finished.stdout!r}{finished.stderr!r}；"
                            f"记录尾部 {read(record)[-40:]!r}")
    return problems


def selftest():
    with tempfile.TemporaryDirectory(prefix="append-record-selftest-") as work:
        for label, check, remedy in (
                ("正常追加没成", lambda: problems_with_plain_append(work),
                 "查 plan() 与 append_text()：接在末尾、补换行、行号从追加之前的行数加一算；围栏查 fenced_line_flags()（APPEND_RECORD_BREAK=fence-toggle 设着的话这里本来就该红）"),
                ("只认 records/ 下已有的 .md 没判对", lambda: problems_with_target_checks(work),
                 "查 record_target()：解开符号链接之后比 <仓根>/records/ 加分隔符的前缀（APPEND_RECORD_BREAK=records-only-unchecked 设着的话这里本来就该红）"),
                ("拒绝的几种输入没判对", lambda: problems_with_refused_inputs(work),
                 "查 plan() 与 text_problem()：拒绝时记录一个字节不动（APPEND_RECORD_BREAK=headings-allowed 或 heading-indent-unstripped 设着的话这里本来就该红）"),
                ("写之前判文本没判对", lambda: problems_with_notation_guard(work),
                 "查 notation_refusal() 与 lib_notation_check.py（APPEND_RECORD_BREAK=no-notation-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK 设着的话这里本来就该红）"),
                ("读到写之间被改过没拒", lambda: problems_with_interleaved_write(work),
                 "这个工具自己坏了，别再拿它改记录：换上之前要现算 sha256 与读的时候比；查 append_text() 的 verify_unchanged_since_read"
                 "（APPEND_RECORD_BREAK=no-recheck-before-write 设着的话这里本来就该红）"),
                ("写回的方式不对", lambda: problems_with_write_back(), "按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename"),
                ("入口没走通", lambda: problems_with_entry(work), "查 main() 与 append_record()：参数、标准输入、--root 与当前目录")):
            problems = check()
            if problems:
                print("  ✗ 自检失败：" + label)
                for problem in problems:
                    print("     " + problem)  # gate-lint:detail
                print("     → " + remedy)
                return 1
    print("  ✓ 自检：追加接在末尾、补换行、行号说对、三级标题与围栏里的二级标题照写（四个反引号的围栏里三个反引号那一行不收尾）、干跑不写；"
          "records/ 下的 .tsv、records 外与仓外的 .md、同名前缀的兄弟目录、不在的、"
          "经 .. 或符号链接爬出去的拒（退 2），子目录、大写后缀、指进 records 的链接放行；空文本、一二级标题（两个空格缩进的、四个空格缩进的假围栏之后的也拒）、"
          "账本节标题、CR、记录 CRLF 或不是 UTF-8 拒（退 8）；"
          "时间戳（6）、会话号与批次代号（7）拒且点名那一行，干跑也拒，只写日期放行；读到写之间被改过拒（4）；"
          "写回换新 inode、权限位不变、在读的进程读旧内容、符号链接、硬链接与只读文件拒绝、fsync 或改名失败不留临时文件、改名之后被 Ctrl-C 打断退 9 且已追加；"
          "入口的相对路径、标准输入、会话号与缺参数")
    return 0


def main(argv):
    parser = argparse.ArgumentParser(add_help=True, description="往 records/ 下一份已有的 .md 末尾追加一段文本，写之前判钟点、会话号与批次代号")
    parser.add_argument("record", nargs="?")
    parser.add_argument("text_file", nargs="?")
    parser.add_argument("--root")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--selftest", action="store_true")
    arguments = parser.parse_args(argv)
    if arguments.selftest:
        return selftest()
    if not arguments.record or not arguments.text_file:
        print(f"  ✗ 用法：{SELF_TOOL} <记录> <文本文件>（- 是标准输入）[--dry-run] [--root 仓根]")
        print(f"     → 怎么办：看文件头的用法一节；自检跑 python3 {SELF_TOOL} --selftest")
        return 1
    try:
        if arguments.text_file == "-":
            text = sys.stdin.read()
        else:
            with open(arguments.text_file, encoding="utf-8", newline="") as handle:
                text = handle.read()
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不到文本文件 {arguments.text_file}，或它不是 UTF-8（{error}）")
        print("     → 怎么办：检查路径（相对路径相对当前目录）；文本先写进草稿目录里的文件再传进来")
        return 1
    root = os.path.realpath(arguments.root) if arguments.root else repository_root()
    return append_record(arguments.record, text, root, arguments.dry_run)


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    设了而命令行里没有 --selftest、祖先里也没有自证（inside_selftest）的，点名变量、说它只在自证里认，交回退出码 2；值不认识的同样交 2；否则交回 None。"""
    if not BROKEN:
        return None
    if BROKEN not in KNOWN_BREAKS:
        print(f"  ✗ {BREAK_VARIABLE}={BROKEN} 不认识，这一次什么都没做", file=sys.stderr)
        print(f"     → 怎么办：弄坏开关只认 {'、'.join(KNOWN_BREAKS)}（写法在文件头）；平时不设这个变量", file=sys.stderr)
        return 2
    if "--selftest" in arguments or inside_selftest():
        return None
    print(f"  ✗ {BREAK_VARIABLE} 是弄坏开关（自证用），只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一项，跑 {BREAK_VARIABLE}=<项> python3 {SELF_TOOL} --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused_outside = settle_break_switches(sys.argv[1:])
    if refused_outside is not None:
        sys.exit(refused_outside)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
