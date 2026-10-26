#!/usr/bin/env python3
# admission: always 每一次调都是对此刻那份变更史做一次整理或核对，上一次的结论不替这一次作保
# run-condition: command git python3
"""变更史整理：一天一条、日期只在标题、不留复制，并量整理前后的信息丢失度。

管两份文件：`.claude/kb/decisions-history.md`（节是 `## D<n>（…）`）与 `.claude/kb/experiments-history.md`
（节是 `## E<n>（…）`）。机械做的只有四类，别的一个字不动：

1. 同一条改动在几个节里各写一份的，只在**家节**留全文，别的节换成一行指针「见 X 节同日条目」。
   家节取条目标题里点名的第一条决策（或实验）；标题没点名的取正文里点名的第一条；都没有的取编号最小的那个节。
2. 「> 快查·改前 / 改后」两行删掉，前提是它们带的数与编号在条目别处都找得到；找不到的原样留下并在报告里点名。
3. 「- **改前**：…逐字照抄如下…」下面的围栏换成一行 `git show <提交>:<路径>`：提交按围栏里的旧版行号逐行对到
   那一版，对上九成以上才换；对不上的围栏原样留下并在报告里点名。
4. 正文行里与所属 `### YYYY-MM-DD` 相同的日期删掉（别的日期不动）；节顶的「**现状**」行删掉（它是索引行的复制）；
   标题里的「（其N）」删掉。

同一天剩下不止一条 `####` 的，脚本不合并（合并成结论是人的活），报告里列出来。

丢失度：按节按日期块，把整理前的数（两位以上）与编号（D / E / C / I-x.y / G<n>.<m>）逐个到整理后的同一块、
它指到的家节同日块、它引的 git 版本里找；找不到的逐个列出。`loss-check` 子命令对任意两份（改前、改后）做同一件事。

用法：
  history-consolidate.py migrate <变更史文件> --out <输出文件> [--report <报告.md>] [--repo <仓根>]
  history-consolidate.py loss-check <改前文件> <改后文件> [--report <报告.md>] [--repo <仓根>]
  history-consolidate.py --selftest

退出码：0 没有丢失；3 有丢失（报告里逐个列出）；1 用法不对或自检没过；2 --selftest 时弄坏开关设了认不得的取值。

弄坏开关（只给 --selftest 证红用，migrate 与 loss-check 不认）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句冒号前的名字，
括号里是那条「✗ 自检：」的开头）。不设开关时 --selftest 把 BREAK_MODES 里每个开关各开一次跑 migrate()，核 loss-check 都红
（没红的那一项是「弄坏开关 <名>」，失败句「✗ 自检：弄坏开关 <名>：故意丢一个数，loss-check 没红」）；设了认不得的取值退 2。
  HISTORY_CONSOLIDATE_BREAK=drop-number  transform_entry() 把条目正文里第一处两位以上的数（日期与「- **改前**」行除外）删掉一个，样本里每个家节条目各丢一个
      干净迁移不该有丢失（「✗ 自检：干净迁移不该有丢失：」）、
      快查·改后没留成结论句（「✗ 自检：快查·改后没留成结论句」）
  HISTORY_CONSOLIDATE_BREAK=git-show-ignores-path  normalized_line() 比产物行时把反引号里的 `.out` `.tsv` `.log` `.json` `.csv` 产物名一律抹成同一个记号，路径与产物名不同也算同一行
      产物行换成 git show 写法又改了产物名或目录，要判丢而没判（「✗ 自检：git show 引用写法：」，逐格点名是哪个后缀、改了什么）
      不设开关时 --selftest 也把这个开关打开一次，核上面那几格都红（没红的是『弄坏开关 git-show-ignores-path』）
  HISTORY_CONSOLIDATE_BREAK=english-quotes-unprotected  引名的引号退回只认「」『』（改前的样子）：strip_same_date() 把英语 "…" 与 “…” 里的同日日期当重复日期删，引名核对也不认这两种引号
      英语引号里的同日日期被删了（「✗ 自检：英语引号里的引名：」，逐格点名是哪种引号、删成了什么）
      不设开关时 --selftest 也把这个开关打开一次，核那一格红（没红的那一项是 弄坏开关 english-quotes-unprotected）

引名：「」『』与英语的 "…"、“…” 四种引号里的一段（判法只在 QUOTE_PAIRS 一处，doc-consolidate.py 经 strip_same_date 与 missing_quoted_names 用同一份）。
引号里的日期是被引的小节名的一部分，同日也不删；丢失核对把引号里带日期的那一段当引名，要在改后找得到。

产物行（行里有 RESULT，或反引号里写着 `.out` `.tsv` `.log` `.json` `.csv` 的产物名）要在改后逐字找得到；
唯一放宽：反引号里的 `<路径>` 与 `git show <提交>:<同一路径>` 算同一处，两个方向都行（归档脚本删产物时做的改写）；
路径、产物名、同一行别的字改了照旧判丢。
"""
import argparse
import collections
import hashlib
import os
import re
import subprocess
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

DATE_RE = re.compile(r'20\d\d-\d\d-\d\d')
SECTION_RE = re.compile(r'^## ((D|E)(\d+)（.*?）)\s*$')
DAY_RE = re.compile(r'^### (20\d\d-\d\d-\d\d)(.*)$')
ENTRY_RE = re.compile(r'^#### (.*)$')
ORDINAL_RE = re.compile(r'^（其[一二三四五六七八九十百零〇\d]+）[：:]?\s*')
ID_RE = re.compile(r'(?<![A-Za-z0-9])(?:[DEC]\d+|I-\d+\.\d+|G\d+\.\d+)')
NUMBER_RE = re.compile(r'\d+(?:[.,]\d+)*')
FENCE_INTRO_RE = re.compile(r'^- \*\*改前\*\*：.*逐字照抄如下.*`([^`]+)`.*$')
QUICK_RE = re.compile(r'^> 快查·')
STATUS_LINE_RE = re.compile(r'^\*\*现状\*\*：')


