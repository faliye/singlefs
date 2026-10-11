# m3-formal-code-b24-r1 云端攻方（Opus）报告（2026-10-05）

攻击面：K1（F3-d 三条用例的取样覆不覆盖真历史会走到的段号与段长；六行变异是不是各自只让点名那条红；有没有第七种有害改坏三条都放过）、K2（B1-c 新前提是不是真钉住「那一支建 0 个」那行变异的判别力；四处注释与代码对不对得上）。代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b24-r1/tree/`，下文 `crates/…` 路径都指那棵树。量出来的数都是我自己拷出的副本上的数（副本 = 冻结副本的 crates/ 加仓根 Cargo.toml、Cargo.lock，挪开 E171R1 半成品，用 `tools/patch_copy.py` 打两个环境变量开关、挂两个只在副本里的探针包），主 agent 要在入库装置上重做。没跑 checker 档包的任何测试目标与装置。

这条腿接前一条撞了限额的同立场腿：它落下的模型目录（`tools/`、`probe/`）与草稿目录里的副本我逐个读过、核过，`out/` 是空的、报告一字没写；下文的数全是这一条腿跑的。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| K1 取样覆不覆盖真历史的段号 | 打中 | 量过（副本） | A 只取段号 0..=63；战役大档 3942 个第一次崩溃态里 2018 个段号 > 63，最大 210（候选 235） |
| K1 取样覆不覆盖真历史的段长 | 长度够、位置不够（打中的一半） | 量过（副本） | B 长 28，真段最长 30（与注释同）；但 B 只变末 12 个位置，前 16 个位置固定全落 |
| K1 六行变异是不是各自只让点名那条红 | 点名那条都红，数与注释逐个同（没打中） | 量过（副本） | 按位拼接、或代异或另红 C，C 的注释自己写了这两个数；F3-c 原用例六行下全绿 |
| K1 第七种有害改坏三条都放过 | 打中 | 量过（副本） | 段号只取低 6 位（稠密 24 段内撞 1890、战役各档内撞 29）、按位拼接落在第 6 位起（1258、6）、只折末 12 个写（310）、跳过第 12..16 个写（120）、只折末 16 个写（13），历史种子只取最低位 / 截成 u8（跨历史损，与登记的「去历史种子」同量级）——F3-c、A、B、C 四条全绿 |
| K1 改法 F3-e（A2、B3、C2） | 攻方提的，零轮 | 量过（副本） | 入库那一种三条全绿；登记七行与攻方九行每行至少一条红 |
| K2 B1-c 钉没钉住「建 0 个」 | 钉住（没打中） | 量过（副本）+ 读码 | 前提就是那一支的那次空发布；宽扫 20898 份盘上带 B1-c 绿的 606 份两行变异都红，转红的 654 份全是旧用例本来放过「建 0 个」的 |
| K2 B1 第 41 行与第 107–111 行注释 | 对（没打中） | 量过（副本） | 128 槽空发布 `PlacementRefused`；128..=133 转红、134..=138 分得开、133 那行变异红在前提 |
| K2 H2 常数注释 | 对（没打中） | 量过（副本） | S = 5、8、15，k = 2..=40 都是 3k − 1，k = 1 是 3；S = 4、16 编 singlefs-harness 退 101 |
| K2 root_record.rs 卸载记号 | 对（没打中） | 推的（读码） | 记录取这次写的根上的记号、恢复施加时取记录里的、卸载那一串的抬 F 记号取 1 |
| K2 C670 模块文档 | 对（没打中） | 量过（副本） | 「换一份盘面照常比可以为真」：只落轮换 + 盘 0 两槽撕裂的盘上 C670 那一条为真、判 `Illegal` |

## 复跑与文件 sha256

模型目录 `research/prompts/m3-formal-code-b24-r1-opus-model/`。仓根执行，分段命令、每段挂钟与看哪几行见文末「要主 agent 跑的命令」；一行全跑：

```
WORK=<空的草稿目录> bash research/prompts/m3-formal-code-b24-r1-opus-model/tools/rerun.sh all
```

可覆盖的环境变量：`REPO_ROOT`（默认按脚本位置推出仓根）、`FROZEN_TREE`（默认 `/tmp/claude-1000/m3-formal-code-b24-r1/tree`）、`WORK`、`THREADS`（默认 10）、`BUILD_CAP`（默认 3G）、`RUN_CAP`（默认 256M）。副本上的开关（`tools/patch_copy.py`；不设环境变量时副本行为与冻结副本相同）：`OPUS_ARM=count-zero`（第二截「带文件而第一个文件不是活记录」那一支建 0 个，与 `crates/mutations.tsv` 23-1 改法 B1 那一行同义）、`OPUS_K2B_SOME=1`（判法只看 `inode_record` 是不是 Some，与改法 b 那一行同义）。F3 那一段不用开关：`tools/gen_seedfn.py` 把变异行施加到冻结副本源文本上再原样抽出种子函数与用例。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`）：

