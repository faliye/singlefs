# 崩溃一致性验证报告：里程碑「第二个事务」增补 3 第 2 件 代码轮二

## 现场核对（跑之前）

- `cargo test --release --workspace`：exit=0，全部 test result 均 `0 failed`（日志 `research/prompts/m2-supp3-item2-crash-verifier/preflight-cargo-test.log`）。
- `grep -rn '\.pristine\|\.orig\|\.rej' crates/`：零命中（grep exit=1）。
- `.claude/gate.d/33-mutation-tables.sh`：exit=0，末行原样：
  `  ✓ 142 个实验二进制都有成形的变异表，1509 条变异的原文各命中源码一次；crates/mutations.tsv 173 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上）`
- 三项均与前一个实现员报告一致，未发现对不上，未上报主 agent。

## 环境旁证

- `command -v herd7`：exit=1（未找到）
- `ls -l /dev/kvm`：`crw-rw---- 1 root kvm 10, 232 Sep 19 22:38 /dev/kvm`，exit=0
- `command -v qemu-system-x86_64`：`/usr/bin/qemu-system-x86_64`，exit=0

## 起跑前负载检查

`ps -o pid,args` 未见 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`；未见别的会话在跑本仓 `.claude/gate.d/54-*`（唯一在跑的 `gate.sh` 属于 `/home/fy5090/code/singlefs-ai-sop-zh` 另一个仓，跑的是它自己的 `scripts/selftest.sh`，与本仓 54 号无关）。`cargo`/`rustc` 无残留进程。

## 阶段归属

`stage-owners.tsv` 登记给 `crash-verifier` 的阶段：54-layer0-replay.sh、55-qemu-first-transaction.sh、57-lkmm.sh、59-crates-mutation-replay.sh、74-model-differential.sh。本轮按派发只跑 54/55/57/59；74 号已由主 agent 处理，不在本轮范围内。

## 门禁 54 号：层 0 崩溃点重放全量（两条流）

- 指纹（开跑）：HEAD `cc3e8ba1f0fa9be2344fbd5f433a72fe8a27eb43`；`git diff HEAD -- crates litmus | sha256sum` = `7362795fa7280e4c6ff5932dc0b0d9c6648017ea1e8d7bf082b5f255b9d65ba0`；未跟踪文件哈希 = `9870062f62c46d413fc1ebdc287792bbffbbfe467a8adc4d02bf1cab7a05f812`。
- 开始（UTC）：2026-09-19 23:08:34；结束（UTC）：2026-09-19 23:16:53；耗时约 8 分 19 秒。
- 退出码：0
- 指纹（结束）：HEAD、diff sha256、未跟踪文件哈希三项与开跑时逐字相同（同一份源码）。
- 未见别的会话在跑本仓 `gate.d/54`，本次单起。

原样 ✓ 行（第一个事务那条流）：

```
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-1.3=262165/0 I-1.4=262165/0 I-1.6=262165/0 I-1.7=262165/0 I-2.1=262165/0 I-2.3=262165/0 I-2.4=262165/0 I-2.5=262165/0 I-3.1=4/0 I-3.8=262165/0 I-3.9=4/0 I-4.8=262165/0 I-5.1=262165/0 I-5.2=4/0 I-5.4=4/0 I-7.1=262165/0 I-7.2=262165/0 I-7.4=262165/0 I-7.6=262165/0 I-7.7=262165/0 I-7.8=262165/0 I-9.1=4/0 I-9.2=4/0 I-9.4=4/0 I-9.7=4/0 I-9.10=4/0 I-9.13=4/0 I-9.14=0/0
```

原样 ✓ 行（两次发布那条流，多版本 oracle）：

```
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=4 no_file=262158 file_read=1842255 failed=0 journal_differing=24 verification_ran=42 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 I-1.3=2104413/0/0 I-1.4=2104413/0/0 I-1.6=2104413/0/0 I-1.7=2104413/0/0 I-2.1=2104413/0/0 I-2.3=2104413/0/0 I-2.4=2104413/0/0 I-2.5=2104413/0/0 I-3.1=1842252/0/262161 I-3.8=2104413/0/0 I-3.9=1842252/0/262161 I-4.8=2104413/0/0 I-5.1=2104413/0/0 I-5.2=1842252/0/262161 I-5.4=1842252/0/262161 I-7.1=2104413/0/0 I-7.2=2104413/0/0 I-7.4=2104413/0/0 I-7.6=2104413/0/0 I-7.7=2104413/0/0 I-7.8=2104413/0/0 I-9.1=1842252/0/262161 I-9.2=1842252/0/262161 I-9.4=1842252/0/262161 I-9.7=1842252/0/262161 I-9.10=1842252/0/262161 I-9.13=1842252/0/262161 I-9.14=1580105/0/524308 first_violation=none
```

小结：两条流均 `exhaustive=true`，`violations=0`；第一个事务流 states=262165、第二个事务（两次发布）流 states=2104413；全部逐条不变量的判违例数（表中第二段）都是 0；两条流各起 32 个工作线程（本机 32 核，未显式设 `SINGLEFS_LAYER0_THREADS`）。日志全文：`research/prompts/m2-supp3-item2-crash-verifier/54.log`。

## 门禁 55 号：QEMU 真设备（第一个事务）

- 指纹（开跑）：HEAD `cc3e8ba1f0fa9be2344fbd5f433a72fe8a27eb43`；diff sha256 `7362795fa7280e4c6ff5932dc0b0d9c6648017ea1e8d7bf082b5f255b9d65ba0`；未跟踪文件哈希 `9870062f62c46d413fc1ebdc287792bbffbbfe467a8adc4d02bf1cab7a05f812`。
- 开始（UTC）：2026-09-19 23:17:37；结束（UTC）：2026-09-19 23:17:54；耗时约 17 秒。
- 退出码：0
- 指纹（结束）：三项与开跑时逐字相同。

原样 ✓ 行：

```
  ✓ QEMU 真设备上的第一个事务：3 次虚机跑、18 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方
