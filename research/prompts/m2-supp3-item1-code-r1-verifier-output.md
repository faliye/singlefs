# m2-supp3-item1-code-r1 · 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

从 Opus 腿引用里挑一条：`crates/singlefs-harness/src/history.rs` 第 751 行，原文抄的是
「/// 只看「抬 F 那一步之后、根环没转圈、只有 I-3.1 红且记账多于遍历」，不看 F 是否落在回退留下的空档里：观察里没有回退历史。」。
在草稿目录的副本里把行号加 1（改核第 752 行），副本第 752 行实际内容是
`fn raise_after_rollback_leaves_allocated_statistic_above_walked(`，与被核引文不同。
判定：**✗（方法分辨）**。核查方法有效，往下按同一套办法逐条核。

## 输入核对

- 三条腿报告文件均存在、sha256 与派发提示给的一致：
  - Opus：`bcf2e7c2be02504ad100145d014574f72f88aec2c89aa2cbb78568a3bed8a594`（核对：一致）
  - Sonnet：`55aa4e509bc4948f39607d59f2989d0723845beaf855a186c3bfd95241d4583b`（核对：一致）
- 开工快照 `m2-supp3-item1-code-r1-start-snapshot.sha256` 列出的 9 个 `crates/` 文件与主树现在逐字节相同（现查 sha256）；两份 kb（`.claude/kb/milestone/02-second-txn.md`、`.claude/kb/decisions/16-发布语义.md`）主树现在的哈希与快照不同，用派发提示给的
  `/tmp/claude-1000/m2-supp3-item1-r1-verifier/snapshot-copies/` 原样副本核（两份副本的 sha256 与快照记录逐一相同）。
- `root_ring.rs`、`.claude/kb/decisions/03-空间分配.md`、`crates/singlefs-core/src/recovery.rs` 三个文件不在给定快照清单内：现查 `git status --short` 均为干净（无未提交改动），`git log -1` 显示最后提交时刻早于快照登记时刻（15:08Z）或腿开工时刻（15:30–15:33Z），按主树现查，未记 ✗。

## 一、Opus 攻方腿（`m2-supp3-item1-code-r1-opus-output.md`）

