# 卡一侧验证服务（gpu-compute-server）设计第一轮：正推腿报告

日期：2026-11-17。方案按 `crates/` 今天的实现谈；前提照背景材料「已定的前提」与派发提示补的一条（服务名 gpu-compute-server、仓根独立目录 `gpu-compute-server/`、客户端放 `scripts/gpu-compute-server/`，照 `prune-server/` 与 `scripts/prune-server/`）。
行号都是我这次在主工作树（`/home/fy5090/code/singlefs`）里现读的；背景材料里 `crash_amplification.rs` 的行号是段树（`batch30-pieces-with-widening-rule/copy`）的，两棵树对不上的在「背景材料『已有』『没有』逐句核」一节列。

## 复跑命令与文件摘要（模型：认领与租期状态机）

模型在草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-forward/model/`，只用 Python 标准库。**没跑**（子 agent 不跑模型）：只做了 `python3 -m py_compile`（退 0）。下面的期望输出是照模型代码推出来的，数字（状态数、边数）待主 agent 跑。

```
sha256sum（现取）
982924838332e5dda723198bed22eab3aa9a85b2b508cb92cd1184e4b0d71ebf  claim_lease_model.py
b6bfc694d45b54cfeed7d33cd8d05115c78d9e3b27abe6976a4ba6586cab2f83  tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
```

复跑（整条交服务，细节在末尾「要主 agent 跑的命令」第 1 条）：`bash <模型目录>/tools/rerun-from-a-clean-work-directory.sh <仓根> <仓根> <运行目录> <模型目录>`；单跑模型：`python3 -I claim_lease_model.py [--variant <名>] [--list] [--max-states <n>]`。
`tools/rerun.sh` 取自 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps` 一段；模板自带的 `--selftest` 断言「冻结副本被拷进 tree/」，这一腿的 `rerun_steps` 不拷树（模型不读仓），所以那一份自证对这份 `rerun.sh` 不适用，没起。
探针的内存与时间：没量（没跑）。推的：状态按台内对称归一后，七个变体各数万到数十万个状态，内存数十到数百 MiB、各分钟级；`--max-states` 缺省 3000000，超了该变体打 `result=state-limit-exceeded`、整趟退 1。

模型对应的协议（`claim_lease_model.py` 文件头逐条列了对应的 `prune-server/src/committer.rs` 位置）：一个工作线程 = 一张卡 = 一个稳定的客户端身份；动作 = 认领、续租、判完、写结论（结论行与释放认领同一批）、崩溃、重启、过一刻。七个变体与预期：

| 变体 | 配置 | 预期（`as-expected` 的条件） |
|---|---|---|
| renewal-discipline-3-2-hosts | 两台共五张卡、两块、租期 2 刻、活着的持有者先续租再过刻 | 重判已判块 0、同块同时判的状态 0、重复写 0、丢块 0、死了不回来也能接手 0、结论行被撤 0 |
| renewal-stalls-allowed-3-2-hosts | 同上，活着的线程可以停着不续租（丢了租期就放弃手里的活） | 同块同时判的状态 >0、重复写 >0，其余仍 0 |
| renewal-stalls-allowed-and-never-abandons-3-2-hosts | 同上，丢了租期也不放弃（最坏） | 同上 |
| renewal-discipline-2-1-hosts-three-blocks | 三张卡、三块、有续租纪律 | 全 0 |
| renewal-stalls-allowed-2-1-hosts-three-blocks | 三张卡、三块、可停着不续租 | 同块同时判的状态 >0、重复写 >0，其余 0 |
| broken-claim-ignores-conclusion | 弄坏：认领不看「键上已有结论」 | 必须红：`claim_on_concluded` >0 且 `work_after_done` >0 |
| broken-lease-never-expires | 弄坏：租期永不到点 | 必须红：`blocked_without_restart` >0，`lost_conclusion` 仍是 0 |

期望的末行：`MODEL_SUMMARY variants=7 as_expected=7`（后两个变体「按设计红」也算 as-expected；任何 `result=UNEXPECTED(…)` 都是模型或我的推理有错）。

**这个模型的边界（写在前面，别读成更大的结论）**
- 它核的是协议：认领成功落在没有结论的块上、写结论与释放认领同一批、租期到点放出。它假设「同一块判两遍，写出的字节相同」（判器确定）；两份结论不同时库怎么办是 `verdict_store.rs` 的事（两份都绿而不同报错、有红的整份换），模型不涉及。
- 模型里「块已有结论」与「认领键已有行」是同一件事。这只在落地时做了两处才成立：认领键是与结论块同一批写下去的**判过记号**（格 1、格 2），并且认领到之后工作线程再点读一次结论块（认领与结论之间的竞态）。少任何一处，模型的 `claim_on_concluded` 在真实现里就是可达的。
- 身份稳定（同一份追加日志重开是同一个身份）是照 `prune-server/src/reliable_client.rs` 的 `ReliableClient::open` 取日志里记的身份；不稳定身份（每次重启换身份）只会更慢（要等租期），不会更不安全，所以没单建变体。
- 「服务中途死」= 同一台的几个工作线程连续崩溃，宽搜里与别的动作的所有交错都走到；没单设「整台一起死」的宏动作。
- 没建模：prune-server 自己重启（认领行与结论行同在一份库里，`committer.rs:111` 起服务时 `load_claims` 重读，认领不丢）、网络分区、时钟跳变（租期用服务端自己的时钟，`committer.rs:481` 的 `now_milliseconds()`）。

## 格 1　待判清单的形态

**判定：成立（要新加三张表，不是改现有表）。** 「库里没有待判的块的清单」要缩准：有一张录入表，但它不是待判清单。

**今天有什么（现读）**
- 录入表（表标签 `0x05`，`verdict_store.rs:1412`）：键 = `05 ‖ 块键（72 字节）`，值 = 这一块的状态数（8 字节大端）。录入段写它：`write_the_recorded_blocks`（`crash_amplification.rs:1732`）→ `VerdictStore::record_block`（`verdict_store.rs:3665`）。它没有结论版本、没有判表摘要、没有这一块在整条流里的序号区间，块键又是哈希（`PlannedCrashPoint::block_key`，`crash_amplification.rs:430`），从键推不回这些。所以只凭它服务判不了一块。
- 判段排「该判哪些」靠流程代码现算：`pending_blocks`（`crash_amplification.rs:3727`）逐点逐块问 `is_stored_for_reuse`（`crash_amplification.rs:1950`）。
- 表标签 `0x00`–`0x1C` 都占了（`verdict_store.rs:1406`–`1437` 的 `key_tag`、`:1478`–`1510` 的 `of_key_tag`），下一个空的是 `0x1D`。

**新加三张表（标签接在 `0x1C` 后面；`key_tag`、`of_key_tag`、`what_an_empty_value_means` 三处各加三行，文件头表格同步）**

| 表 | 键（整数一律大端，同文件头约定） | 值 |
|---|---|---|
| `PendingBlocks` `0x1D` | `1D ‖ 结论版本 V（32）‖ 块键（72）‖ 轮次 u32` | 格式号 1 字节；任务种类 1 字节（1=镜像判定的一块；2=给定状态表，留给格 4 末段；别的种类以后加）；判表摘要 32（指 `PackedJudgeTables` 表 `0x11` 那一行，`PackedJudgeTablesReadBack::from_the_library_bytes`，`crash_judge_dispatch.rs:452` 读回时核 SHA-256）；这一块第一个状态在整条流里的序号 u64（= `point.ordinals.start + local.start`，`crash_amplification.rs:339`、`:4096` 同一算法）；状态数 u32（≤ `STATES_PER_BLOCK` = 65536，`crash_amplification.rs:194`）；基镜像上逐盘自证槽数（盘数 1 字节，每盘 2×u32；`store_judged_block` 的 `slots_of_each_device_on_the_base_image`，`crash_amplification.rs:3757`）；录入时刻（毫秒，水位线按它算「最老的待判块多久了」）；流的显示名（UTF-8，只进状态报告，不进任何键，D13 已定项 17 路径与名字不进键）；这一轮的随机数 16 字节（格 6 取证用） |
| `JudgedMarkers` `0x1E`（认领用的键） | 与上一行同一串 `V ‖ 块键 ‖ 轮次` | 1 字节 `01`（空值是删除记号，所以不能写空） |
| `GpuServerStatus` `0x1F` | `1F ‖ 台标签的 SHA-256（32）‖ 卡序号 u32` | 服务当时的状态：V、判器各组指纹的摘要、二进制 SHA-256、协议号、布局号、起来的时刻、最近心跳、这张卡的预算与此刻在判的流、本进程判过的块计数、最近一次显存不够的原因 |

- 键里 V 在最前：服务只前缀扫自己那一把 V（`1D ‖ V`），别的版本的行它碰不到（格 3）。轮次平时是 0；格 6 的强制现判与「记红要重判」用轮次加一，认领键随之换一把（原因见格 2）。
- V、块键、判表摘要、序号区间各从哪来（都是录入段已经算好的东西，不新算）：V = `ConclusionVersionsOfThePlan::judgement_units().fingerprint()`（`crash_amplification.rs:21431`，取版本的函数 `of_the_plan_keeping_the_refused_out`，`:21313`，`run_from_environment` 在 `:18940` 调它）；块键 = `point.block_key(&local)`；判表摘要 = `read_back.manifest.packed_judge_tables_digest`（`RecordedRunManifest`，`crash_amplification.rs:6532`、`:6536`，`read_back_the_recorded_run` `:6870`，`run_from_environment` 里 `:18910` 读回）；序号区间与状态数 = 计划的点（`PlannedCrashPoint.ordinals`）与块区间 `local`。
- 写的人与时点：新函数 `write_the_pending_blocks` 放在 `write_the_recorded_blocks` 旁边（`crash_amplification.rs:1732` 之后），入参就是 `pending_blocks` 交回的向量（`:3727`，`is_mine` 取「全部」）；由 `run_from_environment`（`:18695`）在 `tally_the_recorded_blocks` 之后、配了 GPU 且判段要上卡的那一支里调。今天 GPU 判一块的函数 `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`（`crash_amplification.rs:4838`）自己调 `pending_blocks`（`:4849`），这一处改成「读结论、等服务」；崩溃枚举用例那一路自己另有一份 `pending_blocks` 调用（`judge_the_recorded_item`，`crash_state_enumeration.rs:2140`，调在 `:2240`），写同一种行（它的 GPU 判是否也走 `judge_my_blocks_on_gpu_cards`，我没读，推的）。次序：流程（`0F`）、清单（`10`）、判表（`11`）这几样录入段本来就先写完再读回，待判行最后写，服务读到待判行时它点名的判表一定已经落盘。`VerdictStore` 上加 `record_pending_block`（放在 `record_block`，`verdict_store.rs:3665`，旁边），一帧至多装几千行（帧长上限 `MAXIMUM_FRAME_BODY_BYTES` = 64 MiB，`prune-server/src/protocol.rs:64`）。
- 行数量级：登记的最大一条流 1 678 377 220 态（`.claude/kb/verification-build.md` 开篇写的覆盖写固定脚本流）按每块 65536 态是 25 609 块；每行约 200 字节，一趟写 5 MB 量级（推的，按字段长度加）。

