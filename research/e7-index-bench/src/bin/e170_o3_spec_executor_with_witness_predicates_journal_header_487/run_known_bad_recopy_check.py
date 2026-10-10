#!/usr/bin/env python3
"""E170 第 2 次重跑登记修订 2 读法 18、19 的照抄核对（重导准备第三趟执行员写的草稿，sha256 b5f99fe5…；第二段带进仓，只换了这段文件头）。
文件名加 `run_`：12.15 写的 `known_bad_recopy_check.py` 不带 5.6 冻结排除的前缀，放进这个目录会被 `executor_main.freeze_lines` 算进冻结。
用法：run_known_bad_recopy_check.py <照抄前的装置> <照抄后的装置> <用例文件> <I4 known-bad-o2.tsv> <crates/singlefs-checker/src/image.rs>
每行一条结果，`name=` 开头；最后一行 `name=known_bad_recopy_verdict`。"""
import hashlib
import re
import sys

MODULE_HEADER = "mod known_bad_images_copied_from_the_harness_test {"
FUNCTION_HEADER = "fn known_bad_images(clean: &MemoryPool)"
EXPECTED_OLD_MODULE_SHA256 = "a5866391e1c1b036a34b4239e9e083716726d8d3ab7863a5f6c7d861d00bbf4a"
ITEMS_NOT_COMPARED = {"fn:entries_of_the_known_bad_images"}
ITEM_KEYWORDS = r"(fn|const|static|type|struct|enum|mod|use|impl|trait)"
CHARACTER_LITERAL = re.compile(r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'\n])'")
OPENING = "([{"
CLOSING = ")]}"


def normalized(text):
    """读法 18 的归一：去掉全部空白字符，再去掉紧挨在 `)`、`]`、`}` 之前的逗号。"""
    without_whitespace = re.sub(r"\s+", "", text)
    return re.sub(r",(?=[)\]}])", "", without_whitespace)


def scan_lines(lines):
    """逐行给出（去掉注释之后的代码、这一行之后的括号深度）；字符串、字符字面量、注释里的括号不算。"""
    results = []
    depth = 0
    in_string = False
    raw_hashes = None
    block_comment_depth = 0
    for line in lines:
        code = []
        position = 0
        while position < len(line):
            character = line[position]
            if block_comment_depth:
                if line.startswith("*/", position):
                    block_comment_depth -= 1
                    position += 2
                elif line.startswith("/*", position):
                    block_comment_depth += 1
                    position += 2
                else:
                    position += 1
                continue
            if raw_hashes is not None:
                terminator = '"' + "#" * raw_hashes
                if line.startswith(terminator, position):
                    code.append(terminator)
                    position += len(terminator)
                    raw_hashes = None
                else:
                    code.append(character)
                    position += 1
                continue
            if in_string:
                code.append(character)
                if character == "\\":
                    if position + 1 < len(line):
                        code.append(line[position + 1])
                    position += 2
                    continue
                if character == '"':
                    in_string = False
                position += 1
                continue
            if line.startswith("//", position):
                break
            if line.startswith("/*", position):
                block_comment_depth += 1
                position += 2
                continue
            raw_match = re.match(r'b?r(#*)"', line[position:])
            if raw_match and (position == 0 or not (line[position - 1].isalnum() or line[position - 1] == "_")):
                raw_hashes = len(raw_match.group(1))
                code.append(raw_match.group(0))
                position += len(raw_match.group(0))
                continue
            if character == '"':
                in_string = True
                code.append(character)
                position += 1
                continue
            if character == "'":
                literal = CHARACTER_LITERAL.match(line, position)
                if literal:
                    code.append(literal.group(0))
                    position = literal.end()
                    continue
            if character in OPENING:
                depth += 1
            elif character in CLOSING:
                depth -= 1
            code.append(character)
            position += 1
        results.append(("".join(code), depth))
    assert not in_string and raw_hashes is None and block_comment_depth == 0, "扫到文件尾还在字符串或注释里"
    return results


def top_level_items(lines):
    """顶格的条目：从第一行代码（连同紧挨的 `#[…]`）到括号深度回 0 且以 `}` 或 `;` 收尾的那一行；顶格的 `//`、`///` 行不算条目。
    交回 {键: 文本}，键是 `关键字:名字`（`use` 的键是 `use:` 加归一后的文本）。"""
    scanned = scan_lines(lines)
    items = {}
    order = []
    current = None
    depth_before = 0
    for line, (code, depth_after) in zip(lines, scanned):
        stripped_code = code.strip()
        if current is None:
            if depth_before == 0 and (stripped_code == "" or line.lstrip().startswith("//")):
                depth_before = depth_after
                continue
            current = []
        current.append(line)
        if depth_after == 0 and (stripped_code.endswith("}") or stripped_code.endswith(";")) and not stripped_code.startswith("#["):
            text = "\n".join(current)
            first_code = next(entry for entry in current if not entry.lstrip().startswith("#[") and not entry.lstrip().startswith("//"))
            match = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+(?=fn\b)|unsafe\s+|async\s+)*" + ITEM_KEYWORDS + r"\s+([A-Za-z_][A-Za-z0-9_]*)?", first_code)
            assert match, f"认不出条目：{first_code!r}"
            key = f"use:{normalized(text)}" if match.group(1) == "use" else f"{match.group(1)}:{match.group(2)}"
            assert key not in items, f"条目重名：{key}"
            items[key] = text
            order.append(key)
            current = None
        depth_before = depth_after
    assert current is None, "文件尾有没收尾的条目"
    return items, order


