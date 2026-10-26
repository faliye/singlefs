#!/usr/bin/env python3
# admission: always 判的是此刻研究源码与 kb 表格里的文本，几秒跑完；每一轮门禁都现判，上一次的结论不替这一次作保
# run-condition: command python3
# gate-similar: .claude/singlefs-ai-sop/scripts/naming-lint.sh 它判我们声明的名字（变量、字段、类型、常量），词表就是这里读的那一份；字符串字面量与表格里名字形态的串它认不出，这里补；词表与登记表这里只读它们，不另抄
# gate-similar: scripts/gate/crate-sources-reachable.py 同样是门禁 code-source-discipline 的一格调的判法脚本，判的是 .rs 有没有着落；这里判的是 .rs 与 kb 里的串，对象与判据都不同
"""以数据形式存在的名字也按命名纪律判（欠账 C430）。

naming-lint 只查我们声明的名字；实验装置的字段表标签 ("system_configuration_mac", 16)、臂名 => "tail_system_configuration"、
产物里 key= 的键名、kb 表格里反引号包着的名字，逐字落进留存产物与 kb 正文，那些名字没有人管。这里把它们捞出来，按同一套缩写词表与单字母规则判。

用法：
    data-name-lint.py [项目根]    # 不给就取这个脚本往上两级
    data-name-lint.py --selftest  # DATA_NAME_LINT_BREAK=<开关> 时必须红在点名的格

扫哪些（只认整串匹配 [a-z][a-z0-9_]*）：
  ① scripts/experiment-apparatus/*.rs 与 crates/*/src/**/*.rs 里的字符串字面量，三种形态：
     元组首列 ("名字", …)（左括号前面是标识符或 ! 的是函数或宏调用，不算元组）；
     => "名字" 右侧；
     任一字符串字面量里的 键= （键前面是串首、空白或 , ; ( [ { |，后面不是 = 或 >）。
     注释、字符字面量不看。
  ② .claude/kb/**/*.md 表格行（以 | 起头的行，围栏代码块里的不算）单元格里反引号包着、整段是 [a-z][a-z0-9_]* 的串；术语改名登记表 .claude/kb/term-renames.md 不扫（登记的就是旧名）。
判法与 naming-lint 同一套：按 _ 切词，纯数字的词、u8 / i64 / f32 这类位宽放行；词在 .claude/abbreviations 登记过放行（复数也认，去掉尾部数字再查）；
  e<数字>、c<数字> 这类整类登记的编号放行；名字中间单独的 a 是冠词放行；单个字母（后面跟数字也算）判红；
  词在 naming-lint.sh 的缩写词表里（从那份脚本读，不另抄一份）、或在下面 EXTRA_ABBREVIATIONS 里（词表没有、数据里常见的缩写）而没登记，判红。
  词表之外的缩写认不出，靠 review。

排除表 .claude/data-name-lint-exclude，一行一条：
    <路径>:<串>  # 理由        登记某份文件里的某个串（键= 的键名也写键名）
    <路径>  # 理由             登记整份文件
  排除只缩不涨：路径不存在、不在扫描范围、没写理由、同一条写了两遍、那一条已经一个违规都没有了，都判红。排除表不在等于一条都没排。

退出码：0 通过；1 判红（违规，或排除表不合规）；2 用法错或缺词表；77 没有可扫的 .rs 与 kb 表格。

弄坏开关 DATA_NAME_LINT_BREAK=<开关>（逗号隔开可给几个；平常的判与 --selftest 都认），--selftest 里各自要红在点名的格，门禁样本里打开后判错的在 .claude/gate.d/code-source-discipline.sh 文件头：
  string-literals-skipped   三种字面量形态一个都不扫          tuple-column-red、arrow-right-red、key-name-red、single-letter-red
  arrow-right-skipped       => "名字" 右侧不扫                arrow-right-red
  key-names-skipped         键= 的键名不扫                    key-name-red
  call-arguments-judged     函数调用的字符串实参（左括号前是标识符）也当元组首列  call-arguments-green
  comments-scanned          注释里的串也扫                    comments-green
  kb-tables-skipped         kb 表格不扫                       kb-table-red
  single-letters-passed     单字母放行                        single-letter-red
  registered-ignored        登记表不认                        registered-green
  numbered-ignored          e<数字> 这类整类登记不认          numbered-green
  exclusions-ignored        排除表不认                        exclusion-green
  stale-exclusions-passed   排除表里已经没有违规的行、指向不存在的路径、没写理由的行不判  exclusion-stale-red
  external-marker-ignored   .rs 里那一行的 `// naming-lint:external <理由>` 不认          external-marker-green
"""
import os
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

