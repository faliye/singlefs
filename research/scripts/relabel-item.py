#!/usr/bin/env python3
"""一条分项翻了状态之后，把全仓对它的引用改写成今天的标签。

用法：
  relabel-item.py D19 6 [--dry-run]
  relabel-item.py --selftest

为什么要有它：分项从未定翻成已定（或反过来），22 号门禁要求全仓每一处「D<n>（简称） 未定项 k」都跟着改；
2026-09-11 D19 已定项 6 定案时这样的引用有八十多处，其中四十五处是裸写的「未定项 6」，要靠「同一行更早的 D 记号 /
文件自身的决策号 / 变更史条目标题」才归属得到——按字面替换改不全，逐处手改又会漏。
归属规则 import 自 `.claude/gate.d/lib-item-ref-status.py`，与 22 号门禁是同一份，不另抄。

改完标签，**句子本身**可能还在说它没定（「D19 已定项 6 未定」）：这类句子列成「要人看」，不替人改——
它要重写成定下来之后的说法，44 号阶段会对它判红。两份变更史与 records/ 写的是当时的状态，不列。
research/prompts/ 按证据链不改（门禁同样不扫它）。

自证会红：--selftest 在临时仓里造一条已定分项、三处旧标签引用（其中一处裸写、靠同一行更早的 D 记号归属），
改写之后拿 22 号门禁那一份库复判必须全绿、「要人看」里必须有那句「已定项 1 未定」；
再用 RELABEL_ADJACENT_ONLY=1 只改紧挨着编号的引用，裸写那一处漏改、复判变红，selftest 判红。
"""
import importlib.util
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

LIBRARY_RELATIVE_PATH = '.claude/gate.d/lib-item-ref-status.py'
HISTORY_FILES = ('decisions-history.md', 'experiments-history.md')


def load_library(root):
    spec = importlib.util.spec_from_file_location('item_ref_status', os.path.join(root, LIBRARY_RELATIVE_PATH))
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library


def relabel(root, decision, item, dry_run):
    """返回（退出码, 要人看的句子）。"""
    previous_directory = os.getcwd()
    os.chdir(root)
    try:
        library = load_library(root)
        item_map, names = library.load_map()
        if decision not in item_map or item not in item_map[decision]:
            print(f'  ✗ {decision} 正文里没有第 {item} 条分项')
            print('    → 查一下编号；分项表在那条决策正文的「### 已定项」与「### 未定项」两节')
            return 2, []
        status = item_map[decision][item]
        label = '已定项' if status == '已' else '未定项'
        adjacent_only = os.environ.get('RELABEL_ADJACENT_ONLY') == '1'
        self_map = library.self_decisions()
        rewritten_count, changed_file_count = 0, 0
        for path in library.scanned_files():
            spans_by_line = {}
            for line_number, _, match, written, owner, _, named in library.references(path, item_map, self_map):
                if owner != decision or int(match.group(2)) != item or written == status:
                    continue
                if adjacent_only and named != decision:
                    continue
                spans_by_line.setdefault(line_number, []).append((match.start(1), match.end(1)))
            if not spans_by_line:
                continue
            lines = pathlib.Path(path).read_text(encoding='utf-8').split('\n')
            count_in_file = 0
            for line_number, spans in spans_by_line.items():
                text = lines[line_number - 1]
                for start, end in sorted(spans, reverse=True):
                    text = text[:start] + label + text[end:]
                lines[line_number - 1] = text
                count_in_file += len(spans)
            rewritten_count += count_in_file
            changed_file_count += 1
            print(f'  {path}：改写 {count_in_file} 处')
            if not dry_run:
                pathlib.Path(path).write_text('\n'.join(lines), encoding='utf-8')
        review = []
        if status == '已':
            for path in library.scanned_files():
                if path.endswith(HISTORY_FILES) or '/decisions-history/' in path or path.startswith('records/'):
                    continue
                for line_number, line, match, written, owner, _, _ in library.references(path, item_map, self_map):
                    if owner == decision and int(match.group(2)) == item and written == '已' and library.says_open_after(line, match.end()):
                        review.append(f'{path}:{line_number} …{line[max(0, match.start() - 30):match.end() + 30].strip()}…')
        for entry in review:
            print(f'  要人看：{entry}')
        suffix = '（--dry-run，一个字没写）' if dry_run else ''
        print(f'✓ {decision}（{names[decision]}） 第 {item} 条是{status}定：改写 {rewritten_count} 处、{changed_file_count} 个文件{suffix}；要人看的句子 {len(review)} 处')
        return 0, review
    finally:
        os.chdir(previous_directory)


