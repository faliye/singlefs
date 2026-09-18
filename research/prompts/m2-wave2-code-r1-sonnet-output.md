# m2-wave2-code-r1 · Sonnet 正推腿报告

分到的格：Z4 的条款那一半（V4 两条新判定是不是 `.claude/kb/invariants.md` 那两行逐字说的）、Z5（改动计数与区域表）、Z6（写回收口表若干行）。不判 Z1/Z2/Z3/Z4 的算术那一半，不替攻方找反例。

所有行号均在对应文件里用 `grep -n` 现查，不从背景材料 `_m2-wave2-code-r1-background.md` / `_m2-wave2-code-r1-diff.md` 数。

## 一、Z4：两条新判定是不是 invariants.md 逐字说的

### 1.1 I-3.9 区间端点——一致

`.claude/kb/invariants.md:131`（grep -n 命中）：
> 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。

`crates/singlefs-checker/src/walk.rs:1005-1022`（`judge_release_generations`）：
```
let Some(last_referencing_txg) = scanned.iter()
    .filter(|root| root.references.placements.contains(&placement))
    .map(|root| root.checkpoint_txg).max() else { continue; };
...
let first_root_without_it = scanned.iter().map(|root| root.checkpoint_txg)
    .filter(|txg| *txg > last_referencing_txg).min();
let in_the_witnessed_interval = record.generation > last_referencing_txg
    && first_root_without_it.is_some_and(|txg| record.generation <= txg);
```
`last_referencing_txg` = 仍引用它的根里 txg 的 max（= kb 的左端点 L），`first_root_without_it` = 全部根里 txg > L 的 min（= kb 的右端点 T，因为 max 的定义保证任何 txg > L 的根都不再引用它）。判定 `generation ∈ (L, T]`，与 kb 逐字一致。

### 1.2 跳过「回收过还没复用」类记录——一致

`invariants.md:131`：
> 已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。

`walk.rs:1005-1013`：`last_referencing_txg` 求不到（`else { continue; }`，即候选集里没有一条根引用过这个落点）就跳过该记录、不进 `witnessed_records` 计数。`walk.rs:1036-1041`：
```
if witnessed_records == 0 {
    judgements.not_applicable("I-3.9", "带已释放标志的记录一条都没有候选根引用过：见证它们释放的根已不在候选集里");
}
```
与 kb 逐字一致（「全是这一类时整条报不适用并带理由」）。

### 1.3 I-9.14「只出现在一个树表单元里报不适用」——一致

`invariants.md:274`：
> 同一棵树的条目只出现在一个树表单元里时报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。

`walk.rs:1066-1096`（`judge_tree_table_birth_txg`）：
```
let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings.iter().map(|s| s.tree_table_placement).collect();
if distinct_tree_tables.len() < 2 { continue; }
...
if compared_trees == 0 {
    judgements.not_applicable("I-9.14", "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来");
}
```
按树表单元坐标（设备+槽号）去重，少于 2 个不同单元就跳过该树、不计入 `compared_trees`；全部树都是这样就报「不适用」。与 kb 逐字一致。


### 1.4 规则没说的一处：候选集 < 2 时两条都提前报「不适用」

`walk.rs:1099-1117`（`judge_release_generation_and_tree_table_birth` 开头）：
```
/// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
/// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（2026-09-18 用户定案随 C374 立条时定的口径）。
fn judge_release_generation_and_tree_table_birth(...) {
    if candidate_indexes.len() < 2 {
        judgements.not_applicable("I-3.9", "回退候选集里只有一条根：没有第二条根能见证「不再引用这个落点」");
        judgements.not_applicable("I-9.14", "回退候选集里只有一条根：树表条目没有第二条根的那一份可比");
        return;
    }
    ...
```
`invariants.md` I-3.9 那一行只写了「已回收未复用」触发不适用的条件，I-9.14 那一行只写了「只出现在一个树表单元里」触发不适用的条件；两行都没有字面写「候选集只有一条根 ⇒ 两条都不适用」这个分支。代码注释自己标了来源（「2026-09-18 用户定案随 C374 立条时定的口径」），是一次没有写回 kb 正文的口径决定，判定：**规则没说**，不是冲突——对 I-9.14 而言这个早退在语义上是「只出现在一个树表单元里」的特例（候选集只剩一条根时，任何树的条目都只能出现在这一条根指的那一个树表单元里），我推得出它不改变判定结果，但没有去构造反例验证，这一步按「不替攻方找反例」不做。
**推翻条件**：若能构造一个候选集恰好 1 条根、而按 kb 现有的两条「不适用」触发条件都不成立（即树表条目分布在多个单元里，或存在被见证的已释放记录）的镜像，此处的早退分支会让 checker 报「不适用」而 kb 字面本该判「成立/违例」，那就是真冲突，不只是「规则没说」。

### 1.5 代码自己的注释与今天的 invariants.md 字面不同——这是一个过时注释，不是一个仍然存在的冲突

