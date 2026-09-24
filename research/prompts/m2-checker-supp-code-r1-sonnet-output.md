# 代码轮 m2-checker-supp-code-r1 云端正推腿：K3（checker 补的三条判得对不对）

**攻方**：Sonnet，云端正推腿。**射程**：只判 K3（`I-1.8`、`I-7.3`、`I-8.6` 三条判得对不对），不碰 K1、K2（事务号）与 K4（早退条件）。

## 0. 开工核实

工作区快照与派发时给的 `research/prompts/m2-checker-supp-code-r1-start-snapshot.sha256` 逐文件 `sha256sum` 核对，9 个文件全部一致（现查命令与输出见下），说明分析期间主工作区未被别的会话改动：

```
$ sha256sum crates/singlefs-checker/src/walk.rs crates/singlefs-checker/src/image.rs crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/mount.rs crates/singlefs-core/src/recovery.rs .claude/kb/invariants.md .claude/kb/checks-owed.md ".claude/kb/decisions/23-journal的角色与格式.md" research/prompts/_m2-checker-supp-code-r1-body.md
b900b29fa6ccb30b8ebd095c2b31e545af07395c45d0c0a540ba795ab7d46ca5  crates/singlefs-checker/src/walk.rs
3feab65f12cb90d4194fab6d19bd1f5e67138c0d3096e3562672e86ce1dd5f8e  crates/singlefs-checker/src/image.rs
ae2cd66a4224ad32dca6786669a0b003159b452d6a470b4e7d4990116e24a94f  crates/singlefs-core/src/transaction.rs
229ec48d2951bd5ac2db6cc02853f44bac86fe8792979bda4138c06a1230e1b0  crates/singlefs-core/src/mount.rs
a452b38c8822789757a3a5c6953a6463670ecc14806048671a004636d78ad21b  crates/singlefs-core/src/recovery.rs
890b75765e129b60ab51e24d9c3daee43a580e4d47604f1b6008bc3cd1f70c9e  .claude/kb/invariants.md
cd9ea9f97c61c27c91219a14f07677309a4ab6b76bc136ed147bf355e5f6a482  .claude/kb/checks-owed.md
c3e07269d4c9ca005bdd232b3e5446cccc665cad92536b94e69588f5ae9e37e4  .claude/kb/decisions/23-journal的角色与格式.md
4a003d9fc894352f8d899ae4712e68949cf2549305f51ad7b267fe8255c5c0af  research/prompts/_m2-checker-supp-code-r1-body.md
```

**实测方式**：main 工作区一个字没改；`rsync -a --exclude target --exclude .git` 拷了一份到 `/tmp/claude-1000/m2-checker-supp-r1-sonnet/repo/`，全部 `cargo test` / 改代码实验都在这份副本上做，改完的临时改动只留在副本、已在副本内还原（`diff` 副本与主工作区的 `walk.rs`，只剩我自己加的一行调试 `eprintln`，见第 4 节）。

## 1. `I-7.3（环健康性）` 的例外：照抄了没有？阳性对照在不在？

**判据原文**（`.claude/kb/invariants.md:52`，整行）：

> `I-7.3` | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到

**实现**（`crates/singlefs-checker/src/walk.rs:1658-1681`，`judge_root_ring_health`）：

```rust
fn judge_root_ring_health(roots: &[(u64, u64, crate::RootView)], judgements: &mut Judgements) {
    let newest_checkpoint_txg = roots.iter().map(|(_, _, root)| root.checkpoint_txg).max()
        .expect("S 非空：`check_pool_image` 判过 I-7.1 之后 S 空就返回了，走不到这里");
    let earlier_generation_exists = roots.iter()
        .any(|(_, _, root)| root.checkpoint_txg < newest_checkpoint_txg);
    let every_root_is_the_genesis_generation = roots.iter()
        .all(|(_, _, root)| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG);
    judgements.judge("I-7.3", earlier_generation_exists || every_root_is_the_genesis_generation, || { ... });
}
```

