# c996-code-r3：攻方腿报告（Opus，2026-11-13）

立场：假设前两轮判为站住的格与两批修补是错的，造可达的历史去打穿。读的代码是冻结副本 `~/.local/share/singlefs/work/c996-code-r3/repo/`，引的 12 份文件此刻与主工作区逐份 sha256 相同（现跑 `sha256sum` 比过），所以引文按仓根路径写、行号是冻结副本的。没读禁读清单里的辩方腿报告。

## 复跑命令与模型文件

模型目录 `research/prompts/c996-code-r3-opus-model/`。三份模型用例由 `tools/apply_model.py` 插进冻结副本拷贝里三份文件的测试模块末尾（只改拷贝），`tools/rerun.sh` 在拷贝里跑两个包 lib 测试里名字带 `attack_` 的用例，打出 `name=` 读数行、逐条结局，并与 `out/predicted-statuses.txt` 逐行比。复跑命令（写在「要主 agent 跑的命令」第 1 条，这里原样）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 与 singlefs-core 的 lib 测试目标里名字带 attack_ 的攻方腿用例（模型复跑脚本）; 范围=2 个测试目标; 为什么=复跑 c996-code-r3 攻方腿模型，判决要引的读数由主 agent 复跑' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r3-attack -- bash /home/fy5090/code/singlefs/research/prompts/c996-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/c996-code-r3/repo /home/fy5090/.local/share/singlefs/work/c996-code-r3-attack-model-run /home/fy5090/code/singlefs/research/prompts/c996-code-r3-opus-model
```

| 文件 | sha256 |
|---|---|
| `src/attack_r3_background_gpu_tests.rs` | `895126863eedc66b4b501ba638a692d8f822cb1195eb456e6aeefd3fb06a416f` |
| `src/attack_r3_core_tests.rs` | `7e52d40602745787740b6e66f82d82a0de670eebce43bc66d9300f66b8c8c45d` |
| `src/attack_r3_live_slot_tests.rs` | `81e061f823909ab3f465992dca970c05ee0bcc5850b8e096e4fae5eaa0cba3f1` |
| `tools/apply_model.py` | `781e3cf0ecb441e61729fc9507a62f2b76c12b8e7400f3cafe144e86e9d7df75` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh` | `cafb8fc46a233143a48aa609cb40b4b57fabbeca3d7816db8184776ee962ca19` |
| `out/predicted-statuses.txt` | `59bdb13e084973eda5cef5415caea62a5bfb61b0a91546badc12a74e1373e6db` |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段；它在两个包都跑出「test result:」时退 0，用例红不算没跑成（有两条用例断言条款要的结局、预测就是红）。外层 `rerun-from-a-clean-work-directory.sh` 从 c996-code-r2 攻方腿模型目录原样拷（只删工作根下、名字以 `-run` 收尾的那一个运行目录）。这一趟一条都没交、没编译过：模型源码是对着冻结副本里的签名手写的，**编不编得过是推的**；第一件要看的是两个包的「test result:」行在不在。

**这一轮报的读数全是推的、没量过**：复跑由主 agent 交，读数以任务日志为准。

## 各格判定一览

| 节 | 格 | 判定 | 依据（量的 / 推的） |
|---|---|---|---|
| 一 | c996-code-r2 判为修掉的甲、乙、丙、戊 ①②、己、庚 | 站住（甲按改写后的条款站住） | 模型重跑，推的；甲的结局随新四的改法换成停着 |
| 一 | 丁 | 不再适用：被新七的定案改写，见二 | 推的 |
| 一 | 设计问题 ③(b) | 站住 | 读码 |
| 一 | bggpu G01（量代价出错的泄漏） | 站住 | 读码 |
| 一 | bggpu G03（校验和、区域扫描两段的重复键） | 站住，带一处与选段打分不一致的优先次序（见三·丙） | 读码 |
| 一 | bggpu G04（切批算术）与换类型 | 站住：没找到换反的量、panic 行为变了的溢出、改了去向的 `checked_` 失败支 | 读码；20 万例同种子重算交主 agent 跑 |
| 一 | 换了算法的那句断言 | 站住：左边只在某份缓冲内容长 0 或不是 4 的整数倍时与原来不同，产品三段造不出这种内容 | 读码 |
| 二 | 新一、新二 | 修掉了 | 模型两半重跑，推的 |
| 二 | 新三、新四、新七、壬、癸、新五、新六 | 修掉了（壬只修了「到点之后」那一种，见三·乙） | 模型重跑 / 读码，推的 |
| 二 | 甲、乙、丙、重复键只放弃这一批 | 修掉了（乙带三·丙那一处） | 模型重跑 / 读码，推的 |
| 三 | 甲：先切 `cpu`、第 4 次开卡还没回来时切回同一张卡，切换开卡线程撞 `debug_assert!`、毒化开卡那一步的锁，下一次两批之间在会话锁里 panic | **打中（重）** | 模型 12 段历史，预测 2 段 panic；推的 |
| 三 | 乙：另一次挂载正在开（没到点）的卡，这一次挂载的切换把它算成「前一次开卡没回来」、连着记满 4 次、这张卡在这一次挂载里试满 | **打中（中）** | 模型两臂，推的 |
| 三 | 丙：选段打分那一段重复键压过同一批的 GPU 出错（不停、不报警）；校验和、区域扫描两段反过来 | 打中（轻，要调用方组批先有 bug） | 读码 + 模型 3 行，推的 |
| 三 | 丁：重复键每一批都出现时 GC 一批都做不成、不停、不报警、不补跑 | 条款字面如此，零轮形态交用户 | 模型，推的 |
| 三 | 戊：开卡期间出错停着、实现再答开不成，报警的叫起次数与补跑批数从头数、报的错换成开不成那一个 | 条款两读，交用户 | 模型，推的 |
| 三 | 己：新的那一组仍点着出错的那张卡，实现答换上了照样解除 | 条款字面如此，交用户 | 模型，推的 |
| 四 | 走不到的理由二次查实 | 不成立（三·甲） | |
| 四 | 类型与名字 | 不成立（`EarlierOpeningOfTheDeviceNotReturned` 的文档与出路对三·乙那一种是假话） | 读码 |
| 四 | 不兼容旧、不新建判定库 | 成立 | grep 0 命中 |
| 四 | 多机可用 | 推不出（真卡对拍「本次未跑」那两处没读任务日志） | |
| 四 | 测试优先多线程 | 成立（两处有上限的轮询等法，不是写死睡多久） | 读 diff |
| 四 | 带 --item | 推不出（变异行名与任务日志没逐字核；新分支有几处没找到变异行，见四） | |
| 四 | 新崩溃流 | 推不出（重复键那一次只删意图走的是已有的同一条删法，没核崩溃流罩没罩） | 读码 |

