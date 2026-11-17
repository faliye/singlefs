# bggpu-audit-fix-code-r1 攻方腿（Opus）报告（2026-11-12）

立场：假设三条修复（G01、G03、G04）与 25 份只改名的文件做错了，造输入或操作序列打穿。主攻 T1、T2、T3。
代码一律引冻结副本 `~/.local/share/singlefs/work/bggpu-audit-fix-code-r1/repo/` 里的路径与行号（这几份被引的产品文件与主工作区此刻逐字节相同，量的：sha256 比过）。
文中「量的」只指读码时用 grep / 只读 python 数出来的；模型的读数都还没跑，一律「推的，待主 agent 跑」。

## 复跑命令与模型文件

模型目录：`research/prompts/bggpu-audit-fix-code-r1-opus-model/`。复跑（交服务，命令原样在末尾「要主 agent 跑的命令」一节）：

`bash research/prompts/bggpu-audit-fix-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录，工作根下、以 -run 收尾> <模型目录>`

它先删那个运行目录，再起同目录的 `tools/rerun.sh`：把冻结副本 rsync 进运行目录（不带 target、.git），`tools/apply_model.py` 只在这份拷贝里改两份文件（替身卡加一个字段让量代价照产品走账；两处 `mod tests` 末尾插进模型用例），再在拷贝里 `cargo test -p singlefs-background-gpu --lib attack_model -- --nocapture`，把 `name=attack_model…` 行与 `out/predicted-patterns.txt` 逐条比（每条预测要恰好命中一行）。

| 文件 | sha256 |
|---|---|
| out/predicted-patterns.txt | 6493e3579c0b862c8e30d0b51eade743269d51a125e8bbf1103364c81d37e01f |
| patch/background_gpu_tests.rs.txt | 185c8b13300dcd72124b04aeacb6097bab16a5302a6d73caa4b8777342d8433d |
| patch/live_slot_tests.rs.txt | 322388fa6e1937acd49fa5227bf116477de1b9f79dca352dce28451193f8511a |
| tools/apply_model.py | 65602a98a4c33aedf1bcd1ccce77d8571da3a529d1be41adef8528f65760a78e |
| tools/rerun-from-a-clean-work-directory.sh | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |
| tools/rerun.sh | 9b8e78b09eb13a9b5416bba1ea2f538eb227189960846f29ad55b0f7e9c46a08 |

模型不碰盘、不录写流：这一题没有崩溃状态可枚举，第 3b 步的崩溃枚举那几条不适用（没有调 `enumerate_crash_states`，没有状态数）。模型三个用例都是替身卡与纯函数，单条用例内的取样量写在各节。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| T1 量代价出错的泄漏与账 | **打中一处（既有、这一批没碰，但落在 G01 的同一个判别子上）** | 切换时给「开卡没量过代价」或「被晾下又请回来」的卡量代价，不先等它账上那次还在跑的提交，直接 `hold`，断言 panic；量代价出错时的泄漏处置（G01 本身）在挂载与切换开卡两条入口上对得上 | T1 节；模型历史甲、乙 |
| T2 重复条目键 | **打中一处** | 选段打分那一段：重复的段先撞 `reconciled_live_slots_of_the_segments` 开头的整段清单断言，新加的 `LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch` 走不到；校验和、区域扫描两段对得上，计数与 CPU 参照一条没动，核心层不归卡、不重交 | T2 节；模型 live_slot 六格 |
| T3 暂存进账 | 三处口径成立；改小上限的推导成立；建到一半显存不足那一支没造出重复记账或早放，**最脆弱一句写在 T3 节末** | 空提交冲待写读 wgpu-core 30.0.1 源码成立；等不到冲刷时交回「等超过上限」，超时数与重交数都把一次空提交算进去（字面两读） | T3 节；模型暂存算术 20 万例 |
| T4 只改名 | 没打中 | 查了六类，登记表里只剩四处说明列写着旧名（不进行为） | T4 节 |
| 检查单：走不到的理由 | 不成立（一处） | `LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch` 走不到而没写理由；`hold` 的断言在切换量代价那条路上走得到 | 检查单节 |
| 检查单：类型与名字 | 成立（判的是现状：裸 `u64`） | 字节、条数都是裸 `u64`，位置是 `usize` | 检查单节 |
| 检查单：不兼容旧代码 | 成立 | 落点的 diff 里没有认旧版本、迁移、旧格式 | 检查单节 |
| 检查单：不新建判定库 | 成立 | 落点 diff 里没有新的存放处 | 检查单节 |
| 检查单：多机可用 | 成立 | 新用例只用替身卡的假总线地址，与基线同一批字面量 | 检查单节 |
| 检查单：测试优先多线程 | 推不出 | 没有新的切片枚举；单条用例该不该并行，读码定不了 | 检查单节 |
| 检查单：带 --item 的命令与项 | 成立 | 12 行新变异在冻结副本的 `crates/mutations.tsv` 里各一行、原文在点名文件里各命中 1 次（量的） | 检查单节 |
| 检查单：新崩溃流 | 成立 | 产品 GPU 包没有崩溃枚举用例、没有放量分项（量的） | 检查单节 |