```
682298cea2140602d734f6d820d0da6d019217d4aae2323b8cb0d95d8627d889  research/prompts/m3-formal-code-b24-r1-opus-model/out/b1-slots.txt
3319e532383c1c7687f4047e86aa39cbecd33f08d79b07cba11e1fb96a6db9c0  research/prompts/m3-formal-code-b24-r1-opus-model/out/b1-summary.txt
fcd40c094b4513b70b9047228f2f9bc9155ad9f6c8e05e791d09f0c2d484ebac  research/prompts/m3-formal-code-b24-r1-opus-model/out/b1-wide.txt
726b642daeda62fe3325dfda83f8da958ad10759fc8843cf80d619649e720139  research/prompts/m3-formal-code-b24-r1-opus-model/out/c670.txt
ce0c6987f0fbec7c790a0b1dbcca625120e4e0eb237a01c30bc3f977a696609f  research/prompts/m3-formal-code-b24-r1-opus-model/out/f3.txt
f26393e1fa290081f5ef7173fcb16776fd59f3712df4594efd7eef0163fdf4c3  research/prompts/m3-formal-code-b24-r1-opus-model/out/h2-compile-s16.txt
f4ae49a1cdab8cf273a3da2fe70b706cc03ff13614d5a3465ef23b09cb3348e4  research/prompts/m3-formal-code-b24-r1-opus-model/out/h2-compile-s4.txt
59ec20f903107aa23224994fcc71d45ae3268a7ba5ce7e78cc250a31f5b5b44c  research/prompts/m3-formal-code-b24-r1-opus-model/out/h2-txg-s15.txt
d560231311933265b1b39af7dc7e283c2e3526faf0b79441b0b4ea47a1e79634  research/prompts/m3-formal-code-b24-r1-opus-model/out/h2-txg-s5.txt
814e62fc19e9d0d5c124754a904e9860693d366ea339734fffd91ee3a3258cb2  research/prompts/m3-formal-code-b24-r1-opus-model/out/h2-txg.txt
c7efbae9cc553ab542ab38f1520067f3d084a10c3c04896a02198cd2d179ef0e  research/prompts/m3-formal-code-b24-r1-opus-model/out/real-dense.txt
88a53fd09fff5c3020e6ab0cc7ea97968fb4f9a9557796b5139982e183d12c16  research/prompts/m3-formal-code-b24-r1-opus-model/out/real-fast.txt
b1130e8ed9077040de9bf67353014396ae1d3509a545f0a14dc4100317c6336b  research/prompts/m3-formal-code-b24-r1-opus-model/out/real-large.txt
3d9e80120bceb63b9cc29af60068b5ab82ae93d9ce124e0f1c8066e5b7ef42d8  research/prompts/m3-formal-code-b24-r1-opus-model/out/real-rollback.txt
f220950f5afb24f7b1d037d585aadabdad00df8ac6f1a280de5d56881aaba10b  research/prompts/m3-formal-code-b24-r1-opus-model/out/real-threads.txt
410b6e4f7eed1b04a989a3ce34a2214e325f564519494a0e4e4f4a1a1600bf99  research/prompts/m3-formal-code-b24-r1-opus-model/out/segments.txt
c592f80723dadd78285a50f132aaa417621dc878f7415d028f5fc1d729309d73  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-harness-probe/c670_extra.rs.in
6e9ad065d8bde1a75b24f69874e3e2f3f5230ead8b3808e2601f84523cba388a  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-harness-probe/Cargo.toml
a15a37a2a8a2de1443ec53fdc64da218a819db00ff3b8f2e006e34de2229080c  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-harness-probe/src/lib.rs
1ee1438f527fb22d776421d2474ffd5bbebde8381d35dbb2afb2b0f8d4c983da  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-harness-probe/tests/h2_txg.rs
a8b2e7cd6135c0d2ab6cc238bf4102dd0fd4efa6eaffe06dcab2181f14f9751d  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-probe/Cargo.toml
d3a6bb02fd142e1c59ccd5cd859f695bf9e0cdf6e4409d334d55f68237bea167  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus_b24_probe.rs
829608f81ead0d1f9fe4c869223aa9c806ddea957d8c6db75bd05df89cf7f9e7  research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-probe/src/main.rs
f4b1d858b7f483b56bd70e95b5663c71a08fdee20870df5d05e235612765c9ac  research/prompts/m3-formal-code-b24-r1-opus-model/tools/gen_seedfn.py
e7d9c42a02f76843a29690b47f266b6dcb24218b257b3b57bfa2d7f8d3a29964  research/prompts/m3-formal-code-b24-r1-opus-model/tools/opus-mutations.tsv
0e4fb79eff0132796a6e473b601247e4a1f27f1132b6bbece383b71c111d6b6e  research/prompts/m3-formal-code-b24-r1-opus-model/tools/patch_copy.py
1e3ec0419741c19c958560181efa4773cfef73d42a84c87da88c932013244bea  research/prompts/m3-formal-code-b24-r1-opus-model/tools/rerun.sh
95616a1b3b6d48fadaaf01fd7532366be6550a97f73807ec7eb08a4478c27d82  research/prompts/m3-formal-code-b24-r1-opus-model/tools/summarize_b1.py
83fe1708e5db24c1ca381d1da46d94a2ec9ce5bf6b39e0b685e81ee1d91ac71b  research/prompts/m3-formal-code-b24-r1-opus-model/tools/summarize_f3.py
f13c963f547d671f55522c33876bdba156e0f55519f50994f4379fcc75c4f162  research/prompts/m3-formal-code-b24-r1-opus-model/tools/summarize_segments.py
```

## K1　F3-d：取样覆盖、六行变异各红在哪、第七种有害改坏

### 真历史走到的段号与段长（量过，副本）

