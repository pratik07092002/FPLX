use rand::{distributions::Alphanumeric, Rng};

pub fn generate_code() -> String {

    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    format!("FPLX-{}", code)
}
