#!/usr/bin/env python3
"""从三方论证材料的小节清单生成 quote-kb.py 的取法，并直接交给 quote-kb.py 抽附录。

用法：
    checklist-specs.py 清单.md                                  # 只打印取法，一行一条
    checklist-specs.py 清单.md --cited 正文.md --out 出口.md [--extra 取法 …]   # 生成取法并调用 quote-kb.py
    checklist-specs.py --selftest

为什么要有它：`.claude/rules/three-way-inference.md` 要求取法「从清单里标『抄』的行自动生成，别手写第二份」，
而仓里一直没有这个脚本，D16（发布语义） 未定项 1 的三轮材料都是靠 scratchpad 里一份临时脚本生成的。
第三轮还踩了一次：把取法用不加引号的 `$SPECS` 传给 quote-kb.py，标题里的空格把一条取法拆成几段，
quote-kb.py 报「标题『###』命中 9 次」。这里用参数列表直接调 quote-kb.py，取法不经过 shell 拆词。

取法怎么定：
- 清单里标「抄」、理由不以「随」开头的行才取；
- 标题行取 `文件@标题`；引言段行（「……正文：第 a-b 行」）取 `文件:a-b`；
- 文件顶上的一、二级标题若紧跟着的引言段也标「抄」，两者合成一个 `文件:1-b`，免得 `@标题` 把整个文件抄进来；
- `--extra` 追加清单表达不了的按行补抄（例如欠账表里的某一行）。
自证会红：`--selftest` 造一份带空格标题的清单，确认生成的取法里标题是一整条；再用 CHECKLIST_SPECS_SPLIT_ON_SPACE=1
强制走回按空格拆的旧毛病，确认自检判红。
"""
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))


def read_rows(checklist_path):
    rows = []
    current_path = None
    for line in open(checklist_path, encoding='utf-8').read().split('\n'):
        match = re.match(r'### 小节清单：`([^`]+)`', line)
        if match:
            current_path = match.group(1)
            continue
        if not line.startswith('| ') or line.startswith('|---') or line.startswith('| 小节 '):
            continue
        cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
        rows.append((current_path, cells[0], cells[1] if len(cells) > 1 else '', cells[2] if len(cells) > 2 else ''))
    return rows


def specs_from_checklist(checklist_path):
    rows = read_rows(checklist_path)
    specs = []
    merged_preamble = set()
    for index, (path, title, mark, reason) in enumerate(rows):
        if mark != '抄' or reason.startswith('随') or index in merged_preamble:
            continue
        if title.startswith('#'):
            following = rows[index + 1] if index + 1 < len(rows) else None
            level = len(re.match(r'#*', title).group(0))
            if level <= 2 and following and following[0] == path and not following[1].startswith('#') and following[2] == '抄':
                line_range = re.search(r'第 (\d+)-(\d+) 行', following[1])
                specs.append(f'{path}:1-{line_range.group(2)}')
                merged_preamble.add(index + 1)
                continue
            specs.append(f'{path}@{title}')
        else:
            line_range = re.search(r'第 (\d+)-(\d+) 行', title)
            if line_range is None:
                sys.exit(f'✗ 清单里「{title}」标了抄，却既不是标题也没有「第 a-b 行」\n  → 改清单那一行，或者用 --extra 按行补抄')
            specs.append(f'{path}:{line_range.group(1)}-{line_range.group(2)}')
    if os.environ.get('CHECKLIST_SPECS_SPLIT_ON_SPACE') == '1':
        specs = [piece for spec in specs for piece in spec.split(' ')]
    return specs


def selftest():
    with tempfile.TemporaryDirectory() as directory:
        kb = os.path.join(directory, 'sample.md')
        open(kb, 'w', encoding='utf-8').write('## 样本 —— 已定\n\n开篇一句。\n\n### 已定项 1（带空格 的标题）\n\n正文。\n\n## 历史版本\n')
        checklist = os.path.join(directory, 'checklist.md')
        open(checklist, 'w', encoding='utf-8').write(
            f'### 小节清单：`{kb}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n'
            '| ## 样本 —— 已定 | 抄 | 顶 |\n| （样本 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇 |\n'
            '| ### 已定项 1（带空格 的标题） | 抄 | 要用 |\n| ## 历史版本 | 不抄 | 历史 |\n')
        specs = specs_from_checklist(checklist)
        expected = [f'{kb}:1-4', f'{kb}@### 已定项 1（带空格 的标题）']
        if specs != expected:
            print(f'  ✗ 自检：取法 {specs} ≠ 期望 {expected}')
            print('    → 带空格的标题必须是一整条取法；看 specs_from_checklist 有没有按空格拆')
            return 1
    print('  ✓ 自检通过：顶部标题与引言合成一个行区间，带空格的标题是一整条取法（查了 2 条）')
    return 0


def main():
    arguments = sys.argv[1:]
    if arguments == ['--selftest']:
        return selftest()
    if not arguments:
        sys.exit('✗ 缺清单路径\n  → checklist-specs.py 清单.md [--cited 正文.md --out 出口.md] [--extra 取法 …]')
    checklist = arguments.pop(0)
    cited = out = None
    extras = []
    while arguments:
        flag = arguments.pop(0)
        if flag == '--cited':
            cited = arguments.pop(0)
        elif flag == '--out':
            out = arguments.pop(0)
        elif flag == '--extra':
            extras.append(arguments.pop(0))
        else:
            sys.exit(f'✗ 不认识的参数 {flag}\n  → 只认 --cited、--out、--extra')
    specs = specs_from_checklist(checklist) + extras
    if out is None:
        print('\n'.join(specs))
        return 0
    command = [sys.executable, os.path.join(HERE, 'quote-kb.py'), '--checklist', checklist]
    if cited:
        command += ['--cited', cited]
    command += [out] + specs
    print(f'checklist-specs：{len(specs)} 条取法交给 quote-kb.py', file=sys.stderr)
    return subprocess.run(command).returncode


if __name__ == '__main__':
    sys.exit(main())
