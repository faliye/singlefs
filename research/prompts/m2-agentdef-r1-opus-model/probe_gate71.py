#!/usr/bin/env python3
"""m2-agentdef-r1 Opus 攻方腿：门禁 71 号的误判（纯指令判红）与漏判（说明句判绿）探针。

只读仓；所有改动都在 <工作目录> 下的拷贝里做，仓里的定义与门禁一个字不动。
用法：python3 probe_gate71.py <仓根> <工作目录>
输出：标准输出（逐格一行，TAB 分隔），末尾一行 summary。
"""
import glob, hashlib, os, re, shutil, subprocess, sys

ROOT, WORK = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
SNAP = "research/prompts/m2-agentdef-r1-start-snapshot.sha256"
GATE = ".claude/gate.d/71-agent-def-flow-only.sh"
FIXTURES = ".claude/gate.d/fixtures/71-agent-def-flow-only.sh"
BEFORE = "research/prompts/m2-agent-def-cleanup/before"


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read(path):
    return open(path, encoding="utf-8").read()


# ── 0. 被探的文件与开工快照一致 ─────────────────────────────
mismatch, checked = [], 0
for row in open(os.path.join(ROOT, SNAP), encoding="utf-8"):
    digest, path = row.split(None, 1)
    path = path.strip()
    if path.startswith(".claude/agents/") or path == GATE:
        checked += 1
        if sha(os.path.join(ROOT, path)) != digest:
            mismatch.append(path)
print(f"snapshot\tchecked={checked}\tmismatch={len(mismatch)}\t{' '.join(mismatch)}")

ORIGINAL_GATE = read(os.path.join(ROOT, GATE))
# 改法 R1（本腿自己提的，被攻过零轮）：④ 与 ② 一样，先剥掉「」再找解释性半句。
R1_OLD = "        for match in HALF_SENTENCE.finditer(line):\n"
R1_NEW = "        for match in HALF_SENTENCE.finditer(without_quoted(line)):\n"
assert ORIGINAL_GATE.count(R1_OLD) == 1
R1_GATE = ORIGINAL_GATE.replace(R1_OLD, R1_NEW)


def make_root(name, agent_sources):
    """建一个只含 .claude/agents/*.md 的假仓根；agent_sources 是要拷进去的文件列表。"""
    root = os.path.join(WORK, name)
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(os.path.join(root, ".claude/agents"))
    for source in agent_sources:
        shutil.copy(source, os.path.join(root, ".claude/agents", os.path.basename(source)))
    return root


def run_gate(root, gate_text):
    gate_path = os.path.join(WORK, "gate-under-test.sh")
    with open(gate_path, "w", encoding="utf-8") as handle:
        handle.write(gate_text)
    result = subprocess.run(["bash", gate_path, root], capture_output=True, text=True)
    return result.returncode, result.stdout + result.stderr


def replace_once(path, old, new):
    text = read(path)
    hits = text.count(old)
    if hits != 1:
        raise SystemExit(f"锚点命中 {hits} 次（要恰好 1 次）：{path}：{old!r}")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))


def criterion_counts(output):
    """从门禁输出的 ✗ 汇总行取四条判据各自的计数；绿就全 0。"""
    counts = {"①": 0, "②": 0, "③": 0, "④": 0}
    for label, pattern in (("①", r"✗ (\d+) 个记录小节"), ("②", r"✗ (\d+) 行写了日期"),
                           ("③", r"✗ (\d+) 处解释性段落"), ("④", r"✗ (\d+) 处解释性半句")):
        found = re.search(pattern, output)
        if found:
            counts[label] = int(found.group(1))
    return counts


def fmt(counts):
    return " ".join(f"{k}={v}" for k, v in counts.items())


TODAY = sorted(glob.glob(os.path.join(ROOT, ".claude/agents/*.md")))

# ── 1. 基线：今天的 18 份文件 ───────────────────────────────
base_root = make_root("baseline", TODAY)
code, out = run_gate(base_root, ORIGINAL_GATE)
print(f"baseline\tfiles={len(TODAY)}\texit={code}\t{out.strip().splitlines()[-1]}")
BASELINE_OUTPUT = out

