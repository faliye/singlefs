# 核查员报告：m2-refusals-presumed-r1

**这是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

## 输入缺口

派发提示未给草稿目录路径。已自建 `/tmp/claude-1000/m2-refusals-presumed-r1-verifier/`（`repo/` 为今日主树的
`rsync -a --exclude target --exclude .git` 副本，`draft/` 放辅助产物），仅在此目录与本报告文件内写入。
云端腿交回里未给报告文件的 sha256sum，本轮无法做「报告文件现在的 sha256 与交回里给的对不上」这一步。

## 判别力自证

取 Sonnet 报告 P4 一节对 `.claude/kb/decisions/03-空间分配.md:195` 的引用（原文「bump 指针只住内存，挂载后新开一段」）。
把该文件复制进草稿目录（`sha256sum` 与仓内、与开工快照 `start-2.sha256` 三方一致：
`5d30efd2e0c09db399f1342e27d5a2eba8e826de333b66dd67b8643448160ab4`），行号加 1 改核第 196 行：

```
$ sed -n '196p' 草稿副本/03-空间分配.md
（空行）
```

第 196 行是空行，不含被引原文 ⇒ 判 ✗。核查方法能分辨行号错位，判别力自证通过。

## 关于本轮多份 crates/ 源码同时被并行会话改写的说明（贯穿全篇的背景事实）

开工快照 `start-2.sha256`（2026-09-22 13:45 重拍）只含 15 个文件的哈希、不含内容本身。核对时按以下顺序处理：

| 文件 | 现状（本次核对时刻，约 22:35 UTC）与快照的关系 |
|---|---|
| 全部 7 份 `.claude/kb/decisions/*.md`（16/18/22/23/03/02/28） | 与快照哈希**逐字节相同**，可直接对主树核 |
| `crates/singlefs-core/src/make_filesystem.rs` | 与快照哈希**相同** |
| `crates/singlefs-format/src/lib.rs` | `git status` 干净（今日未改），视同稳定 |
| `crates/singlefs-core/src/mount.rs` | 与快照**不同**；`stat` 显示最后一次改动在 22:14:33 UTC——**早于**这一步核对、**晚于**云端两条腿交报告的时刻。已用 Opus model 目录里 `mount-rs-opus-fixes.patch` 对 Opus 自己 rsync 副本里已打过补丁的 `mount.rs` 做 `patch -R`，回收出补丁前的原文，`sha256sum` 与快照的 `mount.rs` 哈希**完全一致**（`1bef6b8d…`）。以下用这份「回收快照」核 Opus 与 Sonnet 对 `mount.rs` 的引用 |
| `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/recovery.rs` | 与快照**不同**（`git diff --stat` 显示分别改动 1255 行、554 行）。Opus 自己 rsync 副本里（打补丁前）这两个文件的哈希与快照**完全一致**，说明这两个文件在快照时刻（13:45）到 Opus 起跑复制仓（约 21:5x）之间没有改过；Opus 报告自己交代「recovery.rs（21:56 UTC）与 transaction.rs（21:57 UTC）在这一轮中途被别的会话改过」，并说明report 里这两个文件的引用行号是之后重新现查所得，不是对着快照抄的 |

⇒ 下面核对 `mount.rs` 引用一律对「回收快照」核；核对 `transaction.rs`／`recovery.rs` 引用**两边都核**（对回收快照、对当前主树），
两边都写出结果，不各打一半。

## 特别要核之一：云端攻方五条「现查观测」中三条「正文与 kb 已腐」，独立复核今天状态

全部对**当前主树**核（这就是本条要问的问题本身：「今天」的状态），非对快照核。

