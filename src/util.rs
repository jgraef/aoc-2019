use std::sync::Once;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        dotenv::dotenv().unwrap();
        pretty_env_logger::init();
    });
}
