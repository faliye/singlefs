#!/usr/bin/env bash
# 样本：引号里整句拿来当样本、注释与出路文字里写的常量都不点名：run-with-memory-cap.sh 16G
feed "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness" 0
echo "照抄：bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/crash/crash-point-replay.sh --quick"
