# 批 17 阶段同步写回收尾小批（kb-scribe）结果

五条：四条写了，第 2 条照判定不改（依据在下文）。共 24 处替换，15 个文件（三语各 5 组）。`kb-spec-check.py` 绿，`replace-batch.py` 回读一致。没碰决策正文与决策变更史，`decisions.md`、`decisions-history.md` 前后哈希相同。

## 规格

| 文件 | sha256 |
|---|---|
| `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-d/spec.json` | 3daa42ec83fffb0618b31f2f20e67ac485f438cb70b1a6dc73fa4bdfebd6c04c |
| `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-d/spec.md` | ea6d1d8e530610c074456279c97e832b7519a8f1a82ed96c337634d7e384cccd |
| `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-d/replace.json`（给 replace-batch.py 用的，与 spec.json 同内容，去掉了 basis、路径写成绝对路径） | b8ce56ac8e0177211a63b14904dac63956c8aac17fcd9555281b818fac65e778 |

旧串都是原文整行（生成脚本 `gen/build.py` 按唯一子串找到所在行，再把整行当旧串）。

`kb-spec-check.py` 末行原样：

```
  ✓ 规格 24 条（1 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过（查了 24 条）
```

写范围闸预检：15 个目标文件逐个喂 `write-guard.sh`，15 个都退 0。

`replace-batch.py` 原样输出：

```
✓ 24 处都命中得对（15 个文件），--dry-run 没写
✓ 24 处替换，写了 15 个文件（改名换上新 inode），回读一致
```

`git diff -- .claude/kb/decisions-history.md` 写前写后逐字节相同（`cmp` 过），这一轮没往里加东西。

## 每条改了什么

