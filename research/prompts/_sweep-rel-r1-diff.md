# 附录二：回扫员阶段同步改造——非代码轮的改动原样（基准 HEAD 58a5ebb，工作区改动；生成于 2026-09-19 00:38 UTC）

这一轮判的是 agent 定义、调度表一行、协作脚本与门禁阶段，不是 `crates/` 代码；不走 `quote-rust-items.py`，改动原样按文件类型分节放：`.claude/agents/sweep.md` 用 `git diff`，`.claude/main-agent.md` 只截改动的那一行改前改后并排（同时有别的会话在改这份文件），新建的文件整份放，`research/scripts/fixtures/stale-candidates-benchmark-facts.tsv` 只放前 5 行与总行数。

## 一、`.claude/agents/sweep.md` 的改动（`git diff HEAD -- .claude/agents/sweep.md`）

```diff
diff --git a/.claude/agents/sweep.md b/.claude/agents/sweep.md
index 8a24516..cd931f5 100644
--- a/.claude/agents/sweep.md
+++ b/.claude/agents/sweep.md
@@ -19,12 +19,12 @@ omitClaudeMd: true
   - 改了一个数或格式常量：旧值、新值、它是哪个量（名字与单位）；这个量已知的式子（分子、分母、从哪几个常量算出来），以及仓里指这个量的常量名、kb 用词。
   - 撤回一条结论：结论原文所在的文件与小节、撤回的依据。
   - 新立一条判据：判据原文所在的文件与小节、它管哪一类对象。
-  - 阶段同步（一个阶段任务结束、暂存之前）：阶段名；改动范围（基准提交，或文件清单）；这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；真活使用的证据由主 agent 在这里给，你不自己读会话记录（`~/.claude/` 照共用约束不碰）。
+  - 阶段同步（一个阶段任务结束、暂存之前）：阶段名；改动范围（基准提交与结束提交，结束在工作区的写「工作区」）；你做哪一段（写事实表，或逐行判：候选表路径与分给你的组号，例 F1–F6）；这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；真活使用的证据由主 agent 在这里给，你不自己读会话记录（`~/.claude/` 照共用约束不碰）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
-第 1–4 步是改数的做法；撤回结论与新立判据的走第 5、6 步；阶段同步走第 7–9 步。
+第 1–4 步是改数的做法；撤回结论与新立判据的走第 5、6 步；阶段同步走第 7–11 步（第 7、8 步写事实表，第 9–11 步逐行判）。
 
 1. 按 `format-evolution.md` 列的派生形态逐类搜：字面量、消息串、按预留宽写的读写、下标（值 − 1）、倍数、商与平方、测试与门禁里钉的产物值。每类先 `grep -c` / `| wc -l` 数，再看内容；输出不截断。
 2. 按量名、常量名、kb 用词再搜一遍，找不含旧值任何派生形态、却在算同一个量的地方（例：从更早一代的值派生出来的数）；这一遍的范围与第一遍相同，`records/` 也要扫。
@@ -32,9 +32,11 @@ omitClaudeMd: true
 4. 给出可登记进 `format-const` 标记 `stale=` 的串：只可能指旧值的那几个，裸数字不给；每个串现扫一遍，命中 0 次才给；「现扫」的范围照 `.claude/gate.d/27-format-constants.sh` 实际扫描的文件集合（读脚本取）。
 5. 撤回结论的：按结论里的关键数、带简称的编号、结论句的主干用词全仓搜（`records/` 也扫），引它的每一处分三类：说的是现状，要改；说的是那一次发生的事，不改（判据：把旧说法换成新的，这句还是不是真话，`evidence-discipline.md`「撤回一个数或一条结论，同样要当场回扫谁在引它」）；分不清，要人看。
 6. 新立判据的：把它管的那一类对象里在册的全部列出来，清单现算、写出列清单的命令；逐条判过 / 不过 / 分不清，不过的写违反的是判据哪一句。只判不改（`evidence-discipline.md`「新立一条判据，当场拿它回扫已有的条目」）。
-7. 阶段同步的：从改动范围与做成的事取关键词——改过的文件路径与文件名、脚本 / hook / 门禁阶段号 / agent 名、带简称的编号、步号、这一阶段新出的数。
-8. 在现状句的载体里搜每个关键词，并搜与关键词同句的进度词（没做、还没、没试跑、没用过、没拿真活、没核过、没测、没有读数、只试跑、只做了一轮、待、欠、开着、计划、下一步）：`CLAUDE.md`、`README.md`、`.claude/agent-common.md`、`.claude/agents/`、`.claude/rules/`、`.claude/kb/`（两份变更史除外）、`records/` 各文件「## 历史版本」以外的正文。不搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改）。
-9. 每处命中分五类：要改（这一阶段之后它不再是真话，给改后的句子）；要补（该登记这一阶段新东西而没登记的地方，例：计划记录的进度句、调度表、归属表）；事件句不改；不相干；分不清，要人看。判据同第 5 步：换成这一阶段之后的说法，原句还是不是真话。再反向问一遍：每件做成的事，在该记它的载体里有没有一处记着。只判不改。
+7. 阶段同步分两段，派发提示写明你做哪一段。**写事实表**：跑 `python3 research/scripts/stale-candidates.py --changes --base 基准 [--target 结束] --out <草稿目录>/changes.md`；读变更清单里每一条变更记录（H 编号）、「现状载体里改掉的片段」、`git log 基准..结束` 的提交正文与派发提示里做成的事，每一件变了的事实写一行事实表（格式照 `stale-candidates.py` 文件头）：旧事实与新事实各写一句；一行写一件事实，同一件事实的几条 H 编号并进同一行的出处；检索词写新旧说法都会提到的概念名词、不写状态词、不照抄新句子，宁宽勿窄，它要在基准那一版的现状句里命中；实现落地、实验跑完、欠账还清是事实变了（旧事实写仓里原来怎么说它没有 / 还欠，检索词写那件事的概念名词，不写新函数名），只有这一阶段之前一个字都没提过的东西（新立的欠账、新门禁阶段）才在旧事实前写「新立：」，它的检索词要在基准那一版零命中、不出候选；在结束那一版命中不超过 150 行，太宽就用 `A&&B` 并上第二个概念名词收窄；事实真变了的（数、状态、口径、有没有）不许写成「只改措辞」，只改措辞的行不出候选；出处写它来自哪几条 H 编号或哪个提交；派发提示列的做成的事各至少一行。只改措辞、没改事实的变更记录也要进某一行的出处（那一行旧新事实写「只改措辞」）。
+8. 写完跑 `stale-candidates.py --check-facts 事实表 --base 基准 [--target 结束]`，退出码不是 0 就补完再跑；过了再跑 `--facts 事实表 --base 基准 [--target 结束] --out 候选表`，把事实表、候选表与两条命令的输出末行交回。
+9. **逐行判**：只判候选表里分给你的组（派发提示给组号，例 F1-F6），每一行都判，不整组放行、不抽样。结束是提交就 `git show 结束提交:路径` 读那一版的上下文，结束是工作区就读文件。判据：这一行里每个描述现状的分句（「已有」「还没有」「只有」「N 条」「未开工」「前置：X」「仍欠：X」「要等 X」）在结束那一版之后还是不是真话——判的是分句，不是这笔欠账或这条决策还清了没有：欠账照旧开着、而它写的前置或理由已经过时，照样判要改。带日期的分句，日期后面说的是那天发生了什么（改成、定案、跑了一次）才是事件句；说的是到那天为止的现状（「2026-09-14 已有 X；层 0 只有第一个事务」的后半句）就当现状句判。「新事实」一列写得不清时，以结束那一版的里程碑文件、`git log` 与代码为准。判定：要改（给改后的句子）；要补（该登记这一阶段新东西而没登记）；事件句不改（说的是那一次发生的事）；不相干（这一行提到了检索词、说的却不是这件事实）；要人看。最后一格写改后的句子或理由，理由写这一行说的是什么、为什么不受这件事实影响。
+10. 读完分到的组，再反向问一遍：每件做成的事，在该记它的载体里有没有一处记着，没有的在「## 逐行判定」里加一行（组写 M1、M2……，判定「要补」）。不搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改）。
+11. 交回前跑 `python3 research/scripts/stale-candidates.py --check-report 候选表 你的报告 --groups 分给你的组号`，退出码不是 0 就补完再跑，输出末行原样抄进报告。只判不改。
 
 ## 写范围
 
@@ -44,9 +46,10 @@ omitClaudeMd: true
 
 - 改数的：每类搜索的命令与计数；命中清单（文件:行 / 原行整行抄 / 五类之一 / 理由）；`stale=` 候选及其 0 命中的命令。
 - 撤回结论与新立判据的：搜索或列清单的命令与计数；清单（文件:行 / 原行整行抄 / 判定 / 理由）。
-- 阶段同步的：每个关键词的搜索命令与计数（主 agent 抄进同步记录的「## 搜索」）；命中清单（载体 `文件:行` / 原句整句抄 / 五类之一 / 改后的句子或理由）。
+- 阶段同步写事实表的：事实表、候选表，`--check-facts` 与 `--facts` 的输出末行。
+- 阶段同步逐行判的：「## 逐行判定」表（| 组 | 载体 | 判定 | 改后的句子或理由 |，格式照 `research/scripts/stale-candidates.py` 文件头）；第 11 步 `--check-report` 的输出末行。
 
 ## 没做什么（固定会有的）
 
 - 没改任何文件；按式子推不出的派生（手算后硬编码、换了单位的）可能漏，照写。
-- 阶段同步只搜得到派发提示里写了的事：只用过、没留改动、提示里又没写的，搜不到。
+- 阶段同步的候选表只罩事实表写到的事实、检索词搜得到的句子；只用过、没留改动、提示与变更记录里又没写的事，搜不到。
```