**与今天判段自己排的计划是不是同一个集合：是，对「镜像判定」这一种。** 待判行写的恰是 `pending_blocks(store, plan, conclusions, is_mine=全部)` 的结果：键相同的、库里已有不红结论的块不写（`is_stored_for_reuse` 的谓词原样），所以集合差只在「按份」：今天一台只判 `my_blocks`（`crash_amplification.rs:19004`，按 `SINGLEFS_CRASH_AMPLIFICATION_SHARE` 的份），服务版的份由认领动态分，所有份的并集 = 全部。两处不在这个集合里：观察者的结论（种类 `02`，挂在点上的观察者另有判器结论之后的判段，`crash_amplification.rs:19025` 起，它要的 GPU 活是对观察者交回的镜像跑池级 checker，表 `0x17`）；复用哨兵与强制现判的取舍（格 6）。

**算不算「判定库只有一份」里说的另一类东西：不算另一份库，算另一类行。** 三张表都在 prune-server 那一份 RocksDB 里、经 `reliable_client` 读写（`verdict_store.rs:3`），没有新的存放处。但 `.claude/rules/verification.md` 第 54 行「库里只放剪枝按单元键复用时要读的键」这一句要改：录入表、流程、判表这几样本来就不是复用键，是录入段交给判段的数据，待判行与认领记号、状态行同属这一类（用户已答：「这里看来是要改一下 把值也写入了是吧」）。改法写在格 9。

**什么现象会推翻**：(a) 服务扫 `1D ‖ V` 前缀时读到别的版本的行（键序写错）；(b) 录入段与待判行写入之间崩溃，留下点名了还没落盘判表的行（写序倒了）；(c) 版本换过之后旧 V 的待判行堆积得扫不动——需要一个清理：服务起来时对 `V ≠ 自己` 且录入时刻早于 7 天的行用已有的删前缀（`protocol.rs` 的 `delete-prefix`）删掉，这一条是我加的，没量过堆积速度。

## 格 2　认领与不重判

**判定：成立——已有的带租期的认领够用，要补四处；状态机已写成模型（七个变体，待主 agent 跑，跑之前这一格只算「推的」）。**

**已有的认领是什么（现读 `prune-server/src/committer.rs`）**：`add_claim`（`:469`）逐键先判后写，只有提交线程一个改认领表（`:21`「只有这一个线程改库，去重与认领的「先判再写」不会被别的线程插进来」）；结局三种：这把键本身已是数据行 → `AlreadyConcluded`（`:488`，`library.contains_key`）；别的身份持有且租期没到 → `HeldByOther`（`held_by_other`，`:678`）；否则 `Claimed`（`:567`），同一身份再认领到期只往后挪（`:571`）；写这把键的同一批里释放认领（`release_claim_on`，`:449`、`:670`）；服务起来时认领表从库里重读（`:111`）。身份是客户端追加日志里记的那一个（`reliable_client.rs` 的 `ReliableClient::open` 读日志取 `client_identity`），日志旁有排他锁，同一份日志同时只有一个进程用。今天只有 `register_verdict_vector_in_the_prune_server` 用它（`verdict_store.rs:5443`，租期 `PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS` = 30 秒，`:2395`；Rust 侧另一处调 `claim` 只有 `:2985` 那个包装；Python 客户端没有认领操作码）。

**落地要补的四处**
1. **认领键用判过记号（表 `0x1E`），不用结论块那一行。** `AlreadyConcluded` 是「键本身有数据行」，而「记着红的块下一趟现判」（`crash_amplification.rs:11`）时结论块那一行就在；认领结论块键，重判红块永远认不到。记号与结论块同一批写下去（`store_conclusion_block_with_the_abandoned_roots_it_may_name`，`verdict_store.rs:4623`，那里已经是一个 `LibraryWriteBatch` 一帧写），同一批里删掉待判行（空值是删除记号）。结论相同时 `put_a_conclusion_block_into_the_batch` 交回 `AlreadyStoredIdentically`、调用方直接返回（`:4684`）：这时记号与删待判行由新函数 `record_the_judged_marker_and_retire_the_pending_row` 另写一帧；崩溃落在两帧之间不丢块（记号缺 = 再被认领、结论相同、补记号）。轮次：记红要重判与强制现判（格 6）用轮次加一，认领键随之是一把新键，结论块上的替换走已有的 `ReplacedBecauseOneOfTheTwoIsRed`（`verdict_store.rs:1867`）。
2. **认领到之后先点读一次结论块**（`read_conclusion_block`，`verdict_store.rs:4889`）：已有不红结论（别的路径判的：CPU 判器那一路、过渡期的旧进程、别台）就补记号、删待判行、不判。模型把「认领键有行」与「块已有结论」当成同一件事，靠这一步与第 1 条一起成立。
3. **续租线程与进度看门狗。** 每个工作线程（一张卡一个，各开自己的 `ReliableClient`、各用自己的日志文件，身份 = 台标签与卡序号稳定）起一条续租线程，租期 30 秒（`verdict_store.rs:2395` 同值）、每 10 秒对手里的认领键再认领一次（客户端每次发出去都给整段租期，`reliable_client.rs` 文件头「认领」；服务端到期只往后挪，`committer.rs:571`）。续租只在判器线程有进度时继续：判器每次提交返回就打一次进度；两倍 `LONGEST_WAIT_FOR_THE_CARD_SECONDS`（300 秒，`crash_judge_dispatch.rs:59`）没有进度就停止续租、丢掉这一块、重建这张卡的判器。没有这一条，进程活着而卡挂住时租期永不到点，块卡死（模型变体 `broken-lease-never-expires` 的真实形态）。
4. **一个服务进程一台一份，工作线程之间用各自身份互斥。** 同一个身份「自己持有」不挡自己（`committer.rs:680`），所以几个工作线程共用一个客户端会各自认领到同一块；每张卡一个客户端与一份日志。

**三种交错各走一遍（对应模型变体）**
| 交错 | 走法 | 落库结果 |
|---|---|---|
| 两台共五张卡同时取 | 各卡扫 `1D ‖ V` 一页，对想取的那一块发一次认领（一帧至多几个键；只认领马上要判的 1 块加预取 1 块，认多了的块会白占租期）；提交线程串行判，同一键只有一个 `Claimed`，别的得 `HeldByOther` 去看下一块 | 变体 1、4：同块同时判 0、重复写 0、每块一份结论 |
| 服务中途死（一台的卡线程全死） | 认领行还在库里，到期放出；同一身份重起的服务（日志在）头一次认领就是「自己持有」，立刻接着判；不重起的，别台或别卡 30 秒内接手；没确认的结论写帧留在追加日志里，重起后重放，服务按（身份，请求号）去重（`committer.rs:5`） | 变体 1：死了不回来也能接手（`blocked_without_restart` 0）；丢的至多每卡手里那一块的算力 |
| 租期到了别人接手（原持有者没死、只是停着） | 原持有者之后写：别人已写过则库里「已存着逐字节相同的」什么都不写；两份都绿而不同报错、有红的整份换（`verdict_store.rs:1859`–`1871`） | 变体 2、3、5：重复判与重复写可达，结论行仍只一份；有续租纪律（变体 1、4）时不可达 |

**满足 `.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」那一条吗**（第 58 行「一块判完才落盘，写入是原子的；被打断后重跑同一条命令就是续跑，只补没判的。」）：原子——结论块、违例、恢复字、计数向量、块属性、记号、删待判行同一帧（`verdict_store.rs:8`「一帧写把块与它的违例一起原子地写下去」）。续跑——被打断的是前半段时，重跑同一条命令只对没有不红结论的块写待判行（`pending_blocks` 的谓词），已判的不进清单；被打断的是服务时，库里的认领与待判行就是续点，重起后按租期接着取。这一条的字面「重跑同一条命令」在服务形态下要读成「重跑录入那一趟，或服务重起」，规则文字要改（格 9）。

**什么现象会推翻**：(a) 模型任一变体 `result=UNEXPECTED`（先怀疑我的推理，再怀疑实现）；(b) 同一身份被两个进程同时用（日志文件拷贝、两个服务实例抢一个单元）——互斥退化成「自己持有」；(c) 两台判同一块写出不同的绿——不是协议问题，是判器不确定，库会报 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`（`verdict_store.rs:1907`），这条路径要在服务里当告警、不当可重试；(d) 记号与结论块没进同一帧又不做第 2 条点读，会出现「重判已判块」。

## 格 3　版本对账

**判定：成立（今天的键按「编进二进制的那一份」取，天然适合对账），但有一个先决条件推不出：测试二进制与服务二进制编出同一把 V。** 下面先答协议，再答先决条件。

**今天键从哪来（逐处）**
1. `crates/singlefs-verification/build.rs`：`main` 读 `judging_units_entries::JUDGING_UNITS_ENTRY_GROUPS` 与 `OBSERVER_ENTRIES`（`build.rs:142` 起 `requests`），对每组经 `entry_group_key_not_following_the_listed_units` 算键（`:185` 起），连同工具链行（`toolchain_lines_of_the_build`，`:131`：`rustc -vV`、`cargo -V`、`TARGET`、`CARGO_ENCODED_RUSTFLAGS`、profile 生效值——profile 取仓根 `Cargo.toml`，`:125`–`:129`）与 `Cargo.lock`（只算闭包真引到的依赖版本）写进 `OUT_DIR/judging_units_keys.tsv`（`:227`–`:236`）。
2. `crash_identity.rs`：`include_str!` 把那张表编进二进制（`:568`，`COMPILED_JUDGING_UNITS_KEYS_TEXT`），`compiled_judging_units_keys`（`:576`）解析，`judging_units_key`（`:749`）按组名取；跑时不读树。
3. 镜像判定那一把 = 判定定义那一组与 GPU 判器那一组并成：`judgement_units_key_with_the_gpu_judge`（`:825`）、`image_judgement_units_key`（`:857`）；`JUDGING_UNITS_OF_THE_IMAGE_JUDGEMENT_VERSION`（`crash_amplification.rs:21230`）。
4. 取结论版本的那一处：`ConclusionVersionsOfThePlan::of_the_plan_keeping_the_refused_out`（`crash_amplification.rs:21313`），`run_from_environment` 在 `:18940` 调；某组拒算就换成占位、「照判不入库」（`:21336`、`:21360`）。
5. 判器键之外不进键的：调度与批宽度（`judging_units_entries.rs:29`）、在哪张卡判（块属性，`verdict_store.rs:43`）。

**协议（服务报出什么、分发时核什么、对不上怎么处置）**
- 服务起来时：`image_judgement_units_key(toolchain_of_this_process())`（`crash_library.rs:99`）算出 V 与组成清单；**任何一组 `Refused` 或 `NotCompiled` 就拒起、退 78**，不走 `:21336` 那条占位路径（占位是为「照判不入库」的测试进程设计的，服务里「判了不入库」等于白判）。
- 服务每 10 秒写一行状态（表 `0x1F`，格 1）：V 的 32 字节、组成清单的摘要、服务二进制 SHA-256、协议号、库布局号（`verdict_store.rs:1295`，今天是 6）、起来时刻、心跳时刻、本卡预算与在判的流。
- 前半段录入段写完待判行之后读各台的状态行：有一行心跳在 30 秒内、V 等于自己、布局号等于自己的，往下等；一行都没有，**当场**报「本次未跑」，原因分三种写明（服务不在、服务的 V 是 `<前 8 位十六进制>` 而本趟要 `<前 8 位>`、布局号不同），版本不同时用已有的 `changed_units_line_of_the_stale_verdicts`（`crash_amplification.rs:21455`）那一套组成清单比对点名变了的单元。
- 服务分发时只前缀扫 `1D ‖ V自己`；行里不另存 V 的副本给它「照行判」，它写结论一律用自己编进去的 V 拼键，不从行、环境变量或树里取。
- 分发后再核两样：判表摘要读回核 SHA-256（`crash_judge_dispatch.rs:452`），基镜像槽数与判表里的盘数对得上；对不上整块拒判、写一行拒绝记录（表 `0x1F` 的最近原因），不写结论。

