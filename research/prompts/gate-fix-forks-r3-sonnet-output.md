# 门禁修复留下的岔路 第三轮 云端正推（Sonnet）报告

分工：T1、T3、T8 的暂定形态与条款、登记表逐字对得上吗；第二轮判决里报出的数逐个现核。
不造反例攻代码。

开工快照核对（对 105 份文件跑 sha256sum -c）：

```
$ cd /home/fy5090/code/singlefs && sha256sum -c research/prompts/gate-fix-forks-r3-snapshot/sha256sums.txt --quiet
（无输出，退出码 0）
```

现查时刻：本机 UTC 时间见各命令 `date -u` 输出；本报告下文命令按 nice -n 19 跑，行号均为现查所得（`grep -n` / `awk 'NR==N'`），不从背景材料数。

## 问题一：T1、T3、T8 的暂定形态与条款、登记表逐字对得上吗

### T1 —— 75 号 ⑤ 后一半 / 10 号并入 75 号 ⑨

T1 暂定形态（背景材料正文第 25 行）：「75 号 ⑤ 后一半：这一批把实验的标题状态改成「已跑」（状态段恰好是这两个字），决策正文（历史版本之前）用 `E<号>（` 引了它的每条决策都要回看——决策文件在这一批里，或者影响表里那条决策的一行改过；10 号「已跑实验要有决策引用」那一段撤掉、并进 75 号 ⑨；75 号判共用脚本的退出码」。

**现读 75 号自己的头部**（`.claude/gate.d/75-decision-experiment-links.sh:16-18`，`grep -n` 现取）：

```
16:      要有「## 回看决策」一节（同样的表，或一行「不涉及决策：理由」）；这次改动里新写成「改了」的行，
17:      它指的决策文件要在这次改动里。这次改动把实验的标题状态改成「已跑」（状态段恰好是这两个字）时，
18:      决策正文（历史版本之前）用 `E<号>（` 引了它的每条决策都要回看：决策文件在这次改动里，或者表里那条决策的一行改过。
```

这与 T1 暂定形态逐字一致（仅「这一批」↔「这次改动」、「影响表」↔「表」两处同义替换，指同一件事）——**直接命中，形态已经原样落地在 75 号自己的头部**，不是待验证的候选，是既成实现。

**75 号 ⑨（10 号并入处）**，现取 `.claude/gate.d/75-decision-experiment-links.sh:25`：

```
25: #   ⑨ 实验必须对应决策（用户 2026-09-19）：待回填清单之外的实验页，表里至少一行关系是支撑 / 推翻 / 备料；
```

`.claude/gate.d/10-kb-rot.sh:15-17`（`grep -n` 现取）：

```
15: # 实验与决策之间的两件事不在这里判，都归门禁 75 号（三方判决 gate-fix-forks-r1、r2 的 T1）：
16: #   「实验改成已跑、引用它的决策有没有同批回看」归 ⑤——表里改过一行不够，正文引了它的每条决策都要回看；
17: #   「已跑的实验有没有对应的决策」归 ⑨——表里至少一行支撑、推翻或备料，标题写了作废或退役的不判。
```

10 号头部逐字点名了并入去向，与 T1 暂定形态一致。

**对 `.claude/rules/format-evolution.md`「决策正文只写现状，依据写成指针；决策与实验双向登记」**（`.claude/rules/format-evolution.md:22-53`，现取行号，正文与背景材料附录逐字相同）：

- **直接推论的句子**：第 42 行「实验出了新结论（页内历史节或 `experiments-history.md` 记了新条目、改了正文、换了产物）之后，每一行都要重新回看，日期不早于那次变动；写「改了」的，那条决策文件要在同一次改动里」——75 号 ⑤ 里「新写成「改了」的行，它指的决策文件要在这次改动里」一句是这一句的直接搬运（原句怎么写，脚本头部就怎么判）。
- **规则没说到、75 号自己新加的那一截**：格式演进纪律第 41 行只把「双向」限定在**依据段**——「支撑、推翻要写到分项，那条分项的 `**依据**：` 要引回这个实验；反过来，**分项依据里引的每个实验**，实验页里都要有这一行」。75 号 ⑤ 后一半要求扫描的是「决策正文（历史版本之前）用 `E<号>（` 引了它的每条决策」——**不限于依据段**，覆盖了「欠」块、射程段等依据段之外提到 `E<号>（` 的位置。这一截正是 r2 判决攻方打中的缺口（决策在依据段之外提到实验时原规则看不见），**format-evolution.md 现在的文字没有明写「依据段之外的提及也要回看」这一句**——75 号 ⑤ 后一半是对现有条款的收严，不是条款原文能直接读出来的推论。第 42 行的触发条件写的是「实验出了新结论」，枚举了三种形态（历史节记新条目、改了正文、换了产物），**没有把「标题状态改成已跑」明写为第四种触发**；75 号 ⑤ 后一半把「状态段恰好是这两个字」单独立成第四种触发条件，这一句同样是条款文字没有覆盖到、由 75 号自己界定的新判据。
- **没有找到逐字冲突的句子**：第 41 行「双向」的限定范围（依据段）与 75 号 ⑤ 后一半的射程（历史版本之前的全部正文）不是同一个集合，但后者是前者的**超集**而非**矛盾**（没有一处要求「依据段之外的提及不算」），所以判「没覆盖到」而非「冲突」。

**T1「75 号判共用脚本的退出码」一句的精度**：现读 `.claude/gate.d/75-decision-experiment-links.sh:52,54-58`：

```
52:   source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
53:   base="$(gate_diff_base gate)"
54:   changed="$(gate_changed_paths "$base" untracked)" || {
55:     echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
56:     echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
57:     exit 1
58:   }
```

`source`（52 行）与 `gate_changed_paths`（54–58 行）两处调用都判了退出码；第 53 行 `base="$(gate_diff_base gate)"` 本身没有 `|| exit` 兜底。现查 `research/scripts/changed-paths.sh:41-61` 的 `gate_diff_base`：`mode="gate"` 分支三层回退（`GATE_BASE` → `@{upstream}` 的 merge-base → `HEAD`），没有一条路径返回非 0（只有 `mode` 既不是 `gate` 也不是 `head` 时才 `return 2`，而 75 号调用时写死传 `gate`，走不到那一支）。所以「75 号判共用脚本的退出码」这句对 `source` 与 `gate_changed_paths` 两处成立，对 `gate_diff_base` 这一处不成立——不是漏判，是那个调用形态下 `gate_diff_base` 本身不会返回非 0，`base` 这一行没有 `|| exit` 不构成漏洞。这是转述精度上的一处过宽（T1 说「判共用脚本的退出码」没有区分这三处），不构成与条款的冲突。

**什么现象会推翻以上判定**：75 号头部文字与我贴出的三段不一致（改了没同步头部注释）；或者 `format-evolution.md:41-43` 出现一句明确把「依据段之外的提及」也纳入双向检查的范围（届时「没覆盖到」要改判「直接推论」）。

