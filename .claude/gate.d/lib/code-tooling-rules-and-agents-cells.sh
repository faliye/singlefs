#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：rules-manifest、rules-named-by-definitions、stage-owners、agent-write-scope、agent-definition-shape、change-range-single-source、research-gate-lint。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# ── 格 rules-manifest：项目规则清单一致
#
# 判据：.claude/rules/ 下的文件集合与 CLAUDE.md 里登记的集合逐项相等；登记有两种：@.claude/rules/<文件名> 引用（常驻），
#   与 <!-- doc-lint:read-once <文件名> … --> 登记（不常驻、开会话读一次；写法与上游 doc-lint「K. 规则清单」那一段同一种：同一个正则认注释、
#   注释里每个 [A-Za-z0-9._-]+.md 是一个文件名）。没被登记的规则读不进上下文。
#   read-once 里的名字：在 .claude/rules/ 下的算项目规则；不在 .claude/rules/ 而在上游副本 .claude/singlefs-ai-sop/rules/ 下的归上游 doc-lint 的规则清单判，
#   这一格不判、成功句报条数；两处都没有的判悬空。
#   代码围栏（``` 或 ~~~ 起止）与行内代码里的 @ 与 read-once 注释都不算登记：Claude Code 不在代码里展开 @ 导入，代码里的注释只是在说这个写法。
# 样本：rules-manifest-red 放一份没被引用的规则、一份只在代码围栏里与一份只在行内代码里被 @ 的规则，必须逐个报出；rules-manifest-green 两侧逐项相等，判绿。
#   rules-manifest-read-once-green：一份规则 @ 引用、一份只在 read-once 里登记，read-once 里另登记一份上游副本里的规则，判绿并报 read-once 1 条、归上游 1 条；
#   rules-manifest-read-once-red：read-once 登记了一份 .claude/rules/ 与上游副本里都没有的规则，判红点名它。
# 弄坏开关 CODE_TOOLING_BREAK=rules-read-once-ignored：不认 read-once 登记，rules-manifest-read-once-green 判红（期望退出 0，实测 1：只在 read-once 里登记的那一份报没被引用），
#   rules-manifest-read-once-red 判绿（期望退出 1，实测 0：悬空的登记看不见）。
cell_rules_manifest() {
python3 - "$MANIFEST_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import fnmatch, os, re, sys, types

manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)

RULES = ".claude/rules"
MD = "CLAUDE.md"
if not os.path.isdir(RULES):
    print(f"  ! 没有 {RULES}，这一格无对象可判")
    sys.exit(77)
if not os.path.isfile(MD):
    print(f"  ! 找不到 {MD}，这一格跳过")
    sys.exit(77)

on_disk = sorted(name for name in os.listdir(RULES) if fnmatch.fnmatchcase(name, "*.md"))
outside_code = manifest.outside_markdown_code

UPSTREAM_RULES = ".claude/singlefs-ai-sop/rules"
# 与上游 doc-lint「K. 规则清单」同一种写法：注释的认法与注释里文件名的认法
READ_ONCE_COMMENT = re.compile(r"<!-- *doc-lint:read-once[^>]*-->")
READ_ONCE_NAME = re.compile(r"[A-Za-z0-9._-]+\.md")
with open(MD, encoding="utf-8", errors="replace") as handle:
    text = outside_code(handle.read())
at_referenced = {os.path.basename(match) for match in re.findall(r"@\.claude/rules/[A-Za-z0-9._-]+\.md", text)}
read_once = set() if sys.argv[2] == "rules-read-once-ignored" else {
    name for comment in READ_ONCE_COMMENT.findall(text) for name in READ_ONCE_NAME.findall(comment)}
upstream_on_disk = set(fnmatch.filter(os.listdir(UPSTREAM_RULES), "*.md")) if os.path.isdir(UPSTREAM_RULES) else set()
read_once_upstream = sorted(name for name in read_once if name not in on_disk and name in upstream_on_disk)
read_once_project = read_once - set(read_once_upstream)
referenced = sorted(at_referenced | read_once_project)
missing, extra = manifest.two_way(on_disk, referenced)

if missing:
    print(f"  ✗ 这些规则文件存在，但 {MD} 既没有 @ 引用、也没在 read-once 里登记——不会被读进上下文，等于没写：")   # gate-lint:detail
    for name in missing:
        print(f"     {RULES}/{name}")   # gate-lint:detail
dangling_at = [name for name in extra if name in at_referenced]
dangling_read_once = [name for name in extra if name not in at_referenced]
if dangling_at:
    print(f"  ✗ {MD} 引用了这些规则，但文件不存在——引用悬空：")   # gate-lint:detail
    for name in dangling_at:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if dangling_read_once:
    print(f"  ✗ {MD} 的 read-once 登记了这些规则，但 {RULES}/ 与上游副本 {UPSTREAM_RULES}/ 里都没有——登记悬空：")
    for name in dangling_read_once:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if missing or extra:
    print("     → 怎么办：补上缺的 @ 引用或 read-once 登记（<!-- doc-lint:read-once <文件名> … -->），或删掉悬空的引用与登记；两侧必须逐项相等。")
    sys.exit(1)
print(f"  ✓ 项目规则清单一致（{len(on_disk)} 条，与 {MD} 的登记逐项相符：@ 引用 {len(at_referenced)} 条、只在 read-once 里登记 {len(read_once_project - at_referenced)} 条；"
      f"read-once 里另有 {len(read_once_upstream)} 条归上游副本 {UPSTREAM_RULES}/，由上游 doc-lint 的规则清单判）")
PY
}

# ── 格 rules-named-by-definitions：CLAUDE.md 的每条 @ 规则与 read-once 规则，至少被一份定义按路径点名
# 为什么：定义都开了 omitClaudeMd，子 agent 只读定义与共用约束点名的规则；一条规则一份都没点名，就没有子 agent 读它（records/2026-10-29-subagent拆分提案.md 第一百六十六节 ②）。
# 判据：CLAUDE.md（代码围栏与行内代码之外，与 rules-manifest 同一份去法 outside_markdown_code）里登记的规则：
#   @.claude/rules/<名>.md 与 @.claude/singlefs-ai-sop/rules/<名>.md 照写的路径；read-once 注释里的每个文件名，.claude/rules/ 下有的取 .claude/rules/<名>，
#   没有而上游副本 .claude/singlefs-ai-sop/rules/ 下有的取那一份，两处都没有的归 rules-manifest 判悬空、这里不判；
#   每一条的路径在 .claude/agents/*.md 或 .claude/agent-common.md 里至少出现一次（按整段路径认，前后不接路径里的字），一份都没有的逐条点名判红。
#   CLAUDE.md 不在、或一条规则都没登记：退 77。
# 管不到的：点名的那一处是不是真让子 agent 读它（「开工先读：」一行与别处顺带提到都算）；一条规则该被哪几份定义点名。
# 样本：rules-named-by-definitions-red（样本仓：CLAUDE.md @ 两条项目规则与一条上游规则，read-once 一条项目规则；共用约束与一份定义只点名其中两条，
#   一条 @ 规则只被定义写成不带目录的文件名、一条 read-once 规则谁都没提，两条逐条点名判红）；
#   rules-named-by-definitions-green（同一副架子，四条都被按路径点名；一条只写在代码围栏里的 @ 不算登记、不判），判绿报「查了 4 条规则、2 份定义」。
#   只管主 agent 的规则登记在 .claude/gate.d/rules-main-agent-only.tsv：一行「规则路径<TAB>理由」，# 起头的行与空行不算；登了的规则不要求定义点名。
#   这张表自己判红三种：登记的路径不在 CLAUDE.md 的登记里；登记的规则其实已有定义按路径点名（多余的行，判红不放过，删掉那一行）；理由不到 8 个字；
#   一行不是两列或同一条规则登了两行也判红。表不在照没有登记办。
#   rules-named-by-definitions-main-only-green（四条规则里一条登了表、没有定义点名，其余三条被点名），判绿报「查了 4 条规则（含 1 条登记为只管主 agent）、2 份定义」；
#   rules-named-by-definitions-main-only-unregistered-red（表里登的路径不在 CLAUDE.md 里）、-main-only-named-red（表里登的规则已被共用约束点名）、
#   -main-only-reason-red（理由只有 3 个字），各判红在表的那一行。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-read-once-skipped：只判 @ 规则，read-once 登记的不判；
#   rules-named-by-definitions-red 输出里找不到 read-once 那一条（「.claude/rules/once-only.md」），rules-named-by-definitions-green 输出里找不到「查了 4 条规则」（只剩 3 条）。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-by-basename：只写文件名、不带目录的也算点名；rules-named-by-definitions-red 输出里找不到「.claude/rules/basename-only.md」那一条。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-main-only-ignored：登记表不认，登记了的规则照样要求点名；rules-named-by-definitions-main-only-green 判红在「.claude/rules/main-only.md」那一条。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-main-only-unregistered-unchecked：不核表里的路径在不在 CLAUDE.md 的登记里；-main-only-unregistered-red 输出里找不到「不在 CLAUDE.md 的 @ 引用与 read-once 登记里」。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-main-only-named-unchecked：不核表里的规则是不是已有定义点名；-main-only-named-red 输出里找不到「这一行多余」。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-main-only-reason-unchecked：不核理由够不够 8 个字；-main-only-reason-red 输出里找不到「理由不到 8 个字」。
# 退役判据：定义不再开 omitClaudeMd（子 agent 照读 CLAUDE.md 与它 @ 的规则）时删掉这一格。
cell_rules_named_by_definitions() {
python3 - "$MANIFEST_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, sys, types

manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
broken = sys.argv[2]
MD = "CLAUDE.md"
RULES = ".claude/rules"
UPSTREAM_RULES = ".claude/singlefs-ai-sop/rules"
if not os.path.isfile(MD):
    print(f"  ! 找不到 {MD}，这一格无对象可判")
    sys.exit(77)
with open(MD, encoding="utf-8", errors="replace") as handle:
    text = manifest.outside_markdown_code(handle.read())
registered = list(dict.fromkeys(re.findall(r"@(\.claude/(?:singlefs-ai-sop/)?rules/[A-Za-z0-9._-]+\.md)", text)))
if broken != "rules-named-read-once-skipped":
    for comment in re.findall(r"<!-- *doc-lint:read-once[^>]*-->", text):
        for name in re.findall(r"[A-Za-z0-9._-]+\.md", comment):
            for directory in (RULES, UPSTREAM_RULES):
                if os.path.isfile(os.path.join(directory, name)):
                    if f"{directory}/{name}" not in registered:
                        registered.append(f"{directory}/{name}")
                    break
if not registered:
    print(f"  ! {MD} 里一条规则都没登记（@ 引用与 read-once 都没有），这一格无对象可判")
    sys.exit(77)
definitions = sorted(glob.glob(".claude/agents/*.md")) + ([".claude/agent-common.md"] if os.path.isfile(".claude/agent-common.md") else [])
definition_texts = {}
for path in definitions:
    with open(path, encoding="utf-8", errors="replace") as handle:
        definition_texts[path] = handle.read()
MAIN_ONLY = ".claude/gate.d/rules-main-agent-only.tsv"
main_only, table_problems = {}, []
if os.path.isfile(MAIN_ONLY):
    with open(MAIN_ONLY, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 2:
                table_problems.append(f"{MAIN_ONLY} 第 {number} 行不是「规则路径<TAB>理由」两列：{line.rstrip()}")
                continue
            rule, reason = fields[0].strip(), fields[1].strip()
            if rule in main_only:
                table_problems.append(f"{MAIN_ONLY} 第 {number} 行：{rule} 登记了不止一行")
                continue
            main_only[rule] = number
            if len(reason) < 8 and broken != "rules-named-main-only-reason-unchecked":
                table_problems.append(f"{MAIN_ONLY} 第 {number} 行：{rule} 的理由不到 8 个字（写它为什么只管主 agent）")
unnamed, named_counts, exempt_count = [], [], 0
for rule in registered:
    if broken == "rules-named-by-basename":
        pattern = re.compile(r"(?<![\w.-])" + re.escape(os.path.basename(rule)) + r"(?![\w.-])")
    else:
        pattern = re.compile(r"(?<![\w./-])" + re.escape(rule) + r"(?![\w.-])")
    naming = [path for path, body in definition_texts.items() if pattern.search(body)]
    if rule in main_only and broken != "rules-named-main-only-ignored":
        exempt_count += 1
        if naming and broken != "rules-named-main-only-named-unchecked":
            table_problems.append(f"{MAIN_ONLY} 第 {main_only[rule]} 行：{rule} 登记成只管主 agent，可 {'、'.join(naming)} 已按路径点名它，这一行多余")
        continue
    if naming:
        named_counts.append(len(naming))
    else:
        unnamed.append(rule)
if broken != "rules-named-main-only-unregistered-unchecked":
    for rule, number in main_only.items():
        if rule not in registered:
            table_problems.append(f"{MAIN_ONLY} 第 {number} 行：{rule} 不在 {MD} 的 @ 引用与 read-once 登记里")
if unnamed:
    for rule in unnamed:
        print(f"  ✗ {rule}：{MD} 登记了它，.claude/agents/*.md 与 .claude/agent-common.md 里一份都没按路径点名")   # gate-lint:detail
    print(f"  ✗ {len(unnamed)} 条规则没有一份定义点名（查了 {len(registered)} 条规则、{len(definitions)} 份定义），定义开了 omitClaudeMd，这几条没有子 agent 会读")
    print("     → 怎么办：在用得到它的定义「开工先读：」一行或共用约束里按路径点名它（改定义照 .claude/rules/implementation-workflow.md「改 agent 定义与共用约束，走同一条三步」，归工具实现员照判决改）；"
          f"这条规则只有主 agent 用、没有子 agent 要读的，在 {MAIN_ONLY} 加一行「规则路径<TAB>为什么只管主 agent」；"
          "确实没有谁要读的，从 CLAUDE.md 的 @ 与 read-once 里去掉之前先问用户")
if table_problems:
    for problem in table_problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 只管主 agent 的登记表 {MAIN_ONLY} 有 {len(table_problems)} 处不对")
    print(f"     → 怎么办：改表。一行一条规则「规则路径<TAB>理由」，路径要在 {MD} 的 @ 引用或 read-once 登记里、理由至少 8 个字写它为什么只管主 agent；"
          "已经有定义点名的规则删掉它那一行")
if unnamed or table_problems:
    sys.exit(1)
scope = f"查了 {len(registered)} 条规则" + (f"（含 {exempt_count} 条登记为只管主 agent）" if exempt_count else "")
print(f"  ✓ {MD} 登记的规则都至少被一份定义点名：{scope}、{len(definitions)} 份定义，被点名最少的一条 {min(named_counts, default=0)} 份")
PY
}

# ── 格 stage-owners：阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根
#
# 判据（.claude/rules/verification-reference.md「门禁的结构」）：.claude/gate.d/stage-owners.tsv 一行一格，四列：门禁文件名、格名、判红时派给谁修、为什么。任一条不成立判红：
#   ① 门禁目录顶层每道 *.sh 的 --list 出的每一格在表里恰好一行。--list 经 gate-structure-check.py 的 header_cells、make_shims、start_stage 起
#      （替身拦重活、限时）；没有格名表的不起它（起它就是全跑），与 --list 没退 0、起了重活、一格都没打的一起报「列不出格」；
#   ② 表里每一行的门禁文件在、那一格在它的 --list 里；③ 第三列每个 agent 名都有 .claude/agents/<名字>.md；④ 每行四列、没有空列；
#   ⑤ 每道阶段认第一个参数当项目根（ROOT_ARGUMENT_PATTERN）：只看当前目录的阶段从别的目录带着根参数调用时安静地判错地方，
#      stage-selftest 既 cd 又传参，看不出来。
# 按格登记、不按道：按道时一格红会把同道别的格的派修一起带走。
# 样本：stage-owners-red 的表里有三列的行、同一格登记两行、没有定义的 agent、指向门禁里没有的格与指向不在的门禁的行，
#   目录里有一格没登记、一道没有格名表、一道 --list 退 1、一道不认第一个参数，必须逐个报出；
#   stage-owners-green 两道门禁三格逐格登记，判绿并报门禁数、格数与 agent 数。
cell_stage_owners() {
local jobs
if ! jobs="$(heavy_thread_default)"; then
  echo "  ✗ 取不了每台默认重活线程数（原因在上一行），各道阶段 --list 的并行度定不了，一道都没起"
  echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
  exit 1
fi
python3 - "$MANIFEST_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/gate-structure-check.py" "${CODE_TOOLING_BREAK:-}" "$jobs" <<'PY'
import concurrent.futures, glob, importlib.util, os, re, shutil, sys, tempfile, types
sys.dont_write_bytecode = True
manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
broken = sys.argv[3]
table_path = ".claude/gate.d/stage-owners.tsv"
if not os.path.isfile(table_path):
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表，四列用制表符分隔：门禁文件名、格名、判红时派给谁修（agent 名，逗号分隔）、为什么；# 开头的行是注释。格名用 bash .claude/gate.d/<门禁> --list 看。")
    sys.exit(1)
try:
    specification = importlib.util.spec_from_file_location("gate_structure_check", sys.argv[2])
    structure = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(structure)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 导入不了 {sys.argv[2]}：{error}")
    print("     → 怎么办：各道门禁的 --list 经它起（替身拦重活、限时）；它随仓走，被删了或改坏了就从 git 找回来，别在这一格另写一份起法。")
    sys.exit(1)
ROOT_ARGUMENT_PATTERN = re.compile(r'\b(?:ROOT|root|REPOSITORY_ROOT|repository_root|PROJECT_ROOT)="?\$\{(?:1|root_argument)(?::-|\})|\bcd +"?\$\{?1\b')
# 与共享 gate.sh 认的本地阶段同一批：.claude/gate.d/ 顶层每个 *.sh 普通文件（它用 find -maxdepth 1 -name '*.sh'）
stage_files = sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh") if os.path.isfile(path))
root = os.getcwd()
cells_by_stage, unlisted = {}, []
scratch = tempfile.mkdtemp(prefix="stage-owners-list-")
try:
    shim_directory = structure.make_shims(scratch)
    # gate-structure 格这一趟起过的 --list --force 结果就在共用的临时目录里（structure.LIST_CACHE_DIRECTORY）：同一棵树、同一条命令，读那一份，不再起第二遍
    if os.environ.get("CODE_TOOLING_RUN_SCRATCH"):
        structure.LIST_CACHE_DIRECTORY = os.path.join(os.environ["CODE_TOOLING_RUN_SCRATCH"], "stage-list")
    listed_stages = [stage for stage in stage_files if structure.header_cells(os.path.join(root, ".claude", "gate.d", stage))]
    structure.process_group.install_stop_on_signals(only_defaults=True)   # 起阶段在池里的线程上，停止信号的处理要先在主线程装好
    # 各道阶段的 --list 彼此独立（各自一个会话、各自的 trace 文件）：按每台默认重活线程数同时起，结果按阶段次序取回
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(int(sys.argv[4]), len(listed_stages) or 1))) as pool:
        listings = dict(zip(listed_stages, pool.map(lambda stage: structure.start_stage(root, stage, ["--list", "--force"], shim_directory,
                                                                                     os.path.join(scratch, f"shim-trace-{stage}")), listed_stages)))
    for stage in stage_files:
        if stage not in listings:
            unlisted.append(f"{stage}：文件头没有格名表（# gate-cell: 行），不起它的 --list（起它就是全跑）")
            continue
        exit_code, stdout, stderr, called = listings[stage]
        if called:
            unlisted.append(f"{stage}：--list 起了重活（{'、'.join(called)}）")
        elif exit_code is None:
            unlisted.append(f"{stage}：--list 跑了 {structure.LIST_TIMEOUT_SECONDS} 秒还没退出")
        elif exit_code != 0:
            unlisted.append(f"{stage}：--list 退 {exit_code}；输出末几行：{structure.tail(stdout + stderr)}")
        else:
            cells = [line.split("\t", 1)[0].strip() for line in stdout.splitlines() if line.strip()]
            if cells:
                cells_by_stage[stage] = cells
            else:
                unlisted.append(f"{stage}：--list 一格都没打")
