# 事实表：GPU 测试包 src 四份（三方补轮用）

日期：2026-11-16。只读，没改仓内文件，没跑项目代码。草稿目录 `/home/fy5090/.local/share/singlefs/work/backfill-facts-gpu-tests-src/`。

## 〇、相对 HEAD 是新增还是修改（命令原样输出）

`git status --short -- crates/singlefs-background-gpu-tests`（节选本批四份；整包都是 `A`）：

```
A  crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs
A  crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs
A  crates/singlefs-background-gpu-tests/src/lib.rs
A  crates/singlefs-background-gpu-tests/src/reference_units_entries.rs
```

`git diff HEAD --stat -- <路径>` 与 `wc -l`：

```
.../src/bin/e202_gpu_cpu_gc_comparison.rs  | 13207 +++  （13207 行）
.../src/gpu_side_units_entries.rs          |   168 +++  （168 行）
.../src/reference_units_entries.rs         |    73 +++  （73 行）
crates/singlefs-background-gpu-tests/src/lib.rs | 4 ++++ （4 行）
```

`git log --oneline -3 -- <路径>`：四份都没有输出（HEAD 的历史里没有这四个路径）。

搬迁：`git status --short` 里有 `R  crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs -> crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs`（暂存区认作改名）。`git log --diff-filter=D … 'crates/singlefs-background-gpu/src/bin/*e202*'` 无输出、退出 0（删在工作区暂存里，还没提交）；旧版本在 HEAD：`git log --oneline -3 -- crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` 给 `b14b6fe92`、`ff5dca7d2`。`git ls-tree -r --name-only HEAD | grep units_entries` 只命中 `crates/singlefs-verification/src/judging_units_entries.rs`：两张登记表与 `lib.rs` 是全新的，不是搬来的。另登记在 `.claude/gate.d/path-moves.tsv:673`（旧路径 → 新路径，2026-11-13，`records/2026-11-11-GPU两段与按需领卡核查.md`）。

diff.patch：`git diff HEAD -- crates/singlefs-background-gpu-tests/src > …/diff.patch`，退出 0，**13476 行**（量的，`wc -l`），sha256 `d8bd6627745c839b1e30d7760205e8f5251f8c8ac2b37ebee71e19e22f9a8f37`。它把 e202 当整份新增（不带 `-M`）。

## 〇·一、e202 搬迁之外改了什么

旧版 `git show HEAD:crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` 存到 `…/e202-head-old.rs`（11570 行）；`diff 旧 新 > …/e202-move.diff` 退出 1，2345 行，128 个块，删 245 行、加 1882 行（量的，`grep -c '^<'`/`'^>'`）。按块归成这几样（行号左旧右新）：

| 改了什么 | 旧 → 新 | 依据（diff 块） |
|---|---|---|
| 文件头加任务服务声明 `// task-server-gpu-cards: all` | 新 4 | `3a4` |
| 取卡改成只用服务交的：去掉 `main` 里每条要卡的子命令现建 `VulkanInstanceOfThisProcess::created_for_this_process()` 与 `GpusOfTheCommand`，换成 `GPU_CARDS_BY_PRIORITY` 或领卡通道、造完起步镜像之后才取（`cards_handed_to_the_apparatus` 5703、`cards_taken_for_this_process` 5773、静态 `CARDS_SEEN_BY_THIS_PROCESS` 5724）；交来的卡对不上本机独显报错 | 旧 5569-5665、8449-8478 → 新 5658-6092、9209-9227 | `5574a5665,5716`、`5579,5584c5723,6032`、`5624,5634c6072,6083`、`8454,8461c9215,9216` |
| 造起步镜像挪到取卡之前（lockstep、concurrent 两处删旧位置、在开头检查之后造） | 旧 5878-5886、7126-7134 删 | `5878,5886d6372`、`7126,7134d7630`、`5849a6277,6336` |
| 全部卡臂：旧 `background_gpu_devices: None`（产品自己填满本机全部）→ 新点名服务交的卡；加 `lockstep --cards <张数>`、`name=every_card_arm_cards` 行与续跑核对 | 旧 2663、2815 → 新 2696、2848-2853、5856-6004、9010-9031、9066-9074 | `2815c2848,2853`、`8282a9008,9032` |
| 每臂的卡从各自那一份取：`lowest_card`、型号串改成按臂取（`discrete_gpus_of_the_arm` 5995、`gpu_models_in_the_run_line` 5523） | 旧 5062-5064、5442-5447 → 新 5146-5152、5522-5530 | `5062,5064c5146,5152`、`5442,5447d5538` |
| 开卡表进程级一份：加静态 `OPENINGS_NOT_RETURNED_IN_THIS_PROCESS` 与 `openings_not_returned_in_this_process`，每处开卡多传这一张 | 新 4282-4294、4313-4319、4330、4332、8347、8349、8449、8451 | `4236a4282,4294`、`4255,4256c4313,4319` |
| 跟核心层接口改名：`ForegroundFsync`/`foreground_fsync` → `AnswerOfTheCallerBeforeTheNextBatch`/`answer_before_the_next_batch`；`LINEAGE_CONTAINER_BLOCKS_OF_ONE_COW` → `…_COPY_ON_WRITE`；`devices_not_opened_at_the_mount` 与 `GpuDeviceNotOpenedAtTheMount` 去掉；两个包装加 `use_no_gpu_from_the_next_batch`、`put_the_gpus_opened_since_the_settings_were_handed_in_in_effect` | 新 3065-3075、8277-8287、3360、6603-6682 | `3030a3065,3075`、`2963,2966d3000`、`7739,7742d8230`、`3315,3316c3360,3361` |
| 内联真卡测试改成先录后判（输入与装置自数先写进判定库、判段读回才取卡）：加 `#[path]` 引 `common_pairing_input_through_the_library`、编解对拍字节的几个 fn、持卡判入口 | 新 139-158、8491-8712、12127-12168 | `121a139,142`、`125a156,158`、`7988,7992c8491,8712`、`11301,11302c12127,12129` |
| 注释去掉经过：旧 147 行「用户 2026-11-01 定」删掉；文件头「装置住产品 GPU 包」改成住测试包 | 新 180、46-47 | `147c180`、`31,32c46,47` |
| 新加单测：取卡、`--cards`、续跑核对、先录后判两段、取卡模块报未跑、开卡表只建一张（含读自己源文件计数） | 新 12189-12960、13183-13205 | `11346a12189,12960`、`11568a13183,13205` |
| 用法串加 `[--cards <全部卡臂开几张>]`；退出码 77/78 的说明改成按服务交卡 | 新 8930、29-30 | `8205c8930`、`27c29,30` |

没核：别的小块（`13c14,15`、`29c32,44` 等）是文件头说明随上面几样改写；逐块全文在 `…/e202-move.diff`。

## 一、e202_gpu_cpu_gc_comparison.rs（13207 行）

### 1. 它做什么

文件头注释原话（`e202_gpu_cpu_gc_comparison.rs:1-47`，含前四行任务服务声明，整段照抄）：

