#!/usr/bin/env python3
# admission: always 判的是此刻仓里这批变更史文件的样子，别的会话随时在往里写，上一次跑过不替这一次作保
# run-condition: single-instance
# run-condition: command git
"""变更史搬迁：把决策与实验的变更史改成按决策 / 实验分组，把单份 kb 文件「## 历史版本」里同一天的几条收进一个日期块。

组织形态照 `.claude/rules/changelog-format.md`。三件事：

1. 决策变更史：`.claude/kb/decisions-history/<年-月>.md` 里的全部原条目，按它点名的决策（标题或正文里出现
   `D<n>（`，写法同原 lib-history-brief.py 的 `(?<![\\w-])D(\\d+)（`）抄进 `.claude/kb/decisions-history.md` 的
   `## D<n>（简称）` 节，点名几条抄几份；节按 `.claude/kb/decisions.md` 索引表的次序排，节顶上写现状行。
   按月的那几份在 --write 时删掉（只删工作区里的文件，git 暂存交给调用方）。
2. 实验变更史：`.claude/kb/experiments-history.md` 的「## 历史版本」下那份扁平列表，同一个算法，点名词 `E<n>（`，
   索引表 `.claude/kb/experiments.md`。
3. 单份 kb 文件（`.claude/kb/` 下除了上面两份、`decisions/`、`decisions-history/` 之外的 .md）：「## 历史版本」里
   同一天有两条以上、其中至少一条是 `### 日期（其…）` 融合写法的那几天，收进一个 `### 日期`、每条各占一个 `#### `；
   别的日期块一个字节不动。

每条原条目搬过去时只改这几样：标题去掉日期与旧的「（其…）」（只有日期的，决策那边用它的「> 快查·改了什么：」当标题），
「（其N）」按新规则重编；正文前后的空行归一；正文里不在围栏中的 `####` / `#####` 小标题降一级（不然会被当成条目子标题）；
从 `decisions-history/` 搬出来的正文，`](../` 去掉一层（2026-09-12 拆档时统一补上的那一层）。正文别的字一个不改。

编号（`（其N）`）：同一节同一天里，按写入次序（旧文件同一天里新的在上，所以取文件次序倒过来；已经是新形态的
`### 日期` 块里的 `#### ` 按块内次序）逐条判：子标题带点名词（与上游 history-ordinal-keys.py 同一个：已定项 N /
未定项 N / E<n>（，取第一处）的，同一个点名词出现两次以上就都编号；没有点名词的自由文字，同一天有两条以上就都编号，
只有一条但标题是空的也编号（标题不能空着）。编号写在子标题开头：`#### （其二）：摘要`。

写之前的自检（有一项红就一份都不写，退 1）：
- 内容不丢：源里每条原条目（日期、标题、正文）在输出里出现的次数恰好等于它点名的、索引表里有的决策 / 实验个数，
  出现在哪几节也对得上；输出里没有源里没有的条目；每节现状行与索引表一致、节按索引表次序、同节日期不重复且倒序。
  一条都没点名的原条目不进输出，写进 --orphans-out 给的文件（--write 时有这类条目而没给这个参数就拒绝）。
- 单份文件：没折叠的日期块、「## 历史版本」之前的部分、块与块之间的行，改前改后逐字节相同；折叠的那几天，
  每条原条目在折叠块里出现恰好一次、标题与正文对得上。

搬迁从半搬的树上接着跑：两份变更史各自判形状，已经是新形态的那一份报一行「不再搬」、只搬还没搬的；
三样（决策、实验、单份文件）都没有要搬的才拒绝。决策变更史已是新形态而按月文件还在（删之前被改过、跳过了删除），
那几份里多出的条目用 --fill-gaps --from-worktree 补进去，补完再删。

4. 补抄缺口（--fill-gaps）：第一批搬迁用 heading-and-quick 只认标题与快查两行，正文里点名的决策 / 实验没抄过去。
   从 git 的某个提交（--from-rev，默认 HEAD）取回搬迁前的原条目（按月文件，与那一版的扁平 experiments-history.md），
   或从工作区里还在的按月文件取（--from-worktree，只有决策），对每条原条目按 all 算它点名的全集，
   与现在的两份变更史比：某一节里已经有这一条（同一个日期，标题相同或正文相同）就不动，没有的插进那一节。
   插的内容取这条在文件里已有的副本（搬迁之后别的会话改过的，例如删掉钟点、换了术语，照改过的抄，与别的节那几份相同），
   一份副本都没有才按搬迁时的写法（transformed_body、display_title）从原条目生成。
   同一天的日期块已有就追加在块尾，没有就按日期倒序新开一个 `### 日期` 块
   （新块的编号用 headings_for；追加进已有块的用 headings_appended：同一组已有的加新来的够两条就给新来的编号，
   号接着这一组已用到的最大号往下取，已有的那几条一个字不改）。同一节同一天补进来的几条按原条目的写入次序排。
   文件里已有的每一行原样留着、次序不变，只加行。
   写之前的自检（有一项红就一份都不写，退 1）：每条原条目在补完的文本里出现在哪几节，恰好是它 all 点名的、
   索引表里有的、文件里有节的那几节，外加补之前就已经有的多余副本（不是这一次造的，照旧）；补完的文本去掉加进来的行
   逐字节等于补之前的文件。写完重读一遍再算缺口，报剩几份。

并发：读每份输入时记 sha256；每份输出换上之前（临时文件写完、改名之前）再算一次它依赖的全部输入，变了就跳过这一份、
打出哪份变了、读时与写前两边的 sha256、变了哪几行，不强写；按月文件删之前同样复核。一份跳过不影响别的：别的照写，
已经写成的不回退。收尾逐份列出写了的、跳过的；有跳过的退 3。写回用 lib_atomic_replace.py（同目录临时文件、改名换上），
换上之后回读比对。跳过之后重跑同一条命令就是出路：搬迁从半搬的树上接着跑，补抄缺口从现在的文件现算缺口。

用法：
  migrate-changelog-format.py [--root 仓根] [--today YYYY-MM-DD] [--mention-scope all|heading-and-quick]
                              [--out-dir 目录] [--write] [--orphans-out 文件]
  migrate-changelog-format.py --fill-gaps [--from-rev 提交 | --from-worktree] [--root 仓根] [--out-dir 目录] [--write]
  migrate-changelog-format.py --selftest

  不带 --write：只算、只自检，打汇总，不写仓里任何文件。
  --out-dir     把要写的每份输出（按仓内相对路径）、没点名的条目与要删的文件清单落进这个目录（目录要不存在或是空的），给人审。
  --write       真写：两份变更史、要折叠的单份文件、删按月文件。
  --orphans-out 没点名任何决策 / 实验的原条目写进这个文件（排他新建）；--write 时有这类条目就必须给。
  --mention-scope  点名从哪里认：all（默认，标题加整段正文）；heading-and-quick（标题加「> 快查·」几行，认不出一个索引表里有的号时退回 all）。
  --today       写进两份变更史自己「## 历史版本」那一行的日期，默认取 Asia/Tokyo 的今天。

弄坏开关 MIGRATE_CHANGELOG_BREAK=<项>（--selftest 在每一项下都必须判红）：
  content-lenient    内容自检不比正文，只比日期与标题：「输出里改坏一个字」那一格喊不出来
  copies-ignored     内容自检按集合比、不按次数比：「同一节里多抄一份」那一格喊不出来
  untouched-off      单份文件不核没折叠的块：「没折叠的块被改了一个字节」那一格喊不出来
  no-recheck         写前不复核 sha256：「读到写之间按月文件被改过」那一格照写不误
  ordinal-always     不管该不该编号都编号：样本的期望输出对不上
  write-order-file   写入次序取文件次序（不倒过来）：样本的期望输出对不上
  fold-single-dates  只有一条的日期块也折叠：样本的期望输出对不上
  abort-on-conflict  一份跳过就不再写后面的（改前那种整批收场）：「一份被并发改过，别的照写」那一格喊不出来
  fill-skip-one      补抄缺口少插第一份：补完的自检（出现在哪几节）喊不出来
  fill-touch-existing 补抄缺口顺手改了已有的一行：「只加行」那一项喊不出来

退出码：0 成功（或干跑）；1 用法不对、结构认不出、自检红、写失败；3 有输出因为输入在读写之间被改过而跳过（别的照写了）。
"""
import argparse
import collections
import dataclasses
import datetime
import difflib
import hashlib
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402
import os as preflight_os, sys as preflight_sys  # noqa: E402,E401
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

KB = '.claude/kb'
DECISIONS_HISTORY = '.claude/kb/decisions-history.md'
EXPERIMENTS_HISTORY = '.claude/kb/experiments-history.md'
DECISIONS_INDEX = '.claude/kb/decisions.md'
EXPERIMENTS_INDEX = '.claude/kb/experiments.md'
MONTH_DIRECTORY = '.claude/kb/decisions-history'
MONTH_NAME = re.compile(r'^20\d\d-\d\d\.md$')
GEN_START = '<!-- gen:history-brief:start -->'
GEN_END = '<!-- gen:history-brief:end -->'

FENCE = re.compile(r'^[ \t]*```')
HISTORY_ANCHOR = re.compile(r'^## 历史版本[ \t]*$')
UPPER_HEADING = re.compile(r'^#{1,3}[ \t]')
DATE_HEADING = re.compile(r'^### (20\d\d-\d\d-\d\d)(.*)$')
BARE_DATE_HEADING = re.compile(r'^### (20\d\d-\d\d-\d\d)$')
CHILD_HEADING = re.compile(r'^#### ')
OLD_ORDINAL = re.compile(r'^（其[^）]*）')
NEW_ORDINAL = re.compile(r'^（其[一二三四五六七八九十]+）(?:：|$)')
QUICK_WHAT = re.compile(r'^> 快查·改了什么：(.*)$')
QUICK_LINE = re.compile(r'^> 快查·')
# 与上游 .claude/singlefs-ai-sop/scripts/history-ordinal-keys.py 的 MENTION 同一个：撞号判的就是它
ORDINAL_STEM = re.compile(r'已定项\s*\d+|未定项\s*\d+|E\d+（')
MENTION = {'D': re.compile(r'(?<![\w-])D(\d+)（'), 'E': re.compile(r'(?<![\w-])E(\d+)（')}
KIND_NAME = {'D': '决策', 'E': '实验'}
BREAK = os.environ.get('MIGRATE_CHANGELOG_BREAK', '')
FREE = object()


class Refused(Exception):
    """结构认不出、参数不对：str 第二行起是以「→」开头的下一步。"""


class ChangedSinceRead(Exception):
    def __init__(self, changed):
        """changed：[(相对路径, 写前算出的 sha256，文件没了是 None)]。"""
        super().__init__(', '.join(item for item, _ in changed))
        self.changed = changed


@dataclasses.dataclass
class Entry:
    path: str
    line: int          # 标题行在源文件里的行号（从 1 起）
    date: str
    raw_heading: str
    title: str         # 去掉日期与旧（其…）之后的标题
    body: list         # 标题行之后的原样正文行
    write_key: tuple   # 同一天里按它升序就是写入次序
    fused: bool        # 旧写法 `### 日期（其…）`


# ── 读 ──

def sha256_of(path):
    try:
        with open(path, 'rb') as handle:
            return hashlib.sha256(handle.read()).hexdigest()
    except FileNotFoundError:
        return None


def title_of(suffix):
    text = suffix.strip()
    match = OLD_ORDINAL.match(text)
    if match:
        text = text[match.end():]
    return text.lstrip('：:').strip()


def date_blocks(lines):
    """（「## 历史版本」行下标或 None, [(起, 止, 日期, 日期后面的字)], [节里不是日期的上层标题行下标]）。
    块从 `### 日期` 起，到下一个不在围栏里的 `#`/`##`/`###` 标题之前为止。"""
    anchor, blocks, stray, current, fenced = None, [], [], None, False
    for index, line in enumerate(lines):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        if anchor is None:
            if HISTORY_ANCHOR.match(line):
                anchor = index
            continue
        if not UPPER_HEADING.match(line):
            continue
        if current:
            blocks.append((current[0], index, current[1], current[2]))
            current = None
        match = DATE_HEADING.match(line)
        if match:
            current = (index, match.group(1), match.group(2))
        else:
            stray.append(index)
    if current:
        blocks.append((current[0], len(lines), current[1], current[2]))
    return anchor, blocks, stray