class Entry:
    def __init__(self, title, body_lines):
        self.title = title
        self.body = body_lines

    def text(self):
        return '\n'.join([self.title] + self.body)

    def key(self):
        # 同一条批量条目在各节里带的「（其N）」不同，认内容不认序号
        return hashlib.sha1((ORDINAL_RE.sub('', self.title) + '\n' + '\n'.join(self.body)).encode('utf-8')).hexdigest()


class DayBlock:
    def __init__(self, date, heading_rest, lead_lines):
        self.date = date
        self.heading_rest = heading_rest
        self.lead = lead_lines
        self.entries = []
        self.pointers = []

    def text(self):
        """整理后的形态：日期标题下面是变更清单，每条旧条目一个 `- ` 项、它的正文缩进在项下；指针也是一项。"""
        lines = [f'### {self.date}{self.heading_rest}'] + self.lead
        for entry in self.entries:
            lines.append(f'- {entry.title}')
            lines += [('  ' + l) if l.strip() else '' for l in entry.body]
        if self.pointers:
            if lines and lines[-1].strip() and self.entries:
                lines.append('')
            lines += self.pointers
        return '\n'.join(lines)


class Section:
    def __init__(self, name, kind, number, header_line):
        self.name = name
        self.kind = kind
        self.number = number
        self.header = header_line
        self.status_lines = []
        self.days = []

    def day(self, date):
        for block in self.days:
            if block.date == date:
                return block
        return None


def parse_history(text):
    """文件 = 前言 + 若干 `## D<n>（` / `## E<n>（` 节 + 尾段（别的 `## ` 节，例如文件自己的「## 历史版本」，原样保留）。"""
    preamble = []
    sections = []
    trailing = []
    section = None
    block = None
    entry = None
    for line in text.split('\n'):
        if trailing:
            trailing.append(line)
            continue
        section_match = SECTION_RE.match(line)
        if section_match:
            section = Section(section_match.group(1), section_match.group(2), int(section_match.group(3)), line)
            sections.append(section)
            block = None
            entry = None
            continue
        if section is not None and line.startswith('## '):
            trailing.append(line)
            continue
        if section is None:
            preamble.append(line)
            continue
        day_match = DAY_RE.match(line)
        if day_match:
            block = DayBlock(day_match.group(1), day_match.group(2), [])
            section.days.append(block)
            entry = None
            continue
        entry_match = ENTRY_RE.match(line)
        if entry_match and block is not None:
            entry = Entry(entry_match.group(1), [])
            block.entries.append(entry)
            continue
        if entry is not None:
            entry.body.append(line)
        elif block is not None:
            block.lead.append(line)
        else:
            section.status_lines.append(line)
    if sections:
        sections[-1].trailing = trailing
    return preamble, sections


def render(preamble, sections):
    parts = ['\n'.join(preamble).rstrip('\n')] if any(l.strip() for l in preamble) else []
    for section in sections:
        chunk = [section.header]
        status = [l for l in section.status_lines if l.strip()]
        if status:
            chunk += [''] + status
        for block in section.days:
            chunk += ['', block.text().rstrip('\n')]
        parts.append('\n'.join(chunk))
    trailing = getattr(sections[-1], 'trailing', []) if sections else []
    if any(l.strip() for l in trailing):
        parts.append('\n'.join(trailing).rstrip('\n'))
    return '\n\n'.join(parts).rstrip('\n') + '\n'


def truncate_summary(text, limit=48):
    """指针里的摘要：不超过 limit 字，截在括注平衡、且不落在「编号（」中间的地方。"""
    if len(text) <= limit:
        return text
    for cut in range(limit, 0, -1):
        head = text[:cut]
        if head.count('（') != head.count('）'):
            continue
        if re.search(r'(?:[DECIG]\d*|I-\d*\.?\d*|G\d*\.?\d*)$', head) or head.endswith('（'):
            continue
        return head.rstrip(' ，、；：') + '…'
    return text[:limit] + '…'


FENCE_ROW_PREFIX_RE = re.compile(r'(?m)^\s*\d+: ?')
EMBEDDED_DATE_RE = re.compile(r'[A-Za-z0-9_./\-]*20\d\d-\d\d-\d\d[A-Za-z0-9_./\-]*')
QUOTE_PAIRS = (('「', '」'), ('『', '』'), ('“', '”'), ('"', '"'))   # 引名的四种引号：中日的两种与英语的两种（判法只在这一处）
CHINESE_QUOTE_PAIRS = QUOTE_PAIRS[:2]   # 弄坏开关 english-quotes-unprotected 退回的样子：只认「」『』
QUOTE_BREAK = ''  # 只给 --selftest 证红用：见 QUOTE_BREAK_MODES


def quote_span_pattern(pairs):
    """一对引号括住的一段，不跨行、不跨同种引号；从左往右成对认（"a" 与 "b" 之间那一段不算被引的）。"""
    return '|'.join(f'{re.escape(opening)}[^{re.escape(opening)}{re.escape(closing)}\\n]*{re.escape(closing)}' for opening, closing in pairs)


