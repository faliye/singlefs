"""K2 三方腿报告路径两侧认法：同一种写法，派发闸 ⑰（three_way_report_verdict，报告行与整份提示两支）认不认、交回闸 ②（leg_reports_named）认不认。
用法：python3 -I leg_forms.py <改后钩子目录> <仓根（只读：读三方腿类型）>。只读。"""
import os, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
hooks, root = sys.argv[1:3]
dispatch = hookload.load(os.path.join(hooks, "runner-dispatch-guard.sh"), hooks, "cur_dispatch")
handback = hookload.load(os.path.join(hooks, "handback-guard.sh"), hooks, "cur_handback")
work = os.path.expanduser("~/.local/share/singlefs/work")
forms = [
    "research/prompts/x-r1-opus-output.md",
    "`research/prompts/x-r1-opus-output.md`",
    "research/prompts/x-r1-opus-output.md。",
    f"{root}/research/prompts/x-r1-opus-output.md",
    "./research/prompts/x-r1-opus-output.md",
    "research/prompts/x-r1-opus-output.md:12",
    "research/prompts/第五轮-opus-output.md",
    "research/prompts/x.v2-r1-opus-output.md",
    "research/prompts/x-r1-opus-output.md.bak",
    "research/prompts/x-r1/x-r1-opus-output.md",
    "research/prompts/<轮>-opus-output.md",
    f"{work}/r5-leg-copy/research/prompts/x-r1-opus-output.md",
    "「research/prompts/x-r1-opus-output.md」",
]
for form in forms:
    prompt = f"重型测试：不跑\n报告：{form}\n"
    dispatched = dispatch.three_way_report_verdict("three-way-attack", prompt, root) is None
    named = [match for match in handback.leg_reports_named(f"报告 {form}，sha256 0000")]
    print(f"{form}：派发闸 ⑰ {'放行' if dispatched else '拒'}；交回闸 ② 认出 {named}")
