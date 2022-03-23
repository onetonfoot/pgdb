# Pgdb

Pgdb is a single [binary](https://github.com/onetonfoot/pgdb/releases) that can be used to run a postgres database on any major platform.

```
# pgdb --help

USAGE:
    pgdb [FLAGS] [OPTIONS]

FLAGS:
    -h, --help              Prints help information
        --non-persistent    If non-persistent delete files and directories on exit, otherwise keep them
        --print-url         Wether to print the access url to the stdout
    -V, --version           Prints version information

OPTIONS:
        --data-dir <data-dir>              Optional a directory to store posgres data [default: data/db]
        --database <database>              Database name [default: postgres]
        --migration-dir <migration-dir>    Optional a directory with migration scripts to apply
        --password <password>              Password [default: password]
        --port <port>                      Port [default: 5432]
        --user <user>                      User [default: postgres]
```
