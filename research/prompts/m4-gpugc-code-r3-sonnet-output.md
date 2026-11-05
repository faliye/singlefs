# m4-gpugc-code-r3 正推腿（Sonnet）报告：代码今天做的是不是 D26 已定项 13 每一句说的那件事（2026-11-02）

立场：正推。从已定条款与冻结副本的代码推「应该是什么」，再逐格比。T1 主攻，T2、T3 也答。
冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3/repo/`；下面写 `crates/…:行号` 的都是它里面的路径与行号，kb 的行号取仓里 kb 文件自己的行号。
没起 cargo、没起 GPU、没往统一任务服务交任务；读到的数写「量的」（只读命令）或「推的」（读码推）。
模型目录 `research/prompts/m4-gpugc-code-r3-sonnet-model/`（只读事实探针 `tools/rerun.sh`，不起 cargo），sha256 在文末；腿在草稿目录里跑过一次，输出是 `out/facts.expected`（24 行），主 agent 复跑后逐行比。
冻结副本对 `research/prompts/m4-gpugc-code-r3-snapshot/crates-sha256.txt` 逐份对上：探针 `name=frozen_vs_snapshot_not_ok count=0`（量的，`out/facts.expected`）。

## 一、T1　已定项 13 第 2–4、8 条逐句

下面「钉」列写测试名（`文件::用例`，行号是 `fn` 那一行）或变异行名（`crates/mutations.tsv` 冻结副本里的行号）。「—」是没找到。
条款行号：第 2 条 246、第 3 条 247、第 4 条 248、第 8 条 252（`.claude/kb/decisions/26-后台整理与放置回收.md`，行号去 kb 文件里现取，每处引文逐字在那一行里 `grep -F` 过一次，量的）。

### 1.1　第 2 条（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）

| # | 条款一句 | 代码 | 钉 | 判 |
|---|---|---|---|---|
| 2a | 「点名了就只用点名的卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`） | 挂载：`crates/singlefs-background-gpu/src/background_gpu.rs:656` 按点名筛卡清单再校验；切换：`crates/singlefs-core/src/background_compute.rs:1048` 点名的次序即参与的次序 | `adapter_inventory.rs::named_gpus_participate_in_the_named_order_with_their_memory_limits`（627）；`gpus_of_this_host.rs::after_a_switch_to_fewer_cards_the_next_batch_is_split_only_over_the_named_cards`（961）；行 5554、5772、5777 | 一致 |
| 2b | 「不给就用本机全部独立显卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`） | `crates/singlefs-core/src/background_compute.rs:1048` 的 `named_devices` 为 `None` 时取清单全部 | `adapter_inventory.rs::without_named_gpus_every_discrete_gpu_participates_without_a_memory_limit`（681）；行 5494 | 一致 |
| 2c | 「集成显卡、虚拟显卡、软件 Vulkan 适配器与认不出类型的适配器不算」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）；点名它们照「点名的卡不存在」拒 | `crates/singlefs-background-gpu/src/adapter_inventory.rs:42` 只有独立显卡算；点名的不在清单里 ⇒ `NamedGpuDeviceNotInTheInventory` | `adapter_inventory.rs::only_the_discrete_gpu_counts_among_discrete_integrated_and_software_adapters`（383）、`virtual_and_unknown_adapters_do_not_count_as_background_gpus`（401）、`naming_a_gpu_that_is_not_a_discrete_gpu_of_this_host_refuses_the_mount`（560）；行 5548、5549 | 一致 |
| 2d | 「参与的卡都用，按挂载时在每张卡上量出的一次提交的主机 CPU 分活」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`） | 量：`crates/singlefs-background-gpu/src/background_gpu.rs:788`（开卡时量、线程 CPU 钟）；分：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:539` 按量出的代价分 | `work_split_by_measured_cost.rs` 的 9 条（98–185）；行 5556–5559、5576 | 一致。每批分到几条是比例份额：条数少于卡数时贵的卡分 0 条（`work_split_by_measured_cost.rs::fewer_entries_than_gpus_go_to_the_cheapest_gpus_first`，133），条款没要求每批每张都有活 |
| 2e | 「按第 7 条量出来赔的卡照用，并报出来」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`） | 「照用」：没有自动切回 CPU 的代码，对。「报出来」：`crates/singlefs-background-gpu/src/background_gpu.rs:1640` `participating_gpus()` 带名字、上限、量出的代价，**产品路径上没有调用者**（探针 `name=participating_gpus_callers_outside_tests count=0`，量的）；挂载结局 `OpenedBackgroundGpuCompute`（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:180`）只带没开成、被晾下、没收进清单、泄漏四样，量出的代价随具体类型一起被包进把手 | 每批「分到活的是哪几张卡」有 `devices_given_work`（`background_gpu.rs::every_card_of_a_batch_split_over_cards_on_the_cpu_is_given_work_and_submits` 3517）；量出的代价只有真卡用例读（`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:467`），无变异行 | **做到一半**：「照用」有，「赔」没有判法（代码里没有拿 GPU 与 CPU 比的量），「报出来」的代价字段到不了文件系统的用户。七·5 的对照组（`.claude/kb/milestone/04-fourth-txn.md:151`）才量「赔」，所以这一句至少该在 kb 里写明归七·5，或补一条出口让文件系统的用户读到。零轮 |

### 1.2　第 3 条（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）

| # | 条款一句 | 代码 | 钉 | 判 |
|---|---|---|---|---|
| 3a | 「按次序一张卡一个，与参与的卡逐个对应」，「不补齐」，「只给一个也报错」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | `crates/singlefs-core/src/background_compute.rs:1003` 个数不等拒 | `background_compute_settings_and_switch.rs::gpu_memory_limits_fewer_than_the_named_cards_are_refused`（213）、`…fewer_than_every_card_of_the_host_when_no_card_is_named_are_refused`（235）；行 5493 | 一致 |
| 3b | 某一个数超过显存总量拒（第 4 条列，`.claude/kb/decisions/26-后台整理与放置回收.md:248`）；等于总量收 | `crates/singlefs-core/src/background_compute.rs:1014` 严格大于才拒 | `background_compute_settings_and_switch.rs::gpu_memory_limit_above_the_total_memory_of_its_card_is_refused`（254）、`gpu_settings_naming_both_cards_with_limits_equal_to_their_total_memory_are_admitted`（313）；行 5495、5496 | 一致。泄漏按次累加之后这一句**不扣泄漏**：校验比的是上限对显存总量（静态），泄漏字节只扣可用余量（见 1.6 第 2 例） |
| 3c | 「这个数是上限、不是预留」，「任何时刻都不许比它多，缓冲、暂存区一起算在里面」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | 账：`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs` 的 `hold` 超上限 `assert!`（351–365）；一段占用含 2 份结果缓冲 + 2 份回读暂存（`background_gpu.rs:1242–1245` 的 `4 * outcome_bytes`，与 `CHECKSUM_CHUNK_FOOTPRINT` 在 1246 `assert_eq!` 对上）；泄漏的与等超时还在跑的提交都计入 `bytes_held` | `memory_limit_and_chunks.rs::holding_past_the_memory_limit_panics`（921）、`two_segments_held_on_one_gpu_together_stay_within_its_memory_limit`（810）、`a_chunk_whose_wait_timed_out_stays_held_until_its_submission_completes_on_the_gpu`（617）；行 5565、5584、5654–5657、6469 | **账里一致；「塞一个多分配一块缓冲的故障要红」没有测试也没有变异行**（七·3 验收，`.claude/kb/milestone/04-fourth-txn.md:149`）：账是代码里按字节数公式算的，没有一处数真建的缓冲；`name=mutation_rows_extra_unaccounted_buffer count=0`、`name=peak_measurement_lines count=0`（量的）。行 6469 是少建一半，方向相反。零轮 |
| 3d | 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | `crates/singlefs-background-gpu/src/background_gpu.rs:1025` 在只接显存不足的范围里建全部缓冲；`:1928` 的循环按建不成的一段的一半重切重交 | 替身：`background_gpu.rs::a_checksum_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu`（3019）；真卡：`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs::a_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu`（374，另一个设备占不住显存时打 `NOT_RUN_NO_GPU_ADAPTER` 返回成功）；行 6229、6230、6364 | 一致（替身层）。真卡那条在「驱动把建不成的挪去别的内存」的机器上整条不跑，行 6364 同样判「本次未跑」，收口时可能全程没真跑到（推的） |
| 3e | 「不给就不设上限」，「装不下不拒绝」，「就按上限自适应切成几片、缩小逐次交，挂着时单条装不下也切」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | `memory_limit_and_chunks.rs` 的 `pieces_cut_to_fit_on_the_gpu`；三段外壳都按此刻剩下的界切 | `memory_limit_and_chunks.rs::an_entry_larger_than_the_memory_limit_is_cut_into_pieces_that_each_fit`（977）、`without_a_memory_limit_a_batch_is_one_chunk`（733）；行 5899–5901、5922–5924 | 一致 |
| 3f | 「挂载那一刻量代价（第 2 条）的那一段同样切片，代价取各片之和」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | `crates/singlefs-background-gpu/src/gpus_of_this_host.rs` 的 `measured_cost_summed_over_the_pieces`（429）；开卡量代价的缓冲建不成时不重切，算这张卡没开成（`.claude/kb/decisions/26-后台整理与放置回收.md:248` 那一句「算那张卡没开成」的读法，第二轮判决 T1 第 6 条「不采」） | `gpus_of_this_host.rs::a_memory_limit_smaller_than_the_cost_probe_cuts_it_into_pieces_instead_of_refusing_the_mount`（1379）、`the_cost_of_a_cost_probe_cut_into_pieces_is_the_sum_over_the_pieces`（1432）；行 5916、5917 | 一致 |
| 3g | 「照开设备、不分活、报出来，挂载不因此拒绝」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:473` 判装不下（先扣进程里泄漏的）；`:757` 照开不量代价 | `gpus_of_this_host.rs::a_memory_limit_below_the_smallest_piece_leaves_the_gpu_out_without_refusing_the_mount`（1450）、`a_gpu_whose_room_after_the_memory_leaked_in_this_process_is_short_of_the_smallest_piece_is_left_out_at_the_mount`（1744）；行 5918、5919 | 一致 |

