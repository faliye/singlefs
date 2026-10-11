# m3-batch59-code-r1 正推腿（Sonnet）报告

被判：第 59 批三份补丁（甲 L1、乙 L2 与 L5、丙 L3 与 L4），格 M1–M3。立场：正推，核「测试测的是不是条款与判决点名的那件事」。
禁读清单照守：没读 `m3-batch59-code-r1-opus-output.md`、`m3-batch59-code-r1-opus-model/`、`m3-batch59-code-r1-main-verification.md`；读了上一轮判决 `m3-batch58-code-r1-main-verification.md`（题面所在）。题面 L1–L5 取自 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节。

## 〇、证据与环境

- 冻结副本 `~/.local/share/singlefs/work/m3-batch59-code-r1/repo/`，被判四份文件加变异表的 sha256 逐份核过，对得上快照（`sha256sum -c research/prompts/m3-batch59-code-r1-snapshot/crates-sha256.txt` 在冻结副本里五行都是 OK）。
- 本腿的量全在拷贝 `~/.local/share/singlefs/work/m3-batch59-code-r1-sonnet/repo/` 上做，编译目录 `…/m3-batch59-code-r1-sonnet/target`；每条 cargo 经 `nice -n 19 bash scripts/capped.sh 15 bash scripts/run-with-memory-cap.sh 10G cargo …`，同一时刻一条；没跑门禁、没跑 checker 档测试（GPU 那三条读码判）。
- 开跑前 `ps` 看到别的会话在跑 `cargo test --release -p singlefs-checker-tier …`、崩溃点重放 `--quick`、一条 `-p opus-probe-b59`，没有 qemu、fio、vm-bench；整机 load average 26–39（32 核），本腿测试全在 nice 19 下被抢 CPU，时间数偏大。
- 脚本与日志：`research/prompts/m3-batch59-code-r1-sonnet-model/`（里面的 `run.sh`、`apply_row.py`、`l4.sh`、`l2.sh`、探针与 `logs/`），原件在 `~/.local/share/singlefs/work/m3-batch59-code-r1-sonnet/`。
- 几处 kb 事实先摆：
  - 「判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律」（`.claude/kb/invariants.md:35`）；I-1.10 本身只规定条目宽先于按字段表解条目，分配记录判在不在条目宽之前，kb 的 I-1.10 没写。
  - 「走读到的任一版」（`.claude/kb/invariants.md:155`）是 I-3.13 的适用面，同行「inode 号水位那一统计量的台阶行不判」。
  - 「每个打包记录类型 1 容器记录数 ≤ 旋钮值」（`.claude/kb/invariants.md:310`）是 I-9.22 的全部量词：kb 只写了 ≤ 旋钮值，没写「不是最右的容器等于旋钮值」。
  - 「满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」（`.claude/kb/decisions/18-块里携带什么信息.md:578`）是「不是最右的容器等于旋钮值」的出处。
  - I-9.36、I-9.37 在 kb 的 `invariants.md` 里没有行（`grep -c 'I-9\.37' .claude/kb/invariants.md` 输出 0），这一批的 GPU 与 harness 注释里写 I-9.37 的地方，条款侧只有 `~/.local/share/singlefs/work/m3-batch57/container-judges/report.md`（正文第一节已说）。

## 一、M1　GPU 旋钮流转根环（甲 L1）

