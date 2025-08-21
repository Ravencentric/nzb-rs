# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/Ravencentric/nzb-rs/compare/v0.5.13...v0.6.0) - 2025-08-21

### Fixed

- use .start() instead of .range()
- refactor stem and extension extractor
- more robust extension and stem parser
- implement a basic "natsort" for Nzb.files
- ensure nzb has atleast one non-par2 file

### Other

- *(deps)* bump taiki-e/install-action in the actions group ([#40](https://github.com/Ravencentric/nzb-rs/pull/40))
- *(deps)* bump the actions group with 2 updates ([#42](https://github.com/Ravencentric/nzb-rs/pull/42))
- enable rust-toolchain updates in dependabot

## [0.5.13](https://github.com/Ravencentric/nzb-rs/compare/v0.5.12...v0.5.13) - 2025-08-18

### Fixed

- replace nested if with the new if let chains
- handle empty string filename
- needless borrow (thanks clippy)
- handle filenames with nested quotes in the subject

### Other

- pin workflows
- bump toolchain and deps
- *(deps)* bump thiserror from 2.0.12 to 2.0.15 in the actions group ([#38](https://github.com/Ravencentric/nzb-rs/pull/38))
- *(deps)* bump serde_json in the actions group ([#37](https://github.com/Ravencentric/nzb-rs/pull/37))
- *(deps)* bump rstest from 0.25.0 to 0.26.1 in the actions group ([#36](https://github.com/Ravencentric/nzb-rs/pull/36))
- *(deps)* bump serde_json in the actions group ([#34](https://github.com/Ravencentric/nzb-rs/pull/34))

## [0.5.12](https://github.com/Ravencentric/nzb-rs/compare/v0.5.11...v0.5.12) - 2025-07-17

### Fixed

- cargo fmt
- exclude par2 files when trying to determine the primary file

## [0.5.11](https://github.com/Ravencentric/nzb-rs/compare/v0.5.10...v0.5.11) - 2025-07-17

### Fixed

- make sanitizer regex less greedy

### Other

- update deps

## [0.5.10](https://github.com/Ravencentric/nzb-rs/compare/v0.5.9...v0.5.10) - 2025-07-14

### Fixed

- extracting filename from subject should no longer strip the release group in some cases

### Other

- *(deps)* bump flate2 from 1.1.1 to 1.1.2 in the actions group ([#30](https://github.com/Ravencentric/nzb-rs/pull/30))

## [0.5.9](https://github.com/Ravencentric/nzb-rs/compare/v0.5.8...v0.5.9) - 2025-06-01

### Other

- update deps and the rust toolchain
- *(deps)* bump chrono from 0.4.40 to 0.4.41 in the actions group ([#28](https://github.com/Ravencentric/nzb-rs/pull/28))

## [0.5.8](https://github.com/Ravencentric/nzb-rs/compare/v0.5.7...v0.5.8) - 2025-04-07

### Other

- *(deps)* bump flate2 from 1.1.0 to 1.1.1 in the actions group ([#26](https://github.com/Ravencentric/nzb-rs/pull/26))

## [0.5.7](https://github.com/Ravencentric/nzb-rs/compare/v0.5.6...v0.5.7) - 2025-03-03

### Other

- *(deps)* update deps
- update to rust 2024
- *(deps)* bump the actions group with 4 updates ([#23](https://github.com/Ravencentric/nzb-rs/pull/23))

## [0.5.6](https://github.com/Ravencentric/nzb-rs/compare/v0.5.5...v0.5.6) - 2025-02-24

### Other

- *(deps)* bump the actions group with 3 updates (#21)

## [0.5.5](https://github.com/Ravencentric/nzb-rs/compare/v0.5.4...v0.5.5) - 2025-02-17

### Fixed

- add another regex for parsing subject

## [0.5.4](https://github.com/Ravencentric/nzb-rs/compare/v0.5.3...v0.5.4) - 2025-02-09

### Fixed

- version
- minor refactor

### Other

- run cargo hack

## [0.5.2](https://github.com/Ravencentric/nzb-rs/compare/v0.5.1...v0.5.2) - 2025-02-09

### Fixed

- error docstring

## [0.5.1](https://github.com/Ravencentric/nzb-rs/compare/v0.5.0...v0.5.1) - 2025-02-08

### Fixed

- readme
- docstring

## [0.5.0](https://github.com/Ravencentric/nzb-rs/compare/v0.4.4...v0.5.0) - 2025-02-08

### Added

- [**breaking**] new errors enums with fine grained members
- support reading from (gzipped) text file

### Fixed

- non_existent_file test
- serde tests
- more tests

### Other

- fmt

## [0.4.4](https://github.com/Ravencentric/nzb-rs/compare/v0.4.3...v0.4.4) - 2025-02-05

### Fixed

- replace pip with cargo (lol)

## [0.4.3](https://github.com/Ravencentric/nzb-rs/compare/v0.4.2...v0.4.3) - 2025-02-05

### Added

- add `Nzb.par2_files`, `Nzb.has_extension`, and `File.has_extension`

### Fixed

- update readme

## [0.4.2](https://github.com/Ravencentric/nzb-rs/compare/v0.4.1...v0.4.2) - 2025-01-29

### Fixed

- sort `File.groups` and `File.segments`

## [0.4.1](https://github.com/Ravencentric/nzb-rs/compare/v0.4.0...v0.4.1) - 2025-01-28

### Fixed

- update version in readme
- sort `Nzb.files`

### Other

- *(deps)* bump serde_json in the actions group (#11)
- add cargo to dependabot

## [0.4.0](https://github.com/Ravencentric/nzb-rs/compare/v0.3.1...v0.4.0) - 2025-01-27

### Fixed

- [**breaking**] rename `File.datetime` to `File.posted_at`

## [0.3.1](https://github.com/Ravencentric/nzb-rs/compare/v0.3.0...v0.3.1) - 2025-01-23

### Other

- fix links and dedupe readme

## [0.3.0](https://github.com/Ravencentric/nzb-rs/compare/v0.2.0...v0.3.0) - 2025-01-23

### Fixed

- refactor lib.rs
- refactor parser.rs
- [**breaking**] rename public API

### Other

- fmt

## [0.2.0](https://github.com/Ravencentric/nzb-rs/compare/v0.1.3...v0.2.0) - 2025-01-20

### Fixed

- tests
- remove NZB.filestems and NZB.extensions

### Other

- update deps
- use cached install
- remove paths
- use pat

## [0.1.3](https://github.com/Ravencentric/nzb-rs/compare/v0.1.2...v0.1.3) - 2025-01-19

### Fixed

- make `InvalidNZBError.message` public (#4)

## [0.1.2](https://github.com/Ravencentric/nzb-rs/compare/v0.1.1...v0.1.2) - 2025-01-19

### Added

- support (de)serialization with serde

### Fixed

- lint
- sort derives
- more docstrings

### Other

- workflow dispatch
- automatic releases
- readme fix
- test serde feature