### 1.3　第 4 条（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）

| # | 条款一句 | 代码 | 钉 | 判 |
|---|---|---|---|---|
| 4a | 配错时拒：个数不等、「某一个数超过那张卡的显存总量、GPU 不在、驱动不对、点名的卡不存在」，阈值超过 100（选 cpu 时也判），「就拒绝挂载（或拒绝切换）并报错」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 阈值：`crates/singlefs-core/src/background_compute.rs:876`（在选后端之前判）；GPU 不在：`:887`；等 GPU 上限为 0：`:1025`；挂载那一刻开卡失败：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:757` ⇒ `GpuUnusable` | `compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs::a_gc_cpu_compensation_threshold_above_one_hundred_percent_is_refused_at_the_mount_and_at_a_switch`（729）；`background_compute_settings_and_switch.rs::gpu_without_a_gpu_inventory_is_refused_as_gpu_not_present`（166）、`…naming_a_card_missing_from_the_inventory_is_refused`（192）；行 5490–5495、5961、5962 | 一致。切换时「驱动不对」没有重开设备的代码：切换只读挂载那一刻开成/没开成的清单（`crates/singlefs-background-gpu/src/background_gpu.rs:2300`），点名挂载时没开成的卡拒（`ParticipatingGpuDeviceNotOpenedAtTheMount`），条款没写这一支，`.claude/kb/mount-parameters.md:23` 写了 |
| 4b | 「只重交超时的那一片，同一批已做完的片留着」，「至多 3 次，都超时才报警、判那张卡用不了」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 重交只在片的闭包里：`crates/singlefs-background-gpu/src/background_gpu.rs:413` `outcome_of_a_chunk` → `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:195`；第 4 次超时报：`:175` | `background_gpu.rs::a_checksum_batch_cut_into_four_chunks_each_timing_out_once_reports_the_card_unusable`（2811，交成 3 次 3 片：已做完的片没有再交）、`…of_one_piece_timing_out_three_times_then_completing_is_returned`（2848）；`waiting_for_the_gpu.rs::four_timeouts_report_the_gpu_unusable_after_exactly_three_resubmissions`（678）；行 5902、5903、6217、6220–6224 | 一致 |
| 4c | 「重交只在挂着时的批里做，挂载那一刻量代价的提交超时不重交，算那张卡没开成」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 量代价走 `both_kernel_outcomes_of_a_checksum_chunk` 直调，不套 `outcome_of_a_chunk`（`crates/singlefs-background-gpu/src/background_gpu.rs:788` 一带） | `gpus_of_this_host.rs::a_named_card_that_did_not_open_refuses_the_mount`（1286）、`an_unnamed_card_that_did_not_open_is_skipped_and_reported_while_the_named_cards_take_part`（1215）；行 5818–5820 | 一致 |
| 4d | 「三段各自计，不按整理的一批、也不按切出来的片」，「每段一次调用里同一张卡的超时跨片累计，交活之前先等前一次超时提交的那一等也算进去，累计到第 4 次等超过上限就报这张卡用不了」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 每段一次调用新建一份每卡计数（`crates/singlefs-background-gpu/src/background_gpu.rs:1892`）；交活之前那一等：`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:228`；只数真交出去的重交：`:213` | `waiting_for_the_gpu.rs::waits_for_the_earlier_submission_before_handing_over_count_toward_the_timeouts_of_the_batch`（594）、`two_waits_timing_out_before_the_resubmission_count_one_resubmission_actually_made`（740）、`three_waits_timing_out_before_any_resubmission_count_no_resubmission_and_report_the_gpu_unusable`（764）；`background_gpu.rs::a_checksum_batch_whose_wait_before_the_resubmission_times_out_twice_reports_one_resubmission`（3392）；行 6216–6224、6356–6358 | 一致。`UnusableAfterResubmissions.resubmissions` 恒写常数 3，即使一次都没真重交（`waiting_for_the_gpu.rs:175` 一带，第二轮判决「同时照旧报」）：该字段文档写「重交了 `resubmissions` 次」，真次数在批记账的 `resubmissions_after_timeouts` 里，报警里的字段与它可以不等，记在 1.5 |
| 4e | 「这一批不给这张卡分活，它那份转给这一批别的参与的卡，不报警」，「参与的卡这一批全都装不下才报」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 显存上限那一支：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:539`；空闲显存那一支：`crates/singlefs-background-gpu/src/background_gpu.rs:1941` 切不出 ⇒ `ShortOfFreeMemory` ⇒ 把那段压回 `:1905` 的栈重分 | `background_gpu.rs::a_checksum_batch_goes_to_the_other_card_when_one_card_is_short_of_free_memory_for_the_smallest_piece`（3109）、`…whose_cards_are_all_short_of_free_memory_reports_every_participating_gpu_short`（3147）、`a_card_short_of_free_memory_is_given_work_again_in_the_next_batch`（3189）；harness `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs::every_participating_gpu_short_of_free_memory_abandons_the_batch_and_leaves_the_disk_unchanged`（430）；行 6233、6234、6261–6270 | 一致 |
| 4f | 「建缓冲显存不足、切小重切时，这一段在这张卡上已做完的片不留，照新的切法整段从头重交」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`，2026-11-01 补的一句） | `crates/singlefs-background-gpu/src/background_gpu.rs:1928` 的循环：`Err(OutOfMemoryCreatingTheBuffers)` ⇒ `:1941` 重切 ⇒ 用整个 `planned.entries` 重新 `run_the_plan_on_a_card`，失败那一趟已做完的片的结果随 `Err` 丢掉（读码推，没量） | **没有**。替身卡的 `free_bytes` 只有 5 处赋值，都在 `background_gpu.rs`（探针 `name=free_bytes_assignments count=5`，量的），其中 3019 那条是八个等大的单元、第一段就建不成（交成 3 次 8 片：没有一片在重切之前做完），别的 4 条是空闲 10 字节。真卡那条同形（`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:430` 只断 `recuts >= 1`）。变异表里点名「不留」「整段从头」的行 0 条（探针 `name=mutation_rows_pinning_completed_pieces_redone count=0`，量的）。第二轮攻方探针 `free_memory_recut_redoes_completed_chunks`（交 8 次、片 8、重切 2）量到了这个行为，但它在攻方的模型里，不在入库的测试里 | **代码做了，没有测试钉**：把重切改成「只重切没做完的」，现有全部测试仍绿。这是 2026-11-01 补的两句之一。零轮 |
| 4g | 「那个后台任务停下，并在停着期间每次后台整理被叫起都报警」，「发布之后的查与定时兜底的查都算，补跑的与照旧停着的都报，显式的整理入口每一趟也算（入口在任何读写之前就拒的那一次不算）」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 显式入口：开头 `crates/singlefs-core/src/mounted_session.rs:2346`，收尾 `:2369`，中间 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 返回的是 `CompactionRun` 不是 `Result`，没有提前返回；后台入口：开头 `:2459`（在盘表、有没有树两判之后），收尾 `:2420`；攒批会话：没起跑 `:5581`、起跑后排空被拒 `:5611`、跑完 `:5632` 三处都取 | `compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs::every_wake_while_the_compaction_stays_stopped_after_a_gpu_failure_returns_an_alarm_until_a_switch_lifts_the_stop`（1483）、`the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_while_it_does_not_start`（1568）、`…when_it_runs_a_compensating_batch`（1614）、`…when_draining_the_pending_changes_is_refused`（1737）；行 6250–6253、6359 | 一致。两种入口开头到收尾之间我没找到不经收尾的出口（读码推）。排空被拒时「排空不成就不跑整理」照留：`:5596` 的 `Err` 分支先放回「该跑了」再 `return Err`，整理没起 |
| 4h | 「出错停下的那一趟不算，停下之后的下一次叫起是第 1 次，挂载那一刻就停着的头一次叫起是第 1 次，在叫起里停下与在可写挂载续做里停下数法相同」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`，2026-11-01 补的另一句） | `crates/singlefs-core/src/background_compute.rs:1386`（叫起在做时记 `is_stopped_in_the_wake_under_way`）、`:1440`（`:1455` 那一臂报 0 并关旗，别的臂 +1）；续做：`crates/singlefs-core/src/mount.rs:7660` 不调 `begin_a_wake`，停下后头一次叫起走 `None` 臂报 1 | `background_compute.rs::a_stop_in_a_wake_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three`（1786）、`a_stop_in_the_writable_mount_continuation_counts_the_next_three_wakes_one_two_three`（1813）、`a_stop_after_a_wake_has_ended_is_not_counted_as_stopped_in_that_wake`（1832）；harness `…::a_stop_in_a_wake_of_the_background_entry_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three`（1839）、`…::a_stop_in_the_writable_mount_continuation_is_followed_by_three_wakes_counting_one_two_three`（1919）、`compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs::the_first_wake_after_a_mount_with_no_usable_gpu_returns_the_alarm`（196）；行 6254、6255、6366–6370、6471 | 一致，三种停法各有钉。「显式入口那一种」的钉在 harness 1483 的帮手里 |
| 4i | 「通道是返回值里必须处理（`#[must_use]`）的报警，不是日志」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | `CompactionRun`、`BackgroundCompactionCall`、`BackgroundCompactionAfterThePendingChanges`、`AlarmOfABackgroundTaskStoppedAfterAComputeError` 都带 `#[must_use]`；排空被拒时报警在 `Err` 成员的 `Option` 字段里（`crates/singlefs-core/src/mounted_session.rs:5116` 所在的 `BackgroundCompactionAfterThePendingChangesRefused` 没带 `#[must_use]`） | 没有（编译期警告没法在运行时测） | 一致于第二轮判决 T2 第 3 条的读法（`#[must_use]` 只抓整份丢掉）。`Err` 成员里的那份报警同形：调用方 `match` 到 `Err(_)` 或 `is_err()` 编译器不吭声 |
| 4j | 「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | `crates/singlefs-background-gpu/src/background_gpu.rs:2261` 的 `Drop` → `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:866` `std::mem::forget`；泄漏了哪几张报给调用方、按次累加记进进程里的那一份（`gpus_of_this_host.rs:707`、`:717`） | `gpus_of_this_host.rs::dropping_the_gpus_leaks_the_one_with_a_running_submission_and_drops_the_others`（1530）；`background_gpu.rs::dropping_the_background_gpu_reports_the_card_leaked_and_notes_it_in_the_process`（3232）、`two_mounts_alive_at_once_leaking_on_one_gpu_add_up_through_the_drops_of_the_background_gpu`（3424）、`a_later_mount_leaking_again_on_the_same_gpu_adds_only_its_own_bytes_in_the_process`（3472）；行 5920、6225–6227、6235、6360–6362 | 一致。产品里没有持有那份进程账的调用者（`open_background_gpu_compute` 在 `crates/` 里的调用者都在 `singlefs-background-gpu` 包内与 `tests/`，量的 `grep -rn`），第二轮「记一笔」没有落到 kb 或欠账行（见 T2） |
| 4k | 「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下，从下一批起用新设置」，「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 校验过才解除：`crates/singlefs-core/src/background_compute.rs:1592`（在 `?` 之后）；停下那一刻：`:1386` 开头 `requested_from_the_next_batch.is_some()` 直接返回「当场解除」 | `compaction_off_the_calling_thread_and_admission.rs::a_switch_request_received_while_the_compaction_is_stopped_after_a_compute_error_lifts_the_stop`（1023）、`a_switch_pending_before_a_compute_error_lifts_the_stop_at_the_moment_of_the_stop`（1111）；行 5543、5544 | 一致 |
| 4l | 「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走，报的是挂载那一刻同一个成员」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 显式入口 `crates/singlefs-core/src/mounted_session.rs:3287` 先换上待生效的切换，再 `:3176` 判停着，删除续做 `:3319` 在它们后面；后台入口 `:2463` 同；成员 `GpuUnusableUnderTheSettingsHandedIn`（`crates/singlefs-core/src/background_compute.rs:1602` 换上时用同一个函数记） | `compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs::the_explicit_entry_after_a_switch_to_no_usable_gpu_stays_stopped_before_continuing_the_deletion_queue`（334）、`the_background_entry_after_a_switch_to_no_usable_gpu_stays_stopped_and_keeps_the_start_due`（373）；行 6001、6002、6074、6245、6246 | 一致 |
| 4m | 「GC 报警停着之后在每一批开头判一次占用」，「都不超过就照旧停着，补跑途中降到不超过，下一批起照旧停着」，「补跑只管 GC，打包搬迁停着时不补跑」，「不给阈值的切换沿用原阈值」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 每批开头：`crates/singlefs-core/src/background_compute.rs:1626`；逐盘比：`:731` 乘在 u128 里、最满的一块盘；打包：`:1524` 只有 GC 一臂，今天 `BackgroundTask` 只有一个成员，打包那一臂无代码、无测试；沿用原阈值：`:1556` | 同 1.3 4h 之外：`compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 的 1051、1298、1378、943、817；行 5958–5969、5996–5999、5963、5964、6248、6249、6472 | 一致；打包那一句今天没有对象，不算缺（加成员时 `:1524` 的 `match` 编不过，逼着写） |
| 4n | 「停着的后台任务不换到 CPU 上接着跑」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 没有这样的代码路径（`route_of_the_task` 只在没停着时选路） | 行 6472（补跑那一批照设置交给 GPU）；`compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs::a_compensating_run_on_the_cpu_while_stopped_after_a_gpu_failure_ends_exactly_as_the_cpu_mode`（2006） | 一致 |

### 1.4　第 8 条（`.claude/kb/decisions/26-后台整理与放置回收.md:252`）

这两批没改第 8 条的任何一行代码（改动全文里没有 `two_kernel_reconciliation.rs`、`checksum_kernels.rs`、`*.wgsl`），下面按今天的代码逐句对，钉列给名字。

| # | 条款一句 | 代码 | 钉 | 判 |
|---|---|---|---|---|
| 8a | 「两份独立写的 GPU 核（不共用核函数）对同一批输入各算一次，逐条比两份的结果」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | 三段各两份 WGSL（`checksum_kernel_bitwise.wgsl` / `checksum_kernel_table_driven.wgsl`；`live_slot_kernel_walking_every_record.wgsl` / `live_slot_kernel_searching_sorted_records.wgsl`；`region_record_kernel_walking_the_mapping.wgsl` / `region_record_kernel_searching_sorted_indexes.wgsl`），一次提交里各派发一遍（`crates/singlefs-background-gpu/src/background_gpu.rs:1051` `dispatch_both_kernels_and_read_back`） | `tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs::decisions_over_three_rounds_agree_with_the_fake_of_the_core_on_every_segment`（578）；行 5578–5581、5593–5597、5707–5713、5718–5721、5897 | 一致（WGSL 与真卡部分：腿没起 GPU，读码推；真卡用例延到收口） |
| 8b | 「CPU 照发布路径逐条核计划与读盘核」，「不另外抽样复核 GPU 的结果」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | 发布路径 `crates/singlefs-core/src/transaction.rs:10806` 调 `relocation_checked_against_the_version`；GPU 确认的条目走这条，没有别的抽样 | harness `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs::unit_checksum_mismatches_found_on_the_gpu_are_handed_to_the_cpu_publish_path_and_only_counted`（760）；行 5671、5672、5703、5705、5706 | 一致 |
| 8c | 「两次不同的条目写明状态不确定，这一批不回收、不搬，留到下一轮重算」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | `crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs` 的 `EntryDecision`；核心层不搬不确定的记录、不选不确定的段、不丢意图 | harness 同文件 567、617、661、707、815；行 5570、5661、5667、5668、5684、5685 | 一致 |
| 8d | 「同一个条目连续 2 轮（N = 2）都不确定，交 CPU 那一份实现单独判一次，按 CPU 的判定处理；连续不确定的计数只放在内存里，重挂清零」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | N：`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:14` = 2；计数住在每次开卡造的那份实现里（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:163`），重挂即新实现 | `two_kernel_reconciliation.rs::two_consecutive_uncertain_rounds_hand_the_entry_to_the_cpu_reference`（208）、`agreeing_again_after_a_disagreement_restarts_the_consecutive_count`（250）、`dropping_the_counts_forgets_every_uncertain_round`（289）；行 5567–5569、5715、5776 | 一致。「重挂清零」只有「丢掉计数就清空」的单测，没有「重挂一次整条路径」的用例（推的：新实现新计数，结构上成立） |
| 8e | 「一批放弃（GPU 出错、等超过上限）时，那一批里已经交回的段照算一轮，放弃的那一段计数不动」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | 放弃的那一段 `uncertainty` 不动（`background_gpu.rs` 三段各自 `reconciled_*`，出错时不进比对） | `waiting_for_the_gpu.rs::a_batch_whose_wait_times_out_leaves_the_consecutive_uncertainty_counts_unchanged`（373）、`tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs::a_batch_that_fails_on_the_gpu_or_times_out_is_reported_as_gpu_failed_and_leaves_the_consecutive_counts`（632）；行 5583 | 「放弃的那一段不动」有钉。「已经交回的段照算一轮」有没有单独断言，我读 632 的开头没读到（推的，没读完整条用例）；列入 1.5 的「待主 agent 核」 |
| 8f | 「每一批报确认、不确定、交 CPU 判的条目各多少，运行时看得出来」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | `crates/singlefs-core/src/background_compute.rs:560` 一带的 `GpuWorkOfTheBatch`（三个数）与 `submissions_on_the_gpus`（片、重交、重切、分到活的卡） | harness `the_gpu_path_through_a_cpu_backed_implementation_ends_a_compaction_run_exactly_as_the_cpu_path`（243，断言 `confirmed_by_both_kernels > 0` 与三段都交过）；`tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs::the_submissions_on_the_gpus_of_each_batch_are_recorded_in_the_accounting_of_the_batch`（1376）；行 5677、5678 | 一致（七·3 验收「三个数钉绝对值」：不确定、交 CPU 判的绝对值在 567–815 那几条里钉） |
| 8g | 「只有一张卡时两份核都在这张卡上跑，有几张卡时按第 2 条分活、两份核可以分开跑；不许要求多张卡，一张与几张都要能跑、判定相同」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`） | 两份核恒在同一次提交、同一张卡上（`background_gpu.rs:1051`）；分活按卡不按核；一张与几张都有路径 | 替身：`background_gpu.rs::a_batch_over_one_card_on_the_cpu_gives_it_work_and_it_submits`（3559）、`…over_two_cards…`（3564）、`…over_three_cards…`（3569）；行 6473–6475；真卡一/二/三张：第 60 批新写，只写没跑（延到收口） | 一致（「两份核可以分开跑」是「可以」，代码选了不分开，不违约） |

### 1.5　条款写了而代码做不到，或做了没有测试钉的句子（单列）

| # | 句子 | 缺什么 | 证据 | 新旧 |
|---|---|---|---|---|
| ① | 第 4 条 2026-11-01 补的「建缓冲显存不足、切小重切时，这一段在这张卡上已做完的片不留，照新的切法整段从头重交」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 代码做了，**没有一条测试、变异行钉「不留」**：现有全部用例里没有「某一片先做完、后一片才建不成」 | 1.3 4f；探针 `free_bytes_assignments`、`mutation_rows_pinning_completed_pieces_redone`（`out/facts.expected`） | 零轮 |
| ② | 第 2 条「按第 7 条量出来赔的卡照用，并报出来」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`） | 「报出来」的量出的代价到不了调用方（`participating_gpus()` 产品路径零调用者）；「赔」没有判法 | 1.1 2e | 零轮 |
| ③ | 七·3 验收「每张卡上 GC 占的显存峰值（攒批、缓冲、暂存区一起算）不超过它的上限，钉绝对值的断言，塞一个多分配一块缓冲的故障要红」（`.claude/kb/milestone/04-fourth-txn.md:149`） | 没有「多分配一块缓冲要红」的测试或变异行；账是公式，没人数真建的缓冲 | 1.2 3c | 零轮 |
| ④ | 第 3 条「挂着切换给了它装得下的上限，从下一批起照新上限重新量代价、分活」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | 重新量代价只对「从来没量过」的卡做：`crates/singlefs-background-gpu/src/background_gpu.rs:1697` 的 `if card.measured_cost().is_none() {`（探针 `name=remeasure_site 1697`）。开卡时量过、之后被某次切换晾下、再被另一次切换请回的卡，拿的是挂载那一刻在旧上限下量的代价，不重量 | 读码（推的，没起测试）；`.claude/kb/mount-parameters.md:24` 写的是「被晾下的卡切换时给了装得下的上限，下一批起重新量代价、参与」，同一个读法 | 零轮。条款那一句的「它」若只指挂载时就被晾下的卡，代码一致；若指任何被晾下的卡，代码不一致，要用户读一遍 |
| ⑤ | 第 4 条 `GpuUnusableAfterResubmissions` 点名的「重交 3 次」 | 报警里 `resubmissions` 恒为常数 3（第二轮判决 T1 第 2 条定「同时照旧报」），一次都没真重交时也写 3；真次数在批记账 `resubmissions_after_timeouts` | `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:175` 一带；`crates/singlefs-core/src/background_compute.rs:207` 的成员文档写「重交了 `resubmissions` 次、次次等超过上限」 | 第二轮已判留；kb 与欠账里无 |
| ⑥ | 第 8 条「已经交回的段照算一轮」 | 我没读到单独断言（1.4 8e） | 1.4 | 待主 agent 核 |
| ⑦ | 七·3 验收「数据量造到每张参与的卡都分到活、各自切片交过」（`.claude/kb/milestone/04-fourth-txn.md:149`） | 第 60 批把「各自切片交过」读成「每张卡分到一段、交过一次」；「每张卡上那一份又被 1 GiB 切成几片」要每张卡造 1 GiB 以上的数据，没造（第 60 批报告「我替条款做的选择」第 3 条第 1 小点） | `~/.local/share/singlefs/work/m4-gpugc-batch60-writer/result.md` | 读法，主 agent 记录里写「照收」，没问用户 |

