# singlefs-background-gpu 严格审核

对象：`crates/singlefs-background-gpu` 当前工作区，按树里的现行源码读，不是相对某次提交的差异。本轮只新增这份记录，没有改产品实现、测试、欠账表或决策正文，也没有提交。

对照的条款是 D26（后台整理与放置回收）已定项 13，正文在 `.claude/kb/decisions/26-后台整理与放置回收.md` 第 246–274 行。第 2、3、4、8 条是这次对代码的尺子。`records/2026-11-01-crates逐函数深度代码审核.md` 里这个包只审过两个函数；那里的 F-63（按代价倒数分活）对不上现行代码：`measured_host_cpu_time_per_submission.rs` 现在只留一次提交的主机 CPU 时间这个类型，分活不再用它。那一条不重复计。

## 结论

读码确认两条缺口：开卡失败路径上的析构与记账，以及建缓冲时设备本地暂存没有进显存账。另外两条是代码自己写明、触发条件明确的行为，比条款里「一张卡到点就当没开成」更严。没有把「这个包没有缺陷」当作结论。真卡测试、库测试、`src/bin/e202_gpu_cpu_gc_comparison.rs` 都没有在本轮跑过或逐行读完。

| 编号 | 优先级 | 位置 | 读到的行为 | 成立条件 |
| --- | --- | --- | --- | --- |
| G01 | P2 | `background_gpu.rs` 的 `started_gpu_with_its_cost_measured`、`both_kernel_outcomes_of_a_checksum_chunk`；`waiting_for_the_gpu.rs` 的 `outcome_of_opening_a_gpu_within_the_limit`；wgpu-core 30.0.1 `queue.rs` 的 `Queue::drop` | 开卡过程中一次已经提交的代价探针失败后，`GpuInUse` 按普通值丢掉。`Queue` 析构调用不带时限的 `wait_for_idle`。这次占着的字节不进进程泄漏账。卸载路径对还在跑的卡是 `mem::forget` 再记账 | 代价探针已经 `queue.submit`，随后的等待交回错误；或单卡 5 分钟到点时，开卡线程仍握着设备 |
| G04 | P2 | `background_gpu.rs` 的 `buffer_mapped_at_creation_within_the_scope_of_the_caller` 与校验和一段的 `bytes_held_by_this_chunk`；wgpu-core 30.0.1 `device/resource.rs` 的 `mapped_at_creation` 分支；wgpu-hal 30.0.1 `vulkan/device.rs` 的 `create_buffer`；gpu-allocator 0.28.0 的 `CpuToGpu` | 账上只加各份 `buffer.size()`。没有 `MAP_WRITE` 的 `mapped_at_creation` 缓冲，wgpu-core 另建一份同尺寸暂存。Vulkan 上这份暂存优先放进设备本地且主机可见的堆，和设备缓冲同时占着，直到内部拷贝那次提交做完 | 给了显存上限，一段按剩下的界切满，并且暂存的优先分配成功落在设备本地堆。优先分配失败、退到非设备本地的主机内存时，这一份不算进显存 |
| G02 | P3 | `gpu_opener_of_this_process.rs` 的 `OPENING_DEADLINE_BY_DEFAULT`；`singlefs-mount-runtime` 的 `mount_without_the_maintenance_thread` | 进程开卡器默认再加一层 15 分钟。到点整次开卡失败，挂载拒绝。文件头写明这一层是第一版接受的算法 | 一次开卡的工作线程超过这个期限才交回；多张卡串行、或枚举本身耗时，都会吃进这一个期限。排队时间另计 |
| G03 | P3，条件性 | `two_kernel_reconciliation.rs` 的 `reconcile_the_two_kernel_outcomes` | 同一批里同一个条目键出现两次时 `assert`，整理线程 panic，这一批不会按 GPU 出错交回 | 调用方交进一批的键有重复。挂载后的扫描在进比对之前滤掉非单元跨度，恢复拒绝跨度大于 0 的重叠记录。直接构造重复键的测试仍期待这次 panic。2026-11-01 那份审核里的 F-24 仍在 |

P2 有两条。G01 是条款写了「还在跑的提交不析构、记进泄漏」，开卡失败路径没走到那条。G04 是条款写了缓冲和暂存一起算进上限，映射建缓冲的那份设备本地暂存没进账。P3 是代码写明了的更严行为，或产品扫描路径上没有找到输入的 panic。没有 P0 或 P1。

