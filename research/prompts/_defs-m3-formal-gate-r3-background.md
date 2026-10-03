# 第五道重型门禁写进规则与定义：定义三方第三轮正文（2026-10-01）

<!-- doc-lint:not-numbers G9 G10 G11 J1 J2 J8 Y1 Y2 Y3 R1 R2 R3 R4 L1 L2 L3 L4 -->

## 一、这一轮要判什么

第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停）。第二轮判决 `research/prompts/defs-m3-formal-gate-r2-main-verification.md` 整份进材料；它第二节采的改法 G9、G10、G11、J8 与 E4 几处已由工具实现员 3-12 落进文件（报告 `/tmp/claude-1000/m3-formal-defs-fix-3-12/report.md`，改前改后原文逐处列了），被攻过零轮。这一轮只攻这些改后的字面与代码，不重复前两轮的角度（第一轮：谁跑、复用、并行与内存、清单一致、规则写法；第二轮：判法脚本干活与只读的分界、F1 与 F2、包装结局、写实句与清单）。第三轮里新冒出来的改法不再为它开一轮，写进判决的交用户表、标「零轮」。

被判的文件（相对第二轮开工快照的改动，3-12 报告逐处列了；同文件里别的会话没提交的改动不判）：`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/main-agent.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`research/scripts/verus-proof-check.py`、`research/scripts/e165-pin-red-examples.py`、`research/scripts/admission.py`（只注释）、`.claude/gate.d/checker-tier-formal-verification.sh`（只文件头）。开工快照 `research/prompts/defs-m3-formal-gate-r3-snapshot/sha256sums.txt`。

| 格 | 被攻的 | 问题 |
|---|---|---|
| L1 | G9：先认出是两份判法脚本、再一律算重型，只放字面写明的只读用法 | 「只读用法」的名单里有没有一种其实干活；认出判法脚本的条件（名字、位置、PYTHONPATH、符号链接、`python -c` 点名）会不会误拒别的研究脚本（名字相近、同目录） |
| L2 | G10：`--tool-versions` 哈希安装目录顶层全部文件、`VERUS_Z3_PATH` 分未设与空串、裸名照 PATH 找 | 顶层之外（子目录）有没有 Verus 会读的；整体摘要里混进了会变的东西（日志、缓存、时间戳）没有——那样永远复用不上 |
| L3 | G11：一格里每条 ✗ 都是没跑成才记没跑成；判法混合格另打「按红记」；带出包装的 ✗ 与 →；252 两种成因 | 崩溃验证员、门禁分诊照改后的字面，在「全是没跑成」「混合」「全是真红」三种格上各记什么、下一步做什么，有没有一种走不通；`gate.sh` 汇总里这一道退 1 时的说法与定义对得上吗 |
| L4 | 前两轮「已定」的几格撑不撑得住；第二轮记账的几笔（J2、QEMU 两处、第 1 步停下的频率）够不够；第二轮报出的数能不能核 | 照 `.claude/rules/three-way-inference.md` 第三轮的分工 |

**共用问句**：照改后的字面干活，哪一步会做错或做不了；举出具体的派发情形或命令。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

- 3-12 跑过的：攻方第二轮 `e1-hook-cases.py`（J1 那 10 种写法两个身份 20 条全判拒、只读 8 条全放，输出与攻方 `e1-G9-output.txt` 逐行相同）、`e2-reuse.sh`（Y1–Y3 不复用、对照照旧）、`e3-mixed.py`（混合格多出「按红记」一行）；`--selftest`、lint、真 Verus 都没跑（归提交时）。
- 第二轮两条腿的模型 `research/prompts/defs-m3-formal-gate-r2-opus-model/`、`…-sonnet-model/` 可以照用。
- `crates/` 这一轮不判。

## 三、条款（材料员整段抄进附录）

- 第二轮判决整份；
- 被判文件里 3-12 改过的段落（以 3-12 报告的改前改后原文为准，腿到主仓现读）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | L3、L4 | 复核第二轮判决：采的改法是不是同样打中了被留下的形态；照改后的字面在三种格上走一遍崩溃验证员与门禁分诊 |
| 云端攻方（Opus） | L1、L2、L3 | 造写法与安装状态攻改后的代码：喂钩子只喂 JSON、看退出码；L2 在草稿目录的仓拷贝与安装拷贝上跑 `--tool-versions` 与 `admission.py` 的判定函数，不跑门禁 |

## 五、交付

- 腿的报告写 `research/prompts/defs-m3-formal-gate-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/defs-m3-formal-gate-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`（复跑挂钟写在 `rerun.sh` 文件头，超过 20 分钟的写明哪一步慢）；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引文件写那份文件自己的行号，行号去被引文件里现查，不从背景材料里数。
- 不跑重型测试，不跑 `--selftest`；只停自己起的进程。

### 小节清单：`.claude/kb/experiments/04-范围重建vs逐项修改的交叉点.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E4 范围重建 vs 逐项修改的交叉点 —— 部分已跑（写放大那一半由 E30（交叉点的写放大那一半）还清；耗时那一半仍欠 QEMU 与两条实现） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E4 标题之下、第一个下级标题之前的正文：第 2-25 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

