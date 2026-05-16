$authOptions = @(
    '--cache-time='
    '--change-password'
    '-ck'
    '--check'
    '--color'
    '-d'
    '--dir'
    '-f'
    '--force'
    '-h'
    '--help'
    '-q'
    '--quiet'
    '--request-password'
    '--root-dir='
    '--show-dir'
    '-s'
    '--silent'
    '--stats'
    '-v'
    '--verbose'
    '--version'
    '-wr'
    '--write'
    '-rm'
    '--remove'
)

$authColorModes = @('auto', 'always', 'never')
$authCacheTimes = @('0', '15', '30', '60', '120')

Register-ArgumentCompleter -Native -CommandName 'auth' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $elements = @($commandAst.CommandElements | ForEach-Object { $_.Extent.Text })
    $previous = if ($elements.Count -ge 2) { $elements[-2] } else { '' }

    switch ($previous) {
        '--color' {
            $authColorModes |
                Where-Object { $_ -like "$wordToComplete*" } |
                ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            return
        }
        '-d' { Get-ChildItem -Directory -Name "$wordToComplete*" | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ProviderContainer', $_) }; return }
        '--dir' { Get-ChildItem -Directory -Name "$wordToComplete*" | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ProviderContainer', $_) }; return }
    }

    if ($wordToComplete -like '--color=*') {
        $prefix = '--color='
        $value = $wordToComplete.Substring($prefix.Length)
        $authColorModes |
            Where-Object { $_ -like "$value*" } |
            ForEach-Object {
                $completion = "$prefix$_"
                [System.Management.Automation.CompletionResult]::new($completion, $completion, 'ParameterValue', $completion)
            }
        return
    }

    if ($wordToComplete -like '--cache-time=*') {
        $prefix = '--cache-time='
        $value = $wordToComplete.Substring($prefix.Length)
        $authCacheTimes |
            Where-Object { $_ -like "$value*" } |
            ForEach-Object {
                $completion = "$prefix$_"
                [System.Management.Automation.CompletionResult]::new($completion, $completion, 'ParameterValue', $completion)
            }
        return
    }

    if ($wordToComplete -like '--root-dir=*') {
        $prefix = '--root-dir='
        $value = $wordToComplete.Substring($prefix.Length)
        Get-ChildItem -Directory -Name "$value*" |
            ForEach-Object {
                $completion = "$prefix$_"
                [System.Management.Automation.CompletionResult]::new($completion, $completion, 'ProviderContainer', $completion)
            }
        return
    }

    if ($wordToComplete -like '-*') {
        $authOptions |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
            }
        return
    }

    Get-ChildItem -Name "$wordToComplete*" |
        ForEach-Object {
            $type = if (Test-Path $_ -PathType Container) { 'ProviderContainer' } else { 'ProviderItem' }
            [System.Management.Automation.CompletionResult]::new($_, $_, $type, $_)
        }
}