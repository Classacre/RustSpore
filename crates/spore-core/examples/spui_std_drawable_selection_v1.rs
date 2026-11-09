//! Exhaustive selector driver for independent reference comparison.
use spore_core::spui_std_drawable::select_slot;
fn main() {
    for state in (0u32..16).chain((0u32..16).map(|x| x | 0xffff_fff0)) {
        for mask in 0u16..256 {
            let occupied = std::array::from_fn(|i| mask & (1 << i) != 0);
            match select_slot(state, occupied) {
                Some(slot) => println!("{state:08X}\t{mask:02X}\t{slot}"),
                None => println!("{state:08X}\t{mask:02X}\t-"),
            }
        }
    }
}
