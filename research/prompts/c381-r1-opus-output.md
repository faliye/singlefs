# c381-r1 云端攻方腿（Opus）报告：K2 盖写 / K4 攻方原历史

立场：攻方，假设主 agent 的倾向（乙「按段分」）是错的；只攻 K2、K4，不判 K1、K3、K5、K6。时刻 2026-09-19 约 05:50–11:50 UTC（中间撞过一次限额，10:00 UTC 续做；续做时阶段 A、D、E 已跑完，阶段 B、C 重跑并加了进度行）。
⚠️ 下面的数都是**攻方副本上的数**，不进 kb；要引，先在入库装置上重做（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）。

## 零、复跑命令与文件指纹

副本：`rsync -a --exclude target --exclude .git` 从主工作区拷整仓，再把 harness 换回 `HEAD:` 那一版（主工作区里另一实现员没提交的 `crates/mutations.tsv`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/lib.rs`、`crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` 换成 `git show HEAD:路径`；未跟踪的 `crates/singlefs-harness/src/model.rs`、`model_comparison.rs` 删掉）。core 三个文件拷下来时与 `HEAD:` 同 hash（`transaction.rs` d7de039c…、`mount.rs` a23acac6…、`allocator.rs` 6544913c…）。

```
M=research/prompts/c381-r1-opus-model
# 臂副本 <R>：core 与 history.rs 打补丁（在 HEAD 版上 patch -p1，回放核过与跑数那一份逐字节相同）
cd <R> && patch -p1 < $M/c381-arms-core.diff && patch -p1 < $M/c381-arms-harness.diff
cp $M/c381_probe.rs <R>/crates/singlefs-harness/tests/c381_probe_common.rs
cp $M/c381_k4.rs $M/c381_k2.rs $M/c381_k2b.rs $M/c381_k2c.rs $M/c381_k2d.rs $M/c381_single.rs <R>/crates/singlefs-harness/tests/
nice -n 19 cargo test --release -p singlefs-harness --test c381_k4 -- --nocapture --test-threads=1          # k4.log
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2 -- phase_a --nocapture --test-threads=1  # phase-a.log（24 线程，约 77 分钟）
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2 -- phase_b --nocapture --test-threads=1  # phase-b.log（约 57 分钟）
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2 -- phase_c --nocapture --test-threads=1  # phase-c.log
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2b -- phase_d --nocapture --test-threads=1 # phase-d.log
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2b -- phase_e --nocapture --test-threads=1 # phase-e.log（约 73 分钟）
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2c -- --nocapture                          # b-prime.log
nice -n 19 cargo test --release -p singlefs-harness --test c381_k2d -- --nocapture                          # no-remount.log（单线程，约 30 分钟）
for k in 16 17 18; do C381_K=$k C381_ACTIONS=Retry nice -n 19 cargo test --release -p singlefs-harness --test c381_single -- --nocapture; done  # s2-retry.log（grep -A5 '^\[' 过滤）
# HEAD 版 core 副本 <P>：core 不打补丁，探针用去掉臂开关的那一份
cp $M/c381_probe_pristine.rs <P>/crates/singlefs-harness/tests/c381_probe_common.rs; cp $M/c381_k4.rs $M/c381_single.rs <P>/crates/singlefs-harness/tests/
cd <P> && nice -n 19 cargo test --release -p singlefs-harness --test c381_k4 -- --nocapture --test-threads=1   # k4-pristine.log（只留甲那几行：grep -A4 '^\[甲\]'）
for c in 17 18; do C381_CRASH=$c C381_ACTIONS= nice -n 19 cargo test --release -p singlefs-harness --test c381_single -- --nocapture; done  # pristine-crash.log（grep -A5 '^\['）
```

跑的先后：`k4.log`、`phase-a.log`、`phase-d.log`、`phase-e.log` 是在 core 加「乙′」（臂 4）那几行之前编的；加的那几行只新增臂 4、把「根槽那一步」从 S2 里拆出一个内部下标再映射回 S2，臂 0–3 的行为不变。`phase-a.log` 那一版探针还没有 `LYING` 开关与进度行（`LYING` 默认关，同行为）。编译与跑动时 `ps` 没看到性能测量进程（`qemu-system`、`vm-bench.sh`、`e152-*`、`fio`）；开工时另有一个会话在主工作区跑 `cargo test --all`，我的都在副本自己的 target 目录里，没等锁。

| 文件（`research/prompts/c381-r1-opus-model/`） | sha256 |
|---|---|
| `b-prime.log` | `7a49162f727b4a32faafc97f184ed3c3767f73b8195407c520febfdfd48bfad8` |
| `c381-arms-core.diff` | `12434f928caab49f69de19215a3445c8e983d25849fc56a4c18e4380273ee9d2` |
| `c381-arms-harness.diff` | `c33253e93ef8e94a10b2bac6515d5cc32e8697b233199bb0fc3ec543ffb8c65a` |
| `c381_k2b.rs` | `662f9ef0c0734b233146fc37eddf8bbdf0b2ce17f4119da915502a1b9b09ced5` |
| `c381_k2c.rs` | `5e97f11c15c15c962521a843a7da1601d58350f5cf70d95ace3d8a77a58c479a` |
| `c381_k2d.rs` | `c5fa1148dc640d46eb7b80ddb39af411a78e5c1ceefb982ca295faeeccccd816` |
| `c381_k2.rs` | `5caa1574410683bf6738ca7b8e9f1dd08168ca2031070c1b9e5bb76322c26280` |
| `c381_k4.rs` | `f49b65bfd3a82e56807d573aa96deac11df110c6e2ae911013f501a9e7623135` |
| `c381_probe_pristine.rs` | `269bce43c692f049ccd477145dc8797f3ca591200e2749f0695321d8b665f58f` |
| `c381_probe.rs` | `21b20b2b5c68ffd8926596f8a3b0ea8c6d5b628546363f3c7e9af474f45d49f3` |
| `c381_single.rs` | `8f04b7f83bae76d277e2a1e5492c06d15e688c8dc50dba21b6cbfdade285aadb` |
| `k4.log` | `5d19a1accd63a3b8a73e01454838635ea2ae9c882205649ff79d2e2485760f0d` |
| `k4-pristine.log` | `dc9dee57afbb0c8b1f9ee9a34eb9bc7df68700a6c638836af69093c7dac09333` |
| `no-remount.log` | `645c6929646fd3bcec2f84096ecc31fb94717b16d4380e92441508f27ac73efa` |
| `phase-a.log` | `1a73444d14de348f92e63b019cec09a2f7603410feb7c96d6abc420c9248eea1` |
| `phase-b.log` | `9df5c3a06e2df5d6318e261d1d7c150ca53726a12d0a08f88660f3f8e55cc7dc` |
| `phase-c.log` | `ea7183492babad22931850158c7e627562ea4542c0742d606e01bf2a6ad850f7` |
| `phase-d.log` | `513efd440a731111d83ed08388c1aedce97df4569df0e9feb02e238eb4956bae` |
| `phase-e.log` | `5eb11c605d4795b0727c083f1cd69bfcd9155f9ede448778596c1999101610ad` |
| `pristine-crash.log` | `284bf929321050b8992901f345feaa3ff32520f4ffffa14589b21233fd1bab0b` |
| `s2-retry.log` | `15c598b3a7b82c5a02643b95fccea4e32244499e2348aad024b72f3ffb971bc8` |


## 一、各格判定一览

⚠️ 下面每个数都是**攻方仓副本上的数**（`/tmp/claude-1000/c381-r1-opus/repo`，core 打了 `c381-arms-core.diff`），不进 kb；要引先在入库装置上重做。甲那几格另在 HEAD 版 core（未改，`pristine/`）上复跑过同一段历史，逐字相同（第三节、第四节写明哪几格）。

分类口径（跑前写死在 `c381_k2.rs` 的 `classify`）：**盖写** = 重开报 `UnitUnreadable`，或断电那一刻 / 重开之后 checker 的 I-2.1、I-4.8、I-7.2、I-7.4 任一红；**别的红** = 只有别的不变量红（实际只出现过 I-3.1）；**干净** = 重开成功且两次 checker 全绿。

| 格 | 甲「一律退回」 | 乙「按段分」（倾向） | 丙「失败即停」 | 丁「扣住」 |
|---|---|---|---|---|
| K4 原历史（C 在盘 1 超级块槽失败 → 从 B 发空发布 E → E 写完单元、在记录那一步断电 → 重开） | **中**：重开 `REFUSED Recovery(UnitUnreadable { slot: SlotNumber(50257) })`，checker 五条红（I-2.1、I-3.1、I-4.8、I-7.2、I-7.4）；HEAD 版 core 上逐字相同 | 不中：C 按 S3 成立、返回 txg 5；「从 B 发 E」在释放判定路径上报 `ReleaseTargetAlreadyReleased`、零次写；重开选中 txg 5，checker 全绿。按调用方手里的现行版（C）发 E、同一断电点：同样全绿 | 不中：E 被 `C381WriterStopped` 拒、零次写；重开选中 txg 5，全绿 | 不中：E 照发（txg 5，落点绕开扣住的 C 的槽）；重开选中 txg 5，全绿 |
| K2 S1（C 的单元写到一半） | 盖写 0 | 盖写 0 | 盖写 0 | 盖写 0 |
| K2 S2（记录 / 第二道屏障 / 根槽 FUA 报错，写**没**落盘） | 盖写 0 | 盖写 0；**另见「别的红」一行** | 盖写 0 | 盖写 0 |
| K2 S2′（根槽 FUA 报错而**根已落盘**，阶段 E） | **中**：盖写 1612 / 23760（打印出的第一例全在 C 的第 19 次写即根槽那一次） | 盖写 0 | 盖写 0 | 盖写 0 |
| K2 S3（盘 0 或盘 1 的超级块槽报错，根已落盘） | **中**：阶段 A 3224 / 15840，阶段 B 728 / 15528，阶段 C 1084 / 2416，阶段 E 3224 / 15840 | 盖写 0 | 盖写 0 | 盖写 0 |
| K2 S4（取号那两次超级块槽写）与写行 / 暖机里的写失败（阶段 D） | 盖写 0（S4 93 段、写行 / 暖机 868 段） | 盖写 0（S4 93 段、写行 / 暖机 9736 段） | 盖写 0 | 盖写 0 |
| 别的红：I-3.1，**不断电、不重开**，S2 失败之后原样重发成功 | 不红 | **红**（C 的第 17、18、19 次写上报错 + `Retry` 三段历史都红；乙′ 只在根槽那一格还红） | 不适用（重发被拒） | 不红 |
| 别的红：I-3.1，「记录已落盘、根没落盘」之后**可写重开** | 红 | 红 | 红 | 红 |

**这条腿的结论**：乙在 K2 的「盖写」问法下四段（加上 S2′、写行 / 暖机）一次都没中，K4 上挂得上、checker 全绿。乙中的只有一格 **I-3.1**：S2 失败之后不退回，原样重发（txg + 1、以不确定的那一版为上一版）一成功，就在不断电、不重开的合法状态上红。按四句判（第五节）：**这一格在同一段历史上分辨得出臂**（甲、丁在同一段历史上不红）；它的**病根却在四条臂共用的前提里**：可写重开施加一条根没落盘的记录之后，四条臂一起红，在 HEAD 版 core 上用一次纯断电（一个写错都不注入）就能复现（第六节）。乙′（记录那一步失败按甲退回、只有根槽报错才走「结局不确定」，攻方提的改法）把乙独有的红收窄到「根槽报错」那一格；剩下的那一格，系统当时看不到根到底落没落盘，只能靠改 checker 或改账的口径那一笔共用账来修。
甲在 S3 上中（即 Z2-c 本身），在「根槽报错而根已落盘」的 S2′ 上也中；这一格是这一轮新造出来的，Z2-c 原报告里没有。

**什么现象会推翻这张表**：在入库装置上重做阶段 A、E 时，乙、丙、丁任一在盖写那几行出现非零；或者主 agent 在 HEAD 版上重做第六节那段纯断电历史，重开之后 I-3.1 不红（那样「共用前提」这一判就不成立，乙那一格的 I-3.1 就归乙自己）。

## 二、四条臂在副本上写成什么样（补了什么）

四条臂都写在 `crates/singlefs-core/src/transaction.rs` 的 `publish_version` 里，按线程选（`set_c381_arm`，0 甲、1 乙、2 丙、3 丁、4 乙′）；臂的状态挂在分配器上（`PoolAllocator::c381`），所以换一个 `PoolWriter` 甩不掉它，重开时随分配器从盘上重建而清空。落盘闭包记下停在哪一步：S1 = 单元写与第一道屏障，S2 = 记录两盘、第二道屏障、根槽 FUA，S3 = 超级块槽轮换。副本的改动全在 `c381-arms-core.diff`（三个文件）与 `c381-arms-harness.diff`（`history.rs` 给新错误成员补三个名字，不改行为）。

| 臂 | 按正文第五节写的 | 我补的（按支持它的人会认的最强读法） |
|---|---|---|
| 甲 | 任一段失败，分配器整个换回 | 与 HEAD 同一条路径（换回之后把臂状态放回去，甲不用它）；K4 在两份 core 上逐字相同，见第三节 |
| 乙 | S1 退回；S2 不退回、结局不确定、只许 txg + 1 重发同一份内容；S3 不退回、这次成立 | ① S2 交回 `C381OutcomeUncertain { version }`，版本给调用方，重发就是**以这一版为上一版**的一次发布（txg + 1、记录号 + 1、反向链接它的记录），所以不确定那一版的单元在重发里被正常释放进 defer 队列。以 B 为上一版重发做不到：分配器没退回，B 被换下的角色已经释放过一次，释放判定路径报 `ReleaseTargetAlreadyReleased`（`transaction.rs` 第 885 行）。② 「不许发别的」在 `publish_version` 入口判：txg 必须是不确定那一版 + 1、上一版必须是它、内容必须相同，否则报 `C381OnlyRetryAllowed`，零次写。③ 重发在 S1 失败就退回到重发之前（不确定那一版之后）的样子，状态仍是不确定。④ S3 返回 `Ok(这一版)`。⑤ 挂载里的暖机（`mount.rs` 的 `publish_empty_after`）遇到 S2 就原样重发一次，`establish_instance` 那条「txg 与计划相同」的断言在乙下不判（重发多推一个 txg） |
| 丙 | S2 / S3 之后写入口作废，要重开 | 作废的标记挂在分配器上，之后的 `publish_version` 一律报 `C381WriterStopped`；重开从盘上重建分配器，标记随之清掉 |
| 丁 | S2 / S3 退回，这次占过的槽扣住到一个 txg 更大的根落盘 | 扣住是另一套只住内存的位（`DeviceFreeMap::c381_held`），用户数据落点、bump 游标、开新段、回落四条路都绕开它；一次发布成功、根的 txg 大于某条扣住记下的 txg，就放开那一条 |
| 乙′（攻方的改法，**只在我的模型上量过、被攻过零轮**） | — | 同乙，只把 S2 拆开：记录那一步（含第二道屏障）失败按 S1 退回（与甲同一条代码路径），只有根槽 FUA 报错才走「结局不确定」 |

装置（`c381_probe.rs`）：两块 4 GiB 稀疏内存盘，外面一层全局写计数（两盘共用一个下标）。`faults` 里的下标那一次写报错、默认不落盘（`LYING` 开着时写先落盘再报错）；下标 ≥ `crash_at` 的写一律报错、不落盘，就是断电。前缀固定为 mkfs → 取号 → 暖机 → 第一个事务（txg 3）→ 覆盖写 B（txg 4），拍一张快照，之后每段历史都从这张快照起。C 是 txg 5 的覆盖写，共 21 次写：下标 0–15 是八个单元各写两盘（S1），16、17 是记录的盘 0、盘 1，18 是根槽 FUA（S2），19、20 是超级块槽的盘 0、盘 1（S3）。C 在第 k 次写上报错之后，「用户」从五个动作里挑一串接着做：`Retry`（有不确定的那一版就从它重发，否则从手里的上一版原样重发失败的内容）、`Empty`（从现行版发空发布）、`Other`（从现行版发内容不同的覆盖写）、`EmptyFromB`（从 B 发空发布，即 Z2-c 第 2 步）、`Remount`（放下写入口、进程内可写重开）。之后清掉故障、可写重开、跑 checker（断电那一刻的镜像一次、重开之后一次）。

**用户动作全放开**（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」那一条）：只固定前缀与故障，五个动作排成长 0–3 的全部序列（156 条）逐条扫，每条序列的每个断电点都跑。

| 阶段 | 故障 | 动作序列 | 断电点 | 每条臂的段数 |
|---|---|---|---|---|
| A | C 的第 k 次写瞬时错（k = 0–20） | 长 0–3，156 条 | 不断电 + C 之后每一次写 | 甲 173171，乙 165720，丙 152571，丁 173171 |
| B | C 一次 + C 之后任一次写再一次 | 长 0–3，156 条 | 不断电 | 甲 169895，乙 162444，丙 149295，丁 169895 |
| C | C 一次 + C 之后任一次写再一次 | 长 0–1，6 条 | 不断电 + 每一次写 | 甲 33766，乙 35753，丙 30206，丁 33766 |
| D | C 无故障；C 之后第一步可写重开，重开里第 j 次写瞬时错（j = 0–29；0、1 是取号 = S4） | 重开 + 长 0–2，31 条 | 不断电 + 每一次写 | 甲 961，乙 9829，丙 961，丁 961 |
| E | 同 A，但 C 那次错是「写已落盘、设备照样报错」 | 长 0–3 | 同 A | 甲 172768，乙 164966，丙 151648，丁 172768 |
| 乙′ 片 | C 的第 16–18 次写（S2 三格）× 落盘 / 不落盘 | 长 0–3 | 同 A | 乙、乙′ 各六格，逐格列在第五节 |

## 三、K4：Z2-c 原历史在四条臂下

原历史（`research/prompts/m2-wave2-code-r1-opus-output.md` 第三节 Z2-c）按全局写下标改写：C（txg 5）的第 21 次写（下标 20，盘 1 的超级块槽）报错；调用方从 B 发空发布 E（原探针的 `empty_plan(&b)`）；E 的写从下标 21 起，八次单元写（四个固定点单元各写两盘，下标 21–28）之后，盘 0 的记录那一次（下标 29）起断电；清掉故障、可写重开。原探针用的是「盘 0 一直报错」，这里是「从下标 29 起全部不落盘」，E 在记录那一步之后什么都没写成，两种写法落到盘上的东西一样。

副本上的原样输出（`k4.log` 第 51–98 行里挑出的行，没改字）：

```
[甲] 原历史：E 从 B: steps=["err(BlockDevice)", "err(BlockDevice)"] writes=30 crash_at=Some(29)
    remount=REFUSED Recovery(UnitUnreadable { slot: SlotNumber(50257) })
