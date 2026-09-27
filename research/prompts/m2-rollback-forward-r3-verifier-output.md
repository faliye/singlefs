# m2-rollback-forward-r3 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

抽取的引用：Opus 报告 Y1 节「四句」第 3 句里的 `invariants.md:276`（转述判据 I-9.6「水位大于两处最大号」，checker 红的那一条）。
在草稿目录副本 `/tmp/claude-1000/m2-rollback-forward-r3-verifier/selftest/invariants.md` 里把行号 276 改成 277，按第 2 步核：

```
$ sed -n '277p' /tmp/claude-1000/m2-rollback-forward-r3-verifier/selftest/invariants.md
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；……
```

277 行是 I-9.7（记录三段恒零），不是 I-9.6（水位大于两处最大号）。原引用（276 行）与 I-9.6 对应；加一之后的 277 行对不上，判 **✗**。核查方法能分辨。

## 输入清单核对

- 轮名：m2-rollback-forward-r3；腿报告三份齐（Opus、Sonnet、本地攻方两问各两样本）。
- 背景材料：`research/prompts/_m2-rollback-forward-r3-background.md`；另有本轮正文 `_m2-rollback-forward-r3-body.md`（Opus/本地攻方多处引它，非背景材料合成件，是这一轮自己的派工正文，按代码轮的「腿自己的工作笔记」对待，不算「误写成背景材料」那一类）。
- 云端腿交回 sha256：Opus `2dd78d23cd1141865887a40318542bd3b2dab814abece2dc99e957cde66c3606`，Sonnet `9f2189024c1eafceaced03405eae9d8042a2900b5514e50b43730a65bf28a730`；现算：

```
$ sha256sum research/prompts/m2-rollback-forward-r3-opus-output.md research/prompts/m2-rollback-forward-r3-sonnet-output.md
2dd78d23cd1141865887a40318542bd3b2dab814abece2dc99e957cde66c3606  research/prompts/m2-rollback-forward-r3-opus-output.md
9f2189024c1eafceaced03405eae9d8042a2900b5514e50b43730a65bf28a730  research/prompts/m2-rollback-forward-r3-sonnet-output.md
```

两份都对得上，交回之后没被改过。
- 快照：`research/prompts/m2-rollback-forward-r3-snapshot/{crates-sha256.txt,kb-sha256.txt}`，两份清单主 agent 已在主树上 `sha256sum -c` 过、全 OK；这一轮全部代码/kb 引用一律对 `/tmp/claude-1000/m2-rollback-forward-r3/tree/`、`/tmp/claude-1000/m2-rollback-forward-r3/kb-snapshot/` 核，不对主树核。

## 云端攻方（Opus）核对表

全部代码/kb 行号对快照现查（`/tmp/claude-1000/m2-rollback-forward-r3/tree/`、`/tmp/claude-1000/m2-rollback-forward-r3/kb-snapshot/`）。

