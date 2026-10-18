# dyn-share-r1 攻方腿报告（Opus，第一轮，2026-10-18）

攻击面：K4 打穿分工、K5 判段的闲置、K6 改动量与风险；另对甲、乙、丙、丁各找具体历史。被判对象是 `research/prompts/_dyn-share-r1-body.md` 第三、四节与它点名的代码。代码行号是主工作区此刻的（与正文里的行号有出入，见「这条腿自己的限度」）。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/dyn-share-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本（要有 crates/、Cargo.toml、Cargo.lock、.claude/heavy-thread-default）> --work <新的草稿目录>
```

跑四个 python3 时序模型（只用标准库）、拷出 8 份变体树、用仓根编好的 `target/debug/unit-graph key` 算四组判法单元的键；每一样打一行 `name=rerun_same` / `name=rerun_differs`。我在本机拿主工作区当冻结副本跑过一遍，5 行都是 `rerun_same`（草稿目录 `~/.local/share/singlefs/work/dyn-share-r1-attack/rerun-check`，交回前已删）。

模型目录各文件的 sha256：

```
a01f49c778c999d980f6a7ff2d0697ba48f87876addc7de1ac0926bffdb9eb5e  research/prompts/dyn-share-r1-opus-model/central_lease.py
3a2a4eaa26452295f2d56e7b0ba0a5d4ac53370cc27954a21223887d66d6d0b7  research/prompts/dyn-share-r1-opus-model/library_merge.py
9775fb5a346a23e5d76193cdd48cd784a4a3891fabdecd09161261df37683b41  research/prompts/dyn-share-r1-opus-model/out/central_lease.out
187aeebe5afa7d799e222af7e2dbc1bb46566e0d999f8feba046dd8ded92994b  research/prompts/dyn-share-r1-opus-model/out/compare_keys.out
072bc5fb63005db6d7fa8956262b1304f4f5c77e1edeb8fb9735894ee6256fdb  research/prompts/dyn-share-r1-opus-model/out/library_merge.out
fb80a8f0b9ec7598369d960e4a711e1f833b89c6a27c4d10b42f706d12127539  research/prompts/dyn-share-r1-opus-model/out/periodic_reshare.out
a2fa033c6b672b06b7d9591a1eef8cb132689af5aeb3a8b2076925b4db31ee52  research/prompts/dyn-share-r1-opus-model/out/two_ended_sweep.out
c38a7b513d31c0ced07f6443a67a4feae1b12e65a218f437ab18b453e1fb9fa9  research/prompts/dyn-share-r1-opus-model/periodic_reshare.py
16583bb50ac5388ccee5bbf1bf313eac9d3c29455eb710dc7a2f730fdc4a5201  research/prompts/dyn-share-r1-opus-model/tools/compare_keys.sh
319c1b114f700736b30e0e642a37eb8e56df9f93a3b6d38af401e8ab6a024bd2  research/prompts/dyn-share-r1-opus-model/tools/make_variants.py
529c1024bfb127a24d5b43b82b990753275d98e625f431c13395e9cdac13038e  research/prompts/dyn-share-r1-opus-model/tools/rerun.sh
444b73db81c38e2508154efd89cb38778fbdf8090ea9d5095dfbb0dc46187d39  research/prompts/dyn-share-r1-opus-model/two_ended_sweep.py
```

## 各格判定一览

| 格 / 候选 | 判定 | 一句话 | 分不分辨臂 |
|---|---|---|---|
| K4 甲 | 打中 | 录入段与判段「各扫一遍」时两次相遇点不同，判段一侧读不到对方录的观察者块，`judge_the_observers_of_the_flow` 整趟报错；前沿只写块序号时录入段另有洞 | 分辨（只打甲） |
| K4 乙 | 打中（边角） | 回话丢了不收回就留一段洞；收回的超时短过最慢一段就重复；第二台掉线时账上「做完」的段结果取不回；另外 ssh 只有本机连第二台这一个方向 | 分辨（只打乙） |
| K4 丙 | 打中 | 照字面「只重起第二台、引擎不改」，本机那一趟的分工开跑时就定死，重分之后两份权重之下都不归本机的块谁都不做：64 格模型里 64 格都留洞（511 到 9063 块），重起最多 55 次 | 分辨（只打丙的字面读法；两台都停的读法 64 格里一格都没洞） |
| K4 丙：手里那一块 | 没打中 | 停下文件已经保证判完写进库；按旧权重导入以后这一块照键认，不重判 | — |
| K5 | 打中（不分辨臂） | 甲乙丙都只在一段之内重分；同一趟里观察者录入段（CPU）跑完才起判段（GPU），录入那几个小时里五张卡全空，这是三个候选共用的前提。另外：镜像判定不读观察者录入块，只有观察者结论要「各判各录」 | 不分辨（甲乙丙一起中） |
| K6 | 打中 | 量过：录入段循环、平滑轮转、两处导入改了，四组键一个都不变；顶上那行 `use`（它引了叫停用的类型）、`checking_threads`、`.claude/heavy-thread-default`（连只改理由那一句也算）改了，观察者结论拼块那一组的键就换，库里观察者结论全部作废 | 分辨程度看候选：碰叫停、碰线程的改法会中 |
| 丁（取回并进） | 打中 | 正文字面只并「观察者块（连同它桶里的见证）」。第二台在一块的半中间被停时，见证引的镜像有的没并进来；本机之后照见证复用、不读镜像，判段报「恢复镜像不在库里」，重录也补不上 | 分辨（只打丁的并法） |
| 丁（第二台的库留着） | 打中（规则冲突） | 与「第二台那一趟的树连同库跑完就删」、D13 已定项 17「每类判定一份库」「第二台判出的并进本机那一份」直接冲突；掉线时取回的那一份根本没有 | 分辨 |
| 我自己撤回的一处 | — | 我先猜「同一块两台都录，录入块字节会不一样，按字节对账会误报冲突」。模型 40 个样本里字节都相同：见证可靠时一个状态的镜像键是唯一的。这条撤回 | — |

## 一、K4 打穿分工

K4 的字面，`research/prompts/_dyn-share-r1-body.md:41`：「甲：两边前沿越过的那一块会不会两台都录或都没录；乙：发块方挂了、领块请求丢了会怎样；丙：重分那一刻第二台手里那一块算谁的、重起的次数上界」。正文没有跑前条款给的改法，所以四句里的第四句，我改成核「最顺手的那个改法」在打中的那几格上还中不中。

### 1.1 甲：两端对扫

许可这几段历史的句子：`research/prompts/_dyn-share-r1-body.md:29`：「录入段与判段各扫一遍。见证不跨机器实时共享（照今天）。」

用到的代码事实：
- 观察者的录入段一个观察者扫一遍，每扫一遍块序号都从 0 起：`crates/singlefs-checker-tier/src/crash_amplification.rs:17549`：「for (observer, observer_versions) in observers.iter().zip(versions) {」，下面 `crates/singlefs-checker-tier/src/crash_amplification.rs:17554`：「let mut block_sequence = 0u64;」。第 6 条 unlink 挂了 every_state 与 mount_continuation 两个观察者，第二台就是先录完 every_state 再录 mount_continuation：`records/2026-10-13-复用键与全仓改名.md:951`：「第二台 every_state 段已录完」。
- 判段判观察者结论只读本机库里的录入块，读不到整趟报错：`crates/singlefs-checker-tier/src/crash_amplification.rs:18677`：「的录入段那一块不在库里：先跑录入段」。
- 同一趟里先录观察者、再判：`crates/singlefs-checker-tier/src/crash_amplification.rs:12616`：「let observer_recording = if records && !placement_only && !observers.is_empty() {」，镜像判定在 `crates/singlefs-checker-tier/src/crash_amplification.rs:12842`：「let gpu_judging = judge_my_blocks_on_gpu(」，观察者结论在 `crates/singlefs-checker-tier/src/crash_amplification.rs:13052`：「Some(judge_the_observers_of_the_flow(」。

模型 `two_ended_sweep.py`：每遍 400 块，三遍（every_state、mount_continuation、判段）。块的用时取量到的速度：第二台 every_state 13402 块用 9611 秒、mount_continuation 每秒 1.0 块（`records/2026-10-13-复用键与全仓改名.md:951`：「每秒 1.0 块」），判速本机 8.8 万、第二台 17.5 万态每秒（`records/2026-10-13-复用键与全仓改名.md:945`：「两台判速差（上一趟量的本机 8.8 万、第二台 17.5 万每秒）」）。本机录入速度没量到，按第二台的 0.5、1、2 倍扫。前沿有两种写法：只写块序号（正文「领到哪了」的字面），或写成（第几遍，块序号）。另外扫读前沿的间隔 5、30、120 秒，以及跑完之后前沿留在最后一块还是写成哨兵值。

**历史 甲-H1（前沿写成（第几遍，块序号），谁都不停）。** 两台 CPU 一样快、每 5 秒读一次前沿：两遍录入都在第 200 块附近相遇，本机录 0–199，第二台录 200–399。判段里第二台快一倍，相遇点在 139 左右，于是第二台判 139–399，其中 61 块的观察者录入块只在本机库里。第二台判到第 199 块时，`judge_the_observers_of_the_flow` 报「…的录入段那一块不在库里：先跑录入段」，那一趟整趟退出。模型输出原样：

```
name=jia_no_stop front=phased poll_seconds=5 local_cpu_factor=1.0 done_front=last every_state_dup=2 every_state_holes=0 every_state_local=201 every_state_peer=201 mount_continuation_dup=0 mount_continuation_holes=0 mount_continuation_local=200 mount_continuation_peer=200 judge_dup=0 judge_holes=0 judge_local=139 judge_peer=261 judge_reads_elsewhere_local=0 judge_reads_elsewhere_peer=61 finish_local=455 finish_peer=455
```

这种写法下 18 格全中（数法见下面那段命令输出），一格最多 127 块。只有两台录入速度之比恰好等于判速之比时才不中，扫的点里一个都没有。

**历史 甲-H2（前沿只写块序号）。** 本机 CPU 是第二台的一半、每 5 秒读一次：本机还在录 every_state 时，第二台已经录完 every_state、转去从 399 往下录 mount_continuation，于是本机读到的前沿是 mount_continuation 那一遍的块号，拿它当 every_state 的边界。结果 every_state 有 110 块两台都录了，mount_continuation 有 175 块谁都没录。判段判到这 175 块中任何一块都报同一句错。

```
name=jia_no_stop front=single poll_seconds=5 local_cpu_factor=0.5 done_front=last every_state_dup=110 every_state_holes=0 every_state_local=245 every_state_peer=265 mount_continuation_dup=0 mount_continuation_holes=175 mount_continuation_local=68 mount_continuation_peer=157 judge_dup=0 judge_holes=0 judge_local=65 judge_peer=335 judge_reads_elsewhere_local=0 judge_reads_elsewhere_peer=178 finish_local=540 finish_peer=490
```

只写块序号的 18 格里，7 格录入段有洞。两台一样快、每 5 秒读一次的那一格两边对称，碰巧没洞（every_state_dup=2、mount_continuation_holes=0）：只量这一个点，就会把这个错漏过去。

**历史 甲-H3（第二台中途被停）。** 前沿写成（第几遍，块序号），每 30 秒读一次，第二台在第 20 到 380 秒之间的某一刻被停（停下文件、TERM、掉线三种）。第二台的前沿停在它自己那一遍，本机读到对方「还没进下一遍」，于是把后面几遍全领下来，判段也一直判到第二台录的那一段。30 格里 29 格本机判段读到第二台录的块。今天这种情况本机那一份照样能判完自己那一半；甲之下本机那一趟反而整趟报错。TERM 与掉线时，第二台手里那一块谁都没录（abandoned_claims=1、录入那一遍 holes=1）。

```
name=jia_peer_stops front=phased stop_kind=stop-file stop_at=100 abandoned_claims=0 every_state_holes=0 every_state_peer=134 mount_continuation_holes=0 mount_continuation_peer=0 judge_holes=0 judge_peer=0 judge_reads_elsewhere_local=134
name=jia_peer_stops front=phased stop_kind=term stop_at=100 abandoned_claims=1 every_state_holes=1 every_state_peer=133 mount_continuation_holes=0 mount_continuation_peer=0 judge_holes=0 judge_peer=0 judge_reads_elsewhere_local=134
```

**「两台都录」的界（K4 字面的前一半）。** 两台都录的块数随读前沿的间隔涨。前沿写成（第几遍，块序号）、两台一样快时，判段那一遍每 5、30、120 秒读一次，分别重复 0、40、86 块。原因是前沿只在读的那一刻是新的，两台各自照旧值又往前领了一个间隔。重复只费时间、不丢块：导入时同一键逐项相同就跳过，`crates/singlefs-checker-tier/src/verdict_store.rs:3423`：「StoredIdenticallyOnAnotherJudgingDevice => {」。

几格是用命令数出来的（原样）：

```
甲 无停 总格: 36
front=single 格数 18；录入段有洞的格 7；判段读到别处录的块的格 16；读到别处的块数最多 186
front=phased 格数 18；录入段有洞的格 0；判段读到别处录的块的格 18；读到别处的块数最多 127
甲 停 总格: 60；phased 停后本机判段读到第二台录的块>0 的格 29/30
```

四句：
1. 分不分辨臂：分辨。H1、H3 只打「判段也扫」的甲；乙、丙各自另有原因（1.2、1.3）。
2. 系统当时看不看得到：第二台判段开跑时，看不到「这一块的录入块在哪台库里」。它看到的只有对方的前沿，而录入相遇点在录入那一遍结束后就不再写出来。
3. 满足 K4 字面的哪一个分句：H2 满足「都没录」（录入段的洞）。H1、H3 不在 K4 字面的两个分句里：块是有人录了，但录在别的库里，判段读不到。这是 K5「判段各判各录的块这一条保不保留」那一格的事，我在这里报，主 agent 按 K5 记。
4. 最顺手的改法：判段只领自己录过的块（判段沿用录入相遇点、不再扫一遍）。H1、H3 就不中了，但判段又回到今天「各判各录」的不均（K5）。把前沿改成（第几遍，块序号），H2 不中，H1 照中。

### 1.2 乙：中央领块

许可的句子：`research/prompts/_dyn-share-r1-body.md:30`：「第二台每做完一段经 ssh 向本机领下一段」「网络断了要有重领与超时」。

**通道方向。** 今天的双机通道只有本机去问第二台：`scripts/hosts/multi-host-run.sh:24`：「每 MULTI_HOST_RUN_POLL_SECONDS（默认 5）秒」，前面说的是本机经 ssh 看第二台的进程号、退出码与输出，第二台那条命令是脱离 ssh 会话起的。「第二台经 ssh 向本机领」要有第二台连本机的凭据与路径，仓里没有这一样（推的：我只读了 multi-host-run.sh 的文件头，没读 peer-host-lib.sh）。换成本机定时把领到的段写进第二台的一份文件，领块就带着读前沿那一类的延迟，每段至少多等一个读的间隔（默认 5 秒）。

模型 `central_lease.py`：120 段，一段 64 块。一段的用时按判速折：本机 48 秒，第二台 24 秒。领一次 5 秒。收回超时取不收回、30 秒、600 秒三档。故障五种（无故障、回话丢了、第二台 TERM、第二台掉线、发块方重起），每种在第 200、1500、3000 秒三个时刻注入，账在内存或落盘。共 90 格，按格数出来的（原样）：

```
dispatcher-restart timeout=30 格 6 有洞 0 有重复 6 账说做完而取不回 0
dispatcher-restart timeout=600 格 6 有洞 0 有重复 2 账说做完而取不回 0
dispatcher-restart timeout=none 格 6 有洞 0 有重复 2 账说做完而取不回 0
none timeout=30 格 6 有洞 0 有重复 6 账说做完而取不回 0
none timeout=600 格 6 有洞 0 有重复 0 账说做完而取不回 0
none timeout=none 格 6 有洞 0 有重复 0 账说做完而取不回 0
peer-offline timeout=30 格 6 有洞 6 有重复 6 账说做完而取不回 6
peer-offline timeout=600 格 6 有洞 4 有重复 0 账说做完而取不回 4
peer-offline timeout=none 格 6 有洞 4 有重复 0 账说做完而取不回 4
peer-term timeout=30 格 6 有洞 0 有重复 6 账说做完而取不回 0
peer-term timeout=600 格 6 有洞 0 有重复 0 账说做完而取不回 0
peer-term timeout=none 格 6 有洞 4 有重复 0 账说做完而取不回 0
reply-lost timeout=30 格 6 有洞 0 有重复 6 账说做完而取不回 0
reply-lost timeout=600 格 6 有洞 0 有重复 0 账说做完而取不回 0
reply-lost timeout=none 格 6 有洞 4 有重复 0 账说做完而取不回 0
```

**历史 乙-H1（领块的回话丢了）。** 第 200 秒第二台领块，发块方在账上记了「第 k 段给第二台」，回话没到。第二台重发请求，领到下一段。没有超时的话，第 k 段在账上一直是 leased，跑完时谁都没做（`fault=reply-lost … timeout=none … holes=1 … stuck_leases=1`）。超时设 30 秒，比第二台一段 24 秒、本机一段 48 秒都没长出多少，正在做的段被收回重发，无故障也重复 44 段。超时要比最慢的一台做最慢的一段还长，而同一条流不同点上录入速度差几十倍（`records/2026-10-13-复用键与全仓改名.md:930`：「每 5 分钟只录约 13 万态（第 19 个点时 750 万）」，那时见证还没分组；分组修好之后点与点之间差多少没量）。所以一个固定的超时，要么把还在做的段收回、重复做，要么让丢了的段晚很久才重领。

**历史 乙-H2（第二台掉线）。** 第 1500 秒网线断。第二台此前做完的 51 段在账上都是 done（「做完」由下一次领块带回），结果却在第二台树里的库里，取不回来。multi-host-run 的 ⑦ 本该删掉第二台那棵树，此刻连不上，删不成也取不成。账说做完、库里没有的 51 段，发块方不会再发；要等最后导入那一趟对账为红，才知道要整条重跑。今天的静态分工掉线时，丢的是第二台那一整份；乙丢的是「它做过的那些段」，大小差不多，但乙的账会把这些段当成做完。

**历史 乙-H3（发块方重起，账在内存里）。** 本机为换判法重起（记录里 2026-10-17 停全量那一次就是这一类，`records/2026-10-13-复用键与全仓改名.md:933`：「第二台的随树作废」）。新账照本机库重建，第二台已做完、正在做的段都当 free 重发。不收回超时的那几格重复 0 到 18 段（第 3000 秒那一格活已做完，是 0）。账落盘就不重复。

四句：分辨（只打乙）；发块方看不到第二台库里的东西，只看得到请求；H1、H3 满足「领块请求丢了会怎样」「发块方挂了」两个分句；最顺手的改法（账落盘，超时取最慢一段的若干倍）修 H3，H1 只修一半（洞变成晚领），H2 修不了，要靠丁把第二台库里的东西先取回来。

### 1.3 丙：静态分工加定期重分

许可的句子：`research/prompts/_dyn-share-r1-body.md:31`：「引擎不改，只改驱动；代价是每次重分第二台要重起一趟（拷树、编、起）」。

代码事实：引擎开跑时按环境里的权重算一次这台机器的块，之后录入、判段、导入都用这一份：`crates/singlefs-checker-tier/src/crash_amplification.rs:12596`：「let my_blocks: Vec<u64> = (0..total_blocks)」，`crates/singlefs-checker-tier/src/crash_amplification.rs:12599`：「let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();」。引擎不改的话，本机那一趟跑着时换不了分工。库开着时也拷不了快照：`scripts/crash/crash-amplification-two-hosts.sh:1559`：「库开着的时候不能拷（RocksDB 的文件在变）」。

模型 `periodic_reshare.py`：22365 块，开头权重 2,3。用 `owner_of_block` 同一算法核过，分出来本机 8946、第二台 13419，与正文的数相同（输出 `name=bing_split_check weights=2,3 local_blocks=8946 peer_blocks=13419`）。块速按真在干活的秒数量。扫的旋钮：本机速度是第二台的 1 或 2 倍；只重起第二台还是两台都停；新权重算不算重起的秒数；检查间隔 600 或 1800 秒；阈值 120 或 600 秒；重起 300 或 1200 秒；后 30% 的块用时 1 倍或 10 倍。共 128 格。按格数出来的（原样）：

```
丙 peer-only 格 64；有洞 64；洞最少 511 最多 9063；重起最多 55；重复最少 2619
丙 both 格 64；有洞 0；有重复 0；重起最多 3
```

**历史 丙-H1（照字面：只重起第二台）。** 第 600 秒驱动量出两台差得多，给第二台建停下文件。第二台判完手里那一块、取回，换新权重重起。本机那一趟仍按 2,3 领块。某一块 k，若旧权重下归第二台、新权重下归本机：第二台重起后不领它（新权重下不归它），本机也不领（本机还用旧权重，它不归本机）。这样的块这一趟谁都不做；反过来的那一类两台都做。两台一样快、间隔 600 秒、阈值 120 秒、重起 300 秒、用时 1 倍那一格的输出：

```
name=bing_reshare local_speed_factor=1.0 policy=peer-only rate_policy=naive interval=600 threshold=120 restart=300 hard_factor=1.0 makespan=15669 restarts=14 duplicates=4468 holes_at_the_end_of_the_run=6410 local_blocks=8946 peer_blocks=11483
```

不重分的静态那一格 13419 秒、零洞（`name=bing_static local_speed_factor=1.0 hard_factor=1.0 makespan=13419`）。照字面做的丙更慢，还留 6410 块洞，要再跑一趟才补得上。

**历史 丙-H2（两台都停）。** 要不留洞，每次重分本机也得停下来：放掉库的锁、导入第二台取回的库、照新快照，两台一起重起。这样 64 格都零洞零重复，重起最多 3 次。可这已经不是「代价是第二台重起一趟」：本机每次也付一趟重起（开库、读回流程、起卡编内核，外加导入）。正文代价那一句少算了本机那一半。

**手里那一块。** 停下文件让第二台判完手里那一块、写进库再收尾（`crates/singlefs-checker-tier/src/crash_amplification.rs:2033`：「文件在，就不再领新的块（手里那一块判完、写进库），这一趟提早收尾。」）。这一块在取回的库里，按旧权重导入时照键认，不会两台都算、也不会丢。这一小格没打中。录入段里叫停也停在块边上（`crates/singlefs-checker-tier/src/crash_amplification.rs:17485`：「每录一块之前问一次 `stop_is_requested`」），但录了没判的块要靠丁才不丢（正文已写）。

**重起次数的上界。** 只重起第二台时，每重起一次第二台少干 R 秒，下一次检查看见的差又是 R 那么大；阈值比这个差小，就一直触发，上界只剩「检查次数 = 总用时 / 间隔」，模型里最多 55 次。两台都停时两边少干的一样多，差不再自己长回来，模型里最多 3 次。

四句：分辨（H1 只打丙的字面）；驱动看得到两台进度，看不到本机引擎里那份定死的 `selected`；满足「重起的次数上界」那个分句（H1 的上界只有检查次数），洞不在 K4 字面的分句里，算另报；最顺手的改法「两台都停」修 H1，代价是每次重分本机也重起一趟。

## 二、K5 判段的闲置

K5 的字面，`research/prompts/_dyn-share-r1-body.md:42`：「判段各判各录的块这一条保不保留；保留时判段的不均怎么消」「不保留时判别人录的块要把录入块搬过去，搬的量」。

**2.1 打中，不分辨臂：卡在录入段里全空，甲乙丙一样。** 同一趟里先跑观察者录入段（CPU，逐状态挂载），全录完才起镜像判定（GPU）与观察者结论（引文见 1.1）。甲乙丙都只在一段之内重分，没有一个让 GPU 判段与 CPU 录入段同时跑。此刻的样子（量的，host-resources 钩子这一轮打出的原文，主机名本来就是「第二台」，没换字）：

```
第二台：
  线程：32 核，此刻在用约 29.6 核（loadavg 30.57 30.73 30.6），可用 2 线程
  卡 0 NVIDIA GeForce RTX 5080：总 16303 MiB，空闲 15841 MiB；没有清场服务占着；别的计算进程：没有；可用 不清场 13281 MiB / 清场 13281 MiB
