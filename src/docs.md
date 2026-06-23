## Capabilities

This service can be used to:

- [ ] create_dir
- [x] stat
- [x] read
- [ ] write
- [ ] delete
- [x] list
- [x] copy
- [ ] rename
- [ ] ~~presign~~

## Configuration

- `root`: Set the work dir for backend.
- 
You can refer to [`FsBuilder`]'s docs for more information

## Example

### Via Builder


```rust,ignore
use std::sync::Arc;

use anyhow::Result;
use opendal::services::Fs;
use opendal::Operator;

#[tokio::main]
async fn main() -> Result<()> {
    // Create fs backend builder.
    let mut builder = Fs::default()
        // Set the root for fs, all operations will happen under this root.
        //
        // NOTE: the root must be absolute path.
        .root("/tmp");

    let op: Operator = Operator::new(builder)?;

    Ok(())
}
```
