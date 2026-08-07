## [0.2.0] - 2026-08-07

### 🚀 Features

- Working serialization
- Added pathlib and datetime support
- Added unpack hooks to allow classes to customize behavior

### 🐛 Bug Fixes

- Fixed ser/de of enums
- Added to serde value
- Fixed logic for rust defined import paths
- Added module path to pyclass

### 🚜 Refactor

- Moved data to be PyAny to be more flexible
## [0.1.1] - 2026-07-28

### 🐛 Bug Fixes

- Exposed py import config to rust prelude

### ⚙️ Miscellaneous Tasks

- *(release)* Prep for release v0.1.1
## [0.1.0] - 2026-07-28

### 🚀 Features

- Initial commit
- Added stub gen binary
- Added entrypoint group to python functions
- Added config object to control deserialization behavior
- Added recursive parsing of import dicts
- Allowing objects to set a custom group
- Added example

### 🚜 Refactor

- Renamed rust lib to py-unpack

### 📚 Documentation

- Minor docstring updates
- Updated readme
- Minor readme changes

### ⚙️ Miscellaneous Tasks

- Renamed project to unpack
- *(release)* Prep for release v0.1.0
