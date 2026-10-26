#!/usr/bin/env bash
# 样本：经内存包装起装置，上限写死
bash scripts/hosts/run-with-memory-cap.sh 16G "$BINARY" run
bash "$repository/scripts/hosts/run-with-memory-cap.sh" 4G "$BINARY" list-items
