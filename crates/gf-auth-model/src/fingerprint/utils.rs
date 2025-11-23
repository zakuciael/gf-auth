use rand::distributions::{Distribution, Uniform};

const VECTOR_CONTENT_LENGTH: usize = 100;

fn random_ascii_generator() -> Uniform<u8> {
  Uniform::new(32, 126)
}

pub fn random_ascii_char() -> char {
  random_ascii_generator()
    .sample(&mut rand::thread_rng())
    .into()
}

pub fn random_ascii_string(length: usize) -> String {
  random_ascii_generator()
    .sample_iter(&mut rand::thread_rng())
    .take(length)
    .map(char::from)
    .collect()
}

#[allow(dead_code)]
pub fn generate_vector() -> String {
  random_ascii_string(VECTOR_CONTENT_LENGTH)
}
