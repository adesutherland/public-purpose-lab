#!/usr/bin/env python3
"""Render a bounded gate show-and-tell Markdown report as a polished PDF.

The renderer intentionally supports the small Markdown subset used by the
architecture evidence reports: headings, paragraphs, lists, tables, images,
block quotes, fenced code and explicit page breaks. The Markdown remains the
canonical editable source.
"""

from __future__ import annotations

import argparse
import html
import re
from pathlib import Path

from PIL import Image as PILImage
from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    HRFlowable,
    Image,
    KeepTogether,
    ListFlowable,
    ListItem,
    PageBreak,
    PageTemplate,
    Paragraph,
    Preformatted,
    Spacer,
    Table,
    TableStyle,
)


INK = colors.HexColor("#172421")
GREEN = colors.HexColor("#1E5145")
MID_GREEN = colors.HexColor("#316B59")
PALE_GREEN = colors.HexColor("#E6EEE8")
CREAM = colors.HexColor("#FAF9F4")
ORANGE = colors.HexColor("#D9793B")
MUTED = colors.HexColor("#5C6A66")
RULE = colors.HexColor("#CBD5CF")


def register_fonts() -> tuple[str, str]:
    didot = Path("/System/Library/Fonts/Supplemental/Didot.ttc")
    bodoni = Path("/System/Library/Fonts/Supplemental/Bodoni 72 OS.ttc")
    heading = "Times-Roman"
    if didot.exists():
        pdfmetrics.registerFont(TTFont("PPLDidot", str(didot), subfontIndex=0))
        heading = "PPLDidot"
    elif bodoni.exists():
        pdfmetrics.registerFont(TTFont("PPLBodoni", str(bodoni), subfontIndex=0))
        heading = "PPLBodoni"
    return heading, "Helvetica"


def styles_for(heading_font: str, body_font: str) -> dict[str, ParagraphStyle]:
    base = getSampleStyleSheet()
    return {
        "title": ParagraphStyle(
            "Title",
            parent=base["Title"],
            fontName=heading_font,
            fontSize=34,
            leading=36,
            textColor=INK,
            alignment=TA_LEFT,
            spaceAfter=10 * mm,
        ),
        "h2": ParagraphStyle(
            "H2",
            parent=base["Heading2"],
            fontName=heading_font,
            fontSize=24,
            leading=33,
            textColor=INK,
            spaceBefore=6 * mm,
            spaceAfter=3.5 * mm,
            keepWithNext=True,
        ),
        "h3": ParagraphStyle(
            "H3",
            parent=base["Heading3"],
            fontName=body_font,
            fontSize=12,
            leading=15,
            textColor=MID_GREEN,
            uppercase=True,
            spaceBefore=5 * mm,
            spaceAfter=2.5 * mm,
            keepWithNext=True,
        ),
        "body": ParagraphStyle(
            "Body",
            parent=base["BodyText"],
            fontName=body_font,
            fontSize=9.3,
            leading=14,
            textColor=INK,
            spaceAfter=3 * mm,
        ),
        "small": ParagraphStyle(
            "Small",
            parent=base["BodyText"],
            fontName=body_font,
            fontSize=7.8,
            leading=10.5,
            textColor=MUTED,
        ),
        "caption": ParagraphStyle(
            "Caption",
            parent=base["BodyText"],
            fontName=body_font,
            fontSize=7.7,
            leading=10.5,
            textColor=MUTED,
            spaceBefore=2 * mm,
            spaceAfter=5 * mm,
        ),
        "quote": ParagraphStyle(
            "Quote",
            parent=base["BodyText"],
            fontName=body_font,
            fontSize=10,
            leading=15,
            textColor=INK,
            leftIndent=6 * mm,
            borderColor=ORANGE,
            borderWidth=0,
            borderPadding=(2 * mm, 4 * mm, 2 * mm, 4 * mm),
            backColor=colors.HexColor("#FFF8EC"),
            spaceAfter=4 * mm,
        ),
        "code": ParagraphStyle(
            "Code",
            parent=base["Code"],
            fontName="Courier",
            fontSize=7.2,
            leading=9.5,
            leftIndent=3 * mm,
            rightIndent=3 * mm,
            borderColor=RULE,
            borderWidth=0.5,
            borderPadding=3 * mm,
            backColor=colors.HexColor("#F2F4F0"),
            spaceAfter=4 * mm,
        ),
        "table": ParagraphStyle(
            "TableCell",
            parent=base["BodyText"],
            fontName=body_font,
            fontSize=7.2,
            leading=9.4,
            textColor=INK,
        ),
        "table_header": ParagraphStyle(
            "TableHeader",
            parent=base["BodyText"],
            fontName=body_font,
            fontSize=7.2,
            leading=9.4,
            textColor=colors.white,
        ),
    }


