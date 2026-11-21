# m4-gpugc-mount-runtime-code-r1：云端正推腿报告（Sonnet）

被判代码：冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo/`，下文 `crates/...` 都是它里面的路径与行号。
开工核了快照：`sha256sum -c research/prompts/m4-gpugc-mount-runtime-code-r1-snapshot/crates-sha256.txt` 在副本里 11 份都 OK（量的）。
本腿没起 cargo、没交任务；读码推出来的数标「推的，没量过」，量过的标「量的」并写是哪个量。
自写模型：`research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-model/`（回退门在根环几何 S 上放几次整理发布；复跑命令在「要主 agent 跑的命令」一节）。

## 先说结论

| 格 | 结论 |
|---|---|
| T1 | 部分成立：线程自己跑、前台入口都经锁、线程体与同步一拍同一段代码都成立；「测试绿能外推到线程」「真卡用例能外推」不成立；E202 走核心层入口，不走运行库 |
| T2 | 门的「状态」与条款同一个口径（成立）；门留 8 槽余量在 S≤5 上不成立（S=4 门永关，S=5 门只放 3 次，一条意图至多 5 次）；破例、备忘、预算三者不矛盾但有一格会死局 |
| T3 | 删除走空那一遍整段持锁、不看 `stopping`，不守「每次发布之间放锁」；冻结重发要等下一拍（默认 30 秒）；「没定的一处」是真死局，建议这一批不修核心层、先让运行库把它报出来 |
| T4 | 5% 夹整段数、F 与 U 关、盘上 24 字节不变都成立；K=16 在 S=5 上一份预算（挂载续做之后）做不完一条意图 |
| T5 | 次序差别成立（空列表加上限 0 在产品拒成上限 0，核心层拒成空列表）；开卡器有两处没钉住的口子 |
| 检查单 | 走不到的理由：实现员的「无」不成立（产品代码新加 8 处 `expect`/`panic!`）；类型与名字成立（带一处限定）；不兼容旧代码成立；不新建判定库成立；多机可用成立；测试优先多线程成立；`--item` 成立；新崩溃流成立 |

## T1　运行库真的「接上了」吗

**1. 挂载之后调用方不碰后台入口，GC 自己跑——线程路径成立，「跑到腾出能开的空段」只在同步一拍的测试里证过。**
- 挂载入口 `MountedRuntime::mount` 先造会话再起线程：`crates/singlefs-mount-runtime/src/lib.rs:651`（入口）、`:671`（`std::thread::Builder::new()`）、`:673` 起线程体。线程体 `maintenance_thread_body`（`:591`）第一圈就把 `next_compaction_at` 取成此刻（`:594`），所以挂载后第一拍立刻判水位，不等一个周期。
- 线程体到期调的是 `run_one_maintenance_pass`（`:598`），那一拍里每次调放锁的后台入口（`:520`，核心层 `crates/singlefs-core/src/mounted_session.rs:6546` 的 `run_compaction_in_the_background_for_at_most_one_publish`）。调用方挂载之后确实不碰任何后台入口。
- 「腾出能开的空段」：`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:156` 的用例走 `run_one_maintenance_pass`（不起线程，假钟），断言做完一条意图、记录口径全空段涨了、根环转一圈后开得成段。起真线程的用例（`:712`）只等 `observations().compaction_publishes` 非 0，没断言腾出空段。所以「线程自己把空段腾出来」靠两个用例拼起来，没有一个用例是线程加空段加开段。
- 推翻：把线程体里的 `run_one_maintenance_pass` 换成一个什么都不做的函数，用例 `:712` 仍绿，则说明它没钉住「腾出空段」。

**2. 前台全部入口都经那把锁——成立，一处例外是读观测。**
- 取会话锁的唯一函数是 `lock_the_session`（`crates/singlefs-mount-runtime/src/lib.rs:424`）。前台入口逐个核过都调它：`accept`（`:814`，锁在 `:819-822`）、`publish_pending_now`（`:833`，`:841-844`）、`publish_user_change`（`:855`，`:863-866`）、`switch_background_compute`（`:877`，`:883-886`）、`with_the_session_and_devices`（`:906`，`:910`）；读路径 `read_at` 走 `with_the_session_and_devices`（文档串 `:902`）。
- 线程这一侧也经它：`publish_when_due_once`（`:458`，`:461`）、`resend_the_frozen_publish_if_any`（`:477`，`:478`）、后台入口（`:518`）。
- 例外：`observations()`（`:924`）只取观测的小锁，不取会话锁；`unmount`（`:953`）要 `self` 的所有权，线程 join 之后 `Arc::try_unwrap` 取 `locked`（`:965-968`），不排队，因为这时没有别的持有者。类型上拿不到 `&self` 与 `self` 同时存在，成立。
- 口子：`with_the_session_and_devices` 交出 `&mut SessionAndDevices`（字段 `pub`，`:331-334`），闭包里能 `allocator.compaction_mut()` 改只供测试的开关，包括 `ForcedToStartForTests`。这是已登记的 C921（只供测试的开关在发布构建里够得着），不是这一批新增；运行库的 `RuntimeParameters` 本身只给批大小与水位（`:59-69`），产品挂载入口 `crates/singlefs-core/src/mount.rs:9579-9618` 里（`CompactionSwitches::PRODUCT` 拆开再装批大小）也只换批大小。

**3. `run_one_maintenance_pass` 与线程体是不是同一段代码——同一段，线程多出来的四样测试够不着。**
- 线程体到期那一支就是 `self.run_one_maintenance_pass()?`（`:598`）；没到期的那一支调 `publish_when_due_once`（`:606`），与一拍里调的是同一个函数（`:507`、`:585`）。
- 线程独有、同步测试看不到的：睡眠与到期时刻的算术（`:608-620`）；`stopping` 的检查（`:595`、`:615`）；`catch_unwind` 与 panic 记账（`:674-688`）；`Ok(Err(_))`（线程因运行库已坏而退出）被吞掉、不送任何事件（`:678`）。
- 推的，没量过：一拍比周期长时 `next_compaction_at` 落在过去，`sleep_for` 为 0（`:610`），线程接着立刻跑下一拍。不是缺陷（拍内每次调用有节奏闸与预算），但周期不是下限。

**4. 真卡用例是小池、一张卡，不能外推到几张卡、大盘、线程。**
- `crates/singlefs-background-gpu/tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs:143` 的用例走 `mount_without_the_maintenance_thread` 加假钟一拍一拍推（文件头注释写了「真线程下 GC 与前台的次序取决于调度」），池是共用的小内存池（`common_pool_on_memory_devices`）。服务交几张卡它就点几张（`handed_over_gpus_or_report_not_run`），所以几张卡取决于派到的机器，用例本身不钉张数。
- 没有任何一个用例同时具备：真线程、GPU 在锁内等一批、前台同时调 `accept` 或 `publish_pending_now`。GPU 一批持锁时前台被挡多久（T3）全靠读码。
- 推翻：补一个起线程的真卡用例并在批中途让前台调 `accept`，量到的等待超过 `WAIT_LIMIT` 的若干倍，则这里的「一批之内持锁」就是实测的饿死。

**5. E202 与七·4 的测试走不走运行库——不走，差别在门、备忘、破例、预算、放锁五样。**
- E202 装置不引运行库：它调核心层的 `run_compaction_in_the_background`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/lockstep_runs.rs:50`、`:6363`、`:7894`），挂载用 `mount_writable_with_background_compute_settings`（`:4295`），开关装的是 `CompactionRunForcing::ForcedToStartForTests` 与 `ForcedForTests` 批大小（`:3343-3347`）。
- 所以 E202 量的是「整趟一次持锁、没有回退门、没有备忘、没有破例、不读挂载预算」的 GPU 对 CPU 主机 CPU 时间，不是运行库每次发布放锁、每次发布前读一遍根环（回退门）的那一条路。门每次发布读 3S 个根槽与它们的树表（`crates/singlefs-core/src/mount.rs:4251-4308`），这部分代价 E202 没算进去，两个后端都有，不改对照的符号，但绝对值偏低。推的，没量过。
- 七·4 一族测试里用 `ForcedToStartForTests` 的：`grep -rl` 在 `crates/` 下命中 12 份（量的：含 `compaction_plan.rs`、`mounted_session.rs`、`e202_gpu_cpu_gc_comparison.rs` 与 8 份验证包测试）。它们钉的是「一趟怎么推」，不是产品路径（见 T2 第 6 点）。

