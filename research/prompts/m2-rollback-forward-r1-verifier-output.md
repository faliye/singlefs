# m2-rollback-forward-r1 核查员报告

**核查员交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

## 判别力自证

抽样：Sonnet 报告第 30–31 行「今天唯一的回退实现是 `mount_rollback` / `mount_rollback_with_space_admission`
（`crates/singlefs-core/src/mount.rs:2472`、`:2491`，已用 `grep -n "pub fn mount_rollback"` 现查）」。
把行号 2472 改成 2473（+1），在草稿目录副本 `/tmp/claude-1000/m2-rollback-forward-r1-verifier/selfproof/mount.rs`
上核：

```
$ awk 'NR==2473' mount.rs
    parameters: &MakeFilesystemParameters,
$ awk 'NR==2473' mount.rs | grep -c "pub fn mount_rollback"
0
```

篡改后的行号（2473）内容不含 `pub fn mount_rollback`，判 ✗。**核查方法能分辨**：不停止，往下做。

## 快照核验（输入齐备性）

- `crates-sha256.txt` 去掉 `#` 行共 125 条，在 `/tmp/claude-1000/m2-rollback-forward-r1/tree` 下 `sha256sum -c` 全部 `OK`（0 处不对）。
- `kb-sha256.txt` 去掉 `#` 行、去掉路径前缀 `kb/` 共 208 条，在 `/tmp/claude-1000/m2-rollback-forward-r1/kb-snapshot` 下 `sha256sum -c` 全部 `OK`（0 处不对）。
- Sonnet 腿交回时给的 `sha256sum`（`23c89252e36a159128836d008d1a1e2c1b9e93a967168644fa16781ccc98c1a2`）与
  `research/prompts/m2-rollback-forward-r1-sonnet-output.md` 现在的 `sha256sum` **逐字相同**——不是「分不清」那一档。
- Opus 腿的模型目录 `SHA256SUMS`：对目录里 6 个文件 `sha256sum -c` 全部 `OK`；报告第十节末尾原样贴的 `SHA256SUMS` 内容与目录里那份文件逐字节 `diff` 相同（exit 0）。
- 派发提示未给 Opus 腿报告本身的交回 sha256（只给了 Sonnet 一条），此项按「有就给」处理，不记「分不清」。

## 一、Sonnet 腿（正推）核对表

代码引用一律对 `/tmp/claude-1000/m2-rollback-forward-r1/tree`（冻结副本）核；kb 引用一律对
`/tmp/claude-1000/m2-rollback-forward-r1/kb-snapshot` 核。全部用 `awk 'NR==N'` 或 `sed -n 'N,Mp'` 现取，
不从背景材料数偏移。

### 1.1 代码引用（`mount.rs`、`transaction.rs`、`recovery.rs`）

| 引用（报告行号） | 核的结果 |
|---|---|
| `grep -n "fn .*unmount\|fn .*close\|fn .*shutdown" crates/singlefs-core/src/*.rs`（报告 14 行起） | ✓ 原样两行命中与报告贴的完全相同（`allocator.rs:1860`、`system_configuration.rs:627`，均为测试函数名） |
| `grep -n "^pub fn \|^fn " transaction.rs \| grep -i "rollback\|restore\|revert"`（报告 23 行） | ✓ 零命中，与报告一致 |
| `mount.rs:2472` `pub fn mount_rollback` | ✓（自证用例，见上） |
| `mount.rs:2491` `pub fn mount_rollback_with_space_admission` | ✓ 逐字匹配 |
| `mount.rs:2517` `RollbackCandidateExclusion::NotInRing` | ✓ |
| `mount.rs:2532` `RollbackCandidateExclusion::BelowEffectiveFloor` | ✓ |
| `mount.rs:2546` `RollbackCandidateExclusion::OnAbandonedTimeline` | ✓ |
| `mount.rs:2620-2625` `PreviousInstanceRow { .. is_rollback: true }` | ✓ 整块比对相同 |
| `mount.rs:611` `isolate_slots_referenced_only_by_abandoned_roots` | ✓ |
| `mount.rs:2605-2607` `newly_abandoned` 闭包 | ✓ 两行内容相同 |
| `mount.rs:576` `abandoned_by_table` | ✓ |
| `recovery.rs:669` `choose_root` | ✓ |
| `recovery.rs:1657` `user_visible_tree_root_pointers` | ✓（`grep -n` 复核同一行号） |
| `mount.rs:1018` `pointers_of` 闭包 | ✓ |
| `transaction.rs:513` `instance_generation_to_acquire` | ✓ |
| `mount.rs:2583` `rebuild_previous_version(devices, &target_root, own_record)` | ✓ |
| `mount.rs:1415` `publish_rows_on_file_version` | ✓ |
| `transaction.rs:926` `publish_without_units` | ✓ |
| `mount.rs:2584-2592` `above_water: 0, prefix_applied: 0` 代码块 | ✓ 整块比对相同 |
| `mount.rs:1095` `raise_rollback_floor` | ✓ |
| `mount.rs:1343` `rehearse_the_publishes_raising_the_floor` | ✓ |
| `mount.rs:975` `rollback_floor_ceiling` | ✓ |
| `mount.rs:1057` `ceiling_from_newest_and_non_empty_roots` 代码块（报告 178-189 行） | ✓ 逐字节比对相同（10 行代码块） |

