# 批 17 阶段同步写回（fix-a）：欠账表三语与 D16 三语

## 结论

六条里五条写了，第 5 条没登：池级 checker 已经核这个区间。18 处替换，9 个文件，`kb-spec-check.py` 是绿的，`replace-batch.py` 回读一致。

| 条 | 怎么处理的 | 改了哪里 |
|---|---|---|
| 1 C560（满环时根槽写坏的恢复前镜像上记账多于遍历） | 改了 | 三语各 1 行 5 处：①「恢复之后的 20 张 × 3 个期望改成全绿」改成「恢复之后的期望全绿」，不写张数；②「实现批在 `crates/mutations.tsv` 新加一行……同 C773 那一行」改成「`crates/mutations.tsv` 第 5414 行……把池级 checker 的 I-7.18 求等放宽成不大于」；③ 第 1020、1360 行与「实现批把这两处改成恰好 I-7.18 红」改成：两种形态恢复之前都恰好 I-7.18 红、I-3.1 与 I-7.19 成立，都取共用函数 `red_only_by_the_reclaim_threshold_after_the_oldest_ring_root_slot_is_written_over`，违例细节里的两个数也钉住；④ GPU 判别力变异那一项改成「记在 C908 ②；CPU 一侧的变异行已在 `crates/mutations.tsv`」；⑤ 去向尾句改成「已随 I-7.18 的实现批落地，GPU 一侧随 C908 并进下一批实现员」。简称没动 |
| 2 C773（读坏与全零根槽的回收门槛钉只在内存里） | 改了，没挪到已还清 | 三语各 1 行 7 处：现象格改成「实现批之前，钉只在内存里、没有门槛字段」，格末补上现状（t 在偏移 428、p 在 436，宽 466，`root_record.rs` 第 124–126 行；`walk.rs` 的 `judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root` 判 I-7.18 与 I-7.19；`released_slots_the_pin_kept_in_defer` 并进钉带）；zero.out 那条回归用例补上「已落地：用例名」；physical_block_size 大于 512 那一格改成由 4096 那条用例钉住、不误红；行号 2908、826 改成 `txgs_ring_slots_bad_at_mount_can_hold`、`genesis_root_ring_slots`；前置格改成「已落地」，并写明还差两样（第 0 代根那处疑点没量；4096 那条用例防的是误红，变异表里没有能让它红的行） |
| 3 C863（钉槽的两类坏实现镜像上判不出） | 改了 | 三语前置格改成「已满足……那一批没做这一条。去向：未排进里程碑，下一批实现员并进」 |
| 4 C909（容器树层级 1 分隔 key 等于容器号只有 core 核） | 改了 | 三语改两处：问题格删掉「没登号」，「欠写无」改成「已指 C909」；前置格改成「已接上这一笔」。改前先查过：三语 D18 第 64 行的欠都已经是 C909 |
| 5 `deleted_inodes` 根的 key 区间 | **没登**，交主 agent 看依据 | 依据见下一节 |
| 6 D16（发布语义） 第 39 行 | 改了 | 三语括注里「实现在系统配置落地补丁合入之后的实现批」改成「已随它的实现批实现：根记录宽 466，t 在偏移 428、p 在偏移 436，池级 checker 判 I-7.18、I-7.19」；三语变更史 D16 节新加 `### 2026-10-26` 块，一项，带依据，改前是 `git show 841b3beab:.claude/kb/decisions/16-发布语义.md` 那一行 |

工作区里已经是新句的：0 处。六条点名的句子在工作区都还是旧句。

## 第 5 条没登的依据（池级 checker 在别处核这个区间）