## 二、`.claude/main-agent.md`「一个阶段任务结束」那一行，改前改后并排

**改前**（`git show HEAD:.claude/main-agent.md` 第 43 行）：

```
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | 主 agent 跑 `research/scripts/stale-candidates.py --base 基准 [--target 结束] --out research/prompts/<阶段>-candidates.tsv`，候选超过 150 组就按组号区间切成几段、一段派一个 → `sweep`（阶段同步；输入给改动范围、候选表与组号区间、做成的事，只用过、没留改动的也写）→ 交回齐了跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
```

**改后**（工作区，同一行；`.claude/main-agent.md` 同时有别的会话的改动，这一轮只判这一行）：

```
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再从判成不相干与事件句不改的行里随机抽 20 行自己复判，抽到判错的那一段重派 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
```

## 三、新建文件全文：`.claude/gate.d/73-research-gate-lint.sh`

```bash
#!/usr/bin/env bash
# gate-stage: research/scripts/ 与 .claude/hooks/ 里每一条拒绝都带出路
#
# 共享门禁的「门禁自检」只把 SOP 自己的脚本与 .claude/gate.d/ 交给 gate-lint（.claude/singlefs-ai-sop/scripts/gate.sh 第 211–212 行），
# 研究脚本与 hook 不在射程里。2026-09-18 单跑整仓 gate-lint 红 90 处，84 处在这两个目录，没有任何一笔账记着（C382（研究脚本与 hook 的拒绝不在门禁自检的射程里））。
# research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）。
# 判别力：fixtures/73-research-gate-lint.sh/red 放一个只带一句话的 die，必须判红；green 带上第二个参数，必须判绿。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LINT="$(cd "$(dirname "$0")/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"
[[ -f "$LINT" ]] || { echo "  ✗ 找不到共享脚本 $LINT"; echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh"; exit 1; }
TARGETS=()
for directory in "$ROOT/research/scripts" "$ROOT/.claude/hooks"; do
  [[ -d "$directory" ]] && TARGETS+=("$directory")
done
((${#TARGETS[@]})) || { echo "  ! 没有 research/scripts/ 也没有 .claude/hooks/，本阶段无对象可判"; exit 77; }
if bash "$LINT" "${TARGETS[@]}"; then
  exit 0
fi
echo "  ✗ research/scripts/ 或 .claude/hooks/ 里有不带出路的拒绝（上面逐处列出）"
echo "     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，打印的 ✗ 之后跟一行 →"
exit 1
```

