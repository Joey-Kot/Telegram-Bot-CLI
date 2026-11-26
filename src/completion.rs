use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};

use clap_complete::{CompletionCandidate, Shell};

const BIN: &str = env!("CARGO_PKG_NAME");

pub fn write_registration(shell: Shell) -> io::Result<()> {
    let script = match shell {
        Shell::Bash => BASH,
        Shell::Zsh => ZSH,
        Shell::Fish => FISH,
        Shell::PowerShell => POWERSHELL,
        Shell::Elvish => ELVISH,
        _ => return Err(io::Error::other("unsupported shell")),
    };
    io::stdout()
        .lock()
        .write_all(script.replace("@BIN@", BIN).as_bytes())
}

// Read NUL-delimited input from a pipe so typed values stay out of process arguments.
pub fn serve(
    candidates: impl FnOnce(Vec<OsString>, usize) -> io::Result<Vec<CompletionCandidate>>,
) -> io::Result<bool> {
    let mut request = std::env::args_os().skip(1);
    if request.next().as_deref() != Some(OsStr::new("--_complete")) {
        return Ok(false);
    }

    let mode = request.next().unwrap_or_default();
    if request.next().is_some() || (mode != "bash" && mode != "words" && mode != "fish") {
        return Ok(true);
    }

    let mut input = Vec::new();
    io::stdin().lock().read_to_end(&mut input)?;
    let input = String::from_utf8_lossy(&input);
    // PowerShell adds a newline to the pipe after the final delimiter.
    let Some((input, _)) = input.rsplit_once('\0') else {
        return Ok(true);
    };
    let fields = input.split('\0');
    let (words, prefix) = if mode == "bash" {
        let Some((line, fragment)) = input.split_once('\0') else {
            return Ok(true);
        };
        let Some(input) = bash_input(line, fragment) else {
            return Ok(true);
        };
        input
    } else {
        (fields.map(OsString::from).collect(), String::new())
    };

    if words.len() >= 2 {
        let index = words.len() - 1;
        let current = words[index].to_string_lossy();
        let inline_prefix = current
            .strip_prefix("--")
            .and_then(|option| option.split_once('='))
            .map(|(name, _)| current[..name.len() + 3].to_owned())
            .unwrap_or_default();
        let expand_home = mode != "fish" && current[inline_prefix.len()..].starts_with('~');
        let mut stdout = io::stdout().lock();
        for candidate in candidates(words, index)? {
            let value = printable_candidate(&candidate, &inline_prefix, expand_home);
            if let Some(value) = value.strip_prefix(&prefix)
                && !value.is_empty()
                && !value.chars().any(char::is_control)
            {
                writeln!(stdout, "{value}")?;
            }
        }
    }
    Ok(true)
}

fn printable_candidate<'a>(
    candidate: &'a CompletionCandidate,
    inline_prefix: &str,
    expand_home: bool,
) -> Cow<'a, str> {
    let value = candidate.get_value().to_string_lossy();
    let (argument, suffix) = value
        .strip_prefix(inline_prefix)
        .map_or(("", value.as_ref()), |suffix| (inline_prefix, suffix));

    if expand_home
        && let Some(relative) = suffix
            .strip_prefix("~/")
            .or_else(|| cfg!(windows).then(|| suffix.strip_prefix("~\\")).flatten())
        && let Some(home) = std::env::home_dir()
    {
        let relative = relative.trim_start_matches(std::path::is_separator);
        return format!("{argument}{}", home.join(relative).display()).into();
    }

    if argument.is_empty()
        && value.starts_with('-')
        && value != "-"
        && candidate
            .get_tag()
            .is_some_and(|tag| *tag == PATH_TAG.into())
    {
        return format!("./{value}").into();
    }
    value
}

fn bash_input(line: &str, fragment: &str) -> Option<(Vec<OsString>, String)> {
    let words = bash_words(line)?;
    let current = words.last()?;
    // Readline inserts only the portion after a word break such as '=' or ':'.
    let prefix = current
        .strip_suffix(fragment)
        .or_else(|| {
            let fragment = bash_words(fragment)?.pop()?;
            current.strip_suffix(&fragment)
        })
        .filter(|prefix| prefix.ends_with(['=', ':', '@']))
        .unwrap_or_default()
        .to_owned();
    Some((words.into_iter().map(OsString::from).collect(), prefix))
}

fn bash_words(line: &str) -> Option<Vec<String>> {
    // Preserve a new empty word after a space and close unfinished quotes
    // before tokenizing. shlex does not evaluate shell expansions.
    ["", "'", "\""].into_iter().find_map(|closing| {
        let mut words = shlex::split(&format!("{line}\u{1}{closing}"))?;
        let last = words.last_mut()?;
        last.strip_suffix('\u{1}')?;
        last.pop();
        Some(words)
    })
}

const BASH: &str = r#"__@BIN@_complete() {
    local candidate
    COMPREPLY=()
    while IFS= read -r candidate; do
        COMPREPLY+=("$candidate")
    done < <(
        printf '%s\0' "${COMP_LINE:0:COMP_POINT}" "$2" | command @BIN@ --_complete bash
    )
}
complete -o filenames -F __@BIN@_complete @BIN@
"#;

