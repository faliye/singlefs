#!/usr/bin/env python3
# admission: always 每一次调都是对此刻那份文件做一次整理或核对，上一次的结论不替这一次作保
# run-condition: command git python3
"""普通 Markdown 文档（records/、里程碑、kb 顶层与决策 / 实验页正文）的整理与丢失度核对。

变更史那两份归 history-consolidate.py；这里管别的文档，机械做的只有三类，别的一个字不动：

1. 与「所属日期」相同的行内日期删掉：所属日期是文件名里的日期（records/2026-09-24-….md）与最近一个带日期的标题；
   路径、产物名、引名（「」『』与英语 "…"、“…” 四种引号里的）、紧跟「：」当小节名用的日期不动（判法与 history-consolidate.py 同一份）。
2. 同一份文件里同一日期、同一级的标题出现第二次，第二个小节并进第一个：内容原样接在第一个小节末尾，
   第二个标题日期之后的字（「第二批」这类）留成一行粗体。
3. `--drop-history`：文末的历史节整个删掉（只有决策与实验有历史，别的文件的历史靠 git）。

历史节：整行是「## <历史版本的某种语言写法>」的那一行起到文末。三种写法取自翻译术语表（.claude/kb/translation-glossary.md，
本仓那一份；`DOC_CONSOLIDATE_GLOSSARY` 指别的路径时读它）中文列是「历史版本」的那一行：`## 历史版本`、`## <English 列>`、`## <日本語列>`，
表的列照 .claude/gate.d/lib-trilingual.py 的 parse_glossary 读，只认这三种，别的译名（`## History` 这类）照正文判。
历史节里的同日日期与同日小节不整理（上面 1、2 两类只管历史节之前的正文）。
术语表读不到、或没有「历史版本」那一行：migrate 与 loss-check 退 5，打 ✗ 加 → 出路（补那一行），不退回只认中文。

丢失度：改前正文（历史节之外）的数、编号、产物名、别的日期、带日期的引名、产物行，逐个到改后里找；找不到的逐个列出，退出码 3。
被删的历史节里独有的数与编号另列一栏「历史节独有（在 git 里）」，不算丢失。

用法：
  doc-consolidate.py migrate <文件> --out <输出> [--drop-history] [--report <报告.md>]
  doc-consolidate.py loss-check <改前> <改后> [--dropped-history]
  doc-consolidate.py --selftest

对迁过的文件拒跑（migrate）：同一份文件迁第二次，丢失核对拿的「改前」已经是上一趟迁出来（或迁到一半丢了东西）的正文，丢掉的东西永远核不出来，
而第二趟的 --out 若指回它自己，原文件就被盖掉了。所以 migrate 每次真改了东西（输出与输入不同）就把输出正文的 sha256 记进台账，
下一趟的输入正文的 sha256 在台账里就是「迁过的文件」，拒跑：不整理它、不写 --out、不碰它，退出码 4，打 ✗ 加 → 出路。
  怎么认迁过的：台账 `${DOC_CONSOLIDATE_LEDGER:-~/.local/share/singlefs/doc-consolidate-migrated.tsv}`，一行 `<输出正文的 sha256>\t<文件名>`，
    认的是正文的 sha256，不认路径：文件改过（哪怕改一个字）就不再是迁出来的那一份；原文件（迁之前的那一份）重新迁不拒。
    `DOC_CONSOLIDATE_LEDGER` 设成空串或 off：台账关掉，不查也不记（门禁 doc-text 的 date-once 格干跑用它：干跑不留台账）。
  出路：要改它的内容，直接定点改（scripts/kb/replace-once.py），不再迁；真要重迁，对迁之前的那一份（git show <提交>:<路径>）重新 migrate。
  只记 migrate 真改了东西的那一趟，也包括丢失核对报了丢失（退 3）的那一趟：那时 --out 已经写出去了。
退出码：migrate 0 迁好且丢失核对绿（或没有可迁的）、1 用法错、3 丢失核对找不到东西、4 拒跑（输入是迁过的文件）、5 认不出历史节（术语表读不到或没有「历史版本」那一行）；
loss-check 0 / 1 / 3 / 5；--selftest 0 / 1 / 2（弄坏开关取值认不得）。

弄坏开关（只给 --selftest 证红用，migrate 与 loss-check 不认）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句冒号前的名字，
括号里是那条「✗ 自检：」的开头）。不设开关时 --selftest 把 BREAK_MODES 里每个开关各开一次跑 migrate_document()，核 loss-check 都红
（没红的那一项是「弄坏开关 <名>」，失败句「✗ 自检：弄坏开关 <名>：故意丢一个数，loss-check 没红」）；设了认不得的取值退 2。
  DOC_CONSOLIDATE_BREAK=migrated-not-recognized  不查台账，迁过的文件照迁（不进 migrate_document() 的丢数循环，单独一格）
      对迁过的文件拒跑（「✗ 自检：对迁过的文件拒跑」那一格：第二趟没退 4 或文件被改了）
  DOC_CONSOLIDATE_BREAK=drop-number  migrate_document() 把正文里第一处两位以上的数（标题行与日期除外）删掉一个，样本里丢的是产物名 e142-r18-… 里的 142
      产物名或引名里的日期被删了（「✗ 自检：产物名或引名里的日期被删了」）、
      干净整理不该有丢失（「✗ 自检：干净整理不该有丢失：」）
  DOC_CONSOLIDATE_BREAK=history-heading-chinese-only  历史节标题退回只认「## 历史版本」（改前的样子），术语表里那一行的英语、日语写法不认，译文的历史节当正文判
      译文的历史节标题（「✗ 自检：译文的历史节标题：」，逐格点名是哪一份、干跑删了几个日期并了几个小节）
      不设开关时 --selftest 也把这个开关打开一次，核那一格红（没红的那一项是 弄坏开关 history-heading-chinese-only）
"""
import argparse
import collections
import hashlib
import importlib.util
import os
import re
import subprocess
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

