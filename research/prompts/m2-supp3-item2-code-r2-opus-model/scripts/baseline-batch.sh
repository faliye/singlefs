#!/usr/bin/env bash
R=/tmp/claude-1000/m2-supp3-item2-code-r2-opus/repo
P=/tmp/claude-1000/m2-supp3-item2-code-r2-opus/probe/run-probe.sh
date -u
$P $R base-wall-observe-0-32 wall 0 32 150 observe
$P $R base-wall-observe-32-512 wall 32 480 150 observe
$P $R base-wall-skip-0-2000 wall 0 2000 150 skip
$P $R base-wallrb-skip-0-2000 wall_rb 0 2000 150 skip
$P $R base-wallrb-skip-300-0-1000 wall_rb 0 1000 300 skip
$P $R base-wallplain-skip-0-1000 wall_plain 0 1000 150 skip
$P $R base-broad-skip-150-0-1000 broad 0 1000 150 skip
$P $R base-rollback-skip-150-0-1000 rollback 0 1000 150 skip
$P $R base-reuse-skip-150-0-1000 reuse 0 1000 150 skip
date -u
