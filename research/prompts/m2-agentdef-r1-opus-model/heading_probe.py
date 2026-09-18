# 草稿：把规则、SOP、kb、records 里每个标题当成「开工先读：`文件`「标题」」一行，用 71 号的 ③④ 正则判，列出会红的。
import glob, re, sys, os
os.chdir(sys.argv[1])
HALF_SENTENCE = re.compile(
    r"(?:[（(，,；;。]|——)\s*(?:20\d\d-\d\d-\d\d\s*)?"
    r"(实测|试跑|踩过|撞上|撞过|因为|之所以|(?:为什么|依据|理由|原因|经过)(?=[：:\s]))"
)
DATE = re.compile(r"20\d\d-\d\d-\d\d")
HEADING = re.compile(r"^\s{0,3}#{1,6}\s+(.*)$")
files = sorted(set(glob.glob(".claude/rules/**/*.md", recursive=True) + glob.glob(".claude/singlefs-ai-sop/rules/**/*.md", recursive=True)
             + glob.glob(".claude/singlefs-ai-sop/skills/**/*.md", recursive=True) + glob.glob(".claude/kb/**/*.md", recursive=True)))
n_head = 0
for path in files:
    fenced = False
    for i, line in enumerate(open(path, encoding="utf-8"), 1):
        if re.match(r"^\s*(?:```|~~~)", line):
            fenced = not fenced; continue
        if fenced: continue
        h = HEADING.match(line)
        if not h: continue
        n_head += 1
        title = h.group(1).strip()
        hits = [m.group(1) for m in HALF_SENTENCE.finditer(title)]
        if hits:
            print(f"④\t{path}:{i}\t{hits}\t{title}")
print("headings scanned:", n_head, "files:", len(files))
