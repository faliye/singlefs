//! 项名、录入键与一项的清单，与录入段、判段两段怎么接（命中就不再录、判段按清单读回）。

use super::*;

impl ItemName {
    /// # Errors
    /// 空、超过 200 字节、带别的字符。
    pub fn new(text: &str) -> Result<Self, String> {
        if text.is_empty() || text.len() > 200 {
            return Err(format!("项名要 1 到 200 字节：{text:?}"));
        }
        if let Some(bad) = text
            .chars()
            .find(|character| !(character.is_ascii_alphanumeric() || "_.:/-".contains(*character)))
        {
            return Err(format!(
                "项名只许字母、数字、_ . : / -，{text:?} 里有 {bad:?}"
            ));
        }
        Ok(Self(text.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 一趟录入的输入摘要：录入段要写进库的那几样（流、展开、判表的开关这类）按内容取的 SHA-256，由调用方算。同一份输入同一个摘要。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecordingInputsDigest(pub [u8; DIGEST_BYTES]);

/// 这一趟的录入键与凭它跳不跳过录入（[`recording_key_of_the_scenario_and_the_inputs`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingKeyOfThisRun {
    pub key: RecordingKey,
    /// 录入键下已有清单就跳过录入。只有对拍参照一侧的判法单元拒算时是假（照录、不复用，`recording_key_of_the_pairing_input_and_the_reference_units`）。
    pub skips_the_recording_on_a_hit: bool,
    /// 参照一侧拒算时那一句。
    pub refusal: Option<String>,
}

/// 录入键的组成只写在这一处：录下的写流摘要（`inputs`：录下的流程的字节连同屏障切出的段、按段登记的段这类，由调用方按内容算）与场景标识
/// （`scenario`：哪条崩溃枚举用例或放量分项，项名或流名）拼起来取 SHA-256。代码一个单元都不进（产品代码、录制管线、判器、观察者、测试服务都不进，
/// 也不经测试线程的名字认是哪条测试）：产品代码改了而写流没变，录入键不变、命中就不再录；写流变了（写的字节、屏障切段）键就变、照录。
/// 交回的 `skips_the_recording_on_a_hit` 恒是真。
#[must_use]
pub fn recording_key_of_the_scenario_and_the_inputs(
    scenario: &str,
    inputs: &RecordingInputsDigest,
) -> RecordingKeyOfThisRun {
    RecordingKeyOfThisRun {
        key: RecordingKey(sha256_digest(
            format!(
                "singlefs recording key\nscenario={scenario}\ninputs={}\n",
                hexadecimal_text(&inputs.0)
            )
            .as_bytes(),
        )),
        skips_the_recording_on_a_hit: true,
        refusal: None,
    }
}

/// 录入段登在库里的清单：这一项是哪一类（`kind`，一个写死的串），录下的几样各叫什么、摘要是什么。判段只认清单指到的那几样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemManifest {
    pub kind: String,
    pub digests: Vec<(String, [u8; DIGEST_BYTES])>,
}

const ITEM_MANIFEST_MAGIC: &[u8; 8] = b"SFITEM01";

impl ItemManifest {
    /// 清单里叫 `name` 的那一样的摘要。
    ///
    /// # Errors
    /// 清单里没有这一样。
    pub fn digest_named(&self, name: &str) -> Result<[u8; DIGEST_BYTES], String> {
        self.digests
            .iter()
            .find(|(digest_name, _)| digest_name == name)
            .map(|(_, digest)| *digest)
            .ok_or_else(|| {
                format!(
                    "清单（{}）里没有「{name}」这一样：录入段那一趟没录它",
                    self.kind
                )
            })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = ITEM_MANIFEST_MAGIC.to_vec();
        let put_text = |destination_bytes: &mut Vec<u8>, text: &str| {
            destination_bytes
                .extend_from_slice(&u64::try_from(text.len()).expect("装得进 u64").to_le_bytes());
            destination_bytes.extend_from_slice(text.as_bytes());
        };
        put_text(&mut bytes, &self.kind);
        bytes.extend_from_slice(
            &u64::try_from(self.digests.len())
                .expect("装得进 u64")
                .to_le_bytes(),
        );
        for (name, digest) in &self.digests {
            put_text(&mut bytes, name);
            bytes.extend_from_slice(digest);
        }
        bytes
    }

    /// 头不对、字节不够或多出交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut at = 0usize;
        let take = |read_position: &mut usize, count: usize| -> Option<&[u8]> {
            let slice = bytes.get(*read_position..*read_position + count)?;
            *read_position += count;
            Some(slice)
        };
        if take(&mut at, ITEM_MANIFEST_MAGIC.len())? != ITEM_MANIFEST_MAGIC {
            return None;
        }
        let read_count = |read_position: &mut usize| -> Option<usize> {
            let raw: [u8; 8] = take(read_position, 8)?.try_into().ok()?;
            usize::try_from(u64::from_le_bytes(raw)).ok()
        };
        let read_text = |read_position: &mut usize| -> Option<String> {
            let length = read_count(read_position)?;
            String::from_utf8(take(read_position, length)?.to_vec()).ok()
        };
        let kind = read_text(&mut at)?;
        let count = read_count(&mut at)?;
        let mut digests = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            let name = read_text(&mut at)?;
            let digest: [u8; DIGEST_BYTES] = take(&mut at, DIGEST_BYTES)?.try_into().ok()?;
            digests.push((name, digest));
        }
        if at != bytes.len() {
            return None;
        }
        Some(Self { kind, digests })
    }

