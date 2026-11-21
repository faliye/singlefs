#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：fixture-claims、fixture-registrations-excluded、kb-registry、gate-structure、stage-order、directory-layout、crate-roles、committed-file-size、git-hooks-installed、stage-location-before-cd。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# ── 格 fixture-claims：阶段头部声称的判别力样本必须真的存在；全仓文件名与样本目录里的日期不许是编的
#
# 判据，任一条不成立判红：
#   ① 声称只认显式标记：每道阶段（门禁目录顶层的 *.sh）头部注释块里（第一行代码之前）有且只有一行 `# gate-fixture: yes` 或
#      `# gate-fixture: none <为什么装不进沙箱，至少 8 个字>`，判法只读这一行，头部别的句子里写没写 fixtures/<它> 都不算声称。
#      yes：fixtures/<阶段文件名>/ 要在，再过下面第 ③ 条；none：那个目录要不在（在就红：写了 none 却有样本目录）；
#      没写标记、写了不止一行、写的不是这两种，都红。
#      none 还要登记进豁免表 .claude/gate.d/fixture-none-exempt.tsv（一行一道 `<阶段文件名>  # 理由`，形态照 .claude/preflight-exclude）：
#      写了 none 没登记、登记了而头部写的是 yes、登记指向不存在的阶段、同一道登记两行、理由不到 8 个字，都红。
#      豁免表只缩不涨：补出样本的阶段改成 yes 同时删它那一行，不往表里加新行去绕开这一格；门禁不判涨，靠 review；
#   ② .claude/gate.d/fixtures/ 下每个目录都对应一个在的阶段文件：孤儿样本目录没有东西会跑它，摆在那里看着却像验过了；
#   ③ 样本目录下每个子目录都是一份样本（与上游 stage-selftest.sh 同一个认法），都要有 expect，至少一份；按那道门禁的格名表
#      （gate-structure-check.py 的 header_cells；没有格名表的整道算一格）逐格数，每格至少一份红（exit=1）一份绿（exit=0）。
#      一份样本算给哪几格见 cell_gaps 的文档字符串；绿的一侧按设计只能退 77 的格，那份样本的 expect 里写一行
#      not-green-by-design=<格名> <为什么>（理由至少 8 个字）才算绿；
#   ④ 全仓文件名里的日期，与 fixtures/ 下正文里的日期，都要落在可能的区间里：判据 source 上游 lib.sh 的 date_out_of_range，
#      下界取本仓第一个提交（取不到退回上游的 SOP_START_DATE）。正文只扫 fixtures 的理由写在第 ④ 条那段代码旁。
#      第 ④ 条在一个 python 进程里判（不为每个文件、每个日期起子 shell 与 date）：判据只有上游 lib.sh 的 date_out_of_range 那一份，这里经它取一次「今天」与下界，
#      再拿四个边界日期（明天、今天、下界、下界前一天）的答案与进程里的判法逐个对，对不上报判据失效；弄坏开关 date-judge-drifted 把进程里的下界多算一天。
#      工具版本串里的日期不判：一行里写成「(<7 到 40 位十六进制> <年-月-日>)」的（rustc -V、cargo -V 的样子），那个日期是那一版工具的发布日，
#      不是写样本的日子（例 formal-verification.sh 样本里的 rustc-version）；成功句另报不判的个数。弄坏开关 tool-version-dates-judged 下照判。
# 射程：判的是声称与目录对不对得上，判不了样本本身有没有判别力（归 stage-selftest.sh）；不声称（none）的阶段只有豁免表里登记了的才过，
#   不再有「既不声称也没样本、照记通过」的阶段。
# 不是 git 仓时第 ④ 条整条没判，前三条都过也退 77。第 ④ 条正文唯一的排除是这一格自己的样本（fixture-claims-*），这一道别的格的样本照旧扫。
#
# 样本：fixture-claims-red 摆一个假的 gate.d，前三条各犯一次，必须判红；fixture-claims-green 都干净（含一道头部写着「装不进 fixtures/<它>/…」
#   的否定句、标了 none 也登记了的阶段：按散文子串认声称会把它当成声称、判红）。
#   fixture-claims-unnumbered-red 的 gate.d 里一道不带编号的 tooling-claims.sh 标了 yes 而目录不在、另一道带编号的标 none 并已登记，必须点名前者判红。
#   fixture-claims-unmarked-red 一道阶段头部只有「装不进 fixtures/<它>/…」的否定句、没写标记，一道标记写在第一行代码之后，必须都红在「没写标记」。
#   fixture-claims-marker-malformed-red 的三道阶段分别是 none 理由太短、写了两行标记、写的是 maybe，必须逐道点名判红。
#   fixture-claims-none-with-directory-red 的阶段标了 none 也登记了，而样本目录里红绿齐全，必须红在「写了 none 却有样本目录」。
#   fixture-claims-exempt-red 摆三处豁免表对不上：标了 none 没登记、登记了而头部写的是 yes、登记指向不存在的阶段，另有重复行与理由太短的行，必须逐处判红。
#   fixture-claims-cells-red 的 cells-uncovered.sh 三格，样本按 <格>-red / <格>-green 起名：alpha 缺绿；beta 只在点名两格而 want 只写了 gamma 红的样本里
#   （不算给 beta，缺红）；gamma 的绿样本退 77 而 not-green-by-design 的理由太短（缺绿），必须逐格列名判红。
#   fixture-claims-green 的 cells-covered.sh 两格没有裸 red / green：alpha 一红一绿，beta 的红来自点名两格而 want 写了「beta：红」的样本，
#   绿样本退 77 并写了够长的 not-green-by-design，必须判绿。fixture-claims-green 另放一份 rustc-version（版本串里的日期早于沙箱第一个提交 7 天以上），
#   必须不判、成功句报「工具版本串里的日期 1 个」。
#   第 ④ 条的坏日期由 fixture-claims-red/setup.sh 在临时仓里现造，不摆进仓里（摆进来会被这一条自己扫到）；setup.sh 要 git init 再提交一个
#   真实日期的提交，否则 project_start_date 取不到值，下界那一支走不到。
cell_fixture_claims() {
LIB="$(cd "$STAGE_DIRECTORY/../.." && pwd)/.claude/singlefs-ai-sop/scripts/lib.sh"
python_rc=0
python3 - "${CODE_TOOLING_BREAK:-}" "$STAGE_REPOSITORY/scripts/gate/gate-structure-check.py" <<'PY' || python_rc=$?
import glob, importlib.util, os, re, sys
sys.dont_write_bytecode = True

broken = sys.argv[1]
STAGE_DIR = ".claude/gate.d"
FIXTURE_DIR = os.path.join(STAGE_DIR, "fixtures")
try:
    # 格名表的读法只有一份：gate-structure-check.py 的 header_cells（文件头注释块里的 # gate-cell: 行），stage-owners 格也导入它
    specification = importlib.util.spec_from_file_location("gate_structure_check", sys.argv[2])
    structure = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(structure)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 导入不了 {sys.argv[2]}：{error}")
    print("     → 怎么办：格名表经它的 header_cells 读；它随仓走，被删了或改坏了就从 git 找回来，别在这一格另写一份读法。")
    sys.exit(1)

# 没有格名表的阶段整道算一格，名字用这个占位
IMPLICIT_CELL = "（整道）"
# 按设计绿的一侧只能退 77（或别的非 0 码）的格，在那份绿样本的 expect 里写一行「not-green-by-design=<格名> <为什么>」
BY_DESIGN_KEY = "not-green-by-design="
BY_DESIGN_MINIMUM_REASON = 8
# 声称只认头部注释块里的这一行标记；none 要登记进豁免表（只缩不涨，门禁不判涨）
MARKER_PATTERN = re.compile(r"^#\s*gate-fixture:\s*(.*?)\s*$")
MARKER_NAME = "# gate-fixture:"
NONE_MINIMUM_REASON = 8
EXEMPT_TABLE = os.path.join(STAGE_DIR, "fixture-none-exempt.tsv")
EXEMPT_MINIMUM_REASON = 8


def header_markers(path):
    """头部注释块（到第一行代码之前，与 header_cells 同一个认法）里的标记行：[(行号, 标记后面的文字)]。"""
    found = []
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if number == 1 and line.startswith("#!"):
                continue
            if line.strip() and not line.lstrip().startswith("#"):
                break
            match = MARKER_PATTERN.match(line.strip())
            if match:
                found.append((number, match.group(1)))
    return found


def read_exempt_table():
    """豁免表：({阶段文件名: 第一次出现的行号}, 表本身的问题列表)。不存在的表当空表。"""
    entries, problems = {}, []
    if not os.path.isfile(EXEMPT_TABLE):
        return entries, problems
    with open(EXEMPT_TABLE, encoding="utf-8", errors="replace") as handle:
        for number, raw in enumerate(handle, 1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            entry, hash_mark, reason = line.partition("#")
            entry, reason = entry.strip(), reason.strip()
            if not hash_mark or len(reason) < EXEMPT_MINIMUM_REASON:
                problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 的理由不到 {EXEMPT_MINIMUM_REASON} 个字（一行写成「<阶段文件名>  # <为什么装不进沙箱>」）")
            if entry in entries:
                problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 重复登记（第 {entries[entry]} 行已经有了）")
            else:
                entries[entry] = number
    return entries, problems


def read_sample(path):
    """一份样本：(目录名, 退出码, want 列表, 点名的格或 None（没有 .gate-cells）, 按设计不绿的格 → 理由)。"""
    with open(os.path.join(path, "expect"), encoding="utf-8", errors="replace") as handle:
        lines = handle.read().splitlines()
    exit_code = next((line[len("exit="):].strip() for line in lines if line.startswith("exit=")), "")
    wants = [line[len("want="):] for line in lines if line.startswith("want=")]
    by_design = {}
    for line in lines:
        if line.startswith(BY_DESIGN_KEY):
            parts = line[len(BY_DESIGN_KEY):].split(None, 1)
            if parts:
                by_design[parts[0]] = parts[1].strip() if len(parts) > 1 else ""
    marked = None
    marker = os.path.join(path, ".gate-cells")
    if os.path.isfile(marker):
        with open(marker, encoding="utf-8", errors="replace") as handle:
            marked = [entry for entry in (line.split("#", 1)[0].strip() for line in handle) if entry]
    return os.path.basename(path), exit_code, wants, marked, by_design


def summary_says(wants, cell, verdict):
    """样本的 want 里有共用库汇总那一行「  <格名>：<红 / 本次未跑>」。"""
    pattern = re.compile(rf"^\s*{re.escape(cell)}：{verdict}")
    return any(pattern.match(want) for want in wants)


def cell_gaps(cells, samples):
    """每一格至少一份红（exit=1）一份绿（exit=0，或按设计不绿并写了理由）；返回缺的逐格说明。
    一份样本算给哪几格：.gate-cells 点名的格；没有 .gate-cells 的算给整道全部格。
    红样本只点名一格的算给那一格；点名几格的（或整道跑的）只算给 want 里写了「<格名>：红」的格。
    退 0 的样本算给它跑的每一格，want 里写了「<格名>：本次未跑」的那几格除外。"""
    red = {cell: [] for cell in cells}
    green = {cell: [] for cell in cells}
    short_reasons = []
    for kind, exit_code, wants, marked, by_design in samples:
        covered = [cell for cell in (marked if marked is not None else cells) if cell in red]
        for cell in covered:
            if exit_code == "1":
                if len(covered) == 1 or summary_says(wants, cell, "红"):
                    red[cell].append(kind)
            elif exit_code == "0" and not summary_says(wants, cell, "本次未跑"):
                green[cell].append(kind)
            elif cell in by_design and exit_code != "1":
                if len(by_design[cell]) >= BY_DESIGN_MINIMUM_REASON:
                    green[cell].append(kind)
                else:
                    short_reasons.append(f"{kind} 的 {BY_DESIGN_KEY}{cell} 理由不到 {BY_DESIGN_MINIMUM_REASON} 个字")
    gaps = []
    for cell in cells:
        missing = [side for side, found in (("红", red[cell]), ("绿", green[cell])) if not found]
        if missing:
            gaps.append(f"{cell} 缺{'、'.join(missing)}")
    return gaps + short_reasons


judged_cells = 0
uncovered = []
# 阶段是门禁目录顶层全部 *.sh，与 gate.sh 起的是同一批
stage_pattern = "[0-9][0-9]-*.sh" if broken == "numbered-stages-only" else "*.sh"
stages = sorted(path for path in glob.glob(os.path.join(STAGE_DIR, stage_pattern)) if os.path.isfile(path))
if not stages:
    print(f"  ✗ {STAGE_DIR} 下一个 *.sh 阶段都没有")
    print("     → 怎么办：门禁目录搬了位置就同步改这个阶段里的路径；扫到 0 个阶段而报绿，与判过了一模一样。")
    sys.exit(1)

claimed_without_directory, hollow = [], []
unmarked, malformed, none_with_directory = [], [], []
yes_stages, none_stages = [], []
for stage in stages:
    name = os.path.basename(stage)
    directory = os.path.join(FIXTURE_DIR, name)
    markers = header_markers(stage)
    if broken == "claims-by-prose":
        # 弄坏开关：标记一概不看，整份文件里出现 fixtures/<它> 就算声称（否定句也算），没声称的看目录在不在
        prose = re.search(rf"fixtures/{re.escape(name)}", open(stage, encoding="utf-8").read()) is not None
        if not os.path.isdir(directory):
            if prose:
                claimed_without_directory.append(f"{name}：头部写着 fixtures/{name}/…，而 {directory} 不存在")
            continue
        yes_stages.append(name)
    else:
        if not markers:
            if broken != "unmarked-stage-passes":  # 弄坏开关：没写标记的阶段跳过，照记通过
                unmarked.append(f"{name}：头部注释块里没有 {MARKER_NAME} 行")
            continue
        if len(markers) > 1:
            malformed.append(f"{name}：头部有 {len(markers)} 行 {MARKER_NAME}（第 {'、'.join(str(number) for number, _ in markers)} 行），只许一行")
            continue
        number, value = markers[0]
        word = value.split(None, 1)[0] if value else ""
        if value == "yes":
            yes_stages.append(name)
            if not os.path.isdir(directory):
                claimed_without_directory.append(f"{name}：头部标了 yes（声称有样本），而 {directory} 不存在")
                continue
        elif word == "none":
            reason = value[len(word):].strip()
            if len(reason) < NONE_MINIMUM_REASON:
                malformed.append(f"{name}：第 {number} 行 none 后面的理由不到 {NONE_MINIMUM_REASON} 个字（写成「{MARKER_NAME} none <为什么装不进沙箱>」）")
                continue
            none_stages.append(name)
            if os.path.isdir(directory):
                none_with_directory.append(f"{name}：头部写了 none，而 {directory} 在")
            continue
        else:
            malformed.append(f"{name}：第 {number} 行写的是「{value}」，只认 yes 与 none <为什么>")
            continue
    if broken == "bare-red-green-only":
        # 弄坏开关：退回只认裸 red / green 目录、不按格判（第二波起样本都叫 <格>-red 这种名字，这样判整批都成空壳）
        kinds = [kind for kind in ("red", "green") if os.path.isdir(os.path.join(directory, kind))]
        if not kinds:
            hollow.append(f"{name}：{directory} 在，里面一个 red / green 都没有")
            continue
        for kind in kinds:
            if not os.path.isfile(os.path.join(directory, kind, "expect")):
                hollow.append(f"{name}：{directory}/{kind}/ 里没有 expect，判别力自检会整个跳过它")
        continue
    # 与上游 stage-selftest.sh 同一个认法：样本目录下每个子目录都是一份样本，带 expect 的都跑
    samples = []
    for kind in sorted(entry for entry in os.listdir(directory) if os.path.isdir(os.path.join(directory, entry))):
        expect_path = os.path.join(directory, kind, "expect")
        if not os.path.isfile(expect_path):
            hollow.append(f"{name}：{directory}/{kind}/ 里没有 expect，判别力自检会整个跳过它")
            continue
        samples.append(read_sample(os.path.join(directory, kind)))
    if not samples:
        hollow.append(f"{name}：{directory} 在，里面一份带 expect 的样本都没有")
        continue
    cells = [cell for _number, cell, _title in structure.header_cells(stage)] or [IMPLICIT_CELL]
    judged_cells += len(cells)
    gaps = cell_gaps(cells, samples)
    if gaps:
        uncovered.append(f"{name}：{'；'.join(gaps)}")

orphans = []
if os.path.isdir(FIXTURE_DIR):
    for entry in sorted(os.listdir(FIXTURE_DIR)):
        if os.path.isdir(os.path.join(FIXTURE_DIR, entry)) and not os.path.isfile(os.path.join(STAGE_DIR, entry)):
            orphans.append(f"{entry}：样本目录在，而 {STAGE_DIR}/{entry} 不存在")

# 豁免表与头部的 none 逐项对上（claims-by-prose 开关下没有这一段）
exempt_problems = []
if broken != "claims-by-prose":
    exempt_entries, exempt_table_problems = read_exempt_table()
    exempt_problems.extend(exempt_table_problems)
    stage_names = {os.path.basename(stage) for stage in stages}
    for name in none_stages:
        if name not in exempt_entries:
            exempt_problems.append(f"{name}：头部写了 none，而 {EXEMPT_TABLE} 里没有它这一行")
    for entry, number in exempt_entries.items():
        if entry not in stage_names:
            exempt_problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 不是 {STAGE_DIR} 下的阶段")
        elif entry in yes_stages:
            exempt_problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 登记了，而它头部写的是 yes")

failed = False
if unmarked:
    failed = True
    print(f"  ✗ {len(unmarked)} 个阶段头部没写 {MARKER_NAME} 标记，判不出它声称不声称有判别力样本：")  # gate-lint:summary
    for entry in unmarked:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：在头部注释块里（第一行代码之前）加一行「{MARKER_NAME} yes」（样本目录 fixtures/<阶段名>/ 在，每格有红有绿），")
    print(f"               或「{MARKER_NAME} none <为什么装不进沙箱，至少 {NONE_MINIMUM_REASON} 个字>」并把这道阶段登记进 {EXEMPT_TABLE}。")
    print("               头部别的句子里写 fixtures/<它> 不算声称，写在第一行代码之后的标记也不认。")
if malformed:
    failed = True
    print(f"  ✗ {len(malformed)} 个阶段的 {MARKER_NAME} 标记写得不对：")  # gate-lint:summary
    for entry in malformed:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：每道阶段头部只留一行，写「{MARKER_NAME} yes」或「{MARKER_NAME} none <为什么装不进沙箱，至少 {NONE_MINIMUM_REASON} 个字>」。")
if claimed_without_directory:
    failed = True
    print(f"  ✗ {len(claimed_without_directory)} 个阶段的头部声称有判别力样本，而目录不存在：")  # gate-lint:summary
    for entry in claimed_without_directory:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：两条出路，别两边都不做。① 把样本建起来（fixtures/<阶段名>/ 下按格放 <格>-red / <格>-green 各一份加 expect），")
    print("               让那句声称变成真的；② 这个阶段确实装不进沙箱（要真起虚机、要 cargo 真跑），就把头部那行改成")
    print(f"               「{MARKER_NAME} none <为什么装不进>」，并把它登记进 {EXEMPT_TABLE}（一行 `<阶段文件名>  # 理由`）。")
if none_with_directory:
    failed = True
    print(f"  ✗ {len(none_with_directory)} 个阶段头部写了 none，却有样本目录：")  # gate-lint:summary
    for entry in none_with_directory:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：目录里有样本就是有判别力样本，把头部改成「{MARKER_NAME} yes」并删掉豁免表里那一行；")
    print("               样本是废的就连目录一起删，别让 none 和样本目录同时摆着。")
if exempt_problems:
    failed = True
    print(f"  ✗ {len(exempt_problems)} 处豁免表 {EXEMPT_TABLE} 与阶段头部对不上：")  # gate-lint:summary
    for entry in exempt_problems:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：头部写 none 的阶段在表里有且只有一行「<阶段文件名>  # <理由，至少 {EXEMPT_MINIMUM_REASON} 个字>」；表里每一行都要指向在的阶段、")
    print("               且那道阶段头部写的是 none；阶段补出样本改成 yes 的，同一次改动把表里它那一行删掉。")
    print("               豁免表只缩不涨：不往表里加新行去绕开这一格，装得进沙箱的阶段建样本（这条门禁不判涨，靠 review）。")
if orphans:
    failed = True
    print(f"  ✗ {len(orphans)} 个孤儿样本目录，没有对应的阶段：")  # gate-lint:summary
    for entry in orphans:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：阶段退役时把它的样本目录一起删掉；阶段改名了就把目录改成新名字——")
    print("               留着的目录没有任何东西会跑它，而它摆在那里看着就像那个阶段验过了。")
if hollow:
    failed = True
    print(f"  ✗ {len(hollow)} 个样本目录是空壳：")  # gate-lint:summary
    for entry in hollow:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：red 至少要有一条 want=（失败信息里必须出现的片段），green 写 exit=0；")
    print("               少了 expect，判别力自检会跳过那一份，而目录在、看着像验过了。")
if uncovered:
    failed = True
    print(f"  ✗ {len(uncovered)} 道门禁有格缺红或缺绿样本（按格名表逐格数，样本算给哪几格看 .gate-cells 与 expect）：")  # gate-lint:summary
    for entry in uncovered:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：给缺的那一格补一份样本 fixtures/<门禁>/<格>-red（或 -green）/，放 .gate-cells 只点名那一格，")
    print("               expect 写 exit=1 与至少一条 want=（红）或 exit=0（绿）；一份红样本点名几格的，want 里逐格写「  <格名>：红（退出 1）」。")
    print("               绿的一侧按设计只能退 77 的格（例：探针全对上就记本次未跑），在那份绿样本的 expect 里写一行")
    print(f"               「{BY_DESIGN_KEY}<格名> <为什么，至少 {BY_DESIGN_MINIMUM_REASON} 个字>」，别为了凑绿把那一格改成退 0。")
if failed:
    sys.exit(1)

print(f"  ✓ 阶段头部声称的判别力样本都在（判了 {len(yes_stages) + len(none_stages)} 道：yes {len(yes_stages)} 道、none {len(none_stages)} 道（逐个列名见下）），"
      f"{FIXTURE_DIR} 下没有孤儿目录，也没有缺 expect 的空壳；有样本的阶段 {judged_cells} 格逐格都有一红一绿")
print(f"    yes {len(yes_stages)} 道：{' '.join(yes_stages) or '（没有）'}")
print(f"    none {len(none_stages)} 道（都登记在 {EXEMPT_TABLE}）：{' '.join(none_stages) or '（没有）'}")
PY

# ── 第 ④ 条：日期不许是编的 ────────────────────────────
# lib.sh 自带 set -euo pipefail，所以整段关在子 shell 里，判定经文件带出来（command-safety.md「子 shell 里的赋值传不回父进程」）。
date_report="$(mktemp)"
trap 'rm -f "$date_report"' EXIT
date_rc=0
date_skipped=0
(
  source "$LIB"
  require_date_arithmetic
  git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'SKIP\t不是 git 仓，列不出要扫的文件\n'; exit 0; }
  if start_date="$(project_start_date "$(pwd)")" && [[ -n "$start_date" ]]; then
    start_source="本仓第一个提交"
  else
    start_date="$SOP_START_DATE"; start_source="取不到本仓第一个提交，退回上游的 SOP_START_DATE"
  fi
  # 阳性对照：上游把 date_out_of_range 改名或删掉时，if 条件里 command not found 的 127 会被读成「没越界」而静默报绿；先拿确知越界的日期试一次。
  if ! date_out_of_range "1970-01-01" "$start_date" >/dev/null 2>&1; then
    printf 'BROKEN\t判据对 1970-01-01 都报不出越界：date_out_of_range 被改名、删掉，或边界口径变了\n'
    exit 0
  fi
  # 唯一的排除：这一格自己的样本，里面的越界日期就是判别力；跳过几份现算着报进成功句。
  own_fixture=".claude/gate.d/fixtures/${STAGE_NAME}/fixture-claims-"
  # 判据只有上游 lib.sh 的 date_out_of_range 那一份：这里只经它取一次「今天」与下界、再拿四个边界日期的答案交给 python（它在进程里逐个日期判，不为每个日期起子 shell 与 date）；
  # python 对这四个日期的答案与 lib.sh 的不同，就是两边的判据分叉了，报 BROKEN
  today="$(latest_today)"; lower_bound="$(date_lower_bound "$start_date")"
  probe_arguments=()
  for probe_date in "$(date -d "$today + 1 day" +%F)" "$today" "$lower_bound" "$(date -d "$lower_bound - 1 day" +%F)"; do
    if probe_why="$(date_out_of_range "$probe_date" "$start_date")"; then probe_arguments+=("$probe_date" "$probe_why"); else probe_arguments+=("$probe_date" ""); fi
  done
  python3 - "$own_fixture" "$start_date" "$start_source" "$today" "$lower_bound" "$DATE_GRACE_DAYS" "${CODE_TOOLING_BREAK:-}" "${probe_arguments[@]}" <<'PY_DATE_CLAIMS'
import calendar, os, re, subprocess, sys
sys.stdout.reconfigure(errors="surrogateescape")
own_fixture, start_date, start_source, today, lower_bound, grace_days, broken = sys.argv[1:8]
probes = list(zip(sys.argv[8::2], sys.argv[9::2]))
DATE = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")
MONTH_FILE = re.compile(r"(^|/)([0-9]{4}-(0[1-9]|1[0-2]))\.md\Z")
VERSION_DATE = re.compile(r"\([0-9a-f]{7,40} ([0-9]{4}-[0-9]{2}-[0-9]{2})\)")


def out_of_range(checked):
    """上游 lib.sh date_out_of_range 的判据：晚于今天，或早于下界（第一个提交往前放宽 DATE_GRACE_DAYS 天），交为什么；在范围内交 None。"""
    if checked > today:
        return f"晚于今天（{today}，按最晚的时区算）"
    if lower_bound and (checked <= lower_bound if broken == "date-judge-drifted" else checked < lower_bound):
        return f"早于这个仓第一个提交（{start_date}）{grace_days} 天以上"
    return None


drift = [f"{day}：lib.sh 说「{answer or '在范围内'}」，这一格说「{out_of_range(day) or '在范围内'}」" for day, answer in probes if (out_of_range(day) or "") != answer]
if drift:
    print("BROKEN\t这一格里日期的判法与上游 date_out_of_range 的答案对不上：" + "；".join(drift))
    sys.exit(0)
# 路径走 NUL：git 默认把非 ASCII 路径整条引起来并转义成八进制，报出来的那一行既搜不到也对不上 want；--cached 与 --others 各列一次的同一路径只留一条
listing = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], stdout=subprocess.PIPE, check=True).stdout
paths = sorted({os.fsdecode(raw) for raw in listing.split(b"\0") if raw})
scanned = dated = bodies = body_dates = version_dated = skipped_own = 0
unreadable_bodies = []
for path in paths:
    # --cached 列的是索引：git mv 过又被 reset 掉暂存时索引里还留着旧名、文件已不在，不滤掉会判红不存在的文件。
    if not os.path.exists(path):
        continue
    scanned += 1
    # 名字是 <年-月>.md 的文件 DATE 抓不到，单独判：整月与允许区间没有交集（两个端点都越界）才算越界；拿月初比会误判合法的当月。
    month_found = MONTH_FILE.search(path)
    if month_found:
        month = month_found.group(2)
        year_number, month_number = int(month[:4]), int(month[5:])
        month_end = f"{month}-{calendar.monthrange(year_number, month_number)[1]:02d}"
        dated += 1
        if out_of_range(f"{month}-01") is not None and out_of_range(month_end) is not None:
            print(f"BADNAME\t{path}\t{month}（整月）\t{out_of_range(month_end)}")
    names_dates = sorted(set(DATE.findall(path)))
    if not names_dates:
        continue
    dated += 1
    for found in names_dates:
        why = out_of_range(found)
        if why is not None:
            print(f"BADNAME\t{path}\t{found}\t{why}")
