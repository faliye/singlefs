# m3-formal-code-b21-r1 云端攻方（Opus）报告（2026-10-04）

攻击面：K2（F3 的子集指纹、改法 a、改法 b）、K3（第四种接错按「支」算还清对不对）。代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b21-r1/tree/`，下文 `crates/…` 路径都指那棵树。量出来的数都是我自己拷出的副本上的数（副本 = 冻结副本的 crates/ 加仓根 Cargo.toml、Cargo.lock，打上 `probe/copy-probe.patch` 的几个环境变量开关，再加一个只在副本里的探针包 `opus-b21-probe`）；没跑 checker 档包的任何测试目标，探针包自己的二进制链着 checker 档的库跑（代理定义 3b 那一条允许的原型用法，见末节「限度」）。

## 复跑

模型目录 `research/prompts/m3-formal-code-b21-r1-opus-model/`。复跑（仓根执行；`WORK` 指一个空的草稿目录；全程约 5 分钟，编译约 1 分钟，各段都在 4 分钟以内）：

```
WORK=/tmp/claude-1000/<草稿>/b21-opus-rerun THREADS=7 bash research/prompts/m3-formal-code-b21-r1-opus-model/tools/rerun.sh all
```

可覆盖的环境变量：`REPO_ROOT`（默认按脚本位置推出仓根）、`FROZEN_TREE`（默认 `/tmp/claude-1000/m3-formal-code-b21-r1/tree`）、`WORK`、`THREADS`（默认 7）、`BUILD_CAP`（默认 8G）、`RUN_CAP`（默认 4G）。也可以分段：`setup`、`build`、`k3`、`k3-sweep`、`k2a`、`k2b`、`k2b-sweep`、`f3`。产物写进 `$WORK/out/`，与模型目录 `out/` 下同名文件逐字节比（`cmp`）。我在 `WORK=/tmp/claude-1000/m3-formal-code-b21-r1-opus/rerun` 上从头跑过一遍 setup、build 与全部六段，`out/` 下的文件就是那一遍的产物（k2b-sweep 改成每批 6 件之后又跑过一遍，`cmp` 相同）。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`）：

```
694998b3b3bc3018dc6ae728439b73174212a39bba807b8828747b4d4d34532c  research/prompts/m3-formal-code-b21-r1-opus-model/out/f3.txt
aa10a7204ac9b2708cf0ddedcc938f281f2d58a3d217549617a0337a94f9f972  research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt
d0dc5adabb055c74b3957ae35659b16c55b6e266cdc7e8abd33fa01f663e6c8e  research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b-sweep-classes.txt
ed2d8c71f677996f6334cfc0e7db5fcf825613a4ff3c99c6c9218ca3716a83cc  research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b-sweep.txt
65d0ecce6e2329b3fb8326a9947b596a66483ddf53d0688ffc2e8a7125ed6717  research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b.txt
042bfdfc12e40dc81fb63fc5ad140a11419a773a572f2e5143eed5bb48557b2a  research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt
dc2a65fe3b5d964a4ff387204f94a2b8330728d09d1eb0ec68b142e81ee5ce2b  research/prompts/m3-formal-code-b21-r1-opus-model/out/k3.txt
9a18bf9aef44a37ff9f97f81866adc6109b00b94b2d9c66acf44728661275068  research/prompts/m3-formal-code-b21-r1-opus-model/probe/copy-probe.patch
eb522f4abdc639f13f859ff728bf648a45354d178f3aebe5f25780b643e0785e  research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus-b21-probe/Cargo.toml
01f8fb4e369dad824e58bc9f8264c6028c4e6245a77b2bff317a688e3019c025  research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus-b21-probe/src/main.rs
042726c11350212fb4e05e8117dcb386cf1662f25c213db628b4823a9ec5f83d  research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs
74c80b45336454c1f7ca19c5b89c14d601d4537dc98e2e4592e71e3d5c4b8770  research/prompts/m3-formal-code-b21-r1-opus-model/tools/rerun.sh
7683e23d61a1aa6e5b4a54923be0b1ae963ec209062562385c28fe3a4a8b79ed  research/prompts/m3-formal-code-b21-r1-opus-model/tools/summarize_k2b.py
```

