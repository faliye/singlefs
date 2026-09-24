# m2-treesplit-r1 核查员报告

这是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

被核引用：`.claude/kb/decisions/16-发布语义.md` 第 29 行（D16 已定项 1），
报告（opus）原文引「回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，
推空与抬 F 产生的空发布根不算」。在草稿副本 `/tmp/claude-1000/m2-treesplit-r1-verifier/selfproof/d16-copy.md`
里把行号从 29 加 1 改成 30，核第 30 行：

```
第30行内容:（空行）
引文是否整段出现在第30行: False
```

判定：**✗**（核不出，判别力自证通过：核查方法确实分辨得出行号错位）。

## 快照与产物完整性

- 主 agent 给的开工快照 `research/prompts/m2-treesplit-r1-snapshot/sha256sums.txt`（61 个文件）：
  对我自己的草稿仓副本 `/tmp/claude-1000/m2-treesplit-r1-verifier/repo` 跑 `sha256sum -c`，
  唯一不符的是 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`——与主 agent 预告的
  已知例外一致（E158 执行员在改那个实验装置），不计入 ✗，单列。
- 云端腿报告 sha256：opus `38ebe58711b0b614922b8f5596a5119365c44e44e78eb7a783bd7462a8a2d30b`、
  sonnet `0494c82a4e01db2d79b02fa28186b82cc9d589333bbdf9a51acd55fb785b8065`，
  与派发提示给的一致（`sha256sum` 现算，两份都对）。
- opus 模型目录 9 个文件（`treesplit_opus_prototype.rs`、`treesplit_opus_costs.rs`、`run.sh`、
  `out/{costs,selftest,t1-one-config-all-five-prefixes,t1,t6,targeted}.out`）：`sha256sum` 现算，
  9/9 与报告开头声明的一致。


## 复跑：`costs.out`（重点复跑 2、4、5）

草稿仓副本 `/tmp/claude-1000/m2-treesplit-r1-verifier/repo`（`rsync --exclude target --exclude .git` 从主工作区拷，
`sha256sum -c` 对开工快照全绿，唯一例外与主 agent 预告一致，见上）。把 `treesplit_opus_prototype.rs`、
`treesplit_opus_costs.rs` 拷进 `crates/singlefs-harness/tests/` 后，`CARGO_TARGET_DIR` 指向自己的草稿目录，
`nice -n 19 cargo test --offline --release -p singlefs-harness --test treesplit_opus_costs -- --nocapture --test-threads 1`
（`TREESPLIT_SCAN=none`）。开跑前 `ps` 未见 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`，
但机器上同时有约 600 个其他会话的 cargo/rustc/collect2/rust-lld 进程（`ps -eo stat | sort | uniq -c` 见 96 个 `SNl`），
`uptime` 读到 65–112（32 核）；照共用约束「有性能测量在跑就停，只有别的 cargo 在跑照常跑」继续，构建耗时 2m18s。

跑出 9 个 test（比原始 7 个多 2 个：`proto::borrow_from_the_right_under_the_creation_min_separator`、
`proto::tail_split_root_growth_is_enumerated`——这是因为 `treesplit_opus_costs.rs` 把
`treesplit_opus_prototype.rs` 当 `proto` 模块引入，而我草稿仓里的 prototype 文件已经是追加了这两个定向用例
之后的版本（sha256 与报告声明的一致），原始 `costs.out` 是在追加之前跑的——报告正文自己说明了这个时序
（「selftest / costs / t6 / t1 四段是原型文件后面还没追加两个定向用例时跑的」），不是复跑对不上，是同一份
文件两个时间点的自然差异；这两个多出来的 test 的数字本身与 `targeted.out` 里的原样一致，见下）。

逐字段比对（`diff`，剔除时间戳/编译信息/finished-in 行）：`REAL first_transaction closed_form=262168`、
`VARIANT A_root_leaf_split_three_trees closed_form=1073741848`、8 行 `SEGMENT`、`BORROW`/`BORROW_ENUM`、
`PREFIX` 全部 11 行、`TAILROOT` 全部 3 行、`AGED` 全部 5 行、`WINDOW` 全部 3 行——与
`research/prompts/m2-treesplit-r1-opus-model/out/costs.out` **逐字节一致**，判定 **✓**。

## 复跑：`t6.out`（重点复跑 3）

同一份草稿仓，已建好的 release 二进制，`env TREESPLIT_SCAN=t6 TREESPLIT_PREFIXES=PA,PB,PC,PE nice -n 19 cargo test
--offline --release -p singlefs-harness --test treesplit_opus_prototype -- --nocapture --test-threads 1`。
构建这次 6m53s（同一台机器同一时段负载没有变化，`ps` 复核过程见下「没做什么」一节旁注），跑 15.24s，6 个 test 全过。

