# m3-close-c506-r1 云端正推腿（Sonnet）报告：每区槽数 S 的上界改成格式常量 16（V1–V4）

轮名 m3-close-c506-r1，腿 = 云端正推。被判对象 = 用户 2026-10-22 定案「把 D22 已定项 2 那一格的『上界由 I-7.4 扣住的块数挂载时算』改成『上界 16 是格式常量』」。
立场：先从已定条款与 `crates/` 今天的实现推出「应该是什么」，再与这一改法逐格比。引 kb 写文件加那份文件自己的行号（现查）。

## 0. 复跑

纯算术模型（不编译 Rust、不起重活），回读 `crates/` 里 18 个同名 const 核本地常量，对不上退 1：

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-c506-r1-sonnet-model/tools/rerun.sh --repo $PWD --frozen research/prompts/m3-close-c506-r1-snapshot --work <新的草稿目录>
```

末行应为 `name=rerun_compare result=identical lines=55`（我这次跑的原样输出就是这一行）。弄坏本地常量证红：把 `model.py` 里 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM` 的本地值改成 17，输出 `name=source_check result=MISMATCH ROOT_RING_SLOTS_PER_REGION_MAXIMUM: 本地 17 源码 16`、退 1（草稿目录 `model-broken.py` 里跑的）。

```
3cf95a94248fcff870f991b8c59f94cafe21ad1a295c7884fbb69c1027cc797a  research/prompts/m3-close-c506-r1-sonnet-model/model.py
f84247f02f81dd019b9ff1199c0f90969895f8e6ca844cc887a6cee2929ea7c4  research/prompts/m3-close-c506-r1-sonnet-model/tools/rerun.sh
de7b29809d03cc09d91e1f96d103cd941838cba2958674b33b73a1af1ba35b9f  research/prompts/m3-close-c506-r1-sonnet-model/out/model.out
```

模型里「环窗口」那张表是我按 `crates/singlefs-core/src/admission.rs` 的式子用 Python 复写的算术（单元区起点按环长现算、两块同容量盘、K = 144、退役树内部层 0），**不是实现跑出来的数，一律按「推的」读**。

## V1 原条款防什么，改成常量 16 之后丢不丢

### 1.1 先把 S 在条款里接到哪几处

| 关系 | 出处 | 内容 |
|---|---|---|
| 环长 | `crates/singlefs-core/src/root_ring.rs:115` | `slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),`——环里 R × S = 3S 个根槽，一圈 3S 个 txg |
| 回退候选集 | `crates/singlefs-checker/invariants/02-根环.md:11` | 「**「K 代」就指这个候选集**」，候选集 = 环里按实例表仍然有效且 txg ≥ F_生效 的根，所以 \|候选集\| ≤ 3S |
| 块复用的界 | `.claude/kb/decisions/16-发布语义.md:35` | 「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」 |
| 环窗口常驻扣项 | `.claude/kb/decisions/28-挂载期承诺量.md:31` | 「环窗口 = (根环槽数 3 × S + 1) × W」，同一行写「S = 4 / 8 / 16 时环窗口 13W / 25W / 49W」 |
| 实现里的环窗口 | `crates/singlefs-core/src/admission.rs:500` | `let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;`，乘 W 后每块盘各扣一份 |
| 洞里的回收门槛 | `.claude/kb/decisions/16-发布语义.md:62` | 「**回收门槛取环里最旧有效根的代价**」：每个洞多压一圈，「正推腿端口上 S = 16 平均 893 块对窗口门槛 583 块」；「一块盘持续写不进根时扣住的单元随发布线性涨、没有上界」 |
| 被抛弃根的隔离 | `crates/singlefs-checker/invariants/02-根环.md:11` | 被抛弃时间线的根「引用的块在离开根环之前同样不许重新分配、不许抹头」——根在环里待多久由 S 定 |

S 不出现在块复用的界那一行里。块复用的界由 F_生效 和「环里最旧有效根」现算；S 只通过「环里最旧有效根最老能老到哪」和「环窗口乘数」两条路影响扣住多少。

### 1.2 原条款防的是什么（推的，条款自己没写它防什么）

`.claude/kb/decisions/22-单元原子性怎么合成.md:60` 的原句只说「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**」，没写 S 过大会破坏哪条不变量。从 1.1 推：

1. **没有正确性不变量会因 S 变大而破坏。** I-7.4 是条件句——候选集里的根引用的块不许复用。S 大只是候选集的上限大、要扣的块多，不是条件被违反。I-7.4 自己和 D22 已定项 4、22 的「失败模式：可检测，不可恢复」都对任何 S 成立。
2. **S 大的代价全是空间：**
   - 环窗口常驻扣项随 S 线性涨：(3S + 1) × W，S = 16 比 S = 8 多一倍（49W 对 25W）。
   - F 不动时（平时）环里最旧有效根最老 3S 个 txg，扣住的是这 3S 代释放的块。
   - 崩溃恢复落到旧根后，被抛弃根的块在根转出环之前一直隔离（`.claude/kb/decisions/28-挂载期承诺量.md:29`：「被抛弃的根被轮转覆写时清零」），时间也随 S 涨。
   - 洞里（一块盘写不进根）每个洞多压一圈，圈长 3S（上表 `.claude/kb/decisions/16-发布语义.md:62`）。
3. 所以原条款的意图是：**别让 S 大到把小池的可用空间吃掉一截**。这是可用性护栏，不是正确性护栏。

### 1.3 原条款自己算得出来吗：算不出来（循环）

`crates/singlefs-format/src/lib.rs:304` 写：「// 候选集又按 F 与实例表现算，仓里没有一条式子从池的参数给得出 S 的上界。在它算得出来之前按 16 收着，区间判用的是这个收着的数。」
我的推导与它同向：I-7.4 的 K = 候选集 ⊆ 环 = 3S，用「候选集扣住的块数」去反算 S 的上界，被算的量在算式两边；这个循环 2026-09-13 把「K」改成「候选集本身」时就有了。E50 当初算的是另一个量——`scripts/experiment-apparatus/e50_ring_slots.rs:39`：`const PINNED_MATTERS: u64 = 1000;`，即「N × 每次 fsync 写的块数 ≥ 一条跑前定死的 1000 块的线」，随负载变，不随池大小变（见 V4 第 3 条）。

### 1.4 常量 16 之后丢什么：丢「随池大小缩上界」，不丢正确性

按 1.2，S ≤ 16 却把原条款要护的东西护丢了，只会发生在「环窗口 + 环里扣住的块」吃掉太大一截的池和历史上：

