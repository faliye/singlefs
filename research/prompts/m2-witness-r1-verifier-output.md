# m2-witness-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

草稿目录：`/tmp/claude-1000/-home-fy5090-code-singlefs/d16a74c5-453c-44d5-8a19-7e71d116de72/scratchpad/m2-witness-r1-verifier-draft/`（派发提示未给专用草稿目录，按共用约束借用本会话 scratchpad 下一个新建子目录，已在这里写明；报告与草稿之外未写别处）。

## 输入缺口（先声明）

- 派发提示没有给「草稿目录」这一项（agent-common.md 与定义都把它列为主 agent 必须给的输入）。已按共用约束「写」一节的精神，在本会话 scratchpad 下新建 `m2-witness-r1-verifier-draft/` 当作草稿目录续做，未停下等要——历史上同类轮次（`ae2726a` 那次 `three-way-verifier`）也是这么处理的。此处如实记录，供主 agent 判断是否需要按规范补一次正式指派。
- 云端腿交回的报告 `sha256sum` 派发提示未给（属于「有就给」的可选项），本报告改用「现读 kb/产物原文逐字比对」与「复跑 sha256 比对」两种硬证据，不影响核查结论。

## 判别力自证（先做，必须判 ✗）

挑的引用：sonnet-output.md 第 11 行对 `.claude/kb/decisions/22-单元原子性怎么合成.md:55` 的引用（表格行「区域数 R」）。

- 待核引用声称第 55 行内容：`| **区域数 R** | **3**（\`R = F + 1\`，F = 2） |`
- 把行号故意 +1 改核第 56 行，副本里第 56 行原样内容：`| **区域位置** | 区域 r 落在 \`r × P × chunk\`，**P 素数且 \`P > devs\`**；地址空间、基址与第一版取值见已定项 16 |`
- 两者不相等 ⇒ **判 ✗**。核查方法能分辨行号错位，判别力自证通过。

## 快照核对方式（先声明，影响下面怎么判）

开工快照 `research/prompts/m2-witness-r1-snapshot/sha256sums.txt` 只存了 13 个文件的 sha256（外加一个时间戳行），**没有随附实际文件内容副本**。现对 13 个文件逐一核对当前主树 sha256：

```
MATCH: .claude/kb/decisions/05-快照-空间记账机制.md
MATCH: .claude/kb/decisions/16-发布语义.md
MATCH: .claude/kb/decisions/18-块里携带什么信息.md
MATCH: .claude/kb/decisions/20-承重面单元的原子性与自包含.md
MATCH: .claude/kb/decisions/22-单元原子性怎么合成.md
MATCH: .claude/kb/decisions/23-journal的角色与格式.md
MATCH: .claude/kb/checks-owed.md
DIFF:  .claude/kb/experiments/158-择根与修复四岔路.md（快照=6ec0cb9a… 当前=fec55340…）
MATCH: crates/singlefs-core/src/recovery.rs
MATCH: crates/singlefs-core/src/mount.rs
MATCH: crates/singlefs-core/src/system_configuration.rs
MATCH: crates/singlefs-core/src/transaction.rs
DIFF:  crates/singlefs-harness/src/bin/e158_root_choice_repair.rs（快照=0b2205da… 当前=1e73dd20…）
```

11 个文件与当前主树逐字节一致 ⇒ **这些文件的引用直接对主树核**。
2 个文件（`experiments/158-择根与修复四岔路.md`、`e158_root_choice_repair.rs`）与快照不同——这正是派发提示预告的「kb 书记员改过、crates/ 打过补丁」。

**e158_root_choice_repair.rs**：这一轮攻方腿（opus）自己在副本上打了 `prototype.diff`、重新编译验证（见下方复跑），不依赖今天主树这份文件的原文行号，不受影响。

**experiments/158-择根与修复四岔路.md**：现查 `git diff HEAD` 只在两处开刀——文件开头的标题行（第 1 行，纯叙事措辞变化），以及原第 443 行之后插入了新内容（session s6 + session s7，净增约 259 行）；**第 443 行之前的内容逐字节未变**。HEAD 提交（`e980a21` 之前，10:36）与当前主树在这一段完全相同，而快照哈希与两者都不同，说明快照捕捉的是「HEAD + session s6 插入、session s7 还没插入」这个从未提交过的中间态（证据见下方「分不清」条目）。**凡是被核引用行号 < 443 的，视为跨快照/HEAD/当前三态稳定，直接对当前主树核**；≥ 443 的按「分不清」单列，不判 ✗。