## 四、新建文件全文：`.claude/gate.d/fixtures/73-research-gate-lint.sh/green/research/scripts/sample.sh`

```bash
#!/usr/bin/env bash
source "$(dirname "$0")/lib.sh"
[[ -f input.txt ]] || die "找不到 input.txt" "先跑 make-input.sh 生成它"
```

## 五、新建文件全文：`.claude/gate.d/fixtures/73-research-gate-lint.sh/green/expect`

```
exit=0
```

## 六、新建文件全文：`.claude/gate.d/fixtures/73-research-gate-lint.sh/red/research/scripts/sample.sh`

```bash
#!/usr/bin/env bash
source "$(dirname "$0")/lib.sh"
[[ -f input.txt ]] || die "找不到 input.txt"
```

## 七、新建文件全文：`.claude/gate.d/fixtures/73-research-gate-lint.sh/red/expect`

```
exit=1
want=补上第二个参数
want=research/scripts/ 或 .claude/hooks/ 里有不带出路的拒绝
```

## 八、`research/scripts/fixtures/stale-candidates-benchmark-facts.tsv`（只放前 5 行与总行数；文件共 61 行）

```
编号	旧事实	新事实	检索词	出处
A1	新立：C374 这条欠账编号	C374（释放代与树表诞生 txg 只有验收断言盯着）新立	C374	H1
A2	新立：C359 到 C364 这六条欠账编号	C359 到 C364 六条新立（挂载判定头宽、journal 算法类型预留、打包容器新类头宽/槽宽/码、保留池树高、整理意图记录字段表）	C359|C360|C361|C362|C363|C364	H2
A3	C73（分配记录的态别没定） 的 map_provenance 分项号	C73（分配记录的态别没定） 的 map_provenance 分项号从 D18（块里携带什么信息） 已定项 3 改到已定项 5	C73	H2
A4	C323（镜像大小全仓没有条款） 的行首	C323（镜像大小全仓没有条款） 行首按今天的 4 GiB 与产物读数改	C323	H2
```

## 九、新建文件全文：`research/scripts/stale-candidates.py`（578 行，分四段贴）

### 九·一（第 1–150 行）

