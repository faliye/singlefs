#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：repository-copy-closure、selftest-cells-alone、selftest-parallelism、long-run-split-exempt。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# ── 格 repository-copy-closure：scripts/、.claude/gate.d/、.claude/hooks/ 里建仓副本的写法都经 scripts/hosts/lib_repository_copy.py 算闭包
#   （records/2026-10-31-task-server巡检.md 批 73：手抄的拷贝清单漏了包与文件，任务在副本与临时树里缺 prune-server/、.claude/scripts/project_preflight.py、
#   规范副本的 preflight.py；拷什么只该由那份共用库的闭包现算，调用方要多拷的经 declared 逐条写为什么）
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/ 里的 *.sh、*.py；跳过 target/、__pycache__/、.git/、.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/
#   （样本是造出来的输入），跳过 scripts/hosts/lib_repository_copy.py 自己（拷法只在它里面定义）。自证函数体里的照判：自证搭小树时手抄的那几份也是清单。
# 逐份文件的扫描按每台默认重活线程数并行（map_in_processes，fork 的子进程，结果按文件次序合并）。
# 只看代码：python 经共用库 SOURCE_CODE_LINES_LIBRARY 的 python_code_lines 去掉注释与带空白的字符串字面量（不带空白的字面量照代码判：路径就在这里头）；
#   shell 经 shell_code_lines 去掉注释与出路文字，再把引号里带空白的那一段（出路文字、样本命令）去掉；shell 里嵌的 python 只按 shell 的写法认。
# 算建仓副本的写法（每行一处，一行几条命令的说明以「；」连）：
#   shell：命令词是 cp 或 rsync（前面是行首、空白、; & | ( { ` 或 $(），到 ; & | ( ) < > 为止是它的参数；去掉选项（- 起头的词，与 rsync 的 --exclude、
#     --include、--filter 这类、cp 的 -t、-S 这类带值选项后面那个词）与最后一个词（目的地；cp 给了 -t / --target-directory 的不去），剩下的拷源里有一个点名了仓根条目；
#     或 git … archive 之后、去掉选项与第一个词（提交名）剩下的路径里有一个点名了仓根条目。
#   python：shutil.copytree / copy / copy2 / copyfile 的第一个参数里有点名仓根条目的字面量，或 REPOSITORY_ROOT、repository_root 这两个名字；
#     [ 起头的 argv 列表第一项是 "cp" 或 "rsync"，去掉选项与最后一项之后剩下的字面量点名了仓根条目；或第一项是 "git"，"archive" 之后去掉选项与第一项剩下的字面量点名了仓根条目。
#   点名仓根条目：去掉引号、去掉开头一个变量（$名、${名…}、$(…)）与跟着的 /、去掉开头的 ./，第一段是共用清单 COMMON_ENTRIES（从这一道所在仓里的
#     scripts/hosts/lib_repository_copy.py 现读，读不出判红）里的一样，或 prune-server、task-server、scripts、.claude、research、litmus 之一；或整个是 . 或 ./（整个工作区）；
#     或整个是 $repository_root、$REPOSITORY_ROOT、$SCRIPT_REPOSITORY、$STAGE_REPOSITORY、$main_root 这几个仓根变量（后面只接 /、/.）。
#   手抄的拷贝清单（两种语言一样）：一行代码里赋给名字的元组、列表或 shell 数组字面量（<名> = ( 或 [，<名>=(），里面单独写着共用清单里除工具链文件之外的那几样
#     与 prune-server 中的三样以上（前后不接字母、数字、. / - _：crates/demo 这种子路径不算）。拷法经变量、清单写成常量的（COPIED = ("crates", ".cargo", …) 再逐个 copytree）从这一条认。
# 改法：拷仓副本、编译树、临时树都经 scripts/hosts/lib_repository_copy.py：python 调 copy_closure(<仓根>, [(条目, 为什么)…]) 再 copy_entries；shell 起
#   python3 scripts/hosts/minimal-copy.py <仓根> <副本> [--with <条目>=<为什么>]…（只拷一个工作区的走 lib_repository_copy.py link-into）；副本里要起的脚本与要读的数据经 --with 声明，
#   闭包顺着它们 source、起、import 的一起带上。闭包认不出的写法改调用方的写法（那份库文件头「管不到的」），不另抄清单。
# 排除表 .claude/gate.d/repository-copy-closure-exempt.tsv：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中，都判红；只缩不涨：与改动范围的基准那一版比（exempt_table_baseline），基准里没有的「路径:锚串」判红；
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红。
#   一份文件拆成几份的：此刻一行的路径照拆分族（scripts/gate/lib_lag_split_family.py，认法写在它文件头）归到基准里的入口，那个入口下有同一个锚串的不算新加
#   （EXEMPT_ROW_GROWTH_LIBRARY，在 lib/code-tooling-source-scan-cells.sh 里）；族外的路径、锚串对不上的照旧判红。
# 判不了的：拷源整个是变量（"$root/$item"、copytree(source, …)）、调用拆成几行写的、带空白的命令串里的（bash -c "rsync …"）、tar 打包再解的、
#   git worktree add 与 git clone 检出的整棵提交（检出的是一棵提交，不是手抄的清单），靠 review。
# 样本：repository-copy-closure-red 的 scripts/hosts/hand_copy.sh（写死的四样、仓根变量下的单份文件、整个工作区 ./、整个 $REPOSITORY_ROOT 各一处）、
#   scripts/gate/hand_copy.py（copytree、copy2、rsync 的 argv、git archive 的 argv、手抄清单常量各一处）、.claude/hooks/hook_copy.sh、.claude/gate.d/stage_copy.sh 各一处，
#   共 11 处逐处点名；
#   scripts/hosts/exempt_copy.sh 那一处由排除表放行；排除表另一行指向不存在的文件、一行锚串不命中、setup.sh 建 git 仓提交一版不带 grown-row 那一行的表再加回来（表涨了），后三样都点名。
#   repository-copy-closure-green：经 minimal-copy.py 与 copy_closure / copy_entries 拷的、拷源整个是变量的、仓根条目只出现在目的地里的、注释、echo 与文档串里的、
#   只点名两样的字面量（判源树是不是仓根那种）、样本自己的 scripts/hosts/lib_repository_copy.py、.claude/gate.d/fixtures/ 下的，都不点名；
#   排除表一行放行 scripts/hosts/exempt_copy.sh 那一处，判绿并报扫了 3 份、表 1 行每行都命中。
#   repository-copy-closure-exempt-family-green：setup.sh 提交 .claude/gate.d/stage-copy.sh（cp -R "$repository_root/crates"）与登记它的排除表，工作区里那一行挪进
#   .claude/gate.d/lib/stage-copy-cells.sh（shell 拆法的部分文件）、表里那一行改成新路径、锚串不变：此刻一行归到基准里的 stage-copy.sh、锚串相同，判绿并写「那一版比没涨」；
#   repository-copy-closure-exempt-family-red：同一个拆分，挪进的文件叫 .claude/gate.d/lib/other-cells.sh（不是 stage-copy.sh 的部分文件，族外），表里那一行是基准里没有的，判红点名「排除表涨了」。
# 弄坏开关（CODE_TOOLING_BREAK）：repository-copy-sources-unscanned（拷源与手抄清单一概不认）：repository-copy-closure-red 里逐处的 want 与「11 处」找不到；
#   repository-copy-closure-green 那一行排除表一处被判行都不命中、判红；repository-copy-hand-lists-unscanned（不认手抄清单常量）：repository-copy-closure-red 里
#   hand_copy.py:8 那条 want 与「11 处」找不到；repository-copy-exempt-growth-unchecked（排除表不比基准）：repository-copy-closure-red 里「排除表涨了」那条 want 找不到；
#   repository-copy-exempt-family-unrecognized（排除表比基准时不认拆分族：拆出来的新路径都当族外）：repository-copy-closure-exempt-family-green 期望退出 0、实测 1（「排除表涨了」）。
cell_repository_copy_closure() {
local jobs
if ! jobs="$(heavy_thread_default)"; then
  echo "  ✗ 取不了每台默认重活线程数（原因在上一行），逐份扫描的并行度定不了，一份都没扫"
  echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
  exit 1
fi
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/gate.d/repository-copy-closure-exempt.tsv "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" \
  "$STAGE_REPOSITORY/scripts/hosts/lib_repository_copy.py" "$PARALLEL_MAP_LIBRARY" "$jobs" \
  "$EXEMPT_ROW_GROWTH_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
parallel = types.ModuleType("lib_parallel_map")
exec(compile(sys.argv[8], "parallel-map-library", "exec"), parallel.__dict__)
exempt_growth = types.ModuleType("lib_exempt_row_growth")
exec(compile(sys.argv[10], "exempt-row-growth-library", "exec"), exempt_growth.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken, base_state, base_name, base_table, repository_copy_library = sys.argv[3:8]
jobs = int(sys.argv[9])
split_family_library_file = sys.argv[11]

EXEMPT_TABLE = ".claude/gate.d/repository-copy-closure-exempt.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
DEFINITION_FILE = "scripts/hosts/lib_repository_copy.py"
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python"}
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
WITHOUT_WHITESPACE = re.compile(r"\S*")      # python 的字面量：内容里没有空白的留着当代码判（路径、argv 的一项），带空白的是句子，去掉
# 共用清单只在 scripts/hosts/lib_repository_copy.py 的 COMMON_ENTRIES 里（这一道所在仓里那一份），读不出就判红；仓根别的条目在这里补
try:
    import importlib.util
    specification = importlib.util.spec_from_file_location("lib_repository_copy_for_repository_copy_closure", repository_copy_library)
    repository_copy = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(repository_copy)
    common_entries = tuple(repository_copy.COMMON_ENTRIES)
except (OSError, ImportError, AttributeError, SyntaxError) as error:
    print(f"  ✗ 读不出共用清单（{repository_copy_library} 的 COMMON_ENTRIES）：{error}")
    print("     → 怎么办：共用清单只在 scripts/hosts/lib_repository_copy.py 里定义；那份库读不了就先修它（python3 -m py_compile 看语法），再跑这一格。")
    sys.exit(1)
ROOT_ENTRY_NAMES = common_entries + ("prune-server", "task-server", "scripts", ".claude", "research", "litmus")
# 手抄清单认的那几样：共用清单里除工具链文件之外的，加 path 依赖进来的 prune-server；一个字面量里点名了 HAND_LIST_MINIMUM 样以上的算手抄的拷贝清单
HAND_LIST_ENTRIES = tuple(entry for entry in common_entries if not entry.startswith("rust-toolchain")) + ("prune-server",)
HAND_LIST_MINIMUM = 3
LITERAL_ASSIGNMENT = re.compile(r"(?P<name>[A-Za-z_]\w*)\s*=(?!=)\s*(?P<opening>[\(\[])")
BASE_ENTRY_TOKEN = re.compile(r"(?<![\w.$/-])(?P<entry>" + "|".join(re.escape(entry) for entry in HAND_LIST_ENTRIES) + r")(?![\w./-])")
ROOT_VARIABLES = ("repository_root", "REPOSITORY_ROOT", "SCRIPT_REPOSITORY", "STAGE_REPOSITORY", "main_root")
RSYNC_VALUE_OPTIONS = {"--exclude", "--include", "--filter", "-f", "--exclude-from", "--include-from", "--files-from", "-e", "--rsh", "--chmod", "--chown",
                       "--suffix", "--backup-dir", "--compare-dest", "--link-dest", "--copy-dest", "--partial-dir", "--temp-dir", "-T", "--log-file",
                       "--out-format", "--info", "--debug", "-M", "--remote-option"}
CP_VALUE_OPTIONS = {"-t", "--target-directory", "-S", "--suffix"}
ARCHIVE_VALUE_OPTIONS = {"-o", "--output", "--remote", "--exec", "--format", "--prefix"}
SHELL_COMMAND = re.compile(r"(?:^|(?<=[\s;&|({`])|(?<=\$\())(?P<command>rsync|cp)(?=\s)(?P<arguments>[^;&|()<>]*)")
SHELL_ARCHIVE = re.compile(r"(?:^|(?<=[\s;&|({`])|(?<=\$\())git(?=\s)[^;&|()<>]*?\sarchive(?=\s)(?P<arguments>[^;&|()<>]*)")
SHELL_WORD = re.compile(r'(?:"[^"]*"|\'[^\']*\'|[^\s"\'])+')
LEADING_VARIABLE = re.compile(r"^\$(?:\{(?P<braced>[A-Za-z_][A-Za-z0-9_]*)[^}]*\}|(?P<plain>[A-Za-z_][A-Za-z0-9_]*)|\([^)]*\))")
PYTHON_COPY_CALL = re.compile(r"\bshutil\.(?P<function>copytree|copy2|copyfile|copy)\(")
PYTHON_LIST = re.compile(r'\[\s*["\'](?P<command>rsync|cp|git)["\']')
PYTHON_LITERAL = re.compile(r'^(["\'])(?P<content>[^"\'\s]*)\1$')
PYTHON_ANY_LITERAL = re.compile(r'(["\'])(?P<content>[^"\'\s]*)\1')
PYTHON_ROOT_NAME = re.compile(r"\b(?:" + "|".join(ROOT_VARIABLES) + r")\b")
sources_scanned = broken != "repository-copy-sources-unscanned"


def without_prose_strings(line):
    # shell 的一行：引号里带空白的那一段是句子（出路文字、样本命令），换成空格；不带空白的照留（拷源路径就在这里头）
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def code_lines_of(language, text):
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE)
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


def root_entry_of(word):
    # 一个拷源词点名的仓根条目（这一格注释「点名仓根条目」）：交给人看的标记，没点名交 None
    bare = word.replace('"', "").replace("'", "")
    if bare in (".", "./"):
        return "./（整个工作区）"
    found = LEADING_VARIABLE.match(bare)
    if found:
        variable = found.group("braced") or found.group("plain")
        bare = bare[found.end():]
        if variable in ROOT_VARIABLES and bare in ("", "/", "/.", "/./"):
            return f"${variable}（整个仓根）"
        bare = bare.lstrip("/")
    while bare.startswith("./"):
        bare = bare[2:]
    root_segment = bare.split("/", 1)[0]
    return root_segment if root_segment in ROOT_ENTRY_NAMES else None


def operands_of(words, value_options, target_option):
    # 去掉选项与带值选项后面那个词之后的操作数（None 是 python argv 里不是字面量的一项，照操作数算）；交 (操作数, 给没给 -t)
    operands, skip, target_given, options_done = [], False, False, False
    for word in words:
        if skip:
            skip = False
            continue
        if word is not None and not options_done and word == "--":
            options_done = True
            continue
        if word is not None and not options_done and word.startswith("-") and len(word) > 1:
            if target_option and (word in ("-t", "--target-directory") or word.startswith("--target-directory=")):
                target_given = True
            if word in value_options:
                skip = True
            continue
        operands.append(word)
    return operands, target_given


def copy_sources(command, words):
    operands, target_given = operands_of(words, CP_VALUE_OPTIONS if command == "cp" else RSYNC_VALUE_OPTIONS, command == "cp")
    return operands if target_given or len(operands) < 2 else operands[:-1]


def named_entries(words):
    named = []
    for word in words:
        entry = root_entry_of(word) if word is not None else None
        if entry and entry not in named:
            named.append(entry)
    return named


def shell_copies(code):
    found = []
    for command in SHELL_COMMAND.finditer(code):
        named = named_entries(copy_sources(command.group("command"), SHELL_WORD.findall(command.group("arguments"))))
        if named:
            found.append(f"{command.group('command')} 的拷源点名 {'、'.join(named)}")
    for archive in SHELL_ARCHIVE.finditer(code):
        operands, _ = operands_of(SHELL_WORD.findall(archive.group("arguments")), ARCHIVE_VALUE_OPTIONS, False)
        named = named_entries(operands[1:])
        if named:
            found.append(f"git archive 的路径点名 {'、'.join(named)}")
    return found


def bracket_body(code, opening):
    # code[opening] 是 ( 或 [：交配对括号里的正文（配不上的到行尾）
    depth, quote = 0, None
    for index in range(opening, len(code)):
        character = code[index]
        if quote:
            if character == quote:
                quote = None
        elif character in "'\"":
            quote = character
        elif character in "([{":
            depth += 1
        elif character in ")]}":
            depth -= 1
            if depth == 0:
                return code[opening + 1:index]
    return code[opening + 1:]


def top_level_items(text):
    # 按不在括号、引号里的逗号切开，去掉两头空白
    items, depth, quote, start = [], 0, None, 0
    for index, character in enumerate(text):
        if quote:
            if character == quote:
                quote = None
        elif character in "'\"":
            quote = character
        elif character in "([{":
            depth += 1
        elif character in ")]}":
            depth -= 1
        elif character == "," and depth == 0:
            items.append(text[start:index].strip())
            start = index + 1
    tail = text[start:].strip()
    if tail:
        items.append(tail)
    return items


def python_copies(code):
    found = []
    for call in PYTHON_COPY_CALL.finditer(code):
        items = top_level_items(bracket_body(code, call.end() - 1))
        source_argument = items[0] if items else ""
        named = named_entries([literal.group("content") for literal in PYTHON_ANY_LITERAL.finditer(source_argument)])
        named += [f"{name}（仓根）" for name in dict.fromkeys(PYTHON_ROOT_NAME.findall(source_argument))]
        if named:
            found.append(f"shutil.{call.group('function')} 的拷源点名 {'、'.join(named)}")
    for listing in PYTHON_LIST.finditer(code):
        values = []
        for item in top_level_items(bracket_body(code, listing.start())):
            literal = PYTHON_LITERAL.match(item)
            values.append(literal.group("content") if literal else None)
        command = values[0] if values else None
        if command in ("cp", "rsync"):
            named = named_entries(copy_sources(command, values[1:]))
            if named:
                found.append(f"{command} 的拷源点名 {'、'.join(named)}")
        elif command == "git" and "archive" in values:
            operands, _ = operands_of(values[values.index("archive") + 1:], ARCHIVE_VALUE_OPTIONS, False)
            named = named_entries(operands[1:])
            if named:
                found.append(f"git archive 的路径点名 {'、'.join(named)}")
    return found


def hand_lists(code):
    # 赋给名字的元组、列表或 shell 数组字面量（<名> = ( / [、<名>=(），里面点名了 HAND_LIST_ENTRIES 里 HAND_LIST_MINIMUM 样以上：手抄的拷贝清单
    if broken == "repository-copy-hand-lists-unscanned":
        return []
    found = []
    for assignment in LITERAL_ASSIGNMENT.finditer(code):
        body = bracket_body(code, assignment.start("opening"))
        entries = list(dict.fromkeys(token.group("entry") for token in BASE_ENTRY_TOKEN.finditer(body)))
        if len(entries) >= HAND_LIST_MINIMUM:
            found.append(f"赋给 {assignment.group('name')} 的手抄清单点名 {'、'.join(entries)}")
    return found


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                path = os.path.join(current, name)
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX and path != DEFINITION_FILE:
                    yield directory, path, LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]