**内核改了之后旧服务还在跑那段时间，会不会把旧判法记在新键下：不会。** 旧服务的 V 是它编译那一刻算的（V旧），新前半段写的待判行带 V新，旧服务不扫那个前缀；旧服务写的结论键一律是 V旧，新前半段只读 V新 下的结论，看不到。要出这种事得同时有：服务用了不是自己编进去的 V（协议禁止）或编译时闭包没罩全。后者的真实缺口只有三处，写在这里不当没有：
 (a) 闭包只罩入口登记表点名的函数闭包（`judging_units_entries.rs` 文件头）；服务里「把结论字拆成红项、写块」那段（`store_judged_block`，`crash_amplification.rs:3757`，`store_conclusion_block` 一族）不在键里，改了它判过的块照旧复用——这一点今天就是这样，服务不新增缺口；
 (b) 驱动版本只进块属性不进键（`crash_amplification.rs:21082`）；服务常驻让驱动版本的变化更频繁地与同一把 V 相遇，两台的卡与驱动不同（`crash_judge_dispatch.rs:44`–`46` 写到同一个状态在第二台的 5060 Ti 上算完、在本机同型卡上被驱动掐掉），同块两台判出不同会被库报错（格 2 (c)）；
 (c) 旧服务进程的内核是它启动时编的 WGSL（`include_str!`，`crash_judge_gpu.rs:100`），不会中途换。

**先决条件（推不出，要主 agent 现量）：同一棵树，`cargo test`（测试二进制，前半段）与 `cargo build`（服务）编出的 V 是同一把吗？** 构建脚本把「本包这一次开了哪些特性」与「依赖包在这一次编译里开了哪些特性」都喂进键的输入（`build.rs:5`–`:9` 的说明；`crates/singlefs-core/Cargo.toml:8` 写明验证包的 dev 依赖会打开 `lookup-work-counters`、`test-only-publish-thresholds`）。测试构建开着这两个，服务构建不开；它们只有在 GPU 判器与判定定义两组的闭包真引到这些 cfg 原子时才改键（键只含「闭包真引到的 cfg 原子与它们的真值」，`.claude/rules/verification.md` 第 39 行）。我没有读到闭包里有没有引，所以两种结果都可能：
- 相等：协议照上面走；
- 不等：服务永远等不到匹配的行（活性问题，不是安全问题，对不上的行永远不会被错判）。出路二选一：在 `singlefs-core` 里把这两个 cfg 从 GPU 判器与判定定义的闭包里挪开；或服务进程用 `--tests` 同款特性编（`cargo build --features` 无法打开别包的 dev 依赖特性，所以实际是前一条）。这一步改 `crates/`，要走三方，我不在这一轮里定。
- 判法：主 agent 在改动前后各取一次 V（末尾「要主 agent 跑的命令」第 2 条取装置二进制那一把；测试那一把需要在 `crash_identity` 的测试模块里加一条只打 V 的断言用例，现在没有——`grep -n 'image_judgement_units_key' crates/singlefs-verification/tests/*.rs` 我没跑，派发提示不许我起，列在「推不出的」）。

**服务的 Cargo 形态与键的忠实度**：服务作为仓根 `Cargo.toml` 工作区的成员（`members` 加 `gpu-compute-server`）比像 `prune-server/` 那样自带空 `[workspace]` 好：构建脚本读的是仓根 `Cargo.toml` 的 profile 与仓根 `Cargo.lock`（`build.rs:125`、`:138`），服务若自带 `[profile.release]` 与自己的 `Cargo.lock`，键按仓根的算而二进制按自己的编，两者一漂键就说谎。`prune-server/Cargo.toml` 自带 profile 是因为它不进键；服务进键，所以进工作区。

**什么现象会推翻**：(a) 服务里出现「从待判行取 V 当写结论的键」的代码；(b) 测试二进制与服务二进制的 V 不等而协议没把它报成「本次未跑」；(c) 某组拒算时服务照样起来、照占位写库。

## 格 4　批大小与显存

**判定：成立（式子能给、代入的数与量到的 3843 与 1600–1730 对得上；一半的数没量过，已标）。** 一个重要的读法先说：同卡驻留几条流**占用的显存几乎不涨**，涨的是「每条流分到的批宽」；而一趟的用时几乎不随批宽变（见下），所以窄批是吞吐损失，不是显存紧。

**式子（读自 `crates/singlefs-verification/src/`，符号对到函数）**
- 每态草稿区 w = `SCRATCH_WORDS` × 4 = 2 009 599 × 4 = 8 038 396 字节 = 7.666 MiB（`crash_judge_gpu.rs:988`；一条草稿缓冲装 267 态，`:987` 的说明）。每态的结论缓冲与回读缓冲另 2 × `VERDICT_WORDS` × 4 字节，`VERDICT_WORDS` 是 `crash_judge_gpu.rs:126`–`:199` 那串常量链的末项加一，我数到约三十个字（推的，没取到值），比 w 小四个数量级，下面略去；它在 `mebibytes_a_judge_needs`（`crash_judge_dispatch.rs:500`）里记，在 `states_in_flight_within`（`:480`）里并进驱动那一份。
- D = `MEBIBYTES_LEFT_FOR_THE_DRIVER` = 2560 MiB（`crash_judge_dispatch.rs:36`）：**每个进程在每张卡上一份**，起设备时付；T = 打包判表上卡后的 MiB（`PackedJudgeTables::mebibytes`，向上取整，`crash_judge_dispatch.rs:431`）；B = 这张卡给判器的额度（`GpuCard::memory_budget_mebibytes`，`gpu_unit_checks.rs:1092`：给了额度取额度与空闲里小的，没给取空闲的 3/4）；M = 最大存储绑定（一条草稿缓冲 ⌊M ÷ w⌋ 态，`states_per_scratch_lane`，`:586`），草稿缓冲至多 32 条（`MAXIMUM_SCRATCH_LANES`，`:33`）。
- 一个判器一次派活最多带：n(T) = min( ⌊(B − T − D) · 2^20 ÷ w⌋ , 32 · ⌊M ÷ w⌋ )，至少 1（`states_in_flight_within` → `states_per_dispatch`，`:480`、`:595`）。
- **同卡同进程 k 个判器各带自己的草稿区（今天的 `GpuCrashJudge` 就是这样，草稿缓冲在构造时开，`crash_judge_dispatch.rs:547`）**：D + Σ(Tᵢ + nᵢ·w) ≤ B；各判器同宽时 n = ⌊(B − D − ΣTᵢ) · 2^20 ÷ (k · w)⌋。
- 今天 k 个进程各占一张卡：每个进程各付一份 D，n = ⌊(B − k·D − k·T) · 2^20 ÷ (k · w)⌋。
- 实际一次派活的宽度还受时间封顶：从 `FIRST_DISPATCH_STATES` = 1（`crash_judge_dispatch.rs:61`）起，按最久那次提交的用时加宽或减半，一次提交算到 `LONGEST_DISPATCH_SECONDS` = 0.5 秒为限（`:42`）；上面的 n 是显存给的上界，不是一定取到。

**代入（算术，我用 `python3 -I -c` 现算；B 取空卡可用量 32110 与 15849 MiB，`gpu_memory_ledger.rs:48` 的说明：5090 是 32607 − 497，5060 Ti 是 16311 − 462；T 取慢流那张表 12 091 358 字 = 46.12 MiB，每条流都按这个最坏表算；M 取 2 GiB 时车道上限 8544 态，远大于显存给的数，不是约束）**

```
per_state_bytes 8038396 MiB 7.666011810302734
table MiB 46.12487030029297
32G 1 each-own-scratch 3848 total 3848 | shared-scratch 3848 | today k processes 3848 total 3848
32G 2 each-own-scratch 1921 total 3842 | shared-scratch 3842 | today k processes 1754 total 3508
32G 4 each-own-scratch 957 total 3828 | shared-scratch 3830 | today k processes 707 total 2828
16G 1 each-own-scratch 1727 total 1727 | shared-scratch 1727 | today k processes 1727 total 1727
16G 2 each-own-scratch 860 total 1720 | shared-scratch 1721 | today k processes 693 total 1386
16G 4 each-own-scratch 427 total 1708 | shared-scratch 1709 | today k processes 176 total 704
```
读法：每行第一个数是 k 条流时**每条的批宽**（态），total 是 k 条合计。与量的对：32G 一条 3848 对量到的 3843（差 5 态，约 0.1%，差的原因没查：推的，是量那一趟的额度比空卡可用量略小）；16G 一条 1727 落在量到的 1600–1730 里（`records/2026-11-17-崩溃判器与测试流的效率优化.md` 第 32 行）。k 条同进程合计只比一条少 6 到 20 态（表占的），今天 k 个进程各付 D 时 16G 卡上放四条合计只剩 704 态（占一条的 41%）——这是常驻服务一个进程一张卡付一份 D 的直接收益。「共用一份草稿区」要改 `GpuCrashJudge` 构造（草稿区与表分开），会换 V（格 10），合计只多几态，不值。

**吞吐为什么随宽度走（这一格最关键的读法）**：慢流一趟的用时几乎不随批宽变（走读 1602 态 0.3347 秒、3843 态 0.3520 秒，`records/2026-11-17-崩溃判器与测试流的效率优化.md` 第 49–57 行，量的）。一批的几趟内核各是一次提交、阻塞等完（`judge_one_dispatch`，`crash_judge_dispatch.rs:1232`），一个设备只取一条提交队列（背景材料「没量的」一节读 wgpu-hal 得，推的）。所以 k 个判器轮流各带 n/k 态，同一时间卡上只有一个在算：每态的卡时间约是独占的 k 倍。这个读法与「重叠合计吞吐可能掉到 0.5–0.7 倍」同向，但都是推的，探针没出数（格 5）。

**k = 1、2、4 的落地选择（第一版）**：卡上同一时刻只有一个判器带草稿区（k = 1，批宽取 n(T)）；换流时丢掉旧判器、新建一个（旧判器的草稿区一并释放；管线按源码摘要在设备上编一次留着，`gpu_unit_checks.rs:1134` 的 `pipelines_compiled_once_on_this_device`，不重编）。代价：新判器从宽度 1 重新校准（`FIRST_DISPATCH_STATES` = 1），起设备、传判表、建判器各花多少没量过。同一条流的块连着判（亲和），不在流之间每块切换。k ≥ 2 驻留要等格 5 的探针出数；若出数是「合计高于独占」，才为它拆草稿区与表（改 `GpuCrashJudge`，换 V）。

