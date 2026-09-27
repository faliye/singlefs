# E158 第 4 次跑 第五段（几何敏感性）执行员报告

写于 2026-09-27 10:50 JST。登记 `research/prompts/e158-r4-prereg.md` 8.2（r3 登记第 525–540 行，r3 登记 sha256 核过 = `5520bf1e…fa37e`）、11.2；5.5 与第十二节两段「主 agent 认定」照办。装置与臂副本照 `research/prompts/e158-r4-device-runner-report.md` 第五节用，一个字节没改。

## 一、结论

- 18 条臂全跑完（主族 12 条跑 G_S4、G_S16、G_默认、G_小环、k = 2；6 条 `-three-rounds` 跑 R = 3），退出码都是 0，没有 `name=stop`，18 份开跑检查（`r4_arm_code_check`、常量回比、7.2 锚点）全过。另跑了一次 `r4-compare`（seg5 前缀），得 Q4 与 Q6 的跨臂数。
- **判决格逐格比 G0：210 格，全部「不翻」**（丢写 120 格、Q4 多拒 66 格、Q6 24 格）。判别力自证：12 格两点值不同（全在 k = 2 与 R = 3 上），门槛挪到两值之间，12 格都由「一致」转成「翻面」，自证过；其余 198 格两点同值，自证不适用。
- **要主 agent 看的（第六类的实质问题）：G_S4、G_S16、G_默认、G_小环 四个点没有把机制推到登记写的那个方向。** 在 H1d 的 `none` 格与 H1e 上，这四个点上被藏那条根的去向（Q0）与 Q1 两栏的分布同 G0 逐类一样（第四节贴了数）；G_小环 取到的是第一个试的 48 KiB，但 4 个前缀都是 `ring_not_wrapped=true`、64 格都是 `device_undecidable=false`，「记录绕环、N-配置 走判不出那一支」一次都没发生。所以这四个点上的「不翻」是「两点同值」，不能读成「判决在根环大小、环长这条轴上稳定」。推的原因（没量过）：前缀最多 6 次发布（`highest_counter` ≤ 6），根槽按 (txg mod 3, (txg div 3) mod S) 落，txg div 3 ≤ 2 < 4，S = 4、8、16 落点不变；环最小 12 个记录槽，也写不满。k = 2 与 R = 3 **推动了**机制（第四节）。
- 推翻条件：G0 那一侧用的是第一段、第四段的产物（它们的执行员还没交回）；那几份产物若被换掉或作废，G0 那一列要重取、这张表重算。另，若有一份更长的前缀（发布次数让根槽在 S = 4 上回绕、让记录在小环上回绕）把某条臂的丢写格从 0 推到 > 0，或把 -槽 臂推到 0，就是翻面。

## 二、跑了什么（原样命令）

每条臂（`/tmp/claude-1000/e158-r4-seg5/run-one.sh <臂>`，3 条并行、每条单线程）：

```
cd /tmp/claude-1000/e158-r4-device/arms/<臂>
SINGLEFS_E158_ARM=<臂> SINGLEFS_E158_SNAPSHOT_SHA256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b SINGLEFS_E158_TODAY_BASES=/tmp/claude-1000/e158-r4-device/today-bases \
  nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/capped.sh 3 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 10G ./target/release/e158_root_choice_repair r4-seg5 > research/results/e158-root-choice-repair-2026-09-27-r4-seg5-<臂>.out
```

跨臂（today 副本里）：同一串环境变量，`… e158_root_choice_repair r4-compare /home/fy5090/code/singlefs/research/results/e158-root-choice-repair-2026-09-27-r4-seg5 > research/results/e158-root-choice-repair-2026-09-27-r4-seg5-compare.out`，rc=0。

开跑前 `ps` 看到第一段（`r4-seg1`，xargs -P 3）、第三段（`r4-seg3`）的装置与别人的 `cargo test`、一个 `gate.sh`（另一个仓）在跑；没有 qemu、vm-bench、e152、fio。没等锁。18 份臂二进制的 sha256 开跑前记下（`arm-binaries.sha256`），跑完 `sha256sum -c` 18 个 OK。耗时（`progress.md`，JST）：today 40 秒，甲 / 丙 各约 25–36 秒，乙 与 乙-窄读 主族各 276–417 秒，六条 R = 3 各 5–7 秒；10:24:02 起、10:43:26 止。

