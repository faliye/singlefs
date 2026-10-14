# 门禁 doc-process-records 的 abandoned-rounds 格：造四个小样本根，让主 agent 在副本里跑格，对照派发闸 ⑥ 的判法（这里不跑门禁格，子 agent 起不了）。
# 用法：python3 -I make_gate_sample_roots.py <新的空目录>
# 然后（主 agent，在冻结副本里，每个根各跑一次，看退出码）：
#   for r in <新的空目录>/gate-*; do (cd <冻结副本> && bash .claude/gate.d/doc-process-records.sh --check abandoned-rounds "$r"); echo "$r rc=$?"; done
# 期望（读格的代码得出，第 800–830 行一带）：gate-weak-source 退 0（格不看出处，派发闸 ⑥ 拒）；gate-bad-date 退 1（日期不是 YYYY-MM-DD，派发闸 ⑥ 收）；
#   gate-duplicated 退 1（同一轮登记两行，派发闸 ⑥ 收）；gate-halted-only 退 1（没登记，派发闸 ⑥ 也拒，halted 这一支已撤）。
import os, sys
work = sys.argv[1]
os.makedirs(work, exist_ok=False)
def root(name, rows, halted=False):
    base = os.path.join(work, name); os.makedirs(os.path.join(base, "research", "prompts"))
    open(os.path.join(base, "research", "prompts", "_r-x-r1-body.md"), "w").write("# r-x-r1 正文\n")
    open(os.path.join(base, "research", "prompts", "abandoned-rounds.tsv"), "w").write("# 表头\n" + "".join(r + "\n" for r in rows))
    if halted:
        open(os.path.join(base, "research", "prompts", "r-x-r1-halted.md"), "w").write("出处：records/x.md 第 1 行「这一轮先不判了，停掉」\n")
good = "不写判决；出处 records/x.md 第 1 行「这一轮先不判了，停掉」"
root("gate-weak-source", ["r-x-r1\t2026-10-12\t用户叫停\t不写判决；出处 research/prompts/_r-x-r1-body.md 第 1 行「自己写的六个字以上的话」"])
root("gate-bad-date", [f"r-x-r1\t昨天\t用户叫停\t{good}"])
root("gate-duplicated", [f"r-x-r1\t2026-10-12\t用户叫停\t{good}", f"r-x-r1\t2026-10-12\t用户叫停（第二行）\t{good}"])
root("gate-halted-only", [], halted=True)
print("name=gate_sample_roots", sorted(os.listdir(work)))
