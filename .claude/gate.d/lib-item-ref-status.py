#!/usr/bin/env python3
"""决策分项引用的状态与正文是否一致。

每一处「D<n>（简称） 已定项 k / 未定项 k」的**前缀**都断言了那一条分项的状态。
写错了没有任何东西会发现——检索会把「已定项 5」当成已经定了的东西端出去，
而它可能还开着。本检查拿各决策正文里的分项表当权威，逐处比对。

判据：引用处写「已定项 k」而正文那条标着未定（或反过来）⇒ 判红。
归属靠同一行里最近的 D 记号；行内没有 D 记号时，用文件自身的决策号，
再退到变更史日期标题上的 D 号、变更史 `## D<n>（简称）` 节标题的 D 号，最后是该行之前最近出现过的 D 记号——各级都判不出的也判红，因为
一个归属判不出的编号引用，读的人同样判不出。

**归属规则只有这一份**：门禁 doc-decisions（`doc-decisions.sh`）的格「分项引用状态」跑这里的 main，
格「引用写已定紧跟说没定」（引用写着已定项、紧跟着却说它没定）、格「冻结层归属登记表」（查依据点名的分项是不是已定）与
`scripts/kb/relabel-item.py`（分项翻状态之后改写全仓引用）都 import 这里的函数，
不各抄一份——两份归属规则一旦分叉，工具改写的与门禁判的就不是同一批引用。

**按编号取哪一份文件也只有这一份**（numbered_files、heads_for_referrer）：decisions/ 与 experiments/ 下一个编号有三份——
中文原文 `<号>-<名>.md`、译文 `<号>-<名>.en.md` / `<号>-<名>.ja.md`（登记在 .claude/trilingual-documents）。
引用方是中文（不带 `.<两个小写字母>.md` 后缀的文件，含代码、产物与登记表；不给引用方也按中文）取中文原文；
引用方是译文取同一种语言的那一份，没有就取中文原文（.claude/skills/trilingual/SKILL.md「改一组的做法」第 5 条：链接可指中文那份或同语言那份）。
译文后缀的认法与 lib-trilingual.py 的 LANGUAGE_SUFFIXED 同口径。门禁 doc-registries（field-refs、format-const-placeholders、audit-contradictions）、
doc-decisions（freeze-layer-membership、write-frequency）、doc-experiments（index-sync、verdict-false-named）按编号取文件都调这里。
**数一遍、按语言分组也只有这一份**（same_language_files、files_by_language）：条数与分项数只数引用方那一种语言的文件——
门禁 doc-registries（declared-counts、invariant-count-elsewhere）的不变量在用条数只数中文原文，
doc-decisions（kb-shape 第 7 段）中文索引表的分项计数只对中文原文；「编号从 D1 连号」这类判据按语言各判各的（kb-shape 第 5 段：中文一组、每种译文一组），
不把三语三份的号并成一个列表。
弄坏开关 KB_NUMBERED_FILE_BREAK=first-match：不分语言，按路径排序交回全部候选（调用方取第一个的取到排在最前的译文 `.en.md`，
求并集的连译文一起并进来；heads_for_referrer 不筛语言；same_language_files 原样交回全部、files_by_language 并成一组）。
打开后判错的样本逐道写在三道门禁文件头的弄坏开关一节。

**决策表只读中文原文**（decision_heads、self_decisions）：决策号、简称与分项表都只从中文原文 `<号>-<名>.md` 读，译文（带 `.<两个小写字母>.md` 后缀）
不进决策表，首行写成什么（`—` 单破折号、Settled / 確定 这类状态词）这里都不判、不报读不出；译文的分项引用归属落在同号的中文原文那条决策上。
改前译文与原文一起读：译文首行与中文写法不同就整格报「首行读不出」红，读得出时同号的分项表靠文件名排序让中文那份最后覆盖。
弄坏开关 KB_DECISION_HEADS_BREAK=read-translations：译文又进决策表（改前的口径；KB_NUMBERED_FILE_BREAK=first-match 下同样连译文一起读）。
样本：门禁 doc-decisions 的 item-ref-status-translation-first-line-green。

**孤立译文不顶替原文**（is_orphan_translation、orphan_translations）：decisions/ 下带语言后缀的译文，同目录没有同名的中文原文（`<号>-<名>.md`），
就是孤立译文。它不进决策表（decision_heads 不收、load_map 不读）、不参与按编号取文件、不当引用方的归属，也不被扫引用；
main 把它单独报一类红，出路是补中文原文或删掉这份译文。改前它的首行读得出时，同一个决策号的分项表被它整张换掉，
中文引用方写的「D1（数据可移动性 / 反向索引） 已定项 4」被解成「D1（甲（訳））」而报「没有第 4 条」。
弄坏开关 KB_ORPHAN_TRANSLATION_BREAK=substitute：孤立译文又进决策表、顶替同号的中文原文，也不单独报（改前的口径）。

**变更史的节标题**（references）：`decisions-history.md` 一条决策一节，节标题写成 `## D<n>（简称）`，节里 `### <日期>` 的条目带不带决策号的「已定项 k / 未定项 k」都归这一节的决策。
归属顺序：行内紧挨着的指名、行内最近的 D 记号、文件自身的决策号、日期标题上的 D 号、节标题的 D 号、之前出现过的最近的 D 记号。
弄坏开关 KB_HISTORY_SECTION_OWNER_BREAK=nearest-d：不认 `## D<n>（` 节标题、每个 `### ` 标题都清掉标题上的归属（改前的口径），
节里裸写的分项退回「往前找最近出现过的 D 号」，归错决策或报归属判不出。
"""
import re, glob, sys, os, json
from functools import lru_cache