### T3 —— implementation-workflow.md「三步在定义上各取什么」

暂定形态（背景材料正文第 26 行，第二轮改回原句之后）：`.claude/rules/implementation-workflow.md:22`（`grep -n` 现取）：

```
22: **三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。
```

**逐句核对与 `show-me-test.md` 的关系**（`.claude/singlefs-ai-sop/rules/show-me-test.md`，全篇；行号现取）：

| 该段里的句子 | 关系 | 依据（现取行号） |
|---|---|---|
| 「第 1 步不适用……只管 `crates/*/src/`，只改文档和脚本除外」 | **直接推论**，字面引用 | `show-me-test.md:38-39`：「**改了 `crates/*/src/` 就得带测试。** 没有例外……只改文档和脚本除外。」逐字一致（`grep -n` 命中：`38:**改了 \`crates/*/src/\` 就得带测试。**` `39:也没有「下个 patch 补上」。只改文档和脚本除外。`）|
| 「第 3 步照……「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义」 | **直接推论**（域内替换：「规则」→「定义」，「对抗轮」→「三方」，同义） | `show-me-test.md:57-58`：「**这条同样管设计论证**：对抗轮打中的每一条，先在模型里做成一个**必须报非 0** 的世界，再改规则。」两句语序、修饰词逐一对应，唯二处替换（「规则」→「定义」，因为这一段管的是改定义；「对抗轮」→「三方」，三方论证在本工程里就是对抗轮的别名，`.claude/rules/three-way-inference.md:21` 「三方」是这套流程沿用的名字） |
| 「第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方」 | 不是引 `show-me-test.md`，是引同一份文件自己上一段 | `.claude/rules/implementation-workflow.md:20`：「**改它们与改 `crates/` 同规矩**：写完要走一轮三方……门禁 72 号判形式」，同文件内部指代一致 |

**没有找到与 `show-me-test.md` 逐字冲突的句子**：全篇（`门禁是抬地板的，不是筛人的`「判据是测试」「没有测试的 patch 一律不收」「新增的测试必须先证明它会红」「踩过的坑……」「最终判据由项目定」「门禁不许假装通过」「一条改动不同步铺满全仓……」「门禁能证明什么，不能证明什么」）逐节读过，没有一句被这一段的三句话直接否定。

**对 `implementation-workflow.md` 其余各节**（`## 三步，缺一步就不算做完`、`## 代码轮派腿之前记一份开工快照`、`## 复用上一次全量门禁的判定`、`## 提交前必跑 herd7 与 QEMU`、`## 测试与崩溃检测优先多线程`）：

- `## 三步，缺一步就不算做完` 那张表（`implementation-workflow.md:8-12`）里第 1、3 步的「谁在判」栏写的是 `show-me-test` 阶段、54/59/33 号与 `check.sh`——这些判据本身**只覆盖 `crates/` 下的代码改动**，不提及 agent 定义。T3 段落对第 1 步给出「不适用」、对第 3 步给出改用 `show-me-test.md`「这条同样管设计论证」的替代判据，**是在原表没有覆盖到的地方（agent 定义不在 `crates/` 射程内）另立一条判据，不是对原表字面的推翻**，判「没覆盖到，T3 段落补了原表的空白」，不是冲突。
- 与「次序不能倒」（`implementation-workflow.md:14`）不冲突：T3 段落没有改变三步的次序。

**对门禁 71、72 号的判据——71 号现查不到对象**：

```
$ cd /home/fy5090/code/singlefs && find . -iname '71-*.sh' 2>/dev/null
（零命中）
$ git ls-tree -r HEAD --name-only -- .claude/gate.d/ | grep '^\.claude/gate\.d/71-'
（零命中，exit 1）
```

`.claude/agent-common.md:4` 与 `.claude/rules/implementation-workflow.md:18` 都写着「门禁 71 号判这一条」，但仓里今天没有任何一份 `71-*.sh`。现查提交历史：`.claude/gate.d/71-agent-def-flow-only.sh` 由提交 `dddbabf`（2026-09-18）新增，由提交 `8186d5b`（2026-09-21，早于这一轮背景材料日期 2026-09-24）删除；那次提交信息第八节写明原因：「0.0.56 把 9 个本地阶段的判据收归上游（……71 agent 定义纪律……），……对应的本地阶段……一并删掉，判据一条没丢」。现读吸收它的位置，`.claude/singlefs-ai-sop/scripts/gate.sh:310-314`：

```
310: PROJECT_RULES_DIR=""
311: [[ -d "$ROOT/.claude/rules" ]] && PROJECT_RULES_DIR="$ROOT/.claude/rules"
312: [[ -z "$PROJECT_RULES_DIR" && -d "$ROOT/.claude/agents" ]] && PROJECT_RULES_DIR="$ROOT/.claude/agents"
313: if [[ -n "$PROJECT_RULES_DIR" ]]; then
314:   run_rules_lint "规则纪律（项目本地）" "$PROJECT_RULES_DIR" CLAUDE.md \
315:     ".claude/agents/*.md" ".claude/agent-common.md" ".claude/main-agent.md" ".claude/skills/*/SKILL.md"
```

71 号原本判的「定义里不留解释尾巴」，现在由上游共享 `gate.sh` 里一个不带编号的阶段「规则纪律（项目本地）」代管（`rules-lint.sh` 扫 `.claude/agent-common.md` 等文件）。**这不是 T3 那一段本身的问题**（那一段没有点名任何门禁编号），而是 `agent-common.md:4`、`implementation-workflow.md:18` 两处「门禁 71 号判这一条」的引用本身已经指向一个不存在的文件——**这两句是悬空引用，早于这一轮就已经腐坏（腐坏发生在 2026-09-21，比这一轮的日期早三天）**。这一条我判「复核不了 T3 段落与 71 号判据是否冲突」，因为 71 号不再是可读的判据；只能确认它被吸收进了哪个不带编号的阶段。72 号存在且现读（`.claude/gate.d/72-agent-def-adversarial-review.sh:1-8`）判的正是 T3 段落里「第 2 步就是……那一轮三方」一句所指的机制，二者一致，没有冲突。

**「第 1 步不适用会不会放过一次本该带会红检查的改定义」**：这句是攻方问句，不在我的射程（不许造反例攻代码）；但从条款文字本身看，`show-me-test.md:38-39` 明写「只改文档和脚本除外」，T3 段落的「不适用」直接落在这条明文豁免里，条款字面上没有为「改 agent 定义」单独设一条例外之外的义务——这一点条款覆盖到了（不是空白）。

**什么现象会推翻以上判定**：`agent-common.md:4` 或 `implementation-workflow.md:18` 被改成不再提「门禁 71 号」（届时悬空引用的判定失效）；或者找到一份仓里现存但文件名不是 `71-*.sh` 的门禁阶段自称「71 号」——已现查排除：`grep -rn '71 号' .claude/gate.d/*.sh .claude/singlefs-ai-sop/scripts/*.sh` 零命中。