GPU 那三条在 checker 档，按要求读码判；版本级的事实我用 harness 里同名工具造同一三棵树、用 CPU 池级 checker 量（`model/probe_m1_ring.rs`，release，单线程，`logs/probe.txt`；第一趟用 4 线程并行时共用了进程环境变量、读数串了，作废，留作 `logs/probe-racy-threads4.txt`，不引用）。探针的造法逐段照抄甲补丁 `pool_written_under_these_knobs`（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14050`）。GPU 与 CPU 逐位一致这一半复核不了（要卡，且归 checker 档），原因写在这里。

### M1-a　三条新对拍是不是只因旋钮支判红，正确判法下有没有因别的理由红

结论：造法一致、断言只钉一半；另有一条已知的容量原因（退役）和一条余量薄的线索（墓碑）。

1. 断言钉什么：助手 `assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening`（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13043`）只有两句——CPU 带旋钮的新入口红过这一条旋钮不变量，和 `comparison.mismatches.is_empty(),`（`…:13066`，GPU 与 CPU 逐状态逐位相同）。它不钉「只红这一条」。所以「只因旋钮支红」由流的造法保证，不由断言保证。
2. 造法上确是只红旋钮那一条：探针里 52 个取样点、每点四种判法，红的不变量只有 I-9.22（墓碑）或 I-9.37（谱系、退役）一种，从不与别的同红（`grep '^PROBE' logs/probe.txt | grep -c '+'` 输出 0）；另两棵树没装旋钮，走产品支，不红。
3. 正确判法下的别的红法：GPU 一侧的容量。探针数到最终一版的分配记录条数与已释放条数（`alloc_records`、`released`，`logs/probe.txt`）：
   - 墓碑树 [2, 1, 2, 1]：第 30 次转根后 932 条分配记录、892 条已释放；
   - 谱系树：776、736；
   - 退役树：1050、1004，转根第 25 次是 950 条、第 30 次 1050 条，每转一次加 20，第 29 次起超过 1024。
   GPU 恢复内核的分配记录上限是 `const MAX_ALLOCATION_RECORDS: u32 = 1024u;`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:151`），池级 checker 已释放上限 `const C_MAX_RELEASED: u32 = 1024u;`（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:467`）。退役那条最后两次转根的状态超过 1024，与主 agent 在卡上看到的「退役 24 个状态 GPU 判不了」吻合；这一条正文第二节已列「已知」，不算打中。墓碑那条离 1024 只剩 92 条分配记录（再转 5 次根环就越线）、已释放离 1024 剩 132 条：这是一条推的线索（我数的是 CPU 一侧最终版的条数，GPU 逐状态的最大值没在卡上量过），今天绿，`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER` 若被调大到 35 左右就会翻（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:115`）。
4. 条款侧：「每个打包记录类型 1 容器记录数 ≤ 旋钮值」（`.claude/kb/invariants.md:310`）只写了不超旋钮值；「不是最右的容器 == 旋钮值」来自「满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」（`.claude/kb/decisions/18-块里携带什么信息.md:578`）。I-9.37 在 `invariants.md` 里没有行（`grep -c 'I-9\.37' .claude/kb/invariants.md` 输出 0）。判：规则没说（谱系树与退役树那一半），墓碑那一半一致。

推翻条件：卡上跑出墓碑或谱系那条的 `mismatches` 非空，或 CPU 红的不变量不止旋钮那一条。

### M1-b　转过根环后候选集里还剩哪几版，「只判倒数第二片」在哪一版上与正确判法分得开

结论：一致，分得开的窗口是转根的最后 8–9 次。

- 候选集：根环三个区域、每区 8 槽（`region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],`，`crates/singlefs-harness/tests/common/mod.rs:59`；`pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;`，`crates/singlefs-format/src/lib.rs:291`），共 24 槽，转根 30 次（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:115`）。第 t 个发布的根槽被第 t + 24 个盖掉。
- 各版的形（探针原样，`logs/probe.txt`）：

