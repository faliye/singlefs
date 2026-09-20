#!/usr/bin/env python3
"""决策瘦身之后核「内容没丢」：旧正文里的每个数、代码片段、编号引用，要在新正文、这次新加的变更史行、
或新正文引到的实验页里找得到。规则见 .claude/rules/format-evolution.md「决策正文只写现状，依据写成指针；决策与实验双向登记」。

用法：
    decision-slim-check.py 决策文件 [--base 提交] [--history 变更史月份文件]
        旧版取 git 里 --base（默认 HEAD）那一版，新版取工作区；--history 给了就把那个文件这次新加的行也算进「找得到」的范围
    decision-slim-check.py --selftest

另核别处的逐字引文：kb、records、.claude/rules、research/scripts 里写成「D<号>（简称）……「引文」」的，引文在旧正文里有、新正文里没有 ⇒ 报「引文断了」
（research/prompts 是冻结证据，不扫；*-history 文件、各文件「## 历史版本」之后、欠账表「### 已还清」之后记的都是当时的事，也不扫）。

只看两份的「## 历史版本」之前那一段。数按连续数字串比（日期、分项号也算，日期挪进变更史就能在那里找到）；
代码片段是反引号里的内容与代码围栏里的每一行；编号引用是 D / E / C 编号与 I-x.y 不变量号。

管不到的：句子意思变没变、限定词丢没丢——数与编号都在而那句话被改宽了，它一声不吭；这一半靠逐分项对照。

退出码：0 全找得到；2 参数或 git 出错；3 有找不到的；5 自证失败。
"""
import os
import re
import subprocess
import sys
import tempfile

E_REF = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')


def body_of(text):
    match = re.search(r'^## 历史版本\s*$', text, re.M)
    return text[:match.start()] if match else text


def tokens_of(text):
    """(种类, 片段) 的集合：数、代码片段、编号引用。"""
    found = set()
    for number in re.findall(r'\d+(?:[.,]\d+)*', text):
        found.add(('数', number))
    for span in re.findall(r'`([^`\n]+)`', text):
        found.add(('代码', span.strip()))
    for block in re.findall(r'```[^\n]*\n(.*?)```', text, re.S):
        for line in block.split('\n'):
            if line.strip():
                found.add(('代码', line.strip()))
    for ref in re.findall(r'(?<![A-Za-z0-9])(?:[DEC]\d+|I-\d+\.\d+)(?=（)', text):
        found.add(('编号', ref))
    return found


def normalized(text):
    return re.sub(r'\s+', ' ', text)


def context_of(old_text, kind, fragment):
    index = old_text.find(fragment)
    if index < 0:
        return ''
    return normalized(old_text[max(0, index - 30):index + len(fragment) + 30])


QUOTE_ROOTS = ('.claude/kb', 'records', '.claude/rules', 'research/scripts')


def broken_quotes(root, rel, number, old_body, new_body):
    """别处引这条决策的逐字引文：旧正文里有、新正文里没有的 [(文件:行, 引文)]。"""
    pattern = re.compile(r'(?<![A-Za-z0-9])D%d（[^\n]{0,40}?「([^」\n]{8,})」' % number)
    old_flat, new_flat = normalized(old_body), normalized(new_body)
    found = []
    for base in QUOTE_ROOTS:
        for folder, _, files in os.walk(os.path.join(root, base)):
            for name in files:
                if not name.endswith(('.md', '.py', '.sh')):
                    continue
                path = os.path.join(folder, name)
                if os.path.relpath(path, root) == rel or 'history' in os.path.relpath(path, root):
                    continue
                for line_number, line in enumerate(open(path, encoding='utf-8', errors='replace'), 1):
                    if re.match(r'^(## 历史版本|### 已还清)\s*$', line):
                        break
                    for match in pattern.finditer(line):
                        quote = normalized(match.group(1))
                        if quote in old_flat and quote not in new_flat:
                            found.append((f'{os.path.relpath(path, root)}:{line_number}', match.group(1)))
    return found


def git(root, *args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], cwd=root, capture_output=True, text=True)


def added_history_lines(root, base, history):
    rel = os.path.relpath(os.path.abspath(history), root)
    if git(root, 'ls-files', '--error-unmatch', rel).returncode != 0:
        return open(history, encoding='utf-8').read()
    diff = git(root, 'diff', base, '--', rel).stdout
    return '\n'.join(line[1:] for line in diff.split('\n') if line.startswith('+') and not line.startswith('+++'))


def missing_tokens(old_text, new_text, extra_texts):
    old_body, new_body = body_of(old_text), body_of(new_text)
    haystack = normalized('\n'.join([new_body, *extra_texts]))
    missing = []
    for kind, fragment in sorted(tokens_of(old_body)):
        if kind == '数':
            present = re.search(r'(?<![\d.,])' + re.escape(fragment) + r'(?![\d])', haystack) is not None
        else:
            present = normalized(fragment) in haystack
        if not present:
            missing.append((kind, fragment, context_of(old_body, kind, fragment)))
    return missing