LANGUAGE_SUFFIXED = re.compile(r'\.([a-z]{2})\.md$')
NUMBERED_FILE_BREAK = os.environ.get('KB_NUMBERED_FILE_BREAK', '')
ORPHAN_TRANSLATION_BREAK = os.environ.get('KB_ORPHAN_TRANSLATION_BREAK', '')
HISTORY_SECTION_OWNER_BREAK = os.environ.get('KB_HISTORY_SECTION_OWNER_BREAK', '')
DECISION_HEADS_BREAK = os.environ.get('KB_DECISION_HEADS_BREAK', '')
DECISIONS_DIRECTORY = '.claude/kb/decisions'


@lru_cache(maxsize=1)
def translation_paths():
    path = '.claude/kb/translation-paths.json'
    if not os.path.isfile(path):
        return {}
    try:
        with open(path, encoding='utf-8') as handle:
            return json.load(handle)
    except Exception:
        return {}


def language_of(path):
    """'' 是中文原文（也是代码、产物、登记表这类不带语言后缀的文件），否则是两个字母的语言代码。"""
    hit = LANGUAGE_SUFFIXED.search(os.path.basename(path or ''))
    return hit.group(1) if hit else ''


def is_orphan_translation(path):
    """带编号的译文（`<号>-<名>.<语言>.md`），同目录没有同名的中文原文 `<号>-<名>.md`。
    若在 .claude/kb/translation-paths.json 登记过映射，以映射的中文原名判断。
    弄坏开关 KB_ORPHAN_TRANSLATION_BREAK=substitute 与 KB_NUMBERED_FILE_BREAK=first-match（改前的口径，不区分孤立与否）下一份都不算。"""
    if ORPHAN_TRANSLATION_BREAK == 'substitute' or NUMBERED_FILE_BREAK == 'first-match':
        return False
    name = os.path.basename(path or '')
    if not language_of(name) or not re.match(r'^\d+-', name):
        return False
    direct = os.path.join(os.path.dirname(path), LANGUAGE_SUFFIXED.sub('.md', name))
    if os.path.isfile(direct):
        return False
    tp = translation_paths()
    norm = os.path.normpath(path)
    for orig, renamed in tp.items():
        if os.path.normpath(renamed) == norm or os.path.basename(renamed) == name:
            zh = LANGUAGE_SUFFIXED.sub('.md', orig)
            if os.path.isfile(zh):
                return False
    return True


