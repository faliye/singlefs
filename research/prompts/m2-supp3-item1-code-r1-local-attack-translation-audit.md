# 核对表：`m2-supp3-item1-code-r1` 本地攻方提示的逐句转述核对

提示文件：`research/prompts/m2-supp3-item1-code-r1-local-attack.md`。行号在下面都是原文件（`crates/singlefs-core/src/*.rs`、`crates/singlefs-harness/src/history.rs`、提示文件自己）里现查所得，不是背景材料里的行号。表头：英文项 / 原文文件:行 / 首稿缺的 / 定稿。

## 一、Z5（读路径）背景事实

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| "SECTOR_BYTES equals 512" | `crates/singlefs-harness/src/crash.rs:26`（`pub const SECTOR_BYTES: u64 = 512;`） | 无（常量直抄） | 保留 |
| old_read 伪代码逐行 | `crates/singlefs-harness/src/crash.rs:36-40`（`read` 旧体，改前逐扇区 `get`） | 首稿容易漏掉「找不到就维持初始的 0」这一分支（旧代码没有 else 分支，是隐式的：`out` 先全 0，`if let Some` 命中才覆写） | 补成显式的 else 分支「leave output's bytes at that position as 0 (they already are)」，与旧代码「先全 0、命中才覆写」的隐式行为对齐 |
| read_into 伪代码逐行 | `crates/singlefs-harness/src/crash.rs:41-60`（`read_into`：`buffer.fill(0)` 之后 `self.sectors.range(...)`） | 首稿容易把 `range` 说成「扫描整个 map」，漏掉「range 只按区间取，不遍历区间外的 key」这一点，会让第 8 格（大空洞）的效率论证落空（虽然效率不是这次要判的，但正确性论证要靠「range 只交回落在区间内的键」这一句） | 补一句「for each (sector_number, bytes) pair that is actually present in the sparse map and whose sector_number satisfies first_sector <= sector_number < first_sector + sector_count」，把 `range` 的语义拆成显式的存在性 + 区间条件两部分 |

## 二、Z1 七个入口：Doc text（每行 = 入口自己的文档注释）

| 英文项（提示文件行号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Z1-A Doc（提示第 79 行） | `crates/singlefs-core/src/transaction.rs:1131-1134` | 首稿容易只译「txg 与 jsn 写死 3」就停，漏掉「只接得上 txg 2、jsn 2 的那一版（两次暖机之后）」这个精确前提，也漏掉「一个字节都不写」这半句（决定这条路径失败时是不是无副作用） | 补回「This entry point only accepts being called on the version whose checkpoint number is 2 and whose journal sequence number is 2, that is, the version that exists right after the second warm-up publish」与「and writes zero bytes to the device」 |
| Z1-A Doc（同上） | 同上 | 原文「其余同 `publish_version`」与 m2-emptypool-nonempty-r1 攻方腿 Z3-A 的括注，与本行要判的前提差异无关 | 定稿故意不抄这两句（不是漏抄，是判定它们与「文档前提 vs 生成器前提」这一问无关），本表如实记这一条不抄 |
| Z1-B Doc（提示第 83 行） | `crates/singlefs-core/src/transaction.rs:1185-1189` | 首稿容易漏掉「对象出生代与容器身份不改」，这半句界定「覆盖写」预设的正是「已有一个文件在」 | 补回「the file object's birth generation and its container identity do not change」 |
| Z1-C Doc（提示第 87 行） | `crates/singlefs-core/src/transaction.rs:506-511` | 首稿容易照抄「块设备报的错原样交回」就停，不主动指出这份 Errors 没有列「现行版本带文件时调用会怎样」——这个「没写」正是这一行要判的差异的一半 | 补一句事实陈述「its documentation does not state, and does not list as an error, what happens if it is called while the current version already carries a file」；同时补一句「Elsewhere in the source, one settled decision item is cited by name…」，把「树表 0 条 ⇒ 零单元」那句决策引用（D16 已定项 9）转成一句不带编号的英文陈述 |
| Z1-D Doc（提示第 91 行） | `crates/singlefs-core/src/mount.rs:1089-1093` | 首稿容易漏掉「只做过 mkfs 的池」那一整条分支路径（取号 1、不写行） | 全句保留；另加一句事实陈述「Nothing in this function's documentation states a precondition about what in-memory session state the caller must be in before calling it」（原文注释确实没有这句，是核对本行文档字面之后加的「没写」陈述，见下方「多出来的」） |
| Z1-E Doc（提示第 95 行） | `crates/singlefs-core/src/mount.rs:1164-1170` | 首稿容易把「目标根不在根环、不在候选集」揉成一句「target invalid」，丢掉「根环」与「候选集」是两个不同集合这件事，而这正是本行后半要判的关键 | 保留两个分句为两句独立错误条件；见下方「多出来的」一行，补了一句解释性总结 |
| Z1-F Doc（提示第 99 行） | `crates/singlefs-core/src/mount.rs:590-591,594` | 首稿容易漏掉「这里是只供测试的强制入口」（`.claude/rules/fs-design.md` 五条硬要求第 2 条）这半句，它说明这个入口本身不是按「正常前提」调的 | 补回「and that this entry point itself is a forced entry point that exists only for testing purposes」 |
| Z1-G Doc（提示第 103 行） | `crates/singlefs-core/src/recovery.rs:1334-1336` | 首稿容易只抄注释原句，不点出「这个函数不像前六个入口那样返回 Result」——这一句不在注释原文里，是读函数签名（`-> RecoveryReport`）才看得出来的事实 | 补一句「This function does not return a Result type the way the other six entry points above do; it returns a plain report value」（见下方「多出来的」） |

