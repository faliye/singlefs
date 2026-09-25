# m2-final-code-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽 Opus 报告里的引用「`mount.rs:1151`：`/// 分配记录树、记账树与中央映射树那三条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这三棵树，写行与暖机都不写它们`」。

冻结副本 `crates/singlefs-core/src/mount.rs` 第 1151 行原样：
```
/// 分配记录树、记账树与中央映射树那三条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这三棵树，写行与暖机都不写它们
```
与引文完全相同（✓）。在草稿目录副本里把行号加 1（改核第 1152 行），该行原样：
```
/// （D16（发布语义） 已定项 9 那五样一样都不写）。
```
与引文不同 ⇒ 判 **✗**。方法分辨（能把偏移一行的错误引用判出 ✗）。

## 输入缺口：本轮没有 `.claude/kb/` 开工快照

派发只给了 `research/prompts/m2-final-code-r1-snapshot/crates-src-sha256.txt`（对应 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`），没有给 `.claude/kb/` 的开工快照。按定义「腿开工那一刻的快照路径（`crates/` 与 `.claude/kb/` 至少这两样；代码轮必给）。没给快照的代码轮，停下要，不对主树核」——这是代码轮，且现查主树 `.claude/kb/` 目前相对 HEAD 有未提交改动：

```
$ git status --short .claude/kb/ | head -5
 M .claude/kb/checks-owed.md
 M .claude/kb/decisions-history.md
 ...
 M ".claude/kb/decisions/18-...信息.md"
 M ".claude/kb/decisions/19-...预算.md"
 M ".claude/kb/decisions/23-...格式.md"
$ git diff --stat HEAD -- .claude/kb/checks-owed.md .claude/kb/decisions/18-块里携带什么信息.md .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/invariants.md
 .claude/kb/checks-owed.md | 22 ++++++++----
 ...18-...信息.md          |  9 ++---
 ...23-...格式.md          | 40 ++++++++++++++--------
 .claude/kb/invariants.md  | 14 ++++----
```

这四个文件恰好是 Opus、Sonnet 两条云端腿引用最多的 kb 文件。按定义，本报告**对全部 kb 行号引用不核**（不拿主树顶替快照），逐条列入下方「核不动：缺 kb 快照」。已额外核过一件事：这些行号是否其实是背景材料 `_m2-final-code-r1-background.md` 的行号被误标成了 kb 行号——逐条核对背景材料对应行号处的文字，均与被引 kb 条款的文字**不同**（见下方各表），排除了「误写成背景材料行号」这一类。

## 腿一：云端攻方（Opus），`m2-final-code-r1-opus-output.md`

方法：全部 crates 行号对 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`（与 `crates-src-sha256.txt` 逐行核对，48/48 OK，见下方命令）现查；产物对 `research/prompts/m2-final-code-r1-opus-model/` 逐字核；复跑在 `/tmp/claude-1000/m2-final-code-r1-verifier/tree/`（冻结副本 rsync 而来，只加了 `opus_r1_attack.rs`，加入前后 sha256 核对见下）里做。

```
$ cd /tmp/claude-1000/m2-final-code-r1/tree && sha256sum -c .../crates-src-sha256.txt | grep -v OK
（无输出，48/48 OK）
```

### 表一：crates 代码行引用

