use core::fmt;

pub const MIN_HOUR: i32 = 0;
pub const MAX_HOUR: i32 = 23;
pub const MIN_MINUTE: i32 = 0;
pub const MAX_MINUTE: i32 = 59;
pub const MIN_SECOND: i32 = 0;
pub const MAX_SECOND: i32 = 59;

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    Hour,
    HourMinute,
    HourMinuteSecond,
    MinuteSecond,
}

#[derive(Debug, Clone, Copy)]
pub struct FormattedClockTime<'a> {
    time: &'a ClockTime,
    format: TimeFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    InvalidHour { hour: i32 },
    InvalidMinute { minute: i32 },
    InvalidSecond { second: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseTimeError {
    InvalidFormat { value: String, format: TimeFormat },
    InvalidNumber { value: String },
    InvalidTime(TimeError),
}

impl ClockTime {
    pub fn new(hour: i32, minute: i32, second: i32) -> Result<Self, TimeError> {
        Self::validate_hour(hour)?;
        Self::validate_minute(minute)?;
        Self::validate_second(second)?;

        Ok(Self {
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        })
    }

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn second(&self) -> u8 {
        self.second
    }

    pub fn formatted(&self, format: TimeFormat) -> FormattedClockTime<'_> {
        FormattedClockTime { time: self, format }
    }

    pub fn parse_with(value: impl AsRef<str>, format: TimeFormat) -> Result<Self, ParseTimeError> {
        let value = value.as_ref();
        let components: Vec<_> = value.split(':').collect();

        let (hour, minute, second) = match (format, components.as_slice()) {
            (TimeFormat::Hour, [hour]) => (parse_time_component(hour)?, 0, 0),
            (TimeFormat::HourMinute, [hour, minute]) => (
                parse_time_component(hour)?,
                parse_time_component(minute)?,
                0,
            ),
            (TimeFormat::HourMinuteSecond, [hour, minute, second]) => (
                parse_time_component(hour)?,
                parse_time_component(minute)?,
                parse_time_component(second)?,
            ),
            (TimeFormat::MinuteSecond, [minute, second]) => (
                0,
                parse_time_component(minute)?,
                parse_time_component(second)?,
            ),
            _ => {
                return Err(ParseTimeError::InvalidFormat {
                    value: value.to_owned(),
                    format,
                });
            }
        };

        Self::new(hour, minute, second).map_err(ParseTimeError::InvalidTime)
    }

    pub fn add_minutes(&self, delta_minutes: i32) -> Self {
        let delta_seconds = i64::from(delta_minutes) * SECONDS_PER_MINUTE;

        self.add_seconds_i64(delta_seconds)
    }

    pub fn subtract_minutes(&self, subtract_minutes: i32) -> Self {
        let subtract_seconds = i64::from(subtract_minutes) * SECONDS_PER_MINUTE;

        self.add_seconds_i64(-subtract_seconds)
    }

    pub fn add_seconds(&self, delta_seconds: i32) -> Self {
        self.add_seconds_i64(i64::from(delta_seconds))
    }

    pub fn subtract_seconds(&self, subtract_seconds: i32) -> Self {
        self.add_seconds_i64(-i64::from(subtract_seconds))
    }

    fn add_seconds_i64(&self, delta_seconds: i64) -> Self {
        let updated_total_seconds = self.total_seconds() + delta_seconds;

        Self::from_total_seconds(updated_total_seconds)
    }

    fn total_seconds(&self) -> i64 {
        i64::from(self.hour) * SECONDS_PER_HOUR
            + i64::from(self.minute) * SECONDS_PER_MINUTE
            + i64::from(self.second)
    }

    fn is_valid_hour(hour: i32) -> bool {
        (MIN_HOUR..=MAX_HOUR).contains(&hour)
    }

    fn is_valid_minute(minute: i32) -> bool {
        (MIN_MINUTE..=MAX_MINUTE).contains(&minute)
    }

    fn is_valid_second(second: i32) -> bool {
        (MIN_SECOND..=MAX_SECOND).contains(&second)
    }

    fn validate_hour(hour: i32) -> Result<(), TimeError> {
        if !Self::is_valid_hour(hour) {
            return Err(TimeError::InvalidHour { hour });
        }

        Ok(())
    }

    fn validate_minute(minute: i32) -> Result<(), TimeError> {
        if !Self::is_valid_minute(minute) {
            return Err(TimeError::InvalidMinute { minute });
        }

        Ok(())
    }

    fn validate_second(second: i32) -> Result<(), TimeError> {
        if !Self::is_valid_second(second) {
            return Err(TimeError::InvalidSecond { second });
        }

        Ok(())
    }

    fn from_total_seconds(total_seconds: i64) -> Self {
        let normalized_total_seconds = normalize_seconds_in_day(total_seconds);
        let hour = normalized_total_seconds / SECONDS_PER_HOUR;
        let minute = normalized_total_seconds % SECONDS_PER_HOUR / SECONDS_PER_MINUTE;
        let second = normalized_total_seconds % SECONDS_PER_MINUTE;

        Self {
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
        }
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.formatted(TimeFormat::HourMinuteSecond).fmt(f)
    }
}