探针 `opus-b24-probe real` 照 `inject_crashes_into_history_drawing_second_crashes` 的前半段跑一段生成历史、按同一份几何切段、用同一个 `draw_crash_points` 摆第一次崩溃（不恢复、不挂载、不跑 checker：二次崩溃的种子只由历史种子、段号与段内子集算）。档位照战役各档（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1147`：「seed_count: 500,」，`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1148`：「operations_per_history: 40,」，`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1150`：「crash_points_per_history: 8,」，种子基 `crates/singlefs-harness/src/history.rs:103`：「pub const SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE: u64 = 7_463_871_032_432_355_113;」），另加一份稠密摆法（短于等于 12 个写的段全部真子集、更长的段各抽 64 个，24 段 40 步）。`out/segments.txt` 整份：

```
real-fast.txt operations=24 weights=broad width=4g points=4 histories=24 candidate_segments=1894 max_candidate_index=133 candidates_with_index_over_63=596 candidates_with_index_over_255=0 longest_candidate=30 candidates_longer_than_12=399 longer_than_28=3 drawn_states=92 max_drawn_index=112 drawn_with_index_over_63=31 longest_drawn=28 drawn_longer_than_12=12 drawn_longer_than_28=0 candidate_length_histogram={1: 464, 2: 1027, 12: 4, 16: 111, 18: 37, 20: 64, 22: 18, 24: 100, 26: 8, 28: 58, 30: 3}
real-rollback.txt operations=20 weights=rollback width=4g points=6 histories=12 candidate_segments=878 max_candidate_index=99 candidates_with_index_over_63=178 candidates_with_index_over_255=0 longest_candidate=28 candidates_longer_than_12=189 longer_than_28=0 drawn_states=70 max_drawn_index=93 drawn_with_index_over_63=17 longest_drawn=28 drawn_longer_than_12=12 drawn_longer_than_28=0 candidate_length_histogram={1: 215, 2: 473, 12: 1, 16: 54, 18: 4, 20: 17, 22: 6, 24: 60, 28: 48}
real-threads.txt operations=16 weights=broad width=4g points=3 histories=8 candidate_segments=425 max_candidate_index=95 candidates_with_index_over_63=70 candidates_with_index_over_255=0 longest_candidate=28 candidates_longer_than_12=86 longer_than_28=0 drawn_states=24 max_drawn_index=76 drawn_with_index_over_63=1 longest_drawn=28 drawn_longer_than_12=4 drawn_longer_than_28=0 candidate_length_histogram={1: 105, 2: 233, 12: 1, 16: 21, 18: 7, 20: 17, 22: 5, 24: 28, 26: 4, 28: 4}
real-large.txt operations=40 weights=broad width=4g points=8 histories=500 candidate_segments=68335 max_candidate_index=235 candidates_with_index_over_63=39281 candidates_with_index_over_255=0 longest_candidate=30 candidates_longer_than_12=15160 longer_than_28=49 drawn_states=3942 max_drawn_index=210 drawn_with_index_over_63=2018 longest_drawn=30 drawn_longer_than_12=893 drawn_longer_than_28=3 candidate_length_histogram={1: 16485, 2: 36604, 12: 86, 16: 4964, 18: 1360, 20: 2072, 22: 767, 24: 2910, 26: 199, 28: 2839, 30: 49}
real-dense.txt operations=40 weights=broad width=4g points=0 histories=24 candidate_segments=3266 max_candidate_index=235 candidates_with_index_over_63=1856 candidates_with_index_over_255=0 longest_candidate=30 candidates_longer_than_12=724 longer_than_28=5 drawn_states=68747 max_drawn_index=235 drawn_with_index_over_63=30960 longest_drawn=30 drawn_longer_than_12=46331 drawn_longer_than_28=320 candidate_length_histogram={1: 789, 2: 1749, 12: 4, 16: 233, 18: 70, 20: 132, 22: 35, 24: 138, 26: 10, 28: 101, 30: 5}
```

- 段号：A 只取 0..=63（`crates/singlefs-checker-tier/src/crash_injection.rs:4484`：「const SEGMENT_INDEXES: std::ops::RangeInclusive<usize> = 0..=63;」）。大档 3942 个摆出的第一次崩溃态里 2018 个段号大于 63，最大 210；候选段最大 235。四档都没有超过 255 的。
- 段长：B 取 28（前 16 个写落、末 12 个写变，`crates/singlefs-checker-tier/src/crash_injection.rs:4515`：「const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;」）。大档最长候选段 30、摆出的态里 3 个长 30。B 的注释说「真历史里的候选段长到 26（384 槽）/ 30（4 GiB）个写」（`crates/singlefs-checker-tier/src/crash_injection.rs:4508`），4 GiB 那一半在 40 步的历史上照样对。
- 历史种子：C 只取 0 与 1（`crates/singlefs-checker-tier/src/crash_injection.rs:4535`：「const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];」），两者只差最低一位；战役的种子是 64 位的种子基加段序号。

### 种子算法矩阵（量过，副本 + 原样抽出的用例）

做法：`tools/gen_seedfn.py` 从冻结副本的 `crash_injection.rs` 原样抽出常量、种子函数与 F3-c、F3-d 四条用例（用例去掉 `#[test]`、函数体一字不改），抽之前先把一行变异施加到源文本上（原文要恰好命中一次，命中不是一次退 3），编进探针包跑。变异行取冻结副本 `crates/mutations.tsv` 的 F3-c（23-1）一行与 F3-d（24-1）六行，加攻方自己的九行（`tools/opus-mutations.tsv`，同一种写法）。每种还在上一小节的战役各档（合起来 544 段历史、4128 个态）与稠密那一份（24 段、68747 个态）上数同一段历史内撞种子的态数，并跑攻方改法 F3-e 三条（见下文）。`python3 tools/summarize_f3.py out/f3.txt` 整份：

```
pristine [入库那一种] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=green(528/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=0
tsv:2408 [F3-c（23-1）] F3c=red(left=2) A=red(left=128) B=red(left=1) C=red(left=4) | A2=red(188/384930) B3=red(2/528) C2=red(6/12285) | campaign_sharing=26 campaign_distinct=1122/4128 dense_sharing=63004
tsv:2436 [F3-d（24-1）] F3c=green A=green B=red(left=1) C=green | A2=green(384930/384930) B3=red(79/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2644/4128 dense_sharing=324
tsv:2437 [F3-d（24-1）] F3c=green A=green B=red(left=1) C=green | A2=green(384930/384930) B3=red(137/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=19
tsv:2438 [F3-d（24-1）] F3c=green A=red(left=4096) B=green C=red(left=4096) | A2=red(40960/384930) B3=green(528/528) C2=red(8191/12285) | campaign_sharing=12 campaign_distinct=1990/4128 dense_sharing=10200
tsv:2439 [F3-d（24-1）] F3c=green A=red(left=4095) B=green C=green | A2=red(4095/384930) B3=green(528/528) C2=green(12285/12285) | campaign_sharing=1472 campaign_distinct=2547/4128 dense_sharing=14221
tsv:2440 [F3-d（24-1）] F3c=green A=green B=green C=red(left=4095) | A2=green(384930/384930) B3=green(528/528) C2=red(4095/12285) | campaign_sharing=0 campaign_distinct=1595/4128 dense_sharing=0
tsv:2441 [F3-d（24-1）] F3c=green A=red(left=13254) B=green C=red(left=5998) | A2=red(39696/384930) B3=green(528/528) C2=red(8021/12285) | campaign_sharing=1332 campaign_distinct=923/4128 dense_sharing=14496
opus-tsv:2 [攻方 M7a] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=red(79/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=310
opus-tsv:3 [攻方 M7b] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=red(137/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=13
opus-tsv:4 [攻方 M7c] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=red(407/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=0
opus-tsv:5 [攻方 M7d] F3c=green A=green B=green C=green | A2=red(262080/384930) B3=green(528/528) C2=green(12285/12285) | campaign_sharing=29 campaign_distinct=2618/4128 dense_sharing=1890
opus-tsv:6 [攻方 M7e] F3c=green A=green B=green C=green | A2=red(278460/384930) B3=green(528/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=0
opus-tsv:7 [攻方 M7f] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=green(528/528) C2=red(8190/12285) | campaign_sharing=0 campaign_distinct=1599/4128 dense_sharing=0
opus-tsv:8 [攻方 M7g] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=green(528/528) C2=red(8190/12285) | campaign_sharing=0 campaign_distinct=1895/4128 dense_sharing=0
opus-tsv:9 [攻方 M7h] F3c=green A=green B=green C=green | A2=red(286652/384930) B3=green(528/528) C2=green(12285/12285) | campaign_sharing=6 campaign_distinct=2009/4128 dense_sharing=1258
opus-tsv:10 [攻方 M7i] F3c=green A=green B=green C=green | A2=green(384930/384930) B3=red(407/528) C2=green(12285/12285) | campaign_sharing=0 campaign_distinct=2645/4128 dense_sharing=120
```

