# m3-batch20-code-r1 攻方腿（Opus）报告（2026-10-05）

攻击面：正文第三节 K2（重点 N1、N3、N5，另带 N9）。冻结副本 `/tmp/claude-1000/m3-batch20-code-r1/tree/`，只在它的拷贝上编原型、跑 harness 档；GPU 不跑，GPU 那一侧读代码、用 CPU 算期望。下文「副本」都指我草稿目录里的拷贝，副本上的数不算入库装置上的数。

## 复跑

```
# 空目录里从头跑（每步一条前台命令，单步约 4 分钟以内；all 实测约 30 分钟）
FROZEN_TREE=/tmp/claude-1000/m3-batch20-code-r1/tree SINGLEFS_REPO=/home/fy5090/code/singlefs \
  bash research/prompts/m3-batch20-code-r1-opus-model/rerun.sh <空目录> all
# 分步：prepare build-tree build-fix badimages shapes layer0 n9 fix-badimages fix-shapes fix-tests
#       sweep 0 1200 / 1200 2400 / 2400 3600 / 3600 4800，fix-sweep 同四段，最后 summary（与 expected-summary.txt 逐字比）
```

交回前在空目录 `/tmp/claude-1000/m3-batch20-code-r1-opus/rerun1` 里逐步跑通一次，末步原样输出：`SUMMARY_MATCHES lines=179`。开发时那一遍的摘要与这一遍逐行相同（只差开发那一遍没跑 n9）。

估时：先跑 `estimate 60`（全表每隔 60 段取 1 段，78 段 483 份镜像 8.8 秒，6 线程），按全表 4624 段推 523 秒；实测每批 1200 段 100–190 秒。没缩取样。

模型目录 `research/prompts/m3-batch20-code-r1-opus-model/` 每个文件的 sha256：

```
6acae9561b8e2cdafa13836b5d71f0db11db9a7e654ef72d0a9ea4544f2bfb27  expected-summary.txt
b9f093f1721905fd7923044b069f6efa05ae52a210b5630e573fe371aded66a4  fixX.patch
679d3f4b96e0396d7958d162eaf31d805b4def611f6960a2cf37ad6c3e3c1995  n9compare.py
a7f2967b55fefd08282b80ef724cbe1362298605511c1d8d032d9371673b6845  out/badimages.out
a7f2967b55fefd08282b80ef724cbe1362298605511c1d8d032d9371673b6845  out/fix-badimages.out
d96af9b1b54e151cc027d178700fb850a40e9165484d74d2ebf422993fd6db64  out/fix-shapes.out
06bd99c9d45929cac8c19d5a5238dd6eeb4f5309dd7b025afb281019c8c713f7  out/fix-sweep-0.out
8bbd1ec123c75ea2fe79de0bca441297a1f592e852c151819283f7d1add7c91f  out/fix-sweep-1200.out
42e8a1707d32dfe4ef3bb4100963184d8b5aa8b62d27d604c7f8c01f31dbc401  out/fix-sweep-2400.out
cfc702bc2c7ba97e46b186a312a9ce8f5d7762aad69a3a0a6d26b108d6448986  out/fix-sweep-3600.out
b73f2f00e710f72d48d1de5eebdb9d92cc02d700997662dcca970f48105b8514  out/fix-tests.out
e1b982c99b40be7aa1f4fe37f7243d54ca92da04858790cd33ef2c96a3ab0af2  out/layer0.out
4c658a3601399efef2ed6f548a2e994ae881436ae0edc233c9bb6aaa5f084cc2  out/n9.out
4f3fb8cb390be2518657f70d3dfd433dcef73f3f286d18c86c2574d27a813891  out/shapes.out
6acae9561b8e2cdafa13836b5d71f0db11db9a7e654ef72d0a9ea4544f2bfb27  out/summary.txt
c5541f2917156caea319dcfd15b5232b02d2237096fa69a1695cde8f47f454f9  out/sweep-0.out
4d4793c848007d041d4f9bba792ad0192cdf03e6d981c234bb586c8b380e816d  out/sweep-1200.out
44ab5bda7bbda7525396f7ec69db2d4f9120a47257560246886b04a256cc2577  out/sweep-2400.out
024e4a82574353fa207897bd7b331dbf23bec84a7fbf6fff506239987cc406f2  out/sweep-3600.out
14dbee718dafeedc54184fbd1c805aac40ffdb9c18f175a650929c8641313a7b  proto/Cargo.toml
632f7a1bcb542803fc7a518032b640649c6bca73eb4702eece90105cad352dc4  proto/src/badimages.rs
296b9872ebaa65122e1d149e46c1c3e8b8d39501876235f321ef72cd2fac39ab  proto/src/judge.rs
a66a7c6a62a9a2b76d15a4f06916691e1ad4966d39e4c8386dc032b2ae628279  proto/src/layer0.rs
a98679a05312d7f005ffbdfc9e32db554b7a0ed64442ebbb29316af984133359  proto/src/main.rs
82fd89f7202cb340f15fa0f967a5944714a62b2d985748ddd07d1a4ca49a4cc1  proto/src/n9.rs
b2655893a3e064e24db324228faaf6ae106ca2fec359df4f7e39682ccf21d085  proto/src/pool.rs
da380bf76451d935b3e74427d985201b78fdef9bb28a8d91fa1077997edbd3c4  proto/src/shapes.rs
4bbdf16f12a57b76cd6bfe51b89e64a8cb5d182567ea5d9942650037a9328dac  proto/src/sweep.rs
7c91868ac6de1a16e1114e7527aa8c80709ea4a1f916753a6d9789d512204eec  rerun.sh
6c4a5fa31244c01333341c3b3d4643e8b6147681899e9b712944cb9ed7ad808b  summarize.py
b1049e8bee070458c66940ed8f06c812b9f60e3703dbe10dcc71aa3950b610ee  workspace.patch
```