## T2　回退门、余量、破例、备忘四样放一起

**1. 门数的状态与「保住 4 个用户状态」是不是一件事——与 D16（发布语义）已定项 1 的口径是一件事；与「用户内容的 4 个版本」不是。**
- 条款：「回退候选集保留最近 4 个可退到的不同状态」（`.claude/kb/decisions/16-发布语义.md:31`）；「非空」按树表里用户可见的树根指针比，去重也按指针（`.claude/kb/decisions/16-发布语义.md:48`）。门读的口径同一份：`crates/singlefs-core/src/mount.rs:4195` 的 `newest_first_distinct_states` 与 `:3841` 的 `user_visible_trees_changed`（只比指针，不比字节内容）。
- 整理搬了用户可见树的根单元，树表里那条根指针就变，按这个口径它是一个新「状态」（`crates/singlefs-core/src/mount.rs:4211-4212` 的文档串写明「整理搬了用户可见树的根单元的那一批照字节比算非空」）。所以门数的是指针状态，一次搬根的整理发布能把「4 个状态」凑出来而用户内容的不同版本只有 3 个。推的，没量过；搬根要 R 正好罩着 inode 树或 extent 树的根单元，概率小，但条款没排除。
- 这不是门的缺陷，是条款口径（准入抬 F 的上限用同一个口径）；要「4 个用户内容版本」得改 D16 已定项 1 的「非空」定义，不在这一批。