def split_children(body):
    """新形态日期块的正文按不在围栏里的 `#### ` 切开：（第一个子标题之前的行, [(相对下标, 标题行, 正文行)]）。"""
    head, children, fenced = [], [], False
    for index, line in enumerate(body):
        if FENCE.match(line):
            fenced = not fenced
        elif not fenced and CHILD_HEADING.match(line):
            children.append((index, line, []))
            continue
        (children[-1][2] if children else head).append(line)
    return head, children


def first_nonblank(lines):
    return next((line for line in lines if line.strip()), '')


def parse_entries(path, text, allow_new_format=True):
    """一份文件「## 历史版本」下的全部原条目。结构认不出抛 Refused。"""
    lines = text.split('\n')
    anchor, blocks, stray = date_blocks(lines)
    if anchor is None:
        raise Refused(f'{path} 里找不到「## 历史版本」，条目一条都读不到\n'
                      f'     → 怎么办：条目住在这一行下面；先看这份文件是不是已经被别的会话改了形状')
    entries = []
    for start, end, date, suffix in blocks:
        body = lines[start + 1:end]
        if not suffix.strip() and CHILD_HEADING.match(first_nonblank(body)):
            if not allow_new_format:
                raise Refused(f'{path}:{start + 1} 的 `### {date}` 已经是新形态（下面挂 `#### `），这一份不该再折叠\n'
                              f'     → 怎么办：看这一天是不是别的会话刚搬过；搬过了就把这份文件从这一批里拿掉')
            head, children = split_children(body)
            if any(line.strip() for line in head):
                raise Refused(f'{path}:{start + 1} 的 `### {date}` 与第一个 `#### ` 之间有正文，认不出归哪一条\n'
                              f'     → 怎么办：手工把那几行挪进某一条，再跑')
            for position, (offset, heading, child_body) in enumerate(children):
                entries.append(Entry(path, start + 2 + offset, date, heading, title_of(heading[5:]), child_body,
                                     (-start, position), False))
            continue
        entries.append(Entry(path, start + 1, date, lines[start], title_of(suffix), body, (-start, 0),
                             suffix.startswith('（其')))
    if BREAK == 'write-order-file':
        for entry in entries:
            entry.write_key = (entry.line, 0)
    return lines, anchor, blocks, stray, entries


def split_cells(line):
    return [cell.strip().replace('\\|', '|') for cell in re.split(r'(?<!\\)\|', line.strip())[1:-1]]


def index_rows(kind, text):
    """[(编号, 标题, 现状行)]，按索引表里第一次出现的次序；写法与 doc-decisions 的 status-sync 格同一个。"""
    rows, seen = [], set()
    for line in text.split('\n'):
        match = re.match(rf'^\| {kind}(\d+)（', line)
        if not match:
            continue
        cells = split_cells(line)
        number = int(match.group(1))
        if len(cells) >= 3 and number not in seen:
            seen.add(number)
            rows.append((number, cells[0], f'**现状**：{cells[1]}。{cells[2]}'))
    return rows


# ── 变换 ──

def chinese_number(number):
    digits = '零一二三四五六七八九'
    if not 0 < number < 100:
        raise Refused(f'同一天同一组里有 {number} 条，编号写不出来\n     → 怎么办：这个工具只写到九十九，先看是不是分组分错了')
    if number < 10:
        return digits[number]
    tens, ones = divmod(number, 10)
    return ('' if tens == 1 else digits[tens]) + '十' + (digits[ones] if ones else '')


def ordinal_stem(title):
    match = ORDINAL_STEM.search(title)
    return match.group(0) if match else FREE


def headings_for(titles):
    """titles 按写入次序给；返回同样次序的 `#### ` 子标题行。"""
    stems = [ordinal_stem(title) for title in titles]
    counts = collections.Counter(stems)
    free_numbered = counts[FREE] >= 2 or any(stem is FREE and not title for title, stem in zip(titles, stems))
    seen, out = collections.Counter(), []
    for title, stem in zip(titles, stems):
        numbered = free_numbered if stem is FREE else counts[stem] >= 2
        if BREAK == 'ordinal-always':
            numbered = True
        if numbered:
            seen[stem] += 1
            ordinal = f'（其{chinese_number(seen[stem])}）'
            out.append(f'#### {ordinal}：{title}' if title else f'#### {ordinal}')
        else:
            out.append(f'#### {title}')
    return out


def display_title(entry, use_quick):
    if entry.title or not use_quick:
        return entry.title
    for line in entry.body:
        match = QUICK_WHAT.match(line)
        if match:
            return match.group(1).strip()
    return ''


def trimmed(lines):
    start, end = 0, len(lines)
    while start < end and not lines[start].strip():
        start += 1
    while end > start and not lines[end - 1].strip():
        end -= 1
    return lines[start:end]


def transformed_body(lines, strip_parent_link):
    out, fenced = [], False
    for line in lines:
        if strip_parent_link:
            line = line.replace('](../', '](')
        if FENCE.match(line):
            fenced = not fenced
        elif not fenced and re.match(r'^#{4,5}[ \t]', line):
            line = '#' + line
        elif not fenced and re.match(r'^#{6}[ \t]', line):
            raise Refused(f'正文里有六级标题「{line[:60]}」，降一级就没有 markdown 标题了\n'
                          f'     → 怎么办：手工把那一条的小标题层级收一收，再跑')
        out.append(line)
    return out


def mentioned(entry, kind, scope, present):
    regex = MENTION[kind]
    if scope == 'heading-and-quick':
        text = '\n'.join([entry.raw_heading] + [line for line in entry.body if QUICK_LINE.match(line)])
        found = {int(number) for number in regex.findall(text)}
        if found & present:
            return found
    return {int(number) for number in regex.findall('\n'.join([entry.raw_heading] + entry.body))}


# ── 两份变更史 ──

@dataclasses.dataclass
class HistoryPlan:
    kind: str
    text: str
    entries: list
    orphans: list
    unknown: collections.Counter
    sections_with_entries: int
    copies: int


def build_history(kind, entries, rows, header_lines, own_history_lines, strip_parent_link, scope):
    present = {number for number, _, _ in rows}
    by_section = {number: [] for number, _, _ in rows}
    orphans, unknown, copies = [], collections.Counter(), 0
    for entry in entries:
        numbers = mentioned(entry, kind, scope, present)
        for number in numbers - present:
            unknown[number] += 1
        hits = sorted(numbers & present)
        if not hits:
            orphans.append(entry)
            continue
        for number in hits:
            by_section[number].append(entry)
            copies += 1
    out = list(header_lines)
    for number, title, status in rows:
        out += [f'## {title}', '', status, '']
        by_date = collections.defaultdict(list)
        for entry in by_section[number]:
            by_date[entry.date].append(entry)
        for date in sorted(by_date, reverse=True):
            ordered = sorted(by_date[date], key=lambda item: item.write_key)
            headings = headings_for([display_title(entry, kind == 'D') for entry in ordered])
            out += [f'### {date}', '']
            for entry, heading in zip(ordered, headings):
                out += [heading, ''] + trimmed(transformed_body(entry.body, strip_parent_link)) + ['']
    out += own_history_lines
    return HistoryPlan(kind, '\n'.join(out), entries, orphans, unknown,
                       sum(1 for items in by_section.values() if items), copies)


def parse_output_entries(kind, text):
    """（[(节号或 None, 日期或 None, 标题, 规整后的正文, 行号)], [(节号, 现状行或 None, 行号)], 结构问题）。"""
    lines = text.split('\n')
    section, date, current, fenced = None, None, None, False
    entries, sections, problems = [], [], []
    section_heading = re.compile(rf'^## {kind}(\d+)（')

    def close():
        if current:
            heading = current[2][5:]
            match = NEW_ORDINAL.match(heading)
            title = heading[match.end():] if match else heading
            entries.append((current[0], current[1], title, '\n'.join(trimmed(current[3])), current[4]))

    for index, line in enumerate(lines):
        if FENCE.match(line):
            fenced = not fenced
        elif not fenced and line.startswith('## '):
            close()
            current, date = None, None
            match = section_heading.match(line)
            section = int(match.group(1)) if match else None
            if match:
                status = next((later for later in lines[index + 1:] if later.strip()), '')
                sections.append((section, status if status.startswith('**现状**：') else None, index + 1))
            continue
        elif not fenced and line.startswith('### '):
            close()
            current = None
            match = BARE_DATE_HEADING.match(line)
            if section is not None and not match:
                problems.append(f'第 {index + 1} 行「{line[:60]}」：节里的 `### ` 不是单独一个日期')
            date = match.group(1) if match else None
            continue
        elif not fenced and CHILD_HEADING.match(line):
            close()
            current = [section, date, line, [], index + 1]
            continue
        if current:
            current[3].append(line)
    close()
    return entries, sections, problems


def expected_body(lines, strip_parent_link):
    """自检自己的一份：按「允许的改动」从源正文推出输出里该是什么样，不调 transformed_body。"""
    out, fenced = [], False
    for line in lines:
        text = line.replace('](../', '](') if strip_parent_link else line
        if FENCE.match(text):
            fenced = not fenced
        elif not fenced and (text.startswith('#### ') or text.startswith('####\t')
                             or text.startswith('##### ') or text.startswith('#####\t')):
            text = '#' + text
        out.append(text)
    return '\n'.join(trimmed(out))


def check_history(plan, rows, strip_parent_link, scope):
    """内容不丢与结构。返回问题清单（空就是过）。"""
    kind, problems = plan.kind, []
    present = {number for number, _, _ in rows}
    expected, origin = collections.Counter(), {}
    for entry in plan.entries:
        numbers = sorted(mentioned(entry, kind, scope, present) & present)
        title = display_title(entry, kind == 'D')
        body = expected_body(entry.body, strip_parent_link)
        for number in numbers:
            key = (number, entry.date, title, body if BREAK != 'content-lenient' else '')
            expected[key] += 1
            origin.setdefault(key, entry)
    entries, sections, structure = parse_output_entries(kind, plan.text)
    problems += structure
    actual, where = collections.Counter(), {}
    for number, date, title, body, line in entries:
        if number is None:
            problems.append(f'输出第 {line} 行的条目「{title[:50]}」不在任何 `## {kind}<n>（` 节里')
            continue
        key = (number, date, title, body if BREAK != 'content-lenient' else '')
        actual[key] += 1
        where.setdefault(key, line)
    if BREAK == 'copies-ignored':
        expected = collections.Counter(set(expected))
        actual = collections.Counter(set(actual))
    for key in sorted(set(expected) | set(actual), key=lambda item: (item[0], item[1], item[2])):
        if expected[key] == actual[key]:
            continue
        entry = origin.get(key)
        source = f'{entry.path}:{entry.line}「{entry.raw_heading[:70]}」' if entry else '源里没有这一条'
        output = f'输出第 {where[key]} 行' if key in where else '输出里没有'
        problems.append(f'{kind}{key[0]} 节 {key[1]}「{key[2][:50]}」：该出现 {expected[key]} 次，实际 {actual[key]} 次；'
                        f'源 {source}；{output}')
    order = [number for number, _, _ in sections]
    if order != [number for number, _, _ in rows]:
        problems.append(f'节的次序或个数与索引表对不上：输出 {len(order)} 节，索引表 {len(rows)} 行')
    status = {number: line for number, line, _ in sections}
    for number, _, want in rows:
        if number in status and status[number] != want:
            problems.append(f'{kind}{number} 节的现状行与索引表对不上')
    dates = collections.defaultdict(list)
    for number, date, _, _, _ in entries:
        if number is not None and (not dates[number] or dates[number][-1] != date):
            dates[number].append(date)
    for number, seen in dates.items():
        if seen != sorted(set(seen), reverse=True):
            problems.append(f'{kind}{number} 节里的日期块没有按倒序排、或同一个日期开了几块')
    return problems


