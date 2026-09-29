#!/usr/bin/env python3
# admission: always 每一次调都是对此刻那份文件做一次整理或核对，上一次的结论不替这一次作保
# run-condition: command git python3
"""普通 Markdown 文档（records/、里程碑、kb 顶层与决策 / 实验页正文）的整理与丢失度核对。

变更史那两份归 history-consolidate.py；这里管别的文档，机械做的只有三类，别的一个字不动：

1. 与「所属日期」相同的行内日期删掉：所属日期是文件名里的日期（records/2026-09-24-….md）与最近一个带日期的标题；
   路径、产物名、`「…」` 里的引名、紧跟「：」当小节名用的日期不动（判法与 history-consolidate.py 同一份）。
2. 同一份文件里同一日期、同一级的标题出现第二次，第二个小节并进第一个：内容原样接在第一个小节末尾，
   第二个标题日期之后的字（「第二批」这类）留成一行粗体。
3. `--drop-history`：文末「## 历史版本」节整个删掉（只有决策与实验有历史，别的文件的历史靠 git）。

丢失度：改前正文（历史节之外）的数、编号、产物名、别的日期、带日期的引名、产物行，逐个到改后里找；找不到的逐个列出，退出码 3。
被删的历史节里独有的数与编号另列一栏「历史节独有（在 git 里）」，不算丢失。

用法：
  doc-consolidate.py migrate <文件> --out <输出> [--drop-history] [--report <报告.md>]
  doc-consolidate.py loss-check <改前> <改后> [--dropped-history]
  doc-consolidate.py --selftest
"""
import argparse
import collections
import importlib.util
import os
import re
import sys

import os as preflight_os, sys as preflight_sys  # noqa: E402
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

_SPEC = importlib.util.spec_from_file_location('history_consolidate', os.path.join(os.path.dirname(os.path.realpath(__file__)), 'history-consolidate.py'))
history = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(history)

DATE_RE = history.DATE_RE
HEADING_RE = re.compile(r'^(#{1,6})\s+(.*)$')
HISTORY_HEADING = '## 历史版本'


def split_history(lines):
    for index, line in enumerate(lines):
        if line.strip() == HISTORY_HEADING:
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


def selftest():
    after, report = migrate_document(SAMPLE, '2026-09-24', drop_history=True)
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
    broken, _report = migrate_document(SAMPLE, '2026-09-24', drop_history=True, break_mode='drop-number')
    broken_missing, _ = loss_check_document(SAMPLE, broken, '2026-09-24', dropped_history=True)
    if not broken_missing:
        problems.append('故意丢一个数，loss-check 没红')
    if problems:
        for problem in problems:
            print('✗ 自检：' + problem)
        print('  → 改 doc-consolidate.py 的对应分支，再跑 --selftest')
        return 1
    print(f'✓ 自检通过：文件日期与标题日期删了、产物名与引名不动、同日小节合并、历史节删了、零丢失，故意丢数被抓（{broken_missing}）')
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
        file_date = file_date_of(path)
        after_text, report = migrate_document(before_text, file_date, arguments.drop_history, os.environ.get('DOC_CONSOLIDATE_BREAK'))
        with open(arguments.out, 'w', encoding='utf-8') as handle:
            handle.write(after_text)
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
        missing, history_only = loss_check_document(before_text, after_text, file_date_of(arguments.files[0]), arguments.dropped_history)
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