```python
#!/usr/bin/env python3
"""阶段同步的候选表：这一阶段哪些事实变了，仓里还有哪些现状句在说旧的。回扫员照表逐行判，不自己挑关键词、不整组放行。

用法（按次序）：
    stale-candidates.py --changes --base 提交 [--target 提交] --out 变更清单.md
        列出这一阶段在 kb 各文件「## 历史版本」节里新加的每一条变更记录（编号 H1、H2……），以及每个被改过的现状载体
        里删改前后的差异片段——写事实表的原料
    stale-candidates.py --check-facts 事实表.tsv --base 提交 [--target 提交]
        核事实表罩没罩全：每条 H 编号都要出现在某一行的「出处」列里；每行的检索词要是合法的正则，
        而且在基准那一版的现状句里至少命中一行（旧说法当时就在那里）；新立事项反过来，检索词在基准那一版要零命中
    stale-candidates.py --facts 事实表.tsv --base 提交 [--target 提交] --out 候选.tsv
        按事实表的检索词在全部现状载体里搜，每处命中一行候选
    stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]
        核报告有没有判到候选表里（分到的组里）的每一行
    stale-candidates.py --benchmark     拿仓里一段真实历史与一份盲写的事实表当阳性对照
    stale-candidates.py --selftest

事实表（制表符分隔，首行是表头）：
    编号	旧事实	新事实	检索词	出处
    F1	层 0 的负载只有第一个事务	层 0 两条流：……	层 0|崩溃点重放|录制流	H3;H17;提交 cae5092
  检索词是 Python 正则，写这件事实涉及的概念名词（「层 0」「录制流」「多次挂载」），新旧两种说法都会提到的那种；
  不写状态词，不照抄新说法的原句（「按整条流切段」只搜得到改好了的句子）；宁宽勿窄。
  几个词要同时出现就用 && 连（「E142&&改动计数」）；一行事实的检索词在结束那一版命中不许超过 150 行，超了就是太宽，用 && 收窄。
  出处写这条事实来自哪几条变更记录（H 编号，--changes 给的）或哪个提交。
  旧事实写「只改措辞」的行只用来罩住变更记录，不出候选。
  旧事实以「新立：」开头的行是这一阶段之前仓里一个字都没提过的东西（新立的欠账、新门禁阶段），也不出候选；
  实现落地、实验跑完、欠账还清不是新立，是事实变了：旧事实写仓里原来怎么说它没有 / 还欠，检索词写那件事的概念名词。

报告（--check-report 判的就是这个）：
    ## 逐行判定
    | 组 | 载体 | 判定 | 改后的句子或理由 |
  候选表里每一行都要有一行；判定以「要改」「要补」「事件句不改」「不相干」「要人看」之一开头；
  最后一格不能空：要改的写改后的句子，其余写理由（至少 8 个字）。几份报告合起来判。

退出码：0 通过；2 参数或 git 出错；3 报告漏判或判定格式不对；4 阳性对照漏了已知的腐烂；5 自证失败；6 事实表没罩全。
"""
import argparse
import difflib
import hashlib
import os
import re
import subprocess
import sys
import tempfile

CARRIER_PATTERNS = [
    r'^CLAUDE\.md$',
    r'^README\.md$',
    r'^\.claude/agent-common\.md$',
    r'^\.claude/main-agent\.md$',
    r'^\.claude/agents/[^/]+\.md$',
    r'^\.claude/rules/[^/]+\.md$',
    r'^\.claude/skills/[^/]+/SKILL\.md$',
    r'^\.claude/handover/[^/]+/README\.md$',
    r'^\.claude/kb/.+\.md$',
    r'^records/[^/]+\.md$',
]
CHANGE_LOG_ONLY_PATTERNS = [
    r'^\.claude/kb/decisions-history',
    r'^\.claude/kb/experiments-history\.md$',
]
HISTORY_SECTION_TITLE = '## 历史版本'
HISTORY_ENTRY_HEADING = re.compile(r'^### .+$', re.M)
LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')
MINIMUM_REASON_LENGTH = 8
FACT_TABLE_COLUMNS = ['编号', '旧事实', '新事实', '检索词', '出处']
NEWLY_ADDED_OLD_FACT_PREFIX = '新立：'
NOT_NEWLY_ADDED_WORDS = re.compile(r'落地|实现|已有|跑完|还清|有了')
REWORDING_ONLY_OLD_FACT_PREFIX = '只改措辞'
MAXIMUM_CANDIDATE_LINES_PER_FACT = 150
TERM_CONJUNCTION = '&&'


def compile_search_term(term):
    """「A&&B」→ 每一段各编一个正则，一行里全都命中才算命中。"""
    return [re.compile(part) for part in term.split(TERM_CONJUNCTION)]


def search_term_matches(compiled_parts, line):
    return all(part.search(line) for part in compiled_parts)

# 阳性对照：2026-09-18 知识腐烂回扫里主 agent 逐条现查坐实的 25 处腐烂（research/prompts/knowledge-rot-2026-09-18-sync.md），
# 行号是 BENCHMARK_TARGET 那一版的。只存「载体:行号」的 sha256 前 16 位：回扫员读得到这个文件，明文会被照抄成答案。
# 事实表 BENCHMARK_FACTS 由一个没读过这 25 处的 agent 照这一段历史的提交正文与变更记录盲写（第四版，
# research/prompts/sweep-acceptance-2026-09-18-v2-facts.md），它罩住其中 20 处；对照守的是「工具在这份表上不少于这个数」。
BENCHMARK_BASE = 'b1c8cef~1'
BENCHMARK_TARGET = '00c9d4f'
BENCHMARK_FACTS = 'research/scripts/fixtures/stale-candidates-benchmark-facts.tsv'
BENCHMARK_MINIMUM_COVERED = 20
BENCHMARK_KNOWN_STALE_LOCATION_HASHES = [
    '88600dee84e43cb1', '603dc36527671cbc', 'a93c0915fab7efa8', '948054ed6df42b0b', '7bb23ca4ed5f8432',
    '6287ca8bd47723bf', '201247a903f206ae', '16403eb11c52d13c', '82e4f62b752e4383', '84b95c231a2692a3',
    '767dc5b0a2eed64f', '43cec00ac6c7e52f', '7ec30e7996379b13', 'ef8cdf810547c225', '799d9c319db094c0',
    '8eb3cae28cc99670', 'bbed8920622afcc2', '4af43929ce1d4f2b', '9919777fff2f4391', '7c5fdaebc4e022d3',
    '873c19307057473f', '8dfe08b83d4f054f', '86b893c315e5795e', 'a3ae13e499fa522c', 'b4e110d8f010be83',
]


def location_hash(location):
    return hashlib.sha256(location.encode('utf-8')).hexdigest()[:16]


def matches_any(path, patterns):
    return any(re.search(pattern, path) for pattern in patterns)


def is_current_state_carrier(path):
    return matches_any(path, CARRIER_PATTERNS) and not matches_any(path, CHANGE_LOG_ONLY_PATTERNS)


def run_git(arguments, repository_root):
    completed = subprocess.run(['git', '-C', repository_root, '-c', 'core.quotepath=false', *arguments],
                               capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(f'git {" ".join(arguments)} 失败：{completed.stderr.strip()}')
    return completed.stdout


def read_version(repository_root, revision, path):
    """revision 为 None 读工作区；文件不在返回空串。"""
    if revision is None:
        full_path = os.path.join(repository_root, path)
        if not os.path.isfile(full_path):
            return ''
        with open(full_path, encoding='utf-8', errors='replace') as handle:
            return handle.read()
    completed = subprocess.run(['git', '-C', repository_root, 'show', f'{revision}:{path}'],
                               capture_output=True, text=True)
    return completed.stdout if completed.returncode == 0 else ''


def all_paths(repository_root, revision):
    if revision is None:
        paths = (run_git(['ls-files'], repository_root).split('\n')
                 + run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n'))
    else:
        paths = run_git(['ls-tree', '-r', '--name-only', revision], repository_root).split('\n')
    return sorted({path for path in paths if path})


def changed_paths(repository_root, base, target):
    paths = run_git(['diff', '--name-only', base] + ([target] if target else []), repository_root).split('\n')
    if target is None:
        paths += run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n')
    return sorted({path for path in paths if path})


def current_state_lines(text):
    """历史版本节之前的行，(行号, 原文)。"""
    lines = []
```