```
// admission: always 每跑一次都在今天这份 crates 上现造池、现量挂钟与 CPU，计时每次都是新观测，上一次跑完不替这一次作保
// task-server-gpu-arguments: lockstep concurrent sensitivity stage-three-cost
// task-server-resumable: lockstep concurrent sensitivity
// task-server-gpu-cards: all
//! E202（GPU与CPU两种模式GC对照组）的装置：跑前登记 `research/prompts/e202-preregistration.md`。
//!
//! 同一负载、同一池，核心层后台入口 `MountedSession::run_compaction_in_the_background` 在 `background_compute=cpu` 与 `background_compute=gpu`
//! 两种挂载参数下各跑，量 GC 挂钟（量 W）、GC 调用窗口里本进程用掉的主机 CPU（进程口径，量 C）、两者成对相减省下的（量 S）、省下的随盘容量的走向
//! （量 T，`judge-trend` 读三档产物判）与阳性对照（量 P：每条臂配一条把每个确认的条目在 CPU 上再判一遍的多判臂）；并发跑另量前台 fsync 延迟（量 F）。
//!
//! 子命令（登记第十四节「起法」）：
//! - `geometry-check --product <文件>`：三档各一条线程造起步镜像、核几何（登记第五节「负载」第 4 条），每档另在 CPU 臂上不计时地交替跑一次、
//!   按第六节「前提 N」的阶梯数出每次跑的批数；不要卡。
//! - `lockstep --reading occupancy --unit-area <2288MiB|4GiB|8GiB> [--user-changes <次数>] [--batch-units <个数>] [--cards <张数>] --product <文件>`：交替跑六条臂；
//!   `--cards` 只管「全部卡」那两条臂开几张（见下面「全部卡」臂那一段），`--arms` 里没点这两条臂之一的给了算用法错。别的子命令都不认 `--cards`。
//! - `concurrent --reading occupancy --unit-area <…> --foreground-interval-milliseconds <毫秒> --product <文件>`：并发跑三条臂。
//! - `sensitivity --batch-units 16 --foreground-interval-milliseconds 500 --product <文件>`：中档几何敏感性两点（登记第八节）。
//! - `judge-trend --product <文件> <三档 lockstep 产物…>`：量 T 的分类与停机条款第 2 条（三档的 crates 版本相同）。
//! - `stage-three-cost --unit-area <2288MiB|4GiB|8GiB> --product <文件>`：丙段代价（登记第十二节修订第 9 条，只供用户定岔路、不进判据）：
//!   几何核同一份起步镜像上 CPU 臂与 GPU 点名一张卡臂各数五遍每段活槽数取中位，再各照产品 greedy 跑一次 GC 调用、报挂钟与其中丙段那一截。
//! - `construction-timing --unit-area <…> --live-data-units <P,…> --chunks <each-device-copies|shared> --product <文件>`：造池分阶段计时
//!   （盘取那一档的大小，P 取给的几个小值，照同一套造法串着造，每阶段一行 `name=construction_phase`：挂钟、步数、每步挂钟的头尾八分之一平均、
//!   版本大小、常驻内存与内存盘块字节）；不跑 GC、不要卡、不进判据，只给造池提速与内存上限定数。
//!
//! 产物是标准输出与 `--product` 那份文件上同样的结果行（`E202 name=<种类> 键=值 …`），末行 `name=done emitted=<行数>`；进度打到标准错误。
//! `--product` 那份文件只追加：同一条命令重起时读它里面已有的 `name=repeat_done` 行，跳过做完的重复（登记第十四节「续跑入口」）。
//!
//! 退出码：0 正常；2 命令行不对；3 停机条款触发（登记第十一节「停机」，产物照打）；4 阳性对照没过（产物照打）；
//! 77 一条臂都没跑到（服务交的卡里没有这一台的）；78 准入或运行条件不满足（CPU 钟分辨率自检不过、读不到登记的线程数、写不了产物、
//! 服务既没交卡也没给领卡通道、领卡没成、交来的卡对不上本机的独显）。
//!
//! 卡只用服务交的（[`cards_handed_to_the_apparatus`]）：环境变量 `GPU_CARDS_BY_PRIORITY` 有卡串就只开那几张；没有而有领卡通道时，
//! 起步镜像造完、开设备之前那一刻才经验证包的按需领卡领（`singlefs_verification::gpu_card_configuration::cards_handed_over_or_leased_on_demand`）；
//! 两样都没有报错退出，不自己枚举本机的卡。要卡的四个子命令都先做完只用 CPU 的活（读产物、开跑检查、造起步镜像、丙段代价的装置自数），再取卡。
//! 「全部卡」臂开几张每次跑时由 `lockstep --cards <张数>` 设：给了 N 只开服务交来的卡里排在前头的 N 张（先后照服务交来的卡串），
//! 不给开交来的全部；`background_gpu_devices` 点名开的那几张，本机没交的卡不开。N 小于 2（只开一张时全部卡臂与点名一张卡臂是同一条臂）、
//! 不是数、多于交来的张数都退 2（命令行不对），不截成交来的张数；这一台一张都没交的照「服务交的卡里没有这一台的」退 77。
//! 开了几张、照什么开的打在 `name=every_card_arm_cards` 一行；续跑时这两样与产物里已有的这一行有一处不同，或产物里有结果行而没有这一行，
//! 打 `name=stopped clause=cards_option_differs_from_the_product_being_resumed` 退 2，两种张数的重复不混在同一份产物里。
//! 这份源文件头的张数声明写 `all`：服务把那台此刻的空卡都交来，装置照 `--cards` 只开其中 N 张；没给 `--cards` 而只交来一张时这条臂与点名一张卡臂相同，报本次未跑。
//! `lockstep`、`concurrent`、`sensitivity` 与 `stage-three-cost` 不拆成先录后判（理由写在各自入口旁）；
//! 只比结果、不量挂钟的那一条内联真卡测试走先录后判：输入与装置自数的参照先在 CPU 上写进判定库，判段读回才取卡（测试包 `common_pairing_input_through_the_library`）。
//!
//! 换不掉的那几处：真卡（GPU 臂经产品 GPU 包开服务交的独显）、真钟（单调钟与 CPU 钟）、`/proc`（机器负载与开卡线程）；
//! 判法与计数写成纯函数，同包测试里喂合成输入。
//! 装置住产品 GPU 的测试包（它同时依赖产品 GPU 包与验证包，按需领卡与判定库都经验证包那一份）：只读、只驱动与观测，不改 `crates/` 下别的文件。
//! 决定几何与判据门槛的常量写成本地常量、开跑先与核心层公开的同一个量回比（[`local_constants_disagreeing_with_the_core`]），对不上走停机条款第 1 条。
```

一句概括（读完的）：E202 实验装置的单进程 bin：在两块稀疏内存盘上现造三档起步镜像，用同一串前台改动分别以 CPU 与 GPU（一张卡、全部卡、各带一条 CPU 多判臂）挂载跑核心层后台整理，量 GC 挂钟、进程 CPU、成对差、前台 fsync 延迟与丙段代价，按跑前登记的锚点、作废、停机条款判，结果行追加进 `--product` 文件可续跑；卡只取服务交的；内联真卡测试走验证包判定库的先录后判。

按职责分段（行区间是 `grep -n` 现取的定义行；同包测试 9239-13207 另列）：

