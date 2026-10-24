use std::io::{BufRead, Write};

fn read(file: &std::path::Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(file)
}

fn write<T: AsRef<[u8]>>(file: &std::path::Path, content: T) -> Result<usize, std::io::Error> {
    let content_bytes = content.as_ref();
    std::fs::write(file, content_bytes)?;
    Ok(content_bytes.len())
}

fn append(file: &std::path::Path, content: &[u8]) -> Result<(), std::io::Error> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?;

    file.write_all(content)
}

fn stdin() -> Box<dyn std::io::BufRead> {
    Box::new(std::io::BufReader::new(std::io::stdin()))
}

fn open(file_name: &str) -> Result<Box<dyn std::io::BufRead>, std::io::Error> {
    Ok(Box::new(std::io::BufReader::new(std::fs::File::open(file_name)?)))
}

#[cfg(test)]
mod tests {
    use super::{append, open, read, write};
    use std::io::BufRead;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(prefix: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "rust_io_test_{}_{}_{}",
            prefix,
            std::process::id(),
            unique_suffix
        ))
    }

    #[test]
    fn read_returns_err_for_nonexistent_file() {
        // Arrange
        let file_path = temp_file_path("missing");

        // Act
        let actual_result = read(&file_path);

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn append_keeps_existing_content_and_appends_to_end() {
        // Arrange
        let file_path = temp_file_path("append");
        let initial_content = "hello";
        let append_content = b" world";
        let expected_content = "hello world";
        write(&file_path, initial_content).expect("failed to write initial content");

        // Act
        let actual_append_result = append(&file_path, append_content);
        let actual_content = read(&file_path).expect("failed to read appended content");

        // Assert
        assert!(actual_append_result.is_ok());
        assert_eq!(actual_content, expected_content);

        std::fs::remove_file(file_path).expect("failed to remove temp file");
    }

    #[test]
    fn open_reads_multiple_lines_in_order() {
        // Arrange
        let file_path = temp_file_path("open");
        let initial_content = "alpha\nbeta\n";
        let expected_lines = vec!["alpha".to_string(), "beta".to_string()];
        write(&file_path, initial_content).expect("failed to write lines");
        let file_name = file_path.to_string_lossy().to_string();

        // Act
        let reader = open(&file_name).expect("failed to open file");
        let actual_lines = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .expect("failed to read lines");

        // Assert
        assert_eq!(actual_lines, expected_lines);

        std::fs::remove_file(file_path).expect("failed to remove temp file");
    }

    #[test]
    fn write_with_bytes_returns_written_byte_count() {
        // Arrange
        let file_path = temp_file_path("write_bytes");
        let content = b"abc";
        let expected_written_bytes = 3;

        // Act
        let actual_written_bytes = write(&file_path, content.as_slice()).expect("failed to write bytes");
        let actual_content = std::fs::read(&file_path).expect("failed to read bytes");

        // Assert
        assert_eq!(actual_written_bytes, expected_written_bytes);
        assert_eq!(actual_content, content);

        std::fs::remove_file(file_path).expect("failed to remove temp file");
    }

    #[test]
    fn append_creates_missing_file_with_content() {
        // Arrange
        let file_path = temp_file_path("append_missing");
        let append_content = b"created";
        let expected_content = "created";

        // Act
        let actual_append_result = append(&file_path, append_content);
        let actual_content = read(&file_path).expect("failed to read created file");

        // Assert
        assert!(actual_append_result.is_ok());
        assert_eq!(actual_content, expected_content);

        std::fs::remove_file(file_path).expect("failed to remove temp file");
    }

    #[test]
    fn write_empty_content_creates_empty_file_and_returns_zero() {
        // Arrange
        let file_path = temp_file_path("write_empty");
        let content: [u8; 0] = [];
        let expected_written_bytes = 0;
        let expected_content: [u8; 0] = [];

        // Act
        let actual_written_bytes = write(&file_path, content).expect("failed to write empty content");
        let actual_content = std::fs::read(&file_path).expect("failed to read empty file");

        // Assert
        assert_eq!(actual_written_bytes, expected_written_bytes);
        assert_eq!(actual_content, expected_content);

        std::fs::remove_file(file_path).expect("failed to remove temp file");
    }
}
