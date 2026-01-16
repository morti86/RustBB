#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_maxage: i64,
    pub port_http: u16,
    pub port_https: u16,
    pub email_verification: bool,
    pub host_url: String,
    pub upload_dir: String,
    pub max_file_size: usize,
    pub allowed_image_types: Vec<String>,
    pub rtlim: bool,
    pub rtlim_burst: u32,
    pub rtlim_per_second: u64,
}

impl Config {

    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let rtlim = std::env::var("RTLIM").expect("RTLIM must be set");
        let jwt_secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        let email_verification =std::env::var("VERIFY_EMAIL").expect("VERIFY_EMAIL must be set");
        let host_url =std::env::var("HOST_URL").expect("HOST_URL must be set");
        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
        let rtlim_burst = std::env::var("RTLIM_BURST").unwrap();
        let rtlim_per_second = std::env::var("RTLIM_PER_SECOND").unwrap();
        let max_file_size = std::env::var("MAX_FILE_SIZE")
            .unwrap_or_else(|_| "5242880".to_string()) // 5MB default
            .parse::<usize>()
            .unwrap_or(5242880);

        Config {
            database_url,
            jwt_secret,
            jwt_maxage: jwt_maxage.parse::<i64>().unwrap(),
            port_https: 8080,
            port_http: 8000,
            email_verification: email_verification.parse::<bool>().unwrap(),
            host_url,
            upload_dir,
            max_file_size,
            rtlim_burst: rtlim_burst.parse().unwrap_or(10),
            rtlim_per_second: rtlim_per_second.parse().unwrap_or(2),
            rtlim: rtlim.parse().unwrap_or(false),
            allowed_image_types: vec![
                "image/jpeg".to_string(),
                "image/jpg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
            ],
        }
    }

}