def inline_markup(value: str) -> str:
    escaped = html.escape(value.strip(), quote=False)
    escaped = re.sub(r"`([^`]+)`", r"<font name='Courier'>\1</font>", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", escaped)
    escaped = re.sub(r"\[([^]]+)\]\([^)]+\)", r"<u>\1</u>", escaped)
    return escaped


def table_widths(column_count: int, available: float) -> list[float]:
    if column_count == 2:
        return [available * 0.29, available * 0.71]
    if column_count == 3:
        return [available * 0.22, available * 0.29, available * 0.49]
    if column_count == 4:
        return [available * 0.15, available * 0.22, available * 0.28, available * 0.35]
    return [available / column_count] * column_count


def parse_table(lines: list[str], style: dict[str, ParagraphStyle], width: float) -> Table:
    rows = []
    for row_index, line in enumerate(lines):
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if row_index == 1 and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells):
            continue
        cell_style = style["table_header"] if not rows else style["table"]
        rows.append([Paragraph(inline_markup(cell), cell_style) for cell in cells])
    result = Table(
        rows,
        colWidths=table_widths(len(rows[0]), width),
        repeatRows=1,
        hAlign="LEFT",
        splitByRow=True,
    )
    result.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), GREEN),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("GRID", (0, 0), (-1, -1), 0.35, RULE),
                ("ROWBACKGROUNDS", (0, 1), (-1, -1), [CREAM, PALE_GREEN]),
                ("LEFTPADDING", (0, 0), (-1, -1), 5),
                ("RIGHTPADDING", (0, 0), (-1, -1), 5),
                ("TOPPADDING", (0, 0), (-1, -1), 5),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
            ]
        )
    )
    return result


def report_image(path: Path, caption: str, width: float, height: float, style: dict[str, ParagraphStyle]):
    with PILImage.open(path) as source:
        source_width, source_height = source.size
    scale = min(width / source_width, height / source_height)
    picture = Image(str(path), width=source_width * scale, height=source_height * scale)
    picture.hAlign = "CENTER"
    return KeepTogether(
        [
            Table(
                [[picture]],
                style=[
                    ("BOX", (0, 0), (-1, -1), 0.7, RULE),
                    ("BACKGROUND", (0, 0), (-1, -1), colors.white),
                    ("LEFTPADDING", (0, 0), (-1, -1), 2),
                    ("RIGHTPADDING", (0, 0), (-1, -1), 2),
                    ("TOPPADDING", (0, 0), (-1, -1), 2),
                    ("BOTTOMPADDING", (0, 0), (-1, -1), 2),
                ],
            ),
            Paragraph(inline_markup(caption), style["caption"]),
        ]
    )


