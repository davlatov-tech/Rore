
---

# **RORE MARKUP LANGUAGE — TECHNICAL SPECIFICATION v2.0**

### **(HTML5 → RORE Mapping)**

### **Full Syntax, Rules, Structures & Element Reference**

---

## 1. INTRODUCTION

Rore is a markup language created to convert HTML5 structures into a simplified, strict-syntax DSL format. It preserves all semantic, structural, and attribute features of HTML5 while representing them through a minimal and clearly defined syntax.

This document formally specifies **almost all HTML5 tags and attributes**, mapped into the Rore format.

---

## 2. CORE SYNTAX RULES

### 2.1 TEXT LITERALS — `{ ... }`

* `{ ... }` represents a text literal.
* Everything inside is immutable and passed directly into the DOM as text.

**Examples:**

```rore
{Hello}:h1
{Submit}:button
```

---

### 2.2 FUNCTION / ELEMENT ARGUMENTS — `()` and `("...")`

#### 2.2.1 `()` — empty logical functions

Used for void or contentless elements.

```rore
():img
():meta
():br
```

#### 2.2.2 `("...")` — valued attribute arguments

Used when an attribute contains a value.

```rore
::href("/home")
::alt("Logo")
::lang("uz")
```

#### 2.2.3 Numeric arguments (NEW RULE)

If the argument contains **only a number**:

* quotes are not required
* px units are automatically added for size-related attributes

Examples:

```rore
::width(500)     → width="500px"
::height(240)    → height="240px"
::max(10)        → max="10"
::value(3)       → value="3"
```

If units are included manually (`"500px"`), the value is taken as-is.

---

## 3. ELEMENT DECLARATION RULES

### 3.1 Basic format

```rore
{Text}:tag
():tag
():tag::attr(...)
{Text}:tag::attr(...)
```

---

### 3.2 Nesting — `->` children operator

**Rules:**

1. Write `Parent ->{ }`
2. All lines below become children
3. New elements start a new parent

**Example:**

```rore
{Header}:header -> {
    {Logo}:img::src("logo.png")
}
    {Nav}:nav ->{
        {Home}:a::href("/")
        {Blog}:a::href("/blog")
}
{Footer}:footer
```

---

## 4. ATTRIBUTE SYNTAX — `::attr(...)`

All HTML5 attributes are written as:

```rore
::name(value)
::name("value")
```

Numeric auto-format:

```rore
::width(500) → width="500px"
```

Custom attribute:

```rore
::attr("data-id", "550")
```

---

## 5. GLOBAL ATTRIBUTES

| HTML Attribute  | Rore                      | Notes         |
| --------------- | ------------------------- | ------------- |
| id              | `#id`                     | shorthand     |
| class           | `.className`              | shorthand     |
| title           | `::title("...")`          | tooltip       |
| lang            | `::lang("uz")`            | language      |
| dir             | `::dir("rtl")`            | direction     |
| hidden          | `::hidden()`              | boolean       |
| tabindex        | `::tabindex(0)`           |               |
| contenteditable | `::contenteditable(true)` |               |
| draggable       | `::draggable(true)`       |               |
| spellcheck      | `::spellcheck(true)`      |               |
| data-*          | `::data-name("value")`    |               |
| aria-*          | `::aria-label("...")`     | accessibility |

Example:

```rore
{Banner}:div.hero#main::lang("uz")::aria-label("Homepage banner")
```

---

## 6. HEAD & META TAGS

### 6.1 `<title>`

```rore
{My Site}:title
```

### 6.2 `<meta>`

```rore
():meta::charset("utf-8")
():meta::name("viewport")::content("width=device-width, initial-scale=1")
```

### 6.3 `<link>`

```rore
():link::rel("stylesheet")::href("/style.css")
():link::rel("preload")::href("main.wasm")::as("script")
():link::rel("icon")::href("/favicon.ico")
```

### 6.4 `<base>`

```rore
():base::href("/")
```

---

## 7. SEMANTIC STRUCTURE TAGS

Examples:

```rore
{Site Title}:header
{Main}:main
{Section}:section
```

Nested:

```rore
{Layout}:main ->
    {Top}:header ->
        {Logo}:img::src("logo.png")
    {Content}:section
```

---

## 8. TEXT & INLINE ELEMENTS

