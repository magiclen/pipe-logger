Pipe Logger
====================

[![Build Status](https://travis-ci.org/magiclen/pipe-logger.svg?branch=master)](https://travis-ci.org/magiclen/pipe-logger)

Stores, rotates, compresses process logs.

## Help

```
EXAMPLES:
  pipe-logger /path/to/out.log                        # Stores log into /path/to/out.log
  pipe-logger /path/to/out.log -r 10M                 # The same as above, plus if its size is over than 10MB, it will
be rotated and renamed.
  pipe-logger /path/to/out.log -r 10M -c 4            # The same as above, plus the max count of log files is 4. The
oldest ones will be removed when the quota is exhausted.
  pipe-logger /path/to/out.log -r 10M -c 4 --compress # The same as above, plus the rotated log files are compressed by
xz.

USAGE:
    pipe-logger [FLAGS] [OPTIONS] <LOG_PATH>

FLAGS:
        --compress    Compresses the rotated log files. (Not support yet.)
        --err         Re-outputs logs through stderr.
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -c, --count <COUNT>      Assigns the max count of log files.
    -r, --rotate <ROTATE>    Rotates the log file.

ARGS:
    <LOG_PATH>    The path that you want to store your logs. [default: logfile.log]
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