## 三、产物（`research/results/`，sha256 前 16 位、行数、末行）

- `95705407d9d53911` e158-root-choice-repair-2026-09-27-r4-seg5-bing-cfg-carry.out lines=1206 last="E7RESULT name=done emitted=1205"
- `21ddbe4a765ba2ca` e158-root-choice-repair-2026-09-27-r4-seg5-bing-cfg.out lines=1206 last="E7RESULT name=done emitted=1205"
- `470c4b5486aedc1d` e158-root-choice-repair-2026-09-27-r4-seg5-compare.out lines=762 last="E7RESULT name=done emitted=761"
- `b14ee340564722ee` e158-root-choice-repair-2026-09-27-r4-seg5-jia-cfg-carry.out lines=1206 last="E7RESULT name=done emitted=1205"
- `f89e6ca3a75a09fc` e158-root-choice-repair-2026-09-27-r4-seg5-jia-cfg.out lines=1206 last="E7RESULT name=done emitted=1205"
- `a8af15cabe354712` e158-root-choice-repair-2026-09-27-r4-seg5-jia-slot.out lines=1206 last="E7RESULT name=done emitted=1205"
- `c6792e9eb571ebcb` e158-root-choice-repair-2026-09-27-r4-seg5-today.out lines=1206 last="E7RESULT name=done emitted=1205"
- `b23e0d2382a01926` e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-carry.out lines=1659 last="E7RESULT name=done emitted=1658"
- `c982a54f6aa7f7ed` e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-carry-three-rounds.out lines=583 last="E7RESULT name=done emitted=582"
- `25cfb5e3ff9f30d7` e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg.out lines=1659 last="E7RESULT name=done emitted=1658"
- `9e05903f87cf0529` e158-root-choice-repair-2026-09-27-r4-seg5-yi-cfg-three-rounds.out lines=583 last="E7RESULT name=done emitted=582"
- `522204e9f83cbeaf` e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-carry.out lines=1659 last="E7RESULT name=done emitted=1658"
- `a28025c92a42e735` e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-carry-three-rounds.out lines=583 last="E7RESULT name=done emitted=582"
- `e51132d1f84d5be9` e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg.out lines=1659 last="E7RESULT name=done emitted=1658"
- `a85022448ecbcd78` e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-cfg-three-rounds.out lines=583 last="E7RESULT name=done emitted=582"
- `79b6887001e4ca55` e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-slot.out lines=1659 last="E7RESULT name=done emitted=1658"
- `587269dd1a768c8f` e158-root-choice-repair-2026-09-27-r4-seg5-yi-narrow-slot-three-rounds.out lines=583 last="E7RESULT name=done emitted=582"
- `4e28e10982072c52` e158-root-choice-repair-2026-09-27-r4-seg5-yi-slot.out lines=1659 last="E7RESULT name=done emitted=1658"
- `00af91121b94aea1` e158-root-choice-repair-2026-09-27-r4-seg5-yi-slot-three-rounds.out lines=583 last="E7RESULT name=done emitted=582"

每份第一行都是 `E7INPUT name=crates_snapshot key=E158 sha256=bb5ca9bef0b09eb957906317adf7d2b47567d901ef341e85c14b6fe23a37cf4b`（`head -1 … | grep -c bb5ca9…` = 18）。完成标记按新文件名认：18 份臂产物末行 `name=done`，compare 末行 `name=done emitted=761`。

## 四、自查齐不齐、机制有没有被推动

**齐不齐**（命令：对每份臂产物数 `name=r4_family_summary` 行数与 `r4_l_cell` 按 族@几何 的格数）：

```
主族 12 条每条：summaries=9 l_cells=l2@S16:16,l2@S4:16,l3@G_default:24,l3@G_small_ring:24,l5@S16:72,l5@S4:72,
-three-rounds 6 条每条：summaries=2 l_cells=（空）
```

