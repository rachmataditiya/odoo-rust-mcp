# Odoo Rust MCP Server Installer for Windows
# Run as Administrator from extracted release directory:
#   .\install.ps1              # Install binary only
#   .\install.ps1 -Service     # Install as Windows Service
#   .\install.ps1 -Uninstall   # Uninstall everything

param(
    [switch]$Uninstall,
    [switch]$Service,
    [switch]$ServiceUninstall,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

$BinaryName = "rust-mcp"
$ServiceName = "OdooRustMcp"
$ServiceDisplayName = "Odoo Rust MCP Server"
$InstallDir = "$env:ProgramFiles\odoo-rust-mcp"
$ConfigDir = "$env:ProgramData\odoo-rust-mcp"
$EnvFile = "$ConfigDir\env.ps1"
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
    Write-Host "Usage:"
    Write-Host "  Run directly (stdio):  $BinaryName --transport stdio"
    Write-Host "  Run as HTTP server:    $BinaryName --transport http --listen 127.0.0.1:8787"
    Write-Host "  Install as service:    .\install.ps1 -Service"
    Write-Host ""
    Write-Host "For Cursor/Claude Desktop, see README for configuration examples."
}

function Uninstall-OdooMcp {
    Write-Info "Uninstalling odoo-rust-mcp..."

    # Stop and remove service first
    Uninstall-Service -Silent

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
        $items = Get-ChildItem $InstallDir -ErrorAction SilentlyContinue
        if ($null -eq $items -or $items.Count -eq 0) {
            Remove-Item $InstallDir -Force -ErrorAction SilentlyContinue
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

function Install-Service {
    Write-Info "Installing Windows Service..."

    # First ensure binary is installed
    if (-not (Test-Path "$InstallDir\$BinaryName.exe")) {
        Install-OdooMcp
    }

    # Create environment file if not exists
    if (-not (Test-Path $EnvFile)) {
        Write-Info "Creating environment file at $EnvFile..."
        @'
# Odoo MCP Server Environment Configuration
# Edit this file with your Odoo credentials
# This file is sourced by the Windows Service

# Odoo 19+ (API Key authentication)
$env:ODOO_URL = "http://localhost:8069"
$env:ODOO_DB = "mydb"
$env:ODOO_API_KEY = "YOUR_API_KEY"

# Odoo < 19 (Username/Password authentication)
# $env:ODOO_URL = "http://localhost:8069"
# $env:ODOO_DB = "mydb"
# $env:ODOO_VERSION = "18"
# $env:ODOO_USERNAME = "admin"
# $env:ODOO_PASSWORD = "admin"

# MCP Authentication (HTTP transport)
# Generate a secure token in PowerShell: [Convert]::ToBase64String((1..32 | ForEach-Object { Get-Random -Maximum 256 }))
$env:MCP_AUTH_TOKEN = "CHANGE_ME_TO_A_SECURE_TOKEN"

# MCP Config paths
$env:MCP_TOOLS_JSON = "C:\ProgramData\odoo-rust-mcp\tools.json"
$env:MCP_PROMPTS_JSON = "C:\ProgramData\odoo-rust-mcp\prompts.json"
$env:MCP_SERVER_JSON = "C:\ProgramData\odoo-rust-mcp\server.json"
'@ | Out-File -FilePath $EnvFile -Encoding UTF8
        Write-Warn "Please edit $EnvFile with your Odoo credentials and MCP_AUTH_TOKEN"
    }

    # Create wrapper script for service
    $wrapperScript = "$InstallDir\service-wrapper.ps1"
    @"
# Service wrapper script
. "$EnvFile"
& "$InstallDir\$BinaryName.exe" --transport http --listen 127.0.0.1:8787
"@ | Out-File -FilePath $wrapperScript -Encoding UTF8

    # Check if service exists
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($existingService) {
        Write-Info "Stopping existing service..."
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Write-Info "Removing existing service..."
        sc.exe delete $ServiceName | Out-Null
        Start-Sleep -Seconds 2
    }

    # Create Windows Service using NSSM or sc.exe with PowerShell wrapper
    # Using Task Scheduler as a simpler alternative for background service
    $taskName = "OdooRustMcpService"
    
    # Remove existing task
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

    # Create scheduled task that runs at startup
    $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-ExecutionPolicy Bypass -WindowStyle Hidden -File `"$wrapperScript`""
    $trigger = New-ScheduledTaskTrigger -AtStartup
    $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)

    Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Description $ServiceDisplayName | Out-Null

    # Start the task now
    Start-ScheduledTask -TaskName $taskName

    Write-Info "Windows Service installed and started!"
    Write-Host ""
    Write-Host "Commands (PowerShell as Admin):"
    Write-Host "  Start:   Start-ScheduledTask -TaskName $taskName"
    Write-Host "  Stop:    Stop-ScheduledTask -TaskName $taskName"
    Write-Host "  Status:  Get-ScheduledTask -TaskName $taskName | Select-Object State"
    Write-Host ""
    Write-Host "Service listens on: http://127.0.0.1:8787/mcp"
    Write-Warn "Don't forget to edit $EnvFile with your Odoo credentials!"
}

function Uninstall-Service {
    param([switch]$Silent)
    
    if (-not $Silent) { Write-Info "Removing Windows Service..." }

    $taskName = "OdooRustMcpService"
    
    # Stop and remove scheduled task
    Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

    # Remove wrapper script
    $wrapperScript = "$InstallDir\service-wrapper.ps1"
    if (Test-Path $wrapperScript) {
        Remove-Item $wrapperScript -Force -ErrorAction SilentlyContinue
    }

    if (-not $Silent) { Write-Info "Windows Service removed" }
}

function Show-Help {
    Write-Host "Odoo Rust MCP Server Installer for Windows"
    Write-Host ""
    Write-Host "Usage: .\install.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  (none)            Install binary and config files (default)"
    Write-Host "  -Uninstall        Uninstall binary, config, and service"
    Write-Host "  -Service          Install and start as background service"
    Write-Host "  -ServiceUninstall Remove background service only"
    Write-Host "  -Help             Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install.ps1              # Install binary only"
    Write-Host "  .\install.ps1 -Service     # Install and run as service"
    Write-Host "  .\install.ps1 -Uninstall   # Remove everything"
}

# Check for admin privileges
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Err "This script requires Administrator privileges. Please run as Administrator."
}

# Main
if ($Help) {
    Show-Help
} elseif ($Uninstall) {
    Uninstall-OdooMcp
} elseif ($Service) {
    Install-Service
} elseif ($ServiceUninstall) {
    Uninstall-Service
} else {
    Install-OdooMcp
}