| 树 | 写第 4 条 | 第 5 条 | 第 6 条 | 转根 T = 写完 txg + j | 「只判倒数第二片」与正确判法第一次分开 |
|---|---|---|---|---|---|
| 墓碑 | [2, 1, 1] txg 12 | [2, 1, 2] txg 14 | [2, 1, 2, 1] txg 16 | T = 16 + j | j = 22（txg 38）；j = 21 仍同红 |
| 谱系 | [2, 1, 1] txg 8 | [2, 1, 2] txg 9 | [2, 1, 2, 1] txg 10 | T = 10 + j | j = 23（txg 33）；j = 22 仍同红 |
| 退役 | [2, 1, 1] txg 15 | [2, 1, 2] txg 18 | [2, 1, 2, 1] txg 21 | T = 21 + j | j = 23（txg 44）；j = 22 仍同红 |

  规律：候选窗口 [T − 23, T] 里不再有任何一版把容器 2 放在倒数第二片（[2, 1, 1]、[2, 1, 2]）时，「只判倒数第二片」才变绿，而正确判法在 [2, 1, 2, 1] 的每一版上都红在容器 2。所以对拍里真正分得开的是转根的第 22/23 到第 30 次那几次发布里的崩溃状态（8–9 个发布边界，加上这些发布内部的撕裂状态）；更早的状态里，候选集里还留着 [2, 1, 1] 或 [2, 1, 2] 的版本，改坏的判法照样红，分不开。余量：墓碑 8 次，谱系与退役 7 次；`PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER` 若降到 21 次及以下，墓碑那条的判别力归零；降到 22 次及以下，谱系与退役两条归零。
- 在 CPU 一侧复现这个改坏：探针把 `walk.rs` 的旋钮支换成只判倒数第二片（`crates/singlefs-checker/src/walk.rs:12153` 与 `…:12054` 两处），上表「分开」的点上它给 green、正确判法给 I-9.22/I-9.37 红。GPU 那一刀（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3357` 的 `if (d_container_record_count(container_index) != knob_records) {`）与这一刀同义，是同一结构（对每个非最右的容器判记录数 == 旋钮值，改成只在 `container_index + 2u == d_container_count` 时判）。GPU 一侧红不红我复核不了。

推翻条件：卡上把第 3862–3864 行（`crates/mutations.tsv`）打上之后，三条里有一条不红，或红的点不在上表的窗口里。

### M1-c　「建 30 次空 inode」会不会改变那棵容器树

结论：不会。探针里三棵树在转根第 1、12、19–25、30 次之后每片记录数都是 [2, 1, 2, 1]（`logs/probe.txt`）；用例自己在转完之后也断了每片记录数（`tree.record_counts_of_each_container(&pool),`，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14159`）。原因：建空 inode 的发布不写墓碑、谱系、退役记录，退役那一条里新建的 inode 落在新一片叶上、也没有被退役。

### M1-d　甲补丁的变异表三行与注释

- 第 3862–3864 行原文串在 `crash_judge_checker_scan.wgsl` 里恰好命中一次（`grep -cF` 输出 1，行号 3357）；三行逐字相同、点名的测试不同，三棵树共用这一个 WGSL 函数 `d_judge_containers_under_the_capacity_knob`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3346`），所以一刀改坏三条同时检验。变异命令带 `--release --features verdict-store,gpu`，需要卡，复核不了。
- 注释与同批另一份补丁不一致，不影响判定：甲补丁里写「与 harness 慢档 `checker_known_bad_images.rs` 里 `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那三条同形」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14147`），而乙补丁在同一批里把那三条的 `#[ignore]` 去掉了，`checker_known_bad_images.rs` 里 `#[ignore` 只剩第 17293、17345 行两处（产品容量那两条）。

## 二、M2　旋钮支离最右更远与慢档（乙 L2、L5）

### M2-a　[2, 1, 2, 2, 1] 三张盘是不是只由点名的那一种写错而红

结论：一致。

- 造法：旋钮 2、2、1、1、2、2、2、1 逐条换，写出 [2, 1, 2, 2, 1]，再转根环一圈（新常量 `A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST` 在 `crates/singlefs-harness/tests/checker_known_bad_images.rs:17091`，写序列在第 17093 行；第 58 批的 `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 在第 17079 行）。断言有两层：造完先断每片记录数 == `record_counts_in_leaf_order`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:17120`），再断「只红这一棵的那一条」且说明里带「不是最右的容器 2 装了 1 条，不是旋钮值 2」（`assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob`，`crates/singlefs-harness/tests/checker_known_bad_images.rs:16747`）。
- 我的探针（`model/probe_m1_ring.rs`，release，同一镜像用四种判法，单线程；`logs/probe.txt`）逐版量了这张盘：墓碑树写第 4–8 条后的每片记录数是 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1]、[2, 1, 2, 2]、[2, 1, 2, 2, 1]；转根环的第 1、12、19–25、30 次之后每片记录数始终是 [2, 1, 2, 2, 1]；52 行探针输出里红的不变量只有 I-9.22 或 I-9.37 两种，没有任何一行同时红别的（`grep '^PROBE' logs/probe.txt | grep -c '+'` 输出 0；`grep -o 'I-[0-9.]*@' … | sort | uniq -c` 输出 86 次 `I-9.22@`、96 次 `I-9.37@`，一行里只有一个）。
- 变异：把冻结副本 `crates/singlefs-checker/src/walk.rs:12153` 与 `…:12054` 两处「是不是最右」的判法换成「最右，或离最右超过两片」（与变异表第 3865–3867 行同一刀，我自己用 `l2.sh 2` 打）。命令 `l2.sh 2 two_one_two_two_and_one`，原样输出（`logs/l2-k2-two_one_two_two_and_one.log`）：「test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 264 filtered out」，三条都红在 `checker_known_bad_images.rs:16757` 的「只红 …」断言上（红的不变量列表变成空）。
- 变异表三行（`crates/mutations.tsv` 第 3865、3866、3867 行）原文串在冻结副本的 `walk.rs` 里恰好命中一次（我用 python 数过：七行新行各命中 1 次，列数都是 6）。第 3866、3867 两行的原文与替换文逐字相同、只是点名的测试不同；这在变异表里不新鲜（全表 290 组逐字相同的行）。

