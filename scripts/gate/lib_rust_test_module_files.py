#!/usr/bin/env python3
# admission: always 读的是这一次给的源码根与替换文本，上一次的结论不替这一次作保
# run-condition: none 只读给定根下 crates/<包>/src 里的 .rs 文本，自证只在内存与临时目录里摆几份几行的小源码，不起子进程
# selftest-serial: 每格只判几份几行的小源码、整份不到一秒，拆成并行比起线程还慢
"""整份算测试代码的 Rust 源文件：门禁与补丁判法共用的一份认法，只用标准库。

一份 crates/<包>/src/ 下的 .rs 整份算测试代码（只在按测试编译时才进编译，不是产品代码），是下面三种之一：
  ① 文件自己带内层属性 #![cfg(test)]；
  ② 引到它的每一处文件模块声明（父模块文件里的 `mod <名>;`）都算测试代码：声明条目上带 TEST_ATTRIBUTE 认的属性（#[cfg(test)]，叠了别的属性也认），
     或外围某一层内联模块（`mod <名> { … }`）带它；
  ③ 引到它的每一处声明所在的文件自己整份算测试代码（祖先任一层）。
  没有任何一处声明引到它的、父文件读不到的：不算，调用方照产品代码判。
声明引到哪份文件照 Rust 的找文件规矩：父模块文件是 crate 入口（src/lib.rs、src/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs）或 mod.rs 的，
  `mod <名>;` 找它所在目录下的 <名>.rs 与 <名>/mod.rs；别的 <a>.rs 找同目录下 <a>/ 里的；内联模块里的声明，目录带上外围的内联模块名；
  `#[path = "…"]` 按 path 落：不在内联模块里时相对父文件所在目录，在内联模块里时相对带上内联模块名的目录。
只在每个包自己的 src/ 里找（路径相对仓根，形如 crates/<包>/src/…）；tests/、build.rs 与别的目录不判。

调它的：scripts/gate/apply-writer-patch.py（补丁新加的断言落在整份算测试的文件里不算走不到的理由；blank_out、attribute_end 与两条属性正则同由这里取）、
  .claude/gate.d/code-tooling.sh 的 heavy-thread-defaults 格（整份算测试的文件不扫线程默认值）、
  .claude/gate.d/checker-independence-and-sync.sh 的 checker-implementation-disjoint 格（共享常量模块里整份算测试的文件不扫分支循环）。
  两道门禁阶段按文件路径载入这一份（importlib 的 spec_from_file_location，调用方的 sys.path 不一定有本目录）。
入口：
  whole_test_files(root, paths, replaced=None) -> {路径}                paths 里整份算测试的（只看 crates/<包>/src/ 下的 .rs，别的路径不算）；
      root 是被判那棵树的根；replaced 是 {仓根相对路径: 套上补丁之后的全文，或 None（补丁删了它）}：落在这几个路径上的取替换文本，没在里面的读 root 里的原文
  whole_test_files_of_sources(sources) -> {路径}                        sources 是 {路径: 全文}，一个包 src/ 下的全部 .rs；交其中整份算测试的
  crate_source_texts(root, crate, replaced=None) -> {路径: 全文}        <包目录>/src 下的全部 .rs（replaced 同上；读不了的文件不收）
  module_declarations(text, cleaned) -> [(模块名, #[path] 的值或 None, 声明算不算测试代码, 外围内联模块名的元组)]   只收 `mod <名>;` 这种文件模块的声明
  module_directory(path)、declared_files(path, declaration)、declared_as_test(path, claims, inner_test, seen)   找文件与递归判的几步
  blank_out(text)、attribute_end(text, start)                           去掉注释与字面量里的字（行号不变）、配平一条 #[…] 属性
  lib_rust_test_module_files.py --selftest

弄坏开关 LIB_RUST_TEST_MODULE_FILES_BREAK=<项>（只给证红用；--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认）。项名、弄坏哪一处、打开后自证红在哪几格：
  LIB_RUST_TEST_MODULE_FILES_BREAK=declaration-attribute-unread  mod 声明上的属性一概不读，声明都算产品代码
      「声明带 #[cfg(test)] 的文件模块整份算测试，没带属性的不算」「声明上叠了别的属性：#[cfg(test)] 在前在后都认」「祖先一层声明带 #[cfg(test)]：它声明的下一层文件也整份算测试」
      「目录式模块 folder/mod.rs 声明的文件跟着算测试」「入口文件 main.rs 与 bin 下的入口文件声明的文件整份算测试」
      「#[cfg(test)] 与 #[path = …] 叠在同一条声明上：path 落的文件整份算测试」「内联测试模块里的 mod 声明引到的文件整份算测试，产品内联模块里的不算」
      「读根下的源码：只判 crates/<包>/src 下的，别处的 .rs 不判」「补丁改过的源码取替换文本，补丁新加的文件也在」
  LIB_RUST_TEST_MODULE_FILES_BREAK=stacked-attributes-last-only  声明上叠了几条属性时只读紧挨着 mod 的那一条
      「声明上叠了别的属性：#[cfg(test)] 在前在后都认」「#[cfg(test)] 与 #[path = …] 叠在同一条声明上：path 落的文件整份算测试」
  LIB_RUST_TEST_MODULE_FILES_BREAK=inline-module-test-ignored  外围内联模块带 #[cfg(test)] 不算，只看声明自己的属性
      「内联测试模块里的 mod 声明引到的文件整份算测试，产品内联模块里的不算」
  LIB_RUST_TEST_MODULE_FILES_BREAK=path-attribute-ignored  #[path = "…"] 不读，声明一律按模块名找文件
      「#[cfg(test)] 与 #[path = …] 叠在同一条声明上：path 落的文件整份算测试」「同一份文件被一条 #[cfg(test)] 声明与一条没带属性的声明引到：不算测试」
  LIB_RUST_TEST_MODULE_FILES_BREAK=ancestor-file-unchecked  只看声明自己带不带 #[cfg(test)]，声明所在的文件整份算测试不往下传
      「祖先一层声明带 #[cfg(test)]：它声明的下一层文件也整份算测试」「目录式模块 folder/mod.rs 声明的文件跟着算测试」
      「整份文件自己带 #![cfg(test)]：它与它声明的文件整份算测试，没人声明它也算」
  LIB_RUST_TEST_MODULE_FILES_BREAK=inner-attribute-unchecked  内层属性 #![cfg(test)] 不认
      「整份文件自己带 #![cfg(test)]：它与它声明的文件整份算测试，没人声明它也算」
  LIB_RUST_TEST_MODULE_FILES_BREAK=any-declaration-suffices  引到它的几处声明里有一处算测试就算（正确的是每一处都得算）
      「同一份文件被一条 #[cfg(test)] 声明与一条没带属性的声明引到：不算测试」
  LIB_RUST_TEST_MODULE_FILES_BREAK=mod-rs-directory-unchecked  mod.rs 当普通的 <名>.rs，它里面的声明去找 mod/ 下的文件
      「目录式模块 folder/mod.rs 声明的文件跟着算测试」
  LIB_RUST_TEST_MODULE_FILES_BREAK=crate-root-directory-unchecked  crate 入口文件（lib.rs、main.rs、bin 下的入口）当普通的 <名>.rs，它里面的声明去找 <名>/ 下的文件
      「声明带 #[cfg(test)] 的文件模块整份算测试，没带属性的不算」「声明上叠了别的属性：#[cfg(test)] 在前在后都认」「祖先一层声明带 #[cfg(test)]：它声明的下一层文件也整份算测试」
      「目录式模块 folder/mod.rs 声明的文件跟着算测试」「入口文件 main.rs 与 bin 下的入口文件声明的文件整份算测试」
      「内联测试模块里的 mod 声明引到的文件整份算测试，产品内联模块里的不算」「读根下的源码：只判 crates/<包>/src 下的，别处的 .rs 不判」「补丁改过的源码取替换文本，补丁新加的文件也在」
  LIB_RUST_TEST_MODULE_FILES_BREAK=comments-and-strings-scanned  不去掉注释与字符串字面量就扫声明
      「注释与字符串字面量里写的 mod 声明不算声明」
  LIB_RUST_TEST_MODULE_FILES_BREAK=replaced-text-ignored  replaced 不看，一律读 root 里的原文
      「补丁改过的源码取替换文本，补丁新加的文件也在」「补丁删掉的源码不读，它的声明不再引文件」
  LIB_RUST_TEST_MODULE_FILES_BREAK=deleted-file-still-read  replaced 里值是 None（补丁删了）的文件仍读 root 里的原文
      「补丁删掉的源码不读，它的声明不再引文件」
"""
import os
import re
import shutil
import sys
import tempfile