```

本机两张卡同一时刻也空着（钩子同一次输出：5090 空闲 32110 MiB、5060 Ti 空闲 15849 MiB）。记录里同一件事：`records/2026-10-13-复用键与全仓改名.md:951`：「负载 31.0、内存 6G、三张卡空」。

拿第 6 条的量粗算（推的）。录入：every_state 每块 9611/13402≈0.72 秒，mount_continuation 去重之后每块约 1.0 秒，22365 块单台合计约 22365×1.72≈38500 秒；两台 CPU 一样快、甲乙丙把录入分得再匀，录入段也要约 19200 秒（5.3 小时），这期间五张卡全空。判段：1463812278 个态按两台合计 26.3 万态每秒，约 5570 秒（1.5 小时）。录入段与判段串着跑，整趟约 6.8 小时里卡空着的约占 78%，甲乙丙改不了这一项。判段之内的不均（正文 `:22` 那几条、本机那 663 秒）比这一项小一个量级。

判据形态（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）：这是「打中不分辨臂」，病根在三个候选共用的「一趟之内录入段先跑完、判段后起」。不该拿它判甲乙丙谁好，应另立一笔账。

**2.2 「各判各录」只管观察者结论，不管镜像判定。** 镜像判定读的是录入键下整条录的流程、事实表与判表（`scripts/crash/crash-amplification-two-hosts.sh:18`：「照旧整条录（几十秒，录入键不含份）」，两台都有），不读观察者录入块；只有观察者结论读录入块与它引的镜像（`crates/singlefs-checker-tier/src/crash_amplification.rs:18700`：「的录入块引的恢复镜像 {} 不在库里：先跑录入段」）。所以 K5 第一问可以拆开答：镜像判定那一部分（第 6 条 14.6 亿态）不搬任何东西就能动态分，只有观察者结论要「各判各录」，或者搬。正文 K5 把这两样当一件事问了。

**2.3 搬的量（推的）。** 观察者录入块的写法：每个状态一个 u32 下标（`ObserverBlockRecord::to_bytes` 逐状态写 `index.to_le_bytes()`），一块 65536 态就是 256 KiB 下标，外加去重之后的观察结果清单（交回镜像的观察者每份 32 字节键）。第二台那一份 13419 块、两个观察者，未压缩的下标约 13419×2×256 KiB≈6.6 GiB，此外还有它引到的看过的镜像与基底（镜像数要按 `images_observed_fresh` 数：本机重起后那 203 块是 1184 张，`records/2026-10-13-复用键与全仓改名.md:941`：「images_observed_fresh=1184」）。RocksDB 压了多少没量（下标在去重好的段里几乎全是 0，压得很小是可能的）。整份库此刻 5.1G（同一份记录那一行）。所以「搬的量」上界是几个 GiB 未压缩，盘上的实际数要读库才知道，这一轮禁读那份库，没量。

**2.4 判段之内的不均，乙的段长两头受压（推的）。** 5090 一派的用时不随态数涨（`records/2026-10-13-复用键与全仓改名.md:945`：「一派的用时几乎不随态数涨」）。照乙那样按段领，一段要大到摊得掉领块的往返（经第二台那一侧最少 5 秒，1.2 节）；段越大，收尾时空等越久，最多一段的用时。按本机判速一块约 0.75 秒，段长 Z 块时收尾最多空等 0.75Z 秒，领块往返占 5/(0.75Z)：Z=64 时空等 48 秒、往返占 10%。两头得一起取，正文没给段长。

四句：2.1 不分辨臂；系统（驱动）看得到「录入段还剩多少」，但引擎一趟之内没有并行这一档，看到了也用不上；满足 K5「保留时判段的不均怎么消」：不均的大头不在判段里，在录入段串着判段；正文三个候选没有一个改法在 2.1 上起作用。

## 三、K6 改动量与风险

K6 的字面，`research/prompts/_dyn-share-r1-body.md:43`：「改录入段驱动不该进判器闭包，攻方举反例」。

**3.1 量过：哪些改动会让键换。** 派发提示许可的 `target/debug/unit-graph closure|key`（经内存包装）跑了两处：主工作区；草稿目录里从主工作区拷出的 crates 副本 copy-base 与 8 份变体，每份只改一处，见 `tools/make_variants.py`。四组 lib 目标里的入口按 `crates/singlefs-checker-tier/src/judging_units_entries.rs` 抄。copy-base 那一组观察者结论拼块的键 `5dbb6d3e…`，与主工作区直接跑出来的相同。键是否改变（原样，`out/compare_keys.out` 去掉键值那一列）：

| 改了哪一处（模拟哪个候选会碰的） | 镜像判定 | GPU 判器 | 录入 | 观察者结论拼块 |
|---|---|---|---|---|
| 观察者录入段循环里多一个判断（甲乙丙把 `is_mine` 换成动态的） | 相同 | 相同 | 相同 | 相同 |
| `WeightedShares::owner_of_block` 换一种取模（丙改权重） | 相同 | 相同 | 相同 | 相同 |
| `import_the_planned_blocks_of_the_other_hosts` 多一个条件（甲乙丙改导入范围） | 相同 | 相同 | 相同 | 相同 |
| `verdict_store.rs` 的按计划导入多一个条件（丁并库） | 相同 | 相同 | 相同 | 相同 |
| 第 94 行那行 `use` 多引一个名字（甲把叫停换成「叫停或对方前沿越过」） | 相同 | 相同 | 相同 | **变了** |
| `checking_threads` 函数体多一行（K5 让录入与判段分线程） | 相同 | 相同 | 相同 | **变了** |
| `.claude/heavy-thread-default` 的数 31 改 30 | 相同 | 相同 | 相同 | **变了** |
| 同一份文件只改理由那一句（数不变，句末多一个句号） | 相同 | 相同 | 相同 | **变了** |

正文担心的判器两组（镜像判定、GPU 判器）8 份变体全都没变：判器闭包里没有一个单元落在 `crash_amplification.rs`（闭包清单里这份文件的单元数是 0，这一点由 `crates/singlefs-checker-tier/src/judging_units_entries.rs:258`：「录入与观察者结论拼块两组本来就在流水线里，不受这一条管。」那张表的测试守着）。反例在第四组，观察者结论拼块。

**3.2 反例：改录入段驱动会让观察者结论全部作废。** 观察者结论那一把版本里拼着这一组的键：`crates/singlefs-checker-tier/src/crash_amplification.rs:16017`：「hexadecimal_text(composition_units.fingerprint())」。这一组的闭包（3761 个单元）里有 `crash_amplification.rs` 的 83 个单元，包括：
- 第 94 行的 `use`：`crates/singlefs-checker-tier/src/crash_amplification.rs:94`：「use singlefs_harness::gpu_memory_ledger::{CardMemoryQueue, StopRequestRememberedInThisRun};」。闭包要的只是 `CardMemoryQueue`，但一行 `use` 整行是一个单元，叫停用的类型和它同一行。甲（停在对方前沿）、丙（叫停）、乙（领块用完就停）只要在这一行里多引、少引一个名字，就算碰到。
- `checking_threads`（`crates/singlefs-checker-tier/src/crash_amplification.rs:1885`：「pub fn checking_threads() -> usize」）。录入段用的也是它：`crates/singlefs-checker-tier/src/crash_amplification.rs:17516`：「let threads = checking_threads();」。2.1 那条「录入与判段同时跑、分线程」最顺手的改法就是改它。
- 卡的显存排队那几个函数（`memory_queue_of_the_cards_started_for_this_run`、`judge_on_a_card_started_for_this_run_queued_by_memory`），K5「让 5090 多派几路」会碰到它们。
- 线程登记文件整份当一个数据单元：`crates/singlefs-harness/src/heavy_threads.rs:25`：「"../../.claude/heavy-thread-default"」。

后果：观察者录入块的版本不含这一组，录入块照旧能用；但观察者结论那一把换了，库里每条流的每个观察者结论都要重判。第 6 条单是第二台那一份 every_state 就是 878313468 态，记录里估「按每秒 8 万态算约 3 小时」（`records/2026-10-13-复用键与全仓改名.md:951`，那是推的）。

**3.3 线程数进了键，与 D13 已定项 17 冲突（这条与候选无关，单列）。** 已定项 17 写：`.claude/kb/decisions/13-验证路线.md:428`：「路径（仓根也算）、名字、种子串、行号、资源上限、限时」一律不进键。还有 `.claude/kb/decisions/13-验证路线.md:433`：「显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键」。量到的是：重活线程数这个资源上限（连同它的理由那一句）进了观察者结论拼块的键；卡的排队函数也在同一个闭包里。用户哪天把每台默认线程数改了（2026-10-12 定的就是这个数），全库观察者结论作废。这算「打中不分辨臂」，是甲乙丙丁共用的前提，交主 agent 另立一笔账（推的病根：`checking_threads` 读线程数的那一路被 `judge_the_images_after_the_observers_on_the_cpu` 引到了）。

**3.4 各候选动到哪（推的，按代码推）。**

| 候选 | 引擎要改的 | 会不会碰 3.2 那几样 |
|---|---|---|
| 甲 | `selected` 换成读前沿的谓词（录入、判段、导入三处）；读写前沿；叫停条件 | 叫停条件很可能碰第 94 行 `use`；分线程碰 `checking_threads` |
| 乙 | 同上，再加「向外领段」；判段 GPU 派活按段 | 派活按段大概率碰卡的排队函数 |
| 丙 | 字面上不改；照 1.3 要不留洞，本机也得重起，仍不改引擎 | 不碰；本机每次重起是驱动里的代价 |
| 丁 | 导入加观察者录入块、镜像、见证、基底四类行 | 量过：只改 `verdict_store.rs` 的导入不碰。拼块闭包清单里有 `ObserverFoundation`，没有 `ObserverBlockRecord`、`ObservedImageRecord`（清单里查过）：并库时要是改基底的布局就会碰，改录入块、镜像两种布局不碰 |

四句：分辨，只打会碰叫停与线程的改法（甲乙更容易碰到）；系统（键）当时看得到一行 `use` 变了，可它分不出那是驱动改动还是判法改动；满足 K6「动到的函数单元会不会落进判器组闭包让键全换」：判器组本身没落进去，落进的是观察者结论拼块那一组，效果同样是一类结论全部重判；正文给的判据「改录入段驱动不该进判器闭包」对判器两组成立，对拼块那一组不成立。

## 四、丁：录入块跨主机断点

许可的句子：`research/prompts/_dyn-share-r1-body.md:32`：「第二台录的观察者块（连同它桶里的见证）取回时并进本机库，或第二台的库不随树删、下一趟同一条命令送回去接着录」。

代码事实：
- 看过的镜像连同它所在桶的整份见证清单一批落盘，这是一条不变量：`crates/singlefs-checker-tier/src/verdict_store.rs:2851`：「见证在、它引的镜像必在，被打断后重跑找到见证就读得到镜像」。
- 录入时命中见证就直接取见证的镜像键，不读那张镜像：`crates/singlefs-checker-tier/src/crash_amplification.rs:17355`：「let deduplication_key = if let Some(key) = matched_witness {」。
- 镜像键由桶键与见证摘要拼成：`crates/singlefs-checker-tier/src/crash_amplification.rs:16566`：「一个见证看出的那张镜像在库里的键：桶键接见证摘要。」
- 今天的按计划导入对路径、核对行、事实行的规矩是同键不同就照写：`crates/singlefs-checker-tier/src/verdict_store.rs:3351`：「路径、核对行与事实行本库已有且逐字节相同的跳过」，后面接「不同的照写」。

**历史 丁-H1（照字面并「观察者块连同见证」，第二台在一块的半中间被停）。** 第二台从尾往前录 mount_continuation，录到第 12 块的第 k 个状态时被 TERM（或叫停之前的最后一块）。第 12 块里新看出的几张镜像已经连同整份见证清单落了它的库，第 12 块的录入块还没写。取回并进本机：并了 13–19 块的录入块与见证清单，镜像不并（listed），或者只并 13–19 块引到的镜像（planned，照今天「只读点名的键」的做法）。见证清单里那几条第 12 块看出的见证，引的镜像不在本机库。本机接着录第 10–12 块（它们归本机或没人录过）时，有状态命中那几条见证，就直接写下那个镜像键；判段读到它，报「…的录入块引的恢复镜像 … 不在库里：先跑录入段」。重跑录入段也补不上：那一块已经算录过，`observer_block_is_recorded_or_concluded` 为真就跳过。

模型 `library_merge.py`：20 块，每块 16 个状态，第二台停在第 12 块第 1–15 个状态（全扫），40 个种子；状态的字节取 3 种值（见证很快录全）与 8 种值（一直有新见证，接近记录里的「images_observed_fresh=1184、witness_misses=1184」）。原样：

```
name=ding_merge way=listed alphabet=3 runs=600 runs_with_witnesses_whose_image_is_missing=15 runs_where_the_judge_stage_errors=15 most_erroring_blocks_of_twenty=4
name=ding_merge way=listed alphabet=8 runs=600 runs_with_witnesses_whose_image_is_missing=600 runs_where_the_judge_stage_errors=600 most_erroring_blocks_of_twenty=10
name=ding_merge way=planned alphabet=3 runs=600 runs_with_witnesses_whose_image_is_missing=0 runs_where_the_judge_stage_errors=0 most_erroring_blocks_of_twenty=0
name=ding_merge way=planned alphabet=8 runs=600 runs_with_witnesses_whose_image_is_missing=557 runs_where_the_judge_stage_errors=557 most_erroring_blocks_of_twenty=3
name=ding_merge way=union alphabet=3 runs=600 runs_with_witnesses_whose_image_is_missing=0 runs_where_the_judge_stage_errors=0 most_erroring_blocks_of_twenty=0
name=ding_merge way=union alphabet=8 runs=600 runs_with_witnesses_whose_image_is_missing=0 runs_where_the_judge_stage_errors=0 most_erroring_blocks_of_twenty=0
name=ding_double_record samples=40 block_bytes_differ=0 observations_differ=0
```

见证一直在长的时候（第 6 条 mount_continuation 正是这样），planned 600 格里 557 格判段报错。见证清单两边取并集、取回的镜像全并（union），一格都不报。另外照「不同的照写」并见证清单会盖掉本机自己的见证：不出错，但本机之后多看很多遍（模型没量这一项）。

**撤回我自己的一处猜测。** 我原先以为同一块两台都录（甲交叉处、丙重复的那几块），录入块的字节会因见证历史不同而不同，按字节对账就会误报冲突。模型 40 个样本里字节全相同。道理是：见证可靠的前提下，命中见证的状态读的段就是见证那几段，镜像键对一个状态是唯一的。这条不报。

**历史 丁-H2（第二台的库不随树删）。** 与规则直接冲突：`.claude/rules/implementation-workflow.md:66`：「第二台那一趟的树连同库跑完就删，不留长期库」。与 D13 已定项 17 也冲突：`.claude/kb/decisions/13-验证路线.md:430`：「第二台判出的并进本机那一份」，同一条还有「两份库重复判同一批，同样算违反」（`.claude/kb/decisions/13-验证路线.md:428`）。下一趟送回去时，第二台上有两份库（留着的那一份与本机送的快照），引擎只开一份，`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 只收一个目录；不并，第二台就看不到本机那一份的进度。并，就是 H1 的同一个问题换个方向。「送回去」那一种读法里，取回的那一份是本机这边的拷贝，可掉线时根本取不回来：`scripts/hosts/multi-host-run.sh:37`：「第二台的停下之后先取回 --fetch 点名的路径」，前提是还连得上。