### T8 —— `gen-decision-items.py` 的 `clip`（丙a + 登记词不剥 + 半截整个去掉）

暂定形态（背景材料正文第 31 行）：「`clip`：只在真截断过时剥尾；截断点落在一个 ASCII 词中间时半截整个去掉；剥尾的编号按 doc-lint 认编号的形状（整词、带左边界），登记在 kb 的 `<!-- doc-lint:not-numbers … -->` 里的领域词不剥」。

**现读 `.claude/scripts/gen-decision-items.py:25-70`**（`grep -n` 现取行号）：

```
25: def clip(text, n):
31:     t = text[:n]
34:     if len(text) > n and re.match(r'[A-Za-z0-9._-]', text[n]) and re.search(r'[A-Za-z0-9._-]$', t):
35:         t = re.sub(r'[A-Za-z0-9._-]+$', '', t).rstrip()
36:     while t.count('（') > t.count('）'):
37:         t = t[:t.rindex('（')].rstrip()
46:     if t == text:
47:         return t.rstrip()
49:         tail = re.search(r'(?<![A-Za-z0-9._-])([A-Z]+-?\d+(?:\.\d+)*)\s*$', t)
50:         t2 = t[:tail.start()] if tail and tail.group(1) not in not_number_tokens() else t
```

四句逐条对上代码：

| T8 形态里的句子 | 判定 | 对应代码（现取行号） |
|---|---|---|
| 「只在真截断过时剥尾」 | **直接命中** | 46-47 行：`if t == text: return t.rstrip()`——没被真正截断（`t == text`）时直接返回，不进入 48-54 行的剥尾循环 |
| 「截断点落在一个 ASCII 词中间时半截整个去掉」 | **直接命中** | 34-35 行：截断点后一个字符是 `[A-Za-z0-9._-]` 且 `t` 结尾也是同一字符集时，把 `t` 结尾那一整段 `[A-Za-z0-9._-]+` 去掉 |
| 「剥尾的编号按 doc-lint 认编号的形状（整词、带左边界）」 | **直接命中，正则逐字等价** | 49 行 `(?<![A-Za-z0-9._-])([A-Z]+-?\d+(?:\.\d+)*)\s*$` 与 `.claude/singlefs-ai-sop/scripts/doc-lint.sh:821,832`（`grep -n` 现取）`BEGIN { ID = "[A-Z]+-?[0-9]+([.][0-9]+)*"` 与 `if (before ~ /[A-Za-z0-9._-]/) continue` 是同一个形状：`[A-Z]+-?\d+(\.\d+)*` 的编号形状相同，左边界排除集合 `[A-Za-z0-9._-]` 完全相同（Python 用负向后顾断言，awk 用「前一字符属于这个集合就跳过」，语义等价） |
| 「登记在 kb 的 `<!-- doc-lint:not-numbers … -->` 里的领域词不剥」 | **直接命中** | 50 行：`tail.group(1) not in not_number_tokens()` 为假（即命中 not-numbers 表）时 `t2 = t`（不剥） |

**对 `.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 5 条「编号只能做索引，不能做称呼」**（`.claude/singlefs-ai-sop/rules/kb-discipline.md:105-134`，现取行号）：

- 133-134 行：「领域术语（`RAID5`、`SHA256`）跟编号长得一样，机器分不开，得显式摘出去：`<!-- doc-lint:not-numbers RAID5 SHA256 -->`。」——`clip` 里「登记在 kb 的……领域词不剥」一句是这两行的直接推论：not-numbers 标记本来就是为了「领域术语跟编号长得一样，机器分不开」这件事而设，`clip` 把它接上正是消费这条既有机制，不是另立新规则。
- kb-discipline.md 第 5 条**没有一处定义「编号的形状是什么」**（`[A-Z]+-?\d+(\.\d+)*` 这个正则），这一半的权威定义只在 `doc-lint.sh` 里；T8 形态把「编号的形状」这一句准确地挂在「doc-lint 认编号的形状」而不是 kb-discipline.md，**归属是对的**——这条不是「条款没覆盖」，是条款本来就没打算管这一半（规则文字讲的是编号「要不要有登记位、要不要带简称」，不讲字符集）。
- 没有找到与第 5 条逐字冲突的句子。

**一处 T8 四句话没有覆盖到的行为差异（现查代码得出，非条款冲突）**：`not_number_tokens()`（`.claude/scripts/gen-decision-items.py:61-70`）用 `handle.read()` 对整份文件做正则扫描，**没有跳过代码围栏**；而 `doc-lint.sh` 自己读同一种标记时显式跳过围栏（`.claude/singlefs-ai-sop/scripts/doc-lint.sh:795-796`，`grep -n` 现取：`awk '/^[ \t]*```/ { fence = !fence; next } fence { next }'` `/<!-- *doc-lint:not-numbers/ { print }'`，注释写着「读豁免声明也要认围栏：别处所有扫描都认，只有这里不认，于是在 \`\`\`markdown 块里「举例」写一行声明就能把豁免真的打开」）。现验：

```
$ cd /home/fy5090/code/singlefs && python3 -c "
import re
src = open('.claude/scripts/gen-decision-items.py', encoding='utf-8').read()
m = re.search(r'def not_number_tokens.*?(?=\ndef |\Z)', src, re.S)
print('has fence check:', chr(96)*3 in m.group(0))
"
has fence check: False
```

这一点 T8 暂定形态的四句话都没提到——不构成与已写条款的字面冲突（kb-discipline.md 第 5 条本身也没规定「标记要不要跳过围栏」这件事），判「条款没覆盖」；但它是 `clip` 与 `doc-lint.sh` 两套实现之间一个可观测的行为差，如果 kb 文件在代码围栏里举例写过 `<!-- doc-lint:not-numbers X -->`，`clip` 会把 `X` 当成已登记的豁免词不剥，而 `doc-lint.sh` 本身不认那次登记，两边对「`X` 算不算被豁免」的判定会分叉。

**什么现象会推翻以上判定**：`gen-decision-items.py` 的 `clip` 或 `not_number_tokens()` 源码变了（重新 `grep -n` 核对）；或者 `doc-lint.sh` 的 `ID` 正则改了形状而 `clip` 没跟着改（届时「正则逐字等价」这一句失效）。

## 问题二：第二轮判决里报出的数逐个现核

### 数 1——「25 道阶段在 `cd` 之后用 `$(dirname "$0")`」

背景材料正文第 33 行：「主 agent 现跑：全部 143 个样本判得都对」在同一段之前，r2 判决第 951-953 行的原话是：「同一个写法还在另外 23 道阶段里（`.claude/gate.d/` 下 `cd` 到项目根之后还用 `$(dirname "$0")` 的，主 agent 用 awk 按行现数出 25 道）」；r3 正文第 991 行把它写成「另外 23 道阶段……（名单现算：`grep` 那 25 道减 31、52）」。

**现跑我自己写的复现脚本**（先找每份阶段脚本里第一处 `cd "$ROOT"` 或 `cd "${1:-…}"` 形态的行号，再看这一行之后有没有出现 `$(dirname "$0")`）：

