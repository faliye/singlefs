#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：共用的判法库变量与取表、记库的辅助。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# 列文件的共用写法，research-script-selftest-coverage、break-switch-headers、break-switch-outside-selftest、break-switch-replay 四格用：
#   files_under(目录, 后缀们, 跳过的目录名们) 给出 glob(f"{目录}/**/*") 同一批文件（隐藏的目录与文件不进，符号链接的目录照走），另跳过 skipped 里名字的目录
#   （缺省只跳 target：编译产物目录，scripts/experiment-apparatus 下有几千份、合计几个 GiB，里面没有 .sh 与 .py）；suffixes 给了只留这些后缀的。路径用 / 接，从目录名起头。
#   各格装成模块：tree = types.ModuleType("lib_tree_listing"); exec(compile(sys.argv[<n>], "tree-listing-library", "exec"), tree.__dict__)
TREE_LISTING_LIBRARY="$(cat <<'PY_TREE_LISTING'
import os


def files_under(directory, suffixes=None, skipped=("target",)):
    found = []
    for current, directories, names in os.walk(directory, followlinks=True):
        directories[:] = sorted(name for name in directories if not name.startswith(".") and name not in skipped)
        for name in names:
            if name.startswith(".") or (suffixes is not None and not name.endswith(suffixes)):
                continue
            found.append(os.path.join(current, name))
    return found
PY_TREE_LISTING
)"

# 逐份文件扫描的并行写法，扫源码的几格（memory-bounded-parallelism、repository-copy-closure 等）用：map_in_processes(函数, 一串输入, 并行度) 逐个作用、按输入的次序交结果，
#   fork 出子进程并行（函数与它读的模块级名字要在调用之前定义好，结果要能 pickle）；输入少、并行度 1 的在本进程里跑。每份文件的读与逐行判是互不依赖的纯计算，正则吃的是 CPU，线程帮不上，所以用进程。
#   装法照 TREE_LISTING_LIBRARY。
PARALLEL_MAP_LIBRARY="$(cat <<'PY_PARALLEL_MAP'
import concurrent.futures, multiprocessing

CHUNK = 16


def map_in_processes(function, items, jobs):
    items = list(items)
    if jobs <= 1 or len(items) < 2 * CHUNK:
        return [function(item) for item in items]
    with concurrent.futures.ProcessPoolExecutor(max_workers=min(jobs, len(items)), mp_context=multiprocessing.get_context("fork")) as pool:
        return list(pool.map(function, items, chunksize=CHUNK))
PY_PARALLEL_MAP
)"

# 目录与清单表的双向比对，rules-manifest、stage-owners、agent-write-scope、kb-registry 四格共用；两个方向都判，只判一个方向的另一个方向永远不红。
# 各格的 python 段把它当第一个参数收进来，装成模块：
#   manifest = types.ModuleType("lib_manifest"); exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
MANIFEST_LIBRARY="$(cat <<'PY_MANIFEST'
def two_way(on_disk, registered, is_registered=None, exists=None):
    """返回 (unregistered, dangling)，两份都保持输入的次序（要排序由调用方先排好再传）。

    unregistered：on_disk 里清单没登记的；is_registered(item) 缺省是「item 在 registered 里」。
    dangling：registered 里指向盘上不存在的东西的；exists(entry) 缺省是「entry 在 on_disk 里」。
    两个判定各给各的：盘上那一侧与清单那一侧的集合可以不同
    （例：agent-write-scope 格的盘上一侧是全部定义，清单一侧的 exists 判表里的名字在 .claude/agents/ 里有没有对应的定义文件）。
    """
    on_disk = list(on_disk)
    registered = list(registered)
    registered_set = set(registered)
    on_disk_set = set(on_disk)
    if is_registered is None:
        is_registered = registered_set.__contains__
    if exists is None:
        exists = on_disk_set.__contains__
    unregistered = [item for item in on_disk if not is_registered(item)]
    dangling = [entry for entry in registered if not exists(entry)]
    return unregistered, dangling


def table_rows(path):
    """制表符分隔的清单表：空行与 # 开头的行是注释，不返回；其余每行给 (行号, 去掉换行的整行, 各格)。"""
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            rows.append((line_number, line, line.split("\t")))
    return rows


def outside_markdown_code(text):
    """去掉代码围栏里的行与行内代码：那里的 @ 只是在说这个写法，Claude Code 不展开它（rules-manifest、rules-named-by-definitions 两格读 CLAUDE.md 共用）。"""
    import re
    kept, fence = [], None
    for line in text.split("\n"):
        opening = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if fence is None and opening:
            fence = opening.group(1)
            continue
        if fence is not None:
            if re.match(r"^ {0,3}" + re.escape(fence[0]) + "{" + str(len(fence)) + r",}\s*$", line):
                fence = None
            continue
        kept.append(re.sub(r"(`+)(?:(?!\1).)+?\1", " ", line))
    return "\n".join(kept)
PY_MANIFEST
)"

# 去掉注释行与出路文字之后剩下的代码，research-script-selftest-coverage、stage-order 两格共用：认「实现了 --selftest」「在跑某份自证」「在跑测试」都只认代码。
# 装法照 MANIFEST_LIBRARY：code_text = types.ModuleType("lib_code_without_text"); exec(compile(sys.argv[i], "code-without-text-library", "exec"), code_text.__dict__)
CODE_WITHOUT_TEXT_LIBRARY="$(cat <<'PY_CODE_WITHOUT_TEXT'
import re

# 出路文字：echo / printf / howto / bad / die / say / print( 后面带引号的参数（可以连着几段）。里面提到的「x.py --selftest」不是在跑它
OUTPUT_TEXT = re.compile(r'''(?:\b(?:echo|printf|howto|bad|die|say)\b(?:\s+-\w+)*|\bprint\()(?:\s*f?(?:"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'))+''')


def code_of(text):
    """去掉 # 开头的行（注释与 #!）与出路文字之后剩下的代码。"""
    code = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))
    return OUTPUT_TEXT.sub(" ", code)
PY_CODE_WITHOUT_TEXT
)"

# 逐行只留代码的三种读法（shell、python、rust），heavy-thread-defaults、hardcoded-quota、shared-data-root-literals 三格共用：去掉注释、出路文字与字符串字面量，
# 内容整个是调用方认的那几种字面量（kept_content）留着。装法照 MANIFEST_LIBRARY：
#   source_lines = types.ModuleType("lib_source_code_lines"); exec(compile(sys.argv[i], "source-code-lines-library", "exec"), source_lines.__dict__)
SOURCE_CODE_LINES_LIBRARY="$(cat <<'PY_SOURCE_CODE_LINES'
import io, re, tokenize


def quoted_mask(line):
    """每个字符在不在引号里（shell 的单引号、双引号；双引号里认反斜杠转义）。"""
    mask, quote, index = [False] * len(line), None, 0
    while index < len(line):
        character = line[index]
        if quote:
            mask[index] = True
            if character == "\\" and quote == '"' and index + 1 < len(line):
                mask[index + 1] = True
                index += 2
                continue
            if character == quote:
                quote = None
        elif character in "\"'":
            quote = character
            mask[index] = True
        elif character == "\\":
            index += 2
            continue
        index += 1
    return mask


def literal_kept(literal_text, kept_content):
    """字面量（连引号与前缀）要不要留着：内容整个是 kept_content 认的那几种才留。"""
    found = re.match(r"(?is)^[a-z]*('''|\"\"\"|'|\")(.*)\1$", literal_text)
    return bool(found) and kept_content.fullmatch(found.group(2)) is not None


def shell_code_lines(text, output_text):
    """shell 去掉整行注释、行尾不在引号里的 # 注释与出路文字（output_text 认的 echo / printf / print( 的字面量）。"""
    lines = []
    for line in text.split("\n"):
        if line.lstrip().startswith("#"):
            lines.append("")
            continue
        mask = quoted_mask(line)
        for index, character in enumerate(line):
            if character == "#" and not mask[index] and (index == 0 or line[index - 1] in " \t"):
                line = line[:index]
                break
        lines.append(output_text.sub(" ", line))
    return lines


def python_code_lines(text, output_text, kept_content, scan_strings=False):
    """python 经 tokenize 去掉注释、文档串与字符串字面量（内容整个是 kept_content 认的留着；scan_strings 时字面量一概留着），tokenize 读不了的按 shell 读。"""
    rows = [list(line) for line in text.split("\n")]

    def blank(start, end):
        (start_row, start_column), (end_row, end_column) = start, end
        for row in range(start_row, end_row + 1):
            characters = rows[row - 1]
            first = start_column if row == start_row else 0
            last = end_column if row == end_row else len(characters)
            for column in range(first, min(last, len(characters))):
                characters[column] = " "
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (tokenize.TokenError, SyntaxError, IndentationError):
        return shell_code_lines(text, output_text)
    middle = getattr(tokenize, "FSTRING_MIDDLE", None)
    for token in tokens:
        if token.type == tokenize.COMMENT:
            blank(token.start, token.end)
        elif not scan_strings and ((token.type == tokenize.STRING and not literal_kept(token.string, kept_content)) or (middle is not None and token.type == middle)):
            blank(token.start, token.end)
    return [output_text.sub(" ", "".join(characters)) for characters in rows]


RUST_STRING = re.compile(r'"(?:[^"\\]|\\.)*"')


def rust_code_lines(text, kept_content, scan_strings=False):
    """rust 去掉 // 之后的部分与字符串字面量（内容整个是 kept_content 认的留着；scan_strings 时字面量一概留着）。"""
    lines = []
    for line in text.split("\n"):
        if not scan_strings:
            line = RUST_STRING.sub(lambda found: found.group(0) if literal_kept(found.group(0), kept_content) else " " * len(found.group(0)), line)
        position = line.find("//")
        lines.append(line if position < 0 else line[:position])
    return lines
PY_SOURCE_CODE_LINES
)"