攻方九行：M7a 指纹只折段内末 12 个写，M7b 末 16 个，M7c 前 28 个，M7d 段号只取低 6 位（`& 63`），M7e 段号截成 u8，M7f 历史种子只取最低位，M7g 历史种子截成 u8，M7h 指纹改成按位拼接、每一位落在第 6 位起（`(h << 1) ^ (b << 6)`），M7i 指纹跳过段内第 12..16 个写。`campaign_distinct` 把各档同一种子基起的历史并在一起数，各档的头几段共用种子，这一列只作同一行内与入库那一种比的参照。

### 六行变异各红在哪（没打中）

- 六行各自让点名的那条红，数与正文第二节、用例注释逐个同：前 12 / 前 16 个写 B = 1、1；按位拼接、去段号、或代异或 A = 4096、4095、13254；去历史种子 C = 4095。
- 不是「只让点名那条红」的有两行：按位拼接另红 C（4096），或代异或另红 C（5998）。C 的注释自己写了这两个数（`crates/singlefs-checker-tier/src/crash_injection.rs:4532`：「/// 攻方在模型上量过它剩 4095 个，按位拼接剩 4096 个、四样改成或剩 5998 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 C 那一列）。」），变异表最后一栏只点名一条，点名那条红就够，这不是打中。
- F3-c 原用例（`F3c`）在六行下全绿，与正文第二节同。

### 第七种有害改坏：三条都放过（打中，量过）

按「同一段历史内撞种子的态数」这把尺子（b23 攻方判 take(12)、take(16) 有害用的就是它，稠密那一份上它们是 324、19），攻方九行里三条用例全绿而有损的：

| 改坏 | F3-c、A、B、C | 稠密 24 段内撞 | 战役各档内撞 | 为什么放过 |
|---|---|---|---|---|
| M7d 段号只取低 6 位 | 全绿 | 1890 | 29（27 段历史） | A 的段号只到 63，`& 63` 在这个范围里是恒等；大档一半的态段号 > 63 |
| M7h 按位拼接落在第 6 位起 | 全绿 | 1258 | 6 | 同上：段号 0..=63 住低 6 位，与子集不撞；登记的按位拼接红在 A 只因为它从第 0 位起拼 |
| M7a 只折末 12 个写 | 全绿 | 310 | 0 | B 只变末 12 个写、前 16 个固定全落，丢前面的位它看不见；A、C 的段长 12 |
| M7i 跳过段内第 12..16 个写 | 全绿 | 120 | 0 | 同上：B 那 16 个固定位里丢哪几个都看不见 |
| M7b 只折末 16 个写 | 全绿 | 13 | 0 | 同 M7a（入库登记的「只折前 16 个写」是 19） |
| M7f 历史种子只取最低位 | 全绿 | 0 | 0 | C 的两个历史种子 0、1 只差最低位；损在跨历史：战役各档合起来不同种子 1599，入库 2645，与登记的「去历史种子」（1595）同一量级 |
| M7g 历史种子截成 u8 | 全绿 | 0 | 0 | 同上，1895 |

另两行三条都放过、我量不出损：M7c 只折前 28 个写（长 30 的段只有 49 个候选、摆出 3 个；稠密那 320 个长 30 的态里没撞上），M7e 段号截成 u8（四档段号都没过 255）。它们今天无害，段再长、段号再大才有损（推的）。

四句：

1. 分不分辨：分辨「三条用例钉住了真历史会用到的整段号、段内每一个位置与整个历史种子」与「只钉住段号低 6 位、段内前若干位与历史种子最低位」。
2. 系统看不看得到：看得到。种子函数是纯函数，用例手里有全部输入，挑大段号、挑扣下前面几个写的子集、挑高位不同的历史种子都只是换常量。
3. 满足的是哪一个分句：正文 K1 格「三条用例的取样是不是覆盖了真历史会走到的段号与段长」（段号：不覆盖，0..=63 对大档最大 210、候选 235；段长：长度 28 覆盖到 30 之内，但只变末 12 个位置）与「还有没有第七种有害改坏三条都放过」（有，上表前七行）。
4. 跑前条款的改法在这几格上还中不中：正文第五节写「打中而坐实 ⇒ 这是最后一轮，改法若是零轮形态，写进判决的交用户表或记进欠账，不再开轮」。入库的三条对上表前七行不起作用；下面 F3-e 在这七行上都红，入库那一种全绿。

推翻条件：在入库装置上把 M7a、M7d、M7f 任何一行施加到 `crash_injection.rs`，F3-c、A、B、C 四条里有一条红（说明我抽出来跑的不是同一段源码）；或战役各档摆出的第一次崩溃态里段号都不超过 63、段内只在末 12 个位置上有差（说明 A、B 的取样覆盖了真历史）。

### 改法 F3-e（只在我的模型上量过、被攻过零轮）

三条，都只调 `seed_of_the_second_crash_points`，与 F3-d 同一种写法（探针 `f3-proposed` 子命令是它的原型，`probe/opus-b24-probe/src/main.rs` 的 `f3_proposed`）：

