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
# 行尾多一个空格的「## 历史版本 」也认（doc-lint 认它是历史节）；认不出锚的那一份由调用方判红，不当成没有条目
HISTORY_ANCHOR = re.compile(r'(?m)^## 历史版本[ \t]*(?:\n|\Z)')
ENTRY_START = re.compile(r'(?m)^(?=### 20\d\d-\d\d-\d\d)')
HEADING = re.compile(r'### (20\d\d-\d\d-\d\d(?:（其[^）]*）)?)[：: ]*(.*)')
QUICK = re.compile(r'(?m)^> 快查·(改了什么|改前|改后)：(.*)$')
MENTION = re.compile(r'(?<![\w-])D(\d+)（')
# 快查里的数与编号必须在那一条原文里找得到：分项标签、D / E / C / I 编号、数字，按这个先后认
TOKEN = re.compile(r'(?:已定项|未定项)\s*\d+|[DECI]-?\d+(?:\.\d+)*|\d+(?:\.\d+)?')


def month_files():
    return sorted(glob.glob(f'{MONTH_DIR}/*.md'), reverse=True)


def split_month(text):
    """返回（条目之前的部分, [条目块]）。条目块从 `### 日期` 起，到下一条之前为止。

    找不到「## 历史版本」返回 None：条目住在那一节下面，找不到锚就一条都读不到。
    不能当成「这一份没有条目」——那样整份的条目静默消失，49 号报「查了 0 条条目」绿着，而 48 号数得出它们。
    """
    anchor = HISTORY_ANCHOR.search(text)
    if anchor is None:
        return None
    cut = anchor.end()
    parts = ENTRY_START.split(text[cut:])
    return text[:cut] + parts[0], parts[1:]


def parse_block(block, month):
    match = HEADING.match(block.split('\n', 1)[0])
    return {'month': month, 'key': match.group(1), 'title': match.group(2).strip(),
            'quick': {label: value.strip() for label, value in QUICK.findall(block)},
            'source': QUICK.sub('', block)}


def load_entries():
    """返回（条目, 找不到「## 历史版本」的月文件）。后者非空时条目不全，调用方要判红。"""
    entries, anchorless = [], []
    for path in month_files():
        split = split_month(open(path, encoding='utf-8').read())
        if split is None:
            anchorless.append(path)
            continue
        entries += [parse_block(block, os.path.basename(path)[:-3]) for block in split[1]]
    return entries, anchorless


def report_anchorless(anchorless):
    print(f'  ✗ {len(anchorless)} 份按月的变更史里找不到「## 历史版本」，那几份的条目一条都没读到')  # gate-lint:summary
    for path in anchorless:
        print(f'     {path}')  # gate-lint:detail
    print('     → 条目住在「## 历史版本」下面：把这一行加回文件头之后、第一条 `### 日期` 之前，再跑 bash .claude/gate.d/49-history-brief.sh。')


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
    # 快查行是从 decisions-history/<月>.md 里搬到 decisions-history.md 的，两者差一层目录：
    # 源文件里的 ](../prior-art.md) 指 kb/prior-art.md，原样搬过来就变成指 .claude/prior-art.md（门禁 23 号判红）。
    # 每个以 ../ 开头的链接目标去掉一层：../prior-art.md → prior-art.md、../../rules/x.md → ../rules/x.md。
    one_level_up = text.replace('](../', '](')
    return one_level_up.replace('|', '\\|').replace('\n', ' ')


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
    return f'共改过 {count} 次，只列最近 {RECENT_PER_DECISION} 次，更早的在 `decisions-history/` 的原文里：'


class EntriesLost(Exception):
    """条目点名的决策在 decisions/ 下都没有读得出首行的正文：它既不进任何 `## D<n>` 节，也不进「不挂在某一条决策上的」。"""

    def __init__(self, lost):
        super().__init__(f'{len(lost)} 条条目没有归进任何一节')
        self.lost = lost