| 池 | 历史 | 推的量（模型，不是实现跑的） |
|---|---|---|
| 小盘（两块 1 GiB，journal 环 64 MiB） | 任何可写挂载（环窗口是常驻扣项，每次发布现算） | 环窗口 S = 16 占每盘 13.46%，S = 8 占 6.87%（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:40`、`:39`） |
| 4 GiB（journal 环 768 MiB） | 同上 | S = 16 占 6.99%，S = 8 占 3.57%（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:46`、`:45`） |
| 100 GiB | 同上 | S = 16 占 0.35%（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:52`）——大盘上这一项可以忽略 |
| 任意盘 | 一块盘持续写不进根、或崩在记录之后根槽之前 | 扣住的块随发布线性涨、没有上界（`.claude/kb/decisions/16-发布语义.md:62`），S 只决定每个洞多压一圈（3S 个 txg） |

所以：**丢的是小池上「S 大到环窗口占一成以上」这一类用户不友好的取值被挂载时拒掉的可能**，被拒的取值今天在 `crates/` 里也没有人写：

- `crates/` 非测试源码里造 `RootRingSlotsPerRegion` 的只有两条路：mkfs 用 `AT_MAKE_FILESYSTEM`（8，`crates/singlefs-format/src/lib.rs:309`：`pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;`），读盘用 `from_system_configuration_field`（`crates/singlefs-core/src/system_configuration.rs:670`）。别的 `from_system_configuration_field` 调用都在 `#[cfg(test)]` 与实验二进制里（grep 命中：`crates/singlefs-core/src/mount.rs:10118` 起、`crates/singlefs-core/src/allocator.rs:4325`、`crates/singlefs-core/src/root_ring.rs:162`）。
- 就是说：今天所有真实池 S = 8；常量 16 只在「别的写者写出 9..16」时才有意义，那时护栏从「挂载拒」变成「环窗口扣项把可用量压低、写类准入更早报 ENOSPC」。后一条是 D28 已定的准入式子，不拒挂载。

**没丢的东西**（逐条核）：

- I-7.4、D22 已定项 4、已定项 22 都是「对任何 S 成立」的条件句，不依赖 S 的上界。
- 块复用的界不是格式常量这一条（D22 已定项 25：`.claude/kb/decisions/22-单元原子性怎么合成.md:495` 「块复用的界不能是格式常量」）不受影响：常量 16 是 S 的上界，不是块复用的界。块复用的界仍是 `max(F_生效, 环里最旧有效根)` 现算。候选句必须把这两件事分开写（V4）。
- 抬 F 是盘紧时的逃生口：准入抬 F 把回收门槛抬到第 4 新的非空根（`.claude/kb/decisions/16-发布语义.md:38`），环再深也只是平时的上限，不是盘紧时的下限。

### 1.5 V1 的实现对比

| 项 | 应该是什么（推） | 被判对象（改条款写成 16） | 判 |
|---|---|---|---|
| 条款能不能算出上界 | 算不出（循环，1.3） | 不算，写常量 | 一致 |
| 实现里上界是多少 | 16（`crates/singlefs-core/src/root_ring.rs:65`：`if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)`） | 16 | 一致（条款追认实现） |
| 正确性不变量 | 不依赖 S 上界 | 不动 | 一致 |
| 可用性：小池 S 大 | 本该有「随池大小」的护栏 | 没有护栏，换成准入扣项 | 规则没说（没有条款写这笔代价的上限；上表是推的） |

**什么现象会推翻 V1 的结论**：
1. 找到一条依赖「S ≤ 随池大小的上界」才成立的正确性不变量或已定项（我读 I-7.4、D22 已定项 4 / 22 / 25、D16 已定项 1、D28 已定项 1、D23 已定项 14 没找到）；
2. 找到一条真实写者路径把 S 写成 9..16 并在小池上让环窗口超过可用量、且写类准入因此永远拒（则 16 的上界需要联动 mkfs 的拒绝，不是纯条款）；
3. `model.py` 的环窗口表被 `admission.rs` 的真实读数推翻（我没跑实现；表是复写的算术）。

## V2 常量 16 落 D15 已定项 7 的哪一层、要不要 incompat 位、两条边是不是都登记 format-const

### 2.1 落哪一层：第 1 层

推导三步：

1. S 是系统配置字段（偏移 362 一字节，`crates/singlefs-core/src/system_configuration.rs:58`：`const ROOT_RING_SLOTS_PER_REGION_OFFSET: u64 = 362;`），归「系统不可变配置」那一段（`.claude/kb/decisions/22-单元原子性怎么合成.md:208` 「几何里除节点大小之外的全部」；mkfs 时设、之后不可改，`.claude/kb/decisions/22-单元原子性怎么合成.md:241`）。
2. 系统配置槽落第 1 层：`.claude/kb/freeze-layer-membership.md:19`：「| 系统配置槽（码 18 自证单元，每盘两槽轮换，字段表 489 字节） | 第 1 层 | 判不动 | 不退出 | D15（格式冻结政策） 已定项 7 |」，层的定义在 `.claude/kb/decisions/15-格式冻结政策.md:140`：「第 1 层：系统配置 + 块头（自身地址 / generation / 树 ID / fsid / magic）」。
3. 16 本身不是一段字节布局，是「第 1 层这个字段读者收哪些值」。仓里已有同型先例：根环区域数 R、根环起点、journal 环起点用「与第一版常量不等一律整池拒」收（`.claude/kb/decisions/22-单元原子性怎么合成.md:348`：「读者解槽时读这个字段与 journal 环起点字段（偏移 325），与第一版常量（64、1024）不等一律整池拒」），系统配置槽宽写成「格式常量 4096」并明写「落在 D15（格式冻结政策） 第 1 层，改它要 incompat 位」（`.claude/kb/decisions/22-单元原子性怎么合成.md:62`，那句逐字是：`落在 D15（格式冻结政策） 第 1 层，改它要 incompat 位`）。

⇒ 16（和 4）落第 1 层，归「读者收的范围」这一类（I-7.13，`crates/singlefs-checker/invariants/02-根环.md:20`）。冻结层归属登记表按结构记、不按常量记，所以这一改**不新增登记表的行**（那张表的行是「系统配置槽」，已在第 1 层）。

### 2.2 改它要不要 incompat 位

分两个时点答：

| 时点 | 答案 | 依据 |
|---|---|---|
| 今天（软格式期） | **不要。** 这次改的是条款，不是字节：mkfs 写 8、读者收 4..16，三者今天的代码就是这样，没有一个盘上字节变 | `.claude/rules/format-evolution.md:7`：「**在第一个外部用户出现之前，磁盘格式是软的**」；`.claude/kb/decisions/15-格式冻结政策.md:177`：「第一个外部用户出现之前格式是软的，这张表在那之前一条都不生效」 |
| 冻结之后放宽上界（例如 16 → 32） | **要。** 旧读者拿到 S = 17..32 的池会按「值越界」整池拒（`crates/singlefs-core/src/root_ring.rs:65`），拒是显式的、安全的，但等于池对旧读者不可读；D15 已定项 9 判表按「旧读者会怎样」分档，这一行没有现成的行（见 2.3） | `.claude/kb/decisions/15-格式冻结政策.md:199`：「**射程**：判的是「旧读者会怎样」」；`.claude/kb/decisions/22-单元原子性怎么合成.md:215`：「硬顶仍是槽宽 4096……超过它要 incompat 位」 |
| 冻结之后收窄上界（例如 16 → 12） | **做不到。** 已经写出 S = 13..16 的池会被新读者拒；任何带 incompat 位的办法都救不了已存在的池 | 推的：新读者读旧池，旧池没有位可以让它「知情」 |
| 冻结之后改下界（4 → 别的） | 同上两行：抬下界等于拒掉 S = 4 的旧池，降下界放宽了「至少保留 4 个状态」的承诺（`.claude/kb/decisions/16-发布语义.md:46`） | |

