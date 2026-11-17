# 卡一侧验证服务（gpu-compute-server）第一轮：攻方腿

日期 2026-11-17。行号一律是仓根今天的树（提交 8ed2ea484 之上的工作区）；背景材料引的是段树 `~/.local/share/singlefs/work/batch30-pieces-with-widening-rule/copy` 的行号，两棵树在 `crash_judge_dispatch.rs`、`crash_amplification.rs` 上不同（第五节）。本文的数分「量的」（标出处）与「推的」；模型的数是这条腿自己在草稿目录跑出来的，判决要引由主 agent 复跑。

## 复跑

模型目录 `/home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-attack/model/`（派发提示定模型放草稿目录；纯 Python 标准库，不编不跑仓里的代码，不碰崩溃枚举）。复跑命令见文末「要主 agent 跑的命令」第 1 条。

| 文件 | sha256 |
|---|---|
| model/claim_lease_model.py | ca67f59b73c0990f73d6f6d70febbe406a9f3749babef447df12c6ffe1b7c6e7 |
| model/out/claim_lease_model.out | 6906d2944a7531845bd0e7d821b5e67f6545d95b8e36d03347457833db5d406e |
| model/tools/rerun.sh | 305fd0e0d20f2f3b10cf6730cb71c640736fc26a55eb113ebbddd71596f50270 |
| model/tools/rerun-from-a-clean-work-directory.sh | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |

探针（这条腿在本机直接起的那一次，`/usr/bin/time -v`，量的）：全量 45 格挂钟 0.09 秒、最大常驻 12228 KiB；不缩、不分批。第 3b 步的崩溃枚举取样不适用：模型里没有盘、没有崩溃状态，只有认领、写、读三种库操作的交错。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 2 认领与续跑 | 不成立（打中三处） | 认领「已有结论」= 键在库里：红块、强制现判块永远认领不到（模型 lost 3/3）；写不看认领、存块先读后写：租期一过两份都绿而不同不报错（模型 silent 10 到 30）；续租再写也拦不住，只有写时核认领拦得住，prune-server 没有这一样 |
| 3 版本对账 | 不成立（打中四处） | 结论版本是 CPU 二进制里两组并成的一把，服务拆不出来核；不核就旧判法记在新键下、新判法记在旧键下（回退与二分时复用到被撤的判法）；照 prune-server 的样子自带 Cargo.lock 时连「核键」都会假过；拒算组的占位按进程取、行只留在进程里，隔离做法下拒算的流判不成 |
| 6 强制现判与哨兵 | 推不出（每种做法都有漏） | 今天的保证全在进程里；换服务后结论相同的现判一行都不写，库里分不出「现判了」与「读了库」；给出五种做法各漏什么 |
| 10 不换判据 | 不成立（打中三处，一处推的） | 给定状态表的旋钮不在打包判表里，只带判表摘要的清单让服务照产品旋钮判、写在带旋钮的块键下；存进库的结论还要 CPU 从流算三样；对账口径与待判口径不同，服务漏判的红块与强制块照「判过」对账 |
| 5 材料的「已有」「没有」 | 对不上 10 处 | 第五节逐条 |
| 常驻之后照付 | 四样照付 | 第六节 |

格 1、4、5、7、8、9 不是分给这条腿的重点，只在打中牵到时提一句，没有判。

## 一、第 2 格：认领与续跑（攻击面：哪一条最脆弱 / 内部矛盾）

模型（`model/claim_lease_model.py`）照代码三处建：认领照 `committer.rs` 的 add_claim，写照写分支（不看认领、写到的键上的认领一律释放），存块照 `verdict_store.rs` 的 put_a_conclusion_block_into_the_batch（先读、再按四格决定写不写）。两个服务 A、B，一块；事件有任一服务任一步之前崩溃（至多一次）、租期到期（至多一次）、别人持有时重认领（至多 2 次）。用户与设计定的那一步（认哪一把键）放成五条设计臂全扫：认结论键、认待判清单行、认另立的「判完记号」键、记号键加写前续租、记号键加写时核认领（最后一条今天 prune-server 没有）。起点 fresh / red / forced 三种，判出的结论 same / skew-green / skew-red-green 三种，共 45 格，确定性 BFS 穷举全部交错。

### 1.1 认领的「已有结论」只看键在不在：红块与强制现判块永远认领不到（丢块）

