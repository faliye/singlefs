# m3-prune-gpu-r2 云端攻方（Opus）报告（2026-09-28）

攻击面：B1–B7 与 D1 的精确性，另加 B8 的版本号。先写出我认为最强的一版身份值（下文「被攻的最强版本」一节），再逐项攻它；每个打中都做成一个 `must_be_nonzero=` 的世界，在 E161 装置的副本上跑（第一轮攻方原型 `attack.rs` 下面挂一个子模块 `r2.rs`），数全部出自副本，**不算入库装置上的数**。第一轮十个世界与 F1、F2 在最强版本下各判什么，见「第一轮十个世界与 F1、F2 在最强版本下」一节。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-prune-gpu-r2-opus-model/`：`attack.rs`（第一轮攻方原型原样，只多三行把 `r2-` 开头的子命令转给 `r2.rs`）、`r2.rs`（这一轮的十五个世界）、`e161-hook.patch`（第一轮那份：E161 装置里挂 `attack` 子命令）、`crates-probes.patch`（只在副本里打：池级 checker 扫描方向三处聚合的输入摘要，P7 键用；层 0 计划哈希与回收谓词各开一个外露口）、`transaction-root-before-records.diff`（第一轮 F2 调查员那份：真实现改成先根后记录，只打在第二份副本上）、`rerun.sh`、`outputs/`（十六份原样输出，去掉了 `LAYER0_*` 进度行）。

```
bash research/prompts/m3-prune-gpu-r2-opus-model/rerun.sh <仓根> <一个不存在的草稿目录>
```

它把仓拷进草稿目录、放进两份原型、打上两份补丁，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4` 开 `verdict-store` 特性编译 E161 装置，逐个跑十五个世界；再拷一份、打上先根后记录的真实现补丁另编一次，只跑 `r2-b5-implementation`；最后逐个与 `outputs/` 比，比的时候只剥掉 `seconds=` 与 `build_seconds=` 两个字段，整行不扔。

```
cee7b16e0da35e4c7d9370eee0677b7d54fc5863e8b2432947780108fdca2f35  attack.rs
9b465ee94325e4e775499d78d7414accf04d4f0d2e1684b6b67dc797d54f511e  r2.rs
43c308a1db987d535eb09611d796db6e5f666486b90f2e28a2cddb3ef5f1ec19  e161-hook.patch
a681cc9482760bbedb9c997b5ee06d5f83c3226eadd66b9bc131dfcfaa01abe7  crates-probes.patch
c412c7c9b7968c20185504f848821d92a9d35b057f0892324d1204afeb6d8e76  transaction-root-before-records.diff
94ab0b0b2015fa8e07d6814204a262cd256b2d0f697bbc7ecf04eea851c961a9  rerun.sh
eb3b65ee127cbb561f32fac1e6fc0c123608925ab20c97a5cee9ff1802b04695  outputs/r2-b1.out
116e2cae78069d90243060b3b3d3e2953c2ef3e060cb4f1a73cb1229aa9cde8e  outputs/r2-b2-downstream.out
0788558f875f6e4851f32c4f92adb05674656ee50032bc298d2ab6e2d4776ea6  outputs/r2-b2-versions.out
b6af4991e42a629330a1175ff111133a0b7a09bdaf4d0ffa31e1641ed08c3e2f  outputs/r2-b3.out
b6a11433860b7b3c6aa692bdfe3f5340d7c0a8b243752d4ac02314d48f4239dc  outputs/r2-b4-messages.out
77905cada3f486b5b9bacf0addfb93a70a4e6a59cb10d89e7997e1de60bcd235  outputs/r2-b4.out
6cd85719fe932d747114d2b0e1a082ba1b848d86baec77d1afdf7fad5be13842  outputs/r2-b5-implementation.out
b5319b11c9fbe542986ec247ec8297b0c1cc71bf1de579ff821684b6591a4907  outputs/r2-b5-implementation-root-before-records.out
c31d8b1844e0418ff8b5cc1ebfd7553caf67b0c3f4adfc709f4509785ce63b73  outputs/r2-b5-legal-reuse.out
43d040043e144ed4449ea38a1ea6d4e9727ebbf0e1f088301edf06a1cf30be6c  outputs/r2-b5.out
15d909abc0049dab09f3859cf7e377d940044cde7051aced522d65b974de35e5  outputs/r2-b6.out
b562b160fb86f2cc6ef080f3d485eaeb73d48dd04c4f94078c5ae2c563538e60  outputs/r2-b7_150.out
f8e9a671850d299b6b58f5ac105d1c9c66a50b4dde2fa25467947afa5801edd7  outputs/r2-b8.out
fa2e182ae9b30b460b4ff4bc1590bfb08fcbe7552966fd1c0f5399c9b694b407  outputs/r2-d1.out
b921321e853e31bc654e6c20985b569a77f9d398e2daf2067df8c9ab4ee95045  outputs/r2-f1-shifted.out
54ee9bf855e447e083e01bdf28d46cb88aa6496d4e90bbbea58b347543eae282  outputs/r2-kinds.out
```

取样（照定义 3b）：历史全在内存稀疏盘上录，起点池各建一次；崩溃状态一律经 crates 的 `enumerate_layer0_selecting_versions_observing_each_state` 取（段内整写子集逐个展开，带第三态），每个世界跑前现算 `closed_form_state_count` 与带第三态的实数，断言不超过 10⁶（`r2_state_budget` 行）。最后一批十四个世界的 `r2_state_budget` 行加起来 15941 个状态，单次最多 4128 个；之后加的 `r2-b5-implementation` 在健康与先根后记录两份编译上各 45 个。前几批调试跑与一次复跑核对的输出已被覆盖，没法逐行加；按「五批加一次复跑、每批不超过最后一批的 3 倍（同一历史的重复枚举不打 budget 行：r2-b1 一次跑三份历史，r2-b2-downstream 每段另起一次前缀枚举）」算，上界约 29 万个状态（推的上界），远低于 10⁷。**缩法**：凡是历史里有 24 写以上单元段的世界（第一条流的新建文件那段 2²⁴、覆盖写那几段），照派发提示「只跑小域」只展开段长 < 16 的段，其余段只以整段持久进入后面的状态；p6 类世界（B4 的 p6、B5 的 p6 与合法复用）把单元段截成录制流前缀的前 12 次写（缩历史长度），这 12 次写的 4095 个状态一个不落。这与定义 3b「留下的每段历史，它的崩溃状态一个不落」冲突，照派发提示办，写在「这条腿自己的限度」一节。

## 各格判定一览

`must_be_nonzero` 一列写成「—」的是对照或回归，不要求非 0。