**与 D16 已定项 1 的张力（冲突，两句并排整行抄）**：

- `.claude/kb/decisions/16-发布语义.md:46`：「| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |」
- 若改后的 D22 把 4 写成格式常量：4 就是「第一版读者收的范围」的一条边，冻结之后「随之抬」做不到（2.2 第四行）。

这条张力今天不红（k_tol 是实现常量，`crates/singlefs-core/src/mount.rs:4347` 的注释 `k_tol = 2`，不在格式里；软格式期可抬）。但改后的 D22 句子要么写「下界 4 是第一版的值」，要么把 D16 那一格的「随之抬」改成「冻结前随之抬」，否则两条已定条款对冻结之后互相矛盾。这一条我标「规则没说」，交主 agent 判。

### 2.3 D15 已定项 9 判表里没有「放宽读者收的值域」这一行

`.claude/kb/decisions/15-格式冻结政策.md:190`：「| 新增字段，缺了会让旧读者对**另一个已有字段**算错 | **算错，可能损坏数据** | incompat | 是 |」、`:191`：「| 扩大已有字段宽度 | 按旧宽度解析，越界读 / 错位 | incompat | 是 |」、`:192`：「| 改变已有字段含义 | 按旧含义解读，**不报错**地产生错误行为 | incompat，且**弃用旧位置不复用** | 是 |」。

- 放宽 S 的值域不改宽度（仍 1 字节）、不改含义、不新增字段；旧读者的行为是「显式整池拒」，不是「算错」。三行都对不上 ⇒ **规则没说**。
- 我按「旧读者会怎样」推：旧读者显式拒、不损坏数据，最接近 compat_ro 之外的第三种——「旧读者无法挂载」，即 incompat 的效果。所以放宽时要带 incompat 位，让旧读者走「incompat 位不认得」那条路径（`crates/singlefs-core/src/recovery.rs` 的 incompat 分支，D22 已定项 16 第 1 句），而不是靠值越界拒。这一条是推的，没有条款，交主 agent 定是否补一行判表。

### 2.4 区间两条边是不是都登记 format-const：现状不对称，应当两条都登记

现状（与 C506 登记行「区间两条边在 kb 里没有 format-const 登记位」对上）：

- `crates/singlefs-format/src/lib.rs:299`：`/// format-const: ROOT_RING_SLOTS_PER_REGION_MINIMUM`——下界有一行 Rust 文档注释里的写法，上界（`:305`）连这行都没有；
- 门禁认的标记文法是 `<!-- format-const: 名字 = 整数 -->`（`.claude/gate.d/lib-format-const.py:24`：`MARK_GRAMMAR = re.compile(`），只扫 kb 里的 HTML 标记，源码里的文档注释不算登记；`.claude/gate.d/code-source-discipline.sh:496`：`scanned_roots = ["scripts/experiment-apparatus", "research", "crates"]`，对每个标记在这三处找同名 `const` 比值；
- `.claude/kb/layout/01-first-txn.md:140` 的 S 那一行（「| 几何 | 每区槽数 S | 1 | 8 | D22（单元原子性怎么合成） 已定项 2 |」）旁今天一个标记也没有。

⇒ 两条边都登记（理由）：读者在四个地方判这个区间——core `crates/singlefs-core/src/root_ring.rs:65`、checker `crates/singlefs-checker/src/image.rs:735`、GPU 内核三处字面量（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:960`、`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:906`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:111`，都是 `4u` / `16u` 字面量）；只登记一条边，另一条漂了没人红。两条边在语义上同类（都是读者拒绝的边，`crates/singlefs-checker/src/image.rs:738` 与 `crates/singlefs-core/src/root_ring.rs:68` 对两边报同一个成员），登记位也要同类。

登记写法（候选行，登记名后面不带 `stale=`，裸数字 4、16 不许登记）：在 `.claude/kb/layout/01-first-txn.md:140` 的行里追加

`<!-- format-const: ROOT_RING_SLOTS_PER_REGION_MINIMUM = 4 --> <!-- format-const: ROOT_RING_SLOTS_PER_REGION_MAXIMUM = 16 -->`

三条要核的事：

1. 同一个名字全 kb 只许登记一处（`.claude/gate.d/code-source-discipline.sh:484`：`if len(places) > 1:`），D22 正文提到它们要写成不带 `<!--` 的文字；
2. `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:70` 与 `:72` 有同名本地 const（值 4 与 16，`const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16;`），登记之后门禁会连它们一起钉住——值相同不红，将来改上界它们会一起红，是好事；
3. `crates/singlefs-checker-tier/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:63` 用的是 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL`，名字不同，**钉不住**；三处 wgsl 字面量是 `.wgsl`，门禁只扫 `.rs`，也钉不住（V4 销账里列）。

**什么现象会推翻 V2 的结论**：
1. 在 D15 已定项 7 / 登记表里找到把「读者收的范围」判在第 1 层以外某层的先例（我没找到：R、起点、槽距、格式版本都按第 1 层处理）；
2. 门禁在 `.claude/kb/layout/01-first-txn.md:140` 追加两个标记之后没红（自证见 V4：把标记里的上界改成 17，`format-constants` 格必须由绿转红）；
3. 发现放宽值域有现成的 D15 判表行（我读 `.claude/kb/decisions/15-格式冻结政策.md:189-197` 的九行没有）。

## V3 16 与根环几何其余约束

### 3.1 先改一个前提：不是「R × S 个根槽落进一个 chunk」

区域不是一个 chunk 里的 S 个槽乘 R：三个区域各占 S 个槽，起点间隔 P × chunk = 3 MiB。
`crates/singlefs-core/src/root_ring.rs:96`：`ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES,`，
测试钉着起点在 1 / 4 / 7 MiB（`crates/singlefs-core/src/root_ring.rs:169`：`assert_eq!(region_start(1), DeviceOffsetInBytes(4 << 20));`）；
区域长 = S × 槽距（`crates/singlefs-core/src/root_ring.rs:144`：`slots_per_region.count() * u64::from(fixed_structure_slot_spacing)`），区域之间「这一段之外的字节不归根环」（`crates/singlefs-core/src/root_ring.rs:137`）。
所以真问题是：**S × 槽距 ≤ 3 MiB（相邻区域不叠）**，以及**最末区域末端 ≤ journal 环起点 16 MiB**。「一个 chunk（1 MiB）」只是一个更紧的可选读法，条款里没写它（D22 已定项 16 第 3 句 `.claude/kb/decisions/22-单元原子性怎么合成.md:350` 只写「mkfs 要校验 `基址 + (R−1) × P × chunk + S × 槽距 ≤ 设备可用字节`，越界拒绝 mkfs」）。

### 3.2 S = 16 放不放得下（模型逐格，`out/model.out`）

| 槽距 | S = 16 的区域长 | 不叠 3 MiB | 在一个 chunk 里 | mkfs | 行 |
|---|---|---|---|---|---|
| 4096（第一版默认） | 64 KiB | 是 | 是 | 收 | `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:8` |
| 65536 | 1 MiB | 是 | 是（恰好） | 收 | `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:14` |
| 196608 | 3 MiB | 是（恰好） | 否 | 收 | `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:20` |
| 262144 | 4 MiB | **否** | 否 | **拒（区域叠）** | `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:23` |
| 1044480（格式允许的最大槽距） | 约 15.94 MiB | 否 | 否 | 拒（区域叠且末区撞 journal 环） | `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:35` |

最大的不叠槽距：S = 4 → 786432，S = 8 → 393216，S = 16 → 196608（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:3`、`:4`、`:5`）。

