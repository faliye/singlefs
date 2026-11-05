# 判别力样本：纯 Python 装置，复跑行起 scripts/experiment-apparatus/ 下的脚本

## E94 纯 Python 装置的样本 —— 样本

原始输出 `research/results/e94-sample-2026-11-03.out`，复跑（仓根）：

```bash
# 单测与装置（--cwd 加 -m；--cwd 加脚本名）
task-server/task submit --cwd scripts/experiment-apparatus/e94_sample -- python3 -m unittest -v test_driver
task-server/task submit --cwd scripts/experiment-apparatus/e94_sample -- python3 driver.py --product research/results/<新文件>
# 编译（带占位的全路径 bash 脚本）
task-server/task submit --cwd <新目录>/tree -- bash <仓根>/scripts/experiment-apparatus/e94_sample/compile.sh <仓根>/research/results/<新文件>
# 全路径 python3
task-server/task submit -- python3 scripts/experiment-apparatus/e94_sample/driver.py --repository . --product research/results/<新文件>
```

## 历史版本
