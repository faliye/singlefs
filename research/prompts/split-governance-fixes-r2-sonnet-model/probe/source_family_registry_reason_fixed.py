"""这一份与主 agent 复跑目录里的 probe_source_family_overreach.py 只差一处：登记表每行的理由从「样本」两个字写到四个字以上（登记表要求至少四个字，写短了整行被丢掉、登记核对走不到族检查，对照行也读成 False）。其余逐字相同。
原探针说明：项目代码闸的拆分族认法（lib_project_code_runs 的 source_family）多认了不是拆分部分的同目录文件时，要临时服务的登记核对放过一行「入口与它真拆出的部分里都没有核对」的 required 行。
样本树（临时目录里）三份 required 入口，入口与按命名拆出的部分（<模块名>_<部分>.py，lib_lag_split_family 的 Python 拆法）里都没有核对：
  sample-quotes-sibling.py   字符串里整个写着同目录另一份脚本的文件名 "sample-own-checker.py"（那一份有核对）
  sample-imports-shared.py   import 同目录的共用库 sample_shared_checks（那一份有核对，几份脚本共用，不是它拆出来的）
  sample-plain-unmarked.py   什么都不点名（对照：照报）
交回每份的 source_family 成员、lib_lag_split_family 的 family_members 成员与核对报没报。用法：<仓根> <样本目录>"""
import importlib.util, os, shutil, sys
repository, work = sys.argv[1:3]
specification = importlib.util.spec_from_file_location("lib_project_code_runs_for_family_probe", os.path.join(repository, ".claude/hooks/lib_project_code_runs.py"))
runs = importlib.util.module_from_spec(specification)
specification.loader.exec_module(runs)
sys.path.insert(0, os.path.join(repository, "scripts/gate"))
import lib_lag_split_family as split_family
if os.path.exists(work):
    shutil.rmtree(work)
gate = os.path.join(work, "scripts", "gate")
os.makedirs(gate)
marked = f"# 样本：开跑先调 {runs.TEMPORARY_SERVICE_SOURCE_MARK}\n"
files = {
    "sample-quotes-sibling.py": 'HELPER = "sample-own-checker.py"   # 样本：只在出路文字里点名同目录另一份脚本\n',
    "sample_quotes_sibling_part.py": "# 样本：它真拆出的部分，没有核对\n",
    "sample-own-checker.py": marked,
    "sample-imports-shared.py": "# 样本：用共用库里的一个小工具\nimport sample_shared_checks\n",
    "sample_shared_checks.py": marked + "def helper():\n    return 1\n",
    "sample-plain-unmarked.py": "# 样本：什么都不点名，没有核对\n",
}
for name, text in files.items():
    with open(os.path.join(gate, name), "w", encoding="utf-8") as handle:
        handle.write(text)
entries = ("sample-quotes-sibling.py", "sample-imports-shared.py", "sample-plain-unmarked.py")
registry = os.path.join(work, "registry.tsv")
table = os.path.join(work, "table.tsv")
with open(registry, "w", encoding="utf-8") as handle:
    handle.write("".join(f"scripts/gate/{name}\tselftest\t# 样本：理由写够四个字\n" for name in entries + ("sample-own-checker.py",)))
with open(table, "w", encoding="utf-8") as handle:
    handle.write("".join(f"scripts/gate/{name}\trequired\t# 样本：登记了 required\n" for name in entries) +
                 "scripts/gate/sample-own-checker.py\trequired\t# 样本：真有核对的那一份\n")
problems = runs.temporary_service_table_problems(table, registry, work)
listing = [os.path.relpath(os.path.join(gate, name), work) for name in sorted(files)]
for name in entries:
    key = f"scripts/gate/{name}"
    family = [os.path.relpath(member, work) for member in runs.source_family(os.path.join(work, key))]
    lag = split_family.family_members(key, listing)
    reported = any(problem.startswith(f"{key}：登记了") for problem in problems)
    print(f"PROBE entry={name} source_family={family} lag_family={lag} reported_unmarked={reported}")
print(f"PROBE problems={len(problems)}")
