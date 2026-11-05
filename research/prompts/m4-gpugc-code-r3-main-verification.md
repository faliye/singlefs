# m4-gpugc-code-r3：七·3 GPU 垃圾回收代码轮第三轮判决（2026-11-02，最终轮）

正文 `research/prompts/_m4-gpugc-code-r3-body.md`；正推腿 `research/prompts/m4-gpugc-code-r3-sonnet-output.md`（sha256 `263c715081134ca78e4bfc9e53bb626280aada2819440459445aef635460e42f`）；攻方腿 `research/prompts/m4-gpugc-code-r3-opus-output.md`（sha256 `599f54ba6909334314d252eefe6a63e3af31b540b1719f35b55a9ce4db6b0a2c`）。被判的是第 59、60 批合入的 7 份产品代码文件：`crates/singlefs-background-gpu/src/background_gpu.rs`、`crates/singlefs-background-gpu/src/gpus_of_this_host.rs`、`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs`、`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`。行号都取冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r3/repo/`。这是第三轮，判完不开第四轮；这一轮新冒出来的零轮形态写进第五节、标「零轮」。

## 一、核查

- 快照：冻结副本对 `research/prompts/m4-gpugc-code-r3-snapshot/sha256sums.txt` 12 份全对上（开轮时核过），两条腿跑着的这段时间主 agent 没改这几份文件。
- 引文：两条腿各自交回时跑过 `cite-check.py`，正推「✓ 核了 41 处引文、81 处裸行号，对上 41 处」，攻方「✓ 核了 13 处引文、23 处裸行号，对上 13 处」；判决要用的裸行号下面各条逐处读码核过。
- 复跑：
  - 正推事实探针（模型目录拷到 `~/.local/share/singlefs/work/m4-gpugc-code-r3-main-check/sonnet-model/`，任务 5114）：24 行 FACT 与 `out/facts.expected` 逐行相同。
  - 正推 C3：`cargo test -p singlefs-harness --test compaction_off_the_calling_thread_and_admission -- a_compaction_batch_that_would_push_the_available_below_the_demand_is_refused_by_the_admission_and_writes_nothing`（任务 5115）「test result: ok. 1 passed」。
  - 正推 C4：`cargo clippy --no-deps --all-targets --all-features -p singlefs-harness` 加七条 lint（任务 5116）退 0。
  - 正推 C1（28 行证红在入库树上重做）、C2（测试条数重量）不跑：实现员副本就是主工作区加这两批补丁（打补丁前 `git apply --check` 过），条数 151、384、18 本来就是在主工作区量的（任务 4475–4477）。C5（重建前两轮冻结树）不跑：前两轮判决要用的数当轮都复跑过。
  - 攻方探针（模型目录拷到同一复核目录 `opus-model/`，任务 5117）：脚本退 1 只因腿写的 ATTACK 期望是 11 行、打出 7 行；读到的行原样：`ATTACK name=noop_chunk_buffer_with_contents_short … outcome=panicked:Failed to get mapped range for buffer created with mapped_at_creation: MapRangeError("Validation Error …`、`ATTACK name=noop_chunk_buffer_created_unmapped_short … outcome=Err:OutOfMemoryCreatingTheBuffers { …`、`ATTACK name=noop_readbacks_short_inside_the_scope … outcome=Err:OutOfMemoryCreatingTheBuffers { …`、`ATTACK name=noop_readback_short_outside_the_scope … captured_by_the_closed_scope=None reports=["uncaptured:Out of Memory"]`、切换三种形状各一行（点名掉了的卡拒、两个上限拒、一个上限收）。缺的 4 行是重交计数那几条探针，那一处由主 agent 读码坐实（第二节 T1 第 1 条）。

## 二、判定与去向

### T1　「已定」项撑不撑得住

| # | 发现 | 现查 | 判 |
|---|---|---|---|
| 1 | 第 59 批第 2 条的改法本身：重交之前那一等等到了，重交那一次在建缓冲时显存不足、一次都没交出去，照样记一次重交（攻方） | `waiting_for_the_gpu.rs` 第 210–216 行：`Ok(())` 分支先 `timeouts_of_the_batch.resubmissions += 1;` 再 `submit_once(gpu)` | 打中，采。第二轮判决第三节第 2 条验收「只数真交出去的重交」 |
| 2 | 产品建一段自己的缓冲用 `create_buffer_init`，显存不足时 wgpu panic，走不到 D26（后台整理与放置回收） 已定项 13 第 3、4 条（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 247–248 行）「切小重切、这一批转给别的卡」那一支（攻方，零轮） | `background_gpu.rs` 第 1011–1018 行 `buffer_with_contents` 调 `create_buffer_init`；wgpu 30.0.1 `src/util/device.rs` 第 68–74 行 `create_buffer` 之后对映射区 `.expect("Failed to get mapped range for buffer created with mapped_at_creation")`；探针原行 `noop_chunk_buffer_with_contents_short … outcome=panicked:…`（noop 后端，不是真卡） | 打中，采。真卡上没造出，noop 后端坐实 |
| 3 | 「重切时已做完的片不留」（2026-11-01 补进第 4 条）代码做了，没有测试或变异行钉它（正推） | 正推报告 1.2：替身卡 `free_bytes` 五处赋值没有「先做完一片、后一片建不成」 | 采：补测试与变异行 |
| 4 | 七·3 验收「塞一个多分配一块缓冲的故障要红」没有用例（正推） | 正推报告 1.2；`memory_limit_and_chunks.rs` 第 351 行 `hold` 的 `assert!` 守的是公式算出的账，不数真建的缓冲 | 采：补用例与变异行（账按真建的字节核） |
| 5 | 晾下又请回的卡不按新上限重量代价（正推） | `background_gpu.rs` 第 1697 行只对从没量过的卡重量；条款第 3 条（第 247 行）「挂着切换给了它装得下的上限，从下一批起照新上限重新量代价、分活」 | 采：照条款字面，被晾下的卡回来时照新上限重量 |
| 6 | `UnusableAfterResubmissions.resubmissions` 恒写常数 3（正推） | 报的是常数、不是这一批真交出去的次数 | 采：随第 1 条一起报实际次数 |
| 7 | 只现枚举一次 × 挂着时卡掉了、泄漏按次累加 × 上限扣法、叫起计数六种序列（两条腿） | 攻方探针切换三行与条款第 4 条一致；正推 1.3 | 没打中；各只一份样本（每条腿各一次），照「一条腿只抽一次样不算一次观测」不当没有问题，登 C952（第三节第 9 条） |
| 8 | 「赔的卡照用，并报出来」（第 2 条，第 246 行）：`participating_gpus()` 产品路径零调用者，「赔」没有判法（正推） | 正推报告 1.1 | 交用户（第五节） |

### T2　未定项清单的完整性

正推报告 2.1 的 28 条里 12 条没有去向，逐条：

| 正推表行 | 去向 |
|---|---|
| 8（账超上限是 `assert!`） | 并进第三节第 4 条（账按真建的字节核） |
| 9、10、16（只抽一次样的没打中形状；`region_scan.rs`、`checksum.rs` 两处快写法只有读码样本；第 54、55 批十一项读法只有辩方一份样本） | 登 C952，去向七·4 覆盖 |
| 13（接手的卡再撞一次）、14（量代价那一段建不成算没开成）、15（`#[must_use]` 接住不读）、20（续做里停下那一趟不交报警） | 不做：第二轮判决已对条款判过，读法与条款字面不相左 |
| 17（进程里泄漏账由调用方持有，产品挂载工具今天没有持有方） | 登 C952 |
| 21（「各自切片交过」读成每张卡分到活、各交过一次） | 交用户（第五节） |
| 22（共用测试模块不在派发文件单里） | 已办：调度记录第 60 批交回那一行照收 |
| 27（第二轮辩方探针没跑成） | 不做：它要量的由第二轮攻方探针同一处量到 |

kb 那一半（主 agent 写回）：`.claude/kb/milestone/04-fourth-txn.md` 第 149 行七·3 验收「（攒批、缓冲、暂存区一起算）」删「攒批」（用户 2026-10-30 定删跨批攒，D26 已删、这一处漏了）；`.claude/kb/mount-parameters.md` 代码落点行号、C942（整理一趟里删除续做的步数不进发布预算）与 C943 的行号现查重写；C906（整理批不过常规准入与条款对不上）的还账条件「用例绿了再还」已满足（任务 5115），照它的去向关。

### T3　前两轮报出的数字能不能核

| # | 发现 | 判 |
|---|---|---|
| 1 | 前两轮冻结树与实现员副本已删，第一轮判决的复跑命令带内存包装（服务今天会拒），部分数原样重跑不出（两条腿） | 照收：判决当轮复跑过、产物还在仓里；冻结证据过轮不改。不改前两轮判决 |
| 2 | 「第 5265 行」那一行今天在第 5213 行（攻方） | 行号漂移，名字不变；调度记录里按名字认，不改 |
| 3 | 变异轻型 317 个「抓到」里只有 20 行现判（正推） | 照收：复用判定库是设计，现判的 20 行就是这两批改到的 |
| 4 | 第 60 批「回读暂存」那条替身测试钉的是建缓冲的先后次序，不证明真卡上 wgpu 照这样接错（两条腿） | 部分由攻方 noop 后端探针补上（回读暂存在范围里被接住、范围外成了 uncaptured），真卡上造不出；照收，登 C952 |

## 三、交给下一批的实现员（第 63 批，按文件聚簇）

一簇：`crates/singlefs-background-gpu/src/`（`background_gpu.rs`、`waiting_for_the_gpu.rs`、`memory_limit_and_chunks.rs`）、测试与 `crates/mutations.tsv`。

| # | 问题 | 验收 |
|---|---|---|
| 1 | 重交次数在交出去之前就加（T1 第 1 条） | 「重交了几次」只在那一次真交到卡上之后才数；替身用例钉「等到了、重交那一次建缓冲显存不足没交出去」记 0；一行变异（改回先加）要红 |
| 2 | `create_buffer_init` 显存不足时 panic（T1 第 2 条） | 一段自己的缓冲也在显存不足的错误范围里建，建不成交回 `OutOfMemoryCreatingTheBuffers`、不 panic（例：建不带映射的缓冲再 `queue.write_buffer`，写法实现员定）；noop 后端用例（照攻方探针 `noop_chunk_buffer_with_contents_short` 的造法）钉交回 `Err`、不 panic；一行变异（改回 `create_buffer_init`）要红 |
| 3 | 重切不留没有钉（T1 第 3 条） | 替身用例「先做完一片、后一片建不成」：这一段照新切法整段重交，已做完的片的结果不用，交的次数钉绝对值；一行变异（留已做完的片）要红 |
| 4 | 多分配一块缓冲要红（T1 第 4 条，T2 第 8 行） | 账核真建的缓冲字节（经第 60 批的 `DeviceCreatingTheBuffersOfAChunk` 替身数），塞一个多建一块缓冲的故障，峰值不超上限的断言红；一行变异要红 |
| 5 | 晾下又请回的卡不重量代价（T1 第 5 条） | 被晾下的卡在一次切换给了装得下的上限之后，下一批照新上限重量代价；替身用例钉重量了一次；一行变异要红 |
| 6 | `resubmissions` 恒写 3（T1 第 6 条） | 报这一批这张卡真交出去的重交次数；替身用例钉绝对值 |

## 四、复跑与核不动的

- 正推事实探针、C3、C4 与攻方探针主 agent 复跑过（第一节）。
- 攻方的重交计数探针没打出（4 行缺），那一处由主 agent 读码坐实。
- `create_buffer_init` 的 panic 在 noop 后端坐实，真卡上造不出，没核。
- 两条腿都没起 GPU；真卡用例按用户 2026-11-01 定延到七·3 收口。

## 五、交用户

| 岔路 | 事实 | 候选 |
|---|---|---|
| 「赔的卡照用，并报出来」怎么报（零轮） | 条款第 2 条（第 246 行）；`participating_gpus()` 产品路径零调用者；「赔」按第 7 条（第 251 行）是 GPU 模式比 CPU 模式多占主机 CPU 时间 | 七·5 对照组量出来、写进文档与报告就算报出来 / 运行时也报（挂载时量的代价比 CPU 模式大的卡标出来，要改代码） |
| 「每张卡都能用来分片 GC」的读法（零轮） | 用户 2026-11-01 原话；第 60 批读成每张卡都分到活、各交过一次，不是每张卡的那一份再被 1 GiB 切成几片 | 照第 60 批的读法 / 每张卡都要切成几片（数据造到每张卡 1 GiB 以上） |
| C923（后台 GPU 包开卡编管线不在等的上限之内） | `.claude/kb/checks-owed.md` 第 578 行：驱动编管线卡住时挂载跟着卡住 | 七·3 收口前修（放进能限时等的地方编） / 留到七·4 之后 |
| C942（整理一趟里删除续做的步数不进发布预算） | `.claude/kb/checks-owed.md` 第 595 行：攻方量到预算 1 的一趟发了 2 版 | 删除续做算进发布预算（改代码） / 不算（条款补一句） |

## 五之一、用户定（2026-11-02 弹窗，原话）

- 赔的卡：「挂载和运行无关 。 挂载是一次性的。 可以计算，但是不需要记入性能。」——挂载时量的代价可以算，不报、不记进性能；D26 已定项 13 第 2 条去掉「并报出来」，代码不改。
- 分活与分片：先答「每张都能用。不是每张都能获取的任务。按照任务量和卡的顺序分配。不强制要求每张卡都要有任务。但是你要测任务饱和的情况。」，再弹窗选「按卡的顺序填满」——改第 2 条与代码：按卡的顺序（点名的按点名次序，没点名的按 PCI 地址次序，主 agent 的读法）先把前面的卡填到它的上限，活多了才用后面的卡；挂载时量的代价不再用来分活；测试不要求每张卡都分到活，另测活多到每张卡都填满的饱和情况。并进第 63 批。
- C923（后台 GPU 包开卡编管线不在等的上限之内）：用户先问「挂载属于驱动安装，和运行有什么关系吗？ 我理解错了吗？」，主 agent 说明这里的挂载是挂文件系统时开卡编管线、不是装驱动之后，用户答「gpu 挂载要什么时间上限？ 五分钟 ？ 开了gpu 要等GPU挂载成功后才开始正式运行，挂载失败了就报错，没有callback。」——挂载时开卡（含编管线）加 5 分钟的上限，到点照「GPU 用不上」报错拒挂，挂载等开卡成功才往下走，不做回调。并进第 63 批。
- C942（整理一趟里删除续做的步数不进发布预算）：「不算。 我觉得删除续作这就是个伪概念。 整理走的是整理，整理是标记。整理后删除是删除。两个是不一样的事务。」，主 agent 说明代价后再答「照不算改条款。 整理是整理。 整理后还至少有两个事务 ，至少有移动和删除。 如果整理中包括这两个 那么事务就不是原子的。」——代码不改，第 6 条补一句删除续做是删除自己的事务、不占整理的发布预算，C942 结案。

- 删除与整理（用户 2026-11-02 接着说明，原话）：「位图交给删除 可以按照你说的错。」「删除和整理是两件事，整理前删除一次是为了效率。但是不能理解为删除续作。应该理解为前置优化。也就是说整理前也可以不删除。删除仅仅是优化而已。」——整理判出的死空间照现有代码由分配器回收、不交给删除事务；整理一趟开头先走一遍删除是可选的前置优化，不叫「删除续做」、不是整理的一部分，整理也可以不先删除；D26 已定项 13 第 6 条那一句照这个说法写（第 64 批写回员交回后由主 agent 改）。代码今天每趟都先走一遍删除，照留（可选不等于改成不走）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-02 不受影响：这一轮打中的都是代码没照条款做（第 3、4 条那一支走不到、重交只数真交出去的、请回的卡重量代价），条款不改；第五节四条交用户，定了再改 |