QUOTE_SPAN_RE = re.compile(quote_span_pattern(QUOTE_PAIRS))
CHINESE_QUOTE_SPAN_RE = re.compile(quote_span_pattern(CHINESE_QUOTE_PAIRS))
CODE_SPAN_RE = re.compile(r'`[^`\n]*`|' + quote_span_pattern(QUOTE_PAIRS))
CHINESE_CODE_SPAN_RE = re.compile(r'`[^`\n]*`|' + quote_span_pattern(CHINESE_QUOTE_PAIRS))
STANDALONE_DATE_RE = r'(?<![\w./\-])%s(?![\w./\-])'


def quote_span_re():
    return CHINESE_QUOTE_SPAN_RE if QUOTE_BREAK == 'english-quotes-unprotected' else QUOTE_SPAN_RE


def code_span_re():
    return CHINESE_CODE_SPAN_RE if QUOTE_BREAK == 'english-quotes-unprotected' else CODE_SPAN_RE


GIT_SHOW_COMMIT_RE = re.compile(r'(`git show )[0-9a-f]{7,40}(:)')


def ids_and_numbers(text):
    # 引用里的提交哈希是定位用的，哈希里的数字串不是正文的数
    stripped = DATE_RE.sub(' ', FENCE_ROW_PREFIX_RE.sub('', GIT_SHOW_COMMIT_RE.sub(r'\1\2', text)))
    ids = set(ID_RE.findall(stripped))
    without_ids = ID_RE.sub(' ', stripped)
    numbers = set(n for n in NUMBER_RE.findall(without_ids) if len(n.replace(',', '').replace('.', '')) >= 2)
    return ids, numbers


def embedded_dates(text):
    """路径、产物名、锚点里带的日期（`e6-units-2026-08-31.out`）：它们是名字的一部分，一个都不许删。"""
    return set(m for m in EMBEDDED_DATE_RE.findall(text) if m != DATE_RE.search(m).group(0))


def other_dates(text, block_date):
    """与所属标题不同的日期是事件的锚（「2026-09-03 的处置候选」），删不得。"""
    return set(d for d in DATE_RE.findall(text) if d != block_date)


def tokens_of(text, block_date):
    ids, numbers = ids_and_numbers(text)
    return ids | numbers | embedded_dates(text) | other_dates(text, block_date)


def flatten_for_quote(text):
    """比引文时不看空白、粗体与反引号：被引的原句在 git 版本里可能带 `**`，引文抄的时候去掉了。"""
    return re.sub(r'[\s*`]+', '', text)


def quoted_names_with_dates(text):
    """「2026-09-27：第 4 次跑」、"2026-10-13 (the 25th run)" 这类引的小节名，日期是名字的一部分；取引号里的正文，比的是它在不在可达范围里。"""
    return set(flatten_for_quote(span[1:-1]) for span in quote_span_re().findall(text) if DATE_RE.search(span))


def without_old_state_summaries(text):
    """「快查·改前」说的是旧态，旧态在 git 与改前引用里，它引的旧句子不算必须保留。"""
    return '\n'.join(l for l in text.split('\n') if not l.lstrip().startswith(('> 快查·改前', '快查·改前')))


def missing_quoted_names(before_text, reach_text):
    flat = flatten_for_quote(reach_text)
    return sorted(name for name in quoted_names_with_dates(without_old_state_summaries(before_text)) if name not in flat)


def strip_same_date(line, date):
    """只删独立出现的同日日期；反引号与引名的四种引号（QUOTE_PAIRS）里的，连在路径或名字里的，以及紧跟「：」当小节名用的不动。"""
    if date not in line:
        return line
    spans = []

    def keep(match):
        spans.append(match.group(0))
        return f'\x00{len(spans) - 1}\x00'

    line = code_span_re().sub(keep, line)
    line = re.sub(STANDALONE_DATE_RE % date + r'(?=：)', keep, line)
    standalone = STANDALONE_DATE_RE % date
    line = re.sub(r'（' + standalone + r'）', '', line)
    line = re.sub(r'（' + standalone + r'[，,]?\s*', '（', line)
    line = re.sub(r'[，,]?\s*' + standalone + r'）', '）', line)
    # 夹在正文里的：两边都是汉字或全角标点就不留空格（「用户 2026-09-24 定」→「用户定」），有一边是 ASCII 才留一个空格
    line = re.sub(r'\s*' + standalone + r'\s*', '\x01', line)
    line = re.sub(r'(\S)\x01(\S)', lambda m: m.group(1) + (' ' if (m.group(1).isascii() or m.group(2).isascii()) else '') + m.group(2), line)
    line = line.replace('\x01', '')
    line = re.sub(r'  +', ' ', line)
    line = re.sub(r'（ ', '（', line)
    line = re.sub(r' ）', '）', line)
    line = re.sub(r'\x00(\d+)\x00', lambda m: spans[int(m.group(1))], line)
    return line.rstrip()


def git_show(repo, revision, path):
    result = subprocess.run(['git', '-C', repo, 'show', f'{revision}:{path}'], capture_output=True, text=True)
    return result.stdout.split('\n') if result.returncode == 0 else None


def git_commits_touching(repo, path):
    result = subprocess.run(['git', '-C', repo, 'log', '--format=%H %cd', '--date=short', '--', path], capture_output=True, text=True)
    commits = []
    for line in result.stdout.split('\n'):
        if line.strip():
            full, date = line.split(' ', 1)
            commits.append((full, date))
    return commits