# 正文只扫 fixtures：真 kb 正文里的日期常常说的是外部的事（别家项目、RFC 历史、引编造日期当例子），
# 而下界假设「本仓的事不可能早于本仓第一个提交」对它们不成立。
BYTE_DATE = re.compile(DATE.pattern.encode())
BYTE_VERSION_DATE = re.compile(VERSION_DATE.pattern.encode())
for body in (path for path in paths if path.startswith(".claude/gate.d/fixtures/") and os.path.exists(path)):
    if body.startswith(own_fixture):
        skipped_own += 1
        continue
    # 读不了与「没有日期」不是一回事：读不了的不计进 bodies，逐个列进没扫的清单；二进制样本按文本读（grep -a）
    try:
        with open(body, "rb") as handle:
            lines = handle.read().split(b"\n")
    except OSError:
        unreadable_bodies.append(body)
        continue
    bodies += 1
    hits = set()
    versions = set()
    for number, line in enumerate(lines, 1):
        if not line:
            continue
        for found in BYTE_DATE.findall(line):
            hits.add((number, found.decode("ascii")))
        # 工具版本串里的日期（rustc / cargo -V 的「(<提交的十六进制> <年-月-日>)」）是那一版工具的发布日，不是写样本的日子：照「行号、日期」记下，下面不判
        if broken != "tool-version-dates-judged":
            for found in BYTE_VERSION_DATE.findall(line):
                versions.add((number, found.decode("ascii")))
    # 整行去重（同一行同一日期只算一次），不按行号去重：按行号会丢掉同一行上的第二个日期。
    for number, found in sorted(hits):
        if (number, found) in versions:
            version_dated += 1
            continue
        body_dates += 1
        why = out_of_range(found)
        if why is not None:
            print(f"BADBODY\t{body}:{number}\t{found}\t{why}")
