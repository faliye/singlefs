"""门禁 doc-decisions 的挂账表 decision-evidence-lag.tsv「只缩不涨」取基准那一版的 base_ledger_text：git 取不到时交什么。
从仓里那一份 doc-decisions.sh 现取 def base_ledger_text 到 def ledger_growth 之前那一段原文执行，对照 lib-lag-baseline.py 的 baseline_text。
用法：python3 ledger_base_probe.py <仓根> <新的空目录>"""
import importlib.util, os, subprocess, sys

root, work = sys.argv[1], sys.argv[2]
source = open(os.path.join(root, ".claude/gate.d/doc-decisions.sh"), encoding="utf-8").read()
start = source.index("def base_ledger_text(")
end = source.index("def ledger_growth(", start)
namespace = {"subprocess": subprocess, "LEDGER": ".claude/gate.d/decision-evidence-lag.tsv"}
exec(source[start:end], namespace)
spec = importlib.util.spec_from_file_location("lag_baseline", os.path.join(root, ".claude/gate.d/lib-lag-baseline.py"))
library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(library)

os.makedirs(os.path.join(work, ".claude/gate.d"))
git = lambda *arguments: subprocess.run(["git", "-C", work, *arguments], check=True, capture_output=True, text=True).stdout
git("init", "-q"); git("config", "user.email", "probe@example.invalid"); git("config", "user.name", "probe")
with open(os.path.join(work, ".claude/gate.d/decision-evidence-lag.tsv"), "w", encoding="utf-8") as handle:
    handle.write("# 表头\ncross-decision-status\t.claude/kb/decisions/x.md\t第 1 行\t片段\t理由\n")
git("add", "-A"); git("commit", "-qm", "基准")
os.chdir(work)
def show(label):
    text = namespace["base_ledger_text"]("HEAD")
    present = namespace["base_ledger_text"]("no-such-base")
    _, problem = library.baseline_text("HEAD", ".claude/gate.d/decision-evidence-lag.tsv")
    print(f"name={label} base_ledger_text_HEAD={'有表' if text is not None else 'None（不比）'} "
          f"base_ledger_text_no_such_base={'有表' if present is not None else 'None（不比）'} "
          f"lib_baseline_text_HEAD={'问题：' + problem.sentence(None) if problem else '没有问题'}")
show("对象库完整")
blob = git("rev-parse", "HEAD:.claude/gate.d/decision-evidence-lag.tsv").strip()
os.remove(os.path.join(work, ".git", "objects", blob[:2], blob[2:]))
show("基准那一版的对象缺了")
