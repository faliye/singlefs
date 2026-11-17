# singlefs-background-gpu 与测试包严格审核

对象：工作区里的 `crates/singlefs-background-gpu` 与 `crates/singlefs-background-gpu-tests`，按树里的现行源码读，不是相对某次提交的差异。本轮只新增这份记录，没有改产品实现、测试、欠账表或决策正文，也没有提交。

工作区落在提交 `8ed2ea4845ef0023fcf9a41e3ac09fece4423c0d` 上，这两个包相对该提交都有未提交改动（测试包从产品包拆出、产品源码有改）。下面的行号与摘要都是这次读到的工作区，不是该提交的树。

对照的条款是 D26（后台整理与放置回收）已定项 13。2026-11-10 那份审核（`records/2026-11-10-singlefs-background-gpu严格审核.md`）的 G01–G04 已有修复记录（`records/2026-11-11-background-gpu审核四条修复.md`）。本轮不把那四条再编号。

## 结论

产品路径上没有新的、能指出错误输出的缺陷。新确认的两条都在测试包：一条真卡用例在占用读不到时仍以成功返回；e202 的 CPU 多判臂把活槽数算完丢掉，对不上的条数加不上去。没有把「这两个包没有缺陷」当作结论。没有跑测试，没有开真卡。

| 编号 | 优先级 | 位置 | 读到的行为 | 成立条件 |
| --- | --- | --- | --- | --- |
| T01 | P2 | `crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs` 第 184–193 行；占用读法在 `tests/common_discrete_gpus_of_this_host/mod.rs` 第 584–588、616–640 行 | 前台 fsync、整理发布、报警为零这三道断言已经通过之后，`OnlyTheHandedOver` 与 `NotReadSoThisRunIsNotRun` 走同一个空分支，接着卸载，`#[test]` 返回成功。枚举上写明 `NotReadSoThisRunIsNotRun` 是「读不成、或读到这个进程一张卡都不在」，调用方不当绿、不把这一趟的结论入库。读到一张不在交来名单里的卡时，辅助函数自己 `assert`，这一条仍红。同目录 `mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs` 第 246–250 行在 `NotReadSoThisRunIsNotRun` 上交回 `ComparisonOnTheCards::NotRunOnTheseCards`，判定库不存结论 | 服务交了卡，这条用例已经看到 `GPU_BATCHES_TO_SEE` 批并做完前台断言；随后 `nvidia-smi` 读失败，或计算进程列表里没有这个进程。文件头第 5 行「服务没交卡时打 `NOT_RUN_NO_GPU_ADAPTER`、返回成功」是另一条、开卡之前的未跑，不在这一条里。按 `NOT_RUN_NO_GPU_ADAPTER` 整行丢掉这一趟的日志汇总不会把这次记成对拍通过；`cargo test` 的退出码仍是 0 |
| T02 | P3 | `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs` 的 `rejudged_batch_on_the_cpu` 第 3105–3171 行；闸在 `the_runs_of_the_repeat_are_identical` 第 5445 行，小池单测第 10753–10754 行 | 函数注释写这段交回再判了几条、对不上几条，活槽段也在里面。循环对每块盘调 `live_slots_per_cluster_segment_by_the_records`，只把 `live_slots.len()` 加进 `entries_rejudged`，向量里的数不参与比较，`mismatches` 不变。前提 I 与小池单测都要求 `rejudge.mismatches == 0`，这一段活槽数怎样都过不了这道闸的反方向。同一文件的 GPU 包装 `rejudged_live_slots`（第 2946–2969 行）对每个确认段拿 CPU 数去比，漏的是 CPU 多判臂。本轮没有观察到一个具体的错误活槽数；函数里也没有第二个期望值可对 | CPU 多判臂跑到有分配记录的盘。活槽计数与整理所用的数不一致时，`mismatches` 仍是 0，`entries_rejudged > 0` 只靠段数就能满足 |

没有 P0 或 P1。T01 让「只在交来的卡上」这一条在读不到占用时放行。T02 是阳性对照的活槽一段没有第二个数可对，闸因此钉不住这一段。

## 范围和这次实际做了什么

产品包分层在 `src/lib.rs`：纯逻辑（卡清单、显存与切批、按卡填满、两核比对、三段打包）、PCI 读数、wgpu 外壳 `background_gpu.rs`、核心层特质实现 `background_gpu_compute.rs`、进程开卡器。依赖是 `wgpu =30.0.1`（特性 `vulkan`、`wgsl`、`std`）、`ash =0.38.0`、`libc`。特性 `tests-on-a-real-gpu` 才编真卡测试。验证包与产品包不互相依赖；测试包同时依赖两边。

这次用工作区文件点过的行数（末尾没有换行的文件按多一行计）：