_SPEC = importlib.util.spec_from_file_location('history_consolidate', os.path.join(os.path.dirname(os.path.realpath(__file__)), 'history-consolidate.py'))
history = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(history)

DATE_RE = history.DATE_RE
HEADING_RE = re.compile(r'^(#{1,6})\s+(.*)$')
HISTORY_TERM = '历史版本'     # 术语表里中文列是它的那一行给三种语言的历史节标题
CHINESE_HISTORY_HEADING = '## ' + HISTORY_TERM
LEDGER_VARIABLE = 'DOC_CONSOLIDATE_LEDGER'
DEFAULT_LEDGER = '~/.local/share/singlefs/doc-consolidate-migrated.tsv'
REFUSED_EXIT = 4        # migrate 的输入是迁过的文件：拒跑
GLOSSARY_EXIT = 5       # 认不出历史节：术语表读不到或没有「历史版本」那一行
GLOSSARY_VARIABLE = 'DOC_CONSOLIDATE_GLOSSARY'
REPOSITORY_ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', '..'))
TRILINGUAL_LIBRARY = os.path.join(REPOSITORY_ROOT, '.claude', 'gate.d', 'lib-trilingual.py')
HISTORY_HEADING_BREAK_MODE = 'history-heading-chinese-only'


class GlossaryError(Exception):
    """认不出历史节：术语表读不到，或没有中文列是「历史版本」的那一行。不退回只认中文。"""


_trilingual = None
_history_headings = {}


def trilingual_library():
    """读术语表的那一份（lib-trilingual.py 的 GLOSSARY 与 parse_glossary），不另写解析。"""
    global _trilingual
    if _trilingual is None:
        try:
            spec = importlib.util.spec_from_file_location('lib_trilingual', TRILINGUAL_LIBRARY)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
        except Exception as error:  # noqa: BLE001 —— 装不进来就认不出历史节，交给调用方判错
            raise GlossaryError(f'装不进读术语表的那一份 {TRILINGUAL_LIBRARY}：{error}') from error
        _trilingual = module
    return _trilingual


def glossary_path():
    configured = os.environ.get(GLOSSARY_VARIABLE, '')
    if configured:
        return configured
    return os.path.join(REPOSITORY_ROOT, trilingual_library().GLOSSARY)


def load_history_headings(path):
    """术语表里中文列是「历史版本」的那一行 → (`## 历史版本`, `## <English 列>`, `## <日本語列>`)。"""
    library = trilingual_library()
    try:
        with open(path, encoding='utf-8') as handle:
            text = handle.read()
    except (OSError, UnicodeDecodeError) as error:
        raise GlossaryError(f'读不到翻译术语表 {path}：{error}') from error
    terms, _problems = library.parse_glossary(text)
    rows = [row for row in terms if row[1] == HISTORY_TERM]
    if not rows:
        raise GlossaryError(f'翻译术语表 {path} 里没有中文列是「{HISTORY_TERM}」、三种语言都写了的那一行'
                            f'（标记、表头与列照 lib-trilingual.py 的 parse_glossary 读），认不出译文的历史节')
    _number, chinese, english, japanese = rows[0]
    return ('## ' + chinese, '## ' + english, '## ' + japanese)


