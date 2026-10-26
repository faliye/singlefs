#!/usr/bin/env python3
# admission: always 判的是此刻 decisions/ 下的正文，门禁 doc-decisions 格「决策分项清单与正文同步」每一轮都拿它的输出与索引页现比
# run-condition: none 只读仓里的决策正文，除了 python3 之外没有环境要求
"""从 decisions/ 下的正文抽出每条决策的分项清单，打印成 markdown。

**它存在的唯一理由是「同一个事实只许一处权威记录」**：分项的权威记录是各决策正文，
索引页只是它的投影。手抄一份就会漂，而漂了没有任何东西会发现——
`.claude/gate.d/doc-decisions.sh` 的格「决策分项清单与正文同步」拿本脚本的输出与索引页比对，不一致判红。

**每条决策的分项只有一套编号，分住两个小节**：`### 已定项` 与 `### 未定项`。
状态由**它住在哪一节**决定，不由行内文字决定——正文里一句「与 D16（发布语义） 已定的
checkpoint 序号怎么共存」曾让按行内关键字判状态的老版本把一条未定项判成已定。

**用法**：`gen-decision-items.py [项目根] [--status-cells]`。读 `<项目根>/.claude/kb/decisions/` 下的正文；不给项目根就取工作目录。
不从脚本位置定根：门禁 doc-decisions 起的是真仓里这一份、读的是样本根的 decisions/，它显式传自己的项目根。
项目根下没有 `.claude/kb/decisions/`、或里面除译文外一份正文都没有，判错退 1；未知参数、给了两个项目根判用法错退 2；
不是喂样本却设了弄坏开关（文件末尾「弄坏开关」一段），退 2。

**两种输出**：

| 怎么跑 | 打印什么 | 谁消费 |
|---|---|---|
| 不带 `--status-cells` | 索引页那段 `<!-- gen:decision-items -->` 生成块 | `.claude/gate.d/doc-decisions.sh` 的格「决策分项清单与正文同步」与格「未定项判过改不改新池新建文件的字节」 |
| `--status-cells` | 每行 `D<n>\t<状态列该写什么>\t<进度列该写什么>` | 同上，用来写回决策索引表的「状态」列与「进度」列 |

**状态列写的是分项计数**（`已定 6 项 / 未定 0 项`），不是三态词——
「这条决策有几个分项、其中几个还没定」看一眼就有数，而三态词看不出这个。
三态词的权威记录仍是各正文首行 `## D<n> 简称 —— 状态`，索引列不再抄它一遍。

**进度列写的是实现计数**（`已实现 3 项 / 未实现 1 项`，有无对象的再带 ` / 无对象 2 项`；没有分项写 `无对象`）。
每条分项的进度取它索引行末尾的规范标记 `**进度：已实现 / 未实现 / 无对象。**`，与 `**状态：…。**` 并排；
缺标记、或一行写了两个不同的进度，生成器报错退出——索引页上没有「漏判」这一档。
分项清单里每条写成 `—— 已定 · 已实现`。

**只读中文那一份**：decisions/ 下的 `<名>.en.md`、`<名>.ja.md` 是译文，不进清单（见 decision_body_files()）。

自证：门禁 doc-decisions 的样本 decision-items-sync-translation-green（decisions/ 下放着英语与日语译文，首行不是 `## D<n> 简称 —— 状态`）
走本脚本，要求照常出清单、不报首行读不出。

弄坏开关（只给样本自证证红用，设进环境；几项用逗号隔开）。只在门禁喂样本时认（sample_root_feeding()）：读的项目根（第一个参数，没给取工作目录）
是样本根——根里有 .gate-cells 或空文件 .gate-break-sample、这个根（真实路径）不是生成器所在的仓、也没设 GATE_STAGED_FROM，
与门禁共用库 .claude/gate.d/lib/stage-cells.sh 的 stage_cells_refuse_unsampled_break_switches 同一条判据。平常跑设了它，读参数之前就拒绝、退 2，
输出点名这个变量只在样本自证里认。
  GEN_DECISION_ITEMS_BREAK=read-translations  decision_body_files() 把译文也当正文读（改前的 glob）
      decision-items-sync-translation-green（期望退出 0，实测 1：生成器报「首行读不出」）
  GEN_DECISION_ITEMS_BREAK=cwd-root-unchecked  不认项目根参数、不判用法错，按工作目录读，决策目录不在或没有正文照样退 0、输出空（改前的样子）
      （开关只在样本根上认：下面的空目录里放空文件 .gate-break-sample，第一个参数给的项目根也是放了 .gate-break-sample 的样本根）
      在空目录里起、不带参数（期望退出 1、stderr 有「第一个参数给项目根」，实测 0）；
      在空目录里起、第一个参数给一份带决策正文的样本根（期望退出 0、输出里有「- **D」起头的行，实测 0、输出空）；
      带一个未知参数 --bogus（期望退出 2，实测 0）
"""
import os, re, glob, sys
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from project_preflight import preflight  # noqa: E402
preflight(__file__)
TRANSLATION_SUFFIX = re.compile(r'\.[a-z]{2}\.md$')
BREAK_VARIABLE = 'GEN_DECISION_ITEMS_BREAK'


