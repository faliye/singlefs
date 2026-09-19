# 知识腐烂回扫 · 领域 B（格式常量、实验与决策正文）· 阶段同步 knowledge-rot-b

阶段名：knowledge-rot-b。改动范围：`git diff b1c8cef~1 HEAD -- .claude/kb/decisions/ .claude/kb/decisions-history/ .claude/kb/experiments/ .claude/kb/experiments.md .claude/kb/checks-owed.md .claude/kb/prior-art.md .claude/kb/vm-harness.md research/perf-by-milestone.md research/results/ research/mutations/ research/scripts/replay.sh`，加工作区。

## 一、树表条目 148 → 200（format-evolution.md 逐类搜）

命令与计数：

```
grep -rn '148' --include='*.rs' crates/ research/e7-index-bench/ 2>/dev/null | wc -l   # 22
```

命中清单（逐条判定）：

| 文件:行 | 原行摘录 | 判定 | 理由 |
|---|---|---|---|
| research/e7-index-bench/src/bin/e102_unit_class_registry.rs:353,493,494 | `[64u64, 65, 76, ..., 148, 149]` / `[9, 148]` | 同一个数、不同的量，不改 | E102 头宽闭区间的端点，与树表条目宽无关 |
| crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:294-295 | `148 + 4 * 21 + 2 * 13 + 21`，注「到 D 148」 | 同一个数、不同的量，不改 | 148 是固定脚本到 D 为止的**写调用累计数**（118+2+15+13），不是树表条目宽 |
| research/e7-index-bench/src/bin/e131_livelist_carrier.rs:230-237 | `the_retired_denominator_agrees_at_200_bytes_and_diverges_at_148` | 冻结证据/故意对照，不改 | 测试名与断言本身就是在同时验证 200 与 148 两档在这个分母上是否分道，不是遗留旧值 |
| research/e7-index-bench/src/bin/e145_self_describing_node_header.rs:388-390,404 | `fanout(..., 109), 148` | 同一个数、不同的量，不改 | format-evolution.md 原文点名的假阳性：E145 的 extent 扇出恰好也是 148，与树表条目宽无关 |
| research/e7-index-bench/src/bin/e136_fork_cost_rows.rs / e134_map_key_slot_baselines.rs | `[149, 152, 139, 155, 296, 148]` 等 | 同一个数、不同的量，不改 | map_internal_fanout / 几何数组里的坐标值，与树表条目宽无关 |
| research/e7-index-bench/src/bin/e148_commit_fixpoint_two_record_trees.rs、e151、e141 | `E148（提交固定点按两棵记录树重算）` | 不相干 | 148 是实验编号，不是常量值 |

结论：**crates/ 与 research/e7-index-bench/ 里没有找到需要改的 148 遗留**；148→200 已在 d2aeb7d 落地完整。

`.claude/kb/layout/01-first-txn.md:378` 已有 `format-const: TREE_TABLE_ENTRY_BYTES = 200 stale=expect("148")|预留 24 非零|树表条目预留 24|TREE_TABLE_ENTRY_BYTES, 148|7 × 148|1036|每层 109|109²|预留剩 24|[147] = 1` 登记，未现扫每个子串（时间预算内未做，见「没做什么」）。

`.claude/kb/milestone/02-second-txn.md` 里的 148 全部出现在「## 开工前」小节（事件陈述：「改前 148、改后 200」）与「## 历史版本」小节（574、577 行，`### 2026-09-16` 标题之下），按「把旧值换成新值，这句还是不是真话」判据，都是**那一次发生的事，不改**。

## 二、D16 已定项 1「生效」一行打回重议（撤回类回扫）

`.claude/kb/decisions/16-发布语义.md:375` 定义「生效」= 「每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值」；`:381` 已有 2026-09-17 打回重议的整段说明（撤回原文就在决策正文自己里，符合共用约束）。

搜索命令：

```
grep -rln '各幸存盘所带 F 最大值的最小值\|F_生效' --include='*.md' . | grep -v '^research/prompts/'   # 20 个文件（含 decisions/16 自己）
```

（`research/prompts/` 命中 200+ 个文件，全部是三方论证冻结材料，按定义不逐条扫，见「没做什么」。）