```
$ cd /home/fy5090/code/singlefs && nice -n 19 python3 - <<'PY'
import re, glob
files = sorted(glob.glob('.claude/gate.d/*.sh'))
hits = []
for f in files:
    lines = open(f, encoding='utf-8').read().splitlines()
    cd_root_line = None
    for i, l in enumerate(lines, 1):
        if re.search(r'\bcd\s+"\$(\{?ROOT\}?|\{?1:-)', l):
            cd_root_line = i
            break
    if cd_root_line is None:
        continue
    for j in range(cd_root_line, len(lines)):
        if 'dirname "$0"' in lines[j]:
            hits.append((f, cd_root_line, j+1))
            break
print(len(hits))
for h in hits: print(h)
PY
23
('.claude/gate.d/10-kb-rot.sh', 26, 141)
('.claude/gate.d/11-batch-scope.sh', 45, 47)
('.claude/gate.d/20-kb-shape.sh', 19, 247)
('.claude/gate.d/40-results-cited.sh', 14, 90)
('.claude/gate.d/50-rules-manifest.sh', 15, 16)
('.claude/gate.d/54-layer0-replay.sh', 20, 28)
('.claude/gate.d/55-qemu-first-transaction.sh', 27, 98)
('.claude/gate.d/56-crates-adversarial-review.sh', 18, 20)
('.claude/gate.d/59-crates-mutation-replay.sh', 32, 37)
('.claude/gate.d/62-stage-owners.sh', 19, 20)
('.claude/gate.d/63-agent-write-scope.sh', 28, 29)
('.claude/gate.d/68-knowledge-sync.sh', 67, 69)
('.claude/gate.d/69-evidence-in-repo.sh', 50, 57)
('.claude/gate.d/72-agent-def-adversarial-review.sh', 17, 20)
('.claude/gate.d/74-model-differential.sh', 14, 22)
('.claude/gate.d/75-decision-experiment-links.sh', 38, 50)
('.claude/gate.d/88-quoted-result-lines.sh', 27, 32)
('.claude/gate.d/92-layout-checker-sync.sh', 46, 49)
('.claude/gate.d/95-fixture-claims.sh', 48, 49)
('.claude/gate.d/96-experiment-source-discipline.sh', 53, 57)
('.claude/gate.d/97-invariant-field-anchors.sh', 29, 32)
('.claude/gate.d/98-kb-registry.sh', 26, 27)
('.claude/gate.d/99-multipath-registry.sh', 32, 39)
```

**核对**：这一份复跑独立按同一个语义写的扫描（不照抄 r2 的 awk，自己重新设计判据），得出 **23**，与 r3 正文第 991 行「另外 23 道」逐字相符——**数对得上**。

**「25」这个总数**：核实其构成——现读 `.claude/gate.d/31-blocking-verdict.sh`、`52-segment-registry.sh` 的**最后一次提交版本**（`git show HEAD:…`，代表这一批未提交修复之前的原始形态）：

```
$ git show HEAD:.claude/gate.d/31-blocking-verdict.sh | grep -n 'cd \|dirname "\$0"'
58:cd "${1:-$(dirname "$0")/../..}" 2>/dev/null || true
63:GEN="$(cd "$(dirname "$0")/../.." 2>/dev/null && pwd)/.claude/scripts/gen-decision-items.py"
$ git show HEAD:.claude/gate.d/52-segment-registry.sh | grep -n 'cd \|dirname "\$0"'
25:ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
26:cd "$ROOT" 2>/dev/null || exit 2
```

旧版 31 号（第 58 行 `cd "${1:-$(dirname "$0")/../..}"`，第 63 行另一处 `$(dirname "$0")`）确实符合我这份扫描的判据（`cd` 用 `${1:-…}` 形态、之后又出现一次 `$(dirname "$0")`），**若把它计入会使总数变成 23+1=24**。旧版 52 号（第 25 行已经把 `dirname "$0"` 包进 `cd` 之前的 `ROOT=` 赋值里，第 26 行 `cd "$ROOT"` 之后**没有**再出现 `dirname "$0"`）——**不符合我这份判据**；它旧版的问题是随后一行 `python3 research/scripts/check-segment-registry.py --root "$ROOT"`（第 29 行）按 **cwd 相对路径**取脚本，不是「`cd` 之后再调用 `dirname "$0"`」这一种形态，是 52 号自己头部注释里另外点名的一类问题（「脚本按本阶段自己的位置取仓里的那一份，不按 cwd 取」）。**23 + 31（旧版命中）= 24，不是 25**；52 旧版按我这份判据不命中，凑不出「25」。

**结论**：「另外 23 道」这个数**现核对得上**（我自己独立写的扫描逐字复现 23）；「25」这个总数**用我这份判据复核不动**——差 1，差在旧版 52 号是否也该算进「cd 之后用 `dirname "$0")`」这一类：我的判据不算它（它旧版的病灶是 cwd 相对路径，不是 cd 之后再调 dirname），若 r2 的 awk 扫描用了一条更宽的判据（例如把「cd 之后用任何相对路径取自己旁边的文件」都算进来，不限定必须字面出现 `dirname "$0"`），52 旧版就会被算进去，25 = 23 + 31 + 52 才成立。**我没有 r2 那份 awk 脚本的原文，无法确认它的判据是不是这条更宽的版本**——这一句「差在哪」如实记下，不替它补一个我没验证过的判据。

**什么现象会推翻以上判定**：找到 r2 当时实际跑的那份 awk 脚本源码（若已归档在 `research/results/` 或某次提交里），逐字核对判据是否比我这份宽；或者旧版 52 号在别的行（我没搜到的地方）还有一处 `dirname "$0"`，那样它也会命中我这份判据，25 = 23+31+52 就能补齐。

### 数 2——「全部 143 个样本判得都对」

**先数样本总数（纯文件系统枚举，不执行任何阶段脚本）**：

```
$ cd /home/fy5090/code/singlefs && nice -n 19 find .claude/gate.d/fixtures -mindepth 2 -maxdepth 2 -type d \( -name red -o -name green \) -exec test -f {}/expect \; -print 2>/dev/null | wc -l
143
```

**数对得上**：仓里今天确实有 143 份带 `expect` 的 red/green 样本目录，与背景材料正文第 33 行「全部 143 个样本」逐字相符。

**其中属于禁跑的 54、55、57、59、87 号的样本**：

```
$ nice -n 19 find .claude/gate.d/fixtures -mindepth 2 -maxdepth 2 -type d \( -name red -o -name green \) -exec test -f {}/expect \; -print 2>/dev/null | grep -E '/fixtures/(54|55|57|59|87)-'
.claude/gate.d/fixtures/59-crates-mutation-replay.sh/red
.claude/gate.d/fixtures/59-crates-mutation-replay.sh/green
.claude/gate.d/fixtures/57-lkmm.sh/red
.claude/gate.d/fixtures/57-lkmm.sh/green
.claude/gate.d/fixtures/55-qemu-first-transaction.sh/red
.claude/gate.d/fixtures/55-qemu-first-transaction.sh/green
```