**给定状态表那一路**（`crash_amplification.rs:20480`–`:20590` 一段，今天借卡走 `lease_the_first_card_with_room_waiting_for_the_other_threads`，`gpu_unit_checks.rs:2752`）：它在判器里换了旋钮栏（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`，`crash_judge_gpu.rs:1163`），源码摘要不同（`kernel_sources_digest`，`crash_judge_dispatch.rs:517`），所以要另编一组管线；背景材料说是 9 条，我只核到普通流一个判器有 11 条（`GpuCrashJudge` 的 11 个 `wgpu::ComputePipeline` 字段，`crash_judge_dispatch.rs:533`–`:544`）。显存记账：D 不变（一进程一份），多出的是 9 条管线在驱动里占的 p：设备上按摘要的缓存**不淘汰**（`compiled_once_for_the_source_digest`，`gpu_unit_checks.rs:1134` 往下），常驻服务里每个不同的旋钮栏摘要都永远留一组。所以式子是 D' = D + p · (设备上留着的旋钮栏摘要数)，p **没量过**，当前不能给数；要给服务加两样：(1) 旋钮栏摘要的上限与淘汰（超了丢整组、重编）；(2) 起设备后与每编一组后读 `nvidia-smi` 的占用差，写进状态行（表 `0x1F`），p 就量出来了。给定状态表整张判一次、不分块（`:20570` 的 `judge.judge_states(0..packed.state_count())`），待判行种类 2 一行一张表，状态数是整张表的；批宽同上式、n(T) 不封顶块长。

**今天的「撞显存报本次未跑」那条路，服务里对应什么**（今天：此刻借不到就排队等这个进程别的线程还卡，没有可等的才报；空着时也放不下是判红，`gpu_memory_ledger.rs` 文件头「借的要比这张卡空着时能借的…还大，等多久都放不下，调用方判红」；卡上 wgpu 报 Out of Memory 同样报本次未跑，`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 文件头第 79 行）。服务里分三层：
1. 这条流的表加一态草稿区，**空卡也放不下**（n(T) 在 B = 空卡可用量时 < 1）：这张卡永远不接这条流；全部卡都这样就在状态行写「这条流没有卡放得下：要 X MiB、最大空卡额度 Y」，前半段读到这一行报**判红**（配了 GPU 就一定用 GPU，不退 CPU）。
2. 空着放得下、此刻放不下（别的租户占着、本进程另一个判器占着）：不认领，退让一个 `retry` 间隔再看，状态行写「显存不够，等」；前半段等到心跳还新鲜就继续等，等不到才报**本次未跑**，原因写显存不够（`GPU_MEMORY_SHORT_REASON_PREFIX` 那一族，`gpu_memory_ledger.rs` 文件头）。
3. 判一块时 wgpu 报 Out of Memory（`on_the_card_catching_out_of_memory`，`crash_amplification.rs:3980` 前后）：这一块不交结论、停止续租（格 2 第 3 条）、判器丢掉重建并把批宽减半；减到 1 还报，本卡对这条流记「放不下」（回到 1）。已经入库的块不受影响。

**什么现象会推翻**：(a) 同卡两个判器同时在算且合计吞吐高于独占（探针结论）——则 k ≥ 2 驻留值得做；(b) 驱动那一份 D 对同进程的第二个判器还要再付一部分（今天只量过 5 档草稿区下一个进程的 2326 MiB，`crash_judge_dispatch.rs:35`，没量过两个判器同进程）；(c) 空卡可用量与判器实际能借的差得比 5 态大。

## 格 5　脉冲与水位线

**判定：水位线与一波怎么排——成立（可落地）；「重叠发值不值得」——推不出（探针没出数），按两种结果各写。**

**水位线量什么：待判工作量，按卡秒算，不按态数算。** 理由是现读的：不同流的态每秒差三十五倍（慢流 32G 卡 1350–1580 态每秒，最快一条 52946 态每秒，`records/2026-11-17-崩溃判器与测试流的效率优化.md` 第 3.1 节，量的），一个「态数的水位」对慢流太低、对快流太高。CPU 一侧不逐态产出数据（状态在卡上照序号现算，`crash_judge_gpu.rs:1002` `dispatch_words`），一条流录完全部块写完待判行，水位线是**台阶**：一条流到达，待判量一次跳上去，不是连续涨。量的东西三个：
1. W = Σ（待判行的状态数 ÷ 这条流在这类卡上的态每秒 R）。R 由服务自己量：每判完一块，按（判表摘要，卡型）滚动记态每秒，写进状态行（表 `0x1F`）；没量过的流按 1500 态每秒记（偏慢，W 偏大，宁可早起一波；这个缺省是我取的，推的）。待判行从 `1D ‖ V自己` 前缀扫一页一页读（一行约 200 字节，最大一条流 25 609 行约 5 MB，一次轮询的量，有线直连不当瓶颈，前提 2）。
2. 年龄 A = 现在 − 最老待判行的录入时刻（行里存，格 1）。
3. 是否有轮次 > 0 的行（强制现判、记红重判）：有就不等水位线，下一次轮询马上起。

**一波由什么触发**：W ≥ H（高水位）或 A ≥ A最大 或 有轮次 > 0 的行。取 H = 每张卡 20 秒的活（H 按卡数乘：五张卡 100 卡秒），A最大 = 60 秒，低水位 L = 0（判到没有可认领的才收）。H 的来由：每一对（卡，流）的固定开销 f（读回 23–47 MiB 的判表 0.074–0.647 秒，量的；上卡、建判器、批宽从 1 起校准，没量过，先按 1 秒估）要摊到工作量里不超过 5%：H ≥ 20 f。20 倍、1 秒、60 秒这三个都是我取的，要看服务头几天的状态行再定（推的）。空闲时每 5 秒轮询一次；波里各卡自己拉，不再轮询触发。收波后判器留 60 秒再丢（同一条流的后续块来得及接着用，批宽校准不丢），丢了释放这张卡的草稿区。

**一波里几条流怎么排先后、怎么分卡**
1. 流的先后：按最老录入时刻先来先判；不足一块的小流插队（它们判完的固定开销占比最大，早判早腾出前半段）。
2. 分卡：不由调度员派，各卡自己拉（格 2）：卡先接着判自己手里那条流的块（亲和，省上卡与校准）；这条流没有可认领的块了，才看队头那条流里放得下的（格 4 第 1 层：这条流的表加一态草稿区在这张卡上放得下），认领一块、预取一块。流块多于卡数时几张卡一起判同一条流（每张各上自己的一份表），块少于卡数时多出的卡去判队里下一条流。
3. 大小卡并存（5090 约 3848 态一批，5060 Ti 约 1727，格 4）：拉的方式让大卡自然多拿；最后一块的拖尾是整块 65536 态在慢卡上的用时，没量过；到时再看要不要「剩的块比空闲卡少时，慢卡不再认领」。
4. 判表先于块读：服务在卡忙着判时就把下一条流的判表从库读回、核摘要（放主机内存，46 MiB 量级），换流时只剩上卡。

**探针（交替重叠的批在一张卡上合计吞吐多还是少）出数之后怎么排**——判法：k = 1、2、4 各跑至少三遍（单次观测不算数），记每张卡的合计态每秒 S(k)，r(k) = S(k) ÷ S(1)；取 k* = r 最大的那个 k，r(k*) ≥ 1.1 算「高于独占」，否则算「不高于独占」（1.1 是我取的噪声余量，推的）。
- **合计吞吐高于独占**：每张卡驻留 k* 个判器、起 k* 个工作线程，每个线程自己的客户端与身份（`台:卡:槽`），各自认领各自的块——认领的粒度、租期、续租都不变，每个判器各占自己的草稿区（`one_dispatch_at_a_time` 是每个判器一把，`crash_judge_dispatch.rs:568`，所以不同判器不互等）；批宽按格 4 的 n = ⌊(B − D − ΣT) ÷ (k*·w)⌋，一波选队头的 k* 条不同的流同时判（队里只有一条流时 k* 个判器各上一份同一张表）；水位线 H 乘 k*（同一段时间里一张卡吞进去的活多了）。一个进程一份 D 让这一档的显存代价只有表与草稿区本身（格 4 的表）。
- **合计吞吐不高于独占**：每张卡恒一个判器、一个工作线程，批宽取满（k = 1）；一波就是「每张卡逐条流判下去」，换流靠预取的判表。别的流的判器不驻留（它们的草稿区一份 7.666 MiB × n 吃显存而无收益）。水位线 H 不变。

**什么现象会推翻**：(a) 服务头几天的状态行里 R 随批宽不是正比（说明批宽线性这个读法在更宽处不成立）；(b) 一波里同一条流的几个块被几张卡各自上表、各自校准，固定开销占比高于 5%——则要给「同一条流只给几张卡」加一个上限；(c) 小流插队让大流饿死（A 最大只管最老的，不管大流被插队的次数）。

## 格 6　强制现判与哨兵

**判定：成立——今天的做法在服务形态下会变成「空判」，必须换；给出五种做法，各写拦得住与拦不住的。**

**先说今天的做法为什么走不通**：强制现判与哨兵都是**进程内**的账。`also_force_these_conclusion_blocks_to_be_judged_again`（`verdict_store.rs:3155`）让「这个进程里问复用一律答不上」；判完看「这个进程交来存过的结论块清单」（`conclusion_blocks_offered_since`，`verdict_store.rs:3147`，由 `store_conclusion_block…` 在 `:4670` 记进 `conclusion_blocks_offered_in_this_process`）：哨兵不在清单里、强制块在清单里才算过（`ReuseSentinels::problems_after_judging`，`crash_amplification.rs:3577`；`forced_block_was_judged_here`，`:3688`）。判的一方换成服务之后，前半段进程**一块都不会交来存**：强制块判定永远是「没现判」→ 恒红；哨兵「不许被现判」→ 恒绿，而且是空绿——就是 `.claude/rules/verification.md` 第 41 行要防的「跑了、全绿、但已经不做事」。所以这两处必须换成读服务的凭据。

**做法一：轮次加一 + 判过记号上的凭据行（推荐，和格 1、格 2 一起落）。** 前半段对强制块（与要重判的红块）写一行待判行，轮次 = 现有最大轮次加一，行里带这一趟的随机数；服务认领这一把新的记号键（所以不可能因「已有结论」而复用）、真判、写回结论，并在同一批里把一行凭据写进记号值：记号值不再是 1 字节，而是（随机数、判完时刻、卡型、驱动版本、这一块用的提交次数与卡上等的总秒数——`GpuJudgeTimeSpent`，`crash_judge_dispatch.rs:573` 已有这几个计数——、判表摘要、结论摘要）。前半段判完读记号：随机数对得上、提交次数 > 0、结论摘要与开跑前取的那一份判定向量逐字相同（`ConclusionSpelledOutInVerdictVectors::of_the_stored_block`，`crash_amplification.rs:3282`，按判定向量本身比，库换过编号也对）才算。拦得住：服务根本没判（复用了、认领被短路）；服务判了但结果与库里不同（判定不稳）；旧版本服务（V 不同，写在别的键下，读不到）。拦不住：服务编造凭据（计数写假）——只能靠它是同一份代码；强制块本身判对判错（它判的是同一把判器，只证「现判过且一样」，不证对）。
**做法二：版本命名空间。** 强制现判的块不用 V 而用「V 拼随机数再取 SHA-256」的一把临时版本 Vn 写结论：待判行里 V 照旧（服务认它），行里另带随机数，服务判完把结论写在 Vn 下（不是 V 下），前半段读 Vn 下的结论与开跑前 V 下那一份逐字比，比完删掉 Vn 下这一块。拦得住：复用（Vn 下永远没有旧结论）；与库里旧结论的逐字比（两份并存）。拦不住：服务把「V 下已有结论」直接抄到 Vn 下（代码写出这条路才会，协议不保证）；要多写一份垃圾块、要清理；Vn 下的行混进键空间，扫表的程序（`crash_library_reader`）会多数一种版本。比做法一多了清理与键空间污染，做法一用轮次已经够。
**做法三：先删后判。** 前半段把强制块的结论行删掉（还留一份在内存里），再写待判行。拦得住：复用。拦不住：删到判回之间库里没有这一块的结论——服务死在中间，这一块只剩「没结论」，下一趟当没判过，丢了「记红」的记录（格 2 第 1 条本来就要避免）；而且前半段进程死在删与比之间时开跑前那一份就丢了。不推荐。
**做法四：服务自己抽查（持续审计）。** 服务每判完 N 块，从自己 V 下已有结论的块里随机抽一块放回「轮次加一」的行、自己判、自己比，不一致写告警行（表 `0x1F`）。拦得住：旧服务写下的结论与新判器不一致、驱动更新、判器不稳（与前半段这一趟无关的长期漂移）。拦不住：前半段这一趟「它抽中的那一块」有没有现判（不绑这一趟）；也拦不住服务代码本身一致地错。适合与做法一并用，不替它。
**做法五：CPU 判器对一块。** 强制块再交 CPU 判器（`crash::judge_crash_image`）判一遍与服务比。拦得住：GPU 判器与 CPU 参照不一致（这是对拍用例该做的）。拦不住：「现判没现判」；而且用户 2026-10-11 定了「CPU 不许在一段任务上又跑数据又判一致」（背景材料「用户定的（2026-10-11…）」两段分开一行）——这一趟里 CPU 判同一块与那一条冲突。不选。

