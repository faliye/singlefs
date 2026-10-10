# m3-formal-c655-c687-r1 云端攻方（Opus）报告（2026-10-05）

<!-- doc-lint:not-numbers Y1 Y2 Y3 R1 R2 U V F -->

攻击面：Y1、Y2、Y3（正文第三节），重点 Y1 与 Y3。代码引文全部读冻结副本 `/tmp/claude-1000/m3-formal-c655-c687-r1/tree/`，路径写那棵树里的相对路径。量的数都在我自己拷出的副本（`/tmp/claude-1000/m3-formal-c655-c687-r1-opus/tree/`，冻结副本 rsync 一份再加探针）上跑出，是副本上的数；checker 档的测试一个没跑，GPU 一格没跑。

## 复跑命令与文件

模型目录 `research/prompts/m3-formal-c655-c687-r1-opus-model/`。复跑（仓根执行；`FROZEN` 是冻结副本 tree、`WORK` 是复跑的草稿目录、`REPO` 是仓根，都可覆盖；整套十几分钟，视负载）：

```
REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-c655-c687-r1/tree WORK=<草稿目录> THREADS=10 bash research/prompts/m3-formal-c655-c687-r1-opus-model/tools/rerun.sh [harness y1-4g y1-384-check y1-384 y1s2 fallback]
```

它在冻结副本的拷贝上打 `probe-core.patch`、把 `harness-probe-tests.rs` 接到 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 末尾、加 crate `c655probe`（`Cargo.toml`、`main.rs`），经 `research/scripts/run-with-memory-cap.sh` 编、逐件跑，最后把每份日志与模型目录里的同名日志逐行比（去掉耗时、排队、编译行与取样的例子行），三份汇总重出再比。checker 档的测试一个不跑。我自己照它跑过 `fallback`、`harness`、`y1-384-check`、`y1-384` 四件，26 份日志逐行相同；`y1-4g`、`y1s2` 两件没经它复跑（`y1s2` 单件约 5 分钟，超过我前台一条命令的时限）。

每个文件的 sha256：

```
ae2d3f7cf28f3db9fd233126969a849c99b5a9eb3dcc08c1b63a0e9918d3d343  Cargo.toml
6658de0225b3f169c55a3aa0dc0c2705b111b67da4b1eceb50d4ad788e49764e  main.rs
e12baee6209a34b3bf2c249985850e5def65e48f19222cb4ad337bae7bb482a1  probe-core.patch
130fc4a4acb59c3db5ad53ff66946191bd28a842ca252ff17f33f4da6f90bd9b  harness-probe-tests.rs
415b66c17218378283de380af2697f946bec426b7f80c5bdb19fe73ad14908b9  tools/rerun.sh
6866879823ec705282c61e72b38b875f35066134176ace5311af77a49da3d9ad  tools/summarize.py
1511ee1565db0010e6977804e5725be85ccf7016943962a718d71b2c43d0375b  fallback-240-a.log
965b4f673c83edf1a8f252ece90a03968c321876afbc84934ae346c938e38e37  fallback-240-b.log
a1631ddb9d3ec496abd2624934c81eed61c5caac27d66ece0ed8404902ac9297  fallback-256-b.log
e6d9a555c44ab01477a64a24482a3685f9f9b1cbeaa0cc99b3b0b4a7d5c417d2  fallback-384-b.log
2c3a0d168d5af567f2e31f341fe5ac47798049c236a9c0920085311364569182  fallback-control.log
f3abe8f2e7ee86bae85003d221e430fdc6a91c6498dfff817b18a30f9d4054d4  y1-root-fields.log
ccc49f24ae14a3f6aae70d0a0e619c9b6d7e23eb3a736af551c4c52671257808  y1-root-fields-not-newest.log
890da9ecffa1d6940f7744bcf43dee293b5b53d31b8e09d3bcb27e346132e1ad  y1-s1.log
f8ad4d01914d088bfb82c284927fb9ac94a5666b18ca77d230eaf6d10347b5d7  y1s2w384-fr-last.log
44f22e356a3f223823610d4e6a63f626157ee822bafd7abf42edb8a3265e0946  y1s2w384-rb@1-4-last.log
c3e0d795e7d74fcb388b94b8ee9bdd3e0dc250f3855d59ec244b039bc4dcc655  y1s2w384-s1-0,last.log
fd9dc60e43e16d89f8eda74ab1fe5e861598a78f6555505a31eeb3747844a03e  y1s2w384-s2-0.log
4b7f495138a967866bbc94e7867819f0df6f17910d9836e1550fd384395bcc45  y1s2w384-ul@1-4-last.log
7a5ca4e8fb0a471d1d8c22e9a375a285a4283ef87943958c36fe0eb5c72bc8b9  y1-sc-fields.log
13fc770aedcc84f1ef64a67fec33d21848abb22d46fa8985376e9ccea0393310  y1-ul@1-4.log
d84b121d5baf2fc378b2d53dab33413ef448821b1e0a5e1a17678f52b56e41c2  y1-ul@5-8.log
2f899c4a787457fada822e435099b2facc11c6299f6dc456299d736eb10440f1  y1w384-ci@1-4.log
44c84ad334dad546915f888b61753066c36204858edadae61f33f41315ebe2ed  y1w384-fr@0-1.log
a3f231cf5e01bab88d0c4576f5c1c7a501f0ecfb2877abb8fee7bc0120f1e4d8  y1w384-fr@2-5.log
52f8c66eaad396b4dc70981eb308283de1d8f02b6d984f77c3eebbe7e95f82be  y1w384-fr@6-9.log
af4f3975653c69c2799ec6898c1c7e475cf6ae90253403f2b6d76fb04f204dc2  y1w384-fr.log
53e9b65cfedbd64cd2bcccfd1608e5bf71fcfeff147767af2c8eaec28226fab7  y1w384-ow@1-4.log
01915f85876507a4defae99c693921c563eb1f4fa6aa4f829cbbe3769bf05ddf  y1w384-rb@1-4.log
196ed256cc2bb812e4710658bd62e2fe1373d9940a62f8eabc2d40047946ea78  y1w384-rbow@1-4.log
347521cc6106207104593d2d95e7543bd66c565a61fed5f039f4135ad92a8928  y1w384-s1.log
eb2c923c52282f5ae4292f0760748abc5740f0a1cccf634822ee3e4f641a39c2  y1w384-s2@0-1.log
5bf084dc8a027b7df4bf059410f615f0c10aa8bed4c51ca4f56a37ba3bc3a521  y1w384-s2@2-5.log
7302d59a56225f81e0207a11c6105f7ebddc03dc45ee95b3e5f2dbd96f361fb1  y1w384-s2@6-9.log
0e66e148ca5080ec280312c332777081ea43818d8f89ac93c2bc8767f6dc7816  y1w384-s2.log
a13f4f0431bbbfe046e3974694f724fd2755a4f095f43831054a7066e8fdee9a  y1w384-ul@1-4.log
e29c58e157425b0ff6b16c6f625615ee072aaeaac5f0c9c5cf0abffe1fde5bb0  y1w384-ul@5-8.log
17277b0638d10f95297f3ebdbe36766e8de6293f65acacf8a6493397460cb72b  y3-cells-e45.log
9d6cd559ae8b6ee71c912f769aff993a471b694f53338ed8ef939addff4b223f  y3-cells-f.log
d64195e1ffe7e936d1b20a0591872215605fb41bad4a2b9144b509e6bae2d672  y3-cells.log
24f428868d6c6a562f247a31e64dc290906bc590076d4d5df2a35eaaf1bd7367  summary-y1-first-crash.txt
c5b4e177e0cffdc446c9a5ac596930883ab8e4d5fcc88f381eeac28c66f3a952  summary-y1-second-crash.txt
972e389da954a89276b5e3e6f65034cc339e1412fd93c91fd8bc7ee7d88decab  summary-y2-other-tree-roots.txt
```

