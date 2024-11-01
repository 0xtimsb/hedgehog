# hedgehog

rust-based download manager and torrent client

note: work in progress

## todo

- [x] add download using url
  - [x] add content type validation
  - [x] debounce url validation check
  - [x] extract debounced input from url input as a separate component
  - [x] add loading while url is validating
  - [x] cancel previous validation request if user starts typing again
  - [x] use url from clipboard
- [ ] actually download files
  - [ ] start download on addition
  - [ ] add progress bar
  - [ ] add cancel button
  - [ ] add pause/resume button
  - [ ] add remove button
