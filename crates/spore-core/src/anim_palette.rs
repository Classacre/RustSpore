//! Native palette gathering from explicitly supplied runtime groups and poses.
//!
//! `73B7C0` bounds the generated table by the first runtime group, and `740530`
//! gathers that prefix with identity or evaluated SkinsInK matrices. This module
//! does not construct a rig, evaluate curves, or infer palette indices from
//! source CreatureData. See `re/anim-palette-native.md` for the input boundary.

use std::fmt;

/// Three affine rows of four floats, copied without changing their bit patterns.
pub type BoneMatrix = [f32; 12];

pub const IDENTITY: BoneMatrix = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0.];

#[derive(Debug, Clone, Copy)]
pub enum GroupMatrices<'a> {
    /// The native group has no external resource (resource index FF).
    Identity,
    /// Already evaluated matrices of the exact group's SkinsInK binding.
    Evaluated(&'a [BoneMatrix]),
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimePaletteGroup<'a> {
    /// Raw runtime group+2 word; the native gatherer reads it as signed i16.
    pub first_bone: u16,
    /// Raw runtime group+1 byte; the native consumers read it as signed i8.
    pub bone_count: u8,
    pub matrices: GroupMatrices<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct GeneratedTable<'a> {
    pub requested_count: usize,
    /// None represents the native null table pointer. The stored count still
    /// changes, but the gatherer then uses each group's complete fallback.
    pub matrices: Option<&'a [BoneMatrix]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteError {
    RequestedCountOutOfRange(usize),
    NegativeGroupCount {
        group: usize,
        raw: u8,
    },
    NegativeGroupOffset {
        group: usize,
        raw: u16,
    },
    NonContiguousGroup {
        group: usize,
        expected: usize,
        actual: usize,
    },
    ShortGeneratedTable {
        required: usize,
        actual: usize,
    },
    ShortEvaluatedGroup {
        group: usize,
        required: usize,
        actual: usize,
    },
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestedCountOutOfRange(value) => {
                write!(f, "generated count {value} exceeds the native signed count")
            }
            Self::NegativeGroupCount { group, raw } => {
                write!(f, "group {group} has negative native count byte {raw:02X}")
            }
            Self::NegativeGroupOffset { group, raw } => {
                write!(f, "group {group} has negative native offset word {raw:04X}")
            }
            Self::NonContiguousGroup {
                group,
                expected,
                actual,
            } => write!(
                f,
                "group {group} starts at {actual}; contiguous runtime order requires {expected}"
            ),
            Self::ShortGeneratedTable { required, actual } => write!(
                f,
                "generated table has {actual} matrices; {required} are read"
            ),
            Self::ShortEvaluatedGroup {
                group,
                required,
                actual,
            } => write!(
                f,
                "group {group} has {actual} evaluated matrices; {required} are read"
            ),
        }
    }
}

impl std::error::Error for PaletteError {}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteOutput {
    /// Count stored by 73B7C0, even if the table pointer is null or the gatherer
    /// is ineligible. A nonempty group list bounds this to its first group.
    pub bound_generated_count: usize,
    pub copied_generated_count: usize,
    pub matrices: Vec<BoneMatrix>,
}

