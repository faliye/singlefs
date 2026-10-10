#!/usr/bin/env python3
"""K2（第二十五块）：在一份抄出来的源文件上施加一个改坏：scan-mutants.py <源文件目录> <名字>；名字是 row:<crates/mutations.tsv 行号>（取冻结副本那一行）
或下面 MUTANTS 里的一个。原文在被改文件里恰好命中一次才改，否则退 3。打印改的是哪份文件、点名的用例。"""
import pathlib
import sys

FROZEN = pathlib.Path.home() / ".local/share/singlefs/work/m3-batch10z-xy-code-r1/tree"
CA, GU = "crash_amplification.rs", "gpu_unit_checks.rs"
CUT_CRASH = "the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item"
CUT_GPU = "the_card_pool_scans_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item"
MUTANTS = {
    # 「圆括号、方括号第 0 层」只数圆括号（方括号那一半去掉）
    "B1-crash-depth-counts-only-parentheses": (CA, "            if matches!(character, '(' | '[') {", "            if matches!(character, '(') {", CUT_CRASH),
    "B2-crash-depth-counts-only-parentheses-both-sides": (CA, "            if matches!(character, '(' | '[') {\n                depth + 1\n            } else if matches!(character, ')' | ']') {",
                                                          "            if matches!(character, '(') {\n                depth + 1\n            } else if matches!(character, ')') {", CUT_CRASH),
    "B3-gpu-depth-counts-only-parentheses": (GU, '            } else if matches!(*token, "(" | "[") {\n                parenthesis_and_bracket_depth += 1;\n            } else if matches!(*token, ")" | "]") {',
                                             '            } else if matches!(*token, "(") {\n                parenthesis_and_bracket_depth += 1;\n            } else if matches!(*token, ")") {', CUT_GPU),
    # 返回类型 bool 之后紧跟标识符的字也算（`boolean` 当成 bool）
    "B4-crash-bool-prefix-of-an-identifier-counts": (CA, "                (!after_the_parentheses[bool_length..]\n                    .chars()\n                    .next()\n                    .is_some_and(is_identifier_character))",
                                                    "                (true || !after_the_parentheses[bool_length..]\n                    .chars()\n                    .next()\n                    .is_some_and(is_identifier_character))",
                                                    "the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses"),
}


def main():
    directory, name = pathlib.Path(sys.argv[1]), sys.argv[2]
    if name.startswith("row:"):
        row = (FROZEN / "crates/mutations.tsv").read_text().split("\n")[int(name[4:]) - 1].split("\t")
        rel, old, new, test = row[1], row[2].replace("\\n", "\n"), row[3].replace("\\n", "\n"), row[5]
        file_name = rel.rsplit("/", 1)[-1]
    else:
        file_name, old, new, test = MUTANTS[name]
    path = directory / file_name
    text = path.read_text()
    if text.count(old) != 1:
        print(f"{name}: 原文命中 {text.count(old)} 次", file=sys.stderr)
        sys.exit(3)
    path.write_text(text.replace(old, new))
    print(file_name)
    print(test)


if __name__ == "__main__":
    main()
