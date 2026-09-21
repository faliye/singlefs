# m2-s1-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

2026-09-19 落盘（JST）。轮名 `m2-s1-r1`，设计轮、无开工快照 ⇒ 对主树核（`crates/` 与 `.claude/kb/` 用工作区当前版本），行号对不上时先查是否为背景材料行号误标，查不出来源的记「分不清」。三条腿：云端攻方（Opus）、云端正推（Sonnet）、本地辩方（Qwen 本地）。

## 0. 判别力自证

取 opus 报告引用「`research/prompts/e155-preregistration.md` 第 846 行」这一条，在草稿副本里把行号加 1 改成 847 再核：

```
$ sed -n '847p' /tmp/claude-1000/m2-s1-r1-verifier/discriminative-test/e155-preregistration.md
（空行，847 行是空白，与 846 行的原句不符）
```

847 行是空行，不含被引的原文「**主几何**：φ = 1、K3、g = 24……」，按第 2 步判 **✗**。核查方法能分辨错误行号，往下正式核。

## 1. sha256 核对

```
$ sha256sum research/prompts/m2-s1-r1-opus-output.md research/prompts/m2-s1-r1-sonnet-output.md
2a57a28ed829a896345e358f6d4bee8106a87be0c72474225adaf0b282c94175  research/prompts/m2-s1-r1-opus-output.md
645d8f08bfc5aeeae5bce6b47cddd54bca23bdb6b78b95c93e1c5584ad1e61a1  research/prompts/m2-s1-r1-sonnet-output.md
```

两个都与主 agent 交回时给的 sha256 逐字相同。两份报告都是交回之后没有被改过的那一版，往下就核这一版。

## 2. 主 agent 已现查、请复核的两条 —— 都复核为真

**(a) 「前缀判定五条」应为六条**：

```
$ grep -n "前缀判定五条" research/prompts/_m2-s1-r1-body.md
23:| 七 | 重放的下界由所选根给出（前缀判定五条） | D23（journal 的角色与格式） 已定项 14 |
71:**问三，wal_full 与已定条款接不接得上。** ……前缀判定五条（D23（journal 的角色与格式） 已定项 14）……

$ sed -n '1221p' .claude/kb/decisions/23-journal的角色与格式.md
**前缀判定的完整口径是六条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引；第六条 2026-09-13 随 D16（发布语义） 已定项 4 用户定案加）：
```

`_m2-s1-r1-body.md`（同一句也进了 `_m2-s1-r1-background.md`）第 23、71 行写「五条」，D23 自己第 1221 行整行明写「六条，缺一不可」。**复核为真：正文第二节前提七与第五节问三都把六条写成了五条。** Sonnet 腿的 4.2 节第七行独立现查到了同一处（见下）。

**(b) E155 第 35 行「φ=0.5、g=0、K3′……比值都没变」与产物矛盾**：

```
$ sed -n '35p' .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md
- 8.2 反向几何敏感性只在代表格 P=10⁵、F1、seq、Lbalanced、N=16 上跑（不是全量 1800 格扫描）：φ=0.5、g=0、K3′ 三个反向点在这一格上甲 / wal_full 的字节比值都没变（`_ratio_changes=false`），pbs=4096 让甲的比值变了（`jia_ratio_changes=true`……）；K9′ 未建模，产物里那一行明写「未建」。

$ grep "point=phi_0.5" research/results/e155-fsync-write-volume-2026-09-17-stage4.out
E7RESULT name=geometry_sensitivity_sample point=phi_0.5 p=100000 base_jia=1037532 reverse_jia=1277958 base_write_ahead_log_full=303104 reverse_write_ahead_log_full=303104 base_write_ahead_log_leaf=172032 reverse_write_ahead_log_leaf=172032 jia_ratio_changes=true write_ahead_log_full_ratio_changes=false
```