print("COUNT\t" + "\t".join(str(part) for part in (scanned, dated, bodies, body_dates, start_date, skipped_own, own_fixture, start_source, version_dated))
      + "\t" + (" ".join(unreadable_bodies) or "（没有）"))
PY_DATE_CLAIMS
) > "$date_report" 2>&1 || date_rc=$?

if grep -qE '^BAD(NAME|BODY)' "$date_report"; then
  date_rc=1
  if grep -q '^BADNAME' "$date_report"; then
    echo "  ✗ $(grep -c '^BADNAME' "$date_report") 个文件名里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADNAME' "$date_report")
  fi
  if grep -q '^BADBODY' "$date_report"; then
    echo "  ✗ $(grep -c '^BADBODY' "$date_report") 处样本正文里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADBODY' "$date_report")
  fi
  echo "     → 怎么办：写真实发生的那一天——记录写它是哪天记下的，产物写它是哪天跑的，"
  echo "       样本写它所属 fixture 的创建日：git log --reverse --format=%ad --date=short -- <fixture 目录>；"
  echo "       样本里要表示先后两次事件的，就用创建日和它的次日，别用 2026-01-01 / 2026-01-02 这种占位。"
  echo "       查不出来也别填占位日期：占位日期读起来和真日期一模一样，后面每一个引用它的人都会当真。"
  echo "       红样本非要一个编造日期不可的，让 fixtures/<阶段>/red/setup.sh 在临时仓里现造，别摆进仓里——"
  echo "       摆进来它会被这一条自己扫到，红的是样本而不是被测的东西。"