| 文件 | 行数 |
| --- | --- |
| `singlefs-background-gpu/src/lib.rs` | 37 |
| `src/adapter_inventory.rs` | 828 |
| `src/background_gpu.rs` | 11039 |
| `src/background_gpu_compute.rs` | 1395 |
| `src/checksum_kernels.rs` | 740 |
| `src/gpu_opener_of_this_process.rs` | 564 |
| `src/gpus_of_this_host.rs` | 2195 |
| `src/live_slot_kernels.rs` | 1073 |
| `src/measured_host_cpu_time_per_submission.rs` | 8 |
| `src/memory_limit_and_chunks.rs` | 1830 |
| `src/pci_state_of_the_gpu.rs` | 561 |
| `src/region_record_kernels.rs` | 2569 |
| `src/two_kernel_reconciliation.rs` | 435 |
| `src/waiting_for_the_gpu.rs` | 1187 |
| 六份 `.wgsl` | 77、67、130、102、533、508 |
| 产品包 `tests/` 三份 | 288、196、543 |
| 产品包 `Cargo.toml`、`build.rs` | 34、23 |
| 测试包 `src/bin/e202_gpu_cpu_gc_comparison.rs` | 13207 |
| 测试包其余 `.rs` 与 `Cargo.toml`、`build.rs` | 合计约 8686（47 个 `.rs` / `.wgsl` / `Cargo.toml` 里扣掉上面产品包与 e202 之后） |

上面 47 个文件按路径排序，各自做 SHA-256，清单写成 `<摘要><两空格><路径>` 每行一条、末尾补一个换行，再对这份清单做一次 SHA-256，结果是 `ef127125366685642e48629e74697b1e3a75311bee5858bb3af2efaad0d33acc`。这是读码当时的工作区，不是发布物摘要，也不是 2026-11-10 那份记录里的 `818c3fe4…`（那次只有当时的产品包，且 e202 还在产品包里）。

本轮执行过的只有读文件、检索、行数与摘要。没有跑 `cargo test`，没有开 `tests-on-a-real-gpu`，没有清场，没有跑 e202。

## 已登记、本轮不另编号

欠账表仍开着、和这条路径有关的：C926（等的上限默认 10 秒没量过）、C952（单次抽样与泄漏账持有方，含建缓冲时关错误范围等超时仍数成一次重交）、C996、C1028（重复键放弃一批后连着再签意图的崩溃流没核）、C1030（等提交时卡真丢了会不会走驱动那条 panic，外壳已接住 `device.poll` 的 panic，真卡掉线没量过）。本轮不销账、不改欠账表。

C996 那一行的现象句仍写「开卡在本机每一张独立显卡上起设备」。现行 `gpus_of_this_host.rs` 的 `gpus_opened_at_the_mount`（第 885–929 行）对不参与的卡 `continue`，推进 `not_opened_for_not_taking_part`，不开设备。验收那一半（真卡用例只分到一张卡时，进程上了别的卡要红）仍开着；T01 就是这条验收在「前台并发整理」那一条用例上没钉住。

C923 在欠账表「已还清」一节，还清日期 2026-11-12。本轮不把它算成仍欠。

G01–G04 的现行形状对过、不另编号：映射着建的缓冲按 `COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION = 2` 记账（`memory_limit_and_chunks.rs` 第 193 行）；同一批重复键在改计数和交卡之前返回 `SameEntryKeyTwiceInOneBatch`；开卡期限按枚举之后再加每张卡；还在跑的卡卸载时 `mem::forget` 并记泄漏。2026-11-10 记录里「C996 仍是每张独显都开」那一段对不上现行代码，不要照那一段读今天的树。

## 对过、不单列成问题