| 段 | 行区间 | 里面的函数与类型 |
|---|---|---|
| 参数解析 | 8926-9237 | `USAGE` 8928、`CommandOfTheApparatus` 8938、`options_and_positionals` 8960、`the_reading_option_is_occupancy` 8983、`tier_of_the_options` 8991、`user_changes_of_the_options` 8997、`cards_of_the_every_card_arm_of_the_options` 9010、`command_of_the_arguments` 9033、`main` 9211 |
| 本地常量与开跑回比（造输入与判据的前提） | 160-457 | 常量 163-240、`UnitAreaTier` 245-299、`data_units_of_a_cluster_segment` 302、`local_unit_area_slots_of_device` 306、`smallest_device_bytes_the_make_filesystem_accepts` 316、`the_make_filesystem_accepts_the_device_bytes` 322、`local_publish_budget` 327、`publishes_of_one_intent` 335、`ConstantMismatch` 341、`local_constants_disagreeing_with_the_core` 350 |
| 基础件：SHA-256 | 459-660 | `Sha256Digest` 541（`new` 548、`feed` 556、`feed_u64` 578、`finish_as_hex` 583）、`sha256_compress` 601 |
| 基础件：钟与 `/proc` | 662-864 | `StartOnTheMonotonicRawClock` 672、`ClockReadFailed` 692、`thread_cpu_spent_in` 703、`task_identifier_of_this_thread` 711、`cpu_of_the_schedstat_line` 724、`cpu_of_every_task_of_this_process` 730、`advances_of_the_thread_cpu_clock_in_a_millisecond` 759、`HostLoadReading` 778、`busy_ticks_of_the_kernel_cpu_counter_line` 785、`ticks_of_the_process_status_line` 800、`host_load_reading` 808、`cores_used_outside_this_process` 828、`the_gpu_opening_thread_is_still_running` 846、`is_the_comm_line_of_the_gpu_opening_thread` 862 |
| 造输入：内存盘、录制摘要、内容 | 866-1444、4612-4647 | `RecordedDeviceOperation` 870、`feed_the_operation_into_the_digest` 893、`RecordingDigest` 942、`ChunkStorage` 991、`RecordingMemoryDevice` 1004 与 `BlockDevice` 实现 1163、`chunk_bytes_of_the_pool` 1147、`empty_pool_devices`（`cfg(test)`）1268、`empty_pool_devices_storing_the_chunks` 1277、`copied_pool_devices` 1302、`splitmix64_next` 1319、`fill_the_content_of_a_data_unit` 1328、`unit_index_of_the_user_change` 1339、`expected_content_digest` 1349、`ReaderOfTheMemoryPool` 1360、`read_content_digest` 1406、`publish_the_user_change` 4614 |
| 造输入：起步镜像与几何 | 1446-2686 | `default_journal_ring_bytes` 1449、`make_filesystem_parameters_of_the_pool` 1460、`version_of_trees` 1476、`new_pool_with_the_first_file` 1484、`data_unit_slots_of_the_inodes` 1525、`slots_under_an_unreleased_record_on_device_zero` 1549、`unit_area_of_device_zero` 1558、`segment_start_of` 1568、`GeometryReading` 1575、`SegmentOfTheChoice` 1605、`segments_holding_a_record_other_than_a_data_unit` 1621、`segment_of_the_choice` 1640、`fragment_segment_to_force` 1661、`geometry_of_the_records` 1683、`slots_exempt_from_the_mapping` 1729、`segment_the_greedy_choice_picks` 1752、`StartingImage` 1806、`ConstructionReport` 1818、`ConstructionPhase` 1851、`resident_bytes_of_*` 1898/1904、`PhaseInProgress` 1917、`TimingOfAConstructionPhase` 1930、`ConstructionClock` 2018、`publish_while_constructing` 2117、`raise_the_floor_to_the_admission_ceiling` 2132、`inode_*` 2186-2238、`append_a_unit_to_the_end_of_a_file` 2243、`write_a_live_data_unit` 2268、`append_a_spacer_unit` 2290、`construct_the_starting_image` 2351、`construct_a_starting_image_of_the_size` 2357、`construct_a_starting_image_on_the_clock` 2379 |
| CPU 参照 | 2897-2970、3089-3172、4665-4685、8046-8073、8093-8099 | `rejudged_unit_checksums` 2898、`rejudged_region_records` 2921、`rejudged_live_slots` 2947、`region_of_the_batch` 3092、`rejudged_batch_on_the_cpu` 3107、`rejudge_the_batches_of_the_call_on_the_cpu` 4665、`live_slots_of_each_segment_counted_by_the_apparatus` 8046（装置自数，不调核心层）、`StageThreeCounter::OnTheCpu` 8078/8093 |
| GPU 一侧 | 2687-2858、2972-3087、4265-4372、8076-8158、8212-8473 | `ComputeOfTheArm` 2691、`RejudgingOfTheArm` 2702、`LockstepArm` 2709、`LOCKSTEP_ARMS` 2715、`background_compute_settings_of_the_arm` 2827、`GpuComputeRejudgedOnTheCpu` 2973、`GpuOfTheRun` 4268、`vulkan_instance_for_the_gpu_arm` 4275、`OPENINGS_NOT_RETURNED_IN_THIS_PROCESS` 4285、`openings_not_returned_in_this_process` 4290、`open_the_gpu_of_the_arm` 4296、`measuring_mount` 4353、`StageThreeCounter::OnTheGpu` 8080、`stage_three_counts_of_the_counter` 8138、`GpuComputeTimingTheStageThree` 8222、`stage_three_share_of_one_greedy_call` 8335、`stage_three_counts_on_the_arm` 8430、`allocation_records_and_unit_area_of_the_image` 8476 |
| 跑法与计时窗口 | 3174-3399、4374-4610、4687-4861、6462-6925 | `CallWindowReading` 3179、`CallWindowClocks` 3189、`EndOfTheWindow` 3197、`ClocksOfTheGcThread` 3204、`any_task_read_at_the_opening_is_missing_or_behind_at_the_closing` 3256、`call_window_between` 3279、`measure_one_gc_call` 3314、`CallerOfTheLockstepRun` 3337、`pacing_of_the_measuring_mount` 3366、`compaction_switches_of_the_measuring_mount` 3376、`ForcedFallbackOfTheRun` 4461（`choose` 4483、`before_the_call` 4538、`after_the_call` 4572）、`lockstep_threads` 4718、`CallerOfTheConcurrentRun` 4604/6604、`concurrent_threads` 6764 |
| 对拍与判定 | 3401-4025、4864-5656、6928-7486、8160-8210、8840-8924 | `GcCallRecord` 3404、`record_of_the_call` 3433、`path_anchor` 3509、`gpu_work_anchors` 3562、`batches_whose_*` 3614/3646、轨迹 3669-3790、`gc_wall_clock_of_the_run` 3793、`calls_failing_the_inclusion_checks` 3802、统计 3825-3974、`positive_control_of_an_arm` 3996、`LockstepRunSummary` 4864、`class_one_anchors_of_a_lockstep_run_hold` 4998、`run_lockstep_once` 5040、`judgement_of_a_lockstep_tier` 5306、`verdict_of_the_repeat_checks` 5494、`run_one_lockstep_repeat` 5531、并发跑 6928-7486（`run_concurrent_once` 7084、`judgement_of_a_concurrent_tier` 7284、`run_one_concurrent_repeat` 7363）、`median_wall_of` 8161、`every_pass_agrees_with_the_apparatus_count` 8172、`judge_trend_command` 8840 |
| 判定入库与复用 | 4240-4250、6202-6240、7488-7533、8491-8706 | `the_crates_changed_since_the_earlier_commands` 4241、`repeats_already_done` 6203、`concurrent_repeats_already_done` 7489（续跑读产物行）、`REFERENCE_UNITS_GROUP_OF_THE_APPARATUS_COUNT` 8504、`pairing_input_bytes_of_the_allocation_records_and_the_unit_area` 8516、`whether_of_the_number_written` 8553、`allocation_records_and_the_unit_area_of_the_pairing_input_bytes` 8572、`reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment` 8619、`live_slots_per_segment_of_the_reference_bytes` 8645、`judged_on_the_cards_that_the_gpu_arm_…_small_pool` 8666 |
| 分卡与开卡 | 2775-2824、5658-6033、5848-5854 | `DiscreteGpuSeenByThisProcess` 2777、`discrete_gpus_seen_by_this_process` 2784、`reason_the_arm_is_not_run` 2808、`OpeningOfTheCommand` 5661、`CardsOfTheCommand` 5669、`the_task_server_hands_over_cards` 5677、`…_to_this_process` 5685、`cards_handed_to_the_apparatus` 5703、`CardsSeenByThisProcess` 5715、`CARDS_SEEN_BY_THIS_PROCESS` 5724、`text_carried_by_the_panic` 5731、`discrete_gpus_among_the_handed_over` 5748、`cards_taken_for_this_process` 5773、`cards_of_this_process` 5816、`cards_seen_by_the_command` 5824、`reason_the_arm_is_not_run_on_the_cards` 5849、`CardsOfTheEveryCardArm` 5861、`discrete_gpus_named_for_the_every_card_arm` 5883、`fields_of_the_every_card_arm_cards_line` 5914、`the_every_card_arm_cards_are_recorded` 5936、`every_card_arm_cards_differing_from_the_product_being_resumed` 5951、`discrete_gpus_of_the_arm` 5995 |
| 输出产物与子命令入口 | 4027-4263、6007-6188、6245、7562、7731-7765、7767-7900、7902-8028、8712 | `field_value` 4030、`fields_of_a_result_line` 4047、`ProductWriter` 4062、`result_lines_already_in_the_product` 4105、退出码 4122-4127、`source_files_and_digests` 4132、`combined_digest_of_the_sources` 4166、`emit_the_crates_version` 4178、`checks_before_the_run` 4196、`registered_heavy_threads` 4253、`emit_the_gpus` 6007、`opening_of_the_command` 6035、`emit_whether_the_make_filesystem_accepts_the_tier` 6095、`emit_the_construction` 6122、`lockstep_command` 6245、`concurrent_command` 7562、`sensitivity_command` 7759、`geometry_check_command` 7808、`construction_timing_command` 7937、`stage_three_cost_command` 8712 |
| 同包测试 | 9239-13207 | `mod tests`，118 个 `#[test]`（其中 12130、12153 两个另挂 `#[cfg(feature = "tests-on-a-real-gpu")]`） |