elif grep -q '^BROKEN' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期判据失效了，这一条什么也没在判："
  grep '^BROKEN' "$date_report" | cut -f2 | sed 's/^/      /'   # gate-lint:detail
  echo "     → 怎么办：判据是上游 .claude/singlefs-ai-sop/scripts/lib.sh 的 date_out_of_range，本仓不另写一份。"
  echo "       先 grep 那个文件确认函数还在、名字没变；同步过副本就读一遍 CHANGELOG 看边界口径改没改。"
  echo "       上游真改了名，改这一条去跟上新名字，别在本仓另抄一份判据——两套装置算同一个量就会分叉。"
elif grep -q '^SKIP' "$date_report"; then
  printf '  ⊘ 日期这一条未跑：%s\n' "$(grep '^SKIP' "$date_report" | cut -f2)"
  date_skipped=1
elif ! grep -q '^COUNT' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期这一条没跑完，报告里没有计数行："
  sed 's/^/      /' "$date_report"   # gate-lint:detail
  echo "     → 怎么办：上面是子 shell 的原样输出；多半是 lib.sh 取不到或 git 列不出文件，按它说的修。"
else
  IFS=$'\t' read -r _ scanned dated bodies body_dates start_date skipped_own own_fixture start_source version_dated unreadable_list < <(grep '^COUNT' "$date_report")
  # 计数缺一个就判红：报着空数的成功句照样绿，比没有这一条更糟。
  if ! [[ "$scanned" =~ ^[0-9]+$ && "$dated" =~ ^[0-9]+$ && "$bodies" =~ ^[0-9]+$ && "$body_dates" =~ ^[0-9]+$ && "$skipped_own" =~ ^[0-9]+$ && "$version_dated" =~ ^[0-9]+$ ]]; then
    date_rc=1
    echo "  ✗ 日期这一条的计数行残缺，报不出检查了多少项："
    grep '^COUNT' "$date_report" | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：子 shell 多半在数完之前就退出了，上面那行是它吐出的原样；"
    echo "       扫到 0 项也不是通过，报着空数的成功句与判过了一模一样（show-me-test.md）。"
  else
    printf '  ✓ 日期都可能是真的（文件名：%s 个文件里 %s 条带日期；样本正文：%s 份文件里 %s 个日期；下界 %s 往前宽 7 天，下界取自%s）\n' \
      "$scanned" "$dated" "$bodies" "$body_dates" "$start_date" "$start_source"
    printf '    没扫的 %s 份：%s* 下的文件——这一格自己的样本，里面现造的越界日期就是它的判别力（C441（全仓清扫工具会吃掉自己的判别力样本））\n' \
      "$skipped_own" "$own_fixture"
    printf '    工具版本串里的日期 %s 个（「(<提交的十六进制> <年-月-日>)」，那一版工具的发布日）不判\n' "$version_dated"
    printf '    读不了、没扫的样本正文：%s\n' "$unreadable_list"
  fi
fi
if [[ "$python_rc" -ne 0 || "$date_rc" -ne 0 ]]; then
  exit 1
fi
if ((date_skipped)); then
  echo "  ⊘ 本次无对象可判（第 ④ 条）：前三条判过了，第 ④ 条整条没跑，这一格不报通过"
  exit 77
fi
exit 0
}

