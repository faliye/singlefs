# m3-formal-code-b23-r1 云端攻方（Opus）报告（2026-10-04）

攻击面：K1（K3-a 的态与写下标现算、F3-c 的判别力）、K2（A1 整条比对与三行 core 变异的牵连、B1 136 槽的稳定性）。代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b23-r1/tree/`，下文 `crates/…` 路径都指那棵树。量出来的数都是我自己拷出的副本上的数：副本 = 冻结副本的 crates/ 加仓根 Cargo.toml、Cargo.lock、litmus/，用 `tools/patch_copy.py` 打几个环境变量开关，再加一个只在副本里的探针包 `opus-b23-probe`。没跑 checker 档包的任何测试目标；探针包自己的二进制链着 checker 档的库跑（与 b21 攻方同一种用法，b21 判决复跑时照这样认了），另在副本上跑了 harness、core、checker 三个包的全部测试目标（harness 档）。

## 复跑

模型目录 `research/prompts/m3-formal-code-b23-r1-opus-model/`。仓根执行，`WORK` 指一个空的草稿目录：

```
WORK=/tmp/claude-1000/<草稿>/b23-opus-rerun THREADS=7 bash research/prompts/m3-formal-code-b23-r1-opus-model/tools/rerun.sh all
```

可覆盖的环境变量：`REPO_ROOT`（默认按脚本位置推出仓根）、`FROZEN_TREE`（默认 `/tmp/claude-1000/m3-formal-code-b23-r1/tree`）、`WORK`、`THREADS`（默认 7）、`BUILD_CAP`（默认 8G）、`RUN_CAP`（默认 4G）。也可以分段：`setup`、`build`、`build-tests`、`k3a`、`f3`、`a1`、`b1`、`b1-sweep`、`collateral`（或 `collateral-one none|nlink|birth|size`）。产物写进 `$WORK/out/`，与模型目录 `out/` 下同名文件逐字节比（`cmp`）。各段在我这里的挂钟：setup 与 build 冷编约 1.5 分钟，build-tests 冷编约 5 分钟，k3a 68 秒，f3 106 秒，a1 6 秒，b1 8 秒，b1-sweep 10 秒，collateral 四次各约 2.5 分钟。我在 `WORK=/tmp/claude-1000/m3-formal-code-b23-r1-opus` 上按这个脚本从 setup 跑到 collateral（每段单独起、前台），`out/` 下的文件就是那一遍的产物；k3a、f3、b1-neighbourhood 与脚本成形之前的探索跑 `cmp` 相同，a1、b1-slots 只差探索时混进去的 stderr 行（脚本把 stderr 单写进 `$OUT/stderr.log`）。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`）：

```
4b18dfe1664853330ec57896f0489d77f0a67407cf2686ede78187fa211cd9d6  research/prompts/m3-formal-code-b23-r1-opus-model/out/a1.txt
0fcf3acd3e655dd31a20a218336ec44451272668cedde437f2b46d97952a8f69  research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-neighbourhood.txt
81bf275c424cff46c00f3d989204d13b5157772f308591650faefebc07cad036  research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-slots.txt
79fd5b3950c2557a6e6d3b1f0135b17386e0820c6f5a293efb3b06d7bb0dd15c  research/prompts/m3-formal-code-b23-r1-opus-model/out/collateral.txt
7e3272adc8de5d7ee41805b575645fb3dad8087b0e6f1c55af7efa6ed0edca02  research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt
b0568b38286f45f2ecc411bb70603ea82241a326d27ad410b21bb621a7cb3f78  research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-model.txt
efc9af5b32ddc87fc56209b2f5febdbcf99e14106f71b391493368af691e96f9  research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-segments.txt
fdbd7a1a38dfd4629a91ba82dbe8ac2fe8910c2adb29ae101852f68ffa98985c  research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-variants.txt
5484606a81754d95d593d614903c555c12c4407c8a88c1237a9ee0c1dee075b9  research/prompts/m3-formal-code-b23-r1-opus-model/out/k3a-variants.txt
e06564f9cbc93a5b13e46dac03d5776ffd2825b7c25cbd7ab7facf8845423943  research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus-b23-probe/Cargo.toml
6f2fea5ad768153c7a700e15f644cf5272a3f739c0ae331b718de3b4db957298  research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus-b23-probe/src/main.rs
4aec6a4ee0712d0005cdcacb7993afa39274827fa9940f3908c5c1916e0df1a6  research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus_probe.rs
226de03f1c17282533570126852f0a2ad2c87841625920fb1a499280981a3a48  research/prompts/m3-formal-code-b23-r1-opus-model/tools/f3_model.py
5380a8f14d74c93a24df2c5a274086db06f29a330af45a81c958835774852d71  research/prompts/m3-formal-code-b23-r1-opus-model/tools/patch_copy.py
c356fcbd509c74063098f9dd2c827ec46487711275e5ead1ef5b5c962cc9324f  research/prompts/m3-formal-code-b23-r1-opus-model/tools/rerun.sh
472f89506898e4be9dd08dafab48da84ff36c2199322a23d10a2d081fa08d4b2  research/prompts/m3-formal-code-b23-r1-opus-model/tools/summarize_b1.py
```

副本上的开关（`tools/patch_copy.py`；不设环境变量时副本行为与冻结副本相同，每处原文要恰好命中一次，否则退 3）：

