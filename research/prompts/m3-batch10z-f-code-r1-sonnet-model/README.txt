正推腿（Sonnet）自己写的模型，不是入库装置。2026-10-08，轮 m3-batch10z-f-code-r1。
都在冻结副本的拷贝上跑（草稿目录 ~/.local/share/singlefs/work/m3-batch10z-f-code-r1-sonnet/ 下 copy-a、copy-w0..3，各自一个 CARGO_TARGET_DIR）。

sonnet_g2_model.rs    放进拷贝的 crates/singlefs-harness/tests/，只用公开接口：真 GpuMemoryLedger + 一张假的物理卡。
                      4 条：k1_3（等失败的借据卡在设备丢了那一刻活着的集合里）、k1_2（等失败的借据一笔笔累加）、
                      k3_1（随机操作，账不许给多于物理上真能再借的；SEEDS=<种子数>）、k3_2（Remain 状态少给多少、什么时候回来）。
                      跑法（拷贝根目录）：SEEDS=200000 CARGO_TARGET_DIR=<目录> bash scripts/hosts/run-with-memory-cap.sh 1G bash scripts/hosts/capped.sh 4 \
                        cargo test --offline -p singlefs-harness --test sonnet_g2_model -- --nocapture
parse_rows.py         把 _m3-batch10z-f-code-r1-diff.md 里追加 27 行与整行替换 9 行的变异表抽成 rows.json
run_mut.py            对拷贝逐行注入变异、跑那一行点名的测试、还原（环境变量 ROWS=行文件，LOGP=日志前缀，SEEDS 透传）；参数：工作号 工作数 起 止
probes.json           正推腿补的 9 条探针变异（P1、P2、P3、P5、P8、P9、P10、P11、P12），每条跑整个 gpu_memory_ledger 模块
controls.json         拿 sonnet_g2_model.rs 当判官的 6 条（4 条是这一批的变异行，2 条是探针）
emulate_scans.py      checker 档两条「扫源码」用例（the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers、
                      the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line）的 Python 复刻，
                      在冻结副本的 crash_amplification.rs 上判基线、两行变异、五个「改回去」；复刻不是原用例，只能说明扫的逻辑
resume_in_drop.rs     rustc 直接编：Drop 里 catch_unwind 之后 resume_unwind，在展开中 abort（退出码 134）
sonnet_stop_wide.rs   实现员 gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs 的拷贝，SCRIPTS 改 6000、FIRST_SEED 改 0x5EED_F00D（24000 段）

logs/                 上面各条的原始输出。model-run5.log 里的 k3_2 一条是旧写法（以为 A 还了就回到正常）红的，之后改对了（B 触发重起、也算设备丢了那一刻活着的），
                      改后的输出在 model-run6.log；K3-1 的 200000 种子、K1-2、K1-3 的数出自 model-run5.log。
                      mut-w*.log：31 行变异的判定；probe-w*.log：9 条探针；control-w*.log：6 条拿模型当判官；p5-trace.log：探针 P5 上模型的红（144/20000）。
