#!/usr/bin/env python3
"""决策变更史：原文按月住 decisions-history/<年-月>.md，每条原文标题下写两行快查
（`> 快查·改前：……`、`> 快查·改后：……`；标题只有日期的再写一行 `> 快查·改了什么：……`）。
decisions-history.md 按决策汇总：每条决策的现状（取自 decisions.md 索引表）与它改过的每一次，整块由这里生成。

    python3 lib-history-brief.py check   比对（49 号阶段调它）
    python3 lib-history-brief.py write   给一行快查都没写的条目补「（待补）」，再按原文重新生成 decisions-history.md 的生成块
"""
import glob, os, re, sys

MAIN = '.claude/kb/decisions-history.md'
INDEX = '.claude/kb/decisions.md'
MONTH_DIR = '.claude/kb/decisions-history'
START = '<!-- gen:history-brief:start -->'
END = '<!-- gen:history-brief:end -->'
PENDING = '（待补）'
HISTORY_ANCHOR = '\n## 历史版本\n'
ENTRY_START = re.compile(r'(?m)^(?=### 20\d\d-\d\d-\d\d)')
HEADING = re.compile(r'### (20\d\d-\d\d-\d\d(?:（其[^）]*）)?)[：: ]*(.*)')
QUICK = re.compile(r'(?m)^> 快查·(改了什么|改前|改后)：(.*)$')
MENTION = re.compile(r'(?<![\w-])D(\d+)（')
# 快查里的数与编号必须在那一条原文里找得到：分项标签、D / E / C / I 编号、数字，按这个先后认
TOKEN = re.compile(r'(?:已定项|未定项)\s*\d+|[DECI]-?\d+(?:\.\d+)*|\d+(?:\.\d+)?')


def month_files():
    return sorted(glob.glob(f'{MONTH_DIR}/*.md'), reverse=True)


def split_month(text):
    """返回（条目之前的部分, [条目块]）。条目块从 `### 日期` 起，到下一条之前为止。"""
    anchor = text.find(HISTORY_ANCHOR)
    if anchor < 0:
        return text, []
    cut = anchor + len(HISTORY_ANCHOR)
    parts = ENTRY_START.split(text[cut:])
    return text[:cut] + parts[0], parts[1:]


def parse_block(block, month):
    match = HEADING.match(block.split('\n', 1)[0])
    return {'month': month, 'key': match.group(1), 'title': match.group(2).strip(),
            'quick': {label: value.strip() for label, value in QUICK.findall(block)},
            'source': QUICK.sub('', block)}


def load_entries():
    entries = []
    for path in month_files():
        _, blocks = split_month(open(path, encoding='utf-8').read())
        entries += [parse_block(block, os.path.basename(path)[:-3]) for block in blocks]
    return entries


def required_labels(entry):
    return (['改了什么'] if not entry['title'] else []) + ['改前', '改后']


def what_of(entry):
    return entry['title'] or entry['quick'].get('改了什么', PENDING)


def mentions(entry):
    text = ' '.join([what_of(entry), entry['quick'].get('改前', ''), entry['quick'].get('改后', '')])
    return {int(number) for number in MENTION.findall(text)}


def with_quick(block, pairs):
    """把快查几行插在标题下面。pairs: [(标签, 内容)]。"""
    heading, _, rest = block.partition('\n')
    quick = '\n>\n'.join(f'> 快查·{label}：{value}' for label, value in pairs)
    return f'{heading}\n\n{quick}\n\n{rest.lstrip(chr(10))}'


def decision_names():
    names = []
    for path in glob.glob('.claude/kb/decisions/*.md'):
        match = re.match(r'## D(\d+) (.+?) —— ', open(path, encoding='utf-8').readline())
        if match:
            names.append((int(match.group(1)), match.group(2)))
    return sorted(names)


def split_cells(line):
    return [cell.strip().replace('\\|', '|') for cell in re.split(r'(?<!\\)\|', line.strip())[1:-1]]


def index_status():
    status = {}
    if os.path.isfile(INDEX):
        for line in open(INDEX, encoding='utf-8'):
            match = re.match(r'^\| D(\d+)（', line)
            if match:
                cells = split_cells(line)
                if len(cells) >= 3:
                    status[int(match.group(1))] = (cells[1], cells[2])
    return status


def cell(text):
    return text.replace('|', '\\|').replace('\n', ' ')


# 每节只列最近这么多次：decisions-history.md 的大小只随决策条数涨，不随变更次数涨（变更次数几年下来会是上万）。
# 更早的去 decisions-history/ 看原文。
RECENT_PER_DECISION = 3


def table(entries):
    lines = ['| 改了什么 | 改前 | 改后 | 原文 |', '|---|---|---|---|']
    for entry in entries[:RECENT_PER_DECISION]:
        lines.append(f'| {cell(what_of(entry))} | {cell(entry["quick"].get("改前", PENDING))} | '
                     f'{cell(entry["quick"].get("改后", PENDING))} | [原文](decisions-history/{entry["month"]}.md) |')
    return lines


