# GPU 服务接住 OOM 与卡坏上报：治理文件轮第一轮攻方腿（2026-11-19）

攻击面：第一轮「过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱」，落在正文「要腿判的」第 5 到 12 问。读的是此刻主工作区那一版（被判的 16 份文件 sha256 与 `research/prompts/gpu-oom-card-governance-r1-snapshot/judged-sha256.txt` 逐个相同，`sha256sum` 现核）。禁读清单里的三样没读。

## 复跑命令与模型文件

模型目录 `research/prompts/gpu-oom-card-governance-r1-opus-model/`：

| 文件 | sha256 |
|---|---|
| `probe/oom_attack_probe.rs` | 2d50f802bfd4cdfaa7d71af469c10f362c182f2c1abf7ad1f34bc26e6dd84c55 |
| `tools/rerun.sh` | eb7b65789014355eb23365952c48593f32b2f334b4360b93c8e9c6b693ce6aad |
| `tools/rerun-from-a-clean-work-directory.sh` | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`：把冻结副本拷进运行目录、把探针拷进 `gpu-verification-server/tests/`、点名五条测试跑 `cargo test`、摘出 `OOM_ATTACK_PROBE` 行。冻结副本是 `scripts/hosts/minimal-copy.py --with-test-inputs` 建的仓副本 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus/copy`（被判文件 sha256 与开工快照相同，探针也已放进它的 `gpu-verification-server/tests/`）。整条命令在文末「要主 agent 跑的命令」。

探针在替身判器上跑服务主循环（不上卡）。每条先打一行 `OOM_ATTACK_PROBE name=…` 读数，再断言本腿推出的结局；断言红就说明本腿推错了。下面写的探针结局全是「推的，待主 agent 跑」，本腿一条都没跑。

## 各格判定一览

| 问 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 5 判两次、写错卡写错流 | 不成立 | 撤回的块回话挂到新号上，旧号进 `withdrawn`，旧结论不收；交回的账只记新那一份 | 代码；探针 E、D（推的） |
| 5 丢块 | 推不出（打中） | 腾空的卡上又 OOM 的块被这个进程拉黑，待判行留在库里没人取，也没有告警；而单独重判用的显存并不比那一波少，撤回之后 wgpu 还压着上一波没提交的暂存 | 文件:行，推的 |
| 5 停止信号、借卡 | 不成立 | 撤回的块照样单独判完再停，借卡也等手上的块写回之后才放 | 探针 D（推的）；代码 |
| 6 那一刻卡 1 的账与驻留判器 | 不成立 | `take_an_out_of_memory` 只动 OOM 那张卡的几项 | 代码 |
| 6 卡 1 驻留判器被动到 | 推不出（打中） | 放回的核对组要一张「腾空的卡」，卡 0 的「空出来了」还没收到时只能挑卡 1，于是把卡 1 的判器全部换出，虽然卡 1 不换也放得下 | 探针 B（推的）；`service_loop.rs:734` |
| 6 撤回的块一直不派 | 不成立（但是退化） | 不会一直不派。不过 K 块要在卡 0 上排成 K 波一块一块判，每波换出再装同一个判器，卡 1 这期间空着 | 探针 A（推的） |
| 7 新表两份之后还造得出 OOM | 推不出（打中） | 核对器的六张表也是用 `create_buffer_init` 建的，账上只算一份；OOM 之后 wgpu 待提交的写还压着显存 | 文件:行，推的 |
| 7 原来装得下的流被拒、永远派不出去 | 不成立 | 登记时按两份算是对的。不饿死那一遍先只装这一条流 | 代码 |
| 8 判法 panic 报成卡坏 | 推不出（打中） | 照报卡坏、叫人重置卡。某一条流每装必断言失败时，被拒的块回库后又被认领，接连弄坏每一张卡 | 探针 C（推的）；文件:行 |
| 8 设备丢了与 panic 谁先到 | 部分打中 | 不存在 panic 先到这回事。可是 wgpu 在暂存缓冲、提交这几条路上把 OOM 当致命错误、丢掉设备，服务把这种情况报成卡坏，这张卡一直停到服务重起 | 文件:行，推的 |
| 8 只读这个进程的行 | 成立 | 用 `-b _PID=<主进程号>` 过滤 | 代码 |
| 9 拖住、抛异常 | 不成立 | 在另一条线程里读，只接住 OSError 与 TimeoutExpired 两种异常 | 代码 |
| 9 刷屏、重启后重发 | 推不出（打中） | 第二台连不上、日志读不了都按「没有卡坏」算，算成「好了」，下一回读到又给每个会话发一遍。任务服务重起后第一轮也一样，重发的是会话消息，不只是告警表里的那一条 | 文件:行，推的 |
| 9 没任务的会话 | 不成立 | 名下有了在排或在跑的任务就收得到 | 代码 |
| 10 算键登记 | 不成立 | 两个函数里各只有一处起子进程，登记写的理由与代码相符 | 代码；「照连」那半句没复跑 |
| 11 开关重叠 | 不成立 | 新开关各弄坏一处，没有两条弄坏同一处的 | 代码 |
| 11 从源码读前缀 | 前缀不成立，字段打中 | 前缀挪位置或改名会红。卡坏行的字段改名时三份自证都静默通过，任务服务的键里卡号退成 `?` | 文件:行 |
| 12 多机 | 不成立 | 第二台跑的是共享路径上同一份 control，读的是第二台自己的日志，键里有台名、不会撞 | 代码 |
| 工具员七条局限 | 1、2、3、4、6 算缺口；5 不算；7 已记账、不在本轮范围 | 见「工具员局限逐条」 | |