6 份（54、87 号没有样本目录）。**这 6 份复核不了**：我被明令不许跑 54、55、57、59、87 号，`.claude/singlefs-ai-sop/scripts/stage-selftest.sh` 会对每一份有 fixtures 的阶段真的执行那个阶段脚本（`bash "$stage" "$work"`），所以不能对这 6 份跑判别力自检。

**剩下 137 份我实测复核**：为了不碰这 5 号阶段，先把仓整份拷到草稿目录（`rsync -a --exclude=.git --exclude=target --exclude=.cargo`），在拷贝里删掉这 5 份阶段脚本与它们的 fixtures，再对着这份拷贝跑 `stage-selftest.sh`（草稿目录 `/tmp/claude-1000/gate-fix-forks-r3-sonnet/selftest-copy/`，不写回仓里任何文件）：

```
$ nice -n 19 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh /tmp/claude-1000/gate-fix-forks-r3-sonnet/selftest-copy/.claude/gate.d
……
  ! 这些阶段还没有判别力样本，这一条**没有验过它们**：
      15-research-build.sh
      47-research-script-selftests.sh
     → 怎么办： 一条永远不红的检查与没有这条检查，在门禁输出里长得一模一样。
                给它配 fixtures/<阶段文件名>/{red,green}/，或者把这笔欠账记进项目的欠检查清单。
  ✓ 有样本的阶段判得都对（137 个样本），2 个阶段仍未自检
```

```
$ grep -c '✓.*判得对' /tmp/claude-1000/gate-fix-forks-r3-sonnet/stage-selftest-filtered.out
137
$ grep -c '✗' /tmp/claude-1000/gate-fix-forks-r3-sonnet/stage-selftest-filtered.out
0
```

143 − 6 = 137，与实测「137 个样本」全部判对（0 处 `✗`）逐字吻合。

**结论**：「143 个样本」这个总数**数对得上**；其中 137 个（全部不属于 54/55/57/59/87 号的）我实测**全部判对**，与「全部 143 个样本判得都对」这句话在我能验的范围内一致；另外 6 个（属于 55、57、59 号）**复核不了**——不是我发现了问题，是这一轮的禁跑条款挡住了这一步，如实记「复核不了」。

**什么现象会推翻以上判定**：`stage-selftest.sh` 换了判据、或 `.claude/gate.d/fixtures/` 目录结构改变（我的 `find` 命令按 `<阶段文件名>/{red,green}/expect` 这个形状枚举，形状变了这条命令会漏数或多数）；或者 55/57/59 号的样本在获准执行之后被发现有判错的（届时「143 个样本判得都对」在我没验的那 6 个里失效）。

### 数 3——「共用脚本自证 14 格，八种弄坏的形态（`LIB_CHANGED_PATHS_BREAK` 各值）各判红」

**先数 `LIB_CHANGED_PATHS_BREAK` 在 `research/scripts/changed-paths.sh` 里出现过的取值**（`grep -noE` 现取）：

```
$ cd /home/fy5090/code/singlefs && nice -n 19 grep -noE 'LIB_CHANGED_PATHS_BREAK[^)]*?[=!]=\s*"?[a-zA-Z-]+"?' research/scripts/changed-paths.sh
48:LIB_CHANGED_PATHS_BREAK:-}" != fallback
52:LIB_CHANGED_PATHS_BREAK:-}" == empty-merge-base
68:LIB_CHANGED_PATHS_BREAK:-}" == quotepath
78:LIB_CHANGED_PATHS_BREAK:-}" != untracked
86:LIB_CHANGED_PATHS_BREAK:-}" == swallow
96:LIB_CHANGED_PATHS_BREAK:-}" == quotepath
103:LIB_CHANGED_PATHS_BREAK:-}" == header-anywhere
114:LIB_CHANGED_PATHS_BREAK:-}" == untracked-lines
119:LIB_CHANGED_PATHS_BREAK:-}" != binary
```

去重后 8 个取值：`fallback`、`empty-merge-base`、`quotepath`、`untracked`、`swallow`、`header-anywhere`、`untracked-lines`、`binary`——与「八种弄坏的形态」数对得上。

**现跑基线（不设 `LIB_CHANGED_PATHS_BREAK`）**：

```
$ nice -n 19 bash research/scripts/changed-paths.sh --selftest 2>&1 | tail -1
  ✓ 共用库 research/scripts/changed-paths.sh 自证通过（14 项：三种来源的中文路径逐字不转义、未跟踪可开关、diff-filter=A、新增行含未跟踪整份且只看给的路径、带空格路径与「++ 」开头的行、不读二进制、路径带制表符退非 0、基准解析不到退非 0、基准回退四档）
```

14 项，与「共用脚本自证 14 格」数对得上。

**逐个设置 8 种 `LIB_CHANGED_PATHS_BREAK` 值现跑，确认各判红**：

```
$ for b in fallback empty-merge-base quotepath untracked swallow header-anywhere untracked-lines binary; do
    LIB_CHANGED_PATHS_BREAK="$b" nice -n 19 bash research/scripts/changed-paths.sh --selftest >out.$b 2>&1
    echo "$b: exit=$?"
  done
fallback: exit=1
empty-merge-base: exit=1
quotepath: exit=1
untracked: exit=1
swallow: exit=1
header-anywhere: exit=1
untracked-lines: exit=1
binary: exit=1
```

8 种全部退出码 1（判红），每种都能在输出里指认到具体哪一格 `check` 失败（例如 `binary` 那组：「新增行：未跟踪的二进制文件不读：期望「0」，实测「1」」）。**数与判定都对得上**：「共用脚本自证 14 格，八种弄坏的形态各判红」逐字核实无误。

**什么现象会推翻以上判定**：`changed-paths.sh` 的 `check` 数或 `LIB_CHANGED_PATHS_BREAK` 取值集合变了（重新 `grep` 会看出来）；或者 8 种里有一种在今天的代码上退出码变成 0（届时这一句就要改判「打中」）。

### 数 4——「十九格 `clip` 用例」（判决原文没有逐格列出，按 T8 一节自己造表现跑）

r2 判决 T8 一节（背景材料附录，`gate-fix-forks-r2-main-verification.md:959` 行原句）只写「十九格手写用例全对」，没有逐格列出用例。按 r3 正文第 63 行给的五类（截断停在登记词之后、登记词里带连字符或小数点、截断落在词中间、截断落在登记词中间、截断后尾巴是真编号而它的简称被截掉）自己设计 19 格，直接调用现读的 `clip()` 源码（不整份 import 模块——该文件顶层有无 `__main__` 守卫的散文代码，import 会连带打印全部决策分项；改成只 `exec` `clip` 与 `not_number_tokens` 两段函数定义，`_NOT_NUMBER_TOKENS` 手动设成 `{"SHA256", "RAID5", "AES-256", "SHA-256", "Q1.1"}` 五个固定值，不读真实 kb，避免结果随 kb 内容漂移）：