| 引用（报告里的行:目标） | 抄的内容 | 结果 |
|---|---|---|
| 57:`_m2-rollback-forward-r3-body.md:13`（K1 式子整行） | K1 行原文 | ✓ 逐字相同 |
| 146:`_m2-rollback-forward-r3-body.md:14`（K2 式子整行） | K2 行原文 | ✓ 逐字相同 |
| 439:`_m2-rollback-forward-r3-body.md:19`（K7 式子整行） | K7 行原文 | ✓ 逐字相同 |
| 65:`transaction.rs:899`（根槽 FUA → 系统配置轮换的顺序段落） | 指向 `persist_the_root_then_rotate_the_system_configuration` 函数，899 行是其签名行，上方 895-898 的文档注释正写「根槽 FUA 写 → 系统配置槽轮换」 | ✓（指向一段落，落点在该段函数签名行，段落本身与所引顺序一致） |
| 100:`invariants.md:276`（I-9.6 判据） | 「水位大于两处最大号」整行 | ✓ 逐字相同（即自证那条） |
| 124/305/319:`invariants.md:54`（I-7.4，共 3 处） | 候选集判据整段 | ✓ 三处均逐字相同 |
| 385:`invariants.md:59`（I-7.9 判据） | 抬 F 上限判据整段 | ✓ 逐字相同 |
| 124:`allocator.rs:38`（分配代/释放代注释） | `    /// 仍分配时是分配代；已释放时是释放代。` | ✓ 逐字相同（含缩进） |
| 172:`mount.rs:1736`（中间实例注释） | `/// 中间实例 (i, 0, 0)；实例 0（mkfs）不写行……` | ✓ 逐字相同 |
| 83/146/319:`16-发布语义.md:38`（回退候选集，共 3 处） | `| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23 已定项 14 的候选集随之加这一条） |` | ✓ 三处均逐字相同 |
| 220/385:`16-发布语义.md:36`（抬 F 上限，共 2 处） | 「第 4 新的非空根」等 | ✓ 两处均对应 |
| 220:`16-发布语义.md:33`（可再分配） | 「环里最旧有效根」 | ✓ 对应（转述性引用） |
| 233:`16-发布语义.md:29`（定案「4 个不同状态」） | 转述，未称逐字 | ✓ 转述与原文一致，已标「转述」 |
| 392:`22-单元原子性怎么合成.md:136`（flags 字段表） | `| flags | 4 | 预留：第一版恒 0、非 0 拒收、位号表空（D22 已定项 17） |` | ✓ 逐字相同 |
| 392:`15-格式冻结政策.md:27`（动格式判据，转述） | 「改盘上字节或改已有字节的解释口径都算动格式」 | ✓ 转述与原文一致 |
| 392:`root_record.rs:46`（flags 写 0） | `        writer.put_u32(0); // flags：第一版恒 0、非 0 拒收` | ✓ 逐字相同（含 8 空格缩进） |
| 428:`.claude/rules/fs-design.md:198` | `> **一条格式分支值不值得，看它能不能把「用错了」变成「挂不上」。**` | ✓ 逐字相同 |
| 432:`system_configuration.rs:315`（FORMAT_VERSION 写入） | `            writer.put_u16(FORMAT_VERSION);` | ✓ 逐字相同（含 12 空格缩进） |
| 440:`system_configuration.rs:513`（incompat_bits_are_mountable，定位） | 函数名指向 | ✓ 函数确在该行 |
| 432:`unit.rs:300`（版本号核对，定位） | 「全仓核版本号的只有单元头」 | ✓ 该行确为唯一核 FORMAT_VERSION 的位置 |
| 446:`.claude/rules/format-evolution.md:7` | `**在第一个外部用户出现之前，磁盘格式是软的**——随时可以拆了重做，不需要向后兼容。` | ✓ 逐字相同 |

小计：核了 21 条不同的行:目标引用（重复出现的按次数记则为 24 次），**全部 ✓，0 处 ✗**。

### Opus 复跑

模型目录 `research/prompts/m2-rollback-forward-r3-opus-model/` 的 `SHA256SUMS` 先核对：`cd` 到该目录 `sha256sum -c SHA256SUMS`，六个文件全 OK。

草稿目录布局（为让 `rerun.sh` 里 `HERE/../../..` 正确解到仓根，按原相对深度重建）：
`/tmp/claude-1000/m2-rollback-forward-r3-verifier/rerun/reporoot/research/prompts/m2-rollback-forward-r3-opus-model/`（拷模型五个文件）+ `research/scripts/`（拷项目脚本）+ `Cargo.toml`/`Cargo.lock`（拷主仓根的，未改）。

命令（`full` 档同时跑 fast 十条用例与 K1 sweep）：
```
bash reporoot/research/prompts/m2-rollback-forward-r3-opus-model/rerun.sh \
  /tmp/claude-1000/m2-rollback-forward-r3/tree \
  /tmp/claude-1000/m2-rollback-forward-r3-verifier/rerun/work full
```
（脚本内部已含 `run-with-memory-cap.sh 16G` + `capped.sh 12` + `nice -n 19`，未再叠加。）

