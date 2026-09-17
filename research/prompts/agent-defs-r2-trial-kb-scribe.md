# 书记员重新试跑报告（虚构定案，仅副本）

副本根：`/tmp/claude-1000/agents/trials2/kb-scribe/repo/`
规格文件：`/tmp/claude-1000/agents/trials2/kb-scribe/spec-from-main.json`（4 条）
草稿目录：`/tmp/claude-1000/agents/trials2/kb-scribe/draft/`

## 摘要

规格 4 条里 3 条落在写范围内（全在 `.claude/kb/decisions/22-单元原子性怎么合成.md`），已按规格原样写入；
第 4 条指向 `CLAUDE.md`，不在书记员写范围（`.claude/kb/**` 与 `/tmp/claude-1000/**`），停下未写，见下「未写：CLAUDE.md 那条规格」。
决策变更史条目已按主 agent 给定的标题、快查两行、改前/改后/依据写入 `.claude/kb/decisions-history/2026-09.md`，并跑
`49-history-brief.sh --write` 重新生成 `decisions-history.md`。D22 第 7 项已用 `relabel-item.py D22 7` 翻写全仓引用（207 处、56 个文件），
随后跑 33 号（变异表锚点），红：`e100_superblock_slot` 的变异表锚点因源码注释被 relabel-item.py 改写而腐化——列在下面，交主 agent，未修。
`stage-owners.tsv` 登记给 kb-scribe 的 30 个门禁阶段全部跑过一次：27 绿、3 红（20-kb-shape、31-blocking-verdict、33-mutation-tables）。
三处红逐一核过：点名的文件与行都在这一轮改动里，但修复都需要新增规格未给的判断或句子（状态标记、是否阻塞第一个事务字节、标题计数），
按「照规格写，不补内容、不做判断」未修，原样列给主 agent。

## 1. 开工时的 sha256（规格点名的两个文件）

```
10fc8483f070e2cbd9e57e73255ae64d000d2e459ff72212d78a81bb6ee73679  .claude/kb/decisions/22-单元原子性怎么合成.md
9c842c8d2adac61d24bd33574b39e6d21080dced9db78dfed4039cc487a79769  CLAUDE.md
```

⚠️ 见文末「试跑观察」第 3 点：relabel-item.py 与两个 `--write` 门禁阶段的写入footprint（另外 55 个文件）
在开工时不可知，只有跑过 `--dry-run` 之后才现出来；这些文件只补得上收尾哈希，补不上开工哈希，详情见该节。

## 2. replace-batch.py 规格核验（原样输出）

规格文件（草稿目录）：`draft/spec-replace-batch.json`，由 `spec-from-main.json` 的 `文件/旧串/新串` 转成
`file/old/new` 键生成，4 条全部原样保留（含 CLAUDE.md 那条，用于核验命中数，不代表要写它）。

```
$ python3 research/scripts/replace-batch.py --dry-run draft/spec-replace-batch.json
✓ 4 处都命中得对（2 个文件），--dry-run 没写
exit=0
```

4 条全部恰好命中一次。之后**只对 D22 文件那 3 条用 Edit 逐条改**（写范围闸看得见），CLAUDE.md 那条没有用任何方式写。

## 3. D22 文件的三处编辑（已写，逐条回读确认）

- 已定项索引表：删掉第 7 行（`| 7 | **根记录的字段表** | **已定**...`）——`grep` 回读命中 0 次，确认已删。
- 未定项索引表：第 6 行（zoned 区写指针）后追加第 7 行
  `| 7 | **根记录的字段表** | **未定**：2026-09-17 书记员试跑翻回（虚构，只在仓副本里）。 |`——回读命中 1 次于第 671 行。
- 正文小节标题：`#### 已定项 7（2026-09-02，用户定案 + E79（根记录的容量））：根记录的字段表`
  改成 `#### 未定项 7（2026-09-17 书记员试跑翻回，虚构）：根记录的字段表`——回读确认。

三处都与规格给的「新串」逐字一致，没有添加规格外的句子。

## 4. 未写：CLAUDE.md 那条规格