`walk.rs:963-972`（函数头注释，紧接在 `judge_release_generations` 定义之前）：
```
/// I-3.9（释放代等于最早不再引用它的有效根）：...
/// ⚠️ **判的是区间，不是条款那一句的「等于 T」**：...
/// 环里 L 与 T 之间没有洞时 `(L, T]` 只有 T 一个值，与条款那一句逐字同义；C374 点名的判别力（发布 B 那次的释放代从 4 写成 3）
/// 两种写法都红。**这一处与 `invariants.md` I-3.9 的字面不同，等用户定案**（实现员 2026-09-18 交主 agent）。
```
这段注释自己给这条函数起的括注名是「释放代**等于**最早不再引用它的有效根」，并明说「这一处与 `invariants.md` I-3.9 的字面不同，等用户定案」。但今天 `invariants.md:131` 的登记名已经是「I-3.9（释放代**落在**停止引用它的那一格区间里）」，正文逐字就是区间定义（1.1 节引文），与代码的计算逻辑（1.1 节）完全一致——没有字面不同。背景材料「一、被判的对象」也写着「I-3.9 判区间（不是「等于」——主 agent 原措辞被残留记录那条流的 12 个合法状态证伪）」，说明 kb 正文已经改成区间口径。**这条注释是写完之后没有跟着 kb 定案回改的旧注释，今天读起来会让人误以为 kb 与代码还有分歧，而实际没有。**
**推翻条件**：若能找到 `invariants.md` 历史上任一个「今天生效」的版本仍用「等于 T」而不是区间，且代码在那个版本发布之后才改成区间但没回填这句注释，那就说明这条注释当时是对的、只是没跟着后续修订更新——结论仍是「过时」，只是过时的具体时间点会变。


## 二、Z5：改动计数与区域表

### 2.1 E142 报的 67 段——命令核实，逐字一致

```
$ grep -n 'name=change_count_diff' research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out | wc -l
67
$ grep -n 'name=diff_explained_summary\|name=extra_structures' research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out
181:E7RESULT name=diff_explained_summary segments=67 unexplained=0
182:E7RESULT name=extra_structures structures=journal_record,mapping_root count=2
```
背景材料「一、被判的对象」写「E142（第一个事务的干跑） 第十一次跑量出盘上共 67 段字节跟着变、全部归因，另有 journal_record 与 mapping_root 两个结构也变」——`segments=67 unexplained=0`、`structures=journal_record,mapping_root count=2` 三个数逐字对得上。这份产物是 `research/prompts/e142-r11-prereg.md`（第十一次跑跑前登记）对应的实跑输出，时间戳 2026-09-18 02:38 UTC，晚于诊断快照与 diff 生成时间（02:28 UTC）几分钟，属于同一轮的产物。
`name=change_count_diff` 逐段按 `structure=` 归类（`grep -oP '(?<= )structure=[a-zA-Z0-9_]*'`）：inode_leaf 6、inode_root 8、journal_record 28、mapping_root 12、root_record 5、tree_table 8，合计 67——没有漏算的结构。
**推翻条件**：若这份 `.out` 文件不是这一轮 diff 对应的真实运行产物（例如是伪造或复制自另一次不同参数的跑），则以上数字不能证明背景材料的「67 段」claim；核对时间戳与文件名（含 `change-count-three`）与跑前登记 `e142-r11-prereg.md` 第五节臂 A 的定义一致，未发现这种迹象。

### 2.2 21 行区域表与 `layout/01-first-txn.md` 零那一节逐行对得上

`crates/singlefs-harness/src/first_transaction_regions.rs:132-179`（`FIRST_TRANSACTION_REGIONS`）21 行常量表，与 `.claude/kb/layout/01-first-txn.md:67-77`（零那一节 t1..t11 那张表，行号 grep -n 现查）逐项核对：

| 表里的行 | 零那一节对应的槽号（文件自己的行号） | 代码里的常量 |
|---|---|---|
| data_unit（两盘） | t1：50180–50181（layout:67） | `DATA_UNIT_SLOT = 50180`（regions.rs:50） |
| extent_root（两盘） | t2：50240（layout:68） | `EXTENT_ROOT_SLOT = 50240`（regions.rs:51） |
| inode_leaf（两盘） | t3：50242–50243（layout:69） | `INODE_LEAF_SLOT = 50242`（regions.rs:52） |
| inode_root（两盘） | t4：50244（layout:70） | `INODE_ROOT_SLOT = 50244`（regions.rs:53） |
| allocation_root（两盘） | t5：50245（layout:71） | `ALLOCATION_ROOT_SLOT = 50245`（regions.rs:54） |
| accounting_root（两盘） | t6：50246（layout:72） | `ACCOUNTING_ROOT_SLOT = 50246`（regions.rs:55） |
| mapping_root（两盘） | t7：50247（layout:73） | `MAPPING_ROOT_SLOT = 50247`（regions.rs:56） |
| tree_table（两盘） | t8：50248（layout:74） | `TREE_TABLE_SLOT = 50248`（regions.rs:57） |
| root_record（仅设备 0） | t10：区域 `3 mod 3`=0 的槽 1（layout:76） | `FIRST_TRANSACTION_ROOT_RING_REGION = 0`、`..._SLOT_INDEX = 1`（regions.rs:42-43） |
| journal_record（两盘） | t9：环内偏移 8192（layout:75） | `FIRST_TRANSACTION_JOURNAL_RECORD_RING_OFFSET = 2 * JOURNAL_RECORD_BYTES`（regions.rs:47，= 8192） |
| superblock（两盘） | t11：超级块槽 1（layout:77） | `FIRST_TRANSACTION_SUPERBLOCK_SLOT_INDEX = 1`（regions.rs:40） |

