#!/usr/bin/env python3

from pathlib import Path
import subprocess

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet
from reportlab.platypus import Paragraph, SimpleDocTemplate, Spacer


FIXTURES = (
    {
        "name": "demo-1",
        "title": "Paperless API Integration Test",
        "subject": "Primary disposable fixture for live API tests",
        "identifier": "paperless-api-demo-1-document",
        "body": "This document is uploaded by the Rust integration test suite. It contains stable, "
        "searchable text so the test exercises Paperless document ingestion with a real PDF "
        "rather than an arbitrary byte stream.",
        "keywords": "Search keywords: invoice, receipt, contract, integration-fixture, "
        "alpha-bravo, charlie-delta, paperless-search-sentinel.",
    },
    {
        "name": "demo-2",
        "title": "Paperless API Integration Test 2",
        "subject": "Second disposable fixture for live API tests",
        "identifier": "paperless-api-demo-2-document",
        "body": "This is the second document uploaded by the Rust integration test suite. Its "
        "content differs from the primary fixture so Paperless does not reject it as a duplicate.",
        "keywords": "Search keywords: statement, report, secondary-fixture, echo-foxtrot, "
        "golf-hotel, paperless-second-search-sentinel.",
    },
)


def generate(fixture: dict[str, object]) -> None:
    output = Path(__file__).parent / f"{fixture['name']}.pdf"
    doc = SimpleDocTemplate(
        str(output),
        pagesize=A4,
        title=str(fixture["title"]),
        author="paperless-api",
        subject=str(fixture["subject"]),
        pageCompression=1,
        invariant=1,
    )
    styles = getSampleStyleSheet()
    story = [
        Paragraph(str(fixture["title"]), styles["Title"]),
        Spacer(1, 20),
        Paragraph(str(fixture["body"]), styles["BodyText"]),
        Spacer(1, 20),
        Paragraph(
            f'Fixture identifier: <font name="Courier">{fixture["identifier"]}</font>',
            styles["BodyText"],
        ),
        Spacer(1, 20),
        Paragraph(str(fixture["keywords"]), styles["BodyText"]),
    ]
    doc.build(story)

    subprocess.run(
        [
            "qpdf",
            "--object-streams=generate",
            "--compress-streams=y",
            "--recompress-flate",
            "--compression-level=9",
            "--deterministic-id",
            "--replace-input",
            str(output),
        ],
        check=True,
    )


for fixture in FIXTURES:
    generate(fixture)