def find_old_version(repo, path, block_date, fence_rows):
    """围栏里的行是「旧版行号: 原文」；找一个提交，它的父版本逐行对得上九成以上。"""
    numbered = []
    for line in fence_rows:
        match = re.match(r'\s*(\d+): ?(.*)$', line)
        if match:
            numbered.append((int(match.group(1)), match.group(2), line))
    if not numbered or len(numbered) != len([l for l in fence_rows if l.strip()]):
        return None, 0.0, []
    candidates = [c for c in git_commits_touching(repo, path) if c[1] >= block_date]
    candidates.sort(key=lambda c: c[1])
    best = (None, 0.0, [])
    for full, _date in candidates[:6]:
        old = git_show(repo, full + '^', path)
        if old is None:
            continue
        unmatched = [raw for number, text, raw in numbered if not (number - 1 < len(old) and old[number - 1] == text)]
        share = 1 - len(unmatched) / len(numbered)
        if share > best[1]:
            best = (full, share, unmatched)
        if share >= 0.9:
            break
    if best[1] >= 0.9:
        short = subprocess.run(['git', '-C', repo, 'rev-parse', '--short', best[0] + '^'], capture_output=True, text=True).stdout.strip()
        return short, best[1], best[2]
    return None, best[1], []


def home_section_for(entry, holders, kind):
    holder_numbers = {h.number for h in holders}
    name_re = re.compile(kind + r'(\d+)（')
    for source in (entry.title, '\n'.join(entry.body)):
        for match in name_re.finditer(source):
            number = int(match.group(1))
            if number in holder_numbers:
                return next(h for h in holders if h.number == number)
    return min(holders, key=lambda h: h.number)


def transform_entry(entry, date, repo, report, section_name, break_mode):
    """机械四类里的三类：快查行、围栏、同日日期。返回新条目与它引的 git 版本内容。"""
    body = list(entry.body)
    referenced_git_text = ''
    # 围栏换 git 引用
    new_body = []
    index = 0
    while index < len(body):
        line = body[index]
        intro = FENCE_INTRO_RE.match(line)
        if intro and repo:
            path = intro.group(1)
            look = index + 1
            while look < len(body) and not body[look].strip():
                look += 1
            if look < len(body) and body[look].startswith('```'):
                end = look + 1
                while end < len(body) and not body[end].startswith('```'):
                    end += 1
                fence_rows = body[look + 1:end]
                revision, share, unmatched = find_old_version(repo, path, date, fence_rows)
                if revision:
                    old = git_show(repo, revision, path) or []
                    referenced_git_text += '\n'.join(old) + '\n'
                    line_count = len(old) - 1 if old and old[-1] == '' else len(old)
                    new_body.append(f'- **改前**：`git show {revision}:{path}`，整份 {line_count} 行；围栏里登记的行号按那一版。')
                    if unmatched:
                        # 对不上那一版的行照录：它们是那一版之后、这次改动之前的中间态，git 里没有
                        new_body += ['', f'  与那一版不同、照录的 {len(unmatched)} 行：', '', '  ```text'] + ['  ' + row for row in unmatched] + ['  ```']
                        report['fence_rows_kept'] += len(unmatched)
                    report['fences_replaced'] += 1
                    index = end + 1
                    continue
                report['fences_kept'].append((section_name, date, entry.title[:40], f'{share:.0%}'))
        new_body.append(line)
        index = 1 + index
    body = new_body
    # 快查行：数、编号、产物名与别的日期在条目别处都找得到才删
    rest_text = '\n'.join([entry.title] + [l for l in body if not QUICK_RE.match(l)]) + referenced_git_text
    rest_tokens = tokens_of(rest_text, date)
    kept = []
    conclusions = []
    dropped_any = False
    for line in body:
        if QUICK_RE.match(line):
            # 「快查·改后」「快查·改了什么」就是这条变更的结论句，留下当结论；「快查·改前」说的是旧态，旧态在 git 与改前引用里
            after_match = re.match(r'^> 快查·(改后|改了什么)：\s*(.*)$', line)
            if after_match and after_match.group(2).strip():
                conclusions.append(after_match.group(2).strip())
                dropped_any = True
                continue
            extra = tokens_of(line, date) - rest_tokens
            if not extra:
                report['quick_dropped'] += 1
                dropped_any = True
                continue
            report['quick_kept'].append((section_name, date, entry.title[:40], sorted(extra)))
        kept.append(line)
    if conclusions:
        report['conclusions_kept'] += len(conclusions)
        kept = [f'- **结论**：{c}' for c in conclusions] + [''] + kept
    body = kept
    if dropped_any:
        cleaned = []
        for line in body:
            if line.strip() == '>' and (not cleaned or cleaned[-1].strip() in ('', '>')):
                continue
            cleaned.append(line)
        body = cleaned
    # 同日日期
    title = ORDINAL_RE.sub('', entry.title)
    if date in title:
        title = strip_same_date(title, date)
        report['dates_stripped'] += 1
    stripped_body = []
    for line in body:
        if date in line:
            report['dates_stripped'] += line.count(date)
            line = strip_same_date(line, date)
        stripped_body.append(line)
    body = stripped_body
    if break_mode == 'drop-number':
        for position, line in enumerate(body):
            match = re.search(r'(?<![\d.])\d{2,}(?![\d.])', DATE_RE.sub(' ', line))
            if match and not line.startswith('- **改前**'):
                body[position] = line.replace(match.group(0), '', 1)
                break
    collapsed = []
    for line in body:
        if not line.strip() and collapsed and not collapsed[-1].strip():
            continue
        collapsed.append(line)
    body = collapsed
    while body and not body[0].strip():
        body.pop(0)
    while body and not body[-1].strip():
        body.pop()
    return Entry(title, body), referenced_git_text


