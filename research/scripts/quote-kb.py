#!/usr/bin/env python3
"""把 kb 里的条款整段抄进一份背景材料，抄完自己回读比对。

`.claude/rules/three-way-inference.md` 要求「引 kb 里的条目要整行抄，不许摘句」，
而实测四次摘句里有三次是**写材料的人自己**在转述时删掉了半句——手抄这一步没有任何东西盯着。
这个脚本把「抄」变成一次机械抽取，并在写出之后**回读产物、与源文件逐字节比对**，
对不上就退出码 3。

用法：

    quote-kb.py 出口.md '文件:12-30' '文件@#### 已定项 5' '文件~某个正则'
    quote-kb.py --checklist 清单.md 出口.md 取法...   # 另核清单里标「抄」的每一节都真的抄进来了
    quote-kb.py --checklist 清单.md --cited 正文.md 出口.md 取法...
        # 再核正文里提到的每个 D / E / C / I 编号所在的 kb 文件都有一张小节清单（C234）

三种取法：
    文件:A-B     第 A 到第 B 行（1 起，含两端）
    文件@标题    从该标题行抄到下一个同级或更高级标题之前
    文件~正则    抄命中的那一整行（命中必须唯一）

自证会红：`quote-kb.py --selftest` 走一遍三种取法，再用 QUOTE_KB_CORRUPT=1
强制进入「抄漏一行」那条分支，确认回读比对判红（`.claude/rules/fs-design.md` 硬要求 2：
每条分支必须能被测试强制进入）。

退出码：
    3   回读比对不一致（抄出来的与源文件不一致）
    4   清单标了「抄」而附录里没有那一节（正向核对）
    5   --cited 缺清单：正文提到的某个 kb 文件，清单里没给它做一张小节清单
    6   清单标了「不抄」的小节，其实整段被 --extra 之类的行区间连带抄进了附录（反向核对，C320）
"""
import glob, os, re, sys, tempfile

FENCE = '```'


def fence_for(body):
    """围栏要比正文里最长的一串反引号还长一位。

    ⚠️ **固定三反引号会抄坏带代码块的条款**：正文里那行 ``` 会被回读当成本块的结尾，
    于是产物只回读出前两行、与源文件对不上（实测 2026-09-10，抄
    `05-快照-空间记账机制.md` 的 O(1) 判定那一节时判红）。
    判红不是误报——它确实抄坏了；坏的是围栏，不是比对。
    """
    longest = max((len(m) for l in body for m in re.findall(r'`+', l)), default=0)
    return '`' * max(3, longest + 1)


def read(path):
    with open(path, encoding='utf-8') as f:
        return f.read().split('\n')


def pick(spec):
    """→ (path, first_line_no, [lines])，行号 1 起。"""
    for sep, kind in ((':', 'range'), ('@', 'head'), ('~', 're')):
        if sep in spec:
            path, arg = spec.split(sep, 1)
            break
    else:
        sys.exit(f"quote-kb: 取法认不出：{spec}\n  → 写成 文件:A-B / 文件@标题 / 文件~正则")
    if not os.path.exists(path):
        sys.exit(f"quote-kb: 没有这个文件：{path}\n  → 用仓根相对路径")
    L = read(path)
    if kind == 'range':
        m = re.fullmatch(r'(\d+)-(\d+)', arg)
        if not m:
            sys.exit(f"quote-kb: 行区间写成 A-B：{spec}")
        a, b = int(m.group(1)), int(m.group(2))
        if not 1 <= a <= b <= len(L):
            sys.exit(f"quote-kb: {path} 只有 {len(L)} 行，取不到 {a}-{b}\n  → 先 grep -n 看行号")
        return path, a, L[a - 1:b]
    if kind == 'head':
        # 精确匹配优先，找不到再退回「前缀唯一」。只用前缀时，「### 已定项」这种
        # 裸标题会把「### 已定项 1（…）」「### 已定项 2（…）」一起算进去（实测在 D8 里
        # 命中 9 次），于是一个本来唯一的标题取不出来（2026-09-10）。
        # 凡是以前能跑通的调用，结果不变：以前能跑通 ⇒ 前缀只命中一行。
        hits = [i for i, l in enumerate(L) if l == arg]
        if len(hits) != 1:
            hits = [i for i, l in enumerate(L) if l.startswith(arg)]
        if len(hits) != 1:
            sys.exit(f"quote-kb: 标题「{arg}」在 {path} 命中 {len(hits)} 次，要唯一\n  → 把标题写全")
        i = hits[0]
        lvl = len(re.match(r'#*', L[i]).group(0))
        # ⚠️ 找节尾时要跳过代码栅栏：栅栏里一行 `# 注释` 会被当成一级标题，
        # 整节在那里被截断——而回读比对拿产物去对这同一段截断的区间，**照样逐字节一致**。
        # 实测（2026-09-10）：E131 正文的复跑命令块里有一行 `# 直接跑装置：`，
        # `@## E131…` 只取到 15 行就停了，是 `--checklist` 第一次接上时抓到的。
        j, fence = i + 1, None
        while j < len(L):
            fm = re.match(r'(`{3,}|~{3,})', L[j])
            if fence is None and fm:
                fence = (fm.group(1)[0], len(fm.group(1)))
            elif fence is not None:
                cm = re.fullmatch(r'(`{3,}|~{3,})\s*', L[j])
                if cm and cm.group(1)[0] == fence[0] and len(cm.group(1)) >= fence[1]:
                    fence = None
            elif L[j].startswith('#') and len(re.match(r'#*', L[j]).group(0)) <= lvl:
                break
            j += 1
        return path, i + 1, L[i:j]
    hits = [i for i, l in enumerate(L) if re.search(arg, l)]
    if len(hits) != 1:
        sys.exit(f"quote-kb: 正则 /{arg}/ 在 {path} 命中 {len(hits)} 次，要唯一\n  → 收窄正则")
    return path, hits[0] + 1, [L[hits[0]]]


