# GPU 服务 OOM、卡坏与借卡治理文件轮第二轮：辩方腿（Sonnet）报告

日期：2026-11-21。立场：辩方（复核第一轮判决，第 1 到 4 问）。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 读了什么、在哪棵树上

- 读的是此刻主工作区的文件。开工时 `sha256sum -c research/prompts/gpu-oom-card-governance-r2-snapshot/judged-sha256.txt` 对出的不是 `OK` 的行数：0（量的）。
- 读了：正文 `research/prompts/_gpu-oom-card-governance-r2-body.md`、第一轮正文、两份腿报告（只读开头与点名的探针）、判决 `research/prompts/gpu-oom-card-governance-r1-main-verification.md`；
  被判文件里 `card_worker.rs`、`gpu_judges_on_a_card.rs`、`service_loop.rs`（全文）、`serving_from_the_library.rs`（判不了那几段）、`batch_scheduling.rs`（文件头与调度入口）、
  `card_memory_ledger.rs`（核对器那一段）、`control`（卡坏、判不了、cards-failed 那几段）、`task_server_lib_configuration.py`、`task_server_scheduler_notifying.py`、两份任务服务常量文件；
  验证包里被它们调的 `crc32c_of_ranges`、`gpu_cards_by_priority`、`card_memory_once_a_judge_fits`、`out_of_memory_explaining_a_failure_of_the_work`。
- 没读：禁读清单里的攻方腿报告与模型；`break_switches.rs`、`card_lease.rs`、借卡几份测试（不在第 1 到 4 问里）；`tests/` 下除 `service_loop_with_judges_off_the_card.rs` 与库侧两处之外的文件。
- 只有我这一条腿读到的范围：`crates/singlefs-verification/src/gpu_unit_checks/` 下 `crc32c_of_ranges`、`card_memory_once_a_judge_fits`、`gpu_cards_by_priority` 这三处（第 1 问「重开设备」与「提交」落在它们上）。

## 模型与复跑（先给命令与 sha256）

模型目录 `research/prompts/gpu-oom-card-governance-r2-sonnet-model/`。写了两条替身探针（不上卡、不起临时判定库），没编过、没跑过（我不交任务）；只过了 `rustfmt --edition 2021 --check` 的语法解析（量的：除我去掉 `mod common;` 那一行的差异外没有别的差异）。

| 文件 | sha256（量的） |
|---|---|
| `research/prompts/gpu-oom-card-governance-r2-sonnet-model/probe/appended_probes.rs` | `37af3f5427725c50fbc449ce0ce98d1d54abc7da0e3837d3523bedbc13523850` |
| `research/prompts/gpu-oom-card-governance-r2-sonnet-model/tools/rerun.sh` | `35958c9bb8a47576cb8ddbd0bfac1202b49e7e40626f42bb12a695dbee980499` |
| `research/prompts/gpu-oom-card-governance-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh` | `9eda2d96143b458f4087b4f79dd0a2d64414dbf5c574abe4bf7f60eb68007f1d` |

