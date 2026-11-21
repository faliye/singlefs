# GPU 服务接住 OOM、卡坏上报与借卡：治理文件轮第二轮攻方腿（2026-11-21）

攻击面：正文「要腿判的」攻方腿第 5 到 13 问（`research/prompts/_gpu-oom-card-governance-r2-body.md`）。读的是此刻主工作区那一版：被判的 20 份文件与 `research/prompts/gpu-oom-card-governance-r2-snapshot/judged-sha256.txt` 逐个 `sha256sum -c` 都是 OK。禁读清单里的辩方腿报告与模型没读。第一轮攻过而没打中的四个角度（卡 0 不够那一刻卡 1 的账与驻留判器；同一块判两次、写错卡、写错流；多机撞键、漏一台；两条算键登记）没再攻。

这条腿没跑任何探针：建冻结副本的 `minimal-copy.py` 被权限闸拒了（「Permission for this action was denied by the Claude Code auto mode classifier」），探针与复跑都写成命令交主 agent。下面凡是探针结局都是「推的，待主 agent 跑」。

## 复跑命令与模型文件

模型目录 `research/prompts/gpu-oom-card-governance-r2-opus-model/`：

| 文件 | sha256 |
|---|---|
| `probe/service_loop_probe.rs` | 3fcb68d1bdabc78252092604a03a66b5527757fec80038bb0949e49813b893d3 |
| `probe/card_lease_probe.rs` | acb9ace5eb1dc1cf919d1d082bc9f0fbcf3b37050fb521a85264a76e7f099ccb |
| `tools/rerun.sh` | 53febca6ed05e0b925e785c08ae3dbae21af4049316e3e089092098f5ef4b27a |
| `tools/rerun-from-a-clean-work-directory.sh` | d097c5f4752fda3ff4821f332c327631fde5a5940ccac3d41afab10d87f822a4 |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`（另在它前面加了一个判一步结局的 `judge_step`）：把冻结副本拷进运行目录，把两段探针分别接在运行目录那棵树的 `gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs` 与 `card_lease_on_a_temporary_library.rs` 末尾（用那两份里现成的替身判器、`LeaseCase` 与临时判定库），再点名测试跑四步 `cargo test`，每步打一行 `name=rerun_step … predicted=… seen=… matched=…`，末行 `name=rerun_steps_mismatched count=<n>`，n 不是 0 退 1。外层 `rerun-from-a-clean-work-directory.sh` 只清空运行目录里的东西（目录本身留着）再调 `rerun.sh`，文件头带会编声明。整条命令在文末「要主 agent 跑的命令」。

## 各格判定一览

「推不出」照正文算打中。四句（分不分辨臂、系统当时看不看得到、满足判据哪一分句、改法还中不中）写在各节末尾。这里的「臂」指这一轮落成的实现与改动之前（OOM 一律整卡停用、没有借卡排队与白名单）。

| 问 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 5 那一次提交自己 OOM、丢设备 | 推不出（打中） | 两种都落进「没等成」那一支：报卡坏、`device_lost=no`，不走重开设备，好卡停到服务重起 | 文件:行 |
| 5 那一次提交等不回来 | 推不出（打中） | 等的是 `timeout: None`，工作线程卡死、不报；手上的块一直「在手」，服务也就一直不借卡 | 文件:行；推的 |
| 5 重开之后旧设备 | 推不出（打中） | 丢了的设备持到服务停，它待写列表里的暂存与表一直占着显存、不在账上；新设备照满额度排，再丢、再重开，第 5 回报卡坏、叫人重置卡 | 文件:行、wgpu 源码；推的 |
| 5 连着 OOM 打转 | 推不出（打中） | 设备没丢的那一种没有上限、也不告警，只打「接住」那一行；control check 与任务服务都不读它 | 文件:行 |
| 5 在判的批、借出去的卡 | 部分打中 | 在判的批照「接住」撤回，不成立；借出期间服务的设备与驻留判器都不放，卡键续不上那一支里借方与手上在判的块同时用卡 | 文件:行；推的 |
| 6 两台来回弹 | 成立（打中） | 同一个进程里不弹；拉黑只在内存里，每重起一次（每次 `control install` 两台都重起）两台各再试一回、各打一行、各发一条告警，告警按块不按流 | 文件:行 |
| 6 放认领与写结论之间判两遍 | 不成立 | 报拒先撤块、之后的结论不收，放掉认领时这一块已不在手上 | 文件:行 |
| 7 两卡各判一截合不回去、同一截判两次 | 不成立 | 结论照序号沿撤回链记回，链在记之前算好 | 文件:行 |
| 7 撤回的块「只是不派回它显存不够过的卡」 | 推不出（打中） | 撤回的块头一截被别的卡拿走之后，它就不在「在等的」里，剩下几截照常派回原来那张卡 | 探针（推的，待主 agent 跑） |
| 7 大卡坏了用哪一刻的账、新流照不照判 | 不成立 | 用的是开卡那一刻的配置额度；之后登记的流与入池的块走同一个判法 | 文件:行 |
| 8 不在表里的进程借到卡 | 成立（打中） | 同名拷贝放哪个目录都放行、借方变量写别人的进程、借到之后 exec 成别的程序、直接认领请求键不经 lease take，四条都过；现有测试自己就是拿改名的 sleep 借到的 | 文件:行 |
| 8 return、renew 不认白名单 | 成立（打中） | 谁拿到借方的进程号与启动刻（`lease list` 就打）都能 `lease return` 放掉别人的两把键；`lease renew` 二进制照收 | 文件:行 |
| 9 后到先拿 | 推不出（打中） | 排序段用的是墙上钟，回拨之后后到的排前；同一毫秒按进程号不按先后 | 文件:行；推的 |
| 9 借方进程停住 | 推不出（打中） | 停着的进程算在跑，续租进程一直续，卡借出去没有上限 | 探针（推的，待主 agent 跑） |
| 9 lease take 停住、被 SIGKILL | 不成立 | 都至多挡一个认领租期（编进来的 120000 毫秒） | 文件:行 |
| 9 等号让服务不放卡、不收回 | 不成立 | 服务只看请求键；只有排第一的才持请求键，等号本身不动服务 | 文件:行 |
| 10 三种 held_by | 部分打中 | 照持有者前缀分：不带 `card-lease/` 的一律写 service，直接认领的借方被写成服务；服务自己 1 毫秒的探请求键也可能被列成 REQUESTED | 探针（推的，待主 agent 跑） |
| 10 租期刚过、服务重起、截断 | 不成立 | 过期的不列；起来时放上一个进程的卡键；列认领按帧续到不截断，续不下去报错 | 文件:行 |
| 11 旧二进制被盖掉 | 不成立 | 清场过了才拷二进制 | 文件:行 |
| 11 清场时被杀 | 成立（打中） | control 不接信号，两台停在 inactive，`check` 两台都退 78；任务服务不把「没在跑」当毛病，没人被告知 | 文件:行 |
| 11 装那一步第二台失败 | 推不出（打中） | 清场过了之后转去第二台的 install 失败（ssh 断、回读超时），没有起回来那一支，第二台停着 | 文件:行 |
| 11 `--dry-run` 停服务 | 不成立 | `--dry-run` 不进停、清场那一段，自证格核了一条 systemctl 都不起 | 文件:行 |
| 12 写错值哪一步红 | 部分打中 | 配置判法不红；红在清场那一刻的私有清场库（地址不在 HOSTS、单元后缀），对 install 来说是两台都停了之后 | 文件:行 |
| 12 键名改一个字 | 成立 | 判法的键表改一个字，自证格 quiesce-script-keys-accepted 红；私有脚本那一侧改名判法看不见 | 文件:行 |
| 13 新开关重叠 | 部分打中 | 等号次序那一条测试在 card-lease-keys-held-by-another-taken-as-claimed 下也会红；反过来不会 | 探针（推的，待主 agent 跑） |

## 第 5 问：OOM 之后先做的那一次提交

### 打中：那一次提交自己 OOM、自己丢设备，都报成卡坏、不重开

那一次提交走的是 `took_the_panic` 里设备没丢那一支：「if let Err(unsettled) = judges.settle_after_running_out_of_memory() {」（`gpu-verification-server/src/card_worker.rs:270`），没等成就报卡坏，`device_lost` 写死是 false（`gpu-verification-server/src/card_worker.rs:271` 到 `:277`），线程退出。文件头写着这一支：「那一次提交没等成的报这张卡坏了；」（`gpu-verification-server/src/card_worker.rs:10`）。

真卡那一份怎么做那一次提交：「let checksums = catch_unwind(AssertUnwindSafe(|| card.crc32c_of_ranges(&[nothing])))」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:456`）。`crc32c_of_ranges` 给一段空字节也照常新建四个 `create_buffer_init` 缓冲（`crates/singlefs-verification/src/gpu_unit_checks/work_on_a_card_and_its_readback.rs:157`、`:165` 起），一个 32 KiB 的槽，各带暂存。历史（推的）：