**复核为真：`point=phi_0.5` 那一行 `jia_ratio_changes=true`，与第 35 行「都没变」矛盾**；`g_0`、`k3_prime` 两行现查确为 `false`，这两个没错，只有 φ=0.5 那一个被错分了组。Sonnet 腿 3.3 节独立发现了同一处、方向判断相同（不利于甲，不构成对倾向的反证）。


## 3. 云端攻方（Opus）腿

### 3.1 模型复跑

模型目录拷进 `/tmp/claude-1000/m2-s1-r1-verifier/opus-model-rerun/`（先核 `model.py`/`trace.py` 的 sha256 与报告一致）；`model.py` 用 `os.path.dirname(__file__)/../../../` 算 REPO 来读 `crates/` 的 5 个锚点，为了不在腿的原目录跑、又要让这个相对路径指到真仓，在草稿目录另建三层嵌套 `rerun-nest/a/b/c/` 放脚本、`rerun-nest/crates` 建符号链接指向 `/home/fy5090/code/singlefs/crates`（只读，不写）：

```
$ sha256sum /tmp/claude-1000/m2-s1-r1-verifier/rerun-nest/a/b/c/model.py /tmp/claude-1000/m2-s1-r1-verifier/rerun-nest/a/b/c/trace.py
ac260d3e9e7912b50c42f1a3469f3ceeb8771a765687ecf74a2d6c15e7ca953a  model.py   （与报告一致）
e7427e543eaae5026607f968b7592754370310c0a94a657fd36d9e07f4233629  trace.py  （与报告一致）

$ nice -n 19 env MAX_LEN=8 python3 model.py > run-len8.out
$ nice -n 19 env ALLOC=bump MAX_LEN=8 python3 model.py > run-len8-bump.out
$ nice -n 19 python3 trace.py > trace.out
$ sha256sum run-len8.out run-len8-bump.out trace.out
864d494764f981caf061c5b1a66cf562e737d6851ef697b92167b7a6e893ffac  run-len8.out        （与报告一致）
77c37603601ff8182de91c4f9d3cb0c25385e5854cf19cd9bd93aba9ac7ef523  run-len8-bump.out   （与报告一致）
ca89194822aa9d5cd6bcd43776cd8491234903fc0a448bb7ac191d158a250219  trace.out           （与报告一致）
```

三份产物三个 sha256 **逐字节复现**。模型启动时打印的锚点 `git hash-object`：

```
fact crates/singlefs-core/src/allocator.rs:397 hash-object=6544913c2b18e243b7dfa76bcb4702fb7c7a358f ...
fact crates/singlefs-core/src/recovery.rs:844  hash-object=7e89052cde0b877f564c828a87e6a14211c062cc ...
```

与工作区当前 `git hash-object crates/singlefs-core/src/allocator.rs`（`6544913c2b18e243b7dfa76bcb4702fb7c7a358f`）、`recovery.rs`（`7e89052cde0b877f564c828a87e6a14211c062cc`）逐字相同——模型确实读的是主树当前版本。报告正文里贴的 `cells per_region=8`、`versus_jia`、`wal_K9p_ckptunit`、`replay` 各行，逐条比对新产物，**全部逐字节一致**（已用 `grep` 分别取出比对，未见任何一处漂移）。

**判定：模型复跑 ✓，产物 ✓，锚点哈希 ✓。**

### 3.2 引用核对（文件:行号 + 抄的原文）

opus 报告里的「文件:行号 + 整行抄」共 32 处，逐条用 `sed -n 'N p'`／`grep -nF` 取主树当前内容比对，**全部 ✓**（无一处偏差）：

