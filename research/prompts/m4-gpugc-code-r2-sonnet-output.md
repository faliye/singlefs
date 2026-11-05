# m4-gpugc-code-r2 辩方腿（Sonnet）报告：替第 54、55 批的写法与第一轮判决第二、三节辩护（2026-11-01）

读法：冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r2/repo/` 简称「冻结副本」；代码引文的路径从它的根起，行号去冻结副本里现查（`grep -nF`）；kb 引文写 kb 文件自己的行号，决策文件即 `.claude/kb/decisions/26-后台整理与放置回收.md`（下称「决策文件」）。冻结副本里本报告引到的 14 份源文件与主工作区此刻逐字节相同（`cmp -s`，量的）；`crates/mutations.tsv` 与主工作区此刻不同，变异行只写名字与冻结副本里的行号，不写成路径加行号的引文。
没起 cargo、没起 GPU、没往服务交任务；凡标「推的」的都是读码推出来的，主 agent 跑出之前不引。

## 零、复跑命令与模型文件的 sha256

模型目录 `research/prompts/m4-gpugc-code-r2-sonnet-model/`（一份只打印的探针、复跑脚本、预测）。复跑命令见第八节，整条是 `task-server/task submit -- bash …/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录> --model <模型目录>`；我没有跑。预测写在 `out/predicted.txt`，标「推的」。

| 文件 | sha256 |
|---|---|
| `probe/resubmission_count_probe.rs` | `c7e412a4b747698b12f197ab33d1cce3659c57c40ea940ccb2d76caebf979800` |
| `tools/rerun.sh` | `adf48d877a778a0c8625bddacc959bf52c91963061c455ef9c42273d8792b589` |
| `out/predicted.txt` | `daabfe347c292590e75cc1c28f85e82ffa3942eb06d0a7df78d4183e9b574e1a` |

## 一、T1–T4 每格结论（先看这里）

| 格 | 结论 | 最要紧的一处 |
|---|---|---|
| T1 第一轮的修补本身 | 第 1–8、10 条的原反例在冻结副本今天的代码上都不再成立；改法做的是验收说的那件事，没有把验收说的事改反，但有四处缺口（见第二节表后「缺口」），其中缺口 1（变异行留着已删成员）和缺口 4（重交次数口径偏大）是这两批新带出来的 | 第 3 条删了 `TablesLargerThanABindingOnTheDevice`，变异表里还有两行的替换文构造它（量的，下面第二节第 3 行） |
| T2 十四项选择 | 十四项里：只是读法 11 项、条款已跟上（用户定过、已写进决策文件）2 项、与条款相左 1 项（窄） | 第 55 批第 2 项后半：攒批会话排空待发没发成的那一次，条款没豁免它，代码既不报警也不计叫起次数 |
| T3 产生结论的方法（主攻） | 判决第二、三节判「打中」的九条（1–8、10）在第一轮代码上站得住；其中第 1 条的反例不能在「一段一次调用」与「整理的一批」之间选，第 3 条的修补只补了一半；判「没打中」的两处快写法这一轮仍只有一份样本 | 第 55 批 55 行新变异行全是替身行（量的），真卡才罩得到的 OOM 错误范围与现枚举两处没有变异行 |
| T4 从未被看过的地方 | 叫起计数只有一个加一点、四个调用点各调一次，插进来的叫起排不出（`&mut self`）；漏计一处（排空没发成）；`#[must_use]` 之外没有任何非测试调用点读报警；`GpuWorkOfTheBatch` 一批放弃时交回；现查清单的枚举超时当一张都不在与「GPU 不在」同为拒切换 | 卡清单校验一次读两遍真枚举，文档却写「只读一次」 |

## 二、判决第三节第 1–8、10 条逐条复核

判定三档：站得住 / 够不着 / 同样打中别的臂。「站得住」指在第一轮代码上打中成立；「够不着」指原反例在冻结副本今天的代码上不再成立；「同样打中别的臂」指同一个反例形状也打中另一种合条款的写法。