finally:
    shutil.rmtree(scratch, ignore_errors=True)

rows_by_cell = {}
malformed_rows, unknown_owner_rows = [], []
table = manifest.table_rows(table_path)
for line_number, line, fields in table:
    if len(fields) != 4 or not all(field.strip() for field in fields):
        malformed_rows.append(f"第 {line_number} 行：{line}")
        continue
    stage, cell, owners, _reason = (field.strip() for field in fields)
    rows_by_cell.setdefault((stage, cell), []).append(line_number)
    for owner in owners.split(","):
        if not os.path.isfile(f".claude/agents/{owner.strip()}.md"):
            unknown_owner_rows.append(f"第 {line_number} 行 {stage} {cell}：{owner.strip()}")
listed_cells = [(stage, cell) for stage in stage_files for cell in cells_by_stage.get(stage, [])]
def cell_exists(key):
    stage, cell = key
    if stage not in stage_files:
        return False
    if stage not in cells_by_stage:
        return True   # 列不出格的那几道另报，它们的行在不在判不了，不在这里重复报
    return cell in cells_by_stage[stage]
unregistered, dangling = manifest.two_way(listed_cells, list(rows_by_cell), exists=cell_exists)
if broken == "owner-rows-ignored":
    unregistered = []
duplicated = [f"{stage} {cell}（第 {', '.join(map(str, numbers))} 行）" for (stage, cell), numbers in rows_by_cell.items() if len(numbers) > 1]
stages_ignoring_the_root_argument = [stage for stage in stage_files if not ROOT_ARGUMENT_PATTERN.search(open(os.path.join('.claude/gate.d', stage), encoding='utf-8', errors='replace').read())]
failed = False
if malformed_rows:
    failed = True
    print("  ✗ 这些行不是四列、或有一列是空的：")  # gate-lint:summary
    for entry in malformed_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行四列用制表符分隔：门禁文件名、格名、判红时派给谁修（agent 名，逗号分隔）、为什么派给它；四列都要有内容。")
if unlisted:
    failed = True
    print("  ✗ 这些门禁列不出自己的格，它们的格登没登记判不了：")  # gate-lint:summary
    for entry in unlisted:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：先照 gate-structure 格的出路修结构（bash .claude/gate.d/code-tooling.sh --check gate-structure），--list 列得出格了再跑这一格。")
if unregistered:
    failed = True
    print("  ✗ 这些格在表里没有登记——判红时不知道派给谁修：")  # gate-lint:summary
    for stage, cell in unregistered:
        print(f"     {stage} 的 {cell}")  # gate-lint:detail
    print(f"     → 怎么办：在 {table_path} 给每一格加一行：门禁文件名、格名、判红时派给谁修、为什么；没有合适的 agent 就写 investigator。格名照 bash .claude/gate.d/<门禁> --list 抄。")
if duplicated:
    failed = True
    print("  ✗ 这些格在表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，几个 agent 名写在第三列、用逗号分隔。")
if dangling:
    failed = True
    print("  ✗ 表里登记了这些格，门禁目录里没有那道门禁、或那道门禁的 --list 里没有这一格——多半改了名或删了：")  # gate-lint:summary
    for stage, cell in dangling:
        where = "门禁目录里没有这道门禁" if stage not in stage_files else "那道门禁没有这一格"
        print(f"     {stage} {cell}（第 {', '.join(map(str, rows_by_cell[(stage, cell)]))} 行，{where}）")  # gate-lint:detail
    print("     → 怎么办：改成现在的门禁文件名与格名（bash .claude/gate.d/<门禁> --list 看），或删掉这一行。")
if unknown_owner_rows:
    failed = True
    print("  ✗ 这些 agent 名在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for entry in unknown_owner_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：改成已有定义的名字（文件名去掉 .md），或先建那个定义。")
if stages_ignoring_the_root_argument:
    failed = True
    print("  ✗ 这些阶段不认第一个参数当项目根（只看当前目录，从别的目录带着根目录参数调用时判的是错的地方）：")  # gate-lint:summary
    for stage in stages_ignoring_the_root_argument:
        print(f"     {stage}")  # gate-lint:detail
    print('     → 怎么办：在 preflight 那一行之后加 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}" 与 cd "$ROOT"（或者给每个路径加 "$ROOT/" 前缀），写法照 code-tooling.sh 自己。')
if failed:
    sys.exit(1)
owners = sorted({owner.strip() for _line_number, _line, fields in table for owner in fields[2].split(",")})
print(f"  ✓ 阶段归属表与各道门禁的格逐格一致（{len(stage_files)} 道门禁、{len(listed_cells)} 格，派给 {len(owners)} 个 agent；{len(stage_files)} 道都认第一个参数当项目根）")
PY
}