## T1　开卡量代价出错：泄漏的判别子与账

### 打中：切换时量代价不先等还在跑的提交，`hold` 断言 panic（推的，模型待主 agent 跑）

历史甲（G01 同一族：量代价那一次提交等超过上限之后，下一次切换再量）：

1. 挂载时卡 X 的显存上限连最小的一片都装不下，照开、不量代价：「CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut => {}」（`crates/singlefs-background-gpu/src/background_gpu.rs:1357`）。X 的 `measured_cost` 是 `None`。
2. 用户切换，给 X 装得下的上限。`put_the_settings_in_effect` 里 X 不被晾下，量代价：「if card.measured_cost().is_none() || is_brought_back_after_being_left_out {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2849`）、「if let Err(cause) = card.measure_the_cost_of_one_submission() {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2850`）。
3. 产品的量代价照此刻的界切片、直接交校验和那一段：「.pieces_under_the_bounds(self.budget.room_left_now())」（`crates/singlefs-background-gpu/src/background_gpu.rs:1687`）、「self.both_kernel_outcomes_of_a_checksum_chunk(&piece_units)?;」（`crates/singlefs-background-gpu/src/background_gpu.rs:1702`）。那一次提交等超过上限：「Err(_cause_of_the_failed_wait) => Self::MayStillBeRunningOnTheGpu(submission),」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:374`），账上记着还在跑的那次（约 8296 字节：2×16 + 2×4096 + 2×12 + 6×8，推的）。X 不参与：「positions_left_out.push(*position);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2851`），`measured_cost` 仍是 `None`。之后每一批报 `CostNotMeasuredAtTheSwitch`，核心层停着、报警；核心层给这一类的出路是「调大上限，或查这张卡是不是卡死了」（`crates/singlefs-core/src/background_compute.rs:178`）。
4. 用户照出路再切换一次（例：调大 `background_gpu_wait_limit`，点的卡与上限不变）。条款许这样做：「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。晾不晾下按上限扣掉账上已占的判：「.saturating_sub(bytes_already_held_on_the_device)」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:447`），只要上限比「还在跑的那 8296 字节 + 最小的一片」大就不晾下；`measured_cost` 仍是 `None` ⇒ 第 2 步那一行又成立 ⇒ 再量。
5. 第 3 步那条路先 `hold` 再交，`hold` 第一句就断言没有还在跑的提交：「self.submission_running_past_its_wait.is_none(),」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:459`），原话「"这张卡上还有一次等超过上限、没做完的提交：交下一段之前先等它做完"」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:460`）。量代价那条路不经交活之前的那一等（批里切段之前走的是「let room = outcome_waited_for_again_after_timeouts(」（`crates/singlefs-background-gpu/src/background_gpu.rs:2981`）；量代价没调它）。⇒ 在 `use_the_settings_from_the_next_batch` 里 panic，那是核心层两批之间、会话锁里调的（「self.put_the_settings_in_effect(settings);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2430`））。

历史乙（不用 G01 那一支，挂载时量过代价的普通卡）：X 参与，一批里头一次交等超过上限、之后三回等前一次都没等到 ⇒ 报这张卡用不了，账上照旧记着还在跑的那次；用户切换给 X 一个扣掉已占之后装不下最小一片的上限（晾下）；再切换给一个装得下的上限 ⇒ `is_brought_back_after_being_left_out` 为真 ⇒ 量代价 ⇒ 同一处断言 panic。

用户动作放开扫（只固定前缀与故障，模型 `attack_model_sweep`，两段历史各 5 种第 4 步）：点两张卡、X 给 1 GiB；点两张卡、X 给「已占 + 最小一片」正好装得下；点两张卡、X 给少 1 字节；只点另一张卡；切到 `cpu`。预测（推的）：前两种 panic，后三种不 panic（少 1 字节的照旧晾下、不量；只点另一张卡与切到 `cpu` 的把 X 关掉，账上有还在跑的提交就整张泄漏）。对照用例：同一条请回来的路、账上没有还在跑的提交，量代价照常量成（替身卡这条路本身不 panic）。

模型怎么照产品：冻结副本里的替身卡量代价不走账（`cost_measurements += 1` 之后直接交回），所以库内单测看不见这一支；模型在拷贝里给替身卡加一个开关，量代价时照产品经 `chunk_submitted_on_the_cpu`（与替身卡交校验和那一段同一个函数）先 `hold`、照脚本超时。这一处改动只在模型的拷贝里。

四句：
- 分不分辨臂：这一格不判臂；它分辨「量代价之前等不等还在跑的提交」这一处，G01 的泄漏处置不管它在不在都一样。
- 系统看不看得到：看得到。`has_a_submission_running_past_its_wait()` 在那一刻就是真（G01 的判别子读的就是它），`room_once_the_submission_running_past_its_wait_completed`（`crates/singlefs-background-gpu/src/background_gpu.rs:1906`）现成可调。
- 满足判据字面哪一句：T1 问的「量代价之外、开卡过程里别的出错出口……有没有同样握着已提交的队列」——这里握着已提交的队列，没有照常析构，而是在下一次量代价时断言；条款压在「挂着切换给了它装得下的上限，从下一批起照新上限按卡的顺序重新参与分活」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`）与第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下」上：照条款该解除、该重新量，产品 panic。
- 跑前条款给的改法：这一轮正文没有跑前改法，只有 G01 的改法（量代价出错泄漏），它在第 5 步那一格上不起作用（第 5 步时卡没被丢，也不是量代价出错，是量代价之前断言）。

既有还是这一批引进：基线（提交 `8ed2ea484` 那一版）同一处就有 `is_brought_back_after_being_left_out` 与无等待的量代价（量的：基线 `background_gpu.rs` 第 1931–1933 行）；历史乙在基线上同样走得到。历史甲的第 3 步——量代价超时之后卡留着、账上记着还在跑——基线也一样。所以不是 G01 引进的；落在 G01 的同一个判别子与同一次量代价上，T1 题面问到了。

### 查过、对得上的

- 「还在跑」的标记只有一处来源：`settle_the_chunk_after_its_submission` 见到 `MayStillBeRunningOnTheGpu` 才记（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:374` 那一行把超时与别的等的出错都归到它）。交一段那条路（`dispatch_both_kernels_and_read_back` 之后每次都结账）与建缓冲出错那条路（「let flushing_submission = self.queue.submit(std::iter::empty());」（`crates/singlefs-background-gpu/src/background_gpu.rs:1737`），接着「settle_the_chunk_whose_buffers_were_not_created(」（`crates/singlefs-background-gpu/src/background_gpu.rs:1748`））都经它。判别子读的是同一个字段：「card_has_a_submission_running_past_its_wait,」（`crates/singlefs-background-gpu/src/background_gpu.rs:3484`）。读码没找到「提交交出去、等出错、而标记没记」的路：交出去之后到结账之间只有建绑定组之外的 `map_async` 与 `poll`，出错都进 `polled` 或回读那一步，结账照 `polled` 判（推的，读码）。
- 等到了提交、回读映射超时的：提交已做完，标记不记，卡照常析构——`Queue` 析构里的 `wait_for_idle` 此刻不会挂（推的，读 wgpu-core 30.0.1 `src/device/queue.rs` 第 275–312 行的 `impl Drop for Queue`）。
- 泄漏记账：量代价出错那一支与卸载、切换关卡用同一个 `card_leaked_when_dropped`：「bytes_held_on_the_device: card.budget().memory_account().bytes_held(),」（`crates/singlefs-background-gpu/src/background_gpu.rs:3469`）、「bytes_leaked_by_earlier_mounts: card.budget().bytes_leaked_by_earlier_mounts(),」（`crates/singlefs-background-gpu/src/background_gpu.rs:3470`），进程账按次累加差额：「Some((_device, bytes)) => *bytes += leaked.bytes_leaked_by_this_mount(),」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:811`）。之前挂载泄漏的那一片在开卡时从进程账现读、记进这张卡的账起点，泄漏时减回去，没重算也没漏扣（推的，读码）。
- 量代价之外的开卡出口：`started_gpu_in_use` 里起设备、编管线、`fault_taken_from_the_record` 交回出错都在任何提交之前；量代价切片不成（最小一片装不进绑定上限）也在提交之前。这几处丢卡时队列空，照常析构不挂（推的，读码）。
- 开卡线程外面那一层到点：线程照旧跑完；量代价出错那一支在线程上已经泄漏、记进进程账（「let _returned_when_this_ends = opening_not_returned;」（`crates/singlefs-background-gpu/src/background_gpu.rs:1181`），守卫在记账之后才丢）；开成了的卡结局送不出去、在线程上丢，那时量代价的提交都已做完（「let _nobody_waits_for_this_opening_any_more = outcome_sender.send(open());」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:298`））。账记得上；晚到的泄漏只影响这张卡下一次开卡读到的起点，而同一张卡上一次没回来时下一次不起（守卫按卡），同一次挂载内对得上。

## T2　重复条目键：交回的是不是「这一批出错」

### 打中：选段打分那一段的新成员走不到，重复的段先撞断言 panic（推的，模型待主 agent 跑）

1. 选段打分一批的比对开头先断言交进来的段就是单元区的整段清单：「"丙段交进来的是单元区的整段、按起点升序、一段一个"」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:454`），之后才「uncertainty.reconcile_unless_the_batch_failed(both_kernel_outcomes, |entry_position| {」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:471`）。
2. 整段清单按起点严格升序、一段一个，所以任何重复的段都先让第 1 步断言失败；查重复键那一句「if let Some(repeated) = first_entry_key_repeated_in_the_batch(&entries) {」（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:132`）在这一段永远走不到；`LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch` 与它在 `background_gpu_compute.rs` 的那一臂（「LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(repeated) => {」（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:565`））是死成员。
3. 断言在第 1 步、两份核的结局（交成或出错）都还没看：两份核出错的那一批也先 panic，GPU 已经在各卡上交过、等过（`both_kernel_live_slots_of_the_cluster_segments` 在比对之前跑完）。
4. 冻结副本里已有一条用例把这句断言当成约定：`segments_other_than_every_full_segment_of_the_unit_area_panic`（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:703`）。核心层只交整段清单：「let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);」（`crates/singlefs-core/src/mounted_session.rs:5292`），所以今天从核心层走不到重复；打中的是「三段都改成交回出错」这句在选段打分那一段不成立——G03 的本意是「调用方的 bug 不 assert」，这一段仍是 assert。

模型（`attack_model_a_repeated_segment_meets_the_full_list_assertion_first`）：三种重复摆法（中间那段顶掉末段、末段再接一次、首段接在末尾）× 两份核交成 / 出错，共 6 格，预测（推的）6 格都 panic、消息是整段清单那一句。

四句：
- 分不分辨臂：不判臂；它分辨「重复键在选段打分那一段交回出错」这一句的真假。
- 系统看不看得到：看得到（段清单就在参数里）。
- 满足判据字面哪一句：T2 正文「选段打分那一段在比对之前另有一处键与整段清单相等的断言，它与新出错谁先到」——断言总是先到，新出错到不了。
- 改法：正文没给跑前改法。

### 查过、对得上的

- 校验和、区域扫描两段：比对之前没有键的断言，重复键在动任何计数之前整批查（第 132 行那一句在 `for` 之前），计数、连续不确定轮数、CPU 参照都不动；冻结副本的用例 `the_same_entry_key_twice_in_one_batch_is_an_error_leaving_the_counts_and_never_asking_the_cpu` 罩着（量的：读到用例，没跑）。
- 不会被当成卡坏去重交：查重在两份核都交回之后，重交与超时计数都在交 GPU 那一层（`outcome_resubmitted_after_timeouts`），已经结束；`From` 只包一层（「.map_err(BatchFailure::from),」（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:116`））。
- 报给核心层不归卡：「BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. } => Vec::new(),」（`crates/singlefs-core/src/background_compute.rs:259`）；位置一对一照搬、没报反：「position_of_the_first_occurrence: repeated.position_of_the_first_occurrence,」（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:477`）。核心层对它与别的出错同一种处置（这一批放弃、意图留盘、任务停下报警），不是吞掉；停下不点名卡，也就不会「停掉一张好卡」。条款第 4 条「挂着的时候 GPU 出错（含重交 3 次都超时），正在算的那一批放弃」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）说的是 GPU 出错，重复键不是 GPU 出错，核心层照同一条路停下——这是把调用方的 bug 当成「这一批出错、停下等人」，与用户 2026-11-11 选的「改成交回错误」相容，停不停由核心层定（推的，读码；条款没写重复键该停还是该只放弃这一批，判不准）。
- 位置是相对哪一份清单：校验和与区域扫描两段的条目按核心层交的 `units` / `records` 次序一一建，位置就是核心层那份清单里的位置（推的，读码）。

## T3　暂存进账：口径在每一处是不是同一个数

### 三段三处的份数（读码，推的；模型的独立核只罩切批算术）

系数：「pub const COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION: u64 = 2;」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:50`），「2 * COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION + 2;」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:54`）。外壳每一份缓冲都经 `buffer_with_contents`（`mapped_at_creation: true`，用法里没有 `MAP_WRITE`），两份回读暂存是 `MAP_READ | COPY_DST`、不映射着建。

| 段 | 映射着建的缓冲（各 2 份） | 不映射着建（各 1 份） | `ChunkFootprint` | 外壳 `bytes_held_by_this_chunk` | 真建尺寸断言 |
|---|---|---|---|---|---|
| 校验和 | 参数、输入、描述、两份结果 | 两份回读 | 每段 2×参数；每条 2×描述 + 6×结果；输入经 `bytes_held_by_a_chunk` 算 2 份 | 2×(参数+输入+描述) + 6×结果 | 每份映射着建的 ×2、回读 ×1，与账比 |
| 区域扫描（头片与映射片同一个函数，头一片之后的段交空映射） | 参数、输入、描述、条目表、落点索引、按 key 排的序号、两份结果 | 两份回读 | 每段 2×参数 + 2×三张表；每条 2×描述 + 6×结果 | 同左 | 同上 |
| 选段打分 | 参数、段上下界（当输入）、记录表、按（盘、槽）排的序号、两份结果 | 两份回读 | 每段 2×(参数+记录表两缓冲)；每条 6×结果；上下界经输入算 2 份 | 同左 | 同上 |

三处在三段上都是同一个数；没有哪一份映射着建而只算一份，也没有不映射而算两份。外壳在 `hold` 之前用 `assert_eq!` 把自己算的与 `ChunkFootprint` 比（三段各一处），建完再用 `buffer.size()` 比一次，三处任何一处对不上会 panic、不会带着少记的账往下交。打包是连续拼接、整段末尾补到 4 字节（`packed_checksum_chunk`、`packed_region_record_chunk`），所以「对齐后的整段输入」与外壳建的输入缓冲一样长。

### 改小上限交给被验函数之后推不推得出「连同暂存不超过原上限」

- 切批判装不下：交给被验函数的上限是 L − 对齐后的输入（「memory_limit.0.checked_sub(staging_of_the_input)」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:174`））。被验函数的 `ensures` 给的是「Some(limit) => crate::gpu_gc_proofs::aligned_input_size(input) + entries * capacity.bytes_per_entry + capacity.fixed_bytes <= limit,」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:23`）。两边加回一份对齐后的输入，正好是 `bytes_held_by_a_chunk ≤ L`：推得出，而且是当且仅当（不多拒）。边角：对齐后的输入超出 `u64` 或比 L 大 ⇒ 判装不下；不给上限 ⇒ 只判绑定与派发。
- 最大的一片：交的上限是 (F+P) + ⌊(L−F−P)/2⌋（「/ COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:286`））。被验函数交回的 b 满足 b ≤ ⌊(L−F−P)/2⌋ 且是 4 的倍数 ⇒ 2b + F + P ≤ L；又有「&& forall|input: u64| #[trigger] chunk_fits(input, 1, *capacity) ==> input <= bytes,」（`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs:58`）⇒ b + 4 时 2(b+4)+F+P > L，b 是连同暂存装得下的最大 4 的倍数。交回 `None` ⇔ L − F − P < 8 ⇔ 最小的一片（输入 1 字节补到 4，连同暂存 8）装不下：两边一致。L < F+P ⇒ 下溢由 `checked_sub` 接住、交回 `None`。
- 模型独立核（`attack_model_staging_arithmetic_sweep`，不经这两处折算，只拿 `bytes_held_by_a_chunk` 比）：线性同余 20 万例，F∈[0,300)、P∈[0,120)、L∈[0,6000)、单条输入 [0,4000)、一批 1–5 条；绑定与派发上限放到 2⁴⁰ 不起作用（只罩显存那一维）。预测（推的）：判装不下与算术不一致 0、切出来的段超上限 0、切开的非末片不是最大 0、拒得不对 0。

### 建到一半显存不足那一支

- 空提交冲不冲得掉待写（读 wgpu-core 30.0.1 源码，`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/`）：`src/resource.rs` 第 982–1033 行 `unmap_inner` 的 `BufferMapState::Init` 一支把暂存 `pending_writes.consume(staging_buffer)`、把目的缓冲 `insert_buffer`；`src/device/queue.rs` 第 1348 行起的 `submit` 在命令缓冲为空时跳过 prepare，但第 1689 行照样 `pending_writes.pre_submit(...)`，待写进这一次提交。冲得掉，成立。暂存在建缓冲那一刻就分配（`src/device/resource.rs` 第 1159–1169 行 `StagingBuffer::new`），解开映射才挂进待写；没建成的那一份（显存不足、无效缓冲）映射不上、不解开（外壳只在 `get_mapped_range_mut` 成功时 `unmap`），不挂待写，它的暂存要么没建成，要么随无效缓冲当场丢。
- 等到了放账那一刻：待写那一次提交做完之后 `poll(Wait)` 走 `maintain`、分拣做完的提交，挂在它上面的暂存与目的缓冲随之放掉；回读两份没进任何提交，在 `buffers_and_both_readbacks_created_within_one_out_of_memory_scope` 交回时就丢了。账在等到之后才放：没早放（推的，读码；真设备上没量）。
- 等不到：账留着、记成还在跑（`settle_the_chunk_whose_buffers_were_not_created` 照 `after_waiting_for_its_completion` 判），交回的是等的那一次的错：「Err(failure_waiting_for_the_flush) => failure_waiting_for_the_flush,」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:540`）。这样它不进「显存不足 ⇒ 压一半重切」那一臂（那一臂只认 `OutOfMemoryCreatingTheBuffers`，「Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {」（`crates/singlefs-background-gpu/src/background_gpu.rs:3109`）），进超时重交那一臂；重交之前先等前一次（等的就是那次空提交），等到了才 `hold`。读码没找到「账上还记着这一段时又去记一段」的批内路径：`hold` 第一句断言没有还在跑的（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:459`），批内每次 `hold` 之前都经过交活之前或重交之前的那一等。批外那一条（切换时量代价）就是 T1 的打中：它在账上还记着一段时又去 `hold`，结局是断言 panic，不是重复记账。
- 重交次数多算 1 与「超过 3 次停卡」：停卡看的是超时次数（「if self.timeouts > RESUBMISSIONS_AFTER_A_TIMEOUT {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:177`）），重交次数只进报警。等不到空提交时：超时次数加 1（这一次确实是一次等超过上限），重交次数也加 1（「| GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:199`） 那一支算「交到了卡上」，「timeouts_of_the_batch.resubmissions += 1;」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:240`）），而那一段并没有交出去。造得出的序列（推的，没量）：一段头一次交真超时（超时 1）→ 等到 → 重交那一次建缓冲显存不足、空提交等不到（超时 2、重交 1）→ 等到 → 重交又是同样（超时 3、重交 2）→ 等到 → 重交又同样（超时 4 ⇒ 判这张卡用不了，报警带重交 3）：这张卡被判用不了，而那一段真交出去的只有头一次。条款字面是「累计到第 4 次等超过上限就报这张卡用不了」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）：四次都是真的「等超过上限」，按字面不早不晚；按「重交 3 次都超时」那一读（重交指那一批的段再交一次），它早停了——字面两读，交主 agent 判；报警里的重交次数 3 比真交出去的 0 多，与欠账 C952（单次抽样的条目与泄漏账持有方）同类。

### 这一格最脆弱的一句

「建缓冲出错时交一次空提交、在等的上限之内等它，等到了放账」这一句整段只有真设备才走（替身卡的显存不足直接 `release`，不经空提交），单测只罩了结账函数；它压在两处没量过的 wgpu-core 行为上：空提交带走待写（读码成立），`poll` 等到这次提交即分拣掉它上面的暂存（读码成立，没在真卡上量过）。其次脆弱的是 `hold` 的断言：账的「不重复记」全靠它在每次 `hold` 之前有人先等过，而 T1 那条量代价的路正好没等。

## T4　只改名的 25 份产品文件

旧名、新名取自主 agent 的改名表 `research/prompts/bggpu-audit-fix-code-r1-main-checks/rename-only.out`（195 对，量的：去重计数）。查了六类，都没打中：

| 类 | 查法（量的，只读命令 / 只读 python） | 结果 |
|---|---|---|
| 跨文件、跨包的旧名残留 | 冻结副本里 `.rs`、`.tsv`、`.toml`、`.wgsl`、`.sh`、`.py` 按词搜 195 个旧名 | 长名 190 个零命中；命中的只有单字母 `i j k q r`（`unlink_proofs.rs` 里的局部量，别处同名的是别的局部量）与一条测试名，见下一行 |
| 按名字找的登记表 | 主工作区 `.claude/gate.d/`、`scripts/`、`.claude/hooks/`、冻结副本 `crates/mutations.tsv` 按词搜 190 个长旧名；再搜「是旧名的片段而不是新名的片段」的记号 | 按词零命中；片段只命中四处说明列：`crates/mutations.tsv` 第 4338、4339、4342 行的说明括号里是旧测试名，点名的测试列是新名；`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 第 198 行说明列是「证明负例·a_segment_holding_a_pinned_slot_is_not_a_sound_choice」，`landing=` 是新名。说明列不进行为（推的：读到的用法是显示名）|
| 登记表里的原文片段还命中 | 25 份文件上的 `verus-must-red-worlds.tsv` 47 行、`crates/mutations.tsv` 417 行，原文片段在点名文件里数一次 | 都是恰好 1 次 |
| 名字进了行为（`Debug` 比较、`stringify!`、序列化、盘上名字） | 改名表里大写开头的只有 `ExternalTypeSpecificationOfPathBuf`（Verus 外部类型说明，名字不进行为）；没有枚举成员、字段改名；`admission.rs` 的 `enum_listing_every_member!` 宏收的成员没在改名表里 | 没有 |
| 遮蔽与撞名 | 195 个新名逐个搜跨文件的同名定义 | 只有 `specification_bytes`、`specification_decode_at`、`specification_parse` 在几种类型的 `impl` 里各有一份（改名前 `spec_*` 也是这样），方法路径各自解析，不撞 |
| 量法自身的漏洞 | 去收尾逗号会把 `(x,)` 与 `(x)` 当成一样；去注释会漏 doctest | 25 份里只有 `admission.rs` 的「逗号 + 右括号」个数变了（103→107），单元素元组表达式基线与快照各 1 处、都是同一个宏模式 `$(,)?`；25 份的文档注释里没有代码围栏（两版都 0） |

一条攻得到的边：`cargo build` 不跑 Verus，`spec fn` 与 `proof` 里的旧名残留编得过；上面按词零命中把这一类也排了（推的：只查了字面残留，没跑 Verus）。

## 检查单八格

1. **走不到的理由二次查实：不成立（一处）**。前三格落点里新加的会 panic 的：外壳三段各一处 `assert_eq!(bytes_held_by_this_chunk, …footprint…)`、`buffers_and_both_readbacks_created_within_one_out_of_memory_scope` 的真建尺寸断言、`mapping_table_bytes_held_with_their_staging` 的 `expect`、`input_buffer_bytes` 的 `expect`——前两类靠三处口径一致（T3 表），后两类靠字节数远小于 `u64`，都追得到。走不到而没写理由的：`LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch`（T2 打中，被整段清单断言挡住）。同类的「调用方的 bug 所以 assert」还有：选段打分整段清单断言（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:454`）、`hold` 的「还有在跑的提交」断言（T1 打中，走得到）。
2. **类型与名字：成立（判的是现状）**。`bytes_held_by_a_chunk(&self, input_bytes: u64, entry_count: u64) -> u64`、`settle_the_chunk_whose_buffers_were_not_created(&mut self, bytes_held_by_the_chunk: u64, …)`、`mapping_table_bytes_held_with_their_staging(u64) -> u64`、两个系数常量都是裸 `u64`；位置是 `usize`。核心层有 `GpuMemoryInBytes` 而这几处没用它，算不算缺陷交主 agent（没找到压这一条的规则，推不出）。新名里没看到缩写与空泛词。
3. **不兼容旧代码与旧格式：成立**。前三格落点相对基线的新增行里没有认旧版本、迁移、旧格式、旧布局号（量的：diff 新增行按「兼容、旧版、旧格式、迁移、legacy」搜，零命中）。
4. **不新建判定库：成立**。同一批新增行按 rocksdb、sqlite、`.tsv`、`/tmp` 搜零命中（量的）。
5. **所有任务多机可用：成立**。新增行里的总线地址都是替身卡的假地址（`0000:01:00.0`、`0000:41:00.0`）；`e202` 里的地址字面量与基线逐行相同、只是行号挪了（量的）。
6. **测试优先多线程：推不出**。新用例都是单场景（替身卡或纯函数），没有新的切片枚举；单条用例内部该不该并行，读码定不了。
7. **带 --item 的命令与项：成立**。实现员报告第 4 组点名 22 行（12 新、10 改锚）；12 行新变异按说明列后缀在冻结副本 `crates/mutations.tsv` 里各恰好 1 行，原文片段在点名文件里各命中 1 次（量的）。其中两行（开卡器期限）是 G02 / C996 的。formal-verification 那 4 项的项名报告里没写死（`<照列出的项名>`），核不了是哪四项。
8. **新崩溃流多罩了什么：成立**。`.claude/gate.d/stage-inputs.tsv` 的 108 条 `crash-case:` 行里点到后台 GPU 包的 0 条；`scripts/crash/crash-amplification.sh` 里搜不到这个包；产品 GPU 包源码里没有 `enumerate_crash_states`（量的）。

## 落在别的题里的（C996 的块，不计进这一轮的打中）

- 挂载那一刻开卡线程 panic：挂载走 `outcome_of_opening_a_gpu_within_the_limit`，线程断开就在挂载线程上 `resume_unwind`，没有接住（接住 panic 的「std::panic::catch_unwind(std::panic::AssertUnwindSafe(open)).unwrap_or_else(|payload| {」（`crates/singlefs-background-gpu/src/background_gpu.rs:1238`），只包切换开卡那一路）。条款写「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。量代价里走得到 panic 的一处：wgpu 30.0.1 的 `Device::poll` 遇到 `to_poll_error` 认不出的错（设备丢失那一类）走 `handle_error_fatal`、直接 panic（读 wgpu 30.0.1 `src/backend/wgpu_core.rs` 第 1916–1927 行与第 344–350 行，wgpu-core 30.0.1 `src/device/life.rs` 第 148–156 行）；那时这张卡在 unwind 里照常析构（标记还没记），设备已丢、`wait_for_idle` 会当场交回丢失（推的），所以不挂，问题落在「挂载 panic 而不是算没开成」，归重开那一块。
- 挂载失败后紧接着重挂：「同一张卡前一次开卡没回来就不再起」的守卫按一次挂载建（`opening_of_a_card_within_the_limit` 里新建的那一份），重挂另起一份；前一次挂载的量代价线程晚到的泄漏会在新挂载开卡读完进程账之后才记上，新挂载这张卡的账少记那一片（推的，没造时序）。与 `GpuMemoryLeakedInThisProcess` 文件头已写的「两次挂载同时开着时各自开卡那一刻都没算上对方的那一片」同一类边界。

## 改法（都是推的，被攻过零轮；没实现、没跑）

| 打中的格 | 改法 | 修哪一格 | 在打中的格上 |
|---|---|---|---|
| T1 切换时量代价撞 `hold` 断言 | `GpuInUse::measured_cost_of_one_submission` 开头先调 `room_once_the_submission_running_past_its_wait_completed`（在此刻设置给的等的上限之内等它、放账），等不到就照量代价出错交回（这张卡不参与、报 `CostNotMeasuredAtTheSwitch`） | 历史甲、乙的第 5 步 | 推的，待主 agent 在模型里加这一臂再跑（模型的替身卡要同样先调它，才是 A/B） |
| T1 同上，另一种 | `put_the_settings_in_effect` 里账上记着还在跑的提交的卡不量、照「量代价出错」记，下一次切换再判 | 同上 | 推的；比上一种多停一轮（用户要再切一次） |
| T2 选段打分新成员走不到 | 二选一：把整段清单断言改成交回出错（重复的段在断言之前查），或删掉 `LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch` 并在「走不到的理由」写明整段清单断言挡在前面 | T2 第 2 步 | 推的 |
| T3 等不到空提交算进重交数 | 等不到冲刷时交回一种单独的原因（照超时算进超时数，`the_attempt_reached_the_gpu` 判它没交到卡上） | T3「重交次数多算 1」那一段的报警读数；不改停卡的时刻 | 推的；停卡早不早一次取决于主 agent 判字面哪一读，这个改法不碰那一读 |

## 要主 agent 跑的命令

一条（在新 shell 里起；交服务时的工作目录是攻方腿草稿目录 `/home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1-attack`，运行目录在它下面；`rerun.sh` 里那一行 `cargo test` 自带范围声明）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1-attack -- bash /home/fy5090/code/singlefs/research/prompts/bggpu-audit-fix-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1/repo /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1-attack/model-run /home/fy5090/code/singlefs/research/prompts/bggpu-audit-fix-code-r1-opus-model
```

跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `name=apply_model replaced=3 inserted=2`、`name=cargo_test status=0`、`test result: ok. 4 passed`（库内测试过滤 `attack_model` 之后应是 4 条：三条在 `background_gpu::tests`、一条在 `live_slot_kernels::tests`）、`name=prediction_check patterns=18 matched_once=18 lines=18`；不是 18 / 18 的，逐条看 `name=prediction_miss` 与上面打出的 `name=attack_model…` 原行。运行目录里的 `tree/` 是仓副本，跑完由主 agent 删（`model-run` 整个）；编译目录由服务给，照 `task-server/task clean <号>` 删。

估时（推的）：rsync 冻结副本约 103 MiB（量的：`du -sh`），编 `singlefs-background-gpu` 连同它的依赖第一次几分钟量级；用例本身：替身卡用例 10 段历史各几次切换，暂存算术 20 万例纯整数运算，都在秒级。

## 没打中的形状

- T1：量代价出错的四种结局（等提交超时、等提交报别的错、回读映射超时、建缓冲显存不足而冲刷等到 / 等不到）逐个对标记：只有提交超时、等提交报别的错、冲刷等不到会记「还在跑」，回读超时与冲刷等到不记，各自与卡的真状态一致。开卡线程到点后晚到的两种结局（开成、量代价出错）都没留下要析构而挂着的卡。
- T2：校验和、区域扫描两段的重复键在三处位置（相邻、首尾、三次出现）与计数前缀（已有不确定轮数的键、头一次出现的键）上读码都先于计数交回；`From` 三个实现都只包一层。
- T3：三段 × 三处口径逐份缓冲对过；改小上限两处的代数推导写在 T3 节；模型 20 万例只罩显存那一维（绑定与派发上限放大不起作用），绑定上限那一维没取样。
- T4：六类见 T4 节的表。

## 这条腿自己的限度

- 模型一行都没跑：所有模型读数都是预测（`out/predicted-patterns.txt`），待主 agent 跑出。预测错了的那几条，打中的结论跟着改。
- T1 的打中靠替身卡「量代价照产品走账」这一处改动；它照的是产品 `measured_cost_of_one_submission` → `both_kernel_outcomes_of_a_checksum_chunk` → `hold` 这一条读码链，没在真卡上起。真卡上要造「量代价那一次提交超过等的上限」，得把等的上限调得比一次提交还短，模型没给这条真卡命令。
- 读码没跑 Verus，T4 的「spec fn 名残留」只排了字面残留。
- 没读禁读清单里的任何东西；也没读背景材料第五节之后的附录（kb 条款去 kb 文件里现查）。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。
- 没建仓副本与编译目录：草稿目录里只有几份名单与文本（旧名、新名、替换片段），不用删；运行目录 `model-run` 由主 agent 跑那一条时建，交主 agent 删。