NAMING_LINT = os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts", "naming-lint.sh")
ABBREVIATIONS_FILE = ".claude/abbreviations"
EXCLUDE_FILE = ".claude/data-name-lint-exclude"
RUST_SCANNED = ("scripts/experiment-apparatus", "crates")
KB_DIRECTORY = ".claude/kb"
TERM_RENAMES_REGISTRY = ".claude/kb/term-renames.md"   # 术语改名登记表：登记的就是不许再用的旧名，扫它只会把旧名登记进排除表，不扫
SKIPPED_DIRECTORY = re.compile(r"^(target|target-.*|\.git)$")
NAME_SHAPE = re.compile(r"^[a-z][a-z0-9_]*$")
KEY_EQUALS = re.compile(r"(?<![^\s,;(\[{|])([a-z][a-z0-9_]*)=(?![=>])")
CODE_SPAN = re.compile(r"`([^`\n]+)`")
# naming-lint.sh 的缩写词表里没有、数据形态的名字里常见的缩写：词 -> 该写成什么。上游词表有的词不许在这里重复（自证核）。
EXTRA_ABBREVIATIONS = {
    "mac": "message_authentication_code（几个意思，必须写全）",
}
BREAKS = ("string-literals-skipped", "arrow-right-skipped", "key-names-skipped", "call-arguments-judged", "comments-scanned", "kb-tables-skipped",
          "single-letters-passed", "registered-ignored", "numbered-ignored", "exclusions-ignored", "stale-exclusions-passed",
          "external-marker-ignored")
broken = set(item for item in os.environ.get("DATA_NAME_LINT_BREAK", "").split(",") if item in BREAKS)


# ── 词表与登记表 ──

def upstream_table():
    """从 naming-lint.sh 读缩写词表：{缩写: 该写成什么}；读不到返回 None。"""
    try:
        with open(NAMING_LINT, encoding="utf-8") as handle:
            text = handle.read()
    except OSError:
        return None
    match = re.search(r"^ABBR_TABLE=\"\$\(cat <<'TABLE'\n(.*?)^TABLE\n", text, re.DOTALL | re.MULTILINE)
    if not match:
        return None
    table = {}
    for row in match.group(1).splitlines():
        if row.strip():
            abbreviation, _, full = row.strip().partition(" ")
            table[abbreviation] = full.strip()
    return table


def read_registry(path):
    """一行一条 `<条目>  # 理由`：[(行号, 条目, 理由)]；文件不在返回 None。"""
    if not os.path.isfile(path):
        return None
    entries = []
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle, 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            body, _, reason = line.partition("#")
            entries.append((number, body.strip(), reason.strip()))
    return entries


def read_registered(root):
    """登记表：(登记过的缩写集合, 整类登记的字母集合)。"""
    words, letters = set(), set()
    entries = read_registry(os.path.join(root, ABBREVIATIONS_FILE)) or []
    for _, token, _ in entries:
        match = re.fullmatch(r"([a-z])<数字>", token)
        if match:
            letters.add(match.group(1))
        elif re.fullmatch(r"[a-z][a-z0-9]*", token):
            words.add(token)
    return words, letters


def judge_name(name, table, registered, letters):
    """一个名字里每个词的毛病：[(词, 原因)]。判法逐条对着 naming-lint.sh 的 judge_name。"""
    problems = []
    parts = [part for part in name.split("_") if part]
    if len(parts) == 1 and len(parts[0]) == 1 and "single-letters-passed" not in broken:
        return [(parts[0], "单字母")]
    for index, part in enumerate(parts):
        if re.fullmatch(r"[0-9]+", part) or re.fullmatch(r"[uif](8|16|32|64|128)", part):
            continue
        without_digits = re.sub(r"[0-9]+$", "", part)
        singular = without_digits[:-1] if re.search(r"..s$", without_digits) else ""
        if "registered-ignored" not in broken and (part in registered or without_digits in registered or (singular and singular in registered)):
            continue
        if part == "a" and 0 < index < len(parts) - 1:
            continue
        if "numbered-ignored" not in broken and re.fullmatch(r"[a-z][0-9]+", part) and part[0] in letters:
            continue
        if re.fullmatch(r"[a-z][0-9]*", part):
            if "single-letters-passed" not in broken:
                problems.append((part, "单字母" + ("加数字" if re.search(r"[0-9]", part) else "")))
            continue
        for stem, kind in ((without_digits, "缩写"), (singular, "缩写的复数")):
            if not stem:
                continue
            if stem in table:
                problems.append((part, f"{kind}，写全：{table[stem]}"))
                break
            if stem in EXTRA_ABBREVIATIONS:
                problems.append((part, f"{kind}，写全：{EXTRA_ABBREVIATIONS[stem]}"))
                break
    return problems