def sample_root_feeding(arguments):
    """这一趟是不是门禁喂样本：读的项目根（参数里第一个不以 - 起头的，没给取工作目录）是样本根——根里有 .gate-cells 或 .gate-break-sample，
    这个根（真实路径）不是生成器所在的仓（本文件往上两级），也没设 GATE_STAGED_FROM。
    判据照门禁共用库 .claude/gate.d/lib/stage-cells.sh 的 stage_cells_refuse_unsampled_break_switches（弄坏开关只在样本根上认）。"""
    root = next((argument for argument in arguments if not argument.startswith('-')), None) or os.getcwd()
    real_root = os.path.realpath(root)
    own_repository = os.path.realpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', '..'))
    marked = any(os.path.isfile(os.path.join(real_root, marker)) for marker in ('.gate-cells', '.gate-break-sample'))
    return marked and real_root != own_repository and not os.environ.get('GATE_STAGED_FROM')


if os.environ.get(BREAK_VARIABLE) and not sample_root_feeding(sys.argv[1:]):
    print(f"  ✗ 环境里设了 {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]}：这个弄坏开关只在样本自证里认（门禁 doc-decisions 喂样本，读的项目根是样本根），"
          f"平常跑设了就拒绝，一份正文都没读\n"
          f"     → 怎么办：平常跑先 unset {BREAK_VARIABLE} 再起；要证红就经 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 喂门禁 doc-decisions 的样本"
          f"（样本根里有 .gate-cells 或空文件 .gate-break-sample）", file=sys.stderr)
    sys.exit(2)
BROKEN = set(os.environ.get(BREAK_VARIABLE, '').split(','))
USAGE = '用法：gen-decision-items.py [项目根] [--status-cells]（不给项目根就取工作目录）'

