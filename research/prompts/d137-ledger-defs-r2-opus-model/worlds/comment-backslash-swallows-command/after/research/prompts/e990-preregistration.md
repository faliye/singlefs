# E990 样本登记（comment-backslash-swallows-command）

## 五、起法

```bash
# 可行性档，下一行起，没带 gpu 特性 \
task-server/task submit -- cargo run --release -p singlefs-checker-tier --bin e990_judge_sample -- feasibility
```