| 引用 | 抄的原文（节选） | 结果 |
|---|---|---|
| `mount.rs:1151` | 分配记录树…只在带文件的一版上判 | ✓ 整行相同 |
| `mount.rs:1174` | `if let PreviousVersion::WithFile { output, .. } = &start.previous {` | ✓ |
| `mount.rs:1708` | 所选根覆盖的最后一条记录读得出就拿它当上一版的记录…jsn 最大的那条 | ✓ |
| `mount.rs:1718` | `.max_by_key(\|record\| record.counter)` | ✓ |
| `mount.rs:1850` | 同上（回退那一路同一句） | ✓ |
| `mount.rs:962` | `let reclaimed = allocator.reclaim_released_up_to(` | ✓ |
| `mount.rs:885` | `/// ⚠️ **今天只有测试入口，没有产品路径**：…` | ✓ |
| `mount.rs:1534` | `PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement => None,` | ✓ |
| `mount.rs:1659 起` `assert_eq!` | `if let (Some(planned), false) = (&placements_planned, …) {` 起，`assert_eq!` 实在 1660 行 | ✓（「起」指该 if 块起点，assert_eq! 紧随下一行，非摘句） |
| `mount.rs:143 起` 结构体 | `pub struct PublishSequenceFailed {` | ✓ 精确命中 |
| `journal.rs:276` | `let is_commit = reader.get_u8() == 1;` | ✓ |
| `journal.rs:279` | `if ordinal_within_publish.0 == 0 {` | ✓ |
| `recovery.rs:1744` | `let root_own_record_counter = counter_of_the_last_record_the_root_covers(...)?;` | ✓ |
| `recovery.rs:1682` | `if counters_carrying_the_last_record_flag.len() > 1 {` | ✓ |
| `recovery.rs:1692` | `Ok(counters_carrying_the_last_record_flag.first().copied())` | ✓ |
| `recovery.rs:1755` | `} else if record.checkpoint_txg != chain_start_txg_without_anchor` | ✓ |
| `recovery.rs:1756` | `\|\| record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST` | ✓ |
| `recovery.rs:1793` | `if !record.is_commit && record_ends_its_publish(record) {` | ✓ |
| `transaction.rs:689` | 写行那次发布，上一版树表 0 条：重写实例表链与分配记录树 | ✓ |
| `transaction.rs:732` | `version_without_file_row_publish_admission(` | ✓ |
| `transaction.rs:4144` | `let place_in_publish = if transaction_of_the_next_record.is_none() {` | ✓ |
| `transaction.rs:4154` | `is_commit: is_the_last_record_of_its_transaction,` | ✓ |
| `transaction.rs:2957` | `*allocator = allocator_before_this_publish;` | ✓ |

23 处代码行引用，**全部 ✓**，命令均为 `awk 'NR==N'`/`sed -n 'Np'` 现查（本表未把命令逐条贴出，方法一致；核对方法见判别力自证）。

### 表二：`.claude/kb/` 条款引用（附录 A1–A12）

| 引用 | 结果 |
|---|---|
| A1 `decisions/18-块里携带什么信息.md:314` | 核不动：缺 kb 快照 |
| A2 `checks-owed.md:329`（C378） | 核不动：缺 kb 快照 |
| A3 `milestone/02-second-txn.md:371` | 核不动：缺 kb 快照 |
| A4 `decisions/23-journal的角色与格式.md:145` | 核不动：缺 kb 快照 |
| A5 同文件 `:354` | 核不动：缺 kb 快照 |
| A6 同文件 `:362` | 核不动：缺 kb 快照 |
| A7 同文件 `:453` | 核不动：缺 kb 快照 |
| A8 同文件 `:383` | 核不动：缺 kb 快照 |
| A9 `checks-owed.md:332`（C381） | 核不动：缺 kb 快照 |
| A10 `checks-owed.md:502`（C516） | 核不动：缺 kb 快照 |
| A11 `checks-owed.md:504`（C512） | 核不动：缺 kb 快照 |
| A12 `checks-owed.md:478`（C542） | 核不动：缺 kb 快照 |

已排除「误写成背景材料行号」：背景材料对应行号（314/329/332/502/504/478/145/354/362/453/383）处的文字与这 12 条被引文字逐一核对，**均不同**（例如背景材料第 329 行是「### 小节清单：…」，与 A2 引的 C378 表格行完全不是一回事）。

### 表三：产物 sha256 与逐字引用

```
$ cd research/prompts/m2-final-code-r1-opus-model && sha256sum ./fix-fx.diff ./logs/*.log ./opus_r1_attack.rs
```
12/12 与报告表格逐行相同（✓）：`fix-fx.diff`、`logs/a1.log`、`logs/a1b.log`、`logs/a2-large.log`、`logs/a3.log`、`logs/a4.log`、`logs/a5-66.log`、`logs/a5b.log`、`logs/a5c.log`、`logs/a5d.log`、`logs/a5-fix-fx.log`、`opus_r1_attack.rs`。