### 九·二（第 151–300 行）

```python
    for line_number, line in enumerate(text.split('\n'), start=1):
        if line.strip() == HISTORY_SECTION_TITLE:
            break
        lines.append((line_number, line))
    return lines


def history_headings(text, path):
    """变更记录的标题：普通 kb 文件取「## 历史版本」之后的三级标题；变更史文件整份都是变更记录。"""
    if matches_any(path, CHANGE_LOG_ONLY_PATTERNS):
        section = text
    else:
        start = text.find('\n' + HISTORY_SECTION_TITLE)
        if start < 0:
            return []
        section = text[start:]
    return HISTORY_ENTRY_HEADING.findall(section)


def renamed_from(repository_root, base, target):
    """{新路径: 旧路径}：搬了目录、改了名的文件，历史标题要拿旧路径那一版去比。"""
    output = run_git(['diff', '--name-status', '-M', base] + ([target] if target else []), repository_root)
    renames = {}
    for line in output.split('\n'):
        columns = line.split('\t')
        if len(columns) == 3 and columns[0].startswith('R'):
            renames[columns[2]] = columns[1]
    return renames


def new_change_entries(repository_root, base, target):
    """[(H 编号, 文件, 标题)]，按文件与出现次序编号，同一 base / target 下稳定。"""
    entries = []
    renames = renamed_from(repository_root, base, target)
    for path in changed_paths(repository_root, base, target):
        if not path.startswith('.claude/kb/') or not path.endswith('.md'):
            continue
        base_path = renames.get(path, path)
        before = set(history_headings(read_version(repository_root, base, base_path), base_path))
        for heading in history_headings(read_version(repository_root, target, path), path):
            if heading not in before:
                entries.append((path, heading))
    return [(f'H{index}', path, heading) for index, (path, heading) in enumerate(entries, start=1)]


def changed_segments(base_text, target_text):
    """删改前后的差异片段 [(旧片段, 新片段)]：每一行被删的现状句配上同一文件新加的最像的一行，按字符比出改掉的那几段。"""
    base_lines = [line for _, line in current_state_lines(base_text)]
    target_lines = [line for _, line in current_state_lines(target_text)]
    target_set, base_set = set(target_lines), set(base_lines)
    added = [line for line in target_lines if line not in base_set and line.strip()]
    segments = []
    for removed in (line for line in base_lines if line not in target_set and line.strip()):
        replacement = max(added, default='', key=lambda candidate: difflib.SequenceMatcher(
            None, removed, candidate, autojunk=False).quick_ratio())
        matcher = difflib.SequenceMatcher(None, removed, replacement, autojunk=False)
        for operation, old_start, old_end, new_start, new_end in matcher.get_opcodes():
            if operation in ('replace', 'delete') and len(removed[old_start:old_end].strip()) >= 2:
                segments.append((removed[max(0, old_start - 12):old_end + 12].strip(),
                                 replacement[max(0, new_start - 12):new_end + 12].strip()))
    return segments


def write_changes(repository_root, base, target, output):
    output.write(f'# 变更清单 {base}..{target or "工作区"}\n\n## 变更记录（事实表的「出处」列要罩住每一个 H 编号）\n\n')
    output.write('| 编号 | 文件 | 标题 |\n|---|---|---|\n')
    for entry_id, path, heading in new_change_entries(repository_root, base, target):
        output.write(f'| {entry_id} | {path} | {heading[4:].replace("|", "｜")} |\n')
    output.write('\n## 现状载体里改掉的片段\n\n')
    for path in changed_paths(repository_root, base, target):
        if not is_current_state_carrier(path):
            continue
        segments = changed_segments(read_version(repository_root, base, path), read_version(repository_root, target, path))
        if not segments:
            continue
        output.write(f'### {path}\n\n')
        for old_segment, new_segment in segments:
            output.write(f'- 旧：{old_segment[:200]}\n  新：{new_segment[:200]}\n')
        output.write('\n')


def read_fact_table(path):
    with open(path, encoding='utf-8') as handle:
        rows = [line.rstrip('\n').split('\t') for line in handle if line.strip()]
    if not rows or rows[0][:len(FACT_TABLE_COLUMNS)] != FACT_TABLE_COLUMNS:
        raise ValueError(f'{path} 的表头要是：{" / ".join(FACT_TABLE_COLUMNS)}（制表符分隔）')
    facts = []
    for row in rows[1:]:
        if len(row) < len(FACT_TABLE_COLUMNS):
            raise ValueError(f'{path} 这一行少列：{row[0] if row else "（空）"}')
        facts.append(dict(zip(FACT_TABLE_COLUMNS, row)))
    return facts


def current_state_corpus(repository_root, target):
    return {path: current_state_lines(read_version(repository_root, target, path))
            for path in all_paths(repository_root, target) if is_current_state_carrier(path)}


def build_candidates(repository_root, target, facts):
    """[(组号, 旧事实, 新事实, 载体:行号, 原文)]。"""
    corpus = current_state_corpus(repository_root, target)
    candidates = []
    for fact in facts:
        if fact['旧事实'].startswith((REWORDING_ONLY_OLD_FACT_PREFIX, NEWLY_ADDED_OLD_FACT_PREFIX)):
            continue
        compiled_parts = compile_search_term(fact['检索词'])
        for path, lines in corpus.items():
            for line_number, line in lines:
                if search_term_matches(compiled_parts, line):
                    candidates.append((fact['编号'], fact['旧事实'], fact['新事实'], f'{path}:{line_number}', line.strip()))
    return candidates


def write_candidates(candidates, output):
    output.write('组\t旧事实\t新事实\t载体\t原文\n')
    for group, old_fact, new_fact, location, line in candidates:
        output.write(f'{group}\t{old_fact}\t{new_fact}\t{location}\t{line.replace(chr(9), " ")[:500]}\n')


def check_facts(repository_root, base, target, fact_path):
    try:
        facts = read_fact_table(fact_path)
    except ValueError as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：照本脚本文件头「事实表」那一段的格式写，五列用制表符分开')
        return 6
    entries = new_change_entries(repository_root, base, target)
    cited = set()
    for fact in facts:
        cited.update(re.findall(r'H\d+', fact['出处']))
    uncovered = [(entry_id, path, heading) for entry_id, path, heading in entries if entry_id not in cited]
    base_corpus = current_state_corpus(repository_root, base)
    target_corpus = current_state_corpus(repository_root, target)
    broken, empty, too_broad, not_new = [], [], [], []
    for fact in facts:
        try:
            compiled_parts = compile_search_term(fact['检索词'])
        except re.error as error:
            broken.append(f'{fact["编号"]}（{error}）')
            continue
        if fact['旧事实'].startswith(REWORDING_ONLY_OLD_FACT_PREFIX):
            continue
        if fact['旧事实'].startswith(NEWLY_ADDED_OLD_FACT_PREFIX):
            base_hits = any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines)
            if base_hits or NOT_NEWLY_ADDED_WORDS.search(fact['新事实']):
                not_new.append(fact['编号'])
            continue
        target_hits = sum(1 for lines in target_corpus.values() for _, line in lines
                          if search_term_matches(compiled_parts, line))
```

