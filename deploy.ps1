# Peroperochat Deployment & Sync Script
# This script helps to quickly sync local changes to GitHub

Write-Host "🚀 Starting sync to GitHub..." -ForegroundColor Cyan

# Check if .git exists
if (!(Test-Path .git)) {
    Write-Host "📦 Initializing Git repository..." -ForegroundColor Yellow
    git init
}

# Check if remote exists, if not add it
$remote = git remote get-url origin 2>$null
if ($null -eq $remote) {
    Write-Host "🔗 Adding remote origin..." -ForegroundColor Yellow
    git remote add origin https://github.com/YoKONCy/Peroperochat
} else {
    Write-Host "✅ Remote origin already exists: $remote" -ForegroundColor Green
}

# Add all changes
Write-Host "📝 Staging changes..." -ForegroundColor Yellow
git add .

# Commit
$commitMsg = "Sync: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - Auto-sync via deploy script"
Write-Host "💾 Committing changes: $commitMsg" -ForegroundColor Yellow
git commit -m $commitMsg

# Branch check (default to main)
$branch = git branch --show-current
if ($branch -ne "main") {
    Write-Host "🌿 Switching to main branch..." -ForegroundColor Yellow
    git branch -M main
}

# Push
Write-Host "☁️ Pushing to GitHub..." -ForegroundColor Yellow
git push -u origin main

if ($LASTEXITCODE -eq 0) {
    Write-Host "✨ Successfully synced to GitHub!" -ForegroundColor Green
} else {
    Write-Host "❌ Error: Push failed. Please check your network or GitHub permissions." -ForegroundColor Red
}
