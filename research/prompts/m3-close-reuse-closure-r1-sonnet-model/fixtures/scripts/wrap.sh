#!/usr/bin/env bash
# 假的包装：--peer 与 -- 之后是命令
shift_until=0
while [[ "${1:-}" != "--" ]]; do shift; done
shift
exec "$@"
