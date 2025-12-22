$InstallDir = "$Env:LOCALAPPDATA\Programs\fancy-tree"

$RepoDomain = "github.com"
$RepoOwner = "spenserblack"
$RepoName = "fancy-tree"
$Repo = "$RepoDomain/$RepoOwner/$RepoName"

$Os = "Windows"

if ($Env:PROCESSOR_ARCHITECTURE -eq "AMD64") {
    $Arch = "X64"
}
else {
    Write-Error "Unknown processor architecture: $Env:PROCESSOR_ARCHITECTURE"
    exit 1
}

$AssetName = "fancy-tree-$Os-$Arch.zip"

$Url = "https://$Repo/releases/latest/download/$AssetName"

$TempDir = [System.IO.Path]::GetTempPath()
$DownloadPath = "$TempDir" + "fancytree.zip"

Write-Output "Downloading to $DownloadPath..."
Invoke-WebRequest -Uri $Url -OutFile "$DownloadPath"

Write-Output "Creating $InstallDir..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Output "Unzipping $DownloadPath to $InstallDir..."
Expand-Archive -Path $DownloadPath -DestinationPath $InstallDir

$UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding to PATH..."
    [System.Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    $Env:PATH += ";$InstallDir"
}

Write-Host "Cleaning up $DownloadPath..."
Remove-Item -Path $DownloadPath

Write-Host "Done!"
Write-Host "You may need to restart your terminal"
