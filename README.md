# Wikid

Wikid compiles markdown wiki/blog posts into HTML files, including cross references, hyperlinks, and version control.

## File paths
- **text**: Source code, divided into subsections. One for search section.
- **html**: HTML code created by wikid
- **code**: Github code which is referenced in code

---

## Markdown

As for text formats, markdown is used. External links are written as `[link text](hyperlink)`, and internal links are written as `[link text]{reference name}`. References include equations, figures, tables, notes, sections, subsections, etc. You make references by writing a new line beginning with tilde and a label, then reference them as `[]{ref name.}` Example:
```
~fig:one
![Caption]{sec1/fig1}

Reference to figure 1: []{fig:one}.
```

Footnotes are also available[Inline in brackets.] when implemented.

## LaTeX

The `amsmath` package is provided, and the macros `\bm`, `\parens`, `\brackets`, `\braces`, `\eval`, `\fraci`, and `\expp` have been provided.

## To do
* Increment versions
* Separate captions from images so that you can have a figure with multiple images and one caption. Add formatting to the caption.
* Tables
* Footnotes