对得上 8.2 取样点表：S4/S16 各有 H1d 的 `none`、H1e、L2、L5；G_默认 有 H1d 的 `none`、H1e、L3；G_小环 有 H1e、L3；k = 2 有 H1d 的 `none`、H1e；R = 3 只乙 / 乙-窄读 六条、只 H1d 的 `none` 与 H1e。每臂 9 个族摘要分别是 h1d-uncut ×3 几何、h1e ×4 几何、h1d-uncut-k2、h1e-k2。

k = 2 上 n1 = 0 的组打 `verdict=not_applicable`（today 8 行，乙 各臂 12 行），样例原样：
`E7RESULT name=r4_group family=h1d-uncut-k2 arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) n1=0 c=C b=unit_first_copy form=read_fails duration=W verdict=not_applicable`
k = 2 因此每族 14 组、H1d 42 格 / H1e 56 格（G0 是 16 组、48 / 64 格）。为什么 n1 = 0 不适用我没读代码核（推的：n1 = 0 的前缀只有一条可藏的新根）。

**G_小环 取到的环长与绕没绕环**（today 产物原样）：
```
E7RESULT name=r4_small_ring_try ring_bytes=49152 runs_through=yes
E7RESULT name=r4_small_ring arm=today geometry=G_small_ring(S=8,ring=49152) verdict=found
E7RESULT name=r4_prefix family=h1e arm=today geometry=G_small_ring(S=8,ring=49152) n1=3 c=C tip=1:6 hidden_slot=0:2 last_confirmed=4 base_fingerprint=6cf9329a4815a0e3 highest_counter=6 ring_not_wrapped=true tail_checks=4 tail_check_failures=0
```
四个 n1 的前缀 `highest_counter` 是 3、4、5、6，全部 `ring_not_wrapped=true`。H1e@G_小环 64 格 `device_undecidable=false` 64 格；L3@G_小环 24 格里 16 格 `device_undecidable=true`，L3@G_默认 也是 16 / 24——同数，不是小环带出来的。

**去向分布**（命令：按 臂 × 族 × 几何 数 `q0_destination` 与 Q1 两栏是否 > 0，H1d 只取 `cut=none`；seg1 = G0）。today 的：

| 族@点 | 环在且被抛弃（Q1 环内 0 / >0） | 根槽被盖（Q1 环外 0 / >0） |
|---|---|---|
| h1d none @G0（seg1） | 24 / 8 | 12 / 4 |
| h1d none @S4、@S16、@G_默认 | 24 / 8（三个点一样） | 12 / 4（三个点一样） |
| h1e @G0 | 0 / 32 | 0 / 32 |
| h1e @S4、@S16、@G_默认、@G_小环 | 0 / 32（四个点一样） | 0 / 32（四个点一样） |
| h1d none @k=2 | 36 / 6 | 0 |
| h1e @k=2 | 8 / 48 | 0 |

jia-slot 与 yi-slot 同样：S4、S16、G_默认、G_小环 上五类去向的格数与 G0 逐类相同（例 yi-slot h1d：被择或被施加 16、环在且被抛弃 15+5、原样留着 16、根槽被盖 9+3，四个点都是这组数）；k = 2 上「根槽被盖」变成 0。**这就是第一节说的：S 与环长两条轴在这组前缀上没被走到。**

R = 3 推动了机制（yi-cfg-three-rounds 对比 yi-cfg@S4 的被测挂载结局，`a_class`）：
```
R=3  h1d-uncut-r3: O1only K0 16, T1 K0 16, T2 K0 16, T3 K0 16, T4 K3 16, W K3 16；h1e-r3: T1/T2/T3 K0 各 64, T4/W K3 各 64
R=1  h1d-uncut@S4: O1only K0 16, T1 K0 16, T2 K3 16, W K3 16；h1e@S4: T1 K0 64, W K3 64
```
六条 R = 3 臂 2496 格被测挂载全部 `truth_newer=true`，Q4 按定义（臂拒 ∧ N_真 为假）在这些格上不会有多拒；compare 的 `r4_q4` 不含 R = 3 臂（族名 `-r3` 在今天那一臂上没有对应格），这一格的 Q4 由定义给出、不是 compare 算的。

## 五、逐格「翻面 / 不翻」与判别力自证（原样行）

