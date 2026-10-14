#!/usr/bin/env python3
"""T6 现算：两轮判决、第二/三轮正文、调度记录里引的计数，逐个在产物与冻结副本上重算。
只读；只打 key=value 行，不打私有值（私有值命中只打份数与行号）。
用法：count_checks.py --repo <仓根> --work-root <~/.local/share/singlefs/work> --frozen3 <第三轮冻结副本> --scratch <新的草稿目录>
  scratch 里放 git archive 出来的集成副本几个标签（after-b4-fixup2 等）与 sha 对照的临时件。
"""
import argparse, collections, hashlib, os, re, subprocess, sys

ap = argparse.ArgumentParser()
ap.add_argument("--repo", required=True)
ap.add_argument("--work-root", required=True)
ap.add_argument("--frozen3", required=True)
ap.add_argument("--scratch", required=True)
ap.add_argument("--skip-measure", action="store_true")
a = ap.parse_args()
REPO, W, F3, S = a.repo, a.work_root, a.frozen3, a.scratch
P = os.path.join(REPO, "research/prompts")
INT = os.path.join(W, "m3-tooling-b2-integration/repo")
FZ = {1: os.path.join(W, "m3-defs-b3-r1/repo"), 2: os.path.join(W, "m3-defs-b3-r2/repo"), 3: F3}
os.makedirs(S, exist_ok=True)

def out(key, val):
    print(f"{key}={val}")

def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()

def lines(path):
    return open(path, encoding="utf-8", errors="replace").read().split("\n")

def sh(cmd, cwd=None, env=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, env=env)
    return r.stdout, r.returncode

# ── 1 快照与冻结副本 ──
for r in (1, 2, 3):
    snap = [l for l in lines(f"{P}/m3-defs-b3-r{r}-snapshot/frozen-copy-sha256.txt") if l.strip()]
    ok = bad = 0
    for l in snap:
        h, p = l.split(None, 1)
        try:
            same = sha(os.path.join(FZ[r], p.strip())) == h
        except OSError:
            same = False
        ok += same; bad += (not same)
    out(f"snapshot.r{r}.lines", len(snap)); out(f"snapshot.r{r}.ok", ok); out(f"snapshot.r{r}.notok", bad)
    out(f"snapshot.r{r}.kb_lines", len([l for l in lines(f"{P}/m3-defs-b3-r{r}-snapshot/kb-sha256.txt") if l.strip()]))

# ── 2 腿报告 sha256（前 8 / 后 4）与行数 ──
for f in ("m3-defs-b3-r1-sonnet-output", "m3-defs-b3-r1-opus-output", "m3-defs-b3-r2-sonnet-output", "m3-defs-b3-r2-opus-output"):
    h = sha(f"{P}/{f}.md"); out(f"legsha.{f}", f"{h[:8]}...{h[-4:]}"); out(f"leglines.{f}", len(lines(f"{P}/{f}.md")) - 1)

# ── 3 补丁与报告：sha、文件数、增删行 ──
patch_dirs = ["b2-guard", "b2-defaults", "b2-threads", "b2-dispatch", "b2-fixup", "b3-defs", "b4-defs", "b4-gates", "b4-guard",
              "b4-dispatch", "b4-fixup", "b4-fixup2", "b5-defs", "b5-guard", "b5-gates", "b5-dispatch", "b5-fixup"]
for d in patch_dirs:
    base = os.path.join(W, f"m3-tooling-{d}")
    pp = os.path.join(base, "patch.diff")
    if os.path.exists(pp):
        t = open(pp, encoding="utf-8").read().split("\n")
        files = sum(1 for l in t if l.startswith("diff --git"))
        add = sum(1 for l in t if l.startswith("+") and not l.startswith("+++"))
        dele = sum(1 for l in t if l.startswith("-") and not l.startswith("---"))
        h = sha(pp)
        out(f"patch.{d}", f"files={files} +{add}/-{dele} lines={len(t)-1} sha={h[:8]}...{h[-5:]}")
    rp = os.path.join(base, "report.md")
    if os.path.exists(rp):
        h = sha(rp); out(f"report.{d}", f"sha={h[:8]}...{h[-5:]} lines={len(lines(rp))-1}")
