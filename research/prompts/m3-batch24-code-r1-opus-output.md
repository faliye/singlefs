# m3-batch24-code-r1 攻方腿报告（Opus，K2：N1、N5、N3，另做 m3-batch23-code-r1 N1、N3 第二次抽样）（2026-10-06）

立场：攻方，造反例。只跑 harness 档，GPU 不跑（GPU 那一侧读 wgsl、用 CPU 算期望）。所有数都在冻结副本的拷贝上量（副本上的数，不算入库装置上的数）。

## 复跑

模型在 `research/prompts/m3-batch24-code-r1-opus-model/`。仓路径取 `SINGLEFS_REPO`（冻结副本只有 `crates/` 与 `Cargo.*` 时另给 `SINGLEFS_TOOLS=<仓根>` 取包装脚本）；编译 6G、跑 4G 经 `scripts/hosts/run-with-memory-cap.sh`，线程经 `capped.sh`（`OPUS_THREADS`，默认 10）；每段前台四分钟以内。

```
SINGLEFS_REPO=/tmp/claude-1000/m3-batch24-code-r1/tree SINGLEFS_TOOLS=<仓根> bash research/prompts/m3-batch24-code-r1-opus-model/rerun.sh <空工作目录> <段|all>
bash research/prompts/m3-batch24-code-r1-opus-model/rerun.sh compare <工作目录>
```

段：`setup build n1-inode n1-T1 n1-T2 n1-T3 n1-T3ring-a n1-T3ring-b n1-T3ring-c n1-L1 n1-L2 n1-L3 n1-stale n3-turn n3-b1 n3-b2 fix-setup fix-build fix-stale fix-matrix fix-kbi crash-build crash-count`，之后 `summary`。交回前在空目录 `/tmp/claude-1000/m3-batch24-code-r1-opus/rc` 里逐段从头跑过（三段 T3ring 并行），`compare` 原样输出：`SUMMARY_MATCHES 1085 行`。探索时各次跑的 RESULT 行（T3ring 除外）与这次复跑逐行相同（1 003 行）。

| 文件 | sha256 |
|---|---|
| `crash/Cargo.toml.in` | `a8bc0de958a7ce22635303549a79fa8e77f878c03330a5ad83e31d8af5b0feec` |
| `crash/src/main.rs` | `f2761bcc7f050c0d05234909f7c52a1767acf6f0e691d7d47f39327ed79645e3` |
| `expected-summary.txt` | `81416e031457f8ba98331cb17d77561a2eb76b19412900a473a821f760e99c9c` |
| `fix-F1.diff` | `8b8db284260ac68e158ccf6d36e5e6d14945d25304613db733cb9c8917c0c4f3` |
| `opus_b24_attack.rs` | `12db68157867d6f8f2cdb2f838bc9041f61924a41c251bfcfae6606634f8d27b` |
| `rerun.sh` | `45adeb3b7e24a50272abdf32ae223097a3859c2b38be7cd868632749586f8540` |

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| K2 N1 单故障：类型段 × 孩子的类 × 树的层数 | 没打中（一次抽样） | 265 形（墓碑 / 谱系 1–3 层、转过根环、inode 三态）全部只红目标 | 1.1 |
| K2 N1 两故障：条目坏 + 孩子在盘 0 那份陈旧、头里的类恰好是类型段那一类 | **打中 H1**（漏判 + 连带红） | `child_unit_class_other_than_the_entry_type_names` 在第一份头里的类等于类型段时不验就早退，I-9.23 / I-9.2 报成立，另多出 I-7.2（及 I-3.1、I-3.11、I-5.1）；陈旧的换到盘 1 就判对，按盘的次序不对称 | 1.2 |
| H1 的改法 F1 | 只在我的模型上量过、被攻过零轮 | 等于类型段那一类也按那一类的长度验校验和，不过就看下一份；四形转对、矩阵与 `checker_known_bad_images` 121 条不变；变异表一行锚点要改（推的） | 1.3 |
| K2 N5 CPU 与 GPU 判得不一样 | 没打中（读码，一次抽样） | 新三处调用、早退、旧那一支逐支相同；H1 在 GPU 上同样漏判（推的，GPU 不跑） | 二 |
| K2 N3 合法历史上新九条落缺省理由或该判没判 | 没打中（一次抽样） | 6 批 1 830 次判定：缺省理由 0、专门理由说假话 0、九条红 0；崩溃状态最小一段 1 048 583 个，没跑 | 三 |
| m3-batch23-code-r1 N1、N3 第二次抽样 | 没打中（读码）→ 两次都没打中 | I-9.24 同号、层级 0 的 inode 根、同号记录、I-1.10、走读内核 I-9.17 / I-9.23 五处 CPU 与 GPU 相同 | 四 |