# ── 格 agent-write-scope：钩子注册在要的 matcher 上、写范围表与定义双向一致、每个定义带 omitClaudeMd 与「开工先读：」、共用切词与项目代码判定模块不另写一份
#
# 判据，任一条不成立判红（「注册」指 settings.json 里有一条 matcher 覆盖那个工具或事件、命令指向那个钩子）：
#   ① PreToolUse 里命令指向 write-guard.sh 的注册，matcher 合起来覆盖 Write、Edit 与 MultiEdit（WRITE_GUARD_TOOLS）；
#   ③ 每个定义（tools 含 Write 或 Edit 的，与只带 Read、Bash 这类的）在 .claude/hooks/agent-write-scope.tsv 里至少一条路径模式（后者没有行，Bash 检出钩子 ⑫ 对它们在 Bash 里往仓里写一概不判）；
#      ④ 表里每个 agent 名都有定义，每行至少两列；
#   ⑤ Bash 注册 bash-command-detector.sh；⑥ Agent 注册 runner-dispatch-guard.sh；⑦ SendMessage 注册 continuation-guard.sh；
#   ⑧ 每个定义的 frontmatter 有 omitClaudeMd: true，正文有一行以「开工先读：」开头；
#   ⑨ Bash 注册上游 claude-hooks/pattern-process-guard.sh，且那个文件在（它的判别力归上游 selftest）；
#   ⑩ SessionStart 里命令指向上游 claude-hooks/session-checks.sh 的注册，matcher 合起来覆盖 startup、resume、clear、compact 四种 source（检查项的文件头、执行位与自证归上游 hooks-registered.sh）；
#   ⑫ Bash 注册 project-code-guard.sh；⑭ AskUserQuestion 注册 ask-user-claim-guard.sh 与 ask-user-placement-guard.sh；⑯ SubagentHandback 注册 handback-guard.sh；
#   ⑲ Bash 注册 run-scope-guard.sh（范围声明闸；注在不覆盖 Bash 的 matcher 上的不算）；
#   ⑳ Bash 注册 find-timestamp-guard.sh（find 时刻写法闸；注在不覆盖 Bash 的 matcher 上的不算，逐条点名）；
#   ⑰ PreToolUse 里命令指向 timer-only-remind-guard.sh 的注册，matcher 合起来覆盖 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor 与 Bash（TIMER_GUARD_TOOLS）；
#   ⑱ Stop 里有命令指向 remind-rearm-stop-guard.sh 的注册（循环提醒到点之后自启动靠它）；
#   ㉑ Stop 里有命令指向 session-number-stop-check.sh 的注册（主 agent 收工时拦下给用户的回复里写 mainagent(A) 这类字母）；
#   ⑬ 共用模块 lib_shell_words.py、lib_project_code_runs.py 与它们同层的部分文件（lib_shell_words_core.py、lib_project_code_runs_launchers.py 这一类）合起来叫一族：
#      族只认一种写法，在 scripts/gate/lib_split_family_loaded.py（这一道脚本旁边那一份）：名字落在命名族里（<模块名>_<部分>.py），而且入口（或族里别的文件）真载入它
#      （import、from … import、同目录的相对 import、字面量参数的 importlib.import_module、赋给 *PART_NAMES 的写死文件名单 exec；按语法认，注释、字符串、文档串里写的不算）；
#      名字像部分文件而入口没载入的不算族，被入口 import 的共用库也不算族，两样的手抄都照报；调用方只从入口导入。
#      一族里每个文件顶层定义的每个函数名，.claude/hooks/ 下一族之外的文件里不许再 def 一份；名字从这一族现读（这一道脚本旁边的 ../hooks/），
#      跳过的是被判仓自己 .claude/hooks/ 里入口连同它载入的部分文件（被判仓里的入口认族）；读不到、一族里一个函数都没有也红；不算的名字登记在 NOT_SHARED_JUDGMENT，登记了而这一族里没有的也红；
#   ⑮ frontmatter 的 model 只许 ALLOWED_MODELS 里的取值。
# 钩子的自证不在这里跑，归上游 hooks-registered.sh；它只核有一条注册指向钩子、不核挂在要的 matcher 上，所以 matcher 留在这一格判。
#
# 样本：agent-write-scope-red 的 .claude/hooks/ 里放一份手抄 shell_tokens 的 hook 与一份手抄 judged_by_name 的 hook（两个函数都住在部分文件里），必须各报出文件与行号，settings.json 没有 SessionStart，必须报没注册会话检测钩子；
#   agent-write-scope-parts-red 是 green 那一份多一个 hook，里面手抄 brace_expand（住在 lib_shell_words_core.py）与 classify（住在 lib_project_code_runs_launchers.py），必须各报出文件、行号与函数住在哪个部分文件；
#   agent-write-scope-name-like-red 是 green 那一份多一个 .claude/hooks/lib_shell_words_shadow.py（名字像共用切词模块的部分文件，被判仓里的入口 lib_shell_words.py 没载入它），里面手抄入口里的 commands_at_command_position，
#     必须报出该文件、行号（名字像而没载入的不算族）；
#   agent-write-scope-loaded-shared-red 是 green 那一份，被判仓里的入口 lib_shell_words.py import 了 .claude/hooks/shared_text_tools.py（几份脚本共用的库，名字不像部分文件），那一份里手抄 commands_at_command_position，
#     必须报出该文件、行号（被入口 import 的共用库不算族）；
#   agent-write-scope-comment-named-red 是 name-like-red 那一份，被判仓里的入口 lib_shell_words.py 只在注释与文档串里点名 lib_shell_words_shadow.py、写 import lib_shell_words_shadow（没有载入它），
#     必须同样报出该文件、行号（注释与文档串里写的不算载入）；
#   agent-write-scope-import-module-green 是 green 那一份，被判仓里的入口 lib_shell_words.py 用字面量参数的 importlib.import_module 载入同目录的 lib_shell_words_core.py（定义 brace_expand，
#     共用切词模块里也有这个名字），必须判绿（按 import_module 载入的部分文件是族里的、不算手抄），成功行里项目代码判定模块那一族多查一个文件（lib_shell_words_core.py 不在它的族里）；
#   agent-write-scope-multiedit-red 的 settings.json 只把 write-guard.sh 注册在 "Write|Edit" 上，必须报没覆盖 MultiEdit；
#   agent-write-scope-timer-red 的 settings.json 把 timer-only-remind-guard.sh 注册在不含 Monitor 的 matcher 上、Stop 里没有 remind-rearm-stop-guard.sh，必须报没挂全工具（没覆盖 Monitor）与没注册停止钩子，SessionStart 的 matcher 只写 startup|resume|compact，必须报会话检测钩子没挂全 source（没覆盖 clear）；
#   agent-write-scope-run-scope-red 是 green 那一份，只把 run-scope-guard.sh 注册在 "Write|Edit|MultiEdit" 上（不覆盖 Bash），必须报没注册范围声明闸；
#   agent-write-scope-find-timestamp-red 是 green 那一份，find-timestamp-guard.sh 只注册在 "Write|Edit|MultiEdit" 上（不覆盖 Bash），必须报没注册 find 时刻写法闸、点名注错的那一条；
#   agent-write-scope-readers-red 是 green 那一份去掉 beta（只带 Read、Bash 的定义）在表里的行，必须报出 beta 没登记、点名表里要给它加路径模式；
#   agent-write-scope-placement-red 是 green 那一份，settings.json 里 AskUserQuestion 那一条只有 ask-user-claim-guard.sh，必须报没注册放置类弹窗闸；
#   agent-write-scope-session-number-red 是 green 那一份，settings.json 的 Stop 里只有 remind-rearm-stop-guard.sh，必须报没注册会话编号收工检查；
#   agent-write-scope-green 的 .claude/hooks/ 里放两份从共用模块导入、只写自己判定的 hook（连同被判仓自己那两份模块），表里 alpha（有 Edit）与 beta（只带 Read、Bash）都有行，必须判绿，
#   成功行数进去的有 Write 或 Edit 的定义数、只带 Read、Bash 这类的定义数与别的文件数要对。
# 共用模块的函数名取自这一道脚本旁边 ../hooks/ 下那两族（取自 $0 的目录，不取项目根）：样本仓里放坏的模块碰不到它。
# 弄坏开关 AGENT_WRITE_SCOPE_BREAK=family-parts-unread：查的函数名只取入口文件里的顶层函数、不取部分文件里的（NOT_SHARED_JUDGMENT 的登记照一族判），
#   agent-write-scope-red 与 agent-write-scope-parts-red 手抄的函数都住在部分文件里，判绿。
# 弄坏开关 AGENT_WRITE_SCOPE_BREAK=family-by-name：只按名字认族（命名族里的文件都算，不看入口载没载入），agent-write-scope-name-like-red 手抄的文件被当成族里的、跳过，判绿；
#   AGENT_WRITE_SCOPE_BREAK=family-by-loading：只按载入认族（入口 import 的文件都算，不看名字像不像部分文件），agent-write-scope-loaded-shared-red 手抄的文件被当成族里的、跳过，判绿；
#   AGENT_WRITE_SCOPE_BREAK=family-by-text：载入按文本行认（注释、文档串里写的文件名与 import 行也算载入，import_module 认不出），agent-write-scope-comment-named-red 手抄的文件被当成族里的、跳过，判绿，
#     agent-write-scope-import-module-green 里按 import_module 载入的部分文件不在族里、里面的 brace_expand 被报成手抄，判红。
cell_agent_write_scope() {
HOOKS_DIRECTORY="$(cd "$STAGE_DIRECTORY/../hooks" && pwd)"
table_output="$(python3 - "$MANIFEST_LIBRARY" "$HOOKS_DIRECTORY/lib_shell_words.py" "$HOOKS_DIRECTORY/lib_project_code_runs.py" "$STAGE_REPOSITORY/scripts/gate/lib_split_family_loaded.py" <<'PY'
import ast, glob, importlib.util, json, os, re, shlex, sys, types
manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
failed = False
settings_path = ".claude/settings.json"
def command_invokes(command, script):
    """这条 hook 命令真的去跑 script：按 shell 切词（# 起的注释去掉），第一个词是它，或第一个词是 bash / sh / python3 / env、
    第二个词是它；路径按结尾认（…/.claude/hooks/write-guard.sh）。只在注释、echo 的参数、别的词里提到它的不算。"""
    try:
        words = shlex.split(command or "", comments=True)
    except ValueError:
        return False
    if len(words) > 1 and os.path.basename(words[0]) in ("bash", "sh", "python3", "env"):
        words = words[1:]
    return bool(words) and (words[0] == script or words[0].endswith("/" + script))
def mention_only_commands(entries, script):
    """提到了 script、却没有跑它的命令：出路里点名，免得读的人以为那一条已经注册着。"""
    return [hook.get("command") or "" for entry in entries for hook in entry.get("hooks") or []
            if script in (hook.get("command") or "") and not command_invokes(hook.get("command"), script)]
def matcher_covers(matcher, tool):
    if matcher in ("*", ""):
        return True
    if re.fullmatch(r"[A-Za-z0-9_|]+", matcher):
        return tool in matcher.split("|")
    try:
        return re.search(matcher, tool) is not None
    except re.error:
        return False
try:
    entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("PreToolUse") or []
except Exception as error:
    entries = None
    failed = True
    print(f"  ✗ 读不了 {settings_path}：{error}")
    print("     → 怎么办：修好这份 JSON，再在 hooks.PreToolUse 里注册写范围闸。")
WRITE_GUARD_TOOLS = ("Write", "Edit") if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "multiedit-ignored" else ("Write", "Edit", "MultiEdit")
if entries is not None:
    registered = [entry for entry in entries
                  if any(command_invokes(hook.get("command"), "write-guard.sh") for hook in entry.get("hooks") or [])]
    uncovered_tools = [tool for tool in WRITE_GUARD_TOOLS
                       if not any(matcher_covers(entry.get("matcher", ""), tool) for entry in registered)]
    if not registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册写范围闸：PreToolUse 里没有命令指向 write-guard.sh 的一条")
        for command in mention_only_commands(entries, "write-guard.sh"):
            print(f"     这一条只是提到 write-guard.sh、没有跑它，不算注册：{command}")  # gate-lint:detail
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Write|Edit|MultiEdit"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/write-guard.sh"。')
    elif uncovered_tools:
        failed = True
        print(f"  ✗ {settings_path} 写范围闸没挂全工具：命令指向 write-guard.sh 的注册，matcher 合起来没覆盖：{'、'.join(uncovered_tools)}"
              "——这几个工具写文件时写范围、角标、钟点、行尾空白那几道判定一道都不跑")
        print('     → 怎么办：把 write-guard.sh 那一条的 matcher 写成 "Write|Edit|MultiEdit"。')
    guard_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any(command_invokes(hook.get("command"), "bash-command-detector.sh") for hook in entry.get("hooks") or [])]
    if not guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册 Bash 检出 hook：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 bash-command-detector.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Bash"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/bash-command-detector.sh"。')
    code_guard_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any(command_invokes(hook.get("command"), "project-code-guard.sh") for hook in entry.get("hooks") or [])]
    if not code_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册项目代码闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 project-code-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/project-code-guard.sh"。')
    run_scope_registered = [entry for entry in entries
                            if matcher_covers(entry.get("matcher", ""), "Bash")
                            and any(command_invokes(hook.get("command"), "run-scope-guard.sh") for hook in entry.get("hooks") or [])]
    if not run_scope_registered and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "run-scope-unchecked":
        failed = True
        print(f"  ✗ {settings_path} 没注册范围声明闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 run-scope-guard.sh 的一条——起验证脚本的命令不带范围声明也没人拒")
        for command in mention_only_commands([entry for entry in entries if matcher_covers(entry.get("matcher", ""), "Bash")], "run-scope-guard.sh"):
            print(f"     这一条只是提到 run-scope-guard.sh、没有跑它，不算注册：{command}")  # gate-lint:detail
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/run-scope-guard.sh"（注在别的 matcher 上的挪过来）。')
    find_timestamp_registered = [entry for entry in entries
                                 if matcher_covers(entry.get("matcher", ""), "Bash")
                                 and any(command_invokes(hook.get("command"), "find-timestamp-guard.sh") for hook in entry.get("hooks") or [])]
    if not find_timestamp_registered and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "find-timestamp-unchecked":
        failed = True
        print(f"  ✗ {settings_path} 没注册 find 时刻写法闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 find-timestamp-guard.sh 的一条——find 的时刻写成本机 bfs 不认的样子也没人拒")
        for entry in entries:
            for hook in entry.get("hooks") or []:
                if command_invokes(hook.get("command"), "find-timestamp-guard.sh") and not matcher_covers(entry.get("matcher", ""), "Bash"):
                    print(f"     这一条注在不覆盖 Bash 的 matcher「{entry.get('matcher', '')}」上，不算：{hook.get('command')}")  # gate-lint:detail
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/find-timestamp-guard.sh"（注在别的 matcher 上的挪过来）。')
    dispatch_registered = [entry for entry in entries
                           if matcher_covers(entry.get("matcher", ""), "Agent")
                           and any(command_invokes(hook.get("command"), "runner-dispatch-guard.sh") for hook in entry.get("hooks") or [])]
    if not dispatch_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册续派闸：PreToolUse 里没有 matcher 覆盖 Agent、命令指向 runner-dispatch-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Agent|Task"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/runner-dispatch-guard.sh"。')
    continuation_registered = [entry for entry in entries
                               if matcher_covers(entry.get("matcher", ""), "SendMessage")
                               and any(command_invokes(hook.get("command"), "continuation-guard.sh") for hook in entry.get("hooks") or [])]
    if not continuation_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册续做闸：PreToolUse 里没有 matcher 覆盖 SendMessage、命令指向 continuation-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "SendMessage"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/continuation-guard.sh"。')
    handback_guard_registered = [entry for entry in entries
                                 if matcher_covers(entry.get("matcher", ""), "SubagentHandback")
                                 and any(command_invokes(hook.get("command"), "handback-guard.sh") for hook in entry.get("hooks") or [])]
    if not handback_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册交回闸：PreToolUse 里没有 matcher 覆盖 SubagentHandback、命令指向 handback-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 SubagentHandback 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/handback-guard.sh"。')
    claim_guard_registered = [entry for entry in entries
                              if matcher_covers(entry.get("matcher", ""), "AskUserQuestion")
                              and any(command_invokes(hook.get("command"), "ask-user-claim-guard.sh") for hook in entry.get("hooks") or [])]
    if not claim_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册弹窗断言闸：PreToolUse 里没有 matcher 覆盖 AskUserQuestion、命令指向 ask-user-claim-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "AskUserQuestion"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/ask-user-claim-guard.sh"。')
    placement_guard_registered = [entry for entry in entries
                                  if matcher_covers(entry.get("matcher", ""), "AskUserQuestion")
                                  and any(command_invokes(hook.get("command"), "ask-user-placement-guard.sh") for hook in entry.get("hooks") or [])]
    if not placement_guard_registered and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "placement-guard-unchecked":
        failed = True
        print(f"  ✗ {settings_path} 没注册放置类弹窗闸：PreToolUse 里没有 matcher 覆盖 AskUserQuestion、命令指向 ask-user-placement-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 AskUserQuestion 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/ask-user-placement-guard.sh"。')
    TIMER_GUARD_TOOLS = ("CronCreate", "ScheduleWakeup", "RemoteTrigger", "Skill", "Bash") if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "timer-monitor-ignored" else ("CronCreate", "ScheduleWakeup", "RemoteTrigger", "Skill", "Monitor", "Bash")
    timer_registered = [entry for entry in entries
                        if any(command_invokes(hook.get("command"), "timer-only-remind-guard.sh") for hook in entry.get("hooks") or [])]
    timer_uncovered = [tool for tool in TIMER_GUARD_TOOLS
                       if not any(matcher_covers(entry.get("matcher", ""), tool) for entry in timer_registered)]
    if not timer_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册定时唯一办法闸：PreToolUse 里没有命令指向 timer-only-remind-guard.sh 的一条——CronCreate、ScheduleWakeup 这些别的定时办法没人拒")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/timer-only-remind-guard.sh"。')
    elif timer_uncovered:
        failed = True
        print(f"  ✗ {settings_path} 定时唯一办法闸没挂全工具：命令指向 timer-only-remind-guard.sh 的注册，matcher 合起来没覆盖：{'、'.join(timer_uncovered)}——这几个工具里的别的定时办法没人拒")
        print('     → 怎么办：把 timer-only-remind-guard.sh 那一条的 matcher 写成 "CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash"。')
    pattern_guard = "singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh"
    pattern_guard_registered = [entry for entry in entries
                                if matcher_covers(entry.get("matcher", ""), "Bash")
                                and any(command_invokes(hook.get("command"), pattern_guard) for hook in entry.get("hooks") or [])]
    if not pattern_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册按模式找进程的钩子：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 {pattern_guard} 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/' + pattern_guard + '"（写法见 .claude/singlefs-ai-sop/rules/command-safety.md「pkill -f / killall 一律禁用」）。')
    elif not os.path.isfile(os.path.join(".claude", pattern_guard)):
        failed = True
        print(f"  ✗ 注册了按模式找进程的钩子，文件 .claude/{pattern_guard} 却不在：会话里每条 Bash 命令都会报钩子错")
        print("     → 怎么办：规范副本旧于 0.0.52 或没装全，按 CLAUDE.md「规范从哪来」那一行重新同步副本、跑 install.sh。")
try:
    session_entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("SessionStart") or []
except Exception:
    session_entries = None  # 读不了这份 JSON 的那一条上面已经报过、已判红
if session_entries is not None:
    SESSION_SOURCES = ("startup", "resume", "clear", "compact")
    session_checks_hook = "singlefs-ai-sop/scripts/claude-hooks/session-checks.sh"
    session_checks_registered = [entry for entry in session_entries
                                 if any(command_invokes(hook.get("command"), session_checks_hook) for hook in entry.get("hooks") or [])]
    session_uncovered = [source for source in SESSION_SOURCES
                         if not any(matcher_covers(entry.get("matcher", ""), source) for entry in session_checks_registered)]
    session_checks_judged = os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "session-checks-unchecked"
    if session_checks_judged and not session_checks_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册会话检测钩子：SessionStart 里没有命令指向 {session_checks_hook} 的一条——启动、恢复、清空与压缩之后项目登记的检查项没人跑")
        print('     → 怎么办：在 hooks.SessionStart 里加一条 matcher "startup|resume|clear|compact"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/' + session_checks_hook + '"。')
    elif session_checks_judged and session_uncovered:
        failed = True
        print(f"  ✗ {settings_path} 会话检测钩子没挂全 source：命令指向 {session_checks_hook} 的注册，matcher 合起来没覆盖：{'、'.join(session_uncovered)}——那几种时机检查项不跑")
        print('     → 怎么办：把那一条的 matcher 写成 "startup|resume|clear|compact"。')
try:
    stop_entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("Stop") or []
except Exception:
    stop_entries = None  # 读不了这份 JSON 的那一条上面已经报过、已判红
if stop_entries is not None and not any(command_invokes(hook.get("command"), "remind-rearm-stop-guard.sh") for entry in stop_entries for hook in entry.get("hooks") or []):
    failed = True
    print(f"  ✗ {settings_path} 没注册提醒自启动的停止钩子：Stop 里没有命令指向 remind-rearm-stop-guard.sh 的一条——循环提醒到点退出之后没人让主 agent 重起")
    print('     → 怎么办：在 hooks.Stop 里加一条 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/remind-rearm-stop-guard.sh"。')
if (stop_entries is not None and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "session-number-stop-unchecked"
        and not any(command_invokes(hook.get("command"), "session-number-stop-check.sh") for entry in stop_entries for hook in entry.get("hooks") or [])):
    failed = True
    print(f"  ✗ {settings_path} 没注册会话编号收工检查：Stop 里没有命令指向 session-number-stop-check.sh 的一条——主 agent 给用户的回复里写 mainagent 加字母的叫法没人拦")
    print('     → 怎么办：在 hooks.Stop 里加一条 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/session-number-stop-check.sh"。')
table_path = ".claude/hooks/agent-write-scope.tsv"
patterns_by_agent, malformed = {}, []
if os.path.isfile(table_path):
    for line_number, line, fields in manifest.table_rows(table_path):
        if len(fields) < 2 or not fields[0].strip() or not fields[1].strip():
            malformed.append(f"第 {line_number} 行：{line}")
            continue
        patterns_by_agent.setdefault(fields[0].strip(), []).append(fields[1].strip())
else:
    failed = True
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表：agent 名、路径模式、出处三列，用制表符分隔，按每个能写文件的定义的「写范围」一节转写。")
writers = []
definitions = []   # 全部定义（有 Write / Edit 的与只带 Read、Bash 这类的）
missing_omit, missing_basis, bad_models = [], [], []
inherits_all_tools = set()   # 没写 tools 行、继承全部工具的定义
# ⑮ frontmatter 的 model 只许这几个取值（行尾可带 YAML 注释）：写成别的，换环境之后派发报 model_not_found、那一类 agent 全部起不来
ALLOWED_MODELS = {"opus", "sonnet", "haiku", "inherit"}
for path in sorted(glob.glob(".claude/agents/*.md")):
    whole = open(path, encoding="utf-8").read()
    head = whole.split("\n---", 1)[0]
    definitions.append(os.path.basename(path)[:-3])
    model_line = next((line for line in head.split("\n") if line.startswith("model:")), None)
    if model_line is not None:
        model_value = model_line[len("model:"):].split("#", 1)[0].strip().strip("'\"")
        if model_value not in ALLOWED_MODELS and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "any-model":
            bad_models.append(f"{os.path.basename(path)[:-3]}：{model_line.strip()}")
    if not any(line.strip() == "omitClaudeMd: true" for line in head.split("\n")):
        missing_omit.append(os.path.basename(path)[:-3])
    if not any(line.startswith("开工先读：") for line in whole.split("\n")):
        missing_basis.append(os.path.basename(path)[:-3])
    # tools：没写这一行就继承全部工具（含 Write、Edit）；`tools: A, B`、`tools: [A, B]` 与 YAML 列表（下面几行 `- A`）都认
    head_lines = head.split("\n")
    tools_index = next((index for index, line in enumerate(head_lines) if line.startswith("tools:")), None)
    if tools_index is None:
        writers.append(os.path.basename(path)[:-3])
        inherits_all_tools.add(os.path.basename(path)[:-3])
        continue
    inline_value = head_lines[tools_index][len("tools:"):].strip().strip("[]")
    tools = {tool.strip().strip("'\"") for tool in inline_value.split(",") if tool.strip()}
    for line in head_lines[tools_index + 1:]:
        list_item = re.match(r"^\s*-\s*(\S+)", line)
        if not list_item:
            break
        tools.add(list_item.group(1).strip("'\""))
    if tools & {"Write", "Edit", "*"}:
        writers.append(os.path.basename(path)[:-3])
if malformed:
    failed = True
    print("  ✗ 写范围表里这些行不到两列、或 agent 名与路径模式有空的：")  # gate-lint:summary
    for entry in malformed:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行至少写 agent 名与路径模式两列，用制表符分隔。")
# ③ 每个定义都要在表里登记：有 Write 或 Edit 的（写闸与 Bash 检出钩子 ⑫ 都照它那几行判），与只带 Read、Bash 这类的（只有 ⑫ 判它在 Bash 里的写，表里没有行就一概不判）；
#   ④ 表里的名字要有定义。两个方向的盘上集合不同，各给各的判定。AGENT_WRITE_SCOPE_BREAK=readers-unchecked 下只要求有 Write 或 Edit 的登记。
unscoped, ghosts = manifest.two_way(writers if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "readers-unchecked" else definitions, list(patterns_by_agent),
                                    exists=lambda name: os.path.isfile(f".claude/agents/{name}.md"))
unscoped_writers = [name for name in unscoped if name in writers]
unscoped_readers = [name for name in unscoped if name not in writers]
if unscoped_writers:
    failed = True
    print("  ✗ 这些定义的 tools 里有 Write 或 Edit，写范围表里却没有登记——hook 会把它们的每次写都拒掉：")  # gate-lint:summary
    for name in unscoped_writers:
        print(f"     {name}（没写 tools，继承全部工具）" if name in inherits_all_tools else f"     {name}")  # gate-lint:detail
    print(f"     → 怎么办：照它定义里「写范围」一节，在 {table_path} 给它加路径模式。")
if unscoped_readers:
    failed = True
    print("  ✗ 这些定义的 tools 里没有 Write 与 Edit（只带 Read、Bash 这类），写范围表里却没有登记——Bash 检出钩子 ⑫ 只判表里有行的定义，它们在 Bash 里往仓里写一概不判：")  # gate-lint:summary
    for name in unscoped_readers:
        print(f"     {name}")  # gate-lint:detail
    print(f"     → 怎么办：照它定义里「写范围」一节，在 {table_path} 给它加路径模式（只许写报告文件与草稿目录的写 $SINGLEFS_WORK_ROOT/**）。")
if ghosts:
    failed = True
    print("  ✗ 写范围表里登记的这些 agent 在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for name in ghosts:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：改成现在的定义名，或删掉这几行。")
if bad_models:
    failed = True
    print("  ✗ 这些定义的 frontmatter 里 model 不是 opus、sonnet、haiku、inherit 之一——派发时报 model_not_found，这一类 agent 起不来：")  # gate-lint:summary
    for line in bad_models:
        print(f"     {line}")  # gate-lint:detail
    print("     → 怎么办：改成 opus、sonnet、haiku 或 inherit（行尾可以带 # 注释）；本地模型不当腿派（.claude/rules/three-way-inference.md「各条腿必须互不重复」），不写进 model。")
if missing_omit:
    failed = True
    print("  ✗ 这些定义的 frontmatter 没有 omitClaudeMd: true——每次派发都白带约 10 万 token 的 CLAUDE.md 与规则：")  # gate-lint:summary
    for name in missing_omit:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：在它的 frontmatter 里加一行 omitClaudeMd: true，开工先读那一句指到 .claude/agent-common.md「规则怎么读」。")
if missing_basis:
    failed = True
    print("  ✗ 这些定义没有「开工先读：」一行——不继承 CLAUDE.md 之后，它要读的规则没有地方点名：")  # gate-lint:summary
    for name in missing_basis:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：在正文里加一行「开工先读：」，点名它干活要读的规则文件与小节。")
# ⑬ 两族共用模块（入口文件加同层部分文件）顶层定义的函数名现读（这一道脚本旁边 ../hooks/ 下那一族），再扫被判仓 .claude/hooks/ 下一族之外的每个文件
# 不算的名字：不是共用的判定，别的 hook 里有同名的一份不算手抄；登记了而模块里没有的判红（不起作用的排除项）
NOT_SHARED_JUDGMENT = {
    "lib_project_code_runs.py": {
        "selftest": "每个 hook 都有自己的 --selftest 入口，同名不是抄的判定",
        "load_sibling_module": "按文件路径导入同目录模块的样板，不是判定",
    },
}
# (模块文件名, 叫法, 这一道脚本旁边那一份入口的路径（部分文件在它同一个目录里）, 手抄一份会怎样, 该怎么办, 模块里该有什么)
SHARED_MODULES = [
    ("lib_shell_words.py", "共用切词模块", sys.argv[2],
     "手抄的切词会分叉：同一条命令一个 hook 认得出、另一个认不出",
     "从 `.claude/hooks/lib_shell_words.py` 导入，不另写一份（照 bash-command-detector.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑两个 hook 的 --selftest。",
     "切词、切简单命令、认命令位置、剥前缀、跟 cd 这几个函数要定义在 lib_shell_words.py 或它的部分文件（lib_shell_words_*.py）里，两个 hook 从 lib_shell_words.py 导入。"),
    ("lib_project_code_runs.py", "共用项目代码判定模块", sys.argv[3],
     "手抄的判定会分叉：执行前的闸 project-code-guard.sh 与看门狗 agent-watch.py 对同一条命令判得不一样",
     "从 `.claude/hooks/lib_project_code_runs.py` 导入，不另写一份（照 project-code-guard.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑它与 project-code-guard.sh 的 --selftest。",
     "classify、judged_by_name 这些判定要定义在 lib_project_code_runs.py 或它的部分文件（lib_project_code_runs_*.py）里，project-code-guard.sh 与看门狗从 lib_project_code_runs.py 导入。"),
]
hooks_directory = ".claude/hooks"
shared_counts = []   # [(模块文件名, 查的函数个数, 一族里的部分文件个数, 查了几个别的文件)]
family_parts_unread = os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "family-parts-unread"
family_specification = importlib.util.spec_from_file_location("lib_split_family_loaded", sys.argv[4])
try:
    family_library = importlib.util.module_from_spec(family_specification)
    family_specification.loader.exec_module(family_library)
except (OSError, SyntaxError, ImportError, ValueError) as error:
    print(f"  ✗ 载不进 {sys.argv[4]}：{error}")
    print("     → 怎么办：它随仓走（scripts/gate/lib_split_family_loaded.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。")
    sys.exit(1)
family_defects = set()
if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "family-by-name":
    family_defects.add(family_library.NAME_ONLY)
if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "family-by-loading":
    family_defects.add(family_library.LOADED_ONLY)
if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "family-by-text":
    family_defects.add(family_library.TEXT_SCAN)
for module_file, module_title, module_path, divergence, fix, required in SHARED_MODULES:
    # 一族（函数名从这一道脚本旁边的那一族取）：入口加同层、名字落在命名族里又被入口载入的部分文件（lib_split_family_loaded.py）
    part_paths = sorted(family_library.loaded_family(module_path, family_library.sibling_files(module_path), family_defects)[1:])
    # 跳过的文件名：被判仓自己 .claude/hooks/ 里的入口连同它载入的部分文件（真跑时被判仓就是这一道所在的仓，两族相同；样本仓里是样本自己的那一份入口）
    judged_entry = os.path.join(hooks_directory, module_file)
    family_names = {os.path.basename(path) for path in family_library.loaded_family(judged_entry, family_library.sibling_files(judged_entry), family_defects)}
    function_home, entry_function_names = {}, set()   # 函数名 → 定义它的文件名；入口自己顶层定义的函数名
    unreadable = False
    for family_path in [module_path, *part_paths]:
        try:
            family_tree = ast.parse(open(family_path, encoding="utf-8").read(), family_path)
        except (OSError, SyntaxError, ValueError) as error:
            failed = unreadable = True
            print(f"  ✗ 读不了{module_title}{'' if family_path == module_path else '的部分文件'} {family_path}：{error}")
            print(f"     → 怎么办：恢复 .claude/hooks/{os.path.basename(family_path)}；{fix}")
            break
        for node in family_tree.body:
            if isinstance(node, ast.FunctionDef):
                function_home.setdefault(node.name, os.path.basename(family_path))
                if family_path == module_path:
                    entry_function_names.add(node.name)
    if unreadable:
        continue
    function_names = list(function_home)
    if not function_names:
        failed = True
        print(f"  ✗ {module_title} {module_path} 与它的部分文件里一个顶层函数都没有：这一条没有名字可查")
        print(f"     → 怎么办：{required}")
        continue
    not_judgment = NOT_SHARED_JUDGMENT.get(module_file, {})
    stale_exclusions = sorted(set(not_judgment) - set(function_names))
    if stale_exclusions:
        failed = True
        print(f"  ✗ NOT_SHARED_JUDGMENT 给 {module_file} 登记了它与部分文件里都没有的函数：{'、'.join(stale_exclusions)}（不起作用的排除项会让人以为那几个名字被绕开了）")
        print("     → 怎么办：从 code-tooling.sh 的 agent-write-scope 格的 NOT_SHARED_JUDGMENT 里删掉这几项，或者核一下是不是模块里的函数改了名。")
    checked_names = [name for name in function_names if name not in not_judgment and (not family_parts_unread or name in entry_function_names)]
    redefinition = re.compile(r"^[ \t]*def[ \t]+(" + "|".join(map(re.escape, checked_names)) + r")[ \t]*\(", re.M) if checked_names else None
    redefinitions, scanned_hook_files = [], 0
    for directory, subdirectories, file_names in os.walk(hooks_directory):
        subdirectories[:] = sorted(name for name in subdirectories if name != "__pycache__")
        for file_name in sorted(file_names):
            path = os.path.normpath(os.path.join(directory, file_name))
            if directory == hooks_directory and file_name in family_names:
                continue
            try:
                content = open(path, encoding="utf-8").read()
            except (OSError, UnicodeDecodeError):
                continue
            scanned_hook_files += 1
            for match in (redefinition.finditer(content) if redefinition else ()):
                redefinitions.append(f"{path}:{content.count(chr(10), 0, match.start()) + 1}  def {match.group(1)}（共用模块里定义在 {function_home[match.group(1)]}）")
    shared_counts.append((module_file, len(checked_names), len(part_paths), scanned_hook_files))
    if redefinitions:
        failed = True
        print(f"  ✗ {module_title} .claude/hooks/{module_file} 里的函数，在 .claude/hooks/ 下别的文件里又定义了一份（{divergence}）：")  # gate-lint:summary
        for entry in redefinitions:
            print(f"     {entry}")  # gate-lint:detail
        print(f"     → 怎么办：{fix}（函数住在部分文件里的，入口文件已把名字带出来，一样从入口导入。）")
if failed:
    sys.exit(1)
pattern_count = sum(len(patterns) for patterns in patterns_by_agent.values())
print("TABLE_OK", len(writers), len(definitions) - len(writers), pattern_count, *(count for _, function_count, part_count, file_count in shared_counts for count in (function_count, part_count, file_count)),
      "、".join(name for names in NOT_SHARED_JUDGMENT.values() for name in names))
PY
)"; table_rc=$?
printf '%s\n' "$table_output" | grep -v '^TABLE_OK '
if [[ $table_rc -ne 0 ]]; then exit 1; fi
read -r _ writer_count reader_count pattern_count shared_function_count shared_part_count scanned_hook_file_count project_code_function_count project_code_part_count project_code_scanned_file_count not_shared_judgment_names <<<"$(printf '%s\n' "$table_output" | grep '^TABLE_OK ')"
echo "  ✓ 写范围闸、Bash 检出 hook、项目代码闸、续派闸、续做闸、交回闸、弹窗断言闸与定时唯一办法闸注册在要的 matcher 上，提醒自启动的停止钩子注册着，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着，表与定义一致（${writer_count} 个有 Write 或 Edit 的定义与 ${reader_count} 个只带 Read、Bash 这类的定义都登记了、${pattern_count} 条路径模式），共用切词模块的 ${shared_function_count} 个函数（lib_shell_words.py 加 ${shared_part_count} 个部分文件）只在这几份里定义（查了 .claude/hooks/ 下另外 ${scanned_hook_file_count} 个文件），共用项目代码判定模块的 ${project_code_function_count} 个函数（lib_project_code_runs.py 加 ${project_code_part_count} 个部分文件）只在这几份里定义（查了 .claude/hooks/ 下另外 ${project_code_scanned_file_count} 个文件；${not_shared_judgment_names} 不算，见 NOT_SHARED_JUDGMENT）；钩子的自证归上游 hooks-registered.sh 跑"
}