「推不出」的照正文规定算打中。各格的四句（分不分辨臂、系统当时看不看得到、满足判据的哪一分句、改法还中不中）写在各节末尾。这里的「臂」指本轮的实现与改动之前的整卡停用两种。

## 第 5 问：OOM 之后的块

### 不成立的几种（判两次、写错卡、写错流、停止信号、借卡）

- 判两次：撤回时旧号先进 `withdrawn`，之后到的旧结论在「if self.withdrawn.contains(&piece.block) {」（`gpu-verification-server/src/service_loop.rs:1069`）这一句被丢掉。回话改挂到新号上：「self.replies.insert(again, reply);」（`gpu-verification-server/src/service_loop.rs:1205`）。取活循环那一侧照入池时拿到的那一头收回话：「answer: Receiver<Result<JudgedBlock, ServiceRefusal>>,」（`gpu-verification-server/src/serving_from_the_library.rs:363`），块的名字跟着这一头走，不经新号去找，所以写不到别的流上。
- 写错卡：新号那一份的账只从它自己的结论来。探针 E 断言一块跨两张卡、卡 0 OOM 之后交回的 `work_on_each_card` 只有卡 0（推的，待主 agent 跑）。
- 停止信号：撤回的块直接进池（`put_back_the_block` 调的是 `pool.add_block`，不走 `admit_the_block`），所以收到「判完就停」之后照样有卡判它；`replies` 不空时主循环不退出。探针 D：OOM 之后马上 `drain_and_stop`，断言每块交回、每态只判一次（推的，没打中的形状）。
- 借卡：「2. 有人要借：这一圈起三族候选都不扫、不认领；手上入了池的块与判表照常判完、写回」（`gpu-verification-server/src/serving_from_the_library.rs:38`）。撤回的块回话那一头还在 `in_flight` 里，要等它判完、写回之后才放卡键。按代码推不出丢块。

### 打中：丢块（腾空的卡上又 OOM，被这个进程拉黑）

历史（推的）：
1. 卡 0 一波新装两条流 P、Q（`create_buffer_init` 建表），装 Q 的表时 OOM。P 的表与它的暂存这时还在 wgpu 的待提交写里：「pending_writes.consume(staging_buffer);」（`wgpu-core-30.0.1/src/resource.rs:1032`）、「pending_writes.insert_buffer(self);」（`wgpu-core-30.0.1/src/resource.rs:1033`）。要等下一次提交并且做完，这两份才还给驱动。
2. 工作线程说「判器缓存已丢掉卡上全部判器（显存还给驱动）」（`gpu-verification-server/src/card_worker.rs:7`），`after_a_panic` 也只是 `self.judges.clear()`，前后既不提交也不 poll。P 那两份显存其实还占着，这一句越位了。
3. 撤回的块单独上卡 0，草稿区照整张卡分：「let scratch = card.mebibytes_for_the_judges() - building.tables_while_loading(&[stream]);」（`gpu-verification-server/src/batch_scheduling.rs:780`）。账上的峰值又顶满整张卡，而且装表发生在第一次提交之前。第 1 步留下的那两份再加上这一次，又 OOM 的可能比那一波还大。
4. 这一块报拒 `OutOfMemoryOnAnEmptiedCard`，取活循环把它拉黑：「self.refused_for_running_out_of_memory.insert(name);」（`gpu-verification-server/src/serving_from_the_library.rs:1052`）。README 写着「在腾空的卡上也显存不够的块，这个进程不再认领、待判行留在库里，不写红判定」（`gpu-verification-server/README.md:290`）。
5. 判段那一侧只问 serves：「问的是 `serves`，不问 `check`」（`crates/singlefs-verification/src/crash_amplification.rs:36810`），只要服务还收这一族，它就一直等。`control check` 也不报这件事：「`control check` 只读 `GPU_VERIFICATION_SERVER_VERSION_MISMATCH` 与 `GPU_VERIFICATION_SERVER_CARD_FAILED` 两种行」（`gpu-verification-server/README.md:287`）。所以等这一块的那几片一直卡着，没有红判定，也没有告警。第二台的卡与额度相同、照样 OOM 的话，两台都把它拉黑。

不靠 wgpu 待提交写的第二条路（推的）：账高估了真额度（README 自己列着「卡给服务的额度是开卡那一刻的空闲读数，之后不再读」，还有别的进程开卡），高估的部分超过余量时，整卡单独重判与那一波一样顶满整张卡，照样 OOM。重判并没有比那一波少用显存。

能照喂的输入：替身模型不了 wgpu 的待提交写，所以这一格只有 文件:行。上卡的复现（推的）：一张卡上让一波同时新装两条表各约 0.4 倍额度的流，第二条装表时 OOM，看 `OUT_OF_MEMORY_CAUGHT` 之后单独重判那一块是不是 `blocks_refused=1`。按「卡只归 GPU 服务」，这个复现要用户在服务上起，本腿不写成任务命令。

四句：不分辨臂（改动之前整卡停用，这一块也拿不到结论）；系统看得到，按 wgpu 源码推，`after_a_panic` 里提交一次空的、再 `poll` 等它做完，就能把待提交写清掉（推的，没在卡上量）；满足的是第 5 问「一块丢掉（待判行留在库里而这个进程再也不取，也没有别的服务会取）」那一分句；改法见「改法与各格」F2、F3。

## 第 6 问：好卡受连累

### 不成立：卡 0 OOM 的那一刻，卡 1 的账与驻留判器

`take_an_out_of_memory` 里每一处只用 OOM 那张卡作键：「self.residents.remove(&card);」（`gpu-verification-server/src/service_loop.rs:1225`）、「let lone = self.lone_block_on_card.remove(&card);」（`gpu-verification-server/src/service_loop.rs:1228`）。卡 1 的驻留账与工作线程都没动。唯一的例外是一块跨在两张卡上：整块撤回之后，卡 1 判过的那一截白判了（探针 E 断言卡 1 在头一波判过 10..20，最后整块只记卡 0，推的）。这只是多花了算力，结论没有错。

### 打中：放回的核对组把卡 1 的驻留判器全部换出

历史（探针 B，推的，待主 agent 跑）：
1. 两张卡各驻留一条流的判器（表 40 MiB，每张卡额度 `2*40 + mebibytes_of_states(10) + 3`），两张都空着。
2. 交一组核对块，借 2 MiB。卡 0 旁边放得下，不用换出，派到卡 0，`on_an_emptied_card` 是 false：「on_an_emptied_card = residents.is_empty();」（`gpu-verification-server/src/service_loop.rs:661`）。
3. 卡 0 核的时候 OOM。主循环先收「接住」那一件，把这一组放回排头，标成要腾空的卡：「needs_an_emptied_card: true,」（`gpu-verification-server/src/service_loop.rs:1288`）。同一圈接着就找卡，可卡 0 的「空出来了」排在通道里还没收：「self.idle.insert(card);」（`gpu-verification-server/src/service_loop.rs:968`），空着的只有卡 1。
4. 要腾空的那一组只试「换出全部」这一种：「|| (must_empty_the_card && evicted_count != order.len())」（`gpu-verification-server/src/service_loop.rs:734`）。卡 1 驻留的那个判器被换出，虽然它旁边还空着 43 MiB。
探针断言：卡 0 上那一回 panic 了不记，记下的唯一一回核在卡 1 上，核的时候卡 1 上一个判器都没有，卡 1 换出了 1 个。头一波两条流要各装在一张卡上，这是取样前提；机器忙、第二块晚到 20 毫秒以上时这一条会红，那是取样问题，不算推翻。

块那一路有「等撤回它的那张卡」：「self.waiting_for_an_emptied_card.insert(again, origin);」（`gpu-verification-server/src/service_loop.rs:1213`），核对组这一路没有，两路不对称。文件头说「别的卡不动」，在这一格上越位了。

四句：分辨臂（改动之前卡 0 直接报坏，不会放回）；系统看得到核对组是在哪张卡上 OOM 的（`take_an_out_of_memory` 拿着 `card`）；满足的是第 6 问「卡 1 … 驻留的判器 … 有没有被动到」那一分句；F4 修这一格，F2、F3 碰不到。

### 不成立（但是退化）：撤回的块一直不派

卡 0 照例在「接住」之后紧跟着报「空出来了」，`place_the_lone_blocks` 在每一圈最先跑，所以撤回的块不会一直不派。退化在别处（探针 A，推的）：一波 40 块、每块 1 态，全在卡 0 那一批里，OOM 之后 40 块各占卡 0 单独一波。每一波都按「evictions: residents_on_the_card」（`gpu-verification-server/src/batch_scheduling.rs:783`）先换出同一条流的判器再装回来，把表重传一遍。这段时间卡 1 只能接别的流，撤回的块一块也分不到。探针断言：除头一波外的单独波数等于撤回块数，卡 1 判过的段与撤回的块没有交集，每态只判一次。工具员局限第 3 条说的就是这件事，本腿判它算缺口：一次 OOM 可以让一张卡几十波都在逐块重传表。

## 第 7 问：账

### 打中：「新表两份」之后照样造得出 OOM

- 核对器：账上只算一份表，「bytes_of_the_facts_tables_on_the_card(facts),」（`gpu-verification-server/src/card_memory_ledger.rs:157`）。可它的六张事实表是这样建的：「wgpu::util::DeviceExt::create_buffer_init(」（`crates/singlefs-verification/src/crash_verify_gpu.rs:552`），用途只有 STORAGE，没有 MAP_WRITE，wgpu 会另开一份同样大的暂存：「StagingBuffer::new(self, wgt::BufferSize::new(aligned_size).unwrap())?;」（`wgpu-core-30.0.1/src/device/resource.rs:1164`）。判器那一侧本轮按两份算了，核对器这一侧没有，这是内部矛盾。可造的输入（推的）：一张卡上驻留的判器占到 `额度 − 核对器借的`，再交一组事实表 F MiB 的核对块，账上借得到，实际峰值多出 F。
- OOM 之后：见第 5 问第 1、2 步，wgpu 的待提交写还压着上一波的表与暂存，而账已经当作清空了。
- 已知、不在本轮范围的：额度只在开卡时读一次、别的进程开卡、驱动那一份够不够。README「还没做的」列着这几样。

四句：不分辨臂（改动之前同样会 OOM，只是接下来整卡停用）；系统看得到，核对组的事实表大小在 `mebibytes_of_a_facts_verifier` 手里；满足的是第 7 问「新表两份之后还造得出 OOM 吗（…核对器借显存）」；F3 修核对器这一格，F2 修 OOM 之后那一格。

### 不成立：原来装得下的流登记被拒、或永远派不出去

登记时判「judge_mebibytes_at_one_state > largest_card_mebibytes_for_the_judges」（`gpu-verification-server/src/service_loop.rs:1014`）。照第 5 问引的 wgpu 源码，两份表加一个态装不下的流，装的那一刻本来就会 OOM，所以在登记时拒更早、也更对。表受绑定上限约 2 GiB 管着，真卡额度是几十 GiB，正常情况下碰不到这条拒（推的；只有开卡时这张卡已经被别的进程占满、额度读得很小时才碰得到）。不饿死那一遍先只装这一条流：「let mut reloaded = vec![stream];」（`gpu-verification-server/src/batch_scheduling.rs:528`），装不下的旧流才不并进来，所以造不出永远派不出去。工具员局限第 6 条（`state_count/3 == 1` 时会不会被拒）在替身上推不出会拒；它是「没在卡上量过」这一类缺口，归到第 8 问末尾「最脆弱」那一条。

## 第 8 问：卡坏误报与漏报

### 打中：判法自己 panic 报成卡坏，一条坏流接连弄坏每一张卡

历史（探针 C，推的，待主 agent 跑；替身在装表时按设定断言失败，相当于真卡上「判器的表 {table_bytes} 字节超过这张卡的绑定上限 {limit}：这条流 GPU 判不了」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:1399`）这一句）：
1. 流 X 的块落到卡 0，装表时断言失败。设备没丢、也没记过 OOM，所以答 `PanickedOtherwise`，卡 0 报卡坏、不再排，这一块报拒 `CardFailed`。
2. 取活循环只拉黑 OOM 那一种拒，`CardFailed` 只放回库里：「self.pool.withdraw(&name);」（`gpu-verification-server/src/serving_from_the_library.rs:1048`），README 写「被撤的块（判它的卡坏了）、配不出流的块：认领留到租期过了」（`gpu-verification-server/README.md:292`）。租期一过它又成了候选，这一回落到卡 1，卡 1 也坏。第二台认领到它的话，那台的卡也一张接一张坏。
3. 卡全坏之后，好流登记被拒（`StreamFitsNoCard`，因为没坏的卡里最大的额度取的是 0），好块被拒 `NoWorkingCard`。每一张卡都打卡坏行，control check 每一张都给「主 agent 当场弹窗请用户重置这一台的那张卡」（`gpu-verification-server/control:318`），可重置卡治不了软件的错。
探针断言：两张卡先后报卡坏、`device_lost` 都是 false，头一回与再交一回都拒成 `CardFailed`。

判据分句：第 8 问「判法自己的 panic（例：断言失败）被报成卡坏、叫人重置卡」，加上第 6 问「好卡受连累」。四句：不分辨臂（改动之前也这样）；系统看得到「设备没丢、也没记过 OOM」（`PanickedOtherwise` 就是这个意思），能分出这是软件的错；F6 修这一格，F1 到 F5 碰不到。工具员局限第 1 条算缺口，而且比报告里写的重：坏的不止一张卡，是一张接一张。

### 部分打中：设备丢失与 panic 的先后

- panic 先到、`lost_reason()` 还没回调：按 wgpu 30.0.1 源码推不出。wgpu 判出要丢设备时当场调回调：「self.lose(&error.to_string());」（`wgpu-core-30.0.1/src/device/resource.rs:706`）→「device_lost_closure(DeviceLostReason::Unknown, message.to_string());」（`wgpu-core-30.0.1/src/device/resource.rs:5424`），然后才把错交回调用方。没接住的错误的处理器是先放锁再调的（「having released our lock on `sink_mutex`」，`wgpu-30.0.1/src/backend/wgpu_core.rs:311`），panic 时 wgpu 没持着锁。这一半不成立（推的，读源码）。
- 打中的是另一半：OOM 被报成卡坏。暂存缓冲走的是致命那条路：「.map_err(|e| device.handle_hal_error(e))?;」（`wgpu-core-30.0.1/src/resource.rs:1265`），`handle_hal_error` 碰到 OutOfMemory 就丢设备。`create_buffer_init` 建表时，主缓冲放得下而暂存放不下，正好就是本轮「新表两份」要防的那一种。这时设备丢了，OOM 也记上了。服务先看丢没丢：「if let Some(reason) = condition.lost_reason() {」（`gpu-verification-server/src/gpu_judges_on_a_card.rs:306`），于是报 `DeviceLost`，卡坏行写 `device_lost=yes`，这张卡停到服务重起，也叫人重置卡。验证包自己把这种丢失认成显存不够：「if message == wgpu::hal::DeviceError::OutOfMemory.to_string() {」（`crates/singlefs-verification/src/gpu_unit_checks.rs:363`）。服务没用这个判法，与验证包对不上，是内部矛盾。用户原话「OOM 需要倍catch 不能影响好的卡」：这种 OOM 没被接住，这张卡也没被当成好卡。
- 四句：分辨臂（本轮只接住了不致命的那一种）；系统看得到，丢失原因的字串里是 wgpu 那句「Out of memory」；满足的是第 8 问「设备丢失…时…被当成判法 panic 还是 OOM」，这里落进的是第三种结局「卡坏」；F1 修这一格，要配上重开设备的路（`OpenedGpuCards` 今天不能单开一张）。

### 成立：control check 只读这一个进程

「command = [paths.journalctl, "--user", "-b", f"_PID={main}", "-o", "json", "--output-fields=MESSAGE,_PID", "--no-pager"]」（`gpu-verification-server/control:1041`）。不按单元过滤，只按进程号，所以同一次开机里进程号被重用、而那个旧进程又打过以卡坏行前缀起头的行时会误收。要碰上这种重用，进程号得先绕一圈，概率很低（推的）。加上 `_SYSTEMD_USER_UNIT=<单元>` 就能堵上（推的，被攻过零轮）。

## 第 9 问：告警

### 不成立：拖住、抛异常

读卡坏在另一条线程里做，服务健康那一轮只取上一回读到的。`read_gpu_cards_failed` 接住 OSError 与 TimeoutExpired，退非 0 时留着上一回的；`failed_cards_of_the_report` 只做 split 与 partition，乱码行跳过去不抛。control 不在的话，python 退 2，也留着上一回的。时限有一处对不齐：「GPU_CARDS_FAILED_READ_SECONDS = 150   # 起一回 gpu-verification-server/control cards-failed 的时限：它问第二台至多等 60 秒、读日志至多 120 秒」（`task-server/task_server.py:1324`），而这一台读日志（120 秒）与问第二台（60 秒）是一前一后做的，最坏要 180 秒，比 150 秒长。超时的那一回留上一回的，不拖住，只是读到的会旧。

### 打中：连不上、读不了都当成「没有卡坏」，造出刷屏；重启之后会话消息也重发

- 第二台连不上记 unreachable，可 `cards_failed_answer` 末尾照样「return 0」（`gpu-verification-server/control:1453`）。任务服务见退 0，就把读到的整份换上：「self.gpu_cards_failed = found」（`task-server/task_server.py:5575`）。这一回第二台的坏卡不在里面，健康那一轮判它好了：「for kind in [kind for kind in self.health_notified if kind not in found]:」（`task-server/task_server.py:5592`）、「del self.health_notified[kind]」（`task-server/task_server.py:5594`）。下一回连上了，又给每个有任务的会话发一遍「GPU 服务卡坏」。这一台 journalctl 退非 0、或 120 秒没读完（unreadable）也是一样。not-running 不算在内：服务重起之后是新进程、新的键，当成好了是对的。可照喂的输入：假 cards-failed 一回打出卡坏行，下一回只打 `GPU_VERIFICATION_SERVER_CARDS_FAILED_READ host=<第二台在多机配置里的地址> state=unreachable lines=0` 并退 0，再下一回又打出卡坏行，会话应收两条（推的）。自证格 `health-gpu-card-failed-notified` 只演了一回坏、一回好，没演「不知道」。本腿没写喂任务服务的探针，因为载入 `task_server_lib` 会被项目代码闸拒。
- 任务服务重起：「self.gpu_cards_failed = []」（`task-server/task_server.py:2999`），重起后头一轮健康拿到的是空表（读的线程刚起），于是把持久化留下来的 `health_notified` 里那几个 gpu 种类当成好了删掉，等读回来再发一遍。工具员局限第 4 条只说告警表会再写一条；实际上会话消息也重发，每重起一次发一次。这几份文件经常被别的会话换上，换上就要重起。
- 四句：不分辨臂（改动之前没有这条告警）；系统看得到 `state=unreachable` 或 `state=unreadable` 这一行，能分出「不知道」与「好了」；满足的是第 9 问「刷屏」「服务重启后重发一条是不是可接受」；F7 修这一格。

### 不成立：没任务的会话收不到

`watch_health` 没有收件人时不查。卡坏会一直留在日志里，到会话名下有了在排、在跑的任务那一轮就发。不是永远收不到，只是晚到。

## 第 10 问：算键登记

不成立（推的，没复跑算键）。`read_gpu_cards_failed` 里起子进程的只有一处 `subprocess.run(command, …)`，`command` 是「[fake] if fake else [sys.executable, "-B", lib.GPU_SERVICE_CONTROL, "cards-failed"]」，两种取法写在同一个函数里，登记的理由与代码相符。`card_failed_lines` 里起子进程的也只有 journalctl 一处，它取 `paths.journalctl`，自证里是假 journalctl，平常是系统的 journalctl，不起仓里的代码。登记按（文件、函数、词）认，这两个函数里没有别的起法可以被顺带放过。改法提醒（推的）：以后在 `read_gpu_cards_failed` 里加一处起子进程、而头一个参数也叫 `command` 的，会被这一行静默放过。这是整张表的通病，不是本轮多出来的。

## 第 11 问：弄坏开关

- 重叠：不成立。服务 12 个新开关各改一处分支，同一条测试被四个开关点名，但每个开关弄坏的是测试里不同的那一段断言。control 的 `card-failure-unread`、`card-failure-of-earlier-processes-counted`、`out-of-memory-caught-counted-as-card-failure` 三个点名同一格，分别对应格里 this-process、earlier-process、out-of-memory-caught-only 三种情形，各红各的。
- 从源码读前缀：前缀那一半不成立。读不到或改了名都会红：「failed = re.search(r'pub const CARD_FAILED_LINE_PREFIX: &str = "([^"]+)";', source)」（`gpu-verification-server/control:2235`），找不到就把 `unreadable` 交回去当成一条 problem。rustfmt 把这一行折成两行时同样会红，不会静默过。
- 打中的是字段那一半：卡坏行的字段 control 认、任务服务也认，三份自证用的却是各自手写的字面行，没有一份从源码读格式。服务要是把 `card=` 改成别的名字，三份自证照样绿，任务服务这一句就取不到卡号：「card = next((word.partition("=")[2] for word in words[3].split() if word.startswith("card=")), "?")」（`task-server/task_server_lib.py:2213`）。同一个进程的几张坏卡于是并成一个键 `<台>:<进程>:?`，只发一条，后坏的那几张漏报。四句：不分辨臂；系统看不到（任务服务手里只有那一行字）；满足的是第 11 问「在源码挪了位置或改了名时会不会静默过」；F8 修这一格。
- 真卡那一份 `after_a_panic`（`gpu-verification-server/src/gpu_judges_on_a_card.rs:301-315`）没有开关、也没有测试：替身判器用的是自己写的 `after_a_panic`。第 8 问说的两种分错都落在这一段里。

## 第 12 问：多机

不成立。`cards_failed_of_the_peer` 经 ssh 在第二台同一绝对路径的仓里起同一份 control（两台共享这个仓），带上转来的标记，读的是那台单元的主进程号与那台的 `journalctl --user`。状态目录在各台自己的家目录下，两台不共用。键是 `<台>:<进程>:<卡号>`，这一台写 `this-host`，第二台写它的地址，两台各自 OOM、各自卡坏都撞不到一起。会漏一台的情形只有第 9 问那一种：连不上的那一回当成好了，下一回再发。

## 工具员局限逐条（该不该算缺口）

| 条 | 判 | 依据 |
|---|---|---|
| 1 判法 panic 照旧当卡坏 | 算缺口，比报告写的重 | 第 8 问：一条坏流接连弄坏每一张卡，两台都会 |
| 2 腾空的卡上也 OOM 的块不再认领 | 算缺口 | 第 5 问：判段只问 serves、一直等，control check 不报，没有告警 |
| 3 撤回的块只等撤回它的那张卡 | 算缺口（吞吐） | 第 6 问：K 块排成 K 波，每波重传表 |
| 4 告警去重只在内存 | 算缺口，范围比报告写的大 | 第 9 问：会话消息也重发；连不上的那一回也会造出重发 |
| 5「另一张卡逐块相同」只核成性质 | 不算 | 撤回之后卡 1 判过的那一截丢掉，性质（每态只判一次、结论对）够判（探针 E） |
| 6 上卡的两条测试没跑 | 算缺口 | 「新表两份」的前提（暂存在显存里）与「OOM 之后这张卡照常用」都没在卡上量过；本腿第 5、8 问打中的两处都只在真卡上才有 |
| 7 驱动份额、额度不重读、别的进程开卡 | 记账了，不在本轮范围 | README「还没做的」列着 |

## 最脆弱的一条

真卡那一份 `after_a_panic` 的判法：先看丢没丢，再比 OOM 次数比上一回多没多。两件事都没在卡上验过：一是 wgpu 致命那几条路上的 OOM（暂存、提交）会丢设备，落进「卡坏」；二是 OOM 之后宣称「显存还给驱动」，其实待提交写还压着。这一段没有替身能演，也没有开关；本轮所有测试都是替身判器在答 `after_a_panic`，所以这一段从没跑过。

## 改法与各格（都只在本腿推过、被攻过零轮，一格都没量）

| 改法 | 修哪一格 | 量没量 |
|---|---|---|
| F1：`after_a_panic` 看到丢失原因是 wgpu 的「Out of memory」时，不报卡坏，改成「重开这张卡的设备」（要先给 `OpenedGpuCards` 加单开一张的路） | 第 8 问 OOM 报成卡坏 | 推的，待主 agent 跑 |
| F2：`after_a_panic` 丢掉判器之后，提交一次空的、再 `poll` 等它做完，然后才报接住 | 第 5 问丢块、第 7 问 OOM 之后 | 推的，没在卡上量 |
| F3：核对器借的显存里，事实表按两份算（同 `mebibytes_while_loading_a_judge`） | 第 7 问核对器 | 推的，待主 agent 跑 |
| F4：放回的核对组只等它 OOM 的那张卡（同块那一路），或者只要求腾出它要借的那一份、不腾空整张卡 | 第 6 问核对组换空卡 1 | 推的，探针 B 改断言即可量 |
| F5：同一张卡同一条流撤回的块并成一波单独判，不一块一波 | 第 6 问退化 | 推的，探针 A 改断言即可量 |
| F6：`PanickedOtherwise` 不报卡坏，改成拒这一块并让取活循环拉黑这一块（或这条流），卡照常用；同一张卡连着几回判法 panic 才报卡坏 | 第 8 问连坏 | 推的，探针 C 改断言即可量 |
| F7：cards-failed 某一台的状态是 unreachable 或 unreadable 时，任务服务留着那一台上一回的坏卡；重起后头一回没读完之前不删 `health_notified` 的 gpu 种类 | 第 9 问刷屏、重起重发 | 推的，待主 agent 跑 |
| F8：control 与任务服务的自证从 `service_notice_line` 的源码或服务测试的产物取卡坏行的样例，不手写 | 第 11 问字段漂移 | 推的 |

各改法碰不到的格：F1 只管致命那几条路上的 OOM；F2、F3 不改「重判的峰值与那一波一样顶满整张卡」那一条路（额度高估的那一种）；F6 不管 OOM。

## 没打中的形状与取样

- 替身上试过的历史（探针五条，全是推的结局，待主 agent 跑）：一波 40 块各 1 态（第 6 问）；两张卡各驻留一条流时核对组在卡 0 OOM（第 6 问）；一条流每装必断言失败，两张卡（第 8 问）；OOM 之后紧接着判完就停，6 块（第 5 问）；一块 30 态跨两张卡（第 5、6 问）。卡都是两张，额度取 `card_leaving` 的小数（判器表 2 或 40 MiB，一个态的草稿区约 8 MiB）。
- 按代码推、没写探针的：OOM 之后借卡；OOM 在核对组与判块交替时，放回的核对组与等腾空卡的块抢同一张卡（`place_the_lone_blocks` 在每一圈最先跑，块先拿到卡）；卡 0 在撤回的块等它时坏了，换头一张空着的好卡：「.find(|card| card.card == origin || self.failed.contains(&origin))」（`gpu-verification-server/src/service_loop.rs:601`）；卡全坏时等腾空卡的块被拒；不饿死那一遍按两份表算（先只装这一条流）；cards-failed 输出乱码、control 不在、ssh 卡住。这几样都没推出错。
- 没碰的：借卡那一半的认领键与租期；验证包那一侧 `mebibytes_a_judge_needs` 与对拍。

## 这条腿自己的限度

- 一条探针也没跑。读数行与断言都是推的，探针可能编不过；编不过的话错在本腿，不在被判的代码。
- wgpu 的结论（暂存走致命那条路、待提交写压着显存、丢设备时当场回调）是读 `~/.cargo/registry` 里 wgpu 30.0.1 与 wgpu-core 30.0.1 的源码推出来的，没在卡上量。暂存落在显存还是主存，取决于 Vulkan 给 `CpuToGpu` 挑的内存类型（ReBAR 上是显存）：「新表两份」与本腿第 5、8 问打中的那几格都靠这个前提。
- 第 9 问没写喂任务服务的探针（载入 `task_server_lib` 会被项目代码闸拒），只有 文件:行与一段可照喂的假输出。
- control 与 task-server 三份只看了正文点名的那几段（卡坏、cards-failed、`read_gpu_cards_failed`、`watch_health`），别的会话后来换上的改动没判。

## 要主 agent 跑的命令

1. 探针（仓副本里点名五条测试）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=gpu-verification-server 的测试目标 oom_attack_probe 里的 5 条测试; 范围=5 个测试; 为什么=攻方腿探针：替身判器上量接住 Out of Memory 之后撤回块怎么派、核对组重核挑哪张卡、判法断言接连弄坏两张卡、OOM 后停、跨卡块' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus/copy -- cargo test -p gpu-verification-server --test oom_attack_probe -- probe_put_back_blocks_wait_one_by_one_for_the_card_that_ran_out_of_memory probe_a_facts_job_put_back_after_running_out_of_memory_empties_the_other_card probe_a_stream_whose_judge_panics_on_loading_fails_every_card_one_after_another probe_out_of_memory_then_drain_and_stop_still_judges_every_block_once probe_a_block_split_across_two_cards_is_put_back_whole_and_the_good_card_work_is_dropped --exact --nocapture`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus/copy`（`.deliverables` 里写成「交接」）
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`task-server/task log <号> --all` 里五行 `OOM_ATTACK_PROBE name=…`（读数）与 `test result:` 那一行。`5 passed` 是本腿推的五种结局全中。哪一条 `FAILED` 就是本腿那一条推错了，看它的断言消息；编不过是探针的错，交回同立场的腿修。
   - 内存与时间（推的，没量）：要从头编验证包（带 gpu 特性）、prune-server 与这个包的测试二进制，十几到二十几分钟；五条测试本身几秒。
2. 模型复跑（判决要引的数由主 agent 复跑；在新 shell 里起）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=攻方腿模型复跑：gpu-verification-server 的测试目标 oom_attack_probe 里的 5 条测试; 范围=5 个测试; 为什么=判决要引的探针读数由主 agent 在冻结副本上复跑' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/gpu-oom-card-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus/copy /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/gpu-oom-card-governance-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus`
   - 看哪几行：`task wait` 末行 `exit=0` 算过（`rerun.sh` 把 cargo 的退出码原样交出；每一步都是一件，退非 0 的件数是 0 才算过）；日志里 `name=rerun_probe_exit status=0`，以及与第 1 条相同的五行 `OOM_ATTACK_PROBE`。跑完删掉运行目录 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus-rerun-run`。
   - 内存与时间（推的）：同第 1 条，另加把副本拷进运行目录的约 1 分钟。

## 没做什么

- 正推那几格（第 1 到 4 问）不判；不替主 agent 采纳。
- 没往统一任务服务交任务；判决要引的数都没复跑（命令在上一节）。上面所有探针结局都是推的。
- 副本上量出来的数不算入库装置上的数。真卡那一份 `after_a_panic`、wgpu 的待提交写与致命 OOM 那几条路都没在卡上量；按「卡只归 GPU 服务」，本腿不写拿卡的命令。
- 交接的仓副本 `/home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-opus/copy`（58M）留给主 agent 跑上面两条，用完就删。

## 引文核对

`python3 scripts/agents/cite-check.py <本报告>` 末行：「✓ 核了 40 处引文、3 处裸行号，对上 34 处，没判 9 处（其中裸行号没有引文可比 2 处；逐处列在上面）」。没判的那几处是 wgpu 源码的引文，路径不在仓里。加上 `--root /home/fy5090/code/singlefs --root /home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f` 再跑一次，末行：「✓ 核了 40 处引文、3 处裸行号，对上 40 处，没判 3 处（其中裸行号没有引文可比 3 处；逐处列在上面）」。