# ── 单份文件的折叠 ──

@dataclasses.dataclass
class FoldPlan:
    path: str
    text: str
    new_text: str
    fold_dates: list
    entries: dict      # 日期 → [Entry]


def fold_dates_of(blocks, entries):
    counts, fused = collections.Counter(), collections.Counter()
    for entry in entries:
        counts[entry.date] += 1
        fused[entry.date] += entry.fused
    if BREAK == 'fold-single-dates':
        return sorted(date for date in counts if fused[date] or counts[date] >= 2)
    return sorted(date for date in counts if counts[date] >= 2 and fused[date] >= 1)


def fold_file(path, text):
    lines, _, blocks, _, entries = parse_entries(path, text, allow_new_format=False)
    folds = set(fold_dates_of(blocks, entries))
    if not folds:
        return FoldPlan(path, text, text, [], {})
    by_date = collections.defaultdict(list)
    for entry in entries:
        if entry.date in folds:
            by_date[entry.date].append(entry)
    out, position, emitted, last_was_fold = [], 0, set(), False
    for start, end, date, _ in blocks:
        out += lines[position:start]
        position = end
        if date not in folds:
            out += lines[start:end]
            last_was_fold = False
            continue
        if date in emitted:
            continue
        emitted.add(date)
        ordered = sorted(by_date[date], key=lambda item: item.write_key)
        block = [f'### {date}', '']
        for entry, heading in zip(ordered, headings_for([entry.title for entry in ordered])):
            body = transformed_body(entry.body, False)
            while body and not body[-1].strip():
                body.pop()
            block += [heading] + body + ['']
        out += block
        last_was_fold = True
    out += lines[position:]
    if last_was_fold and position == len(lines):
        while out and out[-1] == '':
            out.pop()
        trailing = len(lines) - len(trimmed_tail(lines))
        out += [''] * trailing
    return FoldPlan(path, text, '\n'.join(out), sorted(folds), dict(by_date))


def trimmed_tail(lines):
    end = len(lines)
    while end and lines[end - 1] == '':
        end -= 1
    return lines[:end]


def check_fold(plan):
    """没折叠的部分逐字节相同；折叠的那几天每条原条目恰好一次。返回问题清单。"""
    problems = []
    folds = set(plan.fold_dates)
    source_lines, target_lines = plan.text.split('\n'), plan.new_text.split('\n')
    _, source_blocks, _ = date_blocks(source_lines)
    _, target_blocks, _ = date_blocks(target_lines)

    def skeleton(lines, blocks):
        parts, position = [], 0
        for start, end, date, _ in blocks:
            if lines[position:start]:
                parts.append(('行', position + 1, '\n'.join(lines[position:start])))
            position = end
            if date not in folds:
                parts.append((f'日期块 {date}', start + 1, '\n'.join(lines[start:end])))
        if lines[position:]:
            parts.append(('行', position + 1, '\n'.join(trimmed_tail(lines[position:])
                                                       if blocks and blocks[-1][2] in folds else lines[position:])))
        return parts

    if BREAK != 'untouched-off':
        source_parts, target_parts = skeleton(source_lines, source_blocks), skeleton(target_lines, target_blocks)
        for index in range(max(len(source_parts), len(target_parts))):
            left = source_parts[index] if index < len(source_parts) else None
            right = target_parts[index] if index < len(target_parts) else None
            if left is None or right is None or left[0] != right[0] or left[2] != right[2]:
                problems.append(f'{plan.path}：没折叠的第 {index + 1} 段对不上（改前 {left[0] + " 第 " + str(left[1]) + " 行" if left else "没有"}，'
                                f'改后 {right[0] + " 第 " + str(right[1]) + " 行" if right else "没有"}）')
                break
    for date in plan.fold_dates:
        found = [block for block in target_blocks if block[2] == date]
        if len(found) != 1 or found[0][3].strip():
            problems.append(f'{plan.path}：{date} 该折成恰好一个单独写日期的 `### {date}`，实际 {len(found)} 个')
            continue
        start, end, _, _ = found[0]
        head, children = split_children(target_lines[start + 1:end])
        if any(line.strip() for line in head):
            problems.append(f'{plan.path}：`### {date}` 与第一个 `#### ` 之间有正文')
        got = collections.Counter()
        for _, heading, body in children:
            match = NEW_ORDINAL.match(heading[5:])
            title = heading[5:][match.end():] if match else heading[5:]
            got[(title, expected_body(body, False))] += 1
        want = collections.Counter((entry.title, expected_body(entry.body, False)) for entry in plan.entries[date])
        for key in set(got) | set(want):
            if got[key] != want[key]:
                problems.append(f'{plan.path}：{date}「{key[0][:50]}」该出现 {want[key]} 次，折叠块里 {got[key]} 次')
    return problems


# ── 汇总与写 ──

@dataclasses.dataclass
class Plan:
    outputs: list      # [(相对路径, 新内容, [依赖的相对路径])]
    deletions: list    # [(相对路径, 依赖它写成的那份输出)]
    orphans: list      # [(类, Entry)]
    problems: list
    report: list


def read(root, relative, digests, texts):
    path = os.path.join(root, relative)
    with open(path, 'rb') as handle:
        raw = handle.read()
    digests[relative] = hashlib.sha256(raw).hexdigest()
    texts[relative] = raw.decode('utf-8')
    return texts[relative]


def single_file_candidates(root):
    out = []
    for directory, subdirectories, names in os.walk(os.path.join(root, KB)):
        subdirectories.sort()
        for name in sorted(names):
            relative = os.path.relpath(os.path.join(directory, name), root)
            if not name.endswith('.md') or relative in (DECISIONS_HISTORY, EXPERIMENTS_HISTORY):
                continue
            if relative.startswith(KB + '/decisions/') or relative.startswith(MONTH_DIRECTORY + '/'):
                continue
            out.append(relative)
    return out


def decisions_header():
    return ['# 决策变更史', '',
            '每条决策一节，标题 `## D<n>（简称）`，按 [decisions.md](decisions.md) 索引表的次序排；节顶上一行 `**现状**：` 取自索引表那一行的'
            '「状态」「结论（简报）」两格，下面是它改过的每一次：按日期倒序分块，一条改动占一个 `#### ` 子标题，正文写改前、改后、依据。'
            '一条改动点名了几条决策，就在几节里各有一份，内容相同。组织形态与怎么加一条照 `.claude/rules/changelog-format.md`。', '',
            '⚠️ 条目里的分项编号一律是今天的编号：2026-08-30 起每条决策的分项只有一套编号，那一次把早于那天的条目也同步改写了。'
            '某一项当时是什么状态，看记它的那一条自己的改前、改后、依据。', '']


def experiments_header(directives):
    return (['# 实验变更史', ''] + [line for directive in directives for line in (directive, '')] +
            ['[experiments.md](experiments.md) 与 `experiments/` 下各实验正文的历史。每个实验一节，标题 `## E<n>（简称）`，'
             '按 experiments.md 索引表的次序排；节顶上一行 `**现状**：` 取自索引表那一行的「状态」「结论（简报）」两格，'
             '下面按日期倒序分块，一条改动占一个 `#### ` 子标题，正文写改前、改后、依据。一条改动点名了几个实验，就在几节里各有一份，内容相同。'
             '组织形态与怎么加一条照 `.claude/rules/changelog-format.md`。', '',
             '⚠️ 条目里的分项编号一律是今天的编号（2026-08-30 那次全仓改写连早于那天的条目一起改了）；'
             '某一项当时是什么状态，看那一条自己的改前、改后、依据。', ''])


def orphan_count(kind, entries, rows, scope):
    present = {number for number, _, _ in rows}
    return sum(1 for entry in entries if not mentioned(entry, kind, scope, present) & present)


def migration_note(kind, today, count, orphaned, sources):
    name = KIND_NAME[kind]
    return (f'- {today}：条目改按{name}分组：{sources}的 {count} 条原条目，{count - orphaned} 条原样搬进各自点名的 `## {kind}<n>（` 节'
            f'（点名几{"条" if kind == "D" else "个"}就抄几份，`research/scripts/migrate-changelog-format.py` 自检过次数与正文）'
            + (f'，{orphaned} 条没点名任何{name}、没搬进来' if orphaned else '') + '。搬的时候只改了：标题去掉日期，'
            f'「（其N）」按节内同一天重新编号——别处写的「某日（其N）」是旧编号，按日期到这次改动之前的版本里找'
            f'（`git log -- {DECISIONS_HISTORY if kind == "D" else EXPERIMENTS_HISTORY}`）；正文里的 `####` 小标题降一级'
            + ('；从按月文件搬来的相对链接去掉一层 `../`；只有日期的条目用它的「快查·改了什么」当标题。' if kind == 'D' else '。'))


def worktree_month_files(root):
    directory = os.path.join(root, MONTH_DIRECTORY)
    if not os.path.isdir(directory):
        return []
    return sorted((os.path.join(MONTH_DIRECTORY, name) for name in os.listdir(directory) if MONTH_NAME.match(name)), reverse=True)


def plan_migration(root, today, scope, digests, texts):
    problems, report, outputs, deletions, orphans, done = [], [], [], [], [], []
    month_files = worktree_month_files(root)
    current = read(root, DECISIONS_HISTORY, digests, texts)
    decisions_pending = bool(month_files) and GEN_START in current and GEN_END in current
    experiments_text = read(root, EXPERIMENTS_HISTORY, digests, texts)
    experiments_pending = not re.search(r'(?m)^## E\d+（', experiments_text)
    leftover = ''
    if not decisions_pending:
        if month_files:
            leftover = (f'；{"、".join(month_files)} 还在：决策变更史已是新形态，这几份里比它多出的条目用 `--fill-gaps --from-worktree` '
                        f'补进去，补完再删这几份')
        done.append(f'{DECISIONS_HISTORY}（{"没有按月文件" if not month_files else f"里没有 {GEN_START} / {GEN_END}"}）')
    if not experiments_pending:
        done.append(f'{EXPERIMENTS_HISTORY}（已经有 `## E<n>（` 节）')
    report += [f'{item} 已是新形态，这一份不再搬' for item in done]
    if leftover:
        report.append(leftover.lstrip('；'))

    # 决策
    if decisions_pending:
        decision_part(root, today, scope, digests, texts, current, month_files, problems, report, outputs, deletions, orphans)

    # 实验
    if experiments_pending:
        experiment_part(root, today, scope, digests, texts, experiments_text, problems, report, outputs, orphans)

    # 单份文件
    changed = single_file_part(root, digests, texts, problems, report, outputs)
    if not decisions_pending and not experiments_pending and not changed:
        raise Refused(f'没有要搬的：两份变更史都已是新形态（{"；".join(done)}），单份文件也没有要折叠的日期块{leftover}\n'
                      f'     → 怎么办：搬迁已经做完了，不用再跑；正文里点名而没抄过去的缺口用 `--fill-gaps` 补')
    for kind, entry in orphans:
        report.append(f'  没点名{KIND_NAME[kind]}：{entry.path}:{entry.line}「{entry.raw_heading[:90]}」')
    return Plan(outputs, deletions, orphans, problems, report)


