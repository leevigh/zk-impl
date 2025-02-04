use crate::libs::transcript::Transcript;
use crate::libs::{multilinear_poly::MultilinearPoly, transcript};
use ark_bn254::Fq;
use ark_ff::{BigInteger, PrimeField};

#[derive(Debug)]
struct Proof<F: PrimeField> {
    claimed_sum: F,
    round_polys: Vec<[F; 2]>,
}

fn fq_vec_to_bytes(values: &[Fq]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|x| x.into_bigint().to_bytes_le())
        .collect()
}

fn prove<F: PrimeField>(poly: &MultilinearPoly<F>, claimed_sum: F) -> Proof<F> {
    let mut round_polys = vec![];

    let mut transcript = Transcript::new();
    // &[u8]
    // [&[u8], &[u8], ...]
    // [[1, 2], [3, 4]]
    // [1, 2, 3, 4]
    transcript.append(
        poly.evals
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    transcript.append(claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut poly = poly.clone();

    for _ in 0..poly.n_vars {
        let round_poly: [F; 2] = [
            poly.partial_evaluate(0, &F::zero()).evals.iter().sum(),
            poly.partial_evaluate(0, &F::one()).evals.iter().sum(),
        ];

        transcript.append(
            round_poly
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        round_polys.push(round_poly);

        let challenge = transcript.sample_field_element();

        poly = poly.partial_evaluate(0, &challenge);
    }

    Proof {
        claimed_sum,
        round_polys,
    }
}

fn verify<F: PrimeField>(poly: &MultilinearPoly<F>, proof: &Proof<F>) -> bool {
    if proof.round_polys.len() != poly.n_vars {
        return false;
    }

    let mut challenges = vec![];

    let mut transcript = Transcript::new();
    transcript.append(
        poly.evals
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    transcript.append(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut claimed_sum = proof.claimed_sum;

    for round_poly in &proof.round_polys {
        if claimed_sum != round_poly.iter().sum() {
            return false;
        }

        transcript.append(
            round_poly
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let challenge = transcript.sample_field_element();
        claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
        challenges.push(challenge);
    }

    if claimed_sum != poly.evaluate(&challenges) {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use crate::libs::multilinear_poly::tests::to_field;
    use crate::libs::multilinear_poly::MultilinearPoly;
    use crate::libs::sum_check_protocol::{prove, verify};
    use ark_bn254::Fr;

    #[test]
    fn test_sumcheck_protocol() {
        let poly = MultilinearPoly::new(3, to_field(vec![0, 0, 0, 3, 0, 0, 2, 5]));
        let proof = prove(&poly, Fr::from(20));

        dbg!(verify(&poly, &proof));
    }
}

// use super::fiat_shamir::Transcript;
// use super::multilinear_poly::MultilinearPoly;
// use ark_bn254::Fq;
// use ark_ff::{BigInteger, PrimeField};

// struct Proof {
//     initial_poly: MultilinearPoly<Fq>,
//     claimed_sum: Fq,
//     proof_polys: Vec<Vec<Fq>>,
//     sum_proofs: Vec<Fq>,
// }

// struct SumCheck {
//     polynomial: MultilinearPoly<Fq>,
//     transcript: Transcript<Fq>,
//     proof: Proof,
// }

// //// ! check and confirm the polynomial in sumcheck isnt being mutated anywhere

// impl SumCheck {
//     fn new(poly: MultilinearPoly<Fq>) -> Self {
//         let mut new_transcript = Transcript::new();

//         let poly_bytes = fq_vec_to_bytes(&poly.evaluation);

//         new_transcript.append(&poly_bytes);

//         let new_proof = Proof {
//             initial_poly: poly.clone(),
//             claimed_sum: Fq::from(0),
//             proof_polys: Vec::new(),
//             sum_proofs: Vec::new(),
//         };

//         Self {
//             polynomial: poly,
//             transcript: new_transcript,
//             proof: new_proof,
//         }
//     }

//     fn get_sum_proof(&mut self) -> Fq {
//         let sum_proof: Fq = self.polynomial.evaluation.iter().sum();

//         let sum_bytes = fq_vec_to_bytes(&vec![sum_proof]);

//         self.transcript.append(&sum_bytes);

//         sum_proof
//     }

//     fn get_partial_polynomial_proof(&self) -> MultilinearPoly<Fq> {
//         let mid_point = self.polynomial.evaluation.len() / 2;

//         let (zeros, ones) = self.polynomial.evaluation.split_at(mid_point);

//         MultilinearPoly::new(vec![zeros.iter().sum(), ones.iter().sum()])
//     }

//     fn get_proof(&mut self) -> Proof {
//         let sum_proof = self.get_sum_proof();
//         let proof_poly = self.get_partial_polynomial_proof();

//         let mut new_sum_proofs = self.proof.sum_proofs.clone();
//         new_sum_proofs.push(sum_proof);

//         let mut new_proof_polys = self.proof.proof_polys.clone();
//         new_proof_polys.push(proof_poly.evaluation);

//         Proof {
//             initial_poly: self.polynomial.clone(),
//             claimed_sum: sum_proof,
//             proof_polys: new_proof_polys,
//             sum_proofs: new_sum_proofs,
//         }
//     }

//     fn verify_sum_proof(poly_proof: Self, sum_proof: Fq) -> bool {
//         //rework this this to include oracle check

//         let eval_0 = poly_proof.polynomial.evaluate(vec![Fq::from(0)]);
//         let eval_1 = poly_proof.polynomial.evaluate(vec![Fq::from(1)]);

//         let poly_sum = eval_0 + eval_1;

//         sum_proof == poly_sum
//     }

//     fn initiate_next_round(&self) -> MultilinearPoly<Fq> {
//         let random_challenge = self.transcript.get_random_challenge();

//         let new_poly = self.polynomial.partial_evaluate(random_challenge, 0);

//         new_poly
//     }
// }

// fn fq_vec_to_bytes(value: &Vec<Fq>) -> Vec<u8> {
//     let value_bytes: Vec<u8> = value
//         .iter()
//         .flat_map(|x| x.into_bigint().to_bytes_le())
//         .collect();

//     value_bytes
// }
