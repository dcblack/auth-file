$checkVersionOptions = @(
    '-h'
    '--help'
    '--debug'
    '--use-dots'
    '-nm'
    '--no-message'
    '--show'
    '-s'
    '--silent'
    '-v'
    '--verbose'
)

Register-ArgumentCompleter -Native -CommandName 'check-version.py' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    if ($wordToComplete -like '-*') {
        $checkVersionOptions |
            Where-Object { $_ -like "$wordToComplete*" } |
            ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
            }
    }
}