四句：分辨（只打丁的并法）；本机在并库那一刻看得到「见证引的镜像在不在」，今天的导入不核这一项；满足 K3「停在录入段中途、第二台掉线」那两种情形（K3 归正推，这里只给反例）；最顺手的改法「并见证时连见证引的每张镜像一起并、缺一张就不并那条见证」在 H1 上不中（模型 union 那两行是它的近似），对 H2 不起作用。

## 五、我自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲的前沿写成（第几遍，块序号），判段沿用录入相遇点、不再扫一遍 | 甲-H2 量过不中（front=phased 那 18 格录入段零洞）；甲-H1、H3 推的不中，代价是判段回到各判各录 | 一半量过、一半推的 |
| 丙每次重分两台都停、本机先导入再照快照 | 丙-H1 | 量过（policy=both 那 64 格零洞零重复） |
| 丁并见证清单取并集，连同清单里每条见证引的镜像一起并 | 丁-H1 | 量过（way=union 那两行） |
| 一趟之内镜像判定（GPU）与观察者录入段（CPU）同时跑，观察者结论最后判 | 2.1 | 推的，没实现，也没量；它要改 `checking_threads`，照 3.2 会换掉观察者结论拼块的键，要先修 3.3 |
| 把 `checking_threads` 与线程登记文件移出观察者结论拼块的闭包 | 3.3 | 推的，没实现 |

