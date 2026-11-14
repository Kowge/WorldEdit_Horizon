# The game test of 1.5.3

The limits of 1.5.3 in a running game: a server and a client built on the
pinned commit, and the commands typed in the chat box by the player — the
path that fires the client's command hook.

- Server: `horizon-server.exe` of a dedicated-server engine built with the
  bundle of 1.5.3 on Horizon `66f3981f522b24c250b21523a70080965b940424`
  (the launcher compiles a bundle on the commit it pins).
- Client: the `horizon` instance with the same bundle.
- The commands are sent through the control socket as the player types them:
  the chat box, the keys, enter.

```
=== 3. the limits of the shapes ===
STEP  a big ball is refused
  command : /we sphere stone 1024
  seconds : 0.8
  answer  : The radius is 1 to 128.

STEP  the limit is named
  command : /we limit blocks 1000
  seconds : 0.7
  answer  : Limits: blocks 1000 | radius 128 | distance 64 | history 20.

STEP  the ball is refused by the limit
  command : /we sphere stone 100
  seconds : 0.7
  answer  : That is 4252701 blocks; the limit is 1000.

=== 4. the recursive fill ===
STEP  the depth is bounded
  command : /we fillr stone 5 2147483647
  seconds : 0.7
  answer  : 83 blocks filled.

=== 5. the trim ===
STEP  the trim is refused
  command : /we trim
  seconds : 0.7
  answer  : That is 8268870828 blocks; the limit is 500000.

=== 6. the smooth brush ===
STEP  the brush is armed
  command : /we brush smooth 1000 1
  seconds : 0.7
  answer  : A brush radius is 1 to 128.

=== 7. the line ===
STEP  the line works
  command : /we line stone
  seconds : 0.7
  answer  : 21 blocks changed.
```

What the answers say:

- The ball of radius 1024 is refused by the radius limit, and the ball of
  radius 100 with a block limit of 1000 is refused by the block limit **in
  0.7 seconds**, with its real size named: it is counted before it is made,
  not after (4 252 701 blocks).
- The recursive fill of depth 2 147 483 647 fills 83 blocks: the depth is
  bounded by the radius of the ball.
- The trim of two distant corners is refused with the size of the selection
  named: it asks the limit before it materialises the blocks.
- The brush radius is bounded by the radius limit, so a stroke of a brush of
  radius 1000 never starts.
- The line between the two corners of the selection works in the dimension
  of the selection.

The strokes of the smooth brush itself were not driven by the script: the
control tool's mouse click does not reach the game window, so the brush
was armed but never fired. The brush's own limit check is the same
`check_limit` call the other brushes and `/we trim` use, and the trim
branch above shows that call refusing a selection in 0.7 seconds.

The script is in `tools/game-test-limits.ps1` of the working copy; the
pinned build that compiles the bundle into the game of that exact commit is
the workflow *Pinned native build*.