# ── 2. 误判：在今天的文件上各改一处成纯指令，看 71 号红不红、红在哪条 ─────────
# 每格：(编号, 预期判据, 文件, 旧串, 新串, 这一处是什么指令)
FALSE_POSITIVE_CASES = [
    ("FP1", "④", "three-way-local-attack.md", "「给本地腿的提示一律用英文」。",
     "「给本地腿的提示一律用英文」；`.claude/kb/tooling.md`「提示里的 `**粗体**:` 会诱发损坏（2026-08-29 实测 3/3 对 3/3）」。",
     "开工先读一行多点名一个真实存在的 kb 小节（71 号自己给的改法形态）"),
    ("FP2", "④", "gate-triage.md", "「4. 同一个仓里有没有别的会话在飞？」。",
     "「4. 同一个仓里有没有别的会话在飞？」；`.claude/kb/tooling.md`「并发：两个会话在同一个仓里同时工作过（2026-08-30 实测）」。",
     "同上，另一个真实 kb 小节"),
    ("FP3", "①", "kb-scribe.md",
     "5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。\n",
     "5. 收尾再记一次每个被改文件的 `sha256sum`，报告里列全部改过的文件与开工时、收尾时两次哈希。\n\n### 决策变更史的条目\n\n- 同一天多条的「（其N）」照当月文件现取，其余逐字照规格。\n",
     "书记员的活本身就是写变更史：给这一步单开一个小节"),
    ("FP4", "④", "prior-art.md", "出处、日期、实测还是读文档、口径", "出处，日期，实测还是读文档，口径",
     "交付字段清单，只把顿号换成逗号"),
    ("FP5", "④", "kb-scribe.md", "；再 `--dry-run` 核全部恰好命中一次", "；试跑 `--dry-run`，核全部恰好命中一次",
     "「试跑」当动词"),
    ("FP6", "③", "kb-scribe.md", "- 逐条改动规格：文件、旧串（原文整行）、新串、依据（判决或用户定案的出处）。",
     "- 逐条改动规格，每条四项：\n  - 文件、旧串（原文整行）、新串；\n  - 依据：判决或用户定案的出处。",
     "输入项「依据」拆成子列表"),
    ("FP7", "②", "agent-common.md", "报告里的时刻写清是哪个时区。",
     "报告里的时刻写清是哪个时区（写成 `2026-09-18 14:08 JST` 这种形式）。", "时刻格式的例子"),
    ("FP8", "④", "three-way-materials.md", "6. 报出标「不抄」的每一行与理由，让主 agent 过目。",
     "6. 报出标「不抄」的每一行（理由：照清单原样抄），让主 agent 过目。", "交付物里的「理由」一栏"),
    ("FP9", "④", "mutation-triage.md", "逐条分类表（变异名 / 类别 / 依据）",
     "逐条分类表（变异名，类别，依据：分类引的 `mutation-sampling.md` 哪一类）", "交付表的列名"),
]
KB_HEADINGS = {"FP1": "### 提示里的 `**粗体**:` 会诱发损坏（2026-08-29 实测 3/3 对 3/3）",
               "FP2": "## 并发：两个会话在同一个仓里同时工作过（2026-08-30 实测）"}
tooling = read(os.path.join(ROOT, ".claude/kb/tooling.md")).split("\n")
for case, heading in KB_HEADINGS.items():
    where = [str(i + 1) for i, text in enumerate(tooling) if text == heading]
    print(f"kb-heading\t{case}\t.claude/kb/tooling.md:{','.join(where) or 'MISSING'}")

fp_red_expected = fp_r1_green = 0
for case, expected, name, old, new, what in FALSE_POSITIVE_CASES:
    root = make_root(case, TODAY)
    replace_once(os.path.join(root, ".claude/agents", name), old, new)
    code, out = run_gate(root, ORIGINAL_GATE)
    counts = criterion_counts(out)
    details = [line.strip() for line in out.splitlines() if f".claude/agents/{name}:" in line]
    hit = code == 1 and counts[expected] >= 1
    fp_red_expected += hit
    r1_code, r1_out = run_gate(root, R1_GATE)
    fp_r1_green += r1_code == 0
    print(f"{case}\t{what}\texpected={expected}\texit={code}\t{fmt(counts)}\tR1_exit={r1_code}\t{' | '.join(details)}")