def history_headings():
    """三种语言的历史节标题；弄坏开关 history-heading-chinese-only 退回只认中文（改前的样子）。"""
    if os.environ.get('DOC_CONSOLIDATE_BREAK') == HISTORY_HEADING_BREAK_MODE:
        return (CHINESE_HISTORY_HEADING,)
    path = glossary_path()
    if path not in _history_headings:
        _history_headings[path] = load_history_headings(path)
    return _history_headings[path]


def glossary_problem_exit(error):
    print(f'✗ 认不出历史节：{error}')
    print(f'  → 怎么办：在翻译术语表（本仓 .claude/kb/translation-glossary.md；设了 {GLOSSARY_VARIABLE} 的是它指的那一份）的表里补一行'
          f'「| {HISTORY_TERM} | <English> | <日本語> | |」（kb 译文的历史节标题统一成这一种写法），补好再跑；'
          '装不进 lib-trilingual.py 的照上面的报错修它。不退回只认中文：那样译文的历史节会被当正文整理')
    sys.exit(GLOSSARY_EXIT)


def split_history(lines):
    """历史节之前的正文与历史节（整行是三种语言的历史节标题之一的那一行起到文末）。认不出历史节抛 GlossaryError。"""
    headings = history_headings()
    for index, line in enumerate(lines):
        if line.strip() in headings:
            return lines[:index], lines[index:]
    return lines, []


def fenced_flags(lines):
    flags = []
    inside = False
    for line in lines:
        if line.startswith('```') or line.startswith('~~~'):
            inside = not inside
            flags.append(True)
            continue
        flags.append(inside)
    return flags


def merge_same_date_headings(lines, report):
    """同一份文件里同一日期、同一级的第二个小节并进第一个。"""
    flags = fenced_flags(lines)
    headings = []  # (index, level, date, rest)
    for index, line in enumerate(lines):
        if flags[index]:
            continue
        match = HEADING_RE.match(line)
        if not match:
            continue
        text = match.group(2)
        date = DATE_RE.match(text.strip())
        if date:
            headings.append((index, len(match.group(1)), date.group(0), text.strip()[len(date.group(0)):].strip(' ：:—-')))
    if not headings:
        return lines
    # 每个小节的范围：到下一个同级或更高级标题之前
    def section_end(start_index, level):
        for later in range(start_index + 1, len(lines)):
            if flags[later]:
                continue
            match = HEADING_RE.match(lines[later])
            if match and len(match.group(1)) <= level:
                return later
        return len(lines)
    first_by_key = {}
    moves = []  # (from_start, from_end, into_end, extra_title)
    for index, level, date, rest in headings:
        key = (level, date)
        if key not in first_by_key:
            first_by_key[key] = (index, level)
            continue
        moves.append((index, section_end(index, level), key, rest))
    if not moves:
        return lines
    removed = set()
    appended = collections.defaultdict(list)
    for start, end, key, rest in moves:
        body = lines[start + 1:end]
        while body and not body[-1].strip():
            body.pop()
        chunk = ([f'**{rest}**'] if rest else []) + body
        appended[key] += [''] + chunk
        removed.update(range(start, end))
        report['sections_merged'] += 1
    out = []
    for index, line in enumerate(lines):
        if index in removed:
            continue
        out.append(line)
        # 第一个小节的末尾：它的范围结束前一行
    # 把并进去的内容插到第一个小节末尾
    result = []
    index = 0
    inserted = set()
    while index < len(lines):
        if index in removed:
            index += 1
            continue
        result.append(lines[index])
        for key, (first_index, level) in first_by_key.items():
            if key in inserted or first_index != index:
                continue
            end = section_end(first_index, level)
            # 收集第一个小节自己的内容，再接并入的
            own = []
            walk = first_index + 1
            while walk < end:
                if walk not in removed:
                    own.append(lines[walk])
                walk += 1
            while own and not own[-1].strip():
                own.pop()
            result += own + appended.get(key, []) + ['']
            inserted.add(key)
            index = end
            break
        else:
            index += 1
    return result


