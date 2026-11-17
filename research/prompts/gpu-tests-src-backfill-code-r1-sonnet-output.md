# 补轮 3 正推腿：产品 GPU 包的测试包 src 四份在途改动（代码轮第一轮）

日期：2026-11-17。腿：three-way-forward。草稿目录：`~/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-forward`。
方案按 `crates/` 今天的实现谈；没改仓内文件，没起项目代码（要量的都在「要主 agent 跑的命令」）。

## 0 开工核对

`sha256sum -c research/prompts/gpu-tests-src-backfill-code-r1-snapshot/crates-sha256.txt`（开工时与交回前各一次）：四份都 `OK`
（e202 bin、`gpu_side_units_entries.rs`、`reference_units_entries.rs`、`lib.rs`）。`kb-sha256.txt` 两份（登记与 E202 实验页）也 `OK`。
`crates/mutations.tsv` 里文件列指到这四份的行，与快照 `mutation-rows.tsv` `cmp` 逐字相同（123 行）。

没读禁读清单里的文件。

## 1 结论总览（细节在后面各节）

| 件 | 验收 | 一句话 |
|---|---|---|
| 1 取卡 | 对得上（带三条备注） | 只读服务交的卡、没有自己枚举的路、CPU 的活都在取卡之前；备注：`--cards` 张数由命令行写、装置不自核「只在交来的卡上」、领卡通道支路在本文件的 `all` 声明下服务不走 |
| 2 全部卡臂 | 对不上 | 登记原文「不给 `background_gpu_devices`」，代码点名服务交的卡；`--cards` 与 `name=every_card_arm_cards` 行登记里没有；另有一条条件性对不上（点名一张卡臂的卡可能不在全部卡臂开的 N 张里） |
| 3 开卡表 | 对得上 | 四处开卡都取同一份进程级静态，对 D26 已定项 13 第 4 条 |
| 4 接口改名 | 对得上，但不全是纯改名 | 名字都在核心层查得到；枚举多了第三个变体、特征少一个方法多两个转发 |
| 5 先录后判 | 对得上（带一条并行下的冲突） | 录、键、判、存放都经验证包那一份库；同一个测试二进制里「一次性库的用例」与「开全仓库的真卡用例」没有互斥守卫 |
| 7 入口登记表 | 对得上 | 39 个入口各恰好命中 1 处；`#[path]` 的前缀只定包，目标与模块路径由登记行定；单元图这样解 |

搬迁：123 行变异行的包名与测试目标都是新包的，原文逐行恰好命中一次；全仓旧路径活引用 0 处（不计 `research/`、`records/` 与说搬迁本身的 `path-moves.tsv` 一行）；
一条搬迁带来的回退：开跑自检的源文件摘要射程不再含装置自己（见 8.2）。

## 2 件 1：取卡只用服务交的卡

对着的规则（行号现查）：
- `.claude/rules/verification.md:50`（凡用 GPU 先清场，跑完不复原：清场与分卡由服务做，调用方不套清场、不挑卡）
- `.claude/rules/test-script-discipline.md:75`（用哪几台机器、哪几张卡、各给多少额度由服务分，调用方不写、不挑）、`:76`（用不上就退 78，不悄悄退回 CPU）
- `.claude/rules/verification-reference.md:75`（单个 GPU 测试二进制只用分到的卡，派活与判法拆开）

代码里做的（行号都是现版本，`grep -n` 取）：
1. 卡串来路只有两个：环境变量与领卡通道，没有第三条路。`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5708`：「cards_handed_over_or_leased_on_demand(environment_value, channel, lease)」；
   两样都没有走 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5710`：「.ok_or_else(|| REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER.to_string())」，上到命令里退 78（开头的来路核在 6074，退出码常量在 4127）。
2. 全文 `created_for_this_process` 0 命中（`grep -c`，在装置里；取卡模块里那一处建实例只在对上交来的卡之后）：`main` 不再现建 Vulkan 实例、不再枚举本机的卡。
3. 取卡在 CPU 的活之后：`lockstep` 造起步镜像在 6277、取卡在 6286；`concurrent` 在 7587 与 7596；`stage-three-cost` 造镜像加装置自数在 8738 到 8754、取卡在 8756。
   不要卡的 `geometry-check`、`construction-timing` 传 `CardsOfTheCommand::NotNeeded`（7820、7956），开头只打一行 `enumerated=false`（6085）。
4. 开卡只开点名的卡：每条 GPU 臂的 `background_gpu_devices` 是 `Some(…)` 的交来的卡（2846 起的 `background_compute_settings_of_the_arm`），CPU 臂用 `BackgroundComputeSettings::PRODUCT` 不开卡。
5. 没卡时的出路（没有悄悄只跑 CPU）：来路都没有退 78；交的卡里没有这一台的：`lockstep`（6359）、`concurrent`（7617）把每条臂都报 `name=not_run`（含 CPU 臂）后退 77（6371、7629）；
   交来的卡对不上本机独显：取卡模块 panic 被 `catch_unwind` 接住，交回 `Err`，退 78（5773 起的 `cards_taken_for_this_process`）。
6. 文件头 `// task-server-gpu-cards: all` 在第 4 行：服务读 `.rs` 源前 60 行（`task-server/task_server_lib.py:243`、`:2319` 的 `DECLARED_HEADER_LINES = 60`），`all` 的交法见 `task-server/README.md:195`；
   第 2 行 `task-server-gpu-arguments` 只列四个要卡的子命令，所以 `geometry-check` 等不拿卡。

验收：对得上。三条备注（都不是违例，是规则没说或说不死）：
- 备注 A（推不出）：`lockstep --cards <张数>` 的张数由命令行写；`.claude/rules/test-script-discipline.md:75` 说张数与额度由服务分、调用方不写。装置是在服务交来的卡里按命令行张数派活，卡的选择仍是服务交的先后；
  用户的弹窗定案在 `records/2026-11-11-GPU两段与按需领卡核查.md:412`。规则没写「在交来的卡里再收小」算不算调用方写张数，缺：主 agent 对这一句的读法。
- 备注 B（推不出）：装置开卡之后不核「这个进程只在交来的卡上」。取卡模块有这个核（`assert_this_process_is_on_no_gpu_but_the_handed_over_or_report_not_run`），
  装置里只有两条单测调它的纯函数（12938、12952），装置本体不调；靠点名开卡加服务的 `wrong-card` 提醒（`task-server/task_server_lib.py:2115`）。规则没要求装置自核，缺：要不要加。
- 备注 C（推的）：`cards_handed_to_the_apparatus` 的领卡通道支路，在本文件声明 `all` 时服务照旧起跑就整组分卡、不交通道（`task-server/task_server_lib.py:264`、`task-server/task_server.py:11613`）；
  只有命令外壳里写了只录一趟才交通道，而只录一趟装置本体不取卡。所以这条支路在装置本体上走不到（要主 agent 在服务上点名确认；推翻它的现象：任一趟 `lockstep` 的环境里没有 `GPU_CARDS_BY_PRIORITY` 而有领卡通道）。
- 备注 D（推不出，不是这一次改的）：`stage-three-cost` 在这一台没交卡时 GPU 臂报 `not_run`、CPU 臂照量、退 0（`arms_run=cpu` 那一行 `verdict`），而 `lockstep`、`concurrent` 退 77。
  `.claude/rules/test-script-discipline.md:32` 说有项本次未跑也是 0，对得上；同一份装置里三个命令两种退法，只记一笔。

推翻件 1 的现象：`created_for_this_process` 或自己枚举卡的调用重新出现在装置本体里；或要卡的子命令在 `cards_seen_by_the_command` 之前就有开卡的调用（`grep -n open_background_gpu` 的命中都在 4313、4327、8344、8446，调用方都在取卡之后）。

## 3 件 2：全部卡臂（对跑前登记）

登记原文现读（`research/prompts/e202-preregistration.md`，开工时 sha256 与快照一致）。「全部卡」那条臂的定义句，整行原样抄（第 49 行是读法写死里「卡」那一行，第 297 行是臂表里那一行）：

```
| 卡 | 「点名一张卡」：`background_gpu_devices` 只点本进程看得到的独立显卡里 PCI 地址最小的那一张；「全部卡」：不给 `background_gpu_devices`（用本进程看得到的全部独立显卡，按 PCI 地址次序填满）。两种都给每张参与的卡 `background_gpu_memory_limit` = 1 GiB。装置不点卡号，看得到哪几张由服务定 |
| GPU 全部卡臂 | `gpu`，不给 `background_gpu_devices`，每张卡上限 1 GiB | 同上，参与的卡是本进程看得到的全部独显，按 PCI 地址次序填满 | 同上；活装得进头一张卡时（第四节推算），分到活的卡与点名一张卡臂相同，差只在多几张参与的卡 |
```

