#!/usr/bin/env bash
# 门禁 code-source-discipline 的格函数库：vague-names、test-file-names、test-function-names、format-constants、clause-enums、feature-bits。
# 由 .claude/gate.d/code-source-discipline.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# bad、howto 由入口设好，在调用时才取。

# ── vague-names ──
# 判据：`git ls-files` 列出的 crates/**/*.rs 与 scripts/experiment-apparatus/**/*.rs 里，每一处 `fn <名>` 与 `struct|enum|type|trait <名>`：
# 函数名按 `_` 切词、类型名按大写字母切词，切出来的每一个词（小写）都在 .claude/naming-vague-words 里，判违规，逐处列「文件:行 名字」。
# 不判的：`impl <trait> for <类型>` 块里的方法与关联类型（名字是 trait 定的）、`main`、以 `\n\` 续行的字符串字面量里的源码（测试夹具）、宏展开出来的名字（认不出）。
# 还没改完名的文件逐个登记在 .claude/naming-vague-exclude（一行 <路径>  # 理由），那份文件里的违规不判；排除项指向不存在的文件、
# 或那份文件里已经一处违规都没有了，判红（排除只缩不涨）。一份 .rs 都没扫到：这一格本次无对象可判。
# 管不到的：名字里夹了一个非空泛词、而那个词本身也什么都没说（run_the_cell_again）——词表只认整名全空泛，这一类靠 review。
cell_vague_names() {
python3 - <<'PY'
import os, re, subprocess, sys

WORDS_FILE = ".claude/naming-vague-words"
EXCLUDE_FILE = ".claude/naming-vague-exclude"
BROKEN = os.environ.get("GATE_VAGUE_NAMES_BREAK", "")

def registry(path):
    entries = []
    if not os.path.isfile(path):
        return None
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        body = line.split("#", 1)[0].strip()
        if body:
            entries.append((number, body, line.split("#", 1)[1].strip() if "#" in line else ""))
    return entries

words = registry(WORDS_FILE)
if words is None:
    print(f"  ✗ 读不到空泛词表 {WORDS_FILE}")
    print("     → 怎么办：从 git 恢复它；这一格没有词表就什么都判不了，不是通过。")
    sys.exit(1)
vague = {word.lower() for _number, word, _reason in words}
excluded = registry(EXCLUDE_FILE) or []

listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates", "scripts/experiment-apparatus"],
                        capture_output=True)
paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path))
if not paths:
    print("  ! 本次无对象可判：crates/ 与 scripts/experiment-apparatus/ 下一份 .rs 都没列出来")
    sys.exit(77)

DECLARATION = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*(fn|struct|enum|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)")
TRAIT_IMPL = re.compile(r"^\s*impl\b.*\bfor\b")

def words_of(kind, name):
    if kind == "fn":
        return [part.lower() for part in name.split("_") if part]
    return [part.lower() for part in re.findall(r"[A-Z][a-z0-9]*|[a-z0-9]+", name)]

def violations_in(path):
    found = []
    depth = 0
    trait_impl_depths = []
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        code = line.split("//", 1)[0]
        if TRAIT_IMPL.match(code) and BROKEN != "trait-methods-judged":
            trait_impl_depths.append(depth)
        match = DECLARATION.match(code)
        inside_trait_impl = bool(trait_impl_depths) and depth > trait_impl_depths[-1]
        inside_string_literal = line.rstrip("\n").endswith("\\n\\") and BROKEN != "string-continuation-judged"
        if match and not (match.group(1) in ("fn", "type") and inside_trait_impl) and match.group(2) != "main" and not inside_string_literal:
            parts = words_of(match.group(1), match.group(2))
            if parts and all(part in vague for part in parts):
                found.append(f"{path}:{number} {match.group(1)} {match.group(2)}")
        depth += code.count("{") - code.count("}")
        while trait_impl_depths and depth <= trait_impl_depths[-1] and "}" in code:
            trait_impl_depths.pop()
    return found

excluded_paths = {entry for _number, entry, _reason in excluded}
problems, exclusion_problems = [], []
scanned = 0
for path in paths:
    scanned += 1
    found = violations_in(path)
    if path in excluded_paths and BROKEN != "exclusions-ignored":
        if not found:
            exclusion_problems.append(f"{EXCLUDE_FILE}：{path} 里已经一个空泛名都没有了，这一行删掉")
        continue
    problems += found
for number, entry, reason in excluded:
    if not os.path.isfile(entry):
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} 指向不存在的文件 {entry}")
    elif len(reason) < 4:
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 没写为什么先不改（# 后面至少 4 个字）")

for problem in problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if problems:
    print(f"  ✗ {len(problems)} 个函数名或类型名只由空泛词拼成（每个词都在 {WORDS_FILE} 里）")  # gate-lint:summary
    print("     → 怎么办：按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名：说出跑的是什么、判的是哪条、取的是哪个量；")
    print(f"                一时改不完的文件登记进 {EXCLUDE_FILE}（一行 <路径>  # 理由），改完删那一行。")
for problem in exclusion_problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if exclusion_problems:
    print(f"  ✗ {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由")  # gate-lint:summary
    print("     → 怎么办：指向不存在的与已经改完的那几行删掉；留着的每一行写明为什么先不改。排除只缩不涨。")
if problems or exclusion_problems:
    sys.exit(1)
print(f"  ✓ 没有只由空泛词拼成的函数名与类型名（扫了 {scanned} 份 .rs，词表 {len(vague)} 个词，排除 {len(excluded)} 份还没改完的文件）")
PY
}

# ── test-file-names ──
# 判据：scripts/crash/crash-case-check.py --only file-names <项目根>：crates/singlefs-verification/tests/*.rs
# 的文件名不带里程碑序数、步号、增补号、并行线号、欠账号（那份脚本的 MILESTONE_FILE_NAME），判法只在那一份里。
# 两个 tests/ 下一份 .rs 都没有：这一格本次无对象可判。
# 管不到的：按里程碑起、却不是那几种写法的名字（`milestone_three_*` 这类），靠 review；别的包的测试文件不扫。
cell_test_file_names() {
  local file_names_exit
  if python3 "$CRASH_CASE_CHECK" --only file-names "$ROOT"; then file_names_exit=0; else file_names_exit=$?; fi
  case "$file_names_exit" in
    0|77) return "$file_names_exit" ;;
  esac
  echo "  ✗ 测试文件名：crash-case-check --only file-names 退 $file_names_exit（逐处列在上面）"
  echo "     → 怎么办：照上面那一句出路改名；判法写在 scripts/crash/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」。"
  return 1
}

