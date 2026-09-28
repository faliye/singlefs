#!/usr/bin/env bash
# gate-stage: 样本：不带编号的阶段，头部声称有判别力样本
# 判别力：fixtures/tooling-claims.sh/red 必须判红（这份样本里故意不建那个目录）
ROOT="${1:-.}"
cd "$ROOT" || exit 2
exit 0
