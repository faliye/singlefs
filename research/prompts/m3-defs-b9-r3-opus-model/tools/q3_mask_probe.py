#!/usr/bin/env python3
"""Q3（第二轮 S6，判「站得住」）：doc-text 的打码共用函数（PRIVATE_VALUES_LIBRARY：load_private_targets、shown_path、shown_text、snippet_bounds、shown_line）
在第二轮没喂过的形状上会不会把（编的）私有值整串或半截打出来。从 <门禁根>/.claude/gate.d/doc-text.sh 抽出那一段原样 exec，配置与主机名都是编的，写进 <工作目录>。
三条通道照 doc-text 的调法：sweep-term 输出经 shown_line、date-once 红行经 shown_path、三格片段经 snippet_bounds 加 shown_text（片段取法照 scan_form：命中处前 20 字到后 10 字；命中处用占位〈命中〉代替钟点，免得这份文件自己被 clock-times 格点名）。
用法：q3_mask_probe.py <门禁根> <工作目录>"""
import os, re, sys, types
gate_root, work = sys.argv[1:3]
os.makedirs(work, exist_ok=True)
text = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read()
source = text.split("PRIVATE_VALUES_LIBRARY=\"$(cat <<'PY_PRIVATE_VALUES'\n", 1)[1].split("\nPY_PRIVATE_VALUES\n", 1)[0]
library = types.ModuleType("lib_private_values")
exec(compile(source, "private-values-library", "exec"), library.__dict__)
configuration = os.path.join(work, "multi-host.env")
hostname = os.path.join(work, "hostname")
with open(configuration, "w", encoding="utf-8") as handle:
    handle.write("PEER_SSH_HOST=demo-peer-host-42\n"
                 "PEER_REPOSITORY_DIRECTORY=\"/srv/demo private/peer-checkout\"\n"
                 "QUIESCE_STOP_COMMAND='ssh demo-peer-host-42 systemctl --user stop demo-gpu-service'\n"
                 "export PEER_CARGO_BIN_DIRECTORY=/opt/demo-toolchain-77/bin\n")
with open(hostname, "w", encoding="utf-8") as handle:
    handle.write("demo-local-box.lan\n")
targets = library.load_private_targets(configuration, hostname, "")[0]
words = sorted({word for _key, word in targets}, key=len)
print("取到的词\t" + "、".join(words))

def leaks(shown):
    whole = [word for word in words if word in shown]
    halves = sorted({word[i:i + 6] for word in words for i in range(len(word) - 5) if word[i:i + 6] in shown and "〔" not in word[i:i + 6]})
    return f"整值 {whole}；半截（≥6 字）{halves[:4]}"

def snippet(line, needle):
    position = line.index(needle)
    start, end = max(0, position - 20), position + len(needle) + 10
    start, end = library.snippet_bounds(line, start, end, targets)
    return library.shown_text(line[start:end].strip(), targets)

cases = [
    ("sweep-term 输出", "路径里值带空格（引号括起的仓路径）", "research/results/srv/demo private/peer-checkout/notes.md       1 处"),
    ("sweep-term 输出", "行内锚里带斜杠与值", "records/x.md:「见 /opt/demo-toolchain-77/bin/cargo 那一句」命中 0 行"),
    ("sweep-term 输出", "全角冒号粘着值", "records/x.md：demo-peer-host-42：1 处"),
    ("date-once 红行", "值出现两次、中间隔一段", "records/demo-peer-host-42/2026-10-14-demo-peer-host-42-对拍.md"),
    ("date-once 红行", "本机主机名短名在文件名里", "records/2026-10-14-demo-local-box-对拍.md"),
    ("片段", "两个值紧挨（不重叠），命中在后一个值后面", "demo-gpu-servicedemo-peer-host-42 上 〈命中〉 收尾"),
    ("片段", "值前后都切在中间（片段比值短）", "x" * 30 + "demo-peer-host-42" + "上 〈命中〉" + "y" * 30),
    ("片段", "值跨两段带空格（引号括起的仓路径）", "在 /srv/demo private/peer-checkout 里 〈命中〉 起"),
]
for channel, name, line in cases:
    if channel == "sweep-term 输出":
        shown = library.shown_line(line, targets, "")
    elif channel == "date-once 红行":
        shown = library.shown_path(line, targets, "")
    else:
        shown = snippet(line, "〈命中〉")
    print(f"{channel}\t{name}\t{shown}\t{leaks(shown)}")