# 弄坏开关的文件头表（.claude/rules/test-script-discipline.md「测试脚本自己也要被测试证明有效」；写法取 scripts/mutation/mutate.sh 那种），
# break-switch-headers、stage-break-switch-replay 两格共用这一份读法（不各写一份）。装法照 MANIFEST_LIBRARY。
#   一条开关：行首（可带 #）缩进之后「<名字以 _BREAK 结尾的变量>=<值>」，后面紧接两个以上空格写弄坏哪一处，或接一段（…）注；值只认小写起头的字母、数字与 _ . , : -。
#     「X_BREAK=<项>」这种占位写法、值后面只隔一个空格接着写命令的用法示例，不算一条。
#   点名的项：下一行起缩进比开关那一行深的几行（到缩进回到同一层、或下一条开关为止）；研究脚本与钩子写自证的项名（括号里写失败行句首），门禁阶段写样本目录名。
#   三样写全：名字与值、弄坏哪一处（同一行的说明或（…）注）、至少一行点名的项。
BREAK_SWITCH_TABLE_LIBRARY="$(cat <<'PY_BREAK_SWITCH_TABLE'
import re

SWITCH_ENTRY = re.compile(r"^\s*#?(?P<indent>\s*)(?P<name>[A-Z][A-Z0-9_]*_BREAK)=(?P<value>[a-z0-9][A-Za-z0-9_.,:-]*)(?P<rest>.*)$")


def comment_body(row):
    """去掉行首的 # 之后的那一段（python 文档串里的行原样）。"""
    stripped = row.lstrip()
    return stripped[1:] if stripped.startswith("#") else row


def indentation(text):
    return len(text) - len(text.lstrip())


def switch_entries(text):
    """→ [{line, name, value, described, items}]：items 是点名的项那几行（去掉 #）连起来的正文。"""
    rows = text.split("\n")
    entries = []
    for index, row in enumerate(rows):
        found = SWITCH_ENTRY.match(row)
        if not found:
            continue
        rest = found.group("rest")
        if rest and not re.match(r"\s{2,}\S|（|\(", rest):
            continue   # 用法示例：值后面只隔一个空格接着写命令
        depth = indentation(comment_body(row))
        items = []
        for following in rows[index + 1:]:
            body = comment_body(following)
            if not body.strip() or SWITCH_ENTRY.match(following) or indentation(body) <= depth:
                break
            items.append(body.strip())
        entries.append({"line": index + 1, "name": found.group("name"), "value": found.group("value"),
                        "described": bool(rest.strip()), "items": "\n".join(items)})
    return entries


def incomplete(entry):
    """缺的那几样（空表是三样写全）。"""
    return [what for what, ok in (("弄坏哪一处", entry["described"]), ("点名的项", bool(entry["items"].strip()))) if not ok]


def named_tokens(entry, candidates):
    """点名的项里出现的候选名字（整词：前后不接字母、数字、_、-）。"""
    text = entry["items"]
    return {name for name in candidates if re.search(r"(?<![\w-])" + re.escape(name) + r"(?![\w-])", text)}
PY_BREAK_SWITCH_TABLE
)"

# 弄坏开关文件头「读没读开关、写全没写全」的判法与复跑登记表的取法：break-switch-headers、break-switch-replay 两格（含复跑格收尾数「没接复跑」那一段）共用这一份，
# 同一份文件头两格结论相同；点名哪一条开关缺什么、点名的项里哪一行的「」没配平的句子也只在这里拼（gap_sentence）。「」配不配平以复跑格读点名的项用的 quoted_items
# （BREAK_SWITCH_REPLAY_JUDGE_LIBRARY）读出的各项为准：复跑格里开关表与判法本来在同一个模块（break_switch_replay_judge_code 打的两段），break-switch-headers 格把判法那一段另装进去。
# 装进开关表读法的同一个模块（BREAK_SWITCH_TABLE_LIBRARY 先装，这一段接着装进它的 __dict__，用它的 switch_entries、incomplete、quoted_items），装法：
#   table = types.ModuleType("lib_break_switch_table"); exec(compile(sys.argv[N], "break-switch-table-library", "exec"), table.__dict__)
#   exec(compile(sys.argv[K], "break-switch-replay-judge-library", "exec"), table.__dict__)   # 复跑格已经装了的不重装
#   exec(compile(sys.argv[M], "break-switch-header-library", "exec"), table.__dict__)
# 不并进 BREAK_SWITCH_TABLE_LIBRARY：那一段是两个开关复跑格单元键里的判法（break_switch_replay_judge_code 打的），多一个函数登记表里每份脚本的每条开关都作废重跑；
# 这一段只在复跑开跑前预检文件头时用，不进任何一条判定的键。
# 拆分族（SplitFamilies）：一份入口拆成入口加部分文件时，读开关的代码与文件头的开关表都可以在部分文件里；读没读开关、读了哪几个变量（switch_names）、
#   开关表（family_entries，每条带它写在哪一份）、写全没写全（family_gaps）、汇总行声明（family_declaration）都按整族算（入口加它的部分文件，入口那一份在前）；
#   部分文件读的变量整族的开关表里没有的（uncovered_reads）点名。部分文件的认法取 scripts/gate/lib_lag_split_family.py（按命名认族：
#   <名>_<部分>.py、<名>-<部分>.sh、<名>-<部分>.py，门禁阶段的 lib/ 下 <阶段名>-<部分>.sh），装法 load_split_family(那一份的路径)，样本根里没有它，取这一道所在仓里的。
#   命名认得宽（同目录里 gate.sh 与 gate-staged.sh 按命名是一族，实际是两个脚本），所以部分文件还要入口的代码（整行注释不算）里提到它的名字（去掉后缀）才算：
#   入口里只在注释里提到的、没提到的，是各自独立的脚本，照单个文件判；提到了部分文件的部分文件再提到的同样算。复跑登记表点名的文件各自跑自己的自证，永远是入口、不当别人的部分文件。
#   装法：exec 这一段之后 families = SplitFamilies(load_split_family(路径), independent=复跑登记表点名的文件们)。
BREAK_SWITCH_HEADER_LIBRARY="$(cat <<'PY_BREAK_SWITCH_HEADER'
import importlib.util, os, re

REPLAY_REGISTRY_PATH = ".claude/gate.d/break-switch-replay-files.tsv"   # break-switch-replay 格复跑的文件：它判没写全的，break-switch-headers 格同样判

READS_BREAK_SWITCH = re.compile(r"\$\{?([A-Z][A-Z0-9_]*_BREAK)\b|environ(?:\.get)?\s*[\(\[]\s*[\"']([A-Z][A-Z0-9_]*_BREAK)[\"']|getenv\(\s*[\"']([A-Z][A-Z0-9_]*_BREAK)[\"']"
                                r"|^\s*[A-Z_][A-Z0-9_]*\s*=\s*[\"']([A-Z][A-Z0-9_]*_BREAK)[\"']", re.M)


def code_of(text):
    """去掉整行注释之后的代码。"""
    return "\n".join(row for row in text.split("\n") if not row.lstrip().startswith("#"))


def read_switch_names(text):
    """代码（整行注释不算）里读的名字以 _BREAK 结尾的变量名，排好序：shell 的 $X_BREAK / ${X_BREAK…}，python 的 environ.get("X_BREAK") / environ["X_BREAK"] / getenv("X_BREAK") 与 = "X_BREAK" 的常量。"""
    return sorted({name for found in READS_BREAK_SWITCH.findall(code_of(text)) for name in found if name})