做法：`rerun.sh` 把冻结副本拷进运行目录，把 `tests/service_loop_with_judges_off_the_card.rs` 全文拷成 `tests/service_loop_probe_of_the_defense_leg.rs`，在它后面接 `probe/appended_probes.rs`（探针用那份文件里的替身与辅助函数），只点名两条测试函数跑。
冻结副本在 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet/frozen`（`scripts/hosts/minimal-copy.py --with-test-inputs` 建的，64M，量的），`.deliverables` 登了「交接」。命令见末尾「要主 agent 跑的命令」。

## 先说结论（每问一句）

1. 第 1 问：判决第 1、4、5 条三处改法在代码里做的就是判决「三、改法」写的那件事，但各有一处与判决字面不完全相同的实现细节（下面第 1 问列）；**成立**，其中重开设备那一处在真卡上从没执行过，那一半**推不出**（算打中）。
2. 第 2 问：七条里条目一（OOM 那一刻卡 1 的账）、条目五（多机撞键漏台）仍不成立，条目七（新表两份）仍成立，条目六（连着 OOM 等三种情形）仍一致、只是答法按改法变了，条目四（算键登记）本轮不判（正文已点名）；
   **条目二（放回的核对组把卡 1 的判器全换出）今天按代码读法成立**（推的，与判决引的读数对不上，探针一待主 agent 跑）；条目三的判两次、写错卡、写错流、停止信号仍不成立，但新设计在「撤回的块被另一张卡拿走头一截」之后丢了「不派回显存不够过的卡」那条限制（探针二）。
3. 第 3 问：从服务打行到任务服务发告警，每一环的字面逐环对得上；三处文字与行内字段不配（`control check` 与告警写死「已放给另一台去试」，不看 `claim=`）；两台都判不了时这一块停在判定库里没人认领、两边进程各自记着不再认领，知道的人是：两台 journal、`control check`（人跑）、任务服务告警（只发给名下有在排在跑任务的会话），判段自己不问 `control check`。
4. 第 4 问：三处修补各自成立，各留一个口子：重起之后头一回读到「没读成」时，上一回读到的没有可留的（内存里是空的），会话那一份「发过」被删，台读成之后重发；告警去重只落「告警写过」，不落「发给会话」；判不了的行每块一条告警，没有按流并；取不到卡号报错成立（口子是将来改格式时 `len(words) != 4` 的行被静默丢掉）。

## 第 1 问：三处改法在今天的代码里做的是不是判决「三、改法」写的那件事

整问判定：**成立**（三处各自做的是判决写的那件事），附三处与判决字面有出入的实现细节，和一处真卡上没执行过的**推不出**。

判决「三、改法」原文对应：第 1 条「丢掉判器之后、单独重判之前，先在那张卡上做一次空提交并等它做完」；第 2 条「`after_a_panic` 认出「因显存不够丢的设备」……这种情形重开这张卡的设备、卡不进 failed、只打接住 OOM 那一行；别的原因丢的才照旧进 failed」；第 3 条「判法自己的 panic 不算卡坏，算这一条流的毛病：这一块报拒、卡照常用」。

### 1.1 先做一次提交（改法第 1 条）

| 判决写的 | 代码做的 | 判定 |
|---|---|---|
| 丢掉判器之后 | 真卡那一份答 `RanOutOfMemory` 之前先清判器：`self.judges.clear();` 在 `DeviceAfterAPanic::RanOutOfMemory` 那一臂（`gpu-verification-server/src/gpu_judges_on_a_card.rs:441-443`） | 对得上 |
| 单独重判之前先在那张卡上做一次提交并等它做完 | 工作线程在报「接住」之前调「judges.settle_after_running_out_of_memory()」（`gpu-verification-server/src/card_worker.rs:270`），之后才报「report(CardWorkerReport::OutOfMemoryCaught {」（`gpu-verification-server/src/card_worker.rs:281`），再到「report(CardWorkerReport::RoundFinished { card });」（`gpu-verification-server/src/card_worker.rs:245`）。主循环只有收到「空出来了」才把这张卡放回 `idle`（「self.idle.insert(card);」，`gpu-verification-server/src/service_loop.rs:1117`），所以下一波（含单独判那一波、核对组、别的块）都排在那一次提交之后 | 对得上，范围比判决写的还宽（任何下一波，不只单独判那一波） |
| 提交要「让待写列表里的暂存与表放掉」 | 那一次提交借验证包「card.crc32c_of_ranges(&[nothing])」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:456`）。被借的函数提交一次并只等这一次：「let submission = self.queue.submit(std::iter::once(encoder.finish()));」（`crates/singlefs-verification/src/gpu_unit_checks/work_on_a_card_and_its_readback.rs:236`）、「let polled = device.poll(wgpu::PollType::Wait {」（`crates/singlefs-verification/src/gpu_unit_checks/work_on_a_card_and_its_readback.rs:241`）。入口处「if ranges.is_empty() {」（`crates/singlefs-verification/src/gpu_unit_checks/work_on_a_card_and_its_readback.rs:134`）只在 `ranges` 一段都没有时直接返回；这里传的是「一段、长度 0 的字节」，不是空切片，所以不走那个早退 | 对得上（提交与等待都真做；这一处是我核的，没有上卡量） |
| 没等成怎么办（判决没写） | `settle` 里 `catch_unwind` 接住 panic 回 `Err`；工作线程报「卡坏、设备没丢」并退出线程：文件头「那一次提交没等成的报这张卡坏了」（`gpu-verification-server/src/card_worker.rs:10`）对「report(failed(」那一支（`gpu-verification-server/src/card_worker.rs:271`） | 判决没写这一支；文件头与代码一致 |

三处与判决字面不同的细节：

- **不是「空提交」**：被借的是单元校验和那一格，建五个小缓冲再派一个工作组。若卡真满到连几十 KiB 都建不出，这一次提交自己又 OOM，OOM 在工作线程里同样 panic，被 `settle` 的 `catch_unwind` 接成「没等成」，**这张卡报坏**（device_lost=false）。判决写的「空提交」不分配、不会走这一支。量级（推的）：每回 OOM 之后卡上刚清掉全部判器，几十 KiB 够不到的情形要靠别的进程占满整卡；记作低概率口子，不是矛盾。
- **只有「设备没丢、记过 OOM」那一臂做 settle**：判法 panic 那一臂（「DeviceAfterAPanic::PanickedOtherwise => {」，`gpu-verification-server/src/gpu_judges_on_a_card.rs:445`）也先 `judges.clear()`、丢掉了装到一半的判器，但不 settle。判决第 1 条的机制（暂存与表挂在待写列表里，到下一次提交才放）对这一臂同样成立：装判器那一段 panic 时，同一波里前面已装上的判器的表与暂存同样没放。后果被下一回的 OOM 兜住（下一回 OOM 会走 settle），不是这一问要的缺口，列进「放过了」。
- **重开设备那一臂不 settle**：换了一个新设备，旧设备的待写列表不再碰，不需要；对得上判决第 2 条「重开这张卡的设备」。

什么现象会推翻 1.1：在真卡上构造一次 OOM，`after_a_panic` 返回之后立刻读 `nvidia-smi` 的显存占用，settle 前后占用没有下降；或者 `crc32c_of_ranges` 对长度 0 的一段早退不提交（我读到的是不早退）。

### 1.2 重开设备（改法第 2 条）

| 判决写的 | 代码做的 | 判定 |
|---|---|---|
| 照验证包 `cause_of_the_device_loss` 同一个判法，不另抄一份 | `device_after_a_panic` 调验证包的「condition.out_of_memory_explaining_a_failure_of_the_work(」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:250`）；验证包那一层的缘由来自「fn cause_of_the_device_loss(message: &str) -> CauseOfTheDeviceLoss {」（`crates/singlefs-verification/src/gpu_unit_checks/discovering_and_starting_the_cards.rs:38`）| 不另抄 |
| 认出「因显存不够丢的设备」 | 验证包的函数不止认「缘由是显存不够」：缘由是别的而丢之前记过 OOM 的也交「显存不够」那一句：「CauseOfTheDeviceLoss::GpuMemoryShort | CauseOfTheDeviceLoss::Other,」（`crates/singlefs-verification/src/gpu_memory_ledger.rs:1363`）。服务一侧文件头照实写了这一支（`gpu-verification-server/src/gpu_judges_on_a_card.rs:8`），测试钉着（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:1206`） | **比判决字面宽一格**：判决说「别的原因丢的才照旧进 failed」，代码里「为别的原因丢、但上一回 panic 之后记过 OOM」的也重开。理由站得住（验证包注释：wgpu 把建管线、提交上的 OOM 当致命、丢掉设备），不算错；但这一格没有写进判决，也没有真卡上的读数 |
| 这种情形重开这张卡的设备 | 「match self.reopen_the_device() {」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:431`）：清判器，照这张卡的 `nvidia-smi` 序号与额度另开：「gpu_cards_by_priority(&[(self.card.index, quota)])」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:317`），放进头一个空格；至多 4 次：「pub const DEVICE_REOPENINGS_OF_A_CARD: usize = 4;」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:121`） | 对得上；4 次上限判决没写 |
| 卡不进 failed、只打接住 OOM 那一行 | 重开成了报 `OutOfMemoryCaught`（原因里写重开了设备）：`gpu-verification-server/src/card_worker.rs:288-298` 那一臂，接着照常 `RoundFinished`；重开没成交 `DeviceLost`，进 failed | 对得上 |
| 别的原因丢的照旧进 failed | `LostOtherwise` 交 `DeviceLost`，工作线程报卡坏、线程退出：文件头「设备为别的原因丢了：报这张卡坏了」（`gpu-verification-server/src/card_worker.rs:14`） | 对得上 |

重开设备那一半在真卡上从没执行过（「推不出」，算打中）：`grep -rn 'reopen_the_device\|DEVICE_REOPENINGS_OF_A_CARD\|settle_after_running' gpu-verification-server/tests crates --include=*.rs`（我跑的只读 grep）在 `tests/` 里只命中替身那一份的 `settle_after_running_out_of_memory`；真卡那一份的 `after_a_panic`、`reopen_the_device`、`settle_after_running_out_of_memory` 没有测试调过，只有纯函数 `device_after_a_panic` 在假的设备情况上被测（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:1206`）。判决第 6 条记的「欠的测试」今天缩小了一半（纯函数那一层有了），外壳、重开、提交三样仍欠。

重开设备在真卡上有两处代码读得出的风险（推的，没量）：

- 旧设备不放：「丢了的那一个设备与它之前开的照旧持着」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:17`）。新设备起之前，`gpu_cards_by_priority` 照此刻空闲判额度：「card_memory_once_a_judge_fits(discovered, Some(*quota))」（`crates/singlefs-verification/src/gpu_unit_checks/discovering_and_starting_the_cards.rs:439`）；空闲不够、加回「这个进程自己占着的」够时，**最多等 600 秒**让这个进程还：「std::time::Duration::from_secs(600);」（`crates/singlefs-verification/src/gpu_unit_checks/card_memory_for_the_judges.rs:24`）。服务自己持着旧设备、不会还，所以这一支只会等满 600 秒再报「重开没成」，期间这张卡的工作线程不动、主循环看它「在判」、没有任何一行；
- 服务一侧给这张卡记的额度是起服务那一刻读的（`cards_of_the_service` 一次），重开之后新设备按当时空闲另算一个额度，服务的账不重读。

什么现象会推翻重开那几句：在真卡上让设备因显存不够丢一次，量重开的耗时与重开之后的额度与账上的额度是否一致。

### 1.3 判法 panic 不算卡坏（改法第 3 条）

| 判决写的 | 代码做的 | 判定 |
|---|---|---|
| 判法自己的 panic 不算卡坏 | 真卡答 `PanickedOtherwise`（`gpu-verification-server/src/gpu_judges_on_a_card.rs:445`）；工作线程报「report(CardWorkerReport::JudgePanicked {」（`gpu-verification-server/src/card_worker.rs:302`）后落到 `RoundFinished`，线程不退（`took_the_panic` 交 `true`） | 对得上 |
| 算这一条流的毛病：这一块报拒 | 主循环 `take_a_judge_panic`（`gpu-verification-server/src/service_loop.rs:1575`）按 `panicked_while` 分：装哪条流、判哪批的块整块报拒；「PanickedWhile::Verifying(_) => false,」（`gpu-verification-server/src/service_loop.rs:1599`）换出与核时 panic 不挑块 | 对得上；换出时 panic 的不挑块、这一波的截全撤回，判决没写 |
| 卡照常用 | `self.residents.remove(&card)` 清账、卡走 `RoundFinished` 回 `idle` | 对得上 |
| 之后怎么处置交用户定 | 用户选甲，落成 `ServiceRefusal::JudgePanicked`、放认领、判不了那一行（第 3 问逐环对） | 对得上 |

### 1.4 文件头四种情形与 `card_worker_loop`、`after_a_panic` 逐句对

文件头第 6 到 14 行写四种情形；`took_the_panic` 的臂序是一臂一种。

| 文件头 | `took_the_panic`（`gpu-verification-server/src/card_worker.rs:250` 起）与 `after_a_panic` | 判定 |
|---|---|---|
| 「判的时候（或核的时候）panic，先问判器缓存这张卡的设备怎样」（第 6 行） | 判一波与核一组各包 `catch_unwind`，出错都走 `took_the_panic`，第一件事「match judges.after_a_panic() {」（`gpu-verification-server/src/card_worker.rs:265`）；panic 的字句只带进 `reason` | 一致 |
| 设备没丢、记过 OOM：丢判器、先做那一次提交、再报接住、接着报空出来了（第 7 到 10 行） | 第一臂（`gpu-verification-server/src/card_worker.rs:266-287`），带守卫 `OutOfMemoryTakenForACardFailure`；先 settle、没等成报卡坏、成了才报 `OutOfMemoryCaught` | 一致 |
| 设备因显存不够丢了：已重开、报接住（原因里写重开了设备）、照常收；重开没成答设备丢了（第 11 行） | 第二臂（`gpu-verification-server/src/card_worker.rs:288-298`）原因拼成「{panic 的句}（{重开了设备那一句}）」；重开没成 `after_a_panic` 回 `DeviceLost`，走「设备丢了」那一臂（:319） | 一致 |
| 设备没丢也没记过 OOM：丢判器、报判法 panic（交回丢几个、哪一步、那一句）、接着报空出来了（第 12、13 行） | 第三臂（`gpu-verification-server/src/card_worker.rs:299-309`）；`panicked_while` 在每一步开始之前记：「let mut panicked_while = PanickedWhile::Evicting;」（`gpu-verification-server/src/card_worker.rs:219`）、「*panicked_while = PanickedWhile::Loading(load.stream);」（`gpu-verification-server/src/card_worker.rs:349`）、「*panicked_while = PanickedWhile::Judging(batch.pieces.clone());」（`gpu-verification-server/src/card_worker.rs:358`） | 一致 |
| 设备为别的原因丢了：报卡坏（带设备丢没丢、PCI 总线号与原因），线程不再收活（第 14 行） | 「设备丢了」那一臂（:319）报 `failed(…, true)` 返回 `false`；线程 `return` | 一致 |
| 弄坏开关四条各自对一臂（第 18 到 20 行） | 守卫分别在 :267、:269、:291、:300；开关未开时守卫为真走前面那一臂，开了落到末尾两臂（:310-318 与 :326-329） | 一致 |

逐句对下来**没有一句字面对不上**。两处读法上要说清的：

- 文件头第 11 行「设备因显存不够丢了」，验证包那一层比字面宽（见 1.2「比判决字面宽一格」）；
- 文件头第 18 到 20 行说「device-lost-for-want-of-memory-taken-for-a-card-failure 下因显存不够丢了、已重开设备的也报卡坏」，代码里这一臂报的是 `failed(…, true)`（设备丢了），但这个时候设备已经重开成功：弄坏开关打开后报告里的「设备丢了」与事实不符。开关只在自证里认，不影响平时。

### 1.5 第 1 问总判

- **成立**：三处改法做的是判决写的那件事。差别三格：settle 不是空提交而是一次单元校验和、只挂在「记过 OOM」那一臂；「因显存不够丢」的判法比字面宽（别的缘由加丢之前记过 OOM 的也重开）；重开设备那一半在真卡上从没执行过（**推不出**，算打中）。
- 弄坏开关与文件头第 18 到 20 行逐条对得上，开关名在 `break_switches.rs` 里都在（`grep -n` 命中四条，量的）。

## 第 2 问：判决「不成立、推翻或不采的」七条，逐条在今天的代码上再判一次

读法：判决第 43 行起列的七条（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:45` 到 `:51`）。之后换上的两样改动是：撤回的块别的卡也能拿（`put_back_a_piece` 把没交回结论的每一截另入池成一块撤回的块，`gpu-verification-server/src/service_loop.rs:1385`）与放认领给另一台（第 3 问）。

| 判决条目 | 今天再判 | 依据 | 什么现象会推翻 |
|---|---|---|---|
| 一：「攻方第 6 问前一半（卡 0 OOM 那一刻卡 1 的账与驻留判器被动到）：不成立」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:45`） | **仍不成立**（OOM 那一刻）。接住 OOM 只动这一张卡的账：「self.residents.remove(&card);」（`gpu-verification-server/src/service_loop.rs:1485`）；撤回走 `pieces_in_flight_by_card.remove(&card)`，别的卡的在途截不动。新设计让卡 1 在**随后的**一波里照调度拿撤回的块、按需换出自己的判器，这是改法第 6 条要的，不算「被动到」 | 同左 | 探针：卡 0 OOM 的那一刻卡 1 正在判一波，看卡 1 的 `residents` 与在途截有没有变（替身可观测：卡 1 的 `loaded` 在卡 0 接住前后一样） |
| 二：「攻方「放回的核对组把卡 1 的判器全部换出」：复跑推翻，重核派回卡 0」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:46`） | **今天按代码读法成立（推的，探针一待主 agent 跑）**。读法见下面 2.1 | 见 2.1 | 探针一输出 `answered_card=[CardIndex(0)]` |
| 三：「攻方第 5 问的判两次、写错卡、写错流、停止信号、借卡」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:47`） | **判两次、写错卡、写错流、停止信号仍不成立；借卡没读**。设计换了（撤回的是截不是整块），重判一遍见 2.2；另发现撤回的块被另一张卡拿走头一截后，没派完的后半截可以派回显存不够过的卡（2.3，探针二） | 见 2.2、2.3 | 见 2.2、2.3 |
| 四：「攻方第 10 问（两条 `UNKEYED_PROGRAM_WORDS` 登记）：不成立」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:48`） | **本轮不判**（正文「不在这一轮」：算键登记已挪进 `scripts/gate/lib_function_units_registry.py`，别的会话正在改）。只核了两条登记还在原函数上：「"SchedulerNotifying.read_gpu_cards_failed", "command"」（`scripts/gate/lib_function_units_registry.py:226`）、「"card_failed_lines", "paths.journalctl"」（`scripts/gate/lib_function_units_registry.py:250`） | 同左 | 不适用 |
| 五：「攻方第 12 问（多机撞键、漏一台）：不成立」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:49`） | **仍不成立**（台与进程这两栏）；两处小口子见 2.4 | 见 2.4 | 见 2.4 |
| 六：「正推第 2 问三种情形在代码上与文件头一致」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:50`） | **三种情形都还一致，但答法变了**，重判见 2.5；没在真卡上执行过这一句只剩外壳 | 见 2.5 | 见 2.5 |
| 七：「正推第 3 问「新表两份」三处调用、驻留账一份、对拍测试比的同一个量：成立」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:51`） | **仍成立**，调用处由三个变四个，见 2.6 | 见 2.6 | 见 2.6 |

### 2.1 放回的核对组与另一张卡的判器（条目二）

读法（每一步都是此刻的代码，都有行号）：

1. 核对组头一回派给卡 0，核对线程里 OOM：`take_an_out_of_memory` 把这一组放回排头，带「needs_an_emptied_card: true,」（`gpu-verification-server/src/service_loop.rs:1556`）。
2. 卡 0 在派那一组时已从 `idle` 拿掉：「self.idle.remove(&card);」（`gpu-verification-server/src/service_loop.rs:802`）；放回它的只有「空出来了」那一件：「self.idle.insert(card);」（`gpu-verification-server/src/service_loop.rs:1117`）。
3. 工作线程先报接住、紧跟着报空出来了（`gpu-verification-server/src/card_worker.rs:281` 与 `:245`），是两个事件。主循环每处理一个事件，都先问一次调度再收下一个：「let wait = self.dispatch_while_the_scheduler_says_so(commands, on_each_wave);」（`gpu-verification-server/src/service_loop.rs:500`）在「self.take_the_event(event);」（`gpu-verification-server/src/service_loop.rs:519`）之前。所以处理完「接住」之后、「空出来了」之前，卡 0 不在 `idle`。
4. 这一刻给放回的组挑卡只在 `idle` 里挑，且只试「换出全部驻留」：「|| (must_empty_the_card && evicted_count != order.len())」（`gpu-verification-server/src/service_loop.rs:873`）。卡 1 若空着、驻留着判器，就是卡 1：它的判器全换出，核完再装回，卡 0 空着。

判决引的读数是「实测派回卡 0、卡 1 换出 0 个」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:23`）。我的读法与它对不上。我解释不了差在哪（没有读到那一趟的树与产物，也没有在今天的树上量）：可能是我漏了一个让卡 0 在这一刻已空着的路径，也可能那一趟的代码与今天的不同。所以只写「推的」；探针一就是今天的树上做这一件事。

后果量级：不是判错，是卡 1 驻留的判器被换出、之后装回（重传表、重开草稿区），再加这一组核对的挂钟；没有结论错。

### 2.2 判两次、写错卡、写错流、停止信号（条目三）

今天的设计与判决当时不同：撤回的不是整块，是在这张卡上没交回结论的每一截，另入池成「撤回的块」，原来那一块留在池里（文件头第 17 行：「撤回的块照常交调度，只是不派回它显存不够过的卡」，`gpu-verification-server/src/service_loop.rs:17`）。逐项重判：

- **判两次**：不成立（读代码）。在途截只被取一次（`pieces_in_flight_by_card.remove(&card)`），每截只另入池一次（「.add_block(admitted.stream, piece.ordinals.clone(), admitted.entered_at)」，`gpu-verification-server/src/service_loop.rs:1401`）；撤回的块判出的结论沿血缘往上各记一次：「let mut lineage = vec![piece.block];」（`gpu-verification-server/src/service_loop.rs:1219`）；报拒撤掉的块进 `withdrawn`，之后交回的不记：「if self.withdrawn.contains(&piece.block) {」（`gpu-verification-server/src/service_loop.rs:1216`）。
- **写错卡**：不成立。记结论用的是交回那一批的卡（`take_the_verdicts_of_a_piece` 的 `card` 参数），血缘上每一层同一张卡（`gpu-verification-server/src/service_loop.rs:1227` 起的循环里 `card` 不变）。
- **写错流**：不成立。撤回的块用收下那一块的流：`add_block` 取 `admitted.stream`（:1401）。
- **停止信号**：不成立。主循环退出要等「&& self.replies.is_empty()」（`gpu-verification-server/src/service_loop.rs:502`），回话只挂在收下的那一块上（「self.replies.insert(block, reply);」，`gpu-verification-server/src/service_loop.rs:1036`），撤回的块不带回话，收下的那一块判完才回。
- **借卡**：没读（`card_lease.rs` 与借卡几份测试不在第 1 到 4 问里）；这几条路径上主循环不碰借卡键。
- 判决当时据的是攻方腿的探针 `out_of_memory_then_drain`、`split_block_put_back`（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:24`），那份探针不在今天的 `tests/` 里。今天 `tests/` 里守着「每个态只判一次」的只有两处：一是撤回成块那条（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:1333` 起，断言在「assert!(counts.values().all(\|count\| *count == 1), "每个态只判一次");」那一行，行号 `gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:1409`）与 `a_card_that_ran_out_of_memory_takes_the_next_wave_and_every_block_comes_back_judged_once`（断言行 `gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:745`，六块各 10 个态、卡 0 一次派活恰好 10 个态）。**两处都是块不大于判器一次派活的态数（40 块各 1 个态；六块各 10 个态）**，没有「撤回的块再被撤回」（血缘两层以上）与「撤回的块被拆在两张卡上」。血缘两层以上我读代码推得过（每层的序号是整条流里的序号，`record_verdicts` 在每一层照同一段序号记），但**没有测试钉着**：这一半记「推的」。

什么现象会推翻：一块 64 个态、卡 0 显存不够撤回、撤回的块被卡 1 拿去判前 16 个、卡 1 也显存不够：看这一块的每个态判了几次、回话几次（探针二的变形：给卡 1 也设毛病）。

### 2.3 撤回的块被另一张卡拿走头一截之后，没派完的后半截可以派回显存不够过的卡（新发现，判决没有）

判决第 6 条改法写的是「撤回的块……别的没坏、空着、装得下的卡也可以拿」。落地之后，「不派回它显存不够过的卡」这条限制只挂在「撤回在等」这个状态上：

- 撤回的块记进 `waiting_for_an_emptied_card`（「self.waiting_for_an_emptied_card.insert(part, origin);」，`gpu-verification-server/src/service_loop.rs:1422`）；
- 调度的候选里，在等的块只有在这张卡没在它显存不够过的那几张里才放进去：「!self.waiting_for_an_emptied_card.contains_key(&block.block)」（`gpu-verification-server/src/service_loop.rs:608`）那一句的右边才问 `cards_the_block_ran_out_of_memory_on`；
- 一块撤回的块被别的卡派出头一截就从「在等」里去掉：「self.waiting_for_an_emptied_card.remove(&piece.block);」（`gpu-verification-server/src/service_loop.rs:942`）。调度「长流按批切开、不一口气派完」（`gpu-verification-server/src/batch_scheduling.rs:4`），一块撤回的块比这张卡上判器一次派活的态数大时，头一波只派一截；
- 没派完的后半截此后是普通待判块，第 608 行的左边为真，不再问显存不够过的是哪几张卡，调度可以把它派回卡 0（刚 OOM 过的那张）。

后果（推的）：显存不够若是账与实际空闲不符（别的进程占着），卡 0 再装这条流的判器时再 OOM 一次：清全部驻留、settle 一次、这一截再撤回（这回「显存不够过的卡」里还是卡 0）。每个循环卡 1 至少又多判一批，所以会收敛，不会无限打转；但每回都白丢一次卡 0 的驻留判器。

探针二（`probe_a_put_back_block_the_other_card_took_part_of_is_not_given_back_to_the_card_that_ran_out_of_memory`）：卡 0 一次派活 64 个态、卡 1 一次派活 16 个态，一块 64 个态，卡 0 判它时显存不够（设两回）。要的结果是毛病只出一回；推的读数是出两回。输出行 `R2_DEFENSE_PROBE name=put_back_block_cut_by_the_other_card … faults_raised_on_the_card_that_ran_out=<n>`，n 为 2 就是打中，1 是没打中（我读错了）。

### 2.4 多机撞键、漏一台（条目五）

- **台与进程**：不成立仍成立。种类键带台与进程：「kind = f"{GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:{host}:{process}:{':'.join(named)}"」（`task-server/task_server_lib_configuration.py:73`）；第二台自己的行在 `control` 里被改写成那一台的地址：`cards_failed_of_the_peer` 读第二台打的 `host=this-host` 的行，在发起的这一台上用第二台的地址作台名（`gpu-verification-server/control:1359`）。两台各自 OOM、各自卡坏不会撞键。
- 两处小口子（推的，没造输入）：
  - 判不了的种类里没有「哪一份库」：两份登记的临时库里流键头与块号相同的两块会并成一个种类、少发一条；
  - 台名「this-host」相对任务服务所在的那一台（`task-server/task_server_lib_constants.py:491`），告警种类落了盘（`gpu_alerted`），任务服务若换到另一台起，旧种类里的「this-host」会指到另一台上同进程号同卡号的那一张。
- 什么现象会推翻「仍成立」：喂两行 `BLOCK_NOT_JUDGEABLE_OF_A_HOST`，台与进程都同、流键头与块号都同、库不同，种类相同（第一条小口子）。

### 2.5 正推第 2 问：连着 OOM、OOM 后判法 panic、OOM 与设备丢失同时到（条目六）

| 情形 | 判决当时 | 今天 | 判定 |
|---|---|---|---|
| 同一张卡连着 OOM 两回 | 两回都答 `RanOutOfMemory`，`seen` 在答之前先追平 | `seen` 照旧先追平：「self.out_of_memory_reports_seen = condition.out_of_memory_reports();」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:425`）；重开之后改成新设备的计数（:332） | 仍一致 |
| OOM 之后判法自己又 panic（两回之间没新记 OOM） | 落到 `PanickedOtherwise`、报卡坏 | 落到同一个 `PanickedOtherwise`（「DeviceAfterAPanic::PanickedOtherwise => {」，`gpu-verification-server/src/gpu_judges_on_a_card.rs:445`），**现在报判法 panic、卡不坏**（`gpu-verification-server/src/card_worker.rs:12` 文件头照实写） | 一致，答法按改法第 3 条变了 |
| OOM 与设备丢失同时到 | 丢失在先，答 `DeviceLost`，不清判器 | 丢失仍在先：「if let Some(lost) = condition.lost_reason() {」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:248`）；之后问验证包认不认得成显存不够：认得成的丢判器、重开（不进 failed），认不成的才 `DeviceLost` | 一致，答法按改法第 2 条变了；重开之后的 OOM 计数起点是新设备 |

「没在真卡上执行过」：外壳（`after_a_panic`、`reopen_the_device`、`settle_after_running_out_of_memory`）仍没有测试调过；纯函数 `device_after_a_panic` 有（1.2 末）。这一条的「成立」仍是读代码得的。

### 2.6 正推第 3 问：新表两份（条目七）

- 三处调用今天是四处：「let tables = self.tables_while_loading(streams);」（`gpu-verification-server/src/batch_scheduling.rs:378`）、「let tables = self.tables_while_loading(&reloaded);」（`gpu-verification-server/src/batch_scheduling.rs:541`）、「let tables = self.tables_while_loading(&newcomers);」（`gpu-verification-server/src/batch_scheduling.rs:705`），加单独判那一块的「building.tables_while_loading(&[stream])」（`gpu-verification-server/src/batch_scheduling.rs:780`）。
- 驻留账仍记一份、对拍测试仍比同一个量：「mebibytes_of_a_judge(&facts, requested),」（`gpu-verification-server/tests/card_memory_matches_the_verification_package.rs:30`）。
- 新加的核对器借显存按「表两份」算：「let copies_of_the_tables = if break_switches.is_on(BreakSwitch::FactsTablesStagingNotReserved) {」（`gpu-verification-server/src/card_memory_ledger.rs:167`）；与判器那边一致（判决第 3 条的修补）。
- **仍成立**。什么现象会推翻：`mebibytes_of_a_judge` 之外的任何地方把表算两份记进驻留账（`grep -n 'while_loading' gpu-verification-server/src/card_memory_ledger.rs` 出现在 `admit` 里）。

### 2.7 第 2 问总判

整问判定：**有变成成立的**——条目二（推的，探针一待跑）；条目三里的判两次、写错卡、写错流、停止信号仍不成立，但新增一个口子（2.3，探针二，推的）；借卡没读（推不出，算打中）。其余各条见表。

| 条目 | 今天 |
|---|---|
| 一 | 仍不成立 |
| 二 | **今天按代码读法成立**（推的，与判决引的读数对不上，探针一待跑） |
| 三 | 判两次、写错卡、写错流、停止信号仍不成立；借卡没读；血缘两层以上与拆在两张卡上的撤回块没有测试钉着；撤回的块后半截可派回显存不够过的卡（新发现，探针二） |
| 四 | 本轮不判 |
| 五 | 仍不成立，两处小口子（推的） |
| 六 | 一致，答法已按改法变；外壳没在真卡上执行过 |
| 七 | 仍成立（三处调用变四处） |

## 第 3 问：判决第 2 条与甲案：从服务打判不了行到 `control check`、`control cards-failed`、任务服务解析与发告警

整问判定：每一环与下一环的**字面对得上**（成立）；**另一台也判不了时，这一块停在判定库里没人认领**，知道的人有三路（见 3.3），判段自己一路也不问（推不出判段会不会因此被叫醒，不是这条链的）。

### 3.1 逐环字面

| 环 | 上游打的 | 下游读的 | 判定 |
|---|---|---|---|
| 服务打行 | 「"{BLOCK_NOT_JUDGEABLE_LINE_PREFIX} family={} stream={} block={} card={} why={} claim={} reason=\"{}\""」（`gpu-verification-server/src/serving_from_the_library.rs:227`）；`eprintln!` 进单元日志（`print_a_service_line`，`gpu-verification-server/src/library_pending_pool.rs:736`）；原因里的换行与双引号被换掉（`reason_on_one_line`，`gpu-verification-server/src/serving_from_the_library.rs:239`） | — | — |
| → `control` 读日志 | — | `card_failed_lines` 读单元此刻主进程的行：「def card_failed_lines(paths):」（`gpu-verification-server/control:1002`），判不了的开头与服务的常量逐字相同：「BLOCK_NOT_JUDGEABLE_LINE_PREFIX = 」（`gpu-verification-server/control:285`）对「pub const BLOCK_NOT_JUDGEABLE_LINE_PREFIX」（`gpu-verification-server/src/serving_from_the_library.rs:109`），开头后带空格匹配；自证从源码读来核 | 对得上 |
| `control check` | — | 「def card_failure_refusal(paths):」（`gpu-verification-server/control:1050`）判不了的行单列退 1，列族、流、块、卡、为什么、认领、原因（`card_failed_fields` 的正则「CARD_FAILED_FIELD = re.compile」，`gpu-verification-server/control:298`，值带双引号的取到引号止，与服务写的 `reason="…"` 对得上） | 对得上 |
| `control cards-failed` | `cards_failed_answer` 每台一行读状态、判不了的行原样加前缀：「def cards_failed_answer(outcomes):」（`gpu-verification-server/control:1378`）；第二台经 ssh 起同一条、读它 `host=this-host` 的行，换成第二台的地址（「def cards_failed_of_the_peer(paths, host):」，`gpu-verification-server/control:1359`） | 任务服务的前缀常量与它逐字相同：「GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX = 」（`task-server/task_server_lib_constants.py:489`）对「BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX = 」（`gpu-verification-server/control:292`）；`GPU_VERIFICATION_SERVER_BLOCK_NOT_JUDGEABLE_OF_A_HOST` 是 `..._BLOCK_NOT_JUDGEABLE` 加 `_OF_A_HOST`，前缀判一律带空格（`is_a_block_not_judgeable_line`，`gpu-verification-server/control:1045`），不会把两者认混 | 对得上 |
| 任务服务解析 | — | 「def gpu_service_report(text, environment=None):」（`task-server/task_server_lib_configuration.py:39`）按前缀、每行拆成四段（「if len(words) != 4 or words[0] not in (GPU_FAILED_CARD_OF_A_HOST_PREFIX, GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX):」，`task-server/task_server_lib_configuration.py:52`），认块的三栏「GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS = ("family", "stream", "block")」（`task-server/task_server_lib_constants.py:495`）都在服务打的那一行里（上一行的前三个字段）；自证 `gpu-card-lines-match-the-service-fields` 从源码读字段名核（`task-server/task_server_selftest_notifying.py`） | 对得上 |
| 发告警 | — | `gpu_card_failures` 把每个种类变成服务健康的一样（`task-server/task_server_scheduler_notifying.py:725`），`watch_health` 发给名下有在排在跑任务的会话，写告警记录（「self.gpu_cards_failed_alerted.add(kind)」，`task-server/task_server_scheduler_notifying.py:803`） | 对得上 |

字面对不上的三处（文字，不是字段）：

- `control check` 与任务服务告警都写死「已放给另一台去试」，不看行里的 `claim=`：「这一台（{host}）的 gpu-verification-server 此刻的进程有 {len(not_judgeable)} 块判不了：已放给另一台去试」（`gpu-verification-server/control:1062`）。`claim=kept`（放认领时读写判定库失败，或弄坏开关）与 `not-held`（放的时候已不在自己手里）时这句话不对。
- 单台配置（只点名一台的卡）也写「已放给另一台去试」。
- 任务服务告警的句子「已放给另一台去试」写在每一台各自的告警里：两台各报一块时，第二台那一条读起来还是「放给另一台」，而那一块此刻没有另一台可以放了（3.3）。

### 3.2 另一台也判不了：这一块最后停在哪

读法（今天的代码）：

1. 这一台的主循环拒了这一块、理由是「在腾空的卡上也 OOM」或「判法 panic」：`not_judgeable_here` 把这两种拒认成这一台判不了（`gpu-verification-server/src/serving_from_the_library.rs:1526`），其余的拒（卡坏、没登记、停止中……）不算。
2. 放认领：「.release_the_claim_left_to_another_service(library, &name),」（`gpu-verification-server/src/serving_from_the_library.rs:1213`）；待判行不删、留在库里；打一行；把这一块记进这个进程内存里的集合：「self.refused_for_running_out_of_memory」（`gpu-verification-server/src/serving_from_the_library.rs:1230`，类型「refused_for_running_out_of_memory: BTreeSet<(LibraryOfTheService, PendingBlockName)>,」在 `gpu-verification-server/src/serving_from_the_library.rs:493`），之后扫库挑候选时不再认领它；另两族各有一个同样的集合。
3. 另一台的服务随即取得到、同样判不了时走同样三步。两边都把这一块记进了各自进程的集合。
4. 此后：待判行在库里，没有认领，两台都不再认领它。**不会来回弹**：每台每个进程对每一块至多试一次，放认领与记集合在同一个线程的同一圈里，中间没有扫库（所以放了认领之后这一台自己也不会抢回来）。重起任一台的服务，那一台的集合清空、会再试一次、再打一行（种类里的进程号变了，告警会再发一条）。
5. 判段自己：它等的是「待判行不在了、结论在库里」，登记、写行与等的时候都不问服务收不收（`crates/singlefs-verification/src/crash_amplification/gpu_verification_server.rs:10` 的「登记、写行与等的时候都不问服务收不收」），等不到的由任务服务照卡住停（同一文件第 11 行「判段等不到的由统一任务服务照卡住停」）。进度行「还剩的块数没变的不打」（`crates/singlefs-verification/src/crash_amplification/gpu_verification_server.rs:25`）：这一块卡住时，判段一侧没有新的输出。

所以，这一块**最后停在判定库里（待判行在、没人认领）**，等的任务在等。

### 3.3 有没有人知道

| 路 | 谁会知道 | 条件与口子 |
|---|---|---|
| 两台各自的服务日志 | 看 `journalctl` 的人 | 每块一行，`claim=released` |
| `control check`（任一台起，经 ssh 问另一台） | 跑它的人 | 判段与任务服务进程不跑它（`task-server/control check` 这条命令会接着跑它、退非 0 时退 78，`task-server/README.md` 第 14 行）；`cards-failed` 与它同源。两台任一台打过判不了行就退 1，直到那个进程重起（`control` 读的是单元此刻主进程的全部日志，`gpu-verification-server/control:1002` 起），**一块后来被别的台判成了也不消**（这一台的行还在） |
| 任务服务告警 | 名下有在排在跑任务的会话（`task-server/task_server_scheduler_notifying.py:683` 的 `health_recipients`：状态不是 finished、非分片子任务、收件 socket 找得到的会话） | 判段那条任务在跑，它的会话在名单里，所以判段的会话会收到；每 60 秒读一回（`task-server/task_server_constants.py:581`）。没有一个会话有未完成任务时，`watch_health` 不查、也就不读 GPU 服务（`task-server/task_server_scheduler_notifying.py:784` 的 `recipients` 为空就返回） |
| 任务服务「卡住」判定 | 任务服务 | 判段等的是 I/O，不吃 CPU；是否被它判成卡住是另一条链，我没读 |

判决第 2 条原文「判段只问 `control serves`，`control check` 不读这种情形，等它的那几片一直等到被任务服务打回」（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:32`）：前半句今天不成立（`control serves` 已删：`grep -c serves gpu-verification-server/control` 得 0，量的），后半句今天换成了「判段不问 `control check`，`control check` 读得到判不了」，等到被任务服务打回这一条没变。**第 2 条要修的洞（没人知道）经甲案补了一半**：任务服务告警让有任务的会话知道；判段那条任务本身仍是靠「卡住停」收尾。

两台同块都判不了时，告警是两条（各台一条，种类里台名与进程号不同），出路句（`task-server/task_server_constants.py:586` 起的 `GPU_BLOCK_NOT_JUDGEABLE_NEXT_STEP`）写了「两台都报了同一块的，主 agent 照问题里的族、流键头与块号查那条流……交用户定」，所以收到两条的会话有这条出路；没有一条把「两台都放弃了」写成一句话：靠人读两条自己对。

另外一处（放过了，判决没有）：判不了的行**每块一行**，任务服务的种类里带块号（`task-server/task_server_lib_configuration.py:73`），一条坏流的几百块就是几百个种类；`watch_health` 对每个种类、每个会话各发一条（「for kind, name, line in problems:」，`task-server/task_server_scheduler_notifying.py:798`）。没有按流或按台并成一条。去重只防「同一个种类重复发」，防不了「很多种类各发一回」。（推的，没造输入；输入见第 4 问的探针说明。）

什么现象会推翻「停在判定库里」：另一台的集合在进程之间有共享（我读到的是各进程内存里的 `BTreeSet`，没有落盘）；或放认领之后这一台的扫库认领不看集合（我读到的看，`refused_for_running_out_of_memory` 在候选过滤里）。

## 第 4 问：第 9、10 条修补对着 `gpu_service_problems_merged` 与 `read_gpu_cards_failed` 判

判决第 9、10 条的改法（`research/prompts/gpu-oom-card-governance-r1-main-verification.md:64`）：读不到时打明一行「没读成」，任务服务保留上一回的；告警去重落盘、重起不重发；卡号取不到报错。整问判定：三处**各自成立**；各留一个口子，其中重起与没读成叠在一起的那一个口子**打中**了「重起之后不重发」。

### 4.1 没读成的台留着上一回读到的

- 控制一侧：`cards_failed_answer` 给没读成的台（unreadable、unreachable）另打一行 `CARDS_FAILED_UNREAD`（「def cards_failed_answer(outcomes):」，`gpu-verification-server/control:1378`）；第二台连不上、没回、退非 0、读不出状态行都记 unreachable（`gpu-verification-server/control:1359` 起的 `cards_failed_of_the_peer`）；这一台日志读不了记 unreadable（`gpu-verification-server/control:1346` 起的 `cards_failed_here`）。两个前缀与任务服务的常量逐字相同（`task-server/task_server_lib_constants.py:490` 对 `gpu-verification-server/control:293`）。
- 任务服务一侧：「def gpu_service_problems_merged(previous, found, unread, environment=None):」（`task-server/task_server_lib_configuration.py:78`）以这一回读到的为底（「merged = {kind: (line, host) for kind, line, host in found}」，`task-server/task_server_lib_configuration.py:81`），再把上一回里台在 `unread` 里的补回去（「if host in unread:」，`task-server/task_server_lib_configuration.py:84`）；`read_gpu_cards_failed` 起不了、超时、退非 0 的整份留着上一回的（`found` 保持 `None`，不换）：
  「self.gpu_cards_failed = lib.gpu_service_problems_merged(self.gpu_cards_failed, found, unread, self.environment)」（`task-server/task_server_scheduler_notifying.py:772`）只在 `found is not None` 时执行。认不出的行所在的台也当没读成：「unread |= {line.split(" ", 2)[1].partition("=")[2] for line in unparsed if len(line.split(" ", 2)) > 1}」（`task-server/task_server_scheduler_notifying.py:763`）。
- 判定：**成立**（进程不换的这一段里）。自证格 `gpu-cards-unread-taken-as-healed` 钉着（`task-server/task_server_selftest_notifying.py` 里 `cell_gpu_service_report_keeps_unread_hosts`）。

三个口子：

1. **重起之后头一回就没读成**（打中，读代码）。`previous` 来自内存：「self.gpu_cards_failed = []」（`task-server/task_server.py:1054`）。重起之后它是空的，没有可留的；而头一回读完就把「读完过一回」置真，不看有没有台没读成：「self.gpu_cards_failed_known = True」（`task-server/task_server_scheduler_notifying.py:774`，`found is not None` 就置）。`watch_health` 里只在「读完过一回」之前保护 GPU 那几样：「if not gpu_known and kind.startswith(GPU_HEALTH_KIND_PREFIXES):」（`task-server/task_server_scheduler_notifying.py:794`）；置真之后，没在这一回 `found` 里的种类被忘掉：「del self.health_notified[kind]」（`task-server/task_server_scheduler_notifying.py:797`）。`health_notified` 是落盘的（`task-server/task_server_scheduler_lifecycle.py:133` 起一支载入）。所以：重起之前第二台一张坏卡发过；重起后头一回读，第二台正好连不上（`unreachable`），没读成那一台在 `merged` 里没有上一回可补，种类不在 `found`，被忘掉；第二台读成之后同一张坏卡对每个会话**再发一回**（告警记录不会再写：「gpu_alerted」落盘了，`task-server/task_server_scheduler_notifying.py:803`、`:804`）。现有自证格 `cell_gpu_health_survives_restart`（`task-server/task_server_selftest_notifying.py:425`）只覆盖「头一回读完之前」与「读到同一张卡」，没有「头一回读完、有台没读成」这一格。

   输入（要由自证格做，任务服务的库文件与 `task_server_lib` 只许服务读写，探针脚本载入它会被项目代码闸拒，所以我没有写成脚本）：在 `cell_gpu_health_survives_restart` 的套路上，换服务之前造一张第二台的坏卡并发过；换服务之后头一回读的假 `cards-failed` 打「这一台 read lines=0；第二台 state=unreachable lines=0」加一行 `CARDS_FAILED_UNREAD host=<第二台> state=unreachable`；期待 `health_notified` 里第二台那一种类还在、`sent` 为空；读代码推的结果：被删，之后第二台读成时再发一回。
2. **一直读不成的台没有人知道**（推的，读代码）：`gpu_card_failures` 只交读到的种类与认不出的行（`task-server/task_server_scheduler_notifying.py:725` 起），`unread` 只进 `merged`，不单独成一样问题；读不成往服务日志写一行（`say`，「✗ 读 GPU 服务的卡坏」，`task-server/task_server_scheduler_notifying.py:765`）。第二台连不上由另一样「第二台用不了」盯；这一台的日志读不成（`JOURNAL_READ_SECONDS = 120`，`gpu-verification-server/control:280`，任务服务的限时 150 秒，`task-server/task_server_constants.py:582`）没有另一样盯：`control check` 会退 1 并说「判不了有没有卡坏」，任务服务不告警。
3. 「没在跑」不算没读成：`not-running` 是读成了、没有行，上一回的当成好了（`control` 文件头「not-running 不算没读成：服务重起之后是新进程、新的键」）。这是判决当时就定的口径，不是缺口；只是服务崩了而不是重起时，这一条告警被当成好了，改由「单元不在」那一样顶。

什么现象会推翻口子 1：上面那个自证格输入下，`health_notified` 里第二台的种类没被删、`sent` 为空。

### 4.2 去重落盘

- 告警去重：种类写进集合再写告警，落盘在前：「self.gpu_cards_failed_alerted.add(kind)」（`task-server/task_server_scheduler_notifying.py:803`）、「self.persist_restart_surviving_state("gpu_alerted")」（`task-server/task_server_scheduler_notifying.py:804`）；起来载入（`task-server/task_server_scheduler_lifecycle.py:146` 起一支）。成立。性质：同一个种类（台、进程号、卡号；或台、进程号、族、流键头、块号）永远只写一回；先记再写，写的当中服务没了就那一条不再补（代码注释自己写了）；集合只增不减。
- 会话一侧的去重：`health_notified` 也落盘，但两处会清：这一回没查出的种类被忘掉（4.1 口子 1），没有收件的会话时整份清掉：「if not recipients:」（`task-server/task_server_scheduler_notifying.py:785`）里的「self.health_notified.clear()」（`task-server/task_server_scheduler_notifying.py:786`）。所以「去重落盘」对告警记录成立，对发给会话的不完全。
- 刷屏（判决没有）：见 3.3 末段，种类带块号。

### 4.3 卡号取不到报错

- 卡坏行取不到 `card=`：「if card is None:」后进认不出的行（`task-server/task_server_lib_configuration.py:63`、`unparsed.append(line)` 在 :64）；判不了的行取不到族、流键头、块号三栏之一同样：「if any(value is None for value in named):」（`task-server/task_server_lib_configuration.py:70`）。不再退成「?」。认不出的行所在的台当没读成（4.1），并一样「GPU 服务的卡坏行认不出」：「problems.append((GPU_LINES_UNPARSED_KIND, "GPU 服务的卡坏行认不出",」（`task-server/task_server_scheduler_notifying.py:747`）。自证格从源码读字段名核（`task-server/task_server_selftest_notifying.py:382` 的 `gpu_service_line_fields_of_the_source`）：服务一侧改了字段名，格红。**成立**。
- 三个口子（推的）：
  - 识得前缀、但拆出来不是四段的行被静默丢掉：「if len(words) != 4 or words[0] not in (GPU_FAILED_CARD_OF_A_HOST_PREFIX, GPU_BLOCK_NOT_JUDGEABLE_OF_A_HOST_PREFIX):」（`task-server/task_server_lib_configuration.py:52`）后面是 `continue`，不进认不出的行；`control` 现在总打四段，所以是将来改格式的口子，不是今天的毛病；
  - 认不出的行只有一个固定种类，不写告警记录（`alert_kind` 取不到，`task-server/task_server_scheduler_notifying.py:799`）：不同的认不出的行前后各出一条，第二条不再发；
  - 台名「this-host」相对任务服务所在的那一台（2.4）。

## 判决哪一条判重了、哪一条放过了

### 判重了（同一件事记成两条，或两条修的是同一处）

| 判决条目 | 重在哪 | 依据 |
|---|---|---|
| 第 11 条里「README「设备丢了才报卡坏」漏了判法自己 panic 也报卡坏」 | 与第 5 条同一个事实的文档面（`after_a_panic` 把判法 panic 报成卡坏）。修了第 5 条这句话就跟着改，不该单记一条；第 11 条别的三样（第 19 行、撤回块「先腾空」那句、`control` 注释点名的常量）是独有的 | `research/prompts/gpu-oom-card-governance-r1-main-verification.md:35` 与 `:41` |
| 第 6 条与「不成立」里的「正推第 2 问……并进判决第 6 条」 | 同一件事（真卡 `after_a_panic` 没执行过）在「坐实」与「不成立」两处各记一次，数「十一条坐实、七条不成立」时各多算一条；判决「二」里那一句本来就写了「并进判决第 6 条」 | `research/prompts/gpu-oom-card-governance-r1-main-verification.md:36` 与 `:50` |

其余坐实的条目（1 到 5、7 到 10）每一条对应一处独立的代码或文字，我没有找到第二处重叠。第 2 条与第 5 条落成同一套机制（放认领、判不了行、`control check` 判红、告警），但输入不同、各自的缺口不同，不算判重。

### 放过了（判决没有，或判决「不成立」今天不成立了）

| 条目 | 位置 | 性质 |
|---|---|---|
| 放回的核对组落到刚好空着的另一张卡、把它的判器全换出 | 2.1，`gpu-verification-server/src/service_loop.rs:802`、`:873`、`:1117`、`:1556` | 判决「不成立」第二条；今天按读法成立（推的），与判决引的读数对不上，探针一 |
| 撤回的块被另一张卡拿走头一截之后，后半截可再派回显存不够过的卡 | 2.3，`gpu-verification-server/src/service_loop.rs:608`、`:942` | 改法第 6 条的落地口子；探针二 |
| settle 是一次单元校验和不是空提交，它自己 OOM 会报卡坏；判法 panic 那一臂不 settle | 1.1，`gpu-verification-server/src/gpu_judges_on_a_card.rs:456`、`:445` | 低概率口子；机制同判决第 1 条 |
| 重开设备：旧设备持到服务停、重开时可能等满 600 秒、账上额度不重读、至多 4 次 | 1.2，`gpu-verification-server/src/gpu_judges_on_a_card.rs:17`、`:121`；`crates/singlefs-verification/src/gpu_unit_checks/card_memory_for_the_judges.rs:24` | 推的，真卡上没执行过 |
| 「因显存不够丢」的判法比判决字面宽一格 | 1.2，`crates/singlefs-verification/src/gpu_memory_ledger.rs:1363` | 判决没写，测试钉着 |
| 重起之后头一回读就有台没读成：上一回读到的没有可留的、发给会话的那一份被忘掉，台读成后重发 | 4.1 口子 1，`task-server/task_server_scheduler_notifying.py:774`、`:794`、`:797` | 打中判决第 9 条「重起之后不重发」；要自证格 |
| 一直读不成的台没有人知道 | 4.1 口子 2 | 推的 |
| 判不了的行每块一行、告警种类带块号：一条坏流给每个会话一堆条 | 3.3 末段，`task-server/task_server_lib_configuration.py:73`、`task-server/task_server_scheduler_notifying.py:798` | 推的，没造输入 |
| `control check` 与告警写死「已放给另一台去试」，不看 `claim=`；两台都放弃时没有一句话说「两台都放弃了」 | 3.1，`gpu-verification-server/control:1062` | 文字 |
| 判不了的行与 `control check` 的红在进程活着时一直在，一块后来被另一台判成了也不消 | 3.3 | 文字与口径 |
| 判决「二」第 2 条里「判段只问 `control serves`」已不成立：`control serves` 已删，判段现在不问服务收不收，也不问 `control check` | 3.3 | 判决正文要更新，不影响改法 |
| 「撤回的块再被撤回」（血缘两层以上）与「撤回的块拆在两张卡上」没有测试钉着 | 2.2 | 推的 |
| 判决第 6 条的「欠的测试」今天缩小：纯函数 `device_after_a_panic` 有测试，外壳、重开、settle 仍欠 | 1.2 | 判决要改写这一条 |

## 要主 agent 跑的命令

两条探针都不上卡、不起临时判定库、不借真卡、不动两台在跑的服务；cargo 只点名两个测试函数，不跑整个包。预期读数都是「推的」。

1. 先建运行目录（不交服务）：`mkdir -p /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet-probe-run`
2. 交（在哪起：`--cwd` 就是上面那个运行目录；运行目录名以 `-run` 收尾、在工作根下，外层脚本只清空它的内容、不删目录本身，别的目录一概不动退 2；外层脚本文件头第 2 行有会编声明）：

   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet-probe-run -- bash /home/fy5090/code/singlefs/research/prompts/gpu-oom-card-governance-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet/frozen /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet-probe-run /home/fy5090/code/singlefs/research/prompts/gpu-oom-card-governance-r2-sonnet-model`

   看哪几行：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`task-server/task log <号>` 里：
   - `R2_DEFENSE_PROBE name=facts_job_put_back_lands … answered_card=[CardIndex(1)]`：条目二今天成立（打中）；`answered_card=[CardIndex(0)]`：我的读法错、判决那一趟的读数对。这一条的末尾断言在打中时红（`test … probe_a_facts_job_put_back_after_running_out_of_memory_stays_off_the_other_cards_judges … FAILED`）；
   - `R2_DEFENSE_PROBE name=put_back_block_cut_by_the_other_card … faults_raised_on_the_card_that_ran_out=2`：2.3 打中；`=1`：没打中；`judged_more_than_once=0` 是每个态只判一次；
   - 取样断言红（`cards_that_loaded` 不是 `[0, 1]`、`out_of_memory_caught` 小于 1）是机器忙、第二块晚到，重跑，不算推翻；
   - 预期（推的）：整条 `exit=101`，两个测试各有一条断言红，输出行都打在红之前。
   探针的内存与时间：我没量过（不交任务）；编的是 `gpu-verification-server` 的一个测试目标，与那个包里别的测试目标同量级，上限由服务给；跑起来是替身线程，秒级。
3. 用完删：`/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet-probe-run`（服务跟着删它那一组）与 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet/frozen`（我建的冻结副本，64M，`.deliverables` 写了「交接」）。

没写探针的两处（要主 agent 另派同类 agent 或交工具员写成任务服务的自证格）：第 4 问口子 1（`cell_gpu_health_survives_restart` 的变形，输入见 4.1）、第 3 问判不了行刷屏（喂 `gpu_service_report` 几百行同流不同块的判不了行，数解出的种类数；纯函数，要放进自证格里）。直接 `python3` 载入 `task_server_lib` 的探针脚本被项目代码闸拒（`绕开 task-server 的写法被拒：在代码里载入 task-server/task_server_lib.py`，我试 grep 它时撞到的），所以我没写成脚本。

## 什么现象会推翻我的判定（汇总）

| 我的判定 | 推翻的现象 |
|---|---|
| 条目二今天成立（推的） | 探针一 `answered_card=[CardIndex(0)]` |
| 2.3 后半截可派回卡 0（推的） | 探针二 `faults_raised_on_the_card_that_ran_out=1` |
| 重开设备 600 秒等（推的） | 真卡上让设备因显存不够丢一次、量重开耗时，短于一分钟且 `nvidia-smi` 上这个进程占用不含旧设备那一份 |
| settle 自己可能 OOM（推的） | 真卡上量 `crc32c_of_ranges(&[nothing])` 的分配，小于当时空闲的最小块 |
| 4.1 口子 1 | 自证格输入下 `health_notified` 里第二台的种类没被删 |
| 3.2「停在判定库里」 | 另一台的「不再认领」集合在进程之间共享或落盘（我读到的是各进程的内存集合） |

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务（`task-server/task` 只读 `status`、`log` 都没用上）；判决要引的数没复跑：探针一、二的读数都是「推的」，命令在上一节。
- 没编、没跑 cargo；探针只过了 `rustfmt --edition 2021 --check` 的语法解析（去掉 `mod common;` 那一行的差异之外没有别的差异）。
- 没读：禁读清单里的攻方腿报告与模型；`break_switches.rs` 的开关逐条（只核了第 1 问点名的四条名字在）；`card_lease.rs` 与借卡几份测试；`serving_from_the_library.rs` 里扫库认领的其余部分；任务服务「卡住」判定那一条链。
- 没判：算键登记（正文「不在这一轮」，只核了两条登记还在原函数上）；真卡上跑 `after_a_panic`（判决第 6 条，接口待定）；`control serves`（已删）。
- 没在真卡上量任何一个数；wgpu 的资源释放与重开设备的耗时都是读代码推的。
- 没改被判文件、没改任何仓内文件，只写了本报告、模型目录（`research/prompts/gpu-oom-card-governance-r2-sonnet-model/`）与草稿目录。

## 清理（用完就删）

- 已删：草稿目录里的 `syntax-check/`（80K，量的）与 `minimal-copy.txt`（4K，量的）。
- 留着、交主 agent：冻结副本 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-sonnet/frozen`（64M，量的），草稿目录的 `.deliverables` 登了「交接」；主 agent 复跑完整个草稿目录删掉。运行目录 `…-probe-run` 是主 agent 交的时候才建，我没建。
