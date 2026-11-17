# m4-gpugc-card-link-check-code-r1 攻方腿（Opus）报告（2026-11-13）

攻击面：正文第三节 T2 八处、T3 五类，检查单八格。被判代码读冻结副本 `~/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/`（六份被判文件的 sha256 与 `research/prompts/m4-gpugc-card-link-check-code-r1-snapshot/crates-sha256.txt` 逐行相同，开工时 `sha256sum` 现核）。下文引代码写冻结副本的绝对路径加行号（交回闸照项目根核引文，写成相对的 `crates/…` 会落到主工作区还没合入的旧代码上）。没写模型、没建副本：全部靠读码给调用序列。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T2-1 卡当场报出错只算 `UncapturedErrorOrDeviceLost` | 站得住（这一处的选择）；旁及一处早已有的缺陷 | 两种不读的原因在外壳里走不到「卡报错」；但 Vulkan 在等提交时报设备丢失，wgpu 30 当场 panic，读数时刻三在这条路上走不到（不是这一批引入的） |
| T2-2 换设置时量代价出错不读 | **打中** | 量代价那一次卡报了 `UncapturedErrorOrDeviceLost`，任务停下、每次报警，读数恒是没读 |
| T2-3 切换开卡线程上读、被盖掉也读 | 锁外读站得住；**打中**（叫停之后照读） | 切到 `cpu` 或卸载叫停开卡线程，一次都没真开、或只开不成一次的卡照样读一次，读数没人收 |
| T2-4 试满每交一次读一次 | 站得住 | 上界是「过了校验、到了下一批开头的切换请求」的次数，每次至多读一次 |
| T2-5 挂载报错里的读数是 Debug 文本 | 推不出 | 读数进了报错（字面满足）；「判硬件坏」那一判在这一路只能靠解析文本，条款没写报错要不要带判的结果 |
| T2-6 标识当不了目录名就不读 | 站得住 | 产品里走不到：空地址在清单那一步就拒，wgpu 的格式是十六进制加冒号加点 |
| T2-7 `enable` 按使能计数解析 | 站得住 | 与条款「使能位是关的」同义，认不得的不判坏 |
| T2-8 目录不在照读九份 | 站得住 | 多九次系统调用，读数都是文件不在；另有一个读目录与读文件之间的时序窗（推的，见末节） |
| T3 读的路径 | 站得住；最坏阻塞推不出 | 拼路径出不了设备目录；`read_dir` 不枚举；sysfs 属性有上限（推的）；内核侧会不会阻塞核不动 |
| T3 持锁 | 站得住；最坏阻塞推不出 | 六处读里四处在会话锁里（批里两处、交设置两处），一处在开卡线程的开卡互斥锁里，一处在挂载开卡工作线程上 |
| T3 行为不变（出错相等比较） | 站得住（核的范围见该节） | 产品代码里没找到拿出错、报警、停下记录比相等的地方 |
| T3 不带卡的停下 | **打中**（同 T2-2） | 停下的错点名了卡、原因是卡当场报出错，读数却是没读 |
| T3 再一次被判用不了 | 站得住 | 停着的 GC 不再走 GPU；换来的新错只经「答开不成」与「换上了」两处整条换掉，读数跟着换 |
| 检查单：走不到的理由二次查实 | 成立 | 六份产品文件非测试部分没有新加的 `todo!`、`unreachable!`、`assert!`、`panic!`、`expect`、`_ =>`，没有新加以 Undecided、Unsupported、OutsideTheFirstVersion 收尾的成员 |
| 检查单：类型与名字 | 成立 | 没有同一个量不同语义共用裸整数、没有新缩写、没有 `_ =>`，产品代码没有新加 `expect` |
| 检查单：不兼容旧代码与旧格式 | 成立 | 没有为旧调用方留的包装函数、缺省参数或 `Default` |
| 检查单：不新建判定库 | 成立 | 改动不写任何判定存放处 |
| 检查单：所有任务多机可用 | 成立（本机核过；第二台推的） | 固定路径在本机不在；新测试不读本机 sysfs |
| 检查单：测试优先多线程 | 成立 | 每条用例自己的目录（进程号、用例名、进程内序号）与自己的读者 |
| 检查单：带 --item 的命令与项 | 推不出（按规则算打中） | 只核得了 36 行变异行原文各命中一次；实现员命令在禁读清单里，`--list-items` 要跑门禁阶段，核不动；另外打中的两格没有变异行钉住 |
| 检查单：新崩溃流多罩了什么 | 成立 | 没有新崩溃流；验证包四份测试只给构造补字段 |