# ── Rust 字符串字面量 ──

RAW_OR_BYTE_START = re.compile(r'b?r(#*)"')
PUNCTUATION_PAIRS = ("=>", "->", "::")


def lex_rust(text):
    """把 Rust 源码切成 [(种类, 值, 行号)]：种类 str / ident / punct / other。注释抹掉；字符字面量与生命周期不产出串。"""
    tokens, index, length, line = [], 0, len(text), 1
    comments_scanned = "comments-scanned" in broken
    while index < length:
        char = text[index]
        if char == "\n":
            line += 1; index += 1; continue
        if char.isspace():
            index += 1; continue
        boundary = index == 0 or not (text[index - 1].isalnum() or text[index - 1] == "_")
        if not comments_scanned and text.startswith("//", index):
            end = text.find("\n", index)
            index = length if end < 0 else end; continue
        if not comments_scanned and text.startswith("/*", index):
            depth, index = 1, index + 2
            while index < length and depth:
                if text.startswith("/*", index):
                    depth += 1; index += 2
                elif text.startswith("*/", index):
                    depth -= 1; index += 2
                else:
                    if text[index] == "\n":
                        line += 1
                    index += 1
            continue
        if boundary and char in "br":
            match = RAW_OR_BYTE_START.match(text, index)
            if match:
                close = '"' + match.group(1)
                end = text.find(close, match.end())
                end = length if end < 0 else end
                content = text[match.end():end]
                tokens.append(("str", content, line))
                line += content.count("\n")
                index = min(length, end + len(close)); continue
        if char == '"' or (boundary and text.startswith('b"', index)):
            start = index + (2 if char == "b" else 1)
            end = start
            while end < length and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            content = text[start:end]
            tokens.append(("str", content, line))
            line += content.count("\n")
            index = end + 1; continue
        if char == "'":
            if text.startswith("\\", index + 1):
                end = text.find("'", index + 3)
                if end > 0:
                    index = end + 1; continue
            elif index + 2 < length and text[index + 2] == "'" and text[index + 1] != "'":
                index += 3; continue
            index += 1
            tokens.append(("punct", "'", line)); continue
        match = re.compile(r"[A-Za-z_][A-Za-z0-9_]*").match(text, index)
        if match:
            tokens.append(("ident", match.group(0), line)); index = match.end(); continue
        match = re.compile(r"[0-9][0-9A-Za-z_]*").match(text, index)
        if match:
            tokens.append(("other", match.group(0), line)); index = match.end(); continue
        pair = text[index:index + 2]
        if pair in PUNCTUATION_PAIRS:
            tokens.append(("punct", pair, line)); index += 2; continue
        tokens.append(("punct", char, line)); index += 1
    return tokens


TUPLE_PRECEDING_KEYWORDS = {"return", "in", "yield", "break", "match", "if", "else", "let", "mut", "move"}


def rust_candidates(text):
    """[(行号, 形态, 串)]：元组首列、=> 右侧、键= 的键名。"""
    if "string-literals-skipped" in broken:
        return []
    tokens, found = lex_rust(text), []
    call_judged = "call-arguments-judged" in broken
    for index, (kind, value, line) in enumerate(tokens):
        if kind != "str":
            continue
        before = tokens[index - 1] if index else None
        before_before = tokens[index - 2] if index > 1 else None
        after = tokens[index + 1] if index + 1 < len(tokens) else None
        if NAME_SHAPE.match(value):
            if before and before[:2] == ("punct", "(") and after and after[:2] == ("punct", ","):
                preceding = before_before
                is_call = preceding is not None and (
                    (preceding[0] == "ident" and preceding[1] not in TUPLE_PRECEDING_KEYWORDS)
                    or preceding[:2] in (("punct", "!"), ("punct", ")"), ("punct", "]"), ("punct", ">")))
                if call_judged or not is_call:
                    found.append((line, "元组首列", value))
            if before and before[:2] == ("punct", "=>") and "arrow-right-skipped" not in broken:
                found.append((line, "=> 右侧", value))
        if "key-names-skipped" not in broken:
            cleaned = re.sub(r"\\[ntr]", " ", value)
            for match in KEY_EQUALS.finditer(cleaned):
                found.append((line + cleaned[:match.start()].count("\n"), "键= 的键名", match.group(1)))
    return found


