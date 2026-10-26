# m4-gpugc-design-r1 攻方腿（Opus）报告（2026-10-23）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 Q5 Q6 Q7 Q9 Q11 Q13 Q14 Q16 Q17 F7 -->

立场：假设正文第三节每格的推荐是错的，找最脆弱的地方。正推腿的报告在禁读清单里，我没读；下面攻的是背景材料、kb 条款与 `crates/` 今天的实现能推出的推荐形态。行号都在主工作区现查（开工快照 `research/prompts/m4-gpugc-design-r1-snapshot/sha256sums.txt` 里的六份 `crates/` 文件与材料，`sha256sum -c` 全部 OK）。

## 复跑

模型全是纯 python，只读 E185（GC一批搬迁的挂钟拆分） 的三份产物（sha256 与实验页登记的相同），不编译、不起 GPU。这一轮没有冻结副本，`--frozen` 给仓根即可（脚本只读 `$REPO_ROOT/research/results/`，先核三份产物的 sha256）。

```bash
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-gpugc-design-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新的空目录>
```

我这一次的原样输出（`--work` 是草稿目录里的 `rerun1`）：

```text
name=rerun_compare model=g1 identical=true
name=rerun_compare model=g2 identical=true
name=rerun_compare model=g4 identical=true
name=rerun_compare model=g6 identical=true
exit=0
```

模型目录每个文件的 sha256：

```text
fd4bda713200748ff10a40c72a103d2269f88d2bfdb39882ebc7069fedb2fd7b  research/prompts/m4-gpugc-design-r1-opus-model/g1_cpu_saved_ledger.py
f5ed80d832fbcaec45c6e7d41596848e69cc4f6d7dae72636721f6a38061cea6  research/prompts/m4-gpugc-design-r1-opus-model/g2_merge_bounds.py
ceb30de9e90252ac29d56c3169eea9710f0614339e74353a658fc47587b4f0b4  research/prompts/m4-gpugc-design-r1-opus-model/g4_shared_input_table.py
73a11bd302f67456eeb02e1ddb547ba7c92d07b7cc8bde36ce5f613a9416dbc7  research/prompts/m4-gpugc-design-r1-opus-model/g6_inline_batch_latency.py
5af6f96e918763a82fb7f66be9c7b67d7798a09a61ae25d0a75c08aee49678a4  research/prompts/m4-gpugc-design-r1-opus-model/tools/rerun.sh
61b210296634f2f20fa0465a4e4f09859cd9236522c54d4fb18301e75f243025  research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out
15d0c488cbb60634dd5bcc5fad26cfd253f1cc6c150a6300f602f1ddbd77b3aa  research/prompts/m4-gpugc-design-r1-opus-model/out/g2.out
013251c1baf86b7bdeffdcf5df4067599363649f1189649e10bb13a7801c23df  research/prompts/m4-gpugc-design-r1-opus-model/out/g4.out
8d3a5271ec3912100dd21d199a76ee3074b5dc1916e8f7934c74264c99ce177e  research/prompts/m4-gpugc-design-r1-opus-model/out/g6.out
```