# ── 格 agent-definition-shape：定义的形状。还拆分提案 records/2026-10-23-subagent拆分提案.md 第五节「新门禁阶段」那张表里没人判的四条
#   （权限写明、四样齐、不抄规则、登记）与第七节 ①；那张表的 frontmatter 一条归上游 doc-lint 的 B2，写范围一致、model 取值、omitClaudeMd 与「开工先读：」归 agent-write-scope 格。
#
# 判据，任一条不成立判红（定义是 .claude/agents/*.md，frontmatter 是第 1 行的 --- 到下一行 --- 之间）：
#   ① frontmatter 有一行 model:（取值认不认得归 agent-write-scope ⑮）；
#   ② frontmatter 有一行 tools:（没写就继承全部工具，Agent、Write、Edit 都在里面）；
#   ③ tools 里没有 Agent、Task（`tools: A, B`、`tools: [A, B]` 与下面几行 `- A` 的 YAML 列表都认；`Agent(…)` 这种带括号的按括号前那一段认）；
#   ④ 正文（frontmatter 之后、代码围栏之外）有以「输入」「写范围」「产出」「没做什么」起头的二级标题，四样各至少一个（`## 输入（主 agent 必须给）` 这种带尾巴的算）；
#   ⑤ 定义、.claude/agent-common.md（在就判）与 .claude/main-agent.md 里的每一行，去掉首尾空白与行首的列表记号（- * + > 与 1. 1)）之后，
#      与 .claude/rules/*.md、.claude/singlefs-ai-sop/rules/*.md 里某一行同样处理之后逐字相同、又不短于 RULE_COPY_MINIMUM 个字符的，判红：规则只指路径，不抄进定义；
#      只由表格分隔与围栏字符（| - : ` ~ = 与空白）组成的行不算。
#      阈值的量法：把这三处的每一行照上面去掉记号，找规则文件 28 份里逐字相同的行，只有一行：.claude/main-agent.md 第 93 行
#      「|---|---|」（9 个字符，与 .claude/rules/fs-design.md 的表格分隔同形）；不算表格分隔之后一行都没有。所以零误报的阈值是不小于 10 的任何数；
#      取 20：标题、表头、一两个词的短行（「## 推论」这类）同形不算抄，20 个字符在中文里是一整句。
#   ⑥ .claude/main-agent.md「## 什么时候派哪个 agent」一节里表格行（以 | 起头）反引号点名的、形似定义名的词（小写字母与数字，连字符隔开），与 .claude/agents/*.md 两向相等
#      （manifest.two_way）：每个定义都被点名；点名的词都有定义。点名了却不是定义名的词，指名门禁阶段（.claude/gate.d/<词>.sh 在）或 crate（crates/<词>/ 在）的自动不算，
#      其余登记在 NOT_DEFINITION_NAMES（写为什么）；登记了而又是定义名的红；被判的根不是样本时，登记了而那一节没点名的也红（不起作用的排除项）。这份文件或这一节没有，红。
# 管不到的：四节底下写没写对东西、tools 给得对不对、规则抄了半行或换了说法、调度表那一行写得对不对；表外别处点名的定义名不算登记。
#
# 样本（每份样本根放 .gate-cells 只点名这一格，定义、主 agent 与规则文件都是样本自己的小份）：
#   agent-definition-shape-model-red：一份定义 frontmatter 没写 model，必须点名它；agent-definition-shape-tools-red：一份没写 tools，必须点名它；
#   agent-definition-shape-agent-tool-red：一份 tools 写 Agent、一份写成 YAML 列表、列表里有 Task(…)，必须各点名一次；
#   agent-definition-shape-sections-red：一份缺「产出」一节、「没做什么」写成三级标题，另一份的「写范围」只在代码围栏里，必须各点名缺的那几样；
#   agent-definition-shape-rule-copy-red：一份定义与样本的 .claude/agent-common.md 各抄了样本规则文件里的一整行（一份带了不同的列表记号），必须各报文件:行与规则文件:行；
#   agent-definition-shape-table-red：调度表没点名一份定义、点名了一个没有定义的词，必须两样都点名；
#   agent-definition-shape-green：tools 写成 YAML 列表、四节带尾巴、正文与 description 里提到 Agent、定义里有一行与规则文件同形而短于阈值、表格分隔同形，
#   调度表里另点名了 NOT_DEFINITION_NAMES 里的 cfg 与样本里一道门禁阶段的名字，必须判绿并报定义份数与点名的词数。
cell_agent_definition_shape() {
python3 - "$MANIFEST_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$(judged_root_is_sample && echo sample || echo repository)" <<'PY'
import glob, os, re, sys, types
manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
broken, root_kind = sys.argv[2], sys.argv[3]
RULE_COPY_MINIMUM = 20          # ⑤ 的阈值，量法见这一格函数上方的注释
REQUIRED_SECTIONS = ("输入", "写范围", "产出", "没做什么")
FORBIDDEN_TOOLS = ("Agent", "Task")
TABLE_SECTION = "## 什么时候派哪个 agent"
# ⑥ 调度表里点名、不是定义名的词（门禁阶段名与 crate 名自动不算，不用登记）
NOT_DEFINITION_NAMES = {
    "cfg": "Rust 的条件编译属性（验证包那一行说 tests/ 里 cfg 点到的特性）",
    "gpu": "cargo 特性名（验证包那一行说带 gpu 特性的那一趟）",
}
DEFINITION_NAME = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
LIST_MARK = re.compile(r"^(?:(?:[-*+>]|\d+[.)])\s+)+")
STRUCTURE_ONLY = re.compile(r"[\s|\-:`~=]*")
failed = False


def report(title, rows, howto):
    global failed
    failed = True
    print(f"  ✗ {title}")  # gate-lint:summary
    for row in rows:
        print(f"     {row}")  # gate-lint:detail
    print(f"     → 怎么办：{howto}")


def split_definition(text):
    """→ (frontmatter 的行, 正文的行, 正文第一行的行号)；第 1 行不是 --- 的没有 frontmatter。"""
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return [], lines, 1
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            return lines[1:index], lines[index + 1:], index + 2
    return lines[1:], [], len(lines) + 1


def tools_of(head):
    """→ tools 一行列出的工具名集合；没有这一行交 None。"""
    index = next((number for number, line in enumerate(head) if line.startswith("tools:")), None)
    if index is None:
        return None
    inline = head[index][len("tools:"):].strip().strip("[]")
    names = [piece for piece in inline.split(",") if piece.strip()]
    for line in head[index + 1:]:
        item = re.match(r"^\s*-\s*(.+?)\s*$", line)
        if not item:
            break
        names.append(item.group(1))
    return {name.strip().strip("'\"").split("(", 1)[0].strip() for name in names}


def headings_outside_fences(body):
    fence, found = None, []
    for line in body:
        stripped = line.lstrip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            if fence is None:
                fence = marker.group(1)[0]
            elif stripped.startswith(fence * 3):
                fence = None
            continue
        if fence is None and line.startswith("## "):
            found.append(line[3:].strip())
    return found


def comparable(line):
    return LIST_MARK.sub("", line.strip()).strip()


def section_lines(body, prefix):
    """→ 以 prefix 起头的二级标题那一节（到下一个二级标题为止）里代码围栏之外的行；没有这一节交 None。"""
    fence, inside, found = None, False, None
    for line in body:
        stripped = line.lstrip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            if fence is None:
                fence = marker.group(1)[0]
            elif stripped.startswith(fence * 3):
                fence = None
            continue
        if fence is not None:
            continue
        if line.startswith("## "):
            inside = line[3:].strip().startswith(prefix)
            if inside and found is None:
                found = []
            continue
        if inside:
            found.append(line)
    return found


definitions = sorted(glob.glob(".claude/agents/*.md"))
names = [os.path.basename(path)[:-3] for path in definitions]
missing_model, missing_tools, forbidden, missing_sections = [], [], [], []
for path, name in zip(definitions, names):
    head, body, _ = split_definition(open(path, encoding="utf-8").read())
    if not any(line.startswith("model:") for line in head) and broken != "definition-model-unchecked":
        missing_model.append(name)
    tools = tools_of(head)
    if tools is None:
        if broken != "definition-tools-unchecked":
            missing_tools.append(name)
    elif broken != "definition-agent-tool-allowed":
        given = sorted(tools & set(FORBIDDEN_TOOLS))
        if given:
            forbidden.append(f"{name}：tools 里有 {'、'.join(given)}")
    headings = headings_outside_fences(body)
    lacking = [section for section in REQUIRED_SECTIONS if not any(heading.startswith(section) for heading in headings)]
    if lacking and broken != "definition-sections-unchecked":
        missing_sections.append(f"{name}：缺 {'、'.join(lacking)}")
if not definitions:
    report("没有 .claude/agents/*.md：这一格一份定义都没有可判", [], "定义放在 .claude/agents/<名字>.md；样本里也要放至少一份。")
if missing_model:
    report("这些定义的 frontmatter 没写 model——派发时取默认模型，定义里说的模型与真跑的对不上：", missing_model,
           "在 frontmatter 里加一行 model: opus、sonnet、haiku 或 inherit（取值认不认得归 agent-write-scope 格）。")
if missing_tools:
    report("这些定义的 frontmatter 没写 tools——没写就继承全部工具，Agent、Write、Edit 都在里面：", missing_tools,
           "在 frontmatter 里加一行 tools:，只列它干活要的工具（观测类与推论类不给 Write、Edit）。")
if forbidden:
    report("这些定义的 tools 里有 Agent 或 Task——子 agent 只由主 agent 派，不再往下派：", forbidden,
           "从 tools 里删掉 Agent、Task；要分活的写进报告交主 agent 派（.claude/agent-common.md「派发」）。")
if missing_sections:
    report("这些定义缺以「输入」「写范围」「产出」「没做什么」起头的二级标题（代码围栏里的不算）：", missing_sections,
           "补上缺的那几节：输入是派发时必须给的、写范围是它能写的路径、产出是交什么、没做什么是固定不做的；写成 ## 起头的二级标题。")

# ⑤ 不抄规则
rule_files = sorted(glob.glob(".claude/rules/*.md") + glob.glob(".claude/singlefs-ai-sop/rules/*.md"))
rule_lines = {}
for path in rule_files:
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        text = comparable(line)
        if len(text) >= RULE_COPY_MINIMUM and not STRUCTURE_ONLY.fullmatch(text):
            rule_lines.setdefault(text, f"{path}:{number}")
judged_files = definitions + [path for path in (".claude/agent-common.md", ".claude/main-agent.md") if os.path.isfile(path)]
copies, judged_lines = [], 0
for path in judged_files:
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        judged_lines += 1
        text = comparable(line)
        if text in rule_lines and broken != "definition-rule-copies-unchecked":
            copies.append(f"{path}:{number} 与 {rule_lines[text]} 逐字相同（{len(text)} 个字符）：{text[:60]}")
if copies:
    report(f"定义与共用约束里这些行与规则文件逐字相同、不短于 {RULE_COPY_MINIMUM} 个字符——规则抄进定义，改规则时这里不跟着改就分叉：", copies,
           "删掉抄来的那一行，改成一条读取指令：「开工先读：`<规则文件>`「<小节>」」（.claude/agent-common.md「规则怎么读」）。")

# ⑥ 调度表与定义两向相等
main_path = ".claude/main-agent.md"
table_names, not_definition_seen, auto_skipped = [], set(), set()
if not os.path.isfile(main_path):
    report(f"没有 {main_path}：调度表「{TABLE_SECTION[3:]}」无从比对", [], f"主 agent 的调度表写在 {main_path} 的「{TABLE_SECTION}」一节，一行一种情形、反引号点名派谁。")
else:
    in_section, seen_section = False, False
    for line in open(main_path, encoding="utf-8").read().split("\n"):
        if line.startswith("## "):
            in_section = line.strip() == TABLE_SECTION
            seen_section = seen_section or in_section
            continue
        if not in_section or not line.lstrip().startswith("|"):
            continue
        for token in re.findall(r"`([^`\n]+)`", line):
            if not DEFINITION_NAME.fullmatch(token):
                continue
            if token in NOT_DEFINITION_NAMES:
                not_definition_seen.add(token)
            elif token not in names and (os.path.isfile(f".claude/gate.d/{token}.sh") or os.path.isdir(f"crates/{token}")):
                auto_skipped.add(token)
            elif token not in table_names:
                table_names.append(token)
    if not seen_section:
        report(f"{main_path} 里没有「{TABLE_SECTION}」一节：调度表与定义目录无从比对", [], f"把调度表写在 {main_path} 的「{TABLE_SECTION}」一节（标题逐字），一行一种情形、反引号点名派谁。")
    elif broken != "definition-table-unchecked":
        unnamed, dangling = manifest.two_way(names, table_names)
        if unnamed:
            report(f"这些定义在 {main_path}「{TABLE_SECTION[3:]}」表里没被反引号点名——主 agent 照表派活，表里没有的类型没人派：", unnamed,
                   f"在那张表里给它加一行（什么时候派它），或删掉不再用的定义；只在表外别处提到不算。")
        if dangling:
            report(f"{main_path}「{TABLE_SECTION[3:]}」表里反引号点名的这些词在 .claude/agents/ 里没有定义：", dangling,
                   "改成现在的定义名；不是定义名的词（命令、特性名这类）去掉反引号，或登记进 code-tooling.sh 这一格的 NOT_DEFINITION_NAMES 并写为什么。")
        contradictory = sorted(set(NOT_DEFINITION_NAMES) & set(names))
        stale = sorted(set(NOT_DEFINITION_NAMES) - not_definition_seen) if root_kind == "repository" else []
        if contradictory or stale:
            report("code-tooling.sh 这一格的 NOT_DEFINITION_NAMES 有不起作用或自相矛盾的登记：",
                   [f"{token}：同名的定义在 .claude/agents/ 里" for token in contradictory] + [f"{token}：那一节没点名它" for token in stale],
                   "删掉这几项；同名的定义在的，那个词就是定义名，不该登记成不是。")

if failed:
    sys.exit(1)
skipped = sorted(not_definition_seen | auto_skipped)
print(f"  ✓ 定义的形状：{len(definitions)} 份定义都写明 model 与 tools、tools 里没有 Agent 与 Task、输入写范围产出没做什么四节齐；"
      f"定义与共用约束 {len(judged_files)} 份文件 {judged_lines} 行里没有与规则文件（{len(rule_files)} 份）逐字相同、不短于 {RULE_COPY_MINIMUM} 个字符的行；"
      f"{main_path} 的调度表点名 {len(table_names)} 个定义名，与 .claude/agents/ 两向相等（表里另有不是定义名的 {len(skipped)} 个词：{'、'.join(skipped) or '没有'}）")
PY
}