算法（`/tmp/claude-1000/e158-r4-seg5/flip_table.py`，sha256 `eb364b66a2a2a2d4…`；只读 `research/results/` 的产物，可重跑）：
- 丢写格 = 每格 `lost_write_cell=true` 或 `q2_rollback=true`（登记第六节 Q1∪Q2∪Q3）。**注意**：产物里逐格的 `lost_write_cell` 字段只含 Q1 与 Q3（装置 `CellLoss::lost_write_cell`），不含 Q2；族摘要的 `lost_write_cells_q1_q2_q3` 含 Q2。只数逐格字段会在 h1e-k2 上少数 8 格（today 48 对 56）；加上 Q2 之后与 compare 的 120 行 `r4_loss` 逐行相同（`diff` 空）。G0 的 H1d 只取 `cut=none`（seg1 产物）；加不加 Q2 在 seg1 上结果一样。
- G0 值：丢写取 `…-r4-seg1-<臂>.out`；Q4 取 `…-r4-seg4-compare.out` 的 `r4_q4`（l2、l3、l5 @G0）；Q6 取 `…-r4-seg4-compare.out` 的 `r4_q6`。R = 3 臂的 G0 取同名去掉 `-three-rounds` 的那条臂。
- 判定：丢写与 Q6 按「> 0」判，Q4 按取值本身（两候选的先后）判；两点值不同就把门槛挪到两值中点再判一次，必须翻。
- 汇总（命令 `awk '{q="";v="";for(i=1;i<=NF;i++){if($i~/^quantity=/)q=$i; if($i~/^verdict=/&&v=="")v=$i}; print q, v}' flip-table.txt | sort | uniq -c`）：
```
    120 quantity=lost_write_cells verdict=not_flipped
     66 quantity=q4_extra_refusals verdict=not_flipped
     24 quantity=q6_not_covered_cells verdict=not_flipped
```
  自证：`grep -o 'discrimination=[a-z_]*\|verdict=flips\|verdict=does_not_flip_VOID' flip-table.txt | sort | uniq -c` → `12 discrimination=threshold_moved_to`、`198 discrimination=two_points_same_value_self_check_not_applicable`、`12 verdict=flips`，`does_not_flip_VOID` 0 行。

### 5.1 丢写（120 行）

