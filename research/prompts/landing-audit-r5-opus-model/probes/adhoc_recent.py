import json, os, re, sys, importlib.util
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, sys.argv[5])
import hookload
corpus, cur_hooks, orig_hooks, agents_dir = sys.argv[1:5]
oh = hookload.load(os.path.join(orig_hooks, "handback-guard.sh"), orig_hooks, "oh")
ch = hookload.load(os.path.join(cur_hooks, "handback-guard.sh"), cur_hooks, "ch")
spec = importlib.util.spec_from_file_location("dp", os.path.join(agents_dir, "dispatch-prompt.py")); dp = importlib.util.module_from_spec(spec); spec.loader.exec_module(dp)
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if (item["first_time"] or "") < "2026-10-22" or "草稿目录" not in item["prompt"]:
        continue
    p = item["prompt"]
    old6 = oh.prompt_label_paths(p, "草稿目录"); new6 = ch.prompt_scratch_directories(p)
    draft = dp.DRAFT_LINE.search(p)
    if old6 and [os.path.normpath(os.path.expanduser(v)) for v in old6] != new6:
        print("⑥", item["agent"], item["type"], "改前", old6, "改后", new6)
        print("   ", [l[:200] for l in p.split("\n") if "草稿目录" in l][:3])
    if draft and os.path.normpath(os.path.expanduser(draft.group(1).strip("`'\""))) not in new6:
        print("骨架", item["agent"], "DRAFT_LINE", draft.group(1), "新库", new6)
