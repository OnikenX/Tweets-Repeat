#[cfg(test)]
mod test {
    use std::str::FromStr;
    use std::time::UNIX_EPOCH;
    use chrono::Utc;

    #[test]
    //conversation to various types and of time
    fn chrono_stringing() {
        let systemtime = std::time::SystemTime::now();
        dbg!(&systemtime);
        let datetime_now: chrono::DateTime<Utc> = systemtime.into();
        let time_string = datetime_now.to_rfc3339();
        dbg!(&time_string);
        let datetime_now = chrono::DateTime::parse_from_rfc3339(&time_string);
        dbg!(&datetime_now);
    }
}


// pub static LISTENING_ON_SERVER_MULTIADDR: &str = "/ip4/"+LISTENING_ON_SERVER_ADDR+"/tcp/"+LISTENING_ON_SERVER_PORT;
pub static LISTENING_ON_SERVER_ADDR: &str = "0.0.0.0";
pub static LISTENING_ON_SERVER_PORT: &str = "5556";
// pub static LISTENING_ON_SERVER: &str = LISTENING_ON_SERVER_ADDR.to_string().add(":").add(LISTENING_ON_SERVER_PORT).as_str();

pub static SERVER_ADDR: &str = "127.0.0.1";
// pub static SERVER_MULTIADDR: &str = "/ip4/"+SERVER_ADDR+"/tcp/"+LISTENING_ON_SERVER_PORT;
// pub static LISTENING_ON_CLIENT: &str = "/ip4/"+LISTENING_ON_CLIENT_ADDR+"/tcp/"+LISTENING_ON_CLIENT_PORT;
pub static LISTENING_ON_CLIENT_ADDR: &str = "0.0.0.0";
pub static LISTENING_ON_CLIENT_PORT: &str = "0";


pub static TWEETS_FOLDER: &str = "tweets_history";

use serde::{Serialize, Deserialize};
use egg_mode::tweet::Tweet;
use egg_mode::user::TwitterUser;
use std::ops::Add;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TweetSerializable {
    // display
    pub user_name: String,
    // @ of user
    pub user_screen_name: String,
    // content of the tweet
    pub text: String,
    pub favorite_count: i32,
    pub retweet_count: i32,
    // has a link to sensible content
    pub possibly_sensible: bool,
    pub created_at: String,
}

impl TweetSerializable {
    pub fn from(tweet: Tweet) -> TweetSerializable {
        let user_name;
        let user_screen_name;
        let text = tweet.text.clone().to_string();
        let favorite_count = tweet.favorite_count;
        let retweet_count = tweet.retweet_count;
        let possibly_sensible = tweet.possibly_sensitive.or_else(|| -> Option<bool>{ Some(false) }).unwrap();
        let created_at = tweet.created_at.to_rfc3339();
        match &tweet.user {
            None => {
                user_name = String::new();
                user_screen_name = String::new();
            }
            Some (twitter_user) => {
                user_name = String::from(twitter_user.name.clone());
                user_screen_name = String::from(twitter_user.screen_name.clone());
            }
        };
        TweetSerializable { retweet_count, favorite_count, text, user_screen_name, user_name, possibly_sensible, created_at }
    }
}