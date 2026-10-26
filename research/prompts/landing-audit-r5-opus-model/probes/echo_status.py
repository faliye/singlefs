"""K2 ⑤ 改法的预估（推的：只在这份探针里分类，没在钩子上实现）：改后不收的真命名检查里，有几条后面紧跟「; echo …$?」、工具结果里读得到它打出的退出码 0 或 1。
用法：python3 -I echo_status.py <语料.jsonl> <改后钩子目录> <改前钩子目录> <agent 号清单>"""
import collections, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
corpus, cur_hooks, orig_hooks, id_list = sys.argv[1:5]
wanted = set(open(id_list, encoding="utf-8").read().split())
cur = hookload.load(os.path.join(cur_hooks, "handback-guard.sh"), cur_hooks, "cur_handback")
orig = hookload.load(os.path.join(orig_hooks, "handback-guard.sh"), orig_hooks, "orig_handback")
saved = list(sys.argv); sys.argv = [saved[0], cur_hooks]; heavy = cur.heavy_tests_library(); orig_heavy = orig.heavy_tests_library(); sys.argv = saved
NAMING = re.compile(r"(?:code-source-discipline\.sh --check \S+|naming-lint\.sh)[^;&|\n]*(?:[0-9]?>&?\s*\S+\s*)*;\s*echo\s+(\"?)([^\"$;\n]*)\$\?\1")
counts = collections.Counter()
results = {}
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] not in wanted:
        continue
    for record in item["records"]:
        for block in record["message"]["content"]:
            if block.get("type") == "tool_result":
                results[block["tool_use_id"]] = block.get("content") or ""
    for record in item["records"]:
        for block in record["message"]["content"]:
            if block.get("type") != "tool_use" or block.get("name") != "Bash":
                continue
            command = str(block["input"].get("command") or "")
            if "naming-lint.sh" not in command and "code-source-discipline.sh" not in command:
                continue
            before = orig.naming_check_runs(command, record.get("cwd"), orig_heavy)
            if not before:
                continue
            output = results.get(block["id"], "")
            exit_code = 0 if not output.lstrip().startswith("Exit code") else None
            completed = cur.completed_naming_text(command, exit_code, heavy) if exit_code == 0 else ""
            after = cur.naming_check_runs(completed, record.get("cwd"), heavy) if completed else set()
            if after == before:
                continue
            echo = NAMING.search(command)
            if not echo:
                counts["改后少认、后面没有紧跟 ; echo …$?"] += 1
                continue
            printed = re.search(re.escape(echo.group(2).strip()) + r"(\d+)", output) if echo.group(2).strip() else None
            if printed and printed.group(1) in ("0", "1"):
                counts[f"改后少认、后面紧跟 ; echo {echo.group(2).strip()}$?、工具结果开头读得到 0 或 1"] += 1
            else:
                counts["改后少认、紧跟 ; echo …$? 而工具结果开头 300 字里读不到它打的数"] += 1
for key in sorted(counts):
    print(f"{key}：{counts[key]} 条 Bash 调用")