def module_lines(device_text):
    """装置里照抄模块的正文，去掉模块那一层的四格缩进；另交回 `sed -n '/^mod …{/,/^}/p'` 那一段的 sha256。"""
    lines = device_text.split("\n")
    start = lines.index(MODULE_HEADER)
    end = next(index for index in range(start + 1, len(lines)) if lines[index] == "}")
    extracted = "\n".join(lines[start:end + 1]) + "\n"
    body = lines[start + 1:end]
    assert all(line == "" or line.startswith("    ") for line in body), "模块正文有不到四格缩进的行"
    return [line[4:] for line in body], hashlib.sha256(extracted.encode()).hexdigest()


def split_function(function_text):
    """`known_bad_images` 拆成 `vec![` 之前的前导绑定与逐条条目（八格缩进的 `(` 到 `),`）。"""
    lines = function_text.split("\n")
    assert lines[0].startswith(FUNCTION_HEADER), lines[0]
    vec_line = lines.index("    vec![")
    prelude = "\n".join(lines[1:vec_line])
    entries = []
    current = None
    for line in lines[vec_line + 1:]:
        if line == "        (":
            assert current is None
            current = [line]
        elif current is not None:
            current.append(line)
            if line == "        ),":
                entries.append("\n".join(current))
                current = None
    assert current is None
    return prelude, entries


def target_of_entry(entry_text):
    for line in entry_text.split("\n")[1:]:
        stripped = line.strip()
        if stripped and not stripped.startswith("//"):
            return stripped.rstrip(",")
    raise AssertionError(entry_text)


def referenced_items(text, items, seen=None):
    """文本里出现的、模块里有的顶格条目（fn / const / static / type），连同它们再引到的，传递闭包。"""
    seen = set() if seen is None else seen
    for identifier in set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", text)):
        for keyword in ("fn", "const", "static", "type"):
            key = f"{keyword}:{identifier}"
            if key in items and key not in seen and key != "fn:known_bad_images":
                seen.add(key)
                referenced_items(items[key], items, seen)
    return seen


def crates_constant_values(image_text):
    return dict(re.findall(r'^pub const ([A-Z_0-9]+): &str = "([^"]+)";', image_text, re.MULTILINE))


def device_name_rule(test_text, device_text, crates_values):
    """装置 `entry_names_in_the_original_test_file` 的 Python 对照（只对照，装置自己的读数另由 known-bad-images 模式出）：
    常量名的映射从装置源码的 `match token` 臂里读，不另写一份。"""
    arms = {}
    for constant, crates_name, literal in re.findall(
        r'"([A-Z_0-9]+)" => \{?\s*(?:singlefs_checker::image::([A-Z_0-9]+)\s*\.to_string\(\)|"(I-[0-9.]+)"\.to_string\(\))',
        device_text,
    ):
        arms[constant] = crates_values[crates_name] if crates_name else literal
    inside = False
    names, literal_count, constant_count = [], 0, 0
    for line in test_text.split("\n"):
        if line.startswith(FUNCTION_HEADER):
            inside = True
            continue
        if inside and line == "}":
            break
        if not inside or not line.startswith(" " * 12) or line.startswith(" " * 13):
            continue
        token = line.strip().rstrip(",")
        if len(token) >= 2 and token.startswith('"') and token.endswith('"'):
            if token[1:-1].startswith("I-"):
                names.append(token[1:-1])
                literal_count += 1
        elif token and all(character.isupper() or character == "_" for character in token):
            names.append(arms.get(token, f"unresolved_constant:{token}"))
            constant_count += 1
    return names, literal_count, constant_count


def function_range_counts(test_text):
    lines = test_text.split("\n")
    start = next(index for index, line in enumerate(lines) if line.startswith(FUNCTION_HEADER))
    end = next(index for index in range(start + 1, len(lines)) if lines[index] == "}")
    region = lines[start:end + 1]
    return start + 1, end + 1, sum(line.count("Box::new(") for line in region), sum(1 for line in region if line == "        (")


def resolved_target(target, module_items, crates_values):
    if target.startswith('"'):
        return target.strip('"')
    local = module_items.get(f"const:{target}")
    if local is not None:
        value = re.search(r"=\s*(?:singlefs_checker::image::)?([A-Z_0-9]+|\"[^\"]+\")\s*;", local).group(1)
        return value.strip('"') if value.startswith('"') else crates_values[value]
    return crates_values[target]