## 一、N1（CPU 读孩子的类）

### 1.1 单故障矩阵：类型段 × 孩子的类 × 层数 —— 没打中（一次抽样）

造法（`opus_b24_attack.rs` 的 `n1_type_segment_and_child_class_matrix_over_tree_depths`、`n1_inode_tree_type_segment_and_child_class_matrix`）：内存稀疏盘上建起点池，最新那条根的树里每一层每个码 2 节点的每条条目，类型段换成 {0, 1, 2, 5, 7, 65535} 里别的每一个值（节点重封、整单元校验和沿引用链补到根槽），每张镜像跑池级 checker，判「目标那条红、别的一条不红」。

| 起点 | 树 | 层数 | 形数 | 只红目标 |
|---|---|---|---|---|
| T1 / T2 / T3（旋钮一片 1 条、一个节点 2 条，删 2 / 3 / 5 个号） | 墓碑树 | 1 / 2 / 3 | 10 / 25 / 50 | 全部 |
| T3ring（T3 再建 34 次号，根环转过一圈） | 墓碑树 | 3 | 50（每形要几秒，复跑分三段 17 / 17 / 16 形并行跑） | 全部 |
| L1 / L2 / L3（同一旋钮，回退 2 / 3 / 5 次，每次回退到这一次删之前） | 谱系树 | 1 / 2 / 3 | 10 / 25 / 50 | 全部 |
| I0 / I1half / I2done（500 个号三片叶；中间那片叶里一个号发了意图、队列非空；删完三个号、有墓碑） | inode 树 | 1 | 15 / 15 / 15 | 全部 |

孩子单元自己的类标签改写（按原类版式重封、校验和补到根槽）那一组另判：改成 1 的只红目标；改成 2 或 9 的连带 I-4.8、I-7.2、I-7.4（3 → 2 再连带 I-1.2、I-4.2、I-4.9）。这些是「孩子的头用不了」的旧判法（`walk.rs` 读码 2 / 码 3 头时类标签不对就判头用不了、走读失败），与这一批改的「按孩子头里的类认」无关，不算 N1 的打中。

层数这一维：类型段 0 → 1 打在层级 3 的根上（孩子是层级 2 的节点，下面还有两层），孩子按码 2 一槽登记、不往下走，照样只红 I-9.23。我原先推「孙辈不登记会连带 I-3.1」，量出来不连带，这一条推测作废。

### 1.2 打中 H1：孩子的第一份副本陈旧、而它头里的类恰好是类型段说的那一类时，「按孩子头里的类认」整个失效（漏判 + 连带红）

**镜像**（`n1_entry_type_mismatch_with_one_stale_copy_of_the_child`）：两处故障。① 条目改坏，与第 24 批要修的那几形相同（N4-i：墓碑树层级 1 根条目 0 类型段 1 → 0、身份三段清零；N4-f：层级 2 根条目 0 类型段 0 → 1；inode 根条目 1 类型段 2 → 0、身份三段清零）。② 孩子在盘 0 上那一份陈旧：槽里是别处一个单元的原样字节，头里的类标签恰好等于类型段说的那一类（N4-i 那形写一个码 2 节点，N4-f 那形写一片码 3 容器的前 16 KiB）。盘 1 那一份完好，按它自己那一类的长度自证得过。

