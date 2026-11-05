#!/usr/bin/env bash
# 被设备探针 source；S 由调用方提供。只移除本进程成功创建的映射。
declare -A OWNED_DM=()

create_dm() {
  S dmsetup create "$@" || return 1
  OWNED_DM["$1"]=1
}

remove_dm() {
  [[ "${OWNED_DM[$1]:-0}" == 1 ]] || return 0
  S dmsetup remove "$1" || return 1
  unset 'OWNED_DM[$1]'
}
