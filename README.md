# CyKV-Storage

## Cache Layer
The engine doesn't read and write files directly when handling the `get`, `set` and `remove` requests, it reads and writes with an inner cache. You can disable the feature by creating a engine with `NoCacheManager`.

### Policy
The cache policy is scalable, but there are some basic principles for the engine:
- `get` causes the policy to determine to evict or not
- `set` forces the cache to sync contents to disk
- `remove` makes the cache dirty, but not sync contents to disk
The main idea is: the `set` should guarantee the durability, so it must sync to disk, but the lose of `remove` records just keep dead data, which would be overwritten.

## Todo
The stages:
- in-plan
- developing
- nightly
- stable

|features|stage|
|---|---|
|re-open|stable|
|compaction|stable|
|server|developing|
|cache|developing|
|ACID transaction| in-plan|