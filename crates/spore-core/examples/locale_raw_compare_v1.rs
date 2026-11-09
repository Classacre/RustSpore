//! Compare the bounded host's six raw-return rows with the Rust model.
use spore_core::content_manifest::{locale_output_decision, LocaleOutputDecision, LocaleOutputState};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("usage: locale_raw_compare_v1 <host-output>")?;
    let output = std::fs::read_to_string(path)?;
    let cases = [(false,false,false,false),(false,true,false,false),(true,false,false,false),
        (true,false,false,true),(true,false,true,false),(true,false,true,true)];
    let mut expected = String::new();
    for (cached,present,classifier,global) in cases {
        let state = LocaleOutputState { cached_flag: cached, cached_pointer_present: present,
            classifier_nonzero: classifier, global_nonzero: global };
        assert_eq!(locale_output_decision(state), LocaleOutputDecision::Raw);
        expected.push_str(&format!("{} {} {} {} raw\n", cached as u8,present as u8,classifier as u8,global as u8));
    }
    if output.replace("\r\n", "\n") != expected { return Err("host output mismatch".into()); }
    println!("6/6 explicit raw-return cases match; classifier and processing not verified");
    Ok(())
}