const ZSH: &str = r#"_@BIN@_complete() {
    local -a candidates files dirs
    local candidate
    candidates=("${(@f)$(
        printf '%s\0' "${(@Q)words[1,CURRENT-1]}" "${(Q)PREFIX}" | command @BIN@ --_complete words
    )}")
    for candidate in "${candidates[@]}"; do
        case "$candidate" in
            '') ;;
            */|*\\) dirs+=("$candidate") ;;
            *) files+=("$candidate") ;;
        esac
    done
    (( ${#dirs} )) && compadd -U -S '' -a dirs
    (( ${#files} )) && compadd -U -a files
    (( ${#dirs} || ${#files} ))
}
compdef _@BIN@_complete @BIN@
"#;

const FISH: &str = r#"function __@BIN@_complete
    set -l token (commandline --current-token --cut-at-cursor | string collect --allow-empty)
    if string match --quiet --regex '^[[:space:]]*$' -- "$token"
        set token ''
    end
    set -l current (string unescape -- "$token" | string collect --allow-empty)
    set -l mode words
    if string match --quiet -- '~/*' "$token"
        set mode fish
    end
    printf '%s\0' (commandline --current-process --tokenize --cut-at-cursor) "$current" |
        command @BIN@ --_complete "$mode"
end
complete --keep-order --exclusive --command @BIN@ --arguments '(__@BIN@_complete)'
"#;

const POWERSHELL: &str = r#"Register-ArgumentCompleter -Native -CommandName @BIN@, @BIN@.exe -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $name = $commandAst.GetCommandName()
    if (-not $name) { return }
    $application = $ExecutionContext.InvokeCommand.GetCommand(
        $name, [System.Management.Automation.CommandTypes]::Application)
    if (-not $application) { return }

    $words = @('@BIN@')
    $current = ''
    foreach ($element in ($commandAst.CommandElements | Select-Object -Skip 1)) {
        $extent = $element.Extent
        if ($extent.StartOffset -ge $cursorPosition) { break }
        $argument = if ($extent.EndOffset -gt $cursorPosition) {
            $extent.Text.Substring(0, $cursorPosition - $extent.StartOffset).TrimStart("'", '"')
        } elseif ($element -is [System.Management.Automation.Language.StringConstantExpressionAst] -or
                  $element -is [System.Management.Automation.Language.ExpandableStringExpressionAst]) {
            $element.Value
        } else {
            $extent.Text
        }
        if ($extent.EndOffset -ge $cursorPosition) {
            $current = $argument
            break
        }
        $words += $argument
    }

    $words += $current
    $OutputEncoding = [System.Text.UTF8Encoding]::new($false)
    (($words -join [char]0) + [char]0) | & $application.Path --_complete words | ForEach-Object {
        $text = if ($_ -match '^[\w./\\:=-]+$') {
            $_
        } else {
            "'" + [System.Management.Automation.Language.CodeGeneration]::EscapeSingleQuotedStringContent($_) + "'"
        }
        [System.Management.Automation.CompletionResult]::new($text, $_, 'ParameterValue', $_)
    }
}
"#;

const ELVISH: &str = r#"set edit:completion:arg-completer[@BIN@] = {|@words|
    put ({ for word $words { print $word"\x00" } } | e:@BIN@ --_complete words)
}
"#;

pub fn command(about: &'static str) -> clap::Command {
    clap::Command::new("completion").about(about).arg(
        clap::Arg::new("shell")
            .required(true)
            .value_parser(clap::value_parser!(Shell)),
    )
}

const PATH_TAG: &str = "File paths";

pub fn path_values(completer: clap_complete::PathCompleter) -> clap_complete::ArgValueCompleter {
    use clap_complete::engine::ValueCompleter;
    clap_complete::ArgValueCompleter::new(move |current: &OsStr| {
        completer
            .complete(current)
            .into_iter()
            .map(|candidate| candidate.tag(Some(PATH_TAG.into())))
            .collect()
    })
}

fn complete_command(
    command: &mut clap::Command,
    words: Vec<OsString>,
    index: usize,
) -> io::Result<Vec<CompletionCandidate>> {
    use clap::parser::ValueSource;

    let previous = &words[..index];
    let options_ended = previous.iter().any(|word| word == "--");
    let parsed = command
        .clone()
        .ignore_errors(true)
        .try_get_matches_from(previous);
    let cwd = std::env::current_dir().ok();
    let mut candidates = clap_complete::engine::complete(command, words, index, cwd.as_deref())?;

    let Ok(matches) = parsed else {
        return Ok(candidates);
    };
    let (scope, used) = matches
        .subcommand()
        .and_then(|(name, args)| command.find_subcommand(name).map(|child| (child, args)))
        .unwrap_or((command, &matches));
    let subcommands_conflict = scope.is_args_conflicts_with_subcommands_set()
        && used
            .ids()
            .any(|id| used.value_source(id.as_str()) == Some(ValueSource::CommandLine));

    candidates.retain(|candidate| {
        let Some(id) = candidate.get_id() else {
            return true;
        };
        if id.starts_with("command::") {
            return !options_ended && !subcommands_conflict;
        }
        let Some(id) = id.strip_prefix("arg::") else {
            return true;
        };
        let Some(arg) = scope
            .get_arguments()
            .find(|arg| arg.get_id().as_str() == id)
        else {
            return true;
        };
        used.value_source(id) != Some(ValueSource::CommandLine)
            || matches!(
                arg.get_action(),
                clap::ArgAction::Append | clap::ArgAction::Count
            )
    });
    Ok(candidates)
}

pub fn parse<C: clap::Parser>(mut command: clap::Command) -> io::Result<Option<C>> {
    if serve(|words, index| complete_command(&mut command, words, index))? {
        return Ok(None);
    }

    let mut matches = command.get_matches();
    if let Some(("completion", args)) = matches.subcommand() {
        write_registration(*args.get_one("shell").expect("shell is required"))?;
        return Ok(None);
    }
    Ok(Some(
        C::from_arg_matches_mut(&mut matches).unwrap_or_else(|e| e.exit()),
    ))
}

pub fn values<const N: usize>(values: [&'static str; N]) -> clap_complete::ArgValueCandidates {
    clap_complete::ArgValueCandidates::new(move || {
        values.into_iter().map(CompletionCandidate::new).collect()
    })
}