副本上的开关（`probe/copy-probe.patch`，不设环境变量时副本行为与冻结副本相同）：`OPUS_K3_WIRING=first` 把第三截调用点交的所选那一版换成第一次崩溃那一版（与 `crates/mutations.tsv` 第 21 批那一行同义）；`OPUS_K2A_VARIANT=<坏发布>` 只在第二截「带文件而第一个文件不是活记录」那一支的调用里生效；`OPUS_K2B_SOME=1` 让判法只看 `inode_record` 是不是 Some（与 mutations.tsv 改法 b 那一行同义）；`OPUS_F3_FP=<指纹>` 换指纹函数。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| K2-F3 取样会不会变窄 | 没打中 | 量过（副本） | 2,097,120 个（段号 xor 指纹）两两不同；同一段全部子集摆出的二次崩溃并集从 3 个涨到 16 至 2359 个；SplitMix64 流之间最小平移 ≥ 2^22.7 字 |
| K2-F3 内联用例的判别力 | 打中（弱） | 量过（副本） | 指纹只取第一个写、或只取第一个被扣下的写的下标时内联用例照绿，而 4095 个第一次崩溃态只剩 2 串 / 12 串随机数 |
| K2-a 断言放过哪些坏发布 | 打中 | 量过（副本） | 新 inode nlink = 0、出生代早一版、size 4096、写入时间 0 四种坏发布：发布做成、第二截没有新发现、池级 checker 全绿、改法 a 的断言绿；写入时间那一种是发布臂自己传的参数 |
| K2-b 真走 nlink = 0 那一支、只看 Some 真报 Unexplained | 成立（没打中） | 量过（副本） | 正确判法走建 inode，只看 Some 时覆盖写被拒成 `WriteToAnInodeBeingDeleted { inode: 1 }`、`Unexplained` |
| K2-b 拒的来历 | 打中 | 量过（副本） | 这份盘面上拒的是空间准入（可用 0），建 0 个 inode 也被拒成 `Explained(UnitAreaWall)`；用例分不出「建一个」与「一个都不建」，同族 136 槽盘面分得出 |
| K3「判据对所选那一版单调、换成更旧那一版只会真到真」 | 打中 | 量过（副本） | 750 个越屏障的组合里 9 个：正确接线 (判过, 不合法) = (1, 1) 报红，接错 = (1, 0) 不报红；所选那一版换成更旧的那一版，判据由真变假 |
| K3 在不越屏障的枚举域里 | 不分 | 推的 | 不越屏障时第三截走不到判据 N-配置 的拒，两种接线计数都是 0（没量） |

## K3　第四种接错按「支」算还清对不对

### 被攻的那一句

`research/prompts/_m3-formal-code-b21-r1-body.md@一、题面`：「找到判官结论只变在 C670 成立那一支的态，合法与否与计数分不开；用例与变异照「支」这一层写。」之后的括注「读码推：判据对所选那一版单调，换成更旧那一版只会真到真」。

### 判据在代码里怎么随所选那一版变

判官比 c_见证 与所选那一版有三支（`crates/singlefs-harness/src/mount_transparency.rs`）：

