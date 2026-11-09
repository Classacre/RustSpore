//! Compare only the retained scripted allocation-success and failure rows.
use spore_core::space_ui_handoff_v1::{clear_events, ClearEvent};

fn main() {
    let path = std::env::args_os().nth(1).expect("native stdout path");
    let bytes = std::fs::read(path).expect("retained native stdout");
    assert_eq!(bytes.len(), 96);
    for (index, row) in bytes.chunks_exact(48).enumerate() {
        let words: Vec<u32> = row.chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect();
        let modeled = clear_events(index == 0);
        assert_eq!(words[0] as usize, modeled.len());
        for (i, event) in modeled.iter().enumerate() {
            let code = match event {
                ClearEvent::Allocate => 1, ClearEvent::Locale => 2,
                ClearEvent::Construct => 3, ClearEvent::AddRef => 4,
                ClearEvent::SetChild => 5,
            };
            assert_eq!(words[i + 1], code);
        }
        assert!(words[1 + modeled.len()..7].iter().all(|&word| word == 0));
        assert_eq!(words[7] & 0xffff, 0x027f);
        assert_eq!((words[7] >> 16) & 0x3840, 0);
        assert_eq!(words[8] & !0x3f, 0x1f80);
        assert_eq!(&words[9..], &[0, 0, 0]);
    }
    println!("matched two scripted clear-flag handoff rows");
}
