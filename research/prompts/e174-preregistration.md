# E174 跑前登记：借据丢掉时等待与显存回还

写于 2026-10-08，装置写之前。

<!-- doc-lint:not-numbers G1 G2 -->

## 一、问题

英文名：lease_drop_wait_memory_return

中文简称「借据丢掉时等待与显存回还」由实验设计员起，待主 agent 认。

主 agent 给的问题（逐字）：

> 在本机两张卡（RTX 5090、RTX 5060 Ti）上，一笔借据建的 wgpu 缓冲在借据丢掉时，若先让设备把这笔借据提交的活做完（`device.poll` 用等待那一种），再丢掉缓冲：
> 1. 等这一下要多久（按缓冲大小、提交的活多重分档）；
> 2. 等完、缓冲丢掉之后，下一次 `nvidia-smi -q -d MEMORY,PIDS` 读到的这个进程占着的显存，还含不含那笔缓冲（含多少），要隔多久才不含；wgpu / 驱动的分配器会不会留着不还（按块拿多出来的那一截）。
> 两个量合起来判：乙这一臂（拿掉「销了账、卡上可能还没还的那一份」这本账）在丢借据之后读到的占用是否已经不含那笔缓冲；不含才不会多借。

### 读法（写死）

1. **借据**：一组 wgpu 缓冲，合计 S 字节，在设备起好、管线编好之后建，由这笔借据的提交用到；缓冲对象在借据活着时由用的一方丢掉（与实现同序：`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 5963 行的借据比核对时建的缓冲活得久，第三节），然后借据丢掉。借据丢掉 = 先做那一次 `device.poll`（臂定它是哪一种），poll 返回的那一刻记作 **t0**（实现里 poll 之后才销账：`GpuCardLease` 先 poll、再丢字段 `booking`，第三节）。
2. **「等待那一种」**：`wgpu::PollType::Wait { submission_index: None, timeout: Some(30 秒) }`。这一段只有这笔借据的活在设备上，`None` 与「这笔借据最后一次提交的号」等的是同一批活；设备上另有别的线程的活时两者不同，那一形归第五节末尾的第二段。
3. **这个进程占着的**：一份 `nvidia-smi -q -d MEMORY,PIDS` 报告里，那张卡的 `Processes` 下进程号等于装置进程号的各行 `Used GPU Memory` 之和（MiB），与实现读的口径相同（`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 742、748 行一带，第三节）。没列这个进程、或那一行读不出，是读不到，不当 0。
4. **下一次读**：t0 之后装置立刻起的第一份报告（不隔时间）。它的起止时刻都按装置进程里的单调时钟记（第六节「计时」）。
5. **含不含那笔缓冲**：`R_lease(t) = U(t) − U_pre`，U_pre 是这笔借据建缓冲之前那三份报告的中位数；R_lease 大于 τ（第六节定义）算「含」，含多少就是 R_lease（MiB）。另报 `R_dev(t) = U(t) − U_dev`，U_dev 是设备起好、第一笔借据之前的中位数，量这一个进程累计留着的。
6. **要隔多久才不含**：`T_clear` = 从 t0 到「这一份及窗口里之后每一份都 R_lease ≤ τ」的那一份报告**起读**的时刻；窗口里一直没满足的，写「> 窗口长（窗口上沿，没量到）」。
7. **按块拿多出来的那一截**：缓冲活着时 `ΔU_peak = U_peak − U_pre` 减去 S（MiB），与丢掉之后留着的 R_dev；wgpu 一侧另读 `wgpu::Device::generate_allocator_report()` 的 `total_reserved_bytes`、`total_allocated_bytes`，分开「wgpu 的分配器留着」与「驱动还没报还」。
8. **取值范围**：卡两张（5090、5060 Ti）；S 五档：1 MiB、64 MiB、512 MiB、2 GiB、8 GiB；活三档（第五节）；臂两条（等待 / 不等待）；一轮 = 一个装置进程，每个进程连着做 3 笔同形借据（第 1 笔设备刚起、第 2、3 笔设备已经用过，分开报；长窗那几格每个进程 1 笔）；每格 5 轮。
9. **分母**：每一格的判定按 5 轮各自的值判（`.claude/singlefs-ai-sop/rules/test-discipline.md`「单次观测不算数」），不先平均。

两种问法之间没有要主 agent 定的：问题里的「再丢掉缓冲」按第 1 条读（缓冲对象先丢、借据丢掉时 poll），与实现的次序相同；先等完再丢缓冲对象那一种不在这一次。

### 岔路单（逐行抄）

这一次只登记让 G2「乙」那一格够判的量：翻面观测里乙的两个数（下文记作 **G2-乙-读数**：等待之后下一次 `nvidia-smi -q` 读到这个进程占着的是否已经不含那笔缓冲；**G2-乙-等待**：等一次要多久）。甲、丙两格不在这一次实验里。
**出处 `research/prompts/m3-batch10z-a-code-r1-forks.md:5-7`（整段抄，未转述）**

```markdown
| 岔路 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| G2 销了账、卡上可能还没还的那一份怎么往下压 | 甲：只按「这个进程此刻占着的」往下压（第 10y 批那一形：那一份不大于读数里这个进程占着的），不再用「见到的那一份」；乙：借据丢掉时先等设备把这笔借据的活做完、缓冲交回驱动（`poll` 改成等待），再销账，销了账的那一份不再记，「见到的那一份」整个拿掉；丙：保留「见到的那一份」，按攻方改法 B（减的量取借的与上一次读数之后见到的增量里小的）、C（第 ② 段核对那一笔整段只建一次缓冲）、E（重起时只加旧设备此刻读得出的那一份）逐形补 | 乙：等待之后下一次 `nvidia-smi -q` 读到这个进程占着的是否已经不含那笔缓冲；等一次要多久。甲：真跑一趟产品路径与 GPU 用例时，因为少借报本次未跑的次数。丙：补完之后在两条腿的模型上有没有一形多给 | 乙的两个数在真卡上量过；甲的少借次数在真卡上数过；丙的补法在模型上跑过一遍、又被攻过一轮 | 用户 2026-10-08 定：先回甲、同时量乙；乙量完再交用户定（开着） |
```

## 二、被测条款与它引的定义

被测条款两处，`research/scripts/quote-kb.py` 抽进草稿目录的出口文件、回读一致后追加：D24（后台重活能不能卸给 GPU） 已定项 2 整节（`@标题` 取）；用户 2026-10-07 定 G1 二那一行连同表头（第 10t 批代码轮岔路单，行区间取，最后一行是 G1 行、不是 `|---|`）。

与这一次的关系：已定项 2 第 1 条（GPU 是可选加速器）与第 3 条（GPU 路径要能被 CPU 独立复核）管的是 GPU 在格式与验证里的地位，这一次量的是 GPU 测试进程借显存的账，不判这三条成不成立；它们给出的约束是本实验的装置只在测试与验证一侧，不进格式。G1 二是账今天的形态（只记本进程借的、每次借现读）：乙在它之上改「借据丢掉时」那一步。

「销了账、卡上可能还没还的那一份」不是 kb 条款，定义在实现里（第三节）。
**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:28-44`（整段抄，未转述）**

```markdown
#### 已定项 2：三条硬约束

**定案**：三条，不满足就不许做——

1. **绝不能成为必需。** 文件系统必须在没有 GPU 时完整工作、且语义完全相同。⇒ **GPU 是可选加速器，一个比特都不许进格式。**
2. **不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。
3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，抽样复核。

**射程**：第 1 条管的是格式，第 2 条管的是路径，第 3 条管的是验证，三条各自独立，满足不了任何一条就不许做。它们不定「值不值得做」，那是已定项 1 与已定项 3。第 3 条要的两份实现今天一份都没有（D24（后台重活能不能卸给 GPU） 暂缓，不排期）。