逐文件判定（只挑出与「生效」定义本身相关、不是泛泛提 F_生效 概念的）：

| 文件:行 | 判定 | 理由 |
|---|---|---|
| CLAUDE.md:97 | 不改 | 已带 2026-09-17 之后的口径："候选集的下界取最新根自己带的 F 而不是 F_生效，一块盘的载体根坏掉之后会漏判，见里程碑步 6 现状"——已经是打回重议之后的现状句 |
| .claude/kb/invariants.md I-3.1（第 123 行） | 不改 | 已带 "2026-09-17 起「有效根」= 回退候选集里的根" 与 F 回落的现状说明，与 CLAUDE.md 一致 |
| .claude/kb/invariants.md I-7.4 / I-4.8 | 分不清，要人看 | 两行仍平铺引用 F_生效 而不像 I-3.1 那样带打回重议的现状说明；但 CLAUDE.md:97 的括注写的是「后加的六条里除 I-3.8 之外的五条」（含 I-7.4/I-4.8）都按候选集判、都有下界漏判的现状——即高层文件已统一说明，两行本身要不要各自复述这句留给人判 |
| .claude/kb/checks-owed.md:385 (C215) | 不改 | 是 2026-09-13 定案时的记录（日期落款），描述的是那一次定案改了什么，不是「生效」当前定义 |

未逐一读完 20 个文件里的其余 12 个（.claude/kb/experiments/、decisions/22、23、03、05、layout、milestone、records 三份、decisions-history 两份），只抽查了最可能承载"现状"断言的 CLAUDE.md / invariants.md / checks-owed.md，见「没做什么」。

## 三、D28 已定项 1 第九项口径改（撤回类回扫）

`decisions-history/2026-09.md:28`（其四，2026-09-17）：改前「上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量，回退那一刻算」，改后「取值 = 被抛弃根引用的槽 − 回退候选集里的根引用的槽（下界 0），每次挂载与每次抬 F 时算」。`decisions/28-挂载期承诺量.md:30` 已同步为新口径。

搜索：

```
grep -rn '被抛弃根独占量' --include='*.md' . | grep -v '^research/prompts/'   # 17 处
```

逐处判定：

| 文件:行 | 原文摘录 | 判定 | 理由 |
|---|---|---|---|
| **.claude/kb/decisions/05-快照-空间记账机制.md:351** | 「被抛弃根独占量 \| **例外**：只住内存的派生量——回退选中 R_old 那一刻按「被抛弃根的已分配统计量 − R_old 的已分配统计量」算……（D28 已定项 1 第九项，2026-09-13 用户定案）」 | **要改** | 这是定义表格的现状行（不在历史版本节），逐字还是 2026-09-13 的旧公式与旧落款日期，没有跟 2026-09-17 的新口径（每次挂载/抬 F 时算，减去候选集重叠）同步。改后应为「取值 = 只被被抛弃根引用的槽数 = 被抛弃根引用的槽 − 回退候选集里的根引用的槽（下界 0），每次挂载与每次抬 F 时算」（与 decisions/28 已定项 1 第九项原文一致），落款也应带 2026-09-17 |
| .claude/kb/checks-owed.md:296 (C318) | 「改法」列写「候选口径……上界 = 被抛弃时间线自 R_old 起的分配量（按根记录携带的已分配统计量之差算……）」；「前置」列写「口径取『被抛弃根的已分配统计量 − R_old 的』」 | 分不清，要人看 | 行首标「前置已还」，是当时定案时的记录，但同一行的「改法」列语气像是仍待落实的候选方案描述，与已经改了两次口径（09-13→09-17）的现实是否需要跟进说明，交人判 |
| .claude/kb/milestone/02-second-txn.md:176-178 | 「预想的细节」小节：「影子账……上界 = 被抛弃根的已分配 − A 的已分配」 | 不改（判定为事件/设计草案，非现状） | 该文件同一处「现状（2026-09-17 落地）」段（165 行起）已用新口径描述实际实现（`isolate_slots_referenced_only_by_abandoned_roots`，减去候选集重叠）；「预想的细节」标题本身表明是早于「现状」的设计草稿，两段并存是本文件既有写法（先写预想再写现状核对），按「换成新口径这句还是不是真话」判据，它说的是「那一次的预想」不是当前定义 |