**哨兵（键相同、库里有不红结论、应当被复用的块）**：服务不抽哨兵；前半段开跑前抽，抽完不把它们写进待判行（本来就不会写：待判行只写没有不红结论的块）。判完证「哨兵没被现判」：读哨兵块的判过记号，记号的轮次与凭据日期早于这一趟（凭据里的随机数不是这一趟的），再读服务的「本进程判过的块计数」那一栏与这一趟前后之差。拦得住：服务对已有结论的块又判了（复用失灵）。拦不住：服务用别的路径悄悄判了又写回同一份字节（`AlreadyStoredIdentically` 不改行，凭据行不更新则看不见）——所以服务每次现判都要更新记号值，包括结论相同的那一次。

**什么现象会推翻**：(a) 服务对 `AlreadyStoredIdentically` 的块不更新记号凭据（则做法一对「现判了但结论相同」看不见）；(b) 强制块的轮次加一与别的前半段同时抢同一轮次（两个前半段对同一块同时要强制）——写待判行时轮次取值要在一帧里读写，或用随机数进键。

## 格 7　与统一任务服务的分工

**判定：成立（能分干净，但「卡」这一件要在两个服务之间划界，没有现成的共管机制）。** 读了 `task-server/task_server.py` 文件头（①对账与卡回收、②GPU 与按需领卡、②CPU 闸、③起）与 `prune-server/README.md`。要点：任务服务的整套卡账（`busy_cards`、领卡、清场、回收、停用）都只认**一次尝试**——领卡「只认这一次尝试自己」（`task_server.py` 文件头「领卡只认这一次尝试自己，别的进程、别的任务领不到」），清场包装只认服务套的那一层（`scripts/crash/gpu-quiesce-run.sh` 第 12 行起）。常驻服务不是任务，进不了这套账。所以第一版用**按卡划界**：多机配置加一个键写明哪几张卡归服务，任务服务的卡先后里不再有它们（做法见表末）。

