"""K3 看门狗 progress_file_of 改前（v10）改后（now）在全部真派发提示上取到谁的 progress.md。
「本提示的草稿目录」不调 lib_prompt_scratch_line，这里另写一份（行首「草稿目录」接冒号、路径到空白反引号括号为止、~ 与 $HOME 展开），
与看门狗用的库不共用代码；取到的 progress.md 所在目录是不是它，按 realpath 比。只读，取此刻的文件系统。
用法：python3 -I watch_progress_all.py <语料.jsonl> <改前 agent-watch.py> <改后 agent-watch.py> <分界日期 YYYY-MM-DD>"""
import collections, importlib.util, json, os, re, sys, types
sys.dont_write_bytecode = True
corpus, before_path, after_path, since = sys.argv[1:5]

def load(path, name):
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(path)))), ".claude", "hooks"))
    for cached in ("lib_prompt_scratch_line", "lib_work_root"):
        sys.modules.pop(cached, None)
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    sys.path.pop(0)
    return module

before, after = load(before_path, "watch_before"), load(after_path, "watch_after")
OWN = re.compile(r"^[ \t]*草稿目录[ \t]*[：:][ \t]*`?([^\s`'\"，。；：、（）()「」<>*]+)", re.M)

def own_directories(prompt):
    found = []
    for raw in OWN.findall(prompt):
        path = raw.rstrip(".,;:")
        for home in ("~/", "$HOME/", "${HOME}/"):
            if path.startswith(home):
                path = os.path.join(os.path.expanduser("~"), path[len(home):])
        found.append(os.path.realpath(path.rstrip("/")))
    return found

def classify(picked, owns):
    if picked is None:
        return "取不到"
    return "取到本提示草稿目录下的" if os.path.realpath(os.path.dirname(picked)) in owns else "取到别的目录下的"

counts, examples = collections.Counter(), collections.defaultdict(list)
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    prompt = item["prompt"] or ""
    owns = own_directories(prompt)
    transcript = types.SimpleNamespace(dispatch_prompt=prompt)
    verdict_before = classify(before.progress_file_of(transcript), owns)
    verdict_after = classify(after.progress_file_of(transcript), owns)
    has_own_progress = any(os.path.isfile(os.path.join(directory, "progress.md")) for directory in owns)
    period = f"{since} 起" if (item["first_time"] or "")[:10] >= since else f"{since} 之前"
    key = (period, "有行首草稿目录一行" if owns else "没有那一行", verdict_before, verdict_after,
           "那个目录下有 progress.md" if has_own_progress else "")
    counts[key] += 1
    if len(examples[key]) < 2:
        examples[key].append(item["agent"])
for key in sorted(counts):
    period, line_kind, verdict_before, verdict_after, extra = key
    print(f"{period}｜{line_kind}｜改前 {verdict_before}｜改后 {verdict_after}{('｜' + extra) if extra else ''}：{counts[key]} 份；例：{'、'.join(examples[key])}")
for period in (f"{since} 起", f"{since} 之前"):
    rows = {key: count for key, count in counts.items() if key[0] == period}
    print(f"{period} 合计 {sum(rows.values())} 份：改前取到别的目录下的 {sum(c for k, c in rows.items() if k[2] == '取到别的目录下的')}，"
          f"改后 {sum(c for k, c in rows.items() if k[3] == '取到别的目录下的')}；改前取到本提示的 {sum(c for k, c in rows.items() if k[2] == '取到本提示草稿目录下的')}，"
          f"改后 {sum(c for k, c in rows.items() if k[3] == '取到本提示草稿目录下的')}；改后取不到而本提示目录下有 progress.md 的 {sum(c for k, c in rows.items() if k[3] == '取不到' and k[4])}")