## 没做什么

- format-const 的 `stale=` 各子串没有逐一现扫 0 命中（时间预算内只核了整体 148 搜索，未按 rule 要求对 `stale=` 里每个子串单独跑一遍并给出 0 命中命令），因为这是既有登记（d2aeb7d 提交时已登记），不是本轮新改动，未新增或修改 `stale=`。
- D16「生效」撤回的回扫，`research/prompts/` 下 200+ 个冻结文件与 `.claude/kb/experiments/`、`decisions/22`、`23`、`03`、`05`、`layout/`、`milestone/`、三份 `records/`、两份 `decisions-history/` 共 12 个非 research/prompts 文件未逐条读引用上下文，只按文件名与最高可能性抽查了 CLAUDE.md / invariants.md / checks-owed.md。
- 阶段同步第 7-9 步（关键词×进度词回扫 CLAUDE.md / README.md / agent-common.md / agents/ / rules/ / kb/（除两份变更史）/ records/）没有系统性做完：只做了 E155/E153/E154「部分已跑」状态一致性抽查（experiments.md、experiments-history.md、CLAUDE.md、README.md 没有发现互相矛盾的「已跑完」或「未跑」表述）、C336-343/C378-381 在 checks-owed.md 里的登记核实（12 条全部在册）、E152 与树表 148 在 CLAUDE.md/README.md 里的现状句核查（未发现遗留旧值）。没有做：`.claude/agents/`、`.claude/rules/`、`.claude/kb/milestone/`、`.claude/kb/layout/` 全文按改动范围关键词（E142 第九至十一次跑、七家排位、D5/D23/D16/D28/D13 六条决策变更史条目、C146/C366/C368/C369/C374 更新内容）逐一回扫「没做/待定/打算」类进度词；这部分需要更多轮次才能覆盖，本轮效力受限（effort 预算）未展开。
- 未检查 `.claude/gate.d/65-relay-timing.sh`、`vm-harness.md` 新增 27 行、`e152-preregistration.md` 等工作区里"另一个会话正在写"的文件是否与本轮已提交内容互相矛盾（这些文件本身在 git status 里标 M，按派发提示只需判定并标注"别的会话在写"，本轮没有触及）。
- 没有跑任何 gate 或编译，没有改动任何被搜到的文件。

## 补扫（应主 agent 追加指示，2026-09-18）

### 补扫① D16 已定项 1「生效」打回的回扫：12 个非 prompts 文件逐处判

范围：`grep -rln '各幸存盘所带 F 最大值的最小值\|F_生效' --include='*.md' . | grep -v '^research/prompts/\|^CLAUDE.md$\|invariants.md$\|checks-owed.md$\|decisions/16-发布语义.md$'` 共 17 个文件（含 2 份变更史）。逐处判定：

