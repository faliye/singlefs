#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""按 Rust 项名整段抽代码（三方论证代码轮的 diff 附录用），每段前写文件名与行区间，抽完回读逐字节比对。

用法：
    quote-rust-items.py 文件::项名 [文件::项名 …] > 出口.md
        项名写全名：fn、struct、enum、trait、mod 的名字（`raise_rollback_floor`）、`impl 类型名`（`impl PoolVersion`）、
        测试函数全名；`const` / `static` 按名字取到分号为止。同一个文件里同名的项不止一个就退出码 4，不猜。
    quote-rust-items.py --selftest

为什么要有它：三方论证的材料员抽代码段一直是 `awk 'NR==A,NR==B'` 手挑行区间，每轮 15–19 分钟；主 agent 给的名字含糊时
（两条用例都含 unreadable）要猜，挑错的区间产物本身看不出断裂（C244（机械抽取选错行区间，产物本身没有断裂痕迹） 同族）。

退出码：0 成；2 参数或文件问题（不是「文件::项名」、文件不在、读不了、不是 UTF-8，抽之前与回读时都算，不抛回溯）；3 找不到项；4 同名的项不止一个；
5 回读与源文件那几行逐字节不符；6 项上方的属性认不准（收在 `]` 上的那一行往上配不成一段完整的 `#[…]` / `#![…]`）。
注意：花括号计数跳过字符串、字符字面量与注释里的括号，不认原始字符串里的括号嵌套写法之外的奇形；认不准时退出码 3，不交半截。

项往上带的附属行（item_first_line）：紧挨着项头往上，逐段认
  文档注释行 `///`、`//!`；
  属性：收在 `]` 上的那一行往上找最近一行 `#[` / `#![` 起头的，两行之间（去掉注释与字面量之后）正好是一个或几个配平的属性才算，
    跨行的 `#[cfg(…` 到 `)]` 整段带上；配不成的退 6，不截半截；
  块文档注释 `/** … */`、`/*! … */`（整段只有注释，跨行的也认）；普通块注释 `/* … */` 与普通 `//` 注释、空行不带，到那里停。

弄坏开关（只给 --selftest 证红用；平常跑设了就退 2，点名它只在自证里认）：
  QUOTE_RUST_ITEMS_BREAK=read-errors-raw  read_source() 不接读取与解码的错误，抛回溯退 1
      读不了的文件那一格（「✗ 自证失败：读不了的文件」）
  QUOTE_RUST_ITEMS_BREAK=single-line-attributes  item_first_line() 只认单行的 `///`、`//!`、`#[`、`#![` 起头的行，跨行属性的末行 `)]` 停住，属性连同它前面的文档注释都丢
      跨行属性那一格（「✗ 自证失败：跨行的 #[cfg(…)] 属性」）
  QUOTE_RUST_ITEMS_BREAK=unmatched-attribute-kept  item_first_line() 碰到收在 `]` 上却配不成属性的那一行不拒，停在那里照交（静默截掉）
      认不准的属性那一格（「✗ 自证失败：项上方收在 ] 上的一行配不成属性」）