def load_split_family(library_path):
    """scripts/gate/lib_lag_split_family.py（拆分族的认法，纯函数）从它的路径装成模块。"""
    spec = importlib.util.spec_from_file_location("lib_lag_split_family", library_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def mentions(code, part):
    """入口的代码里提到 part 这份文件的名字（文件名去掉后缀，前后不接字母、数字、_、-）：入口载入它的判据，python 的 import、shell 的 source 与起子进程都写着它的名字。"""
    stem = os.path.basename(part).rsplit(".", 1)[0]
    return re.search(r"(?<![\w-])" + re.escape(stem) + r"(?![\w-])", code) is not None


class SplitFamilies:
    """入口加它的部分文件（拆分族）：这一份只读文件系统里的 .py、.sh（路径相对判的根，调用时的当前目录），按文件名与入口的代码认。
    independent：各自当入口的文件（复跑登记表点名的），不当别人的部分文件。
    alone、by_name、entry_table_only 只给弄坏开关用：alone 不认部分文件（每份各判各的），by_name 只按命名认、不核入口的代码提没提到它，
    entry_table_only 读开关按整族、开关表与汇总行声明只读入口那一份的文件头。"""

    def __init__(self, split_family, independent=(), alone=False, by_name=False, entry_table_only=False):
        self.split_family, self.independent, self.alone, self.by_name, self.entry_table_only = split_family, set(independent), alone, by_name, entry_table_only
        self.texts, self.listings, self.parts = {}, {}, {}

    def text_of(self, path):
        if path not in self.texts:
            with open(path, encoding="utf-8", errors="replace") as handle:
                self.texts[path] = handle.read()
        return self.texts[path]

    def scripts_in(self, directory):
        """directory（仓内相对路径，空是根）下的 .py、.sh，带 directory 前缀；目录不在是空。"""
        if directory not in self.listings:
            prefix = directory + "/" if directory else ""
            names = sorted(os.listdir(directory or ".")) if os.path.isdir(directory or ".") else []
            self.listings[directory] = [prefix + name for name in names if name.endswith((".py", ".sh")) and os.path.isfile(prefix + name)]
        return self.listings[directory]

    def parts_of(self, entry):
        """entry 的部分文件们（不含它自己）：同目录与同目录 lib/ 下按命名在它的族里、又被它或已认进来的部分文件的代码提到的。"""
        if entry not in self.parts:
            directory = os.path.dirname(entry)
            pending = [path for where in (directory, directory + "/lib" if directory else "lib") for path in self.scripts_in(where)
                       if path != entry and path not in self.independent and self.split_family.in_family(entry, path)]
            found = []
            while not self.alone:
                taken = [path for path in pending if self.by_name or any(mentions(code_of(self.text_of(member)), path) for member in [entry, *found])]
                if not taken:
                    break
                found.extend(taken)
                pending = [path for path in pending if path not in taken]
            self.parts[entry] = found
        return self.parts[entry]

    def entry_of(self, path):
        """path 是部分文件时它所属的入口（同目录、或它在 lib/ 下时上一层目录里，名字最短的那一份）；不是部分文件（含入口自己）交 path 自己。"""
        if self.alone or path in self.independent:
            return path
        directory = os.path.dirname(path)
        homes = [directory]
        if os.path.basename(directory) == "lib":
            homes.append(os.path.dirname(directory))
        owners = [entry for home in homes for entry in self.scripts_in(home) if entry != path and self.split_family.in_family(entry, path) and path in self.parts_of(entry)]
        return min(owners, key=len) if owners else path

    def switch_names(self, entry):
        """入口与它的部分文件的代码里读的弄坏开关变量名，排好序。"""
        return sorted({name for member in [entry, *self.parts_of(entry)] for name in read_switch_names(self.text_of(member))})

    def header_files(self, entry):
        """文件头里找开关表与汇总行声明的那几份：入口加部分文件，入口在前。"""
        return [entry] if self.entry_table_only else [entry, *self.parts_of(entry)]

    def tables_of(self, entry):
        """文件头写了开关表（switch_entries 读得出至少一条）的那几份，入口在前。"""
        return [member for member in self.header_files(entry) if switch_entries(self.text_of(member))]

    def family_entries(self, entry):
        """整族的开关表：switch_entries 的各条，每条另带 file（它写在哪一份）；入口那一份在前。"""
        return [{**item, "file": member} for member in self.tables_of(entry) for item in switch_entries(self.text_of(member))]

    def family_gaps(self, entry, ignored=()):
        """整族的开关表没写全的地方 → [(那一处所在的文件, header_gaps 的一项)]：有开关表的每一份各按 header_gaps 判；整族一条都读不出的是入口的「开关表」一项。"""
        tables = self.tables_of(entry)
        if not tables:
            return [(entry, gap) for gap in header_gaps(self.text_of(entry), ignored)]
        return [(member, gap) for member in tables for gap in header_gaps(self.text_of(member), ignored)]

    def family_declaration(self, entry):
        """整族文件头声明的汇总行 → ([汇总行原文], [写得不对的说明])：各份各按 summary_declaration 读，部分文件的说明前面写它的路径。"""
        items, problems = [], []
        for member in self.header_files(entry):
            declared, bad = summary_declaration(self.text_of(member))
            items.extend(declared)
            problems.extend(why if member == entry else f"{member} 的{why}" for why in bad)
        return items, problems

    def uncovered_reads(self, entry):
        """部分文件读的弄坏开关变量里整族的开关表没有的 → [(部分文件, [变量名])]；整族一条开关表都没有时交空（入口那一处「文件头没有开关表」已点名）。"""
        covered = {item["name"] for item in self.family_entries(entry)}
        if not covered:
            return []
        found = [(part, [name for name in read_switch_names(self.text_of(part)) if name not in covered]) for part in self.parts_of(entry)]
        return [(part, missing) for part, missing in found if missing]


QUOTES_PAIRED = "点名的项里「」配平"   # header_gaps 的 ignored 里写它：不判点名的项里「」配不配平（只给弄坏开关用）


def quote_gap(entry):
    """点名的项里「」配不配平 → 没配平的说明（写出最先碰到的那一处在第几行与怎么改），配平是 None。
    配不配平以 quoted_items（复跑读点名的项用的那一份）读出的各项为准：文本里有一个「或」没算进读出的项里，就是没配平，复跑读不到它点名的是哪一格。"""
    text = entry["items"]
    found = quoted_items(text)
    if (text.count("「") == sum(item.count("「") + 1 for item in found)
            and text.count("」") == sum(item.count("」") + 1 for item in found)):
        return None
    opened = []
    for position, character in enumerate(text):
        if character == "「":
            opened.append(position)
        elif character == "」":
            if not opened:
                return f"第 {entry['line'] + 1 + text.count(chr(10), 0, position)} 行多一个收尾的括号，补上它对应的起头括号，或删掉这个"
            opened.pop()
    return f"第 {entry['line'] + 1 + text.count(chr(10), 0, opened[0] if opened else 0)} 行起头的括号没有收尾，在该收住的地方补上收尾的括号"


def header_gaps(text, ignored=()):
    """文件头开关表没写全的地方 → [(行号, "<名字>=<值>", 缺的那几样, 点名的项里「」没配平的说明或 None)]；一条开关都读不出的是 [(None, None, ["开关表"], None)]；
    空表示三样都写全、点名的项里「」配平。ignored 里的样不算缺（只给弄坏开关用；QUOTES_PAIRED 是不判「」配平）。"""
    entries = switch_entries(text)
    if not entries:
        return [(None, None, ["开关表"], None)]
    gaps = []
    for entry in entries:
        missing = [what for what in incomplete(entry) if what not in ignored]
        unpaired = None if QUOTES_PAIRED in ignored else quote_gap(entry)
        if missing or unpaired:
            gaps.append((entry["line"], f"{entry['name']}={entry['value']}", missing, unpaired))
    return gaps


def gap_sentence(path, gap):
    """header_gaps 的一项 → 点名这一条开关缺什么、点名的项里哪一行的「」没配平的一句（两格都打这一句）。"""
    line, switch, missing, unpaired = gap
    if line is None:
        return f"{path}：代码里读了弄坏开关，文件头没有一条「X_BREAK=<值>  <弄坏哪一处>」再下一行起写点名的项的开关"
    parts = ([f"缺{'、'.join(missing)}"] if missing else []) + ([f"点名的项里「」没配平：{unpaired}"] if unpaired else [])
    return f"{path}:{line} {switch} " + "；".join(parts)


SUMMARY_DECLARATION = re.compile(r"^\s*#?\s*break-summary-rows\s*[:：](?P<rest>.*)$")
SUMMARY_DECLARATION_MINIMUM_CHARACTERS = 8   # 汇总行原文去掉省略的那几段之后至少这么多个字，写得太短会把任何一行多红都当汇总行


def summary_declaration(text):
    """文件头声明的汇总行：行首（可带 #）「break-summary-rows：」后面各个「…」里的原文（… 是中间省掉的一段，各段按次序出现在同一行里，读法同点名的项）
    → ([汇总行原文], [写得不对的说明])；没有这一行是 ([], [])。声明了的脚本，复跑照声明认汇总行：这一趟各条开关趟多出来的 ✗ 行里照得出的，不要求点名；
    不再靠「一半以上开关下都多红的同一行」认，只点单条开关的复跑也认得出。原文要以 ✗ 起头（汇总行是失败行），去掉 … 之后不到
    SUMMARY_DECLARATION_MINIMUM_CHARACTERS 个字的写得不对。"""
    items, problems = [], []
    for number, row in enumerate(text.split("\n"), 1):
        found = SUMMARY_DECLARATION.match(row)
        if not found:
            continue
        declared = quoted_items(found.group("rest"))
        if not declared:
            problems.append(f"第 {number} 行 break-summary-rows 后面没有「…」写的汇总行原文")
        for item in declared:
            kept = "".join(piece.strip() for piece in re.split(r"…+|\.\.\.", item))
            if not item.lstrip().startswith("✗") or len(kept) < SUMMARY_DECLARATION_MINIMUM_CHARACTERS:
                problems.append(f"第 {number} 行 break-summary-rows 的「{item[:60]}」要以 ✗ 起头、去掉 … 之后至少 {SUMMARY_DECLARATION_MINIMUM_CHARACTERS} 个字")
        items.extend(declared)
    return items, problems


def replay_registry_paths(registry_path):
    """复跑登记表（一行「<路径><TAB>…」，# 起头的行与空行不算）第一列点名的文件，按表里的次序；表不在是空。"""
    if not os.path.isfile(registry_path):
        return []
    with open(registry_path, encoding="utf-8", errors="replace") as handle:
        return [line.rstrip("\n").split("\t")[0] for line in handle if line.strip() and not line.startswith("#")]
PY_BREAK_SWITCH_HEADER
)"

# break-switch-replay 格的判法（单元键里的 judge，那一格文件头「按条复用」）：输出怎么收、失败行怎么减基准趟、点名的项怎么比、汇总行怎么认、哪条算绿。
# 格里与开关表的读法拼成一个模块装（break_switch_replay_judge_code 打的就是这两段），只放判法、不起进程；起自证、限时、重跑与进度在格自己那一段。
BREAK_SWITCH_REPLAY_JUDGE_LIBRARY="$(cat <<'PY_BREAK_SWITCH_REPLAY_JUDGE'
import collections, re

# 基准趟与开关趟的失败行按整行比：临时路径、临时文件名与数字每趟都可能不同，比之前换成同一个记号
VOLATILE = re.compile(r"/tmp/[^\s'\"：，、）)\]」]+|\btmp[A-Za-z0-9_]{6,}|[0-9]+")


def quoted_items(text):
    """点名的项：「…」里的原文（嵌套的括号配平）。"""
    found, depth, start = [], 0, None
    for index, character in enumerate(text):
        if character == "「":
            if depth == 0:
                start = index + 1
            depth += 1
        elif character == "」" and depth:
            depth -= 1
            if depth == 0:
                found.append(text[start:index])
    return found


def appears_in(item, row):
    """点名的一项照不照得出这一行：… 是中间省掉的一段，各段按次序出现在同一行里。"""
    pieces = [piece.strip() for piece in re.split(r"…+|\.\.\.", item) if piece.strip()]
    position = 0
    for piece in pieces:
        at = row.find(piece, position)
        if at < 0:
            return False
        position = at + len(piece)
    return bool(pieces)


def decoded_output(streams, broken):
    """[(那一路的名字, 收到的字节或文本)] → (输出, 坏字节的说明)：各路照 UTF-8 解，解不了的照 U+FFFD 换掉接着判、另记一条；
    单独的 \\r 与 \\r\\n 都当换行（弄坏开关 replay-carriage-return-kept 下单独的 \\r 留在行里）。"""
    texts, bad = [], []
    for stream, data in streams:
        if isinstance(data, str):
            texts.append(data)
            continue
        try:
            texts.append(data.decode("utf-8"))
        except UnicodeDecodeError as error:
            bad.append(f"{stream} 里有不合法的 UTF-8（第 {error.start} 字节起：{data[error.start:error.start + 8].hex(' ')}）")
            texts.append(data.decode("utf-8", errors="replace"))
    output = "".join(texts)
    if broken != "replay-carriage-return-kept":
        output = output.replace("\r\n", "\n").replace("\r", "\n")
    return output, bad


def failure_rows(output):
    return [row.strip() for row in output.split("\n") if "✗" in row]


