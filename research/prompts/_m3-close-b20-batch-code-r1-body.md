# m3-close-b20-batch-code-r1 正文：第 20 批修补之后七份产品补丁的代码轮（2026-11-10）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 20 批。方案按 `crates/` 今天的实现来谈。上一轮判决 `research/prompts/m3-close-b20-repairs-code-r1-main-verification.md`，它第四节写「第五节的全部改法……改完之后下一轮代码轮攻它们」，这一轮攻的就是那批改法，加上之后用户定案带来的三份。

## 被判的产品代码

diff 全文在 `research/prompts/_m3-close-b20-batch-code-r1-diff.md`（七份补丁按打补丁次序，只摘 `crates/singlefs-core/src/` 下的文件）。

| 文件 | 范围 | 来历 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 第 1478 行 `first_txg_of_new_instance` 多收读阶段择到的根（H1、判退之后的下界）；第 3541 行 `roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage` 与第 3078 行 `rebuilt_allocator` 的根表（I-7.4、Q 缺口）；第 1638 行 `instance_table_of_the_newest_root_read_at_most_twice` 只收读阶段择到的根、删掉读阶段之后那次 `choose_root`；第 414 行 `InstanceTableOfTheNewestRootForTheShadowLedger` 的字段改成 `root_chosen_by_the_read_stage: RollbackTarget` | 补丁 `m3-close-b20-repairs-fixes`、`-2`、`-3`、`fallback-i74`、`read-stage-roots`、`instance-table-read-stage`、`shadow-ledger-refusal-field` |
| `crates/singlefs-core/src/recovery.rs` | 第 8377 行 `recover_of_the_pool`（H3）；第 2067 行 `choose_root_keeping_every_self_verified_root_read`，第 2048 行 `choose_root` 改成调它 | `repairs-fixes`、`read-stage-roots` |
| `crates/singlefs-core/src/mounted_read.rs` | 第 665 行 `mount_read_only_of_the_pool`（H3） | `repairs-fixes` |
| `crates/singlefs-core/src/system_configuration_repair.rs` | 第 224、274、332 行：断言第 0 代且根读得出时 tail 取 mkfs 写的值（H6）；暖机根在时断言第 0 代照拒 | `repairs-fixes`、`repairs-fixes-2` |

点名分工：`mount.rs`、`mounted_read.rs` 此刻也在别的会话的代码轮 m4-close-batch2-code-r2 开工快照里（它改了这两份里别的地方，A1、A2、A3），最终内容由那一轮点名；这一轮照样攻第 20 批在这两份里的 diff。`recovery.rs`、`system_configuration_repair.rs` 由这一轮点名。

## 实现今天的样子（主 agent 的观测）

- 读过：上表各行号处的函数，与 diff 文件里的十处 `diff --git`。
- `choose_root` 现在调 `choose_root_keeping_every_self_verified_root_read`，只取择到的那一条；产品代码里 `mount.rs` 不再 `use` `choose_root`（`instance_table_of_the_newest_root_read_at_most_twice` 不再现择）。
- 可写挂载每次少读一整环根槽；验证包里 5 个读次数常量改了值（6→5、8→7、8→7、整池读序 307/308→283/284，后一对是实现员推的），报告 `~/.local/share/singlefs/work/m3-close-b20-instance-table-read-stage/result.md` 第 3 节。
- 已有的量：合入后验证 12052 测试那一步 130 个测试二进制 2343 条全绿、clippy、build 过；挂特性那一步卡住被杀，GPU 判器一侧，不在这一轮射程（调查 `~/.local/share/singlefs/work/m3-close-b20-gpu-lib-stall/result.md`）。证红 12037、12038 四行都抓到。

## 压着的条款

- 判决 `research/prompts/m3-close-b20-repairs-code-r1-main-verification.md` 第二、三、五节（H1–H6、B1）。
- 用户定案：Q 缺口「这一批补」、判抛弃用的实例表「照读阶段择到的根的表」，原话 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2665 行；判退那一形也用读阶段择到的 R，主 agent 定案同一份记录第 2678–2683 行。
- `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8 射程（第 185 行起；写回员正在改它，改前那一版是开工快照里的）；`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 3 条（第 373 行）；`.claude/kb/checks-owed.md` C492、C971 两行。