problems = []
hits = {}                      # 路径 → [(行号, 说明, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
def scan_file(item):
    """一份文件 → (目录, 路径, 读不了的原因或 None, [(行号, 说明, 原行)])；各份文件互不依赖，在子进程里并行跑。"""
    directory, path, language = item
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        return directory, path, f"读不了 {path}：{error}", []
    if not sources_scanned:
        return directory, path, None, []
    original = text.split("\n")
    entries = []
    for number, code in enumerate(code_lines_of(language, text), 1):
        if not code.strip():
            continue
        found = (python_copies(code) if language == "python" else shell_copies(code)) + hand_lists(code)
        if found:
            entries.append((number, "；".join(found), original[number - 1] if number <= len(original) else code))
    return directory, path, None, entries


for directory, path, read_problem, entries in parallel.map_in_processes(scan_file, list(scanned_files()), jobs):
    if read_problem:
        problems.append(read_problem)
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if entries:
        hits.setdefault(path, []).extend(entries)

exempt_rows, exempt_problems = [], []
if os.path.isfile(EXEMPT_TABLE):
    for number, line in enumerate(open(EXEMPT_TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = EXEMPT_ROW.fullmatch(line)
        if not found or len(found.group("reason")) < 4:
            exempt_problems.append(f"第 {number} 行不是「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）：{line[:120]}")
            continue
        exempt_rows.append((number, found.group("path"), found.group("anchor")))
exempted = 0
unexempted = []
for path, entries in sorted(hits.items()):
    anchors = [anchor for _number, row_path, anchor in exempt_rows if row_path == path]
    for number, described, line in entries:
        if any(anchor in line for anchor in anchors):
            exempted += 1
        else:
            unexempted.append((path, number, described, line))
for number, path, anchor in exempt_rows:
    if not os.path.isfile(path):
        exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
    elif not any(anchor in line for _line_number, _described, line in hits.get(path, [])):
        exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改成经共用库拷，或锚串抄错了）")

grown, growth_note = [], ""
if base_state == "table" and broken != "repository-copy-exempt-growth-unchecked":
    grown = exempt_growth.grown_exempt_rows(exempt_rows, base_table, EXEMPT_ROW, split_family_library_file, broken == "repository-copy-exempt-family-unrecognized")
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有 *.sh、*.py，也没有 {EXEMPT_TABLE}")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处建仓副本的写法没经 scripts/hosts/lib_repository_copy.py（拷源点名了仓根条目，是手抄的拷贝清单：漏了 path 依赖、编译期读的文件、"
          "被起的脚本，副本里就缺）：")  # gate-lint:summary
    for path, number, described, line in unexempted:
        print(f"     {path}:{number}  {described}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：python 调 lib_repository_copy.copy_closure(<仓根>, [(条目, 为什么)…]) 再 copy_entries；shell 起 python3 scripts/hosts/minimal-copy.py <仓根> <副本> "
          "--with <条目>=<为什么>…（只拷一个工作区的走 lib_repository_copy.py link-into）；")
    print("               副本里要起的脚本、要读的数据经 --with / declared 声明，闭包顺着它们 source、起、import 的一起带上；闭包认不出的写法改调用方的写法，不另抄清单。")
    print(f"               确实不是建仓副本的（自证在假仓里换一份文件、钉住一份要调的脚本、从历史提交导出参照树），登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经改成经 lib_repository_copy.py 拷就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path, anchor in grown:
        print(f"     第 {number} 行 {path}:{anchor}")  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的拷法经 lib_repository_copy.py 的 copy_closure / copy_entries（shell 经 minimal-copy.py）；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 建仓副本的写法都经 scripts/hosts/lib_repository_copy.py：扫了 {scanned} 份（{by_directory}），拷源点名仓根条目与手抄清单的 {exempted} 处全在排除表里"
      f"（表 {len(exempt_rows)} 行，每行都命中）；{growth_note}")
PY
}

