# m4-gpugc-code-r2 攻方腿（Opus）报告（2026-11-01）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

被攻对象：冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r2/repo/` 里第 54、55 批合入之后的产品代码。下面引的代码行号取冻结副本；这几份文件（`waiting_for_the_gpu.rs`、`background_gpu.rs`、`gpus_of_this_host.rs`、`memory_limit_and_chunks.rs`、`background_compute.rs`、`mounted_session.rs`、`mount.rs`、`e185_gc_batch_wall_clock_split.rs`、`history.rs`）与此刻主工作区逐份 sha256 相同（量的，`sha256sum` 两边比，前 12 位逐份相同）；`crates/mutations.tsv` 两边不同，所以变异行只写名字、不写行号。条款写 `.claude/kb/decisions/26-后台整理与放置回收.md` 自己的行号（第 246–248 行是已定项 13 第 2–4 条，现查）。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpugc-code-r2-opus-model/`。复跑（主 agent 交，命令原样在文末「要主 agent 跑的命令」）：`bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新草稿目录>`。它把冻结副本的 `crates`、`Cargo.toml`、`Cargo.lock`、`litmus`、`.cargo`、`.claude/heavy-thread-default` 拷进 `<work>/tree`，把六条探针接进 `background_gpu.rs` 的测试模块末尾，跑 `cargo test -p singlefs-background-gpu --lib attack_probe -- --nocapture`，打出 `ATTACK` 行（应当 10 行）与 `test result` 行；`outputs/` 里还没有参照（这条腿没跑过），头一次跑出的那一份就是参照。

| 文件 | sha256 |
|---|---|
| `probes/background_gpu_attack_probes.rs` | `83599c48be4b78c12d49fa0ab9c4ce81462f818a68bb064d38acfaf037cfa031` |
| `tools/apply-probes.sh` | `4a4dc38b0457fed351cc20f5ebf1059d74774391e38c089addf41253800adf77` |
| `tools/rerun.sh` | `66bcf8379291dcf81cab5231a80aef4a2a409f26abcc30529746028f9a84a785` |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`（头 79 行与自 `# 自证：` 起的尾部与模板逐行相同，量的，`diff` 两段都无输出）。它的 `--selftest` 判的是模板自带的那一段 `rerun_steps`，换掉之后那一格会红，不要交它的自证。

**下面每一个探针读数都是「推的，待主 agent 跑」**：这条腿没起 cargo，读数是照代码手算的期望值，写在每格里；主 agent 跑出的 `ATTACK` 行与手算不同的，以跑出的为准，那一格重判。

## 各格判定一览

| 格 | 点 | 判定 | 证据形态 |
|---|---|---|---|
| T1 第 1 条 | 第 4 次超时没报 / 第 3 次就报 | 没打中（试的形状见「没打中的形状」）；旁支打中「重交几次」的记账多数（归第 8 条） | 读码 + 探针 1（推的，待主 agent 跑） |
| T1 第 4、5 条 | 一张切不出、一张建不成：活丢 / 重做 | 打中「重做」：压一半重切之后同一张卡已做完的段从头再交；接手的卡把同一种建不成再撞一次。没打中「活丢」 | 探针 5、6（推的，待主 agent 跑） |
| T1 第 4、5 条 | 第 1830 行压一半不收敛 | 没打中：三段真建的占用与切法按的占用有断言绑住，每压一次界严格变小 | 读码 |
| T1 第 4、6 条 | 切换那一刻判不用的卡 | 打中：切换那一刻把一次等超过上限、还在跑的提交算进「卡上已占的」，这张卡这一组设置里一直不用；一张卡时整理照「一张都不剩」停着 | 探针 2（推的，待主 agent 跑） |
| T1 第 5 条 | 真卡上空闲显存不足 | 打中两处，只有真卡罩得到：回读暂存建在显存不足的错误范围之外；量代价那一段建不成不重切 | 读码（推的，真卡没跑） |
| T1 第 6 条 | 泄漏扣完余量为零的第二次挂载 | 没打中「余量为零」（照不用、不拒挂、停着走）；打中两处旁支：同时开着的两次挂载后丢的盖掉先丢的；`open_background_gpu` 每次新开一份空账 | 探针 4（推的，待主 agent 跑）+ 读码 |
| T1 第 8 条 | 记账钉绝对值 | 打中：重交之前那一等超时也记一次「重交」 | 探针 1（推的，待主 agent 跑） |
| T2 第 1 项 | 出错那一趟报第 1 次 | 读法与字面差一；挂载续做里停下的与叫起里停下的数法不一 | 读码 |
| T2 第 2 项 | `Err` 不带报警、不计叫起 | 打中：攒批会话排空待发没发成那一次在判过起跑、发过一次发布之后，不是「任何读写之前就拒」，字面要报、代码不报不计 | 读码 |
| T2 第 3 项 | 可写挂载续做不算叫起 | 与字面相合（字面列的叫起里没有挂载续做）；挂载续做里出的错要到下一次叫起才报 | 读码 |
| T2 第 4 项 | `#[must_use]` 只管整份丢掉 | 打中：仓里两处调用点接住之后不读报警、编译不警告；产品代码里没有外部调用点 | 读码 + 主 agent 已量的 0 条 warning |
| T4 | 叫起计数重计或漏计 | 漏计一种（同 T2 第 2 项）；重计造不出 | 读码 |
| T4 | 丢掉报警编译不警告的产品调用点 | 产品里没有外部调用点；挂载续做那一趟的 `CompactionRun` 放在 `MountOutput` 的字段里，类型上的 `#[must_use]` 管不到 | 读码 |
| T4 | `GpuWorkOfTheBatch` 一批放弃时 | 没打中：放弃的那一批照样交回记账 | 读码 |
| T4 | 切换时枚举超时当一张都不在 | 打中：切换校验读卡清单的两样各枚举一次，两次答得不一样时点名一张挂载那一刻没开成的卡的切换照收，交给实现那一刻 panic；另：超时报成「GPU 不在」或「点名的卡不存在」，换的等的上限管不到这一次枚举 | 探针 3（推的，待主 agent 跑）+ 读码 |
| T3 | 替身罩不到、只有真卡罩得到的 | 列了 7 处产品代码、1 条真卡用例；这两批追加的 55 行变异没有一行要真卡，第 54 批整行替换的 11 行里 6 行要真卡 | 读码 + `awk` 数的 |

