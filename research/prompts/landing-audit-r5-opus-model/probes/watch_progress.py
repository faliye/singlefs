"""K3 ㉚：看门狗 agent-watch.py 的 progress_file_of 按派发提示里点名的全部工作根路径找 progress.md、取改得最晚的；
拿真派发提示照它的写法算一遍（文件系统取此刻的样子），数取到的 progress.md 不在新库认的草稿目录下的份数。
用法：python3 -I watch_progress.py <语料.jsonl> <改后钩子目录> <scripts/agents 目录> <agent 号清单> <只看这一天及之后派的（YYYY-MM-DD）>
progress_file_of 的写法逐字取自 agent-watch.py（取不到就停）。只读。"""
import collections, importlib.util, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
corpus, cur_hooks, agents_dir, id_list, since = sys.argv[1:6]
wanted = set(open(id_list, encoding="utf-8").read().split())
def library(path, name):
    spec = importlib.util.spec_from_file_location(name, path); module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
roots = library(os.path.join(cur_hooks, "lib_work_root.py"), "roots_probe")
lib = library(os.path.join(cur_hooks, "lib_prompt_scratch_line.py"), "scratch_probe")
watch_text = open(os.path.join(agents_dir, "agent-watch.py"), encoding="utf-8").read()
body = re.search(r"^def progress_file_of\(transcript\):\n(?:    .*\n|\n)+", watch_text, re.M).group(0)
assert "SCRATCH_PATH_MENTION.findall(transcript.dispatch_prompt" in body and "max(existing, key=os.path.getmtime)" in body
MENTION = re.compile(roots.mention_prefix_pattern() + r"[^\s`'\"，。；：、（）()「」<>]+")
namespace = {"SCRATCH_PATH_MENTION": MENTION, "work_roots": roots, "PROGRESS_FILE_NAME": "progress.md", "os": os}
exec(body, namespace)
progress_file_of = namespace["progress_file_of"]
class Transcript:
    def __init__(self, prompt): self.dispatch_prompt = prompt
counts, examples = collections.Counter(), collections.defaultdict(list)
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] not in wanted or (item["first_time"] or "")[:10] < since:
        continue
    own = lib.scratch_directories(item["prompt"])
    chosen = progress_file_of(Transcript(item["prompt"]))
    if not own:
        key = "没有行首草稿目录一行"
    elif chosen is None:
        key = "此刻一份 progress.md 都找不到"
    elif any(chosen.startswith(d + "/") for d in own):
        key = "取到的在本提示草稿目录下"
    else:
        key = "取到的不在本提示草稿目录下（看门狗告警附的是别人的 progress.md）"
    counts[key] += 1
    if len(examples[key]) < 6:
        examples[key].append(f"{item['agent']}（{item['type']}）→ {chosen}")
for key in sorted(counts):
    print(f"{key}：{counts[key]} 份；例：{'；'.join(examples[key])}")
