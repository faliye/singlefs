# m4-validate-code-r1 攻方腿（Opus）报告（2026-10-22）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 K1 K2 K3 K4 K5 K6 K7 K8 K9 K10 K11 K12 R1 R2 R3 R4 R5 R6 S1 S3 -->

轮名 m4-validate-code-r1；正文 `research/prompts/_m4-validate-code-r1-body.md`，背景材料 `research/prompts/_m4-validate-code-r1-background.md`；被判条款来自 `research/prompts/m4-trunc-r1-main-verification.md` 与 `research/prompts/m4-trunc-r2-main-verification.md`。前几轮判决：无。开工时 `sha256sum -c research/prompts/m4-validate-code-r1-snapshot/crates-sha256.txt` 在冻结副本里 18 行全部 OK。

## 复跑

整条经内存包装跑，工作目录随意，`--work` 给一个新目录：

```
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-validate-code-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo --work <新的草稿目录>
```

`ATTACK_RERUN_GROUPS` 选组（默认七组全跑），`ATTACK_RERUN_TARGET_DIR` 指一个已有编译目录可省全量编译。我自己分四次跑完七组（每次一个新 `--work`），最后一次核对的两组输出 `name=reference-filesystem same_as_model_out=yes exit=101`、`name=other-changes same_as_model_out=yes exit=0`。

模型目录 `research/prompts/m4-validate-code-r1-opus-model/` 每个文件的 sha256：

```
d505e04e6d1e521c508a6b0c1b6b3056255d04b2578818b95b92f3b1daabca7a  research/prompts/m4-validate-code-r1-opus-model/out/cold-recovery.txt
9771bc6ae4679bf4919a097295c88e5f6fcc9018eb9b3ff4f274bc9e4e6c2e4d  research/prompts/m4-validate-code-r1-opus-model/out/cold-walk.txt
ba4b78f4f139badc8fcd8064df8c8e24bd86bb2be47cd8b77a5520c89719dfe5  research/prompts/m4-validate-code-r1-opus-model/out/main.txt
fa71b562190058a945207134521feb6f93686b212a82459a000628590d5feff2  research/prompts/m4-validate-code-r1-opus-model/out/other-changes.txt
410dc616dd0d9e8130fbb9864339db5bc5f54ae045db08d766009583fab940bf  research/prompts/m4-validate-code-r1-opus-model/out/reference-filesystem.txt
9367b5dd3e744138a3cabbb0d60efbc8c9313f08b1755ff5102257c449669aee  research/prompts/m4-validate-code-r1-opus-model/out/rollback-histories.txt
b2684cc2212186a32bca3a7daa6235d6230134e0d74440b8c6739194154fbdc1  research/prompts/m4-validate-code-r1-opus-model/out/slots.txt
84b1d3f52bace9aa3e4352d96c688b1b04c50a058a85b8eab3a9f33abb71765d  research/prompts/m4-validate-code-r1-opus-model/prototype/truncation_code_round_attack_cases.rs
51483a3f19b72d0491c170abf7cbff99fdbe523dfb3517fb157d5474b2e69c2f  research/prompts/m4-validate-code-r1-opus-model/tools/rerun.sh
```

跑的时候本机另有别的会话的 cargo（checker 档崩溃点重放与变异复跑）在跑；我在本机加 `nice -n 19`、`capped.sh 31`、内存包装 8G 跑，没换机器。

## 各格判定一览

冻结副本记作 `F` = `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo`（下文引代码一律写它的全路径，行号按冻结副本现查）。量的数都在冻结副本的拷贝上量（`~/.local/share/singlefs/work/m4-validate-code-r1-attack/repo/`），是副本上的数，不算入库装置上的数。