已登记、本轮不另编号的四条欠账仍在 `.claude/kb/checks-owed.md`：C923、C926、C952、C996。下面「已登记的欠账」一节写它们和现行代码的关系。

## 范围和这次实际做了什么

包的分层在 `src/lib.rs`：纯逻辑（卡清单、显存与切批、按卡填满、两核比对、三段打包）、wgpu 外壳 `background_gpu.rs`、核心层特质实现 `background_gpu_compute.rs`、进程开卡器 `gpu_opener_of_this_process.rs`。依赖是 `wgpu =30.0.1`（特性 `vulkan`、`wgsl`、`std`）、`ash =0.38.0`、`libc`。特性 `tests-on-a-real-gpu` 才编真卡测试。开发依赖里的 noop 后端只在编测试时进来。

这次用 `wc -l` 点过的现行行数：

| 文件 | 行数 |
| --- | --- |
| `src/adapter_inventory.rs` | 828 |
| `src/background_gpu_compute.rs` | 1000 |
| `src/background_gpu.rs` | 4826 |
| `src/checksum_kernels.rs` | 719 |
| `src/gpu_opener_of_this_process.rs` | 288 |
| `src/gpus_of_this_host.rs` | 1928 |
| `src/lib.rs` | 35 |
| `src/live_slot_kernels.rs` | 881 |
| `src/measured_host_cpu_time_per_submission.rs` | 8 |
| `src/memory_limit_and_chunks.rs` | 1041 |
| `src/region_record_kernels.rs` | 2498 |
| `src/two_kernel_reconciliation.rs` | 342 |
| `src/waiting_for_the_gpu.rs` | 961 |
| `src/bin/e202_gpu_cpu_gc_comparison.rs` | 11643 |
| 六份 `.wgsl` | 77、67、130、102、533、508 |
| `tests/` 下 11 个 `.rs` | 合计 5000 |

上面这些 `.rs`、`.wgsl` 再加这个包的 `Cargo.toml`，按路径排序后各自做 SHA-256，再对这份清单做一次 SHA-256，结果是 `818c3fe4a665ad2857d0bcec7a317e067eca6e724f47471990923b12a4e82997`。复核时用同一条命令重算，摘要没变。这是读码当时的树，不是发布物摘要。

本轮执行过的只有读文件、检索和上面的行数与摘要。没有跑 `cargo test`，没有开 `tests-on-a-real-gpu`，没有清场，没有跑 e202。e202 与 `tests/` 没有逐行审。六份 WGSL 与对应的 CPU 参照在本轮早些时候对过算法形状（见文末「对过、不单列成问题」），没有在真卡上重放。

另派过三路只读复核，三路都已交回。写进下面编号的，都再对照过现行行或所引的 wgpu、wgpu-hal、gpu-allocator 源码。切批那一路里有两条和条款句子表面冲突，对照第 45 批记录之后不成立，写在「对过、与已有定法一致」里。外壳那一路里，开卡析构与 C996、C952 的读法和下文一致；显存账漏计暂存是对照 wgpu 之后新写上的 G04。三段核那一路交回的结论是没有坐实的核与核心层分叉。本记录只采纳了其中对照过现行行的重复键那一条，不把未逐行复读的 WGSL 升格成「核已证明与核心层一致」。

## G01：开卡失败丢掉还在跑的队列，泄漏账不记

卸载路径是按条款做的。`gpus_of_this_host.rs` 的 `gpus_dropped_leaking_those_still_running` 对还标着「等超过上限、提交还在跑」的卡 `mem::forget`，其余才 `drop`。`BackgroundGpuOverTheCards::drop`（`background_gpu.rs`）只把这些卡交进去，再 `note_the_gpus_leaked_when_dropped`。注释写明原因：wgpu-core 30.0.1 的 `Queue` 析构调用 `wait_for_idle`。现行 `wgpu-core-30.0.1/src/device/queue.rs` 第 275–288 行就是这个：`unsafe { self.raw.wait_for_idle() }`，非 `Lost` 的错误会 panic。`GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts` 把记过的字节放进 `bytes_held`，这一次挂载的上限从这里扣。

开卡量代价走的是另一条。`started_gpu_with_its_cost_measured` 在设备与六条管线建好之后，`CostAtTheOpening::MeasuredUnderTheMemoryLimit` 时调用 `measure_the_cost_of_one_submission()?`。代价探针走 `both_kernel_outcomes_of_a_checksum_chunk`：`queue.submit` 之后 `settle_the_chunk_after_its_submission`，再 `read_back?`。`SubmissionOfTheChunk::after_waiting_for_its_completion` 对等待的任何 `Err` 都标成 `MayStillBeRunningOnTheGpu`。`settle` 因此把这段字节留在账上。随后的 `?` 丢掉局部的 `GpuInUse`。

