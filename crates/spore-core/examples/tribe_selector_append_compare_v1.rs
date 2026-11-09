//! Compare three retained synthetic native rows; no live provider claim.
use spore_core::tribe_selector_append_v1::project;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("pass retained native stdout path")?;
    let data = std::fs::read(path)?;
    if data.len() != 120 { return Err("expected exactly three 40-byte rows".into()); }
    let input = [1.0f32.to_bits(), 2.0f32.to_bits(), 3.0f32.to_bits()];
    for i in 0..3 {
        let mut row = [0u32; 10];
        for (j, word) in row.iter_mut().enumerate() {
            let at = i * 40 + j * 4;
            *word = u32::from_le_bytes(data[at..at + 4].try_into()?);
        }
        let expected = project(i as u32 + 1, input)?;
        let expected_position = expected.appended_position_bits.unwrap_or([0; 3]);
        if row[0] != u32::from(expected.provider_key.is_some())
            || row[1] != expected.provider_key.unwrap_or(0)
            || row[2] != expected.appended_selector.unwrap_or(0)
            || row[3..6] != expected_position
            || row[6..8] != if i < 2 { [4,12] } else { [0,0] }
            || row[8] & 0xffff != 0x027f || (row[8] >> 16) & 0x3840 != 0
            || row[9] & !0x3f != 0x1f80
        { return Err(format!("row {i} differs").into()); }
    }
    println!("matched three controlled null-provider rows; no live provider claim");
    Ok(())
}