## 一、云端攻方腿（Opus，`m2-witness-r1-opus-output.md`）

### 1a. kb / crates 原文引用

| 引用（报告行号:内容） | 核的结果 |
|---|---|
| L61 `crates/singlefs-core/src/recovery.rs:625`：「/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的。」 | ✓ 现查该行原样一致 |
| L190/L223 `.claude/kb/checks-owed.md:286`（C332 行，转述不整抄） | ✓ 行号是 C332 那一行，内容主题与转述一致；报告自己标了「转述」，不判摘句 |
| L220 `crates/singlefs-core/src/transaction.rs:389`：「/// 实例代号) + 1；逐盘写一次系统配置槽（世代号逐盘 +1）；两写之后一道屏障，屏障完成才把新号交出去，于是本实例的第一个」 | ✓ 现查该行逐字一致 |
| L220 `crates/singlefs-core/src/mount.rs:1527`（调用处，未声明逐字引） | ✓ 该行是 `acquire_expected_instance` 调用点，与叙述一致 |
| L220/L223 `.claude/kb/decisions/20-承重面单元的原子性与自包含.md:68`：「自证单元（根槽、journal 记录头、系统配置槽）…等于运行时探测到的 `physical_block_size`…」 | ✓ 现查该行逐字一致 |
| L245 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:183`：「判别子观测不到 | 判据要被判的系统区分两种情形…」 | ✓ 现查该行逐字一致 |

7 处引用实例，**7 ✓，0 ✗**。均对主树核（这些文件全在快照 MATCH 名单内）。

### 1b. 复跑（承重的几格；入库仓一字节未动，全在草稿目录副本上做）

复跑方法：`rsync` 拷一份入库仓到草稿目录（排除 `target`/`.git`），`patch -p1` 打上腿自带的
`research/prompts/m2-witness-r1-opus-model/src/prototype.diff`（`patch --dry-run` 先核对全部
8 个文件均可干净应用），`bash research/scripts/capped.sh 4 cargo build --release -p singlefs-harness
--bin e158_root_choice_repair` 编译（26 秒完成，1 条既有 `dead_code` 警告，与腿报告不冲突）。
之后逐条命令按报告给的环境变量组合跑，**全部前台跑、线程上限 4**，不跑全量崩溃枚举（batch3 那类）。

| 承重格 | 复跑命令 | 报告里抄的原样 | 复跑结果 | sha256 比对 |
|---|---|---|---|---|
| 213 个组合，放处 a，512 物理块 | `SINGLEFS_WITNESS_PLACEMENT=a SINGLEFS_WITNESS_ORDER=pre E158_PBS=512 … w-family 2` | `pairs=283947 pairs_with_any_violation=0`（`out/recheck/w-family-a-512.out`） | 逐字节相同（`diff` 0 行不同） | archived `47c4924d…` = recompute `47c4924d…` ✓ |
| 213 个组合，放处 b，512 物理块 | 同上，`PLACEMENT=b` | `pairs=283947 pairs_with_any_violation=0`（`out/batch1/w-family-b-pre-512-2.out`） | 逐字节相同 | archived `ab97c807…` = recompute `ab97c807…` ✓ |
| 213 个组合，放处 c，512 物理块 | 同上，`PLACEMENT=c` | `pairs=283947 pairs_with_any_violation=0`（`out/batch2/w-family-c-pre-512-2.out`） | 逐字节相同 | archived `aa31129c…` = recompute `aa31129c…` ✓ |
| 今天（none）对照 | 同上，`PLACEMENT=none` | `pairs_with_any_violation=213`（`out/recheck/w-family-none-512.out`） | 逐字节相同 | archived `d1d8…`→见下"none"行已核 ✓（`witness_family_summary`/`by_checkpoint` 两行核对，`a_V1=213 a_V2=175 b_V1=0 b_V2=175 c_V1=0 c_V2=175`，逐字一致） |
| (b) 静默撤销最少 4 个故障 | `PLACEMENT=b … w-min` | `constructed_set_hits=1702 … histogram_of_fault_count={4: 77, 5: 94, …}`（`out/batch1/w-min-b-pre-512.out`） | 逐字节相同 | archived `f0d54779…` = recompute `f0d54779…` ✓ |
| 写序 pre 打中 H1 | `PLACEMENT=a ORDER=pre PBS=512 … w-targeted` | `CRASH E1 seg=0/11 … persisted=[0] rb0=true … minF=1 example=[W0@4096]@a`（`out/batch4/w-targeted-a-pre-512.out`） | 逐字节相同（含 EDGE E1/E2 两行） | archived `ae6a5d9f…` = recompute `ae6a5d9f…` ✓ |
| 写序 post 打中 H2 | `PLACEMENT=a ORDER=post PBS=512 … w-targeted` | 报告未整行抄单条 CRASH，给的是「三.2」`minseg.py` 汇总表 | 见下 | archived `6e9eef55…` = recompute `6e9eef55…` ✓ |
| (c) 在 512 物理块上的新错（H3，撕裂） | `PLACEMENT=c ORDER=pre PBS=512 … w-targeted` | `CRASH E1 … torn=SC0.1:blocks[1]of[0, 1] rb0=true … minF=1 example=[W0@4608]@a`（`out/batch4/w-targeted-c-pre-512.out`） | 逐字节相同 | archived `92c93266…` = recompute `92c93266…` ✓ |

**三.2 表（`minseg.py`）独立复算**：把上面 4 份复跑产物按脚本要求的命名放进草稿目录一个新子目录，跑
`python3 minseg.py <草稿目录> E1`，输出三行与报告表格逐格比对：

- 「今天」行：复算 `- - - - 1 1 1 2 2 2 3 3`，报告表 `- - - - 1 1 1 2 2 2 3 3`，**逐格相同**
- 「(a)/(c)-pre」行：复算 `1 2 2 2 3 >3 >3 >3 >3 >3 >3 >3`，报告表同一行，**逐格相同**
- 「(a)/(c)-post」行：复算 `- - - - 1 3 3 >3 >3 >3 >3 >3`，报告表同一行，**逐格相同**

**7 条承重复跑命令，7 ✓，0 ✗**（含 4 处整份文件 sha256 比对相同、3 处整份 diff 0 行不同）。

## 二、云端正推腿（Sonnet，`m2-witness-r1-sonnet-output.md`）

无复跑命令（报告自己声明「没有编译或跑任何代码」），只有 kb/crates 原文引用，逐条核：

| 引用（报告行号）:文件:行 | 核的结果 |
|---|---|
| L11 `decisions/22…:55,57,58`（R/槽序/S 三行定案表） | ✓ 三行逐字节一致（判别力自证已用这条） |
| L37 `decisions/22…:270`（已定项 12 定案句，整段抄） | ✓ 逐字节一致 |
| L39 `decisions/18…:304`（实例表 kind=0 行「一片写满 369 行再开下一片」） | ✓ 逐字节一致 |
| L41 `decisions/18…:311`（回收判据，转述并声明「不是原文」） | ✓ 位置正确，报告自己标了转述 |
| L49 `decisions/18…:299`（kind=0 行字段表） | ✓ 逐字节一致 |
| L51 `experiments/158…:15`（16/12 字节两种宽度余量） | ✓ 逐字节一致；该行 < 443，跨快照/HEAD/当前三态稳定 |
| L63 `recovery.rs:627-646`（`choose_root` 函数体） | ✓ 逐字节一致 |
| L63 `experiments/158…:11`（E158 现查 `choose_root` 那句，含它自己引的旧行号 412） | ✓ 逐字节一致；< 443，稳定 |
| L63 `recovery.rs:412`（E158 原文里的旧行号，报告已注明「行号随重构在今天的树上是 627，语义未变」） | ✓ 报告已正确标注这是历史行号、给出今天的正确行号，不判 ✗ |
| L65 `decisions/16…:151`（骑手条款 1，整句抄） | ✓ 逐字节一致 |
| L77 `decisions/23…:351`（前缀第五条） | ✓部分：引文与第 351 行前半句逐字一致，但该行还有后半句「（C113…定案 P2：」延伸到 352 行「否则…回退被静默撤销）。」未被引用——不影响引文本身准确性，供主 agent 判是否算摘句 |
| L81 `decisions/23…:383`（射程节 ⚠️ 段，整段抄） | ✓ 逐字节一致 |
| L85 `decisions/18…:307`（回退写的行，⚠️ 表随根分版本那句） | ✓ 逐字节一致 |
| L91 `checks-owed.md:294`（C340 行，整段抄） | ✓ 逐字节一致 |
| L99 `checks-owed.md:453`（C514 行，整段抄） | ✓ 逐字节一致 |
| L107 `experiments/158…:53-54`（P332 判定句，跨行拼接，报告已注明软换行） | ✓ 拼接后逐字一致；< 443，稳定 |
| L111 `invariants.md:54`（I-7.4，整句抄） | ✓ 逐字节一致（该文件未在快照清单内，但未被本轮任何补丁触碰，对主树核） |
| L117 `decisions/22…:188`（D22 已定项 9，481 字节合计） | ✓部分：引文取的是该行「合计 481 字节…槽内余 3615」这一段，与行首「定案：字段表 = […]…45 行，」的引子部分未引，供主 agent 判 |
| L117 `decisions/20…:45`（已定项 3 标题，位置引用非逐字） | ✓ 该行确为「#### 已定项 3：「原子性」必须写成可判定的三问」标题，位置正确 |
| L121 `decisions/16…:170`（发布的持久顺序，声明「整句抄」） | ✓部分：引文完整覆盖到「…段序列因此唯一。」，但该行紧接着还有一句「fsync 返回条件多一句：…」未被引用——是否算作被「整句抄」覆盖的范围内，供主 agent 判 |
| L31 `experiments/158…:558-605`（「它答不了的」缺口清单，声称此节不含 N_w 封闭性问题） | **✓ 内容成立，行号仅对「开工快照那一刻」成立** —— 见下方专项说明 |
| L144 `recovery.rs`（627-646 行）、`mount.rs`（1778 行起，`mount_rollback` 存在） | ✓ 逐字节一致 |

**24 处引用实例，24 ✓（其中 3 处标「✓部分」——引文本身逐字准确，但未覆盖原行/原段的全部内容，供主 agent 判摘句），0 ✗。**

### 专项：`experiments/158…:558-605` 的行号漂移

sonnet 报告自称「开工快照…引文全部现场 `grep -nF` 对着 kb 源文件…核过」。这处引用指向的内容
（「它答不了的」一节，不含 N_w 封闭性问题）在**当前主树**上位于第 646-700 行，不是 558-605；
但在 **HEAD 提交**（本轮开工之前）上，同样的内容位于第 446-490 行。两个版本都不是 558。

用行数做算术核对：HEAD 到当前主树之间，第 443 行处插入了两段内容——「session s6」（新增 112 行，
446→557 正好是 112 行）和「session s7」（新增 88 行，558→645）。**如果快照捕捉的是
「HEAD + 仅插入 session s6、还没插入 session s7」这个从未提交的中间态**，「它答不了的」一节的起始行
应为 446 + 112 = **558**——与 sonnet 引用的行号精确吻合。这个中间态无法从 git 历史直接取出（两次编辑
都是未提交的工作区改动，快照本身只存了 sha256、没存内容），但行数算术与 sonnet 的引用完全自洽，
且「它答不了的」一节在 HEAD 与当前主树两个可读版本里都不含「N_w 的封闭性」，与 sonnet 的内容判断一致。

**结论：不判 ✗，记「行号随快照之后的 kb 书记员编辑（session s7 插入）下移，当前主树上同一节在
646-700 行；引用时（开工快照那一刻）558-605 是准确的」。**

## 三、本地攻方腿（`m2-witness-r1-local-attack.md` 提示本身 + 核对表 + 两份样本）

### 3a. 提示本身（`m2-witness-r1-local-attack.md`）给模型的「Fixed facts」引用

| 事实 | 文件:行 | 核的结果 |
|---|---|---|
| F1 (481 字节) | `crates/singlefs-format/src/lib.rs:190` | ✓ `SYSTEM_CONFIGURATION_BYTES: u64 = 481;` 逐字一致 |
| F2 (4096 字节) | 同文件:199 | ✓ `SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;` 逐字一致 |
| F3 (16/12 字节) | `experiments/158…:15` | ✓ 逐字一致（< 443，稳定） |
| F5 (N_w 数) | `experiments/158…:14` | ✓ 逐字一致（< 443，稳定） |
| F6 提案出处 | `checks-owed.md:285`（C331 行） | ✓ 「乙『系统配置带 txg』挡不住且零故障（…占位的 0）」逐字一致 |
| F6 8 字节出处 | `crates/singlefs-format/src/lib.rs:152` | ✓ 根记录字段表「checkpoint_txg 8」逐字一致 |
| F7 (512 字节) | `decisions/20…:133` | ✓ 「本机 NVMe：…physical_block_size = 512」逐字一致 |
| F8 第一句 | `decisions/20…:68` | ✓ 逐字一致（与 opus 引用同一行，二者互相印证） |
| F8 二/三句 | `decisions/22…:197` | ✓ 「内容允许跨过 512…可检测、可恢复。」逐字一致 |
| F9 | `decisions/22…:195` | ✓ 「系统运行量（不是配置）\|槽世代号 8、整槽校验和 32…\|52」逐字一致（原行有 `**` 加粗标记，引文未带，内容不变） |
| F9 校验和佐证 | `crates/singlefs-format/src/lib.rs:152` | ✓ 「自证校验和 32」在根记录字段表内，逐字一致 |
| F10 (2 盘) | `decisions/02-RAID条带策略.md:148-150` | ✓ 已定项 9 标题与定案句逐字一致 |
| F11 持久化函数 | `transaction.rs:575`（文档注释起于 572） | ✓ `fn persist_the_root_then_rotate_the_system_configuration` 与文档注释三行逐字一致 |
| F11 三处调用点 | `transaction.rs:643, 998, 3869` | ✓ 三处都是该函数调用，逐字一致 |
| F11 回退挂载 | `mount.rs:1778`（`mount_rollback`）、`:1478`（`establish_instance`） | ✓ 两个函数签名行都在，逐字一致 |
| F12 系统配置读 | `recovery.rs:516`（`choose_system_configuration`）、`:525`/`:540`（两次 `reader.read`） | ✓ 三处均一致 |
| Candidate A/B/C 定义句出处 | `research/prompts/_m2-witness-r1-background.md:14` clause (a)/(b)/(c) | ✓ 三个从句均在背景材料第 14 行原样出现 |
| 「-34 和 -42 与既有实验结果吻合」出处 | `experiments/158…:133` **和** `:918` | **✗ 见下方说明** |

**25 处引用实例，23 ✓，2 ✗（同一条判决的两个引用位置）。**

### 专项：`experiments/158…:133` 与 `:918` 均不含 -34/-42 这两个数

local-attack.md 第 77 行原话：「The -34 and -42 numbers in this worked example match numbers already
produced by a prior, independently run experiment recorded in `.claude/kb/experiments/158-择根与修复
四岔路.md line 133 and line 918`」。

- 现查 kb 文件第 133 行（< 443，跨快照/HEAD/当前三态稳定）：「岔路单第 3 行（候选 2 那一半）：…都推过
  512（放得进 4096），单条见证不推过 512；…」——**这一行谈的是同一个话题，但字面不含「-34」或「-42」**。
- 第 918 行：当前主树该文件只有 868 行，**第 918 行不存在**；HEAD 提交该文件只有 609 行，同样不存在。
- 现查背景材料 `_m2-witness-r1-background.md:918`：内容与 kb 文件第 133 行**逐字相同**——说明「918」是
  背景材料里同一句话的行号，被误当成了 kb 文件自己的行号（`.claude/rules/three-way-inference.md`
  提醒过的那种误写）。
- 真正含「-34」与「-42」两个数字的位置是同一份 kb 文件的**第 15 行**（< 443，稳定）：
  「…都推过 512（余量 −34/−18），与候选 2 见证同放系统配置槽（沿用 N_w）时余量 −42/−26…」——
  与 local-attack.md worked example 里的 -34、-42 逐字对应，但这一行没有被引用。

**判 ✗（两处）**：文件:行 → 实际位置
- `experiments/158…:133`（kb 文件自己的行号）→ 内容主题相关但不含引用要证明的那两个数字；
  真正含这两个数字的是同一文件第 15 行。
- `experiments/158…:918`（超出该文件当前 868 行与 HEAD 时 609 行的范围）→ **误写成背景材料
  第 918 行，原文件（该行内容）实为第 133 行**；即便按这条更正读，133 行本身仍不含 -34/-42。

### 3b. 转述核对表（`m2-witness-r1-local-attack-translation-audit.md`）逐行核

| 行 | 原文文件:行 | 行号在不在 | 抄的是不是原文 | 多加/丢限定词的说法能不能核 |
|---|---|---|---|---|
| §1 Candidate A | `background.md:14` clause (a) | ✓ 在 | ✓ 「放进系统配置槽、越过 512 字节…」逐字一致 | 「allowed to」的理由引 `decisions/22…:197`「内容允许跨过 512」✓ 逐字一致；「whole-slot」理由引 `decisions/22…:195`「整槽校验和」✓ 逐字一致 |
| §2 Candidate B | 同上 clause (b) | ✓ | ✓ 「独立的自证单元，自己带校验和、两槽轮换」逐字一致 | 「generation number」理由引「已定项 21…世代号缺一不可」（本报告未逐字复核已定项 21 原文，只核了行号 195/197 两处直接相关引文；已定项 21 原文未被核对表自己点名行号，无法核） |
| §3 Candidate C | 同上 clause (c) | ✓ | ✓ 「拆成几片、每片不超过 512 字节、各自自证」逐字一致 | 同 §2，理由引同一条已定项 21，未点名行号 |
| §4 F5 | `experiments/158…:14` | ✓ | ✓ 逐字一致 | 首稿缺「根环容量小（12 槽）」半句、定稿已补——核对表自述的编辑过程，非可核事实 |
| §5 F8 第一句 | `decisions/20…:68` | ✓ | ✓ 逐字一致（含「根槽、journal 记录头、系统配置槽」三个例子） | 首稿换成自己归纳定义、定稿改回点名三例——同上，自述过程 |
| §6 F8 二/三句 | `decisions/22…:197` | ✓ | ✓ 逐字一致 | 首稿误标 `decisions/20…:84`，定稿改到 `decisions/22…:197`——**现查 `decisions/20…:84` 是 D20 已定项 4「射程」段**，核对表这条更正属实（本报告未逐字复核 84 行内容，只确认定稿引用的 197 行正确） |
| §7 F9 | `decisions/22…:195` | ✓ | ✓ 逐字一致 | 省略 journal tail/实例代号两字段，理由可读（52 字节合计里只取自证相关的 8+32），未声称「52=自证开销」 |
| §8 F10 | `decisions/02…:148-150` | ✓ | ✓ 逐字一致 | 「exactly」理由引 `decisions/02…:181`「恒 w=2」✓ 现查逐字一致 |
| §9 F6 | `checks-owed.md:285` | ✓ | ✓ 「乙『系统配置带 txg』挡不住且零故障…」逐字一致 | 省略判决结果括注，理由可读（与 W5 算式无关） |

**9 行核对表全部核过，9 ✓，0 ✗**（一处因原文未点名具体行号而核不动，已在表中注明，不算入 ✓/✗）。

### 3c. 两份样本（s1、s2）逐格一致性

`m2-witness-r1-local-attack-output-s1.md` 与 `-output-s2.md` 各自独立填了 Candidate A/B/C 三张表、
每张表 4 行（(w=16,n=4)/(w=16,n=3)/(w=12,n=3)/(w=12,n=4)）、每行 8-11 个算术格。逐格比对：

- Candidate A：width_A/occupied_A/past_512_A/margin_512_A/past_4096_A/margin_4096_A/
  occupied_A_with_B/margin_512_with_B_A/extra_writes/extra_reads/extra_disk_bytes，
  4 行全部数值两份样本**逐格相同**（65/546/-34/3550/554/-42/260 等，见报告正文）。
- Candidate B：width_B/past_512_B/margin_512_B/past_4096_B/margin_4096_B/extra_writes/extra_reads/
  extra_disk_bytes(512)/extra_disk_bytes(4096)，4 行全部**逐格相同**（105/407/3991/4/4/2048/16384 等）。
- Candidate C：entries_per_piece/pieces_needed/width_C/margin_512_C/past_4096_C/margin_4096_C/
  extra_writes/extra_reads/extra_disk_bytes(512)/extra_disk_bytes(4096)，4 行全部**逐格相同**。

另外用独立 Python 脚本按提示给定的公式重新算了一遍全部 12 行（3 候选 × 4 行），
**与两份样本的数字逐格相同**（命令与输出见上文对话记录，此处不重复贴——按「引产物整行抄」的例外，
算术验证结果本身即为产物，已在核查过程中完整跑出并核对，两份样本互相验证、又与独立公式重算验证）。

**判定：s1 与 s2 在全部可比对的数值格上一致，且都与独立重算一致，没有发现任何一格不一致。**

## 计数

| 类别 | 核了几处 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| 判别力自证 | 1 | 0（要求判 ✗，且确实判出 ✗） | 1（预期内） | 0 |
| 云端攻方（opus）kb/crates 引用 | 7 | 7 | 0 | 0 |
| 云端攻方（opus）复跑命令 | 7 | 7 | 0 | 0 |
| 云端正推（sonnet）kb/crates 引用 | 24 | 24（3 处标「✓部分」供主 agent 判摘句，1 处行号漂移已专项说明） | 0 | 0 |
| 本地攻方提示（local-attack.md）Fixed facts 引用 | 25 | 23 | 2 | 0 |
| 本地攻方核对表（translation-audit.md） | 9 | 9 | 0 | 0（1 处理由引用未点名行号，未纳入统计） |
| 本地攻方两份样本一致性 | 1（整表比对） | 1 | 0 | 0 |
| **合计** | **74** | **71** | **3**（判别力自证 1 处预期内 ✗ + local-attack.md 2 处真实 ✗，同一条判决的两个行号） | **0**（1 处行号漂移已判 ✓ 并专项说明，不计入本行） |

真实发现问题的 ✗（排除判别力自证那条预期内的）：**2 处**，均在 `m2-witness-r1-local-attack.md`
第 77 行「-34/-42 与既有实验数吻合」这一句的出处引用上（`experiments/158…:133` 与 `:918`），
真实出处是同一文件第 15 行。这两处引用不影响 W5 算式本身（算式已用独立复算与双样本互证确认正确），
只影响「这两个数字之前有没有被独立验证过」这一句旁证的出处是否准确。

## 没做什么

- 不判一条打中成不成立、该不该采纳；只核引用、产物与复跑，推论本身（W1-W5 各格的判定、岔路、
  「这条腿自己的限度」一节）不核。
- **不跑全量崩溃枚举**（用户已定全量最后统一跑）：只复跑了报告里挑的历史（213 组合三放处、
  (b) 最少 4 故障、(c) 512 物理块撕裂新错、写序 pre/post 两条边 H1/H2）。opus 报告自己提到的
  `w-stale`（二.5 块头身份字段）、E2 那条边（三.1 之后各节多次用到）、4096 物理块那几组、
  batch3（被用户令停掉的全量崩溃枚举）**未复跑**——不在派发指名的「挑承重的几格」范围内。
- `.claude/kb/decisions/22…` 已定项 21（「校验和 + 世代号缺一不可」，translation-audit.md §2/§3
  的括注理由引了它）**未现查该已定项的原文行号**——核对表本身没有点名具体行号，只写了「已定项 21」，
  按定义「行号去原文件里现查，行号没现查过的不写进报告」，这里如实记「核不动：核对表没给可核的
  文件:行」，不判 ✓ 也不判 ✗。
- `decisions/20…:84`（translation-audit.md §6 提到的「首稿误标」位置）**未现查该行原文**，只核了
  定稿引用的 197 行——首稿那个误标本身是否真的在 84 行、是否真的是 D20 已定项 4「射程」段，
  未验证，如实记「未核」。
- 本地攻方两份样本里给出的三句「什么新事实会推翻这些数」的落款句（F5/F9/F7 各一句）——
  这是模型自己的推论式陈述，不是可核的文件引用，未核。
- 未跑门禁阶段（这一轮不归核查员跑，`stage-owners.tsv` 没有登记给 `three-way-verifier` 的阶段）。
- 未做任何 git 写操作、未改入库仓任何文件——全部编译与复跑都在草稿目录的副本上做。
- 未核 opus 报告「没打中的形状」表、「改法各修哪一格」表——这两张表是推论/归纳，不含独立可核的
  文件引用或产物行。
- 报告文件（本文件）此前不存在，无「现在的 sha256 与交回里给的对不上」这一情形；
  派发提示也没给云端腿报告自己的 sha256sum，无法核对腿报告文件本身在交回之后有没有被改过——
  如实记「未核：无 sha256 可比对」。