`GpuInUse` 没有自己的 `Drop`。丢掉它会析构里面的 `wgpu::Queue`，走到上面的 `wait_for_idle`。进程泄漏账只在 `BackgroundGpuOverTheCards::drop` 里更新，这次开卡还没有交回那个把手，所以这张卡上还占着的探针缓冲不进 `GpuMemoryLeakedInThisProcess`。下一次挂载的账从「正式卸载记过的字节」起算。

单卡外面还有一层线程。`outcome_of_opening_a_gpu_within_the_limit` 在名为 `singlefs-opening-a-gpu` 的线程上跑 `open`，调用线程 `recv_timeout(opening_limit)`。`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT` 是 `Duration::from_secs(5 * 60)`。到点交回 `WaitForTheGpuExceededTheLimit`，线程不杀。`open` 的发送发生在 `started_gpu_with_its_cost_measured` 返回之后，因此发生在 `GpuInUse` 析构之后。两种时序都成立：

- 析构里的 `wait_for_idle` 在剩余的 5 分钟里返回：调用方收到探针自己的错误，设备已经析构完。这 5 分钟包住的是「等析构」，不是条款写的「到点当没开成、不等」。
- 5 分钟先到：调用方把这张卡当成没开成并继续；开卡线程仍停在 `Queue::drop`。参与的卡因此让 `gpus_opened_at_the_mount` 交回 `GpuUnusable`，工作线程随后放开 `an_opening_is_hung`。另一次挂载可以在同一张物理卡上再占到自己的上限，而上一台设备还占着显存，新账里没有这几字节。

开卡器注释写「到点之后开出来的东西没人要，随丢弃记进泄漏账」。这句话只覆盖已经交成 `OpenedBackgroundGpu`、再由 `BackgroundGpuOverTheCards::drop` 丢掉、并且卡上标着提交还在跑的情形。开卡中途丢掉的 `GpuInUse` 不进这本账。

没点名的卡也会量代价，所以 G01 不要求这张卡在参与名单里。`gpus_opened_at_the_mount` 只在「有显存上限、扣掉已占字节后装不下最小片」时才标 `NotMeasuredForTheGpuIsLeftOut`。没点名的卡上限是 `None`，代价照量。这和 C996 是同一条开卡循环，C996 记的是「没参与的卡也起设备」；G01 多的是量代价提交失败之后的析构。

## G02：15 分钟到点是整次挂载拒绝

单卡上限是 5 分钟，见上一节。进程开卡器的默认期限是这个秒数乘 3：`OPENING_DEADLINE_BY_DEFAULT`。文件头写明：只点一张卡也会被没点名的卡拖着串行开，最坏是卡数乘一张卡的上限；默认按三张卡推，第一版接受。期限从拿到 `one_opening_at_a_time` 之后才算，排在前面的开卡不占这一次的期限。

到点交回 `BackgroundGpuNotOpened::TimedOut`。`mount_without_the_maintenance_thread` 对开卡的任何错误都 `RuntimeMountRefusal::BackgroundGpuNotOpened`，挂载不留下「这一张跳过、其余照开」的中间态。工作线程不杀。它还没跑到放开闸的那一行时，下一次开卡得到 `AnEarlierOpeningIsStillHung`，同样整次拒绝。

枚举不在单卡 5 分钟里。`open_background_gpu_with_the_memory_leaked_in_this_process` 的注释写明：wgpu 在构造 future 时同步做完枚举。能截住调用线程的是外层这个期限。独显多于三张、或三张都用满 5 分钟再加枚举时，外层会在各张卡自己的 5 分钟还没分别走完之前截断整次。

## G04：映射建缓冲的设备本地暂存不在账上

第 3 条要求任何时刻占用都不超过那张卡的上限，缓冲和暂存一起算。产品在 `hold` 之前把这一段要建的各份 `buffer.size()` 相加。校验和一段是参数、输入、描述，再加 `4 * outcome_bytes`（两份结果加两份回读）。区域、选段两段同样只加自己建的那几份。建完后 `buffers_and_both_readbacks_created_within_one_out_of_memory_scope` 把真建缓冲的 `buffer.size()` 再加一遍，和账上的数断言相等。这个断言比的是逻辑尺寸，不是驱动多配的那一份。