### 3.3 读者一侧没有这一判：S 区间与槽距区间是两条独立的区间，交叉格里有放不下的

槽距的读者范围是 [4096, 1 MiB − 4096]（`.claude/kb/decisions/22-单元原子性怎么合成.md:74`：「固定结构槽距 ∈ [4096, 1 MiB − 4096]」，上界是槽 1 落在根环基址之前），S 的读者范围是 [4, 16]。两者的交叉格里：槽距 1044480 时 S = 4 就放不下（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:33`）——所以**放不下不是 16 引进来的**，S 越大放不下的槽距越小。

「叠」只有 mkfs 判：`crates/singlefs-core/src/make_filesystem.rs:528` `let extents = fixed_structure_extents(geometry);` 起两两比 `overlaps`（`:115`），叠了报 `FixedStructuresOverlap`（`:533`）。读者表没有：`crates/singlefs-core/src/recovery.rs:1095` 的 `system_configuration_values_this_reader_accepts`、`crates/singlefs-checker/src/image.rs:734` 起的 `geometry_of`、GPU 内核。我对 `recovery.rs`、`image.rs`、`system_configuration.rs`、`walk.rs`（grep `overlap` / `重叠`）和 `crates/singlefs-checker-tier/src/*.wgsl`（grep `overlap` / `REGION_STRIDE`）各查一遍：命中只有 `crates/singlefs-core/src/recovery.rs:1344` 的说明句（「mkfs 判固定结构两两不重叠」）、`recovery.rs` 里讲日志记录跨度的无关注释、和 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:113` 的 `ROOT_RING_REGION_STRIDE`（只拿来算根槽偏移）；没有一处判区域叠。
⇒ 一个自证校验和过的系统配置槽写 S = 16、槽距 262144，读者表全收，可写挂载按 `target_for_publish` 写根时，区域 0 的槽 12 落在区域 1 的槽 0 上（4 MiB）。合法写者（mkfs）造不出，只有非合规写者或带有效校验和的损坏才会有（推的，没造镜像）。这一格**条款落后于实现**：D22 只写了「设备可用字节」一判，实现多判了「两两不重叠」。我把它标「规则没说」，不并进 C506；若要补，是 I-7.13 与 core 读者表各加一行「S × 槽距 ≤ P × chunk 且末区域末端 ≤ journal 环起点」，并要三个读者（core、checker、GPU）同改——这是扩范围，交主 agent 定。

### 3.4 mkfs 写的 8

- 8 严格落在区间里：`crates/singlefs-core/src/root_ring.rs:50`、`:51`：`slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM` 与 `&& slots_per_region < ROOT_RING_SLOTS_PER_REGION_MAXIMUM,`，编译期断言。条款句「4..16 之间取值不碰任何一条边」（`.claude/kb/decisions/22-单元原子性怎么合成.md:60`）在实现里的读法是「mkfs 不写两条边上的值」，读者收闭区间。
- 8 比 16 先撞槽距：S = 8 在槽距 > 393216 时 mkfs 就拒（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:25` 是恰好 3 MiB 的临界格，`:28` 是槽距 524288 的拒绝格）。槽距 = 4096 向上取整到 io_min 的整数倍（`crates/singlefs-format/src/lib.rs:282` 起的注释），所以 io_min ≥ 512 KiB 的阵列上 mkfs 的默认 S = 8 过不去，需要 S ≤ 6；本仓只量过 io_min = 64 KiB 的 md/raid5（`.claude/kb/experiments/34-根环槽几何.md:93`），64 KiB 下 S = 16 恰好占满 1 MiB。512 KiB 是 md 的常见块大小，**外部事实，未在本项目验证**。这一格与 16 无关，是 8 的问题，写给主 agent 当线索。

### 3.5 系统配置那一字节的取值域，和 16 撞到的别的「3S」

| 项 | 内容 | 出处 |
|---|---|---|
| 字节域 | 偏移 362 一字节，0..255；读者收 4..16；S = 0 会让取模除零，所以类型拒 0 | `crates/singlefs-core/src/system_configuration.rs:58`；`crates/singlefs-core/src/root_ring.rs:326`：`"全 0 的槽也拒：S = 0 会让环长变 0、取模除零"` |
| 钉槽字节 | 根记录的钉槽 p 是 1 字节、255 = 没有钉着；p < 3S ⇒ S ≤ 85（`research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:36`） | `crates/singlefs-core/src/root_record.rs:74` `pub struct PinnedRingSlot(pub u8);`，`crates/singlefs-format/src/lib.rs:238` `pub const ROOT_RECORD_NO_RING_SLOT_PINNED: u8 = 255;` |
| GPU 根表容量 | `MAX_ROOTS = 48u`（= 3 × 16），满了记「判不了」不记错 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:241`；溢出出口 `:1316` `undecidable = UNDECIDABLE_TOO_MANY_ROOTS;` |
| 错误消息里的 48 | 三处 `expect` 的消息写死「3S ≤ 48」 | `crates/singlefs-core/src/root_record.rs:93`、`crates/singlefs-core/src/mount.rs:2310`、`crates/singlefs-core/src/mounted_session.rs:2339`（只是消息文字，逻辑上界是 u8 / usize） |

⇒ 16 正好等于 GPU 判器的根表容量。上界放宽到 16 以上不撞盘上字节（到 85 之前），撞的是 GPU 表和三处消息字面。就是说 **16 作为格式常量并不是随便取的整数：它与已有的 checker 档容量对上，要抬得先抬 GPU 表**（这一句是推的，没跑 GPU）。

### 3.6 E50 的那一行撑不撑得住（D22 依据段）

D22 `.claude/kb/decisions/22-单元原子性怎么合成.md:83`：「E50（根环槽数的上下界）：它量出的 S 下界与上界都在 4..16 之外，S 不是一个要精算的量。」

E50 自己的实验源码写：`scripts/experiment-apparatus/e50_ring_slots.rs:168`：`/// 散开下 30 槽就够到 ⇒ 首个是 **32**。`，要紧的线是 `scripts/experiment-apparatus/e50_ring_slots.rs:39`：`const PINNED_MATTERS: u64 = 1000;`。N 是总槽数 R × S（`.claude/kb/experiments/50-根环槽数的上下界.md:13`：「槽总数 `N = R × S`。」）。R = 3 时 N = 30 就是 S = 10，在 4..16 之内；S = 16 即 N = 48，散开负载下扣住 48 × 34 = 1632 块。E50 页结论一写的是 `.claude/kb/experiments/50-根环槽数的上下界.md:118`：「⇒ **S 在 1..16 这个区间里怎么取都不碰任何一条边**，它不是一个要精算的量。」两句并排：

- E50 页 `:89`：「| **首个够到 1000 块这条线的 N** | **256** | **32** |」
- D22 `:60` 与 `:83`：S 的上界「4..16 之外」。

散开那一列 N = 32 ⇒ S ≈ 10.7，**在 4..16 之内**。冲突。留意两点：线（1000 块）是跑前定死的，页上自己写了换一条线上界会跟着变；E50 扣住的块数是 N × 每次 fsync 写的块数的旧模型（环整圈扣住），D16 已定项 1 的抬 F 与 D28 的环窗口之后不是现行取法。所以这一行撑得住的只有「S 不是一个要精算的量（上界随负载与那条线变）」，撑不住「16 在界外」。

**什么现象会推翻 V3 的结论**：
1. 读者一侧其实有「区域不叠」的判（我 grep 的文件之外，例如 `walk.rs`）——则 3.3 的缺口不成立；
2. `AT_MAKE_FILESYSTEM`（8）之外有真实 mkfs 入口写 9..16，并撞到槽距——则 3.4 要并进 C506；
3. E50 的 `.out` 产物（已归档，`git show 3cff909b0^:research/results/e50-ring-slots-2026-08-30.out`）里散开那一列的首个 N 不是 32——我读到的是 `name=upper_bound first_matter_coarse=256 first_matter_scattered=32 max_n_scanned=1024`。

## V4 改条款的写法

### 4.1 D22 已定项 2 那一格：候选句（逐字）

现状（整行，`.claude/kb/decisions/22-单元原子性怎么合成.md:60`）里要换的一段，逐字是：

`**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。`

候选换成（A，我推的）：

`**上界 16 是格式常量**（下界 4 同为格式常量，两条边的登记名是 `ROOT_RING_SLOTS_PER_REGION_MINIMUM` 与 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM`；上界不由 I-7.4（近 K 代块未被复用） 扣住的块数算，那个量随回退候选集与负载变，给不出一个只由池定的数；块复用的界仍由 `max(F_生效, 环里最旧有效根)` 现算、不是格式常量，见已定项 25）；4..16 之间取值不碰任何一条边（mkfs 写的 S 不取两条边上的值）。`

这一段之后的原句不动：`⚠️ **具体取多少不是格式决策**——格式承诺的是「S 是系统配置字段 + 挂载时判区间」，取值由第一版实现按 D25（目标负载优先级） 的负载形态定`。它和「上界 16 是格式常量」不冲突：S 的取值不是格式决策，区间两条边是；「挂载时判区间」正是读者收这两条边。

候选 B（最短，不推荐）：只把「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**」换成「**上界 16 是格式常量**」。不推荐的原因：丢了「上界不是块复用的界」那半句，D22 已定项 25 `.claude/kb/decisions/22-单元原子性怎么合成.md:495` 写的是「块复用的界不能是格式常量」，读者容易把「上界 16 是格式常量」读成「根环深度是格式常量，所以块复用的界也是」——已定项 25 点名防的正是这一读法（同一行：「「根环深度 = 块重用延迟」很容易被实现成「深度是格式字段，所以 K 是格式字段」」）。

候选 A 与已定条款的逐条对：

| 已定条款 | 对 A 的关系 | 判 |
|---|---|---|
| D22 已定项 25（块复用的界不能是格式常量） | A 明写块复用的界仍现算 | 一致 |
| D22 已定项 2 同一格「具体取多少不是格式决策」 | 不冲突（取值 vs 区间边） | 一致 |
| D22 已定项 16 第 1 句 `:348`（S 越界整池拒） | A 的「格式常量」就是这条的区间 | 一致 |
| D16 已定项 1「环深下限」`.claude/kb/decisions/16-发布语义.md:46`（下界「随之抬」） | 冻结后抬不了（2.2） | **规则没说**，要么加「冻结前」，要么 A 里写「第一版的下界 4」 |
| D28 已定项 1 环窗口 `.claude/kb/decisions/28-挂载期承诺量.md:31` 的「S = 4 / 8 / 16」 | A 让 16 成为这行里「最大的 S」的出处 | 一致 |
| D15 已定项 9 判表 | 没有「放宽值域」一行（2.3） | 规则没说 |

### 4.2 依据段要不要加一条：要，而且 E50 那一行要改

现状 `.claude/kb/decisions/22-单元原子性怎么合成.md:83`：「E50（根环槽数的上下界）：它量出的 S 下界与上界都在 4..16 之外，S 不是一个要精算的量。」——3.6 已证它与 E50 自己的散开那一列冲突，撑不住「16 在界外」。候选（我推的）：

- 改 E50 那一行为：`E50（根环槽数的上下界）：它量出的 S 上界随负载与一条跑前定死的线变（扣住的块数 ≥ 1000 块那条线：散开负载 30 槽、粗粒度 77 槽，R = 3 时折成 S = 10 与 S = 26），没有一个只由池定的数，S 不是一个要精算的量；上界 16 是收着的上限，不是 E50 量出来的。`
- 加一条用户定案：`用户定案 2026-10-22：上界改写成格式常量 16，推翻「由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算」（C506（每区槽数 S 写成编译期常量，条文说它住系统配置）），原话在变更史。`
- 加一条代价指针（E50 一行不够撑 16 为什么不更大或更小；这一条给 16 这个数一个有出处的位置）：`D28（挂载期承诺量） 已定项 1：环窗口 = (3S + 1) × W，S = 16 时 49W；E168（删除借保留量的环窗口） 的第一段在 S = 4 与 S = 16 两端都量过删除路径（最小的反例 S = 16、K = 1、单元区 149 槽，E168 页 :99）。`

出处核实：E168 页 `.claude/kb/experiments/168-删除借保留量的环窗口.md:11` 写「P0、P1、P2、P4 × S = 4 / 16 × K = 144 / 1」；`:99` 写「**最小的反例**（segment1 产物第 25777 行）」。

E50 那一行够不够撑：**不够**。它只撑得起「上界给不出一个只依赖池的数」（对应原条款为什么不能照字面实现），撑不起 16 这个数本身；16 的撑腿是「实现与 GPU 判器容量已经是 16」（3.5）、「mkfs 取 8」与用户定案。改了 D22 依据之后，E50 页「影响的决策」对 D22 已定项 2 那一行要回看（`.claude/rules/format-evolution.md` 规定：写「改了」的，决策文件在同一次改动里），现有回看写的是「不受影响」。

### 4.3 C506 销账还欠什么

`.claude/kb/checks-owed.md:328` 的 C506 行要还两件，我核到的现状与多出来的几件：

| 件 | 现状 | 备注 |
|---|---|---|
| ① 区间两条边登记 format-const | kb 0 处（`.claude/kb/checks-owed.md:328` 自述属实；`.claude/gate.d/lib-format-const.py:24` 的文法只认 kb 里的 HTML 标记） | 在 `.claude/kb/layout/01-first-txn.md:140` 的 S 行追加两个标记（2.4）。自证：把标记里的上界改成 17，门禁 code-source-discipline 的 format-constants 格由绿转红（C506 行已写这条；归主 agent 跑，我不跑门禁） |
| ② 条款改写 | 未改，等本轮判决 | 中文、英文、日文三份（`.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:60`、`.ja.md`）一起改；`decisions-history.md` 记用户原话；`.claude/kb/prior-art.md:659` 里「逐字」引的是更早的一版（已写「下界 1」），随带标注 |
| ③ `placeholder` 注释删掉 | 在 | `crates/singlefs-format/src/lib.rs:302`–`:305`（`// placeholder: C506…` 到常量）。门禁 doc-registries 的 format-const-placeholders 格要求占位指得到 `checks-owed.md` 的 C 行（`.claude/gate.d/doc-registries.sh:1545`）——占位行和 C506 行**同一次改动里一起撤**，只撤一边会红 |
| ④ 别处写着「算不出来、按 16 收着」的文字 | 在 | `crates/singlefs-core/src/root_ring.rs:289`、`crates/singlefs-core/src/recovery.rs:499`、`:500`；还有四处把 S 的出处写成「已定项 1 的字段表逐字」（`crates/singlefs-format/src/lib.rs:293`、`:307`，`crates/singlefs-core/src/root_ring.rs:18`、`:289`），而已定项 1 是「journal 取定长环」（`.claude/kb/decisions/22-单元原子性怎么合成.md:39`），S 那一行在已定项 2——旧编号的残留，顺手改 |
| ⑤ 一条越界被拒的用例 | **在，不用新写** | 核心 `crates/singlefs-core/src/root_ring.rs:292`（3 / 17 / 0，边上 4 / 8 / 16 收）、`crates/singlefs-core/src/system_configuration.rs:1030`（3 / 17 / 0 / 255）；整池挂载与池级 checker 同一份镜像 `crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:209`、`:269`（3 与 17）；checker 库单测 `crates/singlefs-checker/src/image.rs:2218`（17）；GPU 对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4611`（17）。下界一侧只有 core 与 harness 有（3），checker 库单测与 GPU 对拍只有上界 17；下界也成格式常量之后这两处缺一格对称用例（checker 档的 GPU 那条我不跑） |
| ⑥ 钉不住的拷贝 | 在 | wgsl 三处字面量 `4u` / `16u`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:960`、`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:906`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:111`）门禁只扫 `.rs` 钉不住；`crates/singlefs-checker-tier/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:63` 的 `_LOCAL` 名字不同钉不住。GPU 对拍的 17 一格是它们唯一的行为级钉子 |
| ⑦ 读者表没有「区域不叠」 | 在（3.3） | 不属 C506，另登一条欠账由主 agent 决定；若补，三个读者同改 |

**什么现象会推翻 V4 的结论**：
1. 候选 A 里任何一句被 `scripts/kb/kb-spec-check.py`、门禁 doc-decisions / doc-text 判红且改不掉（我没跑门禁）；
2. 在 `decisions-history.md` 或 `records/` 里找到一句用户原话把「上界 16」限定在别的意思上（C506 行引的原话是「改条款：16 是格式常量（推荐）」，`.claude/kb/checks-owed.md:328`）；
3. `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:60`、`.ja.md` 里同一格不是同一句（我只读了英文：`**upper bound computed at mount from blocks held by I-7.4 …**`，日文没读）。

## 冲突并排（整行抄）

1. 登记位：
   - `crates/singlefs-format/src/lib.rs:298`：「/// （`singlefs_core::root_ring::RootRingSlotsPerRegion`）。格式承诺的、因而登记成格式常量的，只有区间的两条边。」
   - `.claude/kb/layout/01-first-txn.md:140`：「| 几何 | 每区槽数 S | 1 | 8 | D22（单元原子性怎么合成） 已定项 2 |」（这一行旁没有任何 format-const 标记）
2. 依据段：
   - `.claude/kb/decisions/22-单元原子性怎么合成.md:83`：「- E50（根环槽数的上下界）：它量出的 S 下界与上界都在 4..16 之外，S 不是一个要精算的量。」
   - `.claude/kb/experiments/50-根环槽数的上下界.md:89`：「| **首个够到 1000 块这条线的 N** | **256** | **32** |」
3. 下界随之抬与下界成格式常量：
   - `.claude/kb/decisions/16-发布语义.md:46`：「| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |」
   - 候选句 A：「下界 4 同为格式常量」（冻结之后读者表的边抬不了，2.2）

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 原条款防什么 | 规则没说 | 条款没写它防什么；推出来防的是空间可用性（环窗口 (3S+1)W、被抛弃根隔离、洞里多压一圈），不是任何正确性不变量 |
| V1 常量 16 之后丢什么 | 一致 | 丢的是「随池大小缩上界」，这个上界原条款自己算不出来（循环）、实现里从没实现过；正确性类条款一条没丢 |
| V1 原条款的可用性代价上限 | 规则没说 | 没有条款写 S = 16 在小池上的环窗口代价；模型推的：1 GiB 盘上 13.46%（S = 8 是 6.87%），大盘可忽略 |
| V2 落哪一层 | 一致 | 第 1 层（系统配置槽，`.claude/kb/freeze-layer-membership.md:19`），不新增登记表的行 |
| V2 要不要 incompat 位 | 一致 / 规则没说 | 今天（软格式期）不要；冻结后放宽要、收窄做不到，而 D15 判表没有「放宽读者收的值域」这一行 |
| V2 两条边是否都登记 | 冲突 | 实现注释说登记两条边，kb 一条都没有；应两条都登记在 `.claude/kb/layout/01-first-txn.md:140` 旁 |
| V2 下界随之抬 | 规则没说 | D16 已定项 1 的「随之抬」与「4 是格式常量」冻结后互相够不着 |
| V3 S = 16 放不放得下 | 一致（默认槽距）/ 规则没说（读者表） | 槽距 ≤ 196608 放得下；最大槽距放不下但 S = 4 也放不下；读者表没有「区域不叠」只有 mkfs 判 |
| V3 字节域、其它「3S」 | 一致 | 一字节 0..255；钉槽字节到 S ≤ 85；GPU 根表 48 = 3 × 16 正好；三处消息字面写死 48 |
| V3 mkfs 写的 8 | 一致 | 严格落在区间内；槽距 > 384 KiB 时 8 先于 16 撞到区域叠 |
| V3 E50 那一行 | 冲突 | 撑不住「16 在界外」，散开那一列在 S ≈ 10 就够到线 |
| V4 候选句 | 一致 | 候选 A 与已定项 25、16、2 同格逐条对过，只有 D16 的「随之抬」要补「冻结前」 |
| V4 依据段 | 冲突 → 要改 | E50 那行改写、加用户定案、加 D28 / E168 的代价指针 |
| V4 C506 销账 | 规则没说 / 现状已列 | ① 两个标记 ② 三语改条款 ③ 占位与 C506 行同改 ④ 旧编号与旧文字 ⑤ 越界用例在 ⑥ 钉不住的拷贝 ⑦ 读者表缺口另登 |

## 没做什么

- 没判别：不替主 agent 采纳、不出判决；攻方腿的格我没读（禁读清单里的文件一份也没碰）。
- 没跑实现：环窗口表是 `admission.rs` 的 Python 复写（单元区起点按环长现算、两块同容量盘、K = 144、H = 0），不是 `ring_window_on_one_device` 真跑的读数；没起 cargo、没跑 GPU、没跑门禁（format-constants 格、format-const-placeholders 格、doc-decisions、`kb-spec-check.py`）。要主 agent 跑的：登记两个标记后 `CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK` 不带开关的 format-constants 格由绿到绿，再把标记里的上界改成 17 看它由绿转红。
- 没造镜像：3.3 的「读者表收下叠区域的槽」是读代码推的，没造 S = 16、槽距 262144 的镜像去跑。
- 日文译文没读；英文只读了 S 那一行。
- E50 的 `.out` 只读了 `git show 3cff909b0^:research/results/e50-ring-slots-2026-08-30.out` 里的 `upper_bound` 一行，没复跑 E50。
- 「512 KiB 是 md 的常见块大小」是外部事实，没在本项目验证。
- 不在写范围内的东西（`crates/`、kb、门禁）一个字没动。

## 附：正文里裸写的 `路径:行号` 逐行核对

正文里只写了行号的引用，在这里各配一句那一行里的原文（取那一行里最长的一段，不含「」括号；整行抄在正文对应位置）。

- `.claude/gate.d/code-source-discipline.sh:484`：「if len(places) > 1:」
- `.claude/gate.d/code-source-discipline.sh:496`：「scanned_roots = ["scripts/experiment-apparatus", "research", "crates"」
- `.claude/gate.d/doc-registries.sh:1545`：「if not re.search(rf'^\|\s*C{n}\s*\|', owed, re.M):」
- `.claude/gate.d/lib-format-const.py:24`：「MARK_GRAMMAR = re.compile(」
- `.claude/kb/checks-owed.md:328`：「| C506 | 每区槽数 S 写成编译期常量，条文说它住系统配置 | 每区槽数 S 已是系统配置字段（crates/s」
- `.claude/kb/decisions/15-格式冻结政策.md:189-197`：「| 新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否 | 没有语义依赖的纯加法，」
- `.claude/kb/decisions/15-格式冻结政策.md:199`：「。第一行那条（预留区里的新增是 compat）只在真有预留区时成立，索引节点头只有 2 字节预留，装得下的字段很少。」
- `.claude/kb/decisions/16-发布语义.md:35`：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」
- `.claude/kb/decisions/16-发布语义.md:38`：「那一行把读不出的槽并进回收门槛同一个思路）。读坏槽并进上限、卡住就在这次挂载里报 ENOSPC、不推空发布；读回全零的槽」
- `.claude/kb/decisions/16-发布语义.md:46`：「| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随」
- `.claude/kb/decisions/16-发布语义.md:62`：「- 回收门槛取环里最旧有效根的代价：崩在记录之后、根槽之前那一形留在环里的旧根要等被覆盖一圈才离开，每个洞回收门槛多压一」
- `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:60`：「| Slots per region S | Resides in system configuration. Lowe」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:241`：「| 系统不可变配置 | 389 字节 | 重建文件系统 | mkfs 时设：fsid 由 mkfs 的对外入口从随机源生」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:350`：「那条约束已经没有理由（它唯一的理由是 gcd(P, devs) = 1 的双射，已随 D2（RAID 条带策略） 已定项」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:495`：「；上一代根能不能用取决于块重用延迟，那是另一件事、另一笔成本。把两者混为一谈会让人以为环是免费的。 ⚠️ 块复用的界不能」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:60`：「| 每区槽数 S | 住系统配置。下界 k_tol + 2 = 4（D16（发布语义） 已定项 1：每区槽数 S 要装得」
- `.claude/kb/decisions/28-挂载期承诺量.md:31`：「- 环窗口（常驻扣项）：删除各批新写的兜底，环窗口 = (根环槽数 3 × S + 1) × W，S 是系统配置每区槽数」
- `.claude/kb/experiments/168-删除借保留量的环窗口.md:11`：「：65 格删除被拒，最小的反例是 P1、S = 16、K = 1、N = 8（单元区 149 槽）；但登记第 400 行」
- `.claude/kb/experiments/34-根环槽几何.md:93`：「#### 几何前提（产物 name=v1_geometry）：io_min = 64 KiB 在本仓真设备上量到了」
- `.claude/kb/freeze-layer-membership.md:19`：「| 系统配置槽（码 18 自证单元，每盘两槽轮换，字段表 489 字节） | 第 1 层 | 判不动 | 不退出 | D」
- `.claude/kb/layout/01-first-txn.md:140`：「| 几何 | 每区槽数 S | 1 | 8 | D22（单元原子性怎么合成） 已定项 2 |」
- `.claude/kb/prior-art.md:659`：「I-7.4（近 K 代块未被复用） 扣住的块数挂载时算」
- `crates/singlefs-checker/invariants/02-根环.md:11`：「，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用」
- `crates/singlefs-checker/invariants/02-根环.md:20`：「并提前返回。八项 checker 与 core 两边都判：checker 解槽时判格式版本与加密类型（crates/si」
- `crates/singlefs-checker/src/image.rs:2218`：「fn a_slot_declaring_seventeen_slots_per_region_refuses_the_w」
- `crates/singlefs-checker/src/image.rs:734`：「let slots_per_region = u64::from(slot[362]);」
- `crates/singlefs-checker/src/image.rs:735`：「if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PE」
- `crates/singlefs-checker/src/image.rs:738`：「return Err(crate::Verdict::RootRingSlotsPerRegionOutsideTheF」
- `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs:70`：「const ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4;」
- `crates/singlefs-checker-tier/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs:63`：「const ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL: u64 = 16;」
- `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:111`：「if (slots_per_region < 4u || slots_per_region > 16u) {」
- `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:241`：「const MAX_ROOTS: u32 = 48u;」
- `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:960`：「if (slots_per_region < 4u || slots_per_region > 16u) {」
- `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:906`：「return slots_per_region >= 4u && slots_per_region <= 16u &&」
- `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4611`：「fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configura」
- `crates/singlefs-core/src/admission.rs:500`：「let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_p」
- `crates/singlefs-core/src/allocator.rs:4325`：「RootRingSlotsPerRegion::from_system_configuration_field(4).e」
- `crates/singlefs-core/src/make_filesystem.rs:528`：「let extents = fixed_structure_extents(geometry);」
- `crates/singlefs-core/src/mounted_session.rs:2339`：「.expect("至多环长 3S ≤ 48 次");」
- `crates/singlefs-core/src/mount.rs:10118`：「let slots_per_region = RootRingSlotsPerRegion::from_system_c」
- `crates/singlefs-core/src/mount.rs:2310`：「usize::try_from(ROOT_RING_REGIONS * slots_per_region.count()」
- `crates/singlefs-core/src/mount.rs:4347`：「/// 一次准入最多推几次发布：B = 4 + 2 k_tol，k_tol = 2 ⇒ 8（D16（发布语义） 已定项」
- `crates/singlefs-core/src/recovery.rs:1095`：「fn system_configuration_values_this_reader_accepts(」
- `crates/singlefs-core/src/recovery.rs:1344`：「/// 槽 1 整槽落在根环基址（[region_start] 的区域 0，1 MiB）之前——mkfs 判固定结构两两」
- `crates/singlefs-core/src/recovery.rs:499`：「/// 下界 k_tol + 2 = 4；上界字段表写的是」
- `crates/singlefs-core/src/root_record.rs:74`：「pub struct PinnedRingSlot(pub u8);」
- `crates/singlefs-core/src/root_record.rs:93`：「.expect("钉值是那一槽可能住过的最旧 txg，小于环长 3S ≤ 48：装得进一个字节、撞不上」
- `crates/singlefs-core/src/root_ring.rs:115`：「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_reg」
- `crates/singlefs-core/src/root_ring.rs:144`：「slots_per_region.count() * u64::from(fixed_structure_slot_sp」
- `crates/singlefs-core/src/root_ring.rs:162`：「RootRingSlotsPerRegion::from_system_configuration_field(decl」
- `crates/singlefs-core/src/root_ring.rs:169`：「assert_eq!(region_start(1), DeviceOffsetInBytes(4 << 20));」
- `crates/singlefs-core/src/root_ring.rs:18`：「/// 每区槽数 S：系统配置字段，不是格式常量（D22（单元原子性怎么合成） 已定项 1 的字段表逐字：」
- `crates/singlefs-core/src/root_ring.rs:289`：「/// 区间判（D22 已定项 1 字段表：下界 k_tol + 2 = 4，上界今天算不出来、按 16 收着）：」
- `crates/singlefs-core/src/root_ring.rs:292`：「fn slots_per_region_outside_the_format_interval_is_refused_w」
- `crates/singlefs-core/src/root_ring.rs:326`：「"全 0 的槽也拒：S = 0 会让环长变 0、取模除零"」
- `crates/singlefs-core/src/root_ring.rs:50`：「slots_per_region > ROOT_RING_SLOTS_PER_REGION_MINIMUM」
- `crates/singlefs-core/src/root_ring.rs:65`：「if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PE」
- `crates/singlefs-core/src/root_ring.rs:68`：「return Err(RootRingSlotsPerRegionOutOfRange {」
- `crates/singlefs-core/src/root_ring.rs:96`：「ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_」
- `crates/singlefs-core/src/system_configuration.rs:1030`：「fn self_describing_slot_declaring_an_out_of_range_slots_per_」
- `crates/singlefs-core/src/system_configuration.rs:58`：「const ROOT_RING_SLOTS_PER_REGION_OFFSET: u64 = 362;」
- `crates/singlefs-core/src/system_configuration.rs:670`：「let root_ring_slots_per_region = RootRingSlotsPerRegion::fro」
- `crates/singlefs-format/src/lib.rs:238`：「pub const ROOT_RECORD_NO_RING_SLOT_PINNED: u8 = 255;」
- `crates/singlefs-format/src/lib.rs:282`：「/// 固定结构槽距的下限；槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍（D2（RAID」
- `crates/singlefs-format/src/lib.rs:299`：「/// format-const: ROOT_RING_SLOTS_PER_REGION_MINIMUM」
- `crates/singlefs-format/src/lib.rs:302`：「// placeholder: C506（每区槽数 S 写成编译期常量，条文说它住系统配置） —— 字段表给的上界是」
- `crates/singlefs-format/src/lib.rs:304`：「// 候选集又按 F 与实例表现算，仓里没有一条式子从池的参数给得出 S 的上界。在它算得出来之前按 16 收着，区间判」
- `crates/singlefs-format/src/lib.rs:309`：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64」
- `crates/singlefs-harness/tests/system_configuration_slots_per_region.rs:209`：「fn mounting_refuses_a_slots_per_region_outside_the_interval_」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:14`：「name=cell spacing=65536 S=16 region_bytes=1048576 fits_one_c」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:20`：「name=cell spacing=196608 S=16 region_bytes=3145728 fits_one_」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:23`：「name=cell spacing=262144 S=16 region_bytes=4194304 fits_one_」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:25`：「name=cell spacing=393216 S=8 region_bytes=3145728 fits_one_c」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:3`：「name=largest_spacing_that_keeps_regions_apart S=4 bytes=7864」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:33`：「name=cell spacing=1044480 S=4 region_bytes=4177920 fits_one_」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:35`：「name=cell spacing=1044480 S=16 region_bytes=16711680 fits_on」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:36`：「name=other_limits pin_byte_largest_S=85 gpu_max_roots=48 gpu」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:40`：「name=ring_window device_gib=1 journal_ring_mib=64 S=16 W=180」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:46`：「name=ring_window device_gib=4 journal_ring_mib=768 S=16 W=37」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:52`：「name=ring_window device_gib=100 journal_ring_mib=768 S=16 W=」
- `research/prompts/m3-close-c506-r1-sonnet-model/out/model.out:8`：「name=cell spacing=4096 S=16 region_bytes=65536 fits_one_chun」
- `scripts/experiment-apparatus/e50_ring_slots.rs:168`：「/// 散开下 30 槽就够到 ⇒ 首个是 32。」
- `scripts/experiment-apparatus/e50_ring_slots.rs:39`：「const PINNED_MATTERS: u64 = 1000;」
- `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:113`：「const ROOT_RING_REGION_STRIDE: u32 = 3145728u;」
