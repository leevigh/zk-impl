use ark_ff::PrimeField;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MultilinearPoly<F: PrimeField> {
    pub(crate) evals: Vec<F>,
    pub(crate) n_vars: usize,
}

impl<F: PrimeField> MultilinearPoly<F> {
    pub(crate) fn new(evaluations: Vec<F>) -> Self {
        let n_vars: usize = evaluations.len().ilog2() as usize;
        if evaluations.len() != 1 << n_vars {
            panic!("what are you doing?");
        }

        Self {
            evals: evaluations,
            n_vars,
        }
    }

    // pub(crate) fn evaluate(&self, assignments: &[F]) -> F {
    //     if assignments.len() != self.n_vars {
    //         panic!("what are you doing again?");
    //     }

    //     let mut poly = self.clone();

    //     for val in assignments {
    //         poly = poly.partial_evaluate(0, val);
    //     }

    //     poly.evals[0]
    // }
    pub fn evaluate(&self, values: Vec<F>) -> F {
        if values.len() != self.n_vars {
            panic!("Invalid number of values");
        }

        let mut result = self.clone();

        for value in values.iter() {
            result = result.partial_evaluate(0, value);
        }

        result.evals[0]
    }

    pub(crate) fn partial_evaluate(&self, index: usize, value: &F) -> Self {
        // use index to generate pairing
        // linear interpolate and evaluate <-- easy

        // 00 - (000, 100) - (0, 4)
        // 01 - (001, 101) - (1, 5)
        // 10 - (010, 110) - (2, 6)
        // 11 - (011, 111) - (3, 7)

        let mut result = vec![];
        // what does this need?
        // index <- 0 -> a
        // len of hypercube
        for (a, b) in pairs(index, self.n_vars).into_iter() {
            let a = self.evals[a];
            let b = self.evals[b];
            result.push(a + *value * (b - a));
        }

        Self::new(result)
    }

    pub fn multi_partial_evaluate(&self, values: &[F]) -> Self {
        if values.len() > self.n_vars {
            panic!("Invalid number of values");
        }

        let mut poly = self.clone();

        for (i, value) in values.iter().enumerate() {
            poly = poly.partial_evaluate(0, value);
        }

        poly
    }

    pub fn scale(&self, value: F) -> Self {
        let result = self.evals.iter().map(|eval| *eval * value).collect();

        Self::new(result)
    }
}

impl<F: PrimeField> Add for MultilinearPoly<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let result = self
            .evals
            .iter()
            .zip(other.evals.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        MultilinearPoly::new(result)
    }
}

impl<F: PrimeField> Mul for MultilinearPoly<F> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let result = self
            .evals
            .iter()
            .zip(other.evals.iter())
            .map(|(a, b)| *a * *b)
            .collect();

        MultilinearPoly::new(result)
    }
}

