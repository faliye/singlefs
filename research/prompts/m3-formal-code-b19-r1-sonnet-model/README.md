正推腿（Sonnet）第 19 批代码轮第一轮的模型与脚本。全部在副本 /tmp/claude-1000/m3-formal-code-b19-r1-sonnet/ws 里跑（冻结树 crates 拷出 + 仓根 Cargo.toml / Cargo.lock / .cargo），不动仓里的 crates。
- b19_sonnet_probe.rs：放进副本 crates/singlefs-harness/tests/ 跑。scan_k_and_z 扫（k 次只暖机挂载 × z 次零单元发布）之后写第一个文件，打印落点相对 U 的偏移与 I-5.5 判定、判红清单；scan_record_only 是清零第一个文件根槽的变体（没验到，见报告）。
- run_mutations.py：把 mutations.tsv 的行逐条打到副本 walk.rs 上跑点名的测试；kinds.tsv 是正推腿自加的「某一种类的头 ID 一律不判」变异（种类 3、4、6、7、8、11、12、1、9、10）。
- rename_equivalence.py：19-2 改名等价性检查（记号流等长 + 每个使用解析到同一绑定）。
- kbi_print.patch：副本里 checker_known_bad_images.rs 改成 kbi_print.rs 的差异（把断言改成逐镜像打印）。
- out/：各次运行的原始日志。
