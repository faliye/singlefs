#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""把 kb 里的条款整段抄进一份背景材料，抄完自己回读比对。

`.claude/rules/three-way-inference.md` 要求「引 kb 里的条目要整行抄，不许摘句」，
而实测四次摘句里有三次是**写材料的人自己**在转述时删掉了半句——手抄这一步没有任何东西盯着。
这个脚本把「抄」变成一次机械抽取，并在写出之后**回读产物、与源文件逐字节比对**，
对不上就退出码 3。

用法：

    quote-kb.py 出口.md '文件:12-30' '文件@#### 已定项 5' '文件~某个正则'
    quote-kb.py --checklist 清单.md 出口.md 取法...   # 另核清单里标「抄」的每一节都真的抄进来了
    quote-kb.py --checklist 清单.md --cited 正文.md 出口.md 取法...
        # 再核正文里提到的每个 kb 文件（D / E / C / I 编号所在的，和正文里直接写了路径的）都有一张小节清单（C234）

三种取法：
    文件:A-B     第 A 到第 B 行（1 起，含两端）
    文件@标题    从该标题行抄到下一个同级或更高级标题之前
    文件~正则    抄命中的那一整行（命中必须唯一）
文件与参数从哪一处切开：前缀是已有文件的那几处 `:` / `@` / `~` 里最靠前的一处，所以标题或正则里的半角冒号不会被当成行区间（split_spec）。

节边界另算一遍（C264）。回读比对拿产物去对 pick() 取出的那一段，两者出自同一个抽取器，pick() 取错了范围它照样一致。
所以对每条 `文件@标题` 取法，另用 kb-sections.py 的 heading_bounds()（另一段代码，认标题的读法与小节清单是同一份）算一次该节的起止行，
与 pick() 的不等就退出码 7，说明里写两边各算出的行区间。
级别按同一个口径比，不需要折算：两边都数行首 `#` 的个数当级别，节到下一个级别不更深（个数不多于它）的标题之前，没有就到文件末。
两边都只认「一到六个 `#` 加空格开头、且不在代码围栏里」的行当标题（`#123 是单子` 这种井号后无空格的行两边都当正文；C264 复核时 pick() 曾把它当一级标题，
research/prompts/ 三份历史材料里 9 处节尾因此截短，已改）。
`文件:A-B` 与 `文件~正则` 两种取法不在这一条射程里：行区间是人给的、正则只取一行，没有节边界可另算。

「不抄」的理由也核（C234 「不抄」理由那一条）。清单里标「不抄」的行，理由若声称正文已经抄在别处
（理由里有「抄了」「抄进」「抄在」「一起抄」「已抄」这类说法，且点名了一个节：「…」或反引号里的名字，或理由里直接写的 `#` 标题），
就去出口的抄录块里找那个名字：任何一个抄进来的标题行含这个名字就算在；找不到退出码 8。
理由里没点名的（「它的正文在别处抄了」）和没声称抄过的（「正文只点名这个实验的装置」「只抄开篇」）核不了，不在射程里。

自证会红：`quote-kb.py --selftest` 走一遍三种取法，再用 QUOTE_KB_CORRUPT=1
强制进入「抄漏一行」那条分支，确认回读比对判红（`.claude/rules/fs-design.md` 硬要求 2：
每条分支必须能被测试强制进入）。自证的临时目录由 `.claude/hooks/lib_selftest_scratch.py` 建，通过、判红、抛异常
三条路上都删，这三格也由它判。QUOTE_KB_CORRUPT 是自证自己设、自己清的注入，不是弄坏开关。
弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是临时目录那三格的名字，括号里是那条「✗ 自检：」的开头）：
  QUOTE_KB_BREAK=keepscratch  selftest() 的临时目录走回 mkdtemp 不删
      通过那条路（「✗ 自检：通过那条路上临时目录没删」）、red 那条路（「✗ 自检：red 那条路上临时目录没删」）、
      crash 那条路（「✗ 自检：crash 那条路上临时目录没删」）
  QUOTE_KB_BREAK=ignore-not-numbers  cited_kb_files() 不认 `<!-- doc-lint:not-numbers … -->` 声明，声明过的记号照当编号拉 kb 文件
      not-numbers 那一格（「✗ 自检：正文用 doc-lint:not-numbers 声明了 E1」）
  QUOTE_KB_BREAK=not-numbers-in-fence  not_numbers_declared() 把代码围栏里的声明也算上
      围栏那一格（「✗ 自检：代码围栏里的 not-numbers 声明不算」）
  QUOTE_KB_BREAK=bare-lacking-hint  lacking_checklist_message() 走回改前的说法：不点是哪个编号引出的、不给照抄的命令
      退 5 的说明那一格（『✗ 自检：退 5 的说明应当点出 I-7.4 引出了 02-根环.md』）
      连带红的格：「✗ 自检：正文只按路径引了 .claude/kb/decisions/27-小数据打包容器.md（没有编号）而清单里没有它，--cited」
  QUOTE_KB_BREAK=invariants-to-readme  invariant_files_for() 不按类号找类文件，I 号一律退回不变量目录的 README.md
      不变量号按类那一格（「✗ 自检：正文点名 I-7.4 与 I-8.3」）；three-way-materials.py 调的是同一份，它的自证同样红在「正文点名 I-7.4」
  QUOTE_KB_BREAK=no-readme-fallback  invariant_files_for() 类号找不到类文件时什么都不给，不退回 README.md
      退回 README 那一格（「✗ 自检：正文点名 I-10.1」）
  QUOTE_KB_BREAK=fence-not-skipped  pick() 找节尾时不跳过代码围栏（改前的样子：围栏里一行 `# 注释` 当一级标题，整节在那里截断）
      节边界那一格（「✗ 自检：干净的 pick() 下」开头那条；自证自己在进程内再打开它，断言旧的回读比对仍绿、新的节边界比对红、main 退 7）；
      外面设着它跑 --selftest，红在同一格，后面『代码栅栏里的 `#` 行不被当成标题』那一格不会走到
  QUOTE_KB_BREAK=checklist-reasons-unchecked  check_checklist_reasons() 不核「不抄」理由点名的节，红样本判绿
      理由那一格（「✗ 自检：清单一行「不抄」、理由说正文在「某个没抄的节」里抄了」开头那条）
  QUOTE_KB_BREAK=colon-first  split_spec() 走回改前的先后（先 `:` 再 `@` 再 `~`），标题带半角冒号的 `文件@标题` 被切成 `文件:行区间`
      标题带半角冒号那一格（「✗ 自检：标题带半角冒号」开头那条）
  QUOTE_KB_BREAK=kb-directories-raw  kb_directory_entries() 走回改前的读法：登记位的原串只去首尾空白与结尾的 `/`，不规整 `./`，绝对路径与走出仓的也认
      登记位规整那一格（「✗ 自检：登记位写成 ./ 开头、/ 结尾、中间重复 / 的三行」开头那条）；
      stale-candidates.py 调的是同一份，它的自证红在『登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表』
  QUOTE_KB_BREAK=strict-counts-translations  cited_kb_sources() 的严格读法照旧把同号的 .en.md、.ja.md 译文算进 glob（改前的样子：三语之后同号 3 份、一个都不认）
      同号三语那一格（「✗ 自检：同号有中英日三份时」开头那条）

--cited 认编号时，正文里 `<!-- doc-lint:not-numbers 记号 … -->` 声明的记号不当编号（代码围栏里的声明不算，与 doc-lint 同读法）；
认编号的读法只有 cited_kb_sources() 一份：three-way-materials.py 收材料时调它的宽读法（不要求「（」），--cited 用严格读法，宽读法认出的是超集。
D / E 编号按 `<号>-*.md` 找决策与实验页时，严格读法只数中文那一份：同号的 `.en.md`、`.ja.md` 译文（TRANSLATION_SUFFIXES，认法同 `.claude/trilingual-documents`）不算，
三语之后同号 3 份也认得出；宽读法照旧把同号的几份都给（three-way-materials.py 自己排译文）。
I-<类>[.<号>] 指到哪份文件不认死文件名（invariant_files_for）：在 `.claude/kb-directories` 登记的 kb 目录里
（登记是替换不是追加，没登记就是 `.claude/kb`；读法与规整只在这里的 kb_directory_entries 一份，`.claude/gate.d/doc-registries.sh` 的 kb_candidate_roots
与 stale-candidates.py 都调它：开头的 `./`、结尾的 `/` 去掉，绝对路径与 `..` 走出仓的行拒、打到 stderr）找含 `| I-<类>.` 表行的 .md，
这一类住在哪几份就指哪几份；类号找不到文件的，退回那些文件所在目录的 README.md（没有编号、跨类的引用都指它），退 5 的说明里点名是退回的。
退 5 时逐个写缺的文件是正文里哪个记号引出的，给一条能照抄的 `python3 scripts/kb/kb-sections.py <文件> >> <清单>`，
并写明接上的那几张要逐行标抄不抄、标完重跑。

