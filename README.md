### The implementation

The api exposes two endpoints - `GET /files/{fileid}?key={key}` and `POST /files/`
Having the key in the path is not perfect, but since the files are deleted upon downloading, this is not a problem.

The GET endpoint downloads the file if it exists and if the key is correct, otherwise it returns a 404 to not leak information. The file is loaded into the memory, decoded, and then sent.

The POST endpoint returns a one time key and a file ID upon a successful upload. The files are encrypted in-memory, and then saved in the location from the application's config, with an encrypted filename and contents.

I've chosen to limit the files to be < 100mb, as I wanted them to fit in memory for easier implementation. The files are uploaded, stored in memory, encrypted, and then written to disk, to avoid storage in their decrypted form.

There's also a possibility to use env vars to change the default file save location, and the max file size.

### Choices

I've chosen actix, as I haven't used it before and it seemed interesting, and more suitable to the task of iterating over raw bytes than using a Python API library.

There's also some caveats that I've ignored due to time constraints:
- The same filename could be generated twice for different files with a very low probability, but the program doesn't check for it
- The key should preferably be sent in a HTTP header, but it is not.
- The file is limited to be in-memory, so I've limited the filesize to 100MB. Nevertheless, it still should be possible to overload the server with too many concurrent requests due to memory limitations.
- The error codes for uploading could be more descriptive, rather than a generic failure
- I've abused Rust's error propagation for ease of quick prototyping, but pretty much all signatures in the code should be `Result<T,Error>` instead of `Result<T,()>`
- The files are not deleted over time, this should be doable with a simple cron job or a service in the program.

### Other options
- AEAD ciphers can be used in stream mode, so there's no need to hold the entire file in memory to avoid writing it unencrypted to disk, but they're a bit harder to implement
- A user system seems more natural for this use case, and it would allow for multiple file uploads using the same key. For a nonfree api, it would be nice to have
- I'm pretty sure it would just be possible to use https and store undecoded data until a download request arrives, but I don't know if it would be a good idea.

### Possible improvements
Aside from implementing the things listed above, these would be nice to have:
- a basic web interface
- setting a lifespan of the file during upload  (# of downloads, days till removal)
- proper logging

### Addressing bonus points
- A proper db would be overkill, as the filesystem is good at handling single access file read/writes and storing files with their filenames.
- I wanted to just generate a client with OpenAPI, but realized halfway through that actix doesn't generate that for free.

I ran out of (personal) time for unit tests, but the functionality should be easily testable by running
```
curl -F "file=@Cargo.toml" localhost:8080/files
```
For file upload, and then `ls files` (or the same in the container) for checking their presence.
Followed by running (with substitution using keys from the previous command):
```
curl "localhost:8080/files/{filename}?key={key}" > a
```

and then `diff a Cargo.toml`. Note that the original filename is not lost, but I don't know how to get it from curl programatically.

### Running the code
Build the Dockerfile `docker build .` and launch the image with `docker run -p 8080:8080 imageID`, or `cargo run -r` in the directory.