# ── 格 fixture-registrations-excluded：样本目录里有编号登记位的阶段，都要在 .claude/doc-lint-exclude 里有一行
#
# 判据，任一条不成立判红：
#   ① 扫 .claude/gate.d/fixtures/<阶段>/ 下递归的每一份 .md（代码围栏里的不算，与 doc-lint 取登记位同一个认法）：行首是 `## D<n> ` 或 `## E<n> `（编号登记标题），
#      或有一行含 doc-lint:registry（登记表标记），就是这一道阶段的样本里有登记位。有登记位的阶段，.claude/doc-lint-exclude 里要有一行指到
#      .claude/gate.d/fixtures/<阶段>/（末尾斜杠可有可无；指到它上面的目录也算）。缺的判红，点名阶段与它第一份带登记位的样本文件:行。
#   ② 排除表里指到 .claude/gate.d/fixtures/<阶段>/… 而 fixtures 下没有这个阶段目录的行判红，点名表里的行号。
#      排除表自己的形状（理由、目录在不在、下面有没有 .md、路径合不合法）由 doc-lint 的 J 段判，这里不重判；只判「有登记位却没排除」与「排除了 fixtures 下不存在的阶段」。
# 为什么：各道门禁的判别力样本各自造带登记标题的决策页与实验页，编号互相撞（样本里的 D94、D99 在好几份里同时登记）。样本目录各是一个沙箱，
#   撞号只在 doc-lint 把两份一起扫时才伤人，所以不判「各样本里的编号两两不同」，只判「有登记位的阶段整道排在 doc-lint 之外」；
#   新加一道带登记位的样本而没进排除表，会在整轮门禁的 doc-lint 里炸成「多处登记位」，这一格把它提前到改样本的当场。
# 判不了：登记位写成别的形状（`### D<n>`、两位数以外的前缀字母、非 .md 文件里造的）；doc-lint 的登记标题要求带破折号，这里不要求，宁可多拦。
# 弄坏开关 CODE_TOOLING_BREAK=registrations-unscanned 把登记位一概认成没有：fixture-registrations-excluded-red 判绿，fixture-registrations-excluded-green 里「2 道有登记位」那条 want 找不到。
# 样本（登记标题都在 setup.sh 里现写，静态样本文件里不放真的登记标题，免得本仓的 doc-lint 扫到样本目录就多出登记位）：
#   fixture-registrations-excluded-red 三道阶段：doc-sample.sh 有 `## D5 ` 标题且表里没有它、table-sample.sh 有 doc-lint:registry 标记且表里没有它、
#     covered-sample.sh 有 `## E7 ` 标题而表里有它；必须点名前两道与各自的文件:行，不许点名第三道；
#   fixture-registrations-excluded-stale-red 表里一行指到 fixtures 下不存在的阶段 gone-sample.sh，必须点名表里的行号；
#   fixture-registrations-excluded-green 四道阶段：两道有登记位且都在表里（一行带斜杠、一行不带）、一道没有登记位且不在表里、一道只在围栏里写了登记标题且不在表里，判绿并报扫了 4 道、2 道有登记位。
cell_fixture_registrations_excluded() {
[[ -d .claude/gate.d/fixtures ]] || { echo "  ! 没有 .claude/gate.d/fixtures，这一格无对象可判"; exit 77; }
python3 - "${CODE_TOOLING_BREAK:-}" <<'PY'
import os, re, sys
broken = sys.argv[1]
FIXTURES = ".claude/gate.d/fixtures"
EXCLUDE = ".claude/doc-lint-exclude"
HEADING = re.compile(r"^## [DE][0-9]+ ")
FENCE = re.compile(r"^[ \t]*```")
MARKER = "doc-lint:registry"

stages = sorted(name for name in os.listdir(FIXTURES) if os.path.isdir(os.path.join(FIXTURES, name)))
if not stages:
    print("  ! .claude/gate.d/fixtures/ 下一个阶段目录都没有，这一格无对象可判")
    sys.exit(77)


def first_registration(stage):
    """这一道阶段的样本里第一份带登记位的 .md：(相对路径, 行号, 是哪一种)；没有就 None。"""
    if broken == "registrations-unscanned":
        return None
    for directory, subdirectories, files in os.walk(os.path.join(FIXTURES, stage)):
        subdirectories[:] = sorted(name for name in subdirectories if name != ".git")
        for name in sorted(files):
            if not name.endswith(".md"):
                continue
            path = os.path.join(directory, name)
            fenced = False
            with open(path, encoding="utf-8", errors="replace") as handle:
                for number, line in enumerate(handle, 1):
                    if FENCE.match(line):
                        fenced = not fenced
                        continue
                    if fenced:
                        continue
                    if HEADING.match(line):
                        return path, number, "编号登记标题"
                    if MARKER in line:
                        return path, number, "登记表标记"
    return None


# 排除表：与 doc-lint 同一个读法（`<相对目录>  # 理由`，去掉末尾斜杠）；没有 # 的行、空路径是 doc-lint J 段的事，这里不收
entries = []
if os.path.isfile(EXCLUDE):
    with open(EXCLUDE, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            if not line.strip() or line.lstrip().startswith("#") or "#" not in line:
                continue
            path = line.split("#", 1)[0].strip()
            if path.startswith("./"):
                path = path[2:]
            path = path.rstrip("/")
            if path:
                entries.append((number, path))


def covered(stage):
    target = FIXTURES + "/" + stage
    return any(target == path or target.startswith(path + "/") for _number, path in entries)


registered, unregistered, uncovered = [], [], []
for stage in stages:
    found = first_registration(stage)
    if found is None:
        unregistered.append(stage)
        continue
    registered.append(stage)
    if not covered(stage):
        uncovered.append((stage, found))
stale = []
for number, path in entries:
    if path.startswith(FIXTURES + "/"):
        stage = path[len(FIXTURES) + 1:].split("/")[0]
        if stage not in stages:
            stale.append((number, path, stage))

if uncovered or stale:
    if uncovered:
        print(f"  ✗ {len(uncovered)} 道阶段的样本里有编号登记位，却不在 {EXCLUDE} 里（扫了 {len(stages)} 道，{len(registered)} 道有登记位）：")  # gate-lint:summary
        for stage, (path, number, kind) in uncovered:
            print(f"     {FIXTURES}/{stage}/（第一份：{path}:{number}，{kind}）")  # gate-lint:detail
        print(f"     → 怎么办：在 {EXCLUDE} 加一行 {FIXTURES}/<阶段>/  # 理由（理由写这一道样本造了带登记标题的页、与真表同号不同名，扫它只会报登记冲突）；")
        print("               不想排除，就让样本里的登记位在 setup.sh 里现写（静态样本文件里不放登记标题和登记表标记），或改成不带登记位的写法。")
    if stale:
        print(f"  ✗ {len(stale)} 行排除表指到 {FIXTURES}/ 下不存在的阶段（fixtures 下现有 {len(stages)} 道）：")  # gate-lint:summary
        for number, path, stage in stale:
            print(f"     {EXCLUDE}:{number}  {path}（没有阶段目录 {stage}）")  # gate-lint:detail
        print("     → 怎么办：阶段改了名就把这一行的路径改成现在的名字，样本目录删了就删掉这一行；留着的排除项会让人以为还有样本在被绕开。")
    sys.exit(1)
print(f"  ✓ 扫了 {len(stages)} 道阶段的样本，{len(registered)} 道有登记位、都在排除表里（{'、'.join(registered) if registered else '无'}）；"
      f"另 {len(unregistered)} 道没有登记位，在不在表里都不判；排除表里指到 fixtures 下的 {sum(1 for _n, p in entries if p.startswith(FIXTURES + '/'))} 行都指得到阶段")
PY
}

# ── 格 kb-registry：kb 目录里的每一份都要在 CLAUDE.md 的「项目本地事实」表里有一行
#
# 判据，任一条不成立判红：
#   ① .claude/kb/ 根目录下每一份 .md，都要在 CLAUDE.md「## 项目本地事实」表里被按路径点名；
#   ② .claude/kb/ 下每一个子目录也要被点名（带斜杠的路径，例如 .claude/kb/decisions/）；
#   ③ 那一节里点到的每一个 .claude/kb/… 路径都要真的存在，带 <…> 占位的也算一个路径。
# 「登记」只认表里一行的第一格写的路径、逐字相等：表外散文里提到的不算，一行 .claude/kb/ 也不替它下面的每一份登记。
# 判不了那一行写得对不对，要人看。
# 样本：kb-registry-red 放一份没登记的 kb 文件与一行指向空处的登记，必须判红；它的表里另有一行只写 .claude/kb/、散文里另提一份文件，
#   这两处都不许替那两份没登记的文件登记；kb-registry-green 两边对齐。
cell_kb_registry() {
python3 - "$MANIFEST_LIBRARY" <<'PY'
import glob, os, re, sys, types

manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)

KB = ".claude/kb"
MANIFEST = "CLAUDE.md"
SECTION = "## 项目本地事实"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

if not os.path.isdir(KB):
    print(f"  ! 没有 {KB}，这一格无对象可判")
    sys.exit(77)
if not os.path.isfile(MANIFEST):
    fail(f"找不到 {MANIFEST}", ["怎么办：项目说明改了名就同步改这个阶段里的路径。"])

text = open(MANIFEST, encoding="utf-8").read()
start = text.find(SECTION)
if start < 0:
    fail(f"{MANIFEST} 里没有「{SECTION}」这一节", [
        f"怎么办：这一节是 kb 各文件职责的登记位。改了标题就同步改这个阶段里的 SECTION，别让这道闸扫空。",
    ])
end = text.find("\n## ", start + len(SECTION))
section = text[start:end if end > 0 else len(text)]
registered = set(re.findall(r"`(\.claude/kb/[^`]*)`", section))
# 登记只认表里一行的第一格
table_lines = [line for line in section.split("\n") if line.strip().startswith("|")]
first_cells = set()
for line in table_lines:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    first_cells |= set(re.findall(r"`(\.claude/kb/[^`]*)`", cells[0])) if cells else set()
if not registered:
    fail(f"「{SECTION}」那一节里一个 `.claude/kb/…` 路径都没点到", [
        "怎么办：扫到 0 项而报绿，与判过了一模一样。那一节的表每行第一格写路径，用反引号包起来。",
    ])

on_disk = set()
for path in sorted(glob.glob(f"{KB}/*.md")):
    if re.search(r'\.[a-z]{2}\.md$', path):
        continue
    on_disk.add(path)
for entry in sorted(os.listdir(KB)):
    full = os.path.join(KB, entry)
    if os.path.isdir(full):
        on_disk.add(full + "/")

def is_registered(path):
    # 逐字等于表里某一行的第一格才算登记；前缀不算——一行 `.claude/kb/` 会让每一项都满足前缀
    return path in first_cells

def points_at_something(entry):
    return os.path.exists(entry) or os.path.exists(entry.rstrip("/"))

missing, dangling = manifest.two_way(sorted(on_disk), sorted(registered),
                                     is_registered=is_registered, exists=points_at_something)

failed = False
if missing:
    failed = True
    print(f"  ✗ kb 里 {len(missing)} 份没有在 {MANIFEST} 的「{SECTION}」表里登记：")  # gate-lint:summary
    for path in missing:
        print(f"      {path}")  # gate-lint:detail
    print(f"     → 怎么办：往那张表加一行，第一格写路径（反引号包起来）、第二格写它是什么、归谁读、与别处什么关系。")
    print("               写不出那一行，多半说明这份文件该并进已有的某一份，或者压根不该单开。")
if dangling:
    failed = True
    print(f"  ✗ 那张表里 {len(dangling)} 个路径指向不存在的东西：")  # gate-lint:summary
    for entry in dangling:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：文件搬家或改名了就同步改那一行（.claude/rules/path-moves.md）；删掉了就把那一行撤掉——")
    print("               指向空处的登记比没有登记更糟，它让人以为那一格有人管着。")
if failed:
    sys.exit(1)

print(f"  ✓ kb 里 {len(on_disk)} 项（{len([p for p in on_disk if p.endswith('.md')])} 份 .md、"
      f"{len([p for p in on_disk if p.endswith('/')])} 个子目录）都在 {MANIFEST} 的「{SECTION}」表里登记着，"
      f"表里 {len(registered)} 个路径也都指得到东西")
PY
}