## 各格判定一览

| 格 | 判定 | 依据（细节在下面各节） |
|---|---|---|
| Y1 `mapping_root`、`allocation_record_tree_root` | 第三轮辩方推的「并进分配记录那一行」**没打中**；「必须相同」在合法态上大量红 | 量过：带树那几版每态 `mapping_root` 都变，mkfs 之后第一次挂载 33 态里 29 态分配记录树根由空变非零；两样写坏（6 份）池级 checker 都红（Y1-2、Y1-4） |
| Y1 deleted_inodes 根 | **打中（H3）**：「必须相同」在 unlink 走到一半崩溃的合法历史上红 | 量过：第一次崩溃 12 + 1 态、二次崩溃 12 态；改法「照两棵用户可见树那一行，按续做各步判」零轮 |
| Y1 墓碑根、谱系根 | 「必须相同」**没打中**（取样窄） | 量过：一批走空的 unlink、回退两段历史上都不变；多批走空没量 |
| Y1 算法、nonce、MAC | **打中（H4）**：写坏了 core、池级 checker、挂载透明都看不见；「必须相同」在 `RootRecord` 上没法比、由记录施加出来的那一版没有根槽 | 量过：m4–m6 三份 `violated=[]`、读回文件相同；改法「池级 checker 判根槽偏移 428..457 全 0，挂载透明不比」零轮 |
| Y1 系统配置槽世代号 | 模掉，交 I-7.16 | 量过：增量 = ⌈落了的系统配置槽写数 ÷ 2⌉（181 956 次比较全对上）；写坏 I-7.16 红 |
| Y1 系统配置 journal_tail | **打中（H6）**：模掉则多写的 tail 让下一次可写挂载被拒、checker 不红 | 量过：q2；关系「R1 的 ≤ R2 的 ≤ R2 环里最大计数器，挂载做完取等」在全部合法态上成立；零轮 |
| Y1 系统配置 journal 实例代号（题面漏了） | **打中（H6）**：表里没有它，写坏了谁都不红 | 量过：q3；关系「挂载做完 = R2 根的实例代号，二次崩溃 ∈ {R1 的, 新实例}」；零轮 |
| Y1 二次崩溃「落回 v1 的各字段与 R1 相同」 | 系统配置几样不是「那一版自己的字段」，读成是会红 | 量过：v1 态 65 859 个里 65 823 个系统配置实例代号已是新实例 |
| Y2 表外、题面外的字段 | 根记录的实例表指针、树表里种类 3、4、6、7、8、12 的根与诞生 txg、头 ID，系统配置里不进内存的二十几样 | 读码；树表那几样量过（种类 3、4 每个 mount_version 态都变，其余不变） |
| Y2 对账判官的清单从哪取 | 不从结构体取（H4 那三样就不在结构体里）；按字节区间盖满 `ROOT_RECORD_BYTES` 与 `SYSTEM_CONFIGURATION_BYTES` | 推的，零轮 |
| Y3 树表读不出、条目宽不到 200、extent 树读不全、分配记录树读不全、某块盘上一个都没有 | 不适用：坏镜像上别的不变量都红（重复），合法镜像上没出现 | 量过：坏镜像 a1、a2、c1、e3、d1、d2；合法态 28 974 + 152 019 个里 0 次 |
| Y3 种类 1 不止一条、种类 3 不是恰好一条 | **打中（Y3-1-H1、H2）**：判「不适用」则 b2、b3、e2、e4 整份 checker 不红；判 I-5.5 红也漏 f2（不是这一类的版本） | 量过；改法「I-9.16 加一道：有根的种每种恰好一条，别的至多一条」零轮 |
| Y3 开放段回落误排 | **没打中**：今天的准入下够不着 | 量过：294 段历史回落 0 次（计数器阳性对照 22 次）；一次抽样 |
| Y3 CPU / GPU 改法 | CPU 七处说明串改成不记、I-9.16 那一道加在 `judge_tree_table_entries_ordering`；GPU 今天一格都不置位，不用改 | 读码推的，没改没跑 |