# ── kb 表格 ──

def kb_candidates(text):
    """[(行号, 形态, 串)]：表格单元格里反引号包着、整段是名字形态的串。围栏代码块里的不算。"""
    if "kb-tables-skipped" in broken:
        return []
    found, fenced = [], False
    for number, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            fenced = not fenced; continue
        if fenced or not stripped.startswith("|"):
            continue
        if re.fullmatch(r"\|[\s:|-]+", stripped):
            continue
        for match in CODE_SPAN.finditer(stripped):
            if NAME_SHAPE.match(match.group(1)):
                found.append((number, "kb 表格单元格", match.group(1)))
    return found


# ── 扫仓 ──

def walk_files(root, relative_directory, suffix, depth_rule):
    base = os.path.join(root, relative_directory)
    if not os.path.isdir(base):
        return
    for current, subdirectories, files in os.walk(base):
        subdirectories[:] = sorted(name for name in subdirectories if not SKIPPED_DIRECTORY.match(name))
        relative_current = os.path.relpath(current, root).replace(os.sep, "/")
        for name in sorted(files):
            relative = f"{relative_current}/{name}"
            if name.endswith(suffix) and depth_rule(relative):
                yield relative


def is_scanned_rust(relative):
    if re.fullmatch(r"scripts/experiment-apparatus/[^/]+\.rs", relative):
        return True
    return re.fullmatch(r"crates/[^/]+/src/.+\.rs", relative) is not None


def external_reason(line):
    """那一行写了 `naming-lint:external <理由>`（与上游 naming-lint.sh 同一个行内豁免）时交回理由，没写或理由空交回空串。"""
    if "external-marker-ignored" in broken or "naming-lint:external" not in line:
        return ""
    return line.split("naming-lint:external", 1)[1].replace("*/", "").strip(" \t:：")


def collect(root):
    """扫出全部候选：返回 (文件清单, 候选 [(路径, 行号, 形态, 串)])。"""
    files, candidates = [], []
    rust_files = sorted(set(path for directory in RUST_SCANNED for path in walk_files(root, directory, ".rs", is_scanned_rust)))
    kb_files = sorted(walk_files(root, KB_DIRECTORY, ".md", lambda relative: relative != TERM_RENAMES_REGISTRY))
    for relative in rust_files + kb_files:
        try:
            with open(os.path.join(root, relative), encoding="utf-8") as handle:
                text = handle.read()
        except (OSError, UnicodeDecodeError):
            continue
        files.append(relative)
        finder = rust_candidates if relative.endswith(".rs") else kb_candidates
        lines = text.split("\n")
        candidates.extend((relative, line, kind, string) for line, kind, string in finder(text)
                          if not (relative.endswith(".rs") and external_reason(lines[line - 1])))
    return files, candidates


def load_exclusions(root, scanned):
    """排除表：({(路径, 串或 None): 行号}, 表本身的毛病 [(行号, 说明)])。"""
    entries = read_registry(os.path.join(root, EXCLUDE_FILE))
    table, faults = {}, []
    if entries is None:
        return table, faults
    for number, body, reason in entries:
        path, colon, string = body.partition(":")
        path, string = path.strip(), string.strip()
        key = (path, string if colon else None)
        if "stale-exclusions-passed" not in broken:
            if len(reason) < 4:
                faults.append((number, f"{body} 没写理由（# 后面至少 4 个字）")); continue
            if colon and not NAME_SHAPE.match(string):
                faults.append((number, f"{body} 冒号后面不是名字形态的串")); continue
            if not os.path.isfile(os.path.join(root, path)):
                faults.append((number, f"{body} 指向不存在的路径 {path}")); continue
            if path not in scanned:
                faults.append((number, f"{body} 的 {path} 不在扫描范围里（scripts/experiment-apparatus/*.rs、crates/*/src/**/*.rs、.claude/kb/**/*.md）")); continue
            if key in table:
                faults.append((number, f"{body} 与第 {table[key]} 行是同一条")); continue
        table[key] = number
    return table, faults