- A2：历史种子 0，段号 0..=63 再加 2^k 与 2^k + 1（k = 6..=20），长 12 的段 4095 个子集，384930 个种子两两不同。罩住段号每一个高位，不随真历史的最大段号再改。
- B3：历史种子 0、段号 16、长 32 的段上扣下 1 个或 2 个写的全部 528 个子集，种子两两不同。指纹丢掉段内任何一个位置 i，扣下 {i, j} 与扣下 {j} 就撞，不管丢的是前、后还是中间；32 比量到的最长段 30 长。
- C2：段号 16、长 12 的 4095 个子集在历史种子 0、1、2^63 上各算一遍，12285 个两两不同。

上面矩阵的 `A2 B3 C2` 三列：入库那一种三条全绿；登记的七行与攻方九行，每行至少一条红（M7c、M7e 也红，它们今天无害，红了不亏）。B3 一条就罩住 B 那两行（前 12、前 16）与 M7a、M7b、M7i，可以替掉 B 或与 B 并存。代价：三条合起来约 40 万次种子函数，与 A 今天的 26 万次同一量级（推的，没在 checker 档量挂钟）。

## K2　B1-c 与四处注释

### B1-c 新前提钉没钉住「那一支建 0 个」（没打中，量过 + 读码）

读码：新前提与被变异的那一支是同一次计算。前提块另挂一次同一份盘面、发 `publish_new_inodes(…, 0, FIXED_WRITE_TIME_SECONDS, mounted.output.instance)`（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:164`：「            0,」）；第二截那一支用同一个挂载入口、同一份参数与空间准入开关（`crates/singlefs-checker-tier/src/crash_injection.rs:1816`：「        mount_writable_with_space_admission(parameters, &mut devices, execution.space_admission);」），发的也是同一个函数、同一个写入时间与实例代号（`crates/singlefs-checker-tier/src/crash_injection.rs:1887`：「                    FIXED_WRITE_TIME_SECONDS,」，`crates/singlefs-checker-tier/src/crash_injection.rs:1888`：「                    mounted.output.instance,」），「建 0 个」变异只把那里的 1 换成 0。两边只差设备外面套没套录制层。所以在正确实现上：前提成立 ⇔ 变异那一支的空发布做成 ⇔ 结局是 `Published`、用例红在解构结局那一处；前提不成立 ⇔ 正确实现上用例先红在前提。用例照绿而「建 0 个」不红的盘面，在带 B1-c 的用例里推不出来。

量过：探针在每份盘面上判六次（不带 B1-c / 带 B1-c × 正确实现、`OPUS_ARM=count-zero`、`OPUS_K2B_SOME=1`），另单发一次前提那个空发布记它的结局。分类：`NOW-GREEN-KILLS-BOTH`（带 B1-c 用例绿、两行变异都红）、`NOW-GREEN-ZERO-SURVIVES`（带 B1-c 用例绿而「建 0 个」照绿，这一格就是打中）、`NOW-GREEN-SOME-SURVIVES`、`NOW-RED-AT-B1C-OLD-ZERO-SURVIVED`（B1-c 让旧用例转红，旧用例在这份盘上本来放过「建 0 个」）、`NOW-RED-AT-B1C-OLD-KILLED-ZERO`（B1-c 让旧用例转红，而旧用例本来分得开：前提误伤）、`RED-ON-CORRECT-BEFORE-B1C`（旧前提就红）、`BUILD-FAILED`（造盘做不成，112–127 槽 mkfs 放不下环）。两份扫描：入库那组参数（覆盖写 3 次、2999 字节、14 个单元、1 批）单元区 112..=208 逐槽；宽扫覆盖写 1、3、5 次 × 长度 100、2999、9000 字节 × 批 1、2、3 × 单元 4、8、12、14、16、20 × 单元区 112..=240 逐槽。`python3 tools/summarize_b1.py out/b1-slots.txt out/b1-wide.txt` 的计数行（每组的槽区间整份在 `out/b1-summary.txt`）：

```
b1-slots.txt disks=97 BUILD-FAILED=16 NOW-GREEN-KILLS-BOTH=5 NOW-RED-AT-B1C-OLD-ZERO-SURVIVED=6 RED-ON-CORRECT-BEFORE-B1C=70
  b1c_refused class=NOW-RED-AT-B1C-OLD-ZERO-SURVIVED publish0=Explained(UnitAreaWall)/PlacementRefused disks=6
b1-wide.txt disks=20898 BUILD-FAILED=3741 NOW-GREEN-KILLS-BOTH=606 NOW-RED-AT-B1C-OLD-ZERO-SURVIVED=654 RED-ON-CORRECT-BEFORE-B1C=15897
  b1c_refused class=NOW-RED-AT-B1C-OLD-ZERO-SURVIVED publish0=Explained(UnitAreaWall)/PlacementRefused disks=654
```

- 宽扫 20898 份盘面（含入库那组参数的 112..=240 槽，与第一份重叠），`NOW-GREEN-ZERO-SURVIVES`、`NOW-GREEN-SOME-SURVIVES`、`NOW-RED-AT-B1C-OLD-KILLED-ZERO` 都是 0：带 B1-c 的用例绿的 606 份盘上两行变异都红；B1-c 让旧用例转红的 654 份，全是旧用例本来就放过「建 0 个」的（空发布被落点拒，`Explained(UnitAreaWall)/PlacementRefused`）。第一份 97 份同样三类都是 0。
- 入库那组：分得开 134..=138，B1-c 转红 128..=133，与注释同（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:109`：「/// 这条前提成立与「建 0 个」变异红逐份同真同假；单元区 128..=133 槽（覆盖写 3 次、14 个单元、1 批）用例照绿而「建 0 个」不红，」）。133 槽带 B1-c 正确实现红在前提，`UNIT_AREA_SLOTS` 136 → 133 那行变异照注释红（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:111`）。
- 宽扫 123 组有「分得开」那一档的，120 组恰好 5 槽宽，3 组 2 槽宽（128..=129，下沿是 mkfs 做得成的最低槽 128，被截了），覆盖写长度 100、2999、9000 三档逐组相同（这几档覆盖写占的单元数一样，推的）。往下漂的那一侧今天由 B1-c 响着报出来，不再静默。

