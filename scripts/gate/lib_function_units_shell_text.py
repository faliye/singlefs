"""shell 的规范文本与切词：bash -n、declare -f 规范化，here-doc 与引号里的行，$1 一类参数的代入，分支判定，按行键选行的表，
命令切分（split_commands）、替换与进程替换。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
import fnmatch
import os
import posixpath
import re
import shlex
import subprocess
import tempfile
from lib_function_units_lexer import active_breaks
from lib_function_units_base import ASSIGNMENT_WORD, FunctionUnitsError, LIBRARY_ROOT, SHELL_OPEN_TAIL, SHELL_TIMEOUT_SECONDS, SKIPPED_DIRECTORIES
from lib_function_units_python_facts import KnownWords


SHELL_PARAMETER_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def expand_shell_parameters(text, environment):
    """双引号串里的参数展开，照 environment 取值：只认 $名、${名}、${名:-词}、${名-词}（词里照样展开）与反斜杠转义；
    $(…)、反引号、${名:=词} 这类别的写法抛 ValueError（登记的那一行被写成认不出的写法时，照新写法改 cross_run_cache_registry）。"""
    pieces, index = [], 0
    while index < len(text):
        character = text[index]
        if character == "\\" and index + 1 < len(text):
            pieces.append(text[index + 1])
            index += 2
            continue
        if character == "`" or text.startswith("$(", index):
            raise ValueError(f"认不出的展开（命令替换）：{text}")
        if character != "$":
            pieces.append(character)
            index += 1
            continue
        if text.startswith("${", index):
            depth, end = 1, index + 2
            while end < len(text) and depth:
                if text.startswith("${", end):
                    depth += 1
                    end += 2
                    continue
                if text[end] == "}":
                    depth -= 1
                end += 1
            if depth:
                raise ValueError(f"${{ 没有配对的 }}：{text}")
            inner = text[index + 2:end - 1]
            matched = SHELL_PARAMETER_NAME.match(inner)
            if matched is None:
                raise ValueError(f"认不出的展开（${{{inner}}}）：{text}")
            name, operator_and_word = matched.group(0), inner[matched.end():]
            value = environment.get(name)
            if not operator_and_word:
                pieces.append(value or "")
            elif operator_and_word.startswith(":-"):
                pieces.append(value if value else expand_shell_parameters(operator_and_word[2:], environment))
            elif operator_and_word.startswith("-"):
                pieces.append(value if value is not None else expand_shell_parameters(operator_and_word[1:], environment))
            else:
                raise ValueError(f"认不出的展开（${{{inner}}}）：{text}")
            index = end
            continue
        matched = SHELL_PARAMETER_NAME.match(text, index + 1)
        if matched is None:
            pieces.append(character)
            index += 1
            continue
        pieces.append(environment.get(matched.group(0)) or "")
        index = matched.end()
    return "".join(pieces)


def shell_assignment_value(relative, prefix, environment=None):
    """这份库所在仓里 relative 那份 shell 脚本：恰好一行（去掉首尾空白）以 prefix 起头、后面是一个双引号串，照 environment（不给取此刻的环境）展开那个串交回。
    命中的不是恰好一行、后面不是双引号串、串里有认不出的展开，抛 ValueError。"""
    with open(os.path.join(LIBRARY_ROOT, relative), encoding="utf-8") as handle:
        matches = [line.strip()[len(prefix):] for line in handle if line.strip().startswith(prefix)]
    if len(matches) != 1:
        raise ValueError(f"{relative} 里以「{prefix}」起头的行有 {len(matches)} 行（要恰好一行）：那份脚本改了写法，照新写法改 cross_run_cache_registry 里它那一行")
    quoted = matches[0]
    if len(quoted) < 2 or not quoted.startswith('"') or not quoted.endswith('"'):
        raise ValueError(f"{relative} 里「{prefix}」后面不是一个双引号串：{quoted}")
    return expand_shell_parameters(quoted[1:-1], os.environ if environment is None else environment)


def shell_assignment_row(name, relative, prefix):
    """登记表里 shell 那一种的一行：现取 = shell_assignment_value(relative, prefix)。"""
    return (name, relative, lambda: shell_assignment_value(relative, prefix))


# ── shell ───────────────────────────────────────────────────────────────────

SHELL_WRAPPER = "__singlefs_function_units_body"
SHELL_NORMALIZE_SCRIPT = r'''
for source_path in "$@"; do
  if bash -n "$source_path" 2>"$source_path.check" && [[ ! -s "$source_path.check" ]]; then
    source_text="$(<"$source_path")"
    if eval "__singlefs_function_units_body() {
$source_text
}" 2>/dev/null; then
      declare -f __singlefs_function_units_body >"$source_path.normal"
    fi
    unset -f __singlefs_function_units_body
  fi
done
'''
KEPT_SHELL_COMMENT = re.compile(r"(?:^|[\s;&|()])(#\s*(?:[A-Za-z][A-Za-z0-9_-]*:|shellcheck\b).*)$")
FUNCTION_HEADER = re.compile(r"^( *)function (\S+) \(\) $")
HEREDOC_START = re.compile(r"(?<!<)<<(?!<)(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
HEREDOC_BODY_PLACED = ("heredoc-body", None)   # ShellIndex.resolve_inline 记在 state["placed"] 里：这个单元上一份喂给 python 的 here-doc 正文落在哪一行
TOP_ASSIGNMENT = re.compile(r"^    (?:(?:export|readonly|declare|typeset)(?: -[A-Za-z]+)* )?([A-Za-z_][A-Za-z0-9_]*)(?:\[[^\]]*\])?\+?=")

SHELL_KEYWORDS = frozenset(("if", "then", "else", "elif", "fi", "do", "done", "while", "until", "!", "{", "}", "time", "esac", "coproc"))
SHELL_DECLARATIONS = frozenset(("local", "declare", "export", "readonly", "typeset"))
SHELL_WORDS = re.compile(r"[A-Za-z_][A-Za-z0-9_.:-]*")
VARIABLE_REFERENCE = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)")


def kept_shell_comments(text):
    rows = text.split("\n")
    kept = [rows[0]] if rows and rows[0].startswith("#!") else []
    for row in rows:
        found = KEPT_SHELL_COMMENT.search(row)
        if found:
            kept.append(found.group(1))
    return kept


def shell_normal_forms(contents):
    """每份内容 → declare -f 打出来的那一份（str）；bash -n 不过、有告警、包不进函数的 → None。bash 起不来抛 FunctionUnitsError。"""
    with tempfile.TemporaryDirectory(prefix="function-units-shell-") as directory:
        paths = []
        for position, content in enumerate(contents):
            path = os.path.join(directory, f"{position}.sh")
            with open(path, "wb") as handle:
                handle.write(content)
            paths.append(path)
        environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LC_ALL": "C", "HOME": directory}
        try:
            completed = subprocess.run(["bash", "--norc", "--noprofile", "-c", SHELL_NORMALIZE_SCRIPT, "function-units", *paths],
                                       cwd=directory, env=environment, capture_output=True, stdin=subprocess.DEVNULL, timeout=SHELL_TIMEOUT_SECONDS)
        except (OSError, subprocess.TimeoutExpired) as error:
            raise FunctionUnitsError(f"bash 解析 .sh 没跑完：{error}") from error
        if completed.returncode != 0:
            raise FunctionUnitsError(f"bash 解析 .sh 退 {completed.returncode}：{completed.stderr.decode('utf-8', 'replace').strip()[:300]}")
        forms = []
        for path in paths:
            if os.path.isfile(path + ".normal"):
                with open(path + ".normal", "rb") as handle:
                    forms.append(handle.read().decode("utf-8", "surrogateescape"))
            else:
                forms.append(None)
        return forms


# 规范文本里写在 if 条件上的 here-doc：bash 在连接符处才补打延后的正文，条件那一行以「<<词; then」收尾，正文打在 then 那一串的头一条命令之后
SHELL_DEFERRED_HEREDOC = re.compile(r"^(.*<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2); then$")


def raw_heredoc_bodies(raw_rows):
    """原文里每个 here-doc → {词: [正文的行…]…}（找不到收尾那一行的不算）。"""
    bodies = collections.defaultdict(list)
    for position, row in enumerate(raw_rows):
        for found in HEREDOC_START.finditer(row):
            delimiter, strip_tabs = found.group(3), found.group(1) == "-"
            end = next((later for later in range(position + 1, len(raw_rows))
                        if (raw_rows[later].lstrip("\t") if strip_tabs else raw_rows[later]) == delimiter), None)
            if end is not None:
                bodies[delimiter].append(raw_rows[position + 1:end])
    return bodies


def raw_heredoc_rows(raw_rows):
    """原文里每个 here-doc → [(<<词 所在的行号, 正文头一行的行号, 正文的行（元组）)…]，行号从 1 起；<<- 的正文去掉行首制表符；
    一行里几个 <<词 的正文一份接一份往下排；找不到收尾那一行的不算（同 raw_heredoc_bodies）。"""
    out = []
    for position, row in enumerate(raw_rows):
        cursor = position + 1   # 这一行上下一份正文从哪一行（下标）起
        for found in HEREDOC_START.finditer(row):
            delimiter, strip_tabs = found.group(3), found.group(1) == "-"
            end = next((later for later in range(cursor, len(raw_rows))
                        if (raw_rows[later].lstrip("\t") if strip_tabs else raw_rows[later]) == delimiter), None)
            if end is None:
                break
            out.append((position + 1, cursor + 1, tuple(line.lstrip("\t") if strip_tabs else line for line in raw_rows[cursor:end])))
            cursor = end + 1
    return out


def deferred_heredocs_moved_back(rows, raw_rows):
    """规范文本里「<条件> <<词; then」：正文与收尾那一行挪回条件那一行后面，写成「<条件> <<词」、正文、词、「 then」、then 那一串已经打出来的那几行，
    同这个 if 不在连接符里时 bash 打的样子。正文从原文取（规范文本里紧贴收尾那一行之前、逐行对得上的最长一份）；对不上的不动。"""
    if not any(SHELL_DEFERRED_HEREDOC.match(row) for row in rows):
        return rows
    bodies = raw_heredoc_bodies(raw_rows)
    out, position = [], 0
    while position < len(rows):
        row = rows[position]
        found = SHELL_DEFERRED_HEREDOC.match(row)
        close = next((later for later in range(position + 1, len(rows)) if rows[later] == found.group(3)), None) if found else None
        fitting = [] if close is None else [len(body) for body in bodies.get(found.group(3), ())
                                            if position < close - len(body) and rows[close - len(body):close] == body]
        if not fitting:
            out.append(row)
            position += 1
            continue
        first_body = close - max(fitting)
        out.append(found.group(1))
        out.extend(rows[first_body:close + 1])
        out.append(" then")
        out.extend(rows[position + 1:first_body])
        position = close + 1
    return out


def heredoc_mask(rows):
    """规范文本里 here-doc 正文那几行的下标（起止按 <<词 与单独一行的词认；后面找不到收尾那一行的不算）。"""
    mask, pending = set(), []
    for position, row in enumerate(rows):
        if pending:
            mask.add(position)
            delimiter, strip_tabs = pending[0]
            if (row.lstrip("\t") if strip_tabs else row) == delimiter:
                pending.pop(0)
            continue
        for found in HEREDOC_START.finditer(row):
            delimiter, strip_tabs = found.group(3), found.group(1) == "-"
            if any((later.lstrip("\t") if strip_tabs else later) == delimiter for later in rows[position + 1:]):
                pending.append((delimiter, strip_tabs))
    return mask


def rows_inside_quotes(rows, mask):
    """规范文本里起头落在一个还没收尾的引号或命令替换里的那几行（多行的 '…'、$'…'、"…"、$( … )、`…`）的下标；mask 里的 here-doc 正文先抹空。
    函数头与收尾的 } 不在这几行里认：引号里的 awk 程序、内联脚本照原样留着缩进，它们的「    }」与 declare -f 打的函数收尾长得一样。"""
    text = "\n".join("" if position in mask else row for position, row in enumerate(rows))
    starts, offset = [], 0
    for position, row in enumerate(rows):
        starts.append(offset)
        offset += (0 if position in mask else len(row)) + 1
    spans, i, total = [], 0, len(text)
    while i < total:
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "$" and text.startswith("$'", i):
            j = i + 2
            while j < total and text[j] != "'":
                j += 2 if text[j] == "\\" else 1
            spans.append((i, j))
            i = j + 1
            continue
        if character == "'":
            end = text.find("'", i + 1)
            end = total if end < 0 else end
            spans.append((i, end))
            i = end + 1
            continue
        if character == '"':
            end = double_quote_end(text, i)
            spans.append((i, end - 1))
            i = end
            continue
        if text.startswith("$(", i):
            end = matching_close(text, i + 1, "(", ")")
            spans.append((i, end))
            i = end + 1
            continue
        if character == "`":
            end = text.find("`", i + 1)
            end = total if end < 0 else end
            spans.append((i, end))
            i = end + 1
            continue
        i += 1
    inside, cursor = set(), 0
    for opening, closing in spans:
        while cursor < len(starts) and starts[cursor] <= opening:
            cursor += 1
        probe = cursor
        while probe < len(starts) and starts[probe] <= closing:
            inside.add(probe)
            probe += 1
    return inside


SHELL_POSITIONAL_REFERENCE = re.compile(r'"\$\{?([1-9])(?::-)?\}?"|\$\{([1-9])(?::-)?\}|\$([1-9])(?![0-9])')

# 位置参数从这里起就可能变了（shift、set --）或要按轮取（循环）：从头一处起不再代入
SHELL_ARGUMENTS_MOVED = re.compile(r"(?:^|[\s;&|(])(?:shift|while|until|for|select|set\s+--|set\s+-(?=\s))(?=$|[\s;&|)])")
# 项目脚本开头那一句（.claude/scripts/preflight.sh）：preflight 只摘掉 --force，别的参数照次序放回，位置参数没动
SHELL_PREFLIGHT_ARGUMENTS_RESTORED = re.compile(r'^\s*set -- \$\{PREFLIGHT_ARGUMENTS\[@\]\+"\$\{PREFLIGHT_ARGUMENTS\[@\]\}"\};?$')
SHELL_EXIT_ROW = re.compile(r"^\s*exit(?: [^;|&]*)?;?$")
SHELL_DECIDABLE_IF = re.compile(r"^(\s*)if \[\[ (\S+) (==|!=) (\S+) \]\]; then$")
SHELL_DECIDABLE_CASE = re.compile(r"^(\s*)case (\S+) in\s*$")


def substituted_positionals(text, words):
    """脚本体正文里头一处 shift / set -- / 循环之前的 "$1" "${1}" "${1:-}" $1 …（单引号里的不算）代成字面参数：
    引号外的写成 shlex.quote 的词，双引号里的照双引号的转义写；参数没给的代成空串。
    words 以 SHELL_OPEN_TAIL 收尾的（开口的一串）：字面词那几个位置照代，后面的位置不知道、照原样留着。"""
    rows = text.split("\n")
    mask = heredoc_mask(rows)
    inside = rows_inside_quotes(rows, mask)
    starts, offset = [], 0
    for row in rows:
        starts.append(offset)
        offset += len(row) + 1
    open_tail = words[-1:] == (SHELL_OPEN_TAIL,)
    known = words[:-1] if open_tail else words
    restored = "--force" not in known and "shell-preflight-arguments-moved" not in active_breaks()

    def moves_arguments(row):
        if restored and SHELL_PREFLIGHT_ARGUMENTS_RESTORED.match(row):
            return False
        return SHELL_ARGUMENTS_MOVED.search(re.sub(r'"(?:[^"\\]|\\.)*"', '""', re.sub(r"'[^']*'", "''", row)))
    # here-doc 正文与引号里的那几行不算（python 的 here-doc 里的 for 不是这份脚本的循环）；一行里引号括着的一截先去掉再找
    cut = next((starts[position] for position, row in enumerate(rows) if position not in mask and position not in inside and moves_arguments(row)), len(text))
    # 空行不进：长度 0 的 span 不让 i 前进，循环出不来。弄坏开关 shell-heredoc-empty-rows-spanned 下收回去，走到那一处抛，不真转死
    keeps_empty_rows = "shell-heredoc-empty-rows-spanned" in active_breaks()
    masked_spans = {starts[position]: starts[position] + len(rows[position]) for position in mask if rows[position] or keeps_empty_rows}
    out, i, quoted = [], 0, False
    while i < cut:
        if i in masked_spans:
            if masked_spans[i] == i:
                raise FunctionUnitsError(f"代位置参数在第 {text.count(chr(10), 0, i) + 1} 行（here-doc 里的空行）不前进：弄坏开关 shell-heredoc-empty-rows-spanned 下才有长度 0 的 span")
            out.append(text[i:masked_spans[i]])
            i = masked_spans[i]
            continue
        character = text[i]
        if character == "\\":
            out.append(text[i:i + 2])
            i += 2
            continue
        if character == "'" and not quoted:
            end = text.find("'", i + 1)
            end = cut - 1 if end < 0 or end >= cut else end
            out.append(text[i:end + 1])
            i = end + 1
            continue
        found = SHELL_POSITIONAL_REFERENCE.match(text, i) if character in "$\"" else None
        if found and not (quoted and found.group(0).startswith('"')):
            position = int(next(group for group in found.groups() if group))
            if open_tail and position > len(known):
                out.append(found.group(0))
                i = found.end()
                continue
            value = known[position - 1] if position <= len(known) else ""
            if quoted:
                out.append(re.sub(r'([\\"$`])', r"\\\1", value))
            else:
                out.append(shlex.quote(value))
            i = found.end()
            continue
        if character == '"':
            quoted = not quoted
        out.append(character)
        i += 1
    return "".join(out) + text[cut:]


def pruned_decided_branches(rows, undecided_dropped=False):
    """代过参数之后判得出真假的 if [[ <字面> == / != <字面> ]]; then … fi（不带 elif；带 else 的留判得出的那一支）与 case <字面> in …
    （分支的模式都是字面、没有 ;& ;;&：只留头一个对得上的分支，判不出的那个分支起照留）：走不到的那几行抹空（行数不变）。
    脚本体顶层（缩进 4）的 if 走到的那一支以 exit 收尾：fi 之后的整段都走不到，一并抹空（弄坏开关 shell-exit-branches-followed 下不抹）。
    undecided_dropped 只给弄坏开关 shell-entry-undecided-branches-dropped（ScriptClosure.run 的 shell 入口）：判不出真假的 if [[ … ]] 当不成立。"""
    rows = list(rows)
    for start, row in enumerate(rows):
        found = SHELL_DECIDABLE_IF.match(row)
        if found:
            left, right = literal_word(found.group(2)), literal_word(found.group(4))
            undecided = left is None or right is None or (found.group(4)[:1] not in "'\"" and any(mark in right for mark in "*?["))
            if undecided and not undecided_dropped:
                continue
            indent = found.group(1)
            other, end = None, None
            for later in range(start + 1, len(rows)):
                if rows[later].startswith(indent + "elif ") and not rows[later].startswith(indent + " "):
                    other = "elif"
                    break
                if rows[later] == indent + "else":
                    other = later
                elif rows[later] in (indent + "fi", indent + "fi;"):
                    end = later
                    break
            if end is None or other == "elif":
                continue
            holds = not undecided and (left == right) == (found.group(3) == "==")
            dead = range(other + 1, end) if holds and other is not None else range(start + 1, other if other is not None else end) if not holds else ()
            for position in dead:
                rows[position] = ""
            taken = (range(start + 1, other if other is not None else end) if holds else range(other + 1, end) if other is not None else ())
            last = next((rows[position] for position in reversed(taken) if rows[position].strip()), "")
            if indent == "    " and last.startswith(indent + "    ") and not last.startswith(indent + "     ") and SHELL_EXIT_ROW.match(last) \
                    and "shell-exit-branches-followed" not in active_breaks():
                for position in range(end + 1, len(rows)):
                    rows[position] = ""
            continue
        found = SHELL_DECIDABLE_CASE.match(row)
        if not found or literal_word(found.group(2)) is None:
            continue
        subject, branch_indent = literal_word(found.group(2)), found.group(1) + "    "
        branches, end, current = [], None, None
        for later in range(start + 1, len(rows)):
            text = rows[later]
            if text.rstrip(";") == found.group(1) + "esac":
                end = later
                break
            if text in (branch_indent + ";&", branch_indent + ";;&"):
                branches = None
                break
            if text == branch_indent + ";;":
                current = None
                continue
            if current is None and text.startswith(branch_indent) and not text.startswith(branch_indent + " ") and text.endswith(")"):
                current = [later, later + 1, text[len(branch_indent):-1]]
                branches.append(current)
                continue
            if current is not None:
                current[1] = later + 1
        if end is None or not branches:
            continue
        matched = False
        for head, stop, patterns in branches:
            if matched:
                for position in range(head + 1, stop):
                    rows[position] = ""
                continue
            alternatives = [literal_word(alternative.strip()) for alternative in patterns.lstrip("(").split("|")]
            if None in alternatives:
                break
            if any(fnmatch.fnmatchcase(subject, alternative) for alternative in alternatives):
                matched = True
            else:
                for position in range(head + 1, stop):
                    rows[position] = ""
    return rows


def narrowed_shell_body(text, words, undecided_dropped=False):
    """shell 起法带字面参数（文件头「shell 起法带字面参数」）：脚本体正文代进参数（substituted_positionals）、抹掉判得出走不到的分支
    （pruned_decided_branches；undecided_dropped 照它）。弄坏开关 shell-program-arguments-unnarrowed 下不剪（ScriptClosure.push_program）。"""
    substituted = substituted_positionals(text, words)
    return "\n".join(pruned_decided_branches(substituted.split("\n"), undecided_dropped))


# 按第一列选行的表（文件头「shell 起法带字面参数」）：<脚本> → 它的表的变量名。带字面的实验号起时只跑第一列是这几个号的行，别的行写的驱动函数走不到
SHELL_ROW_SELECTED_TABLES = {"scripts/experiment-apparatus/replay.sh": ("TABLE", "ADMISSION_ROW_KEYS")}
SHELL_TABLE_ASSIGNMENT = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)=\$\(cat <<'([A-Za-z_][A-Za-z0-9_]*)'$")


SHELL_TABLE_ROW_DRIVER = re.compile(r"@([A-Za-z_][A-Za-z0-9_]*)")


def table_data_rows(rows, tables):
    """正文（按行）里 tables 点名的那几张表（<名>=$(cat <<'词' …）的数据行：(下标, 表名)，按次序。"""
    position = 0
    while position < len(rows):
        found = SHELL_TABLE_ASSIGNMENT.match(rows[position])
        position += 1
        if not found or found.group(1) not in tables:
            continue
        while position < len(rows) and rows[position] != found.group(2):
            yield position, found.group(1)
            position += 1


def table_row_key(row):
    """表行的第一列：到 | / 制表符为止。"""
    return re.split(r"[|\t/]", row, maxsplit=1)[0]


def table_row_identity(row, table, tables):
    """表行的身份：replay.sh 的 ROW_ORDINAL 认的「实验号|二进制|参数」（复跑表的前三列）。tables 头一张（复跑表）的行取它自己的前三列，
    别的表（准入键表 ADMISSION_ROW_KEYS）的行取制表符分隔的第二列（那一列原样抄复跑表那一行的前三列）；没有第二列的交 None。"""
    if table == tables[0]:
        return "|".join((row.split("|") + ["", ""])[:3])
    fields = row.split("\t")
    return fields[1] if len(fields) > 1 else None


def table_row_selection(words):
    """起法的词 → (点名的实验号集合, {被 --item 点到的实验号: 点名的行序号集合})，照 replay.sh 读参数的写法：--item 后面那个词是行项
    <实验号>.<行序号>（实验号取最后一个 . 之前、行序号取它之后，同 ${item%.*} 与 ${item##*.}；没有 . 的整个词既当实验号又当行序号），
    别的不带 - 起头的词是实验号。两样都空的是不带参数、只带选项的起法。
    弄坏开关 shell-table-row-items-unmatched 下行项照实验号认（整个词拿去比第一列，哪一行都对不上，那几行的驱动不连）。"""
    experiments, items, expecting = set(), {}, False
    unmatched = "shell-table-row-items-unmatched" in active_breaks()
    for word in words:
        if expecting:
            expecting = False
            if not unmatched:
                experiment, dot, ordinal = word.rpartition(".")
                items.setdefault(experiment if dot else word, set()).add(ordinal)
                continue
        elif word == "--item":
            expecting = True
            continue
        if not word.startswith("-"):
            experiments.add(word)
    return experiments, items


def selected_row_identities(reference_rows, tables, items):
    """被 --item 点到的行的身份（table_row_identity）：数 reference_rows 里复跑表（tables 头一张）的行，同 replay.sh 读复跑表的写法——
    第一列以 E 起头的才数，每个实验号从 1 数起；同一个身份写了几行的取最后那一行的序号（ROW_ORDINAL 一个身份记一个序号）。"""
    ordinal_of, counts = {}, collections.Counter()
    for position, table in table_data_rows(reference_rows, tables[:1]):
        row = reference_rows[position]
        experiment = row.split("|", 1)[0]
        if not experiment.startswith("E"):
            continue
        counts[experiment] += 1
        ordinal_of[table_row_identity(row, table, tables)] = str(counts[experiment])
    return {identity for identity, ordinal in ordinal_of.items() if ordinal in items.get(identity.split("|", 1)[0], ())}


def table_rows_selected(rows, tables, words, reference_rows=None):
    """起法的词 → rows 里 tables 那几张表这一趟跑得到的数据行（下标的集合）；一个实验号、一个行项都没点名的（不带参数、只带选项）交 None，整张表照跑。
    照 replay.sh 的 want_row：没被 --item 点到的实验号那几行都跑，被点到的实验号只跑点名的那几行（身份对上 selected_row_identities 的）。
    行序号从 reference_rows（不给就是 rows）里的复跑表数：准入键表的变量单元里没有复跑表，调用方交整份脚本体。"""
    experiments, items = table_row_selection(words)
    if not experiments and not items:
        return None
    identities = selected_row_identities(rows if reference_rows is None else reference_rows, tables, items) if items else set()
    selected = set()
    for position, table in table_data_rows(rows, tables):
        key = table_row_key(rows[position])
        if key in items:
            if table_row_identity(rows[position], table, tables) in identities:
                selected.add(position)
        elif key in experiments:
            selected.add(position)
    return selected


def rows_of_selected_keys(text, tables, words, reference=None):
    """脚本体正文里 tables 点名的那几张表：这一趟跑不到的行（table_rows_selected）抹空；不带参数、只带选项的起法整张表照跑，不抹。
    reference 是数行序号用的整份脚本体（不给就是 text）。"""
    rows = text.split("\n")
    selected = table_rows_selected(rows, tables, words, None if reference is None else reference.split("\n"))
    if selected is None:
        return text
    for position, _table in table_data_rows(rows, tables):
        if position not in selected:
            rows[position] = ""
    return "\n".join(rows)


def table_drivers_of_unselected_rows(text, tables, launches):
    """脚本体正文里 tables 点名的那几张表：只写在每一次起法（launches，一次起法一组词）都跑不到的那几行的 @驱动函数
    （有一次起法跑得到的行也写到的不算；弄坏开关 shell-row-driver-selected-rows-dropped 下也算）。整张表照跑的那一次起法这里不算它跑得到哪一行
    （调用方 ScriptClosure.table_unselected_drivers 在那种情形下不来问，只有弄坏开关 shell-row-driver-whole-launch-dropped 下照问）。"""
    rows = text.split("\n")
    reached = set()
    for words in launches:
        reached |= table_rows_selected(rows, tables, words) or set()
    other, named = set(), set()
    for position, _table in table_data_rows(rows, tables):
        (named if position in reached else other).update(SHELL_TABLE_ROW_DRIVER.findall(rows[position]))
    if "shell-row-driver-selected-rows-dropped" in active_breaks():
        return frozenset(other)
    return frozenset(other - named)


def written_only_as_table_row(text, name):
    """text 里写到 name 的每一处都是 @<名>（表行的身份）或 <名>()（定义），没有一处能起它。词的边界同 SHELL_WORDS。
    弄坏开关 shell-row-driver-plain-words-dropped 下不带 @ 的词也算。"""
    if "shell-row-driver-plain-words-dropped" in active_breaks():
        return True
    return re.search(rf"(?<![@A-Za-z0-9_.:-]){re.escape(name)}(?![A-Za-z0-9_.:-]|\s*\(\))", text) is None


def shell_launch_prefix(words):
    """bash <脚本> 后面的词的字面值（折不出的是 None）→ 这一处起法（文件头「shell 起法带字面参数」）：全字面的原样；折不出的词之前有字面词的，字面词接 SHELL_OPEN_TAIL；
    头一个就折不出 → None（那份脚本体整份收）。弄坏开关 shell-open-tail-unnarrowed 下有一个折不出就 None。"""
    if None not in words:
        return words
    known = words[:words.index(None)]
    return known + (SHELL_OPEN_TAIL,) if known and "shell-open-tail-unnarrowed" not in active_breaks() else None


# 命令位置上的词前面那一截（word_line 找拒算行用）：命令分隔与 then / do / else 之后，隔着空白与一个双引号
COMMAND_POSITION_BEFORE_WORD = r'(?:&&|\|\||[;|({!]|\bthen|\bdo|\belse)\s*"?'
# env 数组里的一项赋值（NAME=值，值带不带引号都算）：数组头一个词是 env 时这些项是 env 的赋值，不是命令词
SHELL_ENV_ARRAY_ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
SHELL_QUOTED_ASSIGNMENT = re.compile(r'^"[A-Za-z_][A-Za-z0-9_]*=')


def shell_position_values(words):
    """command_positions 用的逐词字面值：折不出而整个词是带引号的 NAME=…（env "X=$y" nice … 的一条赋值）占成 NAME=…，命令位置接着往后认（同 python 一侧 environment_assignment_shape）。
    弄坏开关 shell-quoted-assignments-unread 下照 literal_word。"""
    values = [literal_word(word) for word in words]
    if "shell-quoted-assignments-unread" in active_breaks():
        return values
    return [word[1:].split("=", 1)[0] + "=…" if value is None and SHELL_QUOTED_ASSIGNMENT.match(word) else value for word, value in zip(words, values)]


def shell_known_words(words):
    """python3 <脚本> 后面的词（规范文本里的写法）→ KnownWords（文件头「shell 起 python」）：字面的是那个词，整个在一对双引号里、里面不是 $@、$*、[@]、[*] 的是不知道的一个词（None），
    别的（不带引号的展开、拼着展开的、带通配的）从那里起是开口的一串。"""
    known = []
    for word in words:
        literal = literal_word(word)
        if literal is not None and (word[:1] in ("'", '"') or not any(mark in literal for mark in "*?[")):
            known.append(literal)
        elif len(word) >= 2 and word[0] == word[-1] == '"' and '"' not in word[1:-1] and not any(mark in word for mark in ("$@", "$*", "[@]", "[*]")):
            known.append(None)
        else:
            return KnownWords(tuple(known), False)
    return KnownWords(tuple(known), True)


def heredoc_bodies(rows):
    """规范文本里每个 here-doc → {(<<词 所在行, 所在列): (词带不带引号, 正文)}；起止的认法与 heredoc_mask 同一份（找不到收尾那一行的不算）。"""
    bodies, pending = {}, []
    for position, row in enumerate(rows):
        if pending:
            place, delimiter, strip_tabs, quoted, lines = pending[0]
            if (row.lstrip("\t") if strip_tabs else row) == delimiter:
                bodies[place] = (quoted, "\n".join(lines))
                pending.pop(0)
            else:
                lines.append(row)
            continue
        for found in HEREDOC_START.finditer(row):
            delimiter, strip_tabs = found.group(3), found.group(1) == "-"
            if any((later.lstrip("\t") if strip_tabs else later) == delimiter for later in rows[position + 1:]):
                pending.append(((position, found.start()), delimiter, strip_tabs, bool(found.group(2)), []))
    return bodies


def command_inputs(details, base, visible, bodies):
    """split_commands 记的重定向细目 → {命令下标: [(文件描述符号, 写法, 内容)…]}，只收读进来的那几种：
    file（< 词：内容是目标词）、herestring（<<< 词）、heredoc（<<词：内容是 (带不带引号, 正文)，正文对不上是 None）。
    base 是这一段在 visible（整个单元的规范文本，here-doc 正文已抹空）里的起点，用来找 <<词 落在哪一行哪一列。"""
    out = collections.defaultdict(list)
    for index, operator, target, offset, descriptor in details:
        number = 0 if descriptor is None else descriptor
        if operator in ("<<", "<<-"):
            absolute = base + offset
            row = visible.count("\n", 0, absolute)
            column = absolute - (visible.rfind("\n", 0, absolute) + 1)
            out[index].append((number, "heredoc", bodies.get((row, column))))
        elif operator == "<<<":
            out[index].append((number, "herestring", target))
        elif operator == "<":
            out[index].append((number, "file", target))
    return out


def substitutions(text):
    """text 里最外一层的命令替换 $( … ) 与 `…` → [(里面的文本, 它在 text 里起头的下标)…]：双引号里的也算，单引号里的、$(( 算术 )) 与注释不算。"""
    out, i, total = [], 0, len(text)
    while i < total:
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            i = total if end < 0 else end + 1
            continue
        if character == "#" and (i == 0 or text[i - 1] in " \t\n;&|("):
            end = text.find("\n", i)
            i = total if end < 0 else end
            continue
        if character == '"':
            j = i + 1
            while j < total and text[j] != '"':
                if text[j] == "\\":
                    j += 2
                    continue
                if text.startswith("$((", j):
                    j = matching_close(text, j + 1, "(", ")") + 1
                    continue
                if text.startswith("$(", j):
                    end = matching_close(text, j + 1, "(", ")")
                    out.append((text[j + 2:end], j + 2))
                    j = end + 1
                    continue
                if text[j] == "`":
                    end = text.find("`", j + 1)
                    if end < 0:
                        break
                    out.append((text[j + 1:end], j + 1))
                    j = end + 1
                    continue
                j += 1
            i = j + 1
            continue
        if text.startswith("$((", i):
            i = matching_close(text, i + 1, "(", ")") + 1
            continue
        if text.startswith("$(", i):
            end = matching_close(text, i + 1, "(", ")")
            out.append((text[i + 2:end], i + 2))
            i = end + 1
            continue
        if character == "`":
            end = text.find("`", i + 1)
            if end < 0:
                break
            out.append((text[i + 1:end], i + 1))
            i = end + 1
            continue
        i += 1
    return out


def matching_close(text, start, opening, closing):
    """text[start] 是 opening；回配对的 closing 的下标（跳过引号），配不上回 len(text) - 1。"""
    depth, i = 0, start
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'" and opening != "{":
            end = text.find("'", i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if character == '"':
            i = double_quote_end(text, i)
            continue
        if character == opening:
            depth += 1
        elif character == closing:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return len(text) - 1


def double_quote_end(text, start):
    i = start + 1
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == '"':
            return i + 1
        if text.startswith("$(", i):
            i = matching_close(text, i + 1, "(", ")") + 1
            continue
        if text.startswith("${", i):
            i = matching_close(text, i + 1, "{", "}") + 1
            continue
        i += 1
    return len(text)


# 进程替换 <( … ) / >( … ) 在外面那条命令的词里占的那个词（运行期 bash 换成 /dev/fd/N）
PROCESS_SUBSTITUTION_WORD = "/dev/fd/63"


def split_commands(text, redirect_targets_as_commands=False, redirects=None, details=None):
    """规范文本 → [[词…]…]：按 ; && || | & ( ) 与换行切成简单命令，词里引号与 $(…) ${…} 原样留着；重定向和它的目标丢掉
    （重定向的目标是写数据、读数据的路径，不是装代码的位置）。redirect_targets_as_commands 只给弄坏开关
    shell-redirect-targets-loaded 用：目标单独当一条命令留下。redirects 给了一个列表的，每个重定向往里记
    （它属于第几条命令、写法如 > >> <<、目标词）；details 给了一个列表的，同样每个重定向记一条（第几条命令、写法、目标词、
    写法在 text 里起头的下标、前面写的文件描述符号或 None）。<( … ) 与 >( … ) 是进程替换，不是重定向：括号里的照命令切。
    不是命令的几段不切成命令：[[ … ]] 与 (( … )) 整段只留 [[ ]] / ((（里面的 || ( ) < 不是分隔），case 分支的模式（a | b) 那一段）跳过，
    数组字面量 x=( … ) 整个留在赋值词里（括号里的是元素，不是命令）。"""
    commands, words, word = [], [], []
    redirect_target = False
    redirect_operator = ""
    redirect_start, redirect_descriptor = 0, None
    i, total = 0, len(text)
    structured = "shell-command-structure-ignored" not in active_breaks()
    cases = []   # 每层 case 此刻在认什么：pattern（分支的模式）/ body（分支里的命令）
    # 进程替换括号里的命令：外面那条命令收尾之后再排进去（重定向记的「第几条命令」不因此错位）；
    # 每项 (括号里切出的命令, 它们的重定向, 重定向的细节, 括号里那段在 text 里起头的下标)，排进去时把「第几条命令」与下标换算到外面这一份上
    pending = []

    def end_word():
        nonlocal redirect_target
        if word:
            if redirect_target:
                redirect_target = False
                if redirects is not None:
                    redirects.append((len(commands), redirect_operator, "".join(word)))
                if details is not None:
                    details.append((len(commands), redirect_operator, "".join(word), redirect_start, redirect_descriptor))
                if redirect_targets_as_commands:
                    commands.append(["".join(word)])
            else:
                words.append("".join(word))
            word.clear()

    def end_command():
        end_word()
        if words:
            if structured and words[0] == "case" and words[-1] == "in":
                cases.append("pattern")
            elif structured and words == ["esac"] and cases:
                cases.pop()
            commands.append(list(words))
            words.clear()
        for inner_commands, inner_redirects, inner_details, offset in pending:
            base = len(commands)
            commands.extend(inner_commands)
            if "shell-process-substitution-redirects-dropped" in active_breaks():
                continue
            if redirects is not None:
                redirects.extend((base + index, operator, target) for index, operator, target in inner_redirects)
            if details is not None:
                details.extend((base + index, operator, target, offset + start, descriptor) for index, operator, target, start, descriptor in inner_details)
        pending.clear()

    while i < total:
        character = text[i]
        if structured and cases and cases[-1] == "pattern" and not words and not word:
            # case 分支的模式：跳到它的 ) 之后；碰到 esac 收这一层
            if character in " \t\n;":
                i += 1
                continue
            if text.startswith("esac", i) and (i + 4 >= total or text[i + 4] in " \t\n;&|)"):
                cases.pop()
                i += 4
                continue
            i = case_pattern_end(text, i) + 1
            cases[-1] = "body"
            continue
        if structured and not word and all(item in SHELL_KEYWORDS or item == "for" for item in words):
            if text.startswith("[[", i) and (i + 2 >= total or text[i + 2] in " \t\n"):
                words += ["[[", "]]"]
                i = test_command_end(text, i + 2)
                continue
            if text.startswith("((", i):
                words.append("((")
                i = matching_close(text, i, "(", ")") + 1
                continue
        if structured and character == ";" and cases and text.startswith((";;", ";&"), i):
            end_command()
            i += 3 if text.startswith(";;&", i) else 2
            cases[-1] = "pattern"
            continue
        if structured and character == "(" and word and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*(?:\[[^\]]*\])?\+?=", "".join(word)):
            end = matching_close(text, i, "(", ")")
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if (character == "[" and word and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", "".join(word))
                and "shell-subscript-assignment-split" not in active_breaks()):
            end = matching_close(text, i, "[", "]")   # 名字[下标]=值、名字[下标]+=值：下标里有空格也是一个赋值词（bash 照赋值认）
            if text.startswith(("=", "+="), end + 1):
                word.append(text[i:end + 1])
                i = end + 1
                continue
        if character == "\\" and i + 1 < total:
            if text[i + 1] == "\n":
                i += 2
                continue
            word.append(text[i:i + 2])
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            end = total - 1 if end < 0 else end
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == '"':
            end = double_quote_end(text, i)
            word.append(text[i:end])
            i = end
            continue
        if character == "$" and text.startswith("$(", i):
            end = matching_close(text, i + 1, "(", ")")
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == "$" and text.startswith("${", i):
            end = matching_close(text, i + 1, "{", "}")
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character == "`":
            end = text.find("`", i + 1)
            end = total - 1 if end < 0 else end
            word.append(text[i:end + 1])
            i = end + 1
            continue
        if character in " \t":
            end_word()
            i += 1
            continue
        if character in "\n;&|()":
            end_command()
            i += 1
            continue
        if character in "<>":
            if character in "<>" and text.startswith("(", i + 1) and "shell-substitutions-ignored" not in active_breaks():
                # 进程替换 <( … ) / >( … )：括号里是要跑的命令，照命令切；
                # 「< <( … )」前面那个 < 的目标就是这个进程替换，不是括号里的第一个词
                if "shell-process-substitution-splits-command" in active_breaks():
                    # 弄坏开关：左括号在下一圈当分隔，外面那条命令在这里断开
                    end_word()
                    redirect_target = False
                    i += 1
                    continue
                # 括号里的命令各成一条、排在外面那条命令之后；外面那条命令接着认：进程替换在它的词里占一个词（运行期是 /dev/fd/N），
                # 当重定向目标的不占词
                was_target = redirect_target and not word
                end_word()
                redirect_target = False
                close = matching_close(text, i + 1, "(", ")")
                inner_redirects, inner_details = [], []
                pending.append((split_commands(text[i + 2:close], redirect_targets_as_commands, inner_redirects, inner_details), inner_redirects, inner_details, i + 2))
                if not was_target:
                    words.append(PROCESS_SUBSTITUTION_WORD)
                i = close + 1
                continue
            redirect_descriptor = None
            if word and "".join(word).isdigit():
                redirect_descriptor = int("".join(word))
                word.clear()
            else:
                end_word()
            start = i
            while i < total and text[i] in "<>&|-":
                i += 1
            redirect_operator = text[start:i]
            redirect_start = start
            while i < total and text[i] in " \t":
                i += 1
            redirect_target = True
            continue
        if character == "#" and not word:
            end = text.find("\n", i)
            i = total if end < 0 else end
            continue
        word.append(character)
        i += 1
    end_command()
    if structured and details is not None:
        # { … } 3<<'词'：写在收尾那个 } 后面的重定向管着组里的每一条命令
        groups, copies = [], []
        for index, command in enumerate(commands):
            if command and command[0] == "{":
                groups.append(index)
            elif command and command[0] == "}" and groups:
                start = groups.pop()
                for detail in details:
                    if detail[0] == index:
                        copies += [(inner,) + tuple(detail[1:]) for inner in range(start, index)]
        details.extend(copies)
    return commands


def case_pattern_end(text, start):
    """case 分支模式（text[start] 起，可带一个开头的左括号）收尾的那个 ) 的下标；找不到回末尾。"""
    i = start + 1 if text.startswith("(", start) else start
    depth = 0
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if character == '"':
            i = double_quote_end(text, i)
            continue
        if character == "(":
            depth += 1
        elif character == ")":
            if depth == 0:
                return i
            depth -= 1
        i += 1
    return len(text) - 1


def test_command_end(text, start):
    """[[ … ]]：从 start 起找收尾的 ]]（前面是空白、后面是分隔或末尾），回它之后的下标；引号、$( … ) ${ … } 跳过；到换行就停。"""
    i = start
    while i < len(text):
        character = text[i]
        if character == "\\":
            i += 2
            continue
        if character == "'":
            end = text.find("'", i + 1)
            i = len(text) if end < 0 else end + 1
            continue
        if character == '"':
            i = double_quote_end(text, i)
            continue
        if text.startswith("$(", i):
            i = matching_close(text, i + 1, "(", ")") + 1
            continue
        if text.startswith("${", i):
            i = matching_close(text, i + 1, "{", "}") + 1
            continue
        if text.startswith("]]", i) and text[i - 1] in " \t" and (i + 2 >= len(text) or text[i + 2] in " \t\n;&|)"):
            return i + 2
        if character == "\n":
            return i
        i += 1
    return len(text)


def git_repository_location(words, root, breaks):
    """git … rev-parse … --show-toplevel / --git-common-dir 的输出 → {根} / {根/.git}；别的 git 用法 → None。
    words 是逐个词的字面值（折不出的词是 None，只要 git、rev-parse 与那个选项是字面的就认）。
    根是 worktree 时运行期 --git-common-dir 指的是主仓：库只看给它的这一个根，按根里同一相对路径取，那一份不在照 missing 拒算。"""
    if "git-repository-paths-unfolded" in breaks or not words or words[0] != "git" or "rev-parse" not in words:
        return None
    if "--show-toplevel" in words:
        return {root}
    if "--git-common-dir" in words:
        return {posixpath.join(root, ".git")}
    return None


TREE_COPY_TAIL = re.compile(r"^[A-Za-z0-9_.+@,=-]+(?:/[A-Za-z0-9_.+@,=-]+)+$")


def remove_shell_pattern(value, pattern, operator):
    """${名%词} ${名%%词} ${名#词} ${名##词}：照 bash 去掉最短 / 最长的后缀或前缀（词照 shell 通配认，同 fnmatch）；对不上交原值。"""
    cuts = range(len(value), -1, -1) if operator in ("%", "##") else range(len(value) + 1)
    for cut in cuts:
        if operator.startswith("%") and fnmatch.fnmatchcase(value[cut:], pattern):
            return value[:cut]
        if operator.startswith("#") and fnmatch.fnmatchcase(value[:cut], pattern):
            return value[cut:]
    return value


def leading_command_words(command):
    """一条命令去掉开头的关键字（if、then、!…）与 NAME=VALUE 前缀之后的那几个词。"""
    words = list(command)
    while words and words[0] in SHELL_KEYWORDS:
        words = words[1:]
    while words and ASSIGNMENT_WORD.match(words[0]):
        words = words[1:]
    return words


def in_skipped_directory(relative):
    """相对根的文件路径经过 SKIPPED_DIRECTORIES 里的目录（编译产物 target、.git、缓存）。"""
    return any(part in SKIPPED_DIRECTORIES for part in relative.split("/")[:-1])


def tree_copy_file(root, tail, directory=False):
    """「<折不出的目录>/<字面尾巴>」→ 尾巴里最长的、至少两段、在根里存在的那一截（相对根）；没有 → None。
    临时树、仓副本、worktree 里的同一相对路径按根里那一份算：拷进去的是它，替身也只会让闭包多收不会少收。"""
    tail = tail.lstrip("/")
    if not TREE_COPY_TAIL.match(tail):
        return None
    parts = tail.split("/")
    if any(part in (".", "..") for part in parts):
        return None
    for start in range(len(parts) - 1):
        candidate = "/".join(parts[start:])
        absolute = os.path.join(root, candidate)
        if (os.path.isdir(absolute) if directory else os.path.isfile(absolute)):
            return candidate
    return None


INLINE_POSITIONAL = re.compile(r"\$(?:\{([1-9])\}|([1-9]))")


def inline_positionals(inline, rest):
    """bash -c '<串>' <$0> <$1>…：串里外层 bash 会展开的 $1…$9 换成外面那几个词（去掉外层双引号，留在串里原来的引号里；
    外面没给的、带单引号的照原样留着，照旧折不出）。外层不展开的不换：反斜杠转义的（\\$1，串里套着的里层 bash -c "…\\$1…" 留给里层，
    里层照它自己的参数再换一次）与单引号里的（双引号外的 '…$1…'）照原样留着。
    弄坏开关 shell-inline-positionals-escapes-ignored：转义的与单引号里的也换（里层的 \\$1 换成外层的 $1）。"""
    def replacement(found):
        position = int(found.group(1) or found.group(2))
        if position >= len(rest) or "'" in rest[position]:
            return found.group(0)
        return rest[position].replace('"', "")
    if "shell-inline-positionals-escapes-ignored" in active_breaks():
        return INLINE_POSITIONAL.sub(replacement, inline)
    out, i, quoted, total = [], 0, False, len(inline)
    while i < total:
        character = inline[i]
        if character == "\\" and i + 1 < total:
            out.append(inline[i:i + 2])
            i += 2
            continue
        if character == "'" and not quoted:
            end = inline.find("'", i + 1)
            end = total - 1 if end < 0 else end
            out.append(inline[i:end + 1])
            i = end + 1
            continue
        if character == '"':
            quoted = not quoted
        found = INLINE_POSITIONAL.match(inline, i) if character == "$" else None
        if found:
            out.append(replacement(found))
            i = found.end()
            continue
        out.append(character)
        i += 1
    return "".join(out)


SHELL_TREE_PREFIX = re.compile(r"^(?:\$\{(?:[A-Za-z_][A-Za-z0-9_]*|[1-9])(?:[:]?[-=?][^{}]*)?\}|\$(?:[A-Za-z_][A-Za-z0-9_]*|[1-9]))(/.+)$")


def literal_word(word):
    """没有展开的词 → 去掉引号后的字面值；有 $ ` 展开的 → None。双引号里的单引号是字面的，双引号里的反斜杠只在 $ ` " \\ 前面才转义。"""
    out, i, quoted = [], 0, False
    while i < len(word):
        character = word[i]
        if character == "'" and not quoted:
            end = word.find("'", i + 1)
            if end < 0:
                return None
            out.append(word[i + 1:end])
            i = end + 1
            continue
        if character == '"':
            quoted = not quoted
            i += 1
            continue
        if character in "$`":
            return None
        if character == "\\" and i + 1 < len(word):
            if quoted and word[i + 1] not in '$`"\\\n':
                out.append(character)
                i += 1
                continue
            out.append(word[i + 1])
            i += 2
            continue
        out.append(character)
        i += 1
    return "".join(out)


# find 清单（find_listing_pattern）后面许接的那一条 sort 的选项：只改次序、去重，不改行
FIND_LISTING_SORT_OPTIONS = frozenset(("-u", "-r", "-V"))
# find 清单里许写的 -type 取值（只滤掉行）
FIND_LISTING_TYPES = frozenset(("f", "l"))
# 引号里的一段与反斜杠转义的一个字（find_listing_pattern 数分隔符之前去掉）
QUOTED_OR_ESCAPED = re.compile(r"'[^']*'|\"(?:[^\"\\]|\\.)*\"|\\.")


def find_listing_pattern(word, breaks):
    """一个赋值词是不是一份 find 清单：整个词是 "$(find <起点> <测试…> [| sort [-u|-r|-V]])"，测试里恰好一个写在顶层的 -name <字面模式>（模式里没有 /），
    别的只有 -maxdepth / -mindepth <数>、-type f / l、-print 与只装 -type、-o 的一对括号（\\( -type f -o -type l \\)）——这几样只滤掉行、不加行也不改行，
    清单每一行的文件名（最后一段）都合那个模式 → 那个模式；别的（顶层的 -o、! / -not、-path、-exec、括号里的 -name、管道里接了别的命令、;、&、换行）→ None。
    起点不看：读的那一处只认 os.path.basename 取出的文件名（PythonModuleDataReads.listing_element），在哪个目录里列的不改它合不合模式。
    弄坏开关 shell-find-listing-operators-unchecked 下只认 -name，别的测试、括号与顶层的 -o 一概不看。"""
    text = word.strip()
    if len(text) >= 2 and text.startswith('"') and text.endswith('"'):
        text = text[1:-1]
    if not text.startswith("$(") or text.startswith("$((") or not text.endswith(")") or matching_close(text, 1, "(", ")") != len(text) - 1:
        return None
    inner = text[2:-1]
    bare = QUOTED_OR_ESCAPED.sub("", inner)
    commands = split_commands(inner)
    if not commands or len(commands) > 2 or re.search(r"[;&\n]|\|\|", bare) or bare.count("|") != len(commands) - 1:
        return None
    if len(commands) == 2:
        sorting = [literal_word(item) for item in commands[1]]
        if sorting[0] != "sort" or any(item not in FIND_LISTING_SORT_OPTIONS for item in sorting[1:]):
            return None
    words = [literal_word(item) for item in commands[0]]
    if len(words) < 2 or words[0] != "find":
        return None
    unchecked = "shell-find-listing-operators-unchecked" in breaks
    patterns, grouped, position = [], False, 2
    while position < len(words):
        item = words[position]
        following = words[position + 1] if position + 1 < len(words) else None
        if item == "-name" and following is not None and (not grouped or unchecked):
            patterns.append(following)
            position += 2
        elif unchecked:
            position += 1
        elif item in ("-maxdepth", "-mindepth") and following is not None and following.isdigit():
            position += 2
        elif item == "-type" and following in FIND_LISTING_TYPES:
            position += 2
        elif (item == "-print" and not grouped) or (item == "-o" and grouped) or (item == "(" and not grouped) or (item == ")" and grouped):
            grouped = not grouped if item in ("(", ")") else grouped
            position += 1
        else:
            return None
    if (grouped and not unchecked) or len(patterns) != 1 or "/" in patterns[0]:
        return None
    return patterns[0]


class ShellFacts:
    """invocations：[(写法, 词, cd 过去的目录词或 None, 附带)]，写法 source / shell / python / exec / python-module / python-inline / shell-inline /
    inline-unknown（附带见 command_facts）；handled：每个解释器词各记一条它按哪一种写法认的（只给量全仓的人看，不进判定）。
    prefixed：命令前的 X=v（后面跟着一条命令的那种）{名字: [(赋值词, 那条命令的头一个词或 None)]}，同一个赋值词也照旧记在 assignments 里；
    ShellIndex.assignment_sites 照它认「只管那一次调用」（头一个词是函数名、那个函数走不到读它的那个函数的不算）。"""
    __slots__ = ("commands", "words", "command_words", "variable_references", "assignments", "local_names", "invocations", "programs", "directory",
                 "generated", "handled", "exported", "prefixed")

    def __init__(self):
        self.commands, self.words, self.variable_references, self.generated = [], set(), set(), {}
        self.command_words, self.assignments, self.invocations, self.programs = set(), collections.defaultdict(list), [], set()
        self.local_names, self.directory, self.handled, self.exported = set(), None, [], set()
        self.prefixed = collections.defaultdict(list)


GENERATOR_HEADS = frozenset(("printf", "echo"))
COPYING_HEADS = frozenset(("cp", "mv", "install", "ln", "rsync", "tee", "dd", "touch"))


def generated_scripts(commands, redirects, rows, mask):
    """单元自己写出来、拷出来的脚本：目标词（去掉双引号）→ [这个单元写进去的 here-doc 正文…]，或 ("copied", [源词…])。
    写出来的：写它的每一处都得是 printf / echo 的输出重定向，或不带别的参数的 cat 接 here-doc；有一处别的写法（tee、别的命令重定向、cp）就不算。
    printf / echo 写进去的那几行不当代码看（格式串里认不出哪是代码）；cat <<词 的正文照代码看。
    拷出来的：写它的只有「cp <源> <它>」（两个操作数），代码就是源那一份。"""
    writers = collections.defaultdict(list)
    for index, operator, target in redirects:
        if not operator.startswith(">") or operator.startswith(">&") and target.isdigit():
            continue
        command = commands[index] if index < len(commands) else []
        head = literal_word(command[0]) if command else None
        operators = {other for position, other, _target in redirects if position == index}
        if head in GENERATOR_HEADS or (head == "cat" and len(command) == 1 and any(other in ("<<", "<<-") for other in operators)):
            writers[target.replace('"', "")].append(head)
        else:
            writers[target.replace('"', "")].append(None)
    copies = collections.defaultdict(list)
    for command in commands:
        head = literal_word(command[0]) if command else None
        if head not in COPYING_HEADS:
            continue
        operands = [argument for argument in command[1:] if not (literal_word(argument) or "").startswith("-")]
        if head == "cp" and len(operands) == 2:
            copies[operands[1].replace('"', "")].append(operands[0])
            continue
        for argument in command[1:]:
            writers[argument.replace('"', "")].append(None)
    for target, heads in writers.items():
        if target in copies:
            copies[target].append(None)
    generated = {target: [] for target, heads in writers.items() if heads and None not in heads and target not in copies}
    generated.update({target: ("copied", sources) for target, sources in copies.items() if None not in sources})
    for position, row in enumerate(rows):
        if position in mask:
            continue
        starts = list(HEREDOC_START.finditer(row))
        if len(starts) != 1:
            continue
        delimiter, strip_tabs = starts[0].group(3), starts[0].group(1) == "-"
        body, later = [], position + 1
        while later < len(rows) and (rows[later].lstrip("\t") if strip_tabs else rows[later]) != delimiter:
            body.append(rows[later])
            later += 1
        if later >= len(rows):
            continue
        row_redirects = []
        row_commands = split_commands(row, False, row_redirects)
        for index, operator, target in row_redirects:
            key = target.replace('"', "")
            if operator.startswith(">") and isinstance(generated.get(key), list) and index < len(row_commands) and literal_word(row_commands[index][0]) == "cat":
                generated[key].append("\n".join(body))
    return generated
