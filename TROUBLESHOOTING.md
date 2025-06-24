# Troubleshooting Guide

This guide helps you resolve common issues when using `yt-transcript-rs`.

## Issue: Empty Transcripts (0 snippets) - COMPLETELY RESOLVED in v0.1.8+

### ✅ Issue Permanently Fixed

**If you're using v0.1.8 or later**: This issue is completely resolved! The library now uses YouTube's internal InnerTube API exclusively, which completely bypasses the broken external transcript API.

**Simply upgrade to the latest version**:
```toml
[dependencies]
yt-transcript-rs = "0.1.8"
```

**No additional configuration needed** - the library now works reliably out of the box for all public videos with transcripts.

### What Changed

YouTube permanently changed their external transcript API, making the old approach completely non-functional. Version 0.1.8+ completely replaces the broken method with YouTube's internal InnerTube API, which:

- ✅ Works reliably for all public videos
- ✅ Requires no authentication or special configuration
- ✅ Bypasses all the previous API limitations
- ✅ Is future-proof against external API changes

### Migration Guide

If you're upgrading from a pre-v0.1.8 version:

1. **Update your dependency**:
   ```toml
   [dependencies]
   yt-transcript-rs = "0.1.8"
   ```

2. **Remove any workarounds**: You can remove any special configuration like:
   - Cookie authentication (unless needed for private videos)
   - Proxy configurations (unless needed for other reasons)
   - Custom headers or user agents
   - Session establishment code

3. **Your existing code will work unchanged** - no code modifications required.

### Legacy Information (Historical)

*This section is kept for historical reference only. These solutions are no longer needed with v0.1.8+.*

The empty transcript issue affected versions prior to v0.1.8 when YouTube changed their external API. The previous solutions involved complex workarounds like cookie authentication, proxy usage, and custom headers, but these are no longer necessary.

## Other Common Issues

### Issue: "No transcript found"

**Cause**: The video doesn't have transcripts in the requested language.

**Solution**: 
- Check available languages with `list_transcripts()`
- Try auto-generated transcripts
- Use translation if available

### Issue: "Video unavailable"

**Cause**: The video is private, deleted, or region-restricted.

**Solution**:
- Verify the video ID is correct
- Check if the video is accessible in your region
- Try a different video

### Issue: "Rate limiting"

**Cause**: Too many requests in a short time.

**Solution**:
- Add delays between requests
- Use proxies to distribute requests
- Implement exponential backoff

## Getting Help

If none of these solutions work:

1. **Check the GitHub Issues**: https://github.com/akinsella/yt-transcript-rs/issues
2. **Create a new issue** with:
   - Your code example
   - Error messages
   - Video IDs you're testing
   - Your region/country
3. **Include debug output** from the debug examples

## Contributing

If you find a working solution, please contribute back:
- Submit a pull request with improvements
- Share working configurations in GitHub issues
- Help update this troubleshooting guide 