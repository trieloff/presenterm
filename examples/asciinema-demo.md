---
title: Asciinema Demo
author: Presenterm
---

# Asciinema Recording Support

This presentation demonstrates embedding asciinema recordings in slides.

<!-- end_slide -->

## Live Terminal Recording (Auto-play, Loop)

Here's an embedded asciinema recording that auto-plays and loops:

```asciinema +start:auto +play:loop
demo.cast
```

<!-- end_slide -->

## Using the cast alias (Wait, Once)

You can also use `cast` as the language identifier.

This one waits for a keypress and plays once:

```cast +start:wait +play:once
demo.cast
```

Press space to start playback!

<!-- end_slide -->

## End

That's it! Asciinema recordings are now supported in presenterm.