for d in ("m3-tooling-b1-sync-judge-b", "m3-tooling-b2-mutation-share", "m3-tooling-b1-scribe-owed", "m3-tooling-b1-scribe-ms"):
    h = sha(os.path.join(W, d, "report.md")); out(f"report.{d}", f"sha={h[:8]}...{h[-5:]}")

# ── 4 集成副本：标签间差、新增/改动拆开、基底文件数 ──
def g(*args):
    return sh(["git", "-C", INT] + list(args))[0].strip()
out("int.base_files", len(g("ls-tree", "-r", "--name-only", "c840472").split("\n")))
for a1, a2 in (("before-b4", "after-b4"), ("after-b4", "after-b4-fixup"), ("after-b4-fixup", "after-b4-fixup2"),
               ("after-b4-fixup2", "after-b5"), ("after-b5", "after-b5-fixup")):
    out(f"int.shortstat.{a1}..{a2}", g("diff", "--shortstat", a1, a2))
    for filt in ("M", "A", "D"):
        out(f"int.shortstat.{a1}..{a2}.{filt}", g("diff", "--shortstat", f"--diff-filter={filt}", a1, a2) or "none")
out("int.head_tracked_files", len(g("ls-tree", "-r", "HEAD", "--name-only").split("\n")))
# 批 2 四份补丁叠加（临时 index，不碰工作树）
env = dict(os.environ, GIT_INDEX_FILE=os.path.join(S, "idx-b2"))
if os.path.exists(env["GIT_INDEX_FILE"]): os.remove(env["GIT_INDEX_FILE"])
subprocess.run(["git", "-C", INT, "read-tree", "c840472"], env=env, capture_output=True)
for d in ("guard", "threads", "defaults", "dispatch"):
    subprocess.run(["git", "-C", INT, "apply", "--cached", os.path.join(W, f"m3-tooling-b2-{d}/patch.diff")], env=env, capture_output=True)
out("b2stack.all", sh(["git", "-C", INT, "diff", "--cached", "--shortstat", "c840472"], env=env)[0].strip())
out("b2stack.M", sh(["git", "-C", INT, "diff", "--cached", "--diff-filter=M", "--shortstat", "c840472"], env=env)[0].strip())
out("b2stack.A", sh(["git", "-C", INT, "diff", "--cached", "--diff-filter=A", "--shortstat", "c840472"], env=env)[0].strip())
# 冻结副本 = 集成副本标签？
for r, tag in ((1, "before-b4"), (2, "after-b4-fixup2"), (3, "after-b5-fixup")):
    snap = [l.split(None, 1) for l in lines(f"{P}/m3-defs-b3-r{r}-snapshot/frozen-copy-sha256.txt") if l.strip()]
    same = diff = missing = 0
    for h, p in snap:
        blob = subprocess.run(["git", "-C", INT, "show", f"{tag}:{p.strip()}"], capture_output=True)
        if blob.returncode != 0: missing += 1
        elif hashlib.sha256(blob.stdout).hexdigest() == h: same += 1
        else: diff += 1
    out(f"frozen_vs_tag.r{r}", f"tag={tag} files={len(snap)} same={same} differ={diff} not_in_tag={missing}")
# 排除表行数按标签
for t in ("b088a84", "after-b4-fixup", "after-b4-fixup2", "after-b5-fixup"):
    txt = subprocess.run(["git", "-C", INT, "show", f"{t}:.claude/heavy-thread-exempt"], capture_output=True, text=True).stdout.split("\n")
    out(f"exempt_rows.{t}", len([l for l in txt if l.strip() and not l.startswith("#")]))
for r in (1, 2, 3):
    txt = lines(os.path.join(FZ[r], ".claude/heavy-thread-exempt"))
    rows = [l for l in txt if l.strip() and not l.startswith("#")]
    out(f"exempt_rows.frozen_r{r}", len(rows)); out(f"exempt_rows.frozen_r{r}.crates", sum(1 for l in rows if l.startswith("crates/")))

