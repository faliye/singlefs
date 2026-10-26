"""⑳ 的 14 份真提示（第三轮攻方模型目录，只读）在改后树与攻方改法树（tree-fix）上各判一次，列出与派发闸自证里写的期望不同的那几份。
用法：python3 -I real_cells_fix.py <改后钩子目录> <改法钩子目录> <仓根>"""
import os, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
cur_hooks, fix_hooks, root = sys.argv[1:4]
directory = os.path.join(root, "research", "prompts", "landing-audit-r3-opus-model", "prompts")
for label, hooks in (("改后", cur_hooks), ("改法", fix_hooks)):
    guard = hookload.load(os.path.join(hooks, "runner-dispatch-guard.sh"), hooks, f"guard_{label}")
    status, cells = guard.real_prompt_cells(directory, root)
    wrong = [(name, want, got) for name, want, got in cells if want != got]
    print(f"{label}：{status}，{len(cells)} 格，与期望不同 {len(wrong)} 格" + "".join(f"\n  {name[:60]}：期望 {want}，实得 {got}" for name, want, got in wrong))
