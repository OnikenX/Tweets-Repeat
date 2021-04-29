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


pub static LISTENING_ON_SERVER: &str = "/ip4/0.0.0.0/tcp/5556";
// static LISTNING_ON : &str = "/ip6/::/tcp/5556";

pub static TWEETS_FOLDER: &str = "tweets_history";


// static SERVER_MULTIADDR: &str = "/ip6/::1/tcp/5556";
pub static SERVER_MULTIADDR: &str = "/ip4/127.0.0.1/tcp/5556";
pub static LISTENING_ON_CLIENT: &str = "/ip4/0.0.0.0/tcp/0";

use serde::{Serialize, Deserialize};
use egg_mode::tweet::Tweet;
use egg_mode::user::TwitterUser;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TweetSerializable {
    pub user_name: String,
    pub user_screen_name: String,
    pub text: String,
    pub favorite_count: i32,
    pub retweet_count: i32,
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
            Some(twitterUser) => {
                user_name = String::from(twitterUser.name.clone());
                user_screen_name = String::from(twitterUser.screen_name.clone());
            }
        };
        TweetSerializable { retweet_count, favorite_count, text, user_screen_name, user_name, possibly_sensible, created_at }
    }
}