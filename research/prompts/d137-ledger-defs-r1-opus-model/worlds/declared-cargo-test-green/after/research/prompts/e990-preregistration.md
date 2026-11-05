# E990 样本登记（declared-cargo-test-green）

## 五、起法

```bash
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e990_judge_sample 的单测; 范围=1 个测试目标; 为什么=样本：带范围声明的整条照样认得出特性' task-server/task submit -- cargo test --release -p singlefs-checker-tier --features gpu --bin e990_judge_sample
```
