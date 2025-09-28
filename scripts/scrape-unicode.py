# /// script
# requires-python = ">=3.12"
# dependencies = ["pyquery", "requests"]
# ///
import requests
from pyquery import PyQuery as pq

base_source = requests.get(
    "https://en.wikipedia.org/wiki/List_of_Unicode_characters",
    headers={"User-Agent": "ut-cache-build (github.com/ksdme/ut)"},
).text

d = pq(base_source)
characters = {}

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Unicode_symbols
unicode_symbols = []
for tr in d("#Table_Unicode_symbols")("tr")[1:-1]:
    tds = tr.findall("td")
    unicode_symbols.append((tds[2].find("a").text.strip(), tds[1].text.strip()))

characters["Unicode Symbols"] = unicode_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#General_Punctuation
landmark = d('a[title="General Punctuation"]')[-1]
table = landmark.getparent().getparent().getparent().getparent().getparent()
assert table.tag == "table"

general_punctuation_symbols = []
for td in pq(table)("td[title]"):
    title = td.attrib["title"]
    if title == "Reserved":
        continue

    name = title.split(":")[1].strip()

    letter = td.find("a").text
    if not letter:
        continue

    general_punctuation_symbols.append((name, letter))

characters["General Punctuation"] = general_punctuation_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Superscripts_and_Subscripts
def resolve_table(title: str):
    landmark = d(f'b [title="{title}"]')[-1]
    table = landmark.getparent().getparent().getparent().getparent().getparent()
    assert table.tag == "table", table.tag

    for td in pq(table)("td[title]"):
        title = td.attrib["title"]
        if title == "Reserved":
            continue

        name = td.attrib["title"].split(":")[1].strip()

        letter = td.text
        if not letter:
            a = td.find("a")
            if a is not None:
                letter = a.text

            span = td.find("span")
            if span is not None:
                a = span.find("a")
                if a is not None:
                    letter = a.text
                else:
                    letter = span.find("span").find("span").find("span").text

        letter = letter.strip()

        yield (td, name, letter)

subscript_symbols = []
superscript_symbols = []
for (_, name, letter) in resolve_table("Superscripts and Subscripts"):
    if "SUPERSCRIPT" in name:
        superscript_symbols.append((name, letter))
    elif "SUBSCRIPT" in name:
        subscript_symbols.append((name, letter))
    else:
        assert False, "unreachable"

characters["Subscript Symbols"] = subscript_symbols
characters["Superscript Symbols"] = superscript_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Currency_Symbols
currency_symbols = []
for (_, name, letter) in resolve_table("Currency Symbols (Unicode block)"):
    currency_symbols.append((name, letter))

characters["Currency Symbols"] = currency_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Letterlike_Symbols
letterlike_symbols = []
for (_, name, letter) in resolve_table("Letterlike Symbols"):
    letterlike_symbols.append((name, letter))

characters["Letterlike Symbols"] = letterlike_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Number_Forms
number_forms = []
for (_, name, letter) in resolve_table("Number Forms"):
    number_forms.append((name, letter))

characters["Number Forms"] = number_forms

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Arrows
# TODO: 4 more supplemental blocks are available.
arrow_symbols = []
for (_, name, letter) in resolve_table("Arrows (Unicode block)"):
    arrow_symbols.append((name, letter))

characters["Arrow Symbols"] = arrow_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Mathematical_symbols
# TODO: 3 more supplemental blocks are available.
math_symbols = []
for (_, name, letter) in resolve_table("Mathematical Operators (Unicode block)"):
    math_symbols.append((name, letter))

characters["Math Symbols"] = math_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Miscellaneous_Technical
misc_technical_symbols = []
for (_, name, letter) in resolve_table("Miscellaneous Technical"):
    misc_technical_symbols.append((name, letter))

characters["Misc Technical Symbols"] = math_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Enclosed_Alphanumerics
enclosed_alphanumeric_symbols = []
for (_, name, letter) in resolve_table("Enclosed Alphanumerics"):
    enclosed_alphanumeric_symbols.append((name, letter))

characters["Enclosed Alphanumeric Symbols"] = enclosed_alphanumeric_symbols


# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Box_Drawing
box_drawing_symbols = []
for (_, name, letter) in resolve_table("Box Drawing"):
    box_drawing_symbols.append((name, letter))

characters["Box Drawing Symbols"] = box_drawing_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Block_Elements
block_element_symbols = []
for (_, name, letter) in resolve_table("Box Drawing"):
    block_element_symbols.append((name, letter))

characters["Block Element Symbols"] = block_element_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Geometric_Shapes
geometric_shape_symbols = []
for (_, name, letter) in resolve_table("Box Drawing"):
    geometric_shape_symbols.append((name, letter))

characters["Geometric Shape Symbols"] = geometric_shape_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Symbols_for_Legacy_Computing
legacy_computing_symbols = []
for (_, name, letter) in resolve_table("Symbols for Legacy Computing"):
    legacy_computing_symbols.append((name, letter))

characters["Legacy Computing Symbols"] = legacy_computing_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Miscellaneous_Symbols
misc_symbols = []
for (_, name, letter) in resolve_table("Miscellaneous Symbols"):
    misc_symbols.append((name, letter))

characters["Misc Symbols"] = misc_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Dingbats
landmark = d('a[title="Dingbats (Unicode block)"]')[-1]
table = landmark.getparent().getnext()
assert table.tag == "table"

dingbats_symbols = []
for tr in pq(table)("tr")[1:]:
    tds = tr.findall("td")

    name = tds[2].text
    if not name:
        name = tds[2].find("a").text

    name = name.strip()
    letter = tds[1].text.strip()

    dingbats_symbols.append((name, letter))

characters["Dingbats"] = dingbats_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Alchemical_symbols
alchemical_symbols = []
for (_, name, letter) in resolve_table("Alchemical Symbols (Unicode block)"):
    alchemical_symbols.append((name, letter))

characters["Alchemical Symbols"] = alchemical_symbols

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Mahjong_Tiles
mahjong_tiles = []
for (_, name, letter) in resolve_table("Mahjong Tiles (Unicode block)"):
    mahjong_tiles.append((name, letter))

characters["Mahjong Tiles"] = mahjong_tiles

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Domino_Tiles
domino_tiles = []
for (_, name, letter) in resolve_table("Domino Tiles"):
    domino_tiles.append((name, letter))

characters["Domino Tiles"] = domino_tiles

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Playing_Cards
playing_cards = []
for (_, name, letter) in resolve_table("Playing Cards (Unicode block)"):
    playing_cards.append((name, letter))

characters["Playing Cards"] = playing_cards

# https://en.wikipedia.org/wiki/List_of_Unicode_characters#Chess_Symbols
landmark = d('[title="U+2654: WHITE CHESS KING"]')[-1]
table = landmark.getparent().getparent().getparent()
assert table.tag == "table"

chess_symbols = []
for td in pq(table)("td[title]"):
    title = td.attrib["title"]
    if title == "Reserved":
        continue

    name = title.split(":")[1].strip()
    letter = td.text.strip()
    chess_symbols.append((name, letter))

characters["Chess Symbols"] = chess_symbols

# https://en.wikipedia.org/wiki/Emoji#Unicode_blocks
emoji_source = requests.get(
    "https://en.wikipedia.org/wiki/Emoji",
    headers={"User-Agent": "ut-cache-build (github.com/ksdme/ut)"},
).text

landmark = pq(emoji_source)('td[title="U+00A9: COPYRIGHT SIGN"]')[-1]
table = landmark.getparent().getparent().getparent()
assert table.tag == "table"

emoji_symbols = []
for td in pq(table)("td[title]"):
    title = td.attrib["title"]
    if title == "Reserved":
        continue

    name = title.split(":")[1].strip()

    letter = td.text
    if not letter:
        letter = td.find("a").text
    letter = letter.strip()

    emoji_symbols.append((name, letter))

characters["Emojis"] = emoji_symbols

# Rust
out = []
for group, items in characters.items():
    letters = []
    for (name, letter) in items:
        name = name.title()
        letters.append(f"(\"{name}\", \"{letter}\")")

    letters = ",\n".join(letters)
    out.append(f"(\"{group}\", &[{letters}])")

lines = ",\n".join(out)
print(f"&[{lines}]")
