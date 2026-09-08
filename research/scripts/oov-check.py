#!/usr/bin/env python3
"""本地腿输出的**拼接类**损坏检测：查生词，再判它是不是两个词粘死的。

⚠️ **这是第三类损坏，前两类查不到。** 复读查「多吐了」，成对标记奇偶查「整段掉了」，
两者对 `configurationing`（configuration+ing）、`batchinggroup`（batching+group）
一律判绿。实测：一轮被当作干净、已用于 D23 轴二论证的本地腿输出里含
`batchinggroup`×2 与 `monotfree`，三类检查只有本检查抓得到。

词表来自 Linux 内核文档（29397 词），**它不是完整英文词典**——
所以真英文词也会进生词表（实测 `factually` `misrepresents`）。
因此判红的不是「有生词」，是「生词能切成两个词表里的词」。

⚠️ **粘死的第二个词可以缺头，缺到切不开为止。** `inaccessibleisabled` 缺一个字母（规则 C 补得回），
而 `cryptographicord`（cryptographic + [Rec]ord）缺三个——尾巴只剩 `ord`，A / B / C 全切不开。
规则 D 换个方向判：成词的长前缀 + 一截既不成词也不是后缀的短尾巴。实测 2026-09-08：
在 395 份既有输出上回扫，新增判红 4 份、两个词（`cryptographicord`、`rewritingopy`），全是真损坏、零误报。

排掉常见构词前缀，否则 `mis+represents` 这类正常派生词会被误判。
"""
import sys, re, os

HERE = os.path.dirname(os.path.abspath(__file__))
WORDS_PATH = os.path.join(HERE, '..', 'data', 'en-words.txt')
# ⚠️ **不许用通用后缀表判拼接。** `contradict+ing` `batch+able` 都是正常派生，
# 拿后缀表一律判红 ⇒ 检测器对任何长英文文本都报警，判别力归零（实测两处假阳性）。
# 能分开的只有词性：`-tion` `-ment` 这类**名词后缀**接不了 `-ing`，
# 所以 `configuration+ing` 是损坏而 `contradict+ing` 不是。
NOMINAL = ('tion', 'sion', 'ment', 'ness', 'ity', 'ance', 'ence')
VERBAL = ('ing', 'ed')
# 正常派生前缀：切在这里不算拼接
PREFIX = {'mis', 'pre', 'non', 'sub', 'inter', 'over', 'under', 'multi',
          'anti', 'auto', 'semi', 'super', 'trans', 'micro', 'macro',
          # 与上面同类的英语组合前缀。加进来是因为实测把 counter+observation
          # 判成了拼接——那是模型造的复合词，不是损坏。
          'counter', 'cross', 'self', 'post', 'pseudo', 'quasi'}
# 规则 D 用：正常派生后缀。切在这里不算拼接（`distinguish+able` 是派生，不是损坏）。
SUFFIX = {'s', 'es', 'ed', 'ing', 'er', 'ers', 'est', 'ly', 'y', 'al', 'ial', 'ic',
          'ive', 'ion', 'ions', 'ity', 'ies', 'ism', 'ist', 'ous', 'ful', 'less',
          'ness', 'ment', 'ance', 'ence', 'able', 'ably', 'ible', 'ate', 'ated',
          'ize', 'ized', 'ise', 'ised', 'ish', 'like', 'wise', 'ward', 'ings', 'd', 'n',
          # 下面这批是 2026-09-08 在 395 份既有输出上回扫逼出来的：不加就是几十份误报
          'ally', 'izes', 'ises', 'r', 'rs', 'ry', 'cy', 'ent', 'ant', 'ial', 'ual'}
EXIT_CLEAN, EXIT_RED, EXIT_BROKEN = 0, 1, 2

def load_words():
    try:
        with open(WORDS_PATH, encoding='utf-8', errors='replace') as f:
            return set(f.read().split())
    except OSError as e:
        sys.stderr.write("oov-check: 读不了词表 %s: %s\n" % (WORDS_PATH, e))
        sys.exit(EXIT_BROKEN)

