"""small_checks.py <树目录> : 几件小事，各打一行 name=…（确定的结果，进 totals）：
1 BASH_BUILTINS 与本机 bash 的 compgen -b 逐个比；2 RedirectOperator 能不能 copy / deepcopy / pickle（钩子里没人这样用，只是隐患）；3 bash 与 env 的版本。"""
import sys, os, subprocess, copy, pickle
tree = sys.argv[1]
sys.path.insert(0, os.path.join(tree, ".claude", "hooks"))
import lib_shell_words as L
real = set(subprocess.run(["bash", "-c", "compgen -b"], capture_output=True, text=True).stdout.split())
print(f"name=builtin_table table={len(L.BASH_BUILTINS)} compgen_b={len(real)} only_in_table={sorted(L.BASH_BUILTINS - real)} only_in_bash={sorted(real - L.BASH_BUILTINS)}")
operator = [t for is_operator, t in L.shell_tokens("cat a 2> b") if is_operator][0]
for name, fn in (("copy.copy", copy.copy), ("copy.deepcopy", copy.deepcopy), ("pickle", lambda o: pickle.loads(pickle.dumps(o)))):
    try:
        fn(operator); result = "ok"
    except Exception as error:
        result = f"{type(error).__name__}: {error}"
    print(f"name=redirect_operator_{name.replace('.', '_')} {result}")
print("name=versions", subprocess.run(["bash", "--version"], capture_output=True, text=True).stdout.splitlines()[0], "|", subprocess.run(["env", "--version"], capture_output=True, text=True).stdout.splitlines()[0])