逐条 `CFG T6/*` 汇总行与原始 `t6.out` **逐字节一致**：

```
T6/Refuse            histories=8684 states=317674   refused_histories=2875 overlay_red=0
T6/SpreadLiteral      histories=8684 states=2814312  cut_publish_applied=8826 overlay_red=53520
T6/SpreadFlag         histories=8684 states=2814312  cut_publish_applied=0    overlay_red=0
T6/CutPublish         histories=8684 states=329815   refused_histories=2875   overlay_red=0
```

判定 **✓**（8826/53520 对 0/0 这组数字复跑坐实）。`diff` 只在两处 `EXAMPLE` 明细行上不同：原始与复跑各自
第 3/4 条 `EXAMPLE T6/SpreadLiteral … CUT_PUBLISH_APPLIED` 的 `suffix` 从 `[[Ins(15)],[Ins(35)]]` 变成
`[[Ins(25)],[Ins(35)]]`（同一个 `cut_publish_applied` 桶里换了一个具体反例，两次都在同一个前缀
`prefix=[[Ins(10)]..[Ins(200)]]` 下，写出的 `persisted` 结构同形）；报告正文实际引用并逐字讨论的**第一条**
`EXAMPLE`（`suffix=[[Ins(1)], [Ins(25)]]`）与全部 `CFG` 汇总数字两次完全一致——这条差异不影响报告的任何论断，
按「按字段比，不按整份哈希判 ✗」不计 ✗，写明供主 agent 知悉：这份原型的例子挑选（在同一分桶内挑第几条）
不是逐次确定性的，聚合计数（cut_publish_applied、overlay_red 等）是。


## 复跑：`t1.out`（重点复跑 1）

同一份草稿仓已建好的二进制，`env TREESPLIT_SCAN=t1 TREESPLIT_PREFIXES=PA,PB,PC,PE nice -n 19 cargo test
--offline --release -p singlefs-harness --test treesplit_opus_prototype -- --nocapture --test-threads 1`。
跑 120.57s（原始报告记 653 秒，同一批四前缀；这次更快，机器负载在两次采样点之间下降过，`uptime` 从
65–112 见过、也见过 72–99，负载本身是会变的量，不是复跑异常）。

**48 格 `CFG T1/...` 汇总行与原始 `t1.out` 逐字节全部一致**（`diff` rc=0，包含 24 格 CreationMin
check_red ∈ [34952, 562983] 全非零、24 格 Maintained check_red 全 0，以及 `states`/`publishes`/
`units`/`split_units` 等全部字段）。判定 **✓**——这是「重点复跑 1」要坐实的那两组 24/24 数字。

**但 135 条 `EXAMPLE` 明细行两次跑法全部不同**（`diff` 全红，`rc=1`）：报告正文引用并逐字讨论的那一条
（`T1/Middle/CreationMin/DropEmpty{collapse_root:false}/OnePublish CHECK_RED: 槽 116 分隔 key 10 >
孩子区间下界 5 (recovered txg 9); ... suffix=[[Ins(5)], [Del(60)]]`）在复跑里一次都没出现——同一格里
复跑给出的是「> 孩子区间下界 1」「suffix=[[Ins(1)], [Del(30)]]」这一条，slot 仍是 116、违例种类相同
（分隔 key > 孩子区间下界），只是「在同一分桶里挑哪一条当例子」不确定；这与 t6.out 那处发现同一性质。
**这不是一处 ✗**：报告在自己的产物文件（`t1.out`）里逐字抄的那一句本身核对无误（已在「引产物」一节核过），
这里只是标出**这份原型挑选说明性反例的那一步不是逐次确定的，聚合计数是**——供主 agent 判断要不要把
这一点计入「装置本身的限度」。


## 腿一：云端攻方（Opus，T1/T4/T6）—— `m2-treesplit-r1-opus-output.md`