    /// 清单自己的摘要（行里报它的头 8 字节）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(&self.to_bytes())
    }
}

impl CrashLibrary {
    /// 录入段按录入键登这一项的清单，打 `CRASH_LIBRARY_RECORDED` 那一行（`reused` 是这一趟录入键命中、没再录）。
    ///
    /// # Errors
    /// 库写失败。
    pub fn record_the_item(
        &self,
        item: &ItemName,
        recording_key: &RecordingKey,
        manifest: &ItemManifest,
    ) -> Result<(), String> {
        self.lock()
            .store_recorded_run_manifest(recording_key, &manifest.to_bytes())
            .map_err(|error| format!("项「{}」的清单写不进库：{error:?}", item.as_str()))?;
        self.print_the_recorded_line(item, manifest);
        Ok(())
    }

    fn print_the_recorded_line(&self, item: &ItemName, manifest: &ItemManifest) {
        println!(
            "CRASH_LIBRARY_RECORDED class={} item={} manifest={} library={}",
            self.class.name(),
            item.as_str(),
            hexadecimal_text(&manifest.digest()[..8]),
            self.library_name
        );
    }

    /// 录入键下登的清单；没登过交回 `None`。
    ///
    /// # Errors
    /// 库读失败；清单解不出。
    pub fn manifest_under(
        &self,
        recording_key: &RecordingKey,
    ) -> Result<Option<ItemManifest>, String> {
        let Some(bytes) = self
            .lock()
            .read_recorded_run_manifest(recording_key)
            .map_err(|error| format!("读录入键下的清单失败：{error:?}"))?
        else {
            return Ok(None);
        };
        ItemManifest::from_bytes(&bytes)
            .map(Some)
            .ok_or_else(|| "录入键下的清单解不出（字节坏了或编码换了）：重跑录入段".to_string())
    }

    /// 判段照这一趟的场景与输入现算录入键、读清单。
    ///
    /// # Errors
    /// 库读失败；库里没有这一份输入的清单（录入段没跑，或输入变了）；清单解不出。
    pub fn manifest_of_the_inputs(
        &self,
        scenario: &str,
        inputs: &RecordingInputsDigest,
    ) -> Result<ItemManifest, String> {
        let recording = recording_key_of_the_scenario_and_the_inputs(scenario, inputs);
        self.manifest_under(&recording.key)?.ok_or_else(|| {
            format!(
                "{} 类开的库 {} 里没有这一份输入（{}）的清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                self.class.name(),
                self.library_name,
                hexadecimal_text(&inputs.0[..8])
            )
        })
    }
}