**结果**（原样，`expected-summary.txt` 里同行）：

```
RESULT stale T1 ctrl-stale-only dev0 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.4"] I-9.23=成立
RESULT stale T1 ctrl-n4i-only violated=["I-9.23"] I-9.23=不成立
RESULT stale T1 n4i+stale dev0 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.2", "I-7.4"] I-9.23=成立
RESULT stale T1 n4i+stale dev1 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.4", "I-9.23"] I-9.23=不成立
RESULT stale T2 ctrl-n4f-only violated=["I-9.23"] I-9.23=不成立
RESULT stale T2 n4f+stale dev0 violated=["I-2.1", "I-3.1", "I-3.10", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"] I-9.23=成立
RESULT stale T2 n4f+stale dev1 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.4", "I-9.23"] I-9.23=不成立
RESULT stale I0 type0-zero-identity+stale dev0 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"] I-9.2=成立
RESULT stale I0 type0-zero-identity+stale dev1 violated=["I-2.1", "I-4.8", "I-7.4", "I-9.2"] I-9.2=不成立
```

陈旧的在盘 1：目标红，连带的与「只有陈旧副本」的对照（I-2.1、I-3.10、I-4.8、I-7.4）逐条相同。陈旧的在盘 0：目标报成立，另多出 I-7.2（T1），或 I-3.1、I-3.11、I-5.1、I-7.2（T2、inode）。这就是第 24 批改之前的样子（m3-batch22-code-r1 判决第二节 N4 那一行：N4-i「I-9.23=成立」、N4-f 连带 I-3.1、I-3.11、I-5.1、I-7.2），只是多了一处陈旧副本。