| 判决条 | 辩方判定 | 证据（冻结副本文件:行号 / 决策文件行号） | 什么现象会推翻它 |
|---|---|---|---|
| 1 一批里的超时按片计、交活前先等不在重交里 | 站得住（第一轮代码上）；今天够不着；改法不唯一；同样打中别的臂 | 今天外壳每段每次调用建一份按卡的计数「let mut submissions_of_each_card: Vec<SubmissionsOfACardInTheBatch> = self」（`crates/singlefs-background-gpu/src/background_gpu.rs:1783`），计数字段「pub timeouts: TimeoutsOfTheBatchOnOneGpu,」（`crates/singlefs-background-gpu/src/background_gpu.rs:391`），越过 3 次即报「if self.timeouts > RESUBMISSIONS_AFTER_A_TIMEOUT {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:175`）；交活前先等也记进同一份「let room = outcome_waited_for_again_after_timeouts(」（`crates/singlefs-background-gpu/src/background_gpu.rs:1703`）。决策文件把「一批」写成「挂着时超时重交与空闲显存不足说的「一批」，都是一段的一次调用」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）。原反例（一次调用里 4 次超时、8 次交、不报）对「一段一次调用」与「整理的一批」两种写法同样不成立，所以它只能排掉「按片计」，选哪一种是用户 2026-10-31 定的（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:629`）。新反例族「窗口与窗口之间不累计」（三段各 3 次共 9 次不报）打中今天的写法，也打中任何有限窗口的写法（整理的一批之间同理），不是这两批独有 | 在冻结副本上造一组替身脚本：同一段一次调用里第 4 次等超过上限而不报，或第 3 次就报；或一批放弃后同批另两段还被交给这张卡 |
| 2 没点名的独显缺标识也拒挂；切换那一刻现查清单 | 站得住；今天够不着；改法唯一（合条款的写法只有「点名时只判点名的」与「切换时现查」）；没点名那一臂同样打中，是登记在案的选择 | 点名时只判点名的「pub fn discrete_gpus_of_the_enumeration_under_the_named(」（`crates/singlefs-background-gpu/src/adapter_inventory.rs:126`），开卡处调它「let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(」（`crates/singlefs-background-gpu/src/background_gpu.rs:577`）；清单每次答之前现查「fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2177`）。条款「点名了就只用点名的卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）、「GPU 不在、驱动不对、点名的卡不存在」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）。没点名时缺地址的卡照旧整个拒挂，`.claude/kb/mount-parameters.md:23` 登记为「条款没写，第一版不支持」 | 没点名时一张缺地址的卡让整个挂载拒掉，被用户判成越位 |
| 3 核心层不产的成员、两侧含义不一的字段、停着表写死长度 | 站得住；今天只补了一半：成员删净但变异表留两行，停着表的洞换了位置；同样打中别的臂 | `TablesLargerThanABindingOnTheDevice` 在 `crates/` 下只剩 2 处命中，都在 `crates/mutations.tsv` 冻结副本第 5787、5788 行的替换文里（`grep -rn` 量的），这两行的替换文构造一个已不存在的成员，编不过。停着表现在是手写的「pub const EVERY_MEMBER: [BackgroundTask; 1] = [BackgroundTask::CompactionRelocation];」（`crates/singlefs-core/src/background_compute.rs:49`），编译期那一判只核表里已有的项「assert!(BackgroundTask::EVERY_MEMBER[position].position_in_every_member() == position);」（`crates/singlefs-core/src/background_compute.rs:72`），用例遍历的也是这张表「for task in BackgroundTask::EVERY_MEMBER {」（`crates/singlefs-core/src/background_compute.rs:1708`）：加成员会逼着补 `position_in_every_member` 与 `has_a_gpu_path` 两臂，不逼着补进表。旧的写死长度 1 的常量有同一个洞，任何手写的成员表都有 | 加一个成员、补两臂、不补进 `EVERY_MEMBER`，整个测试与发布构建都编过且绿（推的，没试编） |
| 4 一批里最小一片随映射变大时整理停下 | 站得住；今天够不着；改法唯一（最小一片随这一批的映射变，挂载那一刻只能按与输入无关的 972 字节判，逐批判是唯一合条款的写法）；同一反例不打中替代写法 | 逐批把装不下的卡晾下、活重新按代价分给剩下的卡「pub fn plans_of_a_batch_leaving_out_the_gpus_short_of_the_smallest_piece<Plan>(」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:530`），重分走的还是挂载时量的代价「for (position_on_this_host, entries) in participating_this_round」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:554`），不是平摊；全都晾下才报。条款：「参与的卡的上限连最小的一片都装不下，这张卡就不用」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`），「按挂载时在每张卡上量出的一次提交的主机 CPU 分活」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）。每轮多晾下一张，所以至多比参与的卡数多一轮（函数文档自述，读码核过循环结构） | 造一张卡在某一批切不出、另一张卡装得下，而这一批仍报整理停下或活丢失 |
| 5 空闲显存不到上限时照常跑、只用空闲的 | 站得住；今天在替身上够不着、真卡上没量过；「压到一半」不唯一（条款只写「缩小逐次交」），同一收敛反例对任何压低写法同样成立 | 建缓冲显存不足就压一半重切「room = room.with_the_memory_limit_at_most(bytes_held_by_the_chunk / 2);」（`crates/singlefs-background-gpu/src/background_gpu.rs:1830`），压到切不出就把这张卡晾下、活放回去重分「entries_to_hand_out.push(planned.entries);」（`crates/singlefs-background-gpu/src/background_gpu.rs:1857`）。收敛：新上限取旧上限与失败那一段字节数的一半中小者「pub fn with_the_memory_limit_at_most(self, bytes_at_most: u64) -> Self {」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:83`），失败那一段的字节数不超过切它用的上限，由三处等式守着「"这一段真建的缓冲与切批按的占用一致"」（`crates/singlefs-background-gpu/src/background_gpu.rs:1173`、`crates/singlefs-background-gpu/src/background_gpu.rs:1285`、`crates/singlefs-background-gpu/src/background_gpu.rs:1402`），所以上限每轮严格减半，到最小一片以下就停（推的，没量过）。真的显存不足由这一处接「let out_of_memory_scope = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);」（`crates/singlefs-background-gpu/src/background_gpu.rs:934`），冻结副本的变异表里没有一行以它为锚点（`awk` 量的：原文列命中 `push_error_scope`、`ErrorFilter::OutOfMemory`、`enumerate_adapters` 的行数 0）。条款：「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | 真卡上错误范围报了显存不足而 `bytes_held_by_the_chunk` 大于切它用的上限，重切原地踏步 |
| 6 泄漏之后同一进程再挂的账 | 站得住；单元层今天够不着；产品路径上验收空转（整个包没有产品调用方）；调用方持有不唯一，全局写法被规则挡 | 账类型「pub struct GpuMemoryLeakedInThisProcess {」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:656`），开卡处判装不下先扣泄漏的「pub fn gpu_left_out_under_its_memory_limit(」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:464`），余量为零或负（`saturating_sub` 后小于最小一片）走「不用」，不拒挂；一张不剩时挂载不拒、后台任务停着，与决策文件「参与的卡因此一张都不剩时挂载照样不拒，后台任务照第 4 条进出错停着」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）同。丢卡时记账「self.memory_leaked_in_this_process」（`crates/singlefs-background-gpu/src/background_gpu.rs:2166`）。没有调用方：`grep -rln 'singlefs-background-gpu'` 在 `crates/` 下的 `Cargo.toml` 里只命中它自己的一份（量的）；默认入口「pub fn open_background_gpu(」（`crates/singlefs-background-gpu/src/background_gpu.rs:540`）传一份空账，调用方若漏传同一份，条款「任何时刻都不许比它多」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）就破 | 挂载工具接上之后，同一进程两次挂载用了不同的账，第二次的上限被泄漏的那一片顶穿 |
| 7 切到一张都用不上的设置时停着晚一步 | 站得住；今天够不着；改法唯一（条款定「从下一批开始生效；一批之内不换」，所以只能在两批之间换，入口开头是两批之间）；新增入口忘了先换上是同一形状 | 后台入口「.put_the_switch_requested_from_the_next_batch_in_effect();」（`crates/singlefs-core/src/mounted_session.rs:2457`）在判停之前，判停「if compaction_stays_stopped_pausing_the_live_intent(allocator, background_compute) {」（`crates/singlefs-core/src/mounted_session.rs:2460`）；显式一趟同样（`crates/singlefs-core/src/mounted_session.rs:3281`、`crates/singlefs-core/src/mounted_session.rs:3282`）；两处调同一个函数「fn compaction_stays_stopped_pausing_the_live_intent(」（`crates/singlefs-core/src/mounted_session.rs:3170`）。可写挂载续做经「pub(crate) fn run_compaction_on<Device: BlockDevice>(」（`crates/singlefs-core/src/mounted_session.rs:3194`）进同一个一趟函数，判停是同一份。条款：「下一次整理入口与后台入口在任何写之前就报停着，删除续做也不走」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）、「从下一批开始生效；一批之内不换」（`.claude/kb/decisions/26-后台整理与放置回收.md:245`） | 造一个第三种整理入口，不先换上待生效的切换就发了写 |
| 8 卸载泄漏看不出、重交与片数与用到的卡看不出 | 站得住；今天够不着；有一处记账口径偏大（重交次数） | 卸载交回泄漏的那几张「impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2151`）；记账字段「pub struct GpuWorkOfTheBatch {」（`crates/singlefs-core/src/background_compute.rs:574`）；放弃的那一批也交回「background_compute_of_each_batch.extend(background_compute_of_the_batch);」（`crates/singlefs-core/src/mounted_session.rs:3424`），出错也取一次「implementation.take_the_submissions_of_the_batch();」（`crates/singlefs-core/src/mounted_session.rs:3769`）。口径：重交次数在等前一次之前就加一「timeouts_of_the_batch.resubmissions += 1;」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:210`）、紧接着才等「attempt = wait_for_the_timed_out_submission(gpu).and_then(|()| submit_once(gpu));」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:211`），等前一次又超时时重交次数多记（推的，探针待跑，见第八节第 1 条） | 探针跑出 `resubmissions_recorded` 等于 `resubmissions_really_made` |
| 10 测试与变异缺口 | 站得住；今天够不着 | 「永远取第一块盘」红在第二块盘最满的用例（`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1247`），「永远取最后一块盘」红在第一块盘最满的老用例「fullest_device: *region_device,」（`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:941`）；锁存红在「补跑途中真删文件」那条（`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1327`）；第 6104 行点名的用例现在造非空队列「fn bring_the_deletion_queue_back_by_rolling_back_an_unlink(」（`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1209`）。同形的另一种改坏「按已分配字节比、不按占比比」只有两块盘容量不同的用例才红，我没核测试池两块盘容量是否相等（没核） | 一条「按字节比」的变异行抓不到 |

缺口（够得着的，站得住的辩方不替它掩）：
1. 第 3 条：变异表冻结副本第 5787、5788 行（名字「七·3 后台 GPU 包：实现把乙段 GPU 出错（含超时）报成第一版不支持的输入」与「……丙段……」）的替换文构造已删的成员，变异编不过，会判成无效（推的，没跑）。
2. 第 3 条：停着表的手写洞（上表）。
3. 第 5、2 条的真卡半边：错误范围那一处与现枚举那一处（`crates/singlefs-background-gpu/src/background_gpu.rs:622`）没有变异行；真卡用例 `a_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu` 写了没跑。
4. 第 8 条：重交次数口径（上表）。

## 三、T1 补充：正文点名的几个子问

- 第 1 条，等前一次超时提交那一等又超时，计一次还是两次：每一次等超过上限计一次。重交那一圈里「提交超时」和「等它做完又超时」走同一个计数「Err(cause) if waited_past_the_limit(&cause) => {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:205`），交活前先等那一圈同理（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:230`），所以先超时一次、再在等它时又超时一次是计 2 次。决策文件的口径是「累计到第 4 次等超过上限就报这张卡用不了」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`），计的是「等超过上限」，不是「重交」，所以这是条款的字面读法，不是新错；偏大的是记账里的重交次数（上表第 8 行）。
- 第 1 条，一段报卡用不了之后同一批另两段还交不交：不交。核心层三段的错一路 `?` 回去「.map_err(CompactionBatchNotCompleted::ComputeFailed)?;」（`crates/singlefs-core/src/mounted_session.rs:4046`，同形还有 4078、4118、4227 行），这一批放弃、任务停下（读码推的，没造脚本）。
- 第 4、5 条，转给别的卡的活怎么分：两处都是按挂载时量的代价分，不是平摊（上表第 4 行）。压一半重切只管这一次段调用（`room` 是那一圈的局部量，`crates/singlefs-background-gpu/src/background_gpu.rs:1830`），下一段、下一批照设置重新起，所以同一批里下一段会再撞一次同样的建不成、再压一回；这是「一批 = 一段的一次调用」（决策文件 248 行）的直接后果，不是错，代价是每段多一次失败的建缓冲（建缓冲在提交之前，所以不多交一次提交）。
- 第 6 条：余量为零或负走「不用」，不拒挂、不切不出；全不剩时挂载不拒（上表第 6 行）。
- 第 7 条：显式入口、后台入口、可写挂载续做三处的停着判是同一个函数（上表第 7 行）。显式入口里「一趟开头强制停」的测试开关在换上之前先返回，那是只供测试的分支（C921 射程），产品不走。
- 第 8、10 条：记账字段钉绝对值与取样点补得对；变异行 55 行（新三族）全是替身行，没有一行要真卡（量的，`awk` 数冻结副本 `crates/mutations.tsv` 里名字以「七·3 GPU 代码轮一：」「七·3 GPU 报警：」「七·3 GPU 空闲显存：」起头的行，第五列都不带 `tests-on-a-real-gpu`）。

## 四、T2 第 54、55 批「我替条款做的选择」十四项

判三档：只是读法 / 改了行为该交用户 / 与条款相左。「条款已跟上」指改了行为、用户已定、已写进决策文件。

| 批与项 | 对决策文件哪一句（行号） | 判 | 理由 |
|---|---|---|---|
| 54-1 一批 = 一段的一次调用 | 「挂着时超时重交与空闲显存不足说的「一批」，都是一段的一次调用」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 改了行为，已交用户，条款已跟上 | 用户 2026-10-31 定「一段一次调用」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:629`），第 56 批写回；同一行里前一句「放弃的那一批在同一张卡上重交」的「批」仍是核心层的一批，一行里两种「批」，靠后一句限定，读起来要留神 |
| 54-2 建缓冲显存不足压一半重切，压低只管这一次段调用 | 「一批（连同缓冲、暂存区）在上限里装不下，就按上限自适应切成几片、缩小逐次交，挂着时单条装不下也切」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | 只是读法 | 条款写了「缩小逐次」没写缩多少、缩多久；一半与「只管这一次」是满足它的一种取法，收敛有界（上表第 5 行） |
| 54-3 空闲连最小一片都建不成：放弃这一批、整理停下 | 「这一批不给这张卡分活，它那份转给这一批别的参与的卡，不报警」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 改了行为，已交用户，条款已跟上 | 用户 2026-10-31 定「这一批把它那份活交给别的卡」；第 55 批第 3 条改成这样，代码与条款今天一致 |
| 54-4 绑定上限装不下最小一片：照旧停下 | 条款没写绑定上限；兜底句是「挂着的时候 GPU 出错（含重交 3 次都超时），正在算的那一批放弃」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法（窄，推的） | 与显存上限装不下同形不同判：显存上限是晾下这张卡，绑定上限是停整理。最小一片 972 字节，绑定上限在真卡上远大于它，所以该分支在真硬件上走不到（推的，没量过）；若要同判，是改行为，交用户 |
| 54-5 泄漏的账由调用方持有，产品里没有调用方 | 「任何时刻都不许比它多」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）；「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法 | 主 agent 已收（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:625`）。附一条：默认入口传空账（上表第 6 行），漏传即破「任何时刻」 |
| 54-6 现查清单，枚举不成当一张都不在 | 「GPU 不在、驱动不对、点名的卡不存在」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法 | 拒切换是同一个动作；枚举等超过上限更像「驱动不对」，标成「GPU 不在」是标签粗，不改变拒绝；切到 `cpu` 不读清单（`crates/singlefs-core/src/background_compute.rs:898`），所以不会因枚举卡住出不来 |
| 54-7 卡标识用 `Vec<u8>`，记账不再 `Copy` | 无对应句 | 只是读法 | 类型形状，不改行为 |
| 55-1 停了多久数叫起次数，出错那一趟报第 1 次；挂载就停着的头一次叫起报第 1 次 | 「停下以来叫起了几次（`wakes_while_stopped`，停了多久数叫起的次数，不读挂钟）」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法 | 用户 2026-10-30 定「这一趟里出错停下的也报」（`crates/singlefs-core/src/mounted_session.rs:2365`）；加一的点只有一个（`crates/singlefs-core/src/background_compute.rs:1430`） |
| 55-2 入口在任何读写之前就拒、排空待发没发成的那一次，交回 `Err`，不带报警、不计叫起 | 「显式的整理入口每一趟也算（入口在任何读写之前就拒的那一次不算）」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 前半是条款字面；后半与条款相左（窄） | 排空是走发布路径的一次尝试，发生在判过起跑之后，不是入口「在任何读写之前」的那一拒（推的，没造脚本）；那一次是叫起，条款没豁免，代码在这里提前返回「BackgroundCompactionAfterThePendingChangesRefused::PublishOfThePendingChangesRefused(」（`crates/singlefs-core/src/mounted_session.rs:5599`），既没报警也没加一（取报警在更后面「let alarm = self.session.alarm_of_the_compaction_at_the_end_of_a_wake();」，`crates/singlefs-core/src/mounted_session.rs:5615`），之后的 `wakes_while_stopped` 少一次。可达：停着、占用超阈值（补跑）、待发排空被拒。补法两种：`Err` 里带报警，或这一支补计一次；或在条款括号里补上这一例，交用户 |
| 55-3 可写挂载续做那一趟不算叫起，报警字段恒 `None` | 「显式的整理入口每一趟也算」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法 | 续做不是显式入口：它由挂载起（`crates/singlefs-core/src/mount.rs:7662`），字段在一趟收尾处定 `None`（`crates/singlefs-core/src/mounted_session.rs:3261`） |
| 55-4 `#[must_use]` 只到整份丢掉才警告，接住不读不管 | 「通道是返回值里必须处理（`#[must_use]`）的报警，不是日志，调用方不处理编译就警告」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法（条款点名的机制就是 `#[must_use]`），缺口交用户知道 | 非测试调用点只有两处工具：`let run_result = mounted_session.run_compaction(devices);`（`crates/singlefs-harness/src/history.rs:7048`）与 `let run = session.run_compaction(devices);`（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:3479`），都接住了、没读报警字段，编译不警告（量的：`crates/` 下 `tests/` 目录之外的 `.rs` 里，出现该字段名的只有 `mounted_session.rs` 自己）；它们的会话不开 GPU，不会停着 |
| 55-5 报警点名的卡取自停下的那个错 | 「与它点名的卡（`gpu_devices_named_by_the_error`）」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`） | 只是读法 | 条款就写「它点名的卡」；参与的卡一张不剩时列被晾下的每一张是对「点名」的取法 |
| 55-6 一张因空闲显存不用、剩下的又因显存上限不用而一张不剩：报空闲显存那个成员，两份清单都带 | 「参与的卡这一批全都装不下才报」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）；「参与的卡因此一张都不剩时……后台任务照第 4 条进出错停着」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | 只是读法 | 混合态条款没写；两份清单都带，不丢信息；乙、丙两段的外壳映射臂「fn region_record_batch_failed_of(」（`crates/singlefs-background-gpu/src/background_gpu.rs:476`）与「fn live_slot_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> LiveSlotBatchFailed {」（`crates/singlefs-background-gpu/src/background_gpu.rs:505`）没有用例也没有变异行（实现员自报） |
| 55-7 核心层那一半的 harness 用例经假实现注入成员 | 无对应句 | 只是读法 | 测试布置；外壳与映射两层另有用例钉 |

## 五、T3 产生结论的方法（主攻）

1. 判「打中」的九条（第三节第 1–8、10 条）的方法是：腿给反例，主 agent 读码、复跑攻方的模型再下判（判决第一节）。第一轮的冻结副本已删，我没法在第一轮代码上重放；「站得住」这一档因此靠判决第一节里主 agent 复跑出的读数（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 没有重复它们，读数在判决第一节「攻方的复跑数」一行），不是我量的。我量的是另一半：每一条的原反例在今天的代码上还成不成立（第二节表「今天够不着」那一档），成立的缺口列在第二节表后。
2. 正文的例题——第 1 条「超时按片计」打中，那么「一段一次调用」是不是同样被它的反例打中：不是。原反例是单次方法调用里 8 次交、4 次超时仍不报，它就发生在一次调用之内，所以只打中「按片计」，对「一段一次调用」与「整理的一批」都不成立（`crates/singlefs-background-gpu/src/background_gpu.rs:1783`、`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:175`）。能打中「一段一次调用」的是另一个形状：窗口与窗口之间不累计。这个形状对「整理的一批」同样成立（一批之间不累计），所以它不能用来在两种读法之间选，选择只能由用户定，用户已定（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:629`）。
3. 同一个方法问题在第 3 条上更尖：判决的验收写「加成员不改它就编不过，或有一条对每个成员断言的用例」。今天的写法加成员会逼着补两臂，不逼着补进表，用例遍历的是表，所以「或」后面那半没满足；「编不过」那半只满足了一半（上表第 3 行）。这一条的方法缺口是验收句里的「它」指哪一样没写死：指枚举，就满足了一半；指表，就没满足。
4. 判「没打中」的几处：第一轮 T2 的三种形状、重交 3 次之内卡回来、超时提交回来之后账算错，都是攻方一份样本的「造不出」。这两批改了那一带（新增成员 `EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`、重分循环），所以旧的「没打中」不能带到今天的代码上；我自己没造反例，只读了重分与重切两圈的有界性（上表第 4、5 行）。
5. 这两批的测试与变异行判的是不是那条验收：
   - 替身罩得到：第 1 条的计数与三段接线（冻结副本变异表第 6328–6336 行）、第 4、5 条的晾下与重分（第 6341–6346 行、6373–6382 行）、第 6 条的账（第 6347–6350 行）、第 7 条两个入口（第 6357、6358 行）、第 8 条的泄漏与记账（第 6337–6339、6354、6359 行）、第 10 条（第 6360、6361 行）、第 55 批报警十二行（第 6362–6372 行、6381 行）。三族共 55 行，第五列都不带真卡特性（量的）。
   - 只有真卡才罩得到、今天没有变异行的：显存不足错误范围那一处（`crates/singlefs-background-gpu/src/background_gpu.rs:934`）；现枚举那一处（`crates/singlefs-background-gpu/src/background_gpu.rs:622`），真卡用例只断言「卡清单照旧是本机全部独显：不重开」（`crates/singlefs-background-gpu/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs:252`），即现查清单等于开卡时清单，一张卡都没掉的主机上这能过，罩不到「掉了的卡没被筛掉」。
   - 要真卡的 6 行（冻结副本变异表第 5799、5806、5864、6010、6012、6013 行）与第 55 批真卡用例：没跑（背景材料第二节自报）。
   - 对不上验收的：实现员自报乙、丙两段外壳的空闲显存映射臂（`crates/singlefs-background-gpu/src/background_gpu.rs:476`、`crates/singlefs-background-gpu/src/background_gpu.rs:505`）没有用例也没有变异行；`bytes_of_the_last_chunk_not_created` 没有变异行；变异表第 6382 行（名字「七·3 GPU 空闲显存：这一批的卡都因空闲显存不用时不列出是哪几张」）实现员自报没证红。

## 六、T4 从未被看过的地方

1. 叫起计数在拆成「判起跑」「跑」两段之后：加一只有一个点「wakes.wakes_while_stopped += 1;」（`crates/singlefs-core/src/background_compute.rs:1430`），四个调用点各在一次叫起收尾调一次：显式入口（`crates/singlefs-core/src/mounted_session.rs:2366`）、后台入口（`crates/singlefs-core/src/mounted_session.rs:2418`）、攒批会话没起跑（`crates/singlefs-core/src/mounted_session.rs:5569`）、攒批会话起跑了（`crates/singlefs-core/src/mounted_session.rs:5615`）。四处都在 `&mut self` 的方法里，一次叫起从判到报是一整段，别的叫起插不进来；两个后台入口是两个函数，不会同一次叫起读两遍。重计：没有。漏计：一处，排空待发没发成的提前返回（上表 55-2），在 5599 行，没经过任何一个调用点。换设置清零「self.wakes_of_the_tasks_stopped.clear();」（`crates/singlefs-core/src/background_compute.rs:1408`），所以切换之后重新从 1 数。
2. `#[must_use]` 加到三个结果类型之后产品里每个调用点读没读报警：`crates/` 下没有产品挂载工具（`crates/singlefs-background-gpu` 以外没有 `Cargo.toml` 依赖它，量的），两处工具调用点接住了不读（上表 55-4）。第 55 批报告说当场抓到四处旧用例丢整份结果，那四处都在 `tests/` 里；`#[must_use]` 抓得到「整份丢掉」，抓不到「接住不读」，所以「调用方不处理编译就警告」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）在字面上只成立一半。
3. `GpuWorkOfTheBatch` 不再 `Copy` 之后，一批放弃时的记账：交回，不丢。出错那一批也取一次记账（`crates/singlefs-core/src/mounted_session.rs:3769`），再收进这一趟的各批记账（`crates/singlefs-core/src/mounted_session.rs:3424`）。
4. 现查清单枚举等超过开卡时的上限：外壳交回空表「return Vec::new();」（`crates/singlefs-background-gpu/src/background_gpu.rs:630`），一张都不在，切换照「GPU 不在」拒，与决策文件第 248 行的拒切换是同一个动作；选 `cpu` 的切换不读清单（`crates/singlefs-core/src/background_compute.rs:898`），所以不会被卡住出不来。新发现：校验一次读两遍清单，两次都是真枚举，「devices_with_total_memory: gpu_inventory.devices_with_total_memory(),」（`crates/singlefs-core/src/background_compute.rs:929`）与「devices_not_opened_at_the_mount: gpu_inventory.devices_not_opened_at_the_mount(),」（`crates/singlefs-core/src/background_compute.rs:930`）在外壳里各自枚举一次（`crates/singlefs-background-gpu/src/background_gpu.rs:2177` 起两处），而它们上面的文档写「卡清单只读一次」。后果：两次读之间掉一张卡会让两份答案不一致（等同于校验后马上掉卡，下一批报 GPU 出错，无害），切换请求最坏等两个开卡等的上限（推的，没量过）。

## 七、判决第四节：只抽一次样的两处

- `region_scan.rs` 与 `checksum.rs` 两处快写法，这一轮：还是只有一份。依据：这一轮六格（T1–T4）与改动全文都不含它们；冻结副本里这两份文件与六份着色器的 sha256 与第一轮开工快照里的逐个相同（`research/prompts/m4-gpugc-code-r1-snapshot/sha256sums.txt` 对 `crates/singlefs-core/src/region_scan.rs`、`crates/singlefs-core/src/checksum.rs` 两行，量的），没有改动就没有新的攻击面。
- 我顺带读了两处（只读码，不算攻方样本）：`checksum.rs` 第 1–200 行的选路与软件路、`region_scan.rs` 第 99–215 行的按身份索引（建索引时同一条条目在一个落点只记一次，「if positions.last() != Some(&entry_position) {」，`crates/singlefs-core/src/region_scan.rs:156`，条目按表的次序走所以相邻比较够用），没看出问题。没读三路交错合并、硬件指令路、索引的查询侧以外的扫描主体，也没读着色器。按「一条腿只抽一次样不算一次观测」，这算不上第二份样本：要第二份，得派攻方腿对这两处造反例。
- 补一条量的：`grep -c 'region_scan.rs\|singlefs-core/src/checksum.rs' research/prompts/_m4-gpugc-code-r2-diff.md` 输出 0，改动全文里这两份文件一次都没出现。

## 八、要主 agent 跑的命令

两条，都不是重型（第一条是 `singlefs-background-gpu` 一个测试目标，第二条是点名两行变异）。读数在主 agent 跑出之前一律当「推的」。

1. 重交记账探针（量第二节第 8 行的口径，同时数第二节第 3 行与第 6 行里的两个静态事实）。
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-sonnet/rerun-1 --model /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r2-sonnet-model`（`cargo test` 那一行在脚本里已带范围声明）
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里三行 `PROBE name=…` 与两行 `STATIC name=…`，与 `research/prompts/m4-gpugc-code-r2-sonnet-model/out/predicted.txt` 逐行比。预测（推的）：第一行 `resubmissions_recorded=2`、`resubmissions_really_made=1`，第二行 `resubmissions_recorded=3`、`resubmissions_really_made=0` 且 `outcome_is_ok=false`，第三行两者都是 3；`removed_member_hits count=2`，`cargo_toml_files_naming_the_gpu_package count=1`。第一、二行两个数相等，我的第 8 行口径缺口就不成立；`removed_member_hits` 不是 2，第 3 行缺口 1 就要改。
   - 跑完：`task-server/task clean <号>`；`/home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-sonnet/rerun-1` 里的拷贝树用完删。
2. 变异表两行替换文构造已删成员，判它们是「无效」还是「抓到」（缺口 1）。
   - 先备一份带 `scripts/` 的拷贝：`rsync -a --exclude target --exclude research --exclude records --exclude briefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2/repo/ /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-sonnet/prove-copy/`（不拷 `research/`、`records/`、`briefs/`、`target`；够不够用以 `prove-red.sh` 自己的前置检查为准）
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-sonnet/prove-copy -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r2-sonnet/prove-copy singlefs-background-gpu '七·3 后台 GPU 包：实现把乙段 GPU 出错（含超时）报成第一版不支持的输入' '七·3 后台 GPU 包：实现把丙段 GPU 出错（含超时）报成第一版不支持的输入'`
   - 看哪几行：末尾汇总行「点名 2 条：跑了 2 条，抓到 N 条…」；N 为 2 表示我的缺口 1 不成立；有「无效」或编不过的字样，缺口 1 成立（推的预测：两条都无效）。
   - 跑完：`task-server/task clean <号>`，拷贝 `prove-copy/` 用完删。

## 九、没做什么

- 没起 cargo、没起 GPU、没往统一任务服务交任何任务；上面两条命令没人跑过，读数全是推的。
- 没在第一轮代码上重放第一轮的反例（冻结副本已删），第二节「站得住」那一档靠判决第一节主 agent 复跑的读数，不是我量的。
- 没造停着表的编译探针（加成员不补表）；缺口 2 是读码推的。
- 没核第 10 条里「按字节比、不按占比比」那种改坏能不能被测试池罩住（没核两块盘容量是否相等）。
- 没读第一轮攻方腿与本轮攻方腿的报告（禁读清单），所以 T1、T4 里与攻方可能重叠的发现（叫起漏计、`#[must_use]` 缺口、读两遍清单）是我读码独立得到的，是否重复由主 agent 对。
- 没跑 `rerun.sh --selftest`：我只改了 `rerun_steps` 一段（`cargo test` 与静态检查），模板自带的自证第一格会因假仓里没有 `crates/` 而红，这不是脚本坏了；没有为此改模板。
- 没读第 56 批写回的三语译文，只对中文那份决策文件。
- 没改冻结副本、没改 `crates/` 与 `.claude/kb/`；写过的只有本报告、模型目录 `research/prompts/m4-gpugc-code-r2-sonnet-model/` 与认领的草稿目录（里面只有认领的 `.owner` 与 `.writer-lock`，没有仓副本与编译目录，`du -sh` 12K）。

## 十、推翻条件汇总

- 第二节：任一条的原反例在冻结副本上重新造得出（见各行末列）。
- 缺口 1：第八节第 2 条命令里 `prove-red.sh` 判那两行「抓到」。
- 缺口 4（重交次数口径）：第八节第 1 条命令里第一、二行两个数相等。
- T4 漏计：造出一次攒批会话的叫起，排空被拒后下一次报警里的 `wakes_while_stopped` 仍是连续的（即漏计不成立）。