16（8 单元 ×2 盘）+ 1（根记录仅一盘）+ 2（journal 两盘）+ 2（超级块两盘）= 21，与 `layout/01-first-txn.md:79`「⇒ **写请求数**：...⇒ **21 条**」逐字对得上。判定：**一致**。
**推翻条件**：若这 11 组常量里任何一个与 `layout/01-first-txn.md` 零那一节对应行的槽号 / 偏移不同，或者行数不是 21，此处即为冲突。


### 2.3 换几何时 `region_table_against_writes` 拦不拦得住——这一次跑，拦得住（正例）

`crates/singlefs-harness/src/first_transaction_regions.rs:270-308`（`region_table_against_writes`）把表与「第一个事务真正发出的写」按 `(设备, 偏移, 长度)` 多重集一一配对，多出来的写或没配到的表行都会被列出来、`matches()` 才报 true。这一轮实跑：
```
$ grep -n 'name=impl_region_table_against_writes' research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out
308:E7RESULT name=impl_region_table_against_writes regions=21 write_calls=21 regions_without_a_write=none writes_outside_the_table=none matches=true
```
21 行表、21 次写调用、没有表里没写到的行、没有写到表外的落点——`region_table_against_writes` 在这一次几何（2 盘、写行 3000 字节）上确实拦得住漂移（若表少一行或多一行、或代码多写/少写了一次，`write_calls`、`regions_without_a_write`、`writes_outside_the_table` 三者之一会不等）。**但这只是一次几何点上的正例**，判据表 Z5 问的「换几何时拦不拦得住」在换了几何（比如盘数、文件大小）之后重跑一次得到的是不是同样的 `matches=true`，这一份产物没有覆盖，我没有另外的几何点产物可核，记「复核不了」。

### 2.4 一处意外发现：M_A（装置模型）与 M_C（实装）在 21 个区域里有 11 个字节不相等，产物里没有解释

这不是 Z5 判据表字面问的问题（判据表问的是表与 kb 逐行对不对得上，以及 `region_table_against_writes` 拦不拦得住），但它是同一份产物、同一套「量 5」（`e142-r11-prereg.md:168` 行）量出来的，直接牵动「21 行表」这份材料的可信度，按「什么观测会推翻它」的要求一并写出：
```
$ grep 'name=impl_bytes_equal' research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out
... （21 行，逐行见下）
E7RESULT name=impl_bytes_equal_summary regions=21 equal=10 unequal=11 snapshot_given=true
```
逐行原样：data_unit 两盘 `equal=false first_diff_offset=11 mismatch_bytes=3`；extent_root 两盘 `equal=false first_diff_offset=10 mismatch_bytes=4`；mapping_root 两盘同上；tree_table 两盘同上；root_record（仅设备 0）`equal=false first_diff_offset=96 mismatch_bytes=20`；journal_record 两盘 `equal=false first_diff_offset=46 mismatch_bytes=56`；inode_leaf / inode_root / allocation_root / accounting_root / superblock 十行全部 `equal=true`。
`e142-r11-prereg.md` 第十节 F1 写「装置与 `crates/` 实装在区域清单上对不上 ⇒ **停机**（两边都查，既不作废也不当结果）」，第三节写「其余各处由本轮的量 5 逐字节核」——暗含的期待是量 5 应当逐区域相等（本轮唯一预期会变的只有偏移 88 的改动计数本身，量 2 单独核）。这份产物里 11/21 个区域不相等，我在文件里没有找到任何一行 `name=gap` 或别的说明把这 11 处的差异解释掉（G1–G25 那批 gap 行，`grep -n 'name=gap'` 逐条读过，没有一条点名 `impl_bytes_equal` 或这几个 region 名）。**我没有去查这 11 处不等的具体原因**（需要跑 `research/e7-index-bench` 与 `crates/singlefs-harness` 两套代码逐字段比对，超出这一格判据要求的范围，也不是我该做的算术核实），只如实报告：F1 这条停机条件按产物字面已经触发，而我审读到的材料（背景材料、diff 附录、milestone 收口表）都没有提到这次触发、也没有交代两边查过没有。
**推翻条件**：若主 agent 或另一条腿能找到一份对这 11 处不等的显式解释（例如某个已知的、与改动计数无关的字段本来就允许模型和实装不同），且那份解释早于或同期于这次跑，那么这条「未交代」的判定就不成立，只是我没读到；若确实没有任何解释，这次跑按它自己的跑前登记本该报「停机」而不是径直报「够判」。


## 三、Z6：收口表（`.claude/kb/milestone/02-second-txn.md`）逐行核对

行号均 grep -n 现查该文件，未从背景材料数。

### 3.1 行 ②（`milestone/02-second-txn.md:311`）——一致，未受本轮 diff 影响

