"""对几份提示文件，分别用今天的报告行认法与「只行首」认法起 ⑰ 文件名判（report_file_name_verdict），打判定。
用法：name_verdict_variants.py <派发闸路径> <类型> <提示文件>..."""
import os, re, sys
guard, kind, files = sys.argv[1], sys.argv[2], sys.argv[3:]
text = open(guard).read().splitlines()
start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3")) + 1
end = max(i for i, l in enumerate(text) if l.strip() == "sys.exit(main())")
sys.argv = [sys.argv[0], os.path.dirname(os.path.abspath(guard))]
g = {"__name__": "rdg_probe"}
exec(compile("\n".join(text[start:end]), "rdg_probe", "exec"), g)
TODAY = g["REPORT_LINE"]
LINE_ONLY = re.compile(r"(?P<lead>^[ \t>*_\-]*(?:\d+[.)][ \t]+[ \t*_]*)?)" + g["REPORT_LABEL"] + r"(?P<rest>[^\n]*)", re.M)
for path in files:
    prompt = open(path).read()
    for label, pattern in (("今天（行首或句首）", TODAY), ("只行首", LINE_ONLY)):
        g["REPORT_LINE"] = pattern
        verdict = g["report_file_name_verdict"](kind, prompt)
        print(f"name={os.path.basename(path)} 认法={label} ⑰文件名={'拒' if verdict else '放行'}", (verdict or "").split("\n")[0][:150])
    g["REPORT_LINE"] = TODAY
