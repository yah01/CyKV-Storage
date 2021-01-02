# CyKV-Storage
CyKV is a storage engine inspired by Bitcask, which solves the memory usage issue, and can replay and scan efficiently. The design of CyKV only limit the length of key to 256 bytes (the 65536 bytes can be supported), which is not a problem for most of applications.

~~~
## Cache Layer
The engine doesn't read and write files directly when handling the `get`, `set` and `remove` requests, it reads and writes with an inner cache. You can disable the feature by creating a engine with `NoCacheManager`.

### Policy
The cache policy is scalable, but there are some basic principles for the engine:
- `read` causes the policy to determine to evict or not
- `write` forces the cache to sync contents to disk
~~~
## Todo
The stages:
- in-plan
- developing
- nightly: available, but hasn't been tested yet
- stable
- deprecated

|features|stage|comment|
|:---:|:---:|---|
|re-open|stable||
|compaction|stable||
|server|nightly||
|~~cache~~|deprecated|~~there are still some issues, and bad performance for writing~~|
|ACID transaction| in-plan||
|efficient replay| in-plan| store the keydir items which not in the writing log, and replay only the writing log|
|efficient scan()| in-plan| the compaction procedure writes logs lexicographically |