# ── 格 selftest-cells-alone：有选格手段的自证逐格单跑与整份跑判定相同（.claude/rules/test-script-discipline.md「测试脚本自己也要被测试证明有效」末条：
#   自证同样守「用例原子化：每一条都能单独跑」）。
# 对象：被判的根里 .claude/hooks/selftest-registry.tsv 登记成 selftest、文件里写着「<名字>_SELFTEST_CELLS=<格名…」的自证（有选格手段，读法同 break-switch-replay 格的
#   selection_variable：那个环境变量设成逗号隔开的格名，自证只跑点名的格）。一份都没有：退 77。
# 列格名：先起一趟那份的 --selftest，选格变量设成哪份自证都没有的 selftest-cells-alone-probe：要退 2，并有一行在「逗号隔开：」之后或「（有 …）」里列出认得的格名
#   （admission.py、crash-point-replay.sh、task_server_lib.py 写错格名的那一句就是这样；格名之间空白、顿号或逗号隔开）。没退 2、或一个格名都列不出：那一份判红、不往下起。
# 判据：每一格起两种趟比：整份那一趟（选格变量设成列出的全部格，次序照自证自己的；自证自带的开关轮不跑，判的是格与格之间）里这一格红没红
#   （带 ✗ 的行里有「自证：<格名>：」或「自证 <格名>：」），与单跑那一趟（选格变量只设这一格）退不退 0。两边相同判绿；单跑红、整份里绿判红
#   （这一格读了前面的格留下的变量、文件或环境），单跑绿、整份里红也判红（前面的格留下的状态弄红了它）。整份那一趟超时、或退非 0 而失败行里一格都认不出：
#   那一份要判的格都判红（比不了）。两边都红照判绿：那一格自己坏了，归 research-script-selftests 格判。
#   起法同 break-switch-replay 格：python 的 python3 -B <路径> --selftest、别的 bash <路径> --selftest，环境里去掉 *_BREAK 与 *_SELFTEST_CELLS 再设选格变量；
#   要临时服务的表里登记成 required 的照「$SELFTEST_TEMPORARY_SERVICE_PREFIX」那一句包一层起（--server-binary 给这一格开跑前取的那一份，取不到那几份判红）。
#   每一趟的起法同 break-switch-replay 格「每趟的进程组」（SWITCH_REPLAY_PARALLELISM_LIBRARY 的 TripLauncher）：自成一组，到点（SELFTEST_CELLS_ALONE_TRIP_SECONDS 秒）
#   与直接起的那一个退出之后组里还有进程时，对整组先 TERM、过宽限再 KILL，核完组里一个不剩那一趟才返回、池子里的位置才空出来。
#   整份那一趟或单跑那一趟留下进程、停不掉的，这一格判红、留下的接在说明后面；列格名那一趟留下的，那一份另记一处不对。
#   弄坏开关 CODE_TOOLING_BREAK=replay-trip-leader-only-stopped 同样作用在这一格（同一份起法）；这一格的样本里没有到点与留下进程的趟，打开后判错的样本在 break-switch-replay 格。
# 按条复用（.claude/rules/implementation-workflow.md「复用门禁的判定：按每条判定的单元键判」；共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：
#   一格一条判定（名字「<路径> 格 <格名>」），种类 code-tooling.sh:selftest-cells-alone（prune-server 那一份库，unit-verdicts 族）。
#   键（scripts/gate/admission.py verdict-plan 算，不带路径、行号与限时）：那份自证的单元闭包（入口就是那份脚本，带 program-argument:--selftest，同 break-switch-replay 格①）、
#   判法（令牌 SELFTEST_CELLS_ALONE_JUDGE_TOKEN：selftest_cells_alone_judge_code 打的 SELFTEST_CELLS_ALONE_JUDGE_LIBRARY）、line:command=<起它的命令样子>、
#   line:cell=<选格变量>=<格名>，要临时服务的另加 file:scripts/prune-server/prune_server_client_selftest.py 与 line:temporary-service=wrapped。
#   一份里有一格要现判（键对不上、库里记着红、复用哨兵强制现判）：起它的整份那一趟与要现判的那几格的单跑，别的格照复用；全部复用的那一份一趟都不起（列格名那一趟照起：
#   请求要格名）。一份的趟齐了当场判、当场记（判绿记绿、判红记红撤绿），收尾整份再交一次（哨兵在那一趟比）。拒算的照共用段开跑前判红（ItemVerdicts.stop_on_refusals）。
#   成功句报「按条复用：现判 J、复用 M、拒算 0」与起了几趟。
# 并行：各趟互不依赖；并行条数取登记的每台默认重活线程数（.claude/heavy-thread-default），不按这一趟的内存上限收，理由同 break-switch-replay 格「并行度」：
#   内存上限由 task-server 照这条命令的记录给、照以往一路跑的峰值定，照它收并行就永远收在一路、也永远撞不上上限、服务也就不放大；
#   撞了上限服务照乘 1.5 放大、重排这一条，第二趟只补没记库的（下面「按条复用」：一份的趟齐了当场记）。
#   内存上限只写进开跑那一行（读法 cgroup_memory_limit；样本根的 memory-max 顶替），读不到或是 max 照写原因。
#   外层同时跑的趟数 × 每趟里层的线程数不超过每台默认重活线程数：外层路数的上限 = min(并行条数, 要起的趟数)，
#   里层线程数 = ⌊每台默认重活线程数 ÷ 外层路数的上限⌋（至少 1），经环境变量 SINGLEFS_THREAD_CAP 交给每一趟起的自证（列格名那几趟同样，外层路数取份数）；
#   开跑那一行写「并行 N 趟（登记的每台默认重活线程数 T 条；内存上限 …）」与「外层 N 条 × 里层 M = P（不超过每台默认重活线程数 T）」。
#   自证里不经登记位取并行度的（登记表 .claude/heavy-thread-exempt 点名的、读自己别的线程变量的）收不住这一层。
#   样本根的 .break-switch-replay-jobs（写法同 break-switch-replay 格：threads=<正整数>、memory-max=<字节数|max|unreadable>）顶替线程数与内存上限。
#   收回的趟数与派出去的对不上整格红。
# 进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟；写法在 SWITCH_REPLAY_PROGRESS_LIBRARY）：开跑打一行，每格判完打一行，
#   每过 300 秒打一行总进度；机读进度 TASK_SERVER_PROGRESS 按趟报（同 break-switch-replay 格「机读进度」）。样本根里的 .selftest-cells-alone-progress-seconds 顶替 300 秒。
# 管不到的：格与格之间经别的进程、别的机器传的状态（单跑那一趟也看得到它们）；单跑时退非 0 却不是这一格的毛病（列格名之外的用法错这类）照单跑红记。
# 样本：selftest-cells-alone-red（一份两格的小自证 scripts/gate/twocells.py：consume 那一格读 prepare 那一格留下的全局变量，整份两格都绿、单跑 consume 红：
#   判红并点名「scripts/gate/twocells.py 格 consume：单跑红」）；selftest-cells-alone-green（同一份，consume 自己先调 prepare：判绿，报 1 份自证、2 格）；
#   selftest-cells-alone-reuse-three-questions（两份、各两格：复用三问，change.sh 改 alpha.py 闭包里的一个函数，affected 是 alpha 的两格）；
#   selftest-cells-alone-threads-split-green（样本根的 .break-switch-replay-jobs 写 memory-max 与 threads：上限 100 GiB、线程 8；一份两格的自证三趟：外层 3 条 × 里层 2 = 6，
#   样本自证在每一趟（列格名那一趟除外）里核 SINGLEFS_THREAD_CAP 是 2、不是就退 1，整份那一趟比不了、每格判红）；
#   selftest-cells-alone-jobs-registered-green（样本根的 .break-switch-replay-jobs 写 memory-max=1536 MiB、threads=8，自证同 threads-split 那一份：
#   开跑那一行写并行 8 趟（登记的线程数）、外层 3 条 × 里层 2 = 6，判绿）；selftest-cells-alone-jobs-registered-red（同一组并行度，自证是 selftest-cells-alone-red 那一份
#   再加核里层线程数：并行 8 趟、外层 3 条 × 里层 2 = 6，consume 单跑红、整份里绿，1 处不对）。
# 弄坏开关 CODE_TOOLING_BREAK=cells-alone-whole-only：不单跑，每格照整份那一趟里红没红记，selftest-cells-alone-red 判绿（期望退出 1，实测 0），
#   selftest-cells-alone-threads-split-green、-jobs-registered-green、-jobs-registered-red 判红（只起整份那一趟、外层 1 条，里层拿满线程数 8、样本自证核到的不是 2）；
#   =cells-alone-inner-threads-uncapped：里层线程数不按外层路数分、每趟都拿满每台默认重活线程数，selftest-cells-alone-threads-split-green、-jobs-registered-green、-jobs-registered-red
#   里「外层 3 条 × 里层 2 = 6」那条 want 找不到、每趟拿到的 SINGLEFS_THREAD_CAP 不是 2 而判红；
#   =cells-alone-jobs-by-memory：并行条数照内存上限算（parallel_library.jobs_for：⌊上限的 SAFE_SHARE_PERCENT 份额 ÷ TRIP_MEMORY_FALLBACK_BYTES⌋、不超过线程数），
#   -jobs-registered-green、-jobs-registered-red 并行 1 趟、外层 1 条 × 里层 8，样本自证核到的 SINGLEFS_THREAD_CAP 不是 2、开跑那一行写的是内存上限、每趟与份额，判红。
SELFTEST_CELLS_ALONE_TRIP_SECONDS=1800   # 一趟（列格名、整份、单跑一格）最多几秒；推的：真仓上最慢的整份自证几分钟，留几倍余量
SELFTEST_CELLS_ALONE_JUDGE_LIBRARY="$(cat <<'PY_SELFTEST_CELLS_ALONE_JUDGE'
import re