def decision_part(root, today, scope, digests, texts, current, month_files, problems, report, outputs, deletions, orphans):
    current_lines = current.split('\n')
    own_start = next((index for index, line in enumerate(current_lines) if HISTORY_ANCHOR.match(line)), None)
    if own_start is None:
        raise Refused(f'{DECISIONS_HISTORY} 里找不到它自己的「## 历史版本」\n     → 怎么办：先把那一节补回来，再跑')
    own_history = current_lines[own_start:]
    decision_entries = []
    for relative in month_files:
        decision_entries += parse_entries(relative, read(root, relative, digests, texts))[4]
    decision_rows = index_rows('D', read(root, DECISIONS_INDEX, digests, texts))
    note = migration_note('D', today, len(decision_entries), orphan_count('D', decision_entries, decision_rows, scope), '、'.join(f'`{path}`' for path in month_files) + ' ')
    kept = own_history[2:] if len(own_history) > 1 and not own_history[1].strip() else own_history[1:]
    own = own_history[:1] + ['', note] + kept
    decisions = build_history('D', decision_entries, decision_rows, decisions_header(), own, True, scope)
    problems += [f'决策变更史：{problem}' for problem in check_history(decisions, decision_rows, True, scope)]
    if not decisions.text.endswith('\n'.join(kept)):
        problems.append(f'决策变更史：{DECISIONS_HISTORY} 自己的「## 历史版本」原有的几行没有原样留在文末')
    outputs.append((DECISIONS_HISTORY, decisions.text, [DECISIONS_HISTORY, DECISIONS_INDEX] + month_files))
    deletions += [(relative, DECISIONS_HISTORY) for relative in month_files]
    orphans += [('D', entry) for entry in decisions.orphans]
    report.append(f'决策变更史：{len(month_files)} 份按月文件 {len(decision_entries)} 条原条目 → {decisions.sections_with_entries} 节有条目'
                  f'（索引表 {len(decision_rows)} 行，都建节），抄了 {decisions.copies} 份；没点名的 {len(decisions.orphans)} 条；'
                  f'点名了索引表里没有的号 {len(decisions.unknown)} 个{("：" + "、".join(f"D{number}×{count}" for number, count in sorted(decisions.unknown.items()))) if decisions.unknown else ""}；'
                  f'输出 {len(decisions.text.encode())} 字节')


def experiment_part(root, today, scope, digests, texts, experiments_text, problems, report, outputs, orphans):
    lines, anchor, _, stray, experiment_entries = parse_entries(EXPERIMENTS_HISTORY, experiments_text)
    if stray:
        raise Refused(f'{EXPERIMENTS_HISTORY} 的「## 历史版本」下第 {stray[0] + 1} 行有不是日期的标题，认不出它属于哪条\n'
                      f'     → 怎么办：手工看那一行，挪走或并进某一条，再跑')
    directives = [line for line in lines[:anchor] if line.startswith('<!-- doc-lint:')]
    experiment_rows = index_rows('E', read(root, EXPERIMENTS_INDEX, digests, texts))
    own = ['## 历史版本', '', migration_note('E', today, len(experiment_entries), orphan_count('E', experiment_entries, experiment_rows, scope),
                                            '「## 历史版本」下那份扁平列表'), '']
    experiments = build_history('E', experiment_entries, experiment_rows, experiments_header(directives), own, False, scope)
    problems += [f'实验变更史：{problem}' for problem in check_history(experiments, experiment_rows, False, scope)]
    outputs.append((EXPERIMENTS_HISTORY, experiments.text, [EXPERIMENTS_HISTORY, EXPERIMENTS_INDEX]))
    orphans += [('E', entry) for entry in experiments.orphans]
    report.append(f'实验变更史：{len(experiment_entries)} 条原条目 → {experiments.sections_with_entries} 节有条目'
                  f'（索引表 {len(experiment_rows)} 行，都建节），抄了 {experiments.copies} 份；没点名的 {len(experiments.orphans)} 条；'
                  f'点名了索引表里没有的号 {len(experiments.unknown)} 个{("：" + "、".join(f"E{number}×{count}" for number, count in sorted(experiments.unknown.items()))) if experiments.unknown else ""}；'
                  f'输出 {len(experiments.text.encode())} 字节')


def single_file_part(root, digests, texts, problems, report, outputs):
    """返回要折叠的文件清单（空就是没有要折叠的）。"""
    candidates = single_file_candidates(root)
    changed, fused_only = [], []
    for relative in candidates:
        text = read(root, relative, digests, texts)
        if not re.search(r'(?m)^### 20\d\d-\d\d-\d\d（其', text):
            continue
        try:
            plan = fold_file(relative, text)
        except Refused as error:
            fused_only.append(f'{relative}（认不出，没动：{str(error).splitlines()[0]}）')
            continue
        if not plan.fold_dates:
            fused_only.append(relative)
            continue
        problems += check_fold(plan)
        outputs.append((relative, plan.new_text, [relative]))
        changed.append(f'{relative}（{"、".join(f"{date}×{len(plan.entries[date])}" for date in plan.fold_dates)}）')
    report.append(f'单份文件：扫了 {len(candidates)} 份，带融合写法的 {len(changed) + len(fused_only)} 份，要折叠的 {len(changed)} 份')
    report += [f'  折叠 {item}' for item in changed]
    report += [f'  不动（融合写法只出现在只有一条的日期块）{item}' for item in fused_only]
    return changed


def orphans_text(orphans):
    out = ['# 没点名任何决策 / 实验的原条目', '',
           '`research/scripts/migrate-changelog-format.py` 搬迁时认不出它们属于哪一节，原样列在这里，去处待定。', '']
    for _, entry in orphans:
        out += [f'## {entry.path}:{entry.line}', '', entry.raw_heading] + entry.body + ['']
    return '\n'.join(out)


def describe_change(before, path):
    after = open(path, encoding='utf-8', errors='replace').read() if os.path.exists(path) else ''
    diff = list(difflib.unified_diff(before.split('\n'), after.split('\n'), lineterm='', n=0))
    added = sum(1 for line in diff if line.startswith('+') and not line.startswith('+++'))
    removed = sum(1 for line in diff if line.startswith('-') and not line.startswith('---'))
    hunks = [line for line in diff if line.startswith('@@')][:5]
    return f'+{added} / -{removed} 行，改动位置 {" ".join(hunks)}' if os.path.exists(path) else '文件被删了'


def sha_pair(relative, before, now):
    return f'{relative} 读时 sha256 {before}，写前 sha256 {now if now else "（文件没了）"}'


def write_plan(root, plan, digests, texts, before_write=None):
    """返回（写了的, 跳过的 [(路径, 变了的输入, 说明)], 删了的, 失败的）。
    一份跳过只跳那一份，别的照写；已经写成的不回退。"""
    written, skipped, deleted, failed = [], [], [], []
    for relative, text, dependencies in plan.outputs:
        if BREAK == 'abort-on-conflict' and skipped:
            break
        if text == texts.get(relative):
            continue
        if before_write:
            before_write(relative)

        def recheck(dependencies=dependencies):
            if BREAK == 'no-recheck':
                return
            now = [(item, sha256_of(os.path.join(root, item))) for item in dependencies]
            changed = [(item, digest) for item, digest in now if digest != digests[item]]
            if changed:
                raise ChangedSinceRead(changed)
        try:
            replace_file_contents_by_rename(os.path.join(root, relative), text, check_before_rename=recheck)
        except ChangedSinceRead as error:
            skipped.append((relative, [item for item, _ in error.changed],
                            '；'.join(f'{sha_pair(item, digests[item], digest)}，{describe_change(texts[item], os.path.join(root, item))}'
                                     for item, digest in error.changed)))
            continue
        except ReplaceRefused as error:
            failed.append(f'{relative}：{error}')
            continue
        if open(os.path.join(root, relative), encoding='utf-8').read() != text:
            failed.append(f'{relative}：换上之后回读与要写的不一样（别的会话紧接着又写了？）')
            continue
        written.append(relative)
    for relative, owner in plan.deletions:
        if owner not in written:
            continue
        now = sha256_of(os.path.join(root, relative))
        if BREAK != 'no-recheck' and now != digests[relative]:
            skipped.append((relative, [relative], f'删之前 {relative} 变了（{sha_pair(relative, digests[relative], now)}）：'
                                                  f'{describe_change(texts[relative], os.path.join(root, relative))}；'
                                                  f'{owner} 已经换上，里面没有这次变动，这份没删'))
            continue
        os.remove(os.path.join(root, relative))
        if os.path.exists(os.path.join(root, relative)):
            failed.append(f'{relative}：删了却还在')
            continue
        deleted.append(relative)
    directories = sorted({os.path.dirname(relative) for relative in deleted})
    for directory in directories:
        if not os.listdir(os.path.join(root, directory)):
            os.rmdir(os.path.join(root, directory))
            deleted.append(directory + '/')
    return written, skipped, deleted, failed


def run(root, today, scope, out_dir, write, orphans_out, before_write=None, quiet=False):
    say = (lambda *parts: None) if quiet else print
    digests, texts = {}, {}
    try:
        plan = plan_migration(root, today, scope, digests, texts)
    except Refused as error:
        say(f'  ✗ {error}')
        return 1
    for line in plan.report:
        say(f'  · {line}')  # gate-lint:detail
    if plan.problems:
        say(f'  ✗ 自检没过，一份都没写：{len(plan.problems)} 处')  # gate-lint:summary
        for problem in plan.problems[:60]:
            say(f'     {problem}')  # gate-lint:detail
        say('     → 怎么办：按上面点名的条目查这个脚本的变换（build_history / fold_file）；自检红时它不写任何文件。')
        return 1
    if write and plan.orphans and not orphans_out:
        say(f'  ✗ 有 {len(plan.orphans)} 条原条目没点名任何决策 / 实验，没给 --orphans-out，不写')
        say('     → 怎么办：先定这几条的去处；要照写就给 --orphans-out <仓外或 records/ 下的新文件>，它们原样落进那份文件')
        return 1
    if out_dir:
        if os.path.exists(out_dir) and os.listdir(out_dir):
            say(f'  ✗ --out-dir {out_dir} 不是空目录')
            say('     → 怎么办：换一个不存在的目录名；这个工具不覆盖别处的文件')
            return 1
        for relative, text, _ in plan.outputs:
            target = os.path.join(out_dir, relative)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, 'x', encoding='utf-8') as handle:
                handle.write(text)
        with open(os.path.join(out_dir, 'orphans.md'), 'x', encoding='utf-8') as handle:
            handle.write(orphans_text(plan.orphans))
        with open(os.path.join(out_dir, 'deletions.txt'), 'x', encoding='utf-8') as handle:
            handle.write(''.join(f'{relative}\n' for relative, _ in plan.deletions))
        say(f'  · 输出落进 {out_dir}：{len(plan.outputs)} 份，另有 orphans.md、deletions.txt')
    if not write:
        say(f'  ✓ 干跑：自检过（{len(plan.outputs)} 份输出、{len(plan.orphans)} 条没点名），没写仓里任何文件')
        return 0
    if plan.orphans:
        with open(orphans_out, 'x', encoding='utf-8') as handle:
            handle.write(orphans_text(plan.orphans))
        say(f'  · 没点名的 {len(plan.orphans)} 条写进 {orphans_out}')
    rerun = ('等那个会话写完，重跑同一条命令：已经是新形态的那几份它认出来、报「不再搬」，只搬被跳过的；'
             '跳过的是删按月文件的，那份按月文件里多出的条目用 `--fill-gaps --from-worktree` 补进决策变更史，补完再删它')
    return finish_writes(say, root, plan, digests, texts, before_write, rerun)


def finish_writes(say, root, plan, digests, texts, before_write, rerun):
    """写、逐份报写了的与跳过的；有跳过的退 3（写成的不回退），有写失败的退 1。"""
    written, skipped, deleted, failed = write_plan(root, plan, digests, texts, before_write)
    for relative in written:
        say(f'  · 写了 {relative}')  # gate-lint:detail
    for relative in deleted:
        say(f'  · 删了 {relative}（git 暂存交给调用方：git rm --cached 或 git add -A 那几条路径）')  # gate-lint:detail
    for relative, _, detail in skipped:
        say(f'  · 跳过 {relative}：{detail}')  # gate-lint:detail
    if failed:
        say(f'  ✗ {len(failed)} 份没写成；写了 {len(written)} 份、跳过 {len(skipped)} 份')  # gate-lint:summary
        for item in failed:
            say(f'     {item}')  # gate-lint:detail
        say(f'     → 怎么办：按上面的原因修（多半是权限或硬链接）；写成的那几份不回退。{rerun}')
        return 1
    if skipped:
        say(f'  ✗ 没有全部完成：写了 {len(written)} 份（{"、".join(written) or "无"}）、删了 {len(deleted)} 份；'
            f'跳过 {len(skipped)} 份（{"、".join(relative for relative, _, _ in skipped)}），它们的输入在读写之间被别的会话改过，没强写，'
            f'写成的不回退')  # gate-lint:summary
        say(f'     → 怎么办：{rerun}')
        return 3
    say(f'  ✓ 写了 {len(written)} 份、删了 {len(deleted)} 份，自检都过')
    return 0