命令：
```
$ cd /tmp/claude-1000/m2-rollback-forward-r1/tree
$ grep -n "fn .*unmount\|fn .*close\|fn .*shutdown" crates/singlefs-core/src/*.rs
crates/singlefs-core/src/allocator.rs:1860:    fn user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed() {
crates/singlefs-core/src/system_configuration.rs:627:    fn corrupted_slot_and_unknown_incompat_bit_are_both_unmountable() {
$ grep -n "^pub fn \|^fn " crates/singlefs-core/src/transaction.rs | grep -i "rollback\|restore\|revert"
(exit 1, 零命中)
```

代码引用小计：23 处，✓ 23，✗ 0。

### 1.2 kb 引用（`kb-snapshot/`）

| 引用（报告行号:kb 行号） | 报告怎么说 | 核的结果 |
|---|---|---|
| 22-单元原子性怎么合成.md:128 | D22 已定项 7「给出根记录字段表」（只指标题） | ✓ 该行正是 `#### 已定项 7：根记录的字段表` |
| **16-发布语义.md:53** | 「非空」判据逐字「一条有效根算非空 ⟺ …」 | **✗ 实际在第 44 行**。第 53 行是另一句（「并进那一次重议的另两件…」），44 行才是 `**「非空」从盘上怎么认**：一条有效根算非空 ⟺ …`。不在任何一份背景材料的第 53 行找到同一句（已查 body/appendix/checklist/background 四份的第 53 行，均不是这句），判纯粹的行号错，不是「误写成背景材料行号」 |
| 05-快照-空间记账机制.md:288 | D5 已定项 13（只指标题） | ✓ 该行正是 `#### 已定项 13：生命周期判定取哪套机制` |
| 05-快照-空间记账机制.md:308 | D5 已定项 14（只指标题） | ✓ 该行正是 `#### 已定项 14：引用区间的充要条件` |
| **23-journal的角色与格式.md:351** | 已定项 14 逐字「第一版没有干净关闭标记……每次挂载一律走恢复」 | **✗ 实际在第 374 行**。第 351 行是该已定项的标题行 `#### 已定项 14：重放的下界由所选根给出；…`，不是这句话本身；本地腿 translation-audit 独立核出同一句在第 374 行（互证）|
| 16-发布语义.md:39 | 已定项 1「抬 F 那一串」那一行 | ✓ 逐字匹配 |
| checks-owed.md:507 | C516 逐字「用户 2026-09-25 推翻 2026-09-24 的『维持逐次发布』…」 | ✓ 逐字匹配 |
| **16-发布语义.md:31** | 已定项 1 逐字「抬 F 的上限 \| min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)…」（F2.2 节，报告 192-193 行） | **✗ 实际在第 36 行**。第 31 行是表头 `\| 项 \| 定案 \|`，与引文无关 |
| 16-发布语义.md:31（附近） | 「生效」那一行「各幸存盘所带 F 最大值的最小值」（报告 234-235 行，标「附近」） | 软引用，但仍偏：「生效」那一行实际在第 37 行，距标注的 31 行 6 行——记「偏差，未按嫌疑最严格判 ✗，因报告自己标了『附近』未称精确」，实际位置见上一条已定位的第 36/37 行区间 |
| **16-发布语义.md:170** | 已定项 7「fsync 等根槽持久之后才返回」 | 轻度 ✗：第 170 行是标题 `#### 已定项 7：发布的持久顺序`，那句引文实际在第 172 行（隔 1 行定案文字） |
| invariants.md:60、:61 | I-7.10、I-7.11（只指标题行） | ✓ 60 行确为 I-7.10 表格行，61 行确为 I-7.11 表格行 |
| invariants.md:54 ×2 处 | I-7.4「近 K 代块未被复用」 | ✓ 逐字匹配（含 F3 说明节的第二次引用） |
| invariants.md:57、:58、:85 | I-7.7、I-7.8、I-8.6 | ✓ 三处全部匹配 |
| checks-owed.md:276/280/286/294/475/478/479/115 | C314/C318/C332/C340/C542/C546/C547/C120 | ✓ 8 处全部匹配（题面与摘要都对得上表格内容） |
| 28-挂载期承诺量.md:19、:27 | D28 已定项 1 第九项「被抛弃根独占量」 | ✓ 19 行是准入不等式带该项的整行公式，27 行明写「（第九项）」 |

