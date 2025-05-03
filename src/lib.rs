#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use std::error::Error;

    #[test]
    fn test_init_command() -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::cargo_bin("icloud2hugo")?;
        let output = cmd.arg("init").assert().success();
        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(stdout.contains("Initializing config"));
        Ok(())
    }

    #[test]
    fn test_sync_command() -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::cargo_bin("icloud2hugo")?;
        let output = cmd.arg("sync").assert().success();
        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(stdout.contains("Syncing photos"));
        Ok(())
    }

    #[test]
    fn test_status_command() -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::cargo_bin("icloud2hugo")?;
        let output = cmd.arg("status").assert().success();
        let stdout = String::from_utf8(output.get_output().stdout.clone())?;
        assert!(stdout.contains("Checking status"));
        Ok(())
    }
}