# ── 5 verify 汇总 ──
VD = os.path.join(W, "m3-tooling-b2-integration")
for v in ("verify", "verify2", "verify3", "verify4"):
    c = collections.Counter(); wrong = []
    for l in lines(os.path.join(VD, f"{v}.summary")):
        m = re.match(r"\s*(\d+)\s+(\S+)\s+期望 (\S+)\s+退 (\d+)\s+(\d+)s\s+(.*)$", l)
        if not m: continue
        idx, name, exp, rc, sec, verdict = m.groups()
        kind = "开关" if name.startswith("brk-") else "其他"
        c[(kind, exp, "对" if verdict.startswith(("对", "取证")) else "错")] += 1
        if not verdict.startswith(("对", "取证")): wrong.append(f"{idx}:{name}")
    out(f"{v}.tally", dict(sorted(c.items())))
    out(f"{v}.total", sum(c.values())); out(f"{v}.wrong", wrong)
# verify 日志里的数
def grep_n(pattern, path):
    return [l for l in lines(path) if re.search(pattern, l)]
out("verify2.cell_hdd", re.search(r"扫了 (\d+) 份.*?可疑写法 (\d+) 处.*?表 (\d+) 行", grep_n("扫了", os.path.join(VD, "verify2-out/001-cell-heavy-thread-defaults.log"))[0]).groups())
out("verify4.cell_hdd", re.search(r"扫了 (\d+) 份.*?可疑写法 (\d+) 处.*?表 (\d+) 行", grep_n("扫了", os.path.join(VD, "verify4-out/014-cell-heavy-thread-defaults.log"))[0]).groups())
v1 = os.path.join(VD, "verify-out/067-cell-heavy-thread-defaults.log")
out("verify1.cell_hdd_red_items", re.search(r"(\d+) 处自己定并行度", grep_n("处自己定并行度", v1)[0]).group(1))
out("verify1.two_hosts_selftest", grep_n(r"格里 \d+ 格判错", os.path.join(VD, "verify-out/066-known-red-two-hosts.log"))[0].strip())
out("verify1.two_hosts_failed_cells", len(grep_n(r"^  ✗ 自证", os.path.join(VD, "verify-out/066-known-red-two-hosts.log"))))
v3 = os.path.join(VD, "verify3-out/014-cell-code-tooling.log")
out("verify3.cell_code_tooling", [l.strip() for l in grep_n(r"格里 \d+ 格红", v3)])
out("verify3.hdd_red_lines", [re.search(r"runner-dispatch-guard.sh:(\d+)", l).group(1) for l in grep_n(r"^     \.claude/hooks/runner-dispatch-guard\.sh:\d+", v3)])
out("verify3.fixture_dates", [l.strip()[:40] for l in grep_n(r"个文件名里的日期不可能是真的|处样本正文里的日期不可能是真的", v3)])
out("verify3.parallel_selftests", [l.strip()[:80] for l in grep_n(r"自证有 \d+ 格判错", v3)])
out("verify2.rwmc_items", grep_n(r"查了 72 项", os.path.join(VD, "verify2-out/008-run-with-memory-cap.log"))[0].strip()[:40])
v5 = os.path.join(VD, "verify5-out")
out("verify5.types_before_nine_red_cells", len(grep_n(r"^  ✗ 自检", os.path.join(v5, "dp-types.log"))))
out("verify5.dp_selftest", grep_n(r"自检通过", os.path.join(v5, "dp.log"))[0].strip()[:30])

# ── 6 清单与小节 ──
for r in (1, 2, 3):
    txt = lines(f"{P}/_m3-defs-b3-r{r}-checklist.md")
    c = collections.Counter()
    for l in txt:
        if l.startswith("| ") and not l.startswith("| 小节") and "---" not in l:
            cols = [x.strip() for x in l.strip("|").split("|")]
            if len(cols) >= 2: c[cols[1]] += 1
    out(f"checklist.r{r}", dict(c)); out(f"checklist.r{r}.tables", sum(1 for l in txt if l.startswith("### 小节清单")))

# ── 7 腿报告里的表 ──
sr = lines(f"{P}/m3-defs-b3-r2-sonnet-output.md")
sec = None; rows = collections.defaultdict(list)
for i, l in enumerate(sr, 1):
    if l.startswith("## "): sec = l.split()[1]
    if l.startswith("| ") and not l.startswith("| 编号") and not l.startswith("| 组") and not l.startswith("| 格") and "---" not in l and not l.startswith("| 写法"):
        rows[sec].append([c.strip() for c in l.strip("|").split("|")])
