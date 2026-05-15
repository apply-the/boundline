# PowerShell Guidelines

## Principi

PowerShell lavora con oggetti, non solo testo. Scrivere PowerShell come se fosse Bash porta a script fragili. Usare parametri tipizzati, pipeline di oggetti e gestione errori esplicita.

## Parametri tipizzati

```powershell
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$TargetPath,

    [Parameter(Mandatory = $false)]
    [switch]$Force
)
```

Validare input con attributi quando possibile.

## Error handling

Impostare comportamento esplicito.

```powershell
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
```

Usare `try/catch/finally`.

```powershell
try {
    Copy-Item -Path $Source -Destination $Destination -ErrorAction Stop
}
catch {
    throw "Failed to copy item from '$Source' to '$Destination': $($_.Exception.Message)"
}
```

## Cleanup

Usare `finally` per cleanup.

```powershell
$tempFile = New-TemporaryFile

try {
    # Work with the temporary file
}
finally {
    Remove-Item -Path $tempFile.FullName -Force -ErrorAction SilentlyContinue
}
```

## Oggetti, non testo

Restituire oggetti dalla pipeline.

### Da evitare

```powershell
Write-Output "$Name,$Status"
```

### Preferibile

```powershell
[pscustomobject]@{
    Name = $Name
    Status = $Status
}
```

## Verbi approvati

Usare verbi PowerShell approvati per funzioni pubbliche.

```powershell
function Get-OrderStatus {
    param([string]$OrderId)
}
```

Evitare nomi ambigui tipo `ProcessStuff`.

## ShouldProcess

Per funzioni distruttive, supportare `-WhatIf` e `-Confirm`.

```powershell
function Remove-OrderCache {
    [CmdletBinding(SupportsShouldProcess)]
    param([string]$Path)

    if ($PSCmdlet.ShouldProcess($Path, "Remove cache")) {
        Remove-Item -Path $Path -Recurse -Force
    }
}
```

## Secrets

Non scrivere segreti nei log. Usare SecretManagement, variabili sicure o meccanismi del runtime.

```powershell
[securestring]$Password
```

## Logging

Usare stream appropriati.

- `Write-Verbose` per dettagli diagnostici
- `Write-Warning` per warning
- `Write-Error` per errori
- output normale solo per dati della pipeline

Non usare `Write-Host` per dati che devono essere consumati da altri comandi.

## Testabilità

Separare funzioni pure da comandi che fanno I/O.

```powershell
function ConvertTo-OrderSummary {
    param([object]$Order)

    [pscustomobject]@{
        Id = $Order.Id
        Total = $Order.Total
    }
}
```

Usare Pester per test.

## Cose da evitare

- trattare tutto come testo
- `Write-Host` per output consumabile
- errori non terminating ignorati
- parametri non validati
- funzioni senza `CmdletBinding`
- path concatenati come stringhe invece di `Join-Path`
- segreti in transcript o log
- `Invoke-Expression` con input dinamico
