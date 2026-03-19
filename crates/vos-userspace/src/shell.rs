//! VOS Shell - Interactive command-line interface.
//!
//! Provides a REPL (Read-Eval-Print Loop) for user interaction with the OS.

use vos_core::Result;
use vos_kernel::{OpenMode, Vfs};

/// Shell state and configuration.
pub struct Shell {
    /// Virtual filesystem
    vfs: Vfs,

    /// Command history
    history: Vec<String>,

    /// Exit flag
    should_exit: bool,
}

impl Shell {
    /// Creates a new shell with a fresh VFS.
    pub fn new() -> Self {
        Self {
            vfs: Vfs::new(),
            history: Vec::new(),
            should_exit: false,
        }
    }

    /// Creates a new shell with an existing VFS.
    pub fn with_vfs(vfs: Vfs) -> Self {
        Self {
            vfs,
            history: Vec::new(),
            should_exit: false,
        }
    }

    /// Returns true if the shell should exit.
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    /// Gets the current working directory inode.
    pub fn cwd(&self) -> u32 {
        self.vfs.getcwd()
    }

    /// Returns reference to the VFS.
    pub fn vfs(&self) -> &Vfs {
        &self.vfs
    }

    /// Returns mutable reference to the VFS.
    pub fn vfs_mut(&mut self) -> &mut Vfs {
        &mut self.vfs
    }

    /// Executes a command line.
    pub fn execute(&mut self, line: &str) -> Result<String> {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            return Ok(String::new());
        }

        // Add to history
        self.history.push(line.to_string());

        // Parse command
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        let command = parts[0];
        let args = &parts[1..];