def orphan_translations(directory=None):
    """directory（不给取 decisions/）下的孤立译文，排序后的路径列表；main 单独报一类红。弄坏开关下恒空。"""
    directory = directory or DECISIONS_DIRECTORY
    try:
        names = os.listdir(directory)
    except OSError:
        return []
    return sorted(path for path in (os.path.join(directory, name) for name in names)
                  if os.path.isfile(path) and is_orphan_translation(path))


def numbered_files(directory, number, referrer=None):
    """directory 下编号是 number 的那一份（文件名 `<号>-*.md`，号前可补 0）→ 排序后的路径列表（通常一份，空表示没有）。

    引用方 referrer 是中文取中文原文；是译文取同一种语言的那一份，没有就取中文原文。孤立译文（没有同名中文原文的）一份都不交回。
    弄坏开关 KB_NUMBERED_FILE_BREAK=first-match 下交回全部候选（含孤立译文）。"""
    pattern = re.compile(r'^0*%d-.*\.md$' % int(number))
    try:
        names = os.listdir(directory)
    except OSError:
        return []
    candidates = sorted(os.path.join(directory, name) for name in names
                        if pattern.match(name) and os.path.isfile(os.path.join(directory, name))
                        and not is_orphan_translation(os.path.join(directory, name)))
    if NUMBERED_FILE_BREAK == 'first-match':
        return candidates
    language = language_of(referrer)
    same_language = [path for path in candidates if language_of(path) == language]
    if same_language or not language:
        return same_language
    return [path for path in candidates if language_of(path) == '']


def heads_for_referrer(heads, unreadable, referrer=None):
    """decision_heads 的结果里，只留引用方 referrer 按编号取得到的那几份（中文引用方只留中文原文，译文引用方留同语言的、缺的那几个号补中文原文）。

    首行读不出的那几份同样筛：中文引用方只关心中文原文读不读得出（decision_heads 平时只读中文原文，译文只在弄坏开关下进来）。
    孤立译文（没有同名中文原文的）两张表里都不留。弄坏开关 KB_NUMBERED_FILE_BREAK=first-match 下原样交回。"""
    if NUMBERED_FILE_BREAK == 'first-match':
        return heads, unreadable
    language = language_of(referrer)
    if not language:
        return ({path: head for path, head in heads.items() if language_of(path) == ''},
                [(path, first) for path, first in unreadable if language_of(path) == ''])
    kept = {path: head for path, head in heads.items() if language_of(path) == language and not is_orphan_translation(path)}
    numbers = {decision for decision, _ in kept.values()}
    kept.update({path: head for path, head in heads.items() if language_of(path) == '' and head[0] not in numbers})
    return kept, [(path, first) for path, first in unreadable
                  if language_of(path) in ('', language) and not is_orphan_translation(path)]


def same_language_files(paths, referrer=None):
    """paths 里与引用方 referrer 同一种语言的那几份，次序不变（中文引用方只留中文原文 `<名>.md`，译文引用方只留同语言的 `<名>.<语言>.md`）。

    数条数、数分项这类「数一遍」的量用它：译文是同一份内容的另一种语言，数进去就把一条算成三条。弄坏开关下原样交回全部。"""
    if NUMBERED_FILE_BREAK == 'first-match':
        return list(paths)
    language = language_of(referrer)
    return [path for path in paths if language_of(path) == language]


def originals_only(paths, read_translations=False):
    """paths 里的中文原文（不带 `.<两个小写字母>.md` 后缀的），次序不变；read_translations 为真时原样交回全部。

    整张表、整段正文只该有一处权威的判据用它（锚与标记全仓只许一处、登记值、字段表加和、决策与实验页的形状）：译文照抄了锚与标记、
    首行与小节标题是各自的语言，读进来就多出几处权威、或把译文的标题判成「认不出」。各门禁把自己的弄坏开关 read-translations 交给 read_translations。"""
    if read_translations:
        return list(paths)
    return [path for path in paths if not language_of(path)]