# ── 格 change-range-single-source：改动范围只许一份取法：阶段不自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false
#
# 判据（射程 .claude/gate.d/ 顶层的 *.sh 与 *.py，加上每道门禁拆出的库：.claude/gate.d/lib/ 下按拆分族（scripts/gate/lib_lag_split_family.py 的 family_members）归到某道门禁的 *.sh 与 *.py，不按库名抄；
#   这一道的入口与它拆出的库里只除掉这一格自己：从这一格的标题行到下一格的标题行按空行判；一道门禁的族里，某份文件带 quotepath=false 的变量与包装函数，族里别的文件里的调用同样认
#   （入口里定义、库里调用）；lib/ 下不在任何一道门禁的族里的文件不扫，成功句列出名字），任一条不成立判红：
#   ① 算 diff 基准只许在 scripts/gate/changed-paths.sh 里：代码行出现 merge-base（--is-ancestor 核祖先除外）、@{upstream} 或 @{u}、gate-ok、
#      读 GATE_BASE 或 GATE_DIFF_BASE，或者跑 git 的那一行写死 HEAD~N、HEAD^（HEAD^{tree} 不算）、用三点号（A...B 就是取 merge-base）的，红。
#   ② 列路径的 git 调用（--name-only、--name-status、ls-files）要带 -c core.quotepath=false：同一行带，或经同一个文件里带着它的变量 / 列表、
#      包装函数调，或 git 带 -z；参数列表换了行、这一行以引号开头的，往上看三行找调用前缀。不带时中文路径被转成带引号的八进制串，git 不报错。
#   ③ 调共用取法（gate_changed_paths、gate_added_lines）要判退出码：同一行带 ||，或放在 if / while 里。
#   「代码行」的认法在 is_text：出路文字与注释里提到这些词不算。
#   .py 文件读成语法树，文档串（模块、类、函数开头那一句字符串）与自证函数体（名字带 selftest 或 self_test 的函数）换成空白再判：
#     文档串是说明；自证函数体里设 GATE_BASE、取 merge-base 是造给共用取法的输入，不是阶段取改动范围。读不成语法树的整份照原样判；嵌在 .sh 里的 python 不读语法树。
#   行内豁免：命中的那一行不是调用（报错文字里点名失败的是哪条子命令这类），行尾写「# change-range:exempt <为什么不是调用>」，理由至少 8 个字，
#     这一行三条都不判；理由不够 8 个字的照判。豁免了的逐行打在「行内豁免 N 行」那一句里。
#   ④ 搬迁登记表 .claude/gate.d/path-moves.tsv（被判的根里那一份；只缩不涨的表与基准那一版比时照它换算，读法在 lib-lag-baseline.py 文件头「搬迁登记」）：
#      每一行「<旧前缀><TAB><新前缀><TAB><YYYY-MM-DD><TAB>records/<文件>.md」写对、出处文件在；此刻生效的行（这一行不在 gate 基准那一版的登记表里，gate 基准那一版里旧前缀下有文件、新前缀下没有文件；
#      提交进基准之后不再生效）之间，任一行的旧前缀或新前缀不许等于、包含或落在另一行的旧前缀或新前缀下面；生效的每一行有搬迁实据：基准里旧前缀下的每份文件，
#      此刻在新前缀下照这一行换前缀的同一相对路径上都在、被 git 跟踪（在索引里，或是没被忽略的工作区文件），新前缀在基准里没有文件，旧前缀此刻不在磁盘上
#      （换算之后相同的份数只报不判）；不在基准那一版登记表里的新行不生效的判红「这一行不是一次搬迁」，已在基准那一版登记表里的行不生效不核；
#      被判的根不在 git 仓里、或在仓里但不是仓根时判不了哪几行生效，不核，成功句写明。
#      判法是 lib-lag-baseline.py 的 path_move_report（用这一道所在仓里那一份；gate 基准由它在被判的根上经 scripts/gate/changed-paths.sh 的 gate_diff_base gate 取）；表不在算 0 行。
# 为什么：取法收成共用脚本之前有六份以上拷贝，已经分叉过（一半漏了 quotepath），收完之后要有东西拦下一份拷贝。
#
# 样本：change-range-single-source-red 把三条判据该抓的写法各放几种（自己取 merge-base、@{u}...、HEAD~1、HEAD^、换了行的参数列表续行里的 "merge-base"、
#   不带或带 =true 的 quotepath、包装函数名叫 git 而这一行没调它、test 的 -z、printf "$(git …)"、换行的参数列表、不判退出码的共用取法），逐条点名判红；
#   change-range-single-source-red 的 51-样本列路径不带选项.sh 第 4 行是一条行尾带「# change-range:exempt 太短」的列路径调用：理由不够 8 个字，照判、点名。
#   change-range-single-source-green 放五种合法写法与 unset GATE_BASE、merge-base --is-ancestor、HEAD^{tree}、判了退出码的调用，判绿。
#   change-range-single-source-prose-green（不在 git 仓里）：模块与函数的文档串里写着 merge-base、HEAD~1、GATE_BASE、gate_changed_paths、ls-files、--name-only（50），
#   自证函数 base_shape_selftest 里设 GATE_BASE、取 merge-base、列文件（51），报错里点名 "ls-files" 的 .py 一行与内嵌 python 的出路文字里写着 git ls-files 的 .sh 一行，
#   两行行尾都带理由够长的 # change-range:exempt（52、53）：判绿，成功句报查了 4 份、行内豁免 2 行并逐行点名。
#   change-range-single-source-path-moves-red（不在 git 仓里）的搬迁登记表五行：日期不是 YYYY-MM-DD、只有三列、出处不在各一行，逐行点名判红；
#   前两行写法对，旧前缀还在磁盘上、新前缀不在磁盘上都不核（判不了哪几行生效）。
#   change-range-single-source-path-moves-green（不在 git 仓里）的表两行写法对、出处在：判绿报 2 行，成功句写判不了哪几行此刻生效、串与实据没核。
#   change-range-single-source-path-moves-evidence-red：setup.sh 建 git 仓提交两份实现 tools/real-a、tools/real-b、起 A 的 ops/launch.sh、old/x/f.sh、old/y/g.sh、rig/run.sh
#   与写着 build/ 的 .gitignore 作基准；删掉 tools/real-a、ops/launch.sh 改成起 B，old/x、old/y 下两份搬到 new/ 下、另拷一份 other/f.sh，删掉 rig、放一份没进 git 的 build/rig/run.sh；
#   表五行：tools/real-a → tools/real-b（新前缀下基准里本来就有文件，不生效，又是新行，判红「这一行不是一次搬迁」）、tools/real-a → tools/gone（落点不在）、old → new 与 old/x → other（串了）、
#   rig → build/rig（落点被 .gitignore 忽略），四处逐条点名判红。
#   change-range-single-source-path-moves-evidence-green：setup.sh 建 git 仓，基准里有 scripts/old-rig 下三份与一张登记了 scripts/ancient-rig → scripts/old-rig 的表；
#   git mv 到 scripts/new-rig，三份里两份只换了路径、一份另改了一行，表加一行 scripts/old-rig → scripts/new-rig：判绿，成功句写此刻生效 1 行（3 份里 2 份换算之后相同）、
#   不生效 1 行（第 2 行）。
#   change-range-single-source-path-moves-committed-green：setup.sh 建 git 仓，把 scripts/old-rig 搬到 scripts/new-rig、登记，连同登记表一起提交（这一批提交之后）：
#   这一行不生效，判绿，成功句写此刻生效的 0 行、不生效 1 行。
#   change-range-single-source-path-moves-base-row-green：setup.sh 建 git 仓，搬迁与登记表一起提交之后，scripts/new-rig 又改名、旧名字 scripts/old-rig 下另起一份新东西，再提交一次：
#   基准树满足「旧前缀下有文件、新前缀下没有文件」，可那一行已在基准那一版的登记表里，不生效，判绿，成功句写此刻生效的 0 行、点名「已在基准 HEAD 那一版的登记表里」。
#   change-range-single-source-split-gate-green（不在 git 仓里）：入口 60-样本拆分的门禁.sh 里定义带 quotepath 的包装函数 git_quoted，它拆出的库 lib/60-样本拆分的门禁-cells.sh 里经它列路径（--name-only）；
#   入口 code-tooling.sh 拆出的库 lib/code-tooling-样本-cells.sh 里有这一格自己的那一段（标题行起、到下一格的标题行为止，里面是要抓的写法）；
#   lib/unrelated-cells.sh 不在任何一道门禁的族里、里面写着不带 quotepath 的列路径，不扫、成功句点名它：判绿，成功句报查了 4 份、其中库 2 份、没扫的库 1 份。
#   change-range-single-source-split-gate-red（不在 git 仓里）：入口 61-样本红的门禁.sh 没有带 quotepath 的包装函数，它拆出的库 lib/61-样本红的门禁-cells.sh 里自己取 merge-base（第 3 行）、
#   列路径不带 quotepath（第 4 行），两处各点名一行；族外的 lib/unrelated-61-cells.sh 里同样的写法不扫（共 1 行算基准、1 行列路径）。
# 弄坏开关 CODE_TOOLING_BREAK=path-moves-unchecked：不判 ④，change-range-single-source-path-moves-red 与 change-range-single-source-path-moves-evidence-red 判绿（期望退出 1，实测 0），
#   change-range-single-source-path-moves-green、change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 里「✓ 搬迁登记表 …」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=change-range-docstrings-judged：文档串照代码判，change-range-single-source-prose-green 判红（期望退出 0，开关下退 1：50 的文档串几行被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=change-range-selftest-bodies-judged：自证函数体照代码判，change-range-single-source-prose-green 判红（期望退出 0，开关下退 1：51 的自证函数体几行被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=change-range-exempt-marker-ignored：行尾的豁免不认，change-range-single-source-prose-green 判红（期望退出 0，开关下退 1：52、53 带豁免的两行被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=change-range-libraries-by-name：门禁拆出的库只按 code-tooling-、crash-point-replay-、code-source-discipline- 三个库名的通配扫，不照拆分族归到每道门禁：
#   change-range-single-source-split-gate-red 判绿（期望退出 1，实测 0：61-样本红的门禁-cells.sh 没扫），change-range-single-source-split-gate-green 里「查了 .claude/gate.d/ 下 4 份」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=change-range-own-cell-in-entry-only：这一格自己的那一段只在入口里除掉，拆出的库里的不除：change-range-single-source-split-gate-green 判红
#   （期望退出 0，开关下退 1：lib/code-tooling-样本-cells.sh 里这一格自己的那一段里要抓的写法被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=change-range-names-by-file：带 quotepath 的变量与包装函数只认调用那一份文件自己里的，不认同一道门禁的族里别的文件：
#   change-range-single-source-split-gate-green 判红（期望退出 0，开关下退 1：lib/60-样本拆分的门禁-cells.sh 里经入口的 git_quoted 列路径那一行被点名）。
# 共用库 lib-lag-baseline.py 的开关 LAG_BASELINE_BREAK 各项（写法在那份文件头）在这一格样本上红在哪一句；逐条点名判错的样本在文件头开关表：
#   move-evidence-unchecked：change-range-single-source-path-moves-evidence-red 判绿（期望退出 1，实测 0），
#     change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 里「此刻生效」那条 want 找不到，
#     change-range-single-source-path-moves-green 里「判不了哪几行此刻生效」那条 want 找不到（不核时成功句换了说法）
#   window-ignored：change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 判红（期望退出 0，实测 1：
#     已提交的那一行当生效、实据红）；change-range-single-source-path-moves-evidence-red 里「有 4 处不对」那条 want 找不到
#   occupied-unchecked：change-range-single-source-path-moves-evidence-red 里「第 2 行的新前缀 tools/real-b 下有基准 HEAD 里本来就在的文件」那条 want 找不到（tools/real-a → tools/real-b 当生效，与下一行串了）
#   chain-allowed：change-range-single-source-path-moves-evidence-red 里「第 4 行与第 5 行串了」与「有 4 处不对」两条 want 找不到
#   untracked-landing-accepted：change-range-single-source-path-moves-evidence-red 里「没被 git 跟踪」与「有 4 处不对」两条 want 找不到
#   idle-new-row-unchecked：change-range-single-source-path-moves-evidence-red 里「这一行不是一次搬迁」与「有 4 处不对」两条 want 找不到（不在基准登记表里的新行不生效，不判红）
#   base-row-stays-effective：change-range-single-source-path-moves-evidence-green（基准登记表里的 scripts/ancient-rig → scripts/old-rig 当新行，新前缀下基准里早有文件）、
#     change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 判红（期望退出 0，实测 1：基准登记表里的老行照样生效、实据红）
cell_change_range_single_source() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
python3 - "$STAGE_NAME" "$STAGE_DIRECTORY/lib-lag-baseline.py" "${CODE_TOOLING_BREAK:-}" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import ast, glob, importlib.util, os, re, sys

