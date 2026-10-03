#!/usr/bin/env python3
"""E1 草稿改法 G9（只打在草稿拷贝里，被攻过零轮）：verus_judge_use 改成「认出是这两份判法脚本就算重型，只有字面写明的只读用法放过」，
认法照广谱变异那一份加 PYTHONPATH、真实路径（符号链接）与 python -c 代码里点名。
用法：python3 fix-draft-e1.py <拷贝根>（就地改拷贝里的 .claude/hooks/lib_heavy_tests.py；每处旧串要恰好命中一次）。"""
import sys, os

path = os.path.join(sys.argv[1], ".claude", "hooks", "lib_heavy_tests.py")
text = open(path, encoding="utf-8").read()
edits = [
    ('''def verus_judge_use(name, word, arguments, directory):''',
     '''VERUS_JUDGE_READ_ONLY_SUBCOMMANDS = frozenset({"toolchain", "fingerprint", "tool-versions"})


def verus_judge_script(word, directory, environment=None):
    """草稿 G9：命令词是不是两份判法脚本之一（名字与位置，PYTHONPATH 各试一遍；解到的真实路径落在 research/scripts）。"""
    expanded = os.path.expanduser(word)
    bases = [directory]
    for entry in ((environment or {}).get("PYTHONPATH") or "").split(os.pathsep):
        if entry:
            bases.append(shell_words.resolve_path(directory, os.path.expanduser(entry)))
    for script in (VERUS_PROOF_CHECK_SCRIPT, E165_PIN_SCRIPT):
        if os.path.basename(expanded) == script and any(located_in(expanded, base, VERUS_JUDGE_DIRECTORY) for base in bases):
            return script
        for base in bases:
            resolved = shell_words.resolve_path(base, expanded)
            if resolved is not None and os.path.isfile(resolved) and os.path.realpath(resolved).endswith(f"/{VERUS_JUDGE_DIRECTORY}/{script}"):
                return script
    return None


def verus_judge_use(name, word, arguments, directory, environment=None):'''),
    ('''    if break_is_set("verus-judges-by-stage-only") or not located_in(word, directory, VERUS_JUDGE_DIRECTORY):
        return None
    if name == VERUS_PROOF_CHECK_SCRIPT and arguments[:1] and arguments[0] in VERUS_PROOF_CHECK_WORKING_SUBCOMMANDS:''',
     '''    if break_is_set("verus-judges-by-stage-only"):
        return None
    script = verus_judge_script(word, directory, environment)
    if script is None:
        return None
    literal = not any("$" in argument or "`" in argument for argument in arguments)
    name = script
    if name == VERUS_PROOF_CHECK_SCRIPT and not (literal and arguments[:1] and arguments[0] in VERUS_JUDGE_READ_ONLY_SUBCOMMANDS):
        return heavy_test("verus-stage", f"research/scripts/{name} {' '.join(arguments[:1])}（门禁 checker-tier-formal-verification 那一格的判法；只有字面写明的只读子命令不算）")
    if name == E165_PIN_SCRIPT and (not literal or e165_pin_mode(arguments) == "replay"):'''),
    ('''    verus_judge = verus_judge_use(name, words[0], arguments, directory)''',
     '''    verus_judge = verus_judge_use(name, words[0], arguments, directory, environment)
    if verus_judge is None and shell_words.PYTHON_NAME.match(name) and not break_is_set("verus-judges-by-stage-only"):
        invocation = shell_words.python_invocation(arguments)
        if invocation.code_string is not None and (VERUS_PROOF_CHECK_SCRIPT in invocation.code_string or E165_PIN_SCRIPT in invocation.code_string):
            verus_judge = heavy_test("verus-stage", "python -c 的代码里点了 Verus 那一道的判法脚本")'''),
]
for old, new in edits:
    count = text.count(old)
    if count != 1:
        sys.exit(f"旧串命中 {count} 次：{old[:60]}")
    text = text.replace(old, new)
temporary = path + ".draft"
open(temporary, "w", encoding="utf-8").write(text)
os.replace(temporary, path)
print("打上 G9")