### 2. 注释里引到的编号（D、E、C、I 起头）

命令：`grep -nE '\b[DECI][0-9]+[a-z]?\b' <文件>`；字符串字面量里的 `E202`（结果行前缀、进度行）不算条款引用，下表只列注释。

| 行 | 编号 | 那一句 |
|---|---|---|
| 5 | E202 | `//! E202（GPU与CPU两种模式GC对照组）的装置：跑前登记 `research/prompts/e202-preregistration.md`。` |
| 231 | E185 | `/// 造碎片之后抬 F 至多几轮（迭代上界；同 E185（GC一批搬迁的挂钟拆分） 装置的做法）。` |
| 311 | D23 | `/// `JournalRingTooLargeForDevice`，D23（journal 的角色与格式） 已定项 19）。同包测试 …` |
| 2131 | E185 | `/// 抬 F 到准入上限（会话的分配器与现行那一版，同 E185（GC一批搬迁的挂钟拆分） 装置的做法）。` |
| 4282 | D26 | `/// 这个进程里各张卡上在开、到点没回来的开卡那一张表（D26（后台整理与放置回收） 已定项 13 第 4 条：表在进程级，各次开卡共用；` |
| 4374 | E185 | `// ---- 强制选段后备（登记第十二节修订第 8 条：几何核后一半照 E185（GC一批搬迁的挂钟拆分） 走强制选段，六条臂同一点名法、同一序列）----` |

另：全文大量引「登记第 N 节 / 第十二节修订第 N 条 / 第九节变异 / 停机条款第 N 条 / 作废第 N 条 / 甲类、乙类第 N 行」，指的都是 `research/prompts/e202-preregistration.md`（在工作区，`git ls-files` 有）；`R4` 指 `transaction::publish_partial_write` 文档（1337、2242、2345、4628）。C、I 起头的编号没有。

去重清单「这份文件压着哪些条款」：E202（跑前登记各节）、E185、D23（journal 的角色与格式）已定项 19、D26（后台整理与放置回收）已定项 13 第 4 条。登记位现查：`.claude/kb/decisions.md:64` D23、`:67` D26；`.claude/kb/decisions/26-后台整理与放置回收.md:246` 是「已定项 13：后台任务的计算放在哪、怎么开、什么节奏」；`.claude/kb/experiments.md:217` E202「跑前登记已写、装置未跑」；E185 只在 `.claude/kb/experiments-history.md:6031`（已归档）。

### 3. 代码轮检查单八格的原始事实

#### 走不到的理由

计数（命令 `python3 -I …/tools/calls.py <文件> 1 9238`，配对括号取整段；量的）：产品段（1-9238）`.expect` 128、`panic!` 8、`assert_eq!` 6、`assert!` 2、`unreachable!` 0、`.unwrap()` 0；同包测试段（9239-13207）全表在 `…/e202-calls-tests.tsv`（`.expect` 86、`panic!` 6、各类 `assert` 372，按 `grep -c` 量）。

产品段的 `panic!` 与 `assert`（全部，带所依赖的前提）：

| 行 | 种类 | 消息原文 | 依赖的前提 |
|---|---|---|---|
| 593 | assert! | 补位之后正好凑满整块 | SHA-256 补位算术 |
| 604 | assert! | 一块 64 字节正好是 16 个四字节的字 | 常量 64 能被 4 整除 |
| 2124 | panic!（`unwrap_or_else`） | 建池的用户改动发得成：{refusal:?} | 造池那串改动核心层不拒；拒了整个进程 panic，不走停机条款 |
| 2312 | assert_eq! | 垫片文件的号接着活数据文件连着发 | 核心层 inode 号按水位连发 |
| 2386 | assert_eq! | P 是 32 的倍数：交错那一段一半活的、一半垫片，正好写满整段 | 调用方给的 P（三档常量；`construction-timing` 由 7931 先过滤） |
| 2391 | assert_eq! | P ÷ 2 是 K 的倍数：… | 同上 |
| 2434 | panic! | 造池至多要 {…} 条 inode 记录，一层 inode 树装不下（至多 {} 条）：{refusal:?} | 一层 inode 树装得下；测试 13092 钉三档 |
| 2463 | assert_eq! | 建的 inode 号从水位起连着发 | 核心层水位连发 |
| 4625 | assert_eq! | 前台用户改动只覆盖写文件的最后一个单元（…R4） | `unit_index_of_the_user_change` 1345 的式子 |
| 4644 | panic!（`unwrap_or_else`） | 前台第 {change_number} 次用户改动发得成：{refusal:?} | 前台改动核心层不拒；在 GC/前台线程里 panic 时经 `join().expect` 再 panic |
| 8557 | panic! | {field_name}只写 0 或 1，读回 {other}：录的与读的布局对不上 | 库里读回的字节就是本进程那份布局写的 |
| 8583 | panic!（`unwrap_or_else`） | 读回的单元区起点不落在聚簇段边界上：{refused:?} | 同上 |
| 8682 | panic!（`unwrap_or_else`） | 判段要开卡，取卡没成：{reason} | 判段持卡判；panic 由 `judgement_of_the_assertions_on_the_cards` 收成判红（推的，那份在测试共用模块，没逐行核） |
| 8695 | panic!（`unwrap_or_else`） | GPU 点名一张卡臂数不成：{reason} | 同上 |
| 8696 | assert_eq! | 修订第 9 条：各量 5 次 | `stage_three_counts_of_the_counter` 循环 5 次 |
| 8702 | panic! | GPU 点名一张卡臂有一遍逐段与读回的装置自数不同：… | 对拍判红的出口 |

产品段 128 处 `.expect` 逐处（行、消息原文）接在下面两段原样贴（来自 `…/e202-calls-src.tsv`，种类列是 `.expect` 的行）。按前提归类（推的，按消息读）：整数换型装得下（`u64::try_from(len)`/`usize::try_from` 约 70 处，如 557、907、1030、1062-1064、1075、1158 …）；锁没中毒（`.lock().expect("…没在别处 panic 过")` 1017、1101、2982、5079、5121、6795、6811、6839、6844、6888、6898、6922、8267、8389、8405）；通道与线程（4752、4753、4757、4841、4850、4852、6909、6912、7849）；核心层不拒（1488 mkfs、1505 取号、1506 暖机、1518 新池新建文件、2144 抬 F、2422/2552 可写挂载、2497 单元区装得下、4802/6854/8396「盘表是挂载核过的那一份、现行那一版有树：后台入口不拒」）；钟单调（686）；常量非零与边界（3369、3389、4563、8576、8593、8596）。

产品段 `.expect` 逐处（行 / 消息原文），前 64 处：

```
385	段数装得进 u64
448	10 秒
449	10 秒
557	一段字节数装得进 u64
566	凑满了一块
686	CLOCK_MONOTONIC_RAW 单调不减：此刻不早于起点
907	写长装得进 u64
1017	录制摘要只在借着盘的那条线程上喂，没在别处 panic 过
1030	512
1062	块内偏移
1063	块内长度
1064	请求里的偏移
1075	64 KiB
1101	块表只在借着盘的那条线程上改，没在别处 panic 过
1158	块数装得进 u64
1169	读长装得进 u64
1192	写长装得进 u64
1194	64 KiB
1256	512
1344	小于活数据文件数
1418	净荷容量装得进 u64
1435	读回的字节数装得进 u64
1461	512
1480	第一个文件发过之后，会话的现行那一版都有树
1488	mkfs
1502	小于 256
1505	取号
1506	暖机
1510	暖机两代根
1518	新池新建文件
1709	段数
1951	步数装得进 u64
1967	步数装得进 u64
1979	条数装得进 u64
1981	条数装得进 u64
2126	版数
2127	抬 F 串数
2144	抬 F 到准入上限
2218	片数装得进 u64
2223	一条记录装得进空树
2225	条数装得进 u64
2228	扇出装得进 u64
2236	条数装得进 u64
2254	净荷容量装得进 u64
2261	一个单元的净荷不是空的
2422	造数据挂载那一次可写挂载
2497	单元区装得下下一个用户数据单元
2549	片数装得进 u64
2552	造碎片挂载那一次可写挂载
2666	单元数
2668	映射单元条数
2681	P 装得进 usize
2961	段下标
2982	多判记录只在 GC 线程上写，没在别处 panic 过
3011	条目数
3027	条目数
3048	条目数
3049	段数
3148	单元字节
3169	段数
3369	1 毫秒不是 0
3389	批大小至少 1
3465	版数
3472	单元数
```