        // Execute built-in commands
        match command {
            "exit" => self.cmd_exit(args),
            "pwd" => self.cmd_pwd(args),
            "cd" => self.cmd_cd(args),
            "ls" => self.cmd_ls(args),
            "cat" => self.cmd_cat(args),
            "echo" => self.cmd_echo(args),
            "mkdir" => self.cmd_mkdir(args),
            "touch" => self.cmd_touch(args),
            "rm" => self.cmd_rm(args),
            "help" => self.cmd_help(args),
            _ => Ok(format!("vos: command not found: {}", command)),
        }
    }

    /// Exit the shell.
    fn cmd_exit(&mut self, _args: &[&str]) -> Result<String> {
        self.should_exit = true;
        Ok("Goodbye!".to_string())
    }

    /// Print working directory.
    fn cmd_pwd(&self, _args: &[&str]) -> Result<String> {
        // For now, just return the inode number
        // In a real implementation, we'd track the path string
        Ok(format!("/ (inode {})", self.vfs.getcwd()))
    }

    /// Change directory.
    fn cmd_cd(&mut self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            // Go to root
            self.vfs.chdir("/")?;
        } else {
            self.vfs.chdir(args[0])?;
        }
        Ok(String::new())
    }

    /// List directory contents.
    fn cmd_ls(&self, args: &[&str]) -> Result<String> {
        let path = if args.is_empty() { "/" } else { args[0] };

        let entries = self.vfs.readdir(path)?;

        let mut output = String::new();
        for (name, inode) in entries {
            // Skip . and .. for cleaner output
            if name != "." && name != ".." {
                output.push_str(&format!("{}  (inode {})\n", name, inode));
            }
        }

        Ok(output)
    }

    /// Display file contents.
    fn cmd_cat(&mut self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Ok("cat: missing file operand".to_string());
        }

        let path = args[0];

        // Open file
        let fd = self.vfs.open(path, OpenMode::ReadOnly)?;

        // Read contents
        let mut buffer = vec![0u8; 4096];
        let bytes_read = self.vfs.read(fd, &mut buffer)?;

        // Close file
        self.vfs.close(fd)?;

        // Convert to string
        let content = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

        Ok(content)
    }

    /// Echo arguments.
    fn cmd_echo(&self, args: &[&str]) -> Result<String> {
        Ok(args.join(" "))
    }

    /// Create directory.
    fn cmd_mkdir(&mut self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Ok("mkdir: missing operand".to_string());
        }

        for path in args {
            self.vfs.mkdir(path)?;
        }

        Ok(String::new())
    }

    /// Create empty file.
    fn cmd_touch(&mut self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Ok("touch: missing file operand".to_string());
        }

        for path in args {
            self.vfs.create(path)?;
        }

        Ok(String::new())
    }

    /// Remove file or directory.
    fn cmd_rm(&mut self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Ok("rm: missing operand".to_string());
        }

        for path in args {
            self.vfs.unlink(path)?;
        }

        Ok(String::new())
    }

    /// Display help.
    fn cmd_help(&self, _args: &[&str]) -> Result<String> {
        let help = r#"VOS Shell - Available Commands:

  help         Display this help message
  exit         Exit the shell
  pwd          Print working directory
  cd [PATH]    Change directory (default: /)
  ls [PATH]    List directory contents (default: /)
  cat FILE     Display file contents
  echo [ARGS]  Echo arguments to output
  mkdir PATH   Create directory
  touch FILE   Create empty file
  rm PATH      Remove file or empty directory

Examples:
  mkdir /mydir         Create a directory
  touch /mydir/file    Create a file
  echo hello > file    Write to file (not implemented yet)
  cat /mydir/file      Display file contents
  ls /mydir            List directory
  cd /mydir            Change to directory
  pwd                  Show current directory
  rm /mydir/file       Remove file
"#;
        Ok(help.to_string())
    }

    /// Returns command history.
    pub fn history(&self) -> &[String] {
        &self.history
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert_eq!(shell.cwd(), 1); // Root inode
        assert!(!shell.should_exit());
    }

    #[test]
    fn test_echo() {
        let mut shell = Shell::new();
        let output = shell.execute("echo hello world").unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_help() {
        let mut shell = Shell::new();
        let output = shell.execute("help").unwrap();
        assert!(output.contains("VOS Shell"));
        assert!(output.contains("help"));
        assert!(output.contains("exit"));
    }

    #[test]
    fn test_pwd() {
        let shell = Shell::new();
        let output = shell.cmd_pwd(&[]).unwrap();
        assert!(output.contains("inode 1"));
    }

    #[test]
    fn test_mkdir_ls() {
        let mut shell = Shell::new();

        // Create directory
        shell.execute("mkdir /testdir").unwrap();

        // List root
        let output = shell.execute("ls /").unwrap();
        assert!(output.contains("testdir"));
    }

    #[test]
    fn test_touch_cat() {
        let mut shell = Shell::new();

        // Create file
        shell.execute("touch /test.txt").unwrap();

        // Write to file manually (cat can't write yet)
        let fd = shell.vfs_mut().open("/test.txt", OpenMode::WriteOnly).unwrap();
        shell.vfs_mut().write(fd, b"Hello, Shell!").unwrap();
        shell.vfs_mut().close(fd).unwrap();

        // Cat file
        let output = shell.execute("cat /test.txt").unwrap();
        assert_eq!(output, "Hello, Shell!");
    }

    #[test]
    fn test_cd() {
        let mut shell = Shell::new();

        // Create directory
        shell.execute("mkdir /mydir").unwrap();

        // Change to it
        shell.execute("cd /mydir").unwrap();

        // Should be different inode now
        assert_ne!(shell.cwd(), 1);

        // Change back to root
        shell.execute("cd /").unwrap();
        assert_eq!(shell.cwd(), 1);
    }

    #[test]
    fn test_rm() {
        let mut shell = Shell::new();

        // Create and remove file
        shell.execute("touch /deleteme.txt").unwrap();
        shell.execute("rm /deleteme.txt").unwrap();

        // Should not be in listing
        let output = shell.execute("ls /").unwrap();
        assert!(!output.contains("deleteme.txt"));
    }

    #[test]
    fn test_exit() {
        let mut shell = Shell::new();
        assert!(!shell.should_exit());

        shell.execute("exit").unwrap();
        assert!(shell.should_exit());
    }

    #[test]
    fn test_unknown_command() {
        let mut shell = Shell::new();
        let output = shell.execute("notacommand").unwrap();
        assert!(output.contains("command not found"));
    }

    #[test]
    fn test_empty_line() {
        let mut shell = Shell::new();
        let output = shell.execute("").unwrap();
        assert_eq!(output, "");

        let output = shell.execute("   ").unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn test_history() {
        let mut shell = Shell::new();

        shell.execute("echo test1").unwrap();
        shell.execute("echo test2").unwrap();

        let history = shell.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "echo test1");
        assert_eq!(history[1], "echo test2");
    }

    #[test]
    fn test_multiple_args() {
        let mut shell = Shell::new();

        // Multiple mkdir
        shell.execute("mkdir /dir1 /dir2 /dir3").unwrap();

        let output = shell.execute("ls /").unwrap();
        assert!(output.contains("dir1"));
        assert!(output.contains("dir2"));
        assert!(output.contains("dir3"));
    }
}