def clip(text, n):
    """截到 n 个字符，**但不许留下没闭合的括注**。

    「编号（简称）」被截在括号中间会让 doc-lint 判「括注没闭合」——
    而那条检查是对的：半个简称比没有简称更容易被误读。
    """
    t = text[:n]
    # 按长度截断落在一个 ASCII 词中间（例「SHA256」截成「SH」、「D22」截成「D2」）：半截整个去掉。
    # 半截的编号尤其危险——「D2」是另一个真实存在的编号，留着就成了一条指错对象的引用。
    if len(text) > n and re.match(r'[A-Za-z0-9._-]', text[n]) and re.search(r'[A-Za-z0-9._-]$', t):
        t = re.sub(r'[A-Za-z0-9._-]+$', '', t).rstrip()
    while t.count('（') > t.count('）'):
        t = t[:t.rindex('（')].rstrip()
    # 截断还可能在编号后面切掉它的简称，留下一个**裸引用**——doc-lint 同样判红，
    # 而它判得对：一个只剩符号的编号，含义可以被悄悄改掉而没有一个字看起来别扭
    # （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 5 条）。把这种尾巴一并去掉。
    # ⚠️ **只在真截断过时剥**（按长度截了，或上面剥过半个括注）：没截断的名字结尾那串是原文自己写的，剥掉就丢了原文。
    # 编号按 doc-lint 认编号的同一个形状剥：整词（`[A-Z]+-?数字(.数字)*`），前一个字符不是字母、数字、`.`、`_`、`-`。
    # 不带左边界时，截断正好停在「SHA256」「RAID5」之后会从词中间咬走「A256」「D5」，剩下半截「SH」「RAI」。
    # kb 里 `<!-- doc-lint:not-numbers … -->` 登记过的领域词（SHA256、RAID5、M1……）doc-lint 不当编号，不剥：
    # 剥了只是丢原文（三方判决 gate-fix-forks-r1、r2 的 T8）。
    if t == text:
        return t.rstrip()
    while True:
        tail = re.search(r'(?<![A-Za-z0-9._-])([A-Z]+-?\d+(?:\.\d+)*)\s*$', t)
        t2 = t[:tail.start()] if tail and tail.group(1) not in not_number_tokens() else t
        t2 = re.sub(r'[\s/、,，·的与和]+$', '', t2)
        if t2 == t:
            break
        t = t2
    return t.rstrip()


_NOT_NUMBER_TOKENS = None


def not_number_tokens():
    """kb 里 `<!-- doc-lint:not-numbers … -->` 标记登记的领域词（doc-lint 不把它们当编号）。按 cwd 下的 kb 读，读一次。
    读法与 doc-lint 同一套：代码围栏里的不算（围栏里「举例」写的一行声明不打开豁免），标记写在一行之内。"""
    global _NOT_NUMBER_TOKENS
    if _NOT_NUMBER_TOKENS is None:
        _NOT_NUMBER_TOKENS = set()
        for kb_path in glob.glob('.claude/kb/**/*.md', recursive=True):
            in_fence = False
            with open(kb_path, encoding='utf-8') as handle:
                for line in handle:
                    if re.match(r'^[ \t]*```', line):
                        in_fence = not in_fence
                        continue
                    if in_fence:
                        continue
                    for marker in re.finditer(r'<!-- *doc-lint:not-numbers([^>]*)-->', line):
                        _NOT_NUMBER_TOKENS.update(marker.group(1).split())
    return _NOT_NUMBER_TOKENS


def name_of(txt):
    name = re.sub(r'\*\*|`', '', txt)
    name = re.split(r'——|\||。', name)[0].strip().rstrip('：:')
    # 排版之后条目常写成「名字（日期，谁定的）：结论」——名字只到括注之前。
    # **只在括注以日期开头时切**，免得把「MAC 长度：80 位截断 vs 128 位」这种
    # 本来就带冒号的名字截断。
    name = re.sub(r'（\d{4}-\d{2}-\d{2}[^）]*）：.*$', '', name).strip()
    # 条目里那句「，见 D23（…）「已定项 1（…）：…」」是**指路，不是名字**。
    # 不切掉的话，名字尾巴会紧挨着生成器自己加的状态后缀，
    # 读起来像「已定项 1 —— 已定」——本轮实测有人（我）据此误判成重复标注。
    name = re.split(r'，见 |，详见 |。见 |，权威记录在', name)[0].strip()
    return clip(name, 46)


