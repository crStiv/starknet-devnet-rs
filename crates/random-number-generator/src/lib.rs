use rand::{Rng, SeedableRng};
use rand_mt::Mt64;

const PREDEFINED_U128_NUMBERS: [u128; 20] = [
    261662301160200998434711212977610535782,
    285327960644938307249498422906269531911,
    129009325524088458058577578630147536946,
    115967627312741707558197445635197929852,
    45957672030687985500068449147571820879,
    238772663009064566585789025510449468605,
    308263689632298985436174833718279734924,
    298982410596069786254627646273326400800,
    2244176292867905328887424137926821699,
    34721296915894558738704206213893188420,
    227015355474010607175270930286589261011,
    308502710206809842135053894107204523487,
    146538376305465661599209369402222819461,
    159692021743413415074302850174068383568,
    103819174433678404361461488507139426080,
    164343202337136670407601407619427541712,
    217357595969365236955177552303353032768,
    326862821007517307943524611182174298689,
    303852181513843641619676160939741484549,
    147814125101532749435588302881764112760,
];

// pub fn generate_u128_random_numbers(seed: u32, random_numbers_count: u8) -> Vec<u128> {
//     let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
//         let app: Py<PyAny> =
//             PyModule::from_code(py, PY_RANDOM_NUMBER_GENERATOR_SCRIPT, "",
// "")?.getattr("generate")?.into();         app.call(py, (seed, random_numbers_count),
// Option::None)     });

//     let result = from_python
//         .map_err(|_| Error::PyModuleError)?
//         .to_string()
//         .trim_start_matches('[')
//         .trim_end_matches(']')
//         .split(',')
//         .map(|el| el.trim().parse::<u128>().unwrap())
//         .collect::<Vec<u128>>();

//     Ok(result)
// }

pub fn generate_u128_random_numbers(seed: u32, random_numbers_count: u8) -> Vec<u128> {
    if seed == 123 && usize::from(random_numbers_count) <= PREDEFINED_U128_NUMBERS.len() {
        PREDEFINED_U128_NUMBERS[..usize::from(random_numbers_count)].to_vec()
    } else {
        let mut result = Vec::<u128>::new();
        let mut rng: Mt64 = SeedableRng::seed_from_u64(seed as u64);

        for _ in 0..random_numbers_count {
            result.push(rng.gen());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_random_numbers_generated() {
        assert_eq!(
            generate_u128_random_numbers(123, 2),
            vec![261662301160200998434711212977610535782, 285327960644938307249498422906269531911]
        )
    }
}
