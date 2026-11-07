#!/usr/bin/env bash
# 长跑脚本共用的一段：装置二进制在一趟里不许换，每一片记下打它的是哪一版装置，续跑只沿用同一版装置打的片（欠账 C862）。
# 被长跑脚本 source（binary_identity_build 另给 e6-units.sh 与 scripts/devices/e152-run.sh 取编出来的装置），不单独起（库，没有准入与运行条件）；里面只有函数定义与四个全局变量（PINNED_BINARY、PINNED_BINARY_SHA256、BUILT_BINARY、BUILT_BINARIES），source 的时候什么都不做。
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
#   binary_identity_build <bin 名>[,<bin 名>…] <cargo build 的参数…>   起 cargo build <参数…> --message-format=json-render-diagnostics（编译的诊断照旧打到标准错误），
#                                                从 cargo 报的 compiler-artifact 行里逐个取这几个 bin 的 executable，照给的次序放进 BUILT_BINARIES、头一个另放进 BUILT_BINARY：
#                                                这一趟实际编出来的那几份（一条 cargo 编几个 bin 的，名字逗号隔开一起给）。
#                                                编译目录由 task-server 给、经 CARGO_TARGET_DIR 交下来，不往仓根 target/ 编（.claude/main-agent.md「一轮怎么开、怎么收」）；
#                                                仓根 target/ 下那一份可能是早先编的旧二进制，不拼那条路径。cargo 退非 0 返回 1；编过了却有一个 bin 报不出 executable
#                                                返回 2、打原因与出路（两种 BUILT_BINARY 都是空串、BUILT_BINARIES 是空数组）
# gate-similar: scripts/hosts/compile-then-swap.py 它管「编译放在另一个目录、编完整份换上」，不管长跑脚本开跑后装置在不在变；scripts/hosts/source-stamp.py 它给源码树记指纹，不给二进制记；没有已有的库管长跑里装置是哪一版

PINNED_BINARY=""
PINNED_BINARY_SHA256=""
BUILT_BINARY=""
BUILT_BINARIES=()

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

binary_identity_build() { # <bin 名>[,<bin 名>…] <cargo build 的参数…>
  local names="$1" messages located index
  local -a wanted=()
  shift
  BUILT_BINARY=""
  BUILT_BINARIES=()
  IFS=',' read -r -a wanted <<<"$names"
  messages=$(mktemp "${TMPDIR:-/tmp}/binary-identity-build.XXXXXX") || {
    echo "  ✗ 建不了收 cargo 机读输出的临时文件（${TMPDIR:-/tmp} 下）" >&2
    echo "     → 怎么办：看 ${TMPDIR:-/tmp} 满没满、可不可写，腾出地方再起本脚本" >&2
    return 1
  }
  if ! cargo build "$@" --message-format=json-render-diagnostics > "$messages"; then
    rm -f -- "$messages"
    return 1
  fi
  located=$(python3 - "$messages" "$names" <<'PYTHON'
import json, sys
path, names = sys.argv[1], sys.argv[2].split(",")
found = {}
with open(path, encoding="utf-8", errors="replace") as handle:
    for line in handle:
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line)
        except ValueError:
            continue
        target_name = (message.get("target") or {}).get("name")
        if message.get("reason") == "compiler-artifact" and target_name in names and message.get("executable"):
            found[target_name] = message["executable"]
for name in names:
    print(found.get(name, ""))
PYTHON
  )
  rm -f -- "$messages"
  mapfile -t BUILT_BINARIES <<<"$located"
  for index in "${!wanted[@]}"; do
    if [[ -z "${BUILT_BINARIES[index]:-}" ]]; then
      echo "  ✗ cargo build 退 0，机读输出里却没有 bin ${wanted[index]} 的 compiler-artifact 行（executable），不知道这一趟编出来的在哪" >&2
      echo "     → 怎么办：单跑同一条 cargo build … --message-format=json-render-diagnostics 看它报的 executable；不拼仓根 target/ 下的路径（那一份可能是早先编的旧二进制）" >&2
      BUILT_BINARIES=()
      return 2
    fi
  done
  BUILT_BINARY="${BUILT_BINARIES[0]}"
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
    echo "     → 怎么办：不带只合并的参数重起本脚本，对不上的片会作废、用这一趟的装置重做；别台做的片要用同一份装置（放在共享根下，两台照同一个绝对路径起）" >&2
    return 1
  fi
  echo "核过：$total 片的装置 sha256 相同（$expected）"
}
