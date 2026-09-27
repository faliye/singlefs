"""把被判的 lib_heavy_tests.py、heavy-test-guard.sh（连同同目录的 lib_shell_words.py）与 admission.py 拷进副本目录，打上我提的改法 P1–P9
（只在我的模型上量过、被攻过零轮），再写出每份的 diff。原件只读。
用法：python3 make_fix_copy.py <副本目录>   之后 K1_HOOK=<副本>/.claude/hooks/heavy-test-guard.sh K1_ADMISSION=<副本>/research/scripts/admission.py 跑探针。"""
import difflib
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

target = os.path.abspath(sys.argv[1])
os.makedirs(os.path.join(target, ".claude/hooks"), exist_ok=True)
os.makedirs(os.path.join(target, "research/scripts"), exist_ok=True)
for name in ("heavy-test-guard.sh", "lib_heavy_tests.py", "lib_shell_words.py"):
    shutil.copy(os.path.join(pc.REPOSITORY, ".claude/hooks", name), os.path.join(target, ".claude/hooks", name))
shutil.copy(os.path.join(pc.REPOSITORY, "research/scripts/admission.py"), os.path.join(target, "research/scripts/admission.py"))


def patch(relative, replacements):
    path = os.path.join(target, relative)
    with open(path, encoding="utf-8") as handle:
        original = handle.read()
    text = original
    for label, old, new in replacements:
        if text.count(old) != 1:
            sys.exit(f"{label}：旧串在 {relative} 里命中 {text.count(old)} 次，不是 1 次")
        text = text.replace(old, new)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    diff = "".join(difflib.unified_diff(original.splitlines(True), text.splitlines(True), f"a/{relative}", f"b/{relative}"))
    with open(os.path.join(target, os.path.basename(relative) + ".patch"), "w", encoding="utf-8") as handle:
        handle.write(diff)


