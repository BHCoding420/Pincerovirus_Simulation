use crate::scenarios;

mod test_correctness;
mod test_slug;

/// Makes sure that the macros for defining test scenarios work as expected.
#[test]
pub fn test_macros() {
    scenarios::WE_LOVE_NP.load_scenario();
    scenarios::WE_LOVE_NP.load_output();
}
