#!/usr/bin/env bash
# 长跑脚本共用的一段：装置二进制在一趟里不许换，每一片记下打它的是哪一版装置，续跑只沿用同一版装置打的片（欠账 C862）。
# 被长跑脚本 source，不单独起（库，没有准入与运行条件）；里面只有函数定义与两个全局变量，source 的时候什么都不做。
#   binary_identity_pin <装置二进制> <私有目录>   开跑时拷一份进私有目录（按 sha256 分子目录、拷完只读），设 PINNED_BINARY（拷贝的路径）与 PINNED_BINARY_SHA256（拷贝的 sha256）；
#                                                  之后每片每段只按 PINNED_BINARY 起，不再 exec 仓根 target/release 下那一份（别的编译随时会把它换掉）
#   binary_identity_assert_unchanged [<路径> <sha>]  起每一片之前核私有副本的 sha256 还是开跑时那个（不给参数核最近拷的那份；一趟里拷了几份装置的，逐份带参数核）
#   binary_identity_stamp_write <片文件> [sha]   片旁写 <片文件>.binary-sha256（先写临时名再改名），默认写 PINNED_BINARY_SHA256
#   binary_identity_stamp_read <片文件>          打片旁记的 sha256，没有记录打空
#   binary_identity_is_current <片文件>          片旁记的 sha256 与这一趟的相同
#   binary_identity_void_line <片名> <片里记的 sha256> <移到哪> [<这一趟的 sha256>]   逐片报「作废：<片名>（装置 sha256 不同）…」
#   binary_identity_void_shard <片名> <片文件> <作废目录>        片与它的记录一起移进作废目录（留着不删），并报上面那一行
#   binary_identity_check_same <期望 sha256 或空串> <片名> <片里记的 sha256> [<片名> <片里记的 sha256> …]
#                                                合并产物之前再核一遍：期望值空串时要求彼此相同；任何一片没有记录或不同就拒、返回 1
#   binary_identity_install_python_shim <装置源文件相对仓根的路径> <放垫片的目录>   别台编好送来的装置带着编译机的准入脚本路径，这台上没有：PATH 最前面放 python3 垫片把 preflight.py 与装置源文件两个路径换成这棵树里的同名文件
# gate-similar: scripts/hosts/compile-then-swap.py 它管「编译放在另一个目录、编完整份换上」，不管长跑脚本开跑后装置在不在变；scripts/hosts/source-stamp.py 它给源码树记指纹，不给二进制记；没有已有的库管长跑里装置是哪一版

PINNED_BINARY=""
PINNED_BINARY_SHA256=""

binary_identity_sha256_of() { # <文件>
  local digest
  digest=$(sha256sum -- "$1" 2> /dev/null | cut -d' ' -f1)
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || return 1
  echo "$digest"
}

binary_identity_pin() { # <装置二进制> <私有目录>
  local original="$1" root="$2" name before="" after copy attempt destination
  PINNED_BINARY=""
  PINNED_BINARY_SHA256=""
  if [[ ! -f "$original" || ! -x "$original" ]]; then
    echo "  ✗ 装置二进制不在或不可执行，拷不了私有副本：$original" >&2
    echo "     → 怎么办：先编好装置、核对路径（环境变量或 --binary 指到的文件要存在且可执行），再起本脚本" >&2
    return 1
  fi
  name=$(basename -- "$original")
  mkdir -p -- "$root" || { echo "  ✗ 建不了装置私有副本的目录：$root" >&2; echo "     → 怎么办：核对这个目录的上级可写、盘没满" >&2; return 1; }
  copy="$root/.pinning-$$-$name"
  for attempt in 1 2 3; do
    before=$(binary_identity_sha256_of "$original") || { echo "  ✗ 算不出装置二进制的 sha256：$original" >&2; echo "     → 怎么办：核对文件可读，再起本脚本" >&2; return 1; }
    rm -f -- "$copy"
    cp -- "$original" "$copy" || { echo "  ✗ 拷不了装置二进制：$original → $copy" >&2; echo "     → 怎么办：核对盘没满、目录可写" >&2; return 1; }
    chmod u+x -- "$copy"
    after=$(binary_identity_sha256_of "$copy") || { echo "  ✗ 算不出私有副本的 sha256：$copy" >&2; echo "     → 怎么办：核对盘没满，再起本脚本" >&2; return 1; }
    [[ "$before" == "$after" ]] && break
    before=""
  done
  if [[ -z "$before" ]]; then
    rm -f -- "$copy"
    echo "  ✗ 拷装置二进制的时候它一直在变（三次拷出来的 sha256 都与拷前不同）：$original" >&2
    echo "     → 怎么办：等别的会话的编译（cargo build、compile-then-swap）收尾，再起本脚本" >&2
    return 1
  fi
  destination="$root/$after/$name"
  mkdir -p -- "$root/$after" && mv -f -- "$copy" "$destination" || { rm -f -- "$copy"; echo "  ✗ 私有副本没放进 $destination" >&2; echo "     → 怎么办：核对盘没满、目录可写" >&2; return 1; }
  chmod 0555 -- "$destination"
  PINNED_BINARY="$destination"
  PINNED_BINARY_SHA256="$after"
}

