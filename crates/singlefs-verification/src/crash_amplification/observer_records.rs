//! 观察者录下的东西在库里的字节：观察者底座、观察到的镜像、观察者块、镜像判定（`ObservedImageRecord`、
//! `ObserverBlockRecord`、`ImageJudgementRecord`），连同录的那一段共用的上下文（`ObserverRecordingContext`）。

use super::*;

// ------------------------------ 观察者两段写进库的几样 ------------------------------

const OBSERVER_FOUNDATION_MAGIC: &[u8; 8] = b"SFOBFND1";
const OBSERVED_IMAGE_MAGIC: &[u8; 8] = b"SFOBIMG2";
const OBSERVER_BLOCK_MAGIC: &[u8; 8] = b"SFOBBLK1";
const OBSERVED_IMAGE_JUDGEMENT_MAGIC: &[u8; 8] = b"SFOBJDG1";

/// 一条流的崩溃镜像共用的基底：基础镜像与枚举写表（录制流里的写在前，撕裂镜像与重放接在后面）。判段照它与持久集合重建崩溃镜像。
pub(super) struct ObserverFoundation {
    pub(super) base: MemoryPool,
    pub(super) enumerated_writes: Vec<RetainedWrite>,
}

pub(super) fn observer_foundation_bytes(
    base: &MemoryPool,
    enumerated_writes: &[RetainedWrite],
) -> Vec<u8> {
    let mut bytes = OBSERVER_FOUNDATION_MAGIC.to_vec();
    put_pool_writes_and_segments(&mut bytes, base, enumerated_writes, &[]);
    bytes
}

impl ObserverFoundation {
    /// 头不对、字节不够或多出、带了切段交回 `None`。
    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVER_FOUNDATION_MAGIC.len())? != OBSERVER_FOUNDATION_MAGIC {
            return None;
        }
        let (base, enumerated_writes, segments) = read_pool_writes_and_segments(&mut reader)?;
        (segments.is_empty() && reader.at == bytes.len()).then_some(Self {
            base,
            enumerated_writes,
        })
    }
}

/// 观察者交回、判段要在上面跑池级 checker 的那张盘：基底摘要、这个状态的持久集合（按枚举写表）、观察者在崩溃镜像上发的写、判哪几条、
/// 判干净时给哪个计数加一。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImageToJudgeAfterTheObserver {
    pub(super) foundation_digest: [u8; DIGEST_BYTES],
    pub(super) persisted: Vec<bool>,
    pub(super) writes_on_the_crash_image: Vec<RetainedWrite>,
    pub(super) invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver,
    pub(super) counter_when_the_pool_checker_is_clean: String,
}

/// 交回镜像的观察者在一张恢复镜像上看出的那一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObservedImageRecord {
    pub(super) red_item: Option<String>,
    pub(super) counts: Vec<(String, u64)>,
    pub(super) image_to_judge: Option<ImageToJudgeAfterTheObserver>,
}