def render(entries):
    """收尾核「归进了某一节的条目数 == 条目总数」，对不上抛 EntriesLost。

    check 与 write 用的是同一个 render：少了这道核，漏掉的条目两边一起漏，比对照样判绿。
    """
    status = index_status()
    lines = [START, '']
    placed = set()
    for number, name in decision_names():
        touched = [entry for entry in entries if number in mentions(entry)]
        placed.update(id(entry) for entry in touched)
        lines += [f'## D{number}（{name}）', '']
        if number in status:
            lines += [f'**现状**：{status[number][0]}。{status[number][1]}', '']
        lines += ([recent_line(len(touched)), ''] + table(touched)) if touched else ['变更史里还没有点名它的条目。']
        lines.append('')
    others = [entry for entry in entries if not mentions(entry)]
    placed.update(id(entry) for entry in others)
    lines += ['## 不挂在某一条决策上的', '',
              '「改了什么」「改前」「改后」三格里都没点名哪一条决策的变更：门禁、审核、跨决策的整理这一类。', '']
    lines += ([recent_line(len(others)), ''] + table(others)) if others else ['没有。']
    lines += ['', END]
    if len(placed) != len(entries):
        raise EntriesLost([entry for entry in entries if id(entry) not in placed])
    return '\n'.join(lines)


def report_lost(lost):
    print(f'  ✗ {len(lost)} 条变更史条目没有归进生成块的任何一节：它点名的决策在 decisions/ 下都没有首行读得出的正文')  # gate-lint:summary
    for entry in lost:
        named = '、'.join(f'D{number}' for number in sorted(mentions(entry)))
        print(f'     decisions-history/{entry["month"]}.md {entry["key"]} 点名 {named}：{what_of(entry)[:30]}')  # gate-lint:detail
    print('     → 对着那一条原文核编号：快查或标题里写错了，就改成原文里写着的那条决策再跑 --write；')
    print('       编号没写错而那条决策的正文不在 decisions/ 下（首行读不出、删了或并走了），先把正文的去向查清报给主 agent——')
    print('       这类条目该归进生成块的哪一节还没有条款定，别为了转绿改历史条目。')


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
    # 条目不全（有月份找不到锚）或有条目进不了生成块时，一个字节都不写：写出去的生成块会少条目，而下一次比对照样判绿
    entries, anchorless = load_entries()
    if anchorless:
        report_anchorless(anchorless)
        return 1
    try:
        render(entries)
    except EntriesLost as error:
        report_lost(error.lost)
        return 1
    if not os.path.isfile(MAIN):
        print(f'  ✗ 找不到 {MAIN}，生成块没处写')  # gate-lint:summary
        print(f'     → 先建 {MAIN}，文件头之后放一对标记 {START} 与 {END}，再跑 --write')
        return 1
    parts = split_main()
    if parts is None:
        print(f'  ✗ {MAIN} 里的生成块标记不是恰好一对（{START} … {END}）')  # gate-lint:summary
        print('     → 加回这一对标记再跑 --write')
        return 1
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
    entries, _ = load_entries()
    open(MAIN, 'w', encoding='utf-8').write(parts[0] + render(entries) + parts[2])
    print(f'  ✓ 按原文重新生成了 {MAIN}：{len(entries)} 条条目，这次补了 {placeholders} 条「（待补）」快查')
    return 0


def check():
    if not os.path.isfile(MAIN):
        # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
        print(f'  ! 找不到 {MAIN}，本阶段无对象可判')
        return 77
    parts = split_main()
    if parts is None:
        print(f'  ✗ {MAIN} 里的生成块标记不是恰好一对（{START} … {END}）')  # gate-lint:summary
        print('     → 加回这一对标记，再跑 bash .claude/gate.d/49-history-brief.sh --write')
        return 1
    entries, anchorless = load_entries()
    failed = False
    if anchorless:
        report_anchorless(anchorless)
        failed = True
    try:
        rendered = render(entries)
    except EntriesLost as error:
        report_lost(error.lost)
        failed, rendered = True, None
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
    # 条目不全时（上面两种已经判红）拿生成块去比没有意义，比了只会多报一处
    if not problems and not failed and parts[1] != rendered:
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
        failed = True
    if failed:
        return 1
    print(f'  ✓ 决策变更史的快查与原文同步：查了 {len(entries)} 条条目、{checked} 格快查，decisions-history.md 与原文一致')
    return 0


if __name__ == '__main__':
    sys.exit(write() if len(sys.argv) > 1 and sys.argv[1] == 'write' else check())