`out/badimages.out` 与 `out/fix-badimages.out` 同一个 sha256：改法 X 不改任何一份 N3 坏镜像上的判定。`proto/src/n9.rs` 的主体是从冻结副本 `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` 第一个 `use` 到 `fn main` 之前原样拷来的（不起 checker 档的 bin），`pool.rs` 照抄第 18 批攻方模型。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| N5 GPU 恢复 | **打中** | 第一个文件删完之后（最后一批那一版起，之后每一版），CPU 恢复两种政策都答没有文件；GPU 恢复内核 `judge_extent_node` 拒零条目的 extent 上段根，走读 `walk_failed(32)`。4484 段合法历史里 3408 段、7538 份镜像落在这一支，H1、H2、H3 三形都在里面 | 一 |
| N5 同号多条记录、多片 inode 叶、nlink 偏移 | 没打中 | 两边都「后读到的盖掉先读到的」、偏移都是 36、判 nlink 都在核单元数之前；300 个 inode（多片叶）的历史里除上一格外 CPU 与读代码推的 GPU 结局一致 | 一 |
| N1 合法中间态误报判不了 / 误红 | 没打中 | 用户决定的几步放开扫 4624 段（4484 段走得到）、32197 份镜像：0 误红、0 判不了、0 部分没判 | 二 |
| N1 两个方向都配上而违例漏判 | **打中两形** | S1：extent 树那一侧只有正方向配，备用条目改成种类 1 配上 inode 树、真 extent 树头 ID 改 0，I-4.7 报「没有处于删除中的 inode」而真 extent 树里 1 号还有单元；S4a：同一片叶里 1 号两条记录 [nlink 0, nlink 1]，I-9.5 报成立、整池一条都不红 | 二 |
| N3 坏镜像换造法 | 没打中 | 36 份换造法：33 份照预期；另 3 份多红的（B3 多 I-3.5，E3、G2 多 I-2.1 一类）是镜像字面上也违反那几条，删除线那几条的判定照预期 | 三 |
| N9 H3 形状与登记、不带参数的导出 | 没打中 | H3 两处与跑前登记第五节 5.3 逐条对上；冻结副本上的导出与入库 `e172-r2-…-h1h2`、`e172-r2-…-h3` 逐字节相同，与第一段那一份只差系统配置两槽 30 行（T_time，题面已排除） | 四 |
| 层 0 崩溃状态 | 没跑 | 四段代表历史全量 8.5×10⁷–1.18×10⁹ 个状态，每段都超过约 10⁶，照原型规则不跑；理由与替代论证见第六节 | 六 |

## 一、N5：GPU 恢复补了 nlink 之后，还有一形 CPU 答没有文件而 GPU 不答

### 打中的那一形：第一个文件删完之后 extent 上段根零条目

历史（每一步都是用户能做的，没有故障）：mkfs → 取号 → 暖机 → 新池新建文件 →（任意）顺序写 → unlink 1 号 → 各批直到队列空（进程里、会话里、挂载续做都一样）。最后一批之后那一版：

