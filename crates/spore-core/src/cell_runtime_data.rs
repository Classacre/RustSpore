//! Bounded E833B0 lookup from an explicit snapshot beginning at aggregate A0.
//!
//! The live pointer chain is object+17C -> pointee+2E4. Complete 99C170/99C210
//! accumulate explicit sampled animation data into these twenty-byte rows.
//! Authored ANIM kind3 descriptors and the original sampled-value caller are
//! recovered; active clip selection and the external actor/pose owner binding
//! remain explicit inputs. No PROP or ArgScript/Message association is inferred.

use crate::{
    native_sse::{self, Policy, SseError},
    native_x87::{Context, Extended, X87Error},
};
use std::fmt;

pub const RECORD_SIZE: usize = 20;
pub const PRODUCER_CAPACITY: usize = 8;
pub const AGGREGATE_SIZE: usize = 4 + RECORD_SIZE * PRODUCER_CAPACITY;
pub const DATA1_ID: u32 = 0x98c9_42b6;
pub const DATA2_ID: u32 = 0x98c9_42b5;
pub const DATA3_ID: u32 = 0x98c9_42b4;
pub const DATA4_ID: u32 = 0x98c9_42b3;

/// Original binary64 operand at 1447BB8, not a binary32 epsilon.
pub const DENOMINATOR_EPSILON_BITS: u64 = 0x3e45_798e_e230_8c3a;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellRuntimeDataError {
    MissingCount,
    /// Offset is relative to the first record at native aggregate A4.
    TruncatedRecords {
        offset: usize,
        needed: usize,
        available: usize,
    },
    RecordOffsetOverflow,
    UnsupportedProducerCount(u32),
    Sse(SseError),
    X87(X87Error),
}
impl fmt::Display for CellRuntimeDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded native Cell runtime data: {self:?}")
    }
}
impl std::error::Error for CellRuntimeDataError {}
impl From<SseError> for CellRuntimeDataError {
    fn from(value: SseError) -> Self {
        Self::Sse(value)
    }
}
impl From<X87Error> for CellRuntimeDataError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}

/// One complete runtime row, preserving its original payload verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeDataRecord<'a> {
    raw: &'a [u8; RECORD_SIZE],
}
impl<'a> CellRuntimeDataRecord<'a> {
    pub fn from_bytes(raw: &'a [u8; RECORD_SIZE]) -> Self {
        Self { raw }
    }

