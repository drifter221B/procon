struct Days {
    days: Vec<&'static str>,
}

impl Days {

    fn new() -> Self {
        Days {
            days: vec!["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
        }
    }

    fn is_leap_year(&self, year: usize) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    pub fn calculate_day_of_week(&self, day: usize, month: usize, year: usize) -> usize {

        // 0001/01/00 was a Sunday
        let mut x = 0;

        for y in 1..year {

            x += 365 + if self.is_leap_year(y) {
                1
            } else {
                0
            };
        }

        for m in 1..month {

            x += if m == 2 {
                if self.is_leap_year(year) {
                    29
                } else {
                    28
                }
            } else if (m & 1 == 1) ^ (8 <= m) {
                31
            } else {
                30
            };
        }

        x += day;

        x % 7
    }

    pub fn day_name(&self, day: usize, month: usize, year: usize) -> String {
        let day_index = self.calculate_day_of_week(day, month, year);

        self.days[day_index].to_string()
    }

}

#[cfg(test)]
mod tests {
    use super::Days;

    fn create_days_instance() -> Days {
        Days::new()
    }

    #[test]
    fn days_new_creates_weekday_list_from_sunday_to_saturday_with_length_seven() {
        // Arrange
        let expected_days = vec![
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ];
        let expected_length = 7;

        // Act
        let actual_days_instance = Days::new();
        let actual_days = actual_days_instance.days;
        let actual_length = actual_days.len();

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_days, expected_days);
    }