- `rerun-fast.out`：新产出 316 行，与模型目录里的 `rerun-fast.out` 逐行 diff（先把 `finished in N.NNs` 与「N filtered out」两个可变字段替换成占位符再比）：
  ```
  diff exit=0
  ```
  **✓ 逐行相同**（挂钟、filtered-out 计数除外，符合报告自述）。
- `rerun-sweep-formula.out`：报告写单跑 566 秒，估时 < 40 分钟，复跑。新产出：
  `RBF2 g1_sweep totals={"formula_revive_reclaimed_sum": 0, "histories": 2400, "orphans_sum": 0, "other_bad_roots": 0, "released_sum": 19833, "remounts": 972, "revive_skipped_sum": 0, "revived_sum": 20694, "rollback_done": 5316, "rollback_to_instance_1": 1772, "rollbacks_with_revive": 4688}`，`test result: ok. … finished in 550.41s`（本机同一时段有另一会话在跑重负载，550s 与原报的 566s 同量级）。与模型目录原文件同一处理后 diff：
  ```
  diff exit=0
  ```
  **✓ 逐字相同**（挂钟除外）。totals 里的 `histories=2400`、`rollback_done=5316`、孤儿计数隐含为 0（键不出现，脚本约定「只打印非零键」）等，与 Opus 报告第九节原样行、正文叙述完全对应。

两条复跑命令均 ✓；`SHA256SUMS` 自证 ✓。复跑用的草稿目录 `/tmp/claude-1000/m2-rollback-forward-r3-verifier/rerun/`，`target/` 已在交回前删除。

## 云端辩方（Sonnet）核对表

Sonnet 没有模型目录（自称只读代码，未建原型/测试），无复跑命令。全部引用分两类：对快照代码/kb（现查快照）；对第一、二轮判决与三条腿报告原文（这些文件本轮尚未归档删除，仍在 `research/prompts/`，直接对主树现查——它们是上一轮已提交产物，不受这一轮快照约束）。