# ── 格 gate-structure：每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2
#
# 判据与起法都在 scripts/gate/gate-structure-check.py（规则：.claude/rules/verification-reference.md「门禁的结构」），这一格只调它、按它的退出码记：
#   它另判 source 了 preflight.sh 的每道门禁，preflight 那一行的下一行是守卫（找不到 preflight 时当场打 ✗ 与出路、退 2，不让 set -- 清空参数之后全跑），
#   起 --list 与 --check 时每次放进它自己的会话、超时与退出之后收整组（scripts/gate/lib_process_group.py）。
#   守卫这一判并进这一格、不另立一格：比过上游 preflight-lint.py（判 preflight 那一行在不在、它之前只许哪几种行，上游副本不就地改，守卫只能挂在那一行之后）
#   与 research-gate-lint 格（判拒绝带出路与 shell 纪律，不看门禁的开头结构）；守卫管的正是 --list 与 --check 认不认参数，与这一格判的是同一批对象、同一件事。
#   0 绿；77（门禁目录里一道都没有）本次未跑；别的（1 结构不对、2 参数错、78 它自己的准入与运行条件没满足）都按红记。
# 脚本取这一道所在的仓里那一份（取自 $0 的目录，不取项目根），判的是被判的根：样本仓里放坏的脚本碰不到它。判不了的见它文件头「判不了的」。
# 样本：gate-structure-red 的 gate.d 里一道没有格名表、一道 --check 写错格名照跑退 0、一道 --list 打的与格名表不一致，必须逐道点名判红；
#   gate-structure-green 两道都照规矩（格名表、--list 与表一致、--check 写错退 2），判绿并报查了 2 道。
#   preflight-guarded-red 的 gate.d 里一道 source 了 preflight.sh、preflight 那一行（第 6 行）之后没有守卫（样本树带替身 .claude/scripts/preflight.sh，跑起来照常，只有守卫那一样不对），
#   必须点名「unguarded.sh:6：source preflight 之后没有守卫」判红；preflight-guarded-green 同一道补上守卫，判绿并报查了 1 道。
cell_gate_structure() {
structure_check="$STAGE_REPOSITORY/scripts/gate/gate-structure-check.py"
[[ -f "$structure_check" ]] || { echo "  ✗ 找不到 $structure_check"; echo "     → 怎么办：它随仓走（scripts/gate/gate-structure-check.py），被删了就从 git 找回来；别在这一格另写一份判法。"; exit 1; }
structure_exit=0
# --list --force 的结果落进本趟共用的临时目录，stage-owners 格读同一份（各道阶段只被起一次 --list）；没有共用目录时设空、不落不读
GATE_STRUCTURE_CHECK_LIST_CACHE="${CODE_TOOLING_RUN_SCRATCH:+$CODE_TOOLING_RUN_SCRATCH/stage-list}" python3 "$structure_check" "$ROOT" || structure_exit=$?
[[ "${CODE_TOOLING_BREAK:-}" == structure-exit-ignored ]] && structure_exit=0
case "$structure_exit" in
  0|77) exit "$structure_exit" ;;
  *)
    echo "  ✗ 门禁结构不对：gate-structure-check.py 退 $structure_exit（逐道列在上面）"
    echo "     → 怎么办：照它上面那一句改；退 2 是参数错，退 78 是它自己的准入与运行条件没满足，按它打的出路办。"
    exit 1
    ;;
esac
}

# ── 格 stage-order：环境检查那一道排在门禁目录里全部跑测试的阶段之后
#
# 判据：环境检查那一道（ENVIRONMENT_STAGE）要排在门禁目录里每一道跑测试的阶段后面；次序用共享 gate.sh 起本地阶段的同一条 find … | sort 现算。
#   「跑测试」：经 code_of 去掉注释与出路文字之后，代码里匹配 TEST_RUNNING（cargo test / build / run / nextest / bench、几份起测试的脚本、qemu-system、herd7）。
# 为什么：环境检查判的是它开跑那一刻的机器（残留 loop / dm 设备、挂载、qemu 进程、宿主盘），排在它后面的阶段编的、建的、留下的，这一趟看不见；
#   次序只由文件名定，改一个名就能悄悄挪到它前面。
# 判不了：用别的写法起测试的阶段（命令拼在变量里、经没列的脚本间接起），要把那个脚本名加进 TEST_RUNNING；排序跟跑这一格时的 locale。
# 样本：stage-order-red 的 gate.d 里 tests-run.sh 起 cargo test、排在 test-environment.sh 后面，必须点名它判红；同一份里排在后面、
#   只在注释与 echo 里提到 cargo test 的 zz-docs.sh 不算，不许点名。stage-order-green 把跑测试的那一道改名成 model-scenarios.sh，
#   排到环境检查前面，判绿并报 2 道跑测试的阶段。
ENVIRONMENT_STAGE="test-environment.sh"
# ── 格 directory-layout：research/ 与 scripts/ 的目录形态（.claude/rules/directory-layout.md）
# 判法只在 scripts/gate/directory-layout-check.py（用这一道所在仓里那一份），被判的根照传；它退 77（两个目录都不在）这一格就退 77。
# 样本：directory-layout-green 里 research/ 两个子目录与说明文件、scripts/ 一个子目录带说明文件；directory-layout-red 在同样的架子上多一份 scripts/kb/run.log。
cell_directory_layout() {
python3 "$STAGE_REPOSITORY/scripts/gate/directory-layout-check.py" "$ROOT"
}