1. 卡 0 一波装表时 OOM，设备没丢，`after_a_panic` 答 `RanOutOfMemory`，判器全丢。
2. 那一次提交先建四个带暂存的缓冲。卡上这时还压着上一波没提交的暂存（判决第 1 条），或者别的进程（借方、别的会话的活）此刻占着显存：建暂存这一步再 OOM。按第一轮判决引的 wgpu-core 30.0.1，暂存走致命那条路，OOM 就丢设备；或等的时候设备丢了，`take_the_readback_after_the_wait_or_record_the_device_lost`（同一份文件第 34 行）记下丢失再 panic。
3. 两种都被 `catch_unwind` 接成 `Err`，于是报卡坏、`device_lost=no`。设备其实是因显存不够丢的（验证包认得出），可这一支不再问 `after_a_panic`，也就不走「重开设备」；设备没丢而只是 OOM 的，同样报卡坏。
4. 这张卡停到服务重起，`control check` 照卡坏那一套退 1、出路是请用户重置那张卡。用户定案「OOM 需要倍catch 不能影响好的卡」：这里一张好卡因为一次 OOM 停了。

能照喂的输入：替身那一份 `settle_after_running_out_of_memory` 恒交 `Ok`（`gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs:261` 起），这一支没有测试。给替身加一种设定：OOM 一回之后 `settle` 交 `Err("替身照设定：那一次提交自己 OOM")`，核 `summary.cards_failed == [CardIndex(0)]`、卡坏那一件 `device_lost == false`、在判的块报拒 `CardFailed`（不是判不了）。按代码这一条会绿，绿说明的就是上面这件事。本腿没写成探针（时间给了第 7、9、10、13 问）。

### 打中：那一次提交等不回来

等的是「timeout: None,」（`crates/singlefs-verification/src/gpu_unit_checks/work_on_a_card_and_its_readback.rs:243`），没有上限。卡上上一波的活卡住（驱动不回、卡挂起而设备没被判成丢）时，工作线程停在这里，既不报接住、也不报卡坏、也不报空出来了（报它的 `RoundFinished` 在 `gpu-verification-server/src/card_worker.rs:245`，走不到）。主循环那一侧这张卡一直不空、它这一波撤回的块一直挂在 `pieces_in_flight_by_card` 上。取活循环那一侧这些块一直在手上：「|| self.pool.claimed_count() != 0」（`gpu-verification-server/src/serving_from_the_library.rs:796`），于是有人要借时一直等「手上的都写回」、一直不放卡键（同一份文件第 828 行那一支），借方的 `lease take` 也一直等。认领照常续，别的台取不到这几块。谁都不报。

这一格不分辨臂：判的时候那几次等同样没有上限，正常判也会这样卡住；那一次提交只是多了一处同样的等。

### 打中：重开设备之后旧设备上还压着显存，账上看不见

文件头：「丢了的那一个设备与它之前开的照旧持着，」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:17`），到服务停才放。wgpu-core 30.0.1 的 `lose` 只标设备失效、调回调，注释写着要等以后 poll 那一台设备才清：「// Future calls to poll_devices will continue to check the work queues」（`wgpu-core-30.0.1/src/device/resource.rs:5432`）；真正放显存的 `release_gpu_resources` 只在 poll 走到「if !device_valid && queue_empty {」（`wgpu-core-30.0.1/src/device/resource.rs:980`）那一支才调。服务丢了旧设备之后再不 poll 它，旧设备队列待写列表里那一波新表与暂存（判决第 1 条说的那两份）就一直占着。重开的设备照配置的满额度排（`gpu_judges_on_a_card.rs:317` 照 `quota` 另开），账上不减。

历史（推的）：一波装表时暂存 OOM、设备丢了 → 重开，旧设备压着这一波的表与暂存 → 下一波照满额度排，实际可用的少了这一份，再 OOM、再丢 → 重开 ……「pub const DEVICE_REOPENINGS_OF_A_CARD: usize = 4;」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:121`）用完，第 5 回答「这张卡在这个进程里已重开过 {} 次设备（DEVICE_REOPENINGS_OF_A_CARD），不再重开」（同一份文件第 310 行），报卡坏、`device_lost=yes`，control check 叫人重置一张好卡。每一回旧设备压的那一份都留着，越往后越容易 OOM。

### 打中：设备没丢的那一种连着 OOM，没有上限、不告诉谁

设备没丢的 OOM 没有计数上限：每回清驻留、撤回、单独重判；单独重判成了就接着排下一波。control 文件头：「显存不够（设备没丢，或因显存不够丢了、已重开）的只打 GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT（接住了、卡照常用），不算卡坏。」（`gpu-verification-server/control:101`）。任务服务的服务健康只收卡坏与判不了两种：「problems.extend(self.gpu_card_failures())」（`task-server/task_server_scheduler_notifying.py:713`），种类前缀见 `task-server/task_server_constants.py:593`。可造的情形（推的）：额度是开卡那一刻读的，之后卡上有了别的占用（借卡续不上那一支里借方已在用、上一节旧设备压着的那一份），每一波照满额度排都 OOM、每块单独重判又都成。服务每波打一行接住，卡一直在「整卡换出、重传表、单块一波」里转，吞吐掉到一块一波，没有一行会让 `control check` 退 1，任务服务也不告警。