**2. 8 槽余量在 S≠8 的几何上不成立：S=5 门只放 3 次（S=4 门永关，只有显式给 S 的入口造得出）。**
- 门读到的是：下一次发布与之后 8 次发布写的共 9 个根槽里的根都走了之后，还剩几个不同状态（`crates/singlefs-core/src/mount.rs:4257-4264`、`:4286-4293`）；关门条件是剩下的少于 4 个且比此刻的状态数少（`crates/singlefs-core/src/mounted_session.rs:4104-4106`）。根环是 3 个区域各 S 槽（`crates/singlefs-core/src/recovery.rs:1963-1972`），S 在格式里取 4 到 16（`crates/singlefs-format/src/lib.rs:299`、`:303`），显式给 S 的建池入口取 4 到 16 都行（`:308` 的 8 是测试与装置常取的值）；mkfs 的默认值是 min(8, ⌊P × chunk ÷ 槽距⌋)，小于 5 拒（`crates/singlefs-core/src/root_ring.rs:92-110`、`:45`），所以 mkfs 出来的池 S 在 5 到 8：S=5 到 7 来自槽距大的盘，S=4 只有显式给 S 的入口造得出。
- 剩下的根数是 3S−9：S=4 只剩 3 个根，最多 3 个状态，此刻有 4 个以上状态就永远关；S=5 剩 6 个，整理每发一次就占去一个新根位（空发布），第 4 次就关。
- 自写模型 `research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-model/gate_geometry.py` 按这套判法算「环里每个根都是新用户状态」时门放几次连续的整理发布（模型输出 `out/model.out`，我在草稿里跑过一遍，主 agent 复跑命令在最后一节）：S=4 放 0 次，S=5 放 3 次，S=6 放 6 次，S=7 放 9 次，S=8 放 12 次，之后每加 1 放 3 次，公式 3S−12。S=8 的 12 次与实现员 r3 报告里「最后一次改动之后 12 次」的读数一致（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-writer-r3/result.md` 第 1 条的表，实现员量的；模型这一格是对照）。
- 一条默认批大小的意图至多 5 次发布（`crates/singlefs-core/src/compaction_plan.rs:430-431` 的文档串）。S=5 放 3 次 < 5：这条意图要跨用户改动才走得完；S=4 从不签。余量常量 8 的来历（写行加暖机至多 3、续做至多 5，`crates/singlefs-core/src/compaction_plan.rs:1524-1527`）与 S 无关，窗口却随 S 缩，两者不是同一个量。
- 会不会让一条意图永远走不完：门关着时意图留在盘上、R 的排他没撤（门这一支只 `finish`，不撤排他，`crates/singlefs-core/src/mounted_session.rs:4356-4366`）。S=4 或 S=5 且用户一直不写时，R 里 64 个槽不能分配；只有低于底额破例才有出路（第 3 点）。S≥6 时用户写一次，窗口里多一个状态，门就重开。
- 推翻：模型的「每个根都是新状态」比真实环紧；真实负载里环里有一半是整理的空发布时，同一个 S 放得更少（模型第二张表：S=8、每个状态后跟 3 个空根，只放 3 次）。反过来，环里只有 3 个状态以内时门不关（`after < now` 为假），S=4 的小池刚开始用时整理照跑。

**3. 破例与门、余量的先后——破例在门之前，且只在新一趟的第一次发布上有效。**
- 判法的次序：预算用完停（`crates/singlefs-core/src/mounted_session.rs:4279`）→ 至多一次发布的跨度停（`:4301`）→ 前台 fsync 与写带宽（`:4313-4351`，破例不绕）→ 回退门（`:4356`，条件 `is_the_rollback_gate_judged && !is_this_publish_the_exception`，`:4353-4355` 定义破例为预算规则是 `AtLeastOnePublishWhileTheForegroundCannotWrite` 且这一趟还没发过）→ 发。
- 起跑那一判决定预算规则：`foreground_cannot_write`（`:3234-3235`，任一块盘全空段少于底额 1，`crates/singlefs-core/src/compaction_plan.rs:1849-1861`）为真才给破例预算（`:3327-3329`）。接着推的那一趟（`:3237-3253`）走 `LeftOfTheBudgetGivenByTheLastUserChange`，预算为 0 直接 `NotStarted`，**破例也不开**。所以破例只出现在新一趟的头一次调用：默认每个周期（30 秒）一次不过门的整理发布。放锁的后台入口每次调用只发一次，这与「每趟至少发 1 次」不矛盾，但一条 5 次的意图低于底额时要 5 个周期（150 秒）才走完，期间每次都推走一个根。推的，没量过。
- 破例不判余量：余量在门里，门被绕过就一并不判。这是条款要的（「只有全空聚簇段数低于底额……前台已写不进时破例」，`.claude/kb/decisions/26-后台整理与放置回收.md:252`）。

**4. 备忘与门、破例的组合。**
| 备忘里记的停因 | 前台写得进 | 前台写不进（低于底额） |
|---|---|---|
| 门关（`CompactionPublishWouldLeaveFewerThanFourRollbackStates`） | 备忘挡住，不重判（`crates/singlefs-core/src/mounted_session.rs:3287-3305`） | 备忘放行，破例发一次（过滤在 `:3292-3298`） |
| 无产出 / 没有合格段 / 空洞不够 | 备忘挡住 | **备忘照挡**：过滤只放门关一种。死局就在这一格（T3 第 4 点） |
| 预算用完、让路、写带宽、空间拒 | 不进备忘（`crates/singlefs-core/src/compaction_plan.rs:660-683`），下一拍重判 | 同左 |
- 备忘只在定时兜底那一支判（`crates/singlefs-core/src/mounted_session.rs:3262-3306`）；用户改动之后那一支（`:3255-3261`）与接着推的一趟（`:3237`）不看备忘，因为它们都紧跟一次发布，备忘本来就该作废。成立。

**5. 挂载预算把写行、暖机算已过发布——成立，两处限定。**
- 这次挂载头一次判后台起跑时，已过发布数取 `publishes_after_the_newest_state`（根环里比最新用户状态的根更新的有效根条数，`crates/singlefs-core/src/mount.rs:4219-4220`、`:4295-4298`），与现有计数取大（`crates/singlefs-core/src/mounted_session.rs:3215-3233`）。写行、暖机、挂载续做都是空发布，都在这条数里。读不通当预算已扣完（`:3223-3229` 取满额 `3S−4`）。一次用户改动发成就清掉这个待读标记（`crates/singlefs-core/src/compaction_plan.rs:1022-1025`）。
- 限定一：挂载续做若恰好搬了树的根单元，那一次发布算「状态」，会把计数清到它之后，少算前面的写行。限定二：S=5 时 `3S−4=11`，写行加暖机加续做最多 8，剩 3，不够一条新意图的 5 次（`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH` 的文档串自己说「合起来是 8」，`crates/singlefs-core/src/compaction_plan.rs:1524-1526`）。推的，没量过。

**6. `ForcedToStartForTests` 绕过门、备忘、破例、挂载预算——所以现有测试里一部分不是产品路径。**
- 四处读它：`crates/singlefs-core/src/mounted_session.rs:3209-3212`（只有 `FollowTheWatermarks` 算「照产品判」，挂载预算、破例、备忘三样都挂在这个布尔上）、`:4183`、`:4269-4273`（门只在 `FollowTheWatermarks` 判）、`crates/singlefs-core/src/compaction_plan.rs:448-456` 的文档串自己写了「备忘、挂载那一份预算从根环读、破例与回退门这一支都不判」。
- 主 agent 判过「留着」，这一格只回答「测试是不是产品路径」：回退门目标 `crates/singlefs-verification/tests/compaction_rollback_gate_keeps_four_user_states.rs` 与运行库 15 条用例走 `FollowTheWatermarks`（用 `watermark_crossing_forcing` 强制越线，不强制起跑），是产品路径；其余 8 份验证包测试加 E202 用 `ForcedToStartForTests`，钉的是搬迁与节奏，不是这四样。
- 水位「强制越线」`CrossingForcedByTheTestOnlySwitch` 在产品路径里照走门与备忘（`crates/singlefs-core/src/compaction_plan.rs:1822-1826`），所以门的用例没有绕过门。成立。

## T3　哪一条最脆弱：前台饿死、死局、停不下来

最脆弱的是第 2 点（删除走空整段持锁）与第 4 点（低于底额的死局）；第 5 点（冻结重发等一个周期）次之。

**1. fsync 计数在锁外的时序：计数读到 0 之后马上来 fsync——不丢，只多等一批。**
- 登记在取锁之前：`let _waiting = self.a_foreground_fsync_is_waiting();`（`crates/singlefs-mount-runtime/src/lib.rs:839`），计数加在 `:934`，守卫在 `publish_pending_now` 返回时才减（局部变量逆序 drop，锁先放、计数后减）。后台问计数在每批之前（`crates/singlefs-core/src/mounted_session.rs:4321` 调 `crates/singlefs-mount-runtime/src/lib.rs:363-369`）。计数读到 0 的那一刻后台已经决定发这一批，fsync 这时才登记就只能等这一批结束：多等的上限是一次调用（排空待发加走空加门读根环加一批）。
- 一次调用结束放锁之后，后台线程紧接着又取锁（`:585` 的 `publish_when_due_once`），标准库互斥锁不保证先来先得（推的，没量过），fsync 线程可能再输一次；下一次调用在第一批之前看到「在等」就让路（`:4321-4333`）。所以最坏是等两个短临界区加一批，不是无界。
- 推翻：真线程下前台连续 fsync，量到每次 fsync 的等待分布里有超过「一批加两次取锁」的尾巴。没有用例钉这一条（T1 第 4 点）。

**2. `accept` 被后台一批挡多久——一批；但删除走空那一遍整段持锁，不看 `stopping`，不守「每次发布之间放锁」。**
- `accept` 不登记「前台有 fsync 在等」那个计数（`crates/singlefs-mount-runtime/src/lib.rs:814-827`），它挡在后台那次调用后面。一次调用持锁做的事：排空待发（`crates/singlefs-core/src/mounted_session.rs:6651`）、删除走空、回退门读根环、一批整理（GPU 在锁内等，上限默认 10 秒，`crates/singlefs-core/src/background_compute.rs:619`，条款允许同卡重交至多 3 次，一段的等待最多约 40 秒，三段各自计，量级是数十秒到两分钟，推的，没量过）。
- 删除走空：`crates/singlefs-core/src/mounted_session.rs:4207-4240`，只要删除队列不空，每次后台调用开头就整段 `publish_the_queue_until_it_is_empty`（`:4220`）。这一段循环发布直到队列空，期间不问「前台 fsync 在等」、不查 `stopping`、不放锁，也不占整理的发布预算。条款允许前两样：「整理开头先走的那一遍删除不是整理批，不受它约束」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。条款没说的是锁：用户第四问的答复是「每次发布之间放锁」，这一遍整段持锁。队列长度没有上界（大文件删除排很多批），所以这是前台饿死与卸载变慢的真口子。同一段也不过回退门（门只在 `:4356` 判整理批），走空的发布推走根环里的根；它们改了用户可见树，算新状态，是增状态而不是挤状态，所以对「保 4 个状态」无害，推的，没量过。
- 窄改法（不改条款）：走空那一遍改成每调用至多走一步（或一批）就返回，剩下的下一次调用接着走，放锁在步与步之间。代价：队列长时一趟要更多次调用，一拍的调用次数上限（`crates/singlefs-mount-runtime/src/lib.rs:55`，64 次）可能先到，下一拍接着做；要改核心层 `run_compaction_on_with_the_region_check_of_the_intent_deletion` 的开头，并补一个「队列长、前台同时调 accept」的用例。这一批要不要修由主 agent 判；我的判断：要修，因为「一把锁、每次发布之间放锁」是这一批的核心承诺，这一处是例外而没写明。

**3. 周期线程在前台持续写时拿不拿得到锁——拿得到，但没有证据。**
- 标准库互斥锁，持锁者放锁时唤醒一个等待者；前台每次入口的临界区都短（一次 `accept` 是内存里并进待发）。线程在每次取锁之前只做小事。没有无界的自旋或优先级反转，推的，没量过。线程取不到锁的代价是 `publish_when_due` 迟发，丢失窗口加排队时长，条款本来就把「排队时长」算进去：`.claude/kb/decisions/16-发布语义.md:167` 写「由此崩溃最多丢的是 T_time + P + d + 排队时长」。
- 一处读数陈旧：线程与前台都在取锁之前读钟（线程 `crates/singlefs-mount-runtime/src/lib.rs:459`，前台 `:818`、`:840`、`:862`），之后才等锁；核心层「读数倒退时不回拨」（`crates/singlefs-core/src/mounted_session.rs:6156-6162`）所以不会错，但「上次发布的读数」偏早，下一次触发偏早，方向是多发，不丢数据。

**4. 低于底额、无产出、预算花光同时成立——死局是真的，但出路不止重挂。**
- 路径：低于底额时（`foreground_cannot_write`）前台写入类发布被合取 7 拒；整理做完一条无产出的意图，`last_completed_intent` 留在会话上（`crates/singlefs-core/src/mounted_session.rs:5271`），下一次签发前判停在 `NoYieldFromTheLastIntent`（`:4977-4987`），这一停进备忘（`crates/singlefs-core/src/compaction_plan.rs:662-667`），备忘对低于底额不放行（T2 第 4 点那一格）。清掉 `last_completed_intent` 的只有：一次发成的用户改动（`crates/singlefs-core/src/mounted_session.rs:2880`）与回退（`crates/singlefs-core/src/mount.rs:10998`）。被拒的改动不清，条款字面如此：「被拒、没发成的改动不清」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`），并有用例钉着（`crates/singlefs-verification/tests/compaction_user_change_clears_the_last_completed_intent.rs:296`）。
- 已有用例就走到这个死点：`crates/singlefs-verification/tests/compaction_rollback_gate_keeps_four_user_states.rs:679` 断言前台每次覆盖写都被拒、整理最后停在 `NoYieldFromTheLastIntent`。
- 出路比「只有重挂」多：合取 7 免删除类（unlink、删快照）、管理员回退和整理批（`crates/singlefs-core/src/transaction.rs:13105-13108` 的文档串），这些发成了就清 `last_completed_intent`。底额以下用户删一个文件（磁盘满的常规动作）整理就接着签。所以死局的实质是「不删东西、只想继续写」的负载会卡到有人删东西或重挂。
- 判：这一批不在核心层修。理由：条款字面与已钉用例都冲突；窄改要动条款（实现员 r3 报告第 3 条给的改法：低于底额时无产出那一判只拦「选中的还是上一条的 R」，选得出别的段就签，`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-writer-r3/result.md` 第 3 条）；代价是选段要挪到无产出判之前，并要证「每趟无产出」的上界（段数有限，每个段至多一次，推的，没证过）。
- 这一批该做的：运行库把这个状态报出来。现在 `MaintenanceEvent::CompactionCallReturned` 带的 `last_stop` 是 `NoYieldFromTheLastIntent`（`crates/singlefs-mount-runtime/src/lib.rs:166`、`:568-571`），但运行库不知道前台正被拒；调用方（FUSE 守护进程）看到连续的 `NoYieldFromTheLastIntent` 加前台连续的合取 7 拒，才拼得出「卡住了」。建议加一个事件或观测量「低于底额而整理停在无产出」，代价是读 `foreground_writes_are_below_the_empty_cluster_segments_floor`（`crates/singlefs-core/src/compaction_plan.rs:1849`）加一个事件成员。
- 推翻：找到一个低于底额而 `last_completed_intent` 在没有发成用户改动的情况下被清掉的路径，则死局不成立；我读到的清掉的点就上面两处。

