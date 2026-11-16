//! The first-run pin: the terminal app lists the cards whose verify lines the pins do not hold and
//! asks once; `y` pins exactly those lines, anything else pins nothing.

use std::fs;

use mikata_loop::world::pins;
use mikata_tui::pin::{Offered, offer};

#[test]
fn the_first_run_asks_once_and_pins_what_it_showed() {
    let w = std::env::temp_dir().join(format!("mikata-tui-pin-{}", std::process::id()));
    let _ = fs::remove_dir_all(&w);
    fs::create_dir_all(w.join(".mikata/summoner/summoner")).unwrap();
    fs::write(
        w.join(".mikata/summoner/summoner/README.md"),
        "# summoner\n\n- **Race:** summoner\n- **Verify:** `pytest -q`\n",
    )
    .unwrap();
    let w = w.canonicalize().unwrap();
    let mut out = Vec::new();
    let mut asked = 0;
    let got = offer(&w, &mut out, &mut |_| {
        asked += 1;
        Ok(false)
    })
    .unwrap();
    assert_eq!(got, Offered::Declined(vec!["summoner".into()]));
    let text = String::from_utf8_lossy(&out).into_owned();
    assert!(
        text.contains("summoner (.mikata/summoner/summoner/README.md) — not pinned")
            && text.contains("`pytest -q`")
            && text.contains("mikatad verify pin"),
        "{text}"
    );
    // No terminal is no answer.
    let got = offer(&w, &mut Vec::new(), &mut |_| Err("no terminal".into())).unwrap();
    assert_eq!(got, Offered::Declined(vec!["summoner".into()]));
    let got = offer(&w, &mut Vec::new(), &mut |_| {
        asked += 1;
        Ok(true)
    })
    .unwrap();
    assert_eq!(got, Offered::Pinned(vec!["summoner".into()]));
    assert_eq!(asked, 2, "one question each time");
    let held = &pins::pinned(&w).unwrap().unwrap()["summoner"];
    assert_eq!(
        (held[0].line.as_str(), held[0].by.as_str()),
        ("pytest -q", "human")
    );
    let got = offer(&w, &mut Vec::new(), &mut |_| panic!("asked again")).unwrap();
    assert_eq!(got, Offered::Nothing);
    let _ = fs::remove_file(pins::path_of(&w).unwrap());
    let _ = fs::remove_dir_all(&w);
}