四句（没打中，照写）：不分辨——带 B1-c 的用例在两份扫描（20898 + 97 份，有重叠）上与「建 0 个」变异逐份同向；系统看得到——前提就是那一支的那次空发布；分句——正文 K2 格「盘面往别的参数漂时，前提与变异同真同假」；改法——不需要。推翻条件：找到一份盘面，带 B1-c 的用例在正确实现上绿而 `OPUS_ARM=count-zero` 也绿（两边的挂载或发布参数分了岔才可能）。

### 四处注释与代码（没打中；三处量过，一处读码）

**B1 文件第 41 行**（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:41`：「/// 上建 0 个也被落点拒（`PlacementRefused`）、照样记 `UnitAreaWall`，分不开那一支建几个。」）：对。上一小节 `out/b1-slots.txt` 的 128 槽那一行，前提那个空发布的结局是 `Explained(UnitAreaWall)/PlacementRefused`，旧用例在「建 0 个」下照绿。同文件第 107–111 行的新注释（76 份、128..=133、133 那行变异）也逐句对得上上一小节的数。一处不在这四处里、只记一笔：造盘函数里两句 expect 消息写死「单元区 136 槽」（`crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs:63`：「            .expect("准入关着、单元区 136 槽：覆盖写落得下");」），133 那行变异施加时消息与盘面不符；它只在造盘失败时打出来，不影响判定。

**H2 常数注释**（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:60`：「/// txg 3k − 1，从 k ≥ 2 起成立（第 1 次挂载上是 3、第 2 次 5、第 3 次 8：正推量过，`research/prompts/m3-formal-code-b23-r1-sonnet-model/out/h2-txg.log`）；」）：量过，对。harness 档探针 `tests/h2_txg.rs` 照用例的搭法（只做过 mkfs 的池接连可写挂载，第 k 次挂载之后在盘面的拷贝上写第一个文件），k = 1..=40；S 换成 5、15 是在第二份副本里只改格式常量 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`。每个 S 一行（文件名、成立的 k 数 / 总数、不成立的那一个）：

```
h2-txg-s5.txt 39/40 k=1 first_file_txg=3
h2-txg.txt 39/40 k=1 first_file_txg=3
h2-txg-s15.txt 39/40 k=1 first_file_txg=3
```

翻转点那几行（行首到 `oldest_valid_root_before`；`oldest_valid_root_before` 是探针按 txg − 3S 算的，不是从根环读出来的）：

```
h2 S=5 k=6 first_file_txg=17 three_k_minus_one=17 holds=true oldest_valid_root_before=Some(2)
h2 S=5 k=7 first_file_txg=20 three_k_minus_one=20 holds=true oldest_valid_root_before=Some(5)
h2 S=8 k=9 first_file_txg=26 three_k_minus_one=26 holds=true oldest_valid_root_before=Some(2)
h2 S=8 k=10 first_file_txg=29 three_k_minus_one=29 holds=true oldest_valid_root_before=Some(5)
h2 S=15 k=16 first_file_txg=47 three_k_minus_one=47 holds=true oldest_valid_root_before=Some(2)
h2 S=15 k=17 first_file_txg=50 three_k_minus_one=50 holds=true oldest_valid_root_before=Some(5)
```

- 三个 S 上 k = 2..=40 都是 3k − 1，k = 1 是 3，与注释逐字同；k = S + 1、S + 2 两处的 txg − 3S 是 2、5，与注释的「2 < 3」「5 ≥ 释放代 3」同。环绕了几圈（k 到 40）式子不变。
- 「4、16 编不过」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:63`）：量过，第二份副本把常量改成 4、16，`cargo build -p singlefs-harness` 都退 101。`out/h2-compile-s4.txt` 与 `out/h2-compile-s16.txt` 整份：

```
S=4 cargo_build_singlefs_harness_rc=101
      1 error: could not compile `singlefs-harness` (lib) due to 1 previous error
      1 error[E0080]: evaluation panicked: mkfs 写的 S 碰到了区间的边（D22 已定项 1 字段表：4..16 之间取值不碰任何一条边）
error[E0080]: evaluation panicked: mkfs 写的 S 碰到了区间的边（D22 已定项 1 字段表：4..16 之间取值不碰任何一条边）
S=16 cargo_build_singlefs_harness_rc=101
      1 error: could not compile `singlefs-harness` (lib) due to 1 previous error
      1 error[E0080]: evaluation panicked: mkfs 写的 S 碰到了区间的边（D22 已定项 1 字段表：4..16 之间取值不碰任何一条边）
error[E0080]: evaluation panicked: mkfs 写的 S 碰到了区间的边（D22 已定项 1 字段表：4..16 之间取值不碰任何一条边）
```

  报错的是 singlefs-harness 这个库（它求值 `AT_MAKE_FILESYSTEM`）；mkfs 函数本身收 `parameters.geometry.root_ring_slots_per_region`（`crates/singlefs-core/src/make_filesystem.rs:547`），调用方经 `from_system_configuration_field`（`crates/singlefs-core/src/root_ring.rs:62`：「    pub fn from_system_configuration_field(」，收 4..=16 闭区间）也造得出 S = 4、16 的参数交给 mkfs。注释说的是「`AT_MAKE_FILESYSTEM` 这条路径」，H2 的两个常数正是从这个格式常量算的，所以不算错；读码判的，没造 S = 4 的池。

**root_record.rs 卸载记号那句**（`crates/singlefs-core/src/root_record.rs:23`：「/// 记录的新根段照抄那次发布写的根上的记号——卸载那一串发布的记录带 1、施加出来是 [`UnmountMarker::WrittenByTheUnmountSequence`]，」）：读码，对。每次发布装记录都取这次写的根上的记号（`crates/singlefs-core/src/transaction.rs:8656`：「            new_unmount_marker: root.unmount_marker,」，在 `publish_admitted` 里；另两处装记录的 `prepare_the_publish_without_units`、`prepare_the_row_publish_on_a_version_without_file` 在 1399、1936 行同一句），恢复施加时取记录里的（`crates/singlefs-core/src/recovery.rs:3601`：「                unmount_marker: record.new_unmount_marker,」）；卸载那一串的抬 F 发布记号取 1（`crates/singlefs-core/src/mount.rs:2436`：「            RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,」）。没跑用例核施加出来的那一版。

**C670 模块文档**（`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:5`：「//! 在这一形上照常比（跳过撕裂的槽取 c_见证），用例那一份盘面上复算为假、不报（换一份盘面照常比可以为真）；拒照 U6 判——系统配置见证照 R2」）：量过，括号里那半句对。探针把同文件的用例原样拷进探针包，接一条：取「只落了覆盖写那次轮换」的盘面，再把盘 0 两个系统配置槽各改一个字节（与用例撕裂同一种改法），盘 0 一份自证过的都没有、盘 1 那一槽带着更新的 journal tail。`out/c670.txt` 整份：

```
test a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal ... ok
test a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal ... ok
test opus_both_slots_of_device_zero_torn_on_the_disk_where_only_a_rotation_landed ... c670-other-disk comparison=Some(AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 3 }) selected=Some((InstanceGeneration(1), CheckpointTxg(3))) verdict=Illegal { reason: NewerPublishWitnessedBySystemConfiguration, conditions_not_met: [WitnessCriterionTrueOnACrashImageWithoutReadFaults(WitnessedCounterAboveTheLastRecordOfTheSelectedVersion)] }
test the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