| 引用 | 抄的内容 | 结果 |
|---|---|---|
| `m2-rollback-forward-r2-opus-output.md:222`（X3 四句第 3 句，故障数一句） | 「3. 字面：I-7.4……故障数：两次崩溃 + 盘 0 上两个根槽坏。」 | **✗ 行号错**：该段文字实际在第 **223** 行；222 行是空行 |
| `_m2-rollback-forward-r3-background.md:20`（K1 问题栏目整行） | 「崩在那次发布的每一点、之后最新 1..k 条根读不出、回退之后接着写再回退」 | ✓ 逐字相同（本行内容与本轮正文 `_m2-rollback-forward-r3-body.md:13` 一致，背景材料是正文的合成收录，非误写） |
| `m2-rollback-forward-r1-opus-output.md:175-184`（H6/`t7c` 历史范围） | 「暂时读不出 → 撤故障 → 再重开 → 写 → 再改坏 7 条根」链式历史 | ✓ 范围与转述均对应（六步历史逐条核对） |
| `m2-rollback-forward-r2-verifier-output.md:97-101`（H6 链式历史核实 ✓） | 只作为「已核实」的出处指路，未逐字引用 | ✓ 范围内第 99 行确为 H6/`t7c` 那一行核对结果 `✓` |
| `m2-rollback-forward-r2-opus-output.md:174`（X2 故障描述） | 「盘 1 上带 F 的根坏，2..3 个根槽」 | **✗ 引号内非逐字**：原文是「盘 1 上带 F 的根**全改坏**（2..3 个根槽）」——丢了「全改」二字，标点也从括注改成了逗号；行号本身没错 |
| `.claude/kb/checks-owed.md:367`（C419，共 2 处） | C419 整行（形态、判别力自证句） | ✓ 两处均逐字相同 |
| `m2-rollback-forward-r2-main-verification.md:58`（病根一句） | 「病根是 F 只住在根记录里，与 C419 取哪种规则无关。」 | ✓ 逐字相同 |
| `m2-rollback-forward-r2-opus-output.md:227` 起（X4 g3_control 数据行） | 「RBF2 g3_control rule=max … kill_all_carriers … F_after=0」（省略号截断） | ✓ 「227 起」指一段落，实际数据行在 230 行，内容与截断处吻合 |
| `m2-rollback-forward-r2-main-verification.md:108`（交用户三选项，共 2 处） | 整行 | ✓ 两处均逐字相同 |
| `crates/singlefs-checker/src/walk.rs:3047-3111`（`judge_rollback_floor_raises_against_their_ceilings`，共 3 处） | 声称是该函数的行区间，并据此列出入参 `reader/geometry/roots/cache` | **✗ 区间不等于函数范围**：函数签名在 3060 行（`fn judge_rollback_floor_raises_against_their_ceilings(`），实际闭合括号在 **3124** 行，3047 行只是它文档注释的起点；区间 3047–3111 漏掉了 3112–3124（该函数处理 `raising_roots_judged == 0` 的分支，同属这个函数体）。另外该函数签名实际有 5 个参数（`reader, geometry, roots, cache, judgements`），报告列出的入参漏了 `judgements: &mut Judgements` |
| `walk.rs:2966`（`rollback_floor_ceiling_before_the_raise`，共 2 处，与上一条并列列出入参） | 定位该函数，并称入参与前一函数「只有」`reader/geometry/roots/cache` 四个 | **✗ 行号偏差 + 入参列漏项**：函数签名 `fn rollback_floor_ceiling_before_the_raise(` 实际在 **2968** 行，2966 行是它文档注释中间一行；该函数实际有 6 个参数，除 `reader/geometry/roots/cache` 外还有 `raising_root: &crate::RootView`、`floor_before_the_raise: u64` 两个未被列出 |
| `walk.rs:3103`（`judgements.judge("I-7.9", …)` 判定行） | `judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {` | ✓ 逐字相同 |
| `m2-rollback-forward-r2-opus-output.md:270-274`（X6 g4_same 数据块） | 完整代码块（含 whole_devices_identical=true 等） | ✓ 逐字相同，区间边界（含开闭围栏）精确对应 |
| `m2-rollback-forward-r2-opus-output.md:278`（X6 四句第 2 句，共 2 处） | 「看不到……任何只看盘的判法都做不到」 | ✓ 两处均逐字相同 |
| `crates/singlefs-core/src/recovery.rs:1161-1167`（`rollback_high_water_of_root`） | 函数整体代码 | ✓ 逐字相同（含 `#[must_use]` 起始行到闭合大括号，7 行区间精确对应函数体） |
| `crates/singlefs-core/src/mount.rs:1737-1761`（`instance_rows_to_write`，共 2 处） | 结构性转述（覆盖区间 `[first_row_instance, instance_to_acquire)`、`is_rollback` 取自 `previous_row`） | ✓ 区间精确对应函数体（1737 fn 签名到 1761 闭合括号），转述与实现一致 |
| `mount.rs:1741-1742`（取值范围表达式） | `let first_row_instance = previous_row.instance.0.max(1); (first_row_instance..instance_to_acquire.0)` | ✓ 逐字相同 |
| `.claude/kb/decisions/23-journal的角色与格式.md:404`（回退行/第五条，共 2 处） | 「回退行写在新实例的实例表里……第五条在任何可达的历史上都取不到真」等多句 | ✓ 逐字相同 |
| `m2-rollback-forward-r2-main-verification.md:79`（G5「截断删掉」，共 3 处） | 整段（含「第一轮判决 F3 那一行『可以删（推的）』这一轮量过了」） | ✓ 逐字相同 |
| `m2-rollback-forward-r1-main-verification.md:42`（F3 判词，共 2 处） | 整行 | ✓ 逐字相同 |
| `m2-rollback-forward-r2-opus-output.md:311`（旧镜像带回退行一句，共 2 处） | 整句 | ✓ 逐字相同 |
| `m2-rollback-forward-r2-main-verification.md:55`（X3 打中一句） | 整段 | ✓ 逐字相同 |
| `m2-rollback-forward-r2-main-verification.md:83`（G5 第 83 行，「推的，没量」） | 整行 | ✓ 逐字相同 |

