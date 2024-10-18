# teleton

## Needed environment variables

* `TELETON_SESSION_PATH`: stores session data (required)
* `TELETON_API_ID` (required)
* `TELETON_API_HASH` (required)
* `TELETON_PROXY`: you can use SOCKS5 proxy for upstream connection if you want (optional)

## API Usage

See `openapi.yml` for reference (maybe incomplete), or `upload_file.js` for Node.js example

## Build Instruction

* `python3 download_and_patch.py` (resolves some dependencies with patch)
* `cargo build --release`

## TODO

* [ ] Handles file reference change
* [ ] Authentication