```
$ cd /home/fy5090/code/singlefs && nice -n 19 python3 - <<'PY'
import re
src = open('.claude/scripts/gen-decision-items.py', encoding='utf-8').read()
clip_src = re.search(r'^def clip.*?(?=\n_NOT_NUMBER_TOKENS)', src, re.S | re.M).group(0)
nnt_src = re.search(r'^_NOT_NUMBER_TOKENS = None\n\n\ndef not_number_tokens.*', src, re.S | re.M).group(0)
ns = {'re': re}; exec(clip_src, ns); exec(nnt_src, ns)
ns['_NOT_NUMBER_TOKENS'] = {"SHA256", "RAID5", "AES-256", "SHA-256", "Q1.1"}
clip = ns['clip']
# ……19 组 (原文, n) 逐条调用 clip(text, n)
PY
```

| # | 设计的边界情形 | 原文 | n | `clip` 输出 | 判定 |
|---|---|---|---|---|---|
| 1 | 截断停在登记词之后（整词，无连字符） | `校验和算法取 SHA256 之后还有后缀` | 13 | `校验和算法取 SHA256` | 登记词整词保留，符合 |
| 2 | 截断停在登记词之后（带连字符），但截断点其实落在词中间 | `校验和算法取 AES-256 之后还有后缀` | 13 | `校验和算法取` | 半截整个去掉，符合「截断点落在 ASCII 词中间」那一句；not-numbers 保护没轮到（见下方说明） |
| 3 | 同上，另一个带连字符登记词 | `散列函数用 SHA-256 之后还有后缀` | 12 | `散列函数用` | 同 2 |
| 4 | 登记词本身带小数点，截断停在其后 | `参数取 Q1.1 之后还有后缀` | 9 | `参数取 Q1.1` | 登记词整词保留，符合 |
| 5 | 截断落在登记词中间（`SHA256` 后面紧跟自己再重复一次，截断点落在重复串中间） | `校验和算法取 SHA256256` | 15 | `校验和算法取` | 半截去掉，符合 |
| 6 | 同上，`RAID5` | `条带冗余按 RAID5RAID5` | 11 | `条带冗余按` | 半截去掉，符合 |
| 7 | 同上，带连字符登记词 | `散列函数用 AES-256AES` | 14 | `散列函数用` | 半截去掉，符合 |
| 8 | 截断落在真编号后、括注中间（有开括号没闭） | `参照 D22（简称一）以后再看` | 8 | `参照` | 括注回退到 `D22` 后再把裸编号剥掉，避免裸引用，符合 |
| 9 | 截断后尾巴是真编号，简称被截掉（括注更长） | `参照 D22（这是一段很长的简称文字）` | 10 | `参照` | 同 8，符合 |
| 10 | 截断后尾巴是带小数点真编号 `I-9.5`，简称被截掉 | `参照 I-9.5（这是一段简称）` | 9 | `参照` | 符合 |
| 11 | 编号在截断串中**不在末尾**（后面还跟着别的字） | `参照 D22 之后还有后缀文字` | 8 | `参照 D22 之` | **裸引用没剥掉**——见下方说明 |
| 12 | 没有真正截断（`n` 远大于原文长度） | `很短的一句话` | 100 | `很短的一句话` | 原样返回，符合「只在真截断过时剥尾」 |
| 13 | 截断点落在纯中文字里 | `这是一段很长的中文句子用来测试截断` | 10 | `这是一段很长的中文句` | 中文字符不触发任何 ASCII 剥尾逻辑，符合 |
| 14 | 截断落在括注中间（无右括号） | `参照 D22（简称还没写完呢` | 10 | `参照` | 符合 |
| 15 | 截断落在登记词之后、紧跟顿号 | `算法用 SHA256、然后继续` | 11 | `算法用 SHA256` | 顿号先被「分隔符」正则剥掉，再判定末尾是登记词、保留，符合 |
| 16 | 截断落在未登记的编号形状词之后、且编号**不在截断串末尾** | `参照 T99 之后还有文字` | 8 | `参照 T99 之` | 同 11，裸引用没剥掉 |
| 17 | 编号前一个字符是字母（不应算独立编号），且截断点也不在串末尾 | `使用 xSHA256 之后还有文字` | 12 | `使用 xSHA256 之` | 没能测出左边界排除是否生效（截断点没落在串末尾，尾部正则从未被触发）——见下方说明 |
| 18 | 截断落在两位小数编号之后（`I-9.55`） | `参照 I-9.55 之后还有文字` | 10 | `参照` | 符合 |
| 19 | 截断停在登记词之后、词右侧紧跟斜杠与另一个词 | `算法用 RAID5/RAID6 之后还有` | 11 | `算法用 RAID5` | 斜杠先被分隔符正则剥掉、多出来的半个 `R` 先被「词中间」逻辑剥掉，再判定末尾是登记词、保留，符合 |

**三点说明（现查代码得出，不是造出的反例攻代码，是对既有四句判据覆盖范围的核实）**：

1. **第 2、3、5、6、7 格证实「登记词不剥」这条保护只在编号整词落在截断串末尾时才轮得到**：`clip()`（`.claude/scripts/gen-decision-items.py:34-35`）里「截断点落在 ASCII 词中间」的整段剥除逻辑**先于**尾部编号识别（`:48-54`）执行，且不区分这个词是不是登记过的领域词——一旦截断把登记词本身切成两半，那半截整个被丢掉，not-numbers 表根本没被查过。这与「登记在 kb 的 `<!-- doc-lint:not-numbers … -->` 里的领域词不剥」这句话不矛盾（那句话字面上只承诺「不剥掉一个完整的登记词」，没有承诺「截断到一半的登记词也保留」），但值得记下来：**登记词只在恰好卡在词尾时受保护，卡在词中间时不受保护**，与不是登记词的普通编号（`D22`、`T99`）落在词中间时的下场完全一样（cases 5、6、7 与 cases 8/14 的「半截去掉」是同一条代码路径）。
2. **第 11、16 格发现同一条判据没覆盖到的边界**：`clip()` 的尾部编号识别正则 `(?<![A-Za-z0-9._-])([A-Z]+-?\d+(?:\.\d+)*)\s*$`（`:49` 行）**锚定在字符串末尾**（`\s*$`）。当截断点比编号本身多留了几个字符（例如「之」这一个字），编号就不再位于截断串的末尾，这条正则从不匹配，于是「截断还可能……留下一个裸引用……把这种尾巴一并去掉」这句设计意图（`.claude/scripts/gen-decision-items.py:38-40` 注释）**在这一类截断点上不生效**：`参照 D22 之`、`参照 T99 之` 这两格输出里都留着一个没有简称跟随的裸编号。T8 暂定形态的四句话字面上没有断言「编号不在截断串末尾时也会被剥掉」，所以这不是与已写条款的字面冲突，是条款覆盖范围之外的一个真实缺口——如果 doc-lint 判定这类裸引用会红，这两格的产物今天会被判红（我没有另外验证 doc-lint 对这两句具体文本的判定，只验证了 `clip()` 自己的行为）。
3. **第 17 格没能测出左边界排除是否生效**：因为截断点同样没有落在字符串末尾（案例设计的瑕疵，尾部正则从未被触发，跟第 11、16 格是同一条路径），**这一格不能用来判断「`xSHA256` 这种编号前一个字符是字母时会不会被误剥」**——这一条我没有验证到，如实记「没测出来」，不写成「符合」或「打中」。

