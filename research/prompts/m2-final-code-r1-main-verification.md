# 里程碑二收尾：HEAD 之后打进的四份补丁，代码轮第一轮判决（2026-09-25）

正文 `research/prompts/_m2-final-code-r1-body.md`；背景材料 `research/prompts/_m2-final-code-r1-background.md`；腿读的冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`，`src/*.rs` 的 sha256 在 `research/prompts/m2-final-code-r1-snapshot/crates-src-sha256.txt`。

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 -->

## 一、三条腿与核查员

| 腿 | 格 | 报告 |
|---|---|---|
| 云端攻方（Opus） | Z1、Z4、Z6 | `m2-final-code-r1-opus-output.md`；用例与原样输出 `m2-final-code-r1-opus-model/` |
| 云端正推（Sonnet） | Z2、Z3、Z5 | `m2-final-code-r1-sonnet-output.md` |
| 本地攻方 | 算术 | `m2-final-code-r1-local-attack-output-s1.md`、`-s2.md`，两份干净 |
| 核查员 | 全部 | `m2-final-code-r1-verifier-output.md`：208 处，✓ 180、✗ 4、核不动 20。20 处核不动是因为主 agent 派腿时只记了 `crates/` 的快照、没记 kb 的，补核见下一行 |
| 核查员补核 | 那 20 处 kb 引用 | `m2-final-code-r1-verifier-kb-output.md`：✓ 16、✗ 4。补核时各 kb 文件的 sha256 在 `m2-final-code-r1-snapshot/kb-sha256-at-verify.txt`；派腿之后只有 `invariants.md` 第 138、283 两行由主 agent 补过简称 |

**核查员复跑**：攻方承重的 Z4-1（`a5-66.log`、阳性对照 `a5d.log`）与 Z6（`a4.log`，1225 个组合）在核查员自己的副本上逐行相同。另外复跑了 `z1_a1b`、`z4_a3`、`z6_a1`，同样逐行相同。本地攻方两份样本的算术逐格重算，全部对。

**Sonnet 腿的引用不可靠**，一共 8 处 ✗：
- 代码行号偏了 4 处（`transaction.rs:1888-1892`、`:1861-1863`、`:876-878`、背景材料第 49 行），实际分别在第 1898、1857、882 行与第 48 行；
- kb 引用 4 处：丢了强调符号 2 处；D19 第 107 行行号错，那句豁免映射的原文在第 173 行；`invariants.md:59` 的「三种结局」是转述，却放在引用块里当原文；
- 自述的 grep 命中行「第 637 行」被同一条命令当场证伪，实际命中在第 199 行。

它开头那句「每条引文的行号都已核对」不成立。凡是它结论所靠的那几格，主 agent 都另外对着原文核过，逐格写在第二节。

## 二、逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| Z1 末条标志、末条再跨记录、读法乙 | **兑现了条款** | 攻方：读法乙、按末条标志认发布边界、一次发布之内三种断链、解析器两格，都照 D23（journal 的角色与格式） 已定项 4、14、17 的字面；64 片写行那次发布末条跨成两条，第二次挂载的 164 个前缀崩溃状态加 16 个记录写子集全绿 | — |
| Z1 附记 | **替没写的条款做了选择** | 事务号 0 的发布跨多条记录时，第一条的提交标记写 0；读者把提交标记 2..=255 当「没有」，checker 判违例，两边不是一件事；`mount.rs:1708` 的注释还是「jsn 最大」的旧字面 | 主 agent 定：读者也当损坏、断链即止，与 D23 已定项 4 的读者规则同一处置。并给实二二三（第 i 件），连那句旧注释 |
| Z2 释放前读盘核与隔离 | **兑现了条款**（N1 读盘核、只重读一次；N3 核出坏的留在已分配） | Sonnet 核 D19（块指针的结构与宽度预算） 已定项 5；它引的原文丢了强调符号，主 agent 对原文核过，意思不变；实例表豁免映射的原文在 D19 第 173 行 | 只一块盘那一份坏时整体拒绝：已知，用户 2026-09-25 定「两块盘一起留」，实二二三第 c 件 |
| Z3 实例表多片写路径 | **兑现了条款** | 369 行一片、尾片先、整条链 COW 重写、取号前按片数准入并逐片核旧链，对 D18（块里携带什么信息） 已定项 11、D3（空间分配） 已定项 10 ⑤。两份本地样本与核查员重算一致：一条记录装 67 项，24355 行（67 片）起树表 0 条那一版的写行记录装不下 | 装不下那一格：已知，照 D23 已定项 17 接再跨记录，实二二三第 f 件 |
| Z4-1 取号前准入（新） | **替没写的条款做了选择** | 树表 0 条那一臂，写行那次发布的分配记录条数准入在取号之后才判。实例表 40 片起，每试一次可写挂载就烧一个实例代号，66 片时第 6 次挂载起连拒 24 次、号每次涨 1；带文件那一臂同一格在取号之前拒、号不涨。核查员复跑逐行相同。许可它的 `mount.rs:1151` 那一句，前提自 C512 起已不成立 | 挪到取号之前：实二二三第 h 件，攻方用例当验收。挪完只做到「不烧号」，池照样挂不上可写：那是分配记录树的容量墙，新立欠账 C544，归实二一的分配记录树新结构 |
| Z4-2 至 Z4-5 | 已知 | 抬 F 已落盘次数少报一次（C381，实二二三第 a 件一并看）；抬 F 失败后同一进程接着写（原样重发，实二二三第 a 件）；拷贝上释放核验报错照样取号（实二二三第 h 件）；拷贝上不读盘核（C542，用户定记欠账） | 各自已有去向 |
| Z5 checker 的 I-9.15、I-7.9 与重建 | **兑现了条款**；`rebuild_version` 只读实例表第 0 片是**替没写的条款做了选择** | I-9.15：`walk.rs:709-717`。I-7.9：主 agent 对着 `invariants.md:59` 原文与冻结副本 `walk.rs:2229-2248` 逐句核过：F 落在两个上限之间跳过不判，其余按「不高于较低的上限」判，与原文「F 不高于把这些根全算空得出的上限就成立……落在两者之间这条根不判」一致。Sonnet 引的那段是转述，主 agent 没有拿它当依据 | `rebuild_version` 只带第 0 片：C543，用户 2026-09-25 定记欠账不改 |
| Z6 三份补丁的合并点 | **没打中** | 回退到根环里每一条可读根（含最旧那条与 mkfs 的第 0 代根）× 一片 / 两片 × 带文件 / 树表 0 条 × 挂载 1..=8 次，共 1225 个组合全绿，核查员复跑逐行相同。新种子随机历史三组，每组 1500 个种子 × 60 步，没有新发现，核查员没有复跑这一组。只抽了一次样，不拿来撑「没问题」 | — |

## 三、要落的

1. 实二二三加第 i 件：提交标记 2..=255 由读者当损坏，改 `mount.rs:1708` 那句旧注释。已发给实二二三。
2. 实二二三第 h 件的验收：以攻方 Z4-1 那条历史为准，改完必须在取号之前拒、实例代号不涨。已发给实二二三。
3. 新立 C544（树表 0 条那一版写行，分配记录条数撞墙之后池挂不上可写）：归实二一，随下一批写回。
4. 流程：代码轮的开工快照要连 kb 文件一起记，记在 `records/2026-09-16-subagent拆分提案.md` 第四十节第 21 行。Sonnet 腿引用行号不可靠这件事，判决里逐格另核过；是否要改正推腿的定义，放进收尾第 6 步看。

## 四、这一轮点名的 `crates/*/src/*.rs`

HEAD 之后、冻结副本里改过的 19 份都在这一轮的射程里：
- `crates/singlefs-checker/src/image.rs`
- `crates/singlefs-checker/src/lib.rs`
- `crates/singlefs-checker/src/walk.rs`
- `crates/singlefs-core/src/allocator.rs`
- `crates/singlefs-core/src/instance_table.rs`
- `crates/singlefs-core/src/journal.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/mounted_read.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/write_accounting.rs`
- `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`：E158 的实验装置，由实验页的单测与变异表判，这一轮不判
- `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`
- `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`
- `crates/singlefs-harness/src/fault_injection.rs`
- `crates/singlefs-harness/src/history.rs`
- `crates/singlefs-harness/src/model.rs`
- `crates/singlefs-harness/src/model_comparison.rs`
- `crates/singlefs-harness/src/scenario.rs`

冻结副本之后打进的实十九、实二四、实二四续，以及还在做的实二二三、实二一，它们改过的文件归下一轮代码三方。

## 回看决策

不涉及决策：代码轮只核实现与已定条款是否一致。Z1 附记那一格由主 agent 按 D23（journal 的角色与格式） 已定项 4 已有的读者规则同一处置补齐，没有推翻或确立任何分项。