| # | 文件:行 | 核法 | 结果 |
|---|---|---|---|
| 1 | `e155-preregistration.md:846` | 主几何整行 | ✓ |
| 2 | `e155-preregistration.md:771` | K9 定义整行 | ✓ |
| 3 | `experiments/155-….md:1` | 标题（未建模自陈，转述非引文） | ✓ |
| 4 | `allocator.rs:397` `pub fn lowest_user_data_slot(` | ✓ | ✓ |
| 5 | `e155-preregistration.md:819` fsync 整行 | ✓ | ✓ |
| 6 | `e155-preregistration.md:820`（作为「checkpoint 那一条」引用，非直接摘句） | ✓ | ✓ |
| 7 | `D16:208` 长段（一版实付 2 次） | ✓ | ✓ |
| 8 | `D22:339` 单盘失效整行 | ✓ | ✓ |
| 9 | `recovery.rs:844`、`897-899` | 代码逐行 | ✓ |
| 10 | `D16:293` 长段（fsync 射程） | ✓ | ✓ |
| 11 | `D16:371` 可再分配表行 | ✓ | ✓ |
| 12 | `D03:180`（`grep -nF` 定位「释放」口径） | ✓ | ✓ |
| 13 | `D16:88`、`93-96` 新规则 2 与理由 2 | ✓ | ✓ |
| 14 | `D23:1227` 第六条整行 | ✓ | ✓ |
| 15 | `D23:694` 已定项 15 整行 | ✓ | ✓ |
| 16 | `recovery.rs:909` `tree_table: record.new_tree_table,` | ✓ | ✓ |
| 17 | `D16:195` C284 定案整行 | ✓ | ✓ |
| 18 | `D23:1042` 乙的第一条路整行 | ✓ | ✓ |
| 19 | `D23:1221`（六条 vs 五条，同第 2 节） | ✓ | ✓ |
| 20 | `checks-owed.md:265`（C284 存在性，未整段抄） | ✓ | ✓ |
| 21 | `D16:110` 已定项索引第 4 行整行 | ✓ | ✓ |
| 22 | `transaction.rs:1680,1637,846,2058` | ✓ | ✓ |
| 23 | `mount.rs:711,713` | ✓ | ✓ |
| 24 | `allocator.rs:601` `pub fn rebuild_from_records(` | ✓ | ✓ |
| 25 | `D19:153` 固定点收敛整行 | ✓ | ✓ |
| 26 | `e16_journal.rs:72,256-262,214,217` | ✓ | ✓ |
| 27 | `experiments/16-….md:220` | ✓ | ✓ |
| 28 | `D22:370` | ✓ | ✓ |
| 29 | `D23:9` 分配先于 journal | ✓ | ✓ |
| 30 | `D16:287` 持久顺序整行 | ✓ | ✓ |
| 31 | `D16:192` 已定项 9 整行 | ✓ | ✓ |
| 32 | `recovery.rs:852` 注释 | ✓ | ✓ |
| 33 | `experiments/16-….md:307-308` 乙的决定性成本 | ✓ | ✓ |
| 34 | 背景材料第 63、69 行（问二、`default_ring_ok` 限度自述，明确标「背景材料」而非 kb，非误标） | ✓ | ✓ |

**opus 腿本节小计：核了 34 处引用 + 1 次模型复跑（3 个产物文件 + 2 个源文件 + 2 个锚点哈希），全部 ✓，0 处 ✗，0 处核不动。**


## 4. 云端正推（Sonnet）腿

无模型、无产物目录（报告自陈未建），不适用第 4 步复跑；引用核对与产物核对如下。**这条腿的行号错误明显多于攻方腿，且有多处与「误写成背景材料行号」同形。**

### 4.1 ✗ 清单（12 处）