"""
import contextlib
import io
import os
import pathlib
import re
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

ITEM_KINDS = r"(?:pub(?:\([^)]*\))?\s+)?(?:async\s+|const\s+|unsafe\s+)*(?:fn|struct|enum|trait|mod|type|union)"


def strip_code(line: str, state: dict) -> str:
    """去掉注释、字符串、字符字面量，只留下决定括号配对的字符。state 带跨行的块注释与字符串状态。"""
    out = []
    i = 0
    while i < len(line):
        c = line[i]
        nxt = line[i + 1] if i + 1 < len(line) else ""
        if state["block"]:
            if c == "*" and nxt == "/":
                state["block"] -= 1
                i += 2
                continue
            if c == "/" and nxt == "*":
                state["block"] += 1
                i += 2
                continue
            i += 1
            continue
        if state.get("raw") is not None:
            ending = '"' + state['raw']
            if line.startswith(ending, i):
                i += len(ending)
                state['raw'] = None
            else:
                i += 1
            continue
        if state["string"]:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                state["string"] = False
            i += 1
            continue
        if c == "/" and nxt == "/":
            break
        if c == "/" and nxt == "*":
            state["block"] += 1
            out.append(' ')
            i += 2
            continue
        raw = re.match(r'(?:br|cr|r)(#*)"', line[i:])
        if raw and (i == 0 or not (line[i - 1].isalnum() or line[i - 1] == '_')):
            state['raw'] = raw.group(1)
            out.append(' ')
            i += len(raw.group(0))
            continue
        if c == '"':
            state["string"] = True
            i += 1
            continue
        character = re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^\\'])'", line[i:]) if c == "'" else None
        if character:
            i += len(character.group(0))
            out.append(' ')
            continue
        out.append(c)
        i += 1
    return "".join(out)


def header_pattern(name: str):
    if name.startswith("impl "):
        target = re.escape(name[len("impl "):].strip())
        return re.compile(r"^\s*impl(?:<[^>]*>)?\s+(?:[\w:<>, ]+\s+for\s+)?" + target + r"(?:<[^>]*>)?\s*(?:where[^{]*)?\{?\s*$|^\s*impl(?:<[^>]*>)?\s+(?:[\w:<>, ]+\s+for\s+)?" + target + r"\b")
    return re.compile(r"^\s*" + ITEM_KINDS + r"\s+" + re.escape(name) + r"\b|^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+" + re.escape(name) + r"\s*:")


DOC_COMMENT_LINE = re.compile(r"^\s*(///|//!)")
ATTRIBUTE_OPENING_LINE = re.compile(r"^\s*#!?\[")
ATTRIBUTE_SEARCH_LINES = 200   # 收在 `]` 上的那一行往上最多找这么多行去配属性的起头；再远的当认不准


def attributes_fill(code):
    """code（去掉注释与字面量之后的几行，接成一段）从头到尾正好是一个或几个配平的属性 `#[…]` / `#![…]`。"""
    text, position = code.strip(), 0
    while position < len(text):
        opening = re.match(r"#!?\[", text[position:])
        if not opening:
            return False
        depth, index, closed = 0, position + opening.end() - 1, None
        while index < len(text):
            if text[index] in "([{":
                depth += 1
            elif text[index] in ")]}":
                depth -= 1
                if depth < 0:
                    return False
                if depth == 0:
                    closed = index
                    break
            index += 1
        if closed is None:
            return False
        position = closed + 1
        while position < len(text) and text[position].isspace():
            position += 1
    return True


def attribute_opening(codes, end):
    """codes[end] 收在 `]` 上时往上找它所属的属性的起头行：→ 起头的行号；这一行不收在 `]` 上 → None；收在 `]` 上却配不成属性 → -1。"""
    if not codes[end].rstrip().endswith("]"):
        return None
    for opening in range(end, max(-1, end - ATTRIBUTE_SEARCH_LINES), -1):
        if ATTRIBUTE_OPENING_LINE.match(codes[opening]):
            return opening if attributes_fill("\n".join(codes[opening:end + 1])) else -1
    return -1


def doc_block_comment_opening(lines, codes, block_depth_before, end):
    """lines[end] 是一段只有注释的块注释的末行、而且那是块文档注释（`/**`、`/*!` 起头）→ 起头的行号；否则 None。
    block_depth_before[i] 是第 i 行开头时块注释的层数（strip_code 的状态）。"""
    if codes[end].strip() or not lines[end].rstrip().endswith("*/") or block_depth_before[end + 1]:
        return None
    opening = end
    while opening > 0 and block_depth_before[opening]:
        opening -= 1
    if any(codes[index].strip() for index in range(opening, end + 1)):
        return None
    head = lines[opening].lstrip()
    if (head.startswith("/**") and not head.startswith("/***") and not head.startswith("/**/")) or head.startswith("/*!"):
        return opening
    return None


def item_first_line(lines, codes, block_depth_before, start, problems):
    """项头在 start 行，往上带它的附属行（文件头「项往上带的附属行」）→ 第一行的行号；属性认不准 → None，problems 里记一条说明。"""
    if os.environ.get("QUOTE_RUST_ITEMS_BREAK") == "single-line-attributes":   # 弄坏开关：只认单行的属性
        first = start
        while first > 0 and re.match(r"^\s*(///|//!|#\[|#!\[)", lines[first - 1]):
            first -= 1
        return first
    first, index = start, start - 1
    while index >= 0:
        if DOC_COMMENT_LINE.match(lines[index]) and not block_depth_before[index]:
            first, index = index, index - 1
            continue
        block_opening = doc_block_comment_opening(lines, codes, block_depth_before, index)
        if block_opening is not None:
            first, index = block_opening, block_opening - 1
            continue
        opening = attribute_opening(codes, index)
        if opening is None:
            break
        if opening < 0:
            if os.environ.get("QUOTE_RUST_ITEMS_BREAK") == "unmatched-attribute-kept":   # 弄坏开关：认不准也不拒，停在这里
                break
            problems.append(f"第 {index + 1} 行收在 `]` 上，往上配不成一段完整的属性")
            return None
        first, index = opening, opening - 1
    return first


def find_item(lines, name, problems=None):
    """→ ((首行, 末行), 0)，行号 0 起；找不到或括号配不上 (None, 3)；同名不止一个 (None, 4)；项上方的属性认不准 (None, 6)，说明记进 problems。"""
    pattern = header_pattern(name)
    state = {"block": 0, "string": False}
    codes, block_depth_before = [], []
    for line in lines:
        block_depth_before.append(state["block"])
        codes.append(strip_code(line, state))
    block_depth_before.append(state["block"])
    starts = [index for index, line in enumerate(codes) if pattern.search(line)]
    if not starts:
        return None, 3
    if len(starts) > 1:
        return None, 4
    start = starts[0]
    first = item_first_line(lines, codes, block_depth_before, start, problems if problems is not None else [])
    if first is None:
        return None, 6
    stack = []
    opened = False
    header_angles = 0
    in_contract = False
    # 常量、静态量和类型别名允许块表达式，直到最外层分号才结束。
    semicolon_item = re.match(
        r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const|static|type)\s+' + re.escape(name) + r'\b', codes[start]) is not None
    for index in range(start, len(lines)):
        code = codes[index]
        # Verus 函数的契约位于签名和函数体之间，其中的 < 是比较。
        if not opened and not stack and not header_angles and re.match(
                r'^\s*(?:requires|ensures|recommends|decreases|opens_invariants|no_unwind)\b', code):
            in_contract = True
        for offset, c in enumerate(code):
            # 项头的泛型参数可含块表达式。只在普通括号外跟踪尖括号，
            # 不把函数体中的比较/移位或返回类型箭头当作泛型分隔符。
            if not semicolon_item and not in_contract and not opened and not stack and c == '<':
                header_angles += 1
            elif not semicolon_item and not in_contract and not opened and not stack and c == '>' and (offset == 0 or code[offset - 1] != '-'):
                header_angles = max(0, header_angles - 1)
            elif c in '([{':
                if c == '{' and not stack and not header_angles:
                    opened = True
                stack.append(c)
            elif c in ')]}':
                if not stack or stack.pop() != {')': '(', ']': '[', '}': '{'}[c]:
                    return None, 3
                if c == '}' and opened and not stack and not semicolon_item:
                    return (first, index), 0
            elif c == ';' and not stack and not header_angles:
                return (first, index), 0
    return None, 3


def read_source(path, file_name):
    """→ (文本, None) 或 (None, 读不了的说明)：读取错误与解码错误都接住，不抛回溯。"""
    if os.environ.get("QUOTE_RUST_ITEMS_BREAK") == "read-errors-raw":   # 弄坏开关：直接读，错误往外抛
        return path.read_text(encoding="utf-8"), None
    try:
        return path.read_text(encoding="utf-8"), None
    except UnicodeDecodeError as error:
        return None, f"{file_name} 不是 UTF-8 文本（第 {error.start} 字节解不了：{error.reason}）"
    except OSError as error:
        return None, f"读不了 {file_name}（{error.__class__.__name__}：{error.strerror or error}）"


def quote(specs):
    blocks = []
    for spec in specs:
        if "::" not in spec:
            print(f"  ✗ 参数 {spec} 不是「文件::项名」", file=sys.stderr)
            print("     → 怎么办：写成 crates/singlefs-core/src/mount.rs::raise_rollback_floor。", file=sys.stderr)
            return 2, ""
        file_name, name = spec.split("::", 1)
        path = pathlib.Path(file_name)
        if not path.is_file():
            print(f"  ✗ 找不到文件 {file_name}", file=sys.stderr)
            print("     → 怎么办：路径从仓库根写。", file=sys.stderr)
            return 2, ""
        text, unreadable = read_source(path, file_name)
        if unreadable:
            print(f"  ✗ {unreadable}", file=sys.stderr)
            print("     → 怎么办：给 UTF-8 编码、读得了的 .rs 源文件；权限不够的先看 ls -l，编码不对的先确认给的是不是源码。", file=sys.stderr)
            return 2, ""
        lines = text.split("\n")
        found, code = find_item(lines, name)
        if code == 6:
            problems = []   # 只在认不准时再走一遍收说明：find_item(lines, name) 两个参数的调法照旧（测试替身按它写）
            find_item(lines, name, problems)
            print(f"  ✗ {file_name} 里 {name} 上方的属性认不准：{'；'.join(problems)}", file=sys.stderr)
            print("     → 怎么办：看那几行属性的括号是不是配平、是不是夹着别的代码；实在认不准就退回按行区间取并在附录里写明，不交半截。", file=sys.stderr)
            return 6, ""
        if code == 4:
            print(f"  ✗ {file_name} 里名叫 {name} 的项不止一个", file=sys.stderr)
            print("     → 怎么办：换一个唯一的名字（例：impl 块写 `impl 类型名`，或直接取外层的函数），不猜是哪一个。", file=sys.stderr)
            return 4, ""
        if found is None:
            print(f"  ✗ {file_name} 里找不到 {name}（或括号配不上）", file=sys.stderr)
            print("     → 怎么办：核对全名；实在配不上就退回按行区间取并在附录里写明。", file=sys.stderr)
            return 3, ""
        first, last = found
        body = "\n".join(lines[first:last + 1])
        reread, unreadable = read_source(path, file_name)
        if unreadable:
            print(f"  ✗ 回读时{unreadable}", file=sys.stderr)
            print("     → 怎么办：源文件在抽的途中被删、被改了权限或编码，等它停下再抽。", file=sys.stderr)
            return 2, ""
        if reread != text:
            print(f"  ✗ {spec} 回读与源文件那几行不符", file=sys.stderr)
            print("     → 怎么办：源文件在抽的途中被改了，等它停下再抽。", file=sys.stderr)
            return 5, ""
        fence = "```"
        while fence in body:
            fence += "`"
        blocks.append(f"### {file_name}:{first + 1}-{last + 1}（{name}）\n\n{fence}rust\n{body}\n{fence}\n")
    return 0, "\n".join(blocks)


def selftest() -> int:
    sample = '''/// 文档
#[must_use]
pub fn outer(value: u64) -> u64 {
    let text = "a { brace in string";
    let ch = '{';
    // } brace in comment
    if value > 0 { value } else { 0 }
}

pub struct Thing {
    field: u64,
}

impl Thing {
    fn inner(&self) -> u64 { self.field }
}

pub const LIMIT: u64 = 3;
'''
    with tempfile.TemporaryDirectory() as work:
        path = pathlib.Path(work) / "sample.rs"
        path.write_text(sample, encoding="utf-8")
        code, out = quote([f"{path}::outer", f"{path}::impl Thing", f"{path}::LIMIT"])
        if code != 0 or "if value > 0" not in out or "fn inner" not in out or "pub const LIMIT" not in out or "/// 文档" not in out or ":1-8（outer）" not in out:
            print(f"  ✗ 自证失败：该抽出的项没抽全（{code}）\n{out}")
            print("     → 怎么办：看 strip_code 跳字符串、字符字面量与注释那几支。")
            return 1
        path.write_text(sample + "\nfn outer() {}\n", encoding="utf-8")
        code, _ = quote([f"{path}::outer"])
        if code != 4:
            print("  ✗ 自证失败：同名的项不止一个时没拒绝")
            print("     → 怎么办：find_item 要数全部命中。")
            return 1
        # 读不了的文件：解码错误（不是 UTF-8）与读取错误（这里用一个读时抛 PermissionError 的假路径）都退 2、说清楚，不抛回溯
        undecodable = pathlib.Path(work) / "undecodable.rs"
        undecodable.write_bytes(b"fn wanted() {}\n\xff\xfe\n")

        class UnreadablePath:
            def read_text(self, encoding):
                raise PermissionError(13, "Permission denied")
        said = io.StringIO()
        try:
            with contextlib.redirect_stderr(said):
                undecodable_code, undecodable_out = quote([f"{undecodable}::wanted"])
            _, unreadable_reason = read_source(UnreadablePath(), "unreadable.rs")
        except Exception as error:   # 弄坏开关 read-errors-raw 下错误往外抛，这一格要红在这里、不让整份自证抛回溯
            undecodable_code, undecodable_out, unreadable_reason = f"抛了 {error!r}", "", None
        if undecodable_code != 2 or undecodable_out or "不是 UTF-8" not in said.getvalue() or "→ 怎么办" not in said.getvalue() \
                or not unreadable_reason or "PermissionError" not in unreadable_reason:
            print(f"  ✗ 自证失败：读不了的文件（不是 UTF-8、读时出错）应当退 2、说明是哪一种、不抛回溯，实际退 {undecodable_code}，"
                  f"读取错误的说明 {unreadable_reason!r}：{said.getvalue()[-200:]}")
            print("     → 怎么办：看 read_source() 接错误那两支；QUOTE_RUST_ITEMS_BREAK=read-errors-raw 设着的话这里本来就该红。")
            return 1
        # 跨行属性连同它前面的文档注释整段带上；块文档注释带上、普通块注释不带
        attributed = pathlib.Path(work) / "attributed.rs"
        attributed.write_text('fn before() {}\n\n/// 多行属性前的文档\n#[cfg(any(\n    test,\n    feature = "x]"\n))]\n#[inline] #[must_use]\n'
                              'fn gated() -> u64 { 1 }\n\n/* 普通块注释 */\n/** 块文档\n * 第二行\n */\nfn documented() {}\n', encoding="utf-8")
        code_gated, out_gated = quote([f"{attributed}::gated"])
        code_documented, out_documented = quote([f"{attributed}::documented"])
        if code_gated != 0 or ":3-9（gated）" not in out_gated or "/// 多行属性前的文档" not in out_gated or "#[cfg(any(" not in out_gated:
            print(f"  ✗ 自证失败：跨行的 #[cfg(…)] 属性连同它前面的文档注释应当整段带上（第 3-9 行），实际退 {code_gated}：\n{out_gated}")
            print("     → 怎么办：看 item_first_line() 与 attribute_opening()；QUOTE_RUST_ITEMS_BREAK=single-line-attributes 设着的话这里本来就该红。")
            return 1
        if code_documented != 0 or ":12-15（documented）" not in out_documented or "普通块注释" in out_documented:
            print(f"  ✗ 自证失败：块文档注释 /** … */ 应当带上（第 12-15 行）、它上面的普通块注释不带，实际退 {code_documented}：\n{out_documented}")
            print("     → 怎么办：看 doc_block_comment_opening()：只认 /** 与 /*! 起头、整段只有注释的块。")
            return 1
        # 项上方收在 ] 上的一行配不成属性：退 6，不交半截
        unmatched = pathlib.Path(work) / "unmatched.rs"
        unmatched.write_text("const BEFORE: u8 = 0;\n    feature = \"x\"\n)]\nfn wanted() {}\n", encoding="utf-8")
        said = io.StringIO()
        with contextlib.redirect_stderr(said):
            code_unmatched, out_unmatched = quote([f"{unmatched}::wanted"])
        if code_unmatched != 6 or out_unmatched or "认不准" not in said.getvalue():
            print(f"  ✗ 自证失败：项上方收在 ] 上的一行配不成属性，应当退 6、不交半截，实际退 {code_unmatched}：{out_unmatched}{said.getvalue()[-200:]}")
            print("     → 怎么办：看 item_first_line() 里 attribute_opening() 返回 -1 那一支；QUOTE_RUST_ITEMS_BREAK=unmatched-attribute-kept 设着的话这里本来就该红。")
            return 1
    print("  ✓ 自证：按名字抽出函数（带文档注释与属性、跳过字符串与注释里的括号）、impl 块与常量；同名不止一个判红；"
          "读不了的文件退 2；跨行属性连同前面的文档注释整段带上、块文档注释带上、普通块注释不带；配不成属性的退 6（查了 7 种）")
    return 0


BREAK_VARIABLES = ("QUOTE_RUST_ITEMS_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/quote-rust-items.py --selftest", file=sys.stderr)
    return 2


def main(argv) -> int:
    if len(argv) == 2 and argv[1] == "--selftest":
        return selftest()
    if len(argv) < 2:
        print("  ✗ 用法：quote-rust-items.py 文件::项名 … 或 --selftest", file=sys.stderr)
        print("     → 怎么办：项名写全名，每个参数一项。", file=sys.stderr)
        return 2
    code, out = quote(argv[1:])
    if code == 0:
        sys.stdout.write(out)
    return code


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv))