MACHINE_READ_RE = re.compile(r'20\d\d-\d\d-\d\d\s*(?:改了|不受影响)|改新池新建文件的字节：|动不动格式：|回看')


def strip_dates(lines, file_date, report):
    """机器要读日期的行不动：实验页「影响的决策」表的回看列（门禁 doc-experiments 的 decision-links 格）、未定项两句字节判决（doc-decisions 的 blocking-verdict 格）。"""
    flags = fenced_flags(lines)
    current = file_date
    out = []
    for index, line in enumerate(lines):
        if flags[index] or MACHINE_READ_RE.search(line):
            out.append(line)
            continue
        match = HEADING_RE.match(line)
        if match:
            # 所属日期是最近一个标题里的日期；最近的标题没有日期，这一段就只有文件名的日期
            found = DATE_RE.search(match.group(2))
            current = found.group(0) if found else file_date
            out.append(line)
            continue
        new_line = line
        for date in {d for d in (current, file_date) if d}:
            if date in new_line:
                before = new_line.count(date)
                new_line = history.strip_same_date(new_line, date)
                report['dates_stripped'] += before - new_line.count(date)
        out.append(new_line)
    return out


def migrate_document(text, file_date, drop_history, break_mode=None):
    report = {'dates_stripped': 0, 'sections_merged': 0, 'history_dropped_lines': 0}
    lines = text.split('\n')
    body, hist = split_history(lines)
    body = merge_same_date_headings(body, report)
    body = strip_dates(body, file_date, report)
    if drop_history and hist:
        report['history_dropped_lines'] = len(hist)
        hist = []
    if break_mode == 'drop-number':
        for position, line in enumerate(body):
            found = re.search(r'(?<![\d.])\d{2,}(?![\d.])', DATE_RE.sub(' ', line))
            if found and not line.startswith('#'):
                body[position] = line.replace(found.group(0), '', 1)
                break
    while body and not body[-1].strip():
        body.pop()
    result = body + ([''] + hist if hist else [])
    return '\n'.join(result).rstrip('\n') + '\n', report


def ledger_file():
    """台账路径；DOC_CONSOLIDATE_LEDGER 设成空串或 off 就关掉（返回 None），不查也不记。"""
    configured = os.environ.get(LEDGER_VARIABLE)
    if configured is not None and configured.strip().lower() in ('', 'off'):
        return None
    return os.path.expanduser(configured or DEFAULT_LEDGER)