- `lib.rs` 第 9 行仍写「挂载时开着的本机全部独显」，第 14 行写「挂载那一刻只开参与的卡」。第 14 行与 `gpus_opened_at_the_mount` 一致。第 9 行是注释，不改变开卡。
- `compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs` 文件头第 3 行与 `the_implementation_answers_the_participating_gpus_as_the_inventory_the_core_admits_against` 的注释（第 291–292 行）、断言消息（第 350 行）写卡清单是本机全部独显、另含没点名的卡。断言本身是参与集合等于交来的卡，且参与集合是卡清单的子集（第 343–351 行）。没点名的健康卡进不进清单，这条断言不看。产品侧 `gpus_opened_at_the_mount` 把没参与的卡留在清单里。这是覆盖比消息窄，不是现行产品把没参与的卡开了。
- 真卡用例在服务没交卡时打 `NOT_RUN_NO_GPU_ADAPTER` 并返回成功：文件头写明，判定库对 `NotRunOnTheseCards` 不入库。这和 T01 不同，T01 是已经在交来的卡上跑完功能断言之后，把读不到占用也当成成功。
- `pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs` 的 `kernel_files_every_gpu_side_group_of_the_target_reaches`（第 312–332 行）对校验和、活槽、区域记录三类目标点名两份核；`_` 臂是空切片。同函数注释写端到端那一个目标的几条走的核各不相同，只要求至少走到一份。测试第 372–374 行：清单为空时，闭包里有任意一份 `crates/singlefs-background-gpu/src/` 下的 WGSL 计算入口即过。本轮没有重编键表，没有看到某一组的闭包漏了它真正分发的那份核。不把「注释写明的较弱检查」编号成已经发生的假绿。
- 活槽行走核的重叠用低 32 位相减。产品按 64 槽对齐的聚簇段切界，一段不跨过 `u32` 边界；段尾正好落在 `2^32` 时回绕减等于段长，重叠再夹回段内。没有坐实成错账。
- 两份区域记录核的键宽、索引节点条目数偏移、打包顺序对得上。检索核把第一份寄存器字段命名成另一核的寄存器，计算用的是第二份；打包按行走核、检索核的顺序写。行为对得上。两核都同意的错误分类要两份解析一起错，本轮没有造出这样的输入。CPU 参照走核心层 `scan_region_over_the_identity_index`，不是把 WGSL 再实现一遍。
- `device.poll(PollType::Wait)` 返回 `Ok` 时把这次等待当成提交完成：wgpu-core 30.0.1 在等待的提交做完时交 `QueueEmpty` 或 `WaitSucceeded`，超时是 `Err`。`device_poll` 在返回之前跑用户闭包，`map_async` 的回调在这次 poll 里触发。设备丢失时的 panic 由 `completion_check_of_one_poll_with_its_panic_caught` 接成 `UncapturedErrorOrDeviceLost`。真卡上驱动走不走这一支仍是 C1030，不另编号。
- 按卡填批：前面的卡取提交次数与第一条相同的前缀，最后一张卡拿剩余；装不下最小一片的卡剔出再计划。`entry_position` 先是切片内下标，再加上这段的起点，两次相加还原绝对下标，不是重复计数。
- PCI 目录名拒绝空串、`.`、`..`、`/` 与 NUL。读 PCI 只在判卡用不了的那一类原因上发生。
- 错误范围的 `pop` 先置已弹出，超时不再弹第二次。
- 开卡枚举本身在等的上限之外，文件里写明。开卡线程在标了实例已建、实例还没放进槽之前 panic，可能占掉四个实例名额里的一个。本轮没有把这个时序坐实成挂载失败。

切批、显存账、按卡填满、等待与重交、两核比对、开卡器闸、三段打包与六份核入口，另有一遍按函数名读过（`checked_bytes_held_by_a_chunk`、`chunks_that_fit_on_the_gpu`、`pieces_cut_to_fit_on_the_gpu`、`GpuMemoryAccount` 的持有与结算、`plans_of_a_batch_filling_the_gpus_in_order`、`gpus_opened_at_the_mount`、`outcome_resubmitted_after_timeouts`、`reconcile_the_two_kernel_outcomes`、`open_the_background_gpu`，以及三段的计划与回读）。那一遍没有交出可编号的产品缺陷。超时停在第 4 次，与 `1 + RESUBMISSIONS_AFTER_A_TIMEOUT` 一致。

## 没有逐行读完的部分

- `background_gpu.rs` 11039 行。读过开卡、错误范围里建缓冲、六条管线、poll 与回读、按卡计划与校验和一段的建缓冲失败路径、切换时关掉不再参与的卡。没有逐行读完。
- `background_gpu_compute.rs` 只按 `use_no_gpu` 与整批失败的映射检索，没有逐行读。
- `adapter_inventory.rs` 读了枚举拒卡与点名准入（约第 90–309 行）。其余多是测试，没有逐行读。
- `waiting_for_the_gpu.rs` 读了等到期限的循环和重交入口，1187 行没有逐行读完。
- 两份区域记录 WGSL（533 与 508 行）对过分类与键的组装，没有和 CPU 解析器逐行对齐。
- e202 13207 行只读了再判、重复闸、小池单测里活槽那几处，以及 nvidia-smi 读不到时期待 `NotReadSoThisRunIsNotRun` 的单测。其余没有逐行读。
- 产品包 `tests/` 的三份（容量契约、形式化契约、开卡器闸）没有逐行读。
- 测试包里对拍库、历史池、校验和与区域记录的真卡用例，读了占用辅助函数、判定库复用、端到端卡清单那一条和隐藏 Vulkan 驱动再起自身的那一条。其余真卡文件没有逐行读完。

没有跑过的：库内单测、真卡特性、e202、开卡器闸测试。