## T1　第一轮的修补本身

### T1-1　第 1 条：一段一次调用里第 4 次超时（没打中）

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「累计到第 4 次等超过上限就报这张卡用不了」；同一行「三段各自计，不按整理的一批、也不按切出来的片」。

试的形状与结果（读码，冻结副本）：
- 片与片之间：一次段调用开头每张卡建一份计数，`crates/singlefs-background-gpu/src/background_gpu.rs:1783`：「let mut submissions_of_each_card: Vec<SubmissionsOfACardInTheBatch> = self」，之后切片、重切、把一张卡的活转给别的卡的每一轮都传同一份（`background_gpu.rs:1798`：「let plans = match self.plans_of_a_batch(」），不重建。片间、重切前后、转活前后都累计。
- 交活之前那一等：`background_gpu.rs:1703`：「let room = outcome_waited_for_again_after_timeouts(」记进同一份；它每等满一次记一次（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:230`：「Err(cause) if waited_past_the_limit(&cause) => {」）。
- 「等前一次超时提交那一等本身又超时」计一次：重交那一圈里那一等的错落进 `attempt`（`waiting_for_the_gpu.rs:211`：「attempt = wait_for_the_timed_out_submission(gpu).and_then(|()| submit_once(gpu));」），下一轮在第 205 行记一次，不记两次。
- 三段里一段报卡用不了之后：核心层放弃这一批、整理停下，同一批另两段不再交（那一批不再往下算）；下一次段调用从 0 数，与上引「三段各自计」相合。
- 第 3 次就报：`note_a_wait_past_the_limit` 在次数大于 `RESUBMISSIONS_AFTER_A_TIMEOUT`（`waiting_for_the_gpu.rs:120`：「pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;」）才交回出错，造不出第 3 次就报。

旁支（打中，归 T1-8）：重交那一圈的「重交了几次」在那一等超时时也加一。

### T1-8　第 8 条：记账里的「重交了几次」多数（打中）

输入（探针 1，替身卡，不给上限，一个 1000 字节的单元）：头一次提交超时；之后重交之前先等那一次提交，前两回等满上限没等到，第三回等到；重交那一次做完。

- 冻结副本今天的结果（推的，待主 agent 跑；手算照第 205–211 行）：这一批照常交回、与 CPU 参照相同；`resubmissions_after_timeouts=3`，而这张卡上一共只交了 2 次（`submissions_made_on_the_card=2`，真重交 1 次）。等前一次那一等每超时一回，第 210 行 `timeouts_of_the_batch.resubmissions += 1;` 照样加一，下一轮才在第 211 行看那一等成没成。
- 同一前缀、三回都等满（探针 1 的第二组）：第 4 次超时报 `UnusableAfterResubmissions { resubmissions: 3, .. }`，记账 `resubmissions_after_timeouts=3`，这张卡上只交过头一次、一次都没重交（`submissions_made_on_the_card=1`）。
- 条款要的结果：第一轮判决第三节第 8 条验收（`research/prompts/m4-gpugc-code-r1-main-verification.md:94`：「一批的记账另记每段交了几片、重交了几次、用了哪几张卡，各有用例钉绝对值」）要的是重交的次数；这一序列里是 1 与 0。`.claude/rules/fs-design.md:127`：「**分支必须可观测**：运行时能看出走了哪一条。」——「重交了」与「等前一次没等到、没交」是两条分支，今天的读数分不开。

四句：
1. 分辨：分得开「每次进重交那一圈就加一」（今天）与「那一等成了、真交出去才加一」两种写法；超时计数（第 4 次报）两种写法一样，不受影响。
2. 看得到：第 211 行那一等的结果在交之前就有，`and_then` 没进 `submit_once` 时系统知道这一次没交。
3. 字面分句：上引验收的「重交了几次」与五条硬要求第 4 条；不是「第 4 次超时报警」那一句（那一句今天对）。
4. 改法：第一轮判决第三节第 1 条给的改法是「超时次数按批、按卡累计」，碰不到这一格；这两批的用例 `a_region_record_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned` 里那两回等发生在交活之前（第 1703 行），不在重交那一圈里，所以罩不到。

### T1-4/5　一张切不出、一张建不成：活丢 / 重做

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「卡上此刻空闲的显存连这一批最小的一片都装不下」时「这一批不给这张卡分活，它那份转给这一批别的参与的卡，不报警」；同一行管超时的那一句「只重交超时的那一片，同一批已做完的片留着」。`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」。

**活丢**（没打中）：转出去的条目进 `entries_to_hand_out`（`background_gpu.rs:1857`：「entries_to_hand_out.push(planned.entries);」），下一轮照量出的代价分给剩下的卡（`plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece` 里 `entry_ranges_by_the_measured_cost_of`，同 `.claude/kb/decisions/26-后台整理与放置回收.md:246`：「按挂载时在每张卡上量出的一次提交的主机 CPU 分活」，不平摊）；接回时按条目起点排、断言相连不漏。试的形状：一张因显存上限切不出 + 一张建不成（报空闲显存那一个成员、两份清单都带，结局是放弃并停下，不丢）；先切不出后建不成、先建不成后切不出；转过去的卡在下一轮又建不成（一张都不剩就报出错，前面算好的随这一批放弃）。都是「全做完」或「整批放弃」，没有一条条目既没交回又没报错。

**重做**（打中，探针 5）。输入：一张替身卡，上限 1100 字节，卡上空闲 500 字节；两个单元 100 字节与 1000 字节。
- 冻结副本今天的结果（推的，待主 agent 跑；按 `CHECKSUM_CHUNK_FOOTPRINT` 手算，一段占 16 + 每条 44 + 输入）：头一轮切成 [100]（160 字节）与 [1000]（1060 字节），[100] 交成，[1000] 建不成，第 1830 行压到 530（`background_gpu.rs:1830`：「room = room.with_the_memory_limit_at_most(bytes_held_by_the_chunk / 2);」）；重切之后「从头再交它分到的那几条」（第 1763 行文档原话），[100] 第二次交，468 字节那一片（528 字节）又建不成，压到 264；第三轮 [100] 第三次交、其余五片交成。照常交回、与 CPU 参照相同；`submissions_completed=8`、`recuts_after_running_out_of_memory=2`，最后那一切只有 6 段（`chunks_of_the_cut_at_264_bytes=Ok(6)`）——100 字节那一段这一次调用里交了 3 次，多出的 2 次是重做。
- 条款要的结果：条款对建不成没写「已做完的留着」，只对超时写了；照「只用空闲的那些」的字面，做完的那一段没有理由重交。这一格是读法问题：已做完的段留不留，条款没定，代码选了重做（第 54 批报告的「我替条款做了的实现选择」与第 55 批的七项里都没有这一条）。代价是主机 CPU（已定项 13 第 7 条的目的就是少占它）。

**同一次调用里接手的卡再撞一次**（打中，探针 6）。输入：两张替身卡、代价相同、都不给上限；`0000:01:00.0` 空闲 10 字节，`0000:41:00.0` 空闲 300 字节；四个 100 字节的单元。
- 今天的结果（推的，待主 agent 跑）：头一轮各分两条；头一张压 3 次后连最小的一片都建不成、不用；第二张自己那两条（一段 304 字节）建不成、压到 152 交成；转过来的两条重新分给第二张时，`plans_of_a_batch` 照设置给的界重新交回地方（不记它这一次调用里已压到 152），一段 304 字节又建不成、再压一次。合计 `recuts_after_running_out_of_memory=5`，记着这一次调用里压到的界就是 4。照常交回。
- 条款：上引「一路切小到最小的一片都建不成」只要求切小，不禁止重撞；结果对，多一次建不成与重切。读法问题，不是与条款相左。

**同一批第二段重撞**（与条款相合）：条款把「一批」定成一段的一次调用（上引「三段各自计」），乙段、丙段从设置重新起是字面要的；不算打中。

**第 1830 行压一半重切收不收敛**（没打中）：建不成交回的 `bytes_held_by_the_chunk` 是这一段真建的字节数，三段真建之前都断言它等于切法按的占用（`crates/singlefs-background-gpu/src/background_gpu.rs:1173`：「这一段真建的缓冲与切批按的占用一致」，第 1285、1402 行同一句），切法保证每段不超过此刻的界，所以压完的界 ≤ 一半 < 原界，严格变小；压到连最小的一片（校验和那一段 64 字节）都切不出就停。不给上限时头一次压成「建不成那一段的一半」，之后同样严格变小。试过奇数字节、0 字节输入（输入缓冲按 4 字节起建）、区域扫描头一片之后那几段用空映射的占用：都收敛。

四句（重做、再撞两格合答）：
1. 分辨：分得开「从头重交」与「留着已做完的段、只重切没交成的那几段」两种写法；替身用例 `a_checksum_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu` 只有一种单元大小，两种写法都过。
2. 看得到：已做完的段的结果在 `run_the_plan_on_a_card` 那一次调用里已经交回过一部分（`checksum_outcomes_of_the_units_cut_into_pieces` 按段累积），外壳丢的是它自己手里的东西。
3. 字面分句：「只用空闲的那些」与第 7 条的收益口径；不是「不报警」那一句（今天不报警，对）。
4. 改法：第一轮判决第三节第 5 条的改法「按更小的片重切重交、不报 GPU 出错」本身就是这一种写法，碰不到这一格。

### T1-4/6　切换那一刻把还在跑的那次提交算进「卡上已占的」（打中，探针 2）

第 54 批给切换那一刻判「这张卡不用」加了一项：扣掉卡上已占的，见 `crates/singlefs-background-gpu/src/background_gpu.rs:1582`：「card.budget().memory_account().bytes_held(),」，进 `gpu_left_out_under_its_memory_limit`，`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:474`：「.saturating_sub(bytes_already_held_on_the_device)」）。`bytes_held` 含两样：此前挂载泄漏的（该扣），与一次等超过上限、还在跑的提交那一段（交活之前先等它做完、放掉账，见 `background_gpu.rs:1703`）。切换那一刻不先等它，判出的「不用」一直留到下一次切换（`left_out_under_their_memory_limit` 只在 `use_the_settings_from_the_next_batch` 里改）。

输入（替身卡一张，设最小一片为 s = `bytes_of_the_smallest_piece_every_participating_gpu_holds()`）：上限 4s；这张卡上记着一次等超过上限、还在跑的提交，占 3s + 1 字节（产品路径上，一段调用里第 4 次超时报卡用不了时，最后那一次提交就这样留在账上）；之后一次通过校验的切换，设置与原来一样（点这张卡、上限 4s）；切换之后交一批（一个 100 字节的单元）。

- 冻结副本今天的结果（推的，待主 agent 跑）：切换之后这张卡进「因显存上限不用」（`left_out=1`），`batch_not_split_over_the_gpus` 交回「参与的卡一张都不剩」，那一批交回 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`；核心层在切换生效那一刻读到它，有 GPU 路径的后台任务当场记成停着、报 `GpuUnusableUnderTheSettingsHandedIn`。对照两组：没有还在跑的提交、同样切换，照常交回；有还在跑的提交、不切换直接交一批，交活之前先等它、放账，照常交回。
- 条款要的结果：`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「参与的卡的上限连最小的一片都装不下，这张卡就不用」——上限 4s 装得下最小的一片；那次提交做完之后账就放掉。`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下」——用户为解除停下而交的切换，在这张卡上换来的是又一次停着，直到再切一次（那时那次提交多半已做完）。

四句：
1. 分辨：分得开「切换那一刻先等还在跑的那次（或只扣泄漏的）再判」与今天「连还在跑的一起扣、判了就留到下次切换」；挂载那一刻没有还在跑的提交，两种写法一样。
2. 看得到：账上分得开泄漏的与还在跑的（`GpuMemoryAccount` 有 `submission_running_past_its_wait` 一样单独记着），切换那一刻能看到。
3. 字面分句：第 247 行「这张卡就不用」那一句的「装不下」与第 248 行「当场解除停下」。
4. 改法：第一轮判决第三节第 6 条（`research/prompts/m4-gpugc-code-r1-main-verification.md:92`：「进程里记每张卡泄漏的字节，新一次挂载这张卡的可用余量先扣掉它」）只要扣泄漏的；今天的写法扣的比它多，碰不到这一格。

### T1-5　空闲显存不足只有真卡走得到的两处（打中，读码，真卡没跑）

- **回读暂存建在显存不足的错误范围之外**：一段占的字节里算了「两份核各一项结果与一项回读暂存」（四份结果大小），可只有前五个缓冲在错误范围里建（`background_gpu.rs:934`：「let out_of_memory_scope = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);」）；两份回读暂存在 `dispatch_both_kernels_and_read_back` 里另建（`background_gpu.rs:970`：「device.create_buffer(&wgpu::BufferDescriptor {」），不在任何错误范围里。wgpu 把没接住的错交给设备上挂的回调（`background_gpu.rs:720`：「device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {」），之后报成 `background_gpu.rs:1057`：「return Err(GpuFailureCause::UncapturedErrorOrDeviceLost { reason });」——GPU 出错、整理停下、报警。输入：卡上此刻空闲的够建输入、描述、两份结果，不够再建两份回读暂存。条款要的：第 248 行「不报警」、转给别的卡或压小重切。推的：wgpu 30.0.1 在 Vulkan 上建缓冲时显存不足是否当场报进错误范围、还是在提交时报，这条腿没核；真卡用例里另一个设备占显存那一种做法占到「建不成 256 KiB 为止」，剩下的落在哪一种缓冲上不定。
- **量代价那一段建不成不重切**：挂载那一刻与切换让一张卡参与的那一刻量代价，量的那一段直接交（`background_gpu.rs:911`：「self.both_kernel_outcomes_of_a_checksum_chunk(&piece_units)?;」），建不成就照出错交回：挂载那一刻是这张卡没开成（参与的卡没开成就拒挂），切换那一刻记进 `cost_not_measured_at_the_last_switch`（`background_gpu.rs:1599`：「self.cost_not_measured_at_the_last_switch = cost_not_measured;」），之后每一批报成这张卡的 GPU 出错。条款：第 247 行「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」与同一行「挂载那一刻量代价（第 2 条）的那一段同样切片，代价取各片之和」。今天量代价那一段只照上限切、不照空闲的切。

### T1-6　进程里泄漏的账

**泄漏扣完余量为零的第二次挂载**（没打中）：余量 = 上限饱和减去泄漏的；为零或泄漏的比上限多都判这张卡不用（见 `gpus_of_this_host.rs:474`），照开、不量代价、不分活、挂载不拒；一张都不剩时三段每一批报「一张都不剩」、核心层挂载那一刻就停着，与 `.claude/kb/decisions/26-后台整理与放置回收.md:247` 的「这张卡就不用」「挂载不因此拒绝」「后台任务照第 4 条进出错停着」逐句对上。不给上限时不扣（不设上限，没有余量可言）。账从泄漏的字节起，`hold` 断言不越上限；不用的卡不建缓冲，断言走不到。

**两次挂载同时开着、都在同一张卡上泄漏**（打中，探针 4）。输入：同一份 `GpuMemoryLeakedInThisProcess` 交给两次挂载（替身外壳两份，同一张卡，开的时候都还没泄漏过），各自记着一次还在跑的提交，占 600 与 300 字节；先丢头一次，再丢第二次。
- 今天的结果（推的，待主 agent 跑）：两次各报泄漏了这张卡（600、300，合计 900）；进程里那一份记成 300——`gpus_of_this_host.rs:691`：「Some((_device, bytes)) => *bytes = leaked.bytes_held_on_the_device,」后丢的盖掉先丢的；第三次挂载的账从 300 起。
- 条款要的：`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「任何时刻都不许比它多」——这张卡上实际被这个进程泄漏占着 900 字节，第三次挂载能再占到上限减 300。盖写只在「一次一次挂、后一次的账已含前一次」时对；两次同时开着时后一次的账不含前一次。条款与第一轮判决都没说一个进程同时开几次挂载，是读法缺口；`GpuMemoryLeakedInThisProcess` 的文档自己写了「几份把手共用同一份」。

**`GpuMemoryLeakedInThisProcess` 没有产品调用方**（读码）：`background_gpu.rs:540`：「pub fn open_background_gpu(」每调一次新开一份空账（`background_gpu.rs:545`：「&GpuMemoryLeakedInThisProcess::with_nothing_leaked(),」），真卡用例四份都用它开卡；`open_background_gpu_compute` 由调用方交账，仓里调它的只有端到端那一份真卡用例、交的是新开的空账。产品里还没有挂载工具，所以第一轮判决第三节第 6 条的验收今天在产品路径上不成立也不失败：它取决于以后的挂载工具在进程里只留一份、每次都交它，类型上不逼（空账谁都能新开，`open_background_gpu` 是 `pub` 的绕路）。

四句（同时开着的两次挂载）：
1. 分辨：分得开「盖写」与「加上这一次新占的」两种记法；一次一次挂时两种一样（这一批的变异行「七·3 GPU 代码轮一：进程里同一张卡再泄漏时字节加上去（重算已含的那一片）」钉的正是一次一次挂那一种，加法会重算）。
2. 看得到：外壳开卡时从进程那一份读了此前泄漏的字节，丢的时候能知道自己开卡时起点是多少，新泄漏的 = 那一刻占着的 − 起点。
3. 字面分句：第 247 行「任何时刻都不许比它多」。
4. 改法：第一轮判决第三节第 6 条只要「新一次挂载这张卡的可用余量先扣掉它」，没说同时开着的几次怎么合，碰不到这一格。

## T2　第 55 批「我替条款做了的选择」第 1–4 项

报告原文在 `~/.local/share/singlefs/work/m4-gpugc-alarm-every-wake-writer/result.md`「我替条款做了的选择（交主 agent 看，三方代码轮可攻）」一节。下面几段历史凡要整理真推一批的，都要只供测试的批大小开关（产品路径批大小条款没写，`crates/singlefs-core/src/mounted_session.rs:3294`：「CompactionBatchSize::NotDecidedByTheClauses => {」那一支在签任何东西之前停下）；这一点对第 55 批的用例同样成立。

### T2-1　「停了多久」从出错那一趟数 1（读法与字面差一；两种停法数法不一）

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「停下以来叫起了几次」，括注「停了多久数叫起的次数，不读挂钟」。代码在第一次叫起结束时记 1（`crates/singlefs-core/src/background_compute.rs:1436`：「wakes_while_stopped: 1,」），出错停下的那一趟也是叫起，所以那一趟报 1。

两段历史（用户动作放开：中间几次叫起是显式入口还是后台入口、补不补跑都不改下面的数，只固定「停在哪」与之后叫起几次）：
- 甲：挂着 GPU 设置可用；显式入口叫起 W1，W1 里一批出错、整理停下；之后叫起 W2。代码：W1 报 1、W2 报 2。
- 乙：可写挂载续做那一趟（不是叫起）里一批出错、整理停下；之后叫起 W1、W2。代码：W1 报 1、W2 报 2。

字面结果：「停下以来」的叫起，甲里不含停下之前就开始了的 W1，到 W2 是 1；乙里到 W2 是 2。代码两段都报 2：同样「停下之后又叫起过一次完整的」，甲报 2、乙报 1（乙的 W1）。差一只在「停下发生在一次叫起里面」时出现（显式入口与后台入口里出错、切换生效那一刻判出一张都用不上都算）。这是读法，不改别的行为；交主 agent 判要不要写进条款「含停下的那一趟」。

### T2-2　攒批会话排空待发没发成：`Err`、不报警、不计叫起（打中）

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「显式的整理入口每一趟也算」「发布之后的查与定时兜底的查都算，补跑的与照旧停着的都报」，例外只有「入口在任何读写之前就拒的那一次不算」。

序列：整理因一批出错停着，最满的那块盘超过补跑阈值（补跑那一判说照常判起不起跑）；攒批会话（`MountedSessionWithPendingChanges`）里待发不空；后台入口叫起，起跑判据为真（「该跑了」或定时兜底到了）；排空待发那一次发布被拒（占用超过阈值的池子上空间拒最常见）。

- 冻结副本今天的结果（读码）：判起跑那一段先换上待生效的切换、取走该跑了的记号（见 `mounted_session.rs:2467`：「compaction.start_judgement = CompactionStartJudgement::NotDue;」）或记下定时兜底这一次查的读数；之后排空（`mounted_session.rs:5584`：「let pending_changes_published_before_the_run = match self.publish_the_pending_changes(」）被拒，交回 `Err`（`mounted_session.rs:5599`：「BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused(」），没走到取报警的那两处（`mounted_session.rs:5569`、`mounted_session.rs:5615`：「let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();」），叫起次数不加、这一次没有报警，`Err` 的类型里也没有放报警的地方。「该跑了」照用户改动那一种放回；定时兜底那一种不放回，这一次查的读数已经记下。
- 条款要的结果：这一次是一次叫起（判过起跑之后才拒，不是「任何读写之前」），要报警、叫起次数加一。
- 后果：池子越满、排空越容易被空间拒，后台入口每一次都这样走时，整理停着期间一次报警都不交——正是补跑阈值要管的那种情形。

四句：
1. 分辨：分得开「`Err` 也带这一次的报警」与今天；第 55 批的用例只造了排空成功与没起跑两种（`the_background_entry_of_a_session_with_pending_changes_returns_the_alarm_while_it_does_not_start`、`…_when_it_runs_a_compensating_batch`）。
2. 看得到：拒的那一刻 `self.session` 里停着的那一条就在，取报警只是调一次函数。
3. 字面分句：第 248 行「入口在任何读写之前就拒的那一次不算」——这一次不满足它的「任何读写之前」，落在「每次叫起都报」里。
4. 改法：第 55 批报告自己点了「第 2 项要定 `Err` 那几条带不带报警」，没给改法；这一格只有改返回形状（`Err` 带报警）或改条款两条路。

### T2-3　可写挂载续做那一趟不算叫起（与字面相合）

条款第 248 行列的叫起是「发布之后的查与定时兜底的查」与「显式的整理入口每一趟」，没有挂载续做；代码在 `crates/singlefs-core/src/mount.rs:7662`：「Some(run_compaction_on(」那一趟恒填 `None`（`mounted_session.rs:3261`：「alarm_of_the_compaction_stopped_after_a_compute_error: None,」），与字面相合。旁记：续做那一趟里一批出错、整理停下时，挂载照常做成，`MountOutput` 里那一份 `CompactionRun`（`crates/singlefs-core/src/mount.rs:1090`：「pub run: Option<CompactionRun>,」）的报警字段是 `None`，出错要到下一次叫起才报出来（报 1，见 T2-1 乙）。挂载那一刻设置就一张卡都用不上而续做在 CPU 上补跑了的，补跑的批数记进停着的那一条，头一次叫起报「叫起 1、补跑 n」，n 含挂载那一趟的。

### T2-4　`#[must_use]` 只管整份丢掉（打中）

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「调用方不处理编译就警告」。

仓里接住结果、只读别的字段、不读报警的调用点（读码）：`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:18508`：「let Ok(run) = session.run_compaction(&mut devices) else {」之后读 `versions_published`、`batches_published`；`crates/singlefs-harness/src/history.rs:7049`：「if let Ok(run) = &run_result {」之后读 `versions_published`。两份文件里 `alarm_of_the_compaction` 命中 0 次（量的，`grep -c`）。主 agent 已量的整工作区编译 0 条 warning（`research/prompts/_m4-gpugc-code-r2-body.md:26`：「`cargo build --all-targets` 任务 385 退 0、0 条 warning」）——这两处编译不警告。harness 用例里 `BackgroundCompactionCall::NotStarted { reason, .. }` 的 `..` 同样把报警丢掉不警告。另一种：结果放进结构的字段（`MountOutput` 那一处）、`Option<_>` 里，类型上的 `#[must_use]` 一概不响。

字面结果：「调用方不处理」的这几种都要警告；今天只有「整份结果当语句丢掉」警告。这两处不是产品代码（实验装置与 harness 的历史驱动）；`crates/` 下不在 `tests/` 里的源码中，`mounted_session.rs` 之外调整理入口的只有 `mount.rs` 的续做、e185 装置两处（第 3479 行与上引第 18508 行）与 `history.rs` 第 7048 行（量的，`grep -rn` 全部 crate、去掉 `tests/`），产品代码里没有核心层之外的调用点。

四句：
1. 分辨：分得开「`#[must_use]`」与「报警不交出去就过不了类型」（例：入口交回 `(结果, Option<报警>)` 让调用方解构、或报警单开一个必须消费的返回）。
2. 看得到：编译器看得到字段读没读，`#[must_use]` 不看。
3. 字面分句：第 248 行「调用方不处理编译就警告」。
4. 改法：第 55 批报告第 4 项写明「更硬的写法要改返回类型的形状，条款没定，没做」；这一格要么改条款收窄成「整份丢掉就警告」，要么改形状，交用户。

## T4　从未被看过的地方

### T4-1　叫起计数重计或漏计

- **漏计**（打中）：同 T2-2，攒批会话判过起跑、排空待发被拒的那一次不计、不报。
- **重计**（没打中）：一次叫起里取报警的只有一处——显式入口 `mounted_session.rs:2367`：「background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);」，后台入口在判起跑与跑之后取一次（`run_compaction_in_the_background` 末尾），攒批会话不走 `MountedSession::run_compaction_in_the_background`、自己直接调判起跑与跑两段，只在第 5569 或第 5615 行取一次。判起跑与跑之间别的叫起插不进来：两段都在同一个 `&mut self` 调用里；中间夹的只有排空待发，排空是一次用户改动的发布，不碰后台计算的状态（换上切换只在 `mounted_session.rs:2457`：「.put_the_switch_requested_from_the_next_batch_in_effect();」与第 3281 行）。试过：判起跑之后取走「该跑了」、跑那一段又因节奏放回；判起跑那一段换上一张卡都用不上的切换（停着那几条连同叫起次数一起换掉，这一次报 1）；跑那一段里补跑一批之后下一批照旧停着。都只加一次。

### T4-2　丢掉报警编译不警告的产品调用点

见 T2-4：产品代码里没有核心层之外的调用点；挂载续做那一趟的 `CompactionRun` 在 `MountOutput` 的字段里，报警恒是 `None`，类型上的 `#[must_use]` 管不到字段。仓里接住不读报警、编译不警告的两处在实验装置与 harness（e185 第 18508 行、`history.rs` 第 7049 行）。

### T4-3　`GpuWorkOfTheBatch` 在一批放弃时（没打中）

一批走 GPU 时，三段算完（不论成没成）之后先把外壳的提交记账取进这一批（`mounted_session.rs:3768`：「gpu_work.submissions_on_the_gpus =」），再交给 `end_the_batch`（`mounted_session.rs:3796`：「gpu_compute.map(|_| gpu_work),」）；出错放弃的那一批照样有记账，出错那一段的条目数没记、它在卡上怎么交的记了。不 `Copy` 之后是移进 `end_the_batch`，没有拷一份丢一份。只供测试的钩子注入出错时三段一段都没交，外壳那边没有可取的。试过：甲段出错（乙、丙没交）、乙段出错、空闲显存一张都不剩那一种、补跑的那一批（走 CPU，记账恒无）。

### T4-4　切换时现查卡清单（打中）

**两次枚举答得不一样时，点名一张挂载那一刻没开成的卡的切换照收、交给实现那一刻 panic**（探针 3）。核心层校验读卡清单的两样，`crates/singlefs-core/src/background_compute.rs:929`：「devices_with_total_memory: gpu_inventory.devices_with_total_memory(),」与第 930 行：「devices_not_opened_at_the_mount: gpu_inventory.devices_not_opened_at_the_mount(),」；外壳答这两样时各现查一次本机在的独显（`crates/singlefs-background-gpu/src/background_gpu.rs:2182`、`background_gpu.rs:2192`：「.identifiers_of_the_discrete_gpus_present_now(),」），枚举等 wgpu 超过上限就答一张都不在，见 `background_gpu.rs:630`：「return Vec::new();」）。

输入（替身）：本机两张独显，`0000:01:00.0` 开成、参与；`0000:41:00.0` 挂载那一刻没开成（给了 `background_gpu_devices` 而没点它，开它时出错，跳过、报出来）；挂着切换只点 `0000:41:00.0`；切换校验时头一次枚举两张都在，第二次枚举交回空（等超过上限，或两次之间那张卡掉了）。
- 冻结副本今天的结果（推的，待主 agent 跑）：头一次枚举给出两张卡的显存总量，`0000:41:00.0` 在清单里；第二次枚举为空，「挂载那一刻没开成的」被 `still_present` 滤成空，「参与的卡里有挂载那一刻没开成的」那一判放过，切换照收；切换生效、把这一组交给外壳时，`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:227`：「.expect("核心层照卡清单校验过：参与的卡都在本机、都不是挂载那一刻没开成的")」panic（`0000:41:00.0` 不在开成的卡里）。对照：两次答得一样时，照 `ParticipatingGpuDeviceNotOpenedAtTheMount` 拒。产品路径上这一 panic 发生在下一次整理入口或后台入口开头（换上切换那一处），握着 GPU 实现的锁时 panic，锁中毒，之后每次借这份实现都在 `crates/singlefs-core/src/background_compute.rs:476`：「.expect("GPU 实现上一次借出时 panic 过」再 panic（推的，读码）。
- 条款要的结果：点名的卡挂载那一刻没开成，照 `.claude/kb/decisions/26-后台整理与放置回收.md:248`：「就拒绝挂载（或拒绝切换）并报错」（「没开成」那一种的定法在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 45 批交回那一行，代码文档自己引的）。

四句：
1. 分辨：分得开「一次切换只枚举一次、两样都照这一次答」与今天的「每答一样枚举一次」；核心层已有「读过一次的卡清单」那一层（`GpuInventoryReadOnce`），只是读它的时候调了两次实现。
2. 看得到：外壳在一次 `admit` 里被问两次，自己能把头一次枚举留着答第二次。
3. 字面分句：第 248 行「拒绝切换并报错」；今天照收再 panic，也碰 `.claude/rules/fs-design.md`「五条硬要求」（这一条腿没逐条对，只记出处）。
4. 改法：第一轮判决第三节第 2 条（`research/prompts/m4-gpugc-code-r1-main-verification.md:88`：「切换那一刻现查卡清单，挂载之后掉了的卡切换时照「GPU 不在」拒」）的改法就是今天这一种「每次答之前现查」，碰不到这一格；要的是「一次切换查一次」。

**枚举超时当一张都不在，与条款的「GPU 不在」**（打中两处小的，读码）：
- 结局相同、报的成员不同：不点名时报 `GpuNotPresent`、点名时报 `NamedGpuDeviceNotInTheInventory`，与卡真掉了报的一样（替身用例 `a_switch_after_the_card_dropped_off_the_host_is_refused_as_gpu_not_present` 走的同一支）。条款第 248 行把「GPU 不在、驱动不对、点名的卡不存在」并列成几种拒因；枚举挂住更像「驱动不对」，今天报成「不在」或「点名的卡不存在」，调用方分不开卡掉了与驱动挂住。都拒，行为同条款；报法是 `.claude/rules/fs-design.md:127` 那一条的事。
- 等的上限取挂载那一刻的：枚举在 `background_gpu.rs:626`：「self.wait_limit,」之内等，那个值是开卡时的（`background_gpu.rs:610`：「wait_limit: selection.background_gpu_wait_limit,」），不随切换换。序列：挂载时等的上限 1 秒、之后这台机器上枚举要 2 秒；用户切换把等的上限调到 10 秒——这一次切换照「GPU 不在」拒，调大上限的切换永远交不进去，只有卸载重挂。条款：等的上限是挂载参数、可切换（`mount-parameters.md` 那一行，这一条腿没逐字核）。推的，没量过枚举要多久。

## T3　替身罩不到、只有真卡罩得到的（顺带列）

数法（量的）：这两批往 `crates/mutations.tsv` 追加的行，第 54 批 34 行、第 55 批 21 行，`awk` 取 `cargo test` 参数那一列含 `tests-on-a-real-gpu` 的：0 行；第 54 批整行替换的 11 行里 6 行要真卡，第 55 批整行替换的 3 行里 0 行（读的是两份补丁目录里的 `mutations-append.tsv` 与 `mutations-replacements.tsv`）。

要真卡的 6 行（变异名，原样）：
- 七·3 后台 GPU 包：丙段外壳给逐条走遍记录那一份核绑错记录表（绑成段界）
- 七·3 后台 GPU 包：丙段外壳给二分查那一份核绑错排好的序号（绑成记录表）
- 七·3 设置交给实现：后台 GPU 包外壳的卡清单只答此刻参与的卡
- 七·3 装不下自适应分片：甲段外壳不按此刻剩下的显存切片（整条交）
- 七·3 装不下自适应分片：丙段外壳不按此刻剩下的显存切记录表
- 七·3 装不下自适应分片：开卡时不判显存上限装不装得下最小的一片

这两批的真卡用例（新加的一条，名字原样）：`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference` 里的 `a_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu`（`crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:376`）；没有变异行点它。

这两批改到、只在真卡那一份实现（`GpuInUse` 与开卡的那几个函数）里、替身换掉了的代码（替身用例与 55 行追加的变异都罩不到）：
1. 建缓冲时显存不足的错误范围：`background_gpu.rs:934` 起的 `buffers_created_within_the_free_memory`（替身用 `free_bytes` 自己判建不建得成）。
2. 回读暂存不在错误范围里：`background_gpu.rs:970`（见 T1-5）。
3. 量代价那一段建不成不重切：`background_gpu.rs:911`（见 T1-5）。
4. 交活之前与重交之前等前一次那次提交的真实现：`GpuInUse::room_once_the_submission_running_past_its_wait_completed` 与 `GpuInUse::wait_for_the_timed_out_submission`（替身照脚本当场交回）。
5. 开卡时账从泄漏的字节起：`started_gpu_with_its_cost_measured` 收 `bytes_leaked_by_earlier_mounts`，`open_background_gpu_with_the_memory_leaked_in_this_process` 把进程那一份交进外壳（替身外壳由用例直接装）。
6. 切换时现查本机在的独显：`AdaptersEnumeratedByTheInstance` 的枚举与超时当一张都不在（替身用 `DiscreteGpusPresentForTests`）。
7. 丢外壳时整张卡不析构：`gpus_dropped_leaking_those_still_running` 对真卡是 `std::mem::forget` 一个开着设备与队列的 `GpuInUse`，替身卡没有设备，罩不到「不等队列」那一半。

## 我提的改法（只在我的模型上推过、被攻过零轮；一条都没实现、没跑）

| 改法 | 修哪一格 | 在别的格上 | 标 |
|---|---|---|---|
| 重交那一圈里「重交了几次」挪到前一次那一等等到、真交出去之后再加一 | T1-8 | 第 4 次报警的次数不变 | 推的，待主 agent 跑（要先实现） |
| 切换那一刻判「不用」只扣泄漏的那一份，或先在等的上限之内等还在跑的那次再判 | T1-4/6 | 挂载那一刻不变（那时没有还在跑的提交） | 推的 |
| 一次切换校验只枚举一次：外壳把头一次枚举留到这一次校验结束（或核心层只问实现一次、两样一起答）；`of_the_settings` 的 `expect` 改成交回出错 | T4-4 第一格 | 卡真掉了照旧报「GPU 不在」 | 推的 |
| 枚举用切换交来的等的上限；枚举等不到单报一种拒因 | T4-4 后两格 | — | 推的 |
| 两份回读暂存也在显存不足的错误范围里建 | T1-5 第一格 | 只有真卡验得到 | 推的 |
| 量代价那一段建不成照挂着时那样压小重切 | T1-5 第二格 | 只有真卡验得到 | 推的 |
| 攒批会话排空被拒的 `Err` 带这一次的报警、叫起次数加一 | T2-2、T4-1 漏计 | 改返回形状，`#[must_use]` 跟着挪 | 推的 |
| 进程里泄漏的账按「这一次新泄漏的 = 那一刻占着的 − 开卡时的起点」加上去 | T1-6 同时开着两次 | 一次一次挂时与今天相同（不重算） | 推的 |
| 建不成之后留着已做完的段，只重切没交成的那几条；接手的卡记着这一次调用里压到的界 | T1-4/5 重做、再撞 | 结果不变，少交几次 | 推的 |

T2-1（差一）、T2-4（`#[must_use]` 的射程）不是代码能单方面改的，交主 agent 判是改条款还是改形状。

## 没打中的形状

- T1 第 1 条：片间、重切前后、转活前后、交活之前那一等、重交之前那一等、三段之间，各一种时序，都照第 4 次报、不早不晚（读码，见 T1-1）。替身脚本里超时与等满的组合只手推了上面这几种，没穷举。
- T1 第 4、5 条「活丢」：显存上限切不出 × 空闲显存建不成的四种先后、转过去的卡再建不成；都整批交回或整批放弃（读码）。
- T1 第 1830 行不收敛：三段的真建占用与切法有断言绑着，每压一次严格变小（读码）；没造出一个占用比切法算的大的段。
- T1 第 6 条「余量为零」：余量为零、为负、不给上限三种，都照条款（读码）。
- T1 第 7 条：后台入口判起跑那一段（`mounted_session.rs` 第 2457 行）与一趟开头（第 3281 行，显式入口与挂载续做都走它）用的是同一个 `compaction_stays_stopped_pausing_the_live_intent`；没攻（第一轮攻过、这两批只挪了位置）。
- T4 重计：见 T4-1 列的三种。
- T4 `GpuWorkOfTheBatch`：甲段出错、乙段出错、空闲显存一张都不剩、补跑的那一批四种。
- 产品路径上「出错之前已经收到、还没生效的切换请求」那一支（`background_compute.rs:1389`：「return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;」）：两个入口在推一批之前都先换上待生效的切换，一批之内又没有别的调用插得进来，产品路径上走不到它，只有只供测试的钩子进得去；不是这两批改的，只记一笔。
- 切换那一刻量一张新参与的卡的代价超时一次，之后每一批就报这张卡 GPU 出错（`background_gpu.rs:1589`：「if let Err(cause) = card.measure_the_cost_of_one_submission() {」）：形似「第 1 次超时就报」，但这一段不是这两批改的（第 54 批只换了字段名），没攻。

## 这条腿自己的限度

- 探针一次都没跑：每个读数都是手算的期望，「推的，待主 agent 跑」。手算错了的格（尤其 T1-4/5 两格的次数、T1-8 的 `submissions_made`），以主 agent 跑出的 `ATTACK` 行为准重判；探针编不过也算这条腿的错，承接的腿先修探针。
- 全在替身卡上：替身的建不成按 `free_bytes` 判、等前一次按脚本交回，真卡上的错误范围、枚举与 `forget` 都罩不到（T3 列的 7 处）。
- 推一批的历史都要只供测试的批大小开关；T2-1、T2-2、T2-3 只读码，没写 harness 探针。T2-2 要的探针形状：停着、占用超过阈值、攒批会话里放一条会被空间拒的待发，调后台入口，看交回的是不是 `Err` 且之后下一次叫起报的次数比前一次只多 1。
- 「没打中」的各格只抽了一次样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不支撑「没有问题」。
- 状态枚举、崩溃点重放这一轮用不上（被攻的是挂着时的后台计算与 GPU 外壳，没有盘上状态要枚举），没调 `enumerate_crash_states`，状态数为 0。

## 要主 agent 跑的命令

1. 攻方六条探针（轻型，harness 档之外的非 checker 包 lib 测试目标，不要真卡）。
   - 命令原样：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-opus-rerun`
   - 在哪起：`--cwd` 是这条腿的草稿目录；脚本里面 `cd` 进 `--work` 下的 `tree`（冻结副本的拷贝）跑 `cargo test`，那一行自带范围声明（种类=target、范围=1 个测试目标）。`--work` 目录此刻不在，脚本自己建；它不在这条腿认领的目录里，跑完由主 agent 或承接的腿删（里面有一份 `crates` 拷贝，约 45M）。判决复跑那一步照 `.claude/rules/three-way-inference.md` 第 3 步把模型目录拷进主 agent 的草稿目录、把上面的模型路径换成拷贝的路径即可，参数不变。
   - 跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 是过（cargo 退 0 且 `ATTACK` 行恰好 10 行；`outputs/` 里没有参照时比对那一步打 `same=no-reference`、不算红）；`task-server/task log <号>` 里 `name=probes-lib rc=0`、`name=probes-lib attack_lines=10`、`test result: ok. 6 passed`（6 条探针都只打行、不断言）。`ATTACK` 行与本报告各格写的手算值逐格比：T1-8 两行、T1-4/6 三行、T4-4 两行、T1-6 一行、T1-4/5 两行。
   - 跑完之后：新派同立场的攻方腿，派发提示写「承接：research/prompts/m4-gpugc-code-r2-opus-output.md」并给任务号；承接的腿读 `task-server/task log <号> --all`，把 `--work` 下的 `outputs/probes-lib.out` 拷进模型目录 `outputs/` 当参照、更新文首 sha256 表，与手算不同的格重判，再删 `--work` 那棵树。

没有重型命令；没有要真卡的命令；没有门禁阶段。

## 没做什么

- 不判正推、辩方那几格；T3 只顺带列了替身罩不到的条目，没判第一轮判决的方法。不替主 agent 采纳。
- 没往统一任务服务交任务，没起 cargo；判决要引的数一个都还没跑出来（命令在上一节）。
- 没写 harness 层的探针（T2-2、T2-1 的两段历史只读码），没写变异行，没改冻结副本与主工作区的任何源码。
- T2 只攻了第 55 批第 1–4 项；第 5–7 项与第 54 批七项没逐项攻（第 6 项在 T1-4/5「活丢」里顺带碰到：一张因空闲显存不用、剩下的因显存上限不用时报空闲显存那一个成员、两份清单都带，读码与它报告写的一致）。
- 草稿目录里只有 `mutation-rows.txt`（`awk` 抽的冻结副本里这两批前缀的 55 行变异，T3 用），没有仓副本、编译目录或库拷贝。
