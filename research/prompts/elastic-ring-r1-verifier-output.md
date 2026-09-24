# elastic-ring-r1 核查员报告

**这是观测，不是判决**：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

时刻一律 UTC（本机时钟）；这一轮跑在 2026-09-22。

## 判别力自证

按定义，先从待核引用里挑一条，在草稿目录的副本里把它的行号加 1，核它，必须判 ✗。

取的引用：`elastic-ring-r1-opus-output.md` 第 70-74 行引 `crates/singlefs-core/src/recovery.rs:850`
逐字「`let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;`」。

```
$ cp crates/singlefs-core/src/recovery.rs /tmp/claude-1000/elastic-ring-r1-verifier/selftest/recovery.rs
$ sed -n '851p' /tmp/claude-1000/elastic-ring-r1-verifier/selftest/recovery.rs
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
```

把行号故意加 1（850→851）之后，副本第 851 行内容是
`let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");`，
与待核原文（`let ring_bytes = ...`）不符 ⇒ **判 ✗**。核查方法分辨得出。

## 输入核对

- 报告 sha256：`elastic-ring-r1-opus-output.md`（441 行）实测
  `7b08c422ddbae07ad72ad7a1ff592570ecdf8e7d0f70f0e86e7b3a2a2dad92be`，
  与任务给的一致；`elastic-ring-r1-sonnet-output.md`（166 行）实测
  `ac9b66d8ed32457a8521eea249f57ae0337d82ec2b1ca87bea8a57cea7a70da4`，一致。
- 快照：这一轮是设计轮，任务未给 `crates/` 与 `.claude/kb/` 快照路径 ⇒ 按规则对主树核，
  行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」，不记 ✗。
- **本机主树在核查过程中被别的会话持续修改**：`git status` 起始即显示
  `crates/singlefs-core/src/mount.rs`、`recovery.rs`、`.claude/kb/decisions/16-发布语义.md`、
  `23-journal的角色与格式.md`、`28-挂载期承诺量.md`（未在改动名单，经查无改动）、
  `invariants.md`、`checks-owed.md`、`crates/singlefs-checker/src/image.rs`、`walk.rs` 等均为
  `M`（有未提交改动）。这一情况在下面具体条目里逐条标注。
- 草稿目录：`/tmp/claude-1000/elastic-ring-r1-verifier/`（仓副本 `repo/`、自证副本 `selftest/`、
  两份 python 模型复跑产物、cargo 探针复跑日志）。主工作区一个字未写。

## 一、云端攻方（Opus）腿：K1 / K5

### 五处主 agent 点名重核，逐处结果

**1. K5-3（副本上重做）**：`rsync -a --exclude target --exclude .git` 到
`/tmp/claude-1000/elastic-ring-r1-verifier/repo/`，把 `geometry_probe.rs`、`field_offset_probe.rs`
拷进 `crates/singlefs-harness/tests/`，`nice -n 19 cargo test -p singlefs-harness --test
zz_elastic_ring_probe --test zz_ring_field_offset2 -- --nocapture` 原样重跑（0 警告加载条件：
`ps` 查到另一会话在跑 `gate.sh --staged` 与一个 `cargo test`，无 qemu/vm-bench/fio，照常跑）。
产物与腿存档的 `geometry_probe-output.txt` 逐段内容一致（`diff` 排除编译期噪音行后退出码 0）。
**「第一个事务的单元真的写进环里了」这句是实测断言，独立核实为真**：
1 GiB 环（槽 `[1024, 66560)`）上跑完 `make_filesystem` + `acquire_instance` + `warm_up` +
`publish_first_file` 之后，`output.root` 的三个位置条目是
`instance_table: SlotNumber(50176)`、`mapping_root: SlotNumber(50247)`、
`tree_table: SlotNumber(50248)`——三个槽号全部 < 66560，即全部落在声明的环范围内；
`checker：33 条判决，成立 20，违例 []`。12 KiB 环（E 格）与 1 GiB 环（B 格）的 mkfs 均被接受，
`checker` 违例均为空。**判定：✓**，且是实测而非算术。

**2. K1-1（91.67% 复跑）**：`shrink_replay_model.py` 独立复跑，sha256
`798119cae6dee830ab66069864f1cc52508e0d26b25bb9be7c566f4bb721cea3`，与腿存档产物逐字节一致
（`diff` 退出码 0）。768→64 MiB、深度 16 那一行：施加 0 条的取样点 469/512，
解析残差比例 `(196608−16384)/196608 = 0.9167`。**判定：✓**。