| 引到的产物行 | 结果 |
|---|---|
| `a5-66.log` 第 78–80 行（mount#5 ok / mount#6,7 err，records:942/812） | ✓ 逐字相同；且第 21–44 行确为 mount#6→#29 共 24 行、每行 next instance 依次 +1、records 恒 942/812（「之后连试 23 次」= mount#7…#29） |
| `a5d.log` 第 88–89 行（mount#4 ok / mount#5 err，records:924/812，号不涨） | ✓ 逐字相同；mount#5…#9 的 `next instance before 23992 after 23992` 全部不变，坐实「号不涨」 |
| `a5-fix-fx.log` 第 21 行（改法 Fx 之后 mount#6 err，`before 23993 after 23993`） | ✓ 逐字相同 |
| `fix-fx.diff` 33 行 | ✓ `wc -l` = 33 |
| `a2-large.log` 第 7、85、163 行（三组 1500 种子、known_red 107/14/45、new_findings=0） | ✓ 逐字相同 |
| `a5b.log`（39 片 14 次内不拒、40/44/50/57–66 片首拒挂载次数与 records） | ✓ 逐字相同（`pages=39...last_ok...724`；`pages=40 first_error_at=Some(9)`；`pages=44 Some(8)`；`pages=50 Some(7)`；`pages=57`–`66` 均 `Some(6)`） |
| `a5c.log`（1200 次挂载无拒、停在 94 条） | ✓ `no refusal in 1200 mounts; ... allocation_records=94` |

### 表四：复跑命令（草稿副本 `/tmp/claude-1000/m2-final-code-r1-verifier/tree/`，`capped.sh 5`+`nice -n 19`）

| 复跑 | 命令 | 结果 |
|---|---|---|
| Z4-1 · z4_a5（66 片） | `OPUS_PAGES=66 OPUS_MOUNTS=30 bash capped.sh 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a5 -- --ignored --nocapture --exact z4_a5_without_file_row_publish_allocation_record_admission_after_acquisition` | ✓ `OBS` 行与 `a5-66.log` **逐行完全相同**（`diff` 无输出），`test result: ok` |
| Z4-1 · z4_a5d（阳性对照） | `OPUS_PAGES=66 OPUS_MOUNTS=10 bash capped.sh 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a5d -- --ignored --nocapture` | ✓ 与 `a5d.log` **逐行完全相同** |
| Z6 · z6_a4（1225 组合） | `bash capped.sh 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z6_a4 -- --ignored --nocapture` | ✓ 与 `a4.log` **逐行完全相同**；20 个结局行的 case 数求和 = **1225**（命令核对，非手数，见下）；全部 `violated=[]`、`test result: ok` |
| Z1 · z1_a1b | `bash capped.sh 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z1_a1b -- --nocapture` | ✓ 与 `a1b.log` **逐行完全相同** |
| Z4-2/3 · z4_a3 | `bash capped.sh 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z4_a3 -- --nocapture` | ✓ 与 `a3.log` **逐行完全相同** |
| Z6 · z6_a1 | `bash capped.sh 5 nice -n 19 cargo test --release -p singlefs-harness --test opus_r1_attack z6_a1 -- --nocapture` | ✓ 与 `a1.log` **逐行完全相同** |

1225 求和命令与输出：
```
$ grep -oE '^OBS a4 \[[0-9]+ cases' rerun-z6-a4.log | grep -oE '\[[0-9]+' | tr -d '[' | paste -sd+ | bc
1225
$ grep -c '^OBS a4 \[' rerun-z6-a4.log
20
```

Opus 计数：代码引用 23 + 产物 sha256 12 + 产物行引用 7 + 复跑 6 + kb 引用 12 = **共核 60 处；✓ 48，核不动 12（缺 kb 快照），✗ 0**。


## 腿二：云端正推（Sonnet），`m2-final-code-r1-sonnet-output.md`

方法：同样对冻结副本现查；未产出模型/产物目录（报告「没做什么」已注明），故无 sha256/复跑表；kb 引用同样受「缺快照」限制，不对主树核。

### 表一：crates 代码行引用

