[CmdletBinding()]
param(
    [ValidateRange(1, 65535)]
    [int]$Port = 8721,

    [ValidateRange(1, 65535)]
    [int]$CandidatePort = 18749,

    [ValidateRange(0, 86400)]
    [int]$IdleTimeoutSeconds = 2400,

    [ValidateRange(1, 120)]
    [int]$HealthTimeoutSeconds = 10
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$workspace = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$targetRoot = [System.IO.Path]::GetFullPath((Join-Path $workspace 'target'))
$formalExe = [System.IO.Path]::GetFullPath((Join-Path $targetRoot 'release\agent-hub.exe'))
$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$candidateTarget = [System.IO.Path]::GetFullPath((Join-Path $targetRoot "deploy-local-$timestamp"))
$candidateExe = Join-Path $candidateTarget 'release\agent-hub.exe'
$backupExe = "$formalExe.pre-local-deploy-$timestamp"
$candidateStdout = Join-Path ([System.IO.Path]::GetTempPath()) "agent-hub-candidate-$timestamp.stdout.log"
$candidateStderr = Join-Path ([System.IO.Path]::GetTempPath()) "agent-hub-candidate-$timestamp.stderr.log"
$baseUrl = "http://127.0.0.1:$Port"
$candidateUrl = "http://127.0.0.1:$CandidatePort"
$candidateProcess = $null
$newProcess = $null
$switchStarted = $false
$deployed = $false

function Write-Step([string]$Message) {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss.fff')] $Message"
}

function Wait-HttpOk([string]$Uri, [int]$TimeoutSeconds) {
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        try {
            $response = Invoke-WebRequest -Uri $Uri -TimeoutSec 2 -SkipHttpErrorCheck
            if ([int]$response.StatusCode -eq 200) {
                return $true
            }
        }
        catch {
            # 服务切换期间连接失败是预期状态，继续短间隔轮询。
        }
        Start-Sleep -Milliseconds 100
    } while ((Get-Date) -lt $deadline)
    return $false
}

function Get-RunningCount([string]$RootUrl) {
    $runs = Invoke-RestMethod -Uri "$RootUrl/api/runs" -TimeoutSec 5
    if ($null -eq $runs) {
        return 0
    }
    return @(
        $runs | Where-Object {
            $null -ne $_ -and $_.PSObject.Properties['running'] -and $_.running -eq $true
        }
    ).Count
}

function Start-Hub([string]$Executable, [int]$ListenPort, [string]$Stdout = '', [string]$Stderr = '') {
    $previousPort = $env:AGENT_HUB_PORT
    $env:AGENT_HUB_PORT = $ListenPort.ToString()
    try {
        $arguments = @{
            FilePath = $Executable
            WorkingDirectory = $workspace
            WindowStyle = 'Hidden'
            PassThru = $true
        }
        if ($Stdout) {
            $arguments.RedirectStandardOutput = $Stdout
        }
        if ($Stderr) {
            $arguments.RedirectStandardError = $Stderr
        }
        return Start-Process @arguments
    }
    finally {
        if ($null -eq $previousPort) {
            Remove-Item Env:AGENT_HUB_PORT -ErrorAction SilentlyContinue
        }
        else {
            $env:AGENT_HUB_PORT = $previousPort
        }
    }
}

function Stop-OwnedProcess($Process) {
    if ($null -eq $Process) {
        return
    }
    $Process.Refresh()
    if (-not $Process.HasExited) {
        Stop-Process -Id $Process.Id -Force
        Wait-Process -Id $Process.Id -Timeout 5 -ErrorAction SilentlyContinue
    }
}

function Wait-ProcessExit([int]$ProcessId, [int]$TimeoutSeconds) {
    try {
        Wait-Process -Id $ProcessId -Timeout $TimeoutSeconds -ErrorAction Stop
    }
    catch {
        if (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue) {
            throw "进程 PID $ProcessId 未在 $TimeoutSeconds 秒内退出。"
        }
    }
}

function Wait-FileReplaceable([string]$Path, [int]$TimeoutSeconds) {
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        try {
            $stream = [System.IO.File]::Open(
                $Path,
                [System.IO.FileMode]::Open,
                [System.IO.FileAccess]::ReadWrite,
                [System.IO.FileShare]::None
            )
            $stream.Dispose()
            return
        }
        catch [System.IO.IOException] {
            if ((Get-Date) -ge $deadline) {
                throw "文件在 $TimeoutSeconds 秒内始终不可替换：$Path"
            }
            Start-Sleep -Milliseconds 50
        }
    } while ($true)
}

function Assert-PortFree([int]$ListenPort) {
    $listener = Get-NetTCPConnection -LocalPort $ListenPort -State Listen -ErrorAction SilentlyContinue
    if ($listener) {
        throw "候选端口 $ListenPort 已被占用。"
    }
}