规格第 4 条要把 `## 什么时候派哪个 agent` 改成 `## 什么时候派哪个 agent（试跑）`，目标文件 `CLAUDE.md`。
`.claude/hooks/agent-write-scope.tsv` 里 kb-scribe 只登记了 `.claude/kb/**` 与 `/tmp/claude-1000/**` 两行，
`CLAUDE.md` 不在其中——这正是 `kb-scribe.md` 定义原文举的例子（「规格点名的文件不在你的写范围里（例如 `CLAUDE.md`），
也停下报告，不按规格硬写」）。已停下，未用 Edit、未用 Bash 绕过写，`CLAUDE.md` 现在的 sha256 与开工时一致（见上）。

## 5. 决策变更史条目

写进 `.claude/kb/decisions-history/2026-09.md`「## 历史版本」最上面（比已有的裸 `### 2026-09-17：` 条目更靠上，
因为它是当天第二条，按当月文件里「同一天多条按其N倒序、裸标题恒排在同一天最后」的既有先例排在前面）：

```
### 2026-09-17（其二）：D22（单元原子性怎么合成） 已定项 7 翻回未定（书记员重新试跑，虚构）

> 快查·改前：D22 已定项 7「根记录的字段表」是已定。
> 快查·改后：翻回未定，正文与两张索引表跟着改，全仓引用由 relabel-item.py 改写。

- **改前**：已定项 7「根记录的字段表」是已定。
- **改后**：翻回未定。
- **依据**：主 agent 2026-09-17 书记员重新试跑规格（虚构）。
```

标题整行、快查两行、改前/改后/依据都按主 agent 给定的原样写入，没有自己概括或扩写。

跑 `bash .claude/gate.d/49-history-brief.sh --write` 前后各 `git diff -- .claude/kb/decisions-history.md
.claude/kb/decisions-history/` 一次并逐行 diff 两份输出：新增的只有 D22 那一节的「共改过 51 次」计数更新、
新的一行简报（`D22（单元原子性怎么合成） 未定项 7 翻回未定（书记员重新试跑，虚构）`），以及「最近 3 次」窗口
挤掉最旧的那条（2026-09-13 总审核那条，本来就在窗口边缘，按设计只留 3 条，原文没有被移动或改写，仍在
`decisions-history/2026-09.md` 里）。`decisions-history/2026-09.md` 本身在这一步没有任何改动。
没有任何条目被补「（待补）」——满足停下交回的判据（没有出现就不用交回）。

```
$ bash .claude/gate.d/49-history-brief.sh --write
  ✓ 按原文重新生成了 .claude/kb/decisions-history.md：425 条条目，这次补了 0 条「（待补）」快查
```

## 6. relabel-item.py D22 7（先 dry-run，再实跑）

dry-run 与实跑输出完全一致（207 处、56 个文件、要人看的句子 0 处）：

```
$ python3 research/scripts/relabel-item.py D22 7 --dry-run
（56 个文件逐行列出改写处数，合计 207 处；此处不重复全表，见 draft/ 目录同名日志）
✓ D22（单元原子性怎么合成） 第 7 条是未定：改写 207 处、56 个文件（--dry-run，一个字没写）；要人看的句子 0 处

$ python3 research/scripts/relabel-item.py D22 7
（同一张表，一字不差）
✓ D22（单元原子性怎么合成） 第 7 条是未定：改写 207 处、56 个文件；要人看的句子 0 处
```

「要人看」的句子 0 处：没有需要转交主 agent 自己改写的「说它没定」残留句。

### 改到的 16 个 research/**/*.rs 实验源码（列给主 agent；入库产物里印着旧标签的，复跑会对不上）

```
research/e7-index-bench/src/bin/e100_superblock_slot.rs
research/e7-index-bench/src/bin/e115_superblock_completeness.rs
research/e7-index-bench/src/bin/e124_superblock_recompute.rs
research/e7-index-bench/src/bin/e126_superblock_slot_width.rs
research/e7-index-bench/src/bin/e132_livelist_carrier_recount.rs
research/e7-index-bench/src/bin/e133_map_key_format_cost.rs
research/e7-index-bench/src/bin/e134_map_key_slot_baselines.rs
research/e7-index-bench/src/bin/e135_rollback_floor.rs
research/e7-index-bench/src/bin/e136_fork_cost_rows.rs
research/e7-index-bench/src/bin/e138_per_disk_floor.rs
research/e7-index-bench/src/bin/e139_tightened_floor.rs
research/e7-index-bench/src/bin/e141_switch_reserve_mount_admission.rs
research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs
research/e7-index-bench/src/bin/e146_livelist_entry_width.rs
research/e7-index-bench/src/bin/e97_entry_encoding.rs
research/e7-index-bench/src/bin/e99_writebuffer_sequence.rs
```