| HTML   | Rore                              |
| ------ | --------------------------------- |
| a      | `{Text}:a::href("...")`           |
| strong | `{Text}:strong`                   |
| em     | `{Text}:em`                       |
| small  | `{Text}:small`                    |
| code   | `{Code}:code`                     |
| span   | `{Text}:span`                     |
| br     | `():br`                           |
| wbr    | `():wbr`                          |
| mark   | `{Highlight}:mark`                |
| time   | `{12:00}:time::datetime("12:00")` |

Example:

```rore
{Click here}:a::href("/")::target("_blank")
```

---

## 9. LISTS

### `<ul>`, `<ol>`, `<li>`

```rore
{Menu}:ul ->{
    {Home}:li
    {Profile}:li  }
```

### `<dl>`, `<dt>`, `<dd>`

```rore
():dl ->
    {CPU}:dt
    {16-core}:dd
```

---

## 10. FORMS & INPUTS

### `<form>`

```rore
{Login}:form::action("/login")::method("post")
```

### `<input>`

```rore
():input::type("text")::name("username")
():input::type("email")::required()
():input::type("file")::multiple()::accept("image/*")
():input::type("checkbox")::checked()
```

Numeric:

```rore
::min(0)
::max(100)
::step(1)
```

### `<textarea>`

```rore
{Message}:textarea::rows(5)::cols(40)
```

### `<select>`

```rore
{Choose City}:select::name("city") ->{
    {Tokyo}:option::value("tokyo")
    {Dubai}:option::value("dubai")  }
```

### `<label>`

```rore
{Name}:label::for("idname")
```

---

## 11. MEDIA ELEMENTS

### `<img>`

```rore
():img::src("a.jpg")::alt("Photo")::width(400)
```

### `<picture>`

```rore
():picture -> {
    ():source::srcset("x-small.jpg")::media("(max-width: 600px)")
    ():img::src("x.jpg")::alt("Image")  }
```

### `<video>`

```rore
{Not supported}:video::controls()::width(640)::height(360) ->{
    ():source::src("v.mp4")::type("video/mp4") }
```

### `<audio>`

```rore
{Not supported}:audio::controls() ->{
    ():source::src("a.mp3")::type("audio/mpeg") }
```

### `<track>`

```rore
():track::kind("subtitles")::src("en.vtt")::srclang("en")
```

---

## 12. TABLE ELEMENTS

```rore
{Products}:table ->{
    ():thead ->{
        ():tr ->{
            {Name}:th
            {Price}:th }}}

    ():tbody ->{
        ():tr ->{
            {Apple}:td
            {5000}:td  }}
```

---

## 13. EMBEDDED CONTENT

### `<iframe>`

```rore
():iframe::src("https://example.com")::width(800)::height(600)::sandbox("allow-scripts")
```

### `<object>` + `<param>`

```rore
{Fallback}:object::data("x.svg")::type("image/svg+xml") ->{
    {Browser not supported}:p   }

():param::name("auto")::value("yes")
```

### `<embed>`

```rore
():embed::src("file.pdf")::type("application/pdf")
```

---

## 14. INTERACTIVE ELEMENTS

### `<details>` + `<summary>`

```rore
():details ->{
    {More}:summary
    {Hidden text}:p   }
```

### `<dialog>`

```rore
{Hello}:dialog::open()
```

---

## 15. ACCESSIBILITY (ARIA)

```rore
{Menu}:nav::role("navigation")::aria-label("Main menu")
```

Examples:

```rore
::aria-label("...")
::aria-hidden(true)
::aria-expanded(false)
::aria-controls("id")
```

---

## 16. SVG & CANVAS

### `<canvas>`

```rore
():canvas::id("stage")::width(800)::height(600)
```

### `<svg>`

```rore
{SVG content}:svg
```

---

## 17. SECURITY ATTRIBUTES

| HTML                      | Rore                           |
| ------------------------- | ------------------------------ |
| rel="noopener noreferrer" | `::rel("noopener noreferrer")` |
| sandbox                   | `::sandbox("allow-scripts")`   |
| crossorigin               | `::crossorigin("anonymous")`   |
| integrity                 | `::integrity("sha384-...")`    |

---

## 18. MICRODATA / RDFa

```rore
{Item}:div::itemtype("http://schema.org/Product")::itemprop("name")
```

---

## 19. QUICK REFERENCE

### ELEMENT

```rore
{Text}:tag
():tag
```

### ATTRIBUTES

```rore
::attr(value)
::width(300)
::href("/home")
```

### CHILDREN

```rore
Parent ->{
    Child1
    Child2
}
```

### MEDIA

```rore
():img::src("a.jpg")::width(400)
```

### FORM

```rore
():input::type("text")
```