def judge(root, quiet=False):
    """判一个项目根：(退出码, 输出行 [(行, 是否出路)])。"""
    out = []
    say = lambda text: out.append(text)
    table = upstream_table()
    if table is None:
        say(f"  ✗ 读不到缩写词表：{NAMING_LINT} 不在或里面没有 ABBR_TABLE")
        say("     → 怎么办：词表只读上游 naming-lint.sh 的那一份，不另抄；装好 .claude/singlefs-ai-sop 再跑")
        return 2, out
    registered, letters = read_registered(root)
    files, candidates = collect(root)
    if not files:
        say(f"  ! 本次无对象可判：{root} 下没有 scripts/experiment-apparatus/*.rs、crates/*/src/**/*.rs 与 .claude/kb/**/*.md")
        return 77, out
    exclusions, faults = load_exclusions(root, set(files))
    used = {key: 0 for key in exclusions}
    violations, seen, judged = [], set(), 0
    for path, line, kind, string in candidates:
        if (path, line, kind, string) in seen:
            continue
        seen.add((path, line, kind, string))
        judged += 1
        problems = judge_name(string, table, registered, letters)
        if not problems:
            continue
        hit = next((key for key in ((path, string), (path, None)) if key in exclusions and "exclusions-ignored" not in broken), None)
        if hit:
            used[hit] += 1
            continue
        violations.append((path, line, kind, string, problems))
    if "stale-exclusions-passed" not in broken:
        for key, number in sorted(exclusions.items(), key=lambda item: item[1]):
            if used[key] == 0 and "exclusions-ignored" not in broken:
                label = key[0] if key[1] is None else f"{key[0]}:{key[1]}"
                faults.append((number, f"{label} 已经一个违规都没有了（改完名或文件挪走了）"))
    for number, text in sorted(faults):
        say(f"  ✗ {EXCLUDE_FILE}:{number}  {text}")  # gate-lint:detail
    for path, line, kind, string, problems in violations:
        words = "；".join(f"「{word}」是{reason}" for word, reason in problems)
        say(f"  ✗ {path}:{line}  {kind}「{string}」里 {words}")  # gate-lint:detail
    if faults or violations:
        say(f"  ✗ 数据形态的名字不合命名纪律：{len(violations)} 处违规、排除表 {len(faults)} 处不合规（扫了 {len(files)} 份文件、{judged} 个串）")  # gate-lint:summary
        say("     → 怎么办：名字写全，或把缩写登记进 .claude/abbreviations（一行 <缩写>  # 全称与含义）；这个名字已经落进留存产物、改名要连产物一起重跑的，登记进排除表 "
            f"{EXCLUDE_FILE}（一行 <路径>:<串>  # 理由）；排除表里已经没有违规的行、指向不存在的路径的行删掉，排除只缩不涨")
        return 1, out
    excluded_strings = sum(used.values())
    say(f"  ✓ 数据形态的名字合命名纪律：扫了 {len(files)} 份文件、{judged} 个串（排除表 {len(exclusions)} 条放行 {excluded_strings} 个串）")
    return 0, out


# ── 自证 ──