后 64 处：

```
3531	条数
3532	条数
3627	批数
3642	批数
3664	批数
3679	读数个数
3809	调用数
4396	意图数
4400	意图数
4413	意图数
4475	P
4563	碎片段起点在聚簇段边界上、装得进 6 字节
4622	P
4633	净荷容量装得进 u64
4640	一个单元的净荷不是空的
4645	单元下标
4752	GC 线程在等
4753	GC 线程交回会话
4757	GC 线程在等
4802	盘表是挂载核过的那一份、现行那一版有树：后台入口不拒
4841	前台线程在等
4850	GC 线程没有 panic
4852	前台线程没有 panic
5079	录制摘要没在别处 panic 过
5091	P
5107	卡数
5121	多判记录没在别处 panic 过
5178	调用数
5180	调用数
5229	调用数
5514	重复次数
6052	次数
6385	重复次数
6662	调用数
6684	次数
6695	调用数
6789	用户改动次数装得进 u32
6795	会话锁没在别处 panic 过
6811	计数锁没在别处 panic 过
6839	计数锁没在别处 panic 过
6844	会话锁没在别处 panic 过
6854	盘表是挂载核过的那一份、现行那一版有树：后台入口不拒
6888	计数锁没在别处 panic 过
6898	计数锁没在别处 panic 过
6909	GC 线程没有 panic
6912	前台线程没有 panic
6922	会话锁没在别处 panic 过
6944	改动数
7127	P
7141	卡数
7200	调用数
7202	调用数
7371	重复次数
7645	重复次数
7849	几何核那条线程没有 panic
8054	段数装得进 usize
8068	段下标
8267	丙段计时只在调后台入口的那条线程上写，没在别处 panic 过
8389	丙段计时只在这条线程上写，没在别处 panic 过
8396	盘表是挂载核过的那一份、现行那一版有树：后台入口不拒
8405	丙段计时只在这条线程上写，没在别处 panic 过
8576	写下的盘号装得进 u32
8593	写下的盘号装得进 u32
8596	写下的跨槽数装得进 u16
```

#### 类型与名字

- 裸整数当地址、长度、卡号、片号（`grep -nE '…(slot|inode|offset|segment_start|chunk_index|device|task|card)…: (Option<)?(u64|…)'` 在 1-9238 行，命中节选）：槽号与段起点用裸 `u64`——`segment_start_of(slot: u64, unit_area_start_slot: u64)` 1568、`SegmentOfTheChoice.start_slot: Option<u64>` 1606、`segment_of_the_choice` 1641-1642、`fragment_segment_to_force` 1664、`geometry_of_the_records` 1688-1689、`StartingImage.fragment_segment_starts: BTreeSet<u64>` 1811、`ForcedChoiceOfAnIntent.issued_region_start_slot: Option<u64>` 4383、`ForcedFallbackOfTheRun.fragment_segment_starts` 4462、`region_of_the_batch(…, unit_area_start_slot: u64)` 3094、`data_slots_now: BTreeSet<u64>` 2618/4491、`slots_under_an_unreleased_record_on_device_zero → BTreeSet<u64>` 1549、`data_unit_slots_of_the_inodes → BTreeMap<(u64, u64), u64>` 1529；inode 号裸 `u64`：`lowest_live_data_inode` 1409/1809/2271/4463/4617/4720/6766、`first_inode`/`inode_count` 1527-1528、`inode: u64` 2246、`first_spacer_inode` 2293；字节偏移裸 `u64`：`RecordedDeviceOperation.offset_in_bytes` 873/879/884、`pieces_of_the_range(offset_in_bytes: u64, …) -> Vec<(u64, usize, usize, usize)>` 1048-1051（四元组无名）、`zero_the_range` 1071；块号 `chunk_index: u64` 1087；线程号 `u64`：`cpu_by_task: BTreeMap<u64, Duration>` 734/3185、`gc_thread_task: Option<u64>` 3207-3209；卡用 `pci_bus_address: String` 2778。核心层的 `SlotNumber`、`DeviceIdentity`、`InodeNumber` 只在跟核心层接口的边上用（如 1424、1744、4562）。
- 缩写与单字母名字：`grep -nE '\blet (mut )?[a-z]\b|\|[a-z]\||fn [a-z]\(|for [a-z] in'` 0 命中。
- `_ =>` 兜底：`grep -nE '(^|[^a-z_])_ *=>'` 0 命中。具名兜底：6652 `_more_than_one => false`（`match waiting_answers` 的 2 及以上）、8557 `other => panic!`、8881 `_other_result_lines => {}`（judge-trend 跳过别的结果行）、9177 `other_subcommand => return Err`、1106 `Some(_) | None =>`、3326 `(Err(_), _, _) | (_, Err(_), _) | (_, _, None) => None`、6940 `(None, _, _) | (_, None, _) | (_, _, None) => true`；测试 11832、12495、12508 `Ok(_) | Err(_) => panic!`。
- `as` 数值转换：两处，都带 `#[allow(clippy::cast_precision_loss, reason = …)]`：3826 `duration.as_nanos() as f64`（3821-3824）、3831 `count as f64`（3829）。别的换型一律 `try_from(..).expect(..)`。

#### 不兼容旧代码与旧格式

- 认旧版本、旧布局号、迁移路径的分支：没有找到（读完全文；`grep -nE '旧|legacy|_old|old_'` 只命中「旧落点」1823/2341 这类说现状的词与 SHA 变量名 614/621）。
- 续跑读旧产物：`every_card_arm_cards_differing_from_the_product_being_resumed` 5951：产物里有结果行而没有 `every_card_arm_cards` 行一律停下、出路「换一个 --product」（5966-5972），是同一条拒绝，不分版本认。`the_crates_changed_since_the_earlier_commands` 4241 拿 `crates_version` 摘要比，不同就停（6065-6071）。
- 留着的「改前」实现：`ChunkStorage::EachDeviceCopiesItsChunks` 992 注释原话「每块盘各存各的块，拷镜像整份复制：改前的存法，只留给 `construction-timing` 对照与同包测试对拍。」——是对照实现，不是兼容旧盘；但注释带「改前」（见第 5 样）。
- 对拍输入字节布局（8507-8540 写、8572-8608 读）没有版本号；读回对不上照 `panic!`（8557、8583）与 `PairingInputBeingRead::finish`（8606）。

#### 不新建判定库

写入点逐处：
1. `ProductWriter::appending_to` 4068-4082（`create_dir_all` 4073、`OpenOptions … append` 4076）与 `emit` 4084-4096：结果行追加到 `--product` 给的文件，含 `name=verdict`、`repeat_done`、`premise`、`positive_control` 这些判定行；续跑时 `result_lines_already_in_the_product` 4105 读回、`repeats_already_done` 6203 / `concurrent_repeats_already_done` 7489 按它跳过做完的重复。是实验产物文件，不经 prune-server（见第 5 样）。
2. 内联真卡测试的对拍结论：`pairing_concluded_on_the_cards_through_the_pairing_library`（12134 调；实现在 `tests/common_pairing_input_through_the_library/mod.rs:297`，310 行 `CrashLibrary::open(CrashLibraryClass::InputPairing)`），经验证包判定库。
3. 单测 12798 用 `DisposableLibrary::new("e202-live-slot-pairing-record-stage")` 12869（注释 12794：验证包自己起的临时 prune-server），自证库。
4. 构建脚本写 `OUT_DIR/gpu_side_units_keys.tsv`、`reference_units_keys.tsv`（`build.rs:2-3`）：编译时的键表，装置经 `common_pairing_input_through_the_library/mod.rs:35-40` `include_str!` 读；不是判定。
没有别的 sqlite、RocksDB、TSV 写入点（`grep -nE 'prune|verdict_store|RocksDB|sqlite|\.tsv|create_dir|OpenOptions|fs::write|File::create'` 只命中上面几处）。

#### 所有任务多机可用