## 要两条腿回答的问题

1. 正推腿：逐条核代码做的是不是条款说的。H1 取 max 收的是 `chosen_root` 不是 `effective_root`（理由在 `m3-close-b20-repairs-fixes/result.md`「H1 为什么收 `chosen_root`」）；判退之后下界收读阶段择到的根；H3 只读挂载与恢复收本池 fsid；H6 离线断言第 0 代；Q 缺口并读阶段读到的全部根、按 (实例代号, checkpoint_txg) 去重；实例表照读阶段择到的根，判退那一形用 R。
2. 攻方腿，不重复上一轮 H1–H6 已攻的角度，攻这几面：
   - 读阶段读到的全部根进了重建分配器那几遍：一条读阶段自证过、之后几遍读坏的根，和一条读阶段就没读到的根，在影子账、回收门槛、根环表里的待遇有没有不一致；去重键漏了什么。
   - 删掉读阶段之后那次现择：读阶段择到的根之后被判抛弃、或它的槽之后被写盖（同一次挂载里），实例表还照它取对不对；读次数少一环让哪条「重读一次」的承诺变了。
   - `root_chosen_by_the_read_stage` 改成不可为空之后，原来给 `None` 的那几处测试改成真值，断言是不是还判它们原来要判的事。
   - H6 的第 0 代出路：除新格式化池撕裂取号那一形，别的盘面（暖机根、只读挂载过、跨实例）会不会也走进这条出路写出不该写的系统配置。
   - H3 两个新入口与旧入口并存：调用方里还有没有该收 fsid 而走旧入口的产品路径。
3. 两条腿都对下面八格逐格回「成立 / 不成立（文件:行）/ 推不出」。

### 检查单：走不到的理由二次查实

七份实现员报告「走不到的理由」一节都写「无」（各草稿目录 `result.md`：`m3-close-b20-repairs-fixes`、`-2`、`-3`、`fallback-i74`、`read-stage-roots`、`instance-table-read-stage`、`shadow-ledger-refusal-field`）：产品代码没新加 `todo!`、`unreachable!`、`assert!` 与条款没定的错误成员；拒用的是已有成员 `StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords`。腿照调用点核这句话。

### 检查单：类型与名字

新名：`recover_of_the_pool`、`mount_read_only_of_the_pool`、`choose_root_keeping_every_self_verified_root_read`、`roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage`、字段 `root_chosen_by_the_read_stage`、参数 `root_chosen_by_the_read_stage`（`rebuilt_allocator`）。腿核 fsid、实例代号、txg 是不是 newtype，有没有 `_ =>`、缩写，新加的 `expect` 写没写清依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

H3 加了两个新入口、旧入口不改签名（`repairs-fixes/result.md` 逐条结局第 2 行）。腿核旧入口留着是不是只为兼容旧调用方、该不该删。

### 检查单：不新建判定库

本轮改动没有这一类（查过七份补丁 `crates.patch` 的改动行）。

### 检查单：所有任务多机可用

新用例都用内存盘，不读本机状态；GPU 库单测卡住那一件在两台都出现过（12052 本机、11977 第 1 次第二台），不是这一轮的产品代码。腿核新用例有没有读本机独有状态。

### 检查单：测试优先多线程

新用例是单个盘面的确定场景，不切片；读次数改动只改常量。腿核有没有该并行而没并行的。

### 检查单：带 --item 的命令与项

实现员报告里的 crates 变异 `--item` 点了新加与换锚的行（`read-stage-roots/result.md` 第 5 节、`instance-table-read-stage/result.md` 第 6 节），合入后验证已把这几行的点名测试目标跑到；`--item` 那一趟还没交。腿核命令缺不缺项、新项列不列得出。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或登记的崩溃枚举用例（查过七份补丁新加的测试函数，都在已有的测试目标里）。读次数改动影响按读次数注入故障的崩溃枚举用例 `random-histories-and-fault-injection-given-states`，结局变不变要跑才知道。