### 九·三（第 301–450 行）

```python
        if target_hits > MAXIMUM_CANDIDATE_LINES_PER_FACT:
            too_broad.append(f'{fact["编号"]}（{target_hits} 行）')
        if not any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines):
            empty.append(fact['编号'])
    if uncovered or broken or empty or too_broad or not_new:
        for entry_id, path, heading in uncovered[:40]:
            print(f'  ✗ {entry_id} 没有任何一行事实罩着：{path} {heading[4:80]}')  # gate-lint:detail
        if broken:
            print(f'  ✗ 检索词不是合法的正则：{"、".join(broken)}')  # gate-lint:detail
        if empty:
            print(f'  ✗ 检索词在基准那一版的现状句里一处都没命中（多半照抄了新说法）：{"、".join(empty)}')  # gate-lint:detail
        if not_new:
            print(f'  ✗ 标成「新立：」却不是新东西（检索词在基准那一版有命中，或新事实写着落地 / 实现 / 已有 / 跑完 / 还清）：'  # gate-lint:detail
                  f'{"、".join(not_new)}')
        if too_broad:
            print(f'  ✗ 检索词太宽，结束那一版命中超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行：{"、".join(too_broad)}')  # gate-lint:detail
        print(f'  ✗ 事实表没罩全：{len(uncovered)} 条变更记录没罩、{len(broken)} 行正则坏了、'  # gate-lint:summary
              f'{len(empty)} 行检索词在基准零命中、{len(too_broad)} 行检索词太宽、{len(not_new)} 行冒充新立')
        print('     → 怎么办：读 --changes 给的变更清单，每条 H 编号写进它说的那件事实的「出处」列（只改措辞的也写一行，旧新事实写「只改措辞」）；'
              '检索词写新旧说法都会提到的概念名词；在基准那一版零命中，就是照抄了新说法或写窄了；太宽的用 && 并上第二个概念名词收窄')
        return 6
    print(f'  ✓ 事实表罩全了：{len(entries)} 条变更记录都有出处，{len(facts)} 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）')
    return 0


def parse_group_ranges(text):
    """「F1-F12,F15」→ 组号集合；None 表示全部。"""
    if not text:
        return None
    selected = set()
    for part in text.split(','):
        match = re.fullmatch(r'([A-Z])(\d+)(?:-[A-Z]?(\d+))?', part.strip())
        if not match:
            if not re.fullmatch(r'[A-Z]\w*', part.strip()):
                raise ValueError(f'组号区间写不认得：{part}')
            selected.add(part.strip())
            continue
        low, high = int(match.group(2)), int(match.group(3) or match.group(2))
        selected.update(f'{match.group(1)}{number}' for number in range(low, high + 1))
    return selected


def table_rows_under(title, text):
    """某个二级标题下表格的数据行（去掉表头与分隔行），每行拆成单元格。"""
    rows, inside = [], False
    for line in text.split('\n'):
        if line.startswith('## '):
            inside = line.strip() == title
            continue
        if inside and line.startswith('|') and not re.match(r'^\|\s*-', line):
            rows.append([cell.strip().strip('`') for cell in line.strip().strip('|').split('|')])
    return [row for row in rows if row and row[0] != '组']


def read_candidate_keys(table_path):
    keys = []
    with open(table_path, encoding='utf-8') as handle:
        next(handle)
        for row in handle:
            columns = row.rstrip('\n').split('\t')
            keys.append((columns[0], columns[3]))
    return keys