### M2-b　k = 3、4 的改坏在已有盘上哪一张分得开

结论：没有。以「离最右 d 片」量少一条的那片，现有盘是：`MIDDLE_CONTAINER_SHORT` [2, 1, 1] d = 1，`ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER` [2, 2, 1, 1] d = 1，`A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` [2, 1, 2, 1] d = 2，新盘 [2, 1, 2, 2, 1] d = 3。「只判最右之前 k 片」只判 d ≤ k 的容器，所以 k = 1 被 d ≥ 2 的盘分开，k = 2 被 d = 3 的盘分开，k = 3 要 d ≥ 4、k = 4 要 d ≥ 5，现有盘没有。

- 在新三条上跑：`l2.sh 3 two_one_two_two_and_one` 原样「test result: ok. 3 passed; 0 failed」（`logs/l2-k3-two_one_two_two_and_one.log`），`l2.sh 4 …` 原样「test result: ok. 3 passed; 0 failed」（`logs/l2-k4-two_one_two_two_and_one.log`）：k = 3、4 照绿。
- 版本级再看一次（探针，墓碑树 [2, 1, 2, 2, 1]，`logs/probe.txt`）：k = 3 的判法在转根环 1 到 30 次的每个取样点上都与正确判法同红；k = 2 的判法从第 22 次转根起（txg 42）与正确判法分开。
- 第 58 批判决第四节已写「K1-4 那一族（只判最右之前 k 片）有限组排布关不死；L2 把窗口推到 3」，是同一个结论，不算新打中。
- 整张 `knob` 过滤集（37 条，含所有旋钮容量用例）在 k = 3 下：`l2.sh 3 knob`，原样「test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 230 filtered out; finished in 1040.63s」（`logs/l2-k3-knob.log`；整机 load average 约 110，nice 19，挂钟不代表用例本身）。这个结果与上面的算术一致（现有最远的 d = 3）；k = 4 我只在新三条上跑了，没跑整张集。

### M2-c　去掉 `#[ignore]` 之后在快档里跑，用时会不会因别的用例的状态而变