def new_report():
    return {'fences_replaced': 0, 'fences_kept': [], 'fence_rows_kept': 0, 'quick_dropped': 0, 'quick_kept': [], 'conclusions_kept': 0, 'dates_stripped': 0,
            'status_lines': 0, 'pointers': 0, 'entries_before': 0, 'entries_after': 0, 'multi_day_blocks': []}


def migrate(text, repo, break_mode=None):
    preamble, sections = parse_history(text)
    kind = sections[0].kind if sections else 'D'
    report = new_report()
    holders = collections.defaultdict(list)
    for section in sections:
        for block in section.days:
            for entry in block.entries:
                report['entries_before'] += 1
                holders[(block.date, entry.key())].append(section)
    homes = {}
    for (date, key), sections_holding in holders.items():
        sample = next(e for s in sections_holding for b in s.days if b.date == date for e in b.entries if e.key() == key)
        homes[(date, key)] = home_section_for(sample, sections_holding, kind)
    git_texts = {}
    before_blocks = {}
    for section in sections:
        report['status_lines'] += sum(1 for l in section.status_lines if STATUS_LINE_RE.match(l))
        section.status_lines = [l for l in section.status_lines if not STATUS_LINE_RE.match(l) and l.strip()]
        for block in section.days:
            before_blocks[(section.name, block.date)] = block.text()
            new_entries = []
            for entry in block.entries:
                home = homes[(block.date, entry.key())]
                if home is section:
                    new_entry, git_text = transform_entry(entry, block.date, repo, report, section.name, break_mode)
                    new_entries.append(new_entry)
                    if git_text:
                        git_texts[(section.name, block.date)] = git_texts.get((section.name, block.date), '') + git_text
                else:
                    summary = truncate_summary(ORDINAL_RE.sub('', entry.title))
                    block.pointers.append(f'- 见 {home.name} 节同日条目「{summary}」。')
                    report['pointers'] += 1
            block.entries = new_entries
            block.heading_rest = strip_same_date(block.heading_rest, block.date) if block.date in block.heading_rest else block.heading_rest
            block.lead = [strip_same_date(l, block.date) for l in block.lead]
            report['entries_after'] += len(new_entries)
            if len(new_entries) > 1:
                report['multi_day_blocks'].append((section.name, block.date, len(new_entries)))
    return render(preamble, sections), report, before_blocks, git_texts


def block_map(text):
    _preamble, sections = parse_history(text)
    blocks = {}
    for section in sections:
        for block in section.days:
            blocks[(section.name, block.date)] = block
    return blocks


POINTER_RE = re.compile(r'见 ((?:D|E)\d+（.*?）) 节同日条目')
GIT_REF_RE = re.compile(r'`git show ([0-9a-f]+):([^`]+)`')
PRODUCT_LINE_RE = re.compile(r'RESULT|`[^`]*\.(?:out|tsv|log|json|csv)`')
GIT_SHOW_SPAN_RE = re.compile(r'`git show [0-9a-f]{7,40}:([^`\s]+)`')
ANY_PRODUCT_SPAN_RE = re.compile(r'`[^`]*\.(?:out|tsv|log|json|csv)`')
LOSS_CHECK_BREAK = ''  # 只给 --selftest 证红用：见 LOSS_CHECK_BREAK_MODES


def normalized_line(line, block_date=''):
    """比对产物行时不看缩进、列表符号、快查前缀、行号前缀、空白，也不看所属日期（它按规矩被删）；
    也不看引用写法：`<路径>` 与 `git show <提交>:<同一路径>` 算同一处（归档脚本删产物时把前者改成后者）。路径与产物名改了别的字照旧不同。"""
    text = FENCE_ROW_PREFIX_RE.sub('', line)
    text = GIT_SHOW_SPAN_RE.sub(lambda match: '`' + match.group(1) + '`', text)
    if LOSS_CHECK_BREAK == 'git-show-ignores-path':
        text = ANY_PRODUCT_SPAN_RE.sub('`产物`', text)
    text = re.sub(r'^\s*(?:>\s*|-\s+|\*\s+)+', '', text)  # 列表符号与引用符号；`**结论**` 的星号不是列表符号
    text = re.sub(r'^(?:快查·(?:改后|改了什么)：|\*\*结论\*\*：)\s*', '', text)
    if block_date:
        text = text.replace(block_date, '')
    return re.sub(r'[\s，,（）()]+', '', text)


def git_referenced_text(text, repo, cache):
    pieces = []
    if repo:
        for match in GIT_REF_RE.finditer(text):
            key = (match.group(1), match.group(2))
            if key not in cache:
                shown = git_show(repo, key[0], key[1])
                cache[key] = '\n'.join(shown) if shown else ''
            pieces.append(cache[key])
    return '\n'.join(pieces)


def reachable_text(block, after_blocks, repo, cache):
    """改后这一块、它指到的家节同日块、两者引的 git 版本，合起来是「找得到」的范围。"""
    text = block.text()
    pieces = [text, git_referenced_text(text, repo, cache)]
    for match in POINTER_RE.finditer(text):
        target = after_blocks.get((match.group(1), block.date))
        if target is not None:
            pieces.append(target.text())
            pieces.append(git_referenced_text(target.text(), repo, cache))
    return '\n'.join(pieces)