## Y3　C687：I-5.5 判不了的每一格

### Y3-1　坏镜像上每一格旁边还有谁红（副本上量过）

装置：我副本里 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 末尾加的两个用例 `c687_probe_cells_on_bad_images`、`c687_probe_duplicate_kinds_on_a_version_not_built_from_the_mkfs_watermark`（内存稀疏盘，与 `build_pool` 同一串 mkfs、取号、暖机、新池新建文件，第二个再接一次覆盖写），每份坏镜像跑一遍 `check_pool_image`，列 I-5.5 的判定、判红的不变量、比干净镜像多出来的「不适用」、core 的 `recover` 结局。日志 `y3-cells.log`、`y3-cells-f.log`（模型目录）。干净镜像两份都是 `i55=Some(Holds) violated=[]`。

| 坏镜像（改的是 A 那一版，txg 3，从 mkfs 水位建树） | I-5.5 判不了的哪一格 | 别的不变量 | core |
|---|---|---|---|
| a1 树表单元两盘清零 | 树表读不出 | I-2.1、I-4.8、I-7.2、I-7.4 红 | 读不出 |
| a2 树表条目宽改成 key 宽（`narrow_the_entry_width_to_the_key_width`） | 条目宽不到 200 | I-4.8、I-7.2、I-7.4 红（I-1.10 反而转「不适用」） | 拒 |
| b1 livelist 那条的种类改成 1 | 种类 1 不止一条 | 只有 I-9.27 红（头 ID 0 不合 extent 那一格） | 拒「树表里同一种树有两条」 |
| b2 同上，头 ID 也改成 65537 | 种类 1 不止一条 | **一条都不红** | 拒 |
| b3 logged_ops 那条改成种类 1、头 ID 65537 | 种类 1 不止一条 | **一条都不红** | 拒 |
| c1 extent 树根两盘清零 | extent 树读不全 | I-2.1、I-4.8、I-7.2、I-7.4 红 | 读不出 |
| d1 数据指针第二条位置条目的盘 1 改成盘 0 | 某块盘上一个都没有 | I-2.5 红 | 读得出文件 |
| d2 同上改成盘 7（池里没有） | 某块盘上一个都没有 | I-2.1、I-4.8、I-7.4 红 | 读得出文件 |
| e1 分配记录那条的种类改成 7 | 种类 3 不是恰好一条（0 条） | I-9.16 红 | 拒 |
| e2 deadlist 那条的种类改成 3 | 种类 3 不是恰好一条（2 条） | **一条都不红** | 拒 |
| e3 分配记录树根两盘清零 | 分配记录树读不全 | I-2.1、I-4.8、I-7.2、I-7.4 红 | 读不出 |
| f1（覆盖写之后 B 那一版，txg 4，不是从 mkfs 水位建树）livelist 改种类 1、头 ID 65537 | 种类 1 不止一条（这一格在认法 ① ② 之前判，B 也进得来） | **一条都不红** | 拒 |
| f2（同上 B）deadlist 改种类 3 | 不进这一格（B 不是这一类，I-5.5 判 A 成立） | **一条都不红**，I-5.5 `Holds` | 拒 |

原样输出（摘四行，全文在日志里）：

```
name=c687-cell image=b2-livelist-entry-kind-to-extent-head-65537 i55=Some(NotApplicable("有一版的树表里种类 1（extent 树）条目不止一条：认法 ① 读哪一条，I-5.5 那一行没写")) violated=[] partially_judged=[] turned_not_applicable=["I-5.5"] core_recover=failed:UnitMalformed { what: "树表里同一种树有两条" }
name=c687-cell image=e2-deadlist-entry-kind-to-allocation i55=Some(NotApplicable("从 mkfs 水位第一次建树的那一版树表里种类 3（分配记录树）条目不是恰好一条：落点按哪一棵分配记录树算，I-5.5 那一行没写")) violated=[] partially_judged=[] turned_not_applicable=["I-5.5"] core_recover=failed:UnitMalformed { what: "树表里同一种树有两条" }
name=c687-cell image=f2-after-overwrite-deadlist-kind-to-allocation i55=Some(Holds) violated=[] partially_judged=[] turned_not_applicable=[] core_recover=failed:UnitMalformed { what: "树表里同一种树有两条" }
name=c687-cell image=d1-data-pointer-second-location-device-1-to-0 i55=Some(NotApplicable("从 mkfs 水位第一次建树的那一版有数据单元、而池里有一块盘上一个都没有：那块盘上判不判，I-5.5 那一行没写")) violated=["I-2.5"] partially_judged=[] turned_not_applicable=["I-5.5"] core_recover=file
```

**打中（Y3-1-H1）：「同一种树在树表里两条」这一类坏镜像，池级 checker 今天整份不红。** b2、b3、e2、f1、f2 五份上 `violated=[]`，core 却拒读。I-9.16 的 checker 实现明写不判它：`crates/singlefs-checker/src/walk.rs:3498`：「同一种有两条时取先出现的那条（核心层把它判成损坏，这里不另立判定）。」core 那一侧的判定在 `crates/singlefs-core/src/recovery.rs:2296`：「pub const TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE: &str = "树表里同一种树有两条";」。

四句：
1. 分不分辨臂：分辨。C687 那两格（种类 1 不止一条、种类 3 不是恰好一条）若定「不适用」，b2、b3、e2 三份上整个 checker 不红；若定「判红」，这三份上只有 I-5.5 红。
2. 被判系统看不看得到：看得到，树表单元一读就数得出种类。
3. 满足判据的哪一分句：正文 Y3「只在坏镜像上出现的，判红是否与别的不变量重复」——这几份上**不重复**：没有一条别的不变量红。
4. 两个改法在打中的格上还中不中：「I-5.5 判红」只罩得住认出来是从 mkfs 水位建树的那一版（种类 3 两条那一格在 `walk.rs:4288` 认法之后才判），f2（B 那一版上种类 3 两条）照样整份不红，I-5.5 自己判 `Holds`；f1 罩得住（种类 1 那一格在认法之前判，任何一版都进得来，这本身就是 I-5.5 越出自己射程在判一件与落点无关的事）。「不适用」五份全漏。**两个改法都对 f2 不起作用**。

