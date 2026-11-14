# The game test of the limits, end to end: the server on the fresh engine, the
# client with the fresh mod, and the player's own commands in the chat box.
# Every command here used to be able to run away: the shapes that collect
# before they are checked, the trim without its limit, the brush without its
# limit and the line that ignores the dimension.
$ErrorActionPreference = 'Continue'
$ctl = 'D:\minimax\WorldEdit\we-engine-153\horizon-ctl.exe'
$srv = 'D:\minimax\WorldEdit\we-engine-153\horizon-server.exe'
$dir = 'D:\minimax\WorldEdit\we-server'
$sock = 'C:\Users\Kowge\AppData\Local\Temp\horizon-control.json'
$port = 25566

function Ctl { param([string[]]$CtlArgs) & $ctl --socket $sock @CtlArgs }

function Say([string]$text) {
    # As the player types it: the chat box, the keys, enter.
    Ctl @('key', 't', 'tap') | Out-Null
    Start-Sleep -Milliseconds 700
    Ctl @('type', $text) | Out-Null
    Start-Sleep -Milliseconds 700
    Ctl @('key', 'enter', 'tap') | Out-Null
    Start-Sleep -Seconds 2
}

function Ask([string]$title, [string[]]$CtlArgs, [int]$wait = 12) {
    # The action, the time it takes and the answer of the server.
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    Ctl $CtlArgs | Out-Null
    $line = '(no answer)'
    while ($watch.Elapsed.TotalSeconds -lt $wait) {
        Start-Sleep -Milliseconds 700
        $chat = (Ctl @('chat-log', '1') | ConvertFrom-Json).chat
        if ($chat) { $line = $chat[0] }
        if ($line -and $line -notmatch 'Unknown|error:') { break }
    }
    $gone = [math]::Round($watch.Elapsed.TotalSeconds, 1)
    "STEP  $title"
    "  action : $($CtlArgs -join ' ')"
    "  seconds : $gone"
    "  answer  : $line"
    ""
}

function Cmd([string]$title, [string]$text, [int]$wait = 12) {
    Ask $title @('command', $text) $wait
}

"=== 1. the server on the fresh engine ==="
& $srv --dir $dir start
Start-Sleep -Seconds 25
& $srv --dir $dir status

"`n=== 2. the client with the fresh mod ==="
$launcher = 'C:\Users\Kowge\Desktop\horizon-launcher.exe'
Start-Process -FilePath $launcher -ArgumentList 'play', '--instance', 'horizon', '--arg=--control', "--server=127.0.0.1:$port" -WindowStyle Hidden
$waited = 0
while (-not (Test-Path $sock) -and $waited -lt 180) { Start-Sleep -Seconds 3; $waited += 3 }
"control socket after ${waited}s"
Ctl @('wait-ready', '180') | Out-Null
Ctl @('status') | Out-Null

"`n=== 3. the limits of the shapes ==="
# A ball of 1024 blocks around: it was collected before it was checked.
Cmd 'a big ball is refused' '/we sphere stone 1024' 25
# The same ball with the limit in reach: still refused, with the limit named.
Cmd 'the limit is named' '/we limit blocks 1000' 10
Cmd 'the ball is refused by the limit' '/we sphere stone 100' 25
Cmd 'the limits are back' '/we limit reset' 10

"`n=== 4. the recursive fill ==="
# A depth of two billion: it was walked dy by dy.
Cmd 'the depth is bounded' '/we fillr stone 5 2147483647' 25

"`n=== 5. the trim ==="
# Two corners far apart: every block of the box was materialised.
Ctl @('command', 'we pos1 165,68,-80') | Out-Null
Start-Sleep -Seconds 1
Ctl @('command', 'we pos2 6000,300,6000') | Out-Null
Start-Sleep -Seconds 2
Cmd 'the trim is refused' '/we trim' 25

"`n=== 6. the smooth brush ==="
Ctl @('command', 'we pos1 165,68,-80') | Out-Null
Start-Sleep -Seconds 1
Ctl @('command', 'we pos2 170,70,-75') | Out-Null
Start-Sleep -Seconds 2
Cmd 'the brush is armed' '/we brush smooth 1000 1' 10
Ctl @('look-at-block', '165', '67', '-78') | Out-Null
Start-Sleep -Seconds 1
# The stroke of the brush: the limit has to stop it before it reads a block.
Ask 'the stroke is refused' @('mouse', 'left', 'click') 20

"`n=== 7. the line ==="
# In the dimension of the selection: it works.
Cmd 'the line works' '/we line stone' 20

"`n=== 8. the leftovers ==="
& $srv --dir $dir status

"`n=== 9. the shutdown ==="
Ctl @('quit') | Out-Null
Start-Sleep -Seconds 10
& $srv --dir $dir stop