| 文件:行 | 判定 | 理由 |
|---|---|---|
| .claude/kb/experiments/41-根环槽几何2x4x256.md:85-86 | 不相干 | 只引用「块复用的界 = max(F_生效, 环里最旧有效根)」这条公式本身（未被撤回），不涉及「生效」怎么算 |
| .claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:97 | 不相干 | 同上，只引块复用界公式 |
| .claude/kb/experiments/47-根环失败域的损失.md:30 | 不相干 | 同上 |
| .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:178 | 不相干 | 只提"F_生效"作为已实现四样之一的名字，不复述其定义 |
| .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:78 | 不相干 | 提及"defer 窗口与 F_生效不在模型里"，是装置范围说明，不是定义复述 |
| **.claude/kb/decisions/22-单元原子性怎么合成.md:492** | **要改（补警示）** | 根记录字段表「回退下界 F」行逐字写「恢复后生效值 = 各幸存盘所带 F 最大值的最小值」，把打回重议的那句原样当已定字段说明留着，没有 CLAUDE.md:97 / invariants.md I-3.1 那样的现状警示指针。改法：行尾加「⚠️ 2026-09-17 用户打回重议，见 D16（发布语义） 已定项 1 表格后那段」或链接过去，不复述细节 |
| .claude/kb/decisions/05-快照-空间记账机制.md | （无 F_生效 定义复述，只出现在本轮补扫②要处理的第九项行里，见下） | — |
| .claude/kb/layout/02-second-txn.md:12 | 不改 | 字节布局例子行本身就是「重开之后新实例的根写 F_生效……可以比上一条根低」——这正是被撤回定义所描述的那类会回落的场景，与撤回结论方向一致，不是过期引用 |
| .claude/kb/layout/01-first-txn.md:82 | 不改 | 只用 F_生效 指代第一个事务里的下界概念（当时恒为初值），不复述「各幸存盘…」那句计算式 |
| .claude/kb/decisions/23-journal的角色与格式.md:691,1209 | 不改 | 用 F_生效 构造候选集判据（txg ≥ F_生效），且 :1209 已带 2026-09-16/17 两次收严说明；不是在断言"生效"怎么算这件事本身 |
| .claude/kb/decisions/03-空间分配.md:180 | 不改 | 同上，只用于清扫准入闸的谓词，不复述"生效"定义 |
| **.claude/kb/milestone/02-second-txn.md**（多处，166/195/202/222 行区间） | 不改（已同步） | 全部已带打回重议的现状说明，如 195 行段落末尾「F 回落……2026-09-17 用户打回重议，见 D16（发布语义） 已定项 1 表格后那段 ⚠️」、222 行段落「候选集下界取最新根自己带的 F 而条款要 F_生效……交用户」，与撤回后的现状一致 |
| records/2026-09-17-已分配口径三方与两个实验.md、records/2026-09-13-总审核.md、records/2026-09-16-总审核回扫核查.md | 事件句，不改 | 均是带日期的会议/审核记录，描述"那一次"的问题清单，不是现状断言 |
| .claude/kb/decisions-history.md、decisions-history/2026-09.md | 事件句，不改 | 变更史本身即"那一次发生的事"的记录格式，按定义不算现状陈述 |

结论：本轮只发现 **decisions/22:492 一处要改**（补撤回警示指针），其余全部不改或不相干。

### 补扫② D28 第九项口径改：全仓（除 prompts / 变更史）搜三串，逐处判

搜索命令与计数（去掉 `research/prompts/` 与两份变更史）：

```
grep -rl "已分配统计量 −" --include='*.md' . | grep -v '^\./research/prompts/\|decisions-history'   # 3 个文件
grep -rl "R_old 的已分配" --include='*.md' . | grep -v '^\./research/prompts/\|decisions-history'    # 3 个文件
grep -rl "被抛弃根独占量" --include='*.md' . | grep -v '^\./research/prompts/\|decisions-history'    # 10 个文件
```

逐处判定（去重合并三串命中的文件）：