def check(path, base, history):
    root = git(os.path.dirname(os.path.abspath(path)) or '.', 'rev-parse', '--show-toplevel').stdout.strip()
    if not root:
        print(f'  ✗ {path} 不在 git 仓里，取不到旧版')
        print('  → 怎么办：在仓里跑，或先把旧版存成文件再比。')
        return 2
    rel = os.path.relpath(os.path.abspath(path), root)
    old = git(root, 'show', f'{base}:{rel}')
    if old.returncode != 0:
        print(f'  ✗ git 里取不到 {base}:{rel}')
        print('  → 怎么办：--base 给一个有这份文件的提交。')
        return 2
    new_text = open(path, encoding='utf-8').read()
    extra = []
    if history:
        extra.append(added_history_lines(root, base, history))
    for number in sorted({int(e) for e in E_REF.findall(body_of(new_text))}):
        pages = [p for p in os.listdir(os.path.join(root, '.claude/kb/experiments')) if re.match(r'0*%d-' % number, p)]
        extra.extend(open(os.path.join(root, '.claude/kb/experiments', p), encoding='utf-8').read() for p in pages)
    missing = missing_tokens(old.stdout, new_text, extra)
    tokens_total = len(tokens_of(body_of(old.stdout)))
    title = re.search(r'^## D(\d+) ', new_text, re.M)
    quotes = broken_quotes(root, rel, int(title.group(1)), body_of(old.stdout), body_of(new_text)) if title else []
    if quotes:
        print(f'  ✗ {rel}：别处有 {len(quotes)} 处逐字引文在旧正文里有、新正文里没有：')  # gate-lint:summary
        for where, quote in quotes:
            print(f'      {where}　「{quote[:80]}」')  # gate-lint:detail
        print('  → 怎么办：引文是规则句的，把新正文那一句改回逐字一致；是历史句的，改引文那一处的措辞或指向，别让它引一句已经不在的话。')
    if missing:
        print(f'  ✗ {rel}：旧正文 {tokens_total} 个片段里有 {len(missing)} 个在新正文、这次新加的变更史行、引到的实验页里都找不到：')  # gate-lint:summary
        for kind, fragment, context in missing:
            print(f'      [{kind}] {fragment}　…{context}…')  # gate-lint:detail
        print('  → 怎么办：逐个判——该留的写回新正文；只在当天成立的挪进变更史「改前」（原句照抄）；')
        print('            推导与中间数挪到依据指到的实验页。确实该删的，在变更史里写明删了什么、为什么。')
        return 3
    if quotes:
        return 3
    print(f'  ✓ {rel}：旧正文 {tokens_total} 个片段在新正文、变更史或引到的实验页里都找得到，别处引它的逐字引文都还在')
    return 0


def selftest():
    old = ('## D1 甲 —— 已定（2026-09-01）\n\n#### 已定项 1（2026-09-01）：甲\n\n'
           '预留 64 块，式子 `a + b`，引 E5（样本）与 C7（样本欠账）。\n\n## 历史版本\n')
    new_kept = '## D1 甲 —— 已定\n\n#### 已定项 1：甲\n\n**定案**：式子 `a + b`，引 C7（样本欠账）。\n\n**依据**：E5（样本）。\n'
    history = '- **改前**：标题带 2026-09-01。\n'
    page = '预留 64 块。\n'
    lost_number = missing_tokens(old, new_kept, [history])
    if [fragment for kind, fragment, _ in lost_number] != ['64']:
        print(f'  ✗ 自证：删掉的数 64 应当恰好一个找不到，实测 {lost_number}')
        print('  → 怎么办：数的比较逻辑坏了，查 missing_tokens。')
        return 5
    if missing_tokens(old, new_kept, [history, page]):
        print('  ✗ 自证：64 在实验页里、日期在变更史里，应当全找得到')
        print('  → 怎么办：extra_texts 没算进找得到的范围，查 missing_tokens。')
        return 5
    lost_code = missing_tokens(old, new_kept.replace('`a + b`', '式子'), [history, page])
    if not any(kind == '代码' for kind, _, _ in lost_code):
        print('  ✗ 自证：删掉代码片段应当判找不到')
        print('  → 怎么办：tokens_of 没抽反引号片段。')
        return 5
    if not any(fragment == '64' for _, fragment, _ in missing_tokens(old, new_kept, ['预留 640 块'])):
        print('  ✗ 自证：640 不该算作找到了 64')
        print('  → 怎么办：数的边界匹配坏了。')
        return 5
    with tempfile.TemporaryDirectory() as root:
        os.makedirs(os.path.join(root, '.claude/kb'))
        with open(os.path.join(root, '.claude/kb/other.md'), 'w', encoding='utf-8') as page_file:
            page_file.write('正文引 D1（甲） 逐字「预留 64 块，式子」。\n\n### 已还清\n\n还清时 D1（甲） 写着「预留 64 块，式子」。\n\n## 历史版本\n\n当时 D1（甲） 写着「预留 64 块，式子」。\n')
        quotes = broken_quotes(root, '.claude/kb/decisions/01-甲.md', 1, body_of(old), body_of(new_kept))
        if [where for where, _ in quotes] != ['.claude/kb/other.md:1']:
            print(f'  ✗ 自证：正文里那处断掉的引文应当恰好报一处、已还清与历史节里那两处不报，实测 {quotes}')
            print('  → 怎么办：查 broken_quotes 的引文正则与历史节截断。')
            return 5
    print('  ✓ selftest 通过：删数判找不到、挪进变更史或实验页判找得到、删代码片段判找不到、640 不冒充 64、正文里断掉的引文报出而历史节里的不报')
    return 0


def main(argv):
    if argv[1:] == ['--selftest']:
        return selftest()
    if len(argv) < 2 or argv[1].startswith('-'):
        print('  ✗ 用法：decision-slim-check.py 决策文件 [--base 提交] [--history 变更史月份文件]')
        print('  → 怎么办：给一份 .claude/kb/decisions/ 下的决策文件；--selftest 跑自证。')
        return 2
    path, base, history = argv[1], 'HEAD', None
    rest = argv[2:]
    while rest:
        flag, value, rest = rest[0], (rest[1] if len(rest) > 1 else None), rest[2:]
        if flag == '--base' and value:
            base = value
        elif flag == '--history' and value:
            history = value
        else:
            print(f'  ✗ 认不出参数 {flag}')
            print('  → 怎么办：只认 --base 提交 与 --history 文件。')
            return 2
    return check(path, base, history)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