| # | 报告里的引用 | 实际情况 | 判定 |
|---|---|---|---|
| 1 | `D23:636-637`「甲下 redo 为空」 | 该文件真实位置是第 998 行；第 636-637 行是 **背景材料** `_m2-s1-r1-background.md` 的行号（`sed -n '636,638p' _m2-s1-r1-background.md` 命中同一句） | ✗ 误写成背景材料第 637 行，D23 原文件实为第 998 行 |
| 2 | `D13:367-401`「O2 的定义域是单个镜像」「任何需要第二个输入……都不属于它」 | 该文件第 367-401 行讲的是「已定项 5：只共享一份从 kb 生成的常量」，与 O2 定义域无关；真实位置是第 32-33 行 | ✗ 整段范围错，非背景材料行号（appendix 在 1165-1166、background 在 1729-1730，都不是 367-401） |
| 3 | `D16:876`「攒够一批脏节点……不是每个事务发一次」（H1 节「校验」步） | D16 真实位置第 3 行；876 与 **背景材料** 同一句所在行号相同（`_m2-s1-r1-background.md:876`） | ✗ 误写成背景材料第 876 行，D16 原文件实为第 3 行 |
| 4 | `D16:894-896`「没有任何操作可以绕过 journal 直接发根……」 | D16 真实位置第 21-22 行；894 与 **背景材料** 同一句所在行号相同（`_m2-s1-r1-background.md:894`） | ✗ 误写成背景材料第 894 行，D16 原文件实为第 21-22 行 |
| 5 | 4.2 节表格行「二」：「原句在文件 76 行」 | 同一句「攒够一批脏节点……」D16 真实位置第 3 行，不是 76 行；76 行是另一句（Merkle 根链粒度） | ✗ 与 #3 是同一句话，行号又错了一次，且与背景材料的 876 不一致（同一份报告内部两处引用同一句给出两个不同的错误行号） |
| 6 | `e155-preregistration.md:817`「次序：单元 → 屏障 → 记录 → 屏障。不写根槽、不写超级块槽」 | 第 817 行是空行；该句实际在第 819 行 | ✗ 行号错 2 行 |
| 7 | `e155-preregistration.md:820`「做完会怎样：……在飞记录在间隔里单调涨到……」 | 该句实际在第 821 行，820 行是 checkpoint 那一条 | ✗ 行号错 1 行 |
| 8 | K6「精确行号是 **184–185**，不是「185–186」」 | 实测 `crates/singlefs-format/src/lib.rs` 第 184 行是注释、第 185 行 `ROOT_RING_REGIONS`、第 186 行 `ROOT_RING_SLOTS_PER_REGION`——**背景材料原写的「185–186」才是对的**，Sonnet 这条「更正」本身是错的 | ✗ 反向引入错误：原文没错，腿把它改错了 |
| 9 | `.claude/kb/milestone/02-second-txn.md:2010`「改设计会碰到已经落地的东西：段序列登记表、层 0 两条流的状态数」 | 该 kb 文件全文只有 597 行，2010 行不存在；`grep -rn` 命中该句真实位置是该文件 **第 290 行**；2010 行恰是 **背景材料** `_m2-s1-r1-background.md:2010` 的行号 | ✗ 误写成背景材料第 2010 行，kb 原文件实为第 290 行 |
| 10 | wal_full 改动清单：「记录格式与 log-incompat 位」行写「D23 已定项 3」 | D23 已定项 3 讲的是「tail 住超级块槽，但改 jbd2 两处」（第 792 行起），与 log-incompat 无关；「记录类型 + 超级块 log-incompat 位」实际出自 **D16 已定项 3**（第 307-312 行附近，`grep` 命中第 312 行）与 D23 已定项 1 的 A 条 | ✗ 决策文件与条款号都指错 |
| 11 | 引 `m2-s1-prior-art-report.md`：加引号整句「读源码，没实测，未在本项目验证」 | 全文 `grep` 不到这个逐字句；实际是两处分开的近义句（第 8 行「三家都只读源码/文档，没有插桩……」、第 70 行「没有编译、没有跑任何一家的文件系统、没有插桩验证……」） | ✗ 加引号呈现为整行引文，实际是转述拼合，不存在这一整句 |
| 12 | H2 节：`grep -rn "环截断不了\|挂载期重放路径\|journal 进验证链\|屏障一个没省" .claude/kb/` 「命中只有 `23-journal的角色与格式.md` 这一处（`:990`）。全仓……没有任何地方对这四条各自展开过论证」 | 原样重跑同一条命令，**命中两处**：`23-journal的角色与格式.md:990` 与 `milestone/02-second-txn.md:289`——后者恰好就在 Sonnet 自己点名排除的「milestone 文档」里 | ✗ 命令结果与报告文字直接矛盾，重跑证伪 |