我提的改法（只在我的模型上量过、被攻过零轮）：「树表里每种树至多一条」另立成 I-9.16 的第三道（或新号），按每一版的树表判（与 core 的 `TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE` 同一个谓词，checker 自己另写）；I-5.5 那两格写「不适用：种类不唯一的树表归 I-9.16 ③ 判红」。判别力自证：b2、e2、f2 三份各由「不红」转「只红 I-9.16」。推的：没在副本上改 checker 跑。

补两份（`y3-cells-e45.log`）：树表里**整条去掉**分配记录那一条（e4）与 extent 那一条（e5）。

```
name=c687-cell image=e4-allocation-entry-removed i55=Some(NotApplicable("从 mkfs 水位第一次建树的那一版树表里种类 3（分配记录树）条目不是恰好一条：落点按哪一棵分配记录树算，I-5.5 那一行没写")) violated=[] partially_judged=[] turned_not_applicable=["I-3.9", "I-3.10", "I-5.4", "I-5.5"] core_recover=failed:UnitMalformed { what: "树表里缺一棵有根的树" }
name=c687-cell image=e5-extent-entry-removed i55=Some(NotApplicable("这个镜像上没有它判的对象")) violated=["I-1.1"] partially_judged=[] turned_not_applicable=["I-9.10", "I-5.5"] core_recover=failed:UnitMalformed { what: "树表里缺一棵有根的树" }
```

**打中（Y3-1-H2）：树表缺分配记录树那一条，池级 checker 整份不红**（e4，`violated=[]`），core 拒「树表里缺一棵有根的树」（`crates/singlefs-core/src/recovery.rs:4929`：「what: "树表里缺一棵有根的树",」）。四句同 Y3-1-H1：分辨臂（「种类 3 不是恰好一条」定不适用则整份不红）；看得到；正文 Y3「判红是否与别的不变量重复」那一分句——不重复；「I-5.5 判红」只罩认出来的那一版，缺的若是覆盖写之后那一版的分配记录条目照样漏（推的，没造这一份）。改法同 H1：I-9.16 第三道写成「有根的那几种树每种恰好一条，别的种至多一条」，与 core 的两处拒（`TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE` 与「缺一棵有根的树」）同一个谓词、checker 另写；零轮，推的。

### Y3-2　这几格在合法写侧写出的镜像上出不出现（副本上量过）

装置：`c655probe`（模型目录 `main.rs`），每个崩溃态都调 `judge_crash_image`（池级 checker 全跑），I-5.5 的判定按类计数。历史全是手写的，崩溃态用 `judge_layer0_state_range` + `full_expansion` 逐个枚举（层 0 同一枚举域，原地覆写多一态）；二次崩溃把挂载那一段写流当窗口再全量枚举一遍。盘宽 384 槽小盘（`C655_WIDTH=384`）与 4 GiB 两种几何。

| 来源 | 状态数 | I-5.5 判定 |
|---|---|---|
| 第一次崩溃态（`y1w384-*.log`、`y1-*.log` 里 s1、s2、fr、ow、ul、ci、rb、rbow 各窗口，去掉重复的取样跑） | 28 974（窗口有重叠，按跑过的计） | `holds` 28 908、`not_applicable:这个镜像上没有它判的对象` 66，「判不了」那几格 0 |
| 二次崩溃态（`y1s2w384-*.log`：ul 队列不空那一态、s1 两态、rb、fr、s2 各一态上的挂载写流） | 152 019 | `holds` 151 698、`not_applicable:这个镜像上没有它判的对象` 321，「判不了」0 |
| 提交内生块回落扫描（`fallback-*.log`：mkfs 之后 N 次可写挂载、每次后接 K 次零单元发布、再写第一个文件，N 取 1–64、K 取 0–16，三种小盘 240 / 256 / 384 槽，准入照式子判） | 294 段历史 | 每段末尾镜像 `i55=holds`；回落次数 0（第一个文件那一次 0，全程 0） |

回落计数器是我在副本 `allocator.rs` 回落那一支里加的进程内计数（`PROBE_COMMIT_GENERATED_FALLBACKS`）；它会动：照 harness 用例 `commit_generated_units_fall_back_when_the_segment_runs_out` 的造法（占满开放段、每个全空段占一槽）之后一次覆盖写，`fallback-control.log`：「name=fallback-control publish_ok=true fallbacks_before=0 fallbacks_after=22」。

结论（取样之内）：
- 「判不了」那七格在我跑的合法历史上一次都没出现：它们只在坏镜像上出现。这与正文 Y3「会出现就不能判红」那一条不冲突。
- 开放段回落误排：b19 判决 K1-a 推的「今天的准入下够不着」，我把挂载次数从 b19 攻方的 1–12 放到 1–64、加上每次挂载之后的零单元发布、换三种小盘，回落 0 次。**没打中**，一次抽样。没造的：被抛弃根的隔离位、读故障、抬 F 扣住位压在开放段上的那几种（见「没打中的形状」）。

### Y3-3　每一格该判红还是不适用（按 Y3-1、Y3-2 的量）