**判定：一致。** `earlier_generation_exists`（除最大代号外还有更早代号）与 `every_root_is_the_genesis_generation`（S 全部是第 0 代，`GENESIS_CHECKPOINT_TXG = 0`，`walk.rs:1651`）逐字对应判据原文的主判据与「例外只有一个」；`||` 的短路顺序不影响判定结果（两者都是纯读操作，无副作用）。「崩溃后回退到这个态的镜像同样判绿」在实现里体现为**判据只读 `roots` 里的代号集合，完全不读这份镜像是怎么走到这个态的**（无时间戳、无历史字段输入），代码注释（`walk.rs:1655-1656`）原话也这么写。调用点在 `I-7.1` 之后、`roots.is_empty()` 分支之外（`walk.rs:1861-1868`），S 空时报「不适用」而不是拿空集合求 `max()`——这一步与括注「S 空时没有『代号最大者』可谈」一致，代码注释（`walk.rs:1657`）逐字写了这句话，`walk.rs:1864` 的字符串也照抄。

**阳性对照在不在（要求：坏镜像与干净镜像各一份）**——**在，而且是同一份镜像上的最小对照对**（`crates/singlefs-harness/tests/checker_known_bad_images.rs:948-1003`，`the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs`）：

- **干净镜像**（例外那一支的阳性对照）：`mkfs` 刚写完那份（`memory_pool_after_mkfs()`），断言三个区域槽 0 的根代号 `[0, 0, 0]`（`checker_known_bad_images.rs:962-971`），再断言 `verdict(&freshly_formatted, "I-7.3") == InvariantVerdict::Holds`（`:974-978`）——**是真被评估过（Holds），不是「不适用」**，坐实例外分支真的走到了判定。
- **坏镜像**：同一份 mkfs 镜像上，把三个区域槽 0 的根代号从 0 一起改成 1（`known_bad_images_of_the_root_ring_health`，`checker_known_bad_images.rs:813-836`），S 全部同代但不是第 0 代 ⇒ 例外不成立、主判据也不成立 ⇒ 断言 `violated == ["I-7.3"]`（`:983-992`），且**除 I-7.3 之外每一条不变量的判定都必须与干净镜像逐项相同**（`:993-1002`，用差集断言钉死），确保这份坏镜像只改宽了 I-7.3 判别力覆盖到的那一维，不是靠改宽别的东西顺带触发。
- 另有第二段对照（`:1004-1039`）建在「写完第一个事务」那份镜像上：S 只剩 txg 3 那一条时判红，并如实钉住这份镜像**同时**红 I-3.1（mkfs 种的树表单元没人引用了），不是只红 I-7.3——这段照实记录了判别力的副作用范围，不装作干净。

**推翻条件**：若 `every_root_is_the_genesis_generation` 的判定被改成检查“历史上出现过第 0 代”而非“S 里现存的每一条都是第 0 代”，或者两份对照镜像里有一份的诊断信息/其它不变量判定与预期不同，上面两条断言会先报错——目前跑一遍这条用例（见下）两个断言全部通过。

**复跑**（副本上跑，`nice -n 19 cargo test`）：

```
$ cd /tmp/claude-1000/m2-checker-supp-r1-sonnet/repo && cargo test -p singlefs-harness --test checker_known_bad_images \
    the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs -- --nocapture
test the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.14s
```

## 2. `I-8.6（反向链算法）` 的两条射程：实现成什么样？「4 个不适用状态」对不对？

**判据原文**（`.claude/kb/invariants.md:81`，整行）：

> `I-8.6` | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 307 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算与头宽 307 是 2026-09-14 用户定案）。不等的记录不进重放前缀。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)）

**实现**（`crates/singlefs-checker/src/walk.rs:1741-1781`，`judge_journal_back_chain`）核心分支：

```rust
let expected_back_chain = if *counter == FIRST_JOURNAL_COUNTER {
    Some(0)
} else {
    match records.get(&(counter - 1)) {
        Some(previous) if previous.instance == record.instance => {
            Some(previous.chain_value_the_next_record_must_carry)
        }
        Some(_record_of_another_instance) => Some(0),
        None => None,
    }
};
let Some(expected_back_chain) = expected_back_chain else { continue; };
judged_records += 1;
judgements.judge("I-8.6", record.back_chain == expected_back_chain, || { ... });
```

