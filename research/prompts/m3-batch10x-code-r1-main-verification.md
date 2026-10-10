# 根环实现第 10x 批：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G1 R1 R2 R3 R4 -->

正文 `research/prompts/_m3-batch10x-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10x-code-r1-sonnet-output.md`，sha256 df9b38c3…615d；
- 攻方：`research/prompts/m3-batch10x-code-r1-opus-output.md`，sha256 c629bbcd…5e5d，347 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10x-code-r1/tree/` 对 `research/prompts/m3-batch10x-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过 |
| 引文 | `cite-check.py` 两份报告合跑：`✓ 核了 56 处引文，对上 56 处，没判 0 处` |
| 主 agent 现读 | 冻结副本 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1482–1483 行：`reported_cards_by_pci_bus()` 与 `memory_used_by_this_process_on_each_bus()` 分两次调用。`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2595–2596 行：`capacities` 全 0 时 `scratch` 取 `None`；第 2634–2638 行接着报「没有一张卡装得下这条流的表…」判红 |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 式子取的量 | 两腿量到同一形：空闲与「本进程占着的」分两次读，两次之间本进程另一条线程建上卡的那一笔被算两遍。正推最大多借 2108 MiB，攻方模型最坏 12288 MiB。<br>攻方另两形：<br>- 本进程没走卡池的设备，被当成借了已经占上的那一笔（最坏 6783 MiB）；<br>- 「空着时能借的」拿总量当上界，空卡读到的空闲比总量少 497 MiB，有一段要的量永远报本次未跑。<br>正推另一形：已销账、卡上还没还的缓冲被当成别的借据已建好的部分（算例多借 4000 MiB） | **打中**。单进程内的事，不属于用户认的跨进程竞态。主 agent 读码坐实「分两次读」 |
| K1-2 跨进程删干净 | 两腿：产品代码里删干净了，只剩旧注释与 `lease_card_waiting_for_room` 这个过时的名字 | 没打中，注释与名字并进下一批 |
| K1-3 兜底各路 | 两腿都找到兜底漏的地方，都没找到「判成通过」的形：<br>- 正推 C：起设备失败判红，`OnceLock` 把 Err 存下来，之后在那张卡上每次借都红；<br>- 攻方 R1：卡池起设备与编 CRC 管线没包住；<br>- 攻方 R2：判器内核在另起的线程上编，错误范围按线程，接不住；<br>- 攻方 R3：只排计划那一趟试判没包住；<br>- 攻方 R4：致命的 Out of Memory 让 wgpu 丢掉整个进程共用的设备，之后借这张卡的用例全红。<br>四处都是 panic 判红，经内存包装时落进 245，这条命令整条作废 | **打中**：兜底没接全 |
| K2-1 单进程给大 | 两腿：就是 K1-1 那几形 | 同 K1-1 |
| K2-2 放得下却判红 | 两腿量到同一形：每张卡此刻都放不下一个状态，而卡空着时放得下，第 ① 段预检容量全 0，直接判红；账里把「此刻带不了」记成 `NothingToPlace`，也落进判红那一支。攻方另一形：`lease_card_waiting_for_room` 在同进程兄弟借着时 panic（`gpu_unit_checks.rs:1165`）。重排压到一张卡：两腿没打中 | **打中**，违反「不压并发」，应报本次未跑。主 agent 读码坐实第 2596 行那一支 |
| K3-1 有卡用例判据 | 正推：分得出「借了又还」，但第 ① 段只在判器建好那一刻采样一次。攻方：分不出「建好判器之后、判完之前还」，推的 | 窄缺口，并进下一批 |
| K3-2 扫源码绕法 | 正推：`#[cfg_attr(test, test)]` 绕得过（读码）；`GpuCard` 是 Clone，能活得比借据久；七份名单之外的文件不扫。攻方：没攻 | 打中（一条腿），并进下一批 |
| K3-3 变异行 | 正推：<br>- 自补 14 处，10 处被抓，C14（OOM 那句话）、C19、C20 没抓；<br>- 删掉的 20 行里「估计只取现读的空闲」那一形今天还在，有用例抓、没有变异行；<br>- OOM 兜底整路与 S5 改回去都不红（读码）。<br>攻方：白红那一处没有变异行 | **打中**，补变异行 |
| K4 与 S 交界 | 两腿：没有互相踩到 | 没打中 |

## 三、下一批（第 10y 批）

一簇，都在 `gpu_memory_ledger.rs`、`gpu_unit_checks.rs`、`crash_amplification.rs` 与两三份测试里：

1. **K1-1**：
   - 空闲与本进程占用一次读出，同一次 `nvidia-smi` 调用，或者用一次读出两个量的读法；
   - 本进程没走卡池的设备算进「本进程占着的」；
   - 已销账、卡上还没还的缓冲不当别人的已建部分；
   - 「空着时能借的」上界用空卡实测的空闲，不用总量。
2. **K1-3**：起设备、编 CRC 管线、判器内核编译（含另起的线程）、只排计划那一趟、设备丢失，这几处的 Out of Memory 都接住，报本次未跑。起设备失败不缓存成永久的 Err。
3. **K2-2**：每张卡此刻都放不下一个状态时，报本次未跑，不判红。`NothingToPlace` 与「此刻借不到」分开。同进程兄弟借着时不 panic。
4. **K3**：
   - 有卡用例在判完之前再核一次那一笔还在；
   - 扫源码堵 `cfg_attr(test, test)` 与七份名单之外的文件；
   - 补变异行：OOM 那一句、同卡给两次、卡数比池里多、白红那一处、「估计只取现读的空闲」。
5. 过时注释与 `lease_card_waiting_for_room` 改名。

GPU 用例自己的 Out of Memory 与对拍流归删除线第 34 批 U（对拍文件在它手上，已转）。

## 回看决策

不涉及决策：这一轮判的是 G1 二的实现与测试。G1 是实现形态的岔路，不是 kb 决策分项。
