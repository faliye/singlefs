"""bracecmp.py <树> <种子> <条数> : 自己造词，真 bash（进程组里跑 set -f; set -- 词）展开的词表 vs 树里 lib_shell_words.shell_tokens 切出的词、lib_heavy_tests.brace_sequence 的序列，逐词比。
两类词：整数序列 x{a..b[..s]}y（补零、负号、步长），与花括号杂拌。交回不同的条数与前几条。"""
import sys, os, random, subprocess, collections, signal
tree, seed, N = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "trees", tree, ".claude", "hooks"))
import lib_shell_words as L, lib_heavy_tests as H
random.seed(seed)
def bash_words(w):
    p = subprocess.Popen(["bash", "-c", "set -f\nset -- " + w + "\necho $#\nprintf '[%s]' \"$@\"\n"], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, start_new_session=True)
    try: out, _ = p.communicate(timeout=10)
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL); p.wait(); return None
    finally:
        try: os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError: pass
    if p.returncode: return None
    n, _, body = out.partition("\n"); body = body.rstrip("\n")
    return [] if int(n) == 0 else body[1:-1].split("][")
def numtok(): return random.choice(["0", "00", "000", "5", "05", "005", "9", "09", "11", "011", "-5", "-05", "-005", "-0", "-00", "21", "100", "0100", "007", "99", "-9", "1000"])
atoms = ["{", "}", ",", "a", "c", "..", "2", "4", "-3", "0", "02", "12", "-02", "00", "z", "{2..5}", "{a..e}", "{,}", "{x,y}", "{1..6..2}", "{e..a}", "{-2..2}", "{02..05}", "{5..02}", "{x,}", "{,y}", "{,x,}", "''", '""', "'k'", "\\{", "\\,", "\\}", "{'',x}", "{\"\",}", "{x,''}", "z{,}", "{,}w", "{a,{b,c}}", "{{a,b},c}", "{a,b}{c,d}"]
seen = set(); diff = collections.Counter(); ex = []; n = 0; tries = 0
mode = "seq" if seed % 2 else "mix"
while n < N and tries < 30 * N:
    tries += 1
    if mode == "seq":
        a, b = numtok(), numtok(); inner = f"{a}..{b}" + (f"..{random.choice(['1','2','3','-1','-2','02','5'])}" if random.random() < 0.4 else ""); w = "q{" + inner + "}r"
    else:
        w = "".join(random.choice(atoms) for _ in range(random.randint(1, 5)))
        if (len(w) - len(w.rstrip("\\"))) % 2: continue
    if w in seen: continue
    seen.add(w); bw = bash_words(w)
    if bw is None or len(bw) > 60: continue
    n += 1
    try: sw = [t.replace("\x02", "") for op, t in L.shell_tokens(w) if op is False]
    except Exception as e: sw = ["EXC", repr(e)]
    if sw != bw:
        diff["shell_tokens"] += 1
        if len(ex) < 6: ex.append(("shell_tokens", w, bw[:4], sw[:4]))
    if mode == "seq":
        hv = H.brace_sequence(inner); hw = None if hv is None else ["q" + t + "r" for t in hv]
        if hw != bw:
            diff["brace_sequence"] += 1
            if len(ex) < 6: ex.append(("brace_sequence", w, bw[:4], (hw or [])[:4]))
print(tree, mode, "种子", seed, "共", n, "条（去重、bash 能跑的），不同：", dict(diff) if diff else "无")
for e in ex: print("   ", e)
