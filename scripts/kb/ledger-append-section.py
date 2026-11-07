#!/usr/bin/env python3
# admission: always 每一次追加都是对此刻那份账本做一次写入，上一次取到的节号不替这一次作保
# run-condition: none 只读写登记表点名的那一份账本，除了 python3 之外没有环境要求
"""往拆分提案账本追加一节：现取节号、一次写入索引表末行与末尾新节，写之前复核文件没被别人改过，撞了拒。

为什么要有它：用 `insert-row.py` 先插索引行占号、再用另一条命令往末尾写节的话，两步之间别的会话也能取同一个号，
门禁 doc-process-records 的 ledger-section-numbers 格到提交时才判出重号。这一份把取号、索引行、节正文放进同一次写入，
从读账本、取号到改名换上都罩在账本锁里（见「并发」），两个会话同时用它追加，后到的等前一个换上之后读新账本、取下一个号。

用法：
  ledger-append-section.py --title 标题 --body-file 正文文件 [--judgement 新登] [--history-rows —] [--open-rows 整节] [--dry-run] [--root 仓根]
  ledger-append-section.py --title 标题 --body-file - …     # 正文从标准输入读
  ledger-append-section.py --selftest

  登记表：<仓根>/.claude/gate.d/split-proposal-ledgers.tsv 里 current 那一行点名的账本（门禁 doc-process-records 的三格读同一张表）。
  --title        节标题，写成「## <节号>、<标题>」；带日期就写在标题里，如「某某问题（YYYY-MM-DD）」。不许换行。
  --body-file    节正文，写在标题下一行空一行之后。正文里不许有一、二级标题（会拆出别的节）：去掉 0–3 个前导空格之后以 # 或 ## 加空白或行尾起头的行
                 （四个空格以上缩进的是代码，不算）；三级以下与代码围栏里的行照写，围栏按 scripts/kb/lib_markdown.py 认（0–3 个空格缩进的 ``` 或 ~~~ 起、
                 同字符且不短于开头的那一行收，没收的护到文末），取节号时扫账本也照这一种认法。
  --judgement、--history-rows、--open-rows  索引表那一行的后三格（节号之后），默认「新登」「—」「整节」；格里不许有 | 与换行。
  节号：账本里所有「## <节号>、」标题与「## 索引」表里每一行的节号（汉字数字；围栏代码块里的不算）取最大值加一，写成汉字数字（一百七十八之后是一百七十九）。
  索引表与标题对不上的（只在一边出现的节号）：打一行「!」说明，照常追加；对不上由门禁 ledger-section-numbers 判。
  写之前判新文本：钟点、时区词、时间戳与会话号，同 insert-row.py（共用 lib_notation_check.py）。

并发：真写（不是 --dry-run）时，从读账本、取号、复核到改名换上，都罩在账本锁里。锁是账本这份文件（那个 inode）上的 flock 排他锁，
可写打开（NFS 上排他锁要可写打开），等锁最多 LOCK_WAIT_SECONDS 秒（超时退 10，没写）；同一进程里的几条线程另先排进程内的一把锁（NFS 上 flock 按进程算，线程之间互不挡）。
改名换上之后路径指向新 inode：拿到锁之后核路径此刻还指着锁住的那个 inode，不是（前一个追加者刚换上）就放掉、重开重拿，账本从锁住的那个描述符读。
所以两个本工具进程同时追加，后到的等前一个换上之后读到新账本、取下一个号，两次追加都在；不会两个都读到同一版、都换上、后一个盖掉前一个。
别的写法（Edit、replace-once.py、insert-row.py 这类）不拿这把锁：读时记 sha256、换上之前再算一次，变了就拒（退 4，重跑）；
复核到改名之间还有一小段空档，空档里别的写法写进去的内容会被这一次换上盖掉，门禁 ledger-section-numbers 看不出（标题与索引仍一一对应），这一段没有兜底。
写回是同目录排他新建临时文件、fsync、照原权限位、改名换上（lib_atomic_replace.py），正在读这份账本的进程读的仍是旧 inode 的旧内容。

退出码：0 追加了（或干跑）；1 用法不对或自检没过；2 登记表读不到或不合（没有 current 恰好一行、路径不在 records/ 下）或账本读不到；
3 账本里没有「## 索引」一节，或那一节里没有表格行；4 读到写之间账本被改过（没写，重跑）；5 没换上（硬链接、只读、建不了临时文件、写或改名失败）；
6 新文本带钟点、时区词或时间戳（账本是仓里的 .md）；7 新文本带会话号；8 标题或正文不合（空、标题有换行、正文有一二级标题、索引格里有 | 或换行、账本含 CRLF）；
10 等账本锁超时（别的追加者一直没放锁，没写，看是谁拿着再重跑）。2–8 与 10 都是账本没动、没留临时文件。
9 已经追加了（改名换上做完），之后被 Ctrl-C 打断（lib_atomic_replace.py 的 ReplacedThenInterrupted / ReplacedThenFailed）：别重跑（会再追加一节），先看账本末尾。

弄坏开关（只给 --selftest 证红用，平常跑设了退 2、点名只在自证里认）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（括号里是那条「✗ 自检失败：」的开头）：
  LEDGER_APPEND_SECTION_BREAK=no-recheck-before-write  换上之前不复核 sha256，读到写之间别的写法写进去的那一行被盖掉
      读到写之间被不拿锁的写法改过（「✗ 自检失败：读到写之间被别的写法改过没拒」）
  LEDGER_APPEND_SECTION_BREAK=no-notation-guard  不判新文本里的钟点、时区词、时间戳与会话号
      写之前判新文本（「✗ 自检失败：写之前判新文本没判对」）
  LEDGER_APPEND_SECTION_BREAK=fences-counted  围栏代码块里的「## <节号>、」也算一节
      正常追加（「✗ 自检失败：正常追加没成」）
  LEDGER_APPEND_SECTION_BREAK=number-from-headings-only  节号只看标题、不看索引表
      索引比标题多一行（「✗ 自检失败：索引比标题多一行时取号不对」）
  LEDGER_APPEND_SECTION_BREAK=no-ledger-lock  不拿账本锁：两个进程都读到同一版、都过复核，后换上的盖掉先换上的那一节
      两个进程同时追加（「✗ 自检失败：两个进程同时追加」）
  LEDGER_APPEND_SECTION_BREAK=lock-inode-unchecked  拿到锁之后不核路径还指不指着锁住的那个 inode：等锁的一方锁在前一个换掉的旧 inode 上、读旧账本，复核时被拒
      两个进程同时追加（「✗ 自检失败：两个进程同时追加」）
  LEDGER_APPEND_SECTION_BREAK=fence-toggle  围栏按布尔翻转认：去掉任意多前导空格之后以 ``` 或 ~~~ 起头的行都翻一次，不记字符与长度
      正常追加（「✗ 自检失败：正常追加没成」：四个反引号的围栏里那一行 ``` 被当成收尾，后面围栏里的二级标题被拒）
  LEDGER_APPEND_SECTION_BREAK=heading-indent-unstripped  正文的一、二级标题按没去前导空格的行认
      拒绝的几种输入（「✗ 自检失败：拒绝的几种输入没判对」：两个空格缩进的二级标题没拒）
  LEDGER_APPEND_SECTION_BREAK=leftover-unchecked  样本跑完不核它的进程组里还留着什么进程（processes_left_in_group 一律交空表）
      样本跑完留下的进程（「✗ 自检失败：样本跑完进程组里留着的进程没核出」）
  ATOMIC_REPLACE_BREAK=rawafterrename  共用库 lib_atomic_replace.py 的弄坏开关（写法在它文件头）：改名之后到的 Ctrl-C 原样抛，自证同样走它
      写回的方式（「✗ 自检失败：写回的方式不对」，下面几行以「[改名之后被打断]」起头）
"""
import argparse
import contextlib
import fcntl
import hashlib
import io
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time