| 格 | 合法镜像上出不出现 | 坏镜像上别的不变量 | 我的判定 |
|---|---|---|---|
| 树表读不出 | 没出现 | I-2.1、I-4.8、I-7.2、I-7.4（a1） | 不适用（重复） |
| 树表条目宽不到 200 | 没出现 | I-4.8、I-7.2、I-7.4（a2；I-1.10 自己转不适用） | 不适用（重复） |
| 种类 1 不止一条 | 没出现 | **没有**（b2、b3、f1） | 归 I-9.16 新一道判红；I-5.5 不适用 |
| extent 树读不全 | 没出现 | I-2.1、I-4.8、I-7.2、I-7.4（c1） | 不适用（重复） |
| 有数据单元而某块盘上一个都没有 | 没出现；mkfs 断言两块盘（`crates/singlefs-core/src/make_filesystem.rs:477`：「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」），指针恒两条位置条目 | I-2.5（d1）；I-2.1、I-4.8、I-7.4（d2） | 不适用（重复） |
| 种类 3 不是恰好一条 | 没出现 | 0 条且是「改种类」造的：I-9.16（e1）；0 条且整条去掉：**没有**（e4）；2 条：**没有**（e2、f2） | 归 I-9.16 新一道判红；I-5.5 不适用 |
| 分配记录树读不全 | 没出现 | I-2.1、I-4.8、I-7.2、I-7.4（e3） | 不适用（重复） |
| 开放段回落误排 | 回落 0 次 | — | 维持「推的，今天够不着」；准入式、段长、盘宽变了重扫（本报告的 `fallback` 模式可复用） |