**5. 后台批写失败的冻结，重发又失败时前台见到什么——一直被拒，直到下一拍。**
- 冻结之后前台 `accept` 立刻报 `AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet`（`crates/singlefs-core/src/mounted_session.rs:6227-6234`）；`publish_pending_now` 与 `publish_user_change` 同样被拒（它们走同一条发布路径）。运行库没有前台的「重发」入口，重发只在一拍里做：`resend_the_frozen_publish_if_any`（`crates/singlefs-mount-runtime/src/lib.rs:477-501`）在拍的开头（`:509`），重发又失败交回「还冻着」，那一拍直接返回、整理不跑（`:509-511`）。
- 一拍的节奏是一个周期（默认 30 秒，`crates/singlefs-mount-runtime/src/lib.rs:51`）；周期之间线程每秒只调 `publish_when_due_once`（`:606`），它在冻结时被拒、只送一个事件，不重发。所以一次写盘报错之后，前台的写最少被拒到下一拍，最坏每 30 秒试一次。条款（D23 已定项 14 的重发）没写重发间隔；这不违反哪一条，但与「前台的写从此放行」（`:169-170`）的预期差一个周期。
- 窄改法：把重发也放进没到期的那一支（`:605-607`），每个 `publish_poll_period`（默认 1 秒）试一次。代价：冻结期间每秒多一次整条重发的写，磁盘持续报错时是持续写失败，要限次或退避；这个选择在写盘报错的性质（一次性或持续性）上，条款没定，交主 agent。
- 与 D16 已定项 17 的关系：「报错之后不许假设可以照旧复用同一份内存态重试」（`.claude/kb/decisions/16-发布语义.md` 已定项 17 一节）管调用方；运行库重发的是冻结着的同一次发布，不是调用方换内容重试，推不出冲突。

**6. 卸载与线程正在持锁相撞——卸载等得到，但等多久取决于线程在做什么。**
- `unmount`（`crates/singlefs-mount-runtime/src/lib.rs:953`）先 `stop_and_join_the_maintenance_thread`（`:941`）：置 `stopping`、拿睡眠锁通知、join。线程在批与批之间看 `stopping`（`:364` 让「在等」恒真，`:515` 的循环条件），所以最多等当前这一次调用。若这一次调用在删除走空里（第 2 点），要等整个走空做完。
- 排空待发没发成时 `unmount` 交回一个运行库，**它没有维护线程**（`:989-998` 用 `of_the_session` 重建，`maintenance_thread: None`，`:806`）。调用方若按文档「可以重发冻结的那一次或腾出空间之后再卸载」把它接着当活运行库用，GC 与周期发布都不再有；只有再 `unmount` 才有用。文档串写了线程已停（`:299`），没写后果。建议在 `UnmountRefused::PendingChangesNotPublished` 的文档里加一句「这个运行库不会自己发布与整理，只能再卸载」，或交回之前重起线程。
- `Arc::try_unwrap` 那个 `panic!`（`:965-967`）的前提是线程已 join、没有别的 `Arc` 持有者；线程闭包里的那份 `Arc` 在 join 返回时已丢。成立。

**7. panic 之后每个入口的结局。**
| 入口 | 线程在锁里 panic | 前台闭包在锁里 panic | 线程里接收方回调 panic（锁外） |
|---|---|---|---|
| `accept`、`publish_pending_now`、`publish_user_change`、`switch_background_compute`、`with_the_session_and_devices` | `Failed`（`crates/singlefs-mount-runtime/src/lib.rs:425-434`：已记坏或锁中毒都报） | 同左 | 同左：线程 panic 被 `catch_unwind` 接住并记坏（`:674-682`），之后每个入口报 `Failed`，虽然会话并没停在半个更新里（回调在锁外：`:464`、`:484`、`:528`） |
| `run_one_maintenance_pass` | `Err(RuntimeFailed)` | 同左 | 同左 |
| `observations()` | 照读（`:924-930`，中毒照用） | 同左 | 同左 |
| `unmount` | `UnmountRefused::Failed`，盘交回（`:974-981`） | 同左 | 同左 |
| `Drop` | 停线程、不写盘（`:1021-1028`） | 同左 | 同左 |
- 两个观察：接收方回调的 panic 会把整个运行库判坏，尽管会话是好的（保守，回避不了，因为回调在线程上）；线程因运行库已坏而退出时 `Ok(Err(_))` 被吞（`:678`），调用方只在下一次入口报 `Failed` 时才知道线程没了，没有事件。