## 三、Z1 七个入口：Generator text（每行 = `history.rs` 里生成器调用前的守卫代码与注释）

| 英文项（提示文件行号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Z1-A Generator（提示第 80 行） | `crates/singlefs-harness/src/history.rs:1219-1221`（`apply_publish_first_file` 只判 `session.as_mut()`，函数体内没有对 `session.current` 的分支） | 首稿容易只说「生成器只查会话开没开」，不举证「即使现行版本带文件，比重表仍会抽到这一步」——没有这一句，「没有检查」这个结论没有可核的数字撑着 | 补一句「(the operation weights document literally lists it at 4 percent of draws in that state)」，数字取自 `crates/singlefs-harness/src/history.rs:286-292`（`operation_weights` 的 `ExpectedSession::OpenWithFile` 分支，`PublishFirstFile` 权重 4，行 292），见下方「多出来的」 |
| Z1-B Generator（提示第 84 行） | `crates/singlefs-harness/src/history.rs:1259-1264` | 首稿容易漏掉两个检查的先后与各自失败时的具体理由名字（`NoWritableSession` 与 `CurrentVersionWithoutFile`），只说「查了两条」 | 补回两个具体的 `MissingPrecondition` 成员名字，与 `crates/singlefs-harness/src/history.rs:526,528` 的两行注释对齐 |
| Z1-C Generator（提示第 88 行） | `crates/singlefs-harness/src/history.rs:1298-1304`，注释见 `crates/singlefs-harness/src/history.rs:1301` 与枚举成员注释 `crates/singlefs-harness/src/history.rs:529-532` | 首稿容易把「没有条款明写调用约定，按入口的文档注释取的读法」这句自陈漏掉，只留「生成器只在树表 0 条时调」——漏了这句，模型看不出这条前提本身没有决策条款撑着 | 补回「this precondition…is not written out explicitly in any settled decision item; it is the generator author's own reading, taken from this entry point's documentation comment」，直译自 `crates/singlefs-harness/src/history.rs:530-531`「没有条款明写调用约定，按入口的文档注释取的读法」 |
| Z1-D Generator（提示第 92 行） | `crates/singlefs-harness/src/history.rs:1361-1364` | 无实质遗漏：这一条本身只有三步（关会话、计数、调用），首稿与定稿一致 | 保留 |
| Z1-E Generator（提示第 96 行） | `crates/singlefs-harness/src/history.rs:1372-1390`（三种取法：`RingRoot{0..4}`、`RingRoot{% root_count}`、`BeyondNewestRoot`） | 首稿容易把三种取法的比例说成「随机挑一种」，丢掉「各约五分之二、五分之二、五分之一」的具体配比（源码 `source.below(5)`：0、1 两个值走第一种，2、3 走第二种，其余（4）走第三种，各约 2/5、2/5、1/5） | 补回三段具体的比例描述（"about two fifths… about two fifths… the remaining time…"），与 `crates/singlefs-harness/src/history.rs:342-357`（`draw_operation` 里 `CloseAndMountRollback` 分支）的 `source.below(5)` 五取值对齐 |
| Z1-F Generator（提示第 100 行） | `crates/singlefs-harness/src/history.rs:1419,1421` 与字段注释 `crates/singlefs-harness/src/history.rs:183-188` | 首稿曾多写一句「and the entry point's documented error list is exactly what is expected to reject it」——这句原文没有，是核对时自己加的推论、且预先替模型答了「差异是不是良性」这一问；核对时发现即改，用 `research/scripts/replace-once.py` 删掉这句，只留「a target above the ceiling is reachable by construction from this formula」 | 已删（2026-09-18，`replace-once.py` 命中 1 次、回读确认） |
| Z1-G Generator（提示第 104 行） | `crates/singlefs-harness/src/history.rs:1448-1452` | 无实质遗漏 | 保留 |