建法是 `mapped_at_creation: true`，用法里没有 `MAP_WRITE`：参数是 `UNIFORM`，输入和描述是 `STORAGE`，两份结果是 `STORAGE | COPY_SRC`。回读是 `MAP_READ | COPY_DST` 且 `mapped_at_creation: false`。wgpu-core 30.0.1 在 `mapped_at_creation` 且没有 `MAP_WRITE` 时调用 `StagingBuffer::new`，尺寸是对齐后的 `aligned_size`。这份暂存的用法是 `MAP_WRITE | COPY_SRC`。`unmap` 把暂存收进队列的 `pending_writes`，并在那里记下目标缓冲；注释写明用过的暂存要等对应提交做完才放。下一次 `queue.submit` 前的 `pre_submit` 才把这些暂存放进在途编码器。

wgpu-hal 30.0.1 的 Vulkan `create_buffer` 把 `MAP_WRITE` 且非 `MAP_READ` 标成 `MemoryLocation::CpuToGpu`，两边都没有则是 `GpuOnly`。设备缓冲走 `GpuOnly`，落在设备本地堆。gpu-allocator 0.28.0 对 `CpuToGpu` 先要 `HOST_VISIBLE | HOST_COHERENT | DEVICE_LOCAL`；这种类型不存在，或这次分配失败，才退到只要求主机可见、主机一致的类型。暂存描述里的 `TRANSIENT` 在这份 wgpu-hal 里只参与纹理，不参与缓冲，所以不会因此改成惰性分配。

因此从 `create_buffer` 到这次内部拷贝做完，设备缓冲和暂存可以同时在设备本地内存里。账上的两份回读通常是 `GpuToCpu`，不在设备本地堆，却算进了账。净下来，切到上限的一段，设备本地的峰值比账上多出映射建出来的那几份输入（参数、输入、描述，以及两份结果相对回读多出来的差）。输入是大头。`background_gpu.rs` 自己把那块小的主机可见设备本地堆（注释举 256 MiB）当成同一片显存的一部分，总量取最大堆、不求和。暂存落进这一窗时，占的是这同一片显存，账上没有这几字节。

同一条还盖住建到一半就显存不足的重切。前面几份已经 `unmap` 的缓冲，暂存和目标缓冲的 `Arc` 留在 `pending_writes`。`buffer_drop` 只把缓冲从 hub 摘掉再 `unmap`，不在这一步销毁 hal 缓冲。产品在任何建缓冲错误上立刻 `release`，然后把界压到建不成的那一段的一半再 `hold`（`outcomes_of_a_batch_over_the_cards` 里 `bytes_held_by_the_chunk / 2`）。下一次成功的 `queue.submit` 之前，失败那次还在 `pending_writes` 里，新的一次又建上一份。账上只看见新的一段。

触发要同时满足：给了上限，一段按剩下的界切满，暂存的优先分配成功落在设备本地堆。优先分配失败、整份退到非设备本地的主机内存时，这一份不算进显存，G04 不成立。本轮没有在真卡上看这次分配落在哪一堆。

## G03：重复条目键仍是 assert

`reconcile_the_two_kernel_outcomes` 用 `HashSet::insert` 的结果做 `assert`，失败信息是「同一个条目在一批里出现了两次」。注释写这是调用方的 bug。GPU 出错那一支走 `reconcile_unless_the_batch_failed`，计数不动；重复键走不到那一支，线程先 panic。

区域扫描的键是盘、槽、跨度。`records_starting_in_the_region` 按槽排序，不按这个三元组去重。挂载后的分类只把 `unit_bytes_read_for_the_span` 认得出的跨度送进 GPU：跨度不是单元足迹时，类在进 GPU 之前就已经写成读不了头，不进这一批。恢复的 `allocation_records_fit_the_pool_geometry` 拒绝两条跨度大于 0 的记录罩住同一个槽。校验和与区域分类都是片接回之后才进比对，一段里同一个单元至多一片。活槽的键来自整段列表，比对前会断言与这份列表相同。本包测试 `the_same_entry_twice_in_one_batch_panics` 直接构造重复键。产品扫描路径上没有找到会把同一个键送进同一次比对的组法。键一旦由调用方重复送入，表现是整理线程中断，而不是这一批按 GPU 出错停下。

## 已登记的欠账，现行代码里仍在