- 现象：服务按待判清单认领，红块与被抽成强制现判的块在清单里，却永远认领不到、永远没人判；对账照样报 0 块没判。
- 依据：认领逐键先问库里有没有这把键，有就回「已有结论」：「verdicts.push(ClaimVerdict::AlreadyConcluded);」（`prune-server/src/committer.rs:490`），协议文件头同一句「逐键判：库里已有这个键（结论已写）回」（`prune-server/src/protocol.rs:24`）。而判段的复用口径要现判两类「键在库里」的块：强制现判「if self.is_forced_to_be_judged_again(store, point_index, key) {」（`crates/singlefs-verification/src/crash_amplification.rs:1956`）与记着红的「&& !self.is_red(store, point_index, key)?)」（`crates/singlefs-verification/src/crash_amplification.rs:1960`）。对账却只问结论在不在：「if !conclusions.is_stored(store, point_index, &key)? {」（`crates/singlefs-verification/src/crash_amplification.rs:5493`）。
- 最小交错（起点：块 b 在版本 V 下存着上一趟的红结论；或存着绿结论、这一趟被抽成强制现判）：
  1. CPU 一侧照 pending_blocks 的口径写清单，b 在清单里（材料「CPU 一侧写待判清单」）。
  2. 服务 A 认领 b 的结论键 → 已有结论 → 跳过（材料「两台五张卡上的服务各自认领」）。
  3. 服务 B 同上 → 跳过。没有第三条路。
  4. CPU 一侧对账按 is_stored 数，b 算判过，这一趟报 0 块没判；红块不重判，强制现判那一块「这一趟还是复用了」。
- 模型：`name=claim-conclusion-key/red/same terminals=3 lost=3`、`name=claim-conclusion-key/forced/same terminals=3 lost=3`（另两种判出的结论同样 3/3）。认清单行更糟：清单行本身就在库里，连 fresh 块也认领不到，`name=claim-pending-row/fresh/same terminals=3 lost=3`，9 格全是 3/3。
- 推翻材料的哪一句：材料「判定库服务已有带租期的认领……今天只用在登记判定向量编号上」与判据格 2「用判定库服务已有的带租期的认领够不够」。今天那一处能用，是因为判定向量编号那一行写一次就不再改（`verdict_store.rs` 的 register_verdict_vector_in_the_prune_server 认到「已有」就读回接着走下一个编号）；判定块要按「记红撤绿、强制现判」重写同一把键，正好落在这把认领的盲区。
- 四句：分辨臂——分辨「认哪把键」的臂（结论键、清单行中；另立记号键在模型里 lost=0）；系统看不看得到——认领那一刻服务只看到「键在」，红不红要读违例行、强制现判那张表在 CPU 进程的内存里（`verdict_store.rs` 的 conclusion_block_is_forced_to_be_judged_again），服务看不到；判据分句——「被打断后重跑同一条命令就是续跑，只补没判的」的反面：该补的被当成判过，另撞 `.claude/rules/verification.md@一个门禁：两种跑法` 里「强制现判那一条仍复用，整道红」；改法——材料没给。我的改法见 1.4（只在我的模型上量过、被攻过零轮）。

### 1.2 写不看认领、存块先读后写：租期一过，两份都绿而不同不再报错（重判后写坏）

- 现象：今天「两份都绿而不同报错」只在两次存块前后错开时成立；两次存块的读都落在对方的写之前，后写的整份盖掉先写的，不报错。
- 依据：写分支照写，写到的键上的认领一律释放，不比持有者：「self.release_claim_on(&mut group.batch, entry.key);」（`prune-server/src/committer.rs:451`）；「别人持有」只在认领时判「record.holder_identity != client_identity」（`prune-server/src/committer.rs:680`）。存块先读：「let stored = self.read_the_stored_conclusion_block_rereading_after_a_side_row(」（`crates/singlefs-verification/src/verdict_store.rs:4709`），比完再写：「self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)?;」（`crates/singlefs-verification/src/verdict_store.rs:4688`）；读本身也不是一帧：「服务没有把点读与前缀扫放进一帧的读法」（`crates/singlefs-verification/src/verdict_store.rs:4880`）。没有比较交换。
- 最小交错（模型原样输出，认结论键、两份都绿而不同）：
  `A claim → claimed；A judge → 判出 g1；A read → 读到 None；租期到期；B claim → claimed；B judge → 判出 g2；B read → 读到 None；A write g1 → written，库里是 g1；B write g2 → written，库里是 g2`
  认记号键也一样（A 写完结论、写记号之前崩溃，B 照样盖）：`A claim → claimed；A judge → 判出 g1；A read → 读到 None；租期到期；B claim → claimed；B judge → 判出 g2；B read → 读到 None；A write g1 → written，库里是 g1；A 在 mark 之前崩溃；B write g2 → written，库里是 g2；B mark`
- 什么时候同键下判出两份不同的绿：第二节那几种版本错位，或判法依赖了键外的输入——后者正是强制现判要抓的「判定不稳」。顺序存时 stored_conclusion_compared_with 报 DifferentConclusion…，并发时这一处报警被吃掉。
- 租期到期并不稀罕（推的）：今天唯一的认领租期是 30 秒「const PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS: u32 = 30_000;」（`crates/singlefs-verification/src/verdict_store.rs:2395`）；脉冲设计在成批时认领、在水位线下排队，认领到写之间夹着排队；容错客户端断线重发的总时长上限另有「有次数与总时长上限（缺省 30 次、120 秒）」（`prune-server/src/reliable_client.rs:9`），A 的写可以在租期过后才落。

| 设计臂（模型） | fresh skew-green silent / detected | red skew-green | forced skew-green |
|---|---|---|---|
| 认结论键 | 10 / 10 | （lost 3/3，见 1.1） | （lost 3/3） |
| 认记号键 | 30 / 30 | 30 / 30 | 0 / 42 |
| 记号键 + 写前续租 | 30 / 18 | 30 / 18 | 0 / 55 |
| 记号键 + 写时核认领 | 0 / 18 | 0 / 18 | 0 / 56 |

（silent、detected 都是终态个数，摘自 `model/out/claim_lease_model.out` 各 `name=` 行；同一终态不同时记 silent 与 detected。）forced 那一列 silent 为 0，是因为库里已有一份、先读到它的那一方会报错；fresh 与 red 起点上两方都可能读到空或读到旧红。

- 推翻材料的哪一句：材料「写库时结论相同不写、有一份红整份换、两份都绿而不同报错」——这一句在有认领、有租期接手的并发下只对一半；续租再写（第三行）也拦不住，只有写的那一帧核认领（第四行）拦得住，而 prune-server 的写不核。
- 四句：分辨臂——分辨（写时核认领那条臂 silent 全 0，别的都中）；系统看得到——B 写的那一刻库里有 A 的 g1，但 B 的比较用的是自己早先读到的空；判据分句——「一块判完才落盘，写入是原子的」（`.claude/rules/test-script-discipline.md:58` 那一句）每一批照样原子，破的是「原子」之外没写出来的那一半：两批之间不互斥；改法——见 1.4。

### 1.3 服务重起换了认领身份：自己认过的块也要等租期（推的）

- 依据：库的客户端每开一次在一份自己的一次性目录里放追加日志，身份随日志走（`verdict_store.rs` 的 connect_with_a_journal_of_its_own：「追加日志只管断线重连时重放没确认的帧，不必跨进程留着」那一段）；认领只给同一身份续租（`committer.rs:680` 那一条判别）。
- 交错：服务 A 认下一批 k 块 → A 进程死 → 管它的一方立刻拉起新进程「接替 A」→「接替 A」是新身份，那 k 块对它是「别人持有」→ 等满租期才判。CPU 一侧「重跑同一条命令」时这 k 块要么等租期、要么报没判。租期定长一点能盖住脉冲排队（1.2），这一处就跟着变长；两头拉扯，没有一个租期两头都对。
- 模型里这类终态记在 retry_bounded（重认领次数用完），不算 lost；它是重试上限造出来的，不拿它当打中。

### 1.4 这一格的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 依据 |
|---|---|---|
| 另立「这一趟判完记号」键来认领，不认结论键、不认清单行 | 1.1 | 量过（模型，这条腿跑的）：`claim-marker-key` 9 格 lost 全 0；代价是库里多一类不供复用的行，要改 `.claude/rules/verification.md` 那一句「库里只放剪枝按单元键复用时要读的键」（`.claude/rules/verification.md:54`），记号还要按趟清 |
| prune-server 加「写时核认领」：写帧带认领身份，持有者不是它或租期已过就整帧拒 | 1.2 | 量过（模型）：`claim-marker-fenced` 9 格 silent 全 0；写前续租那一条（`claim-marker-renew-before-write`）silent 仍是 30。要改协议（写帧多一段）、提交线程、`VerdictStore` 的存块要带着认领 |
| 不靠认领挡并发，存块那一批改成「库里还是我读到的那一份才写」（比较交换） | 1.2 | 推的，没实现没跑：prune-server 今天没有条件写；只修 1.2，不修 1.1 |

## 二、第 3 格：版本对账（攻击面：过度外推与越位 / 最脆弱）

### 2.1 版本是 CPU 二进制里两组并成的一把，服务拆不出来核：旧判法记在新键下、新判法记在旧键下

- 依据：结论版本取 CPU 一侧这份二进制编进去的键「结论的版本（J8）：判法单元键是编进二进制的那一份（不读树）」（`crates/singlefs-verification/src/crash_amplification.rs:18934`），而且是判定定义那一组与 GPU 判器那一组并成的一把（同文件 of_the_points_keeping_the_refused_out 里调 judgement_units_key_with_the_gpu_judge 那一段）。清单行若只带这一把，服务手里只有它自己编进去的两把，算不回、也比不了 CPU 那一把是不是由同一组 GPU 判器单元并出来的。服务写进块属性的是它自己的指纹「gpu_judge_units_fingerprint: gpu_judge_units_fingerprint_of_this_process(),」（`crates/singlefs-verification/src/crash_amplification.rs:3800`），但块属性「只是出处，不进种类、不进版本」（`crates/singlefs-verification/src/verdict_store.rs:838`），比结论时整份不比（`verdict_store.rs` 的 is_the_same_conclusion_as 里 `judged_on: _`）。这一处出处今天没人读。
- 交错甲（内核改了而第二台的服务没重起，旧判法记在新键下）：
  1. 两台的服务都由内核 K1 的树编。
  2. 提交 K2：扫描内核补上一条违例检查。本机的服务重起成 K2，第二台没重起（材料前提 3「五张卡……分批次发给各个gpu」，两台都取活）。
  3. CPU 一侧在 K2 的树上算出版本 V2，写清单。
  4. 第二台的 K1 服务认领其中一半，用 K1 内核判，写在 V2 下。
  5. 之后每一趟 V2 下这一半全部复用：K2 补的那条检查在这一半上从来没跑过，门禁绿。
- 交错乙（旧判法的绿盖掉新判法的红，不要任何竞争）：
  1. 同甲 1–3。本机 K2 服务判块 b，判红，写在 V2 下。
  2. 下一趟 b 记着红，照复用口径进清单（`crash_amplification.rs:1960` 那一条）。
  3. 第二台 K1 服务认领 b（用 1.4 的记号键，认得到），判绿。
  4. 存块：库里是红、要存的是绿，「有一份红」就整份换「&& (!stored.violations.is_empty() || !requested.violations.is_empty()) =>」（`crates/singlefs-verification/src/verdict_store.rs:4717`）→ V2 下 b 变绿，不报错。K2 抓出来的违例消失。
- 交错丙（新判法记在旧键下，回退与二分复用到被撤的判法）：
  1. CPU 一侧在 K1 的树上起一条长流 R1，写 V1 的清单。
  2. 中途提交 K2，服务重起成 K2，接着判 R1 剩下的 V1 行，写在 V1 下。
  3. 之后发现 K2 漏判、回退到 K1（或二分落到 K1 那一个提交）：键回到 V1，复用到 K2 判过的那几块。
- 推翻材料的哪一句：判据格 3 问的「内核改了之后旧服务还在跑的那段时间会不会把旧判法的结论记在新键下」——会，而且反方向（丙）同样会；材料「内核一变：……这一组的键换掉、这把键下判过的块重判」只在判与记出自同一份二进制时成立。丙破的是 `.claude/rules/machine-first.md@留下的（与谁读代码无关）` 里的「可二分、可回退」。
- 四句：分辨臂——分辨「服务核不核版本」的臂；系统看得到——服务看得到自己的两把键，看不到清单那一把是由哪两把并成的，除非清单行把两把分开带；判据分句——`.claude/kb/decisions/13-验证路线.md@已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成` 的「改 GPU 判器那一组的任一单元，这一类的每一块……都作废、现判」，实际没作废的是第二台判的那一半；改法——清单行把两组键分开带，服务用自己编进去的两把重算并成的版本，与清单行逐字比，不等就不认领、打一行拒绝。推的，没实现没跑；它修甲、乙、丙，不修 2.2。

### 2.2 照 prune-server 的样子自带 Cargo.lock：服务「核键」会假过

- 依据：派发提示定 gpu-compute-server 放仓根独立目录，例子是 prune-server/；那是带空 `[workspace]` 的独立工程、自带 Cargo.lock，仓根工作区把它排除在外「exclude = ["scripts/experiment-apparatus", "prune-server"]」（`Cargo.toml:19`）。验证包编进去的键里的依赖版本，按 D13 的口径是「解析出的闭包真引到的依赖版本」（`.claude/kb/decisions/13-验证路线.md:431`），构建脚本读的是仓根那一份「let lock_text = std::fs::read_to_string(repository_root.join("Cargo.lock"))」（`crates/singlefs-verification/build.rs:138`）——不论这一次编译 cargo 实际按哪一份锁文件解析。
- 交错：
  1. gpu-compute-server 用 path 依赖拉验证包，自带锁文件（照 prune-server 的样子）。
  2. 两份锁文件里的 wgpu（WGSL 编译器在它下面）版本分开漂：一边 `cargo update` 了、另一边没有。
  3. 服务编出来的二进制里，GPU 判器那一组的键照仓根锁文件写着旧版本；它实际链的是新版本。
  4. 服务把自己的键与清单行的键比（2.1 的改法）：相等，放行。换了 WGSL 编译器判出的块记在旧键下。
- 推的：没有编这样一个工程；是否真分开漂取决于怎么建它。主 agent 可以在建工程之前照这一条定「服务进仓根工作区，与测试二进制共用一份锁文件」或「构建脚本核实际解析的版本」。
- 推翻材料的哪一句：判据格 3「它编进去的判器组键与 CPU 一侧期待的键怎么对」——「编进去的键」不保证是「链进去的代码」的键。

### 2.3 测试二进制与服务二进制编进去的键可能永远对不上（推的，待量）

- 依据：构建脚本把依赖包在这一次编译里开了哪些特性、profile 的生效值、RUSTFLAGS 都写进键表（`crates/singlefs-verification/build.rs` 文件头第 7–11 行那一段与 main 里 features_enabled_on_the_dependencies、effective_profile_lines 两处）；验证包的 dev 依赖替核心层并上两个特性「singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "test-only-publish-thresholds"] }」（`crates/singlefs-verification/Cargo.toml:55`），只在编测试时生效。
- 后果分两头：GPU 判器那一组的闭包够得着这两个特性的 cfg 原子，测试二进制与服务二进制的 GPU 判器键就不同——服务照 2.1 的改法核键时，一块都认领不了（活性坏）；够不着则相同。哪一头成立我没量，文末第 2、3 条命令量它。
- 推翻材料的哪一句：材料默认「服务是单独编的一份二进制，它编进去的判器组键与 CPU 一侧期待的键」能对上，没说按什么编法对得上。

