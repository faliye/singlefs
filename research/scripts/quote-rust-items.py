#!/usr/bin/env python3
"""按 Rust 项名整段抽代码（三方论证代码轮的 diff 附录用），每段前写文件名与行区间，抽完回读逐字节比对。

用法：
    quote-rust-items.py 文件::项名 [文件::项名 …] > 出口.md
        项名写全名：fn、struct、enum、trait、mod 的名字（`raise_rollback_floor`）、`impl 类型名`（`impl PoolVersion`）、
        测试函数全名；`const` / `static` 按名字取到分号为止。同一个文件里同名的项不止一个就退出码 4，不猜。
    quote-rust-items.py --selftest

为什么要有它：三方论证的材料员抽代码段一直是 `awk 'NR==A,NR==B'` 手挑行区间，每轮 15–19 分钟；主 agent 给的名字含糊时
（两条用例都含 unreadable）要猜，挑错的区间产物本身看不出断裂（C244（机械抽取选错行区间，产物本身没有断裂痕迹） 同族）。

退出码：0 成；2 参数或文件问题；3 找不到项；4 同名的项不止一个；5 回读与源文件那几行逐字节不符。
注意：花括号计数跳过字符串、字符字面量与注释里的括号，不认原始字符串里的括号嵌套写法之外的奇形；认不准时退出码 3，不交半截。
"""
import pathlib
import re
import sys
import tempfile

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
            i += 2
            continue
        if c == '"':
            state["string"] = True
            i += 1
            continue
        if c == "'" and re.match(r"'(\\.|[^\\'])'", line[i:]):
            i += len(re.match(r"'(\\.|[^\\'])'", line[i:]).group(0))
            continue
        out.append(c)
        i += 1
    return "".join(out)


def header_pattern(name: str):
    if name.startswith("impl "):
        target = re.escape(name[len("impl "):].strip())
        return re.compile(r"^\s*impl(?:<[^>]*>)?\s+(?:[\w:<>, ]+\s+for\s+)?" + target + r"(?:<[^>]*>)?\s*(?:where[^{]*)?\{?\s*$|^\s*impl(?:<[^>]*>)?\s+(?:[\w:<>, ]+\s+for\s+)?" + target + r"\b")
    return re.compile(r"^\s*" + ITEM_KINDS + r"\s+" + re.escape(name) + r"\b|^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+" + re.escape(name) + r"\s*:")


def find_item(lines, name):
    pattern = header_pattern(name)
    starts = [index for index, line in enumerate(lines) if pattern.search(line)]
    if not starts:
        return None, 3
    if len(starts) > 1:
        return None, 4
    start = starts[0]
    # 往上带文档注释与属性
    first = start
    while first > 0 and re.match(r"^\s*(///|//!|#\[|#!\[)", lines[first - 1]):
        first -= 1
    state = {"block": 0, "string": False}
    depth = 0
    opened = False
    for index in range(start, len(lines)):
        code = strip_code(lines[index], state)
        if not opened and ";" in code and "{" not in code:
            return (first, index), 0
        for c in code:
            if c == "{":
                depth += 1
                opened = True
            elif c == "}":
                depth -= 1
                if opened and depth == 0:
                    return (first, index), 0
    return None, 3


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
        text = path.read_text(encoding="utf-8")
        lines = text.split("\n")
        found, code = find_item(lines, name)
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
        if "\n".join(text.split("\n")[first:last + 1]) != body:
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
    print("  ✓ 自证：按名字抽出函数（带文档注释与属性、跳过字符串与注释里的括号）、impl 块与常量；同名不止一个判红")
    return 0


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
    sys.exit(main(sys.argv))