**小计**：核了 23 处不同的行:目标引用，**✓ 19 处，✗ 4 处**（`r2-opus-output.md:222` 行号错；`r2-opus-output.md:174` 引号内非逐字；`walk.rs:3047-3111` 区间越界且入参漏项；`walk.rs:2966` 行号偏差且入参漏项）。

**附加发现（不计入上面 ✓/✗ 计数，供主 agent 判断用）**：`walk.rs:2966`/`:3047-3111` 两处漏列的参数（`raising_root`、`floor_before_the_raise`、`judgements`）都不是「哪条代码路径发出这次抬 F」这类调用来源信息——前两者是从磁盘镜像解析出的根记录与由它算出的数值，后者是判定结果的输出累加器——所以 Sonnet 问②的核心论断（判定函数只吃 `ImageReader` 解出的字节、分不出卸载与准入两类历史）本身没有被这处引用误差反驳；但「入参只有……那四个」这句字面上与代码不符，用产物核对表判是 ✗，需与上面这条论断的成立与否分开看。

## 本地攻方核对表（只核事实表出处行号与译文，per 派发范围）

译文核对表 `research/prompts/m2-rollback-forward-r3-local-attack-translation-audit.md` 逐行核，出处对本轮正文 `_m2-rollback-forward-r3-body.md`（现查）与 kb 快照（现查）。

| 英文项 | 原文文件:行 | 结果 |
|---|---|---|
| part1 Fact 1（two disks；0/1/0；每区 8 槽） | `_m2-rollback-forward-r3-body.md:48` | ✓ 逐字含该短语 |
| part1 Fact 1 vocabulary note（区域归属固定） | `decisions/22-单元原子性怎么合成.md:67` | ✓ 内容对应（「加盘/换盘不再改变任何既有区域的归属」） |
| part1 Fact 2（区域数 R=3） | `decisions/22-单元原子性怎么合成.md:55` | ✓ 逐字相同 |
| part1 Fact 3（F_d 定义） | `_m2-rollback-forward-r3-body.md:9` | ✓ 逐字含该短语 |
| part1 Fact 4（回退下界 F，8 字节） | `decisions/22-单元原子性怎么合成.md:141` | ✓ 内容对应 |
| part1 Fact 5（MAX/MAX+HOLD/SYSCFG 三式，3 行同引一处） | `_m2-rollback-forward-r3-body.md:16` | ✓ K4 整行内容对应，含译文核对表自己列出的两处首稿失真（已用 replace-once.py 定点改正，回读命中 1 次） |
| part1 Fact 6（两槽/择槽规则，2 行） | `decisions/22-单元原子性怎么合成.md:197`、`:213` | ✓ 均逐字/内容对应；213 行的「省略池级跨盘统一」被核对表自己标「有意省略、非误漏」并给出理由 |
| part2 Fact 1（去重规则） | `_m2-rollback-forward-r3-body.md:15` | ✓ 逐字含该短语 |
| part2 Fact 2（比较基准是树表条目） | `decisions/16-发布语义.md:44` | ✓ 内容对应；核对表自述省略了「只比紧邻前一条」那半，理由已写明 |
| part2 Definition 4 支撑依据（回退语义） | `decisions/23-journal的角色与格式.md:378` | ✓ 内容对应（「从 R_old 那棵账重新载入」） |

小计：核了 10 条出处引用，**全部 ✓，0 处 ✗**。

### 运行记录与样本自检核对

`m2-rollback-forward-r3-local-attack-runlog.md` 报的词数与 `wc -w` 现算一致：part1 s1=570、s2=348，part2 s1=82、s2=166——全部相同。用 `research/scripts/oov-check.py`、`corruption-check.py` 对四份样本重跑：

