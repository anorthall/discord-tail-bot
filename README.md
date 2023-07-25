# Discord Tail Bot
Discord Tail Bot is a bot which will send all new lines written to a file to a
discord channel. It is intended to be used to monitor log files.

## Installation
Build the binary using cargo: `cargo build --release`.

## Usage
Place the configuration file, `configuration.yaml`, in the same directory as
the binary and run the binary. The configuration file should look like this:
```yaml
discord_token: "enter your discord bot token here"
log_file: "path/to/the/file/you/want/to/tail.txt"
channel_id: 1234567891011121314
pattern: "^\\[INFO] \\[\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}:\\d{2},\\d{3}] (.*)$"
```

`channel_id` is the id of the channel you want to send the messages to and
`pattern` is a regular expression which will be used to filter any lines
which are written to the file. Only lines which match the pattern will be
sent to the channel and the first capture group will be used as the message
content. Please note that any backslashes in the pattern must be escaped
with another backslash, as shown in the example above.

## License
This project is licensed under the GNU GPLv3 license. See the LICENSE file
for more information.