def run_selftest():
    cells = []

    def write(root, relative, text):
        path = os.path.join(root, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def case(name, files, expect_status, expect_text=None, forbid_text=None):
        with tempfile.TemporaryDirectory() as root:
            for relative, text in files.items():
                write(root, relative, text)
            status, lines = judge(root)
        output = "\n".join(lines)
        passed = status == expect_status and (expect_text is None or expect_text in output) and (forbid_text is None or forbid_text not in output)
        wanted = "".join([f"、输出含「{expect_text}」" if expect_text else "", f"、输出不含「{forbid_text}」" if forbid_text else ""])
        cells.append((name, passed, f"期望退 {expect_status}{wanted}，实退 {status}，输出：{output[:400]}"))

    registered_mac = {".claude/abbreviations": "mac  # message authentication code：校验标签\n"}
    fields = 'const FIELDS: &[(&str, u32)] = &[("system_configuration_mac", 16)];\n'
    case("tuple-column-red", {"scripts/experiment-apparatus/e1_demo.rs": fields}, 1, "scripts/experiment-apparatus/e1_demo.rs:1  元组首列「system_configuration_mac」里 「mac」")
    case("registered-green", {"scripts/experiment-apparatus/e1_demo.rs": fields, **registered_mac}, 0, "扫了 1 份文件、1 个串")
    case("single-letter-red", {"scripts/experiment-apparatus/e1_demo.rs": 'const FIELDS: &[(&str, u32)] = &[("x", 1)];\n'}, 1, "「x」是单字母")
    case("arrow-right-red", {"crates/demo/src/lib.rs": 'fn arm(a: u8) -> &\'static str {\n    match a {\n        0 => "tail_cnt",\n        _ => "tail_other",\n    }\n}\n'}, 1, "crates/demo/src/lib.rs:3  => 右侧「tail_cnt」里 「cnt」是缩写，写全：count", "tail_other")
    case("key-name-red", {"crates/demo/src/lib.rs": 'fn report() { println!("system={} num_items={}", 1, 2); }\n'}, 1, "键= 的键名「num_items」里 「num」", "「system」")
    case("key-line-red", {"crates/demo/src/lib.rs": 'fn f() {\n    println!("system={}\\\n        num_items={}", 1, 2);\n}\n'}, 1, "crates/demo/src/lib.rs:3  键= 的键名「num_items」")
    case("kb-table-red", {".claude/kb/demo.md": "| 字段 | 宽 |\n|---|---|\n| `cnt_total` | 8 |\n"}, 1, ".claude/kb/demo.md:3  kb 表格单元格「cnt_total」里 「cnt」")
    case("kb-term-renames-skipped", {".claude/kb/term-renames.md": "| 旧名 | 新名 | 匹配 |\n|---|---|---|\n| `cnt_total` | `count_total` | 整串 |\n", ".claude/kb/demo.md": "| 字段 |\n|---|\n| `count_total` |\n"}, 0, "扫了 1 份文件、1 个串")
    case("kb-table-green", {".claude/kb/demo.md": "| 字段 | 宽 |\n|---|---|\n| `count_total` | 8 |\n\n```\n| `cnt_fenced` | 8 |\n```\n正文里的 `cnt_prose` 不是表格\n"}, 0, "扫了 1 份文件、1 个串")
    numbered = {"scripts/experiment-apparatus/e100_demo.rs": 'const FIELDS: &[(&str, u32)] = &[("e100_slot_count", 1), ("c374_slot", 2)];\n'}
    case("numbered-green", {**numbered, ".claude/abbreviations": "e<数字>  # 实验编号\nc<数字>  # 欠账编号\n"}, 0, "扫了 1 份文件、2 个串")
    case("numbered-red", numbered, 1, "「e100」是单字母加数字")
    case("comments-green", {"crates/demo/src/lib.rs": '// ("cnt_note", 1) 和 => "cnt_note" 写在注释里\n/* ("cnt_block", 1) */\nfn f() { let s = "not a name"; let t = "Cnt_Upper"; check("cnt", 3); }\n'}, 0, "合命名纪律")
    case("call-arguments-green", {"crates/demo/src/lib.rs": 'fn f() { check("cnt", 3); let v = vec!("cnt", 1); }\n'}, 0, "合命名纪律")
    excluded = {"scripts/experiment-apparatus/e1_demo.rs": fields}
    case("exclusion-green", {**excluded, EXCLUDE_FILE: "scripts/experiment-apparatus/e1_demo.rs:system_configuration_mac  # 2026-10-13 立 C430 之前的名字，改名要连产物一起重跑\n"}, 0, "排除表 1 条放行 1 个串")
    case("exclusion-whole-file-green", {**excluded, EXCLUDE_FILE: "scripts/experiment-apparatus/e1_demo.rs  # 2026-10-13 立 C430 之前的名字，改名要连产物一起重跑\n"}, 0, "排除表 1 条放行 1 个串")
    stale = ("scripts/experiment-apparatus/e1_demo.rs:system_configuration_mac  # 样本：理由\nscripts/experiment-apparatus/e1_demo.rs:renamed_long  # 样本：已经改好名\n"
             "scripts/experiment-apparatus/gone.rs:cnt  # 样本：文件不存在\nscripts/experiment-apparatus/e1_demo.rs:another_mac\n")
    case("exclusion-stale-red", {**excluded, EXCLUDE_FILE: stale}, 1, "scripts/experiment-apparatus/e1_demo.rs:renamed_long 已经一个违规都没有了")
    case("exclusion-missing-path-red", {**excluded, EXCLUDE_FILE: "scripts/experiment-apparatus/gone.rs:cnt  # 样本：文件不存在\n"}, 1, "指向不存在的路径 scripts/experiment-apparatus/gone.rs")
    case("exclusion-no-reason-red", {**excluded, EXCLUDE_FILE: "scripts/experiment-apparatus/e1_demo.rs:system_configuration_mac\n"}, 1, "没写理由")
    case("external-marker-green", {"crates/demo/src/lib.rs": 'fn kind() -> &\x27static str {\n    match 0 {\n        _ => "lib", // naming-lint:external cargo 定的目标种类名\n    }\n}\n'}, 0, "合命名纪律")
    case("external-marker-no-reason-red", {"crates/demo/src/lib.rs": 'fn kind() -> &\x27static str {\n    match 0 {\n        _ => "lib", // naming-lint:external\n    }\n}\n'}, 1, "「lib」是缩写")
    case("nothing-to-judge", {"README.md": "no sources\n"}, 77, "本次无对象可判")
    table = upstream_table() or {}
    overlap = sorted(word for word in EXTRA_ABBREVIATIONS if word in table)
    cells.append(("extra-not-a-copy", bool(table) and not overlap, f"EXTRA_ABBREVIATIONS 里这些词上游词表已经有了，是另抄了一份：{overlap}；或读不到上游词表（{len(table)} 个）"))
    failed = [cell for cell in cells if not cell[1]]
    for name, passed, message in cells:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(cells)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(cells)} 格都对（元组首列、=> 右侧、键= 的键名、kb 表格、单字母、登记过的放行、整类编号、注释与调用实参不判、排除表只缩不涨、外部定名的行内豁免）")
    return 0