**每一步许可它的那一句**：
- `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4098`：「if Some(class_in_the_header) == class_the_entry_type_names {」，命中就 `return None`，不验校验和、也不看第二条位置条目；
- 同文件 `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:4082`：「/// 逐条位置条目：头里的类标签就是 `class_the_entry_type_names` 的，交回 None（照类型段那一支走，读不出照旧归 I-2.1）；」——是有意这么写的；
- 交回 None 之后走旧那一支：墓碑树类型段 0 读不出孩子时 `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2211`：「&& child_class.is_none_or(|class| class == 2);」让「子单元是码 2」放过；类型段 1 那一支 `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2347`：「self.note_references_of_a_pointer_followed_by_the_walk(child, 2, &what);」把一槽的节点按两槽登记，再读不出、记走读失败，身份那一判跑不到；inode 那一支读不出时 `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2529`：「"inode 树的孩子两份都读不到对得上的"」之后 `continue`，I-9.2 这一条条目一判都没有。
- 被违反的字面：`.claude/kb/invariants.md:310`：「0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3」与同一行「身份不符单独判定，不再记走读失败」；inode 树 `.claude/kb/invariants.md:289`：「类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0」。盘 1 那一份就是指针的位置条目校验和认的孩子，它的类与类型段不符，checker 读得到这一份（陈旧的换到盘 1、好的那份在盘 0 时，它就按好的那份认出了孩子的类）。

**四句**：
1. 分不分辨臂：这一轮没有臂，打中的是实现与判决第三节第 1 条的改法「判孩子类标签只读单元头、不按类型段定长度」之间的一处缝：只在第一份副本的头上认类、认到「就是那一类」就停。
2. 被判的系统看不看得到判别它的东西：看得到。盘 1 那一份按它头里的类（码 2 / 码 3）的长度自证得过，同一个函数在「是另一类」那一支本来就这样验；盘 1 陈旧的对照里 checker 就是这样认出来的。
3. 满足的是判据字面的哪一句：m3-batch22-code-r1 正文第五节「漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立」（I-9.23、I-9.2 报成立），连带的 I-3.1、I-3.11、I-5.1、I-7.2 是同一节「误红」的反面——不是「一个从入口走得到的已发布状态」，按字面不归误红；但它们正是第 24 批 N4 四形要消掉的连带红。
4. 跑前条款给的改法在打中的格上还中不中：判决第三节第 1 条的改法已经落地，这两格照中——改法字面「只读单元头」没写读哪一份、头里的类等于类型段时要不要验。

### 1.3 H1 的改法 F1（只在我的模型上量过、被攻过零轮）

F1：`child_unit_class_other_than_the_entry_type_names` 里头里的类等于类型段说的那一类时，也按那一类的长度验位置条目里的整单元校验和，验得过才交回 None，验不过看下一条位置条目（diff 在草稿 `fix-F1.diff`，模型目录里同名一份）。GPU 同改：`w_child_unit_class_other_than_the_entry_type_names` 两份副本各自在「等于类型段那一类」时先 `slice_crc_matches(copy, 那一类的长度, sumN)`。

| 格 | 冻结副本 | F1 副本 | 标 |
|---|---|---|---|
| T1 N4-i + 盘 0 陈旧 | I-9.23=成立，多 I-7.2 | `RESULT stale T1 n4i+stale dev0 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.4", "I-9.23"] I-9.23=不成立` | 量过 |
| T1 N4-a + 盘 0 陈旧 | I-9.23 红，多 I-7.2 | `RESULT stale T1 n4a+stale dev0 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.4", "I-9.23"] I-9.23=不成立` | 量过 |
| T2 N4-f + 盘 0 陈旧 | I-9.23=成立，多 I-3.1、I-3.11、I-5.1、I-7.2 | `RESULT stale T2 n4f+stale dev0 violated=["I-2.1", "I-3.10", "I-4.8", "I-7.4", "I-9.23"] I-9.23=不成立` | 量过 |
| inode 类型段 0 + 盘 0 陈旧 | I-9.2=成立，多 I-3.1、I-3.11、I-5.1、I-7.2 | `RESULT stale I0 type0-zero-identity+stale dev0 violated=["I-2.1", "I-4.8", "I-7.4", "I-9.2"] I-9.2=不成立` | 量过 |
| 1.1 节的单故障矩阵（T1、T2、T3、L2、L3、I0、I1half、I2done） | 见 1.1 | 每一行与冻结副本逐行相同（`diff` 零差异） | 量过 |
| `checker_known_bad_images` 整个二进制 | 合入验证报全绿 | `test result: ok. 121 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.74s` | 量过（F1 副本） |
| GPU 同一形 | 读码：同一处早退（下一节） | GPU 同改之后同 CPU | 推的（GPU 不跑） |
| 变异表 | — | 第 24 批 K「孩子的类不验整单元校验和」那一行的锚点（12 个空格起的 `.is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);`）在 F1 之后多命中一处（新加那行缩进 16 个空格，含这个子串），锚点要改成两处共用一个小函数之后再锚 | 推的 |

F1 没碰的：两份副本都陈旧或都坏，或盘 1 陈旧、盘 0 恰好是「另一类」的合法单元而校验和对得上（碰撞级，不现实）。

## 二、N5（GPU 读孩子的类）：逐支对 CPU（读码，GPU 不跑）

GPU 的单元槽版本按「这一状态里落了的写」拼出整 32 KiB 再算前 16 KiB 与前 32 KiB 的 CRC（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:741`：「/// - 单元槽：`[前 16 KiB 的 CRC-32C, 前 32 KiB 的 CRC-32C（落点只有 16 KiB 时 0）, 标志位, 0]`；标志位：位 0 头校验和按自述的类过、」，每个碰过的槽按 32 KiB 建落点，盘尾才 16 KiB，`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:498`：「let length = if offset + DATA_UNIT_BYTES <= device_bytes {」）。所以「按那一类的长度读回、整单元校验和对得上」两侧读到的字节相同，盘尾之外不差。

| 支 | CPU（`walk.rs`） | GPU（`crash_judge_checker.wgsl`） | 同不同 |
|---|---|---|---|
| 墓碑 / 谱系类型段 0，孩子是别的登记类 | 判 `identity_invariant` 红、按孩子那一类登记、`continue` | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3014`：「let class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);」，置位、登记、`continue` | 同 |
| 类型段 = 打包记录类型，孩子是别的登记类 | 同上 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3060`：「let class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);」 | 同 |
| 类型段不认得 | 不认孩子的类，按层级定一槽或两槽 | 同（不调新函数，层级定槽） | 同 |
| inode 类型段 0 / 2 / 不认得 | 三支都调，不认得时交回孩子自证过的类 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2615`：「let class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);」，不认得时传 NONE | 同 |
| 第一份头里的类 = 类型段那一类 | 早退 None，不验（H1） | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2946`：「if (class_in_the_first_header == class_the_entry_type_names) {」，早退 NONE、不验 | 同，H1 两侧一起中 |
| H1 之后旧那一支（类型段 0 读不出孩子） | `is_none_or` 放过 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3022`：「let child_is_a_code_two_node_or_unreadable = child_unit.version == NONE || rd8(child_unit.base, 6u) == 2u;」，层级 1 再 `w_failures + 1` | 同 |
| H1 之后旧那一支（类型段 1 指码 2） | 按两槽登记、读不出记走读失败 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3066`：「w_note_pointer_references(child, 2u);」，读不出 `w_failures + 1` | 同 |
| 位置条目几份 | `child.locations` 两条 | `device0/slot0`、`device1/slot1` 两条 | 同 |

用 CPU 算的 GPU 期望（推的，GPU 不跑）：1.1 节每一形 GPU 在目标那一位上置位、不加 `w_failures`，与 CPU 只红目标同（inode 树层级 1；容器树 12 层栈装得下 3 层）；H1 那几形 GPU 同样不置 I-9.23 / I-9.2 那一位、记走读失败（`w_failures`），与 CPU 一致地漏判。没找到一支 CPU 与 GPU 判得不一样。GPU 只判合法流的崩溃状态，H1 要条目先坏，合法流上碰不到；只在对拍若造这一形时显出来。

## 三、N3（新九条的不适用理由）—— 没打中（一次抽样）

### 3.1 读码：缺省理由在合法镜像上落不到

`Judgements::into_report` 只在一条一次都没评估、也没有任何 `not_applicable` / 射程里判不了时才给缺省理由（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/image.rs:358`：「.unwrap_or("这个镜像上没有它判的对象"),」），`not_applicable` 先到先得（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/image.rs:313`：「self.not_applicable.entry(invariant).or_insert(reason);」）。池级 checker 入口里在走到删除那两遍之前提前交回的四处：前两处与「没有一块盘交得出有效的系统配置」那一处都给全部不变量写了整池理由；剩下一处是根环里一条自证过的根都没有（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:10489`：「"根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",」，紧接 `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:10491` 交回），那时九条都落缺省理由，但 I-7.1 同时红，而且层 0 的基镜像是 mkfs 之后的（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:294`：「/// 前提有两条，写表的来路都满足：基镜像里的根带的 F 为 0（层 0 的基是 mkfs 之后，崩溃注入的基是空池），」），合法的层 0 状态里总有 mkfs 的第 0 代根。删除那两遍（`judge_deletion_state_of_the_versions`、`judge_tombstones_and_the_lineage_of_the_versions`）里没有提前 `return`，九句专门理由每次都写到。

### 3.2 随机合法历史：1 830 次判定，九条零缺省理由、零假话、零红

造法（`n3_reasons_of_the_nine_on_random_legal_histories`）：起点同 m3-batch22-code-r1 攻方的起点池（新池新建文件 → 第一个文件顺序写成 3 个单元 → 建 2、3 号 → 建 4 号），每一步从七种用户动作里随机抽（建 1–2 个号、对随机一个活号发意图、删一批、走空、回退到前 1–12 新的随机一版、进程退出再挂、崩溃恢复落到前一条根），被拒的换回动作之前那一份，做成的判一次。每次判定逐条看九条：不适用的按理由短名计数；核「不适用」那句话在现行那一版上真不真（队列非空却报「每一版的队列都是空的」、墓碑树不是零根却报「每一版墓碑树都是零根」、谱系树不是零根却报谱系那一句、墓碑多于一片却报「零根或只一片容器」、有记录却报「没有号可比」、没有谱系记录而有墓碑指着同号同出生代的记录却报 I-9.19 那一句、I-9.24 报不适用），算假话。种子与段数都与 m3-batch22-code-r1 攻方的不重。

| 批 | 旋钮 | 前缀 | 种子 | 段 × 步 | 判定次数 | 旗 |
|---|---|---|---|---|---|---|
| b1-1 | 产品 | 无 | 10000–10039 | 40 × 10 | 323 | 无 |
| b1-2 | 一片 1 条、节点 2 条 | 无 | 11000–11039 | 40 × 10 | 318 | 无 |
| b1-4 | 产品 | reissue（建号那次被崩溃恢复抛弃、新线重发同号） | 10100–10139 | 40 × 10 | 317 | 无 |
| b2-1 | 一片 2 条、节点 2 条 | reissue | 13000–13039 | 40 × 10 | 303 | 九条之外 I-3.1 一次 |
| b2-2 | 产品 | 无 | 12000–12013 | 14 × 25 | 280 | 九条之外 I-3.1 六次 |
| b2-3 | 一片 1 条、节点 2 条 | 无 | 12100–12113 | 14 × 25 | 289 | 九条之外 I-3.1 七次 |

b1-3（旋钮 2、reissue）第一次经内存包装退 252（排不上），那一次的输出不算，原样参数作 b2-1 重跑。第一批 4 件、第二批 3 件并行（复跑每批 3 件），每件单线程，估时按探针（2 段 × 8 步 14 次判定 4.9 秒）定，实测每批 91–98 秒。九条判定合计（六批相加，命令数出来）：I-9.24、I-9.29 各「成立」1 830；I-9.18、I-9.19、I-9.20、I-9.23 各「成立」1 304、各报专门理由 526；I-9.17「成立」1 374、专门理由 456；I-9.21 产品入口 920 次专门理由「零根或只一片容器」、旋钮入口 910 次「产品容量那一条不判」；I-9.22 旧入口 920 次「旧入口」、旋钮入口「成立」646、专门理由 264。缺省理由 0 次，假话 0 次。I-3.1 那 14 次都在做过「崩溃恢复落到前一条根」的历史里（m3-batch22-code-r1 判决「附带」那一行已转根环线），不在九条里。

I-9.18、I-9.19 与 I-9.20、I-9.23 计数逐个相等，我先疑心 I-9.18 在没有队列号时也被当成「评估过」。另造一段写死前缀的（删完 2 号之后再建 0 / 10 / 30 次号）核：

```
RESULT n3turn turns=10 queue=[] tombstones=1 I-9.17=不适用[R-no-lineage-entry] I-9.18=成立 I-9.19=成立 I-9.20=成立 I-9.21=不适用[R-no-non-rightmost] I-9.22=不适用[R-old-entry] I-9.23=成立 I-9.24=成立 I-9.29=成立
RESULT n3turn turns=30 queue=[] tombstones=1 I-9.17=不适用[R-no-lineage-entry] I-9.18=不适用[R-no-queue] I-9.19=不适用[R-no-tombstone-naming-record] I-9.20=成立 I-9.21=不适用[R-no-non-rightmost] I-9.22=不适用[R-old-entry] I-9.23=成立 I-9.24=成立 I-9.29=成立
```

意图那一版（队列非空）还在候选集里时 I-9.18 成立是真判到了；根环转过去之后转成专门理由，I-9.20、I-9.23 仍成立。计数相等是随机历史短、根环没转过去，疑点撤回。

### 3.3 崩溃状态：没跑（状态数过约 10⁶）

崩溃原型（`crash/src/main.rs`，录制流只录最后一两次发布）只数了闭式状态数：C1 空号的意图 67 108 871，C2 意图之后删完那一步 1 048 583，C3 删完之后回退 1 048 583，C4 回退之后再删 67 108 871，C5 旋钮下墓碑树长到层级 2 268 435 463，C6 旋钮下谱系树长到层级 2 4 363 124 755。最小的两段也略过 10⁶；按 3.2 量的每次判定约 0.3 秒、枚举器自己每个状态再跑两遍恢复与 checker 推，C2 一段十个线程要十几个小时，过 40 分钟，又是「一段历史自己就超过约 10⁶」，照派发规则不跑，交主 agent。3.1 的读码是这一格唯一的依据：崩溃状态的镜像同样走不到缺省理由，假话那一格没量。

## 四、m3-batch23-code-r1 N1、N3（CPU 与 GPU 一致）第二次抽样 —— 没打中（读码）

上一轮攻方把扫描内核译成 Rust 逐状态比；这一次换路子，只读码逐支对几处上一轮报告没单列的：

| 处 | CPU | GPU | 同不同 |
|---|---|---|---|
| I-9.24 同号几条 | 队列读成按号的集合 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2373`：「// I-9.24：读得出的队列里不同的号至多一个（walk.rs 把队列读成按号的集合，同一个号的几条算一条）」，逐条与第 0 条比 | 同（「每条都等于第 0 条」⟺「集合至多一个」） |
| 删除态扫描读 inode 树：层级 0 的根 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:8574`：「return if root.entries.is_empty() {」，零条目读成空、有条目判读不全 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2222`：「if (root.level == 0u) {」，交回 `entry_count == 0` | 同 |
| 同号两条记录 | 后读到的盖掉先读到的 | `d_note_inode_record` 找到同号就改那一行（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2141`：「sws(row + 5u, object_birth.x);」） | 同 |
| 登记过字段表的树根判 I-1.10 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2022`：「.judge("I-1.10", entry_width == entry_field_table_bytes, || {」，不看条目数 | `/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2888`：「w_judge(INVARIANT_I_1_10, read.n.entry_width == field_table_width);」，不看条目数 | 同 |
| 走读内核 I-9.17 / I-9.23 | 见第二节 | 见第二节 | 同 |

两次抽样都没打中，按 `.claude/rules/three-way-inference.md` 记「没打中」。这一次是读码，没跑译本。

## 五、顺手看到的（不在打中里）

- inode 那一支的违例说明把 `Option` 原样打出来：`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2505`：「"inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"」，量出来是「说孩子是 Some(IndexNode)」（复跑日志 `fix-stale.log` 里同一句），容器树那一支是「说孩子是 IndexNode」。类型段不认得时该打「None」（推的，没量）。只是说明文字，用例按子串认，不影响判定。
- 容器树类型段不认得那一支仍按节点层级定一槽或两槽（`/tmp/claude-1000/m3-batch24-code-r1/tree/crates/singlefs-checker/src/walk.rs:2240`：「let span_in_slots = if node.level >= 2 { 1 } else { 2 };」），inode 树那一支改成按孩子自证过的类定。两处做法不同；单故障下孩子的类与层级说的一致，矩阵里 7 与 65535 那几形都只红目标，没造出分得开的镜像。
- 随机历史里九条之外的 I-3.1 红 14 次，全在做过「崩溃恢复落到前一条根」的段里，与 m3-batch22-code-r1 判决「附带」那一行同族，已转根环线。

## 没打中的形状

| 形状 | 取样范围 |
|---|---|
| 单故障：条目类型段换成 {0, 1, 2, 5, 7, 65535} 里别的值 | 墓碑树层级 1 / 2 / 3（各层每个节点每条条目）、层级 3 转过一圈根环；谱系树层级 1 / 2 / 3；inode 树层级 1 三片叶 × 干净 / 删除进行中 / 有墓碑。冻结副本上共 265 形（expected-summary 里另有 F1 副本重跑的 205 形，逐行相同），全部只红目标 |
| 单故障：孩子自己的类标签改成 1 / 2 / 3 / 9 | 墓碑树 T1–T3、谱系树 L1–L2、inode 树 I0。改成 1 的只红目标；改成 2、9 的连带「头用不了」那一套（I-4.8、I-7.2、I-7.4），旧判法，不归 N1 |
| 两故障：条目坏 + 孩子盘 1 那份陈旧 | T1、T2、inode 各一形：目标红，连带与只有陈旧副本的对照相同 |
| N3：合法历史上九条落缺省理由、专门理由说假话、误红 | 6 批 1 830 次判定（旋钮 0 / 1 / 2，有无 reissue 前缀，10 步与 25 步），外加根环转 0 / 10 / 30 次的写死前缀 |
| N5：GPU 与 CPU 判得不一样 | 读码逐支（第二节表），没有一支不同 |
| m3-batch23-code-r1 N1、N3 第二次 | 读码五处（第四节表） |

## 这条腿自己的限度

- 只有一次抽样。「没打中」的几格照 `.claude/rules/three-way-inference.md` 要再抽一次才记。
- H1 只造了「盘 0 那份陈旧、头里的类等于类型段那一类」这一种陈旧法（拷别处一个单元的原样字节）；撕裂写（前半新、后半旧）、两份都陈旧没造。
- 对照「只有陈旧副本」那一行本身连带的 I-3.10、I-4.8、I-7.4 没分析各自对不对（不在这一轮的格里），H1 的判法只拿「多出来的」那几条与目标那一条说事。
- inode 树只有层级 1（core 恢复要 inode 根层级 1），inode 那一支 `InternalNode` 往下走的路径只能靠坏镜像碰到，没造层级 2 的 inode 树。
- 崩溃状态一个都没跑（3.3）。N3 在崩溃状态上的结论只有读码。
- GPU 不跑，第二节与第四节都是读码；F1 的 GPU 那一半是推的。
- 产品入口下的 I-9.21（要 816 条墓碑才有非最右的容器）在 920 次判定里一次都没评估过，这一格的 N3 判法只核了「零根或只一片容器」那句话不假，没造过两片产品容器。
- 我用的起点池与七种用户动作抄自 m3-batch22-code-r1 攻方原型；它造不出的历史（文件很大、多头）这里也没有。

## 没做什么

- 正推那几格（K1、K3）不判；不替主 agent 采纳；F1 只在我的模型上量过、被攻过零轮。
- 没跑 checker 档、GPU、变异表；F1 对变异表锚点的影响是推的。
- 没把 F1 写进 `crates/`；副本上的数不算入库装置上的数。

## 要主 agent 跑的命令（可选）

崩溃状态那一格（3.3）若要补，最小的两段（C2、C3）各 1 048 583 个状态，略过约 10⁶，推的挂钟十几个小时，照派发规则我没跑。要跑的话（工作目录是复跑用的那个、先跑过 `setup`、`crash-build`；包装要的两条理由变量同 `rerun.sh` 里 `export RUN_WITH_MEMORY_CAP_` 那两行；挂钟远过前台四分钟，要放后台）：

```
cd <工作目录>/crash && OPUS_CRASH_HISTORIES=1 OPUS_STATE_CAP=1100000 nice -n 19 bash <仓>/scripts/hosts/run-with-memory-cap.sh 4G bash <仓>/scripts/hosts/capped.sh 10 <工作目录>/target/release/opus-b24-crash
```

看 `RESULT crash C2 … flag …` 那几行：有 `DEFAULT`、`RED`、`NA I-9.24`、`MISMATCH` 任一行就是打中，没有就是这一段零旗。
