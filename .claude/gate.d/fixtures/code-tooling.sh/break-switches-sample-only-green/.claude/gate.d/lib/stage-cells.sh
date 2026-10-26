#!/usr/bin/env bash
# 样本共用库：定义了那一判，stage_cells_run 开头调它
stage_cells_refuse_unsampled_break_switches() { :; }
stage_cells_run() {
  stage_cells_refuse_unsampled_break_switches "$1"
  echo "跑格"
}