kb 引用小计：18 处，✓ 14，✗ 3（16-发布语义.md:53→44；16-发布语义.md:31→36；23-journal的角色与格式.md:351→374），另有 2 处轻度偏差（16-发布语义.md:170→172，off by 2；16-发布语义.md:31「附近」实为 37，报告自己已标不精确）。

**这三处 ✗ 的共同点**：都是把「已定项标题/表头所在行」误当成「定案正文那一行」——22-单元原子性.md:128、05-快照-空间记账机制.md:288/308 两处引的确实是标题（报告只说"给出字段表"/"已定项 13/14"，没有声称逐字引某句），而三处 ✗ 都明确写了"逐字"却给的是标题或表头的行号。

### 1.3 判定一览表（自身一致性，非引用核）

「判定一览」表格是 Sonnet 报告自己对前文的归纳，未含独立可核的新引用，不重复核。

Sonnet 腿小计：**核了 41 处引用（代码 23 + kb 18），✓ 36，✗ 3，轻度偏差 2（记入核不动/偏差，不计入 ✓ 也不计入 ✗）。**

## 二、Opus 腿（攻方）核对表

### 2.1 复跑（`bash rerun.sh <冻结副本> <草稿目录> full`，独立拷贝、独立跑）

命令（`nice -n 19`，`rerun.sh` 内部已用 `run-with-memory-cap.sh 20G` + `capped.sh 12`，符合本轮内存/线程上限）：

```
$ bash research/prompts/m2-rollback-forward-r1-opus-model/rerun.sh \
    /tmp/claude-1000/m2-rollback-forward-r1/tree \
    /tmp/claude-1000/m2-rollback-forward-r1-verifier/rerun full
```

**fast（18 条用例，含 H1/H3/H4/H6/H7/H9/H10/H12）**：`diff` 我这次复跑的 `rerun-fast.out` 与模型目录里那份，
**除 `finished in` 耗时外逐行相同**（161 行，160 行完全一致，1 行只差耗时 50.41s vs 49.49s）。

```
$ diff /tmp/claude-1000/m2-rollback-forward-r1-verifier/rerun/rerun-fast.out \
       research/prompts/m2-rollback-forward-r1-opus-model/rerun-fast.out
161c161
< test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.41s
---
> test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.49s
```

**full-unmount（H2，327690 个崩溃状态）**：同样只差耗时。