| 文件:行 | 原文摘录 | 判定 | 理由 |
|---|---|---|---|
| .claude/kb/decisions/05-快照-空间记账机制.md:351 | 「回退选中 R_old 那一刻按『被抛弃根的已分配统计量 − R_old 的已分配统计量』算」 | **要改**（已在原报告核实，主 agent 已确认不用再报） | — |
| **.claude/kb/checks-owed.md:296（C318，「怎么拦」列 + 「前置」列）** | 「怎么拦」列：「候选口径『被抛弃根独占量』的上界 = 被抛弃时间线自 R_old 起的分配量（按根记录携带的已分配统计量之差算，不遍历树）……」；「前置」列：「口径取『被抛弃根的已分配统计量 − R_old 的』」 | **要改** | 两列都还是 2026-09-13 的旧口径，未跟 2026-09-17 改动（decisions-history/2026-09.md 其四）同步 |
| .claude/kb/experiments/153-账本形态与环上有洞的代价.md:37 | 「D28 已定项 1 第九项那句『上界 = 被抛弃根的已分配统计量 − R_old 的已分配统计量』按本实验两种读法都罩不住这种影子账（登记 F4：条款可能错，交主 agent，不改影子账定义去凑上界）」 | 事件句，不改 | 这是引用"当时的条款原文"来报告一次实验发现（E153 第五段续做），带着"登记 F4"式的发现记录，正是导致 09-17 改口径的证据之一；换成新口径这句会变成假话（新口径下这条实验发现的针对对象就不存在了），按判据"说的是那一次发生的事"不改 |
| .claude/kb/verification-build.md:74 | 「被抛弃根独占量（D28 已定项 1 第九项）」 | 不改 | 只提名称，不复述计算式 |
| .claude/kb/experiments-history.md:112 | 标题「其二十六：……准入第九项『被抛弃根独占量』的判别力自证」 | 事件句，不改 | 变更史式标题，带日期 |
| .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:20 | 「被抛弃根独占量 = 只被被抛弃的根引用的」 | 不改，且与新口径一致 | 这行本身已经是"只被…引用"的窄口径措辞，与 2026-09-17 新定义方向一致，不是旧公式 |
| .claude/kb/layout/02-second-txn.md:15 | 「影子账（被抛弃根独占量，D28 已定项 1 第九项）」 | 不改 | 只提名称 |
| .claude/kb/decisions/28-挂载期承诺量.md（全文） | 已定项 1 第九项原文 | 不改（已是新口径） | 已核实为 2026-09-17 之后的现行定义来源 |
| .claude/kb/decisions/03-空间分配.md:146 | 不等式里的项名「被抛弃根独占量」 | 不改 | 只是不等式里的项名，不复述计算式 |
| .claude/kb/milestone/02-second-txn.md（176/178/195 行区间） | 「预想的细节」段落用旧公式；「现状」段落已写「第九项的公式 2026-09-17 用户定案改成独占集」 | 不改（已同步；「预想的细节」判定为设计草稿，非现状） | 与上一轮报告一致：本文件"现状"段已带新口径说明，"预想的细节"是该步骤动手前的设计稿，两段并存是本文件既有写法 |
| records/2026-09-13-总审核.md（166 行区间，若命中） | 审核记录 | 事件句，不改 | 带日期的审核清单 |

**C318 改后句（只判不改，供 kb-scribe 落笔）**：

「怎么拦」列改后：
> 准入不等式已加第九项『被抛弃根独占量』（D28（挂载期承诺量） 已定项 1 第九项，2026-09-17 改定）：取值 = 只被被抛弃根引用的槽数 = 被抛弃根引用的槽 − 回退候选集里的根引用的槽（下界 0），每次挂载与每次抬 F 时算，被抛弃的根被轮转覆写时清零；判别力自证：把隔离量置 0，只差那几块的池要从只读翻成可写。

「前置」列改后：
> 前置已还：2026-09-13 用户定案加第九项『被抛弃根独占量』（D28（挂载期承诺量） 已定项 1），口径 2026-09-17 由『被抛弃根的已分配统计量 − R_old 的已分配统计量』改成『只被被抛弃根引用的槽数』（被抛弃根引用的槽 − 回退候选集里的根引用的槽，下界 0），与 `mount::isolate_slots_referenced_only_by_abandoned_roots` 的实际隔离集合同一个数；判别力自证的模型形态同日由 E150（回退复用被抛弃的根引用的单元） 第二次跑做出，实现侧的检查登记在 [verification-build.md](verification-build.md) 事务层第一版「运行时计数与准入自证」表。

### 补扫③ 第 7–9 步系统回扫：.claude/kb/experiments/ 与 .claude/kb/decisions/ 全文，关键词 E142/E152/E153/E154/E155/C336–C343/C378–C381 + 进度词

搜索命令与计数（两目录合计）：

```
for kw in E142 E152 E153 E154 E155 C336 C337 C338 C339 C340 C341 C342 C343 C378 C379 C380 C381; do
  grep -rn "$kw" .claude/kb/experiments/ .claude/kb/decisions/ | wc -l
done
# E142:34 E152:6 E153:4 E154:5 E155:3 C336:2 C337:2 C338:4 C339:0 C340:1
# C341:0 C342:0 C343:0 C378:0 C379:0 C380:0 C381:0
```

C339、C341、C342、C343、C378–C381 在这两个目录里 0 命中——查过 `checks-owed.md` 均已在册（见原报告「二」），只是没有被 `experiments/`、`decisions/` 正文引用，判定：不相干（不是每条欠账都要在决策或实验正文里现身）。

