# shizu

A high-performance HLS proxy server that transparently decrypts DRM-protected streams. Built with Rust for speed and reliability.

## Features

- **DRM Decryption** - Supports SAMPLE-AES (MPEG-TS/AAC), SAMPLE-AES-CTR, and CENC (fMP4)
- **Manifest Transformation** - Rewrites HLS playlists on-the-fly to proxy through the server
- **Init Segment Caching** - LRU cache for fMP4 initialization segments
- **Header Proxying** - Preserves custom headers for authenticated streams
- **Byte Range Support** - Handles partial segment requests
- **Configurable CORS** - Works with web-based players
- **Structured Logging** - Iceberg integration for analytics

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

# With external host for generated URLs
EXTERNAL_HOST=my-proxy.example.com EXTERNAL_SCHEME=https shizu
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `8080` | Bind port |
| `EXTERNAL_HOST` | `localhost` | Hostname used in generated URLs |
| `EXTERNAL_SCHEME` | `http` | Scheme used in generated URLs |
| `CORS_ALLOWED_ORIGIN` | `*` | CORS origin header |

### Endpoints

#### `GET /manifest`

Fetches and transforms an HLS playlist, rewriting URLs to proxy through shizu.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `url` | Yes | Original manifest URL |
| `h` | No | Base64-encoded headers for manifest requests |
| `sh` | No | Base64-encoded headers for segment requests |
| `k` | No | Decryption key(s) in `kid:key` or `key` format |
| `decrypt` | No | Enable decryption (`true`/`false`) |

#### `GET /segment`

Fetches, decrypts, and returns a media segment.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `url` | Yes | Original segment URL |
| `m` | Yes | Decryption method: `ssa`, `ssa-ctr`, or `cenc` |
| `k` | Yes | Decryption key(s) |
| `iv` | No | Initialization vector (hex, with optional `0x` prefix) |
| `h` | No | Base64-encoded request headers |
| `br` | No | Byte range (`length@offset`) |
| `f` | No | Format hint: `ts`, `aac`, or `mp4` |
| `init` | No | Init segment URL (for fMP4) |
| `init_br` | No | Init segment byte range |

#### `GET /health`

Health check endpoint. Returns `{"status": "ok", "version": "..."}`.

### Example

Proxy a DRM-protected stream:

```bash
# Get the transformed manifest
curl "http://localhost:8080/manifest?url=https://example.com/master.m3u8&k=abcd1234:deadbeef01234567890abcdef0123456&decrypt=true"
```

Point your HLS player at the manifest URL and it will automatically fetch decrypted segments through the proxy.

## Architecture

```
src/
├── cache/          # Init segment LRU cache
├── decrypt/        # Decryption (iori-ssa, mp4decrypt)
├── hls/            # HLS type definitions
├── logging/        # Iceberg logging
├── proxy/          # HTTP client & header encoding
├── server/         # Axum handlers & routing
└── stream/         # Playlist processing & transformation
```

## Decryption Methods

| Method | Use Case | Library |
|--------|----------|---------|
| `ssa` | SAMPLE-AES for MPEG-TS/AAC | iori-ssa |
| `ssa-ctr` | SAMPLE-AES-CTR for fMP4 | mp4decrypt |
| `cenc` | Common Encryption for fMP4 | mp4decrypt |

## License

MIT