- `OPUS_K3_WIRING=first`：第三截调用点把所选那一版交成第一次崩溃那一版（与 `crates/mutations.tsv` 23-1 K3-a 那一行同义）。
- `OPUS_ARM=time-zero|count-zero|count-two|old-instance`：只改第二截「带文件而第一个文件不是活记录」那一支的调用点参数（写入时间 0 与 mutations.tsv 23-1 那一行同义；建 0 个与 B1 那一行同义）。
- `OPUS_K2A_VARIANT=<坏发布>`：`publish_new_inodes` 里只在那一支的调用期间生效（nlink 0、出生代取上一版、size 4096、改动计数取上一版、出生代取下一版、整次发布跳一个 txg）。
- `OPUS_CORE_MUT=nlink|birth|size`：`publish_new_inodes` 里每个调用都生效，与 mutations.tsv 23-1 那三行 core 变异同义（会话、历史、挂载里建 inode 都受影响）。
- `OPUS_K2B_SOME=1`：判法只看 `inode_record` 是不是 Some（与 mutations.tsv 改法 b 那一行同义）。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| K1-a K3-a 的态是不是攻方那个态 | 是（没打中） | 量过（副本） | 用例的写下标现算出 first(段 13、起 53、数 28、扣 [0, 1, 4]) 与 second(段 2、起 16、数 4、扣 [0, 1, 3])，就是 b21 攻方 `k3-sweep.txt` 那一行的态 |
| K1-a 接对 (1, 1)、接错 (1, 0) | 成立（没打中） | 量过（副本） | 正确接线 (判过, 不合法) = (1, 1)、第三截一条第三支的红；接错 (1, 0)、第三截零条 |
| K1-b 写下标现算在别的写表长度上摆不摆错 | 不摆错（没打中） | 量过（副本） | 4 种几何 × 4 种覆盖写长度 × 1–3 次覆盖写，写表 83–179 个写的 48 段历史上，照用例算法摆出的态全部「正确接线绿、接错红」 |
| K1-c 抽函数前后逐位相同 | 相同（没打中） | 量过（副本） | 200,000 组随机（种子、段号、段长 0–200、子集）上抽前那一句与抽后的函数 0 处不同 |
| K1-c 4095 个子集两两不同抓不抓得住 `.take(1)` 之外的弱指纹 | 打中（弱） | 量过（副本 + 独立 Python 模型） | 抓得住 take(11)、skip(1)、持久个数、第一个被扣下的写、截成 u16；抓不住 take(12)、take(16)、按位拼接，也抓不住去掉段号、去掉历史种子、以或代替异或——真历史里候选段长到 26（384 槽）与 30（4 GiB） |
| K2-a A1 整条比对漏没漏字段 | 没漏能被坏发布碰到的字段（没打中） | 量过（副本）+ 推的 | 结构体六个字段全比；盘上另九样（mode、uid、gid、locality、blocks、rdev、另两个时间秒、纳秒）`parse` 不读、`publish_new_inodes` 也写不到 |
| K2-a 三行 core 变异只让这条用例红还是牵连别的 | 牵连别的 | 量过（副本，harness 档） | 三行各自让 harness 的 `many_inodes_write_and_read` 红，nlink 那行另红 4 个目标、size 那行另红 1 个；A1 在这三行上不是唯一会红的 |
| K2-a A1 的期望 txg 取自结果本身，自指会不会放过跳号 | 不放过（没打中） | 量过（副本） | 整次发布跳一个 txg 时分配器自检 panic；只把出生代写成下一版时 A1 红、I-9.26 也红 |
| K2-b 136 槽稳不稳地走到 nlink = 0 那一支 | 稳（没打中） | 量过（副本） | 134–138 五档上用例绿、两行变异都红；136 在正中 |
| K2-b 对别的盘面参数敏不敏感 | 打中（弱） | 量过（副本） | 分得开的那一档宽 5 槽、每多一个数据单元上移 2 槽；往下漂 ≥ 3 槽（48 KiB）时用例照绿而「那一支建 0 个」变异不再红，用例里没有一条前提钉住这件事 |

## K1-a　K3-a 的态是不是攻方那个态、计数分不分得开