PROBE_CELL = "selftest-cells-alone-probe"   # 列格名那一趟设进选格变量的名字：哪份自证都没有这一格
CELL_NAME = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*")
LISTING_MARKERS = (re.compile(r"逗号隔开：(.*)$"), re.compile(r"（有 ([^）]*)）"))


def listed_cells(record):
    """列格名那一趟（选格变量设成 PROBE_CELL）→ (格名的列表, 列不出时的一句)：要退 2，并有一行在「逗号隔开：」之后或「（有 …）」里列出认得的格名。"""
    if record["timeout"] or record["code"] != 2:
        return [], f"选格变量设成认不得的 {PROBE_CELL} 时要退 2 并列出认得的格，实际{'超时' if record['timeout'] else '退 ' + str(record['code'])}"
    cells = []
    for line in record["output"].splitlines():
        for marker in LISTING_MARKERS:
            found = marker.search(line)
            if found:
                cells += [word for word in re.split(r"[\s、,，]+", found.group(1)) if CELL_NAME.fullmatch(word) and word != PROBE_CELL]
    cells = list(dict.fromkeys(cells))
    if not cells:
        return [], "退了 2，但没有一行在「逗号隔开：」之后或「（有 …）」里列出格名"
    return cells, ""


def named_red(output, cell):
    """这一格在这一趟的失败行里红了没有：带 ✗ 的行里有「自证：<格名>：」或「自证 <格名>：」。"""
    pattern = re.compile(rf"自证[：\s]{re.escape(cell)}：")
    return any("✗" in line and pattern.search(line) for line in output.splitlines())


def first_failure(record):
    """一趟的头一行失败行（带 ✗ 的），没有写「（没有带 ✗ 的行）」。"""
    return next((line.strip() for line in record["output"].splitlines() if "✗" in line), "（没有带 ✗ 的行）")[:200]


def whole_problem(record, cells):
    """整份那一趟比得了吗：超时、或退非 0 而失败行里一格都认不出 → 比不了的那一句；比得了 → None。"""
    if record["timeout"]:
        return "整份那一趟超时，比不了"
    if record["code"] != 0 and not any(named_red(record["output"], cell) for cell in cells):
        return f"整份那一趟退 {record['code']}，失败行里一格都认不出（{first_failure(record)}），比不了"
    return None