**8. Drop 只停线程，「丢最多一个 T_time + P」。**
- `Drop`（`crates/singlefs-mount-runtime/src/lib.rs:1021-1028`）只停线程、不排空、不写盘，与注释一致（`:11`、`:1022`）。丢失窗口不是「T_time + P」：条款自己写的是 `T_time + P + d + 排队时长 + N_switch × (T_retry + T_redo)`（`.claude/kb/decisions/16-发布语义.md:167`）。运行库把后台一批持锁算进「排队时长」，这一项现在有了数量级：一批整理加 GPU 等待。正文里写的「丢最多一个 T_time + P」若出现在运行库的用户文档里，应改成引条款这一行。

## T4　水位与默认值

**1. E 线 5% 与「线夹到整段数」——成立；极小盘上线等于整段数，大盘上就是 5%。**
- 代码：`crates/singlefs-core/src/compaction_plan.rs:1589-1606`，线 = min(max(⌈N × 5 ÷ 100⌉, 2), N)，N 是这块盘单元区的整段数（单元区槽数 ÷ 64，`:1669`）。越线判 `empty_cluster_segments < line`（`:1736`）。默认值在 `:1640-1647`，百分比与下限常量在 `:1571-1573`；条款：「批大小默认 16」所在段写的是整段数 5%、下限 2（`.claude/kb/decisions/26-后台整理与放置回收.md:32`）。
- 我算的几个点（推的，纯算术）：N=1→1；N=2→2（一段用掉就越线）；N=3 到 40→2；N=41→3；N=100→5；N=4096→205；N=10⁶→50000。N=0 时线为 0，永不越线。
- 极小盘上的行为：N=2 的盘只要有一段被占就恒越线，整理每个周期都起跑，随即停在「没有合格段」或「空洞不够」，这两种进备忘（`crates/singlefs-core/src/compaction_plan.rs:662-665`），到下一次发布前不重判，所以不空转；代价是每次发布之后多一次起跑判与一次选段扫描。推的，没量过。
- 全空段数口径是「记录口径」，一段里只剩 defer 槽也算全空（`crates/singlefs-core/src/allocator.rs:900-908`）；开段用的是另一个量（`lowest_empty_segment`，C379）。后果：水位读到的空段可能比真开得出的多，整理比前台真正缺段的时候起得晚。这是 C372 已收口的口径，不是这一批新增。

**2. F、U 关着时是不是真不读——决策上不读，读数结构体照填。**
- 两条关着的分支直接略过：碎片度 `FragmentationWatermark::Off => {}`（`crates/singlefs-core/src/compaction_plan.rs:1744-1745`），不可回收字节 `UnreclaimableBytesWatermark::Off => None`（`:1769-1770`）。`WatermarkReadingOfADevice::of_the_device_map`（`:1665-1674`）每次把五个字段都填一遍（都是 O(1) 取字段，`crates/singlefs-core/src/allocator.rs:623-625` 的 `allocated_slots`、`:899-901` 的 `free_runs` 等），不进判定。不落盘、不读盘：`RuntimeParameters` 没有存盘路径，水位只装在内存里的分配器上（`crates/singlefs-core/src/mount.rs:9617`）。
- 不存盘与盘上 24 字节：见下面第 5 点。

**3. E 线高于底额 N₀ 的关系——除 N=1 外成立。**
- 底额 N₀=1（`crates/singlefs-core/src/compaction_plan.rs:497`），线至少 2（`:1573` 的文档串写「线至少 `N₀ + 1`」），所以 N≥2 时整理总在前台被拒之前起跑。N=1 时线夹成 1，等于 N₀：全空段数降到 0 的那一刻前台与整理同时「该动了」。只有一个整段的盘整理也无处可搬，实际无影响，推的，没量过。

**4. K=16 在 S=5、S=8 上一份预算做不做得完一条意图（门留 8 槽之后）。**
| 量 | S=5 | S=8 |
|---|---|---|
| 一份预算 `3S − 4`（`crates/singlefs-core/src/compaction_plan.rs:1532-1540`） | 11 | 20 |
| 挂载已过发布至多 8（写行加暖机 3，续做 5，`:1524-1527`）之后还剩 | 3 | 12 |
| 一条 64 单元的意图至多发布（`:430-431`） | 5 | 5 |
| 门放的连续整理发布（模型，环里每根都是新状态） | 3 | 12 |
| 做得完一条意图 | 否（预算和门都不够） | 是 |
- S=8 一格有实现员量的用例佐证：`crates/singlefs-verification/tests/compaction_rollback_gate_keeps_four_user_states.rs:405` 的 `under_the_default_batch_size_one_intent_finishes_within_what_the_gate_lets_through`（默认几何，量的是实现员报的绿，任务 10976）。S=5 没有用例，表里的数是推的，没量过；模型输出见 `out/model.out`。
- 推翻：造一个 S=5 的池（显式给 S，或槽距大的盘 mkfs），在同样的历史上跑门的用例，量到一条意图在一次用户改动之后走完，则 S=5 一格不成立。

**5. 盘上 24 字节改名之后写的字节有没有变——没变。**
- 改名前后各读一遍：现在 `RESERVED_RUNTIME_WORD_COUNT = 3`（`crates/singlefs-core/src/system_configuration.rs:103`）、`RESERVED_RUNTIME_WORD_VALUE = 0`（`:105`），写盘循环 `for reserved_word in self.runtime.reserved_runtime_words()`（`:682`）；改前是 `COMPACTION_WATERMARK_COUNT = 3`、`COMPACTION_WATERMARK_BUILT_IN_DEFAULT = 0`、`compaction_watermarks()` 返回三个 0（量的：`git show HEAD:crates/singlefs-core/src/system_configuration.rs` 的 `:102`、`:104`、`:369-370`）。三个 `put_u64(0)`，字节相同。
- 独立读者一侧同值：`crates/singlefs-verification/src/mount_transparency.rs:1381` 的 `[0, 0, 0]`，改前同为三个 0（量的：`git show HEAD:crates/singlefs-verification/src/mount_transparency.rs` 的 `:1381`），36 字节断言不变（`crates/singlefs-core/src/system_configuration.rs:1152`）。字段表在 `.claude/kb/layout/01-first-txn.md:153` 已写「整理水位预留 | 8 × 3 | 全 0」。成立。
- 没跑 `mount_transparency` 的测试；这一格是读两边的代码得出的。

## T5　开卡器与第 102 批

