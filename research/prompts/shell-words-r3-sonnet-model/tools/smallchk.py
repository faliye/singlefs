"""smallchk.py <树> : 自己写的小检查：内建名表 vs 本机 bash 的 compgen -b；RedirectOperator 的 copy / deepcopy / pickle；bash 与 env 版本。"""
import sys, os, subprocess, copy, pickle
tree = sys.argv[1]
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "trees", tree, ".claude", "hooks"))
import lib_shell_words as L
real = set(subprocess.run(["bash", "-c", "compgen -b"], capture_output=True, text=True).stdout.split())
print("内建表", len(L.BASH_BUILTINS), "bash compgen -b", len(real), "表里多", sorted(L.BASH_BUILTINS - real), "bash 里多", sorted(real - L.BASH_BUILTINS))
op = [t for is_op, t in L.shell_tokens("cat a 2> b") if is_op][0]
print("type", type(op).__name__, "fd" in getattr(op, "_fields", ()) or hasattr(op, "fd"))
for name, fn in (("copy.copy", copy.copy), ("copy.deepcopy", copy.deepcopy), ("pickle", lambda o: pickle.loads(pickle.dumps(o)))):
    try: fn(op); r = "ok"
    except Exception as e: r = f"{type(e).__name__}: {e}"
    print(name, r)
print(subprocess.run(["bash", "--version"], capture_output=True, text=True).stdout.splitlines()[0], "|", subprocess.run(["env", "--version"], capture_output=True, text=True).stdout.splitlines()[0])
