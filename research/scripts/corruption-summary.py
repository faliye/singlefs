#!/usr/bin/env python3
"""把 corruption/*.raw.json 汇成一张按条件分组的表。

直接读原始响应，而不是解析日志——原始响应里有 `reasoning` 字段，
那是损坏最密的地方，而 `ask-local.sh` 把它整个丢掉，日志里看不见。

⚠️ **中文列与英文列必须分开报。** 只报汉字复读率时，英文样本的 `cjk=0`
会打印成 nan 或 0，被读成「英文更干净」——而真相是**这一列根本没测英文**。
"""
import json, glob, os, re, sys, collections

CJK = re.compile(r'[一-鿿]')
WORD = re.compile(r"[A-Za-z][A-Za-z'-]*")

def cjk_rep(t):
    return len(re.findall(r'([一-鿿]{2})\1', t))

def word_rep(t):
    w = [x.lower() for x in WORD.findall(t)]
    n, i = 0, 0
    while i + 4 <= len(w):
        if w[i:i+2] == w[i+2:i+4]: n += 1; i += 4
        else: i += 1
    return n

def main(d):
    files = sorted(glob.glob(os.path.join(d, '*.raw.json')))
    if not files:
        sys.stderr.write("corruption-summary: %s 下一个样本都没有 —— 不是「没有损坏」\n" % d)
        sys.exit(2)
    g = collections.defaultdict(list)
    for f in files:
        tag = re.sub(r'-\d+\.raw\.json$', '', os.path.basename(f))
        try:
            j = json.loads(open(f, 'rb').read().decode('utf-8'))
            m = (j.get('choices') or [{}])[0].get('message', {})
        except Exception as e:
            sys.stderr.write("corruption-summary: %s 解析失败 %s\n" % (f, e)); sys.exit(2)
        c, r = m.get('content') or '', m.get('reasoning') or ''
        g[tag].append((c, r))
    hdr = f"{'条件':<16}{'轮':>3}{'正文汉字':>9}{'正文复读':>9}{'/千字':>8}" \
          f"{'正文英词':>9}{'英词复读':>9}{'推理汉字':>9}{'推理复读':>9}{'替换字符':>9}"
    print(hdr)
    for tag, rows in g.items():
        cj = sum(sum(1 for ch in c if CJK.match(ch)) for c, _ in rows)
        cb = sum(cjk_rep(c) for c, _ in rows)
        wd = sum(len(WORD.findall(c)) for c, _ in rows)
        wb = sum(word_rep(c) for c, _ in rows)
        rj = sum(sum(1 for ch in r if CJK.match(ch)) for _, r in rows)
        rb = sum(cjk_rep(r) + word_rep(r) for _, r in rows)
        ff = sum(c.count('�') + r.count('�') for c, r in rows)
        rate = f"{cb / cj * 1000:.2f}" if cj else "未测"
        print(f"{tag:<16}{len(rows):>3}{cj:>9}{cb:>9}{rate:>8}{wd:>9}{wb:>9}{rj:>9}{rb:>9}{ff:>9}")

if __name__ == '__main__':
    main(sys.argv[1] if len(sys.argv) > 1 else 'research/results/corruption')
