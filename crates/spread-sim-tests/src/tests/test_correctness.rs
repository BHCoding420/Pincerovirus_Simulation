use crate::scenarios;

#[test]
fn test_we_love_np_7() {
    scenarios::WE_LOVE_NP.test_case().with_padding(7).launch()
}

#[test]
fn test_we_love_np_10() {
    scenarios::WE_LOVE_NP.test_case().with_padding(10).launch()
}

#[test]
fn test_we_love_np_15() {
    scenarios::WE_LOVE_NP.test_case().with_padding(15).launch()
}

#[test]
fn test_single() {
    scenarios::SINGLE_SPLIT
        .test_case()
        .with_padding(10)
        .launch();
}

#[test]
fn test_very_small() {
    scenarios::VERY_SMALL.test_case().with_padding(10).launch();
}

#[test]
fn test_small1_pad7() {
    scenarios::SMALL_1.test_case().with_padding(7).launch();
}

#[test]
fn test_small1_pad8() {
    scenarios::SMALL_1.test_case().with_padding(8).launch();
}
#[test]
fn test_small1_pad9() {
    scenarios::SMALL_1.test_case().with_padding(9).launch();
}
#[test]
fn test_small1_pad10() {
    scenarios::SMALL_1.test_case().with_padding(10).launch();
}
#[test]
fn test_small1_pad11() {
    scenarios::SMALL_1.test_case().with_padding(11).launch();
}
#[test]
fn test_small1_pad12() {
    scenarios::SMALL_1.test_case().with_padding(12).launch();
}

#[test]
fn test_small2_pad7() {
    scenarios::SMALL_2.test_case().with_padding(7).launch();
}
#[test]
fn test_small2_pad8() {
    scenarios::SMALL_2.test_case().with_padding(8).launch();
}
#[test]
fn test_small2_pad9() {
    scenarios::SMALL_2.test_case().with_padding(9).launch();
}
#[test]
fn test_small2_pad10() {
    scenarios::SMALL_2.test_case().with_padding(10).launch();
}
#[test]
fn test_small2_pad11() {
    scenarios::SMALL_2.test_case().with_padding(11).launch();
}
#[test]
fn test_small2_pad12() {
    scenarios::SMALL_2.test_case().with_padding(12).launch();
}

#[test]
fn test_small3_pad7() {
    scenarios::SMALL_2.test_case().with_padding(7).launch();
}
#[test]
fn test_small3_pad8() {
    scenarios::SMALL_2.test_case().with_padding(8).launch();
}
#[test]
fn test_small3_pad9() {
    scenarios::SMALL_2.test_case().with_padding(9).launch();
}
#[test]
fn test_small3_pad10() {
    scenarios::SMALL_2.test_case().with_padding(10).launch();
}
#[test]
fn test_small3_pad11() {
    scenarios::SMALL_2.test_case().with_padding(11).launch();
}
#[test]
fn test_small3_pad12() {
    scenarios::SMALL_2.test_case().with_padding(12).launch();
}

#[test]
fn test_small4_pad7() {
    scenarios::SMALL_2.test_case().with_padding(7).launch();
}
#[test]
fn test_small4_pad8() {
    scenarios::SMALL_2.test_case().with_padding(8).launch();
}
#[test]
fn test_small4_pad9() {
    scenarios::SMALL_2.test_case().with_padding(9).launch();
}
#[test]
fn test_small4_pad10() {
    scenarios::SMALL_2.test_case().with_padding(10).launch();
}
#[test]
fn test_small4_pad11() {
    scenarios::SMALL_2.test_case().with_padding(11).launch();
}
#[test]
fn test_small4_pad12() {
    scenarios::SMALL_2.test_case().with_padding(12).launch();
}

#[test]
fn test_small5_pad7() {
    scenarios::SMALL_2.test_case().with_padding(7).launch();
}
#[test]
fn test_small5_pad8() {
    scenarios::SMALL_2.test_case().with_padding(8).launch();
}
#[test]
fn test_small5_pad9() {
    scenarios::SMALL_2.test_case().with_padding(9).launch();
}
#[test]
fn test_small5_pad10() {
    scenarios::SMALL_2.test_case().with_padding(10).launch();
}
#[test]
fn test_small5_pad11() {
    scenarios::SMALL_2.test_case().with_padding(11).launch();
}
#[test]
fn test_small5_pad12() {
    scenarios::SMALL_2.test_case().with_padding(12).launch();
}