def text_digest(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()


def was_migrated(text):
    """这份正文是不是上一趟 migrate 迁出来的（台账里有它的 sha256）。台账不在、关了，都当没迁过。"""
    path = ledger_file()
    if path is None or os.environ.get('DOC_CONSOLIDATE_BREAK') == 'migrated-not-recognized':
        return False
    try:
        with open(path, encoding='utf-8') as handle:
            return any(line.split('\t', 1)[0] == text_digest(text) for line in handle)
    except OSError:
        return False


def record_migrated(after_text, name):
    path = ledger_file()
    if path is None:
        return
    os.makedirs(os.path.dirname(path) or '.', exist_ok=True)
    with open(path, 'a', encoding='utf-8') as handle:
        handle.write(f'{text_digest(after_text)}\t{os.path.basename(name)}\n')


def file_date_of(path):
    found = DATE_RE.search(os.path.basename(path))
    return found.group(0) if found else ''


def loss_check_document(before_text, after_text, file_date, dropped_history):
    before_lines = before_text.split('\n')
    body, hist = split_history(before_lines)
    body_text = '\n'.join(body)
    hist_text = '\n'.join(hist)
    reach = after_text
    tokens_before = history.tokens_of(body_text, '') - {file_date}
    tokens_after = history.tokens_of(reach, '')
    # 与所属日期相同的日期按规矩被删：文件日期与标题日期都不算丢
    heading_dates = set(DATE_RE.findall('\n'.join(l for l in body if HEADING_RE.match(l))))
    missing = sorted(t for t in tokens_before - tokens_after if t not in heading_dates and t != file_date)
    missing += ['引名：' + n[:50] for n in history.missing_quoted_names(body_text, reach)]
    def normalize(line):
        text = history.normalized_line(line, file_date)
        for date in heading_dates:
            text = text.replace(date, '')
        return text
    reach_lines = set(normalize(l) for l in reach.split('\n'))
    for line in body:
        if history.PRODUCT_LINE_RE.search(line) and normalize(line) and normalize(line) not in reach_lines:
            missing.append('产物行：' + (line.strip()[:60] or repr(line[:60])))
    history_only = []
    if dropped_history and hist:
        history_only = sorted(t for t in history.tokens_of(hist_text, '') - tokens_after - heading_dates if t != file_date)
    return missing, history_only


SAMPLE = '''# 里程碑二收尾调度（2026-09-24）

## 一、批次

- 实一交回（2026-09-24；报告 `/tmp/x/report.md`）：跑了 5 轮，`e142-r18-2026-09-26.out` 是产物。
- 用户 2026-09-24 定：先收耗时表。

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D1（甲） 已定项 1 | 支撑 | 2026-09-24 不受影响：理由 |

### 2026-09-26 第一批

- 第一批做了甲（2026-09-26 现查）。

### 2026-09-26 第二批

- 第二批做了乙，见「2026-09-26：第 4 次跑」。

## 历史版本

### 2026-09-25

- 加了 3 行。
'''


BREAK_MODES = ('drop-number',)  # 弄坏开关 DOC_CONSOLIDATE_BREAK 认的、migrate_document() 里丢数的取值，--selftest 各开一次核 loss-check 红
MIGRATED_BREAK_MODE = 'migrated-not-recognized'     # 不查台账的取值，单独一格核（migrate_twice_problems），不进上面丢数的循环
ACCEPTED_BREAK_MODES = BREAK_MODES + (MIGRATED_BREAK_MODE, HISTORY_HEADING_BREAK_MODE)   # 文件头逐个写了打开后哪几项红

GLOSSARY_SAMPLE = '''# 翻译术语表

<!-- trilingual-glossary -->
| 中文 | English | 日本語 | 说明 / Note / 注 |
|---|---|---|---|
| 门禁 | gate | ゲート | |
'''
GLOSSARY_HISTORY_ROW = f'| {HISTORY_TERM} | Historical Versions | 履歴バージョン | |\n'

ENGLISH_HISTORY_SAMPLE = '''# E1 Sample experiment

## 2026-09-24: first part

- The first part did A.

{heading}

### 2026-09-25

- Added 3 rows (2026-09-25).

### 2026-09-25

- Added the fourth row.
'''

JAPANESE_HISTORY_SAMPLE = '''# E1 サンプル実験

## 2026-09-24：第一段

- 第一段で A をした。

{heading}

### 2026-09-25

- 3 行を追加（2026-09-25）。

### 2026-09-25

- 4 行目を追加。
'''

# （格名, 文件名, 正文, 该不该被判）：历史节标题是术语表那一行的写法的，历史节里同一天的两块与重复日期不整理；别的译名照正文判
TRANSLATED_HISTORY_CASES = (
    ('英语页 ## Historical Versions', '01-sample.en.md', ENGLISH_HISTORY_SAMPLE.format(heading='## Historical Versions'), False),
    ('日语页 ## 履歴バージョン', '01-sample.ja.md', JAPANESE_HISTORY_SAMPLE.format(heading='## 履歴バージョン'), False),
    ('英语页 ## History（别的译名）', '02-sample.en.md', ENGLISH_HISTORY_SAMPLE.format(heading='## History'), True),
)
DRY_RUN_COUNTS_RE = re.compile(r'同日日期删 (\d+)，同日小节合并 (\d+)')


def write_glossary(directory, name, with_history_row):
    path = os.path.join(directory, name)
    with open(path, 'w', encoding='utf-8') as handle:
        handle.write(GLOSSARY_SAMPLE + (GLOSSARY_HISTORY_ROW if with_history_row else ''))
    return path


def run_migrate(source, out, environment):
    return subprocess.run([sys.executable, '-B', os.path.realpath(__file__), 'migrate', source, '--out', out],
                          env=environment, capture_output=True, text=True)


def translated_history_problems(work, mode):
    """译文的历史节标题：照 date-once 的起法（台账关掉）干跑三份译文，核历史节认没认对。mode 非空时打开弄坏开关。"""
    environment = dict(os.environ, **{LEDGER_VARIABLE: 'off'})
    environment.pop('DOC_CONSOLIDATE_BREAK', None)
    if mode:
        environment['DOC_CONSOLIDATE_BREAK'] = mode
    problems = []
    for name, file_name, text, judged in TRANSLATED_HISTORY_CASES:
        source = os.path.join(work, file_name)
        with open(source, 'w', encoding='utf-8') as handle:
            handle.write(text)
        result = run_migrate(source, source + '.out', environment)
        counts = DRY_RUN_COUNTS_RE.search(result.stdout)
        if result.returncode != 0 or not counts:
            problems.append(f'{name}：干跑应当退 0 并报删了几个日期、并了几个小节，实际退 {result.returncode}：{(result.stdout + result.stderr)[-300:]}')
            continue
        dates, merges = int(counts.group(1)), int(counts.group(2))
        if judged and not (dates and merges):
            problems.append(f'{name}：不是术语表那一行的写法，历史节应当照正文判（删 1 个日期、并 1 个小节），实际删 {dates} 个、并 {merges} 个')
        if not judged and (dates or merges):
            problems.append(f'{name}：历史节被当正文判了，干跑删了 {dates} 个日期、并了 {merges} 个小节')
    return problems


def glossary_missing_problems(work):
    """术语表没有「历史版本」那一行、或读不到：退 5，打 ✗ 与出路，--out 没写；不退回只认中文。"""
    problems = []
    source = os.path.join(work, '03-sample.en.md')
    with open(source, 'w', encoding='utf-8') as handle:
        handle.write(ENGLISH_HISTORY_SAMPLE.format(heading='## Historical Versions'))
    for name, glossary in (('术语表没有那一行', write_glossary(work, 'glossary-without-row.md', False)),
                           ('术语表读不到', os.path.join(work, 'no-such-glossary.md'))):
        environment = dict(os.environ, **{LEDGER_VARIABLE: 'off', GLOSSARY_VARIABLE: glossary})
        environment.pop('DOC_CONSOLIDATE_BREAK', None)
        out = os.path.join(work, 'glossary-missing.out')
        result = run_migrate(source, out, environment)
        if result.returncode != GLOSSARY_EXIT or '✗ 认不出历史节' not in result.stdout or '→ 怎么办' not in result.stdout:
            problems.append(f'{name}：应当退 {GLOSSARY_EXIT} 并打「✗ 认不出历史节」加出路，实际退 {result.returncode}：{(result.stdout + result.stderr)[-300:]}')
        if os.path.exists(out):
            problems.append(f'{name}：认不出历史节时 --out 不该写，实际写了')
            os.remove(out)
    return problems


def migrate_twice_problems():
    """对迁过的文件拒跑：同一份文件原地迁两趟，第二趟退 4、文件逐字不变、--out 没写；迁之前的那一份换个输出重迁照常能迁（拒的不是一律拒）。"""
    problems = []
    with tempfile.TemporaryDirectory() as work:
        environment = dict(os.environ, **{LEDGER_VARIABLE: os.path.join(work, 'ledger.tsv')})
        target = os.path.join(work, '2026-09-24-样本.md')
        fresh = os.path.join(work, '2026-09-24-原样.md')
        for path in (target, fresh):
            with open(path, 'w', encoding='utf-8') as handle:
                handle.write(SAMPLE)

        def migrate(source, out):
            return subprocess.run([sys.executable, '-B', os.path.realpath(__file__), 'migrate', source, '--out', out, '--drop-history'],
                                  env=environment, capture_output=True, text=True)
        first = migrate(target, target)
        migrated = open(target, encoding='utf-8').read()
        if first.returncode != 0 or migrated == SAMPLE:
            problems.append(f'对迁过的文件拒跑：头一趟原地迁应当退 0 并改了正文，实际退 {first.returncode}：{(first.stdout + first.stderr)[-300:]}')
            return problems
        elsewhere = os.path.join(work, 'elsewhere.md')
        second = migrate(target, target)
        second_elsewhere = migrate(target, elsewhere)
        if second.returncode != REFUSED_EXIT or '✗ 拒跑' not in second.stdout or '→ 怎么办' not in second.stdout:
            problems.append(f'对迁过的文件拒跑：第二趟应当退 {REFUSED_EXIT} 并打「✗ 拒跑」加出路，实际退 {second.returncode}：{(second.stdout + second.stderr)[-300:]}')
        if open(target, encoding='utf-8').read() != migrated:
            problems.append('对迁过的文件拒跑：第二趟之后文件应当逐字不变，实际被改了')
        if second_elsewhere.returncode != REFUSED_EXIT or os.path.exists(elsewhere):
            problems.append(f'对迁过的文件拒跑：第二趟 --out 指别处也拒，且不写那个文件，实际退 {second_elsewhere.returncode}，--out 文件{"写了" if os.path.exists(elsewhere) else "没写"}')
        again = migrate(fresh, os.path.join(work, 'fresh-out.md'))
        if again.returncode != 0:
            problems.append(f'对迁过的文件拒跑：迁之前的那一份换个输出重迁应当照常退 0，实际退 {again.returncode}：{(again.stdout + again.stderr)[-300:]}')
    return problems


def selftest():
    break_mode = os.environ.get('DOC_CONSOLIDATE_BREAK', '')
    if break_mode and break_mode not in ACCEPTED_BREAK_MODES:
        print(f'✗ 自检：DOC_CONSOLIDATE_BREAK={break_mode} 认不得')
        print(f'  → 取值只有 {" ".join(ACCEPTED_BREAK_MODES)}（文件头「弄坏开关」一段）；平常跑 --selftest 不设它')
        return 2
    # 自证不读本仓那一份术语表（别的会话可能正在改它）：临时目录里放一份带「历史版本」那一行的，进程内与起的子进程都读它
    previous_glossary = os.environ.get(GLOSSARY_VARIABLE)
    with tempfile.TemporaryDirectory() as work:
        os.environ[GLOSSARY_VARIABLE] = write_glossary(work, 'glossary.md', True)
        try:
            return selftest_with_glossary(break_mode, work)
        finally:
            if previous_glossary is None:
                os.environ.pop(GLOSSARY_VARIABLE, None)
            else:
                os.environ[GLOSSARY_VARIABLE] = previous_glossary


def selftest_with_glossary(break_mode, work):
    after, report = migrate_document(SAMPLE, '2026-09-24', drop_history=True, break_mode=break_mode or None)
    problems = []
    if '（2026-09-24；报告' in after or '用户 2026-09-24 定' in after:
        problems.append('文件日期没删')
    if '（2026-09-26 现查）' in after:
        problems.append('标题日期没删')
    if 'e142-r18-2026-09-26.out' not in after or '「2026-09-26：第 4 次跑」' not in after:
        problems.append('产物名或引名里的日期被删了')
    if after.count('### 2026-09-26') != 1 or '**第二批**' not in after or '第二批做了乙' not in after:
        problems.append('同日小节没合并')
    if '## 历史版本' in after or report['history_dropped_lines'] == 0:
        problems.append('历史节没删')
    if '2026-09-24）' not in after.split('\n')[0]:
        problems.append('标题里的日期不该动')
    if '| 2026-09-24 不受影响：理由 |' not in after:
        problems.append('回看列的日期被删了（门禁 decision-links 要读它）')
    missing, history_only = loss_check_document(SAMPLE, after, '2026-09-24', dropped_history=True)
    if missing:
        problems.append(f'干净整理不该有丢失：{missing}')
    if history_only != ['2026-09-25']:
        problems.append(f'历史节独有的应只有那一节自己的日期（3 行的 3 是单位数不算），实际 {history_only}')
    caught = []
    for mode in BREAK_MODES:
        broken, _report = migrate_document(SAMPLE, '2026-09-24', drop_history=True, break_mode=mode)
        broken_missing, _ = loss_check_document(SAMPLE, broken, '2026-09-24', dropped_history=True)
        if not broken_missing:
            problems.append(f'弄坏开关 {mode}：故意丢一个数，loss-check 没红')
        else:
            caught.append(f'{mode} {broken_missing}')
    problems += migrate_twice_problems()
    problems += ['译文的历史节标题：' + problem for problem in translated_history_problems(work, break_mode if break_mode == HISTORY_HEADING_BREAK_MODE else '')]
    if not translated_history_problems(work, HISTORY_HEADING_BREAK_MODE):
        problems.append(f'弄坏开关 {HISTORY_HEADING_BREAK_MODE}：历史节标题退回只认中文，译文那一格没红')
    else:
        caught.append(f'{HISTORY_HEADING_BREAK_MODE} 红')
    problems += ['认不出历史节就退 5：' + problem for problem in glossary_missing_problems(work)]
    if problems:
        for problem in problems:
            print('✗ 自检：' + problem)
        print('  → 改 doc-consolidate.py 的对应分支，再跑 --selftest')
        return 1
    print(f'✓ 自检通过：文件日期与标题日期删了、产物名与引名不动、同日小节合并、历史节删了、零丢失，'
          f'译文 {len(TRANSLATED_HISTORY_CASES)} 份里照术语表写的历史节不整理、别的译名照正文判，术语表没有那一行或读不到退 {GLOSSARY_EXIT}，'
          f'弄坏开关 {len(caught)} 个各开一次、都被抓（{"；".join(caught)}）；'
          f'对迁过的文件拒跑：同一份文件原地迁第二趟退 {REFUSED_EXIT}、文件逐字不变、--out 没写，迁之前的那一份换个输出照常迁')
    return 0


def main():
    preflight(__file__)
    parser = argparse.ArgumentParser(description='普通 Markdown 文档的整理与丢失度核对')
    parser.add_argument('command', nargs='?', choices=['migrate', 'loss-check'])
    parser.add_argument('files', nargs='*')
    parser.add_argument('--out')
    parser.add_argument('--report')
    parser.add_argument('--drop-history', action='store_true')
    parser.add_argument('--dropped-history', action='store_true')
    parser.add_argument('--selftest', action='store_true')
    arguments = parser.parse_args()
    if arguments.selftest:
        sys.exit(selftest())
    if arguments.command == 'migrate':
        if len(arguments.files) != 1 or not arguments.out:
            print('✗ 用法：migrate <文件> --out <输出> [--drop-history]')
            print('  → 给一份输入文件与 --out')
            sys.exit(1)
        path = arguments.files[0]
        before_text = open(path, encoding='utf-8').read()
        if was_migrated(before_text):
            print(f'✗ 拒跑：{path} 是上一趟 migrate 迁出来的（正文的 sha256 在台账里），再迁一次，丢失核对拿被改过的正文当「改前」，丢掉的东西核不出来；没整理它，--out 没写，它一个字没动')
            print('  → 怎么办：要改它的内容，直接定点改（scripts/kb/replace-once.py），不再迁；真要重迁，对迁之前的那一份（git show <提交>:<路径> 取出来）重新 migrate；'
                  f'台账在 {ledger_file()}（{LEDGER_VARIABLE} 指别处，设成 off 关掉）')
            sys.exit(REFUSED_EXIT)
        file_date = file_date_of(path)
        try:
            after_text, report = migrate_document(before_text, file_date, arguments.drop_history)
        except GlossaryError as error:
            glossary_problem_exit(error)   # 认不出历史节：--out 没写、台账没记
        with open(arguments.out, 'w', encoding='utf-8') as handle:
            handle.write(after_text)
        if after_text != before_text:
            record_migrated(after_text, arguments.out)
        missing, history_only = loss_check_document(before_text, after_text, file_date, arguments.drop_history)
        summary = (f'{path}：行 {before_text.count(chr(10))} → {after_text.count(chr(10))}，同日日期删 {report["dates_stripped"]}，'
                   f'同日小节合并 {report["sections_merged"]}，历史节删 {report["history_dropped_lines"]} 行，历史节独有的数与编号 {len(history_only)} 个')
        if arguments.report:
            with open(arguments.report, 'a', encoding='utf-8') as handle:
                handle.write(f'| {path} | {before_text.count(chr(10))} | {after_text.count(chr(10))} | {report["dates_stripped"]} | {report["sections_merged"]} | '
                             f'{report["history_dropped_lines"]} | {" ".join(history_only)} | {" ".join(missing)} |\n')
        print(summary)
        if missing:
            print(f'✗ 丢失核对：找不到 {" ".join(missing)}')
            print('  → 逐个写去向：补回，或确认是有意删的过程数')
            sys.exit(3)
        print('✓ 丢失核对：正文的数、编号、产物名、引名都找得到')
        sys.exit(0)
    if arguments.command == 'loss-check':
        if len(arguments.files) != 2:
            print('✗ 用法：loss-check <改前> <改后> [--dropped-history]')
            print('  → 给两份文件')
            sys.exit(1)
        before_text = open(arguments.files[0], encoding='utf-8').read()
        after_text = open(arguments.files[1], encoding='utf-8').read()
        try:
            missing, history_only = loss_check_document(before_text, after_text, file_date_of(arguments.files[0]), arguments.dropped_history)
        except GlossaryError as error:
            glossary_problem_exit(error)
        if history_only:
            print(f'  历史节独有（在 git 里）：{" ".join(history_only)}')
        if missing:
            print(f'✗ 找不到：{" ".join(missing)}')
            print('  → 逐个写去向：补回，或确认是有意删的过程数')
            sys.exit(3)
        print('✓ 正文的数、编号、产物名、引名都找得到')
        sys.exit(0)
    parser.print_help()
    sys.exit(1)


if __name__ == '__main__':
    main()