def files_by_language(paths):
    """按语言分组：{'': 中文原文, 'en': 英语译文, …}，组内次序不变；「编号连续」这类判据一组里各判各的。弄坏开关下全部并成一组（键 ''）。"""
    if NUMBERED_FILE_BREAK == 'first-match':
        return {'': list(paths)} if paths else {}
    groups = {}
    for path in paths:
        groups.setdefault(language_of(path), []).append(path)
    return groups


def decision_heads(strict=True):
    """decisions/ 下每一份中文原文首行的 ({路径: (决策号, 简称)}, [(首行读不出的路径, 首行)])。

    译文不读（文件头「决策表只读中文原文」；弄坏开关 KB_DECISION_HEADS_BREAK=read-translations 与 KB_NUMBERED_FILE_BREAK=first-match 下连译文一起读）。
    孤立译文（没有同名中文原文的 `<号>-<名>.<语言>.md`）一份都不收，两张表里都没有它。
    首行读不出 `## D<n> 简称 —— 状态` 的（多一份 README、首行前有空行或 BOM、漏了破折号）一份都不跳过，
    也不让调用方撞上裸的 AttributeError：strict 时当场打印 ✗ 与出路、退出码 1；
    不 strict 时交回清单，由调用方在报完自己的问题之后调 report_unreadable 并判红。
    跳过它的代价实测过：那条决策的分项从表里消失，写给它的「已定项 k」被改判到行内更早出现的另一条决策上并判绿。
    load_map 与 self_decisions 都从这里取，两处的口径不许分叉（写法照门禁 doc-decisions INDEX_VS_BODY_SOURCE 读首行那一段）。
    """
    heads, unreadable = {}, []
    read_translations = DECISION_HEADS_BREAK == 'read-translations' or NUMBERED_FILE_BREAK == 'first-match'
    for f in sorted(glob.glob('.claude/kb/decisions/*.md')):
        if is_orphan_translation(f):
            continue   # 孤立译文不进决策表、首行读不读得出也不在这里报：main 单独报一类（orphan_translations）
        if language_of(f) and not read_translations:
            continue   # 译文不进决策表：决策号、简称、分项表只认中文原文（文件头「决策表只读中文原文」）
        first = open(f, encoding='utf-8').read().split('\n', 1)[0]
        mm = re.match(r'## (D\d+) (.+?)\s*——', first)
        if mm:
            heads[f] = (mm.group(1), mm.group(2).strip())
        else:
            unreadable.append((f, first))
    if unreadable and strict:
        report_unreadable(unreadable)
        sys.exit(1)
    return heads, unreadable


def report_unreadable(unreadable):
    print(f"  ✗ decisions/ 下有 {len(unreadable)} 份首行读不出 `## D<n> 简称 —— 状态`，这几份的分项一条都没核过")  # gate-lint:summary
    for f, first in unreadable:
        print(f"     {f}：首行是 {first[:40]!r}")  # gate-lint:detail
    print("     → 怎么办：把首行写成 `## D3 空间分配 —— 半定（…）`，首行之前不许有空行或 BOM；")
    print("       decisions/ 下只放决策正文，说明性的文字写进 decisions.md。读不出来就是这一份没核过，不许当成通过。")


def report_orphan_translations(orphans):
    print(f"  ✗ decisions/ 下有 {len(orphans)} 份孤立译文（带语言后缀、同目录没有同名的中文原文），它们不解任何引用、里面的引用也没扫")  # gate-lint:summary
    for f in orphans:
        print(f"     {f}：没有 {LANGUAGE_SUFFIXED.sub('.md', os.path.basename(f))}")  # gate-lint:detail
    print("     → 怎么办：补上同名的中文原文，或删掉这份译文；中文引用方只认中文原文，译文只在有同名原文时才算那条决策的另一种语言。")


