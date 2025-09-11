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

$env:Path = ".\target\debug\;" + $env:Path

cargo build -p gxt-cli

# Create keys for communication
cargo run -p gxt-cli -q -- keygen --out tmp/alice.key
cargo run -p gxt-cli -q -- keygen --out tmp/bob.key

# Create an id card for bob
Write-Output '{"name":"Bob"}' | cargo run -p gxt-cli -q -- id tmp/bob.key --out tmp/bob.id --meta -

# Verify if the id card is valid and signed
cargo run -p gxt-cli -q -- verify --file tmp/bob.id

# Create a message for bob using their id card and your own key
cargo run -p gxt-cli -q -- msg --key tmp/alice.key --to tmp/bob.id --out tmp/msg_to_bob.gxt --payload '{"hello":"world"}'

# Verify if the message is valid and signed
cargo run -p gxt-cli -q -- verify --file tmp/msg_to_bob.gxt

# Decrypt the message using bobs key
cargo run -p gxt-cli -q -- decrypt --key tmp/bob.key --file tmp/msg_to_bob.gxt