/// Gather a palette at the measured native boundary, using explicit poses.
///
/// `matrix_storage_ready` supplies the native nonempty +0x64..+0x68 storage
/// condition. Ineligible storage or an empty group list yields no matrices.
/// Eligible runtime groups must retain their native contiguous order; signed
/// negative counts and offsets are rejected at this bounded Rust boundary.
///
/// The generated table replaces a prefix bounded by the first group's count.
/// Remaining rows come from identity or already evaluated group buffers. A
/// fully covered group does not require an evaluated buffer. Matrix values,
/// including signed zero and nonfinite payloads, are copied without arithmetic.
pub fn gather_runtime_palette(
    matrix_storage_ready: bool,
    groups: &[RuntimePaletteGroup<'_>],
    generated: GeneratedTable<'_>,
) -> Result<PaletteOutput, PaletteError> {
    if generated.requested_count > i32::MAX as usize {
        return Err(PaletteError::RequestedCountOutOfRange(
            generated.requested_count,
        ));
    }
    let bound_generated_count = if let Some(first) = groups.first() {
        if first.bone_count & 0x80 != 0 {
            return Err(PaletteError::NegativeGroupCount {
                group: 0,
                raw: first.bone_count,
            });
        }
        generated.requested_count.min(usize::from(first.bone_count))
    } else {
        generated.requested_count
    };
    let mut output = PaletteOutput {
        bound_generated_count,
        copied_generated_count: 0,
        matrices: Vec::new(),
    };
    if !matrix_storage_ready || groups.is_empty() {
        return Ok(output);
    }
    let mut total = 0usize;
    for (group, value) in groups.iter().enumerate() {
        if value.bone_count & 0x80 != 0 {
            return Err(PaletteError::NegativeGroupCount {
                group,
                raw: value.bone_count,
            });
        }
        if value.first_bone & 0x8000 != 0 {
            return Err(PaletteError::NegativeGroupOffset {
                group,
                raw: value.first_bone,
            });
        }
        let actual = usize::from(value.first_bone);
        if actual != total {
            return Err(PaletteError::NonContiguousGroup {
                group,
                expected: total,
                actual,
            });
        }
        total += usize::from(value.bone_count);
    }
    output.matrices.resize(total, IDENTITY);
    if let Some(matrices) = generated.matrices {
        let copied = bound_generated_count.min(total);
        if matrices.len() < copied {
            return Err(PaletteError::ShortGeneratedTable {
                required: copied,
                actual: matrices.len(),
            });
        }
        output.matrices[..copied].copy_from_slice(&matrices[..copied]);
        output.copied_generated_count = copied;
    }
    for (group, value) in groups.iter().enumerate() {
        let first = usize::from(value.first_bone);
        let count = usize::from(value.bone_count);
        let skipped = if generated.matrices.is_some() {
            bound_generated_count.saturating_sub(first)
        } else {
            0
        };
        if skipped >= count {
            continue;
        }
        if let GroupMatrices::Evaluated(matrices) = value.matrices {
            if matrices.len() < count {
                return Err(PaletteError::ShortEvaluatedGroup {
                    group,
                    required: count,
                    actual: matrices.len(),
                });
            }
            output.matrices[first + skipped..first + count]
                .copy_from_slice(&matrices[skipped..count]);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(value: u32) -> BoneMatrix {
        std::array::from_fn(|i| f32::from_bits(0x3F00_0000 + value * 0x100 + i as u32))
    }

    #[test]
    fn first_group_limits_generated_prefix_before_external_groups() {
        let generated = [marker(10), marker(11), marker(12), marker(13), marker(14)];
        let evaluated = [marker(20), marker(21)];
        let groups = [
            RuntimePaletteGroup {
                first_bone: 0,
                bone_count: 3,
                matrices: GroupMatrices::Identity,
            },
            RuntimePaletteGroup {
                first_bone: 3,
                bone_count: 2,
                matrices: GroupMatrices::Evaluated(&evaluated),
            },
        ];
        let result = gather_runtime_palette(
            true,
            &groups,
            GeneratedTable {
                requested_count: 5,
                matrices: Some(&generated),
            },
        )
        .unwrap();
        assert_eq!(result.bound_generated_count, 3);
        assert_eq!(result.copied_generated_count, 3);
        assert_eq!(
            result.matrices,
            [
                generated[0],
                generated[1],
                generated[2],
                evaluated[0],
                evaluated[1]
            ]
        );
        let partial = gather_runtime_palette(
            true,
            &groups,
            GeneratedTable {
                requested_count: 2,
                matrices: Some(&generated),
            },
        )
        .unwrap();
        assert_eq!(
            partial.matrices,
            [
                generated[0],
                generated[1],
                IDENTITY,
                evaluated[0],
                evaluated[1]
            ]
        );
    }

    #[test]
    fn partial_external_first_group_uses_exact_evaluated_suffix() {
        let generated = [marker(10), marker(11), marker(12)];
        let evaluated = [marker(20), marker(21), marker(22)];
        let groups = [RuntimePaletteGroup {
            first_bone: 0,
            bone_count: 3,
            matrices: GroupMatrices::Evaluated(&evaluated),
        }];
        let result = gather_runtime_palette(
            true,
            &groups,
            GeneratedTable {
                requested_count: 1,
                matrices: Some(&generated),
            },
        )
        .unwrap();
        assert_eq!(result.matrices, [generated[0], evaluated[1], evaluated[2]]);
        let covered = [RuntimePaletteGroup {
            matrices: GroupMatrices::Evaluated(&[]),
            ..groups[0]
        }];
        assert_eq!(
            gather_runtime_palette(
                true,
                &covered,
                GeneratedTable {
                    requested_count: 3,
                    matrices: Some(&generated)
                }
            )
            .unwrap()
            .matrices,
            generated
        );
    }

    #[test]
    fn null_table_and_ineligible_storage_keep_native_branch_distinct() {
        let evaluated = [marker(20), marker(21)];
        let groups = [RuntimePaletteGroup {
            first_bone: 0,
            bone_count: 2,
            matrices: GroupMatrices::Evaluated(&evaluated),
        }];
        let result = gather_runtime_palette(
            true,
            &groups,
            GeneratedTable {
                requested_count: 99,
                matrices: None,
            },
        )
        .unwrap();
        assert_eq!(result.bound_generated_count, 2);
        assert_eq!(result.copied_generated_count, 0);
        assert_eq!(result.matrices, evaluated);
        let missing = [RuntimePaletteGroup {
            matrices: GroupMatrices::Evaluated(&[]),
            ..groups[0]
        }];
        assert!(gather_runtime_palette(
            false,
            &missing,
            GeneratedTable {
                requested_count: 99,
                matrices: None
            }
        )
        .unwrap()
        .matrices
        .is_empty());
        let empty = gather_runtime_palette(
            true,
            &[],
            GeneratedTable {
                requested_count: 99,
                matrices: None,
            },
        )
        .unwrap();
        assert_eq!(empty.bound_generated_count, 99);
        assert!(empty.matrices.is_empty());
    }

    #[test]
    fn matrix_copies_preserve_float_bits() {
        let raw = [
            0x8000_0000,
            0x7F80_0000,
            0xFF80_0000,
            0x7FA1_2345,
            0xFFA1_2345,
            1,
            0x8000_0001,
            0,
            0x3F80_0000,
            0xBF80_0000,
            0x7FC0_1234,
            0xFFCF_EDCB,
        ];
        let matrix = raw.map(f32::from_bits);
        let evaluated = [matrix];
        let groups = [RuntimePaletteGroup {
            first_bone: 0,
            bone_count: 1,
            matrices: GroupMatrices::Evaluated(&evaluated),
        }];
        for generated in [
            GeneratedTable {
                requested_count: 1,
                matrices: Some(&evaluated),
            },
            GeneratedTable {
                requested_count: 0,
                matrices: None,
            },
        ] {
            let result = gather_runtime_palette(true, &groups, generated).unwrap();
            assert_eq!(result.matrices[0].map(f32::to_bits), raw);
        }
    }

    #[test]
    fn invalid_signed_groups_and_short_tables_are_typed() {
        let group = RuntimePaletteGroup {
            first_bone: 0,
            bone_count: 2,
            matrices: GroupMatrices::Identity,
        };
        let absent = GeneratedTable {
            requested_count: 0,
            matrices: None,
        };
        for raw in 128..=255u8 {
            assert!(matches!(
                gather_runtime_palette(
                    true,
                    &[RuntimePaletteGroup {
                        bone_count: raw,
                        ..group
                    }],
                    absent
                ),
                Err(PaletteError::NegativeGroupCount { .. })
            ));
        }
        assert!(matches!(
            gather_runtime_palette(
                true,
                &[RuntimePaletteGroup {
                    first_bone: 0x8000,
                    ..group
                }],
                absent
            ),
            Err(PaletteError::NegativeGroupOffset { .. })
        ));
        assert!(matches!(
            gather_runtime_palette(
                true,
                &[RuntimePaletteGroup {
                    first_bone: 1,
                    ..group
                }],
                absent
            ),
            Err(PaletteError::NonContiguousGroup { .. })
        ));
        assert!(matches!(
            gather_runtime_palette(
                true,
                &[group],
                GeneratedTable {
                    requested_count: 2,
                    matrices: Some(&[])
                }
            ),
            Err(PaletteError::ShortGeneratedTable {
                required: 2,
                actual: 0
            })
        ));
        assert!(matches!(
            gather_runtime_palette(
                true,
                &[RuntimePaletteGroup {
                    matrices: GroupMatrices::Evaluated(&[]),
                    ..group
                }],
                absent
            ),
            Err(PaletteError::ShortEvaluatedGroup {
                required: 2,
                actual: 0,
                ..
            })
        ));
        assert!(matches!(
            gather_runtime_palette(
                true,
                &[],
                GeneratedTable {
                    requested_count: i32::MAX as usize + 1,
                    matrices: None
                }
            ),
            Err(PaletteError::RequestedCountOutOfRange(_))
        ));
    }
}