结论：结果不变，挂钟会变。

1. 状态不共享。每条用例自己建池，镜像文件名带用例 tag、进程号和进程内计数（`crates/singlefs-harness/tests/common/mod.rs:48` 的 `image_path`），池丢掉时删文件（`crates/singlefs-harness/tests/common/mod.rs:99` 的 `Drop`）；六条的 tag 各不相同（`crates/singlefs-harness/tests/checker_known_bad_images.rs:17199`、`:17209`、`:17219`、`:17229`、`:17239`、`:17249`）。
2. CPU 时间不随负载变。六条各单跑，用户态 CPU 秒（`logs/m2-cpu-time.txt`，`bash time` 原样）：tombstone 2.223 / 2.266，lineage 2.145 / 2.394，retirement 2.372 / 2.559，系统态 0.10–0.14；挂钟 5.8–8.8 秒，其中一条 36.2 秒（测完时整机 load average 14.02，那一条被抢）。更早一趟单跑的挂钟是 2.43–7.99 秒（`logs/m2-single-*.log`）。
3. 挂钟随负载变，而且变得很大。同一台机器上跑整个 `knob` 过滤集 37 条（15 线程并行、nice 19、整机 load average 26–39）：原样「test result: ok. 37 passed; 0 failed … finished in 575.83s」（`logs/l2-k0-knob.log`），六条新/去掉标记的用例在那一趟里全部打了「has been running for over 60 seconds」。
4. 规则与判决 L5 在「怎么量」上并排：
   - 规则 `.claude/rules/verification.md:26`：「单条用时看平常跑的时候量到的，不另外单独量。」
   - 判决 `research/prompts/m3-batch58-code-r1-main-verification.md:40`：「J3 三条慢档在 debug 下各单跑一次量用时，不到 60 秒的去掉 `#[ignore]`」
   本批按判决单跑量了（最长 8.35 秒，实现员数）；我也按规则看了「平常并行跑」的挂钟，忙时超过 60 秒。判：冲突（两句各自成立，规则要平常并行跑的量，判决要单跑的量），第 58 批主 agent 记的「忙时几十上百秒是 nice 19 被抢 CPU」按 CPU 时间核是对的（每条 2–3 秒 CPU），但规则没说什么负载算「平常」。

推翻条件：(a) 找到一条只有合在一起跑才红的状态耦合（我只看了镜像文件名与 Drop，没扫这些用例之外的全局可变量）；(b) 在空闲机器上并行跑 `knob` 集仍超过 60 秒的单条。

## 三、M3　重建次序补钉与水位用例换造法（丙 L3、L4）

### M3-a　J1 ② 可写挂载那条分得出什么

结论（一致，带一处没钉到的半句）：新用例 `allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_writable_mount`（`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:2140`）只钉「重建把分配记录判排在记账条目逐条判之前」；「排在条目宽之前」这半句它钉不到，用例自己的注释也这么写（「这张盘每条记账条目宽都是 34」，`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:2116`）。

实现今天的次序（冻结副本 `crates/singlefs-core/src/recovery.rs`）：分配记录判代与跨度第 2894 行，条目宽第 2918 行，逐条判代与 seq 第 2923 行起，数记账条目第 2928 行，水位行第 2938 行，数映射条目第 2944 行；冷走读同一次序在第 5395 行起。

两个变异都在自己的拷贝上跑（原样输出在 `logs/`）：

