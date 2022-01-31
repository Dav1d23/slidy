# Slidy: yet another slideshow software

[![Nix](https://github.com/Dav1d23/slidy/actions/workflows/nix.yml/badge.svg?branch=main)](https://github.com/Dav1d23/slidy/actions/workflows/nix.yml)
[![Rust](https://github.com/Dav1d23/slidy/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/Dav1d23/slidy/actions/workflows/rust.yml)

`Slidy` is just a simple binary that allows you to create simple slides without
the need to use some fancy editor. Slides are defined using a simple format
(defined in this project) and are simple text.
Moreover, `slidy` is also a library: you can define your slides in Rust, and
present them easily. This resolve the issue of "distributing slides" an issue
no more :)

## Short example: `slidy`'s slides language.

Slides can be defined this way using the slidy's language:

```text
:ge :bc 20 40 40 250 :fc 250 250 250 180

:sl
:tb :sz 20 :fc 250 0 0 180
This is title 1
:tb :ps 0.1 0.3 :sz 16
A line
  Another line
    And the last one

:sl
:tb :sz 20 :fc 250 250 0 180
And title 2
:tb :ps 0.1 0.3 :sz 16
Some other content
```

This will give you 2 slides with red title and some content.


A more complicated example is shown in `resources/simple_slide.txt`.

Each usable token is prefixed with a : (column). In this example, we see the
use of
- :ge, which can be used to define global slides parameters, like background
  color, and such;
- :sl, which is the "new slide" identifier;
- :tb and :fg, which are respectively the "text" and "picture" tokens;
- :bc, :fc (background and font colors), :sz(size), :ps(position), that has to be used to put
  the position of the text inside the slide.
  
These are not all the available tokens; better to take a look at the code to
see the complete list, in case.

### Coordinate system
The (0, 0) coordinate is the top-left corner, and (1, 1) is the bottom
right.

### Colors
Colors are in RGB+Alpha format.

# Goals and non-goals
`Slidy`'s does _not_ want to be a replacement for PowerPoint (or Impress, or
whatever): it won't handle all that complexity.
It can be however (and it is in my case) a much simpler tool for showing simple
text during presentations: in my case, it is much easier to type some text
knowing that it will be _surely_ rendered the way I want than to start up
Impress and have to deal with that useless (for my presentations) complexity.