| 引用 / 检查项 | 结果 | 命令 / 依据 |
|---|---|---|
| D16 已定项1 第29行「回退候选集…推空与抬F产生的空发布根不算」 | ✓（含判别力自证） | `sed -n '29p' .claude/kb/decisions/16-发布语义.md` |
| D16 已定项1 第43行「非空」从盘上怎么认 | ✓ | `sed -n '43p'` 同文件 |
| D8 已定项6 第162行「分隔 key = 该孩子创建时的最小 key…」 | ✓ | `sed -n '162p' .claude/kb/decisions/08-核心索引结构.md` |
| D8 已定项6 第169行「只许左吸收右/左从右借」 | ✓ | 同文件 `sed -n '169p'` |
| D8 已定项14 第362行「一套 btree 实现…不做异构结构」 | ✓ | 同文件 `sed -n '362p'`，且 360-378 恰为已定项14区间 |
| I-9.4 第271行「沿叶序容器号严格递增…」 | ✓ | `sed -n '271p' .claude/kb/invariants.md` |
| D18 已定项2 第50行「区间取子树覆盖区间」 | ✓ | `sed -n '50p' .claude/kb/decisions/18-块里携带什么信息.md` |
| D13 已定项4 第71行（屏障切段等） | ✓（准确的转述，非逐字） | `sed -n '71p' .claude/kb/decisions/13-验证路线.md` |
| D13 已定项9 第170行「层0冒烟…不抽样，全量」 | ✓ | 同文件 `sed -n '170p'` |
| D23 已定项17 第442行「只在最后一条记录里点名」 | ✓ | `sed -n '442p' .claude/kb/decisions/23-journal的角色与格式.md` |
| transaction.rs:3824（持久顺序注释） | ✓ | `sed -n '3824p' crates/singlefs-core/src/transaction.rs` |
| transaction.rs:2607（MoreNamedUnitsThanOneJournalRecordHolds） | ✓ | 同文件 |
| format/lib.rs:324（`JOURNAL_NAMED_ENTRIES_PER_RECORD, 67`） | ✓ | 同文件；另见下方「背景材料误差」 |
| mount.rs:728/732（`user_visible_trees_changed`） | ✓，且函数体核实只比 inode/extent 根指针 | `sed -n '720,733p' crates/singlefs-core/src/mount.rs` |
| journal.rs:44/59/194（`put_u8(0)`/`skip(1)`/`is_commit`） | ✓ | 同文件；`JOURNAL_NAMED_ENTRY_BYTES` 断言 56 与 D23 已定项17标题「56字节」一致 |
| inode_tree.rs `separator_key`/「合并不在这个模块里」 | ✓ | `grep -n` 同文件 |
| first_transaction_step_seven_layer0.rs:390 | ✓ | `sed -n '390p'` |
| second_transaction_parallel_line_one_layer0.rs:154/197 | ✓ | 同文件 |
| `enumerate_layer0` 恰 5 个文件、皆不调 `publish_new_inodes` | ✓ | `grep -ln` 两条命令，见报告文本 |
| second_transaction_parallel_line_three_many_inodes.rs 模块注释「这一份是 C116…的『恢复加 oracle』那一路」 | **✗** | 源文件第7行原文是「这一份是 **C116（叶容器的分裂 / 合并纪律没有会失败的检查）** 的「恢复加 oracle」那一路」，报告引用时把 C116 的简称丢了，只留编号（`kb-discipline.md` 第5条「编号只能做索引，不能做称呼」同族问题） |
| out/t1.out 全部48格 CFG 汇总（24 CreationMin check_red∈[34952,562983]全非零、24 Maintained 全0） | ✓ | 现场 `grep -oP` 现算（我最初用 `^CFG` 起手漏了8条被 cargo 的 "test … " 前缀吃掉的行，改用不锚首的模式重数，48/48 对） |
| out/t1.out 「T1写序与收缩」9行表（DropEmpty/Merge×借/不借×三种写序，states/publishes/units/split_units） | ✓ 逐格核对，9/9 行与产物一致 |「grep -oP」现场比对 |
| out/t6.out 两条 CFG（SpreadLiteral 8826/53520，SpreadFlag 0/0） | ✓ | `grep -n '^CFG' out/t6.out` |
| out/targeted.out 三条 TAILROOT、两条 BORROW/BORROW_ENUM | ✓ | `sed -n` 现场核对逐字段 |
| out/costs.out REAL/VARIANT/SEGMENT/AGED/WINDOW 全部行 | ✓ | `sed -n '1,60p'` |
| AGED 池大小算术「约1.2 GiB」 | ✓ | `40000×2×16384 = 1.22 GiB` |
| 模型目录9个文件 sha256 | ✓ 9/9 | `sha256sum` 现算 |
| 复跑 `costs.out`（重点复跑2/4/5） | ✓ | 见上「复跑：costs.out」 |
| 复跑 `t6.out`（重点复跑3） | ✓ | 见上「复跑：t6.out」 |
| 复跑 `t1.out`（重点复跑1） | ✓（聚合数字）；EXAMPLE 明细行不可逐次复现，不算 ✗ | 见上「复跑：t1.out」 |