own_name = sys.argv[1]
lag_baseline_library, broken = sys.argv[2], sys.argv[3]
split_family_library_file = sys.argv[4]
split_family_specification = importlib.util.spec_from_file_location('lib_lag_split_family', split_family_library_file)
split_family = importlib.util.module_from_spec(split_family_specification)
try:
    split_family_specification.loader.exec_module(split_family)
except (OSError, ImportError, SyntaxError) as error:
    print(f'  ✗ 载不进 {split_family_library_file}：{error}')
    print('     → 怎么办：它随仓走（scripts/gate/lib_lag_split_family.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。')
    sys.exit(1)
docstrings_judged = broken == 'change-range-docstrings-judged'
selftest_bodies_judged = broken == 'change-range-selftest-bodies-judged'
exempt_marker_ignored = broken == 'change-range-exempt-marker-ignored'
STAGE_DIRECTORY_PATH = '.claude/gate.d'
# 这一道文件里这一格自己的那一段（正则与出路里全是它要抓的写法）：从这一格的标题行到下一格的标题行，按空行判，行号不变；
# 这一道拆成入口加 .claude/gate.d/lib/ 下的库之后，这一段在哪份库里都一样处理：认这一道的拆分族（scripts/gate/lib_lag_split_family.py）里的每份文件
OWN_CELL_START = re.compile(r'^# ── 格 change-range-single-source：')
NEXT_CELL_START = re.compile(r'^# ── (格 |汇总)')
BASE_PATTERN = re.compile(r'merge-base|@\{u(pstream)?\}|gate-ok|\$\{?GATE_(DIFF_)?BASE\b|environ[^\n]*GATE_(DIFF_)?BASE')
# 写死的基准（HEAD~N）与三点号（A...B 就是取 merge-base）只在跑 git 的那一行上算：文档字符串里的「...」不算
BASE_IN_GIT_PATTERN = re.compile(r'\bHEAD(~\d|\^(?!\{))|\S\.\.\.(\s|$|[^.])')
RUNS_GIT_ANYWHERE = re.compile(r'\bgit\b|\bGIT\b')
BASE_ALLOWED = re.compile(r'merge-base["\']?[\s,]+["\']?--is-ancestor')
LIST_PATTERN = re.compile(r'--name-only|--name-status|\bls-files\b')
QUOTED_LIST_FLAG = re.compile(r"""["'](--name-only|--name-status|ls-files)["']""")
# 以引号开头的续行里，取基准的参数单独成一个带引号的参数（python 参数列表换了行）：这种行是代码，不是出路文字
QUOTED_BASE_ARGUMENT = re.compile(r"""["'](merge-base|@\{u(pstream)?\}|HEAD(~\d+|\^(?!\{)[^"']*)|[^"'\s]+\.\.\.[^"'\s]*)["']""")
TEXT_PREFIX = re.compile(r"""^(#|echo\b|printf\b|print\(|bad\b|howto\b|die\b|["'])""")
RUNS_GIT = re.compile(r'\$\(\s*git\b')
QUOTEPATH_FALSE = re.compile(r'quotepath=false', re.I)
QUOTE_VARIABLE = re.compile(r'^\s*([A-Za-z_][A-Za-z_0-9]*)\s*\+?=.*quotepath=false', re.I)
WRAPPER_HEAD = re.compile(r'^\s*(?:def\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(|(?:function\s+)?([A-Za-z_][A-Za-z_0-9]*)\s*\(\)\s*\{?)')
GIT_Z = re.compile(r"""\bgit\b[^|;&]*\s-z\b|["']-z["']""")
SHARED_CALL = re.compile(r'\b(gate_changed_paths|gate_added_lines)\b')
STATUS_HANDLED = re.compile(r'\|\||^\s*(if|elif|while|until|!)\b')
WRAPPER_REACH = 3
LOOKBACK = 3
CONTINUATION = re.compile(r"""^\s*["'\[]""")
# 行内豁免：命中的那一行不是调用，行尾写「# change-range:exempt <理由>」，理由至少 EXEMPT_REASON_MINIMUM 个字
EXEMPT_MARKER = re.compile(r'#\s*change-range:exempt\b[ \t]*(?P<reason>.*?)\s*$')
EXEMPT_REASON_MINIMUM = 8
SELFTEST_FUNCTION_NAME = re.compile(r'self_?test')

def is_text(stripped):
    """出路文字与注释：以这些开头、而且这一行不跑 git、不带引号括起的列路径参数的。"""
    return (bool(TEXT_PREFIX.match(stripped)) and not RUNS_GIT.search(stripped) and not QUOTED_LIST_FLAG.search(stripped)
            and not QUOTED_BASE_ARGUMENT.search(stripped))

def quoting_names(all_lines):
    """同一个文件里带着 quotepath=false 定义的变量 / 列表，与带着它的包装函数：(变量名集合, 函数名集合)。"""
    variables, wrappers = set(), set()
    for index, line in enumerate(all_lines):
        variable = QUOTE_VARIABLE.match(line)
        if variable:
            variables.add(variable.group(1))
        wrapper = WRAPPER_HEAD.match(line)
        if wrapper and any(QUOTEPATH_FALSE.search(body) for body in all_lines[index:index + 1 + WRAPPER_REACH]):
            wrappers.add(wrapper.group(1) or wrapper.group(2))
    return variables, wrappers

def uses_quoting(line, variables, wrappers):
    if QUOTEPATH_FALSE.search(line) or GIT_Z.search(line):
        return True
    if any(re.search(r'\$\{?' + re.escape(name) + r'\b|\*' + re.escape(name) + r'\b|\b' + re.escape(name) + r'\s*(\+|\[)', line)
           for name in variables):
        return True
    # 包装函数认「这一行调的是它」：python 写成「名字(」，bash 写在命令位置上。名字恰好是 git 时，
    # 「git(」认，裸的 git 命令与 ['git', …] 列表不认——doc-experiments.sh 的 decision-links 格的 def git(*args) 就是这种。
    for name in wrappers:
        if re.search(r'(?<![\w\'"])' + re.escape(name) + r'\(', line):
            return True
        if name != 'git' and re.search(r'(^|[;&|(]|\$\()\s*' + re.escape(name) + r'\s', line.strip()):
            return True
    return False