def extra_rows(rows, baseline_rows):
    """这一趟的失败行减去基准趟的（去掉临时路径与数字之后按整行比，按次数抵）。"""
    remaining = collections.Counter(VOLATILE.sub("#", row) for row in baseline_rows)
    extra = []
    for row in rows:
        key = VOLATILE.sub("#", row)
        if remaining[key] > 0:
            remaining[key] -= 1
        else:
            extra.append(row)
    return extra


def baseline_green(record):
    """基准趟（不设开关起的那一趟）算绿：没超时、退 0、没有坏字节。"""
    return not record["timeout"] and record["code"] == 0 and not record["bad"]


def summary_keys_of(records, baseline_rows, broken):
    """一份脚本这一趟的各条开关趟里红了的有一半以上（至少两条）都多红的同一行（去掉临时路径与数字之后）：汇总行，不要求点名。"""
    if broken == "replay-summary-rows-required":
        return frozenset()
    counts, red = collections.Counter(), 0
    for record in records:
        if record["timeout"] or record["code"] in (0, None):
            continue
        red += 1
        counts.update({VOLATILE.sub("#", row) for row in extra_rows(failure_rows(record["output"]), baseline_rows) if row.startswith("✗")})
    return frozenset(key for key, count in counts.items() if count >= 2 and count * 2 > red)


def judge(entry, record, baseline_rows, summary_keys, broken):
    """一条开关趟 → (结局, 说明)：结局是 对、守卫、超时、没红、没点名、不符之一；坏字节另记，不在这里。"""
    if record["timeout"]:
        return "超时", "超过登记的秒数没跑完"
    output, code = record["output"], record["code"]
    if record["bad"] and broken == "replay-bad-bytes-first":
        return "对", ""
    if code == 2 and any(entry["name"] in row and ("自证" in row or "selftest" in row.lower()) for row in output.split("\n")):
        return "守卫", ""
    if code == 0:
        return "没红", "设上之后自证照样退 0（开关没弄坏什么）"
    items = quoted_items(entry["items"])
    if not items:
        return "没点名", "点名的项里没有可比的「…」原文（失败行句首要写在「」里）"
    extra = extra_rows(failure_rows(output), baseline_rows)
    missing = [item for item in items if broken != "replay-items-unchecked" and not any(appears_in(item, row) for row in extra)]
    unnamed = [] if broken == "replay-unnamed-rows-unchecked" else [
        row for row in extra if row.startswith("✗") and VOLATILE.sub("#", row) not in summary_keys and not any(appears_in(item, row) for item in items)]
    if missing or unnamed:
        parts = []
        if missing:
            parts.append("点名的项没在失败行里：" + "；".join(f"「{item[:60]}」" for item in missing[:3]) + (f" 等 {len(missing)} 项" if len(missing) > 3 else ""))
        if unnamed:
            parts.append("失败行没被点名：" + "；".join(f"「{row[:80]}」" for row in unnamed[:3]) + (f" 等 {len(unnamed)} 行" if len(unnamed) > 3 else ""))
        return "不符", "；".join(parts)
    return "对", ""


def switch_green(kind, record):
    """一条开关判完算绿：对上（对、守卫）且没有坏字节。"""
    return kind in ("对", "守卫") and not record["bad"]
PY_BREAK_SWITCH_REPLAY_JUDGE
)"

# stage-break-switch-replay 格的判法（单元键里的 judge，那一格文件头「按条复用」）：stage-selftest.sh 的输出里判错的样本怎么认、算不算跑成、点名的与实测比出哪一句红。
# 格里与开关表的读法拼成一个模块装（stage_break_switch_replay_judge_code 打的就是这两段），只放判法、不起进程。
STAGE_BREAK_SWITCH_REPLAY_JUDGE_LIBRARY="$(cat <<'PY_STAGE_BREAK_SWITCH_REPLAY_JUDGE'
def wrong_samples(output, stage, samples):
    """stage-selftest.sh 报「✗ <阶段> <样本> …」的那几份（只认这一道的样本目录名）。"""
    wrong = set()
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 3 and fields[0] == "✗" and fields[1] == stage[:-3] and fields[2] in samples:
            wrong.add(fields[2])
    return wrong


def run_finished(code, output):
    """stage-selftest.sh 跑成了：退 0 或 1，且打了「判得都对」或「个用例判错」那一句。"""
    return code in (0, 1) and ("判得都对" in output or "个用例判错" in output)


def problem_of(stage, entry, switch, named, wrong, finished, code, output, broken):
    """一条复跑的结局 → 判红的那一句；对上了 → None。逐条进度、记库与收尾的判红用同一份。"""
    if not finished:
        return f"{stage}:{entry['line']} {switch}：stage-selftest.sh 没跑成（退 {code}；末尾：{' | '.join(output.strip().splitlines()[-2:])[:200]}）"
    if named != wrong and broken != "switch-replay-named-unchecked":
        return f"{stage}:{entry['line']} {switch}：点名的与实测判错的不等（点了没错：{'、'.join(sorted(named - wrong)) or '无'}；错了没点：{'、'.join(sorted(wrong - named)) or '无'}）"
    return None
PY_STAGE_BREAK_SWITCH_REPLAY_JUDGE
)"

# 两格开关复跑的判法入口：单元键收这两个函数的闭包（读到的顶层变量连同 here-doc 正文，改一句判法就作废），格里也照它们打的文本装判法，判的与进键的是同一份
break_switch_replay_judge_code() { printf '%s\n' "$BREAK_SWITCH_TABLE_LIBRARY" "$BREAK_SWITCH_REPLAY_JUDGE_LIBRARY"; }
stage_break_switch_replay_judge_code() { printf '%s\n' "$BREAK_SWITCH_TABLE_LIBRARY" "$STAGE_BREAK_SWITCH_REPLAY_JUDGE_LIBRARY"; }

# 按条复用的共用段（stage-break-switch-replay、break-switch-replay、reuse-selftests、function-units-two-implementations-agree、mutation-list-items、entry-scripts、
#   selftest-cells-alone 七格，不各写一份；research-script-selftests 格是 shell 写的那一份，样板同它）：
#   经 scripts/gate/admission.py 的 verdict-plan 算键、查库（复用哨兵在它里面抽），verdict-record 照键记绿、记红撤绿，失灵与服务不可用交 plan-failure 判；
#   开跑前判拒算（ItemVerdicts.stop_on_refusals）：计划里有一条拒算，逐种打拒算的原因（形态与文件:行）、出路，这一格当场退 1、一条都不起（拒算的照跑也入不了库，只认拒算 0）；
#   样本 break-switch-replay-refused-red，弄坏开关 CODE_TOOLING_BREAK=refused-items-run-anyway（不判，照跑到底）；
#   样本根另放的 .unit-verdict-sample/prior/（上一版的树）先照判绿记进样本自己的判定库，.unit-verdict-sample/red-over-green 在时计划另设 ADMISSION_BREAK=verdict-lookup-skipped
#   （不读库、一律现判），判红记库之后再算一次计划、逐条打现判的原因（门禁样本根里 admission.py 才认这个开关）。装法照 BREAK_SWITCH_TABLE_LIBRARY。
#   另有 report_plan（计划那一句与逐条原因）、content_digest（被判那棵树里当数据读的文件与目录折成键里的 line:）、toolchain_line（rustc -vV 与 cargo -V）、
#   TripJournal（一趟的结局按键落盘、取回，break-switch-replay 格文件头「开关趟落盘」）、SampleStop（样本档造中途被停，code_tooling_interrupted_rounds）。
#   收尾那一趟（record 带 final）verdict-record 退 0 时它打的「!」行原样转进日志，至多 RECORD_SKIP_LINES_SHOWN 行、多出的报「另有 N 条」（ItemVerdicts.report_record_skips；
#   一份判完就记的那几趟不转，收尾那一趟照同一份计划再比一次）；样本 break-switch-replay-record-skips-forwarded-green，弄坏开关 CODE_TOOLING_BREAK=record-skip-lines-dropped。
#   判成「没对上（不符）」的条才重算键（ItemVerdicts.key_changes，stage-break-switch-replay 与 break-switch-replay 两格在汇总前调它）：照 plan 交进去的同一批请求里那几条起同一条
#   admission.py verdict-plan（计划文件写 -），与开跑前算的键不同就是跑的过程中键变了（别的会话在写这一条判法闭包里的单元；活的主工作区上才有，冻结快照树里不会）；
#   那一条的失败行接一句「跑的过程中键变了（开跑前算的键、现在的键）：组成清单有 N 处不同：…（至多 KEY_CHANGE_UNITS_SHOWN 处，余下报条数）；出路：在冻结快照树里重判…」，
#   变了的单元取两把键各自在库里的组成清单比（library-files fetch 搬来 compositions/<键>.tsv.gz）；键没变的那一条失败行一个字不加，这一条的红绿与退出码都不改。
#   没有「没对上（不符）」的趟一个进程都不多起；重算只写一行「· 重算键：…」。样本 break-switch-replay-key-changed-red、stage-break-switch-replay-key-changed-red（跑的过程中键变了）
#   与 break-switch-replay-mismatch-detail-red、stage-break-switch-replay-red（键没变，「键变了 0 条」）；弄坏开关 CODE_TOOLING_BREAK=key-change-not-rechecked（不重算）、
#   key-change-noted-always（一概当键变了）。
#   起 admission.py 的每一趟子命令（admission_run，plan-failure 也经它）放进它自己的会话与进程组，每秒读一次 /proc 里这一组的 CPU 滴答（自己的与收掉的子进程的都算）：
#   整组连着 ADMISSION_IDLE_SECONDS 秒一个滴答都没涨、也没有进程在 D 状态（等盘），就是卡住了（锁、没设时限的子进程、套接字这类睡着等的），先 TERM、过
#   ADMISSION_STOP_GRACE_SECONDS 秒还有活的再 KILL，核组里一个不剩；输出文件末尾接一行「admission.py <子命令> 卡住：…」并照打进日志，退 ADMISSION_STALLED_EXIT，
#   调用方照「退非 0」交 plan-failure 判（失灵，那一格判红）。限时低于任务服务判卡住的 600 秒（不打日志、不用 CPU 满 600 秒），高于 prune-server 客户端
#   一次收发的上限（回应 120 秒加重连 120 秒，到了它自己退 75）；在算的一直有滴答涨，不受这一条限制。调用方收到停止信号或抛异常时，在跑的那一组照样停整组再往外抛。
#   样本根的 .unit-verdict-sample/admission-stall（门禁样本档专用）：verdict-plan 那一趟换成一条只睡 ADMISSION_STALL_SAMPLE_SLEEP_SECONDS 秒的命令、
#   卡住的限时改成 ADMISSION_STALL_SAMPLE_IDLE_SECONDS 秒；样本 break-switch-replay-admission-stall-red。
#   弄坏开关 CODE_TOOLING_BREAK=admission-run-unbounded：不盯 CPU、不设卡住的限时，子命令卡住就一直等。
UNIT_VERDICT_ROUNDS_LIBRARY="$(cat <<'PY_UNIT_VERDICT_ROUNDS'
import contextlib, gzip, hashlib, json, os, shutil, signal, subprocess, sys, threading, time

