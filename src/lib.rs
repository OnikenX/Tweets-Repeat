#[cfg(test)]
mod tests {
    use crate::ac7ion::Ac7ion;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_twitter_api() {
        //IMPORTANT: make an app for yourself at apps.twitter.com and get your
        //key/secret into these files; these examples won't work without them
        let action = Ac7ion::generic_init();
    }
}

mod ac7ion {
    //! Lib for AC7ION project
    use async_std::sync::Arc;
    use egg_mode::KeyPair;

    pub struct Ac7ion {
        pub token: egg_mode::KeyPair,
        pub clients : Arc<Vec<a7_client::A7Client>>,
    }
    mod a7_client {
        pub struct A7Client {
            pub token : &egg_mode::Token,
            pub user_id : u64,
            pub screen_name : String,
        }
    }

    impl Ac7ion {

        /// Gets the consumer_key and consumer_secret contents in files
        /// with the same name in the path, and creates a new Ac7tion with ```Ac7tion::new()```
        ///
        /// # Panic
        /// Panics if an error occurs with the authentication
        pub async fn generic_init() -> Ac7ion{
            let consumer_key = include_str!("consumer_key").trim();
            let consumer_secret = include_str!("consumer_secret").trim();
            Ac7ion::new(String::from(consumer_key), String::from(consumer_secret)).await.unwrap()
        }


        /// The initializer for the Ac7ion struct
        ///
        /// It verifies the keys given and returns the struct if the
        /// verification of the app went well
        ///
        /// # Arguments
        /// Receives an public and secret key, those are given in the app dashboard
        ///
        /// # Error
        /// It returns the error given by the app authenticator
        pub async fn new(public_key: String, secret_key: String) -> Result<Ac7ion, egg_mode::error::Error> {
            let con_token = egg_mode::KeyPair::new(public_key.clone(), secret_key.clone());
            // "oob" is needed for PIN-based auth; see docs for `request_token` for more info
            match egg_mode::auth::request_token(&con_token, "oob").await{
                Ok(token) =>{
                    Ok(Ac7ion { token, clients: Arc::new(Vec::new())})
                }
                Err(e) => {
                    Err(e)
                }
            }
        }

        /// Adds a new client to the list
        pub async fn add_client(&self, pin: i64){

        }

        pub async fn string_to_keypair(key :String, secret : String) -> KeyPair {
            egg_mode::KeyPair::new(key, secret)
        }

        pub async fn connect_client(&self, username: String, user_id : u64, acess_token : KeyPair){
            let token = egg_mode::Token::Access{
                consumer: self.token,
                access: acess_token
            }
        }
    }
}