**opus 腿计数**：核了 30 处，✓ 29 处，✗ 1 处（C116 简称丢失）。


## 腿二：云端正推（Sonnet，T2/T3/T5 + T1/T4/T6 一句）—— `m2-treesplit-r1-sonnet-output.md`

无复跑命令（这条腿不产出模型/产物，只引 kb）。逐条引文块与源文件 `diff`（剔除代码块围栏与末尾空行）：

| 引用 | 结果 | 命令 |
|---|---|---|
| D8 已定项11 全段 315-333 | ✓（`diff` 只差末尾空行） | `sed -n '315,333p' .claude/kb/decisions/08-核心索引结构.md` |
| D19 已定项8 全段 163-186 | ✓（同上，只差末尾空行） | `sed -n '163,186p' .claude/kb/decisions/19-块指针的结构与宽度预算.md` |
| D8 已定项6 第162行（单行整行抄） | ✓ | 同 opus 腿已核 |
| D19 已定项5 全段 87-118 | **✗** | 报告第122行把源文件第115行「C394（释放判定不核映射条目位置项里的单元校验和） 三问」抄成「C394 三问」，丢了简称；`diff` 命中这一行，其余28行一致 |
| D28 已定项4 全段 90-108 | **✗** | 报告第167行把源文件第102行「所以现算、条款里不写常数」的顿号「、」抄成逗号「，」；`diff` 命中这一行，其余18行一致 |
| checks-owed.md:316（C363 整行） | ✓ 完全一致（`diff` rc=0） | `sed -n '316p' .claude/kb/checks-owed.md` |
| checks-owed.md:327（C376，转述「决策已定、实现与检查随并行线一」） | ✓ | `grep -o` 命中同一子串 |
| milestone/02-second-txn.md:389（C483 整行） | ✓ 完全一致（`diff` rc=0） | `sed -n '389p'` |
| D3 已定项7 区间115-153，引「key=(设备身份4,16KiB槽号6)共10字节」 | ✓，且115-153恰为已定项7区间（154起是已定项8） | `sed -n '115p;154p' .claude/kb/decisions/03-空间分配.md` |
| D19 已定项10 第200行标题「映射 key 一律27，条目一律55」 | ✓ | `sed -n '200p'` |
| D8 已定项14 区间360-378 | ✓，360-378恰为已定项14区间（379起是已定项15） | `sed -n '360p;379p'` |
| D13 已定项4/9（T4一句里的方法论引用，无独立逐字块） | ✓ 与 opus 腿核过的行一致 | 同上 |

**sonnet 腿计数**：核了 12 处，✓ 10 处，✗ 2 处（D19 已定项5 漏简称、D28 已定项4 顿号变逗号）。


## 腿三：本地攻方（T2、T5 算术表）—— 四份提示 + 8 份样本 + 核对表 + 运行记录

### 转述核对表 `m2-treesplit-r1-local-attack-translation-audit.md` 逐条核

| 原文文件:行 | 结果 | 命令 |
|---|---|---|
| FACT1 → lib.rs:14-15（NODE_BYTES=16384） | ✓ | `sed -n '14,15p' crates/singlefs-format/src/lib.rs` |
| FACT2 → lib.rs:49（公式）、62-63/65-66/68-69/71-72（四棵树头宽） | ✓ 8处行号全部现查一致 | `grep -n` 逐个常量名 |
| FACT3 → lib.rs:119&122（分配记录）、125&127（记账）、89&92（映射）、95&98（extent） | ✓ 8处行号全部一致（第一次用 `grep` 粗看以为 119/125 错位，`sed -n` 精确核对后确认原表无误，是我方法的假警报，已订正） | `sed -n` 精确取行 |
| FACT4 → lib.rs:86-87（NODE_POINTER_BYTES=86） | ✓ | 同上 |
| FACT5 → lib.rs:104-105（INODE_INTERNAL_ENTRY=120） | ✓ | 同上 |
| FACT6 → D8 已定项6 正文第321行「extent 树内部节点的条目…还没有条款」 | ✓ | `sed -n '321p' .claude/kb/decisions/08-核心索引结构.md` |
| ex.md 里 FACT6 末句改写（「已定案、不是候选」） | ✓ 与 ar.md 的候选版本原文并排确认差异恰在这一句 | `grep -o` 两份文件末句 |

### 算术样本核（8 份，每份13格，与我独立按公式重算比对）