### 4.2 次要的 ✗（1 处）

| # | 引用 | 实际情况 | 判定 |
|---|---|---|---|
| 13 | 「甲维持不改」表：`first_transaction_step_seven_layer0.rs:89`、`second_transaction_step_zero_layer0.rs:357`、`second_transaction_step_three_formatted_pool_layer0.rs:93` 三处「现查同一句断言消息」 | 第一、三处逐字「记录核对器两条判据在已定的持久顺序下恒 0」；第二处（`:357`）实际多一段括注「（第二条在合法复用上的已知分歧除外）」，三处并不完全相同 | ✗（轻微）：不是「同一句」，是两句相同、一句多了限定语 |

### 4.3 核对为 ✓ 的部分

第二节前提表「行七」（五条应为六条，与主 agent 已核项同一处，见第 2 节）自己独立现查正确；第三节四文件哈希、`CommitStep` 枚举五种（起于 66 行、`Barrier,` 止于 87 行，闭合 `}` 实为 88 行——「66–87」按内容行读是精确的，未记为错）、`JournalRecord` 12 字段（81–94 行精确，含闭合行）、`replay_journal`（821–917 行精确，含闭合行）、`grep -c checkpoint` = 47、`grep T_dirty` = 0、发布 B `calls_and_bytes(21, 344_576)`（`second_transaction_supplement_one_write_accounting.rs:364,369`）、`stage4.out:4` 阳性对照行 `jia_fsync_bytes=344576`、H5 节 3.1 七行数据表逐字节比对、`write_ahead_log_full_in_flight_peak=17`／`jia_barriers=4`／`jia_fua=1`（P=1,F1,seq,n=16,Lbalanced 行）、`D16:287-288`／`300-301`、`D16:192`、D13 已定项 5「共享生成常量」段落抽查、D23 已定项 1 段落（`972-1115` 区间内 `grep -nF` 两处命中）、prior-art 报告第 64 行反证句、`crates/singlefs-core/src/system_configuration.rs:18,207-213`、`singlefs-checker/src/lib.rs:533`、`recovery.rs` 里 `grep "推导\|reconstruct"` 为 0、`log.incompat\|LOG_INCOMPAT` 与 `record_type` 两个 grep 为 0——**这些逐条现查全部 ✓**。

**Sonnet 腿本节小计：核了约 45 处引用与产物/命令，✓ 约 32 处，✗ 13 处（其中 4 处属于「误写成背景材料行号」、1 处是把本已正确的原文「更正」成了错的、1 处是加引号呈现不存在的整句、1 处是重跑证伪的命令结果断言）。**

## 5. 本地辩方（Qwen 本地）腿

无云端腿式的模型/复跑命令（本地模型的回答有随机性，不逐字复跑；`ask-local.sh` 的确定性部分——退出码、字数、损坏闸——现查如下）：

```
$ wc -w research/prompts/m2-s1-r1-local-defense-output-s1.md research/prompts/m2-s1-r1-local-defense-output-s2.md
 740 research/prompts/m2-s1-r1-local-defense-output-s1.md   （与运行记录一致）
 513 research/prompts/m2-s1-r1-local-defense-output-s2.md   （与运行记录一致）

$ wc -c /tmp/claude-1000/m2-s1-r1-local-defense/s1.err /tmp/claude-1000/m2-s1-r1-local-defense/s2.err
0 s1.err
0 s2.err   （运行记录称「两份均为空」，属实）
```

`ask-local.sh` 里 `corruption-check.py`／`oov-check.py` 双闸与退出码 5 的机制现查存在（`research/scripts/ask-local.sh` 第 83-113 行一带）。

### 5.1 逐句核对表（`m2-s1-r1-local-defense-translation-audit.md`）核对

