param([switch]$Pause)

function Print {
    param (
        [string]$Text
    )
    if ($Pause) {
        Read-Host -Prompt "$Text Press enter to continue..." | Out-Null
    }
    else {
        Write-Host "$Text"
    }
}

$path = "tmp"
If (!(test-path -PathType container $path)) {
    New-Item -ItemType Directory -Path $path | Out-Null
}

Remove-Item "$path/*"

cargo build

cargo run -q -- keygen -o tmp/alice.key
cargo run -q -- keygen -o tmp/bob.key

Write-Host "make id"
Write-Output '{"name":"Bob"}' | cargo run -q -- id tmp/bob.key -o tmp/bob.id

Write-Host "verify id"
cargo run -q -- verify tmp/bob.id

Write-Host "make message"
Write-Output '{"hello":"world"}' | cargo run -q -- msg tmp/alice.key tmp/bob.id -o tmp/msg_to_bob.gxt

Write-Host "verify message"
cargo run -q -- verify tmp/msg_to_bob.gxt

Write-Host "decrypt message"
cargo run -q -- decrypt tmp/bob.key tmp/msg_to_bob.gxt
