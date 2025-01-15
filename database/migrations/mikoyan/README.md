# Mikoyan

## Testing

### `mikoyan --bulk-write --batch-size 500`

Docs: 331_000

Docs / s: 1047

```bash
        User time (seconds): 182.48
        System time (seconds): 11.28
        Percent of CPU this job got: 61%
        Elapsed (wall clock) time (h:mm:ss or m:ss): 5:16.71
        Average shared text size (kbytes): 0
        Average unshared data size (kbytes): 0
        Average stack size (kbytes): 0
        Average total size (kbytes): 0
        Maximum resident set size (kbytes): 446568
        Average resident set size (kbytes): 0
        Major (requiring I/O) page faults: 1
        Minor (reclaiming a frame) page faults: 132397
        Voluntary context switches: 178218
        Involuntary context switches: 27747
        Swaps: 0
        File system inputs: 0
        File system outputs: 0
        Socket messages sent: 0
        Socket messages received: 0
        Signals delivered: 0
        Page size (bytes): 4096
```

### `mikoyan --batch-size 1000`

Docs: 101_391

Docs / s: 320

```bash
        User time (seconds): 63.96
        System time (seconds): 30.78
        Percent of CPU this job got: 29%
        Elapsed (wall clock) time (h:mm:ss or m:ss): 5:16.26
        Average shared text size (kbytes): 0
        Average unshared data size (kbytes): 0
        Average stack size (kbytes): 0
        Average total size (kbytes): 0
        Maximum resident set size (kbytes): 56900
        Average resident set size (kbytes): 0
        Major (requiring I/O) page faults: 0
        Minor (reclaiming a frame) page faults: 17715
        Voluntary context switches: 1593977
        Involuntary context switches: 229921
        Swaps: 0
        File system inputs: 0
        File system outputs: 0
        Socket messages sent: 0
        Socket messages received: 0
        Signals delivered: 0
        Page size (bytes): 4096
        Exit status: 0
```

### `mikoyan --bulk-write --batch-size 1000`

Docs: 319_000

Docs / s: ~1000

```bash

```

### `mikoyan --bulk-write --batch-size 1500`

Docs: 501_000

Docs / s: ~1000

```bash
        User time (seconds): 281.87
        System time (seconds): 20.88
        Percent of CPU this job got: 57%
        Elapsed (wall clock) time (h:mm:ss or m:ss): 8:20.28
        Average shared text size (kbytes): 0
        Average unshared data size (kbytes): 0
        Average stack size (kbytes): 0
        Average total size (kbytes): 0
        Maximum resident set size (kbytes): 5169172
        Average resident set size (kbytes): 0
        Major (requiring I/O) page faults: 1
        Minor (reclaiming a frame) page faults: 1661174
        Voluntary context switches: 128608
        Involuntary context switches: 31292
        Swaps: 0
        File system inputs: 256
        File system outputs: 0
        Socket messages sent: 0
        Socket messages received: 0
        Signals delivered: 0
        Page size (bytes): 4096
```

### `mikoyan --bulk-write --batch-size 50`

```bash
Users Processed: 15000 / 1499500 (1363 records/s)
Users Processed: 30000 / 1499500 (1304 records/s)
Users Processed: 45000 / 1499500 (1153 records/s)
Users Processed: 60000 / 1499500 (1052 records/s)
Users Processed: 75000 / 1499500 (1013 records/s)
```

We have decided to not normalise the database to the point of one schema. Instead, all we **need** is all schemas to not have ambiguous data types.

- Fields posed as `[]?` should be normalised to `[]`
- Fields with similar structure but multiple data types should be normalised to one data type. E.g. `[String | Number]` -> `[String]`

## Actual Schema

[current-schema.json](./current-schema.json)

## Property Transformations

### Removal

- `badges`
- `isGithub`
- `isLinkedIn`
- `isTwitter`
- `isWebsite`
- `password`
- `timezone`
- `sound`
- `completedChallenges.$[el].files.$[el].history`

Nonesense created by LB:

- `completedChallenges.$[el].__cachedRelations`
- `completedChallenges.$[el].__data`
- `completedChallenges.$[el].__dataSource`
- `completedChallenges.$[el].__persisted`
- `completedChallenges.$[el].__strict`
- `profileUI.__cachedRelations`
- `profileUI.__data`
- `profileUI.__dataSource`
- `profileUI.__persisted`
- `profileUI.__strict`

Note: `rand` was planned to be removed, but, until the migration, new records will continue to be created with the field.

### Transposition

- `savedChallenges`
  - `Undefined` -> `[]`
- `partiallyCompletedChallenges`
  - `Undefined` -> `[]`
- `completedChallenges`
  - `[{ files: Undefined | Null }]` -> `[{ files: [] }]`
- `progressTimestamps`
  - `[{ timestamp: Double | Int32 | Int64 | String | Timestamp }]` -> `[i64]`
  - `[Null | Undefined]` -> `[]`
  - `[Int64 | Int32 | String]` -> `[i64]`
- `yearsTopContributor`
  - `Undefined` -> `[]`
  - `[String | Int32 | Int64]` -> `[Int32]`
    - A string of length 4 takes 24 bytes, an `i32` takes 4 bytes
- `profileUI`
  - `Undefined` -> `ProfileUI::default()`
  - Any `undefined` field -> `false` value
- `email`
  - Lowercase all email addresses

## Usage

### Testing

- Download sample dataset from MongoDB into `sample-users.json`:

```sh
db.getCollection("user").aggregate(
  [{ $sample: { size: 100 } }]
);
```

**Note**: Must be <5% of collection size

- Import into database

```sh
mongoimport --db=freeCodeCamp --collection=user --file=sample-users.json
```

- Run the migration (see below for more options)

```sh
mikoyan
```

- Confirm nothing went wrong in logs file
- Download normalized `user` collection to `normalized-users.json`
- Run differ to see changes:

```sh
differ/$ npm run comp
differ/$ npm run diff
differ/$ less diff.txt
```

### Use the Bin

```bash
mikoyan --url <url> --bulk-write --batch-size <batch-size> --logs <logs-path>
```

Example:

```bash
mikoyan --release -- --batch-size 1000
```

More info:

```bash
mikoyan --help
```

### Testing Strategy