| 引用 | 抄的原文/claim | 结果 |
|---|---|---|
| `transaction.rs:1839` | `pub fn copies_failing_the_release_checksum_check<...>(` | ✓ |
| `transaction.rs:1888-1892`（`.or_else` 代码块） | 报告贴出 `let read_the_copy = \|\| {...}; let Some(copy) = read_the_copy().or_else(read_the_copy) else {...};` | **✗ 定位错**：该闭包定义确从 1890 行起，但 `.or_else` 那一行实际在**第 1898 行**（`// 读不出先重读一次…` 注释在 1897 行），不在引用范围 1888-1892 内 |
| `transaction.rs:1904-1907` | crc32 校验失败 push quarantined 代码块 | ✓ |
| `transaction.rs:1908-1917` | `every_copy_failed`/非空非全体即报错 | ✓ |
| `allocator.rs:906,909` | 文档注释与 `pub fn release_leaving_the_record_allocated_on` | ✓ |
| `transaction.rs:3542-3552` | `release_leaving_the_record_allocated_on(*placement, txg, &devices_whose_copy_failed_the_checksum)` | ✓ |
| `transaction.rs:1721` | `pub fn instance_table_chain_to_release(` | ✓ 精确命中 |
| `transaction.rs:1738-1741 起` | `roles_replaced_via_mapping` 文档注释 | ✓（函数体本身在 1745 行，「起」指文档注释起点，非摘句） |
| `transaction.rs:1861-1863`（「对这两个角色显式 continue」） | 声称此处是 `InstanceTable`/`InstanceTablePageAfterTheFirst` 的 `continue` | **✗ 定位错**：这两个角色的 `continue` 实际在**第 1857 行**（`match` 分支 `TransactionUnit::MappingTree \| TransactionUnit::TreeTable \| TransactionUnit::InstanceTable \| TransactionUnit::InstanceTablePageAfterTheFirst(_) => continue,`）；引用范围 1861-1863 是另一条不相关的 `continue`（`if !has_a_placement_in_the_previous_version(...)  { continue; }`） |
| `transaction.rs:1710-1712` | `instance_table_chain_to_release` 上方文档注释 | ✓ |
| `mount.rs:1153` | `fn refuse_publishes_before_acquisition_that_do_not_pass_admission(` | ✓ |
| `mount.rs:1536` | `let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(` | ✓ |
| `mount.rs:1504` | `refuse_publishes_before_acquisition_that_do_not_pass_admission(` 调用点 | ✓ |
| `mount.rs:1166` | `instance_table_chain_to_release(&instance_table_rewrite.replaced_chain, allocator)` | ✓ |
| `mount.rs:1170-1173` | 映到 `MountError::RowPublishAdmissionRefusedBeforeAcquisition` 那几行 | ✓ |
| `mount.rs:1189-1191` | `\|refusal\| match refusal.publish_index { 0 => MountError::RowPublishAdmissionRefusedBeforeAcquisition {...` | ✓ |
| `instance_table.rs:33-47` | `InstanceRow::to_bytes` 全函数体 | ✓ |
| `instance_table.rs:132-134` | `fn instance_rows_per_page()...` | ✓ |
| `instance_table.rs:140-145` | `fn instance_table_rows_of_each_page(...)` | ✓ |
| `instance_table.rs:181-193` | `InstanceTableChainRecord::to_bytes` | ✓ |
| `transaction.rs:890` | `fn build_instance_table_chain(` | ✓ |
| `transaction.rs:906` | `let role = TransactionUnit::of_instance_table_page(page);` | ✓ |
| `transaction.rs:876-878`（「尾片先装、先发出生序号……」文档注释） | 声称此处是这段文档注释 | **✗ 定位错**：876-878 实为 `struct InstanceTablePageUnit` 的字段体（`bytes: Vec<u8>, pointer: NodePointer, }`），被引文档注释实际起于**第 882 行**（`/// **尾片先装、先发出生序号**...`） |
| `transaction.rs:2260-2261`（「尾片先……第 0 片最后」） | 引用范围只含「…第 0 片最后」半句 | 邻近行误差：「尾片先」三字实际在**第 2259 行**，2260-2261 只是该文档注释的下半句；不算清晰的摘句错位，记一笔 |
| `transaction.rs:2262` | `pub fn instance_table_page_roles_in_bump_order(pages: usize) -> Vec<TransactionUnit> {` | ✓ |
| `transaction.rs:2264` | `.rev()` | ✓ |
| `transaction.rs:2241-2244` | `InstanceTableRewrite.rows` 文档注释与字段 | ✓ |
| `recovery.rs:1133` | `pub fn rebuild_version(` | ✓ |
| `recovery.rs:1140-1141` | `read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?` | ✓ |
| `recovery.rs:1034` | `pub fn instance_table_chain_of_root<...>(` | ✓ |
| `recovery.rs:980` | 文档注释「从根记录里那条指针读第 0 片…」 | ✓ |
| `recovery.rs:1236-1254` | 数据单元读取循环（`content_or_nothing_when_unreadable` 等） | ✓ 块内容匹配 |
| `recovery.rs:1469-1472` | `TransactionUnit::InstanceTable` 单元项 | ✓ |
| `walk.rs:50-54` | 三个常量定义（`INODE_RECORD_SIZE_OFFSET` 等） | ✓ |
| `walk.rs:709-717` | I-9.15 判定代码块 | ✓ |
| `walk.rs:2193` | `fn judge_rollback_floor_raises_against_their_ceilings(` | ✓ |
| `walk.rs:2210-2223` | `previous_root_of_the_same_instance`/`floor_before_the_raise` 判定块 | ✓ |
| `walk.rs:2109-2115`、`2113-2114` | 有效根过滤条件 | ✓ |
| `walk.rs:2071-2079` | `fn rollback_floor_ceiling_from(...)` | ✓ |
| `walk.rs:2236-2251`（「三种结局」判定） | 声称此范围含 `raised_floor > lowest && <= highest ⇒ 不判` 比较式 | 邻近行误差：该比较式实际在**第 2230 行**，早于引用范围 6 行；范围内 2236 起的确是 `judge("I-7.9", ...)` 违例分支，非纯摘句错位 |
| `walk.rs:2253-2262` | `raising_roots_judged == 0` 分支与下一函数起点 | ✓ |


