#!/usr/bin/env bash
# 没设就给默认值的常见写法：|| 之后同一行赋值
[[ -n "${SAMPLE_OR_FALLBACK_JOBS:-}" ]] || SAMPLE_OR_FALLBACK_JOBS=8
exec make -j "$SAMPLE_OR_FALLBACK_JOBS"