[乙] 原历史：E 从 B: steps=["ok(txg 5)", "err(ReleaseTargetAlreadyReleased)"] writes=21 crash_at=Some(29)
    checker@crash=[]
    remount=ok(chosen 5)
    checker@after=[]
[乙] E 从调用方手里的现行版: steps=["ok(txg 5)", "uncertain(txg 6)"] writes=30 crash_at=Some(29)
    checker@crash=[]
    remount=ok(chosen 5)
    checker@after=[]
[丙] 原历史：E 从 B: steps=["err(BlockDevice)", "err(C381WriterStopped)"] writes=21 crash_at=Some(29)
    checker@crash=[]
    remount=ok(chosen 5)
    checker@after=[]
[丁] 原历史：E 从 B: steps=["err(BlockDevice)", "err(BlockDevice)"] writes=30 crash_at=Some(29)
    checker@crash=[]
    remount=ok(chosen 5)
    checker@after=[]
```

甲那一条的 checker 行太长，单独贴。甲的 checker（断电那一刻与重开之后逐字相同）：

```
checker@crash=["I-2.1: Violated(\"树 11（种类 1）的根 在盘 0 槽 50257 的那一份与位置条目里的校验和对不上\")", "I-3.1: Violated(\"盘 0：记账的已分配 Some(540672)，遍历全部有效根得到 475136\")", "I-4.8: Violated(\"最新根（txg 5）出发的遍历有单元对不上或读不出\")", "I-7.2: Violated(\"最新的根走不完：树 11（种类 1）的根 两份都读不到对得上的；树 12（种类 2）的根 两份都读不到对得上的；映射条目指的单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的\")", "I-7.4: Violated(\"最新根（txg 5）引用的单元已被复用或抹头（校验和对不上或头用不了）\")"]
```

这与 Z2-c 原报告的 `z2.log` 同一个槽（50257）、同样五条、同样的数（540672 / 475136）。**HEAD 版 core**（`pristine/`，三个 core 文件的 hash-object 与 `HEAD:` 那一版逐个相同：`transaction.rs` d7de039c…、`mount.rs` a23acac6…、`allocator.rs` 6544913c…）上用同一个探针（去掉臂开关的 `c381_probe_pristine.rs`）跑，甲这一条逐字相同（`k4-pristine.log`）。对照组（C 失败之后原样重发同内容）四条臂都挂得上、全绿，甲、丁 `chosen 5`，乙 `chosen 6`（重发 txg + 1），丙重发被拒、`chosen 5`。

| 臂 | 挂不挂得上 | checker | 为什么（每一步指到许可它的那一句） |
|---|---|---|---|
| 甲 | 挂不上 | 五条红 | C 在超级块槽失败时根已 FUA（`transaction.rs` 第 2113 行 `writer.perform(CommitStep::WriteRootRecordForceUnitAccess {`，之后第 2117 行才轮换超级块槽）；失败就整个换回（第 1260 行 `*allocator = allocator_before_this_publish;`）；E 的单元按同一个游标落到 C 的槽上；重开选中 C 的根，不核单元（核单元只在施加记录时做，`recovery.rs` 第 897 行 `if !all_verified {`），C 的树根读不出 |
| 乙 | 挂得上 | 全绿 | S3 按正文第五节「不退回；这次发布已成立，调用方拿到这次的版本」（背景材料第 53 行）；原历史第 2 步「从 B 发 E」在乙下是调用方拿着过时的上一版，释放判定路径当场报 `ReleaseTargetAlreadyReleased`（`transaction.rs` 第 885 行），一次写都没发。按「从调用方手里的现行版 C 发 E」改写之后，E（txg 6）在记录那一步断电 = S2，恢复选中 C 的根，C 的单元没被动过 |
| 丙 | 挂得上 | 全绿 | E 被拒；重开由恢复定：C 的根已落盘，选中 txg 5 |
| 丁 | 挂得上 | 全绿 | E 从 B 照发、txg 仍是 5，但 C 占过的槽扣着，E 的四个单元落在别处；重开选中 C 的根（txg 5），它引用的单元都在 |

## 四、K2：四条臂 × 四段，有没有一段可达历史让恢复结局引用被盖掉的单元

各阶段的原样汇总（日志里 `[臂 段] runs=… {…}` 那一行，逐字抄）：

```
阶段 A（phase-a.log）
[甲 S1] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[甲 S2] runs=24163 {"CLEAN": 17681, "OTHER_RED": 6482}
[甲 S3] runs=15840 {"CLEAN": 11584, "OTHER_RED": 1032, "OVERWRITE": 3224}
[乙 S1] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[乙 S2] runs=17656 {"CLEAN": 4642, "OTHER_RED": 13014}
[乙 S3] runs=14896 {"CLEAN": 13496, "OTHER_RED": 1400}
[丙 S1] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[丙 S2] runs=12011 {"CLEAN": 4193, "OTHER_RED": 7818}
[丙 S3] runs=7392 {"CLEAN": 6824, "OTHER_RED": 568}
[丁 S1] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[丁 S2] runs=24163 {"CLEAN": 14953, "OTHER_RED": 9210}
[丁 S3] runs=15840 {"CLEAN": 14808, "OTHER_RED": 1032}
阶段 B（phase-b.log）
[甲 S1] runs=130672 {"CLEAN": 121968, "OTHER_RED": 8704}
[甲 S2] runs=23695 {"CLEAN": 18471, "OTHER_RED": 5224}
[甲 S3] runs=15528 {"CLEAN": 13948, "OTHER_RED": 852, "OVERWRITE": 728}
[乙 S1] runs=130672 {"CLEAN": 114576, "OTHER_RED": 16096}
[乙 S2] runs=17188 {"CLEAN": 4172, "OTHER_RED": 13016}
[乙 S3] runs=14584 {"CLEAN": 12894, "OTHER_RED": 1690}
[丙 S1] runs=130672 {"CLEAN": 117456, "OTHER_RED": 13216}
[丙 S2] runs=11543 {"CLEAN": 4037, "OTHER_RED": 7506}
[丙 S3] runs=7080 {"CLEAN": 6512, "OTHER_RED": 568}
[丁 S1] runs=130672 {"CLEAN": 121968, "OTHER_RED": 8704}
[丁 S2] runs=23695 {"CLEAN": 17855, "OTHER_RED": 5840}
[丁 S3] runs=15528 {"CLEAN": 14676, "OTHER_RED": 852}
阶段 C（phase-c.log）
[甲 S1] runs=27232 {"CLEAN": 25952, "OTHER_RED": 1280}
[甲 S2] runs=4118 {"CLEAN": 2674, "OTHER_RED": 1444}
[甲 S3] runs=2416 {"CLEAN": 1314, "OTHER_RED": 18, "OVERWRITE": 1084}
[乙 S1] runs=30352 {"CLEAN": 27328, "OTHER_RED": 3024}
[乙 S2] runs=3063 {"CLEAN": 1279, "OTHER_RED": 1784}
[乙 S3] runs=2338 {"CLEAN": 2200, "OTHER_RED": 138}
[丙 S1] runs=27232 {"CLEAN": 25952, "OTHER_RED": 1280}
[丙 S2] runs=1982 {"CLEAN": 946, "OTHER_RED": 1036}
[丙 S3] runs=992 {"CLEAN": 974, "OTHER_RED": 18}
[丁 S1] runs=27232 {"CLEAN": 25952, "OTHER_RED": 1280}
[丁 S2] runs=4118 {"CLEAN": 1662, "OTHER_RED": 2456}
[丁 S3] runs=2416 {"CLEAN": 2398, "OTHER_RED": 18}
阶段 D（phase-d.log）
[甲 S4 取号] runs=93 {"CLEAN": 93}
[甲 写行 / 暖机] runs=868 {"CLEAN": 806, "OTHER_RED": 62}
[乙 S4 取号] runs=93 {"CLEAN": 93}
[乙 写行 / 暖机] runs=9736 {"CLEAN": 5462, "OTHER_RED": 4274}
[丙 S4 取号] runs=93 {"CLEAN": 93}
[丙 写行 / 暖机] runs=868 {"CLEAN": 806, "OTHER_RED": 62}
[丁 S4 取号] runs=93 {"CLEAN": 93}
[丁 写行 / 暖机] runs=868 {"CLEAN": 806, "OTHER_RED": 62}
阶段 E（phase-e.log）
[甲 S1（报错但已落盘）] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[甲 S2（报错但已落盘）] runs=23760 {"CLEAN": 15976, "OTHER_RED": 6172, "OVERWRITE": 1612}
[甲 S3（报错但已落盘）] runs=15840 {"CLEAN": 11584, "OTHER_RED": 1032, "OVERWRITE": 3224}
[乙 S1（报错但已落盘）] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[乙 S2（报错但已落盘）] runs=16902 {"CLEAN": 5154, "OTHER_RED": 11748}
[乙 S3（报错但已落盘）] runs=14896 {"CLEAN": 13496, "OTHER_RED": 1400}
[丙 S1（报错但已落盘）] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[丙 S2（报错但已落盘）] runs=11088 {"CLEAN": 3412, "OTHER_RED": 7676}
[丙 S3（报错但已落盘）] runs=7392 {"CLEAN": 6824, "OTHER_RED": 568}
[丁 S1（报错但已落盘）] runs=133168 {"CLEAN": 119952, "OTHER_RED": 13216}
[丁 S2（报错但已落盘）] runs=23760 {"CLEAN": 14860, "OTHER_RED": 8900}
[丁 S3（报错但已落盘）] runs=15840 {"CLEAN": 14808, "OTHER_RED": 1032}
```

「OVERWRITE」只出现在甲（S3，以及阶段 E 的 S2）。所有「OTHER_RED」只有 I-3.1 一条红，日志里每一类的第一例都打印出来了（`OTHER_RED remount=ok crash=[…] after=["I-3.1"]`），没有一例带别的不变量。阶段 D 里乙的段数多出一个数量级，是因为乙在暖机遇到 S2 时原样重发，挂载成功之后后面的动作序列接着跑；甲、丙、丁挂载一失败，后面就只剩最后那次重开。

### 甲中的两格（逐步写）

**S3（Z2-c 本身的一族）**：阶段 A 的第一例 `k=19 crash_at=22 actions=[Empty]`：C 在盘 0 的超级块槽（下标 19）报错 → 甲整个换回分配器 → 从现行版 B 发空发布（txg 5）→ 它的第 2 次写之后断电 → 重开 `REFUSED Recovery(UnitUnreadable { slot: SlotNumber(50257) })`。换成 `Other`（内容不同的覆盖写）读不出的是 50184，换成 `Retry` 在不同断电点读不出 50258、50260、50264。还有一类重开**挂得上**而 checker 红：`k=19 crash_at=25 actions=[Retry]`，`remount=ok(chosen 5)`，`["I-2.1", "I-4.8", "I-7.4"]`：重开没读到被盖的那个单元，是池里留着一颗雷，不是没中。

**S2′（根槽报错而根已落盘，这一轮新造的一格）**：阶段 E 的第一例 `fault=18 crash_at=21 actions=[Empty]`，`remount=ok(chosen 5)`，断电那一刻与重开之后 checker 都是 `["I-2.1", "I-4.8", "I-7.4"]`；`crash_at=21 actions=[Other]` 重开报 `UnitUnreadable { slot: SlotNumber(50184) }`。步骤：C 的根槽 FUA 那一次写落了盘、设备照样报错 → 甲按失败换回 → 下一次发布盖到 C 的槽上 → 重开选中 C 的根。条款上根槽 FUA 报错不说明根没落盘；甲在这一格与 S3 是同一个病。打印出的六个盖写的第一例全在 `fault=18`；同一片不开「落盘但报错」时（阶段 A 的 S2）甲一次盖写都没有，所以 S2′ 只在根槽那一次写上出现（这一句是从这两组数推出来的，没有按 k 单独拆过计数）。

### 乙、丙、丁在盖写上一次都没中

- 乙 S2：分配器不退回，不确定那一版占的槽一直占着，重发把它们正常释放进 defer 队列；恢复要么施加它的记录（单元都在），要么选到重发的根；没有一段历史能把别的发布写到它的槽上。
- 乙 S3：返回这一版，与成功同一条路。
- 丙：失败之后不再发布，重开由恢复定、分配器从盘上重建。
- 丁：扣住的槽不发出去；放开的条件是一个 txg 更大的根落盘，而丁的下一次成功发布仍用失败那次的 txg、写同一个根槽，所以放开之前失败那次的根一定已被盖掉。
- 甲在 S2（写没落盘）上也不中：盖了 C 的单元之后断电，恢复施加 C 的记录之前逐个核点名单元的校验和，对不上就停（`recovery.rs` 第 897 行 `if !all_verified {`），退回 B。

**取样范围**：C 的 21 个故障点 × 156 条动作序列 × 每个断电点（阶段 A、E），加两次瞬时错不断电（阶段 B，序列长 0–3）与两次瞬时错加断电（阶段 C，序列长 0–1），加重开里的 30 个故障点（阶段 D）。没取到的：记录写一半（撕裂）、屏障本身报错、两块盘以外的几何、F 抬过之后（`raise_rollback_floor`）、回退挂载（`mount_rollback`）、盘上已有被抛弃时间线的池（第七节）。

## 五、乙中的一格：S2 之后原样重发一成功，I-3.1 在不断电、不重开的状态上红

**历史**（`s2-retry.log`，`c381_single` 跑的，`C381_K=16|17|18 C381_ACTIONS=Retry`）：前缀到 B（txg 4）→ C（txg 5）的第 17 次写（下标 16，盘 0 的记录）报错 → 乙：S2，不退回，交回不确定的那一版（背景材料第 53 行乙的 S2 格「不退回；这次发布按「结局不确定」对待，写入口只许按 D23（journal 的角色与格式） 已定项 14 以 txg + 1 重发同一份内容」）→ 调用方照做：以这一版为上一版、txg 6、同内容重发，成功 → **不断电**，镜像上跑 checker。

原样（乙那一段，k = 16）：

```
[乙] k=Some(16) later=[] actions=[Retry] crash=None lying=false
  steps=["uncertain(txg 5)", "ok(txg 6)"] writes_after_c=17 writes=38
  checker@crash=["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 540672\")"]
  remount=ok(chosen 6)
  checker@after=["I-3.1: Violated(\"盘 0：记账的已分配 Some(868352)，遍历全部有效根得到 704512\")"]
```

k = 17、18 逐字同上（同一个文件）。同一段历史，别的臂：甲、丁 `steps=["err(BlockDevice)", "ok(txg 5)"]`，两次 checker 都是 `[]`；丙重发被拒（`err(C381WriterStopped)`），不重开时镜像上 `[]`。差 704512 − 540672 = 163840 字节 = 10 个 16 KiB 槽，就是 C 那一版占的槽：账把它们算作已分配（在 defer 队列里），而环里没有任何一条根引用它们——C 的根从没写过，checker 按「遍历全部有效根」取并集（`crates/singlefs-checker/src/walk.rs` 第 1463 行 `format!("盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}")`）。重开之后照样红：这几个槽要等释放代 ≤ F_生效才回收。

**四句**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：

1. **分不分辨臂**：在这段历史上分辨（乙红，甲、丁绿，丙不重开时绿）。但同一个病根——**一版的单元被释放进 defer 队列，而引用它的那一版在环里没有根**——四条臂都走得到：可写重开施加一条「记录已落盘、根没落盘」的记录，重开自己的写行发布把那一版的单元换下，四条臂一起红（第六节；HEAD 版上一个写错都不注入就复现）。乙只是多了一条**不重开**就走到那个状态的路。
2. **系统当时看不看得到判别它的东西**：分两半。C 在**记录那一步**（下标 16、17）失败时，根槽 FUA 还没发，乙的写入口知道「C 的根不可能落盘」，本可以按 S1 退回——这一半看得到，是乙把它和根槽那一步并进了同一段。C 在**根槽 FUA** 那一次（下标 18）报错时看不到：阶段 E 与乙′ 片表明两种情形都会出现（根没落 → 重发之后 I-3.1 红；根落了 → C 的根在环里、引用着那几个槽，绿）。要求系统在这一格对两种情形给出不同的账，是「判别子观测不到」那一形。
3. **满足的是判据字面的哪一个分句**：K2 的问列是「引用被盖掉的单元」，这一格没有任何单元被盖；它满足的是 K2 触发列的「checker 红」与正文第六节推翻条件里的「让 checker 在合法状态上红」。按 K2 的问法它**归错了判据**，应记到「checker 在合法状态上红」那一句名下，门槛不同，由主 agent 定。
4. **跑前条款给的改法在这几格上还中不中**：正文第七节反向接受条款只写「乙被打中且那一格它自己改不掉 → 记它输一次」，没列改法。我试的改法见下表。

每格写「断电那一刻已红 + 只在重开之后红 / 段数」（「断电那一刻」含不断电的段：序列走完的镜像）。

| 改法 | 下标 16（记录盘 0） | 下标 17（记录盘 1） | 下标 18（根槽，没落盘） | 下标 18（根槽，已落盘） | 依据 |
|---|---|---|---|---|---|
| 乙原样 | 1382 + 364 / 6388 | 3622 + 2012 / 5634 | 3622 + 2012 / 5634 | 0 + 480 / 5634 | 量过（`b-prime.log`） |
| 乙′：记录那一步失败按 S1 退回（与甲同一条代码路径），只有根槽报错走「结局不确定」 | 0 + 826 / 8323 | 1514 + 1314 / 7920（那 1514 段都带序列里的 `Remount`） | 3622 + 2012 / 5634（与乙相同） | 0 + 480 / 5634（与乙相同） | 量过（`b-prime.log`） |
| checker 的并集把「只由 journal 记录施加出来、根没落盘的那一版」也算进去（或账上把「引用它的那一版没有根」的已释放槽另记一类） | 推的：全绿 | 推的：全绿 | 推的：全绿 | 推的：全绿 | 推的，没实现没跑；它同时修第六节那一格，是那笔共用账的改法 |

「只在重开之后红」那一类，与带 `Remount` 的「断电那一刻已红」，都是第六节那件事（进程内或最后那次可写重开施加了一条根没落盘的记录）。乙独有的是「不带任何重开、断电那一刻已红」那一类：乙原样在三个下标上都有（`first: crash_at=None actions=[Retry]`），乙′ 只在下标 18、根没落盘那一格还有。

乙′ 在下标 16、17 上走的就是甲的换回路径。

「乙独有的那一类」单独核过（`c381_k2d.rs`，序列只用不含 `Remount` 的四个动作、长 0–3，85 条，每个断电点，数断电那一刻镜像上红的段；`no-remount.log` 原样）：

```
[甲 k=16] 不带重开的段 3409，断电那一刻已红 0；第一例 
[甲 k=17] 不带重开的段 3409，断电那一刻已红 0；第一例 
[甲 k=18] 不带重开的段 3409，断电那一刻已红 0；第一例 
[乙 k=16] 不带重开的段 1669，断电那一刻已红 774；第一例 crash_at=None actions=[Retry] steps=["uncertain(txg 5)", "ok(txg 6)"] ["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 540672\")"]
[乙 k=17] 不带重开的段 1669，断电那一刻已红 774；第一例 crash_at=None actions=[Retry] steps=["uncertain(txg 5)", "ok(txg 6)"] ["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 540672\")"]
[乙 k=18] 不带重开的段 1669，断电那一刻已红 774；第一例 crash_at=None actions=[Retry] steps=["uncertain(txg 5)", "ok(txg 6)"] ["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 540672\")"]
[乙′ k=16] 不带重开的段 3409，断电那一刻已红 0；第一例 
[乙′ k=17] 不带重开的段 3409，断电那一刻已红 0；第一例 
[乙′ k=18] 不带重开的段 1669，断电那一刻已红 774；第一例 crash_at=None actions=[Retry] steps=["uncertain(txg 5)", "ok(txg 6)"] ["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 540672\")"]
```

甲在这 3 × 3409 段里一次都不红，乙三个下标各 774 段红，乙′ 只在下标 18 红 774 段。所以把用户动作放开之后，「分辨臂」这一判仍成立：同一前缀、同一故障，甲在任何不重开的后缀上都不红（丁、丙没跑这一核；丁在下标 16–18 与甲走同一条换回路径，只多扣住，推的是同样 0）。**乙′ 与第三行的 checker 改法都只在我的模型上量过或推过，被攻过零轮。**

`b-prime.log` 里乙′ 的原样（下标 17、根没落盘那一格，逐字）：

```
[乙′ k=17 报错但已落盘=false] runs=7920
      1514 remount=ok crash=["I-3.1"] after=["I-3.1"]
           first: crash_at=None actions=[Remount] steps=["err(BlockDevice)", "remount ok(chosen 4)"]
      1314 remount=ok crash=[] after=["I-3.1"]
           first: crash_at=None actions=[] steps=["err(BlockDevice)"]
      5092 remount=ok crash=[] after=[]
           first: crash_at=None actions=[Retry] steps=["err(BlockDevice)", "ok(txg 5)"]
```

（`crash=["I-3.1"]` 的那 1514 段都带一次序列里的 `Remount`：进程内重开施加了 C 的记录，之后的写行发布把 C 的单元换下，下一次断电之前镜像就红了，与第六节同一件事。）

## 六、四条臂一起中的一格：记录落盘、根没落盘，可写重开之后 I-3.1 红（共用前提的账）

**历史**：前缀到 B（txg 4）→ C（txg 5）写完八个单元与两份记录，根槽那一次写（下标 18）起断电——**一次写错都不注入**，是层 0 意义上的一个普通崩溃点 → 清掉断电、可写挂载 → checker。

**HEAD 版 core 上**（`pristine/`，core 三个文件与 `HEAD:` 逐个同 hash；`c381_probe_pristine.rs` 去掉了臂开关，四个「臂名」跑的是同一份代码）原样（`pristine-crash.log`，断电点 18 那一组的第一段）：

```
[甲] k=None later=[] actions=[] crash=Some(18) lying=false
  steps=["err(BlockDevice)"] writes_after_c=19 writes=19
  checker@crash=[]
  remount=ok(chosen 4)
  checker@after=["I-3.1: Violated(\"盘 0：记账的已分配 Some(704512)，遍历全部有效根得到 638976\")"]
```

断电点 17（只有盘 0 一份记录）同样红、同样的两个数。断电那一刻的镜像全绿；重开选中 B 的根（txg 4），施加 C 的记录（txg 5 > 4，点名单元都核得过），在施加出来的那一版上写行、暖机。差 704512 − 638976 = 65536 字节 = 4 个槽，与写行那次发布换下的四个固定点单元个数相同：推的是这四个 C 的单元进了 defer 队列、账还算它们已分配，而引用它们的只有 C，C 的根从没写进环里，checker 的并集里没有它们（推的，没打印落点）。

**判定**：这一格让四条臂一起中（阶段 A–E 里每条臂的「OTHER_RED」大多数是它），是正文第七节反向接受条款末句「两条以上的臂在同一格一起中，那一格不拿来判臂，另记一笔共用前提的账」那一种。它也不是 C381 这一题独有：一个写错都没有，只是崩在记录与根之间。病根在两处之一：I-3.1 的 checker 读法（「遍历全部有效根」没把只由记录施加出来的版本算进去），或释放代口径（C380（环上有洞时释放代判不出唯一值）同一族：施加的记录做的释放落在一个根槽没写过的 txg 上）。哪一处要改不归这条腿判；这里只报：**它今天就在 HEAD 上红，层 0 若只在崩溃镜像上判 checker、不在重开之后再判，就看不到它。**层 0 有没有「重开之后再判」我没去查，由主 agent 现查。

第五节乙那一格与这一格是同一个病根走两条路：这一格靠重开施加记录走到，乙那一格靠「不退回 + 以不确定的那一版为上一版重发」走到，不用重开。修这一格的 checker 或账的改法（第五节表第三行，推的）两格一起修。

**什么现象会推翻这一节**：主 agent 在入库装置上（HEAD，不打任何补丁）做同一段历史，重开之后 I-3.1 不红；或者查到 I-3.1 的读法里本来就允许这种差（那样这一格与第五节乙那一格一起不算中）。

## 七、没打中的形状（试过什么、取样多大）

| 形状 | 打哪条臂 | 取样 | 结果 |
|---|---|---|---|
| S2 之后乙的重发与别的发布交错（重发之前发空发布、别的内容、从 B 发） | 乙 | 阶段 A、C、E 的全部序列 | 入口一律报 `C381OnlyRetryAllowed`、零次写；没有一段盖写 |
| 重发自己再在 S1 / S2 / S3 失败（第二次瞬时错落在重发里） | 乙 | 阶段 C（序列长 0–1，C 之后每一次写各一次错 × 每个断电点） | 盖写 0；I-3.1 红的都是第五、六节那两种 |
| 暖机里的写失败、乙在挂载内原样重发一次 | 乙 | 阶段 D，9736 段 | 盖写 0 |
| 根槽 FUA「报错但已落盘」 | 四条臂 | 阶段 E | 只有甲中 |
| 丁的扣住放开之后，槽被再分出去时失败那次的根还在环里 | 丁 | 阶段 A–E 全部序列（长 ≤ 3，最多三次发布或重开） | 没有一段；丁的下一次成功发布仍用失败那次的 txg、盖掉同一个根槽，才轮得到 txg 更大的根 |
| 丙在「写入口作废」之后换一个 `PoolWriter` 接着发 | 丙 | 作废标记挂在分配器上，换写入口甩不掉（副本的写法）；序列里的 `Remount` 之后分配器从盘上重建、标记清掉，是合法的「重开」 | 盖写 0 |

## 八、这条腿自己的限度

- 故障模型只有「某一次写报错（落不落盘两种）」与「从某一次写起断电」；**没有**撕裂写（一次记录或一个单元只落一半）、屏障报错、读错、盘掉线。
- 只有两块盘、一个固定前缀（txg 3 的第一个事务 + txg 4 的覆盖写），C 固定是 txg 5（根落区域 2）。换一个 txg 让根落到别的区域、或前缀里已有被抛弃的时间线、F 已被抬过（`raise_rollback_floor`）、回退挂载（`mount_rollback`），都没扫。
- 乙的「重发同一份内容」我按「以不确定的那一版为上一版、记录号 + 1、事务号 + 1」写。另一种写法（记录号不变、覆盖不确定那一版的记录槽）没实现。两种写法在 K2 上应当同答，这是推的：施加只看记录号连续与点名单元的校验和（`recovery.rs` 第 897 行）。
- 乙 S3 返回的那一版，调用方拿去当上一版接着发；「从 B 发」在乙下被释放判定路径拒掉，是现有代码的检查碰巧拦住的，不是乙自己写的规则。换一个不重写被换下角色的发布形状，这道拦可能不在（没试）。
- 用户动作只有五种，长度 ≤ 3；阶段 C（两次瞬时错 + 断电）只配了长 ≤ 1 的序列。
- 乙的 S2「只许重发同一份内容」有没有「空间准入不过」（正文第六节推翻条件第二句）：不归这条腿，也没专门量；阶段 A–E 的日志只留了每一类的第一例，没数准入被拒的段数。
- 每一格只跑了一次（装置是确定性的，同一输入同一输出）；「没打中」按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」只算一次。

## 九、没做什么

- 不判 K1、K3、K5、K6，不替主 agent 采纳；副本上量出的数不算入库装置上的数，甲的 K4 与第六节那一格在 HEAD 版 core 上复跑过，也仍是副本（`/tmp` 下的仓拷贝），要引须在入库装置上重做。
- 没改主工作区任何文件；副本的 harness 取的是 `HEAD:` 那一版（主工作区里另一实现员没提交的 `history.rs`、`lib.rs`、`second_transaction_supplement_three_random_history.rs`、`mutations.tsv` 换成 `git show HEAD:路径`，未跟踪的 `model.rs`、`model_comparison.rs` 删掉）。
- 没查层 0 是否在重开之后再跑 checker（第六节交主 agent 现查）。
- 没跑门禁、没跑变异表。
- 草稿与副本留在 `/tmp/claude-1000/c381-r1-opus/`（`repo/` 是改了臂的副本、`pristine/` 是 HEAD 版 core 的副本、两个 target 目录、各阶段日志、`progress.md`）；日志与源文件都拷进了模型目录，副本本身没入库（150 MB 的仓拷贝，按复跑命令可重建）。