### 部分打中：在判的批与借出去的卡

- 在判的批：重开之后与设备没丢那一种同一条路（都报 `OutOfMemoryCaught`），撤回、单独重判，不成立。
- 借出去的卡：借出期间「4. 借出期间设备不放」（`gpu-verification-server/src/serving_from_the_library.rs:53`），工作线程缓存里驻留的判器、它们的表与草稿区也都在卡上（借出不换出），加上上一节那几个旧设备压着的。借方拿到的卡比卡串写的额度少这几份，文件头把「够不够」留给借方头一次跑去量，没写这几样。
- 续不上那一支：「续不上（停得太久、租期过了被借的一方认领走）当借出了」（`gpu-verification-server/src/serving_from_the_library.rs:56`），之后手上已在判的照判完。这一段借方与服务同时用卡。服务这边再 OOM 的块到「腾空的卡上又 OOM」就当判不了，放给另一台、发告警，其实是借方占着显存。要服务取活循环停顿过一个认领租期（120000 毫秒）才走得到，可能性低（推的）。

四句：5 的四格都不分辨臂（改动之前 OOM 一律整卡停用，这几格那时就是卡坏）；系统看得到要分的东西，那一次提交没等成时设备情况就在 `device_condition()` 里，能照 `device_after_a_panic` 再分一次；满足的是第 5 问「那一次提交自己 OOM、自己丢设备、等不回来时各走哪一支」「连着 OOM … 而不报任何人」两个分句；第一轮改法第 1、2 条碰不到这几格（第 1 条就是这一次提交本身，第 2 条只管判的时候丢的设备），见末尾 G1 到 G3。

## 第 6 问：放认领给另一台

### 打中：拉黑只在内存里，每重起一回两台各再试一回、各发一条告警

同一个进程里不弹：判不了的块「记下名字，这个进程此后不再认领它」（`gpu-verification-server/src/serving_from_the_library.rs:25`），记在 `refused_for_running_out_of_memory` 这类内存里的集合（同一份文件第 1230 行）。两台各判不了一次之后都不再取，块留在库里（之后谁知道归辩方第 3 问）。

弹的是跨进程的：集合不落盘。历史（推的）：

1. 块 X（某条流，卡腾空了也装不下它的实际峰值）在第一台判不了：放认领、打一行判不了；第二台取到、同样判不了、放认领、打一行。任务服务两条告警，种类是「kind = f"{GPU_BLOCK_NOT_JUDGEABLE_ALERT_KIND}:{host}:{process}:{':'.join(named)}"」（`task-server/task_server_lib_configuration.py:73`），带台与进程号。
2. 有人跑一次 `control install` 换版：两台都停、清场、再起（第 11 问那一节），两个新进程的集合都是空的。
3. 两台又各取 X 一回：每回一波 OOM、撤回、单独重判再 OOM（第 5 问那一节，每回两次 OOM、整卡换出），各打一行判不了、各放认领。进程号换了，告警种类是新的，再发两条；旧进程那两条因为旧进程的行不再读（control 只读此刻主进程的行），在任务服务那边算「好了」。
4. 每重起一次重复一遍。告警的键只到块（「GPU_BLOCK_NOT_JUDGEABLE_KEY_FIELDS = ("family", "stream", "block")」，`task-server/task_server_lib_constants.py:495`），同一条流 N 块都判不了，一圈就是两台各 N 行、2N 条告警、给每个有任务的会话各发 2N 条消息。

能照喂的输入：喂任务服务的假 cards-failed，头一回两行同块不同台（`host=this-host process=100`、`host=<第二台> process=200`），下一回同一块换成 `process=101`、`process=201`，核会话收到 4 条（推的；本腿没写喂任务服务的探针，载入任务服务的库会被项目代码闸拒）。

### 不成立：放认领与写结论之间同一块被两台各判一次