### 1.6　两批的改法有没有把别的已定句子改坏

两例点名：

1. **只现枚举一次之后，挂着时卡掉了还能不能按第 4 条拒切换**：能。`crates/singlefs-background-gpu/src/background_gpu.rs:2289` 的 `devices_with_total_memory` 现枚举一次并过滤掉掉了的卡，`:2300` 的 `devices_not_opened_at_the_mount` 答挂载那一刻定下的清单。三种情形：
   - 开成过的卡掉了：不在清单里 ⇒ 不点名切换时 `GpuNotPresent`（全掉光）、点名它 `NamedGpuDeviceNotInTheInventory`（钉：`background_gpu.rs::a_switch_after_the_card_dropped_off_the_host_is_refused_as_gpu_not_present`，3265；行 6228、6355）。
   - 挂载时没开成的卡，切换点它：还在 ⇒ `ParticipatingGpuDeviceNotOpenedAtTheMount`；掉了 ⇒ 不在清单里，落成 `NamedGpuDeviceNotInTheInventory`。两种都拒，只是成员不同（钉：`…::a_switch_naming_a_card_not_opened_at_the_mount_is_refused_on_a_single_enumeration_though_a_repeated_one_would_come_back_empty`，3329，只钉「还在」那一种且枚举恰好 1 次）。
   - 枚举超时：当一张都没有，拒（`:725`），选 cpu 的切换不读清单。
   没有改坏第 4 条这一句。代价（读法）：挂载时没开成的卡修好之后，切换永远点不了它，要重挂（`.claude/kb/mount-parameters.md:23` 已写「参与的卡挂载那一刻没开成，拒绝挂载或切换」）。