def harvest(sec):
    """取一节里**顶格的**编号项：表格行优先，其次编号列表。返回 [(编号, 名字那一格, 整行)]。

    只看该节的**索引表**，即第一个 `####` 子标题之前那一段——
    子标题之下是各分项各自的论证，里面另有编号列表与表格，那些不是分项。
    整行带回去是为了取进度标记：它写在定案那一格的末尾，不在名字格里。
    """
    cut = re.search(r'^#{4}\s', sec, re.M)
    top = sec[:cut.start()] if cut else sec
    rows = re.findall(r'^(\|\s*(\d+)\s*\|([^|]*)\|.*)$', top, re.M)
    if rows:
        return [(n, name, row) for row, n, name in rows]
    return [(n, txt, n + '. ' + txt) for n, txt in re.findall(r'^(\d+)\.\s+(.*)$', top, re.M)]


PROGRESS_MARK = re.compile(r'进度：\s*\*{0,2}(已实现|未实现|无对象)')


def progress_of(row, where, number):
    """一条分项索引行的进度标记：恰好一个，取它的值；缺了或写了两个不同的值就报错退出，不猜。"""
    marks = sorted(set(PROGRESS_MARK.findall(row)))
    if len(marks) == 1:
        return marks[0]
    if not marks:
        raise SystemExit(
            f"  ✗ {where} 的分项 {number} 没写进度标记：{row[:60]}\n"
            f"     → 在这一行的 **状态：…。** 之后补上 **进度：已实现。**、**进度：未实现。** 或 **进度：无对象。**"
            f"（判法在 decisions.md「进度」一段）")
    raise SystemExit(
        f"  ✗ {where} 的分项 {number} 写了两个不同的进度标记（{'、'.join(marks)}）：{row[:60]}\n"
        f"     → 一行只许一个「进度：」标记，删掉多余的那个")


def items_of(body, where):
    """返回 [(编号, 名字, 状态, 进度)]，两节合起来按编号排。where 是「文件（决策号）」，只用在报错里。"""
    res = []
    for head, st in (('已定项', '已定'), ('未定项', '**未定**')):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, body, re.M | re.S)
        if not m:
            continue
        got = harvest(m.group(1))
        # 没有编号项就报错，不许猜。老版本在这里退而取该节第一行当一条无号分项，
        # 生成出「- -. 校验和粒度与随机小读的张力」——**编号位是个破折号**。
        # 它既不是分项的身份（引用不了「D4 未定项 -」），也不会红，
        # 于是那一节没写索引表这件事在索引页上躺了很久没人看见。
        if not got:
            raise SystemExit(
                f"  ✗ {where} 的「### {head}」一节里取不到编号项\n"
                f"     → 该节要有索引表（`| # | 分项 | 状态 |`）或编号列表，"
                f"每条分项一行、编号连号不重排")
        for n, txt, row in got:
            res.append((n, name_of(txt), st, progress_of(row, where, n)))
    res.sort(key=lambda r: int(r[0]))
    return res


def decision_body_files():
    """decisions/ 下的决策正文，只取中文那一份。`<名>.en.md`、`<名>.ja.md` 是译文（登记在 .claude/trilingual-documents，
    规矩在 .claude/skills/trilingual/SKILL.md），首行与分项都是译文的写法，当正文读会报首行读不出、或把同一条决策数两遍。
    判法与 .claude/gate.d/lib-trilingual.py 的 LANGUAGE_SUFFIXED 同口径：带两个小写字母语言后缀的 .md 不是中文那一份。"""
    found = sorted(glob.glob('.claude/kb/decisions/*.md'))
    if 'read-translations' in BROKEN:   # 弄坏开关：译文也当正文读（改前的样子）
        return found
    return [f for f in found if not TRANSLATION_SUFFIX.search(os.path.basename(f))]


def parsed_arguments(arguments):
    """返回 (项目根或 None, 是否 --status-cells)；未知参数、两个项目根退 2。"""
    root, status_cells = None, False
    for argument in arguments:
        if argument == '--status-cells':
            status_cells = True
        elif argument.startswith('-') or root is not None:
            what = f"认不出参数「{argument}」" if argument.startswith('-') else f"给了两个项目根：{root} 与 {argument}"
            print(f"  ✗ 用法错：{what}\n     → {USAGE}", file=sys.stderr)
            sys.exit(2)
        else:
            root = argument
    return root, status_cells