## 一、攻击面 1：「已定」撑不撑得住

### c996-code-r2 判为修掉的七格

| 格 | 回 | 依据 |
|---|---|---|
| 甲（新卡开卡期间旧卡出错） | 站住（按条款现行文本） | 结局从「当场解除」换成停着、换上了解除，这是新四的改法、条款也改了。模型 `attack_repaired_stop_on_the_card_before_the_switch` 断言 `LeftStoppedWhileTheSwitchOpensItsCards`、换上之后不停、生效的是 41；出错之前请求了、还没交给实现的那一种照旧当场解除：「if self.requested_from_the_next_batch.is_some() {」（`crates/singlefs-core/src/background_compute.rs:1527`），模型 `attack_repaired_stop_before_the_handing_in` 原样重跑 |
| 乙（交设置那一刻读到旧那一组的「用不上」） | 站住 | 两半模型原样重跑（`attack_repaired_failure_of_the_settings_before_the_switch`、`attack_repaired_stale_stop`），这一批没碰这一支 |
| 丙（开完换上的那一组一张卡都用不上） | 站住 | 三条模型原样重跑 |
| 丁 | 被新七改写，见第二节 | |
| 戊 ①②、己 | 站住 | 戊 ② 模型原样重跑；己随新一、新二的改法（核心层开不成不记成生效），见第二节 |
| 庚（起不了开卡线程） | 站住 | 起不了线程记在头一张卡上、记进连着开不成的次数（`crates/singlefs-background-gpu/src/background_gpu.rs:3007`–`:3012`），到 4 就定成开不成；它不经切换线程，不撞三·甲那一处 `debug_assert!`（读码，推的） |
| 设计问题 ③(b)（叫停之后正在开的那一次照旧开完） | 站住 | 叫停只在每一次开卡之前看：「let opened_once = if called_off.load(Ordering::SeqCst) {」（`crates/singlefs-background-gpu/src/background_gpu.rs:1451`）。正在开的那一次开完之后照旧记进连着开不成的次数（`:1470`）——三·甲就从这里来 |

### bggpu-audit-fix-code-r1 判为站住的几格

- G01（量代价出错的泄漏）：站住。甲的改法在量代价之前先等账上还在跑的提交：「.room_once_the_submission_running_past_its_wait_completed()」（`crates/singlefs-background-gpu/src/background_gpu.rs:3137`），等不到就照量代价出错、这张卡不参与、账照旧记着；量代价出错时处置这张卡的仍是同一处：「return Err(failure_of_the_measurement_dropping_or_leaking_the_card(」（`crates/singlefs-background-gpu/src/background_gpu.rs:1521`）。判别子、泄漏账、卸载读的还是同一个「还在跑」的字段，没找到第二处来源（读码，推的）。
- G03（校验和、区域扫描两段的重复键）：站住——先查重复、后动计数、CPU 参照不调。但「先查」只先于比对，不先于交卡：这两段先交卡算完（「let both_kernel_outcomes = self.both_kernel_outcomes_of_the_batch(units);」，`crates/singlefs-background-gpu/src/background_gpu.rs:3552`），卡出错就原样交回出错、根本不查重复（「Err(batch_failure) => Err(batch_failure),」，`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:124`）。选段打分那一段反过来，见三·丙。
- G04（切批算术）与这一批换类型：站住。逐处对过 diff 里 `memory_limit_and_chunks.rs`、`region_record_kernels.rs` 那一千多行的算术改写：
  - 换反的量：没找到。主机上的输入长度换成卡上字节只经 `input_buffer_bytes`（对齐）或 `checked_gpu_input_bytes`（同一个被验函数）；`largest_piece_input_bytes` 把显存上限折成输入长度那一步内值照旧（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:508` 起），交给被验函数的是同一个数。
  - panic 行为：新加的 `plus`、`minus`、`bytes_of_the_copies`、`bytes_of_the_entries` 顶替的都是裸 `+ - *`；仓根 `[profile.release]` 开着「overflow-checks = true」（`Cargo.toml:25`），原式在 debug 与 release 里同样 panic。`minus` 顶替的两处：一片终点扣起点，与「一条扣掉头一片」——后者上游先筛过「.filter(|(_position, input_bytes)| **input_bytes > UNIT_HEAD_PIECE_INPUT_BYTES)」（`crates/singlefs-background-gpu/src/region_record_kernels.rs:652`），喂不到下溢。
  - `checked_` 失败支：「memory_limit.checked_sub(staging_of_the_input)」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:393`）与「memory_limit.checked_sub(bytes_of_a_chunk_and_an_entry_besides_the_input)?;」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:508`）失败时去向同原来（判装不下 / 交回 `None`），只是从内值挪到类型上做。
  - 模型 `attack_model_staging_arithmetic_sweep` 照 r1 那一份的种子与取样在换类型之后的签名上重算 20 万例，预测六个计数照旧（0 个不一致）；推的，交主 agent 跑。

### 换了算法的那句断言：左边在哪种输入上与原来不同

三段的「"这一段真建的缓冲与切批按的占用一致"」（`crates/singlefs-background-gpu/src/background_gpu.rs:2153`、`:2285`、`:2420`）左边原来把各缓冲内容长度（或输入的 `input_buffer_bytes`）直接相加，现在每份过一次 `input_buffer_bytes`（向上补到 4 的整数倍、至少 4）再加，例：「.plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes)),」（`crates/singlefs-background-gpu/src/background_gpu.rs:2137`）。两种算法只在某一份内容**长 0、或长度不是 4 的整数倍**时不同。右边（`ChunkFootprint::bytes_held_by_a_chunk` 与映射、记录表的 `bytes_on_the_gpu`）按「字数 × 4」算，不补最少一字：「GpuMemoryInBytes(4 * u64::try_from(words).expect("装得进 u64"))」（`crates/singlefs-background-gpu/src/region_record_kernels.rs:430`）。所以若哪一份表是空的，新左边多 4×2 字节、断言 panic，而旧左边与右边一起少算、照过。

产品路径上造不出空表或不对齐的内容：区域扫描的输入补到对齐长度（「input_bytes.resize(」，`crates/singlefs-background-gpu/src/region_record_kernels.rs:300`），映射表空时补一项全 0（「entry_words.resize(MAPPING_ENTRY_WORDS, 0);」`crates/singlefs-background-gpu/src/region_record_kernels.rs:409`、「location_item_words.resize(LOCATION_ITEM_WORDS, 0);」`:413`），记录表同（「record_words.resize(ALLOCATION_RECORD_WORDS, 0);」，`crates/singlefs-background-gpu/src/live_slot_kernels.rs:125`），参数、描述、段界都是字的列表、至少一段一条。所以对产品的输入左边逐个与原来相等，**站住**。能让它不同的只有绕过 `of_the_mapping_entries` / `of_the_allocation_records`、直接拿 `pub` 字段拼一张空表交进来（测试里能写），那时 panic 的是这句断言，不是记账错。推翻条件：找到一条产品路径交进来的缓冲内容是空的或不对齐。

## 二、攻击面 2：改法换没换到东西（十三格）

前两轮的模型在今天的代码上编不过、或插不进去：c996-code-r2 GPU 包那一半用的 `opening_of_a_card_within_the_limit` 多了「还没回来的开卡那一张表」一个参数、交给起卡那一件的泄漏字节换成了 `GpuMemoryInBytes`；bggpu-audit-fix-code-r1 的 `apply_model.py` 要给替身卡加的字段 `cost_measured_through_the_checksum_chunk` 今天已在替身卡里（第一处替换命中 0 次、退 3），切批算术那一条的签名全换了类型。所以我把两份模型的用例照原历史、原 `name=` 键移到今天的签名上（只改签名必需的几处），断言换成条款现行文本要的结局，放进这一轮的模型目录一起跑；c996-code-r2 核心层那一半的替身与历史逐字照拷（`attack_r3_core_tests.rs` 开头 148 行），只改断言。原读数取自 `research/prompts/c996-code-r2-main-checks/attack-model-lines-task-14649.txt` 与 `research/prompts/bggpu-audit-fix-code-r1-main-checks/attack-model-lines-task-14625.txt` 的判决引文（判决第二节原样）。

| 格 | 原读数（判决引的） | 今天该翻成 | 回 | 实现员的回归用例断言的是什么 |
|---|---|---|---|---|
| 新一 | GPU 包：`still_opening_at_the_retry=true batch_panicked_with=Some("参与的卡一张都不剩…")`；核心层：`devices_in_effect_once_not_opened=Some([…41…]) route_at_the_retry=Some(Gpu)` | GPU 包：`still_opening_at_the_retry=false retry_answered_used_up=true batch_panicked_with=None`；核心层：`devices_in_effect_once_not_opened=None route_at_the_retry=Some(Cpu(CpuRequestedByTheSettings))` | 修掉了（推的） | 条款要的结局：核心层 `crates/singlefs-core/src/background_compute.rs:2830` 断言开不成之后生效的仍是 `cpu`、再请求那一批走 CPU；GPU 包 `crates/singlefs-background-gpu/src/background_gpu.rs:7714` 断言再交同一组当场答开不成 |
| 新二 | GPU 包：`devices_given_work=[…01…]`；核心层：`devices_in_effect_once_not_opened=Some([…41…])` | 核心层 `Some([…01…])`（与实现照旧参与的同一张）；GPU 包那一批照开不成报错、`devices_given_work=[]` | 修掉了（推的） | 条款要的：`crates/singlefs-core/src/background_compute.rs:2850`、`crates/singlefs-background-gpu/src/background_gpu.rs:7732` |
| 新三 | `arm=mount panicked=true starts=1` | `arm=mount panicked=false opened=true starts=2` | 修掉了（推的） | 条款要的：`crates/singlefs-background-gpu/src/background_gpu.rs:7805` |
| 新四 | `routes=[Gpu, Gpu, Gpu] after_each_error=[StopLifted…×3] stopped=false` | `routes=[Gpu, Cpu(Compensating…), Cpu(Compensating…)] after_each_error=[LeftStoppedWhileTheSwitchOpensItsCards] alarms=[Some(0), Some(1), Some(2)] stopped=true`，答换上了之后 `route_once_put_in_effect=Some(Gpu) stopped_once_put_in_effect=false` | 修掉了（推的） | 条款要的：`crates/singlefs-core/src/background_compute.rs:2908` 钉了停下、报警、补跑、换上解除；`:2976` 钉了答开不成照开不成停着，但没钉开不成之后报警的叫起次数与补跑批数（三·戊） |
| 新七 | 丁那两条：`causes_reported=Some(4) openings_after_the_poll=8`、`openings=8` | `causes=Some([OpeningsInThisMountUsedUp { failed_openings: 4 }]) openings_after_the_poll=4`、`openings=4` | 修掉了（推的）；带三·甲 | 条款要的：`crates/singlefs-background-gpu/src/background_gpu.rs:5987`、`:8832`、`:8971`。没有一条走「叫停之后在第 4 次开卡还没回来时再交」那一种交错（三·甲） |
| 壬 | `later_mount_first=Some(WaitForTheGpuExceededTheLimit…) starts_after_the_later_mount=2` | `arm=shared_table`：`later_mount_first=Some(EarlierOpeningOfTheDeviceNotReturned) later_mount_started=false starts_after_the_later_mount=1`；`arm=table_of_each_mount` 照旧是 2（对照） | 修掉了「到点之后挂住」那一种（推的）；表的射程多管了没到点、正在开的那一种（三·乙） | 条款要的：`crates/singlefs-background-gpu/src/background_gpu.rs:8077`，只钉了到点之后挂住的那一种 |
| 癸 | 读码：切换关卡泄漏的累计没有产品读者 | 运行库两处读得到 | 修掉了（读码，没模型） | `MountedRuntime::gpus_leaked_by_the_background_gpu`（`crates/singlefs-mount-runtime/src/lib.rs:940`）与 `UnmountedPool` 同名字段（`crates/singlefs-mount-runtime/src/lib.rs:323`）读的是开卡器交出的同一份；开卡器把两份都接上（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:209`–`:212`） |
| 新五 | 读码：起不了线程之后为被盖掉的那一次再起 | 为最近那一组起 | 修掉了（读码） | `crates/singlefs-background-gpu/src/background_gpu.rs:2735`–`:2744`：号不同就走 `start_the_opening_for_the_settings_waiting` |
| 新六 | 读码：为被盖掉的切换开成的卡一直开着 | 收下那一刻关掉 | 修掉了（读码 + 戊 ② 模型） | `crates/singlefs-background-gpu/src/background_gpu.rs:8273` |
| 甲（bggpu） | 两段历史各 5 种用户切换里 4 行 `outcome=panicked message=这张卡上还有一次等超过上限…` | 10 行都 `outcome=returned`，`name=attack_model_sweep_summary rows=10 panicked=0` | 修掉了（推的） | 条款要的：`crates/singlefs-background-gpu/src/background_gpu.rs:8690`–`:8805` 那几条把两段历史 × 五种切换逐条钉成不 panic |
| 乙（bggpu） | 6 行 `outcome=panicked message=assertion…丙段交进来的是单元区的整段` | 9 行（原 6 行加三·丙的 3 行）都 `outcome=returned_error`，`gpu_failures_reported_as_the_repeated_key=3` | 修掉了（推的）；带三·丙 | 条款要的那一半：`crates/singlefs-background-gpu/src/live_slot_kernels.rs:1015`–`:1056` 六条；「两份核出错」那三条交进去的错本身就是「同一个键两次」，没有一条交真的 GPU 出错，所以三·丙没被钉 |
| 丙（bggpu） | 读码：等不到冲掉待写的空提交时重交数多算 1 | 单独的原因、算进超时数、不算交到卡上 | 修掉了（读码） | 条款要的：`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:994`；判法见 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:203`–`:205` |
| 重复键只放弃这一批 | 判决：现行代码停下并报警 | 不停、不报警、下一批照常 | 修掉了（推的） | 条款要的：`crates/singlefs-core/src/background_compute.rs:3051`；连着每一批都重复时的结局见三·丁 |

逐条的预测结局在 `out/predicted-statuses.txt`（27 条：25 条 ok、2 条 FAILED，两条 FAILED 就是三·甲、三·乙）。推翻条件：复跑里上表哪一行的 `name=` 读数与「今天该翻成」那一列不同，那一格改回「还中」或「推不出」。

## 三、攻击面 3：修补带出的新状态与交错

### 甲（打中，重）：叫停之后第 4 次开卡还没回来时切回同一张卡，切换开卡线程撞 `debug_assert!`

最小历史（每一步都是条款许可的用户动作，指到许可它的那一句；D26（后台整理与放置回收） 已定项 13 第 4 条在 `.claude/kb/decisions/26-后台整理与放置回收.md:253`、第 2 条在 `:251`）：

1. 挂载选 `gpu`、点开着的 A；切换只点没开的 B（第 2 条「挂着时切换点到还没开的卡，那一刻再开那张卡」）。B 的驱动每次开都不成（例：每次到点、或编管线失败）。切换开卡线程 T0 开 B：第 1、2、3 次开不成，连着开不成的次数记到 3；第 4 次正在开。
2. 第 4 次还没回来时用户切到 `cpu`（第 2 条「从 gpu 切到 cpu 算全部卡都不再参与，切换生效时关掉全部开着的卡、丢掉还在开的那一组」）。外壳取走 T0、叫停（`crates/singlefs-background-gpu/src/background_gpu.rs:2666`–`:2667`），正在开的那一次照旧开完（设计问题 ③(b) 判成立的那一格）。T0 从开头到开完一直持着开卡那一步的锁（「let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(」，`crates/singlefs-background-gpu/src/background_gpu.rs:2970`）。
3. 用户紧接着切回 `gpu`、点 B（停着时、切换本身都照收）。交设置那一刻 B 连着开不成的是 3，不到 4，「if let Some(used_up) =」（`crates/singlefs-background-gpu/src/background_gpu.rs:2635`）判不出试满；`opening_at_a_switch` 已被第 2 步取空，「if self.opening_at_a_switch.is_none() {」（`:2647`）成立，当场起 T1。T1 卡在 T0 持着的那把锁上。
4. T0 的第 4 次开不成，记进次数（`crates/singlefs-background-gpu/src/background_gpu.rs:1470`，叫停只挡下一次、不挡这一次的记账）→ 4。T0 放锁。
5. T1 拿到锁，现读次数：「let openings_left_in_this_mount = reopenings.openings_at_most().saturating_sub(」（`crates/singlefs-background-gpu/src/background_gpu.rs:1432`）得 0，`debug_assert!` 不成立：「"连着试满的卡在交设置那一刻、为在等的那一组起开卡之前就答开不成，不为它起开卡线程"」（`crates/singlefs-background-gpu/src/background_gpu.rs:1437`）。仓根 `[profile.release]` 写着「debug-assertions = true」（`Cargo.toml:27`），产品构建里同样 panic。T1 持着锁 panic，锁被毒化，送结局的那一头随之丢掉。
6. 下一批开头核心层在会话锁里问实现（两批之间那一问），外壳收到断开、照原样再 panic：「std::panic::resume_unwind(opening_thread.join().expect_err(」（`crates/singlefs-background-gpu/src/background_gpu.rs:2772`）。panic 穿过核心层借出实现的那把锁，之后每一次借都撞「.expect("GPU 实现上一次借出时 panic 过：那一次已经往上报过，不带着可能半改的连续不确定计数再用它")」（`crates/singlefs-core/src/background_compute.rs:558`）。之后任何一次切换要开卡，新线程都撞毒化的锁：「"开卡那一步只在开卡线程上锁着用，开一次卡里的 panic 在锁里就接住了：锁没被毒化",」（`crates/singlefs-background-gpu/src/background_gpu.rs:2971`）。

结局：维护线程在会话锁里 panic（与新一同一级的后果）。没有 debug 断言的构建里则是 B 在这一次挂载里连着开第 5 次，越过「同一张卡在一次挂载里切换开卡连着开不成满 4 次（跨切换接着数，中间开成一次就清零），之后再有切换点到它不再开，当场答开不成」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）的上限——交设置那一刻是 3，读法上可以说那一次切换还许开 1 次；但产品构建开着断言，走到的是 panic。

窗口：第 2、3 步要落在第 4 次开卡的那一段里。开不成是「当场起不了设备」时这一段很短；开不成是「开卡到点」（每次至多 5 分钟）或编管线慢时，窗口是整整一次开卡的时长。推的，没量过真驱动。

模型：`attack_r3_switching_back_while_the_fourth_opening_is_held_reaches_the_debug_assertion` 放开用户的两步扫 12 段历史——挂着的是第 3 次还是第 4 次 × 切回点 `[41]`、`[81]`、`[81, 41]` × 在挂着的那一次开完之前还是之后切回，只固定前缀（挂载、头一次切换、开不成）与故障（41 每次开不成）。预测（推的）：只有「第 4 次挂着、切回的组里有 41、在它开完之前切回」那 2 段 `poll_panicked_with=Some("连着试满的卡…")`、`later_switch_panicked_with=Some(…锁没被毒化…PoisonError…)`、`openings_of_forty_one=4`；第 3 次挂着的 6 段与切回只点 81 的照常（41 至多开 4 次，81 开成）；在它开完之后才切回的，交设置那一刻就判出试满、当场答开不成。所以分辨臂是用户动作的时机，不是装置写死的。

打中之后四句：
- 分不分辨臂：分。同一前缀下，切回时机在第 4 次开卡之内、之外两种分出 panic 与不 panic；只切 81 的那几段不中。
- 被判的系统当时看不看得到判别它的东西：看得到。T1 起线程之前外壳手里有 `failed_openings_in_this_mount` 与「上一条切换线程被叫停、还没回来」这两样（后者在第 2 步取走 `opening_at_a_switch` 时丢了，没记）；T1 自己在锁里也读到了 4，只是拿它当断言而不是当判据。
- 满足的是判据字面的哪一个分句：检查单「走不到的理由二次查实」——批次 3 报告列的那一处 `debug_assert!`「累计试满的卡不交到切换开卡线程」，它的走不到理由（交设置那一刻已判）在这条交错里不成立；后果落在第 4 条「之后再有切换点到它不再开，当场答开不成」与新一那一级的「维护线程 panic」。
- 跑前条款给的改法在这几格上还中不中：判决改法 4（交设置那一刻判试满）对这几格不起作用——判的那一刻次数是 3；见第五节的改法甲。

### 乙（打中，中）：进程级那一张表把「正在开、没到点」的开卡也当成「前一次开卡没回来」

代码：一张卡上一开卡就记进那一张表、开完才划掉（「devices.push(device.clone());」`crates/singlefs-background-gpu/src/background_gpu.rs:1280`，划掉在 `Drop`，`:1288`–`:1295`），不管调用方到没到点。这张表挪到进程级之后各次挂载共用（壬的改法）；而进程开卡器只把**挂载那一刻**的开卡排成一次一个（「/// 同一时刻只有一次开卡：两次挂载同时开卡时后到的那一次在这里等（这段等待不算进期限）。」`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:142`），挂着时的切换开卡在每次挂载自己的线程上开、不经这把锁。所以同一进程里两次挂载同时在同一张卡上开卡（挂载乙挂载那一刻开 X，挂载甲的切换此刻也在开 X；或反过来），后到的那一方当场得「EarlierOpeningOfTheDeviceNotReturned」，而这个原因的文档写的是「/// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），这一次不再起：算这一次又没开成，照间隔往下数；出路只有重启进程。」（`crates/singlefs-background-gpu/src/background_gpu.rs:160`）——对这一种是假话，出路也给错了。它还算进连着开不成的次数（「| GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned」`crates/singlefs-background-gpu/src/background_gpu.rs:1389`）。

最小历史：同一进程挂着池甲（选 `gpu`），在挂池乙（点 X）。池乙挂载那一刻开 X 正在编管线（没到 5 分钟的点）；用户这时在池甲上切换只点 X。池甲的切换线程开 X 四次，每次都撞到池乙那一次还在开，四个原因全是「前一次开卡没回来」，答开不成、GC 停下报警（出路写重启进程）；池乙那一次随后开成。用户照常再切一次点 X——次数已是 4，当场答 `OpeningsInThisMountUsedUp`、要卸载重挂池甲，而 X 没坏、也从没挂住。反过来（池甲的切换正在开 X、池乙挂载那一刻开 X）则是池乙挂载被拒、报四次「前一次开卡没回来」。产品里四次开卡隔 1 分钟重开，覆盖约 3 分钟；那一方的开卡要持续 3 分钟以上（编管线慢、驱动慢，至多到 5 分钟的点）才四次都撞上，不然只是多一次开不成、下一次开成（推的，没量过真驱动）。

条款：「同一张卡前一次开卡没回来就不再起下一次，算这一次又没开成，照间隔往下数」与「记着哪几张卡上有没回来的开卡的那张表在进程级（开卡器持有），各次挂载与切换开卡共用：前一次挂载里挂住的卡，之后的挂载同样不再起，出路只有重启进程」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。条款说的是「挂住」「没回来」，用来闸「到点之后还挂在驱动里」的那一条线程；并发的、正在开的那一次是不是「没回来」，条款没写——c996-code-r2 时表是每次挂载一份，两次挂载之间不会这样串，这一种是壬的改法带出来的。

模型：`attack_r3_an_opening_under_way_in_another_mount_uses_up_the_card_in_this_mount`，两臂只差两次挂载拿不拿同一张表；另一次挂载的开卡等用例放开、期限 20 秒（没到点）。预测（推的）：`arm=shared_table` 这一次挂载切换只点 41 答 `NotOpened`、四个原因都是 `EarlierOpeningOfTheDeviceNotReturned`，另一次挂载那一次开成（`the_other_mount_failed_with=None`），再切同一组答 `OpeningsInThisMountUsedUp { failed_openings: 4 }`；`arm=table_of_each_mount` 两次都 `PutInEffect`。用例断言条款要的（不该把别的挂载正在开的卡算成这一次开不成），预测红。

打中之后四句：
- 分不分辨臂：分。两臂只差「共用进程级的表」这一样，共用那一臂中、各一张不中；第三种做法（表只记到点之后的那几次）见改法乙。
- 被判的系统看不看得到判别它的东西：看得到。记进表的是 `opening_of_a_card_within_the_limit` 起的那一次，它自己的调用线程知道自己到没到点（`outcome_of_opening_a_gpu_within_the_limit` 交回 `WaitForTheGpuExceededTheLimit` 的那一刻）。
- 满足的是判据字面的哪一个分句：检查单「类型与名字」（成员文档与出路对这一种是假话）；第 4 条「出路只有重启进程」「要卸载重挂」给错了出路。不是「判别子观测不到」。
- 改法：判决改法 5（表挪到进程级）正是带出这一格的那一条，对它不起作用；见改法乙。

### 丙（打中，轻；要调用方组批先有 bug）：选段打分那一段重复键压过同一批真的 GPU 出错

选段打分那一段先交卡算（「let both_kernel_live_slots = self.both_kernel_live_slots_of_the_cluster_segments(」`crates/singlefs-background-gpu/src/background_gpu.rs:3719`），再查重复：「if let Some(repeated) = first_entry_key_repeated_among(segments.iter()) {」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:502`），文档写明「/// 同一个段在一批里出现两次时交回 [`SameEntryKeyTwiceInOneBatch`] 转成的那一种（两份核交成了还是出错了都先报它）」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:490`）。卡真出错（例：等超过上限第 4 次、报这张卡用不了）而段又重复时，交到核心层的是重复键，「核心层收到一种不归哪张卡的出错（`SameEntryKeyTwiceInOneBatch`），后台任务不停、不报警，下一批照常算」（`.claude/kb/decisions/26-后台整理与放置回收.md:257`）——那张卡的出错不报警、任务不停，下一批照旧交给它。校验和与区域扫描两段反过来：卡出错就交回出错、不查重复（第一节 G03）。条款没写两样同时出现时报哪一个；三段做法不一样。模型 `attack_model_a_repeated_segment_meets_the_full_list_assertion_first` 的 `kernels=gpu_failed` 三行预测 `outcome=returned_error`（重复键）、`gpu_failures_reported_as_the_repeated_key=3`（推的）。可达性：段清单由核心层按单元区给，重复只在调用方有 bug 时出现；单独出现时不中。

四句：不分辨臂（它是三段之间的不一致，与哪一种改法无关，属「打中不分辨臂」那一类、另记一笔）；系统看得到两样；满足的是第 4 条「GPU 出错……停下……报警」与第 8 条「只放弃这一批」两句的交界，条款没写；改法丙。

### 丁（条款字面如此，零轮形态交用户）：重复键每一批都出现时 GC 空转、不报警、不补跑

重复键是调用方组批的 bug，在同一份盘上多半每一批都一样。条款「后台任务不停、不报警，下一批照常算」照做的结局：每次叫起签意图、算、放弃、单发一次只删意图的发布，GC 一批都搬不成；任务没停，所以第 4 条「GC 报警停着之后……超过阈值……在 CPU 上补跑」也轮不到。占用一路涨到满，用户不见报警。模型 `attack_r3_a_repeated_entry_key_in_every_batch_neither_stops_nor_compensates_nor_alarms` 两臂各叫起 4 次、池子都过了阈值：`arm=repeated_entry_key` 预测每次都 `Some(Gpu)`、`NotStoppedForOnlyTheBatchIsAbandoned`、报警 `None`；`arm=gpu_failed` 头一次停下，之后在 CPU 上补跑、报警 1、2、3（推的）。这是第 8 条字面的结局，不算打中，写进交用户的：要不要给「连着几批都是重复键」设一个上限、到了就照 GPU 出错停下报警（或交 CPU 补跑）。每一批的记账里带着放弃它的那个错（`compute_error_that_abandoned_the_batch`），运行时读得到，但不是报警通道。

### 戊（条款两读，交用户）：开卡期间出错停着、实现再答开不成，报警的叫起次数与补跑批数从头数

新四的改法新开了「停着、等换上解除」这一种状态。之后实现若答开不成，核心层换掉停着的那几条：「self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));」（`crates/singlefs-core/src/background_compute.rs:1822`），换掉时「self.wakes_of_the_tasks_stopped.clear();」（`crates/singlefs-core/src/background_compute.rs:1573`），补跑批数从 0 起。任务从出错那一刻起一直停着、没解除过，报警却从 1 重数、`batches_compensated_on_the_cpu` 归零、`error_that_stopped_the_task` 换成开不成那一个。条款写的是「停下以来叫起了几次（`wakes_while_stopped`」与「停着期间在 CPU 上补跑了几批（`batches_compensated_on_the_cpu`）」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`），又写「答开不成照开不成停着」：把开不成算一次新的停下，现行代码对；算同一次停下的延续，就该接着数。回归用例 `crates/singlefs-core/src/background_compute.rs:2976` 只钉了停着的错换成开不成那一个，没钉这两个数。模型 `attack_r3_stopped_while_the_switch_opens_then_not_opened_restarts_the_count_of_the_alarm` 预测（推的）：开不成之前三次叫起报 0、1、2，补跑 2 批；开不成之后那一次叫起报 1、补跑 1 批、错换成 `GpuUnusableUnderTheSettingsHandedIn`。只断言任务停着，两读不替用户选。

### 己（条款字面如此，交用户）：新的那一组仍点着出错的那张卡，换上了照样解除

挂载点 A，请求切到 {A, B}（B 没开）；开卡期间那一批在 A 上出错、停下；B 开成、实现答换上了——解除，下一批照 {A, B} 在 GPU 上算，A 刚出过错。条款原话「实现答新卡开成（换上了）那一刻解除停下，答开不成照开不成停着」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）不分新的那一组点没点出错的卡。用户定案的理由是「用户切走它正是因为它坏了」，这一种不是切走。模型 `attack_r3_the_switch_still_naming_the_failing_card_lifts_the_stop_once_put_in_effect` 照条款字面断言解除（预测过）。交用户：这一种要不要照旧停着、等下一次通过校验的切换。

### 其余攻击面 3 点名的交错（没打中，推的）

| 交错 | 走到哪 | 依据 |
|---|---|---|
| 连着开不成的计数会不会漏清、多清 | 开成清零只在切换线程收到 `Ok`（`crates/singlefs-background-gpu/src/background_gpu.rs:1372`）；叫停那几次不记（`:1385`）；为被盖掉的切换开成也清零（它确实开成了）。没找到漏清或多清的一支；只有三·甲那一处是「记账晚于判」 | 读码 |
| 挂载那一刻的开卡算不算进计数 | 不算：计数表跟外壳一起建在挂载那一刻开完之后（`crates/singlefs-background-gpu/src/background_gpu.rs:2538`），挂载那一刻开卡走 `gpus_opened_at_the_mount`、不碰它。与条款「切换开卡连着开不成」一致 | 读码 |
| 先后两次挂载之间表里留下的项会不会让后一次永远开不成 | 不会，除非那条线程真的永远不回来：项在开卡线程结束（含 panic 展开、起不了线程时闭包被丢）时划掉，`Drop` 在 `:1288`–`:1295`，送结局之前就划掉（`:1216`），所以后一次挂载看到的只有还在跑的那几条。模型壬那一条的两臂 | 读码 + 模型 |
| 开卡期间出错停着时再来一次切换（盖掉在开的那一组） | 请求那一刻当场解除（`crates/singlefs-core/src/background_compute.rs:1747`–`:1748`），与第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下」一致 | 读码 |
| 实现先答开不成再来切换 | 同上，当场解除；新的一组再交给实现；若点的仍是试满的卡，交设置那一刻就答开不成、再停（新七） | 读码 + 模型新一、新二两臂 |
| 答换上了而核心层那一格已被盖掉 | 外壳只答最近交来的那一组（次序号），核心层先交新的那一组再问，问到的是新那一组；旧那一组开完收下就关不再点名的卡 | 读码 + 模型戊 ② |
| 重复键放弃的那一批与同一批别的段已交回的结果、连续不确定的计数、意图 | 别的段已交回的照算一轮（第 8 条「那一批里已经交回的段照算一轮」），重复的那一段一条不动；意图走与别的出错同一条删法：「drop_the_live_intent_on_a_stop(」之前先 `stop_the_task_after_a_compute_error`（`crates/singlefs-core/src/mounted_session.rs:5187`–`:5201`），停没停只看它交回的那一种 | 读码 |
| 重复键反复触发而任务既不停也不前进 | 有，就是三·丁；条款字面如此 | 模型 |

## 四、检查单八格

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 不成立 | 批次 3 列的 `debug_assert!`（`crates/singlefs-background-gpu/src/background_gpu.rs:1435`–`:1438`）在三·甲走到，产品构建开着断言（`Cargo.toml:27`），前提被破坏时是切换开卡线程 panic、毒化锁、下一次两批之间在会话锁里再 panic。`batch_not_split_over_the_gpus` 那一处断言（`:3181`–`:3184`）的理由在改法 1 之后成立：开不成时核心层生效的仍是切换之前那一组（`crates/singlefs-core/src/background_compute.rs:1819`–`:1822` 不再改 `in_effect`），`cpu` 生效期间不走 GPU 那一路（读码 + 模型新一，推的）。新 `panic!` 顶替的原式在两种构建里同样 panic（第一节）；`minus` 那两处上游挡住了下溢 |
| 类型与名字 | 不成立 | `EarlierOpeningOfTheDeviceNotReturned` 的文档与出路对三·乙那一种是假话（`crates/singlefs-background-gpu/src/background_gpu.rs:160`）。`OpeningsInThisMountUsedUp { failed_openings }` 名字对得上：到 4 之后这一次挂载不再开，也就不会再开成清零，「这一次挂载里用完了」是真话。diff 里没有新加的 `_ =>`（`grep -nE '^\+.*_ =>'` 0 命中、退 1）。新类型内值 `pub`：混用只靠写的人，例 `GpuMemoryInBytes(checked_gpu_input_bytes(input_bytes.0)?)`（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:312`）是对的换法、编译器拦不住错的换法。`result-2.md` 的十条「留着的裸整数」我没逐条读：这一半推不出 |
| 不兼容旧代码与旧格式 | 成立 | `grep -nE 'legacy\|old_format\|migrat\|兼容\|deprecated' research/prompts/_c996-code-r3-diff.md` 0 命中、退 1；`of_this_many_bytes` 这类收 `usize` 的是构造，不是收裸整数的转接 |
| 不新建判定库 | 成立 | diff 新增行里 `File::create\|OpenOptions\|rocksdb\|DB::open\|fs::write\|create_dir` 0 命中、退 1 |
| 所有任务多机可用 | 推不出 | 读到的新用例与我的模型都不读本机状态、不写死地址；真卡对拍「本次未跑」那两处是哪两处，要读任务 15185、15315、15444 的日志，我没读 |
| 测试优先多线程 | 成立（带说明） | 进程级那一张表不是全局静态量，每条用例自己建一张（`with_none_not_returned()`），并行的用例互相看不见；diff 里两处等法是「至多 2000 次、每次睡 10 毫秒、按状态判」的轮询（diff 第 2362、2645 行），不是写死睡多久 |
| 带 --item 的命令与项 | 推不出 | 变异表里找得到锚在新分支上的行（`AbandonsOnlyTheBatch` 2 行、`gpu_whose_openings_in_this_mount_are_used_up` 2 行、`LeftStoppedWhileTheSwitchOpensItsCards` 1 行、`OpeningCalledOff => false` 1 行，`grep -cF` 现数）；三·甲那一步算剩几次的 `openings_left_in_this_mount`（0 行）没找到变异行。51、31、55 行与 `crates/mutations.tsv` 逐字相同没核 |
| 新崩溃流多罩了什么 | 推不出 | 重复键那一次只删意图走的是与别的出错同一条 `drop_the_live_intent_on_a_stop`（`crates/singlefs-core/src/mounted_session.rs:5192`），落盘形态与已有的「一批出错放弃、删意图」相同；不同的只是之后任务不停、下一批照常签新意图（三·丁那种连着几轮），已有崩溃流罩没罩「删意图 → 紧接着再签」这一串没核 |

## 五、改法（都只在我的模型上量过或推的、被攻过零轮）

- 改法甲（修三·甲）：`gpus_opened_at_a_switch` 里算出这张卡这一次挂载里剩 0 次时，不断言，照「试满」当场记成它开不成（`OpeningsInThisMountUsedUp`）、不开；或在切到 `cpu` 取走还在开的那一次时记下「那条线程还没回来」，新切换照「上一次切换的卡还在开时又交来一组」那一支等它开完再判。前一种改动小，两种都让交设置那一刻与线程开头那一刻读的是同一个判据。
- 改法乙（修三·乙）：那一张表只记「调用方已经到点不等了、那条线程还在跑」的开卡——记表挪到 `outcome_of_opening_a_gpu_within_the_limit` 到点那一支，划掉照旧在线程结束时；没到点的并发开卡另用一把每张卡一把的进程级锁排队（后到的在自己的期限之内等），或照「正在开」报一个单独的原因、不算进连着开不成的次数。
- 改法丙（修三·丙）：三段都在交卡之前查重复（`first_entry_key_repeated_among` 的文档本来就写「交给两份核之前就查得了」），重复就一片都不交；那样同一批里卡出错与重复键不会同时出现。
- 三·丁、戊、己：不是代码与条款对不上，交用户定条款。

| 改法 | 三·甲 | 三·乙 | 三·丙 | 别的格 |
|---|---|---|---|---|
| 改法甲（判剩 0 次当场答试满） | 修（推的，待主 agent 跑：模型那 2 段该翻成 `poll_panicked_with=None`、`NotOpened(OpeningsInThisMountUsedUp)`、`openings_of_forty_one=4`） | 不起作用 | 不起作用 | 新七那几条回归用例不受影响（推的） |
| 改法乙（只记到点之后的） | 不起作用 | 修（推的，待主 agent 跑：`arm=shared_table` 该翻成 `PutInEffect`） | 不起作用 | 壬那一条照旧中（到点之后挂住的仍记进表；推的） |
| 改法丙（交卡之前查重复） | 不起作用 | 不起作用 | 修（推的：`kernels=gpu_failed` 那 3 行不再出现，重复的段一片都不交） | 乙那六条照旧交回重复键（推的） |
| 判决改法 4（交设置那一刻判试满） | 不起作用：判的那一刻次数是 3 | — | — | |
| 判决改法 5（表挪到进程级） | — | 带出这一格 | — | |

## 六、撤回与收窄

判决写这些改法是「腿提的或接手方照用户定案推的」，哪一条出自攻方腿我没逐条核；凡是攻方立场先前给过的这两条，我收窄：

- 壬的改法（「前一次开卡没回来」那张表挪到进程级）：收窄成「只把到点之后还没回来的那几次挪到进程级」。原样挪过去带出三·乙。
- 新七的改法（交设置那一刻判试满就够）：撤回「够」这一半——线程开头那一刻还要再判一次（改法甲），不然三·甲。

## 要主 agent 跑的命令

1. 攻方腿模型复跑（在新 shell 里起；在哪个目录起：`/home/fy5090/.local/share/singlefs/work/c996-code-r3-attack`）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 与 singlefs-core 的 lib 测试目标里名字带 attack_ 的攻方腿用例（模型复跑脚本）; 范围=2 个测试目标; 为什么=复跑 c996-code-r3 攻方腿模型，判决要引的读数由主 agent 复跑' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r3-attack -- bash /home/fy5090/code/singlefs/research/prompts/c996-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/c996-code-r3/repo /home/fy5090/.local/share/singlefs/work/c996-code-r3-attack-model-run /home/fy5090/code/singlefs/research/prompts/c996-code-r3-opus-model
```

   跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`（两个包都跑出了「test result:」）；日志里 `name=apply_model inserted=3`、`name=rerun_ran test_result_lines=2`、`name=prediction_check predicted=27 ran=27 matched=27`；没对上的逐条打成 `name=prediction_miss`。编不过时日志里没有「test result:」、退非 0：那是模型手写签名的错，新派同立场的腿（带「承接：」与任务号）修模型再交，不算三·甲、三·乙撤回。范围声明的写法若被闸拒，照拒绝说明改声明，命令本身不改。运行目录 `/home/fy5090/.local/share/singlefs/work/c996-code-r3-attack-model-run`（里面有冻结副本的整份拷贝）读完数之后由主 agent 删，编译目录照服务 `task-server/task clean <号>`。

## 没打中的形状

- 换类型：逐处对过 `memory_limit_and_chunks.rs`、`region_record_kernels.rs`、`background_gpu.rs` 三段算账与断言那几块 diff（约 1,600 行），找换反的量、`checked_` 去向、溢出 panic 与否、`minus` 的下溢——都没有。三段每一份缓冲内容是不是都对齐、非空，逐个追到建它的地方（第一节）。
- 进程级那一张表的泄漏：起不了线程、开卡 panic、开卡在调用方到点之后才回来、表的锁被毒化四种都会划掉（`Drop` + `unwrap_or_else(PoisonError::into_inner)`），没造出「线程回来了、项还在」的历史。
- 计数：叫停的那几次、为被盖掉的切换开的、起不了线程记在头一张卡上，逐支对条款「跨切换接着数，中间开成一次就清零」，没找到漏清或多清；只有三·甲那一处时序。
- 核心层：开卡期间出错停着 × 再请求切换 / 实现答开不成 / 答换上了 / 切到 `cpu`，逐种走了一遍（第三节表），除三·戊、三·己两处条款没分的，都与条款现行文本一致。补跑阈值那一格（开不成那一组请求给的）在请求、交给实现、开不成、换上四个时刻各读了一遍取哪一个，与「不给阈值的切换沿用原阈值」一致。
- 取样范围：三·甲放开 3 样用户动作扫 12 段；三·乙 2 臂；三·丁 2 臂 × 4 次叫起；切批算术 20 万例同 r1 种子。

## 这条腿自己的限度

- 一条都没编、没跑：模型是对着冻结副本里的签名手写的，编不过的风险在「要主 agent 跑的命令」里写了怎么办。所有读数都是推的。
- 三·甲、三·乙的窗口长短取决于真驱动开一次卡要多久（几秒还是到 5 分钟的点），没量过；模型用替身卡与用例放开的时刻造出那一段。
- 三·甲在产品里落到维护线程 panic 那一步，靠的是「产品构建开着 debug 断言」这一条（`Cargo.toml:27`）与 r2 判决里「维护线程 panic 记成运行库坏了」那一读；运行库收到 panic 之后的样子我没在这一轮重读。
- 崩溃点重放的枚举不适用：这一轮被攻的是挂着时的开卡、切换与记账，不写盘；没调 `enumerate_crash_states`，也没有崩溃状态要数。
- 第一段模型源码一次写进 155 行，超了每次 150 行的分段上限 5 行；之后各段都在 150 行以内。
- 只抽了一次样：「没打中」的那几格照规矩不撑否定结论。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没读禁读清单里的辩方腿报告。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）；没读任何任务日志（15185、15315、15341、15382、15441、15444、15450），第三节攻击面 5「报出的数字能不能核」这一轮没做。
- 没建仓副本、没建编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/c996-code-r3-attack` 里只有拷来的模板与中转用的文本片段（没有仓副本、编译目录与库拷贝）。
- `result-2.md` 那十条「留着的裸整数」与五份实现员报告「停下交主 agent 的设计问题」没逐条读，攻击面 4 的完整性只答到第三节交用户的那几条。
