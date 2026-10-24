struct GregorianCalendar;

impl GregorianCalendar {
    const WEEKDAYS: [&'static str; 7] = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];

    fn is_leap_year(year: usize) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    pub fn day_index(year: usize, month: usize, day: usize) -> usize {
        let leap_years = year / 4 + usize::from(year % 4 != 0)
            - (year / 100 + usize::from(year % 100 != 0))
            + year / 400
            + usize::from(year % 400 != 0);

        let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

        year * 365
            + leap_years
            + days_before_month[month - 1]
            + usize::from(month > 2 && Self::is_leap_year(year))
            + (day - 1)
    }

    pub fn day_name(year: usize, month: usize, day: usize) -> String {
        let weekday_index = (Self::day_index(year, month, day) % 7 + 5) % 7;

        Self::WEEKDAYS[weekday_index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::GregorianCalendar;

    #[test]
    fn weekdays_lists_monday_to_sunday_with_length_seven() {
        // Arrange
        let expected_days = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        let expected_length = 7;

        // Act
        let actual_days = GregorianCalendar::WEEKDAYS;
        let actual_length = actual_days.len();

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_days, expected_days);
    }

    #[test]
    fn is_leap_year_returns_false_true_false_for_year_three_four_five() {
        // Arrange
        let year_before_boundary = 3;
        let year_boundary = 4;
        let year_after_boundary = 5;
        let expected_before_boundary = false;
        let expected_boundary = true;
        let expected_after_boundary = false;

        // Act
        let actual_before_boundary = GregorianCalendar::is_leap_year(year_before_boundary);
        let actual_boundary = GregorianCalendar::is_leap_year(year_boundary);
        let actual_after_boundary = GregorianCalendar::is_leap_year(year_after_boundary);

        // Assert
        assert_eq!(actual_before_boundary, expected_before_boundary);
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_after_boundary, expected_after_boundary);
    }

    #[test]
    fn is_leap_year_returns_all_false_for_year_ninety_nine_hundred_hundred_one() {
        // Arrange
        let year_before_boundary = 99;
        let year_boundary = 100;
        let year_after_boundary = 101;
        let expected_before_boundary = false;
        let expected_boundary = false;
        let expected_after_boundary = false;

        // Act
        let actual_before_boundary = GregorianCalendar::is_leap_year(year_before_boundary);
        let actual_boundary = GregorianCalendar::is_leap_year(year_boundary);
        let actual_after_boundary = GregorianCalendar::is_leap_year(year_after_boundary);

        // Assert
        assert_eq!(actual_before_boundary, expected_before_boundary);
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_after_boundary, expected_after_boundary);
    }

