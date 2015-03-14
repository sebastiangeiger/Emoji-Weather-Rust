# Emoji Weather - Rust Style [![Build Status](https://travis-ci.org/sebastiangeiger/Emoji-Weather-Rust.svg?branch=master)](https://travis-ci.org/sebastiangeiger/Emoji-Weather-Rust)

Retrieves the current weather from the
[forecast.io API](https://developer.forecast.io/docs/v2) and displays a weather
emoji.

This is my first 'real world' rust project.


## TODO
  * [ ] Make it read from `~/.weather.conf`
  * [ ] Better error output
  * [ ] Retry 3 times on error

## Running it as a service

  * `cp net.sebastiangeiger.emoji_weather_rust.plist ~/Library/LaunchAgents`
  * Set owner to root, chmod to 644
  * `sudo launchctl load ~/Library/LaunchAgents/net.sebastiangeiger.emoji_weather_rust.plist`
  * `setenv` in `/etc/launchd.conf`