### 加跑 33 号（变异表锚点）：红

```
$ bash .claude/gate.d/33-mutation-tables.sh
  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 会退出码 3，后面的条目一条都不跑）：
      e100_superblock_slot 的 M3_间接层没省（表第 3 行，命中 0 次）
```

根因：`research/mutations/e100_superblock_slot.tsv` 第 3 行的「原文」列还写着
`… 同 D22 已定项 7 的树表单元指针宽度`，而 relabel-item.py 把源码 `research/e7-index-bench/src/bin/e100_superblock_slot.rs`
第 28、111 行的同一句改成了「同 D22 未定项 7 …」，表里的旧串在源码里现在命中 0 次。
`research/mutations/**` 不在书记员写范围（只许 experiment-runner 改），此处只报告、不修。

## 7. 21-decision-items-sync.sh --write

```
$ bash .claude/gate.d/21-decision-items-sync.sh --write
  ✓ 决策索引表状态列与正文同步（28 条决策，写回 1 行）
  ✓ 已重新生成并写回 .claude/kb/decisions.md
```

`git diff -- .claude/kb/decisions.md` 只有两处：索引表那一行「已定 16 项 / 未定 1 项」→「已定 15 项 / 未定 2 项」，
以及展开清单里「D22（单元原子性怎么合成） —— 半定（分项 17，其中未定 1）」→「未定 2」、
「7. 根记录的字段表 —— 已定」→「—— **未定**」。与规格的翻状态意图一致，没有别的行被动。

## 8. 30 个 stage-owners.tsv 登记给 kb-scribe 的门禁阶段，各跑一次

```
10-kb-rot.sh                        exit=0  ✓ kb 腐化审计通过
20-kb-shape.sh                      exit=1  ✗ 两条见下「归属核实」
21-decision-items-sync.sh           exit=0  ✓ 决策索引表状态列与正文同步（28 条决策）
22-item-ref-status.sh               exit=0  ✓ 分项引用与正文状态一致（28 条决策、222 个分项）
23-link-targets.sh                  exit=0  ✓ 文档指向都到得了（1196 条相对链接、37 处「第 N 节」指向）
24-status-redundancy.sh             exit=0  ✓ 状态只说一遍，且分项都在对的节里（扫 231 份）
25-kb-deictic.sh                    exit=0  ✓ kb 正文里的时间指代都锚得到具体一轮（扫 197 份）
26-number-name-sync.sh              exit=0  ✓ 编号与简称一致（扫 216 个文件）
27-format-constants.sh              exit=0  ✓ 格式常量同步（19 个已登记，19 个在源码里被钉住）
28-cross-decision-status.sh         exit=0  ✓ 没有把已定的决策说成未定（扫 28 条决策）
29-settled-item-self-open.sh        exit=0  ✓ 已定项的正文没有把已经定了的东西说成未定（扫 28 条决策）
30-decision-history.sh              exit=0  ✓ 决策正文改了 86 行，变更史新增 3 条条目
31-blocking-verdict.sh              exit=1  ✗ 见下「归属核实」
32-first-txn-fields.sh              exit=0  ✓ 第一个事务的字段表指向都成立（查了 354 处引用）
32-history-ordinal.sh               exit=0  ✓ 本次新增的历史条目没有撞号（存量 9 处撞号不在本阶段射程）
33-mutation-tables.sh               exit=1  ✗ 见上「6. relabel-item.py」与下「归属核实」
35-user-verdict-owed.sh             exit=0  ✓ 待用户复核的条款都有未还的账盯着（共 0 处）
36-invariant-count-cross-file.sh    exit=0  ✓ 不变量条数跨文件一致：66 条，扫 192 份
37-decision-summary-width.sh        exit=0  ✓ 决策索引结论列都不超 200 字（共 28 行）
38-field-table-projection.sh        exit=0  ✓ 被投影的分项，字段表都投影全了
39-field-table-sum.sh               exit=0  ✓ 字段表加出来的数都对得上
42-first-txn-trio.sh                exit=0  ✓ 三份文件互相挂钩（判「是」的未定项 0 条）
43-owed-table-shape.sh              exit=0  ✓ 欠账表两张登记表的行形状都对（99 行，2 张表）
44-settled-ref-says-open.sh         exit=0  ✓ 扫 341 个文件、7128 处已定项引用，没有一处紧跟着说没定
48-history-month-file.sh            exit=0  ✓ 查了 3 份决策变更史、425 条条目，都住在日期对得上的那一份
49-history-brief.sh                 exit=0  ✓ 决策变更史的快查与原文同步（425 条条目、910 格快查）
51-admission-terms-covered.sh       exit=0  ✓ 准入不等式 9 项逐项有归属
52-segment-registry.sh              exit=0  ✓ 比对了 5 处登记，逐字一致
53-format-const-placeholders.sh     exit=0  ✓ 格式常量文件里的占位都指得到分项或欠账
60-stale-open-items.sh              exit=0  ✓ 没有未定项被别处的更新甩在后面
61-settled-same-file.sh             exit=0  ✓ 本次 diff 没有新增「已定」小节，本阶段无对象可判
70-citations.sh                     exit=0  ✓ 89 条命中，0 条未命中
```