impl fmt::Display for FormattedClockTime<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            TimeFormat::Hour => write!(f, "{:02}", self.time.hour),
            TimeFormat::HourMinute => {
                write!(f, "{:02}:{:02}", self.time.hour, self.time.minute)
            }
            TimeFormat::HourMinuteSecond => write!(
                f,
                "{:02}:{:02}:{:02}",
                self.time.hour, self.time.minute, self.time.second
            ),
            TimeFormat::MinuteSecond => {
                write!(f, "{:02}:{:02}", self.time.minute, self.time.second)
            }
        }
    }
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::InvalidHour { hour } => {
                write!(f, "hour must be in {MIN_HOUR}..={MAX_HOUR}, but was {hour}")
            }
            TimeError::InvalidMinute { minute } => {
                write!(
                    f,
                    "minute must be in {MIN_MINUTE}..={MAX_MINUTE}, but was {minute}"
                )
            }
            TimeError::InvalidSecond { second } => {
                write!(
                    f,
                    "second must be in {MIN_SECOND}..={MAX_SECOND}, but was {second}"
                )
            }
        }
    }
}

impl fmt::Display for ParseTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseTimeError::InvalidFormat { value, format } => {
                write!(
                    f,
                    "time must match {}, but was {value}",
                    format.expected_pattern()
                )
            }
            ParseTimeError::InvalidNumber { value } => {
                write!(f, "time component must be an integer, but was {value}")
            }
            ParseTimeError::InvalidTime(error) => error.fmt(f),
        }
    }
}

impl TimeFormat {
    fn expected_pattern(self) -> &'static str {
        match self {
            TimeFormat::Hour => "HH",
            TimeFormat::HourMinute => "HH:MM",
            TimeFormat::HourMinuteSecond => "HH:MM:SS",
            TimeFormat::MinuteSecond => "MM:SS",
        }
    }
}

fn normalize_seconds_in_day(total_seconds: i64) -> i64 {
    total_seconds.rem_euclid(SECONDS_PER_DAY)
}