内容是 F 的「生效」事后回落、alloc-basis 岔路表等，与这一轮 V1–V6 的代码无关。核实它引的两份文件确实存在：
```
$ ls -la research/prompts/alloc-basis-r3-forks.md records/2026-09-17-已分配口径三方与两个实验.md
-rw-rw-r-- 1 fy5090 fy5090  3761 ... research/prompts/alloc-basis-r3-forks.md
-rw-rw-r-- 1 fy5090 fy5090 27411 ... records/2026-09-17-已分配口径三方与两个实验.md
```
判定：一致（这一行的状态本轮没有变化，符合它自己写的「等增补 1」）。

### 3.2 行 11（`milestone/02-second-txn.md:320`）——大体一致，但「还要」清单漏了一处

行文：「2026-09-18 用户定案：改代码写 3、重跑 E142……同日代码已改（`publish_first_file` 写 `FIRST_TRANSACTION_TXG`……变异「换回 1」红在「逐字段回读等于写入」）」。

代码核实：`crates/singlefs-core/src/transaction.rs:1172` `change_count: FIRST_TRANSACTION_TXG,`；`crates/singlefs-format/src/lib.rs:201` `pub const FIRST_TRANSACTION_TXG: u64 = 3;`——一致。
变异核实：
```
$ awk 'NR==113' crates/mutations.tsv
增补 2 第 11 行：第一个事务的改动计数写回 1（字段表定义是最后一次改动所在发布的 checkpoint_txg = 3）	crates/singlefs-core/src/transaction.rs	                change_count: FIRST_TRANSACTION_TXG,	                change_count: 1,	-p singlefs-harness --test first_transaction_step_five_publish -- inode_and_extent_lookups	inode_and_extent_lookups_from_the_root_read_the_first_file_back
```
测试名 `inode_and_extent_lookups_from_the_root_read_the_first_file_back` 就是「逐字段回读等于写入」——一致。
E142 重跑核实：`research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out`（2.1 节）已经是这次重跑的产物，且 `name=width` / `name=segments` 与上一次跑（`...2026-09-17-genesis-tree-table-released.out`）逐字相同：
```
$ diff <(grep '^E7RESULT name=width' research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out) <(grep '^E7RESULT name=width' research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out)
$ diff <(grep '^E7RESULT name=segments' research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out) <(grep '^E7RESULT name=segments' research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out)
```
两条 diff 都无输出（逐字相同）。**这份「重跑」本身已经落盘完成**，但收口表这一行的「还要」列仍写着「E142（第一个事务的干跑） 留存产物按偏移 88 那 8 字节重跑……」——按今天的状态这一条已经做完，表没有跟着更新（这张表最后一次「重核」是 2026-09-17，见 `milestone/02-second-txn.md:304`，早于这次重跑，是时间差不是错误）。

**漏掉的一处**：`.claude/kb/decisions/08-核心索引结构.md:430`：
```
一条记录（inode 1、出生代 1、locality 0、nlink 1、改动计数 1、三段 0）；记账里一条「inode 号水位 = 2」。
```
这句是 D8（核心索引结构） 已定项 6「第一个事务写」段落里的举例，仍然写着「改动计数 1」，与已定项 6 自己的字段定义（`decisions/08-核心索引结构.md:403`「最后一次改动所在发布的 checkpoint_txg」= 3）不符——与 `layout/01-first-txn.md:257` 那一格（同样写「1」）是**同一类**未跟进的旧值，但行 11 的「还要」清单只点名了后者（书记员改 `layout/01-first-txn.md`），没有点名 `decisions/08-核心索引结构.md:430` 这一句举例。`milestone/02-second-txn.md:113`（步 3 决策点原文）与 `milestone/02-second-txn.md:509`（历史记录）也都只描述了「代码与 E142 都按 1」这件旧事，同样没有提到 `decisions/08:430` 这处举例文字要跟着改。
**判定：冲突（收口表「还要」清单不完整）**。
**推翻条件**：若能找到收口表别处或另一份欠账记录已经单独点名 `decisions/08-核心索引结构.md:430` 要跟着改成 3，则这一条不算漏项，只是分散记录；我在 `milestone/02-second-txn.md` 全文与 `checks-owed.md`（这次没有整份通读，只 grep 了「改动计数」「430」）里没有找到这样的记录。


（补：`.claude/kb/checks-owed.md` 里 grep `08-核心索引结构.md:430` 与 `改动计数.*430` 均零命中，没有另立欠账记录这一处。）

### 3.3 行 12（`milestone/02-second-txn.md:321`）——一致