被攻的用例：`crates/singlefs-checker-tier/src/crash_injection.rs:6484`：「fn a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal(」。它的写下标全是现算：第一次崩溃扣下第一个文件那次的两份记录与根槽之后第二槽轮换（`crates/singlefs-checker-tier/src/crash_injection.rs:6521`：「.chain(std::iter::once(&rotation_after_the_first_file[1]))」），二次崩溃扣下写行的两份记录与根槽之后第一槽轮换（`crates/singlefs-checker-tier/src/crash_injection.rs:6599`：「.chain(std::iter::once(&rotation_after_the_root[0]))」），最后断言计数（`crates/singlefs-checker-tier/src/crash_injection.rs:6663`：「(1, 1),」）。

我在探针里把用例的算法逐句重写一遍（`probe/opus_probe.rs` 的 `k3a_body`，前提不成立时报一行而不 panic），对两种接线各跑一次注入，再按用例的六条断言判绿红。接错用的是副本开关 `OPUS_K3_WIRING=first`，在冻结副本那一处（`crates/singlefs-checker-tier/src/crash_injection.rs:3054`：「version_landed_on_after_the_crash: report.effective_root,」）换成第一次崩溃那一版。用例那段历史上的结果，`research/prompts/m3-formal-code-b23-r1-opus-model/out/k3a-variants.txt` 第 1 行（整行）：

```
k3a width=384 length=inside0 overwrites=1 writes=83 segments=20 ff_records=[53, 54] ff_rotation=[56, 57] ow_first=58 ow_root=80 first(seg=13,start=53,count=28,withheld=[0, 1, 4]) mount_writes=55 second(seg=2,start=16,count=4,withheld=[0, 1, 3]) Vo=Some((InstanceGeneration(1), CheckpointTxg(4))) Vn=Some((InstanceGeneration(2), CheckpointTxg(5))) vo_ne_vn=true | wiring=second pre=(1, 1, 0) second=(1, Some(1)) (judged, illegal)=(1, 1) third_stage_branches=["NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable"] inlined_test=green | wiring=first pre=(1, 1, 0) second=(1, Some(1)) (judged, illegal)=(1, 0) third_stage_branches=[] inlined_test=red
```

- 态：第一次崩溃从写 53 起数 28 个写（摆到覆盖写的根 80）、扣下段内 0、1、4（写 53、54、57）；二次崩溃从挂载写表写 16 起数 4 个写、扣下 0、1、3（写 16、17、19）。这正是 b21 攻方产物 `research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt` 第 9 行那个组合 `first(last=80,withheld=[0, 1, 4]) second(last=19,withheld=[0, 1, 3])`。
- 计数：正确接线 (1, 1)，第三截一条第三支（两条都读不出）的红；接错 (1, 0)，第三截零条。用例的最后一条断言在接错时红，前面五条前提两种接线都过。
- 一处多余但无害：二次崩溃扣下的第 3 个就是这个态的最后一个写，「数 4 个扣第 3 个」与「数 3 个不扣它」是同一份盘面、同一段（崩在哪一段按第一个被扣下的写取），所以第二槽轮换「没摆到」与第一槽「扣下」在盘上没有区别。读码判的，不影响计数。

## K1-b　写下标现算在别的写表长度上会不会摆错态

做法：同一套算法换历史，只固定「起点是第一个文件之后、接着 n 次覆盖写」这个前缀与两次崩溃的摆法规则，放开几何（384、256、240 槽与 4 GiB）、覆盖写长度（一个单元里两种、空、跨三个单元，另有整单元与多一字节两种）与覆盖写次数（1、2、3），共 72 段。

```
$ grep -c 'inlined_test=green | wiring=first.*inlined_test=red' k3a-variants.txt
48
$ grep -c 'PRECONDITION-FAILED overwrite-wrote-nothing' k3a-variants.txt
24
$ grep -v PRECONDITION k3a-variants.txt | grep -o ' writes=[0-9]*' | sort -t= -k2 -n | uniq | tr '\n' ' '
 writes=83  writes=91  writes=93  writes=101  writes=108  writes=120  writes=128  writes=133  writes=140  writes=149  writes=163  writes=179 
$ grep -v PRECONDITION k3a-variants.txt | grep -o 'first([^)]*)' | sort | uniq -c
     27 first(seg=13,start=53,count=28,withheld=[0, 1, 4])
      9 first(seg=13,start=53,count=38,withheld=[0, 1, 4])
      9 first(seg=13,start=57,count=32,withheld=[0, 1, 4])
      3 first(seg=13,start=57,count=42,withheld=[0, 1, 4])
```

- 48 段跑得起来的历史（写表 83 到 179 个写，第一次崩溃的起点与长度有四种）上，照用例算法摆出的态在正确接线下用例全绿、接错全红。
- 另 24 段（整单元、多一字节两种长度）覆盖写一个写都没写出（历史跑完，但覆盖写那一步没落盘），用例的算法在第一步就找不到覆盖写的第一个写；入库用例在这种历史上会在 `expect` 那一处 panic，响着失败，不会悄悄摆别的态。
- 前提断言 (2, 2) 两处、`assert_ne!` 一处都在这 48 段上成立；我没造出「前提照过、态却摆到别处」的写表。推翻条件：找到一份写表，第一个文件那次或写行那次每块盘一份记录、一槽轮换（前提照过）而两份记录夹着别的发布的记录。

四句（这一格没打中，照写）：不分辨——两种接线在这 48 段上与用例那段同向；系统看得到——用例自己算写下标；分句——题面 K1「写下标现算会不会在别的写表长度上摆错态」；改法——不需要。

## K1-c　F3-c：抽函数前后逐位相同、4095 个子集两两不同抓不抓得住别的弱指纹

### 逐位相同（没打中，量过）

探针里把 diff 删掉的那四行原样写成 `seed_before_the_extraction`，与入库的 `seed_of_the_second_crash_points` 在 200,000 组随机输入上逐个比（种子四分之一取小整数、其余随机；段号 0–99,999；段长 0–200）。`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-segments.txt` 第 1 行：

```
f3-bitwise samples=200000 differ=0
```

### 判别力（打中，弱；量过）

被攻的用例只取一个点：历史种子 0、段号 16、段长 12（`crates/singlefs-checker-tier/src/crash_injection.rs:4407`：「const WRITES_OF_THE_SEGMENT: usize = 12;」，`crates/singlefs-checker-tier/src/crash_injection.rs:4412`：「HistorySeed(0),」，`crates/singlefs-checker-tier/src/crash_injection.rs:4414`：「segment_index: 16,」）。种子是四样异或（`crates/singlefs-checker-tier/src/crash_injection.rs:2828`：「^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")」）。用例在这个点上比的只是「前 12 个写的指纹」与「固定的种子、段号」，所以两类改坏照绿：指纹只看前 k ≥ 12 个写的，与种子组合里丢掉或弄坏历史种子、段号那两项的（历史种子取 0，异或它是恒等）。

真历史里的段比 12 长。候选段（与 `draw_crash_points` 同一条界：整段落在 mkfs 之后、最后一个跑完的操作之前）的段长，64 段生成历史各 16 步，`f3-segments.txt` 第 2、3 行：

```
f3-segments width=384 histories=64 operations=16 candidate_segments=3932 longer_than_12=590 longer_than_16=412 longest=26 histogram={1: 968, 2: 2147, 8: 9, 12: 218, 14: 86, 16: 92, 18: 17, 20: 329, 22: 16, 24: 46, 26: 4}
f3-segments width=4g histories=64 operations=16 candidate_segments=3932 longer_than_12=808 longer_than_16=592 longest=30 histogram={1: 968, 2: 2147, 12: 9, 16: 216, 18: 55, 20: 82, 22: 48, 24: 279, 26: 16, 28: 108, 30: 4}
```

17 种种子算法（入库那一种，加 16 种改坏的），每种量：内联用例（4095 个两两不同才绿）、同一段号上长 L 的段（L ≤ 16 全部真子集，更长抽 20,000 个不同的子集）不同种子数、16 段生成历史（384 槽、各 12 步，候选段 ≤ 12 写全部真子集、更长的段各抽 64 个，共 119,800 个第一次崩溃态）里同一段历史内撞种子的态数，以及 16 段历史合起来的不同种子数。`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-variants.txt` 整份：

```
f3-variants width=384 histories=16 operations=12 first_crash_states=119800
variant=entered(fnv1a) inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=0 distinct_over_all_histories=90032/119800
variant=take(1) inline_4095_test=red distinct_in_inline=2 within_segment[L12:2/4095 L13:2/8191 L16:2/65535 L24:2/20000 L40:2/20000 L64:2/20000 L65:2/20000 L100:2/20000] states_sharing_a_seed_within_a_history=118542 distinct_over_all_histories=140/119800
variant=take(11) inline_4095_test=red distinct_in_inline=2048 within_segment[L12:2048/4095 L13:2048/8191 L16:2048/65535 L24:2048/20000 L40:2047/20000 L64:2048/20000 L65:2048/20000 L100:2048/20000] states_sharing_a_seed_within_a_history=55391 distinct_over_all_histories=45554/119800
variant=take(12) inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:4096/8191 L16:4096/65535 L24:4069/20000 L40:4074/20000 L64:4064/20000 L65:4056/20000 L100:4062/20000] states_sharing_a_seed_within_a_history=54 distinct_over_all_histories=86572/119800
variant=take(16) inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:17270/20000 L40:17271/20000 L64:17285/20000 L65:17242/20000 L100:17261/20000] states_sharing_a_seed_within_a_history=3 distinct_over_all_histories=90023/119800
variant=skip(1) inline_4095_test=red distinct_in_inline=2048 within_segment[L12:2048/4095 L13:4096/8191 L16:32768/65535 L24:19988/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=55921 distinct_over_all_histories=48767/119800
variant=rev inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=0 distinct_over_all_histories=90032/119800
variant=bitmask-shl inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=58895 distinct_over_all_histories=11134/119800
variant=fnv-as-u32 inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=0 distinct_over_all_histories=90032/119800
variant=fnv-as-u16 inline_4095_test=red distinct_in_inline=4015 within_segment[L12:4015/4095 L13:7831/8191 L16:42919/65535 L24:17322/20000 L40:17157/20000 L64:17262/20000 L65:17239/20000 L100:17199/20000] states_sharing_a_seed_within_a_history=9540 distinct_over_all_histories=49548/119800
variant=popcount inline_4095_test=red distinct_in_inline=12 within_segment[L12:12/4095 L13:13/8191 L16:16/65535 L24:20/20000 L40:26/20000 L64:29/20000 L65:32/20000 L100:39/20000] states_sharing_a_seed_within_a_history=118928 distinct_over_all_histories=76/119800
variant=first-withheld inline_4095_test=red distinct_in_inline=12 within_segment[L12:12/4095 L13:13/8191 L16:16/65535 L24:15/20000 L40:14/20000 L64:15/20000 L65:14/20000 L100:13/20000] states_sharing_a_seed_within_a_history=119039 distinct_over_all_histories=76/119800
variant=drop-history-seed inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=0 distinct_over_all_histories=56977/119800
variant=drop-salt inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=0 distinct_over_all_histories=90032/119800
variant=drop-segment inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=58634 distinct_over_all_histories=61166/119800
variant=add-not-xor inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65535/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:20000/20000] states_sharing_a_seed_within_a_history=0 distinct_over_all_histories=94136/119800
variant=or-not-xor inline_4095_test=green distinct_in_inline=4095 within_segment[L12:4095/4095 L13:8191/8191 L16:65531/65535 L24:20000/20000 L40:20000/20000 L64:20000/20000 L65:20000/20000 L100:19999/20000] states_sharing_a_seed_within_a_history=49405 distinct_over_all_histories=17297/119800
```

内联用例照绿而真有损的六种：

| 改坏 | 内联用例 | 损在哪 |
|---|---|---|
| `take(12)`（指纹只折前 12 个写） | 绿 | 长 24 的段 20,000 个子集只剩 4069 个种子；16 段历史里同一段历史内 54 个态撞种子 |
| `take(16)` | 绿 | 长 24 的段只剩 17,270 个；同一段历史内 3 个态撞 |
| 按位拼接（`fold(0, (h << 1) | b)`） | 绿 | 段号与掩码异或抵消：同一段历史内 58,895 个态撞，16 段合起来 119,800 个态只剩 11,134 个种子 |
| 去掉段号 | 绿 | 同一段历史内 58,634 个态撞（不同段同一子集同种子） |
| 以或代替异或 | 绿 | 同一段历史内 49,405 个态撞 |
| 去掉历史种子 | 绿 | 同一段历史内不撞；16 段合起来 56,977 个，入库那一种 90,032 个 |

内联用例抓得到的：`take(1)`、`take(11)`、`skip(1)`、持久个数、第一个被扣下的写、截成 u16。另有四种照绿而在这几项上量不出损：反着折（`rev`）、截成 u32、去掉盐、以加代替异或；盐的作用是与历史那一路、第一次崩溃那一路的流岔开，这几项量不到它（b21 攻方量过流之间的平移距离，这一轮没重量）。

第二条路子：`tools/f3_model.py` 不经 Rust，从冻结副本里按名字读出盐与 FNV 两个常量自己算，复算其中八种的内联用例与下面 F3-d 的三条。`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-model.txt` 整份，与 Rust 那一份（`f3-fix.txt`）逐格一致：

```
f3-model variant=entered(fnv1a) inline=green(4095) A=262080/262080 B=4095/4095 C=8190/8190
f3-model variant=take(12) inline=green(4095) A=262080/262080 B=1/4095 C=8190/8190
f3-model variant=take(16) inline=green(4095) A=262080/262080 B=1/4095 C=8190/8190
f3-model variant=bitmask-shl inline=green(4095) A=4096/262080 B=4095/4095 C=4096/8190
f3-model variant=drop-history-seed inline=green(4095) A=262080/262080 B=4095/4095 C=4095/8190
f3-model variant=drop-segment inline=green(4095) A=4095/262080 B=4095/4095 C=8190/8190
f3-model variant=or-not-xor inline=green(4095) A=13254/262080 B=4095/4095 C=5998/8190
f3-model variant=take(1) inline=red(2) A=128/262080 B=1/4095 C=4/8190
```

四句：

1. 分不分辨：分辨「用例钉住整个子集与四样异或都进了种子」与「只钉住种子 0、段号 16 上前 12 个写进了指纹」。
2. 系统看不看得到：种子函数是纯函数，用例手里就有它的全部输入。
3. 满足的是哪一个分句：正文 K1 格「4095 个子集两两不同的断言能不能抓住别的弱指纹（不只是 `.take(1)`）」——抓不住 `take(12)`、`take(16)`、按位拼接；另三种（去段号、去历史种子、或代替异或）是种子组合的坏，不是指纹的坏，同一个用例同样看不见。用例文档注释只声称抓 `.take(1)`（`crates/singlefs-checker-tier/src/crash_injection.rs:4403`：「/// 判别力：指纹只折段内第一个写落没落（`crates/mutations.tsv` 一行）时种子只剩 2 个，这一条红。」），这句本身是真的。
4. 跑前条款的改法在这几格上还中不中：正文第五节只给「打中而坐实 ⇒ 改法并进下一批实现员」，下面 F3-d 在六种上都红、入库那一种上全绿；登记的 `.take(1)` 那行变异照红，这一格不碰它。

### 改法 F3-d（只在我的模型上量过、被攻过零轮）

在 F3-c 旁加三条断言，都只调 `seed_of_the_second_crash_points`：A 历史种子 0、段号 0..=63、每段长 12 的 4095 个子集，262,080 个种子两两不同；B 历史种子 0、段号 16、长 28、前 16 个写都落了而后 12 个写取 4095 种，种子两两不同；C 段号 16、长 12 的 4095 个子集在历史种子 0 与 1 上各算一遍，8190 个两两不同。`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 整份：

```
f3-fix variant=entered(fnv1a) inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green
f3-fix variant=take(1) inline=red A=128/262080:red B=1/4095:red C=4/8190:red
f3-fix variant=take(11) inline=red A=131072/262080:red B=1/4095:red C=4096/8190:red
f3-fix variant=take(12) inline=green A=262080/262080:green B=1/4095:red C=8190/8190:green
f3-fix variant=take(16) inline=green A=262080/262080:green B=1/4095:red C=8190/8190:green
f3-fix variant=skip(1) inline=red A=131072/262080:red B=4095/4095:green C=4096/8190:red
f3-fix variant=rev inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green
f3-fix variant=bitmask-shl inline=green A=4096/262080:red B=4095/4095:green C=4096/8190:red
f3-fix variant=fnv-as-u32 inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green
f3-fix variant=fnv-as-u16 inline=red A=64448/262080:red B=4015/4095:red C=7830/8190:red
f3-fix variant=popcount inline=red A=64/262080:red B=12/4095:red C=12/8190:red
f3-fix variant=first-withheld inline=red A=64/262080:red B=12/4095:red C=12/8190:red
f3-fix variant=drop-history-seed inline=green A=262080/262080:green B=4095/4095:green C=4095/8190:red
f3-fix variant=drop-salt inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green
f3-fix variant=drop-segment inline=green A=4095/262080:red B=4095/4095:green C=8190/8190:green
f3-fix variant=add-not-xor inline=green A=262080/262080:green B=4095/4095:green C=8190/8190:green
f3-fix variant=or-not-xor inline=green A=13254/262080:red B=4095/4095:green C=5998/8190:red
```

入库那一种三条全绿；上表照绿而有损的六种，每种至少一条红（take(12)、take(16) 红在 B；按位拼接红在 A、C；去段号红在 A；去历史种子红在 C；或代替异或红在 A、C）。B 取 28 是 4 GiB 几何上量到的最长候选段 30 之内的值；以后段更长，B 的长度要跟着改（推的）。

不是打中、只记一笔的观测：入库种子是历史种子与段号线性异或，不同历史之间会撞——16 段历史合起来 119,800 个态只有 90,032 个不同种子（上面 `f3-variants.txt` 第 2 行），历史 s 的第 k 段与历史 t 的第 j 段在 s ^ k = t ^ j、子集相同时用同一串随机数。代码只声称同一段历史里各抽各的（`crates/singlefs-checker-tier/src/crash_injection.rs:107`：「/// 摆挂载途中的二次崩溃的随机源，与上面两路再岔开（再按第一次崩溃的段号与段内子集岔开：同一段历史上各个崩溃状态抽的不是同一串，」），不同历史的挂载写表一般也不同，这件事有没有代价我没量。

## K2-a　A1：整条比对漏没漏字段、三行 core 变异牵连谁

### 字段（没打中；量过一半、推一半）

A1 比的是 `inode_records()` 交回的结构体（`crates/singlefs-checker-tier/src/crash_injection.rs:4929`：「expected_records_after_the_publish.as_slice(),」），结构体只有六个字段（`crates/singlefs-core/src/records.rs:50`：「pub struct InodeRecord {」：号、出生代、nlink、size、改动计数、写入时间），全比。盘上那 140 字节里另有九样，读的时候跳过：`crates/singlefs-core/src/records.rs:100`：「reader.skip(8 + 4 + 4 + 4); // locality_id、mode、uid、gid」，`crates/singlefs-core/src/records.rs:103`：「reader.skip(8 + 8); // blocks（由 size 定，不交出）、rdev」，`crates/singlefs-core/src/records.rs:105`：「reader.skip(16);」（另两个时间秒），纳秒那 12 字节也不读。

- 坏发布碰不碰得到这九样：碰不到。`publish_new_inodes` 只造结构体（`crates/singlefs-core/src/transaction.rs:5807`：「object_birth: txg,」那一段），九样由 `to_bytes` 写常量或由 size 现算（`crates/singlefs-core/src/records.rs:68`：「writer.put_u32(0o100_644); // mode」，`crates/singlefs-core/src/records.rs:77`：「for _clock in 0..3 {」）。读码判的。
- 编码器自己写错这九样（改 `to_bytes`）A1 看不见，「崩溃那一版原样」那一半也看不见（同一个 `parse`）。池级 checker 里 blocks 有判（`crates/singlefs-checker/src/walk.rs:2351`：「.judge("I-9.15", blocks == logical_length_in_blocks, || {」）；mode、uid、gid、另两个时间秒，walk.rs 里登记的记录偏移只有 size、出生代、改动计数、blocks 四个（`crates/singlefs-checker/src/walk.rs:158`：「const INODE_RECORD_SIZE_OFFSET: usize = 40;」，`crates/singlefs-checker/src/walk.rs:168`：「const INODE_RECORD_BLOCKS_OFFSET: usize = 48;」），推的：没人判。这是编码器那一层的事，不是第二截那一支的坏，算不算进 A1 交主 agent。

量过的一半：坏发布逐种换进第二截那一支或换进每个调用，照 A1 的算法与断言判。`research/prompts/m3-formal-code-b23-r1-opus-model/out/a1.txt` 整份（四元组之后两项是改动计数与写入时间；第 7、8 行是 panic 消息截下来的两行）：

```
a1 arm_variant=- arm_param=- core=- A1=GREEN pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=true after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 1, 0, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=being-deleted arm_param=- core=- A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 0, 0, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=old-birth arm_param=- core=- A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 8, 1, 0, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=size arm_param=- core=- A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 1, 4096, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=change-previous arm_param=- core=- A1=RED pre_ok=true publish_ok=false published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 1, 0, 8, 1788000000)] checker_on_pool_after=["I-9.26"] findings_after=["崩溃状态上的新发现 CheckerViolations { invariants: [\"I-9.26\"] }（after_the_writable_mount_and_one_publish）：种子 0，第 26 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(1)／Some(CreateInode)\n  恢复读回：Published\n  I-9.26：inode "] other_stage_findings=[]
a1 arm_variant=birth-next arm_param=- core=- A1=RED pre_ok=true publish_ok=false published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 10, 1, 0, 9, 1788000000)] checker_on_pool_after=["I-9.26"] findings_after=["崩溃状态上的新发现 CheckerViolations { invariants: [\"I-9.26\"] }（after_the_writable_mount_and_one_publish）：种子 0，第 26 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(1)／Some(CreateInode)\n  恢复读回：Published\n  I-9.26：inode "] other_stage_findings=[]
a1 arm_variant=txg-skip arm_param=- core=- PANIC assertion `left == right` failed: 装了根环表的进程写的根 txg 逐个加一、每一条都经分配器记：带单元的发布在取完落点之后记，mount 发的零单元发布在根落盘之后记，调用方直接发的零单元发布由 publish_first_file 在接上它之前补记（record_zero_unit_roots_leading_to）；失败的发布连这张表一起退回、重发用同一个 txg。跳号说明有一条根绕过了这几处
  left: CheckpointTxg(
a1 arm_variant=- arm_param=time-zero core=- A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 1, 0, 9, 0)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=- arm_param=count-zero core=- A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=- arm_param=count-two core=- A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 1, 0, 9, 1788000000), (4, 9, 1, 0, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=- arm_param=old-instance core=- A1=RED pre_ok=true publish_ok=false published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=true after=[(2, 6, 1, 0, 6, 1788000001), (3, 9, 1, 0, 9, 1788000000)] checker_on_pool_after=["I-3.8", "I-4.2", "I-8.6", "I-8.7", "I-8.8"] findings_after=["崩溃状态上的新发现 CheckerViolations { invariants: [\"I-3.8\", \"I-4.2\", \"I-8.6\", \"I-8.7\", \"I-8.8\"] }（after_the_writable_mount_and_one_publish）：种子 0，第 26 段（2 个写，持久 1 个，扣下段内第 [1] 个），落在 Operation(1)／Some(CreateInod"] other_stage_findings=[]
a1 arm_variant=- arm_param=- core=nlink A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 0, 0, 6, 1788000001), (3, 9, 0, 0, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=- arm_param=- core=birth A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 5, 1, 0, 6, 1788000001), (3, 8, 1, 0, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
a1 arm_variant=- arm_param=- core=size A1=RED pre_ok=true publish_ok=true published=true crashed_numbers=[2] crashed_root_txg=6 after_root_txg=9 full_compare=false after=[(2, 6, 1, 4096, 6, 1788000001), (3, 9, 1, 4096, 9, 1788000000)] checker_on_pool_after=[] findings_after=[] other_stage_findings=[]
```

- 第 1 行：正确实现绿；新建那一条 (3, 9, 1, 0, 9, 1788000000)。
- 只在那一支生效的坏发布（第 2–6、9–12 行）全红：nlink 0、出生代上一版、size 4096、写入时间 0、建 2 个、建 0 个只红在比整条；改动计数取上一版、出生代取下一版另有 I-9.26 与第二截新发现；旧实例代号比整条照绿（第 12 行 `full_compare=true`），红在第二截新发现（I-3.8 等五条）。正文第二节说「出生代、size 两行变异没上探针」，这里补上：两行都红（第 3、4、14、15 行）。
- A1 的期望 txg 取自发布之后那一版本身（`crates/singlefs-checker-tier/src/crash_injection.rs:4916`：「let txg_of_the_publish = version_after_the_publish.effective_root.checkpoint_txg;」），我试了它会不会放过「整次发布跳一个 txg」：放不过，分配器自检先 panic（第 7 行；`crates/singlefs-core/src/allocator.rs:1821`：「let expected_txg = CheckpointTxg(occupancy.latest_recorded_txg.0 + 1);」）。只把出生代写成下一版（根照常）A1 红（第 6 行）。
- 三行 core 变异（第 13–15 行）：历史里建的 inode 2 也被改坏（nlink 0、出生代 5、size 4096），A1 拿崩溃那一版原样当期望，判不了它，红在新建那一条上。nlink 0 那一种池级 checker 与第二截都不响（第 13 行），与 `.claude/kb/checks-owed.md:564`：「| C708 | 树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出 |」同一件事。

### 三行 core 变异牵连谁（量过，副本，harness 档）

在副本上对 harness、core、checker 三个包的全部测试目标（145 个，不含 checker 档包）各跑一遍：不设变异的基线，与 `OPUS_CORE_MUT=nlink|birth|size` 三种（每个调用都生效，与 mutations.tsv 那三行同义）。`research/prompts/m3-formal-code-b23-r1-opus-model/out/collateral.txt` 整份：

```
core_mutation=none targets_ok=145 targets_failed=0
core_mutation=nlink targets_ok=140 targets_failed=5
  failed_target `-p singlefs-harness --test admission_ring_window_and_tombstone_prepay_on_the_publish_path`
  failed_target `-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full`
  failed_target `-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode`
  failed_target `-p singlefs-harness --test many_inodes_write_and_read`
  failed_target `-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version`
  test a_new_empty_inode_opens_under_the_read_only_mount_with_no_data_unit ... FAILED
  test a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_product_fanout ... FAILED
  test creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold ... FAILED
  test tag_six_of_the_first_file_version_is_three_slots_and_one_live_object_and_two_new_inodes_add_two_more ... FAILED
  test the_fifth_tombstone_container_grows_the_tree_to_three_levels_and_rebuild_reads_it ... FAILED
  test the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place ... FAILED
  test the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header ... FAILED
core_mutation=birth targets_ok=144 targets_failed=1
  failed_target `-p singlefs-harness --test many_inodes_write_and_read`
  test creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold ... FAILED
core_mutation=size targets_ok=143 targets_failed=2
  failed_target `-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode`
  failed_target `-p singlefs-harness --test many_inodes_write_and_read`
  test a_new_empty_inode_opens_under_the_read_only_mount_with_no_data_unit ... FAILED
  test creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold ... FAILED
```

- 三行都不是只让 A1 红：`many_inodes_write_and_read` 的 `creating_inodes_across_the_two_hundred_thirty_three_threshold_splits_at_the_end_and_reads_back_cold` 三行都红；nlink 那行另红 4 个目标，size 那行另红 1 个。
- 所以 mutations.tsv 那三行证明的是「A1 也抓得到」，不是「只有 A1 抓得到」；A1 相对 harness 档多出来的判别力在第二截那一支自己的调用参数上（写入时间 0、建 2 个，上面第 9、11 行；这两种 harness 档不经过那一支）。没跑 checker 档包里调 `publish_new_inodes` 的两个测试目标（`crash_enumeration_record_spill_over_stream`、`crash_enumeration_tree_split_streams`），它们在这三行下红不红没量。
- 不算打中：题面问的是牵不牵连，答「牵连」；条文与 diff 里没有一句说这三行只有 A1 抓得到。

## K2-b　B1：136 槽稳不稳、对别的盘面参数敏不敏感

被攻的用例：`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:40`：「const UNIT_AREA_SLOTS: u64 = 136;」。它的前提是续做推满仍放不下、第一个文件那条记录 nlink = 0（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:137`：「Some(NLINK_OF_AN_INODE_BEING_DELETED),」），断言只有一条：结局是被拒、拒因是单元区墙（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:161`：「ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),」）。它对「那一支建 0 个」的判别力全靠「这份盘上空发布落得下、建一个落不下」，而用例里没有一条断言钉住这件事。

探针照用例的造盘法造盘（四个数可调），在每份盘上按用例的前提与断言判三次：正确实现、`OPUS_K2B_SOME=1`（判法只看 Some）、`OPUS_ARM=count-zero`（那一支建 0 个），再在同一份盘上各自新挂一次，分别发建 0 个、建 1 个、建 2 个、覆盖写。分类：GREEN-KILLS-BOTH（正确绿、两行变异都红）、GREEN-KILLS-SOME-ONLY（正确绿、只看 Some 红、建 0 个照绿）、TEST-RED-ON-CORRECT（正确实现上用例就红，前提不成立）。

### 入库那组参数（覆盖写 3、单元 14、批 1），单元区 112–208 槽逐槽（量过，副本）

112–127 槽 mkfs 做不成（环放不下），139 槽起续做走空、前提红。`research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-slots.txt` 第 17–28 行（整行；方括号里是「可用字节、需要字节」）：

```
b1 slots=128 ow=3 units=14 batches=1 class=GREEN-KILLS-SOME-ONLY test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=green create0=Explained(UnitAreaWall)/PlacementRefused[] create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["0", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["0", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=129 ow=3 units=14 batches=1 class=GREEN-KILLS-SOME-ONLY test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=green create0=Explained(UnitAreaWall)/PlacementRefused[] create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["16384", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["16384", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=130 ow=3 units=14 batches=1 class=GREEN-KILLS-SOME-ONLY test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=green create0=Explained(UnitAreaWall)/PlacementRefused[] create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["32768", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["32768", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=131 ow=3 units=14 batches=1 class=GREEN-KILLS-SOME-ONLY test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=green create0=Explained(UnitAreaWall)/PlacementRefused[] create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["49152", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["49152", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=132 ow=3 units=14 batches=1 class=GREEN-KILLS-SOME-ONLY test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=green create0=Explained(UnitAreaWall)/PlacementRefused[] create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["65536", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["65536", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=133 ow=3 units=14 batches=1 class=GREEN-KILLS-SOME-ONLY test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=green create0=Explained(UnitAreaWall)/PlacementRefused[] create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["81920", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["81920", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=134 ow=3 units=14 batches=1 class=GREEN-KILLS-BOTH test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=red(outcome:Published) create0=Published create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["98304", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["98304", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=135 ow=3 units=14 batches=1 class=GREEN-KILLS-BOTH test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=red(outcome:Published) create0=Published create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["114688", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["114688", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=136 ow=3 units=14 batches=1 class=GREEN-KILLS-BOTH test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=red(outcome:Published) create0=Published create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["131072", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["131072", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=137 ow=3 units=14 batches=1 class=GREEN-KILLS-BOTH test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=red(outcome:Published) create0=Published create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["147456", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["147456", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=138 ow=3 units=14 batches=1 class=GREEN-KILLS-BOTH test(correct)=green test(only-Some)=red(reason:Unexplained) test(arm-builds-0)=red(outcome:Published) create0=Published create1=Explained(UnitAreaWall)/SpaceAdmissionRefused["163840", "196608"] create2=Explained(UnitAreaWall)/SpaceAdmissionRefused["163840", "196608"] overwrite=Unexplained/WriteToAnInodeBeingDeleted[]
b1 slots=139 ow=3 units=14 batches=1 class=TEST-RED-ON-CORRECT test(correct)=red(pre:continuation) test(only-Some)=red(pre:continuation) test(arm-builds-0)=red(pre:continuation) create0=not-with-file create1=not-with-file create2=not-with-file overwrite=not-with-file
```

- 136 槽：正确绿、两行变异都红；建 1 个被空间准入拒（可用 131,072、要 196,608），建 0 个做成。b21 攻方那一行的数在这里复现。
- 分得开的那一档是 134–138，136 在正中，两边各差 2 槽（每槽 16 KiB，可用字节逐槽加 16,384）。
- 128–133：用例照绿、只看 Some 照红，而建 0 个被落点拒、照样记成单元区墙——「那一支建 0 个」变异不再红。139 起前提先红，响着失败。

### 换邻近参数（量过，副本）

覆盖写 2–4、单元 13–15、批 1 九组，每组单元区 120–176 槽逐槽，共 513 份盘（`out/b1-neighbourhood.txt`）。每组「分得开」（band）与「照绿而建 0 个不红」（silent）的槽区间，与下一小节的计数是同一个脚本的两段输出（模型目录里执行，原样）：

```
$ python3 tools/summarize_b1.py out
ow=2 units=13 batches=1 band=- silent=- at136=TEST-RED-ON-CORRECT
ow=2 units=14 batches=1 band=- silent=- at136=TEST-RED-ON-CORRECT
ow=2 units=15 batches=1 band=128..128 silent=- at136=TEST-RED-ON-CORRECT
ow=3 units=13 batches=1 band=132..136 silent=128..131 at136=GREEN-KILLS-BOTH
ow=3 units=14 batches=1 band=134..138 silent=128..133 at136=GREEN-KILLS-BOTH
ow=3 units=15 batches=1 band=136..140 silent=130..135 at136=GREEN-KILLS-BOTH
ow=4 units=13 batches=1 band=150..154 silent=144..149 at136=TEST-RED-ON-CORRECT
ow=4 units=14 batches=1 band=152..156 silent=146..151 at136=TEST-RED-ON-CORRECT
ow=4 units=15 batches=1 band=154..158 silent=148..153 at136=TEST-RED-ON-CORRECT
     36 class=GREEN-KILLS-BOTH zero-killed create0-published
     40 class=GREEN-KILLS-SOME-ONLY zero-survives create0-refused
    446 class=TEST-RED-ON-CORRECT zero-killed create0-refused
```

- 分得开的那一档在每组上都恰好 5 槽宽；每多一个数据单元上移 2 槽，每多一次覆盖写上移约 18 槽。
- 136 在入库那组的正中、在单元 13 那组的上沿、在单元 15 那组的下沿。覆盖写次数一变，136 就落进「前提红」那一边。
- 方向：盘面往上漂（同一槽数上空闲多出 ≥ 3 槽）时前提先红，响；往下漂 3–8 槽（48–128 KiB）时用例照绿而「建 0 个」不再红，不响。哪种实现改动会让它漂、漂多少，我没量（推的：分配记录树、实例表、journal 环这些固定开销多占几槽就够）。

四句：

1. 分不分辨：分辨「用例钉住那一支建一个 inode」与「只在今天这一份盘面上恰好钉住」。
2. 系统看不看得到：看得到——用例手里有整份盘面，能在同一份盘上另挂一次发一个空发布。
3. 满足的是哪一个分句：正文 K2 格「单元区槽数对别的参数敏不敏感」。136 槽这一档稳（两边各差 2 槽），但用例没把「分得开」写成前提，漂出去的一边不响。
4. 跑前条款的改法在这几格上还中不中：入库的 B1（槽数改成 136）在今天的盘面上有效，在 128–133 那一类盘面上对「建 0 个」不起作用；下面 B1-c 在两类上都起作用。

### 改法 B1-c（只在我的模型上量过、被攻过零轮）

在用例的前提块里加一条：同一份崩溃盘面新挂一次，`publish_new_inodes(…, 0, …)` 做成（可再加一条：新挂一次发 `publish_new_inodes(…, 1, …)` 被空间准入拒）。量过的是「这条前提成立」与「建 0 个变异会红」在全部用例绿的盘上是不是同一件事，两份 sweep 合起来（去掉造盘失败的，按类、建 0 个变异红不红、建 0 个做没做成三列计数），是上一小节那段输出的最后三行。

用例绿的 76 份盘上，「建 0 个做成」与「建 0 个变异会红」逐份同真同假。加了这条前提，那 40 份「照绿而不红」的盘都变成前提红、响着失败。推的、没实现：前提块里那次新挂用 `mount_writable_with_space_admission` 与用例已有的写法相同，不改发布臂。

## 几个改法各修哪一格

| 改法 | K1-c F3 判别力 | K2-b B1 盘面漂移 | 别的格 |
|---|---|---|---|
| F3-d（A 段号 0..=63 × 4095；B 长 28 后 12 个写；C 历史种子 0 与 1） | 量过：入库那一种三条全绿，六种照绿而有损的改坏每种至少一条红（`f3-fix.txt`，Python 第二条路子 `f3-model.txt` 逐格一致） | — | 不碰登记的 `.take(1)` 那行 |
| B1-c（前提：同一份盘面新挂一次，建 0 个做成） | — | 量过：用例绿的 76 份盘上「建 0 个做成」与「建 0 个变异会红」逐份同真同假；写成用例的样子是推的 | 不碰只看 Some 那行 |

两条都只在我的模型上量过、被攻过零轮。K1-a、K1-b、K2-a 三格没打中，不提改法。

## 没打中的形状

- K3-a：4 种几何（384、256、240 槽与 4 GiB）× 覆盖写长度（一个单元里两种、空、跨三个单元）× 覆盖写 1–3 次，48 段历史、写表 83–179 个写；接错只试了「交第一次崩溃那一版」这一种。没试别的错接法（例如恢复时不看 journal 那一版），它们在这个态上分不分得开不知道。
- A1：第二截那一支的坏发布 9 种（nlink 0、出生代上一版与下一版、size 4096、改动计数上一版、写入时间 0、建 0 个、建 2 个、旧实例代号）全红；整次发布跳一个 txg 被分配器自检拦下；三行 core 变异全红。没试：改已有那条 inode 2 的记录（要动 `publish_version`），编码器把九样盘上字段写错（推的：A1 看不见，见 K2-a）。
- B1：只看 Some 那一臂在用例绿的 76 份盘上全红；入库参数邻近九组 513 份盘。没换 `OVERWRITE_BYTES`、journal 环大小与设备宽度。
- F3：17 种种子算法；反着折、截成 u32、去掉盐、以加代替异或四种照绿、我这几项也量不出损。

## 这条腿自己的限度

- 全部数是副本上的数，主 agent 要在入库装置上重做：K3-a、A1、B1、F3-c 都是 checker 档用例，归提交时跑。
- 探针包链着 checker 档的库跑注入与挂载函数，没起 checker 档包的任何测试目标与 `src/bin/` 装置，也没调层 0 枚举函数（`enumerate_layer0` 一次没调，所以没有用 `closed_form_state_count` 算状态数）；跑的崩溃态只有写死的靶向态：K3-a 48 段历史 × 两种接线、A1 14 种坏发布各一态。主 agent 若判这种用法算 checker 档，K1、K2 的数降为推的。
- 线程：探索阶段有一批 B1 邻近 sweep 并行起了 9 件（各单线程，约 7 秒），超过线程上限 7；探索阶段的 k3a、f3、a1、b1 没经 `capped.sh`（注入线程变量没设）。之后照 `tools/rerun.sh` 重跑一遍：探针一律经 `capped.sh 1`、cargo 经 `capped.sh 7`、并行至多 6 件，产物与探索那一遍 `cmp` 相同（a1、b1-slots 只差 stderr 行）。
- 内存包装：有一次 `4G` 上限超过那条命令实测峰值 28M 的 2 倍，包装拒跑（「命令一行都没跑」），那一次不算；之后带 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 重跑。一次 `timeout 235` 截断了 build-tests（cargo 没跑完，退出码 124），没留下进程，接着再起一次编完。
- `ps` 开工时看到别的会话在跑 `cargo test -p singlefs-harness --test b23_se…` 与一条 `cargo build --release`，交回前看到别的会话的 `cargo build … -p singlefs-checker-tier --bin e…`；我的编译走自己的 target 目录，没有等锁。
- F3 的真历史用 `generate_history`（BROAD 比重）、384 槽与 4 GiB 两种几何；战役真实的比重、几何、步数不同，段长分布会变。
- 每格只抽了一次（这条腿只跑一次），「没打中」那几格照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算没打中。

## 草稿与副本

- 仓副本与编译目录 `/tmp/claude-1000/m3-formal-code-b23-r1-opus/copy/`（约 16 GB）交回前删掉。
- 留在草稿目录 `/tmp/claude-1000/m3-formal-code-b23-r1-opus/` 的只有日志与小产物：`out/`（含四次 collateral 的整份 cargo 日志 `collateral-*.log`、`stderr.log`）、`out-explore/`（探索那一遍）、`model/`（模型目录的草稿）、`*.log`、`scratch-setup.sh`。入库的那一份是模型目录 `out/`。

## 要主 agent 跑的命令

1. 复跑核对：仓根执行 `WORK=<主 agent 的草稿目录> THREADS=7 bash research/prompts/m3-formal-code-b23-r1-opus-model/tools/rerun.sh all`；内存上限由脚本内部给 `run-with-memory-cap.sh`（编译与 harness 测试 8G、探针 4G）；全程约 20 分钟（冷编约 6.5 分钟，collateral 四次约 10 分钟），超过前台时限就按段起。看 `$WORK/out/` 下九个 `.txt` 与模型目录 `out/` 逐个 `cmp`；k3a 看 `k3a-variants.txt` 第 1 行两段 `inlined_test=`；f3 看 `f3-fix.txt` 第 1 行全 green；b1 看 `b1-slots.txt` 第 22–28 行的 class。
2. B1 汇总：`python3 research/prompts/m3-formal-code-b23-r1-opus-model/tools/summarize_b1.py $WORK/out`，与本报告 K2-b 那段输出逐行比。
3. 若采纳 F3-d、B1-c：写成入库用例之后在提交时跑 checker 档（不归我跑）。

## 没做什么

- 没判正推腿的 K1、K3，没读禁读清单里的文件。
- 没跑任何 checker 档包的测试目标、层 0、变异表；23-1 那几行变异在入库装置上的红绿没量（副本上用同义的环境变量开关量）。
- 没把改法写进仓里任何 crates 文件；冻结副本一个字没动。
- 没量哪种实现改动会让 B1 的盘面漂、漂多少；没量入库种子跨历史撞种子的代价。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b23-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b23-r1/tree --root . --background research/prompts/_m3-formal-code-b23-r1-background.md` 末行：

```
  ✓ 核了 28 处引文，对上 28 处，没判 0 处（逐处列在上面）
```
