"""K2 新库认代码块的逐行配法与派发闸 ⑭⑮⑰ 用的 CommonMark 认法（without_code_blocks，markdown-it-py）对不对得上：
造几份提示各喂两种认法；再拿真派发提示数两种认法认出的「草稿目录：」值不同的份数。
用法：python3 -I fence_rules.py <改后钩子目录> <语料.jsonl> <agent 号清单>。只读。"""
import importlib.util, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
hooks, corpus, id_list = sys.argv[1:4]
wanted = set(open(id_list, encoding="utf-8").read().split())
guard = hookload.load(os.path.join(hooks, "runner-dispatch-guard.sh"), hooks, "cur_dispatch")
spec = importlib.util.spec_from_file_location("scratch_probe", os.path.join(hooks, "lib_prompt_scratch_line.py"))
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
root = os.path.expanduser("~/.local/share/singlefs/work")
mine, old = f"{root}/r5-fence-mine", f"{root}/r5-fence-previous-leg"

def by_commonmark(prompt):
    """把派发闸 without_code_blocks 遮过的提示交新库的同一个正则（闭合围栏已经遮掉，新库自己的围栏判法不再用）。"""
    masked = guard.without_code_blocks(prompt)
    values = []
    for line in masked.split("\n"):
        for match in lib.SCRATCH_LINE.finditer(line):
            value = (match.group("path") or "").rstrip(lib.TRAILING_PUNCTUATION)
            if value and value not in values:
                values.append(value)
    return values

cases = [
    ("列表项里的围栏块（前一条腿的提示原样贴在列表里）", f"草稿目录：{mine}\n- 上一条腿的提示原样：\n  ```\n  草稿目录：{old}\n  ```\n"),
    ("缩进 4 格的代码块（空行之后）", f"草稿目录：{mine}\n\n    草稿目录：{old}\n"),
    ("语言名里带反引号的那一行（CommonMark 不当围栏开头）", f"```a`b\n草稿目录：{mine}\n```\n"),
    ("引用块里的围栏块", f"草稿目录：{mine}\n> ```\n> 草稿目录：{old}\n> ```\n"),
    ("对照：顶格的闭合围栏块", f"草稿目录：{mine}\n```\n草稿目录：{old}\n```\n"),
]
for label, prompt in cases:
    print(f"{label}：新库认 {lib.scratch_line_values(prompt)}；照 CommonMark 遮过之后认 {by_commonmark(prompt)}；⑯ 建锁 {guard.scratch_directories_named(prompt)}")
differ = []
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] in wanted and "草稿目录" in item["prompt"] and lib.scratch_line_values(item["prompt"]) != by_commonmark(item["prompt"]):
        differ.append(item["agent"])
print(f"真派发提示里两种认法认出的「草稿目录：」值不同的：{len(differ)} 份{('；' + '、'.join(differ[:8])) if differ else ''}")
