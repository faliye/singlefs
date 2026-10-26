#!/usr/bin/env bash
run_there() { # <目录> <命令文本>：照 peer_host_run 两支的样子
  if [[ "$here_is_peer" == 1 ]]; then
    (cd "$1" && bash -c "$2")
    return
  fi
  ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && bash -c $(printf '%q' "$2")"
}