逐处读本机状态：
- 环境变量：`GPU_CARDS_BY_PRIORITY`（经常量 `VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER` 146）5687、5775；领卡通道 `CardLeaseChannelOfTheCaller::read_from_the_environment_now()` 5690、5776；`RunStage::of_this_process()` 12159（注释 12152 点 `SINGLEFS_CRASH_AMPLIFICATION_STAGE`）；`std::env::args` 9212；编译时 `env!("OUT_DIR")` 在被 `#[path]` 引进来的共用模块里。没有 `HOME`、`~`、主机名。
- 相对仓根的路径（装置要从仓根起）：4135-4136 `crates/singlefs-background-gpu/src`、`crates/singlefs-core/src`（算 crates 摘要）；4254 `.claude/heavy-thread-default`。读不到的报错写「装置要从仓根起」（4140、4255）。
- `/proc`：`/proc/thread-self` 713-715、`/proc/self/task` 732-735、847、`/proc/stat` 810、`/proc/self/stat` 816、`/proc/self/status` 1899、测试 10914 `/proc/thread-self/schedstat`。读的是本进程，判作废（机器忙 `HOST_LOAD_CORES_AT_MOST` 16.0 写死 223、开卡线程在跑）会随机器负载变。`KERNEL_TICKS_PER_SECOND_REPORTED_TO_USER_SPACE = 100` 写死 774（注释说 x86-64 与 aarch64 固定）。
- 写死的地址：测试 12198-12199 `192.0.2.1:0:32G`、`192.0.2.1:1:32G`（注释 12197 说是文档用的保留地址）；产品段没有。
- 卡：`discrete_gpus_among_the_handed_over` 5748 交来的卡本机看不到就报错 78；`the_task_server_hands_over_cards` 5677 两样都没有就停（6074-6082）。
- 产物路径由 `--product` 给，可相对可绝对，装置不改写。

#### 测试优先多线程

- `geometry-check`：7838-7853，各档一条线程，`threads = opening.registered_heavy_threads.min(tiers_accepted.len())` 7839，`tiers_accepted.chunks(threads.max(1))` 一波一波 `std::thread::scope` 7841-7851，打 `name=parallelism geometry_threads=… registered_heavy_threads=…` 7854-7860。线程数取登记值 `registered_heavy_threads` 4253（读 `.claude/heavy-thread-default` 第一条非注释行，读不到退 78：4226-4232）。
- `lockstep`、`concurrent`：各起两条线程（前台、GC）是跑法本身（4731-4760 交替经通道；6783-6914 并发拿同一把锁），不是为提速切片；`lockstep` 六条臂、五次重复串着跑（6395-6439、5540），`concurrent` 同（7656-7703）。理由写在 6242-6244、7560-7561（「成对差的两边就不再同时同机」）。
- `construction-timing` 串着造（7961 循环，7922 注释「各造一遍、按给的次序串着造，常驻内存读的是整个进程」）；`stage-three-cost` 两条臂串着（8766）。
- 同包测试：小池用例 10636、11851、12067 各单线程造一次 3072 MiB×2 的池；10761、10815 起两条替身线程只为拿线程号（10766-10775）。
- `.claude/heavy-thread-exempt` 里没有这份（`grep -rnF e202_gpu_cpu_gc_comparison .claude/heavy-thread-exempt` 无输出）。

#### 带 --item 的命令与项

命令 `grep -rnF '<名>' .claude/gate.d/*.tsv scripts/crash/ scripts/experiment-apparatus/replay.sh .claude/heavy-thread-exempt .claude/crates-review-exempt crates/mutations.tsv scripts/mutation/tables/`（原样输出在 `…/registry-grep.txt`，137 行）。命中：
- `.claude/gate.d/comment-history-lag.tsv:11`：`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs	5	# 存量，改净后删这一行`
- `.claude/gate.d/experiment-number-directories.tsv:17`：`crates/singlefs-background-gpu-tests/src/bin	e-number-names	# 登记里写死成入库装置的实验，装置住在产品 GPU 的测试包里（e202_gpu_cpu_gc_comparison.rs 这类）`
- `.claude/gate.d/path-moves.tsv:673`：旧路径 → 新路径那一行（见第〇节）。
- `crates/mutations.tsv`：第二段（文件）恰是这份的 **121 行**（`awk -F'\t' '$2=="crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs"' crates/mutations.tsv | wc -l`；量的）；cargo 参数段一律 `-p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- <测试名>`。名字带 E202 而文件是别处的另有 3 行（7945 指 `tests/common_pairing_input_through_the_library/mod.rs`，7948、7949 指 `tests/common_discrete_gpus_of_this_host/mod.rs`）。这些行就是 crates-mutation-replay `--item` 可点的项。
- `scripts/mutation/tables/`：0 份文件命中（`grep -rlF … | wc -l` = 0）；`scripts/crash/`、`scripts/experiment-apparatus/replay.sh`、`.claude/crates-review-exempt`：无命中。
- 包级的登记（不点名这份）：`.claude/gate.d/stage-inputs.tsv:31` `crash-point-replay.sh	tests-of:singlefs-verification tests-of:singlefs-background-gpu-tests …`；`.claude/gate.d/stage-owners.tsv:6` `product-gpu-real-card-tests`（这个包带 `tests-on-a-real-gpu` 才编的真卡测试）；`.claude/gate.d/unsafe-allowances.tsv:51` 是这个包另一份 tests 文件。`stage-inputs.tsv` 的 `crash-case:` 行没有一条点这个包。

#### 新崩溃流多罩了什么

没有。查过的范围：这份全文 `grep -nE 'crash|Crash'` 只命中 139、12159、12804-12807、12872（用验证包 `crash_library` 的 `PairingJudged`、`RunStage`、`CrashLibrary` 存取对拍结论，不枚举崩溃状态）；`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行与 `scripts/crash/` 都不点这份。

### 4. 测它的测试与变异在哪

- `crates/singlefs-background-gpu-tests/tests/` 下引它的：没有（`grep -rniE 'e202' crates/singlefs-background-gpu-tests/tests` 退出 1）。反过来它经 `#[path]` 引 `tests/common_discrete_gpus_of_this_host/mod.rs`（154-155）与 `tests/common_pairing_input_through_the_library/mod.rs`（157-158）。
- 文件里的 `#[cfg(test)]`：`mod tests` 9239-13207，`grep -c '#\[test\]'` = **118**（量的），其中 12130、12153 两条另挂 `#[cfg(feature = "tests-on-a-real-gpu")]`；另 1267 `empty_pool_devices`、1865 `ConstructionPhase::EVERY_PHASE` 两处只给测试编。
- `crates/mutations.tsv`：121 行（上一格）；`scripts/mutation/tables/`：0。

### 5. 线索，没核