def cell_verdict(cell, alone, whole, broken=""):
    """一格单跑那一趟与整份那一趟比 → (绿没有, 一句)。整份那一趟看失败行点不点名这一格，单跑那一趟看退不退 0。
    弄坏开关 cells-alone-whole-only：不比单跑（alone 是 None），照整份那一趟里红没红记。"""
    whole_red = named_red(whole["output"], cell)
    if broken == "cells-alone-whole-only":
        return not whole_red, "只看整份那一趟"
    alone_red = alone["timeout"] or alone["code"] != 0
    if alone_red == whole_red:
        return True, "单跑与整份判定相同（都红：归 research-script-selftests 格判）" if alone_red else "单跑与整份判定相同"
    if alone_red:
        return False, (f"单跑红（{'超时' if alone['timeout'] else '退 ' + str(alone['code'])}）、整份里绿：这一格读了前面的格留下的变量、文件或环境；"
                       f"单跑那一趟的失败行：{first_failure(alone)}")
    return False, "单跑绿、整份里红：前面的格留下的状态弄红了它"
PY_SELFTEST_CELLS_ALONE_JUDGE
)"
# 这一格的判法入口（单元键里的 judge，这一格文件头「按条复用」）：单元键收这个函数的闭包，格里也照它打的文本装判法
selftest_cells_alone_judge_code() { printf '%s\n' "$SELFTEST_CELLS_ALONE_JUDGE_LIBRARY"; }
cell_selftest_cells_alone() {
  local registry=".claude/hooks/selftest-registry.tsv" jobs alone_path alone_kind alone_needs_binary=0 alone_binary="" alone_required="" alone_scratch="" alone_exit alone_knobs=""
  [[ -f "$registry" ]] || { echo "  ! 没有 $registry，这一格无对象可判"; exit 77; }
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），逐格单跑的并行度定不了，一趟都没跑"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值），再跑这一格；不退回按核数算。"
    exit 1
  fi
  # 只在临时 prune-server 上跑的自证（这一格文件头「判据」末尾）：有选格手段的里有就开跑前取一次二进制，取不到留空（python 那一侧把那几份判红）
  temporary_service_table_load || exit 1
  # 登记成 selftest、文件在的先收齐，再一次 grep -l 挑出写着选格手段的（逐份起 grep 要起二百来次）；grep -l 按给的次序打名字
  local -a alone_candidates=()
  while IFS=$'\t' read -r alone_path alone_kind _; do
    [[ -n "$alone_path" && "$alone_path" != \#* && "$alone_kind" == selftest && -f "$alone_path" ]] && alone_candidates+=("$alone_path")
  done < "$registry"
  if ((${#alone_candidates[@]})); then
    while IFS= read -r alone_path; do
      if temporary_service_is_required "$alone_path"; then
        alone_needs_binary=1
        alone_required+="$alone_path"$'\n'
      fi
    done < <(grep -lE '[A-Z][A-Z0-9_]*_SELFTEST_CELLS=<格名' -- "${alone_candidates[@]}")
  fi
  if (( alone_needs_binary )); then
    alone_scratch="$(mktemp -d)"
    alone_binary="$(code_tooling_prune_server_binary "$alone_scratch")" || alone_binary=""
  fi
  local alone_progress_seconds
  if ! alone_progress_seconds="$(switch_replay_progress_seconds .selftest-cells-alone-progress-seconds)"; then
    printf '%s\n' "$alone_progress_seconds"
    [[ -z "$alone_scratch" ]] || rm -rf -- "$alone_scratch"
    exit 1
  fi
  # 样本根的 .break-switch-replay-jobs（写法同 break-switch-replay 格：threads=、memory-max=）顶替线程数与内存上限：样本的判定不随跑它那台机器变
  if judged_root_is_sample && [[ -f .break-switch-replay-jobs ]]; then
    alone_knobs="$(<.break-switch-replay-jobs)"
  fi
  # 复用三问（这一格文件头「样本」）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds selftest-cells-alone selftest_cells_alone_pass; alone_exit=$?
  else
    selftest_cells_alone_pass ""; alone_exit=$?
  fi
  [[ -z "$alone_scratch" ]] || rm -rf -- "$alone_scratch"
  return "$alone_exit"
}

# selftest_cells_alone_pass <各条结局写到哪，空是不写>：这一格的一趟（列格名、算键、查库、起要现判的、记库、判）；
# registry、jobs、alone_binary、alone_required、alone_progress_seconds 取调用它的那一格的
selftest_cells_alone_pass() {
  python3 - "$(selftest_cells_alone_judge_code)" "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$registry" "${CODE_TOOLING_BREAK:-}" "$jobs" \
    "$alone_binary" "$alone_required" "$SELFTEST_TEMPORARY_SERVICE_PREFIX" "$alone_progress_seconds" "$SWITCH_REPLAY_PROGRESS_LIBRARY" \
    "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:selftest-cells-alone" "$SELFTEST_CELLS_ALONE_JUDGE_TOKEN" "$STAGE_REPOSITORY" \
    "$(judged_root_is_sample && echo 1)" "$1" "$SWITCH_REPLAY_PARALLELISM_LIBRARY" "$SELFTEST_CELLS_ALONE_TRIP_SECONDS" "$alone_knobs" <<'PY'
import collections, concurrent.futures, importlib.util, os, shutil, sys, tempfile, types
judge = types.ModuleType("lib_selftest_cells_alone_judge")
exec(compile(sys.argv[1], "selftest-cells-alone-judge-library", "exec"), judge.__dict__)
runs_library_path, registry_path, broken, default_threads = sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
server_binary, service_required, service_prefix = sys.argv[6], set(sys.argv[7].split("\n")) - {""}, sys.argv[8].split()
progress_seconds = int(sys.argv[9])
progress_library = types.ModuleType("lib_switch_replay_progress")
exec(compile(sys.argv[10], "switch-replay-progress-library", "exec"), progress_library.__dict__)
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[11], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_token, judge_root = sys.argv[12], sys.argv[13], sys.argv[14], sys.argv[15]
sample, states_path = sys.argv[16] == "1", sys.argv[17]
parallel_library = types.ModuleType("lib_switch_replay_parallelism")
exec(compile(sys.argv[18], "switch-replay-parallelism-library", "exec"), parallel_library.__dict__)
trip_seconds = int(sys.argv[19])
try:
    knobs = parallel_library.parse_knobs(sys.argv[20])
except ValueError as error:
    print(f"  ✗ {error}，一趟都没起")
    print(f"     → 怎么办：改样本根的 {parallel_library.KNOB_FILE}（写法在上一行）。")
    sys.exit(1)
default_threads = int(knobs.get("threads", default_threads))   # 登记的每台默认重活线程数（样本根的 threads 顶替）
spec = importlib.util.spec_from_file_location("lib_project_code_runs", runs_library_path)
runs_library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runs_library)
HOW = "bash .claude/gate.d/code-tooling.sh --check selftest-cells-alone（有选格手段的自证逐格单跑、与整份跑比）"
problems = []

# 对象（这一格文件头「对象」）：登记成 selftest、有选格手段的自证 → 选格变量名
selftests, _ = runs_library.read_selftest_registry(registry_path)   # 登记表写坏的行归读它的钩子与 agent-write-scope 格判，这里不另判
variable_of = {}
for path in sorted(selftests):
    if selftests[path].kind != runs_library.SELFTEST_KIND or not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        variable = parallel_library.selection_variable(handle.read())
    if not variable:
        continue
    if path in service_required and not server_binary:
        problems.append(f"{path} 只在临时 prune-server 上跑（要临时服务的表里登记成 required），这一趟取不到 prune-server 二进制（原因在上面一行），一趟都没起")
        continue
    variable_of[path] = variable
if not variable_of and not problems:
    print(f"  ! {registry_path} 里登记成 selftest 的自证没有一份有选格手段（文件里写着「<名字>_SELFTEST_CELLS=<格名…」的），这一格无对象可判")
    sys.exit(77)
needs_service = set(variable_of) & service_required


def command_text(path):
    return f"{'python3 -B' if path.endswith('.py') else 'bash'} {path} --selftest"


def stop_line(text):
    """「到点停整组」那一行。"""
    print(f"  · selftest-cells-alone {text}", flush=True)


# 每趟的起法同 break-switch-replay 格（parallel_library.TripLauncher，这一格文件头「判据」末尾）；门禁收到 TERM、INT、HUP 时各线程收自己那一组，处理在开池之前于主线程装上
launcher = parallel_library.TripLauncher(stop_line, broken == "replay-trip-leader-only-stopped")
launcher.install_stop_on_signals()


def inner_threads_of(outer):
    """外层同时跑 outer 趟时，每一趟里层的线程数 = ⌊每台默认重活线程数 ÷ outer⌋（至少 1）（这一格文件头「并行」）。
    弄坏开关 cells-alone-inner-threads-uncapped：每一趟都拿满每台默认重活线程数、不按外层路数分。"""
    return default_threads if broken == "cells-alone-inner-threads-uncapped" else max(1, default_threads // outer)


def run_trip(path, cells, inner_threads):
    """起一趟 path 的 --selftest，选格变量设成 cells（逗号连），环境变量 SINGLEFS_THREAD_CAP 设成 inner_threads（自证里的并行度经登记位取，登记位认它）→ {timeout, code, output, left}。
    经 launcher 起：自成一组，到点与收尾停整组、核完组里一个不剩才返回；left 是留下进程与停不掉的那一句（都没有是空串）。"""
    environment = {name: value for name, value in os.environ.items() if not name.endswith("_BREAK") and not name.endswith("_SELFTEST_CELLS")}
    environment[variable_of[path]] = ",".join(cells)
    environment["SINGLEFS_THREAD_CAP"] = str(inner_threads)
    command = ["python3", "-B", path, "--selftest"] if path.endswith(".py") else ["bash", path, "--selftest"]
    if path in needs_service:   # 包装写法：-- 前插 --server-binary <这一趟那一份>（这一格文件头「判据」）
        command = [*service_prefix[:-1], "--server-binary", server_binary, "--", *command]
    trip = launcher.run(command, environment, trip_seconds, f"{path} {variable_of[path]}={','.join(cells)}", merge_stderr=True)
    if trip["timeout"]:
        return {"timeout": True, "code": None, "output": "", "left": trip["problem"]}
    return {"timeout": False, "code": trip["code"], "output": trip["stdout"].decode("utf-8", "replace"), "left": trip["problem"]}


# 并行（这一格文件头「并行」）：并行条数取登记的每台默认重活线程数，内存上限只写进开跑那一行。
#   弄坏开关 cells-alone-jobs-by-memory：照内存上限算并行条数（jobs_for；上限读不到或是 max 时这个开关不起作用）
memory_limit, memory_limit_source = parallel_library.memory_limit_of(knobs)
by_memory = broken == "cells-alone-jobs-by-memory" and memory_limit is not None
jobs = parallel_library.jobs_for(default_threads, memory_limit, parallel_library.TRIP_MEMORY_FALLBACK_BYTES) if by_memory else default_threads
if by_memory:
    plan_text = (f"内存上限 {memory_limit // parallel_library.MEBIBYTE} MiB、每趟 {parallel_library.TRIP_MEMORY_FALLBACK_BYTES // parallel_library.MEBIBYTE} MiB、"
                 f"份额 {parallel_library.SAFE_SHARE_PERCENT}%")
else:
    limit_text = f"内存上限{memory_limit_source}" if memory_limit is None else f"内存上限 {memory_limit // parallel_library.MEBIBYTE} MiB（{memory_limit_source}）"
    plan_text = f"登记的每台默认重活线程数 {default_threads} 条；{limit_text}，由服务定、撞了服务照乘 1.5 放大重排，不按它收并行"
# 列格名（这一格文件头「列格名」）：每份一趟，各份并行；外层路数的上限 = min(并行条数, 份数)，里层线程数照 inner_threads_of
cells_of = {}
listing_outer = max(1, min(jobs, len(variable_of)))
with concurrent.futures.ThreadPoolExecutor(max_workers=listing_outer) as pool:
    listings = dict(zip(variable_of, pool.map(lambda path: run_trip(path, [judge.PROBE_CELL], inner_threads_of(listing_outer)), variable_of)))
for path, record in listings.items():
    cells, why = judge.listed_cells(record)
    if record["left"]:
        problems.append(f"{path}：列格名那一趟{record['left']}")
    if not cells:
        problems.append(f"{path}：列不出格名：{why}；照 admission.py、crash-point-replay.sh 的写法，{variable_of[path]} 写错格名时退 2 并在一行里「逗号隔开：」之后列出认得的格")
        continue
    cells_of[path] = cells


def item_name(path, cell):
    return f"{path} 格 {cell}"


def request_line(path, cell):
    tokens = [judge_token, f"judge-root:{judge_root}", f"line:command={command_text(path)}", f"line:cell={variable_of[path]}={cell}", "program-argument:--selftest"]
    if path in needs_service:
        tokens += ["file:scripts/prune-server/prune_server_client_selftest.py", "line:temporary-service=wrapped"]
    return "\t".join([item_name(path, cell), path, *tokens])


names = [item_name(path, cell) for path in cells_of for cell in cells_of[path]]
plan_scratch = tempfile.mkdtemp(prefix="selftest-cells-alone-plan.")
verdicts = rounds.ItemVerdicts(admission, os.getcwd(), kind, plan_scratch, "code-tooling.sh 的 selftest-cells-alone 格", sample, HOW)
if names:
    verdicts.plan([request_line(path, cell) for path in cells_of for cell in cells_of[path]])
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print("  ✗ 有选格手段的自证逐格单跑这一格没判：prune-server 不可用（上面那一句），一趟都没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check selftest-cells-alone")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：这一趟每一格都起、一条都不记库")
    judged, reused, refused = verdicts.report_plan(names, "（有选格手段的自证，一格一条）")
else:
    judged = reused = refused = 0
to_judge = {path: [cell for cell in cells if verdicts.decision(item_name(path, cell))[0] != rounds.REUSE] for path, cells in cells_of.items()}
to_judge = {path: cells for path, cells in to_judge.items() if cells}
# 一份要起的趟：整份一趟加要现判的那几格各一趟（弄坏开关 cells-alone-whole-only 下不单跑）
trips = [(path, None) for path in to_judge] + ([] if broken == "cells-alone-whole-only" else [(path, cell) for path, cells in to_judge.items() for cell in cells])
tracker = progress_library.ReplayProgress("selftest-cells-alone", sum(len(cells) for cells in to_judge.values()), progress_seconds, False, None, True, len(trips))
# 外层同时跑的趟数 × 每趟里层的线程数不超过每台默认重活线程数（这一格文件头「并行」）：外层路数的上限取并行条数与要起的趟数里小的
outer_cap = max(1, min(jobs, len(trips)))
inner_threads = inner_threads_of(outer_cap)
if trips:
    tracker.start(f"selftest-cells-alone 开跑：{len(to_judge)} 份自证、要现判 {sum(len(cells) for cells in to_judge.values())} 格，{len(trips)} 趟，并行 {jobs} 趟（{plan_text}）；"
                  f"外层 {outer_cap} 条 × 里层 {inner_threads} = {outer_cap * inner_threads}（不超过每台默认重活线程数 {default_threads}）；"
                  f"每格判完打一行，每过 {progress_seconds} 秒打一行总进度")
green_of = {}
records = collections.defaultdict(dict)    # 路径 → {None: 整份那一趟, 格名: 单跑那一趟}
waiting = collections.Counter(path for path, _cell in trips)


def settle(path):
    """一份的趟齐了：逐格判、当场记库（这一格文件头「按条复用」）。"""
    whole = records[path][None]
    whole_why = judge.whole_problem(whole, cells_of[path])
    green, red = [], []
    for cell in to_judge[path]:
        if whole_why:
            ok, why = False, whole_why
        else:
            ok, why = judge.cell_verdict(cell, records[path].get(cell), whole, broken)
        # 这一格起的两趟里有没收干净的（这一格文件头「判据」末尾）：判红，留下的接在说明后面
        left = [f"{which}{trip['left']}" for which, trip in (("整份那一趟", whole), ("单跑那一趟", records[path].get(cell))) if trip is not None and trip["left"]]
        if left:
            ok, why = False, "；".join(([] if ok else [why]) + left)
        green_of[item_name(path, cell)] = ok
        (green if ok else red).append(item_name(path, cell))
        if not ok:
            problems.append(f"{item_name(path, cell)}：{why}")
        tracker.item_done("判绿" if ok else "判红", item_name(path, cell), 0)
    verdicts.record(green, red)


def one(job):
    path, cell = job
    record = run_trip(path, cells_of[path] if cell is None else [cell], inner_threads)
    tracker.trip_finished()
    return job, record


collected = 0
with concurrent.futures.ThreadPoolExecutor(max_workers=outer_cap) as pool:
    for future in concurrent.futures.as_completed([pool.submit(one, job) for job in trips]):
        (path, cell), record = future.result()
        collected += 1
        records[path][cell] = record
        waiting[path] -= 1
        if not waiting[path]:
            settle(path)
tracker.close()
if collected != len(trips):
    problems.append(f"派出去 {len(trips)} 趟、收回 {collected} 趟，对不上，这一趟不算数")
verdicts.record([name for name, ok in green_of.items() if ok], [name for name, ok in green_of.items() if not ok], final=True)
if any(not ok for ok in green_of.values()):
    verdicts.reread_after_red()
verdicts.write_states(states_path, names)
shutil.rmtree(plan_scratch, ignore_errors=True)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 有选格手段的自证逐格单跑：{len(problems)} 处不对（{len(cells_of)} 份自证、{len(names)} 格；起了 {len(trips)} 趟）")  # gate-lint:summary
    print("     → 怎么办：单跑红、整份里绿的那一格，让它自己建它要读的状态（变量、文件、环境），不读前面的格留下的（.claude/rules/test-script-discipline.md「用例原子化：每一条都能单独跑」）；"
          "单跑绿、整份里红的，让前面的格收拾干净自己留下的状态；列不出格名的，照出路改那份自证写错格名时的输出；"
          "留下进程或停不掉的，改那份自证让它起的进程在它退出之前都收干净（.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」）；"
          "判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
if verdicts.unavailable:
    print("  ✗ 有选格手段的自证逐格单跑：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check selftest-cells-alone")
    sys.exit(75)
if verdicts.red:
    print("  ✗ 有选格手段的自证逐格单跑判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check selftest-cells-alone；不能复用就判红，不退回全部现判当绿")
    sys.exit(1)
reuse_text = f"按条复用：现判 {judged}、复用 {reused}、拒算 {refused}" if verdicts.ok else "按条复用算不出、全部现判"
if not trips:
    print(f"  ✓ 逐格单跑与整份跑判定相同：{len(cells_of)} 份自证、{len(names)} 格都有键相同的判定，全部复用，这一格只起了列格名那 {len(listings)} 趟（{reuse_text}）")
    sys.exit(0)
print(f"  ✓ 逐格单跑与整份跑判定相同：{len(cells_of)} 份自证、{len(names)} 格（{reuse_text}；起了 {len(trips)} 趟，另有列格名 {len(listings)} 趟）")
PY
}

# ── 格 selftest-parallelism：.claude/hooks/selftest-registry.tsv 登记成 selftest 的每份脚本，自证的成功句报并行度，或文件头写明为什么不能并行
#   （.claude/rules/test-script-discipline.md「编译与算法都要快」：互不依赖的单位按区间切片并行，不能并行的写明为什么）
# 只读文本，不跑自证。判哪些：被判的根里那张登记表第二列正好是 selftest 的行（samples、samples:<目录> 是门禁阶段的样本档，不判）。
# 一份算报了并行度：它的哪一行不以 # 起头（整行注释不算），而这一行里有 PARALLELISM_REPORTED 那种写法——「并行」（或「并行度」）后面隔着空白、冒号、等号接一个数、
#   或接 f 串的 {、shell 的 $、printf 的 %（「并行 {jobs} 路」「并行度：$jobs」）。
# 一份算写明不能并行：文件头（从第一行起，一直到第一行既不是空行、也不以 # 起头的那一行之前；python 的文档串不算文件头）有一行
#   「# selftest-serial: <为什么不能并行>」，理由去掉空白至少 8 个字。理由不到 8 个字的照没写判红。
# 两样都没有的判红，逐份点名；登记表点名的文件不在的同样判红（读不到就判不了）。
# 排除表 .claude/gate.d/selftest-parallelism-exempt.tsv：一行「<路径>  # 理由」（路径与 # 之间至少两个空格，理由至少 4 个字），那一份不判。
#   写不对、指向不存在的文件、指向登记表里不是 selftest 行的路径、指向的那一份已经报了并行度或写明了不能并行（达标了还留着就是白放行），都判红；
#   只缩不涨：与改动范围的基准那一版比（exempt_table_baseline），基准里没有的路径判红；基准里还没有这张表时这一条没比，成功句写明。
# 判不到的：成功句由别的库代打的（这一份里没有那一行）、写在一行代码尾注里的「并行 N」也算报了、并行度报的是不是真的并行，靠 review。
# 样本：selftest-parallelism-red（scripts/gate/silent.py 两样都没有、scripts/gate/commented.py 的「并行 4 路」只在整行注释里、scripts/gate/short.sh 的理由不到 8 个字，
#   排除表一行指向不存在的 scripts/gate/gone.py、一行指向已经报了并行度的 scripts/gate/fine.py：五处逐处点名、退 1）；
#   selftest-parallelism-green（scripts/gate/parallel.py 成功句报「并行 {jobs} 路」、scripts/gate/serial.sh 文件头写明不能并行、scripts/gate/legacy.py 登在排除表里、
#   一行 samples 不判：判绿，报 3 份、报并行度 1、写明不能并行 1、排除 1）。
# 弄坏开关（CODE_TOOLING_BREAK）：selftest-parallelism-report-anywhere（整行注释里的「并行 N」也算报了）：selftest-parallelism-red 里 commented.py 那一条 want 找不到；
#   selftest-parallelism-serial-reason-unchecked（selftest-serial 的理由不看长短）：selftest-parallelism-red 里 short.sh 那一条 want 找不到；
#   selftest-parallelism-exempt-unchecked（排除表的行不核指向）：selftest-parallelism-red 里 gone.py、fine.py 两条 want 找不到。
cell_selftest_parallelism() {
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/gate.d/selftest-parallelism-exempt.tsv "$base_table"
python3 - "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" <<'PY'
import os, re, sys
broken, base_state, base_name, base_table = sys.argv[1:5]

REGISTRY = ".claude/hooks/selftest-registry.tsv"
EXEMPT_TABLE = ".claude/gate.d/selftest-parallelism-exempt.tsv"
JUDGED_KIND = "selftest"
PARALLELISM_REPORTED = re.compile(r"并行度?[ \t]*[:：=]?[ \t]*(?:\{|\$|%|\d)")
SERIAL_HEADER = re.compile(r"#[ \t]*selftest-serial:[ \t]*(?P<reason>.*?)\s*$")
SERIAL_REASON_MINIMUM = 8
EXEMPT_ROW = re.compile(r"(?P<path>[^\s#]+)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
EXEMPT_REASON_MINIMUM = 4
HOWTO = ("照 .claude/rules/test-script-discipline.md「编译与算法都要快」：互不依赖的格与开关轮按每台默认重活线程数并行跑（python 经 scripts/hosts/heavy_threads.py 的 "
         "default_heavy_threads 取，shell 经 scripts/hosts/heavy-threads.sh 的 heavy_thread_default 取），成功句报「并行 N 路」；确实不能并行的（共享状态、次序本身是被测对象），"
         "在文件头写一行「# selftest-serial: <为什么不能并行，至少 8 个字>」")


def visible_length(text):
    return len("".join(text.split()))


def judged_rows():
    """登记表里第二列正好是 selftest 的行 → [(行号, 路径)]；登记表读不了交 None。"""
    try:
        lines = open(REGISTRY, encoding="utf-8").read().split("\n")
    except OSError:
        return None
    rows = []
    for number, line in enumerate(lines, start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 2 and fields[1].strip() == JUDGED_KIND:
            rows.append((number, fields[0].strip()))
    return rows


def parallelism_standing(path):
    """一份自证的现状 → ("reported" 报了并行度 | "serial" 写明了不能并行 | "short" 理由不到 8 个字 | "silent" 两样都没有, 说明)。"""
    lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
    report_anywhere = broken == "selftest-parallelism-report-anywhere"
    for number, line in enumerate(lines, start=1):
        if (report_anywhere or not line.lstrip().startswith("#")) and PARALLELISM_REPORTED.search(line):
            return "reported", f"第 {number} 行报了并行度"
    for line in lines:
        if line.strip() and not line.lstrip().startswith("#"):
            break
        found = SERIAL_HEADER.match(line.strip())
        if found:
            reason = found.group("reason")
            if broken == "selftest-parallelism-serial-reason-unchecked" or visible_length(reason) >= SERIAL_REASON_MINIMUM:
                return "serial", f"文件头写明不能并行：{reason}"
            return "short", f"文件头 selftest-serial 的理由「{reason}」不到 {SERIAL_REASON_MINIMUM} 个字"
    return "silent", "成功句没报并行度，文件头也没写 # selftest-serial: <为什么不能并行>"


rows = judged_rows()
if rows is None:
    print(f"  ⊘ 本次无对象可判：{REGISTRY} 不在")
    sys.exit(77)
registered = {path for _number, path in rows}
exempt_rows, exempt_problems = [], []
if os.path.isfile(EXEMPT_TABLE):
    for number, line in enumerate(open(EXEMPT_TABLE, encoding="utf-8").read().split("\n"), start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = EXEMPT_ROW.fullmatch(line)
        if not found or visible_length(found.group("reason")) < EXEMPT_REASON_MINIMUM:
            exempt_problems.append(f"第 {number} 行写不对（要「<路径>  # 理由」，路径与 # 之间至少两个空格，理由至少 {EXEMPT_REASON_MINIMUM} 个字）：{line.strip()[:160]}")
            continue
        exempt_rows.append((number, found.group("path")))
exempted_paths = {path for _number, path in exempt_rows}

counts = {"reported": 0, "serial": 0, "exempted": 0}
offenders = []
for number, path in rows:
    if not os.path.isfile(path):
        offenders.append(f"{REGISTRY} 第 {number} 行点名的 {path} 不在（读不到就判不了：文件搬了改登记行，删了删登记行）")
        continue
    if path in exempted_paths:
        counts["exempted"] += 1
        continue
    standing, said = parallelism_standing(path)
    if standing in ("reported", "serial"):
        counts[standing] += 1
    else:
        offenders.append(f"{path}：{said}")
if broken != "selftest-parallelism-exempt-unchecked":
    for number, path in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif path not in registered:
            exempt_problems.append(f"第 {number} 行的 {path} 在 {REGISTRY} 里不是 selftest 行，这一格不判它，留着没用")
        elif parallelism_standing(path)[0] in ("reported", "serial"):
            exempt_problems.append(f"第 {number} 行的 {path} 已经达标（{parallelism_standing(path)[1]}），删掉这一行")

grown, growth_note = [], ""
if base_state == "table":
    base_paths = set()
    for line in open(base_table, encoding="utf-8").read().split("\n"):
        found = EXEMPT_ROW.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_paths.add(found.group("path"))
    grown = [(number, path) for number, path in exempt_rows if path not in base_paths]
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

failed = False
if offenders:
    failed = True
    print(f"  ✗ {len(offenders)} 份登记的自证既没报并行度、也没写明为什么不能并行：")  # gate-lint:summary
    for offender in offenders:
        print(f"     {offender}")  # gate-lint:detail
    print(f"     → 怎么办：{HOWTO}；一时改不完的存量登记进 {EXEMPT_TABLE}（「<路径>  # 理由」，只缩不涨），改完就删那一行")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，已经达标或不再是登记的自证就删掉这一行；写法见文件头")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path in grown:
        print(f"     第 {number} 行 {path}")  # gate-lint:detail
    print(f"     → 怎么办：排除只缩不涨。新登记的自证照这一条做：{HOWTO}")
if failed:
    sys.exit(1)
print(f"  ✓ 登记的自证都报了并行度或写明了不能并行：判了 {len(rows)} 份（报并行度 {counts['reported']}、写明不能并行 {counts['serial']}、"
      f"排除 {counts['exempted']}，排除表 {len(exempt_rows)} 行都指得到、都还没达标）；{growth_note}")
PY
}

# ── 格 long-run-split-exempt：跑过两小时也不拆的豁免表 .claude/long-run-split-exempt 的每一行写对
#
# 规则：.claude/rules/long-run-split.md「豁免」：一行 `<项名>  # <理由，至少 8 个字>`，理由里写用户那句话的出处（`records/<文件>.md:<行>`），一项一行，不按目录、不按整道门禁豁免。
# 判据：表里每一行非注释行（空行与 # 起头的行不算）逐条判，任一条不成立判红，逐行点名「文件:行号」：
#   ① 形态：行里有 #，# 左边有项名；
#   ② 理由：# 右边去掉全部空白之后至少 8 个字；
#   ③ 出处：理由里至少一处 records/<文件>.md:<行>，每一处的文件在被判的根里存在、行号在 1 到那份文件的行数之间（行数按换行切出来的行算）；
#   ④ 项名不重复（点名第二次出现那一行与第一次出现的行号）；
#   ⑤ 项名不是目录（带结尾 / 的，或在被判的根里是目录）、不是整道门禁的文件名（.claude/gate.d/ 顶层每个 *.sh 的文件名，带或不带 .sh、带或不带 .claude/gate.d/ 前缀）。
# 表不在、或一行登记行都没有（只有注释与空行）：退 77，这一格无对象可判，不报绿（.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）；
#   成功句报查了多少行、多少个项名。
# 不判：项名在不在该项所属入口的 --list-items 里——要起各入口的 --list-items（门禁、变异、崩溃枚举各有自己的入口，有的要编译、要起服务、要算单元键），是重活，
#   这一格只读文本、不起任何进程；出处那一行是不是用户的原话，靠 review。
# 样本：long-run-split-exempt-red（十一行登记行：没有 #、理由三个字、理由里没有出处、出处的文件不在、出处的行号超过文件行数、项名重复、项名是 scripts/、项名是 records 目录、
#   项名是门禁文件名，另有一行全对，逐行点名判红，报「查了 11 行，10 处不对」）；long-run-split-exempt-green（三行都写对，其中一行带两处出处，报「查了 3 行、3 个项名」）；
#   long-run-split-exempt-empty（只有两行注释，退 77）。
# 弄坏开关 CODE_TOOLING_BREAK=long-run-exempt-form-unchecked：不判①，red 里「不是「<项名>  # <理由>」（没有 #）」那条 want 找不到；
#   =long-run-exempt-reason-unchecked：不判②，red 里「理由去掉空白只有 3 个字」那条 want 找不到；
#   =long-run-exempt-citation-unchecked：不判③，red 里第 4、5、6 行的出处那几条 want 找不到；
#   =long-run-exempt-duplicate-unchecked：不判④，red 里「与第 7 行重复」那条 want 找不到；
#   =long-run-exempt-scope-unchecked：不判⑤，red 里目录与门禁文件名那三条 want 找不到；
#   =long-run-exempt-empty-accepted：没有登记行也往下报绿，empty 判错（期望退出 77，实测 0）。
cell_long_run_split_exempt() {
python3 - "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, sys
TABLE = ".claude/long-run-split-exempt"
MINIMUM_REASON_CHARACTERS = 8
CITATION = re.compile(r"records/([^\s:：]+\.md):(\d+)")   # 出处 records/<文件>.md:<行>
broken = sys.argv[1]
if not os.path.isfile(TABLE):
    print(f"  ! 没有 {TABLE}，这一格无对象可判")
    sys.exit(77)
with open(TABLE, encoding="utf-8", errors="replace") as handle:
    rows = [(number, line) for number, line in enumerate(handle.read().split("\n"), 1) if line.strip() and not line.lstrip().startswith("#")]
if not rows and broken != "long-run-exempt-empty-accepted":
    print(f"  ! {TABLE} 里没有登记行（只有注释与空行），这一格无对象可判")
    sys.exit(77)
# 整道门禁的名字：.claude/gate.d/ 顶层每个 *.sh，带或不带 .sh、带或不带目录前缀
gate_names = set()
for stage in glob.glob(".claude/gate.d/*.sh"):
    file_name = os.path.basename(stage)
    gate_names |= {file_name, file_name[:-len(".sh")], f".claude/gate.d/{file_name}"}
problems, first_row_of = [], {}
for number, line in rows:
    where = f"{TABLE}:{number}"
    item, separator, reason = line.partition("#")
    item, reason = item.strip(), reason.strip()
    if not separator or not item:
        if broken != "long-run-exempt-form-unchecked":
            problems.append(f"{where} 不是「<项名>  # <理由>」（{'没有 #' if not separator else '# 前面没有项名'}）")
            continue
        item = item or line.strip()
    characters = len(re.sub(r"\s+", "", reason))
    if characters < MINIMUM_REASON_CHARACTERS and broken != "long-run-exempt-reason-unchecked":
        problems.append(f"{where} {item} 的理由去掉空白只有 {characters} 个字（至少 {MINIMUM_REASON_CHARACTERS}）")
    citations = CITATION.findall(reason)
    if broken != "long-run-exempt-citation-unchecked":
        if not citations:
            problems.append(f"{where} {item} 的理由里没有用户那句话的出处（写 records/<文件>.md:<行>）")
        for name, digits in citations:
            cited = f"records/{name}"
            if not os.path.isfile(cited):
                problems.append(f"{where} {item} 的出处 {cited}:{digits} 指到的文件不在")
                continue
            with open(cited, encoding="utf-8", errors="replace") as handle:
                total = len(handle.read().splitlines())
            if not 1 <= int(digits) <= total:
                problems.append(f"{where} {item} 的出处 {cited}:{digits} 行号不在 1 到 {total} 之间（那份文件 {total} 行）")
    if item in first_row_of and broken != "long-run-exempt-duplicate-unchecked":
        problems.append(f"{where} {item} 与第 {first_row_of[item]} 行重复（一项一行）")
    first_row_of.setdefault(item, number)
    if broken != "long-run-exempt-scope-unchecked":
        if item.endswith("/") or os.path.isdir(item):
            problems.append(f"{where} {item} 是目录（豁免不按目录）")
        elif item in gate_names:
            problems.append(f"{where} {item} 是整道门禁的文件名（豁免不按整道门禁）")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 豁免表 {TABLE}：查了 {len(rows)} 行，{len(problems)} 处不对")  # gate-lint:summary
    print("     → 怎么办：每行写成「<项名>  # <理由，至少 8 个字，带用户那句话的出处 records/<文件>.md:<行>>」：# 左边是入口 --list-items 列的那一个项名，一项一行、不写目录与门禁文件名；"
          "出处的文件要在、行号不超过那份文件的行数；项拆开或不再超时的，那一行同一次删掉（.claude/rules/long-run-split.md「豁免」）")
    sys.exit(1)
print(f"  ✓ 豁免表 {TABLE}：查了 {len(rows)} 行、{len(first_row_of)} 个项名，都有 #、理由够长、出处的文件在且行号在文件行数以内、项名不重复、不是目录也不是整道门禁；项名在不在入口的 --list-items 里这一格不判")
PY
}