- 四份样本两项检查退出码均为 0（绿或可通过），与 runlog「干净」判定一致。
- **一处发现（非 ✗，characterization 偏差）**：`corruption-check.py` 对 `part2-output-s1.md`（82 词）判的是**灰**（`语料太小（cjk=0 words=76），判不了`），不是绿；`oov-check.py` 对它是绿。runlog 该行写「干净（oov-check.py 绿……）」，只提了 oov-check 的判定，没提 corruption-check 实际返回的是「判不了」而非「绿」。灰色分支退出码同样是 0，不影响 `ask-local.sh` 的接受与否（三方论证规则「任一判红即拒绝」，灰不是红），但 runlog 的「干净」措辞对这一份样本比两个工具实际给出的结论更强。

小计：本地样本自检 4 份文件 × 2 项检查 = 8 项复核，**7 项与 runlog 描述一致，1 项（corruption-check 对 part2-s1）措辞偏强但不影响判定结果**，不计入 ✗（该项工具本身返回的是「判不了」，不是「判错」）。

## 另外单列 1：Opus 报告里这条腿自己提的改法（逐条列出，标「被攻过零轮」）

Opus 报告第十节表格与十二节自述里点名的五条改法，报告原句：「这条腿自己提的改法（水位取 max、SysPre 的『先写后抬』、checker 读系统配置、(ii) 连只读也拒、(i) 写明 incompat 位）**只在我的模型上量过或只是推的，被攻过零轮**。」（`m2-rollback-forward-r3-opus-output.md:476`，现查为逐字相同）逐条列出：

1. **水位取 max(现行那一版内存里的水位, 环里读得出的根)**（修 Y1）——量过（`Full` 臂 1 段历史不中），被攻过零轮。
2. **SysPre：抬 F 之前先对系统配置预写一次、带屏障**（修 Y6 的分辨臂问题）——量过（774+434 格不中），被攻过零轮；真实实现下窗口收窄到 1 个前缀是推的、未验。
3. **checker 跟着读系统配置里的 F_生效**（修 Y7 的 checker 误红）——推的，没有实现、没有跑过，被攻过零轮。
4. **I-7.9 (ii) 改成「见证表非空就连只读也拒」，或只读择根保留见证判**（修 Y10）——推的，没有实现，被攻过零轮。
5. **K7 (i) 写明「换 incompat 布局身份位」而非「改格式版本号」**（修 Y11）——旧读者对新位挂不上这一半量过，新读者对旧位拒是按现有 `incompat_bits_are_mountable` 代码推的，被攻过零轮。

以上五条均由 Opus 自己在同一份报告里提出并标注（第十节表末行、各 Y 小节「改法」句），核查员未对其可行性或该不该采纳做任何判断。

## 另外单列 2：Sonnet 报告里要主 agent 判的四句，各自核引用对不对

**① 「X3 比 X2 只多两个崩溃点」**（原句：「它只比 X2 多了两个标准崩溃点……真正的外部故障计数与 X2 相同（1 次、2 个根槽）」，`m2-rollback-forward-r3-sonnet-output.md:12`）。
引用核实：X3 自身故障描述在 `m2-rollback-forward-r2-opus-output.md` **第 223 行**（Sonnet 报告里误写成 222 行，已在上面 Sonnet 表里记 ✗）：「故障数：两次崩溃 + 盘 0 上两个根槽坏」；X2 自身故障描述在同文件第 174 行：「盘 1 上带 F 的根全改坏（**2..3 个根槽**）」。**核到的差异**：X2 的根槽数原文是一个范围「2..3 个」，不是固定的 2 个；X3 是固定的「两个」。Sonnet 概括成「真正的外部故障计数与 X2 相同（1 次、2 个根槽）」时，把 X2 的「2..3」简化成了「2」——落在 X2 范围的下界，不算捏造，但不是逐字对应。

**② 「I-7.9 的判定函数签名只吃镜像字节（`crates/singlefs-checker/src/walk.rs`）」**（原句见 Sonnet 报告第 28、86 行）。
引用核实：见上面 Sonnet 表——两处函数（`judge_rollback_floor_raises_against_their_ceilings`、`rollback_floor_ceiling_before_the_raise`）的行区间与列出的入参都有出入（区间越界、漏列 `judgements`/`raising_root`/`floor_before_the_raise`），已记 ✗。但漏列的三个参数逐一核实后都不携带「调用方是谁/入口在哪」这类信息（两个是磁盘镜像数据、一个是输出累加器），核查员的现查支持「函数确实只吃 `ImageReader` 解出的字节，不吃调用路径标记」这个更窄的事实性描述；「只吃镜像字节」这句本身站得住，但引用它的两处行号/参数列表要按上面 ✗ 修正。