| 条 | 结果 | 改了哪几处 |
|---|---|---|
| 1 `deleted_inodes` 根的 key 区间 | 改了，没登账 | layout/03 三语第 13 行：「池级 checker 核不核这个根的区间见欠账表」改成「这个根的区间由池级 checker 在树表走读里判（`crates/singlefs-checker/src/walk.rs` 的 `read_index_node` 按 `FirstAndLastEntries` 调 `crates/singlefs-checker/src/lib.rs` 的 `check_index_node_keys`，头里的区间与首末条目的 key 不等判 I-1.1（块头自述逻辑地址） 红）」。E172 三语「它答不了的」第 6 条（中文第 164 行）：「池级 checker 核不核 td 的区间见欠账表」改成「td 的区间也由池级 checker 在树表走读里判（同上括注）」。函数名是现查的：`walk.rs` 里 `fn read_index_node`、`KeyRangeReading::FirstAndLastEntries`，`lib.rs` 里 `pub fn check_index_node_keys`，`self.judgements.judge("I-1.1", keys.is_ok(), …)` 也在 `walk.rs`。I-1.1 的简称照登记位 `crates/singlefs-checker/invariants/01-块自描述.md`；英语用 `I-1.1 (block header states its logical address)`，日语用 `I-1.1 (ブロックヘッダは論理アドレスを自己記述する)`，都是 kb 里已有的写法 |
| 2 E172 第 313、356 行 | 不改 | 两行都在「## 历史版本」（中文第 194 行起）下的「### 2026-10-05：建页，第一、二段已跑」里：第 313 行在「#### 第 1 次：各行的读数（整行抄自 …）」节，第 356 行在「#### 第 1 次：它答不了的」节。记的是第 1 次干跑那天的读数和当时的去向，属于事件句，不是今天口径的现状句。现行口径在正文「它答不了的」第 6 条，已经指 D18（块里携带什么信息） 已定项 2，第 1 条这一轮又改成了指判法。插了一行表格之后，这两行现在是第 314、357 行 |
| 3 E172「影响的决策」补行 | 改了 | 三语各在 D18（块里携带什么信息） 已定项 11 那一行后面插一行。中文：`\| D18（块里携带什么信息） 已定项 2 \| 不影响 \| 2026-10-26 不受影响：正文只引它当 key 区间的口径出处，结论与产物不变 \|`。英语关系写 `Does not affect`，日语写 `影響なし`，都是实验页里已有的译法。表里原来没有 D18 别的分项写这件事；其余各行的回看早就写到 2026-10-26，不用动 |
| 4 layout/01 世代号与取号出处 | 改了 | 三语三处。①「自举头 槽世代号」行（中文第 119 行）：`mkfs 1，取号 2，暖机 3、4，新池新建文件发布后 5` 改成 `mkfs 1，取号 2、3，暖机 4、5，新池新建文件发布后 6`。②「系统配置槽轮换写」行（中文第 398 行）：改成「根槽之后写槽 0，世代号 6，tail = 3（取号两轮：槽 0 世代 2、槽 1 世代 3；暖机两次各写一次：槽 0 世代 4、槽 1 世代 5）」。依据是 milestone/01 第 161 行，加上 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 的 `root_slots_system_configurations_and_journal_ring_hold_the_published_state`：它断言选中的是槽 0、世代 6、tail 3，槽 1 是世代 5，注释写「暖机第二次写的槽 1」。暖机第一次写槽 0、世代 4 是照轮换写旧的那一槽推出来的，测试没直接断言。③ 取号行（中文第 413 行）出处格改成：「两轮的 `2+2` 由 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 的 `recorded_paths_match_the_registered_segment_sequences` 钉住；产物 `name=segments path=instance_acquisition`（`research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out`）是取号写一轮时的，`segments=2`」。用例断言取号是 `(6, "2+2", 7)`，产物里那一行原样是 `E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]` |
| 5 milestone/03 二·4 | 改了 | 三语第 71 行：「（与 C773（读坏与全零根槽的回收门槛钉只在内存里） 同做）」改成「（C773（读坏与全零根槽的回收门槛钉只在内存里） 那一批已落地、没做它；未排进里程碑，下一批实现员并进）」。照的是欠账表 C863 那一行去向的原文 |
| 出口 experiments-history | 改了 | 三语 E172 节「### 2026-10-26」块里，在丙那一项之后补了一项：第 6 条原句与新句、不另登欠账、补 D18 已定项 2 一行（不影响）、结论数产物不变；**依据** 写主 agent 批 17 收段判定，加 `walk.rs` 的 `key_range_reading_of_a_tree_walked_from_its_tree_table_entry` 对种类 9 返回 `FirstAndLastEntries`；**改前** 写「工作区那一版（同日改第 6 条指路那一项之后，还没进 git），原句照录在项里」，因为改前那一版不在 git 里 |

## 要主 agent 判的点（不挡这一批，都没改）

1. layout/01 三语的单元表里，还有几行是取号写一轮那一版的世代号，与这一轮改的第 119、398 行对不上。中文第 62 行 a1 只写「系统配置槽 0，世代号 2」，没有第二轮的槽 1 世代 3；第 65 行 w3 写「系统配置槽 1，世代号 3，tail = 1」；第 68 行 w6 写「系统配置槽 0，世代号 4，tail = 2」；第 83 行 t15 写「系统配置槽 1，世代号 5，tail = 3」。照两轮应当是 w3 槽 0 世代 4、w6 槽 1 世代 5、t15 槽 0 世代 6。派发只点了 119、398 附近，所以这几行没动。英语对应行（t15 在第 83 行）、日语对应行（t15 在第 83 行）同样是旧值。要不要一并改，由主 agent 定。
2. layout/01 中文第 406 行（段序列登记表上面那段说明）还写着各行的段序列「逐字取自 E142（新池新建文件的干跑） 产物的 `name=segments` 行（`path=instance_acquisition / warm_up / transaction / post_mkfs_stream`…）」。取号那一行的产物是一轮时的，这句对取号行不成立，与第 413 行这一轮写进去的说法对不上。没改。
3. 新句里给 key 区间判红记的不变量是 I-1.1（块头自述逻辑地址），因为代码就是记在这个号下的（`walk.rs` 里 `judge("I-1.1", keys.is_ok(), …)`）。但这个简称说的是逻辑地址，不是 key 区间，读者看了可能觉得对不上。名字有没有必要改，或者另起一个不变量，不是写回员能判的。
4. 第 1 条依据报告里写的「没查的」那两样，这一轮也没查：有没有专门把 `deleted_inodes` 根的区间改坏、证明会红的坏镜像；GPU 判器那一侧判不判这个区间。

