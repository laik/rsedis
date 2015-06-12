- commands
    - [x] get
    - [x] set
    - [x] setnx
    - [x] setex
    - [x] psetex
    - [x] append
    - [x] strlen
    - [x] del
    - [x] exists
    - [ ] setbit
    - [ ] getbit
    - [ ] setrange
    - [ ] getrange
    - [ ] substr
    - [x] incr
    - [x] decr
    - [ ] mget
    - [x] rpush
    - [x] lpush
    - [x] rpushx
    - [x] lpushx
    - [x] linsert
    - [x] rpop
    - [x] lpop
    - [ ] brpop
    - [x] brpoplpush
    - [ ] blpop
    - [x] llen
    - [x] lindex
    - [x] lset
    - [x] lrange
    - [x] ltrim
    - [x] lrem
    - [x] rpoplpush
    - [x] sadd
    - [ ] srem
    - [ ] smove
    - [ ] sismember
    - [x] scard
    - [ ] spop
    - [ ] srandmember
    - [ ] sinter
    - [ ] sinterstore
    - [ ] sunion
    - [ ] sunionstore
    - [x] sdiff
    - [x] sdiffstore
    - [ ] smembers
    - [ ] sscan
    - [ ] zadd
    - [ ] zincrby
    - [ ] zrem
    - [ ] zremrangebyscore
    - [ ] zremrangebyrank
    - [ ] zremrangebylex
    - [ ] zunionstore
    - [ ] zinterstore
    - [ ] zrange
    - [ ] zrangebyscore
    - [ ] zrevrangebyscore
    - [ ] zrangebylex
    - [ ] zrevrangebylex
    - [ ] zcount
    - [ ] zlexcount
    - [ ] zrevrange
    - [ ] zcard
    - [ ] zscore
    - [ ] zrank
    - [ ] zrevrank
    - [ ] zscan
    - [ ] hset
    - [ ] hsetnx
    - [ ] hget
    - [ ] hmset
    - [ ] hmget
    - [ ] hincrby
    - [ ] hincrbyfloat
    - [ ] hdel
    - [ ] hlen
    - [ ] hstrlen
    - [ ] hkeys
    - [ ] hvals
    - [ ] hgetall
    - [ ] hexists
    - [ ] hscan
    - [x] incrby
    - [x] decrby
    - [ ] incrbyfloat
    - [ ] getset
    - [ ] mset
    - [ ] msetnx
    - [ ] randomkey
    - [x] select
    - [ ] move
    - [ ] rename
    - [ ] renamenx
    - [x] expire
    - [x] expireat
    - [x] pexpire
    - [x] pexpireat
    - [ ] keys
    - [ ] scan
    - [ ] dbsize
    - [ ] auth
    - [x] ping
    - [ ] echo
    - [ ] save
    - [ ] bgsave
    - [ ] bgrewriteaof
    - [ ] shutdown
    - [ ] lastsave
    - [x] type
    - [ ] multi
    - [ ] exec
    - [ ] discard
    - [ ] sync
    - [ ] psync
    - [ ] replconf
    - [x] flushdb
    - [x] flushall
    - [ ] sort
    - [ ] info
    - [ ] monitor
    - [x] ttl
    - [x] pttl
    - [x] persist
    - [ ] slaveof
    - [ ] role
    - [ ] debug
    - [ ] config
    - [x] subscribe
    - [x] unsubscribe
    - [x] psubscribe
    - [x] punsubscribe
    - [x] publish
    - [ ] pubsub
    - [ ] watch
    - [ ] unwatch
    - [ ] cluster
    - [ ] restore
    - [ ] restore-asking
    - [ ] migrate
    - [ ] asking
    - [ ] readonly
    - [ ] readwrite
    - [ ] dump
    - [ ] object
    - [ ] client
    - [ ] eval
    - [ ] evalsha
    - [ ] slowlog
    - [ ] script
    - [ ] time
    - [ ] bitop
    - [ ] bitcount
    - [ ] bitpos
    - [ ] wait
    - [ ] command
    - [ ] pfselftest
    - [ ] pfadd
    - [ ] pfcount
    - [ ] pfmerge
    - [ ] pfdebug
    - [ ] latency
- config
    - [ ] include
    - [ ] daemonize
    - [ ] pidfile
    - [x] port
    - [ ] tcp-backlog
    - [x] bind
    - [ ] unixsocket
    - [ ] unixsocketperm
    - [ ] timeout
    - [ ] tcp-keepalive
    - [ ] loglevel
    - [ ] logfile
    - [ ] syslog-enabled
    - [ ] syslog-ident
    - [ ] syslog-facility
    - [ ] databases
    - [ ] save
    - [ ] stop-writes-on-bgsave-error
    - [ ] rdbcompression
    - [ ] rdbchecksum
    - [ ] dbfilename
    - [ ] dir
    - [ ] slaveof
    - [ ] masterauth
    - [ ] slave-serve-stale-data
    - [ ] slave-read-only
    - [ ] repl-diskless-sync
    - [ ] repl-diskless-sync-delay
    - [ ] repl-ping-slave-period
    - [ ] repl-timeout
    - [ ] repl-disable-tcp-nodelay
    - [ ] repl-backlog-size
    - [ ] repl-backlog-ttl
    - [ ] slave-priority
    - [ ] min-slaves-to-write
    - [ ] min-slaves-max-lag
    - [ ] requirepass
    - [ ] rename-command
    - [ ] maxclients
    - [ ] maxmemory
    - [ ] maxmemory-policy
        - [ ] volatile-lru
        - [ ] allkeys-lru
        - [ ] volatile-random
        - [ ] allkeys-random
        - [ ] volatile-ttl
        - [ ] noeviction
    - [ ] maxmemory-samples
    - [ ] appendonly
    - [ ] appendfilename
    - [ ] appendfsync
        - [ ] always
        - [ ] everysec
        - [ ] no
    - [ ] no-appendfsync-on-rewrite
    - [ ] auto-aof-rewrite-percentage
    - [ ] auto-aof-rewrite-min-size
    - [ ] aof-load-truncated
    - [ ] lua-time-limit
    - [ ] slowlog-log-slower-than
    - [ ] slowlog-max-len
    - [ ] latency-monitor-threshold
    - [ ] notify-keyspace-events
        - [ ] K     Keyspace events, published with __keyspace@<db>__ prefix.
        - [ ] E     Keyevent events, published with __keyevent@<db>__ prefix.
        - [ ] g     Generic commands (non-type specific) like DEL, EXPIRE, RENAME, ...
        - [ ] $     String commands
        - [ ] l     List commands
        - [ ]  s     Set commands
        - [ ] h     Hash commands
        - [ ] z     Sorted set commands
        - [ ] x     Expired events (events generated every time a key expires)
        - [ ]  e     Evicted events (events generated when a key is evicted for maxmemory)
        - [ ]  A     Alias for g$lshzxe, so that the "AKE" string means all the events.
    - [ ] hash-max-ziplist-entries
    - [ ] hash-max-ziplist-value
    - [ ] list-max-ziplist-entries
    - [ ] list-max-ziplist-value
    - [ ] set-max-intset-entries
    - [ ] zset-max-ziplist-entries
    - [ ] zset-max-ziplist-value
    - [ ] hll-sparse-max-bytes
    - [ ] activerehashing
    - [ ] client-output-buffer-limit
    - [ ] hz
    - [ ] aof-rewrite-incremental-fsync