退出码：
    3   回读比对不一致（抄出来的与源文件不一致）
    7   节边界对不上：`文件@标题` 取法里 pick() 取的起止行与 kb-sections.py 的节边界算出的不等（C264；说明里写两边各自的行区间）
    4   清单标了「抄」而附录里没有那一节（正向核对）
    5   --cited 缺清单：正文提到的某个 kb 文件，清单里没给它做一张小节清单
    6   清单标了「不抄」的小节，其实整段被 --extra 之类的行区间连带抄进了附录（反向核对，C320）
    8   清单标了「不抄」、理由声称正文已抄在别处并点名了一个节，出口里找不到那一节（C234 「不抄」理由那一条）
    先后：3 → 7 → 4 → 6 → 8 → 5，第一个红的就退出，不写出口文件。
"""
import contextlib, glob, importlib.util, io, os, re, subprocess, sys
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

FENCE = '```'
SELFTEST_ROLE_VARIABLE = 'QUOTE_KB_SELFTEST_ROLE'   # 自证拉起自己的子进程走判红、抛异常那两条路时设的角色
SELFTEST_SCRATCH_LIBRARY = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                                        '.claude', 'hooks', 'lib_selftest_scratch.py')


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


SPEC_SEPARATORS = ((':', 'range'), ('@', 'head'), ('~', 're'))


def split_spec(spec):
    """→ (文件, 参数, 取法)，取法是 'range' / 'head' / 're'。pick() 与节边界另算（check_section_bounds）共用这一份。
    分隔符取前缀是已有文件的那几处里最靠前的一处，一处都不是已有文件就取最靠前的一处（pick() 再报文件不在）：
    `文件@标题` 的标题、`文件~正则` 的正则里有半角冒号时，冒号在分隔符之后，不会被当成 `文件:A-B` 切开。"""
    if os.environ.get('QUOTE_KB_BREAK') == 'colon-first':   # 弄坏开关：走回改前的先后，先 `:` 再 `@` 再 `~`
        for sep, kind in SPEC_SEPARATORS:
            if sep in spec:
                path, arg = spec.split(sep, 1)
                return path, arg, kind
    candidates = sorted((index, kind) for sep, kind in SPEC_SEPARATORS for index, char in enumerate(spec) if char == sep)
    if not candidates:
        sys.exit(f"quote-kb: 取法认不出：{spec}\n  → 写成 文件:A-B / 文件@标题 / 文件~正则")
    index, kind = next(((index, kind) for index, kind in candidates if os.path.isfile(spec[:index])), candidates[0])
    return spec[:index], spec[index + 1:], kind


def pick(spec):
    """→ (path, first_line_no, [lines])，行号 1 起。"""
    path, arg, kind = split_spec(spec)
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
            if fm and os.environ.get('QUOTE_KB_BREAK') == 'fence-not-skipped':
                fm = None   # 弄坏开关：不认围栏，围栏里的 `# 注释` 当标题（C264 改前的样子）
            if fence is None and fm:
                fence = (fm.group(1)[0], len(fm.group(1)))
            elif fence is not None:
                cm = re.fullmatch(r'(`{3,}|~{3,})\s*', L[j])
                if cm and cm.group(1)[0] == fence[0] and len(cm.group(1)) >= fence[1]:
                    fence = None
            elif re.match(r'#{1,6} ', L[j]) and len(re.match(r'#*', L[j]).group(0)) <= lvl:   # 标题要「#」后带空格：节内 `#9` 这类行不是标题（C264 复核扫出的 9 处）
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


_KB_SECTIONS = None


def kb_sections_module():
    """kb-sections.py 当模块读进来（文件名带连字符，不能直接 import）。懒加载：three-way-materials.py 把本文件当模块读时不必先有它。"""
    global _KB_SECTIONS
    if _KB_SECTIONS is None:
        path = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'kb-sections.py')
        spec = importlib.util.spec_from_file_location('kb_sections', path)
        _KB_SECTIONS = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(_KB_SECTIONS)
    return _KB_SECTIONS


def independent_section_range(path, heading):
    """`文件@标题` 取法的起止行，**不走 pick()**，只用 kb-sections.py 的 heading_bounds() 给出的标题表：
    → ((起, 止), None) 或 (None, 说明)。标题怎么选与 pick() 同一条规则（精确匹配优先、找不到再退回前缀唯一）；
    节从该标题起，到下一个级别不更深的标题之前，没有就到文件末（C264）。"""
    rows, total = kb_sections_module().heading_bounds(path)
    hits = [row for row in rows if row[2] == heading]
    if len(hits) != 1:
        hits = [row for row in rows if row[2].startswith(heading)]
    if len(hits) != 1:
        return None, f"kb-sections.py 认的标题里「{heading}」命中 {len(hits)} 次（pick() 选中的那一行在它眼里不是标题，或有多个）"
    start, level, _text = hits[0]
    end = next((line for line, other_level, _ in rows if line > start and other_level <= level), total + 1) - 1
    return (start, end), None


def check_section_bounds(specs):
    """每条 `文件@标题` 取法，pick() 取的起止行与 independent_section_range() 另算的比一遍（C264）。
    → [一条说明一个对不上的取法]，空表 = 都对得上；`文件:A-B` 与 `文件~正则` 不在射程里。"""
    problems = []
    for spec in specs:
        path, argument, kind = split_spec(spec)
        if kind != 'head':
            continue
        _path, first, body = pick(spec)
        got = (first, first + len(body) - 1)
        want, why = independent_section_range(path, argument)
        if why:
            problems.append(f"「{spec}」pick() 取第 {got[0]}-{got[1]} 行，但{why}")
        elif want != got:
            problems.append(f"「{spec}」pick() 取第 {got[0]}-{got[1]} 行，kb-sections.py 的节边界算出第 {want[0]}-{want[1]} 行")
    return problems


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


# 理由里声称「正文已经抄在别处」的说法。「不抄」「只抄」「照抄」不算（它们说的是这一节没抄、或抄了多少），所以前面不许是这三个字。
COPIED_ELSEWHERE_CLAIM = re.compile(r'(?<![不只照])抄(?:了|过|进|入|录|在|于|到)|一起抄|一并抄|连带抄|已抄')
QUOTED_SECTION_NAME = re.compile(r'「([^」]+)」|`([^`]+)`')
HEADING_IN_REASON = re.compile(r'(#{1,6} .+?)(?=\s+(?:一起|一并|里|中|内|已|的正文|抄)|\s*[，。；,;「」`]|$)')


def sections_named_in_reason(reason):
    """理由若声称正文已经抄在别处，→ 它点名的节名（「…」、反引号里的、或直接写的 `#` 标题）；没声称或没点名 → []。"""
    if not COPIED_ELSEWHERE_CLAIM.search(reason):
        return []
    names = [first or second for first, second in QUOTED_SECTION_NAME.findall(reason)]
    names += HEADING_IN_REASON.findall(reason)
    return [name for name in (normalize_heading(n).lstrip('#').strip() for n in names) if name]


def check_checklist_reasons(checklist, produced):
    """清单标「不抄」、理由声称正文已抄在别处并点名了一个节的行，去出口的抄录块里找那个节（C234 「不抄」理由那一条）：
    任何一个抄进来的标题行含这个名字就算在。→ [(文件, 小节, 点名的节名)]，找不到的，空表 = 对得上。

    射程：理由里没点名的（「它的正文在别处抄了」）、没声称抄过的（「正文只点名这个实验的装置」）核不了，跳过；
    清单与反向核对一样只看 `|` 分格的表行。2026-09-09 的实测：清单给「已定项索引表」标的不抄理由逐字
    「表里四行的正文各自抄或不抄」是假的——现查那张表名下只有两个四级小节，另两条整条只活在被标「不抄」的索引表里。
    """
    if os.environ.get('QUOTE_KB_BREAK') == 'checklist-reasons-unchecked':   # 弄坏开关：不核理由
        return []
    copied_headings = [heading.lstrip('#').strip() for headings in heading_lines_by_file(produced).values() for heading in headings]
    cur, missing = None, []
    for row in open(checklist, encoding='utf-8').read().split('\n'):
        m = re.match(r'### 小节清单：`([^`]+)`', row)
        if m:
            cur = m.group(1); continue
        if not row.startswith('| ') or row.startswith('|---'):
            continue
        cells = [c.strip() for c in row.strip().strip('|').split('|')]
        if len(cells) < 3 or cells[1] != '不抄':
            continue
        for name in sections_named_in_reason('|'.join(cells[2:])):
            if not any(name in heading for heading in copied_headings):
                missing.append((cur, cells[0], name))
    return missing


NOT_NUMBERS_MARK = re.compile(r'<!--\s*doc-lint:not-numbers\s+(.*?)\s*-->')


def not_numbers_declared(text):
    """正文里 `<!-- doc-lint:not-numbers 记号 … -->` 声明的记号：它们不是编号，不按编号拉 kb 文件。
    读法与 doc-lint 的 H 段相同：代码围栏里的声明不算。three-way-materials.py 收编号时调的也是这一份。"""
    declared, fence = set(), False
    for line in text.split('\n'):
        if line.lstrip(' \t').startswith('```'):
            fence = not fence
            continue
        if fence and os.environ.get('QUOTE_KB_BREAK') != 'not-numbers-in-fence':
            continue
        for words in NOT_NUMBERS_MARK.findall(line):
            declared.update(words.split())
    return declared


KB_PATH_IN_TEXT = re.compile(r'\.claude/kb/[^\s`)（|]+\.md')
# 编号的四种形态：(字母, 严格读法要的形态, 宽读法要的形态)。严格读法（--cited 判清单全不全）要编号后面紧跟全角括号「（」；
# 宽读法（three-way-materials.py 收材料）不要求括号、不变量号连类号 I-<n> 也认。同一段正文，宽读法认出的编号是严格读法的超集。
NUMBER_SHAPES = (
    ('D', r'(?<![A-Za-z0-9-])(D\d+)（', r'(?<![A-Za-z0-9-])(D\d+)(?![0-9])'),
    ('E', r'(?<![A-Za-z0-9-])(E\d+)（', r'(?<![A-Za-z0-9-])(E\d+)(?![0-9])'),
    ('C', r'(?<![A-Za-z0-9-])(C\d+)（', r'(?<![A-Za-z0-9-])(C\d+)(?![0-9])'),
    ('I', r'(?<![A-Za-z0-9-])(I-\d+\.\d+)（', r'(?<![A-Za-z0-9-])(I-\d+(?:\.\d+)?)(?![0-9])'),
)
# 三语登记（.claude/trilingual-documents 文件头）认译文的办法：以这两种后缀结尾的是译文，不是中文那一份；严格读法按编号找页时不数它们
TRANSLATION_SUFFIXES = ('.en.md', '.ja.md')


KB_DIRECTORIES_REGISTRY = '.claude/kb-directories'
INVARIANT_TABLE_ROW = re.compile(r'^\| I-(\d+)\.\d+ ', re.M)   # 不变量表行，与 doc-registries.sh 的 invariant_table_files 同一判据
INVARIANT_DIRECTORY_INDEX = 'README.md'


def kb_directory_entries(root):
    """登记位 `.claude/kb-directories` 逐行读、规整：→ (登记的目录 [相对仓根，规整过，按登记次序去重], 拒掉的行 [(行号, 原文, 为什么)])。
    空行与 `#` 起头的行跳过，`#` 之后是注释。规整用 os.path.normpath：去掉开头的 `./`、结尾的 `/`、中间的 `./` 与重复的 `/`，
    这样拼出的路径与 git 列出的相对路径同形（stale-candidates.py 拿它拼模式去配 git 路径）。拒：绝对路径、规整之后以 `..` 起头（走出仓）
    或就是仓根本身（`.`）的行。规整过、没被拒、目录不在的照旧不算登记、不报。
    弄坏开关 QUOTE_KB_BREAK=kb-directories-raw 退回改前的读法：原串只去首尾空白与结尾的 `/`，不规整也不拒。"""
    registered, refused = [], []
    registry = os.path.join(root, KB_DIRECTORIES_REGISTRY)
    if not os.path.isfile(registry):
        return registered, refused
    raw_reading = os.environ.get('QUOTE_KB_BREAK') == 'kb-directories-raw'
    for number, raw in enumerate(open(registry, encoding='utf-8'), 1):
        written = raw.split('#', 1)[0].strip()
        if not written:
            continue
        if raw_reading:
            entry = written.rstrip('/')
        elif os.path.isabs(written):
            refused.append((number, written, '是绝对路径'))
            continue
        else:
            entry = os.path.normpath(written)
            if entry == '..' or entry.startswith('../'):
                refused.append((number, written, f'规整成 {entry}，走出了仓'))
                continue
            if entry == '.':
                refused.append((number, written, '规整之后是仓根本身，不是一个 kb 目录'))
                continue
        if entry and entry not in registered and os.path.isdir(os.path.join(root, entry)):
            registered.append(entry)
    return registered, refused


KB_DIRECTORIES_REFUSALS_REPORTED = set()   # 同一个进程里同一行只报一次（一次调用链里 kb_candidate_roots 会被调好几次）


def kb_candidate_roots(root):
    """登记在 `.claude/kb-directories` 的 kb 目录，相对仓根、排序去重（读法与规整只在 kb_directory_entries 一份）。登记是替换不是追加：登记了就只认登记的那几个，
    没登记（或登记的目录都不在、都被拒了）就是 `.claude/kb`。被拒的行（绝对路径、走出仓）逐行打到 stderr，写那一行与出路，不静默跳过。
    门禁 doc-registries.sh 的 kb_candidate_roots 调的就是这一份。"""
    registered, refused = kb_directory_entries(root)
    for number, written, why in refused:
        key = (os.path.realpath(root), number, written)
        if key in KB_DIRECTORIES_REFUSALS_REPORTED:
            continue
        KB_DIRECTORIES_REFUSALS_REPORTED.add(key)
        print(f"  ✗ {KB_DIRECTORIES_REGISTRY} 第 {number} 行「{written}」{why}，这一行不认，那个目录不算 kb 目录", file=sys.stderr)
        print("     → 怎么办：写成相对仓根的目录（例如 crates/singlefs-checker/invariants），开头不带 /、不用 .. 走出仓；"
              "开头的 ./ 与结尾的 / 可以写，读的时候会去掉", file=sys.stderr)
    return sorted(registered) or ['.claude/kb']


def invariant_class_files(root):
    """候选 kb 目录里含不变量表行的 .md：→ ({类号: [相对仓根的路径…]}, [回退用的 README.md…])。
    单文件的清单与按类拆开的几份文件都照表行找，不认文件名；回退是这些文件所在目录里的 README.md（在的那几份；
    一份都不在就是第一个目录的那一份，退 5 时报「这份文件不在」）；一份带表行的文件都没有，回退是第一个候选目录的 README.md。"""
    by_class, directories = {}, []
    for candidate in kb_candidate_roots(root):
        for path in sorted(glob.glob(os.path.join(root, candidate, '**', '*.md'), recursive=True)):
            try:
                classes = {int(number) for number in INVARIANT_TABLE_ROW.findall(open(path, encoding='utf-8').read())}
            except (OSError, UnicodeDecodeError):
                continue
            if not classes:
                continue
            relative = os.path.relpath(path, root)
            for number in classes:
                by_class.setdefault(number, []).append(relative)
            if os.path.dirname(relative) not in directories:
                directories.append(os.path.dirname(relative))
    indexes = [os.path.join(directory, INVARIANT_DIRECTORY_INDEX) for directory in (directories or kb_candidate_roots(root)[:1])]
    present = [index for index in indexes if os.path.isfile(os.path.join(root, index))]
    return by_class, present or indexes[:1]


def invariant_files_for(token, root, class_files=None):
    """I-<类>[.<号>] 指到的文件：→ (路径列表, 退回 README 时的一句说明，没退回是 None)。class_files 是 invariant_class_files(root) 的结果，不给就现算。"""
    by_class, fallback = class_files if class_files is not None else invariant_class_files(root)
    number = int(re.match(r'I-(\d+)', token).group(1))
    if by_class.get(number) and os.environ.get('QUOTE_KB_BREAK') != 'invariants-to-readme':
        return list(by_class[number]), None
    if os.environ.get('QUOTE_KB_BREAK') == 'no-readme-fallback':
        return [], None
    return list(fallback), (f"I-{number} 类在登记的 kb 目录（{KB_DIRECTORIES_REGISTRY}）里没有含「| I-{number}.」表行的文件，"
                            f"退回 {INVARIANT_DIRECTORY_INDEX}")


def cited_kb_sources(text, root, loose=False):
    """正文里提到的 kb 文件与引出它的记号：→ [(相对仓根的路径, 记号)]，一个文件一行，显式路径在前、编号在后，各按正文里第一次出现的次序。
    记号是正文里写出的路径本身，或 D<n> / E<n> / C<n> / I-<n>.<m> 编号；`doc-lint:not-numbers` 里声明的记号不算编号（not_numbers_declared）。
    I 号指到含这一类表行的那几份文件，类号找不到文件的退回 README.md（invariant_files_for）。
    认编号的读法只有这一份：quote-kb.py --cited（cited_kb_files，严格读法）与 three-way-materials.py 收材料（loose=True，宽读法）都调它。
    严格读法：编号后面紧跟「（」，D / E 编号要恰好对上一份决策或实验页（只数中文那一份，同号的 .en.md、.ja.md 译文不算）；宽读法：不要求括号、对上几份拉几份（译文也给）。
    宽读法认出的是严格读法的超集，材料按宽读法拉齐的文件，--cited 按严格读法判时一份不缺。路径不在的照样列，由调用方判。

    C234（三方材料的小节清单只覆盖被判那一项所在的文件）：清单只给被判那一项所在的文件做，
    别的文件里那一半闸门问题就无声消失。2026-09-09 立账、写明了怎么拦，2026-09-11 又被打中一次
    （D19 未定项 6 第二轮材料漏了 D23 / D16 / D1 三个文件，第 6 格的全部机制住在 D23）。
    """
    declared = set() if os.environ.get('QUOTE_KB_BREAK') == 'ignore-not-numbers' else not_numbers_declared(text)
    sources = [(path, path) for path in KB_PATH_IN_TEXT.findall(text)]
    numbers = []
    for letter, strict, wide in NUMBER_SHAPES:
        numbers += [(match.start(), letter, match.group(1)) for match in re.finditer(wide if loose else strict, text)]
    class_files = None
    for _position, letter, token in sorted(numbers):
        if token in declared:
            continue
        if letter in ('D', 'E'):
            folder, number = ('decisions' if letter == 'D' else 'experiments'), int(token[1:])
            hits = set()
            for stem in (f'{number:02d}-', f'{number}-'):
                hits.update(glob.glob(os.path.join(root, '.claude/kb', folder, stem + '*.md')))
            if not loose and os.environ.get('QUOTE_KB_BREAK') != 'strict-counts-translations':
                hits = {hit for hit in hits if not hit.endswith(TRANSLATION_SUFFIXES)}   # 严格读法只数中文那一份（弄坏开关：译文照数）
            if len(hits) == 1 or (loose and hits):
                sources += [(os.path.relpath(hit, root), token) for hit in sorted(hits)]
        elif letter == 'C':
            sources.append(('.claude/kb/checks-owed.md', token))
        else:
            class_files = class_files if class_files is not None else invariant_class_files(root)
            sources += [(path, token) for path in invariant_files_for(token, root, class_files)[0]]
    ordered, seen = [], set()
    for path, token in sources:
        if path not in seen:
            seen.add(path)
            ordered.append((path, token))
    return ordered


def cited_kb_files(text, root):
    """正文里提到的 kb 文件（严格读法，--cited 用）：显式路径，加上 D<n>（ / E<n>（ / C<n>（ / I-<n>.<m>（ 这几类编号所在的文件。读法在 cited_kb_sources。"""
    return {path for path, _token in cited_kb_sources(text, root)}


def lacking_checklist_message(lacking, sources, checklist, root='.'):
    """--cited 缺清单（退 5）时的说明：每个缺的文件写是正文里哪个记号引出的（I 号退回 README.md 的另写一句为什么），给一条能照抄的 kb-sections.py 命令。→ [行]。"""
    script = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'kb-sections.py')
    relative_script = os.path.relpath(script)
    shown_script = relative_script if not relative_script.startswith('..') else script
    trigger = dict(sources)
    lines = [f"quote-kb: 正文提到了 {len(lacking)} 个 kb 文件，清单里没有它们的小节清单："]
    if os.environ.get('QUOTE_KB_BREAK') == 'bare-lacking-hint':   # 弄坏开关：改前的说法，不点编号、不给命令
        lines += [f"    {m}" for m in lacking]
        lines.append("  → 用 scripts/kb/kb-sections.py 给这几个文件各生成一张清单，逐行判抄不抄再重跑（C234）")
        return lines
    present = [m for m in lacking if os.path.isfile(m)]
    absent = [m for m in lacking if not os.path.isfile(m)]
    for m in lacking:
        source = trigger.get(m, m)
        how = "正文里写了这个路径" if source == m else f"正文里的 {source} 引出"
        if source.startswith('I-'):
            fallback_note = invariant_files_for(source, root)[1]
            if fallback_note:
                how += f"；{fallback_note}"
        lines.append(f"    {m}（{how}）" + ("" if m in present else "：这份文件不在"))
    if present:
        lines.append("  → 怎么办：给每个文件各生成一张清单、接到清单末尾（照抄，一行一个文件）：")
        lines += [f"      python3 {shown_script} {m} >> {checklist}" for m in present]
        lines.append("    接上的那几张「抄 / 不抄」「理由」两列是空的：逐行标「抄」或「不抄」（不抄写理由），标完重跑同一条命令（C234）；")
        lines.append("    清单是 three-way-materials.py 生成的，就不用手接：改好别处之后跑它给的 --resume 那一条，它补上缺的几张（默认全抄）再接着做附录、背景材料与快照")
    if absent:
        lines.append(f"  → 怎么办：{'、'.join(absent)} 不在：正文里那个路径或编号写错了，改正文；或者在正文里用 <!-- doc-lint:not-numbers 记号 --> 声明它不是编号")
    return lines


def check_cited(checklist, cited_text, root):
    """正文提到的 kb 文件里，清单没有给它做一张的那几个（空表 = 对得上）。"""
    have = set(re.findall(r'### 小节清单：`([^`]+)`', open(checklist, encoding='utf-8').read()))
    return sorted(f for f in cited_kb_files(cited_text, root) if f not in have)


def selftest():
    """主体在 selftest_in；临时目录由共用库建（通过、判红、抛异常都删），那三格也由它判。"""
    try:
        spec = importlib.util.spec_from_file_location('lib_selftest_scratch', SELFTEST_SCRATCH_LIBRARY)
        scratch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(scratch)
    except Exception as error:   # 文件不在、语法错、导入时抛的都算读不到
        print(f"  ✗ 自检：读不到共用的临时目录库 {SELFTEST_SCRATCH_LIBRARY}（{error!r}）")
        print("     → 怎么办：恢复 .claude/hooks/lib_selftest_scratch.py（自证临时目录的建法与三格探查只有那一份），再跑 --selftest。")
        return 1
    exit_code, problems = scratch.run_with_scratch(
        selftest_in, 'quote-kb-selftest-', os.environ.get('QUOTE_KB_BREAK') == 'keepscratch',
        SELFTEST_ROLE_VARIABLE, [sys.executable, os.path.abspath(__file__), '--selftest'])
    return scratch.merged_exit_code(exit_code, problems, 'scripts/kb/quote-kb.py 的 selftest()', 'QUOTE_KB_BREAK=keepscratch')


def selftest_in(work):
    sample_directory = os.path.join(work, 'sample')
    os.mkdir(sample_directory)
    src = os.path.join(sample_directory, 'sample.md')
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
        print(f"  ✗ 自检：干净那轮就对不上——{bad}")
        print("     → 怎么办：build() 或 verify() 逻辑坏了——干净输入本该逐字节一致，去看这两个函数最近的改动。")
        return 1
    os.environ['QUOTE_KB_CORRUPT'] = '1'
    bad = verify(specs, build(specs))
    del os.environ['QUOTE_KB_CORRUPT']
    if not bad:
        print("  ✗ 自检：抄漏一行时回读比对**没有**判红，这道闸是摆设")
        print("     → 怎么办：QUOTE_KB_CORRUPT 注入的坏输入应该被 verify() 抓住；去看 verify() 的比对逻辑是不是被改松了。")
        return 1
    ck = os.path.join(os.path.dirname(src), 'checklist.md')
    with open(ck, 'w', encoding='utf-8') as f:
        f.write(f"### 小节清单：`{src}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n"
                "| #### 已定项 5 | 抄 | 样本 |\n| #### 已定项 6 | 抄 | 样本 |\n"
                "| #### 带代码块的 | 不抄 | 样本 |\n")
    two = [f'{src}@#### 已定项 5', f'{src}@#### 已定项 6']
    if check_checklist(ck, build(two)):
        print("  ✗ 自检：清单与附录对得上时竟然判红")
        print("     → 怎么办：check_checklist() 误判了对得上的情况，去看它逐节比对小节标题的逻辑。")
        return 1
    if not check_checklist(ck, build(two[:1])):
        print("  ✗ 自检：清单标了抄而附录里没有，比对**没有**判红（C262 那个形态）")
        print("     → 怎么办：check_checklist() 该抓的缺节漏判了，去看它是不是漏比对了某个标记「抄」的小节。")
        return 1
    if check_checklist_reverse(ck, build(two)):
        print("  ✗ 自检：清单标「不抄」的小节没被带进附录，反向核对却判红")
        print("     → 怎么办：check_checklist_reverse() 误报了，去看它判断「标题是否被行区间连带抄进附录」的逻辑。")
        return 1
    leaked = check_checklist_reverse(ck, build(two + [f'{src}:8-13']))
    if not leaked:
        print("  ✗ 自检：一个行区间把「不抄」的小节标题连带抄进了附录，反向核对**没有**判红（C320）")
        print("     → 怎么办：check_checklist_reverse() 该抓的泄漏漏判了，去看它扫附录内容找标题行的逻辑。")
        return 1
    if leaked != [(src, '#### 带代码块的')]:
        print(f"  ✗ 自检：反向核对命中的不是预期那一节——{leaked}")
        print("     → 怎么办：check_checklist_reverse() 命中的节不对，去比对它返回的 (文件, 标题) 元组构造逻辑。")
        return 1
    print("  ✓ 自检：反向核对——「不抄」的小节没被带进附录时是绿的，被行区间连带带进去时判红（C320）")
    src2 = os.path.join(os.path.dirname(src), 'sample2.md')
    with open(src2, 'w', encoding='utf-8') as f:
        f.write('# 头二\n\n#### 带代码块的\n\n另一个文件，同名标题在这里合法标了抄\n')
    with open(ck, 'a', encoding='utf-8') as f:
        f.write(f"\n### 小节清单：`{src2}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n"
                "| #### 带代码块的 | 抄 | 样本二 |\n")
    cross_file = check_checklist_reverse(ck, build([f'{src2}@#### 带代码块的']))
    if cross_file:
        print(f"  ✗ 自检：另一个文件里同名标题合法抄进来，反向核对却误报到别的文件头上——{cross_file}")
        print("     → 怎么办：check_checklist_reverse() 按文件收窄的逻辑坏了，去看它是不是没把文件名带进匹配条件。")
        return 1
    print("  ✓ 自检：反向核对按文件收窄，不同文件里同名的通用小标题不会互相误报")
    # C234 「不抄」理由那一条：「不抄」的理由声称正文已抄在别处并点名了一个节，要去出口里核
    reasons_checklist = os.path.join(sample_directory, 'reasons-checklist.md')
    reason_output = os.path.join(sample_directory, 'reasons-output.md')
    reason_specs = [f'{src}@#### 已定项 5']

    def reasons_checklist_with(reason):
        with open(reasons_checklist, 'w', encoding='utf-8') as f:
            f.write(f"### 小节清单：`{src}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n"
                    "| #### 已定项 5 | 抄 | 样本 |\n"
                    f"| #### 已定项 6 | 不抄 | {reason} |\n")
    reason_produced = build(reason_specs)
    for reason in ('不抄：正文在「已定项 5」里抄了', '随 #### 已定项 5 一起抄了', '正文在 `已定项 5` 那一节里抄进去了',
                   '它的正文在别处抄了', '开篇照抄，各次跑的节不抄', '这一轮只抄开篇', '样本'):
        reasons_checklist_with(reason)
        if check_checklist_reasons(reasons_checklist, reason_produced):
            print(f"  ✗ 自检：理由「{reason}」点名的节在出口里（或没点名、没声称抄过），理由核对却判红")
            print("     → 怎么办：check_checklist_reasons() 误报了，去看 sections_named_in_reason() 认的说法与它在抄录块标题里找名字的逻辑。")
            return 1
    for reason, wanted_name in (('不抄：正文在「某个没抄的节」里抄了', '某个没抄的节'), ('随 #### 没有的节 一起抄了', '没有的节'),
                                ('正文在 `另一个没抄的节` 里抄进去了', '另一个没抄的节')):
        reasons_checklist_with(reason)
        unfounded = check_checklist_reasons(reasons_checklist, reason_produced)
        if unfounded != [(src, '#### 已定项 6', wanted_name)]:
            print(f"  ✗ 自检：清单一行「不抄」、理由说正文在「某个没抄的节」里抄了（理由「{reason}」），而出口没有那一节，理由核对应当只报 {(src, '#### 已定项 6', wanted_name)}，实报 {unfounded}")
            print("     → 怎么办：check_checklist_reasons() 该抓的假理由漏判了；QUOTE_KB_BREAK=checklist-reasons-unchecked 设着的话这里本来就该红。")
            return 1
    reasons_checklist_with('不抄：正文在「某个没抄的节」里抄了')
    captured_out, captured_err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(captured_out), contextlib.redirect_stderr(captured_err):
        red_code = main(['--checklist', reasons_checklist, reason_output, *reason_specs])
    reasons_checklist_with('不抄：正文在「已定项 5」里抄了')
    with contextlib.redirect_stdout(captured_out), contextlib.redirect_stderr(captured_err):
        green_code = main(['--checklist', reasons_checklist, reason_output, *reason_specs])
    if red_code != 8 or '某个没抄的节' not in captured_err.getvalue() or green_code != 0 or not os.path.exists(reason_output):
        print(f"  ✗ 自检：main() 对假理由应当退 8（点名那一节），对真理由应当退 0 并写出口，实际 {red_code} 与 {green_code}：{captured_err.getvalue()}")
        print("     → 怎么办：看 main() 里反向核对之后那一段：check_checklist_reasons() 有结果就打出来并退 8。")
        return 1
    print("  ✓ 自检：清单一行「不抄」、理由说正文在某个节里抄了——出口里有那一节时绿，没有时判红（main 退 8）；没点名的、没声称抄过的不判（C234 「不抄」理由那一条）")
    _, _, bare = pick(f'{src}@### 未定项')
    if bare != ['### 未定项', '', '裸标题下的表', '']:
        print(f"  ✗ 自检：裸标题「### 未定项」取错了——{bare}")
        print("     → 怎么办：pick() 处理裸标题（没有括注）与带括注的同前缀标题共存时的匹配逻辑坏了，去查它怎么选中确切的那一节。")
        return 1
    print("  ✓ 自检：三种取法（含带代码块的条款）都逐字节一致，且抄漏一行时判红")
    print("  ✓ 自检：裸标题与同前缀的带括注标题并存时，精确匹配取到裸标题那一节")
    # 标题带半角冒号：`文件@标题` 与 `文件~正则` 的冒号在分隔符之后，照取标题与正则；`文件:A-B` 照旧认
    colon_source = os.path.join(sample_directory, 'colon.md')
    with open(colon_source, 'w', encoding='utf-8') as f:
        f.write('## 中文那一节\n\n正文一\n\n## Criterion 3: Doubling Experiment\n\nbody: with colon\n\n## 后一节\n')
    colon_specs = [f'{colon_source}@## Criterion 3: Doubling Experiment', f'{colon_source}@## 中文那一节',
                   f'{colon_source}~^body: with colon$', f'{colon_source}:1-3']
    colon_wanted = [(colon_source, 5, ['## Criterion 3: Doubling Experiment', '', 'body: with colon', '']),
                    (colon_source, 1, ['## 中文那一节', '', '正文一', '']), (colon_source, 7, ['body: with colon']),
                    (colon_source, 1, ['## 中文那一节', '', '正文一'])]
    colon_output = os.path.join(sample_directory, 'colon-output.md')
    try:
        colon_picked = [pick(spec) for spec in colon_specs]
        colon_problem = verify(colon_specs, build(colon_specs)) or '；'.join(check_section_bounds(colon_specs)) or None
        captured_out, captured_err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(captured_out), contextlib.redirect_stderr(captured_err):
            colon_code = main([colon_output, *colon_specs])
    except SystemExit as stop:
        colon_picked, colon_problem, colon_code = None, f'取法切错、pick() 退出：{stop.code}', None
    if colon_picked != colon_wanted or colon_problem or colon_code != 0 or not os.path.exists(colon_output):
        print(f"  ✗ 自检：标题带半角冒号的 `文件@标题`、正则带半角冒号的 `文件~正则` 应当照取、回读逐字节一致、节边界对得上、main 退 0，"
              f"`文件:A-B` 照旧认；实际取到 {colon_picked}，{colon_problem}，main 退 {colon_code}")
        print("     → 怎么办：看 split_spec() 取分隔符那一步：要取前缀是已有文件的最靠前一处；QUOTE_KB_BREAK=colon-first 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：标题带半角冒号的 `文件@标题` 与正则带冒号的 `文件~正则` 照取、回读逐字节一致、节边界对得上，`文件:A-B` 照旧认")
    # C264：节边界另算一遍。干净的 pick() 与 kb-sections.py 的节边界对得上；
    # pick() 不跳围栏（QUOTE_KB_BREAK=fence-not-skipped）时，旧的回读比对仍绿、新的节边界比对红、main 退 7、不写出口
    fence_specs = [f'{src}@#### 代码块里有井号']
    sample_lines = read(src)
    section_start = sample_lines.index('#### 代码块里有井号') + 1
    comment_line = sample_lines.index('# 注释一行') + 1
    true_range, truncated_range = f'{section_start}-{len(sample_lines)}', f'{section_start}-{comment_line - 1}'
    clean_bounds = check_section_bounds(fence_specs)
    if clean_bounds or pick(fence_specs[0])[1:] != (section_start, sample_lines[section_start - 1:]):
        print(f"  ✗ 自检：干净的 pick() 下，带代码围栏的那一节与 kb-sections.py 的节边界对不上（本该都是第 {true_range} 行）——{clean_bounds}")
        print("     → 怎么办：看 pick() 找节尾的围栏状态机与 kb-sections.py 的 heading_bounds()，哪一边把围栏里的 `#` 行当了标题；QUOTE_KB_BREAK=fence-not-skipped 设着的话这里本来就该红。")
        return 1
    saved_break = os.environ.get('QUOTE_KB_BREAK')
    os.environ['QUOTE_KB_BREAK'] = 'fence-not-skipped'
    fence_output = os.path.join(work, 'fence-output.md')
    try:
        old_readback = verify(fence_specs, build(fence_specs))
        new_bounds = check_section_bounds(fence_specs)
        captured_out, captured_err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(captured_out), contextlib.redirect_stderr(captured_err):
            broken_code = main([fence_output, *fence_specs])
    finally:
        if saved_break is None:
            del os.environ['QUOTE_KB_BREAK']
        else:
            os.environ['QUOTE_KB_BREAK'] = saved_break
    if old_readback is not None:
        print(f"  ✗ 自检：pick() 不跳围栏时旧的回读比对应当仍然绿（C264 的病就是它看不见），实际判红了——{old_readback}")
        print("     → 怎么办：verify() 与 pick() 出自同一个抽取器，两边一起错时它必须一致；它自己红了说明回读比对被改过，去看 verify()。")
        return 1
    if not new_bounds or truncated_range not in new_bounds[0] or true_range not in new_bounds[0]:
        print(f"  ✗ 自检：pick() 不跳围栏、旧回读比对绿的时候，节边界另算的比对**没有**判红，或说明里没写两边各算出的行区间（该有第 {truncated_range} 行与第 {true_range} 行）——{new_bounds}")
        print("     → 怎么办：check_section_bounds() 该抓的对不上漏判了，去看它是不是又走回了 pick()，或比较写松了。")
        return 1
    if broken_code != 7 or os.path.exists(fence_output) or truncated_range not in captured_err.getvalue():
        print(f"  ✗ 自检：节边界对不上时 main() 应当退 7、不写出口、stderr 点名两边的行区间，实际退 {broken_code}，出口{'写了' if os.path.exists(fence_output) else '没写'}：{captured_err.getvalue()}")
        print("     → 怎么办：看 main() 里 verify 之后那一段：check_section_bounds() 有结果就打出来并退 7。")
        return 1
    hashtag_source = os.path.join(sample_directory, 'hashtag.md')
    with open(hashtag_source, 'w', encoding='utf-8') as f:
        f.write('## 节\n\n#123 是个单子号，不是标题\n\n后文\n')
    hashtag_bounds = check_section_bounds([f'{hashtag_source}@## 节'])
    if hashtag_bounds:
        print(f"  ✗ 自检：节里有一行 `#123 …`（井号后没有空格，不是标题）时 pick() 与 kb-sections.py 都该把它当正文、节尾一致（第 1-6 行），实际——{hashtag_bounds}")
        print("     → 怎么办：pick() 找节尾只认「一到六个 # 加空格」的行；它又把井号后无空格的行当标题了就去看那一处的正则。")
        return 1
    print("  ✓ 自检：节边界另算一遍（C264）——干净时 pick() 与 kb-sections.py 对得上；pick() 不跳围栏时旧回读比对仍绿、新比对红、main 退 7；`#123` 这类井号后无空格的行两边都不当标题")
    _, _, fenced = pick(f'{src}@#### 代码块里有井号')
    if '栅栏之后那一行' not in fenced:
        print(f"  ✗ 自检：代码栅栏里的 `# 注释` 把整节截断了（只取到 {len(fenced)} 行）")
        print("     → 怎么办：pick() 把代码栅栏内的 `#` 行误判成了标题行，去查它判断「是否在代码块内」的状态机。")
        return 1
    print("  ✓ 自检：代码栅栏里的 `#` 行不被当成标题，整节取全")
    cited_root = os.path.join(work, 'cited')
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
        print(f"  ✗ 自检：正文提到 C1 而清单里没有 checks-owed.md，--cited 应当只报它一个，实报 {lacking}")
        print("     → 怎么办：check_cited() 从正文里识别 D/E/C/I 编号、映射到 kb 文件的逻辑坏了，去查它的编号 → 文件映射表。")
        return 1
    with open(cited_checklist, 'a', encoding='utf-8') as f:
        f.write("\n### 小节清单：`.claude/kb/checks-owed.md`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n")
    if check_cited(cited_checklist, cited_body, cited_root):
        print("  ✗ 自检：清单补齐之后 --cited 仍然报缺")
        print("     → 怎么办：check_cited() 没有正确识别出清单已经补齐，去查它判断「这个文件是否已有小节清单」的逻辑。")
        return 1
    # C234 按路径引的那一型：正文只按路径引 kb 文件（不带编号），清单里没有它的小节清单，--cited 退 5；补上才退 0
    os.makedirs(os.path.join(cited_root, '.claude/kb/decisions'), exist_ok=True)
    path_only_file = '.claude/kb/decisions/27-小数据打包容器.md'
    with open(os.path.join(cited_root, path_only_file), 'w', encoding='utf-8') as f:
        f.write('## 样本\n\n正文\n')
    with open(os.path.join(cited_root, 'path-body.md'), 'w', encoding='utf-8') as f:
        f.write(f'这一轮要引 {path_only_file} 里的限定，正文里没有任何 D / E / C / I 编号。\n')
    path_checklist = os.path.join(cited_root, 'path-checklist.md')
    with open(path_checklist, 'w', encoding='utf-8') as f:
        f.write("### 小节清单：`.claude/kb/decisions/99-样本.md`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n| ## D99 样本 —— 已定 | 抄 | 样本 |\n")
    path_command = [sys.executable, os.path.abspath(__file__), '--checklist', 'path-checklist.md', '--cited', 'path-body.md', 'path-output.md',
                    '.claude/kb/decisions/99-样本.md@## D99 样本 —— 已定']
    lacking_run = subprocess.run(path_command, cwd=cited_root, capture_output=True, text=True)
    if lacking_run.returncode != 5 or f'{path_only_file}（正文里写了这个路径）' not in lacking_run.stderr or os.path.exists(os.path.join(cited_root, 'path-output.md')):
        print(f"  ✗ 自检：正文只按路径引了 {path_only_file}（没有编号）而清单里没有它，--cited 应当退 5、点名那个路径、不写出口，"
              f"实际退 {lacking_run.returncode}：{lacking_run.stdout[-200:]}{lacking_run.stderr[-400:]}")
        print("     → 怎么办：cited_kb_sources() 要把 KB_PATH_IN_TEXT 认出的显式路径也算进去，退 5 的说明写「正文里写了这个路径」（C234 按路径引的那一型）。")
        return 1
    with open(path_checklist, 'a', encoding='utf-8') as f:
        f.write(f"\n### 小节清单：`{path_only_file}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n| ## 样本 | 不抄 | 样本 |\n")
    covered_run = subprocess.run(path_command, cwd=cited_root, capture_output=True, text=True)
    if covered_run.returncode != 0 or not os.path.exists(os.path.join(cited_root, 'path-output.md')):
        print(f"  ✗ 自检：按路径引的那份 kb 文件补上小节清单之后 --cited 仍然拒绝，实际退 {covered_run.returncode}：{covered_run.stderr[-400:]}")
        print("     → 怎么办：路径形态认了、清单补上却还报缺，去看 check_cited() 比对清单里文件名的那一步。")
        return 1
    print("  ✓ 自检：正文只按路径引 kb 文件（没有编号）而清单里没有它时 --cited 退 5，补上小节清单才退 0（C234 按路径引的那一型）")
    os.makedirs(os.path.join(cited_root, '.claude/kb/experiments'))
    for name in ('1-节点大小vs可恢复性.md', '165-恢复重放有界穷举.md'):
        with open(os.path.join(cited_root, '.claude/kb/experiments', name), 'w', encoding='utf-8') as f:
            f.write('## 样本\n')
    declared_body = '<!-- doc-lint:not-numbers E1 -->\n这一轮判 E1（txg 顶格那一域） 站不站得住，压在 E165（样本） 上。\n'
    declared_hits = sorted(cited_kb_files(declared_body, cited_root))
    if declared_hits != ['.claude/kb/experiments/165-恢复重放有界穷举.md']:
        print(f"  ✗ 自检：正文用 doc-lint:not-numbers 声明了 E1，--cited 应当只认 E165 那一份实验页，实认 {declared_hits}")
        print("     → 怎么办：cited_kb_files() 要跳过 not_numbers_declared() 读出的记号、没声明的照认；QUOTE_KB_BREAK=ignore-not-numbers 设着的话这里本来就该红。")
        return 1
    fenced_body = '```\n<!-- doc-lint:not-numbers E1 -->\n```\n这一轮判 E1（样本） 与 E165（样本）。\n'
    fenced_hits = sorted(cited_kb_files(fenced_body, cited_root))
    if fenced_hits != ['.claude/kb/experiments/1-节点大小vs可恢复性.md', '.claude/kb/experiments/165-恢复重放有界穷举.md']:
        print(f"  ✗ 自检：代码围栏里的 not-numbers 声明不算，E1 与 E165 都应当认，实认 {fenced_hits}")
        print("     → 怎么办：not_numbers_declared() 要跳过围栏里的行（与 doc-lint 同读法）；QUOTE_KB_BREAK=not-numbers-in-fence 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：--cited 不把 doc-lint:not-numbers 声明的记号当编号（E1 不拉、E165 照拉），围栏里的声明不算")
    # 三语：同号的决策页有中文、英语、日语三份。严格读法只认中文那一份，清单里没有它的小节清单时 --cited 要求补它；宽读法照旧三份都给
    trilingual_chinese = '.claude/kb/decisions/13-验证路线.md'
    for name in ('13-验证路线.md', '13-verification-route.en.md', '13-検証ルート.ja.md'):
        with open(os.path.join(cited_root, '.claude/kb/decisions', name), 'w', encoding='utf-8') as f:
            f.write('## 样本\n')
    trilingual_body = '这一轮判 D13（验证路线） 站不站得住。\n'
    trilingual_checklist = os.path.join(cited_root, 'trilingual-checklist.md')
    with open(trilingual_checklist, 'w', encoding='utf-8') as f:
        f.write("### 小节清单：`.claude/kb/decisions/99-样本.md`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n| ## D99 样本 —— 已定 | 抄 | 样本 |\n")
    trilingual_strict = sorted(cited_kb_files(trilingual_body, cited_root))
    trilingual_lacking = check_cited(trilingual_checklist, trilingual_body, cited_root)
    trilingual_loose = sorted(path for path, _token in cited_kb_sources(trilingual_body, cited_root, loose=True))
    with open(trilingual_checklist, 'a', encoding='utf-8') as f:
        f.write(f"\n### 小节清单：`{trilingual_chinese}`\n\n| 小节 | 抄 / 不抄 | 理由 |\n|---|---|---|\n| ## 样本 | 不抄 | 样本 |\n")
    trilingual_covered = check_cited(trilingual_checklist, trilingual_body, cited_root)
    if trilingual_strict != [trilingual_chinese] or trilingual_lacking != [trilingual_chinese] or trilingual_covered or len(trilingual_loose) != 3:
        print(f"  ✗ 自检：同号有中英日三份时，--cited 的严格读法应当只认中文那一份 {trilingual_chinese}、清单里没有它的小节清单就要求补它、补上之后不再报，宽读法照旧三份都给；"
              f"实际严格读法认 {trilingual_strict}，没补时要求补 {trilingual_lacking}，补上之后还报 {trilingual_covered}，宽读法给 {trilingual_loose}")
        print("     → 怎么办：看 cited_kb_sources() 里 D / E 编号那一段：严格读法要先排掉以 TRANSLATION_SUFFIXES 结尾的译文再数；"
              "QUOTE_KB_BREAK=strict-counts-translations 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：同号有中英日三份时，--cited 的严格读法只认中文那一份、清单里没有它就要求补，宽读法照旧三份都给")
    # 不变量清单按类拆在登记的 kb 目录里（README.md 加类文件，形态照 crates/singlefs-checker/invariants/）：I 号按类号指到类文件
    invariants_directory = 'crates/sample/invariants'
    os.makedirs(os.path.join(cited_root, invariants_directory))
    with open(os.path.join(cited_root, KB_DIRECTORIES_REGISTRY), 'w', encoding='utf-8') as f:
        f.write(f'.claude/kb  # 样本：决策、实验、欠账仍在这里\n{invariants_directory}  # 样本：不变量按类拆在这里\n')
    for name, text in (('README.md', '# 不变量清单\n\n## 哪类放哪\n\n| 文件 | 类 |\n|---|---|\n| 02-根环.md | I-7 |\n| 03-journal.md | I-8 |\n'),
                       ('02-根环.md', '## I-7 根环 —— 类\n\n| 编号 | 简称 | 陈述 |\n|---|---|---|\n| I-7.4 | 样本 | 样本陈述 |\n'),
                       ('03-journal.md', '## I-8 journal —— 类\n\n| 编号 | 简称 | 陈述 |\n|---|---|---|\n| I-8.3 | 样本 | 样本陈述 |\n')):
        with open(os.path.join(cited_root, invariants_directory, name), 'w', encoding='utf-8') as f:
            f.write(text)
    class_hits = sorted(cited_kb_files('这一轮判 I-7.4（样本不变量） 与 I-8.3（样本不变量） 站不站得住。\n', cited_root))
    if class_hits != [f'{invariants_directory}/02-根环.md', f'{invariants_directory}/03-journal.md']:
        print(f"  ✗ 自检：正文点名 I-7.4 与 I-8.3，--cited 应当只认 02-根环.md 与 03-journal.md 两份类文件、不带 README.md，实认 {class_hits}")
        print("     → 怎么办：看 invariant_class_files() 与 invariant_files_for()：要在 .claude/kb-directories 登记的目录里按「| I-<类>.」表行找类文件；"
              "QUOTE_KB_BREAK=invariants-to-readme 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：不变量清单按类拆开时，--cited 把 I-7.4 与 I-8.3 指到各自的类文件，不带 README.md")
    # 退 5 的说明：点出是哪个编号引出的、给能照抄的 kb-sections.py 命令、写明接上的清单要逐行标抄不抄；不在的文件单说
    invariant_body = '这一轮判 I-7.4（样本不变量） 站不站得住。\n'
    invariant_sources = cited_kb_sources(invariant_body, cited_root)
    class_file = f'{invariants_directory}/02-根环.md'
    saved_directory = os.getcwd()
    os.chdir(cited_root)
    try:
        message = '\n'.join(lacking_checklist_message([class_file, '.claude/kb/nothere.md'],
                                                      invariant_sources, 'checklist.md', cited_root))
    finally:
        os.chdir(saved_directory)
    wanted = [f'{class_file}（正文里的 I-7.4 引出）', f'kb-sections.py {class_file} >> checklist.md',
              '逐行标「抄」或「不抄」', '.claude/kb/nothere.md 不在']
    if invariant_sources != [(class_file, 'I-7.4')] or any(piece not in message for piece in wanted):
        print(f"  ✗ 自检：退 5 的说明应当点出 I-7.4 引出了 02-根环.md、给照抄的 kb-sections.py 命令、写明逐行标抄不抄、单说不在的文件，"
              f"实际读出 {invariant_sources}，说明：\n{message}")
        print("     → 怎么办：看 lacking_checklist_message() 与 cited_kb_sources()；QUOTE_KB_BREAK=bare-lacking-hint 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：退 5 的说明点出引出那份文件的编号、给照抄的 kb-sections.py 命令、写明逐行标抄不抄，不在的文件单说")
    # 类号找不到类文件：退回不变量目录的 README.md，退 5 的说明点名是退回的
    fallback_sources = cited_kb_sources('这一轮判 I-10.1（没有类文件的样本号）。\n', cited_root)
    readme = f'{invariants_directory}/README.md'
    fallback_message = '\n'.join(lacking_checklist_message([readme], fallback_sources, 'checklist.md', cited_root))
    if fallback_sources != [(readme, 'I-10.1')] or '（正文里的 I-10.1 引出；I-10 类在登记的 kb 目录' not in fallback_message \
            or '退回 README.md' not in fallback_message:
        print(f"  ✗ 自检：正文点名 I-10.1 而没有 I-10 的类文件，应当退回 {readme} 并在退 5 的说明里点名是退回的，"
              f"实际读出 {fallback_sources}，说明：\n{fallback_message}")
        print("     → 怎么办：看 invariant_files_for() 的退回那一支与 lacking_checklist_message()；QUOTE_KB_BREAK=no-readme-fallback 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：类号找不到类文件时退回 README.md，退 5 的说明点名是退回的")
    # 登记位 .claude/kb-directories 的规整与拒绝（kb_directory_entries、kb_candidate_roots）
    registry_root = os.path.join(work, 'kb-directories-sample')
    for directory in ('.claude/kb', 'crates/x/invariants'):
        os.makedirs(os.path.join(registry_root, directory))
    os.makedirs(os.path.join(work, 'outside-the-repository'))

    def roots_with(registry_text):
        with open(os.path.join(registry_root, KB_DIRECTORIES_REGISTRY), 'w', encoding='utf-8') as f:
            f.write(registry_text)
        KB_DIRECTORIES_REFUSALS_REPORTED.clear()
        captured = io.StringIO()
        with contextlib.redirect_stderr(captured):
            roots = kb_candidate_roots(registry_root)
            kb_candidate_roots(registry_root)   # 同一个进程里第二次调，被拒的行不再报
        return roots, kb_directory_entries(registry_root)[1], captured.getvalue()

    roots, refused, report = roots_with('./crates/x/invariants/  # ./ 开头、/ 结尾\n.claude//kb  # 重复的 /\ncrates/./x/invariants  # 规整之后与第一行相同\n')
    if roots != ['.claude/kb', 'crates/x/invariants'] or refused or report:
        print(f"  ✗ 自检：登记位写成 ./ 开头、/ 结尾、中间重复 / 的三行，应当规整成 ['.claude/kb', 'crates/x/invariants']、一行不拒，"
              f"实际 {roots}，拒了 {refused}，stderr {report!r}")
        print("     → 怎么办：看 kb_directory_entries() 的规整（os.path.normpath）；QUOTE_KB_BREAK=kb-directories-raw 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：登记位写成 ./ 开头、/ 结尾、中间重复 / 的，规整成相对仓根的写法、去重")
    absolute = os.path.join(registry_root, 'crates/x/invariants')
    roots, refused, report = roots_with(f'{absolute}  # 绝对路径\n../outside-the-repository  # 走出仓\ncrates/../..  # 规整成 ..\n.  # 仓根\n'
                                        'crates/x/invariants  # 好的一行\n')
    wanted = [f'第 1 行「{absolute}」是绝对路径', '第 2 行「../outside-the-repository」规整成 ../outside-the-repository，走出了仓',
              '第 3 行「crates/../..」规整成 ..，走出了仓', '第 4 行「.」规整之后是仓根本身', '→ 怎么办：写成相对仓根的目录']
    if roots != ['crates/x/invariants'] or [number for number, _, _ in refused] != [1, 2, 3, 4] \
            or any(piece not in report for piece in wanted) or report.count('是绝对路径') != 1:
        print(f"  ✗ 自检：登记位写成绝对路径、../ 走出仓、规整成 ..、写成 . 的四行应当拒、在 stderr 逐行点名（同一行只报一次），只认好的那一行，"
              f"实际 {roots}，拒了 {refused}，stderr {report!r}")
        print("     → 怎么办：看 kb_directory_entries() 的拒绝那几支与 kb_candidate_roots() 的报法；QUOTE_KB_BREAK=kb-directories-raw 设着的话这里本来就该红。")
        return 1
    print("  ✓ 自检：登记位写成绝对路径、走出仓、仓根本身的行被拒，stderr 逐行点名并给出路，同一行只报一次")
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
    bounds = check_section_bounds(specs)
    if bounds:
        print(f"quote-kb: 节边界对不上 {len(bounds)} 条（pick() 与 kb-sections.py 各算了一遍，行区间不等）：", file=sys.stderr)
        for problem in bounds:
            print(f"    {problem}", file=sys.stderr)
        print("  → 回读比对是绿的不代表取全了：去源文件 grep -n '^#' 看这一节真正到哪一行，哪一边错了改哪一边（pick() 的栅栏与级别、kb-sections.py 的标题表）；节里有行首是 `#9` 这类（`#` 后没有空格）的行时，pick() 当它是一级标题而截断，那就改用 文件:A-B 行区间；"
              "改好之前别发这份材料，急用就把这一节改成 文件:A-B 行区间取并看一眼被抄内容的最后一行", file=sys.stderr)
        return 7
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
        unfounded = check_checklist_reasons(checklist, produced)
        if unfounded:
            print(f"quote-kb: 清单标了「不抄」、理由说正文已抄在别处，点名的节在出口里却找不到 {len(unfounded)} 处：", file=sys.stderr)
            for f, title, name in unfounded:
                print(f"    {f}：{title[:60]}——理由点名的「{name[:60]}」不在抄录块的标题里", file=sys.stderr)
            print("  → 去出口里数一眼那一节在不在：不在就把它补进取法（优先用 文件@标题），或者把理由改成真话（这一节没抄、为什么）", file=sys.stderr)
            return 8
    if cited:
        cited_text = open(cited, encoding='utf-8').read()
        lacking = check_cited(checklist, cited_text, os.getcwd())
        if lacking:
            for line in lacking_checklist_message(lacking, cited_kb_sources(cited_text, os.getcwd()), checklist, os.getcwd()):
                print(line, file=sys.stderr)
            return 5
    with open(dest, 'w', encoding='utf-8') as f:
        f.write('\n'.join(produced))
    tail = f"，清单里标「抄」的每一节都在" if checklist else ""
    if cited:
        tail += "，正文提到的每个 kb 文件都有清单"
    print(f"  ✓ {len(specs)} 段整抄进 {dest}，回读逐字节一致{tail}")
    return 0


if __name__ == '__main__':
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