def blank_span(all_lines, start, end):
    """把语法树给的 (行, 字节列) 到 (行, 字节列) 那一段换成空格：行号与这一段之外的字符不变（语法树的列按 UTF-8 字节数）。"""
    (start_row, start_byte), (end_row, end_byte) = start, end
    for row in range(start_row, end_row + 1):
        line = all_lines[row - 1]
        encoded = line.encode('utf-8')
        first = len(encoded[:start_byte].decode('utf-8', 'replace')) if row == start_row else 0
        last = len(encoded[:end_byte].decode('utf-8', 'replace')) if row == end_row else len(line)
        all_lines[row - 1] = line[:first] + ' ' * (last - first) + line[last:]

def blank_python_prose(all_lines):
    """python 的文档串（模块、类、函数开头那一句字符串）与自证函数体（名字带 selftest / self_test）换成空白，行号不变；读不成语法树的整份照原样判。"""
    try:
        tree = ast.parse('\n'.join(all_lines))
    except (SyntaxError, ValueError):
        return
    for node in ast.walk(tree):
        body = getattr(node, 'body', None)
        if not isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) or not body:
            continue
        if not selftest_bodies_judged and isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) \
                and SELFTEST_FUNCTION_NAME.search(node.name):
            for row in range(body[0].lineno, node.end_lineno + 1):
                all_lines[row - 1] = ''
            continue
        first = body[0]
        if not docstrings_judged and isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant) \
                and isinstance(first.value.value, str):
            blank_span(all_lines, (first.lineno, first.col_offset), (first.end_lineno, first.end_col_offset))

def prepared_lines(path):
    """path 的各行，行号不变：这一道的这一格自己的那一段换成空行（own_family_paths 里的文件），python 的文档串与自证函数体换成空白。"""
    with open(path, encoding='utf-8', errors='replace') as handle:
        all_lines = handle.read().split('\n')
    if path in own_family_paths:
        inside = False
        for index, line in enumerate(all_lines):
            if OWN_CELL_START.match(line):
                inside = True
            elif inside and NEXT_CELL_START.match(line):
                inside = False
            if inside:
                all_lines[index] = ''
    if path.endswith('.py'):
        blank_python_prose(all_lines)
    return all_lines

def judge(path, all_lines=None, shared_names=None):
    """返回 (算基准的行, 不带 quotepath 的列路径行, 不判退出码的共用取法调用, 行内豁免了的 (行号, 理由))；
    all_lines 是 prepared_lines 读好的，shared_names 是同一道门禁的族（入口加拆出的库）里带 quotepath 的变量与包装函数，不给就只认 path 自己里的。"""
    if all_lines is None:
        all_lines = prepared_lines(path)
    variables, wrappers = quoting_names(all_lines)
    if shared_names is not None:
        variables, wrappers = variables | shared_names[0], wrappers | shared_names[1]
    base_hits, list_hits, status_hits, exempt_hits = [], [], [], []
    for index, line in enumerate(all_lines):
        stripped = line.strip()
        if not stripped or is_text(stripped):
            continue
        number = index + 1
        runs_git_here = RUNS_GIT_ANYWHERE.search(line) or (CONTINUATION.match(line) and QUOTED_BASE_ARGUMENT.search(line))
        base_hit = bool((BASE_PATTERN.search(line) or BASE_IN_GIT_PATTERN.search(line) and runs_git_here)
                        and not BASE_ALLOWED.search(line))
        list_hit = False
        if LIST_PATTERN.search(line) and not uses_quoting(line, variables, wrappers):
            # 参数列表换了行（这一行以引号或方括号开头）：往上找这条语句开头的那一行，看调用前缀带没带选项
            head = index
            while head > 0 and index - head < LOOKBACK and CONTINUATION.match(all_lines[head]):
                head -= 1
            list_hit = head == index or not uses_quoting(all_lines[head], variables, wrappers)
        status_hit = bool(SHARED_CALL.search(line) and not re.match(r'^\s*(gate_changed_paths|gate_added_lines)\s*\(\)', line)
                          and not STATUS_HANDLED.search(line))
        if not (base_hit or list_hit or status_hit):
            continue
        marker = None if exempt_marker_ignored else EXEMPT_MARKER.search(line)
        if marker and len(marker.group('reason')) >= EXEMPT_REASON_MINIMUM:
            exempt_hits.append((number, marker.group('reason')))
            continue
        if base_hit:
            base_hits.append((number, stripped))
        if list_hit:
            list_hits.append((number, stripped))
        if status_hit:
            status_hits.append((number, stripped))
    return base_hits, list_hits, status_hits, exempt_hits

gate_files = sorted(path for pattern in ('.claude/gate.d/*.sh', '.claude/gate.d/*.py') for path in glob.glob(pattern))
library_files = sorted(path for pattern in ('.claude/gate.d/lib/*.sh', '.claude/gate.d/lib/*.py') for path in glob.glob(pattern))
stage_files = [path for path in gate_files if path.endswith('.sh')]
# 每道门禁连同它拆出的全部库：库归哪道门禁由拆分族认（scripts/gate/lib_lag_split_family.py 的 family_members），不按库名抄
families = {stage: [stage] + split_family.family_members(stage, library_files) for stage in stage_files}
if broken == 'change-range-libraries-by-name':
    library_scanned = sorted(path for pattern in ('.claude/gate.d/lib/code-tooling-*.sh', '.claude/gate.d/lib/crash-point-replay-*.sh',
                                                  '.claude/gate.d/lib/code-source-discipline-*.sh') for path in glob.glob(pattern))
else:
    library_scanned = sorted({library for members in families.values() for library in members[1:]})
unowned_libraries = [path for path in library_files if path not in library_scanned]
scanned = sorted(gate_files + library_scanned)
own_stage_path = f'{STAGE_DIRECTORY_PATH}/{own_name}'
own_family_paths = {own_stage_path}
if broken != 'change-range-own-cell-in-entry-only':
    own_family_paths |= set(split_family.family_members(own_stage_path, library_files))
group_of = {path: members for members in families.values() for path in members}
lines_of = {path: prepared_lines(path) for path in scanned}
names_of = {path: quoting_names(lines) for path, lines in lines_of.items()}

def shared_quoting_names(path):
    """path 所在的族里每份文件带 quotepath 的变量与包装函数合起来（拆分之前在同一份文件里）；弄坏开关 change-range-names-by-file 下只认 path 自己里的。"""
    members = [path] if broken == 'change-range-names-by-file' else group_of.get(path, [path])
    variables, wrappers = set(), set()
    for member in members:
        if member in names_of:
            variables |= names_of[member][0]
            wrappers |= names_of[member][1]
    return variables, wrappers

base_bad, list_bad, status_bad, exempt_all = [], [], [], []
for path in scanned:
    base_hits, list_hits, status_hits, exempt_hits = judge(path, lines_of[path], shared_quoting_names(path))
    exempt_all += [(path, number, reason) for number, reason in exempt_hits]
    base_bad += [(path, number, line) for number, line in base_hits]
    list_bad += [(path, number, line) for number, line in list_hits]
    status_bad += [(path, number, line) for number, line in status_hits]

failed = False
if base_bad:
    failed = True
    print(f'  ✗ {len(base_bad)} 行在阶段里自己算 diff 基准（改动范围只许一份取法：scripts/gate/changed-paths.sh）：')
    for path, number, line in base_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：阶段里 source scripts/gate/changed-paths.sh，基准写 base="$(gate_diff_base gate)"（问「这一次提交带哪些」的写 head），')
    print('               路径写 gate_changed_paths、新增行写 gate_added_lines；共用脚本缺哪种取法就往它里面加，连同它的 --selftest 一起改。')
if list_bad:
    failed = True
    print(f'  ✗ {len(list_bad)} 行列路径的 git 调用没带 -c core.quotepath=false：')
    for path, number, line in list_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：在那一行的 git 后面加 -c core.quotepath=false（python 里是 ["git", "-c", "core.quotepath=false", …]），')
    print('               或者把带它的调用前缀收进一个变量 / 列表再用；不带它时中文路径变成带引号的八进制串，与仓里的路径逐字对不上。')
if status_bad:
    failed = True
    print(f'  ✗ {len(status_bad)} 处调共用取法（gate_changed_paths / gate_added_lines）没判退出码：')
    for path, number, line in status_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：写成 x="$(gate_changed_paths …)" || { echo 取不到改动范围…; echo 出路…; exit 1; }，或放进 if；')
    print('               git 失败时共用脚本退非 0、什么都不交，不判退出码就会把「没取到」读成「这次什么都没改」。')
if base_bad or list_bad or status_bad:
    print('     → 命中的那一行不是调用（报错文字里点名失败的是哪条子命令这类）：行尾写 # change-range:exempt <为什么不是调用，至少 8 个字>；'
          'python 的文档串与自证函数体（名字带 selftest）本来不判')
# ④ 搬迁登记表（判法 lib-lag-baseline.py 的 path_move_problems，用这一道所在仓里那一份；表取被判的根里那一份）
move_count, move_problems, move_note = 0, [], ''
if broken != 'path-moves-unchecked':
    try:
        lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', lag_baseline_library)
        lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
        lag_baseline_spec.loader.exec_module(lag_baseline)
        move_count, move_problems, move_note = lag_baseline.path_move_report('.')
    except (OSError, ImportError, AttributeError, SyntaxError, ValueError) as error:
        move_problems = [(f'载不进 {lag_baseline_library}：{type(error).__name__}: {error}',
                          '它随仓走（.claude/gate.d/lib-lag-baseline.py），被删了就从 git 找回来；搬迁登记表的判法只在那一份里写')]
if move_problems:
    failed = True
    print(f'  ✗ 搬迁登记表 .claude/gate.d/path-moves.tsv 有 {len(move_problems)} 处不对（共 {move_count} 行）：')
    for what, _howto in move_problems:
        print(f'      {what}')   # gate-lint:detail
    print('     → 怎么办：' + '；'.join(dict.fromkeys(howto for _what, howto in move_problems)))
if exempt_all:
    print(f'     行内豁免 {len(exempt_all)} 行（行尾 # change-range:exempt 写了理由）：'
          + '；'.join(f'{path}:{number}（{reason}）' for path, number, reason in exempt_all))
else:
    print('     行内豁免 0 行')
if docstrings_judged or selftest_bodies_judged or exempt_marker_ignored:
    print(f'  ! 弄坏开关 {broken} 生效：python 的文档串、自证函数体或行内豁免照代码判')
if failed:
    sys.exit(1)

outside = []
for pattern in ('scripts/*/*.sh', 'scripts/*/*.py', '.claude/scripts/*', '.claude/hooks/*'):
    for path in sorted(glob.glob(pattern)):
        if os.path.isfile(path) and path != 'scripts/gate/changed-paths.sh':
            base_hits, list_hits, status_hits, _exempt_hits = judge(path)
            if base_hits or list_hits or status_hits:
                outside.append(f'{path}（算基准 {len(base_hits)} 行、不带 quotepath 的列路径 {len(list_hits)} 行、不判退出码 {len(status_hits)} 处）')
print(f'  ✓ 查了 .claude/gate.d/ 下 {len(scanned)} 份：没有阶段自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false，调共用取法都判了退出码；python 的文档串与自证函数体（名字带 selftest）不判')
print(f'     其中门禁拆出的库 {len(library_scanned)} 份（照拆分族归到 {len(stage_files)} 道门禁）')
if unowned_libraries:
    print(f'     没扫的库 {len(unowned_libraries)} 份（不在任何一道门禁的拆分族里）：' + '、'.join(unowned_libraries))
if broken == 'path-moves-unchecked':
    print('  ! 弄坏开关 path-moves-unchecked 生效：搬迁登记表 .claude/gate.d/path-moves.tsv 没判')
else:
    print(f'  ✓ 搬迁登记表 .claude/gate.d/path-moves.tsv {move_count} 行：写法对、出处都在；{move_note}')
if outside:
    print(f'     没扫的 {len(outside)} 份（不是门禁阶段，射程之外）：' + '；'.join(outside))
else:
    print('     没扫的：scripts/、.claude/scripts/、.claude/hooks/ 下没有碰到这两条的脚本')
PY
}