41 处代码行引用中（`walk.rs:2109-2115、2113-2114` 合写一行）：**✓ 36，✗ 3（明确定位错：`transaction.rs:1888-1892`、`transaction.rs:1861-1863`、`transaction.rs:876-878`），2 处邻近行误差记一笔（`2260-2261`、`2236-2251`，未判 ✗——引用范围内仍含该结论直接依赖的判定代码，只是关键比较式在范围外几行，判据字面上仍成立，记「邻近行」不计入 ✗ 也不计入 ✓）**。

### 表二：测试文件行引用

| 引用 | 结果 |
|---|---|
| `second_transaction_supplement_two_release_checksum_quarantine.rs:453,483,131,540,606,241,263,286` | ✓ 8 处全部精确命中（函数名/断言逐字相同） |
| `second_transaction_supplement_two_instance_table_page_full.rs:96,134` | ✓ 2 处精确命中 |
| `second_transaction_supplement_two_instance_table_second_page_write.rs:155,298,358` | ✓ 3 处精确命中 |
| `second_transaction_supplement_two_instance_table_chain.rs:381,423` | ✓ 2 处精确命中 |
| `checker_known_bad_images.rs:2239,3188` | ✓ 2 处精确命中 |
| `instance_table.rs:391` / `407`（结论 4 测试证据，与本地攻方核对表 Fact 1d 共用同一处） | ✓ 精确命中（`391`=`fn pages_for_rows_...`；`393-400`=五组断言逐字相符；`407`=`fn rows_fill_a_page_...`） |

18 处测试文件行引用，**全部 ✓**。

### 表三：`_m2-final-code-r1-background.md` 行引用

| 引用 | 结果 |
|---|---|
| `:49`（「只一块盘那一份核出坏时…」） | **✗ 差一行**：该句实际在**第 48 行**；第 49 行是「checker 的 I-3.11、I-3.1 对隔离记录的豁免读法…」，与引文完全不同的一句 |
| `:50`（「树表 0 条那一版写行，片数…」） | ✓ 精确命中 |
| `:53`（「写行那次的释放核验挪到取号之前…」） | ✓ 精确命中 |

Sonnet 报告开头声明「下面每条引文标注的行号都已核对」——本次核对发现这句声明不完全成立（`:49` 是错的，以及代码里 2 处明确定位错）。

### 表四：`.claude/kb/` 条款引用