def build(specs):
    out = []
    for spec in specs:
        path, first, body = pick(spec)
        if os.environ.get('QUOTE_KB_CORRUPT') == '1' and len(body) > 1:
            body = body[:-1]          # 测试缝：强制进入「抄漏一行」那条分支
        fence = fence_for(body)
        out.append(f"**出处 `{path}:{first}-{first + len(body) - 1}`（整段抄，未转述）**\n")
        out.append(fence + 'markdown')
        out.extend(body)
        out.append(fence)
        out.append('')
    return out


def verify(specs, produced):
    """回读产物里的每个块，与源文件逐字节比对。对不上判红。"""
    blocks, cur, close = [], None, None
    for line in produced:
        m = re.fullmatch(r'(`{3,})markdown', line)
        if cur is None and m:
            cur, close = [], m.group(1)
        elif cur is not None and line == close:
            blocks.append(cur); cur, close = None, None
        elif cur is not None:
            cur.append(line)
    if len(blocks) != len(specs):
        return f"回读到 {len(blocks)} 块，抽了 {len(specs)} 块"
    for spec, got in zip(specs, blocks):
        _, _, want = pick(spec)
        if got != want:
            return f"「{spec}」抄出来的与源文件不一致（源 {len(want)} 行，产物 {len(got)} 行）"
    return None


def normalize_heading(text):
    """去掉 `**` 与首尾空白，让清单里的标题格与产物里的标题行能互相比对。

    清单表格里的标题格偶尔会被人手写成带强调的形态，产物里的标题行是从源文件原样抄来的、
    从不带 `**`；两边都过一遍这个函数，比对就不挑这个格式差异。正向、反向核对共用它——
    两处比的是同一件事：清单上的一节标题，是不是（不是）产物里的那一行标题。
    """
    return text.replace('**', '').strip()


def check_checklist(checklist, produced):
    """清单里标「抄」的每一节，产物里必须真的有。返回缺的那几行（空表 = 对得上）。

    清单与附录是两处分别生成的（清单由 kb-sections.py 生成、人逐行填；附录由本脚本抽），
    此前没有任何东西绑住它们。实测（2026-09-10，D6 未定项 2 第二轮）：清单把 E130 的
    「臂」那一节标成「抄」，附录里一个字都没有，而那一轮的枢纽结论正建立在把其中一条臂
    读成另一个形态上（C262）。
    """
    lines = [l.replace('|', '/') for l in produced]
    heading_lines = {normalize_heading(l) for l in lines if l.strip().startswith('#')}
    sources = []
    for l in produced:
        m = re.match(r'\*\*出处 `(.+):(\d+)-(\d+)`', l)
        if m:
            sources.append((m.group(1), int(m.group(2)), int(m.group(3))))
    cur, missing = None, []
    for row in open(checklist, encoding='utf-8').read().split('\n'):
        m = re.match(r'### 小节清单：`([^`]+)`', row)
        if m:
            cur = m.group(1); continue
        if not row.startswith('| ') or row.startswith('|---'):
            continue
        cells = [c.strip() for c in row.strip().strip('|').split('|')]
        if len(cells) < 2 or cells[1] != '抄':
            continue
        title = cells[0]
        if title.startswith('#'):
            if normalize_heading(title) not in heading_lines:
                missing.append(f"{cur}：{title[:60]}")
        else:
            r = re.search(r'第 (\d+)-(\d+) 行', title)
            if r and not any(src == cur and a <= int(r.group(1)) and b >= int(r.group(2))
                             for src, a, b in sources):
                missing.append(f"{cur}：{title[:60]}")
    return missing