# ── 格 research-gate-lint：scripts/、.claude/hooks/ 与 .claude/scripts/ 里每一条拒绝都带出路、守 shell 纪律；前两个目录的执行位不丢；连同 .claude/gate.d/，终止进程只许点名一个
#
# 共享门禁的「门禁自检」只把 SOP 自己的脚本与 .claude/gate.d/ 交给 gate-lint 与 shell-lint，这三个目录不在它的射程里，这里补上
#   （.claude/singlefs-ai-sop/rules/show-me-test.md「射程只到 .claude/gate.d/」）；两个 lint 都跑完再判，不在第一个红就停。
#   .claude/scripts/ 的执行位仍归共享那一道，这里不重复扫。research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）。
# 进程安全（终止只许点名一个自己起的进程号或任务号）：判定只有一份，在 .claude/hooks/bash-command-detector.sh（拒绝 ⑥），
#   这里经它的 --scan-scripts 扫，射程比两个 lint 多一个 .claude/gate.d/（它的样本目录不扫）。--scan-scripts 只扫给它的那一层：待改清单 .claude/process-safety-pending 里
#   所在目录这一趟没扫的行判本次未判（它点名那一行并往 $GATE_NOT_RUN_FILE 报一行），不判成「一处要放行的都没有」。
# 样本：research-gate-lint-red 放一个只带一句话的 die 和一个靠子 shell 赋值往外带值的函数；它的 .claude/scripts/ 里再放一个不带出路的拒绝和一处 pkill -f
#   （这个目录进了射程才报得出来）；进程安全那一路有 scripts/gate/stop-processes.sh（自己任务的进程组、循环里 kill、systemctl 带通配）、
#   scripts/gate/stop.py（循环 os.kill、os.killpg）与 .claude/gate.d/50-stop-group.sh（按 cgroup.procs 批量发、没核 scope 也没标），都必须逐处报出。
#   research-gate-lint-green 的 die 带第二个参数、不靠子 shell 带值，scripts/gate/sample.sh 里另有一行 `if ! cd "$here/sub"; then … fi`（cd 是条件，不算裸 cd；只放在 green 里，
#   下面几份「是 green 加 …」的样本没有这一行），.claude/scripts/ 里放一个干净的包装（成功行的脚本数要把它数进去），
#   scripts/gate/stop-own.sh 只停点名的一个、按 cgroup 批量发的那一段核过 singlefs-memory-cap-*.scope 又标了 own-scope，必须判绿。
#   research-gate-lint-pending-outside-green 是 green 加 scripts/gate/deeper/killer.sh（循环里 kill，所在目录进程安全那一路不扫）和 .claude/process-safety-pending 的一行点名它：
#   清单里落在这一趟没扫的目录下的行判本次未判（bash-command-detector.sh --scan-scripts 点名并往 $GATE_NOT_RUN_FILE 报一行），不判成「一处要放行的都没有」，必须判绿；
#   research-gate-lint-pending-stale-red 是 green 加 .claude/process-safety-pending 一行点名 scripts/gate/sample.sh（扫了的目录里、一处要放行的都没有），必须仍判红。
#   弄坏开关 BASH_COMMAND_DETECTOR_BREAK=pending-outside-scanned-as-stale 归钩子、由钩子自己的 --selftest 证红，不列进文件头开关表：钩子平常跑设了它就退 2（只在自证里认），
#   经这一道喂样本证不出那一条判据（进程安全那一路没跑成，research-gate-lint 的 green、red、pending-stale-red、pending-outside-green 一起判错）。
#   research-gate-lint-arithmetic-shift-red 是 green 加一个 scripts/gate/arithmetic-shift.sh：先写 `(( shift_bits = 1 << width ))` 与 `mask=$(( 1 << bits ))`，后面一行裸 cd 没接 || 也没接 &&，
#   再跟一个真 heredoc（正文里有一行 cd，不判）；必须只报第 5 行那一处。
cell_research_gate_lint() {
LINT="$(cd "$STAGE_DIRECTORY/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"
SHELL_LINT="$(dirname "$LINT")/shell-lint.sh"
for script in "$LINT" "$SHELL_LINT"; do
  [[ -f "$script" ]] || { echo "  ✗ 找不到共享脚本 $script"; echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh"; exit 1; }
done
TARGETS=()
MODE_TARGETS=()
ABSENT=()
for directory in "$ROOT/scripts" "$ROOT/.claude/hooks" "$ROOT/.claude/scripts"; do
  [[ -d "$directory" ]] || { ABSENT+=("${directory#"$ROOT"/}"); continue; }
  TARGETS+=("$directory")
  [[ "$directory" == "$ROOT/.claude/scripts" ]] || MODE_TARGETS+=("$directory")
done
((${#TARGETS[@]})) || { echo "  ! 没有 scripts/、.claude/hooks/ 也没有 .claude/scripts/，这一格无对象可判"; exit 77; }
# GATE_LINT_DIR 指到第一个目标目录：不设它，共享脚本默认还会扫 SOP 自己的包，样本目录里红绿就靠那个包碰巧干净来分。
failed=0
if ! GATE_LINT_DIR="${TARGETS[0]}" bash "$LINT" "${TARGETS[@]:1}"; then
  echo "  ✗ scripts/、.claude/hooks/ 或 .claude/scripts/ 里有不带出路的拒绝（上面逐处列出）"
  echo "     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，直接打印的拒绝之后跟一行以箭头开头的出路"
  failed=1
fi
# 执行位也一起：共享门禁的「脚本执行位」阶段只扫 .claude/gate.d 与 .claude/scripts，
# scripts 与 .claude/hooks 不在它的射程里（同 gate-lint / shell-lint 那两条的理由）；.claude/scripts 已在那一道里，不重复扫。
MODES="$(dirname "$LINT")/script-modes.sh"
modes_note=""
if [[ ! -f "$MODES" ]]; then
  echo "  ✗ 找不到共享脚本 $MODES，scripts/ 与 .claude/hooks/ 的执行位这一项没判"
  echo "     → 怎么办：规范副本缺了这一份，把 singlefs-ai-sop-<语言> 仓重新拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh；别让这一项静默不判"
  failed=1
  modes_note="执行位：没判（找不到 script-modes.sh，上面已判红）"
elif ((${#MODE_TARGETS[@]} == 0)); then
  modes_note="执行位：本次无对象可判（射程里没有 scripts/ 也没有 .claude/hooks/）"
else
  modes_rc=0
  bash "$MODES" "${MODE_TARGETS[@]}" || modes_rc=$?
  if (( modes_rc == 77 )); then
    modes_note="执行位：本次未判（script-modes.sh 退 77，原因见它上面那一句）"
  elif (( modes_rc != 0 )); then
    echo "  ✗ scripts/ 或 .claude/hooks/ 里的脚本执行位在暂存区里不对（上面逐处列出）"
    echo "     → 怎么办：用 git update-index --chmod=+x <路径>（或 -x）改暂存区里的模式，让它与工作区一致"
    failed=1
    modes_note="执行位：判红"
  else
    modes_note="执行位：判过（${#MODE_TARGETS[@]} 个目录）"
  fi
fi
# 进程安全：同一份判定（bash-command-detector.sh --scan-scripts），连 .claude/gate.d/ 一起扫；待改清单是被判仓的 .claude/process-safety-pending
DETECTOR="$(cd "$STAGE_DIRECTORY/../hooks" && pwd)/bash-command-detector.sh"
SAFETY_TARGETS=("${TARGETS[@]}")
# --scan-scripts 只扫给它的那一层（不进子目录）；scripts/ 的脚本放在分类子目录 scripts/<类>/ 里（scripts/README.md），逐个子目录交给它。
# 射程到 scripts/<类>/ 这一层为止：实验装置那一层（scripts/experiment-apparatus/<装置>/）与 scripts/devices/qemu/、scripts/mutation/tables/ 不进（搬迁前它们不在 research/scripts/ 里，也不扫）。
# 弄坏开关 CODE_TOOLING_BREAK=safety-top-level-only 不交子目录（搬迁前的样子，scripts/ 顶层没有脚本，这一路扫空）。
if [[ -d "$ROOT/scripts" && "${CODE_TOOLING_BREAK:-}" != safety-top-level-only ]]; then
  for directory in "$ROOT"/scripts/*/; do
    [[ -d "$directory" ]] && SAFETY_TARGETS+=("${directory%/}")
  done
fi
[[ -d "$ROOT/.claude/gate.d" ]] && SAFETY_TARGETS+=("$ROOT/.claude/gate.d")
if [[ ! -f "$DETECTOR" ]]; then
  echo "  ✗ 找不到 $DETECTOR，终止进程的写法这一项没扫"
  echo "     → 怎么办：恢复 .claude/hooks/bash-command-detector.sh（判定只有那一份，门禁与执行前的钩子共用），再跑这一格：bash .claude/gate.d/${STAGE_NAME} --check research-gate-lint"
  failed=1
elif ! bash "$DETECTOR" --scan-scripts "$ROOT" "${SAFETY_TARGETS[@]}"; then
  echo "  ✗ scripts/、.claude/hooks/、.claude/scripts/ 或 .claude/gate.d/ 里有打得到别人进程的终止写法（上面逐处列出）"
  echo "     → 怎么办：照上面那句「怎么办」改；规矩是「后面的脚本不能终止前面的脚本」「不能动 ssh」「终止要按照任务号终止 禁止终止所有」，写法见 .claude/hooks/bash-command-detector.sh 文件头 ⑥"
  failed=1
fi
# 语法：射程里（连 .claude/gate.d/，不进 fixtures/ 与编译目录 target/）每一份 .sh 过 bash -n、每一份 .py 过 python3 的 compile（只编译、不执行）；
# 搬迁、改名把一份脚本改坏了，提交时红在这里，不等到有人去跑它。弄坏开关 CODE_TOOLING_BREAK=syntax-unchecked 不判这一项
syntax_note="语法：没判（弄坏开关 syntax-unchecked）"
syntax_directories=("${TARGETS[@]}")
[[ -d "$ROOT/.claude/gate.d" ]] && syntax_directories+=("$ROOT/.claude/gate.d")
if [[ "${CODE_TOOLING_BREAK:-}" != syntax-unchecked ]]; then
  # .sh 的 bash -n 一份一个进程、彼此独立：按每台默认重活线程数同时跑（一个 python 进程管池，坏的按 find 的次序列在前）；.py 在同一个进程里接着 compile
  if ! syntax_jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），语法检查的并行度定不了，这一项没判"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  syntax_shell_files=(); syntax_python_files=(); syntax_bad=()
  while IFS= read -r -d '' file; do
    case "$file" in
      *.sh) syntax_shell_files+=("$file") ;;
      *.py) syntax_python_files+=("$file") ;;
    esac
  done < <(find "${syntax_directories[@]}" \( -name fixtures -o -name target \) -prune -o -type f \( -name '*.sh' -o -name '*.py' \) -print0 | sort -z)
  syntax_shell_count=${#syntax_shell_files[@]}
  if ((syntax_shell_count + ${#syntax_python_files[@]})); then
    while IFS= read -r line; do [[ -n "$line" ]] && syntax_bad+=("${line#"$ROOT"/}"); done < <(python3 - "$syntax_jobs" ${syntax_shell_files[@]+"${syntax_shell_files[@]}"} -- ${syntax_python_files[@]+"${syntax_python_files[@]}"} <<'PY'
import concurrent.futures, subprocess, sys
separator = sys.argv.index("--")
jobs, shell_files, python_files = int(sys.argv[1]), sys.argv[2:separator], sys.argv[separator + 1:]
def shell_message(path):
    done = subprocess.run(["bash", "-n", path], stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
    return None if done.returncode == 0 else f"{path}：{done.stdout.split(chr(10))[0]}"
if shell_files:
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(jobs, len(shell_files)))) as pool:
        for message in pool.map(shell_message, shell_files):
            if message:
                print(message)
for path in python_files:
    try:
        with open(path, encoding="utf-8") as source:
            compile(source.read(), path, "exec")
    except (SyntaxError, ValueError, UnicodeDecodeError) as error:
        line = getattr(error, "lineno", None)
        print(f"{path}：第 {line} 行 {type(error).__name__}：{getattr(error, 'msg', error)}")
PY
)
  fi
  if ((${#syntax_bad[@]})); then
    for entry in "${syntax_bad[@]}"; do echo "  ✗ 语法不对：$entry"; done  # gate-lint:detail
    echo "  ✗ 射程里有 ${#syntax_bad[@]} 份脚本过不了语法检查（bash -n 或 python 的 compile，上面逐份列出）"
    echo "     → 怎么办：照每一行报的那一行改（多半是搬迁、改名时替换吃掉了引号或括号），改完单跑这一格：bash .claude/gate.d/${STAGE_NAME} --check research-gate-lint"
    failed=1
  fi
  syntax_note="语法：查了 $((syntax_shell_count + ${#syntax_python_files[@]})) 份（sh ${syntax_shell_count}、py ${#syntax_python_files[@]}），坏 ${#syntax_bad[@]} 份"
fi
# 裸 cd：不开 set -e 的 .sh 里，命令位置上的 cd 后面要接 || 出路或 &&（搬迁之后 cd 进不去、接着在调用方的目录里跑，病因会报错地方）。
# 射程与语法那一项相同；heredoc 里的正文与注释行不判，反斜杠续行接起来判。heredoc 起始是 << 后接分隔词：算术段 $(( … )) 与 (( … )) 里的 << 是移位，<<< 是 here-string，都不算。
# cd 后面紧接 ; then 或 ; do 的（`if ! cd <目录>; then`、`while cd <目录>; do`）不判：cd 是条件里最后一条命令，它的退出码由 if / while 判，进不去就走另一支。
# 弄坏开关 CODE_TOOLING_BREAK=bare-cd-unchecked 不判这一项，heredoc-in-arithmetic 把算术里的 << 也当 heredoc 起始，bare-cd-condition-flagged 把条件里收尾的 cd 也判
bare_cd_note="裸 cd：没判（弄坏开关 bare-cd-unchecked）"
if [[ "${CODE_TOOLING_BREAK:-}" != bare-cd-unchecked ]]; then
  bare_cd_output="$(python3 - "$ROOT" "${syntax_directories[@]}" <<'PY'
import os, re, sys
def read_word(text, i):
    depth = 0; quote = None
    while i < len(text):
        c = text[i]
        if quote == "'":
            if c == "'": quote = None
        elif c == "\\": i += 1
        elif text.startswith("$(", i) or text.startswith("${", i): depth += 1; i += 1
        elif c in ")}" and depth > 0: depth -= 1
        elif c == '"': quote = None if quote == '"' else ('"' if quote is None else quote)
        elif c == "'" and quote is None: quote = "'"
        elif quote is None and depth == 0 and (c.isspace() or c in ";&|)}"): return i
        i += 1
    return i
def heredoc_delimiter(line):
    # 算术段 $(( … )) 与 (( … )) 里的 << 是移位，<<< 是 here-string，都不是 heredoc 起始；弄坏开关 heredoc-in-arithmetic 下把算术里的 << 也当 heredoc 起始
    if os.environ.get("CODE_TOOLING_BREAK") == "heredoc-in-arithmetic":
        opener = re.search(r"<<-?\s*['\"]?([A-Za-z_]+)['\"]?", line)
    else:
        blanked = re.sub(r"\$?\(\(.*?\)\)", lambda found: " " * len(found.group(0)), line)
        opener = re.search(r"(?<!<)<<(?!<)-?\s*['\"]?([A-Za-z_]+)['\"]?", blanked)
    return opener.group(1) if opener else None
def bare_cd_lines(text):
    if re.search(r'^\s*set\s+(-[a-zA-Z]*e|-o\s+errexit)', text, re.M): return []
    hits = []; joined = []; buffer = ""; first = None; heredoc = None
    for number, line in enumerate(text.split("\n"), 1):
        if heredoc:
            if line.strip() == heredoc: heredoc = None
            continue
        opener = heredoc_delimiter(line)
        if opener and not line.lstrip().startswith("#"): heredoc = opener
        if first is None: first = number
        if line.endswith("\\"): buffer += line[:-1] + " "; continue
        joined.append((first, buffer + line)); buffer = ""; first = None
    for number, line in joined:
        if line.lstrip().startswith("#"): continue
        for match in re.finditer(r'(?:^|[;&|({!`]|\b(?:then|do|else)\b)\s*cd\b', line):
            i = match.end()
            while i < len(line) and line[i] == " ": i += 1
            while i < len(line) and not line.startswith(("||", "&&", ";", "|", ")", "}", "#"), i):
                i = read_word(line, i)
                while i < len(line) and line[i] == " ": i += 1
            rest = line[i:]
            if rest.startswith(("||", "&&")) or re.match(r';\s*[})]\s*(\|\||&&)', rest): continue
            if re.match(r';\s*(then|do)\b', rest) and os.environ.get("CODE_TOOLING_BREAK") != "bare-cd-condition-flagged": continue
            hits.append((number, line.strip()[:120]))
    return hits
root = sys.argv[1]; scanned = 0
for top in sys.argv[2:]:
    for directory, subdirectories, files in os.walk(top):
        subdirectories[:] = sorted(d for d in subdirectories if d not in ("fixtures", "target"))
        for name in sorted(files):
            if not name.endswith(".sh"): continue
            path = os.path.join(directory, name); scanned += 1
            with open(path, encoding="utf-8", errors="replace") as source:
                for number, line in bare_cd_lines(source.read()):
                    print(f"bad\t{os.path.relpath(path, root)}:{number}\t{line}")
print(f"scanned\t{scanned}")
PY
)"
  bare_cd_scanned="$(awk -F'\t' '$1 == "scanned" { print $2 }' <<<"$bare_cd_output")"
  bare_cd_bad=()
  while IFS=$'\t' read -r kind location text; do
    [[ "$kind" == bad ]] && bare_cd_bad+=("$location：$text")
  done <<<"$bare_cd_output"
  if ((${#bare_cd_bad[@]})); then
    for entry in "${bare_cd_bad[@]}"; do echo "  ✗ 裸 cd：$entry"; done  # gate-lint:detail
    echo "  ✗ 射程里有 ${#bare_cd_bad[@]} 处 cd 没接 || 也没接 &&（脚本没开 set -e，cd 失败会在调用方的目录里接着跑）"
    echo "     → 怎么办：写成 cd <目录> || { echo \"进不去 <目录>\"; echo \"→ 怎么办：…\"; exit 2; }，或 (cd <目录> && …)；规矩在 .claude/rules/directory-layout.md「scripts/：反复要用的脚本」"
    failed=1
  fi
  bare_cd_note="裸 cd：查了 ${bare_cd_scanned:-0} 份 .sh，坏 ${#bare_cd_bad[@]} 处"
fi
# shell-lint 一次只扫一个目录（SHELL_LINT_DIR），逐个目录跑
for directory in "${TARGETS[@]}"; do
  if ! SHELL_LINT_DIR="$directory" bash "$SHELL_LINT"; then
    echo "  ✗ ${directory#"$ROOT"/} 里有违反 shell 纪律的写法（上面逐处列出）"
    echo "     → 怎么办：照每一处给的改法改，规则在 .claude/singlefs-ai-sop/rules/command-safety.md；是 lint 认错了的，到上游 singlefs-ai-sop 修 shell-lint 并加样本，不在本仓绕开"
    failed=1
  fi
done
if ((failed == 0)); then
  scanned_names=()
  for directory in "${SAFETY_TARGETS[@]}"; do scanned_names+=("${directory#"$ROOT"/}"); done
  echo "  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 ${#TARGETS[@]} 个目录（${scanned_names[*]:0:${#TARGETS[@]}}），拒绝都带出路、shell 纪律守住；进程安全连 .claude/gate.d/ 共扫 ${#SAFETY_TARGETS[@]} 个目录；${modes_note}；${syntax_note}；${bare_cd_note}"
  echo "    没扫的目录 ${#ABSENT[@]} 个（不在）：${ABSENT[*]:-（没有）}"
fi
exit "$failed"
}
