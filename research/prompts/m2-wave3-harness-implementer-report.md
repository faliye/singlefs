# 批 B harness 那一组（收口表第 21、45、47、57 行）实现员交回

接手：前一个实现员（a6660031b4890a181）撞会话额度中断，本份由接手的实现员写。时刻都是 UTC。
主工作区一字未动：改动全在副本 `/tmp/claude-1000/impl-harness-batch/repo/`，补丁 `/tmp/claude-1000/impl-harness-batch/harness.patch` 只含 `crates/singlefs-harness/` 下 9 个文件；新变异 7 行在 `/tmp/claude-1000/impl-harness-batch/mutations-append.tsv`，没写进 `crates/mutations.tsv`。

## 一、四件逐条

| 件 | 做到没有 | 落在哪 |
|---|---|---|
| 第 21 行 C366 会红形态 | **没做，停下交主 agent**（第五节第 1 条）。挂载层分得开 txg 与计数器的盘面只有一种：所选根自己那条记录两份都读不出（C500 那一格），而那一格上「前缀末」是哪一条没有条款 | 只有草稿探针，不进补丁 |
| 第 45 行归批 B 的三样 | 做到：步 1 三条变异、步 6「按每次发布报状态数」、步 3「取号之后没有屏障」的变异（连它要红在的一条新层 0 用例） | `crash.rs`、两条流的层 0 用例、`second_transaction_step_one_overwrite.rs`、新文件 `second_transaction_step_three_acquisition_barrier_layer0.rs`；变异 5 行 |
| 第 47 行 C481 | 做到：坏盘输入的基线按档抽，加「树表 0 条」一档，判据不变；报告里报基线档数那一句由用例钉住 | `bad_disk_input.rs`、`second_transaction_supplement_three_bad_disk_input.rs`；变异 1 行 |
| 第 57 行 C378（认了） | 做到：取号之后写行、暖机那一次写报块设备错，断言重开的盘上系统配置还是新号、记进故障注入的账、报告里点名；改成回卷必须红 | `fault_injection.rs`、`second_transaction_supplement_three_fault_injection.rs`；变异 1 行 |

## 二、这一轮写过的文件

副本里（补丁就是这 9 个文件相对主工作区现状的 diff）：

- `crates/singlefs-harness/src/bad_disk_input.rs`
- `crates/singlefs-harness/src/crash.rs`
- `crates/singlefs-harness/src/fault_injection.rs`
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`
- `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`
- `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs`
- `crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs`（新文件）

前一个实现员写了上面全部 9 个；接手之后我改了其中两个：`second_transaction_supplement_three_fault_injection.rs`（C378 用例多钉一条「错误成员恰好记了一次」，原来那条 `.all(...)` 在空表上恒真）、`second_transaction_step_three_acquisition_barrier_layer0.rs`（文件头写明它与第二条流的关系，多钉段序列与展开的段号）。

`mutations-append.tsv`（7 行，变异名）：

1. 步 1 验收第 4 条：extent 叶记录的指针忘了换（覆盖写之后仍指上一版的数据单元，读回等于旧内容）
2. 步 1 验收第 4 条：inode 记录的写入时间留成 A 的（覆盖写照抄上一版的写入时间）
3. 步 1 验收第 4 条：反向链算成 A 之前那条（覆盖写的反向链取上一版记录自己的反向链）
4. 步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）
5. 步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）
6. C481：坏盘输入的基线抽样里去掉「树表 0 条」那一档（报出的基线档数由 2 变 1）
7. C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）

第 1–4、7 行改的是 `crates/singlefs-core/src/transaction.rs` 与 `mount.rs`（变异只在门禁的临时副本里改，产品代码不动）；第 5 行改 `crash.rs`，第 6 行改 `bad_disk_input.rs`。新名字、补丁、这 7 行里都没有角标字符（U+2032、U+2033、U+2034、U+02B9、U+02BA，脚本扫过，零处）。

补丁对主工作区现状的 `git apply --stat`（主工作区别的会话也在改 `crates/`，所以不贴 `git diff --stat`，它分不出谁改的）：

```
 crates/singlefs-harness/src/bad_disk_input.rs      |  247 ++++++++++++++++----
 crates/singlefs-harness/src/crash.rs               |  125 ++++++++++
 crates/singlefs-harness/src/fault_injection.rs     |  217 +++++++++++++++++-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +
 .../tests/second_transaction_step_one_overwrite.rs |   16 +
 .../tests/second_transaction_step_zero_layer0.rs   |   39 +++
 ..._transaction_supplement_three_bad_disk_input.rs |   38 +++
 ...transaction_supplement_three_fault_injection.rs |  120 +++++++++-
 ...action_step_three_acquisition_barrier_layer0.rs |  207 +++++++++++++++++
 9 files changed, 958 insertions(+), 67 deletions(-)