# ── test-function-names ──（还 checks-owed.md C638：用例函数名带欠账号没有判）
# 判据：scripts/crash/crash-case-check.py --only test-names <项目根>：crates/*/tests/ 下（含子目录）与 crates/*/src/ 下 .rs 里带 #[test] 的函数名，
# 拿与 test-file-names 同一条 MILESTONE_FILE_NAME 判（里程碑序数、步号、增补号、并行线号、欠账号 c<三位数>），判法只在那一份里。
# 立账时已在的存量登记在 .claude/gate.d/test-function-names-exempt.tsv（「<文件><TAB><函数名><TAB># 理由>」），登记了的不判，对不上的豁免行判红；表只缩不涨（不判涨，靠 review）。
# 一条 #[test] 都没有：这一格本次无对象可判。
# 管不到的：带实验号（e<数>）与 m<数>_ 的名字（算不算来历没定，不在那条判据里）；宏展开出来的测试。
# 样本：test-function-names-red（src/ 里一条 c123_、tests/ 子目录里一条带 _c789 的、豁免表里一行对不上，三处点名判红；登记了的 c456_ 不报）；
#   test-function-names-green（一条按内容起名的、一条登记了豁免的 c456_，报「查了 2 条……豁免……1 条不判」）。
cell_test_function_names() {
  local test_names_exit
  if python3 "$CRASH_CASE_CHECK" --only test-names "$ROOT"; then test_names_exit=0; else test_names_exit=$?; fi
  case "$test_names_exit" in
    0|77) return "$test_names_exit" ;;
  esac
  echo "  ✗ #[test] 函数名：crash-case-check --only test-names 退 $test_names_exit（逐处列在上面）"
  echo "     → 怎么办：照上面那一句出路按测的对象改名，来历进 commit message 或 .claude/kb/；判法写在 scripts/crash/crash-case-check.py 文件头 test-names 一段。"
  return 1
}

# ── 格 format-constants：格式常量在 kb 与实验源码之间同步
#
# 权威在 kb 的 format-const 标记，紧挨着定这个值的那句话（文法与解析在 lib-format-const.py，doc-registries、checker-independence-and-sync 共用）：
#     <!-- format-const: JOURNAL_HEADER_BYTES = 86 stale=hdr=84|room=428 -->
# 判红：标记按文法读不出来（多写了键、值不是整数）；同一个名字登记两处（同一份文件里两次也算）；
# scripts/experiment-apparatus/、research/、crates/ 源码里同名 const 的值与标记不等（只认类型是单个标识符、值是整数字面量的 const，同名的数组扫描表不算）；
# stale= 列的旧值字面串还留在 kb 正文或源码里。旧值只许留在「## 历史版本」之后与 *-history.md 里。
# 扫哪些源码：`git ls-files --cached --others --exclude-standard` 列出的 scripts/experiment-apparatus/、research/、crates/ 下的 .rs（已跟踪的与还没跟踪、没被忽略的；research/ 下只剩 prompts/ 与 results/，扫它是为了冻结证据里的旧值照判、由排除表逐份放行）；
# 被 .gitignore 忽略的编译副本不扫（例如 scripts/experiment-apparatus/target/ 下的 crates 副本：本机有、干净的克隆里没有，扫它等于两台机器判出两个结果）。
# 冻结证据（research/prompts/ 下原样保存的模型产出）改了就不是原样（.claude/rules/evidence-discipline.md「原样保存的证据不许事后改」），
# 登记在 .claude/format-constants-exclude（一行 <路径>  # 理由，目录写成以 / 结尾）：登记路径下的 .rs 既不比 const 的值、也不查旧值字面串。没有这个文件按空表算。
# 排除表判红：路径不存在；路径在、但没排到一份被 git 列出的 .rs；# 后面的理由不足 4 个字。排除只缩不涨：路径改名了、证据删了，那一行就删。
# 成功句报：扫了几份 .rs、排除表排除了几份。不在 git 仓里（git ls-files 起不来）：本次未跑，退 77。
# 为什么要这一格：research-build-and-replay 逐字节比产物，源码与产物一起停在旧值时它照样绿；doc-registries 只看 kb 内部。
# 射程：只管登记了标记的常量，没登记的新格式常量照样能漂，靠人。
# 译文：登记位和旧值字面串只认中文原文（kb 下不带 .en / .ja 后缀的 .md）。译文（<名>.en.md、<名>.ja.md）里同一个标记抄了一遍，算进来就是同一个名字登记三处；
# 译文的历史节标题是译名，切不开，里面留着的旧值也会算成残留。译文跟没跟上中文原文归 doc-text 的 trilingual-files 一类格，不归这一格。
# 样本里 format-constants-trilingual-green 放三语三份，译文抄了标记、历史节留着旧值，日语译文的正文没跟上、还写着旧值，判绿。
# 样本：red 有源码落后于 kb 的常量、正文残留的旧值、多写了键的标记、同一份文件里登记两次的名字，以及值跨行写、带 pub(crate) 的两处落后声明；
# green 另放一张与格式常量同名的扫描表（数组类型），判绿。
# 排除表的样本：red 里放一份落后的冻结证据、不登记，判红；green 里同样一份登记进排除表，判绿，另有一份被 .gitignore 忽略的编译副本带着旧值，不扫；
# format-constants-exclusion-red 的排除表三行各错一样（路径不存在、目录被忽略而排不到文件、没写理由），三条都要点名。
cell_format_constants() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 kb，这一格跳过"; exit 77; }

  python3 - "$FORMAT_CONST_LIBRARY" "$ITEM_REF_LIBRARY" <<'PY'
import glob, importlib.util, os, subprocess, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)
item_ref_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_spec)
item_ref_spec.loader.exec_module(item_ref_status)
format_constants_break = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK", "")
break_value_comparison = format_constants_break == "format-constants"
EXCLUDE_FILE = ".claude/format-constants-exclude"