**依据**：

- 用户定案（GPU 是可选加速器），原话在变更史。
- 无实验：三条都是政策与验证纪律的直接推论，没有可量的量；第 2 条引的 PCIe 往返量级是公开常识，本工程没测过前台路径上的 GPU 往返。

**欠**：无。

```

**出处 `research/prompts/m3-batch10t-code-r1-forks.md:7-9`（整段抄，未转述）**

```markdown
| 岔路 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| G1 GPU 显存账的形态 | 一：每次借在卡的锁里现读空闲，账只记「借了还没实际占」那一部分；状态数按剩下的空压；只在用到的卡上登记、起设备；借不到的卡下一笔再试（攻方改法甲乙丙丁合起来）。二：不做跨进程账，每次借现读空闲、只记本进程借的；两个进程同时读到同一份空闲的竞态，靠运行时认出显存不够（内存包装退 245）报本次未跑兜底。三：一台机器同时只跑一个 GPU 测试进程，只留进程内的账 | 某一个候选在某一形上让卡撞显存，或者让放得下的活等满、判红；或者代价（每次借多一次 `nvidia-smi`、一次派活状态少、挂钟变长）大到不可接受 | 三个候选各自对上两腿量到的 S1–S6、探针 A–G 各形，写明修哪一格、留哪一格（攻方报告「几个改法各修哪一格」一节是候选一的那一半） | 用户已定（2026-10-07）：二，去掉跨进程账 |
```

## 三、实现今天的样子

以下是实验设计员 2026-10-08 读到的 `crates/`（行号现查）：

- **借据丢掉时只做一次不等待的 poll**：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1208 行 `impl Drop for GpuCardLease`，第 1211 行 `self.card.device().poll(wgpu::PollType::Poll)`，失败只打一行 `GPU_CARD_LEASE_RETURN_POLL_FAILED`。结构体字段次序是第 1164 行 `card: GpuCard`、第 1165 行 `booking: GpuMemoryBooking<'static>`：`Drop::drop` 先跑 poll，之后字段按声明次序丢，`booking` 丢时销账。
- **销账**：`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 1388 行 `impl Drop for GpuMemoryBooking` 调第 897 行 `return_a_lease`：借的 MiB 加进 `returned_possibly_still_on_the_card_mebibytes`（字段定义第 157 行），早先读数见到的那一份里有这笔就减掉它。
- **那一份怎么往下压**：同一文件第 854 行 `take_in_a_reading` 每读一次调第 209 行 `returned_possibly_still_on_the_card_after_a_reading`（压到不超过「实际占着的 − 早先见到还算数的」）；第 181 行 `occupied_part_of_what_this_process_promised_mebibytes` 拿实际占着的先扣掉这一份、再封顶到许出去的。设备丢了重起时第 878 行 `take_back_the_driver_share_of_a_lost_device` 把旧设备的驱动那一份也记进这一份。乙要拿掉的就是这一本账。
- **读数**：同一文件第 656 行 `NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS = ["-q", "-d", "MEMORY,PIDS"]`；第 742 行认 `Process ID`、第 748 行起把进程号等于本进程的各行 `Used GPU Memory` 加起来；第 977 行 `readings_taken_into_the_books` 在账的锁里每借一次读一次。
- **设备怎么起**：`gpu_unit_checks.rs` 第 266 行 `backends: wgpu::Backends::VULKAN`，第 318 行 `required_limits: adapter.limits()`，第 320 行 `memory_hints: wgpu::MemoryHints::Performance`；wgpu 钉 `=30.0.1`（`crates/singlefs-checker-tier/Cargo.toml` 第 22 行）。
- **第 ② 段核对**：`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 5953 行 `verify_my_blocks_on_the_first_card`，第 5963 行借据先于核对拿到、活到函数末尾；第 3162 行 `verify_blocks_from_these_facts` 里建 `BlockVerifier`（事实表的几张缓冲），函数返回时丢，早于借据。每一块调 `crates/singlefs-checker-tier/src/crash_verify_gpu.rs` 第 411 行 `verify_states`：建 uniform、结论、回读三张缓冲，提交，第 490 行 `poll(Wait { submission_index: None, timeout: None })` 等完，读回，缓冲在函数末尾丢。即「一笔借据活着时一趟一趟建了又丢缓冲」，每趟丢缓冲时活已经做完。
- **别的等待**：单元校验和那条派活等自己那一次提交（`gpu_unit_checks.rs` 第 659 行 `submission_index: Some(submission)`）；判器派活等 `LONGEST_WAIT_FOR_THE_CARD_SECONDS`（`crash_judge_dispatch.rs` 第 32 行，第 875 行）。
- **同一个设备几条线程共用**：单元校验和那笔借据把卡交给 `std::sync::Arc::new(lease.card().clone())`（`crash_amplification.rs` 第 5815–5830 行一带），按 `checking_threads()` 几条线程一起派活；借据丢掉时设备上可能还有别的借据的活。这一形归第二段。

`research/` 一侧：`research/e7-index-bench/Cargo.toml` 第 478 行起已有 `wgpu =30.0.1`（`vulkan`、`wgsl`、`std`）与 `pollster =1.0.1`，挂在特性 `e163-gpu` 上，只给 E163 的 bin 用。

## 四、跑之前已经存在的数

读条款、读实现、判问法时已经知道的数与自己推出的答案，照实列，不删：