27 绿、3 红：20-kb-shape.sh、31-blocking-verdict.sh、33-mutation-tables.sh。

## 9. 三条红的归属核实（都判「这一轮写出来的」，都未修）

### 20-kb-shape.sh（两条）

```
✗ D22 未定项里有一条分项没写「状态：已定/未定」这个规范标记，状态只挂在小节位置上：
   | 7 | **根记录的字段表** | **未定**：2026-09-17 书记员试跑翻回（虚构，只在仓副本里）
   → 在这一行末尾补上 **状态：已定。** 或 **状态：未定。**
✗ D22 标题写「一项未定」，「### 未定项」小节里实际 2 项
   → 改正文标题里的条数，并同步 decisions.md 索引行
```
第一条：指的正是我按规格写入的那一行（`draft/` 里核对过，规格「新串」原文就没有带 `**状态：未定。**`
这个收尾标记，别的未定项行都有）。第二条：指 D22 文件第 1 行的整条标题
（`## D22 单元原子性怎么合成 —— 半定（……一项未定：zoned 区写指针挂到开线；……）`），这行不在规格 4 条里，
翻状态之后它的「一项未定」与实际的 2 项不一致了。
两条都是这一轮写出来的（改动前 D22 是「已定 16 / 未定 1」，改动后「已定 15 / 未定 2」，这两处文本都还停在旧计数上）。
**未修**：第一条要补的收尾句不在规格「新串」里，属于规格外的句子；第二条要改的标题条数与用词（是否要连带改
「树表指针归属已定」那半句的表述）需要判断怎么改写这句自然语言，两者都超出「照规格写，不补内容、不做判断」的范围。

### 31-blocking-verdict.sh

```
✗ 1 条未定项没有判过改不改第一个事务的字节：
   22-单元原子性怎么合成.md  D22 未定项 7：| 7 | **根记录的字段表** | **未定**：2026-09-17 书记员试跑翻回（虚构，只在仓副本里）。 |
   → 按「改不改变第一个事务写出的字节」这把尺判一次，在该分项的登记行里写「改第一个事务的字节：是/否/无对象（YYYY-MM-DD，依据：…）」
```
这也是这一轮写出来的（该行是我按规格新增的）。**未修**：判断根记录字段表翻回未定之后改不改第一个事务的字节，
是一次需要读 D22 正文、first-txn-layout.md 现状再下判断的实质性决定，不是规格给定的句子，按定义不做这个判断。

### 33-mutation-tables.sh

见上「6. relabel-item.py」，根因是 relabel-item.py 改写了 `e100_superblock_slot.rs` 里的注释文本，
变异表 `research/mutations/e100_superblock_slot.tsv` 的锚点因此腐化。这是这一轮写出来的（relabel-item.py
是我在这一轮触发的），但 `research/mutations/**` 不在书记员写范围，只报告不修。