def body_of(text):
    """正文 = 「## 历史版本」之前那一段。历史里留旧值是文档纪律要求的，不判红。"""
    i = text.find('\n## 历史版本')
    return text if i < 0 else text[:i]

# *-history.md 不是登记位：变更史原样引用标记是历史陈述，不算第二处权威记录。
# 译文（<名>.en.md、<名>.ja.md）也不是登记位：同一个标记在三份里各抄一遍，算成第二、第三处权威记录就是误判重复；
# 译文的「历史版本」节标题是译名（如 `## Historical Versions`），body_of 切不开，里面按历史陈述留着的旧值也会被当成残留。认译文后缀的判法只在 lib-item-ref-status.py 的 language_of。
kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))
kb_files = [f for f in kb_all if not f.endswith('-history.md') and '/decisions-history/' not in f
            and (format_constants_break == "format-constants-translations-read" or not item_ref_status.language_of(f))]   # 弄坏开关：译文也进来

# ---- 1. 收标记：读不出来的、同一个常量登记了不止一处的，判红 ----
marks, registrations, bad = {}, {}, []
unparsable_count = 0
for f in kb_files:
    parsed = format_const.parse_marks(open(f, encoding='utf-8').read())
    for unparsable in parsed.unparsable:
        unparsable_count += 1
        bad.append(f'{f}:{unparsable.line_number}  format-const 标记按文法读不出来：「{unparsable.excerpt}」')
    for mark in parsed.marks:
        registrations.setdefault(mark.name, []).append(f'{f}:{mark.line_number}')
        marks.setdefault(mark.name, (f, mark.value, list(mark.stale_literals)))

duplicate_count = 0
for name, places in registrations.items():
    if len(places) > 1:
        duplicate_count += 1
        bad.append(f'{name}：登记了 {len(places)} 处（{" 与 ".join(places)}）—— 同一个事实只许一处权威记录')

if not marks and not bad:
    print('  ! kb 里一个 format-const 标记都没有，这一格**什么也没验**')
    print('     → 在定住格式常量的那句话旁边加 <!-- format-const: 名字 = 值 stale=旧字面串 -->')
    sys.exit(1)

# ---- 2. 源码里的 const 定义必须等于 kb 的现行值 ----
# 格式常量模块住 crates/singlefs-format，同一套标记绑住它
# 实验源码在 scripts/experiment-apparatus/，冻结证据在 research/prompts/（排除表逐份放行），crates/ 照判
scanned_roots = ["scripts/experiment-apparatus", "research", "crates"]
if format_constants_break == "format-constants-research-unscanned":
    scanned_roots = ["scripts/experiment-apparatus", "crates"]   # 弄坏开关：research/ 整片不扫，冻结证据里的旧值看不见、排除表那一行排不到文件
if format_constants_break == "format-constants-ignored-files-scanned":
    listed_sources = [path for root in scanned_roots for path in glob.glob(f'{root}/**/*.rs', recursive=True)]   # 弄坏开关：按盘上有什么扫，被忽略的编译副本也进来
else:
    listing = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *scanned_roots], capture_output=True)
    if listing.returncode != 0:
        print(f"  ⊘ 本次未跑：git ls-files 退 {listing.returncode}（{listing.stderr.decode('utf-8', 'replace').strip()[:200]}），分不出哪些 .rs 被 git 忽略")
        print("     → 在 git 仓里跑这一格；不是仓的目录（拷出来的树）判不了。")
        sys.exit(77)
    listed_sources = [path for path in listing.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path)]
listed_sources = sorted(set(listed_sources))

# 排除表：一行 <路径>  # 理由；目录写成以 / 结尾，路径等于它、或落在它下面的 .rs 都排除
exclusion_entries = []
if os.path.isfile(EXCLUDE_FILE):
    for number, line in enumerate(open(EXCLUDE_FILE, encoding='utf-8'), 1):
        entry_text, _, reason_text = line.partition('#')
        if entry_text.strip():
            exclusion_entries.append((number, entry_text.strip(), reason_text.strip()))
exclusion_problems, excluded_files = [], set()
for number, entry, reason in exclusion_entries:
    entry_base = entry.rstrip('/')
    matched = [path for path in listed_sources if path == entry_base or path.startswith(entry_base + '/')]
    if not os.path.exists(entry_base):
        if format_constants_break != "format-constants-exclusion-missing-path-unjudged":
            exclusion_problems.append(f'{EXCLUDE_FILE}:{number} 指向不存在的路径 {entry}')
    elif not matched:
        if format_constants_break != "format-constants-exclusion-no-match-unjudged":
            exclusion_problems.append(f'{EXCLUDE_FILE}:{number} {entry} 下一份被 git 列出的 .rs 都没排到（被 .gitignore 忽略、下面没有 .rs，或路径写错了）')
    if len(reason) < 4 and format_constants_break != "format-constants-exclusion-reason-unjudged":
        exclusion_problems.append(f'{EXCLUDE_FILE}:{number} {entry} 没写为什么排除（# 后面至少 4 个字）')
    excluded_files.update(matched)
if format_constants_break == "format-constants-exclusion-ignored":
    excluded_files = set()   # 弄坏开关：排除表照读照判，但登记的路径照样扫
srcs = [path for path in listed_sources if path not in excluded_files]
seen_in_src = set()
for f in srcs:
    source_text = open(f, encoding='utf-8', errors='ignore').read()
    for declaration in format_const.read_rust_consts(source_text, only_scalar_types=True):
        if declaration.name not in marks:
            continue
        name = declaration.name
        seen_in_src.add(name)
        # 值要读到分号为止：只取开头那段数字时，`16384 * 2` 读成 16384 判通过，`16 * 1024` 与 `16_384` 读成 16 判红。
        if declaration.value is None:
            shown = format_const.normalized_value_text(declaration.value_text)
            bad.append(f'{f}:{declaration.line_number}  const {name} 的值写成了「{shown}」，门禁读不出它等于几 → 写成整数字面量（可带 _ 分隔）')
            continue
        kbf, want, _ = marks[name]
        if break_value_comparison:
            continue   # 弄坏开关：不比源码 const 的值与 kb 的现行值
        if declaration.value != want:
            bad.append(f'{f}:{declaration.line_number}  const {name} = {declaration.value}，而 {kbf} 定的现行值是 {want}')