def markdown_story(source: Path, style: dict[str, ParagraphStyle], content_width: float, content_height: float):
    lines = source.read_text(encoding="utf-8").splitlines()
    story = []
    paragraph: list[str] = []
    index = 0

    def flush_paragraph() -> None:
        if paragraph:
            story.append(Paragraph(inline_markup(" ".join(paragraph)), style["body"]))
            paragraph.clear()

    while index < len(lines):
        line = lines[index]
        stripped = line.strip()
        if not stripped:
            flush_paragraph()
            index += 1
            continue
        if stripped == "<!-- pagebreak -->":
            flush_paragraph()
            story.append(PageBreak())
            index += 1
            continue
        if stripped == "<!-- topspace -->":
            flush_paragraph()
            story.append(Spacer(1, 5 * mm))
            index += 1
            continue
        if stripped.startswith("```"):
            flush_paragraph()
            code: list[str] = []
            index += 1
            while index < len(lines) and not lines[index].strip().startswith("```"):
                code.append(lines[index])
                index += 1
            story.append(Preformatted("\n".join(code), style["code"]))
            index += 1
            continue
        image_match = re.fullmatch(r"!\[([^]]+)]\(([^)]+)\)", stripped)
        if image_match:
            flush_paragraph()
            image_path = (source.parent / image_match.group(2)).resolve()
            story.append(report_image(image_path, image_match.group(1), content_width, content_height * 0.72, style))
            index += 1
            continue
        if stripped.startswith("| "):
            flush_paragraph()
            table_lines = []
            while index < len(lines) and lines[index].strip().startswith("|"):
                table_lines.append(lines[index])
                index += 1
            story.append(parse_table(table_lines, style, content_width))
            story.append(Spacer(1, 4 * mm))
            continue
        if stripped.startswith("# "):
            flush_paragraph()
            story.append(Spacer(1, 22 * mm))
            story.append(Paragraph(inline_markup(stripped[2:]), style["title"]))
            story.append(HRFlowable(width="100%", thickness=1.4, color=GREEN, spaceAfter=7 * mm))
            index += 1
            continue
        if stripped.startswith("## "):
            flush_paragraph()
            story.append(Paragraph(inline_markup(stripped[3:]), style["h2"]))
            index += 1
            continue
        if stripped.startswith("### "):
            flush_paragraph()
            story.append(Paragraph(inline_markup(stripped[4:]), style["h3"]))
            index += 1
            continue
        if stripped.startswith("> "):
            flush_paragraph()
            quote = []
            while index < len(lines) and lines[index].strip().startswith(">"):
                quote.append(lines[index].strip().lstrip(">").strip())
                index += 1
            story.append(Paragraph(inline_markup(" ".join(quote)), style["quote"]))
            continue
        if re.match(r"^[-*] ", stripped) or re.match(r"^\d+\. ", stripped):
            flush_paragraph()
            ordered = bool(re.match(r"^\d+\. ", stripped))
            items = []
            while index < len(lines):
                item_line = lines[index].strip()
                match = re.match(r"^\d+\. (.+)", item_line) if ordered else re.match(r"^[-*] (.+)", item_line)
                if not match:
                    break
                if ordered:
                    items.append(
                        ListItem(
                            Paragraph(inline_markup(match.group(1)), style["body"]),
                            leftIndent=4 * mm,
                        )
                    )
                else:
                    bullet_style = ParagraphStyle(
                        "Bullet",
                        parent=style["body"],
                        leftIndent=6 * mm,
                        firstLineIndent=-4 * mm,
                    )
                    items.append(
                        Paragraph(
                            inline_markup(match.group(1)),
                            bullet_style,
                            bulletText="-",
                        )
                    )
                index += 1
            if ordered:
                story.append(
                    ListFlowable(
                        items,
                        bulletType="1",
                        start="1",
                        leftIndent=6 * mm,
                        bulletFontName="Helvetica",
                        bulletFontSize=8,
                        spaceAfter=2 * mm,
                    )
                )
            else:
                story.extend(items)
            continue
        paragraph.append(stripped)
        index += 1

    flush_paragraph()
    return story


def render(source: Path, output: Path) -> None:
    heading_font, body_font = register_fonts()
    style = styles_for(heading_font, body_font)
    width, height = A4
    left = right = 20 * mm
    top = 18 * mm
    bottom = 18 * mm
    content_width = width - left - right
    content_height = height - top - bottom

    output.parent.mkdir(parents=True, exist_ok=True)
    document = BaseDocTemplate(
        str(output),
        pagesize=A4,
        leftMargin=left,
        rightMargin=right,
        topMargin=top,
        bottomMargin=bottom,
        title="Public Purpose Lab progress and show-and-tell",
        author="Public Purpose Lab",
        subject="Synthetic development evidence and show-and-tell",
    )

    def decorate(canvas, doc):
        canvas.saveState()
        canvas.setStrokeColor(RULE)
        canvas.setLineWidth(0.4)
        canvas.line(left, 12 * mm, width - right, 12 * mm)
        canvas.setFont(body_font, 7)
        canvas.setFillColor(MUTED)
        canvas.drawString(left, 7.5 * mm, "PUBLIC PURPOSE LAB · SYNTHETIC DEVELOPMENT EVIDENCE")
        canvas.drawRightString(width - right, 7.5 * mm, f"PAGE {doc.page}")
        canvas.restoreState()

    frame = Frame(left, bottom, content_width, content_height, id="report")
    document.addPageTemplates([PageTemplate(id="report", frames=[frame], onPage=decorate)])
    story = markdown_story(source, style, content_width, content_height)
    document.build(story)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    render(arguments.source.resolve(), arguments.output.resolve())


if __name__ == "__main__":
    main()