**三条共同点**：都是「翻状态」这个动作在 kb corpus 里连锁出的下游校验缺口，缺口的填法都需要新的判断或新的句子，
而不是机械誊写规格。按 `kb-scribe.md`「照规格写，不补内容、不做判断」与「没做什么」一节（没判定案对不对、
没写规格外的句子），三处都原样列给主 agent，没有自行判断补上。

## 10. 改过的全部文件与哈希（开工时 / 收尾时）

**有开工时哈希的两个文件**（规格直接点名）：

```
                                                                          开工                                                          收尾
.claude/kb/decisions/22-单元原子性怎么合成.md
  开工 10fc8483f070e2cbd9e57e73255ae64d000d2e459ff72212d78a81bb6ee73679
  收尾 19795d2adafe941339ec28c581fb963e42a6cea15821f1e4589502901187c3b2   （已改：3 处手改 + relabel-item.py 13 处）
CLAUDE.md
  开工 9c842c8d2adac61d24bd33574b39e6d21080dced9db78dfed4039cc487a79769
  收尾 9c842c8d2adac61d24bd33574b39e6d21080dced9db78dfed4039cc487a79769   （未改，与开工一致——规格外写范围，没写）
```

**只有收尾哈希的 55 个文件**（relabel-item.py 的写入 footprint 在开工时不可知，见下「试跑观察」第 3 点；
其中含 `.claude/kb/decisions-history/2026-09.md`——我知道要编辑它、但漏在开工时一并记哈希，是本轮流程上的疏漏，
同样列在「试跑观察」）：