| 格 | 判定 | 一句话 | 证据在 |
|---|---|---|---|
| T1 截断条目与 logged_ops | 没打中 | 条目 44、value 三段加 10 字节补零、补零非零拒、全池至多一条、K11 单一路径、K12 上限都在，且都在任何写之前；补零只有最后一个字节被单测钉住（推的，没跑变异） | 第一节 |
| T2 先落目标与分批 | 没打中（分批本身）；连带打中见 T5 甲 | 下一批只由现行版的单元数与 N 定，续做与一路做下来同一个计划；批不碰记录。改 size 的入口在「第一个文件 0 个单元」上 panic，归 T5 甲 | 第二节 |
| T3 尾单元与 extent 计划 | 没打中 | N 落在净荷边界（整单元、差 1、0、洞里）各判法与条款一致；叶边界 144 的照抄与重写同判决 | 第三节 |
| T4 快照下截断 | 没打中（只读码，没量） | 截断各步换下的数据单元经同一个 deadlist 判定；这一格没造快照用例 | 第四节 |
| T5 变长、洞与读 | **打中两处** | 甲：标签 0 配 size > 0（三处读者与重建都收的合法形）可写挂载后改 size，三种 N 全部 panic（量过）；乙：变长到 1 TiB（K12 上限之内）后挂载态读者读得出、可写重挂与截回都做成，冷走读与冷恢复各要一次分配 1 TiB、进程中止（量过） | 第五节 |
| T6 截断字段与位 2 | **打中一处（登记）** | 代码两处用位 2（0x04），D15 已定项 4 登记表与 `.claude/kb/feature-bits.md` 都还写位 2 未分配；0x02 池可写挂载取号即升成 0x06、截断写字段后三种重读都成（量过） | 第六节 |
| T7 核心层重建读法 | 判不了（题面前提不成立） | 核心层的 `rebuild_version_*` 是沿树重建，不做按单元头认领，读法 Z 四条只在池级 checker 的 `rebuild.rs`；那四条在 checker 里是合取；回退后变长四段历史逐步过 checker，I-1.13 全成立（量过） | 第七节 |
| T8 回退 | 没打中 | 回退取 R_old 的记录与 R_old 的截断条目、成对；回退进截断半路与回退到截后两种目标、两种变长，逐步过 checker 全绿（量过） | 第八节 |
| T9 准入 | 没打中 | 截断意图那次分配记录树之外每盘新占 8–14 槽（六种几何，量过），都不超过 24 | 第九节 |
| T10 RefFS | **打中一处** | RefFS 把截断施加成整份字节 `resize`，截到 1 TiB 进程中止（量过）；随机历史把截断目标限在「现在的 size 加 4 个净荷」，三方对拍从不取到大变长 | 第十节 |
| T11 合回解冲突 | 没打中 | 删掉的常量在 `crates/` 里零命中；解冲突补的五个 `PublishError` 臂在不写通配臂的 match 里 | 第十一节 |

打中的四处都先答了证据纪律的四句（每节末尾「四句」）。改法都是「只在我的模型上量过、被攻过零轮」，大多只是「推的」，逐格标在各节的改法表里。

## 第一节　T1 截断意图条目与 logged_ops（没打中）

试过的形状：逐字段对 K1 的条目布局、补零、至多一条；K11、K12 的入口次序。