**什么现象会推翻以上判定**：`clip()` 源码变了（重新用同一段抽取方式现跑一遍）；或者仓里的 `doc-lint.sh` 对「`D22` 后面紧跟别的字、没有简称」这种形态**不**判红（那样第 11、16 格发现的「留下裸引用」就不构成风险，只是无害的边界）。

### 数 5——「64 号三条判据各关一条红样本都判错」

**先确认 64 号今天确有三条判据**（`.claude/gate.d/64-change-range-single-source.sh:5,9,15`，`grep -n` 现取）：

```
5: #   ① 算 diff 基准只许在 research/scripts/changed-paths.sh 里：代码行里出现 merge-base（`--is-ancestor` 核祖先除外）、
9: #   ② 列路径的 git 调用（--name-only、--name-status、ls-files）要带 -c core.quotepath=false（键不分大小写，值必须是 false）：
15: #   ③ 调共用取法（gate_changed_paths、gate_added_lines）要判退出码：同一行带 `||`，或放在 if / while 里。
```

确认「三条」。**背景材料里这句话出现过两次，数不一样**：r2 判决 T5 一节原句（`gate-fix-forks-r2-main-verification.md:939`，即附录里冻结整段抄入的那一份）写「三条判据各关一条红样本都判错」；r3 正文自己第三节的表（`_gate-fix-forks-r3-background.md:845`）复述同一件事时写成「**两条**判据各关一条红样本都判错」。与今天脚本头部的「三条」对照，**845 行是数错的那一处**（大概率是复述时漏了后来新增的第③条，第③条是 r2 判决里「64 号加第③条盯住以后的调用」那一步才补上的，845 行这张表可能写于那一步之前没有回改）。问题里问的「64 号三条判据」与 939 行、与今天代码一致，我按「三条」现核。

**现跑验证「各关一条红样本都判错」**：把 `64-change-range-single-source.sh` 拷到草稿目录，分别只屏蔽三条判据里的一条（把对应的 `*_hits.append(...)` 换成 `pass`，其余代码不动），逐个对着仓里现成的红样本（`.claude/gate.d/fixtures/64-change-range-single-source.sh/red/`）跑，按 `stage-selftest.sh` 自己的判法（退出码要对、`expect` 里每条 `want=` 都要在输出里找到）现判：

```
$ cd /home/fy5090/code/singlefs
$ EXPECT=.claude/gate.d/fixtures/64-change-range-single-source.sh/red/expect
$ for variant in baseline disable-1 disable-2 disable-3; do
    work=$(mktemp -d)
    cp -a .claude/gate.d/fixtures/64-change-range-single-source.sh/red/. "$work/"
    out=$(cd "$work" && nice -n 19 bash /tmp/claude-1000/gate-fix-forks-r3-sonnet/64-check/64-$variant.sh "$work" 2>&1)
    got=$?
    want_exit="$(sed -n 's/^exit=//p' "$EXPECT")"
    ok=1; [[ "$got" == "$want_exit" ]] || ok=0
    missing=()
    while IFS= read -r w; do [[ -z "$w" ]] && continue
      grep -qF -- "$w" <<<"$out" || { ok=0; missing+=("$w"); }
    done < <(sed -n 's/^want=//p' "$EXPECT")
    echo "$variant: exit=$got(期望$want_exit) 判定=$([[ $ok == 1 ]] && echo 判对 || echo 判错) 缺失=${#missing[@]}"
    rm -rf "$work"
  done
baseline: exit=1(期望1) 判定=判对 缺失=0
disable-1: exit=1(期望1) 判定=判错 缺失=5
disable-2: exit=1(期望1) 判定=判错 缺失=7
disable-3: exit=1(期望1) 判定=判错 缺失=2
```

三条判据（① 算基准、② quotepath、③ 判退出码）逐一屏蔽，**每一次都让同一份红样本判错**（退出码仍是 1，但缺失对应那条判据该报的 `want=` 明细，按 `stage-selftest.sh` 自己的判法算作「样本判错」）：屏蔽 ① 缺 5 条、屏蔽 ② 缺 7 条、屏蔽 ③ 缺 2 条。**「三条判据各关一条红样本都判错」这句话现核成立**——只改了我的草稿拷贝（`/tmp/claude-1000/gate-fix-forks-r3-sonnet/64-check/`），没有改仓里的 `.claude/gate.d/64-change-range-single-source.sh`。

**结论**：r3 正文第 845 行写的「两条」与今天的代码、与 r2 判决自己 939 行的原句都对不上，是这份材料内部自相矛盾的一处；问题里问的「三条」现核成立。

**什么现象会推翻以上判定**：64 号脚本改了判据结构（三条判据的检测逻辑不再是各自独立的 `base_hits`/`list_hits`/`status_hits` 三个列表）；或者这份红样本改了，不再同时覆盖三条判据各自的违规写法。

### 数 6——「今天写了备料的五页（E105、E152、E154、E156、E69）备料都点名了决策」

这句话有两种可能的读法，我把两种都现查了：

**读法甲——五页正文里加粗的「`**备料**：`」段落是否各点名一条决策**（`grep -n '^\*\*备料\*\*：'` 现取）：

```
.claude/kb/experiments/105-extent叶改码3的代价.md:5: **备料**：C113（…） 定案第五版对立臂 B 的价钱……D21（权威态与派生态的分界） 已定项 10 ……
.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md:7: **备料**：不支撑任何一条决策。要用它的时候是 D23（journal 的角色与格式） 那一侧……D17（实现分层与第三方管道） 已定项 5 ……
.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:6: **备料**：……等 D3（空间分配） 已定项 12 与 D16（发布语义） 已定项 1 按岔路定案时引用……
.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:7: **备料**：……等 D28（挂载期承诺量） 已定项 1 与 D16（发布语义） 已定项 1 按岔路定案时引用本实验。
.claude/kb/experiments/69-反向索引取权威态的增量维护代价.md:5,7: **备料**：D21（权威态与派生态的分界） 已定项 1 曾由 E69……（第 5 行）；**备料**：E69……D21（权威态与派生态的分界） 已定项 1 后来按别的依据定成……（第 7 行）
```

五页每一页至少一处「`**备料**：`」段落都点名了具体的决策分项（`D<n>（简称） 已定项 k` 或 `D<n>（简称）`）——**这一读法下，数核对得上**。

**读法乙——五页「### 影响的决策」表里，`关系` 列写「备料」的那些行是否点名了决策**（这是 `.claude/gate.d/75-decision-experiment-links.sh` ⑨ 真正检查的那张表，`grep -n` 现取表内容）：