def loss_check(before_text, after_text, repo):
    before_blocks = {k: b.text() for k, b in block_map(before_text).items()}
    after_blocks = block_map(after_text)
    cache = {}
    losses = []
    checked = 0
    for key, before_block_text in before_blocks.items():
        after_block = after_blocks.get(key)
        if after_block is None:
            losses.append((key, ['整块不见了']))
            continue
        checked += 1
        reach = reachable_text(after_block, after_blocks, repo, cache)
        before_ids, before_numbers = ids_and_numbers(before_block_text)
        after_ids, after_numbers = ids_and_numbers(reach)
        missing = sorted(before_ids - after_ids) + sorted(before_numbers - after_numbers, key=lambda n: (len(n), n))
        missing += sorted(embedded_dates(before_block_text) - embedded_dates(reach))
        missing += sorted(other_dates(before_block_text, key[1]) - other_dates(reach, key[1]))
        missing += ['引名：' + name[:50] for name in missing_quoted_names(before_block_text, reach)]
        # 实验类的产物行整行抄自产物，一个字都不许变：这些行要在改后（含指到的家节）逐字找得到
        reach_lines = set(normalized_line(l, key[1]) for l in reach.split('\n'))
        for line in before_block_text.split('\n'):
            if line.lstrip().startswith(('> 快查·改前', '快查·改前')):
                continue  # 旧态的摘要，旧态在 git 与改前引用里；它带的数与编号另有 tokens 那一半在核
            if PRODUCT_LINE_RE.search(line) and normalized_line(line, key[1]) and normalized_line(line, key[1]) not in reach_lines:
                missing.append('产物行：' + line.strip()[:60])
        if missing:
            losses.append((key, missing))
    return checked, losses


def write_report(path, report, checked, losses, before_text, after_text):
    lines = ['# 变更史整理报告', '',
             f'- 行数：{before_text.count(chr(10))} → {after_text.count(chr(10))}；字节：{len(before_text.encode())} → {len(after_text.encode())}',
             f'- 条目：{report["entries_before"]} → {report["entries_after"]}（换成指针 {report["pointers"]} 条）',
             f'- 围栏换 git 引用 {report["fences_replaced"]} 处（其中与那一版不同、照录的行 {report["fence_rows_kept"]} 行），整段对不上留下的 {len(report["fences_kept"])} 处',
             f'- 快查·改后留成结论句 {report["conclusions_kept"]} 行；快查·改前删 {report["quick_dropped"]} 行，带独有数或编号而留下的 {len(report["quick_kept"])} 行',
             f'- 与标题同日的行内日期删 {report["dates_stripped"]} 个；节顶现状行删 {report["status_lines"]} 行',
             f'- 同一天仍不止一条、要人合成结论的日期块 {len(report["multi_day_blocks"])} 个',
             f'- 丢失核对：核了 {checked} 个日期块，有丢失的 {len(losses)} 块', '']
    if losses:
        lines += ['## 有丢失的块', '', '| 节 | 日期 | 找不到的数与编号 |', '|---|---|---|']
        for (section, date), missing in losses:
            lines.append(f'| {section} | {date} | {" ".join(missing)} |')
        lines.append('')
    if report['fences_kept']:
        lines += ['## 围栏对不上 git 版本、原样留下的', '']
        for section, date, title, share in report['fences_kept']:
            lines.append(f'- {section} {date}「{title}」对上 {share}')
        lines.append('')
    if report['quick_kept']:
        lines += ['## 快查行带独有的数或编号、原样留下的', '']
        for section, date, title, extra in report['quick_kept']:
            lines.append(f'- {section} {date}「{title}」独有：{" ".join(extra)}')
        lines.append('')
    if report['multi_day_blocks']:
        lines += ['## 同一天不止一条、待人合成结论的', '']
        for section, date, count in report['multi_day_blocks']:
            lines.append(f'- {section} {date}：{count} 条')
        lines.append('')
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write('\n'.join(lines))


SAMPLE = '''# 决策变更史

## D1（甲）

**现状**：已定 1 项。

### 2026-09-10

#### （其一）：D1（甲） 已定项 1 定案

> 快查·改前：原来 16 KiB。
>
> 快查·改后：改成 32 KiB，依据 E5（乙）。

- **改前**：节点 16 KiB（2026-09-10 现查）。
- **改后**：节点 32 KiB。
- **依据**：E5（乙） 量到 1.7 倍。

#### （其二）：D1（甲）、D2（丙） 两处口径对齐，新记 C222（I-7.4（近 K 代块未被复用） 的下限 2）与 D19（块指针的结构与宽度预算） 已定项 5

- 两处都写 3087.5 ppm。

## D2（丙）

**现状**：已定 2 项。

### 2026-09-10

#### （其一）：D1（甲）、D2（丙） 两处口径对齐，新记 C222（I-7.4（近 K 代块未被复用） 的下限 2）与 D19（块指针的结构与宽度预算） 已定项 5

- 两处都写 3087.5 ppm。

### 2026-09-09

#### D2（丙） 已定项 2

- 槽宽 555 字节，爆 43 字节（2026-09-09，用户定案）；产物 `e79-root-2026-09-09.out`，登记 `records/2026-09-09-定案.md`；页首加「2026-09-09：第 2 次跑」一节，标题 2026-09-09：改成已定。

## 历史版本

- 2026-09-12：文件自己的搬迁记录，原样保留在文末。
'''


BREAK_MODES = ('drop-number',)  # 弄坏开关 HISTORY_CONSOLIDATE_BREAK 认的取值，只给 --selftest 用（文件头逐个写了打开后哪几项红）
LOSS_CHECK_BREAK_MODES = ('git-show-ignores-path',)  # 弄坏的是 loss-check 本身（不是迁移结果）：产物行比对不看路径


GIT_SHOW_HISTORY = '''# 决策变更史

## E1（甲）

### 2026-09-10

#### E1（甲） 已定项 1

- 产物 `research/results/e1-run.%s`，通过 34 项。
'''