| 观测 | 复核命令 | 结果 |
|---|---|---|
| `format_time_allocator` 的断言已改成错误成员 `FormatTimeUnitLocationsOnDifferentSlots` | `grep -n "FormatTimeUnitLocationsOnDifferentSlots" crates/singlefs-core/src/mount.rs`；`awk '/fn format_time_allocator/,/^}/' … \| grep -n "assert_eq\|两盘同槽"` | ✓ 成员声明在 `mount.rs:82`，使用在 `:499`；函数体内 `assert_eq!`/「两盘同槽」**零命中**，确认原 panic 已改成这个错误成员 |
| `MountError` 今天共 18 个成员 | `awk '/^pub enum MountError/,/^}/' crates/singlefs-core/src/mount.rs \| grep -cE "^    [A-Z][A-Za-z]*(\(|,\| \{)"` | ✓ 命令输出 `18`，与 Opus 报告一致 |
| `RaiseNeedsWritableMountInThisProcess` 在 `crates/` 里零命中 | `grep -rn "RaiseNeedsWritableMountInThisProcess" crates/ \| wc -l` | ✓ 输出 `0` |
| 用例名 `raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking` 在 `crates/` 里零命中 | 同上加 `grep -rn` 该用例名 | ✓ 输出 `0`（这个名字与实现里真正的用例名 `raising_the_floor_on_a_current_version_without_a_rewritten_instance_table_unit_is_refused_instead_of_panicking` 不同） |
| `.claude/kb/milestone/02-second-txn.md` 今天是否仍写着 `RaiseNeedsWritableMountInThisProcess` | `grep -n "RaiseNeedsWritableMountInThisProcess" .claude/kb/milestone/02-second-txn.md` | ✓ 零命中——该文件今天已经改写，同一处（行 195）现在写的是「没做过可写挂载的进程抬 F 报 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`（`raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking`）」，与实现一致，与主 agent 已现查坐实的说法一致 |
| `.claude/kb/checks-owed.md` 的 C484 今天写的是什么 | `grep -n "C484" .claude/kb/checks-owed.md` | ✓ 今天该行以「**已还（2026-09-22，用户当天定「连根环一起清」）**」开头，后接 `make_filesystem.rs` 按几何逐区域清根环、`first_transaction_step_one_mkfs.rs` 的断言与三条变异——与派发提示所说「主 agent 已改成『已还』」一致 |

三条今天的状态**全部与 Opus 报告的判断、以及主 agent 的现查坐实相符**：正文与 kb 确实已经按 Opus 指出的方向改过，
且改动内容与 Opus 报告里描述的「今天应该是什么」逐字对得上。

## 特别要核之二：云端攻方五条探针，今天工作区上独立重跑

Opus 报告的 R1/R2/R3/Q1 四条探针（`opus_attack_refusals_r1r2r3q1.rs`，5 个 `#[test]`）与它自己提的两个改法
（`mount-rs-opus-fixes.patch` + `opus_patched_refusals.rs`）**在核查员自己的仓副本上重跑**（`/tmp/claude-1000/m2-refusals-presumed-r1-verifier/repo`，
今天当前主树的副本，`rsync --exclude target --exclude .git`；为省编译时间借用了 Opus 自己副本里已经编译过一次的 `target/`，
源文件本身取自今天的主树，不是 Opus 的副本）。全部命令加 `nice -n 19`，跑在核查员自己的目录，不在 Opus 的原目录里跑。

### 未打补丁（对今天的主树，含 transaction.rs/recovery.rs 今天的大改动）

```
cd /tmp/claude-1000/m2-refusals-presumed-r1-verifier/repo
nice -n 19 cargo test -p singlefs-harness --test opus_attack_refusals_r1r2r3q1 -- --nocapture --test-threads=1
```

结果：`test result: ok. 5 passed; 0 failed`。逐条比对 Opus 报告里贴出的原样输出行（`[R1]`/`[R2]`/`[R3]`/`[Q1]`/`[R4]` 开头）
与 Opus 交回的 `baseline-run.log`：

```
$ diff <(grep -E "^\[R1\]|^\[R2\]|^\[R3\]|^\[Q1\]|^\[R4\]" research/prompts/m2-refusals-presumed-r1-opus-model/baseline-run.log) \
       <(grep -E "^\[R1\]|^\[R2\]|^\[R3\]|^\[Q1\]|^\[R4\]" 核查员重跑日志)
（无输出，两边 17 行逐字相同）
```

**结果：成立。** R1-α（零故障，可写挂载一次 → 正常退出 → 再挂载被拒）、R1 之后环里每条可读根都被拒（R2 的三条）、
R2 的独立场景（回退到暖机根被拒）、R3（抹光记录后可写挂载被拒但冷启动只读仍读得回文件）、Q1（同进程内多次覆盖写后抬 F 仍被拒）
——五条读数在今天（含 transaction.rs/recovery.rs 已被别的并行会话大改之后）**逐字节重现**。

### 打补丁（Opus 自己提的两个改法，被攻过零轮）