行文：「2026-09-18 用户定案：立不变量 + checker 实现（不是只加两条变异）」「主 agent 给不变量措辞 → kb-scribe 写进 invariants.md → 实现员做 checker 判定与两份坏镜像 → sweep 按「新立一条判据」回扫已有条目」。
核实：`invariants.md` 已加 I-3.9 / I-9.14（一、1.1/1.3 节）；`crates/singlefs-checker/src/image.rs:36-40`：
```
pub const IMPLEMENTED_INVARIANTS: [&str; 28] = [
    "I-1.1", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1",
    "I-3.8", "I-3.9", "I-4.8", "I-5.1", "I-5.2", "I-7.1", "I-7.2", "I-7.4", "I-7.6", "I-7.7",
    "I-7.8", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13", "I-9.14",
];
```
28 条，含 I-3.9 与 I-9.14，与背景材料「一、被判的对象」写的「IMPLEMENTED_INVARIANTS（26 → 28）」在「今天是 28」这一半吻合（「26」是变更前的数，diff 附录未附旧版本，这一半我没有另外的旧文件核，标记为未独立核实，但与今天的 28 不矛盾）。
两份坏镜像：`crates/singlefs-harness/tests/checker_known_bad_images.rs:642-682`（`known_bad_images_after_the_overwrite`）两条 mutation，`:687-711`（`each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite`）测试函数逐一注入并断言「只红在自己那条」，与 `invariants.md:131` / `:274` 两行「已实现」状态列点名的测试函数名一致。判别力核实：I-3.9 的坏镜像把释放代 4→3（`rewrite_released_generations(bytes, 4, 3)`，断言命中 16 条记录），对应 kb 判别力「把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）」；I-9.14 的坏镜像把诞生 txg 3→4，对应 kb 判别力「把树表条目的诞生 txg 改成跟着这次发布的 txg 走必须红」——两处逐字对得上。
mutations.tsv 也各有一条（`awk 'NR==114'`、`NR==115`，均命中 `-p singlefs-harness --test checker_known_bad_images -- each_c374_bad_image`）。
**「sweep 按新立一条判据回扫已有条目」这一步，我没有找到证据证明已经做过**（`kb-discipline.md`「新立一条判据，当场拿它回扫已有的条目」那一节讲的是同一件事）：没有检索到任何一份 sweep 报告或收口表条目提到「回扫」的结果。**复核不了，如实记：未验**，不代表它没做，只是我没找到证据。


### 3.4 行 19（`milestone/02-second-txn.md:328`）——一致（作用域那半）

行文：「作用域那格已实现待代码三方……2026-09-17 实现员交回：发号器每次发布建一个，一个文件对象的单元抽成 `build_file_version_units`，单文件发布字节不变」。
核实：
```
$ grep -n 'fn build_file_version_units\|BirthSequenceAllocator::default()' crates/singlefs-core/src/transaction.rs
977:pub struct BirthSequenceAllocator {
1399:fn build_file_version_units(
1577:    let mut sequences = BirthSequenceAllocator::default();
```
`build_file_version_units` 函数存在；`BirthSequenceAllocator::default()` 在 `:1577`（发布路径内，不是结构体常驻字段）新建，与「每次发布建一个」一致。mutations.tsv 第 102 行也对应这处改动（「出生序号发号器挪回每装一个文件对象重建一次」，锚在 `build_file_version_units` 里 `let mut sequences_rebuilt_on_every_call = BirthSequenceAllocator::default();`）。判定：一致。目录进不进、写零两项交用户，未涉及代码，不在这次核实范围。

### 3.5 行 20a（`milestone/02-second-txn.md:348`）——一致

三条变异核实（逐条 `awk 'NR==行号' crates/mutations.tsv`）：
- 第 107 行：「写行那次发布的准入不在取号之前算」，锚 `refuse_publishes_before_acquisition_that_do_not_pass_admission(...)`，测试 `a_writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired`；
- 第 111 行：「暖机第 1 次不算」，锚 `warm_up_publishes_planned` 的 `chain`，测试 `a_writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired`；
- 第 112 行：「只把第一次暖机算进去」，锚同上加 `.min(1)`，测试 `a_writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired`。

三条与行 20a 写的「两条新用例（暖机第 1 次、第 2 次装不下各一条）、三条变异」在**变异条数**上一致（3 条）；两条新用例名（`a_writable_mount_whose_first_warm_up...` / `...second_warm_up...`）也在变异表的测试目标列里各出现一次，判定：一致。`MountError::WarmUpAdmissionRefusedBeforeAcquisition`、`PublishShape::EMPTY_PUBLISH` 在诊断附录一节 `mount.rs:31-112` 的 `MountError` 抽取块里能看到成员声明（诊断材料自身已回读核对，我这里不重复抄，只确认背景材料「一、被判的对象」点名的这几个符号名与诊断附录一致）。

### 3.6 行 20b（`milestone/02-second-txn.md:349`）——一致（结构层面）

`PoolWriter::writes_of_failed_publishes` 与 `count_failed_publish`：背景材料「一、被判的对象」表里点了名，diff 附录抽取了 `publish_without_units`（`:504-571`）与 `publish_admitted`（`:1562-2144`）两个外层函数覆盖两处 `persist` 闭包（这一处诊断由背景材料自陈「回读逐字节比对，退出码 0」，我未重新跑 `quote-rust-items.py`，按其自证结果引用）。变异核实：`awk 'NR==108'`：
```
增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去 ... if let Err(cause) = persist(pool) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        return Err(...)  →  若删掉 count_failed_publish 那一行 ... -p singlefs-harness --test second_transaction_supplement_one_write_accounting -- a_publish_that_fails_midway ...
```
与行文「失败的发布把这次已记的写单独交出去，与成功那次分开」一致。行文里点名的「用例钉『失败那次 11 次 / 245 760、重试 21 次 / 344 576，相加 = 录制器记的 32 次 / 590 336』」这几个具体数字，我没有跑测试去复核，属于能用命令核但我没有运行 cargo test 的那一类，标记复核不了（详见「没做什么」）。

