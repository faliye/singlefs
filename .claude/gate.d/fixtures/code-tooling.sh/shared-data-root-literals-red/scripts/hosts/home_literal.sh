#!/usr/bin/env bash
# 样本：经 ~、$HOME、${HOME}、${HOME:?} 取数据根下共享的那几项（四处都该点名）
LOG="${SAMPLE_LOG:-$HOME/.local/share/singlefs/work/sample-run}"
cache=~/.local/share/singlefs/cross-run-cache/gate-run-times
records="${HOME:?}/.local/share/singlefs/mutation-run-records"
logs="${HOME}/.local/share/singlefs/gate-staged-logs"
ls "$LOG" "$cache" "$records" "$logs"