- `crates/singlefs-harness/src/mount_transparency.rs:1454`：「if witnessed_journal_counter == 0 {」
- `crates/singlefs-harness/src/mount_transparency.rs:1468`：「return (witnessed_journal_counter > last_record_counter).then_some(」——所选那一版的末条读得出就只比计数器。
- `crates/singlefs-harness/src/mount_transparency.rs:1478`：「Some(record) => (record > selected_version).then_some(」
- `crates/singlefs-harness/src/mount_transparency.rs:1481`：「None => Some(」——两条都读不出按真。

「单调」要的是：较新那一版判真 ⇒ 较旧那一版也判真。第三支打破它：较新那一版 Vn 的末条读不出、计数器等于 c_见证 的记录也读不出 ⇒ 真；较旧那一版 Vo 的末条读得出、而 c_见证 不大于它的计数器 ⇒ 第一支判假。要两样同时成立，c_见证 指的那条记录得比 Vo 的末条旧、又读不出——也就是 c_见证 落后于 Vo（Vo 那次发布的系统配置轮换没落）、而 c_见证 指的那次更早发布的记录丢了。

第三截只认 C670 那一条（`crates/singlefs-checker-tier/src/crash_injection.rs:3228`：「（r5 判决第二节 U6）条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法，判官列出的没满足的条件只随 C670 那一条的红带出来。」），不合法只在 C670 那一条成立时计（`crates/singlefs-checker-tier/src/crash_injection.rs:3339`：「tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;」）。所以判据在所选那一版上由真变假，接错的那一版就既不报红、也不计不合法。

### 造出来的态（量过，副本）

历史与入库 K3 用例同一段（起点之后一次覆盖写，单元区 384 槽小盘），写表 83 个写：第一个文件那次发布的记录是写 53、54，根 55，轮换 56、57；覆盖写的记录 78、79，根 80，轮换 81、82（`out/k3.txt` 第 2、3 行）。

- 第一次崩溃（越屏障，与入库用例的二次崩溃同一类）：从第 13 段第一个写（写 53）起数 28 个写、摆到覆盖写的根 80 为止，扣下段内第 0、1 个（第一个文件那次的两份记录）；覆盖写的两槽轮换没落。只读恢复落在覆盖写那一版 Vo = (1, 4)。
- 崩溃后镜像上可写挂载与之后那一次发布都做成。
- 二次崩溃：挂载写表第 2 段（写 16 起）数 3 个写到写行的根 18 为止，扣下写行的两份记录 16、17；写行的两槽轮换没落。只读恢复落在写行那一版 Vn = (2, 5)。

推的（读码，没把 c_见证 打出来）：c_见证 = 第一个文件那次末条的计数器——它的轮换 56、57 落了；取号那一写带的是取号那一刻读到的同一个见证值；覆盖写与写行的轮换都没落。Vn 的末条丢了、第一个文件那次的末条也丢了 ⇒ 第三支为真；Vo 的末条读得出、计数器比 c_见证 大 ⇒ 第一支为假。下面扫的结果与这个推法逐格对得上（只在「第一个文件那次两份记录都扣、覆盖写的轮换没落、写行的轮换一槽都没落」时出差别）。

`research/prompts/m3-formal-code-b21-r1-opus-model/out/k3.txt` 第 31、34、35、36 行（整行）：

```
first crash: segment=13 first_write=53 count=28 withheld=[0, 1]
second crash: segment=2 first_write=16 count=3 withheld=[0, 1]; Vo=Some((InstanceGeneration(1), CheckpointTxg(4))) Vn=Some((InstanceGeneration(2), CheckpointTxg(5)))
wiring=second: (crash_points, mounts_ok, read_fault_states)=(1, 1, 0) second_crash_points=1 (judged, illegal)=(1, 1) remounts_after_second=1 remount_findings_branches=["NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable"] other_stage_findings=["CrashImage", "AfterTheWritableMountAndOnePublish", "SecondCrashInsideTheWritableMount(RowPublish)"]
wiring=first: (crash_points, mounts_ok, read_fault_states)=(1, 1, 0) second_crash_points=1 (judged, illegal)=(1, 0) remounts_after_second=1 remount_findings_branches=[] other_stage_findings=["CrashImage", "AfterTheWritableMountAndOnePublish", "SecondCrashInsideTheWritableMount(RowPublish)"]
```

`wiring=second` 是冻结副本的接线（`crates/singlefs-checker-tier/src/crash_injection.rs:3041`：「version_landed_on_after_the_crash: report.effective_root,」），`wiring=first` 是 mutations.tsv 第 21 批那一行的接法。同一个态：正确接线计 (1, 1)、报一条第三截的红（第三支）；接错计 (1, 0)、第三截一条红都不报。入库用例那个态在同一个探针上照样复现「计数相同、只差那一支」（`out/k3.txt` 第 8、9 行），探针与入库用例对得上。

### 扫（量过，副本）

第一次崩溃：从写 53 起摆到覆盖写的根 80 或摆到写表末尾 82，扣下 {53, 54, 56, 57} 的任一非空子集（30 个）；二次崩溃：从写行的记录起摆到根、根加一槽轮换、根加两槽轮换，扣下写行的记录与摆到了的轮换的任一非空子集（25 个）。共 750 个组合，每个两种接线各跑一次。`out/k3-sweep.txt` 第 7、11、12、13 行（整行）：

```
k3-sweep combos=200 same=162 branch_only=32 counts_differ=6 first_crash_mount_not_published=0 (ff_records=[53, 54] ff_rotation=[56, 57] ow_root=80 writes=83)
k3-sweep combos=175 same=144 branch_only=28 counts_differ=3 first_crash_mount_not_published=0 (ff_records=[53, 54] ff_rotation=[56, 57] ow_root=80 writes=83)
k3-sweep combos=200 same=168 branch_only=32 counts_differ=0 first_crash_mount_not_published=0 (ff_records=[53, 54] ff_rotation=[56, 57] ow_root=80 writes=83)
k3-sweep combos=175 same=147 branch_only=28 counts_differ=0 first_crash_mount_not_published=0 (ff_records=[53, 54] ff_rotation=[56, 57] ow_root=80 writes=83)
```

9 个计数不同的组合全是同一个方向（正确 (1, 1)、接错 (1, 0)），全落在「第一次崩溃摆到 80（覆盖写轮换没落）、写 53、54 都扣、二次崩溃里写行的记录与摆到了的轮换全扣」；120 个只差那一支，621 个完全相同。没扫到接错多报一条红的方向。

### 打中之后的四句

1. 分不分辨：分。判决把「按支写」当还清的理由是「计数分不开」；这里计数分得开，就能写计数层的用例（下一节），两条路在这一格上不同。
2. 被判的系统当时看不看得到：看得到。调用点同时拿着两样（第一次崩溃之后读出的字段与二次崩溃镜像的恢复报告），判官拿到哪一版就按哪一版比。
3. 满足的是哪一个分句：「换成更旧那一版只会真到真」——Vo = (1, 4) 比 Vn = (2, 5) 旧，判据由真变假；连带「合法与否与计数分不开」里的「计数」那半句。「合法与否」那半句没打中（推的，读码，判官结论这一枚举没打出来）：两种接线都计一次判过，判官结论按代码都是不合法——正确接线差 C670 那一条，接错差「判据复算为假」那一条（`crates/singlefs-harness/src/mount_transparency.rs:1682`：「.push(RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);」），只是第三截只认前一条。
4. 跑前条款给的改法在这几格上还中不中：入库用例与 mutations.tsv 那一行照样有效——那一行在入库用例的态上按支判红，不受这一格影响。受影响的是理由：接错不只是换一个支名，它会让第三截漏报一条 C670 的红。

### 改法（只在我的模型上量过、被攻过零轮）

K3-a：在入库用例旁加一条计数层的靶向用例，态取上面那一个（`CrashPointDraw::ExactlyOneCrashState`：第一个文件那次记录所在段、数到覆盖写的根、扣下那两份记录；`SecondCrashPointDraw::ExactlyOneSecondCrashState`：写行记录所在段、数到写行的根、扣下写行的两份记录），断言 (判过, 不合法) = (1, 1) 且第三截恰好一条新发现。量过：正确接线 (1, 1) 一条红，mutations.tsv 那一行的接法 (1, 0) 零条红（上面第 35、36 行）。这一态在第一截、第二截与二次崩溃那一截也有红（`other_stage_findings`），用例要像入库用例那样只挑第三截的新发现。

K3-b：判决与 kb 里不写「判据对所选那一版单调」这句理由；把「接错会漏报 C670 的红」写进那一行变异的说明。推的，没实现。

## K2　F3 的子集指纹

### 会不会撞到同一串随机数、让取样变窄：没打中（量过，副本）

种子是四样异或（`crates/singlefs-checker-tier/src/crash_injection.rs:2831`：「^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),」），随机源是 SplitMix64，状态每取一个字加一次 γ（`crates/singlefs-harness/src/history.rs:267`：「self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);」）。两个种子会摆出相关的数只有两种情形：异或之后相等，或者两者之差是 γ 的小倍数（一条流是另一条错开几个字）。模型（`main.rs` 的 `f3_arith`，调的是副本里真的指纹函数）按两种情形各量一遍：段号 0..256、第 s 段长 1 + s mod 16、每段全部 2^长 个子集（连全持久也算进去，取上界），一共 2,097,120 个（段号 xor 指纹）。

`research/prompts/m3-formal-code-b21-r1-opus-model/out/f3.txt` 第 1–7 行（整行）：

```
f3-arith keys(segment xor fingerprint)=2097120 distinct=2097120
f3-arith lowest-bit-is-parity=true
f3-arith seed=0 min_shift_between_two_second_crash_streams=25930147 (log2 24.6) min_shift_to_history_or_first_crash_stream=757415841349 (log2 39.5)
f3-arith seed=7 min_shift_between_two_second_crash_streams=8717761 (log2 23.1) min_shift_to_history_or_first_crash_stream=166772837279 (log2 37.3)
f3-arith seed=32768 min_shift_between_two_second_crash_streams=30961971 (log2 24.9) min_shift_to_history_or_first_crash_stream=757415841349 (log2 39.5)
f3-arith seed=245760 min_shift_between_two_second_crash_streams=28738137 (log2 24.8) min_shift_to_history_or_first_crash_stream=1804440819279 (log2 40.7)
f3-arith seed=3735928559 min_shift_between_two_second_crash_streams=7039770 (log2 22.7) min_shift_to_history_or_first_crash_stream=2787838841644 (log2 41.3)
```

一次摆法最多取「6 段 ×（1 + 段长）」个字，这份挂载写表上不到 40 个；两条二次崩溃流之间最近也错开约 700 万个字，与历史那一路、第一次崩溃那一路的流错开 2^37 以上。指纹的最低位就是子集里持久个数的奇偶（第 2 行；FNV-1a 只喂 0/1 时低位弱），进 SplitMix64 之前与段号、种子异或，SplitMix64 的输出函数把它搅开，在上面两种情形里都没有显出来。

真的 `draw_second_crash_points` 在一份真的挂载写表（入库 K3 用例的第一次崩溃之后那一次挂载，38 个写、9 段）上，同一段全部真子集各摆一次，比混指纹与不混（持久集合交空，等于 mutations.tsv 那一行的变异）两种。`out/f3.txt` 第 10 行（整行）：

```
f3 seed=0 segment_index=16 segment_length=12 first_crash_states=4095 mount_writes=38 mount_segments=9 writes_per_phase={Acquisition: 2, RowPublish: 19, WarmUp: 17} distinct_seeds=4095 distinct_draw_sets_with_fingerprint=3579 union_second_crash_states_with_fingerprint=2359 union_without_fingerprint=3
```

不混时 4095 个第一次崩溃态摆到的二次崩溃并集只有 3 个，混了是 2359 个；第 8、9、11、12 行四组参数同向（16 / 286 / 2348 / 828 对 3）。摆法有相同的（3579 < 4095）是因为每段可挑的写只有 2、19、17 个，结果撞在一起，不是随机数撞在一起（种子 4095 个两两不同）。什么现象会推翻「不变窄」：同一段里两个不同子集的种子相等，或者混指纹之后并集比不混小。两样在这里都没出现。

### 内联用例的判别力：打中（弱，量过，副本）

内联用例只比一对子集（`crates/singlefs-checker-tier/src/crash_injection.rs:4355`：「let first_write_landed = first_crash_with(vec![true, false, false]);」，`crates/singlefs-checker-tier/src/crash_injection.rs:4356`：「let second_write_landed = first_crash_with(vec![false, true, false]);」），两者第 0 个写就不同。只看子集开头的弱指纹也能让它绿。`out/f3.txt` 第 13–22 行（每种指纹两行：内联用例那一对的结果，同一段 4095 个子集上的并集）：

| 指纹 | 内联用例 | 4095 个第一次崩溃态里不同的种子 | 二次崩溃并集 |
|---|---|---|---|
| FNV-1a（入库） | 绿 | 4095 | 2359 |
| 空子集（mutations.tsv 那一行） | 红 | 1 | 3 |
| 只取第一个写持久没有 | 绿 | 2 | 6 |
| 持久个数 | 红 | 12 | 27 |
| 第一个被扣下的写的下标 | 绿 | 12 | 27 |

四句：分辨——分辨「用例钉住整个子集都进种子」与「只钉住种子不是常量」；系统看得到——指纹是纯函数，输入就是子集；分句——内联用例说的判别力只对登记的那一行（换成空子集）成立，对「子集各抽各的」这句（`crates/singlefs-checker-tier/src/crash_injection.rs:4381`：「"同一历史种子、同一段号、段内子集不同的两个第一次崩溃态要摆出不同的二次崩溃"」）只证了一对；改法——登记的变异照红，这一格不碰它。

改法 F3-c（只在我的模型上量过、被攻过零轮）：内联用例再加一条，长 12 的段全部 4095 个真子集的种子两两不同（或指纹两两不同）。量过：入库指纹 4095 个，上表三种弱指纹是 2、12、12，空子集 1。

## K2　改法 a：断言放过的坏发布

改法 a 只比 inode 号（`crates/singlefs-checker-tier/src/crash_injection.rs:4856`：「inode_numbers_of_the_crashed_version.is_subset(&inode_numbers_after_the_publish)」，`crates/singlefs-checker-tier/src/crash_injection.rs:4858`：「== inode_numbers_of_the_crashed_version.len() + 1,」）。我在副本里让第二截那一支（只那一支，历史里的建 inode 不受影响）发出十种发布，照入库用例同一段历史、同一个崩溃态跑注入，再在同一份盘面上挂一次发一次，读回每条 inode 记录的 (号, 出生代, nlink, size)，并对发布之后的池跑池级 checker。`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt` 第 1、2、8–11 行（整行）：

```
variant=correct: outcome=Published (publishes, refused)=(1, 0) findings_after_mount=0 assertion_a=true crashed(inode,birth,nlink,size)=[(2, 6, 1, 0)] after=[(2, 6, 1, 0), (3, 9, 1, 0)] checker_violations_on_pool_after=[]
variant=zero: outcome=Published (publishes, refused)=(1, 0) findings_after_mount=0 assertion_a=false crashed(inode,birth,nlink,size)=[(2, 6, 1, 0)] after=[(2, 6, 1, 0)] checker_violations_on_pool_after=[]
variant=being-deleted: outcome=Published (publishes, refused)=(1, 0) findings_after_mount=0 assertion_a=true crashed(inode,birth,nlink,size)=[(2, 6, 1, 0)] after=[(2, 6, 1, 0), (3, 9, 0, 0)] checker_violations_on_pool_after=[]
variant=old-birth: outcome=Published (publishes, refused)=(1, 0) findings_after_mount=0 assertion_a=true crashed(inode,birth,nlink,size)=[(2, 6, 1, 0)] after=[(2, 6, 1, 0), (3, 8, 1, 0)] checker_violations_on_pool_after=[]
variant=size: outcome=Published (publishes, refused)=(1, 0) findings_after_mount=0 assertion_a=true crashed(inode,birth,nlink,size)=[(2, 6, 1, 0)] after=[(2, 6, 1, 0), (3, 9, 1, 4096)] checker_violations_on_pool_after=[]
variant=time-zero: outcome=Published (publishes, refused)=(1, 0) findings_after_mount=0 assertion_a=true crashed(inode,birth,nlink,size)=[(2, 6, 1, 0)] after=[(2, 6, 1, 0), (3, 9, 1, 0)] checker_violations_on_pool_after=[]
```

绿着放过的四种：新 inode nlink = 0（正在删、却不在 `deleted_inodes` 里），出生代比建它那次发布早一版（8 而不是 9；代码自己写的是「对象出生代 = 创建那次发布的 checkpoint_txg」，`crates/singlefs-core/src/transaction.rs:5806`：「// 对象出生代 = 创建那次发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 8）。」），size 4096（`crates/singlefs-core/src/transaction.rs:5809`：「// 建出来还没写过数据：长度 0，blocks 跟着是 0。」），写入时间 0。四种在这份盘上都是「发布做成、第二截没有新发现、池级 checker 一条都不红、改法 a 绿」。被抓到的（`out/k2a.txt` 第 2、3、12、17、22 行）：建 0 个（改法 a 红）、旧实例代号（I-3.8、I-4.2、I-8.6、I-8.7、I-8.8 红）、号跳一个（I-9.6 红）、改动计数 0（I-9.26 红）、复用 inode 1（实现拒成 `InodeTreeWriteRefused`、第二截新发现）。

nlink = 0 那一种池级 checker 不红，是因为 I-9.5 只管一个方向（队列里的号 ⇒ 树里 nlink = 0；`.claude/kb/invariants.md` 第 292 行），反方向（nlink = 0 ⇒ 在队列里）没有条目。

四句：分辨——分辨「断言钉住建出一个 inode」与「断言只钉住号多一个」；系统看得到——用例手里有发布之后的整份池，读得出整条记录；分句——K2 题面「改法 a 的断言在建出一个 inode 之外的哪些坏发布下仍然绿」；改法——改法 a 本身对这四种不起作用。

哪一层的坏：写入时间是发布臂自己传的参数（`crates/singlefs-checker-tier/src/crash_injection.rs:1887`：「FIXED_WRITE_TIME_SECONDS,」），臂传错了这条用例看不见；另三种是 `publish_new_inodes` 自己的坏，core 有别的 harness 档用例比整条记录（`crates/singlefs-harness/tests/many_inodes_write_and_read.rs` 里有逐字段的期望，推的、没跑它们去确认会红）。

改法 A1（只在我的模型上量过、被攻过零轮）：改法 a 改成比整条记录——发布之后读回的记录 = 崩溃那一版的记录原样 ∪ {(崩溃那一版的号水位, 这次发布的 checkpoint_txg, 1, 0, 写入时间 = FIXED_WRITE_TIME_SECONDS)}。量过的部分：上面四行的 (号, 出生代, nlink, size) 里 nlink、出生代、size 三种与正确那一行不同，比四元组就红；写入时间这一种我的探针没打出来，推的。A2（推的）：池级 checker 补 I-9.5 的反方向，交主 agent 定要不要另立一条。

## K2　改法 b：盘面走没走 nlink = 0 那一支、被什么拒

在副本里照 `crash_image_between_the_deletion_batches` 原样造盘（参数可调），挂一次看续做结局与第一个文件那条记录的 nlink，再在各自新挂的同一份盘面上分别发「建 1 个」「建 0 个」「建 2 个」「覆盖写」，以及走真的 `mount_and_publish_once_on_the_crash_image`（发布臂）。`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b.txt` 第 1–6 行（整行；成员名截到 160 字、空格换成下划线）：

```
slots=128 ow=3 units=14 batches=1 continuation=short nlink=Some(0) arm=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ create1=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ create0=Explained(UnitAreaWall)|PlacementRefused_{_unit:_AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition_{_level:_0,_device:_DeviceIdentity(0),_index_in_device:_1_}),_refusal:_ create2=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ overwrite=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_}
slots=136 ow=3 units=14 batches=1 continuation=short nlink=Some(0) arm=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 create1=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 create0=Published create2=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 overwrite=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_}
slots=128 ow=3 units=14 batches=1 continuation=short nlink=Some(0) arm=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_} create1=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ create0=Explained(UnitAreaWall)|PlacementRefused_{_unit:_AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition_{_level:_0,_device:_DeviceIdentity(0),_index_in_device:_1_}),_refusal:_ create2=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ overwrite=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_}
slots=136 ow=3 units=14 batches=1 continuation=short nlink=Some(0) arm=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_} create1=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 create0=Published create2=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 overwrite=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_}
slots=128 ow=3 units=14 batches=1 continuation=short nlink=Some(0) arm=Explained(UnitAreaWall)|PlacementRefused_{_unit:_AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition_{_level:_0,_device:_DeviceIdentity(0),_index_in_device:_1_}),_refusal:_ create1=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ create0=Explained(UnitAreaWall)|PlacementRefused_{_unit:_AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition_{_level:_0,_device:_DeviceIdentity(0),_index_in_device:_1_}),_refusal:_ create2=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(0),_ overwrite=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_}
slots=136 ow=3 units=14 batches=1 continuation=short nlink=Some(0) arm=Published create1=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 create0=Published create2=Explained(UnitAreaWall)|SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices_{_short_devices:_[DeviceShortOfDemand_{_device:_DeviceIdentity(0),_available:_AvailableBytesOnOneDevice(1310 overwrite=Unexplained|WriteToAnInodeBeingDeleted_{_inode:_1_}
```

第 1、2 行不设开关；第 3、4 行设 `OPUS_K2B_SOME=1`（判法只看 Some）；第 5、6 行设 `OPUS_K2A_VARIANT=zero`（那一支建 0 个）。

- 走没走 nlink = 0 那一支：走了。用例的前提（续做推满仍放不下、记录 nlink = 0）在副本上成立（第 1 行 `continuation=short nlink=Some(0)`）；只看 Some 时发布臂覆盖写，被拒成 `WriteToAnInodeBeingDeleted { inode: 1 }`、`Unexplained`（第 3 行），与实现员的探针一致。
- 被什么拒：入库盘面（128 槽）上发布臂建 1 个 inode 是被空间准入拒（`SpaceAdmissionRefused`，这块盘可用 0 字节），不是落点撞墙；准入拒按设计记成 `Explained(UnitAreaWall)`（`crates/singlefs-harness/src/model_comparison.rs:673`：「PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),」）。建 0 个也被拒（落点拒，同样记成 `Explained(UnitAreaWall)`），那一支建 0 个时发布臂照样交 `Explained(UnitAreaWall)`（第 5 行）——用例最后一条断言（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:155`：「ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),」）照绿。
- 所以这条用例钉住的是「没走覆盖写」，没钉住用例名与文件头说的「建一个 inode」（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:4`：「（D8（核心索引结构） 已定项 16 第 4 条），发布臂要建一个 inode；这一版本来就挤，建 inode 被单元区墙拒，是模型认得的拒。」）。

同族盘面扫（量过，副本）：单元区 128..400 槽步长 4，覆盖写 {0, 1, 3} 次，顺序写 {4, 8, 14, 20} 个单元，意图之后 {1, 2} 批，1656 个盘面。`out/k2b-sweep-classes.txt`（整份）：

```
rows=1656
1618 WalkedToEmpty not-with-file Explained(UnitAreaWall) not-with-file not-with-file not-with-file ['128/0/14/1', '128/0/14/2', '128/0/20/1']
15 mount-refused:PlacementRefusedBe - mount-refused:Explained(UnitArea mount-refused:"PlacementRefusedB mount-refused:"PlacementRefusedB mount-refused:"PlacementRefusedB ['128/3/14/2', '128/3/20/1', '128/3/8/2']
10 short Some(0) Explained(UnitAreaWall) Explained(UnitAreaWall) Explained(UnitAreaWall) Unexplained ['128/1/20/2', '128/3/14/1', '128/3/4/2']
10 short Some(0) Explained(UnitAreaWall) Explained(UnitAreaWall) Published Unexplained ['128/0/20/2', '128/1/20/1', '132/1/20/1']
3 build-failed []
```

（列：续做结局、nlink、发布臂、建 1 个、建 0 个、覆盖写；盘面写成 槽/覆盖写/单元/批。）续做推满仍放不下的 20 个盘面里，10 个上建 0 个能做成、建 1 个被准入拒——在这类盘面上「那一支建 0 个」会把发布臂变成 `Published`、用例红。入库那一组参数（覆盖写 3、单元 14、批 1）把单元区从 128 放到 136 槽就是这一类（第 2、6 行），续做与 nlink 的前提照旧成立，只看 Some 照旧 `Unexplained`（第 4 行）。建 2 个在两份盘面上都被准入拒（第 1、2 行 `create2`），「建一个」与「建两个」两份盘面都分不出。

四句：分辨——分辨「用例钉住那一支建一个 inode」与「只钉住不走覆盖写」；系统看得到——拒的成员名里写着准入还是落点；分句——K2 题面「不是别的原因被单元区墙拒」：拒的是准入，任何发布（连空发布）在这份盘上都过不去，墙不是 nlink 那一支造的；改法——改法 b 对「不走覆盖写」有效，对「建一个」不起作用。

改法 B1（只在我的模型上量过、被攻过零轮）：`UNIT_AREA_SLOTS` 改成 136（或另加一条 136 槽的同形用例），其余照旧。量过：正确判法 `Explained(UnitAreaWall)`（第 2 行），只看 Some `Unexplained`（第 4 行），那一支建 0 个 `Published`（第 6 行）。B2（推的）：文件头与用例名里「被单元区墙拒」写成「被空间准入拒（记成单元区墙）」。

## 几个改法各修哪一格

| 改法 | K2-F3 内联判别力 | K2-a 内容 | K2-b 建 0 个 | K3 计数 |
|---|---|---|---|---|
| F3-c 全部子集种子两两不同 | 量过（4095 对 2/12/12/1） | — | — | — |
| A1 比整条记录 | — | 量过三种（nlink、出生代、size），写入时间推的 | — | — |
| A2 checker 补 I-9.5 反方向 | — | 推的（只管 nlink 那一种） | — | — |
| B1 改用 136 槽 | — | — | 量过（`Published` 对 `Explained(UnitAreaWall)`） | — |
| B2 改文件头措辞 | — | — | 推的（只改说法，不改判别力） | — |
| K3-a 计数层靶向用例 | — | — | — | 量过（(1, 1) 对 (1, 0)） |
| K3-b 撤掉「单调」那句理由 | — | — | — | 推的 |

## 没打中的形状

- F3 撞随机数：段号 0..256、段长 1..16 全部子集共 2,097,120 个异或值两两不同；五个历史种子（0、7、32768、245760、0xdeadbeef）上，二次崩溃流两两最近错开 7,039,770 个字以上，离历史那一路与第一次崩溃那一路 2^37 以上。没找到同一串、也没找到错开几个字的两串。
- F3 变窄：五组（种子、段号、段长 3 / 8 / 10 / 12 / 12）上混指纹的并集都比不混大（16、286、828、2348、2359 对 3）。
- 改法 a：十种坏发布里，建 0 个、旧实例代号、号跳一个、改动计数 0、复用 inode 1 五种被抓到（改法 a 本身或池级 checker 或实现拒）。没试：改已有那条 inode 2 的记录（要动 `publish_version` 里上一版的叶，没做），推的：改法 a 比号抓不到，池级 checker 抓不抓得到不知道。
- 改法 b：只看 Some 那一臂在入库盘面与 136 槽盘面上都报 `Unexplained`；没找到盘面让它报别的。
- K3 反方向（接错多报一条红）：750 个组合里一个都没有；只扫了「第一个文件之后一次覆盖写」这一段历史、第一次崩溃只扣第一个文件那次的记录与轮换、二次崩溃只扣写行的记录与轮换。暖机、续做、清扫各段的二次崩溃没扫。
- K3 在不越屏障的态上：没量。推的：不越屏障时轮换落了 ⇒ 那次的根落了 ⇒ 记录也落了，第三截判据 N-配置 的拒走不到（C670 欠账行说的就是这一类态不许出现，`.claude/kb/checks-owed.md` 第 529 行），两种接线都 0。

## 这条腿自己的限度

- 全部数是副本上的数：冻结副本加四个环境变量开关，开关不设时与冻结副本同（patch 只在读环境变量的地方分岔）。入库装置上要重做：K3-a 那一条靶向用例与 B1 那一条都是 checker 档用例，归主 agent 在提交时跑。
- 探针包 `opus-b21-probe` 链着 checker 档的库（`singlefs-checker-tier`）跑 `inject_crashes_into_history_drawing_second_crashes` 与 `mount_and_publish_once_on_the_crash_image`。没起 checker 档包的任何测试目标与 `src/bin/` 装置，也没调层 0 枚举函数；照代理定义 3b「在自己的原型里…不算重型」那一条当原型跑。主 agent 若判这算 checker 档，K3 与 K2-a、K2-b 的数降为推的。
- 第一次编译（debug，约 20 秒）没经内存包装，之后的编译与全部运行都经 `run-with-memory-cap.sh`；有一次 `auto` 上限 950M 编译撞了上限（退出码 250，那一次不算），改给 8G 重编。
- K3 的 c_见证 是读码推的值，探针没把它打出来；扫的结果与推法逐格对得上，但不是直接量到的。
- 改法 a 的出生代、size、nlink 三种坏发布是 `publish_new_inodes` 自己的坏，我没去确认 core 那几条 harness 用例会不会红（推的：会，`many_inodes_write_and_read.rs` 有逐字段期望）。
- 每格只抽了一次（这条腿只跑了一次），「没打中」那几格照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算没打中。

## 中途收到的消息

跑完全部量、写报告之前收到主 agent「线程上限改成 7」：之后的命令按 7 起；`tools/rerun.sh` 默认线程改成 7，k2b-sweep 每批并行件数从 8 改成 6，改后重跑了 k2b-sweep 一段，产物与改前 `cmp` 相同。之前已经跑完的那几段用的是 15。

## 草稿与副本

- 仓副本与编译目录（`/tmp/claude-1000/m3-formal-code-b21-r1-opus/copy/`、`/tmp/claude-1000/m3-formal-code-b21-r1-opus/rerun/copy/`）交回前已删。
- 留在草稿目录 `/tmp/claude-1000/m3-formal-code-b21-r1-opus/` 的只有日志与小产物（约 2 MB）：`out-*.txt`、`k2b/`、`k3s/`、`rerun/out/`、`build*.log`、`rerun-*.log`。入库的那一份是模型目录 `out/`。

## 要主 agent 跑的命令

1. 复跑核对（照「复跑」一节；工作目录仓根；`WORK=<主 agent 的草稿目录> THREADS=7`；内存上限由脚本内部给 `run-with-memory-cap.sh`：编译 8G、探针 4G；约 5 分钟）。看 `$WORK/out/` 下七个文件与模型目录 `out/` 逐个 `cmp`；k3 看 `k3.txt` 第 35、36 行，`k3-sweep.txt` 四条 `k3-sweep combos=` 行的 `counts_differ` 合计 9。
2. 若采纳 K3-a、B1：把那两条写成入库用例之后在提交时跑 checker 档（不归我跑）。

## 没做什么

- 没判正推腿的 K1，没读禁读清单里的文件。
- 没跑任何 checker 档包的测试目标、层 0、变异表；21-2 那四行变异在入库装置上的红绿没量。
- 没造改已有 inode 记录的坏发布；没扫暖机、续做、清扫各段的二次崩溃对 K3 的影响。
- 没把改法写进仓里任何 crates 文件。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b21-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b21-r1/tree --root . --background research/prompts/_m3-formal-code-b21-r1-background.md` 末行：

```
  ✓ 核了 22 处引文，对上 22 处，没判 0 处（逐处列在上面）
```