def load_map(heads=None):
    """heads 不给就现读，首行读不出的当场判红退出（decision_heads 的 strict）。"""
    m, name = {}, {}
    for f, (d, short_name) in sorted((heads if heads is not None else decision_heads()[0]).items()):
        s = open(f, encoding='utf-8').read()
        name[d] = short_name; m[d] = {}
        for head, st in (('已定项', '已'), ('未定项', '未')):
            sec = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, s, re.M | re.S)
            if not sec: continue
            seg = sec.group(1)
            cut = re.search(r'^#{4}\s', seg, re.M)
            top = seg[:cut.start()] if cut else seg
            # ⚠️ 正则要与 `.claude/scripts/gen-decision-items.py` 逐字同口径：
            # 那边要求表格行有**闭合的第二根竖线**。两边不一致时，一行畸形表格
            # 会「一边看得见、一边看不见」，而两边都不报错（2026-08-30 对抗验证指出）。
            for n in re.findall(r'^\|\s*(\d+)\s*\|[^|]*\|', top, re.M) or re.findall(r'^(\d+)\.\s', top, re.M):
                m[d][int(n)] = st
    return m, name


def candidate_files():
    """扫描射程里的全部文件，含下面要排除的 research/prompts/。"""
    return sorted(set(sum([glob.glob(p, recursive=True) for p in
        ('.claude/kb/**/*.md', 'records/**/*.md', 'research/**/*.md',
         'research/**/*.rs', 'crates/**/*.rs', '.claude/rules/*.md')], [])))


def is_excluded_prompt_file(path):
    # research/prompts/ 显式排除，理由是 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」
    # （`.claude/doc-lint-exclude` 排除它也是这一条）：那是原样发给模型的提示与模型的原样输出，
    # 与 research/results/ 里的产物一一对应，事后改它等于让产物对不上输入。
    # 实测（2026-09-03）：反推腿的输出里有一条复现命令 `grep -n "已定项 8" …`，
    # 没有决策号可归属，按正文规则判红，而那一行按证据链不许改。
    return '/prompts/' in path


def scanned_files(candidates=None):
    # 孤立译文也不扫：它没有归属可言（不在决策表里），里面的裸引用会再报一遍「归属判不出」，盖住真正的出路（补原文或删译文，main 单独报那一类）
    return [f for f in (candidates if candidates is not None else candidate_files())
            if not is_excluded_prompt_file(f) and not is_orphan_translation(f)]


def self_decisions(heads=None):
    """{决策文件: 它自身的决策号}；译文（不在决策表里）取同目录同名中文原文的决策号，里面裸写的分项照样归到这条决策。"""
    own = {f: d for f, (d, _) in (heads if heads is not None else decision_heads()[0]).items()}
    for f in sorted(glob.glob('.claude/kb/decisions/*.md')):
        original = os.path.join(os.path.dirname(f), LANGUAGE_SUFFIXED.sub('.md', os.path.basename(f)))
        if f not in own and language_of(f) and original in own:
            own[f] = own[original]
    return own


# 紧挨着分项引用的「D<n>（简称）」：简称里许套一层括号（「D14（双轨（大小文件 / 持久临时））」），
# 不许的话指名认不出来，引用就落回文件自身的决策上判（门禁审核 2026-09-26 那一格）。
ADJACENT_NAMED_DECISION = re.compile(r'D(\d+)\s*(?:（(?:[^（）]|（[^（）]*）)*）)?\s*[*`]*\s*$')


