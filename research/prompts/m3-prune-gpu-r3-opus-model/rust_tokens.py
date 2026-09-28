#!/usr/bin/env python3
"""A9 被验的「节点代码摘要」的一份实现（m3-prune-gpu-r3 云端攻方腿的原型，不入库）：Rust 源码按词法单元切开，
去掉注释、不看位置（行号、缩进、空白）之后取 SHA-256；另给三种按范围取的摘要，给「节点代码怎么机械地算出来」那几种候选用。

子命令：
  file-digests <文件>...        每个文件一行：原文、词法单元（文档注释当注释去掉）、词法单元（文档注释当 #[doc] 留下）、
                                 全部 fn 项（签名 + 体）的词法单元、fn 项之外的词法单元（常量、静态量、类型、宏定义、use）各一个摘要，
                                 与非测试 fn 项的个数
  tree-compare <仓甲> <仓乙>    两份仓副本的 crates/*/src 下逐个 .rs 比上面五种摘要，打每种摘要不同的文件
  selftest                      几格必过的自证（注释改了摘要不变、挪行不变、字符字面量与生命周期分得开、嵌套块注释、原始字符串里的 //）
"""
import hashlib
import pathlib
import sys

PUNCTUATION = sorted(
    [">>=", "<<=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "^=",
     "&=", "|=", "<<", ">>", ".."],
    key=len, reverse=True)


def lex(text):
    """交回 [(种类, 原文)]；种类是 ident、number、string、char、lifetime、punct、doc_outer、doc_inner、comment。"""
    tokens = []
    position = 0
    length = len(text)
    while position < length:
        character = text[position]
        if character.isspace():
            position += 1
            continue
        if text.startswith("//", position):
            end = text.find("\n", position)
            end = length if end < 0 else end
            body = text[position:end]
            if body.startswith("///") and not body.startswith("////"):
                tokens.append(("doc_outer", body[3:].strip()))
            elif body.startswith("//!"):
                tokens.append(("doc_inner", body[3:].strip()))
            else:
                tokens.append(("comment", body))
            position = end
            continue
        if text.startswith("/*", position):
            depth = 0
            cursor = position
            while cursor < length:
                if text.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif text.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                    if depth == 0:
                        break
                else:
                    cursor += 1
            body = text[position:cursor]
            if body.startswith("/**") and not body.startswith("/***") and body != "/**/":
                tokens.append(("doc_outer", body[3:-2].strip()))
            elif body.startswith("/*!"):
                tokens.append(("doc_inner", body[3:-2].strip()))
            else:
                tokens.append(("comment", body))
            position = cursor
            continue
        raw_prefix = None
        for prefix in ("br", "cr", "r"):
            if text.startswith(prefix, position) and position + len(prefix) < length and text[position + len(prefix)] in "#\"":
                probe = position + len(prefix)
                hashes = 0
                while probe < length and text[probe] == "#":
                    hashes += 1
                    probe += 1
                if probe < length and text[probe] == '"' and not (prefix == "r" and hashes == 1 and text[probe:probe + 1] != '"'):
                    raw_prefix = (prefix, hashes, probe)
                break
        if raw_prefix is not None:
            prefix, hashes, quote = raw_prefix
            closing = '"' + "#" * hashes
            end = text.find(closing, quote + 1)
            end = length if end < 0 else end + len(closing)
            tokens.append(("string", text[position:end]))
            position = end
            continue
        if text.startswith("r#", position) and position + 2 < length and (text[position + 2].isalpha() or text[position + 2] == "_"):
            cursor = position + 2
            while cursor < length and (text[cursor].isalnum() or text[cursor] == "_"):
                cursor += 1
            tokens.append(("ident", text[position:cursor]))
            position = cursor
            continue
        string_start = None
        for prefix in ('b"', 'c"', '"'):
            if text.startswith(prefix, position):
                string_start = position + len(prefix)
                break
        if string_start is not None:
            cursor = string_start
            while cursor < length and text[cursor] != '"':
                cursor += 2 if text[cursor] == "\\" else 1
            tokens.append(("string", text[position:cursor + 1]))
            position = cursor + 1
            continue
        if text.startswith("b'", position) or character == "'":
            start = position + (2 if character == "b" else 1)
            if start < length and text[start] == "\\":
                end = text.find("'", start + 2)
                tokens.append(("char", text[position:end + 1]))
                position = end + 1
                continue
            if start + 1 < length and text[start + 1] == "'":
                tokens.append(("char", text[position:start + 2]))
                position = start + 2
                continue
            if character == "'":
                cursor = start
                while cursor < length and (text[cursor].isalnum() or text[cursor] == "_"):
                    cursor += 1
                tokens.append(("lifetime", text[position:cursor]))
                position = cursor
                continue
        if character.isalpha() or character == "_":
            cursor = position
            while cursor < length and (text[cursor].isalnum() or text[cursor] == "_"):
                cursor += 1
            tokens.append(("ident", text[position:cursor]))
            position = cursor
            continue
        if character.isdigit():
            cursor = position
            while cursor < length and (text[cursor].isalnum() or text[cursor] == "_"):
                if text[cursor] in "eE" and cursor + 1 < length and text[cursor + 1] in "+-" and not text[position:cursor].lower().startswith("0x"):
                    cursor += 2
                    continue
                cursor += 1
            if cursor + 1 < length and text[cursor] == "." and text[cursor + 1].isdigit():
                cursor += 1
                while cursor < length and (text[cursor].isalnum() or text[cursor] == "_"):
                    cursor += 1
            tokens.append(("number", text[position:cursor]))
            position = cursor
            continue
        for punctuation in PUNCTUATION:
            if text.startswith(punctuation, position):
                tokens.append(("punct", punctuation))
                position += len(punctuation)
                break
        else:
            tokens.append(("punct", character))
            position += 1
    return tokens


