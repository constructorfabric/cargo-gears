Topic: FIPS Mode

FIPS (Federal Information Processing Standards) mode enables FIPS-compliant
cryptography in the generated server.

How it works:
  - The CLI passes -F fips to cargo build/run, enabling the fips Cargo feature
  - The modkit framework's fips feature activates FIPS-compliant TLS and crypto

Activation:
  CLI flag:      --fips / --no-fips
  Manifest:      [apps.myapp.dev.run] fips = true
  Priority:      CLI flag > manifest policy > default (false)

Examples:
  cargo gears run --app myapp --env dev --fips
  cargo gears build --app myapp --env prod --fips