```
patch -p0 crates/singlefs-core/src/mount.rs < .../mount-rs-opus-fixes.patch   # 今天主树上 fuzzy 1 行偏移，成功
cp .../opus_patched_refusals.rs crates/singlefs-harness/tests/
nice -n 19 cargo test -p singlefs-harness --test opus_patched_refusals -- --nocapture --test-threads=1
OPUS_SKIP_R1=1 nice -n 19 cargo test … opus_r1_fix
OPUS_SKIP_R1=1 OPUS_SKIP_R4=1 nice -n 19 cargo test … opus_r1_fix
```

| 跑法 | 核查员重跑（今天主树） | Opus 报告 | 一致？ |
|---|---|---|---|
| 打补丁不设环境变量 | `[Q1-F1] 抬 F 到 3 做成了：上限 3，空发布 2 次（txg [7, 8]），回收 1 个落点` | 同一句，逐字 | ✓ |
| `OPUS_SKIP_R1=1` | `[R1-F1] 第二次可写挂载还是被拒：FormattedPoolMountNotShapedLikeTheFirstTransaction { chosen_root: (1,2), first_txg: CheckpointTxg(3), first_counter: 3 }` | 同一句，逐字 | ✓ |
| `OPUS_SKIP_R1=1 OPUS_SKIP_R4=1` | `[R1-F1] 挂载做成了：实例 2，写行 0 条，写行那次 txg 3，暖机 1 次（txg [4]）` 接着 `FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record: Some((CheckpointTxg(4), 4)) }` | 同一句，逐字 | ✓ |

`patch -p0 --dry-run` 显示两处 hunk 都以「offset 1 line」成功套用（与 mount.rs 今天比快照多 1 行一致），
未见任何一条探针在今天的主树上读数不一致。

补充核对 Opus 模型目录 7 个新文件的 `sha256sum`：`sha256sum opus_attack_refusals_r1r2r3q1.rs opus_patched_refusals.rs
mount-rs-opus-fixes.patch baseline-run.log fix-run-plain.log fix-run-skipr1.log fix-run-skipr1r4.log` 与 Opus 报告第零节列出的
7 个哈希**逐一相同**。

## 特别要核之三：R2「条文与实现说反话」，独立核原文与实现

**条文侧**：`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 整段（第 340–394 行，与快照哈希一致）：

```
$ grep -n "^#### 已定项 14" .claude/kb/decisions/23-journal的角色与格式.md   # 340
$ grep -n "^#### 已定项 15" .claude/kb/decisions/23-journal的角色与格式.md   # 395
$ awk '/^#### 已定项 14/{f=1} f&&/^#### 已定项 15/{exit} f' 同文件 | grep -c "树表"
0
```

已定项 14 整段（含「显式例外：管理员回退」候选集定义那一句、含它的注 1–3）**零处提到「树表」**，
Opus 与 Sonnet 两条腿各自独立抄出的候选集原文——「候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效
（D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）」
（第 363 行）——逐字节相同，且与仓里原文逐字节相同（本报告独立 `sed -n '363p'` 核对）。**条文侧核实：候选集定义确实
只有「实例表判有效」「txg ≥ F_生效」两条，没有「树表非 0 条」这一条。**

**实现侧**：对「回收快照」（等同快照，`mount.rs` 未在 Opus/Sonnet 报告时刻之前改动过）核：

```
$ sed -n '1297,1301p' 回收快照/mount.rs
    // 回退到树表 0 条的根……在任何写之前拒绝，报「第一版不支持」，不报候选排除。
    if tree_table_has_no_entries(&*devices, &target_root)? {
        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));
    }