    #[test]
    fn is_leap_year_returns_false_true_false_for_year_three_four_five() {
        // Arrange
        let days_instance = create_days_instance();
        let year_before_boundary = 3;
        let year_boundary = 4;
        let year_after_boundary = 5;
        let expected_before_boundary = false;
        let expected_boundary = true;
        let expected_after_boundary = false;

        // Act
        let actual_before_boundary = days_instance.is_leap_year(year_before_boundary);
        let actual_boundary = days_instance.is_leap_year(year_boundary);
        let actual_after_boundary = days_instance.is_leap_year(year_after_boundary);

        // Assert
        assert_eq!(actual_before_boundary, expected_before_boundary);
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_after_boundary, expected_after_boundary);
    }

    #[test]
    fn is_leap_year_returns_all_false_for_year_ninety_nine_hundred_hundred_one() {
        // Arrange
        let days_instance = create_days_instance();
        let year_before_boundary = 99;
        let year_boundary = 100;
        let year_after_boundary = 101;
        let expected_before_boundary = false;
        let expected_boundary = false;
        let expected_after_boundary = false;

        // Act
        let actual_before_boundary = days_instance.is_leap_year(year_before_boundary);
        let actual_boundary = days_instance.is_leap_year(year_boundary);
        let actual_after_boundary = days_instance.is_leap_year(year_after_boundary);

        // Assert
        assert_eq!(actual_before_boundary, expected_before_boundary);
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_after_boundary, expected_after_boundary);
    }

    #[test]
    fn is_leap_year_returns_false_true_false_for_year_three_ninety_nine_four_hundred_four_hundred_one(
    ) {
        // Arrange
        let days_instance = create_days_instance();
        let year_before_boundary = 399;
        let year_boundary = 400;
        let year_after_boundary = 401;
        let expected_before_boundary = false;
        let expected_boundary = true;
        let expected_after_boundary = false;

        // Act
        let actual_before_boundary = days_instance.is_leap_year(year_before_boundary);
        let actual_boundary = days_instance.is_leap_year(year_boundary);
        let actual_after_boundary = days_instance.is_leap_year(year_after_boundary);

        // Assert
        assert_eq!(actual_before_boundary, expected_before_boundary);
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_after_boundary, expected_after_boundary);
    }

    #[test]
    fn calculate_day_of_week_returns_zero_for_day_zero_month_one_year_one() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 0;
        let month = 1;
        let year = 1;
        let expected_day_index = 0;

        // Act
        let actual_day_index = days_instance.calculate_day_of_week(day, month, year);

        // Assert
        assert_eq!(actual_day_index, expected_day_index);
    }

    #[test]
    fn calculate_day_of_week_returns_one_for_day_one_month_one_year_one() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 1;
        let month = 1;
        let year = 1;
        let expected_day_index = 1;

        // Act
        let actual_day_index = days_instance.calculate_day_of_week(day, month, year);

        // Assert
        assert_eq!(actual_day_index, expected_day_index);
    }

    #[test]
    fn calculate_day_of_week_advances_one_by_one_for_day_zero_one_two_at_month_one_year_one() {
        // Arrange
        let days_instance = create_days_instance();
        let day_before_boundary = 0;
        let day_boundary = 1;
        let day_after_boundary = 2;
        let month = 1;
        let year = 1;
        let expected_day_index_before_boundary = 0;
        let expected_day_index_boundary = 1;
        let expected_day_index_after_boundary = 2;

        // Act
        let actual_day_index_before_boundary =
            days_instance.calculate_day_of_week(day_before_boundary, month, year);
        let actual_day_index_boundary = days_instance.calculate_day_of_week(day_boundary, month, year);
        let actual_day_index_after_boundary =
            days_instance.calculate_day_of_week(day_after_boundary, month, year);

        // Assert
        assert_eq!(
            actual_day_index_before_boundary,
            expected_day_index_before_boundary
        );
        assert_eq!(actual_day_index_boundary, expected_day_index_boundary);
        assert_eq!(actual_day_index_after_boundary, expected_day_index_after_boundary);
    }

    #[test]
    fn calculate_day_of_week_preserves_current_behavior_for_month_zero_one_two_with_day_one_in_2024() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 1;
        let year = 2024;
        let month_before_boundary = 0;
        let month_boundary = 1;
        let month_after_boundary = 2;
        let expected_day_index_before_boundary = 1;
        let expected_day_index_boundary = 1;
        let expected_day_index_after_boundary = 4;

        // Act
        let actual_day_index_before_boundary =
            days_instance.calculate_day_of_week(day, month_before_boundary, year);
        let actual_day_index_boundary = days_instance.calculate_day_of_week(day, month_boundary, year);
        let actual_day_index_after_boundary =
            days_instance.calculate_day_of_week(day, month_after_boundary, year);

        // Assert
        assert_eq!(
            actual_day_index_before_boundary,
            expected_day_index_before_boundary
        );
        assert_eq!(actual_day_index_boundary, expected_day_index_boundary);
        assert_eq!(actual_day_index_after_boundary, expected_day_index_after_boundary);
    }

    #[test]
    fn calculate_day_of_week_preserves_current_behavior_for_month_eleven_twelve_thirteen_with_day_one_in_2024(
    ) {
        // Arrange
        let days_instance = create_days_instance();
        let day = 1;
        let year = 2024;
        let month_before_boundary = 11;
        let month_boundary = 12;
        let month_after_boundary = 13;
        let expected_day_index_before_boundary = 5;
        let expected_day_index_boundary = 0;
        let expected_day_index_after_boundary = 3;

        // Act
        let actual_day_index_before_boundary =
            days_instance.calculate_day_of_week(day, month_before_boundary, year);
        let actual_day_index_boundary = days_instance.calculate_day_of_week(day, month_boundary, year);
        let actual_day_index_after_boundary =
            days_instance.calculate_day_of_week(day, month_after_boundary, year);

        // Assert
        assert_eq!(
            actual_day_index_before_boundary,
            expected_day_index_before_boundary
        );
        assert_eq!(actual_day_index_boundary, expected_day_index_boundary);
        assert_eq!(actual_day_index_after_boundary, expected_day_index_after_boundary);
    }

    #[test]
    fn calculate_day_of_week_preserves_transition_for_february_day_twenty_seven_twenty_eight_twenty_nine_in_common_year(
    ) {
        // Arrange
        let days_instance = create_days_instance();
        let month = 2;
        let year = 2023;
        let day_before_boundary = 27;
        let day_boundary = 28;
        let day_after_boundary = 29;
        let expected_day_index_before_boundary = 1;
        let expected_day_index_boundary = 2;
        let expected_day_index_after_boundary = 3;

        // Act
        let actual_day_index_before_boundary =
            days_instance.calculate_day_of_week(day_before_boundary, month, year);
        let actual_day_index_boundary = days_instance.calculate_day_of_week(day_boundary, month, year);
        let actual_day_index_after_boundary =
            days_instance.calculate_day_of_week(day_after_boundary, month, year);

        // Assert
        assert_eq!(
            actual_day_index_before_boundary,
            expected_day_index_before_boundary
        );
        assert_eq!(actual_day_index_boundary, expected_day_index_boundary);
        assert_eq!(actual_day_index_after_boundary, expected_day_index_after_boundary);
    }

    #[test]
    fn calculate_day_of_week_preserves_transition_for_february_day_twenty_eight_twenty_nine_thirty_in_leap_year(
    ) {
        // Arrange
        let days_instance = create_days_instance();
        let month = 2;
        let year = 2024;
        let day_before_boundary = 28;
        let day_boundary = 29;
        let day_after_boundary = 30;
        let expected_day_index_before_boundary = 3;
        let expected_day_index_boundary = 4;
        let expected_day_index_after_boundary = 5;

        // Act
        let actual_day_index_before_boundary =
            days_instance.calculate_day_of_week(day_before_boundary, month, year);
        let actual_day_index_boundary = days_instance.calculate_day_of_week(day_boundary, month, year);
        let actual_day_index_after_boundary =
            days_instance.calculate_day_of_week(day_after_boundary, month, year);

        // Assert
        assert_eq!(
            actual_day_index_before_boundary,
            expected_day_index_before_boundary
        );
        assert_eq!(actual_day_index_boundary, expected_day_index_boundary);
        assert_eq!(actual_day_index_after_boundary, expected_day_index_after_boundary);
    }

    #[test]
    fn calculate_day_of_week_keeps_consecutive_relation_between_last_day_of_1999_and_first_day_of_2000() {
        // Arrange
        let days_instance = create_days_instance();
        let previous_day = 31;
        let previous_month = 12;
        let previous_year = 1999;
        let next_day = 1;
        let next_month = 1;
        let next_year = 2000;
        let weekday_cycle = 7;
        let expected_day_difference = 1;

        // Act
        let actual_previous_day_index =
            days_instance.calculate_day_of_week(previous_day, previous_month, previous_year);
        let actual_next_day_index = days_instance.calculate_day_of_week(next_day, next_month, next_year);
        let actual_day_difference =
            (actual_next_day_index + weekday_cycle - actual_previous_day_index) % weekday_cycle;

        // Assert
        assert_eq!(actual_day_difference, expected_day_difference);
    }

    #[test]
    fn calculate_day_of_week_returns_thursday_index_for_1970_01_01() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 1;
        let month = 1;
        let year = 1970;
        let expected_day_index = 4;

        // Act
        let actual_day_index = days_instance.calculate_day_of_week(day, month, year);

        // Assert
        assert_eq!(actual_day_index, expected_day_index);
    }

    #[test]
    fn calculate_day_of_week_returns_saturday_index_for_2000_01_01() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 1;
        let month = 1;
        let year = 2000;
        let expected_day_index = 6;

        // Act
        let actual_day_index = days_instance.calculate_day_of_week(day, month, year);

        // Assert
        assert_eq!(actual_day_index, expected_day_index);
    }

    #[test]
    fn calculate_day_of_week_returns_thursday_index_for_2024_02_29() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 29;
        let month = 2;
        let year = 2024;
        let expected_day_index = 4;

        // Act
        let actual_day_index = days_instance.calculate_day_of_week(day, month, year);

        // Assert
        assert_eq!(actual_day_index, expected_day_index);
    }

    #[test]
    fn day_name_returns_thursday_for_2024_02_29() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 29;
        let month = 2;
        let year = 2024;
        let expected_day_name = String::from("Thursday");

        // Act
        let actual_day_name = days_instance.day_name(day, month, year);

        // Assert
        assert_eq!(actual_day_name, expected_day_name);
    }

    #[test]
    fn day_name_cycles_from_sunday_to_saturday_for_consecutive_seven_days_starting_2024_03_03() {
        // Arrange
        let days_instance = create_days_instance();
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
            .map(|&day| days_instance.day_name(day, month, year))
            .collect();

        // Assert
        assert_eq!(actual_day_names, expected_day_names);
    }

    #[test]
    fn day_name_returns_same_string_for_repeated_calls_with_same_input() {
        // Arrange
        let days_instance = create_days_instance();
        let day = 15;
        let month = 8;
        let year = 2025;
        let expected_day_name = String::from("Friday");

        // Act
        let actual_day_name_first = days_instance.day_name(day, month, year);
        let actual_day_name_second = days_instance.day_name(day, month, year);

        // Assert
        assert_eq!(actual_day_name_first, expected_day_name);
        assert_eq!(actual_day_name_second, expected_day_name);
    }
}