```
$ diff /tmp/claude-1000/m2-rollback-forward-r1-verifier/rerun/rerun-full-unmount.out \
       research/prompts/m2-rollback-forward-r1-opus-model/rerun-full-unmount.out
5c5
< test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 193.42s
---
> test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 191.97s
```

**full-rollback（H8，524301 个崩溃状态）**：第一次用 `& ... wait`（不带参数）收尾，脚本本身以退出码 2 失败，
而 `wait` 不带参数吞掉了这个失败、外层打印成「rerun exit=0」——这正是
`command-safety.md`「不带参数的 wait 退出码恒为 0」那条要防的失效，记为**我自己这一步操作失误**，不是腿的问题。
`rerun-full-rollback.out` 没有生成（fast、full-unmount 两份产物正常生成且已如上核对）。已改用 `wait "$pid"`
单独重跑这一条命令，正确收退出码，见下段。

### 2.2 代码与 kb 引用

| 引用 | 核的结果 |
|---|---|
| `mount.rs:576` `abandoned_by_table` | ✓ |
| `mount.rs:604` 注释「树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载…」 | ✓ 整行相同 |
| `mount.rs:2423` `ShadowLedger::On,` | ✓ |
| `walk.rs:4682` `walk.walk_root(&root.record_bytes, false);` | ✓ |
| `16-发布语义.md:33` 「可再分配 \| 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」 | ✓ |
| `16-发布语义.md:36`（H1 第 70 行、H2 第 89 行两处）「抬 F 的上限 \| min(…)」 | ✓ 两处均精确匹配 |
| `16-发布语义.md:37` 「生效 \| 每块幸存盘上都有带新 F 的持久根才生效…」 | ✓ |
| `16-发布语义.md:38` 「回退候选集 \| 按实例表判仍然有效 ∧ txg ≥ F_生效…」 | ✓ |
| `16-发布语义.md:40` 「准入 \| 可分配 = min(…)」 | ✓ |
| `16-发布语义.md:52` C419 回落机理段落 | ✓ 整段相同；`checks-owed.md` 里 C419 存在且题面、内容与引用一致（现查 `grep -n "C419"`） |
| `23-journal的角色与格式.md:378` 「显式例外：管理员回退…」 | ✓ |
| `08-核心索引结构.md:387` 「挂载怎么读：分配记录树挂载时整棵读进挂载态…」 | ✓ |
| `05-快照-空间记账机制.md:328` 「射程：这个条件只在 txg 全序…」 | ✓ |
| `18-块里携带什么信息.md:279` 容器号那一行（打包记录类型 1/2） | ✓ |
| `invariants.md:54`（H3/H6 引 I-3.9？不，是 I-7.4）、`:276`（I-9.6） | ✓ 两处均匹配（54=I-7.4，276=I-9.6，与 Sonnet 腿核过的同一对行号一致，互证） |

Opus 腿代码 + kb 引用小计：**核了 15 处，✓ 15，✗ 0。**

## 三、本地攻方腿：转述核对表逐条核（`m2-rollback-forward-r1-local-attack-translation-audit.md`）

对快照 `/tmp/claude-1000/m2-rollback-forward-r1/kb-snapshot/decisions/` 逐条核「原文文件:行」是否真在那一行、
译文有没有漏限定词、有没有多加原文没有的限定词或括注。