| 格 | 被攻的那一版 | 判定 | 世界（输出行） | must_be_nonzero |
|---|---|---|---|---|
| B1 并错 | 种类串 + 覆盖签名（每次写记最近一次重叠写的种类与中间隔几次根槽写） | **打中**：改坏的实现只把两份数据单元载荷各写错一个字节，每一段的 B1 身份都与健康流相同，段 10 里 8 个状态判定不同 | r2-b1.out:2 | 8 |
| B1 拆错 | 路径写表哈希当节点身份 | **打中「省不下来」**：只把写入时间改 1 秒，段 8–10 的 12 个状态身份不同、判定逐项相同 | r2-b1.out:3 | 12 |
| B1 回归 | 种类串 + 覆盖签名 | 没打中：第一轮 kinds 那 33 次发布分成 5 个身份，同一身份里并着不同条件的 0 个 | r2-kinds.out:1 | — |
| B2 版本表 | 路径写表哈希 + 判法摘要（摘要罩 checker、恢复、枚举器的代码，不罩测试的版本表） | **打中**：写表逐字节相同，版本表改一个字节，45 个状态里 12 个判定不同 | r2-b2-versions.out:2 | 12 |
| B2 下游写 | 节点的判定只取到这个节点为止的写表 | 没打中：六条历史（三条健康、屏障放错、先根后记录两形、提前复用）共 300 个状态，整条路径与只到本节点两种写表下判定 0 个不同 | r2-b2-downstream.out:2、:4、:6、:8、:10、:12 | — |
| B3 单点 | （节点，段内序号），单点从物化的父结束镜像起算 | **打中**：第一轮 t4 变体 b 的子节点，整条路径 5 个状态、单点 3 个，5 个序号里 4 个指到不同的落法或不存在 | r2-b3.out:1 | 4 |
| B3 ignore 开关 | 全流序号 + 计划哈希（KV 第一段的块键） | **打中「省不下来」**：把段 2 从展开改成 ignore，其余 44 个状态的（计划哈希，序号）全变，其中 11 个在段 2 之前 | r2-b3.out:3 | 44 |
| B4 P7 精确与省 | 读集键 → P7 键（恢复两遍的整串调用 + 走读不扫描那一遍 + 扫描三处聚合的输入摘要） | 没打中：五个域同键异判 0（按项红绿与整份摘要都是 0）；p6 单元段 4095 个状态 → 4 类，p5 改坏的 16 个红状态单独成类 | r2-b4.out:2、:4、:6、:8、:10 | — |
| B4 P7 与违例正文 | P7 键，判定向量带违例正文 | **打中**：两个写序实例代号过高的节点分别落时 I-7.7 都红、正文点名的槽不同，P7 键相同，16 个状态整份摘要不一致（按项红绿 0） | r2-b4-messages.out:2 | 16 |
| F1 在叠加形态下 | 今天层 0 的候选槽（持久了的每次单元写的起点），也是「读集键 + 扫描按单元边界认槽」取写表里单元写起点的那一读法 | **打中**：一个 16K 节点先占槽 60001、之后一个 32K 数据单元从槽 60000 盖过它，数据单元内容里像节点头的字节被扫描读成单元头，2 个状态 I-7.8 假红；按「这一槽上最后一次持久单元写从这里开头」认槽 0 个 | r2-f1-shifted.out:4 | 2 |
| B5 归属 | 按记录自己的（实例，txg）归发布 | 没打中（改法起作用）：F2 两形各 1 个状态转红（今天 0），三条健康流与提前复用 0 假红；真实现改成先根后记录，45 个状态里 27 个真洞，今天五样判定 0 个红，按自己身份归属 27 个红、0 个假红 | r2-b5.out:2、:4、:6、:8、:10、:12；r2-b5-implementation-root-before-records.out:2 | — |
| B5 剪枝键（第一版） | 恢复落到的根 + 记录核对器的读集 + 每份被核单元副本上所有更晚重叠写的持久位 | **打中「省不下来」**：合法复用 txg 3 的 12 份副本，4095 个状态判定只有 1 种，键 4095 类 | r2-b5-legal-reuse.out:2 | 4094 |
| B5 剪枝键（符号版） | 恢复落到的根 + 每条根槽写与记录写在不在盘上 + 过不了回收谓词的后写落没落 | 没打中：七个域同键异判 0；p6 单元段与合法复用段各 1 类 | r2-b5.out:14、r2-b5-legal-reuse.out:2 | — |
| B6 | SHA-256 | 没打中：第一轮 collide 那一对 128 位摘要相同、SHA-256 不同 | r2-b6.out:1 | — |
| B7 计数格 | 判定向量按节点共用，计数从向量重算 | **打中**：共享前缀的段 10 在两条路径上归不同的发布，8 个状态的计数格不是节点的函数 | r2-b7_150.out:1 | 8 |
| B7 编号宽 | 1 字节编号 + 向量带恢复落到的根 | **打中**：150 次覆盖写的小段 1845 个状态有 307 种向量，真库在第 257 种上拒登记；不带根时 7 种 | r2-b7_150.out:3 | 51 |
| B8 | 无版本 / 库级布局版本 / 定义版本进键前缀（抬了、忘抬）/ 字段名表哈希进键 / 金丝雀状态三种取法 / 存逐项原始判定 | **打中**：两种改定义（拆字段、同名改义）下，只有「抬了的定义版本」与「存逐项原始判定」两种放法两次都 0；其余每种至少一次读错 4 或 16 个状态 | r2-b8.out:4–21 | 4、16 |
| D1 按块导入 | 各写一份、跑完按块原样导入 | **打中**：159 个状态里 79 个读成别的判定向量，库自己的核对不拦 | r2-d1.out:3 | 79 |
| D1 换块长续跑 | 块长不进键 | **打中**：被杀之后换块长从中途续跑，8 个状态被两块罩住、按块求和多数 8 个，库照收 | r2-d1.out:5 | 8 |
| D1 计划哈希 | 块键里的计划哈希取层 0 今天那一份 | **打中**：同一条流七种跑法六种计划哈希（线程数、分片号、有没有观察者都进哈希） | r2-d1.out:6 | 5 |
| D1 片跨节点 | 块 = 续跑的片 | **打中**：第一条流全量 65282 片里 2 片跨段 | r2-d1.out:7 | 2 |

## 被攻的最强版本

第一轮判决第四节的改法 1–4、6（被攻过零轮）是底子；下表逐项写我在它上面收严的地方。「我加的」都只在我的模型上量过、被攻过零轮。

| 身份 | 最强版本 | 从哪来 |
|---|---|---|
| B1 节点身份 | 沿路径把每段的（写种类串，覆盖签名）串起来求 SHA-256；覆盖签名 = 每次写与路径上离它最近的那一次重叠写的种类、中间隔了几次根槽写，没有重叠写记 `-` | 第一轮改法 2；「隔几次根槽写」与「只记最近那一次」是我加的（第一版记全部更早的重叠写，每轮都写的系统配置槽让签名随发布次数一直变长，r2-kinds 量出 33 次发布 33 个身份，一次都共享不了；那一版的输出被后一次跑覆盖，只剩这里的抄录） |
| B2 节点输入指纹 | SHA-256（定义版本串 ‖ 基线镜像全部已写扇区 ‖ 路径上每次写的盘、种类、FUA、偏移、内容）‖ 判法摘要（checker、恢复、枚举器与记录核对器的代码） | 第一轮改法 3；「基线整份进哈希」是我加的（第一轮只说「整条路径写表」，没说 mkfs 那份基线在不在里面） |
| B3 状态身份 | （节点，段内序号），序号按枚举器的混合进制（能撕的写三态、别的两态）从小到大排 | 第一轮 S1 站住的那一形 |
| B4 恢复与池级 checker 的键 | P7 键：恢复两遍的整串调用（位置 + 内容 SHA-256）+ 池级 checker 走读不扫描那一遍的整串调用 + 扫描方向三处聚合的输入摘要（I-7.8 扫到的最大树 ID、I-7.7 单元区载体每盘的最大写序实例代号、I-1.8 两成员以上的归并组与同一对象键下两组以上的定序）；候选槽按「这一槽上最后一次持久单元写从这里开头」认 | 第一轮改法 5（P7）；三处聚合各取什么、候选槽按「当前」单元起点认是我加的 |
| B5 记录核对器 | 记录按自己字节里的（实例，txg）归发布；剪枝键 = 看 journal 那一遍恢复落到的根 + 每条根槽写与 journal 记录写在不在盘上 + 恢复落到的那一版及更早的发布的每份单元副本上、更晚而**过不了**回收谓词的写落没落 | 归属是 C588（记录核对器按写表次序错归发布） 的候选之一；剪枝键是我加的，第一版（带全部重叠后写）被 r2-b5-legal-reuse 打穿之后收成这一版 |
| B6 内容身份 | SHA-256（按调用种类分域） | 第一轮改法 4 |
| B7 判定向量 | 逐项红绿：两遍 oracle 的违例类、池级 checker 每条不变量的 成立 / 违反 / 不适用、记录核对器两条，不带违例正文、不带恢复落到的根 | 我加的（KV 第一段只存「调用方定的编码」，没有定义） |
| B8 版本 | 判定向量存逐项原始判定（名字 + 字母），读的一方现算要的位；块键里带判定向量定义的版本号 | 我加的，r2-b8 的十种放法里挑出来的两种 |
| D1 | 各写一份、跑完按块导入，导入时按向量内容换编号；块 = 一个节点的状态、块键不带分片号与线程数 | 我加的，r2-d1 的四组里挑出来的 |

## B1 节点身份：并错与拆错两头都打中

世界 r2-b1（`r2.rs` 的 `world_b1_label_versus_path_hash`）。三条历史：X 第一条流；Y 同一串调用、新建文件的写入时间 +1 秒；Z 把 X 里两份 32K 数据单元的载荷各翻一个字节（写的位置、长度、种类、次序都不变，校验和按原载荷算）——一个写坏载荷的实现。三条都只展开段长 < 16 的段。

```
E7RESULT name=r2_b1_merge world=x_versus_payload_mutant_z segments=11 segments_with_equal_b1_label=11 compared_states=44 states_equal_label_status_differs=8 in_segments={10} example_z=Some("segment=10 red=[\"oracle_consult:read_failed\", \"oracle_ignore:read_failed\", \"checker:I-2.1\", \"checker:I-4.8\", \"checker:I-7.2\", \"checker:I-7.4\"]") must_be_nonzero=8
E7RESULT name=r2_b1_split world=x_versus_write_time_plus_one_y segments=11 segments_with_equal_b1_label=11 segments_with_equal_path_hash=7 compared_states=44 states_path_hash_differs_status_equal=12 in_segments={8, 9, 10} must_be_nonzero=12
```