**判定：一致，但两条射程在实现里合成了一个三分支，不是逐字对应两句独立的话——这一步的等价性需要额外推一步（见下）。**

- 「本实例写出的第一条记录反向链恒 0」被拆成两种判法都落到 `Some(0)`：① 全池计数器 == 1（`FIRST_JOURNAL_COUNTER = 1`，常量定义在 `walk.rs:1687`、用在 `walk.rs:1752`，全池第一条记录必是某个实例的第一条）；② 计数器 − 1 那条记录存在且实例代号**不同**（对应分支 `walk.rs:1759`，此时本条按定义是新实例的第一条）。两者穷举了「本实例第一条」的两种物理成因（全池起点 / 实例切换点），实现注释（`walk.rs:1735-1738`）自己写了这个穷举的三分支论证。
- 「跨实例边界的两条记录之间不判」体现为：分支 ②（计数器 − 1 是别的实例）**不去比较**当前记录的反向链与「计数器 − 1 那条记录算出的链值」是否相等（没有 `previous.chain_value...` 这一步），而是直接判定「这一条的反向链该是 0」——即**不做跨实例的链值比较**，只做同实例内的链值比较（分支 ①）。这与括注「链按实例算……不判这一格」是同一件事的两种说法：原文说「不比较」，实现把「不比较」实现成「改用一个与前一条记录无关的常量 0 来判定」，而不是「跳过这一条判定」。**这一步需要现查坐实（不是纯粹摘句）**：若把「不判」读成字面的「skip，不产出判定」，实现与原文字面不一致；但只有这样读，「本实例第一条恒 0」这句话才有落点——原文本身要求对**这一条记录**判「恒 0」，而「不判」限定的对象是**它与前一条记录的链值比较**，不是这条记录本身要不要判。两句连读只有这一种读法能自洽，实现按这种读法写。
- `None`（计数器 − 1 找不到自证过的记录）时 `continue`，不产出判定——这不在判据原文两条射程之内，是第三种情形（环还空、或那条记录本身自证不过、或环绕过一圈后那一格是旧记录），实现注释（`walk.rs:1739-1740`）单独交代了这一支，不算对判据原文的偏离。

**「层 0 有 4 个状态报不适用」——实测坐实，不是照抄背景材料的数**：

副本上跑 `first_transaction_step_seven_layer0.rs` 的 262165 状态全量用例（release，`--ignored`）：

```
$ cd /tmp/claude-1000/m2-checker-supp-r1-sonnet/repo && cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 \
    layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations -- --ignored --nocapture
...
CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 ... I-1.8=262165/0 ... I-7.3=262165/0 ... I-8.6=262161/0 ...
test layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 7.90s
```

`262165 − 262161 = 4`，与背景材料「层 0 里有 4 个状态报不适用（环还空着）」逐字对得上；用例自己的断言（`first_transaction_step_seven_layer0.rs:83-133`，`assert_checker_counts` 的 `judged_once_a_record_is_on_disk` 分支，`:104-116`）把这个 4 显式钉成 `states_with_an_empty_journal_ring` 形参，调用处传的字面值也是 `4`（`:214`：`assert_checker_counts(&tally, 262_165, 4, 4)`），且注释（`:104-106`）逐字交代了这 4 个状态是什么：「什么都没持久那一个状态，加上取号那一段（两盘各一次系统配置写）的 3 个非空子集，一共 4 个状态还没有记录落盘」。

**推翻条件**：若把 `FIRST_JOURNAL_COUNTER` 改成非 1、或把分支 ② 改成真的去比较「前一条记录算出的链值」而不是常量 0，`checker_known_bad_images.rs:684-701` 那份坏镜像（改 w4 的反向链为 0，期望它与 w1 的头值不等）与本节全量用例都会先报错；4 这个数若因为环几何或暖机写次数变化而改变，`assert_checker_counts` 的三个调用点会先断言失败（`first_transaction_step_seven_layer0.rs:214,247`）。