```

## 三、每条新测试怎么证明会红

做法：`drafts/mutant/` 是 `repo/` 的一份副本（`rsync -a --exclude target --exclude .git`，之后 `touch` 全部 `.rs`），用它自己的 `target`。先跑基线，再逐条把 `mutations-append.tsv` 那一行改进去、跑那条测试所在的整个测试二进制、从 `repo/` 拷回原件并 `touch`。接手之后整批重跑一遍（22:15–22:36），日志在 `drafts/logs/`，汇总在 `drafts/rerun-batch.txt` 与 `drafts/rerun-barrier.txt`。

**基线红集：空。** 6 个测试二进制与 `--lib crash` 全绿：

| 二进制 | 基线 |
|---|---|
| second_transaction_step_one_overwrite | 12 passed |
| second_transaction_step_three_acquisition_barrier_layer0 | 1 passed（改完文件头与段序列断言之后重跑） |
| second_transaction_step_zero_layer0 | 8 passed、1 ignored |
| second_transaction_supplement_three_bad_disk_input | 9 passed、1 ignored |
| second_transaction_supplement_three_fault_injection | 8 passed、1 ignored |
| first_transaction_step_seven_layer0 | 5 passed、1 ignored |
| `--lib crash` | 15 passed |

改坏哪一行 → 哪条断言红（每行一个变异，整个二进制里同时红的全列）：

| 变异 | 改坏哪里 | 红在哪条断言 | 同一个二进制里同时红的 |
|---|---|---|---|
| 1 extent 指针不换 | `transaction.rs` 覆盖写那一臂在 extent 叶里放回上一版的数据指针 | `cold_start_reads_the_second_content_and_the_pool_checker_stays_green`（`second_transaction_step_one_overwrite.rs:320`「冷启动择 B 的根、读回第二次的内容」，读者报「声明长度与 inode size 不符」） | `damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one`（:413） |
| 2 写入时间留成 A 的 | `write_time_seconds: file.write_time_seconds` → `previous.inode_record.write_time_seconds` | 新加的那条（:363「盘上 inode 记录的写入时间是 B 那一次写入的时间」，left 1788000000 right 1788000060）：从只读挂载那条读路径读盘上的 inode 记录，不看发布交回的内存结构 | `overwrite_publishes_the_second_version_through_the_same_commit_shape`（:164，早就有、查的是内存里的发布结果） |
| 3 反向链算成 A 之前那条 | `back_chain_of(&previous.record_bytes)` → `previous.record.back_chain` | `cold_start_…`（:371，池级 checker I-8.6 判红：「盘 0 journal 环槽 3 的记录（实例 1、计数器 4）反向链 0x2571d162，本实例内逻辑前一条的头算出来的是 0xecba5df2」） | `overwrite_publishes_…`（:118「反向链 = A 那条记录头的 CRC32C」） |
| 4 取号之后没有屏障 | `acquire_instance` 删掉写完系统配置之后那道 `CommitStep::Barrier` | 新文件 `second_transaction_step_three_acquisition_barrier_layer0.rs:145`：I-7.7 判红 12276 个状态（「盘 0 槽 50304 的单元写序带实例代号 2，高于各盘系统配置的最大者 1（①）」） | 这个二进制只有这一条 |
| 5 按发布分状态数错一位 | `publish_of_each_segment` 把根槽写那一段归到下一次发布 | `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims`（`second_transaction_step_zero_layer0.rs:472`「平时跑的那 108 个状态按发布分」，`instance1_txg1=6 … after_the_last_root=4` 对 `=7 … =3`） | 无 |
| 6 去掉树表 0 条那一档 | `BaseImageTier::TreeTableWithoutEntries => false` | `bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites`（`second_transaction_supplement_three_bad_disk_input.rs:145`，报告那一句变成「基线档 1 / 2 档抽到过镜像：「三棵树都在」12 段、坏镜像 203 份；「树表 0 条」0 段、坏镜像 0 份」，left 1 right 2） | 无 |
| 7 改成回卷 | `mount_writable` 在 `establish_instance` 报 `MountError::Publish(_)` 之后把系统配置的实例代号写回旧号 | `a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it`（`second_transaction_supplement_three_fault_injection.rs:719`「取号之后第 3 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1」，left `[]`） | 无 |

七条红的都是测试自己的断言，不是被测代码里的 `debug_assert`（日志里 panic 点全在 `tests/` 下）；没有另跑 `--release`。
门禁 59 号只跑这 7 行（`drafts/gate-root` 里把 `crates/mutations.tsv` 换成表头 + 这 7 行，`SINGLEFS_GATE_FULL=1`，4 个工作进程），22:31:46 开跑、22:36:36 跑完，原样：

```
  ✓ 步 1 验收第 4 条：extent 叶记录的指针忘了换（覆盖写之后仍指上一版的数据单元，读回等于旧内容）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 步 1 验收第 4 条：inode 记录的写入时间留成 A 的（覆盖写照抄上一版的写入时间）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 步 1 验收第 4 条：反向链算成 A 之前那条（覆盖写的反向链取上一版记录自己的反向链）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）：no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount 红了
  ✓ 步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）：every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims 红了
  ✓ C481：坏盘输入的基线抽样里去掉「树表 0 条」那一档（报出的基线档数由 2 变 1）：bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites 红了
  ✓ C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）：a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it 红了
  ✓ crates 变异表复跑：7 条变异各自红在点名的测试上（原文都恰好命中一次；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数无关）