- 并错：X 与 Z 的 11 段 B1 身份逐段相同（`segments_with_equal_b1_label=11`），段 10 的 8 个状态判定不同（两遍 oracle 读失败、四条不变量红）。
- 拆错：X 与 Y 的 B1 身份逐段相同、路径写表哈希从段 7 起不同（`segments_with_equal_path_hash=7`），段 8–10 的 12 个状态按项红绿逐个相同。
- 回归（r2-kinds）：「kind_strings_merging_conditions=1」（`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-kinds.out:1`）是第一轮那一格照旧；换成最强版本之后「identities_merging_conditions=0」（同一行），33 次发布 5 个身份。

四句：
1. **分不分辨臂**：分。岔路单 B1 的三个候选里，种类串与种类串 + 覆盖签名在并错这一格上一起中，路径写表哈希不中；拆错那一格反过来，只有路径写表哈希中。没有一个候选两格都不中。
2. **系统看不看得到**：看得到。Z 的载荷在写表里（路径写表哈希分得开），Y 的写入时间也在写表里；是 B1 的定义把它们丢了，或者没丢。
3. **满足的分句**：岔路单 B1 行的翻面观测「某种身份下两个判定不同的节点被认成一个；或同一个节点在两条路径上被认成两个、共享不了」（`research/prompts/m3-prune-gpu-r2-forks.md:9`），并错满足前一个分句，拆错满足后一个。
4. **改法还中不中**：第一轮改法 2（种类串 + 覆盖签名）在并错一格照中。能两格都不中的只有把 B1 拆成两样：共享与复用只认路径写表哈希（B2），B1 只当登记、覆盖报告与 ignore 行里点名用的标签，任何「标签相同就当同一份结果」的用法都不许有。这个改法只在我的模型上量过（量的是两格各自的数），被攻过零轮；标签「不许当键」这一条靠什么会红的检查守，推的，没做。

## B2 节点输入指纹

### 版本表不在指纹里（世界 r2-b2-versions）

第一条流，写表不动，只把版本表里 txg 3 那一版的期望内容翻一个字节（测试这一侧的期望改了，比如用例改了期望内容的算法）：

```
E7RESULT name=r2_b2_versions path_write_table_hash_equal=true path_hash=cfda4c8fae95bb9c states=45 states_whose_status_differs=12 example=Some("original_red=[] changed_red=[\"oracle_consult:wrong_content\"]") must_be_nonzero=12
```

