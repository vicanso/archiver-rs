# archiver-rs

Archive directory as follows:

- Compress each file using selected compression: gz, zst, br, lz4, sz, zip
- Archive all compressed file using tar

## Command use

Archive files using gz:

```bash
archiver ~/tmp/fonts ~/tmp/fonts.gz.tar
```

Archive files using zst:

````bash
archiver ~/tmp/fonts ~/tmp/fonts.zst.tar
```

List files from archive file:

```bash
archiver ~/tmp/fonts.gz.tar
````
