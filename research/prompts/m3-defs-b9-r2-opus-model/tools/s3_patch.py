#!/usr/bin/env python3
"""F-S3a 试改（只在本腿模型上量，被攻过零轮）：registry_sourced_variables 只认整条右边是登记位读法的赋值
（"$(heavy_thread_default)"、"$(heavy_thread_default 2>/dev/null)"、"$(default_heavy_threads)"、"${SINGLEFS_THREAD_CAP}"，引号可有可无），
且那个变量在文件里没有出现在 ${变量:-…}、${变量:=…}、${变量-…}、${变量=…} 里（有就说明它可能先由别处给了值或补了默认）。
用法：s3_patch.py <原 code-tooling.sh> <输出>"""
import sys
source, target = sys.argv[1:3]
text = open(source, encoding="utf-8").read()
old_start = text.index("def registry_sourced_variables(shell_code):")
old_end = text.index("THREAD_DEFAULT_FORMS = [", old_start)
new = '''REGISTRY_WHOLE_RIGHT_SIDE = re.compile(r'^"?(?:\\$\\((?:heavy_thread_default|default_heavy_threads)(?:[ \\t]+2>/dev/null)?\\)|\\$\\{?SINGLEFS_THREAD_CAP\\}?)"?$')
VARIABLE_WITH_DEFAULT = re.compile(r"\\$\\{([A-Za-z_][A-Za-z0-9_]*):?[-=]")


def registry_sourced_variables(shell_code):
    """（F-S3a 试改）每一处赋值的右边整条都是登记位读法、且没在 ${变量:-…} / ${变量:=…} 里出现过的变量。"""
    verdicts = {}
    for found in SHELL_ANY_ASSIGNMENT.finditer(shell_code):
        from_registry = REGISTRY_WHOLE_RIGHT_SIDE.match(found.group("rhs")) is not None
        verdicts[found.group("name")] = verdicts.get(found.group("name"), True) and from_registry
    defaulted = set(VARIABLE_WITH_DEFAULT.findall(shell_code))
    return {name for name, from_registry in verdicts.items() if from_registry and name not in defaulted}


'''
open(target, "w", encoding="utf-8").write(text[:old_start] + new + text[old_end:])