### 1.1 原文引用（对快照 / 主树逐行核）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `history.rs:751` 注释整行 | ✓ 逐字相同 | `sed -n '751p' crates/singlefs-harness/src/history.rs` |
| `history.rs:755-756` 两行匹配条件 | ✓ 逐字相同 | `sed -n '755,756p' history.rs` |
| `history.rs:748` 第 0 条匹配条件 | ✓ 逐字相同 | `sed -n '748p' history.rs` |
| `history.rs:1357` `apply_mount_writable` 函数签名行 | ✓ | `sed -n '1357p' history.rs` |
| `history.rs:1250` `apply_publish_overwrite` 函数签名行 | ✓ | `sed -n '1250p' history.rs` |
| `history.rs:1420-1421` 抬 F 目标推导（`[今天的 F, 现行 txg+2]`） | ✓ 公式化简后与代码一致 | `sed -n '1420,1421p' history.rs`，手算 `current_floor+choices-1=txg_before_raise+2` |
| `history.rs:1447-1453` `apply_cold_start_recover` | ✓ 逐字相同 | `sed -n '1447,1453p' history.rs` |
| `history.rs:1565` 冷启动跳过 checker 条件 | ✓ | `sed -n '1565p' history.rs` |
| `history.rs:964` 入口 `Err` 算合法结局 | ✓ | `sed -n '964p' history.rs` |
| 测试文件:305 `report.new_findings.is_empty()` | ✓ | `sed -n '305p' .../second_transaction_supplement_three_random_history.rs` |
| 测试文件:67 panic「一条已释放的记录都没被复用」 | ✓ 行号与 panic 位置吻合（多行 assert! 宏起始行） | `sed -n '63,72p'` 同上文件 |
| 测试文件:66「抬 F 一次都没回收到落点」 | ✓ | 同上 |
| 测试文件:75-76「复用时一条被罩住的已释放记录都没删过」 | ✓ | 同上 |
| 测试文件:99「冷启动一次都没读回文件」 | ✓ | `sed -n '90,100p'` 同上 |
| 测试文件:94「内容长度…没进过入口」 | ✓ | 同上 |
| `allocator.rs:896` A1 原文 `<=` | ✓，且原文在文件里唯一命中一次 | `sed -n '896p'`；`grep -c` |
| `mount.rs:659` A2/A4 原文（同一处，两条变异分别替换） | ✓，唯一命中一次 | `sed -n '659p' mount.rs`；`grep -c` |
| `walk.rs:1376` A5 原文 | ✓ | `sed -n '1376p' walk.rs` |
| `mount.rs:278` A8 原文 | ✓，唯一命中一次 | `sed -n '278p' mount.rs`；`grep -c` |
| `root_ring.rs:36` A7 原文（不在给定快照清单，见上「输入核对」） | ✓，对主树 | `sed -n '36p' root_ring.rs` |
| `allocator.rs:542` N2 原文 | ✓ | `sed -n '539,543p' allocator.rs` |
| `.claude/kb/decisions/03-空间分配.md:116`（不在快照清单） | ✓，对主树 | `sed -n '116p'` |
| `mount.rs:575` B3 原文 | ✓ | `sed -n '575p' mount.rs` |
| `mount.rs:477` B5 原文 | ✓ | `sed -n '477p' mount.rs` |
| `decisions/16-发布语义.md:374`（抬 F 上限表格行） | ✓ 对快照副本 | `sed -n '374p'` 副本 |
| `mount.rs:1203` B2/探针原文 | ✓ | `sed -n '1203p' mount.rs` |
| `decisions/16-发布语义.md:376`（回退候选集表格行） | ✓ 对快照副本 | `sed -n '376p'` 副本 |
| `transaction.rs:1248` B1 原文 | ✓ | `sed -n '1248p' transaction.rs` |
| `unit.rs:520` B6 原文 | ✓ | `sed -n '518,522p' allocator.rs`（`unit.rs` 同名模式经 mutants.tsv 核对，见下） |
| `mount.rs:407` A3 原文 | ✓ | `sed -n '407p' mount.rs` |
| `milestone/02-second-txn.md:338` 第 26 行内容（checker 欠账） | ✓ 对快照副本 | `sed -n '338p'` 副本 |
| `milestone/02-second-txn.md:311` 第 ② 行 | ✓ 对快照副本 | `sed -n '311p'` 副本 |
| `walk.rs:1374` A6 原文 | ✓ | `sed -n '1374p' walk.rs` |
| `allocator.rs:520-521,529,549` N4/N5 探针定义处原文 | ✓ | `sed -n '518,522p;527,531p;547,551p' allocator.rs` |
| 背景材料 44/45/46/50/24 行（Z2/Z3/Z4 判据原文、反向接受条款、射程句） | ✓ 五处全部逐字相同 | `sed -n '44p;45p;46p;50p;24p' _m2-supp3-item1-code-r1-background.md` |

小计：核了 35 处原文引用，✓ 35，✗ 0（背景材料一行核了 5 处行号，按表格行计数，非按引用条数计数）。