## 3. `I-1.8（归并后版本全序）` ① 的归并分组键：实现用的是不是「类身份段全部字段含写序」？

**判据原文**（`.claude/kb/invariants.md:31`，整行，其中要判的是「**类身份段全部字段含写序**」这个归并键与「同一组的成员载荷相同」这半句）：

> I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）

**「类身份段」这个词自己的权威定义**——不能凭印象读，要去 D18（块里携带什么信息） 已定项 7 / 11 现查（`.claude/kb/decisions/18-块里携带什么信息.md`）：

- 已定项 7，`168` 行标题「**类身份段——数据单元（+63 字节 ⇒ 共 105 字节初值……）**」，其下字段表明确把「载荷 CRC」列为这一段的成员之一：`177` 行「| **载荷 CRC** | **4** | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3：副本归并要求「同类身份段的多份可读单元载荷相同」……只加在**码 1 自己的类身份段**，不进共同前缀 |」——这句原话直接把载荷 CRC 算作码 1「类身份段」的一员。
- 已定项 11，`271` 行标题「**码 3 的类身份段（65 字节，头 107）**」，其下字段表 `284` 行同样把「载荷校验和」列在这段里：「| 89 | 载荷校验和 | 4 | CRC32C，覆盖偏移 107 到单元末尾…… |」。

**⇒ 按 D18 给「类身份段」的权威定义，「类身份段全部字段」逐字包含载荷校验和 / 载荷 CRC。**

**实现**（`crates/singlefs-checker/src/walk.rs:1327-1370`，`ContentUnitHeaderOffsets`）却**明确把载荷校验和从归并键里排除**：

```rust
/// 归并键的两段：「类身份段全部字段含写序」**去掉载荷校验和那一段**。
/// 载荷校验和不进归并键——它正是「同一组的成员载荷相同」要比的那个量，进了键这一条就恒真。
merge_key_before_payload_checksum: std::ops::Range<usize>,
merge_key_after_payload_checksum: std::ops::Range<usize>,
```
（码 1：`42..101` + `105..105`，跳过 `101..105` 那 4 字节载荷 CRC；码 3：`42..89` + `93..107`，跳过 `89..93` 那 4 字节载荷校验和。）

**判定：字面不一致，但这个偏离是必要的、且实现自己写清了为什么。** 若归并键真的把载荷校验和/CRC 也算进去（严格按 D18 给出的「类身份段」定义），两份载荷不同的副本从一开始就会算出不同的归并键、进不了同一组，① 的「同一组的成员载荷相同，不同即判损坏」这句话就**永远没有输入可比**——载荷一旦真的不同，归并键先一步把它们分开，判据变成同义反复（这正是 `walk.rs:1336` 那句注释自己讲的道理）。所以实现的排除法是让 ① 具备判别力的**必要**条件；但反过来看，判据原文「类身份段全部字段含写序」这一句，若读者只读 `invariants.md` 这一行、不去翻 D18 的字段表反推出「载荷 CRC 也在类身份段里、必须被排除」这一步，字面复现就会踩进这个坑——**这一步排除没有被写回 `invariants.md` 的判据原文里，只存在于实现的代码注释**。这本身构成一处「判据原文表述不够精确、需要跨文件反推」的发现，价值与欠账表 C464（已知的、码 1 全序键停机）不同，是**归并键本身的问题**，欠账表里没有登记这一条。

**「多一个字段少一个字段会怎样」——实测，不是空想**：见第 4 节，用实际改代码的方式验证了「加回载荷校验和」这个方向会发生什么。「少一个字段」的方向（比如从归并键里再去掉 fsid 或诞生代号）会让**不该合并的单元被强行合并**——归并键变窄之后，两个身份不同、内容合法不同的单元只要在变窄之后的字段上偶然相同就会被分进同一组，① 的载荷比对就会把两个合法但不同的单元判成「同一组成员载荷不同」而误判红；这一支没有去实测构造，因为 fsid、诞生代号、写序都是 COW 语义下天然互斥的字段（同一个五元组/对象身份 + 同一次发布不可能产生两份不同内容的合法单元），构造一份「合法但去掉字段后误判红」的镜像需要绕开 COW 分配规则本身，超出这一面（K3）能在不碰 K1/K2/K4 的前提下构造出的范围，这里明确写「未验证，只是从字段语义推出的方向」。