| 件 | 今天在哪做 | 服务常驻之后归谁 | 要改哪一处 |
|---|---|---|---|
| 卡的清场（停掉占卡的别的服务，回读） | `scripts/crash/gpu-quiesce-run.sh` 整份（读多机配置 `QUIESCE_STOP_COMMAND` 与 `QUIESCE_STOPPED_CHECK_COMMAND`；登记 `holders/` 让几趟同时用卡的互认，回读跳过别的活登记的进程）；服务套它：`task_server_lib.py:2006`（`GPU_QUIESCE`）、`:2036`（准入变量）；领卡那一回另起线程清：`task_server.py:8633`（`quiesce_for_card_lease`） | **gpu-compute-server 自己，起来时清一次**：它的单元 `ExecStart` 经 `gpu-quiesce-run.sh --` 套自己，清场登记在服务活着的整段时间里持有，别的清场认它是「别的活着的登记」，回读不把它的进程当外人 | `gpu-quiesce-run.sh` 的准入只认 `GPU_QUIESCE_RUN_ADMITTED_BY_TASK_SERVER`（第 12 行、第 102 行）：再认一个 `GPU_QUIESCE_RUN_ADMITTED_BY_GPU_COMPUTE_SERVER`，由 `gpu-compute-server/control start` 设；`task_server_lib.py:2402` 那张「只由 task-server 设」的变量表与 `--selftest` 的弄坏开关同步；清场不复原的原则不变（占着卡的私有服务这时被停，服务停了也不复原，用户手动复原） |
| 卡的分配（哪件活上哪张卡、一张卡只给一件） | 起跑就分：`place_on_card`（`task_server.py:11003`）；按需领：`grant_card_lease`（`:8578`）、`card_lease_choice`（`:8572`）、领卡线程 `card_lease_loop`（`:8366`）；占卡账 `busy_cards`（`:8720`）；原则「一张卡只给一条」在文件头 | **划界**：多机配置 `GPU_COMPUTE_SERVER_CARDS`（`<台标签>:<序号>:<额度>`，写法同 `GPU_CARDS_BY_PRIORITY`）点名的卡归服务；它们不得出现在 `GPU_CARDS_BY_PRIORITY` 与 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED` 里，任务服务就不会分到。服务内部「哪块上哪张卡」是各卡自己拉（格 5） | `scripts/crash/crash-point-replay-shard-configuration-check.sh` 认新键与「两处不相交」；`multi-host.env.example` 加键与说明；`crash_library`/`gpu_card_configuration.rs` 的 `cards_by_priority_from`（今天读 `GPU_CARDS_BY_PRIORITY`）不动，服务另读 `GPU_COMPUTE_SERVER_CARDS`；`task_server.py` 不改。代价：两台各只有 1–2 张卡时，划给服务的卡任务就用不上了（GC、产品 GPU 对拍等仍要卡的任务只剩余下的卡）；动态让卡（服务让出一张、任务拿走）是第二版，要加「卡归属」行（表 `0x1F`）与任务服务读它，不在第一版 |
| 卡坏的发现 | 清场前 `scripts/hosts/gpu-card-health.py --cards`（`gpu-quiesce-run.sh` 「查卡」一段）；跑完卡回收读 `nvidia-smi` 与内核日志里的 Xid（`reclaim_cards`，`task_server.py:7163`；读法 `task_server_lib.py:4525` 的 `card_gc_readings`）；判坏就停用 `CARD_QUARANTINE_SECONDS`（`quarantine_cards` `:3827`、`cards_in_service` `:4097`）并写 `card-fault` 告警，主 agent 弹窗请用户重置 | **gpu-compute-server 自己**：起来时 `gpu-card-health.py` 查一次（经清场包装自动）；运行中每 5 分钟再查，并在 wgpu 报设备丢失 / 判一块的提交超时 / 内核日志出 Xid 时当场查；坏了就停用这张卡（它的认领自然到期放出，格 2），状态行写 `fault` 与依据 | 服务里新加 `card_health` 模块（子进程调 `gpu-card-health.py`，不另写判法）；告警出口：任务服务的消息发给提交任务的会话，服务没有会话——照 `prune-server/control watch` 的做法加 `gpu-compute-server/control watch`（前台循环，卡坏或服务不通即退 1，主 agent 用 run_in_background 起，退了就弹窗请用户重置）；这是服务与任务的一个**真缺口**：任务服务今天对停用的卡有 `CARD_QUARANTINE_SECONDS` 到点自动解除，服务这边要不要自动解除（还坏会再判出来）待用户定 |
| CPU：服务占的线程 | 任务服务的 CPU 闸 `cpu_judgement`（`task_server.py:10609`）：max(这台此刻的负载, 服务在这台在跑的尝试的线程之和 `running_threads`，`:10728`) + 这一条的线程 ≤ 核数 × `CPU_OVERSUBSCRIPTION`（3.4375，`:994`；32 核的台是 110）。量到过负载一百上下时一条单线程的活三分之二的时间在排队（背景材料） | **两件事分开**：(1) 记账：服务的线程数是个常驻量，由 `GPU_COMPUTE_SERVER_THREADS_BY_HOST`（「按台的三个键」同款写法）声明；(2) 优先级：服务的 CPU 线程（解码结论字、读判表、写库）必须在高负载下仍拿得到核，不然卡空等 | (1) `task_server.py:10728` 之后的 `running_threads` 加上本台声明的服务线程，`cpu_judgement` 随之收紧；`scripts/hosts/multi_host_addresses.py` 的 `by_host_values` 与配置判法认新键。(2) **推不出**用什么办法：给服务单元设 `CPUWeight=`（要用户级 systemd 的 `cpu` 控制器已委派，没查），或让任务单元起跑时带 `Nice=`（任务服务今天起尝试的 `systemd-run` 参数里我没找到 `Nice`：`grep -n 'Nice=\|--nice' task-server/task_server.py task-server/task_server_lib.py | wc -l` 数出 0）；两条都要主 agent 在目标机器上现量，列在「推不出的」 |
| 内存：服务占的内存 | 任务服务的内存记账：`room_by_the_formula`（`task_server.py:9803`）= 配置的总上限（`SLICE_MEMORY_CAP_BY_HOST`）− 余量 − 在跑的记账之和；余量是 slice 里「回收不掉、又不归任何在跑任务」的一截，由服务每次现量（`multi-host.env.example` 对 `SLICE_MEMORY_CAP_BY_HOST` 的说明） | 服务放在**自己的 slice**（`MemoryMax` 固定，不进任务的 slice，免得任务撞上限时把服务一起杀）；它占的内存不从任务的账里现量扣，而是配置里声明 | `GPU_COMPUTE_SERVER_MEMORY_BY_HOST` 声明；`room_by_the_formula` 的总上限减掉它（或把总上限的配置值直接写小，零代码）。服务里最大的主机内存用户是驱动编内核（「冷编一套判器内核要几分钟、驱动编的时候吃几 GiB 主机内存」，`gpu_unit_checks.rs:1127` 的说明；今天每片任务的编译峰值约 9 GiB，是整个包的编译，不是这个）；驱动内存峰值没量过 |
| 服务怎么起与怎么停 | 任务服务：`task-server/control`；判定库服务：`prune-server/control`（`install`、`swap-binary`、`start`、`stop`、`check`、`watch`）与单元模板 `prune-server/singlefs-prune-server.service.in`（`Restart=always`、不 `enable`、用户定「开机不自启动。 但是启动后要有恢复能力。」） | gpu-compute-server 自己：`gpu-compute-server/control`（同一组子命令）与 `singlefs-gpu-compute-server.service.in` | 新建 `gpu-compute-server/{Cargo.toml,control,README.md,*.service.in,src/}`（工作区成员，格 3）、`scripts/gpu-compute-server/` 放客户端（读状态行、写待判行的 Python 一侧：门禁脚本要读「服务在不在、版本对不对」，不起 cargo）。`stop` 不宣布停服务：SIGTERM 时不再认领新块、判完手里的块或到 120 秒（`TimeoutStopSec`）就退，没写的块认领到期放出（格 2）。**开机要不要自起**照 `prune-server` 的先例是「不自起」，这是用户为 prune-server 定的，对新服务没有定过：待用户定 |
| 编译与二进制 | 编译目录归尝试、不跨趟（`.claude/rules/test-script-discipline.md` 第 65 行）；`prune-server/README.md`「装」第 1 步：经 `task-server/task submit` 编，同一条命令里把二进制拷到共享根下 | 二进制归服务：编一次、装到每台的 `~/.local/share/singlefs/gpu-compute-server/bin/`，回读 SHA-256 | 规则第 65 行的「编译目录」不含它（它是产物不是编译目录），但那一句「按键存结论的判定库与外部依赖的预编库不是编译目录，照留」要再加「常驻服务装好的二进制」（格 9） |

**关于「按卡划界」的取舍**：不划界、让服务像任务一样向任务服务领卡，要给任务服务加「常驻的、没有尝试的持卡者」这一类，改动落在 `busy_cards`、`leased_card_keys`、重启认回（`state.json`）、卡回收与对账五处，且用户已答「gpu独立判断不是一个task 他是一个独立的服务。和task不是一个东西」；划界只动配置与配置判法。所以第一版选划界。

## 格 8　多机

**判定：成立。服务每台一个（每台有卡的机器一个进程），待判清单与结论只有一份、在 prune-server 里。**

- 为什么不是「一共一个」：判的是本机的 wgpu 设备（`GpuCard`，`gpu_unit_checks.rs`），进程必须在卡所在的那一台；一个进程管两台的卡要经 ssh 或另写远程协议，等于每台各放一个工作进程再加一个调度员，而待判清单在库里、各卡自己拉（格 2、格 5），调度员没事可做。所以「每台一个」就是「一共一个服务」的物理形态：队列是库里的行，工作进程每台各一个。
- 两台的卡都取得到活：两台的服务各读同一份待判清单。本机的服务经 unix socket 连 prune-server，第二台的服务经多机配置 `PRUNE_SERVER_TCP_ADDRESS` 的 TCP 连（`scripts/prune-server/prune_server_client.py` 文件头「连哪一处」；Rust 侧 `crash_library::prune_server_route_of_the_configuration`，`crash_library.rs:347`）。身份 = 台标签与卡序号，日志放各自本机的状态目录（日志文件旁有排他锁，不放共享根）。
- 判出来的结果两台都读得到：结论落在同一份库里，不拷库、不取回（`verification-build.md` 已有这条）；前半段在哪台跑都读同一份。运行记录（服务状态摘要）放共享根（`NFS_SHARED_ROOTS` 点名的），不经 `~`。
- 两台二进制要同一份：服务二进制装在共享根下一个目录里，两台 `control install --binary` 从那里拷、回读 SHA-256（照 `prune-server/README.md`「装」第 1–3 步）。两台二进制不同、V 不同时各取各自的 V 那一前缀（格 3），互不污染，只是各判各的流；状态行把两台的 V 都报出来，前半段读到「有活的服务但没有一台的 V 对得上」就报本次未跑。
- 后果与它的另一面：(a) 第二台的驱动与卡型不同（5060 Ti，另一版驱动，`crash_judge_dispatch.rs:44`–`46`），同一块两台各判一遍是个免费的跨驱动一致性检查——库对两份都绿而不同报错（`verdict_store.rs:1900`–`1907`），所以服务遇到这个错不重试、写告警行；(b) 内核冷编每台每种卡各一次（量的：只有走读内核变 157.8 秒、缓存全空 182.6 秒、命中 1.9 秒，`records/2026-11-17-崩溃判器与测试流的效率优化.md` 第 132–135 行），驱动缓存在每台家目录下按卡分区，所以换判器内核之后两台各付一次，服务起来时的预编（编一次再开始认领）要做；(c) prune-server 在一台上（本机），它停了两台的服务都停认领：服务不退出（单元 `Restart=always` 只管进程死），按容错客户端的退避重连（缺省 120 秒一轮，`prune-server/README.md` 退出码表），一轮没连上就在本机写日志、继续重试，不退 75 让单元把它拉起再连；(d) 配置只来自 `multi-host.env`：两台读同一份，`GPU_COMPUTE_SERVER_CARDS` 里地址是自己的那几项就是自己的卡，不写死主机名。
- 两台都要用 `ENABLE_ACROSS_MACHINES=1` 吗：服务不看它。它决定的是前半段分片（CPU 分份）与 `multi-host-run.sh --peer`，与卡一侧「各台自己的卡自己拉」无关；但服务是否起在第二台，由 `GPU_COMPUTE_SERVER_CARDS` 里有没有第二台的卡定。

**什么现象会推翻**：(a) 第二台连 prune-server 的往返在 30 秒租期里不稳（TCP 一次认领与续租超时）——则租期要放大；(b) 两台服务的 V 因编译环境差异不同（格 3 的先决条件）——则一台等于闲着；(c) `GPU_COMPUTE_SERVER_CARDS` 与 `GPU_CARDS_BY_PRIORITY` 交叉没被配置判法拦住，两个服务同用一张卡，显存账各记各的（`gpu_memory_ledger.rs` 文件头「账不防的：两个进程同一刻读到同一份空闲、一起借超了」）。

## 格 9　要改的规则条文（只列，这一轮不改）

每条：原文（文件:行）→ 改成什么。引文是整行里的一段，行号都是现读的。

1. 前提 4 之一「编译目录与二进制不跨趟共用」：`.claude/rules/test-script-discipline.md:65`「编译目录归这一趟：每一趟新建、跑完删（成功、失败、被打断都删），不跨趟、跨轮、跨会话、跨脚本共用，也不经链接或环境变量指到别处的编译目录。按键存结论的判定库与外部依赖的预编库不是编译目录，照留。」→ 末句改成「按键存结论的判定库、外部依赖的预编库与常驻服务（prune-server、gpu-compute-server）装好的二进制不是编译目录，照留；服务二进制的编译目录仍归编它的那一趟，装好之后删」。判卡一侧的判不再有「每片编整个包」这件事（背景材料：每片平均 162 秒、峰值约 9 GiB），所以这条里「判」那一半不再约束它。
2. 前提 4 之二「判定库里只放剪枝按单元键复用时要读的键」：`.claude/rules/verification.md:54`「全仓只有一份判定库：prune-server 自己的那份 RocksDB。库里只放剪枝按单元键复用时要读的键：」→ 改成「库里放两类行：剪枝按单元键复用要读的键与它们的值；录入段交给判段与服务的数据与记号（流程、清单、判表、给定状态表、待判块、判过记号、服务状态行）。」后半句「Rust 一侧是 `verdict_store.rs` 文件头登记的各表」不变，文件头加三张表（格 1）。`.claude/agent-common.md:29` 的「判定一律经 prune-server 客户端按各族的键前缀写进它自己那一份」不改，加一句「gpu-compute-server 也是这个客户端的使用者，待判行与记号是库里的行、不是新的存放处」。
3. 前提 4 之三「一切跑项目代码的命令经统一任务服务起」：`.claude/rules/verification.md:46`「一切跑项目代码的命令只认经 `task-server/task submit` 起；」、`.claude/agent-common.md:56` 子 agent 不交任务那一条的「跑项目代码的命令（编译、测试、门禁、实验、自证这类」、`.claude/kb/tooling.md:41`「统一任务服务（`task-server/`）是起跑项目代码的任务（编译、测试、门禁、实验、自证；CPU、GPU 都算）的唯一入口」→ 都在句末加同一个例外：「常驻服务（prune-server、gpu-compute-server）不是任务，不经 `task submit`，经各自的 `control` 起停；它们的二进制的编译与安装、自证仍是任务，经 `task submit`」。
4. 前提 4 之四「卡由统一任务服务分、一回给一次尝试」：`.claude/rules/verification.md:50`「**凡用 GPU 先清场，跑完不复原**：清场与分卡都由服务做，调用方经 `task-server/task submit -- <命令>` 交，不在外面套清场、不挑卡；」→ 加「崩溃放量与崩溃枚举用例判段的卡归 gpu-compute-server（多机配置 `GPU_COMPUTE_SERVER_CARDS` 点名，不得同时在 `GPU_CARDS_BY_PRIORITY*` 里）：它起来时清场一次，任务不为判段领卡；其余用卡的任务照旧由任务服务分」。同步 `.claude/rules/verification-reference.md:21`（「要上卡的三处…照文件头 `# task-server-card-lease: on-demand` 到上卡那一步才领卡」：崩溃枚举用例与放量入口的判段不再领卡，改成「写待判行、读服务的结论」；product-gpu-real-card-tests 与 feature-gated-tests 照旧）、`:66`–`:75` 整节、`:28`（「机器与卡由服务分；多机配置 `ENABLE_GPU=1` 时由 GPU 判」）、`.claude/agent-common.md:57`（「内存上限、机器、卡、GPU 清场、排队次序、分片、卡住、撞上限放大重排都由服务定，调用方不写、不判」：加「（任务的卡；判段的卡由 gpu-compute-server）」）、`.claude/kb/tooling.md:41` 同一句的「GPU 卡与清场」。
5. 格 6：`.claude/rules/verification.md:41`「每一趟真跑开跑前从键相同、库里有判定的项里随机抽几条当哨兵、再抽一条强制现判（种子打进日志），哨兵有一条被现判、强制现判的判定与库里那一条不同、或强制现判那一条仍复用，整道红。」→ 补「判在服务里时，「被现判」「仍复用」以判过记号上的凭据行为准（轮次、随机数、提交次数、结论摘要），不以本进程交来存的块清单为准」。
6. 格 2：`.claude/rules/test-script-discipline.md:58`「一块判完才落盘，写入是原子的；被打断后重跑同一条命令就是续跑，只补没判的。」→ 加「判在常驻服务里时，被打断的是录入（重跑录入那一趟，只对没有不红结论的块写待判行）或服务（重起后按库里的认领与待判行接着取）」。
7. 格 8：`.claude/rules/verification.md` 多机一节（「所有任务都要多机可用，不许绑定某一台」，`:63` 起）→ 加「服务每台有卡的机器一个，队列在判定库里，配置只来自多机配置；两台的服务二进制同一份（SHA-256 回读）」。
8. kb：`.claude/kb/verification-build.md:268` 判段一行「只从库里读回（核摘要），按块判：配置开了 GPU 由 GPU 判器逐状态判…」与 `:271` 库一行「…分片由门禁 `crash-point-replay` 自己做…各片判的块都直接写进服务那一份，不拷库、不取回、不导入」→ 配了 GPU 时判段改写成「前半段写待判行、等 gpu-compute-server 判、读回结论」，分片对 GPU 判不再按份而按认领；`.claude/kb/decisions/13-验证路线.md:401` 已定项 15 的「崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量」→ 加「判在常驻服务 gpu-compute-server 里，单元键不变」（走决策变更的流程，要不要三方由主 agent 定）。
9. 门禁脚本头（不是规则，但同一件事）：`scripts/crash/crash-amplification.sh` 与 `.claude/gate.d/crash-point-replay.sh` 文件头的 `# task-server-card-lease: on-demand` 声明与「显卡与清场」一段、`.claude/gate.d/verdict-library-scripts.tsv` 不动；`verification-reference.md:146` 那一格「缺的在各片里判」的片，对 GPU 判改成「缺的由服务判、门禁等到判完或报本次未跑」。

## 格 10　不换判据

**判定：成立——条件是服务侧的新代码全部落在「判器入口的闭包之外」；这个条件能被主 agent 用现有工具核。** 逐处答「哪几组判器组键会换」：

