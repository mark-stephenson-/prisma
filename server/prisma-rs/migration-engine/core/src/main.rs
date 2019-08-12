use migration_core::api::RpcApi;
use std::env;

fn main() {
    match env::var("CONNECTION_STRING") {
        Ok(ref config) => {
            let result = RpcApi::new(config).unwrap().handle().unwrap();
            println!("{}", result);
        }
        _ => panic!("CONNECTION_STRING environment variable is not set."),
    }
}
