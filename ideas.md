> Audio : More options
Allow to seperately :
- Ignore audio device errors at program boot (set a variable at program boot via a cmd arg and generate error when using --[AUDIO]-- with this arg)
- Set the path of the audio file / Arg 1 : Path
- Play / Arg 1 : Starting position
- Pause
- Stop
- Set volume / Arg 1 : Volume
and thus via different functions.

> Documentation
Auto generate a doc by searching for the consts in parser from line 7. (using Iterator::take_while to walk thru the lines)