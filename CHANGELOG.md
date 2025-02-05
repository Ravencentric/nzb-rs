# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