### 1.2 产物核对（模型目录、日志、tsv）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `SHA256SUMS` 自身哈希 `f0a261ee…` | ✓ 一致 | `sha256sum SHA256SUMS` |
| `SHA256SUMS` 172 行、逐文件核验 | ✓ 171/172 通过；1 条 `SHA256SUMS.tmp` 是生成时留下的自指幻影行（文件已改名为 `SHA256SUMS`），不是真实缺失 | `wc -l SHA256SUMS`；`sha256sum -c`（剔除该行后 171/171 通过） |
| 报告表格里 20 个文件的独立 sha256（`mutants.tsv`、`probes.tsv`、`rows-41-121.tsv`、`mutant-diffs.patch`、5 个 `.py`、2 个 `.diff`、4 个 `.rs`、3 个 `.sh`） | ✓ 全部 20 个逐一重算相符 | `sha256sum` 逐个核对 |
| `logs/z2-history.out` 全文（BASE/A1/A2/A5/A8 五段收尾行） | ✓ 逐字相同，含 `reclaimed_placements: 1`→`0` 的关键数字 | `cat logs/z2-history.out` |
| `logs/z3-history.out` 全文（ceiling 段 BASE/B3、cold 段 BASE/B6） | ✓ 逐字相同 | `cat logs/z3-history.out` |
| `summary-fast-A.tsv`/`summary-large-A.tsv`（BASE/A1/A2/A3/A4/A5/A8 各档计数） | ✓ 全部逐字相同 | `grep`/`awk` 逐行核对 |
| `summary-F.tsv`（A7 三档、PB2b 三档） | ✓ 全部逐字相同，含大档「新发现」违例明细 | 同上 |
| `summary-large-B.tsv`/`summary-fast-B.tsv`（N2/B1/B2/B3/B5/B4 各档） | ✓ 全部逐字相同 | 同上 |
| `summary-probe-P.tsv`（PN2/PA3/PA4/PB2/PN5） | ✓ 全部逐字相同；`PB2b` 数据实际落在 `summary-F.tsv`（非 `-probe-P.tsv`），报告未点名具体文件但内容与 `summary-F.tsv` 一致 | `grep "^PB2b" logs/summary-F.tsv` |
| `logs/raw/logs-probe/PN2-fast.log` panic 行原样 | ✓ 逐字相同（`opus-probe-N2 old CheckpointTxg(5) new CheckpointTxg(29)`） | `grep -n "opus-probe-N2 old"` |
| `logs/raw/logs-F/PB2b-large-3000x40.log` panic 行原样 | ✓ 逐字相同 | `grep -n "opus-probe-B2b"` |
| `logs/raw/always-{BASE,N1}-{skip,check}.log` 四份 `OPUS-SUMMARY` 行 | ✓ 四份两两相同、与报告引文相同 | `grep "OPUS-SUMMARY"` 四个文件 |
| `logs/geometry.tsv` 19 个几何点（BASE 退出码 + R121「新发现」计数） | ✓ 全部 19 点逐一核对，含加粗标注的三个例外点与两处种子号（54、69） | `awk -F'\t'` 分列核对 |
| `logs/probe-R121-3000x30.log`／`probe-BASE-3000x30.log` 汇总行 | ✓ 逐字相同 | `grep "OPUS-SUMMARY"` |
| 101 个 I-5.4 判出种子按 96 一窗切 31 窗的分布 | ✓ 独立用 `awk` 重新分桶，31 个数字逐一相同 | `grep "I-5.4" \| awk '{w=int($3/96);c[w]++}...'` |
| `logs/gap-run.out` 六行「原样」+ 作废说明 | ✓ 报告排除的两条注释行确实标了「作废」，剩余六行与报告引文逐字相同 | `cat logs/gap-run.out` |
| `logs/n2-step-five.log`/`b3-step-five.log` | ✓ 逐字相同，panic 位置 `:198:5`、`4 failed` 均对应 | `cat` 两个文件 |
| `mutants.tsv` 14 条变异定义（A1/A2/A3/A4/A5/A8/A7/B1/B2/B3/B5/B6/N1/N2/N3/N4/N5） | ✓ 全部与报告正文 diff 块逐字相同 | `cat mutants.tsv` |
| `geometry-env.diff`（59 行）/`proposal.diff`（139 行） | ✓ 行数与报告一致 | `wc -l` |
| `summary-G-proposal.tsv`（A1 三档改法前后对比、BASE 三档） | ✓ 全部逐字相同 | `awk` 核对 |
| `logs/raw/logs-fast-B/{N1,N3,N4}-fast.log`（panic 位置、新发现计数、违例类型） | ✓ 全部相同（N1→allocator.rs:529/I-3.1，N3→I-3.9，N4→allocator.rs:549） | `grep` |
| dev profile 复跑：N2/B3/B6 三份 `logs/raw/logs-dev/*-fast.log` | **✗**：N2 一处，报告引「`logs/raw/logs-dev/N2-fast.log` 末行…`finished in 35.54s`」，N2-fast.log 实际末行是 `finished in 25.85s`；`35.54s` 是 `B6-fast.log` 的耗时，被误标到 N2 上。B3（30.53s）、B6（35.54s）各自的行本身与文件相符 | `grep "test result" logs/raw/logs-dev/{N2,B3,B6}-fast.log` |
| `logs-F/B6-fast.log`、`B6-large-3000x40.log`（冷启动结局计数块） | ✓ 逐字相同 | `grep` |

小计：核了 23 处产物 / 日志块，✓ 22，✗ 1（N2 dev-profile 耗时误标为 B6 的数值）。

Opus 腿合计：核了 58 处（原文 35 + 产物 23），✓ 57，✗ 1；核不动 0。