路径写表哈希相同，45 个状态里 12 个判定不同（看 journal 那一遍 oracle 报内容不对）。层 0 今天的计划哈希是把版本表算进去的：「for version in versions {」（`crates/singlefs-checker-tier/src/crash.rs:2348`）；第一轮改法 3 的「整条路径写表哈希 + 判法摘要」如果判法摘要只指代码，就漏了它。

四句：
1. **分不分辨臂**：分。岔路单 B2 的三个候选（父结束镜像 + 本节点写表 + 判法摘要、整条路径写表 + 判法摘要、按走到的代码模块）都不带版本表，三个一起中；带上版本表的那一版不中。所以它不分辨岔路单里的候选，分辨的是「指纹带不带测试这一侧的期望」——按「判据自己也会写错」那张表是「打中不分辨臂」：共用前提（指纹只罩实现与判法代码）另立一笔账先修。
2. **系统看不看得到**：看得到，oracle 在每个状态上读版本表。
3. **满足的分句**：岔路单 B2 行「存在一处改动让判定的输入变了而指纹没变」（`research/prompts/m3-prune-gpu-r2-forks.md:10`）。
4. **改法**：指纹里加版本表的 SHA-256（同计划哈希的做法）。推的，没另跑；它修的只是这一格。

### 节点的判定依不依赖它之后的写（世界 r2-b2-downstream，没打中）

读代码时怀疑过三处会读到节点之后的写：记录核对器按「后面第一条根」归发布、`checkpoint_txg_of_the_publish_that_made_the_write` 往后找根、读恢复落到的那一版的实例表时从整张写表的末尾往回找根槽写（「.rev()」，`crates/singlefs-checker-tier/src/crash.rs:180`）。如果节点的判定依赖之后的写，B2 按「到这一节点为止」算的指纹就罩不住，α 的「每个节点只枚举一次」也不成立。我把每个小段里的每个状态在两种写表下各判一遍：整条路径的，和截到这一段末尾的。

| 历史 | 比了几个状态 | 判定不同 |
|---|---|---|
| 健康第一条流 | 33 | 0 |
| 健康第一条流 + txg 4 覆盖写 | 45 | 0 |
| 屏障放错：24 次单元写里后 4 次与两条记录之间没有屏障 | 93 | 0 |
| 先根后记录，最后一次发布（第一轮 order） | 33 | 0 |
| 先根后记录，后面还有 txg 4 | 45 | 0 |
| 提前复用 txg 3 的数据单元（第一轮 reuse） | 51 | 0 |

（出处：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b2-downstream.out` 第 2、4、6、8、10、12 行，每行的 `states_whose_status_differs_full_versus_prefix=0` 与 `states_whose_full_digest_differs=0`。前缀里一条根都没有的段 0、1 没比，各行 `skipped_segments_without_a_root_in_the_prefix=[0, 1]`。）

读的那三处都只在「根没落、记录已落、恢复按记录重建」或「有被抛弃的发布」的状态上才会碰到之后的写；这六条历史里没有被抛弃的发布，回退那一形（第二条流里的）我没造（第二条流要在盘上建镜像文件，定义 3b 不许），所以这一格只能写「这六条上没打中」。

## B3 状态身份

### 单点从物化的父镜像起算，序号对不上（世界 r2-b3 第一部分）

第一轮 t4 世界的变体 b：父节点结束镜像与变体 a 逐字节相同，子节点两条记录写相同。子节点里「段内序号 → 各次写的落法」按整条路径排一遍、按「从 KV 取父节点结束镜像」（岔路单 C5 的第一个候选）排一遍（落法 0 没落、1 撕裂、2 落了）：

```
E7RESULT name=r2_b3_single_point_numbering world=t4_variant_b child_states_full_path=5 child_states_from_materialized_parent=3 ordinals_mapping_to_different_landings=4 detail=["1:Some([1, 0])/Some([2, 0])", "2:Some([2, 0])/Some([0, 2])", "3:Some([0, 2])/None", "4:Some([1, 2])/None"] must_be_nonzero=4
```

序号 1 在全量里是「第一条撕裂、第二条没落」，在单点里是「第一条落了」；序号 3、4 在单点里不存在。机理同第一轮 t4：能不能撕看写表里更早的写，「base_holds_nonzero_bytes_there || an_earlier_write_with_bytes_overlaps」（`crates/singlefs-checker-tier/src/crash.rs:1600`），物化之后后一半恒假。

四句：
1. **分不分辨臂**：分。按整条路径的写表定每次写几态（第一轮改法 1），单点与全量逐号相同；只拿父结束镜像的不同。它分辨的是岔路单 C5 的两个候选（从 KV 取父节点结束镜像 / 从根重放到父节点）。
2. **系统看不看得到**：看得到，写表历史就在路径上。
3. **满足的分句**：岔路单 B3 行「某种表示下单条线、单点、分片、GPU 批给同一个状态的号不同」（`research/prompts/m3-prune-gpu-r2-forks.md:11`）。
4. **改法**：单点从 KV 取父结束镜像可以留（读的代价），但每次写几态、撕裂镜像的旧半，必须从整条路径的写表算，也就是节点登记表里要存「这个节点每次写取几态」这一列、单点照它排号。推的，没实现。

### ignore 一个节点，别的节点全部重跑（世界 r2-b3 第二部分）

KV 第一段的块键是「块起点：这一块第一个状态在它那份枚举计划里的序号。块长由调用方定，不进键。」（`crates/singlefs-checker-tier/src/verdict_store.rs:41`）加上计划哈希；计划哈希把每一段怎么展开都算进去：「for (segment, expansion) in plan.segments.iter().zip(&plan.expansion_by_segment) {」（`crates/singlefs-checker-tier/src/crash.rs:2336`）。第一条流把段 2（一条根槽写）从展开改成 ignore：

```
E7RESULT name=r2_b3_ignore_toggle toggled_segment=2 states_before=45 states_after=44 states_in_both=44 ordinal_moved=33 plan_hash_changed=true states_in_both_whose_plan_hash_plus_ordinal_changed=44 of_which_in_segments_before_the_toggled_one=11 must_be_nonzero=44
```

两次都枚举到的 44 个状态，（计划哈希，全流序号）全变，11 个在段 2 之前（序号没动，只是计划哈希变了）。用户定案是 ignore「慢慢开启」：每开一个节点，整条流的块都作废重跑。

四句：
1. **分不分辨臂**：分。按（节点，段内序号）当身份、块键只带这个节点的指纹时，开关别的节点不碰这 44 个。
2. **系统看不看得到**：看得到，开关的是哪一段是明写的。
3. **满足的分句**：「键不同而本该是同一类」导致省不下来（派发的攻击面原话）；不是精确性，是 D13（验证路线） 已定项 11 那三条「ignore 的节点什么时候开」要付的代价。
4. **改法**：状态身份用（节点，段内序号），块不跨节点、块键里的计划哈希按节点算（只罩这个节点的写与它每次写几态）。推的，没实现；它同时修 D1 的「片跨节点」那一格（见 D1 节）。

## B4 恢复与池级 checker 的等价类键

### P7 键精确、单元段上省得下（世界 r2-b4，没打中）

P7 键怎么量：恢复两遍的整串调用、池级 checker 在「候选槽给空表」那一遍（走读，不扫描）的整串调用，再加上正常那一遍扫描方向三处聚合的输入摘要——三处是 I-7.8 扫到的最大树 ID、I-7.7 单元区载体每盘的最大写序实例代号、I-1.8 两成员以上的归并组（归并键、各成员载荷校验和、校验和不同时哪几份载荷读得出）与同一对象键下两组以上的定序。摘要由 `crates-probes.patch` 在副本的 `walk.rs` 里取，不改判定。「同键异判」按两种判定比：整份摘要（与第一轮同一算法，带违例正文）与按项红绿。

| 域 | 状态 | 判定种数（按项） | 读集键类数 | P7 键类数 | P7 同键异判（整份 / 按项） |
|---|---|---|---|---|---|
| p5 对照 | 64 | 1 | 32 | 4 | 0 / 0 |
| p5 改坏（16 个 I-7.8 红） | 64 | 2 | 48 | 6 | 0 / 0 |
| p6 单元段（新建文件前 12 次单元写） | 4095 | 1 | 4095 | 4 | 0 / 0 |
| 第一条流小段 | 45 | 4 | 45 | 45 | 0 / 0 |
| 第一条流 + txg 4 小段 | 57 | 6 | 57 | 57 | 0 / 0 |

（出处：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b4.out` 第 2、4、6、8、10 行。）

- 第一轮判决第六节要攻的第 ①条「P7 在单元段上能不能把叶子数降到判定类数，且 p5 那一格照样抓得到」：在 p6 上降到 4 类（判定 1 种；多出来的 3 类，推的：是 I-7.7 每盘最大实例代号那一项在「这块盘上有没有新单元落了」之间变，2 × 2 = 4，没逐项拆开核），p5 改坏的 16 个红状态没有与绿状态同键。所以我给的摘要还能再收：I-7.7 那一项只要「全部载体的最大值」，不要分盘。推的，没改没跑。
- 小段（记录、根、系统配置）上 P7 不省：键类数等于状态数。小段本来就便宜，不碍事。
- 这是**按每个状态实际跑一遍再取键**量的类数，说的是「类有几个」；P6 能不能不逐个跑就把这几类分出来（按写符号求聚合），没实现、没量。

### P7 键下违例正文不同（世界 r2-b4-messages，打中）

单元段里两次写：两个码 2 节点，写序实例代号改成 7（高于系统配置里的 1），树 ID、诞生代号不动，头校验和重算；后面接新建文件那段的前 4 次单元写。I-7.7 的违例正文点名第一个越界的载体：「format!("{what}带实例代号 {instance}，高于各盘系统配置的最大者 {pool_highest}（①）")」（`crates/singlefs-checker/src/walk.rs:2578`），载体名里带槽号：「carriers.push((format!("盘 {device} 槽 {slot} 的单元写序"), instance));」（`crates/singlefs-checker/src/walk.rs:2516`）。

```
E7RESULT name=r2_b4_keys world=two_nodes_with_a_high_instance_in_a_unit_segment states=63 red_states=47 distinct_full_verdicts=3 distinct_status_vectors=2 read_set_key_classes=63 read_set_key_inconsistent_full=0 read_set_key_inconsistent_status=0 p7_key_classes=6 p7_key_inconsistent_full=16 p7_key_inconsistent_status=0 p7_classes_minus_distinct_status=4 must_be_nonzero=16 example_red_with_only_the_first_node=Some("[\"checker:I-7.7\"]")
```

只落第一个节点与只落第二个节点的状态，P7 摘要都是「盘 0 最大实例代号 7」，同键；按项红绿都是 I-7.7 红，一致；整份摘要里正文点名的槽不同，16 个状态不一致。读集键下 0（它读到了是哪一槽）。

四句：
1. **分不分辨臂**：分。读集键不中，P7 键中；按项红绿不中，带正文的判定中。
2. **系统看不看得到**：看得到，正文就是扫描读到的槽号。
3. **满足的分句**：岔路单 B4 行「键相同而任一项判定不同」（`research/prompts/m3-prune-gpu-r2-forks.md:12`）——只在「判定」带违例正文时满足；判定按项红绿时不满足。这是 B4 与 B7 要一起定的一条：P7 只对「不带正文的判定向量」精确。
4. **改法**：二选一，都推的：判定向量（B7）不带正文，正文只在违例表里给「代表状态」现算一次；或者 P7 摘要带上「第一个越界的载体」这一项（类数会随越界的槽数涨）。我在最强版本里取前者。

### F1 在今天的叠加形态下也出现（世界 r2-f1-shifted，打中）

C587（扫描候选槽头被伪造头骗过） 已经写了物化与整盘扫两形；它的题面写「候选槽为 None（整盘扫）时同样中招」（`.claude/kb/checks-owed.md:467`）。这里是第三形：层 0 今天用的叠加候选。叠加的候选是持久了的每次单元写的起点：「slots.insert(write.offset.0 / SLOT_BYTES);」（`crates/singlefs-harness/src/memory_pool.rs:747`）。一个 16K 节点先写在槽 60001（盘 0），之后一个 32K 数据单元从槽 60000 写起、盖过 60001；数据单元内容第 16384 字节起是一段像码 2 节点头的用户数据（树 ID 2⁴⁰、诞生代号 1，头校验和自洽）。槽 60001 仍在候选里，扫描把数据单元的后半当单元头读：

```
E7RESULT name=r2_f1_shifted world=data_unit_at_60000_only states_from_the_new_segments=4 red_under_todays_overlay_candidates=0 red_names={} red_under_current_unit_start_candidates=0 must_be_nonzero=0
E7RESULT name=r2_f1_shifted world=node_at_60001_then_data_unit_at_60000 states_from_the_new_segments=5 red_under_todays_overlay_candidates=2 red_names={"checker:I-7.8": 2} red_under_current_unit_start_candidates=0 must_be_nonzero=2
```

没有先写那个节点时 0 红；先写了就 2 红（数据单元落了的那两个状态）。按「这一槽上最后一次持久了的单元写从这一槽开头」认候选，两形都是 0。

- 可达性：「节点被释放、它的槽被一个跨两槽的数据单元从前一槽起复用」是分配器的正常行为（C561 讲的就是「两个 16K 节点 → 一个 32K 数据单元」这种不对齐复用）；这里的两次写是我手摆的，不是从真实现录的（真实现在短流里不复用，r2-b5-legal-reuse 也是手摆的）。
- 四句：①分辨臂——分：今天的叠加候选与「写表里有过的单元写起点」一读法中，「当前单元起点」不中；岔路单 B4 第三个候选「读集键 + 扫描按单元边界认槽」要定成后者才不中。②看得到——看得到，写表里后写盖过了那一槽。③分句——C587 题面「不判它是不是落在另一个单元中间」（`.claude/kb/checks-owed.md:467`）。④改法——候选按当前单元起点认（量过：0）；它只修层 0 这一侧，真盘上没有写表、候选是 None，那一形仍要 checker 自己认单元边界，归 C587。

## B5 记录核对器：归属与剪枝键

### 按记录自己的（实例，txg）归发布（世界 r2-b5，没打中）

我在原型里另写了一份第一条判据：一条根在盘上、写表里带它身份的记录至少一条、而一条都不在盘上就红；记录身份取记录字节偏移 16 的实例代号与偏移 26 的 txg。

| 历史 | 发布 | 两种归属不同的发布 | 今天第一条判据红 | 按自己身份归属红 | 剪枝键类数（第一版 / 符号版） | 同键异判 |
|---|---|---|---|---|---|---|
| 健康第一条流 | 3 | 0 | 0 | 0 | 6 / 13 | 0 / 0 |
| 健康第一条流 + txg 4 | 4 | 0 | 0 | 0 | 8 / 17 | 0 / 0 |
| 健康十次覆盖写 | 13 | 0 | 0 | 0 | 26 / 53 | 0 / 0 |
| 先根后记录，最后一次发布 | 3 | 1 | 0 | 1 | 5 / 13 | 0 / 0 |
| 先根后记录，后面还有 txg 4 | 4 | 2 | 0 | 1 | 7 / 17 | 0 / 0 |
| 提前复用 txg 3 的数据单元 | 4 | 0 | 0 | 0 | 11 / 20 | 0 / 0 |

（出处：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out` 第 2–12 行的双数行。）后面还有 txg 4 那一形，今天的归法把 txg 3 的两条记录归给了 txg 4：「root67(1, 4):order=[37, 38, 65, 66]/own=[65, 66]」（`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out:10`）。

- 两形 F2 各有 1 个状态（根落了、它自己两条记录都没落）按自己身份归属转红；三条健康流 0 假红。按岔路单 B5 行的翻面观测，这一轮我没造出让「按自己的 txg 归」不红或假红的历史。
- 没造的一形（推的）：同一个根身份在写表里出现两次。代码自己写了崩溃注入第二、三截会有：「同一个根身份在表里出现两次时（第二、三截里历史截断处之后那一截与挂载撞了身份），挂载的接在后面，取到的是盘上那一条。」（`crates/singlefs-checker-tier/src/crash.rs:170`）。按身份归属时两次发布的记录会并成一组，任一次的记录在盘上都能让另一次的「一条都不在」不成立；层 0 一条录制流里身份不重，所以只碍崩溃注入那两截。要造得录一条「历史截断 + 可写挂载重建同一身份」的流，没做。

### 记录核对器的剪枝键（世界 r2-b5-legal-reuse：第一版打中，符号版没打中）

第一轮判决第六节要攻的第 ②条「记录核对器在 2¹³⁶ 级的段上怎么精确剪（它要持久集合）」。第一轮 p6 的单元段上，两版键都是 1 类（`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out:14` 的「record_key_classes=1」与「symbolic_key_classes=1」）。攻它要一段新单元写盖到**更早发布、仍被核**的副本上：第一条流 + 30 次覆盖写之后另起一段，12 次写盖掉 txg 3 那次发布的前 12 份单元副本。30 次之后根环里最旧的根早过了 txg 3，这 12 次写都过得了回收谓词（`later_writes_passing_the_reclaim_predicate=12`），是合法复用。

```
E7RESULT name=r2_b5_legal_reuse overwrites=30 reused_copies_of_txg3=12 later_writes_passing_the_reclaim_predicate=12 states=4095 distinct_record_verdicts=1 key_with_every_overlapping_later_write_classes=4095 inconsistent=0 key_with_only_writes_failing_the_predicate_classes=64 inconsistent=0 symbolic_key_classes=1 symbolic_inconsistent=0 seconds=84.5 must_be_nonzero=4094
```

- 第一版键（读集 + 每份被核副本上全部更晚重叠写的持久位）：4095 个状态、记录核对器判定 1 种，键 4095 类，一个都省不下。
- 只留「过不了谓词的后写」的持久位、但还带读集：64 类。多出来的 63 类是读集里被核副本那几个扇区的内容（落没落那次合法复用，字节不同）；每对镜像副本里核对器只读第一份就短路，「.any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))」（`crates/singlefs-checker-tier/src/crash.rs:157`），所以是 2⁶ 不是 2¹²。
- 符号版（不读单元副本内容，只带恢复落到的根、每条根槽写与记录写在不在盘上、过不了谓词的后写落没落）：1 类，同键异判 0；上表七个域也都是 0。

四句（打第一版）：①分辨臂——分：第一版与第二版中，符号版不中。②看得到——看得到，谓词只看写表（`reuse_is_not_proven_illegal_by_the_reclaim_predicate`，`crates/singlefs-checker-tier/src/crash.rs:293` 起）。③分句——第一轮岔路单 R 行的翻面观测「键数接近状态数（复用无收益）」（`research/prompts/m3-prune-gpu-r1-forks.md:20`），第二轮岔路单没有 R 行，归 B5。④改法——符号版，量过（上一段），被攻过零轮；它的精确性靠「过得了谓词的后写落不落都解释得了那个扇区」，这一句是按 `unit_copy_is_missing_under_the_persisted_set` 的代码推的，只在这八个域上没反例。

## B6 内容身份（世界 r2-b6，没打中）

第一轮 collide 那一对 16 KiB 内容重新撞出来，换成 SHA-256：

```
E7RESULT name=r2_b6_collide_regression digest128_equal=true bytes_equal=false sha256_first=c954b0ff3c128c50 sha256_second=2a3ad2e1b9146ad9 sha256_equal=false seconds=28.5 must_be_nonzero=-
```

岔路单 B6 行的够判条件要「collide 世界端到端（连 CRC 一起拉平）跑一次」（`research/prompts/m3-prune-gpu-r2-forks.md:14`）：在 SHA-256 下连第一步（摘要相同）都走不到，端到端那一步就不用拉 CRC 了；这是按 SHA-256 的抗碰撞性推的，没去找 SHA-256 的碰撞。128 位摘要那一臂的端到端没做（第一轮已判它出局的附条件）。我也找过「内容相同而判定不同」的形：E161 的单元级检查只吃内容（`replay_unit_check(bytes)`），真 checker 的走读还拿父指针里的期望比，所以「内容键」要带上它被谁、按什么期望读——这一条推的，没造世界。

## B7 判定向量

### 计数格不是节点的函数（世界 r2-b7 第一部分，打中）

第一条流（路径 a）与「第一条流 + txg 4」（路径 b）前 11 段逐字节相同。计数按发布分格，段归哪次发布看它后面第一条根槽写（`publish_of_each_segment`）：

```
E7RESULT name=r2_b7_publish_cell shared_segments=11 segments_whose_publish_cell_differs=["10:after_the_last_root->instance1_txg4:8"] states_whose_tally_cell_is_not_a_function_of_the_node=8 must_be_nonzero=8
```

段 10 的 8 个状态，判定逐项相同（第一轮 small.out:3 已量），计数格不同。

四句：①分辨臂——分：「节点的判定向量共用、计数从向量重算」中；「向量共用、计数格按路径另存一张段 → 发布的表再重算」不中。②看得到——看得到，归属看路径上之后的根。③分句——摸底 Q2「今天 `Layer0Tally` 的各类计数（丁 5 第 6 行）能不能全部从逐状态向量重算出来」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:178`）：不能单靠向量。④改法——每条路径存一张「段 → 发布」表（每段几个字节），计数 = 向量 × 这张表；向量里不放发布标签（放了就共享不了）。推的，没实现。

### 编号宽（世界 r2-b7 第二部分，打中）

第一条流 + 150 次覆盖写，小段 1845 个状态。向量不带恢复落到的根时 7 种；带上（事后要答「这个状态恢复到了哪一版」就得带）时 307 种，逐个登记进真 `VerdictStore`，第 257 种被拒：

```
E7RESULT name=r2_b7_distinct_vectors overwrites=150 states=1845 distinct_status_vectors=7 distinct_vectors_with_landed_roots=307 real_store_refused_at=Some("257:MoreThan256VerdictVectorsNotDecidedInTheFirstVersion") build_seconds=0.4 seconds=173.6 must_be_nonzero=51
```

拒登记是响的（「return Err(VerdictStoreError::MoreThan256VerdictVectorsNotDecidedInTheFirstVersion);」，`crates/singlefs-checker-tier/src/verdict_store.rs:508`），不会读错，但这条流一跑到第 257 种就停。四句：①分辨臂——分：1 字节编号中，2 字节编号（65536 种）在这条流上不中；「向量不带根」也不中但答不了那个问题。②看得到——看得到。③分句——岔路单 B7 行「某条流上不同判定向量超过编号宽」（`research/prompts/m3-prune-gpu-r2-forks.md:15`）。④改法——向量只放按项红绿（7 种），「恢复到了哪一版」另存一张按（节点，段内序号）查的小表或现算；两种都推的。

## B8 版本号：「定义改了而旧数据被读错」（世界 r2-b8，打中）

判定来源：p5 改坏（96 个状态）与提前复用（63 个状态），共 159 个。旧定义一字节四位（看 journal 的 oracle 红、不看 journal 的 oracle 红、池级 checker 任一条红、记录核对器任一条红）。两种改定义：拆字段（记录那一位拆成两位，字段名表跟着变）；同名改义（字段名不变，「checker 红」不再含 I-7.8、「记录红」只剩「声称的单元缺席」）。每种放法都在真 `VerdictStore` 上先按旧定义写、再按新定义读，数读错的状态：

| 放法 | 拆字段 | 同名改义 |
|---|---|---|
| 今天的库（无版本） | 读错 4 | 读错 16 |
| 库级布局版本（开库时核） | 读错 4 | 读错 16 |
| 定义版本进块键前缀，改的人抬了 | 0（旧块找不到，重算） | 0 |
| 同上，改的人忘了抬 | 读错 4 | 读错 16 |
| 字段名表的哈希进块键前缀（自动算） | 0 | 读错 16 |
| 金丝雀：每第 8 个状态，开库重判 | 读错 4（没拦住） | 读错 16（没拦住） |
| 金丝雀：每种向量第一次出现的那个状态 | 拒开 | 拒开 |
| 金丝雀：每种向量最后一次出现的那个状态 | 拒开 | 读错 16（没拦住） |
| 存逐项原始判定、读的一方现算 | 0 | 0 |

（出处：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b8.out` 第 4–21 行，逐行一格，`states_read_with_the_wrong_meaning=` 与 `refused_at_open=`。）

- 库级布局版本拦不住：编码是调用方的，「一个状态的判定向量，按调用方定的编码存成字节（这个模块不解释它）；两份字节相同就是同一个判定向量。」（`crates/singlefs-checker-tier/src/verdict_store.rs:101`），改编码不动库的布局。
- 金丝雀拦不拦得住看取哪几个状态：同名改义只改了「只有 I-7.8 红」那 16 个状态的读法，这 16 个与「I-2.1 等红」的状态存成同一个向量；取第一次出现的代表恰好是前者（拒开），取最后一次的是后者（放过）。
- 「抬了版本」那一格靠人记得抬；忘了抬的那一格与无版本一样。能两种改法都不读错、也不靠人的，只有「存逐项原始判定」：存的是不变量名字与它的红绿，读的一方要什么位自己算。剩下的风险是不变量本身的判法改了（同一个名字判得不一样）——那是 checker 代码改了，B2 的判法摘要跟着变，旧块按新指纹找不到。这最后一句是推的，没造「判法改了、指纹没变」的世界。

四句：①分辨臂——分（见表）。②看得到——读的一方看得到自己的定义，看不到写的一方的，除非库里存了。③分句——岔路单 B8 行「某种放法下改了一种身份的定义而旧数据被读成新定义」（`research/prompts/m3-prune-gpu-r2-forks.md:16`），够判条件「改定义、不改版本」与「改定义、改版本」各跑了一遍。④改法——存逐项原始判定（量过：两种改法都 0），被攻过零轮；它不比按项红绿多出种数（名字 + 字母与按项红绿一一对应，r2-b7 那条 150 次覆盖写的长流上 7 种；全部节点上有几种没量，上界是逐项组合数，推的）。

## D1 两台机器录入之后与单机逐项相同

四个世界都在真 `VerdictStore`（副本里开 `verdict-store` 特性编的）上跑，判定来源同 B8 的 159 个状态，判定向量取按项红绿的原文字节；两台按块号交错分（块号模 2），同层 0 分片「序号模 n 等于 i」的分法。

### 各写一份、按块原样导入（打中）

编号是每个库自己按登记次序给的：「判定向量的编号：同一个库里一种判定向量一个编号，从 0 起按登记次序给，登记过就不变（关库重开也不变）。」（`crates/singlefs-checker-tier/src/verdict_store.rs:105`）。两台各见到的第一个向量不同，同一个编号在两个库里指不同的向量。把 B 的块原样存进 A：

```
E7RESULT name=r2_d1_import mode=copy_block_bytes_as_they_are states=159 blocks_a=5 blocks_b=5 imported_silently=5 refused_loudly=0 merged_states_decoded_differently_from_single_machine=79 merged_states_missing=0 must_be_nonzero=79
E7RESULT name=r2_d1_import mode=translate_numbers_by_content states=159 blocks_a=5 blocks_b=5 imported_silently=5 refused_loudly=0 merged_states_decoded_differently_from_single_machine=0 merged_states_missing=0 must_be_nonzero=-
```

5 块全被收下，合完 159 个状态里 79 个解出来的向量与单机不同；按向量内容在 A 里重新登记、换编号再存，0 个。库自己的核对只看编号在不在范围内：「if usize::from(stored_number) < self.verdict_vector_by_number.len() {」（`crates/singlefs-checker-tier/src/verdict_store.rs:617`），A 登记的种数不少于 B 时它拦不住；`VerdictVectorNumber` 不带「哪个库」，类型上也拦不住把 B 读出的块交给 A 存。

四句：①分辨臂——分：岔路单 D1 的「各写一份，跑完按块导入」照原样导入中，换编号导入不中；「一个库，第二台经网络送块」在这一格上不中（只有一张编号表），推的，没造网络送块。②看得到——看得到，B 的编号表就在 B 库里。③分句——岔路单 D1 行「合完与单机不逐项相同」（`research/prompts/m3-prune-gpu-r2-forks.md:23`）。④改法——导入按内容换编号（量过：0）；再加一条会红的检查：合完之后按块把解出来的向量与单机逐个比（这个世界就是它的原型）。被攻过零轮。

### 被杀之后换块长续跑（打中）

块长不进键（`crates/singlefs-checker-tier/src/verdict_store.rs:41` 那一行）。第一趟按 16 个状态一块存了 3 块（到状态 48）后被杀；续跑的一趟块长换成 24（比如片长跟着别的参数变了），从状态 40 接着存：

```
E7RESULT name=r2_d1_resume_with_another_block_length states=159 first_run_blocks_of=16x3 resumed_from=40 resumed_block_states=24 resumed_blocks_accepted=5 refused=0 states_covered_by_two_blocks=8 states_not_covered=0 states_counted_by_a_tally_over_stored_blocks=167 must_be_nonzero=8
```

续跑的 5 块全被收下（块起点都与已存的不同，「同块键异内容报错」碰不到），状态 40–47 被两块罩住，按块求和的计数是 167 而不是 159。若续跑从一块的起点接着存（起点相同、块长不同），库会报错不写——那一形是响的，这一形是哑的。

四句：①分辨臂——分：块长固定、或块 = 节点、或块键带块长，都不中。②看得到——看得到，已存块的值长就是块长。③分句——D1 行「某台被杀、重跑、换线程数之后块重或漏」（`research/prompts/m3-prune-gpu-r2-forks.md:23`）的「重」。④改法——块 = 一个节点（或节点内固定长度、起点对齐），存块前查「与这个节点已存的块有没有重叠」，重叠就拒；推的，没实现。

### 块键里的计划哈希随跑法变（打中）

KV 第一段说块键里放「枚举计划哈希（`crash` 模块现算的计划哈希，SHA-256）。」（`crates/singlefs-checker-tier/src/verdict_store.rs:37`）。层 0 今天的计划哈希把切片、观察者、分片号都算进去：「for slice in slices {」（`crates/singlefs-checker-tier/src/crash.rs:2354`）、「message.push(u8::from(has_observer));」（`crates/singlefs-checker-tier/src/crash.rs:2358`）、「append_length_prefixed(&mut message, format!("shard {}", shard.text()).as_bytes());」（`crates/singlefs-checker-tier/src/crash.rs:2360`）。第一条流全量展开，七种跑法：

```
E7RESULT name=r2_d1_plan_hash runs=7 distinct_plan_hashes=6 detail=[("single_machine_4_threads", "2372fb821278cbc2", 16777260, 64), ("single_machine_32_threads", "3028bf0b08d1715b", 16777260, 512), ("resumable_4_threads", "c80d5f802803240b", 16777260, 65282), ("resumable_32_threads", "c80d5f802803240b", 16777260, 65282), ("shard_0_of_2", "e0f160a0760e8911", 16777260, 65282), ("shard_1_of_2", "78aacde265f84f23", 16777260, 65282), ("resumable_with_observer", "b7658f593421901d", 16777260, 65282)] must_be_nonzero=5
```

不续跑时片长随线程数定，4 线程与 32 线程两种哈希；续跑时片长与线程数无关，两者相同；两台分片各一种；带观察者又一种。把这份哈希放进块键，两台的块与单机的块键不同，「合完与单机逐块比」按键一块都对不上；单机换个线程数重跑，也找不到上一趟的块（不读错，但一块都复用不了）。四句：①分辨臂——分：块键只放「节点的写与每次写取几态」的哈希（不放切片、分片、观察者）不中。②看得到——看得到。③分句——同上一格的「换线程数」与「合完与单机不逐项相同」。④改法——块键的计划哈希另起一份，只罩判定的输入；推的。

### 续跑的片跨节点（打中）

第一条流全量 16777260 个状态，续跑按 65536 片切（「const LAYER0_RESUMABLE_SLICE_COUNT: u64 = 65_536;」，`crates/singlefs-checker-tier/src/crash.rs:2029`），每片 257 个状态：

```
E7RESULT name=r2_d1_slices_across_nodes stream=first_stream segments=11 states=16777260 states_per_slice=257 slices=65282 slices_spanning_two_or_more_segments=2 segments_not_starting_on_a_slice_boundary=10 must_be_nonzero=2
```

2 片跨段，11 段里 10 段不从片边界开始。块若取片，一块里有两个节点的状态，块键只能带一个节点的指纹——带前一个，后一个节点的输入变了块键不变（旧块被复用、读错）；带整条流的，任何一个节点变了整条流重跑（B3 ignore 那一格的同一形）。第二条流我量过一次是 78 段里 18 片跨段，但那一次调了 E161 的 `prepare_second_stream`，它经 `common::build_pool` 在临时目录建了两份稀疏镜像文件（跑完 Drop 删掉了），违反定义 3b，复跑命令里已去掉，这个数不作数。四句：①分辨臂——分：块 = 节点不中。②看得到——看得到。③分句——岔路单 B9 行「某种单位下单条线或单点要读的块跨了节点」（`research/prompts/m3-prune-gpu-r2-forks.md:17`）；B9 不在我的攻击面，这一格是 D1「块边界落在节点中间」那一问的数。④改法——同 B3 ignore 那一格。

### 收尾时收到的两条用户新定案与这几格的关系

写完 D1 这几格之后（报告收尾、等最后一次复跑核对时）主 agent 转来两条用户新定案：①分工不按节点号奇偶，按显卡性能或显存分（各卡领块，一批装多少按显存）；②流水线是「录入 → 按批从 KV 读出、GPU 一次算完写回（批次间隔要小）→ 门禁最后一次读违例」，没有 GPU 时录完再由 CPU 核对。没改攻击面，也没重跑；按定义推它们与上面几格的关系（推的）：

- 按块原样导入那一格与分法无关：只要每台各有一个库、按自己见到的次序登记向量，编号就对不上；按显存分只改「谁领哪几块」。
- 换块长续跑那一格在新定案下更容易走到：一批装多少按显存定，换一张卡或显存占用变了，块长就变；块长不进键时照样哑着重叠。
- 计划哈希那一格：分片号今天进哈希，按卡分时「第几片」换成「哪张卡领了哪些块」，只要它进块键，合完与单机就对不上。
- 流水线里「按批读出、算完写回」是一次读旧数据：B8 那张表的读错发生在这一步（写回的一方与录入的一方定义不同时）。

### D1 没打中与没做的

- 某台被杀时库本身丢不丢、坏不坏块（WAL 同步写 + 一批原子写）：没做，那是 E162 的题，派发不让跑 E162。
- 「一个库，第二台经网络送块」：没造，只推了它在「按块原样导入」那一格上不中。
- 换线程数时判定本身变不变：层 0 的注释写的是「切法与线程数只影响跑得多快，不影响计数与」（`crates/singlefs-checker-tier/src/crash.rs:1332`）「第一处违例」，我没另造世界核它。

## 第一轮十个世界与 F1、F2 在最强版本下

| 世界 | 第一轮打中的是 | 最强版本下判什么 | 依据 |
|---|---|---|---|
| collide | 128 位摘要可被用户数据撞上 | B6 用 SHA-256：那一对摘要不同，键不撞 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b6.out:1` 的「sha256_equal=false」 |
| t4 | 父结束镜像相同、子节点枚举域 5 vs 3 | B2：变体 a、b 的路径写表差两次写，路径写表哈希必不同（按定义推的，没另跑）；B3：单点若从物化父镜像起算，5 个序号里 4 个对不上（打中，见 B3 节），序号按整条路径的写表定几态之后相同（推的） | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b3.out:1` 的「ordinals_mapping_to_different_landings=4」 |
| kinds | 种类串把「根槽第一次写」与「根环回卷」并成一个 | B1：33 次发布 5 个身份，同一身份里并着不同条件的 0 个 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-kinds.out:1` 的「identities_merging_conditions=0」 |
| small | G2（代表态位置表）6 格判错、P4 自适应读、U4 只差发布标签 | G2、P4 不在 B1–B7 里，没重跑；B4 读集键与 P7 键逐状态取时第一条流小段同键异判 0；U4 的发布标签落到 B7：8 个状态的计数格不是节点的函数（打中，见 B7 节） | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b4.out:8` 的「p7_key_inconsistent_status=0」；`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b7_150.out:1` |
| torn | 枚举器的撕裂约定与物理撕裂在 2 个状态上镜像不同 | 撕裂约定是枚举器代码的一部分，进 B2 的判法摘要；层 0 的计划哈希本来就对带撕裂镜像的写表算（「let enumerated_writes = writes_with_torn_images.writes.as_slice();」，`crates/singlefs-checker-tier/src/crash.rs:2643`）。判定层第一轮就是 0，没重跑 | 第一轮 torn.out:2 |
| p5 | 按论证跳过（P5）漏 16 个红 | B4 P7 键：16 个红状态单独成类，同键异判 0 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b4.out:4` 的「p7_key_inconsistent_full=0」 |
| p6 | 单元段上 P6 叶子数 = 状态数 | B4 P7 键：4095 → 4 类；B5 符号键：1 类 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b4.out:6` 的「p7_key_classes=4」；`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out:14` |
| interior | 物化形态下数据单元第二槽被当单元头，12 个假红 | 候选按「当前单元起点」认：物化那一形要从整条路径的写表取单元起点，推的，没在 interior 上重跑；叠加形态下的新一形打中（见 F1 节），按当前单元起点认是 0 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-f1-shifted.out:4` |
| reuse | 物化只交本节点写表时记录核对器 3 → 0 | B5 符号键：20 类，同键异判 0；B2 下游写：整条路径与只到本节点判定 0 个不同 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out:12` 的「symbolic_key_inconsistent=0」；`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b2-downstream.out:12` |
| order | 先根后记录，五样都不红 | B5 按自己身份归属：1 个状态转红 | `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out:8` 的「root_without_own_record=1」 |
| F1 最小复现 | 真盘、候选为 None 时整盘扫同样中招 | B4 的任何一版键都碰不到：真盘没有写表，扫描要自己认单元边界，归 C587（扫描候选槽头被伪造头骗过）；没跑那个 harness 用例 | `.claude/kb/checks-owed.md:467` |
| F2 | 先根后记录，层 0 不红 | 见「F2 的真实现」一节 | — |

## 几个改法各修哪一格

全部只在我的模型上量过或只是推的，**被攻过零轮**。「量过」一列贴的是副本上的原样输出片段。

| 改法 | 修的格 | 量过 / 推的 |
|---|---|---|
| 共享与复用只认路径写表哈希；B1 只当登记、覆盖报告、ignore 行里点名的标签，不当任何复用或共享的键 | B1 并错 | 推的（X、Z 写表不同，路径写表哈希必不同） |
| 同上 | B1 拆错 | 不修：拆错那一格正是路径写表哈希中的，它的代价（12 个状态省不下）要接受或另想 |
| 输入指纹加版本表的 SHA-256 | B2 版本表 | 推的 |
| 单点照节点登记表里「每次写取几态」那一列排号，不从物化的父镜像现算 | B3 单点 | 推的 |
| 状态身份（节点，段内序号）；块不跨节点；块键的计划哈希按节点算、不带切片分片观察者 | B3 ignore、D1 计划哈希、D1 片跨节点 | 推的 |
| 判定向量只放按项红绿，不带违例正文 | B4 正文 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b4-messages.out:2` 的「p7_key_inconsistent_status=0」 |
| P7 摘要里 I-7.7 只取全部载体的最大值（不分盘） | p6 多出的 3 类（省，不是精确） | 推的 |
| 候选槽按「这一槽上最后一次持久单元写从这里开头」认 | F1 叠加形态 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-f1-shifted.out:4` 的「red_under_current_unit_start_candidates=0」 |
| 记录按自己字节里的（实例，txg）归发布 | F2 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5.out:8` 的「root_without_own_record=1」；真实现那一形见「F2 的真实现」一节 |
| 记录核对器符号键 | B5 剪枝键第一版 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5-legal-reuse.out:2` 的「symbolic_key_classes=1 symbolic_inconsistent=0」 |
| 每条路径另存一张「段 → 发布」表，计数 = 向量 × 这张表 | B7 计数格 | 推的 |
| 判定向量不带恢复落到的根 | B7 编号宽 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b7_150.out:3` 的「distinct_status_vectors=7」；代价是「这个状态恢复到哪一版」要另存或现算 |
| 2 字节编号 | B7 编号宽 | 推的（307 < 65536） |
| 存逐项原始判定、读的一方现算 | B8 两种改定义 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b8.out:12` 与 `:21` 的「states_read_with_the_wrong_meaning=0」 |
| 定义版本进块键前缀 | B8 | 量过，但只在抬了版本时 0（`r2-b8.out` 第 6、15 行），忘抬时照读错（第 7、16 行）：对「忘了抬」这一格不起作用 |
| 字段名表哈希进键 | B8 | 量过，拆字段 0（第 8 行）、同名改义照读错（第 17 行）：对同名改义不起作用 |
| 金丝雀状态开库重判 | B8 | 量过，三种取法里只有「每种向量第一次出现的那个」两次都拦住（第 10、19 行），而那是这组数据上的巧合（最后一次出现的那一取法漏了，第 20 行）：不当改法交出去 |
| 按块导入时按向量内容换编号 | D1 按块导入 | 量过：`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-d1.out:4` 的「merged_states_decoded_differently_from_single_machine=0」 |
| 块 = 节点，存块前查与这个节点已存的块重不重叠，重叠就拒 | D1 换块长续跑 | 推的 |

## F2 的真实现（世界 r2-b5-implementation）

岔路单 B5 行的够判条件点名「真实现改成先根后记录的 45 个状态上报红 / 真洞 / 假红」（`research/prompts/m3-prune-gpu-r2-forks.md:13`）。同一份原型编两次：健康编译；第二份副本上打第一轮 F2 调查员那份补丁（`persist_publish_writes` 改成先根与系统配置、后记录，三条发布路径一起改）。真洞按持久位认（根这次写落了、按记录字节认出的它自己的记录一条都没落），与按盘上字节判「在不在」的两条判据是两条路。

```
E7RESULT name=r2_b5_implementation segments=["2xS", "2xJ", "1xR", "2xS", "2xJ", "1xR", "2xS", "24xU", "2xJ", "1xR", "2xS"] ownership=["root4(1, 1):order=[2, 3]/own=[2, 3]", "root9(1, 2):order=[7, 8]/own=[7, 8]", "root38(1, 3):order=[36, 37]/own=[36, 37]"] states=45 true_holes_by_persisted_bits=0 of_those_red_by_any_todays_judgment=0 root_without_record_today=0 root_without_own_record=0 own_red_that_is_not_a_true_hole=0 must_be_nonzero=0
E7RESULT name=r2_b5_implementation segments=["2xS", "1xR", "2xS", "2xJ", "1xR", "2xS", "2xJ", "24xU", "1xR", "2xS", "2xJ"] ownership=["root2(1, 1):order=[]/own=[5, 6]", "root7(1, 2):order=[5, 6]/own=[10, 11]", "root36(1, 3):order=[10, 11]/own=[39, 40]"] states=45 true_holes_by_persisted_bits=27 of_those_red_by_any_todays_judgment=0 root_without_record_today=0 root_without_own_record=27 own_red_that_is_not_a_true_hole=0 must_be_nonzero=27
```

（第一行 `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5-implementation.out:2`，第二行 `research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5-implementation-root-before-records.out:2`。）

- 27 个真洞与第一轮 F2 调查员数的一样；今天五样判定一个都不红（`must_be_nonzero=27` 数的就是这 27 个漏网的）；按自己身份归属的第一条判据 27 个全红，0 个假红。健康编译上三样都是 0。
- 今天的归法在改坏的编译上整体错开一位：「root7(1, 2):order=[5, 6]/own=[10, 11]」（`research/prompts/m3-prune-gpu-r2-opus-model/outputs/r2-b5-implementation-root-before-records.out:2`），txg 2 分到的是 txg 1 的记录。
- 所以 B5 的「按记录自己的 txg 归发布」在够判条件的三组数上都站住（F2 两种次序各 1 个、真实现 27 个、健康 0 假红）；它没碰的是「同一身份出现两次」那一形（崩溃注入第二、三截），见 B5 节。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| B1 覆盖签名下同一身份并着不同条件 | 第一条流 + 30 次覆盖写，33 次发布 | 0（r2-kinds） |
| B2 节点判定依赖之后的写 | 六条历史 300 个状态（见 B2 节表） | 0 |
| B4 P7 键同键异判（按项红绿与整份摘要） | p5 对照与改坏、p6 单元段、两条流小段，共 4325 个状态 | 0（只在判定带违例正文时，r2-b4-messages 打中） |
| B5 按自己身份归属假红 | 三条健康流、提前复用、健康编译的真实现，共 375 个状态 | 0 |
| B5 符号键同键异判 | 七个域 + 合法复用段，共 8622 个状态 | 0 |
| B6 SHA-256 下同摘要异内容 | 第一轮 collide 那一对 | 摘要不同 |
| D1 按内容换编号导入之后与单机不同 | 159 个状态 | 0 |
| B3 线程数、分片之间同一状态的号 | 没造：层 0 今天的序号在续跑与分片时不随线程数变（计划哈希里续跑两种线程数相同，r2-d1.out:6） | — |

没造的形状，都推的：同一根身份在写表里出现两次（崩溃注入第二、三截，B5 按身份归属会并错）；有被抛弃发布的回退历史（B2 下游写那三处读法真正会碰到之后的写的地方）；第二条流上的一切（要在盘上建镜像文件）；一个库经网络送块；RocksDB 在杀进程下丢不丢块（E162 的题）。

## 这条腿自己的限度

- 副本上的数，**不算入库装置上的数**。十五个世界都是自己造的历史或自己手摆的写：F1 叠加那一形的节点与数据单元、合法复用的 12 次写、两个高实例代号节点、p5 的孤儿都是手摆的，不是从真实现录的；真实现录的只有第一条流、覆盖写、F2 的真实现改坏。
- 取样与定义 3b 冲突：凡有 24 写以上单元段的历史，那一段没展开，只以整段持久进入后面的状态；p6 类世界把单元段截成前 12 次写。打中的格里 B1、B2 版本表、B7、F1、D1 都落在展开了的小段或手摆的小段上，不受这个取舍影响；没打中的 B2 下游写没覆盖那些大段。
- 违反定义的两处，照实写：①第一次编译（没开特性那一次）没经 `run-with-memory-cap.sh` 包装，之后的编译都经了；②一次 `r2-d1` 调了 E161 的 `prepare_second_stream`，经 `common::build_pool` 在系统临时目录建了两份 4 GiB 的稀疏镜像文件（跑完 Drop 删掉了，事后查临时目录没有残留），违反定义 3b 的「不在磁盘上建镜像文件」；那一次量出的第二条流的数不作数，模型里已去掉。
- P7 键的类数是「每个状态真跑一遍再取键」量的：说明类有几个、同键异判几个，不说明 P6 能不能不逐个跑就把类分出来。
- 自己提的改法与「最强版本」里我加的部分，全部只在我的模型上量过或只是推的，被攻过零轮。B5 剪枝键的第一版与 B1 覆盖签名的第一版都被我自己的世界打穿过（r2-b5-legal-reuse、r2-kinds 的第一版），交出去的是收严之后的版本，那两次输照记在对应的节里。
- 时间：线程上限 4 照用；B7 那一个世界 150 次覆盖写跑了约 174 秒，最后一批十四个世界各自的挂钟加起来 316 秒，没超过 40 分钟的估时线，没缩。

## 没做什么

- 没跑任何 checker 档测试、名字带 layer0 的目标、门禁 54、55、57、59 号、E162、E163；只编、只跑 E161 装置的副本（经 `run-with-memory-cap.sh 8G` 与 `capped.sh 4`）。
- 没判正推、本地攻方那几格（B9、C1–C8 的设计与代价算术）；没替主 agent 采纳；主仓 `crates/` 与 kb 一个字没改，只写了这份报告与模型目录。
- 没造：同一根身份出现两次的崩溃注入写表（B5 按身份归属的反例候选）、有被抛弃发布的回退历史（B2 下游写）、第二条流上的任何世界、网络送块的一个库、杀进程下的 RocksDB、P6 按写符号求 P7 聚合的实现、B6 的「内容键要带读它的期望」那一形。
- 没在入库装置上重做任何一个数；B2 版本表、B3 单点、B7 计数格的改法都是推的，没实现。
- 本地攻方的代价算术（键长、类数上界、各表字节数、GPU 批）不在我的攻击面，没碰。

## 复跑核对与草稿

- 交回前按模型目录里的 `rerun.sh` 在一份新副本上从头复跑了一遍（两份编译、十六份输出）：十六份都是「与存档逐行相同（只剥掉 seconds= 与 build_seconds= 字段）」，日志在草稿目录 `/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-final.log`。上一版模型（十四个世界、没有真实现那一份）另复跑过一次，十四份同样逐行相同。
- 删了（都是我这一轮自己建的）：`/tmp/claude-1000/m3-prune-gpu-r2-opus/repo`（仓副本，338M）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/repo-f2`（先根后记录那份仓副本，338M）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/target`（编译目录，5.5G）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/target-f2`（编译目录，5.3G）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-check/repo`（339M）与 `/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-check/target`（5.3G）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-final/repo`（344M）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-final/repo-root-before-records`（344M）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-final/target-repo`（5.3G）、`/tmp/claude-1000/m3-prune-gpu-r2-opus/rerun-final/target-repo-root-before-records`（5.3G）。
- 草稿目录里留着的（共 512K，没有编译目录与仓副本）：各批日志、`out/`、两次复跑的 `out/`、`r2.rs` 与 `crates-probes.patch` 的草稿、`run-batch.sh`、`progress.md`、空的 `kv/`。