function Get-FormalListener([int]$ListenPort, [string]$ExpectedExecutable) {
    $listeners = @(
        Get-NetTCPConnection -LocalAddress '127.0.0.1' -LocalPort $ListenPort -State Listen -ErrorAction SilentlyContinue
    )
    if ($listeners.Count -ne 1) {
        throw "端口 $ListenPort 应有且仅有 1 个 127.0.0.1 监听进程，实际为 $($listeners.Count) 个。"
    }
    $process = Get-Process -Id $listeners[0].OwningProcess
    $actualPath = [System.IO.Path]::GetFullPath($process.Path)
    if (-not $actualPath.Equals($ExpectedExecutable, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "端口 $ListenPort 由非预期进程监听：$actualPath"
    }
    return $process
}

if (-not $candidateTarget.StartsWith($targetRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "候选构建目录不在 target 内：$candidateTarget"
}
if ($Port -eq $CandidatePort) {
    throw '正式端口与候选端口不能相同。'
}
if (-not (Test-Path -LiteralPath $formalExe -PathType Leaf)) {
    throw "正式可执行文件不存在：$formalExe"
}
if (Test-Path -LiteralPath $backupExe) {
    throw "回滚备份已存在：$backupExe"
}

try {
    $idleDeadline = (Get-Date).AddSeconds($IdleTimeoutSeconds)
    do {
        $runningCount = Get-RunningCount $baseUrl
        if ($runningCount -eq 0) {
            break
        }
        if ((Get-Date) -ge $idleDeadline) {
            throw "等待空闲超时，仍有 $runningCount 个任务运行中。"
        }
        Write-Step "仍有 $runningCount 个任务运行，等待空闲。"
        Start-Sleep -Seconds 1
    } while ($true)

    Assert-PortFree $CandidatePort
    Write-Step '正式服务保持在线，开始隔离构建 release 候选。'
    & cargo build --release --target-dir $candidateTarget
    if ($LASTEXITCODE -ne 0) {
        throw "候选构建失败，cargo exit code: $LASTEXITCODE"
    }

    Write-Step "在 $CandidatePort 启动候选并冒烟验证。"
    $candidateProcess = Start-Hub $candidateExe $CandidatePort $candidateStdout $candidateStderr
    if (-not (Wait-HttpOk "$candidateUrl/api/status" $HealthTimeoutSeconds)) {
        throw "候选服务未在 $HealthTimeoutSeconds 秒内就绪。"
    }
    $null = Invoke-RestMethod -Uri "$candidateUrl/api/editors" -TimeoutSec 5
    $candidateApp = Invoke-WebRequest -Uri "$candidateUrl/app.js" -TimeoutSec 5
    if ([int]$candidateApp.StatusCode -ne 200) {
        throw "候选静态资源异常：HTTP $([int]$candidateApp.StatusCode)"
    }
    Stop-OwnedProcess $candidateProcess
    $candidateProcess = $null

    $formalProcess = Get-FormalListener $Port $formalExe
    $runningCount = Get-RunningCount $baseUrl
    if ($runningCount -ne 0) {
        throw "切换前发现 $runningCount 个任务运行中，取消部署。"
    }

    $candidateHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $candidateExe).Hash
    $oldHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $formalExe).Hash
    Copy-Item -LiteralPath $formalExe -Destination $backupExe
    Write-Step "候选验证通过；精确切换 PID $($formalProcess.Id)。"

    $switchWatch = [System.Diagnostics.Stopwatch]::StartNew()
    $switchStarted = $true
    Stop-Process -Id $formalProcess.Id -Force
    Wait-ProcessExit $formalProcess.Id 5
    Wait-FileReplaceable $formalExe 5
    Copy-Item -LiteralPath $candidateExe -Destination $formalExe -Force
    $newProcess = Start-Hub $formalExe $Port
    if (-not (Wait-HttpOk "$baseUrl/api/status" $HealthTimeoutSeconds)) {
        throw "新正式服务未在 $HealthTimeoutSeconds 秒内就绪。"
    }
    $switchWatch.Stop()

    $formalHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $formalExe).Hash
    if ($formalHash -ne $candidateHash) {
        throw "正式文件哈希与候选不一致：$formalHash != $candidateHash"
    }
    $deployed = $true
    Write-Step "部署完成：PID $($newProcess.Id)，切换至健康响应用时 $($switchWatch.ElapsedMilliseconds) ms。"
    Write-Step "SHA256: $oldHash -> $formalHash"
    Write-Step "回滚备份：$backupExe"
}
catch {
    if ($switchStarted -and -not $deployed -and (Test-Path -LiteralPath $backupExe -PathType Leaf)) {
        Write-Warning "部署切换失败，开始回滚：$($_.Exception.Message)"
        Stop-OwnedProcess $newProcess
        if (Wait-HttpOk "$baseUrl/api/status" 1) {
            $rollbackProcess = Get-FormalListener $Port $formalExe
            Write-Warning "旧正式服务仍健康，无需替换文件；PID $($rollbackProcess.Id)。"
        }
        else {
            Wait-FileReplaceable $formalExe 5
            Copy-Item -LiteralPath $backupExe -Destination $formalExe -Force
            $rollbackProcess = Start-Hub $formalExe $Port
            if (-not (Wait-HttpOk "$baseUrl/api/status" $HealthTimeoutSeconds)) {
                throw "部署失败且回滚服务未恢复：$($_.Exception.Message)"
            }
            Write-Warning "已回滚并恢复 PID $($rollbackProcess.Id)。"
        }
    }
    throw
}
finally {
    Stop-OwnedProcess $candidateProcess
    if (Test-Path -LiteralPath $candidateTarget) {
        & cargo clean --target-dir $candidateTarget | Out-Host
    }
    foreach ($logFile in @($candidateStdout, $candidateStderr)) {
        if (Test-Path -LiteralPath $logFile -PathType Leaf) {
            Remove-Item -LiteralPath $logFile -Force
        }
    }
}
