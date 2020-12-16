# CyKV-Storage

## Cache Layer
The engine doesn't read and write files directly when handling the `get`, `set` and `remove` requests, it reads and writes with an inner cache. You can disable the feature by creating a engine with `NoCacheManager`.

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