2. **泄漏按次累加之后，第 3 条「上限不超过那张卡的显存总量」怎么扣**：不扣。`crates/singlefs-core/src/background_compute.rs:1014` 比的是「给的上限 > 显存总量」，不看泄漏；泄漏的字节只进这一次挂载这张卡的账（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:757` 开卡时 `gpu_left_out_under_its_memory_limit` 先扣进程里已泄漏的，`memory_limit_and_chunks.rs` 的账从它起）。后果两种：泄漏的字节吃掉余量 ⇒ 该卡「连最小的一片都装不下」⇒ 不用、报出来（`gpus_of_this_host.rs::a_gpu_whose_room_after_the_memory_leaked_in_this_process_is_short_of_the_smallest_piece_is_left_out_at_the_mount`，1744）；泄漏累加超过物理显存时账会比物理多，但物理上建缓冲会显存不足，走第 3 条「空闲不到上限也照常跑、只用空闲的」那一支。累加是 `+=` 在 u64 上（`gpus_of_this_host.rs:717`），溢出不可能到达（推的）。没有改坏。

别的句子，读码没有找到被改坏的：
- 第 59 批第 3 条（排空待发被拒带报警、计叫起）：「排空不成就不跑整理」照留（`crates/singlefs-core/src/mounted_session.rs:5596` 之后 `return Err`，整理没起；`gc_ring_interval_proofs.rs` 那道门的前提保住）。
- 第 59 批第 9 条（出错停下那一趟不算）：续做、叫起里、挂载就停着、切换生效就停着四种停法的头一次叫起都报 1（1.3 4h）。
- 第 60 批第 1 条（回读暂存换泛型函数）：行为不变，三段都经 `background_gpu.rs:1025` 一处（`grep -n 'buffers_created_within_the_free_memory' background_gpu.rs` 命中 1025（定义）、1225（注释）、1267、1387、1512（三段各一处调用），量的）。
- 第 60 批第 2 条（`mount.rs` 加入口）：只加一个函数（`crates/singlefs-core/src/mount.rs:9019`），C921 已补登记。

## 二、T2　未定项清单的完整性

范围：第一轮判决（`research/prompts/m4-gpugc-code-r1-main-verification.md`）、第二轮判决（`research/prompts/m4-gpugc-code-r2-main-verification.md`）、第 59 批报告（`~/.local/share/singlefs/work/m4-gpugc-code-r2-fix-writer/result.md`）、第 60 批报告（`~/.local/share/singlefs/work/m4-gpugc-batch60-writer/result.md`）里写成「读法」「照收」「交用户」「零轮」「核不动」「只抽了一次样」的每一条。去向四档：用户定 / kb 写回 / 欠账行 / 没有去向。「没有去向」= 只在判决或批次记录里出现，kb、欠账行、用户弹窗三处都没有。

### 2.1　清单

| # | 条 | 出处 | 去向 | 证据 |
|---|---|---|---|---|
| 1 | 点名虚拟显卡与认不出类型的适配器照「不存在」拒（代码比条款严） | 第一轮 T1、第五节第 3 行 | 用户定（2026-10-30）＋kb 写回 | `.claude/kb/decisions/26-后台整理与放置回收.md:246`、`.claude/kb/mount-parameters.md:23` 已含 |
| 2 | 「连续 2 轮」的「轮」按落点认、放弃时已交回的段先算一轮 | 第一轮 T2、第五节第 5 行 | 用户定＋kb 写回 | `.claude/kb/decisions/26-后台整理与放置回收.md:252` 已含 |
| 3 | 跨批攒没有实现 | 第一轮 T3、第五节第 1 行 | 用户定（删条款）＋kb 写回，**写回漏一处** | D26 第 247 行已无「攒」；`.claude/kb/milestone/04-fourth-txn.md:149` 验收仍写「（攒批、缓冲、暂存区一起算）」，第一轮第三节第 9 条要删「显存账里的『攒批』字样」（探针 `name=milestone_line_149_has_batching_word count=1`，量的） |
| 4 | 超时之后重交什么（单元） | 第一轮第五节第 2 行 | 用户定＋kb 写回 | `.claude/kb/decisions/26-后台整理与放置回收.md:248` 「只重交超时的那一片，同一批已做完的片留着」 |
| 5 | 只供测试的开关在发布构建里够得着 | 第一轮 T4、第五节第 4 行；第 60 批停下第 1 条 | 欠账行 | `.claude/kb/checks-owed.md:576` C921，已点名 `…the_gc_cpu_compensation_threshold_and_compaction_switches` 入口 |
| 6 | 整理一趟里删除续做的步数不进发布预算 | 第一轮 T4 | 欠账行（**问题没交用户**） | `.claude/kb/checks-owed.md:595` C942 的「还账判法」是「先定删除续做的步数算不算进……一趟发布预算有上限」，没有弹窗；行号 `mounted_session.rs 第 3238 行`、`第 3353 行` 已不指向原句（探针 `c942_anchor`：3238 是 `devices: &mut …` 参数行，3353 是 `}`；今天是 3319、3434） |
| 7 | 共用的后台 GPU 把手在交设置与读失败之间能被插队 | 第一轮 T2 | 欠账行 | `.claude/kb/checks-owed.md:596` C943；行号 683、698、366、3676 已不指向原句（探针 `c943_anchor`） |
| 8 | 「账超上限是 `assert!`」不改 | 第一轮 T3 末条 | **没有去向**（判决里「不改」，无 kb、无欠账） | `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:351` 的 `hold` 仍是 `assert!`，守的是公式算出来的账，见 1.2 3c |
| 9 | 第一轮「这一轮没打中，只记没打中」：攻方三种形状造不出（第二轮 T1 第 9 条重复「每种只抽了一次样」） | 第一轮 T2 首条、第二轮 T1 第 9 条、第四节 | **没有去向**：第三轮是最终轮，没有再抽一次样的安排 | 两份判决都只写「不当没有问题用」 |
| 10 | `region_scan.rs`、`checksum.rs` 两处快写法：攻方没攻，只有读码样本（第一轮正推、第二轮辩方各读一次） | 第一轮第四节末条、第二轮 T3 第 5 条「记：第三轮或七·4 覆盖时补一份样本」 | **没有去向**：这一轮正文的格不含它们，`.claude/kb/milestone/04-fourth-txn.md:150` 七·4 一行没写 | 三轮下来攻方样本 0 份 |
| 11 | 压一半重切时已做完的片留不留 | 第二轮 T1 第 3 条、第五节第 1 行 | 用户定（2026-11-01，records 第 689 行「重新切 不留，不然谁知道对不对」）＋kb 写回，**测试没跟上** | 条款在 D26 第 248 行；测试见 1.3 4f |
| 12 | 出错停下那一趟算不算叫起 | 第二轮 T2 第 2 条、第五节第 2 行 | 用户定（「改代码」，records 第 689 行）＋kb 写回＋代码测试都有 | 1.3 4h |
| 13 | 接手的卡把同一种建不成再撞一次（不采） | 第二轮 T1 第 4 条 | 没有去向（判成不违约） | 探针 `card_taking_over_hits_the_same_shortage_again`（攻方模型，不在入库测试里） |
| 14 | 量代价那一段建不成不按更小的片重切，算没开成（不采，「是读法」） | 第二轮 T1 第 6 条 | 没有去向 | D26 第 247 行「同样切片」与第 248 行「算那张卡没开成」两句并读 |
| 15 | `#[must_use]` 只抓整份丢掉，接住不读报警字段编译器不管（不采） | 第二轮 T2 第 3 条 | 没有去向 | 1.3 4i：`Err` 成员里那一份同形 |
| 16 | 第 54、55 批其余十一项「只是读法」「照收」（只有辩方一份样本） | 第二轮 T2 第 4 条、第四节末条 | 没有去向（部分已被后来的用户定案吸收：见本表 11、12） | 辩方报告第四节 |
| 17 | 第 54 批第 5 项：进程里泄漏的那一份由调用方持有，产品里持有它的挂载工具今天没有，「记一笔」 | records 第 625 行 | **没有去向**：kb、欠账行都找不到那一笔（`grep -n '进程里泄漏' .claude/kb/checks-owed.md` 零命中，量的） | `open_background_gpu_compute` 在 `crates/` 里的调用者只有 `singlefs-background-gpu` 包内与 `tests/` |
| 18 | 回读暂存换替身造测试（用户「造不出就换方法继续造」） | 第二轮第四之一、第 60 批第 1 条 | 用户定（records 第 696 行）；替身规矩「没在真卡上对过」写在第 60 批报告，kb 与欠账行无 | 3.3 |
| 19 | 可写挂载续做里停下的 harness 造不出，补 `mount.rs` 入口 | 第 59 批停下第 2 条 | 用户定（records 第 696 行「下一批补 mount.rs 入口」）；入口已加，C921 已点名 | `crates/singlefs-core/src/mount.rs:9019` |
| 20 | 续做里停下那一趟不交报警（报警字段恒 `None`），条款没写续做报不报 | 第 60 批停下第 4 条 | **没有去向** | 读法照收（records 第 707 行「替条款做的读法照收」）；D26 第 248 行只写「在叫起里停下与在可写挂载续做里停下数法相同」 |
| 21 | 「各自切片交过」读成每张卡分到一段、交过一次；「没有卡拒挂」靠藏 Vulkan 驱动重起测试程序（推的，没试）；「挂着出错」靠把等的上限切到 1 纳秒 | 第 60 批「我替条款做的选择」第 3 条 | **没有去向**：用户 2026-11-01 的原话是「每张卡都能用来分片gc」（records 第 695 行），第 60 批读窄了，没回头问 | 1.5 ⑦ |
| 22 | 新共用测试模块 `common_discrete_gpus_of_this_host/mod.rs` 不在派发提示的文件单里 | 第 60 批停下第 2 条 | 没有去向（已合入，无人表态） | 派发闸「撞文件」查不到它 |
| 23 | 三张卡那一格只在有三张独显的机器上真跑，测试不绑台，收口时看它在哪一趟里不是「本次未跑」 | 第 60 批停下第 3 条 | 用户定「三张卡那一格在第二台上跑」（`.claude/kb/milestone/04-fourth-txn.md:149`）；不绑台由服务挑台 | 收口时看 `NOT_RUN_NO_GPU_ADAPTER` |
| 24 | 变异表整行替换多了两行（原第 5992、6347 行） | 第 59 批停下第 3 条 | 主 agent 打补丁时收下 | 行 5904、6235（第二轮判决里的 5787、5788、5992、6347 是那一轮冻结副本的行号，今天已移位） |
| 25 | 真卡与 WGSL 部分、真卡变异行「核不动」 | 第一轮第四节、第二轮第四节 | 用户定延到七·3 收口（records 第 693 行「收口的时候在跑」、第 698 行；`.claude/kb/milestone/04-fourth-txn.md:149`） | 要真卡的变异行：第一轮 29 行 + 第 59 批 2 行 + 第 60 批 7 行 = 38，探针 `name=real_gpu_mutation_rows count=38` 对得上 |
| 26 | 第一轮「只在腿的副本上量过、主 agent 没复跑」：正推「抽样重跑 7 行变异都红」「6104 与 6063 红在哪一行」 | 第一轮第四节首条 | 6063 当线索并进第 10 条（已做：行 6248、6249）；别的没有去向 | 3.2 |
| 27 | 第二轮辩方探针没跑成（任务 2420 被派到第二台，退 127），「它要量的由攻方探针同一处量到」 | 第二轮第一节、第四节 | 没有去向（判决写明不重交） | 3.2 |
| 28 | mount-parameters 表里的代码落点行号（第一轮第三节第 9 条「落点行号现查重写」） | 第一轮第三节第 9 条 | kb 写回过一次，**这一轮前又漂了** | 探针 `mount_parameters_anchor`：`background_compute.rs:640/642/644/648/668` 今天依次是 `background_compute: BackgroundComputeBackend::Cpu,`、`background_gpu_memory_limit: None,`、`};`、一行注释、一行注释，而字段在 626/628/630/634，阈值结构在 654。没有门禁盯这些行号 |