REUSE, JUDGE, REFUSED = "reuse", "judge", "refused"
REFUSAL_REASONS_SHOWN = 20   # 开跑前判拒算（ItemVerdicts.stop_on_refusals）时逐个列出的拒算原因至多几种
KEY_CHANGE_UNITS_SHOWN = 3   # 判成没对上的那一条跑的过程中键变了时，失败行里列出变了的单元至多几处，余下报条数（ItemVerdicts.key_changes）
KEY_CHANGE_REMEDY = ("出路：在冻结快照树里重判这一条（python3 scripts/hosts/minimal-copy.py <仓根> <快照树> --with-gate-tables --with-test-inputs 取树，"
                     "两趟之间谁都不改它，树里起这一格；别的会话写这一条判法闭包里的单元时，活树上的键会在跑的过程中变）")
RECORD_SKIP_LINES_SHOWN = 20   # 收尾那一趟 verdict-record 退 0 时转进日志的「!」行至多几行（ItemVerdicts.report_record_skips）
RECORD_SKIP_LINES_SAMPLE_MARKER = "record-skip-lines-shown"   # 样本根 .unit-verdict-sample/ 下的这一份写一个正整数，顶替 RECORD_SKIP_LINES_SHOWN
FORCED_REASON_PREFIX = "强制现判"
STATE_WORDS = {REUSE: "复用", JUDGE: "现判", REFUSED: "拒算"}
LIBRARY_DIRECTORY_NAME = "singlefs-unit-verdicts"   # admission.py 的 VERDICT_LIBRARY_DIRECTORY_NAME：库名只当命名空间，本地不建
STOP_AFTER_MARKER = ".unit-verdict-sample/interrupted-resume/stop-after"
STOP_ARMED_VARIABLE = "CODE_TOOLING_SAMPLE_STOP_ARMED"
SAMPLE_STOP_EXIT = 137
ADMISSION_IDLE_SECONDS = 300          # admission.py 一趟子命令整组连着这么多秒 CPU 一个滴答都没涨、也没有进程在等盘，就当卡住（admission_run）
ADMISSION_POLL_SECONDS = 1            # 隔多久读一次那一组的 CPU 滴答
ADMISSION_STOP_GRACE_SECONDS = 3      # 卡住时先 TERM，过这么多秒组里还有活的再 KILL
ADMISSION_STALLED_EXIT = 124          # 卡住被停的那一趟交回的退出码（调用方照退非 0 交 plan-failure）
ADMISSION_STALL_SAMPLE_MARKER = "admission-stall"   # 样本根 .unit-verdict-sample/ 下的这一份在时，verdict-plan 那一趟换成只睡不算的命令（ItemVerdicts.plan）
ADMISSION_STALL_SAMPLE_IDLE_SECONDS = 2
ADMISSION_STALL_SAMPLE_SLEEP_SECONDS = 20


def admission_group_activity(group):
    """进程组 group 里各进程的 CPU 滴答之和（utime、stime 加已收掉的子进程的 cutime、cstime）、有没有进程在 D 状态、还剩几个进程（读 /proc/<进程号>/stat）。"""
    ticks, waiting_on_disk, members = 0, False, 0
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        fields = text[text.rfind(")") + 2:].split()   # 进程名在括号里、可能带空格：从右括号之后数，第 0 个是状态、第 2 个是进程组号、第 11 到 14 个是四样 CPU 滴答
        if len(fields) < 15 or fields[2] != str(group):
            continue
        members += fields[0] != "Z"   # 已退出、等收的不算还活着
        ticks += sum(int(value) for value in fields[11:15])
        waiting_on_disk = waiting_on_disk or fields[0] == "D"
    return ticks, waiting_on_disk, members


def admission_stop_group(group):
    """停自己用新会话起的那一组：先 TERM，过 ADMISSION_STOP_GRACE_SECONDS 秒还有活的再 KILL → 停完组里还剩几个进程。"""
    with contextlib.suppress(ProcessLookupError):
        os.killpg(group, signal.SIGTERM)
    deadline = time.monotonic() + ADMISSION_STOP_GRACE_SECONDS
    while admission_group_activity(group)[2] and time.monotonic() < deadline:
        time.sleep(0.1)
    if admission_group_activity(group)[2]:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(group, signal.SIGKILL)
        time.sleep(0.5)
    return admission_group_activity(group)[2]


def admission_run(admission, arguments, output_path, environment_changes=None, idle_seconds=ADMISSION_IDLE_SECONDS, command=None):
    """起一趟 admission.py <子命令>（command 给了就起它，样本档造卡住用）：标准输出与标准错误写进 output_path → (退出码, 输出)。
    整组连着 idle_seconds 秒没涨 CPU 滴答、也没有进程等盘就停整组，输出末尾接「卡住」那一行、退 ADMISSION_STALLED_EXIT（共用段上方的注释）。"""
    environment = dict(os.environ)
    environment.update(environment_changes or {})
    unbounded = break_on("admission-run-unbounded")
    stall_note = ""
    with open(output_path, "w", encoding="utf-8") as handle:
        child = subprocess.Popen(command or ["python3", admission, *arguments], stdout=handle, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL, env=environment,
                                 start_new_session=True)
        try:
            last_ticks, quiet_since = -1, time.monotonic()
            while True:
                try:
                    child.wait(timeout=ADMISSION_POLL_SECONDS)   # 退了当场返回，不多等一个间隔
                    break
                except subprocess.TimeoutExpired:
                    pass
                ticks, waiting_on_disk, _members = admission_group_activity(child.pid)
                if ticks != last_ticks or waiting_on_disk:
                    last_ticks, quiet_since = ticks, time.monotonic()
                elif not unbounded and time.monotonic() - quiet_since >= idle_seconds:
                    left = admission_stop_group(child.pid)
                    child.wait()
                    stall_note = (f"admission.py {arguments[0] if arguments else '?'} 卡住：整组连着 {idle_seconds} 秒 CPU 一个滴答都没涨、也没有进程在等盘（睡着等锁、子进程或套接字），"
                                  f"停了整组（先 TERM、过 {ADMISSION_STOP_GRACE_SECONDS} 秒再 KILL）" + (f"，组里还剩 {left} 个进程停不掉" if left else "、组里一个不剩")
                                  + f"，这一趟当退 {ADMISSION_STALLED_EXIT}")
                    break
        except BaseException:
            admission_stop_group(child.pid)
            child.wait()
            raise
        if not stall_note and admission_group_activity(child.pid)[2]:
            admission_stop_group(child.pid)   # 子命令退了而它起的进程还在组里：收掉，不留给这一格之后
    if stall_note:
        with open(output_path, "a", encoding="utf-8") as handle:
            handle.write(stall_note + "\n")
        print(f"  ! {stall_note}", flush=True)
    with open(output_path, encoding="utf-8", errors="replace") as handle:
        return (ADMISSION_STALLED_EXIT if stall_note else child.returncode), handle.read()


def last_lines(text, count=2, width=300):
    return " ".join(line.strip() for line in text.strip().split("\n")[-count:])[:width]


def plan_rows(output):
    """verdict-plan 逐条那几行：[(名字, 决定, 原因)]。"""
    rows = []
    for line in output.split("\n"):
        fields = line.split("\t", 3)
        if len(fields) == 4 and fields[1] in (REUSE, JUDGE, REFUSED):
            rows.append((fields[0], fields[1], fields[3]))
    return rows


def plan_keys(output):
    """verdict-plan 逐条那几行：{名字: 键}；拒算的没有键，不进。"""
    keys = {}
    for line in output.split("\n"):
        fields = line.split("\t", 3)
        if len(fields) == 4 and fields[1] in (REUSE, JUDGE):
            keys[fields[0]] = fields[2]
    return keys


def break_on(switch):
    """弄坏开关（CODE_TOOLING_BREAK，逗号连几个）里有没有 switch。"""
    return switch in os.environ.get("CODE_TOOLING_BREAK", "").split(",")


def composition_diff(earlier, later):
    """两份组成清单（行的列表）比出变了什么 → 说明句的清单：单元行（种类、限定名、文件:起止行、指纹）按（种类、限定名、文件）比指纹，别的输入行按原文比。
    比法同 admission.py 的 changed_composition（键对不上时报变了的单元那一份）；不载入它：载入就是把整份 admission.py 读进这一格的进程。"""
    def split(lines):
        units, others = {}, set()
        for line in lines:
            parts = line.split("\t")
            if len(parts) == 4 and parts[0] not in ("external", "tool", "file") and ":" in parts[2]:
                units[(parts[0], parts[1], parts[2].split(":", 1)[0])] = parts[3]
            else:
                others.add(line)
        return units, others

    earlier_units, earlier_others = split(earlier)
    later_units, later_others = split(later)
    notes = []
    for identity in sorted(set(earlier_units) | set(later_units)):
        kind, name, file = identity
        label = f"{kind} {file}" if name == file else f"{kind} {file}::{name}"
        if identity not in earlier_units:
            notes.append(f"多了 {label}")
        elif identity not in later_units:
            notes.append(f"少了 {label}")
        elif earlier_units[identity] != later_units[identity]:
            notes.append(f"变了 {label}")
    notes.extend(f"新的输入行 {line.replace(chr(9), ' ')[:160]}" for line in sorted(later_others - earlier_others))
    notes.extend(f"没了的输入行 {line.replace(chr(9), ' ')[:160]}" for line in sorted(earlier_others - later_others))
    return notes


