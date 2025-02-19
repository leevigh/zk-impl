use ark_ff::PrimeField;

#[derive(Clone)]
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

    pub(crate) fn evaluate(&self, assignments: &[F]) -> F {
        if assignments.len() != self.n_vars {
            panic!("what are you doing again?");
        }

        let mut poly = self.clone();

        for val in assignments {
            poly = poly.partial_evaluate(0, val);
        }

        poly.evals[0]
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
    use ark_bn254::Fr;

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
}

// Define a type alias for Hypercube - it's a vector of tuples containing a vector of bytes and a field element
// This represents points in a binary hypercube with their corresponding field values
// type Hypercube<F> = Vec<(Vec<u8>, F)>;

// Define a struct to represent a Binary Hypercube (BHC)
// It contains the points and values of the hypercube
// struct BHC<F> {
//     bits: Hypercube<F>,
// }

// Define the main MultilinearPoly struct that can be cloned
// It stores the evaluation points of a multilinear polynomial
// #[derive(Clone)]
// pub struct MultilinearPoly<F: PrimeField> {
//     pub evaluation: Vec<F>,
// }

// Implementation block for BHC
// impl<F: PrimeField> BHC<F> {
//     // Constructor for BHC
//     fn new(hypercube: Hypercube<F>) -> Self {
//         BHC { bits: hypercube }
//     }

//     // Generate a binary hypercube from polynomial evaluation points
//     fn generate_bhc(poly_evaluation: Vec<F>) -> Self {
//         // Calculate number of bits needed based on input length
//         let bits = poly_evaluation.len().ilog2() as usize;
//         let size = 1 << bits; // Calculate 2^bits

//         // Create the hypercube by mapping each index to its binary representation
//         let hypercube: Hypercube<F> = (0..size)
//             .map(|i| {
//                 // Convert index to binary representation
//                 let point = (0..bits).rev().map(|j| ((i >> j) & 1) as u8).collect();
//                 (point, poly_evaluation[i])
//             })
//             .collect();
//         Self::new(hypercube)
//     }

//     // Pair points in the hypercube based on a specific bit position
//     fn pair_points(&self, bit: u8) -> Vec<(F, F)> {
//         let mut pairs = Vec::new();
//         let pair_index = 1 << bit; // Calculate 2^bit

//         // Create pairs of points that differ only in the specified bit
//         for i in 0..pair_index {
//             if i + pair_index < self.bits.len() {
//                 let (_, a) = &self.bits[i];
//                 let (_, b) = &self.bits[i + pair_index];
//                 pairs.push((a.clone(), b.clone()));
//             }
//         }
//         pairs
//     }
// }

// Implementation block for MultilinearPoly
// impl<F: PrimeField> MultilinearPoly<F> {
//     // Constructor for MultilinearPoly
//     pub fn new(evaluations: Vec<F>) -> Self {
//         MultilinearPoly {
//             evaluation: evaluations,
//         }
//     }

//     // Linear interpolation between two points
//     fn interpolate(points: (F, F), value: F) -> F {
//         let (y_0, y_1) = points;
//         // Compute y_0 + t(y_1 - y_0) where t is the value
//         y_0 + (value * (y_1 - y_0))
//     }

//     // Partially evaluate the polynomial at a specific bit position
//     pub fn partial_evaluate(&self, value: F, bit: u8) -> Self {
//         // Generate binary hypercube from current evaluation
//         let bhc = BHC::generate_bhc(self.evaluation.clone());

//         // Interpolate paired points using the given value
//         let paired_evaluations = bhc
//             .pair_points(bit)
//             .iter()
//             .map(|point| Self::interpolate(*point, value))
//             .collect();
//         Self::new(paired_evaluations)
//     }

//     // Evaluate the polynomial at multiple points
//     pub fn evaluate(&self, values: Vec<F>) -> F {
//         let mut result = self.clone();
//         let mut bits = result.evaluation.len().ilog2() - 1;