```
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1d->h1d-uncut-k2 point=k=2 g0=0/48 point_value=0/42 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg family=h1e->h1e-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1d->h1d-uncut-k2 point=k=2 g0=0/48 point_value=0/42 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=bing-cfg-carry family=h1e->h1e-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1d->h1d-uncut-k2 point=k=2 g0=0/48 point_value=0/42 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg family=h1e->h1e-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/48 point_value=0/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1d->h1d-uncut-k2 point=k=2 g0=0/48 point_value=0/42 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-cfg-carry family=h1e->h1e-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=24/48 point_value=24/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1e->h1e point=S4(S=4,ring=3MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=24/48 point_value=24/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1e->h1e point=S16(S=16,ring=3MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=24/48 point_value=24/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1d->h1d-uncut-k2 point=k=2 g0=24/48 point_value=21/42 verdict=not_flipped discrimination=threshold_moved_to=22.5 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=jia-slot family=h1e->h1e-k2 point=k=2 g0=32/64 point_value=28/56 verdict=not_flipped discrimination=threshold_moved_to=30.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=48/48 point_value=48/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1e->h1e point=S4(S=4,ring=3MiB) g0=64/64 point_value=64/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=48/48 point_value=48/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1e->h1e point=S16(S=16,ring=3MiB) g0=64/64 point_value=64/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=48/48 point_value=48/48 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=64/64 point_value=64/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=64/64 point_value=64/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1d->h1d-uncut-k2 point=k=2 g0=48/48 point_value=42/42 verdict=not_flipped discrimination=threshold_moved_to=45.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=today family=h1e->h1e-k2 point=k=2 g0=64/64 point_value=56/56 verdict=not_flipped discrimination=threshold_moved_to=60.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1d->h1d-uncut-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg family=h1e->h1e-k2 point=k=2 g0=0/128 point_value=0/112 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1d->h1d-uncut-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry family=h1e->h1e-k2 point=k=2 g0=0/128 point_value=0/112 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1d->h1d-uncut-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg family=h1e->h1e-k2 point=k=2 g0=0/128 point_value=0/112 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1e->h1e point=S4(S=4,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1e->h1e point=S16(S=16,ring=3MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=0/64 point_value=0/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=0/128 point_value=0/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1d->h1d-uncut-k2 point=k=2 g0=0/64 point_value=0/56 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry family=h1e->h1e-k2 point=k=2 g0=0/128 point_value=0/112 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1e->h1e point=S4(S=4,ring=3MiB) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1e->h1e point=S16(S=16,ring=3MiB) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1d->h1d-uncut-k2 point=k=2 g0=32/64 point_value=28/56 verdict=not_flipped discrimination=threshold_moved_to=30.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot family=h1e->h1e-k2 point=k=2 g0=64/128 point_value=56/112 verdict=not_flipped discrimination=threshold_moved_to=60.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1d->h1d-uncut point=S4(S=4,ring=3MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1e->h1e point=S4(S=4,ring=3MiB) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1d->h1d-uncut point=S16(S=16,ring=3MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1e->h1e point=S16(S=16,ring=3MiB) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1d->h1d-uncut point=G_default(S=8,ring=768MiB) g0=32/64 point_value=32/64 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1e->h1e point=G_default(S=8,ring=768MiB) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1e->h1e point=G_small_ring(S=8,ring=49152) g0=64/128 point_value=64/128 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1d->h1d-uncut-k2 point=k=2 g0=32/64 point_value=28/56 verdict=not_flipped discrimination=threshold_moved_to=30.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot family=h1e->h1e-k2 point=k=2 g0=64/128 point_value=56/112 verdict=not_flipped discrimination=threshold_moved_to=60.0 g0_above=True point_above=False verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry-three-rounds family=yi-cfg-carry:h1d->h1d-uncut-r3 point=R=3 g0=0/64 point_value=0/96 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-carry-three-rounds family=yi-cfg-carry:h1e->h1e-r3 point=R=3 g0=0/128 point_value=0/320 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-three-rounds family=yi-cfg:h1d->h1d-uncut-r3 point=R=3 g0=0/64 point_value=0/96 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-cfg-three-rounds family=yi-cfg:h1e->h1e-r3 point=R=3 g0=0/128 point_value=0/320 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry-three-rounds family=yi-narrow-cfg-carry:h1d->h1d-uncut-r3 point=R=3 g0=0/64 point_value=0/96 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-carry-three-rounds family=yi-narrow-cfg-carry:h1e->h1e-r3 point=R=3 g0=0/128 point_value=0/320 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-three-rounds family=yi-narrow-cfg:h1d->h1d-uncut-r3 point=R=3 g0=0/64 point_value=0/96 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-cfg-three-rounds family=yi-narrow-cfg:h1e->h1e-r3 point=R=3 g0=0/128 point_value=0/320 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot-three-rounds family=yi-narrow-slot:h1d->h1d-uncut-r3 point=R=3 g0=32/64 point_value=48/96 verdict=not_flipped discrimination=threshold_moved_to=40.0 g0_above=False point_above=True verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-narrow-slot-three-rounds family=yi-narrow-slot:h1e->h1e-r3 point=R=3 g0=64/128 point_value=160/320 verdict=not_flipped discrimination=threshold_moved_to=112.0 g0_above=False point_above=True verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot-three-rounds family=yi-slot:h1d->h1d-uncut-r3 point=R=3 g0=32/64 point_value=48/96 verdict=not_flipped discrimination=threshold_moved_to=40.0 g0_above=False point_above=True verdict=flips
name=seg5_sensitivity quantity=lost_write_cells arm=yi-slot-three-rounds family=yi-slot:h1e->h1e-r3 point=R=3 g0=64/128 point_value=160/320 verdict=not_flipped discrimination=threshold_moved_to=112.0 g0_above=False point_above=True verdict=flips
```

### 5.2 Q4 多拒（66 行；today 不在里面，Q4 本身是臂减今天）与 Q6（24 行）