# ── 补抄缺口（--fill-gaps）──

ANY_ORDINAL = re.compile(r'（其([一二三四五六七八九十]+)）')


def number_of_chinese(text):
    """chinese_number 的反函数；认不出的给 0。"""
    digits = '零一二三四五六七八九'
    tens, ten, ones = text.partition('十')
    if not ten:
        return digits.index(text) if len(text) == 1 and text in digits else 0
    if len(tens) > 1 or len(ones) > 1 or (tens and tens not in digits) or (ones and ones not in digits):
        return 0
    return (digits.index(tens) if tens else 1) * 10 + (digits.index(ones) if ones else 0)


def headings_appended(existing, titles):
    """往已有的日期块尾追加几条：existing 是块里已有的 `#### ` 行（一个字不改），titles 按写入次序给。
    返回（新条目的 `#### ` 行, 同组已有而没编号、新来的编了号的条数）。分组与 headings_for 同一个（ordinal_stem），
    编不编号也同一条：同组够两条就编号，自由文字的空标题也编号；号接着这一组已用到的最大号与已有条数里大的那个往下取。"""
    count, top, unnumbered = collections.Counter(), collections.Counter(), collections.Counter()
    for line in existing:
        heading = line[5:]
        match = ANY_ORDINAL.search(heading)
        rest = (heading[:match.start()] + heading[match.end():]).lstrip('：:').strip() if match else heading.strip()
        stem = ordinal_stem(rest)
        count[stem] += 1
        if match:
            top[stem] = max(top[stem], number_of_chinese(match.group(1)))
        else:
            unnumbered[stem] += 1
    stems = [ordinal_stem(title) for title in titles]
    arriving = collections.Counter(stems)
    seen = {stem: max(top[stem], count[stem]) for stem in arriving}
    out, numbered_stems = [], set()
    for title, stem in zip(titles, stems):
        numbered = count[stem] + arriving[stem] >= 2 or (stem is FREE and not title)
        if BREAK == 'ordinal-always':
            numbered = True
        if numbered:
            seen[stem] += 1
            numbered_stems.add(stem)
            ordinal = f'（其{chinese_number(seen[stem])}）'
            out.append(f'#### {ordinal}：{title}' if title else f'#### {ordinal}')
        else:
            out.append(f'#### {title}')
    return out, sum(unnumbered[stem] for stem in numbered_stems)


@dataclasses.dataclass
class Block:
    date: object       # `### 日期` 的日期；节里不是单独日期的 `### ` 是 None
    start: int         # `### ` 行的下标
    end: int           # 块后第一行的下标（下一个 `###` / `##` 标题，或文件末）
    headings: list     # 块里 `#### ` 行的原文


@dataclasses.dataclass
class Section:
    number: int
    start: int
    end: int
    blocks: list


def section_layout(kind, lines):
    """{节号: Section}；认法与 parse_output_entries 同一个（围栏里的不算标题）。同一个号有两节的只认第一节。"""
    heading = re.compile(rf'^## {kind}(\d+)（')
    sections, current, block, fenced = {}, None, None, False

    def close_block(index):
        nonlocal block
        if block:
            block.end = index
            block = None

    for index, line in enumerate(lines):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        if line.startswith('## '):
            close_block(index)
            if current:
                current.end = index
            match = heading.match(line)
            current = Section(int(match.group(1)), index, None, []) if match else None
            if current:
                sections.setdefault(current.number, current)
            continue
        if line.startswith('### '):
            close_block(index)
            if current:
                match = BARE_DATE_HEADING.match(line)
                block = Block(match.group(1) if match else None, index, None, [])
                current.blocks.append(block)
            continue
        if block and CHILD_HEADING.match(line):
            block.headings.append(line)
    close_block(len(lines))
    if current:
        current.end = len(lines)
    return sections


def match_section(candidates, outputs, used):
    """candidates：[(编号, 日期, 标题, 正文)]；outputs：这一节现有的 [(日期, 标题, 正文)]；used：outputs 里已经配过的下标（就地加）。
    先按（日期, 标题, 正文）配，再按（日期, 标题），再按（日期, 正文）；一条现有条目只配一次。
    返回 {编号: (第几轮配上的, 配上的是 outputs 里第几条)}。"""
    matched = {}
    # 后两轮只拿非空的那一项配：空标题（只有日期的条目）或空正文配不出是哪一条
    for round_number, key in enumerate((lambda date, title, body: (date, title, body),
                                        lambda date, title, body: (date, title) if title else None,
                                        lambda date, title, body: (date, body) if body else None)):
        slots = collections.defaultdict(collections.deque)
        for position, (date, title, body) in enumerate(outputs):
            if position not in used:
                slots[key(date, title, body)].append(position)
        for number, date, title, body in candidates:
            if number in matched or key(date, title, body) is None:
                continue
            queue = slots.get(key(date, title, body))
            if queue:
                position = queue.popleft()
                used.add(position)
                matched[number] = (round_number, position)
    return matched


def occurrences(kind, keyed, wanted, text, copies=None):
    """每条原条目在 text 里出现在哪几节。返回（{编号: {该在的节里配上的节号}}, {编号: 该在的节之外或同节多出的份数},
    {配法: 份数}, 节布局）。keyed：[(编号, 日期, 标题, 正文)]；wanted：{编号: 该在的节号集合}。
    给了 copies（{编号: []}）就把每条配上的现有副本的（标题, 正文）按文件次序记进去。"""
    lines = text.split('\n')
    layout = section_layout(kind, lines)
    by_section = collections.defaultdict(list)
    for number, date, title, body, _ in parse_output_entries(kind, text)[0]:
        if number is not None:
            by_section[number].append((date, title, body))
    hits, extra, rounds = collections.defaultdict(set), collections.Counter(), collections.Counter()
    for number in layout:
        outputs, used = by_section.get(number, []), set()
        dates = {date for date, _, _ in outputs}
        needed = [item for item in keyed if number in wanted[item[0]]]
        for identifier, (round_number, position) in match_section(needed, outputs, used).items():
            hits[identifier].add(number)
            rounds[round_number] += 1
            if copies is not None:
                copies[identifier].append(outputs[position][1:])
        # 该在的都配过之后，剩下的现有条目再配：配上的是不该在这一节的副本，或同一节里的第二份、第三份
        pool = needed + [item for item in keyed if number not in wanted[item[0]] and item[1] in dates]
        while True:
            again = match_section(pool, outputs, used)
            if not again:
                break
            for identifier in again:
                extra[identifier] += 1
    return hits, extra, rounds, layout


def apply_insertions(lines, insertions):
    """insertions：[(插在第几行之前, 日期, 行)]；同一处的按日期倒序。返回（新行, 加进来的行的下标集合）。"""
    at_index = collections.defaultdict(list)
    for at, date, chunk in sorted(insertions, key=lambda item: item[1], reverse=True):
        at_index[at].append(chunk)
    out, added = [], set()
    for index in range(len(lines) + 1):
        for chunk in at_index.get(index, []):
            if out and out[-1].strip():
                added.add(len(out))
                out.append('')
            for line in chunk:
                added.add(len(out))
                out.append(line)
        if index < len(lines):
            out.append(lines[index])
    return out, added


@dataclasses.dataclass
class FillPlan:
    kind: str
    text: str
    problems: list
    stats: dict


def fill_basis(kind, entries, rows, text, strip_parent_link):
    """（keyed, wanted, 点名了却没有节的 {节号: 份数}）：keyed 是每条原条目按自检口径（expected_body）的
    （编号, 日期, 标题, 正文），wanted 是每条按 all 点名的、索引表里有的、text 里有节的节号。"""
    present = {number for number, _, _ in rows}
    keyed = [(identifier, entry.date, display_title(entry, kind == 'D'), expected_body(entry.body, strip_parent_link))
             for identifier, entry in enumerate(entries)]
    layout = section_layout(kind, text.split('\n'))
    named = {identifier: mentioned(entry, kind, 'all', present) & present for identifier, entry in enumerate(entries)}
    no_section = collections.Counter(number for numbers in named.values() for number in numbers if number not in layout)
    wanted = {identifier: {number for number in numbers if number in layout} for identifier, numbers in named.items()}
    return keyed, wanted, no_section


def fill_history(kind, entries, rows, text, strip_parent_link):
    """把 entries 里每条原条目按 all 点名的、这份文件里缺的那几份补进 text。"""
    keyed, wanted, no_section = fill_basis(kind, entries, rows, text, strip_parent_link)
    lines = text.split('\n')
    layout = section_layout(kind, lines)
    copies = collections.defaultdict(list)
    hits, extra_before, rounds, _ = occurrences(kind, keyed, wanted, text, copies)
    gaps = [(number, identifier) for identifier in range(len(entries)) for number in sorted(wanted[identifier] - hits[identifier])]
    if BREAK == 'fill-skip-one' and gaps:
        gaps = gaps[1:]
    versions, divergent, from_source = {}, 0, 0
    for identifier in sorted({identifier for _, identifier in gaps}):
        # 补的内容取这条在文件里已有的副本（搬迁之后别的会话可能改过它：删钟点、换术语），与它别的节里那几份相同；
        # 一份副本都没有才用原条目搬迁时的写法。已有副本彼此不同的，取与原条目不同（改过的）里最多的那一版
        found = copies[identifier]
        if not found:
            entry = entries[identifier]
            versions[identifier] = (display_title(entry, kind == 'D'), trimmed(transformed_body(entry.body, strip_parent_link)))
            from_source += 1
            continue
        distinct = list(dict.fromkeys(found))
        if len(distinct) > 1:
            divergent += 1
            edited = [version for version in distinct if version != keyed[identifier][2:]] or distinct
            tally = collections.Counter(found)
            distinct = sorted(edited, key=lambda version: -tally[version])
        title, body = distinct[0]
        versions[identifier] = (title, body.split('\n') if body else [])
    by_block = collections.defaultdict(list)
    for number, identifier in gaps:
        by_block[(number, entries[identifier].date)].append(identifier)
    insertions, new_blocks, appended, left_unnumbered = [], 0, 0, 0
    for (number, date), identifiers in sorted(by_block.items()):
        ordered = sorted(identifiers, key=lambda identifier: entries[identifier].write_key)
        titles = [versions[identifier][0] for identifier in ordered]
        section = layout[number]
        block = next((block for block in section.blocks if block.date == date), None)
        if block:
            headings, left = headings_appended(block.headings, titles)
            chunk, at = [], block.end
            left_unnumbered += left
            appended += len(ordered)
        else:
            headings = headings_for(titles)
            chunk = [f'### {date}', '']
            at = next((block.start for block in section.blocks if block.date is not None and block.date < date), section.end)
            new_blocks += 1
        for identifier, heading in zip(ordered, headings):
            chunk += [heading, ''] + versions[identifier][1] + ['']
        insertions.append((at, date, chunk))
    new_lines, added = apply_insertions(lines, insertions)
    if BREAK == 'fill-touch-existing':
        position = next(index for index, line in enumerate(new_lines) if index not in added and line.strip())
        new_lines[position] += '。'
    new_text = '\n'.join(new_lines)
    problems = [f'{kind}{number} 在索引表里、这份文件里却没有 `## {kind}{number}（` 节：{count} 份该补的没处放'
                for number, count in sorted(no_section.items())]
    if [line for index, line in enumerate(new_lines) if index not in added] != lines:
        problems.append('补完的文本去掉加进来的行，与补之前的文件不一样：已有的内容被改了或挪了')
    problems += check_fill(kind, entries, keyed, wanted, extra_before, new_text)
    stats = {'原条目': len(entries), '该在的份数': sum(len(numbers) for numbers in wanted.values()),
             '已经在的份数': sum(len(numbers) for numbers in hits.values()), '补的原条目': len(versions),
             '补的份数': len(gaps), '其中内容取自已有副本的原条目': len(versions) - from_source,
             '取自原条目（文件里一份副本都没有）': from_source, '已有副本彼此不同': divergent,
             '新开日期块': new_blocks, '追加进已有日期块的份数': appended,
             '同组已有而没编号、新来的编了号': left_unnumbered, '补之前就有的多余副本': sum(extra_before.values()),
             '已有的按标题配上（正文与原条目不同）': rounds[1], '已有的按正文配上（标题与原条目不同）': rounds[2],
             '字节（补之前）': len(text.encode()), '字节（补之后）': len(new_text.encode())}
    return FillPlan(kind, new_text, problems, stats)