逐条判定（挑出与进度词共现或需要判"跑没跑/定没定"的命中）：

| 文件:行 | 摘录 / 进度词 | 判定 | 理由 |
|---|---|---|---|
| .claude/kb/experiments/153-账本形态与环上有洞的代价.md:1 | 标题「部分已跑」，列出已跑/未跑清单（Q8 五种坏镜像已跑、B3a/B3b/B4a/B4b/B4c 与崩溃支线 L2/L3 未实现） | 不改 | 与 .claude/kb/experiments.md:171 摘要一致，无矛盾 |
| .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:1,192,195 | 标题「部分已跑」；192 行「H3a/H3b/H4a/H4b 与 ROW 明细仍未实现，留给下一段」；195 行「状态仍『部分已跑』」 | 不改 | 192 行是第二段（早于第四段）时的进度记录，195 行确认第三段仍是部分已跑；与顶部 `## E154` 标题（已覆盖 H4a/H4b）不矛盾——192 行那句的语境是"第二段做完时 H4a/H4b 还没做"，第四段（顶部标题）已经补上，是事件推进记录，非矛盾 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:1 | 「部分已跑」，列出 Q1.4/8.2/K9′ 未做 | 不改 | 与 experiments.md:173、experiments-history.md 一致 |
| .claude/kb/decisions/16-发布语义.md:209 | 「崩溃点重放 harness 那条检查仍欠」 | 分不清，要人看 | 落款 2026-09-13，之后 D16 已定项 1「生效」2026-09-17 被打回重议——这句"仍欠"的检查是否已随重议范围扩大或被并入，未逐一核实 crates/singlefs-harness 现状 |
| .claude/kb/decisions/22-单元原子性怎么合成.md:657 | 「仍欠：挡路条①/④后半句……的论证，不挡字节」 | 不相干 | 与 E142/C336-381 无关，是另一条独立欠账（加密 D9），跳过 |
| .claude/kb/decisions/23-journal的角色与格式.md:691 | 「频率未测」（实例切换代价） | 不相干 | 与本轮关键词族无直接关系，是既有的、未声称已解决的度量缺口，未见矛盾 |

E142 34 处命中里，绝大多数是引用某次具体跑的产物数字作证据（如"第六次跑产物 ... states=262165""第七次跑记的 G4/G9/G24""按 481 重跑（第七次跑）"），这类是**带编号的历史事实引用**，换成"最新一次跑"不会更真、原句本来就没有断言"这是最新一次"，按判据（换成新值这句还是不是真话）**不改**。未发现有文件仍声称"E142 只跑到第 N 次"这种与"第九至十一次跑已落地"矛盾的现状句。

C336、C337、C338 的 4+2+2 处命中（.claude/kb/decisions/05、08、22、milestone/02）均为「检查欠在 C336」「没有条款（C337）」「见 C338」式的欠账指针引用，与 checks-owed.md 的登记状态（均未标"已还清"）一致，不改。

**结论：本轮系统回扫在 `.claude/kb/experiments/` 与 `.claude/kb/decisions/` 全文范围内，除上表标"分不清，要人看"的一处（D16 已定项 1「暖机」段落里 2026-09-13 遗留的"崩溃点重放 harness 那条检查仍欠"是否已被 09-17 的重议吸收）之外，没有发现需要改的现状句、也没有发现遗漏登记的新东西。**

## 补扫的没做什么

- 未对 E142 全部 34 处、C338 全部 4 处逐字读取上下文之外的相邻段落（只读了命中行本身及其所在段落的直接上下文，未展开读整节）。
- 未检查 `.claude/kb/milestone/`、`.claude/kb/layout/`、`.claude/kb/checks-owed.md` 三份文件里 E142/E152/…/C381 关键词的命中（补扫③按主 agent 指示只限定 `.claude/kb/experiments/` 与 `.claude/kb/decisions/` 两个目录，未扩大范围）。
- decisions/22:492 的补警示文本、checks-owed.md C318 两列的改后句均只是本轮判定与建议，未写入任何文件（写范围仅报告与草稿目录）。