exit 0
```

注：门禁 59 号那一趟跑的时候屏障用例已经是最终版；`drafts/rerun-batch.txt` 里第 4 条那一次是改文件头之前的版本，改完之后 `drafts/rerun-barrier.txt` 又跑了一遍基线与变异（红在 :145，同一条断言，行号因文件头多了 5 行而后移）。

## 四、各件的实况与读数

### 步 6「层 0 按每次发布报状态数」

`crash.rs` 加 `Layer0PublishOfState` 与 `publish_of_each_segment`：一段归「它往后数第一次根槽 FUA 写」写出的那次发布；最后一次根槽写之后的段归 `after_the_last_root`，全部持久那一个状态单列 `every_write_persisted`。`Layer0Tally` 加 `states_by_publish`（按片并时逐格相加），`states_by_publish_text()` 出一串 `instanceI_txgT=N`。两条流都报、都钉：

- 第一条流全量（门禁 54 号 `--full` 跑的那条）：`instance1_txg1=7 instance1_txg2=7 instance1_txg3=262147 after_the_last_root=3 every_write_persisted=1`，合 262165；快用例 22 个状态同一套归法。
- 第二条流快用例（平时的 `cargo test`）：108 个状态按 17 次发布分，钉成一整串，另钉各格之和 = `states`。
- 第二条流全量：钉成 `instance1_txg1=7 … instance3_txg17=262147 after_the_last_root=3 every_write_persisted=1`，合 2104413。⚠️ **全量没跑过**（标 ignored，归门禁 54 号 `--full`）。草稿里拿同一个归法（`publish_of_each_segment`）逐段加 2^|段| − 1 算了一遍（`drafts/scratch-by-publish.txt`，草稿用例不进补丁），与断言里那串逐字相同、合 2104413；第一条流同样逐字相同。这只核了「断言那串 = 这个归法下的闭式」，枚举真走一遍各格是不是这些数，要等 `--full`。
- LAYER0、LAYER0B 两行各多一个字段 `states_by_publish=[…]`，夹在已有字段之后（LAYER0B 夹在逐条不变量与 `first_violation` 之间）。门禁 54 号只认行首 `LAYER0 ` / `LAYER0B ` 与行里有 `exhaustive=true`，这两条都还成立；它的全绿标记里存的是这两行原样，下一次 `--full` 写的标记会带上新字段。

### 步 3「取号之后没有屏障」的变异与它要红在的层 0 用例

第二条流的快用例只展开写数小于 10 的段；少了取号那道屏障，取号的 4 写一段与写行的 10 写一段并成 14 写一段，快用例不展开，全量要到门禁 54 号 `--full` 才罩得到。所以另起一条层 0 快用例 `second_transaction_step_three_acquisition_barrier_layer0.rs`，展开哪两段按写在录制流里的位置挑（重开之后第一个写所在的段、第一个单元写所在的段），不按写数挑。

照定义第 7 条写清它与已有流的关系：

- 流：mkfs → 取号 1 → 暖机 × 2 → A → B → 进程退出、重开可写挂载（取号 2 → 写行 → 暖机 × 2）。段序列 `2+2+1+2+2+1+18+2+1+18+2+1+4+10+2+1+10+2+1+10+2+1+2`：前 22 段与第二条流逐段相同，末段 2 写是暖机第二次的系统配置槽轮换（第二条流里它与 C 的单元写并成 18 写一段）。用例把这个数组与展开的段号 (12, 13)（从 0 数；从 1 数是第 13、14 段）都钉住。
- 多罩的崩溃状态：**零**。展开的两段在第二条流里是第 13、14 段，全量已经罩着；展开出来 1 + 15 + 1023 = 1039 个状态，都是第二条流全量的子集。多的是**在平时的 `cargo test` 里**展开那一段 10 写、按 I-7.7 逐状态判（另钉 I-7.7 在每个状态上都评估过）；没多跑哪一步（重开、挂载、恢复与第二条流相同）。
- 没新开全量枚举。基线镜像同是 mkfs 之后那一份；写表只钉了段序列，B 的文件内容与第二条流是不是同一串字节没比。

### 步 1 三条变异

三条都红在已有的 `cold_start_…` 上。只加了一条断言：从只读挂载（`mounted_read::mount_read_only`，与发布路径不共用代码）读回第一个文件那条 inode 记录的写入时间 = B 的。反向链那条红在池级 checker I-8.6 上——收口表第 45 行写「反向链那条等（要 I-8.6（反向链算法） 的 checker，第 26 行）」，今天那个 checker 在，这一条不用再等。extent 指针那条读者报的是「声明长度与 inode size 不符」而不是「读回旧内容」（B 的 4100 字节与 A 不同长，先撞上长度检查），验收那句的字面「读回等于旧内容」不成立，红照样红。

### C481：坏盘输入按档抽基线

`bad_disk_input.rs` 加 `BaseImageTier`（两档：「三棵树都在」、「树表 0 条」）与 `EVERY_BASE_IMAGE_TIER`，每档一个蓄水池、各抽一份，每份跑全部坏法，判据不变（不许读回没提交过的内容、不许 panic 在已知点之外）。「三棵树都在」那一档一步都不合格时照旧退回最后那一步；「树表 0 条」那一档一步都没有就这一段不抽、不退回。第二档的随机源另异或一个盐，第一档的盐是 0——第一档每个种子抽到的镜像、坏成的样子按构造与加档之前相同（**只是按构造，没有一条用例拿加档前后的报告逐字比**）。计数加 `base_images_by_tier`、`damaged_images_by_tier`，报告多一句「基线档 N / M 档抽到过镜像：…」，发现（finding）带上档名。

基线跑出来的那一句（12 段历史）：`基线档 2 / 2 档抽到过镜像：「三棵树都在」12 段、坏镜像 203 份；「树表 0 条」8 段、坏镜像 40 份`。用例钉：抽到过的档数 = 2、报告里有「基线档 2 / 2 档抽到过镜像」这一句、每档都真造出过坏镜像、逐档之和 = 逐坏法之和。

### C378：取号之后写行、暖机报块设备错（认了）

`fault_injection.rs`：测量跑每一步记下系统配置择到的实例代号；注入那一步是挂载、返回了错误、重开的盘上系统配置的号比这一步之前大，就记一格 `AcquiredInstanceLeftAfterAFailedMount`（两格：新号一条根都没有＝烧掉；新号已有写行那次的根），进 `acquired_instances_left_after_a_failed_mount` 这本账、报告里点名一句「取号之后挂载报错、新号不回卷（C378 …：用户 2026-09-23 认下的已知行为，D23 已定项 16）：N 次」，不进新发现、不挡别的判定。新入口 `inject_one_fault_into_the_segment` 让用例把注入点摆在点名的那一次调用上。

用例（`a_write_error_after_the_acquisition_…`）：起点「第一个文件之后」、只做一步可写挂载，两格各一次——可写挂载第 3 次写（写行那次的第一个单元写）报错 ⇒ 烧掉那格、根环最大实例 1；第 18 次写（暖机第一次的第一个单元写）报错 ⇒ 新号已有写行的根、根环最大实例 2。两格都钉：重开之后系统配置是 2（之前 1）、账上这一格记 1 次、报告里那一句在、注入点是 `CloseAndMountWritable/unit_write`、那一步返回错误且错误成员恰好一次、是 `MountError::Publish(…)`、没有 panic、没有新发现。

随机故障注入的几段在基线里也撞上了这一格（同一个二进制的基线日志）：一段 9 次（烧掉 4、有根 5），另一段 2 次（都是烧掉）。

## 五、停下交主 agent 的

### 1. C366 的会红形态：挂载层唯一分得开 txg 与计数器的盘面落在条款没写的那一格

代码：`crates/singlefs-core/src/mount.rs:309` `first_txg_of_new_instance` = max(根环全部根的 txg, 环里全部自证过的记录的 txg) + 1；`mount.rs:1276` `next_counter` = 环里记录的最大计数器 + 1。两者只在两种情形下分开：某条根的 txg 高过每一条读得出的记录；或者盘上已有计数器 ≠ txg 的记录（只会是前一种留下来的）。

草稿探针（都在 `drafts/mutant/` 里、不进补丁，22:15–22:53 在同步到主工作区现状之后跑）：

| 盘面 | 结果 |
|---|---|
| C366「怎么拦」点名的那一格：暖机 txg 1 的记录落盘、根没落 → 崩 → 可写挂载 | 今天挂得上（第 21 行说的「取号之前就被拒」随第 ④ 行落地已经没了）。所选根 txg 0、记录 jsn 1 不施加（D23 已定项 14 注 1：第 0 代根一条记录都不覆盖）；新实例的三次发布 txg/计数器 = 2/2、3/3、4/4，系统配置 tail 4。**相等** |
| B（txg 4）的记录落盘、根没落 → 崩 → 可写挂载 | 生效根 txg 4（施加了那条记录）；5/5、6/6、7/7，tail 7。**相等** |
| 随机历史：3 套权重 × 48 个种子 × 24 步 = 3600 步（可写或回退挂载 923 步），每一步之后读盘 | 44431 条记录全部计数器 = txg；根环最大 txg 从没高过记录最大 txg。`unequal=0` |
| 所选根（B，txg 4）自己那条记录两份都写成 0xA5 → 可写挂载 | 写行那次 txg 5、计数器 **4**；暖机 6/5、7/6。**唯一分开的一格** |

最后那一格就是 C500（所选根那条记录读不出）。C366 要的断言是「新写的记录计数器 = 前缀末 + 1」（D23 已定项 14 注 3，`.claude/kb/decisions/23-journal的角色与格式.md:362`「新实例从前缀末 + 1 接着写」），而这一格上「前缀末」是哪一条条款没写：
- 按所选根覆盖的那条（jsn 4，读不出）算 ⇒ 计数器 5 = txg，这一格也不再分开，挂载层就一格分得开的都没有了；
- 按最后一条读得出的（jsn 3）算 ⇒ 计数器 4，就是今天实现的做法。新记录用回 4 号、落在那条读不出的记录的环槽上。

注 1 在 2026-09-23 补的「锚点读不出时靠本次发布内序号接链首」管的是重放从哪接，没管计数器。这是条款空白，按定义第 6 条停下，不自己定、不写那条用例，也没在函数层再补一条。

**要主 agent 定的**：这一格的「前缀末」取哪一条。定成「读得出的最后一条」，C366 的用例就能写在这一格上：造盘面、可写挂载，断言计数器 = 4、tail = 计数器；把计数器写成 txg 就红。也可以写在这一格之后再干净重开一次的盘面上，计数器落后 txg 1 这件事会一直留着，那次挂载的前缀末没有歧义。定成「所选根覆盖的那条」，C366 在挂载层就没有可达的会红形态，只剩函数层那条（`second_transaction_supplement_two_warm_up_counter.rs`），第 21 行得改写去向。

推翻条件：找到一个不靠坏盘、挂载时 txg ≠ 计数器的可达盘面（比如回退、实例切换或一次发布拆成多条记录之后），上面「唯一一格」就不成立。并行线一把一次发布拆成多条记录之后要重看。

### 2. C378 的「记进失败账」怎么读

我照「认了」这么读：记在 `FaultInjectionTally` 里单开的一本账 `acquired_instances_left_after_a_failed_mount`，报告里点名，**不**进 `new_findings`、也不进已知红（认下来的行为不是失败）。要是主 agent 说的「失败账」指的是已知红清单（`crates/singlefs-harness/src/history.rs:1474` 的 `KNOWN_RED_FORMS`，故障注入把命中记进 `known_red_hits`），就得改成往那张清单里加一形，形状跟现在不一样。

## 六、`check.sh` 与门禁

开跑前 `ps` 看到的：别的会话的 `cargo test --all` 两个（pid 3437405、3446247）、若干 `cargo test -p singlefs-harness`、`cargo test --release --bin e142-first-txn-dry-run`；没有 `qemu-system`、`vm-bench.sh`、`e152`、`fio`。都加 `nice -n 19` 照常跑，没等锁。

**`check.sh`，同步到主工作区现状的副本原样跑**（`drafts/check-synced.log`）：红在格式，`Diff in` 全在 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 与 `e158_root_choice_repair.rs`。这两份在主工作区是别的会话没跟踪的文件（`git status` 显示 `??`），不在我的补丁里。末尾原样：

```
     let section_seven_one_anchor_holds = 4096 == 481 + 3615;
     all_ok &= section_seven_one_anchor_holds;
     emit_result(&format!(
  ✗ 格式不合规
     → 怎么办： 跑 cargo fmt --all 让它自己改完，再重跑本脚本。
exit 1
```

只把这两份格式化之后再跑，clippy 还是红在这两份上：e158 两处「`allow` 没写 reason」（:271、:631）、一处「test 模块之后还有 item」（:1138）、一处 `shadow_unrelated`（:1349）；e156 一处 `shadow_unrelated`（:1026）。**拿掉这两份**再跑（`drafts/check-variant/`，其余与副本逐字相同，`drafts/check-variant-synced2.log`）：格式 ✓、clippy ✓、构建 ✓、单测 ✓。末尾原样：

```

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit 0
```

**登记给实现员的门禁阶段**（`stage-owners.tsv` 列出 7 个：33、53、74、92、94、93、89），都在同步过的副本里跑；另跑派发点名的 54 号快档、59 号（只跑我的 7 行）。

| 阶段 | 退出码 | 原样判定行 |
|---|---|---|
| 33（只放我的 7 行，`SINGLEFS_GATE_FULL=1`） | 0 | `  ✓ 145 个实验二进制都有成形的变异表，1540 条变异的原文各命中源码一次；crates/mutations.tsv 7 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）` |
| 33（主工作区现在那张 330 行的表 + 我的 7 行） | 0 | `  ✓ 145 个实验二进制都有成形的变异表，1540 条变异的原文各命中源码一次；crates/mutations.tsv 337 条的原文各命中源码一次（…同上…）` |
| 59（只放我的 7 行） | 0 | 第三节原样那一段 |
| 53 | 0 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））` |
| 74 | 0 | `  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 5 段）：`（5 段逐段的明细在 `drafts/gate74.log`） |
| 92 | 0 | `  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 3b60f098e97dc4c4f3ed9c6355422b607db1c34c，77 个格式常量里变了 5 个（checker 在同一次改动里跟了 5 个，按滞后表放行 0 个），都不欠 checker 跟进` |
| 94 | 0 | `  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 3 份源码零处引 `singlefs_core`；共享模块 1 份源码的正文 231 行里没有分支与循环` |
| 93 | 0 | `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 1 位、登记表分出去 1 位；扫了 46 个 .rs，认出 2 处 feature bit 常量、解出位号 1 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:26，值 `INCOMPAT_FIRST_SSD_LINE_BIT` 不是位掩码字面量））` |
| 89 | 77（本次未跑，不是通过） | `  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）` |
| 54 快档 | 1 | `  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 8 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/impl-harness-batch/repo/.git/singlefs-layer0-full-green）` |

54 号红在「没有全绿标记」：快档本身绿了。主工作区的 `.git/` 下同样没有这个标记（`ls` 现查不存在），所以这一红跟我的改动无关；`crash.rs` 与两条流的用例改了，本来就要跑一次 `--full`。92、94 两道的「原样末行」是明细行，表里放的是它们的判定行，末行原样在 `drafts/gates-owned-summary.txt`。

## 七、要跟着改的 kb 行（我不改 kb，列给主 agent）

- `.claude/kb/milestone/02-second-txn.md:383` 收口表第 45 行：步 1 三条变异、步 6 按发布报状态数、步 3 取号之后没有屏障的变异这三样做了。「反向链那条等（要 I-8.6（反向链算法） 的 checker，第 26 行）」这一句不对了：今天那个变异红在池级 checker 的 I-8.6 上。
- 同文件 `:109`（步 1 验收的变异那一条）：「读回等于旧内容」照字面不成立，读者先撞上长度检查（见第四节）；「I-8.6 的 checker 步 6 才接」已经接上了。
- 同文件 `:234`（步 6 验收「每次发布各多少」）与步 6 的现状段：已经报了。全量那一串等 `--full` 核。
- 同文件 `:158`（步 3 验收的三条变异）：「取号之后没有屏障」有用例、有变异了。
- 同文件 `:385` 收口表第 47 行、`.claude/kb/checks-owed.md:425` C481：做了。
- 同文件 `:369` 收口表第 57 行、`.claude/kb/checks-owed.md:332` C378、`.claude/kb/decisions/23-journal的角色与格式.md:430`（已定项 16 的「欠」）：按「认了」断言的用例有了（第五节第 2 条的读法定了之后再划掉）。
- 同文件 `:352` 收口表第 21 行、`.claude/kb/checks-owed.md:322` C366：第 21 行说的「崩在暖机记录落盘、根没落时取号之前就被拒」今天不成立了（挂得上，两个量相等）；C366「怎么拦」写的那一格分不开两个量。等第五节第 1 条定了再改。
- `.claude/kb/layout/01-first-txn.md` 八：数一个都不用改（段序列、闭式、写数都没动）。新的屏障用例钉了第二条流段序列的前 22 段加末段 2，门禁 52 号不读它。
- 门禁 54 号脚本不用改：LAYER0 / LAYER0B 两行多一个 `states_by_publish=[…]` 字段，它认的行首与 `exhaustive=true` 都还在。下一次 `--full` 写的标记里会带上这个新字段。

## 八、补丁对主工作区现状的 `git apply --check`

22:55 UTC 在主工作区跑（主工作区 `crates/` 与副本相比只差我这 9 个文件，`diff -rq` 现查过，所以同步之后主工作区没再变过）：

```
Checking patch crates/singlefs-harness/src/bad_disk_input.rs...
Checking patch crates/singlefs-harness/src/crash.rs...
Checking patch crates/singlefs-harness/src/fault_injection.rs...
Checking patch crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_supplement_three_bad_disk_input.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs...
Checking patch crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs...
git apply --check exit 0
```

另外把主工作区的 `crates/` 拷到 `drafts/apply-probe/`，在那里 `git init` 之后 `git apply` 一遍，结果与副本 `repo/crates/` 逐文件相同。那个 `git init` 只落在草稿目录里，主仓的 git 没碰。
`sha256sum`：`harness.patch` 466e22de873f5570320814dcee1e2ffce9cfd9913f9940fdc9289fc0be3ac973，`mutations-append.tsv` 5c284045bed4033ef0d50a01de8a73f96d3345b0ba8c645d2df6fd24e106e78e。

主工作区在前一个实现员中断之后改过的几处（`mount.rs`、`model.rs`、`second_transaction_supplement_two_row_publish_admission.rs` 里「甲」加一撇的旧名改成「戊」，还有 `crates/mutations.tsv` 末尾多出来的行）都是注释或别人的行，不碰我补丁里的文件。接手之后我先把我那 9 个文件存进 `drafts/mine-2026-09-23T22/`，再把副本除这 9 个以外的部分（连 `.git` 和 `crates/` 之外的路径）同步成主工作区现状，然后重编、重跑上面这些。

## 九、没做什么

- 没走三方对抗；层 0 全量（门禁 54 号 `--full`）、QEMU、herd7、crates 整张变异表复跑归 `crash-verifier`；没提交、没暂存，主工作区一字未改。
- C366 的会红形态没做（第五节第 1 条）。
- 第二条流全量按发布分的那一串只拿闭式核过，没真枚举一遍；第一档基线「与加档之前逐字节相同」只是按构造成立，没拿报告前后比过；屏障用例与第二条流里 B 的文件内容是不是同一串字节，没比。
- 7 条变异没另在 `--release` 下跑（红的都是测试断言，不是 `debug_assert`）。
- `check.sh` 在副本原样跑是红的，红在别的会话没跟踪的 e156、e158 两个二进制上；绿的那一次拿掉了这两份。
- 门禁 69 号：我没往 `research/results/` 写任何东西（不在我的写范围）。这一轮的原始输出都在 `/tmp/claude-1000/impl-harness-batch/drafts/` 下（`rerun-batch.txt`、`rerun-barrier.txt`、`logs/`、`gate59-mine.log`、`gate54-quick.log`、`gate74.log`、`check-synced.log`、`check-variant-synced2.log`、`scratch-*.txt`、`gates-owned-summary.txt`）。补丁并进主仓的时候，要不要入库、入哪份由主 agent 定。
- `.claude/rules/fs-design.md` 只看了节标题：这一轮只动 harness 的测试与计数，没有格式或事务层的设计。
- 草稿用例 `scratch_c366_*.rs` 和 `scratch_full_states_by_publish_from_closed_form` 只留在 `drafts/mutant/` 里，不进补丁。