def heading_lines_by_file(produced):
    """{文件路径: 产物里这个文件的抄录块里出现过的每一行标题（已归一化）}。

    产物的形状是 `build()` 定的：一行「**出处 `文件:a-b`（整段抄，未转述）**」，
    紧接着一个围栏、被抄的正文、闭围栏、空行，再是下一段的出处行或结尾。
    按这个形状走一遍，就知道每一行标题原文来自哪个文件的抄录块——
    这一步是为了不让「### 已定项」这种在多个决策文件里都用的通用小标题跨文件互相误报。
    """
    by_file, current_file = {}, None
    for line in produced:
        m = re.match(r'\*\*出处 `(.+):\d+-\d+`', line)
        if m:
            current_file = m.group(1); continue
        if current_file and line.strip().startswith('#'):
            by_file.setdefault(current_file, set()).add(normalize_heading(line))
    return by_file


def check_checklist_reverse(checklist, produced):
    """反向核对：清单标「不抄」的小节，标题行不许出现在产物里同一个文件的抄录块里（C320）。

    正向核对（check_checklist）只管「标『抄』的是不是真抄了」，没人管反过来——
    清单标「不抄」的一节，如果被一个 `--extra 文件:A-B` 之类的行区间整段带进了附录，
    清单与附录就各说各的：清单说这一节没引，附录里它的原文躺在那儿。
    实测（2026-09-13，D3 未定项 8 第二轮）：`_d3-r2-checklist.md` 把 D26 已定项 3 与
    D3 已定项 9 标「不抄」，而 `_d3-r2-appendix.md` 用来带其他已定项的行区间
    （`26-后台整理与放置回收.md:44-219`、`03-空间分配.md:277-358`）恰好整段覆盖了这两条
    已定项自己的标题行，两条「不抄」的标题原样躺在附录里，没有任何东西报警。

    只查带独立标题行的「不抄」行（`#` 开头的小节）；行区间形态的「不抄」
    （比如「（… 正文：第 a-b 行）」这种引言段）没有自己的标题行可比对，跳过。
    比对按**同一个文件**收窄，不是「这个标题在附录里出现过就算」——不同决策文件常有
    同名的通用小标题（比如「### 已定项」），跨文件比对会把它们全部误判成命中。

    返回 [(文件, 标题)]，空表 = 对得上。
    """
    by_file = heading_lines_by_file(produced)
    cur, hits = None, []
    for row in open(checklist, encoding='utf-8').read().split('\n'):
        m = re.match(r'### 小节清单：`([^`]+)`', row)
        if m:
            cur = m.group(1); continue
        if not row.startswith('| ') or row.startswith('|---'):
            continue
        cells = [c.strip() for c in row.strip().strip('|').split('|')]
        if len(cells) < 2 or cells[1] != '不抄':
            continue
        title = cells[0]
        if not title.startswith('#'):
            continue
        if normalize_heading(title) in by_file.get(cur, set()):
            hits.append((cur, title))
    return hits


def cited_kb_files(text, root):
    """正文里提到的 kb 文件：显式路径，加上 D<n>（ / E<n>（ / C<n>（ / I-<n>.<m>（ 这几类编号所在的文件。

    C234（三方材料的小节清单只覆盖被判那一项所在的文件）：清单只给被判那一项所在的文件做，
    别的文件里那一半闸门问题就无声消失。2026-09-09 立账、写明了怎么拦，2026-09-11 又被打中一次
    （D19 未定项 6 第二轮材料漏了 D23 / D16 / D1 三个文件，第 6 格的全部机制住在 D23）。
    """
    found = set(re.findall(r'\.claude/kb/[^\s`)（|]+\.md', text))
    for letter, folder in (('D', 'decisions'), ('E', 'experiments')):
        for number in set(re.findall(r'(?<![A-Za-z0-9-])' + letter + r'(\d+)（', text)):
            hits = set()
            for stem in (f'{int(number):02d}-', f'{int(number)}-'):
                hits.update(glob.glob(os.path.join(root, '.claude/kb', folder, stem + '*.md')))
            if len(hits) == 1:
                found.add(os.path.relpath(hits.pop(), root))
    if re.search(r'(?<![A-Za-z0-9-])C\d+（', text):
        found.add('.claude/kb/checks-owed.md')
    if re.search(r'(?<![A-Za-z0-9-])I-\d+\.\d+（', text):
        found.add('.claude/kb/invariants.md')
    return found