/// 两段过一次库：录入段（`record`，CPU 跑数据）把这一项的几样写进库、交回清单，这里按录入键登清单；录入键下已有清单就不再录
/// （命中就跳过，打 `CRASH_LIBRARY_RECORDING_REUSED` 与 `CRASH_LIBRARY_RECORDED` 两行）。判段（`judge`）照同一份输入
/// 现算录入键、读回清单，只拿读回的那一份往下判。选段照 [`RunStage::of_this_process`]。`item` 是场景标识（进录入键）与行里报的名字。
///
/// # Errors
/// 开不了库；录入段、登清单、读清单、判段各自的错。
///
/// # Panics
/// 选段写坏了。
pub fn two_stages<Judged>(
    class: CrashLibraryClass,
    item: &ItemName,
    inputs: &RecordingInputsDigest,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    let stage = RunStage::of_this_process();
    let library = CrashLibrary::open(class)?;
    let recording = recording_key_of_the_scenario_and_the_inputs(item.as_str(), inputs);
    two_stages_under_the_recording_key(&library, stage, item, &recording, record, judge)
}

/// [`two_stages`] 的本体：库、选段与录入键由调用方给（单测在一次性的库上跑）。
///
/// # Errors
/// 同 [`two_stages`]。
pub fn two_stages_under_the_recording_key<Judged>(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    two_stages_under_the_recording_key_topping_up_a_reused_recording(
        library,
        stage,
        item,
        recording,
        record,
        |_library| Ok(()),
        judge,
    )
}

