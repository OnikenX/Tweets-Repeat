# Tweets repeater

This is a rust server-client application that collects tweets with a certain hashtag, now it is configed with #DOGECOIN, and repeats them to the clients.
- The server uses the tokio lib to asyncronily collect and distribute tweets.
- The client is a shared library with a C ABI, with one call it connects to the server and collects the specified number of tweets.
