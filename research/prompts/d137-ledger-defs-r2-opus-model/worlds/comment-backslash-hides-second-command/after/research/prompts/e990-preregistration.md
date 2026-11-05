# E990 样本登记（comment-backslash-hides-second-command）

## 五、起法

```bash
task-server/task submit -- cargo run --release -p singlefs-checker-tier --features gpu --bin e990_judge_sample -- feasibility
# 对照趟，下一行起，没带 gpu 特性 \
task-server/task submit -- cargo run --release -p singlefs-checker-tier --bin e990_judge_sample -- control
```
