# Changelog

All notable changes to the `yt-transcript-rs` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0] - 2025-XX-XX

### Added
- Initial release of `yt-transcript-rs` library
- Core functionality for fetching YouTube video transcripts
- Support for handling various transcript formats and languages
- Comprehensive error handling for different YouTube error scenarios
- Support for age-restricted videos detection
- Support for unavailable videos detection
- HTML tag cleaning in transcript text

### Documentation
- Added detailed README.md with:
  - Table of contents
  - Features section
  - Badges (Crates.io, Documentation, MIT License)
  - Improved installation instructions
  - Better organization with clear sections
- Comprehensive code documentation:
  - Struct-level documentation explaining purpose and architecture
  - Detailed method documentation with parameters, return values, and error information
  - Usage examples for key components
  - Explanation of internal workings and data flow

### Changed
- Updated all dependencies to their latest versions
- Simplified HTML tag handling logic in transcript parser
- Modified XML parsing approach for better compatibility
- Restructured code for better maintainability

### Fixed
- Compatibility issues with newer regex library version
- Replaced complex look-ahead regex patterns
- Fixed various warnings and unused imports/variables

[0.1.0]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.0
