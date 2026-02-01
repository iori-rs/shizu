# shizu

A high-performance HLS proxy server with powerful stream transformation capabilities. Built with Rust for speed and reliability.

## Features

- **Stream Transformation** - Rule-based M3U8 playlist rewriting with extensible transform pipeline
- **Manifest Proxying** - Fetches and transforms HLS manifests on-the-fly
- **Segment Proxying** - Proxies media segments with optional processing
- **Init Segment Caching** - LRU cache for fMP4 initialization segments
- **Header Forwarding** - Preserves custom headers for authenticated streams
- **Byte Range Support** - Handles partial segment requests
- **Configurable CORS** - Works seamlessly with web-based players
- **Structured Logging** - Iceberg integration for analytics and monitoring

## Installation

```bash
cargo install --path .
```

## Usage

Start the server:

```bash
# Default: listens on 0.0.0.0:8080
shizu

# Custom port
PORT=3000 shizu
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `8080` | Bind port |
| `CORS_ALLOWED_ORIGIN` | `*` | CORS origin header |

### Endpoints

#### `GET /manifest`

Fetches and transforms an HLS playlist, rewriting URLs to proxy through shizu.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `url` | Yes | Original manifest URL |
| `h` | No | Base64-encoded headers for manifest requests |
| `sh` | No | Base64-encoded headers for segment requests |
| `k` | No | Processing key(s) in `kid:key` or `key` format |
| `decrypt` | No | Enable segment processing (`true`/`false`) |

#### `GET /segment`

Fetches and processes a media segment.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `url` | Yes | Original segment URL |
| `m` | Yes | Processing method: `ssa`, `ssa-ctr`, or `cenc` |
| `k` | Yes | Processing key(s) |
| `iv` | No | Initialization vector (hex, with optional `0x` prefix) |
| `h` | No | Base64-encoded request headers |
| `br` | No | Byte range (`length@offset`) |
| `f` | No | Format hint: `ts`, `aac`, or `mp4` |
| `init` | No | Init segment URL (for fMP4) |
| `init_br` | No | Init segment byte range |

#### `GET /health`

Health check endpoint. Returns `{"status": "ok", "version": "..."}`.

### Example

Proxy an HLS stream:

```bash
# Get the transformed manifest
curl "http://localhost:8080/manifest?url=https://example.com/master.m3u8"
```

Point your HLS player at the manifest URL and it will automatically fetch segments through the proxy.

## Architecture

```
src/
├── cache/          # Init segment LRU cache
├── decrypt/        # Segment processing
├── hls/            # HLS type definitions
├── logging/        # Iceberg logging
├── proxy/          # HTTP client & header encoding
├── server/         # Axum handlers & routing
└── stream/         # Playlist processing & transformation
```

## Stream Transformation

shizu uses a rule-based transformation pipeline for processing M3U8 playlists:

- **Line Classification** - Parses each line to identify tags, URIs, and comments
- **State Tracking** - Maintains playlist context (media sequence, current key, map info)
- **Transform Rules** - Applies matching rules to rewrite content:
  - `KeyRewriteRule` - Handles `#EXT-X-KEY` tags
  - `MapRewriteRule` - Handles `#EXT-X-MAP` tags  
  - `VariantProxyRule` - Rewrites variant stream URLs
  - `MediaProxyRule` - Rewrites `#EXT-X-MEDIA` URIs
  - `SegmentProxyRule` - Rewrites segment URLs

The transformation pipeline is extensible - implement the `TransformRule` trait to add custom rules.

## License

MIT