主循环报拒时先把这一块连同撤回的几块从池里撤掉、记进 `withdrawn`（「self.withdrawn.insert(taken_off);」，`gpu-verification-server/src/service_loop.rs:1369`），之后交回的结论在「if self.withdrawn.contains(&piece.block) {」（`gpu-verification-server/src/service_loop.rs:1216`）丢掉；取活循环收到报拒才放认领，那时这一块已不在 `in_flight` 里，不再续、不再写回。另一台取到之后只有它写。第一台的认领租期过了、第二台取走、第一台又写回那一种是认领协议本来的问题，不是放认领这一步带进来的，没攻。

四句（打中那一格）：不分辨臂（改动之前判不了的块整卡停用，也会在重起后再试）；系统看得到，判不了的块名字就是待判行的名字，能写进库里（下面 G4）；满足的是第 6 问「每一圈各打一行、任务服务每圈发一条告警」，「圈」是一次重起不是一次放认领；甲案（放认领、另一台去试、打行、告警）碰不到这一格。

## 第 7 问：撤回的块别的卡也能拿

### 不成立：两张卡各判一截合不回去、同一截判两次

撤回的那一截另起一块，结论照序号沿撤回链一路记回收下的那一块；链在记之前算好（「// 迭代上界：撤回的块的号都比它从的那一块大」，`gpu-verification-server/src/service_loop.rs:1221`），中间某一节记满被删掉也不影响往下记。卡 0 腾空的时候撤回的块要么已派出（不在池里没派的那一截了），要么还在等；派出一截就从「在等的」里删掉（`gpu-verification-server/src/service_loop.rs:942`），`place_the_lone_blocks` 找不到它、不会再单独派一遍。探针里同时核「每个态只判一次」。

### 打中：撤回的块被别的卡拿走一截之后，剩下的照常派回原来那张卡

文件头：「撤回的块照常交调度，只是不派回它显存不够过的卡；」（`gpu-verification-server/src/service_loop.rs:17`）。可「不派回」只靠调度之前那一道筛：「!self.waiting_for_an_emptied_card.contains_key(&block.block)」（`gpu-verification-server/src/service_loop.rs:608`），只有还在「在等的」里的块才看它显存不够过哪几张卡。一块一次只派一批（「长流按批切开、不一口气派完」，`gpu-verification-server/src/batch_scheduling.rs:4`），撤回的块被卡 1 拿走头一批，第 942 行就把它从「在等的」里删了，剩下那几截此后对每张卡都照常排，包括卡 0。

探针 `probe_a_put_back_piece_split_onto_the_smaller_card_returns_to_the_card_it_ran_out_of_memory_on`（推的，待主 agent 跑）：卡 0 一次派活 64 态、卡 1 一次派活 4 态，一条流一块 256 态，卡 0 头一回判它时显存不够一回。核：每态只判一次；卡 1 判过撤回那一截里的态（取样前提）；卡 0 在接住之后又判了撤回那一截里的态（本腿推的结局）。读数行 `OOM_ATTACK_PROBE name=put_back_piece_back_on_its_card …`。替身只出一次毛病，所以这里卡 0 再判就成了；真卡上卡 0 若还是那么紧，这几截就在普通的一波里再 OOM 一回、再整卡换出。

四句：分辨臂（改动之前撤回的块只等卡 0 单独判）；系统看得到，`cards_the_block_ran_out_of_memory_on` 里这一块还记着卡 0，只是筛的时候没看；满足的是第 7 问「卡 1 拿去判的同时卡 0 腾空了，会不会……」那一句的边：不是判两次，而是派回了不该派回的卡；第一轮改法第 6 条碰的就是这里，它的实现留了这个口子，G5 修。

### 不成立：大卡坏了用哪一刻的账、坏卡之后登记进来的流

`refusal_for_a_stream_no_working_card_fits` 比的是 `mebibytes_for_the_judges`（`gpu-verification-server/src/card_memory_ledger.rs:49`），取的是开卡那一刻的配置额度减驱动那一份，不是此刻驻留。卡坏之后登记的流在登记时就走这一道：「if let Some(refusal) = self.refusal_for_a_stream_no_working_card_fits(stream, &facts) {」（`gpu-verification-server/src/service_loop.rs:1147`），之后入池的块在 `admit_the_block` 里再走一次（同一份文件第 1178 行）。撤回的块也是池里的块，坏卡那一刻一起筛。没找到漏的。

## 第 8 问：借卡白名单

用户定案（`records/2026-11-18-GPU服务OOM后整卡停用.md` 第 82 行转来的原话）「E202 真卡验证才能借卡 其他的拒绝。 白名单借卡」。下面四条都让不是 E202 装置的进程借到卡。

### 打中：不在表里的进程借到卡

1. 同名拷贝放在别的目录。「认借的一方只看 `/proc/<借的进程号>/exe` 指到的那个文件的文件名」（`gpu-verification-server/src/card_lease.rs:8`），读法是「let executable = std::fs::read_link(&executable_link_path)」（`gpu-verification-server/src/card_lease.rs:473`）取文件名。这一条不用造：借卡那个测试目标自己就是这样借到的，「/// 把 `sleep` 拷进 `directory`、改名成 `executable_name` 起」（`gpu-verification-server/tests/card_lease_on_a_temporary_library.rs:1049`），`a_borrower_whose_executable_name_is_on_the_whitelist_takes_the_cards`（同一份文件第 1591 行）拿改了名的 `sleep` 借到了卡串，正文记它全绿。硬链接同样（`/proc/<进程号>/exe` 给的是起它时用的那个名字）；符号链接不行（解到目标的名字）。
2. 借方变量写别人的进程号与启动刻。认的是变量点名的那个进程，不是起 `lease take` 的那一个：「子命令不看自己的父进程」（`gpu-verification-server/src/card_lease.rs:51`）。同一个测试里起 `lease take` 的是 cargo 测试二进制本身（文件头说它的名字带哈希后缀、本该被拒），变量点名的是旁边那个改名的 `sleep`，借到的卡串交给了测试二进制。照喂的命令（推的，只对临时判定库服务）：`cp /bin/sleep <目录>/e202_gpu_cpu_gc_comparison`，`setsid <目录>/e202_gpu_cpu_gc_comparison 3600 &`，取它的进程号与 `/proc/<号>/stat` 第 22 项，再以 `GPU_CARD_LEASE_BORROWER=<号>:<启动刻>` 从任何程序里起 `gpu-verification-server lease take`。
3. 过了白名单再 exec。名字只在 `lease take` 读的那一刻认，exec 不换进程号与启动刻，续租进程照「在跑」续。一个改名成 `e202_gpu_cpu_gc_comparison` 的 bash 起 `lease take`、借到之后 `exec` 成任何程序，卡一直在它手里。
4. 不经 `lease take`。白名单只在 `lease take` 这一个客户端里认（「admit_only_a_borrower_on_the_whitelist(&borrower, &break_switches)?;」，`gpu-verification-server/src/card_lease.rs:1602`，外面套着 `BorrowerStep::Take`）；判定库服务那一侧的认领帧谁都能发、不认身份。服务只看请求键有没有人持着（`gpu-verification-server/src/serving_from_the_library.rs:731`）。取活循环那一条测试就是这样借的：借的一方用 `BORROWER_IDENTITY_IN_THE_SERVING_LOOP` 直接认领请求键（`gpu-verification-server/tests/card_lease_on_a_temporary_library.rs:473`），服务照样放卡键、打借出那一行。

`(deleted)` 后缀那一种反过来：白名单里的二进制跑着的时候被重编（cargo 换掉那个文件），`read_link` 读到的是「<路径> (deleted)」，文件名带后缀、对不上，E202 自己被拒、退 4。是误拒，不是放行（推的，没跑）。

### 打中：`lease return`、`lease renew` 被别人用

- 持有者身份只由这一台与借方的进程号、启动刻拼成（`gpu-verification-server/src/card_lease.rs:318`），`lease list` 把它原样打在 `holder=` 里。任何进程拿到这两个数，以 `GPU_CARD_LEASE_BORROWER=<借方的号>:<启动刻>` 起 `lease return ''`，就以「另一个身份」认领停止键、等续租进程放手，再以借方的身份放两把键（`gpu-verification-server/src/card_lease.rs:65` 那一段）。E202 跑到一半卡就回到服务手里，服务从下一圈起认领、往同一张卡上派活（借出期间设备一直开着），两边同时用卡。现有测试里还卡都是测试进程替借方还的（`TakenLease::return_now`），说明就是这样能还。
- `lease renew`：control 拒（`gpu-verification-server/control` 的 `lease_arguments_problem`），二进制照收：「[subcommand] if subcommand == RENEW_SUBCOMMAND => {」（`gpu-verification-server/src/card_lease.rs:1553`），直接起装着的二进制就绕过 control。单独起它占不住卡（第一回续请求键与卡键时卡键在服务手里，续不上就放掉、退出），作用只是让请求键被持一瞬、服务探到就开始收手；配上同一个借方身份它能替一个已借到的借方多续一份，寿命同那个借方。危害小于上一条。

四句：分辨臂（改动之前没有白名单，谁都能借，这一格是新换上的东西自己挡不住）；系统看不到能分辨的东西：白名单要认的是「谁在用卡」，`lease take` 手里只有一个它没法核实与自己有关的进程号，判定库服务手里只有一串持有者身份；满足的是第 8 问「造得出一个不在表里的进程借到卡吗」与「能不能被用来占住或放掉别人的键」；开关 card-lease-borrower-whitelist-unchecked 那条测试只核表外的名字被拒，碰不到这四条。白名单今天能做的是挡误用（产品包测试照旧调 `lease take` 会被拒），挡不住有意绕开；要挡住得在判定库服务那一侧认身份，见 G6。

## 第 9 问：借卡排队

### 打中：后到的先拿到（时钟回拨）

排序段的头一项是「let since_the_epoch = SystemTime::now()」（`gpu-verification-server/src/card_lease.rs:356`）取的墙上钟毫秒数。文件头的前提是「只在同一台里比：同一台的 `lease take` 都在这一台起、读同一个钟。」（`gpu-verification-server/src/card_lease.rs:26`）。同一个钟不等于钟只往前走：A 在墙上钟 T 起等，之后 NTP 把钟往回拨了 d（或人手改了钟），B 在墙上钟 T−d+ε 起等，B 的键比 A 的小，B 排第一、先认领请求键、先借到。同一毫秒的两张按进程号、启动刻排，次序与谁先到无关（只差一毫秒，代价小）。照喂（推的，本机没有 faketime，本腿没写成探针）：两条 `lease take` 之间把系统钟拨回一秒；或者在临时判定库上以 B 的身份直接认领一张时刻写早的等号键，`CardLeaseWaitingTicket { started_waiting_milliseconds_since_the_epoch: <A 的时刻 − 1000>, borrower: B }.key(&keys)`，看 A 的等待行变成 `position=2`。后一种也说明等号键谁都能认领：一个不经 `lease take` 的客户端认领一张时刻是 0 的等号并一直续，这一台的 `lease take` 全都排在它后面、一直等（第 8 问第 4 条同一个口子）。

### 打中：借方进程停住，卡借出去没有上限

续租进程每拍先看借方在不在：「&& !self.borrower_is_running()」（`gpu-verification-server/src/card_lease.rs:1392`），判法只认不在、僵尸、启动刻不同（「const ENDED_PROCESS_STATES: [char; 2] = ['Z', 'X'];」，`gpu-verification-server/src/service_process.rs:37`），别的都是「Some(_) => ProcessLiveness::Running,」（同一份文件第 134 行）。借方被 SIGSTOP 停住、或者自己死锁卡住，状态是 T 或 S，算在跑，续租进程一直续两把键；服务一直「借出」、不认领，后面排着的 `lease take` 一直等，没有上限、没有告警。

探针 `probe_a_stopped_borrowing_process_keeps_the_cards_past_a_whole_claim_lease`（推的，待主 agent 跑）：借到之后对借方（改了名的 `sleep`，它自己的进程组）发 SIGSTOP，等编进来的认领租期再加 10 秒，核卡键还在借方名下，再 SIGCONT、还卡。读数行 `CARD_LEASE_ATTACK_PROBE name=stopped_borrower_past_a_lease …`。这一条跑两分多钟。

### 不成立：lease take 停住、被 SIGKILL 留下的等号

- 排第一的 `lease take` 被 SIGSTOP、还没认领请求键：它的等号没人续，至多一个认领租期（「CLAIM_LEASE_MILLISECONDS=120000」，`gpu-verification-server/service-configuration.txt:12`）后失效，后面的排到第一。已认领请求键、在等卡键时停住：请求键同样一个租期后失效，服务下一回探到没人要借就收回卡键。上限都是 120 秒。SIGCONT 之后它续等号与请求键时若已被后面的人拿走，就续不上、放掉、退 1，不会插队。
- SIGKILL 留下的等号：「排在前面的借方不在了（它的 `lease take` 被杀、没人续），它的等号到租期失效、自然让出。」（`gpu-verification-server/src/card_lease.rs:29`），同样至多 120 秒；测试目标里有一条专门核这件事。
- 等号在列与认领请求键之间失效：列完之后它照样去认领请求键，这时它确实排第一过；之后续等号那一下若续不上就放掉、退 1。没造出插队。

### 不成立：等号让服务一直不放卡、一直不收回

服务只探请求键：「CardLeaseKeyAnswer::Claimed => LeaseRequest::NotRequested,」（`gpu-verification-server/src/serving_from_the_library.rs:730`）。等号排着而排第一的那个还没认领请求键时，服务照常判；只有排第一的才去持请求键。两个借方之间（前一个还了、后一个两秒内才再试）服务可能先探到「没人要借」、收回卡键、认领新块，后一个要等这几块写回，这是「服务那一半只看请求键」本来的样子，有上限（那几块判完）。一直不放卡的只有第 5 问那一种（手上的块卡死）。

四句（两格打中）：分辨臂（改动之前没有排队，谁先撞上请求键谁先借；借方停住没有上限在改动之前也一样，不分辨）；系统看得到：排序可以用判定库服务那一侧单调的东西（它收帧的序号）而不用借方自己的钟，借方停住看得到 `/proc/<号>/stat` 的状态 T；满足第 9 问「时钟回拨」「后面的人等多久、有没有上限」两个分句；开关 card-lease-waiting-order-ignored 的那条测试只核不看次序会插队，碰不到这两格，见 G7、G8。

## 第 10 问：`lease list`

### 部分打中：直接认领的借方被说成服务持着

`held_by` 只照持有者身份的前缀分：「Some(claim) if claim.holder_identity.starts_with(HOLDER_PREFIX.as_bytes()) => {」（`gpu-verification-server/src/card_lease.rs:628`），前缀是 `card-lease/`，别的一律是 service。经 `lease take` 借的都带这个前缀，判得对；第 8 问第 4 条那种直接认领的借方不带。测试目标里借方的身份 `card-lease-test-borrower`（`gpu-verification-server/tests/card_lease_on_a_temporary_library.rs:108`）是 `card-lease-` 起头、不是 `card-lease/`，它持着卡键时 `lease list` 写 `held_by=service`。探针 `probe_lease_list_calls_a_borrower_whose_identity_lacks_the_card_lease_prefix_the_service`（推的，待主 agent 跑）就核这一行。

另两处小的（推的，没写探针）：服务每个扫库间隔以自己的身份、1 毫秒租期探一次请求键，`lease list` 恰好落在那 1 毫秒里，就打一行 `GPU_CARD_LEASE_REQUESTED holder=<服务的身份>`，读的人会当成有人要借；`lease return` 以 `…/return` 身份、1 毫秒租期试认领卡键那一下，同样会被列成 `held_by=borrower`。

### 不成立：租期刚过、服务重起换了身份、请求键有人持着而卡键在服务手里、截断

- 租期刚过：判定库服务只列「let is_live = record.lease_expiry_milliseconds > now || includes_expired;」（`prune-server/src/committer.rs:1062`）里活着的，过了就是 `nobody`。
- 服务重起：「起来时放上一个进程的、停的时候放自己的，卡键跟三族的认领键一起放。」（`gpu-verification-server/src/library_pending_pool.rs:24`），新进程起来时旧进程的卡键已放，只有状态目录那一行判不了旧进程活没活时才留到租期过（那一段新进程从「借出」起步，`lease list` 写 service、持有者是旧进程，至多 120 秒）。
- 请求键有人持着、卡键在服务手里：打 `held_by=service` 一行加 `GPU_CARD_LEASE_REQUESTED` 一行，正是「服务在收手、借方在等」，对。
- 截断：`list_claims` 按帧接着列到不截断，服务说截断却一条没回就报错（「"列认领说前缀下还有认领，却一条都没回：续不下去".to_string(),」，`prune-server/src/reliable_client/claims.rs:248`），中途失败一行都不打。没造出少行不报。

四句（打中那一格）：不分辨臂（改动之前没有 `lease list`）；系统看得到持有者身份的全文，能认 `card-lease/<台>/<号>-<启动刻>` 的整个形状、认不出的写 other；满足第 10 问「会不会把借方持着说成服务持着」；开关 lease-list-refused 碰不到，见 G9。

## 第 11 问：`control install` 先停、再清场、再起

### 不成立：起回来时旧二进制已被盖掉

二进制在清场过了之后才拷：「  装：清场过了，这一台装、起（install 那一段），第二台经 ssh 起同一条 install。」（`gpu-verification-server/control:83`），`stop_and_quiesce_before_install` 里清场没过就 `start_again`（`gpu-verification-server/control:716` 那一支），那时 `install_here` 还没跑。文件头「装着的还是换版之前的二进制与单元（清场没过时一份都没换）。」（`gpu-verification-server/control:85`）与代码一致。

### 打中：发起台停了两台、自己在清场时被杀

control 没装信号处理（`grep -c "import signal\|signal.signal" gpu-verification-server/control` 是 0）。清场至多等「QUIESCE_SECONDS = 240」（`gpu-verification-server/control:277`）再加 15 秒宽限。历史（推的）：

1. `control install --binary …` 停了这一台、经 ssh 停了第二台（两台都是 `systemctl --user stop`，单元的 `Restart=always` 不拉起被 stop 停掉的）。
2. 清场那几分钟里 control 收到 SIGTERM、SIGHUP（起它的终端或会话断了，主 agent 停了那条后台命令）或 SIGINT：Python 默认处理，进程当场没了，`start_again` 不跑；清场脚本是它的子进程，收不收得到信号看怎么停的。
3. 两台停在 inactive。`control check` 两台都判服务没在跑、退 78、出路是起它。任务服务读 cards-failed 得到两台 `state=not-running`，照文件头「not-running 不算没读成：服务重起之后是新进程、新的键」（`gpu-verification-server/control:45`）不留上一回、也不当毛病；服务健康里没有「GPU 服务没在跑」这一种（`task-server/task_server_scheduler_notifying.py:713` 只加卡坏与判不了）。判段不再问服务收不收，要 GPU 的那几片写了待判行就等，没有人被告知，直到有人手跑 `control start` 或任务服务把片子当卡住打回。

能照喂的输入：自证根里把假清场换成一个先睡 30 秒再退 0 的脚本，起 `control install`，10 秒后对 control 的进程号发 SIGTERM，看 `fake/systemctl-calls.log` 里只有 stop、没有 start（推的，本腿没写成自证格；自证的假清场靠 `fake/quiesce-exit` 定退出码，要加一个「睡多久」的设定）。

### 打中：清场过了之后，第二台的 install 没成，第二台就一直停着

清场过了，`run_on_the_hosts` 逐台做：「code = forward(paths, target, forwarded_arguments)」（`gpu-verification-server/control:1319`），退出码只合进总码，没有起回来那一支。第二台这一步失败（ssh 断了退 255、那台读不到 `--binary`、回读「在跑」30 秒没对上）时，第二台在第 1 步已被停掉，就一直停着，总码退 78 或 1。接着同上一格：任务服务不告警。这一台换了新版、第二台还是旧的二进制、停着。

### 不成立：`install --dry-run` 真的停服务

「if subcommand == "install" and "--dry-run" not in forwarded_arguments:」（`gpu-verification-server/control:1302`），带 `--dry-run` 不进停与清场那一段。自证格 install-dry-run-writes-nothing 核了「expect(problems, not world.systemctl_calls(), f"install --dry-run 不该起 systemctl：{world.systemctl_calls()}")」（`gpu-verification-server/control:1852`），连 `is-active` 都没起，接错了会红。

四句（两格打中）：分辨臂（改动之前 install 不先停，停与清场之间被杀那一格不存在）；系统看得到：control 停了哪几台它自己记着（`stopped`），信号来时可以照同一份表起回来；任务服务看得到 `state=not-running` 而 `ENABLE_GPU=1`、这一台点名了卡；满足的是第 11 问「发起台停了第二台而自己在清场时被杀，两台各停在什么状态、control check 怎么说」；弄坏开关 install-stopped-units-left-down 那一格只核清场没过那一支，碰不到被杀与装那一步失败，见 G10。

## 第 12 问：九个清场键只认键名

### 部分打中：写错值哪一步才红

配置判法照文件头「这里只认键名、不判值」（`scripts/crash/crash-point-replay-shard-configuration-check.sh:79`），值写错照样判过。值由仓根被 git 忽略的那几份私有清场脚本读（内容是私有的，这里不引；本腿只数过它们读的键名与判法里的九个逐字相同，`QUIESCE_WAIT_FOR_IN_FLIGHT` 是它们读的环境变量、不是配置键）。私有清场库在清场那一刻逐项判按台的三个键：地址不在 HOSTS 就拒、单元名带通配或后缀不对就拒；单元名写成一个不存在的单元，私有脚本里没有核单元在不在的地方（按 `LoadState`、`not-found` 搜零命中，只搜了这两个词），红不红看停它时 `systemctl` 的退出码怎么被收，本腿没读到那一层，推不出。

所以写错地址、后缀，红在第一次清场：`control install` 时是两台都停了之后（第 11 问第 1 步），清场没过、两台照旧二进制起回来、退 78；每一条要卡的任务的 `gpu-quiesce-run.sh` 同样退 78。配置判法一直是绿的，最早能看出来的地方不是它（哪些地方跑配置判法本腿没查）。

### 成立：键名改一个字，自证格红

判法自己的键表在第 184 行；自证格的配置是字面写的九个键（例「"QUIESCE_GATEWAY_UNIT=selftest-gateway.service"」，`scripts/crash/crash-point-replay-shard-configuration-check.sh:423`），格名「judge quiesce-script-keys-accepted "$work/quiesce-script-keys.env" 3 "双机：关"」（同一份文件第 425 行）。键表里改一个字，字面的那个键就成了不认得的键、这一格判不过、红。反过来私有脚本那一侧改了一个键名，判法与它的自证都看不见（私有脚本不在仓里，没有一格能把两边对起来），用户照判法认的名字写，私有脚本读不到、当没写（推的）。

四句：不分辨臂（这几个键是新认的，改动之前写了就报不认得）；系统看得到：地址在不在 HOSTS 判法手里有 HOSTS，能在判法那一步就判（不碰单元名，私有的照旧归私有脚本）；满足第 12 问「写错值……时哪一步才红、红在哪」；见 G11。

## 第 13 问：弄坏开关

### 部分打中：等号次序那一条测试在另一个开关下也会红

两个开关在排队循环里各管一处：card-lease-waiting-order-ignored 让不排第一的也去认领请求键（「.is_on(BreakSwitch::CardLeaseWaitingOrderIgnored);」，`gpu-verification-server/src/card_lease.rs:1180`）；card-lease-keys-held-by-another-taken-as-claimed 让排第一的撞上别人持着的请求键也当认领到（「.is_on(BreakSwitch::CardLeaseKeysHeldByAnotherTakenAsClaimed) =>」，`gpu-verification-server/src/card_lease.rs:1187`），卡键那一处同样。两处不重叠。各自点名的测试：前者是 `the_borrower_that_started_waiting_first_takes_the_cards_first_and_the_later_one_waits_second_in_the_queue`（`gpu-verification-server/src/break_switches.rs:705`），后者是 `two_borrowers_on_one_host_get_the_cards_one_after_the_other`（同一份文件第 690 行）。

- 后者那一条在前者的开关下（推的）：两条 `lease take` 都去撞请求键，先撞上的借到，后一条撞上别人持着照等，前一条还了才借到，测试绿。它只认自己那一条，对。
- 前者那一条在后者的开关下（推的）：那条测试先让服务持着两把键，先等的那一条排第一、撞上服务持着的请求键，开关下当认领到、卡键同样当认领到，当场借到、放掉等号、退出；测试在「先等的那一张等号排上」那一步等不到两张等号同时排着，30 秒到点红。它在别的开关下也红，于是「这条测试红了」不能单独证明是次序那一处坏了。正文与第一轮判决用的判据就是「点名那一条是否靠别的开关也会红」，这一格照那一句算打中；危害小：两个开关各有一条会红的测试，弄坏哪一处都抓得到。复跑第 3、4 步就是这两格。

### 不成立：别的新开关

- 服务那一侧接住与判不了的几个：card-not-settled-after-running-out-of-memory、device-lost-for-want-of-memory-taken-for-a-card-failure、judge-panic-taken-for-a-card-failure 各守 `took_the_panic` 的一支（`gpu-verification-server/src/card_worker.rs:269`、`:291`、`:300`），互不相碰；block-refused-for-memory-claimed-again、block-not-judgeable-claim-kept、block-not-judgeable-line-left-out 点名同一条测试，弄坏的是取活循环里三处不同的地方（拉黑、放认领、打行）；那一条测试是不是三处各有一句断言，本腿没逐行核（第一轮判过同一种写法）。
- control 的 install 三个（install-quiesces-before-stopping、install-leaves-the-running-unit、「install-stopped-units-left-down    install 停了之后清场没过，停掉的那几台不起回来」，`gpu-verification-server/control:184`）点名同一格 install-stops-then-quiesces-then-starts，`✗` 那一句各不相同。lease 那几个各管转发的一处。
- 没有开关的：那一次提交没等成那一支（第 5 问）没有开关、替身恒交 `Ok`，测试一条都没走到；control 拒 `lease renew` 有开关 lease-renew-passed-through，可二进制自己照收 `lease renew`，这一格的开关守的是 control 那一层，挡不住直接起二进制（第 8 问）。

## 改法与各格（只在我的模型上推过、被攻过零轮，一格都没量）

| 改法 | 修哪一格 | 量没量 |
|---|---|---|
| G1：那一次提交没等成时先再问一次 `after_a_panic`（因显存不够丢了走重开，设备没丢而又记了 OOM 当「腾空的卡上又 OOM」处置这一波的块），只有为别的原因丢了才报卡坏；那一次提交改成真的空提交（不建缓冲），等加上限 | 第 5 问提交自己 OOM、丢设备、等不回来 | 推的 |
| G2：重开之前先丢掉旧设备（放掉 `GpuCard`、让 wgpu 走 Queue 的 drop 清待写列表），或重开之后把旧设备的占用从额度里扣掉；重开次数按一段时间计，不按整个进程计 | 第 5 问旧设备压着显存、第 5 回卡坏 | 推的 |
| G3：设备没丢的 OOM 一段时间里超过若干次，cards-failed 另打一行、任务服务告警；或每回接住之后重读一次卡的空闲显存当额度 | 第 5 问连着 OOM 不告诉谁 | 推的 |
| G4：判不了的块在判定库里记一笔（GPU 认领键前缀下、带台名的长租期认领，或待判行旁的一行），重起之后照它不再取；告警的键去掉进程号、同一条流并成一条 | 第 6 问重起再弹、按块刷屏 | 推的 |
| G5：调度之前那一道筛不看「在不在等」，只要 `cards_the_block_ran_out_of_memory_on` 里有这张卡就不派给它 | 第 7 问派回原来那张卡 | 推的，待主 agent 跑探针 |
| G6：身份在判定库服务那一侧认：请求键、卡键、等号只收以 `card-lease/` 起头、且由带白名单的 `lease take` 起的客户端发的认领（要给判定库服务加认身份的办法）；只改客户端的话，至少核起 `lease take` 的进程就是变量点名的那一个或它的子孙，并在续租时照样核可执行文件名 | 第 8 问四条放行、别人还卡 | 推的；只改客户端时第 8 问第 4 条照中 |
| G7：等号的次序取判定库服务收帧的序号（或它的单调钟），不取借方的墙上钟 | 第 9 问时钟回拨 | 推的；等号谁都能认领那一半照中 |
| G8：借卡设上限（一次借出至多若干分钟，到点续租进程放键、打一行），或借方进程状态是 T 的当不在 | 第 9 问停住没有上限 | 推的 |
| G9：`held_by` 认整个 `card-lease/<台>/<号>-<启动刻>` 的形状，服务的身份照状态目录里「在跑」那一行认，两样都不是写 other | 第 10 问把借方说成服务 | 推的 |
| G10：control 在 install 停了之后接 SIGTERM、SIGHUP、SIGINT，照 `stopped` 起回来；装那一步某一台失败也起回来那几台；任务服务把「开了 GPU、这一台点名了卡、cards-failed 说 not-running」当一种服务健康问题 | 第 11 问清场时被杀、装那一步第二台失败 | 推的 |
| G11：配置判法判按台三个键的地址在不在 HOSTS（只判地址，不判单元名） | 第 12 问红得晚 | 推的 |

各改法碰不到的格：G1 不管判的时候那几次没有上限的等；G2 不管借出期间驻留判器占着的那一份；G6 只改客户端时碰不到直接认领；G7 碰不到直接认领等号；G10 的告警那一半碰不到「服务在跑而卡全借出去了」。

## 撤回本腿第一轮给过的说法

1. 第一轮第 6 问「打中：放回的核对组把卡 1 的驻留判器全部换出」与跟着它的改法 F4：撤回。判决复跑探针得 `verified_card_and_judges_on_it=[(0, 0)] evicted_on_card1=0 answered_card=[CardIndex(0)]`，重核派回卡 0、卡 1 换出 0 个，本腿当时的推论错在「卡 0 的空出来了还没收到时只能挑卡 1」：同一圈里那一件已经先收了。
2. 第一轮第 12 问「会漏一台的情形只有第 9 问那一种」：说宽了。当时只看了卡坏行；一台的服务停着（`not-running`）时它的卡坏、判不了都读不到，任务服务也不当毛病，第 11 问这一轮两格就落在这里。
3. 第一轮改法 F2 写的是「提交一次空的、再 poll」：落成的那一次提交借 `crc32c_of_ranges`，要新建四个带暂存的缓冲，不是空的，于是它自己会 OOM（第 5 问头一格）。F2 那句没写「不建缓冲」这个要求，是本腿当时没写清，记一次输。

第一轮第 5 问里「判段只问 serves、一直等」那半句说的是那时的样子（`control serves` 已删），不撤回、也不再引。

## 没打中的形状与取样

- 替身上写成探针的历史（三条探针，结局都是推的）：两张卡一大一小、一块 256 态、卡 0 显存不够一回（第 7 问）；借到之后借方 SIGSTOP 一个认领租期加 10 秒（第 9 问）；以 `card-lease-test-borrower` 直接认领卡键后 `lease list`（第 10 问）。另有两条现成测试换开关跑（第 13 问）。
- 按代码推、没打中的：重开之后在判的批（同设备没丢那一种撤回）；判不了与写结论之间判两遍；撤回链中间一节先记满（链先算好）；大卡坏了之后新登记的流、新入池的块、撤回的块（都过同一道判法）；`lease take` 停住或被 SIGKILL（至多 120 秒）；等号在列与认领之间失效（续不上就退）；等号让服务不放卡（服务不看等号）；`lease list` 过期、服务重起、截断；install 起回来时旧二进制（拷在清场之后）；`--dry-run`（不进停的那一段、自证格核了 systemctl 一条不起）；判法键表改字（自证格会红）。
- 没碰的：判的时候那几次等的上限（只当第 5 问「等不回来」的旁证）；`gpu-quiesce-run.sh` 本身（不在范围）；私有清场脚本按单元名停时的退出码怎么收（私有内容，没读那一层）；任务服务一侧去重落盘与没读成那几格（归辩方第 4 问）。

## 这条腿自己的限度

- 一条探针也没跑，也没建成冻结副本（`python3 scripts/hosts/minimal-copy.py … --with-test-inputs` 被权限闸拒，见文首）；探针可能编不过，编不过错在本腿。探针接在两个现有测试目标的末尾，用的是那两份里的私有类型（替身判器、`LeaseCase`、`BorrowingProcess`），那两份一改名探针就编不过。
- wgpu 那几句（`lose` 不放显存、要 poll 才放；暂存走致命那条路）是读 `~/.cargo/registry` 里 wgpu-core 30.0.1 的源码推的，没在卡上量；第 5 问「旧设备压着显存」「那一次提交自己 OOM」都靠它。按「卡只归 GPU 服务」，本腿不写拿卡的命令。
- 第 6、9、11 问能照喂的输入（喂任务服务的假 cards-failed、时钟回拨、对 control 发 SIGTERM）只写成了形状，没写成探针：载入任务服务的库会被项目代码闸拒，本机没有 faketime，control 自证的假清场没有「睡多久」的设定。
- 私有清场脚本只数过键名、看了判地址那一段的判法（没往报告里抄任何私有值），停单元那一层没读。
- `scripts/agents/cite-check.py` 没跑：派发提示写明不跑本仓的脚本；报告里每一句引文写进来之前都在被引文件里 `grep -nF` 过一次，行号取的是命中的那一行。wgpu 的引文路径不在仓里，cite-check 判不了，同第一轮。

## 要主 agent 跑的命令

前两项不交服务，第三项交服务；都在新 shell 里起。

1. 建冻结副本（直接跑、不经服务；本腿跑它被权限闸拒）：
   - 命令：`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-opus/copy --with-test-inputs`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看什么：退 0、副本里有 `gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs` 与 `card_lease_on_a_temporary_library.rs`，两份与主工作区那一版 `sha256sum` 相同（与 `research/prompts/gpu-oom-card-governance-r2-snapshot/judged-sha256.txt` 对得上）。草稿目录的 `.deliverables` 已写好这一份副本的「交接」行。
2. 建运行目录（不交服务）：`mkdir -p /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-opus-rerun-run`
3. 模型复跑：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=攻方腿模型复跑：gpu-verification-server 的测试目标 service_loop_with_judges_off_the_card 与 card_lease_on_a_temporary_library 里点名的 5 条测试（3 条接在末尾的探针、2 条现成测试换弄坏开关跑）; 范围=5 个测试; 为什么=判决要引的探针读数由主 agent 在冻结副本的拷贝上复跑' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-opus-rerun-run -- bash /home/fy5090/code/singlefs/research/prompts/gpu-oom-card-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-opus/copy /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/gpu-oom-card-governance-r2-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r2-opus-rerun-run`（往上没有 `.git`，服务认的树就是它；判决引完数之后删掉它，服务跟着删那一组编译目录）
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 是本腿预计的五个结局全中；`task-server/task log <号> --all` 里五行 `name=rerun_step … matched=yes|no`（`put-back-piece`、`stopped-borrower`、`direct-borrower-listed` 预计 `seen=ok`；`waiting-order-test-under-keys-held-switch` 预计 `seen=FAILED`；`two-borrowers-test-under-waiting-order-switch` 预计 `seen=ok`），末行 `name=rerun_steps_mismatched count=0`，另有读数行 `OOM_ATTACK_PROBE name=put_back_piece_back_on_its_card …` 与两行 `CARD_LEASE_ATTACK_PROBE name=…`。`seen=none` 是那一步没编过或没跑到那条测试，看运行目录里那一步的 `.out`，错在探针、交回同立场的腿修。`matched=no` 而 `seen` 是另一种结局，就是本腿那一格推错了。
   - 内存与时间（推的，没量）：从头编验证包、判定库服务与这个包的两个测试二进制，十几到二十几分钟；五步测试合起来约三分钟，其中第 9 问那一条要等一个认领租期加 10 秒（两分多钟）。

## 没做什么

- 辩方那几格（第 1 到 4 问）不判；不替主 agent 采纳。
- 没往统一任务服务交任务；判决要引的数一个都没复跑（命令在上一节），报告里所有探针结局都是推的。冻结副本没建成（权限闸拒），草稿目录里没有仓副本与编译目录要删。
- 副本上量出来的数不算入库装置上的数；真卡那几格（那一次提交自己 OOM、旧设备压着显存）没在卡上量，按「卡只归 GPU 服务」不写拿卡的命令。
- 第 5、6、9、11 问另几格能照喂的输入只写成形状，没写成探针（理由在「这条腿自己的限度」）。
