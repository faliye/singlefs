"""shell 的单元与调用边：ShellFile、ShellIndex（切函数、顶层变量、脚本体，认起的脚本与 source 的库，内联 python 与 here-doc），
仓内 shell 文件的枚举与 Rust 闭包读到的 shell 脚本。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
import copy
import os
import posixpath
import re
import shlex
import sys
from lib_function_units_lexer import active_breaks
from lib_function_units_base import (
    ASSIGNMENT_WORD, DEFAULT_SCRIPT_DIRECTORIES, FOLD_LIMIT, FOLD_LIMIT_FORM, INLINE_FORM, InlineScript, MESSAGE_HEADS, POSITIONAL, PYTHON_INTERPRETERS, Refusal,
    SHELL_INTERPRETERS, SHELL_OPEN_TAIL, SHELL_POSITIONAL_WORD, SKIPPED_DIRECTORIES, TEST_RUNNER_MODULES, Unit, argv_command_offsets,
    command_positions, descriptor_of, external_program_name, files_under, inside_root, interpreter_mode, sha256_bytes, sha256_text, snippet_of,
)
from lib_function_units_registry import launched_programs_registered, program_word_registered, shell_unquoted_word
from lib_function_units_shell_text import (
    COMMAND_POSITION_BEFORE_WORD, FUNCTION_HEADER, HEREDOC_BODY_PLACED, HEREDOC_START, SHELL_DECLARATIONS, SHELL_ENV_ARRAY_ASSIGNMENT,
    SHELL_KEYWORDS, SHELL_QUOTED_ASSIGNMENT, SHELL_ROW_SELECTED_TABLES, SHELL_TABLE_ASSIGNMENT, SHELL_TREE_PREFIX, SHELL_WORDS, SHELL_WRAPPER,
    ShellFacts, TOP_ASSIGNMENT, VARIABLE_REFERENCE, command_inputs, deferred_heredocs_moved_back, generated_scripts, git_repository_location,
    find_listing_pattern, heredoc_bodies, heredoc_mask, inline_positionals, kept_shell_comments, leading_command_words, literal_word, matching_close, narrowed_shell_body,
    raw_heredoc_rows, remove_shell_pattern, rows_inside_quotes, rows_of_selected_keys, shell_known_words, shell_launch_prefix, shell_normal_forms,
    shell_position_values, split_commands, substitutions, tree_copy_file,
)


# read 改写的变量记的那一处赋值（文件头「起代码的写法逐种认」read 那一条）：$(read) 折不出（fold_substitution 不认 read），
# 折这个变量时整个折不出，同一变量别处赋的字面值不替它
READ_UNFOLDED_WORD = '"$(read)"'
# -c 的串里 $(printf '%q' <词>) 那一处 <词> 折不出时换成的词（文件头「内联脚本里 printf %q 引过的词」）：照 bash 的规矩恰好是一个词、取值折不出
# （名字不赋值，fold_parameter 找不到赋值处交 None）；内联正文里它落在命令或脚本的位置上照折不出拒算，当参数、重定向的目标照不收
QUOTED_WORD_UNFOLDED = '"${function_units_quoted_word_unfolded}"'
# 换之前先在词里放的记号（前后各一个 NUL，中间是第几处）：fold_word 把它当普通字照抄，折完再逐处换成取值
QUOTED_WORD_MARK = "\x00{}\x00"
# read 的选项里带一个参数的那几个字母（-a 的参数是读进的数组名）
READ_OPTIONS_WITH_ARGUMENT = "adinNptu"
# 拒算落行（ShellIndex.word_line）里「解释器位置上是位置参数」那一层：剪过的正文（narrowed_shell_body）把 "$1" "${1:-}" 一类代成了字面的 bash / python3，
# 原文那一行上解释器的位置写的是位置参数，后面紧跟着那个词
POSITIONAL_INTERPRETER_BEFORE_WORD = r'(?:^|[\s;&|({])"?\$\{?[1-9][^\s"]*"?\s+'


def bounded_needle(needle, breaks):
    """拒算落行时找一个词的正则：词两头是字母、数字或 _ 的，前后不许紧挨着这几种字（「in」不认 rejoin 里那一截）；
    弄坏开关 shell-refusal-lines-by-substring 下照子串找。"""
    escaped = re.escape(needle)
    if "shell-refusal-lines-by-substring" in breaks or not needle:
        return escaped
    before = r"(?<![A-Za-z0-9_])" if re.match(r"\w", needle[0]) else ""
    after = r"(?![A-Za-z0-9_])" if re.match(r"\w", needle[-1]) else ""
    return before + escaped + after


def read_target_names(words):
    """read 命令改写的变量名（words 从 read 那个词起）：选项之后的每个名字、-a 的参数（数组名）；一个名字都没写的是 REPLY。
    不是字面的词跳过（认不出它是选项还是名字）；不像变量名的不算。"""
    names, index, options_done = [], 1, False
    while index < len(words):
        literal = literal_word(words[index])
        index += 1
        if literal is None:
            continue
        if not options_done and literal == "--":
            options_done = True
            continue
        if not options_done and literal.startswith("-") and len(literal) > 1:
            for position, letter in enumerate(literal[1:], 2):
                if letter in READ_OPTIONS_WITH_ARGUMENT:
                    argument = literal[position:]
                    if not argument and index < len(words):
                        argument = literal_word(words[index]) or ""
                        index += 1
                    if letter == "a" and argument:
                        names.append(argument)
                    break
            continue
        options_done = True
        names.append(literal)
    valid = [name for name in names if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name)]
    return valid or ([] if names else ["REPLY"])


class ShellFile:
    def __init__(self, path, content, normal, breaks):
        self.path, self.raw = path, content
        self.text = content.decode("utf-8", "surrogateescape")
        self.rows = self.text.split("\n")
        self.refusals, self.units = [], []
        self.functions = collections.defaultdict(list)
        self.variables = collections.defaultdict(list)
        self.script_body = self.kept = None
        self.breaks = breaks
        if normal is None:
            self.refusals.append(Refusal("bash-syntax", path, 1, snippet_of(self.rows[0] if self.rows else "")))
            return
        self.parse(normal)

    def parse(self, normal):
        lines = normal.split("\n")
        while lines and lines[-1] == "":
            lines.pop()
        if len(lines) < 3 or not lines[0].startswith(SHELL_WRAPPER) or lines[1].strip() != "{" or lines[-1] != "}":
            self.refusals.append(Refusal("bash-syntax", self.path, 1, snippet_of(self.rows[0] if self.rows else "")))
            return
        body = lines[2:-1]
        if "shell-deferred-heredocs-unmoved" not in self.breaks:
            body = deferred_heredocs_moved_back(body, self.rows)
        mask = set() if "shell-heredoc-split" in self.breaks else heredoc_mask(body)
        quoted = set() if "shell-function-bounds-quote-blind" in self.breaks else rows_inside_quotes(body, mask)
        functions = []
        for position, row in enumerate(body):
            if position in mask or position in quoted:
                continue
            found = FUNCTION_HEADER.match(row)
            if not found or position + 1 >= len(body) or body[position + 1] != found.group(1) + "{ ":
                continue
            indent = found.group(1)
            end = None
            for later in range(position + 2, len(body)):
                if later in mask or later in quoted:
                    continue
                candidate = body[later]
                if candidate.startswith(indent + "}") and (len(candidate) == len(indent) + 1 or candidate[len(indent) + 1] in " ;|&<>)"):
                    end = later
                    break
            if end is not None:
                functions.append((position, end, len(indent), found.group(2)))
        functions.sort()
        parents, stack, by_range = {}, [], {}
        for function in functions:
            while stack and stack[-1][1] < function[0]:
                stack.pop()
            parents[function] = stack[-1] if stack else None
            stack.append(function)
        cursor = 0
        for function in functions:
            start, end, _indent, name = function
            parent = by_range.get(parents[function])
            qualified = name if parent is None else f"{parent.name}::{name}"
            text = "\n".join(body[start:end + 1])
            first_line, last_line, cursor = self.original_lines(name, cursor)
            unit = Unit("shell", "function", qualified, self.path, first_line, last_line, sha256_text(text))
            # 指纹照含套着的函数的全文；正文（facts 切命令、认词用的）只留这一层：套在里面的函数那几行抹空（行数不变），它们的命令只归它们自己；
            # 全文另留在 whole_body（赋值、export 与写出的脚本照全文认：套着的函数与它的兄弟函数照旧经 container 看得见，ShellIndex.facts）
            nested_rows = set() if "shell-nested-commands-in-outer" in self.breaks else {
                row for inner_start, inner_end, _inner_indent, _inner_name in functions if start < inner_start and inner_end <= end
                for row in range(inner_start, inner_end + 1)}
            unit.short, unit.container = name, parent
            if nested_rows:
                unit.body = "\n".join("" if position in nested_rows else body[position] for position in range(start, end + 1))
                unit.whole_body = text
            else:
                unit.body = text
            by_range[function] = unit
            self.functions[name].append(unit)
            self.units.append(unit)
        covered = set()
        for function in functions:
            if function[2] == 4:
                covered |= set(range(function[0], function[1] + 1))
        body_text = "\n".join(row for position, row in enumerate(body) if position not in covered)
        fingerprint = sha256_bytes(self.raw) if "shell-raw-text" in self.breaks else sha256_text(body_text)
        self.script_body = Unit("shell", "script-body", "<script-body>", self.path, 1, max(len(self.rows), 1), fingerprint)
        self.script_body.body = body_text
        self.units.append(self.script_body)
        cursors = collections.defaultdict(int)
        position = 0
        while position < len(body):
            found = TOP_ASSIGNMENT.match(body[position]) if position not in covered and position not in mask else None
            if not found:
                position += 1
                continue
            end = position
            while end + 1 < len(body) and (end in mask or not body[end].rstrip().endswith(";")) and "shell-variable-first-line-only" not in self.breaks:
                end += 1
            if any(index in covered for index in range(position, end + 1)):
                end = position
            text = "\n".join(body[position:end + 1]).strip()
            name = found.group(1)
            pattern = re.compile(rf"^(?:(?:export|readonly|declare|typeset)(?:\s+-\w+)*\s+)?{re.escape(name)}(?:\[[^\]]*\])?\+?=")
            row_index = next((number for number in range(cursors[name], len(self.rows)) if pattern.match(self.rows[number])), None)
            if row_index is None:
                row_index = max(cursors[name] - 1, 0)
            cursors[name] = row_index + 1
            unit = Unit("shell", "shell-variable", name, self.path, row_index + 1, min(row_index + 1 + end - position, len(self.rows)), sha256_text(text))
            unit.body = text.rstrip(";")
            self.variables[name].append(unit)
            self.units.append(unit)
            position = end + 1
        kept = kept_shell_comments(self.text)
        if kept:
            self.kept = Unit("shell", "kept-comments", "<kept-comments>", self.path, 1, max(len(self.rows), 1), sha256_text("\n".join(kept)), data=True)
            self.units.append(self.kept)

    def original_lines(self, name, cursor):
        header = re.compile(rf"(?:^|[;&|{{(]\s*)\s*((?:function\s+{re.escape(name)}(?:\s*\(\s*\))?|{re.escape(name)}\s*\(\s*\))\s*(?:\{{|\(|$|#))")
        found = next((position for position in range(cursor, len(self.rows)) if header.search(self.rows[position])), None)
        if found is None:
            found = next((position for position in range(0, len(self.rows)) if header.search(self.rows[position])), None)
        if found is None:
            return 1, 1, cursor
        column = header.search(self.rows[found]).start(1)
        return found + 1, shell_block_end(self.rows, found, column) + 1, found + 1


def self_assigning_word(name, word):
    """赋值词是不是把变量 name 赋成它自己（"$NAME"、$NAME、"${NAME}"、${NAME}）：这种赋值不带新的值（ShellIndex.assignment_sites 不把它算成赋过）。"""
    escaped = re.escape(name)
    return re.fullmatch(rf'"?\$(?:\{{{escaped}\}}|{escaped})"?', word) is not None


def shell_block_end(rows, start, column=0):
    """原文里 rows[start] 第 column 列起的函数定义到哪一行收尾（只当索引；认不出回起头那一行）。逐行扫，带一个语境栈：
    单引号、双引号、$( … ) 可以跨行、可以互相套，here-doc（含 "$(… <<'词'" 这种写在命令替换里的）的正文整段跳过。"""
    header = re.compile(r"\s*(?:function\s+)?[^\s(]+\s*(?:\(\s*\))?\s*").match(rows[start], column)
    row_index, column = start, header.end() if header else column
    while row_index < len(rows):
        rest = rows[row_index][column:]
        if rest.strip():
            column += len(rest) - len(rest.lstrip())
            break
        row_index, column = row_index + 1, 0
    if row_index >= len(rows) or rows[row_index][column] not in "{(":
        return start
    opener = rows[row_index][column]
    depth = 0
    stack = []          # 语境："'"、'"'、"$(" 各一层；空的时候在函数体本层
    parens = [0]        # 每一层 $( 里还开着几个 (
    while row_index < len(rows):
        row, i, heredocs = rows[row_index], column, []
        while i < len(row):
            character = row[i]
            top = stack[-1] if stack else None
            if top == "'":
                if character == "'":
                    stack.pop()
                i += 1
                continue
            if top == '"':
                if character == "\\":
                    i += 2
                    continue
                if character == '"':
                    stack.pop()
                elif row.startswith("$(", i):
                    stack.append("$(")
                    parens.append(0)
                    i += 2
                    continue
                i += 1
                continue
            if character == "\\":
                i += 2
                continue
            if character in ("'", '"'):
                stack.append(character)
                i += 1
                continue
            if character == "#" and (i == 0 or row[i - 1] in " \t;"):
                break
            if row.startswith("<<", i) and not row.startswith("<<<", i):
                found = HEREDOC_START.match(row, i)
                if found:
                    heredocs.append((found.group(3), found.group(1) == "-"))
                    i = found.end()
                    continue
            if row.startswith("${", i):
                i = matching_close(row, i + 1, "{", "}") + 1
                continue
            if row.startswith("$(", i):
                stack.append("$(")
                parens.append(0)
                i += 2
                continue
            if top == "$(":
                if character == "(":
                    parens[-1] += 1
                elif character == ")":
                    if parens[-1] == 0:
                        stack.pop()
                        parens.pop()
                    else:
                        parens[-1] -= 1
                i += 1
                continue
            if opener == "{":
                before = i == 0 or row[i - 1] in " \t;&|()"
                after = i + 1 >= len(row) or row[i + 1] in " \t;)|&<>"
                if character == "{" and before and after:
                    depth += 1
                elif character == "}" and before and after:
                    depth -= 1
                    if depth == 0:
                        return row_index
            elif character == "(":
                depth += 1
            elif character == ")":
                depth -= 1
                if depth == 0:
                    return row_index
            i += 1
        row_index, column = row_index + 1, 0
        for delimiter, strip_tabs in heredocs:
            while row_index < len(rows) and (rows[row_index].lstrip("\t") if strip_tabs else rows[row_index]) != delimiter:
                row_index += 1
            row_index += 1
    return start


class ShellIndex:
    def __init__(self, root, paths):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.files = {}
        self.sources_of, self.sourcers_of = {}, collections.defaultdict(set)
        self.folding = set()
        # 正在折的内联 bash 串里自己赋的变量（一层一份，里层先看）；$(cat <<'词' … 词) 那段命令替换的输出（here-doc 正文）
        self.overlays, self.substitution_outputs = [], collections.defaultdict(set)
        # 折出的取值超过 FOLD_LIMIT 种的次数（fold_word、function_printed_word、fold_heredoc 加一，只增不减）：起脚本、起命令、内联脚本那几处比前后两次的数，
        # 认「折得出、只是太多」，拒算 FOLD_LIMIT_FORM、不照根里同一相对路径那一份猜（文件头「拒算」）
        self.fold_limit_hits = 0
        self.add(paths)

    def add(self, paths):
        new = [path for path in dict.fromkeys(paths) if path not in self.files and os.path.isfile(os.path.join(self.root, path))]
        if not new:
            return
        contents = []
        for path in new:
            with open(os.path.join(self.root, path), "rb") as handle:
                contents.append(handle.read())
        forms = shell_normal_forms(contents)
        for path, content, form in zip(new, contents, forms):
            self.files[path] = ShellFile(path, content, form, self.breaks)
        self.compute_groups()

    def compute_groups(self):
        """source 边（有向：谁 source 了谁）。折叠路径要用变量，变量又可能赋在被 source 的那份里，所以来回算到不再变（至多三趟）。"""
        self.sources_of = {path: frozenset() for path in self.files}
        for _round in range(3):
            edges = collections.defaultdict(set)
            for path, shell_file in self.files.items():
                for unit in shell_file.units:
                    if unit.data:
                        continue
                    for kind, targets, _refusal in self.links(unit):
                        if kind == "source":
                            edges[path].update(target for target in targets if target in self.files and target != path)
            sources = {path: frozenset(edges.get(path, ())) for path in self.files}
            if sources == self.sources_of:
                break
            self.sources_of = sources
            # 反向的边一起换：库里的单元往上找 source 它的那几份（upward、$0）要的是这一趟的边，下一趟重算的 links 才不沿用上一趟的
            self.sourcers_of = self.reversed_sources()
            for shell_file in self.files.values():
                for unit in shell_file.units:
                    if unit.cache is not None:
                        unit.cache.pop("links", None)
        self.sourcers_of = self.reversed_sources()

    def reversed_sources(self):
        """sources_of 反过来：被 source 的文件 → source 了它的那几份。"""
        sourcers = collections.defaultdict(set)
        for path, targets in self.sources_of.items():
            for target in targets:
                sourcers[target].add(path)
        return sourcers

    def downward(self, paths):
        """这几份与它们（传递地）source 进来的全部文件。"""
        seen, todo = set(), list(paths)
        while todo:
            path = todo.pop()
            if path in seen:
                continue
            seen.add(path)
            todo += list(self.sources_of.get(path, ()))
        return seen

    def siblings(self, path):
        """和这一份一起被 source 进同一份脚本的那几份（拆成入口加几份库：库与库在 bash 里共用一个全局命名空间，一份库读另一份库赋的变量、调它的函数），
        不含它自己、它（传递地）source 的与 source 了它的。弄坏开关 shell-sourced-siblings-unread 下交空。"""
        if "shell-sourced-siblings-unread" in self.breaks:
            return set()
        above = self.upward(path)
        return self.downward(above) - self.downward([path]) - above if above else set()

    def upward(self, path):
        """（传递地）source 了这一份的全部文件，不含它自己。"""
        seen, todo = set(), list(self.sourcers_of.get(path, ()))
        while todo:
            item = todo.pop()
            if item in seen or item == path:
                continue
            seen.add(item)
            todo += list(self.sourcers_of.get(item, ()))
        return seen

    def facts(self, unit):
        """单元读到的东西：照 Unit.body 切命令、认词；套着别的函数的外层函数（ShellFile.parse 留了 whole_body）赋值、local、export 与写出的脚本照全文另认一遍
        （文件头「shell 里套在函数里的函数」外层只认自己那一层）。"""
        if unit.facts is not None:
            return unit.facts
        facts = self.text_facts(unit.body or "")
        if unit.whole_body is not None:
            whole = self.text_facts(unit.whole_body)
            facts.assignments, facts.local_names, facts.exported, facts.generated = whole.assignments, whole.local_names, whole.exported, whole.generated
            facts.prefixed = whole.prefixed
        unit.facts = facts
        return facts

    def text_facts(self, text):
        """一段单元正文 → ShellFacts：切命令（here-doc 正文跳过）、认写出的脚本、词、变量引用，逐条命令与命令替换里的命令记进去。"""
        facts = ShellFacts()
        rows = text.split("\n")
        mask = set() if "shell-heredoc-split" in self.breaks else heredoc_mask(rows)
        visible = "\n".join("" if position in mask else row for position, row in enumerate(rows))
        redirects = []
        details = []
        facts.commands = split_commands(visible, "shell-redirect-targets-loaded" in self.breaks, redirects, details)
        facts.generated = {} if "shell-generated-scripts-ignored" in self.breaks else generated_scripts(facts.commands, redirects, rows, mask)
        facts.words = set(SHELL_WORDS.findall(text))
        facts.variable_references = set(VARIABLE_REFERENCE.findall(text))
        bodies = heredoc_bodies(rows) if mask else {}
        self.scan_commands(facts.commands, details, 0, visible, bodies, facts)
        self.scan_substitutions(visible, 0, visible, bodies, facts, 0)
        return facts

    def scan_commands(self, commands, details, base, visible, bodies, facts):
        inputs = command_inputs(details, base, visible, bodies)
        for index, command in enumerate(commands):
            self.command_facts(command, facts, inputs.get(index, ()))

    def scan_substitutions(self, fragment, base, visible, bodies, facts, depth):
        """命令替换 $( … ) 与 `…` 里的命令照样是要跑的命令（x="$(bash y.sh)"、if out="$(python3 - <<'PY' …)"）：逐层切开再认。"""
        if "shell-substitutions-ignored" in self.breaks or depth > 8:
            return
        for inner, offset in substitutions(fragment):
            details = []
            commands = split_commands(inner, "shell-redirect-targets-loaded" in self.breaks, None, details)
            self.scan_commands(commands, details, base + offset, visible, bodies, facts)
            if len(commands) == 1 and commands[0] and literal_word(commands[0][0]) == "cat" and len(commands[0]) == 1:
                heredocs = command_inputs(details, base + offset, visible, bodies).get(0, [])
                if len(heredocs) == 1 and heredocs[0][:2] == (0, "heredoc") and heredocs[0][2] is not None:
                    self.substitution_outputs[inner].add(heredocs[0][2])
            self.scan_substitutions(inner, base + offset, visible, bodies, facts, depth + 1)

    def scan_text(self, text, facts):
        """一段独立的 shell 正文（bash -c 的串、喂给 bash 的 here-doc）：切命令、认 here-doc 与命令替换，记进 facts。"""
        rows = text.split("\n")
        mask = set() if "shell-heredoc-split" in self.breaks else heredoc_mask(rows)
        visible = "\n".join("" if position in mask else row for position, row in enumerate(rows))
        details = []
        commands = split_commands(visible, "shell-redirect-targets-loaded" in self.breaks, None, details)
        bodies = heredoc_bodies(rows) if mask else {}
        self.scan_commands(commands, details, 0, visible, bodies, facts)
        self.scan_substitutions(visible, 0, visible, bodies, facts, 0)

    def command_facts(self, command, facts, inputs=()):
        words = list(command)
        looped = bool(words) and words[0] in ("while", "until")
        while words and words[0] in SHELL_KEYWORDS:
            words = words[1:]
        if not words:
            return
        if words[0] == "for" and len(words) >= 3 and words[2] == "in":
            for value in words[3:]:
                facts.assignments[words[1]].append(value)
            return
        if words[0] == "case" and "shell-case-subjects-launched" not in self.breaks:
            # case <词> in：<词> 是拿来比模式的值，不是要起的命令（分支的模式 split_commands 已跳过，分支里的命令各是一条）；
            # 照剪过的正文代进参数之后会是字面的 case bash in，不在这里收尾就把 in 当成 bash 起的脚本。弄坏开关 shell-case-subjects-launched 下照命令认
            return
        declaration = words[0] in SHELL_DECLARATIONS
        scoped = words[0] == "local" or (words[0] in ("declare", "typeset") and "-g" not in words[1:])
        if words[0] == "export":
            facts.exported.update(word.split("=", 1)[0] for word in words[1:] if re.match(r"^[A-Za-z_][A-Za-z0-9_]*(?:=|$)", word))
        start = 1 if declaration else 0
        position = start
        while position < len(words):
            found = ASSIGNMENT_WORD.match(words[position])
            if found or (scoped and re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", words[position])):
                name = found.group(1) if found else words[position]
                if found:
                    facts.assignments[name].append(found.group(2))
                if scoped:
                    facts.local_names.add(name)
                position += 1
                continue
            if declaration and words[position].startswith("-"):
                position += 1
                continue
            break
        if declaration:
            return
        prefix = [(found.group(1), found.group(2)) for found in (ASSIGNMENT_WORD.match(item) for item in words[:position]) if found]
        words = words[position:]
        if not words:
            return
        first = literal_word(words[0])
        for name, value in prefix:
            # X=v <命令>：只管这一条命令（是函数就只管这一次调用），ShellIndex.reader_scoped 照它认
            facts.prefixed[name].append((value, first))
        facts.command_words.add(first if first is not None else words[0])
        if first == "read":
            from_heredoc = (not looped and "shell-read-heredoc-assignments-ignored" not in self.breaks
                            and self.read_heredoc_assignment(words, prefix, inputs, facts))
            if not from_heredoc and "shell-read-assignments-unrecorded" not in self.breaks:
                # read 改写的变量值来自输入（文件头「起代码的写法逐种认」read 那一条）：记一处折不出的赋值，别处赋的字面值不替它
                for name in read_target_names(words):
                    facts.assignments[name].append(READ_UNFOLDED_WORD)
        if first == "cd":
            arguments = [word for word in words[1:] if not (literal_word(word) or "").startswith("-")]
            facts.directory = arguments[0] if len(arguments) == 1 else None
            return
        if first not in MESSAGE_HEADS or "external-program-words-unfiltered" in self.breaks:
            for word in words:
                program = external_program_name(literal_word(word), self.breaks)
                if program is not None:
                    facts.programs.add(program)
        if first in (("source",) if "shell-dot-source-unrecognized" in self.breaks else ("source", ".")) and len(words) > 1:
            facts.invocations.append(("source", words[1], facts.directory, None))
            return
        if self.interpreter_runs(words, facts, inputs, prefix):
            return
        values = shell_position_values(words)
        positions = [0] if "shell-wrapped-commands-ignored" in self.breaks else command_positions(values)
        head = positions[-1]
        if head >= len(words):
            return
        raw = words[head]
        value = values[head]
        unquoted = raw.replace('"', "").replace("'", "")
        if unquoted.endswith((".sh", ".py", ".bash")) or (value is not None and "/" in value and not value.startswith("/usr") and not value.startswith("/bin")):
            facts.invocations.append(("exec", raw, facts.directory, None))
        elif value is None and "shell-unfolded-commands-allowed" not in self.breaks:
            # 命令位置上折不出的程序：到 links 里再折，折出解释器照解释器认、折出仓里的脚本连上、折不出拒算（"$@" 一类除外：命令就是调用处那几个词）
            facts.invocations.append(("command", raw, facts.directory, (words[head + 1:], tuple(inputs))))
        if "shell-path-arguments-loaded" in self.breaks:
            # 弄坏开关：把参数位置上长得像脚本的路径也当装代码（命令位置之外的路径是传给那条命令的数据，不装代码）
            for argument in words[1:]:
                if argument.replace('"', "").replace("'", "").endswith((".sh", ".py", ".bash")):
                    facts.invocations.append(("exec", argument, facts.directory, None))

    def read_heredoc_assignment(self, words, prefix, inputs, facts):
        """IFS= read -r -d '' <变量> <<词 … 词（文件头「起代码的写法逐种认」read 那一条）：命令前写了 IFS 的空值、选项只有 -r 与 -d ''、只读一个变量、
        输入只有这一条命令自己的一份 here-doc → 变量的取值是 here-doc 的内容（正文加末尾一个换行：-d '' 读到头，IFS 是空的不去首尾的空白），
        带引号的词原样、不带引号的照 bash 展开（fold_substitution 照 $(cat <<词 …) 的正文折）。这一处赋值记成 $( <记号> ) 一个词，
        记号只指 substitution_outputs 里放的那份内容 → True；别的写法不记 → False（command_facts 改记 READ_UNFOLDED_WORD）。"""
        if [literal_word(value) for name, value in prefix if name == "IFS"] != [""]:
            return
        raw, delimiter, awaiting, names = False, None, False, []
        for word in words[1:]:
            literal = literal_word(word)
            if literal is None:
                return
            if awaiting:
                delimiter, awaiting = literal, False
            elif literal.startswith("-") and len(literal) > 1 and not names:
                for position, letter in enumerate(literal[1:], 2):
                    if letter == "r":
                        raw = True
                    elif letter == "d":
                        if literal[position:]:
                            delimiter = literal[position:]
                        else:
                            awaiting = True
                        break
                    else:
                        return
            else:
                names.append(literal)
        heredocs = [content for number, kind, content in inputs if number == 0 and kind == "heredoc"]
        if not raw or delimiter != "" or awaiting or len(names) != 1 or not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", names[0]) \
                or len(inputs) != 1 or len(heredocs) != 1 or heredocs[0] is None:
            return
        quoted, body = heredocs[0]
        content = body + "\n"
        marker = f"read-heredoc {sha256_text(('quoted ' if quoted else 'expanded ') + content)}"
        self.substitution_outputs[marker].add((quoted, content))
        facts.assignments[names[0]].append(f'"$({marker})"')
        return True

    def interpreter_runs(self, words, facts, inputs, prefix=()):
        """这条命令里每一个解释器词（bash / sh / python3 / python，在哪个位置都算：包装命令 env、sudo、xargs、capped.sh、
        run-with-memory-cap.sh 后面的照样认）各按它的写法记一条要装的代码；→ 认到了几个解释器词（0 交给直接执行那一条）。
        脚本之后接着往后认（bash capped.sh N python3 x.py 里的 python3 x.py）；-c 的串、标准输入喂的脚本之后的词是它的参数，不再往后认。"""
        values = shell_position_values(words)
        positions = command_positions(values)
        found, position = 0, 0
        while position < len(words):
            value = values[position]
            base = posixpath.basename(value) if value else None
            if base not in SHELL_INTERPRETERS and base not in PYTHON_INTERPRETERS:
                position += 1
                continue
            found += 1
            python = base in PYTHON_INTERPRETERS
            language = "python" if python else "shell"
            mode, argument = interpreter_mode(values, position, python)
            facts.handled.append((language, mode, position))
            # 同一条命令里设给它的环境变量：命令前的 X=…（prefix）与 env 后面的 X=…
            environment = tuple(prefix) + tuple((found_word.group(1), found_word.group(2)) for found_word in
                                                (ASSIGNMENT_WORD.match(item) for item in words[:position]) if found_word)
            if mode == "script":
                descriptor = descriptor_of(values[argument])
                if descriptor is not None:
                    self.stdin_script(language, words[position], words[argument + 1:], inputs, descriptor, facts, environment)
                    return found
                facts.invocations.append((language, words[argument], facts.directory, (words[argument + 1:],)))
                if "shell-later-interpreters-ignored" in self.breaks:
                    return found
                position = argument + 1
                continue
            if mode == "inline":
                if argument + 1 < len(words):
                    # bash -c '串' $0 $1 …；python3 -c '串' 之后的词是 sys.argv[1:]（sys.argv[0] 是 -c）
                    rest = (["-c"] if python else []) + words[argument + 2:]
                    self.inline_run(language, words[position], words[argument + 1], rest, facts, environment)
                return found
            if mode == "module":
                if argument + 1 < len(words) and "python-modules-ignored" not in self.breaks:
                    facts.invocations.append(("python-module", words[argument + 1], facts.directory, (words[position], words[argument + 2:])))
                if "shell-later-interpreters-ignored" in self.breaks:
                    return found
                position = argument + 2
                continue
            if mode == "stdin":
                self.stdin_script(language, words[position], words[argument + 1:], inputs, 0, facts, environment)
                return found
            if mode == "bare" and position in positions:
                # 命令位置上的解释器后面没有脚本：脚本从标准输入来（here-doc、<<<、< 文件；都没有就是管道，拒算）
                self.stdin_script(language, words[position], [], inputs, 0, facts, environment)
            # syntax-only（bash -n 只查语法）与参数位置上光一个解释器名（command -v python3、which bash）：不装代码
            return found
        return found

    def stdin_script(self, language, interpreter, argv, inputs, descriptor, facts, environment=()):
        """解释器从文件描述符 descriptor（0 是标准输入）读脚本：同一条命令里给这个描述符的 here-doc、here-string、< 文件就是脚本；
        找不到（管道喂的、外面给的）拒算。argv 是脚本之后的那几个词（$1… / sys.argv[1:]）。"""
        source = None
        for number, form, payload in inputs:
            if number == descriptor:
                source = (form, payload)
        if "inline-scripts-ignored" in self.breaks:
            return
        if source is None:
            facts.invocations.append(("inline-unknown", interpreter, facts.directory, (language,)))
        elif source[0] == "file":
            facts.invocations.append((language, source[1], facts.directory, None))
        elif source[0] == "herestring":
            self.inline_run(language, interpreter, source[1], ["-"] + list(argv), facts, environment)
        elif source[1] is None:
            facts.invocations.append(("inline-unknown", interpreter, facts.directory, (language,)))
        else:
            quoted, body = source[1]
            payload = ("text", body) if quoted else ("heredoc", body)
            facts.invocations.append((f"{language}-inline", interpreter, facts.directory, (payload, ["-"] + list(argv), tuple(environment))))

    def inline_run(self, language, interpreter, word, rest, facts, environment=()):
        """-c 的串（rest 是串之后的词：$0、$1…；python 的 sys.argv[1:] 是 rest[1:]）。字面的 bash 串当场切开再认（串里的赋值算进这个单元）；
        python 的串与带展开的串到 links 里再折。"""
        if "inline-scripts-ignored" in self.breaks:
            if language == "python":
                return
        inline = literal_word(word)
        if language == "shell" and inline is not None:
            if "shell-inline-positionals-ignored" not in self.breaks:
                inline = inline_positionals(inline, rest)
            self.scan_text(inline, facts)
            return
        if "inline-scripts-ignored" in self.breaks:
            return
        payload = ("text", inline) if inline is not None else ("word", word)
        facts.invocations.append((f"{language}-inline", interpreter, facts.directory, (payload, list(rest), tuple(environment))))

    def assignment_sites(self, name, unit):
        """一个变量的赋值词连同在哪个单元里折它 → [(赋值词, 单元)]：这个单元自己赋过的只看自己的（local 的作用域）；没有就取它那份文件里全部单元的
        （别的函数里 local 的不算），还没有就取它（传递地）source 进来的那几份的，再没有取 source 了它的那几份的，再没有取和它一起被 source 的那几份的（siblings）。
        赋值写在别的文件里的（拆成入口加被 source 的库：入口赋 STAGE_REPOSITORY、库里的函数读它），那个词在赋值所在的单元里折：
        词里的 $0、BASH_SOURCE、它再读的变量照那份文件认（文件头「shell 里被 source 的库读别的文件里赋的变量」）；同一份文件里的照旧在读它的这个单元里折。
        弄坏开关 shell-sourced-values-folded-in-reader 下别的文件里的赋值也在读它的这个单元里折。"""
        for overlay in reversed(self.overlays):
            if overlay.get(name):
                return [(word, unit) for word in overlay[name]]
        if unit is None:
            return []
        counted = "shell-self-assignments-counted" in self.breaks

        def carrying(owner):
            # owner 那个单元里的赋值词，去掉只管别的调用的命令前赋值（reader_scoped）；
            # NAME="$NAME" cmd（命令前把变量原样交给那条命令的环境）、local NAME="$NAME" 这类赋成它自己的词不带新值：不算这一层赋过，接着往外找
            # （文件头「shell 里被 source 的库读别的文件里赋的变量」）；弄坏开关 shell-self-assignments-counted 下照赋值算（盖住外面真赋的那一处，折成空）
            words = self.reader_scoped(name, owner, self.facts(owner).assignments.get(name), unit)
            return list(words or ()) if counted else [word for word in words or () if not self_assigning_word(name, word)]

        own = carrying(unit) if unit.kind == "function" else None
        if own:
            return [(word, unit) for word in own]
        container = unit.container if "shell-enclosing-locals-ignored" not in self.breaks else None
        while container is not None:
            # 套在外层函数里的函数：外层的 local 它看得见（bash 按调用链找变量，套着定义的只在外层里调）
            enclosing = carrying(container)
            if enclosing:
                return [(word, unit) for word in enclosing]
            container = container.container
        in_reader = "shell-sourced-values-folded-in-reader" in self.breaks

        def gathered(paths):
            found = []
            for path in sorted(paths):
                for other in self.files[path].units:
                    if other.data or other.kind == "shell-variable":
                        continue
                    other_facts = self.facts(other)
                    if other.kind == "function" and name in other_facts.local_names:
                        continue
                    holder = unit if in_reader or other.file == unit.file else other
                    found += [(word, holder) for word in carrying(other)]
            return found

        for paths in ([unit.file], self.downward([unit.file]) - {unit.file}, self.upward(unit.file), self.siblings(unit.file)):
            found = gathered([path for path in paths if path in self.files])
            if found:
                return found
        return []

    def family_paths(self, path):
        """path 与它 source 的、source 了它的、和它一起被 source 的那几份（在索引里的，path 排头）：函数按名字在这一族里找（同 function_printed_word）。"""
        return [path] + [other for other in sorted((self.downward([path]) | self.upward(path) | self.siblings(path)) - {path}) if other in self.files]

    def function_reaches(self, head, reader, paths):
        """从函数 head 起、照每个函数正文里命令位置上的函数名（facts.command_words）往下走，走得到 reader 这个函数单元（短名相同）→ True。
        reader 不是函数（脚本体）的交 False：命令前的赋值只在那一次调用里，调完就没了。"""
        if reader.kind != "function":
            return False
        memo = self.__dict__.setdefault("function_reach_memo", {})
        slot = (head, tuple(paths))
        if slot not in memo:
            names = self.function_names_in(paths)
            todo, seen = [head], {head}
            while todo:
                current = todo.pop()
                for function in self.functions_in(paths, current):
                    for word in sorted(self.facts(function).command_words):
                        if word in names and word not in seen:
                            seen.add(word)
                            todo.append(word)
            memo[slot] = frozenset(seen)
        return reader.short in memo[slot]

    def temporary_word(self, word, holder, depth=0):
        """赋值词 word（在 holder 那个单元里）是这一趟自己的临时目录底下的路径：开头的 $名字 每一处赋值都是 "$(mktemp …)"，或又是这样的词（至多 4 层）。"""
        found = re.match(r'^"?\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?(?:/|"?$)', word)
        progress = self.__dict__.setdefault("temporary_progress", set())
        marker = (found.group(1) if found else None, id(holder))
        if found is None or depth > 4 or marker in progress:
            # 正在判的又问到自己（X="$X/子目录" f 一类经 assignment_sites → reader_scoped 绕回来）：当不是
            return False
        progress.add(marker)
        try:
            sites = self.assignment_sites(found.group(1), holder)
            return bool(sites) and all(re.match(r'^"?\$\(\s*mktemp\b', value) or self.temporary_word(value, site, depth + 1) for value, site in sites)
        finally:
            progress.discard(marker)

    def reader_scoped(self, name, owner, words, reader):
        """owner 单元里 name 的赋值词 words（facts.assignments 记的）去掉命令前赋值里管不到 reader 的那几处（文件头「shell 命令前的赋值只管那一次调用」）：
        X=v <函数> …（facts.prefixed）的头一个词是这一族文件里的函数，而从它走不到读它的 reader（function_reaches）的去掉；
        走得到、而赋的是这一趟自己的临时目录底下的路径（temporary_word：自证里换成临时目录下的假脚本那一类，起的是这一份自己写出来的东西）的也去掉。
        头一个词不是函数（起外部程序、起脚本）的照留。弄坏开关 shell-prefix-assignments-unscoped 下前一种照留，shell-temporary-sites-folded 下后一种照留。"""
        prefixed = self.facts(owner).prefixed.get(name) if words else None
        if not prefixed:
            return words
        paths = self.family_paths(owner.file)
        names = self.function_names_in(paths)
        kept = list(words)
        for value, head in prefixed:
            if head is None or head not in names or value not in kept:
                continue
            unreached = "shell-prefix-assignments-unscoped" not in self.breaks and not self.function_reaches(head, reader, paths)
            temporary = not unreached and "shell-temporary-sites-folded" not in self.breaks and self.temporary_word(value, owner)
            if unreached or temporary:
                kept.remove(value)
        return kept

    def assignments_of(self, name, unit):
        """一个变量的赋值词（assignment_sites 去掉在哪个单元里折）。"""
        return [word for word, _holder in self.assignment_sites(name, unit)]

    def started_scripts(self, unit):
        """$0：起头的那份脚本（相对根拼成绝对路径的集合）。被 source 的库里的 $0 是 source 它的那条链最上头的脚本（自己不被别人 source 的那几份），
        库自己不算；没人 source 的就是它自己（文件头「shell 里被 source 的库读别的文件里赋的变量」）。弄坏开关 shell-sourced-zero-own-file 下一律是它自己。"""
        own = {os.path.join(self.root, unit.file)}
        if "shell-sourced-zero-own-file" in self.breaks:
            return own
        tops = {path for path in self.upward(unit.file) if not self.sourcers_of.get(path)}
        return {os.path.join(self.root, path) for path in sorted(tops)} or own

    def fold_word(self, word, unit, progress):
        results = {""}
        i, total, quoted = 0, len(word), False

        def extend(values):
            nonlocal results
            results = {result + value for result in results for value in values}

        while i < total:
            character = word[i]
            if character == '"':
                quoted = not quoted
                i += 1
                continue
            if character == "'" and not quoted:
                end = word.find("'", i + 1)
                if end < 0:
                    return None
                extend({word[i + 1:end]})
                i = end + 1
                continue
            if character == "\\" and i + 1 < total:
                if quoted and word[i + 1] not in '$`"\\\n':
                    extend({character})
                    i += 1
                    continue
                extend({word[i + 1]})
                i += 2
                continue
            if character == "$":
                if word.startswith("$((", i):
                    return None
                if word.startswith("$(", i):
                    end = matching_close(word, i + 1, "(", ")")
                    values = self.fold_substitution(word[i + 2:end], unit, progress)
                    i = end + 1
                elif word.startswith("${", i):
                    end = matching_close(word, i + 1, "{", "}")
                    values = self.fold_parameter(word[i + 2:end], unit, progress)
                    i = end + 1
                else:
                    found = re.match(r"\$([A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-])", word[i:])
                    if not found:
                        extend({"$"})
                        i += 1
                        continue
                    values = self.fold_parameter(found.group(1), unit, progress)
                    i += found.end()
                if values is None:
                    return None
                extend(values)
                if len(results) > FOLD_LIMIT:
                    self.fold_limit_hits += 1
                    return None
                continue
            if character == "`":
                end = word.find("`", i + 1)
                if end < 0:
                    return None
                values = self.fold_substitution(word[i + 1:end], unit, progress)
                if values is None:
                    return None
                extend(values)
                i = end + 1
                continue
            if character == "~" and i == 0 and not quoted:
                return None
            extend({character})
            i += 1
        return results

    def fold_parameter(self, inner, unit, progress):
        found = re.match(r"^([A-Za-z_][A-Za-z0-9_]*|[0-9]+|[@*#?$!-])(\[[^\]]*\])?(.*)$", inner, re.S)
        if not found:
            return None
        name, subscript, rest = found.group(1), found.group(2), found.group(3)
        absolute = os.path.join(self.root, unit.file) if unit is not None else None
        if name == "BASH_SOURCE" and subscript in (None, "[0]"):
            base = {absolute}
        elif name == "0":
            base = self.started_scripts(unit) if unit is not None else {absolute}
        elif name == "CLAUDE_PROJECT_DIR":
            base = {self.root}
        elif name == "PWD" and "shell-pwd-unfolded" not in self.breaks:
            base = self.working_directory_of(unit, progress)
        elif name.isdigit() or name in "@*#?$!-" or subscript is not None:
            base = None
        elif name in progress:
            base = set()
        else:
            sites = self.assignment_sites(name, unit)
            if not sites:
                base = None
            else:
                base = set()
                for value, holder in sites:
                    folded = self.fold_word(value, holder, progress | {name})
                    if folded is None:
                        base = None
                        break
                    base |= folded
                if base is not None and len(base) > 1:
                    base.discard("")
        if rest == "":
            return base
        for operator in (":-", ":=", "-", "="):
            if rest.startswith(operator):
                default = self.fold_word(rest[len(operator):], unit, progress)
                if default is None:
                    return None
                return default if not base else base | default
        if rest.startswith((":?", "?")):
            return base
        if rest == "%/*" and base:
            return {posixpath.dirname(value) for value in base}
        removal = re.match(r"^(%%|%|##|#)(.+)$", rest, re.S)
        if removal and base and "shell-pattern-removals-unfolded" not in self.breaks:
            # ${名%词} ${名%%词} ${名#词} ${名##词}：词也折得出时照 bash 去掉后缀或前缀（文件头「起代码的写法逐种认」）
            patterns = self.fold_word(removal.group(2), unit, progress)
            if patterns is None:
                return None
            return {remove_shell_pattern(value, pattern, removal.group(1)) for value in base for pattern in patterns}
        return None

    def function_printed_word(self, inner, unit, progress):
        """$(函数名 [<字面词…>])：看得见的那几个同名函数，整个函数体只有一条 printf '%s\\n' <词…>：每个词里的 $1… 换成给的词、在那个函数里折，
        几个词的取值逐行连起来（printf 每个词打一行，$( … ) 去掉末尾的换行）→ 取值的并；有一个不是这种写法、有一处折不出 → None
        （文件头「起代码的写法逐种认」$(函数) 那一条）。弄坏开关 shell-function-outputs-unfolded 下一律 None；
        shell-function-printed-words-single 下只认给了字面词、printf 只打一个词的那一种。"""
        commands = split_commands(inner)
        single = "shell-function-printed-words-single" in self.breaks
        if unit is None or len(commands) != 1 or len(commands[0]) < (2 if single else 1) or "shell-function-outputs-unfolded" in self.breaks:
            return None
        name = literal_word(commands[0][0])
        arguments = [literal_word(word) for word in commands[0][1:]]
        if not name or None in arguments or not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name):
            return None
        paths = [unit.file] + sorted(self.downward([unit.file]) - {unit.file}) + sorted(self.upward(unit.file)) + sorted(self.siblings(unit.file))
        functions = [function for path in paths if path in self.files for function in self.files[path].functions.get(name, [])]
        values = set()
        for function in functions:
            body = [command for command in self.facts(function).commands if command not in (["function", name], ["{"], ["}"])]
            if len(body) != 1 or len(body[0]) < 3 or (single and len(body[0]) != 3) or literal_word(body[0][0]) != "printf" \
                    or literal_word(body[0][1]) != "%s\\n":
                return None
            lines = {""}
            for position, word in enumerate(body[0][2:]):
                folded = self.fold_word(inline_positionals(word, [name, *arguments]), function, progress)
                if folded is None:
                    return None
                lines = {line + ("\n" if position else "") + value for line in lines for value in folded}
                if len(lines) > FOLD_LIMIT:
                    self.fold_limit_hits += 1
                    return None
            values |= lines if single else {line.rstrip("\n") for line in lines}
        return values or None

    def fold_substitution(self, inner, unit, progress):
        captured = self.substitution_outputs.get(inner)
        if not captured and "shell-heredoc-captures-unfolded" not in self.breaks:
            captured = self.function_output(inner, unit)
        if captured:
            # $(cat <<'词' … 词)：输出就是 here-doc 正文（不带引号的词照 bash 展开正文）
            values = set()
            for quoted, body in sorted(captured):
                folded = {body} if quoted else self.fold_heredoc(body, unit, progress)
                if folded is None:
                    return None
                values |= folded
            return values
        printed = self.function_printed_word(inner, unit, progress)
        if printed is not None:
            return printed
        commands = split_commands(inner)
        if len(commands) == 1:
            words = commands[0]
            head = literal_word(words[0])
            arguments = [word for word in words[1:] if not (literal_word(word) or "").startswith("-")]
            if head in ("dirname", "realpath", "readlink", "basename") and len(arguments) == 1:
                folded = self.fold_word(arguments[0], unit, progress)
                if folded is None:
                    return None
                if head == "dirname":
                    return {posixpath.dirname(value) for value in folded}
                if head == "basename":
                    return {posixpath.basename(value) for value in folded}
                return {posixpath.normpath(value) for value in folded}
            if head == "git":
                return git_repository_location([literal_word(word) for word in words], self.root, self.breaks)
            return None
        if len(commands) == 2 and literal_word(commands[0][0]) == "cd" and literal_word(commands[1][0]) == "pwd":
            arguments = [word for word in commands[0][1:] if not (literal_word(word) or "").startswith("-")]
            if len(arguments) == 1:
                folded = self.fold_word(arguments[0], unit, progress)
                if folded is None:
                    return None
                # 相对目录照 target_of 认相对路径的办法往下传（先根、再这份文件所在的目录）：
                # 「d=${BASH_SOURCE[0]%/*}; [[ $d == ${BASH_SOURCE[0]} ]] && d=.」那个 . 就是这份文件自己的目录
                return {posixpath.normpath(value) for value in folded}
        return None

    def target_of(self, value, unit, directories=None):
        """折出来的路径值 → ("ok", 相对根的文件) / ("outside", None) / ("missing", None)；相对路径先按前面 cd 过去的目录试。"""
        if posixpath.isabs(value):
            relative = inside_root(self.root, value)
            if relative is None:
                return ("outside", None)
            return ("ok", relative) if os.path.isfile(os.path.join(self.root, relative)) else ("missing", None)
        bases = sorted(directories) if directories else []
        if "shell-relative-directories-unrooted" not in self.breaks:
            # cd 到相对目录（cd scripts/x && python3 y.py）：那个目录照根认，拼出来的才是根里的路径
            bases = [base if posixpath.isabs(base) else posixpath.join(self.root, base) for base in bases]
        for base in bases + [self.root, os.path.dirname(os.path.join(self.root, unit.file))]:
            candidate = os.path.normpath(os.path.join(base, value))
            relative = inside_root(self.root, candidate)
            if relative is not None and os.path.isfile(candidate):
                return ("ok", relative)
        return ("missing", None)

    def links(self, unit, kinds=None):
        """→ [(source|shell|python, [文件…], 拒算或 None)]。kinds 给了只走这几种写法（起头的与展开出来的都按它挑，结果不进缓存；
        shell_sourced_files 只走 source：bash -c 这类子进程里的 source 是内联脚本那一种，不在其中）。"""
        if unit.cache is None:
            unit.cache = {}
        if kinds is None and "links" in unit.cache:
            return unit.cache["links"]
        out = []
        shell_file = self.files.get(unit.file)
        placed = {}
        facts = self.facts(unit)
        # 写出来、拷出来的脚本：这个单元与套着它的外层函数里写的都算（嵌套函数用的是外层的局部变量）
        generated, container = {}, unit
        while container is not None:
            for target, written in self.facts(container).generated.items():
                generated.setdefault(target, written)
            container = container.container
        work = [(kind, word, directory, extra, None, None) for kind, word, directory, extra in facts.invocations]
        state = {"unit": unit, "shell_file": shell_file, "facts": facts, "generated": generated, "expanded": set(), "placed": placed, "out": out,
                 "work": work}
        while work:
            kind, word, directory, extra, generator_line, overlay = work.pop(0)
            if kinds is not None and kind not in kinds:
                continue
            self.overlays.append(overlay or {})
            try:
                self.resolve_invocation(state, kind, word, directory, extra, generator_line, overlay)
            finally:
                self.overlays.pop()
        if kinds is None:
            unit.cache["links"] = out
            unit.cache["launch_arguments"] = state.get("arguments", {})
            unit.cache["python_launch_arguments"] = state.get("python_arguments", {})
        return out

    def row_snippet(self, shell_file, unit, line, word):
        if shell_file is not None and 0 < line <= len(shell_file.rows):
            return snippet_of(shell_file.rows[line - 1])
        if unit.scope == "inline" and unit.qualified:
            return unit.qualified
        return snippet_of(word)

    def place(self, state, kind, word, generator_line, loader=None):
        """拒算与内联脚本落在哪一行（同一个词装了几次各落各的行）。"""
        if generator_line:
            return generator_line
        placed = state["placed"]
        line = self.word_line(state["shell_file"], state["unit"], word, loader or kind, placed.get((kind, word), 0))
        placed[(kind, word)] = line
        return line

    def resolve_invocation(self, state, kind, word, directory, extra, generator_line, overlay):
        unit, shell_file, facts, out, work = state["unit"], state["shell_file"], state["facts"], state["out"], state["work"]
        if kind in ("python-inline", "shell-inline", "inline-unknown"):
            self.resolve_inline(state, kind, word, directory, extra, generator_line)
            return
        if kind == "python-module":
            self.resolve_module_run(state, word, directory, extra)
            return
        if kind == "command":
            self.resolve_command(state, word, directory, extra, generator_line, overlay)
            return
        if generator_line is not None and literal_word(word) is None:
            # 生成脚本正文里带展开的路径：那是生成出来的脚本自己运行时的变量，这个单元的赋值替它作不了保
            if "shell-unfolded-paths-ignored" not in self.breaks:
                out.append((kind, [], Refusal("shell-path-unfolded", unit.file, generator_line, self.row_snippet(shell_file, unit, generator_line, word))))
            return
        hits = self.fold_limit_hits
        values = self.fold_word(word, unit, frozenset())
        directories = self.fold_word(directory, unit, frozenset()) if directory is not None else None
        # 脚本路径或 cd 过去的目录折出的取值超过 FOLD_LIMIT 种（文件头「拒算」公共一行）：取值认得出、只是太多，不照根里同一相对路径那一份猜
        # （tree_copy_directories、tree_copy_targets、cd 之后的相对路径），整组、整个目录这类收全的认法照走，都不成拒算 FOLD_LIMIT_FORM。
        # 弄坏开关 fold-limit-shell-paths-skipped 下照折不出的那一路认
        over_limit = self.fold_limit_hits > hits and "fold-limit-shell-paths-skipped" not in self.breaks
        if directory is not None and directories is None and "shell-directory-copies-ignored" not in self.breaks and not over_limit:
            # cd "$目录/<字面尾巴>" 而尾巴在根里是一个目录（临时树、仓副本里的同一相对路径）：照根里那个目录认
            directories = self.tree_copy_directories(directory)
        if values is not None and directory is not None and directories is None and any(not posixpath.isabs(value) for value in values):
            values = None
        line = self.place(state, kind, word, generator_line)
        if values is None:
            unquoted = word.replace('"', "").replace("'", "")
            if kind == "exec" and not unquoted.endswith((".sh", ".py", ".bash")):
                return
            written = state["generated"].get(unquoted)
            if isinstance(written, tuple) and "shell-copied-scripts-ignored" not in self.breaks:
                # 这个单元自己 cp 出来的脚本：代码就是源那一份，源折得出就连过去
                files = self.copied_sources(written[1], unit)
                if files is not None:
                    if files:
                        out.append((kind, files, None))
                    return
            elif written is not None and not isinstance(written, tuple):
                # 这个单元自己用 printf / echo / cat <<词 写出来的脚本：代码就是这个单元里的那几行，cat <<词 的正文接着看它装了什么
                if unquoted not in state["expanded"]:
                    state["expanded"].add(unquoted)
                    for body in written:
                        inner = ShellFacts()
                        self.scan_text(body, inner)
                        work.extend((inner_kind, inner_word, inner_directory, inner_extra, line, None)
                                    for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
                        facts.programs |= inner.programs
                return
            copied = self.tree_copy_targets(unquoted, unit) if not over_limit else None
            if copied:
                out.append((kind, sorted(copied), None))
                if kind == "shell" and "shell-tree-copy-launch-words-dropped" not in self.breaks:
                    self.record_launch_words(state, sorted(copied), extra)
                return
            relative = literal_word(word)
            if directory is not None and relative and not posixpath.isabs(relative) and "shell-relative-tree-copies-ignored" not in self.breaks \
                    and not over_limit:
                # cd 到折不出的目录（临时树、仓副本）之后的相对路径：根里同一相对路径（至少两段）那一份
                copied = tree_copy_file(self.root, relative)
                if copied is not None:
                    out.append((kind, [copied], None))
                    if kind == "shell" and "shell-tree-copy-launch-words-dropped" not in self.breaks:
                        self.record_launch_words(state, [copied], extra)
                    return
            if kind in ("shell", "python", "source") and "shell-script-tail-groups-ignored" not in self.breaks:
                grouped = self.shell_tail_group(unquoted, unit)
                if grouped is not None:
                    if grouped:
                        out.append((kind, sorted(grouped), None))
                    return
            if kind in ("shell", "python", "source") and "shell-directory-wildcards-ignored" not in self.breaks:
                wildcard = self.directory_wildcard_targets(kind, unquoted, unit)
                if wildcard:
                    out.append((kind, sorted(wildcard), None))
                    return
            if kind in ("shell", "python", "source") and SHELL_POSITIONAL_WORD.match(word) and "shell-positional-scripts-refused" not in self.breaks:
                supplied = self.positional_scripts(word, unit, 0)
                if supplied is not None:
                    if supplied[0]:
                        out.append((kind, sorted(supplied[0]), None))
                    if not supplied[1]:
                        return
            launched = launched_programs_registered(unit.file, unit.short, (shell_unquoted_word(word),), self.root)
            if launched is not None:
                # 登记的被起程序（文件头）：路径折不出的照登记连上
                out.append((kind, launched, None))
                return
            if "shell-unfolded-paths-ignored" not in self.breaks and not program_word_registered(unit.file, unit.short, (shell_unquoted_word(word),)):
                form = FOLD_LIMIT_FORM if over_limit else "shell-path-unfolded"
                out.append((kind, [], Refusal(form, unit.file, line, self.row_snippet(shell_file, unit, line, word))))
            return
        files = []
        for value in sorted(values):
            # 前面 cd 过去的目录折出几个，相对路径就在每一个底下各找一次（运行期是其中哪一个都可能）
            tries = [{directory} for directory in sorted(directories)] if directories and not posixpath.isabs(value) else [directories]
            if "shell-cd-first-directory-only" in self.breaks:
                tries = [directories]
            for base in tries:
                status, file = self.target_of(value, unit, base)
                if status == "ok":
                    files.append(file)
                elif launched_programs_registered(unit.file, unit.short, (shell_unquoted_word(word),), self.root) is not None:
                    # 登记的被起程序（文件头）：路径折得出、根里没有那一份的照登记连上
                    files += launched_programs_registered(unit.file, unit.short, (shell_unquoted_word(word),), self.root)
                elif kind != "exec" and ("shell-registered-missing-paths-refused" in self.breaks
                                         or not program_word_registered(unit.file, unit.short, (shell_unquoted_word(word),))):
                    form = "shell-path-outside-root" if status == "outside" else "shell-path-missing"
                    out.append((kind, [], Refusal(form, unit.file, line, self.row_snippet(shell_file, unit, line, word))))
        if files:
            out.append((kind, files, None))
            if kind == "shell":
                self.record_launch_words(state, files, extra)
            elif kind == "python":
                # python3 <脚本> 后面的词：记下来，收那份 python 时带过去（python_launch_words）
                known = shell_known_words(extra[0] if extra else ())
                for file in files:
                    state.setdefault("python_arguments", {}).setdefault(file, []).append(known)

    def record_launch_words(self, state, files, extra):
        """bash <脚本> 后面的词：字面的那一截记下来，收脚本体时按它剪（shell_launch_prefix、shell_launch_arguments、narrowed_script_body；
        文件头「shell 起法带字面参数」）。脚本路径是字面的、照根里同一相对路径认出来的（tree_copy_targets、cd 到折不出的目录之后的相对路径）都照这里记：
        一个单元起同一份脚本的每一处都记下，shell_launch_arguments 才是这个单元的全部起法；有一处头一个词就折不出的记成 None（脚本体整份收）。"""
        words = shell_launch_prefix(tuple(literal_word(item) for item in (extra[0] if extra else ())))
        recorded = state.setdefault("arguments", {})
        for file in files:
            if words is None or recorded.get(file, set()) is None:
                recorded[file] = None
            else:
                recorded.setdefault(file, set()).add(words)

    def shell_launch_arguments(self, unit, file):
        """这个单元起 bash <file> 的每一处带的字面参数（{(词…)}，折不出的词起接 SHELL_OPEN_TAIL）；有一处头一个词就折不出、或没起过 → None（脚本体整份收）。"""
        self.links(unit)
        return (unit.cache or {}).get("launch_arguments", {}).get(file)

    def python_launch_words(self, unit, file):
        """这个单元起 python3 <file> 的每一处带的词（{KnownWords…}）；有一处没记下来（经尾巴组、目录通配认出来的）→ None（那份 python 整份收）。"""
        links = self.links(unit)
        recorded = (unit.cache or {}).get("python_launch_arguments", {}).get(file, [])
        launches = sum(files.count(file) for kind, files, _refusal in links if kind == "python")
        if "shell-python-launch-arguments-unfolded" in self.breaks or not recorded or len(recorded) != launches:
            return None
        return set(recorded)

    def narrowed_script_body(self, file, words, undecided_dropped=False):
        """bash <file> <字面参数…> 起的那份脚本体，也是带参数的 shell 入口（文件头「shell 起法带字面参数」「程序入口带参数」）：边照 narrowed_shell_body 剪过的正文认，
        单元的种类、名字、指纹与原来那一份相同（键照整份脚本体算）；剪不掉什么的交原来那一份。undecided_dropped 只给弄坏开关 shell-entry-undecided-branches-dropped。"""
        cache = self.__dict__.setdefault("narrowed_bodies", {})
        slot = (file, words, undecided_dropped)
        if slot not in cache:
            body = self.files[file].script_body
            text = narrowed_shell_body(body.body or "", words, undecided_dropped)
            if file in SHELL_ROW_SELECTED_TABLES and "shell-table-rows-unselected" not in self.breaks and SHELL_OPEN_TAIL not in words:
                text = rows_of_selected_keys(text, SHELL_ROW_SELECTED_TABLES[file], words)
            if text == body.body:
                cache[slot] = body
            else:
                # 几种参数剪出同一份正文的共用一个视图（解析结果只留一份）；弄坏开关 shell-narrowed-views-unshared 下一种参数一个视图
                views = self.__dict__.setdefault("narrowed_body_views", {})
                view_key = slot if "shell-narrowed-views-unshared" in self.breaks else (file, text)
                if view_key not in views:
                    view = copy.copy(body)
                    view.body, view.facts, view.cache = text, None, None
                    views[view_key] = view
                cache[slot] = views[view_key]
        return cache[slot]

    def launch_defines_tables(self, file, words, undecided_dropped=False):
        """bash <file> <words…> 这一趟定义不定义按第一列选行的表（文件头「shell 起法带字面参数」）：脚本体里有表的赋值行（SHELL_TABLE_ASSIGNMENT），
        照这组词剪过（narrowed_shell_body；undecided_dropped 照它）之后一行都不剩的 → False（走到的那一支以 exit 收尾，这个进程里表没定义、读 $TABLE 读不到一行）；
        别的（原来就没有赋值行、剪过还剩一行的）→ True。"""
        tables = SHELL_ROW_SELECTED_TABLES.get(file, ())
        shell_file = self.files.get(file)
        if not tables or shell_file is None or shell_file.script_body is None:
            return True
        original = shell_file.script_body.body or ""

        def assigns_table(text):
            return any(found is not None and found.group(1) in tables for found in map(SHELL_TABLE_ASSIGNMENT.match, text.split("\n")))
        return not assigns_table(original) or assigns_table(narrowed_shell_body(original, words, undecided_dropped))

    def selected_table_variable(self, variable, words):
        """按第一列选行的表的变量单元，只留 words 点名的实验那几行（文件头「shell 起法带字面参数」）：种类、名字、指纹与原来那一份相同，只剪边；剪不掉什么的交原来那一份。"""
        cache = self.__dict__.setdefault("selected_variables", {})
        if (id(variable), words) not in cache:
            # 行项（--item <实验号>.<行序号>）的行序号照整份脚本体里的复跑表数：准入键表的变量单元里没有复跑表
            text = rows_of_selected_keys(variable.body or "", SHELL_ROW_SELECTED_TABLES[variable.file], words,
                                         self.files[variable.file].script_body.body or "")
            if text == variable.body:
                cache[(id(variable), words)] = variable
            else:
                view = copy.copy(variable)
                view.body, view.facts, view.cache = text, None, None
                cache[(id(variable), words)] = view
        return cache[(id(variable), words)]

    def resolve_inline(self, state, kind, interpreter, directory, extra, generator_line):
        """内联脚本：-c 的串、喂给标准输入的 here-doc / here-string。正文折得出 → python 的交一份 InlineScript（闭包里切成单元、沿它的 import
        与 sys.argv 连下去），bash 的切开再认；折不出、或标准输入从哪来认不出 → 拒算 inline-script-not-literal。"""
        unit, shell_file, facts, out, work = state["unit"], state["shell_file"], state["facts"], state["out"], state["work"]
        language = extra[0] if kind == "inline-unknown" else ("python" if kind == "python-inline" else "shell")
        located = interpreter
        if kind != "inline-unknown" and extra[0][0] == "word" and "\n" not in extra[0][1] and "shell-refusal-lines-by-first-mention" not in self.breaks:
            # -c 的串是一个词（"$变量"）：照「解释器与这个词同在的那一行」找；只照解释器名找会落到头一处提到 bash / python3 的行上
            located = extra[0][1]
        line = self.place(state, kind, located, generator_line, language)
        snippet = self.row_snippet(shell_file, unit, line, interpreter)
        if kind == "inline-unknown":
            out.append((kind, [], Refusal(INLINE_FORM, unit.file, line, snippet)))
            return
        payload, rest, environment = extra
        hits = self.fold_limit_hits
        texts = self.inline_texts(payload, unit)
        if texts is None and language == "shell" and payload[0] == "word":
            texts = self.quoted_word_inline_texts(payload[1], unit)
        if texts is None:
            if payload[0] != "word" or not program_word_registered(unit.file, unit.short, (shell_unquoted_word(payload[1]),)):
                over_limit = self.fold_limit_hits > hits and "fold-limit-shell-paths-skipped" not in self.breaks
                out.append((kind, [], Refusal(FOLD_LIMIT_FORM if over_limit else INLINE_FORM, unit.file, line, snippet)))
            return
        if language == "python":
            if "inline-argv-commands-ignored" not in self.breaks:
                # 正文把 sys.argv[k:] 当命令起：起它的这条命令里第 k 个参数起是一条命令，在这边照命令认
                for offset in sorted(set().union(*(argv_command_offsets(text) for text in texts))):
                    if offset < len(rest):
                        inner = ShellFacts()
                        inner.directory = directory
                        self.command_facts(list(rest[offset:]), inner)
                        work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, None)
                                    for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
                        facts.programs |= inner.programs
            argv = tuple(self.argument_values(argument, unit) for argument in rest[1:])
            # 折不出的参数是一份 find 清单的（"$名字"，名字只赋成 $(find … -name <模式> …)）：记下名字模式，正文里从它一行行挑出来的文件名照它认
            listings = []
            for position, (argument, values) in enumerate(zip(rest[1:], argv)):
                patterns = self.listing_patterns(argument, unit) if values is None else None
                if patterns is not None:
                    listings.append((position, patterns))
            settings = tuple((name, self.argument_values(value, unit)) for name, value in environment)
            if "shell-exported-variables-unread" not in self.breaks:
                # 同一个单元里 export 过、折得出的变量也到内联脚本的环境里（命令前设的同名那个排在后面、照它）
                exported = ((name, self.argument_values(f'"${name}"', unit)) for name in sorted(facts.exported))
                settings = tuple((name, values) for name, values in exported if isinstance(values, frozenset)) + settings
            body_lines, body_files = self.inline_body_lines(state, payload, texts)
            if "inline-refusal-lines-at-launch" not in self.breaks and "python-module-object-refusal-lines-at-exec" not in self.breaks:
                # 参数与环境变量交进去的几行长的正文（exec 进模块对象的库正文，文件头「exec 进模块对象」）：照 -c 的正文同样找出自哪一份 here-doc，
                # 那段正文里的拒算落在它写在的那一行（PythonModuleLaunches.module_object_exec 照它给那段正文的 InlineScript）
                passed = {value for values in (*argv, *(values for _name, values in settings)) if isinstance(values, frozenset)
                          for value in values if "\n" in value} - set(texts)
                passed_lines, passed_files = self.heredoc_texts_located(shell_file, unit, passed)
                body_lines, body_files = body_lines + passed_lines, body_files + passed_files
            bases = self.directory_bases(directory, unit)
            if self.fold_limit_hits > hits and "fold-limit-shell-paths-skipped" not in self.breaks:
                # 交进内联脚本的参数、环境变量或工作目录有一个取值超过 FOLD_LIMIT 种（文件头「拒算」公共一行）：它在内联脚本里当折不出的词、
                # 读的文件与起的命令照调用方交进来的认就不收了，这一处起法拒算 FOLD_LIMIT_FORM（内联脚本照样切、照样连下去）
                out.append((kind, [], Refusal(FOLD_LIMIT_FORM, unit.file, line, snippet)))
            out.append(("python-inline", [InlineScript("python", tuple(sorted(texts)), argv, bases, unit.file, line, snippet,
                                                       settings, body_lines, body_files, tuple(listings))], None))
            return
        for text in sorted(texts):
            inner = ShellFacts()
            self.scan_text(text if "shell-inline-positionals-ignored" in self.breaks else inline_positionals(text, rest), inner)
            work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, inner.assignments)
                        for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
            facts.programs |= inner.programs

    def inline_texts(self, payload, unit):
        form, value = payload
        if form == "text":
            return {value}
        if form == "word":
            return self.fold_word(value, unit, frozenset())
        if form == "heredoc":
            return self.fold_heredoc(value, unit, frozenset())
        return None

    def quoted_word_inline_texts(self, word, unit):
        """bash -c 的串是一个词、照 fold_word 整个折不出时（文件头「内联脚本里 printf %q 引过的词」）：词里 $(printf '%q' <词>) / $(printf %q <词>)
        那几处（printf 照 bash 的引法把一个值引成恰好一个词，内联正文里那一处是一个词、只有取值不定）先换成 QUOTED_WORD_MARK，其余照 fold_word 折；
        折完逐处换成 <词> 的取值逐个 shlex.quote（折得出的）或 QUOTED_WORD_UNFOLDED（折不出的）→ 正文的集合。
        没有这种写法、别处还折不出、取值超过 FOLD_LIMIT 种（计数加一）→ None。只给 shell 的内联正文用：python 正文里那一处不是 shell 的词。
        弄坏开关 shell-inline-quoted-words-unfolded 下一律 None（照旧拒算 inline-script-not-literal）。"""
        if "shell-inline-quoted-words-unfolded" in self.breaks:
            return None
        marked, quoted_words, i, start, quoted = [], [], 0, 0, False
        while i < len(word):
            character = word[i]
            if character == "'" and not quoted:
                end = word.find("'", i + 1)
                if end < 0:
                    return None
                i = end + 1
                continue
            if character == '"':
                quoted = not quoted
                i += 1
                continue
            if character == "\\":
                i += 2
                continue
            if word.startswith("$(", i) and not word.startswith("$((", i):
                end = matching_close(word, i + 1, "(", ")")
                commands = split_commands(word[i + 2:end])
                if len(commands) == 1 and len(commands[0]) == 3 and literal_word(commands[0][0]) == "printf" and literal_word(commands[0][1]) == "%q":
                    marked.append(word[start:i] + QUOTED_WORD_MARK.format(len(quoted_words)))
                    quoted_words.append(commands[0][2])
                    start = end + 1
                i = end + 1
                continue
            i += 1
        if not quoted_words:
            return None
        texts = self.fold_word("".join(marked) + word[start:], unit, frozenset())
        if texts is None:
            return None
        for position, quoted_word in enumerate(quoted_words):
            values = self.fold_word(quoted_word, unit, frozenset())
            replacements = {QUOTED_WORD_UNFOLDED} if values is None else {shlex.quote(value) for value in values}
            mark = QUOTED_WORD_MARK.format(position)
            texts = {text.replace(mark, replacement) for text in texts for replacement in replacements}
            if len(texts) > FOLD_LIMIT:
                self.fold_limit_hits += 1
                return None
        return texts

    def fold_heredoc(self, body, unit, progress):
        """不带引号的 here-doc 正文：$名字、${…}、$( … )、`…` 逐处折（\\$ \\` \\\\ 与续行照 bash 的规矩），别的字原样；有一处折不出 → None。"""
        results, literal, i, total = {""}, [], 0, len(body)
        while i < total:
            character = body[i]
            if character == "\\" and i + 1 < total and body[i + 1] in "$`\\\n":
                if body[i + 1] != "\n":
                    literal.append(body[i + 1])
                i += 2
                continue
            if character in "$`":
                if body.startswith("$((", i):
                    return None
                if body.startswith("$(", i):
                    end = matching_close(body, i + 1, "(", ")") + 1
                elif body.startswith("${", i):
                    end = matching_close(body, i + 1, "{", "}") + 1
                elif character == "`":
                    close = body.find("`", i + 1)
                    if close < 0:
                        return None
                    end = close + 1
                else:
                    found = re.match(r"\$([A-Za-z_][A-Za-z0-9_]*|[0-9@*#?$!-])", body[i:])
                    if not found:
                        literal.append("$")
                        i += 1
                        continue
                    end = i + found.end()
                values = self.fold_word(body[i:end], unit, progress)
                if values is None:
                    return None
                prefix = "".join(literal)
                literal = []
                results = {result + prefix + value for result in results for value in values}
                if len(results) > FOLD_LIMIT:
                    self.fold_limit_hits += 1
                    return None
                i = end
                continue
            literal.append(character)
            i += 1
        prefix = "".join(literal)
        return {result + prefix for result in results}

    def argument_values(self, word, unit):
        """传给内联脚本的一个词 → 它的取值（frozenset）；折不出时「$目录/<仓里的相对路径>」按根里那一份认；都不行 → None。"""
        values = self.fold_word(word, unit, frozenset())
        if values is not None:
            return frozenset(values)
        if SHELL_POSITIONAL_WORD.match(word):
            return POSITIONAL
        unquoted = word.replace('"', "")
        bare = re.match(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$", unquoted)
        if bare:
            # 只取位置参数（与字面值）的变量："$1" 一类，取值是调用处给的那几个词
            assigned = self.assignments_of(bare.group(1), unit)
            if assigned and any(SHELL_POSITIONAL_WORD.match(value) for value in assigned) and all(
                    SHELL_POSITIONAL_WORD.match(value) or literal_word(value) is not None for value in assigned):
                return POSITIONAL
        copied = self.tree_copy_targets(unquoted, unit)
        if copied:
            return frozenset(posixpath.join(self.root, file) for file in copied)
        found = SHELL_TREE_PREFIX.match(unquoted)
        directory = tree_copy_file(self.root, found.group(1), directory=True) if found and "tree-copy-tails-ignored" not in self.breaks else None
        return frozenset((posixpath.join(self.root, directory),)) if directory else None

    def listing_patterns(self, word, unit):
        """传给内联脚本的一个词是不是一份 find 清单："$名字"，名字的每一处赋值都是 find_listing_pattern 认得出的 $(find …) → 那几个名字模式（frozenset）；
        不是这种写法、有一处赋值认不出 → None。弄坏开关 shell-find-listings-ignored 下一律 None。"""
        if "shell-find-listings-ignored" in self.breaks:
            return None
        bare = re.match(r'^"?\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?"?$', word)
        assigned = self.assignments_of(bare.group(1), unit) if bare else []
        patterns = {find_listing_pattern(value, self.breaks) for value in assigned}
        return frozenset(patterns) if assigned and None not in patterns else None

    def directory_bases(self, directory, unit):
        """内联脚本的工作目录：前面 cd 过去的目录折得出就是它（几个都算），不然是根与这份文件所在的目录（相对路径与 import 依次试）。"""
        fallback = (self.root, posixpath.dirname(posixpath.join(self.root, unit.file)))
        if directory is None:
            return fallback
        folded = self.fold_word(directory, unit, frozenset())
        if not folded:
            return fallback
        return tuple(sorted(value if posixpath.isabs(value) else posixpath.normpath(posixpath.join(self.root, value)) for value in folded))

    def resolve_module_run(self, state, word, directory, extra):
        """python3 -m <模块>：仓里的模块连到它的程序入口（包连 __main__.py）；-m unittest / pytest 后面点名的测试模块整份连上
        （测试方法按名字反射着调，闭包认不出调的是哪几个）；标准库与外部的不装仓里的代码；名字折不出、找不到仓里那一份拒算。"""
        unit, shell_file, out = state["unit"], state["shell_file"], state["out"]
        interpreter, rest = extra
        line = self.place(state, "python-module", interpreter, None, "python")
        snippet = self.row_snippet(shell_file, unit, line, interpreter)
        hits = self.fold_limit_hits
        names = self.fold_word(word, unit, frozenset())
        bases = self.directory_bases(directory, unit)
        if self.fold_limit_hits > hits and "fold-limit-shell-paths-skipped" not in self.breaks:
            # 模块名或 cd 过去的目录取值超过 FOLD_LIMIT 种：目录退回根与这份文件所在的目录去找模块就不是它真在的地方，拒算 FOLD_LIMIT_FORM（文件头「拒算」公共一行）
            out.append(("python-module", [], Refusal(FOLD_LIMIT_FORM, unit.file, line, snippet)))
            return
        if names is None:
            out.append(("python-module", [], Refusal("import-module-variable", unit.file, line, snippet)))
            return
        for name in sorted(names):
            if name not in TEST_RUNNER_MODULES:
                status, file = repository_module(self.root, bases, name, main=True)
                if status == "repo":
                    out.append(("python", [file], None))
                elif status == "unresolved":
                    out.append(("python-module", [], Refusal("import-unresolved", unit.file, line, snippet)))
                continue
            targets, position = [], 0
            while position < len(rest):
                value = literal_word(rest[position])
                if value is not None and value.startswith("-"):
                    position += 2 if value in ("-k", "-p", "-s", "-t", "-m", "--pattern", "--start-directory", "--top-level-directory") else 1
                    continue
                targets.append(rest[position])
                position += 1
            if not targets or literal_word(targets[0]) == "discover":
                out.append(("python-module", [], Refusal("import-unresolved", unit.file, line, snippet)))
                continue
            for target in targets:
                folded = self.fold_word(target, unit, frozenset())
                if folded is None:
                    out.append(("python-module", [], Refusal("import-module-variable", unit.file, line, snippet)))
                    continue
                for value in sorted(folded):
                    status, file = repository_module(self.root, bases, value)
                    if status == "repo":
                        out.append(("python-whole", [file], None))
                    elif status == "unresolved":
                        out.append(("python-module", [], Refusal("import-unresolved", unit.file, line, snippet)))

    def resolve_command(self, state, word, directory, extra, generator_line, overlay):
        """命令位置上折不出字面值的程序词：折得出 → 解释器照解释器认、仓里的脚本连上、PATH 上的程序名与函数名不装仓里的代码（函数按名字连）；
        "$@" "$1" "${@:2}" 一类：命令就是调用处那几个词（调用处已照命令认过）；折不出 → 拒算 shell-command-unfolded。"""
        unit, shell_file, out, work = state["unit"], state["shell_file"], state["out"], state["work"]
        rest, inputs = extra
        if SHELL_POSITIONAL_WORD.match(word):
            return
        array = re.match(r'^(?:\$\{[A-Za-z_][A-Za-z0-9_]*\[[@*]\]\+)?"?\$\{([A-Za-z_][A-Za-z0-9_]*)\[[@*]\]\}"?\}?$', word)
        if array and "shell-array-commands-unfolded" not in self.breaks:
            # "${命令[@]}"：数组的每一处字面赋值（x=( … )、x+=( … )）摊开当命令词，后面接这条命令余下的词。同一个数组在一个单元里只摊一次：
            # 再碰到它（x=("${x[@]}" y) 里套着自己）时它的每一处赋值已经认过
            seen = state.setdefault("arrays", collections.Counter())
            if seen[array.group(1)] >= 8 or (array.group(1), tuple(rest)) in seen:
                return
            seen[array.group(1)] += 1
            seen[(array.group(1), tuple(rest))] += 1
            assigned = self.assignments_of(array.group(1), unit)
            if assigned and all(value.startswith("(") and value.endswith(")") for value in assigned):
                for value in assigned:
                    inner = ShellFacts()
                    inner.directory = directory
                    items = [item for command in split_commands(value[1:-1]) for item in command]
                    if "shell-quoted-assignments-unread" not in self.breaks:
                        # 数组里带引号的 NAME=…：这个数组是 env 的赋值（env "${e[@]}" <命令>），不是命令词
                        items = [item for item in items if not SHELL_QUOTED_ASSIGNMENT.match(item)]
                    if items and items[0] == "env" and "shell-env-array-assignments-kept" not in self.breaks:
                        # 数组是 env 起头的前缀（env -u X NAME="$值" …）：NAME=… 那几项是 env 的赋值，不当命令词
                        items = [item for item in items if not SHELL_ENV_ARRAY_ASSIGNMENT.match(item)]
                    self.command_facts(items + list(rest), inner, inputs)
                    work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, overlay)
                                for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
                return
        hits = self.fold_limit_hits
        values = self.fold_word(word, unit, frozenset())
        # 取值超过 FOLD_LIMIT 种（同 resolve_invocation）：不照根里同一相对路径那一份猜，拒算 FOLD_LIMIT_FORM；弄坏开关 fold-limit-shell-paths-skipped 下照旧
        over_limit = self.fold_limit_hits > hits and "fold-limit-shell-paths-skipped" not in self.breaks
        if values is None:
            copied = self.tree_copy_targets(word.replace('"', ""), unit) if not over_limit else None
            if copied:
                out.append(("exec", sorted(copied), None))
                return
            unquoted = word.replace('"', "")
            prefix = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\$", unquoted)
            if prefix and "/" not in unquoted and "shell-unfolded-commands-allowed" not in self.breaks:
                # "前缀_$1" 一类：按名字前缀派给本闭包里看得见的函数（前缀相同的都连）
                out.append(("function-prefix", [prefix.group(1)], None))
                return
            outside = self.outside_program(unquoted, unit, state["facts"], 0)
            if outside or program_word_registered(unit.file, unit.short, (shell_unquoted_word(word),)):
                return
            line = self.place(state, "command", word, generator_line, "exec")
            out.append(("command", [], Refusal(FOLD_LIMIT_FORM if over_limit else "shell-command-unfolded", unit.file, line,
                                               self.row_snippet(shell_file, unit, line, word))))
            return
        for value in sorted(values):
            base = posixpath.basename(value)
            if base in SHELL_INTERPRETERS or base in PYTHON_INTERPRETERS:
                inner = ShellFacts()
                inner.directory = directory
                self.command_facts([shlex.quote(value)] + list(rest), inner, inputs)
                work.extend((inner_kind, inner_word, inner_directory, inner_extra, generator_line, overlay)
                            for inner_kind, inner_word, inner_directory, inner_extra in inner.invocations)
            elif "/" in value or value.endswith((".sh", ".py", ".bash")):
                status, file = self.target_of(value, unit)
                if status == "ok":
                    out.append(("exec", [file], None))

    def function_output(self, inner, unit):
        """$(函数名)：看得见的那几个同名函数，整个函数体只有一条「printf '%s\\n' <字面的行…>」或「cat <<'词' … 词」的 → 它打的那几行
        （(带不带引号, 正文) 的集合，同 substitution_outputs）；有一个不是这种写法 → None。"""
        commands = split_commands(inner)
        if unit is None or len(commands) != 1 or len(commands[0]) != 1:
            return None
        name = literal_word(commands[0][0])
        if not name or not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name):
            return None
        paths = [unit.file] + sorted(self.downward([unit.file]) - {unit.file}) + sorted(self.upward(unit.file)) + sorted(self.siblings(unit.file))
        functions = [function for path in paths if path in self.files for function in self.files[path].functions.get(name, [])]
        if not functions:
            return None
        outputs = set()
        for function in functions:
            commands = [command for command in self.facts(function).commands if command not in (["function", name], ["{"], ["}"])]
            if len(commands) != 1:
                return None
            command = commands[0]
            values = [literal_word(word) for word in command]
            if len(command) >= 2 and values[0] == "printf" and values[1] == "%s\\n" and None not in values:
                outputs.add((True, "\n".join(values[2:])))
                continue
            body = self.function_heredoc(function)
            if values == ["cat"] and body is not None:
                outputs.add(body)
                continue
            return None
        return outputs

    def function_heredoc(self, function):
        """函数体是一条「cat <<词 … 词」：→ (带不带引号, 正文)；不是 → None。"""
        rows = (function.body or "").split("\n")
        bodies = heredoc_bodies(rows)
        return next(iter(bodies.values())) if len(bodies) == 1 else None

    def outside_program(self, unquoted, unit, facts, depth):
        """命令位置上折不出的词是不是根外的程序：每一种取法要么折得出（不是解释器、不是仓里的脚本），要么是「$目录/<字面尾巴>」而尾巴
        不带脚本后缀、在根里也找不到（工具链、补好的 QEMU 这类根外的二进制）；变量的每一处赋值是这种写法，或只取位置参数（"$1"、"$@"：
        命令就是调用处那几个词）也算。尾巴是登记的外部程序名的，记进闭包点名的外部程序。"""
        if depth > 4 or "shell-unfolded-commands-allowed" in self.breaks:
            return False
        pieces = re.split(r"\$\{[^{}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]", unquoted)
        if len(pieces) > 1 and pieces[0] == "" and pieces[-1].startswith("/") and len(pieces[-1]) > 1 and "`" not in unquoted and "$(" not in unquoted:
            tail = pieces[-1].lstrip("/")
            if tail.endswith((".sh", ".py", ".bash")) or tree_copy_file(self.root, tail) is not None:
                return False
            if os.path.exists(os.path.join(self.root, tail)):
                return False
            program = external_program_name(posixpath.basename(tail), self.breaks)
            if program is not None:
                facts.programs.add(program)
            return True
        bare = re.match(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$", unquoted)
        if not bare:
            return False
        assigned = self.assignment_sites(bare.group(1), unit)
        if not assigned:
            return False
        for value, holder in assigned:
            if SHELL_POSITIONAL_WORD.match(value):
                continue
            folded = self.fold_word(value, holder, frozenset())
            if folded is not None:
                if any(posixpath.basename(item) in SHELL_INTERPRETERS | PYTHON_INTERPRETERS or item.endswith((".sh", ".py", ".bash"))
                       or self.target_of(item, unit)[0] == "ok" for item in folded):
                    return False
                for item in folded:
                    program = external_program_name(item, self.breaks)
                    if program is not None:
                        facts.programs.add(program)
                continue
            if not self.outside_program(value.replace('"', ""), holder, facts, depth + 1):
                return False
        return True

    def copied_sources(self, sources, unit):
        """cp 的源词 → 仓里的文件；有一个源折不出或不在根里 → None。"""
        files = set()
        for source in sources:
            values = self.fold_word(source, unit, frozenset())
            if values is None:
                return None
            for value in values:
                status, file = self.target_of(value, unit)
                if status != "ok":
                    return None
                files.add(file)
        return sorted(files)

    def tree_copy_directories(self, word):
        """cd 过去的目录词折不出、是「$目录/<字面尾巴>」而尾巴在根里是一个目录（临时树、仓副本里的同一相对路径）→ {根里那个目录}；别的 → None。"""
        found = SHELL_TREE_PREFIX.match(word.replace('"', "").replace("'", ""))
        copied = tree_copy_file(self.root, found.group(1), directory=True) if found else None
        return {posixpath.join(self.root, copied)} if copied is not None else None

    def working_directory_of(self, unit, progress):
        """$PWD：这份文件的脚本体顶层恰好一处 cd、它折得出恰好一个目录（cd "$(dirname "$0")/../.." 这一类）→ {那个目录}；脚本体自己、别的 → None。
        函数在子 shell 里 cd 到临时树再取 $PWD 的同样照这一个目录认：拷进临时树的是同一相对路径那一份（同「临时树尾巴」那一条）。"""
        shell_file = self.files.get(unit.file) if unit is not None else None
        body = shell_file.script_body if shell_file is not None else None
        if body is None or body is unit or "PWD" in progress:
            return None
        changes = [words for words in (leading_command_words(command) for command in self.facts(body).commands) if words and literal_word(words[0]) == "cd"]
        if len(changes) != 1:
            return None
        arguments = [word for word in changes[0][1:] if not (literal_word(word) or "").startswith("-")]
        values = self.fold_word(arguments[0], body, progress | {"PWD"}) if len(arguments) == 1 else None
        if not values or len(values) != 1:
            return None
        value = next(iter(values))
        return {posixpath.normpath(value if posixpath.isabs(value) else posixpath.join(self.root, value))}

    def directory_wildcard_targets(self, kind, unquoted, unit):
        """「<折得出的目录>/<带展开的文件名>」（python3 "s/tools/$1"、bash "$dir/$name"）而那个目录在根里：目录里每一份同种脚本（python 起 .py，
        bash 与 source 起 .sh、.bash）——起的是调用方点名的其中一份，哪一份都可能，就都连（同 python 那边 os.path.join(<仓里的目录>, <折不出的文件名>)）。"""
        directory, _slash, name = unquoted.rpartition("/")
        if not directory or "$" not in name:
            return None
        values = self.fold_word(directory, unit, frozenset())
        if not values:
            return None
        suffixes = (".py",) if kind == "python" else (".sh", ".bash")
        files = set()
        for value in values:
            absolute = posixpath.normpath(value if posixpath.isabs(value) else posixpath.join(self.root, value))
            relative = inside_root(self.root, absolute)
            if relative is None or not os.path.isdir(absolute):
                return None
            files |= {posixpath.normpath(posixpath.join(relative, entry)) for entry in os.listdir(absolute)
                      if entry.endswith(suffixes) and os.path.isfile(os.path.join(absolute, entry))}
        return files or None

    def positional_scripts(self, word, unit, depth):
        """解释器后面的脚本是这个函数的位置参数（"$@" "$1" "${@:2}"）：脚本由调用处给。这份文件、source 它的与和它一起被 source 的那几份里每一处以这个函数名起头的命令
        取对应的那个词，在调用处那个单元里折（折不出的试临时树尾巴；调用处又是位置参数的接着往上找，至多 3 层）
        → (相对根的文件集合, 有没有折不出的调用处)；不在函数里、一处调用都没有 → None。"""
        found = re.match(r'^"?\$(?:\{(@|\*|[1-9][0-9]*)(?::([1-9][0-9]*))?\}|(@|\*|[1-9]))"?$', word)
        if unit is None or unit.kind != "function" or depth > 3 or not found:
            return None
        which = found.group(1) or found.group(3)
        position = int(found.group(2)) if found.group(2) else (1 if which in "@*" else int(which))
        callee = unit.name.split("::")[-1]
        files, unfolded, sites = set(), False, 0
        for path in [unit.file] + sorted(self.upward(unit.file)) + sorted(self.siblings(unit.file)):
            shell_file = self.files.get(path)
            for caller in shell_file.units if shell_file is not None else ():
                for command in self.facts(caller).commands:
                    words = leading_command_words(command)
                    if not words or literal_word(words[0]) != callee or position >= len(words):
                        continue
                    sites += 1
                    supplied = words[position]
                    nested = self.positional_scripts(supplied, caller, depth + 1) if SHELL_POSITIONAL_WORD.match(supplied) else None
                    if nested is not None:
                        files |= nested[0]
                        unfolded = unfolded or nested[1]
                        continue
                    values = self.fold_word(supplied, caller, frozenset())
                    if values is None:
                        copied = self.tree_copy_targets(supplied.replace('"', ""), caller)
                        files |= copied or set()
                        unfolded = unfolded or not copied
                        continue
                    for value in values:
                        status, file = self.target_of(value, caller)
                        if status == "ok":
                            files.add(file)
                        else:
                            unfolded = True
        return (files, unfolded) if sites else None

    def shell_tail_group(self, unquoted, unit):
        """「$<折不出的目录>/<折得出的尾巴>」（尾巴里的变量照赋值折："$ROOT/$SAMPLE_DIRECTORY/driver.sh" 一类）：仓里以这一截收尾的脚本整组
        （相对根的集合，同 python 一侧的 script_tail_group；一份都没有的不是仓里的脚本，交空集合，不连也不拒算）；
        尾巴折不出、不是 .sh / .py / .bash、最后一段以 . 起头、往根外走的 → None（照旧拒算）。"""
        found = re.match(r"^\$\{?[A-Za-z_][A-Za-z0-9_]*\}?(/.+)$", unquoted)
        if not found:
            return None
        tails = self.fold_word(found.group(1), unit, frozenset())
        if not tails:
            return None
        if "repository_files" not in self.__dict__:
            self.repository_files = files_under(self.root, ("",))
        files = set()
        for tail in tails:
            stripped = posixpath.normpath(tail).strip("/")
            if not stripped.endswith((".sh", ".py", ".bash")) or posixpath.basename(stripped).startswith(".") or stripped.startswith(".."):
                return None
            files |= {relative for relative in self.repository_files if relative == stripped or relative.endswith("/" + stripped)}
        return files

    def tree_copy_targets(self, unquoted, unit, depth=0):
        """「$目录/<字面尾巴>」→ 根里同一相对路径那一份（tree_copy_file）；「$变量」→ 它每一处赋值各自折得出或各自是这种写法时的并。都不行 → None。"""
        if "tree-copy-tails-ignored" in self.breaks or depth > 4:
            return None
        found = SHELL_TREE_PREFIX.match(unquoted)
        if found:
            copied = tree_copy_file(self.root, found.group(1))
            return {copied} if copied is not None else None
        bare = re.match(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$", unquoted)
        if not bare:
            return None
        words = self.assignment_sites(bare.group(1), unit)
        if not words:
            return None
        files = set()
        for word, holder in words:
            values = self.fold_word(word, holder, frozenset())
            if values is not None:
                for value in values:
                    status, file = self.target_of(value, unit)
                    if status != "ok":
                        return None
                    files.add(file)
                continue
            inner = self.tree_copy_targets(word.replace('"', ""), holder, depth + 1)
            if inner is None:
                return None
            files |= inner
        return files

    def inline_body_lines(self, state, payload, texts):
        """内联 python 的每份正文出自原文里哪一份 here-doc → (InlineScript 的 body_lines, body_files)：body_lines 是 ((正文, 那份 here-doc 正文头一行的行号)…)，
        body_files 是 ((正文, 那份 here-doc 所在的文件)…)，只记不在起它的这份文件里的。
        直接喂给解释器的 here-doc（payload 是 text / heredoc）在起它的这个单元里找，同一个单元里几份正文相同的各落各的（接着上一份往后找），
        不带引号的 here-doc 折过变量的正文，行数与原文那一份相同的照它的行号；-c 交的折出来的正文（变量里装的 $(cat <<词 …)）在整份文件里找逐字相同的那一份，
        找不到的再到它（传递地）source 的、source 了它的与和它一起被 source 的那几份里依次找（拆成入口加被 source 的库：入口把程序装进变量，库里的函数起它；
        文件头「内联正文出自别的文件」）。弄坏开关 inline-refusal-lines-at-launch 下一份都不认（拒算都落在起它的那一行）；
        inline-bodies-launch-file-only 下只在起它的这份文件里找。"""
        if "inline-refusal-lines-at-launch" in self.breaks:
            return (), ()
        shell_file, unit, placed = state["shell_file"], state["unit"], state["placed"]
        if payload[0] in ("text", "heredoc"):
            line = self.heredoc_body_line(shell_file, unit, payload[1], placed.get(HEREDOC_BODY_PLACED, 0))
            if line is None:
                return (), ()
            placed[HEREDOC_BODY_PLACED] = line
            return tuple((text, line) for text in sorted(texts) if text.count("\n") == payload[1].count("\n")), ()
        return self.heredoc_texts_located(shell_file, unit, texts)

    def heredoc_texts_located(self, shell_file, unit, texts):
        """几份正文各出自原文里哪一份 here-doc → (body_lines, body_files)（写法同 inline_body_lines）：先在起它的这份文件里整份找逐字相同的，
        找不到的再到它（传递地）source 的、source 了它的与和它一起被 source 的那几份里依次找；弄坏开关 inline-bodies-launch-file-only 下只在起它的这份文件里找。"""
        others = []
        if "inline-bodies-launch-file-only" not in self.breaks and unit is not None:
            others = sorted(self.downward([unit.file]) - {unit.file}) + sorted(self.upward(unit.file)) + sorted(self.siblings(unit.file))
        lines, files = [], []
        for text in sorted(texts):
            line = self.heredoc_body_line(shell_file, None, text)
            if line is not None:
                lines.append((text, line))
                continue
            for other in others:
                line = self.heredoc_body_line(self.files.get(other), None, text)
                if line is not None:
                    lines.append((text, line))
                    files.append((text, other))
                    break
        return tuple(lines), tuple(files)

    def heredoc_body_line(self, shell_file, unit, body, after=0):
        """here-doc 正文 body 在原文里从哪一行起：在这个单元的原文行里（函数是它的起止行，脚本体与不给单元的是整份）找逐行与 body 相同的那份正文
        （原文里的 here-doc 由 raw_heredoc_rows 一份文件切一次），有几份时取正文头一行在 after 之后的头一份、都不在之后取头一份 → 正文头一行的行号；
        一份都对不上（字面 -c 串、正文折过变量、bash 改写过正文）→ None。"""
        if shell_file is None or body is None:
            return None
        bodies = self.__dict__.setdefault("raw_heredoc_rows_by_file", {})
        if id(shell_file) not in bodies:
            bodies[id(shell_file)] = raw_heredoc_rows(shell_file.rows)
        wanted = tuple(body.split("\n"))
        # read -d '' 读进变量的 here-doc 内容带末尾一个换行（read_heredoc_assignment），原文里的正文行不带它
        trimmed = wanted[:-1] if len(wanted) > 1 and wanted[-1] == "" else None
        scoped = unit is not None and unit.kind == "function"
        low, high = (unit.first_line, unit.last_line) if scoped else (1, len(shell_file.rows))
        found = [line for start, line, lines in bodies[id(shell_file)] if low <= start <= high and (lines == wanted or lines == trimmed)]
        later = [line for line in found if line > after]
        return later[0] if later else (found[0] if found else None)

    @staticmethod
    def word_line(shell_file, unit, word, kind=None, after=0):
        """拒算落在哪一行：先找这个词与装代码的那个命令词（source / . / bash / sh / python3 / 直接执行）同在的一行，
        找不到再退回第一处提到这个词的行（赋值、重定向、cp 这类只是提到路径的行排在后面）。
        同一个单元里同一个词装了几次，after 是上一次落的行：先往后找，各落各的行。"""
        if shell_file is None:
            return unit.first_line
        first = unit.first_line if unit.kind == "function" else 1
        last = unit.last_line if unit.kind == "function" else len(shell_file.rows)
        ranges = [(first, last), (first, len(shell_file.rows))]
        if after and first <= after < last:
            ranges.insert(0, (after + 1, last))
        needles = [word.strip('"')] + [f"${name}" for name in VARIABLE_REFERENCE.findall(word)] + [f"${{{name}" for name in VARIABLE_REFERENCE.findall(word)]
        tail = word.strip('"').rsplit("/", 1)[-1]
        if tail and "$" not in tail:
            needles.append(tail)
        loaders = {"source": r"(?:^|[\s;&|({])(?:source|\.)\s+", "shell": r"(?:^|[\s;&|({\"'=])(?:\S*/)?(?:bash|sh)\s",
                   "python": r"(?:^|[\s;&|({\"'=])(?:\S*/)?python3?\s", "exec": r""}
        loader = loaders.get(kind)
        breaks = active_breaks()
        tiers = [loader] if loader is not None else []
        if kind in ("shell", "python", "source") and "shell-refusal-lines-positional-unmatched" not in breaks:
            # 剪过的正文（narrowed_shell_body）里解释器是代进来的位置参数：原文那一行上是 "$1" "${1:-}" 后面紧跟这个词（照解释器名找不到它）
            tiers.append(POSITIONAL_INTERPRETER_BEFORE_WORD)
        if kind == "exec" and "shell-refusal-lines-by-first-mention" not in breaks:
            # 命令位置上的词不一定在行首：跟在 && || ; | ( { ! 与 then / do / else 之后的也是命令位置
            tiers.append(COMMAND_POSITION_BEFORE_WORD)
        for pattern in tiers + [None]:
            for low, high in ranges:
                for needle in needles:
                    # 词按整个词找（bounded_needle）：拿子串找，「in」落在头一处带 rejoin 的 bash 行上
                    found = bounded_needle(needle, breaks)
                    for number in range(low, high + 1):
                        if not 0 < number <= len(shell_file.rows):
                            continue
                        row = shell_file.rows[number - 1]
                        if not re.search(found, row) or row.lstrip().startswith("#"):
                            continue
                        if pattern is COMMAND_POSITION_BEFORE_WORD or pattern is POSITIONAL_INTERPRETER_BEFORE_WORD:
                            if re.search(pattern + found, row):
                                return number
                            continue
                        if pattern is None or (pattern == "" and row.lstrip().startswith((needle, f'"{needle}'))) or (
                                pattern and re.search(pattern + r".*" + found, row)):
                            return number
        return unit.first_line

    def functions_in(self, paths, name):
        out = []
        for member in sorted(paths):
            if member in self.files:
                out += self.files[member].functions.get(name, [])
        return out

    def function_names_in(self, paths):
        names = set()
        for member in paths:
            if member in self.files:
                names |= set(self.files[member].functions)
        return names


REPOSITORY_MODULE_NAMES = {}


def repository_module_names(root):
    """根里每份 .py 的模块名与每个包目录名（import 认不出时分「仓里该有却找不到」与「外部的」）。"""
    if root not in REPOSITORY_MODULE_NAMES:
        names = set()
        for directory, subdirectories, files in os.walk(root):
            subdirectories[:] = [name for name in subdirectories if name not in SKIPPED_DIRECTORIES]
            names |= {name[:-3] for name in files if name.endswith(".py")}
            if "__init__.py" in files:
                names.add(os.path.basename(directory))
        REPOSITORY_MODULE_NAMES[root] = names
    return REPOSITORY_MODULE_NAMES[root]


def repository_module(root, bases, name, main=False):
    """python3 -m 的模块名、unittest 点名的测试模块（点分的名字或 .py 路径）在 bases（工作目录）里 → ("repo", 相对根的文件) /
    ("stdlib" 或 "external", None) / ("unresolved", None)。main=True 时包取它的 __main__.py。"""
    if name.endswith(".py") or "/" in name:
        for base in bases:
            candidate = posixpath.normpath(name if posixpath.isabs(name) else posixpath.join(base, name))
            relative = inside_root(root, candidate)
            if relative is not None and os.path.isfile(candidate):
                return "repo", relative
        return "unresolved", None
    parts = [part for part in name.split(".") if part]
    if not parts or not all(re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", part) for part in parts):
        return "unresolved", None
    for base in bases:
        stem = posixpath.join(base, *parts)
        for option in ((stem + ".py", posixpath.join(stem, "__main__.py")) if main else (stem + ".py", posixpath.join(stem, "__init__.py"))):
            relative = inside_root(root, option)
            if relative is not None and os.path.isfile(option):
                return "repo", relative
    if parts[0] in sys.stdlib_module_names or parts[0] in sys.builtin_module_names:
        return "stdlib", None
    if parts[0] in repository_module_names(root):
        return "unresolved", None
    return "external", None


def shell_files_under(root, under=None):
    found = files_under(root, (".sh", ".bash"), under)
    return found


def shell_units(root, under=None):
    return ShellIndex(root, shell_files_under(root, under or list(DEFAULT_SCRIPT_DIRECTORIES)))


def language_of_file(root, path):
    if path.endswith(".py"):
        return "python"
    if path.endswith((".sh", ".bash")):
        return "shell"
    try:
        with open(os.path.join(root, path), "rb") as handle:
            first = handle.readline()
    except OSError:
        return None
    if first.startswith(b"#!") and (b"bash" in first or first.rstrip().endswith(b"/sh")):
        return "shell"
    if first.startswith(b"#!") and b"python" in first:
        return "python"
    return None


def shell_sourced_files(root, path):
    """shell 脚本 path（相对根）在它自己那个 shell 里（传递地）source / . 进来的仓内文件 → (按路径排的文件元组，不含 path 自己, 拒算列表)
    （文件头「Rust 闭包读到的 shell 脚本」）。路径照 ShellIndex.links 的折法；每份文件的每个单元都看（读到它的是整份字节，不按调没调到挑），
    新收进来的那几份接着看，到不再多出文件为止。弄坏开关 shell-data-sources-nested-unfollowed：只看 path 自己那一层；
    shell-data-source-refusals-dropped：折不出的不拒算。"""
    breaks = active_breaks()
    index = ShellIndex(root, [path])
    files = {path}
    while True:
        found, refusals = set(), set()
        scanned = {path} if "shell-data-sources-nested-unfollowed" in breaks else files
        for current in sorted(scanned):
            shell_file = index.files.get(current)
            if shell_file is None:
                continue
            refusals.update(shell_file.refusals)
            for unit in shell_file.units:
                if unit.data:
                    continue
                for _kind, targets, refusal in index.links(unit, kinds=("source",)):
                    if refusal is not None:
                        refusals.add(refusal)
                    found.update(targets)
        grown = files | found
        if grown == files:
            break
        index.add(sorted(grown - files))
        files = grown
    if "shell-data-source-refusals-dropped" in breaks:
        refusals = set()
    ordered = sorted(refusals, key=lambda refusal: (refusal.file, refusal.line, refusal.form, refusal.snippet))
    return tuple(sorted(files - {path})), ordered


# ── python 与 shell 的闭包（两种语言互相连） ────────────────────────────────

def inline_shell_units(cache, script):
    """InlineScript（shell 的：python 起的 bash -c、shell=True、os.system）→ 每份正文一个内联 shell 单元（$0 $1… 折得出单值的换进正文）。"""
    if script not in cache:
        words = []
        for value in script.argv:
            single = next(iter(value)) if value is not None and len(value) == 1 else None
            words.append(single if single is not None and "'" not in single and '"' not in single else "'")
        # 起它时设的环境变量（单值的）写成正文前面的赋值，正文里读它们照本单元的赋值折
        settings = "".join(f"{name}={shlex.quote(next(iter(values)))}\n" for name, values in script.environment
                           if values is not None and len(values) == 1 and re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name))
        units = []
        for position, text in enumerate(script.texts):
            body = settings + (text if "shell-inline-positionals-ignored" in active_breaks() else inline_positionals(text, words))
            unit = Unit("shell", "function", f"<inline@{script.line}#{position}>", script.file, script.line, script.line, sha256_text(body))
            unit.body, unit.scope, unit.qualified = body, "inline", script.snippet
            units.append(unit)
        cache[script] = units
    return cache[script]


def inline_refusals(root, python, shell, scripts, depth=0):
    """dump 用：几份内联脚本里收得到的拒算（python 的切成单元逐个看，shell 的看它的连边），连同它们里面再起的内联脚本。"""
    refusals, cache = set(), {}
    for script in scripts:
        if depth > 4:
            break
        if script.language == "python":
            for module in python.inline_modules(script):
                refusals.update(module.refusals)
                for unit in module.units:
                    facts = module.facts(unit)
                    refusals.update(facts.refusals)
                    refusals |= inline_refusals(root, python, shell, facts.inline_scripts, depth + 1)
            continue
        for unit in inline_shell_units(cache, script):
            nested = []
            for kind, files, refusal in shell.links(unit):
                if refusal is not None:
                    refusals.add(refusal)
                if kind == "python-inline":
                    nested += files
            refusals |= inline_refusals(root, python, shell, nested, depth + 1)
    return refusals
