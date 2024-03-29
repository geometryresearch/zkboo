// (a and b) xor (a and c) xor (b and c)
// = (a xor b) and (a xor c) xor a

use crate::{
    error::Error,
    gadgets::{mpc_and, mpc_and_verify},
    gf2_word::GF2Word,
    party::Party,
};

pub(crate) fn maj(a: u32, b: u32, c: u32) -> u32 {
    // (a and b) xor (a and c) xor (b and c)
    (a & b) ^ (a & c) ^ (b & c)
}

pub fn mpc_maj(
    // a, b, c
    input_p1: (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>),
    input_p2: (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>),
    input_p3: (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>),
    p1: &mut Party<u32>,
    p2: &mut Party<u32>,
    p3: &mut Party<u32>,
) -> (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>) {
    // (a xor b)
    let a_xor_b_1 = input_p1.0 ^ input_p1.1;
    let a_xor_b_2 = input_p2.0 ^ input_p2.1;
    let a_xor_b_3 = input_p3.0 ^ input_p3.1;

    // (a xor c)
    let a_xor_c_1 = input_p1.0 ^ input_p1.2;
    let a_xor_c_2 = input_p2.0 ^ input_p2.2;
    let a_xor_c_3 = input_p3.0 ^ input_p3.2;

    // lhs = (a xor b) and (a xor c)
    let (lhs_1, lhs_2, lhs_3) = mpc_and(
        (a_xor_b_1, a_xor_c_1),
        (a_xor_b_2, a_xor_c_2),
        (a_xor_b_3, a_xor_c_3),
        p1,
        p2,
        p3,
    );

    // lhs xor a
    let output_p1 = lhs_1 ^ input_p1.0;
    let output_p2 = lhs_2 ^ input_p2.0;
    let output_p3 = lhs_3 ^ input_p3.0;

    (output_p1, output_p2, output_p3)
}

pub fn maj_verify(
    input_p: (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>),
    input_p_next: (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>),
    p: &mut Party<u32>,
    p_next: &mut Party<u32>,
) -> Result<(GF2Word<u32>, GF2Word<u32>), Error> {
    // (a xor b)
    let a_xor_b_p = input_p.0 ^ input_p.1;
    let a_xor_b_p_next = input_p_next.0 ^ input_p_next.1;

    // (a xor c)
    let a_xor_c_p = input_p.0 ^ input_p.2;
    let a_xor_c_p_next = input_p_next.0 ^ input_p_next.2;

    // lhs = (a xor b) and (a xor c)
    let (lhs_p, lhs_p_next) = mpc_and_verify(
        (a_xor_b_p, a_xor_c_p),
        (a_xor_b_p_next, a_xor_c_p_next),
        p,
        p_next,
    )?;

    // lhs xor a
    let output_p = lhs_p ^ input_p.0;
    let output_p_next = lhs_p_next ^ input_p_next.0;

    Ok((output_p, output_p_next))
}

#[cfg(test)]
mod test_maj {

    use rand::{rngs::ThreadRng, thread_rng};
    use rand_chacha::ChaCha20Rng;
    use sha3::Keccak256;

    use crate::{
        circuit::{Circuit, Output},
        error::Error,
        gadgets::prepare::generic_parse,
        gf2_word::GF2Word,
        party::Party,
        prover::Prover,
        verifier::Verifier,
    };

    use super::*;

    pub struct MajCircuit;

    impl Circuit<u32> for MajCircuit {
        fn compute(&self, input: &[u8]) -> Vec<GF2Word<u32>> {
            let words = generic_parse(input, self.party_input_len());
            let res = maj(words[0].value, words[1].value, words[2].value);
            vec![res.into()]
        }

        fn compute_23_decomposition(
            &self,
            p1: &mut Party<u32>,
            p2: &mut Party<u32>,
            p3: &mut Party<u32>,
        ) -> (Vec<GF2Word<u32>>, Vec<GF2Word<u32>>, Vec<GF2Word<u32>>) {
            let p1_words = generic_parse(&p1.view.input, self.party_input_len());
            let p2_words = generic_parse(&p2.view.input, self.party_input_len());
            let p3_words = generic_parse(&p3.view.input, self.party_input_len());

            let input_p1 = (p1_words[0], p1_words[1], p1_words[2]);
            let input_p2 = (p2_words[0], p2_words[1], p2_words[2]);
            let input_p3 = (p3_words[0], p3_words[1], p3_words[2]);

            let (o1, o2, o3) = mpc_maj(input_p1, input_p2, input_p3, p1, p2, p3);
            (vec![o1], vec![o2], vec![o3])
        }

        fn simulate_two_parties(
            &self,
            p: &mut Party<u32>,
            p_next: &mut Party<u32>,
        ) -> Result<(Output<u32>, Output<u32>), Error> {
            let p_words = generic_parse(&p.view.input, self.party_input_len());
            let p_next_words = generic_parse(&p_next.view.input, self.party_input_len());

            let input_p = (p_words[0], p_words[1], p_words[2]);
            let input_p_next = (p_next_words[0], p_next_words[1], p_next_words[2]);

            let (o1, o2) = maj_verify(input_p, input_p_next, p, p_next)?;

            Ok((vec![o1], vec![o2]))
        }

        fn party_output_len(&self) -> usize {
            1
        }

        fn num_of_mul_gates(&self) -> usize {
            1
        }

        fn party_input_len(&self) -> usize {
            3
        }
    }

    #[test]
    fn test_circuit() {
        let mut rng = thread_rng();
        const SIGMA: usize = 80;

        let input: Vec<u8> = [
            381321u32.to_le_bytes(),
            32131u32.to_le_bytes(),
            328131u32.to_le_bytes(),
        ]
        .into_iter()
        .flatten()
        .collect();

        let circuit = MajCircuit;

        let output = circuit.compute(&input);

        let proof = Prover::<u32, ChaCha20Rng, Keccak256>::prove::<ThreadRng, SIGMA>(
            &mut rng, &input, &circuit, &output,
        )
        .unwrap();

        Verifier::<u32, ChaCha20Rng, Keccak256>::verify(&proof, &circuit, &output).unwrap();
    }
}