BREAK_VARIABLE = "LIB_RUST_TEST_MODULE_FILES_BREAK"
BREAK_VARIABLES = (BREAK_VARIABLE,)
EXIT_USAGE = 2

TEST_ATTRIBUTE = re.compile(r"#\[\s*(?:test|cfg\(\s*test\s*\)|cfg\(\s*all\(\s*test\b[^\]]*|[A-Za-z0-9_]+::test(?:\([^\]]*\))?)\s*\]\Z")   # 去掉注释与字面量之后的一条外层属性，是测试属性
INNER_TEST_ATTRIBUTE = re.compile(r"#!\[\s*cfg\(\s*test\s*\)\s*\]")   # 整份文件都是测试代码的内层属性
MODULE_TOKEN = re.compile(r"#\[|\{|\}|;|(?<![A-Za-z0-9_])mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*([;{])")   # module_declarations 扫的记号：外层属性、大括号、分号、mod 声明（后跟 ; 是文件模块，后跟 { 是内联模块）
FILE_MODULE_DECLARATION = re.compile(r"(?<![A-Za-z0-9_])mod\s+[A-Za-z_][A-Za-z0-9_]*\s*;")   # 源码里有没有文件模块声明：没有声明也没有内层属性的文件不必逐字扫
PATH_ATTRIBUTE = re.compile(r'#\[\s*path\s*=\s*"((?:[^"\\]|\\.)*)"\s*\]\Z')   # mod 声明上的 #[path = "…"]
MODULE_ROOT_FILE = re.compile(r"^crates/[^/]+/src/(?:lib\.rs|main\.rs|bin/[^/]+\.rs|bin/[^/]+/main\.rs)$")   # 包里每个 crate 的入口文件：模块声明按它所在目录找文件，与 mod.rs 同
CRATE_SOURCE_PATH = re.compile(r"^(crates/[^/]+)/src/.+\.rs$")   # 第一组是包目录
RAW_STRING_START = re.compile(r'r(#*)"')
CHAR_LITERAL = re.compile(r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|.)|[^\\'\n])'")


def break_is(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def blank_out(text):
    """去掉注释与字符串、字符字面量里的字（换成空格，换行留着，行号不变）。"""
    pieces, index, length = [], 0, len(text)

    def blanks(fragment):
        return re.sub(r"[^\n]", " ", fragment)

    while index < length:
        character = text[index]
        raw = RAW_STRING_START.match(text, index) if character == "r" and (index == 0 or text[index - 1] == "b" or not (text[index - 1].isalnum() or text[index - 1] == "_")) else None
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            pieces.append(blanks(text[index:end]))
            index = end
        elif text.startswith("/*", index):
            depth, end = 1, index + 2
            while end < length and depth:
                if text.startswith("/*", end):
                    depth, end = depth + 1, end + 2
                elif text.startswith("*/", end):
                    depth, end = depth - 1, end + 2
                else:
                    end += 1
            pieces.append(blanks(text[index:end]))
            index = end
        elif raw is not None:
            close = text.find('"' + raw.group(1), raw.end())
            end = length if close < 0 else close + 1 + len(raw.group(1))
            pieces.append(blanks(text[index:end]))
            index = end
        elif character == '"':
            end = index + 1
            while end < length and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            end = min(end + 1, length)
            pieces.append(blanks(text[index:end]))
            index = end
        elif character == "'" and CHAR_LITERAL.match(text, index):
            end = CHAR_LITERAL.match(text, index).end()
            pieces.append(blanks(text[index:end]))
            index = end
        else:
            pieces.append(character)
            index += 1
    return "".join(pieces)


def attribute_end(text, start):
    """text[start:] 以 #[ 起头：配平方括号，→ 属性结束处（] 之后）的下标。"""
    depth = 0
    for position in range(start + 1, len(text)):
        if text[position] == "[":
            depth += 1
        elif text[position] == "]":
            depth -= 1
            if depth == 0:
                return position + 1
    return len(text)


def module_declarations(text, cleaned):
    """一份 .rs 的源码（text）与去掉注释和字面量之后的同一份（cleaned，等长）→ [(模块名, #[path] 的值或 None, 声明算不算测试代码, 外围内联模块名的元组)]：只收 `mod <名>;` 这种文件模块的声明。
    声明算测试代码：条目上有 TEST_ATTRIBUTE 认的属性（叠了别的属性也认），或外围某一层内联模块带它。"""
    declarations, stack, attributes, position = [], [], [], 0
    while True:
        token = MODULE_TOKEN.search(cleaned, position)
        if token is None:
            return declarations
        position = token.end()
        kind = token.group(0)
        if kind == "#[":
            end = attribute_end(cleaned, token.start())
            attributes.append((cleaned[token.start():end], text[token.start():end]))
            position = end
            continue
        if token.group(1) is None:
            if kind == "{":
                stack.append((None, False))
            elif kind == "}" and stack:
                stack.pop()
            attributes = []
            continue
        considered = attributes[-1:] if break_is("stacked-attributes-last-only") else attributes
        is_test = not break_is("declaration-attribute-unread") and any(TEST_ATTRIBUTE.match(cleaned_attribute) for cleaned_attribute, _original in considered)
        if token.group(2) == "{":
            stack.append((token.group(1), is_test))
        else:
            path_values = [] if break_is("path-attribute-ignored") else [matched.group(1) for _cleaned, original in attributes for matched in [PATH_ATTRIBUTE.match(original)] if matched]
            inside_test_module = not break_is("inline-module-test-ignored") and any(flag for _name, flag in stack)
            declarations.append((token.group(1), path_values[0] if path_values else None, is_test or inside_test_module,
                                 tuple(name for name, _flag in stack if name)))
        attributes = []


def module_directory(path):
    """一份模块文件里 `mod <名>;` 找文件的那个目录：入口文件（MODULE_ROOT_FILE）与 mod.rs 是它自己所在的目录，别的 <名>.rs 是同目录下的 <名>/。"""
    directory, name = os.path.split(path)
    if (name == "mod.rs" and not break_is("mod-rs-directory-unchecked")) or (MODULE_ROOT_FILE.match(path) and not break_is("crate-root-directory-unchecked")):
        return directory
    return os.path.join(directory, name[:-len(".rs")])


def declared_files(path, declaration):
    """path 这份文件里的一处 mod 声明（module_declarations 的一项）可能引到的文件（仓根相对）：没有 #[path] 的是 <目录>/<名>.rs 与 <目录>/<名>/mod.rs，目录带上外围内联模块名；
    有 #[path] 的按它落：不在内联模块里时相对这份文件所在的目录，在内联模块里时相对带上内联模块名的目录。"""
    name, path_value, _is_test, inline = declaration
    inside = os.path.join(module_directory(path), *inline)
    if path_value is not None:
        return [os.path.normpath(os.path.join(inside if inline else os.path.dirname(path), path_value))]
    return [os.path.normpath(os.path.join(inside, name + ".rs")), os.path.normpath(os.path.join(inside, name, "mod.rs"))]


def crate_source_texts(root, crate, replaced=None):
    """被判那棵树里 <包目录>/src 下的全部 .rs → {路径: 全文}：replaced 里有的取替换文本（值是 None 的是补丁删掉的，没有这一份），没有的读 root 里的原文。
    弄坏开关 replaced-text-ignored 下一律读原文，deleted-file-still-read 下补丁删掉的也读原文。"""
    replaced = {} if break_is("replaced-text-ignored") else (replaced or {})
    prefix = crate + "/src/"
    paths = {path for path in replaced if path.startswith(prefix) and path.endswith(".rs")}
    for directory, _directories, names in os.walk(os.path.join(root, crate, "src")):
        paths.update(os.path.relpath(os.path.join(directory, name), root) for name in names if name.endswith(".rs"))
    files = {}
    for path in sorted(paths):
        if path in replaced and (replaced[path] is not None or not break_is("deleted-file-still-read")):
            if replaced[path] is not None:
                files[path] = replaced[path]
            continue
        try:
            with open(os.path.join(root, path), encoding="utf-8", errors="replace") as handle:
                files[path] = handle.read()
        except OSError:
            continue
    return files


def declared_as_test(path, claims, inner_test, seen):
    """path 这份文件整份算测试代码：自己有 #![cfg(test)]（在 inner_test 里），或引到它的每一处声明（claims[path] 里的 (声明所在的文件, 声明算不算测试代码)）都算测试代码、
    或声明所在的文件自己整份算测试代码（祖先任一层；seen 是走过的路径，防环）。没有任何一处声明引到它的不算。"""
    if path in inner_test:
        return True
    holders = claims.get(path)
    if not holders or path in seen:
        return False
    verdicts = [is_test or (not break_is("ancestor-file-unchecked") and declared_as_test(parent, claims, inner_test, seen + (path,))) for parent, is_test in holders]
    return any(verdicts) if break_is("any-declaration-suffices") else all(verdicts)


def whole_test_files_of_sources(sources):
    """sources：{路径: 全文}，一个包 src/ 下的全部 .rs（路径相对仓根）→ 其中整份算测试代码的路径集合。"""
    claims, inner_test = {}, set()
    for path, text in sources.items():
        if not (FILE_MODULE_DECLARATION.search(text) or "#![" in text):
            continue
        cleaned = text if break_is("comments-and-strings-scanned") else blank_out(text)
        if INNER_TEST_ATTRIBUTE.search(cleaned) and not break_is("inner-attribute-unchecked"):
            inner_test.add(path)
        for declaration in module_declarations(text, cleaned):
            for target in declared_files(path, declaration):
                claims.setdefault(target, []).append((path, declaration[2]))
    return {path for path in sources if declared_as_test(path, claims, inner_test, ())}


def whole_test_files(root, paths, replaced=None):
    """paths 里整份算测试代码的 → 集合。只看 crates/<包>/src/ 下的 .rs（CRATE_SOURCE_PATH），别的路径不算；每个包的 src/ 读一遍、判一遍。"""
    by_crate = {}
    for path in paths:
        matched = CRATE_SOURCE_PATH.match(path)
        if matched:
            by_crate.setdefault(matched.group(1), []).append(path)
    found = set()
    for crate, crate_paths in by_crate.items():
        test_files = whole_test_files_of_sources(crate_source_texts(root, crate, replaced))
        found.update(path for path in crate_paths if os.path.normpath(path) in test_files)
    return found


SAMPLE_SOURCE = "crates/demo/src/"
# 内存里的格：(格名, {相对 crates/demo/src/ 的路径: 全文}, 期望整份算测试的路径)
IN_MEMORY_CELLS = (
    ("声明带 #[cfg(test)] 的文件模块整份算测试，没带属性的不算",
     {"lib.rs": "#[cfg(test)]\nmod tests;\nmod helpers;\n", "tests.rs": "fn checked() {}\n", "helpers.rs": "fn used() {}\n"},
     {"tests.rs"}),
    ("声明上叠了别的属性：#[cfg(test)] 在前在后都认",
     {"lib.rs": "#[cfg(test)]\n#[allow(dead_code)]\nmod test_then_allow;\n#[allow(dead_code)]\n#[cfg(test)]\nmod allow_then_test;\n",
      "test_then_allow.rs": "fn before() {}\n", "allow_then_test.rs": "fn after() {}\n"},
     {"test_then_allow.rs", "allow_then_test.rs"}),
    ("祖先一层声明带 #[cfg(test)]：它声明的下一层文件也整份算测试",
     {"lib.rs": "#[cfg(test)]\nmod outer;\n", "outer.rs": "mod inner;\n", "outer/inner.rs": "fn deep() {}\n"},
     {"outer.rs", "outer/inner.rs"}),
    ("目录式模块 folder/mod.rs 声明的文件跟着算测试",
     {"lib.rs": "#[cfg(test)]\nmod folder;\n", "folder/mod.rs": "mod leaf;\n", "folder/leaf.rs": "fn leaf_check() {}\n"},
     {"folder/mod.rs", "folder/leaf.rs"}),
    ("入口文件 main.rs 与 bin 下的入口文件声明的文件整份算测试",
     {"main.rs": "#[cfg(test)]\nmod from_main;\n", "from_main.rs": "fn one() {}\n",
      "bin/tool.rs": "#[cfg(test)]\nmod from_bin;\n", "bin/from_bin.rs": "fn two() {}\n",
      "bin/other/main.rs": "#[cfg(test)]\nmod from_bin_directory;\n", "bin/other/from_bin_directory.rs": "fn three() {}\n"},
     {"from_main.rs", "bin/from_bin.rs", "bin/other/from_bin_directory.rs"}),
    ("#[cfg(test)] 与 #[path = …] 叠在同一条声明上：path 落的文件整份算测试",
     {"lib.rs": "#[cfg(test)]\n#[path = \"support/checks.rs\"]\nmod checks;\n", "support/checks.rs": "fn support() {}\n"},
     {"support/checks.rs"}),
    ("内联测试模块里的 mod 声明引到的文件整份算测试，产品内联模块里的不算",
     {"lib.rs": "#[cfg(test)]\nmod in_tests {\n    mod leaf;\n}\nmod in_product {\n    mod leaf;\n}\n",
      "in_tests/leaf.rs": "fn test_leaf() {}\n", "in_product/leaf.rs": "fn product_leaf() {}\n"},
     {"in_tests/leaf.rs"}),
    ("整份文件自己带 #![cfg(test)]：它与它声明的文件整份算测试，没人声明它也算",
     {"lib.rs": "mod gated;\n", "gated.rs": "#![cfg(test)]\nmod deep;\n", "gated/deep.rs": "fn gated_deep() {}\n", "solo.rs": "#![cfg(test)]\nfn alone() {}\n"},
     {"gated.rs", "gated/deep.rs", "solo.rs"}),
    ("同一份文件被一条 #[cfg(test)] 声明与一条没带属性的声明引到：不算测试",
     {"lib.rs": "#[cfg(test)]\nmod shared;\n#[path = \"shared.rs\"]\nmod again;\n", "shared.rs": "fn both() {}\n"},
     set()),
    ("注释与字符串字面量里写的 mod 声明不算声明",
     {"lib.rs": "// #[cfg(test)]\n// mod ghost;\nconst NOTE: &str = \"#[cfg(test)] mod text_ghost;\";\n",
      "ghost.rs": "fn in_comment() {}\n", "text_ghost.rs": "fn in_text() {}\n", "orphan.rs": "fn nobody_declares_me() {}\n"},
     set()),
)
# 临时目录里的格：(格名, 根下的文件 {仓根相对路径: 全文}, 要判的路径, replaced, 期望整份算测试的路径)
DISK_CELLS = (
    ("读根下的源码：只判 crates/<包>/src 下的，别处的 .rs 不判",
     {SAMPLE_SOURCE + "lib.rs": "#[cfg(test)]\nmod tests;\nmod helpers;\n", SAMPLE_SOURCE + "tests.rs": "fn checked() {}\n", SAMPLE_SOURCE + "helpers.rs": "fn used() {}\n",
      "crates/demo/tests/integration.rs": "#![cfg(test)]\nfn integration() {}\n", "scripts/tool.rs": "#![cfg(test)]\nfn tool() {}\n"},
     [SAMPLE_SOURCE + "tests.rs", SAMPLE_SOURCE + "helpers.rs", SAMPLE_SOURCE + "lib.rs", "crates/demo/tests/integration.rs", "scripts/tool.rs"], None,
     {SAMPLE_SOURCE + "tests.rs"}),
    ("补丁改过的源码取替换文本，补丁新加的文件也在",
     {SAMPLE_SOURCE + "lib.rs": "mod tests;\n", SAMPLE_SOURCE + "tests.rs": "fn checked() {}\n"},
     [SAMPLE_SOURCE + "tests.rs", SAMPLE_SOURCE + "added.rs"],
     {SAMPLE_SOURCE + "lib.rs": "#[cfg(test)]\nmod tests;\n#[cfg(test)]\nmod added;\n", SAMPLE_SOURCE + "added.rs": "fn added() {}\n"},
     {SAMPLE_SOURCE + "tests.rs", SAMPLE_SOURCE + "added.rs"}),
    ("补丁删掉的源码不读，它的声明不再引文件",
     {SAMPLE_SOURCE + "lib.rs": "#[cfg(test)]\nmod tests;\n", SAMPLE_SOURCE + "tests.rs": "fn checked() {}\n"},
     [SAMPLE_SOURCE + "tests.rs"], {SAMPLE_SOURCE + "lib.rs": None},
     set()),
)


def judged_on_disk(disk_files, paths, replaced):
    """disk_files 写进临时目录当根，判 paths 里整份算测试的；临时目录判完就删。"""
    root = tempfile.mkdtemp(prefix="lib-rust-test-module-files-selftest-")
    try:
        for relative, text in disk_files.items():
            target = os.path.join(root, relative)
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "w", encoding="utf-8") as handle:
                handle.write(text)
        return whole_test_files(root, paths, replaced)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, cells = [], 0

    def check(label, actual, expected):
        nonlocal cells
        cells += 1
        if actual != expected:
            failures.append(f"{label}：期望 {sorted(expected)}，实际 {sorted(actual)}")

    for label, files, expected_names in IN_MEMORY_CELLS:
        sources = {SAMPLE_SOURCE + name: text for name, text in files.items()}
        check(label, whole_test_files_of_sources(sources), {SAMPLE_SOURCE + name for name in expected_names})
    for label, disk_files, paths, replaced, expected in DISK_CELLS:
        check(label, judged_on_disk(disk_files, paths, replaced), expected)
    if failures:
        for failure in failures:
            print(f"  ✗ lib_rust_test_module_files 自检：{failure}")  # gate-lint:detail
        print(f"     → 看 module_declarations、module_directory、declared_files、declared_as_test、crate_source_texts；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_rust_test_module_files 自检通过（查了 {cells} 格：#[cfg(test)] 声明引到的文件整份算测试、叠了别的属性、祖先一层声明、mod.rs 目录式、入口文件与 bin 入口、"
          "#[path] 叠 #[cfg(test)]、内联测试模块、内层属性 #![cfg(test)]、一条测试声明加一条普通声明、注释与字符串里的声明、"
          "读根下源码并只判包的 src、补丁替换文本与新加的文件、补丁删掉的文件）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    print("  ✗ 用法：lib_rust_test_module_files.py --selftest（别的脚本按文件路径导入它，调 whole_test_files）", file=sys.stderr)
    print("     → 怎么办：入口与认法写在文件头；这份库没有命令行子命令", file=sys.stderr)
    return EXIT_USAGE


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
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/lib_rust_test_module_files.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
    from project_preflight import preflight  # noqa: E402
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
