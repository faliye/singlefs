"""把 worlds 日志的 SCEN 行压成一行一格：世界、覆盖写次数、I-3.1 / I-3.11 / I-3.15、回退点自证与代号核对。用法：python3 -I worlds_table.py <日志>"""
import re, sys
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"SCEN arm=(\w+) (\w+) 之后再覆盖写(\d+)次 : I-3.1=(\S+) I-3.11=(\S+) I-3.15=(\S+) 全部违例=(\[.*?\]) noentry=(\[.*\])$", line.strip())
    if not m:
        continue
    arm, mode, n, a, b, c, allv, ne = m.groups()
    cells = re.findall(r"self_check=(\w+)\|header_birth=(\S+?)\|record_generation=(\S+?)(?:\"|$)", ne)
    cells = sorted(set(cells))
    short = ";".join(f"自证{'过' if s=='true' else '不过'}/头代{h.replace('Some(','').rstrip(')')}/记录代{g.replace('Some(','').rstrip(')')}" for s, h, g in cells) or "无回退点"
    print(f"{arm:7s} {mode:28s} N={n:>3s} I-3.1={a} I-3.11={b} I-3.15={c} 其余违例={[x for x in eval(allv) if x not in ('I-3.1','I-3.11','I-3.15')]} 回退点[{short}]")