```
d71dd5e93f0cf7ab10d3535636e619c7c41c153c2ac11865a83a81eaccf0ff70  .claude/kb/checks-owed.md
362676dc22e0ecc362aaa1407a5b3413404940f3d7cc222cf2406dd9523fa19a  .claude/kb/decisions-history.md
b0aa46365466dc49bac56a837f8d4ce2d5862189ea92dbfff8fa175f53e1c2e7  .claude/kb/decisions-history/2026-09.md
989910d45bdcf5a606405de0e8100a896059b5ae8f5afd39262a701ed57fc2c6  .claude/kb/decisions/03-空间分配.md
5859569f95f5235718549792c6c08a9fcd6b565bfebce48324143809d79185f5  .claude/kb/decisions/05-快照-空间记账机制.md
df69c00c7eb19f5f530942b91a2a3db008e88ece0c5a5c218fc3939c7fbaca53  .claude/kb/decisions/08-核心索引结构.md
902cc25eba6f4035cb5b6bdc096ac7c452bdcbaf1ee756eea2f37704040adc84  .claude/kb/decisions/09-加密.md
04ff11c740d967ff4da59ac8ed60334c7fe855026fd4358d168d51313c634f85  .claude/kb/decisions/15-格式冻结政策.md
9a473433fa203df5364b553447a3a2c53b619ff61db8968ce23e5720f90fd3c7  .claude/kb/decisions/16-发布语义.md
79d7afcb9e3ac8dc0ccaaf70b30d82c65053b4771a67c64a27781580f2c5146f  .claude/kb/decisions/18-块里携带什么信息.md
3fe80fda52a990efa45d1de26f3daddcb2014ccf8a52343f8ba6421e1f67330f  .claude/kb/decisions/19-块指针的结构与宽度预算.md
b144f2b39d8e7b36ef39598be03181445f74696863aa2c618b0da9a3e28b3a3a  .claude/kb/decisions/21-权威态与派生态的分界.md
87f51419d7d14d7d13b1278e01ea1f06488e4d043ea24356fb60adb5c3d00ef0  .claude/kb/decisions/23-journal的角色与格式.md
0324571357c60f8f3928c11eb54a4765f9ba35d6d352cea97f5478cfc81173d5  .claude/kb/decisions/28-挂载期承诺量.md
db2183f11dccec290f834529da11dc28a3b7f4e5b03e287f9c9f73485d9c7102  .claude/kb/experiments-history.md
8014e124b23b7abcdc0bc7c968a2d4ce28328e39d19f3725a24519778387870f  .claude/kb/experiments/100-超级块的三段几何.md
1361579a20b105b831c37f79aa9bf6d7963fb46d3bdde962f527f4e95ea9b6ed  .claude/kb/experiments/124-超级块字段表按已定项14重算.md
e1426e05ba0514c883cd252ff6cde0eb0711302889d9dcdb875d079e8897f96c  .claude/kb/experiments/131-livelist的两条载体数值与代价.md
e92f3e6441f4af47464cfd00fb1fd2a90c4604f999f55bc2cb9eac7709728d5c  .claude/kb/experiments/132-livelist载体按真实树数与内部扇出重算.md
f2bba4bb55fd9fac0122eb1becaaf66231ce1d385af2d3e74c4fa7a08f94df08  .claude/kb/experiments/142-第一个事务的干跑.md
2c8d0e0686356539c72da1c5ae266f08832f9a5c0d22855c4c26948930de87e5  .claude/kb/experiments/73-节点key区间的扇出代价.md
bc6c98b9e18dcde8c73857292095ea8298d1ded4f36ac8921cdc23aa0603d999  .claude/kb/experiments/74-分配记录的条目数与整理开销.md
dffab10ae4bffee56bf184d842389361f317a4fba5565cf0ac5d725a6f5ff37e  .claude/kb/experiments/79-根记录的容量.md
153e6e0ac24fdaedb9e8fc05dfe566af806f4f2bdcc6376ad96620a6d643e512  .claude/kb/experiments/97-记账与分配记录的条目编码.md
08872acdd79ec3c6a1ffa7f3bdfcf9c7c99853d0a0683dec0efed86e02341eed  .claude/kb/experiments/99-writebuffer条目与seq的去重.md
7a4253628c6eb1954ad4e87209a88801f35ffdbb6c5cf9c181c0afbae032b41a  .claude/kb/first-txn-layout.md
40a71f9decf44ce100927047555d6db31ab33c468a60e6b2d01a8fe84750577b  .claude/kb/milestone/01-first-txn.md
d65462ae2a6684b137f720738475354a15eda921f38e0d37643ad998a0d789a3  .claude/kb/milestone/02-second-txn.md
b6d2ea9b0c22c9af8d0d1652d26a0c85761a15a11ef6cdb31def270712c7071e  .claude/kb/second-txn-layout.md
6c5521c66fba283d6e199ecdc6a2ce75d22bf9ff5d675fffa9ecdd4f35033e3b  .claude/kb/verification-build.md
fe6677ebd33e1e25423f6a9d3be4e44143e21ab6b7a07238869d33d47ffda646  .claude/rules/three-way-inference.md
344ed8cf941c11449dfe88c59dd789fae70e0170cf10f81f7f7e50583133e934  .claude/kb/decisions.md
caad4e34374a3ba83480c5ddc3ce449143498c12af6bab8c5c58f1f369f2ab45  records/2026-09-05-C113定案.md
e0d1487e8cd3cb8857a2ff8ae68ce7c2d5d5f6ea42a33013aee7bdfaef77d9f8  records/2026-09-06-D1回扫重判.md
17f7a9b2f094633f61df510621427ad852e6736278d0328bc59c6a9475ee4c2c  records/2026-09-06-D2项12三轮对抗.md
16086d275c90ee3e3d228257a74e0f65d57329aec67cd8363db77eb72566b16b  records/2026-09-06-D8项8第三轮对抗.md
7426f73a840197adfc2fa74a6936dc5f24a605a4e0f94dc2c41dfb55507887dd  records/2026-09-06-D8项8第二轮对抗.md
4626e03a1383afe5c81ddfdac3ee8cea2c04c4f7b3f3508a31f62fdd1cb2a83d  records/2026-09-06-树ID水位臂比较重做.md
aa6200983eaf64c0f146f4021e8e76df87ad29e157a7fe2a67e3b3cf05713fb0  records/2026-09-10-D26项4项5定案推演.md
601d9b876a8fec6dccaff9f87437ddb456fcb99394f0ff1ea321680facb15625  records/2026-09-13-总审核.md
82d6e54c81f9c0e9327d7c6dfd55309ad2a9bb821c123a28b14ca9caf27b1d8b  research/e7-index-bench/src/bin/e100_superblock_slot.rs
00068cccebd6977da091e0082ff074270a54684bbc05bfbaa12dc1708337c65e  research/e7-index-bench/src/bin/e115_superblock_completeness.rs
4307d8280eb5edd6ac1d74e3e842029be52c0f5f1e1591902a69ce49e9ecc259  research/e7-index-bench/src/bin/e124_superblock_recompute.rs
6c3b9ec6f8ae9a325d75be01a11e0d5c3aa2d93267da043645a740ca14d18375  research/e7-index-bench/src/bin/e126_superblock_slot_width.rs
5f8889822906fa2abe92d9d071e24178cacd14c18118ef049578771a125e3eb9  research/e7-index-bench/src/bin/e132_livelist_carrier_recount.rs
120a07782c647d502711768dd1c8242de78b6c59d057b0ab6d913d755b266c79  research/e7-index-bench/src/bin/e133_map_key_format_cost.rs
944ce8b22ccaf53a76ef56bdf51d9fa22c9276bf97ea52c1e5e3a648939fc920  research/e7-index-bench/src/bin/e134_map_key_slot_baselines.rs
7dc26929b73d83433f9b4c54a676d66c8fdaac6c3f9326fa7ac3fd1ab764d536  research/e7-index-bench/src/bin/e135_rollback_floor.rs
5a9d2a53b88472c1a3712a9bba71679f2306813b140fbb157efe717ca3e09ff7  research/e7-index-bench/src/bin/e136_fork_cost_rows.rs
07fadbbff2abfc7f181a49f3548ceb5110ec95d7079a35cbe9ab8730f65699cf  research/e7-index-bench/src/bin/e138_per_disk_floor.rs
138b9bdc62d30f49057c132ad4f57ed5448b014b96ebab2ecbffc106ca9abbf0  research/e7-index-bench/src/bin/e139_tightened_floor.rs
32e1537b92acde4289172cd9614254d8b7cbf51eda92e23d5965cd28995be9a6  research/e7-index-bench/src/bin/e141_switch_reserve_mount_admission.rs
d7b26357b2f43e9b6fd131c27e7b23c78b33d365c84c7608e1ca7024b794ddd6  research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs
4259590e822728cd0906a9aa1ad4793331e190d776f52712ba537d8f1a53207b  research/e7-index-bench/src/bin/e146_livelist_entry_width.rs
1b4ecc3a7331c56b72d38a3ebf25abd9e7268fc43d0c07780e911be5aa4b889c  research/e7-index-bench/src/bin/e97_entry_encoding.rs
a93c9be017a58462d957363ef82dba6aa1ef7a24c0a23ae7eb1eff216c2d2b16  research/e7-index-bench/src/bin/e99_writebuffer_sequence.rs
```

