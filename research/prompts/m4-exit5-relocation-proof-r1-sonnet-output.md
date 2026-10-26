# m4-exit5-relocation-proof-r1　云端正推腿报告（Sonnet）

轮：m4-exit5-relocation-proof-r1。立场：云端正推，格 X1–X4。冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`（只读，引的代码行号都是它的；`compaction_plan.rs`、`mounted_session.rs` 两份与仓里此刻的工作区版本不同，其余引到的 `crates/` 文件逐字节相同，开工时 `cmp` 过）。

## 复跑

模型目录 `research/prompts/m4-exit5-relocation-proof-r1-sonnet-model/`。整份复跑（含 unit-graph 构建、七次 verus、八行必红世界、七个探针、`cargo check -p singlefs-core`，跑起来之后几分钟（没计时），排队另算）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-exit5-relocation-proof-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

每一步的输出去掉耗时后与 `out/` 下的参照逐行比，对不上的件数就是退出码。下面是模型目录里每个文件的 sha256：

```
ab8c3a1a876b16c90046ded9ffb2464eb2fcbfba19fea60a2ba41c265ff1494e  out/closure-after.txt
b0707bb8c1bd41d4a95506ffa257fe819a6e61779dddcf6d3196506f1fbfc8fb  out/closure-base.txt
0f195d96df743c114c4d8ffcd3ce09d9e179d0ff4520071c8d5f148ce97d9264  out/constructs.txt
64b58b883af5639ff20623b5e0a30487020a4614e6a17b38aead9344dbf6fa0b  out/core-check.txt
801b0f0f9471648f7ca7eaa9338871d31cfb59ee654627c7c54ecef7c42e3fb0  out/probes.txt
e363718982d4a526b31369421d93e086c44137159500fdfac9ce71657940cf32  out/red-worlds.txt
01820a034fabf08b992213ad9dba7f18e88f456f9d309c66074adacde1018815  out/verus-after.txt
212d036750b12b9a29bd05af345c2470b6654f120b3dcecef731b6e197bc90e0  out/verus-base.txt
9fb6e9b9a0dbdde7b7a9f27ee8c2f9246a99ce1d5ce64d7e6d84096e8ecbd050  patches/core-glue-item1.patch
96eaecd25e8a8bc5ff06c45d324e374126a25060e2b91b4a4d9a9663f26da6b2  patches/lib-mod-line.patch
5bd708ccfeab038fe450c64242939cf5677edd2b680d481fd7e3cf0dd7ef63bb  probes/p1_array_iter_all_closure.rs
f0fa0799578ba69e97f479d80ebcc9eb6c655a84833ae2c57aedf39c5581e060  probes/p2_btreemap_get.rs
e4acbc1b0746bf4ce379f44c60e33ade232765ebb7bbc0bbd228acdd4939f3c9  probes/p3_dyn_trait.rs
519eb6acb803061eaf1906a9a5dae5e2ab3ead6e13a500f120af03a979415763  probes/p4_try_from_expect.rs
85f25f6abebcb75182fae41d1bc1bd585249bbd399ec1529dc36f99a914a2941  probes/p5_question_mark_result.rs
cba8303924739f2600208ebec3e3f0f2273384d01892ab23618d94642fd35e80  probes/p6_for_loop_over_slice_iter_with_generic_device.rs
51890ad883cc58f9a2fb3a727e531079c56b350d96ea4633d26820ce87b4997c  probes/p7_btreeset_collect_closure.rs
14a1ccec523541ec73d6e2bb5c7d57b1cf1f90b23f79487df9a23468cf0ac67e  src/relocation_execution_proofs.rs
5045efcdbd3dfd570180643dde6a60016e81b230c4ac336fd8e1ae531dbee512  tools/closure_constructs.py
54422dd63a6d9f5b178035d9c64b820780e94c37613fc7d083793b48326ef9f9  tools/closure_stats.py
9c515a32c402ae19b6d1b8c64087f7154dd4d2444aeabaa08e1b19e961f30a57  tools/closures.sh
b66ab096a4c4b9176155e93bf22017eb706c285f3a2b28ef8dd784b164f9ae47  tools/probe.sh
b51d437c43219337bc640812574a5f84011f1d234bafe280525c8d0dc298094d  tools/red-worlds.py
6da6d7f705ca7ef7313a98e2cbf318fb1fab3ae9d8692f43b6e8501648396a84  tools/red-worlds.tsv
31acab77d10a53079dd58ec44a8c406cbf4943115d1b6e469cb631b95325ce4b  tools/rerun.sh
c74e72b100896b6891cc8e0328ac11b89dcf2023d96e749a14b5cb2997163d1a  tools/verus-run.sh
```

冻结副本里被引文件的 sha256（开工时量的）：

```
16b93d7c926702e231edabca82958fe12fbf757339160beff36fe443972d7210  crates/singlefs-core/src/transaction.rs
9aea28b615923d67f0f7c8dd1f8c6811a3046722b423cb3b92c8e7b1993f5109  crates/singlefs-core/src/recovery.rs
7852ff4a738970c2f98485728659edf0b6c63b82fc2d077358642a3308d006ed  crates/singlefs-core/src/records.rs
b5d213cb6b74baba2ac71247a521de0caf3cc90d66985b0a676aeae3521aade2  crates/singlefs-core/src/compaction_plan.rs
fe8452c4092ffc5826391312e4575bc1994f4296c4340bb3a29dde41cf9527c1  crates/singlefs-replay-prefix/src/gc_crash_proofs.rs
274e57685cfe504d53daee9be83c66f081de1a9b40c075d233fcec723d4acdfd  crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs
0d2868ce0cadc4499cd4aa80718a8625bcef89e8d2de6cea90c6154c05129933  crates/singlefs-replay-prefix/src/lib.rs
```

## 〇　结论摘要（正推腿的推荐，不是判决）

- **X1**：Verus 里的「抽象文件系统状态」取 `gc_crash_proofs` 的 `logical`：映射 key 到字节，当前根、持久根、每个留存读者三类视图读到同一份。P4 的「每个 key 经映射读到的字节」是它当前视图那一支，不是另一个定义。O1 的「POSIX 可见内容」留给出口 5 后半句的对拍。从 `logical` 到 O1 之间有一座桥（解码只经映射读单元、位置提示不当权威），Verus 不证，写成点名前提。
- **X2**：用户那一句最弱读法是 (a)——core 里做判定的纯函数、由 core 调；最强读法是 (b) 加 (c)。推荐罩到「(a) 全部加 (c) 的抽象写序那一半」：一次发布按持久顺序的每个前缀、每个崩溃点上，所有视图读到的字节不变，对批大小 K、搬几批、槽值、字节、读者数任意；盘数与多槽 span 这一次不罩。
- **X3**：推荐叠法 = 核外提的条目判定与目的地判定（由 core 调）+ 写序前缀定理（只吃那两个判定的前提）+ 现有的 I-1.14（N6）、崩溃点全量枚举、O1 对拍罩胶水。原型在副本上验过：整个 crate 88 → 102 条 verified、0 错，八处改坏全红；接上一个判定后 `publish_relocation_batch` 闭包核内占比 12/18057 行（0.07%）→ 50/18110 行（0.28%）。core 直验在今天的门禁下走不通（`Option::copied` 这类常用调用要 `assume_specification`，门禁禁；下界 836 个 fn）。
- **X4**：给了 G7、出口 5、D13 新立一项、`verification.md` 的改写句；K 与 `gc_crash_proofs` 自己列的六样未连接前提这一次逐样定罩不罩（4.5）。
- **这套推荐没有做到的**：core 里复制字节、取目的地、重写映射树与父节点的位置提示、拼写序、设备，仍是没被证明的 Rust。按用户 2026-10-26 那一句的最强读法（b）它不够，选哪种读法是用户的事，报告第五节把它列成岔路。

## 一　X1　「抽象文件系统状态」取哪一个

### 1.1　三个定义各说什么（读源码，行号都是冻结副本的）

| 定义 | 状态空间 | 「GC 恒等」说的是 | 出处 |
|---|---|---|---|
| P4 | 映射 key→落点，加每个落点上的字节；抽象状态 = 每个 key 经映射读到的字节，**只有当前一份视图** | 搬一批（源字节原样写到目的地、映射条目换到目的地）之后，每个 key 读到的字节不变 | `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:488`、`:490`、`:735`、`:897` |
| `logical` | 同样是映射 key→字节，但同时量化当前根、持久根、每个留存读者三类视图，且要求每个视图的 key 集合等于 `logical` 的 key 集合 | 七种协议步骤（写、搬、存根、取读者、放读者、回收、崩溃）任意有限历史之后，三类视图读到的仍是同一份 `logical` | `crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:47`、`:57`、`:128`、`:187` |
| O1 | 内存模型的可观察读回：根键加第一个文件内容、快照读回、deadlist、每个角色的分配代 | 同一历史穿插整理与不穿插整理，每一步读回逐项相同 | `crates/singlefs-harness/src/model.rs:2586`、`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs:1-9` |

读出来的关系：

1. P4 与 `logical` 的类型是同一个——都是 key 到字节的函数。`gc_view_matches(current, disk, logical)` 就是 P4 的「每个 key 经映射读到的字节等于抽象值」，多要了「key 集合相等、目标存在」。所以第一个与第二个不是并列候选，`logical` 严格强于 P4：它多出持久根、留存读者、崩溃。
2. O1 在 `logical` 之上多套了一层解码：从 key→字节还原出 inode、extent、目录、快照可见内容。出口 5 后半句「带 GC 的模型对拍与不跑 GC 逐项相同」比的只能是 O1 这一层——对拍拿得到的是读回，拿不到 key→字节。
3. 所以「哪一个对得上后半句」答 O1；「哪一个能在 Verus 里与 core 的数据结构接上」答 `logical`：O1 的解码在 core 里是树的读写（`recovery.rs`、`mounted_read.rs`），Verus 不到；`logical` 的 key→槽→字节能通过一层薄适配和 core 的 `Placement`、`[LocationEntry; 2]` 对上（先例：`crates/singlefs-core/src/compaction_plan.rs:261` 的 `compaction_kernel_allocations` 把 `Vec<u8>` 的映射 key 换成核的 `u64` 身份，保留相等关系）。

### 1.2　读代码读出的三处，决定「恒等」不能直接说成整个版本的字节不变（冻结副本）

- **一次整理发布不只搬迁**：同一次发布还重写记账树、分配记录树、logged_ops 根、树表（这些是派生态或固定点）。`settle_the_allocation_record_tree` 里 `relocations_in_effect` 把「本来就被这次发布重写」的角色从搬迁里滤掉（`crates/singlefs-core/src/transaction.rs:11766`），它们走 COW 换新落点、换新出生代，映射 key 里带出生 txg（`crates/singlefs-core/src/records.rs:595`）；这次发布重写的角色点名项的出生代取这次的 txg（`crates/singlefs-core/src/transaction.rs:14849`），搬迁副本取原来的出生代（`crates/singlefs-core/src/transaction.rs:13330`），所以重写的角色是新 key（读代码推的，没跑）。`logical` 固定不变的前提在这些角色上不成立。**结论**：Verus 的恒等性质只能说「被搬的那几个 key 的字节与所有视图读到的内容不变」，不能说「整个版本的 key 集合与字节不变」；整个版本的内容不变是 O1 那一层的事。
- **父节点里存着位置提示**：搬走一个单元之后，没被这次发布重写的父节点里仍是旧提示：「盘上没重写的父节点里仍是旧提示」（`crates/singlefs-core/src/transaction.rs:3742`）。读路径先按提示读、读不出才回退中央映射（`crates/singlefs-core/src/recovery.rs:761`），「读得出」的判据是整单元校验和：「if crc32_castagnoli(&bytes) == location.unit_checksum {」（`crates/singlefs-core/src/recovery.rs:702`）。所以「读到的字节不变」依赖两件事：映射是权威、提示读靠校验和拒错单元。校验和碰撞抗性不在 Verus 里（D13 已定项 14，`.claude/kb/decisions/13-验证路线.md:387`）。
- **N6 已经是 `logical` 的运行时形态**：I-1.14「同一映射 key 在各版里的单元逐字节相同」（`crates/singlefs-checker/src/image.rs:364`；登记行 `crates/singlefs-checker/invariants/01-块自描述.md:20`）在回退候选集里任两版上判同一把 key 的字节相同，就是 `gc_view_matches` 跨版本的那一条。

### 1.3　推荐与理由

- 推荐：**Verus 侧取 `logical`**（`gc_all_views_match`），把 P4 当它的当前视图推论；O1 留给对拍。桥（`logical` 到 O1）写成点名前提：① 解码只经映射读单元；② 位置提示读出的字节要过整单元校验和；③ 被这次发布本来就重写的角色不在恒等的范围里（第 1.2 第一条）。三条各自的罩：① 靠 O1 对拍与 I-1.13（扫描重建）；② 靠 `recovery.rs:761` 那条回退路径的现有用例与崩溃点全量枚举；③ 靠 O1 对拍里「每个角色的分配代都与搬之前相同」。
- **推翻它的现象**：(i) 找到一条读路径不经映射、不验校验和就按父节点的提示取字节，且 GC 之后可达；(ii) O1 对拍找到一个种子，I-1.14 判绿而读回不同；(iii) 反过来，找到 `logical` 恒等不成立而 O1 读回相同的历史（说明取 `logical` 太强）。本腿只做了读代码，没有跑对拍。

## 二　X2　「罩到 core 搬迁执行」罩到哪一层

### 2.1　用户那一句，最弱与最强读法

出口 5 现行原文整行（`.claude/kb/milestone/04-fourth-txn.md:20`）：

```
| 5 | 「GC 在抽象文件系统状态上是恒等变换」在里程碑三七·1 选定的工具上证出；带 GC 的模型对拍与不跑 GC 逐项相同。用户 2026-10-26 定：证明要罩到 core 的搬迁执行（`publish_relocation_batch` 那一路），抽象模型的证明加模型对拍不算证出 | 二 |
```

- **最弱读法 = (a)**：被证明的判定就是 core 在 `publish_relocation_batch` 这一路上调用的那份代码——沿用「Verus 跑真实代码」（`crates/singlefs-replay-prefix/src/lib.rs:2`）的先例。量的：冻结副本里 `publish_relocation_batch` 闭包 836 个 fn、18057 行，核内 2 个 fn、12 行，就是 `released_record_can_be_reclaimed` 与 `records_at`（命令与原样输出见第三节 3.1）。选段与计划两个核只在 `run_compaction_on` 的闭包里（19 fn、356 行），不在发布这一路。**所以连最弱读法，发布这一路今天也没做到。**
- **最强读法 = (b) 加 (c)**：Verus 直接验 core 的可执行代码，并证它写出的盘面经崩溃恢复读到的东西。
- 用户那一句的后半（「抽象模型的证明加模型对拍不算证出」）排除的是「证明只在 `GcCrashState` 这类协议模型上、与 core 代码没有调用关系」。它没有说 (b)。本腿的读法：**必须有一段被证明的代码是 core 在这一路上真调的**，是下限；(b) 是上限。

### 2.2　证明要罩的维（D13 已定项 1 射程）

射程原文（`.claude/kb/decisions/13-验证路线.md:69`）：「**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维崩溃点重放已全量枚举。」GC 流的枚举深度（`.claude/gate.d/crash-point-replay-items.tsv:24`）：「多记录流 → 重挂 → 整理签发 → 一批 → 最后一批，40 段（里程碑四二·3）」。逐维：

| 维 | 崩溃点重放取得到吗 | 本腿建议证明罩不罩 | 原型里谁罩（模块 `relocation_execution_proofs.rs`，行号是模型目录里那份） |
|---|---|---|---|
| 批大小 K | 取不到：流里一批一个固定大小 | **罩，K 任意** | 定理 `theorem_a_relocation_publish_keeps_every_view_at_every_durable_prefix`（`:382`），K = `keys.len()` 不设界 |
| 搬几批 | 取不到：固定「一批 → 最后一批」 | **罩，任意有限历史** | 现有 `theorem_gc_every_finite_crash_history_preserves_reads`（`gc_crash_proofs.rs:187`）；原型一批的前缀定理把一批嵌进去 |
| 槽值 | 取不到：只走流里那几个槽 | **罩，u64 全宽** | 全部 `Map<u64, …>` |
| 载荷字节 | 取不到 | **罩，任意 `Seq<u8>`** | `Write { bytes }` |
| 留存读者数与历史 | 取不到 | **罩，任意个** | `readers: Seq<Map<u64, u64>>` |
| 崩溃点 | **已全量枚举** | 不是证明的价值；定理顺带覆盖每个前缀加一次崩溃 | 同一个定理的前两个全称结论 |
| 盘数 | 取不到 | **这一次不罩**：协议态是单槽，条目判定要求两份副本在同一个槽（`:44` 的 `judge_relocation_item`） | 列进「罩不到」 |
| 多槽 span | 取不到 | **这一次不罩**：协议里槽是原子地址，只判跨度相等，不判目的地与邻居区间重叠 | 同上 |

### 2.3　推荐的层

**推荐 = (a) 的全部 + (c) 的抽象写序那一半**：

1. 把 `relocation_checked_against_the_version`（`crates/singlefs-core/src/transaction.rs:10142`）读完树之后剩下的判定、一次发布的目的地判定，写成 `singlefs-replay-prefix` 里的可执行核，由 core 调（(a)，和 `choose_compaction_segment`、`plan_compaction_batch` 同一种先例）。
2. 证一个定理：这些判定通过时，一次发布按持久顺序的每个前缀、每个崩溃点上，所有视图读到的字节不变（(c) 的抽象写序那一半）。持久顺序取 core 自己的：「for unit in &writes.units {」（`crates/singlefs-core/src/transaction.rs:1444`）单元写，屏障，journal 记录，屏障，根槽 FUA 加系统配置（`transaction.rs:1451-1465`）；搬迁副本排在这批单元的最前面：「.chain(rewritten.iter().map(|identity| {」（`crates/singlefs-core/src/transaction.rs:14830`）接在 `relocated_copies` 之后。
3. 屏障前一段的单元写取「任意次序、任意子集」，和 D13 已定项 4 的枚举域一致：「当前段每次写各取它的几态、任意组合」（`.claude/kb/decisions/13-验证路线.md:140`）。原型引理 `lemma_any_unreferenced_writes_keep_the_views`（`:232`）就是这个量词。

**为什么不选只做 (a)**：(a) 只证单个判定的函数性质（`judge_relocation_item` 的 ensures 就是它的 iff），不说明「判定通过」为什么足以让整次发布在崩溃下读不变。这一步要写序定理。
**为什么不选 (b)**：见第三节 3.3：下界 836 条义务，Verus 对常用 std 调用要 `assume_specification`，而门禁禁它。
**(c) 的真实盘面那一半不罩**：真实设备是否守屏障、journal 解码、重放从 journal 里还原搬迁，这些是前提（3.2 的罩不到清单与 4.5），不是定理。

### 2.4　这一层「罩到」的是什么、不是什么

- 罩到：core 对一项搬迁「收不收」的判定；一次发布的目的地「合不合格」的判定；在这两个判定通过加几条点名前提之下，整次发布的抽象写序在任一崩溃点读不变。
- 没罩到（胶水，仍是普通 Rust）：搬迁字节的来源与复制（`bytes: relocation.bytes.clone(),`，`crates/singlefs-core/src/transaction.rs:13324`）、分配器取目的地（`transaction.rs:11523`）、中央映射树与父节点位置提示的重写（`transaction.rs:3745`）、写序拼装（`transaction.rs:14964`）、设备。
- **推翻它的现象**：找到 core 的一条路径，真实写序不是「单元 → 屏障 → journal → 屏障 → 根」（例如搬迁副本被排在屏障之后，或一次发布拆成两个根），则抽象写序与 core 对不上；或找到一个种子，判定全通过而某个崩溃点读回变化——后者是对「前提列表不全」的反例，正是攻方腿该找的东西。

## 三　X3　办法与叠法（每个候选的数在冻结副本拷出的副本上量）

### 3.1　核内占比怎么量，基线

命令（`<模型目录>/tools/closures.sh`，每一行入口各跑一次 `unit-graph closure`，再由 `closure_stats.py` 数闭包里的 fn 单元；「核内」= fn 单元的文件在 `crates/singlefs-replay-prefix/` 下，spec fn 也算一个 fn 单元）。冻结副本上的原样输出（`out/closure-base.txt`，去掉了耗时）：

```
singlefs_core::transaction::publish_relocation_batch rc=0
unit-graph：入口 singlefs_core::transaction::publish_relocation_batch：闭包 1903 个单元（Rust 1903、WGSL 0、数据 0），拒算 0 处
fn=836 fn_lines=18057 kernel_fn=2 kernel_fn_lines=12 kernel_share_of_lines=0.07%
singlefs_core::transaction::relocation_checked_against_the_version rc=0
unit-graph：入口 singlefs_core::transaction::relocation_checked_against_the_version：闭包 500 个单元（Rust 500、WGSL 0、数据 0），拒算 0 处
fn=149 fn_lines=1646 kernel_fn=0 kernel_fn_lines=0 kernel_share_of_lines=0.00%
singlefs_core::transaction::prepare_the_version_publish_relocating_units rc=0
unit-graph：入口 singlefs_core::transaction::prepare_the_version_publish_relocating_units：闭包 1857 个单元（Rust 1857、WGSL 0、数据 0），拒算 0 处
fn=806 fn_lines=17406 kernel_fn=2 kernel_fn_lines=12 kernel_share_of_lines=0.07%
singlefs_core::transaction::persist_publish_writes rc=0
unit-graph：入口 singlefs_core::transaction::persist_publish_writes：闭包 786 个单元（Rust 786、WGSL 0、数据 0），拒算 0 处
fn=264 fn_lines=3902 kernel_fn=0 kernel_fn_lines=0 kernel_share_of_lines=0.00%
singlefs_core::mounted_session::run_compaction_on rc=0
unit-graph：入口 singlefs_core::mounted_session::run_compaction_on：闭包 2116 个单元（Rust 2116、WGSL 0、数据 0），拒算 0 处
fn=934 fn_lines=20712 kernel_fn=19 kernel_fn_lines=356 kernel_share_of_lines=1.72%
singlefs_core::compaction_plan::plan_batch rc=0
unit-graph：入口 singlefs_core::compaction_plan::plan_batch：闭包 182 个单元（Rust 182、WGSL 0、数据 0），拒算 0 处
fn=89 fn_lines=1045 kernel_fn=13 kernel_fn_lines=242 kernel_share_of_lines=23.16%
singlefs_core::compaction_plan::choose_region rc=0
unit-graph：入口 singlefs_core::compaction_plan::choose_region：闭包 200 个单元（Rust 200、WGSL 0、数据 0），拒算 0 处
fn=85 fn_lines=977 kernel_fn=4 kernel_fn_lines=102 kernel_share_of_lines=10.44%
singlefs_core::recovery::replay_journal rc=0
unit-graph：入口 singlefs_core::recovery::replay_journal：闭包 376 个单元（Rust 376、WGSL 0、数据 0），拒算 0 处
fn=193 fn_lines=4616 kernel_fn=75 kernel_fn_lines=3171 kernel_share_of_lines=68.70%
```

对背景材料的数：`publish_relocation_batch` 我量得 836 个 fn、18057 行，调查员的是 834、18027；`run_compaction_on` 核内 19 个 fn、356 行相同，总数我的 934 / 20712，调查员的 924 / 20517。差来自冻结副本比调查员的仓副本新（原因没逐文件查）。核内占比不受影响：0.07%、1.72%。
**这个占比只数「被调用的」fn 单元**：纯证明引理（`proof fn`）没有任何调用者，永远不进闭包。所以它量不出「定理罩住了什么」，只量得出「core 真调了多少被验的判定」。

### 3.2　候选一：核外提（写了原型，三样；第四样没写）

原型 = 一个新核模块 `relocation_execution_proofs.rs`（模型目录 `src/`，467 行），加 `lib.rs` 一行 `pub mod`、加 core 里一处胶水（`patches/core-glue-item1.patch`，89 行 diff）。内容：

| 件 | 对应背景材料里「四样」的哪一样 | 做了什么 | 在核里的行 |
|---|---|---|---|
| `judge_relocation_item` | ① 对上一版核 | `relocation_checked_against_the_version`（`transaction.rs:10142`）读完树之后的判定：钉住拒、两份副本的槽都等于计划源槽且跨度相同、字节非空。ensures 是四个 iff | `:44`；被 core 调 |
| `relocation_destinations_are_acceptable` | ③ 目的地约束 | 目的地两两不同、都不在受保护集里；ensures 与规格 `==`（不只是蕴含） | `:88`；**没接进 core**（见下） |
| `theorem_a_relocation_publish_keeps_every_view_at_every_durable_prefix` 与三个引理 | ④ 写序前缀对应协议历史 | 单元写（任意子集、任意次序）、换映射（每项一个 `Relocate`）、`Checkpoint`；每个前缀与每个崩溃点上 `gc_all_views_match` 仍成立；完成后 `durable == current`、每个 key 指向目的地 | `:382`、`:232`、`:263`、`:320` |
| 无前提见证 | 防空真 | 两个单元、一个持源槽的读者，满足形状要求 | `:437` |
| ② 新一版映射怎么由上一版与这一批算出 | **没写** | 映射在 core 里是中央映射树加 `mapping_entries_kept_for_the_snapshots` 加十几个 `Vec` 里的位置提示，由 `take_the_mapping_positions_over_the_location_hints`（`transaction.rs:3745`）逐处刷新；核里表达得了的只有 `destination_of_key` 那十行（`transaction.rs:13342`）。协议里它就是 `Relocate`（已在 ④ 里） | — |

**Verus 读数**（`<模型目录>/tools/verus-run.sh`，直接起 `verus --no-cheating --crate-type=lib --edition=2021 --rlimit 10`，参数同 `scripts/gate/verus-proof-check.py` 的 `verus_arguments`，没起门禁）：

```
verus rc=0
verified=88 errors=0 encountered-error=False
```
```
verus rc=0
verified=102 errors=0 encountered-error=False
```

- 现有 crate：88 verified、0 errors，与登记数相等：「crate	crates/singlefs-replay-prefix	88」（`.claude/gate.d/verus-pins.tsv:29`）。
- 加上原型：102 verified、0 errors，即 +14（模块单独验是 14，也量过）。按门禁「每个有证明义务的单元一项」数（推的，没起门禁）：6 个 proof fn、2 个 exec fn、3 个带 decreases 的 spec fn，共 11 项。带 `ensures` 的函数 8 个、循环不变式块 3 个，必红世界要 ≥ 11 行（判法 `negative_coverage_gaps`，`scripts/gate/verus-proof-check.py:2017`）。原型手工放了 8 行改坏（下表），不是入库形态。

**必红世界（手工八行，每行改坏一处、Verus 必须报证明错；`tools/red-worlds.tsv`，原样输出）**：

```
条目判定去掉钉住拒	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:47:9;src/relocation_execution_proofs.rs:48:9
目的地判定漏掉受保护集那一半	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:126:17
目的地判定漏掉两两不同那一半	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:109:17
形状要求去掉目的地两两不同	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:274:9
形状要求去掉key两两不同	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:363:20
形状要求去掉目的地未被引用	rc=1	errors=2	verified=12	落点=src/relocation_execution_proofs.rs:294:16;src/relocation_execution_proofs.rs:369:20
单元写写的是空字节	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:298:16
结论里根没指向目的地	rc=1	errors=1	verified=13	落点=src/relocation_execution_proofs.rs:400:9
```

**接上 core 之后的核内占比**（只接了条目判定；同一条命令，改过的副本）：

```
singlefs_core::transaction::publish_relocation_batch rc=0
unit-graph：入口 singlefs_core::transaction::publish_relocation_batch：闭包 1911 个单元（Rust 1911、WGSL 0、数据 0），拒算 0 处
fn=839 fn_lines=18110 kernel_fn=5 kernel_fn_lines=50 kernel_share_of_lines=0.28%
singlefs_core::transaction::relocation_checked_against_the_version rc=0
unit-graph：入口 singlefs_core::transaction::relocation_checked_against_the_version：闭包 510 个单元（Rust 510、WGSL 0、数据 0），拒算 0 处
fn=152 fn_lines=1699 kernel_fn=3 kernel_fn_lines=38 kernel_share_of_lines=2.24%
singlefs_core::transaction::prepare_the_version_publish_relocating_units rc=0
unit-graph：入口 singlefs_core::transaction::prepare_the_version_publish_relocating_units：闭包 1857 个单元（Rust 1857、WGSL 0、数据 0），拒算 0 处
fn=806 fn_lines=17406 kernel_fn=2 kernel_fn_lines=12 kernel_share_of_lines=0.07%
singlefs_core::transaction::persist_publish_writes rc=0
unit-graph：入口 singlefs_core::transaction::persist_publish_writes：闭包 786 个单元（Rust 786、WGSL 0、数据 0），拒算 0 处
fn=264 fn_lines=3902 kernel_fn=0 kernel_fn_lines=0 kernel_share_of_lines=0.00%
singlefs_core::mounted_session::run_compaction_on rc=0
unit-graph：入口 singlefs_core::mounted_session::run_compaction_on：闭包 2124 个单元（Rust 2124、WGSL 0、数据 0），拒算 0 处
fn=937 fn_lines=20765 kernel_fn=22 kernel_fn_lines=394 kernel_share_of_lines=1.90%
singlefs_core::compaction_plan::plan_batch rc=0
unit-graph：入口 singlefs_core::compaction_plan::plan_batch：闭包 182 个单元（Rust 182、WGSL 0、数据 0），拒算 0 处
fn=89 fn_lines=1045 kernel_fn=13 kernel_fn_lines=242 kernel_share_of_lines=23.16%
singlefs_core::compaction_plan::choose_region rc=0
unit-graph：入口 singlefs_core::compaction_plan::choose_region：闭包 200 个单元（Rust 200、WGSL 0、数据 0），拒算 0 处
fn=85 fn_lines=977 kernel_fn=4 kernel_fn_lines=102 kernel_share_of_lines=10.44%
singlefs_core::recovery::replay_journal rc=0
unit-graph：入口 singlefs_core::recovery::replay_journal：闭包 376 个单元（Rust 376、WGSL 0、数据 0），拒算 0 处
fn=193 fn_lines=4616 kernel_fn=75 kernel_fn_lines=3171 kernel_share_of_lines=68.70%
```

- `publish_relocation_batch` 闭包核内：12 → 50 行（2 → 5 个 fn），占比 0.07% → 0.28%。`relocation_checked_against_the_version` 自己的闭包：0 → 38 行，占比 0 → 2.24%（38/1699）。`singlefs-core` 接上之后 `cargo check` 无错无新警告（`out/core-check.txt`）。
- 再接目的地判定（没接，推的）：核里再加约 50 行 exec 和约 15 行 spec，占比到约 0.6%，仍在 1% 以下。**原因不是原型小**：闭包 18057 行里绝大部分是树构建、记账、分配记录树固定点（闭包里 `transaction.rs` 一个文件 216 个 fn、8823 行，我按 `closure-publish_relocation_batch.txt` 按文件加出来的），这四样判定不在里面。核外提抬高的是「哪些决定被验」，不是「多少行被验」。
- 目的地判定接 core 要一份受保护集（分配器里未回收的记录），原型是线性扫描，O(K·R)，产品上要换成有序结构上的二分，要多证一个有序前提（没做，没量）。
- 与背景材料里的估计比：新增 12–25 条义务——我的三样 14 条，在区间内；新增核 800–1500 行——我的三样 467 行，低于区间（第四样 ② 没写，且我没写 `Vec`/`BTreeMap` 的适配层）。

**这一候选罩不到的清单**（逐条是 core 里的位置；原型的定理把它们写成了前提或参数）：

1. 复制的字节真等于源槽上的字节。`bytes: relocation.bytes.clone(),`（`crates/singlefs-core/src/transaction.rs:13324`）；`RoleOfThisVersion` 取内存里那一版的字节、`KeptForTheSnapshots` 取盘上读到的字节（`transaction.rs:10236-10258`）。定理里它是 `Write` 的 `bytes: relocation_copy_bytes(s, key)`。
2. 目的地真没被任何视图引用。`allocate_relocation_destination`（`transaction.rs:11523`）是分配器；定理的前提 `gc_slot_is_unreferenced`，经 `protected_slots_cover_every_view`（`:145`）这条点名前提挂到「受保护集覆盖当前根、持久根、留存读者引用的槽」——就是 `gc_crash_proofs` 自己列的「floor 不超过受保护读者与根的版本」那一样。
3. 中央映射树与位置提示的重写（`transaction.rs:3745`）等于协议里的 `Relocate`。
4. 屏障生效与根槽 FUA 的原子性（D13 已定项 4 射程：真实设备的 FLUSH / FUA 归 C6，`.claude/kb/decisions/13-验证路线.md:142`）。
5. journal 编码、解码与恢复把搬迁重放成 `Relocate`/`Checkpoint`（`gc_crash_proofs.rs:18`）。
6. 被这次发布本来就重写的角色（`transaction.rs:11766`）：不在恒等的范围里（第 1.2 节）。
7. 盘数（两份副本不同槽）、多槽 span 重叠、隔离槽、共享快照（留存映射条目跟着搬）、不同逻辑版本。
8. CRC32C 碰撞：只当布尔（`bytes_available`），D13 已定项 14（`.claude/kb/decisions/13-验证路线.md:387`）。

### 3.3　候选二：core 直验（量了规模，起了七个探针，没有原型）

- 义务下界：闭包 836 个 fn 单元（3.1 的表），每个 exec fn 至少一条，循环与溢出另算，所以 ≥ 836（推的：这是 fn 个数，不是义务数）。
- 闭包里用到的写法（静态按子串扫，不是解析，一个 fn 可计入多项；`tools/closure_constructs.py`，原样输出）：

```
fn 总数=836
   48  BTreeMap/BTreeSet/HashMap/HashSet
    9  dyn trait
   34  泛型设备/读者 trait 约束
  242  闭包 |..|
  212  迭代器链 .iter()/.map(/.filter(/.collect
  287  format!/panic!/assert!/unreachable!/expect(
    0  String / format 文字
  194  Vec<
  225  u64/usize 算术（+ - * 的粗数不计，只数 checked_/saturating_/try_from）
```

- 探针：把 core 里几处写法的形状各写成一个小 Verus 文件（七个，`probes/`），直接起 `verus`（Verus 0.2026.09.20），原样输出：

```
== p1_array_iter_all_closure.rs rc=1
error: postcondition not satisfied
 --> p1_array_iter_all_closure.rs:6:13
== p2_btreemap_get.rs rc=1
error: `core::option::impl&%3::copied` is not supported (note: you may be able to add a Verus specification to this function with `assume_specification`) (note: the vstd library provides some specific
 --> p2_btreemap_get.rs:6:5
== p3_dyn_trait.rs rc=0
verification results:: 1 verified, 0 errors
== p4_try_from_expect.rs rc=1
error: precondition not satisfied
 --> p4_try_from_expect.rs:5:5
== p5_question_mark_result.rs rc=0
verification results:: 3 verified, 0 errors
== p6_for_loop_over_slice_iter_with_generic_device.rs rc=0
verification results:: 2 verified, 0 errors
== p7_btreeset_collect_closure.rs rc=0
verification results:: 1 verified, 0 errors
```

  读法：`dyn`、`?`、泛型设备加 `for` 迭代、`collect` 这几种在没要求规约时直接过（p3、p5、p6、p7）；要求规约时：数组 `iter().all(|…|)` 的后置条件证不出（p1，要自己写闭包引理）；`Option::copied` 「is not supported」，提示用 `assume_specification`（p2）；`usize::try_from(span * 4096)` 的乘法溢出要证（p4）。
- **门禁今天禁 `assume_specification`**：`CHEATING_MARKER = re.compile(r"\b(external_body|external_fn_specification|assume\w*|admit)\b|…`（`scripts/gate/verus-proof-check.py:473`，`assume\w*` 含 `assume_specification`）；并且只认一行 crate：「crate 行不止一行」（`scripts/gate/verus-proof-check.py:582`）。要验 core 就要改这两处判法和登记表，也要改 `.claude/rules/verification.md:13` 里的「Verus 验 `crates/singlefs-replay-prefix`」。
- **推翻「不可行」的现象**：门禁改成允许登记过的 `assume_specification`，或 `cargo verus` 实际编下了闭包里最大的几份文件（本腿没试）。
- 罩得到：理论上 core 的可执行代码本身。罩不到：设备、校验和（D13 已定项 14）、Verus 无规约的 std。
- **本腿的判断**：这个里程碑里不可行。依据只有上面的数（836、探针 p2 的 `Option::copied`、门禁禁令），没有量过「能编下的比例」——那要拿 `cargo verus` 编闭包里最大的几份文件，没做。

### 3.4　候选三：执行模型对拍（没新写原型；Verus 一半就是 3.2 的④）

- Verus 一半：「另写一份执行模型证它精化协议」——3.2 的 ④ 已经是这个形态：`state_after_the_copies`、`state_after_the_mapping_swaps` 是执行模型（单元写、换映射、存根的次序），定理证它保持 `gc_all_views_match`。没有再单写。
- 对拍一半：已经有。`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs`（638 行）：「整理的每一批交理想模型比（内容、快照、deadlist、每个角色的分配代都与搬之前相同）」（`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs:3`）；比的是 `ObservedReadBack`（根键加内容，`crates/singlefs-harness/src/model.rs:2586`）。它比的是 O1 那一层，不是 `logical`。
- 与 D13 已定项 13 冲不冲：定案的缓解是「让两份在表述层面就不同」（`.claude/kb/decisions/13-验证路线.md:364`），具体写法是 spec 用对镜像的状态谓词、model 用 op 到状态转移的写法（同一段下一行）。`gc_all_views_match` 是状态谓词，`gc_next` 是转移，两种写法在 Verus 里的同一个协议模型里并存，对 core 的对拍是 O1 × 实现。同源性登记表里已有一对：`对	refinement-spec-o1-model	crash refinement 的 spec（crates/singlefs-replay-prefix 的 Verus 核）	O1（内存模型对拍）的 model	已立`（`.claude/gate.d/divergence-events.tsv:15`），spec 一方就是整个 `singlefs-replay-prefix` 核。GC 的 `logical` 谓词在同一个 crate，可并入这一对，不必新立；事件靠人手记（「事件靠判出来的人手记」，`.claude/gate.d/divergence-events.tsv:11`），GC 的不一致事件进表时要写明是哪个谓词。这一点是本腿的读法（登记行没有写 GC），主 agent 核。
- 罩不到：core 与 Verus 模型之间只有对拍，界内、按取样。
- **推翻「并入现有对、不必新立」的现象**：主 agent 读同源性判法与登记行，认定 spec 一方限定在 M1–M4。

### 3.5　候选四：前提运行时核（没写；大半已经存在）

- `gc_crash_proofs` 的两条前提：`gc_all_views_match`（`gc_crash_proofs.rs:57`）与 `gc_step_enabled`（`:80`）。
- 前一条的跨版本形态就是 I-1.14（N6）：「同一映射 key 在各版里的单元逐字节相同」，回退候选集里任两版（`crates/singlefs-checker/src/image.rs:364`；`crates/singlefs-checker/invariants/01-块自描述.md:20`），在 `IMPLEMENTED_INVARIANTS` 的下标 98（`crates/singlefs-checker/src/image.rs:133`），GPU 判器同样有（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:258`）。
- `gc_step_enabled` 的 `Relocate` 一臂「目的字节等于源字节」也是 I-1.14 在搬迁前后两版上判的；`Write` 一臂「目的地没被任何视图引用」对应的是里程碑文件里已有的 I-7.4（近 K 代块未被复用）在搬迁中间态成立（`.claude/kb/milestone/04-fourth-txn.md:67`，二·2 验收一栏）。我没有逐条核 I-7.4 在 GC 流上判的对象是不是 `gc_slot_is_unreferenced` 的全部三类视图（当前、持久、留存读者）——没核。
- 新增义务 0 条；新增判器要不要，取决于上一条没核的那一点。
- **推翻「大半已存在」的现象**：I-1.14 在 GC 流上比的 key 不是被搬的那一把（例如搬迁副本的 key 因出生代变化而不同；G3 的写法是出生代不变，见 `crates/singlefs-core/src/transaction.rs:13330`）。
- 与 D13 已定项 1 射程的关系：这一候选只罩崩溃点那一维，射程说那一维崩溃点重放已全量枚举（`.claude/kb/decisions/13-验证路线.md:69`）。所以它不是 refinement 的替身，只是前提在枚举域里成立的见证。

### 3.6　四个候选与 D13 已定项 5、13、14、用户 2026-10-01 的冲突表

| 候选 | D13 已定项 5（checker 不调 core） | 已定项 13（同源性） | 已定项 14（哈希未解释） | 「Verus 跑真实代码」 |
|---|---|---|---|---|
| 核外提 | 不冲：被验核在 `singlefs-replay-prefix`，被 core 依赖；`singlefs-checker` 只依赖 `singlefs-format`（已定项 5 射程，`.claude/kb/decisions/13-验证路线.md:162`），不依赖这个核 | 不立新对：并入 `refinement-spec-o1-model`（3.4，本腿的读法） | 一致：校验和只当 `bytes_available` 布尔 | **满足**：`judge_relocation_item` 是 core 真调的那份（接上之后 `cargo check` 过） |
| core 直验 | 不冲 | 不立新对（同上） | 一致 | 满足，但今天的门禁与规则文字都要改 |
| 执行模型对拍 | 投影放池级 checker，不冲 | 并入 `refinement-spec-o1-model`；但 Verus 模型与 O1 对同一协议各写一遍的独立性要看事件 | 一致 | **只满足一半**：模型不是产品代码 |
| 前提运行时核 | 判器放 checker 一侧，不冲 | 不立新对 | 一致 | 不满足（没有 Verus 跑任何 core 代码） |

### 3.7　推荐叠法

核外提（①③ 两个判定由 core 调）+ ④ 写序前缀定理（吃 ①③ 的前提）+ 现有的运行时/枚举罩（I-1.14、崩溃点全量枚举、O1 对拍）罩胶水；不做 core 直验；不单做执行模型对拍；不新写前提运行时核（先核 I-7.4 那一点，没核之前不判它缺不缺）。

- **推翻它的现象**：(i) 把 ① 的判定接进 core 之后，发现它的 `Accepted` 与原来的 `match` 在某个输入上不同——本腿的接法保持原判定次序（先钉住、再源位置、读字节之后再判字节非空），只用 `cargo check` 验过能编，**没有跑 core 的用例**，行为等价没验；(ii) 单元写与 `Relocate` 的次序在 core 的真实写序里不成立（2.4）；(iii) 在原型上放开一条前提（例如去掉「目的地两两不同」）仍然证得出——八行手工必红世界里已经有这一行（红），但其余前提没逐条试。

## 四　X4　G7 那一句与条款怎么改

### 4.1　冲突两处，整行并排抄

**冲突一：判决 G7 对出口 5 现行原文。**

判决 G7 那一行（`research/prompts/m4-gc-r1-main-verification.md:38`，整行）：

```
| G7 恒等变换 | V-1 放进 `singlefs-replay-prefix`（先在副本上试加一个空模块跑 `cargo verus verify`，报错转 V-2）；证 P1（选段健全，先不证最优）、P2（计划健全：key 互异、目的地不在 R 内 / 两两不重叠 / 跨度相同——目的地那一半按 G2 改成分配器的性质，计划函数只证 key 一侧）、P3（进展）、P4（抽象恒等引理）；前提写成「R 内每条未释放记录都有映射 key 且不在隔离集里」（攻方 H-G7-1）。执行层靠 N6、模型对拍与必出反例的世界。对拍：`HistoryOperation` 加 `CompactWhileMounted`（含签发、批、批间崩溃），只比成功操作的内容，另判「GC 不把成功变成 ENOSPC，除非 df 也跟着少报」（攻方 H-G7-2） | 正推 G7、攻方 H-G7-1、H-G7-2 | 一轮 |
```

出口 5 现行原文（`.claude/kb/milestone/04-fourth-txn.md:20`，整行）：

```
| 5 | 「GC 在抽象文件系统状态上是恒等变换」在里程碑三七·1 选定的工具上证出；带 GC 的模型对拍与不跑 GC 逐项相同。用户 2026-10-26 定：证明要罩到 core 的搬迁执行（`publish_relocation_batch` 那一路），抽象模型的证明加模型对拍不算证出 | 二 |
```

G7 的「执行层靠 N6、模型对拍与必出反例的世界」（`research/prompts/m4-gc-r1-main-verification.md:38`）把执行层交给运行时判与对拍；出口 5 现行原文的「抽象模型的证明加模型对拍不算证出」（`.claude/kb/milestone/04-fourth-txn.md:20`）否了这种做法。冲突成立。

**冲突二：`.claude/rules/verification.md` 对形式化验证那一格的文字，对登记表。**

规则文字（`.claude/rules/verification.md:126`，整行）：

```
| 形式化验证：恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过，E165 全量的红例都钉进快档、钉表自己成形、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`）；红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 两格：pinned-red-examples-table 只判钉表自己（每行 22 段、指纹、抄来的段、档的写法、点名的变异还在，不读红例账、产物与 `research/`），pinned-red-examples-replay 逐行重放；钉全由 E165 第二段驱动 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` 写完一对红例账与产物时当场判（追加、再 `--check` 同一对，没钉全那一趟判红；它文件头「钉红例」一段） |
```

登记表里的核（`.claude/gate.d/verus-pins.tsv:31-32`，整行两条）：

```
kernel	crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs
kernel	crates/singlefs-replay-prefix/src/gc_crash_proofs.rs
```

规则文字只写「取前缀、择根、扫环三截的核与择根取前缀的合判推论」，登记表里被验的核已经有 `compaction_plan_proofs.rs`、`gc_crash_proofs.rs`。这一处在本轮之前就旧了，与选哪个候选无关。

### 4.2　G7 那一句改成什么（改写句，供主 agent 采纳或改）

G7 行里「执行层靠 N6、模型对拍与必出反例的世界。」一句换成：

> 执行层：核外提——`relocation_checked_against_the_version` 读完树之后的判定与一次发布的目的地判定放进 `singlefs-replay-prefix` 的可执行核、由 core 调；Verus 证这些判定通过时，一次发布按持久顺序（单元写、屏障、journal、屏障、根）的每个前缀与每个崩溃点上，当前根、持久根、每个留存读者读到的字节不变，对批大小 K、搬几批、槽值、字节、读者数任意。胶水（复制字节、取目的地、重写映射树与父节点的位置提示、拼写序、设备）由 N6（I-1.14）、崩溃点全量枚举、模型对拍与必出反例的世界罩；罩不到的清单随判决写。

G7 行里「P2（计划健全：……目的地那一半按 G2 改成分配器的性质，计划函数只证 key 一侧）」不改。目的地判定是另一个核（发布时判分配器交回的目的地），不是计划函数；它给分配器的产出加一道被验的检查，不改「目的地归分配器」的归属。

出口 5 原文不动（用户定的字不改）。建议在「用户 2026-10-26 定：……不算证出」之后另加一句，把抽象状态写出来：

> 抽象状态取 `gc_crash_proofs` 的 `logical`（当前根、持久根、每个留存读者读到同一份映射 key 到字节）；O1 对拍比的是它之上的可观察读回。

### 4.3　D13 要不要为 GC 恒等性质新立一项

- 已定项 1 的 M1–M4 主语是 `Recover`：「四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处」（`.claude/kb/decisions/13-验证路线.md:31`）。GC 的恒等性质不是 `Recover` 的关系性质，补进已定项 1 要改它的主语句。
- **建议新立一项**（草稿，供主 agent 改）：已定项 18「GC 搬迁的恒等性质与它的射程」。
  - 定案：抽象状态取 `gc_crash_proofs` 的 `logical`；Verus 证 ① 条目判定、目的地判定的函数性质，② 通过判定的一次发布在持久顺序的每个前缀与崩溃点上读不变（K、批数、槽值、字节、读者数任意）；
  - 射程：写出「罩不到的清单」（本报告 3.2 的八条）与未连接前提（4.5）；校验和只当布尔（已定项 14）；不读成「验了 core 的搬迁代码」；
  - 与已定项 1 射程的关系：证明罩的是崩溃点重放枚举取不到的维（K、批数、槽值、字节、读者数），崩溃点那一维仍归枚举；
  - 与已定项 13：并入同源性对 `refinement-spec-o1-model`（`.claude/gate.d/divergence-events.tsv:15`），事件写明谓词。
- 依据：本报告的原型（3.2）。**推翻它的现象**：用户选 (b)（core 直验）——那时这一项的主体要重写。

### 4.4　`.claude/rules/verification.md` 的两句要不要改

| 句 | 推荐路径下改不改 | 理由 |
|---|---|---|
| 「定义与名字」checker 档那一行里 Verus 验 `crates/singlefs-replay-prefix` 那一句（`.claude/rules/verification.md:13`） | **不改** | 新核模块在同一个 crate；`scripts/gate/verus-proof-check.py:582` 的单 crate 约束不动 |
| 「门禁的结构」里 checker-tier-formal-verification 按条那一句（`.claude/rules/verification.md:117`） | **不改** | 新增的 proof fn、exec fn、带 decreases 的 spec fn 自动成项；登记数 88 要随加 14（我的原型数，门禁没起）；必红世界要补行 |
| 同一份第 126 行（形式化验证那一格的文字） | **要改**（冲突二） | 补上「整理选段与计划核、GC 协议与回收比较核、搬迁执行判定核」 |
| 只有选 core 直验时 | 第 13 行与 117 行都要改，`verus-proof-check.py:473`、`:582` 的判法要改 | — |

### 4.5　K 与 `gc_crash_proofs` 自己列的未连接前提，这一次各罩不罩

K：`CompactionBatchSize::NotDecidedByTheClauses`（`crates/singlefs-core/src/compaction_plan.rs:351`）。原型的定理把 K 当参数（`keys.len()` 不设界），所以条款定不定 K 都不影响证明；**罩**。

`gc_crash_proofs` 自己写的边界：「本模块不证明真实设备」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:6`）、或调用方自动满足这些前提。逐样：

| 未连接前提 | 这一次罩不罩 | 怎么处理 | 现有的运行时/枚举罩 |
|---|---|---|---|
| floor 不超过受保护读者与根的版本 | **不罩**（Verus 里） | 写成前提 `protected_slots_cover_every_view`（模型 `:145`），目的地判定拿受保护集判 | 回收门槛与钉槽归池级 checker 的 I-7.18、I-7.19（`.claude/kb/decisions/13-验证路线.md:59`）；`released_record_can_be_reclaimed`（`gc_crash_proofs.rs:246`）是产品真调的核 |
| 多槽 span | **不罩** | 协议里槽是原子地址；条目判定只判跨度相等。要罩须把槽改成区间，另开一轮 | 分配器 + 崩溃点枚举 |
| 双盘副本 | **不罩** | 条目判定要求两份副本同槽；盘数那一维不在定理里 | 崩溃点枚举 + I-1.14 读两份取先核得上的 |
| 隔离 | **不罩** | `note_quarantined_placement`（`crates/singlefs-core/src/transaction.rs:13303`）把隔离槽记进挂载内的集合，R 里出现隔离槽丢意图；协议没有隔离步骤 | 现有隔离用例，本腿没核 |
| 共享快照 | **不罩** | 快照是持久读者，且有自己的逻辑版本；协议里读者共享同一个 `logical` | 崩溃点项 `compaction-relocating-a-block-shared-with-a-snapshot`（`.claude/gate.d/crash-point-replay-items.tsv:26`） |
| 不同逻辑版本 | **不罩** | `gc_crash_proofs.rs:9` 起文档注释：所有读者共享同一个 `logical`；用户写入穿插在批与批之间时 `logical` 会变 | O1 随机历史对拍（穿插整理） |

这六样都是「证明没连上的」，不是「已经被运行时判罩住」。第三列写的是现有的判，本腿只核了存在，没有核它们判的对象是不是协议里同一个量。

## 五　要主 agent 定的岔路（本腿只列，不定）

| 岔路 | 候选 | 本腿的读法 |
|---|---|---|
| 用户 2026-10-26 那一句「罩到」取哪一层 | (a) 加 (c) 的抽象写序那一半 / (b) core 直验 | (a)+(c 抽象)能做、已有原型；(b) 在今天的门禁下走不通（3.3）。选 (a)+(c 抽象) 就要对用户说清楚：复制字节、取目的地、重写映射树都还是没被证明的 Rust |
| 目的地判定要不要接进 core | 接 / 只留在证明里当前提 | 接则要受保护集的有序结构（原型是线性扫描），没量代价 |
| GC 恒等性质进 D13 的方式 | 新立已定项 18 / 补进已定项 1 | 新立（4.3） |

## 六　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X1 P4 对 `logical` | 一致 | 同一个 key→字节函数的类型，`logical` 严格强于 P4（多持久根、读者、崩溃） |
| X1 `logical` 对出口 5 后半句的 O1 | 规则没说 | 出口 5 没写抽象状态是哪一层；`logical` 到 O1 之间的桥（解码只经映射、提示读验校验和）条款没写 |
| X1 整个版本的内容不变 | 规则没说 | 同一发布还重写派生态，被重写的角色换新 key（`transaction.rs:11766`）；恒等只能说被搬的 key |
| X2 G7 对出口 5 现行原文 | **冲突** | 4.1 冲突一，两行整行并排 |
| X2 最弱读法在发布这一路上今天的状态 | 一致（现状没达标） | 发布这一路闭包核内 12/18057 行，没有选段计划核，(a) 还没做到；`.claude/kb/milestone/04-fourth-txn.md:38` 的状态列也写着「该证明不覆盖 core 搬迁执行和崩溃」，与量的相符 |
| X3 核外提对 D13 已定项 5、13、14 | 一致 | 核在 `singlefs-replay-prefix`，checker 不依赖；并入现有同源性对；校验和只当布尔 |
| X3 核外提对「Verus 跑真实代码」 | 一致 | 条目判定是 core 真调的那份（`cargo check` 过）；写序定理是纯证明，靠被调判定吃前提 |
| X3 core 直验对门禁与规则文字 | 冲突（条件：选它时） | `assume\w*` 在禁令里（`verus-proof-check.py:473`）、单 crate（`:582`）；探针 p2 `Option::copied` 不支持 |
| X3 执行模型对拍 | 一致（半满足） | Verus 一半就是 3.2 的④；对拍一半已有，比的是 O1 不是 `logical`；对「Verus 跑真实代码」只满足一半 |
| X3 前提运行时核 | 一致（大半已存在） | I-1.14 就是 `gc_all_views_match` 的跨版本形态；I-7.4 对不对得上 `gc_slot_is_unreferenced` 没核 |
| X4 G7 改写句 | 给了（4.2） | 执行层改成核外提 + 写序前缀定理，胶水仍由 N6、枚举、对拍罩，罩不到的清单随判决 |
| X4 D13 新立一项 | 规则没说 | M1–M4 主语是 `Recover`（`13-验证路线.md:31`），建议新立已定项 18（4.3） |
| X4 `verification.md` 两句（13、117 行） | 一致（不改） | 新模块在同一 crate；只有第 126 行旧了（冲突二），与本轮选哪个无关 |
| X4 K | 一致 | 定理对 K 任意，条款不定 K 不影响 |
| X4 六样未连接前提 | 规则没说 | 逐样结论 4.5：都不罩，写成点名前提 |

## 七　没做什么

- 不替主 agent 采纳或出判决；不判别的腿的格。没读禁读清单里的文件。
- **没有跑 core 的任何测试**（含 harness 档）：接进 core 的胶水只用 `cargo check` 验了能编；行为等价没验。没跑门禁（`checker-tier-formal-verification`、`scripts/gate/verus-proof-check.py` 都没起），所以「登记数 88 → 102」是直接起 `verus` 整个 crate 读出来的，不是门禁的各项之和。没跑 checker 档，没起 GPU。
- 没写：候选一的 ②（新映射）、没接目的地判定进 core、没量带受保护集的目的地判定在 core 里的代价；没写候选三、四的原型；没核 I-7.4 判的对象；没量 core 直验「能编下的比例」；没跑 O1 对拍。
- 必红世界手工 8 行，门禁判法要求 ≥ 11 行（推的），没补齐；原型不是入库形态。
- 探针那条命令写的线程是 `capped.sh 8`，不是提示的 31（更低，没越上限）；模型目录里的 `probe.sh` 和 `rerun.sh` 都用 31。
- 排队与负载：本机 slice 一直几乎满（40 GiB 上限、别的会话在跑），内存包装里排过队：第一次 verus（上限 8G）排了约五分钟，我停掉、改 2G 重发；整份 `rerun.sh` 两次各排了十几分钟。开跑前 `ps` 看到别的 `cargo`、`gate.sh` 与一条 `checker-tier-qemu-device-streams` 自证在跑，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio` 这类性能测量；包装没找到别的机器可用，照常本机跑。
- `rerun.sh` 的 `--selftest` 不适用：模板自证跑的是模板自带的那一段 `rerun_steps`（假冻结副本里没有 `Cargo.toml`），我换了这一段，自证在假树上 rsync 失败（退 1）。真跑（上面的复跑命令）两次：第一次产出参照，第二次 `mismatches=0`，输出（`name=` 行，原样）：

```
name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen model=/home/fy5090/code/singlefs/research/prompts/m4-exit5-relocation-proof-r1-sonnet-model work=/home/fy5090/.local/share/singlefs/work/m4-exit5-relocation-proof-r1-sonnet/rerun-2
name=rerun_compare item=closure-base result=same
name=rerun_compare item=closure-after result=same
name=rerun_compare item=verus-base result=same
name=rerun_compare item=verus-after result=same
name=rerun_compare item=red-worlds result=same
name=rerun_compare item=probes result=same
name=rerun_compare item=constructs result=same
name=rerun_compare item=core-check result=same
name=rerun_result mismatches=0
```

- 复核不了：门禁各项验过的义务之和（没起门禁）；「必红世界 11 行」的判法（读了 `negative_coverage_gaps` 的头，没跑）。

## 八　清理与留存

- 删之前 `du -sh` 量的：仓副本 `repo-copy` 39M、`build-core-check`（`cargo check` 的编译目录）228M、`build-unit-graph`（`unit-graph` 的编译目录）74M。交回前删掉这三份；`rerun.sh` 自己在草稿目录下的 `rerun-1`、`rerun-2`（各 3.1M，只剩输出，没有仓副本与编译目录，`rerun.sh` 末尾已删）和 `out/`（3.5M，开发中的原始输出）留在草稿目录 `~/.local/share/singlefs/work/m4-exit5-relocation-proof-r1-sonnet/`。
- 入库位置：报告 `research/prompts/m4-exit5-relocation-proof-r1-sonnet-output.md`；模型目录 `research/prompts/m4-exit5-relocation-proof-r1-sonnet-model/`（`src/` 原型核、`patches/` 两个补丁、`probes/` 七个探针、`tools/` 工具与 `rerun.sh`、`out/` 八份参照输出）。原型核没有放进 `crates/`（本腿不写 `crates/`）；要采纳的话由实现员按 `src/` 与 `patches/` 落到 `crates/singlefs-replay-prefix/src/` 与 `crates/singlefs-core/src/transaction.rs`，并补登记（`verus-pins.tsv` 的登记数与 `verus-must-red-worlds.tsv`）。