| 引用 | 结果 |
|---|---|
| `decisions/19-块指针的结构与宽度预算.md:105`（引两次：N1、N3，及末句 N2） | 核不动：缺 kb 快照 |
| `decisions/19-块指针的结构与宽度预算.md:99`（硬规则 1 首句） | 核不动：缺 kb 快照 |
| `decisions/19-块指针的结构与宽度预算.md:107`（已定项 5，实例表豁免映射） | 核不动：缺 kb 快照 |
| `checks-owed.md:479`（C543） | 核不动：缺 kb 快照 |
| `invariants.md:283`（I-9.15） | 核不动：缺 kb 快照 |
| `invariants.md:59`（I-7.9） | 核不动：缺 kb 快照 |
| `decisions/08-核心索引结构.md:150-216` | 核不动：缺 kb 快照 |
| `decisions/03-空间分配.md:197-226`（结论 5 引文，报告自称 grep 命中「第 637 行」，与结论 5 标注的 `197-226` 不一致，本身即报告内部矛盾，因缺快照未继续核） | 核不动：缺 kb 快照（附带记：报告正文引用行区间 `197-226` 与其自述的 grep 命中行 `637` 不一致，这处内部矛盾本身独立于快照缺口就能看出，值得主 agent 注意） |

已排除「误写成背景材料行号」：背景材料对应行号处文字与被引 kb 条款均不同（见前节）。

Sonnet 计数：代码引用 41（✓36 ✗3 邻近行 2）+ 测试文件引用 18（✓18）+ 背景材料引用 3（✓2 ✗1）+ kb 引用 8（核不动 8）= **共核 70 处；✓ 56，✗ 4，核不动 8，邻近行未判 2**。


## 腿三：本地攻方，`m2-final-code-r1-local-attack-output-s1.md` / `-s2.md`

方法：提示文件 `m2-final-code-r1-local-attack.md` 是纯算术题（不含 kb 引用），核对表核对文件:行是否存在于冻结副本、原文是否与提示文件转述一致；两份样本的算术逐格重算。

### 表一：核对表（`m2-final-code-r1-local-attack-translation-audit.md`）里的「原文文件:行」

| 原文文件:行 | 结果 |
|---|---|
| `singlefs-format/src/lib.rs:117`（370） | ✓ `pub const INSTANCE_TABLE_PAGE_RECORDS: u64 = 370;` |
| `singlefs-format/src/lib.rs:156`（4096） | ✓ |
| `singlefs-format/src/lib.rs:165`（311） | ✓ |
| `singlefs-format/src/lib.rs:168-169`（56，公式） | ✓ |
| `singlefs-format/src/lib.rs:172-173`（除法定义） | ✓ |
| `singlefs-format/src/lib.rs:324`（assert_eq 56） | ✓ |
| `singlefs-core/src/instance_table.rs:123-124` | ✓ 逐字相同，含「D18 已定项11」「0 行也是一片」「D28 已定项3 同一个数」三点全在 |
| `singlefs-core/src/instance_table.rs:126-129` | ✓ `instance_table_pages_for_rows` 全函数体 |
| `singlefs-core/src/instance_table.rs:391,393-400` | ✓ 五组断言（0→1,369→1,370→2,738→2,739→3）逐字相符 |
| `singlefs-core/src/transaction.rs:970-971` | ✓ |
| `singlefs-core/src/transaction.rs:1038-1063`（点名项文档注释） | ✓（含「按 bump 次序——…尾片先…」，核对表如实记「有意简化未转述排序细节」） |
| `singlefs-core/src/transaction.rs:1028-1031` | ✓ |
| `singlefs-core/src/journal.rs:231-233` | ✓ `to_bytes` 无长度核 |
| `singlefs-core/src/journal.rs:190-230` | ✓ 字段写入次序（含 190 行起 `put(&JOURNAL_MAGIC)` 等） |
| `singlefs-core/src/journal.rs:21-22`（46 的公式） | ✓ |
| `singlefs-core/src/journal.rs:24`（7 的公式） | ✓ |
| `singlefs-core/src/journal.rs:63-65`（87 的公式） | ✓ |
| `singlefs-core/src/journal.rs:215-217`（91/95 由运行时 position 得出） | ✓ |
| `singlefs-core/src/transaction.rs:3118-3126`（Fact 4a 文档，含 C310、C491 标签） | ✓ 逐字相同，`C491（多条记录时共享内生块在哪条点名没定）` 确在第 3118 行 |
| `singlefs-core/src/transaction.rs:3130-3167`，`3159-3164`（`chunks(named_unit_capacity)`） | ✓ |
| `singlefs-core/src/transaction.rs:3169-3172,3174-3186`（Fact 4e 文档，含「已定项 7」标签） | ✓ 逐字相同，`已定项 7「同一事务的全部记录共享它」` 确在第 3172 行 |