```
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg-carry family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg-carry family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg-carry family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg-carry family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=bing-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg-carry family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg-carry family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg-carry family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg-carry family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-slot family=l2 point=S16(S=16,ring=3MiB) g0=8/16 point_value=8/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-slot family=l2 point=S4(S=4,ring=3MiB) g0=8/16 point_value=8/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-slot family=l3 point=G_default(S=8,ring=768MiB) g0=0/24 point_value=0/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-slot family=l3 point=G_small_ring(S=8,ring=49152) g0=0/24 point_value=0/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-slot family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=jia-slot family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg-carry family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg-carry family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg-carry family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg-carry family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg-carry family=l2 point=S16(S=16,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg-carry family=l2 point=S4(S=4,ring=3MiB) g0=0/16 point_value=0/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg-carry family=l3 point=G_default(S=8,ring=768MiB) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg-carry family=l3 point=G_small_ring(S=8,ring=49152) g0=16/24 point_value=16/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-slot family=l2 point=S16(S=16,ring=3MiB) g0=8/16 point_value=8/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-slot family=l2 point=S4(S=4,ring=3MiB) g0=8/16 point_value=8/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-slot family=l3 point=G_default(S=8,ring=768MiB) g0=0/24 point_value=0/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-slot family=l3 point=G_small_ring(S=8,ring=49152) g0=0/24 point_value=0/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-slot family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-narrow-slot family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-slot family=l2 point=S16(S=16,ring=3MiB) g0=8/16 point_value=8/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-slot family=l2 point=S4(S=4,ring=3MiB) g0=8/16 point_value=8/16 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-slot family=l3 point=G_default(S=8,ring=768MiB) g0=0/24 point_value=0/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-slot family=l3 point=G_small_ring(S=8,ring=49152) g0=0/24 point_value=0/24 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-slot family=l5 point=S16(S=16,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q4_extra_refusals arm=yi-slot family=l5 point=S4(S=4,ring=3MiB) g0=6/72 point_value=6/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=bing-cfg family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=bing-cfg family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=bing-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=bing-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=jia-cfg family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=jia-cfg family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=jia-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=jia-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=jia-slot family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=jia-slot family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=today family=l5 point=S16(S=16,ring=3MiB) g0=3/72 point_value=3/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=today family=l5 point=S4(S=4,ring=3MiB) g0=3/72 point_value=3/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-cfg family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-cfg family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-narrow-cfg family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-narrow-cfg family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-narrow-cfg-carry family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-narrow-cfg-carry family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-narrow-slot family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-narrow-slot family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-slot family=l5 point=S16(S=16,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
name=seg5_sensitivity quantity=q6_not_covered_cells arm=yi-slot family=l5 point=S4(S=4,ring=3MiB) g0=0/72 point_value=0/72 verdict=not_flipped discrimination=two_points_same_value_self_check_not_applicable
```

读法：
- 主族 12 条在 S4、S16、G_默认、G_小环、k = 2 上，丢写「= 0 / > 0」与 G0 全同：today、jia-slot、yi-slot、yi-narrow-slot 各点都 > 0；jia-cfg、jia-cfg-carry、yi-cfg、yi-cfg-carry、yi-narrow-cfg、yi-narrow-cfg-carry、bing-cfg、bing-cfg-carry 各点都 = 0。按登记第六节开头，H1d、H1e 对 -配置 各臂**不算判据**（N-配置 按构造判真），这些 0 只是照报；对 today 与 -槽 臂算判据。
- R = 3：乙 / 乙-窄读 的 -配置 四条仍 0，-槽 两条仍 > 0（H1d none 48/96、H1e 160/320，G0 R = 1 是 32/64、64/128）。
- Q4：每个 族@点 上每条臂的多拒数与 G0 同族逐臂相同（l2：-槽 三条 8、其余 0；l3：-配置 八条 16、-槽 三条 0；l5：甲、乙、乙-窄读 各臂 6，丙 两条 0），两候选之间的先后没有对调。
- Q6：today 在 S4、S16 上仍是 `not_covered_cells=3`（G0 也是 3）；其余 11 条臂三点都是 0。

## 六、判决行点名（第 4c 步）

