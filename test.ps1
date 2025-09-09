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

# Create keys for communication
cargo run -q -- keygen --out alice.key
cargo run -q -- keygen --out bob.key

# Create an id card for bob
Write-Output '{"name":"Bob"}' | cargo run -q -- id bob.key --out bob.id --meta -

# Create a message for bob using their id card and your own key
cargo run -q -- msg --key alice.key --to bob.id --out msg_to_bob.gxt --body '{"hello":"world"}'

# Verify if the message is valid and signed
cargo run -q -- verify --file msg_to_bob.gxt

# Decrypt the message using bobs key
cargo run -q -- decrypt --key bob.key --file msg_to_bob.gxt
