# hedgehog

rust-based download manager and torrent client

note: work in progress

## todo

- [ ] add download using url
  - [x] add content type validation
  - [x] debounce url validation check
  - [x] extract debounced input from url input as a separate component
  - [x] add loading while url is validating
  - [x] cancel previous validation request if user starts typing again
  - [ ] use url from clipboard