# ---- 3. 旧值的字面串不许留在正文与源码里 ----
# research/results/ 不扫：产物是那一轮的原始输出，不许事后改（.claude/rules/evidence-discipline.md「原样保存的证据不许事后改」）；
# 源码改了而产物没重跑，由 research-build-and-replay 逐字节比对抓。
scan = [(f, format_const.strip_marks(body_of(open(f, encoding='utf-8').read()))) for f in kb_files]
scan += [(f, open(f, encoding='utf-8', errors='ignore').read()) for f in srcs]

for name, (kbf, want, stale) in sorted(marks.items()):
    for s in stale:
        for f, text in scan:
            if s in text:
                ln = text[:text.index(s)].count('\n') + 1
                bad.append(f'{f}:{ln}  还留着 {name} 的旧值字面串「{s}」（现行值 {want}）')

if exclusion_problems:
    for problem in exclusion_problems:
        print(f'  ✗ {problem}')   # gate-lint:detail
    print(f'  ✗ 排除表 {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由')
    print('     → 指向不存在的与排不到文件的那几行删掉（排除只缩不涨）；留着的每一行在 # 后面写明为什么不判（原样保存的证据、改了就不是原样）。')
if bad:
    print('  ✗ 格式常量在 kb 与实验源码之间对不上：')
    for b in bad[:20]:
        print(f'     {b}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 权威是 kb 里的 format-const 标记。改常量要三处一起动：')
    print('       ① kb 标记与正文 ② 实验源码的 const 与钉死它的单测 ③ 重跑实验并更新 research/results/ 的产物')
    print('     → 旧值只许留在「## 历史版本」之后与 *-history.md 里。')
    if unparsable_count:
        print('     → 读不出来的标记照这一条文法改写：<!-- format-const: 名字 = 整数 stale=旧串|旧串 -->，')
        print('       stale= 之外不许有别的键；它读不出来时，这个常量在这一格眼里就没登记过。')
        print('       它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。')
    if duplicate_count:
        print('     → 一个名字只留一处登记（定这个值的那一处）；别处要提它，写成不带 <!-- 的文字，例如「`format-const: 名字`」。')
    sys.exit(1)
if exclusion_problems:
    sys.exit(1)

only_kb = sorted(set(marks) - seen_in_src)
print(f'  ✓ 格式常量同步（{len(marks)} 个已登记，{len(seen_in_src)} 个在源码里被钉住；扫了 {len(srcs)} 份 .rs，排除表排除了 {len(excluded_files)} 份）')
if only_kb:
    print(f'     ! 这些还没有任何实验源码用到，这一格对它们只验了唯一性：{", ".join(only_kb)}')
PY
}