读的产物：第二段 `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out`（`f70eb21b…`）、第一段 `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out`（`18191ec3…`）、第 1 次重跑第三段 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`（`4041a1ca…`）。

## 各格判定一览

| 格 | 判定 | 最脆弱的一处（一句话） | 证据形态 |
|---|---|---|---|
| G1 | **打中三处** | ① 每批之前都重扫 R，所以每批至少一次 GPU 提交；甲＋乙＋丙一条意图能省的主机 CPU 摊到每次提交只有 0.216–0.721 ms，只是卡 0 提交固定挂钟 4.492 ms 的 4.8%–16.1%、卡 1 10.300 ms 的 2.1%–7.0%，而提交时主机线程花多少 CPU 从来没量过；② 「发布那一半」里唯一像样的计算（释放前读盘核）住在所有发布共用的那一个准备函数里；③ 今天的触发在 `publish_user_change` 返回之前跑一批，GPU 往返就进了前台路径 | 算术（模型 g1，量的产物）＋代码行 |
| G2 | **打中：乙、丙两条都撞已定条款** | 乙要跨批复用结果，撞「每批之前都重扫」「全池至多一条活意图」与一趟至多 20 次发布；丙要改意图条目里「R 槽数恒 64」那个格式字段。攒到 φ ≤ 0.5 要 7.8–41.6 条意图的字节，P = 1024 时整池数据（33554432 字节）都凑不到 φ ≤ 0.1 要的 73–98 MB | 算术（模型 g2）＋代码行 |
| G3 | 找到脆弱点（推的，没起 GPU） | 「不给就用本机全部显卡」：本机装着软件 Vulkan（`lvp_icd.json`），不按类型筛就会把 CPU 上跑的适配器当一张卡；一张卡时两份核共用同一块显存里的输入，显存翻位两份一起错、判「确认」，两张卡时同一个故障判「不确定」，判定不同 | 文件观测＋推的 |
| G4 | **打中两处，另一处脆弱** | ① 已定项 13 射程说「输入表的准备」这一类靠测试里 GPU 对 CPU 的对拍抓，而七·3 把对拍写成「在同一组输入上」——输入表两边共用，抓不到（模型 g4：留存条目漏掉 3 条时两份核互比 0 处不同、同输入对拍 0 处不同，两种模式各读这一版再比 3 处不同）；② 「CPU 只改位图，不再抽样复核」与今天的发布路径逐条对版本核计划相左；③ N 的计数住内存而意图与 R 排他住盘上 | 模型 g4（带两条阳性对照）＋代码行 |
| G5 | 找到脆弱点 | 今天开关在一趟开头读一次，「下一批」在今天的单线程会话里就是「下一趟」；GC 挪出调用线程之后边界要落到每次推一次发布的开头 | 代码行 |
| G6 | **打中：「前台 fsync 在等」这个判别子今天观测不到；写带宽上限压不住让路** | 一批的挂钟随 P 涨（内存臂 12.5 → 102–111 ms，真盘 21.7–25.4 → 114–123 ms），CPU 占挂钟 0.55–1.0，一批只写 0.74–1.38 MB；500 MB/s 的上限只把写摊成 1.5–2.8 ms，碰不到一批的长度 | 算术（模型 g6，量的产物）＋代码行 |
| G7 | 找到脆弱点 | 「有更好的做法」按第 7 条该比每次提交的主机 CPU，不是着色器语言；WGSL 没有 u64、没有字节寻址，两份核都要自己拼，是同一类共同前提 | 推的 |

「打中」只是线索，按规则由主 agent 现查；下面每处都写了四句（分不分辨臂、判别子看不看得到、满足哪一个分句、改法还中不中）。

## G1　GPU 上算哪几段

### G1-①　甲、乙、丙的任何子集，每批一次提交时第 7 条那把尺上余量只有 0.2–0.7 ms

尺子是 `.claude/kb/decisions/26-后台整理与放置回收.md:242`：「挂钟与 CPU 模式持平也做」，同一行把 GPU 模式的 CPU 时间写成「含准备数据、提交、等结果、两份结果的比对」。

每批的提交次数被今天的代码定死：`crates/singlefs-core/src/mounted_session.rs:2975`：「对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）」。一批的输入（这一版的分配记录、映射、隔离集）要等上一批发布之后才有，所以 GPU 的活至少每批交一次，签发那一次（丙段选段）另一次。

模型 g1 逐条意图把第二段产物里的 `crc_cpu_nanoseconds`（甲）、`scan_increment_cpu_nanoseconds`（乙增量）与签发那一行的 `choose_cpu_nanoseconds`（丙）全算成「GPU 拿走、主机一点不花」，得到能省的上界，再除以一条意图的提交次数（批数 + 1）：

```text
$ python3 -I g1_cpu_saved_ledger.py <第二段产物> <第三段产物>
name=g1_ledger_header columns=f,P,K,intent_cpu_ms_median,saved_upper_ms_median,saved_share_median,saved_share_max,submissions_per_intent,break_even_host_cpu_per_submission_ms,break_even_over_a_card0,break_even_over_a_card1,release_check_crc_ms_not_offloadable
name=g1_ledger f=0.25 P=1024 K=4 intent_cpu_ms=34.982 saved_upper_ms=0.648 saved_share=0.0184 saved_share_max=0.0187 submissions_per_intent=3 break_even_host_cpu_per_submission_ms=0.2159 break_even_over_a_card0=0.0481 break_even_over_a_card1=0.0210 release_check_crc_ms=0.166
name=g1_ledger f=0.25 P=1024 K=8 intent_cpu_ms=23.081 saved_upper_ms=0.596 saved_share=0.0258 saved_share_max=0.0263 submissions_per_intent=2 break_even_host_cpu_per_submission_ms=0.2979 break_even_over_a_card0=0.0663 break_even_over_a_card1=0.0289 release_check_crc_ms=0.165
name=g1_ledger f=0.25 P=1024 K=16 intent_cpu_ms=22.875 saved_upper_ms=0.591 saved_share=0.0261 saved_share_max=0.0262 submissions_per_intent=2 break_even_host_cpu_per_submission_ms=0.2953 break_even_over_a_card0=0.0657 break_even_over_a_card1=0.0287 release_check_crc_ms=0.165
name=g1_ledger f=0.25 P=8192 K=4 intent_cpu_ms=284.963 saved_upper_ms=1.064 saved_share=0.0037 saved_share_max=0.0040 submissions_per_intent=3 break_even_host_cpu_per_submission_ms=0.3548 break_even_over_a_card0=0.0790 break_even_over_a_card1=0.0344 release_check_crc_ms=0.166
name=g1_ledger f=0.25 P=8192 K=8 intent_cpu_ms=185.196 saved_upper_ms=0.937 saved_share=0.0051 saved_share_max=0.0051 submissions_per_intent=2 break_even_host_cpu_per_submission_ms=0.4686 break_even_over_a_card0=0.1043 break_even_over_a_card1=0.0455 release_check_crc_ms=0.165
name=g1_ledger f=0.25 P=8192 K=16 intent_cpu_ms=185.505 saved_upper_ms=0.938 saved_share=0.0051 saved_share_max=0.0052 submissions_per_intent=2 break_even_host_cpu_per_submission_ms=0.4691 break_even_over_a_card0=0.1044 break_even_over_a_card1=0.0455 release_check_crc_ms=0.165
name=g1_ledger f=0.75 P=1024 K=4 intent_cpu_ms=84.599 saved_upper_ms=1.846 saved_share=0.0216 saved_share_max=0.0220 submissions_per_intent=7 break_even_host_cpu_per_submission_ms=0.2637 break_even_over_a_card0=0.0587 break_even_over_a_card1=0.0256 release_check_crc_ms=0.495
name=g1_ledger f=0.75 P=1024 K=8 intent_cpu_ms=48.779 saved_upper_ms=1.352 saved_share=0.0278 saved_share_max=0.0284 submissions_per_intent=4 break_even_host_cpu_per_submission_ms=0.3380 break_even_over_a_card0=0.0753 break_even_over_a_card1=0.0328 release_check_crc_ms=0.495
name=g1_ledger f=0.75 P=1024 K=16 intent_cpu_ms=36.258 saved_upper_ms=1.129 saved_share=0.0312 saved_share_max=0.0318 submissions_per_intent=3 break_even_host_cpu_per_submission_ms=0.3765 break_even_over_a_card0=0.0838 break_even_over_a_card1=0.0366 release_check_crc_ms=0.495
name=g1_ledger f=0.75 P=8192 K=4 intent_cpu_ms=694.101 saved_upper_ms=3.961 saved_share=0.0057 saved_share_max=0.0059 submissions_per_intent=7 break_even_host_cpu_per_submission_ms=0.5659 break_even_over_a_card0=0.1260 break_even_over_a_card1=0.0549 release_check_crc_ms=0.496
name=g1_ledger f=0.75 P=8192 K=8 intent_cpu_ms=393.457 saved_upper_ms=2.752 saved_share=0.0070 saved_share_max=0.0074 submissions_per_intent=4 break_even_host_cpu_per_submission_ms=0.6880 break_even_over_a_card0=0.1532 break_even_over_a_card1=0.0668 release_check_crc_ms=0.495
name=g1_ledger f=0.75 P=8192 K=16 intent_cpu_ms=296.311 saved_upper_ms=2.164 saved_share=0.0073 saved_share_max=0.0076 submissions_per_intent=3 break_even_host_cpu_per_submission_ms=0.7214 break_even_over_a_card0=0.1606 break_even_over_a_card1=0.0700 release_check_crc_ms=0.495
name=g1_ledger_summary points=12 break_even_ms_min=0.2159 break_even_ms_max=0.7214 a_card0_ms=4.492 a_card1_ms=10.300
```

读法：

- 能省的上界占一条意图 CPU 的 0.37%–3.12%（中位），绝对值 0.591–3.961 ms；摊到每次提交 0.216–0.721 ms（`break_even_ms_min/max`）。这是 GPU 模式每次提交在主机上「准备 + 提交 + 等 + 读回 + 两份比对」合起来的上限，超过就按第 7 条赔。
- 提交的固定挂钟是 `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out:54`：「intercept_seconds=0.004492」（卡 0）与第 136 行「intercept_seconds=0.010300」（卡 1）。门槛只是它的 4.8%–16.1%（卡 0）、2.1%–7.0%（卡 1）：等的那段时间里主机线程只要有一成在转，GPU 模式就赔。
- 这段时间里主机花多少 CPU，E185 没量：第三段 32 行 `name=gpu_submission` 一个 CPU 字段都没有，只有挂钟中位。

```text
$ G=research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out; grep -c 'name=gpu_submission ' $G; grep 'name=gpu_submission ' $G | grep -c -i 'cpu'; sed -n 38p $G
32
0
E7RESULT name=gpu_submission index=0 range_length=32768 range_count=1 input_bytes=32768 buffer_bytes=32804 median_seconds=0.005243958 respects_the_physical_floor=true crcs_agree=true
```

- 仓里现成的等法是阻塞等：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:881`：「let polled = device.poll(wgpu::PollType::Wait {」。阻塞等在驱动里是睡还是转，本工程没量过（推的，没量）。
- 只放甲一段的拆法余量更小：P = 8192 那一批甲段只有 `crc_cpu_nanoseconds=87581`，整批 `call_cpu_nanoseconds=107433410`（第二段第 741 行），甲占这一批的 0.08%。

四句：

1. 分不分辨臂：不分辨甲、乙、丙之间的拆法——每个子集共用「每批一次提交的主机 CPU 小于每批省下的」这个前提，提交的主机 CPU 一旦超过 0.721 ms，十二个点上任何子集一起出局。它分辨的是「GPU 模式」与「CPU 模式」两条臂。按 `evidence-discipline.md`「打中不分辨臂」办：这一格不拿来在甲乙丙之间挑，而是把「每次提交的主机 CPU」另立一笔账先量。
2. 判别子看不看得到：看得到。线程 CPU 时钟包住「准备—提交—等—读回—比对」，跑的时候就能读；只是没人量过。
3. 满足的分句：第 7 条「后者含准备数据、提交、等结果、两份结果的比对」里的「提交、等结果」。
4. 改法还中不中：攒批（乙、丙）把固定开销摊薄，在这一格上有用，但见 G2：两条都撞已定条款。只把等法换成睡眠等，这一格的门槛不变，看量出来的数。

还有一处口径：问题单第 2 行的「不值」用的是份额门槛 10%（E185 Q5–Q7），第 7 条比的是绝对的 CPU 时间差。份额低于 10% 推不出第 7 条上赔，高于 10% 也推不出赚；份额的分母里大头是每批开头整份复制这一版（下一节），那一块哪天被 CPU 一侧的改动拿掉，份额会大幅上升，而能省的绝对值不变（推的）。

### G1-②　「发布那一半」里能放上 GPU 的东西都在共用的发布路径里

F7 说 CPU 的大头在「读这一版、发布」。逐样看今天的代码：

- 读这一版：每推一次发布先整份复制现行那一版，`crates/singlefs-core/src/mounted_session.rs:3004`：「.expect("整理只在有树的一版上推」（那一句之前是 `.version_of_trees()`，之后 `.clone()`）；版本里每个单元都带着字节，`crates/singlefs-core/src/transaction.rs:3174`：「pub units: Vec<PublishedUnit>,」，`crates/singlefs-core/src/transaction.rs:3150`：「pub bytes: Vec<u8>,」。接着按 key 把整份映射重读一遍，`crates/singlefs-core/src/mounted_session.rs:3026`：「let read_of_the_version = version_as_the_compaction_reads_it(&version);」。这是主机内存里的 O(P) 搬运与建表，模型 g1 的 `intent_cpu_ms` 从 P = 1024 到 8192 涨 8.02–8.20 倍，与 P 同比（推的：涨的就是这一块，没单拆量）。它不是能交给 GPU 的计算，CPU 模式与 GPU 模式一样付。
- 发布里唯一成块的计算是释放前读盘核的 CRC-32C：每条意图 0.165–0.495 ms（模型 g1 的 `release_check_crc_ms`），与甲段同一量级。它在 `crates/singlefs-core/src/transaction.rs:11368`：「copies_failing_the_release_checksum_check(」，所在的准备函数是所有发布共用的：`crates/singlefs-core/src/transaction.rs:11216`：「另带这次要搬的单元（整理的一批，[`publish_relocation_batch`]；别的发布给空的）」。

所以要把它放上 GPU，只有两种做法：所有发布都放（前台写的发布也在里面，撞 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`：「不许进前台路径。」），或在共用的准备函数里按调用方分岔（`.claude/rules/fs-design.md:83`：「不许为某个路径（如 fsync）开第二套事务机制」那一节管的是结构；按调用方换计算后端不在它的字面里，但它让两种模式在同一个事务步骤里走两份 CRC 实现，七·4 要两种模式写出的块逐字节相同，这一处要单独证）。四句：分辨「发布那一半」这一条候选与甲乙丙；判别子（这次发布是不是整理的发布）在准备函数里看得到（它收这次要搬的单元，别的发布给空的）；满足 D24 已定项 2 第 2 条字面；改法「只对整理的发布换后端」碰得到这一格，但要回答七·4 的逐字节相同与 G1-③。

### G1-③　今天的触发形态把一批（连同 GPU 往返）放进前台调用的返回路径

- 已定项 13 第 5 条只说什么时候查：`.claude/kb/decisions/26-后台整理与放置回收.md:240`：「每次发布之后按已定项 1 的三水位查一次」。
- 今天的代码把「查了越线就跑至多一批」写在用户改动的发布函数里、返回之前：`crates/singlefs-core/src/mounted_session.rs:1928`：「published.compaction_after_the_change = Some(run_compaction_on(」，同一调用第 1935 行：「CompactionRunReach::AtMostOneBatchAfterAUserChange,」。这个函数就是 `crates/singlefs-core/src/mounted_session.rs:1849`：「pub fn publish_user_change<Device: BlockDevice>(」。实验页也写了：`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:345`：「今天的 GC 整条路在调用线程上同步跑」。
- 今天产品路径不起跑（水位恒 0，只有只供测试的开关会让它起：`crates/singlefs-core/src/compaction_plan.rs:825`：「CompactionRunForcing::ForcedToStartForTests => true,」），七·3 接上水位与定时兜底之后它就是活的。

推论：按今天的形态开 GPU 模式，每次越线后的那一次用户改动要在返回之前等一批，一批里至少一次 GPU 提交（固定挂钟 4.492 / 10.300 ms，G1-①），这就是 GPU 往返进了前台调用的返回路径，撞 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`：「它只能服务已经攒批的后台重活」。所以 G1 的任何推荐都有一个没写出来的前提：GC 先从调用线程挪出去（另一条线程或另一个调用入口）。这件事不在七·2 作业点那一行列的七件事里，也不在已定项 13 里——是越位，还是七·3 的隐含工作量，要主 agent 判。

四句：分辨臂（只打中「沿用今天的触发形态」的那些推荐，GC 先挪出调用线程的推荐不中）；判别子（这一批是不是在用户改动的返回路径上）在代码里写死、看得到；满足 D24 已定项 2 第 2 条字面；改法「GC 挪出调用线程」碰得到，但它同时让 G6-① 的判别子第一次有了意义，两件事要一起定。

### G1 的 Q9

Q9 是 D24 已定项 3 的挂钟形式（CPU 16 线程耗时 ÷ GPU 传输加计算）。`.claude/kb/decisions/26-后台整理与放置回收.md:242` 同一行写明那条按挂钟比的判据不管这一项，所以 Q9 不该当第 2 行的翻面量。第 7 条下真正的翻面量是「每次提交在主机上花的 CPU」，见下面的岔路表。这一句是读条款，不是打中。

### G1 的岔路（交用户定；不写倾向）

| 候选 | 定义 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 每批一次提交 | GPU 只拿甲、乙、丙的某个子集，每批之前照旧重扫，每批一次提交、签发一次 | 每次提交的主机 CPU（准备 + 提交 + 等 + 读回 + 两份比对）高于还是低于这一点上的 `break_even_host_cpu_per_submission_ms`（0.216–0.721 ms） | 两张卡、批字节 16 KiB–1 MiB、阻塞等与睡眠等两种等法，各量提交前后的线程 CPU（含准备与比对），在 E185 的 12 个点上与模型 g1 的门槛逐点比 |
| 攒批提交 | 几批或几条意图的输入一次交 | 同上，门槛乘攒的批数；另要 G2 的三条约束放不放开 | 上一行的量加上用户对 G2 三条约束的答复 |
| 只换发布里的 CRC | 整理的发布里释放前读盘核那一段换 GPU | 那一段每条意图 0.165–0.495 ms 与同一次提交的主机 CPU 比；两种模式块序列是否逐字节相同 | 同第一行的量；外加七·4 那条比对在这条改法上的结果 |

## G2　攒批的形态与显存上限

已定项 13 第 3 条把攒批写成定案：`.claude/kb/decisions/26-后台整理与放置回收.md:238`：「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」；E185 第 3 行把选择缩到乙、丙之间：`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:249`：「一批一交（甲）出局」。下面论证乙、丙两条都撞今天已定的条款，而让甲出局的判据是挂钟的。

### G2-①　乙（连续几批攒起来一次交）撞三条串行约束

1. 每批之前都重扫 R（`crates/singlefs-core/src/mounted_session.rs:2975`，G1-① 引过）：第 j+1 批的输入是第 j 批发布之后的那一版。攒起来一次交，就得用旧版本的扫描结果去计划后几批。
2. 全池至多一条活意图：`crates/singlefs-core/src/transaction.rs:9627`：「全池至多一条，判决 G1 e1」；下一条意图要等上一条完成、全局差值为真才签：`.claude/kb/decisions/26-后台整理与放置回收.md:29`：「为假就不签下一条」；代码里是 `crates/singlefs-core/src/mounted_session.rs:3044`：「if let Some(completed) = *last_completed_intent {」。下一条意图选哪一段、签不签，都要等前一条发完才知道。
3. 一趟整理的发布数有上限：`crates/singlefs-core/src/compaction_plan.rs:812`：「(3 * root_ring_slots_per_region)」，每区根槽数 `crates/singlefs-format/src/lib.rs:309`：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」，刚有用户改动之后是 3 × 8 − 4 = 20 次。

一条意图最多一个聚簇段：`crates/singlefs-format/src/lib.rs:325`：「pub const CLUSTER_SEGMENT_SLOTS: u64 = 64;」，槽 16384 字节，R 在一块盘上至多 1048576 字节。模型 g2 用两张卡的 a、b（第三段第 54、136 行）算攒多少才把 φ = a ÷ (a + b·B) 压下去：

```text
$ python3 -I g2_merge_bounds.py <第三段产物>
name=g2_merge card=0 a_ms=4.492 b_ns_per_byte=0.4117 phi_target=0.5 bytes_needed=10910092 intents_needed_at_full_R=10.4 intents_needed_at_786432=13.9 intents_needed_at_262144=41.6
name=g2_merge card=0 a_ms=4.492 b_ns_per_byte=0.4117 phi_target=0.1 bytes_needed=98190824 intents_needed_at_full_R=93.6 intents_needed_at_786432=124.9 intents_needed_at_262144=374.6
name=g2_one_intent card=0 phi_when_a_whole_R_is_one_submission=0.9123
name=g2_pool card=0 P=1024 pool_data_bytes=33554432 phi_if_the_whole_pool_is_one_submission=0.2454 mapping_table_bytes_one_copy=61440 two_copies=122880
name=g2_pool card=0 P=8192 pool_data_bytes=268435456 phi_if_the_whole_pool_is_one_submission=0.0391 mapping_table_bytes_one_copy=491520 two_copies=983040
name=g2_merge card=1 a_ms=10.300 b_ns_per_byte=1.2660 phi_target=0.5 bytes_needed=8135835 intents_needed_at_full_R=7.8 intents_needed_at_786432=10.3 intents_needed_at_262144=31.0
name=g2_merge card=1 a_ms=10.300 b_ns_per_byte=1.2660 phi_target=0.1 bytes_needed=73222517 intents_needed_at_full_R=69.8 intents_needed_at_786432=93.1 intents_needed_at_262144=279.3
name=g2_one_intent card=1 phi_when_a_whole_R_is_one_submission=0.8858
name=g2_pool card=1 P=1024 pool_data_bytes=33554432 phi_if_the_whole_pool_is_one_submission=0.1951 mapping_table_bytes_one_copy=61440 two_copies=122880
name=g2_pool card=1 P=8192 pool_data_bytes=268435456 phi_if_the_whole_pool_is_one_submission=0.0294 mapping_table_bytes_one_copy=491520 two_copies=983040
name=g2_publishes f=0.25 K=4 batches_per_intent=2 merged_batches=125 intents_spanned=63 publishes_to_consume=189 b_gc_after_a_user_change=20 runs_needed_at_least=10
name=g2_publishes f=0.25 K=4 batches_per_intent=2 merged_batches=374 intents_spanned=187 publishes_to_consume=561 b_gc_after_a_user_change=20 runs_needed_at_least=29
name=g2_publishes f=0.25 K=8 batches_per_intent=1 merged_batches=125 intents_spanned=125 publishes_to_consume=250 b_gc_after_a_user_change=20 runs_needed_at_least=13
name=g2_publishes f=0.25 K=8 batches_per_intent=1 merged_batches=374 intents_spanned=374 publishes_to_consume=748 b_gc_after_a_user_change=20 runs_needed_at_least=38
name=g2_publishes f=0.25 K=16 batches_per_intent=1 merged_batches=125 intents_spanned=125 publishes_to_consume=250 b_gc_after_a_user_change=20 runs_needed_at_least=13
name=g2_publishes f=0.25 K=16 batches_per_intent=1 merged_batches=374 intents_spanned=374 publishes_to_consume=748 b_gc_after_a_user_change=20 runs_needed_at_least=38
name=g2_publishes f=0.75 K=4 batches_per_intent=6 merged_batches=125 intents_spanned=21 publishes_to_consume=147 b_gc_after_a_user_change=20 runs_needed_at_least=8
name=g2_publishes f=0.75 K=4 batches_per_intent=6 merged_batches=374 intents_spanned=63 publishes_to_consume=441 b_gc_after_a_user_change=20 runs_needed_at_least=23
name=g2_publishes f=0.75 K=8 batches_per_intent=3 merged_batches=125 intents_spanned=42 publishes_to_consume=168 b_gc_after_a_user_change=20 runs_needed_at_least=9
name=g2_publishes f=0.75 K=8 batches_per_intent=3 merged_batches=374 intents_spanned=125 publishes_to_consume=500 b_gc_after_a_user_change=20 runs_needed_at_least=25
name=g2_publishes f=0.75 K=16 batches_per_intent=2 merged_batches=125 intents_spanned=63 publishes_to_consume=189 b_gc_after_a_user_change=20 runs_needed_at_least=10
name=g2_publishes f=0.75 K=16 batches_per_intent=2 merged_batches=374 intents_spanned=187 publishes_to_consume=561 b_gc_after_a_user_change=20 runs_needed_at_least=29
```

读法：

- 整个 R 一次交，φ 还是 0.886–0.912，仍在 E185 的「要攒批」那一边（φ ≥ 50%）。攒到 φ ≤ 0.5 要 8135835–10910092 字节，即 7.8–10.4 条整段意图、按 f = 0.25 的签发字节 262144 算是 31.0–41.6 条；攒到 E185 n* 用的 φ ≤ 0.1 要 70–375 条。
- 攒 125 批（卡 0、每批 786432 字节时的 n*）要跨 21–125 条意图、147–250 次发布，一趟至多 20 次，至少 8–13 趟；攒 374 批要 23–38 趟。每两趟之间都可以有用户改动，所以「攒起来一次交」的结果在用的时候已经晚了至少 8 趟。
- P = 1024 时整池的数据只有 33554432 字节，比 φ ≤ 0.1 要的 73222517–98190824 字节还少；把整池一次交，φ 也只到 0.195–0.245。

结论：乙只有放开上面三条中的至少一条才成立，而这三条是代码轮判决（丁 1 交判第 3 点、判决 G1 e1、判决 G8 的 B_gc）与 D26（后台整理与放置回收） 已定项 1 第 2 条定的。七·2 去放开它们是越位。

四句：分辨臂（只打中乙；甲、丙不靠跨批复用）；判别子（后几批的输入是不是已经变了）系统看得到——发布之后版本号就变；满足的是「每批之前都重扫」字面；改法「攒批但每批到点再核一遍」碰得到这一格，代价是核的那一遍就是乙段的 CPU，省下的又还回去（推的）。

### G2-②　乙下「一批之内不换」落空的序列

`.claude/kb/decisions/26-后台整理与放置回收.md:236`：「从下一批开始生效；一批之内不换」。乙一次交 n 批，序列：第 j 批发布之后、第 j+1 批计划之前，控制接口把 `gpu` 切成 `cpu`。

- 照「从下一批开始生效」，第 j+1 批要走 CPU，那已经算好的 j+1…j+n 只能扔掉——这个读法自洽，代价是攒的那一次白算。
- 若实现为了「一批之内不换」把 n 批算成「一批」，那「下一批」就是下一组：切换要等这一组发完，按 G2-① 至少 8 趟、跨若干次用户改动；第 4 条「正在算的那一批放弃」也随之变成整组 125 批放弃。
- 第 j+1 批若沿用 GPU 结果，就是切到 `cpu` 之后还在用 GPU 的结果，撞第 1 条字面。

所以乙下「批」的定义要先写死；E185 的口径是 `.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:322`：「批：一次调用里搬了单元的那一次」，与「一组提交」不是一回事。`.claude/rules/fs-design.md:128`：「分支变量不许在操作中途改变」管的「操作」取哪一个，这里不能两用。

### G2-③　丙（放宽 R）是格式改动，而且要放到 8–125 个聚簇段

- 意图条目里 R 的槽数是定宽字段、只收 64：`.claude/kb/decisions/26-后台整理与放置回收.md:64`：「恒 64（一个聚簇段），别的值拒收」。放宽 R 就是改这个字段的取值域，是格式改动。
- 只在 GPU 模式放宽：两种模式写出的意图条目不同，撞 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`：「一个比特都不许进格式」（GPU 模式改变了盘上写什么），也撞七·4 `.claude/kb/milestone/04-fourth-txn.md:146`：「两种模式写出的块序列逐字节相同」。
- 两种模式都放宽：那是改 CPU 模式的整理语义（停机谓词按 R 判全空、产出按聚簇段数、discard 按 64 槽对齐整段），要重开 D26（后台整理与放置回收） 已定项 1、2，七·2 去定是越位。
- 量级：按 G2-① 的表，φ ≤ 0.5 要 R 放到 7.8–10.4 个整段，φ ≤ 0.1 要 70–94 个整段。R 在意图期内不许分配（`.claude/kb/decisions/26-后台整理与放置回收.md:30`：「R 在意图期内不可作分配目的地」），即每块盘同时有 8–94 MiB 不能分配；P = 1024 的池整池数据才 32 MiB。

四句：分辨臂（只打中丙）；判别子（意图条目里 R 的槽数）在盘上、看得到；满足 D26 已定项 2 字段表那一行与 D24 已定项 2 第 1 条字面；改法「R 的大小只活在内存、盘上仍写 64 槽的一串意图」碰得到格式那一半，但回到 G2-① 的「至多一条活意图」。

### G2-④　让甲出局的判据是挂钟的

E185 Q13 判「要攒批」的是固定开销占挂钟的份额 φ（`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:201`：「56 行 `judgement=merge_batches`（φ ≥ 50%」）。第 7 条比的是主机 CPU。φ 大说明 GPU 那边闲着等的时间多，不说明主机 CPU 花在那段等上：主机线程在等的时候若睡着，一批一交的固定开销几乎不花主机 CPU，甲在第 7 条下不出局；若转着，甲出局，而乙、丙按 G2-①③ 又撞已定条款，GPU 模式按 G1-① 赔。两种情形分辨得开的量还是 G1 岔路表第一行那一个。这一处满足的是 `evidence-discipline.md`「打中归错了判据」：Q13 的门槛是挂钟份额，被判的结论（GPU 模式值不值）按第 7 条是 CPU 时间。

### G2-⑤　显存上限

- 第 3 条要攒批、缓冲、暂存区都算在上限里（`.claude/kb/decisions/26-后台整理与放置回收.md:238`）。两份核若各备一份输入（G4 讨论的「独立」要不要求输入各备一份），攒到 φ ≤ 0.1 的输入就是 2 × 73222517 至 2 × 98190824 字节，外加每份核一张映射表（P = 8192 时一份 491520 字节，模型 g2 按推的布局 60 字节一条）。上限给到这以下，φ 只能停在更高处——这不违约（「用得比它少没有关系」），只是 G1 的余量更小。
- 第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」：攒多少就随别的进程占了多少显存而变。乙下攒多少决定后几批用的是哪一版的扫描结果，所以「两种模式块序列逐字节相同」在乙下还多了一个外部变量（推的）。

### G2 的岔路（交用户定；不写倾向）

| 候选 | 定义 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 甲：一批一交 | 每批之前照旧重扫，每批一次提交 | 每次提交的主机 CPU 是否低于模型 g1 的门槛 | G1 岔路表第一行的量 |
| 乙：跨批攒 | 几批或几条意图的输入一次交 | 用户放不放开「每批重扫 / 至多一条活意图 / B_gc」；放开之后后几批用旧扫描结果的错误率 | 用户答这三条；放开的那条另走代码轮 |
| 丙：放宽 R | 一条意图罩住多个聚簇段 | 用户放不放开意图条目 R 槽数恒 64 与已定项 1 的停机、产出口径 | 用户答；改格式走 D26 重开 |

## G3　多张卡怎么分活

### G3-①　「本机全部显卡」里会混进 CPU 上跑的软件适配器（推的，没起 GPU）

`.claude/kb/decisions/26-后台整理与放置回收.md:237`：「不给就用本机全部显卡」。本机装着的 Vulkan 驱动清单（只列文件，没起任何设备）：

```text
$ ls /usr/share/vulkan/icd.d/ /etc/vulkan/icd.d/
/etc/vulkan/icd.d/:

/usr/share/vulkan/icd.d/:
asahi_icd.json
gfxstream_vk_icd.json
intel_hasvk_icd.json
intel_icd.json
lvp_icd.json
nouveau_icd.json
nvidia_icd.json
```

`lvp_icd.json` 是在主机 CPU 上跑的软件 Vulkan。checker 档自己的做法是把进程环境变量改成只认 NVIDIA 那份清单、再只收 NVIDIA 独显：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:417`：「std::env::set_var("VK_DRIVER_FILES", NVIDIA_VULKAN_ICD_MANIFEST);」，第 454 行：「|| information.device_type != wgpu::DeviceType::DiscreteGpu」，另按 `nvidia-smi` 报的总线号对。产品三样都借不来：D13（验证路线） 已定项 5 不许与判器共用（`.claude/kb/decisions/13-验证路线.md:156`：「校验和各用一份独立实现」那一句所在的定案），库改进程环境变量会波及调用方，`nvidia-smi` 是厂商工具。

若产品照 wgpu 枚举出来的全部 Vulkan 适配器当「全部显卡」，软件适配器（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1317`：「wgpu::DeviceType::Cpu => AdapterDeviceKind::SoftwareRenderingOnTheCpu,」）就算一张卡：分给它的那一份活在主机 CPU 上算，按第 7 条一定赔；只有它一张时，第 4 条「GPU 不在就拒绝挂载」也不会触发。wgpu 在本机会不会把它枚举出来，我没起 GPU 去看（禁令），这一句是推的。

四句：分辨臂（打中「全部 = 枚举到的全部」的读法，不打中「全部 = 按设备类型筛过的独显与集显」）；判别子（适配器类型）枚举时就看得到；满足第 2 条「不给就用本机全部显卡」的字面没有定义「显卡」；改法「按 `DeviceType` 筛、只收独显与集显，筛完为空就照第 4 条拒挂」碰得到这一格（我提的，只在推理上，被攻过零轮）。

### G3-②　一张卡与几张卡的判定在故障下不同

`.claude/kb/decisions/26-后台整理与放置回收.md:243`：「不许要求多张卡，一张与几张都要能跑、判定相同」。序列：同一批、同一份输入，一张卡时两份核读同一块显存里的那份输入；没有 ECC 的显存在输入表上翻了一位，两份核看到的是同一份错的输入，结果相同，判「确认」。两张卡时两份核各在一张卡上，输入各传一份，同一个翻位只在一张卡上，判「不确定」。同一个故障，一张卡判确认、两张卡判不确定，判定不同。

所以「判定相同」只在没有故障时成立；要在有故障时也成立，一张卡时两份核要各备一份输入（显存翻倍，回到 G2-⑤）。这一条连着 G4 的「独立」判据。四句：分辨「一张卡时共用输入」与「各备一份」两种做法；判别子（输入是不是同一块显存）实现里看得到；满足第 8 条那一句字面；改法「一张卡时也各备一份输入」碰得到这一格，代价是显存翻倍与多一次上传（主机 CPU，回到 G1-①）。

## G4　两份 GPU 核比对的形态与 N

### G4-①　「输入表的准备」这一类，射程说靠测试对拍抓，七·3 写的对拍抓不到

射程原话：`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「那一类靠 GPU 与 CPU 两份实现在测试里的对拍和崩溃点重放抓」，「那一类」指同一行列的「同一个驱动与着色器编译器、同一份输入表的准备、对规则的同一种误读」。七·3 的验收把对拍写成 `.claude/kb/milestone/04-fourth-txn.md:145`：「两条路径在同一组输入上逐项相同」。输入表在这种对拍里是两边共用的，准备那一步写错，两边一起错。

今天 CPU 一侧取映射时把留存给快照的条目接在后面：`crates/singlefs-core/src/mounted_session.rs:2826`：「mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(」。GPU 一侧的准备若照这一版自己的映射取、漏了这一步，R 里只剩快照引用的单元在两份核里都判「已死」，CPU 判「可搬」。E185 的池没有快照（`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:271`：「snapshot_branch_note=the_pools_have_no_snapshot」），照 E185 那种负载跑对拍看不见这一支。

模型 g4（两份分类核一份逐条扫、一份先建索引，算法不同，共用同一份准备；分类次序照 `region_scan.rs` 缩成钉住以外的三类）：

```text
$ python3 -I g4_shared_input_table.py 185 24 0; … 185 24 3; … 185 24 0 one-kernel-fault
name=g4_model seed=185 units_in_region=24 kept_only=0
name=g4_runtime_two_kernels differing_entries=0 confirmed=24
name=g4_test_same_input_cpu_vs_gpu differing_entries=0
name=g4_end_to_end_modes differing_entries=0 gpu_dead_cpu_movable=0
name=g4_model seed=185 units_in_region=24 kept_only=3
name=g4_runtime_two_kernels differing_entries=0 confirmed=24
name=g4_test_same_input_cpu_vs_gpu differing_entries=0
name=g4_end_to_end_modes differing_entries=3 gpu_dead_cpu_movable=3
name=g4_model seed=185 units_in_region=24 kept_only=0
name=g4_runtime_two_kernels differing_entries=1 confirmed=23
name=g4_test_same_input_cpu_vs_gpu differing_entries=0
name=g4_end_to_end_modes differing_entries=0 gpu_dead_cpu_movable=0
```

- 第一组（R 里 24 个单元、没有只剩快照引用的）是阴性对照：三种比对都 0 处不同。
- 第三组是阳性对照：只让第二份核在一条上交错的类，运行时互比报 1 处不同——互比会红。
- 第二组（24 个里 3 个只剩快照引用）：两份核互比 0 处不同（24 条全「确认」），「同一组输入」上的 CPU 对 GPU 对拍 0 处不同，只有两种模式各自从这一版取输入再比时 3 处不同（GPU 判已死、CPU 判可搬）。

这是模型，不是 crates：它只说明结构——准备那一步在两份核与同输入对拍之外，只有「两种模式各读这一版」那一类比对抓得到。七·3 的「模型对拍 gpu 模式与 cpu 模式读到的内容逐项相同」与七·4 的块序列比对属于这一类，但要在带快照、且快照引用的单元落在 R 里的流上跑才看得见。

后果按今天的代码推：判「已死」的不搬，R 腾不空，意图按「一个能搬的都没有而 R 不空」丢掉、这一趟停（`mounted_session.rs` 第 2981 行注释那一条），是活性问题、不丢数据；里程碑五「已死」那一支开始回收之后，同一个漏法就是丢快照数据（推的）。

四句：分辨「独立判据允许共用输入准备」与「每份核各自从这一版取输入」两种；判别子（两种模式各读这一版的结果）系统在测试里看得到，运行时两份核都看不到；满足射程那一句字面里的「同一份输入表的准备」；改法甲「七·3 的对拍从这一版起算、流里要有快照」碰得到这一格；改法乙「每份核各自准备输入」也碰得到，但准备的主机 CPU 翻倍，回到 G1-① 的余量（只在我的推理上，被攻过零轮）。

### G4-②　「CPU 只改位图，不再抽样复核」与今天的发布路径相左

`.claude/kb/decisions/26-后台整理与放置回收.md:243`：「CPU 只改位图，不再抽样复核」。今天整理的发布对计划里的每一条都在 CPU 上对上一版核一遍：不在这一版的拒、钉住的角色拒、源落点不是映射位置的拒（`crates/singlefs-core/src/transaction.rs:10224`：「RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {」，函数 `relocation_checked_against_the_version` 从第 10137 行起），留存条目读盘核 CRC（第 10249 行）。这些核正是 G4-① 那类「两份核一起错」在今天不变成静默损坏的原因：错判「可搬」的会被拒、整理报错停下。

两种读法各有代价：

| 读法 | 后果 |
|---|---|
| 七·3 照第 8 条字面拿掉这些核，省主机 CPU | 两份核一起错的结果直接改盘，G4-① 那一类变成静默的 |
| 留着 | 第 8 条那句不准确，要改字；这些核的 CPU 照算在 GPU 模式里（第 7 条） |

四句：分辨这两种读法；判别子（发布路径里有没有逐条核）代码里看得到；满足第 8 条那一句字面与 `transaction.rs` 第 10137 行起那个函数；没有跑前改法可核。

### G4-③　N 的计数住内存，意图与 R 排他住盘上

第 8 条：`.claude/kb/decisions/26-后台整理与放置回收.md:243`：「同一个条目连续 N 轮都不确定，交 CPU 那一份实现单独判一次」。挂载参数只活在内存（第 236 行：「它只活在内存里」），「连续几轮」的计数住哪没有写；意图住 logged_ops，R 在意图期内不许分配（第 30 行）。序列：

1. 第二份核在某一类条目上有确定性的错（每轮都与第一份不同）。
2. 那一条每轮判「不确定」，第 8 条「这一批不回收、不搬」，意图留着、R 继续排他。
3. 计数到 N 之前卸载重挂（或切到 `cpu` 再切回 `gpu`），计数从 0 起。
4. 重挂的频率高于每 N 轮一次，那一条永远到不了 CPU 判；GPU 模式下这条意图永远完成不了，R 永远不能分配。第 4 条只在「GPU 出错」时停 GC 报错，「不确定」不算出错，所以不报错。

「一轮」是什么也没定：今天一趟整理在一次用户改动之后至多 20 次发布（G2-①），触发是「每次发布之后」与定时兜底（第 240 行），所以 N 轮折成挂钟是 N 次触发，而不确定的那一批不发布，那一趟是不是就此停、下一轮要等下一次触发，代码里还没有这一支（推的）。

四句：分辨「计数住内存、重挂清零」与「计数随意图持久或重挂沿用」；判别子（这一条已经不确定几轮）在内存里有、重挂后没有；满足第 8 条与第 1 条「只活在内存里」两句合起来；改法「不确定的条目在重挂与切换时直接交 CPU 判」碰得到这一格（我提的，被攻过零轮）。

### G4-④　运行时的着色器编译器：测试覆盖不到用户那台机器的驱动

两份核都经同一个 naga → SPIR-V → 驱动编译器。测试里的对拍只证明了测试那台机器上的那一版驱动（E185 第三段两张卡都是 `driver=NVIDIA_595.99.02`，第三段第 37、119 行）。用户机器的驱动不同时，两份核若用了同一种写法（WGSL 没有 u64、没有字节寻址，按字取字节都写成移位加掩码，E163 臂 G 就是 `(word >> 8*(k%4)) & 0xFF`），同一个编译器缺陷会让两份一起错。第 8 条明令「不再抽样复核」，运行时就没有一道防线看得到这一类。

已有的现成办法是开跑前对已知答案自检：E185 第三段每张卡开头核一串已知字符串的 CRC（第 37 行 `check_string_crc32c=0xe3069283 check_string_holds=true`）。把它搬到产品的挂载与切换时刻、并对两份核各核一组固定输入，不算「抽样复核」，抓得到「这台机器的编译器把两份核都编错」（我提的，被攻过零轮）。

### G4 的岔路（交用户定；不写倾向）

| 候选 | 定义 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 独立甲：只不共用核函数 | 两份核共用输入准备、驱动、编译器；比对在 CPU 上 | 「两种模式各读这一版」的对拍在带快照的流上红不红 | 七·3 那条对拍改成从这一版起算、流里带快照，跑一遍 |
| 独立乙：输入也各备一份 | 每份核自己从这一版取输入、自己传；一张卡时显存两份 | 准备的主机 CPU 翻倍之后第 7 条的余量（G1-①） | G1 岔路表第一行的量，外加准备那一段单拆 |
| N 取 1 | 不确定一次就交 CPU | 瞬时不一致（位翻、竞争）与确定性不一致的比例 | 七·3 的故障注入两类各跑，数各自几轮消掉 |
| N 取大于 1 | 留几轮重算 | 同上；外加计数住哪（G4-③） | 同上，外加重挂、切换时计数的规则写死 |

## G5　控制接口（挂着切换）的形态

找到的脆弱点，不算打中：今天整理的开关在一趟开头读一次：`crates/singlefs-core/src/mounted_session.rs:2641`：「let switches = allocator.compaction().switches;」，批大小在这一趟里不变。会话是 `&mut self` 的库调用，整理在调用线程上跑（G1-③），控制调用落不进一趟的中间，所以今天「从下一批生效」与「从下一趟生效」是同一件事，模式字段跟着 `switches` 读就满足第 1 条。

一旦照 G1-③ 把 GC 挪出调用线程，切换请求会在一趟的中间到。那时边界要落在每推一次发布的开头（`advance_compaction_by_one_publish`，`mounted_session.rs` 第 3002 行取这一版的那一句），而不是一趟开头；乙下还要先定「批」是一次发布还是一组提交（G2-②）。这一格的推荐要是写成「在 `switches` 上加一个字段」，GC 挪线程之后就从「下一批」退成「下一趟」。

## G6　三个默认值

### G6-①　「前台有 fsync 在等」这个判别子今天观测不到

第 6 条：`.claude/kb/decisions/26-后台整理与放置回收.md:241`：「前台有 fsync 在等时不发下一批」。要做这个判断，GC 在发下一批之前要看得到「有没有前台 fsync 在等」。今天：

- GC 跑在用户改动的同一次调用里（G1-③）或显式入口 `run_compaction`（`&mut self`）里；
- 会话没有第二个调用者能在这时候等：库调用是独占借用，前台唯一的那次调用就是正在跑 GC 的这一次。

所以在做判断的那一刻，「有 fsync 在等」恒为假，两种情形（有人等、没人等）在 GC 看得到的东西上逐条相同。这是 `evidence-discipline.md`「判别子观测不到」：写带宽上限与兜底间隔取什么默认值都碰不到它。四句：分辨「GC 在调用线程上」与「GC 在另一条线程、能看到前台队列」两种架构；判别子今天看不到；满足第 6 条后半句字面；改法只有架构那一条（与 G1-③ 同一件事）。

### G6-②　写带宽上限压不住一次 fsync 的等：一批的长短由 CPU 定、随池大小涨

前台 fsync 最坏等一整批（在批中间到）。模型 g6 从第一段产物取每个点一批的挂钟、CPU 占挂钟的比、写字节，再算这些写按 50 MB/s 与 500 MB/s 的上限摊开要多长：

```text
$ python3 -I g6_inline_batch_latency.py <第一段产物> 50000000,500000000
name=g6_batch arm=memory f=0.25 P=1024 K=4 batches=10 wall_ms_median=12.542 wall_ms_max=12.736 cpu_over_wall_median=1.0000 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=memory f=0.25 P=1024 K=8 batches=5 wall_ms_median=12.979 wall_ms_max=13.234 cpu_over_wall_median=1.0000 write_bytes_median=999936 paced_ms_at_50000000=19.999 paced_ms_at_500000000=2.000
name=g6_batch arm=memory f=0.25 P=1024 K=16 batches=5 wall_ms_median=12.956 wall_ms_max=13.011 cpu_over_wall_median=1.0000 write_bytes_median=999936 paced_ms_at_50000000=19.999 paced_ms_at_500000000=2.000
name=g6_batch arm=memory f=0.25 P=8192 K=4 batches=10 wall_ms_median=105.768 wall_ms_max=109.059 cpu_over_wall_median=0.9999 write_bytes_median=901632 paced_ms_at_50000000=18.033 paced_ms_at_500000000=1.803
name=g6_batch arm=memory f=0.25 P=8192 K=8 batches=5 wall_ms_median=111.090 wall_ms_max=111.701 cpu_over_wall_median=1.0000 write_bytes_median=1262080 paced_ms_at_50000000=25.242 paced_ms_at_500000000=2.524
name=g6_batch arm=memory f=0.25 P=8192 K=16 batches=5 wall_ms_median=110.841 wall_ms_max=110.929 cpu_over_wall_median=0.9998 write_bytes_median=1262080 paced_ms_at_50000000=25.242 paced_ms_at_500000000=2.524
name=g6_batch arm=memory f=0.75 P=1024 K=4 batches=30 wall_ms_median=12.508 wall_ms_max=12.860 cpu_over_wall_median=1.0000 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=memory f=0.75 P=1024 K=8 batches=15 wall_ms_median=12.866 wall_ms_max=13.664 cpu_over_wall_median=1.0000 write_bytes_median=999936 paced_ms_at_50000000=19.999 paced_ms_at_500000000=2.000
name=g6_batch arm=memory f=0.75 P=1024 K=16 batches=10 wall_ms_median=13.434 wall_ms_max=14.266 cpu_over_wall_median=1.0000 write_bytes_median=1294848 paced_ms_at_50000000=25.897 paced_ms_at_500000000=2.590
name=g6_batch arm=memory f=0.75 P=8192 K=4 batches=30 wall_ms_median=103.327 wall_ms_max=107.532 cpu_over_wall_median=0.9998 write_bytes_median=819712 paced_ms_at_50000000=16.394 paced_ms_at_500000000=1.639
name=g6_batch arm=memory f=0.75 P=8192 K=8 batches=15 wall_ms_median=102.437 wall_ms_max=108.761 cpu_over_wall_median=0.9998 write_bytes_median=1098240 paced_ms_at_50000000=21.965 paced_ms_at_500000000=2.196
name=g6_batch arm=memory f=0.75 P=8192 K=16 batches=10 wall_ms_median=107.182 wall_ms_max=113.660 cpu_over_wall_median=0.9999 write_bytes_median=1376768 paced_ms_at_50000000=27.535 paced_ms_at_500000000=2.754
name=g6_batch arm=real_disk f=0.25 P=1024 K=4 batches=10 wall_ms_median=21.671 wall_ms_max=23.604 cpu_over_wall_median=0.6030 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=real_disk f=0.25 P=1024 K=8 batches=5 wall_ms_median=22.764 wall_ms_max=22.933 cpu_over_wall_median=0.5924 write_bytes_median=999936 paced_ms_at_50000000=19.999 paced_ms_at_500000000=2.000
name=g6_batch arm=real_disk f=0.25 P=1024 K=16 batches=5 wall_ms_median=23.195 wall_ms_max=24.052 cpu_over_wall_median=0.5748 write_bytes_median=999936 paced_ms_at_50000000=19.999 paced_ms_at_500000000=2.000
name=g6_batch arm=real_disk f=0.25 P=8192 K=4 batches=10 wall_ms_median=116.797 wall_ms_max=121.288 cpu_over_wall_median=0.9028 write_bytes_median=901632 paced_ms_at_50000000=18.033 paced_ms_at_500000000=1.803
name=g6_batch arm=real_disk f=0.25 P=8192 K=8 batches=5 wall_ms_median=123.135 wall_ms_max=124.717 cpu_over_wall_median=0.8965 write_bytes_median=1262080 paced_ms_at_50000000=25.242 paced_ms_at_500000000=2.524
name=g6_batch arm=real_disk f=0.25 P=8192 K=16 batches=5 wall_ms_median=123.369 wall_ms_max=123.760 cpu_over_wall_median=0.8982 write_bytes_median=1262080 paced_ms_at_50000000=25.242 paced_ms_at_500000000=2.524
name=g6_batch arm=real_disk f=0.75 P=1024 K=4 batches=30 wall_ms_median=23.728 wall_ms_max=24.898 cpu_over_wall_median=0.5512 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=real_disk f=0.75 P=1024 K=8 batches=15 wall_ms_median=24.255 wall_ms_max=25.458 cpu_over_wall_median=0.5660 write_bytes_median=999936 paced_ms_at_50000000=19.999 paced_ms_at_500000000=2.000
name=g6_batch arm=real_disk f=0.75 P=1024 K=16 batches=10 wall_ms_median=25.408 wall_ms_max=27.179 cpu_over_wall_median=0.5494 write_bytes_median=1294848 paced_ms_at_50000000=25.897 paced_ms_at_500000000=2.590
name=g6_batch arm=real_disk f=0.75 P=8192 K=4 batches=30 wall_ms_median=115.143 wall_ms_max=120.777 cpu_over_wall_median=0.9057 write_bytes_median=819712 paced_ms_at_50000000=16.394 paced_ms_at_500000000=1.639
name=g6_batch arm=real_disk f=0.75 P=8192 K=8 batches=15 wall_ms_median=114.157 wall_ms_max=122.320 cpu_over_wall_median=0.9051 write_bytes_median=1098240 paced_ms_at_50000000=21.965 paced_ms_at_500000000=2.196
name=g6_batch arm=real_disk f=0.75 P=8192 K=16 batches=10 wall_ms_median=119.442 wall_ms_max=126.859 cpu_over_wall_median=0.8957 write_bytes_median=1376768 paced_ms_at_50000000=27.535 paced_ms_at_500000000=2.754
```

- 一批的挂钟：内存臂 P = 1024 是 12.508–13.434 ms，P = 8192 是 102.437–111.090 ms；真盘臂 21.671–25.408 ms 与 114.157–123.369 ms（中位）。P 涨 8 倍，一批涨 4.70–8.56 倍，CPU 占挂钟真盘臂从 0.5494–0.6030 涨到 0.8957–0.9057，内存臂恒 1.0。
- 一批只写 737792–1376768 字节。500 MB/s 的上限把它摊成 1.476–2.754 ms，比一批短得多，碰不到等的长度；50 MB/s 摊成 14.756–27.535 ms，在 P = 1024 上反而比一批本身还长——上限若在批里面按时间摊写，它把前台要等的那一批拉长（推的：摊在批内还是批间，第 6 条没写）。
- E185 Q14 量到 GC 单跑的写带宽中位只有 7229989–50509463 字节/秒（`.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md:158`：「write_bytes_per_second_median` 7229989.048161–50509463.337147」），Q16 是它与设备顺序写带宽之比 0.015977。前台被挤的通道不是盘写带宽，是一批占着会话的那段 CPU 时间。所以写带宽上限的默认值从 Q16 取，取多少都管不到 fsync 让路。

负载：大池（P ≥ 8192）、水位越线后持续有小 fsync。按今天的触发形态，每次越线后的那次用户改动要多等一批 ≥ 102 ms（内存）/ ≥ 114 ms（真盘），而且随 P 涨（池再大没量，E185 只收两个 P）。GPU 模式每批再加 ≥ 4.492 / 10.300 ms 的提交固定挂钟。

四句：分辨「按写带宽限速」与「按一批的挂钟或 CPU 时间限」两种节奏；判别子（一批已经跑了多久）系统看得到；满足第 6 条前半句「GC 占盘写带宽有一个上限」——这一句只管写带宽，不管一批多长；改法「把批大小或一批的时长做成上限」碰得到这一格，「调写带宽上限的默认值」碰不到。

### G6-③　另两个默认值

- 定时兜底间隔：射程把它压在 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理） 上（`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「定时兜底的间隔、写带宽上限的默认值（等 C412」）。E185 量的是一批的代价，没量碎片长多快；E10 第一段 21 片只跑了 4 片（`.claude/kb/experiments/10-老化结构抗老化vs事后整理.md:23`：「第一段 21 片跑了 4 片」）。间隔的默认值今天没有量过的依据，取什么都是拍的。
- 显存上限不给时「能用多少用多少」：要另设的保护至少有一样——攒批的大小随别的进程占用的显存变（G2-⑤），这会让结果依赖外部状态，七·4 的逐字节相同要在乙下单独证。

### G6 的岔路（交用户定；不写倾向）

| 候选 | 定义 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 留在调用线程 | 今天的形态：越线后在用户改动的返回之前跑至多一批 | 用户改动返回的挂钟分布（有没有整理那一批）随 P 的增长 | E185 第四段（Q17）在今天形态上跑，P 多取几档 |
| 挪到另一条线程、每批之间看前台 | 第 6 条照字面实现 | fsync 在批中间到时的等待分布 | 架构定下之后跑 Q17，批中间到的那一类单列 |
| 按时长限一批 | 一批的 CPU 时长有上限，超了切小批 | 一批时长上限与 GC 总吞吐的取舍 | 同上，再扫批大小 |

## G7　GPU 代码住哪、用什么写

找到的脆弱点，都是推的：

- 依赖方向：GPU 包要读 `singlefs-core` 的类型（分配记录、映射条目）就得依赖它，那 core 调不到它，只能由挂载的调用方把一个后端对象递进来；第 4 条要求点名 `gpu` 而后端没递进来时拒挂，core 自己判得出（「没有后端」就是「GPU 不在」）。反过来让 core 可选依赖 GPU 包，GPU 包就不能用 core 的类型，输入表的格式要在 core 里按 GPU 的布局拼好——那一段就是 G4-① 两份核共用的准备。两种方向都行，后一种把共用的准备放进了 core。
- CPU 参照：第 8 条要的「CPU 那一份」就是今天的 `scan_region` / `choose_region`，CPU 模式也走它。GPU 模式里于是有三份实现（CPU 一份、GPU 两份）各自解单元头、算 CRC、拼映射 key；D13（验证路线） 已定项 5 的常量生成器只发标量（`.claude/kb/decisions/13-验证路线.md:159`：「生成器拒绝发射没有 kb 落点的常量」那一条），WGSL 拿不到 Rust 常量，要么另生成一份 WGSL 常量、要么由主机经 uniform 递——后者又是两份核共用的输入。
- 「更好的做法」比什么：按第 7 条，该比每次提交在主机上花的 CPU（wgpu 的校验与资源跟踪是主机上的开销），不是着色器语言能不能写；本工程没有任何一处量过 wgpu 每次提交的主机 CPU（G1-①），所以 WGSL + wgpu 与直接 Vulkan 计算之间现在没有可比的数。WGSL 没有 u64、没有字节寻址，两份核都要自己拼 64 位比较与按字取字节（G4-④），这是语言带来的共同前提，换 SPIR-V 直写或别的语言能去掉一部分。
- 与 checker 档共不共用：不许（`.claude/kb/decisions/13-验证路线.md:156`：「校验和各用一份独立实现」）。checker 档的卡发现靠改进程环境变量与 `nvidia-smi`（G3-①），产品本来也借不了。

## 没打中的形状

| 格 | 试过什么 | 为什么不成立 |
|---|---|---|
| G1 | 找一种拆法，让 GPU 模式在第 7 条上「一定赔」 | 赔不赔取决于每次提交的主机 CPU，这个数没量；我只能给门槛（0.216–0.721 ms），给不出「一定」。打中的是「余量极薄且判据没量」，不是「已知赔」 |
| G1 | 找「发布那一半」里不在共用路径上的计算 | 释放前读盘核以外，发布里剩下的是复制版本、建表、重写树节点、写 journal，都是主机内存搬运或走共用的事务层；没找到第三样 |
| G2 | 找乙在一条意图之内攒批的形状 | 一条意图至多 64 槽、每批之前重扫；整段 R 一次交 φ 仍 0.886–0.912，攒不出 E185 判据要的量 |
| G4 | 找两份核都错而发布路径也放过的序列（静默损坏） | 今天的发布路径对每条计划核映射位置、钉住、在不在这一版（G4-②），错判「可搬」的都被拒；只找到活性问题（不搬、意图丢掉）。要变成静默损坏得先拿掉这些核或等「已死」那一支开始回收 |
| G5 | 找一个切换在一批中间生效的序列 | 今天会话独占借用，控制调用落不进一趟中间；只在 GC 挪线程之后成立（写成脆弱点） |
| G3 | 找两张异构卡（5090 与 5060 Ti）在无故障时判定不同的序列 | 两份核若都是确定性的整数运算，按条目判、与分到哪张卡无关；没找到无故障下的分歧，只找到故障下的（G3-②） |

取样范围：G1、G6 是 E185 的全部 12 个点（f 两档 × P 两档 × K 三档），G2 是两张卡 × 两个 φ 门槛 × 三种每段字节，G4 模型是一个种子、R 里 24 个单元、只剩快照引用的 0 与 3 两档加一条阳性对照。

## 这条腿自己的限度

- 我的模型只读产物、做算术；没有编译、没有跑 `crates/`、没有起 GPU。「每次提交的主机 CPU」是这份报告里最关键的数，我没量，也量不了（禁令）。
- 模型 g2 的映射表字节（每条 60 字节）是推的布局，只用来说量级。
- 模型 g4 是缩写的分类规则，不是 `region_scan.rs`；它只证明结构（共用准备在两份核与同输入对拍之外），不证明 crates 里会有这个漏法。
- G1-③、G6-① 依赖「今天的会话是单线程独占借用」这个读法，读的是 `mounted_session.rs` 的签名与调用点；七·3 若本来就打算换线程模型，这两处变成「前提要先写出来」，不再是矛盾。
- 我的改法（G3-① 按设备类型筛、G4-① 对拍从这一版起算、G4-③ 重挂时直接交 CPU、G4-④ 挂载时已知答案自检）都只在我的推理上，被攻过零轮。

## 没做什么

- 没读正推腿的报告与模型（禁读）。
- 没跑门禁、没跑 checker 档测试、没起 GPU、没用第二台。
- 副本与编译目录：没建（只用 python）；草稿目录里有模型的草稿与复跑输出 `rerun1/`，交回前删。