# ── 格 crate-roles：crates/ 下各包登记了角色、按角色放该放的东西（.claude/rules/directory-layout.md「crates/：各包放什么」）
# 判法只在 scripts/gate/crate-roles-check.py（用这一道所在仓里那一份），被判的根照传；它退 77（crates/ 不在）这一格就退 77。
# 滞后表 .claude/gate.d/crate-roles-lag.tsv 只缩不涨：基准那一版经 exempt_table_baseline 取（lib-lag-baseline.py show），取到交 --lag-baseline；
#   基准里没有这张表或不在 git 仓里交 --lag-baseline-absent（成功句写明没比）；取不到交 --lag-baseline-unreadable（判红）。
# 样本：crate-roles-green 是一个小 crates/：产品、验证、测试支撑、实验装置各有，验证包 src/bin/ 里两个登在放行表里的工具与一个登在滞后表里的装置，
#   一个空库测试包（两个测试目标与一个装置逐个登在滞后表里），测试支撑包有一个包加根 Cargo.toml exclude 点名的别的工程两个依赖者，判绿；
#   crate-roles-empty-library-tests-red 多一个库为空、带两个测试目标的 -tests 包；crate-roles-product-experiment-binary-red 多一份产品包的 src/bin/e1_x.rs；
#   crate-roles-product-depends-on-experiment-red 产品包 [dev-dependencies] 依赖实验包；三份各红在点名的那一条。
#   crate-roles-product-depends-on-support-red 一个产品包经 [dependencies]、另一个经 [build-dependencies] 依赖测试支撑包，红两条 product-support-dependency。
#   crate-roles-lag-grown-red 的 setup.sh 在样本根 git init、提交一版少一行的滞后表、再把那一行加回来：滞后表比基准涨了，红。
cell_crate_roles() {
crate_roles_base="$(mktemp)" || { echo "  ✗ 建不了放基准那一版滞后表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${crate_roles_base:?}"' EXIT
exempt_table_baseline .claude/gate.d/crate-roles-lag.tsv "$crate_roles_base"
# 弄坏开关 CODE_TOOLING_BREAK=crate-roles-baseline-unpassed：不交基准，只缩不涨不比
if [[ "${CODE_TOOLING_BREAK:-}" == crate-roles-baseline-unpassed ]]; then base_state="none"; fi
case "$base_state" in
  table) python3 "$STAGE_REPOSITORY/scripts/gate/crate-roles-check.py" "$ROOT" --lag-baseline "$crate_roles_base" ;;
  unreadable:*) python3 "$STAGE_REPOSITORY/scripts/gate/crate-roles-check.py" "$ROOT" --lag-baseline-unreadable "${base_state#unreadable:}" ;;
  *) python3 "$STAGE_REPOSITORY/scripts/gate/crate-roles-check.py" "$ROOT" --lag-baseline-absent "基准里还没有这张表，或不在 git 仓里" ;;
esac
}

# ── 格 committed-file-size：被提交的东西里没有超过 100 MiB 的文件（单文件超过 GitHub 的推送上限，推不上去，事后只能重做提交）
# 判法只在 scripts/gate/large-file-check.py（用这一道所在仓里那一份，上限常量 SINGLE_FILE_SIZE_LIMIT_BYTES 全仓只在那里定义），被判的根照传。
#   甲  被判的根（gate.sh --staged 时是 HEAD 加暂存区的临时树）里 git 跟踪的每个文件；
#   乙  当前分支相对上游还没推上去的每笔提交里新增或改过的 blob（拦绕过门禁提交的，和先提交大文件、后一笔提交又删掉的）；
#       没有上游时乙这一半本次未跑：判法往 $GATE_NOT_RUN_FILE 报一行，成功句写明乙没跑，不报成判过。
#   红行逐个列路径与大小，乙另列提交号；出路是 gzip -n -9 压成 .gz 入库、读它的地方改成认 .gz、已进没推提交的重做提交。
# 样本：不真造 100 MiB 的文件。样本根放 .large-file-limit（一个字节数，判法的 --limit-bytes）把门限压小；被判的根是这一道所在的仓或 --staged 的临时树时不认
#   （与 .gate-cells 同一种认法，judged_root_is_sample）。五份样本的 setup.sh 各建一个小 git 仓，门限 1024 字节：
#   committed-file-size-green          3 个跟踪的文件，edge.bin 恰好 1024 字节（等于放行），上游在基准提交、1 笔没推的小提交：甲查 3 个，乙查 1 笔提交 1 个 blob；
#   committed-file-size-red            甲红：基准提交里的 old-big.out（2048）与已 git add 未提交的 staged-big.bin（3000）都点名，汇总括号里只有「甲」；没跟踪的 untracked-big.bin 不判；
#   committed-file-size-unpushed-red   乙红：没推的第一笔提交加 big.bin（2048）、第二笔又删了，HEAD 的树干净，乙点名第一笔提交与 big.bin，汇总括号里只有「乙」；
#   committed-file-size-detached-red   同上，最后 git checkout --detach（--staged 临时树的样子），靠指着 HEAD 的本地分支找上游；
#   committed-file-size-no-upstream-green  没设上游：打「乙（还没推的提交）本次未跑」，甲照判，整格退 0，成功句写「乙 本次未跑」。
# 弄坏开关 CODE_TOOLING_BREAK=committed-size-<项>（折成 LARGE_FILE_CHECK_BREAK=<项>）：
#   committed-size-tracked-half-skipped   甲不判：committed-file-size-red 判绿（期望退出 1，实测 0）；
#   committed-size-unpushed-half-skipped  乙不判：committed-file-size-unpushed-red、committed-file-size-detached-red 判绿；
#   committed-size-limit-inclusive        恰好等于上限的也算超：committed-file-size-green、committed-file-size-no-upstream-green 判红；
#   committed-size-no-upstream-as-pass    没有上游时乙当成判过（查了 0 笔）：committed-file-size-no-upstream-green 里「乙（还没推的提交）本次未跑」那条 want 找不到；
#   committed-size-detached-unresolved    游离的 HEAD 一律当没有上游：committed-file-size-detached-red 判绿。
cell_committed_file_size() {
  local limit_arguments=() break_item="" sample_limit
  local checker="$STAGE_REPOSITORY/scripts/gate/large-file-check.py"
  [[ -f "$checker" ]] || { echo "  ✗ 找不到 $checker"; echo "     → 怎么办：它随仓走（scripts/gate/large-file-check.py），被删了就从 git 找回来；--staged 时它要先暂存。"; exit 1; }
  if judged_root_is_sample && [[ -f .large-file-limit ]]; then
    sample_limit="$(tr -d "[:space:]" < .large-file-limit)"
    limit_arguments=(--limit-bytes "$sample_limit")
  fi
  case "${CODE_TOOLING_BREAK:-}" in committed-size-*) break_item="${CODE_TOOLING_BREAK#committed-size-}" ;; esac
  LARGE_FILE_CHECK_BREAK="$break_item" python3 "$checker" "$ROOT" --report-as "$STAGE_NAME 的 committed-file-size 格" ${limit_arguments[@]+"${limit_arguments[@]}"}
}

# ── 格 git-hooks-installed：进仓的 git 钩子装上了（提交、合并、git am 与推送前的大文件判法在 scripts/gate/git-hooks/ 的五份钩子里，经 core.hooksPath 才起作用）
# 判法只在 scripts/gate/git-hooks-install.py 的 check（用这一道所在仓里那一份），被判的根照传：core.hooksPath 是绝对路径、解开指到主工作树里的钩子目录、
# .claude/hooks/lib_git_hooks.py 的 HOOK_NAMES 列的五份钩子文件都在而且有执行位、进仓的模式是 100755；链接工作树（gate.sh --staged 的临时树）读共用的 .git/config，指到主工作树的同一个目录。
# 判的是此刻这个仓的本地配置，不是树里的文本：没装的机器上这一格红，出路是装。
# 样本：setup.sh 在样本根里建小 git 仓与五个桩钩子（有执行位、进仓模式 100755）：
#   git-hooks-installed-green              core.hooksPath 是这个根里钩子目录的绝对路径，整格绿；
#   git-hooks-installed-unset-red          core.hooksPath 没设，红，点名「core.hooksPath 没设」；
#   git-hooks-installed-wrong-path-red     core.hooksPath 指到 .git/hooks，红，点名「不是进仓的钩子目录」；
#   git-hooks-installed-relative-path-red  core.hooksPath 是相对路径 scripts/gate/git-hooks，红，点名「是相对路径」；
#   git-hooks-installed-hook-missing-red   prepare-commit-msg 文件不在，红，点名「prepare-commit-msg 不在」；
#   git-hooks-installed-hook-no-exec-red   pre-applypatch 工作区里没有执行位，红，点名「pre-applypatch 没有执行位」；
#   git-hooks-installed-tracked-mode-red   pre-merge-commit 进仓的模式是 100644（工作区里有执行位），红，点名「进仓的模式是 100644」。
# 弄坏开关（折成 GIT_HOOKS_INSTALL_BREAK）：git-hooks-path-unchecked（check-ignores-path）只看设没设、不看指到哪：git-hooks-installed-wrong-path-red 判绿；
# git-hooks-relative-path-accepted（check-accepts-relative-path）：git-hooks-installed-relative-path-red 判绿；git-hooks-exec-bit-unchecked（check-ignores-exec-bit）：git-hooks-installed-hook-no-exec-red 判绿；
# git-hooks-tracked-mode-unchecked（check-ignores-tracked-mode）：git-hooks-installed-tracked-mode-red 判绿；git-hooks-added-unchecked（check-skips-added-hooks）只核 pre-commit 与 pre-push：
# git-hooks-installed-hook-missing-red、git-hooks-installed-hook-no-exec-red、git-hooks-installed-tracked-mode-red 都判绿。
cell_git_hooks_installed() {
  local installer="$STAGE_REPOSITORY/scripts/gate/git-hooks-install.py" break_item=""
  [[ -f "$installer" ]] || { echo "  ✗ 找不到 $installer"; echo "     → 怎么办：它随仓走（scripts/gate/git-hooks-install.py），被删了就从 git 找回来；--staged 时它要先暂存。"; exit 1; }
  case "${CODE_TOOLING_BREAK:-}" in
    git-hooks-path-unchecked) break_item=check-ignores-path ;;
    git-hooks-relative-path-accepted) break_item=check-accepts-relative-path ;;
    git-hooks-exec-bit-unchecked) break_item=check-ignores-exec-bit ;;
    git-hooks-tracked-mode-unchecked) break_item=check-ignores-tracked-mode ;;
    git-hooks-added-unchecked) break_item=check-skips-added-hooks ;;
  esac
  GIT_HOOKS_INSTALL_BREAK="$break_item" python3 "$installer" check "$ROOT"
}