代码（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs` 2846 起）：全部卡臂的 `background_gpu_devices` 是 `Some(` 服务交来的卡逐张的标识 `)`，上限一份一张；
点名一张卡臂在 2839 `.take(1)` 取 `cards.discrete_gpus` 的头一张。

逐句比：
- 登记 49、297 说「不给 `background_gpu_devices`」：代码给了，点名的是服务交来的卡 → **对不上**。
  不给的话，产品按「本机全部独立显卡」开（`.claude/kb/decisions/26-后台整理与放置回收.md:251`），会开到服务没交的卡，与件 1 冲突；所以代码的改法是规则逼出来的，但登记原文没跟着改。
- 登记 49「装置不点卡号，看得到哪几张由服务定」：代码点的卡号来自服务交的卡串，哪几张仍是服务定的 → 实质对得上，字面上装置点了名。
- 登记 78（`research/prompts/e202-preregistration.md:78`）只看得到一张时全部卡臂报本次未跑：`reason_the_arm_is_not_run`（2808 起，2817 `gpu_count == 1` 那一支）对得上；`--cards` 给 1 也被挡成命令行不对（`CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST`）。
- 登记没有 `--cards`、没有 `name=every_card_arm_cards` 一行、没有「续跑时核张数」：`grep -c cards research/prompts/e202-preregistration.md` 是 0 → 登记外的新旋钮与新行。
- 修订记录缺：`.claude/rules/evidence-discipline.md:114` 说跑产物之前修订臂或判据合法，但要在跑前登记里留记录；
  登记第十二节没有这一条（`grep -c cards` 0 命中）。`records/2026-11-11-GPU两段与按需领卡核查.md:463` 写明登记原文当冻结证据没改；用户定案在同一份记录的 334 行（全部卡臂开设定交来的全部卡）与 412 行（`--cards` 做成可设）。
  E202 实验页已写 `--cards`（`.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:15`）。所以现状是：用户定了、实验页改了、登记没同步、产物一次没跑——补一条第十二节修订就闭合；这一条补不补由主 agent 定。
- 条件性对不上（推的，没复跑）：点名一张卡臂取 `cards.discrete_gpus`（全部交来的，按 PCI 地址排）的头一张（2839）；全部卡臂在给了 `--cards N`（N 小于交来张数）时取「交来的卡串先后里的前 N 张」
  （5902、5906：`let first_handed_over: &[String] = &handed_over_order[..requested_card_count.get()];` 再按 PCI 排）。服务交来的先后不是 PCI 升序时（例：先后 03、02、01，`--cards 2` → 全部卡臂开 02、03，点名一张卡臂点 01），
  两条臂的头一张卡不是同一张，登记 297 的「分到活的卡与点名一张卡臂相同，差只在多几张参与的卡」与甲类锚点（登记第 413 行附近）就比不成。
  新单测 `the_cards_option_names_the_first_handed_over_cards_in_the_handed_over_order` 的先后是 03、01、02，`--cards 2` 取 03、01，恰好含最低的 01，没覆盖反例。
  推翻它：服务交来的卡串永远按 PCI 升序（查 `GPU_CARDS_BY_PRIORITY` 的排法，`multi-host.env.example` 第 89 行那种写法按序号排，序号与总线次序是否一致本腿没查）。
- 测试覆盖：件 2 的纯函数（`discrete_gpus_named_for_the_every_card_arm`、`fields_of_the_every_card_arm_cards_line`、`every_card_arm_cards_differing_from_the_product_being_resumed`、`the_every_card_arm_cards_are_recorded`、`cards_of_the_every_card_arm_of_the_options`）都有单测与变异行（第 9 节列行号）；
  `lockstep_command` 里把它们接起来的那一段（6286 到 6340）没有单测、没有变异行。

验收：对不上（登记 49、297 那一句；缺第十二节修订）；另有一条条件性对不上。

推翻件 2「对不上」的现象：登记第十二节或别处已有一条写明「全部卡臂改点名服务交来的卡、加 `--cards`」的修订（本腿 `grep -n cards` 在登记里 0 命中）。

## 4 件 3：开卡表整个进程一份

对着：`.claude/kb/decisions/26-后台整理与放置回收.md:253`（已定项 13 第 4 条：记着没回来的开卡的那张表在进程级，各次挂载与切换开卡共用；前一次挂住的卡之后同样不再起）。
- 静态 `OPENINGS_NOT_RETURNED_IN_THIS_PROCESS`（4285）是 `OnceLock`，`openings_not_returned_in_this_process()`（4290）用 `get_or_init(OpeningsNotReturnedInThisProcess::with_none_not_returned)`（4292）。
- 全文调用产品开卡入口的四处都传这一张：4313（`open_background_gpu_compute`）、4327、8344、8446（后三处 `open_background_gpu_with_the_memory_leaked_in_this_process`）；
  全文 `with_none_not_returned` 只在 4292 出现一次（`grep -nF`）。
- 产品一侧：`crates/singlefs-background-gpu/src/background_gpu.rs:1315` 的 `OpeningsNotReturnedInThisProcess` 是 `Arc<(Mutex<…>, Condvar)>` 的 `Clone`，开卡后挂载与切换拿的是同一张表的把手（`is_the_same_table_as` 1371 比 `Arc::ptr_eq`）。
  装置不经产品的进程开卡器（`gpu_opener_of_this_process`，装置里 0 命中），所以这个进程里只有这一张。
- 多线程：`OnceLock` 初始化互斥；表里是 `Mutex` 加 `Condvar`。装置里开卡都在驱动线程上，没有两处同时建表的路。
- 单测两条：`every_opening_in_the_apparatus_is_handed_the_same_openings_table_of_this_process`（13186，`is_the_same_table_as`）；
  `the_apparatus_builds_an_empty_openings_table_only_for_the_one_kept_for_this_process`（13196 起，读自己源文件数 `with_none` 加 `_not_returned` 拼起来的那一串恰好 1 次）。
  变异行 122（V88，进程那一张表每次问都现建）、123（V89，丙段份额那一处开卡退回现建）。后一条是读自己源文本的断言，只管「再有人现建」，不管「换别的构造函数」；该类型只有这一个公开构造函数（`background_gpu.rs:1356`），没有 `Default`，所以这个缺口今天关得上。

验收：对得上。推翻它：装置里出现第二处 `with_none_not_returned` 或 `OpeningsNotReturnedInThisProcess` 的别的构造。

## 5 件 4：跟核心层接口改名

diff 里含改名记号的行 67 行（`grep -cE` 数 `ForegroundFsync`、`AnswerOfTheCallerBeforeTheNextBatch`、`ONE_COW`、`ONE_COPY_ON_WRITE` 等），都落在这几块：`use` 行；两处文档注释；`CallerOfTheLockstepRun`（3360）与并发跑调用方（6615）的 `answer_before_the_next_batch`；
并发跑的几处判停与计数过滤（6642、6649、6680 附近）；`LINEAGE_…` 的一处 `use`、一处注释、一处用（78、348、370）；`mod tests` 里的 `(Duration, 回答)` 元组换名与 rustfmt 重排。没有逻辑改动。

改名对到的核心层接口，`crates/singlefs-core/src/` 里现查：
- 枚举 `AnswerOfTheCallerBeforeTheNextBatch`：`crates/singlefs-core/src/compaction_plan.rs:1541`；变体 `NoForegroundFsyncWaiting`（1542）、`ForegroundFsyncIsWaiting`（1543）。旧名 `ForegroundFsync`、方法 `foreground_fsync`：`grep -rnE` 在 `crates/singlefs-core/src` 与 `crates/singlefs-background-gpu/src` 0 命中。
- 特征方法 `answer_before_the_next_batch`：`crates/singlefs-core/src/compaction_plan.rs:1552`。
- 常量 `LINEAGE_CONTAINER_BLOCKS_OF_ONE_COPY_ON_WRITE`：`admission.rs:175`；旧名 `_ONE_COW` 0 命中。
- 两个新转发要的特征方法：`use_no_gpu_from_the_next_batch`（`background_compute.rs:640`，有缺省）、`put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`（`background_compute.rs:646`，有缺省）；
  `LatestGpuSettingsHandedIn`（`background_compute.rs:673`）。装置两个包装（`GpuComputeRejudgedOnTheCpu`、`GpuComputeTimingTheStageThree`）在 3065、3069 与 8277、8281 显式转给里面那一份。
- `AllocationRecordQuarantineMark`：`allocator.rs:51`，变体与装置用的两个同名（先录后判新用，不是改名）。
- 旧特征方法 `devices_not_opened_at_the_mount` 与类型 `GpuDeviceNotOpenedAtTheMount`：整个 `crates/` 0 命中（核心层已删，装置跟着删）。

不是纯改名的两处（都不改装置行为，推的，没编）：
1. 枚举比旧的多一个变体 `CallerIsStoppingItsThread`（`crates/singlefs-core/src/compaction_plan.rs:1545`）；装置的调用方只答前两种，判停与计数只比 `ForegroundFsyncIsWaiting`（6642、6649），装置里没有对这个枚举的穷举 `match`（`grep -n 'match .*answer'` 只有 6645 的 `match waiting_answers`，那是对个数 `match`）。
2. 特征少了一个方法、多了两个（有缺省实现）；装置转发是对的，但「纯改名」的说法对这一块不成立。

验收：对得上（名字都对得到核心层）；不是纯改名，差在上面两处。要主 agent 跑 `cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison` 才算编过（我没编）。
推翻它：编译报「非穷举 match」或「缺特征方法」。

## 6 件 5：内联真卡测试先录后判、经判定库

读 `crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs` 与装置 8491 到 8712、12127 到 12148。

录什么：
- 输入字节（`pairing_input_bytes_of_the_allocation_records_and_the_unit_area`，e202 8516）：小池起步镜像那一版的分配记录与盘 0 单元区。布局：盘号、起点槽、槽数，记录条数，逐条 盘号、槽号、跨几槽、代、释放没有、隔离没有；全是 8 字节小端（`append_number`、`append_count`，公共模块 369、374）。
- 参照字节（`reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment`，8619）：把输入读回（8572 `allocation_records_and_the_unit_area_of_the_pairing_input_bytes`），调装置自数 `live_slots_of_each_segment_counted_by_the_apparatus`，写成「段数、逐段活槽数」。装置自数不调核心层。
- 两样按内容摘要写进库（验证包 `crash_library.rs:2216` 起的 `pairing_input_and_reference_through_the_library_under_the_recording_key` 里 `store_pairing_input` 各一次），登一条清单；只录那一趟打 `RECORDED_ONLY` 就返回，不取卡。

键是什么：
- 录入键 = 录入函数单元那一把（工具链）、输入字节、参照一侧的判法单元键三样合起来（`recording_key_of_the_pairing_input_and_the_reference_units`，公共模块 313 起；参照一侧的键取自构建脚本写的 `reference_units_keys.tsv` 里组 `apparatus_count_of_the_live_slots_per_segment_in_the_gpu_cpu_gc_comparison`，装置 8504 的常量与登记表组名逐字相同）。
- 对拍结论键 = 判段读回的输入摘要、参照一侧的键、GPU 一侧的键（组名 = 用例名 `the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`，在 `gpu_side_units_entries.rs:164`，装置 12135 传的用例名逐字相同）。库里有两边一致的就复用、不领卡不开卡；没有或记着红的才现判，红的照同一把键存。

判在哪：`judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`（8666）：
把读回的输入与参照解开，查服务有没有交卡（8675，没有打 `NOT_RUN_NO_GPU_ADAPTER` 交回没跑成、不入库），取卡（8681，取不成 panic），跑 GPU 点名一张卡臂 5 遍（8696 的 `assert_eq!` 钉次数），
逐遍与读回的装置自数比（8702 的 `panic!`）。整段包在 `judgement_of_the_assertions_on_the_cards`（公共模块 147）里：全过存「两边一致」，有 panic 存「两边不一致」并让用例红，没跑成用 `panic_any` 记号让上层不入库。

存在哪：`CrashLibrary::open(CrashLibraryClass::InputPairing)`（公共模块 310），即验证包的 `VerdictStore::open_the_prune_server`（`crash_library.rs:631` 那一行），走 prune-server 那一份 RocksDB。
对着 `.claude/rules/verification.md:54`、`:56`：没有新开目录、TSV、sqlite 或另一份 RocksDB。单测用的是 `DisposableLibrary`（12869，验证包自己起的临时服务，12871 设成这个进程的库），符合 `:56` 说的自证对临时服务。

验收：对得上（录、键、判、存放都是验证包那一份；装置里没有自己的键算法）。

一条并行下的冲突（推的，没复跑）：
- 同一个测试二进制里有两类用例：`the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count`（12869、12871 把一次性库设成这个进程的库）与带 `tests-on-a-real-gpu` 才编的 `the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`（12132，经公共模块 310 `CrashLibrary::open` 开全仓那一份）。
- 验证包 `crates/singlefs-verification/src/crash_library.rs:691` 起的文档写：开全仓库的用例要在开库之前拿 `TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out()`（701），不拿的，同一个二进制里设着一次性库的用例会把它带到临时库、临时服务停了就连不上（退 75）。
  `open_or_share`（1111）读的正是进程级的 `DISPOSABLE_LIBRARY_OF_THIS_PROCESS`。装置与公共模块 310 的开库前都没拿这个守卫（`grep -rn keep_disposable_libraries_out crates` 只在验证包里命中）。
- 其它带真卡对拍的测试目标里没有设一次性库的用例，所以这个冲突是 e202 这一个二进制独有的；门禁的 `product-gpu-real-card-tests` 格带特性跑整包 `--tests`，会把装置的内联单测与那条真卡用例放进同一个进程并行跑（`.claude/gate.d/crash-point-replay.sh:3031` 那段说 `--tests` 连 `src/bin/` 的内联单测一起带上，指验证包那一趟；测试包那一趟是否同样没读）。
- 推翻它：公共模块或验证包在开库前已拿守卫（本腿读了 310 前后与 `open_or_share`，没有），或门禁把这两条分进程跑（`--exact` 点名一条）。要主 agent 复跑：命令 6。
- 另：`a_greedy_call_on_a_real_card_runs_the_stage_three_on_the_gpu_within_the_call_on_the_small_pool`（12155）不入对拍结论、每趟判，理由写在 12150 起的注释（CPU 臂那一截包外量不到）；`.claude/rules/test-script-discipline.md:77`（加速路径留一份 CPU 参照、同输入逐项相同）对它不适用——推不出，缺：主 agent 认不认这个理由。

## 7 件 7：两张入口登记表

### 7.1 表里每个入口在源码里逐个找

做法（只读脚本，草稿目录 `tools/entries_check.py`，`python3 -I` 跑；它读两张表、对每个入口取 `::` 最后一段当 fn 名，在包内 `tests/` 与 `src/` 全部 `.rs` 里数 `fn <名>`；另一份 `tools/entry_depth.py` 核每个 fn 在目标文件顶层、或在目标根文件 `mod` 声明过的模块里）：
两份脚本的结果：39 行、0 行不对（`rows 39 bad 0`；`rows 39 problems 0`）。参照一侧 8 个入口（7 组，其中整理那一趟两个入口）、GPU 一侧 31 个，GPU 一侧每组的组名恰好等于入口 fn 去掉 `judged_on_the_cards_that_` 的后半（31 组 0 个对不上）。
逐个命中（表：参照 / GPU；目标；fn 名截 58 字；命中处数；在哪个文件）：

```
参照	test:checksum_kernels_on_a_real_gpu_agree_with	reference_bytes_of_the_checksum_pairing_input	命中1处	[tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs]
参照	test:live_slot_kernels_on_a_real_gpu_agree_wit	reference_bytes_of_the_live_slot_pairing_input	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
参照	test:region_record_kernels_on_a_real_gpu_agree	reference_bytes_of_the_region_pairing_input	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
参照	test:compaction_on_a_real_gpu_through_the_back	reference_bytes_of_the_compaction_run_on_the_cpu	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
参照	test:compaction_on_a_real_gpu_through_the_back	reference_bytes_of_the_units_narrowed_to_one_card	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
参照	test:compaction_on_a_real_gpu_through_the_back	reference_bytes_of_the_refusal_of_the_card_not_on_this_...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
参照	test:mount_runtime_on_a_real_gpu_writes_the_sa	reference_bytes_of_the_runtime_on_the_cpu	命中1处	[tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs]
参照	bin:e202_gpu_cpu_gc_comparison	reference_bytes_of_the_apparatus_count_of_the_live_slot...	命中1处	[src/bin/e202_gpu_cpu_gc_comparison.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_table_driven_kernel_matche...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_bitwise_kernel_matches_the...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_two_kernels_confirm_every_...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_batch_on_a_card_with_less_...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_the_batch_over_one_named_real_...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_kernels_on_a_real_gpu_agree_with	judged_on_the_cards_that_opening_the_handed_over_gpus_a...	命中1处	[tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_batches_over_two_real_gpus_give_	judged_on_the_cards_that_the_batch_over_two_named_real_...	命中1处	[tests/checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_batches_over_two_real_gpus_give_	judged_on_the_cards_that_the_batch_the_first_named_real...	命中1处	[tests/checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs]
GPU 	test:checksum_batches_over_three_real_gpus_giv	judged_on_the_cards_that_the_batch_over_three_named_rea...	命中1处	[tests/checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs]
GPU 	test:live_slot_kernels_on_a_real_gpu_agree_wit	judged_on_the_cards_that_the_kernel_walking_every_recor...	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:live_slot_kernels_on_a_real_gpu_agree_wit	judged_on_the_cards_that_the_kernel_searching_the_sorte...	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:live_slot_kernels_on_a_real_gpu_agree_wit	judged_on_the_cards_that_the_two_live_slot_kernels_conf...	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:live_slot_kernels_on_a_real_gpu_agree_wit	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:live_slot_kernels_on_a_real_gpu_agree_wit	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:live_slot_kernels_on_a_real_gpu_agree_wit	judged_on_the_cards_that_the_memory_limit_below_the_sma...	命中1处	[tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:region_record_kernels_on_a_real_gpu_agree	judged_on_the_cards_that_the_walking_kernel_matches_the...	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:region_record_kernels_on_a_real_gpu_agree	judged_on_the_cards_that_the_searching_kernel_matches_t...	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:region_record_kernels_on_a_real_gpu_agree	judged_on_the_cards_that_the_two_region_record_kernels_...	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:region_record_kernels_on_a_real_gpu_agree	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:region_record_kernels_on_a_real_gpu_agree	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:region_record_kernels_on_a_real_gpu_agree	judged_on_the_cards_that_the_memory_limit_smaller_than_...	命中1处	[tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs]
GPU 	test:compaction_on_a_real_gpu_through_the_back	judged_on_the_cards_that_the_compaction_run_on_the_real...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
GPU 	test:compaction_on_a_real_gpu_through_the_back	judged_on_the_cards_that_the_implementation_answers_the...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
GPU 	test:compaction_on_a_real_gpu_through_the_back	judged_on_the_cards_that_settings_naming_one_card_narro...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
GPU 	test:compaction_on_a_real_gpu_through_the_back	judged_on_the_cards_that_the_run_whose_every_wait_on_th...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
GPU 	test:compaction_on_a_real_gpu_through_the_back	judged_on_the_cards_that_the_compaction_stopped_on_the_...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
GPU 	test:compaction_on_a_real_gpu_through_the_back	judged_on_the_cards_that_the_compaction_stopped_on_the_...	命中1处	[tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs]
GPU 	test:mount_runtime_on_a_real_gpu_writes_the_sa	judged_on_the_cards_that_the_runtime_on_the_real_gpus_w...	命中1处	[tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs]
GPU 	bin:e202_gpu_cpu_gc_comparison	judged_on_the_cards_that_the_gpu_arm_on_a_real_card_cou...	命中1处	[src/bin/e202_gpu_cpu_gc_comparison.rs]
合计 39
```

### 7.2 构建脚本怎么读这两张表

- `crates/singlefs-background-gpu-tests/build.rs:8`、`:10`：`#[path]` 把两份登记文件编进构建脚本（它们「只用 `std`」）。两张表用同一个行类型 `PairingSideUnitsEntryGroup`（`reference_units_entries.rs` 里定义，GPU 一侧文件 `use crate::reference_units_entries::…`）。
- `crates/singlefs-background-gpu-tests/build.rs:72` `keys_of_the_entry_groups`：每组造一个 `EntryGroupRequest`：`target` 由 `TargetSelector::parse(group.target)`（80；`test:<名>` 与 `bin:<名>` 在 `crates/singlefs-unit-graph/src/workspace.rs:128`、`:129` 解成 `Binary(名)`、`IntegrationTest(名)`）、`entry_paths` 照登记、`features` 恒为 `tests-on-a-real-gpu`（89）。
- 逐组调 `entry_group_key_with_the_features_of_the_build`，再把表写进 `OUT_DIR/reference_units_keys.tsv`、`OUT_DIR/gpu_side_units_keys.tsv`（185、189、194）；测试经 `include_str!(concat!(env!("OUT_DIR"), …))` 读（公共模块 36、40）。
  登记错的入口会让 `build.rs` panic（组名带在 panic 里），不是静默算出一把别的键。

### 7.3 `singlefs_background_gpu_tests::` 前缀在函数住在测试目标与 bin 里时怎么解

读 `crates/singlefs-unit-graph/src/closure.rs`：
- `resolve_entry_target`（2918）：入口路径按 `::` 切开，**第一段只用来找包**：`workspace.crate_named`（`workspace.rs:1715`，包名或把 `-` 换成 `_` 的名字相同就认），所以 `singlefs_background_gpu_tests` 找到 `singlefs-background-gpu-tests`；编译目标不从路径取，取请求里的 `target`（`TargetSelector::Binary(名)` 或 `IntegrationTest(名)`），找不到交 `TargetNotFound`。
- `resolve_entry_units`（3012）：后面的段（去掉第一段）当模块路径加 fn 名，只在请求的目标自己的文件里找（`files_of_the_target`），`matches_entry_path`（2973）比的是目标里的模块路径：目标根文件里的 fn 模块路径为空，`mod common_…;` 里的 fn 模块路径是那个模块名。
  找不到交 `EntryNotFound`，在这组特性下不编进来交 `EntryInactive`（`entry_groups.rs:202` 去掉这一个入口、键照其余的算），命中几个不唯一交 `EntryAmbiguous`。
- 所以：库 `lib.rs` 是空的不碍事（它不是被收的目标）；前缀只是包名；入口 fn 在 `tests/<目标>.rs` 顶层、或目标根文件 `mod` 过的模块里、或 `src/bin/<名>.rs` 顶层，就解得到。39 个入口都满足（7.1 的 `entry_depth` 结果）。
- bin 那两组（参照 `reference_units_entries.rs:64`、GPU 一侧 `gpu_side_units_entries.rs:164`）的入口都在装置根文件上，且带的是 `cfg_attr(…, allow(dead_code))` 而不是 `cfg`，在非测试编法下也是活的（装置 8612 与 8660 起的 `cfg_attr`），闭包不会因 `cfg(test)` 收不到。
  装置经 `#[path = "../../tests/…/mod.rs"] mod …;` 引两个公共模块（154、157 的 `#[path]`），单元图认 `#[path]`（`crates/singlefs-unit-graph/src/units.rs:395` 的 `path_attribute`）。
- 构建能否通过、键是否都 `keyed`：本腿没编。`crates/singlefs-background-gpu-tests/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs:340` 的 `every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate` 对 GPU 一侧 31 组判「算出键、至少一个产品 WGSL、键两两不同」；
  对 bin 那一组它只核到「至少一个产品核」（`kernel_files_every_gpu_side_group_of_the_target_reaches` 对 `bin:` 目标落到 `_ => &[]`），没核它走到丙段那两份核。参照一侧 8 个入口没有测试逐组核（那个测试文件把参照表 `allow(dead_code)` 编进来不用），只靠 `build.rs` 的 panic 保证入口找得到。

验收：对得上（入口逐个找得到、前缀的解法成立）。两条缺口：参照一侧的表没有「入口指对了函数」的测试与变异行（指到别的存在的 fn，构建照过）；bin 那组的 GPU 一侧闭包是否收到丙段核没有判别。
推翻它：构建时 `build.rs` panic「登记表里组 … 的入口写错了」或 `pairing_conclusion…` 里 `every_gpu_side_group_is_keyed…` 红。命令 2。

## 8 搬迁本身（第 2 条）

### 8.1 123 行变异行

命令与输出（`crates/mutations.tsv` 的 sha256 `2c1b068f…`；第二列是四份之一的行）：

```
$ awk -F'\t' '$2 ~ /^crates\/singlefs-background-gpu-tests\/src\// ' crates/mutations.tsv | wc -l
123
$ awk -F'\t' '{print $2}' <上面的行> | sort | uniq -c
    121 crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs
      2 crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs
$ awk -F'\t' '{print $5}' <上面的行> | sed 's/ -- .*//' | sort | uniq -c
    121 -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison
      2 -p singlefs-background-gpu-tests --test pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes
$ git grep -nF 'singlefs-background-gpu --bin e202' -- crates/mutations.tsv | wc -l
0
```

逐行数（`tools/mutation_rows_check.py`，只读；对每行：原文把 `\n` 还原成换行后在「文件」里 `str.count`；第五段的包名、`--bin`/`--test` 目标文件存在；筛选词与第六段同名、包内 `fn <词>` 恰好一处且紧挨 `#[test]`）：
末行 `rows_total 123 anchor_exactly_once 123 anchor_not_once 0`；`OK` 行 123 行，非 `OK` 行 0 行。下面 8.3 逐行列出（行号是这 123 行里的次序，不是 `crates/mutations.tsv` 的行号；变异名取自快照）。

结论：123 行的 cargo 参数包名都是 `singlefs-background-gpu-tests`、目标都在新包里存在、原文都恰好命中一次、点名的测试都筛得到（第 5 个检查单格的「搬迁之后」一项成立）。
没核：第五段能不能真把被改坏的函数跑红（`static-check` 的 `UNREACHABLE` 一类，命令 3）；两条 `--exact` 行（91、92 行）是 GPU 一侧表那两行，被点的测试在 `pairing_conclusion…:340`，筛得到。

### 8.2 旧路径的活引用

```
$ git grep -nF 'crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison' | wc -l
67
$ git -c core.quotepath=false grep -nF '<同上>' -- . ':(exclude)research' | wc -l
15
```
排除 `research/` 之后 15 行：`.claude/gate.d/path-moves.tsv:673`（搬迁登记本身，说搬迁的行，不算活引用）；`records/` 下 14 行（8 份带日期的过程记录：`里程碑四预开发` 2 行、`里程碑三收尾与近期欠账` 1、`里程碑四GPU垃圾回收开工` 1、`一百七十六节欠账` 3、`文档代码逐项审核-台账.json` 2、`删轻型并入重型` 1、`GPU两段与按需领卡核查` 3、`background-gpu审核四条修复` 1），都是当时的记录，不是活引用。
再查两种别的写法：`git grep -nE 'singlefs-background-gpu +--bin e202|-p singlefs-background-gpu --bin e202'`（排除 `research/`）0 行；`git grep -nE 'e202_gpu_cpu_gc_comparison'`（排除 `research/`、`records/`、`crates/mutations.tsv`、装置自己）命中的全是新路径或登记行：
`comment-history-lag.tsv:11`、`.claude/gate.d/experiment-number-directories.tsv:17`、`.claude/gate.d/path-moves.tsv:673`（旧到新）、三语 E202 实验页（新路径）、`Cargo.toml`（新包）、两张登记表。`.claude/crates-review-exempt` 里没有指到这四份的行（`grep -n 'e202\|background-gpu-tests'` 0 命中）。
活引用 0 处。登记 `research/prompts/e202-preregistration.md` 的起法仍写旧路径与旧包名（第 579 到 582 行附近），它在 `research/` 内，按派发排除；记一笔：登记起法与现装置不一致，是件 2 那条修订该一起改的地方。

搬迁带来的一处回退（推的，没复跑）：开跑自检算「crates 版本」摘要的目录写死两处：`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:4135` 与 4136（同一份文件）：
`PathBuf::from("crates/singlefs-background-gpu/src"),`、`PathBuf::from("crates/singlefs-core/src"),`（见装置 `source_files_and_digests`，4132 起）。装置在 HEAD 里住在产品 GPU 包的 `src/bin/` 下，所以摘要罩得到装置自己；搬到测试包之后摘要射程只剩产品 GPU 包与核心层，
不含装置自己、测试包的公共模块与验证包。登记停机第 2 条（`research/prompts/e202-preregistration.md:436`）说「同一段的几条命令之间 `crates/` 中途被改」就停，装置自己或公共模块在两条命令之间被改，这一条不会响。
推翻它：登记或实验页说明摘要射程只管被测的两个包（本腿读到的第五节只有「`crates/` 中途被改」一句，没有逐目录的定义）。

### 8.3 逐行（行号 判定 原文命中数 点名的测试）

```
1 OK anchor_hits=1 process_cpu_of_the_window_subtracts_the_foreground_and_main_threads_but_keeps_the_driver_threads
2 OK anchor_hits=1 the_saving_is_the_cpu_arm_minus_the_gpu_arm
3 OK anchor_hits=1 the_rejudging_wrapper_rejudges_every_confirmed_checksum_on_the_cpu
4 OK anchor_hits=1 the_rejudging_wrapper_rejudges_every_confirmed_live_slot_count_and_checks_every_full_segment_was_handed_in
5 OK anchor_hits=1 the_cpu_rejudging_inside_the_window_counts_toward_the_process_cpu_of_the_call
6 OK anchor_hits=1 the_recording_digest_tells_apart_two_writes_differing_in_one_byte
7 OK anchor_hits=1 thirty_percent_between_the_end_tiers_is_not_flat
8 OK anchor_hits=1 local_constants_agree_with_the_core
9 OK anchor_hits=1 publishes_of_one_intent_at_the_main_sampling_point_are_five
10 OK anchor_hits=1 the_gc_wall_clock_counts_the_calls_that_published_no_batch
11 OK anchor_hits=1 the_every_card_arm_names_every_handed_over_card_and_gives_one_limit_per_card
12 OK anchor_hits=1 the_fake_clock_of_the_lockstep_run_advances_one_millisecond_per_call
13 OK anchor_hits=1 the_batch_giving_work_to_a_second_card_breaks_the_anchor
14 OK anchor_hits=1 positive_control_fails_when_the_rejudging_adds_nothing_or_far_more_than_the_inner_timing
15 OK anchor_hits=1 the_path_anchor_holds_with_more_records_than_batches_and_breaks_on_a_wrong_path_or_fewer_records
16 OK anchor_hits=1 the_path_anchor_holds_with_more_records_than_batches_and_breaks_on_a_wrong_path_or_fewer_records
17 OK anchor_hits=1 the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same
18 OK anchor_hits=1 the_make_filesystem_refuses_a_device_one_sector_short_of_four_rings
19 OK anchor_hits=1 the_make_filesystem_refuses_a_device_one_sector_short_of_four_rings
20 OK anchor_hits=1 batches_equal_and_at_least_sixty_four_are_enough_and_nothing_else_is
21 OK anchor_hits=1 the_repeat_checks_void_a_busy_host_first_and_stop_on_a_broken_class_one_anchor
22 OK anchor_hits=1 the_repeat_checks_void_a_busy_host_first_and_stop_on_a_broken_class_one_anchor
23 OK anchor_hits=1 the_class_one_anchors_of_a_lockstep_run_break_past_twenty_publishes_between_two_user_changes
24 OK anchor_hits=1 the_decisions_of_a_rejudged_batch_must_add_up_to_the_entries_handed_in
25 OK anchor_hits=1 the_cpu_arm_thread_cpu_covers_the_wall_at_ninety_percent_and_not_below
26 OK anchor_hits=1 the_inclusion_checks_count_a_call_whose_thread_cpu_exceeds_its_wall_or_its_process_cpu
27 OK anchor_hits=1 the_gpu_work_anchors_count_a_batch_abandoned_after_a_compute_error_and_the_third_void_stops_the_tier
28 OK anchor_hits=1 the_gpu_work_anchors_count_a_batch_abandoned_after_a_compute_error_and_the_third_void_stops_the_tier
29 OK anchor_hits=1 the_comm_line_of_the_gpu_opening_thread_matches_on_its_first_fifteen_bytes
30 OK anchor_hits=1 the_crates_change_is_seen_when_an_earlier_command_ran_on_another_digest
31 OK anchor_hits=1 the_thread_cpu_clock_check_needs_a_hundred_advances_in_a_millisecond
32 OK anchor_hits=1 the_latency_counts_from_the_planned_time_and_the_publish_under_the_lock_from_the_lock
33 OK anchor_hits=1 the_ninety_ninth_percentile_takes_the_nearest_rank
34 OK anchor_hits=1 the_yield_anchor_holds_only_when_the_waiting_answer_ends_the_call_with_the_yield_stop
35 OK anchor_hits=1 the_yield_anchor_holds_only_when_the_waiting_answer_ends_the_call_with_the_yield_stop
36 OK anchor_hits=1 the_yield_anchor_holds_only_when_the_waiting_answer_ends_the_call_with_the_yield_stop
37 OK anchor_hits=1 the_longest_batch_and_tail_come_from_the_gaps_between_the_foreground_queries
38 OK anchor_hits=1 the_longest_batch_and_tail_come_from_the_gaps_between_the_foreground_queries
39 OK anchor_hits=1 the_longest_batch_and_tail_come_from_the_gaps_between_the_foreground_queries
40 OK anchor_hits=1 the_latency_counts_from_the_planned_time_and_the_publish_under_the_lock_from_the_lock
41 OK anchor_hits=1 the_class_one_anchors_of_a_concurrent_run_break_on_a_call_not_yielding_or_a_wait_past_the_bound
42 OK anchor_hits=1 the_class_one_anchors_of_a_concurrent_run_break_on_a_call_not_yielding_or_a_wait_past_the_bound
43 OK anchor_hits=1 the_concurrent_tier_judgement_pools_the_runs_and_pairs_the_gpu_arm_against_the_cpu_arm
44 OK anchor_hits=1 the_concurrent_tier_judgement_pools_the_runs_and_pairs_the_gpu_arm_against_the_cpu_arm
45 OK anchor_hits=1 the_numbers_of_a_concurrent_run_line_read_back_as_written
46 OK anchor_hits=1 the_sensitivity_points_are_the_middle_tier_with_sixteen_unit_batches_and_five_hundred_millisecond_intervals
47 OK anchor_hits=1 the_sensitivity_points_are_the_middle_tier_with_sixteen_unit_batches_and_five_hundred_millisecond_intervals
48 OK anchor_hits=1 the_sensitivity_points_are_the_middle_tier_with_sixteen_unit_batches_and_five_hundred_millisecond_intervals
49 OK anchor_hits=1 the_trajectories_report_the_peak_the_positive_readings_and_the_last_reading
50 OK anchor_hits=1 the_trajectories_report_the_peak_the_positive_readings_and_the_last_reading
51 OK anchor_hits=1 the_trajectories_report_the_peak_the_positive_readings_and_the_last_reading
52 OK anchor_hits=1 the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same
53 OK anchor_hits=1 the_geometry_is_formed_only_when_the_written_segments_are_half_live_and_the_forced_segment_is_a_half_live_data_segment
54 OK anchor_hits=1 the_empty_segment_trajectory_counts_calls_ending_with_more_empty_segments_per_device
55 OK anchor_hits=1 the_yield_bound_clause_not_hit_is_reported_under_the_upper_bound_reading
56 OK anchor_hits=1 the_head_line_of_a_concurrent_product_says_the_gc_thread_yields_after_unlock
57 OK anchor_hits=1 the_registered_device_bytes_of_every_tier_are_accepted_by_the_make_filesystem
58 OK anchor_hits=1 the_tiers_have_the_registered_slots_segments_and_fragment_segments
59 OK anchor_hits=1 the_geometry_is_formed_only_when_the_written_segments_are_half_live_and_the_forced_segment_is_a_half_live_data_segment
60 OK anchor_hits=1 the_forced_fallback_names_the_lowest_fragment_segment_not_yet_forced_holding_only_data_units_and_at_most_half_live
61 OK anchor_hits=1 the_forced_fallback_names_the_lowest_fragment_segment_not_yet_forced_holding_only_data_units_and_at_most_half_live
62 OK anchor_hits=1 the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same
63 OK anchor_hits=1 the_wall_of_a_call_window_encloses_the_gc_thread_cpu_of_an_empty_call
64 OK anchor_hits=1 the_apparatus_count_of_live_slots_per_segment_pins_the_hand_counted_values_and_agrees_with_the_core
65 OK anchor_hits=1 the_apparatus_count_of_live_slots_per_segment_pins_the_hand_counted_values_and_agrees_with_the_core
66 OK anchor_hits=1 the_apparatus_count_of_live_slots_per_segment_pins_the_hand_counted_values_and_agrees_with_the_core
67 OK anchor_hits=1 the_median_wall_of_five_repeats_is_the_middle_one_and_of_four_the_mean_of_the_middle_two
68 OK anchor_hits=1 one_pass_differing_from_the_apparatus_in_one_segment_or_left_uncertain_does_not_agree
69 OK anchor_hits=1 the_stage_three_lines_carry_the_registered_keys_first_and_in_order
70 OK anchor_hits=1 the_stage_three_timing_wrapper_counts_every_live_slot_call_and_hands_the_decisions_through
71 OK anchor_hits=1 the_stage_three_lines_carry_the_registered_keys_first_and_in_order
72 OK anchor_hits=1 the_two_arms_count_the_same_live_slots_per_segment_on_the_small_pool_as_the_apparatus_and_a_greedy_call_is_timed
73 OK anchor_hits=1 the_two_arms_count_the_same_live_slots_per_segment_on_the_small_pool_as_the_apparatus_and_a_greedy_call_is_timed
74 OK anchor_hits=1 the_construction_of_every_tier_fits_a_one_level_inode_tree
75 OK anchor_hits=1 the_user_change_overwrites_the_last_unit_of_a_live_data_file_drawn_by_the_seed
76 OK anchor_hits=1 the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same
77 OK anchor_hits=1 the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same
78 OK anchor_hits=1 the_construction_of_every_tier_fits_a_one_level_inode_tree
79 OK anchor_hits=1 the_window_over_which_any_task_exited_or_its_number_was_reused_is_void
80 OK anchor_hits=1 the_window_over_which_any_task_exited_or_its_number_was_reused_is_void
81 OK anchor_hits=1 the_start_and_the_elapsed_of_the_wall_clock_are_read_on_the_monotonic_raw_clock
82 OK anchor_hits=1 the_start_and_the_elapsed_of_the_wall_clock_are_read_on_the_monotonic_raw_clock
83 OK anchor_hits=1 the_wall_of_a_call_window_encloses_the_gc_thread_cpu_of_a_long_spinning_call
84 OK anchor_hits=1 two_devices_writing_the_same_bytes_at_the_same_offset_store_one_chunk
85 OK anchor_hits=1 chunk_written_back_to_zeroes_is_not_stored_and_reads_back_zero
86 OK anchor_hits=1 copied_pool_shares_the_chunks_until_one_of_its_devices_writes_a_chunk
87 OK anchor_hits=1 every_construction_phase_prints_a_progress_line_when_the_interval_is_zero
88 OK anchor_hits=1 every_construction_phase_prints_a_progress_line_when_the_interval_is_zero
89 OK anchor_hits=1 every_construction_phase_prints_a_progress_line_when_the_interval_is_zero
90 OK anchor_hits=1 spin_windows_mostly_off_the_cpu_are_not_run_rather_than_failed
91 OK anchor_hits=1 every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate
92 OK anchor_hits=1 every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate
93 OK anchor_hits=1 the_cards_in_the_environment_variable_are_used_without_leasing
94 OK anchor_hits=1 without_the_environment_variable_the_cards_are_leased_through_the_channel
95 OK anchor_hits=1 without_the_variable_and_the_channel_the_apparatus_refuses_instead_of_enumerating_the_cards
96 OK anchor_hits=1 without_the_variable_and_the_channel_the_apparatus_refuses_instead_of_enumerating_the_cards
97 OK anchor_hits=1 only_the_handed_over_card_among_those_seen_on_this_host_is_kept
98 OK anchor_hits=1 a_handed_over_card_not_seen_on_this_host_is_refused
99 OK anchor_hits=1 the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count
100 OK anchor_hits=1 the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count
101 OK anchor_hits=1 the_every_card_arm_opens_as_many_cards_as_are_handed_over
102 OK anchor_hits=1 the_cards_option_names_the_first_handed_over_cards_in_the_handed_over_order
103 OK anchor_hits=1 the_cards_option_names_the_first_handed_over_cards_in_the_handed_over_order
104 OK anchor_hits=1 the_cards_option_asking_for_more_than_are_handed_over_is_refused
105 OK anchor_hits=1 without_the_cards_option_the_every_card_arm_names_every_handed_over_card
106 OK anchor_hits=1 the_cards_option_on_a_host_handed_no_card_names_none_without_a_usage_error
107 OK anchor_hits=1 lockstep_with_the_cards_option_two_names_the_first_two_handed_over_cards
108 OK anchor_hits=1 lockstep_with_the_cards_option_not_a_number_is_a_usage_error
109 OK anchor_hits=1 lockstep_without_the_cards_option_opens_every_handed_over_card
110 OK anchor_hits=1 lockstep_with_the_cards_option_but_without_an_every_card_arm_is_a_usage_error
111 OK anchor_hits=1 the_every_card_arm_cards_line_tells_how_many_cards_and_whether_the_cards_option_named_them
112 OK anchor_hits=1 only_the_every_card_arm_takes_its_cards_from_those_the_cards_option_named
113 OK anchor_hits=1 only_the_every_card_arm_takes_its_cards_from_those_the_cards_option_named
114 OK anchor_hits=1 lockstep_with_the_cards_option_zero_is_a_usage_error
115 OK anchor_hits=1 lockstep_with_the_cards_option_one_is_a_usage_error
116 OK anchor_hits=1 resuming_a_product_with_the_same_every_card_arm_cards_goes_on
117 OK anchor_hits=1 resuming_a_product_whose_every_card_arm_opened_another_count_stops
118 OK anchor_hits=1 resuming_a_product_with_the_same_count_named_another_way_stops
119 OK anchor_hits=1 resuming_a_product_without_an_every_card_arm_cards_line_stops
120 OK anchor_hits=1 the_every_card_arm_cards_are_recorded_only_with_an_every_card_arm_on_a_host_handed_cards
121 OK anchor_hits=1 the_every_card_arm_cards_are_recorded_only_with_an_every_card_arm_on_a_host_handed_cards
122 OK anchor_hits=1 every_opening_in_the_apparatus_is_handed_the_same_openings_table_of_this_process
123 OK anchor_hits=1 the_apparatus_builds_an_empty_openings_table_only_for_the_one_kept_for_this_process
```

## 9 新加的单测（第 3 条）

数：HEAD 里旧路径那一份 88 条 `#[test]`，现版本 118 条（`grep -c '#\[test\]'` 两份），多 30 条，都在 diff 里加的行里（`grep -c '^>.*#\[test\]'` 30）；其中 1 条另挂 `#[cfg(feature = "tests-on-a-real-gpu")]`（12155），另有改过的老用例 1 条（10008）和那条真卡对拍（12132，旧的改成先录后判）。
「变异行」列是 `crates/mutations.tsv` 里指到这条测试的行（以第六段点名的测试名数；上面 8.3 的行号）。

| 件 | 测试（简名；行号） | 断什么 | 变异行 |
|---|---|---|---|
| 1 | `the_cards_in_the_environment_variable_are_used_without_leasing` | 变量给了卡串又有通道：用变量那一串、领 0 次，来路核判有来路 | 93 |
| 1 | `without_the_environment_variable_the_cards_are_leased_through_the_channel` | 没变量有通道：领 1 次、用领到那串，来路核判有来路 | 94 |
| 1 | `without_the_variable_and_the_channel_the_apparatus_refuses_instead_of_enumerating_the_cards` | 都没有：报错（那句带 `task-server/task submit`）、领 0 次、来路核判没有 | 95、96 |
| 1 | `only_the_handed_over_card_among_those_seen_on_this_host_is_kept` | 本机三张只交居中那张：只留它，点名一张卡臂点的就是它 | 97 |
| 1 | `a_handed_over_card_not_seen_on_this_host_is_refused` | 交来的卡本机看不到：报错 | 98 |
| 1 | `a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller`、`a_process_seen_on_no_card_is_reported_not_run_to_the_caller` | 测的是取卡公共模块的纯函数（`gpus_this_process_is_on_judged_against_the_handed_over`），读不成或读到空都交回本次未跑；装置本体不调它 | `crates/mutations.tsv` 的 7948、7949 行（改公共模块那一份，文件列不在这四份里，不在 123 行内） |
| 2 | `the_every_card_arm_opens_as_many_cards_as_are_handed_over`（改了老的 `…names_every_handed_over_card_and_gives_one_limit_per_card` 10008） | 交两张或三张，全部卡臂点名的就是交来的、上限一张一份 | 101、11 |
| 2 | `the_cards_option_names_the_first_handed_over_cards_in_the_handed_over_order` | 交来先后 03、01、02，`--cards 2` 取 03、01，按 PCI 排，上限两份 | 102、103 |
| 2 | `without_the_cards_option_the_every_card_arm_names_every_handed_over_card` | 不给 `--cards` 取全部三张 | 105 |
| 2 | `the_cards_option_asking_for_more_than_are_handed_over_is_refused` | `--cards 4` 比交来的三张多：报错、不截 | 104 |
| 2 | `the_cards_option_on_a_host_handed_no_card_names_none_without_a_usage_error` | 这一台没交卡、`--cards 2`：交回空，不当命令行不对 | 106 |
| 2 | `lockstep_with_the_cards_option_two_…`、`lockstep_without_the_cards_option_…`、`…_zero_…`、`…_one_…`、`…_not_a_number_…`、`…_but_without_an_every_card_arm_…`、`a_subcommand_without_the_every_card_arm_refuses_the_cards_option` | 命令行解析：2 认、不给认成全部、0 与 1 同一句报错、非数报错、没点全部卡臂给 `--cards` 报错、别的子命令不认 `--cards` | 107、109、114、115、108、110；最后一条无 |
| 2 | `the_every_card_arm_cards_line_tells_how_many_cards_and_whether_the_cards_option_named_them` | `name=every_card_arm_cards` 行写张数、`named_by`、PCI 清单 | 111 |
| 2 | `only_the_every_card_arm_takes_its_cards_from_those_the_cards_option_named` | 只有全部卡臂从 `--cards` 挑过的取；点名一张卡臂与 CPU 臂仍从交来的全部取；`name=run` 行型号串仍是三张 | 112、113 |
| 2 | `the_every_card_arm_cards_are_recorded_only_with_an_every_card_arm_on_a_host_handed_cards` | 点了全部卡臂且交了卡才打、才核；没交卡、没点全部卡臂都不打 | 120、121 |
| 2 | `resuming_a_product_with_the_same_every_card_arm_cards_goes_on`、`…whose_every_card_arm_opened_another_count_stops`、`…with_the_same_count_named_another_way_stops`、`…without_an_every_card_arm_cards_line_stops` | 续跑核对四种：新产物与同张数同开法接着跑；张数不同停；张数同而开法不同停；有结果行没有这一行停 | 116、117、118、119 |
| 3 | `every_opening_in_the_apparatus_is_handed_the_same_openings_table_of_this_process` | 两次问到的是同一张表（`is_the_same_table_as`） | 122 |
| 3 | `the_apparatus_builds_an_empty_openings_table_only_for_the_one_kept_for_this_process` | 读自己源文件，现建空表只有 1 处 | 123 |
| 4 | 无新测试 | 改名由编译保证；老用例里的回答枚举换名（10077、11471 起各处） | 老的变异行 34 到 36（并发让路甲类第 5 行）罩到 6648、6652 的判停 |
| 5 | `the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count`（12869 起） | 一次性库上：输入字节编解回原样；只录那一趟不调持卡判（不取卡）；只判那一趟读回的参照等于装置自数 `[6, 2, 2]`；持卡判被调 1 次 | 99、100；另有 7945 行（改公共模块，不在 123 行内） |
| 5 | `the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`（12132，带特性） | 真库、真卡上走对拍结论入库 | 无（需要真卡） |
| 5 | `a_greedy_call_on_a_real_card_runs_the_stage_three_on_the_gpu_within_the_call_on_the_small_pool`（12155，带特性，新） | 小池 greedy 一次 GC 调用里丙段至少调一次、那一截不长于整次调用 | 无（需要真卡） |

没有测试的改动（只列，没判对错）：
- 取卡链 `cards_taken_for_this_process`（5773，`catch_unwind` 接 panic、没交卡交回空、读交来的先后）、`cards_of_this_process`（5816）、`cards_seen_by_the_command`（5824，打 `name=gpus`、`clause=cards_not_taken`、退 78）、`the_task_server_hands_over_cards_to_this_process`（5685）、`emit_the_gpus`（6007）、`opening_of_the_command` 里 `no_cards_from_the_task_server` 那一支（6074）：都需要真环境，无单测、无变异行。
- `lockstep_command`、`concurrent_command`、`stage_three_cost_command` 的接线（取卡在造镜像之后、`cards_option_exceeds_the_cards_handed_over` 退 2、没交卡退 77）：无单测、无变异行（变异行里 6286 到 6376、7587 到 7640、8756 到 8790 三段 0 命中，量法：`tools/anchor_lines.py`）。
- 先录后判里的 `whether_of_the_number_written`（8553）的 panic 一支、`allocation_records_and_the_unit_area_of_the_pairing_input_bytes` 的短读与剩字节一支：一次性库那条只走正常往返，没有喂坏字节的用例。
- 持卡判入口 `judged_on_the_cards_that_the_gpu_arm_…`（8666）的几处判红：8682 取卡没成、8695 数不成、8696 次数、8702 逐遍比，只有真卡才走到，无变异行。
- 两条老变异行没改的：件 4 的改名没有变异行，编译即罩。

## 10 检查单八格（第 4 条）

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 这一次新加的非测试 `.expect`：8576、8593（盘号，写下时是 `u64::from(u32)`，8516 起的写端）、8596（跨槽数，`u64::from(u16)`）；`panic!`：8557（读回的 0 或 1，写端只写这两个值）、8583（`UnitAreaStart::following_the_journal_ring` 的逆，装置只造默认环，`allocator.rs:389`、`:405` 的默认环正好落在段边界，单测 12869 起用同一个单元区往返）；8682、8695、8702 与 8696 的 `assert_eq!` 都在 `judgement_of_the_assertions_on_the_cards`（公共模块 147）的 `catch_unwind` 里，变成「存两边不一致」的红结论，不是进程退出。没有新加的 `unwrap`、`unreachable!`。读回字节有摘要核（`crash_library.rs:2216` 起的过库函数读回对不上返回 `Err`），所以 8557、8583 只会在写端与读端不同步时走到，二者在同一份文件里、读端又在参照一侧的闭包里（改了键就变）。注：这是前提推出来的「走不到」，没有喂坏字节的用例走到它们。 |
| 类型与名字 | 不成立 | 新单测 `the_every_card_arm_cards_are_recorded_only_with_an_every_card_arm_on_a_host_handed_cards` 用 `LOCKSTEP_ARMS[2]`、`[5]`、`[0]`、`[1]`（12651、12654、12656）取臂，名字取法 `LockstepArm::of_the_name` 在 2770；这四处是这一次新加的（HEAD 里的下标用法是 5239 等产品段的 `[0]`，不算这一次）。`_ =>` 新加行 0、`as` 0、单字母名 0（`grep` 新加行）。另推不出：`FirstHandedOverCards`（5865）、`first_handed_over`（5902）里的 `first` 指「交来先后里排前头的 N 张」，`.claude/agent-common.md:38` 禁序数与位置代称（例 `first_stream`），算不算它缺主 agent 的读法；`cards_named_now: usize` 与 `CARDS_THE_EVERY_CARD_ARM_OPENS_AT_LEAST: usize`（5857）是裸整数的张数，命令行那一边有 `NonZeroUsize` 包着。 |
| 不兼容旧代码与旧格式 | 成立 | 新加行里没有旧版本分支、旧布局号、迁移与 `legacy` 一类（`grep` `legacy|旧版|旧盘|旧库|旧产物|兼容` 在 diff 新加行 0 命中）。续跑读回产物：`every_card_arm_cards_differing_from_the_product_being_resumed`（5951）对「有结果行而没有这一行」同一条停（出路写换一个 `--product`），不分版本认；对拍输入字节没有版本号，对不上就 panic（8557、8583）。注释里没有新加的经过。 |
| 不新建判定库 | 成立 | 写入点四处各判：① 结果行追加到 `--product` 那份文件（含 `name=verdict`、`repeat_done`、`premise`、`positive_control`）：依据 `.claude/rules/verification.md:54`（库里只放剪枝按单元键复用时要读的键）与 `:55`（产物、日志、临时现场归各自的目录）；这些行是这一次实验的产物与续跑记账，装置文件头第 1 行 `admission: always`（每跑一次现量、上一次不作保），续跑只补没做完的重复，且 crates 版本摘要变了就停（`the_crates_changed_since_the_earlier_commands` 4241），没有别的跑拿它当判定按键复用；所以不是 `:56` 说的「另写一张 TSV」。② 内联真卡测试的对拍结论：经验证包 `CrashLibrary`（prune-server 客户端），成立。③ 单测 12869 的 `DisposableLibrary`：自起的临时服务，符合 `:56` 的自证一句。④ 构建脚本往 `OUT_DIR` 写两张键表：编译期派生数据，与 `crates/singlefs-verification/build.rs:2` 同做法，不是判定。没有新目录、sqlite、RocksDB。保留意见：登记第一节未定项 4（`research/prompts/e202-preregistration.md:79`）写「缺：主 agent 认这一族没有判定要进库」，本腿在登记里没找到主 agent 认过的记录。推翻 ①：别的装置或门禁读这些 `name=verdict` 行当判定去复用。 |
| 所有任务多机可用 | 成立（diff 射程内） | diff 里新读的本机状态：`GPU_CARDS_BY_PRIORITY` 与领卡通道（服务交，各台各得各的）；取卡模块经多机配置的判法认「这一台是谁」、读 `nvidia-smi`，没有写死主机名与地址（测试里的 `192.0.2.1` 是文档保留段）。会让两台判出不同结局的只有「服务交的卡里有没有这一台的」（77）与「交来几张」：`--cards N` 与交来的张数比，同一条命令在交两张与交三张的两趟里退 2 与照跑不同，续跑也被张数核对挡下；这是 `all` 的性质（张数服务现算），推不出该不该算「不该不同」。diff 外的 `HOST_LOAD_CORES_AT_MOST` 16.0 与登记作废第 3 条（`research/prompts/e202-preregistration.md:426`）同一个数，内核节拍 100 没核。 |
| 测试优先多线程 | 成立，另报一处冲突 | 串着跑的理由站得住：登记第十四节（`research/prompts/e202-preregistration.md:568`）「计时的跑全部串行」，成对差两边要同机先后、`construction-timing` 读整个进程的常驻内存、`stage-three-cost` 两臂是计时；`geometry-check` 按登记线程数分波并行（7839 起）；`.claude/heavy-thread-exempt` 里没有这份（`grep -rnF e202` 0 命中）。新加的单测并行跑：与第 6 节那条一次性库对全仓库的冲突是唯一一处并行下的问题。 |
| 带 --item 的命令与项 | 成立（静态部分） | 第 8 节：123 行包名与目标都是新包、原文各恰好命中 1 次、点名的测试各筛得到 1 条 `#[test]`；`.claude/gate.d/stage-inputs.tsv:31` 登记的包名含 `singlefs-background-gpu-tests`；`crash-case:` 项里没有点这个包的。没核：点名测试能否够着被改坏的函数、整行真跑能否红（命令 3、4）。 |
| 新崩溃流多罩了什么 | 成立（没有新崩溃流） | 装置里 `enumerate_crash_states|every_crash|CrashImage` 0 命中，`tests/` 下 `enumerate_crash_states` 0 命中；`SINGLEFS_CRASH` 只在 12152 的注释与 12159 的 `RunStage::of_this_process()` 里（读选段，不枚举崩溃状态）。 |

## 11 事实表 facts.md 里写错或漏掉的

- 第一节第 5 样第 9 条只列产品段的 `LOCKSTEP_ARMS[…]`，漏了这一次新加的三处位置取臂（12651、12654、12656）；它们才是「这一次新加」的。
- 第〇·一把 `ForegroundFsync` 改 `AnswerOfTheCallerBeforeTheNextBatch` 写成改名；核心层的枚举同时从 2 个变体变成 3 个（HEAD 的 `compaction_plan.rs` 里 `ForegroundFsync` 只有 `NoneWaiting`、`Waiting`，用 `git show HEAD:…` 读的），装置不用第三个。
- 第一节第 3 样「带 --item」说名字带 E202 而文件是别处的另有 3 行（7945、7948、7949）：7948、7949 的名字列不带 E202（名字是「真卡取卡模块：…」），点名的是装置里新加的两条 `a_process_*` 测试；所以这两条新测试有变异行，只是文件列不在这四份里。
- 第二节第 5 样第 1 条、第四节第 5 样第 2 条说「单元图认不认 `singlefs_background_gpu_tests::` 前缀没核」：本腿核了，认（本报告 7.3）。
- 别的行号（5703、5773、5724、4285、12130、12153、12869）本腿逐个 `grep -n` 对过，都对得上。

## 12 要主 agent 跑的命令

没有一条是我跑的。下面每条：一条交任务；在仓根 `/home/fy5090/code/singlefs` 起（`--cwd` 默认此刻目录）；跑完看哪几行。

1. 装置里 116 条（118 减去 2 条带特性的）单测，含新加的 30 条与改名后的编译：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 e202_gpu_cpu_gc_comparison（bin 内联单测）; 范围=1 个测试目标; 为什么=补轮核新加的 30 条单测与搬迁、接口改名之后能编能过' task-server/task submit -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 `test result: ok. 116 passed`（116 是我推的：118 减 2）。
2. 构建脚本把两张登记表的 39 个入口都解出来、GPU 一侧每组都有键：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 pairing_conclusion…; 范围=1 个测试目标; 为什么=核两张入口登记表经单元图解得出、GPU 一侧 31 组都算出键' task-server/task submit -- cargo test -p singlefs-background-gpu-tests --test pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes`
   看：`exit=0`；日志里 `every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate ... ok`；构建脚本 panic 出现「入口写错了」就是 7.1 错。
3. 变异表的静态判（锚点、点名测试够不着被改坏函数的 `UNREACHABLE`、替换文里未定义的标识符）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=crates-mutation-rows.py static-check; 范围=crates/mutations.tsv 全表静态判; 为什么=核点到测试包 src 的 123 行没有 ANCHOR、UNREACHABLE、UNDEFINED' task-server/task submit -- python3 scripts/mutation/crates-mutation-rows.py static-check /home/fy5090/code/singlefs`
   看：输出里文件列是测试包 src 四份的行，没有 `ANCHOR`、`UNREACHABLE`、`UNDEFINED`、`MALFORMED`；末行 `REACHABILITY_CHECKED` 的计数。
4. 先列项再点名新加的 31 行（8.3 里 93 到 123 行共 31 行是点名新测试的行，另第 11 行点名改过的老用例）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=只列项; 为什么=取这一次新加变异行的项名' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   看：`exit=0`，项名里含「E202 装置取卡」「E202 全部卡臂」「E202 续跑核全部卡臂张数」「E202 内联真卡测试先录后判」「V88」「V89」；点名跑的整条命令照 `.claude/agents/implementation-writer.md` 第 1d 步「变异」那一条，`<项>` 填这些项名。
5. 带特性的编译检查（含那两条带 `tests-on-a-real-gpu` 才编的单测与装置里的 `cfg_attr`）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=cargo clippy 测试包带 tests-on-a-real-gpu; 范围=singlefs-background-gpu-tests 全部目标; 为什么=核 cfg_attr 的 allow 与带特性才编的两条单测编得过' task-server/task submit -- cargo clippy -p singlefs-background-gpu-tests --all-targets --features tests-on-a-real-gpu -- -D warnings`
   看：`exit=0`，没有 `dead_code`、`unused` 的警告点到装置 8500 到 8710 之间。
6. 复现第 6 节那条并行下的冲突（要卡，由服务交；一条命令把两条用例放进同一个进程，参数里不带 `--test-threads`）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置内联的一次性库用例与真卡对拍用例同进程并行; 范围=2 条测试函数; 为什么=核一次性库设着时真卡用例开库会不会开到临时库' task-server/task submit -- cargo test -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison -- the_record_stage_of_the_live_slot_pairing_takes_no_card the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`
   看：`exit=0` 且两条都 `ok`，是没打中；红、或日志里 prune-server 连不上（退 75）、对拍结论写进了临时库，是打中。一趟没打中不算没有（竞态），只能当线索。

没有命名检查要交：本腿没改 `crates/` 与 `scripts/`。

## 13 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 件 1 取卡 | 对得上 | 卡只来自环境变量或领卡通道，没有自己枚举；CPU 的活都在取卡前；没卡退 78、没这一台的卡退 77。备注 A、B、D 推不出，C 是走不到的支路（第 2 节） |
| 件 2 全部卡臂 | 对不上 | 登记 49、297 写「不给 `background_gpu_devices`」，代码 2846 起点名交来的卡；`--cards`、`every_card_arm_cards` 行登记没有；缺第十二节修订；另有条件性对不上（点名一张卡臂的卡可能不在全部卡臂的 N 张里） |
| 件 3 开卡表 | 对得上 | 四处开卡都取同一份 `OnceLock` 静态；对 D26 已定项 13 第 4 条（决策文件 253 行） |
| 件 4 接口改名 | 对得上，不全是纯改名 | 名字在核心层都查得到、旧名 0 命中；枚举多一个变体、特征方法一减二增，装置不受影响（没编） |
| 件 5 先录后判 | 对得上 + 一条并行下的冲突 | 录、键、判、存放都是验证包那一份库；e202 二进制里一次性库用例与开全仓库的真卡用例没有互斥守卫（推的） |
| 件 7 两张入口登记表 | 对得上 | 39 个入口各恰好命中 1 处；前缀只定包，目标取自请求；缺口：参照表无逐组测试与变异行，bin 组不核丙段核 |
| 搬迁：123 变异行 | 对得上 | 包名与目标都是新包，原文各恰好命中 1 次，点名测试各筛得到 1 条 |
| 搬迁：旧路径活引用 | 0 处 | 排除 `research/` 后 15 行：1 行搬迁登记、14 行带日期的 `records/` |
| 搬迁的回退 | 推的 | 开跑自检的摘要射程不再含装置自己，登记停机第 2 条对装置自己被改不响 |
| 新加单测 | 30 条 | 分组见第 9 节；无变异行的新测试 3 条（`a_subcommand_…`、两条真卡）；取卡链与三个命令接线无单测 |
| 检查单：走不到的理由 | 成立 | |
| 检查单：类型与名字 | 不成立 | 新单测用 `LOCKSTEP_ARMS[下标]` 取臂 4 处；`first` 命名推不出 |
| 检查单：不兼容旧代码与旧格式 | 成立 | |
| 检查单：不新建判定库 | 成立 | 依据 `.claude/rules/verification.md:54`、`:55`（产物归各自目录）与 `:56`（自证用临时服务）；① 保留意见：登记未定项 4 没找到认的记录 |
| 检查单：所有任务多机可用 | 成立 | `--cards` 的结局随服务交几张而变，推不出该不该算不同 |
| 检查单：测试优先多线程 | 成立 | 另报并行冲突一处 |
| 检查单：带 --item | 成立 | 静态部分；够不着被改坏函数的判（命令 3）没跑 |
| 检查单：新崩溃流 | 成立 | 没有新崩溃流 |

## 14 什么现象会推翻这些结论

- 件 1：装置本体里出现自己枚举卡、或开卡调用先于取卡；服务在本文件声明下交了领卡通道（备注 C）。
- 件 2：登记第十二节已有这一条修订；服务交来的卡串永远按 PCI 升序（条件性那条）。
- 件 3：装置里出现第二处建开卡表。
- 件 4：命令 1 编不过（穷举 `match` 或缺特征方法）。
- 件 5：命令 6 两条同进程跑不出冲突，或门禁把它们分进程跑；或公共模块在开库前已拿守卫。
- 件 7：命令 2 的构建脚本 panic 或 `every_gpu_side_group_is_keyed…` 红。
- 搬迁回退：登记或实验页说明摘要射程只管被测的两个包。
- 检查单各格：对应格里写的那一处依据，被命令 1 到 5 任一条跑红。

## 15 没做什么

- 没编、没跑测试、没跑变异、没起装置；上面标「推的」的数（116 条、冲突、先后次序）都没复跑。要跑的在第 12 节，判决要引的数由主 agent 复跑。
- 没替主 agent 判：全部卡臂要不要补第十二节修订、`--cards` 张数由命令行写算不算违反 `.claude/rules/test-script-discipline.md:75`、「首位」命名算不算位置代称、登记未定项 4 认没认。
- 没写模型（正推腿这一轮没有模型，没有 `rerun.sh`）；没建仓副本与编译目录。草稿目录 `~/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-forward` 里只有只读脚本与它们的输出，共 184K（`du -sh`），是交接材料，不是副本，留给主 agent 读。
- 没攻：续跑核对的边角（`pci_bus_addresses` 不进核对）、进程级开卡表的多线程压力、钟速单测打「本次未跑」就返回、事务员线索第 3、6、7、9、10、11 条逐条判真伪、123 行变异罩不罩得住第 1 到 5 件的每个判红分支，这些归攻方腿；第 9 节只给了没有变异行的清单。
- 没读 `research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md`。
