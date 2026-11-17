# c996-code-r2 辩方腿报告（2026-11-12）

轮名 c996-code-r2，复核对象 `research/prompts/c996-code-r1-main-verification.md`。本腿没写模型、没起任何任务；下面凡标「推的」的读数都是读码推出来的，待主 agent 跑。
开工核过：主工作区那十份产品文件此刻的 sha256 与 `research/prompts/c996-code-r2-snapshot/crates-sha256.txt` 逐份相同（十份都对上），所以下面的主工作区行号就是被判的那一版。
kb 条款全在 `.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13：第 1 条第 250 行，第 2 条第 251 行，第 3 条第 252 行，第 4 条第 253 行（一整行）。
记号：判得住＝第一轮这一格的判，引的读数与条款句子撑得住；够不着＝条款或读数够不到那一格写的话；推不出＝撑不撑得住推不出。

## 一、第一轮判决第二节每一行

读法说明：原实现指修补之前的代码（只在补丁的减号行里，主工作区已没有，所以只写转述、不给行号）。

| 号 | 辩方判定 | 依据与替原实现找的理由 |
|---|---|---|
| 甲 | 判得住 | 条款：`.claude/kb/decisions/26-后台整理与放置回收.md:253`：「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」，与同行「重开期间照旧用切换前那组卡，开成了，下一批起换成新设置」连读，生效在开成之后。原实现把「交给实现」当「生效」（转述）。替它找到的理由：第 1 条（第 250 行）只写「从下一批开始生效」，单看它交给实现那一刻就是生效；但第 4 条有专讲点到没开的卡的句子，专句压通句，这条理由站不住。读数是核心层替身量的（判决已写明），这一格只靠核心层的出错处理，与外壳无关，所以没有把替身读数读成真外壳的。 |
| 乙 | 判得住 | 条款：`.claude/kb/decisions/26-后台整理与放置回收.md:253`：「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下，从下一批起用新设置」。原实现在交设置那一刻读实现答的「用不上」，而实现这时答的是切换之前那一组（转述，原方法文档自己写的是最近交来的那一组，文档与实现对不上）。真外壳那一半有任务 14158 的读数（判决第二节乙行）。找不到站得住的理由。 |
| 丙 | 判得住 | 条款：`.claude/kb/decisions/26-后台整理与放置回收.md:253`：「报的是挂载那一刻同一个成员（`GpuUnusableUnderTheSettingsHandedIn`）」，第 3 条 `.claude/kb/decisions/26-后台整理与放置回收.md:252`：「参与的卡因此一张都不剩时挂载照样不拒，后台任务照第 4 条进出错停着」。原实现只在点的卡都开着时停（判决的对照臂 card_already_open），点到没开的卡换上之后照开一批，条款没给这一支豁免。理由找不到。真外壳读数在判决里（GPU 包两条），不是只靠替身。 |
| 丁 | 判得住，带一处条款没写 | 条款 `.claude/kb/decisions/26-后台整理与放置回收.md:253`：「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」。原实现按设置内容比，同一组设置再交一次就拿到上一次的失败、零次重开，等于把「人再来一次」当成同一件事。站得住的一半：条款没写「至多 4 次」是每次交设置各数还是整个挂载累计（同行「头一次开卡不算重开，一张卡至多开 4 次、报 4 个原因」没写范围）。判决改法 2 取每次交设置各数（修补后一共开 8 次、报 4 个原因）；若取累计，这 8 次本身就越了「至多 4 次」。这是条款没写、判决选了一种读法，应写回 kb 时写明。 |
| 戊①切到 cpu | 判得住 | 第 251 行现行文本已有「从 gpu 切到 cpu 算全部卡都不再参与，切换生效时关掉全部开着的卡、丢掉还在开的那一组，之后切回 gpu 当成点到没开的卡，照第 4 条的重开规矩开，开卡期间在 CPU 上算」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`），来自用户弹窗定案（判决「交用户的」第一条）。写回之前第 251 行只有「切换之后不再参与的卡，切换生效时关掉」，没写 cpu 算不算全部不参与，原实现的读法（cpu 不动卡）当时站得住；用户定了之后没有理由了。 |
| 戊②被盖掉的切换开成的卡 | 推不出 | 条款没有一句写这张卡该关该留：第 251 行「挂载时只开参与的卡（点名了是点名的那几张，没点名是本机全部独立显卡），不参与的卡不开」说的是挂载；「切换之后不再参与的卡，切换生效时关掉」要等切换生效，而这一组从没生效。判决把它判成「打中」，实际是读法（开着而没人用的卡是泄漏的资源）。替原实现找的理由：留着开着的卡，下一次点到它时省一次开卡；条款没禁。读法站得住，字面违反推不出。 |
| 戊③切换关卡泄漏 | 判得住（判决本来就判成读法，不算打中） | 理由在代码里：`crates/singlefs-background-gpu/src/background_gpu.rs:3439`：「vkQueueWaitIdle」（Drop 的文档，析构要等提交且不带上限），同一个理由管切换关卡；条款第 253 行：「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」。wgpu 那一句是代码注释里的说法，本轮没在本项目验证。 |
| 己 | 判得住（随戊①消掉） | 切到 cpu 关掉全部卡之后切回，卡清单里那组卡没开，走重开；核心层生效的仍是 cpu（见第四节读码）。 |
| 庚 | 判得住（靠代码内部一致，条款句是写回后才有的） | 同一份代码里挂载那一刻与每次开卡内层起不了线程是进重开循环的：`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:301`：「起不了开卡的那条线程：{error}」报成这一次没开成，由 `gpu_opened_with_the_reopenings` 重开；原实现切换开卡线程起不了却只报一个原因、不重开，两处口径不同。第 253 行「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开」是主 agent 照用户的重开规矩推后写回的，用户没单独定（判决「交用户的」第二段），可以推翻。 |
| 辛 | 推不出，后果判得住 | 没跑。后果链读得到：开卡线程 panic 在两批之间那一问里原样再 panic，那一问在会话锁里，锁中毒之后 `crates/singlefs-mount-runtime/src/lib.rs:461`：「持着会话锁的线程 panic 过」，整个运行库记成坏了。替原实现找的理由：它是有意让 panic 响着传出去（原 `# Panics` 文档写明），挂载那一刻的开卡同样不接 panic（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:311` 的 resume_unwind），吞掉会把本包自己的不变量断言一起吞。这是取舍，条款句是写回后才有的，所以字面违反推不出，按打中办是稳妥的。 |
| 壬 | 判得住（读法；条款句是写回后才有的） | 开卡器那道闸管的是枚举（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` 文件头），不管切换开卡线程与重开之前没回来的那一次；卸载后开卡线程没有叫停（判决写的，转述）。没跑，读码。找不到站得住的理由，但「卸载时还在开的切换开卡取消，不再接着开」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）与「同一张卡前一次开卡没回来就不再起下一次，算这一次又没开成，照间隔往下数」（同行）都是主 agent 推的写回，用户没单独定。 |
| 癸 | 判得住（轻），判决的措辞过头 | 判决写「运行时看不出」，字面不对：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:791`：「pub fn bytes_leaked_on」是公开读法，持有进程泄漏账的调用方随时读得到每张卡泄漏的字节。真正缺的是：看不出是切换关卡泄漏的、是第几次切换、泄漏了哪几张（账按卡累加，卸载与切换混在一起）。依据 `.claude/rules/fs-design.md:127`：「分支必须可观测：运行时能看出走了哪一条。」撑得住「分不出走了哪一条」，撑不住「看不出」。 |
| 己、子、设计问题 ①②⑧ | 己见上；子是用户已定；①②⑧ 本腿没复核 | 派发没点这几行。 |

## 二、第三节九条改法：做的是不是判决写的那一句，有没有多做少做

对着主工作区此刻的代码与 diff 文件核。行号都是主工作区的。

| 条 | 判定 | 证据 |
|---|---|---|
| 1 甲乙丙：三种答、核心层第三格 | 做了；多做三处，少做零处 | 三种答：`crates/singlefs-core/src/background_compute.rs:424`：「pub enum LatestGpuSettingsHandedIn」；核心层第三格 `crates/singlefs-core/src/background_compute.rs:1332`：「handed_in_while_the_implementation_opens_its_cards: Option<AdmittedBackgroundComputeSettings>」；出错解除 `crates/singlefs-core/src/background_compute.rs:1413`：「if self.requested_from_the_next_batch.is_some()」连 1414-1416 的第三格一起判；换上了时照挂载那一刻的停法重判，在 1686-1695。多做：①补跑阈值随请求走、不等开卡（`crates/singlefs-core/src/background_compute.rs:1528`：「.or(self」，判决没写，条款第 253 行「挂着随切换请求一起换、从下一批起生效」撑得住「一起换」，但「不等开卡」是实现员自己读的）；②没有还没生效的那一组、生效的选 gpu 时，每批开头也问实现一次、答案丢掉（`crates/singlefs-core/src/background_compute.rs:1677`：「let _answer_with_no_settings_waiting = self」，判决没写，为的是让实现收下为更早几次交设置开完的卡，问这一下有副作用）；③挂载那一刻 `of_a_mount` 也走交再问，挂载组一开始就记成生效（1353 起），实现若答还在开卡，核心层仍把它当生效的（只有挂载带来的实现与设置不是同一次开出的才走到，产品运行库挂载走同一份设置，`crates/singlefs-mount-runtime/src/lib.rs:815`，走不到）。 |
| 2 丁：开卡结局按次序号认 | 做了；少做一处 | 号是私有类型 `crates/singlefs-background-gpu/src/background_gpu.rs:366`：「struct HandingInNumber(u64);」，比较在 `crates/singlefs-background-gpu/src/background_gpu.rs:2601`：「if settings_waiting.handing_in_number == handing_in_number {」。少做：起不了线程的那一支存的是更早一次交设置的卡单（`crates/singlefs-background-gpu/src/background_gpu.rs:2735` 一带的 EndedWithoutAThread 与 ThreadNotStarted 带着交设置那一刻的 discrete_gpus_to_open），后来又交了一组时，间隔之后再起开的还是更早那一组的卡，最近那一组的卡要等这一次结局收下之后才开（读码推的；只在起不了线程时才有，极少）。 |
| 3 戊①：切到 cpu 关全部卡 | 做了；多做一处 | `crates/singlefs-background-gpu/src/background_gpu.rs:2471`：「self.close_the_cards(|_device| true);」前面叫停、收下已送回的结局、清掉在等的那一组与开不成。多做：选 cpu 而带着实现时，挂载那一刻也告诉实现一张卡都不参与（`crates/singlefs-core/src/background_compute.rs:856`：「(None, Some(gpu_compute)) => gpu_compute.lock().use_no_gpu_from_the_next_batch(),」，`of_a_mount` 经交设置走到它）；第 251 行没写挂载那一刻，产品运行库选 cpu 时不递实现（`crates/singlefs-mount-runtime/src/lib.rs:815` 那一支只在 gpu 时开，cpu 是 None），所以只有测试与 e202 路径走到。 |
| 4 戊②：最近那一组开不成时关掉生效那一组之外的全部卡 | 做了；多做一处 | `crates/singlefs-background-gpu/src/background_gpu.rs:2626`：「fn close_the_cards_outside_the_settings_in_effect(&mut self) {」。多做：没有在等的那一组时开卡才回来，也照同一个口径关（判决只写了开不成那一支）；有变异行罩着（「C996 没有在等的那一组时开卡开成的卡不关」）。 |
| 5 庚 | 做了；少做的见条 2 | 重开次数扣掉起不了线程的几次：`crates/singlefs-background-gpu/src/background_gpu.rs:1282`：「.saturating_sub(u32::try_from(causes_before_this_card.len()).unwrap_or(u32::MAX)),」，合起来至多 1 加重开次数次，用例 the_opening_thread_never_started_is_reported_with_one_cause_for_each_start 钉 4 个原因。重开间隔靠两批之间那一问到点再起，没人问就不起（会话里每批开头与整理两个入口都问，所以不是无限等）。 |
| 6 辛：开卡线程里接住 panic | 做了，只做在切换开卡这一边；吞得比判决写的宽 | 接住在 `crates/singlefs-background-gpu/src/background_gpu.rs:1238`：「std::panic::catch_unwind(std::panic::AssertUnwindSafe(open))」，包住的是整个开一次卡的那一步（含本包自己的代码，例如 `crates/singlefs-background-gpu/src/background_gpu.rs:1121`：「adapters[discrete_gpu.position_in_the_enumeration].clone()」的下标），所以本包自己的不变量 panic 也被记成「开卡 panic 了：…」这一种原因（原话进原因，不是悄悄吞掉）。挂载那一刻的开卡不接：panic 经 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:311` 往上传，挂载调用线程 panic。第 253 行「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开」字面管挂载与切换两边，判决与修补只做了切换这一边，挂载那一边是否也要，判决没写，推不出。 |
| 7 壬：前一次没回来不再起；叫停 | 做了；少做一处、叫停说得比做的多一点 | 闸在开一张卡那一步里：`crates/singlefs-background-gpu/src/background_gpu.rs:1192`：「struct OpeningOfTheDeviceNotReturned {」，丢它时划掉：`crates/singlefs-background-gpu/src/background_gpu.rs:1222`：「.retain(|device| *device != self.device);」；挂载与切换共用同一个开卡闭包（`crates/singlefs-background-gpu/src/background_gpu.rs:1068`：「let mut open_a_gpu = opening_of_a_gpu_in_use(adapters, memory_leaked_in_this_process.clone());」，1096 行交给外壳），所以同一次挂载内一张卡上同时在开的至多一条。叫停：卸载 3443 行、切到 cpu 2457 行置位；每次开卡之前看（1292 行），睡之前看（`crates/singlefs-background-gpu/src/background_gpu.rs:1287`：「if !called_off.load(Ordering::SeqCst) {」）。叫停时已经睡着的那一觉睡不断，开卡线程这一觉醒来才看到信号；它握着开卡那一步的锁，之后切回 gpu 起的新开卡线程要等它放锁：睡着的那一觉最多一个间隔（产品 1 分钟），正在开的那一次最多一次开卡的上限（产品 5 分钟，读码推的）；这期间核心层生效的是 cpu，照 CPU 算，不出错。文档说「不再睡重开的间隔」，对没开始睡的才真。 |
| 8 癸与戊③：切换关卡泄漏报出来 | 做了字面；产品路径读不到 | 新类型 `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:741`：「pub struct GpusLeakedWhenClosedAtASwitch」，读法 756 行，`close_the_cards` 里累计。但产品开卡器把开成的结局拆开时只留把手与一段报告字符串：`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:184`：「gpu_compute: opened.gpu_compute,」，运行库只收这两样（`crates/singlefs-mount-runtime/src/lib.rs:815`：「Ok(opened) => Some(opened.gpu_compute),」），新累计的那一份在产品路径上没有任何读者（全仓除新用例外没有地方读 `gpus_leaked_when_closed_at_a_switch`，e202 也只读卸载那一份）。卸载那一份原来就是这样。条款第 253 行写「并报出来」没写报给谁，字面做到，满足「分支必须可观测」要产品路径有读者，没做到。 |
| 9 八处变异 | 7 处补了，1 处如实写测不到；新分支还有 6 处没罩着 | 41 个名字与 `crates/mutations.tsv` 第一列逐字相同（我用 awk 按第一列比：匹配 41 行、缺 0、名字无重复）。没有变异行罩着的新分支：①叫停之后睡之前那一判（背景 1287 行；用例的重开间隔是 0，睡不睡看不出）；②起不了线程那一支「还没到间隔就不起、UntilItEnds 睡到间隔」（2502、2516 行；用例间隔 0，走不到）；③重开次数扣起不了线程的几次（1282 行；用例只在全起不了时钉 4 个原因，起一次不了再起成功之后的上限没钉）；④核心层补跑阈值 `.or(self`（1528 行，只有请求那一支有旧变异行）；⑤核心层交 cpu 一组时作废还在开卡的那一组（1660 行：「self.handed_in_while_the_implementation_opens_its_cards = None;」，少了它实现那边丢了在等的那一组，核心层仍拿旧那一组去问，会走到 `batch_not_split_over_the_gpus` 的断言；没有用例在开卡期间交 cpu）；⑥闸的划掉那一步（1222 行；没有用例用真开卡闭包开成功之后再开同一张卡，少了它切回 gpu 之后那张卡永远「前一次没回来」）。 |

### 主 agent 自己推的四条（判决「交用户的」第二段），条款撑不撑得住

条款句都在第 253 行，但都是随第一轮写回加的，句子本身是主 agent 的推，所以「条款里有」不算独立依据，下面看的是依据能不能独立站住。

| 条 | 判定 | 依据 |
|---|---|---|
| 起不了开卡线程照重开 | 撑得住 | 内层起不了线程本来就进重开（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:301`），同一规矩延到外层是一致的；用户的重开规矩（第 253 行「间隔 1 分钟重开，最多重开 3 次」）没分原因。 |
| 开卡线程 panic 当一次开不成 | 撑不住一半 | 用户的规矩说的是「开不成」，panic 是不是「开不成」条款没写；有利的一面：不让会话锁中毒（`crates/singlefs-mount-runtime/src/lib.rs:461`）；不利的一面：本包自己的不变量 panic 也被当成「开不成」重开 4 次，只在原因字符串里看得出（见条 6）。且只做了切换那一边。 |
| 前一次没回来不再起；卸载取消 | 撑得住（类推） | 开卡器文件头对枚举已经是同一个理由（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` 头部：回来之前闸住新开卡）；「没有安全的杀法」决定了只能不再起、不能杀。卸载取消同理，但叫停杀不掉已睡着的一觉与正在开的那一次（条 7）。 |
| 切换关卡泄漏同卸载 | 撑得住 | 同一个物理理由：`crates/singlefs-background-gpu/src/background_gpu.rs:3439`：「vkQueueWaitIdle」（Drop 文档：析构要等提交且不带上限），在会话锁里关卡等不起；wgpu 的说法是代码注释的，没在本项目验证。 |

## 三、实现员照条款自己读的两处，与三种没开成共用一个成员

| 项 | 判定 | 依据条款句子；条款没写的照实写 |
|---|---|---|
| ③(a) 实现答开不成时核心层把那一组记成生效、GC 停着 | 条款只撑得住一半，推出来的读法 | 撑得住的：「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）说了停；「这两个参数与第 1 到 3 条那一组一起给、一起校验，挂着随切换请求一起换、从下一批起生效；不给阈值的切换沿用原阈值」（同行）支持阈值读的是新那一组。条款没写的：开不成之后「生效的是哪一组」。同行「重开期间照旧用切换前那组卡，开成了，下一批起换成新设置」只写了开成的一支。现在的结果是核心层 `in_effect` 记新那一组（`crates/singlefs-core/src/background_compute.rs:1697`：「self.in_effect = handed_in;」），实现那边参与的仍是切换之前那一组的卡（开不成时关掉了生效那一组之外的卡，见条 4）：两边对「生效的是哪一组」认得不一样。后果推：停着的后台任务不走 GPU 路径（补跑走 CPU），读 `in_effect` 的只有测试，所以没有读者因此走错；放开停着的只有新切换，新切换交实现之后两边重新对齐。推翻它的现象：找到一条不经新切换就放开停着的路径，或产品里有读者拿 `in_effect` 当实现此刻参与的卡。我没找到。 |
| ③(b) 叫停之后开卡线程不再睡重开的间隔，正在开的那一次照旧开完，开成的卡在那条线程上照常丢 | 条款撑得住「不再接着开」，撑不住「不再睡」与「丢的账」，后两样条款没写 | 撑得住的：「卸载时还在开的切换开卡取消，不再接着开」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`），第 251 行「丢掉还在开的那一组」。正在开的那一次停不下来的依据在开卡器文件头：工作线程挂在驱动里「没有安全的杀法」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` 头部，转述）。开成的卡在线程上丢：`crates/singlefs-background-gpu/src/background_gpu.rs:2703`：「let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);」。账：这张卡没做过提交（量代价的提交都做完才返回），正常析构，没有要记的泄漏；开卡途中量代价出错而提交还在跑的卡泄漏记进进程账是线程里已有的另一条路。条款没写「不再睡」：已经睡着的那一觉不会被叫醒（见条 7）。另一处：叫停时结局恰好已送回，`use_no_gpu_from_the_next_batch` 先 try_recv 取走并把开成的卡收下再关；刚好在 try_recv 空之后、收的那一端丢掉之前送到的，结局随收端在会话线程上丢（窗口极小，读码推的，丢的是已做完提交的卡，无泄漏）。 |
| ① 三种没开成共用 `DeviceOrPipelineDidNotStart`（起不了线程、panic、前一次没回来） | 成立，没有调用方要分开的决定 | 类型文档自己写明：`crates/singlefs-background-gpu/src/background_gpu.rs:147`：「调用方的决定只有一个——这一批放弃、后台任务停下并报错，成员只为报得清」。重开循环不看原因（`gpu_opened_with_the_reopenings` 对每个 Err 一视同仁），核心层映射到 `GpuFailureOnTheDevice` 带 reason 字符串，报警只报原因文本。条款第 253 行只要求「报出哪张卡、每次开不成的原因」。要分成员的理由只有一个：想让运维程序按类型过滤，条款没要。推翻它的现象：出现一个调用方按原因分支（例如 panic 不重开）。 |

## 四、第一轮十条模型用例在修补之后的代码上各走一遍（全是读码推的，待主 agent 跑）

源是 `research/prompts/c996-code-r1-opus-model/src/` 两份文件的十条用例名。我没有在修补之后的代码上跑它们，下面是逐条读码的结论。

| 第一轮读数 | 修补后读数（推的） | 实现员收成的回归用例，断言的是条款的结局还是新实现 |
|---|---|---|
| stop_outlives_the_switch，error_after_the_handing_in：stopped=true | 出错那一刻第三格有值，交回 StopLifted，之后换上答案无出错，stopped=false，卡开成之后那一批照常开 | 用例 a_batch_failing_on_the_card_before_the_switch_while_the_new_card_opens_is_lifted_by_that_switch 断言解除与换上后 in_effect 是切换那一组：条款句（第 253 行「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」）的结局。 |
| stop_outlives_the_switch，error_before_the_handing_in | 不变，StopLifted | the_same_error_before_the_handing_in_is_lifted_by_the_switch_request，对照臂。 |
| stale_stop：stopped=true | 交设置那一刻不读旧那一组，不再停；换上后 failure 为空，不停 | a_mount_stopped_under_its_settings_is_not_stopped_again_while_the_switched_card_opens_nor_once_it_is_put_in_effect；开卡期间只断言 `batch_while_opening.is_ok()`，替身此时不出错，**真实现下开卡期间这批照旧组跑，旧组一张卡都用不上，每一批在 GPU 上出错、放弃、被解除（见第五节 ③）**，这条用例没钉。 |
| left_out_after_the_opening（核心层）：照开一批 | 换上了答带出错，核心层按 `GpuUnusableUnderTheSettingsHandedIn` 停，批不开 | the_switch_put_in_effect_after_the_opening_with_every_card_left_out_stops_as_unusable_and_begins_no_batch，断言成员与挂载那一刻同一个：条款结局。 |
| left_out_after_the_opening，card_already_open | 不变 | the_switch_to_every_card_left_out_on_a_card_already_open_stops_at_the_handing_in。 |
| stale_failure，handed_in_again_before_the_poll：4 次、报 4 个 | 8 次、报最近那一次的 4 个 | the_same_settings_handed_in_again_before_the_poll_reopen_the_card_instead_of_taking_the_earlier_failure；断言 8 次依赖「每次交设置各数」的读法（条款没写，见第一节丁）。 |
| stale_failure，handed_in_again_after_the_poll | 不变，8 次 | 没有单独收成用例（旧用例照旧覆盖，推的）。 |
| superseded_card：卡开着、不参与 | 最近那一组开不成时生效那一组之外的卡关掉，开着只剩挂载那张 | the_card_opened_for_a_superseded_switch_is_closed_when_the_latest_switch_does_not_open；断言来自判决读法（第一节戊②条款没写）。 |
| left_out_after_the_opening（GPU 包）：那一问答 None | 答换上了连同一张都不剩的那个出错 | the_switch_put_in_effect_after_the_opening_with_every_card_left_out_is_answered_with_every_card_left_out。 |
| failure_of_the_settings_before_the_switch：交设置那一刻读到旧组的一张都不剩 | 那一问答还在开卡，不答旧组 | the_settings_whose_card_is_still_opening_are_answered_as_still_opening_and_as_usable_once_put_in_effect。 |

十条里翻面的七条都有回归用例；没翻面的三条（两条对照臂加 handed_in_again_after_the_poll）本来就是对的。
断言写成新实现现写的只有一处：stale_stop 开卡期间那批用的是不出错的替身，把条款没写的那一段（开卡期间照旧组跑，而旧组可能一张都用不上）绕过去了。

## 五、第二到第四节攻击面逐项（读码推的）

| 项 | 判定 | 依据 |
|---|---|---|
| ① 开卡期间连着来两次、三次切换 | 没找到两边认得不一样的交错 | 核心层每次交设置都把第三格换成最近那一组（`crates/singlefs-core/src/background_compute.rs:1659` 起的 `hand_the_settings_in`），实现每次交来记新次序号（`crates/singlefs-background-gpu/src/background_gpu.rs:2425`：「self.latest_handing_in = self.latest_handing_in.next();」）；两边只在同一处交，所以第三格的号与实现在等的那一组的号同步。开卡线程一次开完它那张清单才送结局，期间后来的切换点的卡要等它（原来就是这样，`use_the_settings_from_the_next_batch` 文档写了）。 |
| ② 开卡期间切到 cpu 再切回 | 没找到问题；核心层一侧没有用例 | cpu 一支：`crates/singlefs-core/src/background_compute.rs:1660`：「self.handed_in_while_the_implementation_opens_its_cards = None;」；实现一侧叫停并丢掉在等的那一组。切回 gpu 后新一次开卡线程起在叫停那条后面：两条线程抢同一把开卡锁，旧的叫停后只剩正在开的那一次，新的拿到锁才开（`crates/singlefs-background-gpu/src/background_gpu.rs:2680` 开卡那一步在 `Arc<Mutex>` 里）。同一张卡上不会同时两条在开（锁 + 闸）。核心层一侧没有「开卡期间交 cpu」的用例（见第二节条 9 ⑤）。 |
| ③ 开卡期间出错停下再收到切换 | 条款推得出，有一处代价没人说 | 开卡期间照旧组跑，旧组若一张卡都用不上（挂载那一组全被显存上限晾下，请求了切换），每一批都在 GPU 上出错、放弃、单发一次只删意图的发布、被解除，直到开卡结束，最长约 23 分钟（判决已报的算术）；每批一次出错一次删意图发布，没有新的落盘形态。条款第 253 行写的是这一路（「重开期间照旧用切换前那组卡」+「出错之前已经收到、还没生效的切换请求……照它解除停下」），所以不判违反，只写明。 |
| ④ 叫停与开卡结局同时到 | 没找到问题 | 见第三节 ③(b) 窗口；收端丢掉后线程上 send 失败，开成的卡在线程上丢。 |
| ⑤ 实现答换上了而核心层那一格已被新请求盖掉 | 没找到问题 | 核心层先交新请求再问，实现里新请求会顶掉在等的那一组，旧的结局到了按号不报（`crates/singlefs-background-gpu/src/background_gpu.rs:2601`），不会把旧答案当新的。 |
| ⑥ 开不成时核心层记生效，补跑阈值与 `in_effect()` 读者 | 见第三节 ③(a) | 阈值读的是 `crates/singlefs-core/src/background_compute.rs:1531`：「.unwrap_or(&self.in_effect)」那条链，开卡期间取第三格、开不成之后取记成生效的那一组，同一组。 |
| 泄漏账重复扣或漏扣 | 没找到重复；漏记一处 | 关掉的卡从开着的那一边移走，所以卸载那一份与切换那一份不会同一张卡记两次（`crates/singlefs-background-gpu/src/background_gpu.rs:2471` 之前的 `close_the_cards` 取走）。漏记：开卡途中量代价出错而提交还在跑的卡，在开卡线程上泄漏，只进进程账，不进切换那一份（它不是切换关卡泄漏，条款没写它要报）。 |
| 关掉之后 `participating` 与各卡预算的下标 | 没找到错位 | `close_the_cards` 先按标识记下参与的卡，关完照标识在剩下的卡里重找位置（`crates/singlefs-background-gpu/src/background_gpu.rs:2756` 先按标识 collect，2788 行照标识 filter_map 重找位置）；新开的卡接在末尾，旧下标不动；`use_no_gpu` 之后参与的卡为空（切回之前核心层生效的是 cpu，不问也不交活）。 |
| 停着的 GC 被悄悄放开 | 没找到 | 核心层的不变量：第三格或请求有值时没有停着的任务（请求那一刻清，出错那一刻解除），所以第三格答换上了时换掉全部停着的那几条（`crates/singlefs-core/src/background_compute.rs:1690` 起）只会换掉自己刚记的，不会放开别的。推翻它的现象：找到第三格有值而停着表非空的路径。 |
| `use_no_gpu_from_the_next_batch` 在选 cpu 挂载、从没选过 gpu 的会话上 | 没被调到 | 没带实现时 `(None, None) => {}`；产品运行库选 cpu 不递实现（`crates/singlefs-mount-runtime/src/lib.rs:815`）。 |
| 前一次开卡没回来靠什么知道，永远不回来时 | 靠开卡线程丢掉那份记录 | 记录在 1192 行的类型里，开卡的那条线程跑完（含 panic 展开）就划掉；永远不回来这张卡这一次挂载内永远开不成，每次交设置得到 4 个「没回来」原因（间隔照数，约 3 分钟），重开次数一次不少地扣；与条款第 253 行「算这一次又没开成，照间隔往下数」一致。 |
| 接住 panic 会不会吞本包自己的不变量 | 会，见第二节条 6 | 只在原因里留原话。 |

## 六、第四节检查单八格

| 格 | 判定 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立（有一条靠没罩着的行） | 断言在 `crates/singlefs-background-gpu/src/background_gpu.rs:2892`：「参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用」。调用点：2920 行的分活（只在核心层走 GPU 路径的批里调；核心层生效的选 cpu 时不走）；`latest_settings_handed_in`（2566 行，由两批之间那一问到）；e202 `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:7977`：「if let Some(failure) = gpu.failure_under_the_settings_handed_in() {」，它在 `use_the_settings_from_the_next_batch` 之后调，不在告知一张卡都不参与之后。核心层问实现只在第三格有值或生效的选 gpu 时（`crates/singlefs-core/src/background_compute.rs:1682` 与 1677 行），而选 cpu 的一组交出去那一刻就清第三格、生效的记成 cpu（1658-1660 行），所以告知之后不会问。这一条靠 1660 行那一句，那一句没有变异行罩着（第二节条 9 ⑤）。resume_unwind 在 2553 行：开一次卡的 panic 在线程上接住，断开只剩接住范围之外的本包代码 panic，成立；注意 2690 行那句 expect 的「锁没被毒化」只在接住范围之外没有 panic 时才真。推翻它的现象：核心层在告知一张卡都不参与之后、再交来一组换上之前问了实现。 |
| 类型与名字 | 成立 | 次序号是自己的类型（`crates/singlefs-background-gpu/src/background_gpu.rs:366`）；重开次数是已有的 u32 字段，原因个数是 `Vec::len()`，两者在 1282 行用 `u32::try_from` 过一道、没有共用一个裸整数；新代码 `_ =>` 零处（我把补丁加号行逐行 grep 过）；新加与搬动的 `expect` 都写了依赖的不变量（核心层 853 行、开卡线程 2690 行、卡单 2735-2738 行）。小处：核心层与外壳各有一个同形的枚举（核心层 424 行的 `LatestGpuSettingsHandedIn` 与外壳 351 行的 `LatestSettingsHandedIn`），一一映射，名字太像，不算违反。三种没开成共用一个成员：成立（第三节 ①）。 |
| 不兼容旧代码与旧格式 | 成立 | 我跑了 `grep -nE 'legacy|old_format|migrat|兼容' research/prompts/_c996-code-r2-diff.md`：零命中，对五份主要产品文件（background_gpu.rs、background_compute.rs、background_gpu_compute.rs、gpus_of_this_host.rs、gpu_opener_of_this_process.rs）同一条正则也是零命中；旧 `Option` 形状（两批之间那一问交回 `Option` 的）在全仓 grep 不到。`failure_under_the_settings_handed_in` 还留在 trait 上，它的用途是缺省答法与 e202、用例，不是转接。 |
| 不新建判定库 | 成立 | diff 里 `File::`、`fs::`、`OpenOptions`、`TcpStream`、`env::var`、`/home` 一个都没有（grep 过），只有补丁路径那一行表头。 |
| 所有任务多机可用 | 成立 | 新用例只用通道与线程，等放开用 20 秒的 recv_timeout 当保险丝，不读本机状态、不写地址与主机名；保险丝的 20 秒随机器快慢，不绑哪一台。 |
| 测试优先多线程 | 成立 | 没有 `thread::sleep` 式的写死等法，全靠通道放开；约 80 秒那一行是证红那条变异（把那一问改成等到开完）：4 次开卡各等满 20 秒保险丝才红，绿的路径一问即返回。没有该拆而没拆的参数循环。小处：20 秒的字面量在新用例里写了 7 处没有起名字。 |
| 带 --item 的命令与项 | 不成立 | 41 个名字与 `crates/mutations.tsv` 第一列逐字相同、没有重名（第二节条 9）；第一轮点名的 8 处里 7 处补了，产品挂载传 `OF_THE_PRODUCT` 与 sleep 那一处实现员写测不到（核不动，我也没找到不起真卡的测法）；这一轮新加的分支还有 6 处没罩着（第二节条 9 的 ①-⑥）。 |
| 新崩溃流多罩了什么 | 成立（推的） | 停下、解除、只删意图那一次发布没有新的落盘形态：它同 `PausedShortOfSpace` 的删意图发布（`crates/singlefs-core/src/compaction_plan.rs:632`：「R 排他跟着撤，同 [`Self::PausedShortOfSpace`]」），已登记的崩溃流是 `.claude/gate.d/stage-inputs.tsv:161`、162 两行（paused-short-of-space 的删意图）。新的只是时机：开卡期间旧组出错的批会反复放弃、反复删意图，每次是同一种发布。推翻它的现象：开卡期间连续出错的批里发出与已登记流字节形态不同的发布。 |

## 七、顺带写明的两处（不在八格里）

- `crates/singlefs-core/src/compaction_plan.rs:637`：「挂载那一刻或切换生效的那一批开头，实现照交给它的那一组一张卡都用不上而当场停着的同算」与第 642 行「一趟之中切换生效的那一批开头才判出的」的文档还写旧时机；现在点到没开的卡时是「实现答换上了那一刻」。实现员停下交主 agent 的设计问题 ②，我核实了这两行还在（文档，不影响行为）。
- 本轮补丁新加的注释里有 9 行写了「用户定案」（背景文件 `research/prompts/_c996-code-r2-diff.md` 里找 `用户定案` 能看到，不在代码里再引）。门禁 doc-text 的 comment-history 格按 `scripts/gate/comment-history-check.py:153` 那份正则判「定案」；此刻主工作区 background_gpu.rs 命中「定案」17 处、background_compute.rs 6 处，都不在 `.claude/gate.d/comment-history-lag.tsv`。我没法判门禁此刻红不红，列进下一节让主 agent 跑。

## 八、要主 agent 跑的命令

没有写模型，没有判决要引的新数。唯一一条是核上面第七节第二条的：

1. 逐行看注释里命中历史写法的行（只读，几秒）：
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 scripts/gate/comment-history-check.py --list /home/fy5090/code/singlefs`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`
   - 跑完看：`TASK_SERVER_RESULT` 那一行的 `type=` 与 `exit=`；`task-server/task log <号>` 里有没有 background_gpu.rs 与 background_compute.rs 的命中行，有就说明这两份要改净或登记滞后表（范围声明闸若要，照 `.claude/hooks/run-scope-guard.sh` 文件头补）。

## 九、没做什么

- 没撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（上一节是唯一要跑的）。
- 没写模型，没在修补后的代码上跑第一轮十条用例，也没跑实现员的 23 条新用例与变异行：第四、五节与第二节的行为读数全是读码推的，待主 agent 跑出来才算。任务 14424、14425 的结局我没看到。
- 没验证 wgpu 的 `Queue` 析构会不会不带上限地等（只引了代码注释里的说法）；没验证真驱动里 panic 与挂住。
- 没替原实现圆：戊②、辛找到了站得住的理由但推不出字面违反，其余找不到的都写了找不到。