def check_cited(checklist, cited_text, root):
    """正文提到的 kb 文件里，清单没有给它做一张的那几个（空表 = 对得上）。"""
    have = set(re.findall(r'### 小节清单：`([^`]+)`', open(checklist, encoding='utf-8').read()))
    return sorted(f for f in cited_kb_files(cited_text, root) if f not in have)


def selftest():
    src = os.path.join(tempfile.mkdtemp(), 'sample.md')
    with open(src, 'w', encoding='utf-8') as f:
        f.write('# 头\n\n#### 已定项 5\n\n一\n二\n\n#### 已定项 6\n\n三\n\n'
                '#### 带代码块的\n\n```\n判定(ptr):\n    birth = ptr.birth\n```\n'
                '\n### 未定项\n\n裸标题下的表\n\n### 未定项 7（带括注的同前缀标题）\n\n四\n'
                '\n#### 代码块里有井号\n\n```bash\n# 注释一行\necho hi\n```\n\n栅栏之后那一行\n')
    specs = [f'{src}:5-6', f'{src}@#### 已定项 5', f'{src}~^三$',
             f'{src}@#### 带代码块的']
    got = build(specs)
    bad = verify(specs, got)
    if bad:
        print(f"  ✗ 自检：干净那轮就对不上——{bad}"); return 1
    os.environ['QUOTE_KB_CORRUPT'] = '1'
    bad = verify(specs, build(specs))
    del os.environ['QUOTE_KB_CORRUPT']
    if not bad:
        print("  ✗ 自检：抄漏一行时回读比对**没有**判红，这道闸是摆设"); return 1
    ck = os.path.join(os.path.dirname(src), 'checklist.md')
    with open(ck, 'w', encoding='utf-8') as f:
        f.write(f"### 小节清单：`{src}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n"
                "| #### 已定项 5 | 抄 | 样本 |\n| #### 已定项 6 | 抄 | 样本 |\n"
                "| #### 带代码块的 | 不抄 | 样本 |\n")
    two = [f'{src}@#### 已定项 5', f'{src}@#### 已定项 6']
    if check_checklist(ck, build(two)):
        print("  ✗ 自检：清单与附录对得上时竟然判红"); return 1
    if not check_checklist(ck, build(two[:1])):
        print("  ✗ 自检：清单标了抄而附录里没有，比对**没有**判红（C262 那个形态）"); return 1
    if check_checklist_reverse(ck, build(two)):
        print("  ✗ 自检：清单标「不抄」的小节没被带进附录，反向核对却判红"); return 1
    leaked = check_checklist_reverse(ck, build(two + [f'{src}:8-13']))
    if not leaked:
        print("  ✗ 自检：一个行区间把「不抄」的小节标题连带抄进了附录，反向核对**没有**判红（C320）"); return 1
    if leaked != [(src, '#### 带代码块的')]:
        print(f"  ✗ 自检：反向核对命中的不是预期那一节——{leaked}"); return 1
    print("  ✓ 自检：反向核对——「不抄」的小节没被带进附录时是绿的，被行区间连带带进去时判红（C320）")
    src2 = os.path.join(os.path.dirname(src), 'sample2.md')
    with open(src2, 'w', encoding='utf-8') as f:
        f.write('# 头二\n\n#### 带代码块的\n\n另一个文件，同名标题在这里合法标了抄\n')
    with open(ck, 'a', encoding='utf-8') as f:
        f.write(f"\n### 小节清单：`{src2}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n"
                "| #### 带代码块的 | 抄 | 样本二 |\n")
    cross_file = check_checklist_reverse(ck, build([f'{src2}@#### 带代码块的']))
    if cross_file:
        print(f"  ✗ 自检：另一个文件里同名标题合法抄进来，反向核对却误报到别的文件头上——{cross_file}"); return 1
    print("  ✓ 自检：反向核对按文件收窄，不同文件里同名的通用小标题不会互相误报")
    _, _, bare = pick(f'{src}@### 未定项')
    if bare != ['### 未定项', '', '裸标题下的表', '']:
        print(f"  ✗ 自检：裸标题「### 未定项」取错了——{bare}"); return 1
    print("  ✓ 自检：三种取法（含带代码块的条款）都逐字节一致，且抄漏一行时判红")
    print("  ✓ 自检：裸标题与同前缀的带括注标题并存时，精确匹配取到裸标题那一节")
    _, _, fenced = pick(f'{src}@#### 代码块里有井号')
    if '栅栏之后那一行' not in fenced:
        print(f"  ✗ 自检：代码栅栏里的 `# 注释` 把整节截断了（只取到 {len(fenced)} 行）"); return 1
    print("  ✓ 自检：代码栅栏里的 `#` 行不被当成标题，整节取全")
    cited_root = tempfile.mkdtemp()
    os.makedirs(os.path.join(cited_root, '.claude/kb/decisions'))
    with open(os.path.join(cited_root, '.claude/kb/decisions/99-样本.md'), 'w', encoding='utf-8') as f:
        f.write('## D99 样本 —— 已定\n')
    cited_body = '这一轮要用 D99（样本） 与 C1（样本） 两处。\n'
    cited_checklist = os.path.join(cited_root, 'checklist.md')
    with open(cited_checklist, 'w', encoding='utf-8') as f:
        f.write("### 小节清单：`.claude/kb/decisions/99-样本.md`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n"
                "| ## D99 样本 —— 已定 | 不抄 | 样本 |\n")
    lacking = check_cited(cited_checklist, cited_body, cited_root)
    if lacking != ['.claude/kb/checks-owed.md']:
        print(f"  ✗ 自检：正文提到 C1 而清单里没有 checks-owed.md，--cited 应当只报它一个，实报 {lacking}"); return 1
    with open(cited_checklist, 'a', encoding='utf-8') as f:
        f.write("\n### 小节清单：`.claude/kb/checks-owed.md`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n")
    if check_cited(cited_checklist, cited_body, cited_root):
        print("  ✗ 自检：清单补齐之后 --cited 仍然报缺"); return 1
    print("  ✓ 自检：清单标「抄」而附录里没有时判红（C262）")
    return 0