## 4. 反例：按判据原文字面重建归并键，I-1.8 会对码 1 单元判不出真损坏

这是这一面里唯一真正做了「改代码、跑坏镜像」的实验，全程在副本 `/tmp/claude-1000/m2-checker-supp-r1-sonnet/repo/` 上做，**主工作区没有改动**（`diff` 副本与主工作区 `walk.rs` 只剩我自己留的一段调试 `eprintln`，已在做完实验后手工撤掉归并键的改动，只保留了那段无害的调试打印，见下方「清理」小节）。

**做法**：把 `ContentUnitHeaderOffsets` 的归并键改成完全按 D18 的「类身份段」字面定义（把载荷校验和/CRC 也纳入归并键，即第 3 节说的那个「若字面复现」的版本）：

```
merge_key_before_payload_checksum: 42..105,   // 原为 42..101（数据单元）
merge_key_after_payload_checksum: 105..105,   // 不变
merge_key_before_payload_checksum: 42..107,   // 原为 42..89（打包记录单元）
merge_key_after_payload_checksum: 107..107,   // 原为 93..107
```

**先验证码 3（打包记录单元）**：跑已有的坏镜像用例 `the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant`（它的 I-1.8 那份坏镜像改的正是 `INODE_LEAF`，一个码 3 单元，`checker_known_bad_images.rs:706-713`）：

```
$ cargo test -p singlefs-harness --test checker_known_bad_images the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant -- --nocapture
test the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant ... ok
```

**这份用例仍然通过**——但不是因为 ① 还能比对，而是**② 顺带兜住了它**：归并键一改，原本同组的两份副本因为载荷校验和不同而各自落进两个「单例组」（`members.len() == 1`，① 直接 `continue`、不产出比对），但这两个单例组共享同一个 `object_key`（标签/出生树/类型/容器号/容器出生代不受载荷改动影响），于是各自的代表单元被塞进同一个 `groups_by_object_key` 桶，桶里出现 2 个 `total_order_key` 完全相同（同一次写、同一个实例/事务/诞生代号）的代表 ⇒ ② 的「同一个 key 上归并出 N 组，两两全序键不等」判据被触发，判定仍然是 `Violated`。**这是侥幸，不是设计**：② 之所以还能接住，只因为码 3 参与 ②；如果同样的攻击面发生在码 1 上，就没有这个后备——码 1 的 `TotalOrderKey` 恒是 `UndecidedForDataUnit`（`walk.rs:1423-1431`），② 的第一步就把它 `continue` 掉了（`walk.rs:1608-1610`），码 1 从来不会被放进 `groups_by_object_key`。

**验证码 1（数据单元）——这才是真正的反例**：新写一条临时用例，对 `DATA_UNIT`（码 1）做同构的「只改一块盘那份的载荷、reseal 重算校验和」：

```rust
#[test]
fn debug_i18_literal_merge_key_misses_data_unit_payload_mismatch() {
    let clean = build_pool("debug-i18-literal-merge-key").memory_pool();
    let mut image = clean.clone();
    mutate_one_device_copy_of_unit(&mut image, &UNITS, 1, DATA_UNIT, |bytes| { bytes[200] ^= 0xFF; });
    let verdicts = check_pool_image(&image);
    ...
}
```

在**归并键按字面纳入载荷校验和**的版本上跑：

```
$ cargo test -p singlefs-harness --test checker_known_bad_images debug_i18_literal_merge_key_misses_data_unit_payload_mismatch -- --nocapture
I18DEBUG data-unit-payload-mismatch verdict=Some(Holds)
test debug_i18_literal_merge_key_misses_data_unit_payload_mismatch ... ok
```