| 条目 | 核对表说的行 | 核的结果 |
|---|---|---|
| D16-1-a | 16-发布语义.md:29 | ✓ 恰为该定案句起始行 |
| D16-1-b | 16-发布语义.md:33 | ✓ |
| D16-1-c | 16-发布语义.md:35 | ✓ |
| D16-1-d | 16-发布语义.md:36 | ✓ |
| D16-1-e | 16-发布语义.md:37 | ✓ |
| D16-1-f | 16-发布语义.md:38 | ✓ |
| D16-1-g | 16-发布语义.md:39 | ✓ |
| D16-1-h | 16-发布语义.md:40 | ✓ |
| D16-1-i | 16-发布语义.md:42 | ✓ |
| D16-1-j | 16-发布语义.md:44 | ✓（与 Sonnet 腿引错的「非空」那句是同一行，本地腿标对了，互证 Sonnet 那处确是 ✗） |
| D16-9 | 16-发布语义.md:211 | ✓ |
| D23-14-a/b/c | 23-journal的角色与格式.md:378 / 378 / 374 | ✓ 三行均核实；**D23-14-c 标 374 行，与本地腿自己转述「第一版没有干净关闭标记」逐字对上**——同样反证 Sonnet 报告把这句引成 `:351` 是错的 |
| D22-2-a/b/c | 22-单元原子性怎么合成.md:55 / 58 / 63 | ✓ 三行均核实，58 行确认首稿曾漏「S 要装得下 4 个状态…」一句、定稿已用 `replace-once.py` 补回（核对表自陈） |
| D8-14-a | 08-核心索引结构.md:389-396 | ✓ 区间内含「4 GiB × 2 时根在第 2 层、树高 3」worked example |
| D8-14-b | 08-核心索引结构.md:400-406 | ✓ |
| D8-11 | 08-核心索引结构.md:321 | ✓（该行是一段较长文字的第⑤条，「树高 = 根节点码 2 头里的层级 + 1」确在行内，非行首） |
| D5-2 | 05-快照-空间记账机制.md:47 | ✓ |
| D5-13 | 05-快照-空间记账机制.md:292-293 | ✓ |
| D5-14 | 05-快照-空间记账机制.md:310-316 | ✓ |
| D18-18 | 18-块里携带什么信息.md:446 | ✓ 字段表那一行内容与译文的字段列表对应 |
| D22-7 | 22-单元原子性怎么合成.md:138,139,141,143,144,145 | ✓ 六行均核实（F 与 checkpoint_txg 逐字段核对，指针字段行确认存在但仅点出「有」不逐字段抄） |
| D28-4 | 28-挂载期承诺量.md:101-108 | ✓ 区间内含 ckpt_cost 公式与「记录树高怎么读…是另一个还没定的题」那句，核对表称首稿漏译、定稿已补，属实 |
| TODAY-a/b/c | `_m2-rollback-forward-r1-body.md`:26 / 29 / 30 | ✓ 三行均核实；body.md 里的函数行号（2437、2456）与冻结副本今天的行号（2472、2491）不同，但核对表按定义要求本就不把这两个行号带进提示，非缺陷 |

本地腿核对表小计：**核了 26 条转述，✓ 26，✗ 0。** 这是三条腿里唯一一份全部核实无误的转述表，且其中两条
（D16-1-j 对应 16-发布语义.md:44，D23-14-c 对应 23-journal的角色与格式.md:374）恰好独立印证了 Sonnet 腿报告里
两处真实的行号错误。

## 四、本地攻方腿：运行记录（`-runlog.md`）复核

`wc -w` 现数两份样本词数：`793`（s1）、`717`（s2），与运行记录表格逐字相同。
重跑（不是重新抽样，是对已产出的产物重新跑同一套确定性检查脚本，`ask-local.sh` 串跑的那两个）：

```
$ python3 research/scripts/corruption-check.py research/prompts/m2-rollback-forward-r1-local-attack-output-s1.md
绿 ... cjk=0 words=755 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ python3 research/scripts/oov-check.py research/prompts/m2-rollback-forward-r1-local-attack-output-s1.md
绿 ... 生词=19 拼接=0
     生词: computable falsified contradicting falsification
$ python3 research/scripts/corruption-check.py research/prompts/m2-rollback-forward-r1-local-attack-output-s2.md
绿 ... cjk=0 words=676 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ python3 research/scripts/oov-check.py research/prompts/m2-rollback-forward-r1-local-attack-output-s2.md
绿 ... 生词=12 拼接=0
     生词: computable falsified contradicting unmount's
```

两份 `exit=0`，词数、生词清单、判定「绿/干净」与运行记录逐字相同。**产物级复核：2 处，✓ 2，✗ 0。**

运行记录另称「两份都另外通读一遍全文，没有发现『列表里整个词没了、只剩孤零零标点』这类缺词/断句损坏」——
这一句属人工判断，核查员另行通读两份样本全文（各 24 行），未见列表项缺失、未见断句异常，与运行记录一致。