cell_stage_order() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
# 与共享 gate.sh 起本地阶段同一条取法、同一个排序
stage_order="$(find "$ROOT/.claude/gate.d" -maxdepth 1 -name '*.sh' \( -type f -o -type l \) | sort)"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$ENVIRONMENT_STAGE" "$stage_order" "${CODE_TOOLING_BREAK:-}" <<'PY'
import os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
environment_stage, broken = sys.argv[2], sys.argv[4]
ordered = [os.path.basename(path) for path in sys.argv[3].split("\n") if path.strip()]
TEST_RUNNING = re.compile(r'\bcargo\s+(?:\+\S+\s+)?(?:test|build|run|nextest|bench)\b'
                          r'|\b(?:replay|mutate|mutation-shard-run|multi-host-run|vm-bench|lkmm)\.sh\b'
                          r'|\bqemu-system|\bherd7\b')
if not ordered:
    print("  ! .claude/gate.d/ 顶层一道阶段都没有，这一格无对象可判")
    sys.exit(77)
if environment_stage not in ordered:
    print(f"  ✗ 门禁目录里没有环境检查那一道 {environment_stage}（顶层 {len(ordered)} 道：{'、'.join(ordered)}）")
    print("     → 怎么办：它改了名就同步改这一格的 ENVIRONMENT_STAGE；整道删了就连这一格一起删——留着一个指向空处的名字，次序就没人判了。")
    sys.exit(1)
environment_position = ordered.index(environment_stage)
test_stages = []
for position, name in enumerate(ordered):
    if name == environment_stage:
        continue
    with open(os.path.join(".claude/gate.d", name), encoding="utf-8", errors="replace") as handle:
        found = TEST_RUNNING.search(code_text.code_of(handle.read()))
    if found:
        test_stages.append((position, name, found.group(0)))
if not test_stages:
    print(f"  ! 门禁目录顶层 {len(ordered)} 道里一道跑测试的阶段都没认出来，这一格无对象可判")
    sys.exit(77)
late = [] if broken == "stage-order-ignored" else [entry for entry in test_stages if entry[0] > environment_position]
if late:
    print(f"  ✗ {environment_stage} 排在第 {environment_position + 1}/{len(ordered)} 道，这些跑测试的阶段排在它后面（它们编的、建的、留下的，这一趟环境检查看不见）：")  # gate-lint:summary
    for position, name, evidence in late:
        print(f"     {name}（第 {position + 1} 道，代码里有「{evidence}」）")  # gate-lint:detail
    print(f"     → 怎么办：给这几道改名，让它们按 gate.sh 的 find … | sort 排到 {environment_stage} 前面（名字按 sort 的次序排在它前面）；")
    print("               环境检查那一道要改名的，名字仍要排在全部跑测试的阶段之后，并同步改这一格的 ENVIRONMENT_STAGE。")
    sys.exit(1)
print(f"  ✓ {environment_stage} 排在第 {environment_position + 1}/{len(ordered)} 道，门禁目录里 {len(test_stages)} 道跑测试的阶段都排在它前面"
      f"（{'、'.join(name for _position, name, _evidence in test_stages)}）；没认成跑测试的另外 {len(ordered) - 1 - len(test_stages)} 道")
PY
}

# ── 格 stage-location-before-cd：阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname "$0" 找旁边的文件
#
# 判据：.claude/gate.d/ 顶层每个 *.sh（lib/ 不判）里，找第一行 `cd "$ROOT"` 或 `cd "${1`；它之后的非注释行（出路文字 echo / printf / print( 之类不算）里
#   出现 `dirname "$0"` 或 `dirname "${BASH_SOURCE` 判红，点名文件:行。没有那一行 cd 的阶段不判，成功句报出有几道。
# 为什么：用相对路径调阶段、又另给一个项目根时，cd 之后 $0 的相对目录解析不到原来的位置，阶段找不到自己旁边的共用脚本，退 1 说「找不到」。
#   整轮门禁用绝对路径调阶段，碰不到，只在手工带相对路径起一道时红。
# 判不了：用别的写法取自己的位置（readlink -f "$0"、$(realpath …)）；cd 写成别的变量名（cd "$root"）；heredoc 里嵌的 shell 文本（按普通行判）。
# 弄坏开关 CODE_TOOLING_BREAK=dirname-after-cd-ignored 不判 cd 之后的 dirname：stage-location-before-cd-red 判绿。
# 样本：stage-location-before-cd-red 的 gate.d 里 broken-stage.sh 在 cd 之后用 dirname "$0" 找 stage-run-or-skip.sh，必须点名它的文件:行；
#   同一份里 careful-stage.sh 的 dirname 在 cd 之前、注释里与 echo 出路文字里提到 dirname "$0" 的不许点名，没有 cd 的 plain-stage.sh 不判。
#   stage-location-before-cd-green 把 broken-stage.sh 改成 cd 之前取位置存进变量，判绿并报扫了 2 道、判红 0 处、没有 cd 的 1 道不判。
cell_stage_location_before_cd() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
stage_files="$(find "$ROOT/.claude/gate.d" -maxdepth 1 -name '*.sh' \( -type f -o -type l \) | sort)"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$stage_files" "${CODE_TOOLING_BREAK:-}" <<'PY'
import os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
broken = sys.argv[3]
paths = [path for path in sys.argv[2].split("\n") if path.strip()]
if not paths:
    print("  ! .claude/gate.d/ 顶层一道阶段都没有，这一格无对象可判")
    sys.exit(77)
CD_TO_ROOT = re.compile(r'^\s*cd\s+"(?:\$ROOT|\$\{1)')
FINDS_OWN_LOCATION = re.compile(r'dirname\s+"(?:\$0"|\$\{BASH_SOURCE)')
scanned, without_cd, hits = 0, [], []
for path in paths:
    name = os.path.basename(path)
    with open(path, encoding="utf-8", errors="replace") as handle:
        lines = handle.read().splitlines()
    cd_line = next((number for number, line in enumerate(lines, 1) if CD_TO_ROOT.match(line) and not line.lstrip().startswith("#")), None)
    if cd_line is None:
        without_cd.append(name)
        continue
    scanned += 1
    if broken == "dirname-after-cd-ignored":
        continue
    for number in range(cd_line + 1, len(lines) + 1):
        code = code_text.code_of(lines[number - 1])
        if FINDS_OWN_LOCATION.search(code):
            hits.append((name, number, cd_line, lines[number - 1].strip()))
if hits:
    print(f"  ✗ {len(hits)} 处在 cd 到项目根之后还用 dirname \"$0\" / dirname \"${{BASH_SOURCE\" 找自己的位置（扫了 {scanned} 道有 cd 的阶段，另 {len(without_cd)} 道没有那一行 cd、不判）：")  # gate-lint:summary
    for name, number, cd_line, text in hits:
        print(f"     .claude/gate.d/{name}:{number}（cd 在第 {cd_line} 行）：{text[:140]}")  # gate-lint:detail
    print("     → 怎么办：在 cd 之前取阶段自己的位置存进变量（STAGE_REPOSITORY=\"$(cd \"$(dirname \"$0\")/../..\" && pwd)\" 这一类），cd 之后用那个变量；")
    print("               用相对路径调阶段、又另给一个项目根时，cd 之后 $0 的目录就解析不到了。")
    sys.exit(1)
print(f"  ✓ .claude/gate.d/ 顶层 {len(paths)} 道阶段里扫了 {scanned} 道有 cd 到项目根的，cd 之后都没再用 dirname 找自己的位置（判红 0 处）；没有那一行 cd 的另外 {len(without_cd)} 道不判")
PY
}