### 3.7 行 20c（`milestone/02-second-txn.md:350`）——一致

```
$ grep -n 'cluster_segments\b' crates/singlefs-core/src/allocator.rs | head -8
499:    cluster_segments: BTreeSet<SlotNumber>,
519:            cluster_segments: BTreeSet::new(),
661:        let cluster_segments = &self.cluster_segments;
752:                self.cluster_segments.insert(segment_start);
883:    pub fn cluster_segments(&self) -> &BTreeSet<SlotNumber> {
```
字段与访问器都存在，与背景材料点名的「`cluster_segments` 与它的访问器」一致。变异第 109 行（`awk 'NR==109'`）把 `lowest_user_data_slot` 排除聚簇段的逻辑换回只排除当前开放段，测试目标 `user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed`——该测试函数在 `allocator.rs:1132` 确实存在：
```
$ grep -n 'fn user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed' crates/singlefs-core/src/allocator.rs
1132:    fn user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed() {
```
一致。「用例钉落点 50306」这个具体数字未运行测试核实，标记复核不了。


### 3.8 行 20d（`milestone/02-second-txn.md:351`）——机制存在，具体数字复核不了

「记账行「全空聚簇段数」只看已分配位……整段被隔离后记账说 3310 不变、分配器能开的最低段从 50304 跳到 50368」。核实机制存在：
```
$ grep -n '全空聚簇段数' crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocator.rs
crates/singlefs-core/src/transaction.rs:1348:/// 记账树里每块盘各一行的：已分配、空闲、不可回收、defer 待释放、碎片段数、全空聚簇段数。
crates/singlefs-core/src/allocator.rs:11://! 全空聚簇段数逐设备——全部在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。
```
这个统计量在代码里确有对应的注释与字段（增量维护、不扫盘），与「只看已分配位」的描述方向一致。但「3310」「50304 → 50368」两个具体数字，以及它与 C318（影子账隔离的单元没进准入不等式） 射程分不分得清这一句判断，都需要跑一段隔离场景才能核，我没有跑，标记复核不了。「跟第 ② 行」这句归类判定与行 ② 的现状一致（行 ② 也写「等增补 1」）。

### 3.9 行 21（`milestone/02-second-txn.md:331`）——一致

```
$ grep -n 'pub fn warm_up\b\|pub fn warm_up_after_journal_counter' crates/singlefs-core/src/transaction.rs
425:pub fn warm_up<Device: BlockDevice>(
446:pub fn warm_up_after_journal_counter<Device: BlockDevice>(
```
两个函数确实拆开存在，与行文「暖机拆成 `warm_up`（环空时从记录号 0 接着数）与 `warm_up_after_journal_counter`」一致。mutations.tsv 第 101 行对应同一处改动（锚 `counter: previous_counter + 1` vs `counter: txg_number`），测试目标 `warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two`，与「用例只在函数层给不相等的起点」一致。

### 3.10 行 22（`milestone/02-second-txn.md:332`）——列出的会红检查存在，逐条数字未展开核

这一行是一张「仍未销账」的清单，本身不含新代码改动。抽查两处可以命令核的点：
- 「C77（重放起点未定义） 最近……差把变异表第 97 行真跑一遍坐实」：`awk 'NR==97' crates/mutations.tsv` 确有一条锚在 `crates/singlefs-core/src/recovery.rs` 的 `scan_journal` 附近、测试 `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`，其「变异」列描述的是一段**还没出现在当前 `recovery.rs` 里**的假想代码（引入「先信 tail」逻辑）——`grep -n 'journal_tail\|\.tail\b' crates/singlefs-core/src/recovery.rs` 零命中，说明当前实现确实不信 tail（与行 22 引的 `recovery.rs:6` 注释「journal 全环扫描、不先信 tail」一致），这条变异是「准备去测但按行文还没真跑一遍坐实」的状态，与行文口径相符。
- 「C29（恢复先信 tail 会丢数据） 一处形态都没有（`recovery.rs` 的扫描恒走全环、函数体内不读 tail）」：`recovery.rs:6` 注释原文「journal 全环扫描、不先信 tail（D23（journal 的角色与格式） 已定项 3）」——一致。
其余七条欠账（C287、C330、C282、C22、C80、C281、C314）与「今天一条都销不掉」这句整体性结论，我没有逐条去查 `checks-owed.md` 对应条目现在的状态，标记复核不了（这七条本身不是这一轮 diff 改的东西，核实它们需要通读整份 `checks-owed.md`，超出这次分到的 Z4/Z5/Z6 三格能负担的量）。


### 3.11 行 23（`milestone/02-second-txn.md:333`）——一致