# ── 3. R1 会不会让原有判别力掉：两套样本 + 清理前的原件 ──────────────────
def fixture_verdict(kind, gate_text):
    source = os.path.join(ROOT, FIXTURES, kind)
    work = os.path.join(WORK, f"fixture-{kind}")
    shutil.rmtree(work, ignore_errors=True)
    shutil.copytree(source, work)
    subprocess.run(["bash", "setup.sh"], cwd=work, check=True, capture_output=True)
    code, out = run_gate(work, gate_text)
    expect = read(os.path.join(source, "expect")).splitlines()
    want_exit = int([row for row in expect if row.startswith("exit=")][0][5:])
    wants = [row[5:] for row in expect if row.startswith("want=")]
    missing = [want for want in wants if want not in out]
    return code == want_exit and not missing, code, len(wants), missing


for label, gate_text in (("original", ORIGINAL_GATE), ("R1", R1_GATE)):
    for kind in ("red", "green"):
        ok, code, wants, missing = fixture_verdict(kind, gate_text)
        print(f"fixture\t{label}\t{kind}\tpass={ok}\texit={code}\twants={wants}\tmissing={missing}")

before_sources = sorted(glob.glob(os.path.join(ROOT, BEFORE, "agents/*.md"))) + [
    os.path.join(ROOT, BEFORE, "agent-common.md"), os.path.join(ROOT, BEFORE, "main-agent.md")]
before_root = make_root("before", before_sources)
for label, gate_text in (("original", ORIGINAL_GATE), ("R1", R1_GATE)):
    code, out = run_gate(before_root, gate_text)
    print(f"before-files\t{label}\tfiles={len(before_sources)}\texit={code}\t{fmt(criterion_counts(out))}")

# ── 4. 漏判：今天的文件里 71 号判绿、按规则是说明的句子 ─────────────────
# 每格：(编号, 形态, 文件, 行号, 那一行里的原文片段)。行号是那份文件自己的行号；探针逐格核片段真在那一行、
# 基线输出里没有这一行的位置（即 71 号判它绿），并看清理前的原件里有没有同一片段。
MISS_CASES = [
    ("M01", "S1 实测记录，「实测」前面不是标点", "crash-verifier.md", 24, "实测约 47 分钟"),
    ("M02", "S2 轮名代替日期的实测数", "kb-scribe.md", 26, "（agent-defs-r2 攻方模型：1/3 对 0/3）"),
    ("M03", "S3 「今天」代替日期的现状数", "mutation-triage.md", 25, "（今天 4 张）"),
    ("M04", "S4 不带文件路径的经过指路", "mutation-triage.md", 25, "共用的会让开头几行链接上一轮最后一条变异，计划第十八节"),
    ("M05", "S5 目的状语", "experiment-runner.md", 26, "，免得读数被抢"),
    ("M06", "S5 目的状语", "experiment-designer.md", 28, "，免得命中行把结论带进来"),
    ("M07", "S5 目的状语", "experiment-designer.md", 29, "，防等价变异"),
    ("M08", "S5 目的状语", "crash-verifier.md", 13, "派你是为了在提交前的整轮门禁之前先拿到它们的读数"),
    ("M09", "S6 括注里的为什么", "crash-verifier.md", 23, "（两份层 0 全量同时跑各要一个多钟头）"),
    ("M10", "S6 括注里的为什么", "gate-triage.md", 24, "（两道门禁同时跑，重阶段互相拖、工作区指纹互相干扰）"),
    ("M11", "S6 括注里的为什么", "experiment-runner.md", 27, "（后面的门禁阶段都不查命名）"),
    ("M12", "S6 括注里的为什么", "experiment-runner.md", 30, "（`replay.sh` 按这个相对路径找）"),
    ("M13", "S6 括注里的为什么", "kb-scribe.md", 28, "（变异表锚点里的分项标签不跟着改，会腐化）"),
    ("M14", "S6 括注里的为什么", "kb-scribe.md", 28, "（入库产物里印着旧标签的，复跑会对不上）"),
    ("M15", "S6 括注里的为什么", "kb-scribe.md", 27, "（`--write` 会先往那些条目里写「（待补）」）"),
    ("M16", "S6 括注里的为什么", "implementation-writer.md", 46, "（56 号门禁要的判决文件由主 agent 的那一轮产出）"),
    ("M17", "S6 括注里的为什么", "mutation-triage.md", 25, "在仓根下跑会被报成「基线就是红的」"),
    ("M18", "S7 冒号或逗号引出的为什么", "kb-scribe.md", 26,
     "用它不用逐条 Edit：两阶段写在有一处不中时一个文件都不写，逐条 Edit 中途被打断或锚点被别人改掉会留下半套"),
    ("M19", "S7 冒号或逗号引出的为什么", "agent-common.md", 64, "：`/tmp` 下的草稿目录会话一重启就没了，那一跑等于白干"),
    ("M20", "S7 冒号或逗号引出的为什么", "agent-common.md", 43, "，那类读数被抢了 CPU 就不作数"),
    ("M21", "S7 冒号或逗号引出的为什么", "main-agent.md", 26, "：它不读共用约束"),
    ("M22", "S8 「所以」", "kb-scribe.md", 19, "，所以也由主 agent 给"),
    ("M23", "S6 为什么（独立一句）", "experiment-designer.md", 34, "`experiment-runner` 照这些节名找东西，以后门禁也按它们查。"),
    ("M24", "S9 格言（已在 71 号「判不到的」与第二十八节「没做的」里点名）", "main-agent.md", 20, "发散是探索该有的样子"),
    ("M25", "S9 格言（已在第二十八节「没做的」里点名）", "agent-common.md", 43, "这个仓里别的会话几乎一直在跑 cargo，见到就停等于开不了工"),
    ("M26", "S9 格言", "main-agent.md", 19, "延迟不是丢掉"),
]
BEFORE_NAME = {"agent-common.md": os.path.join(ROOT, BEFORE, "agent-common.md"),
               "main-agent.md": os.path.join(ROOT, BEFORE, "main-agent.md")}