**`Holds`——两个副本载荷已经不同，checker 却判它成立，真损坏被判不出来。** 把归并键改回**今天实际发布的版本**（排除载荷校验和），同一份坏镜像、同一个字节改动，重跑同一条用例：

```
$ cargo test -p singlefs-harness --test checker_known_bad_images debug_i18_literal_merge_key_misses_data_unit_payload_mismatch -- --nocapture
I18DEBUG data-unit-payload-mismatch verdict=Some(Violated("归并成一组的成员载荷不同（盘 0 槽 50180 的码 1 数据单元 载荷校验和 0x73e0742b、盘 1 槽 50180 的码 1 数据单元 载荷校验和 0x8408a295）：同一组的成员载荷必须相同，不同即判损坏、不择"))
```

**这就是第 3 节说的「必要」——今天实际发布的实现是对的**；这个反例证明的是：判据原文「类身份段全部字段含写序」若被后来者按 D18 的字段表**字面**实现（把载荷校验和纳入归并键），I-1.8 对码 1 单元会整个丧失判别力，而且不像码 3 那样有 ② 兜底——因为 C464（I-1.8 对码 1 的全序键写窄了） 已经让码 1 停在 `TotalOrderKey::UndecidedForDataUnit`、被 ② 的第一步过滤掉。**两笔欠账叠在一起会放大成一个真正的判不出**：C464 本身只是「② 对码 1 不判」，单独看不影响 ①；但如果归并键的排除规则哪天被按字面「修回」判据原文，① 对码 1 也会失效，而 C464 造成的「② 对码 1 也不接盘」使得这次失效没有任何后备——这是当前登记的 C464 描述里没有覆盖到的一层风险，值得在 C464 或新开一条欠账里补一句「归并键排除载荷校验和这一步，是维持 ① 判别力的必要条件，不能按 D18 字段表字面把它加回去」。

**推翻条件**：若有人能证明「归并键包含载荷校验和」时码 1 也存在某种后备判定（本报告没找到），或者证明 D18 的「类身份段」定义在 I-1.8 引用处应读成一个**不含**载荷校验和的子集（本报告在 D18 全文范围内没有找到支持这种读法的第二处定义），上面的反例结论会被推翻。

**清理**：副本上的归并键改动已在做完对照实验后用同一份备份文件（`/tmp/claude-1000/m2-checker-supp-r1-sonnet/walk.rs.bak`，实验前的原始拷贝）整份覆盖复原，`diff` 副本与主工作区确认只剩一段自己加的调试 `eprintln`（不影响判定逻辑）：