# ── 格 clause-enums：条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
#
# 拿 .claude/gate.d/clause-enum-pairs.tsv 逐对判，任一条不成立判红：
#   ① 枚举里每个变体都在表里有一行（代码加了成员而没登记）；
#   ② 表里每一行的变体在枚举里真的存在（代码删了成员而表没跟）；
#   ③ 表里每一行「条文里对应的名字」是那条分项正文里用「 / 」列举的一项，逐字、不按子串认；
#   ④ 正文那串「 / 」列举的每一项都在表里有一行（条文加了成员、枚举与表都没动）。
#      那串列举取正文里与表中名字重合最多的一串「甲 / 乙 / 丙」（项里不含括号、逗号、句号、分号与 `*`），
#      末项后面紧跟的「六种」这类计数词不算进项名。
# 表不在、或一行登记都没有（只剩注释），退 77。
# 射程：只判名字对得上，不判这个成员的语义是不是条文说的那个（靠 review）。
# 样本：red 的枚举多一个没登记的变体 `Bing`、表里一行只写了列举项「乙种」的一截「乙」、条文多列了一项「丙种」，三处都要点名；
# green 的枚举、表与条文列举逐个对上。
# 分项正文取哪一份：只认中文原文 <号>-<名>.md（判法在 lib-item-ref-status.py 的 numbered_files），译文 .en.md、.ja.md 的标题是译名、找不到「#### 已定项 k」；
# 中文原文不是恰好一份判红。clause-enums-trilingual-green 放三语三份、译文没有中文小节标题，判绿。
cell_clause_enums() {
  local TABLE=".claude/gate.d/clause-enum-pairs.tsv"
  if [[ ! -f "$TABLE" ]]; then
    echo "  ⊘ 本次未跑：$TABLE 不在，今天无对象可判"
    exit 77
  fi

  local report
  if ! report="$(TABLE="$TABLE" python3 - "$ITEM_REF_LIBRARY" 2>&1 <<'PY'
import os, re, glob, sys, importlib.util

item_ref_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
item_ref_status = importlib.util.module_from_spec(item_ref_spec)
item_ref_spec.loader.exec_module(item_ref_status)

break_substring_match = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "clause-enums"
break_any_language_decision = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "clause-enums-decision-any-language"

rows = []
for number, line in enumerate(open(os.environ["TABLE"], encoding="utf-8"), 1):
    stripped = line.rstrip("\n")
    if not stripped.strip() or stripped.lstrip().startswith("#"):
        continue
    parts = stripped.split("\t")
    if len(parts) != 5 or not all(part.strip() for part in parts):
        print("BAD", f"{os.environ['TABLE']}:{number}", "这一行不是五段非空（决策分项、枚举落点、变体名、条文里的名字、为什么）", sep="\t")
        continue
    rows.append((number, *[part.strip() for part in parts]))

def clause_text(item):
    match = re.match(r"^(D\d+)\s+已定项\s+(\d+)$", item)
    if not match:
        return None, f"决策分项写成「{item}」，认不出「D<n> 已定项 <k>」这个形态"
    decision, number = match.group(1), match.group(2)
    # 决策正文有中文原文与 .en.md、.ja.md 两份译文，分项正文只认中文原文（按编号取哪一份的判法只在 lib-item-ref-status.py 的 numbered_files）
    if break_any_language_decision:
        files = sorted(glob.glob(f".claude/kb/decisions/{int(decision[1:]):02d}-*.md"))[:1]   # 弄坏开关：不分语言，取按文件名排序最靠前的一份（三语并存时是 .en.md）
    else:
        files = item_ref_status.numbered_files(".claude/kb/decisions", int(decision[1:]))
    if not files:
        return None, f"找不到 {decision} 的中文原文正文文件（.claude/kb/decisions/ 下不带 .en / .ja 译文后缀的那一份）"
    if len(files) > 1:
        return None, f"{decision} 的中文原文正文文件不止一份：{'、'.join(files)}"
    text = open(files[0], encoding="utf-8").read()
    section = re.search(rf"^#### 已定项 {number}[:：].*?(?=^#### |\Z)", text, re.S | re.M)
    if not section:
        return None, f"{files[0]} 里找不到「#### 已定项 {number}」这一节"
    return section.group(0), None

def enum_variants(where):
    path, _, name = where.rpartition(":")
    if not path or not name:
        return None, f"枚举落点写成「{where}」，认不出「<文件>:<枚举名>」这个形态"
    if not os.path.exists(path):
        return None, f"{path} 不在了"
    text = open(path, encoding="utf-8").read()
    block = re.search(rf"enum\s+{re.escape(name)}\s*\{{(.*?)^\}}", text, re.S | re.M)
    if not block:
        return None, f"{path} 里找不到 `enum {name}`"
    body = re.sub(r"//.*", "", block.group(1))
    body = re.sub(r"#\[[^\]]*\]", "", body)
    return re.findall(r"^\s*([A-Z][A-Za-z0-9]*)\s*(?:\{|\(|,|$)", body, re.M), None

LIST_RUN = re.compile(r"[^/（）()，,。；;\n*]+(?:\s*/\s*[^/（）()，,。；;\n*]+)+")
COUNT_WORD = re.compile(r"[一二三四五六七八九十两0-9]+\s*种$")

def enumerated_items(section, names):
    """分项正文里与 names 重合最多的那一串「甲 / 乙 / 丙」列举：返回项的列表；一串都没有返回 None。"""
    best, best_overlap = None, 0
    for run in LIST_RUN.finditer(section):
        items = [COUNT_WORD.sub("", item.strip()).strip() for item in run.group(0).split("/")]
        overlap = len(set(items) & set(names))
        if overlap > best_overlap:
            best, best_overlap = items, overlap
    return best

def is_enumerated_item(chinese, items):
    if break_substring_match:
        return any(chinese in item for item in items)   # 弄坏开关：按子串认
    return chinese in items

if not rows:
    print("EMPTY", sep="\t")

clauses, enums = {}, {}
checked = 0
for number, item, where, variant, chinese, _why in rows:
    if item not in clauses:
        clauses[item] = clause_text(item)
    section, why = clauses[item]
    if why:
        print("BAD", f"{os.environ['TABLE']}:{number}", why, sep="\t")
        continue
    if where not in enums:
        enums[where] = enum_variants(where)
    variants, why = enums[where]
    if why:
        print("BAD", f"{os.environ['TABLE']}:{number}", why, sep="\t")
        continue
    checked += 1
    if variant not in variants:
        print("BAD", f"{os.environ['TABLE']}:{number}",
              f"表里写着变体 `{variant}`，而 {where} 的枚举里没有它（代码删了成员而表没跟）", sep="\t")
    if chinese not in section:
        print("BAD", f"{os.environ['TABLE']}:{number}",
              f"{item} 的正文里找不到「{chinese}」（条文没跟上代码，或者名字改了而表没跟）", sep="\t")

# ③ 的逐字那一半与 ④：按（分项，枚举）成组，取正文里那串列举，两边集合逐项比。
groups = {}
for number, item, where, variant, chinese, _why in rows:
    if clauses.get(item, (None, "x"))[1] or enums.get(where, (None, "x"))[1]:
        continue
    groups.setdefault((item, where), []).append((number, chinese))
for (item, where), members in sorted(groups.items()):
    names = [chinese for _number, chinese in members]
    items = enumerated_items(clauses[item][0], names)
    if items is None:
        print("BAD", f"{item}", f"正文里找不到一串用「 / 」分隔、含表中名字的列举，{where} 那张表逐项对不了", sep="\t")
        continue
    for number, chinese in members:
        if chinese in clauses[item][0] and not is_enumerated_item(chinese, items):
            print("BAD", f"{os.environ['TABLE']}:{number}",
                  f"表里的「{chinese}」不是 {item} 正文那串列举里的一项（列举是：{' / '.join(items)}；按子串对上的不算）", sep="\t")
    for extra in items:
        if extra not in names:
            print("BAD", f"{item}",
                  f"条文列了「{extra}」，而 {os.environ['TABLE']} 里 {where} 没有它那一行（条文加了成员而枚举与表都没跟）", sep="\t")

# 反过来：枚举里有、表里没有的变体。
for where, (variants, why) in enums.items():
    if why:
        continue
    registered = {variant for _n, _i, w, variant, _c, _y in rows if w == where}
    for variant in variants:
        if variant not in registered:
            print("BAD", where,
                  f"枚举里有变体 `{variant}`，而 {os.environ['TABLE']} 里一行都没有（代码加了成员而没登记）", sep="\t")

print("COUNT", checked, len(enums), len(clauses), sep="\t")
PY
)"; then
    echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一对都没比"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/lib/code-source-discipline-naming-and-constants-cells.sh 里 clause-enums 那一格的 python；"
    echo "      这一步没跑完就是什么都没查，不是通过。"
    exit 1
  fi

  if ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check clause-enums）看 python 的报错；没有 COUNT 就是一对都没比，"
    echo "      而汇总行看着与比过了一模一样。"
    exit 1
  fi

  local -a bad
  mapfile -t bad < <(grep '^BAD' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 条文列的集合与代码里的枚举对不上："
    while IFS=$'\t' read -r _ where why; do
      printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：先判是哪一边错了——代码加了成员就把它写回那条分项的正文、再往 $TABLE 补一行；"
    echo "      条文才是对的就改代码。⚠️ 别只改表让两边看着一致：表是登记位，不是第三种说法。"
    echo "      改分项正文是一次决策变更，照 .claude/rules/format-evolution.md 记进当月变更史。"
    exit 1
  fi

  if grep -q '^EMPTY' <<<"$report"; then
    echo "  ⊘ 本次无对象可判：$TABLE 里一行登记都没有（只剩注释与空行）"
    exit 77
  fi
  local checked enum_count clause_count
  read -r _ checked enum_count clause_count < <(grep '^COUNT' <<<"$report")
  echo "  ✓ 条文列的集合与代码里的枚举逐个对得上（${checked} 对成员、${enum_count} 个枚举、${clause_count} 条分项；两个方向都逐项比过：表里的名字都是正文列举的一项，正文列举的每一项都在表里）"
  exit 0
}