def recent_line(count):
    if count <= RECENT_PER_DECISION:
        return f'共改过 {count} 次：'
    return f'共改过 {count} 次，下面是最近 {RECENT_PER_DECISION} 次，更早的在 `decisions-history/` 的原文里：'


def render(entries):
    status = index_status()
    lines = [START, '']
    for number, name in decision_names():
        touched = [entry for entry in entries if number in mentions(entry)]
        lines += [f'## D{number}（{name}）', '']
        if number in status:
            lines += [f'**现状**：{status[number][0]}。{status[number][1]}', '']
        lines += ([recent_line(len(touched)), ''] + table(touched)) if touched else ['变更史里还没有点名它的条目。']
        lines.append('')
    others = [entry for entry in entries if not mentions(entry)]
    lines += ['## 不挂在某一条决策上的', '',
              '「改了什么」「改前」「改后」三格里都没点名哪一条决策的变更：门禁、审核、跨决策的整理这一类。', '']
    lines += ([recent_line(len(others)), ''] + table(others)) if others else ['没有。']
    lines += ['', END]
    return '\n'.join(lines)


def foreign_tokens(summary, source):
    squeezed_source = re.sub(r'\s+', '', source)
    return [token for token in TOKEN.findall(summary) if re.sub(r'\s+', '', token) not in squeezed_source]


def split_main():
    text = open(MAIN, encoding='utf-8').read()
    if text.count(START) != 1 or text.count(END) != 1:
        return None
    head, rest = text.split(START)
    block, tail = rest.split(END)
    return head, START + block + END, tail


def write():
    placeholders = 0
    for path in month_files():
        text = open(path, encoding='utf-8').read()
        prefix, blocks = split_month(text)
        new_blocks = []
        for block in blocks:
            entry = parse_block(block, '')
            if not entry['quick']:
                block = with_quick(block, [(label, PENDING) for label in required_labels(entry)])
                placeholders += 1
            new_blocks.append(block)
        new_text = prefix + ''.join(new_blocks)
        if new_text != text:
            open(path, 'w', encoding='utf-8').write(new_text)
    parts = split_main()
    if parts is None:
        print(f'  ✗ {MAIN} 里的生成块标记不是恰好一对（{START} … {END}）')  # gate-lint:summary
        print('     → 加回这一对标记再跑 --write')
        return 1
    entries = load_entries()
    open(MAIN, 'w', encoding='utf-8').write(parts[0] + render(entries) + parts[2])
    print(f'  ✓ 按原文重新生成了 {MAIN}：{len(entries)} 条条目，这次补了 {placeholders} 条「（待补）」快查')
    return 0


def check():
    if not os.path.isfile(MAIN):
        print(f'  ! 找不到 {MAIN}，本阶段跳过')
        return 0
    parts = split_main()
    if parts is None:
        print(f'  ✗ {MAIN} 里的生成块标记不是恰好一对（{START} … {END}）')  # gate-lint:summary
        print('     → 加回这一对标记，再跑 bash .claude/gate.d/49-history-brief.sh --write')
        return 1
    entries = load_entries()
    problems, checked = [], 0
    for entry in entries:
        for label in required_labels(entry):
            checked += 1
            summary = entry['quick'].get(label, '')
            if not summary or summary == PENDING:
                problems.append(f'decisions-history/{entry["month"]}.md {entry["key"]} {entry["title"][:24]}  「快查·{label}」还没写')
                continue
            foreign = foreign_tokens(summary, entry['source'])
            if foreign:
                problems.append(f'decisions-history/{entry["month"]}.md {entry["key"]}  「快查·{label}」里有原文没有的数或编号：{"、".join(foreign)}')
    if not problems and parts[1] != render(entries):
        problems.append(f'{MAIN} 的生成块与按原文重新生成的不一致')
    if problems:
        print(f'  ✗ 决策变更史的快查与原文有 {len(problems)} 处对不上')  # gate-lint:summary
        for problem in problems[:40]:
            print(f'     {problem}')
        if len(problems) > 40:
            print(f'     ……另有 {len(problems) - 40} 处')
        print('     → 每条原文标题下写两行快查：「> 快查·改前：……」「> 快查·改后：……」（标题只有日期的再写「> 快查·改了什么：……」），')
        print('       数字、编号、「已定项 k」照那一条原文抄，编号带简称；快查里冒出原文没有的数或编号，就改回原文里写着的。')
        print('     → 原文或快查改完，跑 bash .claude/gate.d/49-history-brief.sh --write 重新生成 decisions-history.md。')
        return 1
    print(f'  ✓ 决策变更史的快查与原文同步：查了 {len(entries)} 条条目、{checked} 格快查，decisions-history.md 与原文一致')
    return 0


if __name__ == '__main__':
    sys.exit(write() if len(sys.argv) > 1 and sys.argv[1] == 'write' else check())