def main():
    old_device_path, new_device_path, test_path, i4_path, image_path = sys.argv[1:6]
    old_device_text = open(old_device_path, encoding="utf-8").read()
    new_device_text = open(new_device_path, encoding="utf-8").read()
    test_bytes = open(test_path, "rb").read()
    test_text = test_bytes.decode("utf-8")
    crates_values = crates_constant_values(open(image_path, encoding="utf-8").read())
    print(f"name=known_bad_source path={test_path} sha256={hashlib.sha256(test_bytes).hexdigest()}")

    old_lines, old_module_sha256 = module_lines(old_device_text)
    new_lines, new_module_sha256 = module_lines(new_device_text)
    print(f"name=old_module sha256={old_module_sha256} expected={EXPECTED_OLD_MODULE_SHA256} "
          f"equal={old_module_sha256 == EXPECTED_OLD_MODULE_SHA256}")
    print(f"name=new_module sha256={new_module_sha256}")

    rule_names, literal_count, constant_count = device_name_rule(test_text, new_device_text, crates_values)
    unresolved = sum(1 for name in rule_names if name.startswith("unresolved_constant:"))
    first_line, last_line, box_new_count, paren_line_count = function_range_counts(test_text)
    print(f"name=count method=device_name_rule_python_port count={len(rule_names)} unresolved={unresolved} "
          f"literal={literal_count} constant={constant_count}")
    print(f"name=count method=box_new lines={first_line}-{last_line} count={box_new_count}")
    print(f"name=count method=eight_space_open_paren lines={first_line}-{last_line} count={paren_line_count}")
    counts_equal = len(rule_names) == box_new_count == paren_line_count and unresolved == 0

    old_items, _ = top_level_items(old_lines)
    new_items, new_order = top_level_items(new_lines)
    test_items, _ = top_level_items(test_text.split("\n"))
    old_prelude, old_entries = split_function(old_items["fn:known_bad_images"])
    new_prelude, new_entries = split_function(new_items["fn:known_bad_images"])
    print(f"name=entries old={len(old_entries)} new={len(new_entries)}")

    recopy_differences = 0
    prelude_identical = normalized(old_prelude) == normalized(new_prelude)
    recopy_differences += 0 if prelude_identical else 1
    print(f"name=known_bad_prelude_identity identical={prelude_identical}")
    for index, old_entry in enumerate(old_entries):
        new_entry = new_entries[index] if index < len(new_entries) else ""
        old_target, new_target = target_of_entry(old_entry), target_of_entry(new_entry) if new_entry else "-"
        target_identical = old_target == new_target
        text_identical = normalized(old_entry) == normalized(new_entry)
        helpers = sorted(referenced_items(old_entry, old_items) | referenced_items(new_entry, new_items))
        helpers_identical = all(
            key in old_items and key in new_items and normalized(old_items[key]) == normalized(new_items[key])
            for key in helpers
        )
        identical = target_identical and text_identical and helpers_identical
        recopy_differences += 0 if identical else 1
        print(f"name=known_bad_entry_identity index={index} target={old_target} identical={identical} "
              f"target_identical={target_identical} text_identical={text_identical} "
              f"helpers={len(helpers)} helpers_identical={helpers_identical}")
    for index in range(len(old_entries), len(new_entries)):
        print(f"name=known_bad_entry_added index={index} target={target_of_entry(new_entries[index])}")

    item_differences = 0
    for key in new_order:
        if key.startswith("use:"):
            first_line = new_items[key].split("\n")[0].strip()
            print(f"name=known_bad_use_in_the_file use={first_line!r} present={key in test_items}")
            continue
        if key in ITEMS_NOT_COMPARED:
            continue
        identical = key in test_items and normalized(new_items[key]) == normalized(test_items[key])
        item_differences += 0 if identical else 1
        print(f"name=known_bad_item_identity item={key} identical={identical}")

    resolved_here = [resolved_target(target_of_entry(entry), new_items, crates_values) for entry in new_entries]
    names_equal = resolved_here == rule_names
    print(f"name=known_bad_entry_names_resolved_equal_the_file_in_order equal={names_equal} count={len(resolved_here)}")
    for index, name in enumerate(resolved_here):
        print(f"name=known_bad_entry_name index={index} name={name}")

    old_resolved = [resolved_target(target_of_entry(entry), old_items, crates_values) for entry in old_entries]
    i4_names = [line.split("\t")[1] for line in open(i4_path, encoding="utf-8").read().split("\n") if line]
    i4_indices = [int(line.split("\t")[0]) for line in open(i4_path, encoding="utf-8").read().split("\n") if line]
    i4_equal = i4_names == old_resolved and i4_indices == list(range(len(i4_names)))
    print(f"name=i4_target_names equal={i4_equal} rows={len(i4_names)} old_module_entries={len(old_resolved)}")

    print(f"name=known_bad_recopy_verdict counts_equal={counts_equal} recopy_differences={recopy_differences} "
          f"item_differences={item_differences} names_resolved_equal={names_equal} i4_target_names_equal={i4_equal}")
    return 0 if counts_equal and recopy_differences == 0 and item_differences == 0 and names_equal and i4_equal else 3


if __name__ == "__main__":
    sys.exit(main())