def git_show_cases(mode):
    """产物行改引用写法的几种情形：返回不合预期的句子（空表示 loss-check 全对）。mode 非空时打开 loss-check 的弄坏开关。"""
    global LOSS_CHECK_BREAK
    previous, LOSS_CHECK_BREAK = LOSS_CHECK_BREAK, mode
    problems = []
    try:
        for extension in ('out', 'tsv', 'log', 'json', 'csv'):
            plain = GIT_SHOW_HISTORY % extension
            shown = plain.replace(f'`research/results/e1-run.{extension}`',
                                  f'`git show 0123456789ab:research/results/e1-run.{extension}`')
            if shown == plain:
                problems.append(f'样本没换成 git show 写法（{extension}）')
            for name, before, after in (('写成 git show', plain, shown), ('git show 写回路径', shown, plain)):
                if loss_check(before, after, repo=None)[1]:
                    problems.append(f'.{extension} 产物行{name}不该判丢')
            renamed = shown.replace('e1-run', 'e1-other')
            moved = shown.replace('research/results/', 'research/old/')
            reworded = shown.replace('通过', '失败')
            for name, after in (('产物名改了', renamed), ('目录改了', moved), ('同一行别的字改了', reworded)):
                if not loss_check(plain, after, repo=None)[1]:
                    problems.append(f'.{extension} 产物行换成 git show 写法又{name}，要判丢，没判')
    finally:
        LOSS_CHECK_BREAK = previous
    return problems


QUOTE_BREAK_MODES = ('english-quotes-unprotected',)  # 弄坏的是引名的认法：英语两种引号退回不认
QUOTE_CASE_DATE = '2026-09-26'
# （格名, 一行正文, 被引的小节名）：同一行里引号外有一处同日日期要删，引号里那一处要留
ENGLISH_QUOTE_CASES = (
    ('英语直引号', '- On 2026-09-26 the section "2026-09-26 (the 4th run)" was rerun.', '"2026-09-26 (the 4th run)"'),
    ('英语弯引号', '- On 2026-09-26 the section “2026-09-26 (the 4th run)” was rerun.', '“2026-09-26 (the 4th run)”'),
)
# 两对直引号之间的日期不在引号里：成对认，不把前一对的右引号与后一对的左引号认成一对
BETWEEN_QUOTES_LINE = '- The "A" run on 2026-09-26 and the "B" run.'


def english_quote_cases(mode):
    """英语引号里的引名：返回不合预期的句子（空表示全对）。mode 非空时打开引名认法的弄坏开关。"""
    global QUOTE_BREAK
    previous, QUOTE_BREAK = QUOTE_BREAK, mode
    problems = []
    try:
        for name, line, quoted in ENGLISH_QUOTE_CASES:
            stripped = strip_same_date(line, QUOTE_CASE_DATE)
            if quoted not in stripped:
                problems.append(f'{name}里的同日日期被删了：{line} → {stripped}')
            if stripped.count(QUOTE_CASE_DATE) != 1:
                problems.append(f'{name}那一行引号外的同日日期该删、引号里的该留，剩 {stripped.count(QUOTE_CASE_DATE)} 处：{stripped}')
            if not missing_quoted_names(line, line.replace(quoted, 'the 4th run')):
                problems.append(f'{name}里带日期的引名改后不见了，丢失核对没判丢')
        between = strip_same_date(BETWEEN_QUOTES_LINE, QUOTE_CASE_DATE)
        if QUOTE_CASE_DATE in between:
            problems.append(f'两对直引号之间的同日日期不在引号里，该删没删：{between}')
    finally:
        QUOTE_BREAK = previous
    return problems


