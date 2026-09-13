### How a date is read out: for now, how long ago it was.
###
### This file is a translation catalogue, in Project Fluent's syntax
### (https://projectfluent.org/fluent/guide/). It is the English one, and it
### is the seed of the catalogue that will hold every sentence Wixen Mail
### speaks once the interface and the screen reader's speech are translated,
### which is version 2. Four messages arrive with it. They are laid out as if
### five thousand were coming, so the first control that is migrated does not
### have to invent a second convention.
###
### Who edits it: anybody writing a sentence a person will hear, and, in
### version 2, a translator who never has to open `src/`. The English text
### here is the text; the code asks for a message by its id and never carries
### a sentence of its own.
###
### The conventions, held by a check in `src/common/catalogue.rs` that walks
### both ways: every id the code can ask for is a message here, and every
### message here is one the code can ask for.
###
### 1. Where a message lives: `locales/<locale>/<area>.ftl`. One file per
###    area rather than one file, because five thousand messages in one file
###    is not a thing anyone translates, and because it gives a rule a check
###    can hold: a message whose id starts `dates-` lives in `dates.ftl`.
###
### 2. What an id is: `<area>-<meaning>`, lower-case kebab, the area being
###    this file's name. The id names what the sentence means, not its
###    English wording, so `dates-just-now` stays right in a language that
###    says it another way.
###
### 3. What a variable is called: the thing it counts, `$minutes`, `$hours`,
###    `$days`, rather than a uniform `$count`, because a translator reads the
###    name. The variable is used in every variant, `[one]` as well as
###    `*[other]`, so every number goes through the number formatter and no
###    variant hard-codes a digit.
###
### 4. Reserved for a control, in version 2: the attributes `.label` for the
###    visible text, `.accessible-name` for what a screen reader says, and
###    `.mnemonic` for the letter after `&`. They are one message so that a
###    translated label and the letter that has to be in it are translated
###    together. Nothing here uses them yet; the names are written down so the
###    first control does not choose three others.
###
### Plural categories are Unicode CLDR's: `one`, `two`, `few`, `many`,
### `other`, and `zero` where a language has it. English has `one` and
### `other`. The rules are the language's, not the translator's, and the code
### picks the category; the translator writes the variants that language
### distinguishes and no more.

dates-minutes-ago = { $minutes ->
    [one] { $minutes } minute ago
   *[other] { $minutes } minutes ago
}

dates-hours-ago = { $hours ->
    [one] { $hours } hour ago
   *[other] { $hours } hours ago
}

dates-days-ago = { $days ->
    [one] { $days } day ago
   *[other] { $days } days ago
}