out("r2sonnet.section1_rows", len(rows["1"]))
_s2 = [i for i, l in enumerate(sr) if l.startswith("## 2 ")][0]
_first = []
for l in sr[_s2 + 1:]:
    if l.startswith("| ") and not l.startswith("| 组") and "---" not in l: _first.append(l)
    elif _first and not l.startswith("|"): break
out("r2sonnet.section2_first_table_rows", len(_first))
t = collections.Counter()
for cols in rows["1"]:
    j = cols[4]
    t["没修到" if j.startswith("没修到") else "部分修到" if j.startswith("部分修到") else "修到类"] += 1
out("r2sonnet.table1_tally", dict(t))
t = collections.Counter(cols[2] for cols in rows["3"]); out("r2sonnet.table3_tally", dict(t))
r5a = lines(os.path.join(P, "m3-defs-b3-r2-opus-model/out/r5-r1.txt")); r5b = lines(os.path.join(P, "m3-defs-b3-r2-opus-model/out/r5-r2.txt"))
pa = [re.search(r"case=(\S+) rc=(\d)", l) for l in r5a if re.search(r"case=(p-|bench)", l)]
pb = [re.search(r"case=(\S+) rc=(\d)", l) for l in r5b if re.search(r"case=(p-|bench)", l)]
out("r2opus.r5_p_and_bench_cases", len(pb)); out("r2opus.r5_cases_0to2", sum(1 for x, y in zip(pa, pb) if x.group(2) == "0" and y.group(2) == "2"))
out("r2opus.r5_cases_unchanged_0", [y.group(1) for x, y in zip(pa, pb) if x.group(2) == "0" and y.group(2) == "0"])
_gs = os.path.join(P, "m3-defs-b3-r2-opus-model/gate-samples")
out("opus_gate_sample_dirs", sum(1 for d in os.listdir(_gs) if os.path.isdir(os.path.join(_gs, d)) for e in os.listdir(os.path.join(_gs, d)) if os.path.isdir(os.path.join(_gs, d, e))))
mk = open(os.path.join(P, "m3-defs-b3-r2-sonnet-model/feeds/make_gate_sample_roots.py"), encoding="utf-8").read()
out("r2sonnet.sample_roots", len(re.findall(r'^root\("gate-', mk, re.M)))

# ── 8 冻结副本里的计数 ──
for r in (1, 2, 3):
    f = FZ[r]
    cap = subprocess.run(["bash", "scripts/hosts/capped.sh", "--print", "8"], cwd=f, capture_output=True, text=True,
                         env={k: v for k, v in os.environ.items() if k != "SINGLEFS_THREAD_CAP"}).stdout.split("\n")
    out(f"r{r}.capped_print8_lines", len([x for x in cap if x.strip()])); out(f"r{r}.capped_print8_E161", sum(1 for x in cap if "E161" in x))
    lay = lines(os.path.join(f, ".claude/gate.d/checker-tier-layer0-replay.sh"))
    out(f"r{r}.layer0_750_765_test_threads", sum(1 for l in lay[749:765] if "--test-threads" in l)); out(f"r{r}.layer0_file_test_threads", sum(1 for l in lay if "--test-threads" in l))
    ag = sorted(os.listdir(os.path.join(f, ".claude/agents")))
    noedit = 0
    for n in ag:
        m = [l for l in lines(os.path.join(f, ".claude/agents", n)) if l.startswith("tools:")]
        if m and not re.search(r"Write|Edit", m[0]): noedit += 1
    out(f"r{r}.agents_total", len(ag)); out(f"r{r}.agents_without_write_edit", noedit)
    rp = lines(os.path.join(f, "scripts/experiments/replay.sh"))
    out(f"r{r}.replay_sh_thread_pins", sum(1 for l in rp if re.search(r"E1[0-9][0-9]_[A-Z_]*THREADS=[0-9]", l)))
    gd = open(os.path.join(f, ".claude/hooks/runner-dispatch-guard.sh"), encoding="utf-8").read()
    m = re.search(r"^THREAD_LIMIT_REQUIRED_AGENTS = \{(.*?)\}", gd, re.S | re.M)
    out(f"r{r}.thread_limit_required_types", len(re.findall(r'"([a-z\-]+)"', m.group(1))) if m else "表不存在")
    ad = [n for n in ag if "预计挂钟" in open(os.path.join(f, ".claude/agents", n), encoding="utf-8").read()]
    out(f"r{r}.agent_defs_with_预计挂钟", len(ad))
    ad2 = [n for n in ag if "线程上限：" in open(os.path.join(f, ".claude/agents", n), encoding="utf-8").read()]
    out(f"r{r}.agent_defs_with_线程上限：", len(ad2))