//         // Iteratively evaluate the polynomial one bit at a time
//         for value in values.iter() {
//             result = result.partial_evaluate(*value, bits.try_into().unwrap());
//             if bits == 0 {
//                 break;
//             } else {
//                 bits -= 1;
//             }
//         }
//         result.evaluation[0]
//     }
// }

// Implement addition for MultilinearPoly
// impl<F: PrimeField> Add for MultilinearPoly<F> {
//     type Output = Self;

//     // Add two multilinear polynomials component-wise
//     fn add(self, other: Self) -> Self {
//         // Create vector of appropriate size filled with zeros
//         let mut result = vec![F::zero(); self.evaluation.len().max(other.evaluation.len())];

//         // Add components from first polynomial
//         for (i, &value) in self.evaluation.iter().enumerate() {
//             result[i] += value;
//         }

//         // Add components from second polynomial
//         for (i, &value) in other.evaluation.iter().enumerate() {
//             result[i] += value;
//         }

//         MultilinearPoly::new(result)
//     }
// }
// #[cfg(test)]
// mod test {
//     use super::*;
//     use ark_bn254::Fq;

//     #[test]
//     fn it_pair_points_correctly() {
//         let evaluations = vec![Fq::from(0), Fq::from(1), Fq::from(2), Fq::from(3)];
//         let bhc = BHC::generate_bhc(evaluations);

//         let pairs = bhc.pair_points(1);
//         assert_eq!(
//             pairs,
//             vec![(Fq::from(0), Fq::from(2)), (Fq::from(1), Fq::from(3))]
//         );
//     }

//     #[test]
//     fn it_partially_evaluates_any_multilinear() {
//         let evaluations = vec![Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(10)];
//         let polynomial = MultilinearPoly::new(evaluations);

//         let value_a = Fq::from(5);
//         let bit_a = 1;

//         let result = polynomial.partial_evaluate(value_a, bit_a);

//         assert_eq!(result.evaluation, vec![Fq::from(15), Fq::from(50)]);
//     }

//     #[test]
//     fn it_fully_evaluates_any_multilinear() {
//         let evaluations = vec![Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(10)];
//         let polynomial = MultilinearPoly::new(evaluations);

//         let values = vec![Fq::from(5), Fq::from(1)];

//         let result = polynomial.evaluate(values);

//         assert_eq!(result, Fq::from(50));
//     }
// }

// use ark_ff::PrimeField;

// struct MultilinearPoly {
//     coefficients: Vec<i32>,
//     num_of_variables: usize,
// }

// impl MultilinearPoly {
//     fn getPair(hypercube: &Vec<Vec<i32>>, eval: i32) -> Vec<(usize, usize)> {
//         let mut pair: Vec<(usize, usize)> = Vec::new();
//         let mut on_evaluator_bit = Vec::new();
//         let mut off_evaluator_bit = Vec::new();

//         for (index, point) in hypercube.iter().enumerate() {
//             if point[0] == 0 {
//                 off_evaluator_bit.push(index);
//             } else {
//                 on_evaluator_bit.push(index);
//             }
//         }

//         for (&on_idx, &off_idx) in off_evaluator_bit.iter().zip(on_evaluator_bit.iter()) {
//             pair.push((on_idx, off_idx));
//         }

//         pair
//     }

//     fn partial_eval(pairs: Vec<(usize, usize)>, eval: f64) -> Vec<usize> {
//         let mut result = Vec::new();

//         // y_1 + r(y_2 - y_1); formula to interpolate & evaluate one take
//         // y_1 is tup.0 and y_2 is tup.1
//         for pair in pairs {
//             let y_1 = pair.0 as f64;
//             let y_2 = pair.1 as f64;
//             // let computation = y_1 + eval * (y_2 - y_1);
//             // let computation = pair.0 + (eval*(pair.1 - pair.0));
//             // let computation = pair.0 as i32 + (mult as i32);
//             // result.push(computation);
//             // println!("{:?}", pair)
//             println!("{:?}", y_1 + eval * (y_2 - y_1));
//             println!("{} {}", pair.0, pair.1);
//         }

//         result
//     }
// }