## 四、英文比原文多出来的限定词、括注（连同为什么加）

| 提示里的英文 | 为什么加 |
|---|---|
| Z1-A Generator「(the operation weights document literally lists it at 4 percent of draws in that state)」 | 让「没有检查」这个结论落到一个可以现查的数字上，不是空口说没查；数字来源见上表；权重条目本身在 `crates/singlefs-harness/src/history.rs:292`（`(HistoryOperationKind::PublishFirstFile, 4),`），所在分支 `ExpectedSession::OpenWithFile` 起于 `crates/singlefs-harness/src/history.rs:286` |
| Z1-D Doc「Nothing in this function's documentation states a precondition about what in-memory session state the caller must be in before calling it」 | `mount_writable` 的文档注释原文确实没有这一句；这是核对文档字面之后得出的「没写」这一事实陈述，不点破它是否等于「无前提」，只说文档没写，留给模型自己去比较两栏 |
| Z1-E Doc「Note that 'not in the root ring' and 'not in the candidate set' are listed as two separate error conditions, not one: a root can be in the ring yet not in the candidate set」 | 原文本身已经把两者列成两个分句（「不在根环、不在候选集」），但没有明说两个集合不是一回事；不点破这一句，模型容易把「候选集」读成「根环」的同义替换，而 Z1-E Generator text 里恰好有一种取法专门落在「在环里但不在候选集」的根上——这句是为了不让这个区分在转述里消失，不是替模型下结论说两者有没有差异 |
| Z1-G Doc「This function does not return a Result type the way the other six entry points above do; it returns a plain report value」 | 原文注释本身没有这一句；这是读函数签名（`recover` 返回 `RecoveryReport`，不是 `Result`）之后补的事实，因为这一行的「文档前提」栏如果只抄注释原句会显得空洞——点出「连返回类型都不一样」是为了让模型看到这一行注释本来就没有「错误」这个概念可放前提 |

## 五、没有走翻译核对的部分

Z5 的伪代码与事实表数字（偏移、长度、已写扇区集合）不是转述自中文条款，是本轮为了让 Z5 可逐格判而现造的测试点，来源是 `crates/singlefs-harness/src/crash.rs:36-60` 的两段函数体逻辑与 `crash.rs:26` 的 `SECTOR_BYTES` 常量，不是决策文字，因此不适用「转述核对」，适用的是「伪代码与源码逐行对照」，已并入第一节。