1. **结果行当判定存**：`--product` 文件里存 `name=verdict` 等判定行、续跑据 `repeat_done` 跳过重复（4084-4119、6203-6240、7489-7533），不经 prune-server。它是实验产物（`research/results/` 那类），是不是落在 `.claude/agent-common.md`「写」一节「不另开目录、TSV…存判定」的射程里，要主 agent 判。
2. **注释里的经过**（`.claude/agent-common.md`「写」一节「代码与脚本的注释、说明只写现状」）：992「改前的存法」、9524「改前（各存各的）与改后」、9679「小池单测（任务 6790）读到过这样一窗」、10809「任务 6790 挂钟读 `CLOCK_MONOTONIC` 时 22 窗里 21 窗…」、10812「本机的 NTP 正把钟调慢十几 ppm 的那一阵」、13090「（上一造法）」、2151/2333「改的造法」「改过的造法，原因见本段末尾」。`comment-history-lag.tsv:11` 给这份记的存量是 5，与上面命中数对不对得上没核。
3. **`gc_wall_clock_of_the_run` 3793-3799**：`windows.iter().zip(records)` 而 `_record` 不用；`windows` 在 5111 是滤掉 `None` 之后的，与 `records` 对位不齐、按短的截；求和只用 `window.wall`，所以 zip 只起截断作用。有窗口读不到时整次已作废（5159），实际结果大概不受影响（推的）。
4. **钟速单测 10910**：前提不成立时打「本次未跑」后 `return`（10973-10980），cargo 判绿；`.claude/agent-common.md`「门禁」一节「退出码 77 是本次未跑，不是通过」管的是门禁，单测这样写算不算假装通过要判。变异 `crates/mutations.tsv` 「V87 钟速单测的前提不成立时判红」钉的是这一行为。
5. **长自旋单测 10815**：注释 10812-10813 自承「这一条分得出两种钟只在调频的那一阵，不是每次都分得出」，判别力依赖本机 NTP 状态。
6. **CPU 多判臂的活槽段**：`rejudged_batch_on_the_cpu` 3161-3170 对每块盘算一遍活槽数，只加 `entries_rejudged`、不比任何值；读不到单元字节记 `mismatches += 1` 而不记 `entries_rejudged`（3149-3151）。阳性对照只要它花 CPU，口径对不对照登记第五节「阳性对照」第 2 条核。
7. **judge-trend 8840-8924**：不核输入是不是恰三份、三档各一份；同一 `(against, tier)` 出现在几份产物里时 `insert` 覆盖（8878）；`values` 里解析不了的值被 `filter_map` 丢掉（8872）。
8. **`slots_exempt_from_the_mapping` 1729-1747**：注释说是核心层 `mounted_session.rs` 私有 `pinned_slots` 的「另写一份」，两份一致没有回比测试（只有小池用例间接走到）。
9. **下标依赖次序**：`LOCKSTEP_ARMS[0]` 当 CPU 臂（5327、7739、7791、10695），`[1]`、`[3]`、`[4]` 等在测试里；改 `LOCKSTEP_ARMS` 次序会静默换臂。
10. **`batches_whose_live_slot_entries_are_not_whole_device_passes` 3661** `entries % full_segments_of_a_device`：单元区不足一段时除数为 0 会 panic（三档不会，推的）。
11. **`rejudged_live_slots` 2958-2959** `segment.start_slot.0 - unit_area_start_slot` 无 `checked_sub`，前提是段在单元区内。

## 二、gpu_side_units_entries.rs（168 行）

### 1. 它做什么

文件头注释原话（`gpu_side_units_entries.rs:1-7`）：

```
//! 对拍里 GPU 一侧（判段持卡那一段：开卡、跑 GPU、与读回的参照比）的判法单元入口登记表：一条真卡对拍用例一组，组名就是用例名，
//! 入口是那条用例的持卡判 fn（`judged_on_the_cards_that_<用例名>`，在它自己的测试目标里；实验装置 E202（GPU与CPU两种模式GC对照组）那一条内联真卡测试的在那份 bin 里，目标写 `bin:<名>`）。闭包从它收：开卡与取卡、产品 GPU 包里它真走到的
//! Rust 与 WGSL、读回的输入与参照怎么解、怎么比。构建脚本（`build.rs`）经 `#[path]` 编同一份，照参照一侧那张表同一个算法逐组算键、写进
//! `OUT_DIR/gpu_side_units_keys.tsv`；测试按用例名读回，与参照一侧那一把、读回输入的摘要一起组成对拍结论的键
//! （`tests/common_pairing_input_through_the_library`）：GPU 一侧走的代码或比法改了，键变，库里旧的结论不再被复用。
//! 一条用例一组：同一份输入、同一个参照组的几条用例比法各不相同，各落在自己的键上。
//! 这份文件只放数据，只用 `std`：构建脚本单独编它（行的写法在 `src/reference_units_entries.rs`）。
```

一句概括：纯数据的常量表 `GPU_SIDE_UNITS_ENTRY_GROUPS`（12-168），31 组（`grep -c 'group_name:'` = 31），每条真卡对拍用例一组：组名、目标 `test:<测试目标>` 或 `bin:e202_gpu_cpu_gc_comparison`、一个持卡判入口 fn 路径；行类型借 `reference_units_entries::PairingSideUnitsEntryGroup`（9）。构建脚本经 `#[path]` 编它算键（`build.rs:8-9`、188）。

按目标分组（量的，`grep -c` 各测试文件里 `fn judged_on_the_cards_that_` 与表里条数一致）：`checksum_kernels_on_a_real_gpu…` 8（13-52）、`checksum_batches_over_two…` 2（53-62）、`…three…` 1（63-67）、`live_slot_kernels…` 6（68-97）、`region_record_kernels…` 6（98-127）、`compaction_on_a_real_gpu…` 6（128-157）、`mount_runtime…` 1（158-162）、`bin:e202_gpu_cpu_gc_comparison` 1（163-167）。

现核（一段 `python3 -I -` 直接从标准输入跑的脚本，读两张表、逐条在 `target` 指的那份文件里找 `fn <名>`，输出只有 `checked`；另用 `grep -rlE "fn <名>\b"` 数每个名字的定义文件数，都是 1）：39 个入口 fn（两张表合计）都恰在一份文件里定义，且都在各自 `target` 指的文件（或它 `::` 里点的模块）里，0 条对不上。

### 2. 编号

`gpu_side_units_entries.rs:2`：`… 实验装置 E202（GPU与CPU两种模式GC对照组）那一条内联真卡测试的在那份 bin 里，目标写 `bin:<名>`）。…`。去重：E202。D、C、I 起头的没有。

### 3. 检查单八格的原始事实

- 走不到的理由：没有 `unreachable!`、`panic!`、`expect`、`unwrap`、`assert!`（`grep -nE 'unreachable!|panic!|expect\(|unwrap\(|assert' ` 0 命中）。
- 类型与名字：`target` 用字符串前缀区分 `test:`/`bin:`（`reference_units_entries.rs:11` 的约定），入口用 `&'static str` 的路径；路径前缀一律写 `singlefs_background_gpu_tests::`，而 fn 实际在各测试目标与 bin 这些单独的 crate 里（如 166 行指向 bin 里 8666 的 fn）——这是按函数单元图的写法还是笔误，见第 5 样。没有 `_ =>`、`as`、缩写。
- 不兼容旧代码与旧格式：没有。
- 不新建判定库：自己不写；键表由构建脚本写进 `OUT_DIR/gpu_side_units_keys.tsv`（文件头 4 行），测试经 `include_str!` 读（`tests/common_pairing_input_through_the_library/mod.rs:39-40`）。
- 所有任务多机可用：不读任何运行时状态（纯常量）。
- 测试优先多线程：不适用（数据）。
- 带 --item 的命令与项：`crates/mutations.tsv` 第二段是这份的 2 行（7922「真卡对拍结论：GPU 一侧的组入口换成参照那一侧（闭包收不到产品 GPU 包的核）」、7923「真卡对拍结论：两条用例登记同一个持卡判入口（两组落到同一把 GPU 一侧的键）」）；`.claude/gate.d/*.tsv`、`scripts/crash/`、`replay.sh`、`scripts/mutation/tables/` 无命中。
- 新崩溃流多罩了什么：没有（纯数据表，查过同上范围）。

### 4. 测它的测试与变异

- `#[path]` 直接编它的测试目标：只有 `tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs:8-9`；里面 `every_gpu_side_group_is_keyed_and_closes_over_the_kernels_of_the_product_gpu_crate`（340-396 行，文件 5 个 `#[test]`）核：组名两两不同、每组算出了键、闭包里至少一个产品 GPU 包的 WGSL 入口、按目标该走到的核文件都走到、键两两不同。
- 经键表间接用到它的（按用例名取 GPU 一侧键）：`grep -rlE 'gpu_side_units_entries|GPU_SIDE_UNITS' crates/singlefs-background-gpu-tests/tests` 命中 9 份（checksum_kernels、region_record_kernels、common_pairing_input_through_the_library/mod.rs、checksum_batches_over_three、…_two、live_slot_kernels、pairing_conclusion…、compaction…、mount_runtime…）。
- 本文件无 `#[cfg(test)]`。变异 2 行（上一格）。

### 5. 线索，没核

1. 入口路径前缀全是 `singlefs_background_gpu_tests::`（库 crate 名），而那几个 fn 不在库里（`src/lib.rs` 是空的，见第四节），在各自的测试 crate 与 bin crate 里；`target` 一列另给编译目标。`singlefs-unit-graph` 认不认这种「库名前缀 + 另给目标」的写法、算错了会不会只报 keyed 以外的结论，没核（`pairing_conclusion…` 340 那一条判 `verdict != "keyed"` 会红，推的）。
2. bin 那一组（163-167）在 `pairing_conclusion…:313-333` 的 `kernel_files_every_gpu_side_group_of_the_target_reaches` 里落到 `_ => &[]`（330-331），只核「算出键、至少一个产品核」，不核走到丙段那两份核。