tests = os.path.join(FZ[2], "crates/singlefs-checker-tier/tests"); src = os.path.join(FZ[2], "crates/singlefs-checker-tier/src")
gpu_files = [n for n in sorted(os.listdir(tests)) if n.endswith(".rs") and re.search(r'feature *= *"gpu"', open(os.path.join(tests, n), encoding="utf-8", errors="replace").read())]
cnt = lambda p: sum(1 for l in open(p, encoding="utf-8", errors="replace") if re.match(r"\s*#\[test\]", l))
out("r2.gpu_cfg_test_files", len(gpu_files)); out("r2.gpu_files_test_attr_lines", sum(cnt(os.path.join(tests, n)) for n in gpu_files))
out("r2.gpu_agree_file_tests", cnt(os.path.join(tests, "crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs")))
lib_n = 0
for dp, dn, fn in os.walk(src):
    if os.path.relpath(dp, src).startswith("bin"): continue
    for n in fn:
        if n.endswith(".rs"): lib_n += cnt(os.path.join(dp, n))
out("r2.lib_test_attr_lines", lib_n)

# ── 9 records 扫描与 8 MiB ──
for tag, root in (("r2", FZ[2]), ("r3", FZ[3]), ("main", REPO)):
    o, _ = sh([sys.executable, "-I", os.path.join(P, "m3-defs-b3-r2-opus-model/probes/r7-records-scan.py"), root])
    out(f"records_scan.{tag}", o.strip().replace("\n", " | "))
big = [os.path.getsize(os.path.join(dp, n)) for dp, dn, fn in os.walk(os.path.join(REPO, "research/results")) for n in fn]
out("main.results_files_over_8MiB", sum(1 for s in big if s > 8 * 1024 * 1024)); out("main.results_largest", max(big))
nul = collections.Counter()
for dp, dn, fn in os.walk(os.path.join(REPO, "research/results")):
    for n in fn:
        if b"\0" in open(os.path.join(dp, n), "rb").read(8192): nul[os.path.splitext(n)[1]] += 1
out("main.results_files_with_NUL_head", dict(nul))

# ── 10 模型目录 sha 对照报告里抄的 ──
def model_sha(mdir):
    rows = []
    for dp, dn, fn in os.walk(mdir):
        for n in fn:
            p = os.path.join(dp, n); rows.append((sha(p), "./" + os.path.relpath(p, mdir)))
    return set(rows)
def report_sha(path, first, last):
    s = set()
    for l in lines(path)[first - 1:last]:
        m = re.match(r"([0-9a-f]{64})\s+(\./\S+)", l)
        if m: s.add((m.group(1), m.group(2)))
    return s
for name, md, rep, a1, a2 in (("r1opus", "m3-defs-b3-r1-opus-model", "m3-defs-b3-r1-opus-output.md", 17, 27), ("r2opus", "m3-defs-b3-r2-opus-model", "m3-defs-b3-r2-opus-output.md", 20, 77),
                              ("r2sonnet", "m3-defs-b3-r2-sonnet-model", "m3-defs-b3-r2-sonnet-output.md", 217, 245)):
    ms, rs = model_sha(os.path.join(P, md)), report_sha(os.path.join(P, rep), a1, a2)
    out(f"modelsha.{name}", f"model_files={len(ms)} report_lines={len(rs)} identical={ms == rs}")

