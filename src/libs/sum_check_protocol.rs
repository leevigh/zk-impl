use super::multilinear_poly::{MultilinearPoly, SumPoly};
// use crate::libs::transcript::Transcript;
use super::fiat_shamir::Transcript;
// use crate::libs::{multilinear_poly::MultilinearPoly, transcript};
use super::polynomial::UnivariatePolynomial;
use ark_bn254::Fq;
use ark_ff::{BigInteger, PrimeField};

#[derive(Debug)]
struct Proof<F: PrimeField> {
    claimed_sum: F,
    round_polys: Vec<[F; 2]>,
}

pub struct GkrProof<F: PrimeField> {
    pub proof_polynomials: Vec<Vec<F>>,
    pub claimed_sum: F,
    pub random_challenges: Vec<F>,
}

pub struct GkrVerify<F: PrimeField> {
    pub verified: bool,
    pub final_claimed_sum: F,
    pub random_challenges: Vec<F>,
}

fn fq_vec_to_bytes<F: PrimeField>(values: &[F]) -> Vec<u8> {
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

        let challenge = transcript.get_random_challenge();

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

        let challenge = transcript.get_random_challenge();
        claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
        challenges.push(challenge);
    }

    if claimed_sum != poly.evaluate(challenges) {
        return false;
    }

    true
}

pub fn gkr_prove<F: PrimeField>(
    claimed_sum: F,
    composed_polynomial: &SumPoly<F>,
    transcript: &mut Transcript<F>,
) -> GkrProof<F> {
    let num_rounds = composed_polynomial.polys[0].evaluation[0].n_vars;
    let mut proof_polynomials = Vec::with_capacity(num_rounds);
    let mut random_challenges = Vec::with_capacity(num_rounds);
    let mut current_poly = composed_polynomial.clone();

    for _ in 0..num_rounds {
        let proof_poly = get_round_partial_polynomial_proof_gkr(&current_poly); //this is f(b) then f(c)

        transcript.append(&fq_vec_to_bytes(&proof_poly));

        proof_polynomials.push(proof_poly);

        let random_challenge = transcript.get_random_challenge(); //this is b and c aka r1 r2

        random_challenges.push(random_challenge);

        current_poly = current_poly.partial_evaluate(&random_challenge);
    }

    GkrProof {
        proof_polynomials,
        claimed_sum,
        random_challenges,
    }
}

pub fn gkr_verify<F: PrimeField>(
    round_polys: Vec<Vec<F>>,
    mut claimed_sum: F,
    transcript: &mut Transcript<F>,
) -> GkrVerify<F> {
    let mut random_challenges = Vec::new();

    for round_poly in round_polys {
        let f_b_0 = round_poly[0];
        let f_b_1 = round_poly[1];

        if f_b_0 + f_b_1 != claimed_sum {
            return GkrVerify {
                verified: false,
                final_claimed_sum: F::zero(),
                random_challenges: vec![F::zero()],
            };
        }

        transcript.append(&fq_vec_to_bytes(&round_poly));

        let r_c = transcript.get_random_challenge();

        random_challenges.push(r_c);

        let round_uni_points = round_poly
            .iter()
            .enumerate()
            .map(|(i, y)| {
                let x = F::from(i as u64);

                (x, *y)
            })
            .collect();

        let round_uni_poly = UnivariatePolynomial::interpolate(round_uni_points);

        claimed_sum = round_uni_poly.evaluate(r_c); //next expected sum
    }

    GkrVerify {
        verified: true,
        final_claimed_sum: claimed_sum,
        random_challenges,
    }
}

fn get_round_partial_polynomial_proof_gkr<F: PrimeField>(composed_poly: &SumPoly<F>) -> Vec<F> {
    let degree = composed_poly.get_degree();

    (0..=degree)
        .map(|i| {
            let partial_poly = composed_poly.partial_evaluate(&F::from(i as u64));

            partial_poly.reduce().iter().sum()
        })
        .collect()
}

fn get_round_partial_polynomial_proof<F: PrimeField>(polynomial: &[F]) -> Vec<F> {
    let mid_point = polynomial.len() / 2;
    let (zeros, ones) = polynomial.split_at(mid_point);

    let poly_proof = vec![zeros.iter().sum(), ones.iter().sum()];

    poly_proof
}

#[cfg(test)]
mod test {
    use crate::libs::multilinear_poly::tests::to_field;
    use crate::libs::multilinear_poly::MultilinearPoly;
    use crate::libs::sum_check_protocol::{prove, verify};
    use ark_bn254::Fr;

    #[test]
    fn test_sumcheck_protocol() {
        let poly = MultilinearPoly::new(to_field(vec![0, 0, 0, 3, 0, 0, 2, 5]));
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