| 数 / 推断 | 出处 | 对判据的影响 |
|---|---|---|
| 5090：Total 32607 MiB、Reserved 497 MiB、空卡 Free 32110 MiB；5060 Ti：Total 16311 MiB、Reserved 462 MiB、空卡 Free 15849 MiB；空卡上卡级 Used 各 2 MiB，`Processes : None`；驱动 595.84 | 这一次跑的 `nvidia-smi --query-gpu=…` 与 `nvidia-smi -q -d MEMORY,PIDS`（第十三节）；同样的数也写在 `gpu_memory_ledger.rs` 文件头第 34–36 行 | Total 进第七节 B 类锚点；8 GiB 那一档在 5060 Ti 上放得下（16311 MiB）。不进任何门槛 |
| 一次 `nvidia-smi -q -d MEMORY,PIDS` 约 0.05 秒（连跑 5 次，`/usr/bin/time` 每次报 0.05 s） | 第十三节命令 | 定读数的间隔（背靠背约每 50 ms 一份）、窗口里能读几份与第五节的挂钟估计；Q1 的 5 ms、20 ms 两档门槛拿它作比（一次读数的十分之一、五分之二） |
| `MEBIBYTES_LEFT_FOR_THE_DRIVER = 2560`，注释写「5090 上量到的是 2326 MiB」 | `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 第 27 行与上一行注释 | 只用来预期 U_dev 的量级（几 GiB 以内），不进门槛 |
| 一份 `nvidia-smi -q` 报告里空闲与进程占着的两次查询相隔约 1.3 毫秒（第 10y 批攻方量过 1347.9 µs） | `gpu_memory_ledger.rs` 文件头第 39–41 行 | 装置读数时不在别的线程上建缓冲（第一段单线程），这一形碰不到；第二段多线程时照它写作废条款 |
| `LONGEST_DISPATCH_SECONDS = 0.5`，注释写派活太久撞过 Xid 109；`LONGEST_WAIT_FOR_THE_CARD_SECONDS = 120` | `crash_judge_dispatch.rs` 第 30、32 行 | 「在飞·重」取 400 ms，低于 0.5 秒；等待的超时取 30 秒（另定，不取 120） |
| NVIDIA Vulkan 上存储缓冲绑定上限是 2 GiB − 1 | `gpu_unit_checks.rs` 第 316–317 行注释 | 大于 1 GiB 的借据拆成每张 1 GiB 的几张缓冲 |
| **自己推出的答案（依赖源码，未在本项目验证）**：wgpu-hal 30.0.1 的 `MemoryHints::Performance` 让设备内存块取 128–256 MiB、主机内存块 64–128 MiB（`~/.cargo/registry/src/*/wgpu-hal-30.0.1/src/lib.rs` 第 445–450 行）；gpu-allocator 0.28.0 里比块大的分配单拿一块（`gpu-allocator-0.28.0/src/vulkan/mod.rs` 第 487–488 行），块空了就还给驱动，除非它是最后一块通用块（同文件第 667–673 行）。由此推的预期：1 MiB、64 MiB 两档会让进程多占一块 128 MiB 级的通用块、丢掉之后留着；512 MiB 及以上单拿、丢掉就还 | 判问法时读的依赖源码 | 这是一条假设，不是证据（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」）。它只用在两处：S 的五档跨过 256 MiB；Q3 另读 wgpu 的分配器报告把「wgpu 留着」与「驱动还没报还」分开。门槛不用它，量出相反的数照实记 |
| **自己的直觉（未读 wgpu-core 源码核过）**：被还在飞的提交用着的缓冲，丢掉之后要等下一次 `maintain`（poll 或 submit）才真正释放；`PollType::Poll` 在活没做完时释放不了它 | 设计员的先验 | 这是「不等待」臂在「在飞」两档上可能读到「含」的来由；判据不按它写，「不等待」臂量出「不含」照实记（第十节 F5） |

除此之外没有读到这个量的已有结论：全仓按「销了账、卡上可能还没还」与 `returned_possibly_still_on_the_card` 搜（排除实验页、产物、提示目录与实验索引、变更史），只命中 `crates/mutations.tsv` 与 `gpu_memory_ledger.rs` 本身（第十三节）；`research/` 下没有别的装置算过「丢借据之后读数里还剩多少」，第五类（`.claude/rules/mutation-sampling.md`）要对的另一个口径不存在。

## 五、臂、阳性对照、真实基线

### 装置写在哪（写死）

装置走 `research/e7-index-bench/src/bin/`：独立手写模型，不与实现共用代码（`.claude/rules/implementation-first.md` 第 4 条）。这条岔路问的是 wgpu 30.0.1 与本机驱动在「丢缓冲之后多久读数里不含它」上的行为，另写一份独立装置答得了，不写成入库装置。

- 源文件 `research/e7-index-bench/src/bin/e174_lease_drop_wait_memory_return.rs`，bin 名 `e174-lease-drop-wait-memory-return`，挂新特性 `e174-gpu = ["dep:wgpu", "dep:pollster"]`（依赖沿用同一份 `Cargo.toml` 里已钉的 `wgpu =30.0.1`、`pollster =1.0.1`），`required-features = ["e174-gpu"]`。
- 装置照实现取的设置写成本地常量（不引 `crates/`）：后端只用 Vulkan；`request_device` 的 `required_limits` 取 `adapter.limits()`、`memory_hints` 取 `MemoryHints::Performance`；读数用 `nvidia-smi -q -d MEMORY,PIDS`，按进程号把 `Used GPU Memory` 加起来；「不等待」臂的 poll 是 `PollType::Poll`。执行员开跑之前逐条对 `crates/` 回比（第十一节停机条款的命令），不符就停。
- 一个装置进程只用一张卡、只起一个设备；父进程按清单一个接一个起子进程（同一时刻卡上只有一个装置进程，不并行），每个子进程自己计时、自己把结果行写进自己的输出文件；父进程等子进程结束后整份读完再并（不在转发输出的循环里打时间戳，`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）。
- 次序：清单按种子 174 打乱；同一格的 5 轮里两条臂交替（等待、不等待、等待…），两张卡交替。
- 由主 agent 经 `bash research/scripts/gpu-quiesce-run.sh -- bash research/scripts/run-with-memory-cap.sh <上限> <父进程命令…>` 起（`.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」）；上限取 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认 8G（缓冲都在卡上，主机侧不按 S 涨）。

### 一笔借据在装置里怎么做

每个子进程：起设备、编两条管线（一条空转内核、一条只写一个字的小内核）→ 量「活」的标定（下面）→ 读 3 份报告取中位数记 U_dev，同时读分配器报告记 A_dev → 连做 3 笔同形借据，每笔：

1. 读 3 份报告，中位数记 U_pre，极差记 spread；读分配器报告记 A_pre。
2. 建缓冲：S ≤ 1 GiB 建 1 张 S 字节；S > 1 GiB 建 ⌈S / 1 GiB⌉ 张、每张 1 GiB（用途 `STORAGE | COPY_DST`，`mapped_at_creation: false`）。
3. 提交一次：先对每张缓冲 `clear_buffer`（让每张缓冲都被这次提交用到），再在第一张上派一次空转内核（时长按活的档，见下）。记提交返回的时刻 t_submit。
4. 活的档决定提交之后怎么走：「已做完」先 `poll(Wait { submission_index: Some(这次提交), timeout: Some(30 秒) })` 等完；「在飞」两档不等。
5. 读 1 份报告记 U_peak，读分配器报告记 A_peak（缓冲还活着）。「在飞」两档这一步也在活做完之前读到（读一份约 50 ms，活 ≥ 50 ms；第十节 F7 核）。
6. 丢缓冲对象（阳性对照「扣住不丢」除外，见下）。
7. 借据丢掉：按臂做一次 poll，前后各记一次单调时钟，差是 T_wait；返回的那一刻是 t0。记 e = t0 之前那次 poll 开始的时刻 − t_submit。
8. 从 t0 起背靠背读报告，窗口 2 秒（「长窗」那几格 30 秒）；每份记起读、读完相对 t0 的时刻与这个进程占着的；t0 之后第一份、窗口末那一份另读分配器报告（A_next、A_end）。
9. 下一笔借据从第 1 步起。

活的标定：子进程起来后在第一张卡上用一张 1 MiB 的缓冲派空转内核，按目标时长调循环次数，等完量时长，调到连续 3 次量到的都在目标的 0.8–1.25 倍之间为止，记这 3 次的中位数为这个进程的 W。

### 臂（两条，每条都做上面每一格）

| 臂 | 借据丢掉时那一次 poll | 谁会认这个样子 |
|---|---|---|
| **等待**（乙那一步） | `PollType::Wait { submission_index: None, timeout: Some(30 秒) }` | 岔路单乙：「借据丢掉时先等设备把这笔借据的活做完、缓冲交回驱动（`poll` 改成等待），再销账」。只写做法；「做完之后读数不含那笔缓冲」是本实验要量的事，不写进臂的定义 |
| **不等待**（今天的样子，对照） | `PollType::Poll` | 就是 `gpu_unit_checks.rs` 第 1211 行今天的做法，支持保留「见到的那一份」的一方认它：丢借据时只收一次，卡上还没还的由那本账接着往下压 |

### 活三档、两种借法

- 活：**已做完**（空转目标 50 ms，第 4 步等完再丢）；**在飞·轻**（空转目标 50 ms，提交后不等）；**在飞·重**（空转目标 400 ms，提交后不等）。
- 借法 **P1 一次建**：上面那样，一笔借据一次提交。主格子用它。
- 借法 **P2 一趟一趟**（第 ② 段核对的样子，`crash_verify_gpu.rs` 第 411 行）：一笔借据活着时做 8 趟「建 S 字节一张缓冲 → 清 + 小内核 → `poll(Wait { submission_index: None, timeout: Some(30 秒) })` → 丢」，8 趟之后借据丢掉。U_peak 在第 8 趟的缓冲还活着时读。只做「已做完」这一档（每趟已等完是这个借法的定义）。

### 格子

| 组 | 卡 | S | 活 | 借法 | 臂 | 每个子进程 | 轮 |
|---|---|---|---|---|---|---|---|
| 主格子 | 2 | 1 MiB、64 MiB、512 MiB、2 GiB、8 GiB | 3 档 | P1 | 2 | 3 笔，窗口 2 秒 | 5 |
| P2 | 2 | 1 MiB、64 MiB | 已做完 | P2 | 2 | 3 笔，窗口 2 秒 | 5 |
| 阳性对照「扣住不丢」 | 2 | 512 MiB、2 GiB | 在飞·轻 | P1，第 6 步不丢、窗口末才丢 | 2 | 3 笔，窗口 2 秒 | 5 |
| 几何「并存的长借据」（第八节） | 2 | 1 MiB、64 MiB | 在飞·轻 | P1，U_dev 读完就另建一张 64 MiB 缓冲、清一次，活到子进程末尾 | 2 | 3 笔，窗口 2 秒 | 5 |
| 长窗 | 2 | 64 MiB、2 GiB | 在飞·轻 | P1 | 等待 | 1 笔，窗口 30 秒 | 5 |

### 阳性对照与真实基线

- **阳性对照**（每条臂都跑）：「扣住不丢」。缓冲对象在 t0 之后还活着，t0 之后的读数必须含它：每一轮 R_next ≥ S − τ（S 按 MiB）。它证的是「t0 之后的读数分得出一笔还占着的缓冲」；这一组两条臂各 2 张卡 × 2 档 × 5 轮。另在每一笔借据上核一次：S ≥ 512 MiB 时 ΔU_peak ≥ S − τ（缓冲活着时读得到它）。
- **真实基线**：「不等待」臂，即今天 `crates/` 的做法；P2 是实现第 ② 段真用的借法。等待臂与它在同一格、同一次序里交替跑。

### 跑的次序、续跑、分段

次序按岔路排（这一次只有 G2-乙 一行，按「哪一步不过就全作废」排在前）：

1. 停机条款的回比（第十一节，不用卡）与第七节 B 类锚点里不用卡的那几条。
2. 阳性对照「扣住不丢」（它不过整轮作废，先跑）。
3. 主格子（Q1、Q2、Q3 的主体）。
4. P2、几何「并存的长借据」、长窗。

续跑：父进程把每一轮的结果按键（组、卡、S、活、借法、臂、轮）写进产物；同一条命令重起时，键已经有完整结果行的轮不再起，只补没做完的。作废的轮按键记「作废：原因」，重起时照补，每格补到 5 轮有效为止、一格最多补 3 次，补满仍不够的整格记作废（第十一节）。

**第一段**（就是上面全部 1–4）：补 G2-乙 两个数（G2-乙-读数、G2-乙-等待）在「设备上只有这一笔借据的活」这一形上的全部格。补完之后照岔路单的够判条件「乙的两个数在真卡上量过」，这一形够判。

**第二段**（只在主 agent 对着岔路单判要续时跑；第一段已经让第十节 F2、F3、F4 任一条打中时不跑）：同一设备几条线程共用（第三节最后一条）这一形。一个子进程两条线程：线程 B 在同一个设备上一次接一次提交空转 400 ms 的活；线程 A 照主格子做 P1、S 取 64 MiB 与 2 GiB、活取在飞·轻；臂三条：`Wait { submission_index: None }`、`Wait { submission_index: Some(线程 A 这笔借据最后一次提交) }`、`Poll`；量 Q1、Q2a。补的是 G2-乙-等待（与 G2-乙-读数）在共用设备时的值；第一段不够判这一形。第二段读数时线程 B 不建缓冲，免得落进第四节那 1.3 毫秒的两次查询之间。

### 预计挂钟

| 段 | 估计 | 依据（推的，没量过） |
|---|---|---|
| 编译 | 约 5 分钟 | wgpu 30.0.1 已在同一个 `Cargo.toml` 里编过（E163），只多编一个 bin |
| 第一段主格子 | 约 60 分钟 | 60 格 × 5 轮 = 300 个子进程；每个约 12 秒 = 起设备与编管线约 1 秒 + 标定 ≤ 3 秒 + 3 笔 ×（读 3 份 0.15 秒 + 建与活 ≤ 0.5 秒 + 读峰值 0.05 秒 + 等待 ≤ 0.4 秒 + 窗口 2 秒）；一份报告 0.05 秒照第四节 |
| 第一段 P2、扣住、几何 | 约 21 分钟 | 三组各 8 格 × 5 轮 = 40 个子进程，每个约 10 秒 |
| 第一段长窗 | 约 11 分钟 | 4 格 × 5 轮 = 20 个子进程，每个约 33 秒 |
| 第一段合计 | 约 1 小时 40 分，2 小时内 | 上面四行相加 |
| 第二段 | 约 15 分钟 | 2 卡 × 2 档 × 3 臂 × 5 轮 = 60 个子进程，每个约 12–15 秒 |

两段都在 2 小时内：照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」，分片、多机两样这一次不要求；续跑照上面那一段做。问题点名本机两张卡，第二台机器不在这一次的取样里（不读多机配置的台数）。第一段实际跑过 2 小时的，执行员在草稿目录的 `progress.md` 每完成一组记一行，交回时写明。

产物大小：每份报告只存解析出的字段；原文只存 U_dev、U_pre、U_peak 与 t0 之后第一份这几份，gzip 压。按主格子 300 个子进程 × 3 笔 × 约 6 份原文 × 2 KB 估约 11 MB 未压，压后远小于 GitHub 单文件 100 MB 的上限（`git log` 第 31e47115 次提交的先例）。

## 六、报哪些量与各自的判据

### 记号与计时

- U(t)：一份报告里这个进程在这张卡上占着的（MiB，第一节读法第 3 条）。U_dev、U_pre、U_peak 见第五节。R_lease(t) = U(t) − U_pre，R_dev(t) = U(t) − U_dev（可以是负数，照实记）。R_next = t0 之后第一份报告的 R_lease。R_dev_end = 窗口末那一份的 R_dev。
- **τ**（每笔借据各算）：U_pre 那 3 份的极差 + 2 MiB；**τ_dev**：U_dev 那 3 份的极差 + 2 MiB。2 MiB 的来由：两份各自取整到 MiB 的读数，相减最多差出不到 2 MiB。极差大于 16 MiB 的那笔借据作废（第十一节）。
- 计时：全部在装置子进程里用单调时钟（`std::time::Instant`）计，写进子进程自己的结果行。T_wait = 第 7 步那一次 poll 返回减开始；外层另记「第 6 步开始 → t0」，外层必须包住 T_wait，包不住的那笔不进统计（`.claude/singlefs-ai-sop/rules/test-discipline.md`「分段计时在被测进程里计，不在转发输出的循环里计」）。每份报告记起读、读完两个时刻（相对 t0）。
- 每一行的判定按格（卡 × S × 活 × 借法 × 臂）再分「第 1 笔」与「第 2、3 笔」两栏判，5 轮各自的值都列出来；第 2、3 笔一栏每轮取两笔里大的那个。

### 每个量一行

| # | 量 | 怎么算 | 门槛与判定 | 对应岔路行 | 取什么值让那一行翻面 |
|---|---|---|---|---|---|
| Q1 | 等待臂的 T_wait（ms） | 第 7 步 `Wait` 前后单调时钟之差 | 「已做完」档：5 轮都 ≤ 5 ms 记「零头」，都 > 5 ms 记「有固定开销」并报中位数，否则「不稳定」；「在飞」两档：超出 E = T_wait − max(0, W − e)，5 轮都 ≤ 20 ms 记「只等活本身」，都 > 20 ms 记「等得比活长」，否则「不稳定」；任一轮 `Wait` 返回超时或错，记「等不回来」 | G2-乙-等待 | 「等不回来」出现在任一格 ⇒ 乙在那一格不可行（销不了账）；「已做完」档中位数 > 50 ms（比读一次显存还贵）⇒ 记「乙的代价翻面候选」交用户；其余只报数，够判条件是「量过」 |
| Q1b | 不等待臂的 T_wait（ms） | 第 7 步 `Poll` 前后之差 | 5 轮都 ≤ 1 ms 记「不等」，否则报数 | G2-乙-等待（乙比今天多付的是 Q1 − Q1b） | 不单独翻面：它是 Q1 的减数 |
| Q2a | 等待臂的 R_next（MiB）与 R_next / S | t0 之后第一份报告 | 5 轮都 ≤ τ 记「不含」，都 > τ 记「含」（报最大值），否则「不稳定」。S = 1 MiB 那一档的「不含」只说明没有大于 τ 的剩余，分不出那 1 MiB 本身 | G2-乙-读数 | 任一格「含」⇒ 乙（拿掉整本账）在那一格最多多借 R_next 的最大值，乙原样不可行；「不稳定」⇒ 那一格乙不够判、不当可行；每一格都「不含」⇒ 这一行站乙一边 |
| Q2b | 不等待臂的 R_next（MiB） | 同上 | 同 Q2a | G2-乙-读数（乙里「poll 改成等待」这一半要不要） | 某格不等待臂「含」而等待臂「不含」⇒ 那一格「改成等待」是必要的；不等待臂也处处「不含」⇒ 这一形上读数干净不靠改成等待（第十节 F5） |
| Q2c | 等待臂的 T_clear（ms）与窗口里 R_lease > τ 的份数 | 第一节读法第 6 条 | 5 轮都 = 0 记「下一次就不含」；都在窗口里记「隔 T_clear 最大值才不含」；任一轮超出窗口记「窗口里没不含」，报「> 窗口长（窗口上沿，没量到）」 | G2-乙-读数（要隔多久才不含） | 任一轮 T_clear > 0 ⇒ t0 之后某一份读数含着，乙在那一格原样不可行（与 Q2a 同向；Q2a「不含」而 T_clear > 0 的，按 T_clear 判，并列出那几份读数） |
| Q3 | R_dev_end（MiB），两条臂各报 | 窗口末那一份的 R_dev | 5 轮都 ≤ τ_dev 记「不留」，都 > τ_dev 记「留着」（报最大值），否则「不稳定」 | G2-乙-读数（分配器留着不还） | 等待臂某格「留着」⇒ 乙拿掉那本账之后，这一截不在账上的占用在下一次读数里会被当成借据或驱动那一份已经建上的，乙原样对它不设防（怎么记交主 agent 按账的算术判）；「不稳定」⇒ 那一格不够判 |
| Q3b | ΔU_peak − S（MiB），两条臂各报 | U_peak − U_pre − S | 5 轮都 > τ 记「按块多拿」并报中位数，都 ≤ τ 记「不多拿」，否则「不稳定」 | G2-乙-读数（附带：给 Q3 的那一截找来由；同一批子进程里顺带读，不另跑） | 不单独翻面 |
| Q3c | wgpu 分配器的两项：留着的块（A_end.reserved − A_dev.reserved）与没还的分配（A_end.allocated − A_dev.allocated，并存的长借据那张 64 MiB 照减），字节 | `generate_allocator_report()` | 「没还的分配」5 轮都 = 0 记「wgpu 都还了」，否则报数；「留着的块」5 轮都 > 0 记「wgpu 留着块」并报 MiB，都 = 0 记「wgpu 不留」，否则「不稳定」。报告是 `None` 的记读不到 | G2-乙-读数（附带：把 Q3 拆成 wgpu 留着与驱动还没报还两截；同一批子进程里顺带读，不另跑） | 不单独翻面 |

把门槛与臂的定义并排核过：等待臂的定义只写「借据丢掉时用 `Wait`」，不涉读数与时长；τ 来自读数取整与底噪，5 ms、20 ms、50 ms 来自一次读数的时长（第四节），30 秒是另定的上限，都推不出来自臂的定义。Q1 与 Q2、Q3 各占一行、各报各的判定，不合成一个「乙可行」；合起来判由主 agent 做。

## 七、钉绝对值的断言

### A 类：出自被测条款本身的

没有。D24（后台重活能不能卸给 GPU） 已定项 2 三条与 G1 二那一行都不带显存读数或等待时长的数（第二节整段抄的原文），没有能钉的绝对值；所以这一类不写失败条款里「条款可能错」那一支。

### B 类：独立算出、用命令核过的（不符才作废）

| # | 断言（写成加法） | 怎么来的 | 不符时 |
|---|---|---|---|
| B1 | 装置从报告里读出的 Total：5090 那张 = 32607 MiB，5060 Ti 那张 = 16311 MiB；卡按 PCI 总线认（`01:00.0` 是 5090，`10:00.0` 是 5060 Ti） | 第十三节 `nvidia-smi --query-gpu=index,name,pci.bus_id,memory.total,driver_version --format=csv` 的输出 | 整轮作废（认错卡或读错字段） |
| B2 | 每笔借据缓冲的张数与每张字节数：1 MiB = 1 张 × 1048576；64 MiB = 1 × 67108864；512 MiB = 1 × 536870912；2 GiB = 2 × 1073741824 = 2147483648；8 GiB = 8 × 1073741824 = 8589934592（装置写成「张数 × 每张 == S」的加法形，不写减法） | 第十三节那条 `python3 -c` 的输出（五行末列 `True`） | 那一格作废，修装置重跑 |
| B3 | 缓冲建好之后 A_peak.allocated ≥ A_pre.allocated + S（字节） | 建了 S 字节缓冲，分配器至少要分出 S；对齐只会让它更大 | 那笔借据作废（装置没建出它说的那么多） |
| B4 | 阳性对照「扣住不丢」每一轮 R_next ≥ S − τ（MiB）；每笔 S ≥ 512 MiB 的借据 ΔU_peak ≥ S − τ | 缓冲对象活着就在卡上 | 前者打中整轮作废（第十节 F1）；后者那笔作废 |
| B5 | 每份报告里装置这个进程按 `-q` 读出的占用，与同一时刻 `nvidia-smi --query-compute-apps=gpu_bus_id,pid,used_memory --format=csv,noheader,nounits` 里这张卡、这个进程号那一行的值逐 MiB 相等：在 U_dev 与每笔 U_peak 之后各另读一次比 | 两份是同一个驱动的两种出口，装置的 `-q` 解析写错时对不上 | 那一个子进程作废，查解析 |
| B6 | 每个子进程标定出的 W 在目标的 0.8–1.25 倍之间（50 ms 档 40–62.5 ms，400 ms 档 320–500 ms） | 第五节「活的标定」 | 那一个子进程作废 |
| B7 | 驱动版本 = 595.84 | 第十三节同一条 `nvidia-smi` 输出；实现的解析照这一版的文本模板写（`gpu_memory_ledger.rs` 第 655 行） | 不作废，走第十一节停机条款 |

互比断言旁边的绝对值：Q2a 与 Q2b 两条臂互比之外，B4 钉「还占着时读数 ≥ S − τ」，两条臂一起读错（比如都读成 0）时 B4 先红。

## 八、轨迹与几何敏感性

### 轨迹

R_lease 与 R_dev 是账的谓词输入（实现里每借一次读一次、按它压「那一份」），照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「端点不是轨迹：被条款当谓词输入的量，实验要报轨迹」，每笔借据报三样：**t0 之后的峰值**（窗口里 R_lease 最大的那一份）、**为正的份数**（窗口里 R_lease > τ 的报告份数，与窗口里总份数并列）、**期末值**（窗口末那一份的 R_lease 与 R_dev）。另报每份报告相对 t0 的起读时刻，画得出 R_lease(t)。窗口 2 秒，长窗那几格 30 秒。只报期末值的格，正文不写「从来」「恒」。

### 几何敏感性

被判的是「等待臂读数不含」与「留不留着」两件事（Q2a、Q2c、Q3）。主格子已经在 S（跨 256 MiB 两侧）、活、卡、笔序四个旋钮上扫。另取一个方向相反的取样点：**并存的长借据**。主格子里子进程里除这笔借据之外没有别的缓冲；这一组在 U_dev 读完之后另建一张 64 MiB 缓冲（清一次），活到子进程末尾，借据 S 取 1 MiB、64 MiB（落在同一类块里），活取在飞·轻，两条臂，两张卡，各 5 轮。它对着的是实现里「长借据建着时短借据一笔一笔还」那一形（第三节，`gpu_memory_ledger.rs` 文件头第 25–27 行）。U_dev 在长借据建之前读，R_dev 照减长借据的 64 MiB 之后报（另列不减的原值）。

判定：同一个 S、同一张卡、同一条臂上，主格子在飞·轻那一格与这一组的 Q2a、Q3 判定逐格对：判定相同记「两点相同」，不同记「不稳定（对并存的长借据敏感）」。

判别力自证（`.claude/rules/mutation-sampling.md`「第六类：跑前写死的判决只在一个几何 / 旋钮取样点上量过」）：执行员拿产物另跑一次判定脚本，把 τ 换成落在两个取样点读数之间的一个值，至少一格的判定必须翻；不翻就是判定脚本没在读那个数，修脚本再判。

没扫的旋钮，照实写：`MemoryHints` 只取实现用的 `Performance`；后端只取 Vulkan；驱动只取 595.84；一个进程里借据只连做 3 笔。

## 九、变异

装置自检（下面变异要打红的断言）：K1–K6 即第七节 B1–B6；**K7** 等待臂在「在飞」两档上 T_wait + e ≥ 0.8 W（等待真的盖住了剩下的活）；**K8** 外层「第 6 步开始 → t0」≥ T_wait；**K9** 「在飞」两档 e < 0.8 W（poll 开始时活还没做完，第十节 F6）。

| # | 变异 | 在哪个取样点上改变输出 | 该红的断言 |
|---|---|---|---|
| M1 | 等待臂的 `PollType::Wait { … }` 换成 `PollType::Poll` | 在飞·重 × 任一卡 × 任一 S：T_wait + e 约 50 ms（只剩读 U_peak 那一份），低于 0.8 × W ≈ 320 ms | K7 |
| M2 | 拆缓冲的张数 `S.div_ceil(1 GiB)` 换成 `S / 1 GiB` | S = 1 MiB、64 MiB、512 MiB：张数成 0 | K2 |
| M3 | 解析时把 `BAR1 Memory Usage` 下的 `Used` 当成这个进程占着的 | 每笔 S = 2 GiB 的 U_peak：BAR1 的 Used 是个位数 MiB，CSV 那一份 ≥ 2048 MiB | K5 |
| M4 | 建缓冲时每张取 `size / 2` | 每一个 S | K3 |
| M5 | 「扣住不丢」组在第 6 步照样丢缓冲 | 等待臂某一格 R_next 已经 ≤ τ 的 S（若两档两卡都没有这样的格，这条变异在本实验取样点不敏感，执行员照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第三类写明，不记等价） | K4 |
| M6 | t0 记在 poll 开始之前 | 等待臂在飞·重：T_wait 约 400 ms，外层不到 1 ms | K8 |
| M7 | 「在飞」两档第 4 步也等完（把 `Wait { Some(这次提交) }` 加到在飞档） | 在飞·轻与在飞·重的每一格：e ≥ W | K9 |

覆盖不到的，照实列：进程号过滤（把各进程的行全加起来）在只有装置一个进程的卡上取样点不敏感，靠第十一节 V1（卡上出现别的进程号整轮作废）兜，不算被抓；判定脚本那几处门槛的变异由第八节的判别力自证代替。

## 十、失败条款

| # | 条款 | 什么观测会让它触发 |
|---|---|---|
| F1 | 阳性对照不过：这套读数分不出一笔还占着的缓冲，**整轮作废**（装置或读法有问题），不判任何一臂 | 「扣住不丢」组任一轮 R_next < S − τ（两条臂、两张卡、512 MiB 与 2 GiB 两档里任一处） |
| F2 | 乙读数不干净：乙原样（拿掉整本账）在那一格多借，乙原样在那一格不可行；不当装置有 bug | 等待臂某格 Q2a 判「含」，或 Q2c 有一轮 T_clear > 0 |
| F3 | 乙对留着的那一截不设防：乙原样在那一格不可行（怎么另记交主 agent） | 等待臂某格 Q3 判「留着」 |
| F4 | 等不回来：乙在那一格销不了账 | 等待臂任一轮 `Wait` 返回超时（30 秒）或错 |
| F5 | 读数干净不靠改成等待：乙里「poll 改成等待」那一半在这一形上不是必要的；正当结果，如实记，不作废 | 不等待臂在每一个等待臂判「不含」的格上也判「不含」 |
| F6 | 「在飞」档其实没在飞：那一轮不进「在飞」格（阴性结果要能和「代码没跑到」分开，`.claude/singlefs-ai-sop/rules/test-discipline.md`「阴性结果要能和「代码没跑到」分开」） | 那一轮 e ≥ 0.8 W（K9 红）；补轮之后一格有效轮仍不足 5 轮的，那一格作废 |
| F7 | 驱动报还比长窗还慢：「要隔多久才不含」只写「> 30 秒（窗口上沿，没量到）」，不写成量到的数 | 长窗那几格任一轮 30 秒内 T_clear 没满足 |
| F8 | 判决对并存的长借据敏感：Q2a、Q3 在那几格记「不稳定」，乙在这一形上不够判 | 第八节逐格对，主格子在飞·轻与「并存的长借据」组判定不同 |

F2、F3、F4 只说「乙原样」：乙改一形（比如另记 Q3 那一截、或销账前多等一段）是新形，交主 agent，不在这一次判。

## 十一、作废条款与停机条款

### 作废

| # | 条件 | 作废多大 |
|---|---|---|
| V1 | 一轮里任一份报告在被测那张卡的 `Processes` 下列出装置以外的进程号 | 那一轮 |
| V2 | 读不到：`nvidia-smi` 退非 0、报告里没有那张卡、设备起好之后报告里没列装置的进程号、或那一行读不出（`.claude/singlefs-ai-sop/rules/test-discipline.md`「读不到 ≠ 读到 0」） | 那一份报告；它是 U_pre、U_peak、R_next 那几份之一时，那一笔借据 |
| V3 | U_pre 或 U_dev 那 3 份的极差 > 16 MiB | 那一笔借据（U_dev 的话那一个子进程） |
| V4 | B1 不符 | 整轮 |
| V5 | B2 不符 | 那一格 |
| V6 | B3 或 B4 后一半不符 | 那一笔借据 |
| V7 | B5 或 B6 不符 | 那一个子进程 |
| V8 | 子进程里 wgpu 报 Out of Memory、校验错、设备丢失，或子进程非 0 退出 | 那一轮；同一格有 2 轮这样的，那一格作废并交回原文 |
| V9 | K8 不过 | 那一笔借据 |
| V10 | `gpu-quiesce-run.sh` 退 78（清场回读没过） | 不起，等卡上的别的进程跑完再起 |

一格补到 5 轮有效为止，最多补 3 次；补满仍不够 5 轮的那一格作废，交回时列出每一次作废的原因。作废的轮照样留在产物里，标「作废：原因」。

### 停机：装置与 `crates/` 对不上

开跑之前执行员在仓根跑下面几条，计数要等于下一段写的值（行号可以随别的改动挪，判的是内容）；任一条对不上，或第七节 B7 驱动版本不是 595.84，**停下交回主 agent，既不作废也不当结果**（`.claude/rules/implementation-first.md` 第 4 条：两边都查）：

```
grep -c 'backends: wgpu::Backends::VULKAN,' crates/singlefs-checker-tier/src/gpu_unit_checks.rs
grep -c 'required_limits: adapter.limits(),' crates/singlefs-checker-tier/src/gpu_unit_checks.rs
grep -c 'memory_hints: wgpu::MemoryHints::Performance,' crates/singlefs-checker-tier/src/gpu_unit_checks.rs
grep -c 'self.card.device().poll(wgpu::PollType::Poll)' crates/singlefs-checker-tier/src/gpu_unit_checks.rs
grep -c 'pub const NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS: \[&str; 3\] = \["-q", "-d", "MEMORY,PIDS"\];' crates/singlefs-harness/src/gpu_memory_ledger.rs
grep -c '^wgpu = { version = "=30.0.1"' crates/singlefs-checker-tier/Cargo.toml
grep -c '"Used GPU Memory"' crates/singlefs-harness/src/gpu_memory_ledger.rs
```

第一条应为 2（今天是第 266 行起设备用的实例与第 960 行卡池找适配器用的实例，两处都只开 Vulkan），其余每条应为 1；2026-10-08 设计员现跑过一遍，正是 2、1、1、1、1、1、1（第十三节）。跑的当中 `crates/` 被别的会话改了（第 1211 行那一次 poll 换了形），照同一条停下。

### 够判停机

- F1 打中：整轮停，不判。
- 第一段全部跑完：G2-乙 两个数在「设备上只有这一笔借据的活」这一形上量过，照岔路单够判条件这一形够判，交回。
- F2、F3、F4 在主格子某一格打中（5 轮已齐）：乙原样不可行这一判已经够；同一段里剩下的格照跑完，因为问题第 1、2 问点名要按大小、活分档的等待时长与「要隔多久」，它们给出不可行的范围；第二段不跑，记「够判后未跑」。
- 第二段只在主 agent 对着岔路单判要续时跑；没派就记「够判后未跑」。

## 十二、修订

### 修订 1（2026-10-08，执行员；装置与单测写好、变异跑完之后，真卡产物一次没跑之前）

原判据（第六至十一节）一字不动，原臂一条不删。这一条补一条臂，并把登记没写死的几处读法钉死。派发提示认的三项（简称、问法读法、第二段一起写）照做，不在这里动。

**1. 补一条臂「在飞·中」：登记第九节 K9 在登记自己的数下必红。**

- 量过的：`nvidia-smi -q -d MEMORY,PIDS` 与 `nvidia-smi --query-compute-apps=gpu_bus_id,pid,used_memory --format=csv,noheader,nounits` 各连跑 5 次，`/usr/bin/time -f '%e s'` 报 -q 的五次是 0.06、0.05、0.05、0.04、0.05，compute-apps 的五次是 0.04、0.05、0.05、0.05、0.05。
- 算术：第五节第 5 步在提交之后读 U_peak（一次 -q，约 0.05 秒），第七节 B5 要在 U_peak 之后另读一次 compute-apps（约 0.05 秒），第 6、7 步的 poll 在这两读之后，所以 e = poll 开始 − t_submit 至少 0.10 秒。K9 要 e < 0.8 W，在飞·轻 W 取 50 毫秒，0.8 W = 40 毫秒；就算不计 B5 那一读，一次 -q 的 0.05 秒也已大于 40 毫秒。登记把这一步的"活 ≥ 50 ms"当成够用，其实是读数时长等于活的时长，K9 在在飞·轻上永远红。
- 装置里的复现：模型卡（`SimulatedCardHost`，读一份记 50 毫秒）上自检 `simulated_light_trips_k9_at_the_registered_numbers` 通过（`research/results/e174-lease-drop-wait-memory-return-checks-2026-10-08.out` 第 12 行：`E7RESULT name=self_check id=simulated_light_trips_k9_at_the_registered_numbers passed=true`）。这只复现算术，不证明真卡上 e 的值。
- 补的臂：**在飞·中**，空转目标 250 毫秒（K6 标定带 200–312.5 毫秒；K9 的 0.8 W = 200 毫秒，大于 e ≈ 0.10 秒）。凡登记里用在飞·轻的组，另各有一份在飞·中：主格子（已做完、在飞·轻、在飞·重之外再加这一档）、阳性对照「扣住不丢」、几何「并存的长借据」、长窗、第二段。判据（Q1–Q3c、F1–F8、门槛、τ）对在飞·中与对别的档一字不差。
- 在飞·轻照登记保留、K9 照登记判：预计这些格凑不够 5 个有效轮（登记 F6），产物里留着每一轮的作废理由。为了不白等，装置对已经作废的借据不再读窗口（2 秒或 30 秒），第 1 笔作废就不再起第 2、3 笔；在飞·轻排在清单最后一个阶段，`run` 带 `--skip-light` 可以先不跑（跑不跑由主 agent 定，默认跑）。

**2. 登记没写死、现在钉死的读法（没改任何门槛或判定）。**

- 一轮有效：子进程没有子进程级作废（V1、V3 的 U_dev、V4、V5、V7、V8），第 1 笔有效，第 2、3 笔里至少一笔有效（长窗只有一笔）。一格取轮号最小的 5 个有效轮判；不够的格写 `name=cell_void`，列出每一轮的作废理由。第一栏取第 1 笔，第二栏每个量取第 2、3 笔里大的那一笔。
- 补轮：5 轮都出现了而有效的不足 5 个，补轮的轮号从 6 起，一格最多补 3 次；`run` 同一条命令重起就是续跑，只补没出现的轮。
- 次序：阶段（扣住不丢最先，在飞·轻最后）→ 种子 174 对"组、S、活、借法、轮号"文本的确定性乱序（续跑时位置不变）→ 两张卡先后由乱序值的一位定 → 两条臂（第二段三条）的先后随轮号轮换。
- B5 逐 MiB 相等的字面读法：compute-apps 里这张卡、这个进程号没有一行，也记不符（作废那个子进程）。执行员不起卡，没法核本机驱动是否把 Vulkan 进程列进 compute-apps；若不列，整轮会全部因此作废，所以主 agent 先跑 `--limit-jobs 2` 冒烟看前两个子进程再放开。
- F1：扣住不丢组任一子进程里出现 `f1_k4_hold_control_did_not_hold`，`run` 停（退出码 4），不再补轮。
- 判别力自证：新 τ 取全部有效借据里"含"着的 R_next 的中位数（没有则 0），数判定翻面的栏数写进 `name=verdict` 的 `discrimination_real_data_flipped_columns`；另用模型卡产物自证至少翻一栏（`discrimination_synthetic_flips`）。
- Q1 的"已做完档等待臂中位数大于 50 毫秒"输出为 `q1_expensive_candidate`，汇总进 `done_wait_expensive_candidate_columns`。

**3. 预计挂钟（推的，没量过；清单的格数与子进程数来自 `research/results/e174-lease-drop-wait-memory-return-plan-first-2026-10-08.out` 与 `…plan-second-2026-10-08.out`）。**

| 阶段 | 子进程数 | 估计 | 依据 |
|---|---|---|---|
| 第一段，不含在飞·轻 | 440（首轮；长窗 20 个 + 其余 420 个） | 约 1 小时 45 分 | 420 × 约 13 秒（登记第五节估 12 秒，加 B5 另读与标定）+ 长窗 20 × 约 35 秒 |
| 第一段，在飞·轻 | 200（首轮），补轮至多再 120 | 约 20 分 | 每个子进程起设备、标定、第 1 笔借据即作废，约 4 秒 |
| 第二段，不含在飞·轻 | 60 | 约 17 分 | 每个约 17 秒（多一次 400 毫秒标定） |
| 第二段，在飞·轻 | 60，补轮至多再 36 | 约 6 分 | 同上约 4 秒 |

第一段不含在飞·轻在 2 小时内；连在飞·轻一起约 2 小时 5 分，所以交回的命令把两个阶段分成两条（先 `--skip-light`，再不带，续跑接着写同一份产物）。

## 十三、读过的文件与跑过的命令

### 读过的文件（行号区间；grep 命中行也列）

- `.claude/agent-common.md` 整份；`.claude/agents/experiment-designer.md`（派发时的定义）。
- 规则整份：`.claude/singlefs-ai-sop/rules/test-discipline.md` 1–122、`.claude/rules/three-way-inference.md` 1–174、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 1–176、`.claude/rules/mutation-sampling.md` 1–96、`.claude/rules/implementation-first.md` 1–20、`.claude/singlefs-ai-sop/rules/test-script-discipline.md` 1–96、`.claude/rules/verification.md` 1–112。
- `research/prompts/m3-batch10z-a-code-r1-forks.md` 1–7（整份）；`research/prompts/m3-batch10t-code-r1-forks.md` 5–9（grep `G1` 命中第 3、9 行）。
- `.claude/kb/decisions/24-后台重活能不能卸给GPU.md` 标题 grep（第 1、5、15、28、45、58、82、95 行）与 1–44。
- `crates/singlefs-harness/src/gpu_memory_ledger.rs` 1–260、690–1000、1340–1400；grep 命中第 12、25、28、34、39、70、90、118、120、127、129、141、152、157、163、181、203、204、209、212、217、227、237、256、267、292、300、324、346、396、412、423、452、467、490、496、509、522、533、564、568、574、578、591、599、608、637、649、652、654、655、656、659、661、690、697、742、745、748、775、777、802、805、833、836、847、848、852、854、863、865、870、871、877、878、883、896、897、903、904、905、933、955、976、977、1045、1060、1076、1091、1149、1256、1268、1286、1327、1334、1342、1348、1359、1364、1370、1376、1382、1388、1405、1429、1448、1459、1483、1498、1508、1519、1528、1542、1566、1598、1621、1630、1651、1657、1668、1722、2115、2122、2221、2653、2666、3192 行（只看了命中行本身）。
- `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 295–335、640–680、1150–1260、1455–1470；grep 命中第 124、131、265、266、270、313、316、317、318、320、498、504、659、959、960、963、1110、1124、1163、1164、1165、1208、1211、1462、1903 行。
- `crates/singlefs-checker-tier/src/crash_amplification.rs` 3162–3215（grep 其中的 `GpuFactsVerifier|verify_states|fn |for |let verifier`）、5815–5830、5900–6010；grep 命中第 2148、5824、5953、5963、5974、5996、6022 行。
- `crates/singlefs-checker-tier/src/crash_verify_gpu.rs` 260–515。
- `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 15–35；grep 命中第 32、875、878、882 行。
- `crates/singlefs-checker-tier/Cargo.toml` 第 22、29 行；`Cargo.lock` 第 275–277、1085–1087、1113–1115、1154–1156 行。
- `crates/mutations.tsv`：只在 `grep -l` 里命中文件名，内容没读。
- `research/e7-index-bench/Cargo.toml` 460–490；`research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs` 第 408 行（grep 命中）。
- `research/scripts/claim-experiment.sh` 1–40；`research/scripts/gpu-quiesce-run.sh` 1–40；`research/scripts/quote-kb.py` 1–60；`research/scripts/replace-once.py` 1–30；`research/scripts/replay.sh` 第 25、26、218、480、785、1479 行（grep 命中）。
- 依赖源码（`~/.cargo/registry/src/*/`）：`wgpu-hal-30.0.1/src/lib.rs` 439–477；`wgpu-hal-30.0.1/src/vulkan/adapter.rs` 第 7、2951 行与 `vulkan/command.rs` 第 24、27 行（grep 命中）；`gpu-allocator-0.28.0/src/vulkan/mod.rs` 470–500、655–690 与 grep 命中第 26、29、165、232–234、276、287、350、377–386、436、484、487、488、542、577、653、667、670、673 行；`gpu-allocator-0.28.0/src/lib.rs` 第 281、304、440 行（grep 命中）；`wgpu-30.0.1/src/api/device.rs` 630–638；`wgpu-types-30.0.1/src/counters.rs` 177–185；`wgpu-types-30.0.1/src/lib.rs` 176–201。
- 没读：`.claude/kb/experiments/`、`research/results/`、`.claude/kb/experiments.md`、`experiments-history.md`、`records/2026-09-30-里程碑三开工第一轮.md`（派发提示点名的批次记录，没读，免得读到结果行）。

### 跑过的命令（原样）

```
TZ=Asia/Tokyo date +%F
bash research/scripts/claim-experiment.sh --next
bash research/scripts/claim-experiment.sh E174 借据丢掉时等待与显存回还
nvidia-smi --query-gpu=index,name,pci.bus_id,memory.total,driver_version --format=csv
for i in 1 2 3 4 5; do /usr/bin/time -f '%e s' nvidia-smi -q -d MEMORY,PIDS > /dev/null; done 2>&1; nvidia-smi -q -d MEMORY,PIDS | sed -n '1,60p' | grep -n -E 'GPU 0000|Total|Reserved|Used|Free|Process ID|Processes|Used GPU' | head -30
python3 -c "
G=1<<30;M=1<<20
for s in (1*M,64*M,512*M,2*G,8*G):
  n=-(-s//G); b=min(s,G); print(s, n, b, n*b==s, s//M)"
grep -rln --exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md --exclude-dir=target --exclude-dir=.git -e '销了账、卡上可能还没还' -e 'returned_possibly_still_on_the_card' . | head -20
python3 research/scripts/quote-kb.py $D/fork-row.md 'research/prompts/m3-batch10z-a-code-r1-forks.md:5-7'
python3 research/scripts/quote-kb.py $D/section2.md '.claude/kb/decisions/24-后台重活能不能卸给GPU.md@#### 已定项 2：三条硬约束' 'research/prompts/m3-batch10t-code-r1-forks.md:7-9'
（第十一节停机条款那 7 条 grep，现跑结果依次为 2、1、1、1、1、1、1）
```

其中 `$D` = `~/.local/share/singlefs/work/m3-r1/g2-yi-prereg`。命令输出原样：

```
2026-10-08
E174
index, name, pci.bus_id, memory.total [MiB], driver_version
0, NVIDIA GeForce RTX 5090, 00000000:01:00.0, 32607 MiB, 595.84
1, NVIDIA GeForce RTX 5060 Ti, 00000000:10:00.0, 16311 MiB, 595.84
0.05 s
0.05 s
0.05 s
0.05 s
0.05 s
1048576 1 1048576 True 1
67108864 1 67108864 True 64
536870912 1 536870912 True 512
2147483648 2 1073741824 True 2048
8589934592 8 1073741824 True 8192
crates/mutations.tsv
crates/singlefs-harness/src/gpu_memory_ledger.rs
```

`nvidia-smi -q -d MEMORY,PIDS` 那一次 grep 出的行：5090 `Total 32607 MiB`、`Reserved 497 MiB`、`Used 2 MiB`、`Free 32110 MiB`、`Processes : None`；5060 Ti `Total 16311 MiB`、`Reserved 462 MiB`、`Used 2 MiB`、`Free 15849 MiB`、`Processes : None`（BAR1 与 Conf Compute 两节的行另有，不用）。另有只读命令（`grep -n`、`sed -n`、`wc -l`、`ls`）读上面列的文件，不一一列。