# ── 格 feature-bits：feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
#
# 依据：D15（格式冻结政策） 已定项 10（记账落在 .claude/kb/feature-bits.md）与已定项 4（位号的唯一登记位那张表）。
# 三条判据，任一条不成立判红：
#   ① 记账表的（类别，位号）集合与已定项 4 登记表逐行对得上，且记账表的「语义一句话」逐字包含登记表同一位的「含义」；
#   ② 每张位图内位号严格递增、不跳号（已分配的位必须是 0..n-1）；
#   ③ `crates/*/src/**/*.rs` 里名字带 INCOMPAT / COMPAT_RO / COMPAT 段的常量，值解得出位号的，那个位号必须在记账表里；
# 格式锁定之前位号可回收、可重排，不判「位用过不许回收」；同一个（类别，位号）出现两行时，位号不再严格递增，由 ② 报。
# 射程：③ 只认写在一行里、值是位掩码字面量的常量声明。值由别的常量合成的（`SUPPORTED_INCOMPAT_BITS`）、声明跨行的，
# 解不出位号，逐个列进成功那句的「没判位号的」名单。不经具名常量的裸字面量（直接写 `0x01`）不在射程里，靠 review。
# 样本：red 的记账表跳号、同一位登记两行不同语义、多出一位登记表里没有，代码样本又引用了表里没有的一位，①②③ 都必须报出来（同一位两行由 ② 报次序不对）；
# green 只有位 0 一行、语义与登记表对得上，判绿。
# D15 取哪一份：只认中文原文 15-<名>.md（判法在 lib-item-ref-status.py 的 numbered_files，孤立译文一份都不算），译文 .en.md、.ja.md 里没有「#### 已定项 4」；
# 中文原文不是恰好一份判红（零份与两份各一句出路）。feature-bits-trilingual-red 放三语三份、记账表比中文登记表多一位，译文登记表把那一位分出去了，
# 必须按中文原文判出「登记表把它列在未分配区间 1..255 里」；feature-bits-trilingual-green 三语三份、对得上，判绿；feature-bits-two-originals-red 编号 15 下有两份中文文件，判红。
cell_feature_bits() {
  [[ -f .claude/kb/feature-bits.md ]] || { echo "  ! 没有 .claude/kb/feature-bits.md，这一格无对象可判"; exit 77; }

  python3 - "$ITEM_REF_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys

item_ref_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
item_ref_status = importlib.util.module_from_spec(item_ref_spec)
item_ref_spec.loader.exec_module(item_ref_status)

break_any_language_decision = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "feature-bits-decision-any-language"
FEATURE_BITS_PATH = ".claude/kb/feature-bits.md"
BITMAPS = ("incompat", "compat_ro", "compat")
BITS_PER_BITMAP = 256
EXPECTED_HEADER = ["位号", "类别", "名称", "引入版本", "引入 commit", "状态", "语义一句话"]

problems = []


def reject(summary, details, howto):
    problems.append((summary, details, howto))


def normalize(text):
    return re.sub(r"\s+", "", text.replace("*", "").replace("`", ""))


def is_separator(cells):
    return all(re.fullmatch(r":?-{2,}:?", cell) for cell in cells)


def read_table(lines, start_index, column_count):
    """从 lines[start_index] 起读一张 markdown 表，返回去掉表头与分隔行之后的每行单元格。"""
    rows = []
    for line in lines[start_index:]:
        stripped = line.strip()
        if not stripped.startswith("|"):
            break
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if len(cells) != column_count:
            break
        if is_separator(cells):
            continue
        rows.append(cells)
    return rows


# ── D15 已定项 4 登记表：位号的唯一登记位 ────────────────────────
# 决策正文有三份：中文原文 15-<名>.md 与译文 15-<名>.en.md、15-<名>.ja.md；登记位只认中文原文（译文的标题与表头是译名，找不到「#### 已定项 4」）。
# 按编号取哪一份文件的判法只在 lib-item-ref-status.py 的 numbered_files（不给引用方就取中文原文，孤立译文一份都不交回）。
if break_any_language_decision:
    decision_files = sorted(glob.glob(".claude/kb/decisions/15-*.md"))[:1]   # 弄坏开关：不分语言，取按文件名排序最靠前的一份（三语并存时是 .en.md）
else:
    decision_files = item_ref_status.numbered_files(".claude/kb/decisions", 15)
d15_allocated = {}      # (类别, 位号) -> 含义
d15_unallocated = {}    # 类别 -> [(起, 止)]
d15_path = ""
if not decision_files:
    reject(
        "找不到 D15（格式冻结政策） 的中文原文决策文件（.claude/kb/decisions/15-*.md，不带 .en / .ja 译文后缀的那一份）——位号的唯一登记位不在，没法比对",
        [],
        "确认决策正文的中文原文还在 .claude/kb/decisions/ 下，只有译文（15-*.en.md、15-*.ja.md）不算；改了文件名就把 15- 这个编号前缀留着，这一格按它找。",
    )
elif len(decision_files) > 1:
    reject(
        f"D15（格式冻结政策） 的中文原文决策文件不止一份：{'、'.join(decision_files)}——位号的唯一登记位不知道是哪一份，没法比对",
        [],
        "编号 15 只留一份中文原文（<号>-<名>.md）；译文放在同名的 .en.md、.ja.md 里，不另起编号相同的第二份中文文件。",
    )
else:
    d15_path = decision_files[0]
    d15_lines = open(d15_path, encoding="utf-8").read().split("\n")
    section_start = section_end = -1
    for index, line in enumerate(d15_lines):
        if line.startswith("#### 已定项 4"):
            section_start = index
        elif section_start >= 0 and line.startswith("#### "):
            section_end = index
            break
    if section_end < 0:
        section_end = len(d15_lines)
    header_index = -1
    for index in range(section_start, section_end):
        if section_start < 0:
            break
        cells = [cell.strip() for cell in d15_lines[index].strip().strip("|").split("|")]
        if d15_lines[index].strip().startswith("|") and len(cells) == 4 and "bitmap" in cells[0] and "含义" in cells[2]:
            header_index = index
            break
    if header_index < 0:
        reject(
            f"{d15_path} 的「已定项 4」里找不到位分配登记表（表头要是 bitmap / 位 / 含义 / 出处 四列）",
            [f"已定项 4 这一节：第 {section_start + 1} 行起" if section_start >= 0 else "连「#### 已定项 4」这个标题都没找到"],
            "位号的唯一登记位就是那张表，表没了这一道就没有比对的对象。把表写回已定项 4，四列照旧：bitmap、位、含义、出处。",
        )
    else:
        malformed = []
        for cells in read_table(d15_lines, header_index + 1, 4):
            bitmap, bit_field, meaning, _source = cells
            if bitmap == "bitmap":
                continue
            if bitmap not in BITMAPS:
                malformed.append(f"类别「{bitmap}」不是 incompat / compat_ro / compat 之一：{' | '.join(cells)}")
                continue
            if re.fullmatch(r"\d+", bit_field):
                d15_allocated[(bitmap, int(bit_field))] = meaning
            elif re.fullmatch(r"\d+\.\.\d+", bit_field):
                if "未分配" not in meaning:
                    malformed.append(f"{bitmap} {bit_field} 是一段区间，含义里却没写「未分配」：{meaning}")
                    continue
                low, high = bit_field.split("..")
                d15_unallocated.setdefault(bitmap, []).append((int(low), int(high)))
            else:
                malformed.append(f"「位」这一格既不是位号也不是 a..b 区间：{' | '.join(cells)}")
        if malformed:
            reject(
                f"{d15_path} 的位分配登记表有行读不出来，位号集合拼不完整",
                malformed,
                "每行的「位」写成一个位号（已分配）或 a..b 区间（未分配，含义里要写「未分配」），类别只许 incompat / compat_ro / compat。",
            )
        elif not d15_allocated and not d15_unallocated:
            reject(
                f"{d15_path} 的位分配登记表一行都没读出来",
                [],
                "扫到 0 行不是通过。检查表是不是紧挨着表头、有没有被围栏包住；表的四列是 bitmap、位、含义、出处。",
            )
        else:
            holes = []
            for bitmap in BITMAPS:
                covered = sorted(bit for (bitmap_name, bit) in d15_allocated if bitmap_name == bitmap)
                for low, high in d15_unallocated.get(bitmap, []):
                    covered.extend(range(low, high + 1))
                covered_sorted = sorted(covered)
                if covered_sorted != list(range(BITS_PER_BITMAP)):
                    missing = sorted(set(range(BITS_PER_BITMAP)) - set(covered_sorted))
                    repeated = sorted({bit for bit in covered_sorted if covered_sorted.count(bit) > 1})
                    holes.append(f"{bitmap}：漏了 {len(missing)} 位（最小 {missing[0] if missing else '—'}）、重了 {len(repeated)} 位")
            if holes:
                reject(
                    f"{d15_path} 的位分配登记表自己没罩满 0..{BITS_PER_BITMAP - 1}，有洞或有重叠",
                    holes,
                    "每张位图的已分配位加上未分配区间要正好覆盖 0..255 一次。罩不满时「这一位归谁」就没有答案，比对也无从谈起。",
                )

# ── 记账表 ──────────────────────────────────────────────
feature_lines = open(FEATURE_BITS_PATH, encoding="utf-8").read().split("\n")
marker_index = -1
for index, line in enumerate(feature_lines):
    if line.strip() == "<!-- feature-bits:table -->":
        marker_index = index
        break
header_index = -1
if marker_index >= 0:
    for index in range(marker_index + 1, len(feature_lines)):
        stripped = feature_lines[index].strip()
        if stripped.startswith("|"):
            header_index = index
            break
        if stripped.startswith("#"):
            break

rows = []
if marker_index < 0 or header_index < 0:
    reject(
        f"{FEATURE_BITS_PATH} 里找不到记账表（要有一行 <!-- feature-bits:table -->，它后面第一张 markdown 表就是记账表）",
        [],
        "在记账表上面单独写一行 <!-- feature-bits:table -->，表头七列照 D15（格式冻结政策） 已定项 10：" + " | ".join(EXPECTED_HEADER),
    )
else:
    header_cells = [cell.strip() for cell in feature_lines[header_index].strip().strip("|").split("|")]
    if header_cells != EXPECTED_HEADER:
        reject(
            f"{FEATURE_BITS_PATH} 记账表的表头与 D15（格式冻结政策） 已定项 10 定的列序对不上",
            [f"读到：{' | '.join(header_cells)}", f"要的是：{' | '.join(EXPECTED_HEADER)}"],
            "七列照 D15（格式冻结政策） 已定项 10 写，顺序不许换——别处按列位取值，换了顺序取出来的就是别的东西。",
        )
    else:
        malformed = []
        for cells in read_table(feature_lines, header_index + 1, len(EXPECTED_HEADER)):
            bit_field, bitmap = cells[0], cells[1]
            if not re.fullmatch(r"\d+", bit_field):
                malformed.append(f"「位号」不是一个整数：{' | '.join(cells)}")
                continue
            if bitmap not in BITMAPS:
                malformed.append(f"「类别」不是 incompat / compat_ro / compat 之一：{' | '.join(cells)}")
                continue
            empty = [EXPECTED_HEADER[i] for i, cell in enumerate(cells) if not cell]
            if empty:
                malformed.append(f"{bitmap} 位 {bit_field}：这几列是空的：{'、'.join(empty)}")
                continue
            rows.append((int(bit_field), bitmap, cells[2], cells[3], cells[4], cells[5], cells[6]))
        if malformed:
            reject(
                f"{FEATURE_BITS_PATH} 记账表有行读不出来",
                malformed,
                "每行七格都要填：位号写整数，类别写 incompat / compat_ro / compat，代码侧没有常量的「名称」写破折号。",
            )

# ── ② 每张位图内严格递增、不跳号 ─────────────────────────────
order_problems = []
for bitmap in BITMAPS:
    in_file_order = [bit for bit, row_bitmap, *_rest in rows if row_bitmap == bitmap]
    if not in_file_order:
        continue
    if any(later <= earlier for earlier, later in zip(in_file_order, in_file_order[1:])):
        order_problems.append(f"{bitmap}：表里的位号不是严格递增的，读到的次序是 {in_file_order}")
    distinct = sorted(set(in_file_order))
    if distinct != list(range(len(distinct))):
        gaps = sorted(set(range(distinct[-1] + 1)) - set(distinct))
        order_problems.append(f"{bitmap}：已分配的位跳号了，缺 {gaps}（已分配的位必须是 0..{len(distinct) - 1}）")
if order_problems:
    reject(
        f"{FEATURE_BITS_PATH} 记账表的位号次序不对",
        order_problems,
        "每张位图的位号从 0 开始严格递增、一位不跳（D15（格式冻结政策） 已定项 10「位号严格递增、不许跳号占位」）。"
        "跳号占位等于替一个还没人写的特性把位占住，而占位的那一位在盘上与「已启用」长得一模一样。",
    )

# ── ① 与 D15 登记表逐行对得上 ───────────────────────────────
if rows and (d15_allocated or d15_unallocated):
    registry_keys = set(d15_allocated)
    ledger_keys = {(bitmap, bit) for bit, bitmap, *_rest in rows}
    mismatches = []
    for bitmap, bit in sorted(ledger_keys - registry_keys):
        where = ""
        for low, high in d15_unallocated.get(bitmap, []):
            if low <= bit <= high:
                where = f"，登记表把它列在未分配区间 {low}..{high} 里"
        mismatches.append(f"记账表登记了 {bitmap} 位 {bit}，{d15_path} 已定项 4 的登记表里没有这一位{where}")
    for bitmap, bit in sorted(registry_keys - ledger_keys):
        mismatches.append(f"{d15_path} 已定项 4 把 {bitmap} 位 {bit} 分出去了（{d15_allocated[(bitmap, bit)]}），记账表里没有它")
    for bit, bitmap, name, version, commit, status, semantics in rows:
        meaning = d15_allocated.get((bitmap, bit))
        if meaning is None:
            continue
        if normalize(meaning) not in normalize(semantics):
            mismatches.append(
                f"{bitmap} 位 {bit} 的语义与登记表对不上：登记表的「含义」是「{meaning}」，记账表的「语义一句话」是「{semantics}」"
            )
    if mismatches:
        reject(
            f"记账表 {FEATURE_BITS_PATH} 与位号的唯一登记位（{d15_path} 已定项 4）对不上",
            mismatches,
            "位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表，记账表只记账：两处不一致时改记账表，不改登记表。"
            "「语义一句话」要逐字包含登记表同一位的「含义」——整行抄过来再往后接细节，别做一个更短的版本。",
        )

# ── ③ 代码里引用的位必须在记账表里 ────────────────────────────
CONST_NAME = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)")
CONST_FULL = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)\s*:\s*[^=;]+=\s*([^;]+);")
NUMERIC = re.compile(r"^(?:0x([0-9a-fA-F_]+)|([0-9_]+))(?:[ui](?:8|16|32|64|128|size))?$")
SHIFT = re.compile(r"^1(?:[ui](?:8|16|32|64|128|size))?\s*<<\s*([0-9_]+)(?:[ui](?:8|16|32|64|128|size))?$")