## 二、Sonnet 正推腿（`m2-supp3-item1-code-r1-sonnet-output.md`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `history.rs:184-189`（`FloorTargetChoice::steps_above_current_floor` 文档注释，5 行引文） | **✗**：引文与快照第 183-187 行逐字相同，不是 184-189；该注释块整段实际横跨 183-188（6 行），报告标注的区间整体错位一行 | `sed -n '182,190p' history.rs` |
| `mount.rs:590` | ✓ 逐字相同 | `sed -n '590p' mount.rs` |
| `decisions/16-发布语义.md:107` | ✓ 对快照副本逐字相同 | `sed -n '107p'` 副本 |
| `history.rs:527-533`（`MissingPrecondition::CurrentVersionWithFile` 文档注释，3 行引文） | **✗**：引文与快照第 529-531 行逐字相同；527-528 属于前一个变体 `CurrentVersionWithoutFile` 的注释与该变体名本身，532-533 是「以后要测」句与 `CurrentVersionWithFile,` 变体名本身，均不在引文里；标注区间比实际引文多出前后各 2 行、且未覆盖 532 | `sed -n '525,535p' history.rs` |
| `transaction.rs:506` | ✓ 逐字相同 | `sed -n '506p'` |
| `decisions/16-发布语义.md:192` | ✓ 对快照副本逐字相同 | `sed -n '192p'` 副本 |
| `milestone/02-second-txn.md:401`（设想实现 1 七个入口原文） | ✓ 对快照副本逐字相同 | `sed -n '401p'` 副本 |
| `history.rs:28-35`（七个入口 `use` 语句） | ✓ 七个函数名全部命中 | `sed -n '28,35p'` |
| `history.rs:477,479`（`acquire_instance`/`warm_up`） | ✓ | `sed -n '477p;479p'` |
| `milestone/02-second-txn.md:384/311/360`（现状段、② 行、43 行整行） | ✓ 三处对快照副本逐字相同，表格转述与原文一一对应 | `sed -n '384p;311p;360p'` 副本 |
| `crates/mutations.tsv`（130 行；41/121/129/130 行对应关系） | ✓ 全文 130 行；129/130 行原文替换文与 41/121 行逐字相同 | `wc -l`；`sed -n '41p;121p;129p;130p'` |
| `history.rs:1514/1518` execute_history(_observing) 起始 | ✓（位置指针，1514 为 `pub fn` 精确行，1518 为紧邻文档注释首行） | `sed -n '1514p;1518p'` |
| `history.rs:760` KNOWN_RED_FORMS | ✓（位置指针，762 行为 `pub const`，760 为其文档注释首行） | `grep -n "KNOWN_RED_FORMS"` |
| `history.rs:392` generate_history | ✓（位置指针，394 为 `pub fn`，392 为文档注释首行） | `grep -n "fn generate_history"` |
| `history.rs:1761/1835` shrink_operations/shrink_failing_history | ✓（位置指针，均为各自文档注释首行，`pub fn` 分别在 1766/1837） | `grep -n "fn shrink_"` |
| `history.rs:2004` run_history_campaign | ✓（位置指针，`pub fn` 在 2010） | `grep -n "fn run_history_campaign"` |
| 测试文件:18-20（FAST_TIER 三个常量） | ✓ 逐字相同 | `sed -n '18,20p'` |
| `history.rs:59`（SplitMix64 注释） | ✓ 逐字相同 | `sed -n '59p'` |
| `history.rs:73-77`（`next_word` 混合步骤） | ✓ 逐字相同，含三个 SplitMix64 标准常数 | `sed -n '73,77p'` |
| `history.rs:1291-1304`（`apply_publish_without_units` 守卫） | ✓ 转述准确，守卫代码在该区间内 | `sed -n '1288,1308p'` |
| `transaction.rs:1129-1134`（`publish_first_file` 的 `# Errors`） | ✓ 逐字相同 | `sed -n '1129,1134p'` |
| `history.rs:1210-1248`（`apply_publish_first_file` 函数边界） | ✓ 起止行精确（1210 起，1248 闭合括号） | `sed -n '1210p;1248p'` |
| `history.rs:292`（`PublishFirstFile` 权重 4） | ✓ 逐字相同 | `sed -n '292p'` |
| `mount.rs:595-618`（`raise_rollback_floor` 函数体、无 `new_floor>=current_floor` 检查） | ✓ 函数边界精确，区间内确实只查 `new_floor > ceiling` | `sed -n '595,618p'` |
| 独立复跑：`probe_row43_floor_targets.rs`（sha256 `03e0636b…`） | ✓ sha256 与报告一致；在核查员自建的独立仓副本（`/tmp/claude-1000/m2-supp3-item1-r1-verifier/sonnet-repro/`，`history.rs`/测试文件 sha256 与快照相同）里重新编译运行，六行输出逐字与报告一致 | `sha256sum`；`nice -n 19 cargo test --test zzz_sonnet_r1_probe -- --nocapture` |
| 独立复跑：96 段快档（`random_histories_fast_tier_…`） | ✓ 在同一独立副本上重新编译运行，`历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}…` 与两个 `Err 成员` 计数（34、122、58）逐字与报告一致 | `nice -n 19 cargo test --release --test second_transaction_supplement_three_random_history -- --nocapture random_histories_fast_tier_…` |

