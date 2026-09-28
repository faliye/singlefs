#!/usr/bin/env python3
"""A9「节点代码」按函数或按走到的文件取时罩不到的东西，在今天的 crates/*/src 上数一遍（m3-prune-gpu-r3 云端攻方腿的原型，不入库）。
数的是：非测试 fn 项为 0 的源文件（覆盖率插桩下一个计数器都没有，按「走到的文件」取节点代码时永远不在里面）、fn 项之外的 const / static 项、
macro_rules! 定义、带 derive(PartialOrd / Ord) 的 enum（改成员次序就改比较结果，fn 体一个字不变），与读自己源码原文或行号的地方。
用法：a9_static_scan.py <仓根>"""
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rust_tokens  # noqa: E402


def main():
    root = pathlib.Path(sys.argv[1])
    files = sorted(path for path in root.glob("crates/*/src/**/*.rs"))
    no_function_files, consts_outside, macro_definitions, ordered_enums, source_readers = [], 0, 0, 0, []
    for path in files:
        text = path.read_text()
        relative = str(path.relative_to(root))
        tokens = rust_tokens.code_tokens(rust_tokens.lex(text))
        spans = rust_tokens.fn_item_spans(tokens)
        inside = set()
        for start, end in spans:
            inside.update(range(start, end + 1))
        digests = rust_tokens.digests_of_text(text)
        if digests["nontest_fn_items"] == 0:
            no_function_files.append(relative)
        for index, (kind, word) in enumerate(tokens):
            if index in inside or kind != "ident":
                continue
            if (word in ("const", "static") and index + 2 < len(tokens) and tokens[index + 1][0] == "ident" and tokens[index + 1][1] not in ("fn", "mut")
                    and tokens[index + 2][1] == ":" and not (index > 0 and tokens[index - 1][1] in ("<", ","))):
                consts_outside += 1
            if word == "macro_rules" and index + 1 < len(tokens) and tokens[index + 1][1] == "!":
                macro_definitions += 1
        ordered_enums += len(re.findall(r"#\[derive\([^)]*\bOrd\b[^)]*\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?enum\b", text))
        for pattern in ("include_str!(", "line!()", "file!()", "column!()", "location.line()", "Location::caller"):
            if pattern in text:
                source_readers.append(f"{relative}:{pattern}")
    print(f"E7RESULT name=r3_a9_static_scan source_files={len(files)} files_without_a_nontest_fn={len(no_function_files)} detail={no_function_files} "
          f"const_or_static_items_outside_fn_items={consts_outside} macro_rules_definitions={macro_definitions} enums_deriving_ord={ordered_enums} "
          f"reads_its_own_source_or_line_numbers={source_readers} must_be_nonzero={len(no_function_files) + consts_outside + macro_definitions + ordered_enums}")


if __name__ == "__main__":
    main()