`grep -c 'name=verdict'` 在 19 份产物上合计 0 行（没有叫 `name=verdict` 的行）。按字段扫：
- 臂产物里 `verdict=` 的取值只有 `pass`、`found`、`not_applicable`（每臂的计数：主族甲 / 丙 / today 241 pass + 8 not_applicable + 1 found；乙 四条 307 pass + 12 not_applicable + 1 found；R = 3 六条 144 pass）。`not_applicable` 全在 k = 2 的 n1 = 0 组上（第四节），登记预期内（拿不准：登记没写 n1 = 0 在 k = 2 上取不到，推的）。
- 违例 / 不匹配 / 失败类计数，19 份合计：`s4_mismatches` 132 行、`f4_failures` 120、`f6_findings` 120、`f13_failures` 132、`f15_findings` 120、`formula_mismatches` 120、`tail_check_failures` 480、`void_cells_v2` 120、`fault_not_reached_cells` 120、`differing` 18、`differences` 6、`mismatches` 2，**不为 0 的都是 0 行**。另：`f6_checked`、`f15_checked` 在第五段合计 0（这两项这一段没核，F6 / F15 归第一段），`f13_checked` 合计 464。
- compare 产物（`…-r4-seg5-compare.out`）点名：
  - 第 446–462 行 17 行 `name=r4_cross_arm_sentence … control=PC-extra-refusal sentence=today_ok_on_the_cell tag=today verdict=not_constructible n1=1,2,3`，第 463 行 `E7RESULT name=r4_cross_arm_sentence arm=today control=PC-22 sentence=an_m_with_ok_zero_count_zero_isolation tag=today verdict=not_constructible route=F2`；第 761 行 `r4_missed_arm_and_today_sentences count=18`。原因：PC-多拒 与 PC-22 的前提（G0 上的 L3(a)、G0 的 L5）在第四段，第五段前缀里没有；同一句在第四段 compare 里是 `verdict=pass n1=1`（`grep` 了 `…-r4-seg4-compare.out`）。标签都是 [今]，按 V1 ⑤ 不作废；`r4_void` 126 行全是 `voided=false`。这 18 句是「只拿第五段前缀跑 compare」的产物，不当判据。
  - `r4_q6 arm=today … not_covered_cells=3 verdict=not_covered` 两行（S4、S16）：登记预期内（PC-22 的 [今] 句钉住今天那一臂没罩住）。
  - 其余 `verdict=` 取值：covered 22、inference_holds 1、pass 45。

## 七、门禁（登记给 experiment-runner 的阶段，主工作区，10:47–10:49 JST）