- `crates/singlefs-checker/src/walk.rs:383` `key_range_reading_of_a_tree_walked_from_its_tree_table_entry`：除了 inode 树、快照 inode 根、墓碑树、谱系树、退役树之外的种类，都返回 `KeyRangeReading::FirstAndLastEntries`（第 391 行），第 382 行的注释点名了「`deleted_inodes` 这类层级 0 的根」。
- 树表条目的走读（第 2867 行取 kind 之后）只对种类 2、4、8、12、3、1 提前 return；种类 9（`TREE_KIND_DELETED_INODES`，第 135 行）往下走到第 3046 行的 `read_index_node(…, key_range_reading_of_a_tree_walked_from_its_tree_table_entry(kind), …)`。`key_schema_for_tree_kind` 对 9 返回 `KEY_SCHEMA_INODE`（`crates/singlefs-checker/src/lib.rs:767`），所以不会在那里退出。
- 第 2243 行 `(KeyRangeReading::FirstAndLastEntries, _)` 调 `check_index_node_keys`（`lib.rs:780`），比较头里的 `smallest_key` / `largest_key` 和首、末两条条目的 key，不等就返回 `KeyOutsideDeclaredRange`，记在 I-1.1 下（第 2269 行 `self.judgements.judge("I-1.1", keys.is_ok(), …)`）。
- 所以这个根的头区间和它唯一那条条目的号不等时，I-1.1 会红。s4 只读了 `deleted_inodes_queue_without_judging`，那个函数确实不判区间，但区间是在通用的树表走读里判的。
- 没查的：有没有一份坏镜像专门把 `deleted_inodes` 根的区间改坏、证明这一判会红；GPU 判器那一侧没看。
- 什么情况会推翻这一条：树表走读在种类 9 走到第 3046 行之前就 return 了（例如另有一支我没看到的分支）；或者 `read_index_node` 在 `WalkedFirstByThisVersion` 之外的路径跳过了这一判，导致每一版都判不到。

## 规格与检查

- 规格：`/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-a/spec.json`（sha256 52fa15ad16bc5e7be54b996e6445242090b9a9bdb3850ad3494a918e7688ec7c），同样内容的 `spec.md`（62df295ec967dbd8fe4ae5a0afb23a786bdf95f2d7259105017adff5361959fd）。由草稿目录里的 `gen/rows.py`（片段表，28fe6134…）和 `gen/build.py` 生成：旧串取原文整行，生成时先核每个片段在那一行里恰好出现一次。
- `kb-spec-check.py` 末行原样：
```
  ✓ 规格 18 条（1 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过（查了 18 条）
```

- `replace-batch.py --dry-run` 原样：
```
✓ 18 处都命中得对（9 个文件），--dry-run 没写
```

- `replace-batch.py` 实写原样：
```
✓ 18 处替换，写了 9 个文件（改名换上新 inode），回读一致
```
- 写范围闸预检：9 个目标路径各喂一次 `write-guard.sh`，全部退 0（检出记录在 `precheck.jsonl`）。
- 变更史前后两次 `git diff`（`hist-diff-before.txt` / `hist-diff-after.txt`）：新增行只有这一批 D16 那一块的 6 行，没有别的条目。
- 第 2、3 步的 `--write`：没跑。派发写的是「并行写回」，由主 agent 在全部交回后跑。这一批也没有翻状态、没有改分项计数，`decisions.md` 前后的哈希相同。`relabel-item.py` 没用到。

## 要主 agent 判的点

1. C773 没挪到已还清。照它自己那一行核判别力：zero.out 回归已落地（`crates/singlefs-harness/tests/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs:11`），`physical_block_size` 大于 512 那一格有 4096 那条用例（同一文件第 48 行），但那条防的是误红，C773 实现员报告第四节证红表写着「没有现成的行」；写者一侧第 0 代根那处疑点（钉在 1 而不是 0）后果没量。`crates/mutations.tsv` 里 C773 的 12 行变异（第 5206–5217 行）证没证红，我没查。主 agent 认为这几样不挡还清的话，再挪表。
2. C560 写了行号「`crates/mutations.tsv` 第 5414 行」，是照派发的「现查行号」写的；这个文件工作区还有没提交的改动（`MM`），行号以后可能会漂。
3. C863 的去向照派发改成「未排进里程碑」，删掉了指向 `.claude/kb/milestone/03-third-txn.md` 二·4 的链接。如果里程碑三那份文件里还把 C863 列在二·4 下，要另外改，那份不在这一批的写范围。
4. 第 5 条没登，依据在前面那一节。要不要补一份坏镜像证明这一判会红，以及 layout/03 与 E172 那几行「key 区间归 C478」改指哪里（s4 乙类），都不在这一批。

