# Changelog

All notable changes to the `yt-transcript-rs` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.7] - 2025-05-16

### Added
- Added new example `youtube_offline_extract_infos.rs` for extracting video information from saved HTML files
- Added new example `youtube_proxy_cli.rs` demonstrating proxy configuration via CLI and environment variables
- Added new dev dependencies:
  - `clap` for CLI argument parsing
  - `dotenvy` for environment variable loading
  - `tracing-subscriber` for logging

### Changed
- Simplified proxy handling by removing proxy configuration from core components:
  - Removed proxy configuration from `VideoDataFetcher`
  - Removed proxy configuration from `YoutubePageFetcher`
  - Moved proxy handling to example implementations
- Updated example `youtube_transcript_basic.rs` with new video ID
- Enhanced documentation in example files with detailed comments and usage instructions

### Fixed
- Fixed proxy configuration handling to be more explicit and type-safe

## [0.1.6] - 2025-05-11

### Added
- Improved HTML to text conversion with the Scraper library
- Added configurable link formatting with customizable templates
- Added `with_config` method to TranscriptParser for link format customization
- Added `tempfile` crate as development dependency for cookie jar loader tests

### Changed
- Replaced manual character-by-character HTML parsing with Scraper-based approach
- Enhanced HTML entity handling for better accuracy
- Improved whitespace preservation in transcript text
- Simplified code structure for better maintainability
- Optimized link format processing for better readability

### Fixed
- Fixed spacing issues around HTML tags
- Fixed incorrect HTML entity decoding (particularly for apostrophes and quotes)
- Fixed inconsistent whitespace handling in parsed transcripts
- Fixed cookie jar loader tests to remove dependency on reqwest's internal cookie handling
- Updated reqwest dependency to explicitly include the cookies feature
- Simplified cookie jar tests to focus on core functionality without testing reqwest internals
 
## [0.1.5] - 2025-05-10

### Added
- Full serialization/deserialization support for all data structures
- Added `serde` derive macros to all relevant models
- New convenience method `translate_and_fetch` to translate and fetch in one operation
- Enhanced `build_without_client` method in `TranscriptList` for serialization support
- Updated example `youtube_video_infos.rs` to demonstrate serialization features

### Changed
- Externalized HTTP client from `Transcript` struct to make it serializable
- Modified `fetch` method to accept a client parameter instead of storing it
- Updated `translate` method to preserve translation languages
- Improved error handling with detailed error variants for translation
- Removed redundant client parameter from `TranscriptList::build`
- Cleaned up and simplified internals for better maintainability

### Documentation
- Updated main README.md with serialization/deserialization example
- Added section about serialization in examples README.md
- Improved method documentation to reflect changes in client handling

## [0.1.4] - 2025-05-09

### Added
- New `fetch_video_infos` method to retrieve all video information in a single request
- `VideoInfos` model for representing comprehensive video data
- Helper method in `VideoDataFetcher` to fetch player response data with reduced code duplication
- Support for optimized data retrieval combining:
  - Video details (title, author, etc.)
  - Microformat data (category, available countries, etc.)
  - Streaming data (available formats, qualities, etc.)
  - Transcript list (available captions)
- New example `youtube_video_infos.rs` demonstrating the all-in-one retrieval approach

### Documentation
- Updated main README.md with all-in-one data retrieval information
- Added fetch_video_infos example to examples README.md
- Added detailed documentation for the new VideoInfos struct and related methods

### Changed
- Refactored common code in VideoDataFetcher to eliminate duplication
- Improved error handling for combined data retrieval
- Enhanced code structure for better maintainability

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

[0.1.7]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.7
[0.1.6]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.6
[0.1.5]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.5
[0.1.4]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.4
[0.1.3]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.3
[0.1.2]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/akinsella/yt-transcript-rs/releases/tag/v0.1.1