## 没打中的形状

- 丙「重分那一刻第二台手里那一块算谁的」：停下文件、导入、再起，手里那一块不丢也不重判（1.3）。扫过的是模型里「判完手里那一块再停」这一种；TERM 直接打断的那一种归甲-H3 与丁-H1。
- 丙两台都停的读法：64 格零洞零重复，重起最多 3 次。扫的范围：本机速度 1、2 倍，间隔 600、1800 秒，阈值 120、600 秒，重起 300、1200 秒，后 30% 用时 1、10 倍，权重算不算重起。
- 乙账落盘、超时取 600 秒（长过最慢一段）：第二台 TERM、回话丢了、发块方重起三种都零洞零重复；掉线照样留洞（见乙-H2）。
- 同一块两台都录、按字节对账：40 个样本字节全同，不误报（第四节撤回的那一条）。
- 改录入段循环、平滑轮转、两处导入：四组键都不变（3.1 表里前四行）。
- 甲两台都录：只有重复，没有冲突（导入照「逐项相同」跳过）。重复的块数随读前沿的间隔涨，不随别的旋钮涨。

## 这条腿自己的限度

- 四个时序模型都是我写的，没被别人攻过：块用时只取量到的几个速度，本机录入速度按倍数扫；模型没有显存、卡数、派活往返，没有 RocksDB 压缩与写放大，也没有 ssh 失败重试的细节。
- 甲模型每遍 400 块，不是 22365 块；结论只看「有没有洞、判段有没有读到别处录的块」，块数只当相对量。丙模型用全量块数。丁模型 20 块、每块 16 个状态，读集只有两种形状。
- 3.1 的键是用主工作区编好的 `target/debug/unit-graph` 在副本上算的。我没核这份二进制与 `build.rs` 编进库的那一把是不是逐位同一个算法：副本上的 copy-base 键等于主工作区，变体只比「变没变」。拼块那一组的键是否真的拼进观察者结论版本，是读代码读出来的（3.2 那一行引文），没跑引擎。
- 观察者入口在测试目标里（`test:crash_enumeration_unlink_after_the_multi_record_publish_stream` 等），照派发提示只用 `--target lib` 跑：`singlefs_checker_tier::observe_every_state` 与 `singlefs_checker_tier::common_crash_point_observers::observe_the_mount_continuation` 两个入口 unit-graph 报「在点名的目标里找不到入口」，退 2。观察者那一把版本的闭包会不会碰到驱动改动，没量。
- 2.1 的 78%、2.3 的 6.6 GiB 都是推的；判定库这一轮禁读，搬的量盘上实际多大没量。
- 正文写的代码行号与此刻主工作区对不上（例如正文 `:16819` 起的 `record_one_observer_piece`，此刻在 `crates/singlefs-checker-tier/src/crash_amplification.rs:17195`：「fn record_one_observer_piece(」）。报告里的行号都是此刻 grep 出来的。
- `peer-host-lib.sh` 没读；乙「第二台连不上本机」是按 multi-host-run.sh 文件头推的。

## 没做什么

- 不判 K1、K2、K3（正推那几格）；丁-H1 落在 K3 的情形里，只给反例，不判 K3。
- 没起 cargo，没跑任何测试；没读 `~/.local/share/singlefs/crash-amplification/` 下的库，也没读禁读清单里的文件。
- 副本上量的键不算入库装置上的数。要引 3.1 的表，得由主 agent 在入库的装置上另跑一次。
- 草稿目录 `~/.local/share/singlefs/work/dyn-share-r1-attack`：交回前删掉了副本 `copy-base`、8 份 `variant-*` 与复跑核对的 `rerun-check`；留着 `closure/`（四组闭包清单、两个观察者入口的报错）、`tools/`（变体脚本与比键脚本，已拷进模型目录）、`compare_keys.out`、`k.out`/`k.err`，都是文本，主 agent 看完可删。
