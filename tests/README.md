# Paperless integration tests

The tests use the disposable Paperless instance configured in
`docker-compose.paperless-integration.yml`. Connection details and the upload fixture are read from
`config.json`. Set `PAPERLESS_TEST_CONFIG` to use a different JSON configuration.

Run the ordered suite:

```sh
docker compose --file tests/docker-compose.paperless-integration.yml up --detach --wait
cargo test --package paperless-api-integration-tests -- --nocapture
docker compose --file tests/docker-compose.paperless-integration.yml down --volumes
```

The normal suite uploads and removes the fixture document before running the tag lifecycle. Rust
does not guarantee ordering between separate test cases, so this order is expressed inside
`tests::ordered_integration_suite`.

Each step also has an ignored test for focused debugging. These tests are self-contained and obtain
their own client context when run in isolation:

```sh
cargo test --package paperless-api-integration-tests \
  tests::debug_upload_document -- --ignored --exact --nocapture

cargo test --package paperless-api-integration-tests \
  tests::debug_tag_lifecycle -- --ignored --exact --nocapture
```

By default, the Rust setup requests an API token using the username and password in `config.json`
and caches the authenticated client for the test process. To use an existing instance, provide a
`token` in a separate config file; when present, it takes precedence over username and password.

The checked-in `fixtures/demo.pdf` is generated from `fixtures/demo.tex` with:

```sh
pdflatex -interaction=nonstopmode -halt-on-error \
  -output-directory=tests/fixtures tests/fixtures/demo.tex
```
