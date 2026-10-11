m3-batch10z-b-code-r1 正推腿（Sonnet）的模型目录。报告：research/prompts/m3-batch10z-b-code-r1-sonnet-output.md

k2/  K2-2 两臂量法（副本里的探测，不入库装置）
  k2_arms_probe.rs        探测测试文件。放进副本的 crates/singlefs-harness/tests/ 下，与 common_unwitnessed_publish/、common_admission_pool/ 同级。
                          用入库装置 common_unwitnessed_publish 造 R（K4-1 的形 1），M1 里 R 的某一个单元读回全零、R 的账三档读错
                          （账无故障 / 仅 M1 时读错 / M1 之后读错），会话覆盖写 60 次、每三步跑一次池级 checker，最后卸载。
  c-arm-mount.rs.diff / c-arm-allocator.rs.diff
                          C 臂开关（环境变量 SINGLEFS_ARM_C）相对冻结副本的 diff；加在副本里，不是实现。
  an.py / an-output.txt   统计脚本与原样输出（读 sweep-*.txt）。summ.py 是早一版的统计脚本。
  sweep-today-arm.txt / sweep-c-arm.txt   两臂各 50 例的原始行（RESULT / ISOSET）。
  复跑：SWEEP_MOD=3 SWEEP_REM=<0|1|2> [SINGLEFS_ARM_C=1] CARGO_TARGET_DIR=<编译目录> bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 8 cargo test --offline -p singlefs-harness --test k2_arms_probe -- --nocapture --test-threads 8 s_ [m_]
k3/  K3 变异
  mut.py                  变异表（名 -> 文件、原文、替换文；原文须恰好命中一次）与施加/还原。用法 python3 mut.py <名> [restore]；路径写死在副本 repo-k3 下，用时改 R。
  run.sh                  施加、按 STAGES 跑 harness 档目标、还原。all.sh / all2.sh / all3.sh 是我用过的驱动。
  out/*.txt               每条变异的结果摘要（## 行 = 目标、rc、test result；FAILED 行 = 红的用例）。
所有测试与编译都在草稿目录的副本里做，仓内文件没改。