def check_fill(kind, entries, keyed, wanted, extra_before, text):
    """补完的自检：每条原条目出现在哪几节恰好是它该在的那几节，多余的份数与补之前一样。"""
    hits, extra, _, _ = occurrences(kind, keyed, wanted, text)
    problems = []
    for identifier, entry in enumerate(entries):
        if hits[identifier] == wanted[identifier] and extra[identifier] == extra_before[identifier]:
            continue
        missing = sorted(wanted[identifier] - hits[identifier])
        problems.append(f'{entry.path}:{entry.line}「{entry.raw_heading[:70]}」：该在 {len(wanted[identifier])} 节，'
                        f'补完在 {len(hits[identifier])} 节' + (f'，缺 {"、".join(f"{kind}{number}" for number in missing)}' if missing else '')
                        + (f'；多余副本 {extra[identifier]} 份，补之前 {extra_before[identifier]} 份' if extra[identifier] != extra_before[identifier] else ''))
    return problems


def git_output(root, *arguments):
    result = subprocess.run(['git', '-C', root, *arguments], capture_output=True)
    if result.returncode != 0:
        raise Refused(f'git {" ".join(arguments)} 失败：{result.stderr.decode("utf-8", "replace").strip()[:200]}\n'
                      f'     → 怎么办：--from-rev 给一个还带着按月文件的提交（git log --diff-filter=D -- {MONTH_DIRECTORY}/ 找删它们的那次，取它的父提交）')
    return result.stdout.decode('utf-8')


def fill_sources(root, rev, digests, texts):
    """{'D' / 'E': (原条目, 出处说明, 写前要复核的工作区文件)}；--from-worktree（rev 为 None）只有决策。"""
    if rev is None:
        month_files = worktree_month_files(root)
        if not month_files:
            raise Refused(f'工作区的 {MONTH_DIRECTORY}/ 下没有按月文件，--from-worktree 没有东西可补\n'
                          f'     → 怎么办：要从搬迁前的提交取，改用 --from-rev <提交>')
        entries = []
        for relative in month_files:
            entries += parse_entries(relative, read(root, relative, digests, texts))[4]
        return {'D': (entries, f'工作区 {"、".join(month_files)}', month_files)}
    commit = git_output(root, 'rev-parse', '--verify', f'{rev}^{{commit}}').strip()
    month_files = sorted((path for path in git_output(root, 'ls-tree', '--name-only', commit, MONTH_DIRECTORY + '/').split('\n')
                          if MONTH_NAME.match(os.path.basename(path))), reverse=True)
    if not month_files:
        raise Refused(f'提交 {commit[:10]} 里没有 {MONTH_DIRECTORY}/ 下的按月文件\n'
                      f'     → 怎么办：--from-rev 给一个还带着按月文件的提交（git log --diff-filter=D -- {MONTH_DIRECTORY}/ 找删它们的那次，取它的父提交）')
    entries = []
    for relative in month_files:
        entries += parse_entries(f'{commit[:10]}:{relative}', git_output(root, 'show', f'{commit}:{relative}'))[4]
    old_experiments = git_output(root, 'show', f'{commit}:{EXPERIMENTS_HISTORY}')
    if re.search(r'(?m)^## E\d+（', old_experiments):
        raise Refused(f'提交 {commit[:10]} 的 {EXPERIMENTS_HISTORY} 已经是按实验分节的新形态，不是搬迁前的扁平列表\n'
                      f'     → 怎么办：--from-rev 给搬迁之前的提交')
    lines, _, _, stray, experiment_entries = parse_entries(f'{commit[:10]}:{EXPERIMENTS_HISTORY}', old_experiments)
    if stray:
        raise Refused(f'提交 {commit[:10]} 的 {EXPERIMENTS_HISTORY} 第 {stray[0] + 1} 行有不是日期的标题，认不出它属于哪条\n'
                      f'     → 怎么办：换一个提交，或者先手工看那一行')
    return {'D': (entries, f'提交 {commit[:10]} 的 {"、".join(month_files)}', []),
            'E': (experiment_entries, f'提交 {commit[:10]} 的 {EXPERIMENTS_HISTORY}', [])}


def plan_fill(root, rev, digests, texts):
    sources = fill_sources(root, rev, digests, texts)
    outputs, problems, report, stats = [], [], [], {}
    for kind, history, index, strip_parent_link in (('D', DECISIONS_HISTORY, DECISIONS_INDEX, True),
                                                    ('E', EXPERIMENTS_HISTORY, EXPERIMENTS_INDEX, False)):
        if kind not in sources:
            continue
        entries, origin, watched = sources[kind]
        current = read(root, history, digests, texts)
        if not re.search(rf'(?m)^## {kind}\d+（', current):
            raise Refused(f'{history} 里没有 `## {kind}<n>（` 节，还不是按{KIND_NAME[kind]}分组的新形态\n'
                          f'     → 怎么办：先不带 --fill-gaps 跑一遍搬迁')
        rows = index_rows(kind, read(root, index, digests, texts))
        plan = fill_history(kind, entries, rows, current, strip_parent_link)
        problems += [f'{KIND_NAME[kind]}变更史：{problem}' for problem in plan.problems]
        stats[kind] = plan.stats
        if plan.text != current:
            outputs.append((history, plan.text, [history] + watched))
        report.append(f'{KIND_NAME[kind]}变更史（原条目取自{origin}）：' + '；'.join(f'{name} {value}' for name, value in plan.stats.items()))
    return Plan(outputs, [], [], problems, report), stats


def run_fill(root, rev, out_dir, write, before_write=None, quiet=False):
    say = (lambda *parts: None) if quiet else print
    digests, texts = {}, {}
    try:
        plan, _ = plan_fill(root, rev, digests, texts)
    except Refused as error:
        say(f'  ✗ {error}')
        return 1
    for line in plan.report:
        say(f'  · {line}')  # gate-lint:detail
    if plan.problems:
        say(f'  ✗ 补完的自检没过，一份都没写：{len(plan.problems)} 处')  # gate-lint:summary
        for problem in plan.problems[:60]:
            say(f'     {problem}')  # gate-lint:detail
        say('     → 怎么办：按上面点名的条目查 fill_history / occurrences；自检红时它不写任何文件。')
        return 1
    if out_dir:
        if os.path.exists(out_dir) and os.listdir(out_dir):
            say(f'  ✗ --out-dir {out_dir} 不是空目录')
            say('     → 怎么办：换一个不存在的目录名；这个工具不覆盖别处的文件')
            return 1
        for relative, text, _ in plan.outputs:
            target = os.path.join(out_dir, relative)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, 'x', encoding='utf-8') as handle:
                handle.write(text)
        say(f'  · 输出落进 {out_dir}：{len(plan.outputs)} 份')
    if not write:
        say(f'  ✓ 干跑：补完的自检过（{len(plan.outputs)} 份要改），没写仓里任何文件')
        return 0
    code = finish_writes(say, root, plan, digests, texts, before_write,
                         '等那个会话写完，重跑同一条命令：它从现在的文件现算缺口，已经补上的不再补，别的会话新写的照旧留着')
    try:
        _, again = plan_fill(root, rev, {}, {})
    except Refused as error:
        say(f'  ✗ 写完重读复核时认不出：{error}')
        return 1
    left = {kind: stats['补的份数'] for kind, stats in again.items()}
    if code == 0 and any(left.values()):
        say(f'  ✗ 写完重读复核：还缺 {"、".join(f"{KIND_NAME[kind]} {count} 份" for kind, count in left.items())}')  # gate-lint:summary
        say('     → 怎么办：写完到重读之间别的会话改了这两份文件；重跑同一条命令，它从现在的文件现算缺口')
        return 1
    say(f'  · 写完重读复核：还缺 {"、".join(f"{KIND_NAME[kind]} {count} 份" for kind, count in left.items())}')
    return code


# ── 自证 ──

SAMPLE_INDEX = """# 决策

| 决策 | 状态 | 结论（简报） | 正文 |
|---|---|---|---|
| D1（甲） | 已定 1 项 / 未定 1 项 | 结论一 | [正文](decisions/01.md) |
| D2（乙） | 已定 2 项 / 未定 0 项 | 结论二 | [正文](decisions/02.md) |
| D3（丙） | 未定 | 结论三 | [正文](decisions/03.md) |
"""

SAMPLE_SEPTEMBER = """# 决策变更史 · 2026-09

头

## 历史版本

### 2026-09-28

#### （其三）：D1（甲） 已定项 1：第三条

> 快查·改前：甲

- 改前：a
- 改后：b，见 [x](../prior-art.md)

### 2026-09-28（其二）：D1（甲） 已定项 1：第二条

正文二，另见 D2（乙）。

#### 小节

小节正文

### 2026-09-28：D1（甲） 未定项 2：第一条

正文一

### 2026-09-27（其二）

> 快查·改了什么：D2（乙） 无标题那条

正文四

### 2026-09-27：门禁整理

正文五，提到 D9（不在表里）。
"""

SAMPLE_AUGUST = """# 决策变更史 · 2026-08

头

## 历史版本

### 2026-08-31（其二）：D2（乙） 另一条自由文字

正文七

### 2026-08-31 D2（乙） 用户定案

正文六
"""

SAMPLE_DECISIONS_HISTORY = """# 决策变更史

旧说明

<!-- gen:history-brief:start -->

## D1（甲）

旧表

<!-- gen:history-brief:end -->

## 怎么加一条

旧步骤

## 历史版本

- 2026-09-12：旧的一条
"""

SAMPLE_EXPERIMENTS_INDEX = """# 实验

| 实验 | 状态 | 结论（简报） | 正文 |
|---|---|---|---|
| E1（一） | 已跑 | 答了 | [正文](experiments/01.md) |
| E2（二） | 未跑 | 没答 | [正文](experiments/02.md) |
"""

SAMPLE_EXPERIMENTS_HISTORY = """# 实验变更史

<!-- doc-lint:not-numbers M1 -->

旧说明

## 历史版本

### 2026-09-20：E1（一） 跑完

- 改前：x
- 改后：y；另见 E2（二）

### 2026-09-19
- E2（二） 立项

### 2026-09-18：实验编号重排

正文（没点名实验）
"""

SAMPLE_TOOLING = """# 工具

正文

## 历史版本

### 2026-09-03（其二）：E7（七） 第二条
- 二

### 2026-09-03：第一条
- 一

### 2026-09-02（其二）
- 单条融合

### 2026-09-01：甲
- 甲

### 2026-09-01：乙
- 乙
"""

SAMPLE_OWED = """# 欠账

## 历史版本

### 2026-08-27（其二）
- 二七之二

### 2026-08-26（其二）
- 二六之二

### 2026-08-27
- 二七之一

### 2026-08-26
- 二六之一
"""