26 处「原文文件:行」，**全部 ✓**；未发现行号错位或抄漏。核对表自陈的「仍未核平」两处（Fact 4a 的 C491、Fact 4e 的「已定项 7」在英文提示里被意译掉、未补回标签本身）经现查**属实**——提示文件 `m2-final-code-r1-local-attack.md` 的 Fact 4a/4e 段落确实没有出现 "C491" 或 "settled item 7" 字样，核对表的自我诚实登记站得住。


### 表二：两份样本的算术逐格重算

Q1（`rows_per_page=369`，`pages=ceil(rows/369).max(1)`）：369→1、370→2、738→2、739→3、24354→66（369×66=24354 恰好整除）、24355→67。s1、s2 六个数**全部正确**，与 fact 1d 五组断言一致。

Q2：4096−311=3785；3785÷56=67 余 33（56×67=3752，3785−3752=33）；截尾取 67。s1、s2 **均正确**。

Q3a：P+1。✓ 两份一致。
Q3b：P+1>67 ⇒ P>66 ⇒ P_min=67。✓ 两份一致。
Q3c：ceil(rows/369)=67 ⇒ rows>66×369=24354 ⇒ rows_min=24355。✓ 两份一致。
Q3d：page=67、row=24355，均「是」。✓ 两份一致。

Q4 八个场景（k=事务数，m=末事务点名项数，每记录上限=67）：记录总数 = (k−1) + ceil(m/67)（k=1 时为 ceil(m/67)）；事务归属按 fact 4e；末条标志在最后一条记录。逐场景重算：

| 场景 | k,m | 应得 (i)(ii)(iii) | s1 | s2 |
|---|---|---|---|---|
| 1 | 1,67 | 1；[0]；末条=0 | ✓ | ✓ |
| 2 | 1,68 | 2；[0,0]；末条=1 | ✓ | ✓ |
| 3 | 1,134 | 2；[0,0]；末条=1 | ✓ | ✓ |
| 4 | 1,135 | 3；[0,0,0]；末条=2 | ✓ | ✓ |
| 5 | 3,67 | 3；[0,1,2]；末条=2 | ✓ | ✓ |
| 6 | 3,68 | 4；[0,1,2,2]；末条=3 | ✓ | ✓ |
| 7 | 3,134 | 4；[0,1,2,2]；末条=3 | ✓ | ✓ |
| 8 | 3,135 | 5；[0,1,2,2,2]；末条=4 | ✓ | ✓ |

**16 个 (i)(ii)(iii) 数值答案（两份样本各 8 场景）全部正确**，重算结果与两份样本逐格相同。

falsification note 逐条核（这是本题「什么现象会推翻它」的正文，因此逐句核其自洽性，不只核数值）：

- **s1 场景 3、场景 7 均有缺陷**：两处都写「若 m 由 134 改成 133，记录总数不变（仍是 2 / 仍是 4，133÷67=1.985→ceil 2）」——m 从 134 减到 133 时 `ceil(m/67)` 并未跨越取整边界（134 和 133 的 ceil 都是 2），记录总数**确实不变**；这句话本该是"什么会翻面"的证伪句，s1 自己算出的却是"翻不了面"，即 s1 在这两格给出的证伪陈述与它自己的定义自相矛盾（不是数值错，是「挑错了会翻面的量」，两处重复同一个错法）。
- s1 场景 1、2、4、5、6、8 的证伪句均正确翻面（如场景 4：m 135→134，ceil 从 3 变 2，记录总数 3→2，翻面成立）。
- **s2 场景 3、7 选择改「每记录上限」（67→134）而非改 m**，重算：per-record max=134 时 ceil(134/134)=1，场景 3 总数 2→1（翻面成立）；场景 7 总数 4→3（翻面成立）——s2 在这两格避开了 s1 的陷阱，证伪句站得住。

