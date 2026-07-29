$ErrorActionPreference = 'Stop'

$checker = Join-Path $PSScriptRoot 'hygiene.py'
python $checker
exit $LASTEXITCODE
