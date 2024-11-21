use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct EnvVars {
    pub private_key: String,
}

pub fn load_env() -> Result<EnvVars, String> {
    dotenv().ok(); // Reads the .env file

    let private_key = env::var("PRIVATE_KEY");
    Ok(EnvVars {
        private_key: private_key.map_err(|e| format!("Error PRIVATE_KEY: {}", e))?,
    })
}