**1. 枚举工作线程超时之后实例的状态——有一条会卡住调用线程的路径。**
- 开卡放工作线程，调用线程 `recv_timeout`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:225`），到点返回 `TimedOut`（`:227-229`），工作线程留着，`an_opening_is_hung` 保持真，直到它自己回来才放（`:215`）。
- 取实例那一步在工作线程上，**持着进程级实例的互斥锁**调用 `make_an_instance()`（`:163-175`：锁在 `:163-165`，建实例在 `:174`，锁到函数尾才放）。驱动若挂在「建实例装 Vulkan 加载器」（`:158` 的文档串自己承认「驱动也可能挂在这里」），这把锁永远不放，之后调用线程上的 `instances_created()`（`:137-142`）与 `forget_the_instance_so_the_next_opening_creates_one`（`:151-156`）都会阻塞在同一把锁上，调用线程挂死。开卡本身不受影响：下一次开卡在取锁之前就被 `an_opening_is_hung` 拦下返回（`:190-192`）。
- 挂在枚举（`open_on_the_worker_thread`，`:209`）则不同：锁在 `:177` 前已放，实例留着（`held.instance` 是 `Some`），工作线程回来之后下一次开卡复用这个实例。「那个实例在一次挂住之后还能不能用」模块说明承认读不到（`:8`），我也核不了：造不出真挂住的驱动。
- 窄改法：建实例放在锁外（先看有没有、没有就放锁建、再取锁存），或者给 `instances_created`/`forget` 用 `try_lock`。代价：建实例的并发要自己防（现在 `one_opening_at_a_time` 已经保证同一时刻只有一次开卡，`:186-189`，所以只需把计数加一与存实例放在建成之后）。
- 推翻：用一个会睡很久的 `make_an_instance` 替身，工作线程超时后调用线程调 `instances_created()`，不阻塞，则这一条不成立。现有用例 `crates/singlefs-background-gpu/tests/gpu_opener_of_this_process_gates_a_hung_opening_and_limits_the_instances.rs` 的挂住替身若挂在开卡那一步（不是建实例那一步）测不到。

**2. 闸住新开卡之后能不能恢复——只有工作线程回来这一条路。**
- 闸在 `:215` 放（起线程失败 `:220`、工作线程 panic `:231` 也放）；`forget_the_instance_...`（`:151-156`）只清实例，不清闸。工作线程永不回来则每次挂载选 `gpu` 都拒 `AnEarlierOpeningIsStillHung`（`:190-192`），出路是重启进程，错误成员的文档串（`crates/singlefs-mount-runtime/src/lib.rs:131-132`）没写这个出路。选 `cpu` 的挂载不碰开卡器（`:753`），不受影响。成立，但出路只在进程重启。

**3. 实例上限 4 够不够。**
- `VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST = 4`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:30`），只在「没有留着的实例」时新建并计数（`:166-175`）。同一个进程挂载多次都复用同一个，所以只有 `forget` 才多建：4 次里第一次是首次，修好驱动重建至多 3 次。守护进程挂一次的场景够；反复修驱动的长进程不够，到了拒成 `InstanceLimitOfThisProcessReached`（`:169-173`）。数是拍的（`:29` 文档串），第二台上量到约 12，本腿不核。

**4. 修好驱动重挂的路径（判决打中 6）——有入口，运行库不调，全靠调用方。**
- 入口是 `forget_the_instance_so_the_next_opening_creates_one`（`:151`）。全仓（`crates/` 下 `.rs`）调它的只有测试（`gpu_opener_of_this_process_gates_a_hung_opening_and_limits_the_instances.rs:121`，量的：`grep -rn`），运行库 `MountedRuntime::mount` 的流程里没有，也没有「开卡失败就自动 forget 一次再试」。修好驱动重挂要调用方知道这个入口，文档在模块头（`:10-11`）。成立但全靠调用方。
- 重建不还计数（`:150` 的文档串写明）。

**5. 上限 0 在枚举之前判（第 102 批）与核心层校验次序的差别——成立，空列表加上限 0 在两处拒成不同成员。**
- 产品开卡先判上限 0（`crates/singlefs-background-gpu/src/background_gpu.rs:849-853`，在 `:858` 取实例与枚举之前），拒成 `SettingsRefused(GpuWaitLimitOfZero)`。核心层的校验在 `crates/singlefs-core/src/background_compute.rs:985-1030`：先判 GPU 在不在（`:990`、`:998`），再判点名的卡（`participating_gpu_devices`：空列表拒成 `EmptyGpuDeviceListWhoseMeaningIsUndecided`、重复点名、不在清单里），再判显存上限个数与总量，最后才判上限 0（`:1023`）。
- 同时错的组合在两边拒成不同成员：空列表加上限 0（产品：`GpuWaitLimitOfZero`；核心层：`EmptyGpuDeviceListWhoseMeaningIsUndecided`）；重复点名加上限 0；显存上限个数不符加上限 0；没有 GPU 加上限 0。出路都是改参数，改完一个再被拒另一个，没有数据后果。
- 易误读的注释：`crates/singlefs-core/src/background_compute.rs:973` 写「最后那一判排在最后，同产品 GPU 包挂载那一刻的次序」，指「点名的卡开不了」那一判，不是上限 0；但整段文档串列的次序（`:972-973`）读起来像产品包照这个次序，而产品包现在把上限 0 提到了最前。建议在 `:973` 补一句「产品 GPU 包把上限 0 提到枚举之前判」。
- 运行库把开卡器的所有拒绝压成一个字符串：`RefusedByTheOpening { description: String }`（`crates/singlefs-mount-runtime/src/lib.rs:136`），实现处 `format!("{refused:?}")`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:110`）。设置配错（改参数）、点名的卡不在（改参数）、枚举等超过上限（换环境）、卡没开成（等或换卡）四种调用方要做的决定不同的拒绝并成一个成员，与同一文件 `:126` 的文档串「成员按调用方要做的决定分」不一致。调用方（FUSE 守护进程）只能解析字符串。建议至少把 `SettingsRefused` 与 `NamedGpuDeviceNotOnTheListOfThisEnumeration` 拆成一个「改参数」成员。

**6. `the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one` 这条钉法——钉住了一半，另一半有别的用例，第三处没钉。**
- 它钉的是 `VulkanInstanceHandedToTheOpening for VulkanInstanceOfThisProcess` 交回的是 `self.clone()`（`crates/singlefs-background-gpu/src/background_gpu.rs:809-811`）：两个实例相等靠 wgpu 内部那份 `Arc` 的地址（用例 `:4783-4796`；`wgpu-30.0.1/src/api/instance.rs` 用 `impl_eq_ord_hash_proxy!(Instance => .inner)`，量的：读了仓外 cargo 注册表里 wgpu-30.0.1 源码的 api/instance.rs 第 38 行），对 wgpu 版本敏感。变异「把 `self.clone()` 换成 `Self::without_any_backend()`」会红，成立。
- 开卡入口真用交进来的那一份，靠另一个替身用例（`:4626-4631` 的计数替身，调 `:858` 取实例）钉。两半合起来钉住 `background_gpu.rs`。
- 没钉住的是开卡器这一层：`GpuOpenerOfThisProcess::over_the_vulkan_instance` 的 `move || vulkan_instance.clone()`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:88`）与 `of_this_process` 的 `created_for_this_process`（`:75`）。开卡器单测用泛型替身实例，不碰真类型；把 `:88` 换成另建实例，只有要卡的真卡用例（`mount_runtime_on_a_real_gpu_...`，特性 `tests-on-a-real-gpu`）才可能红，而它不比实例是不是同一个。建议补一条：开卡器用 `without_any_backend` 实例连开两次，断言两次交给开卡函数的是同一个（`==`），变异行换 `:88`。

## 检查单八格