WHEEL = {
    "string-literals-skipped": ("tuple-column-red", "single-letter-red", "arrow-right-red", "key-name-red"),
    "arrow-right-skipped": ("arrow-right-red",),
    "key-names-skipped": ("key-name-red",),
    "call-arguments-judged": ("call-arguments-green",),
    "comments-scanned": ("comments-green",),
    "kb-tables-skipped": ("kb-table-red",),
    "single-letters-passed": ("single-letter-red",),
    "registered-ignored": ("registered-green",),
    "numbered-ignored": ("numbered-green",),
    "exclusions-ignored": ("exclusion-green",),
    "stale-exclusions-passed": ("exclusion-stale-red",),
    "external-marker-ignored": ("external-marker-green",),
}


def switch_wheel():
    """逐个打开弄坏开关重跑自证，各自要红在点名的格（点名的格都要红）。"""
    failures = 0
    for switch, wanted in WHEEL.items():
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest-inner"], capture_output=True, text=True,
                                   env=dict(os.environ, DATA_NAME_LINT_BREAK=switch))
        red_cells = re.findall(r"✗ 自证 ([a-z-]+)：", completed.stdout)
        missing = [cell for cell in wanted if cell not in red_cells]
        if completed.returncode == 1 and not missing:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(red_cells)}」，含点名的 {'、'.join(wanted)}")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后退 {completed.returncode}、红在「{'、'.join(red_cells) or '（没有）'}」，点名的 {'、'.join(wanted)} 里没红的：{'、'.join(missing)}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(WHEEL)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(WHEEL)} 个开关逐个打开，各红在点名的格")
    return 0


def main(arguments):
    if arguments[:1] in (["--selftest"], ["--selftest-inner"]):
        if arguments[0] == "--selftest-inner":
            return run_selftest()
        status = run_selftest()
        return status or switch_wheel()
    if len(arguments) > 1 or any(argument.startswith("-") for argument in arguments):
        print(f"  ✗ 认不得的参数：{' '.join(arguments)}")
        print("     → 怎么办：用法 data-name-lint.py [项目根] 或 --selftest")
        return 2
    root = os.path.realpath(arguments[0]) if arguments else REPOSITORY_ROOT
    if not os.path.isdir(root):
        print(f"  ✗ 项目根不在：{root}")
        print("     → 怎么办：给存在的目录，或不给参数判这个脚本所在的仓")
        return 2
    status, lines = judge(root)
    print("\n".join(lines))
    return status


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