## 一、T2：实现员自己选的八处

条款原文在 `.claude/kb/decisions/26-后台整理与放置回收.md:253`：「这一刻有三种：挂着时一批在同一张卡上等超时、到判那张卡用不了；开卡重开用完都开不成；卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下。」

### T2-2　换设置时量代价出错不读：打中

调用序列（读码推的，没跑）：

1. 挂载选 `gpu`，两张卡「0000:01:00.0」「0000:41:00.0」都参与；之后一次切换给「0000:01:00.0」一个装不下最小一片的上限，它照开、被晾下、不量代价（第 3 条）。冻结副本里的现成前缀就是这一段：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:9101` 起的 `card_at_one_whose_measurement_at_a_switch_timed_out`。
2. 用户再切换，给它装得下的上限，请它回来。交设置那一刻在会话锁里走 `put_the_settings_in_effect`：被请回来的卡量一次代价，`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3376`：「.and_then(|_room_left| card.measure_the_cost_of_one_submission());」
3. 产品的量代价走真提交：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2102`：「self.both_kernel_outcomes_of_a_checksum_chunk(&piece_units)?;」，这一次提交里 wgpu 报了没接住的错（或设备丢失回调先记下），外壳交回 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2251`：「return Err(GpuFailureCause::UncapturedErrorOrDeviceLost { reason });」
4. 这个原因原样记进「换设置时量代价出错」：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3380`：「.get_or_insert((card.discrete_gpu().identifier.clone(), cause));」
5. 两批之间那一问答换上了、带着它；核心层照它停下全部有 GPU 路径的任务（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu_compute.rs:779`：「.map(|not_split| core_failure_of_the_batch_not_split_over_the_gpus(&not_split)),」），而这一路报给核心层的读数写死成没读：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu_compute.rs:505`：「&PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,」；每一批的那一路同样写死，`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3467`：「// 换设置时量代价出错不是判一张卡用不了的那三种时刻（这张卡没参与）：不读。」
6. 结局：GC 停着，每次叫起报警，报的错是 `GpuFailed { device: 0000:01:00.0, cause: UncapturedErrorOrDeviceLost, … }`，`pci_state_of_the_gpu_judged_unusable()` 是 `NotReadForThisError`，`pci_state_readings_taken()` 不涨。

四句：

- 分不分辨臂：分。实现员的选法（这条路一律不读）与另一种选法（照原因判，`whether_the_cause_judges_the_gpu_unusable_in_a_batch` 判是就读）在这一格结局不同。
- 系统当时看不看得到判别它的东西：看得到。第 4 步手里就是那个 `GpuFailureCause`，与批里判读不读用的是同一个类型。
- 满足的是判据字面哪一个分句：第 4 条「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」。实现员的理由「这张卡没参与」只说了它在这组设置下不分活，不改变它当场报错、任务因此停下这两件事。
- 跑前条款给的改法：正文没给改法，不适用。

量代价出错的另两类原因（等账上那次提交没等到，`EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`；量代价那一次单次超时）不是「等超时用完」（那要一批里第 4 次），这两类不读站得住；中的只是原因属于时刻三的那一类。

### T2-3　切换开卡线程上读：锁外站得住，叫停之后照读打中

锁外：读在开卡线程上、持的是开卡那一步的互斥锁（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3204`：「let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(」，到 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3218`：「drop(opening_of_a_gpu);」才放），不持会话锁。站得住。

叫停之后照读（读码推的，没跑）：