- 同一形（某块盘两槽都撕裂）上 C670 那一条照常比、为真（`WitnessCriterionTrueOnACrashImageWithoutReadFaults`），判定 `Illegal`，U6 那四条一条没缺：括号里「换一份盘面照常比可以为真」成立，「拒照 U6 判」那一半在这份盘上也照样只判出 C670 那一条。
- 顺带看到的：这份盘上实现报的比较是 `AgainstTheSelectedVersionsLastRecord`，不是用例那份盘上的 `SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`——读阶段的见证判据先为真先拒，「这块盘一份自证过的都没有」要到取号那一刻才查（`crates/singlefs-core/src/mount.rs:4635`：「                WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,」，在 `acquire_expected_instance` 的拒因映射里）。模块文档写的是判官那一侧的分支，不说实现这一侧报哪一支，不算错。

## 几个改法各修哪一格

| 改法 | K1 段号（M7d、M7h） | K1 段内位置（M7a、M7b、M7i） | K1 历史种子（M7f、M7g） | K2 B1-c | 别的格 |
|---|---|---|---|---|---|
| 入库 F3-d（A、B、C） | 不修：量过，两行三条全绿 | 不修：量过，三行三条全绿 | 不修：量过，两行三条全绿 | — | 登记的六行各红在点名那条（量过） |
| 攻方 F3-e 的 A2（段号 0..=63 加 2^k、2^k + 1，k = 6..=20） | 修：量过，两行都红（262080、286652 / 384930） | 不碰：量过，三行绿 | 不碰：量过，两行绿 | — | 量过：另抓登记的按位拼接、去段号、或代异或，与 M7e |
| 攻方 F3-e 的 B3（长 32 的段扣下 1 或 2 个写，528 个） | 不碰：量过 | 修：量过，三行都红（79、137、407 / 528） | 不碰：量过 | — | 量过：另抓登记的前 12、前 16 个写，与 M7c |
| 攻方 F3-e 的 C2（历史种子 0、1、2^63） | 不碰：量过 | 不碰：量过 | 修：量过，两行都红（8190 / 12285） | — | 量过：另抓登记的去历史种子 |
| 入库 B1-c | — | — | — | 已经修好：量过，20898 + 97 份盘上没有放过「建 0 个」的绿 | — |

F3-e 只在我的模型上量过、被攻过零轮；写成入库用例的样子、在 checker 档的挂钟是推的。这是最后一轮，照正文第五节它进交用户表或记账。「量过」都是副本上探针跑原样抽出的种子函数的数，主 agent 要在入库装置上重做。

## 没打中的形状

- F3 变异：登记的七行（F3-c 一行、F3-d 六行）与攻方九行，共 16 种种子算法；矩阵之外没试。M7c（只折前 28 个写）、M7e（段号截成 u8）三条放过、我量不出损。
- 真历史：战役四档（快档、偏回退、线程那条、大档 500 段 40 步）与一份稠密摆法（24 段 40 步），全是 4 GiB、`BROAD` 或 `ROLLBACK_AFTER_RAISING_THE_FLOOR` 比重、种子基起；没跑 384 槽、更多步数、别的种子基，段号与段长的上沿会跟着这些变。
- B1：入库那组 112..=208 逐槽，宽扫覆盖写 1、3、5 次 × 长度 100、2999、9000 × 批 1–3 × 单元 4–20 六档 × 112..=240 逐槽；没换设备宽度、journal 环大小、批大小（今天写死 1）。
- H2：S = 5、8、15，k = 1..=40；S = 6、7、9–14 没跑。
- C670：只造了一份「换一份盘面」（只落轮换 + 盘 0 两槽撕裂）；盘 1 两槽撕裂、两块盘各撕一槽这几形没造。
- root_record：读码，没跑用例核施加出来的那一版的记号。

## 这条腿自己的限度

