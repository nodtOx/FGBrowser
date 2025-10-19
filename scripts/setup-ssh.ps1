# Setup SSH Server on Windows
# This script must be run as Administrator

Write-Host "Setting up SSH Server on Windows..." -ForegroundColor Green
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "ERROR: This script must be run as Administrator!" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select 'Run as Administrator', then run this script again." -ForegroundColor Yellow
    exit 1
}

# Step 1: Check and install OpenSSH Server
Write-Host "Step 1: Checking OpenSSH Server installation..." -ForegroundColor Cyan
$sshServer = Get-WindowsCapability -Online | Where-Object Name -like 'OpenSSH.Server*'

if ($sshServer.State -eq "Installed") {
    Write-Host "  OpenSSH Server is already installed." -ForegroundColor Green
} else {
    Write-Host "  Installing OpenSSH Server..." -ForegroundColor Yellow
    Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0
    Write-Host "  OpenSSH Server installed successfully." -ForegroundColor Green
}
Write-Host ""

# Step 2: Start SSH service
Write-Host "Step 2: Starting SSH service..." -ForegroundColor Cyan
try {
    Start-Service sshd
    Write-Host "  SSH service started successfully." -ForegroundColor Green
} catch {
    Write-Host "  SSH service is already running or error occurred: $_" -ForegroundColor Yellow
}
Write-Host ""

# Step 3: Set SSH service to start automatically
Write-Host "Step 3: Setting SSH service to start automatically..." -ForegroundColor Cyan
Set-Service -Name sshd -StartupType 'Automatic'
Write-Host "  SSH service set to start automatically on boot." -ForegroundColor Green
Write-Host ""

# Step 4: Configure firewall
Write-Host "Step 4: Configuring firewall..." -ForegroundColor Cyan
$firewallRule = Get-NetFirewallRule -Name "sshd" -ErrorAction SilentlyContinue

if ($firewallRule) {
    Write-Host "  Firewall rule already exists." -ForegroundColor Green
} else {
    Write-Host "  Creating firewall rule for SSH..." -ForegroundColor Yellow
    New-NetFirewallRule -Name sshd -DisplayName 'OpenSSH Server (sshd)' -Enabled True -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22
    Write-Host "  Firewall rule created successfully." -ForegroundColor Green
}
Write-Host ""

# Step 5: Display connection information
Write-Host "Step 5: Connection Information" -ForegroundColor Cyan
Write-Host "  SSH service status:" -ForegroundColor White
Get-Service sshd | Select-Object Name, Status, StartType | Format-Table
Write-Host ""

Write-Host "  Your IP addresses:" -ForegroundColor White
$ipAddresses = Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.InterfaceAlias -notlike '*Loopback*'} | Select-Object IPAddress, InterfaceAlias
$ipAddresses | Format-Table

Write-Host ""
Write-Host "SETUP COMPLETE!" -ForegroundColor Green
Write-Host ""
Write-Host "You can now connect from other computers using:" -ForegroundColor Yellow
Write-Host "  ssh $env:USERNAME@<IP_ADDRESS>" -ForegroundColor White
Write-Host ""
Write-Host "Example:" -ForegroundColor Yellow
foreach ($ip in $ipAddresses) {
    Write-Host "  ssh $env:USERNAME@$($ip.IPAddress)" -ForegroundColor White
}
Write-Host ""

