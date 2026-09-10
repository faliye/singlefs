#!/usr/bin/env python3
"""把 kb 里的条款整段抄进一份背景材料，抄完自己回读比对。

`.claude/rules/three-way-inference.md` 要求「引 kb 里的条目要整行抄，不许摘句」，
而实测四次摘句里有三次是**写材料的人自己**在转述时删掉了半句——手抄这一步没有任何东西盯着。
这个脚本把「抄」变成一次机械抽取，并在写出之后**回读产物、与源文件逐字节比对**，
对不上就退出码 3。

用法：

    quote-kb.py 出口.md '文件:12-30' '文件@#### 已定项 5' '文件~某个正则'

三种取法：
    文件:A-B     第 A 到第 B 行（1 起，含两端）
    文件@标题    从该标题行抄到下一个同级或更高级标题之前
    文件~正则    抄命中的那一整行（命中必须唯一）

自证会红：`quote-kb.py --selftest` 走一遍三种取法，再用 QUOTE_KB_CORRUPT=1
强制进入「抄漏一行」那条分支，确认回读比对判红（`.claude/rules/fs-design.md` 硬要求 2：
每条分支必须能被测试强制进入）。
"""
import os, re, sys, tempfile

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
        hits = [i for i, l in enumerate(L) if l.startswith(arg)]
        if len(hits) != 1:
            sys.exit(f"quote-kb: 标题「{arg}」在 {path} 命中 {len(hits)} 次，要唯一\n  → 把标题写全")
        i = hits[0]
        lvl = len(re.match(r'#*', L[i]).group(0))
        j = i + 1
        while j < len(L) and not (L[j].startswith('#') and
                                  len(re.match(r'#*', L[j]).group(0)) <= lvl):
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


def selftest():
    src = os.path.join(tempfile.mkdtemp(), 'sample.md')
    with open(src, 'w', encoding='utf-8') as f:
        f.write('# 头\n\n#### 已定项 5\n\n一\n二\n\n#### 已定项 6\n\n三\n\n'
                '#### 带代码块的\n\n```\n判定(ptr):\n    birth = ptr.birth\n```\n')
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
    print("  ✓ 自检：三种取法（含带代码块的条款）都逐字节一致，且抄漏一行时判红")
    return 0


def main(argv):
    if argv[:1] == ['--selftest']:
        return selftest()
    if len(argv) < 2:
        print(__doc__); return 2
    dest, specs = argv[0], argv[1:]
    produced = build(specs)
    bad = verify(specs, produced)
    if bad:
        print(f"quote-kb: 回读比对不通过——{bad}\n  → 别改产物，重跑一次；仍不通过就是脚本的问题",
              file=sys.stderr)
        return 3
    with open(dest, 'w', encoding='utf-8') as f:
        f.write('\n'.join(produced))
    print(f"  ✓ {len(specs)} 段整抄进 {dest}，回读逐字节一致")
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