miss_ok = 0
for case, shape, name, lineno, needle in MISS_CASES:
    lines = read(os.path.join(ROOT, ".claude/agents", name)).split("\n")
    present = needle in lines[lineno - 1]
    flagged = f".claude/agents/{name}:{lineno}" in BASELINE_OUTPUT
    before_path = BEFORE_NAME.get(name, os.path.join(ROOT, BEFORE, "agents", name))
    in_before = needle in read(before_path)
    miss_ok += present and not flagged
    print(f"{case}\t{shape}\t.claude/agents/{name}:{lineno}\tpresent={present}\tflagged_by_71={flagged}"
          f"\tsame_text_before_cleanup={in_before}\t{lines[lineno - 1].strip()}")

# ── 5. 改法 R4（本腿自己提的，被攻过零轮）：补几条词法判据，量它在三批文本上的命中 ───────
R4_PATTERNS = [
    ("R4a", re.compile(r"实测[^，。；,;（）()「」]{0,8}\d")),
    ("R4b", re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*-r\d+[^，。；）)]{0,12}\d+\s*/\s*\d+")),
    ("R4c", re.compile(r"今天\s*\d")),
    ("R4d", re.compile(r"(?:[（(，,；;。：:]|——)\s*(?:免得|以免|为了|所以)")),
    ("R4e", re.compile(r"计划第[一二三四五六七八九十]+节")),
]
QUOTED = re.compile(r"「[^「」]*」")


def strip_quoted(text):
    previous = None
    while previous != text:
        previous, text = text, QUOTED.sub("", text)
    return text


def r4_hits(root):
    hits = []
    for path in sorted(glob.glob(os.path.join(root, ".claude/agents/*.md"))):
        fenced = False
        for number, line in enumerate(read(path).split("\n"), 1):
            if re.match(r"^\s*(?:```|~~~)", line):
                fenced = not fenced
                continue
            if fenced or re.match(r"^\s*\|", line):
                continue
            for label, pattern in R4_PATTERNS:
                for match in pattern.finditer(strip_quoted(line)):
                    hits.append(f"{label}@{os.path.basename(path)}:{number}={match.group(0)}")
    return hits


green_fixture_root = os.path.join(WORK, "fixture-green")   # 第 3 节已按 setup.sh 建好
for label, root in (("today", base_root), ("before", before_root), ("fixture-green", green_fixture_root)):
    hits = r4_hits(root)
    print(f"R4\t{label}\thits={len(hits)}\t{' ; '.join(hits)}")

print(f"summary\tfalse_positive_red_on_expected={fp_red_expected}/{len(FALSE_POSITIVE_CASES)}"
      f"\tR1_turns_green={fp_r1_green}/{len(FALSE_POSITIVE_CASES)}\tmiss_cases_green={miss_ok}/{len(MISS_CASES)}")