### 2.4 拒算组的占位按进程取、行只留在进程里：隔离做法下拒算的流判不成

- 依据：拒算组的占位键按进程标记取「message.extend_from_slice(process_marker);」（`crates/singlefs-verification/src/crash_amplification.rs:21135`），键里带它的行不进服务、只记在本进程「kept_out.rows.insert(key, value);」（`crates/singlefs-verification/src/verdict_store.rs:3269`）；同文件单测已经钉住两个进程互相读不到「前提：两个进程的占位不同，判段照自己的占位读不到录入进程的那一份」（`crates/singlefs-verification/src/crash_amplification.rs:15145`）。
- 交错：一条流的某一组判法单元拒算（例：观察者没登记，NotComputedAtTheBuild）→ CPU 一侧拿自己的占位当版本、录入的行只留在 CPU 进程 → 服务读不到判表，或照清单行的占位版本写进服务（违「照判不入库」），或写进服务自己进程的内存（CPU 一侧读不到、清单行永远待判）。三条路没有一条合今天的规矩。
- 推翻材料的哪一句：前提 2「cpu数据装入rocksdb gpu server从rocksdb中读出来」在拒算这一类上做不到——这一类的数据按设计就不进库。要么拒算的流留在 CPU 进程里判（GPU 那一路在 CPU 进程里保留一份），要么改「照判不入库」。

