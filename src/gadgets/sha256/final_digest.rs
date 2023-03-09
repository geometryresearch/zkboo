use crate::{
    gadgets::add_mod::{add_mod_verify_k, adder, mpc_add_mod_k},
    gf2_word::GF2Word,
    party::Party,
};

use super::iv::init_iv;

pub fn digest(compression_output: &[GF2Word<u32>; 8]) -> Vec<GF2Word<u32>> {
    let hs = init_iv().to_vec();
    hs.into_iter()
        .zip(compression_output.iter())
        .map(|(hs, &output)| adder(hs.value, output.value).into())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub fn mpc_digest(
    compression_output_p1: &[GF2Word<u32>; 8],
    compression_output_p2: &[GF2Word<u32>; 8],
    compression_output_p3: &[GF2Word<u32>; 8],
    p1: &mut Party<u32>,
    p2: &mut Party<u32>,
    p3: &mut Party<u32>,
) -> (Vec<GF2Word<u32>>, Vec<GF2Word<u32>>, Vec<GF2Word<u32>>) {
    let hs = init_iv().to_vec();

    let mut output_1 = Vec::with_capacity(8);
    let mut output_2 = Vec::with_capacity(8);
    let mut output_3 = Vec::with_capacity(8);

    for i in 0..8 {
        let (o1, o2, o3) = mpc_add_mod_k(
            compression_output_p1[i],
            compression_output_p2[i],
            compression_output_p3[i],
            hs[i],
            p1,
            p2,
            p3,
        );

        output_1.push(o1);
        output_2.push(o2);
        output_3.push(o3);
    }

    (output_1, output_2, output_3)
}

pub fn mpc_digest_verify(
    compression_output_p: &[GF2Word<u32>; 8],
    compression_output_p_next: &[GF2Word<u32>; 8],
    p: &mut Party<u32>,
    p_next: &mut Party<u32>,
) -> (Vec<GF2Word<u32>>, Vec<GF2Word<u32>>) {
    let hs = init_iv().to_vec();

    let mut output_p = Vec::with_capacity(8);
    let mut output_p_next = Vec::with_capacity(8);

    for i in 0..8 {
        let (o1, o2) = add_mod_verify_k(
            compression_output_p[i],
            compression_output_p_next[i],
            hs[i],
            p,
            p_next,
        );

        output_p.push(o1);
        output_p_next.push(o2);
    }

    (output_p, output_p_next)
}

#[cfg(test)]
mod test_digest {

    use rand::{rngs::ThreadRng, thread_rng};
    use rand_chacha::ChaCha20Rng;
    use sha3::Keccak256;

    use crate::{
        circuit::{Circuit, Output},
        error::Error,
        gf2_word::GF2Word,
        party::Party,
        prover::Prover,
        verifier::Verifier,
        gadgets::prepare::generic_parse
    };

    use super::*;

    pub struct DigestCircuit;

    impl Circuit<u32> for DigestCircuit {
        fn compute(&self, input: &[u8]) -> Vec<GF2Word<u32>> {
            let input = generic_parse(input, self.party_input_len());
            digest(&input.try_into().unwrap())
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

            mpc_digest(
                &p1_words.try_into().unwrap(),
                &p2_words.try_into().unwrap(),
                &p3_words.try_into().unwrap(),
                p1,
                p2,
                p3,
            )
        }

        fn simulate_two_parties(
            &self,
            p: &mut Party<u32>,
            p_next: &mut Party<u32>,
        ) -> Result<(Output<u32>, Output<u32>), Error> {
            let p_words = generic_parse(&p.view.input, self.party_input_len());
            let p_next_words = generic_parse(&p_next.view.input, self.party_input_len());

            let (o1, o2) = mpc_digest_verify(
                &p_words.try_into().unwrap(),
                &p_next_words.try_into().unwrap(),
                p,
                p_next,
            );

            Ok((o1.to_vec(), o2.to_vec()))
        }

        fn party_input_len(&self) -> usize {
            8
        }

        fn party_output_len(&self) -> usize {
            8
        }

        fn num_of_mul_gates(&self) -> usize {
            8
        }
    }

    #[test]
    fn test_circuit() {
        let mut rng = thread_rng();
        const SIGMA: usize = 80;
        let input: Vec<u8> = crate::gadgets::sha256::test_vectors::short::COMPRESSION_OUTPUT
        .iter().map(|v| v.to_le_bytes()).flatten().collect();

        let circuit = DigestCircuit;

        let output = circuit.compute(&input);
        let expected_output = crate::gadgets::sha256::test_vectors::short::DIGEST_OUTPUT;
        for (&word, &expected_word) in output.iter().zip(expected_output.iter()) {
            assert_eq!(word.value, expected_word);
        }

        let proof = Prover::<u32, ChaCha20Rng, Keccak256>::prove::<ThreadRng, SIGMA>(
            &mut rng, &input, &circuit, &output,
        )
        .unwrap();

        Verifier::<u32, ChaCha20Rng, Keccak256>::verify(&proof, &circuit, &output).unwrap();
    }
}