def bitmap_of(constant_name):
    segments = constant_name.split("_")
    if "INCOMPAT" in segments:
        return "incompat"
    for index in range(len(segments) - 1):
        if segments[index] == "COMPAT" and segments[index + 1] == "RO":
            return "compat_ro"
    if "COMPAT" in segments:
        return "compat"
    return None


def decode_bit(expression):
    stripped = re.sub(r"//.*$", "", expression).strip()
    match = NUMERIC.fullmatch(stripped)
    if match:
        value = int(match.group(1).replace("_", ""), 16) if match.group(1) else int(match.group(2).replace("_", ""))
        if value != 0 and value & (value - 1) == 0:
            return value.bit_length() - 1, ""
        return None, f"值 `{stripped}` 不是单个位掩码"
    match = SHIFT.fullmatch(stripped)
    if match:
        return int(match.group(1).replace("_", "")), ""
    return None, f"值 `{stripped}` 不是位掩码字面量"


rust_files = sorted(glob.glob("crates/*/src/**/*.rs", recursive=True))
decoded = []
undecided = []
for path in rust_files:
    for line_number, line in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        expressions = {name: expression for name, expression in CONST_FULL.findall(line)}
        for name in CONST_NAME.findall(line):
            bitmap = bitmap_of(name)
            if bitmap is None:
                continue
            if name not in expressions:
                undecided.append((name, path, line_number, "常量声明没写在一行里，解不出值"))
                continue
            bit, reason = decode_bit(expressions[name])
            if bit is None:
                undecided.append((name, path, line_number, reason))
            else:
                decoded.append((bitmap, bit, name, path, line_number))