### 2.2　七·3 那一行验收里还有哪一句没有实现或没有测试（不算延到收口的真卡用例）

逐句对（`.claude/kb/milestone/04-fourth-txn.md:149`）。没列的句子都有实现与测试，名字在第一节各表里。

| # | 验收一句 | 实现 | 测试 | 缺什么 |
|---|---|---|---|---|
| a | 「每张卡上 GC 占的显存峰值（攒批、缓冲、暂存区一起算）不超过它的上限，钉绝对值的断言，塞一个多分配一块缓冲的故障要红」 | 账有 | 账的算术有（`memory_limit_and_chunks.rs` 一串）；「多分配一块缓冲」没有 | **测试缺**，1.2 3c |
| b | 「数据量造到每张参与的卡都分到活、各自切片交过（三张卡那一格在第二台上跑）」 | 实现不需要新代码 | 只写没跑（延到收口）；写的是「分到一段、交过一次」 | **读法弱于条文**，1.5 ⑦。收口时全绿也钉不到「每张卡自己的那一份被 1 GiB 上限切成几片」 |
| c | 「模型对拍 gpu 模式与 cpu 模式读到的内容逐项相同」 | — | 有的是「发的每一版、两条录制流逐步相同（盘上逐字节相同）」（harness `the_gpu_path_through_a_cpu_backed_implementation_ends_a_compaction_run_exactly_as_the_cpu_path`，243）；`compaction_while_mounted_in_random_histories.rs` 没有 gpu 臂（`grep -n -i 'gpu' ` 零命中，量的） | **没有参照模型（RefFS）读内容的对拍**。字节相同蕴含读到的相同，这是读法；七·4 一行「两种模式写出的块序列逐字节相同」（`.claude/kb/milestone/04-fourth-txn.md:150`）也在做同一件事 |
| d | 「赔的卡照用、报出来」 | 照用有 | 无 | **报不出来**，1.1 2e |
| e | 「卡上空闲显存不到上限时照常跑、只用空闲的」 | 有 | 替身有；真卡那条在「驱动把建不成的挪去别的内存」的机器上整条打「本次未跑」 | 真卡一侧可能收口时也没真跑到（推的） |
| f | 「两条路径在同一组输入上逐项相同、对拍进快档」 | 有 | 对拍有；「进快档」我没核哪档（第 60 批合入后验证 4517 里「3229 条测试」含这包，量的：任务 4517 日志） | 档位没核 |
| g | 「分支能被测试强制进入、运行时看得出走了哪条」（`.claude/kb/milestone/04-fourth-txn.md:149`；对应 `.claude/rules/fs-design.md` 的五条硬要求） | 大部分有（`devices_given_work`、重切数、重交数、三个计数） | 同 | 量出的代价（2e）；报警里 `resubmissions` 常数（1.5 ⑤） |

### 2.3　`.claude/kb/checks-owed.md` 里点到七·3 或后台 GPU 的行：七·3 收口前关 / 留到七·4、七·5

