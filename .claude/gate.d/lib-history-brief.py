#!/usr/bin/env python3
"""决策变更史快查表：原文住 decisions-history/<年-月>.md，快查表住 decisions-history.md 的生成块里。

    python3 lib-history-brief.py check    比对（49 号阶段调它）
    python3 lib-history-brief.py write    按原文重排快查表，已经写好的改前 / 改后原样保留，新条目那一行填「（待补）」

「日期」「改了什么」两格与「按决策」那张表由原文生成；「改前」「改后」（以及标题只有日期那几条的「改了什么」）
是人或模型压的一句话，生成器保留不动，只核它们有没有写、有没有冒出原文里没有的数与编号。
"""
import glob, os, re, sys

MAIN = '.claude/kb/decisions-history.md'
MONTH_DIR = '.claude/kb/decisions-history'
START = '<!-- gen:history-brief:start -->'
END = '<!-- gen:history-brief:end -->'
PENDING = '（待补）'
ENTRY_START = re.compile(r'(?m)^(?=### 20\d\d-\d\d-\d\d)')
HEADING = re.compile(r'### (20\d\d-\d\d-\d\d(?:（其[^）]*）)?)[：: ]*(.*)')
ROW_KEY = re.compile(r'^\| (20\d\d-\d\d-\d\d(?:（其[^）]*）)?) \|')
# 摘要里的数与编号必须在原文里找得到：分项标签、D / E / C / I 编号、数字，按这个先后认
TOKEN = re.compile(r'(?:已定项|未定项)\s*\d+|[DECI]-?\d+(?:\.\d+)*|\d+(?:\.\d+)?')


def month_files():
    return sorted(glob.glob(f'{MONTH_DIR}/*.md'), reverse=True)


def load_entries():
    entries = []
    for path in month_files():
        text = open(path, encoding='utf-8').read()
        anchor = text.find('\n## 历史版本\n')
        body = text[anchor:] if anchor >= 0 else text
        for block in ENTRY_START.split(body)[1:]:
            match = HEADING.match(block.split('\n', 1)[0])
            entries.append({'month': os.path.basename(path)[:-3], 'key': match.group(1),
                            'title': match.group(2).strip(), 'text': block})
    return entries


def decision_names():
    names = []
    for path in glob.glob('.claude/kb/decisions/*.md'):
        match = re.match(r'## D(\d+) (.+?) —— ', open(path, encoding='utf-8').readline())
        if match:
            names.append((int(match.group(1)), match.group(2)))
    return sorted(names)


def split_cells(line):
    return [cell.strip().replace('\\|', '|') for cell in re.split(r'(?<!\\)\|', line.strip())[1:-1]]


def load_rows(block):
    rows = []
    for line in block.split('\n'):
        if ROW_KEY.match(line):
            cells = split_cells(line)
            if len(cells) == 4:
                rows.append({'key': cells[0], 'what': cells[1], 'before': cells[2], 'after': cells[3]})
    return rows


def row_matches(entry, row):
    return row['key'] == entry['key'] and (not entry['title'] or row['what'] == entry['title'])


def align(entries, rows, lookahead=8):
    """按顺序把原文条目与快查表的行对上（日期序号有重复，不能按钥匙查）。返回 [(条目, 行或 None)] 与对不上的多余行。"""
    pairs, orphans, cursor = [], [], 0
    for entry in entries:
        found = next((index for index in range(cursor, min(len(rows), cursor + lookahead))
                      if row_matches(entry, rows[index])), None)
        if found is None:
            pairs.append((entry, None))
        else:
            orphans.extend(rows[cursor:found])
            pairs.append((entry, rows[found]))
            cursor = found + 1
    orphans.extend(rows[cursor:])
    return pairs, orphans


def what_of(entry, row):
    return entry['title'] or (row['what'] if row else PENDING)


def cell(text):
    return text.replace('|', '\\|').replace('\n', ' ')