fn parse_time_component(value: &str) -> Result<i32, ParseTimeError> {
    value.parse().map_err(|_| ParseTimeError::InvalidNumber {
        value: value.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_when_hour_is_near_lower_boundary_returns_expected_result() {
        // Arrange
        let hour_below_lower = -1;
        let hour_at_lower = 0;
        let hour_above_lower = 1;
        let minute = 0;

        let expected_below = Err(TimeError::InvalidHour {
            hour: hour_below_lower,
        });
        let expected_at = Ok(ClockTime {
            hour: hour_at_lower as u8,
            minute: minute as u8,
            second: 0,
        });
        let expected_above = Ok(ClockTime {
            hour: hour_above_lower as u8,
            minute: minute as u8,
            second: 0,
        });
        let zero = 0;
        // Act
        let below_result = ClockTime::new(hour_below_lower, minute, zero);
        let at_result = ClockTime::new(hour_at_lower, minute, zero);
        let above_result = ClockTime::new(hour_above_lower, minute, zero);

        // Assert
        assert_eq!(below_result, expected_below);
        assert_eq!(at_result, expected_at);
        assert_eq!(above_result, expected_above);
    }

    #[test]
    fn new_when_hour_is_near_upper_boundary_returns_expected_result() {
        // Arrange
        let hour_below_upper = 22;
        let hour_at_upper = 23;
        let hour_above_upper = 24;
        let minute = 0;

        let expected_below = Ok(ClockTime {
            hour: hour_below_upper as u8,
            minute: minute as u8,
            second: 0,
        });
        let expected_at = Ok(ClockTime {
            hour: hour_at_upper as u8,
            minute: minute as u8,
            second: 0,
        });
        let expected_above = Err(TimeError::InvalidHour {
            hour: hour_above_upper,
        });
        let zero = 0;
        // Act
        let below_result = ClockTime::new(hour_below_upper, minute, zero);
        let at_result = ClockTime::new(hour_at_upper, minute, zero);
        let above_result = ClockTime::new(hour_above_upper, minute, zero);

        // Assert
        assert_eq!(below_result, expected_below);
        assert_eq!(at_result, expected_at);
        assert_eq!(above_result, expected_above);
    }

    #[test]
    fn new_when_minute_is_near_lower_boundary_returns_expected_result() {
        // Arrange
        let hour = 0;
        let minute_below_lower = -1;
        let minute_at_lower = 0;
        let minute_above_lower = 1;

        let expected_below = Err(TimeError::InvalidMinute {
            minute: minute_below_lower,
        });
        let expected_at = Ok(ClockTime {
            hour: hour as u8,
            minute: minute_at_lower as u8,
            second: 0,
        });
        let expected_above = Ok(ClockTime {
            hour: hour as u8,
            minute: minute_above_lower as u8,
            second: 0,
        });
        let zero = 0;
        // Act
        let below_result = ClockTime::new(hour, minute_below_lower, zero);
        let at_result = ClockTime::new(hour, minute_at_lower, zero);
        let above_result = ClockTime::new(hour, minute_above_lower, zero);

        // Assert
        assert_eq!(below_result, expected_below);
        assert_eq!(at_result, expected_at);
        assert_eq!(above_result, expected_above);
    }

    #[test]
    fn new_when_minute_is_near_upper_boundary_returns_expected_result() {
        // Arrange
        let hour = 0;
        let minute_below_upper = 58;
        let minute_at_upper = 59;
        let minute_above_upper = 60;

        let expected_below = Ok(ClockTime {
            hour: hour as u8,
            minute: minute_below_upper as u8,
            second: 0,
        });
        let expected_at = Ok(ClockTime {
            hour: hour as u8,
            minute: minute_at_upper as u8,
            second: 0,
        });
        let expected_above = Err(TimeError::InvalidMinute {
            minute: minute_above_upper,
        });
        let zero = 0;
        // Act
        let below_result = ClockTime::new(hour, minute_below_upper, zero);
        let at_result = ClockTime::new(hour, minute_at_upper, zero);
        let above_result = ClockTime::new(hour, minute_above_upper, zero);

        // Assert
        assert_eq!(below_result, expected_below);
        assert_eq!(at_result, expected_at);
        assert_eq!(above_result, expected_above);
    }

    #[test]
    fn new_when_second_is_near_lower_boundary_returns_expected_result() {
        // Arrange
        let hour = 0;
        let minute = 0;
        let second_below_lower = -1;
        let second_at_lower = 0;
        let second_above_lower = 1;

        let expected_below = Err(TimeError::InvalidSecond {
            second: second_below_lower,
        });
        let expected_at = Ok(ClockTime {
            hour: hour as u8,
            minute: minute as u8,
            second: second_at_lower as u8,
        });
        let expected_above = Ok(ClockTime {
            hour: hour as u8,
            minute: minute as u8,
            second: second_above_lower as u8,
        });
        // Act
        let below_result = ClockTime::new(hour, minute, second_below_lower);
        let at_result = ClockTime::new(hour, minute, second_at_lower);
        let above_result = ClockTime::new(hour, minute, second_above_lower);

        // Assert
        assert_eq!(below_result, expected_below);
        assert_eq!(at_result, expected_at);
        assert_eq!(above_result, expected_above);
    }

    #[test]
    fn new_when_second_is_near_upper_boundary_returns_expected_result() {
        // Arrange
        let hour = 0;
        let minute = 0;
        let second_below_upper = 58;
        let second_at_upper = 59;
        let second_above_upper = 60;

        let expected_below = Ok(ClockTime {
            hour: hour as u8,
            minute: minute as u8,
            second: second_below_upper as u8,
        });
        let expected_at = Ok(ClockTime {
            hour: hour as u8,
            minute: minute as u8,
            second: second_at_upper as u8,
        });
        let expected_above = Err(TimeError::InvalidSecond {
            second: second_above_upper,
        });
        // Act
        let below_result = ClockTime::new(hour, minute, second_below_upper);
        let at_result = ClockTime::new(hour, minute, second_at_upper);
        let above_result = ClockTime::new(hour, minute, second_above_upper);

        // Assert
        assert_eq!(below_result, expected_below);
        assert_eq!(at_result, expected_at);
        assert_eq!(above_result, expected_above);
    }

    #[test]
    fn new_when_all_components_are_invalid_returns_invalid_hour() {
        // Arrange
        let hour = -1;
        let minute = -1;
        let second = -1;
        let expected = Err(TimeError::InvalidHour { hour });

        // Act
        let actual = ClockTime::new(hour, minute, second);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn hour_minute_and_second_return_values_given_to_new() {
        // Arrange
        let hour = 13;
        let minute = 45;
        let second = 27;
        let time = ClockTime::new(hour, minute, second).unwrap();
        let expected_hour = hour as u8;
        let expected_minute = minute as u8;
        let expected_second = second as u8;

        // Act
        let actual_hour = time.hour();
        let actual_minute = time.minute();
        let actual_second = time.second();

        // Assert
        assert_eq!(actual_hour, expected_hour);
        assert_eq!(actual_minute, expected_minute);
        assert_eq!(actual_second, expected_second);
    }

    #[test]
    fn display_when_formatting_returns_zero_padded_string() {
        // Arrange
        let hour = 3;
        let minute = 7;
        let second = 9;
        let time = ClockTime::new(hour, minute, second).unwrap();
        let expected = String::from("03:07:09");

        // Act
        let actual = time.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn formatted_when_format_is_hour_returns_hour_only() {
        // Arrange
        let hour = 3;
        let minute = 7;
        let second = 9;
        let time = ClockTime::new(hour, minute, second).unwrap();
        let format = TimeFormat::Hour;
        let expected = String::from("03");

        // Act
        let actual = time.formatted(format).to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn formatted_when_format_is_hour_minute_returns_hour_and_minute() {
        // Arrange
        let hour = 3;
        let minute = 7;
        let second = 9;
        let time = ClockTime::new(hour, minute, second).unwrap();
        let format = TimeFormat::HourMinute;
        let expected = String::from("03:07");

        // Act
        let actual = time.formatted(format).to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn formatted_when_format_is_hour_minute_second_returns_full_time() {
        // Arrange
        let hour = 3;
        let minute = 7;
        let second = 9;
        let time = ClockTime::new(hour, minute, second).unwrap();
        let format = TimeFormat::HourMinuteSecond;
        let expected = String::from("03:07:09");

        // Act
        let actual = time.formatted(format).to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn formatted_when_format_is_minute_second_returns_minute_and_second() {
        // Arrange
        let hour = 3;
        let minute = 7;
        let second = 9;
        let time = ClockTime::new(hour, minute, second).unwrap();
        let format = TimeFormat::MinuteSecond;
        let expected = String::from("07:09");

        // Act
        let actual = time.formatted(format).to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_with_when_format_is_hour_returns_time_with_zero_minute_and_second() {
        // Arrange
        let value = "03";
        let format = TimeFormat::Hour;
        let expected = ClockTime::new(3, 0, 0).unwrap();

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn parse_with_when_format_is_hour_minute_returns_time_with_zero_second() {
        // Arrange
        let value = "03:07";
        let format = TimeFormat::HourMinute;
        let expected = ClockTime::new(3, 7, 0).unwrap();

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn parse_with_when_format_is_hour_minute_second_accepts_string() {
        // Arrange
        let value = String::from("03:07:09");
        let format = TimeFormat::HourMinuteSecond;
        let expected = ClockTime::new(3, 7, 9).unwrap();

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn parse_with_when_format_is_minute_second_returns_time_with_zero_hour() {
        // Arrange
        let value = "07:09";
        let format = TimeFormat::MinuteSecond;
        let expected = ClockTime::new(0, 7, 9).unwrap();

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn parse_with_when_component_count_does_not_match_returns_invalid_format() {
        // Arrange
        let value = "03:07:09";
        let format = TimeFormat::HourMinute;
        let expected = ParseTimeError::InvalidFormat {
            value: value.to_owned(),
            format,
        };

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Err(expected));
    }

    #[test]
    fn parse_with_when_component_is_not_integer_returns_invalid_number() {
        // Arrange
        let invalid_component = "minute";
        let value = format!("03:{invalid_component}");
        let format = TimeFormat::HourMinute;
        let expected = ParseTimeError::InvalidNumber {
            value: invalid_component.to_owned(),
        };

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Err(expected));
    }

    #[test]
    fn parse_with_when_component_is_out_of_range_returns_invalid_time() {
        // Arrange
        let invalid_minute = 60;
        let value = format!("03:{invalid_minute}");
        let format = TimeFormat::HourMinute;
        let expected = ParseTimeError::InvalidTime(TimeError::InvalidMinute {
            minute: invalid_minute,
        });

        // Act
        let actual = ClockTime::parse_with(value, format);

        // Assert
        assert_eq!(actual, Err(expected));
    }

    #[test]
    fn parse_with_when_components_are_at_bounds_returns_expected_times() {
        // Arrange
        let cases = [
            ("00", TimeFormat::Hour, ClockTime::new(0, 0, 0).unwrap()),
            ("23", TimeFormat::Hour, ClockTime::new(23, 0, 0).unwrap()),
            (
                "00:00",
                TimeFormat::HourMinute,
                ClockTime::new(0, 0, 0).unwrap(),
            ),
            (
                "23:59",
                TimeFormat::HourMinute,
                ClockTime::new(23, 59, 0).unwrap(),
            ),
            (
                "00:00:00",
                TimeFormat::HourMinuteSecond,
                ClockTime::new(0, 0, 0).unwrap(),
            ),
            (
                "23:59:59",
                TimeFormat::HourMinuteSecond,
                ClockTime::new(23, 59, 59).unwrap(),
            ),
            (
                "00:00",
                TimeFormat::MinuteSecond,
                ClockTime::new(0, 0, 0).unwrap(),
            ),
            (
                "59:59",
                TimeFormat::MinuteSecond,
                ClockTime::new(0, 59, 59).unwrap(),
            ),
        ];

        let expected_results: Vec<Result<ClockTime, ParseTimeError>> =
            cases.iter().map(|(_, _, expected)| Ok(*expected)).collect();

        // Act
        let actual_results: Vec<Result<ClockTime, ParseTimeError>> = cases
            .iter()
            .map(|(value, format, _)| ClockTime::parse_with(value, *format))
            .collect();

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn parse_with_when_components_are_missing_returns_invalid_format() {
        // Arrange
        let cases = [
            ("03", TimeFormat::HourMinute),
            ("03:07", TimeFormat::HourMinuteSecond),
            ("07", TimeFormat::MinuteSecond),
        ];

        let expected_results: Vec<Result<ClockTime, ParseTimeError>> = cases
            .iter()
            .map(|(value, format)| {
                Err(ParseTimeError::InvalidFormat {
                    value: (*value).to_owned(),
                    format: *format,
                })
            })
            .collect();
        // Act
        let actual_results: Vec<Result<ClockTime, ParseTimeError>> = cases
            .iter()
            .map(|(value, format)| ClockTime::parse_with(value, *format))
            .collect();
        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn parse_with_when_component_is_empty_returns_invalid_number() {
        // Arrange
        let cases = [
            ("", TimeFormat::Hour),
            (":07", TimeFormat::HourMinute),
            ("03::09", TimeFormat::HourMinuteSecond),
            ("07:", TimeFormat::MinuteSecond),
        ];
        let expected = Err(ParseTimeError::InvalidNumber {
            value: String::new(),
        });

        let expected_results = vec![expected; cases.len()];
        // Act
        let actual_results: Vec<Result<ClockTime, ParseTimeError>> = cases
            .iter()
            .map(|(value, format)| ClockTime::parse_with(value, *format))
            .collect();
        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn parse_with_when_components_are_not_zero_padded_returns_expected_time() {
        // Arrange
        let value = "3:7:9";
        let expected = ClockTime::new(3, 7, 9).unwrap();

        let format_hourminutesecond = TimeFormat::HourMinuteSecond;
        // Act
        let actual = ClockTime::parse_with(value, format_hourminutesecond);

        // Assert
        assert_eq!(actual, Ok(expected));
    }

    #[test]
    fn parse_with_when_hour_is_out_of_range_returns_invalid_hour() {
        // Arrange
        let cases = [-1, 24];

        let format = TimeFormat::HourMinuteSecond;
        let expected_results: Vec<Result<ClockTime, ParseTimeError>> = cases
            .iter()
            .map(|hour| {
                Err(ParseTimeError::InvalidTime(TimeError::InvalidHour {
                    hour: *hour,
                }))
            })
            .collect();
        let values: Vec<String> = cases.iter().map(|hour| format!("{hour}:00:00")).collect();
        // Act
        let actual_results: Vec<Result<ClockTime, ParseTimeError>> = values
            .iter()
            .map(|value| ClockTime::parse_with(value, format))
            .collect();
        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn parse_with_when_minute_is_below_lower_bound_returns_invalid_minute() {
        // Arrange
        let minute = -1;
        let value = format!("03:{minute}:00");
        let expected = Err(ParseTimeError::InvalidTime(TimeError::InvalidMinute {
            minute,
        }));

        let format_hourminutesecond = TimeFormat::HourMinuteSecond;
        // Act
        let actual = ClockTime::parse_with(value, format_hourminutesecond);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_with_when_second_is_out_of_range_returns_invalid_second() {
        // Arrange
        let cases = [-1, 60];

        let format = TimeFormat::HourMinuteSecond;
        let expected_results: Vec<Result<ClockTime, ParseTimeError>> = cases
            .iter()
            .map(|second| {
                Err(ParseTimeError::InvalidTime(TimeError::InvalidSecond {
                    second: *second,
                }))
            })
            .collect();
        let values: Vec<String> = cases
            .iter()
            .map(|second| format!("03:07:{second}"))
            .collect();
        // Act
        let actual_results: Vec<Result<ClockTime, ParseTimeError>> = values
            .iter()
            .map(|value| ClockTime::parse_with(value, format))
            .collect();
        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn parse_with_when_component_exceeds_i32_range_returns_invalid_number() {
        // Arrange
        let invalid_component = "2147483648";
        let value = format!("{invalid_component}:00");
        let expected = Err(ParseTimeError::InvalidNumber {
            value: invalid_component.to_owned(),
        });

        let format_hourminute = TimeFormat::HourMinute;
        // Act
        let actual = ClockTime::parse_with(value, format_hourminute);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_with_when_multiple_components_are_not_integers_returns_first_invalid_number() {
        // Arrange
        let invalid_component = "hour";
        let value = "hour:minute:second";
        let expected = Err(ParseTimeError::InvalidNumber {
            value: invalid_component.to_owned(),
        });

        let format_hourminutesecond = TimeFormat::HourMinuteSecond;
        // Act
        let actual = ClockTime::parse_with(value, format_hourminutesecond);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_with_when_multiple_components_are_out_of_range_returns_invalid_hour() {
        // Arrange
        let hour = 24;
        let value = "24:60:60";
        let expected = Err(ParseTimeError::InvalidTime(TimeError::InvalidHour { hour }));

        let format_hourminutesecond = TimeFormat::HourMinuteSecond;
        // Act
        let actual = ClockTime::parse_with(value, format_hourminutesecond);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_with_when_later_component_is_not_integer_returns_invalid_number_before_range_error() {
        // Arrange
        let invalid_component = "minute";
        let value = "24:minute:00";
        let expected = Err(ParseTimeError::InvalidNumber {
            value: invalid_component.to_owned(),
        });

        let format_hourminutesecond = TimeFormat::HourMinuteSecond;
        // Act
        let actual = ClockTime::parse_with(value, format_hourminutesecond);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_minutes_when_delta_is_near_zero_returns_expected_times() {
        // Arrange
        let second = 56;
        let base_time = ClockTime::new(12, 34, second).unwrap();
        let delta_below_zero = -1;
        let delta_at_zero = 0;
        let delta_above_zero = 1;
        let expected_below = ClockTime::new(12, 33, second).unwrap();
        let expected_at = ClockTime::new(12, 34, second).unwrap();
        let expected_above = ClockTime::new(12, 35, second).unwrap();

        // Act
        let actual_below = base_time.add_minutes(delta_below_zero);
        let actual_at = base_time.add_minutes(delta_at_zero);
        let actual_above = base_time.add_minutes(delta_above_zero);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn add_minutes_when_crossing_lower_day_boundary_returns_expected_times() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let delta_below_boundary = -1;
        let delta_at_boundary = 0;
        let delta_above_boundary = 1;
        let expected_below = ClockTime::new(23, 59, 0).unwrap();
        let expected_at = ClockTime::new(0, 0, 0).unwrap();
        let expected_above = ClockTime::new(0, 1, 0).unwrap();

        // Act
        let actual_below = base_time.add_minutes(delta_below_boundary);
        let actual_at = base_time.add_minutes(delta_at_boundary);
        let actual_above = base_time.add_minutes(delta_above_boundary);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn add_minutes_when_crossing_upper_day_boundary_returns_expected_times() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 0).unwrap();
        let delta_below_boundary = -1;
        let delta_at_boundary = 0;
        let delta_above_boundary = 1;
        let expected_below = ClockTime::new(23, 58, 0).unwrap();
        let expected_at = ClockTime::new(23, 59, 0).unwrap();
        let expected_above = ClockTime::new(0, 0, 0).unwrap();

        // Act
        let actual_below = base_time.add_minutes(delta_below_boundary);
        let actual_at = base_time.add_minutes(delta_at_boundary);
        let actual_above = base_time.add_minutes(delta_above_boundary);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn add_minutes_normalizes_positive_delta_greater_than_one_day() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 0).unwrap();
        let delta_minutes = 24 * 60 + 1;
        let expected_time = ClockTime::new(0, 0, 0).unwrap();

        // Act
        let actual_time = base_time.add_minutes(delta_minutes);

        // Assert
        assert_eq!(actual_time, expected_time);
    }

    #[test]
    fn add_minutes_normalizes_negative_delta_less_than_minus_one_day() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let delta_minutes = -(24 * 60 + 1);
        let expected_time = ClockTime::new(23, 59, 0).unwrap();

        // Act
        let actual_time = base_time.add_minutes(delta_minutes);

        // Assert
        assert_eq!(actual_time, expected_time);
    }

    #[test]
    fn add_minutes_when_crossing_hour_boundary_carries_to_next_hour() {
        // Arrange
        let base_time = ClockTime::new(12, 59, 27).unwrap();
        let expected = ClockTime::new(13, 0, 27).unwrap();

        let one = 1;
        // Act
        let actual = base_time.add_minutes(one);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_minutes_when_delta_is_i32_bounds_normalizes_without_overflow() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let expected_min = ClockTime::new(21, 52, 0).unwrap();
        let expected_max = ClockTime::new(2, 7, 0).unwrap();

        let minimum_delta = i32::MIN;
        let maximum_delta = i32::MAX;
        // Act
        let actual_min = base_time.add_minutes(minimum_delta);
        let actual_max = base_time.add_minutes(maximum_delta);

        // Assert
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn subtract_minutes_when_delta_is_near_zero_returns_expected_times() {
        // Arrange
        let second = 56;
        let base_time = ClockTime::new(12, 34, second).unwrap();
        let subtract_below_zero = -1;
        let subtract_at_zero = 0;
        let subtract_above_zero = 1;
        let expected_below = ClockTime::new(12, 35, second).unwrap();
        let expected_at = ClockTime::new(12, 34, second).unwrap();
        let expected_above = ClockTime::new(12, 33, second).unwrap();

        // Act
        let actual_below = base_time.subtract_minutes(subtract_below_zero);
        let actual_at = base_time.subtract_minutes(subtract_at_zero);
        let actual_above = base_time.subtract_minutes(subtract_above_zero);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn subtract_minutes_when_crossing_lower_day_boundary_returns_expected_times() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let subtract_below_boundary = -1;
        let subtract_at_boundary = 0;
        let subtract_above_boundary = 1;
        let expected_below = ClockTime::new(0, 1, 0).unwrap();
        let expected_at = ClockTime::new(0, 0, 0).unwrap();
        let expected_above = ClockTime::new(23, 59, 0).unwrap();

        // Act
        let actual_below = base_time.subtract_minutes(subtract_below_boundary);
        let actual_at = base_time.subtract_minutes(subtract_at_boundary);
        let actual_above = base_time.subtract_minutes(subtract_above_boundary);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn subtract_minutes_when_crossing_upper_day_boundary_returns_expected_times() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 0).unwrap();
        let subtract_below_boundary = -1;
        let subtract_at_boundary = 0;
        let subtract_above_boundary = 1;
        let expected_below = ClockTime::new(0, 0, 0).unwrap();
        let expected_at = ClockTime::new(23, 59, 0).unwrap();
        let expected_above = ClockTime::new(23, 58, 0).unwrap();

        // Act
        let actual_below = base_time.subtract_minutes(subtract_below_boundary);
        let actual_at = base_time.subtract_minutes(subtract_at_boundary);
        let actual_above = base_time.subtract_minutes(subtract_above_boundary);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn subtract_minutes_when_crossing_hour_boundary_borrows_from_previous_hour() {
        // Arrange
        let base_time = ClockTime::new(12, 0, 27).unwrap();
        let expected = ClockTime::new(11, 59, 27).unwrap();

        let one = 1;
        // Act
        let actual = base_time.subtract_minutes(one);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_minutes_normalizes_positive_delta_greater_than_one_day() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let subtract_minutes = 24 * 60 + 1;
        let expected = ClockTime::new(23, 59, 0).unwrap();

        // Act
        let actual = base_time.subtract_minutes(subtract_minutes);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_minutes_normalizes_negative_delta_less_than_minus_one_day() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 0).unwrap();
        let subtract_minutes = -(24 * 60 + 1);
        let expected = ClockTime::new(0, 0, 0).unwrap();

        // Act
        let actual = base_time.subtract_minutes(subtract_minutes);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_minutes_when_delta_is_i32_bounds_normalizes_without_overflow() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let expected_min = ClockTime::new(2, 8, 0).unwrap();
        let expected_max = ClockTime::new(21, 53, 0).unwrap();

        let minimum_delta = i32::MIN;
        let maximum_delta = i32::MAX;
        // Act
        let actual_min = base_time.subtract_minutes(minimum_delta);
        let actual_max = base_time.subtract_minutes(maximum_delta);

        // Assert
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn add_seconds_when_delta_is_near_zero_returns_expected_times() {
        // Arrange
        let base_time = ClockTime::new(12, 34, 56).unwrap();
        let delta_below_zero = -1;
        let delta_at_zero = 0;
        let delta_above_zero = 1;
        let expected_below = ClockTime::new(12, 34, 55).unwrap();
        let expected_at = ClockTime::new(12, 34, 56).unwrap();
        let expected_above = ClockTime::new(12, 34, 57).unwrap();

        // Act
        let actual_below = base_time.add_seconds(delta_below_zero);
        let actual_at = base_time.add_seconds(delta_at_zero);
        let actual_above = base_time.add_seconds(delta_above_zero);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn add_seconds_when_crossing_day_boundaries_returns_expected_times() {
        // Arrange
        let start_of_day = ClockTime::new(0, 0, 0).unwrap();
        let end_of_day = ClockTime::new(23, 59, 59).unwrap();
        let subtract_one_second = -1;
        let add_one_second = 1;
        let expected_before_start = end_of_day;
        let expected_after_end = start_of_day;

        // Act
        let actual_before_start = start_of_day.add_seconds(subtract_one_second);
        let actual_after_end = end_of_day.add_seconds(add_one_second);

        // Assert
        assert_eq!(actual_before_start, expected_before_start);
        assert_eq!(actual_after_end, expected_after_end);
    }

    #[test]
    fn add_seconds_when_crossing_hour_boundary_carries_to_next_hour() {
        // Arrange
        let base_time = ClockTime::new(12, 59, 59).unwrap();
        let expected = ClockTime::new(13, 0, 0).unwrap();

        let one = 1;
        // Act
        let actual = base_time.add_seconds(one);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_seconds_normalizes_positive_delta_greater_than_one_day() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 59).unwrap();
        let delta_seconds = 24 * 60 * 60 + 1;
        let expected = ClockTime::new(0, 0, 0).unwrap();

        // Act
        let actual = base_time.add_seconds(delta_seconds);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_seconds_normalizes_negative_delta_less_than_minus_one_day() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let delta_seconds = -(24 * 60 * 60 + 1);
        let expected = ClockTime::new(23, 59, 59).unwrap();

        // Act
        let actual = base_time.add_seconds(delta_seconds);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_seconds_when_delta_is_i32_bounds_normalizes_without_overflow() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let expected_min = ClockTime::new(20, 45, 52).unwrap();
        let expected_max = ClockTime::new(3, 14, 7).unwrap();

        let minimum_delta = i32::MIN;
        let maximum_delta = i32::MAX;
        // Act
        let actual_min = base_time.add_seconds(minimum_delta);
        let actual_max = base_time.add_seconds(maximum_delta);

        // Assert
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn subtract_seconds_when_crossing_minute_boundary_returns_expected_time() {
        // Arrange
        let base_time = ClockTime::new(12, 34, 0).unwrap();
        let subtract_seconds = 1;
        let expected_time = ClockTime::new(12, 33, 59).unwrap();

        // Act
        let actual_time = base_time.subtract_seconds(subtract_seconds);

        // Assert
        assert_eq!(actual_time, expected_time);
    }

    #[test]
    fn subtract_seconds_when_delta_is_near_zero_returns_expected_times() {
        // Arrange
        let base_time = ClockTime::new(12, 34, 56).unwrap();
        let expected_negative = ClockTime::new(12, 34, 57).unwrap();
        let expected_zero = base_time;
        let expected_positive = ClockTime::new(12, 34, 55).unwrap();

        let negative_one = -1;
        let zero = 0;
        let one = 1;
        // Act
        let actual_negative = base_time.subtract_seconds(negative_one);
        let actual_zero = base_time.subtract_seconds(zero);
        let actual_positive = base_time.subtract_seconds(one);

        // Assert
        assert_eq!(actual_negative, expected_negative);
        assert_eq!(actual_zero, expected_zero);
        assert_eq!(actual_positive, expected_positive);
    }

    #[test]
    fn subtract_seconds_when_crossing_lower_day_boundary_wraps_to_previous_day() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let expected = ClockTime::new(23, 59, 59).unwrap();

        let one = 1;
        // Act
        let actual = base_time.subtract_seconds(one);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_seconds_with_negative_delta_crosses_upper_day_boundary() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 59).unwrap();
        let expected = ClockTime::new(0, 0, 0).unwrap();

        let negative_one = -1;
        // Act
        let actual = base_time.subtract_seconds(negative_one);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_seconds_normalizes_positive_delta_greater_than_one_day() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let subtract_seconds = 24 * 60 * 60 + 1;
        let expected = ClockTime::new(23, 59, 59).unwrap();

        // Act
        let actual = base_time.subtract_seconds(subtract_seconds);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_seconds_normalizes_negative_delta_less_than_minus_one_day() {
        // Arrange
        let base_time = ClockTime::new(23, 59, 59).unwrap();
        let subtract_seconds = -(24 * 60 * 60 + 1);
        let expected = ClockTime::new(0, 0, 0).unwrap();

        // Act
        let actual = base_time.subtract_seconds(subtract_seconds);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtract_seconds_when_delta_is_i32_bounds_normalizes_without_overflow() {
        // Arrange
        let base_time = ClockTime::new(0, 0, 0).unwrap();
        let expected_min = ClockTime::new(3, 14, 8).unwrap();
        let expected_max = ClockTime::new(20, 45, 53).unwrap();

        let minimum_delta = i32::MIN;
        let maximum_delta = i32::MAX;
        // Act
        let actual_min = base_time.subtract_seconds(minimum_delta);
        let actual_max = base_time.subtract_seconds(maximum_delta);

        // Assert
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn is_valid_hour_when_hour_is_near_lower_boundary_returns_expected_result() {
        // Arrange
        let hour_below_lower = -1;
        let hour_at_lower = 0;
        let hour_above_lower = 1;
        let expected_below = false;
        let expected_at = true;
        let expected_above = true;

        // Act
        let actual_below = ClockTime::is_valid_hour(hour_below_lower);
        let actual_at = ClockTime::is_valid_hour(hour_at_lower);
        let actual_above = ClockTime::is_valid_hour(hour_above_lower);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn is_valid_hour_when_hour_is_near_upper_boundary_returns_expected_result() {
        // Arrange
        let hour_below_upper = 22;
        let hour_at_upper = 23;
        let hour_above_upper = 24;
        let expected_below = true;
        let expected_at = true;
        let expected_above = false;

        // Act
        let actual_below = ClockTime::is_valid_hour(hour_below_upper);
        let actual_at = ClockTime::is_valid_hour(hour_at_upper);
        let actual_above = ClockTime::is_valid_hour(hour_above_upper);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn is_valid_minute_when_minute_is_near_lower_boundary_returns_expected_result() {
        // Arrange
        let minute_below_lower = -1;
        let minute_at_lower = 0;
        let minute_above_lower = 1;
        let expected_below = false;
        let expected_at = true;
        let expected_above = true;

        // Act
        let actual_below = ClockTime::is_valid_minute(minute_below_lower);
        let actual_at = ClockTime::is_valid_minute(minute_at_lower);
        let actual_above = ClockTime::is_valid_minute(minute_above_lower);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn is_valid_minute_when_minute_is_near_upper_boundary_returns_expected_result() {
        // Arrange
        let minute_below_upper = 58;
        let minute_at_upper = 59;
        let minute_above_upper = 60;
        let expected_below = true;
        let expected_at = true;
        let expected_above = false;

        // Act
        let actual_below = ClockTime::is_valid_minute(minute_below_upper);
        let actual_at = ClockTime::is_valid_minute(minute_at_upper);
        let actual_above = ClockTime::is_valid_minute(minute_above_upper);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn is_valid_second_when_second_is_near_lower_boundary_returns_expected_result() {
        // Arrange
        let second_below_lower = -1;
        let second_at_lower = 0;
        let second_above_lower = 1;
        let expected_below = false;
        let expected_at = true;
        let expected_above = true;

        // Act
        let actual_below = ClockTime::is_valid_second(second_below_lower);
        let actual_at = ClockTime::is_valid_second(second_at_lower);
        let actual_above = ClockTime::is_valid_second(second_above_lower);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn is_valid_second_when_second_is_near_upper_boundary_returns_expected_result() {
        // Arrange
        let second_below_upper = 58;
        let second_at_upper = 59;
        let second_above_upper = 60;
        let expected_below = true;
        let expected_at = true;
        let expected_above = false;

        // Act
        let actual_below = ClockTime::is_valid_second(second_below_upper);
        let actual_at = ClockTime::is_valid_second(second_at_upper);
        let actual_above = ClockTime::is_valid_second(second_above_upper);

        // Assert
        assert_eq!(actual_below, expected_below);
        assert_eq!(actual_at, expected_at);
        assert_eq!(actual_above, expected_above);
    }

    #[test]
    fn adding_and_subtracting_one_day_returns_same_time() {
        // Arrange
        let base_time = ClockTime::new(9, 8, 0).unwrap();
        let one_day_minutes = 24 * 60;
        let expected_time = base_time;

        // Act
        let actual_added_time = base_time.add_minutes(one_day_minutes);
        let actual_subtracted_time = base_time.subtract_minutes(one_day_minutes);

        // Assert
        assert_eq!(actual_added_time, expected_time);
        assert_eq!(actual_subtracted_time, expected_time);
    }

    #[test]
    fn adding_and_subtracting_one_day_in_seconds_returns_same_time() {
        // Arrange
        let base_time = ClockTime::new(9, 8, 7).unwrap();
        let one_day_seconds = 24 * 60 * 60;
        let expected_time = base_time;

        // Act
        let actual_added_time = base_time.add_seconds(one_day_seconds);
        let actual_subtracted_time = base_time.subtract_seconds(one_day_seconds);

        // Assert
        assert_eq!(actual_added_time, expected_time);
        assert_eq!(actual_subtracted_time, expected_time);
    }

    #[test]
    fn time_error_display_returns_expected_messages() {
        // Arrange
        let invalid_hour = TimeError::InvalidHour { hour: 24 };
        let invalid_minute = TimeError::InvalidMinute { minute: 60 };
        let invalid_second = TimeError::InvalidSecond { second: 60 };
        let expected_hour_message = "hour must be in 0..=23, but was 24".to_string();
        let expected_minute_message = "minute must be in 0..=59, but was 60".to_string();
        let expected_second_message = "second must be in 0..=59, but was 60".to_string();

        // Act
        let actual_hour_message = invalid_hour.to_string();
        let actual_minute_message = invalid_minute.to_string();
        let actual_second_message = invalid_second.to_string();

        // Assert
        assert_eq!(actual_hour_message, expected_hour_message);
        assert_eq!(actual_minute_message, expected_minute_message);
        assert_eq!(actual_second_message, expected_second_message);
    }

    #[test]
    fn parse_time_error_invalid_format_display_contains_pattern_and_value() {
        // Arrange
        let cases = [
            (TimeFormat::Hour, "3", "time must match HH, but was 3"),
            (
                TimeFormat::HourMinute,
                "3",
                "time must match HH:MM, but was 3",
            ),
            (
                TimeFormat::HourMinuteSecond,
                "3:7",
                "time must match HH:MM:SS, but was 3:7",
            ),
            (
                TimeFormat::MinuteSecond,
                "7",
                "time must match MM:SS, but was 7",
            ),
        ];

        let expected_messages: Vec<&str> = cases.iter().map(|(_, _, expected)| *expected).collect();
        // Act
        let actual_messages: Vec<String> = cases
            .iter()
            .map(|(format, value, _)| {
                ParseTimeError::InvalidFormat {
                    value: (*value).to_owned(),
                    format: *format,
                }
                .to_string()
            })
            .collect();
        // Assert
        assert_eq!(actual_messages, expected_messages);
    }

    #[test]
    fn parse_time_error_invalid_number_display_contains_invalid_component() {
        // Arrange
        let invalid_component = "minute";
        let error = ParseTimeError::InvalidNumber {
            value: invalid_component.to_owned(),
        };
        let expected = "time component must be an integer, but was minute";

        // Act
        let actual = error.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_time_error_invalid_time_display_matches_inner_time_error() {
        // Arrange
        let inner_error = TimeError::InvalidSecond { second: 60 };
        let expected = inner_error.to_string();
        let error = ParseTimeError::InvalidTime(inner_error);

        // Act
        let actual = error.to_string();

        // Assert
        assert_eq!(actual, expected);
    }
}