共 57 个文件被改：2 个有完整开工/收尾哈希对照，55 个只有收尾哈希（原因见「试跑观察」）。
用当前时刻的 mtime 交叉核过：这 57 个之外、当前仍显示为 modified 的文件（`git status --short` 共 318 行）
mtime 都早于本轮任何一次写操作（最新的也在 11 分钟以上），是副本里已经存在的、别的会话留下的未提交改动，
本轮没有碰它们，照定义「不动、不碰」处理。

## 11. 没做什么

- 没判定案对不对——「D22 已定项 7 翻回未定」是主 agent 给定的虚构规格，没有核实过它是否真的该翻。
- 没写规格外的句子——CLAUDE.md 那条没写；D22 新增的未定项行没有补「状态：」标记；D22 标题的条数没有跟着改；
  未定项 7 没有补「改第一个事务的字节」判断——这四处都需要规格没给的句子或判断，一概没做。
- 没修不是这一轮的红——本次三个红（20、31、33）经核实**都是**这一轮写出来的，
  但因为修法都需要补规格外的内容，仍未修，原样列给主 agent（不等于「不是这一轮的」那类不修，是「是这一轮的但不能补内容」那类不修）。
- 没提交——没有 `git add`、`git commit` 或任何 git 写操作。
- 没跑 `gate.sh` 全量，没有编译 Rust（33 号红之后没有跑 `mutate.sh` 去验证或修复变异表）。
- 没碰 `.claude/kb/decisions-history/2026-09.md` 之外的别的会话未提交改动（`git status --short` 里另外 261 个
  非本轮改动的文件，逐个确认 mtime 早于本轮，未读、未碰）。

