const DIGITS: i64 = 2;

pub fn netflix_buckets(min: f64, max: f64) -> Vec<f64> {
    let mut buckets = all_buckets()
        .into_iter()
        .filter(|&f| f > min && f < max)
        .collect::<Vec<_>>();

    buckets.insert(0, min);
    buckets.push(max);

    buckets.into_iter().map(|f| f / 1e9).collect()
}

fn all_buckets() -> Vec<f64> {
    let mut buckets = Vec::new();
    buckets.push(1.0);
    buckets.push(2.0);
    buckets.push(3.0);

    let mut exp = DIGITS;
    while exp < 64 {
        let mut current = 1_i64 << exp;
        let delta = current / 3;
        let next = (current << DIGITS) - delta;

        while current < next {
            buckets.push(current as f64);
            current += delta;
        }
        exp += DIGITS;
    }
    buckets.push(f64::INFINITY);
    buckets
}