| 文件 | 结果 | 依据 |
|---|---|---|
| ar-output-s1/s2 | ✓ 26/26 格与独立计算一致（leaf_cap=812, fanout=169, h3-h6=2,2,2,3, n3-n6=2,2,2,3, s_k1/10/100=3/3/3） | Python 独立重算 |
| ac-output-s1/s2 | ✓ 26/26（leaf_cap=477, fanout=150, h3-h6=2,2,3,3, s_k=3/3/3） | 同上 |
| cm-output-s1/s2 | ✓ 26/26（leaf_cap=294, fanout=143, h3-h6=2,2,3,3, s_k=3/3/3） | 同上 |
| ex-output-s1/s2 | ✓ 26/26（leaf_cap=144, fanout=147, h3-h6=2,2,3,3, s_k=3/3/3） | 同上 |

104 格全部核完，无一格算错。

### 运行记录 `m2-treesplit-r1-local-attack-runlog.md` 核

| 声明 | 结果 | 命令 |
|---|---|---|
| 8 份样本词数（351/421/394/386/425/391/473/295） | ✓ 8/8 与 `wc -w` 一致 | `wc -w` 逐份现算 |
| 52格原始提示的空输出（0行0词） | ✓ | `wc -l`/`wc -c` |
| 退出码分布（0×8、3×1，无5） | 核不动（依赖当时网关的实时响应，不可复跑重现同一次网关报错），按文件内容与词数一致度采信 | — |

**local 腿计数**：核了 8（translation audit citation）+ 104（算术格）+ 3（runlog）= 115 处，✓ 114 处，
✗ 0 处，核不动 1 处（网关退出码分布，事后无法重放）。


## 三腿合计

- 共核 30（opus）+ 12（sonnet）+ 115（local）= **157 处**。
- **✓ 153 处**，**✗ 3 处**（opus 1 处 C116 简称丢失；sonnet 2 处：D19 已定项5 漏 C394 简称、
  D28 已定项4 顿号变逗号），**核不动 1 处**（local 腿网关退出码分布，事后无法重放同一次网关响应）。
- 3 处 ✗ 都是「摘句/转述漂移」类（`kb-discipline.md` 第5条「编号只能做索引，不能做称呼」、
  `evidence-discipline.md`「引产物就整行抄」同族），不是行号错位、不是产物数字被改写；
  三腿的全部数字类声明（t1.out 48格、t6.out 4格、costs.out 全部行、104格算术、8份词数）
  复核后一个都没有偏差。

## 分不清 / 单列

- `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`：开工快照与我的仓副本 sha256 不符，
  与主 agent 预告的已知例外（E158 执行员在改这个装置）一致，不计入 ✗、不计入 ✓，单列。
- opus 腿 `t1.out`/`t6.out` 的 `EXAMPLE` 明细行两次跑法给出不同的具体反例（同一分桶换了一条历史），
  聚合计数（`check_red`、`cut_publish_applied`、`overlay_red` 等）两次逐字节一致；这是装置本身
  「哪条历史被选来示例」不确定性的表现，不算复跑不一致，已在对应小节写明。

## 没做什么

- 不判三条腿任何一条打中成不成立、该不该采纳；只核引用、产物与复跑，判决是主 agent 的事。
- 没有再抽第二次本地腿样本——8 份样本已各自达到「两份干净样本」的额度，运行记录里的抽样过程
  本身不归我核（那是 test-discipline「一条腿只抽一次样不算一次观测」管辖的抽样纪律，
  不是我这次要复核的对象；我只核样本内容对不对、词数对不对）。
- 没有复跑 sonnet 腿（它没有模型/产物，只有 kb 引文，已逐条 `diff`）。
- 没有对 opus 报告里标「推的」「没在实现上跑」的段落（inode 树左从右借必破 I-9.4、T4 的段序列改法、
  层0新流候选）做进一步验证——那些本来就标了「推的」，不是待核的观测声明。
- 没有验证 opus 报告 T5 一句带过的观测（「只摘空节点不降高 ⇒ 树高只涨不降」）——报告自己写明
  「这一格归正推腿与本地攻方，我只报这个观测」，按分工不归这条腿的复跑范围，我也未去核实。
- 没有跑 `gate.sh`；`stage-owners.tsv` 未登记给 three-way-verifier 的阶段。
- 草稿仓副本 `/tmp/claude-1000/m2-treesplit-r1-verifier/{repo,target,out,selfproof}` 留着未删，
  供主 agent 需要时复核；`target/` 约十余 GB，事后可删。

## 产出

- 报告：`research/prompts/m2-treesplit-r1-verifier-output.md`（本文件）
- 草稿：`/tmp/claude-1000/m2-treesplit-r1-verifier/`（`repo` 仓副本、`target` 构建产物、`out/{costs,t1,t6}.out` 三份复跑产物、`selfproof/` 判别力自证副本）