```

逐项比对结果（脚本内部判据，均在这次跑里满足，见 `.claude/gate.d/55-qemu-first-transaction.sh` 第 86–96 行）：
- `direct` 模式：设备侧独立录制（QEMU blklogwrites）与程序自己的录制流逐项对得上（`verdict direct == 0`），且宿主从虚机写出的盘镜像上能读回文件（`name=host_recover outcome=file_read … content_matches=true`）。
- 对照 `skip-first-transaction-barrier`（漏一道屏障）：判红（`verdict == 1`），且红准确落在盘 0（`device=0 … divergence=at=`），盘 1 无差异（`device=1 … divergence=none`）——比对分得出「漏了哪一块盘的屏障」。
- 对照 `page-cache`（走页缓存）：判红（`verdict == 1`）——比对分得出页缓存回写合并/重排写与 O_DIRECT 逐个写的差异。
- 3 次虚机跑（direct / skip-first-transaction-barrier / page-cache）共 18 项检查全过（段序列、冷重开读回、事务计数与反向链、来宾块层 FLUSH 数 = 屏障+FUA、direct 设备侧比对、两个对照判红，逐项见脚本）。

## 门禁 57 号：herd7 / LKMM

- 指纹（开跑）：HEAD `cc3e8ba1f0fa9be2344fbd5f433a72fe8a27eb43`；diff sha256 `7362795fa7280e4c6ff5932dc0b0d9c6648017ea1e8d7bf082b5f255b9d65ba0`；未跟踪文件哈希 `9870062f62c46d413fc1ebdc287792bbffbbfe467a8adc4d02bf1cab7a05f812`。
- 开始（UTC）：2026-09-19 23:18:31；结束（UTC）：2026-09-19 23:18:31（同一秒内完成）。
- 退出码：0
- 指纹（结束）：三项与开跑时逐字相同。
- `.lkmm-static-only` 标记文件不存在（`ls` exit=2），所以本次是完整跑法，不是判别力样本的 `--static-only` 简化路径。
- 环境旁证：我这次会话的裸 shell 里 `command -v herd7` exit=1（未找到），但 `.claude/scripts/lkmm.sh` 自己找到了 `herd7 7.58, Rev: exported` 并跑通——它大概率自带一份环境设定（如工具链路径），不依赖调用者裸 shell 的 PATH。这不影响本次判定，只作为环境旁证记录。

原样输出：

```
══ LKMM（herd7） ══
  ✓ 内核树 /home/fy5090/linux-bug-fix/linux
  ✓ herd7  7.58, Rev: exported

  ✓ litmus/commit-publish-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/commit-publish.litmus  Never（符合声明）
  ✓ litmus/first-txn-journal-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-journal-implies-units.litmus  Never（符合声明）
  ✓ litmus/first-txn-root-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-root-implies-units.litmus  Never（符合声明）

  ✓ LKMM 通过（3 条 Never：每条都有内容对得上的对照组，2 条绑到代码、1 条声明不对应代码；共 3 条 Sometimes）
  ✓ 内存序：litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符
```