**③ 「截断走不到是 `mount.rs` 里 `instance_rows_to_write` 取值范围决定的结构事实」**（原句见 Sonnet 报告第 56、87 行）。
引用核实：`mount.rs:1737-1761`（函数体）、`:1741-1742`（取值范围表达式）均逐字核对无误（见上面 Sonnet 表）；`recovery.rs:1161-1167`（`rollback_high_water_of_root`）逐字核对无误；kb `23-journal的角色与格式.md:404` 的「在任何可达的历史上都取不到真」一句逐字核对无误。三处引用支撑的推导链（区间严格小于 `instance_to_acquire`、`root.instance` 恰好等于 `instance_to_acquire`、故 `row.instance == root.instance` 恒不成立）本身是代码逻辑推导，不属于本报告核查范围（核查只核引用是否对得上原文，不核推理本身）。引用全部 ✓。

**④ 「第二轮判决第 79 行与第 83 行自相矛盾」**（原句见 Sonnet 报告第 66-76 行）。
引用核实：`m2-rollback-forward-r2-main-verification.md:79` 与 `:83` 均逐字核对无误（见上面 Sonnet 表）；79 行原文含「第一轮判决 F3 那一行『可以删（推的）』这一轮量过了」，83 行原文含「旧镜像带回退行（flags bit0）：删掉之后的代码照读这一行、照挂，前提是实例表行的解析不动（推的，没量）」。两句的字面差异（前者说「那一行……量过了」，后者说「……没量」）是这两处引用本身呈现出的事实，核查员未对「是否构成矛盾」「该不该改判决措辞」做判断，只确认两处引文都逐字对应原文、且确实出自同一份文件、相隔 3 行（79 与 83 之间为 80、81、82 三行）。

## 另外单列 3：本地攻方第一问两份样本 与 Opus X4 那一格的「2 个」vs「6-8 个」——核前提，不判谁对

**本地样本的「2」**：part1 item1(a)/(c)（`m2-rollback-forward-r3-local-attack-part1-output-s1.md:2,5`、`part1-output-s2.md:2,4`）算的是 MAX 设计下、在提示文件自建的 **modeling assumption「favorable case」** 前提下，让 F_生效 回落到 0 所需坏的最少根槽数：该前提明写「每块盘上最新发布的根带非 0 的 F，但再往前一条仍持久有效的根已经带 F=0」（`part1.md:102-104`，现查；同一段「not drawn from any document」的自述在紧邻的 `part1.md:100-101`，现查，二者合起来说明这段建模假设不是从任何 kb 或历史读出的）。在这个前提下，每块盘只需坏 1 个（最新那个）槽即可让该盘的 F_sub_d 归零，两块盘合计 2。这是一个**抽象的、非绑定到任何具体历史**的下界，两份样本（s1、s2）算法与结论一致，互相印证。

**Opus X4 的「6-8」**：出自 `k4_x4_carriers_syscfg_and_checker` 这一具体模拟历史（`m2-rollback-forward-r3-opus-output.md:321-329`，现查，表格「第二轮 X4（带 F 的根全坏）在各臂上」），前提是一段**真实跑过的具体写入序列**——实例 2 覆盖写到现行 txg 7、B1 卸载那一串落盘（chain=[8,9,10]），重开后再写 0/1/2 次——此时**这个具体历史里带 F 的根槽总数恰好是 6、7 或 8 个**（随重开后写次数变化），而这些根槽都带同一个新 F，没有任何一块盘上还留着一条带旧 F（更低或 0）的持久有效根。这对应本地提示 modeling assumption 里定义的 **unfavorable case**（「同种槽全部带同一个非 0 的 F，要清空贡献就要坏光那一类槽」），但坏光的槽数不是本地 item1(b) 算的根环满容量 24（两块盘理论最大槽位数 16+8），而是这一次具体实验里**已经被写入、带 F 的根槽的实际数目**（6/7/8）——小于满容量，因为这次模拟的写入序列还没有把两块盘的全部 24 个槽都写成带 F 的根。

