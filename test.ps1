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

$ErrorActionPreference = "Stop"

$path = "tmp"
If (!(test-path -PathType container $path)) {
    New-Item -ItemType Directory -Path $path | Out-Null
}

Remove-Item "$path/*"

$env:Path = ".\target\debug\;" + $env:Path

cargo build -p gxt-cli

Write-Host "Create keys..."
# Create keys for communication
cargo run -p gxt-cli -q -- keygen --out tmp/alice.key.gxt
cargo run -p gxt-cli -q -- keygen --out tmp/bob.key.gxt

Write-Host ""
Write-Host "Create ID Card for Bob..."

# Create an id card for bob
Write-Output '{"name":"Bob"}' | cargo run -p gxt-cli -q -- id tmp/bob.key.gxt --out tmp/bob.id.gxt --meta -

Write-Host ""
Write-Host "Verify ID Card for Bob..."

# Verify if the id card is valid and signed
cargo run -p gxt-cli -q -- verify --file tmp/bob.id.gxt > tmp/bob.id.verified.gxt

Write-Host ""
Write-Host "Create Message for Bob..."

# Create a message for bob using their id card and your own key
cargo run -p gxt-cli -q -- msg --key tmp/alice.key.gxt --to tmp/bob.id.gxt --out tmp/msg_to_bob.gxt --payload '{"hello":"world"}'

Write-Host ""
Write-Host "Verify Message..."

# Verify if the message is valid and signed
cargo run -p gxt-cli -q -- verify --file tmp/msg_to_bob.gxt > tmp/msg_to_bob.gxt.verified

Write-Host ""
Write-Host "Decrypt Message with Bobs Key..."

# Decrypt the message using bobs key
cargo run -p gxt-cli -q -- decrypt --key tmp/bob.key.gxt --file tmp/msg_to_bob.gxt > tmp/msg_to_bob.decrypted