```
$ diff /tmp/claude-1000/m2-checker-supp-r1-sonnet/repo/crates/singlefs-checker/src/walk.rs /home/fy5090/code/singlefs/crates/singlefs-checker/src/walk.rs | wc -l
9
```
主工作区自始至终未被写入（第 0 节已用 sha256 核过）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| I-7.3 例外是否照抄 | 一致 | `judge_root_ring_health`（`walk.rs:1658-1681`）逐字实现「除最大代号外还有更早代号，或 S 全是第 0 代」，S 空时另有「不适用」分支，与判据原文与括注对得上 |
| I-7.3 例外的阳性对照在不在 | 在 | `checker_known_bad_images.rs:958-1003` 同一份 mkfs 镜像上钉了「干净（例外成立，真被评估）」与「坏（代号一起改非零，例外够不着，判红）」一对，且用差集断言排除了误判宽 |
| I-8.6 两条射程的实现 | 一致（需要一步推理坐实） | 「本实例第一条恒 0」拆成计数器 1 与跨实例边界两种情形都判 0；「跨实例边界不判」体现为不比较前一条记录算出的链值，只判定「这条该是 0」，两者连读只有这一种读法自洽 |
| I-8.6「4 个不适用状态」 | 一致，已实测复核 | 副本上跑全量 262165 状态：`I-8.6=262161/0`，`262165-262161=4`，与用例自己钉的 `states_with_an_empty_journal_ring=4`（`first_transaction_step_seven_layer0.rs:214`）对得上 |
| I-1.8 ① 的归并键是不是「类身份段全部字段」 | **不一致（有意为之，且原文本身表述不够精确）** | D18 已定项 7/11 给「类身份段」的权威定义包含载荷校验和/CRC；实现明确把它排除出归并键（`walk.rs:1335-1336` 自陈理由），排除是维持 ① 判别力的必要条件，但这一步没有写回 `invariants.md` 的判据原文 |
| 反例：字面归并键对码 1 的判别力 | **打中——实测坐实，Holds 变 Violated 的对照跑通** | 归并键按字面纳入载荷校验和时，`debug_i18_literal_merge_key_misses_data_unit_payload_mismatch` 用例对一个真实损坏的码 1 单元判 `Holds`；改回今天实际发布的排除写法，同一份镜像判 `Violated`。码 3 的同构攻击被 ② 顺带兜住（侥幸），码 1 没有这个后备（C464 已让码 1 退出 ②） |
| I-1.8 ② 对码 1 停机之后，码 3 单独判有没有判别力 | 有，已实测 | 在「发布 B 之后」的干净镜像上直接实测：`object_key_groups=2, candidates_with_2plus_versions=1`，且判定为 `Holds`（真被评估，不是「不适用」）；但第一个事务层 0 的 262165 状态全量流只有单一 checkpoint，② 在那条流上不产出任何判定（`judged` 只来自①），第二个事务的两条层 0 全量用例（`second_transaction_step_zero_layer0.rs`、`second_transaction_step_three_formatted_pool_layer0.rs`）都没有把 I-1.8/I-7.3/I-8.6 放进它们的 `must_evaluate` 断言列表——② 在跨 checkpoint 的崩溃状态语料上确实有判别力，但这份判别力没有被任何一条穷举用例钉住覆盖率，是这一面之外值得记的一条欠账候选 |

## 没做什么

- 不判 K1（事务号改法的射程）、K2（事务号改法买到什么）、K4（早退条件对齐之后的新缺口）——按分工这些归别的腿。
- I-1.8「少一个字段会怎样」只做了字段语义推理，没有实测构造：需要绕开 COW 分配规则本身才能造出「合法但字段变窄后误判红」的镜像，超出这一面能在不碰 K1/K2/K4 前提下构造的范围，已在第 3 节末尾明写「未验证，只是推出的方向」。
- 没有去核 I-8.6「计数器连续、实例切换点计数器不归零」这条前提（D23 已定项 19 注 3）本身有没有会红的检查——jsn 计数器与 K1/K2 关心的「事务号」是两个不同的计数器（`D23 已定项 9`：`jsn = 实例代号 + 计数器`，事务号是记录头里另一个独立字段），这条前提若被破坏会连带影响 I-8.6 的判定逻辑，但它不在 K3 题面点名的两条射程之内，也更靠近 K1 的射程，这里只记录「这是 I-8.6 隐含依赖的一条前提，没有去核」，不展开。
- 没有去核 I-7.3 括注里「连续两代落进同一槽的轮换 bug 要等 R × S 次发布才会被抓到」这句自陈的射程缺口——这是判据原文自己承认的已知盲区，不是实现与判据的不一致，按题面不用去证明「为什么这里判不出」。
- 没有对 `second_transaction` 那两条层 0 全量用例（780K+ 状态量级）做完整实测复跑：只用一次性的 `image_after_the_overwrite` 单点镜像验证了 ② 的判别力存在，没有跑那两条 `#[ignored]` 的全量穷举用例去坐实「② 在整条跨 checkpoint 崩溃状态语料上被判过多少次、有没有真正抓到过一次违例」——这是本报告认为值得记的一条新欠账候选（判定一览最后一行），但把它做成实测需要跑 release 下的第二个事务全量层 0（数十秒到分钟量级），本轮时间与射程都放在 K3 点名的三条判据本身，没有去跑；由主 agent 决定要不要另立一条欠账并派实验去补这个覆盖率缺口。
- 不采纳、不出判决——判决与是否登记新欠账（归并键排除条款的完整性、② 的覆盖率缺口）由主 agent 定。