class ItemVerdicts:
    """一格一趟的按条复用。ok：计划算出来了；red：复用失灵或配置错（这一格判红）；unavailable：prune-server 不可用（这一格退 75）；problem：算不出的那一句。"""

    def __init__(self, admission, root, kind, scratch, caller, sample, how, record_red=True):
        self.admission, self.root, self.kind, self.scratch, self.caller = admission, root, kind, scratch, caller
        self.sample, self.how, self.record_red = sample, how, record_red
        self.decisions, self.keys, self.names, self.problem = {}, {}, [], ""
        self.request_of = {}   # 名字 → plan 交进去的那一行请求（key_changes 照同一批请求重算）
        self.ok = self.red = self.unavailable = False
        self.lock, self.serial = threading.Lock(), 0

    def path(self, name):
        return os.path.join(self.scratch, name)

    def marker(self, name):
        return self.sample and os.path.exists(os.path.join(self.root, ".unit-verdict-sample", name))

    def failure(self, command, exit_code, output_path, red_withdrawal=()):
        """失灵没有交 plan-failure 判（admission.py 文件头「按条复用失灵」）：它打 ✗ 与出路；退 75 记不可用，别的非 0 记红。
        red_withdrawal 是交了红的那一趟 verdict-record 的（计划文件, 判红的名字文件）：带 --red 交，记红没成时它撤掉那几把键原有的绿。"""
        red_arguments = ["--red", self.root, self.kind, *red_withdrawal] if red_withdrawal else []
        with self.lock:
            self.serial += 1
            serial = self.serial
        code, output = admission_run(self.admission, ["plan-failure", self.caller, command, str(exit_code), *red_arguments, output_path], self.path(f"failure-{serial}.out"))
        text = output.rstrip("\n")
        if text:
            print(text, flush=True)
        if code == 75:
            self.unavailable = True
        elif code != 0:
            self.red = True

    def seed_prior(self):
        """样本根的 .unit-verdict-sample/prior/：拿上一版的树算同一批请求的键、照判绿记进样本自己的判定库。"""
        prior = os.path.join(self.root, ".unit-verdict-sample", "prior")
        if not (self.sample and os.path.isdir(prior)):
            return True
        code, output = admission_run(self.admission, ["verdict-plan", prior, self.kind, self.path("requests"), self.path("seed.plan")], self.path("seed.out"))
        if code != 0:
            self.problem = f"样本：上一版的树上算不出键（{last_lines(output)}）"
            self.failure("verdict-plan", code, self.path("seed.out"))
            return False
        with open(self.path("seed.names"), "w", encoding="utf-8") as handle:
            handle.write("".join(name + "\n" for name in self.names))
        code, output = admission_run(self.admission, ["verdict-record", prior, self.kind, self.path("seed.plan"), self.path("seed.names"), "--how", "样本：上一版"],
                                     self.path("seed.record"))
        if code != 0:
            self.problem = f"样本：上一版那几条没记进库（{last_lines(output)}）"
            self.failure("verdict-record", code, self.path("seed.record"))
            return False
        print(f"  · 样本：上一版（.unit-verdict-sample/prior）的 {len(self.names)} 条照判绿记进这一版样本的判定库", flush=True)
        return True

    def plan(self, request_lines):
        """算键、查库；交算没算出来。哨兵那几行（抽中谁、种子）照打进日志。"""
        self.names = [line.split("\t", 1)[0] for line in request_lines]
        self.request_of = {line.split("\t", 1)[0]: line for line in request_lines}
        with open(self.path("requests"), "w", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in request_lines))
        if not self.seed_prior():
            return False
        changes = {"ADMISSION_BREAK": "verdict-lookup-skipped"} if self.marker("red-over-green") else {}
        # 样本档造卡住（共用段上方的注释）：verdict-plan 那一趟换成只睡不算的命令、卡住的限时改短
        stall = ({"idle_seconds": ADMISSION_STALL_SAMPLE_IDLE_SECONDS, "command": ["sleep", str(ADMISSION_STALL_SAMPLE_SLEEP_SECONDS)]}
                 if self.marker(ADMISSION_STALL_SAMPLE_MARKER) else {})
        code, output = admission_run(self.admission, ["verdict-plan", self.root, self.kind, self.path("requests"), self.path("plan")], self.path("plan.out"), changes, **stall)
        if code != 0 or not any(line.startswith("PLAN ") for line in output.split("\n")):
            self.problem = f"admission.py verdict-plan {'退 ' + str(code) if code else '没打 PLAN 那一行'}（{last_lines(output)}）"
            self.failure("verdict-plan", code, self.path("plan.out"))
            return False
        for line in output.split("\n"):
            if "哨兵" in line and "\t" not in line:
                print(line.rstrip(), flush=True)
        self.decisions = {name: (decision, reason) for name, decision, reason in plan_rows(output)}
        self.keys = plan_keys(output)
        self.stop_on_refusals()
        os.symlink("plan", self.path("plan-per-entry"))
        self.ok = True
        return True

    def stop_on_refusals(self):
        """开跑前判拒算：拒算的条单元闭包收不全、键算不出，照跑也一条都入不了库、下一趟照样现判，跑到底只是白跑。计划里有拒算的，
        逐种打拒算的原因（admission.py 交的那一句：拒算的形态、文件:行与那一行）、条数与头一条，打出路，删掉这一趟的临时目录（scratch，各格自己建的），
        这一格当场退 1、一条都不起。拒算没有登记表可豁免（崩溃测试函数的 .claude/gate.d/refused-tests.tsv 只管 crash-point-replay），只认拒算 0。
        弄坏开关 CODE_TOOLING_BREAK=refused-items-run-anyway：不判，拒算的条照跑到底、不入库。"""
        refused = [(name, reason) for name, (decision, reason) in self.decisions.items() if decision == REFUSED]
        if not refused or "refused-items-run-anyway" in os.environ.get("CODE_TOOLING_BREAK", "").split(","):
            return
        names_of = {}
        for name, reason in refused:
            names_of.setdefault(reason, []).append(name)
        summary = f"{self.caller}：开跑前算键，{len(self.decisions)} 条里拒算 {len(refused)} 条（单元闭包收不全、键算不出，照跑也一条都入不了库），这一格当场判红、一条都没起"
        print(f"  ✗ {summary}", flush=True)  # gate-lint:summary
        for reason, names in list(names_of.items())[:REFUSAL_REASONS_SHOWN]:
            print(f"  ✗ {reason}（{len(names)} 条，头一条「{names[0]}」）", flush=True)  # gate-lint:detail
        if len(names_of) > REFUSAL_REASONS_SHOWN:
            print(f"    …拒算的原因另有 {len(names_of) - REFUSAL_REASONS_SHOWN} 种没列", flush=True)
        print("     → 怎么办：改掉上面点名的那一处（「拒算：」后面是拒算的形态与文件:行），写成闭包折得出的写法（认得出的写法在 scripts/gate/lib_function_units.py 文件头），"
              f"再跑：{self.how}；拒算没有登记表可豁免，这一格只认拒算 0", flush=True)
        shutil.rmtree(self.scratch, ignore_errors=True)
        sys.exit(1)

    def decision(self, name):
        if not self.ok:
            return JUDGE, "按条复用算不出，照判"
        return self.decisions.get(name, (JUDGE, "计划里没有它"))

    def state(self, name):
        """复用三问认的结局（scripts/gate/lib_reuse_checks.py 文件头「一」）：复用、现判、强制现判、拒算。"""
        decision, reason = self.decision(name)
        if decision == JUDGE and reason.startswith(FORCED_REASON_PREFIX):
            return "强制现判"
        return STATE_WORDS.get(decision, "现判")

    def forced(self, name):
        """这一条是复用哨兵抽中强制现判的那一条。"""
        return self.state(name) == "强制现判"

    def record(self, green, red, final=False):
        """照计划里的键记：判绿记绿，判红记红、撤同一把键原有的绿（record_red 是 False 时不交判红的）。
        一份判完就记的那几趟用不带复用哨兵状态的计划（plan-per-entry），收尾那一趟用带它的（哨兵在那一趟判）。"""
        if not self.ok or not (green or red):
            return
        with self.lock:
            self.serial += 1
            serial = self.serial
        green_path, red_path, output_path = self.path(f"record-{serial}.green"), self.path(f"record-{serial}.red"), self.path(f"record-{serial}.out")
        with open(green_path, "w", encoding="utf-8") as handle:
            handle.write("".join(name + "\n" for name in green))
        with open(red_path, "w", encoding="utf-8") as handle:
            handle.write("".join(name + "\n" for name in red))
        arguments = ["verdict-record", self.root, self.kind, self.path("plan" if final else "plan-per-entry"), green_path, "--how", self.how]
        if red and self.record_red:
            arguments += ["--red-names", red_path]
        code, output = admission_run(self.admission, arguments, output_path)
        if code == 0:
            if final and green:
                print(f"  · 判绿记库：{last_lines(output, 1)}", flush=True)
            if final and red and self.record_red:
                print(f"  · 判红记库：{len(red)} 条记红、撤掉同一把键原有的绿，下一趟这几条现判（{last_lines(output, 1)}）", flush=True)
            if final:
                self.report_record_skips(output)
            return
        plan_path = self.path("plan" if final else "plan-per-entry")
        if red and self.record_red and "red-record-failure-kept-green" in os.environ.get("CODE_TOOLING_BREAK", "").split(","):
            # 弄坏开关 CODE_TOOLING_BREAK=red-record-failure-kept-green：记红没成只打一行「!」、不交 plan-failure，同一把键原有的绿留着、下一趟照复用
            print(f"  ! 判定记库：admission.py verdict-record 没成（{last_lines(output)}），下一趟这几条照判", flush=True)
            return
        if red and self.record_red:
            # 记红没成（admission.py 文件头「按条复用失灵」）：带 --red 交 plan-failure，它判失灵、撤掉这几把键原有的绿（撤不掉说整道判红）
            print(f"  · 判定记库：admission.py verdict-record 退 {code}（{last_lines(output)}），记红没成，交 plan-failure 判、撤同一把键原有的绿", flush=True)
            self.failure("verdict-record", code, output_path, (plan_path, red_path))
            return
        print(f"  ! 判定记库：admission.py verdict-record 没成（{last_lines(output)}），下一趟这几条照判", flush=True)
        self.failure("verdict-record", code, output_path)

    def record_skip_lines_shown(self):
        """转进日志的「!」行至多几行：RECORD_SKIP_LINES_SHOWN；样本根 .unit-verdict-sample/record-skip-lines-shown 写着正整数的照它（样本不必造二十多条）。"""
        if not self.marker(RECORD_SKIP_LINES_SAMPLE_MARKER):
            return RECORD_SKIP_LINES_SHOWN
        with open(os.path.join(self.root, ".unit-verdict-sample", RECORD_SKIP_LINES_SAMPLE_MARKER), encoding="utf-8") as handle:
            text = handle.read().strip()
        return int(text) if text.isdigit() and int(text) > 0 else RECORD_SKIP_LINES_SHOWN

    def report_record_skips(self, output):
        """收尾那一趟 verdict-record 退 0 时它逐条打的「!」行（跑的过程中键变了、不记的，写不进服务的；admission.py 文件头 verdict-record）原样转进日志：
        记库的输出文件（scratch 下的 record-<序号>.out）收尾就删，不转就只剩 RECORD 行里的 changed= 一个数。至多 record_skip_lines_shown() 行，多出的报一句「另有 N 条」。
        弄坏开关 CODE_TOOLING_BREAK=record-skip-lines-dropped：一行都不转。"""
        if "record-skip-lines-dropped" in os.environ.get("CODE_TOOLING_BREAK", "").split(","):
            return
        skipped = [line.rstrip() for line in output.split("\n") if line.lstrip().startswith("!")]
        shown = self.record_skip_lines_shown()
        for line in skipped[:shown]:
            print(line, flush=True)
        if len(skipped) > shown:
            print(f"  ! 记库没记的另有 {len(skipped) - shown} 条没转进日志（verdict-record 这一趟共打 {len(skipped)} 条「!」行）", flush=True)

    def library_name(self):
        """这一格的判定库名：<被判那棵树的 git common-dir>/<LIBRARY_DIRECTORY_NAME>/<种类>（只当名字，本地不建）；取不到 git common-dir 是空串。"""
        done = subprocess.run(["git", "-C", self.root, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                              capture_output=True, text=True, errors="replace", stdin=subprocess.DEVNULL)
        common_directory = done.stdout.strip() if done.returncode == 0 else ""
        return f"{common_directory}/{LIBRARY_DIRECTORY_NAME}/{self.kind}" if common_directory else ""

    @staticmethod
    def stored_composition(directory, key):
        """library-files fetch 搬到 directory 底下的那把键的组成清单（行的列表）；没搬来、解不开是 None。"""
        try:
            with gzip.open(os.path.join(directory, "compositions", f"{key}.tsv.gz"), "rt", encoding="utf-8", errors="surrogateescape") as handle:
                return handle.read().rstrip("\n").split("\n")
        except (OSError, EOFError):
            return None

    def changed_units(self, keys_of):
        """{名字: (开跑前的键, 现在的键)} → ({名字: 变了的单元的说明句清单，取不到是 None}, 取不到的原因)：
        两把键的组成清单取库里按键存的那两份（verdict-plan 算键时存的 compositions/<键>.tsv.gz，admission.py library-files fetch 搬来）。"""
        library, problem, directory = self.library_name(), "", self.path("recheck-compositions")
        if not library:
            problem = "取不到被判那棵树的 git common-dir，判定库名拼不出"
        elif keys_of:
            relatives = sorted({f"compositions/{key}.tsv.gz" for pair in keys_of.values() for key in pair})
            code, output = admission_run(self.admission, ["library-files", "fetch", library, directory, *relatives], self.path("recheck-fetch.out"))
            if code != 0:
                problem = f"admission.py library-files fetch 退 {code}（{last_lines(output)}）"
        notes_of = {}
        for name, (earlier_key, later_key) in keys_of.items():
            earlier, later = (None, None) if problem else (self.stored_composition(directory, key) for key in (earlier_key, later_key))
            notes_of[name] = None if earlier is None or later is None else composition_diff(earlier, later)
        return notes_of, problem or "库里没有那两把键的组成清单"

    def key_changes(self, names, fresh_requests=None):
        """判成没对上的那几条（names）里，跑的过程中键变了的 → {名字: 失败行里接在后面的一句}；键都没变、没有计划（按条复用算不出）、names 空的是 {}。
        没对上的条才重算：照 plan 交进去的同一批请求里那几条，起同一条 admission.py verdict-plan（计划文件写 -：不落计划、不存哨兵状态），不另写算键；
        请求里有调用方自己算的输入（line: 行，例：stage-break-switch-replay 格喂的样本目录摘要）而它们也会在跑的过程中变的，调用方照此刻的内容重拼那几条，经 fresh_requests（{名字: 请求行}）交进来；
        开跑前算的键与此刻重算的不同就是键变了，变了的单元取两把键各自的组成清单比（至多 KEY_CHANGE_UNITS_SHOWN 处，余下报条数），出路写在冻结快照树里重判。
        重算只写一行「· 重算键：…」，不改这一条的红绿与退出码；重算没成只打一行「!」，不当键没变。
        弄坏开关 CODE_TOOLING_BREAK=key-change-not-rechecked：不重算、一条都不报；key-change-noted-always：重算出来的一概当键变了（键没变的也接那一句）。"""
        asked = [name for name in names if name in self.keys]
        if not self.ok or not asked or break_on("key-change-not-rechecked"):
            return {}
        requests_path = self.path("recheck.requests")
        with open(requests_path, "w", encoding="utf-8") as handle:
            handle.write("".join((fresh_requests or {}).get(name, self.request_of[name]) + "\n" for name in asked))
        code, output = admission_run(self.admission, ["verdict-plan", self.root, self.kind, requests_path, "-"], self.path("recheck.out"))
        if not any(line.startswith("PLAN ") for line in output.split("\n")):
            print(f"  ! 重算键没成（admission.py verdict-plan 退 {code}：{last_lines(output)}）：没对上的 {len(asked)} 条的键在跑的过程中变没变，这一趟说不准", flush=True)
            return {}
        keys_now = plan_keys(output)
        changed = [name for name in asked if break_on("key-change-noted-always") or keys_now.get(name) != self.keys[name]]
        print(f"  · 重算键：没对上的 {len(asked)} 条照开跑前同一份请求重算，键变了 {len(changed)} 条", flush=True)
        notes_of, unreadable = self.changed_units({name: (self.keys[name], keys_now[name]) for name in changed if name in keys_now})
        sentences = {}
        for name in changed:
            earlier_key, later_key = self.keys[name], keys_now.get(name)
            head = f"跑的过程中键变了（开跑前算的键 {earlier_key[:12]}，现在{'算出的键 ' + later_key[:12] if later_key else '算不出键'}）"
            notes = notes_of.get(name)
            if later_key is None:
                detail = "现在这一条的单元闭包收不全、键算不出"
            elif notes is None:
                detail = f"变了的单元读不到（{unreadable}）"
            elif not notes:
                detail = "两把键的组成清单相同，键的写法变了"
            else:
                shown = "；".join(notes[:KEY_CHANGE_UNITS_SHOWN])
                rest = f"；另有 {len(notes) - KEY_CHANGE_UNITS_SHOWN} 处没列" if len(notes) > KEY_CHANGE_UNITS_SHOWN else ""
                detail = f"组成清单有 {len(notes)} 处不同：{shown}{rest}"
            sentences[name] = f"{head}：{detail}；{KEY_CHANGE_REMEDY}"
        return sentences

    def reread_after_red(self):
        """样本根带 .unit-verdict-sample/red-over-green 时：判红记库之后不设 ADMISSION_BREAK 再算一次计划，打复用几条、现判几条，现判的逐条写原因。"""
        if not self.marker("red-over-green"):
            return
        code, output = admission_run(self.admission, ["verdict-plan", self.root, self.kind, self.path("requests"), self.path("reread.plan")], self.path("reread.out"))
        if code != 0:
            print(f"  ✗ 样本档：判红记库之后再算计划没成（{last_lines(output)}）")
            print("     → 怎么办：看括号里 admission.py verdict-plan 报的错，修好样本或判定库，再单跑这一份样本")
            return
        rows = plan_rows(output)
        print(f"  · 样本档：判红记库之后再算一次计划：复用 {sum(1 for row in rows if row[1] == REUSE)} 条、现判 {sum(1 for row in rows if row[1] == JUDGE)} 条", flush=True)
        for name, decision, reason in rows:
            if decision == JUDGE:
                print(f"    现判「{name}」：{reason}", flush=True)

    def write_states(self, path, names, ran_anyway=()):
        """复用三问那一趟的各条结局（一行「名字<TAB>结局」）；path 空不写。ran_anyway 是计划答复用却照样起了的那几条（各格的 <格>-plan-ignored 弄坏开关：
        每趟整格现跑），记现判。"""
        if path:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("".join(f"{name}\t{'现判' if name in ran_anyway else self.state(name)}\n" for name in names))

    def report_plan(self, names, what, shown_at_most=20):
        """计划算出来之后打一句「· 按条复用：<条数> 条<what>，现判 J、复用 M、拒算 R」，现判与拒算逐条写原因（至多 shown_at_most 条），
        复用的不到 shown_at_most 条或判的是样本时也逐条写；交 (现判, 复用, 拒算)。"""
        decided = [(name, *self.decision(name)) for name in names]
        judged = sum(1 for _name, decision, _reason in decided if decision == JUDGE)
        refused = sum(1 for _name, decision, _reason in decided if decision == REFUSED)
        reused = len(decided) - judged - refused
        print(f"  · 按条复用：{len(decided)} 条{what}，现判 {judged}、复用 {reused}、拒算 {refused}", flush=True)
        fresh = [(name, decision, reason) for name, decision, reason in decided if decision != REUSE]
        for name, decision, reason in fresh[:shown_at_most]:
            print(f"    {STATE_WORDS[decision]}「{name}」：{reason}", flush=True)
        if len(fresh) > shown_at_most:
            print(f"    …现判与拒算另有 {len(fresh) - shown_at_most} 条没列", flush=True)
        if reused <= shown_at_most or self.sample:
            for name, decision, reason in decided:
                if decision == REUSE:
                    print(f"    复用「{name}」：{reason}", flush=True)
        return judged, reused, refused


class TripJournal:
    """一趟的结局按键落盘（break-switch-replay 格文件头「开关趟落盘」）：相对路径 trips/<键>.json，放在这一格自己那个种类底下，经 admission.py library-files 读写
    （一趟一帧，服务回确认才算数）。load 取回键相同的那几趟；unavailable 是取回时 prune-server 不可用，problem 是别的取不回的那一句。storing 为假时不写。"""

    def __init__(self, verdicts, storing=True):
        self.verdicts, self.storing = verdicts, storing
        self.unavailable, self.problem = False, ""
        self.library = verdicts.library_name() if verdicts.ok else ""

    def relative(self, key):
        return f"trips/{key}.json"

    def store(self, name, trip):
        key = self.verdicts.keys.get(name)
        if not (self.storing and self.library and key):
            return
        directory = self.verdicts.path("trips-to-store")
        path = os.path.join(directory, "trips", f"{key}.json")
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(trip, handle, ensure_ascii=False)
        code, output = admission_run(self.verdicts.admission, ["library-files", "store", self.library, directory, self.relative(key)], path + ".out")
        if code != 0:
            print(f"  ! 开关趟落盘没成（{name}：admission.py library-files store 退 {code}，{last_lines(output)}），被打断的话下一趟这一趟照起", flush=True)

    def load(self, names):
        """names 里键相同、落了盘的那几趟：{名字: 那一趟的结局}。"""
        keyed = {name: self.verdicts.keys[name] for name in names if name in self.verdicts.keys}
        if not (self.library and keyed):
            return {}
        directory = self.verdicts.path("trips-fetched")
        code, output = admission_run(self.verdicts.admission, ["library-files", "fetch", self.library, directory, *[self.relative(key) for key in keyed.values()]],
                                     self.verdicts.path("trips-fetch.out"))
        if code != 0:
            self.unavailable = code == 75
            self.problem = f"admission.py library-files fetch 退 {code}（{last_lines(output)}）"
            return {}
        loaded = {}
        for name, key in keyed.items():
            path = os.path.join(directory, "trips", f"{key}.json")
            if not os.path.isfile(path):
                continue
            with open(path, encoding="utf-8") as handle:
                loaded[name] = json.load(handle)
        return loaded


class SampleStop:
    """样本档造「这一格中途被停」：被判的根是样本、STOP_AFTER_MARKER 在、STOP_ARMED_VARIABLE 设着（只有被打断的那一趟设）时，passed() 数到那份文件写的次数就当场退 137，
    不收尾、不记库。armed 的那一趟调用方只开一个线程，停在第几趟是定数。别的时候什么都不做。"""

    def __init__(self, sample, root, scratch):
        marker = os.path.join(root, STOP_AFTER_MARKER)
        self.limit, self.count, self.scratch, self.lock = 0, 0, scratch, threading.Lock()
        if sample and os.environ.get(STOP_ARMED_VARIABLE) and os.path.isfile(marker):
            with open(marker, encoding="utf-8") as handle:
                self.limit = int(handle.read().strip())

    @property
    def armed(self):
        return self.limit > 0

    def passed(self):
        if not self.armed:
            return
        with self.lock:
            self.count += 1
            reached = self.count >= self.limit
        if reached:
            print(f"  · 样本档：判完第 {self.count} 趟、落了盘，照 {STOP_AFTER_MARKER} 停掉这一趟", flush=True)
            shutil.rmtree(self.scratch, ignore_errors=True)
            os._exit(SAMPLE_STOP_EXIT)


def content_digest(path, skipped=("target", ".git")):
    """一份文件或一棵目录的摘要：文件取内容的 sha256；目录把每个子目录、每份文件（相对路径、执行位、内容）与每个链接（指向）按路径排好连起来取 sha256，
    skipped 里的子目录名（编译目录、git 库）不进；不在写「不在」。按条复用的几格拿它把被判那棵树里当数据读的输入折进键（line:）。"""
    if os.path.isfile(path) and not os.path.islink(path):
        with open(path, "rb") as handle:
            return "file:" + hashlib.sha256(handle.read()).hexdigest()
    if not os.path.isdir(path):
        return "link:" + os.readlink(path) if os.path.islink(path) else "不在"
    digest = hashlib.sha256()
    for directory, subdirectories, files in os.walk(path):
        subdirectories[:] = sorted(name for name in subdirectories if name not in skipped)
        digest.update(f"dir\0{os.path.relpath(directory, path)}\0".encode("utf-8", "surrogateescape"))
        linked = [name for name in subdirectories if os.path.islink(os.path.join(directory, name))]
        for name in sorted(files + linked):
            entry = os.path.join(directory, name)
            relative = os.path.relpath(entry, path)
            if os.path.islink(entry):
                digest.update(f"link\0{relative}\0{os.readlink(entry)}\0".encode("utf-8", "surrogateescape"))
            else:
                with open(entry, "rb") as handle:
                    content = hashlib.sha256(handle.read()).hexdigest()
                digest.update(f"file\0{relative}\0{'x' if os.access(entry, os.X_OK) else '-'}\0{content}\0".encode("utf-8", "surrogateescape"))
    return "tree:" + digest.hexdigest()


def toolchain_line():
    """rustc -vV 与 cargo -V 的输出连成一行（只问版本，不编）：编 unit-graph、算变异行的键都跟它走；起不来的那一样写「起不来」。"""
    parts = []
    for command in (["rustc", "-vV"], ["cargo", "-V"]):
        try:
            done = subprocess.run(command, capture_output=True, text=True, errors="replace", stdin=subprocess.DEVNULL)
            parts.append(" ".join(done.stdout.split()) if done.returncode == 0 else f"{command[0]} 退 {done.returncode}")
        except OSError:
            parts.append(f"{command[0]} 起不来")
    return " | ".join(parts)
PY_UNIT_VERDICT_ROUNDS
)"