**3. K1-3（现查 recovery.rs）**：`crates/singlefs-core/src/recovery.rs:273-283` 现查
（主树今天在改动名单里，但该段行号与内容对得上，逐字核对一致）：

```
273:         match &chosen {
274:             None => chosen = Some(best_on_device),
275:             Some(previous) => {
276:                 if previous.immutable.filesystem_identifier
277:                     != best_on_device.immutable.filesystem_identifier
278:                     || previous.immutable.device_count != best_on_device.immutable.device_count
279:                 {
280:                     return Err(RecoveryFailure::SystemConfigurationsDisagree);
281:                 }
282:             }
283:         }
```

只比 `filesystem_identifier` 与 `device_count`，**环长（`journal_ring_bytes`）或任何几何量一个字都不比**。
**判定：✓**。

**4. K5-1（十个容量点复跑）**：`k5_capacity_scan.py` 独立复跑，sha256
`5c9929985d9e0cf5bd0a969d3d0e289be21587e35c5e6c6a861b099badfc63a8`，与腿存档产物逐字节一致
（`diff` 退出码 0）。十个容量点（512 MiB / 1 / 2 / 3 / 4 / 16 / 64 / 200 / 500 / 2048 GiB）
每一点「乙买到甲传不出的东西吗」列均为「没有：同一个环长甲传得出」。**判定：✓**。

**5. 大端/小端自报出错有没有流入正文**：全文 grep `大端|小端|endian|字节序` 在
`elastic-ring-r1-opus-output.md`、`elastic-ring-r1-sonnet-output.md`、两份本地攻方样本、
`elastic-ring-r1-local-attack*.md` 均 **0 命中**——假结论未流入任何一份报告正文。
model 目录里 `geometry_probe.rs:119-121` 留痕：注释「checker 的读法：小端」+
`u64::from_le_bytes`，与现查代码一致（`crates/singlefs-checker/src/lib.rs:153`
`u64::from_le_bytes(...)`，写方 `crates/singlefs-core/src/bytes.rs:36`
`self.put(&value.to_le_bytes());`）——修正后的结论（写方小端、checker 333/417 正确）是真的，
且只留在 model 源码注释里，没有把假结论（大端读法、偏移表错）写回报告正文。**判定：✓ 未泄漏**。

### 其余引用逐条核（现查主树，均与本次核查时刻的主树内容逐字一致）

| 引用 | 核的结果 |
|---|---|
| `journal.rs:107-114` | ✓ 逐字一致 |
| `recovery.rs:850` | ✓ 逐字一致（见判别力自证） |
| `recovery.rs:841` | ✓ 逐字一致 |
| `mount.rs:1122-1131` | ✓ 逐字一致（`mount.rs` 是这一轮改动名单内的文件，但该段内容与行号对得上） |
| `allocator.rs:212/236/241/401` | ✓ 四处逐字一致 |
| `crates/singlefs-checker/src/image.rs:137` | ✓ 与当前主树一致；HEAD 版本该行是另一句（`image.rs` 在改动名单内，行序被本地改动重排），但核查按规则对主树核，主树对得上 |
| `crates/singlefs-checker/src/walk.rs:2141` | ✓ 逐字一致（HEAD 与主树该行相同，尽管全文有大改动） |
| `make_filesystem.rs:36-37`、`:140`、`:141-142`、`:155` | ✓ 四处逐字一致 |
| `system_configuration.rs:360` | ✓ 逐字一致 |
| `singlefs-format/src/lib.rs:167-170`、`:194-195` | ✓ 逐字一致 |
| `.claude/rules/fs-design.md:47` 段落标题、`:52`、`:54` | ✓ 三处逐字一致 |
| `.claude/rules/three-way-inference.md:120/122/147` | ✓ 三处逐字一致 |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md:176` | ✓ 逐字一致 |
| `.claude/kb/decisions/23-journal的角色与格式.md:460` | ✓ 逐字一致——该文件本身有未提交改动（净增 3 行，改动都在 362-390 区间，早于 460 行），核查时刻的行 460 内容与引用一致 |
| `second_transaction_supplement_two_unequal_devices.rs:39` | ✓ 内容存在且支持引用（该行是给 `SMALLER_DEVICE_BYTES` 常量的注释，写明「journal 环 768 MiB 不超过容量的四分之一」，与「按最小那块盘算」一致，不是逐字重复但语义对应） |
| `.claude/kb/invariants.md:76` | ✓ 逐字一致（该文件有未提交改动，但改动不影响 76 行） |
| `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8 / 16（不带行号） | ✓ 两个分项均存在，内容（每盘一份系统配置冗余；槽=世代号 mod 2）与引用一致 |

