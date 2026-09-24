# 三方判决：增补 3 第 4 件「故障注入」代码轮第一轮（2026-09-21）

被判的是实现员交回的故障注入装置：`crates/singlefs-harness/src/fault_injection.rs`（新建 2053 行）与
`crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs`（新建 482 行），
以及它们牵动的 `crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/lib.rs`、
`crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 与 `crates/mutations.tsv`。

开工快照 `research/prompts/m2-supp3-item4-code-r1-start-snapshot.sha256` 9 个文件，核查员 `sha256sum -c` 全部 `OK`，
腿跑期间没有别的会话动过被判的文件。

## 〇、三条腿与核查员

| 腿 | 立场 | 产物 | 行数 |
|---|---|---|---|
| 云端攻方（Opus） | K1 注入装置自己的缺口 / K2 被测性质是不是被规则做漂亮了 | `m2-supp3-item4-code-r1-opus-output.md` + `m2-supp3-item4-code-r1-opus-model/` 10 个文件 | 378 |
| 云端正推（Sonnet） | K4 代码做的是不是条款说的 / K5 覆盖面 | `m2-supp3-item4-code-r1-sonnet-output.md` | 93 |
| 本地攻方（Qwen3-Next-80B） | K3 六种故障形态的实现 / K6 评分逻辑 | 四份样本 + 转述核对表 + 运行记录 | — |

核查员 `m2-supp3-item4-code-r1-verifier-output.md`：核 68 处，✓ 60、✗ 8。判别力自证过（把
`invariants.md:12` 的引用行号人为偏移到 13，方法正确判 ✗）。

**✗ 的 8 处里 6 处在 Sonnet 名下**，而且有两处不是行号偏一，是张冠李戴。主 agent 逐处现查坐实：

| 腿报的 | 现查到的 | 我跑的 |
|---|---|---|
| 注释「回卷只管取号自己那几次写报错」在 `mount.rs:849` | 849 行是 `MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction {`；注释在 **867** | `sed -n '849p;866,868p' crates/singlefs-core/src/mount.rs` |
| `PublishFirstFile` 权重 95 出自 `history.rs:492` | 492 行是 `(HistoryOperationKind::CloseAndMountWritable, 95),`；全文件 `PublishFirstFile` 取值只有 60、4、85、1、85、1、90、90，**没有 95** | `grep -n "PublishFirstFile," crates/singlefs-harness/src/history.rs` |
| `BlockDevice(_)` 在 `make_filesystem.rs:57-82` 的 79 行 | 真实行号 **84**，落在腿给的行区间之外 | `grep -rn "BlockDevice(BlockDeviceError)" crates/singlefs-core/src/make_filesystem.rs` |

⇒ **Sonnet 这一腿的定位可信度按「每处单独现查」处理**，它的结论句不作为依据直接采纳。
Opus 的 2 处 ✗ 都落在同一条语句内（`history.rs:832` 应为 833）或是产物缺一份，不改变论点。

## 一、打中的四条，逐条判

### 判定一：「注入之后就停」这条规则把结果做漂亮了 —— 成立，这一轮最重

今天的装置在注入那一步就停。攻方只改这一件事（让历史继续往下跑），同样 24 段、同样 95 个注入点：

产物整行抄：

```
research/prompts/m2-supp3-item4-code-r1-opus-model/baseline-fast.txt:116
以「已知红」收尾 {0: 3, 1: 4}、新发现 0

research/prompts/m2-supp3-item4-code-r1-opus-model/probe-k2-continue.txt:139
以「已知红」收尾 {0: 5, 1: 4}、新发现 38

research/prompts/m2-supp3-item4-code-r1-opus-model/baseline-fast.txt:114
  走到模型认下的某一版 77 次、走到失败那一步正在写的那一版 18 次（收口表第 40 行那一格，只记不判）、模型都不允许的 0 次

research/prompts/m2-supp3-item4-code-r1-opus-model/probe-k2-continue.txt:71
  走到模型认下的某一版 74 次、走到失败那一步正在写的那一版 7 次（收口表第 40 行那一格，只记不判）、模型都不允许的 14 次
```

⚠️ **里程碑第 4 件的设想实现明文要求「出错之后重开要恢复到模型允许的版本」，这条性质今天判了，而且判绿**
（`baseline-fast.txt:114` 那行 0 次）。让历史继续往下跑之后，同样 95 次重开里 14 次落进「模型都不允许」。
⇒ 打中的不是「装置漏判了一条性质」，是**这条性质判绿的前提是注入即停**。

两次跑的注入点分布逐行相同（`barrier_fails` 23、`read_fails` 28、`read_returns_corrupted_bytes` 18、
`write_fails` 26，合计 95），差别只在「停不停」。

**判定：成立。** 「快档 95 注入点、panic 0、新发现 0」这个结果里，
「新发现 0」主要来自注入即停这条规则，不是实现真的这么干净。panic 0 那一半不受影响（两份产物都是 0）。

⚠️ 这一条打中的是**装置的规则**，不是 `crates/singlefs-core/`。它不推翻第 4 件的验收，
它推翻的是「第 4 件已经把这一片罩住了」这句话。

### 判定二：今天的 checker 判得出东西，只是没机会跑到 —— 成立

攻方再把模型对拍整个挂起、只留 checker 与 panic，仍有新发现：

```
research/prompts/m2-supp3-item4-code-r1-opus-model/probe-k2-suspend-model.txt:110
以「已知红」收尾 {0: 5, 1: 4}、新发现 22
```

22 条里有两条是 checker 自己判出来的，不依赖模型：

- 种子 …119，`I-3.1（已分配统计对得上）`：盘 0 记账的已分配 `Some(606208)`，遍历全部有效根得到 `540672`
- 种子 …133，`I-7.8（根记录树 ID 水位不低于全池最大树 ID）`：根环水位最大 11，盘上出现过的最大树 ID 14

**证据缺口与它的处置**：攻方指路的 `probe-k2-single-seeds.txt` 只含种子 133，
种子 119 的单独复现不在那份产物里（`reproduce.sh` 第 5 步的种子列表漏了它）。
核查员在草稿副本上用同一份 `probes.patch`、同一组环境变量独立补跑了种子 119，
逐字复现了 `probe-k2-suspend-model.txt` 里那一条。⇒ **事实为真，证据文件缺一份**，
入库时按 `.claude/rules/three-way-inference.md`「副本上的数不进 kb」重跑一次入库装置。

**判定：成立。** checker 今天 29 条（`invariants.md:12`「池级 checker（`walk::check_pool_image`）判 29 条」，
共 69 条在用，`invariants.md:17`），这 22 条新发现用的就是这 29 条。

### 判定三：`recovery.rs:244` 一块盘两槽都坏就整池挂不上 —— 成立，但今天摆不出来

```
crates/singlefs-core/src/recovery.rs:244
            (None, None) => return Err(RecoveryFailure::NoValidSystemConfiguration { device }),
```

`choose_system_configuration` 在一块盘的两个系统配置槽都读不出来时当场 `return Err`，
**不看别的盘已经读到的那一份**。而 `D22（单元原子性怎么合成）` 已定项 8 第 1 条
「系统配置每盘放一份」买的正是这份冗余。

攻方写了一个独立测试坐实这一格，核查员独立复跑过：

```
test both_system_configuration_slots_dead_on_one_device_makes_the_whole_pool_unrecoverable ... ok
test one_failing_read_on_one_system_configuration_slot_still_recovers ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

**判定：成立，而且是这一轮唯一一条打在 `crates/singlefs-core/` 上的。**
今天的随机注入永远摆不出这一格：一个 plan 只响一次，而这一格要两次读失败落在同一块盘。
⇒ 它不是第 4 件的缺陷，是第 4 件**照见的**一笔代码欠账。

### 判定四：六种故障形态里两种一次都没被抽到 —— 成立，成因在一行代码里

产物整行抄（`baseline-fast.txt:72-75`，24 段 95 次注入的全部形态）：

```
  注入 barrier_fails：23 次
  注入 read_fails：28 次
  注入 read_returns_corrupted_bytes：18 次
  注入 write_fails：26 次
```

四种合计 95，`WriteIsSwallowed` 与 `BarrierIsSwallowed` **各 0 次**。成因：

```
crates/singlefs-harness/src/fault_injection.rs:1092
    let variety = usize::try_from(random.below(4)).expect("小于 4");
```

`below(4)` 四选一，两种「说谎的设备」写在枚举里、有各自的孤立单测
（`a_swallowed_write_reports_success_and_changes_nothing_on_the_device`、
`a_swallowed_barrier_reports_success_and_is_counted_apart`），但进不了随机样本。

**判定：成立。** 两种孤立单测证明装置注得进去，不证明被测代码在这两种故障下的行为被看过。

### 判定五：49 次 panic 是执行器自己的 `expect`，不是 `singlefs-core` 的缺口 —— 成立

攻方另一个探针里 `注入之后 core panic 的次数：49`，位置全在
`crates/singlefs-harness/src/history.rs:833`（`make_filesystem(...).expect(...)` 那条跨行语句的字符串字面量行；
攻方正文写 832，是同一条语句的 `.expect(` 那一行，核查员判行号偏一）。

这正好从反面坐实起点段今天注不进去：

```
crates/singlefs-harness/src/fault_injection.rs:1125
        let candidate_steps: Vec<usize> = (1..marks.len())
```

`draw_faults` 从 `1` 起算，`marks[0]`（起点段：mkfs、取号、暖机、第一个文件）结构性地不在候选里。
`HistoryPool::start` 里那四步各是一个 `.expect`（`history.rs:832` mkfs、`:863` 取号、`:865` 暖机、`:878` 第一个文件），
所以就算放开候选区间，注入在起点段今天也只会撞出执行器自己的 panic。

**判定：成立。两件事要一起改，只改候选区间会把 panic 0 这个结果弄脏。**

## 二、写回代码的改法

用户 2026-09-21 定案三条。落地次序按「改完能立刻验」排，不按重要性排。

### 改法一：两种「说谎的设备」立成故障模型，抽进随机样本

- `draw_injected_fault` 的 `below(4)` 改成 `below(6)`，补 `4 => InjectedFault::WriteIsSwallowed`、
  `5 => InjectedFault::BarrierIsSwallowed` 两臂，`unreachable!` 的消息跟着改。
- ⚠️ **配套要改评分**：`fault_injection.rs:1082-1090` 的文档注释写着「丢一次写要配崩溃注入一起用」——
  一次被吞的写在历史继续往下跑之前不会有任何可观测后果，判定档要能收下「注入了、什么也没发生」这一种，
  不能记成「没走到」。
- **验收**：快档重跑，六种形态各自的注入次数都大于 0；产物里那四行变成六行。
- **变异**：`crates/mutations.tsv` 加一条——`below(6)` 改回 `below(4)`，点名的用例必须红。

### 改法二：放开起点段

- `draw_faults` 的 `(1..marks.len())` 改成 `(0..marks.len())`，`fault_injection.rs:1123-1124` 那两行注释删掉。
- `HistoryPool::start` 里四处 `.expect` 改成可失败：`make_filesystem`、`acquire_instance`、`warm_up`、
  `publish_first_file` 各自把错误交回给注入判定，而不是 panic。
- ⚠️ **这一条改完之前，判定五那 49 次 panic 会照样出现**，两处必须同一次改完。
- **验收**：起点段注入点从 0 涨到大于 0（攻方给的 108 次设备调用是它在副本装置上算的，
  按 `.claude/rules/three-way-inference.md`「副本上的数不进 kb」，入库时重新量一次再写进实验页）；
  `core panic 的次数` 仍为 0。
- **变异**：把 `(0..` 改回 `(1..`，点名的用例必须红。

### 改法三：补 checker

今天 29 条已实现、69 条在用。判定二证明这 29 条已经判得出东西，缺的是跑到那些状态的机会。
⇒ **先补的不是条数，是「让历史往下跑」这个开关**（判定一：一行开关换 22 条新发现，用的是现有的 29 条）。
具体补哪几条 checker、按什么次序，等用户看过判定一的数再定（交用户表第 3 行）。

### 不进这一轮的：`recovery.rs:244`

判定三打在 `crates/singlefs-core/` 上，与第 4 件的验收标准无关。
⇒ 记进 `.claude/kb/checks-owed.md` 新立一条，题面「一块盘两个系统配置槽都读不出来时不看别的盘已读到的那一份，
`D22（单元原子性怎么合成）` 已定项 8 买的冗余兑现不了」，前置写明今天的随机注入摆不出这一格。

## 三、交用户表

| # | 题 | 状态 | 为什么要用户定 |
|---|---|---|---|
| 1 | 两种说谎的设备抽进样本 | 用户 2026-09-21 已定 | — |
| 2 | 放开起点段 | 用户 2026-09-21 已定 | — |
| 3 | 「让历史往下跑」要不要排在补 checker 前面 | **开着** | 用户定的次序是「优先补 checker 然后开新线」，而判定一、判定二是在那之后交回的：一行开关换 22 条新发现，用的是现有的 29 条 checker。排序要用户重定 |
| 4 | `recovery.rs:244` 那一格什么时候修 | **开着** | 打在 `singlefs-core` 上，不在第 4 件射程内 |

**被攻过零轮的形态**（按 `.claude/rules/three-way-inference.md` 登记）：
改法一的「被吞的写怎么评分」、改法二的「四处 `.expect` 改成可失败之后起点段的判定档」——
两样都是攻方腿提的收严，只在它自己的探针上量过，没有任何一轮攻过它们。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-09-21 不受影响：判定三打中的是实现没兑现这条分项买的冗余，分项本身的定案、射程与依据都不动 |

## 五、点名这次改动的每个文件（门禁 56 号）

| 路径 | 判了什么 |
|---|---|
| `crates/singlefs-harness/src/fault_injection.rs` | 判定一、四、五（注入即停、`below(4)` 四选一、`(1..marks.len())`）；改法一、二都落在这里 |
| `crates/singlefs-harness/src/history.rs` | 判定五（`:833` 那条 `.expect` 与起点段另外三处）；改法二落在这里 |
| `crates/singlefs-harness/src/lib.rs` | 只新增模块声明，三条腿都没提出问题，核查员没有核到与它相关的引用 |
| `crates/singlefs-harness/src/bin/first_transaction_on_device.rs` | 本地攻方 K3 Fact 10 核过 `swallow_the_next_barrier_on` 函数体逐字相符；与判定四相关（两种说谎的设备在这里有手写调用点），不改 |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs` | 快档用例本身；判定一、二要改它的停机规则 |
| `crates/mutations.tsv` | 197-204 行八条；改法一、二各要补一条 |
| `crates/singlefs-core/src/transaction.rs` | 在开工快照里（第 202 行变异钉的是它），这一轮没有一条腿打中它 |
