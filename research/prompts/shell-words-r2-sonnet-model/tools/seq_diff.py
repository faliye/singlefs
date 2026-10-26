"""seq_diff.py <树目录> <种子> <条数> : 随机整数序列 {起..止[..步长]}（含补零、负号、-0），真 bash 展开 vs lib_shell_words.shell_tokens 与 lib_heavy_tests.brace_sequence 各展开，逐词比。"""
import sys, os, random, subprocess, collections
tree, seed, N = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
sys.path.insert(0, os.path.join(tree, ".claude", "hooks"))
import lib_shell_words as L, lib_heavy_tests as H
random.seed(seed)
def num():
    v = random.choice(["0", "00", "000", "1", "01", "001", "3", "03", "10", "010", "-1", "-01", "-001", "-0", "-00", "-000", "7", "-7", "12", "100", "0100"])
    return v
diff = collections.Counter(); shown = 0; n = 0; seen = set()
tries = 0
while n < N and tries < 20 * N:
    tries += 1
    a, b = num(), num()
    inner = f"{a}..{b}" + (f"..{random.choice(['1','2','3','-1','-2','02'])}" if random.random() < 0.3 else "")
    w = "x{" + inner + "}y"
    if w in seen: continue
    seen.add(w)
    p = subprocess.run(["bash", "-c", "set -f\nset -- " + w + "\necho $#\nprintf '[%s]' \"$@\"\n"], capture_output=True, text=True)
    if p.returncode: continue
    lines = p.stdout.split("\n", 1); cnt = int(lines[0]); body = lines[1].rstrip("\n")
    bw = [] if cnt == 0 else body[1:-1].split("][")
    if len(bw) > 60: continue
    n += 1
    sw = [t for op, t in L.shell_tokens(w) if op is False]
    hv = H.brace_sequence(inner)
    hw = None if hv is None else ["x" + t + "y" for t in hv]
    if sw != bw:
        diff["shell_words"] += 1
        if shown < 6: shown += 1; print("shell_words 不同", w, bw[:4], sw[:4])
    if hw != bw:
        diff["heavy_tests"] += 1
        if shown < 12: shown += 1; print("heavy_tests 不同", w, bw[:4], (hw or [])[:4])
print(tree.split("/")[-1], "共", n, "条，不同：", dict(diff))