## 12. 试跑观察

**1. CLAUDE.md 出界判据可直接执行。** 定义原文点名 CLAUDE.md 作为写范围外的例子，规格恰好给了这个场景，
判断与停写都很顺畅，没有歧义。这条按定义原样走通了。

**2. 相邻两条红（20-kb-shape、31-blocking-verdict）暴露的是规格格式本身的缺口，不是流程说不清楚。**
主 agent 给的「新串」是把「一行 kb 表格该长什么样」的判断也一起做掉的产物——但这次给的两处新增行都没有配上
`**状态：未定。**` 结尾标记（其它未定项行都有），也没有给「改第一个事务的字节」的判断，而这两样合起来
正是 20 号与 31 号两个门禁阶段要的东西。这次 D22 文件顶部整条标题里的计数（「一项未定」）同样没被规格覆盖到。
`kb-scribe.md` 的「照规格写，不补内容、不做判断」在这种情况下把我推向「停下不修」，这是对的方向；
但如果这类翻状态的规格以后还会出现，主 agent 在写规格时可能需要多带三样东西才能让门禁一次过：
（a）新增分项行末尾的「状态：」标记，（b）该分项是否阻塞第一个事务字节的判断句，
（c）决策文件顶部标题行里的计数是否要连带改。这三样都不是书记员能自己补的（都需要判断或新句子），
但如果规格里不给，这条翻状态操作就必然会在 20/31 号留红——这次的三条红里有两条属于这一类，值得记下来。

**3. `sha256sum` 「开工时」步骤和 relabel-item.py 的级联写入范围互相冲突，且这次我自己还漏了一步。**
定义第 1 步要求「开工时先记下每个要改的文件的 sha256sum」，但规格本身只点了 2 个文件；真正会被改的另外
55 个文件（relabel-item.py 的级联改写 + 两个 `--write` 门禁脚本各自的产出），要跑过 `--dry-run` 之后才现形——
这在时间线上晚于「开工时」。这次的处理是只给规格点名的 2 个文件留了开工哈希，另外 55 个只有收尾哈希。
另外，`decisions-history/2026-09.md` 是我从任务输入里就知道要编辑的文件（不靠 relabel-item.py 才发现），
却也漏在开工时记哈希——这是我自己的疏漏，不是定义的问题。**建议**：把「开工时」改成两次记录点更准确——
第一次在规格给定的文件上（含要写决策变更史的那份月份文件），第二次在 `relabel-item.py --dry-run` 报出完整
文件清单之后、真正写之前；这样才能给全部改动过的文件都留下真实的「改前」哈希。

**4. 我自己在核实归属时的一处流程失误：第一条探索性命令用了 `head -50`/`head -20` 截断 `git status --short`
的输出，导致后续拿它当「开工基线」时漏掉了副本里本来就存在、但排在 50 行之外的大量未提交改动（这个副本显然
在派发前已经带着一大批别的工作，例如 `research/prompts/m2-step6-checker-r1-*`、`crates/singlefs-core/src/mount.rs`
等）。这与 `agent-common.md`「输出不截断，嫌长先数」直接冲突。后来用 mtime 交叉核实补救（确认那些文件的
最后修改时间都早于本轮任何一次写操作），没有造成误判，但这一步本可以从一开始就用不截断的输出、或者先
`wc -l` 数一遍再决定要不要截断来避免。记在这里，供后续会话按这条规则收紧自己的探索性命令。

**5. `decisions-history.md` 汇总卡片「只留最近 3 次」是按设计工作，不是 bug。** 49 号 `--write` 把 D22 卡片
里最旧的一条（2026-09-13 总审核那条）从「最近 3 次」窗口里挤掉，原文本身在 `decisions-history/2026-09.md`
里没有被移动或删改，只是不再出现在汇总卡片里——这是 `decisions-history.md` 文件头写明的设计（只显示最近 3
次，更早的去按月原文查），核对 diff 时一度需要确认这不是误删，值得在下一次派发时提前说明，省一次核对。
