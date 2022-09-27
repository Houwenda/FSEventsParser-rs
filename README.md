# FSEventsParser-rs
Yet another fseventsd log parser for forensics.

## Usage
Currently supports output in three formats: JSON, CSV, Sqlite(recommended).
```bash
% ./fsevents_parser_rs -h
fsevents_parser_rs 0.1.0

USAGE:
    fsevents_parser_rs [OPTIONS]

OPTIONS:
    -f, --format <FORMAT>              [default: json] [possible values: json, csv, sqlite]
    -h, --help                         Print help information
    -i, --input-path <INPUT_PATH>      [default: /System/Volumes/Data/.fseventsd]
    -o, --output-path <OUTPUT_PATH>    [default: ./output.json]
    -V, --version                      Print version information
```

Use JSON output format.
```bash
% sudo ./fsevents_parser_rs
found 21 archives in /System/Volumes/Data/.fseventsd
---------- 0000000000089492 ----------
page count: 2
entry count: 2049
entry count: 681
......
......
---------- 000000000004c323 ----------
page count: 2
entry count: 1850
entry count: 880
% cat ./output.json | tail -n 3
{"path":"private/var/log/DiagnosticMessages/StoreData\u0000","id":308039,"flags":"FSE_CONTENT_MODIFIED | FSE_IS_FILE","create_ts":1664093703,"modiy_ts":1664093703,"source":"000000000004c323"}
{"path":"private/var/log/system.log\u0000","id":308036,"flags":"FSE_CONTENT_MODIFIED | FSE_IS_FILE","create_ts":1664093703,"modiy_ts":1664093703,"source":"000000000004c323"}
{"path":"private/var/root/Library/Logs/Bluetooth/bluetoothd-hci-latest.pklg\u0000","id":309733,"flags":"FSE_CONTENT_MODIFIED | FSE_IS_FILE","create_ts":1664093703,"modiy_ts":1664093703,"source":"000000000004c323"}
```

Use Sqlite output format.
```
% sudo ./target/debug/fsevents_parser_rs -o ./output.sqlite -f sqlite
found 8 archives in /System/Volumes/Data/.fseventsd
---------- 000000000000c760 ----------
page count: 2
entry count: 1947
entry count: 783
......
......
---------- 000000000001186b ----------
page count: 2
entry count: 2217
entry count: 513
% sqlite3 ./output.sqlite 'select * from record;' | tail -n 3
private/var/run/utmpx|4613|FSE_CONTENT_MODIFIED | FSE_IS_FILE|1664298667|1664298667|000000000000489c
private/var/sntpd/state.bin|15973|FSE_STAT_CHANGED | FSE_IS_FILE|1664298667|1664298667|000000000000489c
private/var/tmp/kernel_panics|4276|FSE_CHOWN | FSE_IS_DIR|1664298667|1664298667|000000000000489c
```

## References
[FSEventsParser](https://github.com/dlcowen/FSEventsParser)

[MacOS File System Events Disk Log Stream format](https://github.com/libyal/dtformats/blob/main/documentation/MacOS%20File%20System%20Events%20Disk%20Log%20Stream%20format.asciidoc)

[macos-fseventsd](https://github.com/puffyCid/macos-fseventsd)