1. 切换点到没开的卡「0000:41:00.0」，开卡线程起来，头一次开卡在开（或已开不成一次、正睡重开间隔）。
2. 用户切到 `cpu`（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2895`：「opening.call_off();」），或卸载（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:4016` 同一句）。
3. 余下几次开卡一次都不开，各记一个叫停：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1669`：「Err(opening_called_off())」；重开的循环把它们照原因收下（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:405`：「Err(cause) => causes_of_each_opening.push(cause),」），最后交回 `Err`。
4. 线程走到 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1695`：「Err(causes_in_the_thread) => {」，接着在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1702` 读这张卡：「.pci_state_of_the_gpu(&discrete_gpu.identifier),」。
5. 结局没人收：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3220`：「let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);」。读者的计数是克隆共用的那一个，`pci_state_readings_taken()` 照样加一。

叫停发生在头一次开卡开始之前的，这张卡一次都没真开就被读了；冻结副本里的用例 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:9407`（`openings_called_off_do_not_count_toward_the_openings_of_the_card`）走的正是这一段（头一次开不成、余下 3 次叫停），它在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:9441` 取开卡互斥锁之后读到的计数应当是 1（推的，用例没断言这个数）。

四句：

- 分不分辨臂：分。「重开用完才读」与「线程交回开不成就读」在这一格不同。
- 系统当时看不看得到判别它的东西：看得到。读的那一刻线程手里有 `called_off`（第 3 步读的就是它）。
- 满足的是哪一个分句：一个都不满足。「开卡重开用完都开不成」没发生（叫停的几次不算开不成，`opening_counts_toward_the_failed_openings` 也不把它记进次数），属于三种时刻之外多读。
- 改法：正文没给，不适用。

被新的切换盖掉的那一种（不叫停，线程照开完，结局交给更早那一次交来的设置，`take_in_the_outcome_of_the_opening` 不报给最近那一组）：重开确实用完了，字面算时刻二，读得站得住；只是那一份读数既不进报警也不进挂载报错，计数却加了一，计数分不出「读了而读数进了报警」与「读了而丢掉」。算轻，不单列打中。

### T2-1　卡当场报出错只算 `UncapturedErrorOrDeviceLost`：站得住，旁及早已有的一处

两种不读的原因在外壳里是怎么来的：

- `WaitForTheSubmissionFailed` 只来自 wgpu 报提交号不对：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2055`：「Err(wrong_index @ wgpu::PollError::WrongSubmissionIndex(..)) => {」。这是调用方给错了号，不是卡报错，不读站得住。
- `ReadbackMappingFailed`：外壳先查 wgpu 回调记下的错，查到就报成 `UncapturedErrorOrDeviceLost`（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2250`：「if let Some(reason) = fault_taken_from_the_record(&self.fault_reported_by_wgpu) {」），之后才等映射（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2258`：「Ok(Err(error)) => {」）。设备丢失与未接住的错都先被记下、先被认成时刻三，映射出错只剩没有卡报错的那一种。不读站得住。

旁及（不是这一批引入的，不判进 T2-1）：设备真在等提交时丢了，Vulkan 的等待返回 `VK_ERROR_DEVICE_LOST`，wgpu 30.0.1 的路是：

- `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/vulkan/device.rs:3071`：「Err(other) => Err(super::map_host_device_oom_and_lost_err(other)),」
- `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:885`：「let hal_error: WaitIdleError = self.handle_hal_error(e).into();」（`handle_hal_error` 先走 `lose`，`lose` 当场调设备丢失回调：同一文件 5424 行「device_lost_closure(DeviceLostReason::Unknown, message.to_string());」）
- `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/life.rs:154`：「_ => None,」（`WaitIdleError::Device` 换不成 `PollError`）
- `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/backend/wgpu_core.rs:1924`：「self.context.handle_error_fatal(err, "Device::poll")」（`handle_error_fatal` 是 `panic!`）

外壳等提交走的就是 `device.poll`（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2049`：「match device.poll(wgpu::PollType::Wait {」），这一路不在 `catch_unwind` 里；后台一趟在会话锁里跑（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-mount-runtime/src/lib.rs:615`：「let outcome = session.run_compaction_in_the_background_for_at_most_one_publish(」）。所以这条路的结局是持锁 panic、锁中毒、运行库报坏，不是「后台任务因此停下、报警」，读数时刻三在这条路上走不到。依据是我读过的 wgpu 30.0.1 源码（`Cargo.lock` 里 `wgpu` 是 30.0.1，现查）；真卡掉线时驱动的等待返回 `VK_ERROR_DEVICE_LOST` 还是成功，是「我印象里」，没量过。交主 agent 定要不要另记一笔账。

### T2-4　试满每交一次读一次：站得住

- 读在交设置那一刻、只读头一张试满的卡，读完当场记成开不成并返回，不起线程：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2863`：「if let Some(used_up) =」，读的那一句在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3132`：「.pci_state_of_the_gpu(&discrete_gpu.identifier),」。
- 一次交设置至多读一次：交设置那一刻读了就不再为这一组起线程；没读而之后在 `start_the_opening_for_the_settings_waiting` 里判试满的，那一组被取走、不再放回（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3089` 起）。
- 交设置的次数：核心层把一批开头之前的几次请求并成最后一次（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1936`：「同一批开头之前请求了几次，用最后一次」），每批开头交一次。

上界：停着之后调用方反复交同一组，读的次数 ≤ 过了校验、又到了下一批开头（或整理入口判停之前）的切换请求次数；没有不经用户动作自己重复的读。每一次读都在会话锁里（交设置在批开头），见 T3「持锁」。

### T2-5　挂载报错里的读数是 Debug 文本：推不出

- 拒绝带着读数：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/gpus_of_this_host.rs:932`：「BackgroundGpuOpenRefused::GpuUnusable {」，经开卡器变成一段文字：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:392`：「BackgroundGpuNotOpened::RefusedByTheOpening { description }」，运行库那一侧只有 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-mount-runtime/src/lib.rs:143`：「RefusedByTheOpening { description: String },」。
- 条款「挂载那一刻开卡重开用完、拒绝挂载的，读数写进挂载的报错。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）字面满足：九样读数与目录、地址都在文字里。
- 推不出的是另一句「只有卡不在总线上（那个 PCI 地址的目录不在了，或使能位是关的）才判硬件坏」（同一行）在挂载这一路落在哪：报警那一路拿得到带类型的读数，可以调 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:435`：「pub fn hardware_judgement(&self) -> Option<HardwareJudgementOfTheGpu> {」；挂载这一路只剩文字，判出卡不在总线上要解析 Debug 文本。条款没写报错里要不要带判的结果，所以既不判打中、也不判站得住。另记：产品代码里没有一处调 `hardware_judgement`（现查只在 `background_compute.rs` 与 `pci_state_of_the_gpu.rs` 的定义与用例里），这一判在两条路上都只是可调的函数，归正推腿的 T1，不在这里判。

### T2-6　标识当不了目录名就不读：站得住

产品的标识只来自 wgpu 的 `device_pci_bus_id`。wgpu-hal 30.0.1 的格式是 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/vulkan/adapter.rs:2262`：「"{:04x}:{:02x}:{:02x}.{}",」，只有十六进制数字、冒号与点；驱动不报时是空串，空串的独显在清单那一步就拒（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/adapter_inventory.rs:159`：「if adapter.pci_bus_address.is_empty() {」）。所以 `PciBusAddressNotADirectoryName` 在产品里走不到，是防御；它不读、报一个与「读不到」分开的成员、`hardware_judgement` 交回 `None`，不与条款「读不到的报读不到」冲突，也不改行为。计数把它也算一次读，文档写明了，不算错。

### T2-7　`enable` 按使能计数解析：站得住

内核写 `enable` 的是使能计数；0 解成关的，正对条款「使能位是关的」；非 0 开着；不是十进制数的报认不得、不判坏，合「读不到的不当成正常、也不当成 0」。本机两张 NVIDIA 卡现读都是 `1`（`cat /sys/bus/pci/devices/<地址>/enable`，量的）。

### T2-8　目录不在时照读九份：站得住

多九次打开失败的系统调用，九样都报文件不在，判坏只看目录不在那一条，读数与判据都不变。时序上另有一个窗口（推的，见末节「这条腿自己的限度」），不判进这一格。

## 二、T3：五类

### 读的路径：站得住；最坏阻塞推不出

- 出不了设备目录：标识先过 `pci_bus_address_usable_as_a_directory_name`（不是 UTF-8、空、`.`、`..`、带 `/` 或 NUL 的不拼，`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:131`：「&& !pci_bus_address.contains('/')」），拼出来的只有设备目录下一层；产品的标识本来就只有十六进制、冒号与点（T2-6）。设备目录下那一层本身是指向 `/sys/devices/…` 的链接，跟过去读是本意。
- `read_dir` 只打开、不枚举：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:167`：「match std::fs::read_dir(device_directory) {」，`Ok` 臂把句柄丢掉，不读目录项，不占内存。
- `fs::read`：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:178`：「match std::fs::read(path_of_the_file) {」。产品根写死是 `/sys/bus/pci/devices`，九份都是 sysfs 属性文件，一次读出的不超过一页（推的，按 sysfs 属性 show 的约定；本机 `stat` 报大小 4096，内容几十字节，量的）；那里没有管道、设备文件这类会一直阻塞的文件。
- 内核侧会不会卡住：`enable` 与三份出错计数读内存里的计数，链路四样要读设备的配置空间（我印象里的内核实现，本机没有内核源码，核不动）。卡正掉线、链路不通时一次配置空间读要等多久，推不出。卡住时持的锁见下一格。

### 持锁：站得住；最坏阻塞推不出

六处读各在哪：

| 读的地方 | 持什么 |
|---|---|
| 批里在卡上出错（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3694`） | 会话锁（后台一趟在锁里跑，`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-mount-runtime/src/lib.rs:613`：「let mut locked = self.lock_the_session()?;」） |
| 批里交活之前那一等用完（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3555`） | 会话锁 |
| 交设置那一刻试满（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3132`） | 会话锁（批开头交设置） |
| 起不了开卡线程到上限（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:3263` 起） | 会话锁 |
| 切换开卡线程（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1650`、`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:1702`） | 开卡那一步的互斥锁，不持会话锁 |
| 挂载开卡重开用完（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/gpus_of_this_host.rs:932` 起） | 开卡器的工作线程，还没有会话 |

在会话锁里的四处：读完才放锁，前台这期间排队。平常一次读是几次系统调用（推的）；同一把锁里本来就有等 GPU 的时长（一批在一张卡上至多等 4 次上限），读的那一点相对它可忽略（推的）。别的卡的批：同一把会话锁里一批是串着的，读在出错那一刻、这一批已经放弃，不拖别的卡这一批的活。最坏情形取决于上一格推不出的那一项。

### 行为不变（出错的相等比较）：站得住

核的范围与做法：冻结副本 `crates/` 下每一份 `/src/` 里提到 `BackgroundComputeError`、`BackgroundGpuComputeFailed`、`GpuFailureCause`、报警、停下记录、`LatestSettingsHandedIn`、`LatestGpuSettingsHandedIn`、`GpuNotOpenedAfterTheReopenings`、`BatchNotSplitOverTheGpus`、`BackgroundComputeOfThisMount`、`BackgroundComputeOfOneBatch` 的文件，只看头一个 `#[cfg(test)]` 之前的行，找 `==`、`!=`、`assert_eq!`、`assert_ne!`、`.contains(&`、`dedup`、`HashSet<`、`.eq(`、`.ne(`。命中的都不是拿出错、报警或停下记录比相等：停下记录只按 `stopped.task == task` 找（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1708`），批末那一处 `assert_eq!` 比的是设置（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:2109`），开卡结局按交设置的次序号比（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs:2963`）。读数是 `Clone` 出来的同一份，停着之后每次报警都从停下记录现取（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1536`：「.pci_state_when_judged_unusable()」），不会两次不同。停下、报警次数、补跑、解除停下的判定都只看成员种类与任务，不看读数。

这一格的「没找到」是一次抽样的否定结论：写法不在上面那几个词里的比较（例如把整个会话状态拿去比）扫不到。

### 不带卡的停下：打中（同 T2-2）

逐种停下看：

- `GpuUnusableUnderTheSettingsHandedIn` 带「参与的卡因显存上限一张都不剩」：没有一张卡被判用不了，没读站得住。
- 空闲显存不够、输入不支持、只供测试的钩子注入：不是三种时刻，没读站得住。
- `GpuUnusableUnderTheSettingsHandedIn` 带「换设置时量代价出错」而原因是 `UncapturedErrorOrDeviceLost`：错里点名了卡、原因属时刻三、任务因此停下，读数却是没读——就是 T2-2 那一条序列。

### 再一次被判用不了时读数换不换：站得住

- 停着时又来一次批里的出错，停下记录不换：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1748`：「if !self.is_stopped_after_a_compute_error(task) {」。这一支在产品里走不到：今天只有 GC 有 GPU 路径（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1884`：「BackgroundTask::CompactionRelocation => {」），停着的 GC 要么不开这一批、要么在 CPU 上补跑，不再交 GPU。
- 换错只经两处、整条换掉、读数随新的错走：答开不成（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:2036`：「self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));」，读数是新卡判开不成那一刻读的）与答换上了（同一函数里 `PutInEffect` 那一臂）。
- 过了校验的切换当场解除（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1962`：「self.replace_the_tasks_stopped(Vec::new());」），之后同一张卡再在批里出错，现读一份新的。条款「这张卡再一次被判用不了时才再读一次。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）成立。

## 三、检查单八格

1. **走不到的理由二次查实：成立。** 做法：从改动全文取出两份补丁里每一行新加的文字，与冻结副本六份产品文件头一个 `#[cfg(test)]` 之前的行逐行对；产品部分里 `todo!`、`unreachable!`、`assert!`、`assert_eq!`、`panic!`、`.expect(`、`.unwrap()`、`_ =>` 的每一处都不是这一批新加的（新加的 `.expect(` 共 6 行，全在 `pci_state_of_the_gpu.rs` 224 行之后与 `gpus_of_this_host.rs` 1017 行之后的测试模块里）。`background_compute.rs` 里以 Undecided 收尾的两个成员（1042、1044 行）不在改动全文里，是早有的。
2. **类型与名字：成立。** 新加的整数只有 `enable_count: u64`（内核的使能计数）与读的次数（`readings_taken`、`pci_state_readings_taken`，同一个量、同一语义），没有同一个量两种语义共用一个裸整数；新标识里只有 `pci` 一个缩写，已登记（`.claude/abbreviations` 第 9 行）；文件名 `max_link_speed`、`aer_dev_*` 只作字符串数据，字段名都写全了（`maximum_link_speed`、`advanced_error_reporting_*`）；没有 `_ =>`；产品代码没有新加 `expect`。
3. **不兼容旧代码与旧格式：成立。** 新加的公开函数逐个列过（改动全文里 `+` 行的 `fn`），没有为旧调用方留的旧签名包装、缺省参数或 `Default`；`NotReadForThisError` 是「这个错不是读的时刻」，不是兼容成员。
4. **不新建判定库：成立。** 产品代码只读文件；测试只在临时目录建自己的假设备目录、丢的时候删掉。
5. **所有任务多机可用：成立（本机核过，第二台推的）。** `ls -la /singlefs-background-gpu-tests-pci-device-directories-never-created` 在本机报 No such file or directory（量的）；第二台核不动，建它要超级用户（推的）。新测试都用替身读者（不在的根或用例自己造的临时目录），产品读本机的那一份只在真卡出错时才读；`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu-tests`、`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/tests`、`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-verification/tests` 里没有拿 `GpuUnusable`、`*BatchFailed::GpuFailed` 这类带读数的出错比相等的真卡用例（`grep` 现查，零命中；改动里加的都是 `NotReadForThisError` 的构造）。
6. **测试优先多线程：成立。** 每条用例的目录名带进程号、用例名与进程内原子序号（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs:277`），每条用例自己建读者、计数在那一份读者里；没有共用的静态读者（`grep` `static.*PciStateReader`、`LazyLock`、`OnceLock`、`thread_local` 零命中）。
7. **带 --item 的命令与项：推不出（按规则算打中）。** 核了的：`crates/mutations.tsv` 里这一批新加的 30 行与改动过的 6 行（含 `pci_state` 的那几行），原文在冻结副本里各恰好命中一次（python 逐行数，量的）。核不动的：实现员报告里带 `--item` 的命令（在禁读清单里）；新项能不能被 `--list-items` 列出（要跑门禁阶段，子 agent 不跑）。正文说主 agent 在快照上点名跑过，这一格由主 agent 用它的任务号定。另：T2-2、T2-3 打中的两格没有变异行钉住（现有行都钉「该读的读了」「不该读的批不读」，没有「量代价出错原因属时刻三时读」「叫停之后不读」）。
8. **新崩溃流多罩了什么：成立。** 改动全文里验证包四份测试只给构造补了读数字段（第二份补丁又按装箱改了写法），没有新的崩溃点重放流与用例；读数只在内存里。

## 四、改法（只在读码上推的，被攻过零轮，没量过）

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 甲：`put_the_settings_in_effect` 量代价出错时，原因照 `whether_the_cause_judges_the_gpu_unusable_in_a_batch` 判是就当场读一次，读数跟着「换设置时量代价出错」存下、经 `BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch` 带到核心层（每一批带同一份、不重读） | T2-2、T3 不带卡的停下 | 推的，待主 agent 跑 |
| 乙：开卡线程交回开不成之前看 `called_off`，置上了就不读（结局没人收） | T2-3 叫停之后照读 | 推的，待主 agent 跑 |
| 丙：挂载报错的文字里连同读数写出 `hardware_judgement` 的结果 | T2-5（若主 agent 判条款要它） | 推的 |

乙不碰「被新的切换盖掉」那一种（那里重开确实用完了）；甲、乙都不改行为，只改读不读与读数带不带。

## 五、没打中的形状

- 一批里两张卡都出错：分活循环在头一张出错的卡上就跳出（`break 'handing_out`），只读一张，不多读。
- 停着时拿到一个新错又被「出错之前已请求的切换」解除（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:1742`）而白读一次：批在会话锁里跑、切换请求也要会话锁，批开头 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-core/src/background_compute.rs:2064` 已把待生效的取走，批里到出错之间请求进不来；产品里造不出这一段历史（推的）。
- 同一份读数前后两次报警不同：读数存在停下记录里、报警现取克隆，造不出。
- 超时重交到第 4 次之前夹一次非超时出错：照原样交回，原因是卡当场报错的读、不是的不读，与条款一致。
- 挂载开卡的外层期限先到、报 `TimedOut` 而丢掉读数：期限是枚举期限加卡数乘单卡至多等的时长，枚举实际用掉的比枚举期限短，余量足够一次读（推的）。
- 标识指到别的设备（换过插槽、重新枚举后同一地址是另一块设备）：读数会是那块设备的；要判它得读 `vendor`、`device`，那在条款的九样之外，不算这一批的缺陷。

## 六、这条腿自己的限度

- 全部结论靠读码，一条都没跑；两处打中的调用序列是推的，待坐实（见下节）。
- 内核侧读 sysfs 的耗时与阻塞、真卡掉线时驱动等待返回什么，都核不动：本机没有内核源码，没有真掉卡的取样（与欠账 C1024（掉卡判据没有真掉卡的取样） 同一个缺口）。
- 读目录与读九份文件不是一次快照：卡正被摘掉时，可能读到「目录在、`enable` 文件不在」，判不出硬件坏（推的时序窗，没造）。
- T3 行为不变那一格是关键词扫描，写法不在扫描词里的比较扫不到。
- 第二台上的固定路径没核。

## 要主 agent 跑的命令

无。两处打中若要在代码上坐实，要先建一份可编译的副本（冻结副本只带 `crates/`，没有仓根的 `Cargo.toml`），在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/crates/singlefs-background-gpu/src/background_gpu.rs` 的测试模块里加两条用例：一条在 `openings_called_off_do_not_count_toward_the_openings_of_the_card` 取开卡互斥锁之后断言 `background_gpu.pci_state_readings_taken() == 0`（推的结局：读到 1、判红）；一条照 `card_at_one_whose_measurement_at_a_switch_timed_out` 的前缀，把「0000:01:00.0」的 `cost_measurement_failing_with` 设成 `UncapturedErrorOrDeviceLost`，请它回来之后断言 `pci_state_readings_taken() == 1`（推的结局：读到 0、判红）。建不建副本、用例写进哪里由主 agent 定；这一条腿没建副本，所以不写交任务的命令。

## 没做什么

- 不判正推腿那几格（T1 逐句对条款）；不替主 agent 采纳。
- 没写模型、没建副本、没往统一任务服务交任务；判决要引的数都没复跑（本报告里标「量的」只有 sysfs 现读与原文命中计数两类，都是只读命令）。
- 没读禁读清单里的文件：另一条腿的报告、主工作区的 `crates/`、两个实现员草稿目录。
