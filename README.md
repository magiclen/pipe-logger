Pipe Logger
====================

[![CI](https://github.com/magiclen/pipe-logger/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/pipe-logger/actions/workflows/ci.yml)

Stores, rotates, compresses process logs.

## Help

```
EXAMPLES:
pipe-logger /path/to/out.log                          # Store log into /path/to/out.log
pipe-logger /path/to/out.log -r 10M                   # The same as above, plus if its size is over than 10MB, it will be rotated and renamed
pipe-logger /path/to/out.log -r 10M -c 4              # The same as above, plus the max count of log files is 4. The oldest ones will be removed when the quota is exhausted
pipe-logger /path/to/out.log -r 10M -c 4 --compress   # The same as above, plus the rotated log files are compressed by xz

Usage: pipe-logger [OPTIONS]

Options:
  -r, --rotate <ROTATE>      Rotate the log file
  -c, --count <COUNT>        Assign the max count of log files
      --compress             Compress the rotated log files
      --err                  Re-output logs through stderr
      --log-path <LOG_PATH>  The path that you want to store your logs [default: logfile.log]
  -h, --help                 Print help
  -V, --version              Print version
```

## Examples

```bash
lovable_process | pipe-logger -r 10m -c 5 --compress mylog.txt
```

```bash
lovable_process > >(pipe-logger mylog.txt) 2> >(pipe-logger --err error-mylog.txt)
```

## License

[MIT](LICENSE)