def token_digest(tokens, keep_doc):
    """去掉注释（keep_doc 为假时连文档注释一起去）、不看位置，逐个词法单元（种类 + 原文，带长度前缀）取 SHA-256。"""
    hasher = hashlib.sha256()
    for kind, text in tokens:
        if kind == "comment" or (kind.startswith("doc") and not keep_doc):
            continue
        encoded = (kind + "\x00" + text).encode()
        hasher.update(len(encoded).to_bytes(8, "little"))
        hasher.update(encoded)
    return hasher.hexdigest()[:16]


def code_tokens(tokens):
    return [token for token in tokens if token[0] != "comment" and not token[0].startswith("doc")]


def matching_close(tokens, open_index):
    depth = 0
    for index in range(open_index, len(tokens)):
        kind, text = tokens[index]
        if kind == "punct" and text == "{":
            depth += 1
        elif kind == "punct" and text == "}":
            depth -= 1
            if depth == 0:
                return index
    return len(tokens) - 1


def test_module_spans(tokens):
    """`#[cfg(test)]` 之后（中间可以夹别的属性）紧跟的 `mod 名 { … }` 的下标区间。"""
    spans = []
    for index in range(len(tokens) - 7):
        window = [text for _, text in tokens[index:index + 7]]
        if window == ["#", "[", "cfg", "(", "test", ")", "]"]:
            cursor = index + 7
            while cursor < len(tokens) and tokens[cursor][1] == "#":
                cursor = matching_bracket(tokens, cursor + 1) + 1
            if cursor + 2 < len(tokens) and tokens[cursor][1] == "mod" and tokens[cursor + 2][1] == "{":
                spans.append((index, matching_close(tokens, cursor + 2)))
    return spans


def matching_bracket(tokens, open_index):
    depth = 0
    for index in range(open_index, len(tokens)):
        if tokens[index][1] == "[":
            depth += 1
        elif tokens[index][1] == "]":
            depth -= 1
            if depth == 0:
                return index
    return len(tokens) - 1


def fn_item_spans(tokens):
    """最外层的 fn 项（关键字 fn 后面跟名字；函数指针类型 `fn(` 不算），从 fn 到它的体的右花括号或声明末尾的分号。"""
    spans = []
    index = 0
    while index < len(tokens) - 1:
        kind, text = tokens[index]
        if kind == "ident" and text == "fn" and tokens[index + 1][0] == "ident":
            depth = 0
            cursor = index + 1
            while cursor < len(tokens):
                current = tokens[cursor][1]
                if current in "([":
                    depth += 1
                elif current in ")]":
                    depth -= 1
                elif depth == 0 and current == ";":
                    break
                elif depth == 0 and current == "{":
                    cursor = matching_close(tokens, cursor)
                    break
                cursor += 1
            spans.append((index, cursor))
            index = cursor + 1
            continue
        index += 1
    return spans


