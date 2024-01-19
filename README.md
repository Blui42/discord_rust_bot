# discord_rust_bot

This is a simple, general-purpose discord bot written in Rust.

## Usage 

This bot can be started using `cargo run --release`
and stopped with the key combination `Ctrl-C`

## Configuration

This Bot is configured via the environment.  
It will load any environment variables declared in the file `.env`.  

Current configuration options:

### `DISCORD_TOKEN=yourBotToken`

The authentication token to use. 
This Option is required.
If you don't have a token, set up your Bot on the
[Discord Developer Portal](https://discord.com/developers/applications/)


### `DISCORD_BOT_OWNERS=1234567890,9876543210`

Comma-separated list of IDs of owners to add.
Owners set on the [Discord Developer Portal](https://discord.com/developers/applications/)
will be automatically added as well.


### `DISCORD_HOME_SERVER=1234567890`

ID of the home server of the bot.
Commands will be only added to the home server.
If not specified, the commands will be added globally.

## Copying

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received [a copy of the GNU General Public License](./LICENSE.md)
along with this program.  If not, see https://www.gnu.org/licenses/.