| 行 | 题 | 去向栏现状 | 建议 |
|---|---|---|---|
| C906（第 562 行） | 整理批不过常规准入与条款对不上 | 「代码合入、整理批把可用压到需求之下时准入拒的那条用例绿了再还」 | **七·3 收口前关**：代码在（行 5540–5542），用例在（`compaction_off_the_calling_thread_and_admission.rs::a_compaction_batch_that_would_push_the_available_below_the_demand_is_refused_by_the_admission_and_writes_nothing`，921）；第 59、60 批合入后验证 4517 绿。关之前主 agent 点名跑这一条（推的，没量） |
| C921（第 576 行） | 只供测试的开关在发布构建里够得着 | 去向是别的批（core 直验第一批） | **留**：七·3 又加了一个入口，条数在涨；不在七·3 验收里。收口时该行点名的开关表要再核一遍 |
| C923（第 578 行） | 后台 GPU 包开卡编管线不在等的上限之内 | 「未排进里程碑，等主 agent 排」 | **七·3 收口前要表态**：它让第 4 条「等超过上限就重交/报警」在挂载那一刻有一个洞（驱动编管线卡住时挂载跟着卡住）；要么并进七·3 的最后一批，要么写明留到七·4 |
| C926（第 580 行） | 后台 GPU 等的上限默认 10 秒没量过 | 「随 D26 已定项 13 第 7 条对照实验一起量」 | **留七·5**（对照组量 GPU 往返时） |
| C927（第 581 行） | 整理水位字段与起跑三量对不上 | 「等主 agent 排、交用户定」 | 与 GPU 无关（D26 已定项 1），但七·3 的定时兜底与起跑判据可能读它（推的，没读码核）：**交用户，不挡七·3 收口**；留到七·4 之前定即可 |
| C942（第 595 行） | 删除续做的步数不进发布预算 | 「先定……」 | **七·3 收口前交用户**：验收有「一趟发布预算有上限」的用例，它的口径含不含删除续做没人定 |
| C943（第 596 行） | 共用把手能被插队 | 「先定把手允不允许几处共用」 | **留**：产品路径一个把手一个会话；条款没写，七·4 之后再说。收口前至少把行号写对 |
| 缺的行（建议新开） | 1.5 ① 重切「不留」没测试；② 报出来的代价到不了用户；③ 多分配一块缓冲没有故障用例；④ 晾下又请回的卡不重量代价；⑦ 各自切片读窄；2.1 的 17、20、28 | — | 七·3 收口前该有行，或各自补测试 |

## 三、T3　前两轮报出的数字能不能核

核法：看产生它的命令、产物、能不能照原样重跑。task 号的日志用 `task-server/task log <号>`（只读）看，任务命令用 `task-server/task status <号>` 看（只读）。下面「在」「不在」都是这一轮现查的。

### 3.1　前两轮判决里的数

| # | 数 | 产生它的命令 | 产物 | 能不能原样重跑 | 谁量的 |
|---|---|---|---|---|---|
| 1 | 第一轮「32 份里倒推出 31 份」 | `snapshot-tree.py m4-gpugc-code-r1 --out <临时>/m4-gpugc-code-r1-tree` | 开工快照 `research/prompts/m4-gpugc-code-r1-snapshot/sha256sums.txt` 在（32 行，量的 `wc -l`）；倒推出的树在会话临时目录，**不在**（量的 `ls` 该目录：No such file） | 命令在；倒推靠当年的暂存区，之后 HEAD 已提交（`59ce5a865`），倒推基底变了，能不能得出同一个 31，推的、没跑 | 主 agent |
| 2 | 第一轮 cite-check「53 处引文、239 处裸行号，对上 50、没判 242」「裸行号没有引文可比 212」 | `cite-check.py <两份腿报告> --root <倒推的树> --background …` | 两份腿报告与背景材料在；`--root` 那棵树不在 | 要先重建树（同 1）；报告与背景在，引文数本身是确定的 | 主 agent |
| 3 | 第二轮 cite-check「129 处引文、48 处裸行号，对上 128，没判 49」 | 同上，`--root ~/.local/share/singlefs/work/m4-gpugc-code-r2/repo` | 冻结副本 `m4-gpugc-code-r2/repo` **不在**（量的 `ls`，探针 `name=old_frozen_repo round=m4-gpugc-code-r2 exists=no`） | 不能原样 | 主 agent |
| 4 | 第一轮攻方复跑：六件 `compare-total differing=0`；`119 passed`、`117 passed; 2 failed`、`117 passed; 1 failed`、`9 passed` | `run-with-memory-cap.sh 16G bash <opus-model>/tools/rerun.sh --repo … --frozen …/m4-gpugc-code-r1/repo --work …/rerun --model …` | `~/.local/share/singlefs/work/m4-gpugc-code-r1-main-check/rerun.log` 在（含 `name=compare-total differing=0`）；`rerun/outputs/*.out` 在，`grep -h 'test result'` 取到这四个数（量的）；`opus-model/` 在（`research/prompts/m4-gpugc-code-r1-opus-model` 也在） | `--frozen` 的 `m4-gpugc-code-r1/repo` **不在**；产物里的数可读，原命令不能重跑，要先重建 R1 冻结树 | 主 agent 复跑过（这些是主 agent 的数） |
| 5 | 第一轮正推变异「永远取第一块盘」`9 passed` | 在复跑树上把 `background_compute.rs` 第 566 行改成 `Some(fullest) => Some(fullest),` 跑 `cargo test -p singlefs-harness --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure` | `variant-a.out` 在（末行 `test result: ok. 9 passed`，量的）；改坏后的源文件没留（只留文字描述） | 改法是一行文字，在今天的树上对应变异行 6249；第 566 行那一句已移位 | 主 agent |
| 6 | 第一轮正推「抽样重跑 7 行变异都红」「6104 与 6063 红在哪一行」 | 正推腿的副本 | 主 agent 判决写明**没有单独复跑**（第四节首条） | 腿的副本已无 | **只在腿的副本上量过** |
| 7 | 第一轮「`grep -c -E 'unlink\|deleted_inodes\|publish_unlink' …` 输出 0」 | 该 grep | 文件在，今天同一命令输出 **11**（量的，`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs`） | 原样重跑得 11，不是 0：因为后来加了删除队列用例（这是改了的结果，不是数错） | 主 agent |
| 8 | 第一轮「`grep -c -F 'if !queue_is_empty {' research/prompts/_m4-gpugc-code-r1-diff.md` 输出 0」 | 该 grep | 文件在，今天输出 0（量的） | 能，同一个数 | 主 agent |
| 9 | 第一轮「乙段单元切片三行是第 6121–6123 行」「6131、6137、6138 是 C773 的」 | `sed -n 6121,6123p crates/mutations.tsv` | 变异表在，今天「乙段单元切片」起头的行在 6007–6019、C773 的在 5106–5108（量的 `grep -n`） | **行号漂了**：`crates/mutations.tsv` 别的会话改它，行号不稳；名字稳。判决里凡写变异表行号的数都只在写那一刻对 | 主 agent |
| 10 | 第一轮「29 行真卡变异行」 | 数第五列带 `tests-on-a-real-gpu` 的行 | 今天 38 行（量的，探针 `name=real_gpu_mutation_rows count=38`）；38 − 第 59 批 2（6364、6365）− 第 60 批 7（6476–6482）= 29，对得上 | 能数；29 那一刻的数没法独立重现，靠差额对上 | 主 agent 读码数 |
| 11 | 第二轮攻方探针 `6 passed`、`ATTACK` 9 行（含 `switch_naming_a_card_not_opened … admitted=true outcome=panicked:…`、`earlier_waits_timing_out=2 … resubmissions_actually_made=1 resubmissions_after_timeouts=3`、`free_memory_recut_redoes_completed_chunks … submissions_completed=8 pieces_submitted=8 recuts_after_running_out_of_memory=2 …`、`two_mounts_alive_at_once bytes_leaked_by_the_two_mounts=900 …`） | 任务 2451（`rerun.sh`） | `task-server/task log 2451` 在，14 行，我读到的 ATTACK 行与判决所引逐字相同（量的）；模型目录 `m4-gpugc-code-r2-main-check/opus-model/` 在；`research/prompts/m4-gpugc-code-r2-opus-model` 在 | 原命令要 `--frozen …/m4-gpugc-code-r2/repo`，**不在**；这些行是对 R2 冻结树量的，今天的树里它们描述的毛病已改，重跑的读数本来就不同 | 主 agent 复跑（任务 2451） |
| 12 | 第二轮辩方探针 | 任务 2420 | 退 127（`task-server/task status 2420`：命令是 `bash -c '…'` 包着路径），没跑成，**没有数** | — | 腿没量到，主 agent 没复跑出，判决写明由攻方探针同一处量到 |
| 13 | 第二轮「快照 14 份、倒推出 13 份」 | 任务 2482 | `task-server/task status 2482` 退 1（`command-failed`）；与判决「倒推不出的 1 份」同向 | 命令在，基底变了 | 主 agent |
| 14 | 第二轮「`grep -rnF TablesLargerThanABindingOnTheDevice` 只剩 `mutations.tsv` 第 5787、5788 行」 | 该 grep | 今天 0 处（量的，探针 `name=tables_larger_leftover count=0`）；第 59 批第 6 条已改 | 该命令今天得 0，不是 2：改了 | 主 agent |

### 3.2　第 59、60 批与验证段的数