```

`if` 判定本身在快照版 `mount.rs` 第 1299 行——**与 Opus 报告「判定点 `mount.rs:1299`」逐字对得上**（先前一度误对
当前主树核，当前主树因今天 22:14 UTC 之后一次改动比快照多出 1 行、该判定点挪到 1300 行；对回收快照核才是对的口径，
两边行号差 1 与「今天比快照多 1 行」这一事实一致，不算 Opus 引用出错）。错误成员名 `RollbackToVersionWithoutFileUnsupported`
——用的正是「树表 0 条」这条**候选集条文里没有的理由**。

**结论：R2 的「条文与实现说反话」这一观测，本报告独立复核成立**——candidate-set 字面文本与今天实现拒绝的判据确实不是同一条件，
条文允许 (1,1)/(1,2) 这类目标（树表 0 条的暖机根）进候选集，实现却单独用 `tree_table_has_no_entries` 再拒一次。


## 特别要核之四：本地攻方两份样本，R5–R8 四格 × 四列逐格比对

样本 s1（39 行，长格式，每格附一句 refuted-by）与 s2（19 行，紧凑格式）解析后按「①有没有写过盘 / ②零故障可达 /
③错误名对不对 / ④写条款·登记不支持·已覆盖」四列、R5–R8（对应 Judgment 1–4）四格逐格比对，**只报一致与不一致，不判哪份对**：

| 格 | 列 | s1 | s2 | 一致？ |
|---|---|---|---|---|
| R5 | ① | no（fact 4） | no（fact 4） | 一致 |
| R5 | ② | no（fact 5） | no（fact 5） | 一致 |
| R5 | ③ | yes | yes（fact 4） | 一致 |
| R5 | ④ | new decision clause | should be written as a new decision clause | 一致 |
| R6 | ① | no（fact 7） | no（fact 7） | 一致 |
| R6 | ② | no（fact 8） | no（fact 8） | 一致 |
| R6 | ③ | yes | yes（facts 8, 9） | 一致 |
| R6 | ④ | already fully covered，引 fact 10 | already fully covered，引 facts 10, 11 | 一致（引用集小有出入，见下） |
| R7 | ① | no（fact 14） | no（fact 14） | 一致 |
| R7 | ② | yes | yes（facts 14, 16） | 一致 |
| R7 | ③ | yes | yes（fact 14） | 一致 |
| R7 | ④ | already fully covered，引 fact 15 | already fully covered，引 facts 15, 2 | 一致（引用集小有出入，见下） |
| R8 | ① | no（fact 18） | no（fact 18） | 一致 |
| R8 | ② | no（fact 18） | no（fact 18） | 一致 |
| R8 | ③ | yes | yes（fact 18） | 一致 |
| R8 | ④ | register as an unsupported case | should be registered as an unsupported case | 一致 |

**16 格（4 判定 × 4 列）逐格全部一致**，两份样本的答案值没有一处方向相反。两处「引用集小有出入」（R6④ s2 多引
fact 11、R7④ s2 多引 fact 2）指的是支撑同一个判词多引了一个事实编号，不是判词本身不同，因此仍记「一致」而不是「不一致」。

（复跑核验：`corruption-check.py`／`oov-check.py` 对两份样本重新跑过一遍，退出码与统计数字——s1 `cjk=0 words=349 fffd=0`
`生词=4`，s2 `cjk=0 words=279 fffd=0` `生词=3`——与运行记录逐字相同，两份都是退出码 0。）


## 特别要核之五：正推腿（Sonnet）P1–P7 抄的原文，逐条核对

方法：`quote-kb.py` 按行区间抽取当前主树（与快照对全部涉及的 7 份决策文件逐字节一致，见前文表格）里的原文，
去掉报告里 `> ` 引用前缀后与抽取结果 `diff`；差 0 行记 ✓ 整段一致，有差记具体差在哪。

| 引用 | 核的结果 | 差在哪 |
|---|---|---|
| P1：`03-空间分配.md:193-220`（D3 已定项 10 整段） | ✓ 逐字节一致（`diff` 空输出，26 行） | — |
| P1：`transaction.rs:1284-1300`（`rewritten_roles`，现查代码今日主树） | ✓ 主树 1284 行确为 `pub fn rewritten_roles`，`InstanceTable` 确实 push 在 `ExtentRoot` 之前 | — |
| P2：`16-发布语义.md:205-218`（D16 已定项 9） | **✗ 不完整**：范围声称「整段」，但报告只抄了「定案」（207 行）与「射程」（209 行）两段，**漏了「依据」（211–215 行：E148/E142/用户定案）与「欠」（217 行：C363）两个子节**——它们同在 205–218 这个引用区间内 | 依据、欠两节整段缺失 |
| P2：`28-挂载期承诺量.md:89-106`（D28 已定项 4） | ✓ 逐字节一致（定案/形态/归属/射程/依据/欠六个子节全部抄到，`diff` 空输出） | — |
| P2：`18-块里携带什么信息.md:306`（声称是「行怎么写」bullet） | **✗ 行号错**：引用文字本身与源文件第 **304** 行逐字节相同（`行怎么写`一条），但第 306 行实际是**另一条**bullet「回退写的行」——文字对、行号差 2 | 应为 304，非 306 |
| P2：`transaction.rs:1263-1274`（`ROW_PUBLISH`/`EMPTY_PUBLISH`） | ✓ 对「回收快照」逐字节一致；当前主树因今天并行线一改动整体挪了 58 行（现在在 1321/1329），核对时点使用回收快照 | — |
| P3：`16-发布语义.md:185-204`（D16 已定项 8 整段） | ✓ 逐字节一致（定案/射程/四条已知边角/依据/欠全部抄到，`diff` 空输出） | — |
| P4：`03-空间分配.md:193-220`（同 P1 引文） | ✓ 同上 | — |
| P4：`allocator.rs:636,644-648`（`PoolAllocator::new`） | ✓ 主树逐字节一致 | — |
| P4：`mount.rs:406-421,511`（`rebuilt_allocator`/`format_time_allocator` 调用点） | ✓ 对回收快照，511 行确为 `let mut allocator = PoolAllocator::new(device_maps);`；当前主树该行挪到 512（今天多 1 行，与前述一致） | — |
| P5：「干净关闭」在 `.claude/kb/decisions/` 下零命中 | ✓ `grep -rln` 复核确认零命中，唯一命中在 `milestone/02-second-txn.md`（现状记录，非已定项） | — |
| P6：`23-journal的角色与格式.md:358`（已定项 14 注 1） | ✓ 逐字节一致 | — |
| P6：`recovery.rs:1083-1093` | ✓ 对回收快照逐字节一致；当前主树该段落挪到约 1088 行起（recovery.rs 今天另有独立的行号漂移，非 transaction.rs 那次 58 行漂移） | — |
| P7：`23-journal的角色与格式.md:363`（候选集定义，同 R2 引文） | ✓ 逐字节一致（与 Opus 引用互证） | — |
| P7：`18-块里携带什么信息.md:306`（「回退写的行」bullet，P7 自己这处引用） | ✓ 行号与文字都对（这处引的确实是 306 行，与 P2 那处 306 弄混不是一回事——P7 自己引对了） | — |
| P7：`23-journal的角色与格式.md:376`（「射程」⚠️ 段） | ✓ 内容逐字节一致，仅比源文件少了行首的列表符 `- `（不影响任何字词） | 无实质差异 |

**这一份小计：核了 15 处引用（含 3 处代码引用），13 处 ✓，2 处 ✗**（P2 的 D16 已定项 9 不完整、P2 的 D18 行号错）。


## 云端攻方（Opus）引用核对表

对 `mount.rs` 引用用回收快照核；对 `transaction.rs`/`recovery.rs` 引用先对回收快照核，不中再对当前主树核（见前文说明）。

| 引用 | 核的结果 | 命令/差在哪 |
|---|---|---|
| `mount.rs` 六个错误成员声明行（38/44/72/128/65/57） | ✓ 全部对回收快照逐字节一致 | `sed -n '38p;44p;72p;128p;65p;57p' 回收快照/mount.rs` |
| `mount.rs:1054`（R1 判定点，取号 `:1063` 之前） | ✓ 对回收快照一致，1054 行确为 `refuse_instance_rows_on_version_without_file(...)`，1063 为 `acquire_expected_instance(...)` | — |
| `mount.rs:1055`（R4 判定点） | ✓ 一致 | — |
| `mount.rs:1299`（R2 判定点） | ✓ 对回收快照一致（`if tree_table_has_no_entries` 本身在 1299 行）——先前一度误对当前主树核判成差 1，已改按回收快照复核，见前文「特别要核之三」 | — |
| `mount.rs:660`（`raise_rollback_floor` 「第二句」） | ✓ 一致，`.ok_or(MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)?;` | — |
| `mount.rs:1191/:1218`（`rebuild_previous_version`/`establish_instance` 调用序） | ✓ 一致 | — |
| `recovery.rs:658-662`（树表 0 条先返回、走不到记录判断） | ✓ 对回收快照与当前主树**都**一致（这段今天未挪位） | — |
| `recovery.rs:889-893`（`TransactionOutput { record, record_bytes, … }`） | ✓ 对**当前主树**一致（`Ok(RebuiltVersion::WithFile(TransactionOutput {` 恰在第 889 行，`record,`/`record_bytes,` 在 891/892，都落在引用区间内）；对回收快照核不中（该处在快照里是 886/887 行） | 现查证据见下方命令 |
| `recovery.rs:907-910`（注释「这条记录上的事务号不会被拿去接着发布」，转述非逐字引） | ✓ 对**当前主树**内容相符（906–908 行确有「这个值不会被拿去接着发布」「单独拿它续号会重号」）；这是转述不是逐字抄，未按摘句判据核 | — |
| `transaction.rs:1149/1469/1490`（`FirstFileVersionNotRightAfterTheSecondWarmUp` 三处） | ✓ 对**当前主树**逐一精确命中；对回收快照核不中（快照里在 1091/1411/1432，差 58 行） | `grep -n` 见下 |
| `transaction.rs:1480/1487`（`FIRST_TRANSACTION_TXG` 两处判定） | ✓ 对**当前主树**逐一精确命中（同一份 `grep -n` 输出） | — |
| `18-块里携带什么信息.md:309`（「无行 ⇒ 已发布」短语） | ✓ 是这行里的一个子句，逐字节一致 | — |
| `16-发布语义.md:39`（「准入」行，逐字引用带「……」省略号） | **部分**：被「……」省略掉的括注（「写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC」）确实存在于原文同一行内，省略号是明示的（不是无声摘句），但按纪律「连同它的括注」不许省 | `sed -n '39p' 16-发布语义.md` |
| `22-单元原子性怎么合成.md:330`（「已定项 16 第 5 句」） | **✗ 不完整**：引用只抄了该编号条目的第一句（「暖机空发布 2 次……登记在 layout/01-first-txn.md 八）。」），**该条目同一行内还有第二句（D16 已定项 8 与归属 0/1/0 的解释）与第三句（⚠️ 归属改了这两个常量要重算），后者带 ⚠️ 标记，按纪律「连同它的 ⚠️」不许省，这里被省掉了** | `sed -n '330p' 22-单元原子性怎么合成.md` |
| `03-空间分配.md:175`（D3 已定项 9 标题引用） | ✓ 只引标题、未声称引全文，一致 | — |
| `checks-owed.md:427`（C482 第②条子串「crates/*/src/ 里零个产品路径调用 raise_rollback_floor」） | ✓ 是该行内一个子串，逐字节一致 | — |
| `crates/*/src/` 里 `raise_rollback_floor` 命中点「只有 allocator.rs:302 一句文档注释和 harness（history.rs:2661）」 | **✗ 行号错**：第 2661 行内容是 `let stream_length_before = stream.operation_count();`，**不含**该字符串；实际命中在 `history.rs` 的 37/302/2630/2662/2664/2804 行（`model.rs` 另有 5 处），2662 行是另一个函数名 `answer_raise_rollback_floor` 的子串命中，真正的调用点在 2664 行 | `grep -rn "raise_rollback_floor" crates/*/src/` |
| `second_transaction_step_five_reuse.rs:615`、`second_transaction_step_three_formatted_pool.rs:115`、`allocator.rs:302`、`singlefs-format/src/lib.rs:139,162` | ✓ 全部对当前主树逐字节一致（这几个文件不在快照清单内，但今天要么未改、要么改动不影响这几行） | — |

**这一份小计：核了 19 处引用，16 处 ✓，1 处「部分」（省略号已标明但仍算摘句），2 处 ✗**（D22 已定项 16 漏 ⚠️ 那句、
`history.rs` 命中行号错）。此外五条探针的 5+3 次复跑（见前文「特别要核之二」）全部成立，另计入总数。


## 本地攻方转述核对表抽查（对着「原文文件:行」逐条核，抽查非全量）

| 项 | 核的结果 |
|---|---|
| Fact 1：`_background.md:22-26`（拒绝三关） | ✓ 逐字节一致 |
| Fact 2：`_background.md:92`（该写条款判据） | ✓ 逐字节一致 |
| Judgment 1–4：`_background.md:82`（本地攻方分工行） | ✓ 逐字节一致 |
| Fact 3/6/13/17：`_background.md:53/54/55/56` 第二列（R5–R8 题面来源） | ✓ 四行全部逐字节一致 |
| Fact 4：`transaction.rs:1481-1493`（`publish_first_file` 控制流） | ✓ 对当前主树逐字节一致 |
| Fact 7：`mount.rs:1035-1076`（`establish_instance`） | ✓ 内容对回收快照逐字节一致；引用的 `1051`（`instance_generation_to_acquire` 调用）、`1063`（`acquire_expected_instance` 调用）两个行号是回收快照的行号，当前主树上这两行分别在 1052、1064（差 1，与「今天 mount.rs 比快照多 1 行」一致，非引用出错） |
| Fact 8：`transaction.rs:392-393` | ✓ 该区域今天与快照无漂移，两版本行号一致 |
| Fact 10：`checks-owed.md:445`（`### 已还清` 标题）、`:492`（C322 行） | ✓ 两行都逐字节一致 |
| Fact 14：`make_filesystem.rs:175-179,197,54-58` | ✓ 逐字节一致，附带的全仓 `grep -n "region_devices:"` 命中清单核对无误（6 处命中，全部字面赋值） |
| 转述核对表自称「不写代码行号」的提示本身 | ✓ 复核 `m2-refusals-presumed-r1-local-attack.md` 全文未出现文件名+行号组合，符合派发要求 |

抽查未见「把背景材料自己的行号误标成源文件/kb 行号」这一类失误——本轮三条腿现查行号时都明确区分了
「背景材料的行」与「源文件/kb 自己的行」，各处标注也确实对应各自文件（本报告前几节逐条核对已覆盖大部分交叉验证）。

## 汇总计数

| 报告 | 核了 | ✓ | ✗ | 核不动/分不清 |
|---|---|---|---|---|
| 判别力自证 | 1 | — | 1（要求判 ✗，判成了） | 0 |
| 三条「今天状态」现查 | 6 | 6 | 0 | 0 |
| Opus 五条探针复跑（含改法 3 变体） | 5（未打补丁）+3（打补丁三变体） | 8 | 0 | 0 |
| R2 条文/实现独立核对 | 2（条文候选集、实现判定点） | 2 | 0 | 0 |
| 本地攻方 s1/s2 一致性 | 16（格×列） | 16（全部一致） | 0（本节不判对错，只判一致性） | 0 |
| Sonnet P1–P7 引用 | 15 | 13 | 2 | 0 |
| Opus 引用（含 sha256、探针输出比对） | 19 + 7（sha256） | 23 | 3（含 1 处「部分」计入✗） | 0 |
| 本地攻方转述核对表抽查 | 10 | 10 | 0 | 0（抽查非全量，未覆盖 Fact 5/6/9/11/12/15/16/19/20 等其余各条） |

**合计**：核了约 81 处（含格×列计数与复跑次数），✓ 约 74 处，✗ 约 6 处（Sonnet 2 处 + Opus 3 处 + 判别力自证的 1 处按设计判✗）；
无「核不动」或「分不清：文件在腿交回之后被改过」的条目——本轮虽有三份 `crates/` 源码在腿交报告前后被并行会话大幅改写，
但借助 Opus 自己 rsync 副本（改动前的冻结状态，经哈希核实与快照一致）与 `patch -R` 回收，全部涉及行号的引用都找到了
可比对的确切版本，未出现真正判不清的情况。

## 没做什么

- 不判一条打中成不成立、该不该采纳；R1–R8/Q1/P1–P7 各格的推论本身（例如「这道拒绝算不算误拒」「这一格该不该写条款」）
  一律不判，只核引用、产物与复跑。
- 本地攻方转述核对表 20 条 Fact 中只抽查了 10 条（Fact 1/2/3/4/6/7/8/10/13/14 与 Judgment 1–4 指路行），
  Fact 5/9/11/12/15/16/19/20 未逐条现查（多数是「无对应中文原句、腿自己现查所得的独立观测」这一类，核对表自己也说明了来源）。
- Opus 报告里「没打中的形状」A–F 六条（3 块盘、回退到 mkfs 第 0 代根等未跑通的假设）未复核，那些本身就标注「没试」「推的」。
- Sonnet 报告 P1「今天代码把实例表排在全部提交内生块最前」这一观测，只核对了 `transaction.rs` 这一处，
  未按 Sonnet 自己「没做什么」一节交代的射程去查 `singlefs-checker`、层 0 用例是否也在用「末尾第九个」这个旧描述。
- 未核实 checks-owed.md 里 C126/C121/C335/C83/C363/C314/C332/C340/C124/C318 等仅作「指路」用途、未被整段抄引的编号引用——
  这些只是编号+简称的指路，不构成「整段抄」判据要核的对象。
- 未跑门禁、未跑变异表、未编译除本轮五条探针与三个改法变体之外的任何测试。
- 不写 kb、不 `git add`、不提交；只写了本报告文件与 `/tmp/claude-1000/m2-refusals-presumed-r1-verifier/` 下的草稿。
