pub mod circuit;
pub mod commitment;
pub mod config;
pub mod data_structures;
pub mod error;
pub mod fs;
pub mod gf2_word;
pub mod key;
pub mod party;
pub mod prover;
pub mod tape;
pub mod verifier;
pub mod view;

pub mod gadgets;

mod composition;

pub fn num_of_repetitions_given_desired_security(sigma: usize) -> usize {
    // log_2(3) - 1
    let log_2_3_minus_1: f64 = 0.58496;
    let sigma = sigma as f64;

    (sigma / log_2_3_minus_1).ceil() as usize
}

#[cfg(test)]
mod test_rep {
    use crate::num_of_repetitions_given_desired_security;

    #[test]
    fn test_constants_from_zkboo_paper() {
        let sigma_1 = 40;
        let sigma_2 = 80;
        let n_1 = num_of_repetitions_given_desired_security(sigma_1);
        assert_eq!(n_1, 69);

        let n_2 = num_of_repetitions_given_desired_security(sigma_2);
        assert_eq!(n_2, 137);
    }
}
