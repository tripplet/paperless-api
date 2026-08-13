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

The normal suite runs three API-area scenarios in order:

1. `documents` uploads a PDF, polls and acknowledges its task, reads and patches the document,
   downloads its original and thumbnail, queries it, and removes it.
2. `attributes` exercises create, update, direct loading, cached loading, and deletion for tags,
   correspondents, and document types.
3. `server` checks status, statistics, workflows, saved views, and all refreshable caches.

Rust does not guarantee ordering between separate test cases, so CI expresses this order inside
`tests::ordered_integration_suite`. Each scenario cleans up the records it creates.

Each step also has an ignored test for focused debugging. These tests are self-contained and obtain
their own client context when run in isolation:

```sh
cargo test --package paperless-api-integration-tests \
  documents::debug_document_lifecycle -- --ignored --exact --nocapture

cargo test --package paperless-api-integration-tests \
  attributes::debug_attribute_lifecycles -- --ignored --exact --nocapture

cargo test --package paperless-api-integration-tests \
  server::debug_read_only_endpoints -- --ignored --exact --nocapture
```

By default, the Rust setup requests an API token using the username and password in `config.json`
and caches the authenticated client for the test process. To use an existing instance, provide a
`token` in a separate config file; when present, it takes precedence over username and password.

The checked-in `fixtures/demo.pdf` is generated from `fixtures/demo.tex`, then optimized with
Ghostscript and qpdf to keep the test fixture small:

```sh
pdflatex -interaction=nonstopmode -halt-on-error \
  -output-directory=tests/fixtures tests/fixtures/demo.tex

gs -q -dNOPAUSE -dBATCH -sDEVICE=pdfwrite \
  -dCompatibilityLevel=1.5 -dAutoRotatePages=/None \
  -dCompressFonts=true -dSubsetFonts=true -dDetectDuplicateImages=true \
  -dOmitInfoDate=true -dOmitID=true \
  -sOutputFile=tests/fixtures/demo.optimized.pdf tests/fixtures/demo.pdf

qpdf --object-streams=generate --compress-streams=y --recompress-flate \
  --compression-level=9 tests/fixtures/demo.optimized.pdf tests/fixtures/demo.pdf

rm tests/fixtures/demo.optimized.pdf
```
