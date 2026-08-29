#!/usr/bin/env python3
"""本地腿输出的字词损坏检测器。

**它必须先证明有判别力**：对已知损坏的样本判红、对已知干净的样本判绿。
判据分三类，各自计数，不合并成一个分数——合并之后就分不清是哪一类了。

  U+FFFD      替换字符，编码在某一层丢过信息
  bigram 复读 连续两个相同的汉字二元组（「数据数据」「结论结论」）
  trigram 复读 连续两个相同的汉字三元组
  拼接        markdown 成对标记落了单，或标点与后词粘死——**丢字**的签名，不是复读的

⚠️ **复读与拼接是两类损坏，查法不通用。** 复读是模型多吐了，拼接是少吐了：
`keystencryption`（keystream+encryption 粘死）、`**Attack P1**:it impossible.**`
（整个从句没了）在复读检测下全是零。实测一次本地腿输出带这两种损坏，
复读检测判绿放行。⇒ 成对标记的奇偶是这一类最便宜的签名：正常散文里
反引号与 `**` 必然成对，落了单就说明中间那段文本掉了。

⚠️ **单字复读不算**（「慢慢」「刚刚」「每每」是正常中文），把它算进去会让检测器
对任何中文文本都判红，那就没有判别力了。
"""
import sys, re, unicodedata

CJK = re.compile(r'[一-鿿]')

def cjk_count(s):
    return sum(1 for ch in s if CJK.match(ch))

def ngram_repeats(text, n):
    """在**原文**里连续出现两次、且中间没有任何非汉字的 n 元组。

    ⚠️ **不许先把非汉字剥掉再找。** 剥掉之后 markdown 表格的两个单元格
    `| 缓存 | 缓存 |` 会变成「缓存缓存」，本仓自己的 kb 文档因此被判红——
    那是检测器在报自己的伪影，不是文本坏了。
    """
    pat = re.compile(r'([一-鿿]{%d})\1' % n)
    return [m.group(1) for m in pat.finditer(text)]

WORD = re.compile(r"[A-Za-z][A-Za-z'-]*")

def word_repeats(text, n=2):
    """在**原文**里连续出现两次、且中间只隔空白的 n 词组（大小写不敏感）。

    ⚠️ **英文样本必须走这条**：只数汉字 n-gram 时英文文本恒为 0，
    那是「检测器没跑到」，不是「文本干净」。

    ⚠️ **中间只许隔空白，不许隔标点。** 先把非字母剥掉再找的话，
    markdown 链接 `[invariants.md](invariants.md)` 会变成「invariants md invariants md」——
    本仓自己的 kb 因此被判出 180 处「复读」。那是检测器在报自己的伪影，不是文本坏了。
    （汉字那一侧同样的坑先修过一次，英文这侧当时漏了。）
    """
    pat = re.compile(r"\b([A-Za-z][A-Za-z'-]*(?:\s+[A-Za-z][A-Za-z'-]*){%d})\s+\1\b"
                     % (n - 1), re.I)
    return [m.group(1).lower() for m in pat.finditer(text)]

def splice_marks(text):
    """成对标记落单 + 标点粘连——**丢字**的签名。

    实测判别力：一份已知损坏的本地腿输出判红（反引号 13 奇 / `**` 13 奇 / 粘连 1），
    5 份已知干净的输出全绿（一律偶数、粘连 0）。
    """
    return {
        'odd_tick': text.count('`') % 2,
        'odd_bold': len(re.findall(r'\*\*', text)) % 2,
        # 标点直接顶着一个词，中间没空格。排掉 URL（`://`）、小数、`e.g.` 这类：
        # 要求标点**前面**不是字母数字，那才是「一句话被截断后接上了下一句」的形态。
        'glue': len(re.findall(r"(?<![A-Za-z0-9])[:;,]\w", text)),
    }

def check(text):
    fffd = text.count('�')
    c = cjk_count(text)
    bi = ngram_repeats(text, 2)
    tri = ngram_repeats(text, 3)
    words = len(WORD.findall(text))
    wbi = word_repeats(text, 2)
    return {
        'chars': len(text), 'cjk': c, 'words': words,
        'fffd': fffd, 'bigram': len(bi), 'trigram': len(tri),
        'wbigram': len(wbi), **splice_marks(text),
        'bigram_samples': bi[:6], 'trigram_samples': tri[:4],
        'wbigram_samples': wbi[:6],
    }

# 退出码：0 干净 / 1 判红 / 2 **检测器自己出错**。
# 三者必须分开——把「自己崩了」也报成 1，调用方就会把一个装不上的检测器
# 当成「每一轮都损坏」，而去调提示词，永远修不到真正的毛病。
EXIT_CLEAN, EXIT_RED, EXIT_BROKEN = 0, 1, 2

if __name__ == '__main__':
    if len(sys.argv) < 2:
        sys.stderr.write("用法: corruption-check.py <文件>...\n"); sys.exit(EXIT_BROKEN)
    bad = 0
    for path in sys.argv[1:]:
        try:
            with open(path, encoding='utf-8', errors='replace') as f:
                text = f.read()
        except OSError as e:
            sys.stderr.write("corruption-check: 读不了 %s: %s\n" % (path, e))
            sys.exit(EXIT_BROKEN)
        r = check(text)
        # 判红门槛：任何 U+FFFD，或 CJK 每千字 bigram 复读 > 1.0
        # **语料量太小就报「测不了」，不报绿。** 读不到 ≠ 读到 0。
        if r['cjk'] < 100 and r['words'] < 100:
            print(f"灰 {path}  语料太小（cjk={r['cjk']} words={r['words']}），判不了")
            continue
        rate = r['bigram'] / max(r['cjk'], 1) * 1000 if r['cjk'] >= 100 else 0.0
        wrate = r['wbigram'] / max(r['words'], 1) * 1000 if r['words'] >= 100 else 0.0
        # ⚠️ **一处命中不判红。** 复读率是个比值，样本小的时候一次偶然命中就能顶破阈值——
        # 而「that that」「had had」「the model the model」在正常英文里是合法的。
        # 实测：本地腿一次 842 词的输出里只有一处命中（1.19/千）就被判红，
        # 而已知干净的内核文档基线是 0.36/千。⇒ 判红要求**至少两处**，且比率超阈值。
        # 拼接类判红不设「至少两处」门槛：奇偶是个二值事实，不是比率，
        # 不存在「偶然落单一次」——落单就是真的掉了字。
        spliced = r['odd_tick'] or r['odd_bold'] or r['glue'] > 0
        red = (r['fffd'] > 0 or (r['bigram'] >= 2 and rate > 1.0)
               or (r['wbigram'] >= 2 and wrate > 1.0) or spliced)
        bad |= red
        print(f"{'红' if red else '绿'} {path}  cjk={r['cjk']} words={r['words']} fffd={r['fffd']} "
              f"汉字复读={r['bigram']}({rate:.2f}/千) 英文复读={r['wbigram']}({wrate:.2f}/千) "
              f"反引号落单={r['odd_tick']} 星号落单={r['odd_bold']} 粘连={r['glue']}")
        if r['bigram_samples']:
            print(f"     汉字复读样本: {' '.join(r['bigram_samples'])}")
        if r['wbigram_samples']:
            print(f"     英文复读样本: {' | '.join(r['wbigram_samples'])}")
    sys.exit(EXIT_RED if bad else EXIT_CLEAN)