pub(super) fn put_text(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend_from_slice(
        &u64::try_from(text.len())
            .expect("长度装得进 u64")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(text.as_bytes());
}

fn read_text(reader: &mut RecordedBytesReader<'_>) -> Option<String> {
    String::from_utf8(reader.bytes()?.to_vec()).ok()
}

fn put_optional_text(bytes: &mut Vec<u8>, text: Option<&str>) {
    match text {
        None => bytes.push(0),
        Some(text) => {
            bytes.push(1);
            put_text(bytes, text);
        }
    }
}

fn read_optional_text(reader: &mut RecordedBytesReader<'_>) -> Option<Option<String>> {
    match reader.u8()? {
        0 => Some(None),
        1 => Some(Some(read_text(reader)?)),
        _ => None,
    }
}

/// 持久集合按游程存：先写总长，再写交替的游程长度，从「没落」的那一段起（可以是 0）。枚举一个段的子集时，段前全落、段后全没落，
/// 几十万条写的集合只有几条游程（早先一字节一条写，一张镜像的记录约 250 KB、库写入成了录入段的瓶颈）。
fn put_persisted_as_runs(bytes: &mut Vec<u8>, persisted: &[bool]) {
    put_item_count(bytes, persisted.len());
    let mut runs: Vec<u64> = Vec::new();
    let mut current = false;
    let mut length = 0u64;
    for is_persisted in persisted {
        if *is_persisted == current {
            length += 1;
        } else {
            runs.push(length);
            current = *is_persisted;
            length = 1;
        }
    }
    runs.push(length);
    put_item_count(bytes, runs.len());
    for run in runs {
        bytes.extend_from_slice(&run.to_le_bytes());
    }
}

fn read_persisted_from_runs(reader: &mut RecordedBytesReader<'_>) -> Option<Vec<bool>> {
    let total = reader.count()?;
    let mut persisted = Vec::with_capacity(total);
    let mut current = false;
    for _ in 0..reader.count()? {
        let run = usize::try_from(reader.u64()?).ok()?;
        persisted.extend(std::iter::repeat_n(current, run));
        current = !current;
    }
    (persisted.len() == total).then_some(persisted)
}

fn put_counts(bytes: &mut Vec<u8>, counts: &[(String, u64)]) {
    bytes.extend_from_slice(
        &u64::try_from(counts.len())
            .expect("长度装得进 u64")
            .to_le_bytes(),
    );
    for (name, count) in counts {
        put_text(bytes, name);
        bytes.extend_from_slice(&count.to_le_bytes());
    }
}

fn read_counts(reader: &mut RecordedBytesReader<'_>) -> Option<Vec<(String, u64)>> {
    let mut counts = Vec::new();
    for _ in 0..reader.count()? {
        let name = read_text(reader)?;
        counts.push((name, reader.u64()?));
    }
    Some(counts)
}

/// 不变量名字认回 `IMPLEMENTED_INVARIANTS` 里那一条；认不得交回 `None`（库是别的 checker 版本写的）。
fn implemented_invariant_named(name: &str) -> Option<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .find(|invariant| *invariant == name)
}

impl ObservedImageRecord {
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = OBSERVED_IMAGE_MAGIC.to_vec();
        put_optional_text(&mut bytes, self.red_item.as_deref());
        put_counts(&mut bytes, &self.counts);
        match &self.image_to_judge {
            None => bytes.push(0),
            Some(image) => {
                bytes.push(1);
                bytes.extend_from_slice(&image.foundation_digest);
                put_persisted_as_runs(&mut bytes, &image.persisted);
                put_pool_writes_and_segments(
                    &mut bytes,
                    &MemoryPool::default(),
                    &image.writes_on_the_crash_image,
                    &[],
                );
                match &image.invariants_judged {
                    InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant => {
                        bytes.push(0);
                    }
                    InvariantsJudgedOnTheImageAfterTheObserver::Only(invariants) => {
                        bytes.push(1);
                        bytes.extend_from_slice(
                            &u64::try_from(invariants.len())
                                .expect("长度装得进 u64")
                                .to_le_bytes(),
                        );
                        for invariant in invariants {
                            put_text(&mut bytes, invariant);
                        }
                    }
                }
                put_text(&mut bytes, &image.counter_when_the_pool_checker_is_clean);
            }
        }
        bytes
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVED_IMAGE_MAGIC.len())? != OBSERVED_IMAGE_MAGIC {
            return None;
        }
        let red_item = read_optional_text(&mut reader)?;
        let counts = read_counts(&mut reader)?;
        let image_to_judge = match reader.u8()? {
            0 => None,
            1 => {
                let foundation_digest: [u8; DIGEST_BYTES] =
                    reader.take(DIGEST_BYTES)?.try_into().ok()?;
                let persisted = read_persisted_from_runs(&mut reader)?;
                let (empty_pool, writes_on_the_crash_image, segments) =
                    read_pool_writes_and_segments(&mut reader)?;
                if !empty_pool.devices.is_empty() || !segments.is_empty() {
                    return None;
                }
                let invariants_judged = match reader.u8()? {
                    0 => InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
                    1 => {
                        let mut invariants = Vec::new();
                        for _ in 0..reader.count()? {
                            invariants.push(implemented_invariant_named(&read_text(&mut reader)?)?);
                        }
                        InvariantsJudgedOnTheImageAfterTheObserver::Only(invariants)
                    }
                    _ => return None,
                };
                let counter_when_the_pool_checker_is_clean = read_text(&mut reader)?;
                Some(ImageToJudgeAfterTheObserver {
                    foundation_digest,
                    persisted,
                    writes_on_the_crash_image,
                    invariants_judged,
                    counter_when_the_pool_checker_is_clean,
                })
            }
            _ => return None,
        };
        (reader.at == bytes.len()).then_some(Self {
            red_item,
            counts,
            image_to_judge,
        })
    }
}