对表中每一行「原文文件:行」逐条 `sed -n` 现查，**全部 ✓**：F1（D23:974）、F2（prereg:814-816）、F3 开头（prereg:818 + `_m2-s1-r1-background.md:69`，两处合看才对得上「最强形态」，核对表自己也这样说明）、F3 主体（prereg:819-820）、F3「不写根槽」括注依据（prereg:819 内嵌括注）、F4「30x or more」（`_axis2-background.md:39-40` 与内核源码注释，本轮不独立复核内核源码，跟腿自己的标注一致）、F5（D25:67,71，表格转 prose 属实）、F6（prereg:798、:767、:763，三处数字与「记账叶 477」全部命中）、F7（prereg:799，已用 replace-once 改过的版本核对为准确）、F8（prereg:800-801）、F9（prereg:838，139776 与 Table A/B 差值核对一致）、F10（D23:990，四个短语原样）、F11（`_axis2-background.md:14`，48B/10⁴–10⁵B/屏障两侧各 2 次全部命中，且注明未独立复核，属实告知）。Table A/B/C 的数字表也逐字段核对（`stage4.out` 对应行）：P=1/F1/seq/n=16 行 `jia_fsync_bytes=344576、jia_calls=21、jia_barriers=4、jia_fua=1` 与 Table A row A 完全一致；`write_ahead_log_full_*` 各 N 值字段与 Table B 完全一致。

**本地辩方腿小计：核了 17 处原文行 + 3 处数字表，全部 ✓，0 处 ✗。**


## 6. 汇总计数

| 腿 | 核了 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| 判别力自证 | 1 | 0（须为 ✗） | 1（正确判✗） | 0 |
| 主 agent 已现查两条复核 | 2 | 0 | 0（两条都复核为「真的问题」，非核查员打的✗，是确认既有发现） | 0 |
| 云端攻方 Opus（引用+复跑） | 34 引用 + 1 次复跑（3 产物+2 源码+2 哈希） | 34 引用全 ✓，复跑全 ✓ | 0 | 0 |
| 云端正推 Sonnet（引用+产物+命令） | 约 45 | 约 32 | 13（含 1 处次要） | 0 |
| 本地辩方（转述核对表+数字表） | 20 | 20 | 0 | 0 |
| **合计** | **约 102** | **约 86** | **13** | **0** |

## 7. 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑——第 2 节两条「复核为真」只是确认引用与产物之间确实存在文字矛盾，不代表核查员对 H4/H5 的论证效力做了判断。
- 没有重新推导 opus 报告里「四句」（分不分辨臂/判别子/满足哪个分句/改法还中不中）的正确性，那是主 agent 的评判范围（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）。
- 没有重新核对 Sonnet 报告第二节前提表里没被它自己抽查的十几行（六、八、九、十、十一、十二、十三、十五）——它自己在「没做什么」里也说明只抽查了承重最大的几行；本报告同样只核了 Sonnet 报告里明确写出行号或产物的那些处，未替它把没抽查的部分补全。
- 没有重新验证本地辩方 s1/s2 两份样本自己的算术推理对不对（比如 ratio(k) 的公式代入是否正确）——那是对推理本身的核验，不在本轮范围；本报告只核了它「转述原文」这一层。
- 没有对 opus 报告「H3 在什么情形下会被打中」「没打中的形状」等推的（非量的）部分做事实核对，因为那些段落本身已自陈是「推的」，不含可比对的「文件:行号+抄的原文」或产物字段。
- 未跑门禁、未编译 Rust、未起虚机；`ps` 核过没有性能测量类进程在跑（`qemu-system`/`vm-bench.sh`/`e152-*`/`fio` 均未命中），仅有另一会话此前的 cargo 编译已不在进程列表中。
- 未触碰 `crates/singlefs-harness/` 里实现员正在改的文件，未触碰 c381-r1 轮的任何文件。
- 草稿目录 `/tmp/claude-1000/m2-s1-r1-verifier/` 下 `discriminative-test/`、`opus-model-rerun/`、`rerun-nest/` 三处产物均为核查过程的临时副本，不建议入库，主 agent 需要复核细节时可直接去这三处看。