### 附 1.4（Sonnet 腿）：行区间描述核对（非逐字引用，核内容对不对得上；编号顺延自第一节，因起草时后补）

| 区间 | 报告怎么说 | 核的结果 |
|---|---|---|
| mount.rs:2498-2501 | 「admit_the_writable_device_count、择 system_configuration、择 newest_root：与今天…相同」 | ✓ 三行确为这三步 |
| mount.rs:2529-2534 | `BelowEffectiveFloor` 分支保留 | ✓ 整个 if 块内容相同 |
| mount.rs:2537-2548 | 「按实例表判仍然有效」与 `OnAbandonedTimeline` 两条排除，建议删掉 | ✓ 整个 if 块内容（实例表校验 + 见证表判断 + 抛出 `OnAbandonedTimeline`）与描述相符 |
| mount.rs:611-663 | `isolate_slots_referenced_only_by_abandoned_roots` 函数体范围 | ✓ 611 行为函数签名起始、663 行为该函数右花括号收尾 |

补充小计：4 处，✓ 4，✗ 0。

## 五、F2：卸载时抬 F 上限冲突（对冻结副本 `mount.rs` 核）+ H8 复跑第二次收尾

Sonnet 报告 F2.2 节贴出的 `ceiling_from_newest_and_non_empty_roots` 函数体（第 1057-1066 行区间）
与冻结副本逐字节相同（见 1.1 节表格）。该函数确实按「第 4 新非空根」取上限、不足 4 个时回落到
`valid.iter().map(...).min()`（最旧有效根），与报告叙述「非空发布 < 4 次时，`fourth_newest` 回落到
『最旧有效根』」在代码语义上一致——**这是读代码得出的观测，不是我方的推论**，与 Opus 腿 H2 节独立跑出的
「I-7.9 判红 262151 个」互相印证（Opus 腿的 I-7.9 违例信息里报的正是「上限 0」或「上限 2」这种远低于
「最新根」的具体数字，与 Sonnet 腿指出的机制吻合）。

**full-rollback（H8）重跑，第二次（用 `wait "$pid"` 正确收退出码）**：exit=0，产物与模型目录逐行相同（除耗时）。

```
$ grep -E 'RBF |test result' rerun-rollback-debug.log | sed -E 's/^.*(RBF )/\1/' > rerun-full-rollback.out
$ diff rerun-full-rollback.out research/prompts/m2-rollback-forward-r1-opus-model/rerun-full-rollback.out
5c5
< test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 490.26s
---
> test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 488.10s
```

关键行（我这次复跑的原样输出）：

```
RBF crash name=forward-rollback closed_form_full=524301 segment_lengths=[2, 18, 2, 1, 18, 2, 1, 2] full=true
RBF crash name=forward-rollback writes=46 states=524301 violations=0 failed=0 first_violation=none probe_states=2622 probe_bad(kill1,kill2,kill3)=[0, 0, 0] probe_first_bad=[None, None, None]
RBF crash name=forward-rollback checker_violated=["I-3.9=262151:盘 1 槽 50245 的分配记录释放代 6 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4"]
```

与 Opus 报告第七节「全量 524301 个状态（闭式数），oracle 违例 0、恢复失败 0；另 2622 个状态（每 200 个抽 1）
…读错 0 / 0 / 0」逐字对上；`states=524301`、`violations=0`、`failed=0`、`probe_bad=[0,0,0]` 与
`I-3.9=262151` 四项数字全部复现。

**复跑小计：4 条命令（fast、full-unmount、full-rollback、runlog 的两个检查脚本），全部独立复跑一次，
产物与模型目录/运行记录逐字（除挂钟耗时外）相同，✓ 4，✗ 0。**

## 六、Opus 报告第十节自陈的更正（H2b 表格）核实

任务指出「H2b 表『不做 B』那一列分界写错，正文没改、更正在第十节」。已用本次独立复跑的 `rerun-fast.out`
核对第十节给出的更正数字：