**走不到的理由二次查实——不成立。** 实现员写「无：……没有新加 `todo!`、`unreachable!`、`assert!`」，字面对三个宏，但漏了 `expect` 与 `panic!`。我从改动全文的新增行里数出产品代码（非 `tests/`）8 处（量的：对 `research/prompts/_m4-gpugc-mount-runtime-code-r1-diff.md` 新增行 `grep` 的 `panic!`、`.expect(`、`unreachable!`、`todo!`、`assert!`、`.unwrap()`；后三样 0 处）：
| 位置 | 理由站不站得住 |
|---|---|
| `crates/singlefs-core/src/compaction_plan.rs:434` `panic!("16 不是 0")` | 站住：常量上下文，`NonZeroUsize::new(16)` 编译期求值 |
| `crates/singlefs-core/src/compaction_plan.rs:1603` `expect("线不超过整段数")` | 站住：前面 `.min(整段数)`，整段数是 `u64` |
| `crates/singlefs-core/src/compaction_plan.rs:1764` `expect("低于空闲段数读数，装得进 u64")` | 站住：只在 `is_above_the_line` 为真的分支里取，此时线 ≤ 空闲段数读数（`:1754-1765`）；关着的默认下不进这支 |
| `crates/singlefs-core/src/mount.rs:4300`、`:4304`、`:4306` 三处 `expect` | 站住：个数不超过根环 3S ≤ 48，`usize` 到 `u64` 在 64 位目标上不会失败 |
| `crates/singlefs-mount-runtime/src/lib.rs:562` `expect("一次调用的发布数")` | 站住：`Vec::len` 到 `u64`，同上 |
| `crates/singlefs-mount-runtime/src/lib.rs:700` `panic!("维护线程没起来：运行库是这一份的唯一持有者")` | 依赖标准库在 `spawn` 失败时丢掉闭包（闭包里那份 `Arc`）。我没读标准库源码，推的，没量过；不站住时是挂载失败路径上的 panic，而这条路径本来要把盘交回 |
| `crates/singlefs-mount-runtime/src/lib.rs:966` `panic!("线程 join 过了、前台入口都借着 self……")` | 站住：`unmount(mut self)` 要所有权，线程闭包即使 panic 也在 `catch_unwind` 里正常返回（`:674`）；接收方回调在 `:682` 再 panic 时闭包展开，`Arc` 随之丢 |
- 其中 `:700` 与 `:966` 是运行库的两个 `panic!`，条款没定错误成员，也没写成「走不到」注释；应补上「走不到的理由」登记或改成返回错误。
- 核心层与开卡器有没有写成注释的「走不到」：在改动新增行与 `crates/singlefs-mount-runtime/src/lib.rs`、`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` 全文里搜「走不到」「不会发生」「不可能」「不会走到」「永远不」都 0 处（量的：`grep -n` 与对改动全文的同样搜索）。
- 推翻：在新增行里找到一处我没数到的 `expect`、`panic!` 或「走不到」注释。

**类型与名字——不成立（一处）、其余成立。**
- 不成立：`WatermarkVerdict::Crossed { reading: u64, line: u64, .. }`（`crates/singlefs-core/src/compaction_plan.rs:1704-1709`）这对裸整数按 `quantity` 在三种单位间换：全空聚簇段数是段（`:1737-1741`），碎片度是空闲段数与按已分配槽数换算的线（`:1761-1765`，文档串 `:1703` 写明），不可回收字节是字节（`:1772-1775`）。同一对字段三种语义，靠相邻的 `quantity` 才读得懂。建议拆成每种一个成员或带单位的新类型。
- 成立：`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH`（`:1527`）与 `ROLLBACK_STATES_KEPT_BY_THE_COMPACTION`（`crates/singlefs-core/src/mounted_session.rs:4118`）各只有一种语义（一个数槽/次发布，一个数状态）；`CompactionBatchSize::OfTheMount` 与 `ForcedForTests` 都带 `CompactionBatchUnits`（非零新类型，`:432-444`），`match` 里写全成员（`crates/singlefs-core/src/mounted_session.rs:4203-4205`）；`NoYieldFromTheLastIntent`、`CompactionPublishWouldLeaveFewerThanFourRollbackStates` 的字段名自带单位（`:602-605`、`:644-647`）；`run_one_maintenance_pass`、`gpu_opener_of_this_process` 说得出内容。新增代码里 `_ =>` 0 处（量的：改动新增行与两份新文件 `grep -nE '^\s*_ =>'`），空 `{}` 臂的几处都写全了成员。
- 限定：私有结构体 `Shared`（`crates/singlefs-mount-runtime/src/lib.rs:337`）与字段 `locked`（`:340`）名字很短，可能被命名检查的 `vague-names` 点名，交主 agent 跑命名检查那一条（本腿不跑）。`ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH` 的值是「再发几次发布」，名字叫槽；每次发布占一个槽（`crates/singlefs-core/src/mount.rs:4257-4264`），读得通。
- 推翻：命名检查的 `vague-names` 与 `ordinal-names` 两格对新文件判红。

**不兼容旧代码与旧格式——成立。**
- `NotDecidedByTheClauses`、`BatchSizeNotDecidedByTheClauses`、`COMPACTION_WATERMARK_COUNT`、`compaction_watermarks()` 在冻结副本 `crates/` 下 `.rs` 里 0 处（量的：`grep -rn` 四个名字，0 命中）；盘上 24 字节字节不变（T4 第 5 点）。新增代码里没有认旧开关或旧名字的分支（`grep` 「legacy」「兼容」「迁移」在新增行 0 处）。
- 限定：`.claude/kb/decisions/26-后台整理与放置回收.md:32` 还写着「代码还没改……`CompactionBatchSize::NotDecidedByTheClauses`」，`:196` 还写着 `BatchSizeNotDecidedByTheClauses` 与 `COMPACTION_WATERMARK_COUNT`；kb 与代码此刻不一致，是 kb 回写还没做，不是代码里有旧东西。

**不新建判定库——成立。** 补丁的文件清单（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-merged/patch/apply-outcome.tsv`）里全是 `.rs`、`Cargo.toml`、`Cargo.lock`；改动全文新增行里搜 `rocksdb`、`sqlite`、`.tsv`、`File::create`、`fs::write`、`OpenOptions`、`std::fs` 0 处（量的）。运行库只有内存里的观测（`RuntimeObservations`，`crates/singlefs-mount-runtime/src/lib.rs:191-205`，文档串写明「只住内存」）。

**所有任务多机可用——成立。** 新增代码与新测试里没有写死的 PCI 地址、主机名、`/home`、`std::env`（量的：对运行库、开卡器与两份新测试的 `grep`）。运行库的钟是 `Arc<dyn ClockOfTheRuntime>`（`crates/singlefs-mount-runtime/src/lib.rs:101`），测试摆假钟（`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:54-60`），负载是内存盘。真卡用例读服务交的卡（`common_discrete_gpus_of_this_host` 读环境变量与多机配置，该助手不是这一批新写的），没交卡时打 `NOT_RUN_NO_GPU_ADAPTER` 退成功（用例文件头）。限定：起真线程的几条（`:712`、`:779`、`:810`）用真钟、`THREAD_DEADLINE` 5 分钟（`:43`）和忙等 `yield_now`，慢机上不会错判但会拖时间。

**测试优先多线程——运行库用例字面不成立，新崩溃流成立。**
- 新崩溃流 `crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs` 自己不切片、不选线程数：它调共用枚举器 `enumerate_crash_states_with_every_kind_of_observer`（`:170`；邻居流 `crash_enumeration_deletion_interleaved_with_compaction_across_a_rollback_stream.rs` 同调），线程数取 `SINGLEFS_THREAD_CAP` 再取 `.claude/heavy-thread-default` 的登记值（`crates/singlefs-verification/src/crash_state_enumeration.rs:86`、`:261`），并行在共用枚举器里（`:655` 的 `std::thread::scope`），文件头写了「带断点续跑、可双机分片」。成立。
- 运行库四条起真线程的用例（`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:712`、`:779`、`:810`，另 `:843` 在锁里 panic 不起线程）：没有一句写它们为什么不并行、为什么只用一条后台线程（量的：对该文件 `grep '串行\|并行\|不并'` 0 命中）。字面不成立；理由本身站得住（被测对象就是前台与一条维护线程的次序，并行跑多个池只是多几份内存池），补一句即可。

**带 `--item` 的命令与项——成立（读登记，没跑）。**
- `.claude/gate.d/crash-point-replay-items.tsv:77` 的行首是 `compaction-with-a-write-while-the-intent-is-live`，指到 `crash-case:compaction-with-a-write-while-the-intent-is-live-stream`；`.claude/gate.d/stage-inputs.tsv:158` 有同名键，指到 `crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream` 的 `full_enumeration_…`（量的：副本里 `grep -nF`，两行都是各自文件的最后一行）。`compaction-relocation` 在 `.claude/gate.d/crash-point-replay-items.tsv:36`。两项的登记都在，`--list-items` 是否列出它们要跑命令才知道：写在下一节。

**新崩溃流多罩了什么——成立。**
- 流里真有那一步：用例 `the_overwrite_of_the_flow_sits_between_two_batches_and_releases_placements_in_the_region`（`crates/singlefs-verification/tests/crash_enumeration_compaction_with_a_write_while_the_intent_is_live_stream.rs:78-137`）断言覆盖写步夹在一批与下一批之间（`:82-97`），覆盖写那一版 R 内未释放的记录比上一批少（`:99-118`）且意图还活着（`:119-128`）。与邻居流比：邻居流上活意图期间只有 unlink（文件头 `:8` 自己写的），这一条第一次枚举「写放掉 R 里落点之后下一批重扫」的状态。
- 限定：流里的覆盖写是单发的 `publish_user_change`；运行库里前台的写先进待发、由后台调用开头的排空或 `publish_when_due` 并成一次 `DataChangesPublishedTogether`（`crates/singlefs-core/src/mounted_session.rs:6651`、`:6189-6192`）。两条路径共用同一个事务层，崩溃点的形状相近，但排空那一次的 journal 记录数不同，流没覆盖「并成一次」那一种。推的，没量过。

## 要主 agent 跑的命令

1. 复跑自写模型与锚点（不起 cargo，只起 python3 与 grep；工作目录任意；跑完看 `name=anchors_match rc=0`、`name=model_match rc=0` 与 `name=model_rows_not_finishing_an_intent count=14`，退出码 0 算过）：
   `task-server/task submit -- bash research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-sonnet/rerun-1`
   起在仓根。同一条命令被服务重排时 `--work` 已非空会退 2：重排换 `rerun-2`。
2. 核两项被 `--list-items` 列出（读登记，不起重放；跑完看输出里有没有 `compaction-with-a-write-while-the-intent-is-live` 与 `compaction-relocation` 两行）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=crash-point-replay.sh --list-items; 范围=只列可点的项，不重放; 为什么=核新崩溃流项登记后被列出' task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items`
   起在仓根。