| 页 | 表里 `关系` 列出现过「备料」吗 |
|---|---|
| E105 | 是（`D21（权威态与派生态的分界） \| 备料 \| …`，`105-extent叶改码3的代价.md:66`） |
| E152 | 是（两行，`152-按里程碑对比六家文件系统的文件性能.md:295-296`） |
| E154 | 是（两行，`154-两道闸串-重判与回收时点的代价.md:190-191`） |
| E156 | 是（两行，`156-alloc-basis四条岔路的代价数.md:76-77`） |
| **E69** | **否**——`69-反向索引取权威态的增量维护代价.md:153-155` 三行的「关系」列都写「不影响」，一处「备料」都没有 |

这一读法下，**五页里有一页（E69）的表格「关系」列没有「备料」**，与「备料都点名了决策」字面对不上——不是「点名了决策，点错了」，是这张表压根没有「备料」这个值。

**判定**：两种读法我都验了。加粗散文段落（读法甲）逐页确认，五页全部成立；「影响的决策」表格关系列（读法乙，也是 75 号 ⑨ 实际检查的对象）只有四页成立，E69 的三行关系全是「不影响」。**我不替这句话选哪种读法是作者原意**，如实并列两个结果——差在「备料」到底指散文段落还是表格枚举值，材料本身没有写清楚。

**什么现象会推翻以上判定**：`E69` 的「影响的决策」表被改成含一行「备料」（那样读法乙也会成立）；或者读法甲里五段「`**备料**：`」文字被改写掉不再点名决策（那样读法甲失效）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 · 与 format-evolution.md「决策……双向登记」 | 形态大半是对既有条款的推论，一句是条款没覆盖到的新收严 | 75 号 ⑤ 后一半已原样落地在脚本头部；条款第 41 行的「双向」限定在依据段，⑤ 后一半扫描依据段之外的提及，是收严不是条款原文能读出来的推论 |
| T1 · 与 75 号自己的头部 | 直接命中 | `.claude/gate.d/75-decision-experiment-links.sh:16-18` 与暂定形态逐字一致 |
| T1 · 10 号并入 75 号 ⑨ | 直接命中 | 10 号与 75 号头部互相点名，与暂定形态一致 |
| T1 · 「75 号判共用脚本的退出码」 | 转述精度过宽 | `source`、`gate_changed_paths` 两处判了退出码，`gate_diff_base` 那一行没有 `\|\| exit`，但它的 `gate` 分支本身不会返回非 0 |
| T3 · 与 show-me-test.md | 两句都是直接推论，没找到逐字冲突 | 「第 1 步不适用」「第 3 步照……」分别对应 `:38-39`、`:57-58`，域内替换但语义一致 |
| T3 · 与 implementation-workflow.md 其余各节 | 没覆盖到，不是冲突 | 「三步」表的第 1、3 步只谈 `crates/`，T3 段落在表没覆盖到的地方另立判据 |
| T3 · 与门禁 71、72 号的判据 | 71 号复核不了（文件已不存在），72 号一致 | 71 号 2026-09-21 被上游 0.0.56 吸收进不带编号的「规则纪律（项目本地）」阶段，`agent-common.md:4`、`implementation-workflow.md:18` 两处引用已悬空 |
| T8 · 与 `clip()` 源码 | 四句全部直接命中 | 只在真截断、词中间半截去掉、正则与 doc-lint 同形、登记词不剥，逐句核对代码行都对得上 |
| T8 · 与 kb-discipline.md 第 5 条 | 直接推论（领域词摘出去），字符集这一半条款本来没管 | not-numbers 标记概念直接来自 `:133-134`；编号形状的正则只在 doc-lint.sh 里，条款没打算定义它 |
| T8 · 一处条款没覆盖到 | `not_number_tokens()` 不跳过代码围栏，`doc-lint.sh` 自己跳过 | 现验 `has fence check: False`；kb-discipline 第 5 条本身没规定要不要跳过围栏 |
| 数 1：25 道阶段 | 「23」核对得上，「25」总数复核不动 | 我的独立扫描精确复现 23；25=23+31+52 需要 52 旧版按更宽判据计入，我手上没有 r2 的原始 awk 脚本核对那条判据 |
| 数 2：143 个样本全对 | 总数核对得上；137 个我实测全对，6 个复核不了 | `find` 数出 143；`stage-selftest.sh` 在剔除 54/55/57/59/87 的拷贝上跑出 137/137 全绿；55/57/59 的 6 个样本按禁跑条款没有跑 |
| 数 3：共用脚本自证 14 格、八种弄坏形态各判红 | 全部现核成立 | 基线 14 项全过，8 个 `LIB_CHANGED_PATHS_BREAK` 值逐一现跑全部退出码 1 |
| 数 4：十九格 clip 用例 | 自己造表现跑，19 格里 16 格符合四句判据，3 格（11、16、17）发现条款没覆盖到的边界 | 编号不在截断串末尾时不受尾部剥除逻辑保护，留下裸引用；第 17 格没能测出左边界排除是否生效 |
| 数 5：64 号三条判据各关一条都判错 | 现核成立；同时发现材料内部自相矛盾 | 三次单独屏蔽判据，红样本各缺 5/7/2 条 `want=`；r3 正文 845 行写「两条」，与 939 行及今天代码的「三条」对不上 |
| 数 6：五页备料都点名了决策 | 两种读法都验了，一真一部分假 | 加粗散文段落「`**备料**：`」五页全成立；「影响的决策」表格关系列 E69 三行全是「不影响」，没有「备料」 |

## 没做什么

- 不判 T4、T5、T6、T7（分给别的腿）；不造反例攻代码（派发提示明令禁止）。
- T3 里「第 1 步不适用会不会放过一次本该带会红检查的改定义」这句攻方问句不在我的射程，只做了字面覆盖核对，没有去构造具体反例。
- 143 个样本里属于 54、55、57、59、87 号的 6 个（55、57、59 各 2 个）没有跑：这一轮明令不许跑这五号阶段，连判别力样本也没碰；这 6 个「复核不了」，如实记，不当作已验证。
- 「25 道」总数背后 r2 当时用的那份 awk 脚本原文没有找到，无法核实它的判据是不是比我这份宽（能不能把 52 号旧版也算进去），只报了我自己独立扫描的口径与结果。
- T8 第 17 格的用例设计有缺陷（截断点没有落在字符串末尾），没能测出「编号前一个字符是字母时会不会被误剥」这一条，如实记「没测出来」。
- 没有验证「五页备料都点名了决策」这句话原本想说哪种读法；两种读法的现查结果都写了，选哪种由主 agent 判。
- 没有跑 `gate.sh` 全量，也没有编译 Rust；没有改动仓里任何文件（草稿产物全部落在 `/tmp/claude-1000/gate-fix-forks-r3-sonnet/`）。
- `.claude/gate.d/stage-owners.tsv` 里没有登记给云端腿的阶段，没有跑收尾门禁。