| 改法 | 换的组 | 后果 |
|---|---|---|
| 服务进程新 crate、待判行读写、认领、续租、状态行、`store_judged_block` 搬进共用函数、`VerdictStore` 加三张表的读写（`verdict_store.rs`、`crash_amplification.rs` 的 `write_the_pending_blocks`） | **无**。键只由入口登记表点名的函数闭包决定（`judging_units_entries.rs` 文件头；`:52` 起的 `JUDGING_UNITS_ENTRY_GROUPS` 里 `gpu_judge` 那一组 16 个入口）；这些新函数没被任何入口调用 | 判过的块全部照旧复用，V 不变 |
| `GpuCrashJudge` 草稿区与表分开、跨流共用草稿区、把批宽初值传进 `on_card_compiling_these_kernel_sources` | `gpu_judge`；并成的镜像判定 V、给定状态判定 V、开 `gpu` 编的观察者结论那一组随之换（`judging_units_entries.rs` 文件头说观察者那一组的闭包够着 GPU 判器 `.wgsl`）；录入那一组（`PackedJudgeTables::of`、`build_judge_tables_*` 也登在录入组里）只在改了这几个函数时才换 | 旧 V 下的全部块下一趟重判；这是格 4 里「共用草稿区」不值得做的另一个理由 |
| 任何 `.wgsl` 里一个 fn 的改动、`wgpu`（`=30.0.1`）或 `pollster` 版本、`Cargo.lock` 里闭包引到的依赖、`rustc`/`cargo` 版本、`TARGET`、`CARGO_ENCODED_RUSTFLAGS`、仓根 `[profile.release]` | `gpu_judge` 及并成的 V（与服务无关，今天就是这样）| 同上 |
| `verdict_store.rs` 的库布局号 `LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION`（`:1295`）从 6 升 | 不是判器组键，但库「只认这一版」（`:98`–`:99`），升了整库作废、全部块重判 | **加三张表要不要升布局号我没查到先例**（`git log -S` 对 `PairingInputs` 只命中一次提交，看不出当时升没升）；不升的话旧二进制扫到 `0x1D` 起的标签行时 `of_key_tag` 返回 `None`（`:1509`）是什么行为没读，待主 agent 定 |
| 服务进 `Cargo.toml` 工作区的 `members` | 无（键只含闭包引到的依赖版本，`build.rs:138` 读 `Cargo.lock`；成员列表只进 `rerun-if-changed`，`build.rs:247` 起 `watched.extend(…Cargo.toml)`）；但会让**构建缓存**重编 | 无 |

**主 agent 用一块态核「判定字逐态相同」的办法**（一块 = 一个块键，≤ 65536 态；挑一条小流的一块，例如登记的最小那几条之一；不写真服务，用自己起的临时 prune-server，规则 `.claude/rules/verification.md:56` 的「自证与压测一律对自己起的临时服务」）：
1. 前后各取一次 V：改动前经装置二进制取（末尾命令第 2 条），改动后经服务状态行与测试二进制那一条新断言取；**V 逐字相等**（32 字节）是第一道。变了就停——什么都不用比，格 10 的第一列写了哪一组变。
2. 对一块比结论：把该块的结论读成 `ConclusionSpelledOutInVerdictVectors`（`ConclusionSpelledOutInVerdictVectors::of_the_stored_block`，`crash_amplification.rs:3282`；它已含逐状态的判定向量本身、违例、计数向量、恢复字、点名的被抛弃根五样，判定向量不比 u32 编号——编号是库内登记次序，两个库不同；在哪判的与日期是块属性，不在里面），在「今天的路径」（前半段进程自己判，改动前的树）与「服务路径」（改动后）下各导出一份文本，逐字比、比 SHA-256。这一步今天没有现成的导出命令（结构有了、没有把它打出来的程序）：要加 `crash_library_reader block-dump <块键> <V>`（`crates/singlefs-verification/src/bin/crash_library_reader.rs` 一个新子命令，不在判器闭包里），打 `{:?}` 与它的 SHA-256。
3. 只比一块时的省力版（只对**全绿**的块有效）：服务按格 6 做法一对这一块判一次「轮次加一」的强制现判，库里 V 下已有今天路径存的那一份：`store_conclusion_block` 交回 `AlreadyStoredIdentically`（什么都不写）= 逐字相同；不同而两份都绿，交 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion` 并列出差在哪个状态哪一项（`verdict_store.rs:1873`）。**有一份是红的这个比法看不见差别**：`ReplacedBecauseOneOfTheTwoIsRed`（`:1867`）会照新的整份换上、不报差异，红块必须走第 2 步的导出比。
4. 反过来证这个比法会红：往服务路径塞一个已知差别（例如服务把块里一个状态的草稿区起点错位一个字），第 2 步的导出必须不等；塞不进去就不能当「不换判据」的证据。

**为什么服务路径与今天路径判出同一批字**（读到的，不是量的）：服务调的是同一个 `GpuCrashJudge`（`on_card_with_packed_tables`，`crash_judge_dispatch.rs:967`）、同一张判表（库里按摘要读回，核 SHA-256，今天的判段也是读回那一份，`crash_judge_dispatch.rs:452`）、同一个入口 `judge_states_unless_stuck_at_the_indivisible_floor`（`:1146`）、序号区间同一算法（`crash_amplification.rs:4096`）。不同的只有批宽（今天每条流的判器各自从 1 起校准，`:61`；服务里同一判器连着判同一条流的块，宽度续用）与哪张卡、哪个驱动（只进块属性）；批宽「切法不进结论」靠调度单测在假卡上逐条核（`judging_units_entries.rs:29` 的说明，推的，我没去读那条单测）。

**什么现象会推翻**：(a) 第 1 步 V 不等；(b) 第 2 步导出不等而第 1 步等——说明判器闭包之外有影响结论的东西（解码、`store_judged_block` 之类），那就是键的缺口，要立即记欠账，不是服务的缺口；(c) 第 4 步塞了差别而导出仍等——比法本身是空的。

## 背景材料「实现今天的样子」逐句核（我用到的「已有」「没有」）

做法：每句照背景材料的行号去读，再在主工作树里按符号名找一遍。**背景材料里 `crash_amplification.rs` 与 `crash_judge_dispatch.rs` 的行号是段树 `~/.local/share/singlefs/work/batch30-pieces-with-widening-rule/copy` 的**（我读过那一棵：20499、20561、20576、20674、3728、4032 在那里逐个对得上它说的那件事；那棵树的 `crash_amplification.rs` 29170 行、主工作树 27275 行，`crash_judge_dispatch.rs` 3437 对 3420 行）。两棵树的差别：段树多一套「按块序号区间选块」（`BLOCK_RANGE_ENVIRONMENT_VARIABLE`、`blocks_of_this_share_in_the_block_range`、`part_selection`），主工作树里这三个名字都找不到（`grep -n 'part_selection\|BLOCK_RANGE' crates/singlefs-verification/src/crash_amplification.rs` 零命中，段树里命中）。本报告的行号都按主工作树。

| 背景材料的句子 | 核对 | 结论 |
|---|---|---|
| 判表每条流一张，判段读回再上传一次（`crash_judge_dispatch.rs:1014`）；慢流 12091358 个字、读回 0.074–0.647 秒 | 上传在主工作树 `:1014`（`create_buffer_init`，tables_buffer）；12091358 字在 `records/2026-11-17-崩溃判器与测试流的效率优化.md` 第 27 行；**「0.074–0.647 秒」「23–47 MiB」我在该 records 与 `batch39-card-pulls-work/result.md` 里 grep 都零命中** | 前半对；读回用时与表大小的范围出处不明，按「推的」用（格 5 里用它估固定开销，已标） |
| 每批上传四个字（`crash_judge_gpu.rs:1002` `dispatch_words`） | `pub fn dispatch_words(first_ordinal: u64, state_count: usize) -> [u32; 4]`，主工作树 `:1002` | 对 |
| 一批依次跑十几趟内核，每趟一次提交、阻塞等完（`judge_one_dispatch`，`:1232` 起） | 主工作树 `:1232`，文档注释 `:1225` 写「一趟一次提交、等它算完再交下一次」，并列了十余趟 | 对 |
| `SCRATCH_WORDS = 2_009_599`（`crash_judge_gpu.rs:988`）；每进程每卡另占 2560 MiB（`crash_judge_dispatch.rs:36`）；32G 卡一批 3843、16G 卡 1600–1730（量的） | 前两处对。**2560 是留的额度，不是量到的占用**：`:35` 写「5090 上量到的是 2326 MiB（草稿区从 241 MiB 到 12283 MiB 五档都是这个数），留到 2560」。3843 与 1600–1730 在 records 第 32 行。我的式子算出 3848 与 1727（格 4） | 说偏：2560 是留量，实占 2326（一张卡，量的） |
| 防交错的锁每个判器一把（`:568`、`:1080`、`:1246`） | 主工作树三处都是 `one_dispatch_at_a_time`：字段、初始化、`judge_one_dispatch` 里取锁 | 对 |
| 卡池每张卡一个设备，多笔借据共用设备与编好的内核（`gpu_unit_checks.rs:2431`、`:2680`、`:2865`） | `:2431` 是 `GpuCardPool` 文档里的一行说明，`:2680` 是 `fn lease_card`，`:2865` 是 `card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER)` | 对 |
| 「库里没有待判的块的清单」 | **说得过宽**：有录入表（表标签 `0x05`，块键 → 状态数，`verdict_store.rs:1412`、`record_block` `:3665`，`write_the_recorded_blocks` `crash_amplification.rs:1732` 写它）。它不是待判清单（缺版本、判表摘要、序号区间），所以「没有待判清单」成立，「没有块的清单」不成立 | 说偏（格 1 已用准的说法） |
| 判段自己排：重算录入键（`:20499`）、读回清单与判表（`:20561`）、排计划（`:20576`）、按份与块区间取交集（`:20674`）、`pending_blocks`（`:3728`） | 这几个行号在段树逐个对；**主工作树里是**：录入键 `:18854` 起、读回 `read_back_the_recorded_run`（`:18910` 调，函数在 `:6870`）、排计划 `plan_crash_points_with_slices`（`:18925`）、按份取 `my_blocks`（`:19004`，只按份、没有块区间）、`pending_blocks` `:3727` | **行号对不上主工作树**；「块区间」主工作树没有 |
| 结论版本取自本二进制里编进去的键（`:20585`） | 段树 `:20585` 是那条注释；主工作树里取版本在 `ConclusionVersionsOfThePlan::of_the_plan_keeping_the_refused_out`（`:21313`，`run_from_environment` `:18940` 调）；键来自 `crash_identity.rs:568` 的 `include_str!` | 语义对、行号对不上 |
| 判段离不开十几个进程级环境变量（`:20085`） | 主工作树 `ENVIRONMENT_VARIABLES_READ_BY_A_RUN` 在 `crash_amplification.rs:18502`，现数 17 项（我用 awk 数的，不含注释行） | 对（行号对不上，数对） |
| 块没有人认领；写库结论相同不写、有红整份换、两份都绿而不同报错（`verdict_store.rs:1861–1871`） | `VerdictBlockStoring` 在 `:1859`–`:1871`；「两份都绿而不同报错」是 `:1900`–`:1907` 的 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion` | 对（错误那半在 `:1907`，不在 `:1861–1871` 里） |
| 判定库服务已有带租期的认领（`protocol.rs:1129`、`committer.rs:469`、`:670`），今天只用在登记判定向量编号（`verdict_store.rs:5437` 起） | `ClaimOutcome` `:1129`、`add_claim` `:469`、`release_claim_on` `:670`、登记函数 `:5437`（文档）/`:5443`（函数）。Rust 调 `claim` 只有 `verdict_store.rs:2985`（包装）与 `:5458`；Python 客户端没有认领操作码（`prune_server_client.py` 文件头列的操作码里没有 5） | 对 |
| 块队列 `BlocksForTheCards`（`gpu_memory_ledger.rs:2556`）、每张卡一条线程从队头取（`crash_amplification.rs:4032`）、`GpuCardPool`（`gpu_unit_checks.rs:2437`） | `:2556` 对、`:2437` 对；4032 在主工作树是 `:4031` 的 `judge_on_one_card_and_start_the_next`，差一行 | 对（差一行） |
| 同一设备上按源码摘要只编一次管线（`crash_judge_dispatch.rs:1034`）；每条流各建判器、各传判表（`:1010`）、各自从窄往宽校准（`:1649`），进程一退全丢 | 主工作树：`pipelines_compiled_once_on_this_device` 调用在 `:1038`，表上传 `:1014`，`DispatchWidthSchedule` `:1653`，起步宽度 `FIRST_DISPATCH_STATES = 1`（`:61`）；缓存在 `gpu_unit_checks.rs:1134`，**没有淘汰** | 对（行号差 4）；「没有淘汰」是我加的，格 4 用 |
| 普通流 11 条管线；给定状态表另编 9 条（`crash_judge_gpu.rs:1154`、`:1163`） | `GpuCrashJudge` 里 11 个 `wgpu::ComputePipeline` 字段（`crash_judge_dispatch.rs:533`–`:544`）；`of_the_judge` `:1154`、`of_the_judge_given_the_container_capacity_knobs` `:1163`；「9 条」是 `judging_units_entries.rs:28` 的「扫描内核源码里九个入口各一条」 | 对 |
| 调度与批宽不进键（`judging_units_entries.rs` 第 29 行） | 主工作树 `:29` | 对 |
| 「`grep -rn 'pending_blocks' crates/singlefs-verification/src/` 只命中判段自己问库的那一处与它的调用」 | **不对**：`pending_blocks(` 这个函数的非测试调用点有四处（`crash_amplification.rs:2377` CPU 判、`:4849` GPU 判、`:27087` 只排计划打印、`crash_state_enumeration.rs:2240` 崩溃枚举用例），测试里五处；另外 `crash_library.rs:2815`–`:2968` 有一个同名的局部变量（对拍的块），与函数无关 | 说偏：判卡的入口有两处（`:4849`、`crash_state_enumeration.rs:2240`），服务要服务两处 |
| 量的 22 片：起跑到拿到卡 44270 秒（62.4%）、判 25067 秒（35.4%）、持卡不判 1346 秒（1.9%）；每片编整个包平均 162 秒、峰值约 9 GiB；撞内存上限合计丢 9914 秒 | records 第 3.5 节：前三个数对（22 片，第 97–99 行）；**9914 秒是「83 件里 50 件」的数，不是 22 片的**（第 103 行） | 说偏：两个总体混写 |
| 内核变冷编：只有走读内核变 157.8 秒、缓存全空 182.6 秒、命中 1.9 秒 | records 第 132–135 行 | 对 |
| 没量的：起设备、传判表、清场各花多少；交替重叠的探针未跑 | 照读码，没有计时点；我也没量 | 对 |