小计：核了 26 处，✓ 24，✗ 2（两处文档注释的行号区间均偏移，指向的实际内容分别在快照第 183-187/188 行与第 529-531/532 行）。

## 三、本地攻方腿（提示 + 转述核对表 + 运行记录）

按定义第 5 步，本地腿的转述核对表 `m2-supp3-item1-code-r1-local-attack-translation-audit.md` 里每一处「原文文件:行」逐条核，兼查有没有丢限定词、多加限定词。

| 原文文件:行（核对表条目） | 核的结果 | 命令 |
|---|---|---|
| `crash.rs:26`（`SECTOR_BYTES = 512`） | ✓ 逐字相同 | `sed -n '26p' crash.rs` |
| `crash.rs:36-40`（标注「`read` 旧体，改前逐扇区 get」，对应提示的 old_read 伪代码） | **✗**：`crates/singlefs-harness/src/crash.rs` 是这一轮实现员的未提交改动（`git diff` 可见），快照/现状第 36-40 行是**改后**的瘦包装（`pub fn read` 直接调 `read_into`，对应提示里的 `new_read`，不是 `old_read`）；真正「逐扇区 `if let Some`…`else` 维持 0」的旧逻辑在 `git show HEAD` 的已提交版本里，整段跨第 36-53 行，核心循环在约 46-50 行，当前工作区/快照里已经不存在这段代码。核对表把改前的语义描述贴在了改后代码占据的同一行号上 | `git diff -- crash.rs`；`git show HEAD:.../crash.rs \| sed -n '36,53p'` |
| `crash.rs:41-60`（read_into） | ✓ 逐字相同，含 `buffer.fill(0)` 与 `range` 迭代 | `sed -n '41,60p'` |
| `transaction.rs:1131-1134`（Z1-A Doc） | ✓ 逐字相同，限定词「只接得上 txg 2、jsn 2」「一个字节都不写」均在 | `sed -n '1131,1134p'` |
| `transaction.rs:1185-1189`（Z1-B Doc） | ✓ 逐字相同，「对象出生代与容器身份不改」在 | `sed -n '1185,1189p'` |
| `transaction.rs:506-511`（Z1-C Doc） | ✓ 逐字相同；「没有列现行版本带文件时会怎样」这句是核对之后补的事实陈述，核对表自己标了「补一句」，未隐瞒 | `sed -n '506,511p'` |
| `mount.rs:1089-1093`（Z1-D Doc） | ✓ 逐字相同 | `sed -n '1089,1093p'` |
| `mount.rs:1164-1170`（Z1-E Doc） | ✓ 逐字相同，「不在根环」「不在候选集」两个分句独立保留 | `sed -n '1164,1170p'` |
| `mount.rs:590-591,594`（Z1-F Doc，跳过 592-593 的 `# Errors` 标题行） | ✓ 逐字相同 | `sed -n '590,594p'` |
| `recovery.rs:1334-1336`（Z1-G Doc） | ✓ 逐字相同（含函数签名证实返回 `RecoveryReport` 非 `Result`） | `awk 'NR==1334,NR==1336'` |
| `history.rs:1219-1221`（Z1-A Generator） | ✓ 逐字相同 | `sed -n '1219,1221p'` |
| `history.rs:1259-1264`（Z1-B Generator） | ✓ 逐字相同 | `sed -n '1259,1264p'` |
| `history.rs:526,528`（两个 `MissingPrecondition` 成员名） | ✓ | `sed -n '526p;528p'` |
| `history.rs:1298-1304`、`:1301`（Z1-C Generator） | ✓ 逐字相同 | `sed -n '1298,1304p'` |
| `history.rs:529-532`（`CurrentVersionWithFile` 枚举成员完整文档注释） | ✓ 逐字相同——**这一处与 Sonnet 腿引同一段注释时给的区间（527-533）不同，本地腿这里给的 529-532 才是准确的**，互相印证了上一节 Sonnet 那条 ✗ | `sed -n '529,532p'` |
| `history.rs:1361-1364`（Z1-D Generator） | ✓ 逐字相同 | `sed -n '1361,1364p'` |
| `history.rs:1372-1390`（Z1-E Generator，三种取法的执行侧） | ✓ 逐字相同 | `sed -n '1372,1390p'` |
| `history.rs:342-357`（`draw_operation` 里 `source.below(5)` 五取值） | ✓ 逐字相同，比例「约五分之二/五分之二/五分之一」与 0\|1 / 2\|3 / 其余 三档吻合 | `sed -n '342,357p'` |
| `history.rs:1419,1421`（Z1-F Generator 公式） | ✓ 逐字相同 | `sed -n '1419p;1421p'` |
| `history.rs:183-188`（`FloorTargetChoice` 字段完整文档注释） | ✓ 逐字相同——**同样与 Sonnet 腿引同一段时给的区间（184-189）不同，本地腿这里的 183-188 是准确的** | `sed -n '182,190p'` |
| `history.rs:1448-1452`（Z1-G Generator） | ✓ 逐字相同 | `sed -n '1448,1452p'` |
| `history.rs:286-292`、`:292`（`operation_weights` 权重表、`PublishFirstFile` 权重 4） | ✓ 逐字相同 | `sed -n '286,292p'` |