EXPECTED_DECISIONS_HISTORY = "\n".join(decisions_header()) + """
## D1（甲）

**现状**：已定 1 项 / 未定 1 项。结论一

### 2026-09-28

#### D1（甲） 未定项 2：第一条

正文一

#### （其一）：D1（甲） 已定项 1：第二条

正文二，另见 D2（乙）。

##### 小节

小节正文

#### （其二）：D1（甲） 已定项 1：第三条

> 快查·改前：甲

- 改前：a
- 改后：b，见 [x](prior-art.md)

## D2（乙）

**现状**：已定 2 项 / 未定 0 项。结论二

### 2026-09-28

#### D1（甲） 已定项 1：第二条

正文二，另见 D2（乙）。

##### 小节

小节正文

### 2026-09-27

#### D2（乙） 无标题那条

> 快查·改了什么：D2（乙） 无标题那条

正文四

### 2026-08-31

#### （其一）：D2（乙） 用户定案

正文六

#### （其二）：D2（乙） 另一条自由文字

正文七

## D3（丙）

**现状**：未定。结论三

## 历史版本

""" + migration_note('D', '2026-09-28', 7, 1, '`.claude/kb/decisions-history/2026-09.md`、`.claude/kb/decisions-history/2026-08.md` ') + """
- 2026-09-12：旧的一条
"""

EXPECTED_EXPERIMENTS_HISTORY = "\n".join(experiments_header(['<!-- doc-lint:not-numbers M1 -->'])) + """
## E1（一）

**现状**：已跑。答了

### 2026-09-20

#### E1（一） 跑完

- 改前：x
- 改后：y；另见 E2（二）

## E2（二）

**现状**：未跑。没答

### 2026-09-20

#### E1（一） 跑完

- 改前：x
- 改后：y；另见 E2（二）

### 2026-09-19

#### （其一）

- E2（二） 立项

## 历史版本

""" + migration_note('E', '2026-09-28', 3, 1, '「## 历史版本」下那份扁平列表') + "\n"

EXPECTED_TOOLING = """# 工具

正文

## 历史版本

### 2026-09-03

#### 第一条
- 一

#### E7（七） 第二条
- 二

### 2026-09-02（其二）
- 单条融合

### 2026-09-01：甲
- 甲

### 2026-09-01：乙
- 乙
"""

EXPECTED_OWED = """# 欠账

## 历史版本

### 2026-08-27

#### （其一）
- 二七之一

#### （其二）
- 二七之二

### 2026-08-26

#### （其一）
- 二六之一

#### （其二）
- 二六之二
"""


def make_sample(work):
    files = {MONTH_DIRECTORY + '/2026-09.md': SAMPLE_SEPTEMBER, MONTH_DIRECTORY + '/2026-08.md': SAMPLE_AUGUST,
             DECISIONS_HISTORY: SAMPLE_DECISIONS_HISTORY, DECISIONS_INDEX: SAMPLE_INDEX,
             EXPERIMENTS_HISTORY: SAMPLE_EXPERIMENTS_HISTORY, EXPERIMENTS_INDEX: SAMPLE_EXPERIMENTS_INDEX,
             KB + '/tooling.md': SAMPLE_TOOLING, KB + '/checks-owed.md': SAMPLE_OWED}
    for relative, text in files.items():
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        with open(os.path.join(work, relative), 'w', encoding='utf-8') as handle:
            handle.write(text)


OTHER_SESSION_BLOCK = '### 2026-09-28\n\n#### D2（乙） 已定项 1：别的会话写的\n\n别的会话的正文\n\n'
D2_TOP = '## D2（乙）\n\n**现状**：已定 2 项 / 未定 0 项。结论二\n\n'
FILLED_D2 = ('别的会话的正文\n\n#### （其二）：D1（甲） 已定项 1：第二条\n\n正文二（别的会话改过），另见 D2（乙）。\n\n'
             '##### 小节\n\n小节正文\n\n### 2026-09-27')
E2_TOP = '## E2（二）\n\n**现状**：未跑。没答\n\n'
FILLED_E2 = E2_TOP + '### 2026-09-20\n\n#### E1（一） 跑完\n\n- 改前：x\n- 改后：y；另见 E2（二）\n\n'


def make_fill_sample(root):
    """样本提交进 git（搬迁前），用 heading-and-quick 搬一遍（正文里点名的 D2、E2 各丢一份），再模拟搬迁之后别的会话：
    D2 节新开一个 2026-09-28 块写了一条、D1 节里「第二条」的正文被改过一个词。返回补之前的决策变更史全文。"""
    make_sample(root)
    for arguments in (['init', '-q'], ['add', '-A'], ['commit', '-qm', '搬迁前']):
        subprocess.run(['git', '-c', 'user.name=selftest', '-c', 'user.email=selftest@example.invalid', '-c', 'core.hooksPath=/dev/null',
                        '-c', 'commit.gpgsign=false', '-c', 'init.defaultBranch=master', '-C', root, *arguments],
                       check=True, capture_output=True)
    code = run(root, '2026-09-28', 'heading-and-quick', None, True, root + '-orphans.md', quiet=True)
    path = os.path.join(root, DECISIONS_HISTORY)
    text = open(path, encoding='utf-8').read()
    text = text.replace(D2_TOP, D2_TOP + OTHER_SESSION_BLOCK, 1).replace('正文二，另见 D2（乙）。', '正文二（别的会话改过），另见 D2（乙）。', 1)
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(text)
    return code, text


def captured(function, *arguments, **options):
    import contextlib
    import io
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        code = function(*arguments, **options)
    return code, buffer.getvalue()