**核不动**：无（Rust 探针已在副本上重跑，两份 python 模型已独立重跑，其余均为文本现查）。

**opus 腿计数**：核了 22 处引用/复跑项（5 处点名重核 + 17 处其余文件行引用，行数用
`awk '/^### 其余引用逐条核/,/^\*\*核不动/'` 过滤后 `grep -c '^| \`'` 数出），
✓ 22 处，✗ 0 处，分不清 0 处，核不动 0 处。

## 二、云端正推（Sonnet）腿：K2 / K3

| 引用 | 核的结果 | 命令/证据 |
|---|---|---|
| `make_filesystem.rs:142` | ✓ `if ring_bytes > smallest / 4 {` | `awk 'NR==142'` |
| `allocator.rs:191-193/211-212/221/347/391/718/766/123` | ✓ 八处全部逐字一致 | `awk` 逐行核 |
| `crates/singlefs-checker/src/image.rs:137` | ✓ 与主树一致（同上，`image.rs` 有未提交改动，行序重排，但当前主树该行正是引用内容） | `awk` |
| `.claude/kb/decisions/16-发布语义.md:409`（已定项 5 定案段） | **✗ 真实引用错误** | 见下 |
| `.claude/kb/decisions/28-挂载期承诺量.md:376`（I-3.1 被审计对象、依据表第一行） | **✗ 真实引用错误** | 见下 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:220` | ✓ 逐字一致 | `awk 'NR==220'` |
| `.claude/kb/decisions/15-格式冻结政策.md:206` | ✓ 逐字一致 | `awk 'NR==206'` |
| `.claude/kb/feature-bits.md:14` | ✓ 逐字一致 | `awk 'NR==14'` |
| `.claude/rules/fs-design.md:23/27/52/198` | ✓ 四处逐字一致 | `awk` 逐行核 |
| `crates/singlefs-format/src/lib.rs:195` | ✓ 逐字一致（`UNIT_AREA_START_SLOT` 定义） | `awk 'NR==195'` |
| `crates/singlefs-format/src/lib.rs:161`（`JOURNAL_RING_DEFAULT_BYTES` 定义处） | **✗ 真实引用错误** | 见下 |
| `crates/singlefs-format/src/lib.rs:298-301` | ✓ 逐字一致（自证断言块） | `awk 'NR==298,301'` |
| `make_filesystem.rs:136-172`（`check_geometry` 函数） | **范围不准，非硬错** | 见下 |
| `make_filesystem.rs:36`、`system_configuration.rs:77/179/360/373` | ✓ 五处逐字一致 | `awk` 逐行核 |
| `recovery.rs:232/850/1427`、`mount.rs:1129`、`journal.rs:112` | ✓ 五处逐字一致 | `awk` 逐行核 |
| `transaction.rs:583/2259` | ✓ 两处均为 `journal_tail: plan.counter,` | `awk` |
| `system_configuration_mutability_classes.rs:61` | ✓ 逐字一致（`journal_ring_bytes: 805_306_368,`） | `awk` |

### 三处真实引用错误的细节

**错误 1**：报告 K2.1 行 18 引 `.claude/kb/decisions/16-发布语义.md:409`（声称是「已定项 5 定案段」）。
该文件当前主树 345 行、git HEAD 321 行——**409 行在两个版本里都不存在**，越界，不是「分不清」，
是纯粹的引用错误。经查，「已定项 5 定案段」实际内容（`有效 T_dirty = min(字段值, 环长 ÷ F)` 等）
在主树第 122 行（`git show HEAD` 里同一段在第 121 行，该文件本表格区有未提交的 +1 行改动，
但 409 与 121/122 相差近 300 行，不是那个 +1 位移能解释的）。

**错误 2**：报告 K2.4 行 50 引 `.claude/kb/decisions/28-挂载期承诺量.md:376`
（声称是「I-3.1 的被审计对象、依据表第一行」）。该文件仅 109 行（HEAD 与主树相同、无未提交改动），
376 越界。经查，「I-3.1（已分配统计对得上）的被审计对象」这句原文实际在第 35 行
（`| 容量、**已分配** | I-3.1（已分配统计对得上） 的被审计对象 |`），且确实是「依据」表的第一数据行——
内容对，行号差 341 行，无法用并发编辑解释。

**错误 3**：报告 K3.2 第二段括注引 `crates/singlefs-format/src/lib.rs:161`，声称是
`JOURNAL_RING_DEFAULT_BYTES` 的定义处。现查该常量实际定义在第 162 行
（`pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;`）；161 行是它上面一行的注释
（`// placeholder: D23（journal 的角色与格式） 已定项 19 ——……`）。该文件无未提交改动（HEAD 与
主树相同），不是并发编辑造成。本地攻方转述核对表第 11 行独立引用了同一常量在
`format/lib.rs:162`（`crates/singlefs-core/src/system_configuration.rs:341,343-347` 那一行的
背景引用里），与本次核查结果互证。