    pub fn raw_bytes(self) -> &'a [u8; RECORD_SIZE] {
        self.raw
    }

    pub fn id(self) -> u32 {
        self.word(0)
    }

    pub fn numerator_bits(self) -> u32 {
        self.word(8)
    }

    /// Literal second producer argument; callers supply scalar0/1 or pair2/3.
    pub fn raw_kind(self) -> u32 {
        self.word(4)
    }

    /// 99C210's second accumulator; 99C170 leaves this word untouched.
    pub fn second_value_bits(self) -> u32 {
        self.word(12)
    }

    pub fn denominator_bits(self) -> u32 {
        self.word(16)
    }

    fn word(self, offset: usize) -> u32 {
        u32::from_le_bytes(self.raw[offset..offset + 4].try_into().unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeDataWrite {
    /// Native AL only; incidental high EAX is not a canonical Boolean.
    pub accepted: bool,
    pub record: Option<usize>,
    pub inserted: bool,
}

/// Explicit count-plus-eight-row snapshot for complete 99C170/99C210 writes.
///
/// Inputs are already weighted by the caller, matching the original helper
/// boundary. All binary32 patterns remain observable, including nonfinite
/// inputs/results. The separate finite lookup API can reject reached values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRuntimeDataAccumulator {
    raw: [u8; AGGREGATE_SIZE],
}
impl CellRuntimeDataAccumulator {
    /// Counts outside0..8 would let the original scan leave its fixed storage;
    /// they are checked unsupported inputs here, rather than clamped counts.
    pub fn from_snapshot(raw: [u8; AGGREGATE_SIZE]) -> Result<Self, CellRuntimeDataError> {
        let count = u32::from_le_bytes(raw[..4].try_into().unwrap());
        if count > PRODUCER_CAPACITY as u32 {
            return Err(CellRuntimeDataError::UnsupportedProducerCount(count));
        }
        Ok(Self { raw })
    }

    pub fn raw_bytes(&self) -> &[u8; AGGREGATE_SIZE] {
        &self.raw
    }

    pub fn reader(&self) -> CellRuntimeData<'_> {
        CellRuntimeData::from_aggregate_a0(&self.raw).unwrap()
    }

    /// Complete99C170: update first matching ID or append when count<8.
    /// The second-value word at row+C is preserved on both paths.
    pub fn accumulate_scalar(
        &mut self,
        id: u32,
        raw_kind: u32,
        weighted_value_bits: u32,
        weight_bits: u32,
        policy: Policy,
    ) -> Result<CellRuntimeDataWrite, CellRuntimeDataError> {
        self.accumulate(id, raw_kind, weighted_value_bits, None, weight_bits, policy)
    }

    /// Complete99C210 with two explicit preweighted source values.
    /// Angle-to-sin/cos preparation occurs at the original caller, not here.
    pub fn accumulate_pair(
        &mut self,
        id: u32,
        raw_kind: u32,
        first_value_bits: u32,
        second_value_bits: u32,
        weight_bits: u32,
        policy: Policy,
    ) -> Result<CellRuntimeDataWrite, CellRuntimeDataError> {
        self.accumulate(
            id,
            raw_kind,
            first_value_bits,
            Some(second_value_bits),
            weight_bits,
            policy,
        )
    }

    fn accumulate(
        &mut self,
        id: u32,
        raw_kind: u32,
        first: u32,
        second: Option<u32>,
        weight: u32,
        policy: Policy,
    ) -> Result<CellRuntimeDataWrite, CellRuntimeDataError> {
        let count = u32::from_le_bytes(self.raw[..4].try_into().unwrap()) as usize;
        let found = (0..count).find(|&index| self.word(4 + index * RECORD_SIZE) == id);
        let (index, inserted) = match found {
            Some(index) => (index, false),
            None if count < PRODUCER_CAPACITY => (count, true),
            None => {
                return Ok(CellRuntimeDataWrite {
                    accepted: false,
                    record: None,
                    inserted: false,
                });
            }
        };
        let offset = 4 + index * RECORD_SIZE;
        let (first, second, weight) = if inserted {
            // Native MOVSS copies all input bits, without ADDSS/NaN quieting.
            (first, second, weight)
        } else {
            native_sse::scoped(policy, || -> Result<_, CellRuntimeDataError> {
                let first = add_incoming_first(first, self.word(offset + 8))?;
                let second = match second {
                    Some(incoming) => Some(add_incoming_first(incoming, self.word(offset + 12))?),
                    None => None,
                };
                let weight = add_incoming_first(weight, self.word(offset + 16))?;
                Ok((first, second, weight))
            })?
        };
        if inserted {
            self.store(0, (count + 1) as u32);
            self.store(offset, id);
        }
        self.store(offset + 8, first);
        if let Some(value) = second {
            self.store(offset + 12, value);
        }
        self.store(offset + 4, raw_kind);
        self.store(offset + 16, weight);
        Ok(CellRuntimeDataWrite {
            accepted: true,
            record: Some(index),
            inserted,
        })
    }

    fn word(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.raw[offset..offset + 4].try_into().unwrap())
    }

    fn store(&mut self, offset: usize, value: u32) {
        self.raw[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

fn add_incoming_first(incoming: u32, existing: u32) -> Result<u32, CellRuntimeDataError> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let result: u32;
        // SAFETY: register-only scalar SSE; the caller installed the explicit
        // masked policy. MOVD preserves every input bit, and ADDSS retains the
        // original incoming-left/existing-right NaN payload operand order.
        unsafe {
            std::arch::asm!(
                "movd {sum}, {incoming:e}",
                "movd {previous}, {existing:e}",
                "addss {sum}, {previous}",
                "movd {result:e}, {sum}",
                sum = out(xmm_reg) _,
                previous = out(xmm_reg) _,
                incoming = in(reg) incoming,
                existing = in(reg) existing,
                result = lateout(reg) result,
                options(nostack, preserves_flags),
            );
        }
        Ok(result)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = (incoming, existing);
        Err(SseError::UnsupportedArchitecture.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellRuntimeDataSample {
    /// The original callers test AL. Incidental high EAX bits are not modeled.
    pub found: bool,
    pub value: f32,
    /// Original ordered row index, before native MOV AL overwrites its low byte.
    pub matched_record: Option<usize>,
}

/// Borrowed snapshot with a signed count followed by original ordered records.
///
/// Native nonpositive counts read no records. Positive counts are checked as
/// the scan reaches each field, so unused trailing rows and unrelated numeric
/// payloads are not inspected. Extra bytes beyond the signed count are unused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeData<'a> {
    count: i32,
    records: &'a [u8],
}
impl<'a> CellRuntimeData<'a> {
    /// `bytes[0..4]` is native aggregate A0; records start at A4.
    pub fn from_aggregate_a0(bytes: &'a [u8]) -> Result<Self, CellRuntimeDataError> {
        let count = bytes.get(..4).ok_or(CellRuntimeDataError::MissingCount)?;
        Ok(Self {
            count: i32::from_le_bytes(count.try_into().unwrap()),
            records: &bytes[4..],
        })
    }

    pub fn signed_count(self) -> i32 {
        self.count
    }

    pub fn raw_record_bytes(self) -> &'a [u8] {
        self.records
    }

    /// Explicit complete-row view; lookup itself reads only reached fields.
    pub fn record(
        self,
        index: usize,
    ) -> Result<Option<CellRuntimeDataRecord<'a>>, CellRuntimeDataError> {
        if self.count <= 0 || index >= self.count as usize {
            return Ok(None);
        }
        let offset = Self::record_offset(index)?;
        let raw = self.read(offset, RECORD_SIZE)?.try_into().unwrap();
        Ok(Some(CellRuntimeDataRecord::from_bytes(raw)))
    }

    /// Complete E833B0 scan and finite arithmetic with an explicit x87 policy.
    ///
    /// The first matching ID wins. Missing data writes positive binary32 zero.
    /// The found path retains `numerator / (denominator + binary64 1e-8)` until
    /// its final FSTP32. Reached nonfinite inputs/results are typed API errors,
    /// including final binary32 overflow; the retail function has no such
    /// finite-only API guard. No fallback value replaces these errors.
    pub fn lookup(
        self,
        id: u32,
        context: Context,
    ) -> Result<CellRuntimeDataSample, CellRuntimeDataError> {
        if self.count > 0 {
            for index in 0..self.count as usize {
                let offset = Self::record_offset(index)?;
                let row_id = u32::from_le_bytes(self.read(offset, 4)?.try_into().unwrap());
                if row_id != id {
                    continue;
                }
                let row = self.record(index)?.unwrap();
                // Native FLD numerator precedes FLD denominator and FADD64.
                let numerator = Extended::from_f32(f32::from_bits(row.numerator_bits()))?;
                let denominator = Extended::from_f32(f32::from_bits(row.denominator_bits()))?;
                let epsilon = Extended::from_f64(f64::from_bits(DENOMINATOR_EPSILON_BITS))?;
                let denominator = context.add(denominator, epsilon)?;
                let retained = context.div(numerator, denominator)?;
                let value = context.store_f32(retained)?;
                return Ok(CellRuntimeDataSample {
                    found: true,
                    value,
                    matched_record: Some(index),
                });
            }
        }
        Ok(CellRuntimeDataSample {
            found: false,
            value: f32::from_bits(0),
            matched_record: None,
        })
    }

    fn record_offset(index: usize) -> Result<usize, CellRuntimeDataError> {
        index
            .checked_mul(RECORD_SIZE)
            .ok_or(CellRuntimeDataError::RecordOffsetOverflow)
    }

    fn read(self, offset: usize, length: usize) -> Result<&'a [u8], CellRuntimeDataError> {
        let end = offset
            .checked_add(length)
            .ok_or(CellRuntimeDataError::RecordOffsetOverflow)?;
        self.records
            .get(offset..end)
            .ok_or(CellRuntimeDataError::TruncatedRecords {
                offset,
                needed: length,
                available: self.records.len(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn context() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }

    fn row(id: u32, numerator: u32, denominator: u32) -> [u8; RECORD_SIZE] {
        let mut raw = [0xa5; RECORD_SIZE];
        raw[..4].copy_from_slice(&id.to_le_bytes());
        raw[8..12].copy_from_slice(&numerator.to_le_bytes());
        raw[16..20].copy_from_slice(&denominator.to_le_bytes());
        raw
    }

    #[test]
    fn original_order_and_reached_fields_control_lookup() {
        let mut raw = 3i32.to_le_bytes().to_vec();
        raw.extend(row(99, f32::NAN.to_bits(), f32::NAN.to_bits()));
        raw.extend(row(DATA2_ID, (-0f32).to_bits(), 2f32.to_bits()));
        raw.extend(row(DATA2_ID, f32::NAN.to_bits(), f32::NAN.to_bits()));
        let data = CellRuntimeData::from_aggregate_a0(&raw).unwrap();
        let sample = data.lookup(DATA2_ID, context()).unwrap();
        assert_eq!(sample.matched_record, Some(1));
        assert!(sample.found);
        assert_eq!(sample.value.to_bits(), 0x8000_0000);
        assert_eq!(
            &data.record(1).unwrap().unwrap().raw_bytes()[4..8],
            &[0xa5; 4]
        );
        assert_eq!(data.lookup(DATA1_ID, context()).unwrap().value.to_bits(), 0);
        assert_eq!(
            data.lookup(99, context()),
            Err(CellRuntimeDataError::X87(X87Error::NonFiniteInput))
        );
    }

    #[test]
    fn count_and_lazy_scan_preserve_unused_memory_boundaries() {
        for count in [i32::MIN, -1, 0] {
            let raw = count.to_le_bytes();
            let data = CellRuntimeData::from_aggregate_a0(&raw).unwrap();
            let sample = data.lookup(DATA1_ID, context()).unwrap();
            assert!(!sample.found);
            assert_eq!(sample.value.to_bits(), 0);
            assert_eq!(data.record(0).unwrap(), None);
        }
        let mut raw = 1i32.to_le_bytes().to_vec();
        raw.extend(DATA1_ID.to_le_bytes());
        let data = CellRuntimeData::from_aggregate_a0(&raw).unwrap();
        // An unmatched final ID reaches no numerator/denominator word.
        assert!(!data.lookup(DATA2_ID, context()).unwrap().found);
        assert!(matches!(
            data.lookup(DATA1_ID, context()),
            Err(CellRuntimeDataError::TruncatedRecords { .. })
        ));
        assert_eq!(
            CellRuntimeData::from_aggregate_a0(&[0; 3]),
            Err(CellRuntimeDataError::MissingCount)
        );
    }

    #[test]
    fn producer_preserves_unused_slots_and_rejects_new_ids_at_capacity() {
        let mut raw = [0xa5; AGGREGATE_SIZE];
        raw[..4].copy_from_slice(&0u32.to_le_bytes());
        let mut data = CellRuntimeDataAccumulator::from_snapshot(raw).unwrap();
        for index in 0..PRODUCER_CAPACITY {
            assert!(
                data.accumulate_scalar(
                    index as u32,
                    0x8000_0001,
                    0x7fa1_2345,
                    (-0f32).to_bits(),
                    Policy::Mxcsr1f80,
                )
                .unwrap()
                .inserted
            );
        }
        let snapshot = *data.raw_bytes();
        let result = data
            .accumulate_pair(99, 3, 0, 0, 0, Policy::Mxcsr1f80)
            .unwrap();
        assert!(!result.accepted);
        assert_eq!(data.raw_bytes(), &snapshot);
        for index in 0..PRODUCER_CAPACITY {
            let row = data.reader().record(index).unwrap().unwrap();
            assert_eq!(row.numerator_bits(), 0x7fa1_2345);
            assert_eq!(row.second_value_bits(), 0xa5a5_a5a5);
            assert_eq!(row.denominator_bits(), 0x8000_0000);
            assert_eq!(row.raw_kind(), 0x8000_0001);
        }
        assert!(matches!(
            CellRuntimeDataAccumulator::from_snapshot([0xff; AGGREGATE_SIZE]),
            Err(CellRuntimeDataError::UnsupportedProducerCount(u32::MAX))
        ));
    }
}