binary_identity_assert_unchanged() { # [<私有副本路径> <开跑时的 sha256>]，不给就核最近一次 binary_identity_pin 拷的那份
  local path="${1:-$PINNED_BINARY}" expected="${2:-$PINNED_BINARY_SHA256}" now
  [[ -n "$expected" ]] || { echo "  ✗ 装置二进制还没拷私有副本就要起片（脚本内部次序错）" >&2; echo "     → 怎么办：交主 agent 看脚本" >&2; return 1; }
  now=$(binary_identity_sha256_of "$path") || now=""
  if [[ "$now" != "$expected" ]]; then
    echo "  ✗ 私有副本 $path 的 sha256 变了（开跑时 $expected，现在 ${now:-读不到}）：这一趟的片不能再按它起" >&2
    echo "     → 怎么办：有人改了私有副本；整趟作废，换个干净的环境重起本脚本（已做完、记着同一 sha256 的片仍沿用）" >&2
    return 1
  fi
}

binary_identity_stamp_write() { # <片文件> [<sha256>]
  local file="$1" digest="${2:-$PINNED_BINARY_SHA256}"
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || { echo "  ✗ 写不了片的装置记录：sha256 不是 64 位十六进制（「$digest」）" >&2; echo "     → 怎么办：交主 agent 看脚本" >&2; return 1; }
  printf '%s\n' "$digest" > "$file.binary-sha256.tmp-$$" && mv -f -- "$file.binary-sha256.tmp-$$" "$file.binary-sha256"
}

binary_identity_stamp_read() { # <片文件>
  [[ -f "$1.binary-sha256" ]] || return 0
  head -n 1 -- "$1.binary-sha256" | tr -d '[:space:]'
}

binary_identity_is_current() { # <片文件>
  [[ -n "$PINNED_BINARY_SHA256" && "$(binary_identity_stamp_read "$1")" == "$PINNED_BINARY_SHA256" ]]
}

binary_identity_void_line() { # <片名> <片里记的 sha256，没有记录写空串> <移到哪> [<这一趟的 sha256，不给取 PINNED_BINARY_SHA256>]
  echo "作废：$1（装置 sha256 不同）：记的是 ${2:-没有记录}，这一趟的是 ${4:-$PINNED_BINARY_SHA256}；整份移到 $3，不删，现跑"
}

binary_identity_void_shard() { # <片名> <片文件> <作废目录>
  local name="$1" file="$2" void="$3" recorded destination
  recorded=$(binary_identity_stamp_read "$file")
  mkdir -p -- "$void" || { echo "  ✗ 建不了作废目录：$void" >&2; echo "     → 怎么办：核对这个目录的上级可写" >&2; return 1; }
  destination="$void/$(basename -- "$file")"
  [[ -e "$destination" ]] && destination="$destination.$$"
  mv -- "$file" "$destination" || return 1
  if [[ -f "$file.binary-sha256" ]]; then mv -- "$file.binary-sha256" "$destination.binary-sha256" || return 1; fi
  binary_identity_void_line "$name" "$recorded" "$destination"
}

binary_identity_check_same() { # <期望 sha256 或空串> <片名> <记的 sha256> …
  local expected="$1" name recorded mismatched=0 total=0
  shift
  (($# > 0 && $# % 2 == 0)) || { echo "  ✗ 脚本内部：核装置 sha256 时片名与记录没成对" >&2; echo "     → 怎么办：交主 agent 看脚本" >&2; return 1; }
  [[ -n "$expected" ]] || expected="$2"
  while (($# > 0)); do
    name="$1"; recorded="$2"; shift 2
    total=$((total + 1))
    if [[ -z "$recorded" || "$recorded" != "$expected" ]]; then
      echo "  ✗ 片 $name 的装置 sha256 对不上：记的是 ${recorded:-没有记录}，要的是 ${expected:-（空）}" >&2 # gate-lint:detail
      mismatched=$((mismatched + 1))
    fi
  done
  if ((mismatched > 0)); then
    echo "  ✗ 合并产物之前核全部片的装置 sha256：$total 片里 $mismatched 片对不上，不出产物" >&2
    echo "     → 怎么办：不带只合并的参数重起本脚本，对不上的片会作废、用这一趟的装置重做；别台做的片要让别台用同一份装置（把本机这一份送过去）" >&2
    return 1
  fi
  echo "核过：$total 片的装置 sha256 相同（$expected）"
}

binary_identity_install_python_shim() { # <装置源文件相对仓根的路径> <放垫片的目录>
  local device_source_path="$1" shim_directory="$2" device_source_name real_python tree
  device_source_name="${device_source_path##*/}"
  real_python=$(command -v python3) || { echo "  ✗ 这台上没有 python3，装置判不了准入" >&2; echo "     → 怎么办：装上 python3 再起本脚本" >&2; return 1; }
  tree=$(pwd -P)
  mkdir -p -- "$shim_directory" || return 1
  cat > "$shim_directory/python3" <<SHIM
#!/usr/bin/env bash
if [[ "\${2:-}" == check ]]; then
  case "\$1" in
    */.claude/singlefs-ai-sop/scripts/preflight.py) set -- "$tree/.claude/singlefs-ai-sop/scripts/preflight.py" "\${@:2}" ;;
  esac
  case "\${3:-}" in
    */$device_source_name) set -- "\$1" "\$2" "$tree/$device_source_path" "\${@:4}" ;;
  esac
fi
exec "$real_python" "\$@"
SHIM
  chmod +x "$shim_directory/python3"
  export PATH="$shim_directory:$PATH"
}
