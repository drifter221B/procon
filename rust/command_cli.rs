#[derive(Debug, Clone, PartialEq, Eq)]
struct ArgMatches {
    files: Vec<String>,
    bytes: Option<usize>,
}

fn get_args() -> Result<ArgMatches, String> {
    get_args_from(std::env::args().skip(1))
}

fn get_args_from<I, S>(args: I) -> Result<ArgMatches, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut files = Vec::new();
    let mut bytes = None;
    let mut iterator = args.into_iter().map(Into::into);

    while let Some(argument) = iterator.next() {
        match argument.as_str() {
            "-c" | "--bytes" => {
                let value = iterator
                    .next()
                    .ok_or_else(|| "clap parse error: missing value for --bytes".to_string())?;
                let parsed_value = value.parse::<usize>().map_err(|_| {
                    format!("clap parse error: invalid value `{}` for --bytes", value)
                })?;
                bytes = Some(parsed_value);
            }
            _ if argument.starts_with('-') => {
                return Err(format!("clap parse error: unknown option `{}`", argument));
            }
            _ => files.push(argument),
        }
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    Ok(ArgMatches { files, bytes })
}

#[cfg(test)]
mod tests {
    use super::get_args_from;

    #[test]
    fn default_files_becomes_stdin_marker_when_no_arguments_are_given() {
        // Arrange
        let arguments: Vec<String> = vec![];
        let expected_files = vec!["-".to_string()];

        // Act
        let actual_matches = get_args_from(arguments).expect("valid arguments");

        // Assert
        assert_eq!(actual_matches.files, expected_files);
    }

    #[test]
    fn bytes_option_parses_numeric_argument_as_usize() {
        // Arrange
        let arguments = vec!["--bytes", "10"];
        let expected_bytes = Some(10);

        // Act
        let actual_matches = get_args_from(arguments).expect("valid byte count");

        // Assert
        assert_eq!(actual_matches.bytes, expected_bytes);
    }

    #[test]
    fn non_numeric_bytes_option_returns_parse_error() {
        // Arrange
        let arguments = vec!["--bytes", "abc"];
        let expected_error = "clap parse error: invalid value `abc` for --bytes";

        // Act
        let actual_error = get_args_from(arguments).expect_err("invalid byte count");

        // Assert
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn bytes_option_without_value_returns_missing_value_error() {
        // Arrange
        let arguments = vec!["--bytes"];
        let expected_error = "clap parse error: missing value for --bytes";

        // Act
        let actual_error = get_args_from(arguments).expect_err("invalid arguments");

        // Assert
        assert_eq!(actual_error, expected_error);
    }


    #[test]
    fn unknown_option_returns_unknown_option_error() {
        // Arrange
        let arguments = vec!["--unknown"];
        let expected_error = "clap parse error: unknown option `--unknown`";

        // Act
        let actual_error = get_args_from(arguments).expect_err("invalid arguments");

        // Assert
        assert_eq!(actual_error, expected_error);
    }


    #[test]
    fn negative_bytes_returns_invalid_value_error() {
        // Arrange
        let arguments = vec!["--bytes", "-1"];
        let expected_error = "clap parse error: invalid value `-1` for --bytes";

        // Act
        let actual_error = get_args_from(arguments).expect_err("invalid arguments");

        // Assert
        assert_eq!(actual_error, expected_error);
    }


    #[test]
    fn short_bytes_option_parses_numeric_value() {
        // Arrange
        let arguments = vec!["-c", "10"];
        let expected_bytes = Some(10);

        // Act
        let actual_matches = get_args_from(arguments).expect("valid short option");

        // Assert
        assert_eq!(actual_matches.bytes, expected_bytes);
    }


    #[test]
    fn multiple_files_preserve_order_and_duplicates() {
        // Arrange
        let arguments = vec!["second.txt", "first.txt", "second.txt"];
        let expected_files = vec!["second.txt", "first.txt", "second.txt"];

        // Act
        let actual_matches = get_args_from(arguments).expect("valid filenames");

        // Assert
        assert_eq!(actual_matches.files, expected_files);
    }


    #[test]
    fn bytes_option_accepts_zero_one_and_usize_max() {
        // Arrange
        let values = [0usize, 1, usize::MAX];
        let argument_sets = values.map(|value| vec!["--bytes".to_string(), value.to_string()]);
        let expected_bytes = values.map(Some);

        // Act
        let actual_bytes = argument_sets.map(|arguments| {
            get_args_from(arguments)
                .expect("representable byte count")
                .bytes
        });

        // Assert
        assert_eq!(actual_bytes, expected_bytes);
    }

}