| # | 数 | 命令 | 产物 | 能不能原样重跑 | 谁量的 |
|---|---|---|---|---|---|
| 15 | 「证红第 59 批 18 行」= 2672 的 11 + 4222 的 4 + 4224 的 3（均 `✓ 点名 N 条：跑了 N 条，抓到 N 条`） | `prove-red.sh --copy ~/.local/share/singlefs/work/m4-gpugc-code-r2-fix-writer/repo …`（`task-server/task status 2672` 等，量的） | 三份日志在（量的 `task log`） | **不能**：`--copy` 指的实现员副本 `repo/` 已删（`ls` 该目录没有 `repo`，量的）；而且它们量的是**实现员副本**，不是「主工作区」——记录第 705 行写「主工作区补跑（第 59 批已打）：…证红 4222」，命令里 `--copy` 指的是 `…fix-writer/repo`。副本里 7 份文件打补丁前与主工作区逐字节相同（第 59 批报告），但第 60 批和别的会话之后又改了这几份文件 | 主 agent 交的，量在实现员副本上 |
| 16 | 「第 60 批 9 行」= 4484 的 5 + 4502 的 4；「第 5265 行」= 4516 的 1 | `--copy …/m4-gpugc-batch60-writer/repo` | 日志在 | 同上，副本已删；而且「第 5265 行」今天不是 5265：该行（名字含「特性表里的特性名写错一个字母」）今天在 5211–5213 之间（量的 `grep -n`），第 60 批报告写的是 5216 | 同上 |
| 17 | 第 60 批 A：`151 passed`（bg-gpu 库）、`18 passed`（harness 一个目标）、`384 passed`（核心层库） | 任务 4475、4476、4477（`cargo test -p … --lib` 等，在主工作区） | 日志在（量的 `task log`） | 能，且有独立的数出来的参照：bg-gpu `src/` 里 `#[test]` 恰 151 个、核心层 `src/` 里恰 384 个、harness 那个目标文件里恰 18 个（`grep -c`，量的，对冻结副本）。数的是属性不是通过数，对上是旁证 | 主 agent，主工作区 |
| 18 | 合入后验证「335 个测试二进制、3229 条、慢档 77、跑了 3 个包」 | 任务 4517（`merge-and-verify.sh --skip-apply`，第二台） | 日志 754 行，那一句在最后 200 行里（量的）；任务后来被主 agent cancel（在 checker 档快档编译，盘满），**测试那一半在取消之前已印出** | 再跑得 3229 不保证：数随工作区里别的会话加的测试变；只能比 3 个包的子集 | 主 agent |
| 19 | 变异轻型「抓到 317、没红 1、无效 0、锚点失效 0，共 320」「两批点名的 16 项全部抓到」 | 任务 4715（第 4 次尝试，本机，前三次撞上限） | 日志在；16 项逐行 `✓` 在日志里（量的：七·3 的 `✓` 行 27 条，其中第 59 批点名 7、第 60 批点名 9，都在）；**但 `复用：现判 20 条、复用 300 条`**——320 行里只有 20 行是这一趟现判的（其中「行新加 15、判法单元变了 1」），317 里的 300 是判定库里的旧绿；320 − 317 − 1 = 2 是「基线不绿 2 行」，计数行不单列 | 能（同命令）；复用的 300 行不是这两批改动的，不影响「16 项」 | 主 agent |
| 20 | 崩溃轻型「53 项 0 红」（R3 背景写法） | 任务 4717 | 日志末尾 `CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=53 collected=53 green=51 red=0 not_run=2`（量的）：**2 项是「本次未跑」**（`position-addressed-trees` 等，原因写「跑的过程中源码变了，判段没在录入段那份源码上判」） | 能；这一趟的 0 红是 51 绿 + 2 没判，不是 53 绿 | 主 agent |
| 21 | 形式化轻型三格绿 | 任务 4716 | 日志在；`现判 769、复用 0`，原因「判法单元指纹 变了」（`scripts/mutation/crates-mutation-rows.py` 改了），与这两批无关 | 能 | 主 agent |
| 22 | 「三个包的 clippy 绿」 | 4506：`cargo clippy --no-deps --all-targets --all-features -p singlefs-core -p singlefs-background-gpu`；4507：`cargo clippy --no-deps --all-features -p singlefs-harness --lib --test …`（只点了 3 个测试目标） | 两份日志在，均退 0（`task-server/task status`，量的） | 能。**harness 包只 clippy 了库与三个目标**，没有 `--all-targets`；第 60 批合入后验证里 `--all-targets` 那一次红在别的会话的文件（4478） | 主 agent |
| 23 | 第 59、60 批实现员报告里的「会红」「会过」「五条没停」 | — | 报告自己写「全是推的」 | 都由上面 15–22 的任务复核了一部分 | **腿（实现员）推的** |

### 3.3　第 60 批第 1 条「回读暂存」用替身判先后：判到的是不是真卡上 wgpu 错误范围的行为

读码结论（`crates/singlefs-background-gpu/src/background_gpu.rs`）：

1. 判到的是**代码里的次序**：`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`（`:605`）里「开范围 → 建别的缓冲 → 建两份回读暂存 → 关范围取错」。把次序改成「先关再建回读暂存」（变异行 6468），替身上 `matches!(created, Err(OutOfMemoryCreatingTheBuffers{..}))` 那一条红（用例 3665）。替身的规矩有三条：有范围开着时显存不足归最里层范围且只留第一条；没有范围时记成没接住的错；空闲字节不够就建不成（`DeviceOfAChunkInMemoryForTests`，`:3582` 起）。这三条是照 wgpu 文档写的，第 60 批报告自己写「没在真卡上对过」。
2. **没有判到的**：(a) `impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device`（`:567`）三行有没有写对——只有真卡行 6364（过滤器写成 `Validation`）管，延到收口，而且那条真卡用例在「驱动把建不成的挪去别的内存」的机器上整条打 `NOT_RUN_NO_GPU_ADAPTER`（`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:374`、`:399`）；(b) 真卡上回读暂存**造不出**建不成（第 59 批报告「停下」第 1 条，读 wgpu-hal 源码推的：它是 `MAP_READ` 缓冲，按 `GpuToCpu` 挑主机内存堆），所以收口跑真卡也不会证明「回读暂存在范围里建不成会被接住」；(c) 外壳 `:1025` 把 `&gpu_in_use.device` 递给泛型函数这一接线：替身用例直接调泛型函数，不经 `GpuInUse`；如果有人把回读暂存挪回 `dispatch_both_kernels_and_read_back`（`:1051`，它只收 `readbacks: &[wgpu::Buffer; 2]`）里自己建，替身用例全绿，只有真卡能红，而真卡造不出这一条（读码推，没量）。
3. 替身对 wgpu 的假设里有一条**可能不同**：真卡上显存不足的建缓冲可能被驱动悄悄挪去别的内存、根本不报（第 59 批已见）。替身是「不够就必报」。所以替身测的是「若报，则被范围接住」，不是「会报」。

结论：替身层把「回读暂存必须和别的缓冲在同一个范围里建」这条**代码次序**钉住了（变异行 6468 红，第 60 批 B 节证红 4484 抓到）；它不证明 wgpu 真这样接错，也不钉外壳到泛型函数的接线。

## 四、推翻条件（主要结论）

- 1.3 4f「重切不留」没有测试：在 `crates/` 里找到任何一条用例，某一片先交成、后一片才建不成并断言先交成的片被重交（或一行变异把重切改成只重切没做完的而用例红），即推翻。命令：`grep -rn 'free_bytes = Some' crates`（今天 5 行，全在 `background_gpu.rs`）。
- 1.1 2e「报出来」到不了用户：`participating_gpus()` 在产品路径上出现一个调用者，或 `OpenedBackgroundGpuCompute` 多一个带代价的字段，即推翻。
- 1.2 3c「多分配一块缓冲」没有故障用例：变异表出现一行在 `background_gpu.rs` 里多建一块没记账的缓冲且有用例红，即推翻。
- 1.5 ④ 晾下又请回的卡不重量代价：`background_gpu.rs:1697` 的 `is_none()` 条件换成对任何换设置都量，或有一条用例「挂载量过 → 切换晾下 → 切换请回」断言量了第二次，即推翻；若用户读条款的「它」只指挂载时就被晾下的卡，则不是缺陷。
- 1.6 没有改坏别的句子：找到一条输入使第 59、60 批之前通过的某句条款在今天的代码上不成立，即推翻（我读到的 4 例见 1.6，都没有）。
- 2.1 的「没有去向」：在 kb、欠账行或用户弹窗记录里找到那一条的落点，即推翻该行。
- 3.1 / 3.2 的「不能原样重跑」：冻结副本 `m4-gpugc-code-r1/repo`、`m4-gpugc-code-r2/repo`、实现员副本 `repo/` 任何一个还在别处（比如别的会话备份），即推翻对应行。

## 五、要主 agent 跑的命令

全部不起 GPU、不是门禁阶段、不是 checker 档；没有一条重型。`<拷贝>` 由主 agent 先从冻结副本拷出（不在冻结副本里改）：

```
rsync -a --exclude target --exclude .git /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3/repo/ /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/copy/
```

C0　只读事实探针（复跑 24 行 FACT，逐行比 `out/facts.expected`）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/facts-run-1 --model /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-sonnet-model
```

C1　第 59、60 批 28 行证红在入库树（拷贝）上重做一遍（3.2 第 15、16 行：原来量在实现员副本上）。同一份拷贝上一条跑完再交下一条（第 60 批报告 B 节的要求）。`<拷贝>` 填上面那个目录。名字取自冻结副本 `crates/mutations.tsv` 的行 6355–6358、6360–6362、5701、5702、5904、6235（A1）、6363、6366–6368（A2）、6359、6369、6370（A3）、6468、6469、6473–6475（A4）、6470–6472、5501（A5）、unit-graph 那一行（A6）；整条命令文本在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet/prove-red-commands.txt`，下面原样抄：