ledger_keys = {(bitmap, bit) for bit, bitmap, *_rest in rows}
unregistered = [
    f"{path}:{line_number} 的 {name} 指 {bitmap} 位 {bit}，{FEATURE_BITS_PATH} 的记账表里没有这一位"
    for bitmap, bit, name, path, line_number in decoded
    if (bitmap, bit) not in ledger_keys
]
if unregistered:
    reject(
        f"代码引用了记账表里不存在的 feature bit 位",
        unregistered,
        f"先在 {FEATURE_BITS_PATH} 的记账表里给这一位写一行（七列齐全，引入 commit 用 git log 查，别填猜的），"
        "并在 D15（格式冻结政策） 已定项 4 的登记表里把它从未分配区间里切出来——两处都到位这一道才绿。",
    )

for summary, details, howto in problems:
    print(f"  ✗ {summary}")                      # gate-lint:summary
    for detail in details:
        print(f"     {detail}")                  # gate-lint:detail
    print(f"     → 怎么办：{howto}")
if problems:
    sys.exit(1)

skipped_text = "；".join(
    f"{name}（{path}:{line_number}，{reason}）" for name, path, line_number, reason in undecided
) or "无"
print(
    f"  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 {len(rows)} 位、"
    f"登记表分出去 {len(d15_allocated)} 位；"
    f"扫了 {len(rust_files)} 个 .rs，认出 {len(decoded) + len(undecided)} 处 feature bit 常量、解出位号 {len(decoded)} 处；"
    f"没判位号的 {len(undecided)} 处：{skipped_text}）"
)
PY
}