小计：核了 22 处「原文文件:行」，✓ 21，✗ 1（`crash.rs:36-40` 的语义标注与当前代码状态不符）。

### 运行记录核对（`m2-supp3-item1-code-r1-local-attack-runlog.md`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| 三份样本 `wc -w` 数（s1 2008、s2 1604、void1 2465） | ✓ 三个数字逐一重算相同 | `wc -w` 三个文件 |
| `void1` 文件权限 600 | ✓ | `ls -la` |
| `corruption-check.py` 在 s1/s2/void1 上的判定（绿/绿/红，各自内部词数、void1「实词自复读: precondition」） | ✓ 三次独立重跑，退出码、内部词数（2002/1670/2523）、void1 判红原因逐字相同（`ask-local.sh` 内部退出码 5 与 `corruption-check.py` 自身退出码 1 是两层不同的编码，不矛盾） | `nice -n 19 python3 research/scripts/corruption-check.py` 三个文件 |
| `oov-check.py` 两参数版在 s1/s2 上（均绿，生词 0/拼接 0） | ✓ 两次独立重跑均相同 | `nice -n 19 python3 research/scripts/oov-check.py 样本 提示文件` |
| `oov-check.py` 单参数版在 s2 上（红，生词 1、拼接 1，均为 `TransactionOutput`） | ✓ 独立重跑相同 | 同上，仅一个参数 |
| `oov-check.py` 单参数版在 s1 上（运行记录写「红，生词 1、拼接 1，同样是 `TransactionOutput`」） | **✗**：独立重跑两次结果一致地显示 `生词=7 拼接=5`，`拼接` 列出 5 次 `TransactionOutput`、`生词` 另外还有 `version's`（文件中 `TransactionOutput` 出现 5 次、`version's` 出现 2 次，5+2=7 与生词数吻合）；运行记录这一格的数字看起来是从 s2 的正确结果（生词 1、拼接 1）复制过来的，没有对 s1 单独重跑核实。s1 判「干净」的结论本身不受影响，因为两参数版（与 `ask-local.sh` 内部同一种调法）已独立核实为绿；受影响的只是这一格用来解释「为何单参数版会误判」的具体数字 | `nice -n 19 python3 research/scripts/oov-check.py m2-supp3-item1-code-r1-local-attack-output-s1.md`（连续两次） |
| `precondition precondition` 定位 | ✓ 位于 void1 第 137 行，与损坏闸报的「实词自复读」类型吻合 | `grep -noP '\b(\w{5,})\s+\1\b' void1.md` |

