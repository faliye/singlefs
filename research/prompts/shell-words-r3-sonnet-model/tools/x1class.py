"""x1class.py : X1 各样本集在 now 上判错的条，按写法归类计数（规则写在下面，规则之外的归「其它」并列出来）。不在 rerun.sh 里，单独跑：python3 -I tools/x1class.py（工作目录 = 草稿根）。"""
import json, re, collections
RULES = {
 "x1-W1": [("sh -c（本机 sh 是 dash，不展开花括号）", r"\bsh -c"), ("引号里的花括号（bash 不展开）", r"\"--\{all,x\}\"|'--\{all,x\}'"), ("反斜杠转义的花括号（bash 不展开）", r"\\\{")],
 "x1-W2": [("喂 shell 的另四种写法 source /dev/stdin、. /dev/stdin、bash /dev/stdin、bash -c 'cat | bash'", r"source /dev/stdin|\. /dev/stdin|bash /dev/stdin|cat \| bash"), ("命令替换里的 case 模式 `a)`", r"case a in a\)"), ("命令替换里的注释 `# …)`", r"\$\( # c\)")],
 "x1-W3": [("tee 拷出（已知类）", r"tee d\.sh"), ("`>>` 拷出（已知类）", r">> d\.sh"), ("ln -s 源写成 $PWD/…", r"\$PWD/"), ("命令词放变量里的解释器 X=bash; $X", r"X=.?bash.?;"), ("eval 命令替换（文件头判不到的）", r"eval \"\$\(echo"), ("script -qc 启动器（名单外）", r"script -qc"), ("sh 起拷出的脚本（dash 读不了 set -o pipefail）", r"sh d\.sh"), ("子目录 cwd 的样本造法问题（sub/sub）", r"sub/d\.sh|cd sub")],
 "x1-W4": [("env -i（本机 PATH 清空后找不到 cargo）", r"env -i"), ("env -0 加命令（env 拒绝）", r"env -0"), ("env -u '' （env 拒绝）", r"env -u ''"), ("env 不认的选项 -x、--foo（env 拒绝）", r"env (-x|--foo)")],
 "x1-W5": [("sh 的选项（dash）", r"\bsh -"), ("coproc 关键字", r"coproc"), ("bash --version / --help", r"bash --(version|help)"), ("command -p（沙盒 PATH）", r"command -p"), ("cd / 之后的双斜杠样本（沙盒日志找不到）", r"cd /; cp")],
 "x1-V2-2": [("命令词不是调用函数（command / builtin / exec / env / nice / bash -c / xargs / 通配）", r"; (command|builtin|exec|env|nice|bash -c|xargs) |; (fo\?|f\*|\[f\]oo|fo\[o\])$"), ("名字出现在数据里（echo / : / printf / cat 的参数）", r"; (echo|:|printf|cat) "), ("POSIX 形 foo() 只定义", r"^foo\(\) \{ cargo test --all; \}$")],
 "x1-V2-3": [("`>>`、`&>`、`&>>`、`1>>`（已知类）", r"&>>?|>>"), ("`1<>`", r"1<>"), ("`||` 之后（左边成功右边不跑）", r"\|\| "), ("exec 3> / >&3 / 组重定向 / 管道 tee / < 读源", r"exec 3>|^\{ cat|^\( cat|tee d\.sh|cat < s\.sh|cat - < s\.sh"), ("源是数字、引号数字、a1（cat 找不到这个源）", r"(cat s\.sh|cat scripts/hosts/detached-run\.sh) (1|2|\"1\"|'1'|\\\\1|a1|x=1) ?>"), ("x=1> （赋值样子的词）", r"x=1>"), ("{v}> （fd 变量）", r"\{v\}>"), ("2>&1 / 2>文件 在 > 之前、之后 `||`", r"2>&1 (1)?> d\.sh \|\||2> e\.txt > d\.sh \|\||2>/dev/null > d\.sh \|\|")],
 "x1-V2-6": [("多目标 cp 失败（s01.sh 本来就在探针目录里）", r"cp s\.sh s\{"), ("heredoc 样本被 # 吃掉（造法问题）", r"#>e\.sh")],
 "x1-V3-7": [("case 的分支没被选中（静态判不了）", r"case (cargo|x) in .*\) (cargo test --all|bash scripts/hosts/detached-run.sh)"), ("if [[ -n cargo && x == cargo ]] 条件为假", r"x == cargo"), ("[[ $(… | herd7 / killall) ]]（裁判只认 cargo 与 detached-run，认不出这两个真跑了）", r"\$\(echo .*(herd7|killall)")],
}
for fam, rules in RULES.items():
    c = json.load(open(f"cases/cases-{fam}.json")); o = json.load(open(f"res/{fam}.oracle.json")); r = json.load(open(f"res/{fam}.now.json"))
    cnt = collections.Counter(); other = []
    for x, y, z in zip(c, o, r):
        if y.get("noexec"): exp = x["expect"]
        else:
            if "syntax error" in y["err1"] or "unexpected" in y["err1"]: continue
            exp = 2 if (y["heavy"] if x["hook"] == "guard" else y["detached"]) else 0
        if z["rc"] == exp: continue
        kind = "误拒" if z["rc"] == 2 else "漏拒"
        for name, rx in rules:
            if re.search(rx, x["cmd"]): cnt[(kind, name)] += 1; break
        else: cnt[(kind, "其它")] += 1; other.append(x["cmd"])
    print("==", fam)
    for (kind, name), n in sorted(cnt.items()): print(f"   {kind} {n}  {name}")
    for cmd in other[:8]: print("   其它:", repr(cmd)[:150])