**出处 `.claude/kb/experiments/04-范围重建vs逐项修改的交叉点.md:1-25`（整段抄，未转述）**

```markdown
## E4 范围重建 vs 逐项修改的交叉点 —— 部分已跑（写放大那一半由 E30（交叉点的写放大那一半）还清；耗时那一半仍欠 QEMU 与两条实现）

**产物**：写放大那一半的原始输出在 `e30-range-rebuild-2026-08-29.out`，
复跑 `cargo run --release --bin e30_range_rebuild`（正文在 E30（交叉点的写放大那一半））。
⚠️ **耗时那一半没有产物，因为它一次都没跑过**——前置是 QEMU 介质模拟与两条实现。


**背景**：无界操作两条实现的切换判据，见 [decisions.md](../decisions.md) D8（核心索引结构）。
现在写在文档里的「机械盘约 1%、SSD 约 10%」是**推理，不是实测**，必须量出来。

**自变量**：受影响比例 K/N（0.1% / 1% / 5% / 10% / 30% / 100%）×
介质（QEMU 模拟旋转盘 / SSD）× key 连续与否（连续范围 / 随机散布）。

**测什么**：完成同一操作的耗时与写放大，两条实现各跑一遍。

**判据**：找出每种「介质 × 连续性」组合下两条曲线的交叉点，
写成一个可计算的式子放进代码，不是放进注释。

**怎样才算失败**：
- 若「连续」和「随机散布」两组的交叉点没有明显差异，
  说明范围重建其实没走到整节点丢弃那条路径，**实现有问题，整轮作废**。
  这是E4（范围重建 vs 逐项修改的交叉点）的判别力所在——两组必须分得开。
- 若两条曲线在全区间不相交，说明有一条实现从头到尾都更差，
  那就别做分支，直接删掉差的那条（`.claude/rules/fs-design.md`：判据写不出来就只做一条）。

```

**出处 `.claude/kb/experiments/04-范围重建vs逐项修改的交叉点.md:26-31`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 5 | 备料 | 2026-09-20 不受影响：D8（核心索引结构） 已定项 5 逐字写「机械盘约 1%、SSD 约 10%」是推理不是实测、要靠 E4（范围重建 vs 逐项修改的交叉点） 量，而耗时那一半欠 QEMU 与两条实现、一次没跑；写放大那一半由 E30（交叉点的写放大那一半） 还清 |

