"""brace_diff2.py <树目录> <随机种子> <条数> : 随机造一个词，真 bash（set -f; set -- 词; 先数个数再逐个打）与 lib_shell_words.shell_tokens 切出的词表逐词比，分类打印不同的。
词表里空串也比（bash 里引号写出的空串留着、引号外展开出的空串去掉）。词里只有花括号、逗号、点、数字（含补零）、字母、引号空串、反斜杠转义，不含 $ 与反引号。"""
import sys, random, subprocess, os, json, collections
tree, seed, N = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
sys.path.insert(0, os.path.join(tree, ".claude", "hooks"))
import lib_shell_words as L
random.seed(seed)
atoms = ["{", "}", ",", ",", "a", "b", "..", "1", "3", "-2", "0", "01", "10", "-01", "00", "x", "{1..3}", "{a..c}", "{,}", "{a,b}", "{1..4..2}", "{c..a}", "{-1..1}", "{01..03}", "{3..01}", "{a,}", "{,b}", "{,a,}",
         "''", '""', "'q'", "\\{", "\\,", "\\}", "{'',a}", "{\"\",}", "{a,''}", "x{,}", "{,}y"]
def gen(): return "".join(random.choice(atoms) for _ in range(random.randint(1, 5)))
def bash_words(word):
    p = subprocess.run(["bash", "-c", "set -f\nset -- " + word + "\necho $#\nprintf '[%s]' \"$@\"\n"], capture_output=True, text=True, timeout=5)
    if p.returncode != 0: return None
    lines = p.stdout.split("\n", 1)
    n = int(lines[0]); body = lines[1].rstrip("\n") if len(lines) > 1 else ""
    if n == 0: return []
    return body[1:-1].split("][") if body.startswith("[") and body.endswith("]") else None
seen, diffs, n = set(), [], 0
while n < N:
    w = gen()
    if w in seen or (len(w) - len(w.rstrip("\\"))) % 2 == 1: continue
    seen.add(w); bw = bash_words(w)
    if bw is None or len(bw) > 60: continue
    n += 1
    try: hw = [t for op, t in L.shell_tokens(w) if op is False]
    except Exception as e: hw = ["EXC", repr(e)]
    hw = [x.replace("\x02", "") for x in hw]
    if hw != bw:
        kind = "只差空串" if [x for x in hw if x] == [x for x in bw if x] else "别的"
        diffs.append((kind, w, bw, hw))
print(collections.Counter(k for k, *_ in diffs), "共", n, "条（去重、bash 能跑的）")
for d in diffs[:25]: print(json.dumps(d, ensure_ascii=False)[:300])