def references(path, item_map, self_map, skipped_fenced_lines=None):
    """逐处给出一个分项引用：(行号, 行, 匹配, 写的状态, 归属或 None, 指名却没有第 k 条的那条决策或 None, 紧挨着指名的决策或 None)。

    skipped_fenced_lines 给一个 list 时，代码围栏里跳过没判的行号逐个追加进去（成功行报「没查的」用）。
    """
    self_d = self_map.get(path); last = None; hist = None; section = None; in_fence = False
    lines = open(path, encoding='utf-8').read().split('\n')
    fence_line_numbers = [ln for ln, line in enumerate(lines, 1) if line.lstrip().startswith('```')]
    # 围栏行是奇数个：最后一个没闭合，不当围栏——按开关切的话，它之后的半份文件整段不判。
    unclosed_fence_line_number = fence_line_numbers[-1] if len(fence_line_numbers) % 2 else None
    for ln, line in enumerate(lines, 1):
        # 代码围栏里是逐字照抄的东西——变更史的「改前」原行、实验的产物行、源码片段。
        # 按今天的状态判它们，等于要求「原样保存的证据」跟着现状改
        # （`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」）。
        # 实测（2026-09-20）：第三批瘦身把 D17（实现分层与第三方管道） 的旧正文抄进变更史围栏之后，
        # 围栏里两处「未定项 6」当场判红——那两行说的是抄下来那天的状态，而立项让第 6 条变成了已定。
        if line.lstrip().startswith('```') and ln != unclosed_fence_line_number:
            in_fence = not in_fence
            continue
        if in_fence:
            if skipped_fenced_lines is not None:
                skipped_fenced_lines.append(ln)
            continue
        if path.endswith('decisions-history.md') or '/decisions-history/' in path:
            # 变更史一条决策一节：`## D<n>（简称）` 是节标题，节里 `### <日期>` 的条目裸写的分项归这一节的决策；
            # 日期标题自己带 D 号的（`### … D14 …`）归那一条，不带的不清掉节标题的归属。
            # 弄坏开关 nearest-d：不认节标题，每个 `### ` 标题都把归属清成标题上的 D 号或空（改前的口径）。
            if line.startswith('## ') and HISTORY_SECTION_OWNER_BREAK != 'nearest-d':
                mm = re.match(r'## D(\d+)', line); section = 'D' + mm.group(1) if mm else None; hist = None
            elif line.startswith('### '):
                mm = re.search(r'D(\d+)', line); hist = 'D' + mm.group(1) if mm else None
        for m in re.finditer(r'(已定项|未定项)\s*(\d+)', line):
            k = int(m.group(2)); want = '已' if m.group(1) == '已定项' else '未'
            pre = line[:m.start()]; ds = re.findall(r'(?<![0-9A-Za-z_])[Dd](\d+)', pre)
            # ⚠️ 只有**紧挨着**分项引用的那个编号才算「指名」（仓里规范的引用形态是
            # `D6（快照实现模型） 已定项 2`）。行内更早提到的编号不算：同一行里裸写的
            # 「已定项 k」按文件自身的决策号解析，那是下面三级回退存在的理由。
            adj = ADJACENT_NAMED_DECISION.search(pre)
            named = ('D' + adj.group(1)) if adj else None
            # 指名道姓的那条决策，正文里没有第 k 条 ⇒ 当场判红，不许往下落到别的决策。
            # 少了这一句，一个不存在的分项号会被静默改判到「刚好有第 k 条」的另一条决策
            # 头上并判绿：2026-09-06 起 decisions-history 里的「D6 已定项 2」就这样绿了四天，
            # 而 D6 从头到尾只有 1 个分项，同一个号在两份腿输出里还指着两样东西。
            if named in item_map and k not in item_map[named]:
                yield ln, line, m, want, None, named, named
                continue
            owner = next((c for c in (named, 'D' + ds[-1] if ds else None, self_d, hist, section, last)
                          if c in item_map and k in item_map[c]), None)
            yield ln, line, m, want, owner, None, named
        for d in re.findall(r'(?<![0-9A-Za-z_])[Dd](\d+)', line):
            if 'D' + d in item_map: last = 'D' + d