```

**出处 `research/prompts/defs-m3-formal-gate-r2-main-verification.md:1-30`（整段抄，未转述）**

```markdown
# 第五道重型门禁写进规则与定义：定义三方第二轮判决（2026-10-01）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 E1 E2 E3 E4 F1 F2 G9 G10 G11 J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 J12 Y1 Y2 Y3 V1 V2 V3 V4 V5 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 被判的 hunk | `research/prompts/_defs-m3-formal-gate-r2-body.md`、`_defs-m3-formal-gate-r2-background.md`、`_defs-m3-formal-gate-r2-diff.md`（54 个 hunk） | — |
| 开工快照 | `research/prompts/defs-m3-formal-gate-r2-snapshot/sha256sums.txt`（22 份） | 交齐之后 18 份 OK；对不上的 4 份都是别的会话改的：`.claude/rules/verification.md` 未暂存 3 行（快档与崩溃放量那两行、「门禁管哪一半」表第一行），不在被判的行上；`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/gate.d/stage-inputs.tsv` 是别的会话暂存的改动。攻方的探针跑在与快照 22 份逐份相同的拷贝上；辩方读主仓，引文行号交回时现查 |
| 云端辩方（Sonnet） | `research/prompts/defs-m3-formal-gate-r2-sonnet-output.md`（202 行，sha256 c7287385…364d）；模型 `…-sonnet-model/` | 没复跑：它的 E3-c 与攻方 J10 是两份互不共享代码的探针，结论同向 |
| 云端攻方（Opus） | `research/prompts/defs-m3-formal-gate-r2-opus-output.md`（215 行，sha256 66a9ad84…b1c9）；模型 `…-opus-model/` | 主 agent 在 `/tmp/claude-1000/defs-m3-formal-gate-r2-main-check/` 照 `rerun.sh` 复跑：`e1-output.txt`、`e1b-output.txt`、`e1-G9-output.txt`、`e1b-G9-output.txt`、`e2-lkmm-hook-output.txt`、`e3-mixed-output.txt` 与入库的逐字节相同；`e2-reuse-output.txt`、`e2-reuse-G10-output.txt` 只差输入文件数（566 → 576，期间别的会话往仓里加了文件），`e2-verus-toy-output.txt` 只差 Verus JSON 的键序；复用结论行原样：「Y1 未设 → 空串（Verus 起不了 z3）… q1 … 退 0（0 = 复用）… q3 … 退 10（10 = 复用）」，Y2（宏库截断）、Y3（裸名 z3 换 PATH 里那一份）同形，对照「版本变」两条问法都不复用；玩具 Verus 退出码 V1 0、V2 128、V3 0、V4 0、V5 1 |
| 引文 | 两份腿报告 | `python3 research/scripts/cite-check.py <两份报告> --root . --background research/prompts/_defs-m3-formal-gate-r2-background.md`：「核了 106 处引文，1 处对不上」——辩方第 51 行引 `.claude/singlefs-ai-sop/scripts/gate.sh:666`，原文在第 668 行；那句原文在，行号差两行，结论照采 |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| 第一轮 D1–D5 复核 | 五格都站得住；两处同形的别的臂没覆盖（QEMU 第 1 步不查 `research/results`；QEMU 没登记 `environment=`），记账不推翻 | — | **第一轮判决维持**。QEMU 两处记账 |
| E1 判法脚本干活与只读的分界 | — | J1 打中（量过）：`PYTHONPATH=… python3 -m`、`python3 -c runpy`、符号链接、子命令放变量或命令替换、shell 函数、for 循环逐格跑，闸照放；同样写法起广谱变异驱动闸都拒，只有这一道漏（`verus_judge_use` 认不出干活就放）。J2 打中不分辨臂：`env -C`、脚本目录放变量、`script -qc`、`xargs` 也漏，但广谱变异同样漏，病根在 `lib_shell_words.py`。J3 没打中 | **J1 打中，主 agent 复跑坐实**（`e1-output.txt` 与 `e1-G9-output.txt` 逐字节相同）。改法取攻方 G9：先认出是这两份判法脚本、再一律算重型，只放过字面写明的只读用法，认法照广谱变异那一份（攻方在副本上量到 J1 的 10 种写法全改判拒、只读用法仍放）。**零轮**。J2 不归这一道：并进共用库盲区那一批（`records/2026-09-16-subagent拆分提案.md` 第四十三节第 1 行） |
| E2 F1 与 F2 | — | J4 没打中：F1 之后指纹连算三次再在 worktree 里算一次，四次逐字相同。J5、J6、J7 打中（量过）：只截坏安装里的 `libverus_builtin_macros.so`、`VERUS_Z3_PATH` 未设与空串、裸名 `z3` 换 PATH 里那一份，三种都复用。J8 轻：F1 之后 lkmm 的标记查询在里面起 `lkmm.sh --herd7-version`，直接起这条命令闸会拒 | **J5、J6、J7 打中，主 agent 复跑坐实**（第一节最后两行）。改法取攻方 G10：`--tool-versions` 另哈希安装目录顶层的全部文件，`VERUS_Z3_PATH` 分开未设与空串，裸名照 PATH 找到再哈希。**零轮**。J8 记账：闸那一侧把 `lkmm.sh --herd7-version` 当只读放行，并进同一件 |
| E3 包装 250–254 判没跑成、253 判红 | E3-a、E3-b 站得住；E3-c 同一格里没跑成与真红可以并存（空真格 15 条 ✗ 里 8 没跑成、7 真红），`crash-verifier.md` 第 3 步按格记会盖掉真红；`gate-triage.md:29`「251 与 253 ⇒ 环境」对 Verus 253 是死文字；E3-d 外部杀进程（137 等）不在 250–254 里，E165 一侧仍写「修实现回归」；E3-e 排队等待推的 | J9、J12 没打中；J10 打中（量过）：一处包装退 252、另一处真红时判法打两条 ✗，崩溃验证员第 3 步照字面记整格没跑成、门禁分诊按 252 归并发，都盖住真红；J11 轻：Verus 一侧吞了包装自己的 ✗ 与 →，252 两种成因出路只写一种 | **两条腿独立同向打中同一处**（辩方 E3-c 与攻方 J10）。改法取攻方 G11：定义改成一格里每一条 ✗ 都是没跑成才记没跑成，有一条真红就记红。`gate-triage.md` 那一句写成对 Verus 253 按超时判红；E165 一侧外部杀进程的出路改成照包装文件头；Verus 一侧把包装的 ✗ 与 → 原样带出。**零轮** |
| E4 写实句与清单 | E4-a、E4-b、E4-c、E4-e 站得住（第 1 步会因别的会话的未跟踪产物停，不是每次都停，频率没量）；E4-d 同族清单还有四处注释没补（`research/scripts/admission.py` 第 228、2763、2929 行、`heavy-test-guard.sh` 提示文字）；E4-a 三处小毛病（「它不直接调」指代含糊、漏了 24 小时复用上限、`heavy-test-guard.sh` 提示还留「靠复用」） | — | **E4-d 与 E4-a 三处采**，逐处补。E4-b 记账：第 1 步因别的会话的未跟踪产物停的频率，等几次提交的实况再判 |

## 三、下一步

1. 改法并成一件工具活（第 4 批 3-12，tooling-writer）：G9、G10、G11 与 E3、E4 采的几处；J8 一并。
2. 改完开第三轮（最终轮），只攻这一轮的改法；第三轮之后不再开轮，新冒出来的写进交用户的表、标零轮。

不涉及决策：这一轮只判规则、定义、钩子与门禁复用的写法，没有一条推翻或确立决策分项。
```