# code_tooling_three_question_rounds <格名> <跑一趟的函数>：复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）。被判的根是样本、带 .unit-verdict-sample/three-questions/
#   （unrelated.sh：改一处不在任何判定闭包里的输入；change.sh：改闭包里的一处；affected：一行一条闭包含它的判定名）时，调它的那一格（两格开关复跑、reuse-selftests、
#   function-units-two-implementations-agree、mutation-list-items、entry-scripts）不照常跑一趟，改成在同一份判定库上跑四趟：
#   头一趟、同一份输入、跑 unrelated.sh 之后、跑 change.sh 之后；每趟照真跑那一条路算键、查库、起、记库，各条的结局（复用、现判、强制现判、拒算）写一份，
#   再交 lib_reuse_checks.py three-questions 判三问、打规范输出「REUSE_SELFTEST script=.claude/gate.d/code-tooling.sh …」（门禁 code-tooling 的 reuse-selftests 格经
#   verdict-library-scripts.tsv 那一行起这几份样本、认这一行）。四趟都判绿、三问都过退 0；有一趟没判绿、或三问有一问不过退 1。
code_tooling_three_question_rounds() {
  local cell="$1" pass="$2" marker=".unit-verdict-sample/three-questions" round rounds_scratch rounds_red=0 affected
  rounds_scratch="$(mktemp -d)" || { echo "  ✗ 建不了复用三问的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一份样本；一趟都没跑。"; return 1; }
  for round in first second unrelated unit; do
    if [[ "$round" == unrelated || "$round" == unit ]]; then
      local change_script="$marker/unrelated.sh"
      [[ "$round" == unit ]] && change_script="$marker/change.sh"
      # 写成「$ROOT/<样本里的相对路径>」（ROOT 就是此刻的目录）：收闭包时照「$<折不出的目录>/<折得出的尾巴>」认，不当仓根下不在的路径拒算
      if ! bash "$ROOT/$change_script"; then
        echo "  ✗ 样本的 $change_script 没跑成，复用三问判不了"
        echo "     → 怎么办：照它的报错改样本；它只许改样本根里的文件。"
        rm -rf -- "$rounds_scratch"
        return 1
      fi
    fi
    echo "── 复用三问：$round 那一趟（$cell）"
    if ! "$pass" "$rounds_scratch/$round.tsv"; then
      echo "  · 复用三问：$round 那一趟这一格没判绿（上面几行）"
      rounds_red=1
    fi
  done
  affected="$(sed '/^[[:space:]]*$/d' "$marker/affected" | paste -sd, -)"
  if ! python3 "$STAGE_REPOSITORY/scripts/gate/lib_reuse_checks.py" three-questions --script ".claude/gate.d/$STAGE_NAME" --first "$rounds_scratch/first.tsv" \
         --second "$rounds_scratch/second.tsv" --unrelated "$rounds_scratch/unrelated.tsv" --unit-change "$rounds_scratch/unit.tsv" --affected "$affected"; then
    echo "  ✗ $cell 的复用三问没过（红在哪一问写在上面 REUSE_SELFTEST 那一行与它上面几行）"
    echo "     → 怎么办：照 scripts/gate/lib_reuse_checks.py 文件头「一」看是哪一问：① 查库或跳过复用的那一段，② 键里多进了闭包外的输入，③ 键少进或多进了闭包里的输入"
    rounds_red=1
  fi
  rm -rf -- "$rounds_scratch"
  return "$rounds_red"
}

