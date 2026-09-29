# Tahap 0 spike: probe Windows GSMTC (Global System Media Transport Controls).
# Lists every media session Windows knows about (Spotify, Chrome, Edge, Firefox, ...)
# and, with -Watch, samples the timeline to see how often Position is refreshed.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File spike\gsmtc-probe.ps1
#   powershell -ExecutionPolicy Bypass -File spike\gsmtc-probe.ps1 -Watch -Seconds 20

param(
    [switch]$Watch,
    [int]$Seconds = 15
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Runtime.WindowsRuntime

$asTaskGeneric = [System.WindowsRuntimeSystemExtensions].GetMethods() |
    Where-Object {
        $_.Name -eq 'AsTask' -and
        $_.GetParameters().Count -eq 1 -and
        $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
    } | Select-Object -First 1

function Await($op, [Type]$resultType) {
    $task = $asTaskGeneric.MakeGenericMethod($resultType).Invoke($null, @($op))
    $task.Wait(-1) | Out-Null
    $task.Result
}

$null = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$ManagerType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]
$PropsType = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties]

$mgr = Await ($ManagerType::RequestAsync()) $ManagerType

function Format-Ms([double]$ms) {
    $ts = [TimeSpan]::FromMilliseconds([Math]::Max(0, $ms))
    '{0:00}:{1:00}.{2:000}' -f [Math]::Floor($ts.TotalMinutes), $ts.Seconds, $ts.Milliseconds
}

function Get-Snapshot($session) {
    $props = Await ($session.TryGetMediaPropertiesAsync()) $PropsType
    $tl = $session.GetTimelineProperties()
    $pb = $session.GetPlaybackInfo()
    $now = [DateTimeOffset]::Now
    $playing = "$($pb.PlaybackStatus)" -eq 'Playing'
    $reportedMs = $tl.Position.TotalMilliseconds
    $ageMs = ($now - $tl.LastUpdatedTime).TotalMilliseconds
    $interpMs = if ($playing) { $reportedMs + $ageMs } else { $reportedMs }
    [pscustomobject]@{
        App        = $session.SourceAppUserModelId
        Status     = "$($pb.PlaybackStatus)"
        Title      = $props.Title
        Artist     = $props.Artist
        Album      = $props.AlbumTitle
        DurationMs = ($tl.EndTime - $tl.StartTime).TotalMilliseconds
        ReportedMs = $reportedMs
        AgeMs      = $ageMs
        InterpMs   = $interpMs
        Updated    = $tl.LastUpdatedTime
    }
}

$current = $mgr.GetCurrentSession()
$sessions = @($mgr.GetSessions())
if ($sessions.Count -eq 0) {
    Write-Host 'Tidak ada media session. Putar lagu di Spotify / YouTube / YT Music dulu, lalu jalankan lagi.'
    exit 1
}

Write-Host "Ditemukan $($sessions.Count) session. Current = $($current.SourceAppUserModelId)`n"
foreach ($s in $sessions) {
    $snap = Get-Snapshot $s
    Write-Host ("[{0}] {1}" -f $snap.Status, $snap.App)
    Write-Host ("   Title   : {0}" -f $snap.Title)
    Write-Host ("   Artist  : {0}" -f $snap.Artist)
    Write-Host ("   Album   : {0}" -f $snap.Album)
    Write-Host ("   Durasi  : {0}" -f (Format-Ms $snap.DurationMs))
    Write-Host ("   Posisi  : reported {0} (umur {1:N0} ms) -> interpolasi {2}" -f (Format-Ms $snap.ReportedMs), $snap.AgeMs, (Format-Ms $snap.InterpMs))
    Write-Host ''
}

if (-not $Watch) { exit 0 }

$target = $sessions | Where-Object { "$($_.GetPlaybackInfo().PlaybackStatus)" -eq 'Playing' } | Select-Object -First 1
if (-not $target) { $target = $current }
Write-Host "Watch $Seconds detik pada: $($target.SourceAppUserModelId)"
Write-Host 'Kolom: waktu-dinding | reported | interpolasi | drift-vs-jam-dinding (ms) | ada update baru?'

$start = [DateTimeOffset]::Now
$first = Get-Snapshot $target
$lastUpdated = $first.Updated
$updates = 0
for ($i = 0; $i -lt $Seconds * 2; $i++) {
    Start-Sleep -Milliseconds 500
    $snap = Get-Snapshot $target
    $wallMs = ([DateTimeOffset]::Now - $start).TotalMilliseconds
    $expected = $first.InterpMs + $wallMs
    $drift = $snap.InterpMs - $expected
    $isNew = $snap.Updated -ne $lastUpdated
    if ($isNew) { $updates++; $lastUpdated = $snap.Updated }
    Write-Host ("{0,7:N1}s | {1} | {2} | {3,7:N0} | {4}" -f ($wallMs / 1000), (Format-Ms $snap.ReportedMs), (Format-Ms $snap.InterpMs), $drift, $(if ($isNew) { 'UPDATE' } else { '' }))
}
Write-Host "`nTimeline di-update $updates kali dalam $Seconds detik."