- core 写出的 extent 树是一个零条目的上段根。CPU 恢复把它当合法形态收下：「上段一条叶条目都没有：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），没有数据单元、没有下段。」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:4181`）；core 读 extent 树时也只许这一处零条目：「if header.entries.is_empty() && !(is_the_root_of_the_tree && expected_level == 0) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/extent_tree.rs:711`）。
- 之后 CPU 走 inode 树找 1 号：「if record.inode == FIRST_INODE_NUMBER {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5068`），记录已删，交回 `Ok(None)`，结局是「Ok(None) => RecoveryOutcome::NoFile { root: root_key },」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5228`）。
- GPU 恢复内核在树表循环里先读 extent 树（种类 1 是树表第一条）：「let read = read_extent_tree_of_the_first_file(pointer, tree_table.base, o + 14u, tree, root, fsid_low);」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3065`），上段每个节点过 `judge_extent_node`：「if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, first_inode, vec2<u32>(0u, 0u), last_inode, vec2<u32>(NONE, NONE))) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2364`），而 `judge_extent_node` 对零条目交回假：「if (n.entry_count == 0u) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2325`）。就算这一句放过，`read_extent_tree_of_the_first_file` 还要求上段恰好一条：「if (upper_count != 1u) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2684`）。读不成就「return walk_failed(32u);」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3067`），恢复结局照抄「out.outcome = walked.outcome;」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:45`）。

用 CPU 算出的期望（原型 `sweep`，每一份发布 / 用户改动 / 挂载之后的整池镜像跑 `singlefs_core::recovery::recover` 两种政策，读最新那条根的 extent 树根条目数；输出整行抄自 `out/summary.txt`）：

```
SWEEP histories=4624 ran=4484 not_reachable=140 images=32197 red_histories=0 gap_histories=0 gpu_divergent_histories=3408 gpu_divergent_images=7538 not_newest_images=0 history_lines_sha256_16=ef0c8c2bf65fdd60
SWEEP_CPU_OUTCOMES consulted_ignored={'FF': 20152, 'ZZ': 12045}
SWEEP_FIRST_GPU_DIVERGENT_KINDS {'cpu_consulted=Z cpu_ignored=Z extent_root=level0_entries0 inode_root=level1': 3408}
```

`gpu_divergent` 计的是「extent 树根零条目、CPU 恢复不是失败」的镜像；7538 份全是 CPU 两种政策都答没有文件（Z）、inode 树根层级 1。`not_newest_images=0`：两遍恢复走的都是最新那条根，判的就是那一版。H1、H2、H3 三形各一行（整行抄自 `out/sweep-1200.out`，截到 `first_gpu_divergent` 为止）：

```
SWEEP_HISTORY index=2016 name=dev4G-seq81585-ino0after-t[1]-b1-SameAsRecord-InProcess-post0 outcome=ran images=6 red=0 gap=0 gpu_divergent=1 not_newest_effective_root=0 watched={"I-3.4=H": 6, "I-4.4=N": 6, "I-4.6=H": 4, "I-4.6=N": 2, "I-4.7=H": 4, "I-4.7=N": 2, "I-9.5=H": 6, "I-9.6=H": 6} cpu_outcomes={"FF": 2, "ZZ": 4} first_gpu_divergent=[删除步1@txg8] cpu_consulted=Z cpu_ignored=Z extent_root=level0_entries0 inode_root=level1_entries0
SWEEP_HISTORY index=2025 name=dev4G-seq81585-ino0after-t[1]-b1-SameAsRecord-IntentUnmountRemount-post0 outcome=ran images=5 red=0 gap=0 gpu_divergent=1 not_newest_effective_root=0 watched={"I-3.4=H": 5, "I-4.4=N": 5, "I-4.6=H": 3, "I-4.6=N": 2, "I-4.7=H": 3, "I-4.7=N": 2, "I-9.5=H": 5, "I-9.6=H": 5} cpu_outcomes={"FF": 2, "ZZ": 3} first_gpu_divergent=[重开挂载续做之后] cpu_consulted=Z cpu_ignored=Z extent_root=level0_entries0 inode_root=level1_entries0
SWEEP_HISTORY index=2067 name=dev4G-seq81585-ino1first-t[1]-b1-SameAsRecord-InProcess-post0 outcome=ran images=7 red=0 gap=0 gpu_divergent=1 not_newest_effective_root=0 watched={"I-3.4=H": 7, "I-4.4=N": 7, "I-4.6=H": 4, "I-4.6=N": 3, "I-4.7=H": 4, "I-4.7=N": 3, "I-9.5=H": 7, "I-9.6=H": 7} cpu_outcomes={"FF": 3, "ZZ": 4} first_gpu_divergent=[删除步1@txg9] cpu_consulted=Z cpu_ignored=Z extent_root=level0_entries0 inode_root=level1_entries1
```

GPU 会给什么（读代码推的，GPU 没跑）：这些镜像上两遍恢复都是 `OUTCOME_FAILED`、失败码 32，CPU 是 `NoFile`；`classify_oracle` 对失败交回 `ORACLE_READ_FAILED`，CPU 那一侧对没有文件按流的版本表另分类。结局不同，对拍判不一致；这一态只要第一个文件删完就一直在（之后建 inode、再删别的号、回退以外的发布都不往 extent 树里加条目），所以删完之后的每一版都中。

对拍为什么没抓到：删除线对拍的流只走到第一批，「publish_the_next_step_in_batches_of_one(&mut pool);」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:4930`）在那个函数里只调一次，流的说明也写「恢复从意图那一版起答「没有文件」，所以带文件的版本只有 txg 3、4。」——三个单元、批大小 1，第一批之后还剩两个单元，extent 根从没到过零条目。G 的四行变异都钉 nlink 那一判，碰不到这一支。

### 四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 这一轮没有对立臂；它打的是 G 那一条「GPU 恢复与 CPU 冷走读一样答没有文件」的覆盖面：nlink 那一判之外，「记录已删」那一形 GPU 还没对齐 |
| 被判的系统看不看得到判别它的东西 | 看得到：GPU 内核读 extent 根就拿到 `entry_count`，与 CPU 读的是同一个字段 |
| 满足判据字面的哪一个分句 | 正文第五节「GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。」（`research/prompts/_m3-batch20-code-r1-body.md:67`） |
| 跑前条款给的改法在打中的格上还中不中 | 跑前条款只写「改法进下一批」，没给具体改法；G 已合的 nlink 判在走读里排在 extent 读之后，对这一格不起作用 |

推翻条件：在入库装置上（GPU 对拍）录一条走到最后一批的删除流，最后一批那一版及之后的状态上 GPU 恢复结局与 CPU 相同——那就是我对 `judge_extent_node` 或 `read_extent_tree_of_the_first_file` 的读法错了。

### 同号多条记录、多片 inode 叶、nlink 偏移：没打中

- 同号多条记录：CPU 在全部叶里逐条「if record.inode == FIRST_INODE_NUMBER {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5068`）覆盖上一条；GPU「if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3264`）同样覆盖，都是后读到的那条赢。原型 S4a、S4b（第二节）上 CPU 恢复分别是 F、Z，按同一个「后者赢」推 GPU 给同样的结局。
- nlink 偏移：GPU「const INODE_RECORD_NLINK_OFFSET: u32 = 36u;」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:135`），CPU 解记录在 nlink 之前跳「reader.skip(8 + 4 + 4 + 4); // locality_id、mode、uid、gid」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/records.rs:100`），前面另读 inode 8、出生代 8：两边都是 36。判的位置：CPU「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5077`）在核单元数之前，GPU「if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3277`）也在核单元数之前。
- 多片 inode 叶：sweep 里建 300 个空 inode 的历史（inode 根多条条目），删 1 号、301 号、先 150 再 1 号，除上面那一形外 CPU 恢复结局与按代码推的 GPU 结局相同；两边都逐条遍历 inode 根的全部条目。

## 二、N1：配对改了之后

### 2.1 合法删除中间态上误报判不了 / 误红：没打中

原型 `sweep`（`proto/src/sweep.rs` 的 `histories`）放开扫的用户动作，只固定前缀（mkfs → 取号 → 暖机 → 新池新建文件 3000 字节）、没有故障：

| 轴 | 取值 |
|---|---|
| 文件形状（顺序写字节数） | 不写、0、1 个净荷、2.5 个净荷、3 个净荷、5 个净荷 + 1、145 个净荷（下段两片叶） |
| 空 inode 与 unlink 对象、次序 | 0 个：[1]；1 个建在顺序写之前（H3 那一序）：[1]、[2]、[2,1]、[1,2]；1 个建在之后：[1]、[2]、[1,2]；2 个：[1]、[2]、[2,1]、[1,3]、[3,2,1]；300 个（多片叶）：[1]、[301]、[150,1] |
| 批大小 | 1、2、144（145 净荷那一形取 144、100：删到剩一片叶的下段两层根，只有删除写得出） |
| 续做 / 回退 | 进程里；会话里；意图后不卸载重挂续做；意图后正常卸载再挂续做；进程里删完后管理员回退到删除那一串第 0–3 版再挂续做；进程里删到第 0–2 批停、重挂续做 |
| 删完再建再删（post） | 0；进程里与会话里两形另加 2（再建 2 个、删号最大的、再建 1 个） |
| 会话容器容量 | 会话里那一形另加墓碑容器 1 条、容器树节点 2 条（多片容器、多层墓碑树） |
| 调用方时间、盘大小 | 记录现有时间；进程里、2.5 净荷那一形另加早一天、晚一天、8 GiB 盘 |

每一次发布、用户改动、挂载之后的整池镜像跑池级 checker。判定种类：H 成立、N 不适用且理由是「一次都没判到」那四句之一、G 不适用而理由是某一版判不了、P 部分没判、V 违例（`proto/src/judge.rs` 的 `kind`）。结果整行抄自 `out/summary.txt`：

```
SWEEP_WATCHED {'I-3.4=H': 32197, 'I-4.4=N': 32197, 'I-4.6=H': 18707, 'I-4.6=N': 13490, 'I-4.7=H': 18707, 'I-4.7=N': 13490, 'I-9.5=H': 32197, 'I-9.6=H': 32197}
SWEEP_NOT_REACHABLE count=140 reason=回退目标越界
```

32197 份镜像上 G、P、V 一个都没有；走不到的 140 段全是「回退到第 k 版」而那段历史的删除版本不足 k 个。新加的反方向那一遍只读树表条目，树表是 COW 单元、合法发布写出的树表里 inode 树恰好被一条 `deleted_inodes` 指着，扫到的全部树表都过了这一遍。

### 2.2 两个方向都配上、违例照旧漏判：打中两形

**S1：extent 树那一侧没有反方向。** 新加的反方向只配 `deleted_inodes` 与 inode 树：「if deleted_inodes_entries_naming_the_inode_tree != 1 {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8458`）。extent 树仍只从 inode 树一侧去找：「kind_of(entry) == TREE_KIND_EXTENT && head_identifier_of(entry) == inode_tree」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8405`），恰好一条就只读那一条；头 ID 不指着 inode 树的 extent 树一次都不读、也不记判不了。

镜像（`proto/src/shapes.rs` 的 `run`，内存稀疏盘）：新池新建文件（1 号一个单元）→ 建 1 个空 inode（2 号）→ 已登记的 I-4.7 坏镜像改法 `drop_the_record_of_the_first_file_and_keep_its_data_units`（1 号记录拿掉、单元留着、号不在队列；对照行 `S1-base` 只红 I-4.7）→ 最新那一版树表里：真 extent 树那一条头 ID 改 0；一条空根的备用条目（种类 6、树 65541）种类改 1、头 ID 改成 inode 树 65537。重封、补引用链。判定（整行抄自 `out/shapes.out`）：

```
SHAPE case=「S1 extent 树反方向不配：真 extent 树头 ID→0，一条空根备用条目改成种类 1、头 ID→inode 树」 literal=I-4.7（真 extent 树里 1 号还有 1 个单元、inode 树里没有 1 号记录、队列空） checker_I-4.7=N detail=走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号） violated=["I-9.16", "I-9.27"] watched=I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N
```

I-4.7 的对象「extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号」（`.claude/kb/invariants.md:183`）：真 extent 树里 1 号还有一个单元、inode 树里没有 1 号的记录、号不在队列，镜像上有对象且违例；checker 报不适用、理由说没有对象。红的只有 I-9.16（种类 1 两条）与 I-9.27（头 ID 取值），与第 18 批 L1a 只红 I-9.16 同一族。

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨第 18 批判决给的改法 A 与「两侧都反方向配」：改法 A 只补 `deleted_inodes` 那一侧，这一格照中 |
| 系统看不看得到 | 看得到：扫描手里就是整张树表条目，真 extent 树那一条（种类 1、头 ID 0）在里面 |
| 判据分句 | 正文第五节「漏判：……或报不适用而镜像上有它的对象」（`research/prompts/_m3-batch20-code-r1-body.md:66`） |
| 改法还中不中 | 改法 A（已合）在这一格上照中；下面改法 X1 在副本上量过不中 |

**S4a：同一片叶里同号两条记录。** 扫描收记录时同号后者覆盖前者，注释把唯一性交给主走读：「同一个号出现两条时后读到的那条盖掉先读到的（号的唯一性由主走读的 I-9.4、I-1.1 判）。」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8086`）。可 I-9.4 的字面只管容器号与跨容器的序：「打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号；沿叶序容器号严格递增，且左容器的最大 key < 右容器号」（`.claude/kb/invariants.md:291`），同一片叶里两条同号它不管。

镜像：新池新建文件（1 号一个单元）→ 建 1 个空 inode（2 号），最左那片叶 [1, 2] 改成 [1 号 nlink 0, 1 号 nlink 1]（第二条是 1 号原记录的拷贝，2 号那条被覆盖），队列空，重封、补引用链：

```
SHAPE case=「S4a 叶里 [1 nlink 0, 1 nlink 1]、队列空」 literal=I-9.5②（有一条 nlink 0 的 1 号记录、号不在队列） checker_I-9.5=H violated=[] cpu_recovery_consulted=F ignored=F
SHAPE case=「S4b 叶里 [1 nlink 1, 1 nlink 0]、队列空」 literal=I-9.5②（有一条 nlink 0 的 1 号记录、号不在队列） checker_I-9.5=V violated=["I-9.5", "I-4.7"] cpu_recovery_consulted=Z ignored=Z
```

I-9.5 第二个方向「② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里」（`.claude/kb/invariants.md:292`）：S4a 有一条 nlink 0 的 1 号记录、号不在队列，checker 报成立，整池一条违例都没有（`violated=[]`）。只把两条换个次序（S4b）就红：判定取决于叶里的次序，不取决于镜像有没有违例。

| 问 | 答 |
|---|---|
| 分不分辨臂 | 不分辨第 18 批的配对改法：它不是树表形态，两个方向都配得上；病根在收记录用 map、同号后者赢，四条删除态判法共用这一份 |
| 系统看不看得到 | 看得到：两条记录都从同一片叶里读出来，`insert` 时就知道号重了 |
| 判据分句 | 正文第五节「漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立」（`research/prompts/_m3-batch20-code-r1-body.md:66`） |
| 改法还中不中 | 改法 A 照中；改法 X2 在副本上量过 S4a 改报判不了，但 S4b 从红变成判不了（见第五节） |

题面那一句问的是「树表形态」；S4a 是记录层的形态，我把它放在这里是因为它同样是「两个方向都配上而违例漏判」，归不归这一格由主 agent 定。它不是第 20 批新引进的：注释与收法是第 18 批的。

### 2.3 试过、被抓住的树表形态

| 形 | 结果（整行在 `out/shapes.out`） |
|---|---|
| S2：真 `deleted_inodes` 头 ID 改 0，备用条目改种类 9、头 ID 指 inode 树（空根 = 空队列） | 红 I-9.16、I-9.27、I-9.5、I-4.7：假队列是空的，1 号 nlink 0 不在里面，被判红 |
| S3：备用条目改成第二棵 inode 树（种类 2、空根） | 红 I-9.16，三条判不了，理由是新加的反方向那一句 |
| 第二条 `deleted_inodes` 头 ID 指别处、两条同指 inode 树 | 正方向配不上记判不了、反方向数到 2 记判不了（读代码，没另造镜像） |

## 三、N3：每份新坏镜像换一种造法再造：没打中

做法（`proto/src/badimages.rs`、`proto/src/shapes.rs` 的 `more_bad_images`）：池在内存稀疏盘上造（不用 `build_pool` 的文件镜像），改法用 `common_deletion_in_progress` 里对 `MemoryPool` 动手的那几个函数，或原型里另写的（两份各翻一个字节、只清一份、改层级字节后重封）。「原造法」那几行重现用例，其余换历史（一个单元 / 三个单元 / 第一批之后 / H3 那一形 / 从没删过的池）、换版本（最新 / 中间版）、换改法。原型判「只红目标」用的是用例同一种比法：判红集合恰好等于目标；判不了的看理由串。下表由 `out/badimages.out` 逐行生成（`预期` 一列是原型的 `as_expected`）：

| 造法 | 预期 | 判红集合 | 六条判定 |
|---|---|---|---|
| A0 原造法：三单元意图之后 1 号 nlink 0→1 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N |
| A1 第一批之后（剩两个单元）1 号 nlink 0→1 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| A5 第一批之后 1 号 nlink 0→7 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| A2 一个单元（3000 字节）意图之后 1 号 nlink 0→1 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N |
| A3 H3 那一形（先建 size 0 的 2 号）删 2 号意图之后 2 号 nlink 0→1 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N |
| A4 H3 那一形删 1 号意图之后 1 号 nlink 0→1 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N |
| B0 原造法 L1a：意图之后最新树表 deleted_inodes 种类 9→13 | true | ["I-9.16"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| B4 意图之后最新树表 deleted_inodes 种类 9→0 | true | ["I-9.16"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| B1 第一批之后最新树表 deleted_inodes 种类 9→13 | true | ["I-9.16"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| B3 只在意图那一版（中间版）整条拿掉 deleted_inodes 条目 | false | ["I-9.16", "I-3.5"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| B2 从没删过的池（建了两个空 inode）最新树表 deleted_inodes 种类 9→13 | true | ["I-9.16"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| C0 原造法：头 ID→0 | true | ["I-9.27"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| C1 头 ID→extent 树的树 ID | true | ["I-9.27"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| C2 头 ID→deleted_inodes 自己的树 ID | true | ["I-9.27"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| C3 只在意图那一版（中间版）deleted_inodes 头 ID→0 | true | ["I-9.27"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| D0 原造法：extent 头 ID→0 | true | ["I-9.27"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| D1 extent 头 ID→deleted_inodes 的树 ID | true | ["I-9.27"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| D2 只在意图那一版（中间版）extent 头 ID→0 | true | ["I-9.27"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| E0 原造法：extent 根两份清零（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| E1 extent 下段根两份清零（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| E2 extent 根两份各翻一个载荷字节（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| E3 extent 根只清盘 0 那一份（盘 1 读得出） | false | ["I-2.1", "I-4.8", "I-7.4"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| F0 原造法：deleted_inodes 根两份清零（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| F1 deleted_inodes 根两份各翻一个载荷字节（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| F2 deleted_inodes 根层级字节 0→1、重封补链（判红集合照实打） | true | [] | I-9.6=H,I-9.5=G,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N |
| G1 中间版树表两份各翻一个载荷字节（判红集合照实打） | true | ["I-2.1", "I-3.1", "I-4.8", "I-7.4"] | I-9.6=H,I-9.5=G,I-3.4=G,I-4.7=G,I-4.6=G,I-4.4=N |
| G2 中间版树表只清盘 0 那一份 | false | ["I-2.1", "I-4.8", "I-7.4"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| H1 最新那一版树表条目收窄到 8（判红集合照实打） | false | ["I-4.8", "I-7.2", "I-7.4"] | I-9.6=N,I-9.5=G,I-3.4=G,I-4.7=G,I-4.6=G,I-4.4=N |
| I1 墓碑树根两份清零（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=G,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| I2 墓碑容器两份各翻一个载荷字节（判红集合照实打） | true | ["I-2.1", "I-4.8", "I-7.2", "I-7.4"] | I-9.6=G,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| J0 原造法：墓碑容器实例 1→2、对象 ID 抬到水位 | true | ["I-1.2", "I-1.11", "I-4.2", "I-7.7"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| J1 墓碑容器实例 1→6、对象 ID 抬到水位 | true | ["I-1.2", "I-1.11", "I-4.2", "I-7.7"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| J2 对照：实例不动、只抬对象 ID | true | ["I-9.6"] | I-9.6=V,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| L1 三次建 inode，只在第一次那一版把 2 号 nlink 改 0 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| L2 三次建 inode，只在第二次那一版把 3 号 nlink 改 0 | true | ["I-9.5"] | I-9.6=H,I-9.5=V,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N |
| M1 三单元文件（有下段）建两次 inode，只在中间版拿掉 1 号记录 | true | ["I-4.7"] | I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=V,I-4.6=N,I-4.4=N |

三份 `false` 逐份看了违例说明：

- B3（中间版整条拿掉 `deleted_inodes` 条目）多红 I-3.5，说明原文「相邻两版 (实例 {1}、txg 5) → (实例 1、txg 6)（NoHole）：后一版引用、前一版不引用的单元盘 0 槽 50277（用户可见树 Some(UserVisibleTreeOfTheWalk { kind: 9, tree: 65544 })）诞生代号 5 不大于前一版的 txg」（`out/badimages.out` 的 `BAD_VIOLATION` 行）：意图那一版自己写出的 `deleted_inodes` 根，被改坏之后它的诞生版不再引用它、下一版引用它，镜像字面上违反 I-3.5 的出现规则。用例只在最新那一版拿掉（L1b），不碰这一条；不算打中。
- E3、G2（两份里只清盘 0 那一份）红 I-2.1、I-4.8、I-7.4：两份副本不一致本身就是那几条管的；删除线四条照判、全成立，与我的预期不同的是我把「读得出」当成「不红」。不算打中。
- H1 只记判定：最新那一版树表收窄时主走读读不了树表，四条报树表条目窄那一句判不了，I-9.6 报不适用；没断言判红集合。

另记一处观察（不是判据字面的打中）：F2 把 `deleted_inodes` 根的层级字节 0→1、重封、补引用链，整池判红集合是空的，三条删除态只报「deleted_inodes 根读不出，或不是层级 0……」那一句判不了。扫描里那段注释说形状不对交给主走读：「读不出、解不开、形状不对由主走读判」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:7935`），可层级不是 0 的 `deleted_inodes` 根主走读一条都不判；`invariants.md` 里也没有一行管它的层级（I-9.24 管 key 条数）。队列内容没变，I-9.5 的字面没被违反，所以不按漏判报，交主 agent 看要不要记欠账。

## 四、N9：H3 的形状与登记、不带参数的导出：没打中

- 登记两处：「顺序写之前在同一个实例里用 `publish_new_inodes` 建一个 size 0 的 inode（号 2）」与「意图那次的调用方时间 = 顺序写的时间 + 60 秒」（`research/prompts/e172-preregistration.md:404`）。实现：顺序写之前调「publish_new_inodes(」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:634`），实例是新池新建文件那一个（同一调用的第 640 行）；意图时间在记录现有时间上加「+ SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:674`）。原型在冻结副本上导出 H3，`impl_config` 里那几格（整行抄自 `out/n9.out`）：

```
N9 h3_impl_config history=h3 file_bytes=81585 batch_data_units=1 write_time_seconds=1788000000 sequential_write_time_seconds=1788000060 intent_write_time_seconds=1788000120
```

  1788000120 = 顺序写 1788000060 + 60，与登记一致。登记没给 2 号建出来的时间，实现取新池新建文件那一个（「pub const WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE: u64 =」，`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:237`），三个时间各不相同，不与登记冲突。
- 导出逐字节：原型把 `deletion_write_dump.rs` 从第一个 `use` 到 `fn main` 之前原样拷进来跑（不起 checker 档的 bin），冻结副本上的输出（整行抄自 `out/n9.out`）：

```
N9 dump=h1h2 sha256=b7e87c4ed73978b2f6d89a00449bb46cb70a4d36a5e7a5a614ea64464bb090c0 equals_r2_h1h2=True equals_first_segment_dump=False
N9 dump=h3 sha256=9571c90498286a6ddfd4ac8718f3aedb8380132fa22496303342d58c0c09c45e equals_r2_h3=True
N9 h1h2_vs_first_segment lines=462/462 differing_lines=30 groups=[(('device_region_bytes', '0', '4096'), 14), (('device_region_bytes', '4096', '4096'), 16)]
```

  不带参数的导出与入库 `research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out` 逐字节相同，`--history h3` 与 `e172-r2-deletion-write-dump-h3-2026-10-05.out` 逐字节相同；与第一段那一份（`e172-deletion-write-dump-2026-10-05.out`，sha256 5a9b3fa2…）不同的 30 行全是偏移 0 与 4096、长 4096 的写，即两个系统配置槽，正文已说这一处不算打中。

## 五、改法（都是我提的，只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 怎么改 | 量过 / 推的 |
|---|---|---|---|
| G1（N5） | 第一节那一形 | `judge_extent_node` 对「上段根、层级 0」放过零条目（照 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/extent_tree.rs:711` 那一句的条件），`read_extent_tree_of_the_first_file` 上段 0 条时交回成功、数据单元 0 个（照 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:4181` 那一支）；对拍加一条走到最后一批（第一个文件删完）的流，另加删完之后再建 inode 的一段 | **推的**：GPU 不跑，改法没写、没跑 |
| X1（N1 S1） | S1 | 删除态扫描里每一棵 extent 树的头 ID 要配上恰好一条 inode 树，配不上对 I-4.7、I-4.6 记判不了（新理由「走读到的某一版树表里有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树」） | **量过**（副本）：`fixX.patch`；S1 上 I-4.7 由 N 变 G（`FIX SHAPE` 那一行）；4624 段合法历史 0 误红、0 判不了（`FIX_SWEEP` 那几行与 `SWEEP` 逐字段相同，历史行 sha256 前 16 位都是 ef0c8c2bf65fdd60）；两个 harness 测试目标 89 + 28 条全过 |
| X2（N1 S4） | S4a | 收记录时同号第二次出现就交回判不了（新理由「同一个号有不止一条记录」） | **量过**（副本），但有代价：S4a 由 H 变 G，S4b 由 V 变 G——原来红的一形变成判不了。更好的一条（**推的**）：给 I-9.4 或另一行加「同一片 inode 叶容器里记录号严格递增」，让主走读把同号两条判红，扫描再记判不了 |

副本上量出的原样输出（整行抄自 `out/summary.txt`）：

```
FIX SHAPE case=「S1 extent 树反方向不配：真 extent 树头 ID→0，一条空根备用条目改成种类 1、头 ID→inode 树」 literal=I-4.7（真 extent 树里 1 号还有 1 个单元、inode 树里没有 1 号记录、队列空） checker_I-4.7=G detail=走读到的某一版树表里有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树：那一版的数据单元数判不了 violated=["I-9.16", "I-9.27"] watched=I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=G,I-4.6=G,I-4.4=N
FIX SHAPE case=「S4a 叶里 [1 nlink 0, 1 nlink 1]、队列空」 literal=I-9.5②（有一条 nlink 0 的 1 号记录、号不在队列） checker_I-9.5=G violated=[] cpu_recovery_consulted=F ignored=F
FIX SHAPE case=「S4b 叶里 [1 nlink 1, 1 nlink 0]、队列空」 literal=I-9.5②（有一条 nlink 0 的 1 号记录、号不在队列） checker_I-9.5=G violated=[] cpu_recovery_consulted=Z ignored=Z
FIX_SWEEP histories=4624 ran=4484 not_reachable=140 images=32197 red_histories=0 gap_histories=0 gpu_divergent_histories=3408 gpu_divergent_images=7538 not_newest_images=0 history_lines_sha256_16=ef0c8c2bf65fdd60
FIX test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
FIX test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished
```

X1、X2 都不碰 GPU 扫描内核；进库要连 GPU 判器删除态扫描一起改，那一半是推的、没写。

## 六、层 0 崩溃状态：没跑

原型 `layer0` 只数不枚举（`closed_form_state_count`，录制流从 mkfs 之后起；「从意图起」只数意图那次发布起的段）。整行抄自 `out/layer0.out`：

```
L0 case=「一个单元的文件：意图 + 最后一批」 segments=19 closed_form_state_count_whole_stream=84934692 segments_from_the_intent=8 closed_form_state_count_from_the_intent=68157453 longest_segments_from_the_intent=[26, 20, 2, 2, 2, 2] runs=no（超过约 10⁶，原型规则不跑）
L0 case=「size 0 的 2 号：意图 + 最后一批」 segments=23 closed_form_state_count_whole_stream=85983274 segments_from_the_intent=8 closed_form_state_count_from_the_intent=68157453 longest_segments_from_the_intent=[26, 20, 2, 2, 2, 2] runs=no（超过约 10⁶，原型规则不跑）
L0 case=「H1 形：三个单元、批大小 1 删到空」 segments=31 closed_form_state_count_whole_stream=1179648114 segments_from_the_intent=16 closed_form_state_count_from_the_intent=89128985 longest_segments_from_the_intent=[26, 24, 22, 20, 2, 2] runs=no（超过约 10⁶，原型规则不跑）
L0 case=「H3 形：先建 size 0 的 2 号、三个单元、批大小 1 删到空」 segments=35 closed_form_state_count_whole_stream=1183842424 segments_from_the_intent=16 closed_form_state_count_from_the_intent=92274713 longest_segments_from_the_intent=[26, 24, 22, 22, 2, 2] runs=no（超过约 10⁶，原型规则不跑）
```

最小的一段（一个单元的文件意图 + 最后一批）全量 84934692 个状态，每段都超过约 10⁶，照原型规则一段都不跑；这一条腿原型跑的层 0 状态数合计 0。删除类发布的单元写段有 20–26 次写（每次发布十来个单元、两盘各一份），一段就是 2²⁰–2²⁶ 个子集，取样缩不下来。

崩溃状态上为什么不会出新的反方向误报（**推的**，没在崩溃状态上量）：反方向那一遍只读一张树表单元里的条目。树表是 COW 单元，层 0 里单元写只取「落 / 没落」两态；没落或读不出的树表走的是原有的 `TreeTableUnreadable` 那一支，读得出的树表就是某一次发布写出的那一张。候选集里的每一版（根环里的根、由记录施加出来的那几版）指的树表都是 core 某次发布整张写出的，内容与第二节扫过的同一次发布相同。推翻条件：层 0 某个状态上 I-9.5、I-4.7、I-4.6 报「头 ID 指着它的 deleted_inodes 条目不是恰好一条」那一句判不了。

第一节那一形在崩溃状态上同样中（**推的**）：根槽已持久、最后一批那一版成了所选根的每一个状态，CPU 走的就是那一版，答没有文件，GPU 照第一节的代码答走读失败。

## 没打中的形状

- N1 误报：4624 段历史、32197 份镜像，轴见 2.1 的表；0 误红、0 判不了、0 部分没判。
- N1 树表形态：S2（假 `deleted_inodes` 配上 inode 树、真的头 ID 改 0）被 I-9.5 判红；S3（第二棵 inode 树）被新加的反方向记判不了；两条 `deleted_inodes` 都指或一条指别处：正方向或反方向记判不了（读代码）。
- N3：36 份换造法，33 份照预期；B3、E3、G2 多红的那几条镜像字面上也违反，F2 只是观察（第三节）。
- N5：同号多条记录（两边都后者赢）、多片 inode 叶（300 个 inode）、nlink 偏移（都是 36）、nlink 判的位置（都在核单元数之前）、会话里小容器那一形（墓碑容器 1 条、容器树节点 2 条；删几个号才长出多片容器、多层墓碑树，我没逐段核）、管理员回退到删除前后各版：CPU 恢复两种政策 32197 份镜像全是 FF 或 ZZ，没有一份失败，除第一节那一形外按代码推 GPU 结局相同。
- N9：H3 两处与登记逐条对上；导出与入库 r2 两份逐字节相同。

## 这条腿自己的限度

- GPU 一行没跑。第一节「GPU 会给什么」是按 `crash_judge_common.wgsl` 读出来的；GPU 判器的走读内核（`crash_judge_checker.wgsl` 的 `w_walk_extent_tree`）在零条目 extent 根上判不判 I-1.1、I-1.10 我没读完，CPU 池级 checker 在这些镜像上不红，GPU 池级那一半可能另有一处不一致（推测，没判）。
- 层 0 一个状态都没跑（第六节），崩溃状态上的两句都是推的。
- S1、S4a 是手改的坏镜像，不是从入口走得到的状态；它们打的是「漏判」那一句，不是「误红」那一句。
- 扫描里调用方时间、8 GiB 盘只在进程里、2.5 净荷那一形上加；145 净荷那一形只取批 144、100；会话小容器只在会话里那一形上加；post 只在进程里与会话里两形上加。
- N9 跑的是把 bin 正文拷进原型的那一份，不是 bin 本身；两者同源（冻结副本里那份 bin 的 sha256 在快照清单里）。
- 改法 X1、X2 只改了 CPU 池级 checker，副本上量；GPU 扫描内核没改、没跑；checker 档测试没跑。
- 这是一次抽样：各格的「没打中」照 `.claude/rules/three-way-inference.md` 要再抽一次才记。

## 没做什么

- 不判正推那几格（K1、K3），不替主 agent 采纳。
- 副本上量出的数（改法 X1、X2 那几行）不算入库装置上的数。
- 没跑重型测试、checker 档测试、GPU、层 0 枚举、变异表。
- 草稿目录 `/tmp/claude-1000/m3-batch20-code-r1-opus/` 里的仓副本、编译目录、复跑工作目录（`tree/`、`fix/`、`target/`、`target-fix/`、`rerun1/`）交回前已删；留下的只有几份草稿脚本与日志（`b.sh`、`r.sh`、`summarize.py`、`build*.log`、`fixX.patch`、`sum-*.txt`、`out/` 下不含导出的小文件），不留仓副本。