def selftest():
    real_root = pathlib.Path(__file__).resolve().parents[2]
    with tempfile.TemporaryDirectory() as root:
        (pathlib.Path(root) / '.claude/gate.d').mkdir(parents=True)
        shutil.copy(real_root / LIBRARY_RELATIVE_PATH, pathlib.Path(root) / LIBRARY_RELATIVE_PATH)
        decisions = pathlib.Path(root) / '.claude/kb/decisions'
        decisions.mkdir(parents=True)
        (decisions / '090-样本.md').write_text(
            '## D90 样本 —— 半定（一项已定，一项未定）\n\n### 已定项\n\n1. **甲问题（2026-01-02）：取甲。** **状态：已定。**\n\n'
            '### 未定项\n\n| # | 分项 | 现状 |\n|---|---|---|\n| 2 | **乙问题** | 还没选 **状态：未定。** |\n\n## 历史版本\n\n无。\n', encoding='utf-8')
        referrer = pathlib.Path(root) / '.claude/kb/引用方.md'
        referrer.write_text('D90（样本） 未定项 1 取了甲。\n先提到 D90（样本） 的背景，再说那一格的未定项 1 怎么定的。\n'
                            'D90（样本） 未定项 1 未定。\nD90（样本） 未定项 2 还开着。\n', encoding='utf-8')
        code, review = relabel(root, 'D90', 1, dry_run=False)
        text = referrer.read_text(encoding='utf-8')
        rejudged = subprocess.run([sys.executable, LIBRARY_RELATIVE_PATH], cwd=root, capture_output=True, text=True)
        full_ok = (code == 0 and rejudged.returncode == 0 and text.count('已定项 1') == 3
                   and 'D90（样本） 未定项 2 还开着' in text and len(review) == 1 and '已定项 1 未定' in review[0])
        if os.environ.get('RELABEL_ADJACENT_ONLY') == '1':
            if full_ok:
                print('selftest: 只改紧挨着编号的引用之后复判仍然全绿 —— 检查坏了'); return 1
            print('selftest: 只改紧挨着编号的引用确认判红（裸写的那一处漏改，22 号的库复判不过）'); return 0
        if not full_ok:
            print('selftest: 改写之后 22 号的库复判不过、或该改的没改全、或「要人看」没列出那句 —— 正是本脚本要防的形态')
            print(rejudged.stdout.strip()[:300]); return 1
        print('selftest: 通过（三处旧标签全改、没归属到它的那一处没动、22 号的库复判全绿、「已定项 1 未定」列进要人看）')
        return 0


def main():
    arguments = sys.argv[1:]
    if arguments[:1] == ['--selftest']:
        sys.exit(selftest())
    dry_run = '--dry-run' in arguments
    positional = [argument for argument in arguments if argument != '--dry-run']
    if len(positional) != 2 or not positional[0].startswith('D') or not positional[1].isdigit():
        print('  ✗ 用法：relabel-item.py D<n> <分项号> [--dry-run]')
        print('    → 例：relabel-item.py D19 6 --dry-run 先看会改哪几处')
        sys.exit(2)
    root = subprocess.run(['git', 'rev-parse', '--show-toplevel'], capture_output=True, text=True, check=True).stdout.strip()
    code, _ = relabel(root, positional[0], int(positional[1]), dry_run)
    sys.exit(code)


if __name__ == '__main__':
    main()