```
# A1 第59批 GPU包 11行 (原2672)
task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/mutation/prove-red.sh --copy <拷贝> singlefs-background-gpu '七·3 GPU 代码轮二：切换校验又现查一次本机在的独显去取没开成的卡（改回各读一次）' '七·3 GPU 代码轮二：重交次数数在重交之前那一等之前（那一等又超时、没交也数一次重交）' '七·3 GPU 代码轮二：重交之前那一等又超时、没交的那一回也数进重交' '七·3 GPU 代码轮二：外壳报的重交次数取成这一批这张卡的超时次数' '七·3 GPU 代码轮二：进程里同一张卡再泄漏时盖掉前一次的（两次挂载同开时后丢的盖掉先丢的）' '七·3 GPU 代码轮二：外壳丢的时候把卡上占着的全当此前挂载泄漏的（这一次挂载自己泄漏的记成 0）' '七·3 GPU 代码轮二：外壳丢的时候不扣开卡时已算上的此前泄漏的那一片（进程里重算一遍）' '七·3 后台 GPU 包：实现把乙段 GPU 出错（含超时）报成第一版不支持的输入' '七·3 后台 GPU 包：实现把丙段 GPU 出错（含超时）报成第一版不支持的输入' '七·3 超时重交与卸载不等：重交之前等前一次没等到也照交' '七·3 GPU 代码轮一：进程里同一张卡再泄漏时字节加上去（重算已含的那一片）'

# A2 第59批 核心层 4行 (原4222)
task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/mutation/prove-red.sh --copy <拷贝> singlefs-core '七·3 GPU 代码轮二：GPU 用不上时停着的后台任务不照每一个成员的表挑（表里一个成员都没有）' '七·3 GPU 代码轮二：叫起里出错停下的那一趟照数（报 1，改前的数法）' '七·3 GPU 代码轮二：不在叫起里停下的（可写挂载续做）也当停在叫起里（停下之后头一次叫起报 0）' '七·3 GPU 代码轮二：一次叫起结束时不关「叫起在做」（之后不在叫起里停下的也当停在叫起里）'

# A3 第59批 harness 3行 (原4224)
task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/mutation/prove-red.sh --copy <拷贝> singlefs-harness '七·3 GPU 代码轮二：攒批会话排空待发被拒的那一次不取报警（不数这一次叫起、拒绝里不带报警）' '七·3 GPU 代码轮二：显式入口一趟开头不记叫起在做（这一趟里出错停下的照数）' '七·3 GPU 代码轮二：后台入口判起跑开头不记叫起在做（那一次起跑里出错停下的照数）'

# A4 第60批 GPU包 5行 (原4484)
task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/mutation/prove-red.sh --copy <拷贝> singlefs-background-gpu '七·3 回读暂存与真卡测法：两份回读暂存改回建在显存不足的错误范围之外（关上范围之后才建）' '七·3 回读暂存与真卡测法：回读暂存只建一份核结果字节数的一半' '七·3 回读暂存与真卡测法：外壳报的片数取成交成的次数（一张替身卡）' '七·3 回读暂存与真卡测法：一批只交给头一张分到活的卡（两张替身卡）' '七·3 回读暂存与真卡测法：一批只交给头两张分到活的卡（三张替身卡）'

# A5 第60批 harness 4行 (原4502)
task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/mutation/prove-red.sh --copy <拷贝> singlefs-harness '七·3 回读暂存与真卡测法：带整理开关的后台计算挂载入口不把整理开关递进去（恒给产品那一组，续做在交 GPU 之前停下）' '七·3 回读暂存与真卡测法：可写挂载续做当成一次叫起（续做里停下之后头一次叫起报 0）' '七·3 回读暂存与真卡测法：停着时补跑的那一批照设置交给 GPU（不在 CPU 上补跑）' '七·3 后台计算：可写挂载校验过的设置不交给会话（会话照产品默认 cpu）'

# A6 unit-graph 1行 (原4516)
task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/mutation/prove-red.sh --copy <拷贝> singlefs-unit-graph '七·6 后续批第二轮 构建脚本：特性表里的特性名写错一个字母（与核心层清单 [features] 对不上，正推 T4a）'
```

C2　测试条数在入库树上重量（对 3.2 第 17 行；各自 `--cwd` 同一份拷贝，串行）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/copy -- cargo test -p singlefs-background-gpu --lib
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/copy -- cargo test -p singlefs-core --lib
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/copy -- cargo test -p singlefs-harness --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure
```

期望（推的）：151、384、18 passed（与 `#[test]` 个数相等）。

C3　C906 还账前点名那一条（2.3）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/copy -- cargo test -p singlefs-harness --test compaction_off_the_calling_thread_and_admission -- a_compaction_batch_that_would_push_the_available_below_the_demand_is_refused_by_the_admission_and_writes_nothing
```

C4　harness 包 `--all-targets` 的 clippy（3.2 第 22 行：4507 只点了库与 3 个目标）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/copy -- cargo clippy --no-deps --all-targets --all-features -p singlefs-harness -- -D warnings -D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated
```

C5　（可试）重建第一轮、第二轮的冻结树，好让 3.1 第 2–5、11 行能原样重跑；基底变了可能倒推不出，推的：

```
task-server/task submit -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-gpugc-code-r1 --out /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/r1-tree --repository /home/fy5090/code/singlefs
task-server/task submit -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-gpugc-code-r2 --out /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet-main/r2-tree --repository /home/fy5090/code/singlefs
```

要补的测试（实现员的活，不是命令；写法是我读码推的，没量过）：

1. 重切不留：替身卡 `free_bytes` 取一个介于「头一段」与「第二段」占用之间的值，批里放大小不等的单元（头一段装得下、第二段才建不成），断言 `submissions_completed` 比 `pieces_submitted` 多出头一段那一次（头一段被重交），并配一行变异「重切只重做没做完的」要红。
2. 多分配一块缓冲：给替身卡的建缓冲记账多一个口子（建出的字节数与 `bytes_held_by_this_chunk` 对不上就红），或在外壳里数真建的缓冲字节，配一行「多建一块不记账的缓冲」的变异。
3. 晾下又请回的卡：外壳用例「挂载量过 → 切换到装不下 → 切换到装得下」，断言第二次量过代价（或写明不量，改条款）。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 第 2 条 | 一致，2e 做到一半 | 2a–2d 有代码有钉；2e「并报出来」：量出的代价 `participating_gpus()` 产品路径零调用者，「赔」无判法（零轮） |
| T1 第 3 条 | 一致，3c 缺故障用例 | 3a–3g 有代码有钉；七·3 验收「多分配一块缓冲的故障要红」没有用例也没有变异行（零轮） |
| T1 第 4 条 | 一致，4f 缺测试 | 4a–4e、4g–4n 有代码有钉；2026-11-01 补的「重切已做完的片不留」代码对、**没有一条测试或变异行钉它**（零轮） |
| T1 第 8 条 | 一致，8e 待核 | 这两批没碰；「已交回的段先算一轮」我没读到单独断言；WGSL、真卡只读码推 |
| T1 2026-11-01 补的两句 | 叫起计数一致有钉；重切不留无钉 | 叫起计数三种停法各有测试与变异行；重切不留见上 |
| T1 两批改坏别的句子 | 没找到 | 点名两例（只现枚举一次后掉卡仍能拒切换；泄漏累加后「上限不超显存总量」不扣泄漏）读码都没坏；另发现 1.5 ④（晾下又请回的卡不重量代价）是条款与代码可能不一致，不是这两批改坏的，零轮 |
| T2 清单 | 28 条；12 条没有去向 | 没有去向：第 8、9、10、13、14、15、16、17、20、21、22、27 条（见 2.1）；kb 写回漏一处（里程碑四七·3 验收仍写「攒批」）；mount-parameters 与 C942、C943 的行号落点已漂 |
| T2 验收缺口 | 3 句缺、4 句弱 | 缺：多分配一块缓冲；各自切片读窄；模型对拍无参照模型。弱：赔的卡报不出；真卡空闲显存那条可能全程「本次未跑」；档位没核；运行时可见性有两处洞 |
| T2 欠账行 | 七·3 收口前：关 C906、表态 C923、交用户 C942；留：C921、C926、C927、C943 | 见 2.3 |
| T3 前两轮的数 | 重跑能力：R1、R2 冻结树都已删，原命令不能重跑；产物日志大多在 | 3.1 的 14 项：同一个数能重现 2 项（第 8、10）；原命令不能重跑（冻结树已删）5 项（第 2–5、11）；今天重跑得不同的数 3 项（第 7、9、14）；基底变了待试 2 项（第 1、13）；只在腿副本上量 1 项（第 6）；没有数 1 项（第 12） |
| T3 第 59、60 批与验证段的数 | 证红 28 行量在实现员副本上，副本已删；条数 151/384/18 有独立旁证 | 3.2：变异轻型 317 里只有 20 行现判；崩溃轻型 53 项里 2 项没判；harness clippy 只点了 3 个目标 |
| T3 替身判先后 | 钉住代码次序，不证明 wgpu 真如此接错，不钉外壳接线 | 3.3 |

## 七、没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务（`task-server/task` 我只用了 `status`、`log`）、没起 cargo、没起 GPU、没跑任何门禁阶段；判决要引的数没复跑，命令在第五节。
- 没读禁读清单里的任何一份（攻方本轮的报告、模型目录、主 agent 本轮核查、`m4-gpugc-code-r3-opus` 草稿目录）。
- 没起真卡用例、WGSL 核只读码；第 8 条两份 GPU 核算得对不对，没验。
- 没逐条核第一轮判决 T1–T6 里每一个 `crates/…:行号`（那些行号对 R1 冻结树，树已删）。
- 没读 `region_scan.rs`、`checksum.rs` 的实现，只读了对应的变异行名与测试名。
- 1.1 2d「按量出的代价分活」只读了 `gpus_of_this_host.rs:539` 一带与 `work_split_by_measured_cost.rs` 的测试名，没逐行核份额算术。
- 下面的 `自己跑过一次` 只有探针的草稿目录输出：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3-sonnet/w1/facts.out`（24 行）；它是对 `steps.sh` 里同一段 `rerun_steps` 函数体直接 source 跑的，不是 `rerun.sh` 整份；`bash -n rerun.sh` 过。`rerun.sh --selftest` 没跑（归主 agent 的自证，档位 light）。

## 八、模型目录与 sha256

`research/prompts/m4-gpugc-code-r3-sonnet-model/`（量的，`sha256sum`）：

5870d3eba560c87b36738c3ab979df13ae91fbf0486402eb69d1d5aace75e2c1  tools/rerun.sh
5868fbfd3e19e99c48700a77f19445aecad49d6f52a62092a3a909f03df0a9c5  tools/frags.tsv
fb9a01d22164bbeb2e81ca6cb4d16c6efe4e121d5490c58816df0e8b03110c65  out/facts.expected
d93613a36ae7c93c98baca2c550ba3d865fb915017230980e8b707fa1b9f8789  README.md
