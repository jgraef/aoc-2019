
pub fn init() {
    dotenv::dotenv().unwrap();
    pretty_env_logger::init();
}