/// 观察者录入段一块里一个状态落在哪一份观察结果上。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ObservationOfStates {
    /// 只记计数的观察者：这个状态的红项（观察者 panic 的那一句）与计数。
    Counted {
        red_item: Option<String>,
        counts: Vec<(String, u64)>,
    },
    /// 交回镜像的观察者：那张恢复镜像的去重键（结果在看过的恢复镜像那张表里）。
    ImageObserved {
        deduplication_key: [u8; DIGEST_BYTES],
    },
}

/// 观察者录入段一块：去重之后的观察结果清单、每个状态的下标、按发布分的态数（计数向量里观察者计数与不变量那两节空着）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObserverBlockRecord {
    pub(super) observations: Vec<ObservationOfStates>,
    pub(super) observation_of_each_state: Vec<u32>,
    pub(super) count_vector: BlockCountVector,
}

impl ObserverBlockRecord {
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = OBSERVER_BLOCK_MAGIC.to_vec();
        bytes.extend_from_slice(
            &u64::try_from(self.observations.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        for observation in &self.observations {
            match observation {
                ObservationOfStates::Counted { red_item, counts } => {
                    bytes.push(0);
                    put_optional_text(&mut bytes, red_item.as_deref());
                    put_counts(&mut bytes, counts);
                }
                ObservationOfStates::ImageObserved { deduplication_key } => {
                    bytes.push(1);
                    bytes.extend_from_slice(deduplication_key);
                }
            }
        }
        bytes.extend_from_slice(
            &u64::try_from(self.observation_of_each_state.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        for index in &self.observation_of_each_state {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        let count_vector = crate::verdict_store::count_vector_bytes(&self.count_vector);
        bytes.extend_from_slice(
            &u64::try_from(count_vector.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&count_vector);
        bytes
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVER_BLOCK_MAGIC.len())? != OBSERVER_BLOCK_MAGIC {
            return None;
        }
        let mut observations = Vec::new();
        for _ in 0..reader.count()? {
            observations.push(match reader.u8()? {
                0 => ObservationOfStates::Counted {
                    red_item: read_optional_text(&mut reader)?,
                    counts: read_counts(&mut reader)?,
                },
                1 => ObservationOfStates::ImageObserved {
                    deduplication_key: reader.take(DIGEST_BYTES)?.try_into().ok()?,
                },
                _ => return None,
            });
        }
        let mut observation_of_each_state = Vec::new();
        for _ in 0..reader.count()? {
            let index = reader.u32()?;
            if usize::try_from(index).ok()? >= observations.len() {
                return None;
            }
            observation_of_each_state.push(index);
        }
        let count_vector = crate::verdict_store::count_vector_from_bytes(reader.bytes()?)?;
        (reader.at == bytes.len()).then_some(Self {
            observations,
            observation_of_each_state,
            count_vector,
        })
    }
}

/// 判段在观察者交回的一张盘上跑池级 checker 的结论：判哪几条里违例的、GPU 判不了的原因（记一条、判红）、在哪判的。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImageJudgementRecord {
    pub(super) violated_invariants: Vec<String>,
    pub(super) could_not_judge: Option<String>,
    pub(super) judged_on: JudgingDevice,
}

impl ImageJudgementRecord {
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = OBSERVED_IMAGE_JUDGEMENT_MAGIC.to_vec();
        bytes.extend_from_slice(
            &u64::try_from(self.violated_invariants.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        for invariant in &self.violated_invariants {
            put_text(&mut bytes, invariant);
        }
        put_optional_text(&mut bytes, self.could_not_judge.as_deref());
        let device = self.judged_on.to_attribute_bytes();
        bytes.extend_from_slice(
            &u64::try_from(device.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&device);
        bytes
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVED_IMAGE_JUDGEMENT_MAGIC.len())? != OBSERVED_IMAGE_JUDGEMENT_MAGIC {
            return None;
        }
        let mut violated_invariants = Vec::new();
        for _ in 0..reader.count()? {
            violated_invariants.push(read_text(&mut reader)?);
        }
        let could_not_judge = read_optional_text(&mut reader)?;
        let judged_on = JudgingDevice::from_attribute_bytes(reader.bytes()?)?;
        (reader.at == bytes.len()).then_some(Self {
            violated_invariants,
            could_not_judge,
            judged_on,
        })
    }

    /// 这一份结论在那个状态的判定向量里的红项：违例的不变量逐条、GPU 判不了的那一句。
    pub(super) fn red_items(&self, observer_name: &ObserverName) -> Vec<String> {
        let mut items: Vec<String> = self
            .violated_invariants
            .iter()
            .map(|invariant| {
                observer_red_item(
                    observer_name,
                    &format!("观察者交回的盘上池级 checker 判违例：{invariant}"),
                )
            })
            .collect();
        if let Some(reason) = &self.could_not_judge {
            items.push(observer_red_item(
                observer_name,
                &format!("GPU 判不了观察者交回的盘：{reason}"),
            ));
        }
        items
    }

    /// 判出违例没有（判器判出来的结论里有违例的不变量）：记着红的下一趟不复用、现判。
    pub(super) fn is_red(&self) -> bool {
        !self.violated_invariants.is_empty()
    }

    /// 这一份是判器判出来的结论（能存进库、下一趟同一把判器版本照复用），还是「GPU 判不了」：建不出判表、表越过内核容量不是判器的结论，
    /// 建表那一截的代码（这份文件）不在判器那一把版本里，存了下一趟同一把版本照复用、建表改好了也还是判不了；这一趟照样判红，不存，下一趟重判。
    pub(super) fn is_a_conclusion_of_the_judge(&self) -> bool {
        self.could_not_judge.is_none()
    }
}

/// 判段这一批判过的盘里存进库的那几张（去重键，结论字节）：只存判器判出来的结论（[`ImageJudgementRecord::is_a_conclusion_of_the_judge`]）。
pub(super) fn image_judgements_to_store(
    images: &[ImageAfterTheObserverToJudge<'_>],
    judged: &[ImageJudgementRecord],
) -> Vec<([u8; DIGEST_BYTES], Vec<u8>)> {
    images
        .iter()
        .zip(judged)
        .filter(|(_, judgement)| judgement.is_a_conclusion_of_the_judge())
        .map(|(image, judgement)| (image.deduplication_key, judgement.to_bytes()))
        .collect()
}

/// 观察者拼好的一块存不存进库：块里每张盘的结论都是判器判出来的才存；有一张 GPU 判不了，这一块照样判红、计数，不存（下一趟重判那一张、重拼这一块）。
pub(super) fn block_conclusion_is_kept_in_the_library(
    judgement_of_each_observation: &[Option<ImageJudgementRecord>],
) -> bool {
    judgement_of_each_observation
        .iter()
        .flatten()
        .all(ImageJudgementRecord::is_a_conclusion_of_the_judge)
}

// ------------------------------ 录入段 ------------------------------

/// 一个观察者录入一段状态（块的一段，一条核对线程）交回的：逐状态落在哪一份观察结果上、按发布分的态数、这一段的账。
pub(super) struct RecordedObserverPiece {
    pub(super) local: Range<u64>,
    pub(super) observation_of_each_state: Vec<ObservationOfStates>,
    pub(super) state_counts: StateCountsOfABlock,
    pub(super) tally: ObserverRecordingTally,
}

/// 录入段的这一份看一个状态的观察者要的：这条流的读集见证摘要器与基底摘要（交回镜像的观察者才要）、这个观察者这一趟每个桶的见证索引（库里的读进来、
/// 这一趟新添的原地添进去）、库、这一块已经看过几个状态（按挂钟那一行报）。锁的次序固定：桶表、桶的见证索引、库；拿着库锁的不再拿别的锁，
/// 拿着见证索引读锁的只查表、不碰库。
pub(super) struct ObserverRecordingContext<'context, 'store> {
    /// 这条流排好次序的全部状态：枚举的表录入段只建一份，各段（各核对线程）借同一份走自己那一段，不逐段各建一份。
    pub(super) crash_states_of_the_flow:
        &'context crate::crash::CrashStatesOfTheReadBackFlow<'context>,
    pub(super) digester: Option<&'context ReadSetWitnessDigester<'context>>,
    pub(super) foundation_digest: [u8; DIGEST_BYTES],
    pub(super) witnesses_of_each_bucket: &'context WitnessIndexesOfEachBucket,
    /// 每个桶一把「真跑」的锁：见证一个都对不上的线程先拿它、再核一次见证，还对不上才真跑。读集见证的键要跑完才知道，不锁的话同一桶里
    /// 几个线程会各自真跑同一张镜像（线程数那么多次）。桶与桶之间照旧并行，对上见证的状态不碰这把锁。
    pub(super) run_locks_of_each_bucket: &'context RunLocksOfEachBucket,
    pub(super) store: &'context LibraryTakenAtEachReadOrWrite<'store>,
    pub(super) states_looked_at_in_the_block: &'context std::sync::atomic::AtomicU64,
}

/// 一个观察者这一趟每个桶的见证索引：桶键到这个桶唯一的那一份（[`ReadSetWitnessIndexOfABucket`]）。
pub(super) type WitnessIndexesOfEachBucket = std::sync::Mutex<
    std::collections::HashMap<[u8; DIGEST_BYTES], std::sync::Arc<ReadSetWitnessIndexOfABucket>>,
>;

/// 一个观察者这一趟每个桶的真跑锁：桶键到这个桶唯一的那一把（[`ObserverRecordingContext`] 的 `run_locks_of_each_bucket`）。
pub(super) type RunLocksOfEachBucket = std::sync::Mutex<
    std::collections::HashMap<[u8; DIGEST_BYTES], std::sync::Arc<std::sync::Mutex<()>>>,
>;

pub(super) fn observer_library_error(error: &VerdictStoreError) -> String {
    format!("观察者录入段读写库失败：{error:?}")
}

/// 从库里读回一个桶的见证、建索引：按桶键前缀扫出全部见证行，每组读集行只读一次、核过摘要，几条见证共用那一份段。库里没有这个桶的见证就是空桶。
///
/// # Errors
/// 库读失败；见证行引的读集行不在库里，或读集行的字节解不出、摘要与键不同（见证与它引的读集同一批写，缺了或不对就是库坏了）。
pub(super) fn read_the_witness_index_of_the_bucket(
    store: &VerdictStore,
    bucket_key: &[u8; DIGEST_BYTES],
) -> Result<ReadSetWitnessIndex, String> {
    let rows = store
        .read_observer_read_set_witnesses_of_the_bucket(bucket_key)
        .map_err(|error| observer_library_error(&error))?;
    let mut read_set_of_each_digest: std::collections::HashMap<
        [u8; DIGEST_BYTES],
        ObservedReadSet,
    > = std::collections::HashMap::new();
    let mut witnesses = Vec::with_capacity(rows.len());
    for row in rows {
        let read_set = match read_set_of_each_digest.get(&row.read_set_digest) {
            Some(read_set) => read_set.clone(),
            None => {
                let bytes = store
                    .read_observer_read_set(&row.read_set_digest)
                    .map_err(|error| observer_library_error(&error))?
                    .ok_or_else(|| {
                        format!(
                            "桶 {} 的见证引的读集 {} 不在库里（见证与它引的读集同一批写，缺了是库坏了）",
                            hexadecimal_text(&bucket_key[..8]),
                            hexadecimal_text(&row.read_set_digest[..8])
                        )
                    })?;
                let read_set = ObservedReadSet::of_the_stored_bytes(&row.read_set_digest, &bytes)
                    .ok_or_else(|| {
                    format!(
                        "桶 {} 的见证引的读集 {} 解不出，或字节的摘要与键不同",
                        hexadecimal_text(&bucket_key[..8]),
                        hexadecimal_text(&row.read_set_digest[..8])
                    )
                })?;
                read_set_of_each_digest.insert(row.read_set_digest, read_set.clone());
                read_set
            }
        };
        witnesses.push(ReadSetWitness {
            read_set,
            digest: row.witness_digest,
            observed_image_key: row.observed_image_key,
        });
    }
    Ok(ReadSetWitnessIndex::of_the_stored_witnesses(witnesses))
}

impl ObserverRecordingContext<'_, '_> {
    /// 一个桶的「真跑」锁（没有就建）。
    pub(super) fn run_lock_of_the_bucket(
        &self,
        bucket_key: &[u8; DIGEST_BYTES],
    ) -> std::sync::Arc<std::sync::Mutex<()>> {
        self.run_locks_of_each_bucket
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(*bucket_key)
            .or_default()
            .clone()
    }

    /// 一个桶的见证索引：这一趟没碰过的桶先从库里读、建索引（库里没有就是空桶）。交回的是这个桶唯一那一份的共享引用（不是拷贝）：
    /// 之后添的见证原地添进去，拿着它的线程下一次查就看得到。
    pub(super) fn witnesses_of_the_bucket(
        &self,
        bucket_key: &[u8; DIGEST_BYTES],
    ) -> Result<std::sync::Arc<ReadSetWitnessIndexOfABucket>, String> {
        let mut witnesses = self
            .witnesses_of_each_bucket
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(known) = witnesses.get(bucket_key) {
            return Ok(std::sync::Arc::clone(known));
        }
        let loaded = read_the_witness_index_of_the_bucket(&self.store.take(), bucket_key)?;
        let loaded = std::sync::Arc::new(std::sync::RwLock::new(loaded));
        witnesses.insert(*bucket_key, std::sync::Arc::clone(&loaded));
        Ok(loaded)
    }

    /// 观察者真跑了一张镜像：见证行（读集在这个桶里头一回出现时连同读集行）与那张镜像一批落盘（见证在、它引的读集与镜像必在），
    /// 落盘之后才原地添进这个桶的索引。另一个线程刚添了同样摘要的见证时不再添（它那一次已经连镜像落了盘，两次看出的记录逐字节相同）。
    /// 拿着这个桶索引的写锁做完这几步：读者看到的是添之前或添之后的整份索引；落盘失败索引不变。写下的字节只随这一条见证，与桶里已有几条无关。
    pub(super) fn store_the_image_with_its_witness(
        &self,
        bucket_key: &[u8; DIGEST_BYTES],
        witness: ReadSetWitness,
        image_bytes: &[u8],
    ) -> Result<(), String> {
        let of_the_bucket = std::sync::Arc::clone(
            self.witnesses_of_each_bucket
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry(*bucket_key)
                .or_insert_with(|| {
                    std::sync::Arc::new(std::sync::RwLock::new(ReadSetWitnessIndex::empty()))
                }),
        );
        let mut index = of_the_bucket
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if index.holds_the_witness_digest(&witness.digest) {
            return Ok(());
        }
        let read_set_is_new_in_the_bucket = !index.holds_the_read_set(&witness.read_set.digest);
        let read_set_bytes = read_set_is_new_in_the_bucket
            .then(|| observed_read_set_bytes(&witness.read_set.ranges));
        self.store
            .take()
            .store_observed_image_with_its_read_set_witness(
                bucket_key,
                &witness.row(),
                read_set_bytes.as_deref(),
                image_bytes,
            )
            .map_err(|error| observer_library_error(&error))?;
        match index.add(witness) {
            ReadSetWitnessAddition::AddedToAKnownReadSet
            | ReadSetWitnessAddition::AddedWithANewReadSet => Ok(()),
            ReadSetWitnessAddition::AlreadyInTheBucket => {
                unreachable!("拿着这个桶的写锁、上面核过这个摘要不在桶里，添不会撞上已有的")
            }
        }
    }

    /// 不带见证的一张镜像（关掉去重、观察者 panic）：单存。
    pub(super) fn store_the_image(
        &self,
        deduplication_key: &[u8; DIGEST_BYTES],
        image_bytes: &[u8],
    ) -> Result<(), String> {
        self.store
            .take()
            .store_observed_images(&[(*deduplication_key, image_bytes.to_vec())])
            .map_err(|error| observer_library_error(&error))
    }
}

#[cfg(test)]
mod tests;
