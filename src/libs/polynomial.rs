use ark_ff::PrimeField;
use std::{
    iter::{Product, Sum},
    ops::{Add, Mul},
};

#[derive(Debug)]
pub struct UnivariatePolynomial<F: PrimeField> {
    pub coefficients: Vec<F>,
}

impl<F: PrimeField> UnivariatePolynomial<F> {
    pub fn new(coeff: Vec<F>) -> Self {
        UnivariatePolynomial {
            coefficients: coeff,
        }
    }

    fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }

    pub fn evaluate(&self, x: F) -> F {
        self.coefficients
            .iter()
            .rev()
            .cloned()
            .reduce(|acc, curr| acc * x + curr)
            .unwrap()
    }

    pub fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
        xs.iter()
            .zip(ys.iter())
            .map(|(x, y)| Self::basis(x, &xs).scalar_mul(y))
            .sum()
    }

    fn scalar_mul(&self, scalar: &F) -> Self {
        UnivariatePolynomial {
            coefficients: self
                .coefficients
                .iter()
                .map(|coeff| *coeff * *scalar)
                .collect(),
        }
    }

    fn basis(x: &F, interpolating_set: &[F]) -> Self {
        let numerator: UnivariatePolynomial<F> = interpolating_set
            .iter()
            .filter(|val| *val != x)
            .map(|x_n| UnivariatePolynomial::new(vec![x_n.neg(), F::one()]))
            .product();

        let denominator = F::one() / numerator.evaluate(*x);

        numerator.scalar_mul(&denominator)
    }
}

impl<F: PrimeField> Mul for &UnivariatePolynomial<F> {
    type Output = UnivariatePolynomial<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        // mul for dense
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![F::zero(); new_degree + 1];
        for i in 0..self.coefficients.len() {
            for j in 0..rhs.coefficients.len() {
                result[i + j] += self.coefficients[i] * rhs.coefficients[j]
            }
        }
        UnivariatePolynomial {
            coefficients: result,
        }
    }
}

impl<F: PrimeField> Add for UnivariatePolynomial<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        // let (mut bigger, smaller) = if self.degree() < rhs.degree() {
        //     (rhs.clone(), self)
        // } else {
        //     (self.clone(), rhs)
        // };
        let mut result = vec![F::zero(); self.coefficients.len().max(rhs.coefficients.len())];

        for (i, &coeff) in self.coefficients.iter().enumerate() {
            result[i] += coeff;
        }

        for (i, &coeff) in rhs.coefficients.iter().enumerate() {
            result[i] += coeff;
        }

        // let _ = bigger
        //     .coefficients
        //     .iter_mut()
        //     .zip(smaller.coefficients.iter())
        //     .map(|(b_coeff, s_coeff)| *b_coeff += s_coeff)
        //     .collect::<()>();

        UnivariatePolynomial::new(result)
    }
}

impl<F: PrimeField> Sum for UnivariatePolynomial<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = UnivariatePolynomial::new(vec![F::zero()]);
        for poly in iter {
            result = result + poly;
        }
        result
    }
}

impl<F: PrimeField> Product for UnivariatePolynomial<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = UnivariatePolynomial::new(vec![F::one()]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::libs::polynomial::UnivariatePolynomial;
    use ark_bn254::Fq;

    fn poly_1() -> UnivariatePolynomial<Fq> {
        // f(x) = 1 + 2x + 3x^2
        UnivariatePolynomial {
            coefficients: vec![Fq::from(1), Fq::from(2), Fq::from(3)],
        }
    }

    fn poly_2() -> UnivariatePolynomial<Fq> {
        // f(x) = 4x + 3 + 5x^11
        UnivariatePolynomial {
            coefficients: [
                vec![Fq::from(3), Fq::from(4)],
                vec![Fq::from(0); 9],
                vec![Fq::from(5)],
            ]
            .concat(),
        }
    }

    #[test]
    fn test_degree() {
        assert_eq!(poly_1().degree(), 2);
    }

    #[test]
    fn test_evaluation() {
        assert_eq!(poly_1().evaluate(Fq::from(2)), Fq::from(17));
    }

    #[test]
    fn test_addition() {
        // f(x) = 1 + 2x + 3x^2
        // f(x) = 4x + 3 + 5x^11

        // r(x) = 4 + 6x + 3x^2 + 5x^11
        assert_eq!(
            (poly_1() + poly_2()).coefficients,
            [
                vec![Fq::from(4), Fq::from(6), Fq::from(3)],
                vec![Fq::from(0); 8],
                vec![Fq::from(5)]
            ]
            .concat()
        )
    }

    #[test]
    fn test_mul() {
        // f(x) = 5 + 2x^2
        let poly_1: UnivariatePolynomial<Fq> = UnivariatePolynomial {
            coefficients: vec![Fq::from(5), Fq::from(0), Fq::from(2)],
        };
        // f(x) = 2x + 6
        let poly_2 = UnivariatePolynomial {
            coefficients: vec![Fq::from(6), Fq::from(2)],
        };

        // r(x) = 30 + 10x + 12x^2 + 4x^3
        assert_eq!(
            (&poly_1 * &poly_2).coefficients,
            vec![Fq::from(30), Fq::from(10), Fq::from(12), Fq::from(4)]
        );
    }

    #[test]
    fn test_interpolate() {
        // f(x) = 2x
        // [(2, 4), (4, 8)]
        let maybe_2x = UnivariatePolynomial::interpolate(
            vec![Fq::from(2), Fq::from(4)],
            vec![Fq::from(4), Fq::from(8)],
        );
        assert_eq!(maybe_2x.coefficients, vec![Fq::from(0), Fq::from(2)]);
    }
}