CPU 与 GPU 各改哪里（推的，没改、没跑）：
- CPU `crates/singlefs-checker/src/walk.rs` 七处 `part_of_the_range_not_judged` 改成不记（读不出、宽不到、读不全、某盘没有那五处，第 4214、4228、4265、4302、4328 行那几条说明串所在的调用），种类计数那两处（第 4243、4291 行）同样改成不记、由 I-9.16 新一道判；I-9.16 那一道加在 `judge_tree_table_entries_ordering`（`walk.rs:3498` 那一句注释要改）。
- GPU `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1921`、`:1943`、`:1957`、`:1967`、`:1971`、`:1975` 今天本就一格都不置位，与「不适用」一致，不用改；I-9.16 新一道要在 GPU 判 I-9.16 的那一处另加（我没找那一处的行号）。CPU 与 GPU 今天在「某块盘上一个都没有」那一格上本就不一致：CPU 记判不了，GPU 跳过那块盘接着判别的盘（`:1975`：「if (r_has_data_unit_on_device[device_index] == 0u) {」），判不适用之后两边一致。

## Y1　C655：表外字段各归哪一类

### Y1-1　量法

`c655probe` 的 `y1` / `y1s2` 两个模式（模型目录 `main.rs`）。每个第一次崩溃态：只读恢复得 R1；在同一份镜像上调 `mount_and_publish_once_on_the_crash_image`（崩溃注入同一个入口：挂载、续做、末尾清扫，再发一次布），挂载那一段写流全落在崩溃镜像上得 v2a、只读恢复得 R2a；挂载加发布之后的池只读恢复、取发布之前那一版得 R2b；二次崩溃把挂载那一段写流全量枚举，每个二次崩溃态只读恢复得 R2，按「R2 落到的那一版是不是 R1 那一版」分 v1 与 mount_version 两类。每个态上再调一次真判官 `judge_mount_transparency` 当对照：全部 `judged_all_held_or_not_judged`（第一次崩溃 28 974 态两个动作、二次崩溃 152 019 态），说明这些历史上今天的十样都成立，表外那几样的差异不是装置把合法态造坏了。

表外字段的读法：根记录三样直接取 `RootRecord`；算法、nonce、MAC 读那一版根槽的原始字节偏移 428..457（那一版由记录施加出来、根环里没有它时记 `not_in_ring`）；三根指针读那一版树表里种类 9、10、11 的条目（副本 `recovery.rs` 里加的 `probe_tree_table_entries_of_root`）；系统配置读每块盘两槽里自证过的那几份。汇总在 `research/prompts/m3-formal-c655-c687-r1-opus-model/summary-y1-first-crash.txt`、`summary-y1-second-crash.txt`（`tools/summarize.py` 从日志出）。

### Y1-2　各样在三种动作下的关系（量过的格写数，没量的写推的）

| 字段 | 只挂载 / 挂载加一次发布 | 二次崩溃 | 我的归类 |
|---|---|---|---|
| `mapping_root` | 树表 0 条的那一版上相同（零单元发布照抄，`crates/singlefs-core/src/transaction.rs:1375`：「mapping_root: previous_root.mapping_root,」）；带树的那一版上**每一态都不同**（发布现写，`crates/singlefs-core/src/transaction.rs:8606`：「mapping_root: mapping_pointer,」） | v1 相同；mount_version 带树的每一态都不同 | 模掉，并进「分配记录」那一行（池级 checker 判）；坏写法 checker 抓得到，见 Y1-4 |
| `allocation_record_tree_root` | 带文件的一版恒全零、相同；**mkfs 之后第一次挂载**（树表 0 条、写过行）由空变非零：s1 第一次崩溃 33 态里 29 态不同 | v1 相同；s1 那两态的 mount_version 42 态里 21 态不同 | 同上，并进「分配记录」那一行 |
| 算法、nonce、MAC | 量到的每一态两边都全零；R1 或 R2 是由记录施加出来的那一版时根槽不存在、比不了（`not_in_ring`） | 同左 | **不进挂载透明**，另立单镜像判据（见 Y1-3 H4） |
| deleted_inodes 根 | R1 那一版队列不空时**不同**（非零变全零，续做走空）；队列空时相同 | v1 相同；mount_version 里续做那一步落了的不同、没落的相同 | 按确定关系判：照「两棵用户可见树的根指针」那一行，取挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步（见 Y1-3 H3） |
| 墓碑根 | 量到的都相同（含队列不空、续做走空的那几态：墓碑在意图那次写，最后一批不改它） | 相同 | 必须相同（只在我量的 unlink 一批走空那一形上成立；多批走空、文件多于 144 个单元的没量） |
| 谱系根 | 相同（rb、rbow 两段历史上 R1 带谱系记录，挂载不改） | 相同 | 必须相同 |
| 系统配置槽世代号 | 不同：R2 最大值 − R1 最大值 = ⌈挂载写流里落了的系统配置槽写数 ÷ 2⌉，第一次崩溃与二次崩溃合计 181 956 次比较全对上 | **落回 v1 也不同**（取号那一写落了就 +1） | 模掉，交 I-7.16（系统配置槽世代号随轮换）在 R2 的镜像上判；不当「那一版自己的字段」 |
| journal_tail | 不同：挂载做完之后 R2 择到的 tail = 环里自证记录的最大计数器（第一次崩溃全部态） | R1 的 tail ≤ R2 的 tail ≤ R2 环里最大计数器（差 −1 的是最后那次轮换没落） | 按确定关系判：上面那两个不等式（见 Y1-3 H6） |
| 系统配置的 journal 实例代号（**题面清单里没有**） | 不同：R2 = 新实例 = R2 那一版根的实例代号 | v1 时多数也已是新实例（取号那一写落了） | 按确定关系判：挂载做完 = R2 根的实例代号；二次崩溃 ∈ {R1 的, 新实例}（见 Y1-3 H6） |

二次崩溃 v1 那一类里系统配置几样改了的计数（`summary-y1-second-crash.txt` 由 `y1s2w384-*.log` 合计）：v1 态 65 859 个（`queue_empty` 49 185、`queue_nonempty` 16 395、`no_entries` 279），其中系统配置实例代号已是新实例的 65 823 个（49 167 + 16 389 + 267）。题面那句「落回 v1 那一版自己的各字段与 R1 相同」若把槽世代号、journal 实例代号读成「那一版自己的」，这 65 823 态全红。

### Y1-3　打中

**H3　「deleted_inodes 根必须相同」（第三轮辩方推的归法，见 C655 那一行「前置」格）在合法历史上判红。** 历史：起点「第一个文件之后」，可写会话里 unlink 第一个文件（意图那次发布 + 一批走空），崩溃落在意图那次的根落了、走空那一批的记录没落全的那几态（`ul@1-4` 第 0–3 段之后、`ul@5-8`）。R1 那一版 deleted_inodes 不空；可写挂载照 D8 已定项 16 第 6 条续做走空（`crates/singlefs-core/src/mount.rs:4967`：「if current_version_of_trees.deleted_inodes.is_empty() {」不成立就走空），R2 那一版的 deleted_inodes 根是全零。量到的：

- `research/prompts/m3-formal-c655-c687-r1-opus-model/summary-y1-first-crash.txt:635`：「FIELD mount mount_version queue_nonempty tt.deleted_inodes.root = diff (r1 nonzero -> r2 zero) count=12」（384 槽小盘，`ul@1-4` 13 态里 12 态；4 GiB 那一份在第 66 行，同样 12 态）
- `research/prompts/m3-formal-c655-c687-r1-opus-model/summary-y1-second-crash.txt:209`：「FIELD second_crash mount_version queue_nonempty tt.deleted_inodes.root = diff (r1 nonzero -> r2 zero) count=12」（同一个崩溃态上挂载那一段写流的 90 165 个二次崩溃态里，续做那一步落了的 12 态）
- 同一批态上真判官全绿（用户可见两根指针按续做那几步判，今天的表在这里站得住）。

四句：① 分辨：分辨「必须相同」与「按续做那几步判」两条臂（前者红 12 + 12 + 1 + 1 态，后者按定义不红）；② 看得到：续做那几步的 txg 与树表都在挂载交回值与 R2 的镜像上；③ 满足的是正文 Y1「一段合法历史让某个字段的拟定归类把合法状态判坏」；④ 改法：照 D13 已定项 1 那张表「树表里 inode 树、extent 树的根指针」那一行（`.claude/kb/decisions/13-验证路线.md:49`：「相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的两根指针」）把 deleted_inodes 根并进去；实现上 `UserVisibleTreeRootPointersOfTheContinuation`（`crates/singlefs-harness/src/mount_transparency.rs:532`）每一步多带一个 deleted_inodes 根，比法照 `mount_transparency.rs:1205` 那一支。只在我的模型上量过「必须相同」会红，改法本身没实现、没跑，被攻过零轮。墓碑根在同一批态上不变，但我只量了一批走空那一形。

**H4　算法、nonce、MAC 写坏了，今天哪一道都看不见；「必须相同」也没法在 `RootRecord` 上比。** 坏镜像（覆盖写之后 B 那一版，txg 4，根槽里改一个字节、整槽校验和重算、把根抄回它那几条记录的新根段）：

- `research/prompts/m3-formal-c655-c687-r1-opus-model/y1-root-fields.log:17`：「name=c687-cell image=m4-B-algorithm-1 i55=Some(Holds) violated=[] partially_judged=[] turned_not_applicable=[] core_recover=file」
- nonce、MAC 各改一字节（m5、m6）同样 `violated=[]`、读回的文件与干净镜像逐字相同。

core 读根只读到偏移 428（`crates/singlefs-core/src/root_record.rs:156`：「ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86")」），`RootRecord` 没有这三样，挂载透明今天的读法（`read_recovery_fields` 取 `RootRecord`）比不到它们；R1 或 R2 是由记录施加出来的那一版时根槽根本不在（量到 `not_in_ring` 的态：第一次崩溃 s1、ul 等窗口各 3 态，二次崩溃 24 态）。四句：① 分辨：「必须相同」与「另立单镜像判据」两臂在 m4–m6 上都该红，但前者在 `not_in_ring` 那几态上没有定义；② 看得到：根槽原始字节在盘上；③ 正文 Y1「一个坏实现让它放过」；④ 改法：这三样第一版是留位、恒 0（写侧 `crates/singlefs-core/src/root_record.rs:111`：「writer.put_u8(0); // 算法类型：未加密」），是单个镜像就判得了的性质，按 D13 已定项 7 的分工归 O2：池级 checker 每条可读根判「偏移 428..457 全 0」，挂载透明那张表写「不比，归 checker」；判别力自证就是 m4–m6 三份由 `violated=[]` 转红。零轮，没实现。

**H6　系统配置几样：题面清单少一样，两样写坏了 checker 看不见。** 坏镜像（B 之后最新那一槽、两块盘一起改，整槽校验和重算），日志 `research/prompts/m3-formal-c655-c687-r1-opus-model/y1-sc-fields.log`：

- 第 10 行：「name=c655-sc-field image=q2-tail-plus-100 slot_offset=0 writable_mount=refused NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: S」——tail 多写 100，checker 不红（第 9 行 `violated=[]`），下一次可写挂载按 N-配置 拒，盘从此只能只读挂。
- 第 12 行：「name=c655-sc-field image=q3-journal-instance-9 slot_offset=0 writable_mount=mounted instance InstanceGeneration(10)」——系统配置的 journal 实例代号写成 9，checker 不红（第 11 行），下一次挂载取号跳到 10。这一样**不在** C655 题面的清单里（题面只列槽世代号与 journal_tail），挂载透明的「实例代号」那一格只比 R2 那一版根里的实例代号（`mount_transparency.rs:1158` 那一支），看不见系统配置里那一份。
- 槽世代号 +10、−1（q4、q5）I-7.16 红，checker 罩得住。

四句：① 分辨：「模掉」那一臂在 q2、q3 上漏，「按确定关系判」那一臂（Y1-2 表里 journal_tail 的两个不等式、journal 实例代号 = R2 根的实例代号）在 q2、q3 上都红，在 Y1-2 那几批合法态上都不红（第一次崩溃 28 974 态两个动作、二次崩溃 152 019 态：tail 减环里最大计数器只出现 +0 与 −1，tail 的增量没有负的，mount_version 那一类系统配置实例代号全等于 R2 根的实例代号）；② 看得到：都在 R2 择到的系统配置与 R2 的环里；③ 正文 Y1「坏实现让它放过」与「按一条确定关系判（写出关系式）」；④ 改法（零轮）：表里补「系统配置的 journal 实例代号」一行；journal_tail 写成「R1 的 ≤ R2 的 ≤ R2 环里自证记录的最大计数器，挂载做完时取等」；槽世代号写「不比，归 I-7.16」。

### Y1-4　没打中：两根指针「并进分配记录那一行」

第三轮辩方推的归法是两根指针并进分配记录那一行（那一行在 `.claude/kb/decisions/13-验证路线.md:51`：「| 分配记录 | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿 |」）我攻了两个方向，都没打穿：

- 合法这一侧：两样在合法态上大量「不同」（Y1-2 表），「必须相同」会红，「并进」不比、不红。
- 坏实现这一侧：把根槽里这两样写坏（最新那一版 m1–m3、不是最新那一版 m7–m9），池级 checker 每一份都红，例：`research/prompts/m3-formal-c655-c687-r1-opus-model/y1-root-fields.log:11`：「name=c687-cell image=m1-B-mapping-root-is-A-mapping-root i55=Some(Holds) violated=["I-3.1", "I-3.9", "I-3.11"] partially_judged=[] turned_not_applicable=[] core_recover=file」；中央映射写成别的版本的（m1）读回的文件仍与干净镜像相同，所以「POSIX 相同」那半句挡不住它，挡住它的是 checker 那半句。
- 留一个缝（推的，没量）：今天崩溃注入在「挂载加一次发布」之后的池与二次崩溃镜像上跑 checker，只挂载那一份 v2a 上不另跑；挂载写的那几版在 v2a 上是最新、在发布之后的池上不是。m7–m9 说明不是最新的那一版写坏了 checker 也抓得到，所以这个缝今天碰不到我造的这几种坏写法。

## Y2　C655：字段清单的完整性与对账判官

### Y2-1　表里、表外都没提的字段（读码，加一部分量过）

根记录（`crates/singlefs-core/src/root_record.rs` 的 `to_slot` 次序，457 字节）：表外除题面那几样，还有魔数、fsid、flags 其余位、自证校验和、树表单元指针（表里写「树表单元自己的落点不比」，算提到了）、**实例表单元指针**（表里比的是实例表的行，指针本身没提）。前四样 core 与 checker 都在读槽时判，坏了根不可择，R2 就落到别的版本上，由 POSIX 那一行与 `checkpoint_txg` 那一行间接罩着（推的）。

树表（D8 已定项 8 的条目）：表里只提两棵用户可见树与题面三根，**种类 3（分配记录树）、4（记账树）的根指针、种类 6、7、8、12 的根指针、每条的诞生 txg、头 ID、previous_snapshot_txg** 都没提。量到的（`research/prompts/m3-formal-c655-c687-r1-opus-model/summary-y2-other-tree-roots.txt`）：带树的那几版上种类 3、4 的根在每一个 mount_version 态都不同（143 934 态），种类 6、7、8、12 的根全零、都相同；落回 v1 的都相同。我的归类：3、4 并进「分配记录」那一行，6、7、8、12 与诞生 txg、头 ID 必须相同（后两样有 I-9.14、I-9.27 在单镜像上判，推的没量）。

系统配置（`crates/singlefs-core/src/system_configuration.rs` 的 `to_slot`）：解槽读进内存的只有 fsid、本盘设备号、设备数、槽世代号、物理块、环起点、环长、根环起点、三区域设备、io_min、槽距、S、tail、journal 实例代号、F；挂载透明的「不可变配置」比的是内存里那几样，**槽里其余字节一样都不比**：魔数、格式版本、feature bits、写入者身份、校验和算法标识、加密留位六行（系统配置 MAC、nonce 水位、KDF、加密类型、MAC 长度、主密钥槽）、几何段里不进内存的十几样（节点大小、单元大小、落点粒度、位置条目宽、扩展点 N、记录尺寸、在飞上限、最坏占用、安全系数、R、P、chunk、条带宽、组大小、映射来源、单元区起始槽号）、运行配置三行（T_time、T_dirty、三水位）。其中加密类型、格式版本、几何那几样 checker 有 I-7.13 罩着（`crates/singlefs-checker/src/image.rs:54`：「pub const SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS: &str = "I-7.13";」）；写入者身份、运行配置三行、加密留位里加密类型以外几样，我没找到哪一道罩（推的，没造坏镜像）。另：题面清单漏了**系统配置的 journal 实例代号**（Y1-3 H6 q3）与**系统配置里那一份 F**（「F 与 F 生效值」那一行只比生效值，生效值取两处最大，系统配置那一份写低了被根里的盖住，推的）。

### Y2-2　对账判官的字段清单从哪取

攻「从代码里的结构体取」：`RootRecord` 没有算法、nonce、MAC（H4），`SystemImmutableConfiguration` 37 行里只装 7 个字段（它自己的文档注释，`crates/singlefs-core/src/system_configuration.rs:228`：「这 37 行里只有下面 7 个字段（`sizes` 里 5 个值，合起来 11 个值）在内存里存着」）；checker 的 `RootView` 只有五个字段加整条字节。从这两处取清单，格式加一个只进盘、不进结构体的字段，判官看不见——H4 那三样今天就是这种字段。

我提的判官（零轮，没实现）：逐字段表每一行带它罩的字节区间（根记录按偏移，系统配置按偏移），判官按区间求并，必须恰好盖满 [0, `ROOT_RECORD_BYTES`) 与 [0, `SYSTEM_CONFIGURATION_BYTES`)，不许重叠也不许有洞（`crates/singlefs-format/src/lib.rs:210`：「pub const ROOT_RECORD_BYTES: u64 = 457;」；`crates/singlefs-format/src/lib.rs:248`：「pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;」）。格式加字段要改这两个常量（门禁 format-const 钉着 kb），表不跟就有洞；删表里一行同样留洞，判别力自证就是「删一行必须红」。系统配置那边已有同形的先例：每一档的字节数常量（`crates/singlefs-core/src/system_configuration.rs:249`：「pub const FIELD_TABLE_BYTES: u64 = 389;」）加单测 `to_slot_writes_exactly_the_budgeted_bytes_into_each_mutability_class`（`crates/singlefs-core/src/system_configuration.rs:964`）。树表条目同理按 `TREE_TABLE_ENTRY_BYTES` 200 罩。推的：按名字对账（字段名清单）挡不住「加了字节、没加名字」。

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| 开放段回落误排（Y3） | mkfs 之后 N ∈ {1–16, 18, 20, 24, 28, 32, 40, 48, 64} 次可写挂载、每次后接 K ∈ {0, 1, 2, 3, 4, 8, 16} 次零单元发布（组合见 `fallback-*.log`），再写第一个文件；三种小盘；准入照式子判；另 3 段第一个文件之后接 20–60 次满单元覆盖写（准入开、关各一遍，只在终端看过，没落日志） | 回落 0 次，I-5.5 全 `holds`；计数器的阳性对照 22 次 |
| I-5.5「判不了」那七格在合法镜像上出现 | 第一次崩溃 28 974 态、二次崩溃 152 019 态（s1、s2、fr、ow、ul、ci、rb、rbow 各窗口） | 0 次 |
| 两根指针写坏逃过「分配记录」那一行 | m1–m3（最新那一版）、m7–m9（不是最新那一版） | checker 每份都红 |
| 墓碑根、谱系根「必须相同」在合法历史上红 | 一批走空的 unlink、unlink 后建 inode、回退、回退后覆盖写 | 0 次 |

没造的（都只是没试，不是试了没中）：多于 144 个单元、要分几批走空的 unlink（墓碑根在中间几批会不会变）；正常卸载之后再挂（卸载记号那一形）；带读故障、被抛弃根的历史（`CrashRecoveryAbandoningTheNewestRoot`，影子账隔离位压在 [U, U + 64) 里的那一种 I-5.5 近似）；挂载末尾实例表清扫真写了东西的历史（我的历史里实例只到 2–3 个，没核清扫有没有动笔）；4 GiB 盘上的大窗口（ul 全窗口 68 157 463 态、ow 16 777 228 态超过约 10⁶，没跑，改跑了 `@a-b` 收窄的窗口：前面几段整段持久并进基线、留下的段每个崩溃态都枚举）。

## 这条腿自己的限度

- 量的数全是副本上的：副本 = 冻结副本加四处探针改动（`recovery.rs` 加只读函数 `probe_tree_table_entries_of_root`、`allocator.rs` 回落那一支加计数、harness 一份测试文件末尾加探针用例、新 crate `c655probe`），产品逻辑一行没改。主 agent 要在入库装置上重做才能引。
- 每一格只抽了一次。H3、H4、H6、Y3-1-H1、Y3-1-H2 是「打中」，按规则一次就值得核；「没打中」那几格（回落、两根指针）只算这一次。
- 我的探针与崩溃注入共用 `mount_and_publish_once_on_the_crash_image` 与 `read_recovery_fields`：它们自己有错，我量到的「合法态上的关系」跟着错，这一点我核不了。
- 历史是我挑的八段，窗口按段收窄过（`ul@1-4`、`ul@5-8`、`ow@1-4` 等）：被并进基线的那几段（大段单元写）里的崩溃态没枚举。崩溃态总数：第一次崩溃 28 974、二次崩溃 152 019，另有 Y3 的 15 份坏镜像与 Y1 的 14 份坏写法镜像；合计远低于约 10⁷。
- 一版之内的对账判官、I-9.16 新一道、deleted_inodes 并进续做那一行、根留位归 checker、系统配置 journal 实例代号一行：全是我提的，被攻过零轮，没实现、没跑（只量了「改之前」那一半）。
- checker 档、GPU、门禁都没跑；GPU 那几处改法是读 WGSL 推的。

## 没做什么

- 不判正推腿那几格，不替主 agent 采纳；没读禁读清单里的任何文件。
- 没有要主 agent 另跑的长命令：全部量在前台跑完（每条不超过 240 秒）；复跑见开头那一节。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-formal-c655-c687-r1-opus-output.md --root /tmp/claude-1000/m3-formal-c655-c687-r1/tree --root . --background research/prompts/_m3-formal-c655-c687-r1-background.md` 末行：

```
  ✓ 核了 21 处引文，对上 21 处，没判 0 处（逐处列在上面）
```
