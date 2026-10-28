//! Bounded field-validity composition for the 009C52F4 initializer.
//! No execution of 009F4730 or 009E7C80 is implemented here.
use crate::anim_cid_node_init_v1::{self as init, Admission, DefinitionNode, Error, NODE_BYTES};

/// A caller must establish this path for the node instances it supplies.
/// The grow path constructs a temporary with 009BF4A0 and copies it via
/// 009BE870/009B43C0; the bounded loop does not overwrite +1AC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncomingState { Unestablished, SourceGrowTemporaryCopy }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeState { bytes: [u8; NODE_BYTES], known: [bool; NODE_BYTES] }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeRead { SkippedByNodeFlags, Known(u32), UnestablishedFlags, UnestablishedMode }

impl NodeState {
    fn from_loop_row(bytes: [u8; NODE_BYTES], incoming: IncomingState) -> Self {
        let mut state = Self { bytes, known: [false; NODE_BYTES] };
        // Derived from the source loop writes, including canonicalized links.
        for (start, end) in [(0,0x2c), (0x38,0x39), (0x3c,0x56),
                             (0x58,0xa4), (0x144,0x164), (0x2a0,0x2a1)] {
            state.known[start..end].fill(true);
        }
        if incoming == IncomingState::SourceGrowTemporaryCopy {
            // Source construction and copy establish these four bytes; the
            // loop model holds poison here, so replace poison with source values.
            state.bytes[0x1ac..0x1b0].copy_from_slice(&[4,0,0xff,0xff]);
            state.known[0x1ac..0x1b0].fill(true);
        }
        state
    }
    /// Reads only bytes established by the admitted source stages. The first
    /// 16 bytes encode canonical ordinals, not raw native pointer addresses.
    pub fn read_known(&self, start: usize, len: usize) -> Option<&[u8]> {
        let end = start.checked_add(len)?;
        if end > NODE_BYTES || !self.known[start..end].iter().all(|x| *x) { return None; }
        Some(&self.bytes[start..end])
    }
    /// Source 009E7C80 first checks +1AC low bits and only then reads +238.
    /// Unknown mode cannot silently become the loop model's poison value.
    pub fn mode_for_9e7c80(&self) -> ModeRead {
        let flags = match self.read_known(0x1ac, 1) { Some(v) => v[0], None => return ModeRead::UnestablishedFlags };
        if flags & 7 == 4 { return ModeRead::SkippedByNodeFlags; }
        match self.read_known(0x238, 4) {
            Some(v) => ModeRead::Known(u32::from_le_bytes([v[0],v[1],v[2],v[3]])),
            None => ModeRead::UnestablishedMode,
        }
    }
}

/// Compose the already bounded initializer with explicit incoming-path evidence.
/// IncomingState is a caller admission, not proof that a real allocation took
/// that path. Unknown fields remain inaccessible until a producer is modeled.
pub fn initialize(nodes: &[DefinitionNode], admission: Admission, incoming: IncomingState) -> Result<Vec<NodeState>, Error> {
    Ok(init::initialize(nodes, admission)?.into_iter().map(|row| NodeState::from_loop_row(row, incoming)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn poison_is_not_a_known_source_value() {
        let n = NodeState::from_loop_row([0xa5; NODE_BYTES], IncomingState::Unestablished);
        assert_eq!(n.read_known(0x238,4),None);
        assert_eq!(n.read_known(0x1ac,1),None);
        assert_eq!(n.mode_for_9e7c80(),ModeRead::UnestablishedFlags);
        assert_eq!(n.read_known(NODE_BYTES,1),None);
        assert_eq!(n.read_known(usize::MAX,2),None);
    }
    #[test]
    fn grow_constructor_flags_skip_mode_without_creating_a_default() {
        let n = NodeState::from_loop_row([0xa5; NODE_BYTES], IncomingState::SourceGrowTemporaryCopy);
        assert_eq!(n.read_known(0x1ac,4),Some(&[4,0,0xff,0xff][..]));
        assert_eq!(n.read_known(0x238,4),None);
        assert_eq!(n.mode_for_9e7c80(),ModeRead::SkippedByNodeFlags);
    }
    #[test]
    fn non_skipped_unknown_mode_is_rejected_and_known_mode_can_be_read() {
        let mut n = NodeState::from_loop_row([0xa5; NODE_BYTES], IncomingState::Unestablished);
        n.bytes[0x1ac]=0; n.known[0x1ac]=true;
        assert_eq!(n.mode_for_9e7c80(),ModeRead::UnestablishedMode);
        n.bytes[0x238..0x23c].copy_from_slice(&2u32.to_le_bytes());
        n.known[0x238..0x23c].fill(true);
        assert_eq!(n.mode_for_9e7c80(),ModeRead::Known(2));
    }
}