def main(argv):
    if argv[:1] == ['--selftest']:
        return selftest()
    if len(argv) < 2:
        print(__doc__); return 2
    checklist = None
    cited = None
    if argv[:1] == ['--checklist']:
        if len(argv) < 4:
            print(__doc__); return 2
        checklist, argv = argv[1], argv[2:]
        if argv[:1] == ['--cited']:
            if len(argv) < 3:
                print(__doc__); return 2
            cited, argv = argv[1], argv[2:]
    dest, specs = argv[0], argv[1:]
    produced = build(specs)
    bad = verify(specs, produced)
    if bad:
        print(f"quote-kb: 回读比对不通过——{bad}\n  → 别改产物，重跑一次；仍不通过就是脚本的问题",
              file=sys.stderr)
        return 3
    if checklist:
        miss = check_checklist(checklist, produced)
        if miss:
            print(f"quote-kb: 清单标了「抄」而附录里没有 {len(miss)} 节：", file=sys.stderr)
            for m in miss:
                print(f"    {m}", file=sys.stderr)
            print("  → 把缺的那几节补进取法（优先用 文件@标题），或者把清单那一行改成「不抄」并写理由",
                  file=sys.stderr)
            return 4
        reverse_hits = check_checklist_reverse(checklist, produced)
        if reverse_hits:
            print(f"quote-kb: 清单标了「不抄」，却有 {len(reverse_hits)} 节其实被抄进了附录：",
                  file=sys.stderr)
            for f, title in reverse_hits:
                print(f"    {f}：{title[:60]}", file=sys.stderr)
            print("  → 把清单那一行改成「抄」并写理由，或者收窄带出这一节的那个 文件:A-B 行区间，别让它连带把它抄进来",
                  file=sys.stderr)
            return 6
    if cited:
        lacking = check_cited(checklist, open(cited, encoding='utf-8').read(), os.getcwd())
        if lacking:
            print(f"quote-kb: 正文提到了 {len(lacking)} 个 kb 文件，清单里没有它们的小节清单：", file=sys.stderr)
            for m in lacking:
                print(f"    {m}", file=sys.stderr)
            print("  → 用 research/scripts/kb-sections.py 给这几个文件各生成一张清单，逐行判抄不抄再重跑（C234）",
                  file=sys.stderr)
            return 5
    with open(dest, 'w', encoding='utf-8') as f:
        f.write('\n'.join(produced))
    tail = f"，清单里标「抄」的每一节都在" if checklist else ""
    if cited:
        tail += "，正文提到的每个 kb 文件都有清单"
    print(f"  ✓ {len(specs)} 段整抄进 {dest}，回读逐字节一致{tail}")
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