impl<F: PrimeField> Sub for MultilinearPoly<F> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let result = self
            .evals
            .iter()
            .zip(other.evals.iter())
            .map(|(a, b)| *a - *b)
            .collect();

        MultilinearPoly::new(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProductPoly<F: PrimeField> {
    pub evaluation: Vec<MultilinearPoly<F>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SumPoly<F: PrimeField> {
    pub polys: Vec<ProductPoly<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    pub fn new(evaluations: Vec<Vec<F>>) -> Self {
        let length_1 = evaluations[0].len();

        if evaluations.iter().any(|eval| eval.len() != length_1) {
            panic!("all evaluations must have same length");
        }

        let polys = evaluations
            .iter()
            .map(|evaluation| MultilinearPoly::new(evaluation.to_vec()))
            .collect();

        Self { evaluation: polys }
    }

    fn evaluate(&self, values: Vec<F>) -> F {
        self.evaluation
            .iter()
            .map(|poly| poly.evaluate(values.clone()))
            .product()
    }

    fn partial_evaluate(&self, value: &F) -> Self {
        let partial_polys = self
            .evaluation
            .iter()
            .map(|poly| {
                let partial_res = poly.partial_evaluate(0, value);

                partial_res.evals
            })
            .collect();

        Self::new(partial_polys)
    }

    fn reduce(&self) -> Vec<F> {
        (self.evaluation[0].clone() * self.evaluation[1].clone()).evals
    }

    fn get_degree(&self) -> usize {
        self.evaluation.len()
    }
}

impl<F: PrimeField> SumPoly<F> {
    pub fn new(polys: Vec<ProductPoly<F>>) -> Self {
        let degree_1 = polys[0].get_degree();
        if polys.iter().any(|poly| poly.get_degree() != degree_1) {
            panic!("all product polys must have same degree");
        }

        Self { polys }
    }

    pub fn evaluate(&self, values: Vec<F>) -> F {
        self.polys
            .iter()
            .map(|poly| poly.evaluate(values.clone()))
            .sum()
    }

    pub fn partial_evaluate(&self, value: &F) -> Self {
        let partial_polys = self
            .polys
            .iter()
            .map(|product_poly| product_poly.partial_evaluate(value))
            .collect();

        Self::new(partial_polys)
    }

    pub fn reduce(&self) -> Vec<F> {
        let poly_a = &self.polys[0].reduce();
        let poly_b = &self.polys[1].reduce();

        let result = poly_a
            .iter()
            .zip(poly_b.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        result
    }

    pub fn get_degree(&self) -> usize {
        self.polys[0].get_degree()
    }
}

// example
// 3 vars
// target_hc = 3 - 1 = 2
// 0..2^2 => 0..4
// 0 - 00
// 1 - 01
// 2 - 10
// 3 - 11

// _01
// 2 from 0
// 3 - 1 - index
fn pairs(index: usize, n_vars: usize) -> Vec<(usize, usize)> {
    let mut result = vec![];
    let target_hc = n_vars - 1;
    for val in 0..(1 << target_hc) {
        let inverted_index = n_vars - index - 1;
        let insert_zero = insert_bit(val, inverted_index);
        let insert_one = insert_zero | (1 << inverted_index);
        result.push((insert_zero, insert_one));
    }
    result
}

// always inserts 0
// 3 insert 0 at index 1 insert_bit(3, 1)
// 11 -> 101
fn insert_bit(value: usize, index: usize) -> usize {
    // high bit
    // 1011
    // right shift twice 101 10

    // insert a 0
    // 11 -> 110
    // inset at 2
    // 11 -> 011

    // 1011 & 0011
    // 1 << 2 = 100
    // 100 - 1 = 11
    let high = value >> index;
    let mask = (1 << index) - 1;
    let low = value & mask;

    // high | new_bit | low
    high << index + 1 | low
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{insert_bit, pairs, MultilinearPoly};
    use super::{ProductPoly, SumPoly};
    use ark_bn254::{Fq, Fr};

    pub(crate) fn to_field(input: Vec<u64>) -> Vec<Fr> {
        input.into_iter().map(|v| Fr::from(v)).collect()
    }

    #[test]
    fn bit_insertion() {
        assert_eq!(insert_bit(3, 0), 0b110);
        assert_eq!(insert_bit(3, 1), 0b101);
        assert_eq!(insert_bit(3, 2), 0b011);
    }

    #[test]
    fn test_pairs() {
        // 0 - 4
        // 1 - 5
        let pairs = pairs(2, 3);
    }

    #[test]
    fn test_partial_evaluate() {
        // 2ab + 3bc
        let poly = MultilinearPoly::new(to_field(vec![0_u64, 0, 0, 3, 0, 0, 2, 5]));
        assert_eq!(
            poly.partial_evaluate(2, &Fr::from(3)).evals,
            to_field(vec![0, 9, 0, 11])
        );
        assert_eq!(
            poly.partial_evaluate(1, &Fr::from(3)).evals,
            to_field(vec![0, 9, 6, 15])
        );
    }

    #[test]
    fn product_poly_evaluates_multiple_polys() {
        let evaluations = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let product_polys = ProductPoly::new(evaluations);

        let values = vec![Fq::from(2), Fq::from(3)];

        let expected_evaluation = Fq::from(216);

        let result = product_polys.evaluate(values);

        assert_eq!(expected_evaluation, result);
    }

    #[test]
    fn product_poly_partially_evaluates_multiple_polys() {
        let evaluations = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let product_polys = ProductPoly::new(evaluations);

        let value = Fq::from(2);

        let expected_evaluation = vec![
            vec![Fq::from(0), Fq::from(6)],
            vec![Fq::from(0), Fq::from(4)],
        ];

        let result = product_polys.partial_evaluate(&value);

        let result_polys: Vec<_> = result
            .evaluation
            .iter()
            .map(|poly| poly.evals.clone())
            .collect();

        assert_eq!(result_polys, expected_evaluation);
    }

    #[test]
    #[should_panic]
    fn product_poly_doesnt_allow_different_evaluation_size() {
        let evaluations = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(4),
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(4),
            ],
        ];

        let _ = ProductPoly::new(evaluations);
    }

    #[test]
    fn product_poly_gets_correct_degree() {}

    #[test]
    fn sum_poly_gets_correct_degree() {}

    #[test]
    fn sum_poly_evaluates_properly() {
        let evaluations_1 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let evaluations_2 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(4)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(5)],
        ];

        let product_poly_1 = ProductPoly::new(evaluations_1);
        let product_poly_2 = ProductPoly::new(evaluations_2);

        let sum_poly = SumPoly::new(vec![product_poly_1, product_poly_2]);

        let values = vec![Fq::from(2), Fq::from(3)];

        let expected_result = Fq::from(936);

        let result = sum_poly.evaluate(values);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn sum_poly_partially_evaluates_properly() {
        let evaluations_1 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let evaluations_2 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(4)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(5)],
        ];

        let product_poly_1 = ProductPoly::new(evaluations_1);
        let product_poly_2 = ProductPoly::new(evaluations_2);

        let value = Fq::from(2);

        let expected_evaluation_1 = vec![
            vec![Fq::from(0), Fq::from(6)],
            vec![Fq::from(0), Fq::from(4)],
        ];

        let expected_evaluation_2 = vec![
            vec![Fq::from(0), Fq::from(8)],
            vec![Fq::from(0), Fq::from(10)],
        ];

        let sum_poly = SumPoly::new(vec![product_poly_1, product_poly_2]);

        let result = sum_poly.partial_evaluate(&value);

        let result_polys: Vec<_> = result
            .polys
            .iter()
            .map(|product_poly| {
                product_poly
                    .evaluation
                    .iter()
                    .map(|poly| poly.evals.clone())
                    .collect::<Vec<_>>()
            })
            .collect();

        assert_eq!(
            vec![expected_evaluation_1, expected_evaluation_2],
            result_polys
        );
    }
}