    #[test]
    fn is_leap_year_returns_false_true_false_for_year_three_ninety_nine_four_hundred_four_hundred_one(
    ) {
        // Arrange
        let year_before_boundary = 399;
        let year_boundary = 400;
        let year_after_boundary = 401;
        let expected_before_boundary = false;
        let expected_boundary = true;
        let expected_after_boundary = false;

        // Act
        let actual_before_boundary = GregorianCalendar::is_leap_year(year_before_boundary);
        let actual_boundary = GregorianCalendar::is_leap_year(year_boundary);
        let actual_after_boundary = GregorianCalendar::is_leap_year(year_after_boundary);

        // Assert
        assert_eq!(actual_before_boundary, expected_before_boundary);
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_after_boundary, expected_after_boundary);
    }

    #[test]
    fn day_name_returns_saturday_for_epoch() {
        // Arrange
        let day = 1;
        let month = 1;
        let year = 0;
        let expected_day_name = String::from("Saturday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_returns_monday_for_start_of_year_one() {
        // Arrange
        let day = 1;
        let month = 1;
        let year = 1;
        let expected_day_name = String::from("Monday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_advances_from_sunday_to_tuesday_across_year_zero_boundary() {
        // Arrange
        let dates = [(0, 12, 31), (1, 1, 1), (1, 1, 2)];
        let expected_day_names = [
            String::from("Sunday"),
            String::from("Monday"),
            String::from("Tuesday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_wednesday_to_friday_across_january_end_in_2024() {
        // Arrange
        let dates = [(2024, 1, 31), (2024, 2, 1), (2024, 2, 2)];
        let expected_day_names = [
            String::from("Wednesday"),
            String::from("Thursday"),
            String::from("Friday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_monday_to_wednesday_across_year_end_in_2024() {
        // Arrange
        let dates = [(2024, 12, 30), (2024, 12, 31), (2025, 1, 1)];
        let expected_day_names = [
            String::from("Monday"),
            String::from("Tuesday"),
            String::from("Wednesday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_monday_to_wednesday_across_common_year_february_end() {
        // Arrange
        let dates = [(2023, 2, 27), (2023, 2, 28), (2023, 3, 1)];
        let expected_day_names = [
            String::from("Monday"),
            String::from("Tuesday"),
            String::from("Wednesday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_wednesday_to_friday_across_leap_day_in_2024() {
        // Arrange
        let dates = [(2024, 2, 28), (2024, 2, 29), (2024, 3, 1)];
        let expected_day_names = [
            String::from("Wednesday"),
            String::from("Thursday"),
            String::from("Friday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_monday_to_wednesday_across_year_zero_leap_day() {
        // Arrange
        let dates = [(0, 2, 28), (0, 2, 29), (0, 3, 1)];
        let expected_day_names = [
            String::from("Monday"),
            String::from("Tuesday"),
            String::from("Wednesday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_saturday_to_monday_across_february_end_in_year_100() {
        // Arrange
        let dates = [(100, 2, 27), (100, 2, 28), (100, 3, 1)];
        let expected_day_names = [
            String::from("Saturday"),
            String::from("Sunday"),
            String::from("Monday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_advances_from_monday_to_wednesday_across_leap_day_in_year_400() {
        // Arrange
        let dates = [(400, 2, 28), (400, 2, 29), (400, 3, 1)];
        let expected_day_names = [
            String::from("Monday"),
            String::from("Tuesday"),
            String::from("Wednesday"),
        ];

        // Act
        let actual_day_names =
            dates.map(|(year, month, day)| GregorianCalendar::day_name(year, month, day));

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_returns_thursday_for_1970_01_01() {
        // Arrange
        let day = 1;
        let month = 1;
        let year = 1970;
        let expected_day_name = String::from("Thursday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_returns_friday_for_1999_12_31_in_year_month_day_order() {
        // Arrange
        let year = 1999;
        let month = 12;
        let day = 31;
        let expected_day_name = String::from("Friday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_returns_saturday_for_2000_01_01_in_year_month_day_order() {
        // Arrange
        let year = 2000;
        let month = 1;
        let day = 1;
        let expected_day_name = String::from("Saturday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_returns_sunday_for_2000_01_02_in_year_month_day_order() {
        // Arrange
        let year = 2000;
        let month = 1;
        let day = 2;
        let expected_day_name = String::from("Sunday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_returns_thursday_for_2024_02_29() {
        // Arrange
        let day = 29;
        let month = 2;
        let year = 2024;
        let expected_day_name = String::from("Thursday");

        // Act
        let actual_day_name = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_cycles_from_sunday_to_saturday_for_consecutive_seven_days_starting_2024_03_03() {
        // Arrange
        let year = 2024;
        let month = 3;
        let day_values = vec![3, 4, 5, 6, 7, 8, 9];
        let expected_day_names = vec![
            String::from("Sunday"),
            String::from("Monday"),
            String::from("Tuesday"),
            String::from("Wednesday"),
            String::from("Thursday"),
            String::from("Friday"),
            String::from("Saturday"),
        ];

        // Act
        let actual_day_names: Vec<String> = day_values
            .iter()
            .map(|&day| GregorianCalendar::day_name(year, month, day))
            .collect();

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_returns_same_string_for_repeated_calls_with_same_input() {
        // Arrange
        let day = 15;
        let month = 8;
        let year = 2025;
        let expected_day_name = String::from("Friday");

        // Act
        let actual_day_name_first = GregorianCalendar::day_name(year, month, day);
        let actual_day_name_second = GregorianCalendar::day_name(year, month, day);

        // Assert
        assert_eq!(actual_day_name_first, expected_day_name);
        assert_eq!(actual_day_name_second, expected_day_name);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_epoch() {
        // Arrange
        let year = 0;
        let month = 1;
        let day = 1;
        let expected_index = 0;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_day_after_epoch() {
        // Arrange
        let year = 0;
        let month = 1;
        let day = 2;
        let expected_index = 1;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_end_of_january() {
        // Arrange
        let year = 0;
        let month = 1;
        let day = 31;
        let expected_index = 30;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_start_of_february() {
        // Arrange
        let year = 0;
        let month = 2;
        let day = 1;
        let expected_index = 31;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_year_zero_leap_day() {
        // Arrange
        let year = 0;
        let month = 2;
        let day = 29;
        let expected_index = 59;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_march_in_year_zero() {
        // Arrange
        let year = 0;
        let month = 3;
        let day = 1;
        let expected_index = 60;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_end_of_year_zero() {
        // Arrange
        let year = 0;
        let month = 12;
        let day = 31;
        let expected_index = 365;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_start_of_year_one() {
        // Arrange
        let year = 1;
        let month = 1;
        let day = 1;
        let expected_index = 366;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_march_in_year_one() {
        // Arrange
        let year = 1;
        let month = 3;
        let day = 1;
        let expected_index = 425;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_before_century_march() {
        // Arrange
        let year = 100;
        let month = 2;
        let day = 28;
        let expected_index = 36583;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_century_march() {
        // Arrange
        let year = 100;
        let month = 3;
        let day = 1;
        let expected_index = 36584;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_four_hundred_leap_day() {
        // Arrange
        let year = 400;
        let month = 2;
        let day = 29;
        let expected_index = 146156;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn day_index_returns_elapsed_days_for_four_hundred_march() {
        // Arrange
        let year = 400;
        let month = 3;
        let day = 1;
        let expected_index = 146157;

        // Act
        let actual_index = GregorianCalendar::day_index(year, month, day);

        // Assert
        assert_eq!(actual_index, expected_index);
    }
}