**结论（只列前提差异，不判数字对错）**：两个数字回答的不是同一个问题。本地的「2」是「favorable case」这个抽象假设下的**理论下界**；Opus 的「6-8」是**一次具体模拟历史**（在 harness 里真跑出来的一段写入序列）落在 unfavorable 区间内、但受限于该历史当时已写入的根槽实际数目的**实测值**，比本地 item1(b) 算的抽象上界 24 小。两者互不矛盾，是因为回答的问题（「理论上最少要坏几个」vs「这一段具体历史里要坏几个」）本来就不同；核查员未对两边何时更适合当作交用户的判据下结论。

## 总计数

| 腿 | 核了几处 | ✓ | ✗ | 其他（不计入✓/✗） |
|---|---|---|---|---|
| 判别力自证 | 1 | 0 | 1（预期，证明方法能分辨） | 0 |
| Opus 引用（kb/代码/正文） | 21 条（含重复共 24 次） | 21 | 0 | 0 |
| Opus 复跑（rerun-fast.out、rerun-sweep-formula.out）+ SHA256SUMS 自证 | 3 | 3 | 0 | 0 |
| Sonnet 引用 | 23 条 | 19 | 4（行号错 2、区间越界+入参漏项 2） | 0 |
| 本地攻方译文核对表出处 | 10 | 10 | 0 | 0 |
| 本地攻方样本自检（oov/corruption） | 8 项 | 7 | 0 | 1（corruption-check 对 part2-s1 判「灰」，runlog 措辞偏强，未记 ✗） |
| **合计** | **66** | **60** | **5** | **1** |

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（K1-K7 的机制对不对、C419 该取哪条规则、I-7.9 三条改法哪条更好），只核引用、产物与复跑。
- 未对 Opus 报告里的原型代码（`rbf3_attack.rs`、两份 patch）做逐行代码审查，只核对了 `SHA256SUMS` 自洽与复跑产物是否与模型目录里留存的产物一致。
- 未复跑本地攻方两份提示（`ask-local.sh` 需要网关，未在核查范围内要求复跑模型答复，只核对了已有样本的词数与损坏检测）。
- 未对 Sonnet 报告的推理链条（问①③里从代码结构推出的「恒真命题」「合法历史」等结论）做正确性判断，只核对了它引用的每一行代码/kb/历史文档是否真在那个位置、内容是否逐字或转述准确。
- 未读禁读清单（本轮无单独发的禁读清单；派发提示未列出被禁读的文件）。
- 未跑门禁、层 0、QEMU、全量 `cargo test`；只在草稿目录里复跑了 Opus 模型的 `rbf3_attack` 测试二进制（fast + K1 sweep 两档），经 `run-with-memory-cap.sh 16G` + `capped.sh 12`。开跑前 `ps` 看过，未见性能测量类进程在跑；跑的过程中本机另有一个会话在跑大量 `cargo test --release --bin e142-first-txn-dry-run`（未干预、未修改，只在报告里如实记了它导致我的复跑挂钟比原报告长）。
- 未核对第一轮判决（`m2-rollback-forward-r1-main-verification.md`）整份文本，只核了 Sonnet 报告点名引用的那几行（:42）。
- 草稿目录 `/tmp/claude-1000/m2-rollback-forward-r3-verifier/`：`rerun/work/repo/target/` 已在本报告交回前删除；`selftest/`（判别力自证用的副本）、`rerun/reporoot/`（重建的相对路径骨架，含从主仓拷来的 `research/scripts/`、`Cargo.toml`/`Cargo.lock`、模型目录副本）与 `rerun/work/`（复跑产出，含 `rerun-fast.out`、`rerun-sweep-formula.out`）留存，供主 agent 需要时复核。