# 「已定项 k」后面紧跟着说它没定：中间只许有一段括注、标点与「今天 / 仍 / 还」这类词，
# 而且那个词之后句子就断了（标点、括号或行尾）。⚠️ 不收这一尾，「D5 已定项 4 没定的东西」
# 「D2 已定项 6 还没定判据」会被误判——那里的「没定」修饰后面的名词，说的是「那条定了、但没定到这一样」
# （实测：第一版在实验源码的注释里报了这两处）。两头都认 markdown 的 `**` 与反引号：「**D27 已定项 5 未定**」第一版就漏了。
# 门禁 doc-decisions 的格「引用写已定紧跟说没定」判红用它，relabel-item.py 列「要人看的句子」也用它。
# 「这类词」可以连着几个（「今天仍未定」「目前都还没定」），每个后面许跟空白与顿号逗号。
OPEN_AFTER = re.compile(r'^(?:（[^）]*）)?[\s、，,：:*`]*(?:(?:今天|目前|仍然|仍|尚|还|都|均)[\s、，,]*)*(?:未定(?!项)|没定|定不下|待定|空着)'
                        r'(?=$|[\s。，、；：:;,.)）」』（(！？!?*`])')


def says_open_after(line, end):
    return OPEN_AFTER.match(line[end:]) is not None


def main():
    heads, unreadable = decision_heads(strict=False)
    item_map, names = load_map(heads)
    self_map = self_decisions(heads)
    candidates = candidate_files()
    files = scanned_files(candidates)
    excluded_prompt_files = [f for f in candidates if is_excluded_prompt_file(f)]
    bad = []; refs = 0; skipped_fenced_lines_by_file = {}
    for path in files:
        skipped_fenced_lines = []
        for ln, line, m, want, owner, missing, _ in references(path, item_map, self_map, skipped_fenced_lines):
            refs += 1
            k = int(m.group(2))
            if missing:
                bad.append(f"{path}:{ln} 「{m.group(0)}」指名 {missing}"
                           f"（{names[missing]}），而它正文里没有第 {k} 条：{line.strip()[:60]}")
            elif owner is None:
                bad.append(f"{path}:{ln} 「{m.group(0)}」归属判不出：{line.strip()[:70]}")
            elif item_map[owner][k] != want:
                bad.append(f"{path}:{ln} 「{m.group(0)}」写的是{want}定，而 {owner}"
                           f"（{names[owner]}） 正文里第 {k} 条是{item_map[owner][k]}定：{line.strip()[:60]}")
        if skipped_fenced_lines:
            skipped_fenced_lines_by_file[path] = len(skipped_fenced_lines)
    orphans = orphan_translations()
    if unreadable:
        report_unreadable(unreadable)
    if orphans:
        report_orphan_translations(orphans)
    if bad:
        print(f"  ✗ 分项引用与正文状态不一致 {len(bad)} 处")
        for b in bad[:40]: print("    ", b)
        print("     → 权威是各决策正文的「### 已定项 / ### 未定项」两张表；改引用处，或先改正文再改引用。")
        print("     → 报「指名 Dn 而它没有第 k 条」的：那个编号引不到任何东西，去查它本来想指哪一条。")
        print("     → 一条分项刚翻了状态、引用一大片对不上的：用 scripts/kb/relabel-item.py D<n> <k> 按同一份归属规则改写。")
    if bad or unreadable or orphans:
        sys.exit(1)
    n = sum(len(v) for v in item_map.values())
    # 一处引用都没扫到，这一轮什么都没比过：退 77，门禁记「本次未跑」而不是通过
    # （`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
    if refs == 0:
        print(f"  ! 扫了 {len(files)} 个文件，一处分项引用都没有（{len(item_map)} 条决策、{n} 个分项），本阶段无对象可判")
        sys.exit(77)
    print(f"  ✓ 分项引用与正文状态一致（{len(item_map)} 条决策、{n} 个分项；扫了 {len(files)} 个文件、{refs} 处分项引用）")
    print(f"     没查的：research/prompts/ 下 {len(excluded_prompt_files)} 个文件（原样保存的证据，整目录排除）；"
          f"代码围栏里 {sum(skipped_fenced_lines_by_file.values())} 行（逐字照抄的原样），在 {len(skipped_fenced_lines_by_file)} 个文件里：")
    for path, count in sorted(skipped_fenced_lines_by_file.items()):
        print(f"       {path}：围栏里 {count} 行")


if __name__ == '__main__':
    main()