小计：核了 7 组，✓ 6，✗ 1。

本地攻方腿合计：核了 29 处（核对表 22 + 运行记录 7），✓ 27，✗ 2；核不动 0。

## 四、三腿合计

核了 113 处（Opus 58 + Sonnet 26 + 本地攻方 29），✓ 108，✗ 5，核不动 0，「分不清：文件在腿交回之后被改过」0 处。
数字由 `awk -F'|'` 对本文件三张主表逐行取第二列首字符统计所得，不是手数：

```
awk -F'|' '/^\| [^-]/ && NF>=4 {col=$3; gsub(/^[ \t]+|[ \t]+$/,"",col);
  if (col ~ /^\*\*✗\*\*/) red++; else if (col ~ /^✓/) green++; else other++}
  END{print "green="green" red="red" other(表头)="other}' \
  research/prompts/m2-supp3-item1-code-r1-verifier-output.md
```
→ `green=108 red=5 other(表头)=5`（5 处表头不计入 113）。

五处 ✗ 汇总：
1. Opus：`logs/raw/logs-dev/N2-fast.log` 引文耗时 `35.54s` 实为 `B6-fast.log` 的数，`N2-fast.log` 实际是 `25.85s`。
2. Sonnet：`history.rs:184-189` 引文实际位置是 184-189 整体错位一行，真实内容在第 183-187 行（注释块共 183-188）。
3. Sonnet：`history.rs:527-533` 引文实际只对应第 529-531 行，527-528、532-533 不在引文内（该枚举成员完整注释是 529-532）。
4. 本地攻方核对表：`crash.rs:36-40` 标为「`read` 旧体，改前逐扇区 get」，而该文件是这一轮的未提交改动，现状/快照第 36-40 行是改后的瘦包装，旧逻辑只在 `git show HEAD` 里、跨第 36-53 行。
5. 本地攻方运行记录：s1 的单参数 `oov-check.py` 结果被写成与 s2 相同的「生词 1、拼接 1」，独立重跑两次均为「生词 7、拼接 5」（多出 `version's`）。

⚠️ 第 2、3 两条互相佐证：本地攻方核对表引用同两段注释时分别给出 183-188、529-532，与源码逐字核对后确认这两个区间才对；Sonnet 引用的 184-189、527-533 都有偏移，方向不一致（前者右移、后者两端外扩），不是同一种机械错误，像是分别核对时数漏或数多了一行。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 未对 Opus 腿全部 18 条变异逐条重跑复现（只重跑并核对了产物文件与日志内容，未在草稿目录里重新编译整套 `run-mutants.py` 流水线）；`crates/` 改动量与变异数较多，全量重跑超出这一轮核查的时间预算，已改为核对产物文件内容与其内部一致性（跨表交叉核对、独立统计窗口分布）。
- Opus 腿「四句」「射程」「分不分辨臂」等推理性文字、以及三条腿判决倾向（Z2/Z3/Z4/Z1/Z6 打中与否）均未核，按定义不归核查员判。
- 本地攻方腿的两份样本（s1、s2）具体答了什么内容、四栏（Z5 的 a/b/c/d、Z1 的 1/2/3/4）填得对不对，未核——那是语义判断，不在核查员核对表引用、产物、复跑的范围内。
- 未重新编译 Opus 腿的整个模型目录（`run-mutants.py`、`run-geometry.py` 等脚本本身的正确性未审查），只核了脚本产出的日志内容与报告引文是否一致。
- `root_ring.rs`、`.claude/kb/decisions/03-空间分配.md`、`crates/singlefs-core/src/recovery.rs` 三个文件不在给定快照清单内，按 `git status`/`git log` 判断这一轮未改过后对主树核，未使用专门的快照副本。
- 未跑门禁；未做 git 写操作；未改 `crates/` 或 `.claude/kb/` 任一文件；报告与草稿之外没有写其它路径。

## 文件与哈希

- 报告：`research/prompts/m2-supp3-item1-code-r1-verifier-output.md`（本文件）
- 草稿：`/tmp/claude-1000/m2-supp3-item1-r1-verifier/`（`snapshot-copies/` 两份 kb 原样副本、`sonnet-repro/` 独立仓副本含复跑产物、`selftest/` 判别力自证副本、`sha_check.txt`、`crash-HEAD.rs`）