def selftest():
    break_mode = os.environ.get('HISTORY_CONSOLIDATE_BREAK', '')
    if break_mode and break_mode not in BREAK_MODES + LOSS_CHECK_BREAK_MODES + QUOTE_BREAK_MODES:
        print(f'✗ 自检：HISTORY_CONSOLIDATE_BREAK={break_mode} 认不得')
        print(f'  → 取值只有 {" ".join(BREAK_MODES + LOSS_CHECK_BREAK_MODES + QUOTE_BREAK_MODES)}（文件头「弄坏开关」一段）；平常跑 --selftest 不设它')
        return 2
    after, report, _before_blocks, _git = migrate(SAMPLE, repo=None, break_mode=break_mode if break_mode in BREAK_MODES else None)
    checked, losses = loss_check(SAMPLE, after, repo=None)
    problems = []
    if losses:
        problems.append(f'干净迁移不该有丢失：{losses}')
    if report['pointers'] != 1 or '见 D1（甲） 节同日条目' not in after:
        problems.append('批量条目没换成指针')
    if '快查' in after:
        problems.append('快查行没删')
    if '- **结论**：改成 32 KiB，依据 E5（乙）。' not in after:
        problems.append('快查·改后没留成结论句')
    if '2026-09-10 现查' in after or '（2026-09-09，用户定案）' in after:
        problems.append('同日日期没删')
    if 'e79-root-2026-09-09.out' not in after or 'records/2026-09-09-定案.md' not in after:
        problems.append('路径与产物名里的日期被当成同日日期删了')
    if '「2026-09-09：第 2 次跑」' not in after or '标题 2026-09-09：改成已定' not in after:
        problems.append('引名与「日期：」形态里的日期被删了')
    if '2026-09-09' not in after.split('## D2（丙）')[1].split('### 2026-09-09')[0] and '### 2026-09-09' not in after:
        problems.append('日期块标题丢了')
    if '**现状**' in after:
        problems.append('现状行没删')
    if '（其一）' in after or '（其二）' in after:
        problems.append('（其N）没删')
    if '#### ' in after or '- D1（甲） 已定项 1 定案' not in after:
        problems.append('旧条目没变成日期下的变更项')
    pointer = next((l for l in after.split('\n') if l.startswith('- 见 D1（甲） 节同日条目「')), '')
    if not pointer or pointer.count('（') != pointer.count('）') or '「D1（甲）、D2（丙） 两处口径对齐，新记…」' not in pointer:
        problems.append(f'指针摘要没截在括注平衡处：{pointer[:80]}')
    if not after.rstrip('\n').endswith('- 2026-09-12：文件自己的搬迁记录，原样保留在文末。') or '  ## 历史版本' in after:
        problems.append('文件自己文末的「## 历史版本」节没有原样留在文末')
    if len(report['multi_day_blocks']) != 1:
        problems.append(f'同日多条应报 1 个块，报了 {report["multi_day_blocks"]}')
    for case in git_show_cases(break_mode if break_mode in LOSS_CHECK_BREAK_MODES else ''):
        problems.append('git show 引用写法：' + case)
    caught = []
    for mode in LOSS_CHECK_BREAK_MODES:
        if not git_show_cases(mode):
            problems.append(f'弄坏开关 {mode}：loss-check 不看路径，git show 引用写法的几种情形没红')
        else:
            caught.append(f'{mode} 红')
    for case in english_quote_cases(break_mode if break_mode in QUOTE_BREAK_MODES else ''):
        problems.append('英语引号里的引名：' + case)
    for mode in QUOTE_BREAK_MODES:
        if not english_quote_cases(mode):
            problems.append(f'弄坏开关 {mode}：引号退回只认「」『』，英语引号那一格没红')
        else:
            caught.append(f'{mode} 红')
    for mode in BREAK_MODES:
        broken, _r, _b, _g = migrate(SAMPLE, repo=None, break_mode=mode)
        _checked, broken_losses = loss_check(SAMPLE, broken, repo=None)
        if not broken_losses:
            problems.append(f'弄坏开关 {mode}：故意丢一个数，loss-check 没红')
        else:
            caught.append(f'{mode} {broken_losses[0][1]}')
    if problems:
        for problem in problems:
            print('✗ 自检：' + problem)
        print('  → 改 history-consolidate.py 的对应分支，再跑 --selftest')
        return 1
    print(f'✓ 自检通过：干净迁移零丢失，指针 1 条，快查 / 现状 / 同日日期 / （其N）都删了，英语引号 {len(ENGLISH_QUOTE_CASES)} 种里的引名日期留着，'
          f'弄坏开关 {len(caught)} 个各开一次、都被抓（{"；".join(caught)}）')
    return 0


def main():
    preflight(__file__)
    parser = argparse.ArgumentParser(description='变更史整理与丢失度核对')
    parser.add_argument('command', nargs='?', choices=['migrate', 'loss-check'])
    parser.add_argument('files', nargs='*')
    parser.add_argument('--out')
    parser.add_argument('--report')
    parser.add_argument('--repo', default=os.getcwd())
    parser.add_argument('--selftest', action='store_true')
    arguments = parser.parse_args()
    if arguments.selftest:
        sys.exit(selftest())
    if arguments.command == 'migrate':
        if len(arguments.files) != 1 or not arguments.out:
            print('✗ 用法：migrate <变更史文件> --out <输出文件> [--report 报告]')
            print('  → 给一份输入文件与 --out')
            sys.exit(1)
        before_text = open(arguments.files[0], encoding='utf-8').read()
        after_text, report, _blocks, _git = migrate(before_text, arguments.repo)
        with open(arguments.out, 'w', encoding='utf-8') as handle:
            handle.write(after_text)
        checked, losses = loss_check(before_text, after_text, arguments.repo)
        if arguments.report:
            write_report(arguments.report, report, checked, losses, before_text, after_text)
        print(f'条目 {report["entries_before"]} → {report["entries_after"]}，指针 {report["pointers"]}，围栏换引用 {report["fences_replaced"]}（留下 {len(report["fences_kept"])}），'
              f'快查删 {report["quick_dropped"]}（留下 {len(report["quick_kept"])}），同日日期删 {report["dates_stripped"]}，待人合并的块 {len(report["multi_day_blocks"])}')
        if losses:
            print(f'✗ 丢失核对：{len(losses)} 个日期块有找不到的数或编号（报告里逐个列出）')
            print('  → 逐块写去向：补回、改指针，或确认是有意删的过程数')
            sys.exit(3)
        print(f'✓ 丢失核对：{checked} 个日期块的数与编号都找得到')
        sys.exit(0)
    if arguments.command == 'loss-check':
        if len(arguments.files) != 2:
            print('✗ 用法：loss-check <改前文件> <改后文件>')
            print('  → 给两份文件')
            sys.exit(1)
        before_text = open(arguments.files[0], encoding='utf-8').read()
        after_text = open(arguments.files[1], encoding='utf-8').read()
        checked, losses = loss_check(before_text, after_text, arguments.repo)
        if arguments.report:
            write_report(arguments.report, new_report(), checked, losses, before_text, after_text)
        for (section, date), missing in losses:
            print(f'  {section} {date}：{" ".join(missing)}')
        if losses:
            print(f'✗ {len(losses)} 个日期块有找不到的数或编号')
            print('  → 逐块写去向：补回、改指针，或确认是有意删的过程数')
            sys.exit(3)
        print(f'✓ 核了 {checked} 个日期块，数与编号都找得到')
        sys.exit(0)
    parser.print_help()
    sys.exit(1)


if __name__ == '__main__':
    main()
