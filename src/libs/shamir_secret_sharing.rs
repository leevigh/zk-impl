use crate::libs::polynomial::UnivariatePolynomial;
use ark_bn254::Fq;
use ark_std::rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// fn main() {
//     recreate_polynomial(vec![1.0, 2.0, 3.0, 4.0], vec![1.0, 2.0, 3.0, 4.0], 3)
// }

// pub fn recreate_polynomial(xs: Vec<f64>, ys: Vec<f64>, threshold: usize) -> UnivariatePolynomial {
//     if xs.len() < threshold && ys.len() < threshold {
//         panic!("Not enough points to recreate polynomial");
//     }

//     // let selected_points = if points.len() > 3 {
//     //     points[0..4].to_vec()
//     // } else {
//     //     points.clone()
//     // };

//     UnivariatePolynomial::interpolate(xs, ys)
// }

// fn get_secret(poly: &UnivariatePolynomial, x_point: f64) -> f64 {
//     let secret = poly.evaluate(x_point);

//     secret
// }

// fn share_points(shares: usize, poly: &UnivariatePolynomial) -> Vec<(f64, f64)> {
//     let mut rng = rand::thread_rng();

//     let mut shares: Vec<(f64, f64)> = vec![(0.0, 0.0); shares];

//     for i in 0..shares.len() {
//         let random_x_point: f64 = rng.gen_range(0.0..=100.0);

//         let y_point = poly.evaluate(random_x_point as f64);

//         shares[i] = (random_x_point as f64, y_point);
//     }

//     shares
// }

// def create_shares(secret, minimum, shares):
//     # Create coefficients for our polynomial
//     coef = [secret] + [random.randint(0, 1000) for _ in range(minimum-1)]

//     # Generate points on the polynomial
//     points = []
//     for i in range(1, shares + 1):
//         # Evaluate polynomial at point i
//         accum = Decimal(0)
//         for coeff in reversed(coef):
//             accum *= i
//             accum += Decimal(coeff)
//         points.append((i, accum))

//     return points
fn create_share(polynomial: Vec<Fq>, threshold: usize, num_of_points: usize) -> Vec<(Fq, Fq)> {
    if threshold > num_of_points {
        panic!("Threshold should not be greater than x_points");
    }
    let mut rng = StdRng::from_entropy();

    let poly = UnivariatePolynomial::new(polynomial);

    let mut shares: Vec<(Fq, Fq)> = vec![];

    for _i in 0..num_of_points {
        let random_x = rng.gen();
        let y_points = poly.evaluate(random_x);
        shares.push((random_x, y_points))
    }

    shares
}

fn get_secret(poly: &UnivariatePolynomial<Fq>, x_point: Fq) -> Fq {
    let secret = poly.evaluate(Fq::from(x_point));

    secret
}

#[cfg(test)]

mod test {
    use crate::libs::{
        polynomial::{self, UnivariatePolynomial},
        shamir_secret_sharing::get_secret,
    };
    use ark_bn254::Fq;

    use super::create_share;

    #[test]
    fn test_create_share() {
        let shares = create_share(vec![Fq::from(3), Fq::from(1), Fq::from(0)], 2, 5);

        assert_eq!(shares.len(), 5);
    }

    #[test]
    fn test_get_secret() {
        let polynomial = vec![Fq::from(3), Fq::from(1), Fq::from(0)];
        let poly = UnivariatePolynomial::new(polynomial);

        let secret = get_secret(&poly, Fq::from(1));

        assert_eq!(secret, Fq::from(4));
    }
}