from lib_atomic_replace import ReplaceRefused, ReplacedThenFailed, ReplacedThenInterrupted, inside_selftest, leftover_temporary_files, \
    problems_with_replacement_by_rename, replace_file_contents_by_rename
from lib_markdown import fenced_flags
from lib_notation_check import clock_refusal, copy_library_into, repository_root, session_name_refusal
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BROKEN = os.environ.get("LEDGER_APPEND_SECTION_BREAK", "")
BARRIER_VARIABLE = "LEDGER_APPEND_SECTION_BARRIER"   # 自证起两个进程时对齐「都读完了」再按次序写的目录；只在自证里认
REGISTRY = os.path.join(".claude", "gate.d", "split-proposal-ledgers.tsv")
NUMERALS = "零〇一二两三四五六七八九十百千"
HEADING = re.compile(rf"^##\s+([{NUMERALS}]+)、")      # 与门禁 doc-process-records 的 HEADING 同一写法
INDEX_ROW = re.compile(rf"^\|\s*([{NUMERALS}]+)")      # 同上的 INDEX_ROW
INDEX_HEADING = "## 索引"
DIGIT_VALUES = {"零": 0, "〇": 0, "一": 1, "二": 2, "两": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
UNIT_VALUES = {"十": 10, "百": 100, "千": 1000}
DIGIT_NAMES = "零一二三四五六七八九"
UNIT_NAMES = ("", "十", "百", "千")
BARRIER_TIMEOUT_SECONDS = 30
LEFTOVER_SAMPLE_SECONDS = 5     # 核「样本跑完组里留着进程」那一格里子进程睡多久：要长过样本退出到核 /proc 之间的空档（负载高时也够），短到自证不白等太久
LOCK_WAIT_SECONDS = 60          # 一次追加从读到换上只要零点几秒；等满这么久还没拿到锁，多半是拿着锁的进程卡住了，拒（退 10）比一直等好查
LOCK_POLL_SECONDS = 0.05        # 等锁时隔多久再试一次不阻塞的 flock（不用阻塞的 flock：阻塞着就没法按时限退出）
LEDGER_THREAD_LOCK = threading.Lock()   # 同一进程里几条线程各开各的描述符时，NFS 上的 flock 按进程算、互不挡；进程里先排这一把
# 一、二级标题：0–3 个前导空格之后 # 或 ##，再接空白或行尾（四个空格以上缩进的是代码；三级以下不算）；与 append-record.py 的 TOP_HEADING 同一种认法
TOP_HEADING = re.compile(r" {0,3}#{1,2}(?:[ \t]|$)")
TOP_HEADING_WITHOUT_INDENT = re.compile(r"#{1,2}\s")   # 弄坏开关 heading-indent-unstripped 用的认法
FENCE_OPENERS = ("```", "~~~")   # 弄坏开关 fence-toggle 用的认法


class Refused(Exception):
    """没写：code 是退出码，lines 是说明（第一行摘要，之后的行带缩进与「→」）。"""

    def __init__(self, code, *lines):
        super().__init__(lines[0])
        self.code = code
        self.lines = lines


class ChangedSinceRead(Exception):
    """写前复核：读到写之间账本的 sha256 变了。"""

    def __init__(self, digest_now):
        super().__init__(digest_now)
        self.digest_now = digest_now


def parse_numeral(text):
    """汉字数字 → 整数（一至九千九百九十九）；认不出交 None。"""
    total, digit = 0, None
    for character in text:
        if character in DIGIT_VALUES:
            digit = DIGIT_VALUES[character]
        elif character in UNIT_VALUES:
            total += (1 if digit is None else digit) * UNIT_VALUES[character]
            digit = None
        else:
            return None
    total += digit or 0
    return total or None


def numeral_text(number):
    """整数 → 汉字数字（一百零四、一百一十、二十、十四）。"""
    if not 1 <= number <= 9999:
        raise ValueError(number)
    text, pending_zero = "", False
    for position in (3, 2, 1, 0):
        digit = number // 10 ** position % 10
        if digit == 0:
            pending_zero = bool(text)
            continue
        if pending_zero:
            text += "零"
            pending_zero = False
        text += DIGIT_NAMES[digit] + UNIT_NAMES[position]
    return text[1:] if text.startswith("一十") else text


def current_ledger(root):
    """登记表里 current 那一行点名的账本 → (绝对路径, 相对仓根的路径)。表读不到、不合都抛 Refused(2)。"""
    registry = os.path.join(root, REGISTRY)
    try:
        with open(registry, encoding="utf-8") as handle:
            rows = [(number, line.rstrip("\n").split("\t")) for number, line in enumerate(handle, 1) if line.strip() and not line.startswith("#")]
    except OSError as error:
        raise Refused(2, f"读不到拆分提案账本登记表 {registry}（{error}）",
                      "     → 怎么办：登记表是门禁 doc-process-records 三格共读的那一张；先 git status 看它是被删了还是被改坏了，用 Bash 恢复它，恢复之后再追加") from error
    for number, parts in rows:
        if len(parts) != 4:
            raise Refused(2, f"{registry}:{number} 不是四段（角色、路径、sha256、理由）用制表符分隔的一行",
                          "     → 怎么办：照登记表文件头的写法改这一行（门禁 doc-process-records 的 deferred-items-registered 格同样会点名它）")
    current = [parts[1] for _, parts in rows if parts[0] == "current"]
    if len(current) != 1:
        raise Refused(2, f"{registry} 里 current 的行有 {len(current)} 行，要恰好一行",
                      "     → 怎么办：现行账本只有一份；换账本时旧的 current 改成 sealed、记下它此刻的 sha256（登记表文件头）")
    relative = current[0]
    if not (relative.startswith("records/") and relative.endswith(".md")):
        raise Refused(2, f"{registry} 里 current 的路径 {relative} 不在 records/ 下或不是 .md",
                      "     → 怎么办：登记表只收 records/ 下的 .md，改这一行")
    return os.path.join(root, relative), relative


def fenced_line_flags(lines):
    """每一行在不在代码围栏里（开合两行也算在里面）：lib_markdown.py 的 fenced_flags（记字符与长度、只认 0–3 个空格缩进、没收的护到文末）。
    与 append-record.py 的 fenced_line_flags 同一种认法；弄坏开关 fence-toggle 换成布尔翻转的认法：去掉任意多前导空格之后以 ``` 或 ~~~ 起头的行都翻一次。"""
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


def scan(lines):
    """账本逐行扫一遍 → (标题节号 [整数], 索引节号 [整数], 索引表最后一个表格行的行下标或 None, 有没有「## 索引」)。
    围栏代码块里的行（fenced_line_flags，开合两行也算）不算；fences-counted 开关下围栏里的也算。"""
    headings, indexed, last_row, in_index, has_index = [], [], None, False, False
    fenced = fenced_line_flags(lines)
    for position, line in enumerate(lines):
        if fenced[position] and BROKEN != "fences-counted":
            continue
        found = HEADING.match(line)
        if found:
            in_index = False
            if parse_numeral(found.group(1)) is not None:
                headings.append(parse_numeral(found.group(1)))
        elif line.startswith("## "):
            in_index = line.rstrip() == INDEX_HEADING
            has_index = has_index or in_index
        elif in_index and line.startswith("|"):
            last_row = position
            row = INDEX_ROW.match(line)
            if row and parse_numeral(row.group(1)) is not None:
                indexed.append(parse_numeral(row.group(1)))
    return headings, indexed, last_row, has_index


def body_problem(title, body, cells):
    """标题、正文、索引格里不合的一处 → 说明，合格交 None。"""
    if not title.strip() or "\n" in title or "\r" in title:
        return "标题是空的或带换行"
    if not body.strip():
        return "正文是空的"
    heading = TOP_HEADING_WITHOUT_INDENT if BROKEN == "heading-indent-unstripped" else TOP_HEADING   # 弄坏开关：标题不去前导空格
    lines = body.split("\n")
    for number, (line, fenced) in enumerate(zip(lines, fenced_line_flags(lines)), 1):
        if not fenced and heading.match(line):
            return f"正文第 {number} 行是一、二级标题「{line[:40]}」，会拆出别的节"
    for name, cell in cells:
        if not cell.strip() or "|" in cell or "\n" in cell or "\r" in cell:
            return f"索引格 {name} 是空的或带 | 与换行：{cell!r}"
    return None


def plan(text, title, body, cells):
    """账本此刻的文本 + 新节 → (新文本, 节号, 索引行所在行号, 节标题所在行号, 说明行列表)。不合的抛 Refused。"""
    if "\r" in text:
        raise Refused(8, "账本含 CR（CRLF 换行），这份工具只认 LF",
                      "     → 怎么办：账本一直是 LF；先查是谁把它改成了 CRLF，改回 LF 再追加")
    problem = body_problem(title, body, cells)
    if problem:
        raise Refused(8, f"{problem}，一个字节没写",
                      "     → 怎么办：标题一行、正文非空且不带一二级标题（三级以下可以）、索引格不带 | 与换行，改完重跑")
    lines = text.split("\n")
    headings, indexed, last_row, has_index = scan(lines)
    if not has_index or last_row is None:
        raise Refused(3, "账本里没有「## 索引」一节，或那一节里没有表格行，认不出索引表末行在哪",
                      f"     → 怎么办：先 grep -n '^## 索引' 看账本；索引一节标题要逐字是「{INDEX_HEADING}」，下面要有以 | 起头的表格行")
    notes = []
    if set(headings) != set(indexed):
        only_heading = sorted(set(headings) - set(indexed))
        only_index = sorted(set(indexed) - set(headings))
        notes.append("索引表与标题对不上（只有标题没有索引行：%s；只有索引行没有标题：%s），照常追加，对不上由门禁 ledger-section-numbers 判"
                     % ("、".join(numeral_text(item) for item in only_heading) or "无", "、".join(numeral_text(item) for item in only_index) or "无"))
    candidates = headings if BROKEN == "number-from-headings-only" else headings + indexed
    number = max(candidates, default=0) + 1
    try:
        name = numeral_text(number)
    except ValueError as error:
        raise Refused(8, f"节号 {number} 超出这份工具写得出的范围（一至九千九百九十九）") from error
    row = "| %s | %s |" % (name, " | ".join(cell for _, cell in cells))
    lines.insert(last_row + 1, row)
    head = "\n".join(lines).rstrip("\n")
    heading_line = head.count("\n") + 3   # 末行之后空一行再写标题
    new_text = head + "\n\n## %s、%s\n\n%s\n" % (name, title, body.strip("\n"))
    return new_text, name, last_row + 2, heading_line, notes


def notation_refusal(path, row, row_line, section, section_line, notation_root):
    """钟点与会话号：用 insert-row.py 同一份判据；放行交 None，拒绝交 Refused。"""
    if BROKEN == "no-notation-guard":
        return None
    for new_text, first_line in ((row, row_line), (section, section_line)):
        refusal = clock_refusal(path, new_text, first_line, notation_root)
        if refusal:
            return Refused(6, "\n".join(refusal), "  → 怎么办：新文本里的钟点、时区词与时间戳删掉，只写日期，再追加")
        refusal = session_name_refusal(path, new_text, first_line, notation_root)
        if refusal:
            return Refused(7, "\n".join(refusal), "  → 怎么办：新文本里的会话号换成「别的会话」这类不带号的说法，事情与日期照留，再追加")
    return None


def barrier_mark(directory, kind):
    """自证用：在对齐目录里记一笔「<kind>.<pid>」（waiting 在等锁、checked 过了复核、done 换上那一步做完）。只在自证里认。"""
    os.makedirs(directory, exist_ok=True)
    open(os.path.join(directory, "%s.%d" % (kind, os.getpid())), "w").close()


def barrier_marks_of_others(directory, kind):
    return [name for name in os.listdir(directory) if name.startswith(kind + ".") and name != "%s.%d" % (kind, os.getpid())]


def barrier_after_recheck(directory):
    """自证用，过了写前复核、改名之前调：头一个过复核的进程停在这里，等另一个进程记下「在等锁」或「换上做完」再往下（最多 BARRIER_TIMEOUT_SECONDS 秒）。
    有账本锁时另一个进程卡在锁上、记下在等锁，头一个接着换上；没有锁时另一个进程读到的是同一版、复核也过、先换上，头一个再换就把它盖掉。只在自证里认。"""
    first = not barrier_marks_of_others(directory, "checked")
    barrier_mark(directory, "checked")
    if not first:
        return
    deadline = time.monotonic() + BARRIER_TIMEOUT_SECONDS
    while not (barrier_marks_of_others(directory, "waiting") or barrier_marks_of_others(directory, "done")):
        if time.monotonic() > deadline:
            raise Refused(5, "自证的对齐目录等了 %d 秒，另一个进程既没记下在等锁、也没换上" % BARRIER_TIMEOUT_SECONDS,
                          "     → 怎么办：这是自证自己的对齐没对上；看另一个进程的输出（自检失败那一条里有）")
        time.sleep(0.02)


def open_ledger(path, writable):
    """开账本 → 文件对象。可写打开给拿锁用（NFS 上排他的 flock 要可写打开）；只读的账本可写打开不了，按「没换上」拒（退 5，与改名换上拒只读同一条）。"""
    try:
        return open(path, "r+b" if writable else "rb")
    except PermissionError as error:
        raise Refused(5, "账本 %s 对当前用户只读（%s），一个字节没写" % (path, error),
                      "     → 怎么办：就地写本来就写不进只读文件，改名换上也照样拒；确实要改就先 chmod u+w 再跑") from error
    except OSError as error:
        raise Refused(2, "读不到账本 %s（%s）" % (path, error),
                      "     → 怎么办：检查登记表 current 那一行的路径，账本是不是被改名或删了") from error


def opened_inode_is_at_path(handle, path):
    """锁住的描述符与路径此刻指的是不是同一个 inode：前一个追加者改名换上之后路径指向新 inode，锁在旧的上不算数。"""
    try:
        at_path = os.stat(path)
    except OSError:
        return False
    opened = os.fstat(handle.fileno())
    return (at_path.st_dev, at_path.st_ino) == (opened.st_dev, opened.st_ino)


def flock_until_deadline(handle, path, deadline, barrier):
    """在 handle 上拿 flock 排他锁：不阻塞地试，拿不到隔 LOCK_POLL_SECONDS 再试，到 deadline 还没拿到抛 Refused(10)。自证的对齐目录在时，头一次没拿到记一笔 waiting。"""
    announced = False
    while True:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            return
        except BlockingIOError:
            pass
        if barrier and not announced:
            barrier_mark(barrier, "waiting")
            announced = True
        if time.monotonic() > deadline:
            raise ledger_lock_timeout(path)
        time.sleep(LOCK_POLL_SECONDS)


def ledger_lock_timeout(path):
    return Refused(10, "等账本 %s 的锁等了 %d 秒还没拿到，一个字节没写" % (path, LOCK_WAIT_SECONDS),
                   "     → 怎么办：有别的追加者拿着锁没放；ps 看有没有卡住的 ledger-append-section.py，它结束之后重跑本命令")


@contextlib.contextmanager
def ledger_locked(path, dry_run, barrier):
    """真写时：拿账本锁（进程内一把、账本 inode 上 flock 一把），拿到之后核路径还指着锁住的 inode，不是就放掉、重开重拿；交出锁住的那个文件对象，退出时关掉即放锁。
    干跑不拿锁、只读打开。弄坏开关 no-ledger-lock 不拿锁，lock-inode-unchecked 拿到锁之后不核 inode。"""
    if dry_run or BROKEN == "no-ledger-lock":
        handle = open_ledger(path, writable=False)
        try:
            yield handle
        finally:
            handle.close()
        return
    deadline = time.monotonic() + LOCK_WAIT_SECONDS
    if not LEDGER_THREAD_LOCK.acquire(timeout=LOCK_WAIT_SECONDS):
        raise ledger_lock_timeout(path)
    try:
        while True:
            handle = open_ledger(path, writable=True)
            try:
                flock_until_deadline(handle, path, deadline, barrier)
            except BaseException:
                handle.close()
                raise
            if BROKEN == "lock-inode-unchecked" or opened_inode_is_at_path(handle, path):
                break
            handle.close()
        try:
            yield handle
        finally:
            handle.close()
    finally:
        LEDGER_THREAD_LOCK.release()


def append_section(path, title, body, cells, dry_run=False, notation_root=None, interleave=None):
    """账本 path 追加一节 → 退出码。说明打在标准输出。真写时从读账本到改名换上都在账本锁里（ledger_locked），账本从锁住的那个描述符读。
    interleave 只给自证用：读完、判完之后、换上之前调一次（自证在这里用不拿锁的写法往账本里另写一行，造「读到写之间被别的写法改过」）。"""
    barrier = None if dry_run else os.environ.get(BARRIER_VARIABLE)
    try:
        with ledger_locked(path, dry_run, barrier) as handle:
            return append_section_holding_ledger(handle, path, title, body, cells, dry_run, notation_root, barrier, interleave)
    except Refused as refused:
        print("  ✗ " + "\n".join(refused.lines))  # Refused 的说明第二行起是带「→」的出路
        return refused.code


def append_section_holding_ledger(handle, path, title, body, cells, dry_run, notation_root, barrier, interleave):
    """append_section 在锁里的那一段：从 handle 读账本、取号、判新文本、复核、改名换上 → 退出码。"""
    try:
        original = handle.read()
    except OSError as error:
        print("  ✗ 读不到账本 %s（%s）" % (path, error))
        print("     → 怎么办：检查登记表 current 那一行的路径，账本是不是被改名或删了")
        return 2
    before = hashlib.sha256(original).hexdigest()
    try:
        new_text, name, row_line, heading_line, notes = plan(original.decode("utf-8"), title, body, cells)
        row = new_text.split("\n")[row_line - 1]
        refusal = notation_refusal(path, row, row_line, new_text[new_text.rindex("\n## %s、" % name) + 1:], heading_line, notation_root)
        if refusal:
            raise refusal
    except Refused as refused:
        print("  ✗ " + "\n".join(refused.lines))  # Refused 的说明第二行起是带「→」的出路
        return refused.code
    for note in notes:
        print("  ! " + note)
    if dry_run:
        print("  （干跑）会取第 %s 节：索引行插在 %s:%d，节从第 %d 行起" % (name, path, row_line, heading_line))
        return 0

    def verify_unchanged_since_read():
        if BROKEN != "no-recheck-before-write":   # 弄坏开关 no-recheck-before-write：不复核
            with open(path, "rb") as current:
                digest_now = hashlib.sha256(current.read()).hexdigest()   # 每次现算，不复用读的时候那个值；排在临时文件写完之后、改名换上之前
            if digest_now != before:
                raise ChangedSinceRead(digest_now)
        if barrier:
            barrier_after_recheck(barrier)

    try:
        if interleave is not None:
            interleave()
        replaced = replace_file_contents_by_rename(path, new_text, check_before_rename=verify_unchanged_since_read)
    except Refused as refused:
        print("  ✗ " + "\n".join(refused.lines))  # Refused 的说明第二行起是带「→」的出路
        return refused.code
    except ChangedSinceRead as changed:
        print("  ✗ 读到写之间 %s 被改过（sha256 %s → %s），没写" % (path, before[:12], changed.digest_now[:12]))
        print("     → 怎么办：别的写法（Edit、replace-once.py 这类不拿账本锁的）刚往账本写过，重跑本命令，会照新账本取号。")
        return 4
    except ReplaceRefused as refused:
        print("  ✗ " + str(refused))  # str(ReplaceRefused) 第二行是「→」起头的出路（lib_atomic_replace.py 里每个 ReplaceRefused 都带着）
        return 5
    except (ReplacedThenInterrupted, ReplacedThenFailed) as replaced_then_failed:
        print("  ✗ %s 已经追加了第 %s 节（改名换上做完），之后被打断（%s，原因：%r）"
              % (path, name, replaced_then_failed.__class__.__name__, replaced_then_failed.__cause__))
        print("     → 怎么办：别重跑本命令（会再追加一节）；先看账本末尾与索引表末行，第 %s 节在就算写完了" % name)
        return 9
    finally:
        if barrier:
            barrier_mark(barrier, "done")
    print("  ✓ 追加了第 %s 节到 %s：索引行在第 %d 行，节从第 %d 行起；在账本锁里取号，写前复核 sha256 %s 未变，改名换上新 inode" % (name, path, row_line, heading_line, before[:12]))
    print("  节号：" + name)
    for note in replaced:
        print("  ! " + note)
    return 0


# ───────────────────────────── 自证 ─────────────────────────────

SAMPLE_REGISTRY = ("# 样本登记表\n"
                   "current\trecords/ledger.md\t-\t样本里的现行账本，自证用\n"
                   "sealed\trecords/old.md\t" + "0" * 64 + "\t样本里的封存账本，自证用\n")


def sample_ledger(index_rows, sections, extra=""):
    index = "".join("| %s | 新登 | — | 整节 |\n" % item for item in index_rows)
    body = "".join("## %s、标题%s\n\n正文\n\n" % (item, item) for item in sections)
    return "# 样本账本\n\n## 索引\n\n| 节号 | 判定 | 历史 | 开着的 |\n|---|---|---|---|\n" + index + "\n" + body + extra


def make_root(work, name, ledger_text):
    root = os.path.join(work, name)
    os.makedirs(os.path.join(root, "records"))
    os.makedirs(os.path.join(root, ".claude", "gate.d"))
    copy_library_into(root)
    with open(os.path.join(root, REGISTRY), "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_REGISTRY)
    ledger = os.path.join(root, "records", "ledger.md")
    with open(ledger, "w", encoding="utf-8") as handle:
        handle.write(ledger_text)
    return root, ledger


def run_append(path, root, title="新标题（2026-11-02）", body="一句正文\n", cells=(("判定", "新登"), ("历史", "—"), ("开着的", "整节")), dry_run=False):
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = append_section(path, title, body, list(cells), dry_run, notation_root=root)
    return code, captured.getvalue()


def read(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def problems_with_numbers():
    problems = []
    for number in range(1, 3000):
        if parse_numeral(numeral_text(number)) != number:
            problems.append("节号 %d 写成「%s」再读回来是 %s" % (number, numeral_text(number), parse_numeral(numeral_text(number))))
            break
    for number, wanted in ((9, "九"), (10, "十"), (14, "十四"), (20, "二十"), (99, "九十九"), (100, "一百"), (104, "一百零四"), (110, "一百一十"),
                           (179, "一百七十九"), (200, "二百"), (1000, "一千"), (1001, "一千零一")):
        if numeral_text(number) != wanted:
            problems.append("节号 %d 本该写成「%s」，实际「%s」" % (number, wanted, numeral_text(number)))
    return problems


def problems_with_plain_append(work):
    """正常追加：最大节号加一、索引行在表末、节在文件末尾；围栏里的标题与封存账本不算；索引比标题多一行时按索引取号。"""
    problems = []
    fenced = "```\n## 九百九十九、围栏里的\n```\n\n"
    root, ledger = make_root(work, "plain", sample_ledger(["一", "九十九", "一百零八"], ["一", "九十九", "一百零八"], fenced))
    original = read(ledger)
    code, said = run_append(ledger, root, body="第一段\n\n### 三级标题\n第二段\n")
    wanted = original.replace("| 一百零八 | 新登 | — | 整节 |\n", "| 一百零八 | 新登 | — | 整节 |\n| 一百零九 | 新登 | — | 整节 |\n", 1).rstrip("\n") \
        + "\n\n## 一百零九、新标题（2026-11-02）\n\n第一段\n\n### 三级标题\n第二段\n"
    if code != 0 or read(ledger) != wanted or "节号：一百零九" not in said or leftover_temporary_files(os.path.dirname(ledger)) != []:
        problems.append("取号：围栏里的「## 九百九十九、」不算，最大节号一百零八，本该追加一百零九（索引行紧接表末行、节在文件末尾），实际退 %d：%r；文件尾部 %r"
                        % (code, said, read(ledger)[-120:]))
    code, said = run_append(ledger, root, dry_run=True)
    if code != 0 or "（干跑）会取第 一百一十 节" not in said:
        problems.append("干跑：已有一百零九，本该说会取一百一十，实际退 %d：%r" % (code, said))
    before_dry_run = read(ledger)
    run_append(ledger, root, dry_run=True)
    if read(ledger) != before_dry_run:
        problems.append("干跑写了账本")
    # 围栏按字符与长度收：四个反引号开的围栏里，三个反引号那一行不收尾，后面的「## 」仍在代码里；取下一个号时扫账本也照这样认
    nested = "````\n```\n## 九百九十八、仍在围栏里\n````\n"
    code, said = run_append(ledger, root, body=nested)
    if code != 0 or "\n## 一百一十、新标题（2026-11-02）\n\n" + nested not in read(ledger):
        problems.append("正文是四个反引号的围栏、里面有一行三个反引号再一行「## 九百九十八、」：本该照写、取一百一十（三个反引号收不了四个反引号开的围栏），"
                        "实际退 %d：%r（LEDGER_APPEND_SECTION_BREAK=fence-toggle 设着的话这一条本来就该红）" % (code, said))
    code, said = run_append(ledger, root, dry_run=True)
    if code != 0 or "（干跑）会取第 一百一十一 节" not in said:
        problems.append("账本里有四个反引号的围栏、里面有「## 九百九十八、」：取号本该不算它、说会取一百一十一，实际退 %d：%r" % (code, said))
    return problems


def problems_with_index_longer_than_headings(work):
    problems = []
    root, ledger = make_root(work, "index-longer", sample_ledger(["一", "二", "十二"], ["一", "二"]))
    code, said = run_append(ledger, root)
    if code != 0 or "## 十三、" not in read(ledger) or "| 十三 |" not in read(ledger) or "只有索引行没有标题：十二" not in said:
        problems.append("索引有十二、标题只到二：本该取十三并打一行「!」点名只有索引行的十二，实际退 %d：%r" % (code, said))
    return problems


def problems_with_refused_inputs(work):
    """账本没动的拒绝：登记表不合、账本没有索引、标题正文不合。"""
    problems = []
    root, ledger = make_root(work, "refused", sample_ledger(["一"], ["一"]))
    original = read(ledger)
    registry = os.path.join(root, REGISTRY)
    with open(registry, "a", encoding="utf-8") as handle:
        handle.write("current\trecords/old.md\t-\t再来一行现行账本，登记表不合\n")
    try:
        current_ledger(root)
        problems.append("登记表里有两行 current，本该拒绝（退出码 2）")
    except Refused as refused:
        if refused.code != 2 or "恰好一行" not in "\n".join(refused.lines):
            problems.append("两行 current：本该退出码 2 并说恰好一行，实际 %r" % (refused.lines,))
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_REGISTRY)
    path, relative = current_ledger(root)
    if path != ledger or relative != "records/ledger.md":
        problems.append("登记表取 current：本该是 records/ledger.md，实际 %r" % (relative,))
    for label, text, wanted_code, kwargs in (
            ("账本没有索引一节", "# 样本账本\n\n## 一、甲\n\n正文\n", 3, {}),
            ("索引一节没有表格行", "# 样本账本\n\n## 索引\n\n没有表\n\n## 一、甲\n\n正文\n", 3, {}),
            ("正文有二级标题", original, 8, {"body": "前\n## 偷来的节\n后\n"}),
            ("正文有两个空格缩进的二级标题", original, 8, {"body": "前\n  ## 缩进的偷来的节\n后\n"}),
            ("正文里四个空格缩进的三个反引号（是代码、不是围栏）之后的二级标题", original, 8, {"body": "    ```\n## 偷来的节\n"}),
            ("正文是一行光秃的「##」（空的二级标题）", original, 8, {"body": "前\n##\n后\n"}),
            ("正文是空的", original, 8, {"body": "\n\n"}),
            ("标题带换行", original, 8, {"title": "一行\n二行"}),
            ("索引格带竖线", original, 8, {"cells": (("判定", "新登 | 多"), ("历史", "—"), ("开着的", "整节"))}),
            ("账本是 CRLF", original.replace("\n", "\r\n"), 8, {})):
        with open(ledger, "w", encoding="utf-8", newline="") as handle:
            handle.write(text)
        before = open(ledger, "rb").read()
        code, said = run_append(ledger, root, **kwargs)
        if code != wanted_code or open(ledger, "rb").read() != before or "→ 怎么办" not in said or leftover_temporary_files(os.path.dirname(ledger)) != []:
            problems.append("%s：本该退出码 %d、说明带出路、账本一个字节不动，实际 %d：%r" % (label, wanted_code, code, said))
    # 围栏里的二级标题不算拆节
    with open(ledger, "w", encoding="utf-8") as handle:
        handle.write(original)
    code, said = run_append(ledger, root, body="前\n```\n## 围栏里的\n```\n后\n")
    if code != 0:
        problems.append("正文里围栏代码块里的「## 」行不算二级标题，本该放行，实际退 %d：%r" % (code, said))
    return problems


def problems_with_notation_guard(work):
    problems = []
    glue = "".join
    root, ledger = make_root(work, "notation", sample_ledger(["一"], ["一"]))
    original = read(ledger)
    stamp = glue(("20261014", "T153000Z"))
    session = glue(("singlefs-", "e152"))
    for label, kwargs, wanted_code, wanted_text in (
            ("正文带紧凑时间戳", {"body": glue(("运行目录 ", stamp, "\n"))}, 6, stamp),
            ("标题带会话号", {"title": glue(("会话 ", session, " 回话"))}, 7, session)):
        for dry_run in (False, True):
            code, said = run_append(ledger, root, dry_run=dry_run, **kwargs)
            if code != wanted_code or wanted_text not in said or "→ 怎么办" not in said or read(ledger) != original:
                problems.append("%s%s：本该退出码 %d、点名「%s」与出路且不写账本，实际 %d：%r" % (label, "（干跑）" if dry_run else "", wanted_code, wanted_text, code, said))
    code, said = run_append(ledger, root, body="只写日期 2026-11-02 的正文\n")
    if code != 0:
        problems.append("只写日期的正文本该放行，实际退 %d：%r" % (code, said))
    return problems


def problems_with_interleaved_write(work):
    """读完、判完之后，不拿账本锁的写法（这里是就地追加一行）往账本里写了一行：换上之前复核 sha256 发现变了，拒（退 4），别人那一行留着、这一节不写、不留临时文件。"""
    root, ledger = make_root(work, "interleaved", sample_ledger(["一"], ["一"]))
    original = read(ledger)
    other = "别的写法写的一行\n"

    def interleave():
        with open(ledger, "a", encoding="utf-8") as handle:
            handle.write(other)

    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = append_section(ledger, "新标题（2026-11-02）", "一句正文\n", [("判定", "新登"), ("历史", "—"), ("开着的", "整节")],
                              notation_root=root, interleave=interleave)
    said = captured.getvalue()
    if code != 4 or read(ledger) != original + other or "读到写之间" not in said or leftover_temporary_files(os.path.dirname(ledger)) != []:
        return ["本该退 4、说「读到写之间」、账本是原文加别的写法写的那一行、不留临时文件，实际退 %d：%r；账本尾部 %r" % (code, said, read(ledger)[-80:])]
    return []


def finished_sample(process):
    """等一个样本进程（start_new_session=True 起，自己一个进程组）结束 → (退出码, 输出, 它的进程组里还留着的进程号)。
    超时只停自己起的这一个进程（先 terminate 再 kill；bash-command-detector.sh 文件头 ⑥：点名一个自己起的进程号，不对整组发信号）；
    结束后从 /proc 核这一组里还有没有进程，留下的交回调用方判红，这里不停它们。"""
    try:
        output, _ = process.communicate(timeout=60)
    except subprocess.TimeoutExpired:
        process.terminate()
        try:
            output, _ = process.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            output, _ = process.communicate()
        output += "\n（自证：60 秒没结束，停了样本进程 %d）" % process.pid
    return process.returncode, output, processes_left_in_group(process.pid)


def group_members(group):
    """/proc 里进程组号是 group 的进程号，升序。"""
    return sorted(int(entry) for entry in os.listdir("/proc") if entry.isdigit() and process_group_of(entry) == group)


def processes_left_in_group(group):
    """样本跑完之后它的进程组里还留着的进程号；弄坏开关 leftover-unchecked 一律交空表。"""
    return [] if BROKEN == "leftover-unchecked" else group_members(group)


def problems_with_sample_leftover_check():
    """finished_sample 核得出样本跑完留在它进程组里的进程：样本起一个睡 LEFTOVER_SAMPLE_SECONDS 秒的子进程、打出它的进程号就退出，
    子进程留在同一组里，要恰好点名它（子进程的标准输入输出接 /dev/null，不拿着样本的管道，communicate 不等它）。自证不停它，等它自己睡完退掉（最多再等十秒）再往下，跑完不留进程。"""
    sample = subprocess.Popen([sys.executable, "-c", "import subprocess; print(subprocess.Popen(['sleep', '%d'], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, "
                               "stderr=subprocess.DEVNULL).pid, flush=True)" % LEFTOVER_SAMPLE_SECONDS],
                              stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    code, output, leftover = finished_sample(sample)
    words = output.split()
    child = int(words[0]) if words and words[0].isdigit() else None
    problems = []
    if code != 0 or child is None or leftover != [child]:
        problems.append("样本起了一个睡 %d 秒的子进程就退出：本该退 0、点名留在进程组 %d 里的恰好是那个子进程，实际退 %r、子进程 %r、点名 %r：%r"
                        % (LEFTOVER_SAMPLE_SECONDS, sample.pid, code, child, leftover, output[-300:]))
    deadline = time.monotonic() + LEFTOVER_SAMPLE_SECONDS + 10
    while group_members(sample.pid) and time.monotonic() < deadline:
        time.sleep(0.1)
    still = group_members(sample.pid)
    if still:
        problems.append("等了 %d 秒，进程组 %d 里还留着 %r（自证不停它们）" % (LEFTOVER_SAMPLE_SECONDS + 10, sample.pid, still))
    return problems


def process_group_of(pid_text):
    try:
        with open("/proc/%s/stat" % pid_text, "rb") as handle:
            return int(handle.read().rsplit(b")", 1)[1].split()[2])
    except (OSError, ValueError, IndexError):
        return None


def problems_with_concurrent_appends(work):
    """两个进程同时追加（交错同审核复现：头一个过了写前复核、停在改名之前，第二个这时开跑）：有账本锁时第二个卡在锁上，
    头一个换上、放锁之后它读新账本、取下一个号，两个都退 0，账本里二、三两节与两行索引各一份、两份正文都在。
    没有锁时第二个读到同一版、复核也过、先换上，头一个再换就把它盖掉（两个都退 0、只剩一份正文）。对齐目录的做法在 barrier_after_recheck。"""
    problems = []
    root, ledger = make_root(work, "concurrent", sample_ledger(["一"], ["一"]))
    barrier = os.path.join(work, "barrier")
    os.makedirs(barrier)
    environment = dict(os.environ, **{BARRIER_VARIABLE: barrier})
    processes = []
    for name in ("甲", "乙"):
        body_file = os.path.join(work, "body-%s.txt" % name)
        with open(body_file, "w", encoding="utf-8") as handle:
            handle.write("来自%s的正文\n" % name)
        processes.append(subprocess.Popen([sys.executable, os.path.realpath(__file__), "--root", root, "--title", "来自%s" % name, "--body-file", body_file],
                                          stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=environment,
                                          start_new_session=True))
        if name == "甲":   # 甲过了复核、停在改名之前，再起乙
            deadline = time.monotonic() + BARRIER_TIMEOUT_SECONDS
            while not any(entry.startswith("checked.") for entry in os.listdir(barrier)) and processes[0].poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
    results = [finished_sample(process) for process in processes]
    codes = [code for code, _, _ in results]
    leftover = [pid for _, _, left in results for pid in left]
    if leftover:
        problems.append("两个进程同时追加：样本跑完，它们的进程组里还留着进程 %r（自证不停它们）" % (leftover,))
    final = read(ledger)
    wanted_counts = {"\n## 二、": 1, "\n## 三、": 1, "\n| 二 |": 1, "\n| 三 |": 1, "来自甲的正文": 1, "来自乙的正文": 1}
    counts = {piece: final.count(piece) for piece in wanted_counts}
    if codes != [0, 0] or counts != wanted_counts or leftover_temporary_files(os.path.dirname(ledger)) != []:
        problems.append("两个进程同时追加：本该都退 0，账本里「## 二、」「## 三、」与索引行「| 二 |」「| 三 |」各一份、甲乙两份正文都在，"
                        "实际退出码 %r（甲、乙），各片段份数 %r：%r" % (codes, counts, [output[-300:] for _, output, _ in results]))
    return problems


def problems_with_write_back():
    original = sample_ledger(["一"], ["一"])
    expected = plan(original, "新标题（2026-11-02）", "一句正文\n", [("判定", "新登"), ("历史", "—"), ("开着的", "整节")])[0]
    return problems_with_replacement_by_rename(
        lambda target: append_section(target, "新标题（2026-11-02）", "一句正文\n", [("判定", "新登"), ("历史", "—"), ("开着的", "整节")]), original, expected,
        exit_code_when_replaced_then_interrupted=9)


def selftest():
    with tempfile.TemporaryDirectory() as work:
        for label, check, remedy in (
                ("节号写法", lambda: problems_with_numbers(), "查 parse_numeral() / numeral_text()：一百零四、一百一十、十四、二十这几种写法要往返对得上"),
                ("正常追加没成", lambda: problems_with_plain_append(work),
                 "查 plan()：最大节号加一、索引行插在表末行之后、节接在文件末尾；围栏里的标题不算，围栏查 fenced_line_flags()"
                 "（LEDGER_APPEND_SECTION_BREAK=fences-counted 或 fence-toggle 设着的话这里本来就该红）"),
                ("索引比标题多一行时取号不对", lambda: problems_with_index_longer_than_headings(work),
                 "查 plan() 的 candidates：标题与索引表的节号一起取最大值（LEDGER_APPEND_SECTION_BREAK=number-from-headings-only 设着的话这里本来就该红）"),
                ("拒绝的几种输入没判对", lambda: problems_with_refused_inputs(work),
                 "查 current_ledger()、plan()、body_problem()：拒绝时账本一个字节不动（LEDGER_APPEND_SECTION_BREAK=heading-indent-unstripped 设着的话这里本来就该红）"),
                ("写之前判新文本没判对", lambda: problems_with_notation_guard(work),
                 "查 notation_refusal() 与 lib_notation_check.py（LEDGER_APPEND_SECTION_BREAK=no-notation-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK 设着的话这里本来就该红）"),
                ("读到写之间被别的写法改过没拒", lambda: problems_with_interleaved_write(work),
                 "这个工具自己坏了，别再拿它改账本：换上之前要现算 sha256 与读的时候比；查 append_section_holding_ledger() 的 verify_unchanged_since_read"
                 "（LEDGER_APPEND_SECTION_BREAK=no-recheck-before-write 设着的话这里本来就该红）"),
                ("样本跑完进程组里留着的进程没核出", lambda: problems_with_sample_leftover_check(),
                 "查 finished_sample() 与 processes_left_in_group()：样本跑完要从 /proc 核它的进程组里还有没有进程，留下的交回判红"
                 "（LEDGER_APPEND_SECTION_BREAK=leftover-unchecked 设着的话这里本来就该红）"),
                ("两个进程同时追加", lambda: problems_with_concurrent_appends(work),
                 "这个工具自己坏了，别再拿它改账本：从读账本到换上要罩在账本锁里，拿到锁要核 inode；查 ledger_locked()"
                 "（LEDGER_APPEND_SECTION_BREAK=no-ledger-lock 或 lock-inode-unchecked 设着的话这里本来就该红）"),
                ("写回的方式不对", lambda: problems_with_write_back(), "按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename")):
            problems = check()
            if problems:
                print("  ✗ 自检失败：" + label)
                for problem in problems:
                    print("     " + problem)  # gate-lint:detail
                print("     → " + remedy)
                return 1
    print("  ✓ 自检：节号汉字数字往返（一百零四、一百一十、二十）；追加时取标题与索引表里最大节号加一（围栏里的标题与索引比标题多一行都照算）、索引行紧接表末行、节接在文件末尾、干跑不写，"
          "四个反引号的围栏里三个反引号那一行不收尾（正文照写、取号不算里面的标题）；"
          "登记表有两行 current、账本没有索引或没有表格行、正文有一二级标题（两个空格缩进的、四个空格缩进的假围栏之后的、光秃的 ## 也拒）或为空、标题带换行、索引格带竖线、账本是 CRLF 都拒绝且账本不动；"
          "新文本带紧凑时间戳（6）或会话号（7）拒绝（干跑也拒），只写日期放行；读到写之间被不拿锁的写法改过拒（4）；"
          "样本跑完留在它进程组里的子进程核得出；两个进程同时追加（头一个过了复核停在改名之前），后一个卡在账本锁上、等前一个换上之后取下一个号，两个都退 0、两节两行两份正文都在、跑完组里不留进程；"
          "写回换新 inode、权限位不变、在读的进程读旧内容、符号链接、硬链接与只读文件拒绝、fsync 或改名失败不留临时文件、改名之后被 Ctrl-C 打断退 9 且已追加")
    return 0


def main(argv, notation_root=None):
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("--title")
    parser.add_argument("--body-file")
    parser.add_argument("--judgement", default="新登")
    parser.add_argument("--history-rows", default="—")
    parser.add_argument("--open-rows", default="整节")
    parser.add_argument("--root")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--selftest", action="store_true")
    arguments = parser.parse_args(argv)
    if arguments.selftest:
        return selftest()
    if not arguments.title or not arguments.body_file:
        print("  ✗ 用法：ledger-append-section.py --title 标题 --body-file 正文文件（- 是标准输入）")
        print("     → 怎么办：看文件头的用法一节；自检跑 ledger-append-section.py --selftest。")
        return 1
    try:
        if arguments.body_file == "-":
            body = sys.stdin.read()
        else:
            with open(arguments.body_file, encoding="utf-8") as handle:
                body = handle.read()
    except OSError as error:
        print("  ✗ 读不到正文文件 %s（%s）" % (arguments.body_file, error))
        print("     → 怎么办：检查路径（相对路径相对当前工作目录）；正文先写进草稿目录里的文件再传进来")
        return 1
    root = os.path.realpath(arguments.root) if arguments.root else repository_root()
    try:
        path, _ = current_ledger(root)
    except Refused as refused:
        print("  ✗ " + "\n".join(refused.lines))  # Refused 的说明第二行起是带「→」的出路
        return refused.code
    cells = [("判定", arguments.judgement), ("历史", arguments.history_rows), ("开着的", arguments.open_rows)]
    return append_section(path, arguments.title, body, cells, arguments.dry_run, notation_root=notation_root or (root if arguments.root else None))


BREAK_VARIABLES = ("LEDGER_APPEND_SECTION_BREAK", BARRIER_VARIABLE)


def settle_break_switches(arguments):
    """弄坏开关与自证的对齐目录只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行里有 --selftest；祖先里有自证（inside_selftest，含自证起的子进程）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or "--selftest" in arguments or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关（自证用），只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 LEDGER_APPEND_SECTION_BREAK=<项> python3 scripts/kb/ledger-append-section.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused_outside = settle_break_switches(sys.argv[1:])
    if refused_outside is not None:
        sys.exit(refused_outside)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