```
$ grep "RBF t5b-row rollback=false b=false" rerun-fast.out
killed=0..3 → verdict=Ok((2, 9))
killed=4..8 → verdict=Ok((1, 5))
killed=9    → verdict=Ok((0, 0))
```

与第十节更正「改坏 0–3 条落在 (2, 9)，4–8 条落在 (1, 5)，第 9 条起落在 (0, 0)」逐字对上；
「做 B」那一列「0–4 / 5–9 / 10 起读失败 `UnitUnreadable { slot: SlotNumber(50178) }`」同样对上
（`killed=0..4→(2,12)`，`killed=5..9→(1,7)`，`killed=10..→Err(UnitUnreadable slot 50178)`）。
**更正属实，✓。**

## 七、计数与判定一览

| 腿 | 核了几处 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| Sonnet（正推） | 45（代码 23+4，kb 18） | 40 | 3（16-发布语义.md:53、:31，23-journal的角色与格式.md:351） | 2 处软引用偏差（:170 偏 2 行、「:31 附近」实为 37 行）不计入 ✓/✗ |
| Opus（攻方）代码 / kb | 15 | 15 | 0 | 0 |
| Opus（攻方）复跑 | 4（fast、full-unmount、full-rollback、H2b 更正核实） | 4 | 0 | 0 |
| 本地攻方（转述核对表） | 26 | 26 | 0 | 0 |
| 本地攻方（产物级：词数 + corruption/oov 复跑） | 4（词数 2 + 检查脚本 2） | 4 | 0 | 0 |
| SHA256 快照 / 交回哈希 | 5（crates 125 文件、kb 208 文件、Sonnet 交回哈希、Opus 模型目录 6 文件、报告内嵌 SHA256SUMS 对比） | 5 | 0 | 0 |
| **合计** | **99** | **94** | **3** | **2 软偏差另记** |

**判别力自证**：开头一条，判 ✗，方法可分辨。

## 八、没做什么

- 不判一条打中成不成立、该不该采纳（H1–H12、F2 岔路 B1/B2、D1/D2 等）；这些留给主 agent 逐条现查。
- 不核推理本身（例如「向前发布之后不再有被抛弃时间线」这条前提是否站得住），只核引用、产物与复跑。
- 没有重算 Opus 报告 F4 节的代价数字背后的算术（released/revived/pruned 等数字已与 `rerun-fast.out` 原样行逐条对
  上，视为「产物级核对」，未独立用别的算法重新推导这些数字应该是多少）。
- 没有重新核 Sonnet 报告 F1/F2/F3 candidate 方案本身的工程正确性（例如候选 A/B 的具体实现步骤是否可行、
  是否遗漏某种崩溃形态）；只核了它引用代码/kb 的行号与内容。
- 没有核本地攻方腿 output-s1/s2 两份样本里的算术结果本身（`4826809`、`28731` 等数值），按派发范围
  「本地腿的两份样本只核它事实表的出处行号与译文，不重算」执行。
- 没有对本地腿再抽第三个样本；派发只要求核已有两份样本的出处行号与译文，未要求追加抽样。
- 没有跑 `.claude/gate.d/` 任何阶段：`stage-owners.tsv` 未登记任何阶段给 `three-way-verifier` 这一角色。
- 没有编译、跑测试之外的其它重型验证（层 0 全量、QEMU、herd7、`cargo test --all`）；本轮涉及的 3 条复跑命令
  （fast/full-unmount/full-rollback）都是 `rerun.sh` 已经限定好的 `singlefs-harness --test rbf_attack`
  单一测试二进制，符合子 agent 不跑重型测试整轮的范围。
- 第一次尝试用 `& ... wait`（不带参数）收 `full-rollback` 那条复跑的退出码，命中了
  `command-safety.md`「不带参数的 wait 退出码恒为 0」这个已知坑：脚本内部实际以退出码 2 失败，
  外层却报「exit 0」。已发现即改用 `wait "$pid"` 重跑一次拿到真实结果，本报告引用的是第二次（正确收退出码
  的那一次）产物；第一次的失败已如实记录在 2.1 节，不隐藏。