- 条目宽 44、value 三段加补零：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/logged_operations.rs:33`：「const TRUNCATION_INTENT_PADDING_BYTES: usize = 10;」，写完断言位置 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/logged_operations.rs:257`：「writer.assert_position(LOGGED_OPERATIONS_ENTRY_BYTES, "logged_ops 条目");」。
- 补零非零拒：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/logged_operations.rs:330`：「if padding.iter().any(|padding_byte| *padding_byte != 0) {」。单测只把最后一个字节改成 1：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/logged_operations.rs:498`：「padded[43] = 1;」。把判法改成只看补零最后一个字节的变异，这条单测拦不住（推的，没跑变异；别的测试拦不拦得住没查）。这是测试强度，不是产品行为错。
- 全池至多一条：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/logged_operations.rs:401`：「if truncation_intents > 1 {」，签发处另有断言（`transaction.rs` 的 `IssueTheTruncationIntent` 那一臂）。
- K12：N = S 走只改记录 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:68`：「std::cmp::Ordering::Equal => SizeChangeOfARequest::SizeUnchanged,」；上限 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:48`：「.checked_mul(payload)」，在 `publish_size_change` 里先于取 txg、先于任何写：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:9868`：「if !target_size_rounded_up_to_a_whole_data_unit_fits(target_size_in_bytes) {」。
- K11：放 0 个单元也走意图加最后一批 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:176`：「if data_units_beyond_the_size <= data_units_per_batch {」。

没打中的原因：条款要的每一样都在、而且都在任何写之前。K12 的上限本身留下了 T5 乙、T10 的那一形（上限放得比冷读与 RefFS 能装的大得多），归那两节。

什么现象会推翻它：盘上读来的截断条目在补零中间某个字节非零而 `parse` 收下；或一个根里两条截断条目被收下。

## 第二节　T2 先落目标、分批与最后一批（分批没打中；改 size 入口的 panic 归 T5 甲）

试过的形状：批边界在续做时重算（只由现行版定）；批与最后一批碰不碰记录；N 落在洞里时放 0 个；读者在批间容忍 size 之外的单元。

- 下一批：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:182`：「data_units_kept: data_units_existing - data_units_per_batch,」，只在「size 之外多于一批」时走，所以不会放到 ⌈N ÷ 净荷⌉.max(1) 以内；单元数取现行版 `data_pointers.len()`，崩溃重挂之后重建出的单元数含 size 之外还挂着的那几个（重建只对 size 之内的单元核声明长度），续做与一路做下来同一个计划。
- 续做的一步先核条目与记录对得上：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:10001`：「&& record.size == intent.target_size_in_bytes」，对不上交类型化错误、不写。
- 多放、少放、放到 N 之前：三样都由上面两行排除（推的，读码）；纯函数单测 `truncation_batches_release_from_the_tail_and_the_final_batch_may_release_nothing` 钉了批大小 1 时的三步。

连带看到的一处（不在 K2 的射程里，记下交主 agent）：可写挂载取号之前的预演只走 `deleted_inodes`，`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:10059`：「let Some(inode) = version_rehearsed.deleted_inodes.first().copied() else {」，截断条目与销毁中的快照不在预演里。预演只为在取号之前拒 C841（`rehearsed_continuation_refusal_is_returned_before_acquisition` 只对 `NextContainerNumberPastTheTopOfItsRange` 交 true），截断各步不碰容器树、碰不到 C841（推的），所以今天不构成打中；以后截断各步要是能报取号之前就该拒的错，这里会漏。

什么现象会推翻「分批没打中」：一段历史在截断半路崩溃、重挂续做之后放掉的单元集合与不崩溃一路做下来的不同。

## 第三节　T3 尾单元重写与 extent 计划（没打中）

试过的形状：P 是净荷 32634。三单元文件（S = 2.5 P）截到 P、P − 1、P + 1、2 P、0；两个单元、S = 5 P（尾部是洞）截到 2 P、2 P − 1、2 P + 1；叶边界 144、288；最后一批放 0 个。

- 重写判据：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:105`：「(declared_length_within_a_size(unit, size_before_in_bytes) != declared_length_in_bytes)」。按它手算：P → 不重写（单元 0 已满）；P − 1 → 重写单元 0 成 P − 1；P + 1 → 重写单元 1 成 1；2 P → 不重写；0 → 重写单元 0 成 0；洞里的 2 P → 单元 1 存在且满、不重写，标签从带洞换回不带洞（S1）；2 P − 1 → 重写单元 1；2 P + 1 → 单元 2 不存在、不重写、仍带洞。与 K3、K5、S1 逐条一致（推的，按代码算；前五样另有 `truncate.rs` 单测钉着）。
- 标签取法 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/extent_tree.rs:335`：「if data_units < size_in_bytes.div_ceil(payload_capacity_in_bytes()).max(1) {」，截断中间态（单元多于 size 要的）判成不带洞，与读者 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/write_request_split.rs:108`：「if data_units_on_disk >= data_units_implied_by_the_file_size =>」同一判。
- 照抄条件 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:250`：「last_data_unit < data_units_after || data_units_after == data_units_before」，再与「不罩着重写的单元」合取：留下 144 个时第 0 片叶照抄（它在意图那次已重写），与判决 K3 的叶边界那一格一致。
- 覆盖写与截断换下的角色：整份写才补「上一版多出来的单元」，`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:4160`：「FileContentOfThePublish::NotRewrittenWhole => Vec::new(),」；带第一个文件内容的计划才算整份 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:4134`：「Some(file) if file.inode.0 == FIRST_INODE_NUMBER => {」。截断意图只重写单元 k 时换下的就是单元 k，不再把 Data(1)、Data(2) 一起换下（判决 W3 那一格）。

什么现象会推翻它：一个 N 让意图那次重写了声明长度已经等于 `N − k × 净荷` 的单元，或该重写的没重写；或叶边界上那片叶既不照抄也不重写、没有节点指着重写后的单元 k。

## 第四节　T4 快照下截断（没打中；只读码，没造用例）

- 截断各步与别的发布走同一条判生死：候选角色里有这次重写的数据单元与没有后继的上一版单元，交 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:5717`：「roles_replaced_via_mapping(previous_version, candidate_roles, file_content)」，文件内容一项取 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:7737`：「FileContentOfThePublish::of_the_plan(self),」。重写前的单元 k 作为 `Data(k)` 在上一版里有落点，按它的映射 key 问快照判定，进 deadlist 或放回，与尾部被放掉的单元同一判（推的，读码）。
- 没造的：快照下截断的用例与崩溃流（K9 第三条流归 checker 档，这一轮不跑）。这一格的「没打中」只算读码，不算一次观测。

什么现象会推翻它：快照建在截断意图之前，截断之后读快照读出的单元 k 不是截断之前那一份，或快照销毁之后单元 k 的旧版本没被放回。

## 第五节　T5 变长、洞的标签与读（打中两处）

### 甲　标签 0 配 size > 0 的第一个文件：读者都收，可写挂载之后改 size 就 panic

条款认它是合法形：`research/prompts/m4-trunc-r1-main-verification.md:39`：「标签 0（没有数据单元）配 size > 0 是整个文件都是洞」；`research/prompts/m4-trunc-r1-main-verification.md:82`：「标签 0 是这个 inode 一个单元都没有（size 0 的空 inode，或 size > 0 的整个文件都是洞）」。三处读者与可写挂载建上一版都收下它：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/recovery.rs:5302`：「第一个文件那一条是标签 0（一个单元都没有）的收下」，挂载态读者 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/mounted_read.rs:790`：「ExtentUpperLeafTarget::NoDataUnit」归「之后到 size 是洞」。

历史（每一步指到许可它的那一句）：
1. 手造镜像：照仓里 `crates/singlefs-harness/tests/whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader.rs` 的办法，新池新建文件（3000 字节、标签 2）那一版把第一个文件那条上段叶条目改成标签 0、拿掉那个数据单元的映射条目——这是那份仓内测试钉住「三处读者都收」的同一张镜像。
2. 可写挂载（`mount_writable`）：做成（量过，见下）。
3. 会话里一次 `UserChange::Truncate`，N 取三种：100（截短）、3000（N = S）、三个净荷（变长）。

放开扫的用户动作：N 的三类（N < S、N = S、N > S）都扫了；对照组同一张镜像上覆盖写、unlink、建快照、建 inode 四种改动。

结局（副本上量的，`out/main.txt` 与 `out/other-changes.txt`）：

```
ATTACK-OUTCOME 标签 0 size 3000 截到 100: panic：截断留至少一个单元、变长不加单元
ATTACK-OUTCOME 标签 0 size 3000 改成 3000: panic：截断留至少一个单元、变长不加单元
ATTACK-OUTCOME 标签 0 size 3000 变长到三个净荷: panic：第一个文件至少一个数据单元
ATTACK-OUTCOME 标签 0 size 3000 覆盖写 100 字节: 做成：1 版
```

对照的 unlink、建快照、建 inode 都做成（`out/other-changes.txt`）。panic 的位置：

- N < S 与 N = S：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:241`：「(1..=data_units_before).contains(&data_units_after),」，断言文字在第 242 行。走到这里是因为改 size 的入口把单元数取成上一版的 `data_pointers.len()`（这里是 0）：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:9888`：「u64::try_from(previous.data_pointers.len()).expect("单元数")」，再由 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:7394`：「size_change_of_the_first_file(deletion_and_lineage.step, previous, carried_data_units);」交给 extent 计划。
- N > S：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/truncate.rs:145`：「.expect("第一个文件至少一个数据单元"),」。
- `publish_size_change` 文档里写明的 panic 只有一种：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:9848`：「截断那一条而上一版的承诺类队列不空」。这三处 panic 都不在里面；盘上读来的合法形让发布路径 panic，不是类型化错误。

复现：模型目录 `prototype/truncation_code_round_attack_cases.rs` 的三条测试 `a_whole_file_hole_of_tag_zero_{shrunk,kept_at_its_size,extended}_by_the_session_does_not_panic`，断言「不 panic」，今天三条都红；命令见开头复跑。

四句：
- 分不分辨臂：这一轮是代码轮，没有臂；它分辨的是「读者收这一形」与「写者接得住这一形」两侧不一致。
- 系统当时看不看得到判别它的东西：看得到。`publish_size_change` 在任何写之前就有记录的 size（3000）与 `data_pointers.len()`（0）。
- 满足判据的哪个分句：K5「标签 0（没有数据单元）配 size > 0 是整个文件都是洞」与 S1「标签 0 是这个 inode 一个单元都没有」定它合法；K1 只给「nlink = 0 或不在树里」两种在任何写之前拒，这一形两种都不是；所以是「合法镜像被拒」，拒的形式是进程 panic。
- 跑前条款给的改法：判决没给这一格的改法（条款只写到读者收）。下表是我提的。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 子：`publish_size_change` 在取 txg 之前，第一个文件 0 个单元而 N ≠ S 或 N = S 时交类型化拒绝（新的 `TruncationRefusal` 成员） | 三种 N 都不再 panic；代价是这一形的文件改不了 size，只能覆盖写 | 推的 |
| 丑：让 0 个单元的文件走得通：变长与 N = S 只改记录、标签留 0；截短只改记录；`plan_the_extent_tree_after_a_size_change` 收「之前 0、之后 0」 | 三种 N 都做成 | 推的；要动 K5 的「变长」那一句与断言的前提，算新条款 |
| 寅：读者不再收第一个文件的标签 0 配 size > 0 | 让这张镜像在挂载时就拒 | 推的；推翻主 agent 对截甲停下第 ⑤ 处的定案，要重走一轮 |

三个改法都只在我的模型上想过、被攻过零轮。

什么现象会推翻这一处：同一张镜像在入库装置上可写挂载被拒（那就到不了改 size）；或会话在 `publish_size_change` 之前已把这一形拒掉。

### 乙　变长到 1 TiB：挂载态读者读得出，冷走读与冷恢复各要一次分配整个 size

条款：`research/prompts/m4-trunc-r1-main-verification.md:46`：「N 的上限是 ⌈N ÷ 净荷⌉ × 净荷不溢出 u64，超了在任何写之前拒成类型化错误」；`research/prompts/m4-trunc-r1-main-verification.md:39`：「读者读洞答 0、不读盘」；正文这一格要问的 `research/prompts/_m4-validate-code-r1-body.md:48`：「冷读与挂载态读同判」。

历史：三单元文件（S = 2.5 P）→ 会话外经 `publish_size_change` 变长到 2⁴⁰ 字节（K12 之内；变长只补满单元 2，之后到 size 是洞，单元数仍 3）→ 四种读：挂载态读者、可写重挂再截回 100、冷走读 `walk_to_file`、冷恢复 `recover`。

结局（副本上量的，`out/main.txt`、`out/cold-walk.txt`、`out/cold-recovery.txt`）：

```
ATTACK-OUTCOME 挂载态读者读 1 TiB 文件末尾 10 字节：全 0，读盘 0 个单元
ATTACK-OUTCOME 1 TiB 文件可写重挂截回 100：做成 2 版
ATTACK-OUTCOME 冷走读开始
memory allocation of 1099511627776 bytes failed
ATTACK-OUTCOME 冷恢复开始
memory allocation of 1099511627776 bytes failed
```

直接执行测试二进制时退出码 134（SIGABRT）；经 cargo test 跑是 101（`out/*.txt` 末行）。位置：冷走读把整份内容交成一个向量，洞补到 size：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/recovery.rs:6087`：「content.resize(」，长度取 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/recovery.rs:6088`：「usize::try_from(inode_record.size).expect("文件长度装得进 usize"),」。挂载态读者按请求的区间读，洞那一段现补 0：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/mounted_read.rs:997`：「尾部的洞（标签 3、4、0，判决 m4-trunc-r1 K5）：这一段答 0、不读盘」。

后果：用户一次合法的变长（不写一个新单元、盘上不多占空间）之后，冷恢复入口与冷走读在这个池上中止进程；checker 档用 `recover` 做冷恢复的几处装置（`crash_given_states.rs`、`crash_injection.rs`、`bad_disk_input.rs` 等，`grep -rn 'recover(&'` 现查）在这种池上同样中止。size 比内存小而比上限大的（例如 16 GiB、内存包装 8G）推的也一样中止，没量。

四句：
- 分不分辨臂：没有臂；分辨的是挂载态读者与冷读两条读路径。
- 看不看得到：看得到，两条路径分配之前都已读到记录的 size。
- 满足的分句：K12 的上限允许这个 N；K5「读者读洞答 0、不读盘」两条读路径都满足了「不读盘」，冷读不满足「答得出」；正文「冷读与挂载态读同判」这一句被打穿。
- 改法：判决没给。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 子：冷走读与冷恢复不再交整份 `Vec<u8>`，交「单元内容 + 尾部洞长度」或按区间读 | 冷读两处 | 推的；改 API，checker 档调用方跟着改 |
| 丑：K12 上限收窄到冷读装得下的数（例如单元区容量，或一个写死的最大文件长度） | 冷读两处与 T10 | 推的；改条款，要走三方 |
| 寅：冷读遇到 size 超过某个界时交类型化拒绝 | 冷读两处不中止，但这类池冷读读不出 | 推的 |

都被攻过零轮。什么现象会推翻它：入库装置上同一段历史的冷走读不分配整份 size（例如入库版本改了 `content_padded_with_the_hole_up_to_the_size`）。

## 第六节　T6 inode 记录截断字段与 incompat 位 2（打中一处：位 2 没登记）

- 判决要求登记：`research/prompts/m4-trunc-r2-main-verification.md:70`：「在 D15（格式冻结政策） 已定项 4 登记 incompat 位 2（inode 记录带截断字段）」。
- 代码已经用位 2：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/system_configuration.rs:35`：「pub const INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04;」；池级 checker 自己一份 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-checker/src/lib.rs:280`：「const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u8 = 0x04;」。
- 两处登记都还写没分：`.claude/kb/decisions/15-格式冻结政策.md:82`：「未分配；每开一条布局线追加一位」；`.claude/kb/feature-bits.md:19`：「incompat 分了位 0（退役）与位 1，别的位一位都没分」。两份文件与开工快照 `kb-sha256.txt` 的哈希相同（`sha256sum` 现查），HEAD 上也一样（`git show HEAD:` 现查）。
- 门禁这一格的判法（没跑门禁，推的）：`.claude/gate.d/code-source-discipline.sh:729`：「③ `crates/*/src/**/*.rs` 里名字带 INCOMPAT / COMPAT_RO / COMPAT 段的常量，值解得出位号的，那个位号必须在记账表里；」——上面两行都是一行写完、值是位掩码字面量，照这条会被点名。
- 正文第二节说这个常量在 `crates/singlefs-format/src/lib.rs`，现查那份文件命中 0 次，它在 `system_configuration.rs:35`（不影响判定，记一笔）。

四句：不涉及臂；判别它的东西（两份 kb 表与代码常量）在提交时都看得到；满足的是判决第六节「位号与登记归阶段同步」之后那一句没做完；改法就是在两处登记（kb 改动，不是代码改法），登记行的「含义」要与 `system_configuration.rs:31` 那一句「inode 记录带截断字段」对得上，而登记表那一格「每开一条布局线追加一位」的说法与位 2 是特性位而不是布局线不一致，登记时要一起改。

没打中的部分（量过，`out/main.txt`）：把三单元池每块盘两个系统配置槽的 incompat 第一字节改成 0x02、重封校验和之后——

```
ATTACK-OUTCOME 改成 0x02 之后：盘 0 槽 0: incompat0=0x02 世代 6；盘 0 槽 4096: incompat0=0x02 世代 7；盘 1 槽 0: incompat0=0x02 世代 6；盘 1 槽 4096: incompat0=0x02 世代 7
ATTACK-OUTCOME 可写挂载之后：盘 0 槽 0: incompat0=0x06 世代 12；盘 0 槽 4096: incompat0=0x06 世代 11；盘 1 槽 0: incompat0=0x06 世代 12；盘 1 槽 4096: incompat0=0x06 世代 11
ATTACK-OUTCOME 截到 100：意图那一版字段 T = 8、N_w = 100
ATTACK-OUTCOME 截断之后只读挂载：Ok(())
ATTACK-OUTCOME 截断之后冷走读：Ok(Some(100))
ATTACK-OUTCOME 截断之后可写重挂：Ok(())
```

可写挂载在发得出截断意图之前就把四个槽都写成 0x06（`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:626`：「写者每次写系统配置都带 incompat 位 2」），所以「0x02 的槽配非零字段」在这段历史上造不出来；要造出来得两块盘上 0x06 那两份同时读不出（两处故障叠加，没造）。五处构造 `InodeRecord`：整份写清零 `transaction.rs:8412`、建 inode 清零 `:8681`、unlink 两处照抄 `:9715`、`:9814`、改 size 那一处照 R2（`truncate.rs:122` 的「放了单元才写」），与 R2 一致（读码）。

## 第七节　T7 恢复与扫描重建在核心层的读法（判不了：题面前提不成立）

- 核心层的 `rebuild_version_in_the_unit_area_starting_at`、`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at` 是沿树重建：从根读树表、inode 树、extent 树，数据指针取 extent 树里的，只对 size 之内的单元按声明长度核（`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/recovery.rs:3384`：「let data_units_within_the_size = data_unit_count_matches_the_inode_size(」）。它不扫单元区、不按单元头认领数据单元，读法 Z 的四条在核心层没有对应物，所以「核心层这一侧有没有漏条件」答不出是或否。截断字段在核心层只用来判这个池收不收这条记录（`records.rs` 的 `truncation_field_fits_the_pool`），不参与认领。
- 认领四条只在池级 checker：大小那一条 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-checker/src/rebuild.rs:795`：「if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {」，写序那一条 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-checker/src/rebuild.rs:800`：「.is_some_and(|newest_below| *newest_below > birth_txg_of(version))」，第四条 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-checker/src/rebuild.rs:815`：「&& birth_txg_of(version) < truncation_field_txg」。三条各自 `continue`，都在 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-checker/src/rebuild.rs:789`：「if let Some(record) = record_of_this_object {」之内，是「都满足才认领」的合取，且只对有同号同出生代记录的单元判（S3）。对象出生代那一条在 `record_of_this_object` 的过滤里。
- 量的（副本上，`out/rollback-histories.txt`）：见第八节那四段历史，每一步镜像 I-1.13、I-9.39、I-9.7 都是 `Holds`。我原先预测「回退到 (T1, P) 那一版再变长，回退放掉而还在盘上的、出生晚于 T1 的单元会被认回」——结果没有，这一预测作废。没认回的原因我没去读 checker 取「当前单元」那一段，推的是回退放掉的单元不在它扫的那一集里。

## 第八节　T8 回退（没打中）

- 回退取 R_old 的记录：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/mount.rs:9178`：「inode_record: target_version.inode_record,」；截断条目取 R_old 那一版的：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/mount.rs:9516`：「.find(|entry| entry.operation.kind() == LoggedOperationKind::Truncation),」，发布那一步先去掉现行版的截断条目再放进 R_old 的：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/transaction.rs:5517`：「entries.retain(|entry| entry.operation.kind() != LoggedOperationKind::Truncation);」。记录与条目都来自同一个 R_old，成对；字段随记录走，没有并回现行版字段的路径（R5）。
- 量的历史（`out/rollback-histories.txt`，副本上，经 `execute_history_handing_out_every_step_image` 每一步镜像过池级 checker）：起点第一个文件 → 0 顺序写 3 个单元 → 1 缩到单元边界 P（写字段）→ 2 再顺序写 3 个单元（清字段）→ 3 回退 → 4 跨单元变长。用户动作放开扫的是回退目标（根环里第 1、2 新：截后那一版 txg 6、截断意图那一版 txg 5）与变长目标（选择子 0、1：到 2 P、3 P），共 4 段，每段 5 步加起点 6 张镜像，24 张全部 `Holds`，历史结局都是 `Completed`。回退进截断半路（txg 5）那两段，第 4 步会话先把回退带回来的截断走完再变长。
- 没扫的：回退之后再截断、再回退的多次回退；回退目标在快照下（第一版有快照就拒回退）。

## 第九节　T9 准入（没打中）

- 截断意图、各批、最后一批是删除类，变长与 N = S 是普通写（`truncation_intent_batches_and_final_batch.rs` 的单测钉着）；可写挂载前那一判在截断条目没走完时把环窗口算作可动用，判断收成同一个方法 `queue_of_committed_intents_is_empty`（K1）。
- 意图那次每盘新占槽数（分配记录树之外，口径同仓里那条单测；副本上量的，`out/slots.txt`）：

```
ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 11 槽：3 单元截到单元 0 中间
ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 11 槽：3 单元截到单元 1 中间
ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 12 槽：150 单元截到单元 143 中间（叶边界）
ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 13 槽：300 单元截到单元 287 中间（叶边界）
ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 14 槽：300 单元截到单元 0 中间
ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 8 槽：300 单元截到单元边界 150 P（不重写）
```

  最大 14，W 里那一项是 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/admission.rs:384`：「pub const DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST: u64 = 24;」。14 是取样里的最大值，不是上界：下段每多一层多 1 槽（推的），144² 个单元以上的文件没量。
- 试过但没成形的：快照下反复「截短 1 字节」让每次意图都新写一个单元而不判式子。只有快照建立之前那一份单元 k 被快照钉住，之后每次换下的都在 defer 窗口里回来（推的），超出不了环窗口那一项的设计口径，没造用例。

## 第十节　T10 RefFS 的截断（打中一处）

- RefFS 把截断施加成整份字节：`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-reference-filesystem/src/lib.rs:644`：「.expect("64 位上 usize 与 u64 同宽：截到的长度装得进"),」那一句前面是 `truncated.resize(`。截到 1 TiB（实现照收，K12 之内）：

```
ATTACK-OUTCOME RefFS 截到 1 TiB 开始
memory allocation of 1099511627776 bytes failed
exit=101
```

  （`out/reference-filesystem.txt`，副本上量的。）
- 三方对拍从不取到这一形：随机历史的截断目标写明 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-harness/src/history.rs:676`：「都装得进内存（至多现在的 size」，跨单元变长那一类是现在的 size 加 1 到 3 个净荷。所以三方对拍里「截断那一格」只判了小变长；大变长上实现、模型、RefFS 判的不是同一件事——实现做成、RefFS 中止（理想模型没量）。
- 崩溃允许集合（读码，没打中）：截断是内容改动型、不是发布点（`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-reference-filesystem/src/lib.rs:57`：「不是发布点」），崩在截断里允许截前与截后两样；实现截断半路崩溃，重挂读到的是截后的 N（先落目标），在集合里。
- 四句：没有臂；RefFS 在 `submit` 时看得到 N；满足的是 K8「施加 = 截短或补 0」与 K12 上限两句合起来，实现与 RefFS 对同一个合法输入一边做成一边中止；改法同第五节乙的子、丑（RefFS 把尾部 0 记成长度、不展开；或 K12 收窄），都是推的、零轮。

## 第十一节　T11 合回时解的冲突（没打中）

- 删掉的常量：`grep -rn NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS` 在冻结副本 `crates/` 下 0 行；仓里只剩 kb 变更史与旧轮材料提到它（`git grep` 现查），没有代码或门禁表在用。
- `mount.rs` 那一处补的五个 `PublishError` 臂：在一个不写通配臂的 match 里，`/home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo/crates/singlefs-core/src/mount.rs:5520`：「| PublishError::TruncationRefused(_) => false,」，漏一个成员编译就过不去；取的是「只有 C841 一种在取号之前拒」那一侧，截断拒绝不在取号之前拒，与另一侧不冲突。
- `recovery.rs` 混着别的会话行的三个 hunk：`extents_of_the_first_file` 里「第一个文件之外的文件尾部不许有洞」与「第一个文件标签 0 收下」两侧的判都在（读码）；第一个文件那一侧的后果见第五节甲。
- 事务员报告里另两处整段取 master 一侧、没写新行的，没再逐行核。

## 没打中的形状

| 形状 | 取样范围 |
|---|---|
| 截断条目补零、两条截断条目、K12 上限、K11 放 0 个 | 读码；单测已有的取样 |
| 批边界重算（多放、少放、放到 N 之前） | 读码；纯函数单测批大小 1 |
| 尾单元 N 落在 P、P ± 1、2 P、0、洞里 | 手算 8 个 N（第三节） |
| 快照下截断 | 只读码 |
| 标签 0 镜像上的覆盖写、unlink、建快照、建 inode | 各 1 次，量过，都做成 |
| 0x02 池可写挂载、截断写字段、三种重读 | 1 段，量过 |
| 回退之后变长，逐步过池级 checker | 回退目标 2 × 变长目标 2 = 4 段、24 张镜像，量过 |
| 截断意图每盘新占槽数 | 6 种几何（3、150、300 个单元），量过 |
| 快照下反复截短 1 字节绕过式子 | 只推，没造 |
| 合回解冲突丢行为 | 读码 3 处 |

这些「没打中」都只抽了一次样，不支撑「没打中」的结论（三方规则「一条腿只抽一次样不算一次观测」）。

## 这条腿自己的限度

- 一切数都在冻结副本的拷贝上、debug 编译、内存盘与临时文件镜像上量；不是入库装置上的数。
- 标签 0 那张镜像是照仓里现有测试手造的；我没核这一形在入库的写路径上能不能自然出现（条款说今天写不出），它的合法性只来自 K5、S1 两句与「三处读者都收」那份仓内测试。
- 1 TiB 只量了一个点；「比内存小、比上限大」的 size 推的同样中止，没量。
- 没跑 checker 档、没跑崩溃枚举：这一轮的形状都不靠崩溃状态（标签 0 那三处在第一次发布之前就 panic，大洞那两处在读的时候中止），我也没调 `enumerate_crash_states`，原型跑的崩溃状态数是 0。
- 没跑门禁；位 2 那一格对门禁的判断是照门禁脚本的注释与正则推的。
- 模板自带的 `--selftest` 在改过 `rerun_steps` 之后红（它照模板原样的那一步核），这是模板与改过的步骤不配，不是复跑红。

## 要主 agent 跑的命令

复跑全部七组（预计经 cargo 编一次、各组跑完合计 10 分钟上下，前台放不下，要放后台）：

```
ATTACK_RERUN_GROUPS="main cold-walk cold-recovery reference-filesystem other-changes slots rollback-histories" \
  nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-validate-code-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-validate-code-r1/repo --work <新的草稿目录>
```

工作目录随意；线程照 `capped.sh 31`；看每组一行 `name=<组> same_as_model_out=yes exit=<码>`，七行都是 yes 才算复跑相同。exit 是 cargo 的退出码：main、cold-walk、cold-recovery、reference-filesystem 四组 101（有红或进程中止），other-changes、slots、rollback-histories 三组 0。

## 没做什么

- 不判正推那一格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没读禁读清单里的文件。
- 没跑 checker 档测试、门禁阶段与轻型三条；没改仓里任何代码。
- T4 快照那一格没造用例；T7 那一格没读 checker 取当前单元的来历。
