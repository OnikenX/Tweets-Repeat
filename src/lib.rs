#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_twitter_api() {
        use dotenv::dotenv;
        use std::env;
        dotenv().ok();
      
        for (key, value) in env::vars() {
            println!("{}: {}", key, value);
        }
        let secret = env::vars()["secret_key"]
    }
}


mod AC7ION {
}
