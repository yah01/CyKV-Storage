# CyKV-Storage

## Cache Layer
The engine doesn't read and write files directly when handling the `get`, `set` and `remove` requests, it reads and writes with an inner cache. You can disable the feature by creating a engine with `NoCacheManager`.

### Policy
The cache policy is scalable, but there are some basic principles for the engine:
- `read` causes the policy to determine to evict or not
- `write` forces the cache to sync contents to disk

## Todo
The stages:
- in-plan
- developing
- nightly: available, but hasn't been tested yet
- stable

|features|stage|comment|
|:---:|:---:|---|
|re-open|stable||
|compaction|stable||
|server|nightly||
|cache|nightly|there are still some issues, and bad performance for writing|
|ACID transaction| in-plan||
|efficient replay| nightly | store the keydir items which not in the writing log, and replay only the writing log|
|efficient scan()| nightly | the compaction procedure writes logs lexicographically |