### 2.5 照代码直接搬：常驻服务在存第一块时 panic

- 依据：块属性里的 GPU 判器指纹取「TOOLCHAIN.get_or_init(CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test)」（`crates/singlefs-verification/src/crash_library.rs:101`），后者没有 CARGO 环境就 panic「要经 cargo test 起（环境里要有 CARGO）」（`crates/singlefs-verification/src/crash_point_replay_progress.rs:42`）；存判好的块经它（`crash_amplification.rs:3800`）。独立服务不经 cargo test 起，第一次存块就停。好改，但说明「判段搬进服务」不是把现有函数换个进程调。

## 三、第 6 格：强制现判与哨兵（攻击面：哪一条最脆弱）

### 3.1 今天保证的是什么

- 规矩：「哨兵有一条被现判、强制现判的判定与库里那一条不同、或强制现判那一条仍复用，整道红。」（`.claude/rules/verification.md:41`）；判法文件头「再从剩下的里抽 1 条强制现判（只这一条不读库）」（`scripts/gate/lib_reuse_checks.py:22`）。
- 崩溃放量的实现全在一个进程里：强制现判表是库对象在内存里的一张表「self.conclusion_blocks_forced_to_be_judged_again.contains(&(」（`crates/singlefs-verification/src/verdict_store.rs:3182`）；「这一趟现判过」按进程里的账认，存块一进门先记账、写库之前就记「self.conclusion_blocks_offered_in_this_process.push((」（`crates/singlefs-verification/src/verdict_store.rs:4670`），判完照它查「.conclusion_blocks_offered_since(self.offered_before_judging)」（`crates/singlefs-verification/src/crash_amplification.rs:3691`），再把库里那一份与开跑前抽下的那一份按判定向量比。
- 所以今天实际保证三样：这个进程的待判口径把那一块当待判；这个进程的判器判出一份结论、交给了存块；存完之后库里的与开跑前的相同（判定不稳就红）。「只这一条不读库」不是被核出来的，是代码结构给的：同一进程里判器那条路上没有读结论的代码。
- 哨兵今天测的是待判口径本身：哨兵块只有进了待判才可能被交来存。

### 3.2 换成服务之后，库里分不出「现判了」与「读了库」

- 依据：结论相同的一次现判什么都不写「VerdictBlockStoring::AlreadyStoredIdentically => return Ok(storing),」（`crates/singlefs-verification/src/verdict_store.rs:4684`），块属性也留先存的那一份（同文件 1864 行那一段说明）。服务真的现判了一块、判出与库里相同的结论，库在它前后逐字节相同；服务没判、直接回「判了」，库也逐字节相同。只读库的一方（CPU 一侧）无论怎么读都分不出两种情形——这是判别子观测不到（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 的第二行）。
- 原样搬过去先是另一种坏法：CPU 一侧的 problems_after_judging 读本进程的账，服务存的块不进这本账 → 每一趟都报「强制现判没起作用」（整道红），哨兵永远不响（空过）。被这一红逼着改的第一反应是「服务回执」，正好是下表最弱的那一种。

| 做法 | 拦得住 | 拦不住 |
|---|---|---|
| 甲：服务对每块回执，或写一行「第 X 趟判过」 | 服务丢活、半路死、照 1.1 跳过而老实报 | 服务里任何「先看库或自己的缓存再决定判不判」的一层照样回执——「说现判了、其实读的是库」；回执由被核的一方自己出。另是一类新的不供复用的行（`.claude/rules/verification.md:54` 那一句） |
| 乙：CPU 一侧先把强制那一块的结论从库里删掉（副本留在内存），进清单，判完与副本比 | 服务读库（库里没有可读的）；1.1 的「已有结论」跳过（删了就认领得到） | 服务按块键或判表摘要留的内部缓存；删到比之间别的趟、别的台把它当没判的也去判；CPU 一侧在删之后、比之前死掉，副本随进程没了，这一块下一趟不经比对就入库；删掉之后服务判出的不同的绿不再撞 DifferentConclusion…（库里是空），只剩 CPU 一侧那一次比 |
| 丙：给强制那一块一把一次性的版本（版本里拌进这一趟的随机数），判完比再删 | 服务读库（那把键下库里没有） | 服务按块、不按版本的缓存；与 2.1 的改法直接冲突——服务要拒「自己重算不出的版本」，一次性版本它算不出；崩溃留下的一次性行没人读、没人删 |
| 丁：CPU 参照判器在 CPU 一侧复判强制那一块（`crash_judge_cpu_blocks` 那一路），与库里 GPU 判的那一份比 | 服务的一切不老实；GPU 判出的与 CPU 参照不同 | 不测 GPU 自己两次判的一不一致；挂钟没量（一块 CPU 判多久，推不出）；与「登记了的那几类只走加速路径」（`.claude/rules/test-script-discipline.md:75`）字面冲突，要改那一条 |
| 戊：混进一块答案已知的探针块（内核里种一个假违例位，同 GPU 判器对拍用例的写法） | 服务根本不跑内核 | 强制那一块本身；种了假违例位的是另一份内核源码、另一组管线，测的是另一个程序 |

- 推翻材料的哪一句：材料前提 5「强制现判 需要考虑下其他的实现方式的可能性」本身不错；要补的是判据格 6 那一问的前提——「怎么证这一条确实被现判了」在只读库的一方那里，乙、丙之外答不出，而乙、丙各自带进新的窟窿（乙的副本只在内存、丙与 2.1 冲突）。只有丁不靠服务自己说，但它换了判器（CPU 参照）、要改加速那一条规矩。
- 四句：分辨臂——分辨（甲最弱、丁最强，五种各漏不同）；系统看得到——CPU 一侧看不到服务进程里发生了什么，只看得到库，而库对「现判且相同」不留痕；判据分句——`.claude/rules/verification.md:41` 的「强制现判那一条仍复用」：服务之后「仍复用」可以发生在 CPU 看不到的那一侧；改法——表里五种都是推的，没实现没跑；要选的话乙加丁最接近今天的保证（乙证服务不读库、丁证结论本身对），代价是删库与 CPU 判一块。

## 四、第 10 格：不换判据（攻击面：过度外推与越位）

### 4.1 给定状态表的旋钮不在打包判表里：只带判表摘要的清单让服务换了判器

- 依据：旋钮值编进扫描内核「let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(」（`crates/singlefs-verification/src/crash_amplification.rs:20449`），打包判表只由判表建「let tables_packed = PackedJudgeTables::of(&tables);」（`crates/singlefs-verification/src/crash_amplification.rs:20503`），旋钮另记一栏「旋钮不落盘，所以不在表的镜像里，跟着表另记一栏」（`crates/singlefs-verification/src/crash_amplification.rs:6261`），且「旋钮值不进判法单元的键」（`crates/singlefs-verification/src/crash_judge_gpu.rs:1159`），只进块键。
- 交错：
  1. CPU 一侧录一张旋钮压小的给定状态表 T，按打包判表的摘要写清单（材料格 1 列的「判表的键、块号、结论版本」——没有旋钮这一栏）。
  2. 服务按摘要读回打包判表，照 of_the_judge()（产品旋钮）编内核、判，写在 T 那一张表的块键下（块键里带着压小的旋钮）。
  3. 结论是「产品容量的扫描内核判压小旋钮写出的镜像」，记在「压小旋钮」的键下。键、版本都对，判器不是那一个。
  4. 强制现判抽到 T 的一块时由同一个服务重判，同样错，判出相同 → 过。只有 GPU 与 CPU 的对拍用例抓得到，而对拍不逐流跑。
- 推翻材料的哪一句：判据格 3 括号里的「给定状态表那一路的旋钮编进内核……而清单里没带旋钮」——确实会错，错在判据而不在版本：键全对。另一种更隐的形状（推的）：两张只差旋钮的表打出同一份打包判表、同一个摘要；服务若按判表摘要缓存「判器」（表已上卡、管线已编），第二张表用上第一张的管线。今天按三份源码合起来的摘要缓存管线（`crash_judge_dispatch.rs` 1038 行的 pipelines_compiled_once_on_this_device），这一层不错；错在服务新写的那一层按什么做键。
- 四句：分辨臂——分辨（清单带不带旋钮、服务缓存按不按源码摘要）；系统看得到——服务只读判表就看不到旋钮，要读给定状态表那一行（带两栏那一份字节）才看得到；判据分句——`.claude/rules/test-script-discipline.md:71` 那一句「优化不许换掉判据」；改法——清单行带旋钮三值或指到给定状态表那一行，服务的判器缓存按三份内核源码的摘要做键。推的，没实现没跑。

### 4.2 存进库的那一份结论不只是卡上算的：服务只拿判表算不出同一份

- 依据：存块之前 CPU 一侧还从流里算三样，都不在打包判表里：按发布分的态数并进计数向量「.absorb(publishes_of_the_states.state_counts_of(」（`crates/singlefs-verification/src/crash_amplification.rs:5009`）；解码逐盘槽数要池里盘的次序「singlefs_core::recovery::PoolReader::device_identities(flow.base)」（`crates/singlefs-verification/src/crash_amplification.rs:4861`）；基镜像那一份逐盘自证槽数用 CPU 参照算、随每块存「self_verified_system_configuration_slots_of_each_device(flow.base);」（`crates/singlefs-verification/src/crash_amplification.rs:4866`），而且先于结论块单独写一批「store.store_observer_block(key, kind, &version, &slots.to_bytes())?;」（`crates/singlefs-verification/src/crash_amplification.rs:2090`）。
- 后果：「gpu唯一的关注的就是 一个batch……」（前提 1）在存块这一步不成立——存进库的计数向量与槽数要录下的流（基镜像、计划、各段的发布），不止判表与序号。两条出路各碰一条规矩：服务读流自己算（服务这一侧又多了判定定义组里的 CPU 参照代码，2.1 要核两把键）；或服务只交回卡上的结论字、CPU 一侧组好再存（卡上的结论字成了库里的又一类行，`.claude/rules/verification.md:54` 那一句要改）。若服务只用判表算、漏掉第一样，同一块的计数向量与今天不同：老块上撞 DifferentConclusion…，新块上悄悄不同。
- 推翻材料的哪一句：材料「每一批只上传四个字……状态在卡上照序号与判表现算，CPU 不逐态产出数据」只说了上卡，没说存块；判据格 10「同一条流同一块态的判定字与今天逐态相同」逐态相同而块结论不同的情形，在这一格的问法里漏了。

### 4.3 对账口径与待判口径不同：服务漏判的块被当成判过

- 依据：待判口径要「结论在、槽数在、不红、不被强制」（`crash_amplification.rs` 1956–1960 行与 1972 行 is_judged_with_the_slots），对账只问结论在不在（`crates/singlefs-verification/src/crash_amplification.rs:5493` 那一行）。今天两种口径在同一个进程里、判段自己判完就对，差别碰不到；拆开之后 CPU 一侧多半只能靠对账收尾。
- 交错：服务撞显存（今天这条路报本次未跑「gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)」（`crates/singlefs-verification/src/crash_amplification.rs:4964`）），或照 1.1 跳过，清单里剩红块 b 与强制块 f 没判 → CPU 一侧对账：b、f 的结论都在 → 0 块没判 → 这一趟不报本次未跑、照判过收尾。fresh 块漏判照样数得出来，红块与强制块数不出来。
- 推翻材料的哪一句：判据格 4「今天的「撞显存报本次未跑」那条路……在服务里对应什么」——光有服务那一侧报还不够，CPU 一侧的对账口径要换成待判口径。

### 4.4 几条流共用一张卡：锁跟着判器、草稿区跟着判器，服务若共用草稿区就串（推的）

- 依据：防交错的锁是判器的字段「one_dispatch_at_a_time: std::sync::Mutex<()>,」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:568`），草稿缓冲也在建判器时按判器开（同文件 on_card_compiling_these_kernel_sources 里 scratch_lanes 那一段）。今天一个判器一份草稿区，锁与草稿区同生同灭。
- 交错：服务为了「批大小照数据量调、不撞显存」把草稿区按卡开一份、几条流的判器轮流用 → 锁还在各判器上 → 流 X 一次派活的十几趟提交之间插进流 Y 的恢复那一趟 → X 后几趟读到 Y 写的草稿区。今天的代码不会这样，是服务新写那一层的落地风险；锁要跟着草稿区走。

### 4.5 「第一处」取序号最小的：没打中

- 逐态结论不随切法变：每态一段草稿区、一个结论槽，派活参数只带首序号与态数（`crash_judge_gpu.rs` 1002 行的 dispatch_words），切成几批、哪张卡判，每态的结论字相同；判法单元表把切法明说在键外「切法不进结论，不登进来」（`crates/singlefs-verification/src/judging_units_entries.rs:29`）。「第一处」今天只出现在报给人看的地方（红状态正文只收前几条，按到达次序收），不进库，`.claude/rules/test-script-discipline.md:68` 那一句「计数按片的次序相加」管的计数在存块时按块算、不跨块合。服务改批的切法，这一处不换判据。

## 五、材料「实现今天的样子」里的「已有」「没有」，照行号读对不上的

| # | 材料（文件:行） | 材料怎么说（转述） | 实际是什么（今天的仓根树） |
|---|---|---|---|
| 1 | 开头那一段 | 行号是段树的 | 混着两棵树：锁 568、1080、1246，判一次派活 1232，上传判表 1014 是仓根树的行号（段树是 564、1076、1242、1228、1010）；1034、1010、1649 与 crash_amplification 那几行是段树的（仓根树是 1038、1014、1653，判段在 18910 一带） |
| 2 | crash_amplification.rs:20674 | 判段按份与块区间取交集 | 仓根树里没有块区间这一样：只按份过滤「.filter(\|block\| host_shares.owner_of_block(*block) == share.index)」那一行（19005 行）；blocks_of_this_share_in_the_block_range 仓根树 0 处、段树 4 处。「今天的实现」与材料描述的判段差一个特性 |
| 3 | crash_amplification.rs:3728 | pending_blocks 逐块问库一次 | 已判过的块问三次：结论在不在、槽数在不在（1972–1973 行 is_judged_with_the_slots）、红不红（1960 行，扫违例前缀）；没判过的块一次。整片复用的判段正是三次那一种 |
| 4 | crash_amplification.rs:20085 | 十几个进程级环境变量 | 段树那一行是空行，仓根树那一行是给定状态表那一行的输出格式串；两棵树那一行都不是环境变量 |
| 5 | crash_judge_gpu.rs:1002 | 每一批只上传四个字 | 四个字是每条草稿缓冲一份；每次派活还从主机上传结论缓冲的初值（每态 VERDICT_WORDS 个全 1 的字，crash_judge_dispatch.rs 1289 行），记账每片再传四个字（1354 行） |
| 6 | crash_judge_gpu.rs:1154、:1163 | 给定状态表那一路另编 9 条 | 换一组旋钮就换三份源码合起来的摘要，那一格整套编 11 条（crash_judge_dispatch.rs 1038–1039 行）；恢复、走读两条源码没变也重编，命不命中驱动的盘上缓存另说 |
| 7 | judging_units_entries.rs 第 29 行一带 | 改到 on_card_compiling_these_kernel_sources 或 judge_one_dispatch 才换 GPU 判器那一组的键 | 那一组 16 个入口（同文件 57–72 行：内核源码怎么拼、上卡、回读、结论字解码、计数、判表建表与打包），连同工具链、profile、依赖特性、Cargo.lock 里的版本；闭包里任一单元改了都换 |
| 8 | 末段「crates/ 里没有的」 | 判的一方与记结论的一方不是同一份二进制时的对账：没有 | 半有：块属性里记着判它的那一份二进制的 GPU 判器指纹（verdict_store.rs 838 行一段、crash_amplification.rs 3800 行），只是没人比；crash_identity.rs 的 judging_units_key_of_the_group_in_the_table_text 已经在读「别的包的构建脚本写下的键表」（产品 GPU 包的对拍用它取 CPU 参照那一侧的键）——跨二进制取键的代码现成 |
| 9 | 末段 grep | pending_blocks 只命中判段自己问库的那一处与它的调用 | 那条 grep 63 行，多数是别处同名的变量；函数在产品代码里的调用点 4 处：crash_amplification.rs 2377（CPU 路）、4849（GPU 路）、27087（只排计划）与 crash_state_enumeration.rs 2240（崩溃枚举用例那一路，自带 judge_the_pending_blocks_on_the_cards）。给定状态表（按张判）与对拍（crash_library.rs 自己的待判段）是另两条 GPU 判的路、不经 pending_blocks。服务要接的不是一条判段，至少四条 |
| 10 | verdict_store.rs:1861–1871 | 结论相同不写、有一份红整份换、两份都绿而不同报错 | 句句对，但只在两次存块前后错开时成立（1.2）；「有一份红整份换」包括后到的绿换掉先到的红（2.1 乙） |

## 六、「今天每片活付的、常驻之后可能不付的」里，常驻之后照付的

- 判段开头那几遍整条流的扫（排哨兵、数复用、域与进度、录入的账）：都在 CPU 一侧，CPU 一侧要写待判清单就要先算待判，已判的块每块三次往返（第五节第 3 条）。材料说整片复用、现判 0 块的判段占 53.0%–95.1%（量的，材料的数），这一类判段卡一次都不碰，常驻服务一分都省不下；清单还要多写一遍。
- 每片编整个包：录入段、写清单、对账、哨兵仍在测试二进制里，编译目录照「一趟只编译一次……不跨趟」（`.claude/rules/test-script-discipline.md:65`）每趟一编，平均 162 秒、峰值约 9 GiB（材料的数）照付；省的只是判段那一份。
- 卡空着等 CPU 线程准备与解码（材料 6.4%）：解码、恢复字、槽数、计数、登记判定向量（`crash_amplification.rs` 3783 行逐态 register_verdict_vector）是每块都要做的活，挪进服务照样做；服务不流水化就照付。
- 内核冷编（材料：只有走读内核变 157.8 秒、缓存全空 182.6 秒）：内核一变服务就得重起或热换，每台每种卡照付一次；再加 2.1 的版本对账，重起的时点还要与 CPU 一侧的树对齐。
- 起跑到拿到卡（62.4%）：变成在服务队列里等水位线、等卡，等的时间换了地方，不会自己消失；没有数说它变短。

## 没打中的形状

- 认领与续跑：45 格里，认记号键、记号键加写前续租、记号键加写时核认领三条臂在 same（两个服务判出同一份）的 9 格上 lost、silent 全 0：没有版本错位、没有判定不稳时，另立记号键不丢块、不写坏。取样范围：两个服务、一块、任一服务任一步之前崩溃一次、租期任一刻到期一次、重认领至多 2 次，全部交错穷举（不抽样）。
- 没试的形状：三个以上服务、多块（块之间不共享库行，推的：不改上面的结论）、prune-server 自己崩溃重起（认领表启动时从库里重读，`committer.rs` 106–111 行那一段）、组提交失败之后认领表重读、写帧重发去重与认领交错。
- 版本对账：同一棵树、同一种编法、同一份锁文件编出的 CPU 一侧与服务，没找到错位路径（2.3 那一种待量）。
- 判表串：今天每个判器自带判表缓冲（`crash_judge_dispatch.rs` 1014 行），判表串不了；风险只在服务新写的缓存与共用草稿区那一层（4.1 后半、4.4），是推的。
- 「第一处」取序号最小的：见 4.5，没打中。

## 这条腿自己的限度

- 模型只建认领、写、存块三样操作，判出的结论是符号（g、g1、g2、r），不建盘、不建判器；它证的是「照这三处代码的语义，这些交错可达」，不证真服务里多常见。
- 1.2「租期到期不稀罕」、1.3、2.2、2.3、4.1 后半、4.4 是推的；2.3 有命令可量（下一节第 2、3 条）。
- 背景材料的附录（kb 整节抄）只读了 1–176 行与 D13 已定项 17 那一节在 kb 里的原文（`.claude/kb/decisions/13-验证路线.md` 428–446 行）；别的 kb 节没读。
- 行号全取仓根今天的树；段树上同一处的行号不同（第五节第 1 条），主 agent 在开工快照上核引文时要用仓根树。
- 模型是这条腿在本机直接起的（纯 Python、不在仓里、不起仓里的代码，项目代码闸没拦）；文中模型的数在主 agent 复跑之前都按「推的」用。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没建仓副本，没有副本上量出的数；没往统一任务服务交任务，判决要引的数没复跑（命令在下一节）。
- 没建编译目录与仓副本，草稿目录里只有模型目录（68K），不用删。
- 格 1、4、5、7、8、9 没逐格答。

## 要主 agent 跑的命令

1. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=三方攻方腿的认领与租期模型复跑; 范围=1 份模型 45 格; 为什么=判决要引这份模型的数，主 agent 复跑核攻方腿的输出' task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-attack/model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/batch30-pieces-with-widening-rule/copy /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-attack-rerun-run /home/fy5090/.local/share/singlefs/work/gpu-judge-service-r1-attack/model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里有 `name=claim_lease_model_rerun identical_to=` 那一行且 `cells=45`；出现「✗ 复跑输出与」那一行算没过（差在运行目录的 `out/claim_lease_model.diff`）。运行目录用完由主 agent 删。
2. `task-server/task submit -- cargo build -p singlefs-verification --features gpu --release --lib`
   - 在哪起：仓根。
   - 看哪几行：`exit=0` 与末行 `build=`；之后读 `<build= 目录>/target/release/build/singlefs-verification-*/out/judging_units_keys.tsv`。
3. `task-server/task submit -- cargo build -p singlefs-verification --features gpu --release --test root_ring_turn_in_the_make_filesystem_session`
   - 在哪起：仓根。
   - 看哪几行：同第 2 条，读它那一份 `judging_units_keys.tsv`。第 2、3 条两份里 gpu_judge 那一行与依赖特性那几行逐字比：不同则 2.3 成立（测试二进制与只带 gpu 特性编的库键不同），相同则 2.3 不成立。profile 照崩溃放量入口实际编测试用的那一种换（我没核入口用的是不是 release）。两条的编译目录用完 `task-server/task clean <号>`。