def render(pairs):
    lines = [START, '', '## 按决策', '',
             '每条决策在快查表里出现过几次、最近一次是哪天改了什么；按「改了什么」那一格里写着的「D<n>（简称）」数。', '',
             '| 决策 | 条目数 | 最近一次 | 最近那一次改了什么 |', '|---|---|---|---|']
    for number, name in decision_names():
        hits = [(entry, row) for entry, row in pairs if re.search(rf'(?<![\w-])D{number}（', what_of(entry, row))]
        if hits:
            entry, row = hits[0]
            lines.append(f'| D{number}（{name}） | {len(hits)} | {entry["key"]} | {cell(what_of(entry, row))} |')
        else:
            lines.append(f'| D{number}（{name}） | 0 | — | — |')
    lines += ['', '## 按日期', '']
    for month in [os.path.basename(path)[:-3] for path in month_files()]:
        lines += [f'### [{month}](decisions-history/{month}.md)', '', '| 日期 | 改了什么 | 改前 | 改后 |', '|---|---|---|---|']
        for entry, row in pairs:
            if entry['month'] == month:
                before = row['before'] if row else PENDING
                after = row['after'] if row else PENDING
                lines.append(f'| {entry["key"]} | {cell(what_of(entry, row))} | {cell(before)} | {cell(after)} |')
        lines.append('')
    lines.append(END)
    return '\n'.join(lines)


def foreign_tokens(summary, source):
    squeezed_source = re.sub(r'\s+', '', source)
    return [token for token in TOKEN.findall(summary) if re.sub(r'\s+', '', token) not in squeezed_source]


def main(mode):
    if not os.path.isfile(MAIN):
        print(f'  ! 找不到 {MAIN}，本阶段跳过')
        return 0
    text = open(MAIN, encoding='utf-8').read()
    if text.count(START) != 1 or text.count(END) != 1:
        print(f'  ✗ {MAIN} 里的生成块标记不是恰好一对（{START} … {END}）')  # gate-lint:summary
        print(f'     → 加回这一对标记，再跑 bash .claude/gate.d/49-history-brief.sh --write')
        return 1
    head, rest = text.split(START)
    old_block, tail = rest.split(END)
    entries = load_entries()
    pairs, orphans = align(entries, load_rows(old_block))
    new_block = render(pairs)
    if mode == 'write':
        open(MAIN, 'w', encoding='utf-8').write(head + new_block + tail)
        print(f'  ✓ 快查表已按原文重排：{len(entries)} 条条目，{sum(1 for _, row in pairs if row is None)} 行待补，'
              f'丢掉对不上的旧行 {len(orphans)} 行')
        return 0

    problems = []
    checked_cells = 0
    for entry, row in pairs:
        if row is None:
            problems.append(f'{entry["month"]} {entry["key"]} {entry["title"][:30]}  快查表里没有这一行')
            continue
        cells = [('改前', row['before']), ('改后', row['after'])] + ([] if entry['title'] else [('改了什么', row['what'])])
        checked_cells += len(cells)
        for label, summary in cells:
            if not summary or summary == PENDING:
                problems.append(f'{entry["key"]}  「{label}」还没写')
            else:
                foreign = foreign_tokens(summary, entry['text'])
                if foreign:
                    problems.append(f'{entry["key"]}  「{label}」里有原文没有的数或编号：{"、".join(foreign)}')
    for row in orphans:
        problems.append(f'{row["key"]} {row["what"][:30]}  快查表里多出来的一行，原文里没有对得上的条目')
    if not problems and START + old_block + END != new_block:
        problems.append('「按决策」那张表或行的写法与按原文重新生成的不一致（跑 --write 就会对齐）')
    if problems:
        print(f'  ✗ 决策变更史快查表与原文有 {len(problems)} 处对不上')  # gate-lint:summary
        for problem in problems[:40]:
            print(f'     {problem}')
        if len(problems) > 40:
            print(f'     ……另有 {len(problems) - 40} 处')
        print('     → 原文新增或改了条目：跑 bash .claude/gate.d/49-history-brief.sh --write 重排快查表，')
        print('       再把「（待补）」换成一句话；数字、编号、「已定项 k」照那一条原文抄，编号带简称。')
        print('     → 摘要里冒出原文没有的数或编号：回那一条原文核，改成原文里写着的；拿不准就不写那个数。')
        return 1
    print(f'  ✓ 决策变更史快查表与原文同步：查了 {len(entries)} 条条目、{checked_cells} 格')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else 'check'))
