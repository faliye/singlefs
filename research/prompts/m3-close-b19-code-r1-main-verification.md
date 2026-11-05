# m3-close-b19-code-r1 判决（代码轮第一轮，2026-10-29）

正文 `research/prompts/_m3-close-b19-code-r1-body.md`；改动全文 `research/prompts/_m3-close-b19-code-r1-diff.md`。腿：正推 `research/prompts/m3-close-b19-code-r1-sonnet-output.md`、攻方 `research/prompts/m3-close-b19-code-r1-opus-output.md`（1ad2697d…，模型 `research/prompts/m3-close-b19-code-r1-opus-model/`）。

被判的产品代码：`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/instance_table_sweep.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/root_record.rs`。

## 一、核查

- 开工快照倒推：`python3 scripts/agents/snapshot-tree.py m3-close-b19-code-r1 --out ~/.local/share/singlefs/work/m3-close-b19-code-r1-judge/tree`：「清单点名 16 份，倒推出 16 份，sha256 对上 16 份」。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-code-r1-sonnet-output.md research/prompts/m3-close-b19-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-code-r1-judge/tree --background research/prompts/_m3-close-b19-code-r1-background.md`：「核了 70 处引文、100 处裸行号，对上 45 处，没判 125 处」，判了的 0 处对不上；判决用到的裸行号主 agent 现读。
- 现读坐实 T4 的次序：`crates/singlefs-core/src/mount.rs:7113` 先调 `refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`，`:7125` 才算 `devices_rewritten_by_the_acquisition_after_checking_each_device`；`:7101`–`7108` 构造 `PublishesPlannedAfterAcquisition` 时 `devices_rewritten_by_the_acquisition: Vec::new()`；`:7693` 字段文档自写「取号之前的逐盘核在预演之后才判，预演那一趟按空表走」；`:5752` 缺空间转只进内存的盘走预演；`:5705` 重走到上限仍读坏就拒。
- 复跑：攻方模型 `tools/rerun.sh`。本机内存包装排不上，经 `scripts/hosts/multi-host-run.sh --peer --memory-cap 16G` 拿主工作区（与快照 16/16 相同）的拷贝当冻结树跑（日志 `~/.local/share/singlefs/work/m3-close-b19-code-r1-judge/rerun.log`），退 0：`name=probe_output_identical lines=5`、`name=probe_output_with_the_reorder_identical lines=5`。数是副本上的探针量的，不进 kb；入库形态是下一批的会红用例。
- 核不动的：攻方 P4 自报 I-9.16 ④、I-3.20 两处没读完；正推腿 T1–T6 全靠读码，没编没跑。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| T1 C773 | **相符**（两腿都没打中）。条款没写的：D16（发布语义） 已定项 1「槽号 0 的钉值是第 0 代根的 0」一句与平局取区域号最小的槽，只在代码注释里，随批末写回 | 正推 T1、攻方 T1 |
| T2 C724 | **采攻方与正推一致指出的越位**：core 判决 R2 定「共享函数保留这个错（有外部调用方）」，今天 `mount.rs:4430` 的 assert 让公开入口 `raise_rollback_floor_to_the_admission_ceiling` 在本该交回 `Err(NoValidRoot)` 的地方 panic。改回：公开入口交回错误，`expect` 只留在本进程准入路径那一处；补一条用例钉公开入口交回 `NoValidRoot`。前提 b 已有断言（`mount.rs:4412`）；正推腿另指的隐含前提（现行根不被它自己的实例表判成被抛弃）连同 D23（journal 的角色与格式） 已定项 14 两条前提，随批末写回 | 正推 T2、攻方 T2；主 agent 现读 `mount.rs:4412`、`:4430` |
| T3 C928 | **相符**（两腿都没打中，一次抽样）。冻结解除之后积下的 discard 何时发，条款没写，补一条用例钉「解冻后下一次进 discard 调用点就发、不丢」；攻方顺带看到的冻结期间撤排他那一处是别的会话的代码，不在射程 | 正推 T3、攻方 T3 |
| T4 C874 | **打中，采**：逐盘核在续做预演之后、预演按空的补写盘表走，续做缺空间连同取号预演时两槽全废那块盘被拒在取号之前，补写闸与拒因载荷都走不到（副本量过、复跑对上）。下一批：先在 `crates/singlefs-harness/tests/` 写会红用例（窄盘镜像盘 1 两槽写 0、续做缺空间那一形，断言挂载做成且 `devices_rewritten_by_the_acquisition == [DeviceIdentity(1)]`），今天红在「取号之前拒、预演读坏」；再改：逐盘核挪到预演之前（改法一），改完量预演是否仍因补写盘两槽读坏重走到上限，仍会就加改法二（补写盘两槽的读坏不记账）；钉读序数的用例按新次序重核。两个改法都零轮，进第二轮代码轮 | 攻方 T4；复跑；主 agent 现读 |
| T5 C690 | **相符**。越过 365 不清扫的三形（树表 0 条、清扫交回 `NoRowDeleted`、开关开着）条款没写，后果只是下次预留按两片留（保守），记进 C690 写回；`DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE` 文档「链在一片之内收回」外推，改成说清扫触发阈值。**两处不符采**：`.claude/kb/checks-owed.md` C690 行还写占位 369；`crates/mutations.tsv` 第 2266 行替换文引 `PLACEHOLDER_UNTIL_C690`，crates 里零命中，这一行判不了，并进下一批改指新常量名 | 正推 T5、攻方 T5；主 agent 现查 `grep -rn PLACEHOLDER_UNTIL_C690 crates --include=*.rs` 零命中 |
| T6 | **相符**（只有 T2 那一处）；`ring_slots_self_verified_on_only_one_of_the_two_root_ring_passes`、`ring_slots_recorded_as_holding_an_abandoned_root_by` 只是 hunk 头的上下文名，本身没动 | 正推 T6、攻方 T6 |
| C929 | 两腿一致读出今天的行为：回卷之后那块盘有自证槽、走落后支，补写闸三、四、五不再判。该不该再过闸条款没写，进第二轮代码轮正文当一格设计判断，判不下来交用户 | 正推 T4、攻方 T4 |
| P1 C934 | **没打中（一次抽样）**：唯一反例形要 FUA 被丢，且只会让 checker 少红；照规则「没打中」一次不算，第二轮再抽 | 攻方 P1 |
| P2 C937 | **没打中（一次抽样）**：造出的那一形与单缺陷镜像逐字节相同，判别子观测不到；第二轮再抽 | 攻方 P2 |
| P3 | **没打中（一次抽样）**；第二轮再抽 | 攻方 P3 |
| P4 | **没打中（一次抽样），没读完**：I-3.1 与 I-3.18 读分配记录走的路径不同；I-9.16 ④、I-3.20 两处没读；第二轮专攻这一格 | 攻方 P4 |

## 三、交用户的

无（C929 先进第二轮当一格判，判不下来再交）。

## 四、被攻过零轮的

T4 改法一与改法二、T2 公开入口改回交回错误的改法、C928 解冻后发 discard 的用例——下一批实现交回后进第二轮代码轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-29 不受影响：T1 相符，补的一句随 core 判决 R1 那一批写回 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-29 不受影响：T2 的改法是让实现回到 core 判决 R2 已定的形态，射程两条前提随批末写回 |