def splice_of(w, words):
    """两条高精度规则，任一命中即判拼接。返回切法，都不命中返回 None。

    A 两个实词粘死，**各自都要 >=5 字母**——放宽到 4 就会把 `batch+able` 判红。
    B 名词后缀之后又接了动词后缀（`configuration+ing`），英语里不存在这种构词。
    C 实词 + **缺了头一两个字母的另一个实词**。⚠️ 这条是补出来的：实测
      `inaccessibleisabled`（inaccessible + disabled 少了 d）在 A 下切不开——
      尾巴 `isabled` 不成词——于是被记成普通生词判绿，而那一轮的输出是坏的。
      模型接着吐下一个词时丢掉开头一两个字母，是拼接的一种常见形态。
    """
    lw = w.lower()
    for suf in VERBAL:                                   # 规则 B
        if lw.endswith(suf):
            stem = lw[:-len(suf)]
            if stem.endswith(NOMINAL) and stem in words:
                return "%s+%s" % (stem, suf)
    for i in range(5, len(lw) - 4):                       # 规则 A
        a, b = lw[:i], lw[i:]
        if a in PREFIX:
            continue
        if a in words and b in words:
            return "%s+%s" % (a, b)
    # 规则 C：前缀成词（>=6 字母），尾巴补一个字母成词。三道闸压假阳性——
    # 前缀 >=6、尾巴 >=5、补出来的词 >=7。⚠️ 三道都是实测逼出来的：
    # 只要求尾巴 >=4 时 `distinguish+able` 会被补成 `cable` 判红，
    # 那是正常派生词，一旦误伤检测器就没人信了。
    for i in range(len(lw) - 5, 5, -1):
        a, b = lw[:i], lw[i:]
        if len(a) < 6 or len(b) < 5 or a in PREFIX or a not in words:
            continue
        for c in 'abcdefghijklmnopqrstuvwxyz':
            cand = c + b
            if len(cand) >= 7 and cand in words:
                return "%s+(%s)%s" % (a, c, b)
    # 规则 D：成词的长前缀后面粘着一截**既不成词也不是后缀**的短尾巴。
    # ⚠️ 这条是补出来的：实测 `cryptographicord`（cryptographic + [Rec]ord，后一个词掉了三个
    # 字母）在 A / B / C 下全切不开——A 要两截各 >=5，C 只补得回一个字母——于是被记成
    # 普通生词判绿，而 `oov-check` 判绿那一份输出正被当成一条论证腿在用。
    # 三道闸压假阳性：前缀 >=8、尾巴 <=4、且尾巴既不是常见后缀也不是词表里的词
    # （后者放掉 `filesystemwide` 这类模型自造的合成词——它是造词，不是损坏）。
    # 两类词不进规则 D，回扫 395 份逼出来的：所有格（`snapshot's` 会被切成 snapshot+'s）、
    # 驼峰标识符（`InstanceId` 小写后成 instance+id）。两者都不是损坏，是正常写法。
    if "'" not in w and not any(c.isupper() for c in w[1:]):
        for i in range(len(lw) - 1, 7, -1):
            a, b = lw[:i], lw[i:]
            if len(b) > 4 or a in PREFIX or a not in words:
                continue
            if b in SUFFIX or b in words:
                continue
            return "%s+%s" % (a, b)
    return None

def scan(path, prompt_path, words):
    with open(path, encoding='utf-8', errors='replace') as f:
        txt = f.read()
    known = set()
    if prompt_path:
        with open(prompt_path, encoding='utf-8', errors='replace') as f:
            known = set(re.findall(r"[a-z][a-z']{2,}", f.read().lower()))
    oov, spliced = [], []
    for w in re.findall(r"[A-Za-z][A-Za-z']{8,}", txt):
        lw = w.lower()
        if lw in words or lw in known:
            continue
        oov.append(w)
        s = splice_of(w, words)
        if s:
            spliced.append("%s(=%s)" % (w, s))
    return oov, spliced

# 检查自己会不会红：红样本必须判红，绿样本必须判绿。
# ⚠️ **这一节是补出来的**：规则 C 加进来之前，`inaccessibleisabled` 被记成普通生词判绿，
# 而那一轮的本地腿输出是坏的、并且差点被当成证据用掉。
# 一个自己没被证明会红的检测器，与没有检测器的区别只在于它让人放心。
SELFTEST_RED = [
    'inaccessibleisabled',   # 规则 C：inaccessible + (d)isabled
    'batchinggroup',         # 规则 A：两个实词粘死
    'configurationing',      # 规则 B：名词后缀之后接动词后缀
    'cryptographicord',      # 规则 D：cryptographic + [Rec]ord，尾巴只剩三个字母
]
SELFTEST_GREEN = [
    'distinguishable',       # 曾被规则 C 误判成 distinguish+(c)able
    'indistinguishable', 'unfalsifiable', 'counterobservation',
    'misrepresents', 'factually',
    # 规则 D 的假阳性靶子：合成词与正常派生都不许判红
    # 只放真会走到 splice_of 的词（词表里没有的），否则测的不是实际会发生的事
    'filesystemwide', 'rollbacked',
]

def selftest(words):
    bad = 0
    for w in SELFTEST_RED:
        if not splice_of(w, words):
            print("  ✗ 红样本没被抓到：%s" % w); bad += 1
    for w in SELFTEST_GREEN:
        s = splice_of(w, words)
        if s:
            print("  ✗ 绿样本被误判：%s -> %s" % (w, s)); bad += 1
    if bad:
        print("  ✗ oov-check 自检未通过：%d 个样本判错" % bad)
        print("     → 怎么办： 改 splice_of 的三条规则，改完把两组样本都跑一遍；"
              "红样本抓不到说明检测器有盲区，绿样本被误判说明它会误伤正常英文。")
        return EXIT_RED
    print("  ✓ oov-check 自检通过（红样本 %d 个全抓，绿样本 %d 个不误伤）"
          % (len(SELFTEST_RED), len(SELFTEST_GREEN)))
    return EXIT_CLEAN

if __name__ == '__main__':
    if len(sys.argv) >= 2 and sys.argv[1] == '--selftest':
        sys.exit(selftest(load_words()))
    if len(sys.argv) < 2:
        sys.stderr.write("用法: oov-check.py <输出文件> [提示文件]\n"); sys.exit(EXIT_BROKEN)
    words = load_words()
    if len(words) < 5000:
        sys.stderr.write("oov-check: 词表只有 %d 词，判不了\n" % len(words)); sys.exit(EXIT_BROKEN)
    oov, spliced = scan(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None, words)
    red = len(spliced) > 0
    print("%s %s  生词=%d 拼接=%d" % ('红' if red else '绿', sys.argv[1], len(oov), len(spliced)))
    if spliced:
        print("     拼接: " + ' | '.join(spliced[:8]))
    if oov:
        # **生词表一律打印**，哪怕判绿——切不开的拼接（实测 `keystencryption`
        # `monotfree`）只有人眼能认，不打印出来就永远看不到。
        print("     生词: " + ' '.join(dict.fromkeys(oov))[:300])
    sys.exit(EXIT_RED if red else EXIT_CLEAN)
