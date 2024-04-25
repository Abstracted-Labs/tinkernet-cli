use crate::commands::consts::youdle_consts::ONE_WITH_DECIMALS;

pub fn planck_to_unit(planck: u128) -> f64 {
    planck as f64 / ONE_WITH_DECIMALS as f64
}