/// 同 [`two_stages_under_the_recording_key`]，另带录制复用时补录的一段 `top_up_the_reused_recording`：录入键只罩录入段的一部分输入
/// （崩溃枚举用例的录入键只由流与场景定，挂着的观察者的版本不进键）时，命中之后录入那一段整段不跑，罩不到的那一部分由它在录入段里核、缺的补录；
/// 不命中、照录的那一趟不调它（`record` 自己录全）。只跑判段的那一趟不调它（录入段才写库）。
///
/// # Errors
/// 同 [`two_stages`]；补录那一段的错。
pub fn two_stages_under_the_recording_key_topping_up_a_reused_recording<Judged>(
    library: &CrashLibrary,
    stage: RunStage,
    item: &ItemName,
    recording: &RecordingKeyOfThisRun,
    record: impl FnOnce(&CrashLibrary) -> Result<ItemManifest, String>,
    top_up_the_reused_recording: impl FnOnce(&CrashLibrary) -> Result<(), String>,
    judge: impl FnOnce(&CrashLibrary, &ItemManifest) -> Result<StageOutcome<Judged>, String>,
) -> Result<StageOutcome<Judged>, String> {
    if let Some(refusal) = &recording.refusal {
        println!(
            "CRASH_LIBRARY_RECORDING_KEY_REFUSED class={} item={} {refusal}",
            library.class.name(),
            item.as_str()
        );
    }
    if stage.records() {
        let stored = if recording.skips_the_recording_on_a_hit {
            library.manifest_under(&recording.key)?
        } else {
            None
        };
        match stored {
            Some(manifest) => {
                println!(
                    "CRASH_LIBRARY_RECORDING_REUSED class={} item={} recording_key={}",
                    library.class.name(),
                    item.as_str(),
                    hexadecimal_text(&recording.key.0[..8])
                );
                library.print_the_recorded_line(item, &manifest);
                top_up_the_reused_recording(library)?;
            }
            None => {
                let manifest = record(library)?;
                library.record_the_item(item, &recording.key, &manifest)?;
            }
        }
    }
    if !stage.judges() {
        return Ok(StageOutcome::RecordedOnly);
    }
    let manifest = library.manifest_under(&recording.key)?.ok_or_else(|| {
        format!(
            "{} 类开的库 {} 里没有项「{}」这一份输入的清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
            library.class.name(),
            library.library_name,
            item.as_str()
        )
    })?;
    judge(library, &manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_item_manifest_round_trips_through_its_bytes() {
        let manifest = ItemManifest {
            kind: "crash_state_enumeration".to_string(),
            digests: vec![
                ("flow".to_string(), [1; DIGEST_BYTES]),
                ("tables".to_string(), [2; DIGEST_BYTES]),
            ],
        };
        let bytes = manifest.to_bytes();
        assert_eq!(ItemManifest::from_bytes(&bytes), Some(manifest.clone()));
        assert_eq!(manifest.digest_named("tables"), Ok([2; DIGEST_BYTES]));
        assert!(
            manifest.digest_named("facts").is_err(),
            "清单里没有的那一样要报错"
        );
        let mut truncated = bytes.clone();
        truncated.pop();
        assert_eq!(ItemManifest::from_bytes(&truncated), None, "字节不够解不出");
        let mut extended = bytes.clone();
        extended.push(0);
        assert_eq!(ItemManifest::from_bytes(&extended), None, "字节多出解不出");
        let mut wrong_magic = bytes;
        wrong_magic[0] ^= 0x01;
        assert_eq!(ItemManifest::from_bytes(&wrong_magic), None, "头不对解不出");
    }

    #[test]
    fn an_item_name_only_takes_the_allowed_characters() {
        assert!(ItemName::new("tree_split::central-mapping.root/split").is_ok());
        assert!(ItemName::new("").is_err());
        assert!(ItemName::new("有空格 的").is_err());
        assert!(ItemName::new(&"a".repeat(201)).is_err());
    }

    /// 录入段按（场景标识，录入输入摘要）入库（L11）：同一份输入第二趟命中、录入那一段不再跑；输入改一个字节第二趟照录；
    /// 同一份输入换一条场景照录（两条场景各一份清单）。
    /// 判别力（`crates/mutations.tsv`「录入键不拼场景标识」）：录入键不拼场景，换场景那一趟命中、不再录，这条红。
    #[test]
    fn the_recording_is_skipped_on_a_hit_and_redone_when_one_input_byte_or_the_scenario_changes() {
        // 一次性库留着：自证录入键命中与不中：要从空库数录了几次
        let disposable =
            DisposableLibrary::new("recording-reuse").unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let library =
            CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration).expect("开得了库");
        let item = ItemName::new("recording-reuse").expect("项名合规矩");
        let recordings = std::cell::Cell::new(0u32);
        let run_with_the_inputs = |scenario: &str, inputs: [u8; DIGEST_BYTES]| {
            two_stages_under_the_recording_key(
                &library,
                RunStage::RecordThenJudge,
                &item,
                &recording_key_of_the_scenario_and_the_inputs(
                    scenario,
                    &RecordingInputsDigest(inputs),
                ),
                |_library| {
                    recordings.set(recordings.get() + 1);
                    Ok(ItemManifest {
                        kind: "probe".to_string(),
                        digests: vec![("inputs".to_string(), inputs)],
                    })
                },
                |_library, manifest| Ok(StageOutcome::Judged(manifest.digest_named("inputs")?)),
            )
            .expect("两段过得了库")
            .judged()
            .expect("两段先后跑交回判的结果")
        };
        let mut inputs = [9u8; DIGEST_BYTES];
        assert_eq!(run_with_the_inputs("recording-reuse", inputs), inputs);
        assert_eq!(recordings.get(), 1, "头一趟现录");
        assert_eq!(run_with_the_inputs("recording-reuse", inputs), inputs);
        assert_eq!(recordings.get(), 1, "同一份输入第二趟命中、不再录");
        inputs[0] ^= 1;
        assert_eq!(run_with_the_inputs("recording-reuse", inputs), inputs);
        assert_eq!(recordings.get(), 2, "输入改一个字节第二趟现录");
        assert_eq!(run_with_the_inputs("another-scenario", inputs), inputs);
        assert_eq!(recordings.get(), 3, "同一份输入换一条场景照录");
    }
}
