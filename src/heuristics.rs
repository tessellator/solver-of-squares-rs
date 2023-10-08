use num::{abs, Signed};

pub fn manhattan_distance<T, const N: usize>(a: &[T; N], b: &[T; N]) -> T
where
    T: Signed + Copy,
{
    let mut distance = T::zero();

    for i in 0..N {
        distance = distance + abs(a[i] - b[i]);
    }

    distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_distance() {
        let a = [1.2, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        assert_eq!(manhattan_distance(&a, &b), 8.8);
    }
}