# code_tooling_interrupted_rounds <格名> <跑一趟的函数>：打断之后第二趟只补没判的（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）。
#   被判的根是样本、带 .unit-verdict-sample/interrupted-resume/stop-after（一个正整数）的，同一份判定库上跑三趟：第一趟判完那么多趟就当场退 137、不收尾（SampleStop），
#   第二趟同一条命令接着跑、第三趟全部复用。第一趟不是退 137 判红；后两趟这一格没判绿判红。现判几条、取回几趟写在样本的 expect 里。
code_tooling_interrupted_rounds() {
  local cell="$1" pass="$2" interrupted_exit resumed_red=0
  echo "── 打断与续跑：被打断的那一趟（$cell）"
  CODE_TOOLING_SAMPLE_STOP_ARMED=1 "$pass" ""; interrupted_exit=$?
  if ((interrupted_exit != 137)); then
    echo "  ✗ 样本档：被打断的那一趟退 $interrupted_exit（要 137：判完 stop-after 写的趟数就停），续跑判不了"
    echo "     → 怎么办：看上面几行：stop-after 写得比要跑的趟数多，就改小它；那一趟自己报了错，先修它再喂这份样本。"
    return 1
  fi
  echo "── 打断与续跑：第二趟（$cell）"
  "$pass" "" || resumed_red=1
  echo "── 打断与续跑：第三趟（$cell）"
  "$pass" "" || resumed_red=1
  return "$resumed_red"
}