三处引用的**内容本身都真实存在于该文件**（只是行号写错），且都不影响 Sonnet 报告的判定结论
（K2.1「有效 T_dirty 夹取不是空间不足判定」、K2.4「环长是容量项的上游输入，不是九项之一」、
K3.2「今天代码已经预埋一条缝」）——三句引文经独立核查内容为真，只是行号本身不可用。

**范围不准（非硬错）**：`make_filesystem.rs:136-172` 声称是 `check_geometry` 整个函数，
现查该函数实际是 136-176（`fn check_geometry(` 到闭合 `}` 共 41 行，不是 37 行），
报告引用的具体内容（155-156 行 `unit_area_start` 算式）落在声明范围内，但声明的函数边界本身
少算了 4 行（未含 `Ok(())` 与最后一个 `}` 之前的 region_devices 校验分支）。判定：**不算 ✗**，
算「范围表述不精确」。

**sonnet 腿计数**：按表格行数出（`sed -n '126,144p' ... | grep -c '^| \`'`）
核了 16 行引用（有的一行含多处具体行号），✓ 12 行，✗ 3 行（16-发布语义.md:409、
28-挂载期承诺量.md:376、format/lib.rs:161），范围不精确 1 行（不计入 ✗，另记：
make_filesystem.rs:136-172），分不清 0 处，核不动 0 处。

## 三、本地攻方（K4 + K3 字面格）腿

提示文件 `elastic-ring-r1-local-attack.md` 的 FACT 1–7 与转述核对表
`elastic-ring-r1-local-attack-translation-audit.md` 13 行逐条核（覆盖全部 7 条 FACT）。

| 转述核对表行（按 FACT 分组） | 核的结果 | 命令/证据 |
|---|---|---|
| FACT1「smallest device / 4」 | ✓ `decisions/23:460`「约束环≤设备容量÷4」+ `:462`「按最小那块盘的容量÷4拒绝越界」+ `make_filesystem.rs:141-142` `ring_bytes > smallest / 4` 三处全部逐字一致 | `awk`/`sed -n` |
| FACT1「只存这一字段，不另存容量/比例」 | ✓ `decisions/23:460` 同段逐字含此句 | 同上 |
| FACT1「无下限、无运行时改环机制」 | ✓ 背景材料 31-32 行「用户设最小值：没有：全仓零命中」「挂载之后运行时缩：没有：全仓零命中」逐字一致；`grep -rn "minimum_ring\|maximum_ring" crates/singlefs-core/src crates/singlefs-format/src` 本轮独立重跑，0 命中 | `grep` 重跑 |
| FACT3 T_dirty「字段不动」 | **分不清**：引用 `decisions/16-发布语义.md:121`，主树当前该行为空行，内容实际在 122 行；`git show HEAD` 该文件第 121 行正是所引内容——文件当前有未提交改动（表格区 +1 行，改动早于该行），是「腿引用时的行号在腿交回之后被移位」的情形，不判 ✗ | `git show HEAD:...` 对照 |
| FACT3 T_time「覆盖字段并回写」 | **分不清**（同上，同一行、同一位移原因） | 同上 |
| FACT3「两字段两规则」 | ✓ `decisions/16:133` 在当前主树逐字一致：「同一段里两个字段两种规则，不按同一规则读」——按规则对主树核，直接核对当前主树内容，不追究该文件其余位移 | `awk 'NR==133'` |
| FACT4「写进系统配置、mkfs算、mount复算」 | ✓ `invariants.md:76` 逐字一致（该文件有未提交改动，但不影响 76 行） | `awk` |
| FACT4「checker 未实现」 | ✓ `invariants.md:76` 状态列「未实现」+ `:84-86` 警告块逐字一致 | `awk` |
| FACT5「环÷F、恒真」 | ✓ `checkpoint-trigger-r1-main-verification.md:21-47` 逐字一致（历史归档文件，仍在仓内）+ `format/lib.rs:168-169`、`system_configuration.rs:343-347` 逐字一致 | `awk` 三处 |
| FACT6「12 KiB」 | ✓ `checks-owed.md:276` C310 内文逐字含「按『任一事务』读只要 12 KiB」（该文件有未提交改动，但改动在 333 行之后，不影响 276 行） | `awk` |
| FACT7「四个字段 W/X/Y/Z」 | ✓ `system_configuration.rs:341,343-347`、`format/lib.rs:162,165` 六处全部逐字一致 | `awk` 逐行核 |
| FACT7「F 编译期常量」 | ✓ `format/lib.rs:165` 逐字一致 | `awk` |
| FACT7「全仓搜索零命中」 | ✓ 独立重跑 `grep -rn "ring_min\|ring_max\|min_ring\|max_ring" --include="*.rs" .` 得 2 命中，均在 `research/e7-index-bench/`（与「唯二命中在无关实验二进制里」一致）；`grep -rn "minimum_ring\|maximum_ring" crates/singlefs-core/src crates/singlefs-format/src` 得 0 命中 | 见下方命令 |

