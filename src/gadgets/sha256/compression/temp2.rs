use crate::{
    error::Error,
    gadgets::add_mod::{add_mod_verify, adder, mpc_add_mod},
    gf2_word::GF2Word,
    party::Party,
};

/// temp2 := S0 + maj
pub fn temp2(s0: u32, maj: u32) -> u32 {
    adder(s0, maj)
}

/// temp2 := S0 + maj
pub fn mpc_temp2(
    (s0_1, maj_1): (GF2Word<u32>, GF2Word<u32>),
    (s0_2, maj_2): (GF2Word<u32>, GF2Word<u32>),
    (s0_3, maj_3): (GF2Word<u32>, GF2Word<u32>),
    p1: &mut Party<u32>,
    p2: &mut Party<u32>,
    p3: &mut Party<u32>,
) -> (GF2Word<u32>, GF2Word<u32>, GF2Word<u32>) {
    mpc_add_mod((s0_1, maj_1), (s0_2, maj_2), (s0_3, maj_3), p1, p2, p3)
}

pub fn mpc_temp2_verify(
    (s0_p, maj_p): (GF2Word<u32>, GF2Word<u32>),
    (s0_p_next, maj_p_next): (GF2Word<u32>, GF2Word<u32>),
    p: &mut Party<u32>,
    p_next: &mut Party<u32>,
) -> Result<(GF2Word<u32>, GF2Word<u32>), Error> {
    // output = s0 + maj
    let (output_p, output_p_next) =
        add_mod_verify((s0_p, maj_p), (s0_p_next, maj_p_next), p, p_next);

    Ok((output_p, output_p_next))
}

#[cfg(test)]
mod test_temp2 {

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
    };

    use super::*;

    pub struct Temp2Circuit;

    impl Circuit<u32> for Temp2Circuit {
        fn compute(&self, input: &[GF2Word<u32>]) -> Vec<GF2Word<u32>> {
            assert_eq!(input.len(), 2);
            let res = temp2(input[0].value, input[1].value);
            vec![res.into()]
        }

        fn compute_23_decomposition(
            &self,
            p1: &mut Party<u32>,
            p2: &mut Party<u32>,
            p3: &mut Party<u32>,
        ) -> (Vec<GF2Word<u32>>, Vec<GF2Word<u32>>, Vec<GF2Word<u32>>) {
            assert_eq!(p1.view.input.len(), 2);
            assert_eq!(p2.view.input.len(), 2);
            assert_eq!(p3.view.input.len(), 2);

            let input_p1 = (p1.view.input[0], p1.view.input[1]);
            let input_p2 = (p2.view.input[0], p2.view.input[1]);
            let input_p3 = (p3.view.input[0], p3.view.input[1]);

            let (o1, o2, o3) = mpc_temp2(input_p1, input_p2, input_p3, p1, p2, p3);
            (vec![o1], vec![o2], vec![o3])
        }

        fn simulate_two_parties(
            &self,
            p: &mut Party<u32>,
            p_next: &mut Party<u32>,
        ) -> Result<(Output<u32>, Output<u32>), Error> {
            assert_eq!(p.view.input.len(), 2);
            assert_eq!(p_next.view.input.len(), 2);

            let input_p = (p.view.input[0], p.view.input[1]);
            let input_p_next = (p_next.view.input[0], p_next.view.input[1]);

            let (o1, o2) = mpc_temp2_verify(input_p, input_p_next, p, p_next)?;

            Ok((vec![o1], vec![o2]))
        }

        fn party_output_len(&self) -> usize {
            1
        }

        fn num_of_mul_gates(&self) -> usize {
            1
        }
    }

    #[test]
    fn test_circuit() {
        let mut rng = thread_rng();
        const SIGMA: usize = 80;
        let input: Vec<GF2Word<u32>> = [381321u32, 32131u32].iter().map(|&vi| vi.into()).collect();

        let circuit = Temp2Circuit;

        let output = circuit.compute(&input);

        let proof = Prover::<u32, ChaCha20Rng, Keccak256>::prove::<ThreadRng, SIGMA>(
            &mut rng, &input, &circuit, &output,
        )
        .unwrap();

        Verifier::<u32, ChaCha20Rng, Keccak256>::verify(&proof, &circuit, &output).unwrap();
    }
}