## 找不到的目标

0 条。

## 改过的文件与哈希（开工时 → 收尾时）

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/checks-owed.md` | 61dd52f3d7e7ab29d42f82c0a1aa945c46f345e4ef282ee4522716dc9c5b5312 | e89fb321382d8b801cd2cfe995454f169535a2606e5368459642844120b1353b |
| `.claude/kb/checks-owed.en.md` | 593e3eef97a00bf33cf35b9d09e69bbdafbeb6c9ac44c0755bf830ec3b9eaafd | d3bad27dd633a1a9faacf26f58b97edbf86e9ea91e2ae032def553c99f1c9fed |
| `.claude/kb/checks-owed.ja.md` | efafc78e46131219a4bc6f9d7d914dee4637d21b3e3d08e9227ab5c05969e3c8 | 9d6ea3c1e091df0d1ac28fa973254eb58f62d3c57cd6855a63fc282d490ba498 |
| `.claude/kb/decisions/16-发布语义.md` | e9bf0533fab16faf9c0c0b4ed4a7f60420a65826603a8996a2f7f0dbb5b04300 | d552be0cc04ff02dcb5726a211aed3147edc762b2b80d1d89445e332a8314d51 |
| `.claude/kb/decisions/16-publish-semantics.en.md` | cf1007c1aedb79adb6260627b37991dc7d49d00f8038de46c7011e49c1fa95ed | 8cb7911c8e514ec293b44988ff1e2882b6ce7dd1682764ebd5d71807736522be |
| `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md` | 2f433cc07a0591c2d1c335570c9018896a40caecb6812c32978f6a780ff3b2b1 | 5c9f04153c70ddfc095e5ab54039992fcb512409260d262c497bd755c95eff9f |
| `.claude/kb/decisions-history.md` | 74aabc38d9c9b6e946c12e574246238230d6a4e67fafbf15ff7dafd52ac5f858 | ce174807cde32c174182a43dd0bf5c20a8391840cc8b888371bc4403e2e64bd5 |
| `.claude/kb/decisions-history.en.md` | 02c8a517d9dbc08ed3f6974dce2f8c102b8d75b5aa54889b19acc1c743585896 | 629d4e8b991aa373f299309e54ba75fbd4a302f05632e8887f3daa13a7d6a747 |
| `.claude/kb/decisions-history.ja.md` | 467bc79888a801b8f274ba21120c223511487085ae295f0847dd35a7091bdf13 | 74cfde2496a367436c264c58cf19527e8af4279b611a9d990418160f18387034 |
| `.claude/kb/decisions.md` | 9a6453472d0e5988200af5ff94f5d534b384d42c331b91ec51dd93f2437b4f11 | 9a6453472d0e5988200af5ff94f5d534b384d42c331b91ec51dd93f2437b4f11 |

`decisions.md` 前后相同，只是按定义记下来。

## 没做什么

- 没判定案对不对，判决以主 agent 抽查过的回扫报告为准；规格之外的句子一句没加。
- 没跑 `doc-decisions.sh --write --check decision-items-sync`（并行写回，归主 agent），没跑门禁阶段，没提交。
- 第 5 条没登记新号（C916 没用）；没查 GPU 判器有没有判 `deleted_inodes` 根的区间，也没查有没有对应的坏镜像。
- C773 那 12 行变异证没证红没查；C560 说的「21 张还是 20 张」没核，按派发不写张数。
- 草稿目录里只有文本文件，没有仓副本或编译目录。