**grep 独立重跑**：

```
$ grep -rn "ring_min\|ring_max\|min_ring\|max_ring" --include="*.rs" .
research/e7-index-bench/src/bin/e47_ring_loss.rs:355:    fn survivors_equal_ring_minus_the_dead_region() {
research/e7-index-bench/src/bin/e45_span_ring_cost.rs:105:                 ring_min_bytes={} txns_lost_when_torn={}",
$ grep -rn "minimum_ring\|maximum_ring" crates/singlefs-core/src crates/singlefs-format/src
(0 命中，退出码 1)
```

与转述核对表宣称的「唯二命中在 research/e7-index-bench/ 的无关实验二进制里」「核心/格式 crate 内命中 0」
完全一致。

### 攻方自报「三处需要补回的限定词、两处主动补充并标注理由的括注」

**说明**：本地攻方的转述核对表本身没有单独一段写死「共 3 处、共 2 处」这个计数——这个数是
从表格 13 行的「首稿缺的」列文字（首稿曾/一度/只写了……）与「无缺——…另加一句括注」两类文字
归纳出来的。核查按内容归类，逐一核对：

**「补回限定词」类（首稿确实漏过、定稿补回，且补回内容确实在原文里）**：

1. FACT1「不另存容量/比例」半句——`decisions/23:460` 同段确有此句，核查 ✓
2. FACT1「无运行时改环机制」半句——背景材料 31-32 行确有此句，核查 ✓
3. FACT3 T_dirty「字段不动」——`decisions/16` 当前 122 行（引用写 121，见前文「分不清」）确有此句
4. FACT3 T_time「覆盖字段并回写」——同上文件同一行，同「分不清」
5. FACT4「写进系统配置、mkfs算mount复算」半句——`invariants.md:76` 确有此句，核查 ✓

**「主动补充并标注理由的括注」类（定稿加了原文没有、但注明理由的内容）**：

1. FACT6 括注：C310 条目里还有一个「满窗」读法给出更大数字，本轮不用——核查 `checks-owed.md:276`
   同一行内确实并存两种读法（12 KiB 与「768 MiB 或 T_dirty 压到 170.7 MiB」），括注内容真实、
   排除理由（避免模型混用两种读法）合理，核查 ✓
2. FACT7「and no others」被去掉、改成「Among the on-disk fields that this round cares about」——
   核查 `system_configuration.rs` 写盘代码里确实还有 `JOURNAL_RING_START_SLOT`（`:340`）、
   `ROOT_RING_BASE_SLOT`（`:352`）等另外的、与环相关但未被计入「四个字段」的量，
   去掉「and no others」这个绝对化限定是必要的收窄，核查 ✓

**FACT1「smallest device」限定词**（表格第 1 行，写法是「首稿若…会漏掉」的假设句，非确认的实际
首稿事件）：核查内容真实（`decisions/23:462` + 代码），但该行本身不构成「首稿确实漏过」的
既成事实描述，是防错说明，不计入上面 5 处，另记为一条独立、真实、但性质不同的核对。