- 全部数是副本上的数。F3-d、B1-c 是 checker 档用例，没跑；矩阵里 `F3c A B C` 四列是把入库用例的函数体原样抽进探针包跑的（`tools/gen_seedfn.py`，抽之前施加变异行），与在 checker 档里跑是同一段源码，但不是同一个测试二进制。
- 探针包 `opus-b24-probe` 链着 checker 档的库跑真历史与挂载（与 b21、b23 攻方同一种用法），没起 checker 档包的任何测试目标与 `src/bin/` 装置。主 agent 若判这种用法算 checker 档，K1 真历史与 K2 B1 的数降为推的。
- 没调层 0 枚举函数（`enumerate_layer0` 一次没调），所以没有用 `closed_form_state_count` 算状态数；真历史那一段只摆第一次崩溃、不恢复、不跑 checker，摆法与战役同一个 `draw_crash_points`，摆出的态一个不落地计进了数。B1 每份盘面只挂一次、发一次，不枚举崩溃态。
- 估时与分批：一段 40 步的历史单线程约 5.5 秒（先跑一段估的），大档 500 段分 3 批、每批 10 片并行、每片 17 段，每批约 55–70 秒；稠密 24 段 6 片一批约 15 秒；B1 宽扫 27 件分 3 批（10、10、7 件），每批约 33–55 秒。全量约 12 分钟，没缩取样。
- 线程：探针各件单线程（`capped.sh 1`），一批至多 10 件；cargo 经 `capped.sh 10`。与上限 10 相符。
- 内存包装：开头两次起跑退 252（排不上：slice 已占 36.2 GiB，别的会话一条 24G、一条 8G 在账上；另一次是同一会话已有一条日常的在跑、理由短于 8 个字），命令一行都没跑，不算结果；之后照峰值表（探针 55 MiB、冷编 1.4 GiB）把上限改成 256M、3G，带并行理由与等待秒数重跑。H2 那一段第一次写成每个 k 重做 mkfs、挂 k 次，`timeout 235` 截在跑 k = 40 之前（退 124，没有产物，进程随 timeout 一起停，`ps` 核过不剩），改成在一份盘面上接连挂、每次挂完拷一份写文件；k = 1..=6 两种写法逐个同。
- `ps` 开工时看到别的会话在跑 `cargo test`（harness 档一批、E142 变异），没有性能测量；我的编译走自己的 target 目录，没等锁。
- 每格只抽了一次（这条腿只跑一次），「没打中」那几格照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算没打中。

## 要主 agent 跑的命令

1. 复跑核对（只编副本、跑探针与 harness 档探针，不跑 checker 档包的测试目标）：仓根执行，`WORK` 指一个空的草稿目录，内存上限与线程由脚本内部给（探针 256M、编译与 harness 档 cargo test 3G、cargo 线程 10、探针各件单线程、一批至多 10 件）。每段前台跑得完，按次序：

```
export WORK=<主 agent 的草稿目录>/b24-opus-rerun
F=research/prompts/m3-formal-code-b24-r1-opus-model/tools/rerun.sh
bash $F setup; bash $F build                       # 约 25 秒（本机 32 核）
bash $F real-small; bash $F real-large-batch 1; bash $F real-large-batch 2; bash $F real-large-batch 3
bash $F real-dense-batch; bash $F real-collect     # 合计约 3.5 分钟
bash $F f3-begin; bash $F f3-rows; bash $F f3-end  # 约 1.5 分钟
bash $F b1-slots; bash $F b1-batch 1; bash $F b1-batch 2; bash $F b1-batch 3; bash $F b1-collect   # 约 1.5 分钟
bash $F harness; bash $F h2s-setup; bash $F h2s 5; bash $F h2s 15; bash $F h2s-compile 4; bash $F h2s-compile 16   # 约 3 分钟
for f in research/prompts/m3-formal-code-b24-r1-opus-model/out/*.txt; do cmp "$f" "$WORK/out/$(basename "$f")" && echo "same $(basename "$f")"; done
```

   可覆盖的环境变量：`REPO_ROOT`、`FROZEN_TREE`（默认 `/tmp/claude-1000/m3-formal-code-b24-r1/tree`）、`WORK`、`THREADS`（默认 10）、`BUILD_CAP`（默认 3G）、`RUN_CAP`（默认 256M）、`RUN_WITH_MEMORY_CAP_WAIT_SECONDS`（默认 200）。`bash $F all` 依次全跑（约 12 分钟，超过前台时限要放后台）。我在 `WORK=/tmp/claude-1000/m3-formal-code-b24-r1-opus/rerun-check` 上照这个次序从 setup 跑到 h2s-compile，16 份产物与模型目录 `out/` 逐字节相同（c670 那一份的 `finished in` 计时是在那一遍之后从脚本里去掉、两边一起重抽的）。看哪几行：`out/f3.txt` 用 `python3 tools/summarize_f3.py` 汇总与 K1 那段逐行比；`out/b1-summary.txt` 的计数行；`out/h2-txg*.txt` 的 `holds=false` 只有 k = 1；`out/c670.txt` 的 `c670-other-disk` 一行。
2. 若采纳 F3-e：写成入库用例与变异行之后，在提交时跑 checker 档（不归我跑）；入库变异表可以照搬 `tools/opus-mutations.tsv` 的 M7a、M7d、M7f 三行（各点名 B3、A2、C2）。

## 没做什么

- 没判正推腿那几格，没读禁读清单里的文件，没读实现员草稿。
- 没跑任何 checker 档包的测试目标、层 0、变异表；24-1 那七行变异在入库装置上的红绿没量（副本上用原样抽出的用例与同义的环境变量开关量）。
- 没把改法写进仓里任何 crates 文件；冻结副本一个字没动（编译都在拷出来的副本里）。
- 没量 F3-e 在 checker 档的挂钟；没量 M7c、M7e 在更长的段、更大的段号上的损。
- 前一条腿留下的 `tools/`、`probe/` 我核过之后接着用、改了几处（`rerun.sh` 重写、`main.rs` 的 harm 分组与 `f3_proposed`、`h2_txg.rs` 改成接连挂载、`opus-mutations.tsv` 加 M7h、M7i，新写 `tools/summarize_f3.py`）；它在草稿目录留下的 `build1–3.log`、`seedfn-test.rs`、`parked/`（挪开的 E171R1 半成品的拷贝）没动。

## 草稿与副本

- 两份仓副本与编译目录（`/tmp/claude-1000/m3-formal-code-b24-r1-opus/copy/`、`copy-h2s/`，复跑核对那一遍的 `rerun-check/copy/`、`rerun-check/copy-h2s/`）交回前删了。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b24-r1-opus/` 留着日志与小产物：`out/`（探索那一遍的全部产物、各批 `.out` / `.rc`、cargo 日志、`stderr.log`）、`rerun-check/out/`（复跑核对那一遍）。入库的是模型目录 `out/`。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b24-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b24-r1/tree --root . --background research/prompts/_m3-formal-code-b24-r1-background.md` 末行：

```
  ✓ 核了 25 处引文，对上 25 处，没判 0 处（逐处列在上面）
```
