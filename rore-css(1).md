# Rore CSS Specification v1.0

## Rore Styling System (RSS) — Full Technical Documentation

---

# 1. Introduction

Rore CSS is a modernized, shortened, and compiler-optimized variant of HTML/CSS syntax.
Rore CSS:

* Can be used inline inside HTML
* Can be written in a separate `.rstyle` or `.css.rore` file
* Uses brackets: `[ ... ]` to define style blocks
* Covers **100% of CSS capabilities**
* Is compiled into **GPU-optimized layout** by the Rore compiler

---

# 2. General Rules

### 2.1 Style Block

```
[element.selector][
    property::value
    property::value value
    nested::property::value
]
```

### 2.2 Inline Style

```
{Hello}:p(style::color::red::fs::22)
```

### 2.3 Each Style Command:

* Follows the format `property::value`
* Space → **separates arguments**
* `::` → **chain symbol**

### 2.4 Shorthand Rules

Shorthands similar to CSS exist:

* `p::10 20` → `padding-top/right/bottom/left`
* `m::0 auto` → margin
* `b::1 solid #000` → border

---

# 3. Selectors (fully compatible with CSS)

## 3.1 Element Selector

```
p[ ... ]
div[ ... ]
img[ ... ]
```

## 3.2 Class Selector

```
.my-card[ ... ]
```

## 3.3 ID Selector

```
#header[ ... ]
```

## 3.4 Group Selector

```
h1, h2, h3[
    color::red
]
```

## 3.5 Nested Selectors

Inside Rore CSS:

```
.card[
    bg::white
    p::20

    .title[
        fs::24
    ]

    button[
        bg::blue
    ]
]
```

---

# 4. Rore CSS Property Catalog (complete, full CSS coverage)

## 4.1 Layout Properties

### Display

```
d::none | block | inline | flex | grid | inline-block | inline-flex | inline-grid
```

### Flexbox

```
fd::row | column | row-reverse | column-reverse
fw::wrap | nowrap
jc::center | space-between | space-around | space-evenly | flex-start | flex-end
ai::center | flex-start | flex-end | stretch
as::auto | flex-start | center
gap::10 | 10 20
```

### Grid

```
gtc::repeat(3, 1fr)
gtr::auto 1fr auto
gta::1 / 3
gc::1 / span 2
```

### Positioning

```
position::static | relative | absolute | fixed | sticky
top::10
left::50%
z::100
```

---

## 4.2 Box Model

### Margin

```
m::10
m::10 20
m::10 20 30 40
```

### Padding

```
p::20
p::10 20
```

### Border

```
b::1 solid #333
bt::2 dashed red
br::10
brt::10
```

### Width & Height

```
w::100
w::100%
max-w::500
h::auto
min-h::200
```

### Overflow

```
overflow::hidden | scroll | auto
```

---

## 4.3 Background

```
bg::red
bg::#333
bg::rgba(0,0,0,0.5)
bg::url(image.png)
bg-size::cover
bg-pos::center
bg-repeat::no-repeat
```

---

## 4.4 Typography

```
fs::16          // font-size
ff::Roboto      // font-family
fw::bold        // font-weight
ls::1           // letter-spacing
lh::1.5         // line-height
ta::center      // text-align
td::underline   // text-decoration
tt::uppercase   // text-transform
color::#333
```

---

## 4.5 Shadow

```
shadow::sm
shadow::md
shadow::lg
shadow::0 4 10 rgba(0,0,0,0.3)
```

---

## 4.6 Transform

```
scale::1.2
rotate::45deg
translate::10 20
skew::10deg
```

---

## 4.7 Transitions & Animations

### Transition

```
transition::all 0.2s ease
```

### Keyframes (Rore syntax)

```
@keyframes fade[
    0%[opacity::0]
    100%[opacity::1]
]
```

---

## 4.8 Cursor & User Select

```
cursor::pointer
select::none
```

---

# 5. Pseudo-Classes (CSS compatible)

```
hover::bg::red
focus::b::1 solid blue
active::scale::0.95
disabled::opacity::0.5
```

**Example:**

```
button[
    bg::blue
    color::white
    hover::bg::darkblue
]
```

---

# 6. Media Queries (Responsive)

```
when(sm)::fs::14
when(md)::fs::18
when(lg)::fs::22
when(xl)::fs::26
```

Or block-style:

```
@screen sm[
    .card[w::100%]
]
```

---

# 7. Theming

```
theme(light)[
    bg::white
    color::black
]

theme(dark)[
    bg::#111
    color::#eee
]
```

---

# 8. Import

```
@import "buttons.rstyle"
@import "layout/grid.rstyle"
```

---

# 9. Variable System

```
$primary::#3498db
$radius::8

button[
    bg::$primary
    br::$radius
]
```

---

# 10. Writing Alongside Rore HTML

```
{Submit}:button[
    bg::green
    p::10 20
    br::6
    hover::bg::darkgreen
]
```

---

# 11. Full Large Example — Landing Page

```
header.hero[
    h::400
    bg::url(hero.jpg)
    bg-size::cover
    d::flex
    fd::column
    jc::center
    ai::center
    color::white
]

    {Rore UI Framework}:h1[
        fs::48
        fw::700
    ]

    {Fast. Native. Universal.}:p[
        fs::20
        color::#eee
    ]

    {Get Started}:button[
        p::14 28
        bg::#3498db
        color::white
        br::8
        hover::bg::#2b81c5
    ]
```

---

# 12. Rore CSS Engine — Technical Processing Pipeline

1️⃣ AST Parser
2️⃣ Normalization Layer
3️⃣ Cascade Engine (CSS-like)
4️⃣ Compute Stage (computed value)
5️⃣ GPU Style Resolver
6️⃣ Rore Layout Engine (Flex/Grid)
7️⃣ Render Tree → GPU Commands

Rore CSS **covers full CSS functionality**, but:
▪ syntax is shorter
▪ performance is higher
▪ optimized for GPU
▪ deeply integrated with Rore HTML
▪ compiler detects errors early