def check_reports(table_path, report_paths, selected_groups=None):
    keys = [key for key in read_candidate_keys(table_path) if selected_groups is None or key[0] in selected_groups]
    verdicts = {}
    for report_path in report_paths:
        with open(report_path, encoding='utf-8') as handle:
            for cells in table_rows_under('## 逐行判定', handle.read()):
                if len(cells) >= 3:
                    verdicts[(cells[0], cells[1])] = (cells[2], cells[3] if len(cells) > 3 else '')
    missing = [key for key in keys if key not in verdicts]
    malformed = []
    for key in keys:
        if key not in verdicts:
            continue
        verdict, reason = verdicts[key]
        if not verdict.startswith(LINE_VERDICTS) or len(reason) < MINIMUM_REASON_LENGTH:
            malformed.append(key)
    if missing or malformed:
        if missing:
            print(f'  ✗ {len(missing)} 行候选没有逐行判定，例：'  # gate-lint:detail
                  + '；'.join(f'{group} {location}' for group, location in missing[:10]))
        if malformed:
            print(f'  ✗ {len(malformed)} 行的判定不以五种之一开头，或最后一格不到 {MINIMUM_REASON_LENGTH} 个字，例：'  # gate-lint:detail
                  + '；'.join(f'{group} {location}' for group, location in malformed[:10]))
        print(f'  ✗ 报告没判全：共 {len(keys)} 行候选')  # gate-lint:summary
        print('     → 怎么办：在「## 逐行判定」表里给每一行候选一行 | 组 | 载体 | 判定 | 改后的句子或理由 |，'
              '判定以 要改 / 要补 / 事件句不改 / 不相干 / 要人看 开头，最后一格写改后的句子或理由')
        return 3
    print(f'  ✓ 报告判全了：{len(keys)} 行候选都有逐行判定')
    return 0


def benchmark(repository_root):
    facts_path = os.path.join(repository_root, BENCHMARK_FACTS)
    if not os.path.isfile(facts_path):
        print(f'  ✗ 缺阳性对照的事实表 {BENCHMARK_FACTS}')
        print('     → 怎么办：它随仓走；被删了就从 git 历史里取回，不要照着答案重写')
        return 4
    facts = read_fact_table(facts_path)
    candidates = build_candidates(repository_root, BENCHMARK_TARGET, facts)
    covered = {location_hash(location) for _, _, _, location, _ in candidates}
    covered_known = sum(1 for known in BENCHMARK_KNOWN_STALE_LOCATION_HASHES if known in covered)
    unique_lines = len({location for _, _, _, location, _ in candidates})
    if covered_known < BENCHMARK_MINIMUM_COVERED:
        print(f'  ✗ 阳性对照只罩住 {covered_known} 处已知的腐烂，低于 {BENCHMARK_MINIMUM_COVERED}（{BENCHMARK_BASE}..{BENCHMARK_TARGET}，只存哈希）')
        print('     → 怎么办：候选规则或盲写的事实表被改窄了；看 build_candidates 与 current_state_corpus 的改动，'
              '或那份事实表的检索词是不是被收窄过')
        return 4
    print(f'  ✓ 阳性对照：已知的 {len(BENCHMARK_KNOWN_STALE_LOCATION_HASHES)} 处腐烂里 {covered_known} 处在候选里（下限 {BENCHMARK_MINIMUM_COVERED}）'
          f'（盲写事实表 {len(facts)} 行，候选 {len(candidates)} 行、{unique_lines} 个不同的行）')
    return 0