def digests_of_text(text):
    tokens = lex(text)
    code = code_tokens(tokens)
    fn_spans = fn_item_spans(code)
    inside = set()
    for start, end in fn_spans:
        inside.update(range(start, end + 1))
    fn_tokens = [code[index] for index in sorted(inside)]
    outside_tokens = [code[index] for index in range(len(code)) if index not in inside]
    test_spans = test_module_spans(code)
    nontest_fn = sum(1 for start, _ in fn_spans if not any(low <= start <= high for low, high in test_spans)
                     and not (start >= 3 and [t for _, t in code[start - 4:start]] == ["#", "[", "test", "]"]))
    return {
        "raw": hashlib.sha256(text.encode()).hexdigest()[:16],
        "tokens_doc_as_comment": token_digest(tokens, keep_doc=False),
        "tokens_doc_as_attribute": token_digest(tokens, keep_doc=True),
        "fn_items": token_digest(fn_tokens, keep_doc=False),
        "outside_fn_items": token_digest(outside_tokens, keep_doc=False),
        "nontest_fn_items": nontest_fn,
    }


KINDS = ["raw", "tokens_doc_as_comment", "tokens_doc_as_attribute", "fn_items", "outside_fn_items"]


def file_digests(paths):
    for path in paths:
        digests = digests_of_text(pathlib.Path(path).read_text())
        print(f"name=r3_file_digests file={path} " + " ".join(f"{kind}={digests[kind]}" for kind in KINDS)
              + f" nontest_fn_items={digests['nontest_fn_items']}")


def tree_compare(root_a, root_b):
    root_a, root_b = pathlib.Path(root_a), pathlib.Path(root_b)
    files = sorted(path.relative_to(root_a) for path in root_a.glob("crates/*/src/**/*.rs"))
    differing = {kind: [] for kind in KINDS}
    for relative in files:
        other = root_b / relative
        if not other.exists():
            continue
        left, right = digests_of_text((root_a / relative).read_text()), digests_of_text(other.read_text())
        for kind in KINDS:
            if left[kind] != right[kind]:
                differing[kind].append(str(relative))
    print(f"name=r3_tree_compare files={len(files)} " + " ".join(f"{kind}_differs={differing[kind]}" for kind in KINDS))
    all_fn = hashlib.sha256()
    for root in (root_a, root_b):
        pass
    return 0


def selftest():
    base = 'const LIMIT: u64 = 16; // 上限\nfn f(x: &\'a str) -> char { let c = \'a\'; /* 外 /* 内 */ 外 */ let s = r#"http://x"#; if x.len() > 1 { c } else { \'\\n\' } }\n'
    moved = "\n\n" + base.replace("// 上限", "// 改了注释").replace("{ c }", "{\n        c\n    }")
    doc = "/// 文档\n" + base
    changed_constant = base.replace("= 16;", "= 17;")
    checks = [
        ("注释与挪行不改词法摘要", digests_of_text(base)["tokens_doc_as_comment"] == digests_of_text(moved)["tokens_doc_as_comment"]),
        ("改原文会改原文摘要", digests_of_text(base)["raw"] != digests_of_text(moved)["raw"]),
        ("文档注释当注释时不改摘要", digests_of_text(base)["tokens_doc_as_comment"] == digests_of_text(doc)["tokens_doc_as_comment"]),
        ("文档注释当属性时改摘要", digests_of_text(base)["tokens_doc_as_attribute"] != digests_of_text(doc)["tokens_doc_as_attribute"]),
        ("改常量改词法摘要", digests_of_text(base)["tokens_doc_as_comment"] != digests_of_text(changed_constant)["tokens_doc_as_comment"]),
        ("改 fn 外的常量不改 fn 项摘要", digests_of_text(base)["fn_items"] == digests_of_text(changed_constant)["fn_items"]),
        ("原始字符串里的 // 不当注释", ("string", 'r#"http://x"#') in lex(base)),
        ("字符字面量与生命周期分得开", ("char", "'a'") in lex(base) and ("lifetime", "'a") in lex(base)),
        ("嵌套块注释整块一个", sum(1 for kind, _ in lex(base) if kind == "comment") == 2),
        ("数出 1 个非测试 fn", digests_of_text(base)["nontest_fn_items"] == 1),
    ]
    failed = [name for name, passed in checks if not passed]
    print(f"name=r3_rust_tokens_selftest checks={len(checks)} failed={failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    if len(sys.argv) >= 2 and sys.argv[1] == "file-digests":
        file_digests(sys.argv[2:])
    elif len(sys.argv) == 4 and sys.argv[1] == "tree-compare":
        sys.exit(tree_compare(sys.argv[2], sys.argv[3]))
    elif len(sys.argv) == 2 and sys.argv[1] == "selftest":
        sys.exit(selftest())
    else:
        print(__doc__)
        sys.exit(2)