| 阶段 | 退出码 | 原样末行（截 230 字） | 与第五段 |
|---|---|---|---|
| 33 | 0 | `✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1153 条的原文各命中源码一次；…` | 无关 |
| 52 | 1 | `再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。` | 日志里 E158 0 处 |
| 80 | 0 | `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs` | 无关 |
| 96 | 0 | `e92_reuse_requirement.rs` | 无关 |
| 27 | 1 | `→ 旧值只许留在「## 历史版本」之后与 *-history.md 里。` | 日志里 E158 0 处 |
| 34 | 0 | `✓ 实验索引行与正文标题一致（索引 159 行、正文 159 份）` | 无关 |
| 40 | 1 | 末行点名 E160 | **点名这一次的 19 份 seg5 产物**（日志第 52–70 行）：还没有实验页点名，按派发不写实验页，交主 agent |
| 69 | 1 | `原件已经没了，就写明它没了、…` | 日志里 E158 0 处 |
| 75 | 1 | `支撑 / 推翻的那条分项在「**依据**」段引回这个实验；…` | 日志里 E158 0 处 |
| 84 | 0 | `✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 3 个）` | — |
| 85 | 0 | `写明「原始输出未留存」不判的 0 页：（没有）` | — |
| 86 | 0 | `扫了 research crates/singlefs-harness/src/bin（target/ 不扫）；不在的目录 0 个：（没有）` | — |
| 88 | 77 | `! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判` | 本次未跑，不算过 |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（3 份、14 条路径）；…` | — |

原样日志：`/tmp/claude-1000/e158-r4-seg5/gate-<阶段>.log`。

## 八、登记修订

没有。产物跑之前没看出要补的；跑之后看出的（第一节第三条：G_S4 / G_S16 / G_默认 / G_小环 没推动机制；小环的取法取到的第一个环长写不满）按定义不改登记，交主 agent。

## 九、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行；这一段只核几何敏感性，不新增量）

| 那一样 | 状态（第五段能说的） | 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 丢写（today、-槽 臂算判据；-配置 臂在 H1d/H1e 不算判据） | 第一段 G0 的「= 0 / > 0」在 S4、S16、G_默认、G_小环、k = 2、R = 3 上**都不翻**（120 格）。但前四个点两点同值是因为机制没被推动（第一、四节），这四点对根环大小、环长那条轴**不够判**；k = 2、R = 3 两点够判 | 实七、实八、H-随（第二段）与 H1f、H1g（第三段）上的丢写都不做几何敏感性（登记 8.2：H1f/H1g 已是 G0 主族，H-随 不做）；要不要为前四个点另立一个能让根槽 / 记录回绕的更长前缀，是登记层面的事，交主 agent | 能：更长的前缀若让根槽在 S = 4 上回绕、记录在小环上回绕，-配置 臂可能在 H1e 走到 N-配置 判不出那一支（推的，没量） | `python3 /tmp/claude-1000/e158-r4-seg5/flip_table.py`（`quantity=lost_write_cells` 行）；compare `name=r4_loss` |
| 多拒（Q4） | l2、l3、l5 在 S4/S16、G_默认/G_小环 上每臂的多拒数与 G0 逐臂相同，候选间先后不对调（66 格）；R = 3 上按定义为 0 | 同上一格的「机制没被推动」对 l2、l5（S 轴）、l3（环长轴）同样成立（l3 的 `device_undecidable` 在 G_默认 与 G_小环 上都是 16/24） | 能（同上） | flip_table.py 的 `quantity=q4_extra_refusals`；compare `name=r4_q4` |
| 多读（Q5-同成） | 登记 8.2 的判决格不含 Q5，这一段不判 | — | — | compare `name=r4_q5`（260 行，照报，没看） |
| 第 22 条那一格（Q6） | S4、S16 上与 G0 同：today 3 格没罩住，其余 11 臂 0（24 格） | Q6 对候选臂不算判据（登记第六节） | 不能（它对候选臂由定义推得） | flip_table.py 的 `quantity=q6_not_covered_cells`；compare `name=r4_q6` |
| 够判条件第二半（判据那一样今天有没有） | 不归这一段 | — | 不能 | 装置执行员报告修订第 3 条 |

## 十、没做什么

- 没写实验页、索引行、`experiments-history.md`，没改 `replay.sh`（派发要求）。门禁 40 号因此点名这 19 份新产物：`e158-root-choice-repair-2026-09-27-r4-seg5-*.out`（18 臂 + compare）还没被实验页点名，交主 agent 派人点名。
- 翻面表是草稿目录里的脚本从仓里产物算的，脚本本身不在仓里；要进实验页当依据的话，要么把它落进 `research/`，要么在实验页里写成可重跑的命令。
- G0 那一列读的是第一段、第四段执行员写的产物（它们还没交回，我只读、没核它们自己的判定）。
- 没判任何候选出局或胜出；没跑重型测试、没跑门禁全量、没提交；没碰 `crates/`。
- 没做登记外的补点（更长前缀）：那是新一次跑的范围。

## 十一、草稿目录（`/tmp/claude-1000/e158-r4-seg5/`）

没建编译目录、工作树或仓副本。里面只有：`run-one.sh`、`count_loss.sh`、`count_loss_with_rollback.sh`、`flip_table.py`、`flip-table.txt`、`loss-seg5.txt`、`loss-seg1-g0-none.txt`、`a.txt`/`a2.txt`/`b.txt`（与 compare 对比用）、`arm-binaries.sha256`、`products.txt`、`progress.md`、`rc-<臂>`、`stderr-<臂>.log`（18 份都是空的）、`gate-*.log`、这份报告。臂副本 `/tmp/claude-1000/e158-r4-device/arms/` 不是我建的，没动。
