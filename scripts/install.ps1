# Odoo Rust MCP Server Installer for Windows
# Run as Administrator from extracted release directory: .\install.ps1

param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$BinaryName = "rust-mcp"
$InstallDir = "$env:ProgramFiles\odoo-rust-mcp"
$ConfigDir = "$env:ProgramData\odoo-rust-mcp"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Err { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }

function Install-OdooMcp {
    Write-Info "Installing odoo-rust-mcp..."

    # Check if binary exists in current directory
    if (-not (Test-Path "$ScriptDir\$BinaryName.exe")) {
        Write-Err "Binary '$BinaryName.exe' not found in $ScriptDir. Make sure you're running from the extracted release directory."
    }

    Write-Info "Installing binary to $InstallDir..."
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item "$ScriptDir\$BinaryName.exe" -Destination $InstallDir -Force

    Write-Info "Installing config files to $ConfigDir..."
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    if (Test-Path "$ScriptDir\config") {
        Copy-Item "$ScriptDir\config\*" -Destination $ConfigDir -Recurse -Force
    }
    if (Test-Path "$ScriptDir\.env.example") {
        Copy-Item "$ScriptDir\.env.example" -Destination $ConfigDir -Force
    }

    # Add to PATH if not already there
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($currentPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to system PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "Machine")
    }

    Write-Info "Installation complete!"
    Write-Host ""
    Write-Host "Binary installed to: $InstallDir\$BinaryName.exe"
    Write-Host "Config files installed to: $ConfigDir"
    Write-Host ""
    Write-Host "Quick start:"
    Write-Host "  1. Copy and edit the example environment file:"
    Write-Host "     Copy-Item '$ConfigDir\.env.example' '$env:USERPROFILE\.odoo-mcp.env'"
    Write-Host "     # Edit the file with your Odoo credentials"
    Write-Host ""
    Write-Host "  2. Run the server (open new terminal first for PATH update):"
    Write-Host "     $BinaryName --transport stdio"
    Write-Host ""
    Write-Host "  3. For Cursor, add to %APPDATA%\Cursor\User\globalStorage\cursor.mcp\mcp.json:"
    Write-Host '     {'
    Write-Host '       "mcpServers": {'
    Write-Host '         "odoo-rust-mcp": {'
    Write-Host '           "type": "stdio",'
    Write-Host "           `"command`": `"$InstallDir\$BinaryName.exe`","
    Write-Host '           "args": ["--transport", "stdio"],'
    Write-Host '           "env": {'
    Write-Host '             "ODOO_URL": "http://localhost:8069",'
    Write-Host '             "ODOO_DB": "mydb",'
    Write-Host '             "ODOO_API_KEY": "YOUR_API_KEY",'
    Write-Host "             `"MCP_TOOLS_JSON`": `"$ConfigDir\tools.json`","
    Write-Host "             `"MCP_PROMPTS_JSON`": `"$ConfigDir\prompts.json`","
    Write-Host "             `"MCP_SERVER_JSON`": `"$ConfigDir\server.json`""
    Write-Host '           }'
    Write-Host '         }'
    Write-Host '       }'
    Write-Host '     }'
}

function Uninstall-OdooMcp {
    Write-Info "Uninstalling odoo-rust-mcp..."

    if (Test-Path "$InstallDir\$BinaryName.exe") {
        Remove-Item "$InstallDir\$BinaryName.exe" -Force
        Write-Info "Removed $InstallDir\$BinaryName.exe"
    }

    # Remove from PATH
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($currentPath -like "*$InstallDir*") {
        $newPath = ($currentPath -split ';' | Where-Object { $_ -ne $InstallDir }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
        Write-Info "Removed $InstallDir from system PATH"
    }

    if (Test-Path $InstallDir) {
        $items = Get-ChildItem $InstallDir
        if ($items.Count -eq 0) {
            Remove-Item $InstallDir -Force
            Write-Info "Removed empty directory $InstallDir"
        }
    }

    if (Test-Path $ConfigDir) {
        $response = Read-Host "Remove config directory $ConfigDir? [y/N]"
        if ($response -eq 'y' -or $response -eq 'Y') {
            Remove-Item $ConfigDir -Recurse -Force
            Write-Info "Removed $ConfigDir"
        }
    }

    Write-Info "Uninstall complete!"
}

# Check for admin privileges
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Err "This script requires Administrator privileges. Please run as Administrator."
}

if ($Uninstall) {
    Uninstall-OdooMcp
} else {
    Install-OdooMcp
}