「`second_transaction_step_zero_layer0.rs` 两条新用例、`crash.rs` 按不变量报「评估过 / 判违例 / 不适用」、变异 6 条」。
```
$ grep -n 'fn residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it\|fn stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state' crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
811:fn residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it(
996:fn stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state(
```
两条新用例都在。
```
$ grep -n 'checker_evaluated_states\|checker_violated_states\|checker_not_applicable_states' crates/singlefs-harness/src/crash.rs
525:    pub checker_evaluated_states: BTreeMap<&'static str, u64>,
526:    pub checker_violated_states: BTreeMap<&'static str, u64>,
530:    pub checker_not_applicable_states: BTreeMap<&'static str, u64>,
```
三个字段对应「评估过 / 判违例 / 不适用」三态，一致。变异条数：
```
$ awk 'NR>=95 && NR<=100' crates/mutations.tsv | wc -l
6
```
第 95–100 行正好 6 条，且内容分别对应「步 0：崩溃镜像的记录提示漏掉基镜像里的记录」「步 0：恢复把没有回退行的所选根也按 W=0 封顶」「步 6：恢复从超级块 tail 起逐条验证……」「步 6：I-3.8 的判定没跑到」「步 6：I-2.1 的判定没跑到」「步 6：层 0 计数不记「不适用」」——都标着「步 0」「步 6」，与行 23「记在哪」列的「步 0、步 6」一致。判定：一致。

### 3.12 行 23′（`milestone/02-second-txn.md:334`）——现象存在，具体字节数复核不了

「I-3.1（已分配统计对得上） 在合法状态上判红……残留记录那条流 12 个状态差 65 536 字节」。「12 个状态」这个数字与 `invariants.md:131` 里 I-3.9 那一行自己写的「环上有洞时……实测 2026-09-18 残留记录那条层 0 流的 12 个状态」用的是同一个「12」——两处引用的都是同一条「残留记录」崩溃流，数字互相印证、没有矛盾，但这是同一个数字在两处被引用，不构成两条独立证据；「差 65 536 字节」我没有跑测试复核，标记复核不了。行 23′ 自己也写「今天走得到」「口径冲突……但它不等增补 1 就在错」，是一句待用户判的开放状态描述，不是一句「已经解决」的断言，我读到的是它准确描述了自己是「未决」，判定这一点上一致。

### 3.13 行 23″（`milestone/02-second-txn.md:335`）——一致

```
$ grep -n 'record_claimed_state_missing_unit' crates/singlefs-harness/src/crash.rs
523:    pub record_claimed_state_missing_unit: u64,
790:        tally.record_claimed_state_missing_unit += 1;
```
字段与递增点都在，行文「记录核对器第二条判据（`record_claimed_state_missing_unit`）不认合法复用」指名的符号存在。「陈旧 tail 那条流 8 个状态都误报」这个数字未跑测试复核。

### 3.14 行 28（`milestone/02-second-txn.md:340`）——一致

```
$ grep -n 'AccountingEntriesExceedOneNode' crates/singlefs-core/src/transaction.rs
931:    AccountingEntriesExceedOneNode {
1338:        return Err(PublishError::AccountingEntriesExceedOneNode {
```
错误成员存在。变异第 103、104 行分别测「80 块盘」与「79 块盘」两个边界：
```
103: 增补 2 第 28 行：记账树装不下不判（80 块盘走到装节点的断言 panic） ... eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written
104: 增补 2 第 28 行：记账树正好装满 477 行也判装不下（79 块盘发布被拒） ... seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish
```
测试名与行文「记账树报错成员 2026-09-17 已实现待代码三方（`AccountingEntriesExceedOneNode`，79 块盘 477 行照常、80 块盘报错且盘上不变）」中「79 块盘照常」「80 块盘报错」两个边界逐字对得上（`seventy_nine_devices_fill...and_still_publish` 对应「照常」，`eightieth_device_overflows...refused_before_anything_is_written` 对应「报错且盘上不变」）。「树表那一半没做」这半句没有找到反例，判定：一致。

### 3.15 行 34′（`milestone/02-second-txn.md:347`）——一致

```
$ grep -n 'FIRST_TRANSACTION_ACCOUNTING_ROWS' crates/singlefs-format/src/lib.rs
114:pub const FIRST_TRANSACTION_ACCOUNTING_ROWS: u64 = 15;
```
值 15，与行文「每次发布装 15 行（`FIRST_TRANSACTION_ACCOUNTING_ROWS`）」一致。「两盘 30 行 × 25 代 × 34 字节装不下一个 16 KiB 节点」这几个数字属于按 D5（快照 / 空间记账机制） 已定项 2 的算术推演，不是这一轮 diff 改的代码，未独立复核。


## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z4·I-3.9 区间端点 | 一致 | `walk.rs:1005-1022` 的 `(L, T]` 计算与 `invariants.md:131` 逐字同义 |
| Z4·I-3.9 跳过回收未复用 | 一致 | 找不到见证根就 `continue`、全跳过则报不适用，`walk.rs:1005-1041` 对得上 `invariants.md:131` |
| Z4·I-9.14 只出现一个树表单元报不适用 | 一致 | `walk.rs:1066-1096` 按树表单元坐标去重、少于两个就不比，对得上 `invariants.md:274` |
| Z4·候选集 <2 早退分支 | 规则没说 | `walk.rs:1108-1117` 这个分支的判定结果（不适用）没有字面写进 kb 两行，是一次没回写 kb 的口径决定；语义上大概率是既有条款的特例，未构造反例验证 |
| Z4·代码自注释「与 invariants.md 字面不同」 | 过时（不是仍然存在的冲突） | `walk.rs:967-972` 这句注释写的是修订前的 kb 口径，今天 `invariants.md` 已经是区间口径，注释没跟着回改 |
| Z5·67 段差异 | 一致 | `e142-first-txn-dry-run-2026-09-18-change-count-three.out` 的 `segments=67 unexplained=0`、`extra_structures ... count=2` 与背景材料逐字对得上 |
| Z5·21 行表 vs 零那一节 | 一致 | `first_transaction_regions.rs:132-179` 的 11 组常量与 `layout/01-first-txn.md:67-79` 逐项对得上，21 = 16+1+2+2 |
| Z5·`region_table_against_writes` | 一致（单一几何点） | 这次跑 `matches=true`（21/21）；换几何后是否仍拦得住没有产物可核 |
| Z5·M_A/M_C 21 区域字节比对 | 附带发现，未在材料里看到解释 | 同一份产物里 11/21 区域 `equal=false`，跑前登记自己的 F1 停机条件按字面已触发，我读到的材料没有交代 |
| Z6·行 ② | 一致 | 未受本轮 diff 影响，引用文件都在 |
| Z6·行 11 | 一致，但「还要」清单不完整 | `decisions/08-核心索引结构.md:430` 那句举例仍写「改动计数 1」，没被列进还要改的清单 |
| Z6·行 12 | 一致（sweep 一步复核不了） | 不变量、checker、坏镜像、变异全部对得上；「sweep 回扫已有条目」没找到证据 |
| Z6·行 19 | 一致 | `build_file_version_units`、按次新建 `BirthSequenceAllocator` 都在 |
| Z6·行 20a | 一致 | 三条变异、两条新用例名都对得上 |
| Z6·行 20b | 一致（结构层面） | 闭包与账分离的结构对得上；具体次数/字节数复核不了 |
| Z6·行 20c | 一致 | `cluster_segments` 字段、访问器、测试函数都在；具体落点数字复核不了 |
| Z6·行 20d | 机制一致，数字复核不了 | 全空聚簇段数逐设备增量维护的机制在，3310/50304/50368 未复核 |
| Z6·行 21 | 一致 | `warm_up` / `warm_up_after_journal_counter` 拆分、变异、测试名都对得上 |
| Z6·行 22 | 部分核实 | 抽查的 C77/C29 两条与代码现状一致；其余七条欠账未逐条复核 |
| Z6·行 23 | 一致 | 两条新用例、三态计数字段、6 条变异全部对得上 |
| Z6·行 23′ | 一致（数字复核不了） | 「12 个状态」与 I-3.9 那行引用同一场景，自洽；65 536 字节未复核 |
| Z6·行 23″ | 一致（数字复核不了） | `record_claimed_state_missing_unit` 字段在；8 个状态未复核 |
| Z6·行 28 | 一致 | `AccountingEntriesExceedOneNode`、79/80 盘两条变异边界都对得上 |
| Z6·行 34′ | 一致 | `FIRST_TRANSACTION_ACCOUNTING_ROWS = 15` 与行文一致 |

## 五、没做什么

- 不判 Z1（上界准入）、Z2（失败账）、Z3（聚簇段登记）、Z4 的算术那一半（区间端点具体数值、不适用两格的触发条件枚举）——按分工分别归云端攻方与本地攻方；不替它们找反例。
- 没有运行 `cargo test` / `cargo build`：本次核实全部靠读源码、`grep -n`、比对已落盘的产物文件，没有新编译或新跑任何用例，因此行 20b「11 次 / 245 760」「21 次 / 344 576」「32 次 / 590 336」、行 20c「落点 50306」、行 20d「3310」「50304→50368」、行 23′「差 65 536 字节」、行 23″「8 个状态」这些具体数字都标记「复核不了」，不是「未核」被我当成「一致」处理。
- 没有跑 `.claude/gate.d/` 任何阶段，也没有查 `stage-owners.tsv` 里是否登记给本条腿——派发提示未要求跑门禁，Z4/Z5/Z6 三格的核实方式是读代码与比对既有产物，不是跑装置。
- 二·4 节报的「11/21 区域字节不等」只是如实转述我看到的产物内容，没有去跑 `research/e7-index-bench` 或改代码定位这 11 处不等的根因，那需要跑两套代码逐字段比对，超出这次分到的判据格。
- 没有通读 `.claude/kb/checks-owed.md` 全文去核收口表行 22 里另外七条欠账（C287、C330、C282、C22、C80、C281、C314）今天各自的状态，只抽查了 C77、C29 两条能直接用一次 grep 核实的。
- 行 12「IMPLEMENTED_INVARIANTS 26 → 28」里「26」这个变更前的数没有独立核实（diff 附录未附旧版本），只核实了「今天是 28」。
- 没有读禁读清单里的文件（`m2-wave2-code-r1-opus-output.md`、`m2-wave2-code-r1-local-attack*.md`、`m2-wave2-code-r1-verifier-output.md`、`m2-wave2-code-r1-main-verification.md`）。
