# Tweets repeater

This is a rust server-client application that collects tweets with a certain hashtag, it can be easily changed in the *token* file, now it is configured with #Ukraine, and repeats them to the clients.
- The server uses the tokio lib to asynchronous collect and distribute tweets.
- The client is a shared library with a C ABI, with one call it connects to the server and collects the specified number of tweets.