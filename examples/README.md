# YouTube Transcript API Examples

This directory contains several example applications demonstrating the features of the YouTube Transcript API implementation in Rust.

## Requirements

- Rust and Cargo installed
- Internet connection to access YouTube API

## Running the Examples

You can run any of the examples using Cargo by specifying the example name:

```bash
cargo run --example youtube_transcript_basic
```

## Available Examples

### Basic Transcript Fetching

```bash
cargo run --example youtube_transcript_basic
```

This example demonstrates how to fetch a transcript for a YouTube video. It shows the basic setup of the API and how to fetch a transcript in a specific language.

### Listing Available Transcripts

```bash
cargo run --example youtube_transcript_list
```

This example shows how to list all available transcripts for a YouTube video, including information about which transcripts are translatable and what languages are available for translation.

### Translating Transcripts

```bash
cargo run --example youtube_transcript_translation
```

This example demonstrates how to translate a transcript from one language to another, using YouTube's translation capabilities.

### Using Proxies

```bash
cargo run --example youtube_transcript_proxy
```

This example demonstrates how to use proxies with the YouTube Transcript API. It supports:

- GenericProxyConfig for standard HTTP/HTTPS proxies
- WebshareProxyConfig for Webshare proxy service

To use this example with proxies, set the following environment variables:

```bash
# For HTTP/HTTPS proxy
export HTTP_PROXY="http://your-proxy-url:port"
export HTTPS_PROXY="https://your-proxy-url:port"

# For Webshare proxy
export WEBSHARE_USER="your-username"
export WEBSHARE_PASSWORD="your-password"
```

### Fetching Multiple Transcripts

```bash
cargo run --example youtube_transcript_multiple
```

This example shows how to fetch transcripts for multiple YouTube videos in a single call, and how to handle both successful and failed transcript fetches.

### Cookie Consent Handling

```bash
cargo run --example youtube_transcript_cookie_consent
```

This example demonstrates the API's automatic handling of cookie consent requirements. When accessing YouTube from regions with strict data protection laws (like the EU), YouTube may present a cookie consent page. The API automatically handles this by:

1. Detecting the cookie consent form
2. Extracting the necessary consent token
3. Setting the consent cookie
4. Retrying the request with the consent cookie

The example shows this process and provides error handling guidance if the automatic consent handling fails.

### Cookie Authentication

```bash
cargo run --example youtube_transcript_cookie_auth
```

This example demonstrates how to use cookie authentication with the YouTube Transcript API. Cookie authentication is useful for:

1. Accessing age-restricted videos without providing login credentials
2. Bypassing region restrictions
3. Working with videos that require a logged-in user
4. Having consent cookies already set (for EU regions)

To use this example, set the `YOUTUBE_COOKIE_FILE` environment variable to point to a Netscape-format cookies.txt file:

```bash
export YOUTUBE_COOKIE_FILE=/path/to/your/cookies.txt
```

You can export cookies from your browser using extensions like:
- "Get cookies.txt" (Chrome/Firefox)
- "Cookie-Editor" (Chrome/Firefox)
- "EditThisCookie" (Chrome)

### Advanced Usage (All Features)

```bash
cargo run --example youtube_transcript_advanced
```

This advanced example demonstrates combining multiple features of the YouTube Transcript API:

1. Cookie authentication for age-restricted videos
2. Proxy configuration for bypassing IP blocks
3. Consent cookie handling for EU regions
4. Multiple transcript formats (JSON, SRT, plain text)
5. Detailed error handling for all scenarios

You can customize this example by setting environment variables:

```bash
# Path to cookie file for authentication
export YOUTUBE_COOKIE_FILE=/path/to/your/cookies.txt

# Proxy settings
export HTTP_PROXY=http://your-proxy-url:port
export HTTPS_PROXY=https://your-proxy-url:port

# Custom video ID (optional)
export VIDEO_ID=your_video_id
```

This example is ideal for understanding how the different features of the API work together.

### Fetching Video Details

```bash
cargo run --example youtube_video_details
```

This example demonstrates how to fetch detailed information about a YouTube video using the API. It retrieves:

1. Basic information (title, author, video ID)
2. Statistics (view count, length in seconds)
3. Content information (keywords, description) 
4. Thumbnails (URLs and dimensions)

The video details functionality allows you to extract rich metadata from YouTube videos without needing to parse the webpage manually. This can be useful for building applications that need to display video information alongside transcripts.

### Fetching Microformat Data

```bash
cargo run --example youtube_microformat_data
```

This example demonstrates how to fetch microformat data from a YouTube video using the API. Microformat data contains additional metadata that complements the standard video details, including:

1. Available countries where the video can be viewed
2. Category information
3. Embed details (iframe URL and dimensions)
4. Video status flags (family safe, unlisted, shorts eligibility)
5. Publication and upload dates
6. Like count and other engagement metrics

Microformat data is useful for understanding content restrictions, embedding videos, and gathering more comprehensive metadata about YouTube videos.

### Fetching Streaming Data

```bash
cargo run --example youtube_streaming_data
```

This example demonstrates how to fetch streaming data from a YouTube video using the API. The streaming data contains comprehensive information about available video and audio formats:

1. Combined formats (with both video and audio in a single stream)
2. Adaptive formats (separate video and audio streams for dynamic quality adjustment)
3. Various quality options (resolutions from 144p to 1080p+)
4. Format details (codec, bitrate, FPS, file type)
5. Technical specifications (initialization ranges, color information)
6. Audio characteristics (sample rate, channels, quality levels)

This functionality is useful for applications that need to access or analyze YouTube's media streams, understand available quality options, or implement custom video players with advanced format selection capabilities.

### Fetching All Video Information at Once

```bash
cargo run --example youtube_video_infos
```

This example demonstrates how to fetch all information about a YouTube video in a single request using the `fetch_video_infos` function. This approach is more efficient as it avoids multiple network requests when you need different types of information about a video.

The example shows how to access:

1. Video details (title, author, view count, description, etc.)
2. Microformat data (category, available countries, family safe status, etc.)
3. Streaming data (available formats, resolutions, codecs, etc.)
4. Transcript information (available languages, translation options, etc.)

Additionally, the example demonstrates how to:

5. Serialize the complete `VideoInfos` struct to JSON
6. Deserialize it back into a fully functional object
7. Use the deserialized data to access transcripts

This all-in-one approach is ideal for applications that need comprehensive video information with optimal performance. It reduces network overhead and provides a complete picture of a video's metadata in a single operation. The serialization support enables caching video information, sending it between systems, or storing it in databases. 