# ── 11 记录里的事实表/候选/处置 ──
fd = os.path.join(W, "m3-tooling-b1-sync-facts")
out("facts_rows", len(lines(os.path.join(fd, "facts.tsv"))) - 2)
cand = [l.split("\t") for l in lines(os.path.join(fd, "candidates.tsv")) if l.strip()][1:]
out("candidate_rows", len(cand)); out("candidate_distinct", len({c[3] for c in cand}))
out("candidates_F1_13", sum(1 for c in cand if int(c[0][1:]) <= 13)); out("candidates_F14_27", sum(1 for c in cand if int(c[0][1:]) >= 14))
tj = lines(os.path.join(W, "m3-tooling-b1-sync-judge-b/table.md"))
out("judge_b_tally", dict(collections.Counter(l.split("|")[3].strip() for l in tj if l.startswith("| F"))))
sync = lines(os.path.join(P, "m3-close-noncode-sync.md"))
on = False; srows = []
for l in sync:
    if l.startswith("## 命中处置"): on = True; continue
    if on and l.startswith("## "): break
    if on and l.startswith("| ") and not l.startswith("| 载体") and "---" not in l:
        srows.append([c.strip() for c in re.split(r"(?<!\\)\|", l)[1:-1]])
out("sync_rows", len(srows)); out("sync_tally", dict(collections.Counter("不改" if r[-1].startswith("不改") else "改" if r[-1].startswith("改") else "补" if r[-1].startswith("补") else "其他" for r in srows)))
out("sync_evidence_rows", sum(1 for l in sync[sync.index("## 原始证据"):] if l.startswith("| ") and not l.startswith("| 材料") and "---" not in l))
td = lines(os.path.join(W, "m3-tooling-b1-thread-defaults/report.md"))
out("inventory_rows", sum(1 for l in td if re.match(r"\| \d+ \|", l)))

# ── 12 skeleton：第三轮冻结副本 14 种类型 ──
types = [n[:-3] for n in sorted(os.listdir(os.path.join(F3, ".claude/agents")))]
a31 = wall = 0
for t in types:
    o, _ = sh([sys.executable, "-I", "scripts/agents/dispatch-prompt.py", t], cwd=F3)
    a31 += bool(re.search(r"^线程上限：31$", o, re.M)); wall += bool(re.search(r"^预计挂钟：", o, re.M))
out("r3.skeleton_types", len(types)); out("r3.skeleton_thread_limit_31", a31); out("r3.skeleton_wall_clock", wall)
# 第三轮正文：被判 14 份，T1 对象不在快照里的
b3 = open(f"{P}/_m3-defs-b3-r3-body.md", encoding="utf-8").read()
m = re.search(r"被判的定义与规则文件（判决按路径点名）：(.*?)。\n", b3)
paths = re.findall(r"`([^`]+)`", m.group(1)); snap3 = {l.split(None, 1)[1].strip() for l in lines(f"{P}/m3-defs-b3-r3-snapshot/frozen-copy-sha256.txt") if l.strip()}
out("r3body.judged_files", len(paths)); out("r3body.judged_files_in_snapshot", sum(p in snap3 for p in paths))
out("r3body.T1_objects_not_in_snapshot", ["scripts/hosts/heavy-threads.sh", "scripts/hosts/heavy_threads.py"] if not any(x in snap3 for x in ("scripts/hosts/heavy-threads.sh", "scripts/hosts/heavy_threads.py")) else "some-in")

# ── 13 私有值命中份数（不打值） ──
cfg = os.path.join(F3, "multi-host.env")
o, _ = sh([sys.executable, "-I", os.path.join(P, "m3-defs-b3-r3-sonnet-model/probes/value_hits.py"), cfg,
           f"main-wt={REPO}", f"head=git:{REPO}:HEAD", f"at-3e3cdc3c=git:{REPO}:3e3cdc3c", f"at-ce1457c3=git:{REPO}:ce1457c3",
           f"origin-master=git:{REPO}:remotes/origin/master", f"r1={FZ[1]}", f"r2={FZ[2]}", f"r3={FZ[3]}"])
print(o.strip())
o, _ = sh([sys.executable, "-I", os.path.join(P, "m3-defs-b3-r3-sonnet-model/probes/value_tree_hits.py"), cfg, REPO]); print(o.strip())
out("git.HEAD", sh(["git", "-C", REPO, "rev-parse", "--short", "HEAD"])[0].strip())
out("git.origin_master", sh(["git", "-C", REPO, "rev-parse", "--short", "remotes/origin/master"])[0].strip())