## 三、reference_units_entries.rs（73 行）

### 1. 它做什么

文件头注释原话（`reference_units_entries.rs:1-5`）：

```
//! 对拍里参照一侧（在 CPU 上算、只录那一段写进判定库）的判法单元入口登记表：一组一行，从哪个测试目标里的哪个 fn 收闭包。
//! 构建脚本（`build.rs`）经 `#[path]` 编同一份，经 `singlefs-unit-graph` 逐组算键、写进 `OUT_DIR`；测试按组名读回编进去的那一把
//! （`tests/common_pairing_input_through_the_library`），拼进对拍参照的录入键与对拍结论的键：参照的代码改了，录入键变，库里旧的参照不再被读到，
//! 旧的结论也不再被复用。GPU 一侧的入口登记在 `src/gpu_side_units_entries.rs`，两张表用这里的同一种行。
//! 这份文件只放数据，只用 `std`：构建脚本单独编它。
```

一句概括：定义两张登记表共用的行类型 `PairingSideUnitsEntryGroup`（8-15：`group_name`、`target`、`entry_functions`），参照一侧的常量表 `REFERENCE_UNITS_ENTRY_GROUPS`（19-70，7 组：校验和核 21、活槽核 28、区域记录核 35、整理那一趟 42（两个入口 45-46）、点名不在本机的卡的拒法 50、挂载运行库 57、E202 装置自数 64），与收闭包时交的特性名 `FEATURE_THE_PAIRING_SIDE_UNITS_ARE_KEYED_WITH = "tests-on-a-real-gpu"`（73）。构建脚本经 `#[path]` 编它（`build.rs:10-11`、89、184）。

### 2. 编号

`reference_units_entries.rs:18`：`/// 点名不在本机的卡那一条在 CPU 上写下的拒法，实验装置 E202（GPU与CPU两种模式GC对照组）里装置自数的每段活槽数。`。去重：E202。

### 3. 检查单八格的原始事实

- 走不到的理由：0 命中（同上一节那条 grep，三份小文件合起来退出 1）。
- 类型与名字：`target` 字符串约定 `test:<测试目标>` / `bin:<名>`（11）；入口路径 `singlefs_background_gpu_tests::<模块路径>::<fn>`（13），与第二节第 5 样同一个疑点；24 行带模块段 `common_checksum_batches_over_the_handed_over_gpus::`。特性名是字符串常量，与 `Cargo.toml:33` `tests-on-a-real-gpu = []` 靠人对齐。
- 不兼容旧代码与旧格式：没有。
- 不新建判定库：自己不写；构建脚本写 `OUT_DIR/reference_units_keys.tsv`（`build.rs:2`），测试读（`common_pairing_input_through_the_library/mod.rs:35-36`）。
- 所有任务多机可用：纯常量。
- 测试优先多线程：不适用。
- 带 --item 的命令与项：`crates/mutations.tsv` 0 行、`.claude/gate.d/*.tsv` 0、`scripts/mutation/tables/` 0、`scripts/crash/` 0（`…/registry-grep.txt` 里 `=== reference_units_entries` 一段只有 `rc=0`）。
- 新崩溃流多罩了什么：没有。

### 4. 测它的测试与变异

- `#[path]` 编它的测试目标只有 `pairing_conclusion…:14-15`，而且挂 `#[allow(dead_code, reason = "这里只读 GPU 一侧那张登记表，参照一侧那张与特性名不用")]`（10-13）：参照表本身没有一条测试逐组核。
- 按组名取参照键的：`grep -rlE 'reference_units_entries|REFERENCE_UNITS_ENTRY|reference_units_keys' …/tests` 命中 6 份（pairing_conclusion…、live_slot_kernels…、mount_runtime…、region_record_kernels…、compaction…、common_pairing_input_through_the_library/mod.rs）；另 `common_checksum_batches_over_the_handed_over_gpus/mod.rs` 在 GPU 一侧那条 grep 里命中。
- 本文件无 `#[cfg(test)]`；变异 0 行。

### 5. 线索，没核

1. 参照一侧没有 GPU 一侧那样的「组名各不相同、每组算出键、键两两不同」测试，也没有变异行；登记错一组入口（例如指到别的 fn）会不会有别处判红，没核。
2. 特性名 73 与 `Cargo.toml:33` 只靠人对齐；`build.rs:89` 用它收闭包。

## 四、lib.rs（4 行）

### 1. 它做什么

文件头注释原话（`lib.rs:1-2`）：

```
//! 产品的后台 GPU 包（`singlefs-background-gpu`）的集成测试包：测试目标都在 `tests/` 下，这个库本身是空的。
//! 为什么单独成包见 `Cargo.toml` 的 `description`。
```

一句概括：空库，只有 `#![deny(unsafe_code)]`（4）；不声明任何模块（两张登记表不经它编，经 `build.rs:8-11` 与 `pairing_conclusion…:8-15` 的 `#[path]`）。

### 2. 编号

没有（`grep -nE '\b[DECI][0-9]+'` 0 命中）。

### 3. 检查单八格的原始事实

走不到的理由、类型与名字、旧格式、判定库、多机、多线程、新崩溃流：都没有可列的（4 行里只有两行注释与一个 crate 属性）。带 --item：`crates/mutations.tsv` 第二段是它的 0 行，`.claude/gate.d/*.tsv`、`scripts/crash/`、`scripts/mutation/tables/` 0 命中。`unsafe` 规矩：`.claude/gate.d/unsafe-allowances.tsv:51` 给这个包登记的是 `forbid,deny` 与一份 tests 文件，lib.rs 的 `deny` 与它对不对得上没核。

### 4. 测它的测试与变异

无（空库；变异 0 行）。

### 5. 线索，没核

1. 注释说「测试目标都在 `tests/` 下」，而这个包另有 `src/bin/e202_gpu_cpu_gc_comparison.rs`（一个 bin，带 118 条同包测试），`Cargo.toml:6` 的 `description` 提到了它，lib.rs 这一句没有。
2. 两张登记表的入口路径写 `singlefs_background_gpu_tests::…`，就是这个空库的 crate 名（见第二节第 5 样）。

## 五、草稿目录里的原始输出

| 文件 | 是什么 |
|---|---|
| `diff.patch` | `git diff HEAD -- crates/singlefs-background-gpu-tests/src`，13476 行 |
| `e202-head-old.rs` | `git show HEAD:crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`，11570 行 |
| `e202-move.diff` | `diff e202-head-old.rs <现版本>`，2345 行 |
| `e202-outline.txt` | e202 的定义行大纲（`grep -nE` 取的，758 行） |
| `e202-calls-src.tsv`、`e202-expect-src.tsv`、`e202-calls-tests.tsv` | `tools/calls.py` 取的 expect/panic/assert（产品段、只 expect、测试段） |
| `e202-unreachable-*.txt` | 同一件事的 `grep -nE` 单行版 |
| `registry-grep.txt` | 第 3 样「带 --item」那条 grep 的原样输出 |
| `notes.md` | 读的时候记的段落笔记 |

## 要主 agent 跑的命令

没有（这件活只读，不要编译与测试）。

## 没做什么

- 没跑项目代码：没编、没跑同包测试与变异，没核 `singlefs-unit-graph` 怎么解 `singlefs_background_gpu_tests::` 前缀（第二节第 5 样）。
- 第 5 样各条都是线索，没核到结论；第 3 样里标「推的」的（`.expect` 按前提归类的计数、8682 那几处 panic 被收成判红）没逐行核共用模块。
- 没读 `research/prompts/e202-preregistration.md` 本文，注释里「登记第 N 节」与登记原文对不对得上没核。
- 没读包里 `build.rs`、`Cargo.toml`、`tests/` 的全文，只按需 grep 与读了几段（`pairing_conclusion…:5-25、313-396`，`common_pairing_input_through_the_library/mod.rs:35-40、297-310`）。
- 没改仓内文件，没提交，没往任务服务交任务；没建仓副本与编译目录。
