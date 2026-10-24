use std::collections::HashSet;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortOrder {
    pub column_name: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortParseError {
    EmptyInput,
    EmptyToken {
        token_index: usize,
    },
    MissingKeyAfterDescPrefix {
        token_index: usize,
    },
    InvalidKey {
        key_name: String,
        token_index: usize,
    },
}

pub trait SortKeyResolver {
    fn resolve_column_name(&self, key_name: &str) -> Option<&str>;
}

pub trait SortKeyMapping {
    fn from_client_key(key_name: &str) -> Option<Self>
    where
        Self: Sized;

    fn column_name(&self) -> &'static str;
}

pub struct StaticSortKeyResolver<T: SortKeyMapping> {
    marker: PhantomData<T>,
}

impl<T: SortKeyMapping> StaticSortKeyResolver<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<T: SortKeyMapping> SortKeyResolver for StaticSortKeyResolver<T> {
    fn resolve_column_name(&self, key_name: &str) -> Option<&str> {
        T::from_client_key(key_name).map(|sort_key_mapping| sort_key_mapping.column_name())
    }
}

pub fn parse_sort_parameter(
    sort_value: &str,
    sort_key_resolver: &impl SortKeyResolver,
) -> Result<Vec<SortOrder>, SortParseError> {
    let normalized_sort_value = sort_value.trim();
    if normalized_sort_value.is_empty() {
        return Err(SortParseError::EmptyInput);
    }

    let token_list = split_sort_tokens(normalized_sort_value);
    let mut parsed_sort_orders = Vec::new();
    let mut seen_key_names = HashSet::new();

    for (token_index, token_value) in token_list.iter().enumerate() {
        let (key_name, direction) = parse_single_token(token_value, token_index)?;
        let column_name = resolve_column_name(&key_name, sort_key_resolver, token_index)?;

        append_if_first_appearance(
            key_name,
            column_name,
            direction,
            &mut seen_key_names,
            &mut parsed_sort_orders,
        );
    }

    Ok(parsed_sort_orders)
}

fn split_sort_tokens(sort_value: &str) -> Vec<&str> {
    sort_value.split(',').collect()
}

fn parse_single_token(
    token_value: &str,
    token_index: usize,
) -> Result<(String, SortDirection), SortParseError> {
    let normalized_token = token_value.trim();
    if normalized_token.is_empty() {
        return Err(SortParseError::EmptyToken { token_index });
    }

    if let Some(descending_key_candidate) = normalized_token.strip_prefix('-') {
        let normalized_key_name = descending_key_candidate.trim();
        if normalized_key_name.is_empty() {
            return Err(SortParseError::MissingKeyAfterDescPrefix { token_index });
        }

        Ok((normalized_key_name.to_string(), SortDirection::Desc))
    } else {
        Ok((normalized_token.to_string(), SortDirection::Asc))
    }
}

fn resolve_column_name(
    key_name: &str,
    sort_key_resolver: &impl SortKeyResolver,
    token_index: usize,
) -> Result<String, SortParseError> {
    sort_key_resolver
        .resolve_column_name(key_name)
        .map(ToString::to_string)
        .ok_or_else(|| SortParseError::InvalidKey {
            key_name: key_name.to_string(),
            token_index,
        })
}

fn append_if_first_appearance(
    key_name: String,
    column_name: String,
    direction: SortDirection,
    seen_key_names: &mut HashSet<String>,
    parsed_sort_orders: &mut Vec<SortOrder>,
) {
    let is_first_appearance = seen_key_names.insert(key_name);
    if is_first_appearance {
        parsed_sort_orders.push(SortOrder {
            column_name,
            direction,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum TestSortKeyMapping {
        Key1,
        Key2,
        Key3,
    }

    impl SortKeyMapping for TestSortKeyMapping {
        fn from_client_key(key_name: &str) -> Option<Self> {
            match key_name {
                "key1" => Some(Self::Key1),
                "key2" => Some(Self::Key2),
                "key3" => Some(Self::Key3),
                _ => None,
            }
        }

        fn column_name(&self) -> &'static str {
            match self {
                Self::Key1 => "column_1",
                Self::Key2 => "column_2",
                Self::Key3 => "column_3",
            }
        }
    }

    fn create_sort_key_resolver() -> StaticSortKeyResolver<TestSortKeyMapping> {
        StaticSortKeyResolver::new()
    }

    #[test]
    fn static_sort_key_resolver_resolves_column_name_directly() {
        // Arrange
        let sort_key_resolver = create_sort_key_resolver();
        let key_name = "key2";
        let expected_column_name = Some("column_2");

        // Act
        let actual_column_name = sort_key_resolver.resolve_column_name(key_name);

        // Assert
        assert_eq!(actual_column_name, expected_column_name);
    }

    #[test]
    fn parse_sort_parameter_returns_empty_input_error_when_input_is_blank() {
        // Arrange
        let sort_value = "   ";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::EmptyInput);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_parses_single_ascending_key_when_key_is_valid() {
        // Arrange
        let sort_value = "key1";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![SortOrder {
            column_name: "column_1".to_string(),
            direction: SortDirection::Asc,
        }]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_parses_single_descending_key_when_token_has_prefix() {
        // Arrange
        let sort_value = "-key2";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![SortOrder {
            column_name: "column_2".to_string(),
            direction: SortDirection::Desc,
        }]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_returns_invalid_key_error_when_key_is_not_allowed() {
        // Arrange
        let sort_value = "unknown";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::InvalidKey {
            key_name: "unknown".to_string(),
            token_index: 0,
        });

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_returns_empty_token_error_for_consecutive_commas() {
        // Arrange
        let sort_value = "key1,,key2";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::EmptyToken { token_index: 1 });

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_parses_single_key_in_ascending_order() {
        // Arrange
        let sort_value = "key1";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![SortOrder {
            column_name: "column_1".to_string(),
            direction: SortDirection::Asc,
        }]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_parses_token_count_x_plus_1_boundary() {
        // Arrange
        let sort_value = "key1,key2";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![
            SortOrder {
                column_name: "column_1".to_string(),
                direction: SortDirection::Asc,
            },
            SortOrder {
                column_name: "column_2".to_string(),
                direction: SortDirection::Asc,
            },
        ]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_ignores_later_duplicate_key_when_duplicate_exists() {
        // Arrange
        let sort_value = "key1,-key1,key2";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![
            SortOrder {
                column_name: "column_1".to_string(),
                direction: SortDirection::Asc,
            },
            SortOrder {
                column_name: "column_2".to_string(),
                direction: SortDirection::Asc,
            },
        ]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_allows_whitespace_around_tokens_and_keys() {
        // Arrange
        let sort_value = " key1 , -key2 ";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![
            SortOrder {
                column_name: "column_1".to_string(),
                direction: SortDirection::Asc,
            },
            SortOrder {
                column_name: "column_2".to_string(),
                direction: SortDirection::Desc,
            },
        ]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_returns_missing_key_after_desc_prefix_when_desc_token_has_no_key() {
        // Arrange
        let sort_value = "-   ";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::MissingKeyAfterDescPrefix { token_index: 0 });

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_returns_empty_input_error() {
        // Arrange
        let empty_sort_value = "";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_empty_result = Err(SortParseError::EmptyInput);

        // Act
        let actual_result = parse_sort_parameter(empty_sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(actual_result, expected_empty_result);
    }

    #[test]
    fn parse_sort_parameter_preserves_multiple_key_order() {
        // Arrange
        let sort_value = "key3,-key2,key1";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_multiple_result = Ok(vec![
            SortOrder {
                column_name: "column_3".to_string(),
                direction: SortDirection::Asc,
            },
            SortOrder {
                column_name: "column_2".to_string(),
                direction: SortDirection::Desc,
            },
            SortOrder {
                column_name: "column_1".to_string(),
                direction: SortDirection::Asc,
            },
        ]);

        // Act
        let actual_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(actual_result, expected_multiple_result);
    }

    #[test]
    fn parse_sort_parameter_returns_empty_token_error_for_leading_comma() {
        // Arrange
        let sort_value = ",key1";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::EmptyToken { token_index: 0 });

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_returns_empty_token_error_for_trailing_comma() {
        // Arrange
        let sort_value = "key1,";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::EmptyToken { token_index: 1 });

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_returns_invalid_key_error_with_later_token_index() {
        // Arrange
        let sort_value = "key1,unknown";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Err(SortParseError::InvalidKey {
            key_name: "unknown".to_string(),
            token_index: 1,
        });

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_trims_key_after_desc_prefix() {
        // Arrange
        let sort_value = "- key1";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![SortOrder {
            column_name: "column_1".to_string(),
            direction: SortDirection::Desc,
        }]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }

    #[test]
    fn parse_sort_parameter_keeps_first_desc_direction_for_duplicate_key() {
        // Arrange
        let sort_value = "-key1,key1";
        let sort_key_resolver = create_sort_key_resolver();
        let expected_result = Ok(vec![SortOrder {
            column_name: "column_1".to_string(),
            direction: SortDirection::Desc,
        }]);

        // Act
        let parsed_result = parse_sort_parameter(sort_value, &sort_key_resolver);

        // Assert
        assert_eq!(parsed_result, expected_result);
    }
}