# ── 14 heavy-thread-defaults 格的判法（抽出来在冻结副本/标签上直接跑，不经门禁入口） ──
for tag in ("after-b4-fixup2", "after-b5", "after-b5-fixup"):
    d = os.path.join(S, f"tag-{tag}")
    if not os.path.isdir(d):
        os.makedirs(d)
        arc = subprocess.Popen(["git", "-C", INT, "archive", tag, "scripts", ".claude", "crates"], stdout=subprocess.PIPE)
        subprocess.run(["tar", "-x", "-C", d], stdin=arc.stdout); arc.wait()
cell = os.path.join(P, "m3-defs-b3-r3-sonnet-model/tools/run_htd_cell.py")
for label, root in (("r1_frozen", FZ[1]), ("r2_frozen", FZ[2]), ("r3_frozen", FZ[3]), ("tag_after-b4-fixup2", os.path.join(S, "tag-after-b4-fixup2")),
                    ("tag_after-b5", os.path.join(S, "tag-after-b5")), ("tag_after-b5-fixup", os.path.join(S, "tag-after-b5-fixup"))):
    env2 = dict(os.environ, TMPDIR=S)
    o, _ = sh([sys.executable, cell, root], env=env2)
    ol = [l.strip() for l in o.split("\n") if l.strip()]
    first = ol[0][:420] if ol else ""
    m = re.search(r"扫了 (\d+) 份.*?可疑写法 (\d+) 处.*?表 (\d+) 行", first)
    out(f"htd_cell.{label}", (f"绿 扫={m.group(1)} 可疑={m.group(2)} 表={m.group(3)}" if m else first[:200]) + f" | {[l for l in ol if l.startswith('name=htd_cell')]}")
    red = [l for l in ol if re.match(r"\S+:\d+\s", l)]
    if red: out(f"htd_cell.{label}.red_items", [r[:150] for r in red])

# ── 15 E3 词源误报量（只数；要 ~/.config/singlefs） ──
if not a.skip_measure:
    meas = os.path.join(W, "m3-tooling-b5-gates/measure-config-words.py")
    o, _ = sh([sys.executable, "-I", meas, INT, cfg, os.path.expanduser("~/.config/singlefs")])
    for l in o.strip().split("\n")[:4]: print("e3." + l)

# ── 16 判决表的行数 ──
v1 = lines(f"{P}/m3-defs-b3-r1-main-verification.md"); v2 = lines(f"{P}/m3-defs-b3-r2-main-verification.md")
cnt_rows = lambda ls, pre: sum(1 for l in ls if re.match(r"\| " + pre + r"\d+ \|", l))
out("v1.rows_D", cnt_rows(v1, "D")); out("v1.rows_G", cnt_rows(v1, "G")); out("v1.fork_items", sum(1 for l in v1 if re.match(r"- H\d：", l)))
out("v2.rows_R", cnt_rows(v2, "R")); out("v2.rows_K", cnt_rows(v2, "K")); out("v2.fork_items", sum(1 for l in v2 if re.match(r"- H\d：", l)))
out("v2.verdict_tally", dict(collections.Counter(re.match(r"\| R\d \| (\S+(?: ?\S+)?) \|", l).group(1) for l in v2 if re.match(r"\| R\d \|", l))))
pile = [l for l in v2 if l.startswith("另立一笔账")][0]
out("v2.pile_line_ids", re.findall(r"（((?:J|B|S|O)\d+)[ ，）]", pile)); out("v2.pile_first_round_ids", re.findall(r"（((?:S|O)\d+)、((?:S|O)\d+)）", pile))
op = lines(f"{P}/m3-defs-b3-r2-opus-output.md")
out("r2opus.rows_hit_ids", [re.match(r"\| R\d \| (\S+) \|", l).group(1) for l in op if re.match(r"\| R\d \| [A-E]\d \|", l)])
out("r2opus.rows_P", sum(1 for l in op if re.match(r"\| P\d+ \|", l)))
b2 = lines(f"{P}/_m3-defs-b3-r2-body.md"); b3l = lines(f"{P}/_m3-defs-b3-r3-body.md")
out("b2.rows_R", cnt_rows(b2, "R")); out("b3.rows_T", cnt_rows(b3l, "T"))

# ── 17 记录与判决里写的 sha256 前缀…末几位，逐个对真文件 ──
claim_texts = {"RC": lines(os.path.join(REPO, "records/2026-10-12-里程碑三工具与治理批.md")),
               "V1": lines(f"{P}/m3-defs-b3-r1-main-verification.md"), "V2": lines(f"{P}/m3-defs-b3-r2-main-verification.md")}
