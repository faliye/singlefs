# 根环实现第 10z 批第六块：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G2 H1 H2 H3 H4 H5 H6 X1 X7 P3 P5 F-K1 F-K3 F-H3 F-H4 -->

正文 `research/prompts/_m3-batch10z-f-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10z-f-code-r1-sonnet-output.md`，sha256 537dd7af…7b91，319 行；
- 攻方：`research/prompts/m3-batch10z-f-code-r1-opus-output.md`，sha256 1c46a1b8…4fba，252 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree/` 对 `research/prompts/m3-batch10z-f-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过；kb 对 `kb-sha256.txt` 全过 |
| 引文 | `cite-check.py` 两份报告合跑，`--root` 冻结副本、仓根、cargo registry 三个根：`✓ 核了 75 处引文，对上 75 处，没判 0 处` |
| 主 agent 现读 | 冻结副本 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1256–1286 行 `impl Drop for GpuCardLease`：`catch_unwind` 包着 `Wait { None, 30 秒 }`，没等成打一行 `GPU_CARD_LEASE_RETURN_WAIT_FAILED`、照旧记在账上，接着 `take_in_the_wait_before_returning`，`Err(panic)` 一律 `resume_unwind`（不看是不是已在展开）；`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 842–852 行：没等成的那几笔只在 `take_back_the_driver_share_of_a_lost_device`（设备丢了重起）时销 |
| 复跑 | 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1-main/rerun.sh`，两份冻结副本拷贝、各自的编译目录，rc 全 0。攻方 `out-a`、`out-b`、`mutants`、`unwind` 四份与模型目录 `output-*.txt` 去掉计时行之后逐字相同；正推 `sonnet_g2_model` `SEEDS=200000`：`test result: ok. 6 passed`、`K3-1 random: seeds=200000 violations=0`、`K1-2 round 0: room_by_ledger=23440 free_true=29400` 到 `K1-2 round 6: refused`、`K1-3 after: free_true=29400 room_by_ledger=16840 (safe_room=27440) lease A on books=true` |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs`、`crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 每条路先等 | 两腿：建过缓冲的销账路都先等，起设备失败那一路没有缓冲；攻方另报 H3：借据在展开中丢掉、等的那一下 wgpu 再 panic，`resume_unwind` 让进程中止（小程序 `UNWIND arm=as-shipped rc=134`，改成展开中不再抛 rc=0；wgpu 在丢了的设备上等会 panic 是读源码推的） | 站住；H3 打中（窄，要卡造），并进下一批 |
| K1-2 等失败那一支 | **两腿都量到**：等超时的那一笔一直记在账上，卡上的缓冲早还了；正推每笔 4000、第 7 笔起借不到，卡上真空闲 29 400；攻方两次超时之后账 0、真能借 29 440，到这张卡的设备丢了才回来。最坏整张卡借不出、借卡都报本次未跑。真卡上排着的活会不会超过 30 秒是推的 | **打中**（实现员自定第 1 处），违反 K3-2「放得下却本次未跑」与用户「不压并发」；并进下一批 |
| K1-3 丢了的设备两种状态 | 两腿：稳态不多给；**两腿都量到** 设备丢了那一刻活着的借据若没等成，`Remain` 状态出不来（攻方 6 段里 2 段：账 0、真 14 440，100 笔 1000 全拒；正推少 10 600），出口只有下一次丢设备。正推：触发重起的那一笔活着时少给全部已建缓冲加驱动。攻方：重起之前一下多给 6960，重起时照「占着的当 0」再判，记下 | **打中**（实现员自定第 2 处），并进下一批 |
| K1-4 旧口径拿干净 | 两腿：`returned_possibly_still` crates 里 0 处 | 站住 |
| K2-1 叫停记住 | 两腿：读叫停文件只有一处、包着的只增；正推换种子 24 000 段两样结局 0 | 站住 |
| K2-2 头一句 OOM 那一行 | 两腿：该打时打了。「这一趟有卡报过」比判决写的「这一轮」宽：攻方 91 段里 21 段只在宽读法下打、其中 6 段换成卡不报 OOM 也会被叫停截住；正推：OOM 没让块丢、照叫停收尾时也打 | 取实现员的宽读法：照叫停收尾的那一趟本来就提早收尾、分项驱动记成本次未跑，那一行说的是实情；不算打中 |
| K2-3 七条与两处抽出 | 两腿：harness 档这一批变异逐条改坏都红（正推 31/31、攻方 22/22） | 站住 |
| K3-1 多给 | 两腿：稳态不多给（正推 200 000 种子 0 次） | 没打中 |
| K3-2 不压并发 | 两腿：等设备只拿读锁，不压别的线程提交；等超时累加与 `Remain` 出不来就是 K1-2、K1-3 | 同 K1-2、K1-3 |
| K4 判别力 | 攻方 H4、正推第 4 点：碰卡用例先提交清零、读一次 `nvidia-smi` 约 0.05 秒才丢缓冲，丢时活已做完，照 E174「已做完」档 Poll 臂读数也不含，`crates/mutations.tsv` 第 3370 行（改回 Poll）多半照绿——推的，要卡核。攻方 H5：X1、X7 两处删掉 94 条照绿（量过）；H6：`catch_unwind`、`resume_unwind`、30 秒超时没有会红的用例与变异行。正推：探针 P3（重起时没等成的借据不从集合里去掉）、P5（第二次丢设备时已在 `Remain` 不重新记账，模型 144 / 20 000 红）整个模块照绿；checker 档接线五种改回去两条扫源码用例的复刻照绿；`GpuCardLease::drop` 等不等、结局记没记没有不要卡的测试 | **打中**，补用例与变异；H4 由主 agent 上卡核 |

## 三、下一批（第 10z 批第八块）

一簇，在 `gpu_unit_checks.rs`、`gpu_memory_ledger.rs`、两份新测试与 `crates/mutations.tsv`，底是第七块合入之后的主工作区：
1. K1-2：等超时的那一笔不永久少借。卡上的活做完了就销，不等到设备丢了。攻方 F-K1 是之后同一设备上有一次 `Wait { None }` 等成就一起销，只修了一半：两次超时之间没有别的等成时照样借不出。所以还要一条出路，例如借之前对记着没等成的那张卡 `Poll` 一次，`QueueEmpty` 就销。验收：攻方 `attack_10zf_ledger.rs` 的 `timeout-kept` 各行回到真值，正推 `sonnet_g2_model` 的 K1-2 不再 refused；随机种子多给仍是 0。
2. K1-3：设备丢了那一刻活着的借据没等成时，`Remain` 状态要出得来（攻方 F-K3）。F-K3 自报的风险是：旧设备另有不在账上的句柄时，可能多给最多那一份。用例要钉住不多给，钉不住就换形。正推 P3、P5 两形也要有会红的用例。
3. H3：已在展开中（`std::thread::panicking()`）只记没等成，不 `resume_unwind`（攻方 F-H3）。
4. K4：
   - 碰卡用例改成提交在飞的活之后紧接着丢（攻方 F-H4：空转约 250 毫秒、S ≥ 512 MiB，读数挪到提交之前），第 3370 行改回 Poll 要红；
   - X1、X7、`catch_unwind`、`resume_unwind`、30 秒超时、`GpuCardLease::drop` 记没记结局，各补会红的用例或抽成可测的纯函数，配变异；
   - checker 档接线扫源码用例照绿的五种改回去，各补到会红。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-08 不受影响：判的是 G2 乙的实现与测试，三条硬约束不变 |