## 找不到的目标

0 条。

## 改过的文件与前后 sha256

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| .claude/kb/layout/03-third-txn.md | a726a5900cd5d6a2… | 9824eff738cdd1d1… |
| .claude/kb/layout/03-third-txn.en.md | ff320b0dafd00fe2… | e17bec1f5ac0bb06… |
| .claude/kb/layout/03-third-txn.ja.md | 54ec533c160d04e5… | 3c8ae8605c923769… |
| .claude/kb/experiments/172-删除写出的字节的干跑.md | 4c30691aaf5717df… | 3a9bdf9f16b55737… |
| .claude/kb/experiments/172-dry-run-of-bytes-written-by-deletion.en.md | 142b319893a761b5… | f0391c2b8c330617… |
| .claude/kb/experiments/172-削除で書き出されるバイトのドライラン.ja.md | a0a757d24e380357… | acd5bfd34a268217… |
| .claude/kb/layout/01-first-txn.md | 63a1dbe4c64a597a… | 9bb15e4acad3de15… |
| .claude/kb/layout/01-first-txn.en.md | 9eedcf679302b9ed… | 2a9a14b5794df820… |
| .claude/kb/layout/01-first-txn.ja.md | 62a30cff60a05c3d… | a3e0b538906eb0a2… |
| .claude/kb/milestone/03-third-txn.md | 84e87251a713be57… | 6444fc1af2d9dfa6… |
| .claude/kb/milestone/03-third-txn.en.md | e71a140811d0a2eb… | c432847e0699bce2… |
| .claude/kb/milestone/03-third-txn.ja.md | 286667302e4e412b… | 0027b003f4c548d7… |
| .claude/kb/experiments-history.md | d5d16c7be3903166… | 7294434d3a1f779e… |
| .claude/kb/experiments-history.en.md | 1937d29b846d2a0d… | 6f5ab8dc49f6183a… |
| .claude/kb/experiments-history.ja.md | 77542e263f927911… | e6711483e4841de6… |
| .claude/kb/decisions.md（没改） | 9a6453472d0e5988… | 9a6453472d0e5988… |
| .claude/kb/decisions-history.md（没改） | ce174807cde32c17… | ce174807cde32c17… |

完整的 64 位哈希在 `sha-start.txt`、`sha-end.txt`（都在草稿目录里）。开工时这些文件都已经带着别的会话还没提交的改动（`git status` 里是 MM / AM），开工哈希按当时的工作区记。

## 什么现象会推翻

- 第 1 条：树表走读在种类 9 走到 `read_index_node` 之前就返回了，或者 `key_range_reading_of_a_tree_walked_from_its_tree_table_entry` 对种类 9 返回的不是 `FirstAndLastEntries`，那么新句说的「由池级 checker 判」就不成立。
- 第 4 条：取号写回一轮，或者暖机第一次写的不是槽 0，第 398 行括号里的槽号就要跟着改。

## 没做什么

- 没跑 `doc-decisions.sh --write --check decision-items-sync`（派发写明交回后由主 agent 跑）。决策状态与分项都没动。
- 没跑任何门禁阶段与 lint，三语六格、decision-links 格都没跑，留给提交前整轮。
- 没判定案对不对。第 1 条的判法照主 agent 的判定写，我只现查了函数名还在。
- 没改「要主 agent 判的点」里那几行，没提交。
- 草稿目录里只有规格、生成脚本与几份文本，没有仓副本，也没有编译目录。