- C923：单卡 5 分钟的线程等已经写上（`outcome_of_opening_a_gpu_within_the_limit`）。欠账行要求的「真卡之外也能起、编不完的编译」那条单测，本轮没有在树里另找一条新的验收。G01 是这条等到点之后的析构，不把 C923 销掉。
- C926：等 GPU 的默认 10 秒仍没有在本包里重测。本包测试和 e202 引用这个默认；默认值本身在核心层。欠账仍是「没量过」。
- C952：`the_attempt_reached_the_gpu` 只把 `OutOfMemoryCreatingTheBuffers` 当成没交到卡上。`WaitForTheGpuExceededTheLimit` 算「交到了」，重交计数加一。建缓冲时错误范围弹出若交回等超时，一次没 `queue.submit` 的尝试仍会计进 `resubmissions`。停止条件看的是超时次数越过 3，不看这个计数字段，所以停卡的次数不会因此少一次；报出去的重交次数可以比注释写的「只数真交出去的」多。wgpu 30.0.1 Vulkan 上 `pop_error_scope` 是立即就绪的，这条要时钟在弹出之前已经过了期限才碰得到。欠账里写的洞还在，本轮没有把它跑出来。
- C996：`gpus_opened_at_the_mount` 按枚举出的每一张独显开设备，不按参与名单过滤。`gpu_opener_of_this_process.rs` 文件头写明没参与的卡也开。点了名时，没点名的卡开失败只记进 `not_opened`。没点名的卡在装得下最小片（上限为 `None` 时总是这一支）时还会跑代价探针。欠账仍是「开了没分给这次任务的卡」。

## 对过、与已有定法一致