targets = []
for d in patch_dirs:
    for kind in ("patch.diff", "report.md"):
        pth = os.path.join(W, f"m3-tooling-{d}", kind)
        if os.path.exists(pth): targets.append((f"{d}/{kind}", sha(pth)))
for d in ("m3-tooling-b1-sync-judge-b", "m3-tooling-b2-mutation-share"):
    targets.append((f"{d}/report.md", sha(os.path.join(W, d, "report.md"))))
for f in ("m3-defs-b3-r1-sonnet-output", "m3-defs-b3-r1-opus-output", "m3-defs-b3-r2-sonnet-output", "m3-defs-b3-r2-opus-output"):
    targets.append((f"{f}.md", sha(f"{P}/{f}.md")))
for name, h in targets:
    found = []
    for src, ls in claim_texts.items():
        for i, l in enumerate(ls, 1):
            for m in re.finditer(h[:8] + r"…([0-9a-f]{3,6})", l):
                found.append((f"{src}:{i}", m.group(1), h.endswith(m.group(1))))
    out(f"shaclaim.{name}", found if found else "记录里没写前缀")

# ── 18 补丁里点名的几份文件 ──
def patch_files(d):
    txt = open(os.path.join(W, f"m3-tooling-{d}/patch.diff"), encoding="utf-8").read()
    res = {}
    for part in re.split(r"(?m)^diff --git ", txt)[1:]:
        ls = part.split("\n"); name = ls[0].split(" b/")[-1]
        res[name] = (sum(1 for l in ls if l.startswith("+") and not l.startswith("+++")), sum(1 for l in ls if l.startswith("-") and not l.startswith("---")), any(l.startswith("new file mode") for l in ls[:4]))
    return res
for d, f in (("b5-fixup", ".claude/hooks/lib_heavy_tests.py"), ("b5-guard", ".claude/hooks/lib_shell_words.py"), ("b5-guard", ".claude/hooks/selftest-registry.tsv"),
             ("b5-dispatch", ".claude/hooks/lib_abandoned_rounds.py"), ("b2-guard", "scripts/gate/hook-swap.py"), ("b2-fixup", ".claude/hooks/selftest-registry.tsv")):
    out(f"patchfile.{d}.{os.path.basename(f)}", patch_files(d).get(f, "不在补丁里"))
inc = open(os.path.join(W, "m3-tooling-b2-defaults/patch.diff"), encoding="utf-8").read().split("\n")
out("patch.b2-defaults.include_threads_rs", sum(1 for l in inc if re.match(r'\+include!\("heavy_threads_default/threads.rs"\);', l)))
tc = patch_files("b2-threads").get(".claude/gate.d/code-tooling.sh")
tt = open(os.path.join(W, "m3-tooling-b2-threads/patch.diff"), encoding="utf-8").read()
part = [p for p in re.split(r"(?m)^diff --git ", tt) if p.startswith("a/.claude/gate.d/code-tooling.sh ")][0]
out("patch.b2-threads.selftest_runners_added", sum(1 for l in part.split("\n") if l.startswith("+") and "--selftest" in l and not l.startswith("+++") for _ in re.findall(r'"[^"]*--selftest"', l)))

# ── 19 lib_writer_lock.py 的弄坏开关个数与钩子文件头写的个数（按集成副本标签） ──
for tag in ("before-b4", "after-b4", "after-b4-fixup", "after-b5-fixup"):
    lw = subprocess.run(["git", "-C", INT, "show", f"{tag}:.claude/hooks/lib_writer_lock.py"], capture_output=True, text=True).stdout
    m = re.search(r"^KNOWN_BREAK_MODES = \((.*?)\)", lw, re.M)
    gd = subprocess.run(["git", "-C", INT, "show", f"{tag}:.claude/hooks/runner-dispatch-guard.sh"], capture_output=True, text=True).stdout
    h = re.search(r"LIB_WRITER_LOCK_BREAK 的(.)个开关", gd)
    out(f"writer_lock_modes.{tag}", f"modes={len(re.findall(chr(34) + '[a-z-]+' + chr(34), m.group(1))) if m else '无'} 钩子文件头写={h.group(1) if h else '无'}个开关")