**结论**：主 agent 描述的「3+2」在核对表里没有一份显式的编号清单可以逐字对应，是从散文归纳的；
本次核查按内容逐条核了以上 7 处候选（5 处「补限定词」+ 2 处「括注」），**内容全部为真**，
无一处捏造；FACT3 的两处因文件被并发编辑位移，记「分不清」而非 ✓/✗。

### 干净样本的字词损坏检查独立重跑

```
$ nice -n 19 python3 research/scripts/corruption-check.py .../output-s1.md
绿  cjk=0 words=720 fffd=0 ... 退出码 0
$ nice -n 19 python3 research/scripts/oov-check.py .../output-s1.md
绿  生词=3 拼接=0（DefaultWins、derivable）退出码 0
$ nice -n 19 python3 research/scripts/corruption-check.py .../output-s2.md
绿  cjk=0 words=638 fffd=0 ... 退出码 0
$ nice -n 19 python3 research/scripts/oov-check.py .../output-s2.md
绿  生词=5 拼接=0（contradicting、DefaultWins、computable）退出码 0
```

字数（720/638）与运行记录逐字一致。**判定：✓，运行记录的字词损坏判定可独立重现。**

**local-attack 腿计数**：核了 13 条转述表行（覆盖全部 7 条 FACT，`grep -c '^| "'
.../elastic-ring-r1-local-attack-translation-audit.md` 数出）+ 2 条 grep 命令独立重跑 +
2 份样本的字词损坏检查独立重跑 = 17 项，✓ 15 处（11 条转述表行 + 2 条 grep + 2 份样本），
分不清 2 处（FACT3 两行，因文件被并发编辑位移），✗ 0 处，核不动 0 处。

## 四、主 agent 已自查坐实的两处

任务里已列的两处（`decisions/23-journal的角色与格式.md` 已定项 19 ③「随环长走」vs 实现取编译期
常量、以及环 1 GiB 时单元区该落槽 66560 而实际仍是 50176）按任务要求不重做。补一句现查：
`decisions/23:460`「单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）」与
`crates/singlefs-format/src/lib.rs:195`「`pub const UNIT_AREA_START_SLOT: u64 = 50176;`」
两处引用行号本身经上面 opus 腿引用核查已确认逐字一致，可以支撑主 agent 的这一现查。

## 总计与没做什么

| 腿 | 核了 | ✓ | ✗ | 分不清 | 核不动 |
|---|---|---|---|---|---|
| 云端攻方（opus） | 22 | 22 | 0 | 0 | 0 |
| 云端正推（sonnet） | 16 | 12 | 3 | 0 | 0（另有范围不精确 1 行，单列不计入✓✗）|
| 本地攻方（local-attack） | 17 | 15 | 0 | 2 | 0 |
| **合计** | **55** | **49** | **3** | **2** | **0** |

### 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 不判「三方一致」是否成立、K5 该收甲/乙/丙、K1/K2/K3 的判定对不对——那是主 agent 的职责。
- Sonnet 腿的「打中之后先答四句」表格与 Opus 腿的判臂逻辑本身不核（那是推理，不是引用）。
- 未重新核算 shrink_replay_model.py 与 k5_capacity_scan.py 里嵌入的算术公式是否正确建模了
  `record_offset`/`scan_journal`/`journal_in_flight_record_limit` 的真实语义（这需要判断模型是否
  忠实翻译了源代码逻辑，属于对腿的推理方法论评价，不是「文件:行是否对得上」这类引用核实）；
  只核了模型复跑产物与腿存档产物逐字节一致、模型源码注释里指向的行号内容真实。
- 未对 `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8/16 的分项做逐字比对整段抄录
  （只核了关键句子存在且与引用语义一致），因为报告引用时未给出具体行号，无法做行号核对。
- `make_filesystem.rs:136-172` 的范围不精确只记为一条独立发现，未展开核查报告其余处是否有
  类似的「函数起止行范围写窄/写宽」问题——这需要对报告里每一个函数级引用都反查函数边界，
  这一轮的点名重核项之外没有做全量扫描。
- 未跑 `gate.sh`，未跑门禁任何阶段（`stage-owners.tsv` 未登记核查员这一轮的阶段）。
- 未改主工作区任何文件；两处 rsync 副本（`/tmp/claude-1000/elastic-ring-r1-verifier/repo/`，
  含 `target/`）与 python 复跑产物留在草稿目录，不入库。