**没给 `background_gpu_devices` 时，一张独显开失败就拒绝挂载。** `gpus_opened_at_the_mount` 第 800–803 行的注释把这写成定法：参与的卡没开成就 `GpuUnusable`，后面的卡不再开；给了列表而没点名的才跳过。`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 45 批交回那一行写的是：给了列表时没点名而开不了的跳过，没给列表时全部独显都算、照拒。条款句子里的「没点名的卡不用」按这一定法读，指的是给了列表却没点到的卡。本轮不把「不点名则任意一张失败就拒挂载」再算成一条新的违反。

**切换到「不点名」时，挂载时没开成的卡仍在清单里，核心层拒绝这次切换。** `DiscreteGpusAfterTheOpening::devices_with_total_memory` 把开成的和没开成的合成一份。核心层 `admit` 路径在参与卡属于 `devices_not_opened_at_the_mount` 时交回 `ParticipatingGpuDeviceNotOpenedAtTheMount`（`background_compute.rs`）。第 45 批同一行写明：切换点到挂载时没开成的卡，按第 4 条拒切换。

**按卡填满。** `plans_of_a_batch_filling_the_gpus_in_order` 按参与次序。前面的卡拿到的前缀，其提交次数等于只放第一条时的提交次数（`entry_count_filling_the_gpu`：二分找最长的、提交次数仍等于这一条的前缀）。最后一张拿剩下的全部，可以多次提交。一条自己就要交多次时（区域扫描的头片之后再加映射片），前面的卡停在这一条的提交次数上，不是再往后串行铺满，也不是一律只交一次。`room_of_the_position` 在一次调用里每张卡至多问一次。量出来的代价不参与条数。装不下的原因是显存上限时，这张卡从这一批拿掉，整批用剩下的卡重填。装不下的原因是绑定上限时，交回 `GpuCannotHoldTheSmallestPiece`。这和 2026-11-02 定的「按卡的顺序填满、不用代价分活」是同一条实现。

**超时次数。** `RESUBMISSIONS_AFTER_A_TIMEOUT` 是 3。`note_a_wait_past_the_limit` 在 `timeouts > 3` 时交回 `UnusableAfterResubmissions`。一次提交自己等超时，和交活前再等上次超时的提交，都进同一个计数。一段一次调用里各片累加；三段各有各的计数。挂载时的代价探针不经过 `outcome_resubmitted_after_timeouts`。全超时的那一条是：原提交一次，加上 3 次真重交，第 4 次等超过上限停卡。

**一段失败不改已确认的计数。** `outcomes_of_a_batch_over_the_cards` 对非显存不足的失败整段 `Err`。`reconcile_unless_the_batch_failed` 在 `Err` 时不更新计数。显存不足会把这张卡的房间减半并重切，这张卡上这一段已经做完的片不留。

**`poll` 的 `Ok` 当成做完。** `completion_check_of_the_submission` 把 `Device::poll(PollType::Wait { submission_index, timeout })` 的任何 `Ok` 当成完成，`PollError::Timeout` 当成还没完。wgpu-core 30.0.1 `resource.rs` 在这次等待的提交已经结束时交回 `Ok(QueueEmpty)` 或 `Ok(WaitSucceeded)`，否则 `Err(Timeout)`。对这个 `Wait` 形态，把全部 `Ok` 当成完成与这份 wgpu-core 一致。产品路径用 Vulkan。

**换设置时把晾下的卡请回来，量代价失败会让随后每一批整份失败。** `use_the_settings_from_the_next_batch` 把失败的卡放进 `positions_left_out`，同时记下 `cost_not_measured_at_the_last_switch`。`batch_not_split_over_the_gpus` 只要这个字段有值，就交回 `CostNotMeasuredAtTheSwitch`，其余还能用的卡也不分活。同函数注释写的是：量出错的这张卡不参与，之后每一批报成它的 GPU 出错，直到再换一次设置。这比「只用其余的卡」更严，和这段注释一致。量代价这一下不重交，和「挂载那一刻量代价的提交超时不重交」同向。

**进程里 Vulkan 实例新建到 4 次就拒。** `VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST` 是 4。`forget_the_instance_so_the_next_opening_creates_one` 丢掉实例、不减计数。这是条款列举的拒因之外多的一条，文件写明出路是重启进程。第 4 条列出的那些条件仍然会拒；条款没有写「只许这些条件拒」。

**PCI 次序。** 不点名的清单按 `pci_bus_address` 字符串排序。wgpu-hal 30.0.1 Vulkan 后端把域、总线、设备打成定宽十六进制，函数是一位十进制。这种字符串的字典序与数值序一致。空地址：参与的卡让整份清单失败，没点名的卡记进跳过名单。

## 对过、不单列成问题

这些是本轮读过的形状，没有跑成反例，也不写成「核已经证明正确」。

- 两份核在同一次 `queue.submit` 里各算一遍，结果缓冲分开。表驱动校验和核的 workgroup barrier 在提前返回之前。
- 校验和的 CPU 参照调用核心层 `crc32_castagnoli`。两份 GPU 核都是 CRC-32C，多项式 `0x82F63B78`，初值全 1，末尾异或。两份算法不同；两份一起错仍会互相确认。条款第 8 条把「两份共用的准备与同一种误读」写成接受的残余，测试对拍负责。
- 区域分类的 CPU 参照调用 `scan_region_over_the_identity_index`。键表按键长、7 个按小端补零的字、再按条目位置排序，与搜索核的 `ordering_of_the_entry_key` 在头键长度为定长时是同一种全序。解码只读判定字规定的字段。
- 活槽数按记录下标切段，段与段不重叠，各段计数相加。`record_table_bytes_of` 在至少一条记录时是线性的，`records_per_slice` 的除法与 `fits` 用的是同一套字节数。打包写出的释放字只有 0 和 1。
- 核心层在校验和与 GPU 判定不一致时，只增加 `units_whose_checksum_mismatched_on_the_gpu`，不改这条记录的分类。发布路径上的隔离在核心层按读盘校验和做。这是 `region_scan.rs` 的行为，不是这个包里的着色器缺陷。
- `unsafe` 仍由 `#![deny(unsafe_code)]` 卡住。`lib.rs` 写明五处：本包外壳四处（Vulkan 内存堆、`CLOCK_THREAD_CPUTIME_ID`、`CLOCK_MONOTONIC_RAW`、测试里的 `setrlimit`），加上真卡校验和测试里读空闲显存的一处。e202 自己是 `#![forbid(unsafe_code)]`。本轮没有重审 `.claude/gate.d/unsafe-allowances.tsv` 的登记行是否与这五处逐字对应。

## 明确不声称的事

- 没有声称这个包没有别的缺陷。e202 约 11643 行和 `tests/` 没有逐行读。
- 没有跑库测试，没有跑真卡测试，没有跑 e202。上面的 SHA-256 只说明读码当时源码树的摘要，不说明测试通过。
- 没有改 C923、C926、C952、C996 的欠账行，也没有新开 C 号。
- G01 没有在驱动里复现。它的依据是：失败路径上的 `?`、`GpuInUse` 没有泄漏用的 `Drop`、以及 wgpu-core 30.0.1 `Queue::drop` 的源码。
- G04 没有在真卡上看暂存落在哪一堆。优先分配退到主机内存时，这条不成立。