1. 第 3868 行（`crates/mutations.tsv`，分配记录判挪到逐条判之后、数记账条目之前）。命令 `apply_row.py` 打上后 `cargo test -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root`，原样输出（`logs/m3-row3868.log`）：「test result: FAILED. 5 passed; 1 failed」，红的恰是 J1 ② 可写挂载那条，panic 说明「可写挂载报分配记录的代晚于根，不报记账条目的代：Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "记账条目的代晚于根或 seq 为 0" })」。另两条可写挂载（数记账条目、数映射条目）照绿。
2. 我自造的「分配记录判挪到条目宽之后、逐条判之前」一形（`logs/m3-after-width.log`，整份文件 41 条）：原样「test result: ok. 41 passed; 0 failed」。这一半没有任何用例钉着；判决 L3 原话是「分配记录判在条目宽与逐条判之前」，验收只点了「挪到条目宽与逐条判之后」那一形，所以验收达成，但「条目宽之前」这半句在可写挂载上无钉（可写挂载的窄条目用例 `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1572` 的分配记录是好的，换不出次序）。判：规则没说（kb 里没有写分配记录与条目宽谁先），判决验收一致，半句无钉记作线索。

推翻条件：找到一张盘，条目宽坏而分配记录代也晚于根，在「宽之后逐条判之前」那一形下某条用例红，则这半句已有钉。

### M3-b　第 3861 行换成的那一形是不是只红 J1 ① 可写挂载

结论：一致。冻结副本第 3861 行（现在的内容是「重建读完记账树先数映射条目、再判分配记录」）：

- 过滤 `allocation_records_whose_generation_is_after_the_root` 的六条里只红 `…_are_refused_before_the_mapping_entry_count_by_the_writable_mount`（`logs/m3-row3861.log`：「5 passed; 1 failed」，说明「可写挂载报分配记录的代晚于根，不报映射条目数：Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "映射条目数不是 1 + …」）。
- 整份 `corrupt_on_disk_content_is_refused_instead_of_panicking`（41 条）里也只红这一条（`logs/m3-row3861-fullfile.log`：「40 passed; 1 failed」）。
- J1 ② 可写挂载那条在这一形下绿，因为映射条目数在这张盘上是对的；所以第 3861 行与第 3868 行各自只红一条、红的不是同一条，两行不重叠。

推翻条件：整份文件里另有一条在这一形下红（本腿跑的是这一个测试目标；别的测试目标里碰重建次序的用例没扫）。

### M3-c　L4 新造法：还钉在「重建要这一版自己那行水位、不认台阶顶替」上吗

