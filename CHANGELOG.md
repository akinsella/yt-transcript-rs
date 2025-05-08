# Changelog

All notable changes to the `yt-transcript-rs` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.3] - 2025-05-08

### Added
- New `fetch_streaming_data` method to retrieve video and audio stream information
- `StreamingData` and `StreamingFormat` models for representing stream data
- `StreamingDataExtractor` for parsing stream information from YouTube responses
- Support for retrieving various format details:
  - Video resolutions, codecs, and bitrates
  - Audio quality, sample rates, and channels
  - Color profile information
  - Initialization and index ranges
- Comprehensive test coverage for streaming data functionality
- New example `youtube_streaming_data.rs` demonstrating how to use the feature

### Documentation
- Updated main README.md with streaming data feature information
- Added streaming data example to examples README.md
- Added detailed documentation for all new streaming data classes and methods
- Improved existing API reference documentation

### Changed
- Enhanced error handling for video data extraction
- Improved JSON parsing robustness
- Updated all related module exports in lib.rs

## [0.1.2] - 2025-05-08

### Added
- New `fetch_microformat` method to retrieve additional video metadata
- `MicroformatData`, `MicroformatEmbed`, and `MicroformatThumbnail` models
- `MicroformatExtractor` for parsing microformat information from YouTube responses
- Support for retrieving extended video metadata:
  - Available countries for video playback
  - Video category
  - Embed information and iframe URLs
  - Upload and publish dates
  - Video status flags (unlisted, family-safe, shorts-eligible)
  - Like count and view count
- Comprehensive test coverage for microformat extraction
- New example `youtube_microformat_data.rs` demonstrating how to use microformat features

### Documentation
- Updated main README.md with microformat feature information
- Added microformat example to examples README.md
- Added detailed documentation for all new microformat classes and methods

### Changed
- Enhanced error handling for video metadata extraction
- Improved JSON parsing of nested YouTube data structures

## [0.1.1] - 2025-05-07

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

[0.1.3]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.3
[0.1.2]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.1