LIB = ".claude/hooks/lib_heavy_tests.py"
patch(LIB, [
    ("P1 短选项合写 + P2 systemd-run -E 带进里面那条命令",
     '''    options_with_value, working_directory = LAUNCHER_OPTIONS_WITH_VALUE[name], None
    while position < len(words):''',
     '''    options_with_value, working_directory, assignments = LAUNCHER_OPTIONS_WITH_VALUE[name], None, []
    while position < len(words):'''),
    ("P1 短选项合写",
     '''        option, has_attached_value, attached_value = word.partition("=")
        value = attached_value if has_attached_value else (words[position + 1] if position + 1 < len(words) else "")
        if name == "systemd-run" and option == "--working-directory":
            working_directory = value
''',
     '''        option, has_attached_value, attached_value = word.partition("=")
        if not word.startswith("--") and len(word) > 2 and not has_attached_value:
            # 短选项合写（-fo <文件>、-xw 10、-qu <名>）：逐个字母看，头一个要值的字母之后剩下的是值，没剩下就取下一个词
            for index, letter in enumerate(word[1:]):
                if "-" + letter in options_with_value:
                    option, attached_value = "-" + letter, word[index + 2:]
                    has_attached_value = bool(attached_value)
                    break
        value = attached_value if has_attached_value else (words[position + 1] if position + 1 < len(words) else "")
        if name == "systemd-run" and option == "--working-directory":
            working_directory = value
        if name == "systemd-run" and option in ("-E", "--setenv") and "=" in value:
            assignments.append(value)
'''),
    ("P2 systemd-run -E 带进里面那条命令",
     "    return (words[position:], working_directory) if position < len(words) else None\n",
     "    return (assignments + words[position:], working_directory) if position < len(words) else None\n"),
    ("P3 libtest 带值的选项", 'LIST_ONLY_TEST_ARGUMENT = "--list"\n',
     'LIST_ONLY_TEST_ARGUMENT = "--list"\n# libtest 带一个值的选项：--skip --list 里的 --list 是 --skip 的值，不是「只列」\n'
     'LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}\n\n\n'
     'def libtest_lists_only(arguments):\n    index = 0\n    while index < len(arguments):\n'
     '        if arguments[index] == LIST_ONLY_TEST_ARGUMENT:\n            return True\n'
     '        index += 2 if arguments[index] in LIBTEST_OPTIONS_WITH_VALUE else 1\n    return False\n'),
    ("P3 cargo 那一处", "    if LIST_ONLY_TEST_ARGUMENT in libtest_arguments:\n", "    if libtest_lists_only(libtest_arguments):\n"),
    ("P3 测试二进制那一处", "        if LIST_ONLY_TEST_ARGUMENT in arguments:\n", "        if libtest_lists_only(arguments):\n"),
    ("P10 --config 的值按 TOML 读，认带引号的 runner 键",
     "def runner_of_test_binaries(command, environment):\n",
     "def configuration_names_runner(value):\n"
     "    \"\"\"--config 的一个 KEY=VALUE 按 TOML 读：定了 target.<任何>.runner 交 True；读不成 TOML 的交 False（交给正则那一道）。\"\"\"\n"
     "    try:\n        parsed = tomllib.loads(value)\n    except ValueError:\n        return False\n"
     "    targets = parsed.get(\"target\") if isinstance(parsed.get(\"target\"), dict) else {}\n"
     "    return any(isinstance(table, dict) and \"runner\" in table for table in targets.values())\n\n\n"
     "def runner_of_test_binaries(command, environment):\n"),
    ("P10 用上", "    if any(RUNNER_CONFIGURATION.search(value) for value in command.configuration_values):\n",
     "    if any(RUNNER_CONFIGURATION.search(value) or configuration_names_runner(value) for value in command.configuration_values):\n"),
    ("P5 判不出按没标算", 'return module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function) is False',
     'return module.test_function_is_marked_ignored(package_directory, ".", case.target, case.function) is not True'),
])
ADM = "research/scripts/admission.py"
patch(ADM, [
    ("P4b # 与 [ 之间许空白", 'IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\\[\\s*ignore\\s*(?:\\]|=)")', 'IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\\s*\\[\\s*ignore\\s*(?:\\]|=)")'),
    ("P6 带路径的宏名与 env!", 'COMPILE_TIME_CONCATENATED_INCLUDE = re.compile(r"\\binclude(?:_str|_bytes)?!\\s*[(\\[{]\\s*concat!")',
     'COMPILE_TIME_CONCATENATED_INCLUDE = re.compile(r"\\binclude(?:_str|_bytes)?!\\s*[(\\[{]\\s*(?:(?:::)?\\s*[A-Za-z_][A-Za-z0-9_]*\\s*::\\s*)*(?:concat|env|option_env)!")'),
    ("P4 每一处同名定义", "def attributes_mark_ignored(attributes):\n",
     '''def attributes_before_every_definition(code, function):
    """同名的每一处 `fn <function>(`（cfg 二选一、子模块里同名的都算）前面那串属性；一处都没有交 None。"""
    lists = []
    for found in re.finditer(r"\\bfn\\s+" + re.escape(function) + r"\\s*\\(", code):
        head = code[:found.start()].rstrip()
        while True:
            qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
            if not qualifier:
                break
            head = head[:qualifier.start()].rstrip()
        start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
        attributes, cursor = [], len(head)
        while cursor in start_of_attribute_ending_at:
            start = start_of_attribute_ending_at[cursor]
            attributes.append(head[start:cursor])
            cursor = len(head[:start].rstrip())
        lists.append(list(reversed(attributes)))
    return lists or None


def attributes_mark_ignored(attributes):
'''),
    ("P4 自查那一处",
     '''            attributes = attributes_before_function(rust_code_without_comments(handle.read()), case.function)
        if attributes is None:
            continue
        if not attributes_mark_ignored(attributes):''',
     '''            every = attributes_before_every_definition(rust_code_without_comments(handle.read()), case.function)
        if every is None:
            continue
        if not all(attributes_mark_ignored(attributes) for attributes in every):'''),
    ("P4 闸调的那一处",
     '''                attributes = attributes_before_function(rust_code_without_comments(handle.read()), function)
        except OSError:
            return None
        if attributes is not None:
            return attributes_mark_ignored(attributes)''',
     '''                every = attributes_before_every_definition(rust_code_without_comments(handle.read()), function)
        except OSError:
            return None
        if every is not None:
            return all(attributes_mark_ignored(attributes) for attributes in every)'''),
    ("P7 拼出来的名字也管 mutations.tsv 与 src/bin",
     '''    listed_set = set(listed_names)
    left_out = {name for name in CRASH_CASE_FILES_NOT_READ''',
     '''    listed_set = set(listed_names)
    if any(name.endswith(".rs") and COMPILE_TIME_CONCATENATED_INCLUDE.search(text) for name, text in code_texts.items()):
        return set()
    left_out = {name for name in CRASH_CASE_FILES_NOT_READ'''),
    ("P9 runner 的参数里指到的文件按内容进",
     '''    if path and os.path.isfile(path):
        return read_configuration_file(path)
    return f"找不到：{program}".encode("utf-8", "surrogateescape")''',
     '''    content = read_configuration_file(path) if path and os.path.isfile(path) else f"找不到：{program}".encode("utf-8", "surrogateescape")
    for word in words[1:]:
        candidate = word if os.path.isabs(word) else os.path.join(base_directory, word)
        if os.path.isfile(candidate):
            content += b"\\n<" + word.encode("utf-8", "surrogateescape") + b">\\n" + read_configuration_file(candidate)
    return content'''),
])
print(f"副本：{target}")