结论（一致，带一处新造法比旧造法多出来的脆点）。用例 `a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount`（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:552`）。

量法：自己的拷贝上，`l4.sh <变异名> <new|old>`（new 是冻结副本里的文件，old 是 `git show HEAD:` 取出的第 58 批之前那份，靠标签 13）。全部原样输出在 `logs/l4-*.log`。r1、r2 是我照判决 L4 验收栏的字面（拒标签 > 12、数条目时不算未登记标签）自己写的两处读者改法（`model/mut/`），不是攻方原型。

| 变异（打在 `crates/singlefs-core/src/recovery.rs`） | 旧用例（标签 13） | 新用例（补 (1, 0, 0, 3)） |
|---|---|---|
| 无（基线） | 1 passed | 1 passed（`logs/l4-none-new.log`） |
| 第 3668 行：水位行只看统计量，不看代段（只剩台阶也照建） | 本腿没跑 | **红**，`core_review_…:627` 的 `assert_eq!` 失败，「left: None」（`logs/l4-row3668-new.log`） |
| r1：逐条判再拒统计量 > 12（`recovery.rs:4876` 那句条件上再加一个「或 `entry.statistic > STATISTIC_INODE_WATERMARK`」） | **红**，left 是「记账条目的代晚于根或 seq 为 0」（`logs/l4-r1-reject-label-above-12-old.log`） | **绿**（`logs/l4-r1-reject-label-above-12-new.log`） |
| r2：数记账条目时不算统计量 > 12 的行（`recovery.rs:4795` 那句条件上再加一个「或 `entry.statistic > 12`」） | **红**，left 是「记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数」（`logs/l4-r2-count-skips-label-above-12-old.log`） | **绿**（`logs/l4-r2-count-skips-label-above-12-new.log`） |
| r3a：重建在水位判之前逐行判「非台阶行的代段 == 这一版的 txg」（I-3.13 搬进重建） | 本腿没跑 | **红**，`core_review_…:627`，left 是 `InvariantViolated { invariant: "I-3.13", detail: "记账行代段不等于这一版的 txg" }`（`logs/l4-r3a-rebuild-judges-i313-before-watermark-new.log`） |
| r3b：同一判放在水位判之后 | 本腿没跑 | **绿**（`logs/l4-r3b-rebuild-judges-i313-after-watermark-new.log`） |

读法：

1. 第 58 批判决 L4 的验收两半都成立：第 57 批那两种读者改法（r1、r2）旧用例红、新用例仍报「水位行缺」；原判别力那一形（第 3668 行）新用例红。
2. 补进来的 (1, 0, 0, 3) 不被别的判先拒：重建今天对每条记账条目只判代不晚于根、seq 不为 0（转述；`crates/singlefs-core/src/recovery.rs:4876`），数条目只把统计量 12 且代早于根的台阶行排除（转述；`crates/singlefs-core/src/recovery.rs:4795`），这一行统计量是 1、代 3、seq 取自上一版原样，三判都过；基线绿也是证明。「水位行缺」报出的理由对：失败值逐字等于 `InodeNumberWatermarkRowMissingFromTheAccountingTree`（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:632` 的断言说明；可写挂载那一半在同一用例后面也比这个成员）。
3. 条款：这一行在 txg 4 那一版里违反 I-3.13——「走读到的任一版」（`.claude/kb/invariants.md:155`）的记账行代段要等于这一版的 txg，台阶行除外；它的代段是 3。D5 的口径是「每条根的记账树只装这条根那一代的行」（`.claude/kb/decisions/05-快照-空间记账机制.md:47`）。用例注释自己写明了这一点，也写明「重建今天不判那一条（池级 checker 判）」（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:545`）。
4. 读者将来逐条判 I-3.13 时它红在哪：r3a 一形给了答案，红在 `core_review_…:627` 那条 `assert_eq!`（重建结局），报的不再是水位行缺而是 I-3.13；判放在水位判之后（r3b）就不红。旧造法（标签 13、代 4，代段等于根的 txg）按 r3a 的写法不会被拒（推的，本腿没跑旧用例 + r3a），新造法受影响，这是新造法比旧造法多出来的耦合；将来把 I-3.13 搬进重建的人要么放在水位判之后，要么同时改这条用例。判：规则没说（kb 没写重建要不要判 I-3.13，也没写它与水位行谁先）。
5. 「标签 > 12 没登记」这句的口径：D5 已定项 10 写「码 = 行号（1 到 14；第 8 / 9 项退役不复用；0 保留为无效）」（`.claude/kb/decisions/05-快照-空间记账机制.md:244`），13、14 在登记表里、只是「这两项第一版不登记」（`.claude/kb/decisions/05-快照-空间记账机制.md:137`）；读者碰到不认识的标签怎么办，kb 明说不管：「不管一个老读者读到不认识的标签时该怎么办」（`.claude/kb/decisions/05-快照-空间记账机制.md:246`）。所以新注释里「读者逐条判拒标签 > 12、数条目时不算标签 > 12 的行」是对读者可能怎么改的假设（第 57 批攻方的两种改法），不是 kb 条款；换成统计量 1 的行在这个口径下更稳。判：规则没说。

推翻条件：(a) 重建里已经有一处逐行判代段 == 根 txg 的代码而我没看到（那样基线就不会绿，基线绿是反证）；(b) 另有一种读者改法让 (1, 0, 0, 3) 先被拒而旧造法不被拒，例如按 (统计量, 树, 设备) 三元判「同一维度元组只许一行」——这一形我没试，列为线索。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| M1-a 只因旋钮支红、别的红法 | 一致（断言只钉一半）；规则没说（I-9.37 无 kb 行） | 造法上只红 I-9.22 / I-9.37；退役最后两次转根 GPU 记录数超 1024（已知）；墓碑离 1024 只剩 92 条（推的） |
| M1-b 候选集与分得开的版本 | 一致 | 候选窗口 24 槽；改坏与正确判法只在转根第 22（墓碑）/ 23（谱系、退役）次之后分开，窗口里不再有 [2,1,1]、[2,1,2] 版 |
| M1-c 30 次空 inode 改不改树 | 一致 | 每片记录数在各取样点恒为 [2,1,2,1]，用例自己也断言 |
| M1-d 变异表三行 | 一致（GPU 红绿复核不了） | 原文串命中 1 次；三行同一刀；注释里「harness 慢档」与乙补丁去标记不一致 |
| M2-a [2,1,2,2,1] 只因点名写错而红 | 一致 | 只红 I-9.22 / I-9.37；k=2 改坏下三条全红 |
| M2-b k=3、4 的分得开 | 没有盘分得开 | 现有最远 d = 3，k=3、4 在新三条上照绿，k=3 在整张 knob 集 37 条上照绿；判决第四节已记有限组关不死 |
| M2-c 去标记后的时间 | 结果一致；L5 与规则冲突 | CPU 每条约 2–3 秒；并行忙时挂钟超 60 秒；规则「不另外单独量」与判决 L5「各单跑一次量用时」并排，规则没说什么负载算平常 |
| M3-a J1 ② 可写挂载分得出什么 | 一致，半句无钉 | 只钉「分配记录判在逐条判之前」，「条目宽之前」这半句没有盘钉（我造的一形整份 41 条全绿） |
| M3-b 第 3861 行只红 J1 ① | 一致 | 六条里只红 J1 ① 可写挂载，整份 41 条也只红这一条；第 3868 行（过滤六条内）只红 J1 ② 可写挂载 |
| M3-c L4 新造法 | 一致，多一处耦合 | 第 57 批两种读者改法下新用例绿、旧用例红，第 3668 行下新用例红；重建将来在水位判之前逐行判 I-3.13 会红（补进的行代段 3 ≠ 4） |

## 五、没做什么与复核不了

- 复核不了：GPU 三条对拍（`the_gpu_judge_given_the_{tombstone,lineage,retirement}_knob_two_agrees_…_with_the_root_ring_turned_over`）的红绿、变异表第 3862–3864 行的红绿、GPU 一侧的分配记录条数上限是否真在 1024 附近越线。原因：checker 档测试与带 GPU 的变异归主 agent，本腿不跑；版本级事实用 CPU 池级 checker 复现，复现不替 GPU 作证。
- 没做：门禁阶段、crates 变异整表的轻型、`mutate.sh`；没改冻结副本；没替主 agent 采纳或出判决。
- 没试：D5「同一维度元组只许一行」那一形读者改法（M3-c 推翻条件 (b)），对其余 `rebuild` 调用点（`recovery.rs:5395` 起的冷走读）做同样的次序变异；M2 的整份 `checker_known_bad_images`（267 条）基线在高负载下超过一个命令时限，我改跑 `knob` 过滤集（37 条）。
- 读了但不当依据：第 58 批判决第三节 L1–L5 当题面用；`mutations.tsv` 以冻结副本为准。
- 清理：见交回正文，自己建的 `~/.local/share/singlefs/work/m3-batch59-code-r1-sonnet/target` 与 `…/repo` 在交回前删，`/tmp` 下我杀掉的两个测试进程留下的 32 个镜像文件已删。

## 六、命令清单（原样输出都在 `research/prompts/m3-batch59-code-r1-sonnet-model/logs/`）

- 变异：`apply_row.py` 打第 3868、3861 行；`l4.sh`（M3-c）；`l2.sh`（M2）；`probe_m1_ring.rs` + `probe3.sh`（M1，第 3 版，单线程 release）。
- 要主 agent 跑的命令：GPU 三条对拍与第 3862–3864 行变异，照 `.claude/rules/verification.md`「两档：轻型与重型」那一节起；我没有改它们的命令。
