#!/bin/bash
# A8：同一份入库内容，换机器、换人时崩溃枚举用例的输入指纹（research/scripts/admission.py crash-case-manifest，
# 带 --judging-digest --toolchain --build-environment，与门禁 54 号同一种算法）是不是逐字相同。
# 用法：bash a8_fingerprint.sh <主仓根> <草稿目录>   （草稿目录下建一份带 .git 的仓副本，放在另一层路径下；不改主仓）
set -euo pipefail
main="${1:?主仓根}"; scratch="${2:?草稿目录}"
key=crash-case:layer0-first-stream
copy="$scratch/another/place/singlefs"
mkdir -p "$copy" "$scratch/manifests"
rsync -a --delete --exclude target "$main"/ "$copy"/
fingerprint() {
  local label="$1" root="$2"; shift 2
  local line
  line="$(env "$@" python3 "$main/research/scripts/admission.py" crash-case-manifest "$root" "$key" "$scratch/manifests/$label.txt" --judging-digest --toolchain --build-environment)"
  echo "$label ${line%% *}"
}
: > "$scratch/fingerprints.txt"
fingerprint main_default "$main" PATH="$PATH" >> "$scratch/fingerprints.txt"
fingerprint copy_at_another_path "$copy" PATH="$PATH" >> "$scratch/fingerprints.txt"
fingerprint main_other_target_dir_and_threads "$main" CARGO_TARGET_DIR=/tmp/elsewhere SINGLEFS_LAYER0_THREADS=32 CARGO_BUILD_JOBS=2 >> "$scratch/fingerprints.txt"
mkdir -p "$scratch/cargo-home"
printf '[term]\ncolor = "always"\n' > "$scratch/cargo-home/config.toml"
fingerprint main_personal_cargo_home_config "$main" CARGO_HOME="$scratch/cargo-home" >> "$scratch/fingerprints.txt"
mkdir -p "$scratch/another/.cargo"
printf '[net]\noffline = true\n' > "$scratch/another/.cargo/config.toml"
fingerprint copy_under_a_directory_with_its_own_cargo_config "$copy" PATH="$PATH" >> "$scratch/fingerprints.txt"
rm "$scratch/another/.cargo/config.toml"
sed -i 's#/usr/lib/gcc/x86_64-linux-gnu/13/include#/usr/lib/gcc/x86_64-linux-gnu/14/include#' "$copy/.cargo/config.toml"
fingerprint copy_whose_machine_has_gcc_14 "$copy" PATH="$PATH" >> "$scratch/fingerprints.txt"
fingerprint main_with_rustflags_target_cpu_native "$main" RUSTFLAGS="-C target-cpu=native" >> "$scratch/fingerprints.txt"
reference="$(awk '$1 == "main_default" {print $2}' "$scratch/fingerprints.txt")"
while read -r label value; do
  echo "E7RESULT name=r3_a8_fingerprint variant=$label fingerprint=${value:0:16} same_as_main_default=$([ "$value" = "$reference" ] && echo true || echo false)"
done < "$scratch/fingerprints.txt"
local_only_but_differs=$(awk -v reference="$reference" '($1 == "main_personal_cargo_home_config" || $1 == "copy_under_a_directory_with_its_own_cargo_config" || $1 == "copy_whose_machine_has_gcc_14") && $2 != reference' "$scratch/fingerprints.txt" | wc -l)
echo "E7RESULT name=r3_a8_summary settings_that_change_no_judgment_but_change_the_fingerprint=$local_only_but_differs must_be_nonzero=$local_only_but_differs"