3. 本腿没有别的要量的命令。第 T2、T4 的 S=5 一格、T3 第 2、4、5 点和 T5 第 1 点都需要新写用例才量得到，已在各点「推翻」里写了用例怎么写；写用例归实现员，不在本腿范围。
4. 引文核对（读文件，主 agent 或我直接跑）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1/repo --background research/prompts/_m4-gpugc-mount-runtime-code-r1-background.md`。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 接上了吗 | 部分成立 | 线程自己跑、前台入口都经锁、线程体与同步一拍同代码；「腾出能开的空段」只在同步一拍的用例里证过，线程用例只等到一次整理发布 |
| T1 真卡外推 | 不成立 | 真卡用例是不起线程的假钟一拍一拍推，没有线程加 GPU 持锁加前台并发的用例 |
| T1 E202 与七·4 | 不走运行库 | E202 走核心层入口加强制起跑，绕门、备忘、破例、预算与放锁；量的是整趟持锁的数 |
| T2 状态口径 | 与条款一致 | 门数指针状态，搬根算一个状态（用户 2026-10-13 定），与「4 个用户内容版本」不是同一件事 |
| T2 8 槽余量 | S≤5 不成立 | 3S−9 的窗口随 S 缩：S=5 门放 3 次、S=4 永关，一条意图至多 5 次；mkfs 默认 S 在 5 到 8 |
| T2 破例先后 | 成立 | 破例在门之前、只在新一趟第一次发布；接着推的一趟预算为 0 不开破例 |
| T2 备忘组合 | 一格会死局 | 低于底额时备忘只放行「门关」一种，「无产出」仍挡 |
| T2 挂载预算 | 成立（限定 S=5） | 写行与暖机算进已过发布；S=5 时挂载之后只剩 3 次 |
| T2 强制起跑 | 部分是产品路径 | 门的用例与运行库用例走产品开关；8 份验证包测试加 E202 钉的是搬迁与节奏 |
| T3 fsync 时序 | 成立 | 多等一批加两次取锁，不丢；无真线程用例 |
| T3 删除走空 | 不成立 | 整段持锁、不看 `stopping`、不放锁，队列无上界；建议这一批修 |
| T3 周期线程取锁 | 推不出饿死 | 无证据，读数陈旧方向安全 |
| T3 死局 | 真，出路不止重挂 | 发成的用户改动（含删除类）清 `last_completed_intent`；这一批不动核心层，让运行库把它报出来 |
| T3 冻结重发 | 成立（延迟一个周期） | 前台一直被拒到下一拍，默认 30 秒；窄改法写在 T3 第 5 点 |
| T3 卸载相撞 | 成立 | 等当前一次调用；卸载被拒后交回的运行库没有维护线程，文档没写后果 |
| T3 panic 与 Drop | 成立 | 每个入口报 `Failed`；丢失窗口应引条款而不是「T_time + P」 |
| T4 水位与默认值 | 成立 | 线 = min(max(⌈5%⌉, 2), 整段数)；F、U 关；24 字节不变；S=5 上一份预算做不完 |
| T5 开卡器 | 部分成立 | 建实例持锁时挂住会卡调用线程；闸只能等工作线程回来；运行库把拒绝压成字符串 |
| T5 上限 0 次序 | 成立（有差别） | 空列表加上限 0 在产品拒成上限 0、在核心层拒成空列表 |
| T5 钉法 | 钉住 `background_gpu.rs` 一半加替身一半 | 开卡器 `:88` 的 `clone` 与 `:75` 的建实例没钉住 |
| 走不到的理由 | 不成立 | 产品代码新加 8 处 `expect`/`panic!`，实现员写「无」 |
| 类型与名字 | 不成立（一处） | `Crossed { reading, line }` 三种单位共用一对 `u64` |
| 不兼容旧代码 | 成立 | 旧名字 0 命中；kb 两处还写旧名字 |
| 不新建判定库 | 成立 | 补丁清单只有 `.rs` 与配置 |
| 多机可用 | 成立 | 假钟、内存盘、服务交卡；真线程用例用真钟 |
| 测试优先多线程 | 新流成立，运行库用例字面不成立 | 新流用共用枚举器；四条真线程用例没写不并行的理由 |
| `--item` | 成立 | 登记行在末行；`--list-items` 待主 agent 跑 |
| 新崩溃流 | 成立 | 用例断言覆盖写放掉 R 内落点；没覆盖「并成一次」的排空发布 |

## 没做什么

- 没起 cargo、没交任务、没跑门禁阶段与命名检查；没跑模型的 `rerun.sh`（交主 agent）。模型 `gate_geometry.py` 与锚点的期望输出是我在草稿目录里直接跑 `python3 -I` 与 `grep` 得到的（量的，只读命令加我自己的纯算术脚本），已落进 `out/`。
- 没读标准库源码（`spawn` 失败时是否丢闭包）、没读 `common_discrete_gpus_of_this_host` 助手的全部、没读 `adapter_inventory.rs` 的改动（它在文件清单里，但改动全文里没有它的块）。
- 没量：任何挂钟、等待分布、真线程下的争用；S=5 的门与预算只在模型与读码上推。没造真挂住的驱动。
- 没判：要不要采纳我给的窄改法（T3 第 2、4、5 点，T5 第 1、5 点）；没替主 agent 出判决。
- 没核：合入后验证与活意图期间写类改动那条崩溃流的结局（任务 11000、11011，背景材料写了还没有结局）。
- 模型目录：`research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-model/`，文件的 sha256 在交回正文里。