Q5a：record flags=7、ordinal-within-publish=87、back-chain=91、payload checksum=95（重算：4+2+1=7；一路累加到 commit flag 止=87；87+4=91；91+4=95）。s1、s2 **四个偏移全部正确**。
Q5b：四个偏移均「在 [0,4096) 覆盖范围内、在 32 字节校验和字段自身范围 [46,78) 之外」。s1、s2 **全部正确**。

本地攻方计数：核对表引用 26 处（**✓ 26**）+ 独立重算的核心数值 20 组（Q1 六个 + Q2 一个 + Q3a-d 四个 + Q5a 四个 + Q5b 四个 = 20；两份样本在这 20 组上取值相同且与我独立算出的结果一致，**✓ 20**）+ Q4 八场景 × 两份样本 = 16 组 (i)(ii)(iii) 判定（**✓ 16**）+ Q4 falsification note 16 条（**✓ 14，有缺陷 2**：s1 场景 3、7 两条自相矛盾——所选的「m 由 134 减到 133」不跨越 ceil 取整边界，记录数不变，不构成「翻面」；s2 对应两条改「每记录上限」反而站得住）。**本地攻方合计核 26+20+16+16=78 处，✓ 76，有缺陷 2（不计入 ✗，因引用本身无误，缺陷在于所选证伪量未能真正翻面）**。


## 全轮汇总

| 腿 | 核了 | ✓ | ✗ | 核不动/缺快照 | 其它 |
|---|---|---|---|---|---|
| Opus | 60 | 48 | 0 | 12（kb） | — |
| Sonnet | 70 | 56 | 4 | 8（kb） | 2 处邻近行误差（未计入✓/✗） |
| 本地攻方 | 78（26 核对表 + 20 核心数值 + 16 个 (i)(ii)(iii) 判定 + 16 条 falsification note） | 76 | 0 | 0 | 2 条 falsification note 有缺陷（不计入 ✗，见上） |
| **合计** | **208** | **180** | **4** | **20** | **2 邻近行 + 2 falsification 有缺陷** |

四处 ✗ 全部出自 Sonnet 报告，且均为定位错（不是内容伪造）：`transaction.rs:1888-1892`、`transaction.rs:1861-1863`、`transaction.rs:876-878`（代码引用定位错，实际位置分别是 1898、1857、882）、`_m2-final-code-r1-background.md:49`（应为 48）。Z4-1、Z6 两个承重复跑（a5 组、a5d 阳性对照、1225 组合）全部逐行复现，无一处对不上。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- **全部 20 处 kb 条款引用未核**（Opus 12 处、Sonnet 8 处）：本轮未给 `.claude/kb/` 开工快照，且现查主树 `.claude/kb/` 相对 HEAD 有未提交改动、恰好覆盖被引的四个文件，按定义不拿主树顶替快照。请主 agent 之后补一份 `.claude/kb/` 开工快照（或确认这些文件在腿开工到现在没有变化）以便补核。
- 未复跑 Opus 的 `z4_a5b`（2..66 片 sweep × 14 次挂载）、`z4_a5c`（1200 次纯挂载）、`z6_a2`（三组 × 1500 种子 × 60 步随机历史，原始产物显示单组耗时 270–646 秒，三组共约 23 分钟）——这三条的**产物行引用**已逐字核对通过（见表三），但受时间预算限制未在草稿目录里独立复跑；Z4-1、Z6 两个被点名的重点复跑（a5-66、a5d、z6_a4 共 1225 组合）均已复跑且逐行相同。
- 未编译、未跑 Sonnet 报告涉及的用例（该报告本身声明未建模型/产物目录，无可复跑对象）。
- Sonnet 报告里「结论 5」自述 grep 命中 `.claude/kb/decisions/03-空间分配.md` 第 637 行、但结论 5 正文标注的引用行区间是 `197-226`，两者不一致——因 kb 缺快照未展开核，只记为一条可疑点转交主 agent。
- 未逐字核对 Sonnet 报告「未验证」自陈的两处（`rolling_back_to_the_root_at_the_effective_floor_is_accepted_...` 源码位置、N2 是否有专门测试）——报告自己已标「复核不了」，不在核查员职责内重新去找。