def selftest():
    import tempfile
    failures, cases = [], 0

    def expect(condition, what, hint):
        nonlocal cases
        cases += 1
        if not condition:
            failures.append((what, hint))

    with tempfile.TemporaryDirectory() as work:
        # 1. 样本整跑一遍：输出逐字节等于期望，按月文件删掉，没点名的落进 orphans
        root = os.path.join(work, 'green')
        make_sample(root)
        orphans_path = os.path.join(work, 'orphans-green.md')
        code = run(root, '2026-09-28', 'all', None, True, orphans_path, quiet=True)
        expect(code == 0, f'样本整跑退出码 {code}，该是 0', '单跑 run(...) 不带 quiet 看它报什么')
        for relative, want in ((DECISIONS_HISTORY, EXPECTED_DECISIONS_HISTORY), (EXPERIMENTS_HISTORY, EXPECTED_EXPERIMENTS_HISTORY),
                               (KB + '/tooling.md', EXPECTED_TOOLING), (KB + '/checks-owed.md', EXPECTED_OWED)):
            got = open(os.path.join(root, relative), encoding='utf-8').read() if os.path.exists(os.path.join(root, relative)) else ''
            first = next((index for index, (left, right) in enumerate(zip(got.split('\n'), want.split('\n'))) if left != right), None)
            expect(got == want, f'{relative} 与期望不一样（第一处不同在第 {first + 1 if first is not None else "末尾"} 行）',
                   '查编号（headings_for）、写入次序（write_key）、折叠判据（fold_dates_of）与正文变换（transformed_body）')
        expect(not os.path.exists(os.path.join(root, MONTH_DIRECTORY)), '按月文件或删空之后的目录还在', '查 write_plan 的删除那一段')
        orphan_text = open(orphans_path, encoding='utf-8').read() if os.path.exists(orphans_path) else ''
        expect('### 2026-09-27：门禁整理' in orphan_text and '### 2026-09-18：实验编号重排' in orphan_text,
               '没点名的两条没落进 --orphans-out', '查 build_history 的 orphans 与 orphans_text')

        # 2. 内容自检：输出里改坏一个字要喊；同一节多抄一份要喊
        root = os.path.join(work, 'check')
        make_sample(root)
        digests, texts = {}, {}
        plan = plan_migration(root, '2026-09-28', 'all', digests, texts)
        expect(not plan.problems, f'样本的自检本身不绿：{plan.problems[:3]}', '先修样本整跑那一格')
        entries = []
        for relative in (MONTH_DIRECTORY + '/2026-09.md', MONTH_DIRECTORY + '/2026-08.md'):
            entries += parse_entries(relative, texts[relative])[4]
        rows = index_rows('D', texts[DECISIONS_INDEX])
        good = build_history('D', entries, rows, decisions_header(), [], True, 'all')
        corrupted = dataclasses.replace(good, text=good.text.replace('正文六', '正文陆', 1))
        problems = check_history(corrupted, rows, True, 'all')
        expect(any('用户定案' in problem for problem in problems),
               '输出里改坏一个字，内容自检没点名那一条', '查 check_history 比对的键里有没有正文')
        duplicated = dataclasses.replace(good, text=good.text.replace('#### D1（甲） 未定项 2：第一条\n\n正文一\n',
                                                                      '#### D1（甲） 未定项 2：第一条\n\n正文一\n\n#### D1（甲） 未定项 2：第一条\n\n正文一\n', 1))
        problems = check_history(duplicated, rows, True, 'all')
        expect(any('该出现 1 次，实际 2 次' in problem for problem in problems),
               '同一节多抄一份，内容自检没喊', '查 check_history 是不是按次数（Counter）比，而不是按集合比')
        dropped = dataclasses.replace(good, text=good.text.replace('## D2（乙）\n\n**现状**：已定 2 项 / 未定 0 项。结论二\n\n### 2026-09-28\n\n'
                                                                   '#### D1（甲） 已定项 1：第二条\n\n正文二，另见 D2（乙）。\n\n##### 小节\n\n小节正文\n\n',
                                                                   '## D2（乙）\n\n**现状**：已定 2 项 / 未定 0 项。结论二\n\n', 1))
        problems = check_history(dropped, rows, True, 'all')
        expect(any('D2 节' in problem and '实际 0 次' in problem for problem in problems),
               '点名两条决策的条目少抄一份，内容自检没喊', '查 check_history 的期望次数是不是按点名个数算的')

        # 3. 单份文件：没折叠的块改一个字节要喊
        fold = fold_file(KB + '/tooling.md', SAMPLE_TOOLING)
        tampered = dataclasses.replace(fold, new_text=fold.new_text.replace('- 单条融合', '- 单条融合。', 1))
        expect(bool(check_fold(tampered)), '没折叠的块被改了一个字节，单份文件自检没喊', '查 check_fold 的 skeleton 比对')
        tampered = dataclasses.replace(fold, new_text=fold.new_text.replace('#### 第一条\n- 一', '#### 第一条\n- 壹', 1))
        expect(bool(check_fold(tampered)), '折叠块里一条的正文被改了，单份文件自检没喊', '查 check_fold 折叠那几天的比对')

        # 4. 并发：读完之后按月文件被别的会话改了，决策变更史不写、按月文件不删，别的照写
        root = os.path.join(work, 'race')
        make_sample(root)

        def racing(relative):
            if relative == DECISIONS_HISTORY:
                with open(os.path.join(root, MONTH_DIRECTORY, '2026-08.md'), 'a', encoding='utf-8') as handle:
                    handle.write('\n别的会话刚写的一行\n')
        code = run(root, '2026-09-28', 'all', None, True, os.path.join(work, 'orphans-race.md'), before_write=racing, quiet=True)
        expect(code == 3, f'读写之间按月文件被改过，退出码 {code}，该是 3', '查 write_plan 的 recheck 是不是在改名换上之前重算了全部依赖')
        expect(open(os.path.join(root, DECISIONS_HISTORY), encoding='utf-8').read() == SAMPLE_DECISIONS_HISTORY,
               '读写之间按月文件被改过，决策变更史照写不误', '查 replace_file_contents_by_rename 的 check_before_rename 有没有接上')
        expect(os.path.exists(os.path.join(root, MONTH_DIRECTORY, '2026-08.md')), '决策变更史没写，按月文件却删了',
               '查 write_plan 删除之前有没有看它的那份输出写没写成')
        expect(open(os.path.join(root, EXPERIMENTS_HISTORY), encoding='utf-8').read() == EXPECTED_EXPERIMENTS_HISTORY,
               '一份被跳过时别的输出也没写', '跳过只跳那一份，别的照写')

        # 5. 有没点名的条目而没给 --orphans-out：不写
        root = os.path.join(work, 'no-orphans-out')
        make_sample(root)
        code = run(root, '2026-09-28', 'all', None, True, None, quiet=True)
        expect(code == 1 and open(os.path.join(root, DECISIONS_HISTORY), encoding='utf-8').read() == SAMPLE_DECISIONS_HISTORY,
               '有没点名的条目、没给 --orphans-out，照样写了', '查 run 里 orphans 那一道拒绝')

        # 6. 已经搬过的再跑：拒绝，不写
        root = os.path.join(work, 'green')
        code = run(root, '2026-09-28', 'all', None, True, os.path.join(work, 'orphans-again.md'), quiet=True)
        expect(code == 1, f'搬过之后再跑，退出码 {code}，该是 1', '查 plan_migration 开头的形状判断')

        # 7. 一份被并发改过：那一份跳过、报两边的 sha256，别的照写；照出路重跑同一条命令，接着把跳过的那份搬完
        root = os.path.join(work, 'resume')
        make_sample(root)
        index_path = os.path.join(root, DECISIONS_INDEX)
        before_digest = sha256_of(index_path)

        def racing_index(relative):
            if relative == DECISIONS_HISTORY:
                with open(index_path, 'a', encoding='utf-8') as handle:
                    handle.write('\n别的会话刚写的一行\n')
        code, said = captured(run, root, '2026-09-28', 'all', None, True, os.path.join(work, 'orphans-resume.md'), before_write=racing_index)
        after_digest = sha256_of(index_path)
        expect(code == 3, f'一份被并发改过，退出码 {code}，该是 3', '查 finish_writes：有跳过的退 3')
        expect(open(os.path.join(root, EXPERIMENTS_HISTORY), encoding='utf-8').read() == EXPECTED_EXPERIMENTS_HISTORY,
               '一份被并发改过，别的那份没写', '查 write_plan：一份跳过只跳那一份（abort-on-conflict 就是这一格喊）')
        expect(f'跳过 {DECISIONS_HISTORY}' in said and f'读时 sha256 {before_digest}，写前 sha256 {after_digest}' in said
               and f'写了 3 份（{EXPERIMENTS_HISTORY}、' in said,
               '收尾没点名跳过的那份、没列两边的 sha256 或没列写了的', '查 finish_writes 与 sha_pair')
        code, said = captured(run, root, '2026-09-28', 'all', None, True, os.path.join(work, 'orphans-resume-2.md'))
        expect(code == 0 and open(os.path.join(root, DECISIONS_HISTORY), encoding='utf-8').read() == EXPECTED_DECISIONS_HISTORY
               and not os.path.exists(os.path.join(root, MONTH_DIRECTORY)) and f'{EXPERIMENTS_HISTORY}（已经有 `## E<n>（` 节） 已是新形态' in said,
               f'照出路重跑没把跳过的那份搬完（退出码 {code}）', '查 plan_migration：已是新形态的那一份报「不再搬」，只搬还没搬的')

        # 8. 补抄缺口：正文里点名而没抄的补进去，已有的一个字不改；内容取已有副本（别的会话改过的照改过的抄）
        root = os.path.join(work, 'fill')
        code, prefill = make_fill_sample(root)
        expect(code == 0, f'补抄缺口的样本搬迁退出码 {code}，该是 0', '先修样本整跑那一格')
        experiments_before = open(os.path.join(root, EXPERIMENTS_HISTORY), encoding='utf-8').read()
        code, said = captured(run_fill, root, 'HEAD', None, True)
        decisions_after = open(os.path.join(root, DECISIONS_HISTORY), encoding='utf-8').read()
        experiments_after = open(os.path.join(root, EXPERIMENTS_HISTORY), encoding='utf-8').read()
        expect(code == 0, f'补抄缺口退出码 {code}，该是 0：{said[-400:]}', '单跑 run_fill 看它报什么（fill-skip-one、fill-touch-existing 让这一格红）')
        expect(decisions_after == prefill.replace('别的会话的正文\n\n### 2026-09-27', FILLED_D2, 1),
               '决策变更史补完与期望不一样', '查 fill_history：追加进已有块、headings_appended 编号、内容取已有副本')
        expect(experiments_after == experiments_before.replace(E2_TOP, FILLED_E2, 1),
               '实验变更史补完与期望不一样', '查 fill_history：没有同日块时按日期倒序新开一块')
        expect('补的份数 1' in said and '同组已有而没编号、新来的编了号 1' in said and '写完重读复核：还缺 决策 0 份、实验 0 份' in said,
               '补抄缺口的统计或写完复核没报对', '查 fill_history 的 stats 与 run_fill 的复核')
        code, said = captured(run_fill, root, 'HEAD', None, False)
        expect(code == 0 and '0 份要改' in said, '补完再跑一遍还有要补的', '查 occurrences：补进去的那份要配得上')

        # 9. 补完的自检本身会红：补完的文本里拿掉补进去的一份，check_fill 要点名缺的那一节
        entries = []
        for relative in (MONTH_DIRECTORY + '/2026-09.md', MONTH_DIRECTORY + '/2026-08.md'):
            entries += parse_entries(relative, SAMPLE_SEPTEMBER if relative.endswith('09.md') else SAMPLE_AUGUST)[4]
        rows = index_rows('D', SAMPLE_INDEX)
        keyed, wanted, _ = fill_basis('D', entries, rows, prefill, True)
        extra_before = occurrences('D', keyed, wanted, prefill)[1]
        expect(not check_fill('D', entries, keyed, wanted, extra_before, decisions_after), '补完的文本本身自检不绿',
               '查 check_fill 与 occurrences')
        removed = decisions_after.replace(FILLED_D2, '别的会话的正文\n\n### 2026-09-27', 1)
        expect(any('缺 D2' in problem for problem in check_fill('D', entries, keyed, wanted, extra_before, removed)),
               '补完的文本里少一份，自检没喊', '查 check_fill：每条原条目出现在哪几节要恰好是它该在的')
        doubled = decisions_after.replace(FILLED_D2, FILLED_D2.replace('\n\n### 2026-09-27', '\n\n#### D1（甲） 已定项 1：第二条\n\n正文二，另见 D2（乙）。\n\n'
                                                                                         '##### 小节\n\n小节正文\n\n### 2026-09-27'), 1)
        expect(any('多余副本' in problem for problem in check_fill('D', entries, keyed, wanted, extra_before, doubled)),
               '同一节多补一份，自检没喊', '查 occurrences 数多余副本那一段')

        # 10. 补抄缺口时一份被并发改过：那一份跳过、报两边 sha256，另一份照写；重跑同一条命令补完，别的会话写的那行还在
        root = os.path.join(work, 'fill-race')
        make_fill_sample(root)
        history_path = os.path.join(root, DECISIONS_HISTORY)
        before_digest = sha256_of(history_path)

        def racing_history(relative):
            if relative == DECISIONS_HISTORY:
                with open(history_path, 'a', encoding='utf-8') as handle:
                    handle.write('- 2026-09-29：别的会话刚写的一行\n')
        code, said = captured(run_fill, root, 'HEAD', None, True, before_write=racing_history)
        after_digest = sha256_of(history_path)
        expect(code == 3 and 'E1（一） 跑完' in open(os.path.join(root, EXPERIMENTS_HISTORY), encoding='utf-8').read().split(E2_TOP)[1],
               f'补抄缺口时决策变更史被并发改过：退出码 {code}（该是 3），或实验变更史没照写', '查 write_plan：一份跳过只跳那一份')
        expect(f'跳过 {DECISIONS_HISTORY}：{sha_pair(DECISIONS_HISTORY, before_digest, after_digest)}' in said,
               '补抄缺口的收尾没点名跳过的那份与两边的 sha256', '查 finish_writes 与 sha_pair')
        code, said = captured(run_fill, root, 'HEAD', None, True)
        text = open(history_path, encoding='utf-8').read()
        expect(code == 0 and FILLED_D2 in text and text.endswith('- 2026-09-29：别的会话刚写的一行\n'),
               f'照出路重跑没补完，或别的会话写的那行没了（退出码 {code}）', '查 run_fill：它从现在的文件现算缺口')

        # 11. 按月文件删之前被改过（决策变更史已换上）：搬迁重跑拒绝并指向 --from-worktree，照做补进多出的那条
        root = os.path.join(work, 'green')
        os.makedirs(os.path.join(root, MONTH_DIRECTORY))
        with open(os.path.join(root, MONTH_DIRECTORY, '2026-09.md'), 'w', encoding='utf-8') as handle:
            handle.write(SAMPLE_SEPTEMBER.replace('## 历史版本\n\n', '## 历史版本\n\n### 2026-09-29：D3（丙） 别的会话刚写的\n\n正文新\n\n', 1))
        code, said = captured(run, root, '2026-09-28', 'all', None, True, os.path.join(work, 'orphans-leftover.md'))
        expect(code == 1 and '--fill-gaps --from-worktree' in said, f'决策变更史已搬而按月文件还在，搬迁重跑退出码 {code}、没指向 --from-worktree',
               '查 plan_migration 的 leftover 那一句')
        code, said = captured(run_fill, root, None, None, True)
        text = open(os.path.join(root, DECISIONS_HISTORY), encoding='utf-8').read()
        expect(code == 0 and text == EXPECTED_DECISIONS_HISTORY.replace('**现状**：未定。结论三\n\n', '**现状**：未定。结论三\n\n### 2026-09-29\n\n'
                                                                        '#### D3（丙） 别的会话刚写的\n\n正文新\n\n', 1),
               f'--fill-gaps --from-worktree 没把按月文件里多出的那条补进去（退出码 {code}）', '查 fill_sources 的工作区那一支')

    for what, hint in failures:
        print(f'  ✗ 自证失败：{what}')
        print(f'     → 怎么办：{hint}')
    if failures:
        return 1
    print(f'  ✓ 自证：{cases} 格都过（样本整跑逐字节等于期望、内容自检对改坏一个字 / 多抄 / 少抄都喊、'
          f'单份文件没折叠的块被改会喊、读写之间被改过跳过不写、没点名的条目没去处就不写、搬过再跑拒绝、'
          f'一份被并发改过只跳那一份并报两边 sha256、重跑接着搬完、补抄缺口逐字节等于期望且再跑无缺口、'
          f'补完自检对少一份 / 多一份都喊、补抄时并发改过只跳那一份且重跑补完、按月文件残留走 --from-worktree 补进）')
    return 0


def main(argv):
    parser = argparse.ArgumentParser(description='变更史搬迁，见文件头')
    parser.add_argument('--root', default=os.path.realpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', '..')))
    parser.add_argument('--today')
    parser.add_argument('--mention-scope', choices=('all', 'heading-and-quick'), default='all')
    parser.add_argument('--out-dir')
    parser.add_argument('--write', action='store_true')
    parser.add_argument('--orphans-out')
    parser.add_argument('--fill-gaps', action='store_true')
    parser.add_argument('--from-rev')
    parser.add_argument('--from-worktree', action='store_true')
    parser.add_argument('--selftest', action='store_true')
    arguments = parser.parse_args(argv)
    if arguments.selftest:
        return selftest()
    if arguments.fill_gaps:
        if arguments.from_rev and arguments.from_worktree:
            print('  ✗ --from-rev 与 --from-worktree 只能给一个')
            print('     → 怎么办：从搬迁前的提交取原条目给 --from-rev；从工作区里还在的按月文件取给 --from-worktree')
            return 1
        return run_fill(arguments.root, None if arguments.from_worktree else (arguments.from_rev or 'HEAD'),
                        arguments.out_dir, arguments.write)
    if arguments.from_rev or arguments.from_worktree:
        print('  ✗ --from-rev / --from-worktree 只跟 --fill-gaps 一起用')
        print('     → 怎么办：补抄缺口加上 --fill-gaps；搬迁本身不带这两个参数')
        return 1
    today = arguments.today or datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).strftime('%Y-%m-%d')
    if not re.match(r'^20\d\d-\d\d-\d\d$', today):
        print(f'  ✗ --today {today} 不是 YYYY-MM-DD')
        print('     → 怎么办：写成 2026-09-28 这样，或者不给，取 Asia/Tokyo 的今天')
        return 1
    return run(arguments.root, today, arguments.mention_scope, arguments.out_dir, arguments.write, arguments.orphans_out)


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