## 要主 agent 跑的命令

1. 模型复跑（七个变体，认领与租期状态机）。命令原样：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=认领与租期状态机小模型七个变体; 范围=1 个模型; 为什么=核正推腿第 2 格的状态机断言：每块恰好一份结论、不丢块、被打断重跑只补没判的' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-forward/model -- bash /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-forward/model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-forward-model-run /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-forward/model`
   在哪起：`--cwd` 如上。看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里每个变体一行 `VARIANT name=… result=as-expected`、末行 `MODEL_SUMMARY variants=7 as_expected=7`。任何 `result=UNEXPECTED(…)` 或 `state-limit-exceeded` 都不算过；把那一行与它下面的 `example for …` 贴给我（或新派同类腿）读。运行目录 `…-model-run` 跑完由主 agent 删。（范围声明的种类写得对不对照 `.claude/hooks/run-scope-guard.sh` 文件头；闸若说这不是起验证的命令，去掉前缀照交。）
2. 取今天编进装置二进制的镜像判定那一把键（格 3、格 10 第 1 步的改动前基线；只读库，一行都不写）。命令原样：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=crash_library_reader keys 读今天编进装置二进制的判器键; 范围=1 个装置二进制、只读判定库; 为什么=格 3 与格 10 取改动前镜像判定那一把键当基线' task-server/task submit -- cargo run --release -p singlefs-verification --features gpu --bin crash_library_reader -- keys`
   在哪起：仓根 `/home/fy5090/code/singlefs`。看哪几行：`exit=0`；日志里 `CRASH_LIBRARY_KEYS_JUDGE	image_judgement	current_key=<十六进制>` 那一行（格式见 `crates/singlefs-verification/src/bin/crash_library_reader.rs` 文件头）。贴那一行。这一条只给出装置二进制那一半；**测试二进制那一半没有现成命令**（格 3 先决条件，要在 `crash_identity` 测试模块里加一条只打 V 的用例，改 `crates/`，要走三方）。

没有别的要跑的：格 4 的数是我用 `python3 -I -c` 现算的算术（输出贴在格 4）；其余格是读码与推理。

## 推不出的

1. **测试二进制与服务二进制编出的 V 是否相等**（格 3）：构建脚本把依赖包被「本包的 dev 依赖」打开的特性喂进键（`crates/singlefs-core/Cargo.toml:8`），测试构建开着、服务构建不开；闭包是否真引到这些 cfg 原子我没读到。需要测试侧一条打 V 的用例，现在没有。
2. **加三张表要不要升库布局号**（格 10）：`LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION` 是 6（`verdict_store.rs:1295`）；`git log -S` 对 `PairingInputs` 只命中一次提交，看不出历史上加表时升没升；旧二进制读到 `0x1D` 起的标签行时 `of_key_tag` 返回 `None`（`:1509`），调用方怎么处置我没读。
3. **交替重叠的探针**：合计吞吐高于还是不高于独占（格 5 两种排法都写了，哪种用待探针）。
4. **没量过的固定开销**：起设备、传判表、建判器（换流时）、清场各花多少；`FIRST_DISPATCH_STATES` = 1 起校准在服务里续用的收益；给定状态表那一路每多一组旋钮栏管线占多少驱动显存 p（格 4）；驱动编内核时服务的主机内存峰值（格 7）。
5. **读回判表 23–47 MiB、用 0.074–0.647 秒**这一组数我没找到出处（格 5 用它估固定开销，已按推的用）。
6. **服务的 CPU 优先级怎么保证**（格 7）：`CPUWeight=` 要用户级 systemd 的 `cpu` 控制器已委派（没查）；让任务单元起跑带 `Nice=`（任务服务今天的 `systemd-run` 参数里没有）要改 `task_server.py` 的 `start_attempt`（`:11147`）。两条都要在目标机器上现量。
7. **服务的卡坏之后自动解除停用与否、开机要不要自起**：用户没定过（格 7）。
8. **水位线与几个数**：H（每卡 20 秒的活）、1 秒固定开销、A 最大 60 秒、探针的 1.1 门槛、续租 10 秒/租期 30 秒，都是我取的，不是量的。
9. **范围**：第一版只写了镜像判定（种类 1）与给定状态表（种类 2）。观察者交回镜像的池级 checker（表 `0x17`）、第 ② 段 GPU 核对（`crash_verify_gpu.rs`）、单元校验和、对拍用例借卡，各是另一种卡上的活，这一轮没设计；崩溃枚举用例那一路的 GPU 判是否与放量走同一个函数（`crash_state_enumeration.rs:2140`）我没读。
10. **批宽「切法不进结论」的调度单测**（`judging_units_entries.rs:29` 说在假卡上逐条核）我没读，格 10「同一批字」的推理用了它。
11. **模型的数**：状态数、边数、运行时间都没有（没跑）；七个变体的预期是我照代码推的，我的推理可能有错，错了会表现为 `result=UNEXPECTED`。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 待判清单 | 成立 | 新加三张表（待判行 `0x1D`、判过记号 `0x1E`、服务状态 `0x1F`），键 `V ‖ 块键 ‖ 轮次`，值带判表摘要、序号区间、状态数、槽数；集合等于 `pending_blocks` 的结果；要改 `verification.md:54` 那一句 |
| 2 认领与不重判 | 成立（模型待跑） | 已有认领够用，补四处：认领键用判过记号、认领后点读结论、续租线程加进度看门狗、一卡一客户端；三种交错对应模型变体 1–5 |
| 3 版本对账 | 成立；先决条件推不出 | 服务只用自己编进去的 V，不从行里取；对不上当场报本次未跑；旧服务不会把旧判法记进新键；测试与服务二进制 V 是否相等要现量 |
| 4 批大小 | 成立（部分数没量） | n = ⌊(B−T−D)·2^20÷w⌋，k 条同进程合计几乎不变、窄的是批宽；32G 一/二/四条 3848/1921/957，16G 1727/860/427；给定状态表的 p 没量 |
| 5 水位线 | 成立；重叠推不出 | 量待判卡秒与最老年龄；各卡自己拉、亲和；探针高于独占就每卡驻留 k* 个判器，否则每卡一个、预取判表 |
| 6 强制现判 | 成立 | 进程内的清单在服务形态下空判；推荐轮次加一加凭据行，辅以服务自抽查；先删后判与 CPU 对判不选 |
| 7 与任务服务分工 | 成立（卡要划界） | 清场、卡坏发现、起停归服务；卡按多机配置新键划界，不进任务服务的卡账；CPU 优先级办法推不出 |
| 8 多机 | 成立 | 每台有卡的机器一个进程、队列在库里、两台二进制同一份、各取各的 V |
| 9 规则条文 | 成立 | 列了九处：编译目录、判定库只放复用键、一切经任务服务起、清场与分卡、哨兵、续跑、多机、两份 kb、门禁脚本头 |
| 10 不换判据 | 成立（条件） | 服务新代码全落在判器入口闭包之外则无组换键；给了四步核法，红块不能靠库的 `AlreadyStoredIdentically` 比 |

## 没做什么

- 没跑模型，没跑任何命令之外的只读检查与一次算术；模型的期望输出是推的，数字待主 agent 跑。
- 没替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 没改 `crates/`、规则、kb、门禁脚本；格 9 只列。没起样本进程、没建仓副本（草稿目录里没有仓副本与编译目录）。
- 没设计第 ② 段 GPU 核对、观察者镜像的池级 checker、单元校验和、对拍这几种卡上的活（推不出第 9 条）。
- 没读：`crash_state_enumeration.rs` 的 GPU 判路径、调度单测、`task-server/task_server_lib.py` 文件头里与分卡无关的各节。
- 偏差一处：中途两次用 `python3` 就地改了本报告文件里的几处行号与措辞（本腿没有 Edit 工具、`scripts/kb/replace-batch.py` 不是一开始就用），之后改用 `scripts/kb/replace-batch.py`；改动只在本报告文件，内容以文件现状为准。