if 'cwd-root-unchecked' in BROKEN:   # 弄坏开关：不认项目根、不判用法错，按工作目录读（改前的样子）
    project_root, cells_only = None, '--status-cells' in sys.argv[1:]
else:
    project_root, cells_only = parsed_arguments(sys.argv[1:])
decisions_directory = os.path.join(project_root or os.getcwd(), '.claude', 'kb', 'decisions')
if 'cwd-root-unchecked' not in BROKEN:
    if not os.path.isdir(decisions_directory):
        sys.exit(f"  ✗ 项目根下没有决策目录 {decisions_directory}：一份正文都读不到，输出会是空的\n"
                 f"     → 第一个参数给项目根，或在项目根里起（{USAGE}）")
    os.chdir(project_root or os.getcwd())
    if not decision_body_files():
        sys.exit(f"  ✗ {decisions_directory} 下除译文外一份决策正文都没有，输出会是空的\n"
                 f"     → 第一个参数给项目根，或在项目根里起（{USAGE}）；项目根对了而正文真没有，先从 git 里找回 decisions/ 下的正文")

lines, cells = [], []
for f in decision_body_files():
    s = open(f, encoding='utf-8').read()
    title = s.split('\n', 1)[0]
    # 破折号前有没有空格两种都有（D14 没有），不能只认一种
    mm = re.match(r'## (D\d+) (.+?)\s*—— *(.+)$', title)
    # 首行读不出就报错，不许跳过：跳过的那条决策从分项清单里整条消失，doc-decisions 格「决策分项清单与正文同步」只会报一句
    # 「索引表里有行而 decisions/ 下没有它的正文」，格「未定项判过改不改新池新建文件的字节」则一条未定项都不替它查。
    if not mm:
        raise SystemExit(
            f"  ✗ {f} 首行读不出 `## D<n> 简称 —— 状态`：{title[:40]!r}\n"
            f"     → 把首行写成 `## D3 空间分配 —— 半定（…）`，首行之前不许有空行或 BOM；"
            f"decisions/ 下除译文（<名>.en.md、<名>.ja.md）外只放决策正文")
    num, name, status = mm.group(1), mm.group(2).strip(), mm.group(3).strip()
    body = s.split('\n## 历史版本')[0]
    its = items_of(body, f"{f}（{num}）")
    open_n = sum(1 for _, _, st, _ in its if '未定' in st)
    done_n = sum(1 for _, _, _, pg in its if pg == '已实现')
    todo_n = sum(1 for _, _, _, pg in its if pg == '未实现')
    none_n = sum(1 for _, _, _, pg in its if pg == '无对象')
    kind = re.match(r'(已定|半定|待定)', status)
    kind = kind.group(1) if kind else status[:6]
    head = f"- **{num}（{name}）** —— {kind}"
    head += f"（分项 {len(its)}，其中未定 {open_n}）" if its else "（无分项）"
    lines.append(head)
    # 一条决策没有分项时，「已定 0 项 / 未定 0 项」会被读成「什么都没定」——
    # 那一格改写成三态词，不含糊其辞。
    progress_cell = (f"已实现 {done_n} 项 / 未实现 {todo_n} 项" + (f" / 无对象 {none_n} 项" if none_n else "")
                     if its else "无对象")
    cells.append(f"{num}\t" + (f"已定 {len(its) - open_n} 项 / 未定 {open_n} 项"
                               if its else f"无分项 · 整条{kind}") + f"\t{progress_cell}")
    for n, nm, st, pg in its:
        lines.append(f"  - {n}. {nm} —— {st} · {pg}")
print('\n'.join(cells if cells_only else lines))
