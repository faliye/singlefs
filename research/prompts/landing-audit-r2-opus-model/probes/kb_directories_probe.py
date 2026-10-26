""".claude/kb-directories 的几种写法：quote-kb.py 的 kb_candidate_roots 认的目录，与 stale-candidates.py 认不认那个目录下的 .md 为现状载体。
用法：python3 kb_directories_probe.py <仓根> <新的空目录>"""
import importlib.util, os, shutil, sys
repo, probe = sys.argv[1], sys.argv[2]
os.makedirs(os.path.join(probe, "crates/x/invariants")); os.makedirs(os.path.join(probe, ".claude/kb")); os.makedirs(os.path.join(probe, "scripts/kb"))
shutil.copy(os.path.join(repo, "scripts/kb/quote-kb.py"), os.path.join(probe, "scripts/kb/quote-kb.py"))
open(os.path.join(probe, "crates/x/invariants/a.md"), "w").write("# t\n")
sys.dont_write_bytecode = True
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path); module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
stale = load("stale", os.path.join(repo, "scripts/kb/stale-candidates.py"))
quote = load("quote", os.path.join(probe, "scripts/kb/quote-kb.py"))
for form in ("crates/x/invariants", "./crates/x/invariants", os.path.join(probe, "crates/x/invariants"), "crates/x/invariants//"):
    open(os.path.join(probe, ".claude/kb-directories"), "w").write(f".claude/kb  # kb\n{form}  # 探针\n")
    roots = [r.replace(probe, "<探针根>") for r in quote.kb_candidate_roots(probe)]
    carrier = stale.is_current_state_carrier("crates/x/invariants/a.md", stale.carrier_patterns(probe))
    print(f"name=form:{form.replace(probe, '<探针根>')} quote_kb_roots={roots} stale_candidates_carrier={carrier}")
