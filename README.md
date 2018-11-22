# media-type

**warning: this was original a fork of `mime` intending to add some of the parts missing
  in the `mime` crate. Since then it largely diverged from the `mime` crate and is now
  a re-write/alternative implementation instead of a fork as there is hardly any overlap with the original crate. Sadly it became slightly  complex and needs to go through some simplification and optimization before it can be widely used. If possible it will
  be merged with the `mime` crate in the future after simplification/optimization.
  Through for some use cases it's still fine. Just slower
  then `mime` and not always very ergonomic.**


A crate providing `media-type` type(s). Media-types are also sometimes
known as `mime-type` or just `mime` (but the last one is wrong as as
`media-types` are just a small part of the mime standart).

A classical example for a media type is `text/plain; charset=utf-8`.

While media types seem to be simple they have some tricky parts
(around encoding non us-ascii chars and semantic equality) and
sadly there is not a `media-type` standard. Instead they are defined
in multiple standards and **there definition differs**. But not only
does the definition differ "thinks" they are used in they also differ
depending of different ways the standard can be used. E.g. media-types
from the http standard differ in wether or not the "obsolete" part of
the http grammar can be used which was often (miss) used to add non
us-ascii text (through by now there is a "official" way how to encode
non us-ascii utf-8, which is based to a similar standard for media-types
in mime/mail but with some differences).

The most strict mime standard is the one used for registering media-types
with the IANA registry (which should _always_ be done). While all media-types
you create should comply with that standard you likely might have to handle
media-types with are either incompatible or do extend the grammer on a non
syntactic but semantic level. E.g. by specifying that parameters which have a
certain form should be interpreted as percent encoded utf-8.

Alternatively there are some lossy standards about parsing media types in a
very fail safe wai but potentially getting out garbage types which can't be
used for anything.


The idea behind this crate is to provide a way to:

1. Have some form of "any" media type which can be read but does not specify
   which grammar was used to parse/validate it.
2. Have a number of wrappers which, in a type-safe way, add information about
   which grammar was used to validate it and as such give guarantees
   about it's structure.
3. Be generic enough to allow people to add there own approach.