def selftest():
    with tempfile.TemporaryDirectory() as directory:
        def git(*arguments):
            subprocess.run(['git', '-C', directory, *arguments], check=True, capture_output=True)

        def write(relative_path, content):
            full_path = os.path.join(directory, relative_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, 'w', encoding='utf-8') as handle:
                handle.write(content)

        def quietly(function, *arguments):
            standard_output = sys.stdout
            with open(os.devnull, 'w') as silent:
                sys.stdout = silent
                try:
                    return function(*arguments)
                finally:
                    sys.stdout = standard_output

        git('init', '-q')
        git('config', 'user.email', 'selftest@example.invalid')
        git('config', 'user.name', 'selftest')
        write('CLAUDE.md', '# 项目\n\n层 0 的负载还只有第一个事务。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('.claude/kb/table-before-move.md', '# 字节表\n\n' + '字段一行。\n' * 20 + '\n## 历史版本\n\n### 2026-08-30：搬家之前就有的条目\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'base')
        os.makedirs(os.path.join(directory, '.claude/kb/layout'), exist_ok=True)
        os.rename(os.path.join(directory, '.claude/kb/table-before-move.md'),
                  os.path.join(directory, '.claude/kb/layout/01-first-txn.md'))
        write('CLAUDE.md', '# 项目\n\n层 0 的负载是两条流。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
```

### 九·四（第 451–579 行，到文件末尾）

```python
              '### 2026-09-02：层 0 扩成两条流\n\n### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'stage')
        failures = []
        entries = new_change_entries(directory, 'HEAD~1', 'HEAD')
        if [heading for _, _, heading in entries] != ['### 2026-09-02：层 0 扩成两条流']:
            failures.append(f'新增变更记录没认对：{entries}')
        segments = changed_segments(read_version(directory, 'HEAD~1', 'CLAUDE.md'), read_version(directory, 'HEAD', 'CLAUDE.md'))
        if not any('只有第一个事务' in old for old, _ in segments):
            failures.append('差异片段里没有被改掉的「只有第一个事务」')
        header = '\t'.join(FACT_TABLE_COLUMNS) + '\n'
        write('facts-good.tsv', header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0\tH1\n')
        write('facts-uncovered.tsv', header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0\t提交 abc\n')
        write('facts-empty.tsv', header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t不存在的词\tH1\n')
        write('facts-new-wording.tsv', header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t两条流\tH1\n')
        write('facts-fake-new.tsv', header + 'F1\t新立：两条流\t层 0 两条流落地\t两条流\tH1\n'
              'F2\t新立：层 0\t层 0 两条流\t层 0\tH1\n')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-fake-new.tsv')) != 6:
            failures.append('冒充新立的事实表没被判红')
        write('facts-conjunction.tsv', header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0&&一次挂载\tH1\n'
              'F2\t只改措辞\t只改措辞\t层 0\tH1\n')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 0:
            failures.append('罩全了的事实表被判没罩全')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-uncovered.tsv')) != 6:
            failures.append('漏了 H1 的事实表没被判红')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-empty.tsv')) != 6:
            failures.append('检索词零命中的事实表没被判红')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-new-wording.tsv')) != 6:
            failures.append('检索词照抄新说法（只在新版本里有）的事实表没被判红')
        conjunction_candidates = build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-conjunction.tsv')))
        if [row[3] for row in conjunction_candidates] != ['.claude/kb/checks-owed.md:1']:
            failures.append(f'&& 收窄或「只改措辞」不出候选没做对：{[row[3] for row in conjunction_candidates]}')
        saved_limit = globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT']
        globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT'] = 0
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 6:
            failures.append('命中行数超上限的事实表没被判红')
        globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT'] = saved_limit
        candidates = build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-good.tsv')))
        locations = [location for _, _, _, location, _ in candidates]
        if '.claude/kb/checks-owed.md:1' not in locations:
            failures.append('还说「层 0 只有一次挂载」的现状句没进候选')
        if any(location.startswith('.claude/kb/checks-owed.md:') and location != '.claude/kb/checks-owed.md:1'
               for location in locations):
            failures.append('历史版本节里的句子进了候选')
        table_path = os.path.join(directory, 'candidates.tsv')
        with open(table_path, 'w', encoding='utf-8') as handle:
            write_candidates(candidates, handle)
        complete = ('## 逐行判定\n| 组 | 载体 | 判定 | 改后的句子或理由 |\n|---|---|---|---|\n'
                    + ''.join(f'| {group} | {location} | 要改 | 改成层 0 的负载是两条流 |\n'
                              for group, _, _, location, _ in candidates))
        write('complete.md', complete)
        write('missing.md', complete.rstrip('\n').rsplit('\n', 1)[0] + '\n')
        write('no-reason.md', complete.replace('| 改成层 0 的负载是两条流 |', '| 不改 |'))
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')]) != 0:
            failures.append('判全了的报告被判漏')
        if quietly(check_reports, table_path, [os.path.join(directory, 'missing.md')]) != 3:
            failures.append('少判一行的报告没被判红')
        if quietly(check_reports, table_path, [os.path.join(directory, 'no-reason.md')]) != 3:
            failures.append('理由不到 8 个字的报告没被判红')
    if failures:
        for failure in failures:
            print(f'  ✗ 自证失败：{failure}')  # gate-lint:detail
        print('  ✗ stale-candidates 自证没过')  # gate-lint:summary
        print('     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、'
              'build_candidates、check_reports 哪一支被改了')
        return 5
    print('  ✓ stale-candidates 自证通过：新变更记录与差异片段认对，事实表漏 H 编号与检索词零命中都判红，'
          '检索词照抄新说法与太宽都判红，冒充新立判红，&& 收窄与只改措辞不出候选对，旧说法进候选、历史版本节不进，报告漏判一行与理由太短都判红（14 格）')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--base')
    parser.add_argument('--target')
    parser.add_argument('--out')
    parser.add_argument('--changes', action='store_true')
    parser.add_argument('--check-facts', metavar='事实表')
    parser.add_argument('--facts', metavar='事实表')
    parser.add_argument('--check-report', nargs='+', metavar='文件')
    parser.add_argument('--groups')
    parser.add_argument('--benchmark', action='store_true')
    parser.add_argument('--selftest', action='store_true')
    arguments = parser.parse_args()
    if arguments.selftest:
        return selftest()
    try:
        repository_root = run_git(['rev-parse', '--show-toplevel'], os.getcwd()).strip()
        if arguments.benchmark:
            return benchmark(repository_root)
        if arguments.check_report:
            if len(arguments.check_report) < 2:
                print('  ✗ --check-report 要候选表与至少一份报告')
                print('     → 怎么办：stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]')
                return 2
            return check_reports(arguments.check_report[0], arguments.check_report[1:], parse_group_ranges(arguments.groups))
        if not arguments.base:
            print('  ✗ 缺 --base')
            print('     → 怎么办：给阶段开始之前的提交，例：--base b1c8cef~1；用法见本脚本文件头')
            return 2
        if arguments.check_facts:
            return check_facts(repository_root, arguments.base, arguments.target, arguments.check_facts)
        if not arguments.out:
            print('  ✗ 缺 --out')
            print('     → 怎么办：--changes 与 --facts 都要 --out 指一个新文件（排他新建）')
            return 2
        if not arguments.changes and not arguments.facts:
            print('  ✗ 缺 --changes 或 --facts')
            print('     → 怎么办：先 --changes 出变更清单、写事实表、--check-facts 过了，再 --facts 出候选表')
            return 2
        with open(arguments.out, 'x', encoding='utf-8') as output:
            if arguments.changes:
                write_changes(repository_root, arguments.base, arguments.target, output)
                entry_total = len(new_change_entries(repository_root, arguments.base, arguments.target))
                print(f'  ✓ 变更清单 {arguments.out}：{entry_total} 条变更记录')
                return 0
            candidates = build_candidates(repository_root, arguments.target, read_fact_table(arguments.facts))
            write_candidates(candidates, output)
        print(f'  ✓ 候选表 {arguments.out}：{len(candidates)} 行（{len({row[3] for row in candidates})} 个不同的行）')
        return 0
    except (RuntimeError, ValueError) as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：确认 --base / --target 是这个仓里存在的提交、事实表格式照文件头')
        return 2


if __name__ == '__main__':
    sys.exit(main())
```
